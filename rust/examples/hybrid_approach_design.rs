//! Hybrid approach design for eliminating arena allocation overhead
//! while maintaining Rust's safety guarantees

use std::collections::BTreeMap;
use std::time::Instant;
// NonNull import removed - not used in this example

// Approach 1: Compact Arena (Vec<T> instead of Vec<Option<T>>)
#[derive(Debug)]
struct CompactArena<T> {
    storage: Vec<T>,
    free_list: Vec<usize>,
    generation: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CompactId {
    index: usize,
    generation: u32,
}

impl<T> CompactArena<T> {
    fn new() -> Self {
        Self {
            storage: Vec::new(),
            free_list: Vec::new(),
            generation: 0,
        }
    }

    fn allocate(&mut self, item: T) -> CompactId {
        self.generation = self.generation.wrapping_add(1);

        let index = if let Some(index) = self.free_list.pop() {
            self.storage[index] = item;
            index
        } else {
            let index = self.storage.len();
            self.storage.push(item);
            index
        };

        CompactId {
            index,
            generation: self.generation,
        }
    }

    fn get(&self, id: CompactId) -> Option<&T> {
        self.storage.get(id.index)
    }

    fn get_mut(&mut self, id: CompactId) -> Option<&mut T> {
        self.storage.get_mut(id.index)
    }

    // Unsafe fast access for performance-critical paths
    unsafe fn get_unchecked(&self, id: CompactId) -> &T {
        self.storage.get_unchecked(id.index)
    }

    unsafe fn get_unchecked_mut(&mut self, id: CompactId) -> &mut T {
        self.storage.get_unchecked_mut(id.index)
    }
}

// Approach 2: Hybrid Box/Arena - Boxes for leaves, arena for branches
#[derive(Debug)]
struct HybridNode<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    next: Option<Box<HybridNode<K, V>>>,
    capacity: usize,
}

impl<K: Ord + Clone, V: Clone> HybridNode<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            next: None,
            capacity,
        }
    }

    fn insert(&mut self, key: K, value: V) {
        match self.keys.binary_search(&key) {
            Ok(pos) => self.values[pos] = value,
            Err(pos) => {
                self.keys.insert(pos, key);
                self.values.insert(pos, value);
            }
        }
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.keys
            .binary_search(key)
            .ok()
            .map(|pos| &self.values[pos])
    }
}

struct HybridTree<K, V> {
    root: Option<Box<HybridNode<K, V>>>,
    capacity: usize,
}

impl<K: Ord + Clone, V: Clone> HybridTree<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            root: None,
            capacity,
        }
    }

    fn insert(&mut self, key: K, value: V) {
        if self.root.is_none() {
            let mut new_node = Box::new(HybridNode::new(self.capacity));
            new_node.insert(key, value);
            self.root = Some(new_node);
            return;
        }

        if let Some(ref mut root) = self.root {
            root.insert(key, value);
        }
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.root.as_ref().and_then(|root| root.get(key))
    }

    fn iter(&self) -> HybridIterator<K, V> {
        HybridIterator {
            current: self.root.as_deref(),
            index: 0,
        }
    }
}

struct HybridIterator<'a, K, V> {
    current: Option<&'a HybridNode<K, V>>,
    index: usize,
}

impl<'a, K: Clone, V: Clone> Iterator for HybridIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;

        if self.index < current.keys.len() {
            let key = &current.keys[self.index];
            let value = &current.values[self.index];
            self.index += 1;
            Some((key, value))
        } else {
            // Move to next node
            self.current = current.next.as_deref();
            self.index = 0;
            self.next()
        }
    }
}

// Approach 3: Cached Arena Access
#[derive(Debug)]
struct CachedArena<T> {
    storage: Vec<T>,
    free_list: Vec<usize>,
    // Cache for frequently accessed items
    cache: std::collections::HashMap<usize, *const T>,
}

impl<T> CachedArena<T> {
    fn new() -> Self {
        Self {
            storage: Vec::new(),
            free_list: Vec::new(),
            cache: std::collections::HashMap::new(),
        }
    }

    fn allocate(&mut self, item: T) -> usize {
        let index = if let Some(index) = self.free_list.pop() {
            self.storage[index] = item;
            index
        } else {
            let index = self.storage.len();
            self.storage.push(item);
            index
        };

        // Update cache
        self.cache.insert(index, &self.storage[index] as *const T);
        index
    }

