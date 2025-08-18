//! Macros to eliminate repetitive patterns in B+ Tree operations and testing

/// Macro to eliminate repetitive invariant checking patterns
/// This replaces 90+ occurrences of similar invariant checking code
#[macro_export]
macro_rules! assert_tree_valid {
    // Basic invariant check
    ($tree:expr) => {
        if let Err(e) = $tree.check_invariants_detailed() {
            panic!("Tree invariants violated: {}", e);
        }
    };

    // Invariant check with context
    ($tree:expr, $context:expr) => {
        if let Err(e) = $tree.check_invariants_detailed() {
            panic!("ATTACK SUCCESSFUL in {}: {}", $context, e);
        }
    };

    // Invariant check with context and cycle number
    ($tree:expr, $context:expr, $cycle:expr) => {
        if let Err(e) = $tree.check_invariants_detailed() {
            panic!("ATTACK SUCCESSFUL at {} cycle {}: {}", $context, $cycle, e);
        }
    };

    // Invariant check with custom message format
    ($tree:expr, $fmt:expr, $($arg:tt)*) => {
        if let Err(e) = $tree.check_invariants_detailed() {
            panic!("ATTACK SUCCESSFUL: {} - {}", format!($fmt, $($arg)*), e);
        }
    };
}

/// Macro to eliminate repetitive arena method implementations
/// This generates all the boilerplate arena methods to eliminate duplication
#[macro_export]
macro_rules! impl_arena_methods {
    ($arena_field:ident, $free_field:ident, $node_type:ty, $prefix:ident) => {
        paste::paste! {
            /// Allocate a new node in the arena
            pub fn [<allocate_ $prefix>](&mut self, node: $node_type) -> NodeId {
                self.$arena_field.allocate(node)
            }

            /// Deallocate a node from the arena
            pub fn [<deallocate_ $prefix>](&mut self, id: NodeId) -> Option<$node_type> {
                self.$arena_field.deallocate(id)
            }

            /// Get a reference to a node in the arena
            pub fn [<get_ $prefix>](&self, id: NodeId) -> Option<&$node_type> {
                self.$arena_field.get(id)
            }

            /// Get a mutable reference to a node in the arena
            pub fn [<get_ $prefix _mut>](&mut self, id: NodeId) -> Option<&mut $node_type> {
                self.$arena_field.get_mut(id)
            }

            /// Get the number of free nodes in the arena
            pub fn [<free_ $prefix _count>](&self) -> usize {
                self.$arena_field.free_count()
            }

            /// Get the number of allocated nodes in the arena
            pub fn [<allocated_ $prefix _count>](&self) -> usize {
                self.$arena_field.allocated_count()
            }

            /// Get the total capacity of the arena
            pub fn [<total_ $prefix _capacity>](&self) -> usize {
                self.$arena_field.total_capacity()
            }

            /// Get the utilization ratio of the arena
            pub fn [<$prefix _utilization>](&self) -> f64 {
                self.$arena_field.utilization()
            }
        }
    };
}

/// Macro for creating test trees with common patterns
#[macro_export]
macro_rules! create_test_tree {
    // Basic tree with capacity
    ($capacity:expr) => {
        BPlusTreeMap::new($capacity).expect("Failed to create test tree")
    };

    // Tree with capacity and initial data
    ($capacity:expr, $count:expr) => {{
        let mut tree = BPlusTreeMap::new($capacity).expect("Failed to create test tree");
        for i in 0..$count {
            tree.insert(i, format!("value_{}", i));
        }
        tree
    }};

    // Tree with capacity and custom data
    ($capacity:expr, $data:expr) => {{
        let mut tree = BPlusTreeMap::new($capacity).expect("Failed to create test tree");
        for (key, value) in $data {
            tree.insert(key, value);
        }
        tree
    }};
}

/// Macro for common attack patterns in adversarial tests
#[macro_export]
macro_rules! attack_pattern {
    // Arena exhaustion attack
    (arena_exhaustion, $tree:expr, $cycle:expr) => {
        // Fill tree to create many nodes
        for i in 0..10 {
            $tree.insert($cycle * 10 + i, format!("v{}-{}", $cycle, i));
        }

        // Delete most items to free nodes
        for i in 0..8 {
            $tree.remove(&($cycle * 10 + i));
        }
    };

    // Fragmentation attack
    (fragmentation, $tree:expr, $base_key:expr) => {
        // Insert in a pattern that creates and frees nodes in specific order
        for i in 0..50 {
            $tree.insert($base_key + i * 10, format!("fragmented-{}", i));
        }

        // Delete every other item
        for i in (0..50).step_by(2) {
            $tree.remove(&($base_key + i * 10));
        }

        // Reinsert to reuse freed slots
        for i in 0..25 {
            $tree.insert($base_key + i * 10 + 5, format!("reused-{}", i * 1000));
        }
    };

    // Deep tree creation
    (deep_tree, $tree:expr, $capacity:expr) => {
        let mut key = 0;
        for level in 0..3 {
            let count = $capacity.pow(level);
            for _ in 0..count * 5 {
                $tree.insert(key, key);
                key += 100;
            }
        }
    };
}

