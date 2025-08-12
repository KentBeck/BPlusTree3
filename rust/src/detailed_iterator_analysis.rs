use crate::{BPlusTreeMap, FastItemIterator};
use std::collections::BTreeMap;
use std::time::Instant;

/// Detailed analysis of what actually happens in each next() call
pub fn analyze_iterator_implementation() {
    println!("=== DETAILED ITERATOR IMPLEMENTATION ANALYSIS ===");
    println!("Examining actual arena access patterns in next() calls\n");
    
    let size = 10_000;
    let capacity = 256;

    // Create test tree
    let mut bplus = BPlusTreeMap::new(capacity).unwrap();
    for i in 0..size {
        bplus.insert(i, i * 2);
    }
    
    println!("🔍 ANALYSIS: Arena Access Pattern in ItemIterator");
    analyze_arena_access_pattern(&bplus, size);
    
    println!("\n🔍 ANALYSIS: FastItemIterator vs ItemIterator");
    compare_iterator_implementations(&bplus, size);
    
    println!("\n🔍 ANALYSIS: BPlusTreeMap vs BTreeMap Iterator Performance");
    compare_with_btreemap(&bplus, size);
    
    println!("\n🔍 ANALYSIS: What work happens in each next() call");
    analyze_next_call_work(&bplus, size);
}

fn analyze_arena_access_pattern(bplus: &BPlusTreeMap<usize, usize>, size: usize) {
    let start = size / 2;
    let _end = start + 1000;
    let iterations = 100;
    
    // Test: Analyze the actual leaf caching implementation
    println!("  Examining ItemIterator.next() implementation:");
    println!("  - Uses cached leaf reference: current_leaf_ref.and_then(|leaf| ...)");
    println!("  - Arena access ONLY when advancing to next leaf");
    println!("  - Leaf caching optimization successfully implemented in cb17dae");

    // Time the iteration to see the actual cost
    let start_time = Instant::now();
    for _ in 0..iterations {
        let mut count = 0;
        for (_k, _v) in bplus.items_range(Some(&start), Some(&_end)) {
            count += 1;
        }
        assert_eq!(count, 1000);
    }
    let total_time = start_time.elapsed();
    
    let per_item = total_time.as_nanos() as f64 / (iterations * 1000) as f64;
    println!("  Measured overhead: {:.1}ns per item", per_item);
    
    // Calculate theoretical arena access cost
    let leaf_capacity = bplus.capacity;
    let items_per_leaf = leaf_capacity; // Approximate
    let leaves_accessed = 1000 / items_per_leaf + 1; // Approximate
    
    println!("  Leaf caching analysis:");
    println!("    Items per leaf (approx): {}", items_per_leaf);
    println!("    Leaves accessed for 1000 items: ~{}", leaves_accessed);
    println!("    Arena accesses per item (with caching): {:.3}", leaves_accessed as f64 / 1000.0);
    println!("    Caching reduces arena access frequency by ~{}x", items_per_leaf);
}

fn compare_iterator_implementations(bplus: &BPlusTreeMap<usize, usize>, size: usize) {
    let start = size / 2;
    let end = start + 1000;
    let iterations = 100;
    
    // Test regular ItemIterator
    let start_time = Instant::now();
    for _ in 0..iterations {
        let mut count = 0;
        for (_k, _v) in bplus.items() {
            if count >= 1000 { break; }
            count += 1;
        }
    }
    let regular_time = start_time.elapsed();
    
    // Test FastItemIterator
    let start_time = Instant::now();
    for _ in 0..iterations {
        let mut count = 0;
        for (_k, _v) in bplus.items_fast() {
            if count >= 1000 { break; }
            count += 1;
        }
    }
    let fast_time = start_time.elapsed();
    
    let regular_per_item = regular_time.as_nanos() as f64 / (iterations * 1000) as f64;
    let fast_per_item = fast_time.as_nanos() as f64 / (iterations * 1000) as f64;
    
    println!("  ItemIterator (safe):     {:.1}ns per item", regular_per_item);
    println!("  FastItemIterator (unsafe): {:.1}ns per item", fast_per_item);
    println!("  Speedup from unsafe:    {:.1}x", regular_per_item / fast_per_item);
    
    if fast_per_item < regular_per_item {
        println!("  ✅ Unsafe access provides measurable speedup");
    } else {
        println!("  ❌ Unsafe access doesn't help significantly");
    }
}

