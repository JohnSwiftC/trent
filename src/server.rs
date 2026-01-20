use crate::LOADED_FILES;
use crate::TrentFile;
use crate::upload;
use tokio::net::TcpListener;

pub async fn start_server() {
    set_files(vec![
        TrentFile::from_path_mmap("testvideo.mp4", 45, String::from("largevideo")).unwrap(),
    ])
    .unwrap();

    let listener = TcpListener::bind("127.0.0.1:6969").await.unwrap();
    let files: &'static [TrentFile] = get_files().unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        let _task = tokio::task::spawn(upload::upload_file(stream, files));
    }
}

fn set_files(files: Vec<TrentFile>) -> Result<(), ()> {
    LOADED_FILES.set(files).map_err(|_| ())?;
    Ok(())
}

fn get_files() -> Option<&'static [TrentFile]> {
    // Some interesting reference stuff here
    LOADED_FILES.get().map(|e| &**e)
}