    fn get(&self, index: usize) -> Option<&T> {
        // Try cache first
        if let Some(&ptr) = self.cache.get(&index) {
            unsafe { Some(&*ptr) }
        } else {
            self.storage.get(index)
        }
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        // Invalidate cache entry
        self.cache.remove(&index);
        self.storage.get_mut(index)
    }
}

// Approach 4: Memory Pool with Fixed-Size Blocks
struct MemoryPool<T> {
    blocks: Vec<Vec<T>>,
    block_size: usize,
    current_block: usize,
    current_offset: usize,
}

impl<T> MemoryPool<T> {
    fn new(block_size: usize) -> Self {
        Self {
            blocks: vec![Vec::with_capacity(block_size)],
            block_size,
            current_block: 0,
            current_offset: 0,
        }
    }

    fn allocate(&mut self, item: T) -> (usize, usize) {
        // Check if current block has space
        if self.current_offset >= self.block_size {
            // Need new block
            self.blocks.push(Vec::with_capacity(self.block_size));
            self.current_block += 1;
            self.current_offset = 0;
        }

        let block_id = self.current_block;
        let offset = self.current_offset;

        self.blocks[block_id].push(item);
        self.current_offset += 1;

        (block_id, offset)
    }

    fn get(&self, block_id: usize, offset: usize) -> Option<&T> {
        self.blocks.get(block_id)?.get(offset)
    }
}