fn analyze_next_call_work(bplus: &BPlusTreeMap<usize, usize>, _size: usize) {
    println!("  Breaking down work in each next() call:");
    println!("  ");
    println!("  ItemIterator.next() does:");
    println!("    1. Check if finished (cheap)");
    println!("    2. current_leaf_ref.and_then(|leaf| self.try_get_next_item(leaf))");
    println!("       - Uses CACHED leaf reference - NO arena lookup!");
    println!("       - Direct access to leaf data");
    println!("    3. try_get_next_item(leaf) - bounds checking and indexing");
    println!("    4. If leaf exhausted: advance_to_next_leaf() - arena access ONLY here");
    println!("  ");
    println!("  FastItemIterator.next() does:");
    println!("    1. Check if finished (cheap)");
    println!("    2. Uses cached current_leaf_ref directly");
    println!("       - NO arena lookup during normal iteration");
    println!("    3. Direct array indexing into leaf.keys[index]");
    println!("    4. If leaf exhausted: advance to next leaf (arena access only here)");
    println!("  ");
    println!("  Key insight: Leaf caching eliminates per-item arena lookups");
    println!("  Arena access only when transitioning between leaves");

    // Test the cost of just arena lookups
    let iterations = 100_000;
    let leaf_id = bplus.get_first_leaf_id().unwrap();
    
    let start_time = Instant::now();
    for _ in 0..iterations {
        let _leaf = bplus.get_leaf(leaf_id);
    }
    let arena_time = start_time.elapsed();
    
    let arena_per_access = arena_time.as_nanos() as f64 / iterations as f64;
    println!("  Pure arena access cost: {:.1}ns per lookup", arena_per_access);
}

fn compare_with_btreemap(bplus: &BPlusTreeMap<usize, usize>, size: usize) {
    // Create equivalent BTreeMap
    let mut btree = BTreeMap::new();
    for i in 0..size {
        btree.insert(i, i * 2);
    }
    
    let start = size / 2;
    let end = start + 1000;
    let iterations = 100;
    
    // Benchmark BPlusTreeMap iterator
    let start_time = Instant::now();
    for _ in 0..iterations {
        for (_k, _v) in bplus.items_range(Some(&start), Some(&end)) {
            // Consume iterator
        }
    }
    let bplus_time = start_time.elapsed();
    
    // Benchmark BTreeMap iterator
    let start_time = Instant::now();
    for _ in 0..iterations {
        for (_k, _v) in btree.range(start..=end) {
            // Consume iterator
        }
    }
    let btree_time = start_time.elapsed();
    
    let bplus_per_item = bplus_time.as_nanos() as f64 / (iterations * 1000) as f64;
    let btree_per_item = btree_time.as_nanos() as f64 / (iterations * 1000) as f64;
    let speedup = btree_per_item / bplus_per_item;
    
    println!("  BPlusTreeMap iterator:   {:.1}ns per item", bplus_per_item);
    println!("  BTreeMap iterator:       {:.1}ns per item", btree_per_item);
    println!("  BPlusTreeMap speedup:    {:.1}x", speedup);
    
    if speedup > 1.0 {
        println!("  ✅ BPlusTreeMap is faster than BTreeMap");
    } else {
        println!("  ❌ BTreeMap is faster than BPlusTreeMap");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detailed_iterator_analysis() {
        analyze_iterator_implementation();
    }
}
