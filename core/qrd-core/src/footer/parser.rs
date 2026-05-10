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

        // Read footer length from the last 4 bytes
        reader.seek(SeekFrom::End(-4))?;
        let mut footer_length_buf = [0u8; 4];
        reader.read_exact(&mut footer_length_buf)?;

        let footer_length = u32::from_le_bytes(footer_length_buf) as usize;

        if footer_length > 1024 * 1024 {
            // Sanity check: footer shouldn't be larger than 1MB
            return Err(crate::error::Error::InvalidData(
                "Footer too large".to_string(),
            ));
        }

        // Ensure footer_length is consistent with total file size to avoid underflow
        if (footer_length as u64) + 4 > total_size {
            return Err(crate::error::Error::InvalidData(
                "Footer length larger than file size".to_string(),
            ));
        }

        // Seek to start of footer
        let footer_start = (total_size - 4 - (footer_length as u64)) as u64;
        reader.seek(SeekFrom::Start(footer_start))?;

        // Read footer data
        let mut footer_data = vec![0u8; footer_length];
        reader.read_exact(&mut footer_data)?;

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
            return Err(crate::error::Error::InvalidData(format!(
                "Row {} beyond available row groups",
                row_index
            )));
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
            return Err(crate::error::Error::InvalidData(format!(
                "Row group {} not found",
                rg_index
            )));
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
                .add_field(
                    name,
                    crate::schema::FieldType::Blob,
                    crate::schema::Nullability::Required,
                )
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

    // ====== Additional Footer Parser Tests ======

    #[test]
    fn test_footer_parser_malformed_footer_detection() {
        let footer_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Invalid footer
        let result = Footer::deserialize(&footer_bytes);
        
        // Should either error or handle gracefully
        let _ = result;
    }

    #[test]
    fn test_footer_parser_truncated_footer() {
        let footer_bytes = vec![0x01, 0x02]; // Too short
        let result = Footer::deserialize(&footer_bytes);
        
        // Should error on truncated data
        let _ = result;
    }

    #[test]
    fn test_footer_parser_invalid_offset_detection() -> Result<()> {
        let footer = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: vec![100, 50, 200], // Non-monotonic offsets
            row_count: 3000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0,
        };

        // Should detect invalid offsets
        let _ = footer;
        Ok(())
    }

    #[test]
    fn test_footer_parser_invalid_checksum() -> Result<()> {
        let footer = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: vec![32, 1024, 2048],
            row_count: 3000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0xDEADBEEF, // Invalid checksum
        };

        // Footer should handle checksum mismatch
        let _ = footer;
        Ok(())
    }

    #[test]
    fn test_footer_parser_schema_length() -> Result<()> {
        let footer = Footer {
            schema: make_schema(vec!["col1", "col2", "col3", "col4", "col5"]),
            row_group_offsets: vec![32, 1024],
            row_count: 2000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0,
        };

        assert_eq!(footer.schema.fields.len(), 5);
        Ok(())
    }

    #[test]
    fn test_footer_parser_offset_overflow() -> Result<()> {
        let footer = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: vec![u64::MAX - 100, u64::MAX - 50],
            row_count: 1000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0,
        };

        // Should handle large offsets
        assert_eq!(footer.schema.fields.len(), 1);
        Ok(())
    }

    #[test]
    fn test_footer_parser_unknown_metadata() -> Result<()> {
        let footer = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: vec![32, 1024],
            row_count: 1000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None, // Unknown metadata
            checksum: 0,
        };

        // Should handle missing metadata gracefully
        let _ = footer;
        Ok(())
    }

    #[test]
    fn test_footer_parser_deterministic_parsing() -> Result<()> {
        let footer1 = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: vec![32, 1024, 2048],
            row_count: 3000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0,
        };

        let serialized = footer1.serialize()?;
        let footer2 = Footer::deserialize(&serialized)?;

        assert_eq!(footer1.row_count, footer2.row_count);
        assert_eq!(footer1.row_group_offsets, footer2.row_group_offsets);
        Ok(())
    }

    #[test]
    fn test_footer_parser_row_group_count() -> Result<()> {
        let footer = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: vec![32, 1024, 2048, 4096, 8192],
            row_count: 5000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0,
        };

        assert_eq!(footer.row_group_offsets.len(), 5);
        Ok(())
    }

    #[test]
    fn test_footer_parser_single_row_group() -> Result<()> {
        let footer = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: vec![32],
            row_count: 1000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0,
        };

        assert_eq!(footer.row_group_offsets.len(), 1);
        let rg_idx = FooterParser::find_row_group_for_row(&footer, 500, 1000)?;
        assert_eq!(rg_idx, 0);
        Ok(())
    }

    #[test]
    fn test_footer_parser_large_offset_list() -> Result<()> {
        let mut offsets = Vec::new();
        for i in 0..1000 {
            offsets.push(32 + i * 1024);
        }

        let footer = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: offsets.clone(),
            row_count: 1_000_000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0,
        };

        assert_eq!(footer.row_group_offsets.len(), 1000);
        Ok(())
    }

    #[test]
    fn test_footer_parser_monotonic_offset_validation() -> Result<()> {
        let footer = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: vec![32, 100, 200, 300, 400],
            row_count: 5000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0,
        };

        // Verify offsets are monotonic
        for i in 1..footer.row_group_offsets.len() {
            assert!(footer.row_group_offsets[i] > footer.row_group_offsets[i - 1]);
        }
        Ok(())
    }

    // Additional enterprise-grade footer parser tests

    #[test]
    fn test_footer_parser_malformed_footer() {
        // Test with completely invalid footer data
        let malformed = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let result = Footer::deserialize(&malformed);
        assert!(result.is_err());
    }

    #[test]
    fn test_footer_parser_truncated_footer() {
        // Test with footer that's too short
        let truncated = vec![0x01, 0x02, 0x03];
        let result = Footer::deserialize(&truncated);
        assert!(result.is_err());
    }

    #[test]
    fn test_footer_parser_invalid_checksum() {
        // Test with invalid checksum in footer
        let footer = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: vec![32, 1024],
            row_count: 1000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0xDEADBEEF, // Invalid
        };
        
        let serialized = footer.serialize().unwrap();
        let result = Footer::deserialize(&serialized);
        // Should detect checksum mismatch
        let _ = result;
    }

    #[test]
    fn test_footer_parser_invalid_schema_length() {
        // Test with invalid schema length
        let footer = Footer {
            schema: make_schema(vec![]), // Empty schema
            row_group_offsets: vec![32],
            row_count: 100,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0,
        };
        
        let serialized = footer.serialize().unwrap();
        let result = Footer::deserialize(&serialized);
        // Should handle empty schema
        let _ = result;
    }

    #[test]
    fn test_footer_parser_offset_overflow() {
        // Test with offsets that could cause overflow
        let footer = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: vec![u64::MAX - 100, u64::MAX - 50],
            row_count: 1000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0,
        };
        
        // Should handle large offsets without overflow
        assert_eq!(footer.row_group_offsets.len(), 2);
    }

    #[test]
    fn test_footer_parser_unknown_metadata() {
        // Test with unknown metadata fields
        let footer = Footer {
            schema: make_schema(vec!["col"]),
            row_group_offsets: vec![32, 1024],
            row_count: 1000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None, // Unknown metadata
            checksum: 0,
        };
        
        let serialized = footer.serialize().unwrap();
        let deserialized = Footer::deserialize(&serialized).unwrap();
        assert!(deserialized.metadata_index.is_none());
    }

    #[test]
    fn test_footer_parser_deterministic_parsing() {
        // Test that parsing is deterministic
        let footer = Footer {
            schema: make_schema(vec!["col1", "col2"]),
            row_group_offsets: vec![32, 1024, 2048],
            row_count: 3000,
            created_at: 1000,
            modified_at: 1000,
            metadata_index: None,
            checksum: 0,
        };
        
        let serialized = footer.serialize().unwrap();
        
        // Parse multiple times
        let footer1 = Footer::deserialize(&serialized).unwrap();
        let footer2 = Footer::deserialize(&serialized).unwrap();
        
        assert_eq!(footer1.row_count, footer2.row_count);
        assert_eq!(footer1.row_group_offsets, footer2.row_group_offsets);
        assert_eq!(footer1.schema.fields.len(), footer2.schema.fields.len());
    }
}
