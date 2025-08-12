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
