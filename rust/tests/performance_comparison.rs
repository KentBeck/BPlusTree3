use bplustree::{BPlusTreeMap, GlobalCapacityBPlusTreeMap};
use std::time::Instant;

#[test]
fn test_insertion_performance_comparison() {
    const TREE_CAPACITY: usize = 64;
    const TEST_SIZE: usize = 10000;
    
    // Generate test data
    let data: Vec<(i32, String)> = (0..TEST_SIZE)
        .map(|i| (i as i32, format!("value_{}", i)))
        .collect();
    
    // Test standard tree
    let start = Instant::now();
    let mut std_tree = BPlusTreeMap::new(TREE_CAPACITY).unwrap();
    for (key, value) in &data {
        std_tree.insert(*key, value.clone());
    }
    let std_duration = start.elapsed();
    
    // Test global capacity tree
    let start = Instant::now();
    let mut gc_tree = GlobalCapacityBPlusTreeMap::new(TREE_CAPACITY).unwrap();
    for (key, value) in &data {
        gc_tree.insert(*key, value.clone()).unwrap();
    }
    let gc_duration = start.elapsed();
    
    println!("=== INSERTION PERFORMANCE ===");
    println!("Standard tree insertion: {:?}", std_duration);
    println!("Global capacity tree insertion: {:?}", gc_duration);
    println!("Performance ratio (std/gc): {:.2}", 
             std_duration.as_nanos() as f64 / gc_duration.as_nanos() as f64);
    
    // Verify both trees work correctly
    assert_eq!(std_tree.len(), TEST_SIZE);
    assert_eq!(gc_tree.len(), TEST_SIZE);
}

#[test]
fn test_lookup_performance_comparison() {
    const TREE_CAPACITY: usize = 64;
    const TEST_SIZE: usize = 10000;
    const LOOKUP_COUNT: usize = 1000;
    
    // Generate test data
    let data: Vec<(i32, String)> = (0..TEST_SIZE)
        .map(|i| (i as i32, format!("value_{}", i)))
        .collect();
    
    // Prepare trees
    let mut std_tree = BPlusTreeMap::new(TREE_CAPACITY).unwrap();
    let mut gc_tree = GlobalCapacityBPlusTreeMap::new(TREE_CAPACITY).unwrap();
    
    for (key, value) in &data {
        std_tree.insert(*key, value.clone());
        gc_tree.insert(*key, value.clone()).unwrap();
    }
    
    // Generate lookup keys
    let lookup_keys: Vec<i32> = (0..LOOKUP_COUNT).map(|i| (i * 10) as i32).collect();
    
    // Test standard tree lookups
    let start = Instant::now();
    for _ in 0..100 {
        for key in &lookup_keys {
            std::hint::black_box(std_tree.get(key));
        }
    }
    let std_duration = start.elapsed();
    
    // Test global capacity tree lookups
    let start = Instant::now();
    for _ in 0..100 {
        for key in &lookup_keys {
            std::hint::black_box(gc_tree.get(key));
        }
    }
    let gc_duration = start.elapsed();
    
    println!("=== LOOKUP PERFORMANCE ===");
    println!("Standard tree lookup: {:?}", std_duration);
    println!("Global capacity tree lookup: {:?}", gc_duration);
    println!("Performance ratio (std/gc): {:.2}", 
             std_duration.as_nanos() as f64 / gc_duration.as_nanos() as f64);
}

#[test]
fn test_memory_usage_estimation() {
    use std::mem;
    
    // Calculate theoretical memory savings
    let capacity_field_size = mem::size_of::<usize>();
    let node_count_estimate = 1000; // Estimate for a moderately sized tree
    
    let memory_saved_per_node = capacity_field_size;
    let total_memory_saved = memory_saved_per_node * node_count_estimate;
    
    println!("=== MEMORY USAGE ANALYSIS ===");
    println!("Estimated memory savings:");
    println!("  Per node: {} bytes", memory_saved_per_node);
    println!("  For {} nodes: {} bytes ({:.2} KB)", 
             node_count_estimate, total_memory_saved, total_memory_saved as f64 / 1024.0);
    
    // Show node sizes
    println!("Node size comparison:");
    println!("  usize (capacity field): {} bytes", mem::size_of::<usize>());
    println!("  Memory saved per leaf node: {} bytes", capacity_field_size);
    println!("  Memory saved per branch node: {} bytes", capacity_field_size);
}

