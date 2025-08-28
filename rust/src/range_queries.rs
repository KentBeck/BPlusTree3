//! Range query operations for BPlusTreeMap.
//!
//! This module contains all range-related operations including range iteration,
//! bounds resolution, and range optimization algorithms.

use crate::iteration::RangeIterator;
use crate::types::{BPlusTreeMap, NodeId, NodeRef};
use std::ops::{Bound, RangeBounds};

/// Type alias for complex range analysis result
type RangeAnalysisResult<K> = (Option<(NodeId, usize)>, bool, Option<(K, bool)>);

// ============================================================================
// RANGE QUERY OPERATIONS
// ============================================================================

impl<K: Ord + Clone, V: Clone> BPlusTreeMap<K, V> {
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

    /// Resolve range bounds into start position, skip flag, and end information.
    pub fn resolve_range_bounds<R>(&self, range: R) -> RangeAnalysisResult<K>
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

    /// Find the starting position for a range query.
    fn find_range_start(&self, key: &K) -> Option<(NodeId, usize)> {
        self.find_leaf_for_key(key)
    }

    /// Find the leaf node and index where a key should be located.
    fn find_leaf_for_key(&self, key: &K) -> Option<(NodeId, usize)> {
        let mut current = &self.root;

        loop {
            match current {
                NodeRef::Leaf(leaf_id, _) => {
                    if let Some(leaf) = self.get_leaf(*leaf_id) {
                        // Find the position where this key would be inserted
                        let index = match leaf.binary_search_keys(key) {
                            Ok(idx) => idx,  // Key found at exact position
                            Err(idx) => idx, // Key would be inserted at this position
                        };
                        return Some((*leaf_id, index));
                    } else {
                        return None;
                    }
                }
                NodeRef::Branch(branch_id, _) => {
                    if let Some(branch) = self.get_branch(*branch_id) {
                        let child_index = branch.find_child_index(key);
                        if let Some(child) = branch.children.get(child_index) {
                            current = child;
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
    // RANGE OPTIMIZATION HELPERS
    // ============================================================================

    // (Removed dead code: optimize_range_query, estimate_range_size, find_last_leaf_position)
}
