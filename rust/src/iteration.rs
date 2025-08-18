//! Iterator implementations for BPlusTreeMap.
//!
//! This module contains all iterator types and their implementations for the B+ tree,
//! including basic iteration, range iteration, and optimized fast iteration.

use crate::types::{BPlusTreeMap, LeafNode, NodeId, NULL_NODE};
use std::ops::Bound;

// ============================================================================
// ITERATOR STRUCTS
// ============================================================================

/// Iterator over key-value pairs in the B+ tree using the leaf linked list.
pub struct ItemIterator<'a, K, V> {
    tree: &'a BPlusTreeMap<K, V>,
    current_leaf_id: Option<NodeId>,
    pub current_leaf_ref: Option<&'a LeafNode<K, V>>, // CACHED leaf reference
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
    pub current_leaf_ref: Option<&'a LeafNode<K, V>>, // CACHED leaf reference
    current_leaf_index: usize,
    finished: bool,
}

/// Iterator over keys in the B+ tree.
pub struct KeyIterator<'a, K, V> {
    items: ItemIterator<'a, K, V>,
}

/// Iterator over values in the B+ tree.
pub struct ValueIterator<'a, K, V> {
    items: ItemIterator<'a, K, V>,
}

/// Optimized iterator over a range of key-value pairs in the B+ tree.
/// Uses tree navigation to find start, then linked list traversal for efficiency.
pub struct RangeIterator<'a, K, V> {
    iterator: Option<ItemIterator<'a, K, V>>,
    skip_first: bool,
    first_key: Option<K>,
}

// ============================================================================
// BPLUSTREE ITERATOR METHODS
// ============================================================================

impl<K: Ord + Clone, V: Clone> BPlusTreeMap<K, V> {
    /// Returns an iterator over all key-value pairs in sorted order.
    pub fn items(&self) -> ItemIterator<'_, K, V> {
        ItemIterator::new(self)
    }

    /// Returns a fast iterator over all key-value pairs using unsafe arena access.
    /// This provides better performance by skipping bounds checks.
    ///
    /// # Safety
    /// This is safe to use as long as the tree structure is valid and no concurrent
    /// modifications occur during iteration.
    pub fn items_fast(&self) -> FastItemIterator<'_, K, V> {
        FastItemIterator::new(self)
    }

    /// Returns an iterator over all keys in sorted order.
    pub fn keys(&self) -> KeyIterator<'_, K, V> {
        KeyIterator::new(self)
    }

    /// Returns an iterator over all values in key order.
    pub fn values(&self) -> ValueIterator<'_, K, V> {
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
        let start_bound = start_key.map_or(Bound::Unbounded, Bound::Included);
        let end_bound = end_key.map_or(Bound::Unbounded, Bound::Excluded);

        let (start_info, skip_first, end_info) =
            self.resolve_range_bounds((start_bound, end_bound));
        RangeIterator::new_with_skip_owned(self, start_info, skip_first, end_info)
    }
}

// ============================================================================
// ITEMITERATOR IMPLEMENTATION
// ============================================================================

impl<'a, K: Ord + Clone, V: Clone> ItemIterator<'a, K, V> {
    pub fn new(tree: &'a BPlusTreeMap<K, V>) -> Self {
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

    pub fn new_from_position_with_bounds(
        tree: &'a BPlusTreeMap<K, V>,
        leaf_id: NodeId,
        index: usize,
        end_bound: Bound<&'a K>,
    ) -> Self {
        let current_leaf_ref = tree.get_leaf(leaf_id);

        let (end_key, end_bound_key, end_inclusive) = match end_bound {
            Bound::Included(key) => (Some(key), None, true),
            Bound::Excluded(key) => (Some(key), None, false),
            Bound::Unbounded => (None, None, false),
        };

        Self {
            tree,
            current_leaf_id: Some(leaf_id),
            current_leaf_ref,
            current_leaf_index: index,
            end_key,
            end_bound_key,
            end_inclusive,
            finished: false,
        }
    }

    /// Helper method to try getting the next item from the current leaf
    fn try_get_next_item(&mut self, leaf: &'a LeafNode<K, V>) -> Option<(&'a K, &'a V)> {
        // Check if we have more items in the current leaf
        if self.current_leaf_index >= leaf.keys_len() {
            return None;
        }

        let key = leaf.get_key(self.current_leaf_index)?;
        let value = leaf.get_value(self.current_leaf_index)?;

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

// ============================================================================
// KEYITERATOR IMPLEMENTATION
// ============================================================================

impl<'a, K: Ord + Clone, V: Clone> KeyIterator<'a, K, V> {
    pub fn new(tree: &'a BPlusTreeMap<K, V>) -> Self {
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

// ============================================================================
// VALUEITERATOR IMPLEMENTATION
// ============================================================================

impl<'a, K: Ord + Clone, V: Clone> ValueIterator<'a, K, V> {
    pub fn new(tree: &'a BPlusTreeMap<K, V>) -> Self {
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

// ============================================================================
// RANGEITERATOR IMPLEMENTATION
// ============================================================================

impl<'a, K: Ord + Clone, V: Clone> RangeIterator<'a, K, V> {
    pub fn new_with_skip_owned(
        tree: &'a BPlusTreeMap<K, V>,
        start_info: Option<(NodeId, usize)>,
        skip_first: bool,
        end_info: Option<(K, bool)>, // (end_key, is_inclusive)
    ) -> Self {
        // Clone end_info to avoid borrowing issues
        let end_info_clone = end_info.clone();

        let (iterator, first_key) = start_info
            .map(move |(leaf_id, index)| {
                // Create iterator with unbounded end, we'll handle bounds in the iterator itself
                let end_bound = Bound::Unbounded;
                let mut iter =
                    ItemIterator::new_from_position_with_bounds(tree, leaf_id, index, end_bound);

                // Set the end bound using owned key if provided
                if let Some((key, is_inclusive)) = end_info_clone {
                    iter.end_bound_key = Some(key);
                    iter.end_inclusive = is_inclusive;
                }

                // Extract first key if needed for skipping, avoid redundant arena lookup
                let first_key = if skip_first {
                    tree.get_leaf(leaf_id)
                        .and_then(|leaf| leaf.get_key(index))
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

// ============================================================================
// FASTITEMITERATOR IMPLEMENTATION
// ============================================================================

impl<'a, K: Ord + Clone, V: Clone> FastItemIterator<'a, K, V> {
    pub fn new(tree: &'a BPlusTreeMap<K, V>) -> Self {
        // Start with the first (leftmost) leaf in the tree
        let leftmost_id = tree.get_first_leaf_id();

        // Get the initial leaf reference if we have a starting leaf
        let current_leaf_ref =
            leftmost_id.map(|id| unsafe { tree.get_leaf_unchecked(id) });

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

            if self.current_leaf_index < leaf.keys_len() {
                let key = leaf.get_key(self.current_leaf_index)?;
                let value = leaf.get_value(self.current_leaf_index)?;
                self.current_leaf_index += 1;
                return Some((key, value));
            } else {
                // Move to next leaf - this is the ONLY arena access during iteration
                if leaf.next != NULL_NODE {
                    self.current_leaf_id = Some(leaf.next);
                    self.current_leaf_ref =
                        unsafe { Some(self.tree.get_leaf_unchecked(leaf.next)) };
                    self.current_leaf_index = 0;
                } else {
                    self.finished = true;
                    return None;
                }
            }
        }
    }
}
