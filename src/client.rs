use std::io;
use tokio::net::TcpStream;

use anyhow;

mod download;
mod getfiles;
mod standard;

use crate::{ClientAction, ClientArgs, DownloadArgs};
use standard::ClientRoute;

pub async fn start_client(args: ClientArgs) -> anyhow::Result<()> {
    let mut stream = TcpStream::connect(args.addr).await?;

    match args.action {
        ClientAction::Download(DownloadArgs { file, output }) => {
            standard::action(
                &mut stream,
                ClientRoute::DownloadFile {
                    name: file,
                    save_name: output,
                },
            )
            .await?;
        }
        ClientAction::GetFiles => standard::action(&mut stream, ClientRoute::GetFiles).await?,
    }

    Ok(())
}
