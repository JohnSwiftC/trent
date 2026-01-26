use std::io;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

pub async fn get_files(stream: &mut TcpStream) -> io::Result<String> {
    let size = stream.read_u32().await?;
    let mut buf = vec![0; size as usize];

    stream.read_exact(&mut buf).await?;

    Ok(parse_bytes(&buf).await)
}

async fn parse_bytes(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());

    let mut left = 0;
    let mut right = 0;

    while right < bytes.len() {
        if bytes[right] == 0 {
            let mut file_name = String::from_utf8_lossy(&bytes[left..right]).to_string();
            left = right;

            left += 1;
            right += 5;

            let flags =
                u32::from_be_bytes(bytes[left..right].try_into().expect("Developer error LOL"));
            if flags & 1 == 1 {
                file_name += ": Compressed\n";
            } else {
                file_name += ": Uncompressed\n";
            }

            left = right;

            result += &("[*] ".to_owned() + &file_name);
        } else {
            right += 1;
        }
    }

    result
}
