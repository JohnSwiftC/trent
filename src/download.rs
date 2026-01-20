use std::fs::File;
use std::io::Write;
use std::io::{self, BufReader, BufWriter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn download_file(mut stream: TcpStream, name: &str, save_name: &str) -> io::Result<()> {
    let mut file_name_bytes = [0u8; 256];
    crate::util::write_str_utf8(&mut file_name_bytes, name);
    stream.write_all(&file_name_bytes).await?;

    let version = stream.read_u32().await?;

    if version != 0 {
        eprintln!("Version conflict, quitting");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Server version unsupported",
        ));
    }

    let flags = stream.read_u32().await?;
    let chunks = stream.read_u32().await?;
    let chunk_size = stream.read_u32().await?;
    let last_chunk_size = stream.read_u32().await?;

    // This flag indicates the file being compressed
    if flags & 1 == 1 {
        download_compressed(stream, save_name, chunks, chunk_size, last_chunk_size).await?;
    }

    Ok(())
}

async fn download_compressed(
    mut stream: TcpStream,
    save_name: &str,
    chunks: u32,
    chunk_size: u32,
    last_chunk_size: u32,
) -> io::Result<()> {
    let compressed_path = format!("{}.trent", save_name);
    let mut compressed_out = File::create_new(&compressed_path)?;
    let mut chunk_buffer = vec![0u8; chunk_size.max(last_chunk_size) as usize];

    for c in 0..chunks {
        stream.write_u32(c).await?;

        if c == chunks - 1 {
            stream
                .read_exact(&mut chunk_buffer[..last_chunk_size as usize])
                .await?;
            compressed_out.write_all(&chunk_buffer[..last_chunk_size as usize])?;
        } else {
            stream
                .read_exact(&mut chunk_buffer[..chunk_size as usize])
                .await?;
            compressed_out.write_all(&chunk_buffer[..chunk_size as usize])?;
        }
    }

    compressed_out.flush()?;

    let save_name: String = save_name.to_owned();
    tokio::task::spawn_blocking(move || -> io::Result<()> {
        let compressed_in = File::open(&compressed_path)?;
        let mut decoder = zstd::stream::read::Decoder::new(BufReader::new(compressed_in))?;

        let out = File::create_new(&save_name)?;
        let mut out = BufWriter::new(out);

        io::copy(&mut decoder, &mut out)?;
        out.flush()?;

        if let Err(e) = std::fs::remove_file(compressed_path) {
            eprintln!("Failed to remove compressed temp file: {}", e);
        }

        Ok(())
    })
    .await
    .map_err(|e| std::io::Error::other(format!("Failed: {}", e)))??;

    Ok(())
}
