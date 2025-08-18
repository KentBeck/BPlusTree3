use bplustree::{BPlusTreeError, BPlusTreeMap, NodeRef};
use std::marker::PhantomData;

mod test_utils;
use test_utils::*;

// ============================================================================
// NODE REF TESTS
// ============================================================================

#[test]
fn test_node_ref_id_and_is_leaf() {
    let leaf: NodeRef<i32, i32> = NodeRef::Leaf(7, PhantomData);
    assert_eq!(leaf.id(), 7);
    assert!(leaf.is_leaf());

    let branch: NodeRef<i32, i32> = NodeRef::Branch(13, PhantomData);
    assert_eq!(branch.id(), 13);
    assert!(!branch.is_leaf());
}

// ============================================================================
// TRANSLATED PYTHON TESTS - Basic Operations
// ============================================================================

#[test]
fn test_insert_overwrite_value() {
    let mut tree = create_tree_4();

    // Insert key 1 with value "one"
    tree.insert(1, "one".to_string());
    assert_eq!(tree.get(&1), Some(&"one".to_string()));

    // Insert key 1 again with value "two"
    tree.insert(1, "two".to_string());

    // Make sure the value at key 1 is now "two"
    assert_eq!(tree.get(&1), Some(&"two".to_string()));
    assert_eq!(tree.len(), 1); // Should still be only one item
}

#[test]
fn test_create_empty_tree() {
    let tree = create_tree_4();
    assert_eq!(tree.len(), 0);
    assert!(tree.is_empty());
    assert_invariants(&tree, "empty tree");
}

#[test]
fn test_insert_and_get_single_item() {
    let mut tree = create_tree_4();
    tree.insert(1, "one".to_string());

    assert_eq!(tree.len(), 1);
    assert!(!tree.is_empty());
    assert_eq!(tree.get(&1), Some(&"one".to_string()));
    assert_invariants(&tree, "single item");
}

#[test]
fn test_insert_multiple_items() {
    let mut tree = create_tree_4();
    tree.insert(1, "one".to_string());
    tree.insert(2, "two".to_string());
    tree.insert(3, "three".to_string());

    assert_eq!(tree.len(), 3);
    assert_eq!(tree.get(&1), Some(&"one".to_string()));
    assert_eq!(tree.get(&2), Some(&"two".to_string()));
    assert_eq!(tree.get(&3), Some(&"three".to_string()));
    assert_invariants(&tree, "multiple items");
}

#[test]
fn test_update_existing_key() {
    let mut tree = create_tree_4();
    tree.insert(1, "one".to_string());
    let old_value = tree.insert(1, "ONE".to_string());

    assert_eq!(tree.len(), 1); // Size shouldn't change
    assert_eq!(tree.get(&1), Some(&"ONE".to_string()));
    assert_eq!(old_value, Some("one".to_string()));
    assert_invariants(&tree, "key update");
}

#[test]
fn test_contains_key() {
    let mut tree = create_tree_4();
    tree.insert(1, "one".to_string());
    tree.insert(2, "two".to_string());

    assert!(tree.contains_key(&1));
    assert!(tree.contains_key(&2));
    assert!(!tree.contains_key(&3));
    assert_invariants(&tree, "contains key");
}

#[test]
fn test_get_with_default() {
    let mut tree = create_tree_4();
    tree.insert(1, "one".to_string());

    assert_eq!(tree.get(&1), Some(&"one".to_string()));
    assert_eq!(tree.get(&2), None);
    assert_eq!(
        tree.get_or_default(&2, &"default".to_string()),
        &"default".to_string()
    );
    assert_invariants(&tree, "get with default");
}

// ============================================================================
// TRANSLATED PYTHON TESTS - Splitting Operations
// ============================================================================

#[test]
fn test_overflow() {
    let mut tree = create_tree_4();
    // With capacity=4, need 5 items to force a split
    tree.insert(1, "one".to_string());
    tree.insert(2, "two".to_string());
    tree.insert(3, "three".to_string());
    tree.insert(4, "four".to_string());
    tree.insert(5, "five".to_string());

    assert_invariants(&tree, "overflow test");
    assert_eq!(tree.len(), 5);
    assert_eq!(tree.get(&1), Some(&"one".to_string()));
    assert_eq!(tree.get(&2), Some(&"two".to_string()));
    assert_eq!(tree.get(&3), Some(&"three".to_string()));
    assert_eq!(tree.get(&4), Some(&"four".to_string()));
    assert_eq!(tree.get(&5), Some(&"five".to_string()));

    assert!(!tree.is_leaf_root());
}

#[test]
fn test_split_then_add() {
    let mut tree = create_tree_4();
    // With capacity=4, need more items to force multiple splits
    tree.insert(1, "one".to_string());
    tree.insert(2, "two".to_string());
    tree.insert(3, "three".to_string());
    tree.insert(4, "four".to_string());
    tree.insert(5, "five".to_string());
    tree.insert(6, "six".to_string());
    tree.insert(7, "seven".to_string());
    tree.insert(8, "eight".to_string());

    // Check correctness via invariants instead of exact structure
    assert_invariants(&tree, "split then add");
    assert_eq!(tree.len(), 8);
    assert_eq!(tree.get(&1), Some(&"one".to_string()));
    assert_eq!(tree.get(&2), Some(&"two".to_string()));
    assert_eq!(tree.get(&3), Some(&"three".to_string()));
    assert_eq!(tree.get(&4), Some(&"four".to_string()));
    assert_eq!(tree.get(&5), Some(&"five".to_string()));
    assert_eq!(tree.get(&6), Some(&"six".to_string()));
    assert_eq!(tree.get(&7), Some(&"seven".to_string()));
    assert_eq!(tree.get(&8), Some(&"eight".to_string()));

    // The simpler implementation may create more leaves, but that's OK
    // as long as invariants hold
    assert!(tree.leaf_count() >= 2); // At minimum need 2 leaves for 8 items with capacity 4
}

#[test]
fn test_many_insertions_maintain_invariants() {
    let mut tree = create_tree_capacity(6);

    // Insert many items
    for i in 0..20 {
        tree.insert(i, format!("value_{}", i));
        assert_invariants(&tree, &format!("insertion {}", i));
    }

    // Verify all items are retrievable
    for i in 0..20 {
        assert_eq!(tree.get(&i), Some(&format!("value_{}", i)));
    }
}

#[test]
fn test_parent_splitting() {
    let mut tree = create_tree_5(); // Small capacity to force parent splits

    // Insert enough items to force multiple levels of splits
    for i in 0..50 {
        tree.insert(i, format!("value_{}", i));
        assert_invariants(&tree, &format!("parent split {}", i));
    }

    // Verify all items are still retrievable
    for i in 0..50 {
        assert_eq!(tree.get(&i), Some(&format!("value_{}", i)));
    }

    // The tree should have multiple levels now
    assert!(!tree.is_leaf_root());

    // TODO: Check that no nodes are overfull when implemented
}

// ============================================================================
// TRANSLATED PYTHON TESTS - Removal Operations
// ============================================================================

#[test]
fn test_remove_single_item_from_leaf_root() {
    let mut tree = create_tree_4();
    tree.insert(1, "one".to_string());

    // Remove the item
    let removed = tree.remove(&1);

    // Tree should be empty
    assert_eq!(removed, Some("one".to_string()));
    assert_eq!(tree.len(), 0);
    assert!(!tree.contains_key(&1));
    assert_invariants(&tree, "remove single item");

    // Should return None when trying to get removed item
    assert_eq!(tree.get(&1), None);
}

#[test]
fn test_remove_multiple_items_from_leaf_root() {
    let mut tree = create_tree_4();
    tree.insert(1, "one".to_string());
    tree.insert(2, "two".to_string());
    tree.insert(3, "three".to_string());

    // Remove items
    let removed = tree.remove(&2);

    // Check state after first removal
    assert_eq!(removed, Some("two".to_string()));
    assert_eq!(tree.len(), 2);
    assert!(tree.contains_key(&1));
    assert!(!tree.contains_key(&2));
    assert!(tree.contains_key(&3));
    assert_eq!(tree.get(&1), Some(&"one".to_string()));
    assert_eq!(tree.get(&3), Some(&"three".to_string()));
    assert_invariants(&tree, "remove multiple first");

    // Remove another item
    let removed = tree.remove(&1);

    // Check state after second removal
    assert_eq!(removed, Some("one".to_string()));
    assert_eq!(tree.len(), 1);
    assert!(!tree.contains_key(&1));
    assert!(tree.contains_key(&3));
    assert_eq!(tree.get(&3), Some(&"three".to_string()));
    assert_invariants(&tree, "remove multiple second");

    // Remove last item
    let removed = tree.remove(&3);

    // Tree should be empty
    assert_eq!(removed, Some("three".to_string()));
    assert_eq!(tree.len(), 0);
    assert_invariants(&tree, "remove multiple last");
}

