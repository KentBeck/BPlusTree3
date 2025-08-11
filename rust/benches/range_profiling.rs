use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bplustree::GlobalCapacityBPlusTreeMap;
use std::collections::BTreeMap;

fn range_query_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_queries");
    
    // Setup our B+ tree
    let mut our_tree = GlobalCapacityBPlusTreeMap::new(16).unwrap();
    for i in 0..10000 {
        our_tree.insert(i, i * 10).unwrap();
    }
    
    // Setup std BTreeMap for comparison
    let mut std_tree = BTreeMap::new();
    for i in 0..10000 {
        std_tree.insert(i, i * 10);
    }
    
    // Benchmark range creation (finding start position)
    group.bench_function("our_tree_range_creation", |b| {
        b.iter(|| {
            let _iter = our_tree.range(black_box(4500)..black_box(5500));
        })
    });
    
    group.bench_function("std_tree_range_creation", |b| {
        b.iter(|| {
            let _iter = std_tree.range(black_box(4500)..black_box(5500));
        })
    });
    
    // Benchmark range iteration (small range)
    group.bench_function("our_tree_small_range", |b| {
        b.iter(|| {
            let items: Vec<_> = our_tree.range(black_box(4990)..black_box(5010)).collect();
            black_box(items);
        })
    });
    
    group.bench_function("std_tree_small_range", |b| {
        b.iter(|| {
            let items: Vec<_> = std_tree.range(black_box(4990)..black_box(5010)).collect();
            black_box(items);
        })
    });
    
    // Benchmark range iteration (large range)
    group.bench_function("our_tree_large_range", |b| {
        b.iter(|| {
            let items: Vec<_> = our_tree.range(black_box(2000)..black_box(8000)).collect();
            black_box(items);
        })
    });
    
    group.bench_function("std_tree_large_range", |b| {
        b.iter(|| {
            let items: Vec<_> = std_tree.range(black_box(2000)..black_box(8000)).collect();
            black_box(items);
        })
    });
    
    group.finish();
}

criterion_group!(benches, range_query_benchmark);
criterion_main!(benches);
