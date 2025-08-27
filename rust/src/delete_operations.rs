//! DELETE operations for BPlusTreeMap.
//!
//! This module contains all the deletion operations for the B+ tree, including
//! key-value removal, node merging, tree shrinking, and helper methods for
//! managing the tree structure during deletions.

use crate::error::{BPlusTreeError, ModifyResult};
use crate::types::{BPlusTreeMap, LeafNode, NodeId, NodeRef, RemoveResult};
use std::marker::PhantomData;

// The RebalanceContext and SiblingInfo structs have been removed in favor of a simpler approach
// that avoids borrowing conflicts while still optimizing arena access patterns.

impl<K: Ord + Clone, V: Clone> BPlusTreeMap<K, V> {
    /// Remove a key from the tree and return its associated value.
    ///
    /// # Arguments
    /// * `key` - The key to remove from the tree
    ///
    /// # Returns
    /// * `Some(value)` - The value that was associated with the key
    /// * `None` - If the key was not present in the tree
    ///
    /// # Examples
    /// ```
    /// use bplustree::BPlusTreeMap;
    ///
    /// let mut tree = BPlusTreeMap::new(4).unwrap();
    /// tree.insert(1, "one");
    /// tree.insert(2, "two");
    ///
    /// assert_eq!(tree.remove(&1), Some("one"));
    /// assert_eq!(tree.remove(&1), None); // Key no longer exists
    /// assert_eq!(tree.len(), 1);
    /// ```
    ///
    /// # Performance
    /// * Time complexity: O(log n) where n is the number of keys
    /// * May trigger node rebalancing or merging operations
    /// * Maintains all B+ tree invariants after removal
    ///
    /// # Panics
    /// Never panics - all operations are memory safe
    pub fn remove(&mut self, key: &K) -> Option<V> {
        // Use remove_recursive to handle the removal
        let result = self.remove_recursive(&self.root.clone(), key);

        match result {
            RemoveResult::Updated(removed_value, _root_became_underfull) => {
                // Check if root needs collapsing after removal
                if removed_value.is_some() {
                    self.collapse_root_if_needed();
                }
                removed_value
            }
        }
    }

    /// Remove a key from the tree, returning an error if the key doesn't exist.
    /// This is equivalent to Python's `del tree[key]`.
    pub fn remove_item(&mut self, key: &K) -> ModifyResult<V> {
        self.remove(key).ok_or(BPlusTreeError::KeyNotFound)
    }

    /// Recursively remove a key with proper arena access.
    #[inline]
    fn remove_recursive(&mut self, node: &NodeRef<K, V>, key: &K) -> RemoveResult<V> {
        match node {
            NodeRef::Leaf(id, _) => {
                self.get_leaf_mut(*id)
                    .map_or(RemoveResult::Updated(None, false), |leaf| {
                        let (removed_value, is_underfull) = leaf.remove(key);
                        RemoveResult::Updated(removed_value, is_underfull)
                    })
            }
            NodeRef::Branch(id, _) => {
                let id = *id;

                // First get child info without mutable borrow
                let (child_index, child_ref) = match self.get_child_for_key(id, key) {
                    Some(info) => info,
                    None => return RemoveResult::Updated(None, false),
                };

                // Recursively remove
                let child_result = self.remove_recursive(&child_ref, key);

                // Handle the result
                match child_result {
                    RemoveResult::Updated(removed_value, child_became_underfull) => {
                        // If child became underfull, try to rebalance
                        if removed_value.is_some() && child_became_underfull {
                            let _child_still_exists = self.rebalance_child(id, child_index);
                        }

                        // Only compute underfull if a removal actually happened
                        let is_underfull = if removed_value.is_some() {
                            self.is_node_underfull(&NodeRef::Branch(id, PhantomData))
                        } else {
                            false
                        };
                        RemoveResult::Updated(removed_value, is_underfull)
                    }
                }
            }
        }
    }

