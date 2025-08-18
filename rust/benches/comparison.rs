use bplustree::BPlusTreeMap;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::prelude::*;
use std::collections::BTreeMap;

fn bench_sequential_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_insertion");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("BTreeMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = BTreeMap::new();
                for i in 0..size {
                    map.insert(black_box(i), black_box(i * 2));
                }
                map
            });
        });

        group.bench_with_input(BenchmarkId::new("BPlusTreeMap", size), size, |b, &size| {
            b.iter(|| {
                let mut map = BPlusTreeMap::new(16).unwrap(); // Reasonable capacity
                for i in 0..size {
                    map.insert(black_box(i), black_box(i * 2));
                }
                map
            });
        });
    }
    group.finish();
}

fn bench_random_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_insertion");

    for size in [100, 1000, 10000].iter() {
        // Pre-generate random data to ensure fair comparison
        let mut rng = StdRng::seed_from_u64(42);
        let data: Vec<(i32, i32)> = (0..*size)
            .map(|_| (rng.gen_range(0..size * 10), rng.gen_range(0..1000)))
            .collect();

        group.bench_with_input(BenchmarkId::new("BTreeMap", size), &data, |b, data| {
            b.iter(|| {
                let mut map = BTreeMap::new();
                for &(key, value) in data {
                    map.insert(black_box(key), black_box(value));
                }
                map
            });
        });

        group.bench_with_input(BenchmarkId::new("BPlusTreeMap", size), &data, |b, data| {
            b.iter(|| {
                let mut map = BPlusTreeMap::new(16).unwrap();
                for &(key, value) in data {
                    map.insert(black_box(key), black_box(value));
                }
                map
            });
        });
    }
    group.finish();
}

