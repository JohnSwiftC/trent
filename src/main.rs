use std::sync::OnceLock;

pub mod cfile;
pub mod client;
pub mod server;
pub mod util;

use cfile::TrentFile;
use server::config::ServerData;

use clap::{Args, Parser, Subcommand};

static SERVER_DATA: OnceLock<ServerData> = OnceLock::new();
static VERSION: u32 = 0;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.cmd {
        Command::Client(args) => client::start_client(args).await.unwrap(),
        Command::Server(args) => server::start_server(args).await,
    }
}

#[derive(Parser, Debug)]
#[command(name = "trent", version, about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Client(ClientArgs),
    Server(ServerArgs),
}

#[derive(Args, Debug)]
pub struct ClientArgs {
    #[arg(long, default_value = "127.0.0.1:5000")]
    addr: String,

    #[command(subcommand)]
    action: ClientAction,
}

#[derive(Subcommand, Debug)]
enum ClientAction {
    Download(DownloadArgs),
    GetFiles,
}

#[derive(Args, Debug)]
struct DownloadArgs {
    #[arg(long)]
    file: String,

    #[arg(short, long)]
    output: String,
}

#[derive(Args, Debug)]
pub struct ServerArgs {
    #[arg(long, default_value = "0.0.0.0:5000")]
    bind: String,
}
