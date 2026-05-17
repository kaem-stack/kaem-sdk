use bytes::{Buf, BufMut, BytesMut};
use serde::{Serialize, de::DeserializeOwned};
use std::marker::PhantomData;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio_util::codec::{Decoder, Encoder, Framed};

pub struct IpcCodec<T> {
    _phantom: PhantomData<T>,
}

impl<T> IpcCodec<T> {
    pub fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<T> Default for IpcCodec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Serialize> Encoder<T> for IpcCodec<T> {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes = bincode::serialize(&item)?;
        dst.reserve(4 + bytes.len());
        dst.put_u32(bytes.len() as u32);
        dst.extend_from_slice(&bytes);
        Ok(())
    }
}

impl<T: DeserializeOwned> Decoder for IpcCodec<T> {
    type Item = T;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<T>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }
        let len = u32::from_be_bytes(src[..4].try_into().unwrap()) as usize;
        if src.len() < 4 + len {
            src.reserve(4 + len - src.len());
            return Ok(None);
        }
        src.advance(4);
        let data = src.split_to(len);
        Ok(Some(bincode::deserialize(&data)?))
    }
}

pub fn framed<T>(stream: UnixStream) -> Framed<UnixStream, IpcCodec<T>> {
    Framed::new(stream, IpcCodec::new())
}

pub async fn send<T: Serialize>(
    stream: &mut UnixStream,
    msg: &T,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bytes = bincode::serialize(msg)?;
    let len = bytes.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&bytes).await?;
    Ok(())
}

pub async fn receive<T: DeserializeOwned>(
    stream: &mut UnixStream,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    Ok(bincode::deserialize(&buf)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn codec_encodes_length_prefix() {
        let mut codec = IpcCodec::<String>::new();
        let mut buf = BytesMut::new();

        codec.encode("hi".to_string(), &mut buf).unwrap();

        let payload = bincode::serialize("hi").unwrap();
        let expected_len = payload.len() as u32;
        assert_eq!(&buf[..4], &expected_len.to_be_bytes());
        assert_eq!(&buf[4..], payload.as_slice());
    }

    #[test]
    fn codec_round_trip() {
        let mut codec = IpcCodec::<String>::new();
        let mut buf = BytesMut::new();

        codec.encode("hello ipc".to_string(), &mut buf).unwrap();
        let result = codec.decode(&mut buf).unwrap();

        assert_eq!(result, Some("hello ipc".to_string()));
        assert!(buf.is_empty());
    }

    #[test]
    fn codec_decode_waits_for_header() {
        let mut codec = IpcCodec::<String>::new();
        let mut buf = BytesMut::from(&[0u8, 0u8][..]); // only 2 of 4 header bytes

        assert_eq!(codec.decode(&mut buf).unwrap(), None);
    }

    #[test]
    fn codec_decode_waits_for_full_payload() {
        let mut codec = IpcCodec::<String>::new();
        let mut buf = BytesMut::new();
        buf.put_u32(100); // claims 100 bytes of payload
        buf.extend_from_slice(&[0u8; 10]); // only 10 bytes present

        assert_eq!(codec.decode(&mut buf).unwrap(), None);
    }

    #[test]
    fn codec_decode_multiple_frames() {
        let mut codec = IpcCodec::<u32>::new();
        let mut buf = BytesMut::new();

        codec.encode(1u32, &mut buf).unwrap();
        codec.encode(2u32, &mut buf).unwrap();
        codec.encode(3u32, &mut buf).unwrap();

        assert_eq!(codec.decode(&mut buf).unwrap(), Some(1u32));
        assert_eq!(codec.decode(&mut buf).unwrap(), Some(2u32));
        assert_eq!(codec.decode(&mut buf).unwrap(), Some(3u32));
        assert_eq!(codec.decode(&mut buf).unwrap(), None);
    }

    #[tokio::test]
    async fn send_receive_round_trip() {
        let (mut a, mut b) = UnixStream::pair().unwrap();

        send(&mut a, &"hello world".to_string()).await.unwrap();
        let received: String = receive(&mut b).await.unwrap();

        assert_eq!(received, "hello world");
    }

    #[tokio::test]
    async fn send_receive_binary_payload() {
        let (mut a, mut b) = UnixStream::pair().unwrap();
        let payload: Vec<u8> = (0u8..=255).collect();

        send(&mut a, &payload).await.unwrap();
        let received: Vec<u8> = receive(&mut b).await.unwrap();

        assert_eq!(received, payload);
    }

    #[tokio::test]
    async fn send_command_receive_event_pattern() {
        let (mut a, mut b) = UnixStream::pair().unwrap();

        // a sends a command, b processes and sends back an event
        tokio::join!(
            async {
                send(&mut a, &10u64).await.unwrap();
                let event: u64 = receive(&mut a).await.unwrap();
                assert_eq!(event, 20u64);
            },
            async {
                let cmd: u64 = receive(&mut b).await.unwrap();
                send(&mut b, &(cmd * 2)).await.unwrap();
            }
        );
    }

    #[tokio::test]
    async fn send_multiple_messages_in_sequence() {
        let (mut a, mut b) = UnixStream::pair().unwrap();

        for i in 0u32..5 {
            send(&mut a, &i).await.unwrap();
        }
        for expected in 0u32..5 {
            let received: u32 = receive(&mut b).await.unwrap();
            assert_eq!(received, expected);
        }
    }
}
