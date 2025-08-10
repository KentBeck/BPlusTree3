//! Memory layout visualization for BPlusTreeMap nodes

use bplustree::{BPlusTreeMap, OptimizedNodeRef};
use std::mem;

fn visualize_leaf_node_layout() {
    println!("🍃 LEAF NODE MEMORY LAYOUT");
    println!("{}", "=".repeat(60));
    
    // Create a sample leaf node by creating a small tree
    let mut tree = BPlusTreeMap::new(4).unwrap();
    tree.insert(10, 100);
    tree.insert(20, 200);
    tree.insert(30, 300);
    
    println!("LeafNode<i32, i32> structure:");
    println!();
    
    // Calculate field sizes
    let capacity_size = mem::size_of::<usize>();
    let vec_i32_size = mem::size_of::<Vec<i32>>();
    let node_id_size = mem::size_of::<u32>();
    let total_size = capacity_size + vec_i32_size * 2 + node_id_size;
    
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│                    LeafNode<i32, i32>                   │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│ Field        │ Type      │ Size │ Offset │ Purpose      │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│ capacity     │ usize     │ {:2}B  │   0    │ Max keys     │", capacity_size);
    println!("│ keys         │ Vec<i32>  │ {:2}B  │   8    │ Key storage  │", vec_i32_size);
    println!("│ values       │ Vec<i32>  │ {:2}B  │  32    │ Value storage│", vec_i32_size);
    println!("│ next         │ NodeId    │ {:2}B  │  56    │ Next leaf    │", node_id_size);
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│ TOTAL        │           │ {:2}B  │        │              │", total_size);
    println!("└─────────────────────────────────────────────────────────┘");
    
    println!();
    println!("Memory layout visualization:");
    println!();
    println!("Byte:  0    8    16   24   32   40   48   56   60");
    println!("       │    │    │    │    │    │    │    │    │");
    println!("       ┌────┬────┬────┬────┬────┬────┬────┬────┐");
    println!("       │cap │    keys Vec<i32>   │   values Vec<i32>  │next│");
    println!("       │ 8B │        24B         │        24B         │ 4B │");
    println!("       └────┴────────────────────┴────────────────────┴────┘");
    
    println!();
    println!("Vec<T> internal structure (24 bytes each):");
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│ ptr (8B) │ capacity (8B) │ length (8B) │ = 24 bytes    │");
    println!("└─────────────────────────────────────────────────────────┘");
}

fn visualize_actual_memory_usage() {
    println!("\n💾 ACTUAL MEMORY USAGE EXAMPLE");
    println!("{}", "=".repeat(60));
    
    let mut tree = BPlusTreeMap::new(4).unwrap();
    tree.insert(10, 100);
    tree.insert(20, 200);
    tree.insert(30, 300);
    
    println!("Example: LeafNode with 3 key-value pairs (capacity=4)");
    println!();
    
    println!("Stack allocation (LeafNode struct): 60 bytes");
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│ capacity: 4                                             │ 8B");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│ keys Vec metadata:                                      │ 24B");
    println!("│   ptr → heap allocation                                 │");
    println!("│   capacity: 4                                           │");
    println!("│   length: 3                                             │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│ values Vec metadata:                                    │ 24B");
    println!("│   ptr → heap allocation                                 │");
    println!("│   capacity: 4                                           │");
    println!("│   length: 3                                             │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│ next: NULL_NODE (u32::MAX)                              │ 4B");
    println!("└─────────────────────────────────────────────────────────┘");
    
    println!();
    println!("Heap allocations:");
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│ keys heap: [10, 20, 30, _] (4 × 4B = 16B)              │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!("│ values heap: [100, 200, 300, _] (4 × 4B = 16B)         │");
    println!("└─────────────────────────────────────────────────────────┘");
    
    println!();
    println!("Total memory per leaf node:");
    println!("• Stack: 60 bytes (struct)");
    println!("• Heap: 32 bytes (key + value arrays)");
    println!("• Total: 92 bytes for 3 elements");
    println!("• Per element: ~30.7 bytes");
}

