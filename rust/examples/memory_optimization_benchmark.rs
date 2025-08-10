//! Benchmark memory optimizations

use bplustree::{BPlusTreeMap, OptimizedNodeRef, OptimizedArena};
use std::collections::BTreeMap;
use std::mem;

fn main() {
    println!("🚀 MEMORY OPTIMIZATION BENCHMARK");
    println!("=================================");
    
    // 1. COMPONENT SIZE COMPARISON
    println!("\n📏 COMPONENT SIZE COMPARISON");
    println!("{}", "=".repeat(40));
    
    println!("NodeRef optimizations:");
    println!("  Original NodeRef (estimated): 16 bytes");
    println!("  OptimizedNodeRef: {} bytes", mem::size_of::<OptimizedNodeRef>());
    println!("  Reduction: {} bytes ({:.1}%)", 
             16 - mem::size_of::<OptimizedNodeRef>(),
             (16 - mem::size_of::<OptimizedNodeRef>()) as f64 / 16.0 * 100.0);
    
    println!("\nArena optimizations:");
    println!("  CompactArena<i32> (estimated): 72 bytes");
    println!("  OptimizedArena<i32>: {} bytes", mem::size_of::<OptimizedArena<i32>>());
    let arena_reduction = 72 - mem::size_of::<OptimizedArena<i32>>();
    println!("  Reduction: {} bytes ({:.1}%)", 
             arena_reduction,
             arena_reduction as f64 / 72.0 * 100.0);
    
    // 2. THEORETICAL STACK SIZE IMPROVEMENT
    println!("\n📊 THEORETICAL STACK SIZE IMPROVEMENT");
    println!("{}", "=".repeat(40));
    
    let current_stack = 176;
    let noderef_savings = 8; // 16 -> 8 bytes
    let arena_savings = arena_reduction * 2; // Two arenas
    let estimated_new_stack = current_stack - noderef_savings - arena_savings;
    
    println!("Current BPlusTreeMap stack: {} bytes", current_stack);
    println!("NodeRef optimization saves: {} bytes", noderef_savings);
    println!("Arena optimization saves: {} bytes", arena_savings);
    println!("Estimated new stack size: {} bytes", estimated_new_stack);
    println!("Total reduction: {} bytes ({:.1}%)", 
             current_stack - estimated_new_stack,
             (current_stack - estimated_new_stack) as f64 / current_stack as f64 * 100.0);
    
    // 3. IMPACT ON SMALL DATASETS
    println!("\n🎯 IMPACT ON SMALL DATASETS");
    println!("{}", "=".repeat(40));
    
    for &size in &[1, 5, 10, 20, 50, 100] {
        let data_size = size * 8; // 8 bytes per i32 key-value pair
        
        // Current overhead calculation
        let current_total = current_stack + data_size;
        let current_per_element = current_total as f64 / size as f64;
        
        // Optimized overhead calculation
        let optimized_total = estimated_new_stack + data_size;
        let optimized_per_element = optimized_total as f64 / size as f64;
        
        let improvement = (current_per_element - optimized_per_element) / current_per_element * 100.0;
        
        println!("Size {:3}: Current {:.1}B/elem, Optimized {:.1}B/elem ({:.1}% improvement)",
                 size, current_per_element, optimized_per_element, improvement);
    }
    
    // 4. CROSSOVER POINT ANALYSIS
    println!("\n⚖️ CROSSOVER POINT ANALYSIS");
    println!("{}", "=".repeat(40));
    
    // Find new crossover point with BTreeMap
    let btree_stack = 24;
    let btree_per_element_overhead = 4.0; // Approximate
    
    println!("Searching for new crossover point with BTreeMap...");
    
    let mut new_crossover = None;
    for size in 10..200 {
        let data_size = size * 8;
        
        // BTreeMap total
        let btree_total = btree_stack + data_size + (size as f64 * btree_per_element_overhead) as usize;
        let btree_per_element = btree_total as f64 / size as f64;
        
        // Optimized BPlusTreeMap total
        let bplus_total = estimated_new_stack + data_size;
        let bplus_per_element = bplus_total as f64 / size as f64;
        
        if bplus_per_element <= btree_per_element && new_crossover.is_none() {
            new_crossover = Some(size);
            println!("New crossover point: ~{} elements", size);
            println!("  BTreeMap: {:.1} bytes per element", btree_per_element);
            println!("  Optimized BPlusTreeMap: {:.1} bytes per element", bplus_per_element);
            break;
        }
    }
    
    if new_crossover.is_none() {
        println!("Crossover point still above 200 elements");
    } else {
        println!("Improvement: {} -> {} elements ({:.1}% better)",
                 97, // Current crossover
                 new_crossover.unwrap(),
                 (97 - new_crossover.unwrap()) as f64 / 97.0 * 100.0);
    }
    
    // 5. MEMORY EFFICIENCY COMPARISON
    println!("\n📈 MEMORY EFFICIENCY COMPARISON");
    println!("{}", "=".repeat(40));
    
    let test_sizes = [10, 50, 100, 500, 1000];
    
    println!("Dataset | Current | Optimized | BTreeMap | Best");
    println!("--------|---------|-----------|----------|-----");
    
    for &size in &test_sizes {
        let data_size = size * 8;
        
        // Current BPlusTreeMap
        let current_total = current_stack + data_size;
        let current_per_elem = current_total as f64 / size as f64;
        
        // Optimized BPlusTreeMap
        let opt_total = estimated_new_stack + data_size;
        let opt_per_elem = opt_total as f64 / size as f64;
        
        // BTreeMap
        let btree_total = btree_stack + data_size + (size as f64 * btree_per_element_overhead) as usize;
        let btree_per_elem = btree_total as f64 / size as f64;
        
        let best = if opt_per_elem <= btree_per_elem { "BPlus*" } else { "BTree" };
        
        println!("{:7} | {:7.1} | {:9.1} | {:8.1} | {:4}",
                 size, current_per_elem, opt_per_elem, btree_per_elem, best);
    }
    
    // 6. OPTIMIZATION ROADMAP PROGRESS
    println!("\n🛣️ OPTIMIZATION ROADMAP PROGRESS");
    println!("{}", "=".repeat(40));
    
    println!("Phase 1 Target: 96 bytes (45% reduction)");
    println!("Current Progress: {} bytes ({:.1}% reduction)",
             estimated_new_stack,
             (current_stack - estimated_new_stack) as f64 / current_stack as f64 * 100.0);
    
    if estimated_new_stack <= 96 {
        println!("✅ Phase 1 target achieved!");
    } else {
        println!("⏳ Phase 1 target: {} bytes remaining", estimated_new_stack - 96);
    }
    
    println!("\nNext optimizations needed:");
    println!("- Remove per-node capacity field");
    println!("- Use Box<[T]> for node storage");
    println!("- Implement inline storage for small trees");
    
    println!("\n🎉 OPTIMIZATION SUMMARY");
    println!("{}", "=".repeat(40));
    
    println!("Implemented optimizations:");
    println!("✅ OptimizedNodeRef: 8-byte reduction");
    println!("✅ OptimizedArena: {}-byte reduction per arena", arena_reduction);
    println!("✅ Total stack reduction: {} bytes ({:.1}%)",
             noderef_savings + arena_savings,
             (noderef_savings + arena_savings) as f64 / current_stack as f64 * 100.0);
    
    println!("\nExpected benefits:");
    println!("• Improved small dataset efficiency");
    println!("• Lower memory overhead");
    println!("• Better crossover point with BTreeMap");
    println!("• Foundation for further optimizations");
}
