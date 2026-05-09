//! Entropy detection and analysis for adaptive compression
//!
//! Detects if data is already compressed/random and skips compression
//! to avoid negative compression ratios.

/// Entropy calculator using Shannon entropy
pub struct EntropyCalculator;

impl EntropyCalculator {
    /// Calculate Shannon entropy (0.0-8.0 for byte data)
    ///
    /// High entropy (>7.5) indicates random/already-compressed data
    /// Low entropy (<4.0) indicates highly repetitive data
    pub fn calculate(data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        // Frequency analysis
        let mut freq = [0usize; 256];
        for &byte in data {
            freq[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;

        for count in freq.iter() {
            if *count > 0 {
                let p = *count as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Quick entropy sample (for large data, sample first N bytes)
    pub fn sample_entropy(data: &[u8], sample_size: usize) -> f64 {
        let sample = if data.len() > sample_size {
            &data[..sample_size]
        } else {
            data
        };
        Self::calculate(sample)
    }

    /// Estimate compression ratio based on entropy
    ///
    /// Returns ratio estimate (0.0-1.0)
    /// High entropy → ratio near 1.0 (incompressible)
    /// Low entropy → ratio near 0.3-0.5 (compressible)
    pub fn estimate_compression_ratio(entropy: f64) -> f64 {
        // Empirical model: high entropy = low compression
        // Uses sigmoid curve to smooth transition
        let normalized = entropy / 8.0; // 8.0 is max entropy for bytes
        1.0 / (1.0 + (-10.0 * (normalized - 0.85)).exp()) // Inflection at 0.85
    }

    /// Detect if data is likely already compressed
    ///
    /// Returns true if data appears to be compressed/random
    pub fn is_likely_compressed(data: &[u8]) -> bool {
        let entropy = Self::calculate(data);
        entropy > 7.4 // Near-random entropy threshold
    }

    /// Detect specific compressed formats by magic bytes
    pub fn has_compression_magic(data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }

        // Common compression magic bytes
        match (&data[0..2], &data[0..4]) {
            // GZIP
            (_, [0x1f, 0x8b, ..]) => true,
            // ZIP
            (_, [0x50, 0x4b, 0x03, 0x04]) => true,
            // ZSTD
            (_, [0x28, 0xb5, 0x2f, 0xfd]) => true,
            // LZ4 frame
            (_, [0x04, 0x22, 0x4d, 0x18]) => true,
            // RAR
            (_, [0x52, 0x61, 0x72, 0x21]) => true,
            // 7-Zip
            (_, [0x37, 0x7a, 0xbc, 0xaf]) => true,
            // BZIP2
            ([0x42, 0x5a, ..], _) => true,
            // LZMA
            ([0x5d, 0x00, ..], _) => data.len() > 12, // LZMA has longer magic
            _ => false,
        }
    }
}

/// Compression selector with adaptive logic
pub struct CompressionSelector;

impl CompressionSelector {
    /// Decide whether to compress and which codec to use
    ///
    /// Returns:
    /// - None if data should not be compressed
    /// - Some(codec) if data should be compressed
    pub fn should_compress(data: &[u8]) -> Option<crate::compression::CompressionCodec> {
        // Don't compress if data is too small
        if data.len() < 1024 {
            return None;
        }

        // Don't compress already-compressed data
        if EntropyCalculator::has_compression_magic(data) {
            return None;
        }

        // Sample entropy for large data
        let sample_size = 8192.min(data.len());
        let entropy = if data.len() > sample_size {
            EntropyCalculator::sample_entropy(data, sample_size)
        } else {
            EntropyCalculator::calculate(data)
        };

        // Check if likely already compressed
        if entropy > 7.4 {
            return None;
        }

        // Use ZSTD for general purpose (good ratio + speed)
        // Could switch to LZ4 for very high throughput needs
        Some(crate::compression::CompressionCodec::Zstd)
    }

    /// Estimate if compression is worthwhile
    ///
    /// Returns true if estimated ratio < PASSTHROUGH_THRESHOLD
    pub fn is_compressible(data: &[u8]) -> bool {
        if EntropyCalculator::has_compression_magic(data) {
            return false;
        }

        let entropy = EntropyCalculator::calculate(data);
        let ratio = EntropyCalculator::estimate_compression_ratio(entropy);

        ratio < 0.98 // Don't compress if ratio > 98%
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_calculation() {
        // Repetitive data should have low entropy
        let repetitive = b"aaaaaaaaaa";
        let entropy = EntropyCalculator::calculate(repetitive);
        assert!(entropy < 1.0);

        // Random data should have high entropy
        let random = vec![0u8, 1, 2, 3, 4, 5, 127, 128, 200, 255];
        let entropy = EntropyCalculator::calculate(&random);
        assert!(entropy > 3.0);
    }

    #[test]
    fn test_compression_ratio_estimation() {
        let low_entropy_ratio = EntropyCalculator::estimate_compression_ratio(2.0);
        let high_entropy_ratio = EntropyCalculator::estimate_compression_ratio(7.5);

        assert!(low_entropy_ratio < high_entropy_ratio);
        assert!(low_entropy_ratio < 0.5);
        assert!(high_entropy_ratio > 0.95);
    }

    #[test]
    fn test_magic_byte_detection() {
        // GZIP magic
        let gzip = vec![0x1f, 0x8b, 0x08, 0x00];
        assert!(EntropyCalculator::has_compression_magic(&gzip));

        // ZSTD magic
        let zstd = vec![0x28, 0xb5, 0x2f, 0xfd];
        assert!(EntropyCalculator::has_compression_magic(&zstd));

        // Regular data
        let regular = b"hello world";
        assert!(!EntropyCalculator::has_compression_magic(regular));
    }

    #[test]
    fn test_compression_selector() {
        // Compressible data
        let compressible = vec![1u8; 10000];
        assert!(matches!(
            CompressionSelector::should_compress(&compressible),
            Some(crate::compression::CompressionCodec::Zstd)
        ));

        // Too small
        let small = b"tiny";
        assert!(CompressionSelector::should_compress(small).is_none());

        // Already compressed
        let mut already_compressed = vec![0x1f, 0x8b, 0x08, 0x00];
        already_compressed.resize(1024, 0);
        assert!(CompressionSelector::should_compress(&already_compressed).is_none());
    }
}
