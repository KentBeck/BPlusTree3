use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::time::Instant;

fn main() {
    println!("Quick Performance Comparison: BPlusTreeMap vs BTreeMap");
    println!("========================================================");

    // Insertion benchmark
    println!("\n=== INSERTION BENCHMARK ===");
    let size = 10000;

    let start = Instant::now();
    let mut btree = BTreeMap::new();
    for i in 0..size {
        btree.insert(i, i * 2);
    }
    let btree_insert_time = start.elapsed();

    let start = Instant::now();
    let mut bplus = BPlusTreeMap::new(16).unwrap();
    for i in 0..size {
        bplus.insert(i, i * 2);
    }
    let bplus_insert_time = start.elapsed();

    println!("BTreeMap insertion ({}): {:?}", size, btree_insert_time);
    println!("BPlusTreeMap insertion ({}): {:?}", size, bplus_insert_time);
    println!(
        "Ratio (BPlus/BTree): {:.2}x",
        bplus_insert_time.as_nanos() as f64 / btree_insert_time.as_nanos() as f64
    );

    // Lookup benchmark
    println!("\n=== LOOKUP BENCHMARK ===");
    let iterations = 100000;

    let start = Instant::now();
    for i in 0..iterations {
        let key = i % size;
        let _ = btree.get(&key);
    }
    let btree_lookup_time = start.elapsed();

    let start = Instant::now();
    for i in 0..iterations {
        let key = i % size;
        let _ = bplus.get(&key);
    }
    let bplus_lookup_time = start.elapsed();

    println!("BTreeMap lookups ({}): {:?}", iterations, btree_lookup_time);
    println!(
        "BPlusTreeMap lookups ({}): {:?}",
        iterations, bplus_lookup_time
    );
    println!(
        "Ratio (BPlus/BTree): {:.2}x",
        bplus_lookup_time.as_nanos() as f64 / btree_lookup_time.as_nanos() as f64
    );

    // Iteration benchmark
    println!("\n=== ITERATION BENCHMARK ===");
    let iter_count = 100;

    let start = Instant::now();
    for _ in 0..iter_count {
        for (k, v) in btree.iter() {
            let _ = (k, v);
        }
    }
    let btree_iter_time = start.elapsed();

    let start = Instant::now();
    for _ in 0..iter_count {
        for (k, v) in bplus.items() {
            let _ = (k, v);
        }
    }
    let bplus_iter_time = start.elapsed();

    println!(
        "BTreeMap iteration ({}x): {:?}",
        iter_count, btree_iter_time
    );
    println!(
        "BPlusTreeMap iteration ({}x): {:?}",
        iter_count, bplus_iter_time
    );
    println!(
        "Ratio (BPlus/BTree): {:.2}x",
        bplus_iter_time.as_nanos() as f64 / btree_iter_time.as_nanos() as f64
    );

    println!("\nNote: Ratio < 1.0 means BPlusTree is faster, > 1.0 means BTreeMap is faster");
}
