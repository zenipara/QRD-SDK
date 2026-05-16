# Quick Start: Using Indexes and Bloom Filters

## 5-Minute Overview

QRD-SDK now includes powerful indexing capabilities to optimize queries. Here's how to use them:

### Basic Bloom Filter

```rust
use qrd_core::validation::BloomFilter;

// Create filter for 10,000 values with 1% false-positive rate
let mut filter = BloomFilter::new(10000, 0.01);

// Insert values
filter.insert(b"user_123");
filter.insert(b"user_456");

// Check membership
if filter.contains(b"user_123") {
    println!("Found!");
}
```

### Basic Range Index

```rust
use qrd_core::validation::RangeIndex;

// Track range of values
let range = RangeIndex::new(
    vec![10u8],    // Min value
    vec![100u8],   // Max value
    1000,          // Element count
);

// Check if value in range
if range.contains(&[50u8]) {
    println!("Value is in range");
}

// Check if ranges overlap
if range.overlaps(&[20u8], &[80u8]) {
    println!("Query range overlaps index range");
}
```

### Combined Indexes with CompositeIndex

```rust
use qrd_core::validation::CompositeIndex;

// Create index for a column
let index = CompositeIndex::new("user_id".to_string())
    .with_bloom_filter(10000)           // Add bloom filter
    .with_range_index(                  // Add range index
        vec![1u8], 
        vec![100u8], 
        10000
    );

println!("Total index size: {} bytes", index.size_bytes());
```

### Predicate Pushdown

```rust
use qrd_core::validation::{Predicate, PredicatePushdownResult};

// Create index (as above)
let index = CompositeIndex::new("age".to_string())
    .with_range_index(vec![18u8], vec![100u8], 5000);

// Query: age > 30
let predicate = Predicate::GreaterThan(vec![30u8]);

match predicate.evaluate(&index) {
    PredicatePushdownResult::MayContain => {
        println!("Row group might have data matching query");
    }
    PredicatePushdownResult::DefinitelyNotContain => {
        println!("Row group is safe to skip");
    }
    PredicatePushdownResult::Inconclusive => {
        println!("No index available, must read row group");
    }
}
```

### Multiple Predicates (AND Logic)

```rust
// Query: age > 20 AND age < 80
let age_index = CompositeIndex::new("age".to_string())
    .with_range_index(vec![18u8], vec![100u8], 5000);

let category_index = CompositeIndex::new("category".to_string())
    .with_bloom_filter(1000);

let age_pred = Predicate::GreaterThan(vec![20u8]);
let cat_pred = Predicate::Equal(b"premium".to_vec());

let age_result = age_pred.evaluate(&age_index);
let cat_result = cat_pred.evaluate(&category_index);

// Combine using AND semantics
let combined = age_result.combine(cat_result);

match combined {
    PredicatePushdownResult::DefinitelyNotContain => {
        println!("Both predicates fail - definitely skip");
    }
    _ => {
        println!("At least one predicate might match - read row group");
    }
}
```

## Predicate Types

```rust
use qrd_core::validation::Predicate;

// Equality
let pred = Predicate::Equal(b"target_value".to_vec());

// Range query
let pred = Predicate::Range(
    b"min_value".to_vec(),
    b"max_value".to_vec(),
);

// Comparison operators
let pred = Predicate::GreaterThan(b"42".to_vec());
let pred = Predicate::LessThan(b"100".to_vec());

// Not equal
let pred = Predicate::NotEqual(b"excluded_value".to_vec());

// Multiple values
let pred = Predicate::In(vec![
    b"value1".to_vec(),
    b"value2".to_vec(),
    b"value3".to_vec(),
]);
```

## Tuning False-Positive Rates

```rust
use qrd_core::validation::BloomFilter;

// Memory-efficient: higher FP rate, smaller filter
let filter = BloomFilter::new(10000, 0.05); // 5% FP rate

// Balanced (default recommendation)
let filter = BloomFilter::new(10000, 0.01); // 1% FP rate

// Precision-focused: lower FP rate, larger filter
let filter = BloomFilter::new(10000, 0.001); // 0.1% FP rate
```

## Index Statistics

