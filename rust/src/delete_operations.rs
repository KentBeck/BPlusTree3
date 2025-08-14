//! DELETE operations for BPlusTreeMap.
//!
//! This module contains all the deletion operations for the B+ tree, including
//! key-value removal, node merging, tree shrinking, and helper methods for
//! managing the tree structure during deletions.

use crate::error::{BPlusTreeError, BTreeResult, ModifyResult};
use crate::types::{BPlusTreeMap, NodeRef, LeafNode, BranchNode, NodeId, RemoveResult};
use std::marker::PhantomData;

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
    fn remove_recursive(&mut self, node: &NodeRef<K, V>, key: &K) -> RemoveResult<V> {
        match node {
            NodeRef::Leaf(id, _) => {
                self.get_leaf_mut(*id)
                    .map_or(RemoveResult::Updated(None, false), |leaf| {
                        let removed_value = leaf.remove(key);
                        let is_underfull = leaf.is_underfull();
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

                        // Check if this branch is now underfull after rebalancing
                        let is_underfull =
                            self.is_node_underfull(&NodeRef::Branch(id, PhantomData));
                        RemoveResult::Updated(removed_value, is_underfull)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_operations_module_exists() {
        // Just a placeholder test to ensure the module compiles
        assert!(true);
    }
}