use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::time::Instant;

fn main() {
    println!("=== Detailed BTreeMap vs BPlusTree Performance Analysis ===\n");
    
    let tree_size = 100_000;
    println!("Building trees with {} elements...", tree_size);
    
    // Build trees
    let mut btree = BTreeMap::new();
    let mut bplus = BPlusTreeMap::new(16).unwrap();
    
    for i in 0..tree_size {
        btree.insert(i as i32, format!("value_{}", i));
        bplus.insert(i as i32, format!("value_{}", i));
    }
    
    println!("Trees built successfully.\n");
    
    // Test 1: Range creation overhead
    test_pure_range_creation(&btree, &bplus, tree_size);
    
    // Test 2: Range iteration performance
    test_range_iteration(&btree, &bplus, tree_size);
    
    // Test 3: Combined performance
    test_combined_performance(&btree, &bplus, tree_size);
    
    // Test 4: Different range positions
    test_range_positions(&btree, &bplus, tree_size);
}

fn test_pure_range_creation(btree: &BTreeMap<i32, String>, bplus: &BPlusTreeMap<i32, String>, tree_size: usize) {
    println!("=== Pure Range Creation Performance ===");
    
    let iterations = 50_000;
    let start_key = (tree_size / 2) as i32;
    
    // Test range creation without iteration
    let btree_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000) as i32;
        let _iter = btree.range(key..key+10);
        // Iterator dropped immediately
    }
    let btree_time = btree_start.elapsed();
    
    let bplus_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000) as i32;
        let _iter = bplus.range(key..key+10);
        // Iterator dropped immediately
    }
    let bplus_time = bplus_start.elapsed();
    
    let btree_ns = btree_time.as_nanos() / iterations as u128;
    let bplus_ns = bplus_time.as_nanos() / iterations as u128;
    let ratio = bplus_ns as f64 / btree_ns as f64;
    
    println!("Range creation ({} iterations):", iterations);
    println!("  BTreeMap:   {:.1}ns per range", btree_ns);
    println!("  BPlusTree:  {:.1}ns per range", bplus_ns);
    println!("  Ratio (B+/BTree): {:.2}x", ratio);
    
    if ratio < 1.0 {
        println!("  ✅ BPlusTree is {:.1}% FASTER at range creation", (1.0 - ratio) * 100.0);
    } else {
        println!("  ❌ BPlusTree is {:.1}% slower at range creation", (ratio - 1.0) * 100.0);
    }
    println!();
}

fn test_range_iteration(btree: &BTreeMap<i32, String>, bplus: &BPlusTreeMap<i32, String>, tree_size: usize) {
    println!("=== Range Iteration Performance ===");
    
    let start_key = (tree_size / 2) as i32;
    let range_sizes = [1, 10, 100, 1000];
    
    for &range_size in &range_sizes {
        let end_key = start_key + range_size;
        
        // Measure creation time separately
        let btree_create_start = Instant::now();
        let btree_iter = btree.range(start_key..end_key);
        let btree_create_time = btree_create_start.elapsed();
        
        let bplus_create_start = Instant::now();
        let bplus_iter = bplus.range(start_key..end_key);
        let bplus_create_time = bplus_create_start.elapsed();
        
        // Measure total time (creation + iteration)
        let btree_total_start = Instant::now();
        let btree_count = btree.range(start_key..end_key).count();
        let btree_total_time = btree_total_start.elapsed();
        
        let bplus_total_start = Instant::now();
        let bplus_count = bplus.range(start_key..end_key).count();
        let bplus_total_time = bplus_total_start.elapsed();
        
        assert_eq!(btree_count, bplus_count);
        
        let btree_iter_time = btree_total_time.saturating_sub(btree_create_time);
        let bplus_iter_time = bplus_total_time.saturating_sub(bplus_create_time);
        
        println!("Range size {} ({} elements):", range_size, btree_count);
        println!("  Creation time:");
        println!("    BTreeMap:   {:.1}µs", btree_create_time.as_micros() as f64);
        println!("    BPlusTree:  {:.1}µs", bplus_create_time.as_micros() as f64);
        
        println!("  Iteration time:");
        println!("    BTreeMap:   {:.1}µs", btree_iter_time.as_micros() as f64);
        println!("    BPlusTree:  {:.1}µs", bplus_iter_time.as_micros() as f64);
        
        println!("  Total time:");
        println!("    BTreeMap:   {:.1}µs", btree_total_time.as_micros() as f64);
        println!("    BPlusTree:  {:.1}µs", bplus_total_time.as_micros() as f64);
        
        let total_ratio = bplus_total_time.as_micros() as f64 / btree_total_time.as_micros() as f64;
        println!("  Ratio (B+/BTree): {:.2}x", total_ratio);
        
        if btree_count > 0 {
            let btree_per_element = btree_iter_time.as_nanos() / btree_count as u128;
            let bplus_per_element = bplus_iter_time.as_nanos() / bplus_count as u128;
            println!("  Per-element iteration cost:");
            println!("    BTreeMap:   {:.1}ns", btree_per_element);
            println!("    BPlusTree:  {:.1}ns", bplus_per_element);
        }
        println!();
    }
}

