use bplustree::BPlusTreeMap;
use std::time::Instant;

fn main() {
    println!("=== Micro Range Benchmark ===\n");

    // Build tree
    let tree_size = 100_000;
    let mut tree = BPlusTreeMap::new(16).unwrap();
    for i in 0..tree_size {
        tree.insert(i as i32, format!("value_{}", i));
    }

    println!("Built tree with {} elements\n", tree_size);

    // Measure a batch of operations to get accurate timing
    let iterations = 10_000;
    let start_key = 50_000;

    println!("Testing {} iterations:", iterations);

    // Test 1: Batch lookup operations
    let lookup_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000); // Vary the key slightly
        let _result = tree.get(&key);
    }
    let lookup_time = lookup_start.elapsed();
    println!(
        "Batch lookups:      {:.2}µs total ({:.3}µs per lookup)",
        lookup_time.as_micros() as f64,
        lookup_time.as_micros() as f64 / iterations as f64
    );

    // Test 2: Batch range creation (no iteration)
    let range_create_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000);
        let _iter = tree.range(key..key + 1);
        // Don't consume iterator, just create it
    }
    let range_create_time = range_create_start.elapsed();
    println!(
        "Batch range create: {:.2}µs total ({:.3}µs per range)",
        range_create_time.as_micros() as f64,
        range_create_time.as_micros() as f64 / iterations as f64
    );

    // Test 3: Batch range + consume one element
    let range_next_start = Instant::now();
    for i in 0..iterations {
        let key = start_key + (i % 1000);
        let _first = tree.range(key..key + 1).next();
    }
    let range_next_time = range_next_start.elapsed();
    println!(
        "Batch range + next: {:.2}µs total ({:.3}µs per operation)",
        range_next_time.as_micros() as f64,
        range_next_time.as_micros() as f64 / iterations as f64
    );

    // Test 4: Batch range + count (consume all)
    let range_count_start = Instant::now();
    for i in 0..100 {
        // Fewer iterations since count() is expensive
        let key = start_key + (i % 100) * 10;
        let _count = tree.range(key..key + 5).count();
    }
    let range_count_time = range_count_start.elapsed();
    println!(
        "Batch range + count:{:.2}µs total ({:.2}µs per count)",
        range_count_time.as_micros() as f64,
        range_count_time.as_micros() as f64 / 100.0
    );

    println!("\n=== Analysis ===");
    let range_create_overhead = (range_create_time.as_micros() as f64 / iterations as f64)
        / (lookup_time.as_micros() as f64 / iterations as f64);
    println!(
        "Range creation overhead vs lookup: {:.1}x",
        range_create_overhead
    );

    let range_next_overhead = (range_next_time.as_micros() as f64 / iterations as f64)
        / (lookup_time.as_micros() as f64 / iterations as f64);
    println!(
        "Range + next overhead vs lookup:   {:.1}x",
        range_next_overhead
    );
}
