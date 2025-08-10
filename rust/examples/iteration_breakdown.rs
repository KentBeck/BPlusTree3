use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::time::Instant;

fn main() {
    println!("Iteration Breakdown Analysis");
    println!("============================");
    
    let size = 10000;
    let mut bplus = BPlusTreeMap::new(16).unwrap();
    
    for i in 0..size {
        bplus.insert(i, i * 2);
    }
    
    println!("Analyzing BPlusTreeMap iteration components...");
    println!();
    
    // Test 1: Iterator creation overhead
    let iterations = 100000;
    let start = Instant::now();
    for _ in 0..iterations {
        let iter = bplus.items();
        std::hint::black_box(iter);
    }
    let creation_time = start.elapsed();
    println!("Iterator creation ({} times): {:?}", iterations, creation_time);
    println!("Per creation: {:.2} ns", creation_time.as_nanos() as f64 / iterations as f64);
    
    // Test 2: First element access (tree traversal to leftmost leaf)
    let start = Instant::now();
    for _ in 0..iterations {
        let mut iter = bplus.items();
        let first = iter.next();
        std::hint::black_box(first);
    }
    let first_access_time = start.elapsed();
    println!();
    println!("First element access ({} times): {:?}", iterations, first_access_time);
    println!("Per first access: {:.2} ns", first_access_time.as_nanos() as f64 / iterations as f64);
    
    // Test 3: Sequential access within same leaf
    let start = Instant::now();
    let test_iterations = 1000;
    for _ in 0..test_iterations {
        let mut iter = bplus.items();
        // Skip to first element
        iter.next();
        // Access next 15 elements (likely same leaf with capacity 16)
        for _ in 0..15 {
            let next = iter.next();
            std::hint::black_box(next);
        }
    }
    let same_leaf_time = start.elapsed();
    println!();
    println!("Same-leaf sequential access ({} * 15): {:?}", test_iterations, same_leaf_time);
    println!("Per same-leaf access: {:.2} ns", same_leaf_time.as_nanos() as f64 / (test_iterations * 15) as f64);
    
    // Test 4: Cross-leaf access (arena lookup overhead)
    let start = Instant::now();
    for _ in 0..test_iterations {
        let mut iter = bplus.items();
        // Skip 16 elements to force leaf boundary crossing
        for _ in 0..16 {
            iter.next();
        }
        // Access the 17th element (different leaf)
        let cross_leaf = iter.next();
        std::hint::black_box(cross_leaf);
    }
    let cross_leaf_time = start.elapsed();
    println!();
    println!("Cross-leaf access ({} times): {:?}", test_iterations, cross_leaf_time);
    println!("Per cross-leaf access: {:.2} ns", cross_leaf_time.as_nanos() as f64 / test_iterations as f64);
    
    // Test 5: Arena lookup overhead
    let start = Instant::now();
    let lookup_iterations = 100000;
    for i in 0..lookup_iterations {
        // Simulate arena lookups by accessing different leaves
        let leaf_id = (i % (size / 16)) as u32; // Approximate leaf count
        // We can't directly test arena lookup without exposing internals,
        // but we can measure iterator advancement which includes it
        let mut iter = bplus.items();
        for _ in 0..(leaf_id * 8) { // Skip to different positions
            if iter.next().is_none() { break; }
        }
        std::hint::black_box(iter);
    }
    let arena_simulation_time = start.elapsed();
    println!();
    println!("Arena access simulation ({} times): {:?}", lookup_iterations, arena_simulation_time);
    println!("Per simulation: {:.2} ns", arena_simulation_time.as_nanos() as f64 / lookup_iterations as f64);
    
    // Test 6: Compare with simple Vec iteration (baseline)
    let vec_data: Vec<(i32, i32)> = (0..size).map(|i| (i, i * 2)).collect();
    let start = Instant::now();
    for _ in 0..1000 {
        for (k, v) in &vec_data {
            std::hint::black_box((k, v));
        }
    }
    let vec_time = start.elapsed();
    println!();
    println!("=== BASELINE COMPARISON ===");
    println!("Vec iteration (1000 * {} items): {:?}", size, vec_time);
    println!("Vec per item: {:.2} ns", vec_time.as_nanos() as f64 / (1000 * size) as f64);
    
    // Full BPlusTree iteration for comparison
    let start = Instant::now();
    for _ in 0..1000 {
        for (k, v) in bplus.items() {
            std::hint::black_box((k, v));
        }
    }
    let bplus_full_time = start.elapsed();
    println!("BPlusTree iteration (1000 * {} items): {:?}", size, bplus_full_time);
    println!("BPlusTree per item: {:.2} ns", bplus_full_time.as_nanos() as f64 / (1000 * size) as f64);
    
    let overhead = (bplus_full_time.as_nanos() as f64 / (1000 * size) as f64) - 
                   (vec_time.as_nanos() as f64 / (1000 * size) as f64);
    println!("BPlusTree overhead vs Vec: {:.2} ns per item", overhead);
    
    println!();
    println!("=== ANALYSIS SUMMARY ===");
    println!("The BPlusTree iteration overhead appears to come from:");
    println!("1. Arena-based memory access patterns");
    println!("2. Linked list traversal between leaves");
    println!("3. Iterator state management and bounds checking");
    println!("4. Less cache-friendly memory layout compared to BTreeMap's direct node traversal");
}
