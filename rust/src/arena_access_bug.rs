//! Critical performance bug: Arena access on every iterator call

use crate::global_capacity_tree::GlobalCapacityBPlusTreeMap;

pub fn demonstrate_arena_access_bug() {
    println!("=== CRITICAL ARENA ACCESS BUG ===");
    
    let mut tree = GlobalCapacityBPlusTreeMap::new(16).unwrap();
    
    // Insert data
    for i in 0..100 {
        tree.insert(i, i * 10).unwrap();
    }
    
    println!("Tree with 100 items, capacity 16");
    println!("Expected ~7 leaves");
    
    // Test a range that spans multiple leaves
    let range = 20..80; // 60 items across ~4 leaves
    println!("Range: {}..{} (60 items across ~4 leaves)", range.start, range.end);
    
    // Current implementation calls arena.get() on EVERY iterator.next() call
    // This means for 60 items, we make 60 arena accesses!
    
    println!("\nCURRENT IMPLEMENTATION PROBLEM:");
    println!("- Iterator calls tree.leaf_arena.get(leaf_id) on EVERY next() call");
    println!("- For 60 items: 60 arena accesses");  
    println!("- Arena accesses per item: 1.0 (TERRIBLE!)");
    
    println!("\nOPTIMAL IMPLEMENTATION:");
    println!("- Cache leaf reference when moving to new leaf");
    println!("- Only access arena when switching leaves");
    println!("- For 60 items across 4 leaves: 4 arena accesses");
    println!("- Arena accesses per item: 0.067 (EXCELLENT!)");
    
    println!("\nPERFORMANCE IMPACT:");
    println!("- Current: 1.0 arena access per item");
    println!("- Optimal: ~0.067 arena access per item");
    println!("- Improvement: 15x reduction in arena accesses!");
    
    // Demonstrate the issue
    let items: Vec<_> = tree.range(range).collect();
    println!("\nActual items collected: {}", items.len());
    println!("Estimated arena accesses with current bug: {}", items.len());
    println!("Estimated arena accesses with fix: ~{}", (items.len() + 15) / 16);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arena_access_bug() {
        demonstrate_arena_access_bug();
    }
}
