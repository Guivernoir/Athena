//! Core writer / reader logic: append, flush, mmap, reopen.

use crate::layout::SegmentHeader;
use crate::record::MemoryRecord;
use memmap2::{Mmap, MmapMut};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

pub struct Store {
    file: File,
    mmap: Option<MmapMut>,
    len: u64,
}

impl Store {
    /// Create / open an append-only store file.
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .append(true)
            .open(path)?;
        let len = file.metadata()?.len();

        Ok(Self {
            file,
            mmap: None,
            len,
        })
    }

    /// Append a single record to the file.
    pub fn append(&mut self, record: &MemoryRecord) -> std::io::Result<()> {
        let bytes = record.to_bytes();
        self.file.write_all(&bytes)?;
        self.len += bytes.len() as u64;
        Ok(())
    }

    /// Flush OS buffers.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.file.sync_all()
    }

    /// Memory-map the file read-only.
    pub fn mmap(&mut self) -> std::io::Result<Mmap> {
        unsafe { Mmap::map(&self.file) }
    }
}