use crate::{
    cfile::CompressedFile,
    handler::{Files, Stream},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn upload_file(Stream(mut stream): Stream, Files(files): Files) {
    let mut file_name_bytes: Vec<u8> = vec![0; 256];
    if stream.read_exact(&mut file_name_bytes).await.is_err() {
        return;
    }

    let mut terminator: usize = 255;
    for (i, b) in file_name_bytes.iter().enumerate() {
        if *b == 0 {
            terminator = i;
            break;
        }
    }

    let file_name = String::from_utf8_lossy(&file_name_bytes[..terminator]);

    let mut file: Option<&'static CompressedFile> = None;

    for f in files {
        if f.name() == file_name {
            file = Some(f);
            break;
        }
    }

    if file.is_none() {
        // handle some no named file case here
        eprint!("Not good!!!");
    }

    let file = file.unwrap();

    stream.write_u32(0).await.unwrap();
    stream.write_u32(file.chunks()).await.unwrap();
    stream.write_u32(file.chunk_size()).await.unwrap();
    stream.write_u32(file.last_chunk_size()).await.unwrap();

    loop {
        let chunk = match stream.read_u32().await {
            Ok(m) => m,
            Err(_e) => return,
        };

        stream
            .write_all(file.get_chunk(chunk).unwrap())
            .await
            .unwrap();
    }
}
