use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::time::Instant;

// Large-scale delete benchmark comparing BPlusTreeMap vs BTreeMap
// Focus: delete performance with large trees (1M+) and capacity 256
// Note: Run in release mode for meaningful results.
fn main() {
    // Configurable via env vars if needed
    let tree_size: usize = std::env::var("TREE_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1_000_000);
    let capacity: usize = std::env::var("CAPACITY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(256);
    let delete_sample: usize = std::env::var("DELETE_SAMPLE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100_000);

    println!("=== Large Delete Benchmark ===");
    println!(
        "Size: {} elements, Capacity: {} keys/node",
        tree_size, capacity
    );
    println!("Delete sample: {} keys (pseudo-random)", delete_sample);

    // Prepare delete keys (pseudo-random deterministic sequence across range [0, tree_size))
    let delete_keys: Vec<usize> = (0..delete_sample)
        .scan(42_u64, |seed, _| {
            *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            Some((*seed as usize) % tree_size)
        })
        .collect();

    // Build maps
    println!("\nBuilding maps...");
    let mut bplus = BPlusTreeMap::new(capacity).expect("init bplus");
    let mut btree = BTreeMap::new();

    let start = Instant::now();
    for i in 0..tree_size {
        bplus.insert(i, i);
    }
    let bplus_build = start.elapsed();

    let start = Instant::now();
    for i in 0..tree_size {
        btree.insert(i, i);
    }
    let btree_build = start.elapsed();

    println!(
        "Build times: BPlusTreeMap={:?}, BTreeMap={:?}",
        bplus_build, btree_build
    );

    // Clone maps to avoid interaction between runs
    println!("\nDeleting ({} keys)...", delete_sample);

    // BPlusTreeMap delete timing
    let mut bplus_copy = bplus; // move
    let start = Instant::now();
    for &k in &delete_keys {
        let _ = bplus_copy.remove(&k);
    }
    let bplus_delete = start.elapsed();

    // BTreeMap delete timing
    let mut btree_copy = btree; // move
    let start = Instant::now();
    for &k in &delete_keys {
        let _ = btree_copy.remove(&k);
    }
    let btree_delete = start.elapsed();

    let bplus_per_op = (bplus_delete.as_nanos() as f64) / (delete_sample as f64);
    let btree_per_op = (btree_delete.as_nanos() as f64) / (delete_sample as f64);
    let ratio = btree_per_op / bplus_per_op;

    println!("\nDelete times:");
    println!(
        "  BPlusTreeMap: {:?} total ({:.1} ns/op)",
        bplus_delete, bplus_per_op
    );
    println!(
        "  BTreeMap:     {:?} total ({:.1} ns/op)",
        btree_delete, btree_per_op
    );
    println!(
        "  Ratio:        {:.2}x {}",
        ratio,
        if ratio > 1.0 {
            "(BPlusTreeMap faster)"
        } else {
            "(BTreeMap faster)"
        }
    );
}
