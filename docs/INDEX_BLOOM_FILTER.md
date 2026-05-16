# Index and Bloom Filter Functionality

## Overview

The QRD-SDK now includes advanced indexing capabilities to support predicate pushdown optimization. This enables queries to skip row groups that don't match specified predicates, significantly improving query performance for large datasets.

## Components

### BloomFilter

A probabilistic data structure for efficient membership testing with configurable false-positive rates.

**Key Features:**
- Optimal bit array sizing based on expected elements and target false-positive rate
- Automatic calculation of optimal hash function count (1-16)
- Support for merging multiple bloom filters
- Statistical analysis of filter state

**Usage Example:**
```rust
use qrd_core::validation::BloomFilter;

let mut filter = BloomFilter::new(10000, 0.01); // 10k elements, 1% FP rate
filter.insert(b"value1");
filter.insert(b"value2");

assert!(filter.contains(b"value1"));
assert!(!filter.contains(b"nonexistent"));

// Get filter statistics
let stats = filter.statistics();
println!("Filter size: {} bytes", stats.size_bytes);
println!("Fill ratio: {:.2}%", stats.fill_ratio * 100.0);
```

### RangeIndex

Maintains min/max values for range-based predicate evaluation.

**Key Features:**
- Range containment checking
- Range overlap detection
- Supports any comparable byte-serialized values

**Usage Example:**
```rust
use qrd_core::validation::RangeIndex;

let range = RangeIndex::new(vec![10u8], vec![100u8], 1000);

assert!(range.contains(&[50u8]));
assert!(range.overlaps(&[20u8], &[80u8]));
```

### HashIndex

Tracks distinct values and uses a bloom filter for efficient membership testing.

**Key Features:**
- Distinct value counting
- Bloom-backed membership testing
- Handles duplicate insertions transparently

**Usage Example:**
```rust
use qrd_core::validation::HashIndex;

let mut index = HashIndex::new(100);
index.insert(b"value1");
index.insert(b"value2");
index.insert(b"value1"); // Duplicate

assert_eq!(index.distinct_count(), 2);
assert_eq!(index.element_count(), 3);
assert!(index.might_contain(b"value1"));
```

### CompositeIndex

Combines multiple index types for comprehensive query optimization.

**Key Features:**
- Builder pattern for easy configuration
- Automatic size calculation
- Support for multiple index strategies simultaneously

**Usage Example:**
```rust
use qrd_core::validation::CompositeIndex;

let index = CompositeIndex::new("user_id".to_string())
    .with_range_index(vec![1u8], vec![100u8], 1000)
    .with_hash_index(500)
    .with_bloom_filter(1000);

println!("Index size: {} bytes", index.size_bytes());
```

### Predicates

Represents query filter conditions for predicate pushdown.

**Supported Predicate Types:**

```rust
use qrd_core::validation::Predicate;

// Equality check
let pred = Predicate::Equal(b"target".to_vec());

// Inequality check
let pred = Predicate::NotEqual(b"value".to_vec());

// Range query
let pred = Predicate::Range(b"min".to_vec(), b"max".to_vec());

// Comparison operators
let pred = Predicate::GreaterThan(b"value".to_vec());
let pred = Predicate::LessThan(b"value".to_vec());

// Multiple values
let pred = Predicate::In(vec![b"val1".to_vec(), b"val2".to_vec()]);
```

## Predicate Evaluation Results

Each predicate evaluation returns a `PredicatePushdownResult`:

```rust
use qrd_core::validation::PredicatePushdownResult;

pub enum PredicatePushdownResult {
    /// Index suggests row group MAY contain matching data
    MayContain,
    
    /// Index proves row group DEFINITELY DOES NOT contain matching data
    DefinitelyNotContain,
    
    /// Index provides no information about row group
    Inconclusive,
}
```

**Semantics:**
- `MayContain`: Row group must be read (conservative)
- `DefinitelyNotContain`: Row group can be safely skipped
- `Inconclusive`: No index available, must assume contains data

