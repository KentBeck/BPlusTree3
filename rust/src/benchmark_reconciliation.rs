use std::collections::BTreeMap;
use crate::GlobalCapacityBPlusTreeMap;
use std::time::Instant;

/// Reconcile the discrepancy between different benchmark results
pub fn investigate_benchmark_discrepancy() {
    println!("=== BENCHMARK DISCREPANCY INVESTIGATION ===");
    println!("Comparing different measurement approaches to understand why results vary");
    println!();
    
    // Test the exact same scenario with different measurement methods
    let tree_size = 10_000;
    let range_size = 1_000;
    
    // Create identical trees
    let mut our_tree = GlobalCapacityBPlusTreeMap::new(16).expect("Failed to create tree");
    let mut std_tree = BTreeMap::new();
    
    for i in 0..tree_size {
        our_tree.insert(i, i).expect("Failed to insert");
        std_tree.insert(i, i);
    }
    
    let start_key = tree_size / 4;
    let end_key = start_key + range_size;
    
    println!("Test scenario: {} items in tree, {} item range query", tree_size, range_size);
    println!();
    
    // Method 1: Simple timing (like my honest test)
    test_simple_timing(&our_tree, &std_tree, start_key, end_key);
    
    // Method 2: Criterion-style micro-benchmarking
    test_criterion_style(&our_tree, &std_tree, start_key, end_key);
    
    // Method 3: Different operations
    test_different_operations(&our_tree, &std_tree, start_key, end_key);
    
    // Method 4: Cold vs warm cache
    test_cache_effects(&our_tree, &std_tree, start_key, end_key);
}

fn test_simple_timing(
    our_tree: &GlobalCapacityBPlusTreeMap<i32, i32>,
    std_tree: &BTreeMap<i32, i32>,
    start_key: i32,
    end_key: i32,
) {
    println!("--- Method 1: Simple Timing (1000 iterations) ---");
    
    let iterations = 1_000;
    
    // Our tree
    let start = Instant::now();
    for _ in 0..iterations {
        let count: usize = our_tree.range(start_key..end_key).count();
        assert_eq!(count, (end_key - start_key) as usize);
    }
    let our_time = start.elapsed();
    
    // Std tree
    let start = Instant::now();
    for _ in 0..iterations {
        let count: usize = std_tree.range(start_key..end_key).count();
        assert_eq!(count, (end_key - start_key) as usize);
    }
    let std_time = start.elapsed();
    
    println!("  Our B+ tree: {:?} ({:.1}µs per query)", our_time, our_time.as_micros() as f64 / iterations as f64);
    println!("  std::BTreeMap: {:?} ({:.1}µs per query)", std_time, std_time.as_micros() as f64 / iterations as f64);
    
    if our_time < std_time {
        let speedup = std_time.as_nanos() as f64 / our_time.as_nanos() as f64;
        println!("  ✅ Our tree is {:.1}x FASTER", speedup);
    } else {
        let slowdown = our_time.as_nanos() as f64 / std_time.as_nanos() as f64;
        println!("  ❌ Our tree is {:.1}x SLOWER", slowdown);
    }
    println!();
}

fn test_criterion_style(
    our_tree: &GlobalCapacityBPlusTreeMap<i32, i32>,
    std_tree: &BTreeMap<i32, i32>,
    start_key: i32,
    end_key: i32,
) {
    println!("--- Method 2: Criterion-style Micro-benchmarking ---");
    
    // Warm up
    for _ in 0..100 {
        let _: usize = our_tree.range(start_key..end_key).count();
        let _: usize = std_tree.range(start_key..end_key).count();
    }
    
    // Measure many small iterations
    let mut our_times = Vec::new();
    let mut std_times = Vec::new();
    
    for _ in 0..100 {
        // Our tree - single iteration timing
        let start = Instant::now();
        let count: usize = our_tree.range(start_key..end_key).count();
        let our_time = start.elapsed();
        assert_eq!(count, (end_key - start_key) as usize);
        our_times.push(our_time);
        
        // Std tree - single iteration timing
        let start = Instant::now();
        let count: usize = std_tree.range(start_key..end_key).count();
        let std_time = start.elapsed();
        assert_eq!(count, (end_key - start_key) as usize);
        std_times.push(std_time);
    }
    
    // Calculate median times (more robust than mean)
    our_times.sort();
    std_times.sort();
    let our_median = our_times[our_times.len() / 2];
    let std_median = std_times[std_times.len() / 2];
    
    println!("  Our B+ tree median: {:?}", our_median);
    println!("  std::BTreeMap median: {:?}", std_median);
    
    if our_median < std_median {
        let speedup = std_median.as_nanos() as f64 / our_median.as_nanos() as f64;
        println!("  ✅ Our tree is {:.1}x FASTER", speedup);
    } else {
        let slowdown = our_median.as_nanos() as f64 / std_median.as_nanos() as f64;
        println!("  ❌ Our tree is {:.1}x SLOWER", slowdown);
    }
    println!();
}

