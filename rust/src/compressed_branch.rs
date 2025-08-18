//! Compressed branch node implementation for B+ tree.
//!
//! This module provides a memory-efficient branch node implementation that stores
//! keys and child node IDs in a compact byte array format. The node is designed
//! to fit exactly in 256 bytes (4 cache lines) for optimal performance.
//!
//! Memory layout:
//! - capacity: usize (8 bytes)
//! - len: usize (8 bytes) 
//! - child_type: ChildType (1 byte)
//! - _phantom: PhantomData (0 bytes)
//! - data: [u8; 239] (239 bytes)
//! Total: 256 bytes, aligned to 64-byte boundary

use std::marker::PhantomData;
use std::mem;
use crate::types::{NodeId, NodeRef, InsertResult, SplitNodeData};

/// Type of children stored in this branch node
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChildType {
    Leaves,
    Branches,
}

/// Compressed branch node that stores keys and child node IDs in a compact format.
/// 
/// The node uses a fixed-size byte array to store data, providing better cache
/// locality compared to Vec-based storage. All children are of the same type
/// (either all leaves or all branches) as determined by the B+ tree structure.
/// 
/// Keys and child IDs are stored in separate contiguous regions within the data array:
/// [key0, key1, ..., keyN, child_id0, child_id1, ..., child_idN+1]
/// 
/// # B+ Tree Invariants
/// 
/// A well-formed branch node must satisfy these invariants:
/// - Must have at least 1 key (and therefore 2 children)
/// - Number of children = number of keys + 1
/// - Keys are stored in sorted order
/// - A branch with 0 keys is malformed and should not exist in a proper B+ tree
/// 
/// The only time a branch might temporarily have 0 keys is:
/// - During construction (before first insertion)
/// - During merging operations (just before the node is eliminated)
/// - When it's the root and about to be replaced by its only child
/// 
/// Methods like `get_child()` will return `None` for malformed branches to prevent
/// access to potentially invalid data.
#[derive(Debug, Clone)]
#[repr(C, align(64))]
pub struct CompressedBranchNode<K, V> {
    /// Maximum number of keys this node can hold
    capacity: usize,
    /// Current number of keys
    len: usize,
    /// Type of all children (leaves or branches)
    child_type: ChildType,
    /// Phantom data to maintain type parameters (zero-sized)
    _phantom: PhantomData<(K, V)>,
    /// Raw storage for keys and child node IDs
    /// Aligned to 8 bytes to ensure proper alignment for both keys and NodeIds
    data: [u64; 27], // 27 * 8 = 216 bytes, total struct should be ~256 bytes
}

