#!/bin/bash

# Profile Range Scans Script
# This script runs various profilers on the Rust BPlusTreeMap range scan benchmarks
# to identify performance bottlenecks in large range operations.

set -e

echo "=== BPlusTreeMap Range Scan Profiling ==="
echo "This script will run multiple profilers to analyze range scan performance."
echo

# Check if we're in the rust directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Please run this script from the rust/ directory"
    exit 1
fi

# Create output directory for profiling results
mkdir -p profiling_results
cd profiling_results

echo "1. Building optimized benchmark with debug symbols..."
RUSTFLAGS="-g -C force-frame-pointers=yes" cargo build --release --bench range_scan_profiling

echo
echo "2. Running baseline benchmark to establish performance metrics..."
cargo bench --bench range_scan_profiling > baseline_results.txt 2>&1
echo "   Baseline results saved to profiling_results/baseline_results.txt"

echo
echo "3. Running with perf (Linux performance profiler)..."
if command -v perf &> /dev/null; then
    echo "   Running perf record..."
    timeout 30s perf record -g --call-graph=dwarf -F 1000 \
        ../target/release/deps/range_scan_profiling-* \
        very_large_single_scan 2>/dev/null || true
    
    if [ -f "perf.data" ]; then
        echo "   Generating perf report..."
        perf report --stdio > perf_report.txt 2>/dev/null || true
        perf annotate --stdio > perf_annotate.txt 2>/dev/null || true
        echo "   Perf results saved to profiling_results/perf_report.txt and perf_annotate.txt"
    fi
else
    echo "   perf not available (Linux only)"
fi

echo
echo "4. Running with Instruments (macOS profiler)..."
if command -v xcrun &> /dev/null && xcrun instruments -h &> /dev/null; then
    echo "   Running Instruments Time Profiler..."
    timeout 30s xcrun instruments -t "Time Profiler" -D instruments_trace.trace \
        ../target/release/deps/range_scan_profiling-* \
        very_large_single_scan 2>/dev/null || true
    echo "   Instruments trace saved to profiling_results/instruments_trace.trace"
    echo "   Open with: open instruments_trace.trace"
else
    echo "   Instruments not available (macOS only)"
fi

echo
echo "5. Running with Valgrind Callgrind (if available)..."
if command -v valgrind &> /dev/null; then
    echo "   Running Valgrind Callgrind..."
    timeout 60s valgrind --tool=callgrind --callgrind-out-file=callgrind.out \
        ../target/release/deps/range_scan_profiling-* \
        very_large_single_scan 2>/dev/null || true
    
    if [ -f "callgrind.out" ]; then
        echo "   Callgrind output saved to profiling_results/callgrind.out"
        echo "   View with: kcachegrind callgrind.out (or qcachegrind)"
        
        if command -v callgrind_annotate &> /dev/null; then
            callgrind_annotate callgrind.out > callgrind_report.txt 2>/dev/null || true
            echo "   Text report saved to profiling_results/callgrind_report.txt"
        fi
    fi
else
    echo "   Valgrind not available"
fi

echo
echo "6. Running with cargo-profdata (Rust-specific profiler)..."
if cargo install --list | grep -q "cargo-profdata"; then
    echo "   Running cargo-profdata..."
    RUSTFLAGS="-C instrument-coverage" cargo build --release --bench range_scan_profiling
    cargo profdata -- ../target/release/deps/range_scan_profiling-* very_large_single_scan || true
else
    echo "   cargo-profdata not installed. Install with: cargo install cargo-profdata"
fi

echo
echo "7. Running custom timing analysis..."
cat > timing_analysis.rs << 'EOF'
use std::time::{Duration, Instant};
use bplustree::BPlusTreeMap;

