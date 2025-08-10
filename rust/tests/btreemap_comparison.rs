use bplustree::{BPlusTreeMap, GlobalCapacityBPlusTreeMap};
use std::collections::BTreeMap;
use std::time::Instant;

#[test]
fn test_insertion_vs_btreemap() {
    const TEST_SIZE: usize = 10000;
    const TREE_CAPACITY: usize = 64;
    
    // Generate test data
    let data: Vec<(i32, String)> = (0..TEST_SIZE)
        .map(|i| (i as i32, format!("value_{}", i)))
        .collect();
    
    // Test std::collections::BTreeMap
    let start = Instant::now();
    let mut btree_map = BTreeMap::new();
    for (key, value) in &data {
        btree_map.insert(*key, value.clone());
    }
    let btree_duration = start.elapsed();
    
    // Test our BPlusTreeMap
    let start = Instant::now();
    let mut bplus_tree = BPlusTreeMap::new(TREE_CAPACITY).unwrap();
    for (key, value) in &data {
        bplus_tree.insert(*key, value.clone());
    }
    let bplus_duration = start.elapsed();
    
    // Test our GlobalCapacityBPlusTreeMap
    let start = Instant::now();
    let mut gc_bplus_tree = GlobalCapacityBPlusTreeMap::new(TREE_CAPACITY).unwrap();
    for (key, value) in &data {
        gc_bplus_tree.insert(*key, value.clone()).unwrap();
    }
    let gc_bplus_duration = start.elapsed();
    
    println!("=== INSERTION PERFORMANCE vs BTreeMap ===");
    println!("std::collections::BTreeMap: {:?}", btree_duration);
    println!("BPlusTreeMap: {:?}", bplus_duration);
    println!("GlobalCapacityBPlusTreeMap: {:?}", gc_bplus_duration);
    println!("BTreeMap vs BPlusTreeMap ratio: {:.2}", 
             btree_duration.as_nanos() as f64 / bplus_duration.as_nanos() as f64);
    println!("BTreeMap vs GlobalCapacityBPlusTreeMap ratio: {:.2}", 
             btree_duration.as_nanos() as f64 / gc_bplus_duration.as_nanos() as f64);
    
    // Verify all trees work correctly
    assert_eq!(btree_map.len(), TEST_SIZE);
    assert_eq!(bplus_tree.len(), TEST_SIZE);
    assert_eq!(gc_bplus_tree.len(), TEST_SIZE);
}

#[test]
fn test_lookup_vs_btreemap() {
    const TEST_SIZE: usize = 10000;
    const LOOKUP_COUNT: usize = 1000;
    const TREE_CAPACITY: usize = 64;
    
    // Generate test data
    let data: Vec<(i32, String)> = (0..TEST_SIZE)
        .map(|i| (i as i32, format!("value_{}", i)))
        .collect();
    
    // Prepare trees
    let mut btree_map = BTreeMap::new();
    let mut bplus_tree = BPlusTreeMap::new(TREE_CAPACITY).unwrap();
    let mut gc_bplus_tree = GlobalCapacityBPlusTreeMap::new(TREE_CAPACITY).unwrap();
    
    for (key, value) in &data {
        btree_map.insert(*key, value.clone());
        bplus_tree.insert(*key, value.clone());
        gc_bplus_tree.insert(*key, value.clone()).unwrap();
    }
    
    // Generate lookup keys
    let lookup_keys: Vec<i32> = (0..LOOKUP_COUNT).map(|i| (i * 10) as i32).collect();
    
    // Test BTreeMap lookups
    let start = Instant::now();
    for _ in 0..100 {
        for key in &lookup_keys {
            std::hint::black_box(btree_map.get(key));
        }
    }
    let btree_duration = start.elapsed();
    
    // Test BPlusTreeMap lookups
    let start = Instant::now();
    for _ in 0..100 {
        for key in &lookup_keys {
            std::hint::black_box(bplus_tree.get(key));
        }
    }
    let bplus_duration = start.elapsed();
    
    // Test GlobalCapacityBPlusTreeMap lookups
    let start = Instant::now();
    for _ in 0..100 {
        for key in &lookup_keys {
            std::hint::black_box(gc_bplus_tree.get(key));
        }
    }
    let gc_bplus_duration = start.elapsed();
    
    println!("=== LOOKUP PERFORMANCE vs BTreeMap ===");
    println!("std::collections::BTreeMap: {:?}", btree_duration);
    println!("BPlusTreeMap: {:?}", bplus_duration);
    println!("GlobalCapacityBPlusTreeMap: {:?}", gc_bplus_duration);
    println!("BTreeMap vs BPlusTreeMap ratio: {:.2}", 
             btree_duration.as_nanos() as f64 / bplus_duration.as_nanos() as f64);
    println!("BTreeMap vs GlobalCapacityBPlusTreeMap ratio: {:.2}", 
             btree_duration.as_nanos() as f64 / gc_bplus_duration.as_nanos() as f64);
}