impl<K, V> CompressedBranchNode<K, V>
where
    K: Copy + Ord,
{
    /// Create a new empty compressed branch node.
    /// 
    /// # Arguments
    /// * `capacity` - Maximum number of keys
    /// * `child_type` - Type of children this node will contain
    /// 
    /// # Returns
    /// A new empty compressed branch node
    pub fn new(capacity: usize, child_type: ChildType) -> Self {
        Self {
            capacity,
            len: 0,
            child_type,
            _phantom: PhantomData,
            data: [0; 27],
        }
    }

    /// Create a new branch node that will contain leaf children.
    pub fn new_leaf_parent(capacity: usize) -> Self {
        Self::new(capacity, ChildType::Leaves)
    }

    /// Create a new branch node that will contain branch children.
    pub fn new_branch_parent(capacity: usize) -> Self {
        Self::new(capacity, ChildType::Branches)
    }

    /// Returns the number of keys in this branch.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the maximum capacity of this branch.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns true if this branch is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns true if this branch is at capacity.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len >= self.capacity
    }

    /// Returns true if this branch node is properly formed.
    /// A proper branch node must have at least 1 key (and therefore 2 children).
    /// The only exception is during construction or when it's about to be eliminated.
    #[inline]
    pub fn is_well_formed(&self) -> bool {
        self.len >= 1
    }

    /// Returns true if this branch node is malformed (has 0 keys).
    /// Such nodes should not exist in a proper B+ tree structure.
    #[inline]
    pub fn is_malformed(&self) -> bool {
        self.len == 0
    }

    /// Returns the type of children this branch contains.
    #[inline]
    pub fn child_type(&self) -> ChildType {
        self.child_type
    }

    /// Returns true if children are leaves.
    #[inline]
    pub fn children_are_leaves(&self) -> bool {
        self.child_type == ChildType::Leaves
    }

    /// Returns true if children are branches.
    #[inline]
    pub fn children_are_branches(&self) -> bool {
        self.child_type == ChildType::Branches
    }

    /// Calculate the maximum number of keys that can fit in the available space.
    pub fn calculate_max_capacity() -> usize {
        let key_size = mem::size_of::<K>();
        let child_size = mem::size_of::<NodeId>();
        let available_space = 216; // data array size: 27 * 8 bytes

        // We need space for N keys and N+1 children
        // So: N * key_size + (N+1) * child_size <= available_space
        // N * (key_size + child_size) + child_size <= available_space
        // N <= (available_space - child_size) / (key_size + child_size)
        let pair_size = key_size + child_size;
        if available_space <= child_size {
            0
        } else {
            (available_space - child_size) / pair_size
        }
    }

    /// Get a pointer to the keys region in the data array.
    #[inline]
    unsafe fn keys_ptr(&self) -> *const K {
        self.data.as_ptr() as *const K
    }

    /// Get a mutable pointer to the keys region in the data array.
    #[inline]
    unsafe fn keys_ptr_mut(&mut self) -> *mut K {
        self.data.as_mut_ptr() as *mut K
    }

    /// Get a pointer to the children region in the data array.
    #[inline]
    unsafe fn children_ptr(&self) -> *const NodeId {
        let keys_size_bytes = self.capacity * mem::size_of::<K>();
        // Align to NodeId boundary (4 bytes for u32)
        let aligned_offset_bytes = (keys_size_bytes + mem::align_of::<NodeId>() - 1) & !(mem::align_of::<NodeId>() - 1);
        (self.data.as_ptr() as *const u8).add(aligned_offset_bytes) as *const NodeId
    }

    /// Get a mutable pointer to the children region in the data array.
    #[inline]
    unsafe fn children_ptr_mut(&mut self) -> *mut NodeId {
        let keys_size_bytes = self.capacity * mem::size_of::<K>();
        // Align to NodeId boundary (4 bytes for u32)
        let aligned_offset_bytes = (keys_size_bytes + mem::align_of::<NodeId>() - 1) & !(mem::align_of::<NodeId>() - 1);
        (self.data.as_mut_ptr() as *mut u8).add(aligned_offset_bytes) as *mut NodeId
    }

    /// Get a reference to a key at the given index.
    #[inline]
    unsafe fn key_at(&self, index: usize) -> &K {
        &*self.keys_ptr().add(index)
    }

    /// Get a mutable reference to a key at the given index.
    #[inline]
    unsafe fn key_at_mut(&mut self, index: usize) -> &mut K {
        &mut *self.keys_ptr_mut().add(index)
    }

    /// Get a reference to a child ID at the given index.
    #[inline]
    unsafe fn child_id_at(&self, index: usize) -> &NodeId {
        &*self.children_ptr().add(index)
    }

    /// Get a mutable reference to a child ID at the given index.
    #[inline]
    unsafe fn child_id_at_mut(&mut self, index: usize) -> &mut NodeId {
        &mut *self.children_ptr_mut().add(index)
    }

    /// Get a key by index (efficient collection accessor method).
    /// 
    /// # Arguments
    /// * `index` - The index of the key to retrieve (0-based)
    /// 
    /// # Returns
    /// Some(&K) if index is valid, None otherwise
    /// 
    /// # Performance
    /// O(1) - Direct access to compressed storage without allocation
    pub fn keys_get(&self, index: usize) -> Option<&K> {
        if index < self.len {
            unsafe {
                Some(self.key_at(index))
            }
        } else {
            None
        }
    }

    /// Get the first key in the node.
    pub fn first_key(&self) -> Option<&K> {
        if self.len > 0 {
            unsafe {
                Some(self.key_at(0))
            }
        } else {
            None
        }
    }

    /// Get the last key in the node.
    pub fn last_key(&self) -> Option<&K> {
        if self.len > 0 {
            unsafe {
                Some(self.key_at(self.len - 1))
            }
        } else {
            None
        }
    }

    /// Get all keys as a Vec (BranchNode compatibility method).
    /// 
    /// **Performance Note**: This creates a new Vec by copying all keys from the compressed storage.
    /// For better performance, consider:
    /// - `keys_get(index)` for O(1) individual key access
    /// - `keys_iter()` for efficient iteration without allocation
    /// 
    /// **Time Complexity**: O(n) where n is the number of keys
    /// **Space Complexity**: O(n) - allocates a new Vec
    pub fn keys(&self) -> Vec<K> {
        let mut keys = Vec::with_capacity(self.len);
        for i in 0..self.len {
            unsafe {
                keys.push(*self.key_at(i));
            }
        }
        keys
    }

    /// Iterator over keys in the branch node.
    pub fn keys_iter(&self) -> KeysIter<K, V> {
        KeysIter {
            node: self,
            index: 0,
            _phantom: PhantomData,
        }
    }

    /// Find the index where a key should be inserted to maintain sorted order.
    /// Returns (index, found) where found indicates if the key already exists.
    pub fn find_key_index(&self, key: &K) -> (usize, bool) {
        for i in 0..self.len {
            unsafe {
                let current_key = self.key_at(i);
                match key.cmp(current_key) {
                    std::cmp::Ordering::Less => return (i, false),
                    std::cmp::Ordering::Equal => return (i, true),
                    std::cmp::Ordering::Greater => continue,
                }
            }
        }
        (self.len, false)
    }

    /// Find the child index for a given key using binary search.
    pub fn find_child_index(&self, key: &K) -> usize {
        for i in 0..self.len {
            unsafe {
                if key <= self.key_at(i) {
                    return i;
                }
            }
        }
        self.len // Last child
    }
}

/// Iterator over keys in a compressed branch node.
pub struct KeysIter<'a, K, V> {
    node: &'a CompressedBranchNode<K, V>,
    index: usize,
    _phantom: PhantomData<(&'a K, &'a V)>,
}