#[test]
fn test_remove_nonexistent_key_returns_none() {
    let mut tree = create_tree_4();
    tree.insert(1, "one".to_string());
    tree.insert(2, "two".to_string());

    // Try to remove non-existent key
    let removed = tree.remove(&3);

    // Should return None
    assert_eq!(removed, None);

    // Tree should be unchanged
    assert_eq!(tree.len(), 2);
    assert_eq!(tree.get(&1), Some(&"one".to_string()));
    assert_eq!(tree.get(&2), Some(&"two".to_string()));
    assert_invariants(&tree, "remove nonexistent");
}

// ============================================================================
// TRANSLATED PYTHON TESTS - More Removal Operations
// ============================================================================

#[test]
fn test_remove_from_tree_with_branch_root() {
    let mut tree = create_tree_4();

    // Insert enough items to create a branch root
    insert_range(&mut tree, 1, 6);

    // Verify we have a branch root
    assert!(!tree.is_leaf_root());
    assert_eq!(tree.len(), 5);

    // Remove an item
    let removed = tree.remove(&2);

    // Check the item was removed
    assert_eq!(removed, Some("value_2".to_string()));
    assert_eq!(tree.len(), 4);
    assert!(!tree.contains_key(&2));
    assert_eq!(tree.get(&1), Some(&"value_1".to_string()));
    assert_eq!(tree.get(&3), Some(&"value_3".to_string()));
    assert_eq!(tree.get(&4), Some(&"value_4".to_string()));
    assert_eq!(tree.get(&5), Some(&"value_5".to_string()));
    assert!(tree.check_invariants());
}

#[test]
fn test_remove_multiple_from_tree_with_branches() {
    let mut tree = BPlusTreeMap::new(4).unwrap();

    // Insert more items to ensure we have multiple levels
    for i in 1..=9 {
        tree.insert(i, format!("value_{}", i));
    }

    // Remove items in various orders
    let removed1 = tree.remove(&3);
    let removed2 = tree.remove(&6);
    let removed3 = tree.remove(&1);

    // Check remaining items
    assert_eq!(removed1, Some("value_3".to_string()));
    assert_eq!(removed2, Some("value_6".to_string()));
    assert_eq!(removed3, Some("value_1".to_string()));
    assert_eq!(tree.len(), 6);
    assert_eq!(tree.get(&2), Some(&"value_2".to_string()));
    assert_eq!(tree.get(&4), Some(&"value_4".to_string()));
    assert_eq!(tree.get(&5), Some(&"value_5".to_string()));
    assert_eq!(tree.get(&7), Some(&"value_7".to_string()));
    assert_eq!(tree.get(&8), Some(&"value_8".to_string()));
    assert_eq!(tree.get(&9), Some(&"value_9".to_string()));

    // Check removed items are gone
    assert!(!tree.contains_key(&1));
    assert!(!tree.contains_key(&3));
    assert!(!tree.contains_key(&6));

    assert!(tree.check_invariants());
}

// ============================================================================
// TRANSLATED PYTHON TESTS - Range and Iterator Operations
// ============================================================================

// TODO: Implement iterator tests after fixing lifetime issues
/*
#[test]
fn test_keys_iterator() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    tree.insert(1, "one".to_string());
    tree.insert(2, "two".to_string());
    tree.insert(3, "three".to_string());

    let keys: Vec<_> = tree.keys().collect();
    assert_eq!(keys, vec![&1, &2, &3]);
}

#[test]
fn test_values_iterator() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    tree.insert(1, "one".to_string());
    tree.insert(2, "two".to_string());
    tree.insert(3, "three".to_string());

    let values: Vec<_> = tree.values().collect();
    assert_eq!(values, vec![&"one".to_string(), &"two".to_string(), &"three".to_string()]);
}

#[test]
fn test_items_iterator() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    tree.insert(1, "one".to_string());
    tree.insert(2, "two".to_string());
    tree.insert(3, "three".to_string());

    let items: Vec<_> = tree.iter().collect();
    assert_eq!(items, vec![
        (&1, &"one".to_string()),
        (&2, &"two".to_string()),
        (&3, &"three".to_string())
    ]);
}

#[test]
fn test_range_iterator() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    for i in 1..=10 {
        tree.insert(i, format!("value_{}", i));
    }

    let range_items: Vec<_> = tree.items_range(Some(&3), Some(&8)).collect();
    assert_eq!(range_items, vec![
        (&3, &"value_3".to_string()),
        (&4, &"value_4".to_string()),
        (&5, &"value_5".to_string()),
        (&6, &"value_6".to_string()),
        (&7, &"value_7".to_string())
    ]);
}
*/

// ============================================================================
// TRANSLATED PYTHON TESTS - Node Operations (for future implementation)
// ============================================================================

// These tests will be implemented when we add the Node trait and specific node operations

// ============================================================================
// STEP 5: BASIC INSERT THROUGH BRANCHNODES
// ============================================================================

#[test]
fn test_insert_through_branch_node() {
    let mut tree = BPlusTreeMap::new(4).unwrap();

    // First, create a tree with a branch root by inserting enough items
    // to cause a leaf split and root promotion
    for i in 1..=5 {
        tree.insert(i, format!("value_{}", i));
    }

    // Verify we have a branch root (not a leaf root)
    assert!(
        !tree.is_leaf_root(),
        "Tree should have a branch root after inserting 5 items"
    );

    // Now insert a new item that should traverse through the branch node
    // to reach the appropriate leaf
    let old_value = tree.insert(3, "updated_value_3".to_string());

    // Verify the insertion worked correctly
    assert_eq!(
        old_value,
        Some("value_3".to_string()),
        "Should return old value when updating existing key"
    );
    assert_eq!(
        tree.get(&3),
        Some(&"updated_value_3".to_string()),
        "Updated value should be retrievable"
    );

    // Insert a completely new key that should also traverse through branch
    let old_value = tree.insert(6, "value_6".to_string());
    assert_eq!(old_value, None, "Should return None when inserting new key");
    assert_eq!(
        tree.get(&6),
        Some(&"value_6".to_string()),
        "New value should be retrievable"
    );

    // Verify tree structure is still valid
    assert!(
        tree.check_invariants(),
        "Tree should maintain invariants after insertions through branch"
    );
    assert_eq!(tree.len(), 6, "Tree should have 6 items");
}

// ============================================================================
// STEP 6: LEAF SPLITTING WITH PARENT UPDATES
// ============================================================================

#[test]
fn test_leaf_split_updates_parent_branch() {
    let mut tree = BPlusTreeMap::new(4).unwrap();

    // First, create a tree with a branch root by inserting enough items
    // to cause a leaf split and root promotion
    for i in 1..=5 {
        tree.insert(i, format!("value_{}", i));
    }

    // Verify we have a branch root
    assert!(!tree.is_leaf_root(), "Tree should have a branch root");
    let initial_leaf_count = tree.leaf_count();

    // Now insert enough items to cause another leaf split
    // This should update the parent branch node with a new separator key
    for i in 6..=9 {
        tree.insert(i, format!("value_{}", i));
    }

    // Verify that a leaf split occurred (more leaf nodes)
    let final_leaf_count = tree.leaf_count();
    assert!(
        final_leaf_count > initial_leaf_count,
        "Should have more leaf nodes after causing another split. Initial: {}, Final: {}",
        initial_leaf_count,
        final_leaf_count
    );

    // Verify all items are still accessible
    for i in 1..=9 {
        assert_eq!(
            tree.get(&i),
            Some(&format!("value_{}", i)),
            "Item {} should be accessible after leaf split",
            i
        );
    }

    // Verify tree structure is still valid
    assert!(
        tree.check_invariants(),
        "Tree should maintain invariants after leaf split with parent update"
    );
    assert_eq!(tree.len(), 9, "Tree should have 9 items");

    // Verify that the range query works correctly across the split
    let range: Vec<_> = tree.items_range(Some(&1), Some(&10)).collect();
    assert_eq!(range.len(), 9, "Range query should return all 9 items");

    // Verify items are in sorted order
    for i in 0..range.len() - 1 {
        assert!(
            range[i].0 < range[i + 1].0,
            "Items should be in sorted order: {:?} should be < {:?}",
            range[i].0,
            range[i + 1].0
        );
    }
}

