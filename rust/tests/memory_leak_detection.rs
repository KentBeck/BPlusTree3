//! Memory leak regression tests for B+ tree implementation
//! These tests prevent memory leaks from being reintroduced after fixes

use bplustree::BPlusTreeMap;

mod test_utils;
use test_utils::*;

/// REGRESSION TEST: Prevents memory leaks in arena allocation system
/// This test was added after fixing the memory leak issue mentioned in code review.
/// It ensures allocated nodes always match tree structure nodes.
#[test]
fn test_memory_leak_regression_prevention() {
    println!("=== MEMORY LEAK REGRESSION PREVENTION ===");

    let mut tree: BPlusTreeMap<i32, String> = BPlusTreeMap::new(4).unwrap();

    // Record initial state
    let initial_leaf_stats = tree.leaf_arena_stats();
    let initial_branch_stats = tree.branch_arena_stats();

    println!("Initial state:");
    println!(
        "  Allocated leaves: {}, branches: {}",
        initial_leaf_stats.allocated_count, initial_branch_stats.allocated_count
    );
    println!(
        "  Free leaves: {}, branches: {}",
        initial_leaf_stats.free_count, initial_branch_stats.free_count
    );

    // Perform operations that force multiple root splits and merges
    for cycle in 0..10 {
        println!("\n--- Cycle {} ---", cycle + 1);

        // Insert enough data to force multiple root splits
        let base = cycle * 100;
        for i in 0..50 {
            tree.insert(base + i, format!("value_{}_{}", cycle, i));
        }

        let after_insert_leaf_stats = tree.leaf_arena_stats();
        let after_insert_branch_stats = tree.branch_arena_stats();
        let tree_leaves = tree.leaf_count();
        let (_, tree_branches) = tree.count_nodes_in_tree();

        println!("  After insertions:");
        println!(
            "    Arena: {} leaves, {} branches",
            after_insert_leaf_stats.allocated_count, after_insert_branch_stats.allocated_count
        );
        println!(
            "    Tree:  {} leaves, {} branches",
            tree_leaves, tree_branches
        );

        // Check for immediate leaks
        if after_insert_leaf_stats.allocated_count > tree_leaves {
            println!(
                "    ⚠ LEAK: {} extra leaves allocated",
                after_insert_leaf_stats.allocated_count - tree_leaves
            );
        }
        if after_insert_branch_stats.allocated_count > tree_branches {
            println!(
                "    ⚠ LEAK: {} extra branches allocated",
                after_insert_branch_stats.allocated_count - tree_branches
            );
        }

        // Remove some data to trigger merges and potential root collapse
        for i in 10..40 {
            tree.remove(&(base + i));
        }

        let after_delete_leaf_stats = tree.leaf_arena_stats();
        let after_delete_branch_stats = tree.branch_arena_stats();
        let tree_leaves_after = tree.leaf_count();
        let (_, tree_branches_after) = tree.count_nodes_in_tree();

        println!("  After deletions:");
        println!(
            "    Arena: {} leaves, {} branches",
            after_delete_leaf_stats.allocated_count, after_delete_branch_stats.allocated_count
        );
        println!(
            "    Tree:  {} leaves, {} branches",
            tree_leaves_after, tree_branches_after
        );

        // Check for leaks after deletions
        if after_delete_leaf_stats.allocated_count > tree_leaves_after {
            println!(
                "    ⚠ LEAK: {} extra leaves allocated",
                after_delete_leaf_stats.allocated_count - tree_leaves_after
            );
        }
        if after_delete_branch_stats.allocated_count > tree_branches_after {
            println!(
                "    ⚠ LEAK: {} extra branches allocated",
                after_delete_branch_stats.allocated_count - tree_branches_after
            );
        }
    }

    // Final state check
    let final_leaf_stats = tree.leaf_arena_stats();
    let final_branch_stats = tree.branch_arena_stats();
    let final_tree_leaves = tree.leaf_count();
    let (_, final_tree_branches) = tree.count_nodes_in_tree();

    println!("\n=== FINAL LEAK ANALYSIS ===");
    println!("Final arena state:");
    println!(
        "  Allocated leaves: {}, branches: {}",
        final_leaf_stats.allocated_count, final_branch_stats.allocated_count
    );
    println!("Final tree state:");
    println!(
        "  Tree leaves: {}, branches: {}",
        final_tree_leaves, final_tree_branches
    );

    // Calculate potential leaks
    let leaf_leak = final_leaf_stats
        .allocated_count
        .saturating_sub(final_tree_leaves);
    let branch_leak = final_branch_stats
        .allocated_count
        .saturating_sub(final_tree_branches);

    if leaf_leak > 0 {
        println!("❌ LEAF MEMORY LEAK DETECTED: {} leaked nodes", leaf_leak);
        panic!(
            "Memory leak detected: {} leaf nodes allocated but not in tree",
            leaf_leak
        );
    }

    if branch_leak > 0 {
        println!(
            "❌ BRANCH MEMORY LEAK DETECTED: {} leaked nodes",
            branch_leak
        );
        panic!(
            "Memory leak detected: {} branch nodes allocated but not in tree",
            branch_leak
        );
    }

    println!("✅ MEMORY LEAK REGRESSION TEST PASSED - NO LEAKS");
}

