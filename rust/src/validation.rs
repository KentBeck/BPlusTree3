//! Validation and debugging utilities for BPlusTreeMap.
//!
//! This module contains all validation methods, invariant checking, debugging utilities,
//! and test helpers for the B+ tree implementation.

use crate::types::{BPlusTreeMap, NodeRef, NodeId};
use crate::error::{BPlusTreeError, TreeResult};

// ============================================================================
// VALIDATION METHODS
// ============================================================================

impl<K: Ord + Clone, V: Clone> BPlusTreeMap<K, V> {
    /// Check if the tree maintains B+ tree invariants.
    /// Returns true if all invariants are satisfied.
    pub fn check_invariants(&self) -> bool {
        self.check_node_invariants(&self.root, None, None, true)
    }

    /// Check invariants with detailed error reporting.
    pub fn check_invariants_detailed(&self) -> Result<(), String> {
        // First check the tree structure invariants
        if !self.check_node_invariants(&self.root, None, None, true) {
            return Err("Tree invariants violated".to_string());
        }

        // Then check the linked list invariants
        self.check_linked_list_invariants()?;

        // Finally check arena-tree consistency
        self.check_arena_tree_consistency()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Check that arena allocation matches tree structure
    fn check_arena_tree_consistency(&self) -> TreeResult<()> {
        // Count nodes in the tree structure
        let (tree_leaf_count, tree_branch_count) = self.count_nodes_in_tree();

        // Get arena counts
        let leaf_stats = self.leaf_arena_stats();
        let branch_stats = self.branch_arena_stats();

        // Check leaf node consistency
        if tree_leaf_count != leaf_stats.allocated_count {
            return Err(BPlusTreeError::arena_error(
                "Leaf consistency check",
                &format!(
                    "{} in tree vs {} in arena",
                    tree_leaf_count, leaf_stats.allocated_count
                ),
            ));
        }

        // Check branch node consistency
        if tree_branch_count != branch_stats.allocated_count {
            return Err(BPlusTreeError::arena_error(
                "Branch consistency check",
                &format!(
                    "{} in tree vs {} in arena",
                    tree_branch_count, branch_stats.allocated_count
                ),
            ));
        }

        // Check that all leaf nodes in tree are reachable via linked list
        self.check_leaf_linked_list_completeness()?;

        Ok(())
    }

    /// Check that the leaf linked list is properly ordered and complete.
    fn check_linked_list_invariants(&self) -> Result<(), String> {
        // Use the iterator to get all keys
        let keys: Vec<&K> = self.keys().collect();

        // Check that keys are sorted
        for i in 1..keys.len() {
            if keys[i - 1] >= keys[i] {
                return Err(format!("Iterator returned unsorted keys at index {}", i));
            }
        }

        // Verify we got the right number of keys
        if keys.len() != self.len() {
            return Err(format!(
                "Iterator returned {} keys but tree has {} items",
                keys.len(),
                self.len()
            ));
        }

        Ok(())
    }

    /// Check that all leaf nodes in the tree are reachable via the linked list.
    fn check_leaf_linked_list_completeness(&self) -> TreeResult<()> {
        // Collect all leaf node IDs from the tree structure
        let mut tree_leaf_ids = Vec::new();
        self.collect_leaf_ids(&self.root, &mut tree_leaf_ids);
        tree_leaf_ids.sort();

        // Collect all leaf node IDs from the linked list
        let mut linked_list_ids = Vec::new();
        let mut current_id = self.get_first_leaf_id();
        while let Some(id) = current_id {
            linked_list_ids.push(id);
            if let Some(leaf) = self.get_leaf(id) {
                current_id = if leaf.next != crate::types::NULL_NODE {
                    Some(leaf.next)
                } else {
                    None
                };
            } else {
                break;
            }
        }
        linked_list_ids.sort();

        // Compare the two lists
        if tree_leaf_ids != linked_list_ids {
            return Err(BPlusTreeError::corrupted_tree(
                "Linked list",
                &format!(
                    "tree has {:?}, linked list has {:?}",
                    tree_leaf_ids, linked_list_ids
                ),
            ));
        }

        Ok(())
    }

    /// Collect all leaf node IDs from the tree structure.
    fn collect_leaf_ids(&self, node: &NodeRef<K, V>, ids: &mut Vec<NodeId>) {
        match node {
            NodeRef::Leaf(id, _) => ids.push(*id),
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.get_branch(*id) {
                    for child in &branch.children {
                        self.collect_leaf_ids(child, ids);
                    }
                }
            }
        }
    }

