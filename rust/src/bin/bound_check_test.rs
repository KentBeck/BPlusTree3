use bplustree::BPlusTreeMap;
use std::time::Instant;

fn main() {
    println!("=== Bound Checking Overhead Test ===\n");

    // Build tree
    let tree_size = 100_000;
    let mut tree = BPlusTreeMap::new(16).unwrap();
    for i in 0..tree_size {
        tree.insert(i, format!("value_{}", i));
    }

    let range_size = 10_000;
    let start_key = tree_size / 2;
    let end_key = start_key + range_size;

    println!(
        "Testing different iteration methods on {} elements:",
        range_size
    );

    // Test 1: Full iteration (no bounds)
    let full_start = Instant::now();
    let full_count = tree.items().count();
    let full_time = full_start.elapsed();
    println!(
        "Full iteration:     {:.2}µs ({:.4}µs per element)",
        full_time.as_micros() as f64,
        full_time.as_micros() as f64 / full_count as f64
    );

    // Test 2: Unbounded range (should be similar to full iteration)
    let unbounded_start = Instant::now();
    let unbounded_count = tree.range(..).count();
    let unbounded_time = unbounded_start.elapsed();
    println!(
        "Unbounded range:    {:.2}µs ({:.4}µs per element)",
        unbounded_time.as_micros() as f64,
        unbounded_time.as_micros() as f64 / unbounded_count as f64
    );

    // Test 3: Bounded range (should show overhead)
    let bounded_start = Instant::now();
    let bounded_count = tree.range(start_key..end_key).count();
    let bounded_time = bounded_start.elapsed();
    println!(
        "Bounded range:      {:.2}µs ({:.4}µs per element)",
        bounded_time.as_micros() as f64,
        bounded_time.as_micros() as f64 / bounded_count as f64
    );

    // Test 4: Very precise range (1 element)
    let precise_start = Instant::now();
    let precise_count = tree.range(start_key..start_key + 1).count();
    let precise_time = precise_start.elapsed();
    println!(
        "Single element:     {:.2}µs ({:.4}µs per element)",
        precise_time.as_micros() as f64,
        precise_time.as_micros() as f64 / precise_count.max(1) as f64
    );

    // Analysis
    let bound_overhead = bounded_time.as_micros() as f64 / unbounded_time.as_micros() as f64;
    println!("\nBound checking overhead: {:.2}x", bound_overhead);

    let startup_cost = precise_time.as_micros() as f64; // Cost for 1 element
    let per_element_cost =
        (bounded_time.as_micros() as f64 - startup_cost) / (bounded_count - 1) as f64;

    println!("Estimated startup cost: {:.2}µs", startup_cost);
    println!("Estimated per-element cost: {:.4}µs", per_element_cost);
}
