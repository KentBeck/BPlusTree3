use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::time::Instant;

fn test_capacity(capacity: usize) -> (f64, f64, f64) {
    let size = 10000;

    // Insertion test
    let start = Instant::now();
    let mut bplus = BPlusTreeMap::new(capacity).unwrap();
    for i in 0..size {
        bplus.insert(i, i * 2);
    }
    let insert_time = start.elapsed().as_nanos() as f64;

    // Lookup test
    let iterations = 10000;
    let start = Instant::now();
    for i in 0..iterations {
        let key = i % size;
        let _ = bplus.get(&key);
    }
    let lookup_time = start.elapsed().as_nanos() as f64;

    // Iteration test
    let start = Instant::now();
    for _ in 0..10 {
        for (k, v) in bplus.items() {
            let _ = (k, v);
        }
    }
    let iter_time = start.elapsed().as_nanos() as f64;

    (insert_time, lookup_time, iter_time)
}

fn btree_baseline() -> (f64, f64, f64) {
    let size = 10000;

    // Insertion test
    let start = Instant::now();
    let mut btree = BTreeMap::new();
    for i in 0..size {
        btree.insert(i, i * 2);
    }
    let insert_time = start.elapsed().as_nanos() as f64;

    // Lookup test
    let iterations = 10000;
    let start = Instant::now();
    for i in 0..iterations {
        let key = i % size;
        let _ = btree.get(&key);
    }
    let lookup_time = start.elapsed().as_nanos() as f64;

    // Iteration test
    let start = Instant::now();
    for _ in 0..10 {
        for (k, v) in btree.iter() {
            let _ = (k, v);
        }
    }
    let iter_time = start.elapsed().as_nanos() as f64;

    (insert_time, lookup_time, iter_time)
}

fn main() {
    println!("Capacity Optimization Test");
    println!("==========================");

    // Get BTreeMap baseline
    let (btree_insert, btree_lookup, btree_iter) = btree_baseline();
    println!("BTreeMap baseline:");
    println!("  Insert: {:.0} ns", btree_insert);
    println!("  Lookup: {:.0} ns", btree_lookup);
    println!("  Iteration: {:.0} ns", btree_iter);
    println!();

    let capacities = [4, 8, 16, 32, 64, 128];

    println!("Capacity | Insert Ratio | Lookup Ratio | Iter Ratio | Notes");
    println!("---------|--------------|--------------|------------|-------");

    for &capacity in &capacities {
        let (insert_time, lookup_time, iter_time) = test_capacity(capacity);

        let insert_ratio = insert_time / btree_insert;
        let lookup_ratio = lookup_time / btree_lookup;
        let iter_ratio = iter_time / btree_iter;

        let best_lookup = lookup_ratio < 1.0;
        let notes = if best_lookup { "⭐ FASTER" } else { "" };

        println!(
            "{:8} | {:12.2}x | {:12.2}x | {:10.2}x | {}",
            capacity, insert_ratio, lookup_ratio, iter_ratio, notes
        );
    }

    println!("\nNote: Ratio < 1.0 = BPlusTree faster, > 1.0 = BTreeMap faster");
}
