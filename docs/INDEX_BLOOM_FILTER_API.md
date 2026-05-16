# Index and Bloom Filter API Reference

## BloomFilter

### Constructor

```rust
pub fn new(expected_elements: usize, false_positive_rate: f64) -> Self
```

Creates a new Bloom filter with optimal sizing.

**Parameters:**
- `expected_elements`: Expected number of elements to insert (typical: 1000-1M)
- `false_positive_rate`: Target false-positive rate as decimal (0.001-0.10)

**Returns:** New `BloomFilter` instance

**Examples:**
```rust
// 10,000 elements with 1% false-positive rate
let filter = BloomFilter::new(10000, 0.01);

// 1M elements with 0.1% FP rate (more memory)
let filter = BloomFilter::new(1000000, 0.001);

// Small filter with higher FP rate
let filter = BloomFilter::new(100, 0.10);
```

### Methods

#### insert

```rust
pub fn insert(&mut self, element: &[u8])
```

Inserts an element into the bloom filter.

**Parameters:**
- `element`: Byte slice representing the element

**Examples:**
```rust
let mut filter = BloomFilter::new(100, 0.01);
filter.insert(b"hello");
filter.insert(b"world");

// Works with any byte data
filter.insert(&[1, 2, 3, 4]);
```

#### contains

```rust
pub fn contains(&self, element: &[u8]) -> bool
```

Tests whether an element might be in the filter.

**Returns:** `true` if element is definitely in the filter or with FP probability. `false` if definitely not.

**Important:** `true` doesn't guarantee membership (false positives possible), but `false` guarantees non-membership.

**Examples:**
```rust
let mut filter = BloomFilter::new(100, 0.01);
filter.insert(b"apple");

assert!(filter.contains(b"apple")); // Definitely true
assert!(!filter.contains(b"banana")); // Definitely false
// filter.contains(random_value) may be true (FP) or false
```

#### merge

```rust
pub fn merge(&mut self, other: &BloomFilter) -> Result<()>
```

Merges another filter into this one. Both filters must have identical parameters.

**Requirements:**
- Filters must have same size
- Filters must have same hash function count
- Both must have same false-positive rate configuration

**Returns:** `Ok(())` on success, `Err()` if filters incompatible

**Examples:**
```rust
let mut filter1 = BloomFilter::new(1000, 0.01);
let mut filter2 = BloomFilter::new(1000, 0.01);

filter1.insert(b"item1");
filter2.insert(b"item2");

filter1.merge(&filter2)?;

assert!(filter1.contains(b"item1"));
assert!(filter1.contains(b"item2")); // Now knows about both
```

#### statistics

```rust
pub fn statistics(&self) -> BloomFilterStats
```

Returns statistics about filter state.

**Returns:** `BloomFilterStats` struct with analysis

**Examples:**
```rust
let mut filter = BloomFilter::new(10000, 0.01);
for i in 0..5000 {
    filter.insert(format!("item_{}", i).as_bytes());
}

let stats = filter.statistics();
println!("Size: {} bytes", stats.size_bytes);
println!("Bits set: {}", stats.bits_set);
println!("Fill ratio: {:.2}%", stats.fill_ratio * 100.0);
println!("Hash functions: {}", stats.num_hashes);
```

#### size_bytes

```rust
pub fn size_bytes(&self) -> usize
```

Returns the memory size of the filter in bytes.

**Examples:**
```rust
let filter1 = BloomFilter::new(1000, 0.01);
let filter2 = BloomFilter::new(10000, 0.01);

println!("Filter 1 size: {} bytes", filter1.size_bytes());
println!("Filter 2 size: {} bytes", filter2.size_bytes());
// filter2 is approximately 10x larger
```

## BloomFilterStats

Statistics about a Bloom filter's state.

**Fields:**
```rust
pub struct BloomFilterStats {
    pub size_bytes: usize,      // Total memory used
    pub bits_set: usize,        // Number of bits set to 1
    pub fill_ratio: f64,        // Proportion of bits set (0.0-1.0)
    pub num_hashes: usize,      // Number of hash functions used
}
```

## RangeIndex

### Constructor

```rust
pub fn new(min_value: Vec<u8>, max_value: Vec<u8>, element_count: usize) -> Self
```

Creates a range index from min/max values.

**Parameters:**
- `min_value`: Minimum value in range (byte-encoded)
- `max_value`: Maximum value in range (byte-encoded)
- `element_count`: Total elements in range (for stats)

**Examples:**
```rust
// Numeric range: 10-100
let range = RangeIndex::new(vec![10u8], vec![100u8], 90);

// String range
let range = RangeIndex::new(
    b"apple".to_vec(),
    b"zebra".to_vec(),
    1000,
);
```

### Methods

#### contains

```rust
pub fn contains(&self, value: &[u8]) -> bool
```

Tests if value is within the range.

**Returns:** `true` if value is between min and max (inclusive)

