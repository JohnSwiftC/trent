use crate::handler::{File, Files, Stream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn upload_file(Stream(mut stream): Stream, Files(files): Files) {
    let mut file_name_bytes: Vec<u8> = vec![0; 256];
    if stream.read_exact(&mut file_name_bytes).await.is_err() {
        return;
    }

    let file_name: String;
    for (i, b) in file_name_bytes.iter().enumerate() {
        if *b == 0 || i == 255 {
            file_name = 
        }
    }

    loop {
        let mut chunk = 0;
        match stream.read_u16().await {
            Ok(m) => chunk = m,
            Err(_e) => return,
        }
    }
}
