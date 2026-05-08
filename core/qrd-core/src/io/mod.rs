//! I/O operations and utilities

use std::io::{Read, Write, Seek};
use crate::error::Result;

/// Reader wrapper
pub struct BufferedReader<R: Read + Seek> {
    inner: R,
    buffer: Vec<u8>,
    buffer_pos: usize,
    buffer_len: usize,
}

impl<R: Read + Seek> BufferedReader<R> {
    /// Create new buffered reader
    pub fn new(inner: R, buffer_size: usize) -> Self {
        BufferedReader {
            inner,
            buffer: vec![0; buffer_size],
            buffer_pos: 0,
            buffer_len: 0,
        }
    }

    /// Read exact bytes
    pub fn read_exact_bytes(&mut self, n: usize) -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(n);
        let mut remaining = n;

        while remaining > 0 {
            if self.buffer_pos >= self.buffer_len {
                self.buffer_len = self.inner.read(&mut self.buffer)?;
                self.buffer_pos = 0;

                if self.buffer_len == 0 {
                    break;
                }
            }

            let available = self.buffer_len - self.buffer_pos;
            let to_copy = available.min(remaining);
            result.extend_from_slice(&self.buffer[self.buffer_pos..self.buffer_pos + to_copy]);
            self.buffer_pos += to_copy;
            remaining -= to_copy;
        }

        Ok(result)
    }

    /// Seek to position
    pub fn seek(&mut self, pos: u64) -> Result<u64> {
        let new_pos = self.inner.seek(std::io::SeekFrom::Start(pos))?;
        self.buffer_pos = self.buffer_len; // Invalidate buffer
        Ok(new_pos)
    }

    /// Get current position
    pub fn position(&mut self) -> Result<u64> {
        Ok(self.inner.seek(std::io::SeekFrom::Current(0))?)
    }
}

/// Writer wrapper
pub struct BufferedWriter<W: Write> {
    inner: W,
    buffer: Vec<u8>,
    buffer_size: usize,
}

impl<W: Write> BufferedWriter<W> {
    /// Create new buffered writer
    pub fn new(inner: W, buffer_size: usize) -> Self {
        BufferedWriter {
            inner,
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
        }
    }

    /// Write bytes
    pub fn write_bytes(&mut self, data: &[u8]) -> Result<()> {
        let mut offset = 0;

        while offset < data.len() {
            let available = self.buffer_size - self.buffer.len();
            let to_write = available.min(data.len() - offset);

            self.buffer.extend_from_slice(&data[offset..offset + to_write]);
            offset += to_write;

            if self.buffer.len() >= self.buffer_size {
                self.flush()?;
            }
        }

        Ok(())
    }

    /// Flush buffer
    pub fn flush_buffer(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            self.inner.write_all(&self.buffer)?;
            self.buffer.clear();
        }
        self.inner.flush()?;
        Ok(())
    }

    /// Flush (alias for flush_buffer)
    pub fn flush(&mut self) -> Result<()> {
        self.flush_buffer()
    }

    /// Finalize
    pub fn finish(mut self) -> Result<()> {
        self.flush_buffer()
    }
}
