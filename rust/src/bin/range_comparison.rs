use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::time::Instant;

fn main() {
    println!("=== BTreeMap vs BPlusTree Range Performance Comparison ===\n");

    // Test with large trees
    let tree_size = 500_000;
    println!("Building trees with {} elements...", tree_size);

    // Build BTreeMap
    let btree_start = Instant::now();
    let mut btree = BTreeMap::new();
    for i in 0..tree_size {
        btree.insert(i as i32, format!("value_{}", i));
    }
    let btree_build_time = btree_start.elapsed();

    // Build BPlusTree
    let bplus_start = Instant::now();
    let mut bplus = BPlusTreeMap::new(16).unwrap();
    for i in 0..tree_size {
        bplus.insert(i as i32, format!("value_{}", i));
    }
    let bplus_build_time = bplus_start.elapsed();

    println!(
        "BTreeMap build time:  {:.2}s",
        btree_build_time.as_secs_f64()
    );
    println!(
        "BPlusTree build time: {:.2}s",
        bplus_build_time.as_secs_f64()
    );
    println!();

    // Test different range sizes
    test_range_sizes(&btree, &bplus, tree_size);

    // Test range positions
    test_range_positions(&btree, &bplus, tree_size);

    // Test range startup vs iteration costs
    test_startup_vs_iteration(&btree, &bplus, tree_size);

    // Test range creation overhead
    test_creation_overhead(&btree, &bplus, tree_size);
}

fn test_range_sizes(
    btree: &BTreeMap<i32, String>,
    bplus: &BPlusTreeMap<i32, String>,
    tree_size: usize,
) {
    println!("=== Range Size Performance Comparison ===");

    let range_sizes = [1, 10, 100, 1000, 10000];
    let start_key = (tree_size / 2) as i32;

    println!("Range Size | BTreeMap Time | BPlusTree Time | Ratio (B+/BTree)");
    println!("-----------|---------------|----------------|------------------");

    for &range_size in &range_sizes {
        let end_key = start_key + range_size as i32;

        // BTreeMap range
        let btree_start = Instant::now();
        let btree_count = btree.range(start_key..end_key).count();
        let btree_time = btree_start.elapsed();

        // BPlusTree range
        let bplus_start = Instant::now();
        let bplus_count = bplus.range(start_key..end_key).count();
        let bplus_time = bplus_start.elapsed();

        let ratio = bplus_time.as_micros() as f64 / btree_time.as_micros() as f64;

        println!(
            "{:10} | {:9.1}µs ({:3}) | {:10.1}µs ({:3}) | {:8.1}x",
            range_size,
            btree_time.as_micros() as f64,
            btree_count,
            bplus_time.as_micros() as f64,
            bplus_count,
            ratio
        );
    }
    println!();
}

fn test_range_positions(
    btree: &BTreeMap<i32, String>,
    bplus: &BPlusTreeMap<i32, String>,
    tree_size: usize,
) {
    println!("=== Range Position Performance (1000 element ranges) ===");

    let range_size = 1000;
    let positions = [
        ("Start", 0),
        ("25%", tree_size / 4),
        ("50%", tree_size / 2),
        ("75%", 3 * tree_size / 4),
        ("End", tree_size - range_size - 1),
    ];

    println!("Position | BTreeMap Time | BPlusTree Time | Ratio (B+/BTree)");
    println!("---------|---------------|----------------|------------------");

    for (label, start_pos) in &positions {
        let start_key = *start_pos as i32;
        let end_key = start_key + range_size as i32;

        // BTreeMap range
        let btree_start = Instant::now();
        let btree_count = btree.range(start_key..end_key).count();
        let btree_time = btree_start.elapsed();

        // BPlusTree range
        let bplus_start = Instant::now();
        let bplus_count = bplus.range(start_key..end_key).count();
        let bplus_time = bplus_start.elapsed();

        let ratio = bplus_time.as_micros() as f64 / btree_time.as_micros() as f64;

        println!(
            "{:8} | {:9.1}µs ({:3}) | {:10.1}µs ({:3}) | {:8.1}x",
            label,
            btree_time.as_micros() as f64,
            btree_count,
            bplus_time.as_micros() as f64,
            bplus_count,
            ratio
        );
    }
    println!();
}

