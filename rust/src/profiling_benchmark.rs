use std::collections::BTreeMap;
use crate::GlobalCapacityBPlusTreeMap;

#[allow(dead_code)]
pub fn run_large_range_profiling() {
    println!("=== LARGE RANGE PROFILING ===");
    
    // Create large tree with 100,000 items
    let mut tree = GlobalCapacityBPlusTreeMap::new(16).expect("Failed to create tree");
    let mut std_map = BTreeMap::new();
    
    println!("Building trees with 100,000 items...");
    for i in 0..100_000 {
        tree.insert(i, format!("value_{}", i)).expect("Failed to insert");
        std_map.insert(i, format!("value_{}", i));
    }
    
    println!("Trees built. Starting profiling...");
    
    // Profile large range queries
    profile_large_ranges(&tree, &std_map);
    
    println!("=== PROFILING COMPLETE ===");
}

fn profile_large_ranges(tree: &GlobalCapacityBPlusTreeMap<i32, String>, std_map: &BTreeMap<i32, String>) {
    use std::time::Instant;
    
    // Test different range sizes
    let test_cases = [
        (1_000, "1K items"),
        (5_000, "5K items"), 
        (10_000, "10K items"),
        (25_000, "25K items"),
        (50_000, "50K items"),
    ];
    
    for (range_size, description) in test_cases {
        println!("\n--- {} Range Query ---", description);
        
        let start_key = 25_000;
        let end_key = start_key + range_size;
        
        // Profile our tree
        let start = Instant::now();
        let our_count: usize = tree.range(start_key..end_key).count();
        let our_time = start.elapsed();
        
        // Profile std::BTreeMap
        let start = Instant::now();
        let std_count: usize = std_map.range(start_key..end_key).count();
        let std_time = start.elapsed();
        
        println!("Our tree:     {:?} ({} items)", our_time, our_count);
        println!("std::BTreeMap: {:?} ({} items)", std_time, std_count);
        
        let speedup = std_time.as_nanos() as f64 / our_time.as_nanos() as f64;
        if speedup > 1.0 {
            println!("Our tree is {:.1}x FASTER", speedup);
        } else {
            println!("std::BTreeMap is {:.1}x faster", 1.0 / speedup);
        }
        
        // Intensive profiling loop for detailed analysis
        println!("Running intensive profiling loop...");
        let start = Instant::now();
        for _ in 0..100 {
            let _: Vec<_> = tree.range(start_key..end_key).collect();
        }
        let intensive_time = start.elapsed();
        println!("100 iterations: {:?} (avg: {:?})", intensive_time, intensive_time / 100);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_large_range_profiling() {
        run_large_range_profiling();
    }
}
