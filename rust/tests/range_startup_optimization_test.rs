use bplustree::BPlusTreeMap;
use std::time::Instant;

#[test]
fn test_range_startup_performance() {
    // Create a moderately sized tree to test range startup performance
    let mut tree = BPlusTreeMap::new(16).unwrap();
    let tree_size = 10_000;
    
    // Insert sequential data
    for i in 0..tree_size {
        tree.insert(i, format!("value_{}", i));
    }
    
    // Test range creation performance for small ranges
    let start_key = tree_size / 2;
    let iterations = 1000;
    
    // Measure time for creating many small range iterators
    let start_time = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 100); // Vary the key slightly
        let _iter = tree.range(key..key+1);
        // Don't consume the iterator, just create it
    }
    let elapsed = start_time.elapsed();
    
    let avg_time_ns = elapsed.as_nanos() / iterations as u128;
    println!("Average range creation time: {}ns", avg_time_ns);
    
    // The optimization should make range creation much faster
    // This is a regression test to ensure we don't slow down
    assert!(avg_time_ns < 10_000, "Range creation too slow: {}ns", avg_time_ns);
}

#[test]
fn test_range_excluded_bounds_optimization() {
    let mut tree = BPlusTreeMap::new(16).unwrap();
    
    // Insert test data
    for i in 0..100 {
        tree.insert(i, format!("value_{}", i));
    }
    
    // Test excluded start bound (this should use the optimized path)
    let start_time = Instant::now();
    let range: Vec<_> = tree.range(50..60).map(|(k, _)| *k).collect();
    let elapsed = start_time.elapsed();
    
    // Verify correctness
    assert_eq!(range, vec![50, 51, 52, 53, 54, 55, 56, 57, 58, 59]);
    
    println!("Excluded bounds range time: {}µs", elapsed.as_micros());
    
    // Test that we can create many excluded bound ranges quickly
    let iterations = 100;
    let start_time = Instant::now();
    for i in 0..iterations {
        let start = 10 + (i % 80);
        let _range: Vec<_> = tree.range(start..start+5).map(|(k, _)| *k).collect();
    }
    let elapsed = start_time.elapsed();
    
    let avg_time_us = elapsed.as_micros() / iterations as u128;
    println!("Average excluded bounds range time: {}µs", avg_time_us);
    
    // Should be reasonably fast
    assert!(avg_time_us < 100, "Excluded bounds range too slow: {}µs", avg_time_us);
}

#[test]
fn test_range_included_bounds_optimization() {
    let mut tree = BPlusTreeMap::new(16).unwrap();
    
    // Insert test data
    for i in 0..100 {
        tree.insert(i, format!("value_{}", i));
    }
    
    // Test included start bound
    let range: Vec<_> = tree.range(50..=59).map(|(k, _)| *k).collect();
    
    // Verify correctness
    assert_eq!(range, vec![50, 51, 52, 53, 54, 55, 56, 57, 58, 59]);
    
    // Test performance
    let iterations = 100;
    let start_time = Instant::now();
    for i in 0..iterations {
        let start = 10 + (i % 80);
        let _range: Vec<_> = tree.range(start..=start+4).map(|(k, _)| *k).collect();
    }
    let elapsed = start_time.elapsed();
    
    let avg_time_us = elapsed.as_micros() / iterations as u128;
    println!("Average included bounds range time: {}µs", avg_time_us);
    
    // Should be reasonably fast
    assert!(avg_time_us < 100, "Included bounds range too slow: {}µs", avg_time_us);
}

#[test]
fn test_range_unbounded_optimization() {
    let mut tree = BPlusTreeMap::new(16).unwrap();
    
    // Insert test data
    for i in 0..50 {
        tree.insert(i, format!("value_{}", i));
    }
    
    // Test unbounded ranges
    let full_range: Vec<_> = tree.range(..).map(|(k, _)| *k).collect();
    assert_eq!(full_range.len(), 50);
    assert_eq!(full_range[0], 0);
    assert_eq!(full_range[49], 49);
    
    let from_range: Vec<_> = tree.range(25..).map(|(k, _)| *k).collect();
    assert_eq!(from_range.len(), 25);
    assert_eq!(from_range[0], 25);
    assert_eq!(from_range[24], 49);
    
    let to_range: Vec<_> = tree.range(..25).map(|(k, _)| *k).collect();
    assert_eq!(to_range.len(), 25);
    assert_eq!(to_range[0], 0);
    assert_eq!(to_range[24], 24);
}

#[test]
fn test_range_empty_results() {
    let mut tree = BPlusTreeMap::new(16).unwrap();
    
    // Insert test data
    for i in 0..10 {
        tree.insert(i * 10, format!("value_{}", i * 10)); // 0, 10, 20, ..., 90
    }
    
    // Test range that should be empty
    let empty_range: Vec<_> = tree.range(5..8).map(|(k, _)| *k).collect();
    assert_eq!(empty_range, vec![]);
    
    // Test range beyond all keys
    let beyond_range: Vec<_> = tree.range(100..200).map(|(k, _)| *k).collect();
    assert_eq!(beyond_range, vec![]);
    
    // Test range before all keys
    let before_range: Vec<_> = tree.range(-10..-1).map(|(k, _)| *k).collect();
    assert_eq!(before_range, vec![]);
}
