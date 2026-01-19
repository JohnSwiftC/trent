use std::fs::File;
use std::io::Write;
use std::io::{self, BufReader, BufWriter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn download_file(mut stream: TcpStream, name: &str, save_name: &str) -> io::Result<()> {
    let mut file_name_bytes = [0u8; 256];
    crate::util::write_str_utf8(&mut file_name_bytes, name);
    stream.write_all(&file_name_bytes).await?;

    let mut compressed_out = File::create_new(save_name)?;

    let version = stream.read_u32().await?;

    if version != 0 {
        eprintln!("Version conflict, quitting");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Server version unsupported",
        ));
    }

    let chunks = stream.read_u32().await?;
    let chunk_size = stream.read_u32().await?;
    let last_chunk_size = stream.read_u32().await?;

    let mut chunk_buffer = vec![0u8; chunk_size as usize];

    for c in 0..chunks {
        stream.write_u32(c).await?;

        if c == chunks - 1 {
            stream
                .read_exact(&mut chunk_buffer[..last_chunk_size as usize])
                .await?;
            compressed_out.write_all(&chunk_buffer[..last_chunk_size as usize])?;
        } else {
            stream.read_exact(&mut chunk_buffer).await?;
            compressed_out.write_all(&chunk_buffer)?;
        }
    }

    let compressed_path = save_name.to_string();
    tokio::task::spawn_blocking(move || -> io::Result<()> {
        let compressed_in = File::open(compressed_path)?;
        let mut decoder = zstd::stream::read::Decoder::new(BufReader::new(compressed_in))?;

        let out = File::create_new("testimage.jpg")?;
        let mut out = BufWriter::new(out);

        io::copy(&mut decoder, &mut out)?;
        out.flush()?;
        Ok(())
    })
    .await
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))??;

    Ok(())
}
