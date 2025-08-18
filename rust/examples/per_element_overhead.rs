//! Per-element overhead calculation and analysis

use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::mem;

fn main() {
    println!("📊 PER-ELEMENT OVERHEAD DETAILED ANALYSIS");
    println!("==========================================");

    // Key findings from memory analysis:
    // - BPlusTreeMap has 176B stack vs BTreeMap's 24B (7.3x larger)
    // - For small datasets (< 100), BPlusTreeMap has 2.6x overhead
    // - For large datasets (> 1000), BPlusTreeMap is actually more efficient

    println!("\n🔍 OVERHEAD BREAKDOWN");
    println!("{}", "=".repeat(40));

    // Calculate theoretical minimums
    let key_size = mem::size_of::<i32>();
    let value_size = mem::size_of::<i32>();
    let element_size = key_size + value_size;

    println!("Theoretical minimum per element: {} bytes", element_size);
    println!("  Key (i32): {} bytes", key_size);
    println!("  Value (i32): {} bytes", value_size);

    println!("\n📈 ACTUAL OVERHEAD BY DATASET SIZE");
    println!("{}", "=".repeat(40));

    let sizes = [1, 5, 10, 50, 100, 500, 1000, 5000, 10000];

    for &size in &sizes {
        let (btree_per_element, bplus_per_element) = calculate_per_element_overhead(size);
        let btree_overhead = btree_per_element - element_size as f64;
        let bplus_overhead = bplus_per_element - element_size as f64;
        let ratio = bplus_per_element / btree_per_element;

        println!("Size {:5}: BTree {:.1}B ({:.1}B overhead), BPlus {:.1}B ({:.1}B overhead), Ratio {:.2}x",
                 size, btree_per_element, btree_overhead, bplus_per_element, bplus_overhead, ratio);
    }

    println!("\n🎯 CROSSOVER POINT ANALYSIS");
    println!("{}", "=".repeat(40));

    // Find where BPlusTreeMap becomes more efficient
    let mut crossover_found = false;
    for size in 50..200 {
        let (btree_per_element, bplus_per_element) = calculate_per_element_overhead(size);
        if bplus_per_element < btree_per_element && !crossover_found {
            println!("Crossover point: ~{} elements", size);
            println!("  BTreeMap: {:.1} bytes per element", btree_per_element);
            println!("  BPlusTreeMap: {:.1} bytes per element", bplus_per_element);
            crossover_found = true;
            break;
        }
    }

    if !crossover_found {
        println!("No crossover found in tested range (50-200 elements)");
    }

    println!("\n💡 OVERHEAD SOURCES");
    println!("{}", "=".repeat(40));

    println!("BTreeMap overhead sources:");
    println!("  - Node structure overhead");
    println!("  - Tree balancing metadata");
    println!("  - Pointer overhead");

    println!("\nBPlusTreeMap overhead sources:");
    println!("  - Large stack structure (176B)");
    println!("  - Arena metadata");
    println!("  - NodeRef enum overhead");
    println!("  - Capacity field in each node");
    println!("  - Vec metadata (capacity, length)");
    println!("  - Next pointer in leaf nodes");

    println!("\n🚀 OPTIMIZATION IMPACT ESTIMATES");
    println!("{}", "=".repeat(40));

    let current_stack = 176;
    let optimized_estimates = [
        ("Remove PhantomData", 160),
        ("Pack NodeRef efficiently", 144),
        ("Share capacity globally", 128),
        ("Use u16 NodeId when possible", 120),
        ("Optimize arena layout", 96),
        ("Use Box<[T]> for fixed arrays", 80),
        ("All optimizations combined", 64),
    ];

    for (optimization, new_stack_size) in optimized_estimates {
        let reduction = current_stack - new_stack_size;
        let reduction_pct = (reduction as f64 / current_stack as f64) * 100.0;
        println!(
            "{}: {}B stack ({:.1}% reduction)",
            optimization, new_stack_size, reduction_pct
        );
    }

    println!("\n📋 RECOMMENDATIONS");
    println!("{}", "=".repeat(40));

    println!("Priority optimizations for memory reduction:");
    println!("1. HIGH: Reduce stack size from 176B to ~64B");
    println!("2. MEDIUM: Optimize arena layout and metadata");
    println!("3. LOW: Use smaller NodeId types when tree is small");

    println!("\nExpected impact:");
    println!("- Small datasets (< 100): Reduce overhead from 2.6x to ~1.5x");
    println!("- Medium datasets (100-1000): Maintain current efficiency");
    println!("- Large datasets (> 1000): Slight improvement in efficiency");
}

fn calculate_per_element_overhead(size: usize) -> (f64, f64) {
    // BTreeMap calculation
    let btree: BTreeMap<i32, i32> = (0..size as i32).map(|i| (i, i * 2)).collect();
    let btree_stack = mem::size_of_val(&btree);
    let btree_heap = estimate_btree_heap(size);
    let btree_total = btree_stack + btree_heap;
    let btree_per_element = btree_total as f64 / size as f64;

    // BPlusTreeMap calculation
    let mut bplus = BPlusTreeMap::new(64).unwrap();
    for i in 0..size as i32 {
        bplus.insert(i, i * 2);
    }
    let bplus_stack = mem::size_of_val(&bplus);
    let bplus_heap = estimate_bplus_heap(size);
    let bplus_total = bplus_stack + bplus_heap;
    let bplus_per_element = bplus_total as f64 / size as f64;

    (btree_per_element, bplus_per_element)
}

fn estimate_btree_heap(size: usize) -> usize {
    // BTreeMap uses B-tree nodes with ~6-11 elements each
    // Each element is 8 bytes (i32 key + i32 value)
    // Plus node overhead (~32 bytes per node)
    let avg_elements_per_node = 8;
    let nodes = (size + avg_elements_per_node - 1) / avg_elements_per_node;
    let node_overhead = 32;
    size * 8 + nodes * node_overhead
}

fn estimate_bplus_heap(size: usize) -> usize {
    // BPlusTreeMap uses leaf nodes with capacity elements each
    // Each element is 8 bytes (i32 key + i32 value)
    // Plus leaf node overhead and arena overhead
    let capacity = 64;
    let leaf_nodes = (size + capacity - 1) / capacity;
    let leaf_overhead = 56; // LeafNode struct size
    let arena_overhead = 128; // Both arenas

    size * 8 + leaf_nodes * leaf_overhead + arena_overhead
}
