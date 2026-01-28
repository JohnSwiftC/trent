use std::io;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
pub enum ServerRoute {
    Upload,
    GetFiles,
    IsAlive,
    Unknown,
}

pub async fn route(stream: &mut TcpStream) -> io::Result<ServerRoute> {
    let version = stream.read_u32().await?;

    if version != crate::VERSION {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Unsupported client version!",
        ));
    }

    let action = stream.read_u32().await?;

    match action {
        0 => Ok(ServerRoute::Upload),
        1 => Ok(ServerRoute::GetFiles),
        2 => Ok(ServerRoute::IsAlive),
        _ => Ok(ServerRoute::Unknown),
    }
}
