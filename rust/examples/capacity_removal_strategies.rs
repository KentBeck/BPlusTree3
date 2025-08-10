//! Strategies for removing per-node capacity field

use std::mem;

// Current approach - each node stores capacity
#[derive(Debug, Clone)]
struct CurrentLeafNode<K, V> {
    capacity: usize,  // 8 bytes per node
    keys: Vec<K>,
    values: Vec<V>,
    next: u32,
}

// Strategy 1: Global capacity in tree root
#[derive(Debug, Clone)]
struct GlobalCapacityLeafNode<K, V> {
    // No capacity field - saved 8 bytes!
    keys: Vec<K>,
    values: Vec<V>,
    next: u32,
}

#[derive(Debug)]
struct TreeWithGlobalCapacity<K, V> {
    capacity: usize,  // Single capacity for entire tree
    root: u32,
    // ... other fields
    _phantom: std::marker::PhantomData<(K, V)>,
}

// Strategy 2: Capacity encoded in Vec allocation size
#[derive(Debug, Clone)]
struct InferredCapacityLeafNode<K, V> {
    keys: Vec<K>,     // Capacity inferred from Vec::capacity()
    values: Vec<V>,   // Must match keys.capacity()
    next: u32,
}

// Strategy 3: Fixed-size arrays with length tracking
#[derive(Debug, Clone)]
struct FixedSizeLeafNode<K, V> {
    keys: Box<[Option<K>]>,    // Fixed size determined at creation
    values: Box<[Option<V>]>,  // Same size as keys
    length: u16,               // Current number of elements
    next: u32,
}

// Strategy 4: Hybrid approach - Vec for growing, Box<[T]> when full
#[derive(Debug, Clone)]
enum HybridStorage<T> {
    Growing(Vec<T>),           // While adding elements
    Fixed(Box<[T]>),          // When node is full/stable
}

#[derive(Debug, Clone)]
struct HybridLeafNode<K, V> {
    keys: HybridStorage<K>,
    values: HybridStorage<V>,
    next: u32,
}

impl<K, V> CurrentLeafNode<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            next: u32::MAX,
        }
    }
    
    fn is_full(&self) -> bool {
        self.keys.len() >= self.capacity
    }
    
    fn can_insert(&self) -> bool {
        self.keys.len() < self.capacity
    }
}

impl<K, V> GlobalCapacityLeafNode<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            next: u32::MAX,
        }
    }
    
    fn is_full(&self, tree_capacity: usize) -> bool {
        self.keys.len() >= tree_capacity
    }
    
    fn can_insert(&self, tree_capacity: usize) -> bool {
        self.keys.len() < tree_capacity
    }
}

impl<K, V> InferredCapacityLeafNode<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            next: u32::MAX,
        }
    }
    
    fn capacity(&self) -> usize {
        // Infer capacity from Vec allocation
        std::cmp::min(self.keys.capacity(), self.values.capacity())
    }
    
    fn is_full(&self) -> bool {
        self.keys.len() >= self.capacity()
    }
    
    // Problem: What if Vec reallocates?
    fn insert(&mut self, key: K, value: V) -> Result<(), &'static str> {
        let original_capacity = self.capacity();
        
        self.keys.push(key);
        self.values.push(value);
        
        // Check if Vec reallocated (capacity changed)
        if self.capacity() != original_capacity {
            return Err("Vec reallocated - capacity inference broken!");
        }
        
        Ok(())
    }
}

impl<K: Clone, V: Clone> FixedSizeLeafNode<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            keys: vec![None; capacity].into_boxed_slice(),
            values: vec![None; capacity].into_boxed_slice(),
            length: 0,
            next: u32::MAX,
        }
    }
    
    fn capacity(&self) -> usize {
        self.keys.len()
    }
    
    fn is_full(&self) -> bool {
        self.length as usize >= self.capacity()
    }
    
    fn insert(&mut self, index: usize, key: K, value: V) -> Result<(), &'static str> {
        if self.is_full() {
            return Err("Node is full");
        }
        
        // Shift elements to make room
        for i in (index..self.length as usize).rev() {
            self.keys[i + 1] = self.keys[i].clone();
            self.values[i + 1] = self.values[i].clone();
        }
        
        self.keys[index] = Some(key);
        self.values[index] = Some(value);
        self.length += 1;
        
        Ok(())
    }
}

impl<K, V> HybridLeafNode<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            keys: HybridStorage::Growing(Vec::with_capacity(capacity)),
            values: HybridStorage::Growing(Vec::with_capacity(capacity)),
            next: u32::MAX,
        }
    }
    
    fn capacity(&self) -> usize {
        match (&self.keys, &self.values) {
            (HybridStorage::Growing(k), HybridStorage::Growing(v)) => {
                std::cmp::min(k.capacity(), v.capacity())
            }
            (HybridStorage::Fixed(k), HybridStorage::Fixed(v)) => {
                std::cmp::min(k.len(), v.len())
            }
            _ => panic!("Inconsistent storage types"),
        }
    }
    
    fn freeze_when_full(&mut self) {
        if self.is_full() {
            // Convert Vec to Box<[T]> to save memory
            match (&mut self.keys, &mut self.values) {
                (HybridStorage::Growing(k), HybridStorage::Growing(v)) => {
                    let keys_box = std::mem::take(k).into_boxed_slice();
                    let values_box = std::mem::take(v).into_boxed_slice();
                    
                    self.keys = HybridStorage::Fixed(keys_box);
                    self.values = HybridStorage::Fixed(values_box);
                }
                _ => {} // Already fixed
            }
        }
    }
    
    fn is_full(&self) -> bool {
        match (&self.keys, &self.values) {
            (HybridStorage::Growing(k), HybridStorage::Growing(v)) => {
                k.len() >= k.capacity() || v.len() >= v.capacity()
            }
            (HybridStorage::Fixed(k), HybridStorage::Fixed(v)) => {
                k.len() >= k.len() // Fixed arrays are always "full"
            }
            _ => panic!("Inconsistent storage types"),
        }
    }
}

