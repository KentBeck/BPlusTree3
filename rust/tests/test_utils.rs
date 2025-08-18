#![allow(dead_code)] // Allow unused utility functions for future tests

/// Comprehensive test utilities to eliminate massive test duplication
/// This module provides reusable patterns for adversarial testing and common operations
use bplustree::BPlusTreeMap;

// ============================================================================
// TREE CREATION UTILITIES - Replace 185 instances of BPlusTreeMap::new()
// ============================================================================

/// Standard tree with capacity 4 (most common pattern)
pub fn create_tree_4() -> BPlusTreeMap<i32, String> {
    BPlusTreeMap::new(4).expect("Failed to create tree with capacity 4")
}

/// Standard tree with capacity 4 for integer keys and values
pub fn create_tree_4_int() -> BPlusTreeMap<i32, i32> {
    BPlusTreeMap::new(4).expect("Failed to create integer tree with capacity 4")
}

/// Standard tree with capacity 5 (for odd capacity testing)
pub fn create_tree_5() -> BPlusTreeMap<i32, String> {
    BPlusTreeMap::new(5).expect("Failed to create tree with capacity 5")
}

/// Standard tree with capacity 6 (for specific testing scenarios)
pub fn create_tree_6() -> BPlusTreeMap<i32, String> {
    BPlusTreeMap::new(6).expect("Failed to create tree with capacity 6")
}

/// Generic tree creation with custom capacity
pub fn create_tree_capacity(capacity: usize) -> BPlusTreeMap<i32, String> {
    BPlusTreeMap::new(capacity).expect(&format!("Failed to create tree with capacity {}", capacity))
}

/// Generic integer tree creation with custom capacity
pub fn create_tree_capacity_int(capacity: usize) -> BPlusTreeMap<i32, i32> {
    BPlusTreeMap::new(capacity).expect(&format!(
        "Failed to create integer tree with capacity {}",
        capacity
    ))
}

// ============================================================================
// DATA POPULATION UTILITIES - Replace 176 for-loop patterns
// ============================================================================

/// Insert sequential data 0..count with string values
pub fn insert_sequential_range(tree: &mut BPlusTreeMap<i32, String>, count: usize) {
    for i in 0..count {
        tree.insert(i as i32, format!("value_{}", i));
    }
}

/// Insert sequential data 0..count with integer values
pub fn insert_sequential_range_int(tree: &mut BPlusTreeMap<i32, i32>, count: usize) {
    for i in 0..count {
        tree.insert(i as i32, i as i32);
    }
}

/// Insert data with custom key multiplier (common pattern: i * multiplier)
pub fn insert_with_multiplier(tree: &mut BPlusTreeMap<i32, String>, count: usize, multiplier: i32) {
    for i in 0..count {
        let key = (i as i32) * multiplier;
        tree.insert(key, format!("value_{}", i));
    }
}

/// Insert data with custom key multiplier for integer trees
pub fn insert_with_multiplier_int(
    tree: &mut BPlusTreeMap<i32, i32>,
    count: usize,
    multiplier: i32,
) {
    for i in 0..count {
        let key = (i as i32) * multiplier;
        tree.insert(key, i as i32);
    }
}

/// Insert data with offset and multiplier (key = offset + i * multiplier)
pub fn insert_with_offset_multiplier(
    tree: &mut BPlusTreeMap<i32, String>,
    count: usize,
    offset: i32,
    multiplier: i32,
) {
    for i in 0..count {
        let key = offset + (i as i32) * multiplier;
        tree.insert(key, format!("value_{}", i));
    }
}

/// Insert data with custom key and value functions
pub fn insert_with_custom_fn<F, G>(
    tree: &mut BPlusTreeMap<i32, String>,
    count: usize,
    key_fn: F,
    value_fn: G,
) where
    F: Fn(usize) -> i32,
    G: Fn(usize) -> String,
{
    for i in 0..count {
        let key = key_fn(i);
        let value = value_fn(i);
        tree.insert(key, value);
    }
}

/// Insert sequential data start..end with string values
pub fn insert_range(tree: &mut BPlusTreeMap<i32, String>, start: usize, end: usize) {
    for i in start..end {
        tree.insert(i as i32, format!("value_{}", i));
    }
}

/// Insert sequential data start..end with integer values
pub fn insert_range_int(tree: &mut BPlusTreeMap<i32, i32>, start: usize, end: usize) {
    for i in start..end {
        tree.insert(i as i32, i as i32);
    }
}

// ============================================================================
// COMBINED TREE CREATION AND POPULATION - Most common patterns
// ============================================================================

