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
struct ClientArgs {
    #[arg(long, default_value = "127.0.0.1:5000")]
    addr: String,

    #[arg(long)]
    available: Option<bool>,

    #[arg(long)]
    file: Option<std::path::PathBuf>,

    #[arg(long)]
    out: Option<std::path::PathBuf>,

    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

#[derive(Args, Debug)]
struct ServerArgs {
    #[arg(long, default_value = "0.0.0.0:5000")]
    bind: String,
}