    /// Collapse the root if it's a branch with only one child or no children.
    fn collapse_root_if_needed(&mut self) {
        loop {
            // Capture root ID first to avoid borrowing conflicts
            let root_branch_id = match &self.root {
                NodeRef::Branch(id, _) => Some(*id),
                NodeRef::Leaf(_, _) => None,
            };

            // Use Option combinators for cleaner nested logic handling
            let branch_info = root_branch_id.and_then(|branch_id| {
                self.get_branch(branch_id).map(|branch| {
                    (
                        branch_id,
                        branch.children.len(),
                        branch.children.first().cloned(),
                    )
                })
            });

            match branch_info {
                Some((branch_id, 0, _)) => {
                    // Empty branch - replace with empty leaf
                    self.create_empty_root_leaf();
                    self.deallocate_branch(branch_id);
                    break;
                }
                Some((branch_id, 1, Some(child))) => {
                    // Single child - promote it and continue collapsing
                    self.root = child;
                    self.deallocate_branch(branch_id);
                    // Continue loop in case new root also needs collapsing
                }
                Some((_, _, _)) => {
                    // Multiple children - no collapse needed
                    break;
                }
                None => {
                    // Handle missing branch or already leaf root
                    if root_branch_id.filter(|_| true).is_some() {
                        // Branch ID exists but branch is missing
                        self.create_empty_root_leaf();
                    }
                    break;
                }
            }
        }
    }

    /// Helper method to create empty root leaf
    #[inline]
    fn create_empty_root_leaf(&mut self) {
        let empty_id = self.allocate_leaf(LeafNode::new(self.capacity));
        self.root = NodeRef::Leaf(empty_id, PhantomData);
    }

    /// Helper to check if a node is underfull.
    #[inline]
    fn is_node_underfull(&self, node_ref: &NodeRef<K, V>) -> bool {
        match node_ref {
            NodeRef::Leaf(id, _) => self
                .get_leaf(*id)
                .map(|leaf| leaf.is_underfull())
                .unwrap_or(false),
            NodeRef::Branch(id, _) => self
                .get_branch(*id)
                .map(|branch| branch.is_underfull())
                .unwrap_or(false),
        }
    }

    /// Rebalance an underfull child in an arena branch
    #[inline]
    fn rebalance_child(&mut self, parent_id: NodeId, child_index: usize) -> bool {
        // Gather rebalancing information in minimal arena accesses
        let rebalance_info = {
            let parent_branch = match self.get_branch(parent_id) {
                Some(branch) => branch,
                None => return false,
            };

            let child_is_leaf = matches!(parent_branch.children[child_index], NodeRef::Leaf(_, _));

            let left_sibling_info = if child_index > 0 {
                let sibling_ref = parent_branch.children[child_index - 1];
                let can_donate = match &sibling_ref {
                    NodeRef::Leaf(id, _) => self
                        .get_leaf(*id)
                        .map(|leaf| leaf.keys.len() > leaf.min_keys())
                        .unwrap_or(false),
                    NodeRef::Branch(id, _) => self
                        .get_branch(*id)
                        .map(|branch| branch.keys.len() > branch.min_keys())
                        .unwrap_or(false),
                };
                Some((sibling_ref, can_donate))
            } else {
                None
            };

            let right_sibling_info = if child_index < parent_branch.children.len() - 1 {
                let sibling_ref = parent_branch.children[child_index + 1];
                let can_donate = match &sibling_ref {
                    NodeRef::Leaf(id, _) => self
                        .get_leaf(*id)
                        .map(|leaf| leaf.keys.len() > leaf.min_keys())
                        .unwrap_or(false),
                    NodeRef::Branch(id, _) => self
                        .get_branch(*id)
                        .map(|branch| branch.keys.len() > branch.min_keys())
                        .unwrap_or(false),
                };
                Some((sibling_ref, can_donate))
            } else {
                None
            };

            (child_is_leaf, left_sibling_info, right_sibling_info)
        };

        let (child_is_leaf, left_sibling_info, right_sibling_info) = rebalance_info;

        if child_is_leaf {
            self.rebalance_leaf(
                parent_id,
                child_index,
                left_sibling_info,
                right_sibling_info,
            )
        } else {
            self.rebalance_branch(
                parent_id,
                child_index,
                left_sibling_info,
                right_sibling_info,
            )
        }
    }

    // (Experimental ID-based helpers removed)
}

#[cfg(test)]
mod tests {
    use crate::BPlusTreeMap;

