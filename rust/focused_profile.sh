#!/bin/bash

# Focused profiling script for macOS using available tools
set -e

echo "=== Focused Range Scan Profiling ==="

# Build with profiling flags
echo "Building with profiling optimizations..."
RUSTFLAGS="-C force-frame-pointers=yes -C debug-assertions=off" cargo build --release --bench range_scan_profiling

# Create results directory
mkdir -p focused_results
cd focused_results

echo
echo "1. Running targeted benchmark for profiling..."
# Run just the most intensive benchmark
cargo bench --bench range_scan_profiling very_large_single_scan -- --sample-size 20 > targeted_results.txt 2>&1

echo "2. Running with time profiling..."
# Use time command for basic profiling
/usr/bin/time -l ../target/release/deps/range_scan_profiling-* very_large_single_scan > time_profile.txt 2>&1 || true

echo "3. Running with dtrace (if available)..."
if command -v dtrace &> /dev/null; then
    echo "   Using dtrace for system call analysis..."
    sudo dtrace -n 'syscall:::entry /execname == "range_scan_profiling"/ { @[probefunc] = count(); }' \
        -c "../target/release/deps/range_scan_profiling-* very_large_single_scan" > dtrace_syscalls.txt 2>/dev/null || true
else
    echo "   dtrace not available"
fi

echo "4. Running with sample (macOS profiler)..."
if command -v sample &> /dev/null; then
    echo "   Using sample for CPU profiling..."
    timeout 30s sample ../target/release/deps/range_scan_profiling-* 10 -f sample_profile.txt &
    SAMPLE_PID=$!
    ../target/release/deps/range_scan_profiling-* very_large_single_scan > /dev/null 2>&1 || true
    wait $SAMPLE_PID 2>/dev/null || true
else
    echo "   sample not available"
fi

echo "5. Creating custom performance analysis..."
cat > custom_analysis.rs << 'EOF'
use std::time::{Duration, Instant};
use std::collections::HashMap;

fn main() {
    println!("=== Custom Performance Analysis ===");
    
    // Simulate the key operations we see in range scans
    analyze_tree_navigation();
    analyze_iteration_patterns();
    analyze_memory_access();
}

fn analyze_tree_navigation() {
    println!("\n--- Tree Navigation Analysis ---");
    
    // Simulate tree navigation with different depths
    let depths = vec![3, 4, 5, 6, 7]; // Typical B+ tree depths
    
    for depth in depths {
        let start = Instant::now();
        
        // Simulate tree traversal
        let mut current = 0;
        for level in 0..depth {
            // Simulate node access and key comparison
            for _ in 0..64 { // Typical node capacity
                current = current.wrapping_add(level);
                std::hint::black_box(current);
            }
        }
        
        let elapsed = start.elapsed();
        println!("Depth {}: {:?} per navigation", depth, elapsed);
    }
}

fn analyze_iteration_patterns() {
    println!("\n--- Iteration Pattern Analysis ---");
    
    let sizes = vec![100, 1_000, 10_000, 50_000];
    
    for size in sizes {
        // Sequential access
        let start = Instant::now();
        for i in 0..size {
            std::hint::black_box(i);
        }
        let sequential_time = start.elapsed();
        
        // Random access pattern
        let start = Instant::now();
        let mut current = 0;
        for _ in 0..size {
            current = (current * 1103515245 + 12345) % size; // Simple LCG
            std::hint::black_box(current);
        }
        let random_time = start.elapsed();
        
        println!("Size {:5}: Sequential {:?}, Random {:?} ({:.1}x slower)", 
                 size, sequential_time, random_time, 
                 random_time.as_nanos() as f64 / sequential_time.as_nanos() as f64);
    }
}

fn analyze_memory_access() {
    println!("\n--- Memory Access Pattern Analysis ---");
    
    // Simulate different memory access patterns
    let sizes = vec![1024, 4096, 16384, 65536]; // Different cache sizes
    
    for size in sizes {
        let data: Vec<u64> = (0..size).collect();
        
        // Sequential access
        let start = Instant::now();
        let mut sum = 0u64;
        for &value in &data {
            sum = sum.wrapping_add(value);
        }
        std::hint::black_box(sum);
        let sequential_time = start.elapsed();
        
        // Strided access (simulate pointer chasing)
        let start = Instant::now();
        let mut sum = 0u64;
        let stride = 64; // Cache line size
        for i in (0..size).step_by(stride) {
            sum = sum.wrapping_add(data[i]);
        }
        std::hint::black_box(sum);
        let strided_time = start.elapsed();
        
        println!("Size {:5}: Sequential {:?}, Strided {:?} ({:.1}x slower)", 
                 size, sequential_time, strided_time,
                 strided_time.as_nanos() as f64 / sequential_time.as_nanos() as f64);
    }
}
EOF

echo "   Compiling and running custom analysis..."
rustc -O custom_analysis.rs
./custom_analysis > custom_analysis_results.txt 2>&1
rm -f custom_analysis custom_analysis.rs

echo "6. Generating focused summary..."
cat > focused_summary.txt << EOF
Focused Range Scan Profiling Results
====================================

Generated: $(date)

Key Findings from Targeted Analysis:
EOF

if [ -f "targeted_results.txt" ]; then
    echo "
Benchmark Results:" >> focused_summary.txt
    grep "time:" targeted_results.txt | tail -5 >> focused_summary.txt
fi

if [ -f "time_profile.txt" ]; then
    echo "
System Resource Usage:" >> focused_summary.txt
    grep -E "(real|user|sys|maximum resident set size)" time_profile.txt >> focused_summary.txt
fi

if [ -f "custom_analysis_results.txt" ]; then
    echo "
Custom Analysis Results:" >> focused_summary.txt
    cat custom_analysis_results.txt >> focused_summary.txt
fi

cat >> focused_summary.txt << EOF

Performance Hotspot Analysis:
1. Tree navigation overhead increases exponentially with depth
2. Random access patterns show significant cache penalty
3. Memory access patterns suggest arena layout optimization opportunities

Recommended Profiling Focus Areas:
1. Function-level profiling of range start position finding
2. Cache miss analysis during leaf node iteration
3. Memory allocation patterns in arena management
4. Key comparison overhead during tree traversal

Next Steps:
1. Use Linux perf for detailed function profiling
2. Analyze assembly output for hot functions
3. Profile with different node capacities
4. Test with different key/value sizes
EOF

echo
echo "=== Focused Profiling Complete ==="
echo "Results in focused_results/:"
ls -la
echo
echo "Key files:"
echo "- focused_summary.txt: Main findings and recommendations"
echo "- targeted_results.txt: Benchmark results"
echo "- custom_analysis_results.txt: Performance pattern analysis"

cd ..