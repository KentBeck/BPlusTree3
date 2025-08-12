use std::collections::BTreeMap;
use crate::GlobalCapacityBPlusTreeMap;
use std::time::Instant;

/// Final auditor experiment that tests the exact same operations as Criterion
pub fn run_final_auditor_experiment() {
    println!("=== FINAL AUDITOR EXPERIMENT ===");
    println!("Testing the EXACT same operations as Criterion benchmarks");
    println!();
    
    let tree_size = 10_000;
    
    // Create identical trees
    let mut our_tree = GlobalCapacityBPlusTreeMap::new(16).expect("Failed to create tree");
    let mut std_tree = BTreeMap::new();
    
    for i in 0..tree_size {
        our_tree.insert(i, i).expect("Failed to insert");
        std_tree.insert(i, i);
    }
    
    // Test 1: Range creation only (what Criterion tests)
    test_range_creation_only(&our_tree, &std_tree);
    
    // Test 2: Small range collect (what Criterion tests)
    test_small_range_collect(&our_tree, &std_tree);
    
    // Test 3: Large range collect (what Criterion tests)
    test_large_range_collect(&our_tree, &std_tree);
    
    // Test 4: Count operation (what I was testing)
    test_count_operation(&our_tree, &std_tree);
    
    println!("=== ANALYSIS ===");
    println!("The discrepancy is explained:");
    println!("1. Criterion tests range CREATION and COLLECT operations");
    println!("2. My tests focused on COUNT operations");
    println!("3. Different operations have different performance characteristics");
    println!("4. Our B+ tree may be faster at iteration but slower at creation/collection");
}

fn test_range_creation_only(
    our_tree: &GlobalCapacityBPlusTreeMap<i32, i32>,
    std_tree: &BTreeMap<i32, i32>,
) {
    println!("--- Test 1: Range Creation Only (Criterion's test) ---");
    
    let iterations = 100_000;
    
    // Our tree - range creation only
    let start = Instant::now();
    for i in 0..iterations {
        let start_key = 4500 + (i % 100); // Vary slightly to prevent optimization
        let _iter = our_tree.range(start_key..start_key + 1000);
        // Don't iterate - just create the range iterator
    }
    let our_time = start.elapsed();
    
    // Std tree - range creation only
    let start = Instant::now();
    for i in 0..iterations {
        let start_key = 4500 + (i % 100);
        let _iter = std_tree.range(start_key..start_key + 1000);
        // Don't iterate - just create the range iterator
    }
    let std_time = start.elapsed();
    
    println!("  Range creation ({} iterations):", iterations);
    println!("    Our B+ tree: {:?} ({:.1}ns per creation)", our_time, our_time.as_nanos() as f64 / iterations as f64);
    println!("    std::BTreeMap: {:?} ({:.1}ns per creation)", std_time, std_time.as_nanos() as f64 / iterations as f64);
    
    if our_time < std_time {
        let speedup = std_time.as_nanos() as f64 / our_time.as_nanos() as f64;
        println!("    ✅ Our tree is {:.1}x FASTER at range creation", speedup);
    } else {
        let slowdown = our_time.as_nanos() as f64 / std_time.as_nanos() as f64;
        println!("    ❌ Our tree is {:.1}x SLOWER at range creation", slowdown);
    }
    println!();
}

fn test_small_range_collect(
    our_tree: &GlobalCapacityBPlusTreeMap<i32, i32>,
    std_tree: &BTreeMap<i32, i32>,
) {
    println!("--- Test 2: Small Range Collect (Criterion's test) ---");
    
    let iterations = 10_000;
    let range_size = 20;
    
    // Our tree - small range collect
    let start = Instant::now();
    for i in 0..iterations {
        let start_key = 4990 + (i % 100);
        let items: Vec<_> = our_tree.range(start_key..start_key + range_size).collect();
        assert_eq!(items.len(), range_size as usize);
    }
    let our_time = start.elapsed();
    
    // Std tree - small range collect
    let start = Instant::now();
    for i in 0..iterations {
        let start_key = 4990 + (i % 100);
        let items: Vec<_> = std_tree.range(start_key..start_key + range_size).collect();
        assert_eq!(items.len(), range_size as usize);
    }
    let std_time = start.elapsed();
    
    println!("  Small range collect ({} items, {} iterations):", range_size, iterations);
    println!("    Our B+ tree: {:?} ({:.1}µs per collect)", our_time, our_time.as_micros() as f64 / iterations as f64);
    println!("    std::BTreeMap: {:?} ({:.1}µs per collect)", std_time, std_time.as_micros() as f64 / iterations as f64);
    
    if our_time < std_time {
        let speedup = std_time.as_nanos() as f64 / our_time.as_nanos() as f64;
        println!("    ✅ Our tree is {:.1}x FASTER at small range collect", speedup);
    } else {
        let slowdown = our_time.as_nanos() as f64 / std_time.as_nanos() as f64;
        println!("    ❌ Our tree is {:.1}x SLOWER at small range collect", slowdown);
    }
    println!();
}

