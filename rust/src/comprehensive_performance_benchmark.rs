use crate::BPlusTreeMap;
use std::collections::BTreeMap;
use std::time::Instant;

/// Comprehensive performance benchmark comparing BPlusTreeMap vs BTreeMap
/// Tests insert, delete, access, and iterate operations on large datasets
pub fn run_comprehensive_benchmark() {
    println!("=== COMPREHENSIVE PERFORMANCE BENCHMARK ===");
    println!("BPlusTreeMap vs BTreeMap - Large Tree & Large Capacity\n");
    
    let tree_size = 1_000_000;
    let capacity = 2048; // Large capacity
    let sample_size = 10_000; // Operations to benchmark
    
    println!("Configuration:");
    println!("  Tree size: {} items", tree_size);
    println!("  BPlusTreeMap capacity: {}", capacity);
    println!("  Sample operations: {}", sample_size);
    println!();
    
    // Create and populate trees
    println!("🔧 Setting up trees...");
    let (bplus, btree) = setup_trees(tree_size, capacity);
    
    println!("📊 Running benchmarks...\n");
    
    // Test each operation
    benchmark_access(&bplus, &btree, tree_size, sample_size);
    benchmark_insert(&bplus, &btree, tree_size, sample_size);
    benchmark_delete(&bplus, &btree, tree_size, sample_size);
    benchmark_iterate(&bplus, &btree, sample_size);
    
    println!("\n=== BENCHMARK COMPLETE ===");
}

fn setup_trees(size: usize, capacity: usize) -> (BPlusTreeMap<usize, usize>, BTreeMap<usize, usize>) {
    let mut bplus = BPlusTreeMap::new(capacity).unwrap();
    let mut btree = BTreeMap::new();
    
    // Populate with sequential data
    for i in 0..size {
        bplus.insert(i, i * 2);
        btree.insert(i, i * 2);
    }
    
    (bplus, btree)
}

fn benchmark_access(bplus: &BPlusTreeMap<usize, usize>, btree: &BTreeMap<usize, usize>, tree_size: usize, sample_size: usize) {
    println!("🔍 ACCESS Performance:");
    
    // Generate random keys for access
    let keys: Vec<usize> = (0..sample_size)
        .map(|i| (i * 997) % tree_size) // Pseudo-random distribution
        .collect();
    
    // Benchmark BPlusTreeMap access
    let start = Instant::now();
    for &key in &keys {
        let _ = bplus.get(&key);
    }
    let bplus_time = start.elapsed();
    
    // Benchmark BTreeMap access
    let start = Instant::now();
    for &key in &keys {
        let _ = btree.get(&key);
    }
    let btree_time = start.elapsed();
    
    let bplus_per_op = bplus_time.as_nanos() as f64 / sample_size as f64;
    let btree_per_op = btree_time.as_nanos() as f64 / sample_size as f64;
    let speedup = btree_per_op / bplus_per_op;
    
    println!("  BPlusTreeMap: {:.1}ns per access", bplus_per_op);
    println!("  BTreeMap:     {:.1}ns per access", btree_per_op);
    println!("  Ratio:        {:.2}x {}", speedup, if speedup > 1.0 { "(BPlusTreeMap faster)" } else { "(BTreeMap faster)" });
    println!();
}

fn benchmark_insert(bplus: &BPlusTreeMap<usize, usize>, btree: &BTreeMap<usize, usize>, tree_size: usize, sample_size: usize) {
    println!("➕ INSERT Performance:");
    
    // Generate new keys for insertion (beyond existing range)
    let new_keys: Vec<usize> = (tree_size..tree_size + sample_size).collect();
    
    // Create fresh trees for insertion testing
    let capacity = bplus.capacity;
    let mut bplus_copy = BPlusTreeMap::new(capacity).unwrap();
    let mut btree_copy = BTreeMap::new();
    
    // Pre-populate with original data
    for i in 0..tree_size {
        bplus_copy.insert(i, i * 2);
        btree_copy.insert(i, i * 2);
    }
    
    // Benchmark BPlusTreeMap insert
    let start = Instant::now();
    for &key in &new_keys {
        bplus_copy.insert(key, key * 2);
    }
    let bplus_time = start.elapsed();
    
    // Reset and benchmark BTreeMap insert
    btree_copy.clear();
    for i in 0..tree_size {
        btree_copy.insert(i, i * 2);
    }
    
    let start = Instant::now();
    for &key in &new_keys {
        btree_copy.insert(key, key * 2);
    }
    let btree_time = start.elapsed();

    let bplus_per_op = bplus_time.as_nanos() as f64 / sample_size as f64;
    let btree_per_op = btree_time.as_nanos() as f64 / sample_size as f64;
    let speedup = btree_per_op / bplus_per_op;
    
    println!("  BPlusTreeMap: {:.1}ns per insert", bplus_per_op);
    println!("  BTreeMap:     {:.1}ns per insert", btree_per_op);
    println!("  Ratio:        {:.2}x {}", speedup, if speedup > 1.0 { "(BPlusTreeMap faster)" } else { "(BTreeMap faster)" });
    println!();
}

