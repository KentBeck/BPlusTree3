use bplustree::BPlusTreeMap;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_key_operations(c: &mut Criterion) {
    // Test with both i32 (cheap to clone) and String (expensive to clone) keys

    // i32 benchmarks
    c.bench_function("i32_insert_1000", |b| {
        b.iter(|| {
            let mut tree = BPlusTreeMap::new(16).unwrap();
            for i in 0..1000 {
                tree.insert(black_box(i), black_box(i * 2));
            }
            tree
        });
    });

    c.bench_function("i32_lookup_1000", |b| {
        let mut tree = BPlusTreeMap::new(16).unwrap();
        for i in 0..1000 {
            tree.insert(i, i * 2);
        }

        b.iter(|| {
            for i in 0..1000 {
                black_box(tree.get(&black_box(i)));
            }
        });
    });

    // String benchmarks - these should show clone overhead
    c.bench_function("string_insert_1000", |b| {
        b.iter(|| {
            let mut tree = BPlusTreeMap::new(16).unwrap();
            for i in 0..1000 {
                let key = black_box(format!("key_{:06}", i));
                let value = black_box(format!("value_{}", i));
                tree.insert(key, value);
            }
            tree
        });
    });

    c.bench_function("string_lookup_1000", |b| {
        let mut tree = BPlusTreeMap::new(16).unwrap();
        for i in 0..1000 {
            tree.insert(format!("key_{:06}", i), format!("value_{}", i));
        }

        b.iter(|| {
            for i in 0..1000 {
                let key = black_box(format!("key_{:06}", i));
                black_box(tree.get(&key));
            }
        });
    });

    c.bench_function("string_contains_key_1000", |b| {
        let mut tree = BPlusTreeMap::new(16).unwrap();
        for i in 0..1000 {
            tree.insert(format!("key_{:06}", i), format!("value_{}", i));
        }

        b.iter(|| {
            for i in 0..1000 {
                let key = black_box(format!("key_{:06}", i));
                black_box(tree.contains_key(&key));
            }
        });
    });
}

criterion_group!(benches, benchmark_key_operations);
criterion_main!(benches);
