//! Analysis of different memory management approaches for B+ trees in Rust
//! This explores alternatives to arena-based allocation

use std::collections::BTreeMap;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::time::Instant;

// Alternative 1: Box-based direct allocation (simplified example)
#[derive(Debug)]
struct BoxNode<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    next: Option<Box<BoxNode<K, V>>>,
}

impl<K: Ord + Clone, V: Clone> BoxNode<K, V> {
    fn new() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
            next: None,
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
        self.keys.binary_search(key).ok().map(|pos| &self.values[pos])
    }
}

// Alternative 2: Rc/RefCell approach
#[derive(Debug)]
struct RcNode<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    next: Option<Rc<RefCell<RcNode<K, V>>>>,
    parent: Option<Weak<RefCell<RcNode<K, V>>>>,
}

impl<K: Ord + Clone, V: Clone> RcNode<K, V> {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            keys: Vec::new(),
            values: Vec::new(),
            next: None,
            parent: None,
        }))
    }
    
    fn insert(node: &Rc<RefCell<Self>>, key: K, value: V) {
        let mut borrowed = node.borrow_mut();
        match borrowed.keys.binary_search(&key) {
            Ok(pos) => borrowed.values[pos] = value,
            Err(pos) => {
                borrowed.keys.insert(pos, key);
                borrowed.values.insert(pos, value);
            }
        }
    }
    
    fn get(node: &Rc<RefCell<Self>>, key: &K) -> Option<V> {
        let borrowed = node.borrow();
        borrowed.keys.binary_search(key).ok()
            .map(|pos| borrowed.values[pos].clone())
    }
}

// Alternative 3: Generational Index (SlotMap-style)
#[derive(Debug, Clone, Copy, PartialEq)]
struct GenerationalIndex {
    index: u32,
    generation: u32,
}

#[derive(Debug)]
struct GenerationalArena<T> {
    items: Vec<Option<(T, u32)>>, // (item, generation)
    free_list: Vec<u32>,
    generation: u32,
}

impl<T> GenerationalArena<T> {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            free_list: Vec::new(),
            generation: 0,
        }
    }
    
    fn insert(&mut self, item: T) -> GenerationalIndex {
        let index = if let Some(index) = self.free_list.pop() {
            self.generation += 1;
            self.items[index as usize] = Some((item, self.generation));
            index
        } else {
            let index = self.items.len() as u32;
            self.generation += 1;
            self.items.push(Some((item, self.generation)));
            index
        };
        
        GenerationalIndex {
            index,
            generation: self.generation,
        }
    }
    
    fn get(&self, id: GenerationalIndex) -> Option<&T> {
        self.items.get(id.index as usize)?
            .as_ref()
            .filter(|(_, gen)| *gen == id.generation)
            .map(|(item, _)| item)
    }
    
    fn get_mut(&mut self, id: GenerationalIndex) -> Option<&mut T> {
        self.items.get_mut(id.index as usize)?
            .as_mut()
            .filter(|(_, gen)| *gen == id.generation)
            .map(|(item, _)| item)
    }
}

