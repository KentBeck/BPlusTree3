//! Comprehensive and objective comparison between BTreeMap and BPlusTreeMap
//! This benchmark aims to demonstrate where each data structure excels

use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;
use std::time::Instant;
use std::hint::black_box;

struct BenchmarkResult {
    name: String,
    btree_time: std::time::Duration,
    bplus_time: std::time::Duration,
    bplus_fast_time: Option<std::time::Duration>,
    ratio: f64,
    fast_ratio: Option<f64>,
}

impl BenchmarkResult {
    fn new(name: &str, btree_time: std::time::Duration, bplus_time: std::time::Duration, bplus_fast_time: Option<std::time::Duration>) -> Self {
        let ratio = bplus_time.as_nanos() as f64 / btree_time.as_nanos() as f64;
        let fast_ratio = bplus_fast_time.map(|fast| fast.as_nanos() as f64 / btree_time.as_nanos() as f64);
        
        Self {
            name: name.to_string(),
            btree_time,
            bplus_time,
            bplus_fast_time,
            ratio,
            fast_ratio,
        }
    }
    
    fn winner(&self) -> &str {
        if let Some(fast_ratio) = self.fast_ratio {
            if fast_ratio < 1.0 { "BPlusTree (Fast)" }
            else if self.ratio < 1.0 { "BPlusTree" }
            else { "BTreeMap" }
        } else {
            if self.ratio < 1.0 { "BPlusTree" } else { "BTreeMap" }
        }
    }
    
    fn best_ratio(&self) -> f64 {
        if let Some(fast_ratio) = self.fast_ratio {
            if fast_ratio < self.ratio { fast_ratio } else { self.ratio }
        } else {
            self.ratio
        }
    }
}

fn run_benchmark<F>(_name: &str, iterations: usize, mut f: F) -> std::time::Duration 
where F: FnMut() {
    // Warmup
    for _ in 0..iterations/10 {
        f();
    }
    
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    start.elapsed()
}

