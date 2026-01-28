use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn is_alive(stream: &mut TcpStream) -> io::Result<()> {
    stream.write_u32(80085).await?;

    Ok(())
}
