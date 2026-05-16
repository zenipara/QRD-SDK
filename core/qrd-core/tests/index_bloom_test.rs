//! Integration tests for bloom filter and index functionality

#[cfg(test)]
mod index_tests {
    use qrd_core::validation::{
        BloomFilter, CompositeIndex, HashIndex, Predicate, PredicatePushdownResult, RangeIndex,
    };

    #[test]
    fn test_bloom_filter_empty_and_insert() {
        let mut filter = BloomFilter::new(10, 0.01);
        assert!(!filter.contains(b"test"));

        filter.insert(b"test");
        assert!(filter.contains(b"test"));
    }

    #[test]
    fn test_bloom_filter_large_dataset() {
        let mut filter = BloomFilter::new(10000, 0.01);

        // Insert 10000 elements
        for i in 0..10000 {
            let key = format!("element_{:05}", i);
            filter.insert(key.as_bytes());
        }

        // Verify inserted elements are found
        for i in 0..100 {
            let key = format!("element_{:05}", i);
            assert!(filter.contains(key.as_bytes()));
        }

        // Check distribution of set bits
        let stats = filter.statistics();
        assert!(stats.fill_ratio > 0.0);
        assert!(stats.fill_ratio < 1.0);
    }

    #[test]
    fn test_bloom_filter_statistics() {
        let mut filter = BloomFilter::new(100, 0.01);

        for i in 0..50 {
            let key = format!("item_{}", i);
            filter.insert(key.as_bytes());
        }

        let stats = filter.statistics();
        assert!(stats.bits_set > 0);
        assert!(stats.fill_ratio > 0.0);
        assert!(stats.num_hashes > 0);
    }

    #[test]
    fn test_bloom_filter_merge_success() {
        let mut filter1 = BloomFilter::new(100, 0.01);
        let mut filter2 = BloomFilter::new(100, 0.01);

        // Populate both filters
        for i in 0..50 {
            let key = format!("set1_{}", i);
            filter1.insert(key.as_bytes());

            let key = format!("set2_{}", i);
            filter2.insert(key.as_bytes());
        }

        // Merge
        assert!(filter1.merge(&filter2).is_ok());

        // Both sets should be present
        assert!(filter1.contains(b"set1_0"));
        assert!(filter1.contains(b"set2_0"));
    }

    #[test]
    fn test_bloom_filter_merge_different_sizes_fails() {
        let filter1 = BloomFilter::new(100, 0.01);
        let filter2 = BloomFilter::new(200, 0.01);

        // Should fail to merge filters with different parameters
        assert!(filter1.size_bytes() != filter2.size_bytes());
    }

    #[test]
    fn test_range_index_contains() {
        let range = RangeIndex::new(vec![10u8], vec![100u8], 100);

        assert!(range.contains(&[50u8]));
        assert!(range.contains(&[10u8])); // Min value
        assert!(range.contains(&[100u8])); // Max value
        assert!(!range.contains(&[9u8]));
        assert!(!range.contains(&[101u8]));
    }

    #[test]
    fn test_range_index_overlaps() {
        let range = RangeIndex::new(vec![10u8], vec![100u8], 100);

        assert!(range.overlaps(&[50u8], &[80u8])); // Completely inside
        assert!(range.overlaps(&[0u8], &[50u8])); // Partial overlap lower
        assert!(range.overlaps(&[50u8], &[200u8])); // Partial overlap upper
        assert!(range.overlaps(&[0u8], &[200u8])); // Complete overlap
        assert!(!range.overlaps(&[101u8], &[200u8])); // No overlap above
        assert!(!range.overlaps(&[0u8], &[9u8])); // No overlap below
    }

    #[test]
    fn test_hash_index_operations() {
        let mut index = HashIndex::new(100);

        // Insert values
        index.insert(b"value1");
        index.insert(b"value2");
        index.insert(b"value3");

        // Check membership
        assert!(index.might_contain(b"value1"));
        assert!(index.might_contain(b"value2"));
        assert!(index.might_contain(b"value3"));
        assert!(!index.might_contain(b"value999"));

        // Check distinct count
        assert_eq!(index.distinct_count(), 3);
        assert_eq!(index.element_count(), 3);
    }

    #[test]
    fn test_hash_index_duplicate_handling() {
        let mut index = HashIndex::new(100);

        index.insert(b"value1");
        index.insert(b"value1"); // Duplicate
        index.insert(b"value1"); // Duplicate

        // Distinct count should be 1, but element_count should be 3
        assert_eq!(index.distinct_count(), 1);
        assert_eq!(index.element_count(), 3);
    }

    #[test]
    fn test_composite_index_creation() {
        let index = CompositeIndex::new("column1".to_string())
            .with_range_index(vec![10u8], vec![100u8], 90)
            .with_hash_index(50)
            .with_bloom_filter(100);

        assert_eq!(index.column_name, "column1");
        assert!(index.range_index.is_some());
        assert!(index.hash_index.is_some());
        assert!(index.bloom_filter.is_some());
        assert!(index.size_bytes() > 0);
    }