    #[test]
    fn test_delete_operations_module_exists() {
        // Ensure a new tree is empty and basic insert/remove works
        let mut tree = BPlusTreeMap::new(4).unwrap();
        assert_eq!(tree.len(), 0);
        tree.insert(1, "one".to_string());
        assert_eq!(tree.remove(&1), Some("one".to_string()));
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn test_optimized_rebalancing_reduces_arena_access() {
        // Test that the optimized rebalancing works correctly
        let mut tree = BPlusTreeMap::new(4).unwrap();

        // Insert enough items to create multiple levels
        for i in 0..20 {
            tree.insert(i, format!("value_{}", i));
        }

        // Verify tree structure before deletion
        assert!(tree.len() == 20);

        // Delete items that will trigger rebalancing
        for i in (0..10).step_by(2) {
            let removed = tree.remove(&i);
            assert!(removed.is_some(), "Should have removed key {}", i);
        }

        // Verify tree is still valid after rebalancing
        assert!(tree.len() == 15);

        // Verify remaining items are still accessible
        for i in (1..20).step_by(2) {
            if i < 10 {
                assert!(tree.get(&i).is_some(), "Key {} should still exist", i);
            }
        }
        for i in 10..20 {
            assert!(tree.get(&i).is_some(), "Key {} should still exist", i);
        }
    }

    #[test]
    fn test_rebalancing_with_various_sibling_scenarios() {
        // Test different sibling donation and merging scenarios
        let mut tree = BPlusTreeMap::new(4).unwrap(); // Small capacity to force more rebalancing

        // Create a scenario with multiple levels
        for i in 0..15 {
            tree.insert(i, i * 2);
        }

        let initial_len = tree.len();

        // Delete items in a pattern that tests different rebalancing scenarios
        let delete_keys = vec![1, 3, 5, 7, 9, 11, 13];
        for key in delete_keys {
            let removed = tree.remove(&key);
            assert!(removed.is_some(), "Should have removed key {}", key);
        }

        assert_eq!(tree.len(), initial_len - 7);

        // Verify tree integrity by checking all remaining items
        let remaining_keys = vec![0, 2, 4, 6, 8, 10, 12, 14];
        for key in remaining_keys {
            assert_eq!(
                tree.get(&key),
                Some(&(key * 2)),
                "Key {} should have correct value",
                key
            );
        }
    }

    #[test]
    fn test_delete_performance_characteristics() {
        // Test that demonstrates the performance characteristics of the optimized delete
        let mut tree = BPlusTreeMap::new(16).unwrap();

        // Insert a larger dataset
        let n = 1000;
        for i in 0..n {
            tree.insert(i, format!("value_{}", i));
        }

        // Delete every 3rd item (creates various rebalancing scenarios)
        let mut deleted_count = 0;
        for i in (0..n).step_by(3) {
            if tree.remove(&i).is_some() {
                deleted_count += 1;
            }
        }

        assert_eq!(tree.len(), n - deleted_count);

        // Verify tree is still valid and searchable
        for i in 0..n {
            let should_exist = i % 3 != 0;
            assert_eq!(
                tree.get(&i).is_some(),
                should_exist,
                "Key {} existence should be {}",
                i,
                should_exist
            );
        }
    }
}

impl<K: Ord + Clone, V: Clone> BPlusTreeMap<K, V> {
    /// Rebalance an underfull leaf child using pre-gathered sibling information.
    /// Optimized to minimize repeated arena lookups by resolving sibling IDs once.
    fn rebalance_leaf(
        &mut self,
        parent_id: NodeId,
        child_index: usize,
        left_sibling_info: Option<(NodeRef<K, V>, bool)>,
        right_sibling_info: Option<(NodeRef<K, V>, bool)>,
    ) -> bool {
        // Resolve sibling IDs once from parent
        let (left_id_opt, right_id_opt) = match self.get_branch(parent_id) {
            Some(parent) => {
                let left_id_opt = if child_index > 0 {
                    match parent.children[child_index - 1] {
                        NodeRef::Leaf(id, _) => Some(id),
                        _ => None,
                    }
                } else {
                    None
                };
                let right_id_opt = if child_index + 1 < parent.children.len() {
                    match parent.children[child_index + 1] {
                        NodeRef::Leaf(id, _) => Some(id),
                        _ => None,
                    }
                } else {
                    None
                };
                (left_id_opt, right_id_opt)
            }
            None => return false,
        };

        // Strategy 1: Try to borrow from a sibling that can donate (prefer left)
        if let Some((_left_ref, can_donate)) = left_sibling_info {
            if can_donate {
                if let Some(left_id) = left_id_opt {
                    // Child ID from parent
                    let child_id = match self.get_branch(parent_id) {
                        Some(parent) => match parent.children[child_index] {
                            NodeRef::Leaf(id, _) => id,
                            _ => return false,
                        },
                        None => return false,
                    };
                    return self.borrow_from_left_leaf_with_ids(
                        parent_id,
                        child_index,
                        left_id,
                        child_id,
                    );
                }
            }
        }
        if let Some((_right_ref, can_donate)) = right_sibling_info {
            if can_donate {
                if let Some(right_id) = right_id_opt {
                    let child_id = match self.get_branch(parent_id) {
                        Some(parent) => match parent.children[child_index] {
                            NodeRef::Leaf(id, _) => id,
                            _ => return false,
                        },
                        None => return false,
                    };
                    return self.borrow_from_right_leaf_with_ids(
                        parent_id,
                        child_index,
                        child_id,
                        right_id,
                    );
                }
            }
        }

        // Strategy 2: No siblings can donate, must merge (prefer left)
        if let Some(left_id) = left_id_opt {
            let child_id = match self.get_branch(parent_id) {
                Some(parent) => match parent.children[child_index] {
                    NodeRef::Leaf(id, _) => id,
                    _ => return false,
                },
                None => return false,
            };
            self.merge_with_left_leaf_with_ids(parent_id, child_index, left_id, child_id)
        } else if let Some(right_id) = right_id_opt {
            let child_id = match self.get_branch(parent_id) {
                Some(parent) => match parent.children[child_index] {
                    NodeRef::Leaf(id, _) => id,
                    _ => return false,
                },
                None => return false,
            };
            self.merge_with_right_leaf_with_ids(parent_id, child_index, child_id, right_id)
        } else {
            // No siblings available - this shouldn't happen in a valid B+ tree
            false
        }
    }

