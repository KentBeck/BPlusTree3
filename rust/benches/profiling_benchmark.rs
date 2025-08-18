use bplustree::BPlusTreeMap;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::prelude::*;

/// Profiling benchmark for balanced workload analysis
/// This benchmark creates a realistic workload with mixed operations
/// to identify performance bottlenecks by function and operation type.

fn profile_balanced_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("balanced_workload_profiling");

    // Realistic workload: 50% lookups, 30% inserts, 20% deletes
    let operations = generate_balanced_operations(50000);

    group.bench_function("mixed_operations_profile", |b| {
        b.iter(|| {
            let mut tree = BPlusTreeMap::new(16).unwrap();

            // Initial population to ensure deletions have targets - start with 100k elements
            for i in 0..100000 {
                tree.insert(i, format!("initial_value_{}", i));
            }

            // Execute mixed operations
            for op in &operations {
                match op {
                    Operation::Insert(key, value) => {
                        black_box(tree.insert(black_box(*key), black_box(value.clone())));
                    }
                    Operation::Lookup(key) => {
                        black_box(tree.get(&black_box(*key)));
                    }
                    Operation::Delete(key) => {
                        black_box(tree.remove(&black_box(*key)));
                    }
                }
            }

            tree
        });
    });

    group.finish();
}

fn profile_individual_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("operation_profiling");

    // Profile each operation type separately to understand relative costs

    // Profile insertions on large trees
    group.bench_function("insertion_only_profile", |b| {
        b.iter(|| {
            let mut tree = BPlusTreeMap::new(16).unwrap();
            for i in 0..200000 {
                tree.insert(black_box(i), black_box(format!("value_{}", i)));
            }
            tree
        });
    });

    // Profile lookups on large trees
    group.bench_function("lookup_only_profile", |b| {
        // Pre-populate tree with 500k elements
        let mut tree = BPlusTreeMap::new(16).unwrap();
        for i in 0..500000 {
            tree.insert(i, format!("value_{}", i));
        }

        // Generate random lookup keys
        let mut rng = StdRng::seed_from_u64(42);
        let lookup_keys: Vec<i32> = (0..100000).map(|_| rng.gen_range(0..500000)).collect();

        b.iter(|| {
            for key in &lookup_keys {
                black_box(tree.get(&black_box(*key)));
            }
        });
    });

    // Profile deletions on large trees
    group.bench_function("deletion_only_profile", |b| {
        b.iter_batched(
            || {
                let mut tree = BPlusTreeMap::new(16).unwrap();
                for i in 0..300000 {
                    tree.insert(i, format!("value_{}", i));
                }
                tree
            },
            |mut tree| {
                for i in 0..100000 {
                    black_box(tree.remove(&black_box(i)));
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn profile_tree_operations_breakdown(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_operations_breakdown");

    // Profile different tree operation patterns

    // Sequential access pattern
    group.bench_function("sequential_access_profile", |b| {
        b.iter(|| {
            let mut tree = BPlusTreeMap::new(16).unwrap();

            // Sequential insertions - scale to large tree
            for i in 0..100000 {
                tree.insert(black_box(i), black_box(format!("seq_value_{}", i)));
            }

            // Sequential lookups
            for i in 0..100000 {
                black_box(tree.get(&black_box(i)));
            }

            // Sequential deletions
            for i in 0..50000 {
                black_box(tree.remove(&black_box(i)));
            }

            tree
        });
    });

    // Random access pattern
    group.bench_function("random_access_profile", |b| {
        b.iter(|| {
            let mut tree = BPlusTreeMap::new(16).unwrap();
            let mut rng = StdRng::seed_from_u64(42);

            // Random insertions - scale to large tree
            for _ in 0..100000 {
                let key = rng.gen_range(0..1000000);
                tree.insert(black_box(key), black_box(format!("rand_value_{}", key)));
            }

            // Random lookups
            for _ in 0..100000 {
                let key = rng.gen_range(0..1000000);
                black_box(tree.get(&black_box(key)));
            }

            // Random deletions
            for _ in 0..50000 {
                let key = rng.gen_range(0..1000000);
                black_box(tree.remove(&black_box(key)));
            }

            tree
        });
    });

    group.finish();
}

fn profile_range_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_operations_profile");

    // Profile range queries which are a key BPlusTree advantage
    group.bench_function("range_query_profile", |b| {
        // Pre-populate tree with 1M elements
        let mut tree = BPlusTreeMap::new(16).unwrap();
        for i in 0..1000000 {
            tree.insert(i, format!("range_value_{}", i));
        }

        b.iter(|| {
            // Various range sizes to stress different code paths
            for start in (0..900000).step_by(100000) {
                for range_size in [100, 1000, 10000].iter() {
                    let end = start + range_size;
                    let _count: usize = tree.range(black_box(start)..black_box(end)).count();
                }
            }
        });
    });

    group.finish();
}

fn profile_memory_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation_profile");

    // Profile arena allocation patterns
    group.bench_function("arena_allocation_profile", |b| {
        b.iter(|| {
            let mut tree = BPlusTreeMap::new(16).unwrap();

            // Pattern that causes many node splits and merges
            // This will stress the arena allocation system on large trees
            for i in 0..200000 {
                tree.insert(black_box(i), black_box(format!("alloc_value_{}", i)));
            }

            // Delete every other element to cause fragmentation
            for i in (0..200000).step_by(2) {
                tree.remove(&black_box(i));
            }

            // Re-insert to test arena reuse
            for i in (0..200000).step_by(2) {
                tree.insert(
                    black_box(i + 1000000),
                    black_box(format!("realloc_value_{}", i)),
                );
            }

            tree
        });
    });

    group.finish();
}

#[derive(Clone, Debug)]
enum Operation {
    Insert(i32, String),
    Lookup(i32),
    Delete(i32),
}

fn generate_balanced_operations(count: usize) -> Vec<Operation> {
    let mut rng = StdRng::seed_from_u64(42);
    let mut operations = Vec::with_capacity(count);

    for _ in 0..count {
        let op_type = rng.gen_range(0..100);
        let key = rng.gen_range(0..1000000);

        let operation = match op_type {
            0..=49 => Operation::Lookup(key), // 50% lookups
            50..=79 => Operation::Insert(key, format!("value_{}", key)), // 30% inserts
            80..=99 => Operation::Delete(key), // 20% deletes
            _ => unreachable!(),
        };

        operations.push(operation);
    }

    operations
}

criterion_group!(
    benches,
    profile_balanced_workload,
    profile_individual_operations,
    profile_tree_operations_breakdown,
    profile_range_operations,
    profile_memory_allocation_patterns
);
criterion_main!(benches);
