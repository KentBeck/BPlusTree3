use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bplustree::GlobalCapacityBPlusTreeMap;
use std::collections::BTreeMap;

fn create_large_tree() -> GlobalCapacityBPlusTreeMap<i32, String> {
    let mut tree = GlobalCapacityBPlusTreeMap::new(16).expect("Failed to create tree");
    
    // Create a large tree with 100,000 items
    for i in 0..100_000 {
        tree.insert(i, format!("value_{}", i)).expect("Failed to insert");
    }
    
    tree
}

fn create_large_btreemap() -> BTreeMap<i32, String> {
    let mut map = BTreeMap::new();
    
    // Create a large BTreeMap with 100,000 items
    for i in 0..100_000 {
        map.insert(i, format!("value_{}", i));
    }
    
    map
}

fn bench_large_range_our_tree(c: &mut Criterion) {
    let tree = create_large_tree();
    
    c.bench_function("large_range_our_tree_10k", |b| {
        b.iter(|| {
            let range = tree.range(10_000..20_000);
            let count: usize = black_box(range.count());
            assert_eq!(count, 10_000);
        })
    });
    
    c.bench_function("large_range_our_tree_50k", |b| {
        b.iter(|| {
            let range = tree.range(25_000..75_000);
            let count: usize = black_box(range.count());
            assert_eq!(count, 50_000);
        })
    });
}

fn bench_large_range_std_btree(c: &mut Criterion) {
    let map = create_large_btreemap();
    
    c.bench_function("large_range_std_btree_10k", |b| {
        b.iter(|| {
            let range = map.range(10_000..20_000);
            let count: usize = black_box(range.count());
            assert_eq!(count, 10_000);
        })
    });
    
    c.bench_function("large_range_std_btree_50k", |b| {
        b.iter(|| {
            let range = map.range(25_000..75_000);
            let count: usize = black_box(range.count());
            assert_eq!(count, 50_000);
        })
    });
}

// Intensive profiling benchmark - designed for profiler analysis
fn bench_intensive_range_profiling(c: &mut Criterion) {
    let tree = create_large_tree();
    
    c.bench_function("intensive_range_profiling", |b| {
        b.iter(|| {
            // Multiple large range queries to generate significant profiling data
            for start in (0..80_000).step_by(10_000) {
                let end = start + 5_000;
                let range = tree.range(start..end);
                let _: Vec<_> = black_box(range.collect());
            }
        })
    });
}

criterion_group!(
    benches,
    bench_large_range_our_tree,
    bench_large_range_std_btree,
    bench_intensive_range_profiling
);
criterion_main!(benches);
