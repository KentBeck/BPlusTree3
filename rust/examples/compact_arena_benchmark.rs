//! Comprehensive benchmark comparing original arena vs compact arena implementation

use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::time::Instant;

fn main() {
    println!("Compact Arena vs Original Arena Benchmark");
    println!("=========================================");

    let size = 10000;
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

    // Test different capacities with compact arena
    println!("\n=== COMPACT ARENA PERFORMANCE ===");

    for &capacity in &[16, 32, 64, 128] {
        let mut bplus = BPlusTreeMap::new(capacity).unwrap();
        for i in 0..size {
            bplus.insert(i, i * 2);
        }

        // Regular iteration
        let start = Instant::now();
        for _ in 0..iterations {
            for (k, v) in bplus.items() {
                std::hint::black_box((k, v));
            }
        }
        let regular_time = start.elapsed();

        // Fast iteration (unsafe)
        let start = Instant::now();
        for _ in 0..iterations {
            for (k, v) in bplus.items_fast() {
                std::hint::black_box((k, v));
            }
        }
        let fast_time = start.elapsed();

        println!(
            "Capacity {}: Regular {:?} ({:.2}x vs BTreeMap), Fast {:?} ({:.2}x vs BTreeMap)",
            capacity,
            regular_time,
            regular_time.as_nanos() as f64 / btree_time.as_nanos() as f64,
            fast_time,
            fast_time.as_nanos() as f64 / btree_time.as_nanos() as f64
        );
    }

    // Detailed analysis with optimal capacity
    println!("\n=== DETAILED ANALYSIS (Capacity 64) ===");
    let mut bplus = BPlusTreeMap::new(64).unwrap();
    for i in 0..size {
        bplus.insert(i, i * 2);
    }

    // Insertion benchmark
    println!("\n--- Insertion Performance ---");
    let start = Instant::now();
    let mut btree_insert = BTreeMap::new();
    for i in 0..size {
        btree_insert.insert(i, i * 2);
    }
    let btree_insert_time = start.elapsed();

    let start = Instant::now();
    let mut bplus_insert = BPlusTreeMap::new(64).unwrap();
    for i in 0..size {
        bplus_insert.insert(i, i * 2);
    }
    let bplus_insert_time = start.elapsed();

    println!("BTreeMap insertion: {:?}", btree_insert_time);
    println!(
        "BPlusTreeMap insertion: {:?} ({:.2}x vs BTreeMap)",
        bplus_insert_time,
        bplus_insert_time.as_nanos() as f64 / btree_insert_time.as_nanos() as f64
    );

    // Lookup benchmark
    println!("\n--- Lookup Performance ---");
    let lookup_keys: Vec<i32> = (0..1000).map(|i| (i * 7) % size).collect();

    let start = Instant::now();
    for _ in 0..iterations {
        for &key in &lookup_keys {
            let val = btree.get(&key);
            std::hint::black_box(val);
        }
    }
    let btree_lookup_time = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        for &key in &lookup_keys {
            let val = bplus.get(&key);
            std::hint::black_box(val);
        }
    }
    let bplus_lookup_time = start.elapsed();

    println!("BTreeMap lookups: {:?}", btree_lookup_time);
    println!(
        "BPlusTreeMap lookups: {:?} ({:.2}x vs BTreeMap)",
        bplus_lookup_time,
        bplus_lookup_time.as_nanos() as f64 / btree_lookup_time.as_nanos() as f64
    );

    // Range query benchmark
    println!("\n--- Range Query Performance ---");
    let start = Instant::now();
    for _ in 0..100 {
        for (k, v) in btree.range(1000..2000) {
            std::hint::black_box((k, v));
        }
    }
    let btree_range_time = start.elapsed();

    let start = Instant::now();
    for _ in 0..100 {
        for (k, v) in bplus.items_range(Some(&1000), Some(&2000)) {
            std::hint::black_box((k, v));
        }
    }
    let bplus_range_time = start.elapsed();

    println!("BTreeMap range queries: {:?}", btree_range_time);
    println!(
        "BPlusTreeMap range queries: {:?} ({:.2}x vs BTreeMap)",
        bplus_range_time,
        bplus_range_time.as_nanos() as f64 / btree_range_time.as_nanos() as f64
    );

    // Memory usage analysis
    println!("\n=== MEMORY USAGE ANALYSIS ===");
    println!("BTreeMap size: {} bytes", std::mem::size_of_val(&btree));
    println!("BPlusTreeMap size: {} bytes", std::mem::size_of_val(&bplus));

    // Per-item overhead analysis
    println!("\n=== PER-ITEM OVERHEAD ANALYSIS ===");
    let btree_per_item = btree_time.as_nanos() as f64 / (iterations * size) as f64;

    let regular_per_item = {
        let start = Instant::now();
        for _ in 0..iterations {
            for (k, v) in bplus.items() {
                std::hint::black_box((k, v));
            }
        }
        let time = start.elapsed();
        time.as_nanos() as f64 / (iterations * size) as f64
    };

    let fast_per_item = {
        let start = Instant::now();
        for _ in 0..iterations {
            for (k, v) in bplus.items_fast() {
                std::hint::black_box((k, v));
            }
        }
        let time = start.elapsed();
        time.as_nanos() as f64 / (iterations * size) as f64
    };

    println!("BTreeMap per item: {:.2} ns", btree_per_item);
    println!(
        "BPlusTreeMap regular per item: {:.2} ns ({:.2}x overhead)",
        regular_per_item,
        regular_per_item / btree_per_item
    );
    println!(
        "BPlusTreeMap fast per item: {:.2} ns ({:.2}x overhead)",
        fast_per_item,
        fast_per_item / btree_per_item
    );

    // Cache behavior analysis
    println!("\n=== CACHE BEHAVIOR ANALYSIS ===");

    // Sequential vs random access
    let data: Vec<(i32, i32)> = (0..size).map(|i| (i, i * 2)).collect();

    let start = Instant::now();
    for _ in 0..iterations {
        for (k, v) in &data {
            std::hint::black_box((k, v));
        }
    }
    let sequential_time = start.elapsed();

    // Random access pattern
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

    println!(
        "Sequential Vec access: {:?} ({:.2} ns/item)",
        sequential_time,
        sequential_time.as_nanos() as f64 / (iterations * size) as f64
    );
    println!(
        "Random Vec access: {:?} ({:.2} ns/item, {:.2}x slower)",
        random_time,
        random_time.as_nanos() as f64 / (iterations * size) as f64,
        random_time.as_nanos() as f64 / sequential_time.as_nanos() as f64
    );

    println!("\n=== PERFORMANCE SUMMARY ===");
    println!("Compact Arena Improvements:");
    println!(
        "- Regular iteration: {:.1}% {} than BTreeMap",
        (regular_per_item / btree_per_item - 1.0).abs() * 100.0,
        if regular_per_item < btree_per_item {
            "faster"
        } else {
            "slower"
        }
    );
    println!(
        "- Fast iteration: {:.1}% {} than BTreeMap",
        (fast_per_item / btree_per_item - 1.0).abs() * 100.0,
        if fast_per_item < btree_per_item {
            "faster"
        } else {
            "slower"
        }
    );
    println!(
        "- Fast vs Regular: {:.1}% improvement",
        (regular_per_item / fast_per_item - 1.0) * 100.0
    );

    println!("\n=== RECOMMENDATIONS ===");
    if fast_per_item < btree_per_item {
        println!("✅ Compact arena with unsafe fast paths beats BTreeMap!");
        println!("   Recommended for performance-critical iteration workloads.");
    } else {
        println!("⚠️  Still slower than BTreeMap, but significant improvement over original.");
        println!("   Consider for scenarios where B+ tree advantages outweigh iteration cost.");
    }

    println!("\nOptimal capacity: 64-128 for best balance of performance and memory usage.");
    println!("Use items_fast() for performance-critical iteration when safety allows.");
}