fn test_large_range_collect(
    our_tree: &GlobalCapacityBPlusTreeMap<i32, i32>,
    std_tree: &BTreeMap<i32, i32>,
) {
    println!("--- Test 3: Large Range Collect (Criterion's test) ---");
    
    let iterations = 100;
    let range_size = 6_000;
    
    // Our tree - large range collect
    let start = Instant::now();
    for i in 0..iterations {
        let start_key = 2000 + (i % 10) * 10;
        let items: Vec<_> = our_tree.range(start_key..start_key + range_size).collect();
        assert_eq!(items.len(), range_size as usize);
    }
    let our_time = start.elapsed();
    
    // Std tree - large range collect
    let start = Instant::now();
    for i in 0..iterations {
        let start_key = 2000 + (i % 10) * 10;
        let items: Vec<_> = std_tree.range(start_key..start_key + range_size).collect();
        assert_eq!(items.len(), range_size as usize);
    }
    let std_time = start.elapsed();
    
    println!("  Large range collect ({} items, {} iterations):", range_size, iterations);
    println!("    Our B+ tree: {:?} ({:.1}ms per collect)", our_time, our_time.as_millis() as f64 / iterations as f64);
    println!("    std::BTreeMap: {:?} ({:.1}ms per collect)", std_time, std_time.as_millis() as f64 / iterations as f64);
    
    if our_time < std_time {
        let speedup = std_time.as_nanos() as f64 / our_time.as_nanos() as f64;
        println!("    ✅ Our tree is {:.1}x FASTER at large range collect", speedup);
    } else {
        let slowdown = our_time.as_nanos() as f64 / std_time.as_nanos() as f64;
        println!("    ❌ Our tree is {:.1}x SLOWER at large range collect", slowdown);
    }
    println!();
}

fn test_count_operation(
    our_tree: &GlobalCapacityBPlusTreeMap<i32, i32>,
    std_tree: &BTreeMap<i32, i32>,
) {
    println!("--- Test 4: Count Operation (My test) ---");
    
    let iterations = 1_000;
    let range_size = 1_000;
    
    // Our tree - count operation
    let start = Instant::now();
    for i in 0..iterations {
        let start_key = 4000 + (i % 100);
        let count: usize = our_tree.range(start_key..start_key + range_size).count();
        assert_eq!(count, range_size as usize);
    }
    let our_time = start.elapsed();
    
    // Std tree - count operation
    let start = Instant::now();
    for i in 0..iterations {
        let start_key = 4000 + (i % 100);
        let count: usize = std_tree.range(start_key..start_key + range_size).count();
        assert_eq!(count, range_size as usize);
    }
    let std_time = start.elapsed();
    
    println!("  Count operation ({} items, {} iterations):", range_size, iterations);
    println!("    Our B+ tree: {:?} ({:.1}µs per count)", our_time, our_time.as_micros() as f64 / iterations as f64);
    println!("    std::BTreeMap: {:?} ({:.1}µs per count)", std_time, std_time.as_micros() as f64 / iterations as f64);
    
    if our_time < std_time {
        let speedup = std_time.as_nanos() as f64 / our_time.as_nanos() as f64;
        println!("    ✅ Our tree is {:.1}x FASTER at count operations", speedup);
    } else {
        let slowdown = our_time.as_nanos() as f64 / std_time.as_nanos() as f64;
        println!("    ❌ Our tree is {:.1}x SLOWER at count operations", slowdown);
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_final_auditor_experiment() {
        run_final_auditor_experiment();
    }
}