// ============================================================================
// STEP 7: ROOT PROMOTION (LEAF TO BRANCH)
// ============================================================================

#[test]
fn test_root_promotion_leaf_to_branch() {
    let mut tree = BPlusTreeMap::new(4).unwrap();

    // Initially, the tree should have a leaf root
    assert!(
        tree.is_leaf_root(),
        "New tree should start with a leaf root"
    );
    assert_eq!(tree.leaf_count(), 1, "New tree should have exactly 1 leaf");

    // Insert items one by one and track when root promotion occurs
    tree.insert(1, "value_1".to_string());
    assert!(
        tree.is_leaf_root(),
        "Tree should still have leaf root after 1 item"
    );

    tree.insert(2, "value_2".to_string());
    assert!(
        tree.is_leaf_root(),
        "Tree should still have leaf root after 2 items"
    );

    tree.insert(3, "value_3".to_string());
    assert!(
        tree.is_leaf_root(),
        "Tree should still have leaf root after 3 items"
    );

    tree.insert(4, "value_4".to_string());
    assert!(
        tree.is_leaf_root(),
        "Tree should still have leaf root after 4 items (at capacity)"
    );

    // This insertion should cause the root leaf to split and promote to a branch
    tree.insert(5, "value_5".to_string());
    assert!(
        !tree.is_leaf_root(),
        "Tree should have branch root after exceeding leaf capacity"
    );
    assert!(
        tree.leaf_count() >= 2,
        "Tree should have at least 2 leaves after root split"
    );

    // Verify all data is still accessible after root promotion
    for i in 1..=5 {
        assert_eq!(
            tree.get(&i),
            Some(&format!("value_{}", i)),
            "Item {} should be accessible after root promotion",
            i
        );
    }

    // Verify tree structure is valid
    assert!(
        tree.check_invariants(),
        "Tree should maintain invariants after root promotion"
    );
    assert_eq!(tree.len(), 5, "Tree should have 5 items");

    // Verify that operations still work correctly after root promotion
    let old_value = tree.insert(3, "updated_value_3".to_string());
    assert_eq!(
        old_value,
        Some("value_3".to_string()),
        "Should be able to update existing key"
    );

    let new_value = tree.insert(6, "value_6".to_string());
    assert_eq!(new_value, None, "Should be able to insert new key");

    // Verify range queries work across the promoted structure
    let range: Vec<_> = tree.items_range(Some(&1), Some(&7)).collect();
    assert_eq!(range.len(), 6, "Range query should return all 6 items");

    // Verify items are in sorted order
    for i in 0..range.len() - 1 {
        assert!(
            range[i].0 < range[i + 1].0,
            "Items should be in sorted order after root promotion"
        );
    }
}

// ============================================================================
// STEP 8: BRANCHNODE SPLITTING
// ============================================================================

#[test]
fn test_branch_node_split_creates_new_level() {
    let mut tree = BPlusTreeMap::new(4).unwrap();

    // Insert enough items to create a multi-level tree structure
    // This should eventually cause branch node splits
    let mut items_inserted = 0;
    let initial_leaf_count = tree.leaf_count();

    // Insert items until we have a significant tree structure
    // With capacity 4, we need enough items to fill multiple branch nodes
    for i in 1..=25 {
        tree.insert(i, format!("value_{}", i));
        items_inserted += 1;

        // Verify invariants are maintained after each insertion
        assert!(
            tree.check_invariants(),
            "Tree invariants should be maintained after inserting item {}",
            i
        );
    }

    // Verify we have more leaf nodes than we started with
    let final_leaf_count = tree.leaf_count();
    assert!(
        final_leaf_count > initial_leaf_count,
        "Should have more leaf nodes after inserting {} items. Initial: {}, Final: {}",
        items_inserted,
        initial_leaf_count,
        final_leaf_count
    );

    // Verify we have a branch root (not a leaf root)
    assert!(
        !tree.is_leaf_root(),
        "Tree should have a branch root after inserting {} items",
        items_inserted
    );

    // Verify all items are still accessible
    for i in 1..=25 {
        assert_eq!(
            tree.get(&i),
            Some(&format!("value_{}", i)),
            "Item {} should be accessible in multi-level tree",
            i
        );
    }

    // Verify tree structure and size
    assert_eq!(tree.len(), 25, "Tree should have 25 items");

    // Verify range queries work correctly across the complex structure
    let range: Vec<_> = tree.items_range(Some(&1), Some(&26)).collect();
    assert_eq!(range.len(), 25, "Range query should return all 25 items");

    // Verify items are in sorted order
    for i in 0..range.len() - 1 {
        assert!(
            range[i].0 < range[i + 1].0,
            "Items should be in sorted order in multi-level tree"
        );
    }

    // Test some additional operations to ensure the tree is fully functional
    let old_value = tree.insert(13, "updated_value_13".to_string());
    assert_eq!(
        old_value,
        Some("value_13".to_string()),
        "Should be able to update existing key in multi-level tree"
    );

    let new_value = tree.insert(26, "value_26".to_string());
    assert_eq!(
        new_value, None,
        "Should be able to insert new key in multi-level tree"
    );

    // Final invariant check
    assert!(
        tree.check_invariants(),
        "Tree should maintain invariants after all operations in multi-level structure"
    );
}

// ============================================================================
// STEP 9: COMPREHENSIVE INSERT TESTING
// ============================================================================

#[test]
fn test_comprehensive_insert_scenarios() {
    // Test with different branching factors
    for capacity in [4, 8, 16] {
        println!(
            "Testing comprehensive insert scenarios with capacity {}",
            capacity
        );

        let mut tree = BPlusTreeMap::new(capacity).unwrap();

        // Test 1: Sequential insertion (ascending order)
        for i in 1..=50 {
            tree.insert(i, format!("seq_value_{}", i));
            assert!(
                tree.check_invariants(),
                "Sequential insert {} failed invariants with capacity {}",
                i,
                capacity
            );
        }

        // Verify all sequential items are accessible
        for i in 1..=50 {
            assert_eq!(
                tree.get(&i),
                Some(&format!("seq_value_{}", i)),
                "Sequential item {} not found with capacity {}",
                i,
                capacity
            );
        }

        // Test 2: Reverse insertion (descending order)
        let mut tree2 = BPlusTreeMap::new(capacity).unwrap();
        for i in (1..=50).rev() {
            tree2.insert(i, format!("rev_value_{}", i));
            assert!(
                tree2.check_invariants(),
                "Reverse insert {} failed invariants with capacity {}",
                i,
                capacity
            );
        }

        // Verify all reverse items are accessible
        for i in 1..=50 {
            assert_eq!(
                tree2.get(&i),
                Some(&format!("rev_value_{}", i)),
                "Reverse item {} not found with capacity {}",
                i,
                capacity
            );
        }

        // Test 3: Random-ish insertion (deterministic pattern)
        let mut tree3 = BPlusTreeMap::new(capacity).unwrap();
        let mut keys: Vec<i32> = (1..=50).collect();
        // Simple deterministic shuffle for reproducibility
        for i in 0..keys.len() {
            let j = (i * 17) % keys.len();
            keys.swap(i, j);
        }

        for key in &keys {
            tree3.insert(*key, format!("rand_value_{}", key));
            assert!(
                tree3.check_invariants(),
                "Random insert {} failed invariants with capacity {}",
                key,
                capacity
            );
        }

        // Verify all random items are accessible
        for i in 1..=50 {
            assert_eq!(
                tree3.get(&i),
                Some(&format!("rand_value_{}", i)),
                "Random item {} not found with capacity {}",
                i,
                capacity
            );
        }

        // Test 4: Multiple updates to same keys
        for i in 1..=25 {
            let old_value = tree3.insert(i, format!("updated_value_{}", i));
            assert_eq!(
                old_value,
                Some(format!("rand_value_{}", i)),
                "Update {} should return old value with capacity {}",
                i,
                capacity
            );
            assert!(
                tree3.check_invariants(),
                "Update {} failed invariants with capacity {}",
                i,
                capacity
            );
        }

        // Verify final state
        assert_eq!(tree.len(), 50, "Sequential tree should have 50 items");
        assert_eq!(tree2.len(), 50, "Reverse tree should have 50 items");
        assert_eq!(tree3.len(), 50, "Random tree should have 50 items");

        // Test range queries on all trees
        let range1: Vec<_> = tree.items_range(Some(&10), Some(&20)).collect();
        let range2: Vec<_> = tree2.items_range(Some(&10), Some(&20)).collect();
        let range3: Vec<_> = tree3.items_range(Some(&10), Some(&20)).collect();

        assert_eq!(
            range1.len(),
            10,
            "Sequential tree range should have 10 items"
        );
        assert_eq!(range2.len(), 10, "Reverse tree range should have 10 items");
        assert_eq!(range3.len(), 10, "Random tree range should have 10 items");

        println!(
            "✓ Capacity {} passed all comprehensive insert tests",
            capacity
        );
    }
}

