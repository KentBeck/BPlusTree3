use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use bplustree::GlobalCapacityBPlusTreeMap;
use std::collections::BTreeMap;

// Controlled experiment for auditor demonstration
fn auditor_range_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("auditor_range_comparison");
    
    // Test different tree sizes
    let tree_sizes = [1_000, 10_000, 100_000];
    let range_sizes = [100, 1_000, 10_000];
    
    for &tree_size in &tree_sizes {
        for &range_size in &range_sizes {
            if range_size > tree_size / 2 { continue; } // Skip invalid ranges
            
            // Create our B+ tree
            let mut our_tree = GlobalCapacityBPlusTreeMap::new(16).expect("Failed to create tree");
            for i in 0..tree_size {
                our_tree.insert(i, i * 2).expect("Failed to insert");
            }
            
            // Create std::BTreeMap
            let mut std_tree = BTreeMap::new();
            for i in 0..tree_size {
                std_tree.insert(i, i * 2);
            }
            
            let start_key = tree_size / 4; // Start at 25% through the tree
            let end_key = start_key + range_size;
            
            // Benchmark our tree
            group.bench_with_input(
                BenchmarkId::new("our_tree", format!("{}k_tree_{}k_range", tree_size/1000, range_size/1000)),
                &(&our_tree, start_key, end_key),
                |b, (tree, start, end)| {
                    b.iter(|| {
                        let count: usize = black_box(tree.range(*start..*end).count());
                        black_box(count);
                    })
                },
            );
            
            // Benchmark std::BTreeMap
            group.bench_with_input(
                BenchmarkId::new("std_tree", format!("{}k_tree_{}k_range", tree_size/1000, range_size/1000)),
                &(&std_tree, start_key, end_key),
                |b, (tree, start, end)| {
                    b.iter(|| {
                        let count: usize = black_box(tree.range(*start..*end).count());
                        black_box(count);
                    })
                },
            );
        }
    }
    
    group.finish();
}

// Test with different data types
fn auditor_data_type_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("auditor_data_types");
    
    // Integer keys, integer values (minimal overhead)
    let mut our_tree_int = GlobalCapacityBPlusTreeMap::new(16).expect("Failed to create tree");
    let mut std_tree_int = BTreeMap::new();
    
    for i in 0..10_000 {
        our_tree_int.insert(i, i * 2).expect("Failed to insert");
        std_tree_int.insert(i, i * 2);
    }
    
    group.bench_function("our_tree_int_int", |b| {
        b.iter(|| {
            let count: usize = black_box(our_tree_int.range(2500..3500).count());
            black_box(count);
        })
    });
    
    group.bench_function("std_tree_int_int", |b| {
        b.iter(|| {
            let count: usize = black_box(std_tree_int.range(2500..3500).count());
            black_box(count);
        })
    });
    
    group.finish();
}

// Memory access pattern test
fn auditor_access_pattern_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("auditor_access_patterns");
    
    let tree_size = 50_000;
    
    // Create trees
    let mut our_tree = GlobalCapacityBPlusTreeMap::new(16).expect("Failed to create tree");
    let mut std_tree = BTreeMap::new();
    
    for i in 0..tree_size {
        our_tree.insert(i, format!("value_{}", i)).expect("Failed to insert");
        std_tree.insert(i, format!("value_{}", i));
    }
    
    // Sequential access (best case)
    group.bench_function("our_tree_sequential", |b| {
        b.iter(|| {
            let items: Vec<_> = black_box(our_tree.range(10_000..15_000).collect());
            black_box(items.len());
        })
    });
    
    group.bench_function("std_tree_sequential", |b| {
        b.iter(|| {
            let items: Vec<_> = black_box(std_tree.range(10_000..15_000).collect());
            black_box(items.len());
        })
    });
    
    // Count only (no allocation)
    group.bench_function("our_tree_count_only", |b| {
        b.iter(|| {
            let count = black_box(our_tree.range(10_000..15_000).count());
            black_box(count);
        })
    });
    
    group.bench_function("std_tree_count_only", |b| {
        b.iter(|| {
            let count = black_box(std_tree.range(10_000..15_000).count());
            black_box(count);
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    auditor_range_benchmark,
    auditor_data_type_benchmark,
    auditor_access_pattern_benchmark
);
criterion_main!(benches);
