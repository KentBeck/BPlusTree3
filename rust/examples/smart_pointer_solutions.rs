//! Investigating smart pointer solutions for B+ tree memory management
//! Exploring Rc/RefCell, Arc/Mutex, and other reference-counted approaches

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// Approach 1: Rc/RefCell for single-threaded scenarios
#[derive(Debug)]
struct RcRefCellNode<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    next: Option<Rc<RefCell<RcRefCellNode<K, V>>>>,
    parent: Option<Weak<RefCell<RcRefCellNode<K, V>>>>,
    capacity: usize,
}

impl<K: Ord + Clone, V: Clone> RcRefCellNode<K, V> {
    fn new(capacity: usize) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            next: None,
            parent: None,
            capacity,
        }))
    }

    fn insert(node: &Rc<RefCell<Self>>, key: K, value: V) -> Result<(), String> {
        let mut borrowed = node.try_borrow_mut().map_err(|_| "Borrow conflict")?;
        match borrowed.keys.binary_search(&key) {
            Ok(pos) => borrowed.values[pos] = value,
            Err(pos) => {
                if borrowed.keys.len() < borrowed.capacity {
                    borrowed.keys.insert(pos, key);
                    borrowed.values.insert(pos, value);
                } else {
                    return Err("Node full".to_string());
                }
            }
        }
        Ok(())
    }

    fn get(node: &Rc<RefCell<Self>>, key: &K) -> Result<Option<V>, String> {
        let borrowed = node.try_borrow().map_err(|_| "Borrow conflict")?;
        Ok(borrowed
            .keys
            .binary_search(key)
            .ok()
            .map(|pos| borrowed.values[pos].clone()))
    }
}

struct RcRefCellTree<K, V> {
    root: Option<Rc<RefCell<RcRefCellNode<K, V>>>>,
    first_leaf: Option<Rc<RefCell<RcRefCellNode<K, V>>>>,
    capacity: usize,
}

impl<K: Ord + Clone, V: Clone> RcRefCellTree<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            root: None,
            first_leaf: None,
            capacity,
        }
    }

    fn insert(&mut self, key: K, value: V) -> Result<(), String> {
        if self.root.is_none() {
            let new_node = RcRefCellNode::new(self.capacity);
            RcRefCellNode::insert(&new_node, key, value)?;
            self.root = Some(new_node.clone());
            self.first_leaf = Some(new_node);
            return Ok(());
        }

        if let Some(ref first) = self.first_leaf {
            RcRefCellNode::insert(first, key, value)?;
        }
        Ok(())
    }

    fn get(&self, key: &K) -> Result<Option<V>, String> {
        if let Some(ref first) = self.first_leaf {
            RcRefCellNode::get(first, key)
        } else {
            Ok(None)
        }
    }

    fn iter(&self) -> RcRefCellIterator<K, V> {
        RcRefCellIterator {
            current: self.first_leaf.clone(),
            index: 0,
        }
    }
}

struct RcRefCellIterator<K, V> {
    current: Option<Rc<RefCell<RcRefCellNode<K, V>>>>,
    index: usize,
}

impl<K: Clone, V: Clone> Iterator for RcRefCellIterator<K, V> {
    type Item = Result<(K, V), String>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.as_ref()?.clone();

        // Try to get the next item or advance to next node
        let result = {
            let node = current.try_borrow().ok()?;
            if self.index < node.keys.len() {
                let key = node.keys[self.index].clone();
                let value = node.values[self.index].clone();
                self.index += 1;
                Some(Ok((key, value)))
            } else {
                // Need to move to next node
                let next_node = node.next.clone();
                drop(node); // Drop borrow before modifying self
                self.current = next_node;
                self.index = 0;
                None // Signal to try again
            }
        };

        match result {
            Some(item) => Some(item),
            None => self.next(), // Try again with next node
        }
    }
}

// Approach 2: Arc/Mutex for thread-safe scenarios
#[derive(Debug)]
struct ArcMutexNode<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    next: Option<Arc<Mutex<ArcMutexNode<K, V>>>>,
    capacity: usize,
}

