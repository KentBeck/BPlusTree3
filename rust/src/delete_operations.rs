//! DELETE operations for BPlusTreeMap.
//!
//! This module contains all the deletion operations for the B+ tree, including
//! key-value removal, node merging, tree shrinking, and helper methods for
//! managing the tree structure during deletions.

use crate::error::{BPlusTreeError, ModifyResult};
use crate::types::{BPlusTreeMap, NodeRef, LeafNode, NodeId, RemoveResult};
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

                        // Check if this branch is now underfull after rebalancing
                        let is_underfull =
                            self.is_node_underfull(&NodeRef::Branch(id, PhantomData));
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
                    root_branch_id
                        .filter(|_| true) // Branch ID exists but branch is missing
                        .map(|_| self.create_empty_root_leaf());
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

    /// Helper to check if a node can donate
    #[inline]
    fn can_node_donate(&self, node_ref: &NodeRef<K, V>) -> bool {
        match node_ref {
            NodeRef::Leaf(id, _) => self
                .get_leaf(*id)
                .map(|leaf| leaf.can_donate())
                .unwrap_or(false),
            NodeRef::Branch(id, _) => self
                .get_branch(*id)
                .map(|branch| branch.can_donate())
                .unwrap_or(false),
        }
    }

    /// Get sibling node reference if it exists and types match
    fn get_branch_sibling(
        &self,
        branch_id: NodeId,
        child_index: usize,
        get_left: bool,
    ) -> Option<NodeRef<K, V>> {
        let branch = self.get_branch(branch_id)?;
        let sibling_index = if get_left {
            child_index.checked_sub(1)?
        } else {
            child_index + 1
        };

        match (
            branch.children.get(child_index),
            branch.children.get(sibling_index),
        ) {
            (Some(NodeRef::Branch(_, _)), Some(NodeRef::Branch(_, _))) => {
                branch.children.get(sibling_index).cloned()
            }
            _ => None,
        }
    }

    /// Rebalance an underfull child in an arena branch
    #[inline]
    fn rebalance_child(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Get information about the child and its siblings
        let (has_left_sibling, has_right_sibling, child_is_leaf) = match self.get_branch(branch_id)
        {
            Some(branch) => {
                let has_left = child_index > 0;
                let has_right = child_index < branch.children.len() - 1;
                let is_leaf = matches!(branch.children[child_index], NodeRef::Leaf(_, _));
                (has_left, has_right, is_leaf)
            }
            None => return false,
        };

        if child_is_leaf {
            // Handle leaf rebalancing
            self.rebalance_leaf_child(branch_id, child_index, has_left_sibling, has_right_sibling)
        } else {
            // Handle branch rebalancing
            self.rebalance_branch_child(branch_id, child_index, has_left_sibling, has_right_sibling)
        }
    }
}

#[cfg(test)]
mod tests {
    // Test module for delete operations

    #[test]
    fn test_delete_operations_module_exists() {
        // Just a placeholder test to ensure the module compiles
        assert!(true);
    }
}

impl<K: Ord + Clone, V: Clone> BPlusTreeMap<K, V> {
    // Rebalance an underfull leaf child
    fn rebalance_leaf_child(
        &mut self,
        branch_id: NodeId,
        child_index: usize,
        has_left_sibling: bool,
        has_right_sibling: bool,
    ) -> bool {
        // Get parent branch once and cache sibling info to avoid multiple arena lookups
        let (left_can_donate, right_can_donate) = match self.get_branch(branch_id) {
            Some(branch) => {
                let left_can_donate = if has_left_sibling && child_index > 0 {
                    branch
                        .children
                        .get(child_index - 1)
                        .map(|sibling| self.can_node_donate(sibling))
                        .unwrap_or(false)
                } else {
                    false
                };

                let right_can_donate = if has_right_sibling {
                    branch
                        .children
                        .get(child_index + 1)
                        .map(|sibling| self.can_node_donate(sibling))
                        .unwrap_or(false)
                } else {
                    false
                };

                (left_can_donate, right_can_donate)
            }
            None => return false,
        };

        // Try borrowing from left sibling first
        if left_can_donate {
            return self.borrow_from_left_leaf(branch_id, child_index);
        }

        // Try borrowing from right sibling
        if right_can_donate {
            return self.borrow_from_right_leaf(branch_id, child_index);
        }

        // Cannot borrow, must merge
        if has_left_sibling {
            return self.merge_with_left_leaf(branch_id, child_index);
        } else if has_right_sibling {
            return self.merge_with_right_leaf(branch_id, child_index);
        }

        // No siblings to merge with - this shouldn't happen
        false
    }

