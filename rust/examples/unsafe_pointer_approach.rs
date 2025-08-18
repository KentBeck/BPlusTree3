//! Exploring unsafe pointer-based approaches for B+ tree implementation
//! WARNING: This is experimental code for performance analysis only

use std::alloc::{alloc, dealloc, Layout};
use std::collections::BTreeMap;
use std::ptr::NonNull;
use std::time::Instant;

// Unsafe pointer-based node
#[repr(C)]
struct UnsafeNode<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    next: Option<NonNull<UnsafeNode<K, V>>>,
    capacity: usize,
}

impl<K: Ord + Clone, V: Clone> UnsafeNode<K, V> {
    fn new(capacity: usize) -> NonNull<Self> {
        let layout = Layout::new::<Self>();
        let ptr = unsafe { alloc(layout) as *mut Self };

        unsafe {
            ptr.write(Self {
                keys: Vec::with_capacity(capacity),
                values: Vec::with_capacity(capacity),
                next: None,
                capacity,
            });
            NonNull::new_unchecked(ptr)
        }
    }

    unsafe fn insert(&mut self, key: K, value: V) {
        match self.keys.binary_search(&key) {
            Ok(pos) => self.values[pos] = value,
            Err(pos) => {
                self.keys.insert(pos, key);
                self.values.insert(pos, value);
            }
        }
    }

    unsafe fn get(&self, key: &K) -> Option<&V> {
        self.keys
            .binary_search(key)
            .ok()
            .map(|pos| &self.values[pos])
    }

    unsafe fn deallocate(ptr: NonNull<Self>) {
        let layout = Layout::new::<Self>();
        // First drop the contents
        std::ptr::drop_in_place(ptr.as_ptr());
        // Then deallocate the memory
        dealloc(ptr.as_ptr() as *mut u8, layout);
    }
}

// Unsafe B+ tree with direct pointers
struct UnsafeBPlusTree<K, V> {
    root: Option<NonNull<UnsafeNode<K, V>>>,
    first_leaf: Option<NonNull<UnsafeNode<K, V>>>,
    size: usize,
    capacity: usize,
}

impl<K: Ord + Clone, V: Clone> UnsafeBPlusTree<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            root: None,
            first_leaf: None,
            size: 0,
            capacity,
        }
    }

    fn insert(&mut self, key: K, value: V) {
        if self.root.is_none() {
            let new_node = UnsafeNode::new(self.capacity);
            unsafe {
                new_node.as_ptr().as_mut().unwrap().insert(key, value);
            }
            self.root = Some(new_node);
            self.first_leaf = Some(new_node);
            self.size = 1;
            return;
        }

        // Simplified insertion - just add to first leaf for demo
        if let Some(first) = self.first_leaf {
            unsafe {
                first.as_ptr().as_mut().unwrap().insert(key, value);
            }
            self.size += 1;
        }
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.first_leaf
            .and_then(|first| unsafe { first.as_ref().get(key) })
    }

    // Unsafe iterator
    fn iter(&self) -> UnsafeIterator<K, V> {
        UnsafeIterator {
            current: self.first_leaf,
            index: 0,
        }
    }
}

impl<K, V> Drop for UnsafeBPlusTree<K, V> {
    fn drop(&mut self) {
        // Clean up all nodes
        let mut current = self.first_leaf;
        while let Some(node) = current {
            unsafe {
                let next = node.as_ref().next;
                // Manual cleanup without calling the generic deallocate
                let layout = std::alloc::Layout::new::<UnsafeNode<K, V>>();
                std::ptr::drop_in_place(node.as_ptr());
                std::alloc::dealloc(node.as_ptr() as *mut u8, layout);
                current = next;
            }
        }
    }
}

struct UnsafeIterator<K, V> {
    current: Option<NonNull<UnsafeNode<K, V>>>,
    index: usize,
}

impl<K: Clone, V: Clone> Iterator for UnsafeIterator<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;

        unsafe {
            let node = current.as_ref();

            if self.index < node.keys.len() {
                let key = node.keys[self.index].clone();
                let value = node.values[self.index].clone();
                self.index += 1;
                Some((key, value))
            } else {
                // Move to next node
                self.current = node.next;
                self.index = 0;
                self.next() // Recursive call to try next node
            }
        }
    }
}

// Alternative: Vec-based arena without Option wrapper
#[derive(Debug)]
struct CompactArena<T> {
    storage: Vec<T>,
    free_indices: Vec<usize>,
}

impl<T> CompactArena<T> {
    fn new() -> Self {
        Self {
            storage: Vec::new(),
            free_indices: Vec::new(),
        }
    }

    fn allocate(&mut self, item: T) -> usize {
        if let Some(index) = self.free_indices.pop() {
            self.storage[index] = item;
            index
        } else {
            let index = self.storage.len();
            self.storage.push(item);
            index
        }
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.storage.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.storage.get_mut(index)
    }

    // Unsafe fast access (no bounds checking)
    unsafe fn get_unchecked(&self, index: usize) -> &T {
        self.storage.get_unchecked(index)
    }

    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        self.storage.get_unchecked_mut(index)
    }
}

