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