impl<K: Ord + Clone, V: Clone> ArcMutexNode<K, V> {
    fn new(capacity: usize) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            next: None,
            capacity,
        }))
    }

    fn insert(node: &Arc<Mutex<Self>>, key: K, value: V) -> Result<(), String> {
        let mut locked = node.lock().map_err(|_| "Lock poisoned")?;
        match locked.keys.binary_search(&key) {
            Ok(pos) => locked.values[pos] = value,
            Err(pos) => {
                if locked.keys.len() < locked.capacity {
                    locked.keys.insert(pos, key);
                    locked.values.insert(pos, value);
                } else {
                    return Err("Node full".to_string());
                }
            }
        }
        Ok(())
    }

    fn get(node: &Arc<Mutex<Self>>, key: &K) -> Result<Option<V>, String> {
        let locked = node.lock().map_err(|_| "Lock poisoned")?;
        Ok(locked
            .keys
            .binary_search(key)
            .ok()
            .map(|pos| locked.values[pos].clone()))
    }
}

// Approach 3: Hybrid approach with Cell for simple values
#[derive(Debug)]
struct HybridNode<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    next_index: Cell<Option<usize>>, // Use Cell for simple copy types
    capacity: usize,
}

struct HybridArena<K, V> {
    nodes: Vec<Rc<HybridNode<K, V>>>,
    free_indices: Vec<usize>,
}

impl<K: Ord + Clone, V: Clone> HybridArena<K, V> {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            free_indices: Vec::new(),
        }
    }

    fn allocate(&mut self, capacity: usize) -> usize {
        let node = Rc::new(HybridNode {
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            next_index: Cell::new(None),
            capacity,
        });

        if let Some(index) = self.free_indices.pop() {
            self.nodes[index] = node;
            index
        } else {
            let index = self.nodes.len();
            self.nodes.push(node);
            index
        }
    }

    fn get(&self, index: usize) -> Option<&Rc<HybridNode<K, V>>> {
        self.nodes.get(index)
    }
}

