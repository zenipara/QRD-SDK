//! Error Correction Code (Reed-Solomon)

use crate::error::{Error, Result};
use reed_solomon_erasure::galois_8::ReedSolomon;
use std::cmp;

/// ECC configuration
#[derive(Debug, Clone)]
pub struct EccConfig {
    /// Number of parity chunks (1-32)
    pub parity_chunks: u8,
    /// Data chunk size in bytes
    pub chunk_size: usize,
}

impl EccConfig {
    /// Create with parity count and default chunk size
    pub fn new(parity_chunks: u8) -> Result<Self> {
        Self::with_chunk_size(parity_chunks, 4096)
    }

    /// Create with parity count and custom chunk size
    pub fn with_chunk_size(parity_chunks: u8, chunk_size: usize) -> Result<Self> {
        if parity_chunks == 0 || parity_chunks > 32 {
            return Err(Error::ConfigError(
                "Parity chunks must be between 1 and 32".to_string(),
            ));
        }
        if chunk_size == 0 || chunk_size > 65536 {
            return Err(Error::ConfigError(
                "Chunk size must be between 1 and 65536 bytes".to_string(),
            ));
        }
        Ok(EccConfig {
            parity_chunks,
            chunk_size,
        })
    }

    /// Calculate total shards needed (data + parity)
    pub fn total_shards(&self, data_shards: usize) -> usize {
        data_shards + self.parity_chunks as usize
    }

    /// Calculate maximum data shards that can be recovered
    pub fn max_data_shards(&self, total_shards: usize) -> usize {
        total_shards.saturating_sub(self.parity_chunks as usize)
    }
}

/// ECC encoder/decoder
pub struct EccCodec {
    rs: ReedSolomon,
    config: EccConfig,
}

impl EccCodec {
    /// Create a new ECC codec
    pub fn new(config: EccConfig) -> Result<Self> {
        // For Reed-Solomon, we need at least 1 data shard
        // We'll determine the actual data shard count during encoding
        Ok(EccCodec {
            rs: ReedSolomon::new(1, config.parity_chunks as usize).map_err(|e| {
                Error::EccError(format!("Failed to create Reed-Solomon codec: {}", e))
            })?,
            config,
        })
    }

    /// Encode data with ECC
    pub fn encode(&mut self, data: &[u8]) -> Result<EccEncodedData> {
        // Split data into chunks
        let data_shards = (data.len() + self.config.chunk_size - 1) / self.config.chunk_size;
        let total_shards = self.config.total_shards(data_shards);

        // Create shards
        let mut shards = Vec::with_capacity(total_shards);

        // Split data into data shards
        for i in 0..data_shards {
            let start = i * self.config.chunk_size;
            let end = cmp::min(start + self.config.chunk_size, data.len());
            let mut chunk = data[start..end].to_vec();

            // Pad chunk to chunk_size if necessary
            while chunk.len() < self.config.chunk_size {
                chunk.push(0);
            }
            shards.push(chunk);
        }

        // Add empty parity shards
        for _ in 0..self.config.parity_chunks {
            shards.push(vec![0; self.config.chunk_size]);
        }

        // Reconstruct Reed-Solomon codec with correct data shard count
        self.rs =
            ReedSolomon::new(data_shards, self.config.parity_chunks as usize).map_err(|e| {
                Error::EccError(format!("Failed to reconstruct Reed-Solomon codec: {}", e))
            })?;

        // Encode parity
        self.rs
            .encode(&mut shards)
            .map_err(|e| Error::EccError(format!("Failed to encode parity: {}", e)))?;

        Ok(EccEncodedData {
            shards,
            data_shards,
            parity_shards: self.config.parity_chunks as usize,
            chunk_size: self.config.chunk_size,
            original_size: data.len(),
        })
    }

    /// Decode and recover data from ECC shards (takes EccEncodedData to access original_size)
    pub fn decode_and_recover(&self, encoded_data: &EccEncodedData) -> Result<Vec<u8>> {
        let shards = encoded_data.shards_as_options();
        if shards.is_empty() {
            return Err(Error::EccError("No shards provided".to_string()));
        }

        let total_shards = shards.len();
        let data_shards = self.config.max_data_shards(total_shards);

        // Convert Option<Vec<u8>> to Vec<Option<Vec<u8>>>
        let mut shard_data: Vec<Option<Vec<u8>>> = shards.iter().cloned().collect();

        // Reconstruct Reed-Solomon codec
        let rs =
            ReedSolomon::new(data_shards, self.config.parity_chunks as usize).map_err(|e| {
                Error::EccError(format!(
                    "Failed to create Reed-Solomon codec for decoding: {}",
                    e
                ))
            })?;

        // Reconstruct data
        rs.reconstruct(&mut shard_data)
            .map_err(|e| Error::EccError(format!("Failed to reconstruct data: {}", e)))?;

        // Collect recovered data
        let mut recovered_data = Vec::new();
        for i in 0..data_shards {
            if let Some(shard) = &shard_data[i] {
                recovered_data.extend_from_slice(shard);
            }
        }

        // Truncate to original size - THIS IS CRITICAL FOR CORRECT RECOVERY
        recovered_data.truncate(encoded_data.original_size);
        Ok(recovered_data)
    }
}

