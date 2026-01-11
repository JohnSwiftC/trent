use std::fs::File;
use std::path::Path;
use zstd;

pub struct CompressedFile {
    bytes: Vec<u8>,
    segments: usize,
    last_segment_size: usize,
    name: String,
}

impl CompressedFile {
    pub fn from_path<T: AsRef<Path>>(
        path: T,
        segments: usize,
        name: String,
    ) -> std::io::Result<Self> {
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

    pub fn from_file(file: File, segments: usize, name: String) -> std::io::Result<Self> {
        let bytes = zstd::encode_all(file, 3)?;
        let size = bytes.len();
        Ok(Self {
            bytes,
            segments,
            last_segment_size: size & segments,
            name,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn chunks(&self) -> u32 {
        self.segments as u32
    }

    pub fn chunk_size(&self) -> u32 {
        self.bytes.len() as u32 / self.segments as u32
    }

    pub fn last_chunk_size(&self) -> u32 {
        self.last_segment_size as u32
    }

    pub fn get_chunk(&'static self, chunk: u32) -> Option<&'static [u8]> {
        if chunk as usize > self.segments {
            return None;
        }

        if chunk as usize == self.segments {
            return Some(&self.bytes[(self.bytes.len() - self.last_segment_size - 1)..]);
        }

        let left = self.bytes.len() / self.segments * chunk as usize;
        let right = left + (self.bytes.len() / self.segments);

        Some(&self.bytes[left..right])
    }
}
