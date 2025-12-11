use std::fs::File;
use std::path::Path;
use zstd;

pub struct CompressedFile {
    bytes: Vec<u8>,
}

impl CompressedFile {
    pub fn from_path<T: AsRef<Path>>(path: T) -> std::io::Result<Self> {
        let file: File = File::open(path)?;

        Ok(Self {
            bytes: zstd::encode_all(file, 3)?,
        })
    }

    pub fn from_file(file: File) -> std::io::Result<Self> {
        Ok(Self {
            bytes: zstd::encode_all(file, 3)?,
        })
    }
}
