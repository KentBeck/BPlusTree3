use bplustree::BPlusTreeMap;
use std::time::{Duration, Instant};
use std::collections::HashMap;

struct ProfileData {
    call_count: u64,
    total_time: Duration,
    min_time: Duration,
    max_time: Duration,
}

impl ProfileData {
    fn new() -> Self {
        Self {
            call_count: 0,
            total_time: Duration::ZERO,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
        }
    }
    
    fn record(&mut self, duration: Duration) {
        self.call_count += 1;
        self.total_time += duration;
        self.min_time = self.min_time.min(duration);
        self.max_time = self.max_time.max(duration);
    }
    
    fn avg_time(&self) -> Duration {
        if self.call_count > 0 {
            self.total_time / self.call_count as u32
        } else {
            Duration::ZERO
        }
    }
}

fn main() {
    println!("Function-Level Delete Profiler");
    println!("==============================");
    
    // Profile different delete scenarios
    profile_delete_scenarios();
}

fn profile_delete_scenarios() {
    let scenarios = vec![
        ("Sequential Deletes", create_sequential_delete_workload()),
        ("Random Deletes", create_random_delete_workload()),
        ("Rebalancing Heavy", create_rebalancing_workload()),
        ("Mixed Operations", create_mixed_workload()),
    ];
    
    for (name, workload) in scenarios {
        println!("\n{}", name);
        println!("{}", "=".repeat(name.len()));
        profile_workload(workload);
    }
}

fn profile_workload(workload: Vec<Operation>) {
    let mut tree = BPlusTreeMap::new(16).unwrap();
    let mut profiles: HashMap<String, ProfileData> = HashMap::new();
    
    // Pre-populate tree
    for i in 0..50_000 {
        tree.insert(i, format!("value_{}", i));
    }
    
    println!("Executing {} operations...", workload.len());
    let total_start = Instant::now();
    
    for op in workload {
        match op {
            Operation::Delete(key) => {
                let start = Instant::now();
                let result = tree.remove(&key);
                let duration = start.elapsed();
                
                profiles.entry("remove".to_string())
                    .or_insert_with(ProfileData::new)
                    .record(duration);
                    
                // Track successful vs failed deletes
                if result.is_some() {
                    profiles.entry("successful_delete".to_string())
                        .or_insert_with(ProfileData::new)
                        .record(duration);
                } else {
                    profiles.entry("failed_delete".to_string())
                        .or_insert_with(ProfileData::new)
                        .record(duration);
                }
            }
            Operation::Insert(key, value) => {
                let start = Instant::now();
                tree.insert(key, value);
                let duration = start.elapsed();
                
                profiles.entry("insert".to_string())
                    .or_insert_with(ProfileData::new)
                    .record(duration);
            }
            Operation::Lookup(key) => {
                let start = Instant::now();
                tree.get(&key);
                let duration = start.elapsed();
                
                profiles.entry("lookup".to_string())
                    .or_insert_with(ProfileData::new)
                    .record(duration);
            }
        }
    }
    
    let total_time = total_start.elapsed();
    println!("Total execution time: {:?}", total_time);
    
    // Print profile results
    println!("\nFunction Profile Results:");
    println!("{:<20} {:>10} {:>12} {:>12} {:>12} {:>12}", 
             "Function", "Calls", "Total (μs)", "Avg (μs)", "Min (μs)", "Max (μs)");
    println!("{}", "-".repeat(80));
    
    let mut sorted_profiles: Vec<_> = profiles.iter().collect();
    sorted_profiles.sort_by(|a, b| b.1.total_time.cmp(&a.1.total_time));
    
    for (name, profile) in sorted_profiles {
        println!("{:<20} {:>10} {:>12} {:>12} {:>12} {:>12}",
                 name,
                 profile.call_count,
                 profile.total_time.as_micros(),
                 profile.avg_time().as_micros(),
                 profile.min_time.as_micros(),
                 profile.max_time.as_micros());
    }
    
    // Calculate delete operation statistics
    if let Some(delete_profile) = profiles.get("remove") {
        println!("\nDelete Operation Analysis:");
        println!("- Total delete calls: {}", delete_profile.call_count);
        println!("- Average delete time: {:?}", delete_profile.avg_time());
        println!("- Delete time range: {:?} - {:?}", delete_profile.min_time, delete_profile.max_time);
        
        if let (Some(success), Some(fail)) = (profiles.get("successful_delete"), profiles.get("failed_delete")) {
            println!("- Successful deletes: {} (avg: {:?})", success.call_count, success.avg_time());
            println!("- Failed deletes: {} (avg: {:?})", fail.call_count, fail.avg_time());
        }
    }
}

#[derive(Clone)]
enum Operation {
    Insert(i32, String),
    Lookup(i32),
    Delete(i32),
}

fn create_sequential_delete_workload() -> Vec<Operation> {
    let mut ops = Vec::new();
    
    // Delete every other element sequentially
    for i in (0..25_000).step_by(2) {
        ops.push(Operation::Delete(i));
    }
    
    ops
}

fn create_random_delete_workload() -> Vec<Operation> {
    let mut seed = 42u64;
    let mut ops = Vec::new();
    
    // Pseudo-random deletes
    for _ in 0..25_000 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let key = (seed % 50_000) as i32;
        ops.push(Operation::Delete(key));
    }
    
    ops
}

fn create_rebalancing_workload() -> Vec<Operation> {
    let mut ops = Vec::new();
    
    // Pattern designed to cause maximum rebalancing
    // Delete in a pattern that creates underfull nodes
    for i in 0..25_000 {
        ops.push(Operation::Delete(i * 2)); // Delete every other element
    }
    
    ops
}

fn create_mixed_workload() -> Vec<Operation> {
    let mut seed = 42u64;
    let mut ops = Vec::new();
    
    // Mixed workload: 40% lookup, 30% delete, 30% insert
    for _ in 0..30_000 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let op_type = seed % 100;
        let key = (seed % 100_000) as i32;
        
        let op = match op_type {
            0..=39 => Operation::Lookup(key),
            40..=69 => Operation::Delete(key),
            70..=99 => Operation::Insert(key, format!("new_value_{}", key)),
            _ => unreachable!(),
        };
        
        ops.push(op);
    }
    
    ops
}