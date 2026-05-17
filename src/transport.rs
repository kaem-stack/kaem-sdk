use serde::{Serialize, de::DeserializeOwned};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub async fn send_message<T: Serialize>(
    stream: &mut UnixStream,
    msg: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = bincode::serialize(msg)?;
    let len = bytes.len() as u32;

    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&bytes).await?;

    Ok(())
}

pub async fn receive_message<T: DeserializeOwned>(
    stream: &mut UnixStream,
) -> Result<T, Box<dyn std::error::Error>> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;

    Ok(bincode::deserialize(&buf)?)
}