/// Create tree with capacity 4 and insert 0..count sequential data
pub fn create_tree_4_with_data(count: usize) -> BPlusTreeMap<i32, String> {
    let mut tree = create_tree_4();
    insert_sequential_range(&mut tree, count);
    tree
}

/// Create integer tree with capacity 4 and insert 0..count sequential data
pub fn create_tree_4_int_with_data(count: usize) -> BPlusTreeMap<i32, i32> {
    let mut tree = create_tree_4_int();
    insert_sequential_range_int(&mut tree, count);
    tree
}

/// Create tree with custom capacity and insert 0..count sequential data
pub fn create_tree_with_data(capacity: usize, count: usize) -> BPlusTreeMap<i32, String> {
    let mut tree = create_tree_capacity(capacity);
    insert_sequential_range(&mut tree, count);
    tree
}

/// Create integer tree with custom capacity and insert 0..count sequential data
pub fn create_tree_int_with_data(capacity: usize, count: usize) -> BPlusTreeMap<i32, i32> {
    let mut tree = create_tree_capacity_int(capacity);
    insert_sequential_range_int(&mut tree, count);
    tree
}

/// Create tree with data using multiplier pattern (common: i * 2, i * 3, i * 5, i * 10)
pub fn create_tree_4_with_multiplier(count: usize, multiplier: i32) -> BPlusTreeMap<i32, String> {
    let mut tree = create_tree_4();
    insert_with_multiplier(&mut tree, count, multiplier);
    tree
}

// ============================================================================
// INVARIANT CHECKING UTILITIES - Replace 44 instances
// ============================================================================

/// Standard invariant check with panic on failure
pub fn assert_invariants(tree: &BPlusTreeMap<i32, String>, context: &str) {
    if let Err(e) = tree.check_invariants_detailed() {
        panic!("Invariant violation in {}: {}", context, e);
    }
}

/// Standard invariant check for integer trees
pub fn assert_invariants_int(tree: &BPlusTreeMap<i32, i32>, context: &str) {
    if let Err(e) = tree.check_invariants_detailed() {
        panic!("Invariant violation in {}: {}", context, e);
    }
}

/// Comprehensive tree validation including ordering
pub fn assert_full_validation(tree: &BPlusTreeMap<i32, String>, context: &str) {
    assert_invariants(tree, context);
    verify_ordering(tree);
}

/// Comprehensive tree validation for integer trees
pub fn assert_full_validation_int(tree: &BPlusTreeMap<i32, i32>, context: &str) {
    assert_invariants_int(tree, context);
    verify_ordering_int(tree);
}

// ============================================================================
// ADVERSARIAL ATTACK PATTERNS - Common deletion patterns
// ============================================================================

/// Execute deletion range attack (delete items from start to end)
pub fn deletion_range_attack(tree: &mut BPlusTreeMap<i32, String>, start: usize, end: usize) {
    for i in start..end {
        tree.remove(&(i as i32));
    }
}

/// Execute deletion range attack for integer trees
pub fn deletion_range_attack_int(tree: &mut BPlusTreeMap<i32, i32>, start: usize, end: usize) {
    for i in start..end {
        tree.remove(&(i as i32));
    }
}

/// Execute alternating deletion pattern (delete every other item)
pub fn alternating_deletion_attack(tree: &mut BPlusTreeMap<i32, String>, count: usize) {
    for i in (0..count).step_by(2) {
        tree.remove(&(i as i32));
    }
}

/// Execute a stress test cycle with automatic invariant checking
pub fn stress_test_cycle<F>(tree: &mut BPlusTreeMap<i32, String>, cycles: usize, attack_fn: F)
where
    F: Fn(&mut BPlusTreeMap<i32, String>, usize),
{
    for cycle in 0..cycles {
        attack_fn(tree, cycle);

        // Unified invariant checking with context
        if let Err(e) = tree.check_invariants_detailed() {
            panic!("ATTACK SUCCESSFUL at cycle {}: {}", cycle, e);
        }
    }
}

/// Standard arena exhaustion attack pattern
pub fn arena_exhaustion_attack(tree: &mut BPlusTreeMap<i32, String>, cycle: usize) {
    let cycle_i32 = cycle as i32;

    // Fill tree to create many nodes
    for i in 0..100 {
        tree.insert(cycle_i32 * 1000 + i, format!("v{}-{}", cycle, i));
    }

    // Delete most items to free nodes
    for i in 0..95 {
        tree.remove(&(cycle_i32 * 1000 + i));
    }

    println!(
        "Cycle {}: Free leaves={}, Free branches={}",
        cycle,
        tree.free_leaf_count(),
        tree.branch_arena_stats().free_count
    );
}

