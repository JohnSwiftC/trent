use anyhow;

mod download;
mod getfiles;
mod isalive;

// Needs to be seen by the peers module
pub mod standard;

use crate::peers::{PeerStore, RequestView};
use crate::{ClientAction, ClientArgs, DownloadArgs};
use standard::ClientRoute;
use tokio::task::JoinSet;

pub async fn start_client(args: ClientArgs) -> anyhow::Result<()> {
    let peer_store = PeerStore::open(&args.peer_db)?;

    let peers = peer_store.peer_list(100, RequestView::Lan)?;

    match args.action {
        ClientAction::Download(DownloadArgs { file, output }) => {}
        ClientAction::GetFiles => {
            let mut tasks = JoinSet::new();
            for peer in peers.peers {
                tasks.spawn(async move {
                    let mut stream = peer.connect(2000).await?;
                    standard::action(&mut stream, ClientRoute::GetFiles).await?;

                    Ok::<(), anyhow::Error>(())
                });
            }

            while let Some(res) = tasks.join_next().await {
                match res {
                    Ok(Ok(())) => (),
                    Ok(Err(e)) => eprintln!("{e}"),
                    Err(e) => panic!("{e} Join"),
                }
            }
        }
        ClientAction::ViewPeers => {
            for peer in peers.peers {
                println!("{:#?}", peer);
            }
        }
    }

    Ok(())
}
