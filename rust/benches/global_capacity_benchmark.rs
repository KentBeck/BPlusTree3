use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use bplustree::{BPlusTreeMap, GlobalCapacityBPlusTreeMap};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

const TREE_CAPACITY: usize = 64;
const SEED: u64 = 42;

fn generate_test_data(size: usize) -> Vec<(i32, String)> {
    let mut rng = StdRng::seed_from_u64(SEED);
    (0..size)
        .map(|_| {
            let key = rng.gen_range(0..size as i32 * 2);
            let value = format!("value_{}", key);
            (key, value)
        })
        .collect()
}

fn bench_insertion_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("insertion_performance");
    group.sample_size(50);
    
    for size in [100, 500, 1000, 5000, 10000].iter() {
        let data = generate_test_data(*size);
        
        // Standard tree insertion
        group.bench_with_input(BenchmarkId::new("standard", size), size, |b, _| {
            b.iter(|| {
                let mut tree = BPlusTreeMap::new(TREE_CAPACITY).unwrap();
                for (key, value) in &data {
                    black_box(tree.insert(*key, value.clone()));
                }
                black_box(tree)
            })
        });
        
        // Global capacity tree insertion
        group.bench_with_input(BenchmarkId::new("global_capacity", size), size, |b, _| {
            b.iter(|| {
                let mut tree = GlobalCapacityBPlusTreeMap::new(TREE_CAPACITY).unwrap();
                for (key, value) in &data {
                    black_box(tree.insert(*key, value.clone()).unwrap());
                }
                black_box(tree)
            })
        });
    }
    group.finish();
}

fn bench_sequential_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_insertion");
    group.sample_size(30);
    
    for size in [1000, 5000, 10000].iter() {
        // Sequential data (worst case for B+ trees)
        let sequential_data: Vec<(i32, String)> = (0..*size)
            .map(|i| (i as i32, format!("value_{}", i)))
            .collect();
        
        group.bench_with_input(BenchmarkId::new("standard_sequential", size), size, |b, _| {
            b.iter(|| {
                let mut tree = BPlusTreeMap::new(TREE_CAPACITY).unwrap();
                for (key, value) in &sequential_data {
                    black_box(tree.insert(*key, value.clone()));
                }
                black_box(tree)
            })
        });
        
        group.bench_with_input(BenchmarkId::new("global_capacity_sequential", size), size, |b, _| {
            b.iter(|| {
                let mut tree = GlobalCapacityBPlusTreeMap::new(TREE_CAPACITY).unwrap();
                for (key, value) in &sequential_data {
                    black_box(tree.insert(*key, value.clone()).unwrap());
                }
                black_box(tree)
            })
        });
    }
    group.finish();
}

fn bench_lookup_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_performance");
    group.sample_size(100);
    
    for size in [100, 1000, 10000, 50000].iter() {
        let data = generate_test_data(*size);
        
        // Prepare trees
        let mut std_tree = BPlusTreeMap::new(TREE_CAPACITY).unwrap();
        let mut gc_tree = GlobalCapacityBPlusTreeMap::new(TREE_CAPACITY).unwrap();
        
        for (key, value) in &data {
            std_tree.insert(*key, value.clone());
            gc_tree.insert(*key, value.clone()).unwrap();
        }

        // Generate lookup keys (mix of existing and non-existing)
        let mut rng = StdRng::seed_from_u64(SEED + 1);
        let lookup_keys: Vec<i32> = (0..1000)
            .map(|_| rng.gen_range(0..*size as i32 * 3))
            .collect();
        
        group.bench_with_input(BenchmarkId::new("standard_lookup", size), size, |b, _| {
            b.iter(|| {
                for key in &lookup_keys {
                    black_box(std_tree.get(key));
                }
            })
        });
        
        group.bench_with_input(BenchmarkId::new("global_capacity_lookup", size), size, |b, _| {
            b.iter(|| {
                for key in &lookup_keys {
                    black_box(gc_tree.get(key));
                }
            })
        });
    }
    group.finish();
}

fn bench_range_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_queries");
    group.sample_size(50);
    
    let size = 10000;
    let data = generate_test_data(size);
    
    // Prepare trees
    let mut std_tree = BPlusTreeMap::new(TREE_CAPACITY).unwrap();
    let mut gc_tree = GlobalCapacityBPlusTreeMap::new(TREE_CAPACITY).unwrap();
    
    for (key, value) in &data {
        std_tree.insert(*key, value.clone());
        gc_tree.insert(*key, value.clone()).unwrap();
    }

    // Test different range sizes
    for range_size in [10, 100, 1000].iter() {
        let start_key = size as i32 / 4;
        let end_key = start_key + *range_size;
        
        group.bench_with_input(BenchmarkId::new("standard_range", range_size), range_size, |b, _| {
            b.iter(|| {
                let mut count = 0;
                for key in start_key..end_key {
                    if std_tree.get(&key).is_some() {
                        count += 1;
                    }
                }
                black_box(count)
            })
        });
        
        group.bench_with_input(BenchmarkId::new("global_capacity_range", range_size), range_size, |b, _| {
            b.iter(|| {
                let mut count = 0;
                for key in start_key..end_key {
                    if gc_tree.get(&key).is_some() {
                        count += 1;
                    }
                }
                black_box(count)
            })
        });
    }
    group.finish();
}

fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    group.sample_size(20);
    
    for size in [1000, 5000, 10000, 25000].iter() {
        let data = generate_test_data(*size);
        
        // Measure standard tree memory allocation patterns
        group.bench_with_input(BenchmarkId::new("standard_memory", size), size, |b, _| {
            b.iter(|| {
                let mut tree = BPlusTreeMap::new(TREE_CAPACITY).unwrap();
                for (key, value) in &data {
                    tree.insert(*key, value.clone());
                }
                
                // Perform operations that stress memory usage
                let mut sum = 0;
                for (key, _) in &data {
                    if let Some(val) = tree.get(key) {
                        sum += val.len();
                    }
                }
                black_box((tree, sum))
            })
        });
        
        // Measure global capacity tree memory allocation patterns
        group.bench_with_input(BenchmarkId::new("global_capacity_memory", size), size, |b, _| {
            b.iter(|| {
                let mut tree = GlobalCapacityBPlusTreeMap::new(TREE_CAPACITY).unwrap();
                for (key, value) in &data {
                    tree.insert(*key, value.clone()).unwrap();
                }
                
                // Perform operations that stress memory usage
                let mut sum = 0;
                for (key, _) in &data {
                    if let Some(val) = tree.get(key) {
                        sum += val.len();
                    }
                }
                black_box((tree, sum))
            })
        });
    }
    group.finish();
}

fn bench_node_splitting_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_splitting_overhead");
    group.sample_size(30);
    
    // Test with small capacity to force frequent splits
    let small_capacity = 8;
    let size = 1000;
    
    // Sequential insertion forces maximum splits
    let sequential_data: Vec<(i32, String)> = (0..size)
        .map(|i| (i as i32, format!("value_{}", i)))
        .collect();
    
    group.bench_function("standard_splitting", |b| {
        b.iter(|| {
            let mut tree = BPlusTreeMap::new(small_capacity).unwrap();
            for (key, value) in &sequential_data {
                black_box(tree.insert(*key, value.clone()));
            }
            black_box(tree)
        })
    });
    
    group.bench_function("global_capacity_splitting", |b| {
        b.iter(|| {
            let mut tree = GlobalCapacityBPlusTreeMap::new(small_capacity).unwrap();
            for (key, value) in &sequential_data {
                black_box(tree.insert(*key, value.clone()).unwrap());
            }
            black_box(tree)
        })
    });

    group.finish();
}

fn bench_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");
    group.sample_size(50);
    
    let size = 10000;
    let data = generate_test_data(size);
    
    // Prepare trees
    let mut std_tree = BPlusTreeMap::new(TREE_CAPACITY).unwrap();
    let mut gc_tree = GlobalCapacityBPlusTreeMap::new(TREE_CAPACITY).unwrap();
    
    for (key, value) in &data {
        std_tree.insert(*key, value.clone());
        gc_tree.insert(*key, value.clone()).unwrap();
    }

    // Generate access pattern that tests cache locality
    let mut rng = StdRng::seed_from_u64(SEED + 2);
    let access_pattern: Vec<i32> = (0..5000)
        .map(|_| {
            // 80% chance of accessing recently accessed keys (cache-friendly)
            // 20% chance of random access (cache-unfriendly)
            if rng.gen::<f32>() < 0.8 {
                rng.gen_range(0..100) // Hot keys
            } else {
                rng.gen_range(0..size as i32 * 2) // Cold keys
            }
        })
        .collect();
    
    group.bench_function("standard_cache_pattern", |b| {
        b.iter(|| {
            for key in &access_pattern {
                black_box(std_tree.get(key));
            }
        })
    });
    
    group.bench_function("global_capacity_cache_pattern", |b| {
        b.iter(|| {
            for key in &access_pattern {
                black_box(gc_tree.get(key));
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_insertion_performance,
    bench_sequential_insertion,
    bench_lookup_performance,
    bench_range_queries,
    bench_memory_efficiency,
    bench_node_splitting_overhead,
    bench_cache_performance
);
criterion_main!(benches);
