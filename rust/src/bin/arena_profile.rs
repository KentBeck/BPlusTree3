use bplustree::BPlusTreeMap;
use std::time::Instant;

fn main() {
    println!("=== Arena Access Performance Profile ===\n");

    // Build tree
    let tree_size = 500_000;
    let mut tree = BPlusTreeMap::new(16).unwrap();
    for i in 0..tree_size {
        tree.insert(i as i32, format!("value_{}", i));
    }

    println!("Built tree with {} elements\n", tree_size);

    // Test single operation costs
    test_single_operations(&tree);

    // Test arena access patterns
    test_arena_lookups(&tree);
}

fn test_single_operations(tree: &BPlusTreeMap<i32, String>) {
    println!("=== Single Operation Costs ===");

    let key = 250_000; // Middle of tree

    // Test single lookup
    let lookup_start = Instant::now();
    let _result = tree.get(&key);
    let lookup_time = lookup_start.elapsed();
    println!(
        "Single lookup:      {:.2}µs",
        lookup_time.as_micros() as f64
    );

    // Test single contains check (similar tree traversal to insert)
    let contains_start = Instant::now();
    let _exists = tree.contains_key(&(key + 1_000_000));
    let contains_time = contains_start.elapsed();
    println!(
        "Single contains:    {:.2}µs",
        contains_time.as_micros() as f64
    );

    // Test single range creation (no iteration)
    let range_create_start = Instant::now();
    let _range_iter = tree.range(key..key + 1);
    let range_create_time = range_create_start.elapsed();
    println!(
        "Range creation:     {:.2}µs",
        range_create_time.as_micros() as f64
    );

    // Test range creation + first element
    let range_first_start = Instant::now();
    let _first = tree.range(key..key + 1).next();
    let range_first_time = range_first_start.elapsed();
    println!(
        "Range + first():    {:.2}µs",
        range_first_time.as_micros() as f64
    );

    println!();
}

fn test_arena_lookups(tree: &BPlusTreeMap<i32, String>) {
    println!("=== Arena Lookup Pattern Analysis ===");

    // Test repeated lookups (should show arena efficiency)
    let keys = [100_000, 200_000, 300_000, 400_000];

    let repeated_start = Instant::now();
    for _ in 0..1000 {
        for &key in &keys {
            let _result = tree.get(&key);
        }
    }
    let repeated_time = repeated_start.elapsed();

    println!(
        "4000 lookups:       {:.2}µs ({:.3}µs per lookup)",
        repeated_time.as_micros() as f64,
        repeated_time.as_micros() as f64 / 4000.0
    );

    // Test range creation pattern
    let range_pattern_start = Instant::now();
    for &key in &keys {
        let _iter = tree.range(key..key + 10);
    }
    let range_pattern_time = range_pattern_start.elapsed();

    println!(
        "4 range creations:  {:.2}µs ({:.2}µs per range)",
        range_pattern_time.as_micros() as f64,
        range_pattern_time.as_micros() as f64 / 4.0
    );

    // Test if tree traversal is the issue
    let traversal_start = Instant::now();
    for &key in &keys {
        // This should follow the same path as range creation
        let _result = tree.get(&key);
    }
    let traversal_time = traversal_start.elapsed();

    println!(
        "4 tree traversals:  {:.2}µs ({:.2}µs per traversal)",
        traversal_time.as_micros() as f64,
        traversal_time.as_micros() as f64 / 4.0
    );

    let range_overhead =
        (range_pattern_time.as_micros() as f64 / 4.0) / (traversal_time.as_micros() as f64 / 4.0);
    println!("Range overhead vs lookup: {:.1}x", range_overhead);
}
