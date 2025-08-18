//! Memory safety audit tests
//! These tests verify that all type conversions are properly bounds-checked

use bplustree::BPlusTreeMap;

mod test_utils;
use test_utils::*;

/// Test arena bounds checking with large data sets
#[test]
fn test_arena_bounds_checking() {
    println!("=== ARENA BOUNDS CHECKING TEST ===");

    let mut tree: BPlusTreeMap<i32, String> = BPlusTreeMap::new(4).unwrap();

    // Test with a reasonable number of items to verify no panics
    // This used to potentially overflow on 64-bit systems
    insert_sequential_range(&mut tree, 10000);

    println!("Successfully inserted 10,000 items");
    println!("Allocated leaves: {}", tree.allocated_leaf_count());
    println!(
        "Allocated branches: {}",
        tree.branch_arena_stats().allocated_count
    );

    // Verify all items are accessible
    for i in 0..10000 {
        assert!(tree.contains_key(&i), "Key {} should be accessible", i);
    }

    // Test deletion with bounds checking
    for i in 0..5000 {
        tree.remove(&i);
    }

    println!("Successfully removed 5,000 items");
    println!("Remaining items: {}", tree.len());

    // Verify remaining items are still accessible
    for i in 5000..10000 {
        assert!(
            tree.contains_key(&i),
            "Key {} should still be accessible",
            i
        );
    }

    println!("✅ Arena bounds checking test passed");
}

/// Test NodeId capacity limits
#[test]
fn test_node_id_capacity_limits() {
    println!("=== NODE ID CAPACITY LIMITS TEST ===");

    let mut tree: BPlusTreeMap<i32, String> = BPlusTreeMap::new(4).unwrap();

    // Test that we can handle NodeId values approaching u32::MAX
    // without panicking due to conversion issues
    let test_size = 50000; // Reasonable test size

    for i in 0..test_size {
        tree.insert(i, format!("test_value_{}", i));

        // Check every 10000 items that conversions are working
        if i % 10000 == 0 && i > 0 {
            let allocated = tree.allocated_leaf_count();
            let in_tree = tree.leaf_count();

            println!(
                "  {} items: {} allocated, {} in tree",
                i, allocated, in_tree
            );

            // Verify no overflow occurred
            assert!(allocated > 0, "Allocation count should be positive");
            assert!(in_tree > 0, "Tree count should be positive");
            assert!(allocated >= in_tree, "Allocated should be >= in tree");
        }
    }

    println!(
        "Successfully handled {} items without conversion errors",
        test_size
    );
    println!("✅ NodeId capacity limits test passed");
}

/// Test arena iteration with type safety
#[test]
fn test_arena_iteration_type_safety() {
    println!("=== ARENA ITERATION TYPE SAFETY TEST ===");

    let mut tree: BPlusTreeMap<i32, String> = BPlusTreeMap::new(6).unwrap();

    // Create a tree with various operations to test iteration safety
    for i in 0..1000 {
        tree.insert(i, format!("iteration_test_{}", i));
    }

    // Remove some items to create fragmentation
    deletion_range_attack(&mut tree, 100, 200);

    // Test that iteration works correctly with type conversions
    let items: Vec<_> = tree.items().collect();
    println!("Iteration collected {} items", items.len());

    // Verify iteration is working properly (1000 - 100 removed = 900)
    assert_eq!(items.len(), 900, "Should have 900 items after removals");

    // Check that items are in order (verifies NodeId conversions in iteration)
    for window in items.windows(2) {
        assert!(
            window[0].0 < window[1].0,
            "Items should be in ascending order: {} >= {}",
            window[0].0,
            window[1].0
        );
    }

    // Test range operations with type safety
    let range_items: Vec<_> = tree.range(300..400).collect();
    assert_eq!(range_items.len(), 100, "Range should contain 100 items");

    println!("✅ Arena iteration type safety test passed");
}

/// Test edge cases that could cause integer overflow
#[test]
fn test_integer_overflow_prevention() {
    println!("=== INTEGER OVERFLOW PREVENTION TEST ===");

    let mut tree: BPlusTreeMap<i32, String> = BPlusTreeMap::new(4).unwrap();

    // Test with large numbers that could cause overflow in calculations
    let large_numbers = [i32::MAX - 1000, i32::MAX - 100, i32::MAX - 10, i32::MAX - 1];

    for &num in &large_numbers {
        tree.insert(num, format!("large_num_{}", num));
    }

    println!("Successfully inserted large numbers");

    // Verify they're all accessible
    for &num in &large_numbers {
        assert!(
            tree.contains_key(&num),
            "Large number {} should be accessible",
            num
        );
    }

    // Test operations with these large numbers
    let items: Vec<_> = tree.items().map(|(k, _)| *k).collect();
    println!("Large numbers in tree: {:?}", items);

    // Test range operations with large numbers
    let range_start = i32::MAX - 500;
    let range_items: Vec<_> = tree.range(range_start..).collect();
    println!(
        "Range from {} contains {} items",
        range_start,
        range_items.len()
    );

    println!("✅ Integer overflow prevention test passed");
}

/// Test memory safety under stress conditions
#[test]
fn test_memory_safety_stress() {
    println!("=== MEMORY SAFETY STRESS TEST ===");

    let mut tree: BPlusTreeMap<i32, String> = BPlusTreeMap::new(8).unwrap();

    // Stress test with many allocations/deallocations
    for round in 0..100 {
        // Allocate a batch
        let base = round * 1000;
        for i in 0..500 {
            tree.insert(base + i, format!("stress_{}_{}", round, i));
        }

        // Deallocate some items
        for i in 100..400 {
            tree.remove(&(base + i));
        }

        // Every 20 rounds, verify integrity
        if round % 20 == 19 {
            let allocated =
                tree.leaf_arena_stats().allocated_count + tree.branch_arena_stats().allocated_count;
            let (tree_leaves, tree_branches) = tree.count_nodes_in_tree();
            let in_tree = tree_leaves + tree_branches;

            println!(
                "Round {}: {} allocated, {} in tree",
                round + 1,
                allocated,
                in_tree
            );

            // Verify no memory safety violations
            assert_eq!(
                allocated, in_tree,
                "Memory safety violation: allocated != in_tree"
            );
        }
    }

    println!("✅ Memory safety stress test passed");
}

/// Test bounds checking in specific arena operations
#[test]
fn test_arena_operations_bounds() {
    println!("=== ARENA OPERATIONS BOUNDS TEST ===");

    let mut tree: BPlusTreeMap<u32, String> = BPlusTreeMap::new(4).unwrap();

    // Test with u32 keys to stress NodeId conversions
    let test_keys = [0u32, 1000, 10000, 100000, 1000000];

    for &key in &test_keys {
        tree.insert(key, format!("bounds_test_{}", key));
    }

    println!("Inserted keys: {:?}", test_keys);

    // Verify all keys are accessible
    for &key in &test_keys {
        assert!(tree.contains_key(&key), "Key {} should be accessible", key);

        let value = tree.get(&key);
        assert!(value.is_some(), "Should be able to get key {}", key);
        assert_eq!(
            value.unwrap(),
            &format!("bounds_test_{}", key),
            "Value should match for key {}",
            key
        );
    }

    // Test removal with bounds checking
    for &key in &test_keys {
        let removed = tree.remove(&key);
        assert!(removed.is_some(), "Should be able to remove key {}", key);
        assert!(
            !tree.contains_key(&key),
            "Key {} should be gone after removal",
            key
        );
    }

    assert!(
        tree.is_empty(),
        "Tree should be empty after removing all keys"
    );

    println!("✅ Arena operations bounds test passed");
}
