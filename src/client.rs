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

    match args.action {
        ClientAction::Download(DownloadArgs { peer, file, output }) => {
            let peers = peer_store.peers_by_name(&peer, RequestView::Lan)?;
            let mut tasks = JoinSet::new();
            for peer in peers {
                // Supposedly a person using this does not have 1 million
                // peers they have named the same thing so this should never be an
                // issue
                let file = file.clone();
                let output = output.clone();

                tasks.spawn(async move {
                    let mut stream = peer.connect(2000).await?;

                    standard::action(
                        &mut stream,
                        ClientRoute::DownloadFile {
                            name: file,
                            save_name: output,
                        },
                    )
                    .await?;

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
        ClientAction::GetFiles => {
            let peers = peer_store.peer_list(100, RequestView::Lan)?;
            let mut tasks = JoinSet::new();
            for peer in peers.peers {
                tasks.spawn(async move {
                    let mut stream = peer.connect(2000).await?;
                    standard::action(&mut stream, ClientRoute::GetFiles { name: peer.name })
                        .await?;

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
            let peers = peer_store.peer_list(100, RequestView::Lan)?;
            for peer in peers.peers {
                println!("{:#?}", peer);
            }
        }
    }

    Ok(())
}