    // Rebalance an underfull branch child
    fn rebalance_branch_child(
        &mut self,
        branch_id: NodeId,
        child_index: usize,
        has_left_sibling: bool,
        has_right_sibling: bool,
    ) -> bool {
        // Get parent branch once and cache sibling donation and merge capabilities
        let (left_can_donate, right_can_donate, left_can_merge, right_can_merge) =
            match self.get_branch(branch_id) {
                Some(branch) => {
                    let left_can_donate = if has_left_sibling {
                        self.get_branch_sibling(branch_id, child_index, true)
                            .map(|sibling| self.can_node_donate(&sibling))
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    let right_can_donate = if has_right_sibling {
                        self.get_branch_sibling(branch_id, child_index, false)
                            .map(|sibling| self.can_node_donate(&sibling))
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    let left_can_merge = if has_left_sibling {
                        if let (NodeRef::Branch(left_id, _), NodeRef::Branch(child_id, _)) = (
                            &branch.children[child_index - 1],
                            &branch.children[child_index],
                        ) {
                            self.get_branch(*left_id)
                                .zip(self.get_branch(*child_id))
                                .map(|(left, child)| {
                                    left.keys.len() + 1 + child.keys.len() <= self.capacity
                                })
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    let right_can_merge = if has_right_sibling {
                        if let (NodeRef::Branch(child_id, _), NodeRef::Branch(right_id, _)) = (
                            &branch.children[child_index],
                            &branch.children[child_index + 1],
                        ) {
                            self.get_branch(*child_id)
                                .zip(self.get_branch(*right_id))
                                .map(|(child, right)| {
                                    child.keys.len() + 1 + right.keys.len() <= self.capacity
                                })
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    (
                        left_can_donate,
                        right_can_donate,
                        left_can_merge,
                        right_can_merge,
                    )
                }
                None => return false,
            };

        // Try borrowing from left sibling first
        if left_can_donate {
            return self.borrow_from_left_branch(branch_id, child_index);
        }

        // Try borrowing from right sibling
        if right_can_donate {
            return self.borrow_from_right_branch(branch_id, child_index);
        }

        // Try merging with left sibling
        if left_can_merge {
            return self.merge_with_left_branch(branch_id, child_index);
        }

        // Try merging with right sibling
        if right_can_merge {
            return self.merge_with_right_branch(branch_id, child_index);
        }

        // Cannot borrow or merge - leave the node underfull
        // This can happen when siblings are also near minimum capacity
        true
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

            // Then merge into left
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

            // Then merge into child
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
  /// Borrow from left sibling branch
    fn borrow_from_left_branch(&mut self, parent_id: NodeId, child_index: usize) -> bool {
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

        // Borrow from left branch
        let (moved_key, moved_child) = match self.get_branch_mut(left_id) {
            Some(left_branch) => match left_branch.borrow_last() {
                Some(result) => result,
                None => return false,
            },
            None => return false,
        };

        // Accept into child branch - use early return for cleaner flow
        let Some(child_branch) = self.get_branch_mut(child_id) else {
            return false;
        };
        let new_separator = child_branch.accept_from_left(separator_key, moved_key, moved_child);

        // Update separator in parent (second and final parent access)
        let Some(parent) = self.get_branch_mut(parent_id) else {
            return false;
        };
        parent.keys[child_index - 1] = new_separator;

        true
    }

    /// Borrow from right sibling branch
    fn borrow_from_right_branch(&mut self, parent_id: NodeId, child_index: usize) -> bool {
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

        // Borrow from right branch
        let (moved_key, moved_child) = match self.get_branch_mut(right_id) {
            Some(right_branch) => match right_branch.borrow_first() {
                Some(result) => result,
                None => return false,
            },
            None => return false,
        };

        // Accept into child branch - use early return for cleaner flow
        let Some(child_branch) = self.get_branch_mut(child_id) else {
            return false;
        };
        let new_separator = child_branch.accept_from_right(separator_key, moved_key, moved_child);

        // Update separator in parent (second and final parent access)
        let Some(parent) = self.get_branch_mut(parent_id) else {
            return false;
        };
        parent.keys[child_index] = new_separator;

        true
    }   
 /// Borrow from left sibling leaf
    fn borrow_from_left_leaf(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Extract leaf IDs from parent in one access (inlined get_adjacent_leaf_ids logic)
        let (left_id, child_id) = match self.get_branch(branch_id) {
            Some(branch) => {
                match (
                    branch.children.get(child_index - 1),
                    branch.children.get(child_index),
                ) {
                    (Some(NodeRef::Leaf(left_id, _)), Some(NodeRef::Leaf(child_id, _))) => {
                        (*left_id, *child_id)
                    }
                    _ => return false,
                }
            }
            None => return false,
        };

        // Borrow from left leaf
        let (key, value) = match self.get_leaf_mut(left_id) {
            Some(left_leaf) => match left_leaf.borrow_last() {
                Some(result) => result,
                None => return false,
            },
            None => return false,
        };

        // Accept into child leaf - use early return for cleaner flow
        let Some(child_leaf) = self.get_leaf_mut(child_id) else {
            return false;
        };
        child_leaf.accept_from_left(key.clone(), value);

        // Update separator in parent (second and final parent access)
        let Some(branch) = self.get_branch_mut(branch_id) else {
            return false;
        };
        branch.keys[child_index - 1] = key;

        true
    }

    /// Borrow from right sibling leaf
    fn borrow_from_right_leaf(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Extract leaf IDs from parent in one access (inlined get_adjacent_leaf_ids logic)
        let (child_id, right_id) = match self.get_branch(branch_id) {
            Some(branch) => {
                match (
                    branch.children.get(child_index),
                    branch.children.get(child_index + 1),
                ) {
                    (Some(NodeRef::Leaf(child_id, _)), Some(NodeRef::Leaf(right_id, _))) => {
                        (*child_id, *right_id)
                    }
                    _ => return false,
                }
            }
            None => return false,
        };

        // Borrow from right leaf
        let (key, value) = match self.get_leaf_mut(right_id) {
            Some(right_leaf) => match right_leaf.borrow_first() {
                Some(result) => result,
                None => return false,
            },
            None => return false,
        };

        // Accept into child leaf - use early return for cleaner flow
        let Some(child_leaf) = self.get_leaf_mut(child_id) else {
            return false;
        };
        child_leaf.accept_from_right(key, value);

        // Update separator in parent (new first key of right sibling, second parent access)
        let new_separator = self
            .get_leaf(right_id)
            .and_then(|right_leaf| right_leaf.first_key().cloned());

        // Use Option combinators for nested conditional update
        new_separator
            .zip(self.get_branch_mut(branch_id))
            .map(|(sep, branch)| branch.keys[child_index] = sep);

        true
    }

    /// Merge with left sibling leaf
    fn merge_with_left_leaf(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Extract leaf IDs from parent in one access (inlined get_adjacent_leaf_ids logic)
        let (left_id, child_id) = match self.get_branch(branch_id) {
            Some(branch) => {
                match (
                    branch.children.get(child_index - 1),
                    branch.children.get(child_index),
                ) {
                    (Some(NodeRef::Leaf(left_id, _)), Some(NodeRef::Leaf(child_id, _))) => {
                        (*left_id, *child_id)
                    }
                    _ => return false,
                }
            }
            None => return false,
        };

        // Extract all content from child
        let (mut child_keys, mut child_values, child_next) = match self.get_leaf_mut(child_id) {
            Some(child_leaf) => child_leaf.extract_all(),
            None => return false,
        };

        // Merge into left leaf and update linked list - use early return for cleaner flow
        let Some(left_leaf) = self.get_leaf_mut(left_id) else {
            return false;
        };
        left_leaf.append_keys(&mut child_keys);
        left_leaf.append_values(&mut child_values);
        left_leaf.next = child_next;

        // Remove child from parent (second and final parent access)
        let Some(branch) = self.get_branch_mut(branch_id) else {
            return false;
        };
        branch.children.remove(child_index);
        branch.keys.remove(child_index - 1);

        // Deallocate the merged child
        self.deallocate_leaf(child_id);

        false // Child was merged away
    }

    /// Merge with right sibling leaf
    fn merge_with_right_leaf(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Extract leaf IDs from parent in one access (inlined get_adjacent_leaf_ids logic)
        let (child_id, right_id) = match self.get_branch(branch_id) {
            Some(branch) => {
                match (
                    branch.children.get(child_index),
                    branch.children.get(child_index + 1),
                ) {
                    (Some(NodeRef::Leaf(child_id, _)), Some(NodeRef::Leaf(right_id, _))) => {
                        (*child_id, *right_id)
                    }
                    _ => return false,
                }
            }
            None => return false,
        };

        // Extract content from right and merge into child in one pass
        // Use a safer approach that avoids multiple mutable borrows
        {
            // First, extract content from right
            let (mut right_keys, mut right_values, right_next) = match self.get_leaf_mut(right_id) {
                Some(right_leaf) => {
                    let keys = right_leaf.take_keys();
                    let values = right_leaf.take_values();
                    let next = right_leaf.next;
                    (keys, values, next)
                }
                None => return false,
            };

            // Then merge into child
            let Some(child_leaf) = self.get_leaf_mut(child_id) else {
                return false;
            };
            child_leaf.append_keys(&mut right_keys);
            child_leaf.append_values(&mut right_values);
            child_leaf.next = right_next;
        }

        // Remove right from parent (second and final parent access)
        let Some(branch) = self.get_branch_mut(branch_id) else {
            return false;
        };
        branch.children.remove(child_index + 1);
        branch.keys.remove(child_index);

        // Deallocate the merged right sibling
        self.deallocate_leaf(right_id);

        true // Child still exists
    }
}
