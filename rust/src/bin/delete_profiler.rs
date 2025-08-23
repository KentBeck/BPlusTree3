use bplustree::BPlusTreeMap;
use std::time::Instant;

fn main() {
    println!("Delete Operation Profiler");
    println!("========================");

    // Test different delete patterns
    profile_sequential_deletes();
    profile_pseudo_random_deletes();
    profile_mixed_workload_deletes();
    profile_rebalancing_heavy_deletes();
}

fn profile_sequential_deletes() {
    println!("\n1. Sequential Delete Pattern");
    println!("---------------------------");

    let mut tree = BPlusTreeMap::new(16).unwrap();

    // Pre-populate with 100k elements
    let start = Instant::now();
    for i in 0..100_000 {
        tree.insert(i, format!("value_{}", i));
    }
    println!("Setup time: {:?}", start.elapsed());

    // Delete first half sequentially
    let start = Instant::now();
    for i in 0..50_000 {
        tree.remove(&i);
    }
    let delete_time = start.elapsed();
    println!("Sequential delete time: {:?}", delete_time);
    println!("Avg per delete: {:?}", delete_time / 50_000);
}

fn profile_pseudo_random_deletes() {
    println!("\n2. Pseudo-Random Delete Pattern");
    println!("--------------------------------");

    let mut tree = BPlusTreeMap::new(16).unwrap();

    // Pre-populate with 100k elements
    for i in 0..100_000 {
        tree.insert(i, format!("value_{}", i));
    }

    // Generate pseudo-random delete sequence using simple PRNG
    let mut keys = Vec::new();
    let mut seed = 42u64;
    for _ in 0..50_000 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let key = (seed % 100_000) as i32;
        keys.push(key);
    }

    // Delete using pseudo-random sequence
    let start = Instant::now();
    for key in keys {
        tree.remove(&key);
    }
    let delete_time = start.elapsed();
    println!("Pseudo-random delete time: {:?}", delete_time);
    println!("Avg per delete: {:?}", delete_time / 50_000);
}

fn profile_mixed_workload_deletes() {
    println!("\n3. Mixed Workload with Deletes");
    println!("------------------------------");

    let mut tree = BPlusTreeMap::new(16).unwrap();
    let mut seed = 42u64;

    // Initial population
    for i in 0..50_000 {
        tree.insert(i, format!("value_{}", i));
    }

    let start = Instant::now();
    let mut delete_count = 0;
    let mut insert_count = 0;
    let mut lookup_count = 0;

    // Mixed operations: 40% lookup, 30% insert, 30% delete
    for _ in 0..100_000 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let op = seed % 100;
        let key = (seed % 100_000) as i32;

        match op {
            0..=39 => {
                tree.get(&key);
                lookup_count += 1;
            }
            40..=69 => {
                tree.insert(key, format!("new_value_{}", key));
                insert_count += 1;
            }
            70..=99 => {
                tree.remove(&key);
                delete_count += 1;
            }
            _ => unreachable!(),
        }
    }

    let total_time = start.elapsed();
    println!("Mixed workload time: {:?}", total_time);
    println!(
        "Operations: {} lookups, {} inserts, {} deletes",
        lookup_count, insert_count, delete_count
    );
    if delete_count > 0 {
        println!("Avg delete time: {:?}", total_time / (delete_count as u32));
    }
}

fn profile_rebalancing_heavy_deletes() {
    println!("\n4. Rebalancing-Heavy Delete Pattern");
    println!("-----------------------------------");

    let mut tree = BPlusTreeMap::new(16).unwrap();

    // Create a tree that will require heavy rebalancing
    // Insert in a pattern that creates many small nodes
    for i in 0..100_000 {
        tree.insert(i * 2, format!("value_{}", i * 2));
    }

    // Now delete every other element to force rebalancing
    let start = Instant::now();
    for i in 0..50_000 {
        tree.remove(&(i * 4)); // Delete every 4th original element
    }
    let delete_time = start.elapsed();

    println!("Rebalancing-heavy delete time: {:?}", delete_time);
    println!("Avg per delete: {:?}", delete_time / 50_000);
    println!("Tree size after deletes: {}", tree.len());
}