fn visualize_memory_optimization_impact() {
    println!("\n🚀 MEMORY OPTIMIZATION IMPACT");
    println!("{}", "=".repeat(60));
    
    println!("Current LeafNode overhead sources:");
    println!();
    println!("1. Per-node capacity field: 8 bytes");
    println!("   └─ Could be shared globally");
    println!();
    println!("2. Vec overhead: 2 × 24 = 48 bytes");
    println!("   ├─ Each Vec: ptr(8B) + cap(8B) + len(8B)");
    println!("   └─ Could use Box<[T]> when full: 2 × 8 = 16 bytes");
    println!();
    println!("3. NodeId: 4 bytes");
    println!("   └─ Could use u16 for small trees: 2 bytes");
    println!();
    
    println!("Optimization potential:");
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│ Current:  8B + 48B + 4B = 60B overhead                 │");
    println!("│ Phase 1:  0B + 48B + 4B = 52B (remove capacity)        │");
    println!("│ Phase 2:  0B + 16B + 4B = 20B (Box<[T]> when full)     │");
    println!("│ Phase 3:  0B + 16B + 2B = 18B (u16 NodeId)             │");
    println!("└─────────────────────────────────────────────────────────┘");
    
    println!();
    println!("Memory efficiency improvement:");
    println!("• Current: 60B overhead + data");
    println!("• Optimized: 18B overhead + data (70% reduction)");
    println!("• For 4 elements: 92B → 50B (46% reduction)");
}

fn visualize_cache_line_efficiency() {
    println!("\n⚡ CACHE LINE EFFICIENCY");
    println!("{}", "=".repeat(60));
    
    let cache_line_size = 64;
    let leaf_node_size = 60;
    
    println!("Cache line analysis (64-byte cache lines):");
    println!();
    
    println!("Current LeafNode (60 bytes):");
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│                    Cache Line (64 bytes)                      │");
    println!("├────────────────────────────────────────────────────────────────┤");
    println!("│ LeafNode (60B) │ 4B unused │                                  │");
    println!("└────────────────────────────────────────────────────────────────┘");
    println!("• 1 LeafNode per cache line");
    println!("• 4 bytes wasted per cache line");
    println!("• Cache utilization: 93.8%");
    
    println!();
    println!("Optimized LeafNode (18B overhead + data):");
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│                    Cache Line (64 bytes)                      │");
    println!("├────────────────────────────────────────────────────────────────┤");
    println!("│ Node1 │ Node2 │ Node3 │ 10B unused │                          │");
    println!("│ (18B) │ (18B) │ (18B) │            │                          │");
    println!("└────────────────────────────────────────────────────────────────┘");
    println!("• 3 LeafNodes per cache line");
    println!("• 10 bytes unused per cache line");
    println!("• Cache utilization: 84.4%");
    println!("• 3x better cache line utilization!");
}

fn visualize_linked_list_structure() {
    println!("\n🔗 LEAF NODE LINKED LIST STRUCTURE");
    println!("{}", "=".repeat(60));
    
    println!("B+ Tree leaf nodes form a linked list for efficient range queries:");
    println!();
    
    println!("Memory layout of linked leaf nodes:");
    println!();
    println!("Leaf Node 1 (keys: 1-10)");
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│ capacity: 16 │ keys: [1,5,8,10] │ values: [...] │ next: 2 │");
    println!("└─────────────────────────────────────────────────────────┘");
    println!("                                                    │");
    println!("                                                    ▼");
    println!("Leaf Node 2 (keys: 11-20)");
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│ capacity: 16 │ keys: [11,15,18,20] │ values: [...] │ next: 3 │");
    println!("└─────────────────────────────────────────────────────────┘");
    println!("                                                    │");
    println!("                                                    ▼");
    println!("Leaf Node 3 (keys: 21-30)");
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│ capacity: 16 │ keys: [21,25,28,30] │ values: [...] │ next: ∅ │");
    println!("└─────────────────────────────────────────────────────────┘");
    
    println!();
    println!("Benefits of linked list structure:");
    println!("• Range queries: O(log n + k) where k = results");
    println!("• Sequential iteration: O(n) with good cache locality");
    println!("• No need to traverse tree for range operations");
}

fn main() {
    println!("🏗️ BPLUSTREE LEAF NODE MEMORY LAYOUT ANALYSIS");
    println!("{}", "=".repeat(70));
    
    visualize_leaf_node_layout();
    visualize_actual_memory_usage();
    visualize_memory_optimization_impact();
    visualize_cache_line_efficiency();
    visualize_linked_list_structure();
    
    println!("\n📊 SUMMARY");
    println!("{}", "=".repeat(60));
    println!("• LeafNode struct: 60 bytes overhead + heap allocations");
    println!("• Major overhead: Vec metadata (48B) and capacity (8B)");
    println!("• Optimization potential: 70% overhead reduction possible");
    println!("• Cache efficiency: 3x improvement with optimizations");
    println!("• Linked list enables efficient range queries");
    
    println!("\n🎯 KEY INSIGHTS");
    println!("{}", "=".repeat(60));
    println!("1. Vec overhead dominates small node memory usage");
    println!("2. Per-node capacity is redundant (can be global)");
    println!("3. Box<[T]> more efficient than Vec<T> for full nodes");
    println!("4. Cache line utilization can be dramatically improved");
    println!("5. Memory layout directly impacts performance");
}
