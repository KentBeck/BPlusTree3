use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::time::Instant;

fn main() {
    println!("Iteration Performance Analysis");
    println!("==============================");

    let size = 10000;

    // Setup both data structures
    let mut btree = BTreeMap::new();
    let mut bplus = BPlusTreeMap::new(16).unwrap();

    for i in 0..size {
        btree.insert(i, i * 2);
        bplus.insert(i, i * 2);
    }

    println!("Dataset size: {} items", size);
    println!("BPlusTree capacity: 16");
    println!();

    // Measure BTreeMap iteration
    let iterations = 1000;
    let start = Instant::now();
    for _ in 0..iterations {
        let mut count = 0;
        for (k, v) in btree.iter() {
            count += 1;
            // Prevent optimization
            std::hint::black_box((k, v));
        }
        std::hint::black_box(count);
    }
    let btree_time = start.elapsed();

    // Measure BPlusTreeMap iteration
    let start = Instant::now();
    for _ in 0..iterations {
        let mut count = 0;
        for (k, v) in bplus.items() {
            count += 1;
            // Prevent optimization
            std::hint::black_box((k, v));
        }
        std::hint::black_box(count);
    }
    let bplus_time = start.elapsed();

    println!("=== ITERATION PERFORMANCE ===");
    println!("BTreeMap ({} iterations): {:?}", iterations, btree_time);
    println!("BPlusTreeMap ({} iterations): {:?}", iterations, bplus_time);
    println!(
        "Ratio (BPlus/BTree): {:.2}x",
        bplus_time.as_nanos() as f64 / btree_time.as_nanos() as f64
    );

    // Calculate per-item costs
    let btree_per_item = btree_time.as_nanos() as f64 / (iterations * size) as f64;
    let bplus_per_item = bplus_time.as_nanos() as f64 / (iterations * size) as f64;

    println!();
    println!("=== PER-ITEM ANALYSIS ===");
    println!("BTreeMap per item: {:.2} ns", btree_per_item);
    println!("BPlusTreeMap per item: {:.2} ns", bplus_per_item);
    println!(
        "Overhead per item: {:.2} ns",
        bplus_per_item - btree_per_item
    );

    // Test different capacities
    println!();
    println!("=== CAPACITY ANALYSIS ===");

    for &capacity in &[4, 8, 16, 32, 64, 128] {
        let mut bplus_cap = BPlusTreeMap::new(capacity).unwrap();
        for i in 0..size {
            bplus_cap.insert(i, i * 2);
        }

        let test_iterations = 100;
        let start = Instant::now();
        for _ in 0..test_iterations {
            let mut count = 0;
            for (k, v) in bplus_cap.items() {
                count += 1;
                std::hint::black_box((k, v));
            }
            std::hint::black_box(count);
        }
        let cap_time = start.elapsed();

        let ratio = cap_time.as_nanos() as f64
            / (btree_time.as_nanos() as f64 / iterations as f64 * test_iterations as f64);
        println!("Capacity {}: {:.2}x vs BTreeMap", capacity, ratio);
    }

    // Analyze iteration patterns
    println!();
    println!("=== ITERATION PATTERN ANALYSIS ===");

    // Test partial iteration (early break)
    let partial_size = 100;

    let start = Instant::now();
    for _ in 0..iterations {
        let mut count = 0;
        for (k, v) in btree.iter() {
            count += 1;
            std::hint::black_box((k, v));
            if count >= partial_size {
                break;
            }
        }
    }
    let btree_partial = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        let mut count = 0;
        for (k, v) in bplus.items() {
            count += 1;
            std::hint::black_box((k, v));
            if count >= partial_size {
                break;
            }
        }
    }
    let bplus_partial = start.elapsed();

    println!("Partial iteration ({} items):", partial_size);
    println!("  BTreeMap: {:?}", btree_partial);
    println!("  BPlusTreeMap: {:?}", bplus_partial);
    println!(
        "  Ratio: {:.2}x",
        bplus_partial.as_nanos() as f64 / btree_partial.as_nanos() as f64
    );
}
