//! Analysis of memory bloat sources in BPlusTreeMap

use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::mem;

fn main() {
    println!("🔍 MEMORY BLOAT SOURCE ANALYSIS");
    println!("================================");
    
    // 1. STACK SIZE BREAKDOWN
    println!("\n📏 STACK SIZE BREAKDOWN");
    println!("{}", "=".repeat(40));
    
    println!("BPlusTreeMap<i32, i32> total: {} bytes", mem::size_of::<BPlusTreeMap<i32, i32>>());
    println!("BTreeMap<i32, i32> total: {} bytes", mem::size_of::<BTreeMap<i32, i32>>());
    println!("Difference: {} bytes ({:.1}x larger)", 
             mem::size_of::<BPlusTreeMap<i32, i32>>() - mem::size_of::<BTreeMap<i32, i32>>(),
             mem::size_of::<BPlusTreeMap<i32, i32>>() as f64 / mem::size_of::<BTreeMap<i32, i32>>() as f64);
    
    // Component analysis
    println!("\nBPlusTreeMap component sizes:");
    println!("  capacity (usize): {} bytes", mem::size_of::<usize>());
    
    // We can't directly access private fields, but we can estimate
    println!("  root (NodeRef): ~{} bytes", 16); // Enum with NodeId + PhantomData
    println!("  leaf_arena: ~{} bytes", 72); // CompactArena with Vec + metadata
    println!("  branch_arena: ~{} bytes", 72); // CompactArena with Vec + metadata
    println!("  padding/alignment: ~{} bytes", 16); // Struct padding
    
    // 2. ARENA OVERHEAD ANALYSIS
    println!("\n🏗️ ARENA OVERHEAD ANALYSIS");
    println!("{}", "=".repeat(40));
    
    // Create empty arenas to measure base overhead
    let empty_tree = BPlusTreeMap::<i32, i32>::new(64).unwrap();
    println!("Empty BPlusTreeMap: {} bytes", mem::size_of_val(&empty_tree));
    
    // Add single element to see per-element cost
    let mut single_tree = BPlusTreeMap::new(64).unwrap();
    single_tree.insert(1, 1);
    println!("Single element BPlusTreeMap: {} bytes", mem::size_of_val(&single_tree));
    
    // Compare with BTreeMap
    let empty_btree = BTreeMap::<i32, i32>::new();
    println!("Empty BTreeMap: {} bytes", mem::size_of_val(&empty_btree));
    
    let mut single_btree = BTreeMap::new();
    single_btree.insert(1, 1);
    println!("Single element BTreeMap: {} bytes", mem::size_of_val(&single_btree));
    
    // 3. NODE STRUCTURE OVERHEAD
    println!("\n🏠 NODE STRUCTURE OVERHEAD");
    println!("{}", "=".repeat(40));
    
    // Estimate node sizes (we can't access private structs directly)
    println!("Estimated LeafNode<i32, i32> overhead:");
    println!("  capacity (usize): {} bytes", mem::size_of::<usize>());
    println!("  keys (Vec<i32>): {} bytes", mem::size_of::<Vec<i32>>());
    println!("  values (Vec<i32>): {} bytes", mem::size_of::<Vec<i32>>());
    println!("  next (NodeId): {} bytes", mem::size_of::<u32>());
    println!("  Total overhead: ~{} bytes", mem::size_of::<usize>() + 2 * mem::size_of::<Vec<i32>>() + mem::size_of::<u32>());
    
    println!("\nEstimated BranchNode<i32, i32> overhead:");
    println!("  capacity (usize): {} bytes", mem::size_of::<usize>());
    println!("  keys (Vec<i32>): {} bytes", mem::size_of::<Vec<i32>>());
    println!("  children (Vec<NodeRef>): {} bytes", mem::size_of::<Vec<u64>>()); // Approximate
    println!("  Total overhead: ~{} bytes", mem::size_of::<usize>() + mem::size_of::<Vec<i32>>() + mem::size_of::<Vec<u64>>());
    
    // 4. COMPARISON WITH OPTIMIZED STRUCTURES
    println!("\n⚡ OPTIMIZATION OPPORTUNITIES");
    println!("{}", "=".repeat(40));
    
    println!("Current inefficiencies:");
    println!("1. PhantomData in NodeRef: {} bytes wasted per reference", mem::size_of::<std::marker::PhantomData<(i32, i32)>>());
    println!("2. Separate capacity in each node: {} bytes per node", mem::size_of::<usize>());
    println!("3. Vec overhead: {} bytes per Vec", mem::size_of::<Vec<i32>>());
    println!("4. Arena metadata: ~{} bytes per arena", 72);
    
    println!("\nOptimization strategies:");
    println!("1. Remove PhantomData from NodeRef");
    println!("2. Use global capacity instead of per-node");
    println!("3. Use Box<[T]> for fixed-size arrays");
    println!("4. Pack NodeRef into smaller representation");
    println!("5. Use smaller NodeId type (u16) when possible");
    
    // 5. MEMORY LAYOUT ANALYSIS
    println!("\n📐 MEMORY LAYOUT ANALYSIS");
    println!("{}", "=".repeat(40));
    
    println!("Current BPlusTreeMap layout (estimated):");
    println!("  [capacity: 8B][root: 16B][leaf_arena: 72B][branch_arena: 72B][padding: 8B]");
    println!("  Total: 176 bytes");
    
    println!("\nOptimized layout (target):");
    println!("  [capacity: 8B][root: 8B][leaf_arena: 24B][branch_arena: 24B]");
    println!("  Total: 64 bytes (63% reduction)");
    
    // 6. IMPACT ON SMALL DATASETS
    println!("\n📊 IMPACT ON SMALL DATASETS");
    println!("{}", "=".repeat(40));
    
    for &size in &[1, 5, 10, 20, 50] {
        let overhead_current = 176.0; // Current stack size
        let overhead_optimized = 64.0; // Target stack size
        let data_size = size as f64 * 8.0; // 8 bytes per i32 key-value pair
        
        let current_ratio = (overhead_current + data_size) / data_size;
        let optimized_ratio = (overhead_optimized + data_size) / data_size;
        let improvement = (current_ratio - optimized_ratio) / current_ratio * 100.0;
        
        println!("Size {}: Current {:.2}x overhead, Optimized {:.2}x ({:.1}% improvement)",
                 size, current_ratio, optimized_ratio, improvement);
    }
    
    println!("\n🎯 PRIORITY RECOMMENDATIONS");
    println!("{}", "=".repeat(40));
    
    println!("HIGH PRIORITY (63% stack reduction):");
    println!("1. Remove PhantomData from NodeRef");
    println!("2. Pack NodeRef into u32 or u64");
    println!("3. Optimize arena layout");
    
    println!("\nMEDIUM PRIORITY (additional heap savings):");
    println!("4. Share capacity globally");
    println!("5. Use Box<[T]> for node storage");
    
    println!("\nLOW PRIORITY (marginal gains):");
    println!("6. Use u16 NodeId for small trees");
    println!("7. Custom allocator optimizations");
}