// ============================================================================
// ARENA-BASED ALLOCATION TESTS
// ============================================================================

#[test]
fn test_leaf_allocation() {
    let mut tree = BPlusTreeMap::<i32, String>::new(4).unwrap();

    // Create some leaf nodes to allocate
    let leaf1 = bplustree::LeafNode::new(4);
    let leaf2 = bplustree::LeafNode::new(4);
    let leaf3 = bplustree::LeafNode::new(4);

    // Test allocation
    let id1 = tree.allocate_leaf(leaf1);
    let id2 = tree.allocate_leaf(leaf2);
    let id3 = tree.allocate_leaf(leaf3);

    // IDs should be sequential starting from 1 (since 0 is the initial arena leaf)
    assert_eq!(id1, 1, "First allocation should get ID 1");
    assert_eq!(id2, 2, "Second allocation should get ID 2");
    assert_eq!(id3, 3, "Third allocation should get ID 3");

    // Test retrieval
    assert!(
        tree.get_leaf(id1).is_some(),
        "Should be able to retrieve leaf 1"
    );
    assert!(
        tree.get_leaf(id2).is_some(),
        "Should be able to retrieve leaf 2"
    );
    assert!(
        tree.get_leaf(id3).is_some(),
        "Should be able to retrieve leaf 3"
    );
    assert!(
        tree.get_leaf(999).is_none(),
        "Should return None for invalid ID"
    );

    // Test mutable retrieval
    assert!(
        tree.get_leaf_mut(id1).is_some(),
        "Should be able to retrieve mutable leaf 1"
    );
    assert!(
        tree.get_leaf_mut(id2).is_some(),
        "Should be able to retrieve mutable leaf 2"
    );
    assert!(
        tree.get_leaf_mut(id3).is_some(),
        "Should be able to retrieve mutable leaf 3"
    );
    assert!(
        tree.get_leaf_mut(999).is_none(),
        "Should return None for invalid mutable ID"
    );

    // Test deallocation
    let deallocated = tree.deallocate_leaf(id2);
    assert!(deallocated.is_some(), "Should be able to deallocate leaf 2");
    assert!(
        tree.get_leaf(id2).is_none(),
        "Deallocated leaf should not be retrievable"
    );

    // Test reuse of deallocated ID
    let leaf4 = bplustree::LeafNode::new(4);
    let id4 = tree.allocate_leaf(leaf4);
    assert_eq!(id4, id2, "Should reuse the deallocated ID");
    assert!(
        tree.get_leaf(id4).is_some(),
        "Should be able to retrieve reused leaf"
    );

    // Test double deallocation
    let deallocated_again = tree.deallocate_leaf(id4); // Use id4 since id2 was reused
    assert!(
        deallocated_again.is_some(),
        "Should be able to deallocate the reused leaf"
    );

    // Now test actual double deallocation
    let double_deallocated = tree.deallocate_leaf(id4);
    assert!(
        double_deallocated.is_none(),
        "Double deallocation should return None"
    );
}

#[test]
fn test_leaf_linked_list() {
    let mut tree = BPlusTreeMap::<i32, String>::new(4).unwrap();

    // Create three leaf nodes
    let leaf1 = bplustree::LeafNode::new(4);
    let leaf2 = bplustree::LeafNode::new(4);
    let leaf3 = bplustree::LeafNode::new(4);

    let id1 = tree.allocate_leaf(leaf1);
    let id2 = tree.allocate_leaf(leaf2);
    let id3 = tree.allocate_leaf(leaf3);

    // Initially, all next pointers should be NULL
    assert_eq!(tree.get_leaf_next(id1), None, "Initial next should be None");
    assert_eq!(tree.get_leaf_next(id2), None, "Initial next should be None");
    assert_eq!(tree.get_leaf_next(id3), None, "Initial next should be None");

    // Set up a linked list: id1 -> id2 -> id3 -> NULL
    assert!(
        tree.set_leaf_next(id1, id2),
        "Should be able to set next pointer"
    );
    assert!(
        tree.set_leaf_next(id2, id3),
        "Should be able to set next pointer"
    );

    // Verify the linked list structure
    assert_eq!(
        tree.get_leaf_next(id1),
        Some(id2),
        "id1 should point to id2"
    );
    assert_eq!(
        tree.get_leaf_next(id2),
        Some(id3),
        "id2 should point to id3"
    );
    assert_eq!(tree.get_leaf_next(id3), None, "id3 should point to NULL");

    // Test setting next to NULL_NODE explicitly
    assert!(
        tree.set_leaf_next(id2, bplustree::NULL_NODE),
        "Should be able to set next to NULL"
    );
    assert_eq!(
        tree.get_leaf_next(id2),
        None,
        "id2 should now point to NULL"
    );

    // Test invalid operations
    assert!(
        !tree.set_leaf_next(999, id1),
        "Should fail to set next on invalid ID"
    );
    assert_eq!(
        tree.get_leaf_next(999),
        None,
        "Should return None for invalid ID"
    );

    // Restore the chain: id1 -> id2 -> id3 -> NULL
    assert!(
        tree.set_leaf_next(id2, id3),
        "Should be able to restore chain"
    );

    // Test circular reference (id3 -> id1)
    assert!(
        tree.set_leaf_next(id3, id1),
        "Should be able to create circular reference"
    );
    assert_eq!(
        tree.get_leaf_next(id3),
        Some(id1),
        "id3 should point to id1"
    );

    // Verify we can traverse the circular structure: id1 -> id2 -> id3 -> id1 (cycle)
    let mut current = Some(id1);
    let mut visited = std::collections::HashSet::new();
    let mut count = 0;

    while let Some(node_id) = current {
        if visited.contains(&node_id) || count > 10 {
            break; // Prevent infinite loop
        }
        visited.insert(node_id);
        current = tree.get_leaf_next(node_id);
        count += 1;
    }

    assert_eq!(
        count, 3,
        "Should visit exactly 3 nodes before hitting the cycle"
    );
    assert!(visited.contains(&id1), "Should have visited id1");
    assert!(visited.contains(&id2), "Should have visited id2");
    assert!(visited.contains(&id3), "Should have visited id3");
}

// TODO: Implement test_leaf_node_creation
// TODO: Implement test_leaf_node_insert
// TODO: Implement test_leaf_node_full
// TODO: Implement test_leaf_find_position
// TODO: Implement test_branch_node_creation
// TODO: Implement test_find_child_index
// TODO: Implement test_branch_node_split
// TODO: Implement test_leaf_can_donate
// TODO: Implement test_branch_can_donate
// TODO: Implement test_leaf_borrow_from_left
// TODO: Implement test_leaf_borrow_from_right
// TODO: Implement test_branch_borrow_from_left
// TODO: Implement test_branch_borrow_from_right
// TODO: Implement test_leaf_merge_with_right
// TODO: Implement test_branch_merge_with_right

// ============================================================================
// TRANSLATED PYTHON TESTS - Capacity Validation
// ============================================================================

#[test]
fn test_invalid_capacity_error() {
    // Test that creating a tree with capacity < 4 should return error
    let result = BPlusTreeMap::<i32, String>::new(3);
    assert!(result.is_err());

    // Test that capacity 4 works
    let _tree = BPlusTreeMap::<i32, String>::new(4).unwrap();
}