/// Encoded data with ECC information
#[derive(Debug, Clone)]
pub struct EccEncodedData {
    /// All shards (data + parity)
    pub shards: Vec<Vec<u8>>,
    /// Number of data shards
    pub data_shards: usize,
    /// Number of parity shards
    pub parity_shards: usize,
    /// Size of each chunk
    pub chunk_size: usize,
    /// Original data size
    pub original_size: usize,
}

impl EccEncodedData {
    /// Get total number of shards
    pub fn total_shards(&self) -> usize {
        self.data_shards + self.parity_shards
    }

    /// Get shard at index
    pub fn shard(&self, index: usize) -> Option<&[u8]> {
        self.shards.get(index).map(|v| v.as_slice())
    }

    /// Get all shards as options (for simulating missing shards)
    pub fn shards_as_options(&self) -> Vec<Option<Vec<u8>>> {
        self.shards.iter().map(|s| Some(s.clone())).collect()
    }

    /// Simulate missing shards
    pub fn with_missing_shards(&self, missing_indices: &[usize]) -> Vec<Option<Vec<u8>>> {
        let mut result = self.shards_as_options();
        for &index in missing_indices {
            if index < result.len() {
                result[index] = None;
            }
        }
        result
    }

    /// Reconstruct original data
    pub fn reconstruct_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for i in 0..self.data_shards {
            let shard = &self.shards[i];
            let actual_size = if i == self.data_shards - 1 {
                // Last shard may contain original data size info
                self.original_size.saturating_sub(i * self.chunk_size)
            } else {
                self.chunk_size
            };
            data.extend_from_slice(&shard[..actual_size]);
        }
        data.truncate(self.original_size);
        data
    }

    /// Serialize EccEncodedData to bytes for storage
    /// Format: [8B original_size][4B data_shards][4B parity_shards][4B chunk_size][4B total_shards]
    ///         [for each shard: [4B shard_len][shard data]]
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        use byteorder::{LittleEndian, WriteBytesExt};
        use std::io::Write;

        let mut result = Vec::new();

        // Metadata header
        result.write_u64::<LittleEndian>(self.original_size as u64)?;
        result.write_u32::<LittleEndian>(self.data_shards as u32)?;
        result.write_u32::<LittleEndian>(self.parity_shards as u32)?;
        result.write_u32::<LittleEndian>(self.chunk_size as u32)?;
        result.write_u32::<LittleEndian>(self.total_shards() as u32)?;

        // Write each shard with its length
        for shard in &self.shards {
            result.write_u32::<LittleEndian>(shard.len() as u32)?;
            result.write_all(shard)?;
        }

        Ok(result)
    }

    /// Deserialize EccEncodedData from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        use byteorder::{LittleEndian, ReadBytesExt};
        use std::io::Cursor;

        if data.len() < 24 {
            return Err(Error::EccError(
                "EccEncodedData bytes too short for header".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);

        // Read metadata header
        let original_size = cursor.read_u64::<LittleEndian>()? as usize;
        let data_shards = cursor.read_u32::<LittleEndian>()? as usize;
        let parity_shards = cursor.read_u32::<LittleEndian>()? as usize;
        let chunk_size = cursor.read_u32::<LittleEndian>()? as usize;
        let total_shards = cursor.read_u32::<LittleEndian>()? as usize;

        // Validate basic header values
        if data_shards == 0 {
            return Err(Error::EccError(
                "EccEncodedData: data_shards must be > 0".to_string(),
            ));
        }

        if data_shards + parity_shards != total_shards {
            return Err(Error::EccError(
                "EccEncodedData: shard count mismatch".to_string(),
            ));
        }

        if chunk_size == 0 || chunk_size > 65536 {
            return Err(Error::EccError(
                "EccEncodedData: invalid chunk_size".to_string(),
            ));
        }

        // Read shards
        let mut shards = Vec::with_capacity(total_shards);
        for _ in 0..total_shards {
            if cursor.position() + 4 > data.len() as u64 {
                return Err(Error::EccError(
                    "EccEncodedData: truncated shard header".to_string(),
                ));
            }

            let shard_len = cursor.read_u32::<LittleEndian>()? as usize;

            // Each shard length should not exceed declared chunk_size
            if shard_len > chunk_size {
                return Err(Error::EccError(
                    "EccEncodedData: shard length larger than chunk_size".to_string(),
                ));
            }

            if cursor.position() as usize + shard_len > data.len() {
                return Err(Error::EccError(
                    "EccEncodedData: truncated shard data".to_string(),
                ));
            }

            let start = cursor.position() as usize;
            let shard_data = data[start..start + shard_len].to_vec();
            cursor.set_position((start + shard_len) as u64);

            shards.push(shard_data);
        }

        // Sanity check: original_size must not exceed total declared data capacity
        let max_possible = data_shards.saturating_mul(chunk_size);
        if original_size > max_possible {
            return Err(Error::EccError(
                "EccEncodedData: original_size larger than data capacity".to_string(),
            ));
        }

        Ok(EccEncodedData {
            shards,
            data_shards,
            parity_shards,
            chunk_size,
            original_size,
        })
    }
}