    /// Recursively check invariants for a node and its children.
    fn check_node_invariants(
        &self,
        node: &NodeRef<K, V>,
        min_key: Option<&K>,
        max_key: Option<&K>,
        _is_root: bool,
    ) -> bool {
        match node {
            NodeRef::Leaf(id, _) => {
                if let Some(leaf) = self.get_leaf(*id) {
                     // Check leaf invariants
                     if leaf.keys_len() != leaf.values_len() {
                         return false; // Keys and values must have same length
                     }

                     // Check that keys are sorted
                     for i in 1..leaf.keys_len() {
                         if let (Some(prev_key), Some(curr_key)) = (leaf.get_key(i - 1), leaf.get_key(i)) {
                             if prev_key >= curr_key {
                                 return false; // Keys must be in ascending order
                             }
                         }
                     }

                    // Check capacity constraints
                    if leaf.keys_len() > self.capacity {
                        return false; // Node exceeds capacity
                    }

                    // Check minimum occupancy
                    if !leaf.keys_is_empty() && leaf.is_underfull() {
                        // For root nodes, allow fewer keys only if it's the only node
                        if _is_root {
                            // Root leaf can have any number of keys >= 1
                            // (This is fine for leaf roots)
                        } else {
                            return false; // Non-root leaf is underfull
                        }
                    }

                    // Check key bounds
                    if let Some(min) = min_key {
                        if !leaf.keys_is_empty() {
                            if let Some(first_key) = leaf.first_key() {
                                if first_key < min {
                                    return false; // First key must be >= min_key
                                }
                            }
                        }
                    }
                    if let Some(max) = max_key {
                        if !leaf.keys_is_empty() {
                            if let Some(last_key) = leaf.last_key() {
                                if last_key >= max {
                                    return false; // Last key must be < max_key
                                }
                            }
                        }
                    }

                    true
                } else {
                    false // Missing arena leaf is invalid
                }
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.get_branch(*id) {
                    // Check branch invariants
                    if branch.keys.len() + 1 != branch.children.len() {
                        return false; // Branch must have one more child than keys
                    }

                    // Check that keys are sorted
                    for i in 1..branch.keys.len() {
                        if branch.keys[i - 1] >= branch.keys[i] {
                            return false; // Keys must be in ascending order
                        }
                    }

                    // Check capacity constraints
                    if branch.keys.len() > self.capacity {
                        return false; // Node exceeds capacity
                    }

                    // Check minimum occupancy
                    if !branch.keys.is_empty() && branch.is_underfull() {
                        if _is_root {
                            // Root branch can have any number of keys >= 1 (as long as it has children)
                            // The only requirement is that keys.len() + 1 == children.len()
                            // This is already checked above, so root branches are always valid
                        } else {
                            return false; // Non-root branch is underfull
                        }
                    }

                    // Check that branch has at least one child
                    if branch.children.is_empty() {
                        return false; // Branch must have at least one child
                    }

                    // Check children recursively
                    for (i, child) in branch.children.iter().enumerate() {
                        let child_min = if i == 0 {
                            min_key
                        } else {
                            Some(&branch.keys[i - 1])
                        };
                        let child_max = if i == branch.keys.len() {
                            max_key
                        } else {
                            Some(&branch.keys[i])
                        };

                        if !self.check_node_invariants(child, child_min, child_max, false) {
                            return false;
                        }
                    }

                    true
                } else {
                    false // Missing arena branch is invalid
                }
            }
        }
    }

    // ============================================================================
    // DEBUGGING AND TESTING UTILITIES
    // ============================================================================

    /// Alias for check_invariants_detailed (for test compatibility).
    pub fn validate(&self) -> Result<(), String> {
        self.check_invariants_detailed()
    }

    /// Returns all key-value pairs as a vector (for testing/debugging).
    pub fn slice(&self) -> Vec<(&K, &V)> {
        self.items().collect()
    }

    /// Returns the sizes of all leaf nodes (for testing/debugging).
    pub fn leaf_sizes(&self) -> Vec<usize> {
        let mut sizes = Vec::new();
        self.collect_leaf_sizes(&self.root, &mut sizes);
        sizes
    }

    /// Prints the node chain for debugging.
    pub fn print_node_chain(&self) {
        println!("Tree structure:");
        self.print_node(&self.root, 0);
    }

    /// Recursively collect leaf sizes for debugging.
    fn collect_leaf_sizes(&self, node: &NodeRef<K, V>, sizes: &mut Vec<usize>) {
        match node {
            NodeRef::Leaf(id, _) => {
                if let Some(leaf) = self.get_leaf(*id) {
                    sizes.push(leaf.keys_len());
                }
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.get_branch(*id) {
                    for child in &branch.children {
                        self.collect_leaf_sizes(child, sizes);
                    }
                }
            }
        }
    }

    /// Print a node and its children recursively for debugging.
    fn print_node(&self, node: &NodeRef<K, V>, depth: usize) {
        let indent = "  ".repeat(depth);
        match node {
            NodeRef::Leaf(id, _) => {
                if let Some(leaf) = self.get_leaf(*id) {
                    println!(
                        "{}Leaf[id={}, cap={}]: {} keys",
                        indent,
                        id,
                        leaf.capacity,
                        leaf.keys_len()
                    );
                } else {
                    println!("{}Leaf[id={}]: <missing>", indent, id);
                }
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.get_branch(*id) {
                    println!(
                        "{}Branch[id={}, cap={}]: {} keys, {} children",
                        indent,
                        id,
                        branch.capacity,
                        branch.keys.len(),
                        branch.children.len()
                    );
                    for child in &branch.children {
                        self.print_node(child, depth + 1);
                    }
                } else {
                    println!("{}Branch[id={}]: <missing>", indent, id);
                }
            }
        }
    }

    // ============================================================================
    // VALIDATION HELPERS FOR OPERATIONS
    // ============================================================================

    /// Check if tree is in a valid state for operations
    pub fn validate_for_operation(&self, operation: &str) -> crate::error::BTreeResult<()> {
        self.check_invariants_detailed().map_err(|e| {
            BPlusTreeError::data_integrity(operation, &format!("Validation for {}: {}", operation, e))
        })
    }
}
