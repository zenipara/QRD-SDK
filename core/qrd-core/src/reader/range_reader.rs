//! Range reader - supports HTTP byte-range requests and partial access
//!
//! Enables:
//! - HTTP Range: bytes=X-Y header compatibility
//! - CDN/Edge caching of specific row groups
//! - Resumable downloads
//! - Bandwidth-efficient access patterns

use crate::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// Byte range specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    /// Start byte (inclusive)
    pub start: u64,
    /// End byte (inclusive)
    pub end: u64,
}

impl ByteRange {
    /// Create new byte range
    pub fn new(start: u64, end: u64) -> Result<Self> {
        if start > end {
            return Err(crate::error::Error::InvalidData(format!(
                "Invalid range: start {} > end {}",
                start, end
            )));
        }
        Ok(ByteRange { start, end })
    }

    /// Get range size in bytes
    pub fn size(&self) -> u64 {
        self.end - self.start + 1
    }

    /// Parse from HTTP Range header (e.g., "bytes=0-1023")
    pub fn parse_http_range(range_str: &str) -> Result<Option<ByteRange>> {
        if !range_str.starts_with("bytes=") {
            return Ok(None);
        }

        let range_part = &range_str[6..];

        if range_part.contains(',') {
            // Multiple ranges not supported
            return Ok(None);
        }

        let parts: Vec<&str> = range_part.split('-').collect();
        if parts.len() != 2 {
            return Ok(None);
        }

        match (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
            (Ok(start), Ok(end)) => Ok(Some(ByteRange::new(start, end)?)),
            _ => Ok(None),
        }
    }

    /// Convert to HTTP Range header format
    pub fn to_http_header(&self) -> String {
        format!("bytes={}-{}", self.start, self.end)
    }
}

/// Range-based reader for partial/streaming access
pub struct RangeReader<R: Read + Seek> {
    inner: R,
    range: ByteRange,
    current_pos: u64,
}

impl<R: Read + Seek> RangeReader<R> {
    /// Create new range reader
    pub fn new(mut inner: R, range: ByteRange) -> Result<Self> {
        // Seek to start of range
        inner.seek(SeekFrom::Start(range.start))?;

        Ok(RangeReader {
            inner,
            range,
            current_pos: range.start,
        })
    }

    /// Get current position within range
    pub fn position(&self) -> u64 {
        self.current_pos
    }

    /// Get bytes remaining in range
    pub fn remaining(&self) -> u64 {
        if self.current_pos > self.range.end {
            0
        } else {
            self.range.end - self.current_pos + 1
        }
    }

    /// Check if range is exhausted
    pub fn is_exhausted(&self) -> bool {
        self.current_pos > self.range.end
    }

    /// Get the underlying range
    pub fn range(&self) -> ByteRange {
        self.range
    }

    /// Get wrapped reader
    pub fn inner(&mut self) -> &mut R {
        &mut self.inner
    }
}

impl<R: Read + Seek> Read for RangeReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.is_exhausted() {
            return Ok(0);
        }

        // Limit read to range bounds
        let available = self.remaining() as usize;
        let to_read = buf.len().min(available);

        let n = self.inner.read(&mut buf[..to_read])?;
        self.current_pos += n as u64;

        Ok(n)
    }
}

impl<R: Read + Seek> Seek for RangeReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(offset) => {
                if offset > self.range.size() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Seek beyond range",
                    ));
                }
                self.range.start + offset
            }
            SeekFrom::End(offset) => {
                let base = self.range.end as i64;
                let new = base + offset;
                if new < 0 || (new as u64) < self.range.start {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Seek before range",
                    ));
                }
                new as u64
            }
            SeekFrom::Current(offset) => {
                let base = self.current_pos as i64;
                let new = base + offset;
                if new < 0 || (new as u64) < self.range.start || (new as u64) > self.range.end {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Seek out of range",
                    ));
                }
                new as u64
            }
        };

        self.inner.seek(SeekFrom::Start(new_pos))?;
        self.current_pos = new_pos;
        Ok(new_pos - self.range.start) // Return position relative to range start
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_byte_range_creation() {
        let range = ByteRange::new(0, 1023).unwrap();
        assert_eq!(range.size(), 1024);
    }

    #[test]
    fn test_byte_range_invalid() {
        assert!(ByteRange::new(100, 50).is_err());
    }

    #[test]
    fn test_parse_http_range() {
        let range = ByteRange::parse_http_range("bytes=0-1023")
            .unwrap()
            .unwrap();
        assert_eq!(range.start, 0);
        assert_eq!(range.end, 1023);

        let no_range = ByteRange::parse_http_range("invalid").unwrap();
        assert!(no_range.is_none());
    }

    #[test]
    fn test_http_range_header() {
        let range = ByteRange::new(100, 200).unwrap();
        assert_eq!(range.to_http_header(), "bytes=100-200");
    }

    #[test]
    fn test_range_reader() {
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let cursor = Cursor::new(data);
        let range = ByteRange::new(2, 5).unwrap();
        let mut reader = RangeReader::new(cursor, range).unwrap();

        let mut buf = [0u8; 4];
        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 4);
        assert_eq!(&buf[..n], &[3, 4, 5, 6]);
        assert!(reader.is_exhausted());
    }

    #[test]
    fn test_range_reader_remaining() {
        let data = vec![1u8; 100];
        let cursor = Cursor::new(data);
        let range = ByteRange::new(10, 20).unwrap();
        let reader = RangeReader::new(cursor, range).unwrap();

        assert_eq!(reader.remaining(), 11);
    }
}