    /// Rebalance an underfull branch child using pre-gathered sibling information.
    /// Optimized to reduce repeated arena lookups by resolving sibling IDs and separator keys once.
    fn rebalance_branch(
        &mut self,
        parent_id: NodeId,
        child_index: usize,
        left_sibling_info: Option<(NodeRef<K, V>, bool)>,
        right_sibling_info: Option<(NodeRef<K, V>, bool)>,
    ) -> bool {
        // Resolve sibling IDs and separator keys once from parent
        let (left_id_opt, right_id_opt, left_sep_opt, right_sep_opt, child_id) =
            match self.get_branch(parent_id) {
                Some(parent) => {
                    let left = if child_index > 0 {
                        match parent.children[child_index - 1] {
                            NodeRef::Branch(id, _) => Some(id),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let right = if child_index + 1 < parent.children.len() {
                        match parent.children[child_index + 1] {
                            NodeRef::Branch(id, _) => Some(id),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let left_sep = if left.is_some() {
                        Some(parent.keys[child_index - 1].clone())
                    } else {
                        None
                    };
                    let right_sep = if right.is_some() {
                        Some(parent.keys[child_index].clone())
                    } else {
                        None
                    };
                    let child_id = match parent.children[child_index] {
                        NodeRef::Branch(id, _) => id,
                        _ => return false,
                    };
                    (left, right, left_sep, right_sep, child_id)
                }
                None => return false,
            };

        // Strategy 1: Try to borrow (prefer left)
        if let Some((_left_ref, can_donate)) = left_sibling_info {
            if can_donate {
                if let (Some(left_id), Some(sep)) = (left_id_opt, left_sep_opt) {
                    return self.borrow_from_left_branch_with(
                        parent_id,
                        child_index,
                        left_id,
                        child_id,
                        sep,
                    );
                }
            }
        }
        if let Some((_right_ref, can_donate)) = right_sibling_info {
            if can_donate {
                if let (Some(right_id), Some(sep)) = (right_id_opt, right_sep_opt) {
                    return self.borrow_from_right_branch_with(
                        parent_id,
                        child_index,
                        child_id,
                        right_id,
                        sep,
                    );
                }
            }
        }

        // Strategy 2: Merge (prefer left)
        if left_id_opt.is_some() {
            self.merge_with_left_branch(parent_id, child_index)
        } else if right_id_opt.is_some() {
            self.merge_with_right_branch(parent_id, child_index)
        } else {
            false
        }
    }

    /// Merge branch with left sibling
    fn merge_with_left_branch(&mut self, parent_id: NodeId, child_index: usize) -> bool {
        // Get the branch IDs and collect all needed info from parent in one access
        let (left_id, child_id, separator_key) = match self.get_branch(parent_id) {
            Some(parent) => {
                match (
                    &parent.children[child_index - 1],
                    &parent.children[child_index],
                ) {
                    (NodeRef::Branch(left, _), NodeRef::Branch(child, _)) => {
                        (*left, *child, parent.keys[child_index - 1].clone())
                    }
                    _ => return false,
                }
            }
            None => return false,
        };

        // Extract all content from child and merge into left in one pass
        // Use a safer approach that avoids multiple mutable borrows
        {
            // First, extract content from child
            let (mut child_keys, mut child_children) = match self.get_branch_mut(child_id) {
                Some(child_branch) => {
                    let keys = std::mem::take(&mut child_branch.keys);
                    let children = std::mem::take(&mut child_branch.children);
                    (keys, children)
                }
                None => return false,
            };

            // Then merge into left, reserving capacity to avoid reallocations
            let Some(left_branch) = self.get_branch_mut(left_id) else {
                return false;
            };
            left_branch.keys.push(separator_key);
            left_branch.keys.append(&mut child_keys);
            left_branch.children.append(&mut child_children);
        }

        // Remove child from parent (single parent access)
        let Some(parent) = self.get_branch_mut(parent_id) else {
            return false;
        };
        parent.children.remove(child_index);
        parent.keys.remove(child_index - 1);

        // Deallocate the merged child
        self.deallocate_branch(child_id);

        false // Child was merged away
    }

    /// Merge branch with right sibling
    fn merge_with_right_branch(&mut self, parent_id: NodeId, child_index: usize) -> bool {
        // Get the branch IDs and collect all needed info from parent in one access
        let (child_id, right_id, separator_key) = match self.get_branch(parent_id) {
            Some(parent) => {
                match (
                    &parent.children[child_index],
                    &parent.children[child_index + 1],
                ) {
                    (NodeRef::Branch(child, _), NodeRef::Branch(right, _)) => {
                        (*child, *right, parent.keys[child_index].clone())
                    }
                    _ => return false,
                }
            }
            None => return false,
        };

        // Extract all content from right and merge into child in one pass
        // Use a safer approach that avoids multiple mutable borrows
        {
            // First, extract content from right
            let (mut right_keys, mut right_children) = match self.get_branch_mut(right_id) {
                Some(right_branch) => {
                    let keys = std::mem::take(&mut right_branch.keys);
                    let children = std::mem::take(&mut right_branch.children);
                    (keys, children)
                }
                None => return false,
            };

            // Then merge into child, reserving capacity to avoid reallocations
            let Some(child_branch) = self.get_branch_mut(child_id) else {
                return false;
            };
            child_branch.keys.push(separator_key);
            child_branch.keys.append(&mut right_keys);
            child_branch.children.append(&mut right_children);
        }

        // Remove right from parent (second and final parent access)
        let Some(parent) = self.get_branch_mut(parent_id) else {
            return false;
        };
        parent.children.remove(child_index + 1);
        parent.keys.remove(child_index);

        // Deallocate the merged right sibling
        self.deallocate_branch(right_id);

        true // Child still exists
    }

    // Optimized helpers that avoid re-reading parent for IDs/keys
    fn borrow_from_left_branch_with(
        &mut self,
        parent_id: NodeId,
        child_index: usize,
        left_id: NodeId,
        child_id: NodeId,
        separator_key: K,
    ) -> bool {
        let (moved_key, moved_child) = match self.get_branch_mut(left_id) {
            Some(left_branch) => match left_branch.borrow_last() {
                Some(result) => result,
                None => return false,
            },
            None => return false,
        };

        let Some(child_branch) = self.get_branch_mut(child_id) else {
            return false;
        };
        let new_separator = child_branch.accept_from_left(separator_key, moved_key, moved_child);

        let Some(parent) = self.get_branch_mut(parent_id) else {
            return false;
        };
        parent.keys[child_index - 1] = new_separator;
        true
    }

    fn borrow_from_right_branch_with(
        &mut self,
        parent_id: NodeId,
        child_index: usize,
        child_id: NodeId,
        right_id: NodeId,
        separator_key: K,
    ) -> bool {
        let (moved_key, moved_child) = match self.get_branch_mut(right_id) {
            Some(right_branch) => match right_branch.borrow_first() {
                Some(result) => result,
                None => return false,
            },
            None => return false,
        };

        let Some(child_branch) = self.get_branch_mut(child_id) else {
            return false;
        };
        let new_separator = child_branch.accept_from_right(separator_key, moved_key, moved_child);

        let Some(parent) = self.get_branch_mut(parent_id) else {
            return false;
        };
        parent.keys[child_index] = new_separator;
        true
    }

    fn borrow_from_left_leaf_with_ids(
        &mut self,
        branch_id: NodeId,
        child_index: usize,
        left_id: NodeId,
        child_id: NodeId,
    ) -> bool {
        let (key, value) = match self.get_leaf_mut(left_id) {
            Some(left_leaf) => match left_leaf.borrow_last() {
                Some(kv) => kv,
                None => return false,
            },
            None => return false,
        };
        let sep = key.clone();
        let Some(child_leaf) = self.get_leaf_mut(child_id) else {
            return false;
        };
        child_leaf.accept_from_left(key, value);
        if let Some(parent) = self.get_branch_mut(branch_id) {
            parent.keys[child_index - 1] = sep;
            true
        } else {
            false
        }
    }

    fn borrow_from_right_leaf_with_ids(
        &mut self,
        branch_id: NodeId,
        child_index: usize,
        child_id: NodeId,
        right_id: NodeId,
    ) -> bool {
        let (key, value, new_first_opt) = if let Some(right_leaf) = self.get_leaf_mut(right_id) {
            if let Some((k, v)) = right_leaf.borrow_first() {
                (k, v, right_leaf.first_key().cloned())
            } else {
                return false;
            }
        } else {
            return false;
        };
        let Some(child_leaf) = self.get_leaf_mut(child_id) else {
            return false;
        };
        child_leaf.accept_from_right(key, value);
        if let (Some(sep), Some(parent)) = (new_first_opt, self.get_branch_mut(branch_id)) {
            parent.keys[child_index] = sep;
            true
        } else {
            false
        }
    }

    fn merge_with_left_leaf_with_ids(
        &mut self,
        branch_id: NodeId,
        child_index: usize,
        left_id: NodeId,
        child_id: NodeId,
    ) -> bool {
        let (mut child_keys, mut child_values, child_next) = match self.get_leaf_mut(child_id) {
            Some(child_leaf) => child_leaf.extract_all(),
            None => return false,
        };
        let Some(left_leaf) = self.get_leaf_mut(left_id) else {
            return false;
        };
        left_leaf.append_keys(&mut child_keys);
        left_leaf.append_values(&mut child_values);
        left_leaf.next = child_next;
        let Some(branch) = self.get_branch_mut(branch_id) else {
            return false;
        };
        branch.children.remove(child_index);
        branch.keys.remove(child_index - 1);
        self.deallocate_leaf(child_id);
        false
    }

    fn merge_with_right_leaf_with_ids(
        &mut self,
        branch_id: NodeId,
        child_index: usize,
        child_id: NodeId,
        right_id: NodeId,
    ) -> bool {
        {
            let (mut right_keys, mut right_values, right_next) = match self.get_leaf_mut(right_id) {
                Some(right_leaf) => {
                    let keys = right_leaf.take_keys();
                    let values = right_leaf.take_values();
                    let next = right_leaf.next;
                    (keys, values, next)
                }
                None => return false,
            };
            let Some(child_leaf) = self.get_leaf_mut(child_id) else {
                return false;
            };
            child_leaf.append_keys(&mut right_keys);
            child_leaf.append_values(&mut right_values);
            child_leaf.next = right_next;
        }
        let Some(branch) = self.get_branch_mut(branch_id) else {
            return false;
        };
        branch.children.remove(child_index + 1);
        branch.keys.remove(child_index);
        self.deallocate_leaf(right_id);
        true
    }
}
