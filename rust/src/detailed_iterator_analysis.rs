use crate::BPlusTreeMap;
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
    
    println!("\n🔍 ANALYSIS: What work happens in each next() call");
    analyze_next_call_work(&bplus, size);
}

fn analyze_arena_access_pattern(bplus: &BPlusTreeMap<usize, usize>, size: usize) {
    let start = size / 2;
    let _end = start + 1000;
    let iterations = 100;
    
    // Test: Count how many times we call get_leaf vs items returned
    println!("  Examining ItemIterator.next() implementation:");
    println!("  - current_leaf_id.and_then(|leaf_id| self.tree.get_leaf(leaf_id))");
    println!("  - This means: ONE arena access per next() call");
    println!("  - No caching of leaf reference between calls");
    
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
    
    println!("  Theoretical analysis:");
    println!("    Items per leaf (approx): {}", items_per_leaf);
    println!("    Leaves accessed for 1000 items: ~{}", leaves_accessed);
    println!("    Arena accesses per item: {:.3}", leaves_accessed as f64 / 1000.0);
    println!("    If arena access = 50ns, expected overhead: {:.1}ns per item", 
             (leaves_accessed as f64 / 1000.0) * 50.0);
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
    println!("    2. current_leaf_id.and_then(|leaf_id| self.tree.get_leaf(leaf_id))");
    println!("       - This is an arena lookup: Vec<Option<T>>[id].as_ref()");
    println!("       - Happens on EVERY call to next()");
    println!("    3. try_get_next_item(leaf) - bounds checking and indexing");
    println!("    4. If leaf exhausted: advance_to_next_leaf() - more arena access");
    println!("  ");
    println!("  FastItemIterator.next() does:");
    println!("    1. Check if finished (cheap)");
    println!("    2. unsafe {{ self.tree.get_leaf_unchecked(current_id) }}");
    println!("       - Still an arena lookup, but skips bounds checking");
    println!("    3. Direct array indexing into leaf.keys[index]");
    println!("    4. If leaf exhausted: move to leaf.next (still arena access)");
    println!("  ");
    println!("  Key insight: NO caching of leaf references between calls");
    println!("  Every next() = at least one arena lookup");
    
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detailed_iterator_analysis() {
        analyze_iterator_implementation();
    }
}
