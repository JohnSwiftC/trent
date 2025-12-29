use std::ffi::OsString;
use std::fs::File;
use std::path::Path;
use zstd;

use std::io::{Error, ErrorKind};

pub struct CompressedFile {
    bytes: Vec<u8>,
    segments: usize,
    last_segment_size: usize,
    name: OsString,
}

impl CompressedFile {
    pub fn from_path<T: AsRef<Path>>(path: T, segments: usize) -> std::io::Result<Self> {
        let name = path
            .as_ref()
            .file_name()
            .ok_or(Error::new(ErrorKind::NotFound, "No file name in path"))?
            .to_owned();
        let file: File = File::open(path)?;
        let bytes = zstd::encode_all(file, 3)?;
        let size = bytes.len();
        Ok(Self {
            bytes,
            segments,
            last_segment_size: size % segments,
            name,
        })
    }

    pub fn from_file<T: Into<OsString>>(
        file: File,
        segments: usize,
        name: T,
    ) -> std::io::Result<Self> {
        let bytes = zstd::encode_all(file, 3)?;
        let size = bytes.len();
        Ok(Self {
            bytes,
            segments,
            last_segment_size: size & segments,
            name: name.into(),
        })
    }
}