// ============================================================================
// STRESS TESTS - These will be implemented after basic functionality works
// ============================================================================

// ============================================================================
// NEW TESTS - Dict-like API
// ============================================================================

#[test]
fn test_key_error_on_missing_key() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    tree.insert(1, "one".to_string());

    // Test that get_item returns error for missing keys
    let result = tree.get_item(&2);
    assert_eq!(result, Err(BPlusTreeError::KeyNotFound));

    // Existing key should work
    let result = tree.get_item(&1);
    assert_eq!(result, Ok(&"one".to_string()));
}

#[test]
fn test_remove_nonexistent_key_raises_error() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    tree.insert(1, "one".to_string());
    tree.insert(2, "two".to_string());

    // Try to remove non-existent key
    let result = tree.remove_item(&3);
    assert_eq!(result, Err(BPlusTreeError::KeyNotFound));

    // Tree should be unchanged
    assert_eq!(tree.len(), 2);
    assert_eq!(tree.get(&1), Some(&"one".to_string()));
    assert_eq!(tree.get(&2), Some(&"two".to_string()));
}

// ============================================================================
// NEW TESTS - Iterator Support
// ============================================================================

#[test]
fn test_iterate_empty_tree() {
    let tree = BPlusTreeMap::<i32, String>::new(4).unwrap();
    let items: Vec<_> = tree.items().collect();
    assert_eq!(items, vec![]);
}

#[test]
fn test_iterate_single_item() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    tree.insert(5, "value5".to_string());

    let items: Vec<_> = tree.items().collect();
    assert_eq!(items, vec![(&5, &"value5".to_string())]);
}

#[test]
fn test_iterate_multiple_items_single_leaf() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    tree.insert(1, "value1".to_string());
    tree.insert(3, "value3".to_string());
    tree.insert(2, "value2".to_string());
    tree.insert(4, "value4".to_string());

    let items: Vec<_> = tree.items().collect();
    assert_eq!(
        items,
        vec![
            (&1, &"value1".to_string()),
            (&2, &"value2".to_string()),
            (&3, &"value3".to_string()),
            (&4, &"value4".to_string())
        ]
    );
}

#[test]
fn test_iterate_multiple_leaves() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    // Insert enough to create multiple leaves
    for i in 1..=9 {
        tree.insert(i, format!("value{}", i));
    }

    let items: Vec<_> = tree.items().collect();
    // Check that we have the right number of items and they're in order
    assert_eq!(items.len(), 9);
    for (i, (key, value)) in items.iter().enumerate() {
        let expected_key = i + 1;
        let expected_value = format!("value{}", expected_key);
        assert_eq!(**key, expected_key);
        assert_eq!(**value, expected_value);
    }
}

#[test]
fn test_keys_iterator() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    tree.insert(1, "one".to_string());
    tree.insert(2, "two".to_string());
    tree.insert(3, "three".to_string());

    let keys: Vec<_> = tree.keys().collect();
    assert_eq!(keys, vec![&1, &2, &3]);
}

#[test]
fn test_values_iterator() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    tree.insert(1, "one".to_string());
    tree.insert(2, "two".to_string());
    tree.insert(3, "three".to_string());

    let values: Vec<_> = tree.values().collect();
    assert_eq!(
        values,
        vec![&"one".to_string(), &"two".to_string(), &"three".to_string()]
    );
}

// ============================================================================
// NEW TESTS - Range Iteration
// ============================================================================

#[test]
fn test_iterate_from_key() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    for i in 0..10 {
        tree.insert(i, format!("value{}", i));
    }

    let items: Vec<_> = tree.items_range(Some(&5), None).collect();
    assert_eq!(items.len(), 5); // keys 5, 6, 7, 8, 9
    for (i, (key, value)) in items.iter().enumerate() {
        let expected_key = i + 5;
        let expected_value = format!("value{}", expected_key);
        assert_eq!(**key, expected_key);
        assert_eq!(**value, expected_value);
    }
}

#[test]
fn test_iterate_until_key() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    for i in 0..10 {
        tree.insert(i, format!("value{}", i));
    }

    let items: Vec<_> = tree.items_range(None, Some(&5)).collect();
    assert_eq!(items.len(), 5); // keys 0, 1, 2, 3, 4
    for (i, (key, value)) in items.iter().enumerate() {
        let expected_key = i;
        let expected_value = format!("value{}", expected_key);
        assert_eq!(**key, expected_key);
        assert_eq!(**value, expected_value);
    }
}

#[test]
fn test_iterate_range() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    for i in 0..20 {
        tree.insert(i, format!("value{}", i));
    }

    let items: Vec<_> = tree.items_range(Some(&5), Some(&15)).collect();
    assert_eq!(items.len(), 10); // keys 5, 6, 7, 8, 9, 10, 11, 12, 13, 14
    for (i, (key, value)) in items.iter().enumerate() {
        let expected_key = i + 5;
        let expected_value = format!("value{}", expected_key);
        assert_eq!(**key, expected_key);
        assert_eq!(**value, expected_value);
    }
}

#[test]
fn test_iterate_from_nonexistent_key() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    for i in [1, 3, 5, 7, 9] {
        tree.insert(i, format!("value{}", i));
    }

    // Start from 4 (doesn't exist, should start from 5)
    let items: Vec<_> = tree.items_range(Some(&4), None).collect();
    assert_eq!(items.len(), 3); // keys 5, 7, 9
    assert_eq!(*items[0].0, 5);
    assert_eq!(*items[1].0, 7);
    assert_eq!(*items[2].0, 9);
}

#[test]
fn test_iterate_empty_range() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    for i in 0..10 {
        tree.insert(i, format!("value{}", i));
    }

    // Start after end (invalid range)
    let items: Vec<_> = tree.items_range(Some(&7), Some(&3)).collect();
    assert_eq!(items, vec![]);
}

// ============================================================================
// NEW TESTS - Invariant Checking
// ============================================================================

#[test]
fn test_invariants_empty_tree() {
    let tree = BPlusTreeMap::<i32, String>::new(4).unwrap();
    assert!(tree.check_invariants());
}

#[test]
fn test_invariants_single_item() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    tree.insert(1, "one".to_string());
    assert!(tree.check_invariants());
}

#[test]
fn test_invariants_after_split() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    // Insert enough items to force a split
    for i in 1..=5 {
        tree.insert(i, format!("value{}", i));
        assert!(
            tree.check_invariants(),
            "Invariants violated after inserting {}",
            i
        );
    }
}

#[test]
fn test_invariants_after_many_operations() {
    let mut tree = BPlusTreeMap::new(4).unwrap();

    // Insert many items
    for i in 0..20 {
        tree.insert(i, format!("value{}", i));
        assert!(
            tree.check_invariants(),
            "Invariants violated after inserting {}",
            i
        );
    }

    // Remove some items
    for i in [1, 5, 10, 15] {
        tree.remove(&i);
        assert!(
            tree.check_invariants(),
            "Invariants violated after removing {}",
            i
        );
    }

    // Insert more items
    for i in 20..30 {
        tree.insert(i, format!("value{}", i));
        assert!(
            tree.check_invariants(),
            "Invariants violated after inserting {}",
            i
        );
    }
}

// ============================================================================
// NEW TESTS - Edge Cases and Stress Tests
// ============================================================================

#[test]
fn test_large_capacity_edge_cases() {
    let mut tree = BPlusTreeMap::new(64).unwrap(); // Large capacity

    // Fill up close to capacity
    for i in 0..60 {
        tree.insert(i, format!("value_{}", i));
        assert!(
            tree.check_invariants(),
            "Invariants violated after inserting {}",
            i
        );
    }

    assert!(tree.is_leaf_root(), "Should still be single-level tree");

    // Delete most items to test underflow handling
    for i in (0..60).step_by(2) {
        // Delete every other item
        tree.remove(&i);
        assert!(tree.check_invariants(), "Delete {} broke invariants", i);
    }

    // Add items back to test growth
    for i in 60..70 {
        tree.insert(i, format!("new_value_{}", i));
        assert!(tree.check_invariants(), "Insert {} broke invariants", i);
    }
}