fn test_combined_performance(btree: &BTreeMap<i32, String>, bplus: &BPlusTreeMap<i32, String>, tree_size: usize) {
    println!("=== Combined Performance (Multiple Small Ranges) ===");
    
    let iterations = 1000;
    let start_key = (tree_size / 4) as i32;
    
    // Test many small ranges (where startup cost matters)
    let btree_start = Instant::now();
    let mut btree_total_elements = 0;
    for i in 0..iterations {
        let key = start_key + (i * 10) as i32;
        btree_total_elements += btree.range(key..key+5).count();
    }
    let btree_time = btree_start.elapsed();
    
    let bplus_start = Instant::now();
    let mut bplus_total_elements = 0;
    for i in 0..iterations {
        let key = start_key + (i * 10) as i32;
        bplus_total_elements += bplus.range(key..key+5).count();
    }
    let bplus_time = bplus_start.elapsed();
    
    assert_eq!(btree_total_elements, bplus_total_elements);
    
    let ratio = bplus_time.as_micros() as f64 / btree_time.as_micros() as f64;
    
    println!("Multiple small ranges ({} ranges of 5 elements each):", iterations);
    println!("  BTreeMap:   {:.1}µs total ({:.1}µs per range)", 
            btree_time.as_micros() as f64, 
            btree_time.as_micros() as f64 / iterations as f64);
    println!("  BPlusTree:  {:.1}µs total ({:.1}µs per range)", 
            bplus_time.as_micros() as f64,
            bplus_time.as_micros() as f64 / iterations as f64);
    println!("  Ratio (B+/BTree): {:.2}x", ratio);
    println!("  Total elements processed: {}", btree_total_elements);
    println!();
}

fn test_range_positions(btree: &BTreeMap<i32, String>, bplus: &BPlusTreeMap<i32, String>, tree_size: usize) {
    println!("=== Range Performance at Different Positions ===");
    
    let range_size = 100;
    let positions = [
        ("Start", 0),
        ("25%", tree_size / 4),
        ("50%", tree_size / 2),
        ("75%", tree_size * 3 / 4),
        ("End", tree_size - range_size - 1),
    ];
    
    for (label, start_pos) in &positions {
        let start_key = *start_pos as i32;
        let end_key = start_key + range_size as i32;
        
        let btree_start = Instant::now();
        let btree_count = btree.range(start_key..end_key).count();
        let btree_time = btree_start.elapsed();
        
        let bplus_start = Instant::now();
        let bplus_count = bplus.range(start_key..end_key).count();
        let bplus_time = bplus_start.elapsed();
        
        assert_eq!(btree_count, bplus_count);
        
        let ratio = bplus_time.as_micros() as f64 / btree_time.as_micros() as f64;
        
        println!("{} position: BTree={:.1}µs, B+={:.1}µs, Ratio={:.2}x ({} elements)",
                label,
                btree_time.as_micros() as f64,
                bplus_time.as_micros() as f64,
                ratio,
                btree_count);
    }
}