fn main() {
    println!("=== Custom Timing Analysis for Range Scans ===");
    
    let tree_size = 1_000_000;
    let range_size = 100_000;
    
    // Build tree
    println!("Building tree with {} items...", tree_size);
    let start_build = Instant::now();
    let mut tree = BPlusTreeMap::new(64).unwrap();
    for i in 0..tree_size {
        tree.insert(i, format!("value_{}", i));
    }
    let build_time = start_build.elapsed();
    println!("Tree build time: {:?}", build_time);
    
    // Test different range sizes
    let range_sizes = vec![100, 1_000, 10_000, 50_000, 100_000];
    
    for &size in &range_sizes {
        let start = tree_size / 4;
        let end = start + size;
        
        // Warm up
        for _ in 0..3 {
            let _: Vec<_> = tree.range(start..end).collect();
        }
        
        // Time the operation
        let iterations = if size < 10_000 { 100 } else { 10 };
        let start_time = Instant::now();
        
        for _ in 0..iterations {
            let items: Vec<_> = tree.range(start..end).collect();
            std::hint::black_box(items);
        }
        
        let elapsed = start_time.elapsed();
        let avg_time = elapsed / iterations;
        let items_per_sec = (size as f64) / avg_time.as_secs_f64();
        
        println!("Range size {:6}: {:8.2?} avg, {:10.0} items/sec", 
                 size, avg_time, items_per_sec);
    }
    
    // Test range iteration vs collection
    let range_size = 50_000;
    let start = tree_size / 4;
    let end = start + range_size;
    
    println!("\n=== Range Iteration Patterns ===");
    
    // Just iterate (don't collect)
    let start_time = Instant::now();
    for _ in 0..10 {
        let mut count = 0;
        for (k, v) in tree.range(start..end) {
            std::hint::black_box(k);
            std::hint::black_box(v);
            count += 1;
        }
        std::hint::black_box(count);
    }
    let iterate_time = start_time.elapsed() / 10;
    
    // Collect all
    let start_time = Instant::now();
    for _ in 0..10 {
        let items: Vec<_> = tree.range(start..end).collect();
        std::hint::black_box(items);
    }
    let collect_time = start_time.elapsed() / 10;
    
    // Count only
    let start_time = Instant::now();
    for _ in 0..10 {
        let count = tree.range(start..end).count();
        std::hint::black_box(count);
    }
    let count_time = start_time.elapsed() / 10;
    
    println!("Iterate only: {:8.2?}", iterate_time);
    println!("Collect all:  {:8.2?}", collect_time);
    println!("Count only:   {:8.2?}", count_time);
    
    println!("\nCollection overhead: {:.1}x", 
             collect_time.as_secs_f64() / iterate_time.as_secs_f64());
}
EOF

echo "   Compiling and running timing analysis..."
rustc --edition 2021 -O timing_analysis.rs -L ../target/release/deps --extern bplustree=../target/release/libbplustree.rlib
./timing_analysis > timing_analysis_results.txt 2>&1 || true
rm -f timing_analysis timing_analysis.rs
echo "   Timing analysis saved to profiling_results/timing_analysis_results.txt"

echo
echo "8. Generating summary report..."
cat > summary_report.txt << EOF
BPlusTreeMap Range Scan Profiling Summary
==========================================

Generated: $(date)
Tree Implementation: Rust BPlusTreeMap with arena allocation

Files Generated:
- baseline_results.txt: Criterion benchmark results
- timing_analysis_results.txt: Custom timing breakdown
EOF

if [ -f "perf_report.txt" ]; then
    echo "- perf_report.txt: Linux perf profiler results" >> summary_report.txt
    echo "- perf_annotate.txt: Linux perf source annotations" >> summary_report.txt
fi

if [ -f "instruments_trace.trace" ]; then
    echo "- instruments_trace.trace: macOS Instruments profiling data" >> summary_report.txt
fi

if [ -f "callgrind.out" ]; then
    echo "- callgrind.out: Valgrind callgrind profiling data" >> summary_report.txt
    echo "- callgrind_report.txt: Callgrind text report" >> summary_report.txt
fi

cat >> summary_report.txt << EOF

Key Areas to Investigate:
1. Range iterator initialization overhead
2. Leaf node traversal efficiency  
3. Memory access patterns during iteration
4. Arena allocation impact on cache performance
5. Linked list traversal vs tree traversal costs

Next Steps:
1. Analyze the profiling data to identify hotspots
2. Focus on the functions consuming the most time
3. Look for cache misses and memory access patterns
4. Consider optimizations like prefetching or SIMD
5. Profile with different tree capacities and structures

EOF

echo "   Summary report saved to profiling_results/summary_report.txt"

echo
echo "=== Profiling Complete ==="
echo "Results are in the profiling_results/ directory:"
ls -la
echo
echo "Key files to examine:"
echo "- summary_report.txt: Overview and next steps"
echo "- baseline_results.txt: Performance baseline"
echo "- timing_analysis_results.txt: Detailed timing breakdown"

if [ -f "perf_report.txt" ]; then
    echo "- perf_report.txt: Function-level performance data"
fi

if [ -f "callgrind_report.txt" ]; then
    echo "- callgrind_report.txt: Instruction-level analysis"
fi

echo
echo "To view results:"
echo "  cat profiling_results/summary_report.txt"
echo "  cat profiling_results/timing_analysis_results.txt"

cd ..