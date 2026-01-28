use std::io;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub enum ClientRoute {
    DownloadFile { name: String, save_name: String },
    GetFiles,
    IsAlive,
}

pub async fn action(stream: &mut TcpStream, route: ClientRoute) -> io::Result<()> {
    stream.write_u32(crate::VERSION).await?;

    match route {
        ClientRoute::DownloadFile { name, save_name } => {
            stream.write_u32(0).await?;
            crate::client::download::download_file(stream, &name, &save_name).await?;
        }
        ClientRoute::GetFiles => {
            stream.write_u32(1).await?;
            print!("{}", crate::client::getfiles::get_files(stream).await?);
        }
        ClientRoute::IsAlive => {
            stream.write_u32(2).await?;
            crate::client::isalive::is_alive(stream).await?;
        }
    }

    Ok(())
}