```rust
use qrd_core::validation::BloomFilter;

let mut filter = BloomFilter::new(10000, 0.01);

// Insert data
for i in 0..5000 {
    filter.insert(format!("value_{}", i).as_bytes());
}

// Get statistics
let stats = filter.statistics();
println!("Size: {} bytes", stats.size_bytes);
println!("Bits set: {}", stats.bits_set);
println!("Fill ratio: {:.1}%", stats.fill_ratio * 100.0);
println!("Hash functions: {}", stats.num_hashes);
```

## Common Patterns

### Pattern 1: Skip Empty Row Groups

```rust
let pred = Predicate::In(vec![b"A".to_vec(), b"B".to_vec()]);
match pred.evaluate(&index) {
    PredicatePushdownResult::DefinitelyNotContain => {
        // Skip this row group entirely
    }
    _ => {
        // Read row group data
    }
}
```

### Pattern 2: Time Series Filtering

```rust
// Index for timestamp column
let ts_index = CompositeIndex::new("timestamp".to_string())
    .with_range_index(
        "2024-01-01".as_bytes().to_vec(),
        "2024-01-31".as_bytes().to_vec(),
        10000,
    );

// Query for recent data
let pred = Predicate::GreaterThan("2024-01-15".as_bytes().to_vec());

match pred.evaluate(&ts_index) {
    PredicatePushdownResult::DefinitelyNotContain => {
        println!("Row group is too old");
    }
    _ => {
        println!("Row group might have recent data");
    }
}
```

### Pattern 3: Categorical Data

```rust
// Index for category column
let cat_index = CompositeIndex::new("category".to_string())
    .with_bloom_filter(500); // 500 distinct categories

// Initialize with known values
let mut index = cat_index;

// Query for specific categories
let pred = Predicate::In(vec![
    b"electronics".to_vec(),
    b"clothing".to_vec(),
]);

match pred.evaluate(&index) {
    PredicatePushdownResult::DefinitelyNotContain => {
        println!("No matching categories in this row group");
    }
    PredicatePushdownResult::MayContain => {
        println!("Might have matching categories");
    }
    _ => {}
}
```

## Performance Tips

1. **Choose Right Index Type**
   - Use BloomFilter for equality/membership queries
   - Use RangeIndex for range queries
   - Use CompositeIndex for mixed query patterns

2. **Tune False-Positive Rates**
   - 0.001-0.01: Best selectivity, more memory
   - 0.01-0.05: Balanced (default range)
   - 0.05+: Maximum space efficiency, higher FP

3. **Index High-Selectivity Columns**
   - Columns with high cardinality (many distinct values)
   - Columns frequently filtered in queries
   - Numeric ranges with specific predicates

4. **Monitor Index Sizes**
   - Use `index.size_bytes()` to track memory
   - Index storage typically 1-5% of data size
   - Adjust false-positive rate if size too large

## Troubleshooting

### Problem: Always Get "MayContain"

**Cause:** No matching index for predicate type

**Solution:**
```rust
// Add appropriate index type
let index = CompositeIndex::new("column".to_string())
    .with_bloom_filter(10000)      // For equality queries
    .with_range_index(min, max, count); // For range queries
```

### Problem: Filter Too Large

**Cause:** False-positive rate too low

**Solution:**
```rust
// Increase false-positive rate
let filter = BloomFilter::new(10000, 0.05); // Was 0.001
```

### Problem: Many False Positives

**Cause:** False-positive rate too high

**Solution:**
```rust
// Decrease false-positive rate
let filter = BloomFilter::new(10000, 0.001); // Was 0.05
```

## Next Steps

1. **Review Full Documentation**
   - See [INDEX_BLOOM_FILTER.md](INDEX_BLOOM_FILTER.md) for detailed guide
   - See [INDEX_BLOOM_FILTER_API.md](INDEX_BLOOM_FILTER_API.md) for API reference

2. **Explore Integration**
   - Once FileWriter/FileReader integration is complete, indexes will automatically be:
     - Built during file writing
     - Stored in file metadata
     - Used for automatic predicate pushdown

3. **Run Examples**
   ```bash
   cargo test -p qrd-core --test index_bloom_test
   ```

## Full Examples

See `/workspaces/QRD-SDK/core/qrd-core/tests/index_bloom_test.rs` for comprehensive examples.

## Related Documentation

- [Complete User Guide](INDEX_BLOOM_FILTER.md)
- [API Reference](INDEX_BLOOM_FILTER_API.md)
- [Implementation Summary](INDEX_IMPLEMENTATION_SUMMARY.md)
- [Schema Evolution](SCHEMA_EVOLUTION.md)
