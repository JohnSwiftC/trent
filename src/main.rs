use std::sync::OnceLock;
use tokio::net::{TcpListener, TcpStream};

pub mod cfile;
pub mod download;
pub mod upload;
pub mod util;

use cfile::TrentFile;

static LOADED_FILES: OnceLock<Vec<TrentFile>> = OnceLock::new();

#[tokio::main]
async fn main() {
    let option: String = std::env::args()
        .collect::<Vec<String>>()
        .get(1)
        .cloned()
        .expect("No arg provided");

    match option.as_str() {
        "server" => start_server().await,
        "client" => start_client().await,
        _ => (),
    }
}

/// Very hacky rn, will solidify later
/// but I need to get file transfer mechanics down here.
async fn start_server() {
    set_files(vec![
        TrentFile::from_path_mmap("testvideo.mp4", 45, String::from("largevideo")).unwrap(),
    ])
    .unwrap();

    let listener = TcpListener::bind("127.0.0.1:6969").await.unwrap();
    let files: &'static [TrentFile] = get_files().unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        let _task = tokio::task::spawn(upload::upload_file(stream, files));
    }
}

/// Read start_server
async fn start_client() {
    let mut stream = TcpStream::connect("127.0.0.1:6969").await.unwrap();

    download::download_file(&mut stream, "largevideo", "newvid.mp4")
        .await
        .unwrap();
}

fn set_files(files: Vec<TrentFile>) -> Result<(), ()> {
    LOADED_FILES.set(files).map_err(|_| ())?;
    Ok(())
}

fn get_files() -> Option<&'static [TrentFile]> {
    // Some interesting reference stuff here
    LOADED_FILES.get().map(|e| &**e)
}
