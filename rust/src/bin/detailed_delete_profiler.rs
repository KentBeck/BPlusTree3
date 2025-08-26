use bplustree::BPlusTreeMap;
use std::time::{Duration, Instant};

fn main() {
    println!("Detailed Delete Operation Profiler");
    println!("==================================");

    // Run comprehensive delete profiling
    profile_delete_operations_detailed();
}

fn profile_delete_operations_detailed() {
    println!("\nDetailed Delete Analysis");
    println!("========================");

    // Test different tree sizes to understand scaling
    let sizes = vec![1_000, 10_000, 50_000, 100_000];

    for size in sizes {
        println!("\n--- Tree Size: {} elements ---", size);
        profile_tree_size(size);
    }

    // Test different capacities
    println!("\n--- Capacity Analysis ---");
    let capacities = vec![8, 16, 32, 64, 128];

    for capacity in capacities {
        println!("\nCapacity: {}", capacity);
        profile_capacity(capacity);
    }
}

fn profile_tree_size(size: usize) {
    // Helper function to create and populate a tree
    let create_tree = || {
        let mut tree = BPlusTreeMap::new(16).unwrap();
        for i in 0..size {
            tree.insert(i as i32, format!("value_{}", i));
        }
        tree
    };

    let setup_start = Instant::now();
    let _tree = create_tree();
    let setup_time = setup_start.elapsed();

    // Profile different delete patterns
    let delete_count = size / 4; // Delete 25% of elements

    // 1. Sequential deletes from start
    let mut tree1 = create_tree();
    let start = Instant::now();
    for i in 0..delete_count {
        tree1.remove(&(i as i32));
    }
    let sequential_time = start.elapsed();

    // 2. Sequential deletes from end
    let mut tree2 = create_tree();
    let start = Instant::now();
    for i in (size - delete_count)..size {
        tree2.remove(&(i as i32));
    }
    let reverse_time = start.elapsed();

    // 3. Middle deletes (causes most rebalancing)
    let mut tree3 = create_tree();
    let start = Instant::now();
    let middle_start = size / 2 - delete_count / 2;
    for i in middle_start..(middle_start + delete_count) {
        tree3.remove(&(i as i32));
    }
    let middle_time = start.elapsed();

    // 4. Scattered deletes (every nth element)
    let mut tree4 = create_tree();
    let step = size / delete_count;
    let start = Instant::now();
    for i in (0..size).step_by(step).take(delete_count) {
        tree4.remove(&(i as i32));
    }
    let scattered_time = start.elapsed();

    println!("  Setup time: {:?}", setup_time);
    println!(
        "  Sequential (start): {:?} ({:?}/op)",
        sequential_time,
        sequential_time / delete_count as u32
    );
    println!(
        "  Sequential (end):   {:?} ({:?}/op)",
        reverse_time,
        reverse_time / delete_count as u32
    );
    println!(
        "  Middle deletes:     {:?} ({:?}/op)",
        middle_time,
        middle_time / delete_count as u32
    );
    println!(
        "  Scattered deletes:  {:?} ({:?}/op)",
        scattered_time,
        scattered_time / delete_count as u32
    );

    // Analyze which pattern is most expensive
    let times = vec![
        ("Sequential (start)", sequential_time),
        ("Sequential (end)", reverse_time),
        ("Middle", middle_time),
        ("Scattered", scattered_time),
    ];

    let slowest = times.iter().max_by_key(|(_, time)| time).unwrap();
    let fastest = times.iter().min_by_key(|(_, time)| time).unwrap();

    println!("  Slowest: {} ({:?})", slowest.0, slowest.1);
    println!("  Fastest: {} ({:?})", fastest.0, fastest.1);
    println!(
        "  Ratio: {:.2}x",
        slowest.1.as_nanos() as f64 / fastest.1.as_nanos() as f64
    );
}

fn profile_capacity(capacity: usize) {
    let mut tree = BPlusTreeMap::new(capacity).unwrap();
    let size = 50_000;

    // Pre-populate
    for i in 0..size {
        tree.insert(i, format!("value_{}", i));
    }

    // Delete middle section (most rebalancing)
    let delete_count = size / 4;
    let middle_start = size / 2 - delete_count / 2;

    let start = Instant::now();
    for i in middle_start..(middle_start + delete_count) {
        tree.remove(&i);
    }
    let delete_time = start.elapsed();

    println!(
        "  Delete time: {:?} ({:?}/op)",
        delete_time,
        delete_time / delete_count as u32
    );
}