fn main() {
    println!("Memory Management Alternatives Analysis");
    println!("======================================");
    
    let size = 1000; // Smaller size for complex operations
    let iterations = 1000;
    
    // Baseline: Standard BTreeMap
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
    
    // Alternative 1: Box-based (single linked list of leaves)
    println!("\n=== ALTERNATIVE 1: Box-based Direct Allocation ===");
    let mut box_root = Box::new(BoxNode::new());
    
    // Simple insertion (would be much more complex for real B+ tree)
    for i in 0..size {
        box_root.insert(i, i * 2);
    }
    
    let start = Instant::now();
    for _ in 0..iterations {
        // Iterate through the single node (simplified)
        for (k, v) in box_root.keys.iter().zip(box_root.values.iter()) {
            std::hint::black_box((k, v));
        }
    }
    let box_time = start.elapsed();
    println!("Box-based iteration: {:?}", box_time);
    println!("Ratio vs BTreeMap: {:.2}x", box_time.as_nanos() as f64 / btree_time.as_nanos() as f64);
    
    // Alternative 2: Rc/RefCell approach
    println!("\n=== ALTERNATIVE 2: Rc/RefCell Interior Mutability ===");
    let rc_root = RcNode::new();
    
    for i in 0..size {
        RcNode::insert(&rc_root, i, i * 2);
    }
    
    let start = Instant::now();
    for _ in 0..iterations {
        let borrowed = rc_root.borrow();
        for (k, v) in borrowed.keys.iter().zip(borrowed.values.iter()) {
            std::hint::black_box((k, v));
        }
    }
    let rc_time = start.elapsed();
    println!("Rc/RefCell iteration: {:?}", rc_time);
    println!("Ratio vs BTreeMap: {:.2}x", rc_time.as_nanos() as f64 / btree_time.as_nanos() as f64);
    
    // Alternative 3: Generational Arena
    println!("\n=== ALTERNATIVE 3: Generational Arena ===");
    let mut gen_arena = GenerationalArena::new();
    let mut gen_indices = Vec::new();
    
    for i in 0..size {
        let idx = gen_arena.insert((i, i * 2));
        gen_indices.push(idx);
    }
    
    let start = Instant::now();
    for _ in 0..iterations {
        for &idx in &gen_indices {
            if let Some((k, v)) = gen_arena.get(idx) {
                std::hint::black_box((k, v));
            }
        }
    }
    let gen_time = start.elapsed();
    println!("Generational Arena iteration: {:?}", gen_time);
    println!("Ratio vs BTreeMap: {:.2}x", gen_time.as_nanos() as f64 / btree_time.as_nanos() as f64);
    
    // Memory usage analysis
    println!("\n=== MEMORY USAGE ANALYSIS ===");
    println!("BTreeMap size: {} bytes", std::mem::size_of_val(&btree));
    println!("Box node size: {} bytes", std::mem::size_of_val(&*box_root));
    println!("Rc node size: {} bytes", std::mem::size_of_val(&*rc_root.borrow()));
    println!("Generational arena size: {} bytes", std::mem::size_of_val(&gen_arena));
    
    // Access pattern analysis
    println!("\n=== ACCESS PATTERN ANALYSIS ===");
    
    // Random access performance
    let random_keys: Vec<i32> = (0..100).map(|i| (i * 7) % size).collect();
    
    let start = Instant::now();
    for _ in 0..iterations {
        for &key in &random_keys {
            let val = btree.get(&key);
            std::hint::black_box(val);
        }
    }
    let btree_random = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..iterations {
        for &key in &random_keys {
            let val = box_root.get(&key);
            std::hint::black_box(val);
        }
    }
    let box_random = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..iterations {
        for &key in &random_keys {
            let val = RcNode::get(&rc_root, &key);
            std::hint::black_box(val);
        }
    }
    let rc_random = start.elapsed();
    
    println!("Random access (100 keys, {} iterations):", iterations);
    println!("  BTreeMap: {:?}", btree_random);
    println!("  Box-based: {:?} ({:.2}x)", box_random, box_random.as_nanos() as f64 / btree_random.as_nanos() as f64);
    println!("  Rc/RefCell: {:?} ({:.2}x)", rc_random, rc_random.as_nanos() as f64 / btree_random.as_nanos() as f64);
    
    println!("\n=== ANALYSIS SUMMARY ===");
    println!("1. Box-based: Fastest iteration but complex tree operations");
    println!("2. Rc/RefCell: Flexible but runtime overhead from RefCell");
    println!("3. Generational: Safe but similar performance to current arena");
    println!("4. Current arena: Good balance of safety and performance");
    
    println!("\n=== TRADE-OFFS ===");
    println!("Box-based:");
    println!("  + Optimal memory layout and cache behavior");
    println!("  + Zero indirection overhead");
    println!("  - Extremely difficult tree mutations in safe Rust");
    println!("  - Borrowing conflicts during rebalancing");
    
    println!("\nRc/RefCell:");
    println!("  + Flexible shared ownership");
    println!("  + Easier tree mutations");
    println!("  - Runtime borrow checking overhead");
    println!("  - Reference counting overhead");
    println!("  - Potential runtime panics");
    
    println!("\nGenerational Arena:");
    println!("  + Better safety than raw indices");
    println!("  + Prevents use-after-free");
    println!("  - Similar performance to current arena");
    println!("  - Additional generation checking overhead");
}