/// Standard fragmentation attack pattern
pub fn fragmentation_attack(tree: &mut BPlusTreeMap<i32, String>, base_key: i32) {
    // Insert in a pattern that creates and frees nodes in specific order
    for i in 0..500 {
        tree.insert(base_key + i * 10, format!("fragmented-{}", i));
    }

    // Delete every other item
    for i in (0..500).step_by(2) {
        tree.remove(&(base_key + i * 10));
    }

    // Reinsert to reuse freed slots
    for i in 0..250 {
        tree.insert(base_key + i * 10 + 5, format!("reused-{}", i * 1000));
    }
}

/// Deep tree creation attack pattern
pub fn deep_tree_attack(tree: &mut BPlusTreeMap<i32, i32>, capacity: usize) {
    let mut key = 0;
    for level in 0..5 {
        let level_u32 = u32::try_from(level).expect("Level should fit in u32");
        let count = capacity.pow(level_u32);
        for _ in 0..count * 10 {
            tree.insert(key, key);
            key += 100; // Large gaps to force deep structure
        }
    }
}

/// Alternating operations attack pattern
pub fn alternating_operations_attack(tree: &mut BPlusTreeMap<i32, String>, round: usize) {
    // Delete from left side
    let left_key = (round * 6) as i32;
    if tree.contains_key(&left_key) {
        tree.remove(&left_key);
    }

    // Insert in middle
    let mid_key = 30 + round as i32;
    tree.insert(mid_key * 2 + 1, format!("mid{}", round));

    // Delete from right side
    let right_key = 118 - (round * 6) as i32;
    if tree.contains_key(&right_key) {
        tree.remove(&right_key);
    }
}

// ============================================================================
// VERIFICATION UTILITIES
// ============================================================================

/// Verify tree ordering after operations
pub fn verify_ordering(tree: &BPlusTreeMap<i32, String>) {
    let items: Vec<_> = tree.items().collect();
    for i in 1..items.len() {
        if items[i - 1].0 >= items[i].0 {
            panic!("Items out of order after operations!");
        }
    }
}

/// Verify tree ordering for integer trees
pub fn verify_ordering_int(tree: &BPlusTreeMap<i32, i32>) {
    let items: Vec<_> = tree.items().collect();
    for i in 1..items.len() {
        if items[i - 1].0 >= items[i].0 {
            panic!("Items out of order after operations!");
        }
    }
}

/// Verify tree has expected number of items
pub fn verify_item_count(tree: &BPlusTreeMap<i32, String>, expected: usize, context: &str) {
    let actual = tree.len();
    if actual != expected {
        panic!(
            "Item count mismatch in {}: Expected {} items, got {}",
            context, expected, actual
        );
    }
}

/// Verify tree has expected number of items (integer version)
pub fn verify_item_count_int(tree: &BPlusTreeMap<i32, i32>, expected: usize, context: &str) {
    let actual = tree.len();
    if actual != expected {
        panic!(
            "Item count mismatch in {}: Expected {} items, got {}",
            context, expected, actual
        );
    }
}

// ============================================================================
// SPECIALIZED TEST SETUPS
// ============================================================================

/// Create a tree with specific structure for branch testing
pub fn create_branch_test_tree(capacity: usize) -> BPlusTreeMap<i32, String> {
    let mut tree = create_tree_capacity(capacity);

    // Build specific tree structure where branches are at minimum
    let keys = vec![
        10, 20, 30, 40, 15, 25, 35, 45, 12, 18, 22, 28, 32, 38, 42, 48,
    ];
    for key in keys {
        tree.insert(key, format!("v{}", key));
    }

    // Delete strategically to make siblings exactly at minimum
    for key in vec![18, 28, 38, 48] {
        tree.remove(&key);
    }

    tree
}

/// Standard setup for concurrent access simulation
pub fn setup_concurrent_simulation() -> (Vec<(bool, i32)>, Vec<(bool, i32)>) {
    let thread1_ops = vec![
        (true, 1),
        (true, 3),
        (true, 5),
        (false, 3),
        (true, 7),
        (false, 1),
    ];
    let thread2_ops = vec![
        (true, 2),
        (true, 4),
        (false, 2),
        (true, 6),
        (true, 8),
        (false, 4),
    ];
    (thread1_ops, thread2_ops)
}

