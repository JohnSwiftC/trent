use crate::ServerData;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn get_files(stream: &mut TcpStream, server_data: &'static ServerData) -> io::Result<()> {
    stream
        .write_u32(server_data.file_information().len() as u32)
        .await?;
    stream.write_all(server_data.file_information()).await?;

    Ok(())
}
