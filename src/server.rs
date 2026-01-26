use anyhow;

use crate::SERVER_DATA;
use crate::TrentFile;
pub mod config;
mod getfiles;
mod standard;
mod upload;

use crate::ServerArgs;
use config::ServerData;
use standard::ServerRoute;

use serde::Deserialize;
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};

#[derive(Deserialize)]
struct Config {
    files: Vec<HashMap<String, FileEntry>>,
}

#[derive(Deserialize)]
struct FileEntry {
    path: String,
    compressed: bool,
}

pub async fn start_server(args: ServerArgs) -> anyhow::Result<()> {
    let config: Config = serde_yaml::from_str(&std::fs::read_to_string(args.config)?)?;

    let mut files = Vec::new();

    for entry in config.files {
        for (name, file) in entry {
            let tf = if file.compressed {
                TrentFile::from_path_zstd_mmap(&file.path, 40, name, 6)?
            } else {
                TrentFile::from_path_mmap(&file.path, 40, name)?
            };
            files.push(tf);
        }
    }

    set_server_data(ServerData::from_files(files))?;
    let server_data = get_server_data()?;

    let listener = TcpListener::bind(args.bind).await?;

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
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