/// Convenience functions for simple encoding/decoding

/// Encode data with ECC
pub fn encode(data: &[u8], config: &EccConfig) -> Result<EccEncodedData> {
    let mut codec = EccCodec::new(config.clone())?;
    codec.encode(data)
}

/// Decode and recover data from encoded data
pub fn decode_and_recover(encoded_data: &EccEncodedData, config: &EccConfig) -> Result<Vec<u8>> {
    let codec = EccCodec::new(config.clone())?;
    codec.decode_and_recover(encoded_data)
}

/// Decode and recover using an external shard options view (some shards may be None)
pub fn decode_and_recover_with_options(
    encoded: &EccEncodedData,
    shard_options: &[Option<Vec<u8>>],
) -> Result<Vec<u8>> {
    if shard_options.len() != encoded.total_shards() {
        return Err(Error::EccError("Shard count mismatch".to_string()));
    }

    // Make a mutable copy for reconstruction
    let mut shard_data: Vec<Option<Vec<u8>>> = shard_options.to_vec();

    let rs = ReedSolomon::new(encoded.data_shards, encoded.parity_shards)
        .map_err(|e| Error::EccError(format!("Failed to create Reed-Solomon codec: {}", e)))?;

    rs.reconstruct(&mut shard_data)
        .map_err(|e| Error::EccError(format!("Failed to reconstruct data: {}", e)))?;

    // Collect recovered data from first data_shards
    let mut recovered = Vec::new();
    for i in 0..encoded.data_shards {
        if let Some(shard) = &shard_data[i] {
            recovered.extend_from_slice(shard);
        }
    }

    recovered.truncate(encoded.original_size);
    Ok(recovered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecc_config() {
        let config = EccConfig::new(4).unwrap();
        assert_eq!(config.parity_chunks, 4);
        assert_eq!(config.chunk_size, 4096);
    }

    #[test]
    fn test_ecc_config_custom_chunk_size() {
        let config = EccConfig::with_chunk_size(2, 1024).unwrap();
        assert_eq!(config.parity_chunks, 2);
        assert_eq!(config.chunk_size, 1024);
    }

    #[test]
    fn test_invalid_parity_chunks() {
        assert!(EccConfig::new(0).is_err());
        assert!(EccConfig::new(33).is_err());
    }

    #[test]
    fn test_invalid_chunk_size() {
        assert!(EccConfig::with_chunk_size(4, 0).is_err());
        assert!(EccConfig::with_chunk_size(4, 70000).is_err());
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let config = EccConfig::with_chunk_size(2, 64).unwrap();
        let original_data =
            b"Hello, QRD ECC world! This is a test message for Reed-Solomon encoding.";

        let encoded = encode(original_data, &config).unwrap();
        assert_eq!(encoded.parity_shards, 2);

        let recovered = decode_and_recover(&encoded, &config).unwrap();

        // Should recover original data
        assert_eq!(&recovered[..original_data.len()], original_data);
    }

    #[test]
    fn test_recovery_from_missing_shards() {
        let config = EccConfig::with_chunk_size(4, 64).unwrap();
        let original_data = b"Test data for ECC recovery testing with missing shards";

        let encoded = encode(original_data, &config).unwrap();

        // Simulate losing 2 data shards (should still recover with 4 parity)
        let missing_indices = vec![0, 2]; // Missing first and third data shards
        let damaged_shards = encoded.with_missing_shards(&missing_indices);

        let recovered = decode_and_recover_with_options(&encoded, &damaged_shards).unwrap();
        assert_eq!(&recovered[..original_data.len()], original_data);
    }

    #[test]
    fn test_reconstruct_data() {
        let config = EccConfig::with_chunk_size(2, 64).unwrap();
        let original_data = b"Short test";

        let encoded = encode(original_data, &config).unwrap();
        let reconstructed = encoded.reconstruct_data();

        assert_eq!(reconstructed, original_data);
    }

    #[test]
    fn test_large_data_chunking() {
        let config = EccConfig::with_chunk_size(2, 1024).unwrap();
        let original_data = vec![42u8; 3000]; // Larger than chunk size

        let encoded = encode(&original_data, &config).unwrap();
        assert!(encoded.data_shards > 1); // Should be split into multiple chunks

        let recovered = decode_and_recover(&encoded, &config).unwrap();

        assert_eq!(&recovered[..original_data.len()], original_data.as_slice());
    }
}
