use std::io;
use tokio::net::TcpStream;

mod download;
mod getfiles;
mod standard;

use crate::ClientArgs;
use standard::ClientRoute;

pub async fn start_client(args: ClientArgs) -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:6969").await.unwrap();

    standard::action(&mut stream, ClientRoute::GetFiles).await?;

    Ok(())
}