#[test]
fn test_sequential_insertion_performance() {
    const TREE_CAPACITY: usize = 32; // Smaller capacity to force more splits
    const TEST_SIZE: usize = 5000;
    
    // Sequential data (worst case for B+ trees - forces maximum splits)
    let sequential_data: Vec<(i32, String)> = (0..TEST_SIZE)
        .map(|i| (i as i32, format!("value_{}", i)))
        .collect();
    
    // Test standard tree
    let start = Instant::now();
    let mut std_tree = BPlusTreeMap::new(TREE_CAPACITY).unwrap();
    for (key, value) in &sequential_data {
        std_tree.insert(*key, value.clone());
    }
    let std_duration = start.elapsed();
    
    // Test global capacity tree
    let start = Instant::now();
    let mut gc_tree = GlobalCapacityBPlusTreeMap::new(TREE_CAPACITY).unwrap();
    for (key, value) in &sequential_data {
        gc_tree.insert(*key, value.clone()).unwrap();
    }
    let gc_duration = start.elapsed();
    
    println!("=== SEQUENTIAL INSERTION PERFORMANCE ===");
    println!("Standard tree sequential insertion: {:?}", std_duration);
    println!("Global capacity tree sequential insertion: {:?}", gc_duration);
    println!("Performance ratio (std/gc): {:.2}", 
             std_duration.as_nanos() as f64 / gc_duration.as_nanos() as f64);
    
    // Verify both trees work correctly
    assert_eq!(std_tree.len(), TEST_SIZE);
    assert_eq!(gc_tree.len(), TEST_SIZE);
}

#[test]
fn test_large_scale_performance() {
    const TREE_CAPACITY: usize = 128;
    const TEST_SIZE: usize = 50000;
    
    // Generate test data
    let data: Vec<(i32, String)> = (0..TEST_SIZE)
        .map(|i| (i as i32, format!("value_{}", i)))
        .collect();
    
    // Test standard tree
    let start = Instant::now();
    let mut std_tree = BPlusTreeMap::new(TREE_CAPACITY).unwrap();
    for (key, value) in &data {
        std_tree.insert(*key, value.clone());
    }
    let std_duration = start.elapsed();
    
    // Test global capacity tree
    let start = Instant::now();
    let mut gc_tree = GlobalCapacityBPlusTreeMap::new(TREE_CAPACITY).unwrap();
    for (key, value) in &data {
        gc_tree.insert(*key, value.clone()).unwrap();
    }
    let gc_duration = start.elapsed();
    
    println!("=== LARGE SCALE PERFORMANCE ===");
    println!("Standard tree large scale insertion: {:?}", std_duration);
    println!("Global capacity tree large scale insertion: {:?}", gc_duration);
    println!("Performance ratio (std/gc): {:.2}", 
             std_duration.as_nanos() as f64 / gc_duration.as_nanos() as f64);
    
    // Test lookup performance on large trees
    let lookup_keys: Vec<i32> = (0..1000).map(|i| i * 50).collect();
    
    let start = Instant::now();
    for key in &lookup_keys {
        std::hint::black_box(std_tree.get(key));
    }
    let std_lookup_duration = start.elapsed();
    
    let start = Instant::now();
    for key in &lookup_keys {
        std::hint::black_box(gc_tree.get(key));
    }
    let gc_lookup_duration = start.elapsed();
    
    println!("Standard tree large scale lookup: {:?}", std_lookup_duration);
    println!("Global capacity tree large scale lookup: {:?}", gc_lookup_duration);
    println!("Lookup performance ratio (std/gc): {:.2}", 
             std_lookup_duration.as_nanos() as f64 / gc_lookup_duration.as_nanos() as f64);
    
    // Verify both trees work correctly
    assert_eq!(std_tree.len(), TEST_SIZE);
    assert_eq!(gc_tree.len(), TEST_SIZE);
}
