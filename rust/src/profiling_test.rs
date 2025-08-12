//! Profiling test to identify range query bottlenecks

use crate::global_capacity_tree::GlobalCapacityBPlusTreeMap;
use std::time::Instant;

pub fn profile_range_operations() {
    println!("=== RANGE QUERY PROFILING ===");
    
    let mut tree = GlobalCapacityBPlusTreeMap::new(16).unwrap();
    let n = 100_000;
    
    // Insert data
    println!("Inserting {} items...", n);
    let start = Instant::now();
    for i in 0..n {
        tree.insert(i, i * 10).unwrap();
    }
    println!("Insert time: {:?}", start.elapsed());
    
    // Profile different aspects of range queries
    profile_range_creation(&tree, n);
    profile_range_iteration(&tree, n);
    profile_range_bounds_checking(&tree, n);
    profile_linked_list_traversal(&tree, n);
}

fn profile_range_creation(tree: &GlobalCapacityBPlusTreeMap<i32, i32>, n: i32) {
    println!("\n--- Range Creation Profiling ---");
    
    let iterations = 10_000;
    let start_key = n / 2;
    let end_key = start_key + 1000;
    
    // Profile range creation only (no iteration)
    let start = Instant::now();
    for _ in 0..iterations {
        let _iter = tree.range(start_key..end_key);
        // Don't iterate, just create
    }
    let creation_time = start.elapsed();
    
    println!("Range creation: {:?} total, {:.2}ns per creation", 
             creation_time, creation_time.as_nanos() as f64 / iterations as f64);
}

fn profile_range_iteration(tree: &GlobalCapacityBPlusTreeMap<i32, i32>, n: i32) {
    println!("\n--- Range Iteration Profiling ---");
    
    let test_cases = vec![
        (10, "tiny"),
        (100, "small"), 
        (1000, "medium"),
        (10000, "large"),
    ];
    
    for (range_size, name) in test_cases {
        let start_key = n / 2 - range_size / 2;
        let end_key = start_key + range_size;
        
        let start = Instant::now();
        let items: Vec<_> = tree.range(start_key..end_key).collect();
        let iteration_time = start.elapsed();
        
        println!("{} range ({}): {:?} total, {:.2}ns per item", 
                 name, items.len(), iteration_time, 
                 iteration_time.as_nanos() as f64 / items.len() as f64);
    }
}

fn profile_range_bounds_checking(tree: &GlobalCapacityBPlusTreeMap<i32, i32>, n: i32) {
    println!("\n--- Bounds Checking Profiling ---");
    
    let iterations = 1000;
    let start_key = n / 2;
    let end_key = start_key + 1000;
    
    // Profile with different bound types
    let start = Instant::now();
    for _ in 0..iterations {
        let _: Vec<_> = tree.range(start_key..end_key).collect();
    }
    let exclusive_time = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _: Vec<_> = tree.range(start_key..=end_key).collect();
    }
    let inclusive_time = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _: Vec<_> = tree.range(..).collect();
    }
    let unbounded_time = start.elapsed();
    
    println!("Exclusive bounds: {:?} ({:.2}µs per query)", 
             exclusive_time, exclusive_time.as_micros() as f64 / iterations as f64);
    println!("Inclusive bounds: {:?} ({:.2}µs per query)", 
             inclusive_time, inclusive_time.as_micros() as f64 / iterations as f64);
    println!("Unbounded: {:?} ({:.2}µs per query)", 
             unbounded_time, unbounded_time.as_micros() as f64 / iterations as f64);
}

fn profile_linked_list_traversal(tree: &GlobalCapacityBPlusTreeMap<i32, i32>, n: i32) {
    println!("\n--- Linked List Traversal Profiling ---");
    
    // Test traversal across multiple leaves
    let start_key = 0;
    let end_key = n; // Full traversal
    
    let start = Instant::now();
    let mut count = 0;
    for _ in tree.range(start_key..end_key) {
        count += 1;
        if count % 10000 == 0 {
            // Sample timing every 10k items
            let elapsed = start.elapsed();
            println!("Processed {} items in {:?} ({:.2}ns per item)", 
                     count, elapsed, elapsed.as_nanos() as f64 / count as f64);
        }
    }
    let total_time = start.elapsed();
    
    println!("Full traversal: {} items in {:?} ({:.2}ns per item)", 
             count, total_time, total_time.as_nanos() as f64 / count as f64);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn run_profiling() {
        profile_range_operations();
    }
}