fn bench_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup");

    for size in [100, 1000, 10000].iter() {
        // Pre-populate both data structures
        let mut btree = BTreeMap::new();
        let mut bplus = BPlusTreeMap::new(16).unwrap();

        for i in 0..*size {
            btree.insert(i, i * 2);
            bplus.insert(i, i * 2);
        }

        // Generate lookup keys
        let mut rng = StdRng::seed_from_u64(42);
        let lookup_keys: Vec<i32> = (0..1000).map(|_| rng.gen_range(0..*size)).collect();

        group.bench_with_input(
            BenchmarkId::new("BTreeMap", size),
            &lookup_keys,
            |b, keys| {
                b.iter(|| {
                    for &key in keys {
                        black_box(btree.get(&black_box(key)));
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("BPlusTreeMap", size),
            &lookup_keys,
            |b, keys| {
                b.iter(|| {
                    for &key in keys {
                        black_box(bplus.get(&black_box(key)));
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("iteration");

    for size in [100, 1000, 10000].iter() {
        // Pre-populate both data structures
        let mut btree = BTreeMap::new();
        let mut bplus = BPlusTreeMap::new(16).unwrap();

        for i in 0..*size {
            btree.insert(i, i * 2);
            bplus.insert(i, i * 2);
        }

        group.bench_with_input(BenchmarkId::new("BTreeMap", size), size, |b, _| {
            b.iter(|| {
                for (key, value) in btree.iter() {
                    black_box((key, value));
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("BPlusTreeMap", size), size, |b, _| {
            b.iter(|| {
                for (key, value) in bplus.items() {
                    black_box((key, value));
                }
            });
        });
    }
    group.finish();
}

fn bench_deletion(c: &mut Criterion) {
    let mut group = c.benchmark_group("deletion");

    for size in [100, 1000, 5000].iter() {
        // Smaller sizes for deletion since it's destructive
        group.bench_with_input(BenchmarkId::new("BTreeMap", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut map = BTreeMap::new();
                    for i in 0..size {
                        map.insert(i, i * 2);
                    }
                    map
                },
                |mut map| {
                    for i in 0..size {
                        black_box(map.remove(&black_box(i)));
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("BPlusTreeMap", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut map = BPlusTreeMap::new(16).unwrap();
                    for i in 0..size {
                        map.insert(i, i * 2);
                    }
                    map
                },
                |mut map| {
                    for i in 0..size {
                        black_box(map.remove(&black_box(i)));
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_mixed_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_operations");

    for size in [100, 1000, 5000].iter() {
        // Generate mixed operations
        let mut rng = StdRng::seed_from_u64(42);
        let operations: Vec<(u8, i32, i32)> = (0..*size)
            .map(|_| {
                let op = rng.gen_range(0..3); // 0=insert, 1=lookup, 2=delete
                let key = rng.gen_range(0..*size);
                let value = rng.gen_range(0..1000);
                (op, key, value)
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("BTreeMap", size), &operations, |b, ops| {
            b.iter_batched(
                || BTreeMap::new(),
                |mut map| {
                    for &(op, key, value) in ops {
                        match op {
                            0 => {
                                map.insert(black_box(key), black_box(value));
                            }
                            1 => {
                                black_box(map.get(&black_box(key)));
                            }
                            2 => {
                                black_box(map.remove(&black_box(key)));
                            }
                            _ => unreachable!(),
                        }
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(
            BenchmarkId::new("BPlusTreeMap", size),
            &operations,
            |b, ops| {
                b.iter_batched(
                    || BPlusTreeMap::new(16).unwrap(),
                    |mut map| {
                        for &(op, key, value) in ops {
                            match op {
                                0 => {
                                    map.insert(black_box(key), black_box(value));
                                }
                                1 => {
                                    black_box(map.get(&black_box(key)));
                                }
                                2 => {
                                    black_box(map.remove(&black_box(key)));
                                }
                                _ => unreachable!(),
                            }
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_capacity_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("capacity_optimization");

    let size = 10000;

    for capacity in [4, 8, 16, 32, 64, 128].iter() {
        group.bench_with_input(
            BenchmarkId::new("insertion", capacity),
            capacity,
            |b, &capacity| {
                b.iter(|| {
                    let mut map = BPlusTreeMap::new(capacity).unwrap();
                    for i in 0..size {
                        map.insert(black_box(i), black_box(i * 2));
                    }
                    map
                });
            },
        );
    }

    // Pre-populate trees with different capacities for lookup benchmarks
    let trees: Vec<_> = [4, 8, 16, 32, 64, 128]
        .iter()
        .map(|&capacity| {
            let mut map = BPlusTreeMap::new(capacity).unwrap();
            for i in 0..size {
                map.insert(i, i * 2);
            }
            (capacity, map)
        })
        .collect();

    // Generate lookup keys
    let mut rng = StdRng::seed_from_u64(42);
    let lookup_keys: Vec<i32> = (0..1000).map(|_| rng.gen_range(0..size)).collect();

    for (capacity, tree) in &trees {
        group.bench_with_input(
            BenchmarkId::new("lookup", capacity),
            &lookup_keys,
            |b, keys| {
                b.iter(|| {
                    for &key in keys {
                        black_box(tree.get(&black_box(key)));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_range_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_queries");

    let size = 100000; // Larger dataset to show optimization benefits

    // Pre-populate both data structures
    let mut btree = BTreeMap::new();
    let mut bplus = BPlusTreeMap::new(16).unwrap();

    for i in 0..size {
        btree.insert(i, i * 2);
        bplus.insert(i, i * 2);
    }

    // Test various range sizes to show where optimization shines
    for range_size in [10, 50, 100, 500, 1000, 5000].iter() {
        let start = size / 2 - range_size / 2;
        let end = start + range_size;

        group.bench_with_input(
            BenchmarkId::new("BTreeMap", range_size),
            range_size,
            |b, _| {
                b.iter(|| {
                    for (key, value) in btree.range(black_box(start)..black_box(end)) {
                        black_box((key, value));
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("BPlusTreeMap_Optimized", range_size),
            range_size,
            |b, _| {
                b.iter(|| {
                    for (key, value) in
                        bplus.items_range(Some(&black_box(start)), Some(&black_box(end)))
                    {
                        black_box((key, value));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_range_edge_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_edge_cases");

    let size = 50000;

    // Pre-populate both data structures
    let mut btree = BTreeMap::new();
    let mut bplus = BPlusTreeMap::new(16).unwrap();

    for i in 0..size {
        btree.insert(i, i * 2);
        bplus.insert(i, i * 2);
    }

    // Benchmark: Small range at beginning
    group.bench_function("small_range_start_BTreeMap", |b| {
        b.iter(|| {
            for (key, value) in btree.range(black_box(0)..black_box(10)) {
                black_box((key, value));
            }
        });
    });

    group.bench_function("small_range_start_BPlusTreeMap", |b| {
        b.iter(|| {
            for (key, value) in bplus.items_range(Some(&black_box(0)), Some(&black_box(10))) {
                black_box((key, value));
            }
        });
    });

    // Benchmark: Small range at end
    group.bench_function("small_range_end_BTreeMap", |b| {
        b.iter(|| {
            for (key, value) in btree.range(black_box(size - 10)..black_box(size)) {
                black_box((key, value));
            }
        });
    });

    group.bench_function("small_range_end_BPlusTreeMap", |b| {
        b.iter(|| {
            for (key, value) in
                bplus.items_range(Some(&black_box(size - 10)), Some(&black_box(size)))
            {
                black_box((key, value));
            }
        });
    });

    // Benchmark: Range from middle to end (no end bound)
    group.bench_function("range_to_end_BTreeMap", |b| {
        b.iter(|| {
            for (key, value) in btree.range(black_box(size / 2)..) {
                black_box((key, value));
            }
        });
    });

    group.bench_function("range_to_end_BPlusTreeMap", |b| {
        b.iter(|| {
            for (key, value) in bplus.items_range(Some(&black_box(size / 2)), None) {
                black_box((key, value));
            }
        });
    });

    // Benchmark: Full iteration
    group.bench_function("full_iteration_BTreeMap", |b| {
        b.iter(|| {
            for (key, value) in btree.iter() {
                black_box((key, value));
            }
        });
    });

    group.bench_function("full_iteration_BPlusTreeMap", |b| {
        b.iter(|| {
            for (key, value) in bplus.items() {
                black_box((key, value));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_sequential_insertion,
    bench_random_insertion,
    bench_lookup,
    bench_iteration,
    bench_deletion,
    bench_mixed_operations,
    bench_capacity_optimization,
    bench_range_queries,
    bench_range_edge_cases
);
criterion_main!(benches);
