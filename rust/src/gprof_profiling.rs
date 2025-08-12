use std::collections::BTreeMap;
use bplustree::GlobalCapacityBPlusTreeMap;

// Standalone profiling binary for gprof analysis
fn main() {
    println!("=== GPROF PROFILING: Large Range Queries ===");
    
    // Create large tree with 100,000 items
    let mut tree = GlobalCapacityBPlusTreeMap::new(16).expect("Failed to create tree");
    let mut std_map = BTreeMap::new();
    
    println!("Building trees with 100,000 items...");
    for i in 0..100_000 {
        tree.insert(i, format!("value_{}", i)).expect("Failed to insert");
        std_map.insert(i, format!("value_{}", i));
    }
    
    println!("Trees built. Starting intensive profiling...");
    
    // Intensive profiling workload for gprof
    for iteration in 0..1000 {
        if iteration % 100 == 0 {
            println!("Iteration {}/1000", iteration);
        }
        
        // Large range queries - this is what we want to profile
        let start_key = (iteration * 73) % 50_000; // Vary starting position
        let range_size = 5_000 + (iteration % 10_000); // Vary range size
        let end_key = start_key + range_size;
        
        // Profile our tree's range query
        let _: Vec<_> = tree.range(start_key..end_key).collect();
        
        // Also profile std::BTreeMap for comparison
        let _: Vec<_> = std_map.range(start_key..end_key).collect();
    }
    
    println!("=== PROFILING COMPLETE ===");
}
