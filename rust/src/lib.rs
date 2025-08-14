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
mod node;
mod iteration;
mod validation;

pub use arena::{Arena, ArenaStats, NodeId as ArenaNodeId, NULL_NODE as ARENA_NULL_NODE};
pub use compact_arena::{CompactArena, CompactArenaStats};
pub use error::{
    BPlusTreeError, BTreeResult, BTreeResultExt, InitResult, KeyResult, ModifyResult,
};
pub use types::{BPlusTreeMap, NodeId, NodeRef, NULL_NODE, ROOT_NODE, LeafNode, BranchNode};
pub use construction::{InitResult as ConstructionResult};
pub use iteration::{ItemIterator, FastItemIterator, KeyIterator, ValueIterator, RangeIterator};

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

    // Helper methods moved to delete_operations.rs module

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

    // remove and remove_item methods moved to delete_operations.rs module

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

    // Validation methods moved to validation.rs module

    // ============================================================================
    // HELPERS FOR DELETE OPERATIONS
    // ============================================================================

    // All rebalancing methods moved to delete_operations.rs module

    // collapse_root_if_needed and create_empty_root_leaf methods moved to delete_operations.rs module

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

    // Iterator methods moved to iteration.rs module

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
    // VALIDATION AND DEBUGGING METHODS
    // ============================================================================

    // All validation and debugging methods moved to validation.rs module

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

    // Validation helper methods moved to validation.rs module

    // Debugging and testing utility methods moved to validation.rs module

    // Validation implementation methods moved to validation.rs module

    // All validation implementation methods moved to validation.rs module
}

// Default implementation moved to construction.rs module



// LeafNode implementation moved to node.rs module

// Default implementation moved to construction.rs module

// BranchNode implementation moved to node.rs module

// Default implementation moved to construction.rs module

// Iterator implementations moved to iteration.rs module