#[test]
fn test_capacity_boundary_conditions() {
    for capacity in [4, 8, 16, 32] {
        let mut tree = BPlusTreeMap::new(capacity).unwrap();

        // Fill exactly to capacity
        for i in 0..capacity {
            tree.insert(i, format!("value_{}", i));
            assert!(
                tree.check_invariants(),
                "Tree at capacity {} should be valid",
                capacity
            );
        }

        // Add one more to trigger split
        tree.insert(capacity, format!("value_{}", capacity));
        assert!(
            tree.check_invariants(),
            "Tree after split at capacity {} should be valid",
            capacity
        );

        // Delete back to capacity
        tree.remove(&capacity);
        assert!(
            tree.check_invariants(),
            "Tree after delete at capacity {} should be valid",
            capacity
        );
    }
}

#[test]
fn test_sequential_vs_random_patterns() {
    // Test sequential insertion
    let mut tree = BPlusTreeMap::new(8).unwrap();
    for i in 0..50 {
        tree.insert(i, format!("value_{}", i));
        assert!(
            tree.check_invariants(),
            "Sequential insert {} broke invariants",
            i
        );
    }

    // Test reverse insertion
    let mut tree = BPlusTreeMap::new(8).unwrap();
    for i in (0..50).rev() {
        tree.insert(i, format!("value_{}", i));
        assert!(
            tree.check_invariants(),
            "Reverse insert {} broke invariants",
            i
        );
    }

    // Test random-ish insertion (using a deterministic pattern)
    let mut tree = BPlusTreeMap::new(8).unwrap();
    let mut keys: Vec<i32> = (0..50).collect();
    // Simple deterministic shuffle
    for i in 0..keys.len() {
        let j = (i * 17) % keys.len(); // Simple pseudo-random pattern
        keys.swap(i, j);
    }

    for key in keys {
        tree.insert(key, format!("value_{}", key));
        assert!(
            tree.check_invariants(),
            "Random insert {} broke invariants",
            key
        );
    }
}

// ============================================================================
// NEW TESTS - Deep Tree and Recursive Insertion
// ============================================================================

#[test]
fn test_deep_tree_insertion() {
    let mut tree = BPlusTreeMap::new(4).unwrap(); // Small capacity to force deep tree

    // Insert enough items to create a deep tree (3+ levels)
    for i in 0..100 {
        tree.insert(i, format!("value_{}", i));
        assert!(
            tree.check_invariants(),
            "Invariants violated after inserting {}",
            i
        );
    }

    // Verify all items are retrievable
    for i in 0..100 {
        assert_eq!(tree.get(&i), Some(&format!("value_{}", i)));
    }

    // Tree should have multiple levels
    assert!(!tree.is_leaf_root());
    assert!(tree.leaf_count() > 10); // Should have many leaves
}

#[test]
fn test_branch_node_splitting() {
    let mut tree = BPlusTreeMap::new(4).unwrap();

    // Insert items in a pattern that will force branch node splits
    for i in 0..50 {
        tree.insert(i, format!("value_{}", i));
        assert!(
            tree.check_invariants(),
            "Invariants violated after inserting {}",
            i
        );
    }

    // Verify the tree structure is correct
    assert!(!tree.is_leaf_root());
    assert_eq!(tree.len(), 50);

    // All items should be retrievable
    for i in 0..50 {
        assert_eq!(tree.get(&i), Some(&format!("value_{}", i)));
    }
}

#[test]
fn test_multi_level_splits() {
    let mut tree = BPlusTreeMap::new(5).unwrap(); // Slightly larger capacity

    // Insert enough items to force multiple levels of splits
    for i in 0..200 {
        tree.insert(i, format!("value_{}", i));
        // Check invariants every 10 insertions to catch issues early
        if i % 10 == 0 {
            assert!(
                tree.check_invariants(),
                "Invariants violated after inserting {}",
                i
            );
        }
    }

    // Final invariant check
    assert!(tree.check_invariants());
    assert_eq!(tree.len(), 200);

    // Verify all items are still accessible
    for i in 0..200 {
        assert_eq!(tree.get(&i), Some(&format!("value_{}", i)));
    }
}

#[test]
fn test_large_sequential_insertion() {
    let mut tree = BPlusTreeMap::new(8).unwrap();

    // Insert a large number of sequential items
    for i in 0..1000 {
        tree.insert(i, i * 2);
        // Check invariants periodically
        if i % 100 == 0 {
            assert!(
                tree.check_invariants(),
                "Invariants violated after inserting {}",
                i
            );
        }
    }

    // Final checks
    assert!(tree.check_invariants());
    assert_eq!(tree.len(), 1000);

    // Spot check some values
    assert_eq!(tree.get(&0), Some(&0));
    assert_eq!(tree.get(&500), Some(&1000));
    assert_eq!(tree.get(&999), Some(&1998));
}

#[test]
fn test_reverse_order_insertion() {
    let mut tree = BPlusTreeMap::new(6).unwrap();

    // Insert items in reverse order to test different split patterns
    for i in (0..100).rev() {
        tree.insert(i, format!("value_{}", i));
        if i % 20 == 0 {
            assert!(
                tree.check_invariants(),
                "Invariants violated after inserting {}",
                i
            );
        }
    }

    // Final checks
    assert!(tree.check_invariants());
    assert_eq!(tree.len(), 100);

    // Verify all items are accessible
    for i in 0..100 {
        assert_eq!(tree.get(&i), Some(&format!("value_{}", i)));
    }
}

// ============================================================================
// NEW TESTS - Advanced Deletion and Rebalancing
// ============================================================================

#[test]
fn test_delete_until_empty() {
    let mut tree = BPlusTreeMap::new(4).unwrap();

    // Insert items
    for i in 0..20 {
        tree.insert(i, format!("value_{}", i));
    }
    assert!(tree.check_invariants());
    assert_eq!(tree.len(), 20);

    // Delete all items
    for i in 0..20 {
        let removed = tree.remove(&i);
        assert_eq!(removed, Some(format!("value_{}", i)));
        if !tree.check_invariants() {
            println!(
                "Tree state after removing {}: len={}, is_leaf_root={}",
                i,
                tree.len(),
                tree.is_leaf_root()
            );
            panic!("Invariants violated after removing {}", i);
        }
    }

    // Tree should be empty
    assert_eq!(tree.len(), 0);
    assert!(tree.is_empty());
    assert!(tree.check_invariants());
}

#[test]
fn test_root_collapse() {
    let mut tree = BPlusTreeMap::new(4).unwrap();

    // Create a tree with branch root
    for i in 0..10 {
        tree.insert(i, format!("value_{}", i));
    }
    assert!(!tree.is_leaf_root());

    // Delete most items to force root collapse
    for i in 0..9 {
        tree.remove(&i);
        assert!(
            tree.check_invariants(),
            "Invariants violated after removing {}",
            i
        );
    }

    // Should still have one item and maintain invariants
    assert_eq!(tree.len(), 1);
    assert_eq!(tree.get(&9), Some(&"value_9".to_string()));
    assert!(tree.check_invariants());
}

#[test]
fn test_alternating_insert_delete() {
    let mut tree = BPlusTreeMap::new(6).unwrap();

    // Alternating pattern of insert and delete
    for i in 0..50 {
        tree.insert(i, format!("value_{}", i));
        if i > 0 && i % 3 == 0 {
            tree.remove(&(i - 2));
        }
        assert!(
            tree.check_invariants(),
            "Invariants violated at iteration {}",
            i
        );
    }

    // Final check
    assert!(tree.check_invariants());
}

#[test]
fn test_delete_from_deep_tree() {
    let mut tree = BPlusTreeMap::new(4).unwrap();

    // Create a deep tree
    for i in 0..100 {
        tree.insert(i, i * 2);
    }
    assert!(tree.check_invariants());
    assert!(!tree.is_leaf_root());

    // Delete items from various parts of the tree
    let to_delete = [5, 25, 50, 75, 95, 10, 30, 60, 80];
    for &key in &to_delete {
        let removed = tree.remove(&key);
        assert_eq!(removed, Some(key * 2));
        assert!(
            tree.check_invariants(),
            "Invariants violated after removing {}",
            key
        );
    }

    // Verify remaining items are correct
    for i in 0..100 {
        if to_delete.contains(&i) {
            assert_eq!(tree.get(&i), None);
        } else {
            assert_eq!(tree.get(&i), Some(&(i * 2)));
        }
    }
}