/// Execute interleaved operations for concurrent simulation
pub fn execute_interleaved_ops(
    tree: &mut BPlusTreeMap<i32, String>,
    thread1_ops: &[(bool, i32)],
    thread2_ops: &[(bool, i32)],
) {
    for i in 0..thread1_ops.len() {
        // Thread 1 operation
        let (is_insert, key) = thread1_ops[i];
        if is_insert {
            tree.insert(key * 10, format!("t1-{}", key));
        } else {
            tree.remove(&(key * 10));
        }

        // Check invariants after each operation
        assert_invariants(tree, &format!("after thread1 op {}", i));

        // Thread 2 operation
        let (is_insert, key) = thread2_ops[i];
        if is_insert {
            tree.insert(key * 10 + 1, format!("t2-{}", key));
        } else {
            tree.remove(&(key * 10 + 1));
        }

        // Check invariants after each operation
        assert_invariants(tree, &format!("after thread2 op {}", i));
    }
}

// ============================================================================
// DEBUGGING AND STATISTICS
// ============================================================================

/// Print tree statistics for debugging
pub fn print_tree_stats(tree: &BPlusTreeMap<i32, String>, label: &str) {
    let leaf_stats = tree.leaf_arena_stats();
    let branch_stats = tree.branch_arena_stats();
    println!(
        "{}: {} items, Free leaves={}, Free branches={}",
        label,
        tree.len(),
        leaf_stats.free_count,
        branch_stats.free_count
    );
    println!("Leaf sizes: {:?}", tree.leaf_sizes());
}

/// Print tree statistics for integer trees
pub fn print_tree_stats_int(tree: &BPlusTreeMap<i32, i32>, label: &str) {
    let leaf_stats = tree.leaf_arena_stats();
    let branch_stats = tree.branch_arena_stats();
    println!(
        "{}: {} items, Free leaves={}, Free branches={}",
        label,
        tree.len(),
        leaf_stats.free_count,
        branch_stats.free_count
    );
    println!("Leaf sizes: {:?}", tree.leaf_sizes());
}

// ============================================================================
// LEGACY COMPATIBILITY - Keep existing test function names working
// ============================================================================

/// Legacy compatibility - create attack tree
pub fn create_attack_tree(capacity: usize) -> BPlusTreeMap<i32, String> {
    create_tree_capacity(capacity)
}

/// Legacy compatibility - create simple tree
pub fn create_simple_tree(capacity: usize) -> BPlusTreeMap<i32, i32> {
    create_tree_capacity_int(capacity)
}

/// Legacy compatibility - populate tree with sequential data
pub fn populate_sequential(tree: &mut BPlusTreeMap<i32, String>, count: usize) {
    insert_sequential_range(tree, count);
}

/// Legacy compatibility - populate tree with sequential integer data
pub fn populate_sequential_int(tree: &mut BPlusTreeMap<i32, i32>, count: usize) {
    insert_sequential_range_int(tree, count);
}

/// Legacy compatibility - populate tree with sequential integer data where value = key * 10
pub fn populate_sequential_int_x10(tree: &mut BPlusTreeMap<i32, i32>, count: usize) {
    for i in 0..count {
        tree.insert(i as i32, (i as i32) * 10);
    }
}

/// Legacy compatibility - verify attack failed
pub fn assert_attack_failed(tree: &BPlusTreeMap<i32, String>, context: &str) {
    assert_invariants(tree, context);
}

/// Legacy compatibility - verify attack failed for integer trees
pub fn assert_attack_failed_int(tree: &BPlusTreeMap<i32, i32>, context: &str) {
    assert_invariants_int(tree, context);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utilities_basic_functionality() {
        let mut tree = create_tree_4();
        insert_sequential_range(&mut tree, 10);

        assert_eq!(tree.len(), 10);
        verify_ordering(&tree);
        assert_invariants(&tree, "basic functionality test");
    }

    #[test]
    fn test_stress_cycle_utility() {
        let mut tree = create_tree_4();

        // Test that stress_test_cycle works correctly
        stress_test_cycle(&mut tree, 5, |tree, cycle| {
            tree.insert(cycle as i32, format!("cycle_{}", cycle));
        });

        assert_eq!(tree.len(), 5);
    }

    #[test]
    fn test_combined_creation_utilities() {
        let tree = create_tree_4_with_data(20);
        assert_eq!(tree.len(), 20);
        assert_full_validation(&tree, "combined creation test");
    }

    #[test]
    fn test_attack_patterns() {
        let mut tree = create_tree_4_with_data(50);

        // Test deletion range attack
        deletion_range_attack(&mut tree, 10, 40);
        assert_eq!(tree.len(), 20);
        assert_full_validation(&tree, "deletion range attack");
    }
}
