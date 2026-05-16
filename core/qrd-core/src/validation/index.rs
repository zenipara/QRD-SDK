//! Index and Bloom Filter Implementation for Predicate Pushdown
//!
//! Provides:
//! - Bloom filters for efficient membership testing
//! - Index structures (B-tree style, hash-based)
//! - Predicate evaluation and pushdown optimization
//! - Serialization/deserialization for metadata storage

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Bloom filter for probabilistic set membership testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloomFilter {
    /// Bit vector for the filter
    bits: Vec<u8>,
    /// Number of bits
    size: usize,
    /// Number of hash functions
    num_hashes: usize,
}

impl BloomFilter {
    /// Create a new bloom filter with approximate size (in bytes)
    pub fn new(expected_elements: usize, false_positive_rate: f64) -> Self {
        // Calculate optimal bit array size
        // m = -1 / ln(2)^2 * n * ln(p)
        let ln2_sq = std::f64::consts::LN_2 * std::f64::consts::LN_2;
        let size_bits = (-1.0 / ln2_sq * expected_elements as f64 * false_positive_rate.ln()).ceil() as usize;
        let size_bytes = (size_bits + 7) / 8;

        // Calculate optimal number of hash functions
        // k = m / n * ln(2)
        let num_hashes = ((size_bytes as f64 * 8.0) / expected_elements as f64 * std::f64::consts::LN_2).ceil() as usize;
        let num_hashes = std::cmp::max(1, std::cmp::min(num_hashes, 16)); // Clamp to reasonable range

        BloomFilter {
            bits: vec![0u8; size_bytes],
            size: size_bytes * 8,
            num_hashes,
        }
    }

    /// Add an element to the filter
    pub fn insert(&mut self, element: &[u8]) {
        for i in 0..self.num_hashes {
            let hash = self.hash(element, i);
            let bit_index = hash % self.size;
            let byte_index = bit_index / 8;
            let bit_position = bit_index % 8;
            self.bits[byte_index] |= 1 << bit_position;
        }
    }

    /// Check if an element might be in the filter
    pub fn contains(&self, element: &[u8]) -> bool {
        for i in 0..self.num_hashes {
            let hash = self.hash(element, i);
            let bit_index = hash % self.size;
            let byte_index = bit_index / 8;
            let bit_position = bit_index % 8;
            if (self.bits[byte_index] & (1 << bit_position)) == 0 {
                return false;
            }
        }
        true
    }

    /// Get the size in bytes
    pub fn size_bytes(&self) -> usize {
        self.bits.len()
    }

    /// Get statistics about the filter
    pub fn statistics(&self) -> BloomFilterStats {
        let bits_set = self.bits.iter().map(|b| b.count_ones() as usize).sum::<usize>();
        let fill_ratio = bits_set as f64 / self.size as f64;

        BloomFilterStats {
            size_bytes: self.bits.len(),
            num_bits: self.size,
            num_hashes: self.num_hashes,
            bits_set,
            fill_ratio,
        }
    }

    /// Hash a value with a specific hash function index
    fn hash(&self, element: &[u8], index: usize) -> usize {
        // Use different seeds for different hash functions
        let seed = ((index as u64).wrapping_mul(0x9e3779b97f4a7c15)) as usize;
        let mut hash = seed ^ 0xcafebabe;

        for &byte in element {
            hash = hash.wrapping_mul(31).wrapping_add(byte as usize);
        }

        hash.wrapping_mul(2654435761)
    }

    /// Merge two bloom filters (OR operation)
    pub fn merge(&mut self, other: &BloomFilter) -> Result<(), String> {
        if self.bits.len() != other.bits.len() || self.num_hashes != other.num_hashes {
            return Err("Cannot merge bloom filters with different parameters".to_string());
        }

        for i in 0..self.bits.len() {
            self.bits[i] |= other.bits[i];
        }

        Ok(())
    }
}

/// Statistics about a bloom filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloomFilterStats {
    /// Size in bytes
    pub size_bytes: usize,
    /// Total number of bits
    pub num_bits: usize,
    /// Number of hash functions used
    pub num_hashes: usize,
    /// Number of bits currently set to 1
    pub bits_set: usize,
    /// Ratio of bits set (0.0 to 1.0)
    pub fill_ratio: f64,
}

