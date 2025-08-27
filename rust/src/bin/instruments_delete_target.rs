use bplustree::BPlusTreeMap;
use std::time::{Duration, Instant};

// A long-running delete-focused workload for Instruments Time Profiler.
// It builds a large tree at a specified capacity, then repeatedly deletes a
// pseudo-random batch of keys and reinserts them to keep the workload steady.
// Configure via env vars: CAPACITY, TREE_SIZE, BATCH_SIZE, DURATION_SEC.
fn main() {
    let capacity: usize = std::env::var("CAPACITY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(256);
    let tree_size: usize = std::env::var("TREE_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2_000_000);
    let batch_size: usize = std::env::var("BATCH_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(500_000);
    let duration_sec: u64 = std::env::var("DURATION_SEC")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(15);

    eprintln!(
        "instruments_delete_target: cap={}, size={}, batch={}, duration={}s",
        capacity, tree_size, batch_size, duration_sec
    );

    // Build initial tree
    let mut tree = BPlusTreeMap::new(capacity).expect("init B+tree");
    for i in 0..tree_size {
        // small values to reduce memory
        tree.insert(i as i32, i as i32);
    }

    // Prepare a pseudo-random but deterministic batch of keys
    let mut keys: Vec<i32> = Vec::with_capacity(batch_size);
    let mut seed = 42_u64;
    for _ in 0..batch_size {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let k = (seed as usize) % tree_size;
        keys.push(k as i32);
    }

    // Run mixed cycles of deletes and reinserts until duration elapses
    let deadline = Instant::now() + Duration::from_secs(duration_sec);
    let mut cycles: u64 = 0;
    while Instant::now() < deadline {
        // Delete phase
        for &k in &keys {
            let _ = tree.remove(&k);
        }
        // Reinsert phase to keep tree size stable
        for &k in &keys {
            tree.insert(k, k);
        }
        cycles += 1;
    }

    eprintln!("completed cycles: {} (cap={}, size={})", cycles, capacity, tree_size);
}

