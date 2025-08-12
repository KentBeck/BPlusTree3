use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use bplustree::BPlusTreeMap;
use rand::prelude::*;

/// Specialized profiling benchmark for large range scans on very large trees.
/// This benchmark is designed to work with gprof and other profilers to identify
/// performance bottlenecks in range query operations.

fn profile_large_range_scans(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_range_scans");
    
    // Test different tree sizes to see how range scan performance scales
    let tree_sizes = vec![100_000, 500_000, 1_000_000, 2_000_000];
    let range_sizes = vec![100, 1_000, 10_000, 50_000];
    
    for &tree_size in &tree_sizes {
        for &range_size in &range_sizes {
            // Skip combinations that would scan most of the tree
            if range_size > tree_size / 10 {
                continue;
            }
            
            group.bench_with_input(
                BenchmarkId::new("sequential_range_scan", format!("tree_{}_range_{}", tree_size, range_size)),
                &(tree_size, range_size),
                |b, &(tree_size, range_size)| {
                    // Pre-populate tree with sequential keys
                    let mut tree = BPlusTreeMap::new(64).unwrap(); // Use larger capacity for better performance
                    for i in 0..tree_size {
                        tree.insert(i, format!("value_{}", i));
                    }
                    
                    b.iter(|| {
                        // Perform multiple range scans across different parts of the tree
                        let mut total_items = 0;
                        let step = (tree_size - range_size) / 10; // 10 different range positions
                        
                        for start in (0..tree_size - range_size).step_by(step) {
                            let end = start + range_size;
                            let count: usize = tree.range(black_box(start)..black_box(end))
                                .map(|(k, v)| {
                                    black_box(k);
                                    black_box(v);
                                    1
                                })
                                .sum();
                            total_items += count;
                        }
                        black_box(total_items);
                    });
                },
            );
        }
    }
    
    group.finish();
}

fn profile_random_range_scans(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_range_scans");
    
    let tree_size = 1_000_000;
    let range_sizes = vec![100, 1_000, 10_000];
    
    for &range_size in &range_sizes {
        group.bench_with_input(
            BenchmarkId::new("random_range_scan", format!("tree_{}_range_{}", tree_size, range_size)),
            &range_size,
            |b, &range_size| {
                // Pre-populate tree with random keys to create a more realistic scenario
                let mut tree = BPlusTreeMap::new(64).unwrap();
                let mut rng = StdRng::seed_from_u64(42);
                let mut keys: Vec<i32> = (0..tree_size).collect();
                keys.shuffle(&mut rng);
                
                for key in keys {
                    tree.insert(key, format!("value_{}", key));
                }
                
                // Pre-generate random range start points
                let mut range_starts: Vec<i32> = Vec::new();
                for _ in 0..100 {
                    let start = rng.gen_range(0..tree_size - range_size);
                    range_starts.push(start);
                }
                
                b.iter(|| {
                    let mut total_items = 0;
                    for &start in &range_starts {
                        let end = start + range_size;
                        let count: usize = tree.range(black_box(start)..black_box(end))
                            .map(|(k, v)| {
                                black_box(k);
                                black_box(v);
                                1
                            })
                            .sum();
                        total_items += count;
                    }
                    black_box(total_items);
                });
            },
        );
    }
    
    group.finish();
}

fn profile_range_iteration_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_iteration_patterns");
    
    let tree_size = 1_000_000;
    let range_size = 10_000;
    
    // Pre-populate tree
    let mut tree = BPlusTreeMap::new(64).unwrap();
    for i in 0..tree_size {
        tree.insert(i, format!("value_{}", i));
    }
    
    // Test different iteration patterns
    group.bench_function("collect_all", |b| {
        b.iter(|| {
            let start = tree_size / 4;
            let end = start + range_size;
            let items: Vec<_> = tree.range(black_box(start)..black_box(end)).collect();
            black_box(items);
        });
    });
    
    group.bench_function("count_only", |b| {
        b.iter(|| {
            let start = tree_size / 4;
            let end = start + range_size;
            let count = tree.range(black_box(start)..black_box(end)).count();
            black_box(count);
        });
    });
    
    group.bench_function("first_n_items", |b| {
        b.iter(|| {
            let start = tree_size / 4;
            let end = start + range_size;
            let items: Vec<_> = tree.range(black_box(start)..black_box(end))
                .take(100)
                .collect();
            black_box(items);
        });
    });
    
    group.bench_function("skip_and_take", |b| {
        b.iter(|| {
            let start = tree_size / 4;
            let end = start + range_size;
            let items: Vec<_> = tree.range(black_box(start)..black_box(end))
                .skip(1000)
                .take(1000)
                .collect();
            black_box(items);
        });
    });
    
    group.finish();
}

fn profile_range_bounds_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_bounds_types");
    
    let tree_size = 1_000_000;
    let range_size = 10_000;
    
    // Pre-populate tree
    let mut tree = BPlusTreeMap::new(64).unwrap();
    for i in 0..tree_size {
        tree.insert(i, format!("value_{}", i));
    }
    
    let start = tree_size / 4;
    let end = start + range_size;
    
    // Test different range bound types
    group.bench_function("inclusive_range", |b| {
        b.iter(|| {
            let count = tree.range(black_box(start)..=black_box(end)).count();
            black_box(count);
        });
    });
    
    group.bench_function("exclusive_range", |b| {
        b.iter(|| {
            let count = tree.range(black_box(start)..black_box(end)).count();
            black_box(count);
        });
    });
    
    group.bench_function("unbounded_from", |b| {
        b.iter(|| {
            let count = tree.range(black_box(start)..).take(range_size).count();
            black_box(count);
        });
    });
    
    group.bench_function("unbounded_to", |b| {
        b.iter(|| {
            let count = tree.range(..black_box(end)).take(range_size).count();
            black_box(count);
        });
    });
    
    group.finish();
}

fn profile_very_large_single_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("very_large_single_scan");
    
    // This benchmark focuses on a single very large range scan
    // to maximize time spent in the range iteration code
    let tree_size = 2_000_000;
    let range_size = 500_000; // 25% of the tree
    
    group.bench_function("massive_range_scan", |b| {
        // Pre-populate tree
        let mut tree = BPlusTreeMap::new(128).unwrap(); // Large capacity for fewer levels
        for i in 0..tree_size {
            tree.insert(i, format!("large_value_string_for_item_{}", i));
        }
        
        b.iter(|| {
            let start = tree_size / 4;
            let end = start + range_size;
            
            // Iterate through the entire range, touching each item
            let mut sum = 0i64;
            for (key, value) in tree.range(black_box(start)..black_box(end)) {
                sum += *key as i64;
                sum += value.len() as i64; // Force access to the value
            }
            black_box(sum);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    profile_large_range_scans,
    profile_random_range_scans,
    profile_range_iteration_patterns,
    profile_range_bounds_types,
    profile_very_large_single_scan
);
criterion_main!(benches);