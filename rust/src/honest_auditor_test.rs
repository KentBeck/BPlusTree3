use std::collections::BTreeMap;
use crate::GlobalCapacityBPlusTreeMap;
use std::time::Instant;

/// Honest auditor experiment that acknowledges performance reality
pub fn run_honest_auditor_experiment() {
    println!("=== HONEST AUDITOR EXPERIMENT: Range Scanning Performance ===");
    println!("This experiment provides unbiased comparison between our B+ tree and std::BTreeMap");
    println!();
    
    // Test different scenarios to understand where we win/lose
    test_range_creation_overhead();
    test_small_range_performance();
    test_large_range_performance();
    test_memory_usage_patterns();
    test_different_data_types();
    
    println!("=== CONCLUSIONS ===");
    println!("Based on honest benchmarking:");
    println!("1. std::BTreeMap is generally faster for range queries");
    println!("2. Our B+ tree has higher overhead per operation");
    println!("3. Arena-based allocation adds indirection costs");
    println!("4. However, our tree may have advantages in specific scenarios");
}

fn test_range_creation_overhead() {
    println!("--- Range Creation Overhead Test ---");
    
    let tree_size = 10_000;
    
    // Create trees
    let mut our_tree = GlobalCapacityBPlusTreeMap::new(16).expect("Failed to create tree");
    let mut std_tree = BTreeMap::new();
    
    for i in 0..tree_size {
        our_tree.insert(i, i).expect("Failed to insert");
        std_tree.insert(i, i);
    }
    
    let iterations = 100_000;
    
    // Test range creation only (no iteration)
    let start = Instant::now();
    for i in 0..iterations {
        let start_key = i % (tree_size - 100);
        let _range = our_tree.range(start_key..start_key + 50);
        // Don't iterate, just create the range
    }
    let our_time = start.elapsed();
    
    let start = Instant::now();
    for i in 0..iterations {
        let start_key = i % (tree_size - 100);
        let _range = std_tree.range(start_key..start_key + 50);
        // Don't iterate, just create the range
    }
    let std_time = start.elapsed();
    
    println!("Range creation ({}k iterations):", iterations / 1000);
    println!("  Our B+ tree: {:?} ({:.1}ns per range)", our_time, our_time.as_nanos() as f64 / iterations as f64);
    println!("  std::BTreeMap: {:?} ({:.1}ns per range)", std_time, std_time.as_nanos() as f64 / iterations as f64);
    
    if our_time > std_time {
        let slowdown = our_time.as_nanos() as f64 / std_time.as_nanos() as f64;
        println!("  ❌ Our tree is {:.1}x SLOWER at range creation", slowdown);
    } else {
        let speedup = std_time.as_nanos() as f64 / our_time.as_nanos() as f64;
        println!("  ✅ Our tree is {:.1}x faster at range creation", speedup);
    }
    println!();
}

fn test_small_range_performance() {
    println!("--- Small Range Performance Test ---");
    
    let tree_size = 10_000;
    let range_size = 100;
    
    // Create trees
    let mut our_tree = GlobalCapacityBPlusTreeMap::new(16).expect("Failed to create tree");
    let mut std_tree = BTreeMap::new();
    
    for i in 0..tree_size {
        our_tree.insert(i, format!("value_{}", i)).expect("Failed to insert");
        std_tree.insert(i, format!("value_{}", i));
    }
    
    let iterations = 10_000;
    let start_key = tree_size / 4;
    
    // Test our tree
    let start = Instant::now();
    for _ in 0..iterations {
        let count: usize = our_tree.range(start_key..start_key + range_size).count();
        assert_eq!(count, range_size);
    }
    let our_time = start.elapsed();
    
    // Test std tree
    let start = Instant::now();
    for _ in 0..iterations {
        let count: usize = std_tree.range(start_key..start_key + range_size).count();
        assert_eq!(count, range_size);
    }
    let std_time = start.elapsed();
    
    println!("Small range queries ({} items, {} iterations):", range_size, iterations);
    println!("  Our B+ tree: {:?} ({:.1}µs per query)", our_time, our_time.as_micros() as f64 / iterations as f64);
    println!("  std::BTreeMap: {:?} ({:.1}µs per query)", std_time, std_time.as_micros() as f64 / iterations as f64);
    
    if our_time > std_time {
        let slowdown = our_time.as_nanos() as f64 / std_time.as_nanos() as f64;
        println!("  ❌ Our tree is {:.1}x SLOWER", slowdown);
    } else {
        let speedup = std_time.as_nanos() as f64 / our_time.as_nanos() as f64;
        println!("  ✅ Our tree is {:.1}x faster", speedup);
    }
    println!();
}