impl<'a, K, V> Iterator for KeysIter<'a, K, V>
where
    K: Copy + Ord,
{
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.node.len {
            let key = unsafe { self.node.key_at(self.index) };
            self.index += 1;
            Some(key)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.node.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, K, V> ExactSizeIterator for KeysIter<'a, K, V>
where
    K: Copy + Ord,
{
}

impl<K, V> CompressedBranchNode<K, V>
where
    K: Copy + Ord,
{
    /// Get a child by index.
    pub fn get_child(&self, index: usize) -> Option<NodeRef<K, V>> {
        // A branch with 0 keys is malformed - it should have at least 1 key (2 children)
        if self.len == 0 {
            return None;
        }
        
        if index <= self.len { // Note: children count is len + 1
            unsafe {
                let node_id = *self.child_id_at(index);
                Some(match self.child_type {
                    ChildType::Leaves => NodeRef::Leaf(node_id, PhantomData),
                    ChildType::Branches => NodeRef::Branch(node_id, PhantomData),
                })
            }
        } else {
            None
        }
    }

    /// Set a child at the given index.
    pub fn set_child(&mut self, index: usize, child: NodeRef<K, V>) {
        if index <= self.len {
            let node_id = match child {
                NodeRef::Leaf(id, _) => {
                    debug_assert_eq!(self.child_type, ChildType::Leaves);
                    id
                }
                NodeRef::Branch(id, _) => {
                    debug_assert_eq!(self.child_type, ChildType::Branches);
                    id
                }
            };
            unsafe {
                *self.child_id_at_mut(index) = node_id;
            }
        }
    }

    /// Get all children as a Vec (BranchNode compatibility method).
    pub fn children(&self) -> Vec<NodeRef<K, V>> {
        let mut children = Vec::with_capacity(self.len + 1);
        for i in 0..=self.len {
            if let Some(child) = self.get_child(i) {
                children.push(child);
            }
        }
        children
    }

    /// Get the first child.
    pub fn first_child(&self) -> Option<NodeRef<K, V>> {
        self.get_child(0)
    }

    /// Get the last child.
    pub fn last_child(&self) -> Option<NodeRef<K, V>> {
        self.get_child(self.len)
    }

    /// Find the appropriate child for a given key.
    pub fn find_child_for_key(&self, key: &K) -> Option<NodeRef<K, V>> {
        let index = self.find_child_index(key);
        self.get_child(index)
    }

    /// Iterator over children in the branch node.
    pub fn children_iter(&self) -> ChildrenIter<K, V> {
        ChildrenIter {
            node: self,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

/// Iterator over children in a compressed branch node.
pub struct ChildrenIter<'a, K, V> {
    node: &'a CompressedBranchNode<K, V>,
    index: usize,
    _phantom: PhantomData<(&'a K, &'a V)>,
}

impl<'a, K, V> Iterator for ChildrenIter<'a, K, V>
where
    K: Copy + Ord,
{
    type Item = NodeRef<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index <= self.node.len {
            let child = self.node.get_child(self.index);
            self.index += 1;
            child
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // For malformed branches (len=0), there are no accessible children
        if self.node.is_malformed() {
            return (0, Some(0));
        }
        
        let remaining = (self.node.len + 1).saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a, K, V> ExactSizeIterator for ChildrenIter<'a, K, V>
where
    K: Copy + Ord,
{
}

impl<K, V> CompressedBranchNode<K, V>
where
    K: Copy + Ord,
{
    /// Insert a key and child at the specified index.
    /// This method can temporarily create an overfull node (len > capacity) for splitting.
    fn insert_at_index(&mut self, index: usize, key: K, child: NodeRef<K, V>) {
        debug_assert!(index <= self.len);

        // Shift keys to the right
        unsafe {
            if index < self.len {
                std::ptr::copy(
                    self.key_at(index),
                    self.key_at_mut(index + 1),
                    self.len - index,
                );
            }
            *self.key_at_mut(index) = key;
        }

        // Shift children to the right (note: children count is len + 1)
        unsafe {
            if index <= self.len {
                std::ptr::copy(
                    self.child_id_at(index + 1),
                    self.child_id_at_mut(index + 2),
                    self.len - index,
                );
            }
        }

        // Increment length first so set_child condition works correctly
        self.len += 1;
        
        // Set the new child
        self.set_child(index + 1, child);
    }

    /// Insert a key-child pair and handle splitting if necessary.
    pub fn insert(&mut self, key: K, child: NodeRef<K, V>) -> InsertResult<K, V> {
        let (index, found) = self.find_key_index(&key);
        
        if found {
            // Key already exists, update the child
            self.set_child(index + 1, child);
            InsertResult::Updated(None)
        } else {
            // Check if split is needed BEFORE inserting
            if !self.is_full() {
                // Room to insert without splitting
                self.insert_at_index(index, key, child);
                // After insertion, the branch should be well-formed
                debug_assert!(self.is_well_formed(), "Branch should have at least 1 key after insertion");
                InsertResult::Updated(None)
            } else {
                // Node is full, need to split
                // Insert first, then split
                self.insert_at_index(index, key, child);

                // Now split the overfull node
                let new_right = self.split();

                // Determine the separator key (first key of right node)
                let separator_key = new_right.first_key().unwrap().clone();

                InsertResult::Split {
                    old_value: None,
                    new_node_data: SplitNodeData::CompressedBranch(new_right),
                    separator_key,
                }
            }
        }
    }

    /// Split this branch node into two nodes.
    /// Returns the new right node, leaving this node as the left node.
    pub fn split(&mut self) -> CompressedBranchNode<K, V> {
        let total_keys = self.len;
        // For more balanced splits, use (total_keys - 1) / 2 for the left side
        // This ensures the right side gets at least as many keys as the left
        let mid = (total_keys - 1) / 2;

        // Create new right node with same child type
        let mut new_right = CompressedBranchNode::new(self.capacity, self.child_type);

        // Calculate how many keys go to the right node
        let right_count = total_keys - mid - 1; // -1 because mid key moves up
        new_right.len = right_count;

        // Copy keys to the right node (skip the separator key at mid)
        unsafe {
            if right_count > 0 {
                std::ptr::copy_nonoverlapping(
                    self.key_at(mid + 1),
                    new_right.keys_ptr_mut(),
                    right_count,
                );
            }
        }

        // Copy children to the right node (mid+1 to end)
        let right_children_count = right_count + 1;
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.child_id_at(mid + 1),
                new_right.children_ptr_mut(),
                right_children_count,
            );
        }

        // Update this node's length (remove separator and right keys)
        self.len = mid;

        new_right
    }

    /// Check if this branch needs to be split.
    pub fn needs_split(&self) -> bool {
        self.len > self.capacity
    }

    /// Check if this branch is underfull.
    pub fn is_underfull(&self) -> bool {
        self.len < self.min_keys()
    }

    /// Check if this branch can donate a key-child pair.
    pub fn can_donate(&self) -> bool {
        self.len > self.min_keys()
    }

    /// Get the minimum number of keys this branch should have.
    pub fn min_keys(&self) -> usize {
        (self.capacity + 1) / 2
    }

    /// Borrow the last key-child pair from this branch.
    pub fn borrow_last(&mut self) -> Option<(K, NodeRef<K, V>)> {
        if self.is_empty() || !self.can_donate() {
            return None;
        }

        let key = unsafe { *self.key_at(self.len - 1) };
        let child = self.get_child(self.len).unwrap();
        
        self.len -= 1;
        Some((key, child))
    }

    /// Borrow the first key-child pair from this branch.
    pub fn borrow_first(&mut self) -> Option<(K, NodeRef<K, V>)> {
        if self.is_empty() || !self.can_donate() {
            return None;
        }

        let key = unsafe { *self.key_at(0) };
        let child = self.get_child(0).unwrap();

        // Shift keys left
        unsafe {
            if self.len > 1 {
                std::ptr::copy(
                    self.key_at(1),
                    self.key_at_mut(0),
                    self.len - 1,
                );
            }
        }

        // Shift children left
        unsafe {
            std::ptr::copy(
                self.child_id_at(1),
                self.child_id_at_mut(0),
                self.len, // len children remain after removing first
            );
        }

        self.len -= 1;
        Some((key, child))
    }

    /// Accept a key-child pair from the left sibling.
    pub fn accept_from_left(&mut self, key: K, child: NodeRef<K, V>) {
        // Shift keys right
        unsafe {
            if self.len > 0 {
                std::ptr::copy(
                    self.key_at(0),
                    self.key_at_mut(1),
                    self.len,
                );
            }
            *self.key_at_mut(0) = key;
        }

        // Shift children right
        unsafe {
            std::ptr::copy(
                self.child_id_at(0),
                self.child_id_at_mut(1),
                self.len + 1,
            );
        }

        // Set the new child at the beginning
        self.set_child(0, child);
        self.len += 1;
    }

    /// Accept a key-child pair from the right sibling.
    pub fn accept_from_right(&mut self, key: K, child: NodeRef<K, V>) {
        unsafe {
            *self.key_at_mut(self.len) = key;
        }
        self.set_child(self.len + 1, child);
        self.len += 1;
    }

    /// Merge another branch node into this one with a separator key.
    pub fn merge_from(&mut self, separator: K, other: &mut CompressedBranchNode<K, V>) {
        debug_assert_eq!(self.child_type, other.child_type);
        
        // Add separator key
        unsafe {
            *self.key_at_mut(self.len) = separator;
        }
        self.len += 1;

        // Copy keys from other
        unsafe {
            if other.len > 0 {
                std::ptr::copy_nonoverlapping(
                    other.key_at(0),
                    self.key_at_mut(self.len),
                    other.len,
                );
            }
        }

        // Copy children from other
        unsafe {
            std::ptr::copy_nonoverlapping(
                other.child_id_at(0),
                self.child_id_at_mut(self.len),
                other.len + 1,
            );
        }

        self.len += other.len;
        other.len = 0; // Clear the other node
    }

    /// Extract all content from this branch (used for merging).
    pub fn extract_all(&mut self) -> (Vec<K>, Vec<NodeRef<K, V>>) {
        let mut keys = Vec::with_capacity(self.len);
        let mut children = Vec::with_capacity(self.len + 1);

        // Copy all keys
        for i in 0..self.len {
            unsafe {
                keys.push(*self.key_at(i));
            }
        }

        // Copy all children
        for i in 0..=self.len {
            if let Some(child) = self.get_child(i) {
                children.push(child);
            }
        }

        self.len = 0;
        (keys, children)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_compressed_branch() {
        let leaf_parent = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        assert_eq!(leaf_parent.capacity(), 10);
        assert_eq!(leaf_parent.len(), 0);
        assert!(leaf_parent.is_empty());
        assert!(!leaf_parent.is_full());
        assert_eq!(leaf_parent.child_type(), ChildType::Leaves);
        assert!(leaf_parent.children_are_leaves());
        assert!(!leaf_parent.children_are_branches());

        let branch_parent = CompressedBranchNode::<i32, i32>::new_branch_parent(10);
        assert_eq!(branch_parent.child_type(), ChildType::Branches);
        assert!(!branch_parent.children_are_leaves());
        assert!(branch_parent.children_are_branches());
    }

    #[test]
    fn test_calculate_max_capacity_for_i32() {
        let max_cap = CompressedBranchNode::<i32, i32>::calculate_max_capacity();
        
        // i32 key = 4 bytes, NodeId = 4 bytes
        // N keys + (N+1) children: N*4 + (N+1)*4 = N*8 + 4
        // 239 bytes available: N*8 + 4 <= 239, so N <= 235/8 = 29
        assert_eq!(max_cap, 26);
    }

    #[test]
    fn test_calculate_max_capacity_for_different_types() {
        // u8 key = 1 byte, NodeId = 4 bytes
        let u8_cap = CompressedBranchNode::<u8, u8>::calculate_max_capacity();
        // N*1 + (N+1)*4 = N*5 + 4 <= 239, so N <= 235/5 = 47
        assert_eq!(u8_cap, 42);

        // u64 key = 8 bytes, NodeId = 4 bytes  
        let u64_cap = CompressedBranchNode::<u64, u64>::calculate_max_capacity();
        // N*8 + (N+1)*4 = N*12 + 4 <= 239, so N <= 235/12 = 19
        assert_eq!(u64_cap, 17);
    }

    #[test]
    fn test_compressed_branch_fits_four_cache_lines() {
        let branch = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        
        // Verify the struct spans exactly 256 bytes
        assert_eq!(std::mem::size_of_val(&branch), 256);
    }

    #[test]
    fn test_key_access_methods() {
        let mut branch = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        
        // Test empty branch
        assert_eq!(branch.keys_get(0), None);
        assert_eq!(branch.first_key(), None);
        assert_eq!(branch.last_key(), None);
        assert_eq!(branch.keys(), Vec::<i32>::new());
        
        // Manually add some keys for testing
        unsafe {
            *branch.key_at_mut(0) = 10;
            *branch.key_at_mut(1) = 20;
            *branch.key_at_mut(2) = 30;
            branch.len = 3;
        }
        
        // Test key access
        assert_eq!(branch.keys_get(0), Some(&10));
        assert_eq!(branch.keys_get(1), Some(&20));
        assert_eq!(branch.keys_get(2), Some(&30));
        assert_eq!(branch.keys_get(3), None);
        
        assert_eq!(branch.first_key(), Some(&10));
        assert_eq!(branch.last_key(), Some(&30));
        assert_eq!(branch.keys(), vec![10, 20, 30]);
    }

    #[test]
    fn test_child_access_methods() {
        let mut branch = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        
        // Test malformed empty branch (0 keys) - should return None for all child access
        // In a proper B+ tree, branches should have at least 1 key (2 children)
        assert_eq!(branch.get_child(0), None);
        assert_eq!(branch.first_child(), None);
        assert_eq!(branch.last_child(), None);
        assert_eq!(branch.get_child(1), None);

        // Manually add some children for testing
        let child1 = NodeRef::Leaf(1, PhantomData);
        let child2 = NodeRef::Leaf(2, PhantomData);
        let child3 = NodeRef::Leaf(3, PhantomData);
        
        branch.len = 2; // 2 keys means 3 children
        branch.set_child(0, child1);
        branch.set_child(1, child2);
        branch.set_child(2, child3);

        // Test child access
        assert_eq!(branch.get_child(0), Some(child1));
        assert_eq!(branch.get_child(1), Some(child2));
        assert_eq!(branch.get_child(2), Some(child3));
        assert_eq!(branch.get_child(3), None);
        
        assert_eq!(branch.first_child(), Some(child1));
        assert_eq!(branch.last_child(), Some(child3));
        
        let children = branch.children();
        assert_eq!(children.len(), 3);
        assert_eq!(children[0], child1);
        assert_eq!(children[1], child2);
        assert_eq!(children[2], child3);
    }

    #[test]
    fn test_find_key_index() {
        let mut branch = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        
        // Add keys: [10, 30, 50]
        unsafe {
            *branch.key_at_mut(0) = 10;
            *branch.key_at_mut(1) = 30;
            *branch.key_at_mut(2) = 50;
            branch.len = 3;
        }
        
        // Test finding existing keys
        assert_eq!(branch.find_key_index(&10), (0, true));
        assert_eq!(branch.find_key_index(&30), (1, true));
        assert_eq!(branch.find_key_index(&50), (2, true));
        
        // Test finding insertion points
        assert_eq!(branch.find_key_index(&5), (0, false));   // Before first
        assert_eq!(branch.find_key_index(&20), (1, false));  // Between 10 and 30
        assert_eq!(branch.find_key_index(&40), (2, false));  // Between 30 and 50
        assert_eq!(branch.find_key_index(&60), (3, false));  // After last
    }

    #[test]
    fn test_find_child_index() {
        let mut branch = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        
        // Add keys: [20, 40, 60]
        unsafe {
            *branch.key_at_mut(0) = 20;
            *branch.key_at_mut(1) = 40;
            *branch.key_at_mut(2) = 60;
            branch.len = 3;
        }
        
        // Test child index finding
        assert_eq!(branch.find_child_index(&10), 0);  // < 20, child 0
        assert_eq!(branch.find_child_index(&20), 0);  // = 20, child 0
        assert_eq!(branch.find_child_index(&30), 1);  // 20 < x < 40, child 1
        assert_eq!(branch.find_child_index(&40), 1);  // = 40, child 1
        assert_eq!(branch.find_child_index(&50), 2);  // 40 < x < 60, child 2
        assert_eq!(branch.find_child_index(&60), 2);  // = 60, child 2
        assert_eq!(branch.find_child_index(&70), 3);  // > 60, child 3
    }

    #[test]
    fn test_insert_single_key() {
        let mut branch = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        let child = NodeRef::Leaf(1, PhantomData);
        
        // Set initial child
        branch.set_child(0, NodeRef::Leaf(0, PhantomData));
        
        let result = branch.insert(10, child);
        
        match result {
            InsertResult::Updated(None) => {
                assert_eq!(branch.len(), 1);
                assert_eq!(branch.keys_get(0), Some(&10));
                assert_eq!(branch.get_child(1), Some(child));
            }
            _ => panic!("Expected Updated(None)"),
        }
    }

    #[test]
    fn test_insert_multiple_keys() {
        let mut branch = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        
        // Set initial child
        branch.set_child(0, NodeRef::Leaf(0, PhantomData));
        
        // Insert keys in order
        branch.insert(20, NodeRef::Leaf(1, PhantomData));
        branch.insert(10, NodeRef::Leaf(2, PhantomData));
        branch.insert(30, NodeRef::Leaf(3, PhantomData));

        assert_eq!(branch.len(), 3);
        assert_eq!(branch.keys(), vec![10, 20, 30]);
        
        // Check children are in correct positions
        // In B+ tree: child[i] contains keys between key[i-1] and key[i]
        assert_eq!(branch.get_child(0), Some(NodeRef::Leaf(0, PhantomData))); // Before 10 (original child)
        assert_eq!(branch.get_child(1), Some(NodeRef::Leaf(2, PhantomData))); // Between 10-20 (from key 10 insertion)
        assert_eq!(branch.get_child(2), Some(NodeRef::Leaf(1, PhantomData))); // Between 20-30 (from key 20 insertion)
        assert_eq!(branch.get_child(3), Some(NodeRef::Leaf(3, PhantomData))); // After 30 (from key 30 insertion)
    }

    #[test]
    fn test_split_functionality() {
        let mut branch = CompressedBranchNode::<i32, i32>::new_leaf_parent(4);
        
        // Fill the branch to capacity and add one more to trigger split
        branch.set_child(0, NodeRef::Leaf(0, PhantomData));
        for i in 1..=5 {
            branch.insert(i * 10, NodeRef::Leaf(i as u32, PhantomData));
        }
        
        // Should have triggered a split
        assert_eq!(branch.len(), 2); // Left node: keys [10, 20], separator 30 moves up
        
        // The split should have been handled by insert, but let's test split directly
        let mut full_branch = CompressedBranchNode::<i32, i32>::new_leaf_parent(4);
        full_branch.set_child(0, NodeRef::Leaf(0, PhantomData));
        
        // Manually fill it
        unsafe {
            for i in 0..4 {
                *full_branch.key_at_mut(i) = ((i + 1) * 10) as i32;
                full_branch.set_child(i + 1, NodeRef::Leaf((i + 1) as u32, PhantomData));
            }
            full_branch.len = 4;
        }
        
        let right = full_branch.split();
        
        assert_eq!(full_branch.len(), 1); // Left: [10], separator 20 moves up
        assert_eq!(right.len(), 2);       // Right: [30, 40]
        
        assert_eq!(full_branch.keys(), vec![10]);
        assert_eq!(right.keys(), vec![30, 40]);
    }

    #[test]
    fn test_borrowing_operations() {
        let mut donor = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        let mut receiver = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        
        // Set up donor with multiple keys (need more than min_keys = 5 for capacity 10)
        donor.set_child(0, NodeRef::Leaf(0, PhantomData));
        donor.insert(10, NodeRef::Leaf(1, PhantomData));
        donor.insert(20, NodeRef::Leaf(2, PhantomData));
        donor.insert(30, NodeRef::Leaf(3, PhantomData));
        donor.insert(40, NodeRef::Leaf(4, PhantomData));
        donor.insert(50, NodeRef::Leaf(5, PhantomData));
        donor.insert(60, NodeRef::Leaf(6, PhantomData));
        
        assert_eq!(donor.len(), 6);
        assert!(donor.can_donate());

        // Test borrow_last
        if let Some((key, child)) = donor.borrow_last() {
            assert_eq!(key, 60);
            assert_eq!(child, NodeRef::Leaf(6, PhantomData));
            assert_eq!(donor.len(), 5);

            receiver.accept_from_right(key, child);
            assert_eq!(receiver.len(), 1);
            assert_eq!(receiver.keys_get(0), Some(&60));
        }

        // Test borrow_first
        if let Some((key, child)) = donor.borrow_first() {
            assert_eq!(key, 10);
            assert_eq!(child, NodeRef::Leaf(0, PhantomData));
            assert_eq!(donor.len(), 4);

            receiver.accept_from_left(key, child);
            assert_eq!(receiver.len(), 2);
            assert_eq!(receiver.keys(), vec![10, 60]);
        }
    }

    #[test]
    fn test_merge_functionality() {
        let mut left = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        let mut right = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        
        // Set up left branch
        left.set_child(0, NodeRef::Leaf(0, PhantomData));
        left.insert(10, NodeRef::Leaf(1, PhantomData));
        left.insert(20, NodeRef::Leaf(2, PhantomData));
        
        // Set up right branch
        right.set_child(0, NodeRef::Leaf(3, PhantomData));
        right.insert(40, NodeRef::Leaf(4, PhantomData));
        right.insert(50, NodeRef::Leaf(5, PhantomData));
        
        assert_eq!(left.len(), 2);
        assert_eq!(right.len(), 2);
        
        // Merge with separator key 30
        left.merge_from(30, &mut right);
        
        assert_eq!(left.len(), 5); // 2 + 1 (separator) + 2
        assert_eq!(right.len(), 0); // Cleared
        assert_eq!(left.keys(), vec![10, 20, 30, 40, 50]);
        
        // Check that all children are preserved
        let children = left.children();
        assert_eq!(children.len(), 6); // 5 keys + 1 = 6 children
    }

    #[test]
    fn test_underfull_detection() {
        let mut branch = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        
        // Empty branch should be underfull
        assert!(branch.is_underfull());
        assert!(!branch.can_donate());
        
        // Add minimum keys
        let min_keys = branch.min_keys();
        branch.set_child(0, NodeRef::Leaf(0, PhantomData));
        for i in 0..min_keys {
            branch.insert(((i + 1) * 10) as i32, NodeRef::Leaf((i + 1) as u32, PhantomData));
        }
        
        assert!(!branch.is_underfull());
        assert!(!branch.can_donate()); // At minimum, can't donate
        
        // Add one more key
        branch.insert(((min_keys + 1) * 10) as i32, NodeRef::Leaf((min_keys + 1) as u32, PhantomData));
        assert!(branch.can_donate()); // Above minimum, can donate
    }

    #[test]
    fn test_keys_iterator() {
        let mut branch = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        
        // Test empty iterator
        let mut iter = branch.keys_iter();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        
        // Add some keys
        unsafe {
            *branch.key_at_mut(0) = 10;
            *branch.key_at_mut(1) = 20;
            *branch.key_at_mut(2) = 30;
            branch.len = 3;
        }
        
        let keys: Vec<&i32> = branch.keys_iter().collect();
        assert_eq!(keys, vec![&10, &20, &30]);
        
        // Test size hint
        let iter = branch.keys_iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_children_iterator() {
        let mut branch = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        
        // Test malformed empty branch (0 keys) - should have no accessible children
        // In a proper B+ tree, branches should have at least 1 key (2 children)
        branch.len = 0;
        
        let children: Vec<NodeRef<i32, i32>> = branch.children_iter().collect();
        assert_eq!(children.len(), 0); // Malformed branch has no accessible children

        // Add keys and children
        branch.set_child(1, NodeRef::Leaf(1, PhantomData));
        branch.set_child(2, NodeRef::Leaf(2, PhantomData));
        branch.len = 2; // 2 keys = 3 children
        
        let children: Vec<NodeRef<i32, i32>> = branch.children_iter().collect();
        assert_eq!(children.len(), 3);
        
        // Test size hint
        let iter = branch.children_iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_extract_all() {
        let mut branch = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        
        // Set up branch with keys and children
        branch.set_child(0, NodeRef::Leaf(0, PhantomData));
        branch.insert(10, NodeRef::Leaf(1, PhantomData));
        branch.insert(20, NodeRef::Leaf(2, PhantomData));
        
        assert_eq!(branch.len(), 2);
        
        let (keys, children) = branch.extract_all();
        
        assert_eq!(keys, vec![10, 20]);
        assert_eq!(children.len(), 3);
        assert_eq!(children[0], NodeRef::Leaf(0, PhantomData));
        assert_eq!(children[1], NodeRef::Leaf(1, PhantomData));
        assert_eq!(children[2], NodeRef::Leaf(2, PhantomData));
        
        // Branch should be empty after extraction
        assert_eq!(branch.len(), 0);
        assert!(branch.is_empty());
    }

    #[test]
    fn test_find_child_for_key() {
        let mut branch = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        
        // Set up branch: keys [20, 40], children [0, 1, 2]
        branch.set_child(0, NodeRef::Leaf(0, PhantomData));
        branch.insert(20, NodeRef::Leaf(1, PhantomData));
        branch.insert(40, NodeRef::Leaf(2, PhantomData));
        
        // Test finding children for various keys
        assert_eq!(branch.find_child_for_key(&10), Some(NodeRef::Leaf(0, PhantomData))); // < 20
        assert_eq!(branch.find_child_for_key(&20), Some(NodeRef::Leaf(0, PhantomData))); // = 20
        assert_eq!(branch.find_child_for_key(&30), Some(NodeRef::Leaf(1, PhantomData))); // 20 < x < 40
        assert_eq!(branch.find_child_for_key(&40), Some(NodeRef::Leaf(1, PhantomData))); // = 40
        assert_eq!(branch.find_child_for_key(&50), Some(NodeRef::Leaf(2, PhantomData))); // > 40
    }
}

impl<K, V> Default for CompressedBranchNode<K, V>
where
    K: Copy + Ord,
{
    fn default() -> Self {
        Self::new_leaf_parent(16) // Default capacity of 16, assume leaf parent initially
    }
}

impl<K, V> PartialEq for CompressedBranchNode<K, V>
where
    K: Copy + Ord + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len || self.capacity != other.capacity || self.child_type != other.child_type {
            return false;
        }
        
        // Compare all keys
        for i in 0..self.len {
            unsafe {
                if *self.key_at(i) != *other.key_at(i) {
                    return false;
                }
            }
        }
        
        // Compare all children
        for i in 0..=self.len {
            unsafe {
                if *self.child_id_at(i) != *other.child_id_at(i) {
                    return false;
                }
            }
        }
        
        true
    }
}

impl<K, V> Eq for CompressedBranchNode<K, V>
where
    K: Copy + Ord + Eq,
{
}

impl<K, V> From<crate::types::BranchNode<K, V>> for CompressedBranchNode<K, V>
where
    K: Copy + Ord,
{
    /// Convert a BranchNode to a CompressedBranchNode.
    /// 
    /// This conversion copies all keys and child references from the BranchNode's Vec storage
    /// into the CompressedBranchNode's compact array storage.
    fn from(branch: crate::types::BranchNode<K, V>) -> Self {
        // Determine child type from first child (if any)
        let child_type = if branch.children.is_empty() {
            ChildType::Leaves // Default assumption
        } else {
            match branch.children[0] {
                NodeRef::Leaf(_, _) => ChildType::Leaves,
                NodeRef::Branch(_, _) => ChildType::Branches,
            }
        };
        
        let capacity = branch.capacity.min(Self::calculate_max_capacity());
        let mut compressed = Self::new(capacity, child_type);
        
        // Set the length first so set_child works correctly
        compressed.len = branch.keys.len().min(compressed.capacity);
        
        // Copy all keys
        for (i, key) in branch.keys.iter().enumerate() {
            if i < compressed.capacity {
                unsafe {
                    *compressed.key_at_mut(i) = *key;
                }
            }
        }
        
        // Copy all children
        for (i, child) in branch.children.iter().enumerate() {
            if i <= compressed.len {  // Use len instead of capacity
                compressed.set_child(i, *child);
            }
        }
        
        compressed
    }
}

#[cfg(test)]
mod trait_tests {
    use super::*;

    #[test]
    fn test_default_trait() {
        let branch = CompressedBranchNode::<i32, i32>::default();
        assert_eq!(branch.capacity(), 16);
        assert_eq!(branch.len(), 0);
        assert!(branch.is_empty());
        assert_eq!(branch.child_type(), ChildType::Leaves);
    }

    #[test]
    fn test_partial_eq_trait() {
        let mut branch1 = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        let mut branch2 = CompressedBranchNode::<i32, i32>::new_leaf_parent(10);
        
        // Empty branches should be equal
        assert_eq!(branch1, branch2);
        
        // Add same data to both
        branch1.set_child(0, NodeRef::Leaf(0, PhantomData));
        branch1.insert(10, NodeRef::Leaf(1, PhantomData));
        branch1.insert(20, NodeRef::Leaf(2, PhantomData));
        
        branch2.set_child(0, NodeRef::Leaf(0, PhantomData));
        branch2.insert(10, NodeRef::Leaf(1, PhantomData));
        branch2.insert(20, NodeRef::Leaf(2, PhantomData));
        
        assert_eq!(branch1, branch2);
        
        // Different data should not be equal
        branch2.insert(30, NodeRef::Leaf(3, PhantomData));
        assert_ne!(branch1, branch2);
        
        // Different capacity should not be equal
        let branch3 = CompressedBranchNode::<i32, i32>::new_leaf_parent(20);
        assert_ne!(branch1, branch3);
        
        // Different child type should not be equal
        let branch4 = CompressedBranchNode::<i32, i32>::new_branch_parent(10);
        assert_ne!(branch1, branch4);
    }

    #[test]
    fn test_from_branch_node_conversion() {
        use crate::types::BranchNode;
        
        // Create a regular BranchNode
        let regular_branch = BranchNode {
            capacity: 10,
            keys: vec![10, 20, 30],
            children: vec![
                NodeRef::<i32, i32>::Leaf(0, PhantomData),
                NodeRef::Leaf(1, PhantomData),
                NodeRef::Leaf(2, PhantomData),
                NodeRef::Leaf(3, PhantomData),
            ],
        };
        
        // Convert to CompressedBranchNode
        let compressed_branch = CompressedBranchNode::from(regular_branch);
        
        // Verify the conversion
        assert_eq!(compressed_branch.len(), 3);
        assert_eq!(compressed_branch.capacity(), 10);
        assert_eq!(compressed_branch.child_type(), ChildType::Leaves);
        assert_eq!(compressed_branch.keys(), vec![10, 20, 30]);
        
        let children = compressed_branch.children();
        assert_eq!(children.len(), 4);
        assert_eq!(children[0], NodeRef::Leaf(0, PhantomData));
        assert_eq!(children[1], NodeRef::Leaf(1, PhantomData));
        assert_eq!(children[2], NodeRef::Leaf(2, PhantomData));
        assert_eq!(children[3], NodeRef::Leaf(3, PhantomData));
    }

    #[test]
    fn test_from_branch_node_with_branch_children() {
        use crate::types::BranchNode;
        
        // Create a BranchNode with branch children
        let regular_branch = BranchNode {
            capacity: 5,
            keys: vec![100, 200],
            children: vec![
                NodeRef::<i32, i32>::Branch(10, PhantomData),
                NodeRef::Branch(11, PhantomData),
                NodeRef::Branch(12, PhantomData),
            ],
        };
        
        // Convert to CompressedBranchNode
        let compressed_branch = CompressedBranchNode::from(regular_branch);
        
        // Verify the conversion
        assert_eq!(compressed_branch.len(), 2);
        assert_eq!(compressed_branch.child_type(), ChildType::Branches);
        assert!(compressed_branch.children_are_branches());
        assert_eq!(compressed_branch.keys(), vec![100, 200]);
        
        let children = compressed_branch.children();
        assert_eq!(children.len(), 3);
        assert_eq!(children[0], NodeRef::Branch(10, PhantomData));
        assert_eq!(children[1], NodeRef::Branch(11, PhantomData));
        assert_eq!(children[2], NodeRef::Branch(12, PhantomData));
    }
}
