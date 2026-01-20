use std::sync::OnceLock;

pub mod cfile;
pub mod client;
pub mod server;
pub mod util;

use cfile::TrentFile;

static LOADED_FILES: OnceLock<Vec<TrentFile>> = OnceLock::new();
static VERSION: u32 = 0;

#[tokio::main]
async fn main() {
    let option: String = std::env::args()
        .collect::<Vec<String>>()
        .get(1)
        .cloned()
        .expect("No arg provided");

    match option.as_str() {
        "server" => server::start_server().await,
        "client" => client::start_client().await,
        _ => (),
    }
}
