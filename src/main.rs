use std::sync::OnceLock;
use tokio::net::{TcpListener, TcpStream};

pub mod cfile;
pub mod download;
pub mod handler;
pub mod upload;
pub mod util;

use cfile::CompressedFile;

use handler::{Context, Files, Handler};

static LOADED_FILES: OnceLock<Vec<CompressedFile>> = OnceLock::new();

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
        CompressedFile::from_file_uncompressed("dogdog.jpg", 15, String::from("dogdog")).unwrap(),
    ])
    .unwrap();

    let listener = TcpListener::bind("127.0.0.1:6969").await.unwrap();
    let files: &'static [CompressedFile] = get_files().unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        let context = Context {
            stream: Some(stream),
            files,
        };

        Handler::call(upload::upload_file, context).unwrap().await;
    }
}

/// Read start_server
async fn start_client() {
    let stream = TcpStream::connect("127.0.0.1:6969").await.unwrap();

    download::download_file(stream, "dogdog", "newdogdog.png")
        .await
        .unwrap();
}

async fn test(Files(files): Files) {
    println!("Hello {}", files.len());
}

fn set_files(files: Vec<CompressedFile>) -> Result<(), ()> {
    LOADED_FILES.set(files).map_err(|_| ())?;
    Ok(())
}

fn get_files() -> Option<&'static [CompressedFile]> {
    // Some interesting reference stuff here
    LOADED_FILES.get().map(|e| &**e)
}

async fn dummy_tcp_stream() -> TcpStream {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let _client = TcpStream::connect(addr).await.unwrap();
    let (server, _) = listener.accept().await.unwrap();

    // `client` and `server` are connected
    server
}
