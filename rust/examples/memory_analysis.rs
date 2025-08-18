//! Memory usage and cache behavior analysis

use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::hint::black_box;
use std::mem;
use std::time::Instant;

fn measure_operation<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start_time = Instant::now();
    let result = f();
    let duration = start_time.elapsed();

    println!("{}: {:.2}ms", name, duration.as_secs_f64() * 1000.0);
    result
}

fn cache_friendly_access_pattern(data: &[i32]) -> i64 {
    let mut sum = 0i64;
    for &x in data {
        sum += x as i64;
    }
    sum
}

fn cache_unfriendly_access_pattern(data: &[i32]) -> i64 {
    let mut sum = 0i64;
    let len = data.len();
    for i in (0..len).step_by(64) {
        if i < len {
            sum += data[i] as i64;
        }
    }
    sum
}

fn main() {
    println!("🧠 MEMORY USAGE AND CACHE BEHAVIOR ANALYSIS");
    println!("===========================================");

    // 1. BASIC MEMORY OVERHEAD COMPARISON
    println!("\n📊 BASIC MEMORY OVERHEAD");
    println!("{}", "=".repeat(40));

    for &size in &[100, 1000, 10000] {
        println!("\nDataset size: {}", size);

        let btree = measure_operation("BTreeMap creation", || {
            let mut tree = BTreeMap::new();
            for i in 0..size {
                tree.insert(i, i * 2);
            }
            tree
        });

        let bplus = measure_operation("BPlusTreeMap creation", || {
            let mut tree = BPlusTreeMap::new(64).unwrap();
            for i in 0..size {
                tree.insert(i, i * 2);
            }
            tree
        });

        let btree_size = mem::size_of_val(&btree);
        let bplus_size = mem::size_of_val(&bplus);

        println!("  Stack size: BTree {}B, BPlus {}B", btree_size, bplus_size);

        drop(btree);
        drop(bplus);
    }

    // 2. CACHE BEHAVIOR SIMULATION
    println!("\n⚡ CACHE BEHAVIOR SIMULATION");
    println!("{}", "=".repeat(40));

    let dataset: Vec<i32> = (0..100_000).collect();

    let friendly_time = measure_operation("Cache-friendly access", || {
        let start = Instant::now();
        let sum = cache_friendly_access_pattern(&dataset);
        let duration = start.elapsed();
        black_box(sum);
        duration
    });

    let unfriendly_time = measure_operation("Cache-unfriendly access", || {
        let start = Instant::now();
        let sum = cache_unfriendly_access_pattern(&dataset);
        let duration = start.elapsed();
        black_box(sum);
        duration
    });

    println!(
        "Cache impact: {:.2}x slower for unfriendly pattern",
        unfriendly_time.as_secs_f64() / friendly_time.as_secs_f64()
    );

    // 3. ITERATION PATTERNS
    println!("\n🔄 ITERATION PATTERNS");
    println!("{}", "=".repeat(40));

    let size = 10000;
    let btree: BTreeMap<i32, i32> = (0..size).map(|i| (i, i * 2)).collect();
    let mut bplus = BPlusTreeMap::new(64).unwrap();
    for i in 0..size {
        bplus.insert(i, i * 2);
    }

    measure_operation("BTreeMap iteration", || {
        let mut sum = 0i64;
        for (k, v) in btree.iter() {
            sum += (*k as i64) + (*v as i64);
        }
        black_box(sum);
    });

    measure_operation("BPlusTreeMap safe iteration", || {
        let mut sum = 0i64;
        for (k, v) in bplus.items() {
            sum += (*k as i64) + (*v as i64);
        }
        black_box(sum);
    });

    measure_operation("BPlusTreeMap fast iteration", || {
        let mut sum = 0i64;
        for (k, v) in bplus.items_fast() {
            sum += (*k as i64) + (*v as i64);
        }
        black_box(sum);
    });

    println!("\n📊 MEMORY ANALYSIS SUMMARY");
    println!("==================================================");
    println!("• BTreeMap typically has lower memory overhead");
    println!("• BPlusTreeMap requires more initial allocation");
    println!("• Cache behavior depends on access patterns");
    println!("• Both structures show different iteration costs");
}