/// Range index for efficient comparison predicates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeIndex {
    /// Minimum value in this index
    pub min_value: Vec<u8>,
    /// Maximum value in this index
    pub max_value: Vec<u8>,
    /// Whether values are sorted (for binary search)
    pub is_sorted: bool,
    /// Approximate count of values
    pub value_count: usize,
}

impl RangeIndex {
    /// Create a new range index
    pub fn new(min: Vec<u8>, max: Vec<u8>, count: usize) -> Self {
        RangeIndex {
            min_value: min,
            max_value: max,
            is_sorted: true,
            value_count: count,
        }
    }

    /// Check if a value might be in this range
    pub fn contains(&self, value: &[u8]) -> bool {
        value >= &self.min_value && value <= &self.max_value
    }

    /// Check if range overlaps with a predicate range
    pub fn overlaps(&self, min: &[u8], max: &[u8]) -> bool {
        self.max_value.as_slice() >= min && self.min_value.as_slice() <= max
    }
}

/// Hash-based index for equality predicates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashIndex {
    /// Set of distinct values (lossy - may have false positives)
    distinct_values: HashSet<Vec<u8>>,
    /// Bloom filter for fast membership testing
    bloom_filter: BloomFilter,
    /// Number of elements added
    element_count: usize,
}

impl HashIndex {
    /// Create a new hash index with expected cardinality
    pub fn new(expected_cardinality: usize) -> Self {
        let bloom = BloomFilter::new(expected_cardinality, 0.01); // 1% false positive rate

        HashIndex {
            distinct_values: HashSet::new(),
            bloom_filter: bloom,
            element_count: 0,
        }
    }

    /// Add a value to the index
    pub fn insert(&mut self, value: &[u8]) {
        self.distinct_values.insert(value.to_vec());
        self.bloom_filter.insert(value);
        self.element_count += 1;
    }

    /// Check if value might be in the index
    pub fn might_contain(&self, value: &[u8]) -> bool {
        self.bloom_filter.contains(value)
    }

    /// Check if value definitely is in the index (false positives possible for bloom only)
    pub fn definitely_contains(&self, value: &[u8]) -> bool {
        self.distinct_values.contains(value)
    }

    /// Get number of distinct values
    pub fn distinct_count(&self) -> usize {
        self.distinct_values.len()
    }

    /// Get number of total elements added
    pub fn element_count(&self) -> usize {
        self.element_count
    }
}

/// Composite index combining multiple index types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeIndex {
    /// Column name
    pub column_name: String,
    /// Range index for comparison predicates
    pub range_index: Option<RangeIndex>,
    /// Hash index for equality predicates
    pub hash_index: Option<HashIndex>,
    /// Bloom filter for membership testing
    pub bloom_filter: Option<BloomFilter>,
}

impl CompositeIndex {
    /// Create a new composite index
    pub fn new(column_name: String) -> Self {
        CompositeIndex {
            column_name,
            range_index: None,
            hash_index: None,
            bloom_filter: None,
        }
    }

    /// Enable range indexing
    pub fn with_range_index(mut self, min: Vec<u8>, max: Vec<u8>, count: usize) -> Self {
        self.range_index = Some(RangeIndex::new(min, max, count));
        self
    }

    /// Enable hash indexing
    pub fn with_hash_index(mut self, expected_cardinality: usize) -> Self {
        self.hash_index = Some(HashIndex::new(expected_cardinality));
        self
    }

    /// Enable bloom filter
    pub fn with_bloom_filter(mut self, expected_elements: usize) -> Self {
        self.bloom_filter = Some(BloomFilter::new(expected_elements, 0.01));
        self
    }

    /// Get size in bytes
    pub fn size_bytes(&self) -> usize {
        let mut total = 0;
        if let Some(bloom) = &self.bloom_filter {
            total += bloom.size_bytes();
        }
        total
    }
}

/// Index statistics for all columns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Indexes for each column
    pub column_indexes: Vec<CompositeIndex>,
    /// Total index size in bytes
    pub total_size_bytes: usize,
    /// Compression ratio (original / indexed)
    pub compression_ratio: f64,
}