fn main() {
    println!("Unsafe Pointer-Based Approaches Analysis");
    println!("========================================");

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

    // Unsafe pointer approach
    println!("\n=== UNSAFE POINTER APPROACH ===");
    let mut unsafe_tree = UnsafeBPlusTree::new(64);
    for i in 0..size {
        unsafe_tree.insert(i, i * 2);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        for (k, v) in unsafe_tree.iter() {
            std::hint::black_box((k, v));
        }
    }
    let unsafe_time = start.elapsed();
    println!("Unsafe pointer iteration: {:?}", unsafe_time);
    println!(
        "Ratio vs BTreeMap: {:.2}x",
        unsafe_time.as_nanos() as f64 / btree_time.as_nanos() as f64
    );

    // Compact arena (Vec<T> instead of Vec<Option<T>>)
    println!("\n=== COMPACT ARENA APPROACH ===");
    let mut compact_arena = CompactArena::new();
    let mut indices = Vec::new();

    for i in 0..size {
        let idx = compact_arena.allocate((i, i * 2));
        indices.push(idx);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        for &idx in &indices {
            if let Some((k, v)) = compact_arena.get(idx) {
                std::hint::black_box((k, v));
            }
        }
    }
    let compact_time = start.elapsed();
    println!("Compact arena iteration: {:?}", compact_time);
    println!(
        "Ratio vs BTreeMap: {:.2}x",
        compact_time.as_nanos() as f64 / btree_time.as_nanos() as f64
    );

    // Unsafe compact arena access
    let start = Instant::now();
    for _ in 0..iterations {
        for &idx in &indices {
            unsafe {
                let (k, v) = compact_arena.get_unchecked(idx);
                std::hint::black_box((k, v));
            }
        }
    }
    let unsafe_compact_time = start.elapsed();
    println!("Unsafe compact arena iteration: {:?}", unsafe_compact_time);
    println!(
        "Ratio vs BTreeMap: {:.2}x",
        unsafe_compact_time.as_nanos() as f64 / btree_time.as_nanos() as f64
    );

    // Memory access pattern analysis
    println!("\n=== MEMORY ACCESS ANALYSIS ===");

    // Sequential access (cache-friendly)
    let data: Vec<(i32, i32)> = (0..size).map(|i| (i, i * 2)).collect();
    let start = Instant::now();
    for _ in 0..iterations {
        for (k, v) in &data {
            std::hint::black_box((k, v));
        }
    }
    let vec_time = start.elapsed();
    println!("Raw Vec iteration: {:?}", vec_time);
    println!(
        "Ratio vs BTreeMap: {:.2}x",
        vec_time.as_nanos() as f64 / btree_time.as_nanos() as f64
    );

    // Pointer chasing simulation
    let mut ptrs: Vec<*const (i32, i32)> = data.iter().map(|item| item as *const _).collect();
    // Shuffle to simulate pointer chasing
    for i in 0..ptrs.len() {
        let j = (i * 7919) % ptrs.len();
        ptrs.swap(i, j);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        for &ptr in &ptrs {
            unsafe {
                let (k, v) = &*ptr;
                std::hint::black_box((k, v));
            }
        }
    }
    let ptr_chase_time = start.elapsed();
    println!("Pointer chasing simulation: {:?}", ptr_chase_time);
    println!(
        "Ratio vs BTreeMap: {:.2}x",
        ptr_chase_time.as_nanos() as f64 / btree_time.as_nanos() as f64
    );

    println!("\n=== PERFORMANCE ANALYSIS ===");
    println!("Performance ranking (fastest to slowest):");
    let mut results = vec![
        ("Raw Vec", vec_time),
        ("Unsafe compact arena", unsafe_compact_time),
        ("Unsafe pointers", unsafe_time),
        ("Compact arena", compact_time),
        ("BTreeMap", btree_time),
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

    println!("\n=== SAFETY vs PERFORMANCE TRADE-OFFS ===");
    println!("Raw Vec:");
    println!("  + Optimal cache behavior");
    println!("  + Zero indirection");
    println!("  - Not suitable for tree structures");

    println!("\nUnsafe Compact Arena:");
    println!("  + Eliminates bounds checking");
    println!("  + No Option wrapper overhead");
    println!("  - Requires unsafe code");
    println!("  - Manual memory safety guarantees");

    println!("\nUnsafe Pointers:");
    println!("  + Direct memory access");
    println!("  + Optimal for tree traversal");
    println!("  - Complex memory management");
    println!("  - High risk of memory safety bugs");

    println!("\nCompact Arena (Safe):");
    println!("  + Eliminates Option wrapper");
    println!("  + Maintains memory safety");
    println!("  - Still has bounds checking overhead");
    println!("  - Index-based indirection");

    println!("\n=== RECOMMENDATIONS ===");
    println!("1. SHORT-TERM: Switch to Vec<T> instead of Vec<Option<T>> in arena");
    println!("2. MEDIUM-TERM: Add unsafe fast paths for hot iteration code");
    println!(
        "3. LONG-TERM: Consider hybrid safe/unsafe design for performance-critical applications"
    );
    println!("4. ALTERNATIVE: Focus on algorithmic optimizations rather than memory layout");
}