#[test]
fn test_delete_all_but_one() {
    let mut tree = BPlusTreeMap::new(5).unwrap();

    // Insert many items
    for i in 0..50 {
        tree.insert(i, format!("value_{}", i));
    }
    if !tree.check_invariants() {
        println!("Final tree structure:");
        tree.print_node_chain();
        panic!("Final invariants check failed");
    }

    // Delete all but the last item
    for i in 0..49 {
        tree.remove(&i);
        if !tree.check_invariants() {
            println!("Invariants failed after removing {}", i);
            tree.print_node_chain();
            panic!("Invariants violated after removing {}", i);
        }
    }

    // Should have exactly one item left
    assert_eq!(tree.len(), 1);
    assert_eq!(tree.get(&49), Some(&"value_49".to_string()));
    assert!(tree.check_invariants());
}

// ============================================================================
// NEW TESTS - Borrowing and Merging (Future Implementation)
// ============================================================================

#[test]
fn test_massive_insertion_deletion_cycle() {
    let mut tree = BPlusTreeMap::new(8).unwrap();

    // Insert a large number of items
    for i in 0..500 {
        tree.insert(i, format!("value_{}", i));
        if i % 50 == 0 {
            assert!(
                tree.check_invariants(),
                "Invariants violated after inserting {}",
                i
            );
        }
    }

    // Delete every other item
    for i in (0..500).step_by(2) {
        tree.remove(&i);
        if i % 50 == 0 {
            assert!(
                tree.check_invariants(),
                "Invariants violated after removing {}",
                i
            );
        }
    }

    // Verify remaining items
    for i in 0..500 {
        if i % 2 == 0 {
            assert_eq!(tree.get(&i), None);
        } else {
            assert_eq!(tree.get(&i), Some(&format!("value_{}", i)));
        }
    }

    assert!(tree.check_invariants());
    assert_eq!(tree.len(), 250);
}

#[test]
fn test_random_deletion_pattern() {
    let mut tree = BPlusTreeMap::new(6).unwrap();

    // Insert items
    for i in 0..100 {
        tree.insert(i, i * 3);
    }
    assert!(tree.check_invariants());

    // Delete in a pseudo-random pattern
    let delete_pattern = [13, 7, 42, 89, 3, 67, 21, 95, 8, 56, 34, 78, 12, 45, 90];
    for &key in &delete_pattern {
        if key < 100 {
            tree.remove(&key);
            assert!(
                tree.check_invariants(),
                "Invariants violated after removing {}",
                key
            );
        }
    }

    // Verify correct items remain
    for i in 0..100 {
        if delete_pattern.contains(&i) {
            assert_eq!(tree.get(&i), None);
        } else {
            assert_eq!(tree.get(&i), Some(&(i * 3)));
        }
    }
}

#[test]
fn test_delete_from_minimal_tree() {
    let mut tree = BPlusTreeMap::new(4).unwrap(); // Minimal capacity

    // Create a tree with just enough items to have a branch root
    for i in 1..=5 {
        tree.insert(i, format!("value_{}", i));
    }
    assert!(!tree.is_leaf_root());
    assert!(tree.check_invariants());

    // Delete items one by one and verify invariants
    for i in 1..=5 {
        tree.remove(&i);
        assert!(
            tree.check_invariants(),
            "Invariants violated after removing {}",
            i
        );
    }

    assert!(tree.is_empty());
    assert!(tree.is_leaf_root());
}

#[test]
fn test_stress_deletion_with_invariants() {
    let mut tree = BPlusTreeMap::new(5).unwrap();

    // Build a moderately complex tree
    for i in 0..200 {
        tree.insert(i, i.to_string());
    }
    assert!(tree.check_invariants());

    // Delete items in chunks and verify invariants after each chunk
    for chunk in (0..200).collect::<Vec<_>>().chunks(10) {
        for &item in chunk {
            tree.remove(&item);
        }
        assert!(
            tree.check_invariants(),
            "Invariants violated after deleting chunk {:?}",
            chunk
        );
    }

    assert!(tree.is_empty());
}

// ============================================================================
// NEW TESTS - Comprehensive Edge Cases and Stress Tests
// ============================================================================

#[test]
fn test_single_key_operations() {
    let mut tree = BPlusTreeMap::new(4).unwrap();

    // Test with single key
    tree.insert(42, "answer".to_string());
    assert_eq!(tree.len(), 1);
    assert_eq!(tree.get(&42), Some(&"answer".to_string()));
    assert!(tree.check_invariants());

    // Update the single key
    let old = tree.insert(42, "new_answer".to_string());
    assert_eq!(old, Some("answer".to_string()));
    assert_eq!(tree.len(), 1);
    assert!(tree.check_invariants());

    // Remove the single key
    let removed = tree.remove(&42);
    assert_eq!(removed, Some("new_answer".to_string()));
    assert_eq!(tree.len(), 0);
    assert!(tree.is_empty());
    assert!(tree.check_invariants());
}

#[test]
fn test_duplicate_key_handling() {
    let mut tree = BPlusTreeMap::new(6).unwrap();

    // Insert same key multiple times
    assert_eq!(tree.insert(1, "first".to_string()), None);
    assert_eq!(
        tree.insert(1, "second".to_string()),
        Some("first".to_string())
    );
    assert_eq!(
        tree.insert(1, "third".to_string()),
        Some("second".to_string())
    );

    assert_eq!(tree.len(), 1);
    assert_eq!(tree.get(&1), Some(&"third".to_string()));
    assert!(tree.check_invariants());
}

#[test]
fn test_extreme_capacity_values() {
    // Test minimum capacity
    let mut tree = BPlusTreeMap::new(4).unwrap();
    for i in 0..20 {
        tree.insert(i, i * 2);
        assert!(
            tree.check_invariants(),
            "Invariants violated at capacity 4, item {}",
            i
        );
    }

    // Test larger capacity
    let mut tree = BPlusTreeMap::new(100).unwrap();
    for i in 0..200 {
        tree.insert(i, i * 3);
        if i % 25 == 0 {
            assert!(
                tree.check_invariants(),
                "Invariants violated at capacity 100, item {}",
                i
            );
        }
    }
}

#[test]
fn test_pathological_deletion_patterns() {
    let mut tree = BPlusTreeMap::new(5).unwrap();

    // Insert items
    for i in 0..50 {
        tree.insert(i, format!("value_{}", i));
    }
    assert!(tree.check_invariants());

    // Delete every 3rd item
    for i in (0..50).step_by(3) {
        tree.remove(&i);
        assert!(
            tree.check_invariants(),
            "Invariants violated after removing every 3rd: {}",
            i
        );
    }

    // Delete every 7th remaining item
    for i in (0..50).step_by(7) {
        tree.remove(&i);
        assert!(
            tree.check_invariants(),
            "Invariants violated after removing every 7th: {}",
            i
        );
    }
}

#[test]
fn test_clustered_key_patterns() {
    let mut tree = BPlusTreeMap::new(6).unwrap();

    // Insert clustered keys (0-9, 100-109, 200-209, etc.)
    for cluster in 0..10 {
        for i in 0..10 {
            let key = cluster * 100 + i;
            tree.insert(key, format!("cluster_{}_{}", cluster, i));
            if key % 50 == 0 {
                assert!(
                    tree.check_invariants(),
                    "Invariants violated at clustered key {}",
                    key
                );
            }
        }
    }

    // Delete entire clusters
    for cluster in [2, 5, 8] {
        for i in 0..10 {
            let key = cluster * 100 + i;
            tree.remove(&key);
        }
        assert!(
            tree.check_invariants(),
            "Invariants violated after removing cluster {}",
            cluster
        );
    }
}

#[test]
fn test_interleaved_operations() {
    let mut tree = BPlusTreeMap::new(7).unwrap();

    // Interleave insertions, deletions, and updates
    for i in 0..100 {
        // Insert
        tree.insert(i, format!("value_{}", i));

        // Update a previous key
        if i > 10 {
            tree.insert(i - 10, format!("updated_{}", i - 10));
        }

        // Delete an even older key
        if i > 20 {
            tree.remove(&(i - 20));
        }

        // Check invariants on every iteration
        assert!(
            tree.check_invariants(),
            "Invariants violated at iteration {}",
            i
        );
    }
}