impl IndexStats {
    /// Calculate total index size
    pub fn calculate_total_size(&mut self) {
        self.total_size_bytes = self.column_indexes.iter().map(|idx| idx.size_bytes()).sum();
    }
}

/// Predicate evaluation result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicatePushdownResult {
    /// Row group definitely contains rows matching predicate
    MayContain,
    /// Row group definitely does NOT contain matching rows
    DefinitelyNotContain,
    /// Inconclusive, need to read row group
    Inconclusive,
}

impl PredicatePushdownResult {
    /// Combine multiple predicate results (AND semantics)
    pub fn combine(self, other: PredicatePushdownResult) -> PredicatePushdownResult {
        match (self, other) {
            (PredicatePushdownResult::DefinitelyNotContain, _) |
            (_, PredicatePushdownResult::DefinitelyNotContain) => {
                PredicatePushdownResult::DefinitelyNotContain
            }
            (PredicatePushdownResult::MayContain, PredicatePushdownResult::MayContain) => {
                PredicatePushdownResult::MayContain
            }
            _ => PredicatePushdownResult::Inconclusive,
        }
    }
}

/// Predicate types for evaluation
#[derive(Debug, Clone)]
pub enum Predicate {
    /// Equality: field = value
    Equal(Vec<u8>),
    /// Inequality: field != value
    NotEqual(Vec<u8>),
    /// Range: field >= min && field <= max
    Range(Vec<u8>, Vec<u8>),
    /// Greater than: field > value
    GreaterThan(Vec<u8>),
    /// Less than: field < value
    LessThan(Vec<u8>),
    /// IN list: field IN (v1, v2, ...)
    In(Vec<Vec<u8>>),
}