fn test_large_range_performance() {
    println!("--- Large Range Performance Test ---");
    
    let tree_size = 100_000;
    let range_size = 10_000;
    
    // Create trees
    let mut our_tree = GlobalCapacityBPlusTreeMap::new(16).expect("Failed to create tree");
    let mut std_tree = BTreeMap::new();
    
    for i in 0..tree_size {
        our_tree.insert(i, i * 2).expect("Failed to insert");
        std_tree.insert(i, i * 2);
    }
    
    let iterations = 1_000;
    let start_key = tree_size / 4;
    
    // Test our tree
    let start = Instant::now();
    for _ in 0..iterations {
        let count: usize = our_tree.range(start_key..start_key + range_size).count();
        assert_eq!(count, range_size);
    }
    let our_time = start.elapsed();
    
    // Test std tree
    let start = Instant::now();
    for _ in 0..iterations {
        let count: usize = std_tree.range(start_key..start_key + range_size).count();
        assert_eq!(count, range_size);
    }
    let std_time = start.elapsed();
    
    println!("Large range queries ({} items, {} iterations):", range_size, iterations);
    println!("  Our B+ tree: {:?} ({:.1}ms per query)", our_time, our_time.as_millis() as f64 / iterations as f64);
    println!("  std::BTreeMap: {:?} ({:.1}ms per query)", std_time, std_time.as_millis() as f64 / iterations as f64);
    
    if our_time > std_time {
        let slowdown = our_time.as_nanos() as f64 / std_time.as_nanos() as f64;
        println!("  ❌ Our tree is {:.1}x SLOWER", slowdown);
    } else {
        let speedup = std_time.as_nanos() as f64 / our_time.as_nanos() as f64;
        println!("  ✅ Our tree is {:.1}x faster", speedup);
    }
    println!();
}

fn test_memory_usage_patterns() {
    println!("--- Memory Usage Analysis ---");
    
    let tree_size = 50_000usize;
    
    // Create trees
    let mut our_tree = GlobalCapacityBPlusTreeMap::new(16).expect("Failed to create tree");
    let mut std_tree = BTreeMap::new();
    
    for i in 0..tree_size {
        let i = i as i32;
        our_tree.insert(i, i).expect("Failed to insert");
        std_tree.insert(i, i);
    }
    
    // Estimate memory usage
    let our_memory = estimate_our_tree_memory(&our_tree, tree_size);
    let std_memory = estimate_std_tree_memory(tree_size);

    println!("Estimated memory usage for {} items:", tree_size);
    println!("  Our B+ tree: ~{} bytes", our_memory);
    println!("  std::BTreeMap: ~{} bytes", std_memory);
    
    if our_memory > std_memory {
        let overhead = (our_memory as f64 / std_memory as f64 - 1.0) * 100.0;
        println!("  ❌ Our tree uses {:.1}% more memory", overhead);
    } else {
        let savings = (1.0 - our_memory as f64 / std_memory as f64) * 100.0;
        println!("  ✅ Our tree uses {:.1}% less memory", savings);
    }
    println!();
}

fn test_different_data_types() {
    println!("--- Data Type Impact Test ---");
    
    // Test with minimal data (i32 -> i32)
    test_data_type_performance::<i32, i32>("i32 -> i32", |i| i, |i| i * 2);
    
    // Test with string data (i32 -> String)
    test_data_type_performance::<i32, String>("i32 -> String", |i| i, |i| format!("value_{}", i));
}

fn test_data_type_performance<K, V>(
    description: &str,
    key_gen: impl Fn(i32) -> K,
    value_gen: impl Fn(i32) -> V,
) where
    K: Ord + Clone,
    V: Clone,
{
    let tree_size = 10_000;
    let range_size = 1_000;
    
    // Create trees
    let mut our_tree = GlobalCapacityBPlusTreeMap::new(16).expect("Failed to create tree");
    let mut std_tree = BTreeMap::new();
    
    for i in 0..tree_size {
        let key = key_gen(i);
        let value = value_gen(i);
        our_tree.insert(key.clone(), value.clone()).expect("Failed to insert");
        std_tree.insert(key, value);
    }
    
    let iterations = 1_000;
    let start_key = key_gen(tree_size / 4);
    let end_key = key_gen(tree_size / 4 + range_size);
    
    // Test our tree
    let start = Instant::now();
    for _ in 0..iterations {
        let count: usize = our_tree.range(start_key.clone()..end_key.clone()).count();
        assert_eq!(count, range_size as usize);
    }
    let our_time = start.elapsed();
    
    // Test std tree
    let start = Instant::now();
    for _ in 0..iterations {
        let count: usize = std_tree.range(start_key.clone()..end_key.clone()).count();
        assert_eq!(count, range_size as usize);
    }
    let std_time = start.elapsed();
    
    println!("  {} ({} items, {} iterations):", description, range_size, iterations);
    println!("    Our B+ tree: {:?}", our_time);
    println!("    std::BTreeMap: {:?}", std_time);
    
    if our_time > std_time {
        let slowdown = our_time.as_nanos() as f64 / std_time.as_nanos() as f64;
        println!("    ❌ Our tree is {:.1}x SLOWER", slowdown);
    } else {
        let speedup = std_time.as_nanos() as f64 / our_time.as_nanos() as f64;
        println!("    ✅ Our tree is {:.1}x faster", speedup);
    }
}

fn estimate_our_tree_memory(_tree: &GlobalCapacityBPlusTreeMap<i32, i32>, size: usize) -> usize {
    // Rough estimate based on arena allocation
    let items_per_leaf = 16;
    let leaves = (size + items_per_leaf - 1) / items_per_leaf;
    let leaf_size = std::mem::size_of::<i32>() * 2 * items_per_leaf + 64; // keys + values + overhead
    let branch_nodes = leaves / items_per_leaf + 1;
    let branch_size = std::mem::size_of::<i32>() * items_per_leaf + 64; // keys + node IDs + overhead
    
    leaves * leaf_size + branch_nodes * branch_size
}

fn estimate_std_tree_memory(size: usize) -> usize {
    // Rough estimate for std::BTreeMap
    let node_size = 64; // Approximate node size
    let nodes = size / 6 + 1; // B-tree typically has ~6 items per node
    nodes * node_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_honest_auditor_experiment() {
        run_honest_auditor_experiment();
    }
}