#[test]
fn test_memory_usage_vs_btreemap() {
    use std::mem;
    
    println!("=== MEMORY USAGE COMPARISON ===");
    
    // Size of key-value pair
    let kv_size = mem::size_of::<(i32, String)>();
    println!("Key-Value pair size: {} bytes", kv_size);
    
    // BTreeMap node overhead (estimated)
    println!("std::collections::BTreeMap:");
    println!("  - Uses B-tree with internal nodes");
    println!("  - Optimized for general-purpose use");
    println!("  - Memory overhead varies with tree structure");
    
    // Our implementations
    println!("BPlusTreeMap:");
    println!("  - Leaf nodes store all data");
    println!("  - Branch nodes only store keys + pointers");
    println!("  - Per-node capacity field: {} bytes", mem::size_of::<usize>());
    
    println!("GlobalCapacityBPlusTreeMap:");
    println!("  - Same as BPlusTreeMap but saves {} bytes per node", mem::size_of::<usize>());
    println!("  - Better memory efficiency for large trees");
    
    // Theoretical analysis
    println!("Theoretical advantages of B+ trees:");
    println!("  - Better cache locality for range queries");
    println!("  - All data in leaf nodes (better for sequential access)");
    println!("  - Predictable memory layout");
    
    println!("Theoretical advantages of BTreeMap:");
    println!("  - Highly optimized implementation");
    println!("  - Better worst-case guarantees");
    println!("  - More compact for small trees");
}

#[test]
fn test_sequential_access_vs_btreemap() {
    const TEST_SIZE: usize = 5000;
    const TREE_CAPACITY: usize = 32;
    
    // Sequential data
    let sequential_data: Vec<(i32, String)> = (0..TEST_SIZE)
        .map(|i| (i as i32, format!("value_{}", i)))
        .collect();
    
    // Test BTreeMap sequential insertion
    let start = Instant::now();
    let mut btree_map = BTreeMap::new();
    for (key, value) in &sequential_data {
        btree_map.insert(*key, value.clone());
    }
    let btree_duration = start.elapsed();
    
    // Test BPlusTreeMap sequential insertion
    let start = Instant::now();
    let mut bplus_tree = BPlusTreeMap::new(TREE_CAPACITY).unwrap();
    for (key, value) in &sequential_data {
        bplus_tree.insert(*key, value.clone());
    }
    let bplus_duration = start.elapsed();
    
    // Test GlobalCapacityBPlusTreeMap sequential insertion
    let start = Instant::now();
    let mut gc_bplus_tree = GlobalCapacityBPlusTreeMap::new(TREE_CAPACITY).unwrap();
    for (key, value) in &sequential_data {
        gc_bplus_tree.insert(*key, value.clone()).unwrap();
    }
    let gc_bplus_duration = start.elapsed();
    
    println!("=== SEQUENTIAL ACCESS vs BTreeMap ===");
    println!("Sequential insertion:");
    println!("std::collections::BTreeMap: {:?}", btree_duration);
    println!("BPlusTreeMap: {:?}", bplus_duration);
    println!("GlobalCapacityBPlusTreeMap: {:?}", gc_bplus_duration);
    println!("BTreeMap vs BPlusTreeMap ratio: {:.2}", 
             btree_duration.as_nanos() as f64 / bplus_duration.as_nanos() as f64);
    println!("BTreeMap vs GlobalCapacityBPlusTreeMap ratio: {:.2}", 
             btree_duration.as_nanos() as f64 / gc_bplus_duration.as_nanos() as f64);
    
    // Test sequential access pattern
    let start = Instant::now();
    for i in 0..TEST_SIZE {
        std::hint::black_box(btree_map.get(&(i as i32)));
    }
    let btree_seq_access = start.elapsed();
    
    let start = Instant::now();
    for i in 0..TEST_SIZE {
        std::hint::black_box(bplus_tree.get(&(i as i32)));
    }
    let bplus_seq_access = start.elapsed();
    
    let start = Instant::now();
    for i in 0..TEST_SIZE {
        std::hint::black_box(gc_bplus_tree.get(&(i as i32)));
    }
    let gc_bplus_seq_access = start.elapsed();
    
    println!("Sequential access performance:");
    println!("std::collections::BTreeMap: {:?}", btree_seq_access);
    println!("BPlusTreeMap: {:?}", bplus_seq_access);
    println!("GlobalCapacityBPlusTreeMap: {:?}", gc_bplus_seq_access);
    println!("BTreeMap vs BPlusTreeMap ratio: {:.2}", 
             btree_seq_access.as_nanos() as f64 / bplus_seq_access.as_nanos() as f64);
    println!("BTreeMap vs GlobalCapacityBPlusTreeMap ratio: {:.2}", 
             btree_seq_access.as_nanos() as f64 / gc_bplus_seq_access.as_nanos() as f64);
}
