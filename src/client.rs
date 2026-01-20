use tokio::net::TcpStream;

pub mod download;

pub async fn start_client() {
    let mut stream = TcpStream::connect("127.0.0.1:6969").await.unwrap();

    download::download_file(&mut stream, "largevideo", "newvid.mp4")
        .await
        .unwrap();
}
