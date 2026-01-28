use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn is_alive(stream: &mut TcpStream) -> io::Result<()> {
    let res = stream.read_u32().await?;

    if res != 80085 {
        Err(io::Error::other("Failed to return proper server value"))
    } else {
        Ok(())
    }
}