impl Predicate {
    /// Evaluate predicate against a composite index
    pub fn evaluate(&self, index: &CompositeIndex) -> PredicatePushdownResult {
        match self {
            Predicate::Equal(value) => {
                if let Some(bloom) = &index.bloom_filter {
                    if bloom.contains(value) {
                        PredicatePushdownResult::MayContain
                    } else {
                        PredicatePushdownResult::DefinitelyNotContain
                    }
                } else if let Some(hash) = &index.hash_index {
                    if hash.might_contain(value) {
                        PredicatePushdownResult::MayContain
                    } else {
                        PredicatePushdownResult::DefinitelyNotContain
                    }
                } else {
                    PredicatePushdownResult::Inconclusive
                }
            }

            Predicate::NotEqual(_) => {
                // NOT_EQUAL can rarely eliminate row groups
                PredicatePushdownResult::MayContain
            }

            Predicate::Range(min, max) => {
                if let Some(range_idx) = &index.range_index {
                    if range_idx.overlaps(min, max) {
                        PredicatePushdownResult::MayContain
                    } else {
                        PredicatePushdownResult::DefinitelyNotContain
                    }
                } else {
                    PredicatePushdownResult::Inconclusive
                }
            }

            Predicate::GreaterThan(value) => {
                if let Some(range_idx) = &index.range_index {
                    if range_idx.max_value > *value {
                        PredicatePushdownResult::MayContain
                    } else {
                        PredicatePushdownResult::DefinitelyNotContain
                    }
                } else {
                    PredicatePushdownResult::Inconclusive
                }
            }

            Predicate::LessThan(value) => {
                if let Some(range_idx) = &index.range_index {
                    if range_idx.min_value < *value {
                        PredicatePushdownResult::MayContain
                    } else {
                        PredicatePushdownResult::DefinitelyNotContain
                    }
                } else {
                    PredicatePushdownResult::Inconclusive
                }
            }

            Predicate::In(values) => {
                // Check if any value might be in the index
                if let Some(bloom) = &index.bloom_filter {
                    for value in values {
                        if bloom.contains(value) {
                            return PredicatePushdownResult::MayContain;
                        }
                    }
                    PredicatePushdownResult::DefinitelyNotContain
                } else if let Some(hash) = &index.hash_index {
                    for value in values {
                        if hash.might_contain(value) {
                            return PredicatePushdownResult::MayContain;
                        }
                    }
                    PredicatePushdownResult::DefinitelyNotContain
                } else {
                    PredicatePushdownResult::Inconclusive
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_filter_basic() {
        let mut filter = BloomFilter::new(100, 0.01);
        
        filter.insert(b"hello");
        filter.insert(b"world");

        assert!(filter.contains(b"hello"));
        assert!(filter.contains(b"world"));
        assert!(!filter.contains(b"nothere"));
    }

    #[test]
    fn test_bloom_filter_false_positive_rate() {
        let mut filter = BloomFilter::new(1000, 0.01);

        // Insert 1000 elements
        for i in 0..1000 {
            let key = format!("key_{}", i);
            filter.insert(key.as_bytes());
        }

        // Check some elements that should be there
        for i in 0..100 {
            let key = format!("key_{}", i);
            assert!(filter.contains(key.as_bytes()));
        }

        // Check some elements that shouldn't be there
        // (may have false positives, but shouldn't be all true)
        let mut false_positives = 0;
        for i in 1000..2000 {
            let key = format!("key_{}", i);
            if filter.contains(key.as_bytes()) {
                false_positives += 1;
            }
        }

        // With 1% false positive rate and 1000 non-elements,
        // expect roughly 10 false positives
        assert!(false_positives < 50, "Too many false positives: {}", false_positives);
    }

    #[test]
    fn test_bloom_filter_merge() {
        let mut filter1 = BloomFilter::new(100, 0.01);
        let mut filter2 = BloomFilter::new(100, 0.01);

        filter1.insert(b"apple");
        filter1.insert(b"banana");

        filter2.insert(b"cherry");
        filter2.insert(b"date");

        // Merge filter2 into filter1
        assert!(filter1.merge(&filter2).is_ok());

        // After merge, filter1 should contain all elements
        assert!(filter1.contains(b"apple"));
        assert!(filter1.contains(b"banana"));
        assert!(filter1.contains(b"cherry"));
        assert!(filter1.contains(b"date"));
    }

    #[test]
    fn test_range_index() {
        let range = RangeIndex::new(vec![10], vec![100], 90);

        assert!(range.contains(&[50]));
        assert!(!range.contains(&[5]));
        assert!(!range.contains(&[105]));
    }

    #[test]
    fn test_hash_index() {
        let mut index = HashIndex::new(10);

        index.insert(b"value1");
        index.insert(b"value2");
        index.insert(b"value3");

        assert!(index.might_contain(b"value1"));
        assert!(index.definitely_contains(b"value1"));
        assert_eq!(index.distinct_count(), 3);
    }

    #[test]
    fn test_predicate_evaluation_equal() {
        let mut index = CompositeIndex::new("test_column".to_string());
        let mut bloom = BloomFilter::new(100, 0.01);
        bloom.insert(b"target_value");
        index.bloom_filter = Some(bloom);

        let predicate = Predicate::Equal(b"target_value".to_vec());
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::MayContain
        );

        let predicate = Predicate::Equal(b"other_value".to_vec());
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::DefinitelyNotContain
        );
    }

    #[test]
    fn test_predicate_evaluation_range() {
        let mut index = CompositeIndex::new("test_column".to_string());
        let range = RangeIndex::new(vec![10], vec![100], 90);
        index.range_index = Some(range);

        let predicate = Predicate::Range(vec![20], vec![50]);
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::MayContain
        );

        let predicate = Predicate::Range(vec![200u8], vec![255u8]);
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::DefinitelyNotContain
        );
    }

    #[test]
    fn test_composite_index() {
        let index = CompositeIndex::new("test_column".to_string())
            .with_range_index(vec![10], vec![100], 90)
            .with_hash_index(50);

        assert!(index.range_index.is_some());
        assert!(index.hash_index.is_some());
    }

    #[test]
    fn test_predicate_combination() {
        let mut result1 = PredicatePushdownResult::MayContain;
        let result2 = PredicatePushdownResult::MayContain;
        
        assert_eq!(result1.combine(result2), PredicatePushdownResult::MayContain);

        result1 = PredicatePushdownResult::DefinitelyNotContain;
        assert_eq!(
            result1.combine(result2),
            PredicatePushdownResult::DefinitelyNotContain
        );
    }
}