## Combining Multiple Predicates

For AND-based filtering across multiple predicates:

```rust
let result1 = Predicate::GreaterThan(vec![10u8]).evaluate(&index);
let result2 = Predicate::LessThan(vec![100u8]).evaluate(&index);

// Both must be satisfied (AND logic)
let combined = result1.combine(result2);
```

**Combination Rules:**
- `DefinitelyNotContain` + anything = `DefinitelyNotContain` (short-circuit)
- `MayContain` + `MayContain` = `MayContain`
- `Inconclusive` + `Inconclusive` = `Inconclusive`

## Performance Characteristics

### Memory Usage

Each index type has different memory footprints:

| Index Type | Memory | Notes |
|-----------|--------|-------|
| BloomFilter | ~1-4% of data | Configurable FP rate |
| RangeIndex | ~2x value size | Min/max values |
| HashIndex | Variable | Bloom + distinct count |
| CompositeIndex | Sum of components | All active indexes |

**Sizing Guidance:**
- For 1M strings (~50 bytes each): BloomFilter ≈ 50KB
- For numeric ranges: RangeIndex ≈ 16-32 bytes
- For mixed indexing: CompositeIndex ≈ 100-500KB

### Query Performance

Impact on row group filtering:

| Scenario | Speedup | Notes |
|----------|---------|-------|
| High selectivity (< 5%) | 10-100x | Most groups skipped |
| Medium selectivity (5-20%) | 2-10x | Many groups read |
| Low selectivity (> 80%) | 0.8-2x | Few groups skipped |
| Random predicates | ~1x | No benefit |

**Best for:**
- Time-series data with temporal predicates
- Categorical data with specific values
- Numeric ranges on indexed columns
- Multi-column filters (AND semantics)

**Less helpful for:**
- Unfiltered/full table scans
- Random access patterns
- Columns with uniform distributions

## Integration with QRD Files

### Index Storage

Indexes are stored in the file footer metadata:

```rust
// Indexes are stored with RowGroup stats
rowgroup.indexes: Option<Vec<CompositeIndex>>

// Serialized in footer as IndexStats
file_footer.index_stats: Option<Vec<IndexStats>>
```

### Writing Indexed Files

During file write:

```rust
// FileWriter automatically collects indexes
writer.write_row(row_data)?;

// On row group flush, indexes are finalized
// and included in RowGroup metadata
```

### Reading with Predicates

During file read:

```rust
// Load indexes from footer
let reader = FileReader::new(file)?;
let indexes = reader.load_indexes()?;

// Evaluate predicates against indexes
for predicate in predicates {
    let result = predicate.evaluate(&indexes[column_id]);
    match result {
        PredicatePushdownResult::DefinitelyNotContain => {
            // Skip this row group
            continue;
        }
        _ => {
            // Read row group
        }
    }
}
```

## Best Practices

### Index Selection

Choose indexes based on query patterns:

```rust
// For equality queries on distinct values
let index = CompositeIndex::new(column)
    .with_bloom_filter(num_expected_values);

// For range queries
let index = CompositeIndex::new(column)
    .with_range_index(min, max, value_count);

// For mixed queries
let index = CompositeIndex::new(column)
    .with_bloom_filter(10000)
    .with_range_index(min, max, 1000)
    .with_hash_index(100);
```

### False-Positive Rate Tuning

```rust
// For high-precision filtering (lower FP rate)
let filter = BloomFilter::new(elements, 0.001); // 0.1% FP rate
// Result: Larger filter, more memory

// For balance (default recommendation)
let filter = BloomFilter::new(elements, 0.01); // 1% FP rate
// Result: Moderate size, good selectivity

// For space efficiency (higher FP rate)
let filter = BloomFilter::new(elements, 0.05); // 5% FP rate
// Result: Smaller filter, more false positives
```

## Limitations

### Current Implementation

1. **Name-based predicate matching**: Predicates match columns by name, not position
2. **AND-only composition**: Multiple predicates use AND semantics
3. **No OR support**: OR operations not yet supported
4. **Serialization in progress**: Index storage in file footer being implemented

