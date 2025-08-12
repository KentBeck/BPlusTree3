use std::time::{Duration, Instant};
use bplustree::BPlusTreeMap;

fn main() {
    println!("=== Custom Timing Analysis for Range Scans ===");
    
    let tree_size = 1_000_000;
    let range_size = 100_000;
    
    // Build tree
    println!("Building tree with {} items...", tree_size);
    let start_build = Instant::now();
    let mut tree = BPlusTreeMap::new(64).unwrap();
    for i in 0..tree_size {
        tree.insert(i, format!("value_{}", i));
    }
    let build_time = start_build.elapsed();
    println!("Tree build time: {:?}", build_time);
    
    // Test different range sizes
    let range_sizes = vec![100, 1_000, 10_000, 50_000, 100_000];
    
    for &size in &range_sizes {
        let start = tree_size / 4;
        let end = start + size;
        
        // Warm up
        for _ in 0..3 {
            let _: Vec<_> = tree.range(start..end).collect();
        }
        
        // Time the operation
        let iterations = if size < 10_000 { 100 } else { 10 };
        let start_time = Instant::now();
        
        for _ in 0..iterations {
            let items: Vec<_> = tree.range(start..end).collect();
            std::hint::black_box(items);
        }
        
        let elapsed = start_time.elapsed();
        let avg_time = elapsed / iterations;
        let items_per_sec = (size as f64) / avg_time.as_secs_f64();
        
        println!("Range size {:6}: {:8.2?} avg, {:10.0} items/sec", 
                 size, avg_time, items_per_sec);
    }
    
    // Test range iteration vs collection
    let range_size = 50_000;
    let start = tree_size / 4;
    let end = start + range_size;
    
    println!("\n=== Range Iteration Patterns ===");
    
    // Just iterate (don't collect)
    let start_time = Instant::now();
    for _ in 0..10 {
        let mut count = 0;
        for (k, v) in tree.range(start..end) {
            std::hint::black_box(k);
            std::hint::black_box(v);
            count += 1;
        }
        std::hint::black_box(count);
    }
    let iterate_time = start_time.elapsed() / 10;
    
    // Collect all
    let start_time = Instant::now();
    for _ in 0..10 {
        let items: Vec<_> = tree.range(start..end).collect();
        std::hint::black_box(items);
    }
    let collect_time = start_time.elapsed() / 10;
    
    // Count only
    let start_time = Instant::now();
    for _ in 0..10 {
        let count = tree.range(start..end).count();
        std::hint::black_box(count);
    }
    let count_time = start_time.elapsed() / 10;
    
    println!("Iterate only: {:8.2?}", iterate_time);
    println!("Collect all:  {:8.2?}", collect_time);
    println!("Count only:   {:8.2?}", count_time);
    
    println!("\nCollection overhead: {:.1}x", 
             collect_time.as_secs_f64() / iterate_time.as_secs_f64());
}
