use std::fs::File;
use std::io;
use std::path::Path;
use memmap2::Mmap;

pub struct MmapReader {
    mmap: Mmap,
}

impl MmapReader {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { mmap })
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.mmap
    }

    pub fn len(&self) -> usize {
        self.mmap.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