**Examples:**
```rust
let range = RangeIndex::new(vec![10u8], vec![100u8], 100);

assert!(range.contains(&[10u8]));   // min
assert!(range.contains(&[50u8]));   // middle
assert!(range.contains(&[100u8]));  // max
assert!(!range.contains(&[9u8]));   // below
assert!(!range.contains(&[101u8])); // above
```

#### overlaps

```rust
pub fn overlaps(&self, min: &[u8], max: &[u8]) -> bool
```

Tests if a range overlaps with this index's range.

**Returns:** `true` if query range overlaps with index range

**Examples:**
```rust
let range = RangeIndex::new(vec![10u8], vec![100u8], 100);

assert!(range.overlaps(&[20u8], &[50u8]));    // inside
assert!(range.overlaps(&[0u8], &[50u8]));     // partial overlap
assert!(range.overlaps(&[50u8], &[150u8]));   // partial overlap
assert!(!range.overlaps(&[101u8], &[200u8])); // no overlap
```

## HashIndex

### Constructor

```rust
pub fn new(initial_capacity: usize) -> Self
```

Creates a hash index with specified capacity.

**Parameters:**
- `initial_capacity`: Expected number of distinct values

**Examples:**
```rust
// Index expecting ~100 distinct values
let index = HashIndex::new(100);

// Index for millions of distinct values
let index = HashIndex::new(1000000);
```

### Methods

#### insert

```rust
pub fn insert(&mut self, element: &[u8])
```

Inserts an element into the index.

**Note:** Duplicate insertions are handled transparently - distinct count only increments once.

**Examples:**
```rust
let mut index = HashIndex::new(100);
index.insert(b"apple");
index.insert(b"banana");
index.insert(b"apple"); // Duplicate, doesn't increment distinct count

assert_eq!(index.distinct_count(), 2);
assert_eq!(index.element_count(), 3); // Total insertions
```

#### might_contain

```rust
pub fn might_contain(&self, element: &[u8]) -> bool
```

Tests if element might be in index (using bloom filter).

**Returns:** `true` if element might be present, `false` if definitely not

**Examples:**
```rust
let mut index = HashIndex::new(100);
index.insert(b"apple");

assert!(index.might_contain(b"apple"));
assert!(!index.might_contain(b"missing")); // Usually false
```

#### definitely_contains

```rust
pub fn definitely_contains(&self, element: &[u8]) -> bool
```

Tests if element is definitely in index (using distinct set).

**Returns:** `true` if element was inserted, `false` otherwise

**Note:** More accurate than `might_contain()` but requires tracking distinct values.

**Examples:**
```rust
let mut index = HashIndex::new(100);
index.insert(b"apple");

assert!(index.definitely_contains(b"apple"));
assert!(!index.definitely_contains(b"banana"));
```

#### distinct_count

```rust
pub fn distinct_count(&self) -> usize
```

Returns the number of distinct elements inserted.

**Examples:**
```rust
let mut index = HashIndex::new(100);
index.insert(b"a");
index.insert(b"b");
index.insert(b"a"); // Duplicate

assert_eq!(index.distinct_count(), 2);
```

#### element_count

```rust
pub fn element_count(&self) -> usize
```

Returns the total number of insertion operations (including duplicates).

**Examples:**
```rust
let mut index = HashIndex::new(100);
index.insert(b"a");
index.insert(b"a");
index.insert(b"b");

assert_eq!(index.element_count(), 3); // Total insertions
assert_eq!(index.distinct_count(), 2); // Unique values
```

## CompositeIndex

### Constructor

```rust
pub fn new(column_name: String) -> Self
```

Creates a new composite index for a column.

**Parameters:**
- `column_name`: Name of the column being indexed

**Examples:**
```rust
let index = CompositeIndex::new("user_id".to_string());
let index = CompositeIndex::new("timestamp".to_string());
```

### Builder Methods

#### with_bloom_filter

```rust
pub fn with_bloom_filter(mut self, expected_elements: usize) -> Self
```

Adds a bloom filter using default 1% false-positive rate.

**Examples:**
```rust
let index = CompositeIndex::new("user_id".to_string())
    .with_bloom_filter(10000);
```

#### with_range_index

```rust
pub fn with_range_index(mut self, min: Vec<u8>, max: Vec<u8>, count: usize) -> Self
```

Adds a range index.

**Examples:**
```rust
let index = CompositeIndex::new("price".to_string())
    .with_range_index(vec![0u8], vec![255u8], 1000);
```

#### with_hash_index

```rust
pub fn with_hash_index(mut self, capacity: usize) -> Self
```

Adds a hash index.

**Examples:**
```rust
let index = CompositeIndex::new("category".to_string())
    .with_hash_index(500);
```

### Methods

#### size_bytes

```rust
pub fn size_bytes(&self) -> usize
```

Returns total memory used by all active indexes.

**Examples:**
```rust
let index = CompositeIndex::new("test".to_string())
    .with_bloom_filter(10000)
    .with_range_index(vec![0u8], vec![255u8], 1000);

println!("Total index size: {} bytes", index.size_bytes());
```

