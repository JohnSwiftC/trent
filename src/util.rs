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