fn main() {
    println!("🔬 COMPREHENSIVE BTREEMAP vs BPLUSTREEMAP COMPARISON");
    println!("=====================================================");
    println!("Objective analysis to determine when each data structure is superior\n");
    
    let mut results = Vec::new();
    
    // Test different dataset sizes
    for &size in &[100, 1000, 10000] {
        println!("📊 DATASET SIZE: {} items", size);
        println!("{}", "=".repeat(50));
        
        // Setup data structures
        let mut btree = BTreeMap::new();
        let mut bplus = BPlusTreeMap::new(64).unwrap(); // Optimal capacity
        
        for i in 0..size {
            btree.insert(i, i * 2);
            bplus.insert(i, i * 2);
        }
        
        // 1. INSERTION PERFORMANCE
        let btree_insert_time = run_benchmark("BTreeMap Insert", 100, || {
            let mut tree = BTreeMap::new();
            for i in 0..size {
                tree.insert(black_box(i), black_box(i * 2));
            }
            black_box(tree);
        });
        
        let bplus_insert_time = run_benchmark("BPlusTreeMap Insert", 100, || {
            let mut tree = BPlusTreeMap::new(64).unwrap();
            for i in 0..size {
                tree.insert(black_box(i), black_box(i * 2));
            }
            black_box(tree);
        });
        
        results.push(BenchmarkResult::new(
            &format!("Insertion ({})", size),
            btree_insert_time,
            bplus_insert_time,
            None
        ));
        
        // 2. LOOKUP PERFORMANCE
        let lookup_keys: Vec<i32> = (0..1000).map(|i| (i * 7) % size).collect();
        
        let btree_lookup_time = run_benchmark("BTreeMap Lookup", 1000, || {
            for &key in &lookup_keys {
                black_box(btree.get(&black_box(key)));
            }
        });
        
        let bplus_lookup_time = run_benchmark("BPlusTreeMap Lookup", 1000, || {
            for &key in &lookup_keys {
                black_box(bplus.get(&black_box(key)));
            }
        });
        
        results.push(BenchmarkResult::new(
            &format!("Lookup ({})", size),
            btree_lookup_time,
            bplus_lookup_time,
            None
        ));
        
        // 3. ITERATION PERFORMANCE
        let iterations = if size >= 10000 { 100 } else { 1000 };
        
        let btree_iter_time = run_benchmark("BTreeMap Iteration", iterations, || {
            for (k, v) in btree.iter() {
                black_box((k, v));
            }
        });
        
        let bplus_iter_time = run_benchmark("BPlusTreeMap Iteration", iterations, || {
            for (k, v) in bplus.items() {
                black_box((k, v));
            }
        });
        
        let bplus_fast_iter_time = run_benchmark("BPlusTreeMap Fast Iteration", iterations, || {
            for (k, v) in bplus.items_fast() {
                black_box((k, v));
            }
        });
        
        results.push(BenchmarkResult::new(
            &format!("Iteration ({})", size),
            btree_iter_time,
            bplus_iter_time,
            Some(bplus_fast_iter_time)
        ));
        
        // 4. RANGE QUERY PERFORMANCE
        let range_start = size / 4;
        let range_end = (size * 3) / 4;
        
        let btree_range_time = run_benchmark("BTreeMap Range", 1000, || {
            for (k, v) in btree.range(black_box(range_start)..black_box(range_end)) {
                black_box((k, v));
            }
        });
        
        let bplus_range_time = run_benchmark("BPlusTreeMap Range", 1000, || {
            for (k, v) in bplus.items_range(Some(&black_box(range_start)), Some(&black_box(range_end))) {
                black_box((k, v));
            }
        });
        
        results.push(BenchmarkResult::new(
            &format!("Range Query ({})", size),
            btree_range_time,
            bplus_range_time,
            None
        ));
        
        // 5. DELETION PERFORMANCE
        let btree_delete_time = run_benchmark("BTreeMap Delete", 100, || {
            let mut tree = btree.clone();
            for i in 0..size/2 {
                tree.remove(&black_box(i));
            }
            black_box(tree);
        });
        
        let bplus_delete_time = run_benchmark("BPlusTreeMap Delete", 100, || {
            let mut tree = BPlusTreeMap::new(64).unwrap();
            for j in 0..size { tree.insert(j, j * 2); }
            for i in 0..size/2 {
                tree.remove(&black_box(i));
            }
            black_box(tree);
        });

        results.push(BenchmarkResult::new(
            &format!("Deletion ({})", size),
            btree_delete_time,
            bplus_delete_time,
            None
        ));
        
        println!();
    }
    
    // EDGE CASE TESTING
    println!("🧪 EDGE CASE ANALYSIS");
    println!("{}", "=".repeat(50));
    
    // Small dataset performance
    let small_size = 10;
    let mut small_btree = BTreeMap::new();
    let mut small_bplus = BPlusTreeMap::new(4).unwrap(); // Minimum capacity
    
    for i in 0..small_size {
        small_btree.insert(i, i);
        small_bplus.insert(i, i);
    }
    
    let small_btree_time = run_benchmark("Small BTreeMap", 10000, || {
        for (k, v) in small_btree.iter() {
            black_box((k, v));
        }
    });
    
    let small_bplus_time = run_benchmark("Small BPlusTreeMap", 10000, || {
        for (k, v) in small_bplus.items() {
            black_box((k, v));
        }
    });
    
    let small_bplus_fast_time = run_benchmark("Small BPlusTreeMap Fast", 10000, || {
        for (k, v) in small_bplus.items_fast() {
            black_box((k, v));
        }
    });
    
    results.push(BenchmarkResult::new(
        "Small Dataset (10 items)",
        small_btree_time,
        small_bplus_time,
        Some(small_bplus_fast_time)
    ));
    
    // Memory usage analysis
    println!("\n💾 MEMORY USAGE ANALYSIS");
    println!("{}", "=".repeat(50));
    
    let btree_1k = {
        let mut tree = BTreeMap::new();
        for i in 0..1000 { tree.insert(i, i); }
        tree
    };
    
    let bplus_1k = {
        let mut tree = BPlusTreeMap::new(64).unwrap();
        for i in 0..1000 { tree.insert(i, i); }
        tree
    };
    
    println!("BTreeMap (1k items): {} bytes", std::mem::size_of_val(&btree_1k));
    println!("BPlusTreeMap (1k items): {} bytes", std::mem::size_of_val(&bplus_1k));
    println!("Memory overhead: {:.1}x", 
             std::mem::size_of_val(&bplus_1k) as f64 / std::mem::size_of_val(&btree_1k) as f64);
    
    // RESULTS SUMMARY
    println!("\n📈 COMPREHENSIVE RESULTS SUMMARY");
    println!("{}", "=".repeat(80));
    println!("{:<25} {:>12} {:>12} {:>12} {:>8} {:>15}", 
             "Operation", "BTreeMap", "BPlusTree", "BPlus(Fast)", "Ratio", "Winner");
    println!("{}", "-".repeat(80));
    
    let mut btree_wins = 0;
    let mut bplus_wins = 0;
    let mut bplus_fast_wins = 0;
    
    for result in &results {
        let winner = result.winner();
        match winner {
            "BTreeMap" => btree_wins += 1,
            "BPlusTree" => bplus_wins += 1,
            "BPlusTree (Fast)" => bplus_fast_wins += 1,
            _ => {}
        }
        
        let fast_time_str = result.bplus_fast_time
            .map(|t| format!("{:.2}ms", t.as_secs_f64() * 1000.0))
            .unwrap_or_else(|| "-".to_string());
        
        let ratio_str = if result.best_ratio() < 1.0 {
            format!("{:.2}x ✓", result.best_ratio())
        } else {
            format!("{:.2}x", result.best_ratio())
        };
        
        println!("{:<25} {:>10.2}ms {:>10.2}ms {:>12} {:>8} {:>15}", 
                 result.name,
                 result.btree_time.as_secs_f64() * 1000.0,
                 result.bplus_time.as_secs_f64() * 1000.0,
                 fast_time_str,
                 ratio_str,
                 winner);
    }
    
    println!("{}", "=".repeat(80));
    println!("SCORE: BTreeMap: {} | BPlusTree: {} | BPlusTree(Fast): {}", 
             btree_wins, bplus_wins, bplus_fast_wins);
    
    // DETAILED ANALYSIS
    println!("\n🔍 DETAILED ANALYSIS");
    println!("{}", "=".repeat(50));
    
    println!("\n🏆 BTreeMap Excels At:");
    for result in &results {
        if result.winner() == "BTreeMap" {
            println!("  • {}: {:.1}% faster", result.name, (result.ratio - 1.0) * 100.0);
        }
    }
    
    println!("\n🚀 BPlusTreeMap Excels At:");
    for result in &results {
        if result.winner().contains("BPlusTree") {
            let improvement = (1.0 - result.best_ratio()) * 100.0;
            println!("  • {}: {:.1}% faster ({})", result.name, improvement, result.winner());
        }
    }
    
    // RECOMMENDATIONS
    println!("\n💡 OBJECTIVE RECOMMENDATIONS");
    println!("{}", "=".repeat(50));
    
    let total_tests = results.len();
    let btree_win_rate = btree_wins as f64 / total_tests as f64;
    let bplus_total_wins = bplus_wins + bplus_fast_wins;
    let bplus_win_rate = bplus_total_wins as f64 / total_tests as f64;
    
    println!("Win Rate: BTreeMap {:.1}% | BPlusTreeMap {:.1}%", 
             btree_win_rate * 100.0, bplus_win_rate * 100.0);
    
    if btree_win_rate > 0.6 {
        println!("\n🎯 RECOMMENDATION: Use BTreeMap");
        println!("   BTreeMap wins {:.1}% of benchmarks and is the safer choice", btree_win_rate * 100.0);
    } else if bplus_win_rate > 0.6 {
        println!("\n🎯 RECOMMENDATION: Use BPlusTreeMap");
        println!("   BPlusTreeMap wins {:.1}% of benchmarks, especially with fast iteration", bplus_win_rate * 100.0);
    } else {
        println!("\n🎯 RECOMMENDATION: Context-Dependent");
        println!("   Performance is roughly equivalent - choose based on specific use case");
    }
    
    println!("\n📋 SPECIFIC USE CASE RECOMMENDATIONS:");
    println!("• Small datasets (< 100 items): BTreeMap");
    println!("• Range-heavy workloads: BTreeMap");
    println!("• Deletion-heavy workloads: BTreeMap");
    println!("• Memory-constrained environments: BTreeMap");
    println!("• Iteration-heavy workloads: BPlusTreeMap with items_fast()");
    println!("• Large datasets with mixed operations: BPlusTreeMap");
    println!("• Database-like access patterns: BPlusTreeMap");
    
    println!("\n⚠️  IMPORTANT NOTES:");
    println!("• BPlusTreeMap fast iteration requires unsafe code");
    println!("• BTreeMap is part of Rust's standard library (more stable)");
    println!("• BPlusTreeMap has higher memory overhead");
    println!("• Performance varies significantly with capacity tuning");
    
    println!("\n🏁 CONCLUSION:");
    if btree_wins > bplus_total_wins {
        println!("BTreeMap demonstrates superior performance in most scenarios.");
        println!("BPlusTreeMap is competitive but not consistently better.");
    } else {
        println!("BPlusTreeMap shows competitive performance with specific advantages.");
        println!("Choice depends on workload characteristics and safety requirements.");
    }
}