## Predicate

Represents a query filter condition for predicate pushdown.

### Enum Variants

```rust
pub enum Predicate {
    Equal(Vec<u8>),
    NotEqual(Vec<u8>),
    Range(Vec<u8>, Vec<u8>),
    GreaterThan(Vec<u8>),
    LessThan(Vec<u8>),
    In(Vec<Vec<u8>>),
}
```

### Methods

#### evaluate

```rust
pub fn evaluate(&self, index: &CompositeIndex) -> PredicatePushdownResult
```

Evaluates this predicate against an index.

**Returns:** Result indicating whether row group might contain matching data

**Examples:**
```rust
let mut index = CompositeIndex::new("value".to_string());
let mut bloom = BloomFilter::new(100, 0.01);
bloom.insert(b"target");
index.bloom_filter = Some(bloom);

let pred = Predicate::Equal(b"target".to_vec());
match pred.evaluate(&index) {
    PredicatePushdownResult::MayContain => {
        println!("Row group must be read");
    }
    PredicatePushdownResult::DefinitelyNotContain => {
        println!("Row group can be skipped");
    }
    _ => {}
}
```

## PredicatePushdownResult

### Enum Variants

```rust
pub enum PredicatePushdownResult {
    MayContain,
    DefinitelyNotContain,
    Inconclusive,
}
```

### Methods

#### combine

```rust
pub fn combine(self, other: PredicatePushdownResult) -> PredicatePushdownResult
```

Combines two results using AND semantics (for multiple predicates).

**Rules:**
- Any `DefinitelyNotContain` makes result `DefinitelyNotContain`
- Otherwise, combines conservatively

**Examples:**
```rust
let r1 = PredicatePushdownResult::MayContain;
let r2 = PredicatePushdownResult::MayContain;
assert_eq!(r1.combine(r2), PredicatePushdownResult::MayContain);

let r1 = PredicatePushdownResult::MayContain;
let r2 = PredicatePushdownResult::DefinitelyNotContain;
assert_eq!(r1.combine(r2), PredicatePushdownResult::DefinitelyNotContain);
```

## Type Conversions

### From Vec<u8>

All predicate values are `Vec<u8>` for flexibility:

```rust
// From byte slices
let pred = Predicate::Equal(b"value".to_vec());

// From strings
let pred = Predicate::Equal("value".as_bytes().to_vec());

// From numbers (requires encoding)
let num: u32 = 42;
let pred = Predicate::Equal(num.to_le_bytes().to_vec());

// From custom types
let my_data = MyType { ... };
let bytes = serde_json::to_vec(&my_data)?;
let pred = Predicate::Equal(bytes);
```

## Error Handling

### BloomFilter::merge()

```rust
pub fn merge(&mut self, other: &BloomFilter) -> Result<()>
```

Returns `Err()` if filters have different parameters. Check before merging:

```rust
let filter1 = BloomFilter::new(1000, 0.01);
let filter2 = BloomFilter::new(2000, 0.01); // Different size!

if filter1.merge(&filter2).is_err() {
    println!("Cannot merge - incompatible filters");
}
```

## Performance Notes

### BloomFilter

- **Time Complexity:**
  - `insert()`: O(k) where k = number of hash functions (typically 1-16)
  - `contains()`: O(k) same as insert
  - `merge()`: O(n) where n = bit array size
  
- **Space Complexity:** O(n) where n depends on expected elements and FP rate

### RangeIndex

- **Time Complexity:**
  - `contains()`: O(1) or O(b) where b = value byte length
  - `overlaps()`: O(b) byte comparisons
  
- **Space Complexity:** O(1) - constant, stores only min/max

### HashIndex

- **Time Complexity:**
  - `insert()`: O(1) average, O(k) for bloom filter
  - `contains()`: O(k) bloom filter lookups
  
- **Space Complexity:** O(n) where n = number of distinct values

## Example: Complete Query Filtering

```rust
use qrd_core::validation::{CompositeIndex, Predicate, PredicatePushdownResult};

fn filter_row_groups(
    row_groups: &[RowGroup],
    predicates: &[Predicate],
) -> Vec<bool> {
    row_groups
        .iter()
        .map(|rg| {
            // Load indexes for this row group
            let index = &rg.composite_index;
            
            // Evaluate all predicates
            let mut result = PredicatePushdownResult::MayContain;
            for pred in predicates {
                result = result.combine(pred.evaluate(index));
            }
            
            // Return true if row group should be read
            match result {
                PredicatePushdownResult::MayContain => true,
                PredicatePushdownResult::Inconclusive => true,
                PredicatePushdownResult::DefinitelyNotContain => false,
            }
        })
        .collect()
}
```

## See Also

- [Detailed Usage Guide](INDEX_BLOOM_FILTER.md)
- [Implementation Source](../core/qrd-core/src/validation/index.rs)
- [Test Suite](../core/qrd-core/tests/index_bloom_test.rs)
