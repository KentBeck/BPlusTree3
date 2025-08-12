//! Analysis of arena access patterns in range queries

use crate::global_capacity_tree::GlobalCapacityBPlusTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Mock arena that counts accesses
pub struct CountingArena<T> {
    inner: crate::compact_arena::CompactArena<T>,
    access_count: Arc<AtomicUsize>,
}

impl<T> CountingArena<T> {
    pub fn new() -> Self {
        Self {
            inner: crate::compact_arena::CompactArena::new(),
            access_count: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    pub fn get_access_count(&self) -> usize {
        self.access_count.load(Ordering::Relaxed)
    }
    
    pub fn reset_count(&self) {
        self.access_count.store(0, Ordering::Relaxed);
    }
}

// For now, let's analyze the current implementation by tracing through the code
pub fn analyze_arena_accesses() {
    println!("=== ARENA ACCESS ANALYSIS ===");
    
    let mut tree = GlobalCapacityBPlusTreeMap::new(16).unwrap();
    
    // Insert test data - let's use a smaller set for detailed analysis
    let n = 1000;
    for i in 0..n {
        tree.insert(i, i * 10).unwrap();
    }
    
    println!("Tree with {} items, capacity 16", n);
    println!("Expected leaves: ~{}", (n + 15) / 16); // Rough estimate
    
    // Analyze different range sizes
    analyze_range_accesses(&tree, 10, "small");
    analyze_range_accesses(&tree, 100, "medium");  
    analyze_range_accesses(&tree, 500, "large");
}

fn analyze_range_accesses(tree: &GlobalCapacityBPlusTreeMap<i32, i32>, range_size: i32, name: &str) {
    println!("\n--- {} Range Analysis (size: {}) ---", name, range_size);
    
    let start_key = 400; // Start from middle
    let end_key = start_key + range_size;
    
    // Manual analysis of what happens during range iteration
    println!("Range: {}..{}", start_key, end_key);
    
    // Step 1: Range creation - find starting position
    println!("1. Range creation (find start position):");
    println!("   - Tree traversal from root to leaf: O(log n) arena accesses");
    println!("   - For tree height ~{}, expect ~{} accesses", estimate_tree_height(1000, 16), estimate_tree_height(1000, 16));
    
    // Step 2: Iteration analysis
    println!("2. Range iteration:");
    
    // Calculate how many leaves we'll traverse
    let items_per_leaf = 16; // capacity
    let expected_leaves = (range_size + items_per_leaf - 1) / items_per_leaf;
    
    println!("   - Expected items: {}", range_size);
    println!("   - Expected leaves to traverse: {}", expected_leaves);
    
    // Arena access pattern during iteration:
    // - 1 access per leaf to get the leaf node
    // - Items within a leaf don't require additional arena access
    println!("   - Arena accesses during iteration: {} (1 per leaf)", expected_leaves);
    
    let total_arena_accesses = estimate_tree_height(1000, 16) + expected_leaves as usize;
    println!("   - Total arena accesses: {} (creation) + {} (iteration) = {}", 
             estimate_tree_height(1000, 16), expected_leaves, total_arena_accesses);
    println!("   - Arena accesses per item: {:.3}", total_arena_accesses as f64 / range_size as f64);
    
    // Actual test
    let items: Vec<_> = tree.range(start_key..end_key).collect();
    println!("   - Actual items collected: {}", items.len());
}

fn estimate_tree_height(n: usize, capacity: usize) -> usize {
    if n <= capacity {
        1 // Just root leaf
    } else {
        // Rough estimate: log_capacity(n)
        let mut height = 1;
        let mut nodes_at_level = 1;
        let mut total_capacity = capacity;
        
        while total_capacity < n {
            height += 1;
            nodes_at_level *= capacity;
            total_capacity += nodes_at_level * capacity;
        }
        
        height
    }
}

// Let's also create a more precise analysis by instrumenting the actual code
pub fn precise_arena_access_analysis() {
    println!("\n=== PRECISE ARENA ACCESS ANALYSIS ===");
    
    let mut tree = GlobalCapacityBPlusTreeMap::new(16).unwrap();
    
    // Insert data
    for i in 0..1000 {
        tree.insert(i, i * 10).unwrap();
    }
    
    // Test different scenarios
    test_arena_efficiency(&tree, 400..410, "10 items, single leaf");
    test_arena_efficiency(&tree, 400..420, "20 items, ~2 leaves"); 
    test_arena_efficiency(&tree, 400..500, "100 items, ~6 leaves");
}

fn test_arena_efficiency(tree: &GlobalCapacityBPlusTreeMap<i32, i32>, range: std::ops::Range<i32>, description: &str) {
    println!("\n--- {} ---", description);
    
    let items: Vec<_> = tree.range(range.clone()).collect();
    let _range_size = range.end - range.start;

    println!("Range: {}..{}", range.start, range.end);
    println!("Items collected: {}", items.len());
    
    // Theoretical analysis:
    // 1. Range creation: O(log n) arena accesses to find start
    // 2. Iteration: 1 arena access per leaf traversed
    
    let tree_height = estimate_tree_height(1000, 16);
    let leaves_traversed = estimate_leaves_in_range(&items, 16);
    let total_accesses = tree_height + leaves_traversed;
    
    println!("Estimated arena accesses:");
    println!("  - Range creation (tree traversal): {}", tree_height);
    println!("  - Iteration (leaf accesses): {}", leaves_traversed);
    println!("  - Total: {}", total_accesses);
    println!("  - Per item: {:.3}", total_accesses as f64 / items.len() as f64);
    
    // Efficiency analysis
    let accesses_per_item = total_accesses as f64 / items.len() as f64;
    if accesses_per_item < 0.2 {
        println!("  ✅ EXCELLENT: < 0.2 arena accesses per item");
    } else if accesses_per_item < 0.5 {
        println!("  ✅ GOOD: < 0.5 arena accesses per item");
    } else if accesses_per_item < 1.0 {
        println!("  ⚠️  FAIR: < 1.0 arena accesses per item");
    } else {
        println!("  ❌ POOR: >= 1.0 arena accesses per item");
    }
}

fn estimate_leaves_in_range<K, V>(items: &[(&K, &V)], capacity: usize) -> usize {
    // Rough estimate: assume items are distributed evenly across leaves
    (items.len() + capacity - 1) / capacity
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arena_access_analysis() {
        analyze_arena_accesses();
        precise_arena_access_analysis();
    }
}
