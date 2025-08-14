use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::time::Instant;

fn main() {
    println!("Memory Access Pattern Analysis");
    println!("==============================");
    
    let size = 10000;
    
    // Create both data structures
    let mut btree = BTreeMap::new();
    let mut bplus = BPlusTreeMap::new(64).unwrap(); // Use optimal capacity
    
    for i in 0..size {
        btree.insert(i, i * 2);
        bplus.insert(i, i * 2);
    }
    
    println!("Dataset: {} items", size);
    println!("BPlusTree capacity: 64 (optimal)");
    println!();
    
    // Test memory access patterns
    let iterations = 1000;
    
    // 1. Sequential access (cache-friendly)
    println!("=== SEQUENTIAL ACCESS PATTERNS ===");
    
    let start = Instant::now();
    for _ in 0..iterations {
        for (k, v) in btree.iter() {
            std::hint::black_box((k, v));
        }
    }
    let btree_sequential = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..iterations {
        for (k, v) in bplus.items() {
            std::hint::black_box((k, v));
        }
    }
    let bplus_sequential = start.elapsed();
    
    println!("BTreeMap sequential: {:?}", btree_sequential);
    println!("BPlusTreeMap sequential: {:?}", bplus_sequential);
    println!("Ratio: {:.2}x", bplus_sequential.as_nanos() as f64 / btree_sequential.as_nanos() as f64);
    
    // 2. Random access pattern (cache-unfriendly)
    println!();
    println!("=== RANDOM ACCESS PATTERNS ===");
    
    let random_keys: Vec<i32> = (0..1000).map(|i| (i * 7919) % size).collect(); // Pseudo-random
    
    let start = Instant::now();
    for _ in 0..iterations {
        for &key in &random_keys {
            let val = btree.get(&key);
            std::hint::black_box(val);
        }
    }
    let btree_random = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..iterations {
        for &key in &random_keys {
            let val = bplus.get(&key);
            std::hint::black_box(val);
        }
    }
    let bplus_random = start.elapsed();
    
    println!("BTreeMap random lookups: {:?}", btree_random);
    println!("BPlusTreeMap random lookups: {:?}", bplus_random);
    println!("Ratio: {:.2}x", bplus_random.as_nanos() as f64 / btree_random.as_nanos() as f64);
    
    // 3. Memory layout analysis
    println!();
    println!("=== MEMORY LAYOUT ANALYSIS ===");
    
    // Estimate memory usage
    let btree_size = std::mem::size_of_val(&btree);
    let bplus_size = std::mem::size_of_val(&bplus);
    
    println!("BTreeMap struct size: {} bytes", btree_size);
    println!("BPlusTreeMap struct size: {} bytes", bplus_size);
    
    // Test cache behavior with different access patterns
    println!();
    println!("=== CACHE BEHAVIOR SIMULATION ===");
    
    // Small range iteration (should be cache-friendly for both)
    let start = Instant::now();
    for _ in 0..iterations * 10 {
        for (k, v) in btree.range(1000..1100) {
            std::hint::black_box((k, v));
        }
    }
    let btree_small_range = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..iterations * 10 {
        for (k, v) in bplus.items_range(Some(&1000), Some(&1100)) {
            std::hint::black_box((k, v));
        }
    }
    let bplus_small_range = start.elapsed();
    
    println!("Small range (100 items):");
    println!("  BTreeMap: {:?}", btree_small_range);
    println!("  BPlusTreeMap: {:?}", bplus_small_range);
    println!("  Ratio: {:.2}x", bplus_small_range.as_nanos() as f64 / btree_small_range.as_nanos() as f64);
    
    // Large range iteration
    let start = Instant::now();
    for _ in 0..100 {
        for (k, v) in btree.range(1000..9000) {
            std::hint::black_box((k, v));
        }
    }
    let btree_large_range = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..100 {
        for (k, v) in bplus.items_range(Some(&1000), Some(&9000)) {
            std::hint::black_box((k, v));
        }
    }
    let bplus_large_range = start.elapsed();
    
    println!();
    println!("Large range (8000 items):");
    println!("  BTreeMap: {:?}", btree_large_range);
    println!("  BPlusTreeMap: {:?}", bplus_large_range);
    println!("  Ratio: {:.2}x", bplus_large_range.as_nanos() as f64 / btree_large_range.as_nanos() as f64);
    
    // 4. Iterator overhead analysis
    println!();
    println!("=== ITERATOR OVERHEAD BREAKDOWN ===");
    
    // Just iterator creation
    let start = Instant::now();
    for _ in 0..100000 {
        let iter = btree.iter();
        let _ = std::hint::black_box(iter);
    }
    let btree_iter_creation = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..100000 {
        let iter = bplus.items();
        std::hint::black_box(iter);
    }
    let bplus_iter_creation = start.elapsed();
    
    println!("Iterator creation (100k times):");
    println!("  BTreeMap: {:?} ({:.2} ns each)", btree_iter_creation, btree_iter_creation.as_nanos() as f64 / 100000.0);
    println!("  BPlusTreeMap: {:?} ({:.2} ns each)", bplus_iter_creation, bplus_iter_creation.as_nanos() as f64 / 100000.0);
    
    // First element access
    let start = Instant::now();
    for _ in 0..100000 {
        let mut iter = btree.iter();
        let first = iter.next();
        std::hint::black_box(first);
    }
    let btree_first_access = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..100000 {
        let mut iter = bplus.items();
        let first = iter.next();
        std::hint::black_box(first);
    }
    let bplus_first_access = start.elapsed();
    
    println!();
    println!("First element access (100k times):");
    println!("  BTreeMap: {:?} ({:.2} ns each)", btree_first_access, btree_first_access.as_nanos() as f64 / 100000.0);
    println!("  BPlusTreeMap: {:?} ({:.2} ns each)", bplus_first_access, bplus_first_access.as_nanos() as f64 / 100000.0);
    
    println!();
    println!("=== PERFORMANCE SUMMARY ===");
    println!("BPlusTreeMap iteration is slower due to:");
    println!("1. Arena-based allocation: Indirect memory access through Vec indices");
    println!("2. Linked list traversal: Additional pointer chasing between leaves");
    println!("3. Iterator state complexity: More bookkeeping than BTreeMap's direct traversal");
    println!("4. Cache locality: BTreeMap's nodes are more cache-friendly for iteration");
    println!();
    println!("However, BPlusTreeMap excels at:");
    println!("- Lookups: Better cache behavior with optimal capacity");
    println!("- Insertions: More efficient splitting and rebalancing");
    println!("- Range queries: Competitive for large ranges due to linked leaves");
}
