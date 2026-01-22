use crate::TrentFile;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn get_files(stream: &mut TcpStream) -> io::Result<()> {
    Ok(())
}