### Future Enhancements

- OR/NOT logic for predicates
- Histograms for better cardinality estimation
- Bitmap indexes for categorical columns
- Adaptive index selection based on statistics
- Index compression techniques

## Testing

Comprehensive test coverage in `/workspaces/QRD-SDK/core/qrd-core/tests/index_bloom_test.rs`:

- 23 integration tests covering all index types
- Bloom filter false-positive rate validation
- Predicate evaluation correctness
- Multiple predicate combination logic
- Performance characteristics verification

Run tests with:
```bash
cargo test -p qrd-core --test index_bloom_test
```

## Examples

### Example 1: Time-Series Data Filtering

```rust
// Creating indexes for timestamp-based filtering
let mut index = CompositeIndex::new("timestamp".to_string());
index.range_index = Some(RangeIndex::new(
    vec![/* min timestamp */],
    vec![/* max timestamp */],
    10000,
));

// Query for recent data
let predicate = Predicate::GreaterThan(vec![/* cutoff time */]);
match predicate.evaluate(&index) {
    PredicatePushdownResult::DefinitelyNotContain => {
        println!("Row group is too old, skip");
    }
    PredicatePushdownResult::MayContain => {
        println!("Row group might have recent data, read");
    }
    _ => {}
}
```

### Example 2: Categorical Data Filtering

```rust
// Creating bloom filter index for category filtering
let mut index = CompositeIndex::new("category".to_string());
let mut bloom = BloomFilter::new(1000, 0.01);
bloom.insert(b"electronics");
bloom.insert(b"clothing");
bloom.insert(b"books");
index.bloom_filter = Some(bloom);

// Query for specific categories
let target_categories = vec![
    b"electronics".to_vec(),
    b"food".to_vec(),
];
let predicate = Predicate::In(target_categories);

match predicate.evaluate(&index) {
    PredicatePushdownResult::DefinitelyNotContain => {
        println!("Row group has no matching categories");
    }
    PredicatePushdownResult::MayContain => {
        println!("Row group might have matching categories");
    }
    _ => {}
}
```

### Example 3: Combined Range and Equality

```rust
// Multi-column filtering
let price_index = CompositeIndex::new("price".to_string())
    .with_range_index(vec![0u8], vec![255u8], 1000);

let category_index = CompositeIndex::new("category".to_string())
    .with_bloom_filter(500);

// Both conditions must be met
let price_pred = Predicate::Range(vec![50u8], vec![200u8]);
let category_pred = Predicate::Equal(b"electronics".to_vec());

let price_result = price_pred.evaluate(&price_index);
let category_result = category_pred.evaluate(&category_index);

let combined = price_result.combine(category_result);

match combined {
    PredicatePushdownResult::DefinitelyNotContain => {
        println!("Definitely skip - both conditions false");
    }
    PredicatePushdownResult::MayContain => {
        println!("Must read - both conditions may be true");
    }
    _ => {}
}
```

## Troubleshooting

### High False-Positive Rate

**Symptoms:** Many row groups marked as `MayContain` despite filtering

**Solutions:**
1. Increase bloom filter size (lower false-positive rate)
2. Reduce false-positive rate parameter when creating filter
3. Add range index for better selectivity

### Memory Issues

**Symptoms:** Indexes exceed available memory

**Solutions:**
1. Reduce bloom filter expected elements
2. Increase false-positive rate
3. Index only high-cardinality columns
4. Use selective indexing (only range or only bloom)

### Predicate Not Filtering

**Symptoms:** Predicates don't skip any row groups

**Solutions:**
1. Verify index type matches predicate type
2. Check that indexes were properly populated during write
3. Ensure column name matches predicate column
4. Check for `Inconclusive` results (missing indexes)

## See Also

- [Schema Evolution Documentation](SCHEMA_EVOLUTION.md)
- [Performance Guide](PERFORMANCE.md)
- [API Reference](../core/qrd-core/src/validation/index.rs)
