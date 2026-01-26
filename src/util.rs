pub fn write_str_utf8(buf: &mut [u8; 256], s: &str) -> usize {
    let mut len = 0;

    for (i, ch) in s.char_indices() {
        let ch_len = ch.len_utf8();
        let end = i + ch_len;

        if end > buf.len() {
            break;
        }

        len = end;
    }

    buf[..len].copy_from_slice(&s.as_bytes()[..len]);
    buf[len..].fill(0);

    len
}

pub fn read_str_utf8(buf: &[u8; 256]) -> &str {
    let len = buf.iter().position(|&b| b == 0).unwrap_or(256);
    std::str::from_utf8(&buf[..len]).unwrap_or("Invalid UTF-8")
}