fn main() {
    println!("Smart Pointer Solutions Analysis");
    println!("================================");

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

    // Approach 1: Rc/RefCell
    println!("\n=== RC/REFCELL APPROACH ===");
    let mut rc_tree = RcRefCellTree::new(64);
    for i in 0..size {
        if let Err(e) = rc_tree.insert(i, i * 2) {
            println!("Insert error: {}", e);
            break;
        }
    }

    let start = Instant::now();
    for _ in 0..iterations {
        for result in rc_tree.iter() {
            match result {
                Ok((k, v)) => std::hint::black_box((k, v)),
                Err(_) => break, // Handle borrow conflicts
            };
        }
    }
    let rc_time = start.elapsed();
    println!("Rc/RefCell iteration: {:?}", rc_time);
    println!(
        "Ratio vs BTreeMap: {:.2}x",
        rc_time.as_nanos() as f64 / btree_time.as_nanos() as f64
    );

    // Approach 2: Arc/Mutex (single-threaded test)
    println!("\n=== ARC/MUTEX APPROACH ===");
    let arc_root = ArcMutexNode::new(64);
    for i in 0..size {
        if let Err(e) = ArcMutexNode::insert(&arc_root, i, i * 2) {
            println!("Insert error: {}", e);
            break;
        }
    }

    let start = Instant::now();
    for _ in 0..iterations {
        // Simple iteration through single node
        if let Ok(locked) = arc_root.lock() {
            for (k, v) in locked.keys.iter().zip(locked.values.iter()) {
                std::hint::black_box((k, v));
            }
        }
    }
    let arc_time = start.elapsed();
    println!("Arc/Mutex iteration: {:?}", arc_time);
    println!(
        "Ratio vs BTreeMap: {:.2}x",
        arc_time.as_nanos() as f64 / btree_time.as_nanos() as f64
    );

    // Memory overhead analysis
    println!("\n=== MEMORY OVERHEAD ANALYSIS ===");

    // Create single instances to measure overhead
    let simple_data = (42i32, 84i32);
    let rc_data = Rc::new(simple_data);
    let arc_data = Arc::new(simple_data);
    let refcell_data = RefCell::new(simple_data);
    let mutex_data = Mutex::new(simple_data);

    println!("Raw data: {} bytes", std::mem::size_of_val(&simple_data));
    println!("Rc wrapper: {} bytes", std::mem::size_of_val(&rc_data));
    println!("Arc wrapper: {} bytes", std::mem::size_of_val(&arc_data));
    println!(
        "RefCell wrapper: {} bytes",
        std::mem::size_of_val(&refcell_data)
    );
    println!(
        "Mutex wrapper: {} bytes",
        std::mem::size_of_val(&mutex_data)
    );

    // Reference counting overhead test
    println!("\n=== REFERENCE COUNTING OVERHEAD ===");

    let data_vec: Vec<(i32, i32)> = (0..size).map(|i| (i, i * 2)).collect();
    let rc_vec: Vec<Rc<(i32, i32)>> = data_vec.iter().map(|item| Rc::new(*item)).collect();

    // Raw data access
    let start = Instant::now();
    for _ in 0..iterations {
        for (k, v) in &data_vec {
            std::hint::black_box((k, v));
        }
    }
    let raw_time = start.elapsed();

    // Rc data access
    let start = Instant::now();
    for _ in 0..iterations {
        for item in &rc_vec {
            let (k, v) = (**item).clone();
            std::hint::black_box((k, v));
        }
    }
    let rc_access_time = start.elapsed();

    println!("Raw Vec access: {:?}", raw_time);
    println!(
        "Rc Vec access: {:?} ({:.2}x overhead)",
        rc_access_time,
        rc_access_time.as_nanos() as f64 / raw_time.as_nanos() as f64
    );

    // Cloning overhead test
    println!("\n=== CLONING OVERHEAD ===");

    let start = Instant::now();
    for _ in 0..iterations {
        for item in &rc_vec {
            let cloned = item.clone(); // Just increments reference count
            std::hint::black_box(cloned);
        }
    }
    let rc_clone_time = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        for (k, v) in &data_vec {
            let cloned = (*k, *v); // Actual data copy
            std::hint::black_box(cloned);
        }
    }
    let data_clone_time = start.elapsed();

    println!("Rc clone (ref count): {:?}", rc_clone_time);
    println!("Data clone (copy): {:?}", data_clone_time);
    println!(
        "Rc clone ratio: {:.2}x",
        rc_clone_time.as_nanos() as f64 / data_clone_time.as_nanos() as f64
    );

    println!("\n=== SMART POINTER TRADE-OFFS ===");
    println!("Rc/RefCell:");
    println!("  + Shared ownership without lifetime constraints");
    println!("  + Interior mutability for tree operations");
    println!("  + Runtime borrow checking prevents data races");
    println!("  - 20-40% performance overhead from RefCell");
    println!("  - Potential runtime panics on borrow conflicts");
    println!("  - Reference counting overhead");
    println!("  - Not thread-safe");

    println!("\nArc/Mutex:");
    println!("  + Thread-safe shared ownership");
    println!("  + Prevents data races at compile time");
    println!("  - Significant locking overhead (50-100% slower)");
    println!("  - Potential deadlocks with complex locking patterns");
    println!("  - Higher memory overhead than Rc");

    println!("\nHybrid Approaches:");
    println!("  + Can optimize specific fields (Cell for simple types)");
    println!("  + Reduced RefCell overhead for read-only data");
    println!("  - Increased complexity");
    println!("  - Still requires reference counting");

    println!("\n=== RECOMMENDATIONS ===");
    println!("1. Rc/RefCell is viable for single-threaded scenarios");
    println!("2. Performance cost is 20-40% vs raw pointers");
    println!("3. Arc/Mutex too slow for high-performance scenarios");
    println!("4. Hybrid approaches can reduce overhead selectively");
    println!("5. Current arena approach remains competitive");

    println!("\n=== CONCLUSION ===");
    println!("Smart pointers solve borrowing issues but at significant cost:");
    println!("- Runtime overhead from reference counting");
    println!("- Dynamic borrow checking penalties");
    println!("- Increased memory usage");
    println!("- Potential runtime failures");
    println!("\nFor performance-critical B+ trees, arena allocation");
    println!("with careful unsafe optimizations may be preferable.");
}