fn test_different_operations(
    our_tree: &GlobalCapacityBPlusTreeMap<i32, i32>,
    std_tree: &BTreeMap<i32, i32>,
    start_key: i32,
    end_key: i32,
) {
    println!("--- Method 3: Different Operations ---");
    
    let iterations = 1_000;
    
    // Test 1: Count only
    println!("  Count only:");
    let start = Instant::now();
    for _ in 0..iterations {
        let _count: usize = our_tree.range(start_key..end_key).count();
    }
    let our_count_time = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _count: usize = std_tree.range(start_key..end_key).count();
    }
    let std_count_time = start.elapsed();
    
    println!("    Our tree: {:?}", our_count_time);
    println!("    std tree: {:?}", std_count_time);
    
    // Test 2: Collect all
    println!("  Collect all:");
    let start = Instant::now();
    for _ in 0..100 { // Fewer iterations due to allocation cost
        let _items: Vec<_> = our_tree.range(start_key..end_key).collect();
    }
    let our_collect_time = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..100 {
        let _items: Vec<_> = std_tree.range(start_key..end_key).collect();
    }
    let std_collect_time = start.elapsed();
    
    println!("    Our tree: {:?}", our_collect_time);
    println!("    std tree: {:?}", std_collect_time);
    
    // Test 3: First 10 items only
    println!("  First 10 items:");
    let start = Instant::now();
    for _ in 0..iterations {
        let _items: Vec<_> = our_tree.range(start_key..end_key).take(10).collect();
    }
    let our_take_time = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _items: Vec<_> = std_tree.range(start_key..end_key).take(10).collect();
    }
    let std_take_time = start.elapsed();
    
    println!("    Our tree: {:?}", our_take_time);
    println!("    std tree: {:?}", std_take_time);
    println!();
}

fn test_cache_effects(
    our_tree: &GlobalCapacityBPlusTreeMap<i32, i32>,
    std_tree: &BTreeMap<i32, i32>,
    start_key: i32,
    end_key: i32,
) {
    println!("--- Method 4: Cache Effects ---");
    
    // Cold cache test - different ranges each time
    println!("  Cold cache (different ranges):");
    let iterations = 1_000;
    
    let start = Instant::now();
    for i in 0..iterations {
        let range_start = (start_key + i * 10) % 8000; // Vary the range
        let _count: usize = our_tree.range(range_start..range_start + 100).count();
    }
    let our_cold_time = start.elapsed();
    
    let start = Instant::now();
    for i in 0..iterations {
        let range_start = (start_key + i * 10) % 8000;
        let _count: usize = std_tree.range(range_start..range_start + 100).count();
    }
    let std_cold_time = start.elapsed();
    
    println!("    Our tree: {:?}", our_cold_time);
    println!("    std tree: {:?}", std_cold_time);
    
    // Warm cache test - same range repeatedly
    println!("  Warm cache (same range):");
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _count: usize = our_tree.range(start_key..end_key).count();
    }
    let our_warm_time = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _count: usize = std_tree.range(start_key..end_key).count();
    }
    let std_warm_time = start.elapsed();
    
    println!("    Our tree: {:?}", our_warm_time);
    println!("    std tree: {:?}", std_warm_time);
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_discrepancy_investigation() {
        investigate_benchmark_discrepancy();
    }
}
