use crate::cfile::TrentFile;
use anyhow;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn upload_file(mut stream: TcpStream, files: &'static [TrentFile]) -> anyhow::Result<()> {
    let mut file_name_bytes: Vec<u8> = vec![0; 256];
    if stream.read_exact(&mut file_name_bytes).await.is_err() {
        return Ok(());
    }

    let mut terminator: usize = 255;
    for (i, b) in file_name_bytes.iter().enumerate() {
        if *b == 0 {
            terminator = i;
            break;
        }
    }

    let file_name = String::from_utf8_lossy(&file_name_bytes[..terminator]);

    let mut file: Option<&'static TrentFile> = None;

    for f in files {
        if f.name() == file_name {
            file = Some(f);
            break;
        }
    }

    let file = if let Some(f) = file {
        f
    } else {
        stream.write_u32(u32::MAX).await?;
        let error_msg = format!("File '{}' not found", file_name);
        let mut error_bytes = [0u8; 256];
        crate::util::write_str_utf8(&mut error_bytes, &error_msg);
        stream.write_all(&error_bytes).await?;
        return Ok(());
    };

    stream.write_u32(crate::VERSION).await?;
    stream.write_u32(file.is_compressed() as u32).await?;
    stream.write_u32(file.chunks()).await?;
    stream.write_u32(file.chunk_size()).await?;
    stream.write_u32(file.last_chunk_size()).await?;

    loop {
        let chunk = match stream.read_u32().await {
            Ok(m) => m,
            Err(_e) => return Ok(()),
        };

        let chunk_data = file
            .get_chunk(chunk)
            .ok_or_else(|| anyhow::anyhow!("Invalid chunk {}", chunk))?;
        stream.write_all(chunk_data).await?;
    }
}
