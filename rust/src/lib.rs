//! B+ Tree implementation in Rust with dict-like API.
//!
//! This module provides a B+ tree data structure with a dictionary-like interface,
//! supporting efficient insertion, deletion, lookup, and range queries.

use std::ops::{Bound, RangeBounds};

// Import our new modules
mod arena;
mod compact_arena;
mod error;
mod macros;
mod types;
mod construction;
mod get_operations;
mod insert_operations;
mod delete_operations;
mod detailed_iterator_analysis;
mod comprehensive_performance_benchmark;

pub use arena::{Arena, ArenaStats, NodeId as ArenaNodeId, NULL_NODE as ARENA_NULL_NODE};
pub use compact_arena::{CompactArena, CompactArenaStats};
pub use error::{
    BPlusTreeError, BTreeResult, BTreeResultExt, InitResult, KeyResult, ModifyResult,
};
pub use types::{BPlusTreeMap, NodeId, NodeRef, NULL_NODE, ROOT_NODE, LeafNode, BranchNode};
pub use construction::{InitResult as ConstructionResult, validation};

use std::marker::PhantomData;

use error::TreeResult;
use types::{
    InsertResult, RemoveResult, SplitNodeData,
};





#[cfg(test)]
mod leaf_caching_tests {
    use super::*;

    #[test]
    fn test_leaf_caching_optimization_proof() {
        let mut tree = BPlusTreeMap::new(4).unwrap(); // Small capacity to force multiple leaves
        
        // Insert enough data to span multiple leaves
        for i in 0..20 {
            tree.insert(i, i * 100);
        }
        
        // Create iterator and verify it has cached leaf reference
        let mut iter = tree.items();
        
        // First call to next() should populate the cache
        let first_item = iter.next();
        assert_eq!(first_item, Some((&0, &0)));
        
        // The key insight: iter.current_leaf_ref should now be Some(...)
        // This proves leaf caching is working
        assert!(iter.current_leaf_ref.is_some(), "Leaf reference should be cached after first next() call");
        
        // Subsequent calls within the same leaf should use cached reference
        let second_item = iter.next();
        assert_eq!(second_item, Some((&1, &100)));
        
        // The cached reference should still be valid
        assert!(iter.current_leaf_ref.is_some(), "Leaf reference should remain cached within same leaf");
        
        // Continue iterating to verify caching works across leaf boundaries
        let mut count = 2; // Already consumed 2 items
        for (k, v) in iter {
            assert_eq!(*k, count);
            assert_eq!(*v, count * 100);
            count += 1;
        }
        assert_eq!(count, 20);
    }

    #[test]
    fn test_fast_iterator_also_uses_leaf_caching() {
        let mut tree = BPlusTreeMap::new(4).unwrap();
        
        // Insert data spanning multiple leaves
        for i in 0..20 {
            tree.insert(i, i * 100);
        }
        
        // Test FastItemIterator also uses leaf caching
        let mut fast_iter = tree.items_fast();
        
        // First call should populate cache
        let first_item = fast_iter.next();
        assert_eq!(first_item, Some((&0, &0)));
        
        // Verify FastItemIterator also caches leaf references
        assert!(fast_iter.current_leaf_ref.is_some(), "FastItemIterator should also cache leaf references");
        
        // Verify it works correctly
        let mut count = 1; // Already consumed 1 item
        for (k, v) in fast_iter {
            assert_eq!(*k, count);
            assert_eq!(*v, count * 100);
            count += 1;
        }
        assert_eq!(count, 20);
    }
}





impl<K: Ord + Clone, V: Clone> BPlusTreeMap<K, V> {
    // ============================================================================
    // CONSTRUCTION
    // ============================================================================

    // Construction methods moved to construction.rs module

    // ============================================================================
    // GET OPERATIONS
    // ============================================================================

    /// Get a reference to the value associated with a key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    // GET operations moved to get_operations.rs module

    /// Helper to check if a node is underfull.
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

    // ============================================================================
    // INSERT OPERATIONS
    // ============================================================================

    // insert method moved to insert_operations.rs module

    // ============================================================================
    // HELPERS FOR INSERT OPERATIONS
    // ============================================================================

    // new_root method moved to insert_operations.rs module

    // insert_recursive method moved to insert_operations.rs module

    // ============================================================================
    // DELETE OPERATIONS
    // ============================================================================

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

    // try_get method moved to get_operations.rs module

    /// Insert with comprehensive error handling and rollback on failure
    pub fn try_insert(&mut self, key: K, value: V) -> ModifyResult<Option<V>>
    where
        K: Clone,
        V: Clone,
    {
        // Validate tree state before insertion
        if let Err(e) = self.check_invariants_detailed() {
            return Err(BPlusTreeError::DataIntegrityError(e));
        }

        let old_value = self.insert(key, value);

        // Validate tree state after insertion
        if let Err(e) = self.check_invariants_detailed() {
            return Err(BPlusTreeError::DataIntegrityError(e));
        }

        Ok(old_value)
    }

    /// Remove with comprehensive error handling
    pub fn try_remove(&mut self, key: &K) -> ModifyResult<V> {
        // Validate tree state before removal
        if let Err(e) = self.check_invariants_detailed() {
            return Err(BPlusTreeError::DataIntegrityError(e));
        }

        let value = self.remove(key).ok_or(BPlusTreeError::KeyNotFound)?;

        // Validate tree state after removal
        if let Err(e) = self.check_invariants_detailed() {
            return Err(BPlusTreeError::DataIntegrityError(e));
        }

        Ok(value)
    }

