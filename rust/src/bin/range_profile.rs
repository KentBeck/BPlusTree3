use bplustree::BPlusTreeMap;
use std::time::Instant;

fn main() {
    println!("=== Range Operation Performance Deep Dive ===\n");

    // Test with large tree
    let tree_size = 500_000;
    println!("Building tree with {} elements...", tree_size);

    let start_time = Instant::now();
    let mut tree = BPlusTreeMap::new(16).unwrap();
    for i in 0..tree_size {
        tree.insert(i as i32, format!("value_{}", i));
    }
    println!("Tree built in {:.2}s\n", start_time.elapsed().as_secs_f64());

    // Test different range sizes to understand the cost structure
    test_range_sizes(&tree, tree_size);

    // Test different range positions
    test_range_positions(&tree, tree_size);

    // Test the overhead of range vs direct iteration
    test_range_vs_iteration_overhead(&tree, tree_size);

    // Test iterator creation vs iteration cost
    test_iterator_creation_cost(&tree, tree_size);
}

fn test_range_sizes(tree: &BPlusTreeMap<i32, String>, tree_size: usize) {
    println!("=== Testing Different Range Sizes ===");

    let range_sizes = [1, 10, 100, 1000, 10000, 50000];
    let start_key = (tree_size / 2) as i32;

    for &range_size in &range_sizes {
        let end_key = start_key + range_size;

        // Time the range operation
        let range_start = Instant::now();
        let count = tree.range(start_key..end_key).count();
        let range_time = range_start.elapsed();

        println!(
            "Range size {:6}: {:4} elements in {:8.2}µs ({:.3}µs per element)",
            range_size,
            count,
            range_time.as_micros() as f64,
            range_time.as_micros() as f64 / count as f64
        );
    }
    println!();
}

fn test_range_positions(tree: &BPlusTreeMap<i32, String>, tree_size: usize) {
    println!("=== Testing Range Positions (1000 element ranges) ===");

    let range_size = 1000;
    let positions = [
        ("Start", 0),
        ("25%", tree_size / 4),
        ("50%", tree_size / 2),
        ("75%", 3 * tree_size / 4),
        ("End", tree_size - range_size - 1),
    ];

    for (label, start_pos) in &positions {
        let start_key = *start_pos as i32;
        let end_key = start_key + range_size as i32;

        let range_start = Instant::now();
        let count = tree.range(start_key..end_key).count();
        let range_time = range_start.elapsed();

        println!(
            "{:5} position: {:4} elements in {:8.2}µs ({:.3}µs per element)",
            label,
            count,
            range_time.as_micros() as f64,
            range_time.as_micros() as f64 / count.max(1) as f64
        );
    }
    println!();
}

fn test_range_vs_iteration_overhead(tree: &BPlusTreeMap<i32, String>, _tree_size: usize) {
    println!("=== Range vs Full Iteration Overhead ===");

    // Test full iteration performance
    let iter_start = Instant::now();
    let full_count = tree.items().count();
    let iter_time = iter_start.elapsed();

    println!(
        "Full iteration: {} elements in {:.2}ms ({:.3}µs per element)",
        full_count,
        iter_time.as_millis(),
        iter_time.as_micros() as f64 / full_count as f64
    );

    // Test equivalent range operation (full range)
    let range_start = Instant::now();
    let range_count = tree.range(..).count();
    let range_time = range_start.elapsed();

    println!(
        "Full range:     {} elements in {:.2}ms ({:.3}µs per element)",
        range_count,
        range_time.as_millis(),
        range_time.as_micros() as f64 / range_count as f64
    );

    let overhead_ratio = range_time.as_micros() as f64 / iter_time.as_micros() as f64;
    println!(
        "Range overhead: {:.2}x slower than direct iteration\n",
        overhead_ratio
    );
}

fn test_iterator_creation_cost(tree: &BPlusTreeMap<i32, String>, tree_size: usize) {
    println!("=== Iterator Creation vs Iteration Cost ===");

    let start_key = (tree_size / 2) as i32;
    let end_key = start_key + 1000;

    // Test just iterator creation (no iteration)
    let create_start = Instant::now();
    let _iter = tree.range(start_key..end_key);
    let create_time = create_start.elapsed();

    println!("Iterator creation: {:.2}µs", create_time.as_micros() as f64);

    // Test iterator creation + first element
    let first_start = Instant::now();
    let _first_element = tree.range(start_key..end_key).next();
    let first_time = first_start.elapsed();

    println!(
        "Creation + first():  {:.2}µs",
        first_time.as_micros() as f64
    );

    // Test full iteration
    let full_start = Instant::now();
    let count = tree.range(start_key..end_key).count();
    let full_time = full_start.elapsed();

    println!(
        "Creation + count():  {:.2}µs ({} elements)",
        full_time.as_micros() as f64,
        count
    );

    let iteration_cost = full_time.as_micros() as f64 - create_time.as_micros() as f64;
    println!(
        "Pure iteration cost: {:.2}µs ({:.3}µs per element)",
        iteration_cost,
        iteration_cost / count as f64
    );

    // Break down the costs
    println!("\n=== Cost Breakdown ===");
    println!(
        "Iterator creation: {:.1}%",
        (create_time.as_micros() as f64 / full_time.as_micros() as f64) * 100.0
    );
    println!(
        "Element iteration: {:.1}%",
        (iteration_cost / full_time.as_micros() as f64) * 100.0
    );
}
