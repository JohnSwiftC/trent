use std::io;
use tokio::net::TcpStream;

mod download;
mod standard;

use standard::ClientRoute;

pub async fn start_client() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:6969").await.unwrap();

    standard::action(
        &mut stream,
        ClientRoute::DownloadFile {
            name: "dogdog".to_owned(),
            save_name: "newdogdog.jpg".to_owned(),
        },
    )
    .await?;

    Ok(())
}
