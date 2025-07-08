use bplustree::BPlusTreeMap;
use std::time::Instant;

fn main() {
    println!("=== Range Startup Performance Benchmark ===\n");
    
    // Test different tree sizes
    let tree_sizes = vec![1_000, 10_000, 100_000];
    
    for &tree_size in &tree_sizes {
        println!("Testing tree with {} elements:", tree_size);
        
        // Build tree
        let mut tree = BPlusTreeMap::new(16).unwrap();
        for i in 0..tree_size {
            tree.insert(i as i32, format!("value_{}", i));
        }
        
        // Test range creation performance
        test_range_creation_performance(&tree, tree_size);
        
        // Test range iteration performance
        test_range_iteration_performance(&tree, tree_size);
        
        println!();
    }
}

fn test_range_creation_performance(tree: &BPlusTreeMap<i32, String>, tree_size: usize) {
    let iterations = 10_000;
    let start_key = (tree_size / 2) as i32;
    
    println!("  Range Creation Performance ({} iterations):", iterations);
    
    // Test 1: Single element ranges (mostly startup cost)
    let start_time = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000) as i32;
        let _iter = tree.range(key..key+1);
        // Don't consume iterator, just measure creation cost
    }
    let creation_time = start_time.elapsed();
    
    let avg_creation_ns = creation_time.as_nanos() / iterations as u128;
    println!("    Single element range creation: {:.2}ns per range", avg_creation_ns);
    
    // Test 2: Excluded bounds (should use optimization)
    let start_time = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000) as i32;
        let _iter = tree.range(key..key+1); // Excluded end bound
    }
    let excluded_time = start_time.elapsed();
    
    let avg_excluded_ns = excluded_time.as_nanos() / iterations as u128;
    println!("    Excluded bounds range creation: {:.2}ns per range", avg_excluded_ns);
    
    // Test 3: Included bounds
    let start_time = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000) as i32;
        let _iter = tree.range(key..=key); // Included end bound
    }
    let included_time = start_time.elapsed();
    
    let avg_included_ns = included_time.as_nanos() / iterations as u128;
    println!("    Included bounds range creation: {:.2}ns per range", avg_included_ns);
}

fn test_range_iteration_performance(tree: &BPlusTreeMap<i32, String>, tree_size: usize) {
    let start_key = (tree_size / 2) as i32;
    
    println!("  Range Iteration Performance:");
    
    // Test different range sizes
    let range_sizes = vec![1, 10, 100, 1000];
    
    for &range_size in &range_sizes {
        if range_size > tree_size / 4 {
            continue; // Skip if range is too large for the tree
        }
        
        let end_key = start_key + range_size as i32;
        
        // Measure total time (creation + iteration)
        let start_time = Instant::now();
        let count = tree.range(start_key..end_key).count();
        let total_time = start_time.elapsed();
        
        // Measure just creation time
        let creation_start = Instant::now();
        let _iter = tree.range(start_key..end_key);
        let creation_time = creation_start.elapsed();
        
        let iteration_time = total_time.saturating_sub(creation_time);
        
        println!("    Range size {}: creation={:.2}µs, iteration={:.2}µs, total={:.2}µs ({} elements)",
                range_size,
                creation_time.as_micros() as f64,
                iteration_time.as_micros() as f64,
                total_time.as_micros() as f64,
                count);
        
        if count > 0 {
            let per_element_ns = iteration_time.as_nanos() / count as u128;
            println!("      Per-element iteration cost: {:.2}ns", per_element_ns);
        }
    }
}
