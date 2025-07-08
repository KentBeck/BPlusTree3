use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::time::Instant;

fn main() {
    println!("=== Focused BTreeMap vs BPlusTree Range Startup Comparison ===\n");
    
    // Test with different tree sizes to see scaling behavior
    let tree_sizes = [1_000, 10_000, 100_000];
    
    for &tree_size in &tree_sizes {
        println!("Testing with {} elements:", tree_size);
        
        // Build trees
        let mut btree = BTreeMap::new();
        let mut bplus = BPlusTreeMap::new(16).unwrap();
        
        for i in 0..tree_size {
            btree.insert(i as i32, format!("value_{}", i));
            bplus.insert(i as i32, format!("value_{}", i));
        }
        
        // Test range creation performance (many iterations for accuracy)
        test_range_creation_performance(&btree, &bplus, tree_size);
        
        // Test small range iteration (where startup cost matters most)
        test_small_range_performance(&btree, &bplus, tree_size);
        
        println!();
    }
}

fn test_range_creation_performance(btree: &BTreeMap<i32, String>, bplus: &BPlusTreeMap<i32, String>, tree_size: usize) {
    let iterations = 10_000;
    let start_key = (tree_size / 2) as i32;
    
    println!("  Range Creation Performance ({} iterations):", iterations);
    
    // Test 1: Single element ranges (pure startup cost)
    let btree_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000) as i32;
        let _iter = btree.range(key..key+1);
        // Don't consume iterator, just measure creation
    }
    let btree_time = btree_start.elapsed();
    
    let bplus_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000) as i32;
        let _iter = bplus.range(key..key+1);
        // Don't consume iterator, just measure creation
    }
    let bplus_time = bplus_start.elapsed();
    
    let btree_ns = btree_time.as_nanos() / iterations as u128;
    let bplus_ns = bplus_time.as_nanos() / iterations as u128;
    let ratio = bplus_ns as f64 / btree_ns as f64;
    
    println!("    Single element range creation:");
    println!("      BTreeMap:   {}ns per range", btree_ns);
    println!("      BPlusTree:  {}ns per range", bplus_ns);
    println!("      Ratio (B+/BTree): {:.1}x", ratio);
    
    // Test 2: Excluded bounds specifically (our optimization target)
    let btree_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000) as i32;
        let _iter = btree.range(key..key+1); // Excluded end bound
    }
    let btree_excluded_time = btree_start.elapsed();
    
    let bplus_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000) as i32;
        let _iter = bplus.range(key..key+1); // Excluded end bound
    }
    let bplus_excluded_time = bplus_start.elapsed();
    
    let btree_excluded_ns = btree_excluded_time.as_nanos() / iterations as u128;
    let bplus_excluded_ns = bplus_excluded_time.as_nanos() / iterations as u128;
    let excluded_ratio = bplus_excluded_ns as f64 / btree_excluded_ns as f64;
    
    println!("    Excluded bounds range creation:");
    println!("      BTreeMap:   {}ns per range", btree_excluded_ns);
    println!("      BPlusTree:  {}ns per range", bplus_excluded_ns);
    println!("      Ratio (B+/BTree): {:.1}x", excluded_ratio);
}

fn test_small_range_performance(btree: &BTreeMap<i32, String>, bplus: &BPlusTreeMap<i32, String>, tree_size: usize) {
    let start_key = (tree_size / 2) as i32;
    
    println!("  Small Range Performance (startup + iteration):");
    
    // Test different small range sizes
    let range_sizes = [1, 5, 10, 50];
    
    for &range_size in &range_sizes {
        let end_key = start_key + range_size;
        
        // BTreeMap
        let btree_start = Instant::now();
        let btree_count = btree.range(start_key..end_key).count();
        let btree_time = btree_start.elapsed();
        
        // BPlusTree
        let bplus_start = Instant::now();
        let bplus_count = bplus.range(start_key..end_key).count();
        let bplus_time = bplus_start.elapsed();
        
        assert_eq!(btree_count, bplus_count, "Count mismatch for range size {}", range_size);
        
        let ratio = bplus_time.as_micros() as f64 / btree_time.as_micros() as f64;
        
        println!("    Range size {}: BTree={}µs, B+={}µs, Ratio={:.1}x ({} elements)",
                range_size,
                btree_time.as_micros(),
                bplus_time.as_micros(),
                ratio,
                btree_count);
    }
}

fn test_range_creation_only(btree: &BTreeMap<i32, String>, bplus: &BPlusTreeMap<i32, String>, tree_size: usize) {
    let iterations = 1000;
    let start_key = (tree_size / 2) as i32;
    
    println!("  Pure Range Creation (no iteration):");
    
    // Measure just iterator creation time
    let btree_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 100) as i32;
        let _iter = btree.range(key..key+10);
        // Drop iterator immediately
    }
    let btree_time = btree_start.elapsed();
    
    let bplus_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 100) as i32;
        let _iter = bplus.range(key..key+10);
        // Drop iterator immediately
    }
    let bplus_time = bplus_start.elapsed();
    
    let btree_ns = btree_time.as_nanos() / iterations as u128;
    let bplus_ns = bplus_time.as_nanos() / iterations as u128;
    let ratio = bplus_ns as f64 / btree_ns as f64;
    
    println!("    Pure creation: BTree={}ns, B+={}ns, Ratio={:.1}x",
            btree_ns, bplus_ns, ratio);
}
