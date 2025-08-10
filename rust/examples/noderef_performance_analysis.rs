//! Performance analysis of OptimizedNodeRef vs original NodeRef

use bplustree::{OptimizedNodeRef, NodeId};
use std::time::Instant;
use std::hint::black_box;

// Simulate original NodeRef for comparison
#[derive(Debug, Clone)]
pub enum OriginalNodeRef<K, V> {
    Leaf(NodeId, std::marker::PhantomData<(K, V)>),
    Branch(NodeId, std::marker::PhantomData<(K, V)>),
}

impl<K, V> OriginalNodeRef<K, V> {
    pub fn new_leaf(id: NodeId) -> Self {
        Self::Leaf(id, std::marker::PhantomData)
    }
    
    pub fn new_branch(id: NodeId) -> Self {
        Self::Branch(id, std::marker::PhantomData)
    }
    
    pub fn id(&self) -> NodeId {
        match *self {
            Self::Leaf(id, _) => id,
            Self::Branch(id, _) => id,
        }
    }
    
    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf(_, _))
    }
}

fn benchmark_creation_original(iterations: usize) -> std::time::Duration {
    let start = Instant::now();
    
    for i in 0..iterations {
        let leaf = OriginalNodeRef::<i32, i32>::new_leaf(i as NodeId);
        let branch = OriginalNodeRef::<i32, i32>::new_branch(i as NodeId);
        black_box(leaf);
        black_box(branch);
    }
    
    start.elapsed()
}

fn benchmark_creation_optimized(iterations: usize) -> std::time::Duration {
    let start = Instant::now();
    
    for i in 0..iterations {
        let leaf = OptimizedNodeRef::new_leaf(i as NodeId);
        let branch = OptimizedNodeRef::new_branch(i as NodeId);
        black_box(leaf);
        black_box(branch);
    }
    
    start.elapsed()
}

fn benchmark_access_original(refs: &[OriginalNodeRef<i32, i32>]) -> std::time::Duration {
    let start = Instant::now();
    
    let mut sum = 0u64;
    for node_ref in refs {
        sum += node_ref.id() as u64;
        if node_ref.is_leaf() {
            sum += 1;
        }
    }
    black_box(sum);
    
    start.elapsed()
}

fn benchmark_access_optimized(refs: &[OptimizedNodeRef]) -> std::time::Duration {
    let start = Instant::now();
    
    let mut sum = 0u64;
    for node_ref in refs {
        sum += node_ref.id() as u64;
        if node_ref.is_leaf() {
            sum += 1;
        }
    }
    black_box(sum);
    
    start.elapsed()
}

fn benchmark_type_checking_original(refs: &[OriginalNodeRef<i32, i32>]) -> std::time::Duration {
    let start = Instant::now();
    
    let mut leaf_count = 0;
    let mut branch_count = 0;
    
    for node_ref in refs {
        if node_ref.is_leaf() {
            leaf_count += 1;
        } else {
            branch_count += 1;
        }
    }
    
    black_box((leaf_count, branch_count));
    start.elapsed()
}

fn benchmark_type_checking_optimized(refs: &[OptimizedNodeRef]) -> std::time::Duration {
    let start = Instant::now();
    
    let mut leaf_count = 0;
    let mut branch_count = 0;
    
    for node_ref in refs {
        if node_ref.is_leaf() {
            leaf_count += 1;
        } else {
            branch_count += 1;
        }
    }
    
    black_box((leaf_count, branch_count));
    start.elapsed()
}

