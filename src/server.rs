use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::SERVER_DATA;
use crate::TrentFile;
use tokio::net::{TcpListener, TcpStream};
pub mod config;
mod getfiles;
mod standard;
mod upload;

use config::ServerData;
use standard::ServerRoute;

/// This function creates a bunch
pub async fn start_server(port: u16) {
    let files = vec![
        TrentFile::from_path_zstd_mmap("dogdog.jpg", 15, String::from("dogdog"), 6).unwrap(),
        TrentFile::from_path_mmap("testvideo.mp4", 40, "newvideoblahblah".to_owned()).unwrap(),
    ];
    set_server_data(ServerData::from_files(files)).unwrap();
    let server_data = get_server_data().unwrap();

    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
    let listener = TcpListener::bind(socket_addr).await.unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        let _task = tokio::task::spawn(handle(stream, server_data));
    }
}

async fn handle(mut stream: TcpStream, server_data: &'static ServerData) -> io::Result<()> {
    match standard::route(&mut stream).await? {
        ServerRoute::Upload => upload::upload_file(stream, server_data.get_files()).await,
        ServerRoute::GetFiles => getfiles::get_files(&mut stream, server_data).await.unwrap(),
        ServerRoute::Unknown => todo!(),
    }

    Ok(())
}

fn set_server_data(server_data: ServerData) -> Result<(), ()> {
    SERVER_DATA.set(server_data).map_err(|_| ())?;
    Ok(())
}

fn get_server_data() -> Option<&'static ServerData> {
    SERVER_DATA.get().map(|e| &*e)
}
