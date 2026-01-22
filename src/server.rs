use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::LOADED_FILES;
use crate::TrentFile;
use tokio::net::{TcpListener, TcpStream};
mod config;
mod getfiles;
mod standard;
mod upload;

use standard::ServerRoute;

pub async fn start_server(port: u16) {
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

    set_files(vec![
        TrentFile::from_path_zstd_mmap("testvideo.mp4", 45, String::from("largevideo"), 10)
            .unwrap(),
    ])
    .unwrap();

    let listener = TcpListener::bind(socket_addr).await.unwrap();
    let files: &'static [TrentFile] = get_files().unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        let _task = tokio::task::spawn(handle(stream, files));
    }
}

async fn handle(mut stream: TcpStream, files: &'static [TrentFile]) -> io::Result<()> {
    match standard::route(&mut stream).await? {
        ServerRoute::Upload => upload::upload_file(stream, files).await,
        ServerRoute::GetFiles => todo!(),
        ServerRoute::Unknown => todo!(),
    }

    Ok(())
}

fn set_files(files: Vec<TrentFile>) -> Result<(), ()> {
    LOADED_FILES.set(files).map_err(|_| ())?;
    Ok(())
}

fn get_files() -> Option<&'static [TrentFile]> {
    // Some interesting reference stuff here
    LOADED_FILES.get().map(|e| &**e)
}
