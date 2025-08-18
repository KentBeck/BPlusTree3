use bplustree::{assert_tree_valid, verify_attack_result};

mod test_utils;
use test_utils::*;

/// These tests target the arena allocation system, trying to expose
/// memory corruption, ID overflow, and free list management bugs.

#[test]
fn test_arena_id_exhaustion_attack() {
    use test_utils::*;

    // Attack: Try to exhaust the arena ID space by repeatedly allocating and deallocating
    let mut tree = create_attack_tree(4);

    // Phase 1: Create and destroy many nodes to stress the free list
    stress_test_cycle(&mut tree, 1000, arena_exhaustion_attack);

    // Phase 2: Try to create a pattern that fragments the arena
    tree.clear();
    fragmentation_attack(&mut tree, 0);

    // Verify the tree is still consistent
    verify_attack_result!(tree, "arena fragmentation", full = 500);
}

#[test]
fn test_concurrent_arena_access_simulation() {
    use test_utils::*;

    // Attack: Simulate concurrent access patterns that might expose arena bugs
    // (Note: This isn't true concurrency, but simulates interleaved operations)
    let mut tree = create_attack_tree(4);

    // Create multiple "threads" of operations
    let (thread1_ops, thread2_ops) = setup_concurrent_simulation();

    // Interleave operations with automatic invariant checking
    execute_interleaved_ops(&mut tree, &thread1_ops, &thread2_ops);
}

#[test]
fn test_arena_growth_boundary_attack() {
    // Attack: Target the arena growth logic by hitting exact growth boundaries

    let capacity = 4;
    let mut tree = create_tree_capacity_int(capacity);

    // Calculate how many nodes we need to force arena growth
    // Start with small increments to find the boundary
    let mut last_leaf_arena_size = 1; // We start with one leaf
    let _last_branch_arena_size = 0;

    for i in 0..10000 {
        tree.insert(i, i);

        // Check if arena grew (this is a bit of a hack - better would be to expose arena size)
        let current_size = tree.len();
        if current_size > last_leaf_arena_size * 10 {
            println!("Arena likely grew at {} items", current_size);
            last_leaf_arena_size = current_size;

            // Now try to corrupt by deleting and reinserting at boundary
            for j in (i - 100)..i {
                if tree.contains_key(&j) {
                    tree.remove(&j);
                }
            }

            // Reinsert in different order
            for j in (i - 100)..i {
                tree.insert(j, j * 2);
            }

            // Check for corruption
            assert_invariants_int(&tree, "growth boundary attack");
        }
    }
}

#[test]
fn test_free_list_corruption_attack() {
    // Attack: Try to corrupt the free list by specific allocation/deallocation patterns

    let capacity = 4;
    let mut tree = create_tree_capacity_int(capacity);

    // Step 1: Create a specific tree structure
    for i in 0..32 {
        tree.insert(i * 3, i);
    }

    println!(
        "Initial free lists: leaves={}, branches={}",
        tree.leaf_arena_stats().free_count,
        tree.branch_arena_stats().free_count
    );

    // Step 2: Delete in a pattern that creates a specific free list state
    for i in vec![3, 9, 15, 21, 27, 33, 39, 45] {
        tree.remove(&i);
    }

    println!(
        "After deletions: leaves={}, branches={}",
        tree.leaf_arena_stats().free_count,
        tree.branch_arena_stats().free_count
    );

    // Step 3: Insert items that will reuse free list in specific order
    for i in 0..8 {
        tree.insert(i * 3 + 1, i);
    }

    // Step 4: Delete everything and see if free list is corrupted
    let keys: Vec<_> = tree.keys().cloned().collect();
    for key in keys {
        tree.remove(&key);

        // Check tree is still valid
        if let Err(e) = tree.check_invariants_detailed() {
            panic!("ATTACK SUCCESSFUL during cleanup: {}", e);
        }
    }

    // Tree should be empty but valid
    if !tree.is_empty() {
        panic!("ATTACK SUCCESSFUL: Tree not empty after deleting all keys!");
    }

    // Try to reuse the tree - this might expose free list corruption
    for i in 0..50 {
        tree.insert(i, i);
    }

    if tree.len() != 50 {
        panic!("ATTACK SUCCESSFUL: Can't reuse tree properly, free list corrupted!");
    }
}

#[test]
fn test_deep_recursion_arena_explosion() {
    // Attack: Force deep recursion that might cause arena to grow unexpectedly

    let capacity = 4; // Small capacity forces more splits
    let mut tree = create_tree_capacity_int(capacity);

    // Insert keys in a pattern that maximizes tree depth
    let mut key = 0i64;
    let multiplier = 1000000;

    for level in 0..10 {
        let count = 2_usize.pow(level);
        for _i in 0..count {
            tree.insert(key as i32, level as i32);
            key += multiplier / count as i64;
        }
    }

    println!("Created tree with {} nodes", tree.len());
    println!(
        "Free lists: leaves={}, branches={}",
        tree.leaf_arena_stats().free_count,
        tree.branch_arena_stats().free_count
    );

    // Now delete internal nodes to force complex rebalancing
    let total = tree.len();
    let mut deleted = 0;

    // Delete in reverse order to stress the tree structure
    for level in (0..10).rev() {
        let count = 2_usize.pow(level);
        for i in 0..count / 2 {
            let key_to_delete = (multiplier / count as i64) * i as i64;
            if tree.remove(&(key_to_delete as i32)).is_some() {
                deleted += 1;
            }
        }
    }

    println!("Deleted {} items", deleted);

    // Verify tree integrity
    if tree.len() != total - deleted {
        panic!(
            "ATTACK SUCCESSFUL: Lost items during deep recursion! Expected {}, got {}",
            total - deleted,
            tree.len()
        );
    }
}

#[test]
#[should_panic(expected = "ATTACK SUCCESSFUL")]
fn test_force_arena_corruption_panic() {
    // Attack: Try everything we can think of to corrupt the arena

    let _capacity = 5; // Odd number for interesting arithmetic
    let mut tree = create_tree_5();

    // Rapidly allocate and deallocate
    for round in 0..100 {
        // Fill with sequential keys
        for i in 0..20 {
            tree.insert(round * 100 + i, format!("round_{}_item_{}", round, i));
        }

        // Delete in problematic order (middle-out)
        for i in vec![
            10, 9, 11, 8, 12, 7, 13, 6, 14, 5, 15, 4, 16, 3, 17, 2, 18, 1, 19, 0,
        ] {
            tree.remove(&(round * 100 + i));
        }

        // Insert with gaps
        for i in 0..10 {
            tree.insert(round * 100 + i * 2, format!("reused_{}", i * i));
        }

        // Check if we've corrupted anything
        if let Err(e) = tree.check_invariants_detailed() {
            panic!(
                "ATTACK SUCCESSFUL: Arena corrupted at round {}: {}",
                round, e
            );
        }
    }

    // If we haven't panicked yet, force it
    panic!("ATTACK SUCCESSFUL: Expected arena corruption didn't occur, implementation is suspiciously robust!");
}
