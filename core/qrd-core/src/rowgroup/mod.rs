//! Row group management and processing

use crate::columnar::ColumnChunk;
use crate::compression::{compress, decompress, CompressionCodec, CompressionLevel};
use crate::encoding::EncodingType;
use crate::error::Result;
use crate::validation::Validator;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use std::io::{Cursor, Read};

/// Encoded and compressed column chunk
#[derive(Debug, Clone)]
pub struct EncodedColumnChunk {
    /// Column index
    pub column_index: usize,
    /// Column name
    pub column_name: String,
    /// Encoding used
    pub encoding: EncodingType,
    /// Compression codec used
    pub compression: CompressionCodec,
    /// Uncompressed data
    pub encoded_data: Vec<u8>,
    /// Compressed data
    pub compressed_data: Vec<u8>,
    /// CRC32 of uncompressed data
    pub crc32: u32,
}

impl EncodedColumnChunk {
    /// Calculate total size
    pub fn total_size(&self) -> u32 {
        (2 + 4 + 4 + 4 + 4 + self.compressed_data.len() + 4) as u32
    }
}

/// Manages a single row group
pub struct RowGroup {
    /// Row count
    pub row_count: u32,
    /// Encoded columns
    pub columns: Vec<EncodedColumnChunk>,
    /// Optional per-column statistics collected during writing
    pub column_stats: Option<Vec<crate::metadata::ColumnStats>>,
}

impl RowGroup {
    /// Create new row group
    pub fn new(row_count: u32) -> Self {
        RowGroup {
            row_count,
            columns: Vec::new(),
            column_stats: None,
        }
    }

    /// Process column chunk (encode + compress)
    pub fn process_column(
        &mut self,
        chunk: ColumnChunk,
        encoding: EncodingType,
        compression: CompressionCodec,
        compression_level: CompressionLevel,
    ) -> Result<()> {
        // Apply encoding
        let encoder = crate::encoding::get_encoder(encoding)?;
        let encoded_data = encoder.encode(&chunk.data)?;

        // Calculate CRC32 of encoded data
        let crc32 = Validator::calculate_crc32(&encoded_data);

        // Apply compression
        let compressed_data = compress(&encoded_data, compression, compression_level)?;

        let encoded_column = EncodedColumnChunk {
            column_index: chunk.column_index,
            column_name: chunk.column_name,
            encoding,
            compression,
            encoded_data,
            compressed_data,
            crc32,
        };

        self.columns.push(encoded_column);
        Ok(())
    }

    /// Serialize row group to bytes
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut result = Vec::new();

        // Row group header
        result.write_u32::<LittleEndian>(self.row_count)?;

        // Calculate total sizes
        let total_uncompressed: u32 =
            self.columns.iter().map(|c| c.encoded_data.len() as u32).sum();
        let total_compressed: u32 =
            self.columns.iter().map(|c| c.compressed_data.len() as u32).sum();

        result.write_u32::<LittleEndian>(total_uncompressed)?;
        result.write_u32::<LittleEndian>(total_compressed)?;
        result.write_u16::<LittleEndian>(self.columns.len() as u16)?;

        // Column chunks
        for column in &self.columns {
            result.push(column.encoding.to_id());
            result.push(column.compression.to_id());
            result.write_u32::<LittleEndian>(column.encoded_data.len() as u32)?;
            result.write_u32::<LittleEndian>(column.compressed_data.len() as u32)?;
            // Write statistics if available
            if let Some(ref stats_vec) = self.column_stats {
                let col_idx = column.column_index;
                if let Some(col_stats) = stats_vec.get(col_idx) {
                    result.write_u32::<LittleEndian>(col_stats.null_count as u32)?;
                    result.write_u32::<LittleEndian>(col_stats.distinct_count as u32)?;
                } else {
                    result.write_u32::<LittleEndian>(0)?;
                    result.write_u32::<LittleEndian>(0)?;
                }
            } else {
                result.write_u32::<LittleEndian>(0)?; // null_count (placeholder)
                result.write_u32::<LittleEndian>(0)?; // distinct_count (placeholder)
            }
            result.extend_from_slice(&column.compressed_data);
            result.write_u32::<LittleEndian>(column.crc32)?;
        }