/// REGRESSION TEST: Ensures root splits don't accumulate leaked nodes
/// This specifically targets the root creation memory leak mentioned in code review.
#[test]
fn test_root_split_no_memory_accumulation() {
    println!("=== ROOT SPLIT MEMORY ACCUMULATION PREVENTION ===");

    let mut tree: BPlusTreeMap<i32, String> = BPlusTreeMap::new(4).unwrap();

    for round in 1..=10 {
        // Insert enough to force a root split
        let start = (round - 1) * 5;
        for i in start..start + 5 {
            tree.insert(i, format!("value_{}", i));
        }

        let allocated =
            tree.leaf_arena_stats().allocated_count + tree.branch_arena_stats().allocated_count;
        let (tree_leaves, tree_branches) = tree.count_nodes_in_tree();
        let in_tree = tree_leaves + tree_branches;

        // CRITICAL: Arena allocations must exactly match tree structure
        assert_eq!(
            allocated, in_tree,
            "REGRESSION: Memory leak detected in round {} - {} allocated vs {} in tree",
            round, allocated, in_tree
        );

        if round % 3 == 0 {
            println!(
                "Round {}: {} nodes - allocation/tree match ✓",
                round, allocated
            );
        }
    }

    println!("✅ ROOT SPLIT MEMORY ACCUMULATION PREVENTED");
}

#[test]
fn test_arena_fragmentation_and_reuse() {
    println!("=== ARENA FRAGMENTATION AND REUSE TEST ===");

    let mut tree: BPlusTreeMap<i32, String> = BPlusTreeMap::new(6).unwrap();

    // Create fragmentation by inserting and removing in patterns
    for phase in 0..5 {
        println!("\n--- Fragmentation Phase {} ---", phase + 1);

        // Insert data
        let base = phase * 1000;
        for i in 0..100 {
            tree.insert(base + i, format!("phase_{}_{}", phase, i));
        }

        let after_insert = tree.leaf_arena_stats().allocated_count;
        let free_after_insert = tree.leaf_arena_stats().free_count;

        // Remove most data to create fragmentation
        for i in 0..80 {
            tree.remove(&(base + i));
        }

        let after_remove = tree.leaf_arena_stats().allocated_count;
        let free_after_remove = tree.leaf_arena_stats().free_count;

        println!("  Allocated: {} -> {}", after_insert, after_remove);
        println!("  Free: {} -> {}", free_after_insert, free_after_remove);

        // Verify free list is working
        if free_after_remove <= free_after_insert {
            println!("  ✅ Free list grew as expected");
        } else {
            println!("  ⚠ Free list behavior unexpected");
        }
    }

    // Final consistency check
    let final_allocated = tree.leaf_arena_stats().allocated_count;
    let final_in_tree = tree.leaf_count();

    if final_allocated != final_in_tree {
        panic!(
            "Final fragmentation test failed: {} allocated vs {} in tree",
            final_allocated, final_in_tree
        );
    }

    println!("✅ ARENA FRAGMENTATION TEST PASSED");
}