    /// Batch insert operations with rollback on any failure
    pub fn batch_insert(&mut self, items: Vec<(K, V)>) -> ModifyResult<Vec<Option<V>>>
    where
        K: Clone,
        V: Clone,
    {
        let mut results = Vec::new();
        let mut inserted_keys = Vec::new();

        for (key, value) in items {
            match self.try_insert(key.clone(), value) {
                Ok(old_value) => {
                    results.push(old_value);
                    inserted_keys.push(key);
                }
                Err(e) => {
                    // Rollback all successful insertions
                    for rollback_key in inserted_keys {
                        self.remove(&rollback_key);
                    }
                    return Err(e);
                }
            }
        }

        Ok(results)
    }

    // get_many method moved to get_operations.rs module

    /// Check if tree is in a valid state for operations
    pub fn validate_for_operation(&self, operation: &str) -> BTreeResult<()> {
        self.check_invariants_detailed().map_err(|e| {
            BPlusTreeError::DataIntegrityError(format!("Validation for {}: {}", operation, e))
        })
    }

    // ============================================================================
    // HELPERS FOR DELETE OPERATIONS
    // ============================================================================

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
    /// Rebalance an underfull child in an arena branch
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

    /// Rebalance an underfull leaf child
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

    /// Rebalance an underfull branch child
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
            .and_then(|right_leaf| right_leaf.keys.first().cloned());

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
        left_leaf.keys.append(&mut child_keys);
        left_leaf.values.append(&mut child_values);
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
                    let keys = std::mem::take(&mut right_leaf.keys);
                    let values = std::mem::take(&mut right_leaf.values);
                    let next = right_leaf.next;
                    (keys, values, next)
                }
                None => return false,
            };

            // Then merge into child
            let Some(child_leaf) = self.get_leaf_mut(child_id) else {
                return false;
            };
            child_leaf.keys.append(&mut right_keys);
            child_leaf.values.append(&mut right_values);
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
    fn create_empty_root_leaf(&mut self) {
        let empty_id = self.allocate_leaf(LeafNode::new(self.capacity));
        self.root = NodeRef::Leaf(empty_id, PhantomData);
    }

    // ============================================================================
    // OTHER API OPERATIONS
    // ============================================================================

    /// Returns the number of elements in the tree.
    pub fn len(&self) -> usize {
        self.len_recursive(&self.root)
    }

    /// Recursively count elements with proper arena access.
    fn len_recursive(&self, node: &NodeRef<K, V>) -> usize {
        match node {
            NodeRef::Leaf(id, _) => self.get_leaf(*id).map(|leaf| leaf.len()).unwrap_or(0),
            NodeRef::Branch(id, _) => self
                .get_branch(*id)
                .map(|branch| {
                    branch
                        .children
                        .iter()
                        .map(|child| self.len_recursive(child))
                        .sum()
                })
                .unwrap_or(0),
        }
    }

    /// Returns true if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if the root is a leaf node.
    pub fn is_leaf_root(&self) -> bool {
        matches!(self.root, NodeRef::Leaf(_, _))
    }

    /// Returns the number of leaf nodes in the tree.
    pub fn leaf_count(&self) -> usize {
        self.leaf_count_recursive(&self.root)
    }

    /// Recursively count leaf nodes with proper arena access.
    fn leaf_count_recursive(&self, node: &NodeRef<K, V>) -> usize {
        match node {
            NodeRef::Leaf(_, _) => 1, // An arena leaf is one leaf node
            NodeRef::Branch(id, _) => self
                .get_branch(*id)
                .map(|branch| {
                    branch
                        .children
                        .iter()
                        .map(|child| self.leaf_count_recursive(child))
                        .sum()
                })
                .unwrap_or(0),
        }
    }

    /// Clear all items from the tree.
    pub fn clear(&mut self) {
        // Clear all arenas and create a new root leaf
        self.leaf_arena.clear();
        self.branch_arena.clear();

        // Create a new root leaf
        let root_leaf = LeafNode::new(self.capacity);
        let root_id = self.leaf_arena.allocate(root_leaf);
        self.root = NodeRef::Leaf(root_id, PhantomData);
    }

    /// Returns an iterator over all key-value pairs in sorted order.
    pub fn items(&self) -> ItemIterator<K, V> {
        ItemIterator::new(self)
    }

    /// Returns a fast iterator over all key-value pairs using unsafe arena access.
    /// This provides better performance by skipping bounds checks.
    /// 
    /// # Safety
    /// This is safe to use as long as the tree structure is valid and no concurrent
    /// modifications occur during iteration.
    pub fn items_fast(&self) -> FastItemIterator<K, V> {
        FastItemIterator::new(self)
    }

    /// Returns an iterator over all keys in sorted order.
    pub fn keys(&self) -> KeyIterator<K, V> {
        KeyIterator::new(self)
    }

    /// Returns an iterator over all values in key order.
    pub fn values(&self) -> ValueIterator<K, V> {
        ValueIterator::new(self)
    }

    /// Returns an iterator over key-value pairs in a range.
    /// If start_key is None, starts from the beginning.
    /// If end_key is None, goes to the end.
    pub fn items_range<'a>(
        &'a self,
        start_key: Option<&K>,
        end_key: Option<&'a K>,
    ) -> RangeIterator<'a, K, V> {
        let start_bound = start_key.map_or(Bound::Unbounded, |k| Bound::Included(k));
        let end_bound = end_key.map_or(Bound::Unbounded, |k| Bound::Excluded(k));

        let (start_info, skip_first, end_info) = self.resolve_range_bounds((start_bound, end_bound));
        RangeIterator::new_with_skip_owned(self, start_info, skip_first, end_info)
    }

    /// Returns an iterator over key-value pairs in a range using Rust's range syntax.
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BPlusTreeMap;
    ///
    /// let mut tree = BPlusTreeMap::new(16).unwrap();
    /// for i in 0..10 {
    ///     tree.insert(i, format!("value{}", i));
    /// }
    ///
    /// // Different range syntaxes
    /// let range1: Vec<_> = tree.range(3..7).map(|(k, v)| (*k, v.clone())).collect();
    /// assert_eq!(range1, vec![(3, "value3".to_string()), (4, "value4".to_string()),
    ///                         (5, "value5".to_string()), (6, "value6".to_string())]);
    ///
    /// let range2: Vec<_> = tree.range(3..=7).map(|(k, v)| (*k, v.clone())).collect();
    /// assert_eq!(range2, vec![(3, "value3".to_string()), (4, "value4".to_string()),
    ///                         (5, "value5".to_string()), (6, "value6".to_string()),
    ///                         (7, "value7".to_string())]);
    ///
    /// let range3: Vec<_> = tree.range(5..).map(|(k, v)| *k).collect();
    /// assert_eq!(range3, vec![5, 6, 7, 8, 9]);
    ///
    /// let range4: Vec<_> = tree.range(..5).map(|(k, v)| *k).collect();
    /// assert_eq!(range4, vec![0, 1, 2, 3, 4]);
    ///
    /// let range5: Vec<_> = tree.range(..).map(|(k, v)| *k).collect();
    /// assert_eq!(range5, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// ```
    pub fn range<R>(&self, range: R) -> RangeIterator<'_, K, V>
    where
        R: RangeBounds<K>,
    {
        let (start_info, skip_first, end_info) = self.resolve_range_bounds(range);
        RangeIterator::new_with_skip_owned(self, start_info, skip_first, end_info)
    }

    /// Returns the first key-value pair in the tree.
    pub fn first(&self) -> Option<(&K, &V)> {
        self.items().next()
    }

    /// Returns the last key-value pair in the tree.
    pub fn last(&self) -> Option<(&K, &V)> {
        self.items().last()
    }

    // ============================================================================
    // RANGE QUERY HELPERS
    // ============================================================================

    fn resolve_range_bounds<R>(
        &self,
        range: R,
    ) -> (
        Option<(NodeId, usize)>,
        bool,
        Option<(K, bool)>,
    )
    where
        R: RangeBounds<K>,
    {
        // Optimize start bound resolution - eliminate redundant Option handling
        let (start_info, skip_first) = match range.start_bound() {
            Bound::Included(key) => (self.find_range_start(key), false),
            Bound::Excluded(key) => (self.find_range_start(key), true),
            Bound::Unbounded => (self.get_first_leaf_id().map(|id| (id, 0)), false),
        };

        // Avoid cloning end bound key when possible
        let end_info = match range.end_bound() {
            Bound::Included(key) => Some((key.clone(), true)),
            Bound::Excluded(key) => Some((key.clone(), false)),
            Bound::Unbounded => None,
        };

        (start_info, skip_first, end_info)
    }

    // ============================================================================
    // RANGE QUERY OPTIMIZATION HELPERS
    // ============================================================================

    /// Find the leaf node and index where a range should start
    fn find_range_start(&self, start_key: &K) -> Option<(NodeId, usize)> {
        let mut current = &self.root;

        // Navigate down to leaf level
        loop {
            match current {
                NodeRef::Leaf(leaf_id, _) => {
                    let leaf = self.get_leaf(*leaf_id)?;
                    
                    // Use binary search instead of linear search for better performance
                    let index = match leaf.keys.binary_search(start_key) {
                        Ok(exact_index) => exact_index,     // Found exact key
                        Err(insert_index) => insert_index,  // First key >= start_key
                    };

                    if index < leaf.keys.len() {
                        return Some((*leaf_id, index));
                    } else if leaf.next != NULL_NODE {
                        // All keys in this leaf are < start_key, try next leaf
                        // Check if next leaf exists and has keys without redundant arena lookup
                        return Some((leaf.next, 0));
                    } else {
                        // No more leaves
                        return None;
                    }
                }
                NodeRef::Branch(branch_id, _) => {
                    let branch = self.get_branch(*branch_id)?;
                    let child_index = branch.find_child_index(start_key);

                    if child_index < branch.children.len() {
                        current = &branch.children[child_index];
                    } else {
                        return None;
                    }
                }
            }
        }
    }

    /// Get the ID of the first (leftmost) leaf in the tree
    fn get_first_leaf_id(&self) -> Option<NodeId> {
        let mut current = &self.root;

        loop {
            match current {
                NodeRef::Leaf(leaf_id, _) => return Some(*leaf_id),
                NodeRef::Branch(branch_id, _) => {
                    if let Some(branch) = self.get_branch(*branch_id) {
                        if !branch.children.is_empty() {
                            current = &branch.children[0];
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
            }
        }
    }

    // ============================================================================
    // ENHANCED ARENA-BASED ALLOCATION FOR LEAF NODES
    // ============================================================================

    // allocate_leaf method moved to insert_operations.rs module

    /// Deallocate a leaf node from the arena.
    pub fn deallocate_leaf(&mut self, id: NodeId) -> Option<LeafNode<K, V>> {
        self.leaf_arena.deallocate(id)
    }

    // Arena access methods moved to get_operations.rs module

    /// Get the number of free leaf nodes in the arena.
    pub fn free_leaf_count(&self) -> usize {
        self.leaf_arena.free_count()
    }

    /// Get the number of allocated leaf nodes in the arena.
    pub fn allocated_leaf_count(&self) -> usize {
        self.leaf_arena.allocated_count()
    }

    /// Get the leaf arena utilization ratio.
    pub fn leaf_utilization(&self) -> f64 {
        self.leaf_arena.utilization()
    }

    // ============================================================================
    // ARENA STATISTICS
    // ============================================================================

    /// Get statistics for the leaf node arena.
    pub fn leaf_arena_stats(&self) -> CompactArenaStats {
        self.leaf_arena.stats()
    }

    /// Get statistics for the branch node arena.
    pub fn branch_arena_stats(&self) -> CompactArenaStats {
        self.branch_arena.stats()
    }

    /// Set the next pointer of a leaf node in the arena.
    pub fn set_leaf_next(&mut self, id: NodeId, next_id: NodeId) -> bool {
        self.get_leaf_mut(id)
            .map(|leaf| {
                leaf.next = next_id;
                true
            })
            .unwrap_or(false)
    }

    /// Get the next pointer of a leaf node in the arena.
    // get_leaf_next method moved to get_operations.rs module

    // ============================================================================
    // CHILD LOOKUP HELPERS (Phase 2)
    // ============================================================================

    /// Find the child index and `NodeRef` for `key` in the specified branch,
    /// returning `None` if the branch does not exist or index is out of range.
    pub fn find_child(&self, branch_id: NodeId, key: &K) -> Option<(usize, NodeRef<K, V>)> {
        self.get_branch(branch_id).and_then(|branch| {
            let idx = branch.find_child_index(key);
            branch.children.get(idx).cloned().map(|child| (idx, child))
        })
    }

    /// Mutable version of `find_child`.
    pub fn find_child_mut(&mut self, branch_id: NodeId, key: &K) -> Option<(usize, NodeRef<K, V>)> {
        self.get_branch_mut(branch_id).and_then(|branch| {
            let idx = branch.find_child_index(key);
            branch.children.get(idx).cloned().map(|child| (idx, child))
        })
    }

    // ============================================================================
    // ENHANCED ARENA-BASED ALLOCATION FOR BRANCH NODES
    // ============================================================================

    // allocate_branch method moved to insert_operations.rs module

    /// Deallocate a branch node from the arena.
    pub fn deallocate_branch(&mut self, id: NodeId) -> Option<BranchNode<K, V>> {
        self.branch_arena.deallocate(id)
    }

    // Branch arena access methods moved to get_operations.rs module

    /// Unsafe fast access to leaf node (no bounds checking)
    /// SAFETY: Caller must ensure id is valid and allocated
    pub unsafe fn get_leaf_unchecked(&self, id: NodeId) -> &LeafNode<K, V> {
        self.leaf_arena.get_unchecked(id)
    }

    /// Unsafe fast access to branch node (no bounds checking)
    /// SAFETY: Caller must ensure id is valid and allocated
    pub unsafe fn get_branch_unchecked(&self, id: NodeId) -> &BranchNode<K, V> {
        self.branch_arena.get_unchecked(id)
    }

    

    // ============================================================================
    // OTHER HELPERS (TEST HELPERS)
    // ============================================================================

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

    /// Count the number of leaf and branch nodes actually in the tree structure.
    pub fn count_nodes_in_tree(&self) -> (usize, usize) {
        if matches!(self.root, NodeRef::Leaf(_, _)) {
            // Single leaf root
            (1, 0)
        } else {
            self.count_nodes_recursive(&self.root)
        }
    }

    /// Recursively count nodes in the tree.
    fn count_nodes_recursive(&self, node: &NodeRef<K, V>) -> (usize, usize) {
        match node {
            NodeRef::Leaf(_, _) => (1, 0), // Found a leaf
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.get_branch(*id) {
                    let mut total_leaves = 0;
                    let mut total_branches = 1; // Count this branch

                    // Recursively count in all children
                    for child in &branch.children {
                        let (child_leaves, child_branches) = self.count_nodes_recursive(child);
                        total_leaves += child_leaves;
                        total_branches += child_branches;
                    }

                    (total_leaves, total_branches)
                } else {
                    // Invalid branch reference
                    (0, 0)
                }
            }
        }
    }

    /// Check that all leaf nodes in the tree are reachable via the linked list.
    fn check_leaf_linked_list_completeness(&self) -> TreeResult<()> {
        // Collect all leaf IDs from the tree structure
        let mut tree_leaf_ids = Vec::new();
        self.collect_leaf_ids(&self.root, &mut tree_leaf_ids);
        tree_leaf_ids.sort();

        // Collect all leaf IDs from the linked list traversal
        let mut linked_list_ids = Vec::new();
        if let Some(first_id) = self.get_first_leaf_id() {
            let mut current_id = Some(first_id);
            while let Some(id) = current_id {
                linked_list_ids.push(id);
                current_id = self.get_leaf(id).and_then(|leaf| {
                    if leaf.next == crate::NULL_NODE {
                        None
                    } else {
                        Some(leaf.next)
                    }
                });
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

    fn collect_leaf_sizes(&self, node: &NodeRef<K, V>, sizes: &mut Vec<usize>) {
        match node {
            NodeRef::Leaf(id, _) => {
                let size = self.get_leaf(*id).map(|leaf| leaf.keys.len()).unwrap_or(0);
                sizes.push(size);
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.get_branch(*id) {
                    for child in &branch.children {
                        self.collect_leaf_sizes(child, sizes);
                    }
                }
                // Missing arena branch contributes no leaf sizes (do nothing)
            }
        }
    }

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
                        leaf.keys.len()
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
                    if leaf.keys.len() != leaf.values.len() {
                        return false; // Keys and values must have same length
                    }

                    // Check that keys are sorted
                    for i in 1..leaf.keys.len() {
                        if leaf.keys[i - 1] >= leaf.keys[i] {
                            return false; // Keys must be in ascending order
                        }
                    }

                    // Check capacity constraints
                    if leaf.keys.len() > self.capacity {
                        return false; // Node exceeds capacity
                    }

                    // Check minimum occupancy
                    if !leaf.keys.is_empty() && leaf.is_underfull() {
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
                        if !leaf.keys.is_empty() && &leaf.keys[0] < min {
                            return false; // First key must be >= min_key
                        }
                    }
                    if let Some(max) = max_key {
                        if !leaf.keys.is_empty() && &leaf.keys[leaf.keys.len() - 1] >= max {
                            return false; // Last key must be < max_key
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
                        return false; // Must have one more child than keys
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
}

// Default implementation moved to construction.rs module



impl<K: Ord + Clone, V: Clone> LeafNode<K, V> {
    // ============================================================================
    // CONSTRUCTION
    // ============================================================================

    // Construction methods moved to construction.rs module

    /// Get a reference to the keys in this leaf node.
    pub fn keys(&self) -> &Vec<K> {
        &self.keys
    }

    /// Get a reference to the values in this leaf node.
    pub fn values(&self) -> &Vec<V> {
        &self.values
    }

    /// Get a mutable reference to the values in this leaf node.
    pub fn values_mut(&mut self) -> &mut Vec<V> {
        &mut self.values
    }

    // ============================================================================
    // GET OPERATIONS
    // ============================================================================

    // GET operations moved to get_operations.rs module

    // ============================================================================
    // INSERT OPERATIONS
    // ============================================================================

    /// Insert a key-value pair and handle splitting if necessary.
    pub fn insert(&mut self, key: K, value: V) -> InsertResult<K, V> {
        // Do binary search once and use the result throughout
        match self.keys.binary_search(&key) {
            Ok(index) => {
                // Key already exists, update the value
                let old_value = std::mem::replace(&mut self.values[index], value);
                InsertResult::Updated(Some(old_value))
            }
            Err(index) => {
                // Key doesn't exist, need to insert
                // Check if split is needed BEFORE inserting
                if !self.is_full() {
                    // Room to insert without splitting
                    self.insert_at_index(index, key, value);
                    // Simple insertion - no split needed
                    return InsertResult::Updated(None);
                }
                // Leaf is at capacity, split first then insert
                let mut new_leaf_data = self.split();
                let separator_key = new_leaf_data.keys[0].clone();

                // Determine which leaf should receive the new key
                if key < separator_key {
                    // Insert into the current (left) leaf
                    self.insert_at_index(index, key, value);
                } else {
                    // Insert into the new (right) leaf
                    match new_leaf_data.keys.binary_search(&key) {
                        Ok(_) => {
                            // This should never happen with correct split logic
                            // Return error instead of panic to maintain stability
                            return InsertResult::Error(BPlusTreeError::data_integrity(
                                "Leaf split operation",
                                "Key unexpectedly found in new leaf after split",
                            ));
                        }
                        Err(new_index) => {
                            new_leaf_data.insert_at_index(new_index, key, value);
                        }
                    }
                }

                // Return the leaf data for arena allocation
                InsertResult::Split {
                    old_value: None,
                    new_node_data: SplitNodeData::Leaf(new_leaf_data),
                    separator_key,
                }
            }
        }
    }

    // ============================================================================
    // HELPERS FOR INSERT OPERATIONS
    // ============================================================================

    /// Insert a key-value pair at the specified index.
    fn insert_at_index(&mut self, index: usize, key: K, value: V) {
        self.keys.insert(index, key);
        self.values.insert(index, value);
    }

    /// Split this leaf node, returning the new right node.
    pub fn split(&mut self) -> LeafNode<K, V> {
        // For B+ trees, we need to ensure both resulting nodes have at least min_keys
        // When splitting a full node (capacity keys), we want to distribute them
        // so that both nodes have at least min_keys
        let min_keys = self.min_keys();
        let total_keys = self.keys.len();

        // Calculate split point for better balance while ensuring both sides have at least min_keys
        // Use a more balanced split: aim for roughly equal distribution
        let mid = total_keys.div_ceil(2); // Round up for odd numbers

        // Ensure the split point respects minimum requirements
        let mid = mid.max(min_keys).min(total_keys - min_keys);

        // Verify this split is valid
        debug_assert!(mid >= min_keys, "Left side would be underfull");
        debug_assert!(
            total_keys - mid >= min_keys,
            "Right side would be underfull"
        );

        // Create new leaf for right half (no Box allocation)
        let mut new_leaf = LeafNode::new(self.capacity);

        // Move right half of keys/values to new leaf
        new_leaf.keys = self.keys.split_off(mid);
        new_leaf.values = self.values.split_off(mid);

        // Maintain the linked list: new leaf inherits our next pointer
        new_leaf.next = self.next;
        // Note: The caller must update self.next to point to the new leaf's ID
        // This can't be done here as we don't know the new leaf's arena ID yet

        new_leaf
    }

    // ============================================================================
    // DELETE OPERATIONS
    // ============================================================================

    /// Remove a key from this leaf node.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        match self.keys.binary_search(key) {
            Ok(index) => {
                self.keys.remove(index);
                Some(self.values.remove(index))
            }
            Err(_) => None,
        }
    }

    // ============================================================================
    // OTHER API OPERATIONS
    // ============================================================================

    /// Returns the number of key-value pairs in this leaf node.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns true if this leaf node is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Returns true if this leaf node is at capacity.
    pub fn is_full(&self) -> bool {
        self.keys.len() >= self.capacity
    }

    /// Returns true if this leaf node needs to be split.
    /// We allow one extra key beyond capacity to ensure proper splitting.
    pub fn needs_split(&self) -> bool {
        self.keys.len() > self.capacity
    }

    /// Returns true if this leaf node is underfull (below minimum occupancy).
    pub fn is_underfull(&self) -> bool {
        self.keys.len() < self.min_keys()
    }

    /// Returns true if this leaf can donate a key to a sibling.
    pub fn can_donate(&self) -> bool {
        self.keys.len() > self.min_keys()
    }

    // ============================================================================
    // OTHER HELPERS
    // ============================================================================

    /// Returns the minimum number of keys this leaf should have.
    pub fn min_keys(&self) -> usize {
        // For leaf nodes, minimum is floor(capacity / 2)
        // Exception: root can have fewer keys
        self.capacity / 2
    }

    // ============================================================================
    // BORROWING AND MERGING HELPERS
    // ============================================================================

    /// Borrow the last key-value pair from this leaf (used when this is the left sibling)
    pub fn borrow_last(&mut self) -> Option<(K, V)> {
        if self.keys.is_empty() || !self.can_donate() {
            return None;
        }
        Some((self.keys.pop().unwrap(), self.values.pop().unwrap()))
    }

    /// Borrow the first key-value pair from this leaf (used when this is the right sibling)
    pub fn borrow_first(&mut self) -> Option<(K, V)> {
        if self.keys.is_empty() || !self.can_donate() {
            return None;
        }
        Some((self.keys.remove(0), self.values.remove(0)))
    }

    /// Accept a borrowed key-value pair at the beginning (from left sibling)
    pub fn accept_from_left(&mut self, key: K, value: V) {
        self.keys.insert(0, key);
        self.values.insert(0, value);
    }

    /// Accept a borrowed key-value pair at the end (from right sibling)
    pub fn accept_from_right(&mut self, key: K, value: V) {
        self.keys.push(key);
        self.values.push(value);
    }

    /// Merge all content from another leaf into this one, returning the other's next pointer
    pub fn merge_from(&mut self, other: &mut LeafNode<K, V>) -> NodeId {
        self.keys.append(&mut other.keys);
        self.values.append(&mut other.values);
        let other_next = other.next;
        other.next = NULL_NODE; // Clear the other's next pointer
        other_next
    }

    /// Extract all content from this leaf (used for merging)
    pub fn extract_all(&mut self) -> (Vec<K>, Vec<V>, NodeId) {
        let keys = std::mem::take(&mut self.keys);
        let values = std::mem::take(&mut self.values);
        let next = self.next;
        self.next = NULL_NODE;
        (keys, values, next)
    }
}

// Default implementation moved to construction.rs module

impl<K: Ord + Clone, V: Clone> BranchNode<K, V> {
    // ============================================================================
    // CONSTRUCTION
    // ============================================================================

    // Construction methods moved to construction.rs module

    // ============================================================================
    // GET OPERATIONS
    // ============================================================================

    // GET operations moved to get_operations.rs module

    // ============================================================================
    // INSERT OPERATIONS
    // ============================================================================

    /// Insert a separator key and new child into this branch node.
    /// Returns None if no split needed, or Some((new_branch_data, promoted_key)) if split occurred.
    /// The caller should handle arena allocation for the split data.
    pub fn insert_child_and_split_if_needed(
        &mut self,
        child_index: usize,
        separator_key: K,
        new_child: NodeRef<K, V>,
    ) -> Option<(BranchNode<K, V>, K)> {
        // Check if split is needed BEFORE inserting
        if self.is_full() {
            // Branch is at capacity, need to handle split
            // For branches, we MUST insert first because split promotes a key
            // With capacity=4: 4 keys → split needs 5 keys (2 left + 1 promoted + 2 right)
            self.keys.insert(child_index, separator_key);
            self.children.insert(child_index + 1, new_child);
            // Return raw data - caller should allocate through arena
            Some(self.split_data())
        } else {
            // Room to insert without splitting
            self.keys.insert(child_index, separator_key);
            self.children.insert(child_index + 1, new_child);
            None
        }
    }

    // ============================================================================
    // HELPERS FOR INSERT OPERATIONS
    // ============================================================================

    /// Split this branch node, returning the new right node and promoted key.
    pub fn split_data(&mut self) -> (BranchNode<K, V>, K) {
        // For branch nodes, we need to ensure both resulting nodes have at least min_keys
        // The middle key gets promoted, so we need at least min_keys on each side
        let min_keys = self.min_keys();
        let total_keys = self.keys.len();

        // For branch splits, we promote the middle key, so we need:
        // - Left side: min_keys keys
        // - Middle: 1 key (promoted)
        // - Right side: min_keys keys
        // Total needed: min_keys + 1 + min_keys
        let mid = min_keys;

        // Verify this split is valid
        debug_assert!(mid < total_keys, "Not enough keys to promote one");
        debug_assert!(mid >= min_keys, "Left side would be underfull");
        debug_assert!(total_keys - mid > min_keys, "Right side would be underfull");

        // The middle key gets promoted to the parent
        let promoted_key = self.keys[mid].clone();

        let mut right_half = BranchNode::new(self.capacity);
        right_half.keys = self.keys.split_off(mid + 1);
        right_half.children = self.children.split_off(mid + 1);
        self.keys.truncate(mid); // Remove the promoted key from left side

        (right_half, promoted_key)
    }

    // ============================================================================
    // HELPERS FOR DELETE OPERATIONS
    // ============================================================================

    /// Merge this branch with the right sibling using the given separator.
    /// Returns true if merge was successful.
    pub fn merge_with_right(&mut self, mut right: BranchNode<K, V>, separator: K) -> bool {
        // Add the separator key
        self.keys.push(separator);

        // Move all keys and children from right to this node
        self.keys.append(&mut right.keys);
        self.children.append(&mut right.children);

        true
    }

    // ============================================================================
    // OTHER API OPERATIONS
    // ============================================================================

    /// Returns true if this branch node is at capacity.
    pub fn is_full(&self) -> bool {
        self.keys.len() >= self.capacity
    }

    /// Returns true if this branch node needs to be split.
    /// We allow one extra key beyond capacity to ensure proper splitting.
    pub fn needs_split(&self) -> bool {
        self.keys.len() > self.capacity
    }

    /// Returns true if this branch node is underfull (below minimum occupancy).
    pub fn is_underfull(&self) -> bool {
        self.keys.len() < self.min_keys()
    }

    /// Returns true if this branch can donate a key to a sibling.
    pub fn can_donate(&self) -> bool {
        self.keys.len() > self.min_keys()
    }

    // ============================================================================
    // OTHER HELPERS
    // ============================================================================

    /// Returns the minimum number of keys this branch should have.
    pub fn min_keys(&self) -> usize {
        // For branch nodes, minimum is floor(capacity / 2)
        // Exception: root can have fewer keys
        self.capacity / 2
    }

    // ============================================================================
    // BORROWING AND MERGING HELPERS
    // ============================================================================

    /// Borrow the last key and child from this branch (used when this is the left sibling)
    pub fn borrow_last(&mut self) -> Option<(K, NodeRef<K, V>)> {
        if self.keys.is_empty() || !self.can_donate() {
            return None;
        }
        let key = self.keys.pop()?;
        let child = self.children.pop()?;
        Some((key, child))
    }

    /// Borrow the first key and child from this branch (used when this is the right sibling)
    pub fn borrow_first(&mut self) -> Option<(K, NodeRef<K, V>)> {
        if self.keys.is_empty() || !self.can_donate() {
            return None;
        }
        let key = self.keys.remove(0);
        let child = self.children.remove(0);
        Some((key, child))
    }

    /// Accept a borrowed key and child at the beginning (from left sibling)
    /// The separator becomes the first key, and the moved child becomes the first child
    pub fn accept_from_left(
        &mut self,
        separator: K,
        moved_key: K,
        moved_child: NodeRef<K, V>,
    ) -> K {
        self.keys.insert(0, separator);
        self.children.insert(0, moved_child);
        moved_key // Return the new separator for parent
    }

    /// Accept a borrowed key and child at the end (from right sibling)
    /// The separator becomes the last key, and the moved child becomes the last child
    pub fn accept_from_right(
        &mut self,
        separator: K,
        moved_key: K,
        moved_child: NodeRef<K, V>,
    ) -> K {
        self.keys.push(separator);
        self.children.push(moved_child);
        moved_key // Return the new separator for parent
    }

    /// Merge all content from another branch into this one, with separator from parent
    pub fn merge_from(&mut self, separator: K, other: &mut BranchNode<K, V>) {
        // Add separator key from parent
        self.keys.push(separator);
        // Add all keys and children from other
        self.keys.append(&mut other.keys);
        self.children.append(&mut other.children);
    }
}

// Default implementation moved to construction.rs module

/// Iterator over key-value pairs in the B+ tree using the leaf linked list.
pub struct ItemIterator<'a, K, V> {
    tree: &'a BPlusTreeMap<K, V>,
    current_leaf_id: Option<NodeId>,
    current_leaf_ref: Option<&'a LeafNode<K, V>>, // CACHED leaf reference
    current_leaf_index: usize,
    end_key: Option<&'a K>,
    end_bound_key: Option<K>,
    end_inclusive: bool,
    finished: bool,
}

/// Fast iterator over key-value pairs using unsafe arena access for better performance.
pub struct FastItemIterator<'a, K, V> {
    tree: &'a BPlusTreeMap<K, V>,
    current_leaf_id: Option<NodeId>,
    current_leaf_ref: Option<&'a LeafNode<K, V>>, // CACHED leaf reference
    current_leaf_index: usize,
    finished: bool,
}

impl<'a, K: Ord + Clone, V: Clone> ItemIterator<'a, K, V> {
    fn new(tree: &'a BPlusTreeMap<K, V>) -> Self {
        // Start with the first (leftmost) leaf in the tree
        let leftmost_id = tree.get_first_leaf_id();
        
        // Get the initial leaf reference if we have a starting leaf
        let current_leaf_ref = leftmost_id.and_then(|id| tree.get_leaf(id));

        Self {
            tree,
            current_leaf_id: leftmost_id,
            current_leaf_ref,
            current_leaf_index: 0,
            end_key: None,
            end_bound_key: None,
            end_inclusive: false,
            finished: false,
        }
    }


    /// Start from specific position with full bound control using owned keys
    fn new_from_position_with_bounds(
        tree: &'a BPlusTreeMap<K, V>,
        start_leaf_id: NodeId,
        start_index: usize,
        end_bound: Bound<&K>,
    ) -> Self {
        let (end_bound_key, end_inclusive) = match end_bound {
            Bound::Included(key) => (Some(key.clone()), true),
            Bound::Excluded(key) => (Some(key.clone()), false),
            Bound::Unbounded => (None, false),
        };

        // Get the initial leaf reference
        let current_leaf_ref = tree.get_leaf(start_leaf_id);

        Self {
            tree,
            current_leaf_id: Some(start_leaf_id),
            current_leaf_ref,
            current_leaf_index: start_index,
            end_key: None,
            end_bound_key,
            end_inclusive,
            finished: false,
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> Iterator for ItemIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            // Use cached leaf reference - NO arena lookup here!
            let result = self
                .current_leaf_ref
                .and_then(|leaf| self.try_get_next_item(leaf));

            match result {
                Some(item) => return Some(item),
                None => {
                    // Either no current leaf or no more items in current leaf
                    if !self.advance_to_next_leaf().unwrap_or(false) {
                        self.finished = true;
                        return None;
                    }
                    // Continue loop with next leaf
                }
            }
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> ItemIterator<'a, K, V> {
    /// Helper method to try getting the next item from the current leaf
    fn try_get_next_item(&mut self, leaf: &'a LeafNode<K, V>) -> Option<(&'a K, &'a V)> {
        // Check if we have more items in the current leaf
        if self.current_leaf_index >= leaf.keys.len() {
            return None;
        }

        let key = &leaf.keys[self.current_leaf_index];
        let value = &leaf.values[self.current_leaf_index];

        // Check if we've reached the end bound using Option combinators
        let beyond_end = self
            .end_key
            .map(|end| key >= end)
            .or_else(|| {
                self.end_bound_key.as_ref().map(|end| {
                    if self.end_inclusive {
                        key > end
                    } else {
                        key >= end
                    }
                })
            })
            .unwrap_or(false);

        if beyond_end {
            self.finished = true;
            return None;
        }

        self.current_leaf_index += 1;
        Some((key, value))
    }

    /// Helper method to advance to the next leaf
    /// Returns Some(true) if successfully advanced, Some(false) if no more leaves, None if invalid leaf
    fn advance_to_next_leaf(&mut self) -> Option<bool> {
        // Use cached leaf reference to get next leaf ID
        let leaf = self.current_leaf_ref?;
        
        let next_leaf_id = (leaf.next != NULL_NODE).then_some(leaf.next);
        
        // Update both ID and cached reference - this is the ONLY arena access during iteration
        self.current_leaf_id = next_leaf_id;
        self.current_leaf_ref = next_leaf_id.and_then(|id| self.tree.get_leaf(id));
        self.current_leaf_index = 0;

        Some(self.current_leaf_id.is_some())
    }
}

/// Iterator over keys in the B+ tree.
pub struct KeyIterator<'a, K, V> {
    items: ItemIterator<'a, K, V>,
}

impl<'a, K: Ord + Clone, V: Clone> KeyIterator<'a, K, V> {
    fn new(tree: &'a BPlusTreeMap<K, V>) -> Self {
        Self {
            items: ItemIterator::new(tree),
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> Iterator for KeyIterator<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next().map(|(k, _)| k)
    }
}

/// Iterator over values in the B+ tree.
pub struct ValueIterator<'a, K, V> {
    items: ItemIterator<'a, K, V>,
}

impl<'a, K: Ord + Clone, V: Clone> ValueIterator<'a, K, V> {
    fn new(tree: &'a BPlusTreeMap<K, V>) -> Self {
        Self {
            items: ItemIterator::new(tree),
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> Iterator for ValueIterator<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next().map(|(_, v)| v)
    }
}

/// Optimized iterator over a range of key-value pairs in the B+ tree.
/// Uses tree navigation to find start, then linked list traversal for efficiency.
pub struct RangeIterator<'a, K, V> {
    iterator: Option<ItemIterator<'a, K, V>>,
    skip_first: bool,
    first_key: Option<K>,
}

impl<'a, K: Ord + Clone, V: Clone> RangeIterator<'a, K, V> {
    fn new_with_skip_owned(
        tree: &'a BPlusTreeMap<K, V>,
        start_info: Option<(NodeId, usize)>,
        skip_first: bool,
        end_info: Option<(K, bool)>, // (end_key, is_inclusive)
    ) -> Self {
        let (iterator, first_key) = start_info
            .map(|(leaf_id, index)| {
                // Create iterator with appropriate end bound using Option combinators
                let end_bound = end_info
                    .as_ref()
                    .map(|(key, is_inclusive)| {
                        if *is_inclusive {
                            Bound::Included(key)
                        } else {
                            Bound::Excluded(key)
                        }
                    })
                    .unwrap_or(Bound::Unbounded);

                let iter =
                    ItemIterator::new_from_position_with_bounds(tree, leaf_id, index, end_bound);

                // Extract first key if needed for skipping, avoid redundant arena lookup
                let first_key = if skip_first {
                    tree.get_leaf(leaf_id)
                        .and_then(|leaf| leaf.keys.get(index))
                        .cloned()
                } else {
                    None
                };

                (Some(iter), first_key)
            })
            .unwrap_or((None, None));

        Self {
            iterator,
            skip_first,
            first_key,
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> Iterator for RangeIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.iterator.as_mut()?.next()?;

            // Handle excluded start bound on first iteration
            if self.skip_first {
                self.skip_first = false;
                if let Some(ref first_key) = self.first_key {
                    if item.0 == first_key {
                        // Skip this item and continue to next
                        continue;
                    }
                }
            }

            return Some(item);
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> FastItemIterator<'a, K, V> {
    fn new(tree: &'a BPlusTreeMap<K, V>) -> Self {
        // Start with the first (leftmost) leaf in the tree
        let leftmost_id = tree.get_first_leaf_id();
        
        // Get the initial leaf reference if we have a starting leaf
        let current_leaf_ref = leftmost_id.and_then(|id| unsafe { Some(tree.get_leaf_unchecked(id)) });

        Self {
            tree,
            current_leaf_id: leftmost_id,
            current_leaf_ref,
            current_leaf_index: 0,
            finished: false,
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> Iterator for FastItemIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            // Use cached leaf reference - NO arena lookup here!
            let leaf = self.current_leaf_ref?;
            
            if self.current_leaf_index < leaf.keys.len() {
                let key = &leaf.keys[self.current_leaf_index];
                let value = &leaf.values[self.current_leaf_index];
                self.current_leaf_index += 1;
                return Some((key, value));
            } else {
                // Move to next leaf - this is the ONLY arena access during iteration
                if leaf.next != NULL_NODE {
                    self.current_leaf_id = Some(leaf.next);
                    self.current_leaf_ref = unsafe { Some(self.tree.get_leaf_unchecked(leaf.next)) };
                    self.current_leaf_index = 0;
                } else {
                    self.finished = true;
                    return None;
                }
            }
        }
    }
}

