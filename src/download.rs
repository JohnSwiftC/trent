use crate::util::write_str_utf8;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use std::fs::File;
use std::io::Error;
use std::io::Write;

pub struct DownloadingFile {
    name: String,
    file: File,
    last_downloaded_chunk: u32,
}

pub async fn download_file(
    mut stream: TcpStream,
    name: &str,
    save_name: &str,
) -> Result<(), Error> {
    let mut file_name_bytes: [u8; 256] = [0; 256];
    write_str_utf8(&mut file_name_bytes, name);

    stream.write_all(&file_name_bytes).await?;

    let mut file = File::create_new(save_name)?;

    let chunks = stream.read_u32().await?;
    let chunk_size = stream.read_u32().await?;
    let last_chunk_size = stream.read_u32().await?;

    let mut chunk_buffer: Vec<u8> = vec![0; chunk_size as usize];
    for c in 1..=chunks {
        stream.write_u32(c).await?;

        if c == chunks {
            stream
                .read_exact(&mut chunk_buffer[..last_chunk_size as usize])
                .await?;

            file.write_all(&chunk_buffer[..last_chunk_size as usize])?;
            break;
        } else {
            stream.read_exact(&mut chunk_buffer).await?;
            file.write_all(&chunk_buffer)?;
        }
    }

    let hmm = zstd::decode_all(file).unwrap();
    let mut new_file = File::create_new("testimage.jpg").unwrap();

    new_file.write_all(&hmm).unwrap();

    Ok(())
}