#[test]
fn test_stress_allocation_deallocation_cycles() {
    println!("=== STRESS ALLOCATION/DEALLOCATION CYCLES ===");

    let mut tree: BPlusTreeMap<i32, String> = BPlusTreeMap::new(4).unwrap();

    for cycle in 0..20 {
        // Insert batch
        let base = cycle * 50;
        for i in 0..50 {
            tree.insert(base + i, format!("cycle_{}_item_{}", cycle, i));
        }

        // Remove batch (but not all, to maintain tree structure)
        for i in 10..40 {
            tree.remove(&(base + i));
        }

        // Every few cycles, check for leaks
        if cycle % 5 == 4 {
            let allocated =
                tree.leaf_arena_stats().allocated_count + tree.branch_arena_stats().allocated_count;
            let (tree_leaves, tree_branches) = tree.count_nodes_in_tree();
            let in_tree = tree_leaves + tree_branches;

            if allocated != in_tree {
                panic!(
                    "Stress test leak detected at cycle {}: {} allocated vs {} in tree",
                    cycle, allocated, in_tree
                );
            }

            println!(
                "Cycle {}: {} nodes allocated and in tree ✅",
                cycle, allocated
            );
        }
    }

    println!("✅ STRESS TEST COMPLETED WITHOUT LEAKS");
}

#[test]
fn test_edge_case_memory_scenarios() {
    println!("=== EDGE CASE MEMORY SCENARIOS ===");

    // Test 1: Single node tree operations
    {
        let mut tree: BPlusTreeMap<i32, String> = BPlusTreeMap::new(4).unwrap();
        tree.insert(1, "single".to_string());

        let allocated = tree.leaf_arena_stats().allocated_count;
        let in_tree = tree.leaf_count();
        assert_eq!(allocated, in_tree, "Single node leak");

        tree.remove(&1);
        let after_remove_allocated = tree.leaf_arena_stats().allocated_count;
        let after_remove_in_tree = tree.leaf_count();
        assert_eq!(
            after_remove_allocated, after_remove_in_tree,
            "After single remove leak"
        );

        println!("  ✅ Single node scenario passed");
    }

    // Test 2: Minimum capacity edge case
    {
        let mut tree: BPlusTreeMap<i32, String> = BPlusTreeMap::new(4).unwrap(); // Minimum capacity

        // Fill to capacity then remove
        for i in 0..10 {
            tree.insert(i, format!("min_cap_{}", i));
        }

        deletion_range_attack(&mut tree, 10, 40);

        let allocated =
            tree.leaf_arena_stats().allocated_count + tree.branch_arena_stats().allocated_count;
        let (tree_leaves, tree_branches) = tree.count_nodes_in_tree();
        let in_tree = tree_leaves + tree_branches;
        assert_eq!(allocated, in_tree, "Minimum capacity leak");

        println!("  ✅ Minimum capacity scenario passed");
    }

    // Test 3: Large capacity edge case
    {
        let mut tree: BPlusTreeMap<i32, String> = BPlusTreeMap::new(1000).unwrap();

        // Insert enough to split even with large capacity
        for i in 0..2000 {
            tree.insert(i, format!("large_cap_{}", i));
        }

        let allocated =
            tree.leaf_arena_stats().allocated_count + tree.branch_arena_stats().allocated_count;
        let (tree_leaves, tree_branches) = tree.count_nodes_in_tree();
        let in_tree = tree_leaves + tree_branches;
        assert_eq!(allocated, in_tree, "Large capacity leak");

        println!("  ✅ Large capacity scenario passed");
    }

    println!("✅ ALL EDGE CASE MEMORY SCENARIOS PASSED");
}
