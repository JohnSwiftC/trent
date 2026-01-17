use std::io::Read;
pub struct CompressedFile {
    bytes: Vec<u8>,
    segments: usize,        // number of chunks
    chunk_size: usize,      // base size
    last_chunk_size: usize, // size of final chunk
    name: String,
}

impl CompressedFile {
    fn new(bytes: Vec<u8>, segments: usize, name: String) -> Self {
        let len = bytes.len();
        let base = len / segments;
        let rem = len % segments;
        let last = base + rem; // remainder goes into final chunk

        Self {
            bytes,
            segments,
            chunk_size: base,
            last_chunk_size: last,
            name,
        }
    }

    pub fn from_file_uncompressed<P: AsRef<std::path::Path>>(
        path: P,
        segments: usize,
        name: String,
    ) -> std::io::Result<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(Self::new(bytes, segments, name))
    }

    pub fn chunks(&self) -> u32 {
        self.segments as u32
    }
    pub fn chunk_size(&self) -> u32 {
        self.chunk_size as u32
    }
    pub fn last_chunk_size(&self) -> u32 {
        self.last_chunk_size as u32
    }

    pub fn get_chunk(&self, chunk: u32) -> Option<&[u8]> {
        let chunk = chunk as usize;
        if chunk >= self.segments {
            return None;
        }

        let start = self.chunk_size * chunk;
        let end = if chunk + 1 == self.segments {
            start + self.last_chunk_size
        } else {
            start + self.chunk_size
        };

        self.bytes.get(start..end)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
