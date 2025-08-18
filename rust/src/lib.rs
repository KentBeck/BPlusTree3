//! B+ Tree implementation in Rust with dict-like API.
//!
//! This module provides a B+ tree data structure with a dictionary-like interface,
//! supporting efficient insertion, deletion, lookup, and range queries.

// Range imports moved to range_queries.rs module

// Import our new modules
// arena.rs removed - only compact_arena.rs is used
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
mod tree_structure;
mod range_queries;

// Generic Arena removed - only CompactArena is used in the implementation
pub use compact_arena::{CompactArena, CompactArenaStats};
pub use error::{
    BPlusTreeError, BTreeResult, BTreeResultExt, InitResult, KeyResult, ModifyResult,
};
pub use types::{BPlusTreeMap, NodeId, NodeRef, NULL_NODE, ROOT_NODE, LeafNode, BranchNode};
pub use construction::{InitResult as ConstructionResult};
pub use iteration::{ItemIterator, FastItemIterator, KeyIterator, ValueIterator, RangeIterator};

// PhantomData import moved to tree_structure.rs module

// Internal type imports removed - no longer needed in main lib.rs





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

    // Tree structure operations moved to tree_structure.rs module

    // Iterator methods moved to iteration.rs module

    // Range query operations moved to range_queries.rs module

    // Range query helper methods moved to range_queries.rs module

    // All arena management and tree structure methods moved to tree_structure.rs module

    

    // ============================================================================
    // VALIDATION AND DEBUGGING METHODS
    // ============================================================================

    // All validation and debugging methods moved to validation.rs module

    // Tree structure counting methods moved to tree_structure.rs module

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