        Ok(result)
    }

    /// Deserialize row group from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);

        let row_count = cursor.read_u32::<LittleEndian>()?;
        let _total_uncompressed = cursor.read_u32::<LittleEndian>()?;
        let _total_compressed = cursor.read_u32::<LittleEndian>()?;
        let column_count = cursor.read_u16::<LittleEndian>()? as usize;

        let mut row_group = RowGroup::new(row_count);

        for col_idx in 0..column_count {
            let mut id_buf = [0u8; 1];
            cursor.read_exact(&mut id_buf)?;
            let encoding_id = id_buf[0];
            
            cursor.read_exact(&mut id_buf)?;
            let compression_id = id_buf[0];

            let encoding = EncodingType::from_id(encoding_id)?;
            
            let compression = CompressionCodec::from_id(compression_id)
                .ok_or_else(|| crate::error::Error::InvalidData(
                    format!("Unknown compression ID: {}", compression_id)
                ))?;

            let _encoded_len = cursor.read_u32::<LittleEndian>()? as usize;
            let compressed_len = cursor.read_u32::<LittleEndian>()? as usize;
            let _null_count = cursor.read_u32::<LittleEndian>()?;
            let _distinct_count = cursor.read_u32::<LittleEndian>()?;

            // Read compressed data
            let mut compressed_data = vec![0u8; compressed_len];
            cursor.read_exact(&mut compressed_data)?;

            // Decompress
            let encoded_data = decompress(&compressed_data, compression)?;

            // Read CRC32
            let crc32 = cursor.read_u32::<LittleEndian>()?;

            let encoded_column = EncodedColumnChunk {
                column_index: col_idx,
                column_name: format!("col_{}", col_idx),
                encoding,
                compression,
                encoded_data,
                compressed_data,
                crc32,
            };

            row_group.columns.push(encoded_column);
        }

        Ok(row_group)
    }

    /// Decode all columns in the row group
    pub fn decode_columns(&self) -> Result<Vec<Vec<u8>>> {
        let mut result = Vec::with_capacity(self.columns.len());

        for column in &self.columns {
            let encoder = crate::encoding::get_encoder(column.encoding)?;
            let decoded = encoder.decode(&column.encoded_data, self.row_count as usize)?;
            result.push(decoded);
        }

        Ok(result)
    }

    /// Calculate row group size in bytes
    pub fn size_bytes(&self) -> u32 {
        self.columns.iter().map(|c| c.total_size()).sum::<u32>() + 16 + 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_group_creation() {
        let rg = RowGroup::new(1000);
        assert_eq!(rg.row_count, 1000);
        assert!(rg.columns.is_empty());
    }

    #[test]
    fn test_encoded_column_chunk_size() {
        let chunk = EncodedColumnChunk {
            column_index: 0,
            column_name: "test".to_string(),
            encoding: EncodingType::Plain,
            compression: CompressionCodec::None,
            encoded_data: vec![1, 2, 3, 4, 5],
            compressed_data: vec![1, 2, 3, 4, 5],
            crc32: 0x12345678,
        };

        let size = chunk.total_size();
        assert!(size > 0);

    }

    #[test]
    fn test_row_group_serialization_round_trip() {
        let mut rg = RowGroup::new(10);
        
        let chunk = EncodedColumnChunk {
            column_index: 0,
            column_name: "test".to_string(),
            encoding: EncodingType::Plain,
            compression: CompressionCodec::None,
            encoded_data: vec![1, 2, 3, 4, 5],
            compressed_data: vec![1, 2, 3, 4, 5],
            crc32: 0x12345678,
        };
        
        rg.columns.push(chunk);
        
        let serialized = rg.serialize().unwrap();
        let deserialized = RowGroup::deserialize(&serialized).unwrap();
        
        assert_eq!(deserialized.row_count, 10);
        assert_eq!(deserialized.columns.len(), 1);
        assert_eq!(deserialized.columns[0].column_index, 0);
    }
}