fn analyze_memory_usage() {
    println!("📊 MEMORY USAGE COMPARISON");
    println!("{}", "=".repeat(50));
    
    println!("Structure sizes (empty nodes):");
    println!("Current:           {} bytes", mem::size_of::<CurrentLeafNode<i32, i32>>());
    println!("Global capacity:   {} bytes", mem::size_of::<GlobalCapacityLeafNode<i32, i32>>());
    println!("Inferred capacity: {} bytes", mem::size_of::<InferredCapacityLeafNode<i32, i32>>());
    println!("Fixed size:        {} bytes", mem::size_of::<FixedSizeLeafNode<i32, i32>>());
    println!("Hybrid:            {} bytes", mem::size_of::<HybridLeafNode<i32, i32>>());
    
    let current_size = mem::size_of::<CurrentLeafNode<i32, i32>>();
    let global_size = mem::size_of::<GlobalCapacityLeafNode<i32, i32>>();
    let savings = current_size - global_size;
    
    println!("\nMemory savings: {} bytes ({:.1}%)", 
             savings, 
             savings as f64 / current_size as f64 * 100.0);
}

fn demonstrate_strategies() {
    println!("\n🔧 STRATEGY DEMONSTRATIONS");
    println!("{}", "=".repeat(50));
    
    // Strategy 1: Global capacity
    println!("\n1. Global Capacity Strategy:");
    println!("✅ Pros:");
    println!("   • Simple implementation");
    println!("   • 8 bytes saved per node");
    println!("   • All nodes have same capacity");
    println!("❌ Cons:");
    println!("   • Need to pass capacity to node operations");
    println!("   • All nodes must have same capacity");
    
    // Strategy 2: Inferred capacity
    println!("\n2. Inferred Capacity Strategy:");
    println!("✅ Pros:");
    println!("   • No extra storage needed");
    println!("   • Can handle different capacities");
    println!("❌ Cons:");
    println!("   • Vec might reallocate unexpectedly");
    println!("   • Fragile - breaks if Vec grows");
    println!("   • Performance cost of capacity() calls");
    
    // Strategy 3: Fixed size arrays
    println!("\n3. Fixed Size Arrays Strategy:");
    println!("✅ Pros:");
    println!("   • Predictable memory usage");
    println!("   • No reallocation issues");
    println!("   • Cache-friendly layout");
    println!("❌ Cons:");
    println!("   • Option<T> overhead");
    println!("   • More complex insertion/deletion");
    println!("   • Memory waste for sparse nodes");
    
    // Strategy 4: Hybrid approach
    println!("\n4. Hybrid Storage Strategy:");
    println!("✅ Pros:");
    println!("   • Best of both worlds");
    println!("   • Memory efficient when full");
    println!("   • Flexible during growth");
    println!("❌ Cons:");
    println!("   • Complex implementation");
    println!("   • Runtime type checking");
    println!("   • Conversion overhead");
}

fn main() {
    println!("🎯 CAPACITY REMOVAL STRATEGIES ANALYSIS");
    println!("{}", "=".repeat(60));
    
    analyze_memory_usage();
    demonstrate_strategies();
    
    println!("\n💡 RECOMMENDED APPROACH");
    println!("{}", "=".repeat(50));
    
    println!("Best strategy: Global Capacity + Hybrid Storage");
    println!();
    println!("Implementation plan:");
    println!("1. Store capacity once in BPlusTreeMap root");
    println!("2. Pass capacity as parameter to node operations");
    println!("3. Use Vec<T> for growing nodes");
    println!("4. Convert to Box<[T]> when nodes become full/stable");
    println!("5. Save 8 bytes per node + Vec overhead when full");
    
    println!("\nExpected savings:");
    println!("• Per-node: 8 bytes (capacity field)");
    println!("• Full nodes: Additional 16 bytes (Vec → Box<[T]>)");
    println!("• Total: Up to 24 bytes per full node");
    
    println!("\n🚨 IMPLEMENTATION CHALLENGES");
    println!("{}", "=".repeat(50));
    
    println!("Key challenges to solve:");
    println!("1. Thread safety: Global capacity access");
    println!("2. API compatibility: Existing methods expect per-node capacity");
    println!("3. Node operations: Must pass capacity parameter");
    println!("4. Hybrid conversion: When to convert Vec → Box<[T]>");
    println!("5. Memory management: Ensure no leaks during conversion");
    
    println!("\n✅ SOLUTION APPROACH");
    println!("{}", "=".repeat(50));
    
    println!("Phase 1: Global capacity");
    println!("• Move capacity to BPlusTreeMap struct");
    println!("• Update all node methods to accept capacity parameter");
    println!("• Maintain Vec<T> storage for now");
    
    println!("\nPhase 2: Hybrid storage");
    println!("• Implement HybridStorage enum");
    println!("• Add conversion logic for full nodes");
    println!("• Optimize memory layout");
    
    println!("\nThis approach provides maximum memory savings while");
    println!("maintaining performance and API compatibility.");
}
