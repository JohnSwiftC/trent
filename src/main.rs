use tokio::net::{TcpListener, TcpStream};

use std::sync::OnceLock;

pub mod handler;
pub mod loading;
pub mod upload;

use loading::CompressedFile;

static LOADED_FILES: OnceLock<Vec<CompressedFile>> = OnceLock::new();

#[tokio::main]
async fn main() {}

fn set_files(files: Vec<CompressedFile>) -> Result<(), ()> {
    LOADED_FILES.set(files).map_err(|_| ())?;
    Ok(())
}

fn get_files() -> Option<&'static [CompressedFile]> {
    // Some interesting reference stuff here
    LOADED_FILES.get().map(|e| &**e)
}