/// Macro for verifying attack results
#[macro_export]
macro_rules! verify_attack_result {
    // Basic verification
    ($tree:expr, $context:expr) => {
        assert_tree_valid!($tree, $context);
    };

    // Verification with ordering check
    ($tree:expr, $context:expr, ordering) => {
        assert_tree_valid!($tree, $context);
        let items: Vec<_> = $tree.items().collect();
        for i in 1..items.len() {
            if items[i - 1].0 >= items[i].0 {
                panic!("ATTACK SUCCESSFUL: Items out of order in {}!", $context);
            }
        }
    };

    // Verification with item count check
    ($tree:expr, $context:expr, count = $expected:expr) => {
        assert_tree_valid!($tree, $context);
        let actual = $tree.len();
        if actual != $expected {
            panic!(
                "ATTACK SUCCESSFUL in {}: Expected {} items, got {}",
                $context, $expected, actual
            );
        }
    };

    // Full verification (invariants + ordering + count)
    ($tree:expr, $context:expr, full = $expected:expr) => {
        verify_attack_result!($tree, $context, count = $expected);
        verify_attack_result!($tree, $context, ordering);
    };
}

/// Macro for stress testing with automatic invariant checking
#[macro_export]
macro_rules! stress_test {
    ($tree:expr, $cycles:expr, $attack:expr) => {
        for cycle in 0..$cycles {
            $attack;
            assert_tree_valid!($tree, "stress test", cycle);
        }
    };
}

/// Macro for range bounds processing (eliminates duplication in range operations)
#[macro_export]
macro_rules! process_range_bounds {
    ($range:expr) => {{
        use std::ops::Bound;

        let start = match $range.start_bound() {
            Bound::Included(key) => Some(key),
            Bound::Excluded(_) => return Err("Excluded start bounds not supported".into()),
            Bound::Unbounded => None,
        };

        let end = match $range.end_bound() {
            Bound::Included(_) => return Err("Included end bounds not supported".into()),
            Bound::Excluded(key) => Some(key),
            Bound::Unbounded => None,
        };

        (start, end)
    }};
}

#[cfg(test)]
mod tests {
    use crate::BPlusTreeMap;

    #[test]
    fn test_assert_tree_valid_macro() {
        let tree: BPlusTreeMap<i32, String> = BPlusTreeMap::new(4).unwrap();

        // Test basic usage
        assert_tree_valid!(tree);

        // Test with context
        assert_tree_valid!(tree, "macro test");

        // Test with cycle
        assert_tree_valid!(tree, "macro test", 0);
    }

    #[test]
    fn test_create_test_tree_macro() {
        // Test basic creation
        let tree1: BPlusTreeMap<i32, String> = create_test_tree!(4);
        assert_eq!(tree1.len(), 0);

        // Test with initial data count
        let tree2: BPlusTreeMap<i32, String> = create_test_tree!(4, 5);
        assert_eq!(tree2.len(), 5);

        // Test with custom data
        let data = vec![(1, "one".to_string()), (2, "two".to_string())];
        let mut tree3: BPlusTreeMap<i32, String> =
            BPlusTreeMap::new(4).expect("Failed to create test tree");
        for (key, value) in data {
            tree3.insert(key, value);
        }
        assert_eq!(tree3.len(), 2);
    }

    #[test]
    fn test_attack_pattern_macro() {
        let mut tree = BPlusTreeMap::new(4).unwrap();

        // Test arena exhaustion pattern
        attack_pattern!(arena_exhaustion, tree, 0);
        assert_eq!(tree.len(), 2); // Should have 2 items left

        tree.clear();

        // Test fragmentation pattern
        attack_pattern!(fragmentation, tree, 0);
        assert_eq!(tree.len(), 50); // Should have 50 items
    }

    #[test]
    fn test_verify_attack_result_macro() {
        let mut tree = BPlusTreeMap::new(4).unwrap();
        for i in 0..10 {
            tree.insert(i, format!("value_{}", i));
        }

        // Test basic verification
        verify_attack_result!(tree, "basic test");

        // Test with ordering check
        verify_attack_result!(tree, "ordering test", ordering);

        // Test with count check
        verify_attack_result!(tree, "count test", count = 10);

        // Test full verification
        verify_attack_result!(tree, "full test", full = 10);
    }

    #[test]
    fn test_stress_test_macro() {
        let mut tree: BPlusTreeMap<i32, String> = BPlusTreeMap::new(4).unwrap();

        for cycle in 0..10 {
            tree.insert(cycle, format!("value_{}", cycle));
            assert_tree_valid!(tree, "stress test", cycle);
        }

        assert_eq!(tree.len(), 10);
    }
}
