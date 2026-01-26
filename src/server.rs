use anyhow;

use crate::SERVER_DATA;
use crate::TrentFile;
use tokio::net::{TcpListener, TcpStream};
pub mod config;
mod getfiles;
mod standard;
mod upload;

use crate::ServerArgs;
use config::ServerData;
use standard::ServerRoute;

/// This function creates a bunch
pub async fn start_server(args: ServerArgs) -> anyhow::Result<()> {
    let files = vec![
        TrentFile::from_path_zstd_mmap("dogdog.jpg", 15, String::from("dogdog"), 6)?,
        TrentFile::from_path_mmap("testvideo.mp4", 40, "newvideoblahblah".to_owned())?,
    ];
    set_server_data(ServerData::from_files(files))?;
    let server_data = get_server_data()?;

    let listener = TcpListener::bind(args.bind).await?;

    while let Ok((stream, _)) = listener.accept().await {
        let _task = tokio::task::spawn(async move {
            if let Err(e) = handle(stream, server_data).await {
                eprintln!("Handle error: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle(mut stream: TcpStream, server_data: &'static ServerData) -> anyhow::Result<()> {
    match standard::route(&mut stream).await? {
        ServerRoute::Upload => upload::upload_file(stream, server_data.get_files()).await?,
        ServerRoute::GetFiles => getfiles::get_files(&mut stream, server_data).await?,
        ServerRoute::Unknown => todo!(),
    }

    Ok(())
}

fn set_server_data(server_data: ServerData) -> anyhow::Result<()> {
    SERVER_DATA
        .set(server_data)
        .map_err(|_| anyhow::anyhow!("Server data already set"))?;
    Ok(())
}

fn get_server_data() -> anyhow::Result<&'static ServerData> {
    SERVER_DATA
        .get()
        .ok_or_else(|| anyhow::anyhow!("Server data not set"))
}