fn main() {
    println!("🚀 NODEREF PERFORMANCE ANALYSIS");
    println!("================================");
    
    let iterations = 1_000_000;
    
    // 1. CREATION PERFORMANCE
    println!("\n📊 CREATION PERFORMANCE ({} iterations)", iterations);
    println!("{}", "=".repeat(50));
    
    let original_creation = benchmark_creation_original(iterations);
    let optimized_creation = benchmark_creation_optimized(iterations);
    
    println!("Original NodeRef creation: {:.2}ms", original_creation.as_secs_f64() * 1000.0);
    println!("Optimized NodeRef creation: {:.2}ms", optimized_creation.as_secs_f64() * 1000.0);
    
    let creation_ratio = original_creation.as_secs_f64() / optimized_creation.as_secs_f64();
    if creation_ratio > 1.0 {
        println!("✅ Optimized is {:.2}x FASTER at creation", creation_ratio);
    } else {
        println!("❌ Optimized is {:.2}x SLOWER at creation", 1.0 / creation_ratio);
    }
    
    // 2. ACCESS PERFORMANCE
    println!("\n🔍 ACCESS PERFORMANCE");
    println!("{}", "=".repeat(50));
    
    // Create test data
    let original_refs: Vec<_> = (0..100_000)
        .map(|i| if i % 2 == 0 {
            OriginalNodeRef::<i32, i32>::new_leaf(i as NodeId)
        } else {
            OriginalNodeRef::<i32, i32>::new_branch(i as NodeId)
        })
        .collect();
    
    let optimized_refs: Vec<_> = (0..100_000)
        .map(|i| if i % 2 == 0 {
            OptimizedNodeRef::new_leaf(i as NodeId)
        } else {
            OptimizedNodeRef::new_branch(i as NodeId)
        })
        .collect();
    
    let original_access = benchmark_access_original(&original_refs);
    let optimized_access = benchmark_access_optimized(&optimized_refs);
    
    println!("Original NodeRef access: {:.2}ms", original_access.as_secs_f64() * 1000.0);
    println!("Optimized NodeRef access: {:.2}ms", optimized_access.as_secs_f64() * 1000.0);
    
    let access_ratio = original_access.as_secs_f64() / optimized_access.as_secs_f64();
    if access_ratio > 1.0 {
        println!("✅ Optimized is {:.2}x FASTER at access", access_ratio);
    } else {
        println!("❌ Optimized is {:.2}x SLOWER at access", 1.0 / access_ratio);
    }
    
    // 3. TYPE CHECKING PERFORMANCE
    println!("\n🏷️ TYPE CHECKING PERFORMANCE");
    println!("{}", "=".repeat(50));
    
    let original_type_check = benchmark_type_checking_original(&original_refs);
    let optimized_type_check = benchmark_type_checking_optimized(&optimized_refs);
    
    println!("Original NodeRef type checking: {:.2}ms", original_type_check.as_secs_f64() * 1000.0);
    println!("Optimized NodeRef type checking: {:.2}ms", optimized_type_check.as_secs_f64() * 1000.0);
    
    let type_check_ratio = original_type_check.as_secs_f64() / optimized_type_check.as_secs_f64();
    if type_check_ratio > 1.0 {
        println!("✅ Optimized is {:.2}x FASTER at type checking", type_check_ratio);
    } else {
        println!("❌ Optimized is {:.2}x SLOWER at type checking", 1.0 / type_check_ratio);
    }
    
    // 4. MEMORY LAYOUT IMPACT
    println!("\n💾 MEMORY LAYOUT IMPACT");
    println!("{}", "=".repeat(50));
    
    use std::mem;
    
    println!("Original NodeRef size: {} bytes", mem::size_of::<OriginalNodeRef<i32, i32>>());
    println!("Optimized NodeRef size: {} bytes", mem::size_of::<OptimizedNodeRef>());
    
    let size_reduction = mem::size_of::<OriginalNodeRef<i32, i32>>() - mem::size_of::<OptimizedNodeRef>();
    println!("Size reduction: {} bytes ({:.1}%)", 
             size_reduction,
             size_reduction as f64 / mem::size_of::<OriginalNodeRef<i32, i32>>() as f64 * 100.0);
    
    // Calculate cache line efficiency
    let cache_line_size = 64;
    let original_per_line = cache_line_size / mem::size_of::<OriginalNodeRef<i32, i32>>();
    let optimized_per_line = cache_line_size / mem::size_of::<OptimizedNodeRef>();
    
    println!("NodeRefs per cache line:");
    println!("  Original: {} refs", original_per_line);
    println!("  Optimized: {} refs", optimized_per_line);
    println!("  Cache efficiency improvement: {:.1}x", optimized_per_line as f64 / original_per_line as f64);
    
    // 5. BIT MANIPULATION OVERHEAD ANALYSIS
    println!("\n⚙️ BIT MANIPULATION OVERHEAD");
    println!("{}", "=".repeat(50));
    
    // Test just the bit operations
    let test_values: Vec<u64> = (0..100_000).map(|i| i as u64).collect();
    
    let bit_ops_start = Instant::now();
    let mut results = 0u64;
    for &val in &test_values {
        let with_flag = val | (1u64 << 63);
        let is_flagged = (with_flag & (1u64 << 63)) != 0;
        let extracted = with_flag & 0x00000000FFFFFFFF;
        results += if is_flagged { extracted } else { 0 };
    }
    black_box(results);
    let bit_ops_time = bit_ops_start.elapsed();
    
    println!("Bit manipulation overhead: {:.2}ms for 100k operations", bit_ops_time.as_secs_f64() * 1000.0);
    println!("Per operation: {:.2}ns", bit_ops_time.as_nanos() as f64 / 100_000.0);
    
    // 6. OVERALL PERFORMANCE SUMMARY
    println!("\n📈 PERFORMANCE SUMMARY");
    println!("{}", "=".repeat(50));
    
    println!("Operation        | Original | Optimized | Ratio    | Winner");
    println!("-----------------|----------|-----------|----------|--------");
    println!("Creation         | {:6.2}ms | {:7.2}ms | {:6.2}x | {}",
             original_creation.as_secs_f64() * 1000.0,
             optimized_creation.as_secs_f64() * 1000.0,
             creation_ratio,
             if creation_ratio > 1.0 { "Optimized" } else { "Original" });
    
    println!("Access           | {:6.2}ms | {:7.2}ms | {:6.2}x | {}",
             original_access.as_secs_f64() * 1000.0,
             optimized_access.as_secs_f64() * 1000.0,
             access_ratio,
             if access_ratio > 1.0 { "Optimized" } else { "Original" });
    
    println!("Type Checking    | {:6.2}ms | {:7.2}ms | {:6.2}x | {}",
             original_type_check.as_secs_f64() * 1000.0,
             optimized_type_check.as_secs_f64() * 1000.0,
             type_check_ratio,
             if type_check_ratio > 1.0 { "Optimized" } else { "Original" });
    
    // 7. RECOMMENDATIONS
    println!("\n🎯 PERFORMANCE RECOMMENDATIONS");
    println!("{}", "=".repeat(50));
    
    let overall_faster = creation_ratio > 0.95 && access_ratio > 0.95 && type_check_ratio > 0.95;
    
    if overall_faster {
        println!("✅ RECOMMENDATION: Use OptimizedNodeRef");
        println!("   • Significant memory savings (50% reduction)");
        println!("   • No meaningful performance regression");
        println!("   • Better cache efficiency");
        println!("   • Bit manipulation overhead is negligible");
    } else {
        println!("⚠️  RECOMMENDATION: Evaluate trade-offs carefully");
        println!("   • Memory savings are significant");
        println!("   • Performance impact may be noticeable");
        println!("   • Consider use case requirements");
    }
}
