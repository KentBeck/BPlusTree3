use bplustree::BPlusTreeMap;
use std::time::Instant;

fn main() {
    println!("=== BPlusTree Function-Level Performance Analysis ===\n");

    // Test with large tree (500k elements)
    let tree_size = 500_000;
    let operations_count = 50_000;

    println!("Tree size: {} elements", tree_size);
    println!(
        "Operations count: {} per operation type\n",
        operations_count
    );

    profile_large_tree_operations(tree_size, operations_count);
}

fn profile_large_tree_operations(tree_size: usize, operations_count: usize) {
    // Simple LCG for deterministic random numbers
    let mut rng_state = 42u64;

    println!("=== Phase 1: Initial Tree Population ===");
    let start_time = Instant::now();
    let mut tree = BPlusTreeMap::new(16).unwrap();

    for i in 0..tree_size {
        tree.insert(i as i32, format!("initial_value_{}", i));
        if i % 100_000 == 0 && i > 0 {
            println!(
                "Inserted {} elements... ({:.2}s)",
                i,
                start_time.elapsed().as_secs_f64()
            );
        }
    }

    let population_time = start_time.elapsed();
    println!(
        "Initial population completed: {:.2}s",
        population_time.as_secs_f64()
    );
    println!(
        "Average insertion time: {:.2}µs\n",
        population_time.as_micros() as f64 / tree_size as f64
    );

    // Profile lookup operations
    println!("=== Phase 2: Lookup Operations ===");
    let lookup_keys: Vec<i32> = (0..operations_count)
        .map(|_| {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            (rng_state % tree_size as u64) as i32
        })
        .collect();

    let lookup_start = Instant::now();
    for (i, key) in lookup_keys.iter().enumerate() {
        let _result = tree.get(key);
        if i % 10_000 == 0 && i > 0 {
            println!(
                "Completed {} lookups... ({:.2}s)",
                i,
                lookup_start.elapsed().as_secs_f64()
            );
        }
    }
    let lookup_time = lookup_start.elapsed();
    println!(
        "Lookup operations completed: {:.2}s",
        lookup_time.as_secs_f64()
    );
    println!(
        "Average lookup time: {:.2}µs\n",
        lookup_time.as_micros() as f64 / operations_count as f64
    );

    // Profile insertion operations (new keys)
    println!("=== Phase 3: Insert Operations ===");
    let insert_keys: Vec<i32> = (0..operations_count)
        .map(|i| (tree_size as i32 + i as i32 + 1000000))
        .collect();

    let insert_start = Instant::now();
    for (i, key) in insert_keys.iter().enumerate() {
        tree.insert(*key, format!("new_value_{}", key));
        if i % 10_000 == 0 && i > 0 {
            println!(
                "Completed {} insertions... ({:.2}s)",
                i,
                insert_start.elapsed().as_secs_f64()
            );
        }
    }
    let insert_time = insert_start.elapsed();
    println!(
        "Insert operations completed: {:.2}s",
        insert_time.as_secs_f64()
    );
    println!(
        "Average insert time: {:.2}µs\n",
        insert_time.as_micros() as f64 / operations_count as f64
    );

    // Profile deletion operations
    println!("=== Phase 4: Delete Operations ===");
    let delete_keys: Vec<i32> = (0..operations_count)
        .map(|_| {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            (rng_state % tree_size as u64) as i32
        })
        .collect();

    let delete_start = Instant::now();
    for (i, key) in delete_keys.iter().enumerate() {
        let _result = tree.remove(key);
        if i % 10_000 == 0 && i > 0 {
            println!(
                "Completed {} deletions... ({:.2}s)",
                i,
                delete_start.elapsed().as_secs_f64()
            );
        }
    }
    let delete_time = delete_start.elapsed();
    println!(
        "Delete operations completed: {:.2}s",
        delete_time.as_secs_f64()
    );
    println!(
        "Average delete time: {:.2}µs\n",
        delete_time.as_micros() as f64 / operations_count as f64
    );

    // Profile range operations
    println!("=== Phase 5: Range Operations ===");
    let range_start = Instant::now();
    let mut total_elements = 0;

    for i in 0..1000 {
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let start_key = (rng_state % (tree_size as u64 - 1000)) as i32;
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let end_key = start_key + ((rng_state % 900) + 100) as i32;

        let count = tree.range(start_key..end_key).count();
        total_elements += count;

        if i % 100 == 0 && i > 0 {
            println!(
                "Completed {} range queries... ({:.2}s)",
                i,
                range_start.elapsed().as_secs_f64()
            );
        }
    }
    let range_time = range_start.elapsed();
    println!(
        "Range operations completed: {:.2}s",
        range_time.as_secs_f64()
    );
    println!(
        "Average range query time: {:.2}µs",
        range_time.as_micros() as f64 / 1000.0
    );
    println!("Total elements in ranges: {}\n", total_elements);

    // Profile mixed workload
    println!("=== Phase 6: Mixed Workload ===");
    let mixed_operations = generate_mixed_operations(operations_count);

    let mixed_start = Instant::now();
    let mut insert_count = 0;
    let mut lookup_count = 0;
    let mut delete_count = 0;

    for (i, op) in mixed_operations.iter().enumerate() {
        match op {
            Operation::Insert(key, value) => {
                tree.insert(*key, value.clone());
                insert_count += 1;
            }
            Operation::Lookup(key) => {
                let _result = tree.get(key);
                lookup_count += 1;
            }
            Operation::Delete(key) => {
                let _result = tree.remove(key);
                delete_count += 1;
            }
        }

        if i % 10_000 == 0 && i > 0 {
            println!(
                "Completed {} mixed operations... ({:.2}s)",
                i,
                mixed_start.elapsed().as_secs_f64()
            );
        }
    }
    let mixed_time = mixed_start.elapsed();
    println!("Mixed workload completed: {:.2}s", mixed_time.as_secs_f64());
    println!(
        "Operations breakdown: {} inserts, {} lookups, {} deletes",
        insert_count, lookup_count, delete_count
    );
    println!(
        "Average mixed operation time: {:.2}µs\n",
        mixed_time.as_micros() as f64 / operations_count as f64
    );

    // Final summary
    println!("=== Performance Summary ===");
    println!(
        "Initial population: {:.2}s ({:.2}µs per insert)",
        population_time.as_secs_f64(),
        population_time.as_micros() as f64 / tree_size as f64
    );
    println!(
        "Lookup operations: {:.2}s ({:.2}µs per lookup)",
        lookup_time.as_secs_f64(),
        lookup_time.as_micros() as f64 / operations_count as f64
    );
    println!(
        "Insert operations: {:.2}s ({:.2}µs per insert)",
        insert_time.as_secs_f64(),
        insert_time.as_micros() as f64 / operations_count as f64
    );
    println!(
        "Delete operations: {:.2}s ({:.2}µs per delete)",
        delete_time.as_secs_f64(),
        delete_time.as_micros() as f64 / operations_count as f64
    );
    println!(
        "Range operations: {:.2}s ({:.2}µs per range)",
        range_time.as_secs_f64(),
        range_time.as_micros() as f64 / 1000.0
    );
    println!(
        "Mixed workload: {:.2}s ({:.2}µs per operation)",
        mixed_time.as_secs_f64(),
        mixed_time.as_micros() as f64 / operations_count as f64
    );

    let total_time =
        population_time + lookup_time + insert_time + delete_time + range_time + mixed_time;
    println!("Total execution time: {:.2}s", total_time.as_secs_f64());

    // Relative performance breakdown
    println!("\n=== Time Distribution ===");
    println!(
        "Initial population: {:.1}%",
        (population_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0
    );
    println!(
        "Lookup operations: {:.1}%",
        (lookup_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0
    );
    println!(
        "Insert operations: {:.1}%",
        (insert_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0
    );
    println!(
        "Delete operations: {:.1}%",
        (delete_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0
    );
    println!(
        "Range operations: {:.1}%",
        (range_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0
    );
    println!(
        "Mixed workload: {:.1}%",
        (mixed_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0
    );
}

#[derive(Clone, Debug)]
enum Operation {
    Insert(i32, String),
    Lookup(i32),
    Delete(i32),
}

fn generate_mixed_operations(count: usize) -> Vec<Operation> {
    let mut rng_state = 42u64;
    let mut operations = Vec::with_capacity(count);

    for _ in 0..count {
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let op_type = rng_state % 100;
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let key = (rng_state % 1000000) as i32;

        let operation = match op_type {
            0..=49 => Operation::Lookup(key), // 50% lookups
            50..=79 => Operation::Insert(key, format!("mixed_value_{}", key)), // 30% inserts
            80..=99 => Operation::Delete(key), // 20% deletes
            _ => unreachable!(),
        };

        operations.push(operation);
    }

    operations
}
