use std::sync::OnceLock;
use tokio::net::{TcpListener, TcpStream};

pub mod download;
pub mod handler;
pub mod loading;
pub mod upload;
pub mod util;

use loading::CompressedFile;

use handler::{Context, Files, Handler};

static LOADED_FILES: OnceLock<Vec<CompressedFile>> = OnceLock::new();

#[tokio::main]
async fn main() {
    set_files(vec![
        CompressedFile::from_path("dogdog.jpg", 15, String::from("dogdog")).unwrap(),
    ])
    .unwrap();
    let files: &'static [CompressedFile] = get_files().unwrap();

    let context = Context {
        stream: Some(dummy_tcp_stream().await),
        files,
    };

    Handler::call(test, context).unwrap().await;
}

async fn start_server() {
    let listener = TcpListener::bind("0.0.0.0:6969").await.unwrap();
    let files: &'static [CompressedFile] = get_files().unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        let context = Context {
            stream: Some(stream),
            files,
        };

        Handler::call(upload::upload_file, context).unwrap().await;
    }
}

async fn start_client() {
    let stream = TcpStream::connect("0.0.0.0:6969").await.unwrap();
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
