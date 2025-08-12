//! Performance test for detailed profiling

use crate::global_capacity_tree::GlobalCapacityBPlusTreeMap;

pub fn perf_range_test() {
    let mut tree = GlobalCapacityBPlusTreeMap::new(16).unwrap();
    
    // Insert 100k items
    for i in 0..100_000 {
        tree.insert(i, i * 10).unwrap();
    }
    
    // Perform many range queries to get good profiling data
    for _ in 0..1000 {
        // Range creation (finding start position)
        let _iter = tree.range(45_000..55_000);
        
        // Range iteration
        let items: Vec<_> = tree.range(45_000..55_000).collect();
        assert_eq!(items.len(), 10_000);
        
        // Different range sizes
        let _small: Vec<_> = tree.range(49_990..50_010).collect();
        let _medium: Vec<_> = tree.range(48_000..52_000).collect();
        let _large: Vec<_> = tree.range(20_000..80_000).collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn perf_test() {
        perf_range_test();
    }
}