    #[test]
    fn test_predicate_equal_found() {
        let mut index = CompositeIndex::new("test".to_string());
        let mut bloom = BloomFilter::new(100, 0.01);
        bloom.insert(b"target");
        index.bloom_filter = Some(bloom);

        let predicate = Predicate::Equal(b"target".to_vec());
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::MayContain
        );
    }

    #[test]
    fn test_predicate_equal_not_found() {
        let mut index = CompositeIndex::new("test".to_string());
        let mut bloom = BloomFilter::new(100, 0.01);
        bloom.insert(b"value1");
        bloom.insert(b"value2");
        index.bloom_filter = Some(bloom);

        let predicate = Predicate::Equal(b"missing".to_vec());
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::DefinitelyNotContain
        );
    }

    #[test]
    fn test_predicate_equal_no_index() {
        let index = CompositeIndex::new("test".to_string());

        let predicate = Predicate::Equal(b"value".to_vec());
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::Inconclusive
        );
    }

    #[test]
    fn test_predicate_range_overlaps() {
        let mut index = CompositeIndex::new("test".to_string());
        let range = RangeIndex::new(vec![10u8], vec![100u8], 100);
        index.range_index = Some(range);

        // Predicate range overlaps with index range
        let predicate = Predicate::Range(vec![50u8], vec![80u8]);
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::MayContain
        );
    }

    #[test]
    fn test_predicate_range_no_overlap() {
        let mut index = CompositeIndex::new("test".to_string());
        let range = RangeIndex::new(vec![10u8], vec![100u8], 100);
        index.range_index = Some(range);

        // Predicate range doesn't overlap with index range
        let predicate = Predicate::Range(vec![200u8], vec![255u8]);
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::DefinitelyNotContain
        );
    }

    #[test]
    fn test_predicate_greater_than() {
        let mut index = CompositeIndex::new("test".to_string());
        let range = RangeIndex::new(vec![10u8], vec![100u8], 100);
        index.range_index = Some(range);

        // Max is greater than value
        let predicate = Predicate::GreaterThan(vec![50u8]);
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::MayContain
        );

        // Max is not greater than value
        let predicate = Predicate::GreaterThan(vec![150u8]);
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::DefinitelyNotContain
        );
    }

    #[test]
    fn test_predicate_less_than() {
        let mut index = CompositeIndex::new("test".to_string());
        let range = RangeIndex::new(vec![10u8], vec![100u8], 100);
        index.range_index = Some(range);

        // Min is less than value
        let predicate = Predicate::LessThan(vec![50u8]);
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::MayContain
        );

        // Min is not less than value
        let predicate = Predicate::LessThan(vec![5u8]);
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::DefinitelyNotContain
        );
    }

    #[test]
    fn test_predicate_in_list() {
        let mut index = CompositeIndex::new("test".to_string());
        let mut bloom = BloomFilter::new(100, 0.01);
        bloom.insert(b"value1");
        bloom.insert(b"value3");
        index.bloom_filter = Some(bloom);

        // Some values in list match
        let predicate = Predicate::In(vec![
            b"value1".to_vec(),
            b"value2".to_vec(),
            b"value3".to_vec(),
        ]);
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::MayContain
        );

        // No values in list match
        let predicate = Predicate::In(vec![b"value999".to_vec(), b"valueXYZ".to_vec()]);
        assert_eq!(
            predicate.evaluate(&index),
            PredicatePushdownResult::DefinitelyNotContain
        );
    }

    #[test]
    fn test_predicate_combination_may_contain() {
        let result1 = PredicatePushdownResult::MayContain;
        let result2 = PredicatePushdownResult::MayContain;

        assert_eq!(result1.combine(result2), PredicatePushdownResult::MayContain);
    }

    #[test]
    fn test_predicate_combination_with_definitely_not() {
        let result1 = PredicatePushdownResult::MayContain;
        let result2 = PredicatePushdownResult::DefinitelyNotContain;

        assert_eq!(
            result1.combine(result2),
            PredicatePushdownResult::DefinitelyNotContain
        );

        let result1 = PredicatePushdownResult::DefinitelyNotContain;
        let result2 = PredicatePushdownResult::MayContain;

        assert_eq!(
            result1.combine(result2),
            PredicatePushdownResult::DefinitelyNotContain
        );
    }

    #[test]
    fn test_bloom_filter_false_positive_rate_realistic() {
        // Create a filter for 10000 elements with 1% FP rate
        let mut filter = BloomFilter::new(10000, 0.01);

        // Insert 5000 elements
        for i in 0..5000 {
            let key = format!("real_{}", i);
            filter.insert(key.as_bytes());
        }

        // Test with non-existent elements
        let mut false_positives = 0;
        for i in 5000..15000 {
            let key = format!("fake_{}", i);
            if filter.contains(key.as_bytes()) {
                false_positives += 1;
            }
        }

        let fp_rate = false_positives as f64 / 10000.0;
        // Should be roughly 1% ± reasonable margin
        assert!(fp_rate < 0.05, "FP rate too high: {}", fp_rate);
    }

    #[test]
    fn test_multiple_predicates_filtering() {
        let mut index = CompositeIndex::new("test".to_string());
        let range = RangeIndex::new(vec![10u8], vec![100u8], 100);
        let mut bloom = BloomFilter::new(100, 0.01);
        bloom.insert(b"value1");
        index.range_index = Some(range);
        index.bloom_filter = Some(bloom);

        // Evaluate multiple predicates
        let pred1 = Predicate::Range(vec![20u8], vec![50u8]);
        let pred2 = Predicate::Equal(b"value1".to_vec());

        let result1 = pred1.evaluate(&index);
        let result2 = pred2.evaluate(&index);

        // Both should be satisfied
        assert_eq!(result1.combine(result2), PredicatePushdownResult::MayContain);
    }

    #[test]
    fn test_index_size_calculation() {
        let mut index = CompositeIndex::new("test".to_string())
            .with_bloom_filter(1000);

        let size_before = index.size_bytes();
        assert!(size_before > 0);

        // Adding more indexes shouldn't reduce size
        index = index.with_range_index(vec![0u8], vec![255u8], 1000);
        let size_after = index.size_bytes();
        assert!(size_after >= size_before);
    }
}
