//! Test to verify leaf caching optimization performance improvement

use crate::global_capacity_tree::GlobalCapacityBPlusTreeMap;
use std::time::Instant;

pub fn test_leaf_cache_performance() {
    println!("=== LEAF CACHE PERFORMANCE TEST ===");
    
    let mut tree = GlobalCapacityBPlusTreeMap::new(16).unwrap();
    
    // Insert a large dataset
    let n = 100_000;
    println!("Inserting {} items...", n);
    for i in 0..n {
        tree.insert(i, i * 10).unwrap();
    }
    
    // Test different range sizes to verify optimization
    test_range_performance(&tree, 1000, "1K items");
    test_range_performance(&tree, 10_000, "10K items");
    test_range_performance(&tree, 50_000, "50K items");
    
    // Test multiple small ranges (should be very fast with caching)
    test_multiple_small_ranges(&tree);
}

fn test_range_performance(tree: &GlobalCapacityBPlusTreeMap<i32, i32>, range_size: i32, description: &str) {
    println!("\n--- {} Range Test ---", description);
    
    let start_key = 25_000; // Start from middle
    let end_key = start_key + range_size;
    
    // Time the range query
    let start = Instant::now();
    let items: Vec<_> = tree.range(start_key..end_key).collect();
    let elapsed = start.elapsed();
    
    let items_per_leaf = 16; // capacity
    let expected_leaves = (range_size + items_per_leaf - 1) / items_per_leaf;
    
    println!("Range: {}..{}", start_key, end_key);
    println!("Items collected: {}", items.len());
    println!("Expected leaves traversed: {}", expected_leaves);
    println!("Time: {:?}", elapsed);
    println!("Time per item: {:.2}ns", elapsed.as_nanos() as f64 / items.len() as f64);
    
    // With leaf caching, we should have excellent performance
    let ns_per_item = elapsed.as_nanos() as f64 / items.len() as f64;
    if ns_per_item < 100.0 {
        println!("✅ EXCELLENT: < 100ns per item (leaf caching working!)");
    } else if ns_per_item < 200.0 {
        println!("✅ GOOD: < 200ns per item");
    } else {
        println!("⚠️  SLOW: >= 200ns per item (leaf caching may not be working)");
    }
}

fn test_multiple_small_ranges(tree: &GlobalCapacityBPlusTreeMap<i32, i32>) {
    println!("\n--- Multiple Small Ranges Test ---");
    
    let num_ranges = 1000;
    let range_size = 10;
    
    println!("Testing {} small ranges of {} items each", num_ranges, range_size);
    
    let start = Instant::now();
    let mut total_items = 0;
    
    for i in 0..num_ranges {
        let start_key = i * 50; // Spread out ranges
        let end_key = start_key + range_size;
        let items: Vec<_> = tree.range(start_key..end_key).collect();
        total_items += items.len();
    }
    
    let elapsed = start.elapsed();
    
    println!("Total items: {}", total_items);
    println!("Total time: {:?}", elapsed);
    println!("Time per range: {:.2}µs", elapsed.as_micros() as f64 / num_ranges as f64);
    println!("Time per item: {:.2}ns", elapsed.as_nanos() as f64 / total_items as f64);
    
    // Multiple small ranges should be very fast with caching
    let ns_per_item = elapsed.as_nanos() as f64 / total_items as f64;
    if ns_per_item < 150.0 {
        println!("✅ EXCELLENT: Multiple small ranges are very fast!");
    } else {
        println!("⚠️  Could be faster - check leaf caching implementation");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_leaf_cache_optimization() {
        test_leaf_cache_performance();
    }
}
