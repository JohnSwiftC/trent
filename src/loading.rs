use std::fs::File;
use std::path::Path;
use zstd;

pub struct CompressedFile {
    bytes: Vec<u8>,
    segments: usize,
    last_segment_size: usize,
}

impl CompressedFile {
    pub fn from_path<T: AsRef<Path>>(path: T, segments: usize) -> std::io::Result<Self> {
        let file: File = File::open(path)?;
        let bytes = zstd::encode_all(file, 3)?;
        let size = bytes.len();
        Ok(Self {
            bytes,
            segments,
            last_segment_size: size % segments,
        })
    }

    pub fn from_file(file: File, segments: usize) -> std::io::Result<Self> {
        let bytes = zstd::encode_all(file, 3)?;
        let size = bytes.len();
        Ok(Self {
            bytes,
            segments,
            last_segment_size: size & segments,
        })
    }
}
