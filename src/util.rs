pub fn write_str_utf8(buf: &mut [u8; 256], s: &str) -> usize {
    let mut len = 0;

    for (i, _) in s.char_indices() {
        if i > buf.len() {
            break;
        }
        len = i;
    }

    let bytes = s.as_bytes();
    buf[..len].copy_from_slice(&bytes[..len]);
    buf[len..].fill(0);

    len
}
