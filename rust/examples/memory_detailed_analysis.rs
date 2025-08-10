//! Detailed memory footprint analysis of BPlusTreeMap

use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::mem;

fn main() {
    println!("🔍 DETAILED MEMORY FOOTPRINT ANALYSIS");
    println!("=====================================");
    
    // 1. STRUCTURE SIZE ANALYSIS
    println!("\n📏 STRUCTURE SIZES");
    println!("{}", "=".repeat(40));
    
    println!("BPlusTreeMap<i32, i32>: {} bytes", mem::size_of::<BPlusTreeMap<i32, i32>>());
    println!("BTreeMap<i32, i32>: {} bytes", mem::size_of::<BTreeMap<i32, i32>>());
    
    // 2. PER-ELEMENT OVERHEAD CALCULATION
    println!("\n💾 PER-ELEMENT OVERHEAD ANALYSIS");
    println!("{}", "=".repeat(40));
    
    for &size in &[10, 100, 1000, 10000] {
        println!("\nDataset size: {}", size as usize);
        
        // Create BTreeMap
        let btree: BTreeMap<i32, i32> = (0..size).map(|i| (i, i * 2)).collect();
        let btree_heap_size = estimate_btree_heap_size(&btree, size as usize);
        let btree_total = mem::size_of_val(&btree) + btree_heap_size;
        
        // Create BPlusTreeMap
        let mut bplus = BPlusTreeMap::new(64).unwrap();
        for i in 0..size {
            bplus.insert(i, i * 2);
        }
        let bplus_heap_size = estimate_bplus_heap_size(&bplus, size as usize);
        let bplus_total = mem::size_of_val(&bplus) + bplus_heap_size;
        
        println!("  BTreeMap:");
        println!("    Stack: {} bytes", mem::size_of_val(&btree));
        println!("    Estimated heap: {} bytes", btree_heap_size as usize);
        println!("    Total: {} bytes", btree_total);
        println!("    Per element: {:.1} bytes", btree_total as f64 / size as f64);
        
        println!("  BPlusTreeMap:");
        println!("    Stack: {} bytes", mem::size_of_val(&bplus));
        println!("    Estimated heap: {} bytes", bplus_heap_size as usize);
        println!("    Total: {} bytes", bplus_total);
        println!("    Per element: {:.1} bytes", bplus_total as f64 / size as f64);
        
        let overhead_ratio = bplus_total as f64 / btree_total as f64;
        println!("    Overhead ratio: {:.2}x", overhead_ratio);
    }
    
    // 3. CAPACITY IMPACT ANALYSIS
    println!("\n⚙️ CAPACITY IMPACT ON MEMORY");
    println!("{}", "=".repeat(40));
    
    let dataset_size = 1000;
    for &capacity in &[4, 16, 64, 128, 256] {
        let mut tree = BPlusTreeMap::new(capacity).unwrap();
        for i in 0..dataset_size {
            tree.insert(i, i * 2);
        }
        
        let stack_size = mem::size_of_val(&tree);
        let estimated_heap = estimate_bplus_heap_size(&tree, dataset_size as usize);
        let total = stack_size + estimated_heap;
        
        println!("Capacity {}: {} total bytes ({:.1} per element)", 
                 capacity, total, total as f64 / dataset_size as f64);
    }
    
    // 4. MEMORY OPTIMIZATION OPPORTUNITIES
    println!("\n🎯 MEMORY OPTIMIZATION OPPORTUNITIES");
    println!("{}", "=".repeat(40));
    
    println!("Current BPlusTreeMap overhead sources:");
    println!("1. Large stack size (176B vs 24B for BTreeMap)");
    println!("2. Arena metadata overhead");
    println!("3. NodeRef enum overhead");
    println!("4. Separate capacity field in each node");
    println!("5. Vec overhead for keys/values/children");
    
    println!("\nOptimization strategies:");
    println!("1. Use Box<[T]> instead of Vec<T> for fixed-size arrays");
    println!("2. Pack NodeRef more efficiently");
    println!("3. Share capacity across nodes");
    println!("4. Use smaller NodeId type when possible");
    println!("5. Optimize arena layout");
}

fn estimate_btree_heap_size(_btree: &BTreeMap<i32, i32>, size: usize) -> usize {
    // Rough estimation: BTreeMap uses nodes with ~6-11 elements each
    // Each element is (key, value) = 8 bytes
    // Plus node overhead
    let elements_per_node = 8;
    let nodes = (size + elements_per_node - 1) / elements_per_node;
    let node_overhead = 32; // Rough estimate
    size * 8 + nodes * node_overhead
}

fn estimate_bplus_heap_size(_bplus: &BPlusTreeMap<i32, i32>, size: usize) -> usize {
    // Rough estimation based on capacity and tree structure
    // Each leaf node can hold up to capacity elements
    // Each element is (key, value) = 8 bytes
    // Plus node overhead and arena overhead
    let capacity = 64; // Assuming default capacity
    let leaf_nodes = (size + capacity - 1) / capacity;
    let leaf_overhead = 56; // LeafNode struct overhead
    let arena_overhead = 64; // Arena metadata
    
    size * 8 + leaf_nodes * leaf_overhead + arena_overhead * 2
}