fn test_startup_vs_iteration(
    btree: &BTreeMap<i32, String>,
    bplus: &BPlusTreeMap<i32, String>,
    tree_size: usize,
) {
    println!("=== Range Startup vs Iteration Cost Analysis ===");

    let start_key = (tree_size / 2) as i32;

    // Test single element ranges (mostly startup cost)
    let btree_single_start = Instant::now();
    let btree_single_count = btree.range(start_key..start_key + 1).count();
    let btree_single_time = btree_single_start.elapsed();

    let bplus_single_start = Instant::now();
    let bplus_single_count = bplus.range(start_key..start_key + 1).count();
    let bplus_single_time = bplus_single_start.elapsed();

    // Test large ranges (startup + iteration cost)
    let large_size = 10000;
    let btree_large_start = Instant::now();
    let btree_large_count = btree
        .range(start_key..start_key + large_size as i32)
        .count();
    let btree_large_time = btree_large_start.elapsed();

    let bplus_large_start = Instant::now();
    let bplus_large_count = bplus
        .range(start_key..start_key + large_size as i32)
        .count();
    let bplus_large_time = bplus_large_start.elapsed();

    println!("Range Type        | BTreeMap  | BPlusTree | Ratio | Analysis");
    println!("------------------|-----------|-----------|-------|----------");
    println!(
        "Single element    | {:6.1}µs ({}) | {:6.1}µs ({}) | {:4.1}x | Startup cost",
        btree_single_time.as_micros() as f64,
        btree_single_count,
        bplus_single_time.as_micros() as f64,
        bplus_single_count,
        bplus_single_time.as_micros() as f64 / btree_single_time.as_micros() as f64
    );

    println!(
        "Large range       | {:6.1}µs ({}) | {:6.1}µs ({}) | {:4.1}x | Startup + iteration",
        btree_large_time.as_micros() as f64,
        btree_large_count,
        bplus_large_time.as_micros() as f64,
        bplus_large_count,
        bplus_large_time.as_micros() as f64 / btree_large_time.as_micros() as f64
    );

    // Calculate per-element iteration cost
    let btree_iter_cost = (btree_large_time.as_micros() as f64
        - btree_single_time.as_micros() as f64)
        / (btree_large_count - btree_single_count) as f64;
    let bplus_iter_cost = (bplus_large_time.as_micros() as f64
        - bplus_single_time.as_micros() as f64)
        / (bplus_large_count - bplus_single_count) as f64;

    println!(
        "Per-element cost  | {:6.3}µs    | {:6.3}µs    | {:4.1}x | Pure iteration",
        btree_iter_cost,
        bplus_iter_cost,
        bplus_iter_cost / btree_iter_cost
    );

    println!();
}

fn test_creation_overhead(
    btree: &BTreeMap<i32, String>,
    bplus: &BPlusTreeMap<i32, String>,
    tree_size: usize,
) {
    println!("=== Range Creation Overhead Test ===");

    let iterations = 10000;
    let start_key = (tree_size / 2) as i32;

    // Test range creation only (no iteration)
    let btree_create_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000);
        let _iter = btree.range(key..key + 1);
        // Don't consume iterator
    }
    let btree_create_time = btree_create_start.elapsed();

    let bplus_create_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000);
        let _iter = bplus.range(key..key + 1);
        // Don't consume iterator
    }
    let bplus_create_time = bplus_create_start.elapsed();

    // Test range creation + first element
    let btree_first_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000);
        let _first = btree.range(key..key + 1).next();
    }
    let btree_first_time = btree_first_start.elapsed();

    let bplus_first_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000);
        let _first = bplus.range(key..key + 1).next();
    }
    let bplus_first_time = bplus_first_start.elapsed();

    println!("Operation         | BTreeMap  | BPlusTree | Ratio | Per Operation");
    println!("------------------|-----------|-----------|-------|---------------");
    println!(
        "Range creation    | {:6.1}ms  | {:6.1}ms  | {:4.1}x | BTree: {:.3}µs, B+: {:.3}µs",
        btree_create_time.as_millis() as f64,
        bplus_create_time.as_millis() as f64,
        bplus_create_time.as_micros() as f64 / btree_create_time.as_micros() as f64,
        btree_create_time.as_micros() as f64 / iterations as f64,
        bplus_create_time.as_micros() as f64 / iterations as f64
    );

    println!(
        "Range + first()   | {:6.1}ms  | {:6.1}ms  | {:4.1}x | BTree: {:.3}µs, B+: {:.3}µs",
        btree_first_time.as_millis() as f64,
        bplus_first_time.as_millis() as f64,
        bplus_first_time.as_micros() as f64 / btree_first_time.as_micros() as f64,
        btree_first_time.as_micros() as f64 / iterations as f64,
        bplus_first_time.as_micros() as f64 / iterations as f64
    );
}
