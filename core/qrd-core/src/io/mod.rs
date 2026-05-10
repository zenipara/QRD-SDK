//! I/O operations and utilities

use crate::error::Result;
use std::io::{Read, Seek, Write};

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

            self.buffer
                .extend_from_slice(&data[offset..offset + to_write]);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_buffered_reader_initialization() {
        let data = vec![1, 2, 3, 4, 5];
        let cursor = Cursor::new(data);
        let reader = BufferedReader::new(cursor, 1024);
        
        assert_eq!(reader.buffer.capacity(), 1024);
    }

    #[test]
    fn test_buffered_reader_read_exact_bytes() -> Result<()> {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let cursor = Cursor::new(data.clone());
        let mut reader = BufferedReader::new(cursor, 4);
        
        let bytes = reader.read_exact_bytes(3)?;
        assert_eq!(bytes, vec![1, 2, 3]);
        
        let more_bytes = reader.read_exact_bytes(3)?;
        assert_eq!(more_bytes, vec![4, 5, 6]);
        
        Ok(())
    }

    #[test]
    fn test_buffered_reader_eof_handling() -> Result<()> {
        let data = vec![1, 2, 3];
        let cursor = Cursor::new(data);
        let mut reader = BufferedReader::new(cursor, 10);
        
        let bytes = reader.read_exact_bytes(3)?;
        assert_eq!(bytes.len(), 3);
        
        let more = reader.read_exact_bytes(1)?;
        assert_eq!(more.len(), 0); // EOF
        
        Ok(())
    }

    #[test]
    fn test_buffered_reader_seek() -> Result<()> {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let cursor = Cursor::new(data);
        let mut reader = BufferedReader::new(cursor, 5);
        
        reader.seek(5)?;
        let bytes = reader.read_exact_bytes(3)?;
        assert_eq!(bytes, vec![5, 6, 7]);
        
        Ok(())
    }

    #[test]
    fn test_buffered_reader_position() -> Result<()> {
        let data = vec![1, 2, 3, 4, 5];
        let cursor = Cursor::new(data);
        let mut reader = BufferedReader::new(cursor, 1024);
        
        let pos = reader.position()?;
        assert_eq!(pos, 0);
        
        reader.read_exact_bytes(2)?;
        let new_pos = reader.position()?;
        assert_eq!(new_pos, 2);
        
        Ok(())
    }

    #[test]
    fn test_buffered_writer_initialization() {
        let cursor = Cursor::new(Vec::new());
        let writer = BufferedWriter::new(cursor, 1024);
        
        assert_eq!(writer.buffer_size, 1024);
        assert_eq!(writer.buffer.len(), 0);
    }

    #[test]
    fn test_buffered_writer_write_bytes() -> Result<()> {
        let cursor = Cursor::new(Vec::new());
        let mut writer = BufferedWriter::new(cursor, 10);
        
        writer.write_bytes(&[1, 2, 3])?;
        assert_eq!(writer.buffer.len(), 3);
        
        writer.write_bytes(&[4, 5])?;
        assert_eq!(writer.buffer.len(), 5);
        
        Ok(())
    }

    #[test]
    fn test_buffered_writer_flush() -> Result<()> {
        let cursor = Cursor::new(Vec::new());
        let mut writer = BufferedWriter::new(cursor, 10);
        
        writer.write_bytes(&[1, 2, 3])?;
        assert_eq!(writer.buffer.len(), 3);
        
        writer.flush()?;
        assert_eq!(writer.buffer.len(), 0);
        
        Ok(())
    }

    #[test]
    fn test_buffered_writer_auto_flush() -> Result<()> {
        let cursor = Cursor::new(Vec::new());
        let mut writer = BufferedWriter::new(cursor, 5);
        
        writer.write_bytes(&[1, 2, 3])?;
        assert_eq!(writer.buffer.len(), 3);
        
        // Writing more data than buffer size should trigger flush
        writer.write_bytes(&[4, 5, 6, 7])?;
        
        Ok(())
    }

    #[test]
    fn test_buffered_writer_finish() -> Result<()> {
        let cursor = Cursor::new(Vec::new());
        let writer = BufferedWriter::new(cursor, 10);
        
        writer.finish()?;
        // Should complete successfully
        Ok(())
    }

    #[test]
    fn test_buffered_writer_deterministic_output() -> Result<()> {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let cursor1 = Cursor::new(Vec::new());
        let mut writer1 = BufferedWriter::new(cursor1, 10);
        writer1.write_bytes(&data)?;
        writer1.flush()?;

        let cursor2 = Cursor::new(Vec::new());
        let mut writer2 = BufferedWriter::new(cursor2, 10);
        writer2.write_bytes(&data)?;
        writer2.flush()?;

        assert_eq!(writer1.inner.get_ref(), writer2.inner.get_ref());
        Ok(())
    }

    #[test]
    fn test_buffered_reader_large_reads() -> Result<()> {
        let data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let cursor = Cursor::new(data.clone());
        let mut reader = BufferedReader::new(cursor, 64);
        
        let bytes = reader.read_exact_bytes(500)?;
        assert_eq!(bytes.len(), 500);
        assert_eq!(bytes, &data[0..500]);
        
        Ok(())
    }

    #[test]
    fn test_buffered_writer_large_writes() -> Result<()> {
        let cursor = Cursor::new(Vec::new());
        let mut writer = BufferedWriter::new(cursor, 64);
        
        let data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        writer.write_bytes(&data)?;
        writer.flush()?;
        
        assert_eq!(writer.inner.get_ref().len(), 1000);
        Ok(())
    }

    #[test]
    fn test_buffered_reader_multiple_seeks() -> Result<()> {
        let data: Vec<u8> = (0..100).collect();
        let cursor = Cursor::new(data);
        let mut reader = BufferedReader::new(cursor, 10);
        
        reader.seek(10)?;
        let bytes1 = reader.read_exact_bytes(5)?;
        
        reader.seek(20)?;
        let bytes2 = reader.read_exact_bytes(5)?;
        
        reader.seek(10)?;
        let bytes3 = reader.read_exact_bytes(5)?;
        
        assert_eq!(bytes1, bytes3);
        Ok(())
    }

    #[test]
    fn test_buffered_writer_empty_write() -> Result<()> {
        let cursor = Cursor::new(Vec::new());
        let mut writer = BufferedWriter::new(cursor, 10);
        
        writer.write_bytes(&[])?;
        assert_eq!(writer.buffer.len(), 0);
        
        writer.flush()?;
        Ok(())
    }

    // Additional enterprise-grade IO tests

    #[test]
    fn test_buffered_reader_eof_handling() -> Result<()> {
        let data = vec![1, 2, 3, 4];
        let cursor = Cursor::new(data);
        let mut reader = BufferedReader::new(cursor, 10);
        
        // Read all data
        let bytes = reader.read_exact_bytes(4)?;
        assert_eq!(bytes, vec![1, 2, 3, 4]);
        
        // Try to read more - should return empty
        let more = reader.read_exact_bytes(1)?;
        assert_eq!(more.len(), 0);
        
        Ok(())
    }

    #[test]
    fn test_buffered_reader_buffered_reads() -> Result<()> {
        let data: Vec<u8> = (0..100).collect();
        let cursor = Cursor::new(data.clone());
        let mut reader = BufferedReader::new(cursor, 20);
        
        // Read in small chunks
        for i in 0..10 {
            let chunk = reader.read_exact_bytes(10)?;
            assert_eq!(chunk, &data[i*10..(i+1)*10]);
        }
        
        Ok(())
    }

    #[test]
    fn test_buffered_writer_partial_writes() -> Result<()> {
        let cursor = Cursor::new(Vec::new());
        let mut writer = BufferedWriter::new(cursor, 10);
        
        // Write data that doesn't fill buffer
        writer.write_bytes(&[1, 2, 3])?;
        assert_eq!(writer.buffer.len(), 3);
        
        // Write more to trigger partial flush
        writer.write_bytes(&[4, 5, 6, 7, 8, 9, 10, 11])?;
        
        Ok(())
    }

    #[test]
    fn test_buffered_reader_invalid_streams() -> Result<()> {
        // Test with empty stream
        let cursor = Cursor::new(Vec::new());
        let mut reader = BufferedReader::new(cursor, 10);
        
        let bytes = reader.read_exact_bytes(1)?;
        assert_eq!(bytes.len(), 0);
        
        Ok(())
    }

    #[test]
    fn test_buffered_writer_deterministic_io_behavior() -> Result<()> {
        let data = vec![1, 2, 3, 4, 5];
        
        // Write same data multiple times
        let cursor1 = Cursor::new(Vec::new());
        let mut writer1 = BufferedWriter::new(cursor1, 10);
        writer1.write_bytes(&data)?;
        writer1.flush()?;
        
        let cursor2 = Cursor::new(Vec::new());
        let mut writer2 = BufferedWriter::new(cursor2, 10);
        writer2.write_bytes(&data)?;
        writer2.flush()?;
        
        assert_eq!(writer1.inner.get_ref(), writer2.inner.get_ref());
        
        Ok(())
    }
}
