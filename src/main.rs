use std::sync::OnceLock;


pub mod cfile;
pub mod client;
pub mod server;
pub mod util;

use cfile::TrentFile;
use server::config::ServerData;

use clap::{Args, Parser, Subcommand, ValueHint};

static SERVER_DATA: OnceLock<ServerData> = OnceLock::new();
static VERSION: u32 = 0;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse().into_command();

    match cli {
        Command::Client(args) => client::start_client(args).await?,
        Command::Server(args) => server::start_server(args).await?,
    }

    Ok(())
}

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

#[derive(Parser, Debug)]
#[command(
    name = "trent",
    version,
    about = "Simple file transfer client/server",
    arg_required_else_help = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    Serve(ServeUx),
    Download(DownloadUx),
    Files(FilesUx),
}

#[derive(Args, Debug)]
struct ServeUx {
    #[arg(
        long,
        default_value = "0.0.0.0:5000",
        value_name = "IP:PORT",
        value_hint = ValueHint::Other
    )]
    bind: String,
}

#[derive(Args, Debug)]
struct FilesUx {
    #[arg(
        long,
        value_name = "IP:PORT",
        value_hint = ValueHint::Other
    )]
    addr: String,
}

#[derive(Args, Debug)]
struct DownloadUx {
    #[arg(
        long,
        value_name = "IP:PORT",
        value_hint = ValueHint::Other
    )]
    addr: String,

    #[arg(long, value_name = "NAME")]
    file: String,

    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath)]
    output: String,
}

impl Cli {
    fn into_command(self) -> Command {
        match self.cmd {
            Cmd::Serve(s) => Command::Server(ServerArgs { bind: s.bind }),
            Cmd::Files(f) => Command::Client(ClientArgs {
                addr: f.addr,
                action: ClientAction::GetFiles,
            }),
            Cmd::Download(d) => Command::Client(ClientArgs {
                addr: d.addr,
                action: ClientAction::Download(DownloadArgs {
                    file: d.file,
                    output: d.output,
                }),
            }),
        }
    }
}
