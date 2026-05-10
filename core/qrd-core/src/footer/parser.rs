//! Footer parser - enables random access without scanning entire file

use crate::error::Result;
use crate::footer::Footer;
use crate::validation::Validator;
use std::io::{Read, Seek, SeekFrom};

/// Parser for QRD file footers
pub struct FooterParser;

impl FooterParser {
    /// Parse footer from file by seeking to end
    ///
    /// QRD files end with:
    /// - Footer data (variable length)
    /// - Footer length (u32)
    /// - Footer CRC32 (u32)
    ///
    /// Total trailer: minimum 8 bytes
    pub fn parse_from_reader<R: Read + Seek>(reader: &mut R) -> Result<Footer> {
        // Seek to end to find footer metadata
        let total_size = reader.seek(SeekFrom::End(0))?;

        if total_size < 40 {
            // Too small to be valid QRD file
            return Err(crate::error::Error::InvalidData(
                "File too small to contain footer".to_string(),
            ));
        }

        // Read footer length and CRC from end
        reader.seek(SeekFrom::End(-8))?;
        let mut footer_length_buf = [0u8; 4];
        let mut crc_buf = [0u8; 4];

        reader.read_exact(&mut footer_length_buf)?;
        reader.read_exact(&mut crc_buf)?;

        let footer_length = u32::from_le_bytes(footer_length_buf) as usize;
        let stored_crc = u32::from_le_bytes(crc_buf);

        if footer_length > 1024 * 1024 {
            // Sanity check: footer shouldn't be larger than 1MB
            return Err(crate::error::Error::InvalidData(
                "Footer too large".to_string(),
            ));
        }

        // Ensure footer_length is consistent with total file size to avoid underflow
        if (footer_length as u64) + 8 > total_size {
            return Err(crate::error::Error::InvalidData(
                "Footer length larger than file size".to_string(),
            ));
        }

        // Seek to start of footer (safe subtraction after check)
        let footer_start = (total_size - 8 - (footer_length as u64)) as u64;
        reader.seek(SeekFrom::Start(footer_start))?;

        // Read footer data
        let mut footer_data = vec![0u8; footer_length];
        reader.read_exact(&mut footer_data)?;

        // Verify CRC32
        let calculated_crc = Validator::calculate_crc32(&footer_data);
        if calculated_crc != stored_crc {
            return Err(crate::error::Error::CrcMismatch {
                expected: stored_crc,
                actual: calculated_crc,
            });
        }

        // Deserialize footer
        Footer::deserialize(&footer_data)
    }

    /// Parse footer from data slice
    pub fn parse_from_slice(data: &[u8]) -> Result<Footer> {
        Footer::deserialize(data)
    }

    /// Find row group containing given row
    pub fn find_row_group_for_row(
        footer: &Footer,
        row_index: u32,
        row_group_size: u32,
    ) -> Result<usize> {
        let rg_index = (row_index / row_group_size) as usize;

        if rg_index >= footer.row_group_offsets.len() {
            return Err(crate::error::Error::InvalidData(
                format!("Row {} beyond available row groups", row_index),
            ));
        }

        Ok(rg_index)
    }

    /// Get offset of row group
    pub fn get_row_group_offset(footer: &Footer, rg_index: usize) -> Result<u64> {
        footer
            .row_group_offsets
            .get(rg_index)
            .copied()
            .ok_or_else(|| {
                crate::error::Error::InvalidData(format!("Row group {} not found", rg_index))
            })
    }

    /// Calculate read range for partial column read
    ///
    /// Returns (start_offset, end_offset) in file
    pub fn get_row_group_range(
        footer: &Footer,
        rg_index: usize,
        file_size: u64,
    ) -> Result<(u64, u64)> {
        if rg_index >= footer.row_group_offsets.len() {
            return Err(crate::error::Error::InvalidData(
                format!("Row group {} not found", rg_index),
            ));
        }

        let start = footer.row_group_offsets[rg_index];

        // End of row group is either start of next row group or footer
        let end = if rg_index + 1 < footer.row_group_offsets.len() {
            footer.row_group_offsets[rg_index + 1]
        } else {
            // Estimate: could be improved with explicit footer offset in header
            file_size - 16 // Approximate footer size
        };

        if start >= end {
            return Err(crate::error::Error::InvalidData(
                "Invalid row group range".to_string(),
            ));
        }

        Ok((start, end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_schema(names: Vec<&str>) -> crate::schema::Schema {
        let mut builder = crate::schema::SchemaBuilder::new();
        for name in names {
            builder = builder
                .add_field(name, crate::schema::FieldType::Blob, crate::schema::Nullability::Required)
                .unwrap();
        }
        builder.build().unwrap()
    }

    #[test]
    fn test_find_row_group_for_row() -> Result<()> {
        let footer = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: vec![32, 1024, 2048],
            row_count: 3000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0,
        };

        // Each row group has 1000 rows (assuming even distribution)
        let rg_idx = FooterParser::find_row_group_for_row(&footer, 500, 1000)?;
        assert_eq!(rg_idx, 0);

        let rg_idx = FooterParser::find_row_group_for_row(&footer, 1500, 1000)?;
        assert_eq!(rg_idx, 1);

        Ok(())
    }

    #[test]
    fn test_get_row_group_offset() -> Result<()> {
        let footer = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: vec![32, 1024, 2048],
            row_count: 3000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0,
        };

        assert_eq!(FooterParser::get_row_group_offset(&footer, 0)?, 32);
        assert_eq!(FooterParser::get_row_group_offset(&footer, 1)?, 1024);
        assert_eq!(FooterParser::get_row_group_offset(&footer, 2)?, 2048);

        Ok(())
    }

    #[test]
    fn test_row_group_range() -> Result<()> {
        let footer = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: vec![32, 1024, 2048],
            row_count: 3000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0,
        };

        let (start, end) = FooterParser::get_row_group_range(&footer, 0, 5000)?;
        assert_eq!(start, 32);
        assert_eq!(end, 1024);

        Ok(())
    }
}