#[test]
fn test_clear_and_reuse() {
    let mut tree = BPlusTreeMap::new(5).unwrap();

    // Populate the tree
    for i in 0..50 {
        tree.insert(i, format!("value_{}", i));
    }
    assert_eq!(tree.len(), 50);
    assert!(tree.check_invariants());

    // Clear the tree
    tree.clear();
    assert_eq!(tree.len(), 0);
    assert!(tree.is_empty());
    assert!(tree.check_invariants());

    // Reuse the tree
    for i in 100..150 {
        tree.insert(i, format!("new_value_{}", i));
    }
    assert_eq!(tree.len(), 50);
    assert!(tree.check_invariants());
}

#[test]
fn test_range_query_edge_cases() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    for i in 0..20 {
        tree.insert(i, format!("value{}", i));
    }

    // Range that covers the entire tree
    let all_items: Vec<_> = tree.items_range(None, None).collect();
    assert_eq!(all_items.len(), 20);

    // Range that starts before the first key
    let from_neg: Vec<_> = tree.items_range(Some(&-5), Some(&5)).collect();
    assert_eq!(from_neg.len(), 5); // 0, 1, 2, 3, 4

    // Range that ends after the last key
    let to_far: Vec<_> = tree.items_range(Some(&15), Some(&100)).collect();
    assert_eq!(to_far.len(), 5); // 15, 16, 17, 18, 19

    // Range with no items
    let no_items: Vec<_> = tree.items_range(Some(&25), Some(&30)).collect();
    assert_eq!(no_items.len(), 0);
}

#[test]
fn test_range_syntax_support() {
    let mut tree = BPlusTreeMap::new(16).unwrap();
    for i in 0..10 {
        tree.insert(i, format!("value{}", i));
    }

    // Test different range syntaxes
    let range1: Vec<_> = tree.range(3..7).map(|(k, v)| (*k, v.clone())).collect();
    assert_eq!(
        range1,
        vec![
            (3, "value3".to_string()),
            (4, "value4".to_string()),
            (5, "value5".to_string()),
            (6, "value6".to_string())
        ]
    );

    let range2: Vec<_> = tree.range(3..=7).map(|(k, v)| (*k, v.clone())).collect();
    assert_eq!(
        range2,
        vec![
            (3, "value3".to_string()),
            (4, "value4".to_string()),
            (5, "value5".to_string()),
            (6, "value6".to_string()),
            (7, "value7".to_string())
        ]
    );

    let range3: Vec<_> = tree.range(5..).map(|(k, _v)| *k).collect();
    assert_eq!(range3, vec![5, 6, 7, 8, 9]);

    let range4: Vec<_> = tree.range(..5).map(|(k, _v)| *k).collect();
    assert_eq!(range4, vec![0, 1, 2, 3, 4]);

    let range5: Vec<_> = tree.range(..).map(|(k, _v)| *k).collect();
    assert_eq!(range5, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn test_range_syntax_with_excluded_bounds() {
    let mut tree = BPlusTreeMap::new(16).unwrap();
    for i in 0..10 {
        tree.insert(i, format!("value{}", i));
    }

    // Test excluded start bound
    let range_excluded_start: Vec<_> = tree
        .range((std::ops::Bound::Excluded(3), std::ops::Bound::Included(7)))
        .map(|(k, _)| *k)
        .collect();
    assert_eq!(range_excluded_start, vec![4, 5, 6, 7]);

    // Test excluded end bound
    let range_excluded_end: Vec<_> = tree
        .range((std::ops::Bound::Included(3), std::ops::Bound::Excluded(7)))
        .map(|(k, _)| *k)
        .collect();
    assert_eq!(range_excluded_end, vec![3, 4, 5, 6]);

    // Test both excluded
    let range_both_excluded: Vec<_> = tree
        .range((std::ops::Bound::Excluded(3), std::ops::Bound::Excluded(7)))
        .map(|(k, _)| *k)
        .collect();
    assert_eq!(range_both_excluded, vec![4, 5, 6]);
}

#[test]
fn test_first_and_last() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    assert_eq!(tree.first(), None);
    assert_eq!(tree.last(), None);

    tree.insert(10, "ten".to_string());
    assert_eq!(tree.first(), Some((&10, &"ten".to_string())));
    assert_eq!(tree.last(), Some((&10, &"ten".to_string())));

    tree.insert(5, "five".to_string());
    tree.insert(15, "fifteen".to_string());
    assert_eq!(tree.first(), Some((&5, &"five".to_string())));
    assert_eq!(tree.last(), Some((&15, &"fifteen".to_string())));
}

#[test]
fn test_get_mut() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    tree.insert(1, "one".to_string());
    tree.insert(2, "two".to_string());

    // Get a mutable reference and modify the value
    if let Some(value) = tree.get_mut(&1) {
        *value = "ONE".to_string();
    }

    assert_eq!(tree.get(&1), Some(&"ONE".to_string()));
    assert_eq!(tree.get(&2), Some(&"two".to_string()));

    // Test with a non-existent key
    assert_eq!(tree.get_mut(&3), None);
}

#[test]
fn test_arena_consistency() {
    let mut tree = BPlusTreeMap::new(4).unwrap();

    // Insert items
    for i in 0..50 {
        tree.insert(i, format!("value_{}", i));
    }

    // Check consistency
    assert!(tree.check_invariants_detailed().is_ok());

    // Delete some items
    for i in (0..50).step_by(3) {
        tree.remove(&i);
    }

    // Check consistency again
    assert!(tree.check_invariants_detailed().is_ok());

    // Count nodes
    let (tree_leaves, tree_branches) = tree.count_nodes_in_tree();
    let leaf_stats = tree.leaf_arena_stats();
    let branch_stats = tree.branch_arena_stats();

    assert_eq!(tree_leaves, leaf_stats.allocated_count);
    assert_eq!(tree_branches, branch_stats.allocated_count);
}

#[test]
fn test_leaf_linked_list_completeness() {
    let mut tree = BPlusTreeMap::new(5).unwrap();

    // Insert items
    for i in 0..100 {
        tree.insert(i, i.to_string());
    }
    assert!(tree.check_invariants_detailed().is_ok());

    // Delete items
    for i in (0..100).step_by(4) {
        tree.remove(&i);
    }
    assert!(tree.check_invariants_detailed().is_ok());
}

#[test]
fn test_try_insert_and_remove() {
    let mut tree = BPlusTreeMap::new(4).unwrap();

    // Successful insert
    assert!(tree.try_insert(1, "one".to_string()).is_ok());
    assert_eq!(tree.get(&1), Some(&"one".to_string()));

    // Successful remove
    assert!(tree.try_remove(&1).is_ok());
    assert_eq!(tree.get(&1), None);

    // Failed remove
    assert!(tree.try_remove(&1).is_err());
}

#[test]
fn test_batch_insert() {
    let mut tree = BPlusTreeMap::new(4).unwrap();

    // Successful batch insert
    let items = vec![(1, "one"), (2, "two"), (3, "three")];
    let result = tree.batch_insert(items.iter().map(|(k, v)| (*k, v.to_string())).collect());
    assert!(result.is_ok());
    assert_eq!(tree.len(), 3);

    // Batch insert with duplicates
    let items2 = vec![(4, "four"), (2, "TWO"), (5, "five")];
    let result2 = tree.batch_insert(items2.iter().map(|(k, v)| (*k, v.to_string())).collect());
    assert!(result2.is_ok());
    assert_eq!(tree.len(), 5);
    assert_eq!(tree.get(&2), Some(&"TWO".to_string()));
}

#[test]
fn test_get_many() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    tree.insert(1, "one".to_string());
    tree.insert(2, "two".to_string());
    tree.insert(3, "three".to_string());

    // Successful get_many
    let keys = vec![1, 3];
    let result = tree.get_many(&keys);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![&"one".to_string(), &"three".to_string()]
    );

    // get_many with missing key
    let keys2 = vec![1, 4, 2];
    let result2 = tree.get_many(&keys2);
    assert!(result2.is_err());
}

#[test]
fn test_validate_for_operation() {
    let mut tree = BPlusTreeMap::new(4).unwrap();
    assert!(tree.validate_for_operation("initial").is_ok());

    tree.insert(1, "one".to_string());
    assert!(tree.validate_for_operation("after insert").is_ok());
}
