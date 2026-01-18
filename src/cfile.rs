use std::{
    fs::File,
    io,
    io::{Read, Write},
    path::Path,
};

use memmap2::Mmap;
use tempfile::NamedTempFile;

enum Owner {
    Original(File),
    Compressed(NamedTempFile),
}

pub struct CompressedFile {
    mmap: Mmap,
    _owner: Owner,
    segments: usize,
    chunk_size: usize,
    last_chunk_size: usize,
    name: String,
}

impl CompressedFile {
    fn new_mmap(mmap: Mmap, owner: Owner, segments: usize, name: String) -> io::Result<Self> {
        if segments == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "segments must be > 0",
            ));
        }

        let len = mmap.len();
        let base = len / segments;
        let rem = len % segments;
        let last = base + rem;

        Ok(Self {
            mmap,
            _owner: owner,
            segments,
            chunk_size: base,
            last_chunk_size: last,
            name,
        })
    }

    /// Compress a file at `path` into a temp file, then mmap it for zero-copy chunk slicing.
    /// `level` is zstd compression level (e.g. 1..=19ish; 3 is common).
    pub fn from_path_zstd_mmap<P: AsRef<Path>>(
        path: P,
        segments: usize,
        name: String,
        level: i32,
    ) -> io::Result<Self> {
        let mut input = File::open(path)?;

        let mut tmp = NamedTempFile::new()?;

        {
            let mut encoder = zstd::stream::Encoder::new(&mut tmp, level)?;
            io::copy(&mut input, &mut encoder)?;
            let mut writer = encoder.finish()?;
            writer.flush()?;
        }

        let file_for_map: &File = tmp.as_file();
        let mmap = unsafe { Mmap::map(file_for_map)? };

        Self::new_mmap(mmap, Owner::Compressed(tmp), segments, name)
    }

    pub fn from_path_mmap<P: AsRef<Path>>(
        path: P,
        segments: usize,
        name: String,
    ) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        Self::new_mmap(mmap, Owner::Original(file), segments, name)
    }

    pub fn name(&self) -> &str {
        &self.name
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

        self.mmap.get(start..end)
    }

    pub fn len(&self) -> usize {
        self.mmap.len()
    }
}