fn benchmark_delete(bplus: &BPlusTreeMap<usize, usize>, btree: &BTreeMap<usize, usize>, tree_size: usize, sample_size: usize) {
    println!("➖ DELETE Performance:");
    
    // Generate keys to delete (from existing range)
    let delete_keys: Vec<usize> = (0..sample_size)
        .map(|i| (i * 991) % tree_size) // Pseudo-random distribution
        .collect();
    
    // Create fresh trees for deletion testing
    let capacity = bplus.capacity;
    let mut bplus_copy = BPlusTreeMap::new(capacity).unwrap();
    let mut btree_copy = BTreeMap::new();
    
    // Pre-populate with original data
    for i in 0..tree_size {
        bplus_copy.insert(i, i * 2);
        btree_copy.insert(i, i * 2);
    }
    
    // Benchmark BPlusTreeMap delete
    let start = Instant::now();
    for &key in &delete_keys {
        let _ = bplus_copy.remove(&key);
    }
    let bplus_time = start.elapsed();
    
    // Reset and benchmark BTreeMap delete
    btree_copy.clear();
    for i in 0..tree_size {
        btree_copy.insert(i, i * 2);
    }
    
    let start = Instant::now();
    for &key in &delete_keys {
        let _ = btree_copy.remove(&key);
    }
    let btree_time = start.elapsed();

    let bplus_per_op = bplus_time.as_nanos() as f64 / sample_size as f64;
    let btree_per_op = btree_time.as_nanos() as f64 / sample_size as f64;
    let speedup = btree_per_op / bplus_per_op;
    
    println!("  BPlusTreeMap: {:.1}ns per delete", bplus_per_op);
    println!("  BTreeMap:     {:.1}ns per delete", btree_per_op);
    println!("  Ratio:        {:.2}x {}", speedup, if speedup > 1.0 { "(BPlusTreeMap faster)" } else { "(BTreeMap faster)" });
    println!();
}

fn benchmark_iterate(bplus: &BPlusTreeMap<usize, usize>, btree: &BTreeMap<usize, usize>, sample_size: usize) {
    println!("🔄 ITERATE Performance:");
    
    let iterations = 100;
    
    // Benchmark BPlusTreeMap iteration (range)
    let start_key = 100_000;
    let end_key = start_key + sample_size;
    
    let start = Instant::now();
    for _ in 0..iterations {
        for (_k, _v) in bplus.items_range(Some(&start_key), Some(&end_key)) {
            // Consume iterator
        }
    }
    let bplus_time = start.elapsed();
    
    // Benchmark BTreeMap iteration (range)
    let start = Instant::now();
    for _ in 0..iterations {
        for (_k, _v) in btree.range(start_key..=end_key) {
            // Consume iterator
        }
    }
    let btree_time = start.elapsed();
    
    let bplus_per_item = bplus_time.as_nanos() as f64 / (iterations * sample_size) as f64;
    let btree_per_item = btree_time.as_nanos() as f64 / (iterations * sample_size) as f64;
    let speedup = btree_per_item / bplus_per_item;
    
    println!("  BPlusTreeMap: {:.1}ns per item", bplus_per_item);
    println!("  BTreeMap:     {:.1}ns per item", btree_per_item);
    println!("  Ratio:        {:.2}x {}", speedup, if speedup > 1.0 { "(BPlusTreeMap faster)" } else { "(BTreeMap faster)" });
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comprehensive_benchmark() {
        run_comprehensive_benchmark();
    }
}