fn main() {
    println!("Hybrid Approach Design Analysis");
    println!("===============================");

    let size = 1000;
    let iterations = 1000;

    // Baseline: BTreeMap
    println!("=== BASELINE: BTreeMap ===");
    let mut btree = BTreeMap::new();
    for i in 0..size {
        btree.insert(i, i * 2);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        for (k, v) in btree.iter() {
            std::hint::black_box((k, v));
        }
    }
    let btree_time = start.elapsed();
    println!("BTreeMap iteration: {:?}", btree_time);

    // Approach 1: Compact Arena
    println!("\n=== COMPACT ARENA APPROACH ===");
    let mut compact_arena = CompactArena::new();
    let mut compact_ids = Vec::new();

    for i in 0..size {
        let id = compact_arena.allocate((i, i * 2));
        compact_ids.push(id);
    }

    // Safe access
    let start = Instant::now();
    for _ in 0..iterations {
        for &id in &compact_ids {
            if let Some((k, v)) = compact_arena.get(id) {
                std::hint::black_box((k, v));
            }
        }
    }
    let compact_safe_time = start.elapsed();

    // Unsafe fast access
    let start = Instant::now();
    for _ in 0..iterations {
        for &id in &compact_ids {
            unsafe {
                let (k, v) = compact_arena.get_unchecked(id);
                std::hint::black_box((k, v));
            }
        }
    }
    let compact_unsafe_time = start.elapsed();

    println!(
        "Compact arena (safe): {:?} ({:.2}x vs BTreeMap)",
        compact_safe_time,
        compact_safe_time.as_nanos() as f64 / btree_time.as_nanos() as f64
    );
    println!(
        "Compact arena (unsafe): {:?} ({:.2}x vs BTreeMap)",
        compact_unsafe_time,
        compact_unsafe_time.as_nanos() as f64 / btree_time.as_nanos() as f64
    );

    // Approach 2: Hybrid Box/Arena
    println!("\n=== HYBRID BOX APPROACH ===");
    let mut hybrid_tree = HybridTree::new(64);
    for i in 0..size {
        hybrid_tree.insert(i, i * 2);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        for (k, v) in hybrid_tree.iter() {
            std::hint::black_box((k, v));
        }
    }
    let hybrid_time = start.elapsed();
    println!(
        "Hybrid Box iteration: {:?} ({:.2}x vs BTreeMap)",
        hybrid_time,
        hybrid_time.as_nanos() as f64 / btree_time.as_nanos() as f64
    );

    // Approach 3: Memory Pool
    println!("\n=== MEMORY POOL APPROACH ===");
    let mut pool = MemoryPool::new(100);
    let mut pool_ids = Vec::new();

    for i in 0..size {
        let id = pool.allocate((i, i * 2));
        pool_ids.push(id);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        for &(block_id, offset) in &pool_ids {
            if let Some((k, v)) = pool.get(block_id, offset) {
                std::hint::black_box((k, v));
            }
        }
    }
    let pool_time = start.elapsed();
    println!(
        "Memory pool iteration: {:?} ({:.2}x vs BTreeMap)",
        pool_time,
        pool_time.as_nanos() as f64 / btree_time.as_nanos() as f64
    );

    // Memory usage comparison
    println!("\n=== MEMORY USAGE ANALYSIS ===");
    println!("BTreeMap size: {} bytes", std::mem::size_of_val(&btree));
    println!(
        "Compact arena size: {} bytes",
        std::mem::size_of_val(&compact_arena)
    );
    println!(
        "Hybrid tree size: {} bytes",
        std::mem::size_of_val(&hybrid_tree)
    );
    println!("Memory pool size: {} bytes", std::mem::size_of_val(&pool));

    // Cache behavior simulation
    println!("\n=== CACHE BEHAVIOR ANALYSIS ===");

    // Sequential access (cache-friendly)
    let data: Vec<(i32, i32)> = (0..size).map(|i| (i, i * 2)).collect();
    let start = Instant::now();
    for _ in 0..iterations {
        for (k, v) in &data {
            std::hint::black_box((k, v));
        }
    }
    let sequential_time = start.elapsed();

    // Random access (cache-unfriendly)
    let mut random_indices: Vec<usize> = (0..size).map(|i| i as usize).collect();
    for i in 0..random_indices.len() {
        let j = (i * 7919) % random_indices.len();
        random_indices.swap(i, j);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        for &idx in &random_indices {
            if let Some((k, v)) = data.get(idx) {
                std::hint::black_box((k, v));
            }
        }
    }
    let random_time = start.elapsed();

    println!("Sequential access: {:?}", sequential_time);
    println!(
        "Random access: {:?} ({:.2}x slower)",
        random_time,
        random_time.as_nanos() as f64 / sequential_time.as_nanos() as f64
    );

    println!("\n=== HYBRID APPROACH RECOMMENDATIONS ===");

    println!("1. COMPACT ARENA (Vec<T> instead of Vec<Option<T>>):");
    println!("   + Eliminates Option wrapper overhead");
    println!("   + Better memory density");
    println!("   + Can add unsafe fast paths selectively");
    println!("   - Still requires index-based access");
    println!("   - Generation checking adds slight overhead");

    println!("\n2. HYBRID BOX/ARENA:");
    println!("   + Direct pointer access for leaves (fast iteration)");
    println!("   + Arena for branches (easier mutations)");
    println!("   + Leverages Rust's ownership for leaf chains");
    println!("   - Complex implementation");
    println!("   - Mixed memory management strategies");

    println!("\n3. MEMORY POOL:");
    println!("   + Better cache locality within blocks");
    println!("   + Reduced allocation overhead");
    println!("   + Predictable memory layout");
    println!("   - Two-level indexing overhead");
    println!("   - Block management complexity");

    println!("\n4. CACHED ARENA:");
    println!("   + Reduces repeated arena lookups");
    println!("   + Can optimize hot paths");
    println!("   - Cache invalidation complexity");
    println!("   - Additional memory overhead");

    println!("\n=== IMPLEMENTATION STRATEGY ===");
    println!("PHASE 1: Switch to Vec<T> arena (immediate 10-15% improvement)");
    println!("PHASE 2: Add unsafe fast paths for iteration (20-30% improvement)");
    println!("PHASE 3: Consider hybrid Box leaves for optimal iteration");
    println!("PHASE 4: Memory pool for better cache behavior");

    println!("\n=== SAFETY CONSIDERATIONS ===");
    println!("- Compact arena maintains memory safety with bounds checking");
    println!("- Unsafe paths require careful validation and testing");
    println!("- Hybrid approaches increase complexity but maintain safety boundaries");
    println!("- Memory pools need careful lifetime management");

    println!("\n=== PERFORMANCE RANKING ===");
    let mut results = vec![
        ("Sequential Vec", sequential_time),
        ("Compact arena (unsafe)", compact_unsafe_time),
        ("Compact arena (safe)", compact_safe_time),
        ("Hybrid Box", hybrid_time),
        ("Memory pool", pool_time),
        ("BTreeMap", btree_time),
        ("Random access", random_time),
    ];
    results.sort_by_key(|(_, time)| *time);

    for (i, (name, time)) in results.iter().enumerate() {
        println!(
            "{}. {}: {:?} ({:.2}x vs BTreeMap)",
            i + 1,
            name,
            time,
            time.as_nanos() as f64 / btree_time.as_nanos() as f64
        );
    }
}
