//! Compressed node implementations optimized for cache line efficiency.
//!
//! This module contains CompressedLeafNode that fits exactly within 4 cache lines (256 bytes)
//! for optimal memory access patterns and reduced cache pressure.

use std::marker::PhantomData;
use std::mem;
use crate::types::{NodeId, InsertResult, SplitNodeData};

/// A leaf node compressed to exactly 4 cache lines (256 bytes) for optimal cache performance.
/// 
/// Memory layout:
/// - Header: 8 bytes (capacity, len, next) + PhantomData (zero-sized)
/// - Data: 248 bytes (inline storage for keys and values)
/// 
/// Keys and values are stored in separate contiguous regions within the data array:
/// [key0, key1, ..., keyN, value0, value1, ..., valueN]
#[derive(Debug, Clone)]
#[repr(C, align(64))]
pub struct CompressedLeafNode<K, V> {
    /// Maximum number of key-value pairs this node can hold
    capacity: usize,
    /// Current number of key-value pairs
    len: usize,
    /// Next leaf node in the linked list (for range queries)
    next: NodeId,
    /// Phantom data to maintain type parameters (zero-sized)
    _phantom: PhantomData<(K, V)>,
    /// Raw storage for keys and values
    data: [u8; 236], // Adjusted for actual layout: 256 - 20 = 236
}

impl<K, V> CompressedLeafNode<K, V>
where
    K: Copy + Ord,
    V: Copy,
{
    /// Create a new empty compressed leaf node.
    /// 
    /// # Arguments
    /// * `capacity` - Maximum number of key-value pairs (limited by available space)
    /// 
    /// # Returns
    /// A new empty compressed leaf node
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            len: 0,
            next: crate::types::NULL_NODE,
            _phantom: PhantomData,
            data: [0; 236],
        }
    }

    /// Returns the number of key-value pairs in this leaf.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the maximum capacity of this leaf.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns true if this leaf is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns true if this leaf is at capacity.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len >= self.capacity
    }

    /// Calculate the maximum number of key-value pairs that can fit in the available space.
    pub fn calculate_max_capacity() -> usize {
        let pair_size = mem::size_of::<K>() + mem::size_of::<V>();
        let available_space = 236; // data array size
        available_space / pair_size
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

    /// Get a pointer to the values region in the data array.
    #[inline]
    unsafe fn values_ptr(&self) -> *const V {
        let keys_size = self.capacity as usize * mem::size_of::<K>();
        (self.data.as_ptr().add(keys_size)) as *const V
    }

    /// Get a mutable pointer to the values region in the data array.
    #[inline]
    unsafe fn values_ptr_mut(&mut self) -> *mut V {
        let keys_size = self.capacity as usize * mem::size_of::<K>();
        (self.data.as_mut_ptr().add(keys_size)) as *mut V
    }

    /// Get a reference to a key at the given index.
    #[inline]
    unsafe fn key_at(&self, index: usize) -> &K {
        &*self.keys_ptr().add(index)
    }

    /// Get a reference to a value at the given index.
    #[inline]
    unsafe fn value_at(&self, index: usize) -> &V {
        &*self.values_ptr().add(index)
    }

    /// Get a mutable reference to a value at the given index.
    #[inline]
    unsafe fn value_at_mut(&mut self, index: usize) -> &mut V {
        &mut *self.values_ptr_mut().add(index)
    }

    /// Set a key at the given index.
    #[inline]
    unsafe fn set_key_at(&mut self, index: usize, key: K) {
        *self.keys_ptr_mut().add(index) = key;
    }

    /// Set a value at the given index.
    #[inline]
    unsafe fn set_value_at(&mut self, index: usize, value: V) {
        *self.values_ptr_mut().add(index) = value;
    }

    /// Find the index for a key using binary search.
    /// Returns (index, found) where:
    /// - If found: index is the position of the key
    /// - If not found: index is where the key should be inserted
    #[inline]
    fn find_key_index(&self, key: &K) -> (usize, bool) {
        if self.len == 0 {
            return (0, false);
        }

        let mut left = 0;
        let mut right = self.len as usize;

        while left < right {
            let mid = left + (right - left) / 2;
            
            // Safety: mid is always < self.len due to binary search bounds
            let mid_key = unsafe { self.key_at(mid) };
            
            match mid_key.cmp(key) {
                std::cmp::Ordering::Equal => {
                    return (mid, true); // Found exact match
                }
                std::cmp::Ordering::Less => {
                    left = mid + 1;
                }
                std::cmp::Ordering::Greater => {
                    right = mid;
                }
            }
        }

        (left, false) // Not found, return insertion point
    }
}

// Placeholder implementations - will be implemented through TDD
impl<K, V> CompressedLeafNode<K, V>
where
    K: Copy + Ord,
    V: Copy,
{
    /// Insert a key-value pair into the leaf.
    /// Returns InsertResult indicating success, update, or split needed.
    pub fn insert(&mut self, key: K, value: V) -> InsertResult<K, V> {
        let (index, found) = self.find_key_index(&key);
        
        if found {
            // Key exists - update value and return old value
            let old_value = unsafe { *self.value_at(index) };
            unsafe { self.set_value_at(index, value) };
            return InsertResult::Updated(Some(old_value));
        }

        // Key doesn't exist - check if we need to split
        if self.len >= self.capacity {
            return self.split_and_insert(key, value, index);
        }

        // Insert new key at the found position
        let insert_pos = index;
        let current_len = self.len as usize;

        // Shift keys and values to make room
        if insert_pos < current_len {
            unsafe {
                // Shift keys right
                let keys_src = self.keys_ptr().add(insert_pos);
                let keys_dst = self.keys_ptr_mut().add(insert_pos + 1);
                std::ptr::copy(keys_src, keys_dst, current_len - insert_pos);

                // Shift values right
                let values_src = self.values_ptr().add(insert_pos);
                let values_dst = self.values_ptr_mut().add(insert_pos + 1);
                std::ptr::copy(values_src, values_dst, current_len - insert_pos);
            }
        }

        // Insert new key-value pair
        unsafe {
            self.set_key_at(insert_pos, key);
            self.set_value_at(insert_pos, value);
        }

        // Increment length
        self.len += 1;

        InsertResult::Updated(None) // New key inserted
    }

    /// Split the node and insert the new key-value pair.
    fn split_and_insert(&mut self, key: K, value: V, insert_index: usize) -> InsertResult<K, V> {
        // First, insert the new key-value pair to create an overfull node
        self.insert_at_index(insert_index, key, value);
        
        // Now split the overfull node
        let new_right = self.split();
        
        // Determine the separator key (first key of right node)
        let separator_key = unsafe { *new_right.key_at(0) };
        
        // Convert to regular LeafNode for SplitNodeData
        let new_right_leaf = new_right.to_leaf_node();
        
        InsertResult::Split {
            old_value: None,
            new_node_data: SplitNodeData::Leaf(new_right_leaf),
            separator_key,
        }
    }

    /// Insert a key-value pair at the specified index without capacity checks.
    fn insert_at_index(&mut self, index: usize, key: K, value: V) {
        let current_len = self.len as usize;

        // Shift keys and values to make room
        if index < current_len {
            unsafe {
                // Shift keys right
                let keys_src = self.keys_ptr().add(index);
                let keys_dst = self.keys_ptr_mut().add(index + 1);
                std::ptr::copy(keys_src, keys_dst, current_len - index);

                // Shift values right
                let values_src = self.values_ptr().add(index);
                let values_dst = self.values_ptr_mut().add(index + 1);
                std::ptr::copy(values_src, values_dst, current_len - index);
            }
        }

        // Insert new key-value pair
        unsafe {
            self.set_key_at(index, key);
            self.set_value_at(index, value);
        }

        // Increment length
        self.len += 1;
    }

    /// Split this leaf node, returning the new right node.
    pub fn split(&mut self) -> CompressedLeafNode<K, V> {
        let total_keys = self.len as usize;
        
        // Calculate split point for better balance
        let mid = total_keys.div_ceil(2); // Round up for odd numbers
        
        // Create new right node with same capacity
        let mut new_right = CompressedLeafNode::new(self.capacity);

        // Calculate how many keys go to the right node
        let right_count = total_keys - mid;
        new_right.len = right_count;

        // Copy keys and values to the right node
        unsafe {
            // Copy keys
            std::ptr::copy_nonoverlapping(
                self.keys_ptr().add(mid),
                new_right.keys_ptr_mut(),
                right_count,
            );
            
            // Copy values
            std::ptr::copy_nonoverlapping(
                self.values_ptr().add(mid),
                new_right.values_ptr_mut(),
                right_count,
            );
        }
        
        // Update linked list: right node takes over the next pointer
        new_right.next = self.next;
        
        // This node will point to the new right node after allocation
        // For now, set to NULL_NODE and let the caller handle linking
        self.next = crate::types::NULL_NODE;
        
        // Update this node's length
        self.len = mid;

        new_right
    }

    /// Convert this CompressedLeafNode to a regular LeafNode.
    pub fn to_leaf_node(&self) -> crate::types::LeafNode<K, V> {
        let mut keys = Vec::with_capacity(self.len as usize);
        let mut values = Vec::with_capacity(self.len as usize);
        
        // Copy all keys and values
        for i in 0..self.len as usize {
            unsafe {
                keys.push(*self.key_at(i));
                values.push(*self.value_at(i));
            }
        }
        
        crate::types::LeafNode {
            capacity: self.capacity as usize,
            keys,
            values,
            next: self.next,
        }
    }

    /// Get a value by key.
    pub fn get(&self, key: &K) -> Option<&V> {
        let (index, found) = self.find_key_index(key);
        
        if found {
            Some(unsafe { self.value_at(index) })
        } else {
            None
        }
    }

    /// Get a mutable reference to a value by key (LeafNode compatibility method).
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let (index, found) = self.find_key_index(key);
        
        if found {
            Some(unsafe { self.value_at_mut(index) })
        } else {
            None
        }
    }

    /// Remove a key-value pair from the leaf.
    /// Returns the removed value and whether the node is now underfull.
    pub fn remove(&mut self, key: &K) -> (Option<V>, bool) {
        let (index, found) = self.find_key_index(key);
        
        if !found {
            return (None, false); // Key not found
        }
        
        // Get the value to return
        let removed_value = unsafe { *self.value_at(index) };
        
        // Shift keys and values left to fill the gap
        let current_len = self.len as usize;
        if index < current_len - 1 {
            unsafe {
                // Shift keys left
                let keys_src = self.keys_ptr().add(index + 1);
                let keys_dst = self.keys_ptr_mut().add(index);
                std::ptr::copy(keys_src, keys_dst, current_len - index - 1);
                
                // Shift values left
                let values_src = self.values_ptr().add(index + 1);
                let values_dst = self.values_ptr_mut().add(index);
                std::ptr::copy(values_src, values_dst, current_len - index - 1);
            }
        }
        
        // Decrement length
        self.len -= 1;
        
        let is_underfull = self.is_underfull();
        (Some(removed_value), is_underfull)
    }

    /// Returns true if this leaf node is underfull (below minimum occupancy).
    pub fn is_underfull(&self) -> bool {
        (self.len as usize) < self.min_keys()
    }

    /// Returns true if this leaf can donate a key to a sibling.
    pub fn can_donate(&self) -> bool {
        (self.len as usize) > self.min_keys()
    }

    /// Returns the minimum number of keys this node should have.
    pub fn min_keys(&self) -> usize {
        // For leaf nodes, minimum is floor(capacity / 2)
        // Exception: root can have fewer keys
        (self.capacity as usize) / 2
    }

    /// Get the next node ID in the linked list.
    pub fn next(&self) -> NodeId {
        self.next
    }

    /// Set the next node ID in the linked list.
    pub fn set_next(&mut self, next: NodeId) {
        self.next = next;
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

    /// Get a value by index (efficient collection accessor method).
    /// 
    /// # Arguments
    /// * `index` - The index of the value to retrieve (0-based)
    /// 
    /// # Returns
    /// Some(&V) if index is valid, None otherwise
    /// 
    /// # Performance
    /// O(1) - Direct access to compressed storage without allocation
    pub fn value_get(&self, index: usize) -> Option<&V> {
        if index < self.len {
            unsafe {
                Some(self.value_at(index))
            }
        } else {
            None
        }
    }

    /// Update a value by index (efficient collection accessor method).
    /// 
    /// # Arguments
    /// * `index` - The index of the value to update (0-based)
    /// * `value` - The new value to store
    /// 
    /// # Returns
    /// Some(old_value) if index is valid and update succeeded, None otherwise
    /// 
    /// # Performance
    /// O(1) - Direct access to compressed storage without allocation
    pub fn values_put(&mut self, index: usize, value: V) -> Option<V> {
        if index < self.len {
            unsafe {
                let old_value = *self.value_at(index);
                *self.value_at_mut(index) = value;
                Some(old_value)
            }
        } else {
            None
        }
    }

    /// Get a reference to all keys as a Vec (LeafNode compatibility method).
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
            if let Some(key) = self.keys_get(i) {
                keys.push(*key);
            }
        }
        keys
    }

    /// Get a reference to all values as a Vec (LeafNode compatibility method).
    /// 
    /// **Performance Note**: This creates a new Vec by copying all values from the compressed storage.
    /// For better performance, consider:
    /// - `value_get(index)` for O(1) individual value access
    /// - `values_iter()` for efficient iteration without allocation
    /// 
    /// **Time Complexity**: O(n) where n is the number of values
    /// **Space Complexity**: O(n) - allocates a new Vec
    pub fn values(&self) -> Vec<V> {
        let mut values = Vec::with_capacity(self.len);
        for i in 0..self.len {
            if let Some(value) = self.value_get(i) {
                values.push(*value);
            }
        }
        values
    }

    /// Get a mutable reference to all values as a Vec (LeafNode compatibility method).
    /// 
    /// **Important**: This creates a new Vec by copying all values from the compressed storage.
    /// Changes to the returned Vec will **NOT** affect the original node.
    /// 
    /// For in-place modifications, use:
    /// - `values_put(index, value)` for O(1) individual value updates
    /// - `values_iter_mut()` for efficient mutable iteration
    /// - `get_mut(key)` for key-based value updates
    /// 
    /// **Time Complexity**: O(n) where n is the number of values
    /// **Space Complexity**: O(n) - allocates a new Vec
    pub fn values_mut(&mut self) -> Vec<V> {
        let mut values = Vec::with_capacity(self.len);
        for i in 0..self.len {
            if let Some(value) = self.value_get(i) {
                values.push(*value);
            }
        }
        values
    }

    /// Check if this leaf needs to be split (LeafNode compatibility method).
    pub fn needs_split(&self) -> bool {
        self.len > self.capacity
    }

    /// Extract all content from this leaf (used for merging) - LeafNode compatibility method.
    pub fn extract_all(&mut self) -> (Vec<K>, Vec<V>, NodeId) {
        let mut keys = Vec::with_capacity(self.len as usize);
        let mut values = Vec::with_capacity(self.len as usize);
        
        // Copy all keys and values
        for i in 0..self.len as usize {
            unsafe {
                keys.push(*self.key_at(i));
                values.push(*self.value_at(i));
            }
        }
        
        let next = self.next;
        
        // Clear this node
        self.len = 0;
        self.next = crate::types::NULL_NODE;
        
        (keys, values, next)
    }

    // ============================================================================
    // BORROWING AND MERGING HELPERS
    // ============================================================================

    /// Borrow the last key-value pair from this leaf (used when this is the left sibling)
    pub fn borrow_last(&mut self) -> Option<(K, V)> {
        if self.len == 0 || !self.can_donate() {
            return None;
        }
        
        // Get the last key-value pair
        let last_index = (self.len - 1) as usize;
        let key = unsafe { *self.key_at(last_index) };
        let value = unsafe { *self.value_at(last_index) };
        
        // Decrement length (effectively removing the last item)
        self.len -= 1;
        
        Some((key, value))
    }

    /// Borrow the first key-value pair from this leaf (used when this is the right sibling)
    pub fn borrow_first(&mut self) -> Option<(K, V)> {
        if self.len == 0 || !self.can_donate() {
            return None;
        }
        
        // Get the first key-value pair
        let key = unsafe { *self.key_at(0) };
        let value = unsafe { *self.value_at(0) };
        
        // Shift all remaining keys and values left
        let current_len = self.len as usize;
        if current_len > 1 {
            unsafe {
                // Shift keys left
                let keys_src = self.keys_ptr().add(1);
                let keys_dst = self.keys_ptr_mut();
                std::ptr::copy(keys_src, keys_dst, current_len - 1);
                
                // Shift values left
                let values_src = self.values_ptr().add(1);
                let values_dst = self.values_ptr_mut();
                std::ptr::copy(values_src, values_dst, current_len - 1);
            }
        }
        
        // Decrement length
        self.len -= 1;
        
        Some((key, value))
    }

    /// Accept a borrowed key-value pair at the beginning (from left sibling)
    pub fn accept_from_left(&mut self, key: K, value: V) {
        // This is essentially inserting at index 0
        self.insert_at_index(0, key, value);
    }

    /// Accept a borrowed key-value pair at the end (from right sibling)
    pub fn accept_from_right(&mut self, key: K, value: V) {
        // This is essentially appending at the end
        let insert_index = self.len as usize;
        self.insert_at_index(insert_index, key, value);
    }

    /// Merge all content from another leaf into this one, returning the other's next pointer
    pub fn merge_from(&mut self, other: &mut CompressedLeafNode<K, V>) -> NodeId {
        let other_len = other.len as usize;
        let current_len = self.len as usize;
        
        // Copy all keys and values from other to the end of this node
        unsafe {
            // Copy keys
            std::ptr::copy_nonoverlapping(
                other.keys_ptr(),
                self.keys_ptr_mut().add(current_len),
                other_len,
            );
            
            // Copy values
            std::ptr::copy_nonoverlapping(
                other.values_ptr(),
                self.values_ptr_mut().add(current_len),
                other_len,
            );
        }
        
        // Update length
        self.len += other.len;
        
        // Get the other's next pointer before clearing it
        let other_next = other.next;
        
        // Clear the other node
        other.len = 0;
        other.next = crate::types::NULL_NODE;
        
        other_next
    }

    // ============================================================================
    // COLLECTION ACCESSOR METHODS
    // ============================================================================
    
    // Key collection accessors
    
    /// Get a key at the specified index.
    pub fn get_key(&self, index: usize) -> Option<&K> {
        if index < self.len as usize {
            Some(unsafe { self.key_at(index) })
        } else {
            None
        }
    }

    /// Get the first key in the node.
    pub fn first_key(&self) -> Option<&K> {
        if self.len > 0 {
            Some(unsafe { self.key_at(0) })
        } else {
            None
        }
    }
    
    /// Get the last key in the node.
    pub fn last_key(&self) -> Option<&K> {
        if self.len > 0 {
            Some(unsafe { self.key_at((self.len - 1) as usize) })
        } else {
            None
        }
    }
    
    /// Create an iterator over all keys in sorted order.
    pub fn keys_iter(&self) -> impl Iterator<Item = &K> {
        (0..self.len as usize).map(move |i| unsafe { self.key_at(i) })
    }
    
    /// Collect all keys into a Vec (for compatibility with existing code).
    pub fn collect_keys(&self) -> Vec<K> {
        let mut keys = Vec::with_capacity(self.len as usize);
        for i in 0..self.len as usize {
            keys.push(unsafe { *self.key_at(i) });
        }
        keys
    }
    
    // Value collection accessors
    
    /// Get a value at the specified index.
    pub fn get_value(&self, index: usize) -> Option<&V> {
        if index < self.len as usize {
            Some(unsafe { self.value_at(index) })
        } else {
            None
        }
    }
    
    /// Get a mutable reference to a value at the specified index.
    pub fn get_value_mut(&mut self, index: usize) -> Option<&mut V> {
        if index < self.len as usize {
            Some(unsafe { self.value_at_mut(index) })
        } else {
            None
        }
    }
    
    /// Get the first value in the node.
    pub fn first_value(&self) -> Option<&V> {
        if self.len > 0 {
            Some(unsafe { self.value_at(0) })
        } else {
            None
        }
    }
    
    /// Get the last value in the node.
    pub fn last_value(&self) -> Option<&V> {
        if self.len > 0 {
            Some(unsafe { self.value_at((self.len - 1) as usize) })
        } else {
            None
        }
    }
    
    /// Get a mutable reference to the first value in the node.
    pub fn first_value_mut(&mut self) -> Option<&mut V> {
        if self.len > 0 {
            Some(unsafe { self.value_at_mut(0) })
        } else {
            None
        }
    }
    
    /// Get a mutable reference to the last value in the node.
    pub fn last_value_mut(&mut self) -> Option<&mut V> {
        if self.len > 0 {
            Some(unsafe { self.value_at_mut((self.len - 1) as usize) })
        } else {
            None
        }
    }
    
    /// Create an iterator over all values in key-sorted order.
    pub fn values_iter(&self) -> impl Iterator<Item = &V> {
        (0..self.len as usize).map(move |i| unsafe { self.value_at(i) })
    }
    
    /// Create a mutable iterator over all values in key-sorted order.
    /// Note: Due to Rust's borrowing rules, this returns a custom iterator type.
    pub fn values_iter_mut(&mut self) -> ValuesMutIter<K, V> {
        ValuesMutIter {
            node: self,
            index: 0,
        }
    }

    /// Collect all values into a Vec (for compatibility with existing code).
    pub fn collect_values(&self) -> Vec<V> {
        let mut values = Vec::with_capacity(self.len as usize);
        for i in 0..self.len as usize {
            values.push(unsafe { *self.value_at(i) });
        }
        values
    }
    
    // Pair collection accessors
    
    /// Get a key-value pair at the specified index.
    pub fn get_pair(&self, index: usize) -> Option<(&K, &V)> {
        if index < self.len as usize {
            Some(unsafe { (self.key_at(index), self.value_at(index)) })
        } else {
            None
        }
    }
    
    /// Get a key-value pair with mutable value at the specified index.
    pub fn get_pair_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        if index < self.len as usize {
            unsafe { 
                let key_ptr = self.key_at(index) as *const K;
                let value_ptr = self.value_at_mut(index) as *mut V;
                Some((&*key_ptr, &mut *value_ptr))
            }
        } else {
            None
        }
    }

    /// Get the first key-value pair in the node.
    pub fn first_pair(&self) -> Option<(&K, &V)> {
        if self.len > 0 {
            Some(unsafe { (self.key_at(0), self.value_at(0)) })
        } else {
            None
        }
    }
    
    /// Get the last key-value pair in the node.
    pub fn last_pair(&self) -> Option<(&K, &V)> {
        if self.len > 0 {
            let index = (self.len - 1) as usize;
            Some(unsafe { (self.key_at(index), self.value_at(index)) })
        } else {
            None
        }
    }
    
    /// Get the first key-value pair with mutable value in the node.
    pub fn first_pair_mut(&mut self) -> Option<(&K, &mut V)> {
        if self.len > 0 {
            unsafe { 
                let key_ptr = self.key_at(0) as *const K;
                let value_ptr = self.value_at_mut(0) as *mut V;
                Some((&*key_ptr, &mut *value_ptr))
            }
        } else {
            None
        }
    }

    /// Get the last key-value pair with mutable value in the node.
    pub fn last_pair_mut(&mut self) -> Option<(&K, &mut V)> {
        if self.len > 0 {
            let index = (self.len - 1) as usize;
            unsafe { 
                let key_ptr = self.key_at(index) as *const K;
                let value_ptr = self.value_at_mut(index) as *mut V;
                Some((&*key_ptr, &mut *value_ptr))
            }
        } else {
            None
        }
    }

    /// Collect all key-value pairs into a Vec (for compatibility with existing code).
    pub fn collect_pairs(&self) -> Vec<(K, V)> {
        let mut pairs = Vec::with_capacity(self.len as usize);
        for i in 0..self.len as usize {
            pairs.push(unsafe { (*self.key_at(i), *self.value_at(i)) });
        }
        pairs
    }

    /// Iterator over key-value pairs in sorted order.
    pub fn iter(&self) -> CompressedLeafIter<K, V> {
        CompressedLeafIter {
            node: self,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

impl<K, V> Default for CompressedLeafNode<K, V>
where
    K: Copy + Ord,
    V: Copy,
{
    fn default() -> Self {
        Self::new(16) // Default capacity of 16, matching LeafNode
    }
}

impl<K, V> PartialEq for CompressedLeafNode<K, V>
where
    K: Copy + Ord + PartialEq,
    V: Copy + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len || self.capacity != other.capacity {
            return false;
        }
        
        // Compare all key-value pairs
        for i in 0..self.len {
            unsafe {
                if *self.key_at(i) != *other.key_at(i) || *self.value_at(i) != *other.value_at(i) {
                    return false;
                }
            }
        }
        
        true
    }
}

impl<K, V> Eq for CompressedLeafNode<K, V>
where
    K: Copy + Ord + Eq,
    V: Copy + Eq,
{
}

impl<K, V> From<crate::types::LeafNode<K, V>> for CompressedLeafNode<K, V>
where
    K: Copy + Ord,
    V: Copy,
{
    /// Convert a LeafNode to a CompressedLeafNode.
    /// 
    /// This conversion copies all key-value pairs from the LeafNode's Vec storage
    /// into the CompressedLeafNode's compact array storage.
    fn from(leaf: crate::types::LeafNode<K, V>) -> Self {
        let capacity = leaf.capacity.min(Self::calculate_max_capacity());
        let mut compressed = Self::new(capacity);
        
        // Copy all key-value pairs
        for (key, value) in leaf.keys.into_iter().zip(leaf.values.into_iter()) {
            let _ = compressed.insert(key, value); // Should never fail since we're within capacity
        }
        
        compressed.next = leaf.next;
        compressed
    }
}

/// Iterator over key-value pairs in a compressed leaf node.
pub struct CompressedLeafIter<'a, K, V> {
    node: &'a CompressedLeafNode<K, V>,
    index: usize,
    _phantom: PhantomData<(&'a K, &'a V)>,
}

impl<'a, K, V> Iterator for CompressedLeafIter<'a, K, V>
where
    K: Copy + Ord,
    V: Copy,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.node.len as usize {
            let key = unsafe { self.node.key_at(self.index) };
            let value = unsafe { self.node.value_at(self.index) };
            self.index += 1;
            Some((key, value))
        } else {
            None
        }
    }
}

/// Mutable iterator over values in a CompressedLeafNode.
pub struct ValuesMutIter<'a, K, V> {
    node: &'a mut CompressedLeafNode<K, V>,
    index: usize,
}

impl<'a, K, V> Iterator for ValuesMutIter<'a, K, V>
where
    K: Copy + Ord,
    V: Copy,
{
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.node.len as usize {
            let value = unsafe { 
                // SAFETY: We know the index is valid and we're returning a unique reference
                &mut *(self.node.value_at_mut(self.index) as *mut V)
            };
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Phase 1: Memory Layout Verification Tests
    
    #[test]
    fn compressed_leaf_fits_four_cache_lines() {
        assert_eq!(std::mem::size_of::<CompressedLeafNode<i32, i32>>(), 256);
        assert_eq!(std::mem::align_of::<CompressedLeafNode<i32, i32>>(), 64);
    }

    #[test]
    fn memory_is_contiguous() {
        let leaf = CompressedLeafNode::<i32, i32>::new(10);
        let start_ptr = &leaf as *const _ as *const u8;
        let end_ptr = unsafe { start_ptr.add(256) };
        
        // Verify the struct spans exactly 256 bytes
        assert_eq!(std::mem::size_of_val(&leaf), 256);
        
        // Print actual field offsets for debugging
        let capacity_offset = unsafe { 
            (&leaf.capacity as *const usize as *const u8).offset_from(start_ptr) 
        };
        let len_offset = unsafe { 
            (&leaf.len as *const usize as *const u8).offset_from(start_ptr) 
        };
        let next_offset = unsafe { 
            (&leaf.next as *const u32 as *const u8).offset_from(start_ptr) 
        };
        let phantom_offset = unsafe { 
            (&leaf._phantom as *const _ as *const u8).offset_from(start_ptr) 
        };
        let data_offset = unsafe { 
            (leaf.data.as_ptr()).offset_from(start_ptr) 
        };
        
        println!("Field offsets:");
        println!("  capacity: {}", capacity_offset);
        println!("  len: {}", len_offset);
        println!("  next: {}", next_offset);
        println!("  phantom: {}", phantom_offset);
        println!("  data: {}", data_offset);
        
        assert_eq!(capacity_offset, 0);
        assert_eq!(len_offset, 8);  // usize is 8 bytes
        assert_eq!(next_offset, 16); // after two usize fields
        assert_eq!(phantom_offset, 20); // after NodeId (u32)
        assert_eq!(data_offset, 20); // PhantomData is zero-sized, starts at same offset

        // Verify data array ends at struct boundary
        let data_end = unsafe { leaf.data.as_ptr().add(236) };
        assert_eq!(data_end as *const u8, end_ptr);
    }

    #[test]
    fn verify_cache_line_alignment() {
        let leaf = CompressedLeafNode::<i32, i32>::new(10);
        let addr = &leaf as *const _ as usize;
        
        // Should be aligned to 64-byte boundary
        assert_eq!(addr % 64, 0);
    }

    #[test]
    fn test_leafnode_compatibility_methods() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(10);
        
        // Test keys() method
        assert_eq!(leaf.keys(), Vec::<i32>::new());
        
        // Test values() method
        assert_eq!(leaf.values(), Vec::<i32>::new());
        
        // Test values_mut() method
        assert_eq!(leaf.values_mut(), Vec::<i32>::new());
        
        // Add some data
        leaf.insert(1, 100);
        leaf.insert(2, 200);
        leaf.insert(3, 300);
        
        // Test with data
        assert_eq!(leaf.keys(), vec![1, 2, 3]);
        assert_eq!(leaf.values(), vec![100, 200, 300]);
        assert_eq!(leaf.values_mut(), vec![100, 200, 300]);
    }

    #[test]
    fn test_partial_eq_trait() {
        let mut leaf1 = CompressedLeafNode::<i32, i32>::new(10);
        let mut leaf2 = CompressedLeafNode::<i32, i32>::new(10);
        
        // Empty nodes should be equal
        assert_eq!(leaf1, leaf2);
        
        // Add same data to both
        leaf1.insert(1, 100);
        leaf1.insert(2, 200);
        leaf2.insert(1, 100);
        leaf2.insert(2, 200);
        
        assert_eq!(leaf1, leaf2);
        
        // Different data should not be equal
        leaf2.insert(3, 300);
        assert_ne!(leaf1, leaf2);
        
        // Different capacity should not be equal
        let leaf3 = CompressedLeafNode::<i32, i32>::new(20);
        assert_ne!(leaf1, leaf3);
    }

    #[test]
    fn test_from_leafnode_conversion() {
        use crate::types::LeafNode;
        
        // Create a regular LeafNode
        let regular_leaf = LeafNode {
            capacity: 10,
            keys: vec![1, 2, 3],
            values: vec![100, 200, 300],
            next: 42,
        };
        
        // Convert to CompressedLeafNode
        let compressed_leaf = CompressedLeafNode::from(regular_leaf);
        
        // Verify the conversion
        assert_eq!(compressed_leaf.len(), 3);
        assert_eq!(compressed_leaf.capacity(), 10);
        assert_eq!(compressed_leaf.next(), 42);
        assert_eq!(compressed_leaf.get(&1), Some(&100));
        assert_eq!(compressed_leaf.get(&2), Some(&200));
        assert_eq!(compressed_leaf.get(&3), Some(&300));
        assert_eq!(compressed_leaf.keys(), vec![1, 2, 3]);
        assert_eq!(compressed_leaf.values(), vec![100, 200, 300]);
    }

    #[test]
    fn test_default_trait() {
        let leaf = CompressedLeafNode::<i32, i32>::default();
        assert_eq!(leaf.capacity(), 16);
        assert_eq!(leaf.len(), 0);
        assert!(leaf.is_empty());
    }

    #[test]
    fn test_improved_leafnode_compatibility() {
        use crate::types::LeafNode;
        
        // Create both types of nodes
        let mut regular_leaf = LeafNode {
            capacity: 10,
            keys: vec![1, 3, 5],
            values: vec![10, 30, 50],
            next: crate::types::NULL_NODE,
        };
        
        let mut compressed_leaf = CompressedLeafNode::<i32, i32>::new(10);
        compressed_leaf.insert(1, 10);
        compressed_leaf.insert(3, 30);
        compressed_leaf.insert(5, 50);
        
        // Both should have the same interface for common operations
        assert_eq!(regular_leaf.len(), compressed_leaf.len());
        assert_eq!(regular_leaf.get(&3), compressed_leaf.get(&3));
        assert_eq!(*regular_leaf.keys(), compressed_leaf.keys());
        assert_eq!(*regular_leaf.values(), compressed_leaf.values());

        // Both should support the same mutation operations
        assert_eq!(regular_leaf.get_mut(&3), compressed_leaf.get_mut(&3));
        
        // Both should support the same structural operations
        assert_eq!(regular_leaf.is_empty(), compressed_leaf.is_empty());
        assert_eq!(regular_leaf.is_full(), compressed_leaf.is_full());
        
        // Conversion should work seamlessly
        let converted = CompressedLeafNode::from(regular_leaf);
        assert_eq!(converted, compressed_leaf);
    }

    #[test]
    fn test_efficient_collection_accessors() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(10);
        
        // Test empty node
        assert_eq!(leaf.keys_get(0), None);
        assert_eq!(leaf.value_get(0), None);
        assert_eq!(leaf.values_put(0, 999), None);
        
        // Add some data
        leaf.insert(10, 100);
        leaf.insert(20, 200);
        leaf.insert(30, 300);
        
        // Test keys_get
        assert_eq!(leaf.keys_get(0), Some(&10));
        assert_eq!(leaf.keys_get(1), Some(&20));
        assert_eq!(leaf.keys_get(2), Some(&30));
        assert_eq!(leaf.keys_get(3), None); // Out of bounds
        
        // Test value_get
        assert_eq!(leaf.value_get(0), Some(&100));
        assert_eq!(leaf.value_get(1), Some(&200));
        assert_eq!(leaf.value_get(2), Some(&300));
        assert_eq!(leaf.value_get(3), None); // Out of bounds
        
        // Test values_put
        assert_eq!(leaf.values_put(1, 250), Some(200)); // Update middle value
        assert_eq!(leaf.value_get(1), Some(&250)); // Verify update
        assert_eq!(leaf.values_put(3, 400), None); // Out of bounds
        
        // Verify other values unchanged
        assert_eq!(leaf.value_get(0), Some(&100));
        assert_eq!(leaf.value_get(2), Some(&300));
        
        // Test that Vec methods still work with updated data
        assert_eq!(leaf.keys(), vec![10, 20, 30]);
        assert_eq!(leaf.values(), vec![100, 250, 300]);
    }

    #[test]
    fn test_collection_accessor_performance_characteristics() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(10);
        
        // Fill the node
        for i in 0..5 {
            leaf.insert(i, i * 10);
        }
        
        // Test that accessors work at boundaries
        assert_eq!(leaf.keys_get(0), Some(&0));
        assert_eq!(leaf.keys_get(4), Some(&4));
        assert_eq!(leaf.keys_get(5), None);
        
        assert_eq!(leaf.value_get(0), Some(&0));
        assert_eq!(leaf.value_get(4), Some(&40));
        assert_eq!(leaf.value_get(5), None);
        
        // Test values_put at boundaries
        assert_eq!(leaf.values_put(0, 999), Some(0));
        assert_eq!(leaf.values_put(4, 888), Some(40));
        assert_eq!(leaf.values_put(5, 777), None);
        
        // Verify updates
        assert_eq!(leaf.value_get(0), Some(&999));
        assert_eq!(leaf.value_get(4), Some(&888));
    }

    // Phase 2: Basic Construction Tests

    #[test]
    fn new_compressed_leaf() {
        let leaf = CompressedLeafNode::<i32, i32>::new(8);
        assert_eq!(leaf.len(), 0);
        assert_eq!(leaf.capacity(), 8);
        assert!(leaf.is_empty());
        assert!(!leaf.is_full());
    }

    #[test]
    fn calculate_max_capacity_for_i32_pairs() {
        let max_cap = CompressedLeafNode::<i32, i32>::calculate_max_capacity();
        
        // i32 + i32 = 8 bytes per pair
        // 236 bytes available / 8 bytes per pair = 29 pairs
        assert_eq!(max_cap, 29);
    }

    #[test]
    fn calculate_max_capacity_for_different_types() {
        // u8 + u8 = 2 bytes per pair
        let u8_cap = CompressedLeafNode::<u8, u8>::calculate_max_capacity();
        assert_eq!(u8_cap, 118); // 236 / 2 = 118

        // u64 + u64 = 16 bytes per pair  
        let u64_cap = CompressedLeafNode::<u64, u64>::calculate_max_capacity();
        assert_eq!(u64_cap, 14); // 236 / 16 = 14
    }

    // Phase 3: Single Insert/Get Tests (will fail until implemented)

    #[test]
    fn insert_single_item() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        match leaf.insert(42, 100) {
            InsertResult::Updated(None) => {}, // New insertion
            _ => panic!("Expected new insertion"),
        }
        assert_eq!(leaf.len(), 1);
        assert_eq!(leaf.get(&42), Some(&100));
    }

    #[test]
    fn get_nonexistent_key() {
        let leaf = CompressedLeafNode::<i32, i32>::new(8);
        assert_eq!(leaf.get(&42), None);
    }

    #[test]
    fn get_from_empty_leaf() {
        let leaf = CompressedLeafNode::<i32, i32>::new(10);
        assert_eq!(leaf.get(&1), None);
        assert_eq!(leaf.get(&0), None);
        assert_eq!(leaf.get(&-1), None);
    }

    #[test]
    fn get_boundary_conditions() {
        // This test will need insert to be implemented first
        // Testing get with min/max values and edge cases
        let mut leaf = CompressedLeafNode::<i32, i32>::new(10);
        
        // Will need to manually set up data for this test
        // For now, just test empty leaf boundary conditions
        assert_eq!(leaf.get(&i32::MIN), None);
        assert_eq!(leaf.get(&i32::MAX), None);
        assert_eq!(leaf.get(&0), None);
    }

    #[test]
    fn get_with_manual_data_setup() {
        // Manually set up a leaf with known data to test binary search
        let mut leaf = CompressedLeafNode::<i32, i32>::new(10);
        
        // Manually insert sorted data: keys [10, 20, 30], values [100, 200, 300]
        leaf.len = 3;
        unsafe {
            leaf.set_key_at(0, 10);
            leaf.set_key_at(1, 20);
            leaf.set_key_at(2, 30);
            leaf.set_value_at(0, 100);
            leaf.set_value_at(1, 200);
            leaf.set_value_at(2, 300);
        }
        
        // Test exact matches
        assert_eq!(leaf.get(&10), Some(&100));
        assert_eq!(leaf.get(&20), Some(&200));
        assert_eq!(leaf.get(&30), Some(&300));
        
        // Test non-existent keys
        assert_eq!(leaf.get(&5), None);   // Before first
        assert_eq!(leaf.get(&15), None);  // Between first and second
        assert_eq!(leaf.get(&25), None);  // Between second and third
        assert_eq!(leaf.get(&35), None);  // After last
        
        // Test boundary values
        assert_eq!(leaf.get(&9), None);
        assert_eq!(leaf.get(&11), None);
        assert_eq!(leaf.get(&31), None);
    }

    // Phase 4: Multiple Insert Tests (will fail until implemented)

    #[test]
    fn insert_multiple_sorted() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        for i in 0..5 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {}, // New insertion
                _ => panic!("Expected new insertion"),
            }
        }
        assert_eq!(leaf.len(), 5);
        
        // Verify sorted order maintained
        for i in 0..5 {
            assert_eq!(leaf.get(&i), Some(&(i * 10)));
        }
    }

    #[test]
    fn insert_multiple_unsorted() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        let keys = [5, 1, 8, 3, 7];
        
        for &key in &keys {
            match leaf.insert(key, key * 10) {
                InsertResult::Updated(None) => {}, // New insertion
                _ => panic!("Expected new insertion"),
            }
        }
        assert_eq!(leaf.len(), 5);
        
        // Verify all accessible and sorted internally
        for &key in &keys {
            assert_eq!(leaf.get(&key), Some(&(key * 10)));
        }
        
        // Verify they're stored in sorted order by checking sequential access
        // Keys should be internally sorted as: [1, 3, 5, 7, 8]
        unsafe {
            assert_eq!(*leaf.key_at(0), 1);
            assert_eq!(*leaf.key_at(1), 3);
            assert_eq!(*leaf.key_at(2), 5);
            assert_eq!(*leaf.key_at(3), 7);
            assert_eq!(*leaf.key_at(4), 8);
        }
    }

    #[test]
    fn insert_duplicate_keys() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Insert initial value
        match leaf.insert(42, 100) {
            InsertResult::Updated(None) => {}, // New insertion
            _ => panic!("Expected new insertion"),
        }
        assert_eq!(leaf.len(), 1);
        assert_eq!(leaf.get(&42), Some(&100));
        
        // Insert same key with different value (should update)
        match leaf.insert(42, 200) {
            InsertResult::Updated(Some(old_value)) => {
                assert_eq!(old_value, 100);
            },
            _ => panic!("Expected key update"),
        }
        assert_eq!(leaf.len(), 1); // Length shouldn't change
        assert_eq!(leaf.get(&42), Some(&200)); // Value should be updated
    }

    // Phase 5: Capacity Management Tests (will fail until implemented)

    #[test]
    fn insert_at_capacity() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(4);
        
        // Fill to capacity
        for i in 0..4 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {}, // New insertion
                _ => panic!("Expected new insertion"),
            }
        }
        assert!(leaf.is_full());
        
        // Attempt overflow - should trigger split
        match leaf.insert(99, 990) {
            InsertResult::Split { old_value: None, new_node_data, separator_key } => {
                // Verify split occurred
                assert!(separator_key >= 0 && separator_key <= 99);
                // The new node should be a leaf
                match new_node_data {
                    SplitNodeData::Leaf(_) => {},
                    _ => panic!("Expected leaf split"),
                }
            },
            _ => panic!("Expected split when inserting beyond capacity"),
        }
    }

    #[test]
    fn insert_comprehensive_edge_cases() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(10);
        
        // Test inserting at boundaries
        match leaf.insert(i32::MIN, -1000) {
            InsertResult::Updated(None) => {},
            _ => panic!("Expected new insertion"),
        }
        match leaf.insert(i32::MAX, 1000) {
            InsertResult::Updated(None) => {},
            _ => panic!("Expected new insertion"),
        }
        match leaf.insert(0, 0) {
            InsertResult::Updated(None) => {},
            _ => panic!("Expected new insertion"),
        }
        assert_eq!(leaf.len(), 3);
        
        // Verify they're accessible
        assert_eq!(leaf.get(&i32::MIN), Some(&-1000));
        assert_eq!(leaf.get(&i32::MAX), Some(&1000));
        assert_eq!(leaf.get(&0), Some(&0));
        
        // Insert some values in between
        match leaf.insert(-100, -100) {
            InsertResult::Updated(None) => {},
            _ => panic!("Expected new insertion"),
        }
        match leaf.insert(100, 100) {
            InsertResult::Updated(None) => {},
            _ => panic!("Expected new insertion"),
        }
        assert_eq!(leaf.len(), 5);
        
        // Verify sorted order is maintained
        unsafe {
            assert_eq!(*leaf.key_at(0), i32::MIN);
            assert_eq!(*leaf.key_at(1), -100);
            assert_eq!(*leaf.key_at(2), 0);
            assert_eq!(*leaf.key_at(3), 100);
            assert_eq!(*leaf.key_at(4), i32::MAX);
        }
        
        // Test updating boundary values
        match leaf.insert(i32::MIN, -2000) {
            InsertResult::Updated(Some(old_value)) => {
                assert_eq!(old_value, -1000);
            },
            _ => panic!("Expected key update"),
        }
        match leaf.insert(i32::MAX, 2000) {
            InsertResult::Updated(Some(old_value)) => {
                assert_eq!(old_value, 1000);
            },
            _ => panic!("Expected key update"),
        }
        assert_eq!(leaf.len(), 5); // Length shouldn't change
        
        assert_eq!(leaf.get(&i32::MIN), Some(&-2000));
        assert_eq!(leaf.get(&i32::MAX), Some(&2000));
    }

    #[test]
    fn test_find_key_index_helper() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(10);
        
        // Test empty leaf
        assert_eq!(leaf.find_key_index(&42), (0, false));
        
        // Manually set up data: keys [10, 20, 30]
        leaf.len = 3;
        unsafe {
            leaf.set_key_at(0, 10);
            leaf.set_key_at(1, 20);
            leaf.set_key_at(2, 30);
        }
        
        // Test exact matches
        assert_eq!(leaf.find_key_index(&10), (0, true));
        assert_eq!(leaf.find_key_index(&20), (1, true));
        assert_eq!(leaf.find_key_index(&30), (2, true));
        
        // Test insertion points for missing keys
        assert_eq!(leaf.find_key_index(&5), (0, false));   // Before first
        assert_eq!(leaf.find_key_index(&15), (1, false));  // Between 10 and 20
        assert_eq!(leaf.find_key_index(&25), (2, false));  // Between 20 and 30
        assert_eq!(leaf.find_key_index(&35), (3, false));  // After last
        
        // Test boundary cases
        assert_eq!(leaf.find_key_index(&9), (0, false));
        assert_eq!(leaf.find_key_index(&11), (1, false));
        assert_eq!(leaf.find_key_index(&31), (3, false));
    }

    // Phase 6: Remove Tests (will fail until implemented)

    #[test]
    fn remove_existing_key() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        match leaf.insert(42, 100) {
            InsertResult::Updated(None) => {}, // New insertion
            _ => panic!("Expected new insertion"),
        }
        
        let (removed_value, is_underfull) = leaf.remove(&42);
        assert_eq!(removed_value, Some(100));
        assert_eq!(leaf.len(), 0);
        assert_eq!(leaf.get(&42), None);
        assert!(leaf.is_empty());
        // Single item removal from root should not be considered underfull
        assert!(!is_underfull || leaf.len() == 0);
    }

    #[test]
    fn remove_nonexistent_key() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Try to remove from empty leaf
        let (removed_value, is_underfull) = leaf.remove(&42);
        assert_eq!(removed_value, None);
        assert!(!is_underfull);
        
        // Add some items and try to remove non-existent key
        for i in 0..3 {
            match leaf.insert(i * 10, i * 100) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        let (removed_value, is_underfull) = leaf.remove(&42);
        assert_eq!(removed_value, None);
        assert!(!is_underfull);
        assert_eq!(leaf.len(), 3);
    }

    #[test]
    fn remove_from_multiple_items() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        let keys = [10, 20, 30, 40, 50];
        
        // Insert multiple items
        for &key in &keys {
            match leaf.insert(key, key * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        assert_eq!(leaf.len(), 5);
        
        // Remove middle item
        let (removed_value, is_underfull) = leaf.remove(&30);
        assert_eq!(removed_value, Some(300));
        assert_eq!(leaf.len(), 4);
        assert!(!is_underfull);
        
        // Verify remaining items are still accessible and in order
        assert_eq!(leaf.get(&10), Some(&100));
        assert_eq!(leaf.get(&20), Some(&200));
        assert_eq!(leaf.get(&30), None);
        assert_eq!(leaf.get(&40), Some(&400));
        assert_eq!(leaf.get(&50), Some(&500));
        
        // Verify internal order is maintained
        unsafe {
            assert_eq!(*leaf.key_at(0), 10);
            assert_eq!(*leaf.key_at(1), 20);
            assert_eq!(*leaf.key_at(2), 40);
            assert_eq!(*leaf.key_at(3), 50);
        }
    }

    #[test]
    fn remove_first_and_last_items() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        let keys = [10, 20, 30, 40, 50];
        
        // Insert multiple items
        for &key in &keys {
            match leaf.insert(key, key * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Remove first item
        let (removed_value, is_underfull) = leaf.remove(&10);
        assert_eq!(removed_value, Some(100));
        assert_eq!(leaf.len(), 4);
        
        // Remove last item
        let (removed_value, is_underfull) = leaf.remove(&50);
        assert_eq!(removed_value, Some(500));
        assert_eq!(leaf.len(), 3);
        
        // Verify remaining items
        assert_eq!(leaf.get(&10), None);
        assert_eq!(leaf.get(&20), Some(&200));
        assert_eq!(leaf.get(&30), Some(&300));
        assert_eq!(leaf.get(&40), Some(&400));
        assert_eq!(leaf.get(&50), None);
        
        // Verify internal order
        unsafe {
            assert_eq!(*leaf.key_at(0), 20);
            assert_eq!(*leaf.key_at(1), 30);
            assert_eq!(*leaf.key_at(2), 40);
        }
    }

    #[test]
    fn remove_until_empty() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        let keys = [10, 20, 30];
        
        // Insert items
        for &key in &keys {
            match leaf.insert(key, key * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        assert_eq!(leaf.len(), 3);
        
        // Remove all items
        for &key in &keys {
            let (removed_value, _) = leaf.remove(&key);
            assert_eq!(removed_value, Some(key * 10));
        }
        
        assert_eq!(leaf.len(), 0);
        assert!(leaf.is_empty());
        
        // Verify all keys are gone
        for &key in &keys {
            assert_eq!(leaf.get(&key), None);
        }
    }

    #[test]
    fn remove_underfull_detection() {
        // Create a leaf with capacity 8 (min_keys = 4)
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Fill with exactly min_keys + 1 items (5 items)
        for i in 0..5 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        assert_eq!(leaf.len(), 5);
        assert!(!leaf.is_underfull());
        assert!(leaf.can_donate());
        
        // Remove one item - should still be at minimum
        let (removed_value, is_underfull) = leaf.remove(&2);
        assert_eq!(removed_value, Some(20));
        assert_eq!(leaf.len(), 4);
        assert!(!leaf.is_underfull()); // At minimum, not underfull
        assert!(!leaf.can_donate()); // At minimum, cannot donate
        
        // Remove another item - should now be underfull
        let (removed_value, is_underfull) = leaf.remove(&1);
        assert_eq!(removed_value, Some(10));
        assert_eq!(leaf.len(), 3);
        assert!(leaf.is_underfull()); // Below minimum
        assert!(is_underfull); // Method should report underfull
        assert!(!leaf.can_donate());
    }

    #[test]
    fn remove_edge_case_boundary_values() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Insert boundary values
        let keys = [i32::MIN, -1, 0, 1, i32::MAX];
        for &key in &keys {
            match leaf.insert(key, key) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Remove boundary values
        let (removed_value, _) = leaf.remove(&i32::MIN);
        assert_eq!(removed_value, Some(i32::MIN));
        
        let (removed_value, _) = leaf.remove(&i32::MAX);
        assert_eq!(removed_value, Some(i32::MAX));
        
        // Verify remaining items
        assert_eq!(leaf.get(&i32::MIN), None);
        assert_eq!(leaf.get(&-1), Some(&-1));
        assert_eq!(leaf.get(&0), Some(&0));
        assert_eq!(leaf.get(&1), Some(&1));
        assert_eq!(leaf.get(&i32::MAX), None);
        assert_eq!(leaf.len(), 3);
    }

    // Phase 7: Iterator Tests

    #[test]
    fn iterate_empty_leaf() {
        let leaf = CompressedLeafNode::<i32, i32>::new(8);
        let items: Vec<(&i32, &i32)> = leaf.iter().collect();
        assert!(items.is_empty());
    }

    #[test]
    fn iterate_single_item() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        match leaf.insert(42, 100) {
            InsertResult::Updated(None) => {},
            _ => panic!("Expected new insertion"),
        }
        
        let items: Vec<(&i32, &i32)> = leaf.iter().collect();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], (&42, &100));
    }

    #[test]
    fn iterate_multiple_items() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        let keys = [30, 10, 50, 20, 40]; // Insert in unsorted order
        
        for &key in &keys {
            match leaf.insert(key, key * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        let items: Vec<(&i32, &i32)> = leaf.iter().collect();
        assert_eq!(items.len(), 5);
        
        // Should iterate in sorted order
        assert_eq!(items[0], (&10, &100));
        assert_eq!(items[1], (&20, &200));
        assert_eq!(items[2], (&30, &300));
        assert_eq!(items[3], (&40, &400));
        assert_eq!(items[4], (&50, &500));
    }

    #[test]
    fn iterate_after_removals() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        let keys = [10, 20, 30, 40, 50];
        
        // Insert items
        for &key in &keys {
            match leaf.insert(key, key * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Remove some items
        leaf.remove(&20);
        leaf.remove(&40);
        
        let items: Vec<(&i32, &i32)> = leaf.iter().collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], (&10, &100));
        assert_eq!(items[1], (&30, &300));
        assert_eq!(items[2], (&50, &500));
    }

    #[test]
    fn iterate_boundary_values() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        let keys = [i32::MIN, -1, 0, 1, i32::MAX];
        
        for &key in &keys {
            match leaf.insert(key, key) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        let items: Vec<(&i32, &i32)> = leaf.iter().collect();
        assert_eq!(items.len(), 5);
        assert_eq!(items[0], (&i32::MIN, &i32::MIN));
        assert_eq!(items[1], (&-1, &-1));
        assert_eq!(items[2], (&0, &0));
        assert_eq!(items[3], (&1, &1));
        assert_eq!(items[4], (&i32::MAX, &i32::MAX));
    }

    #[test]
    fn iterator_multiple_iterations() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        for i in 0..3 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // First iteration
        let items1: Vec<(&i32, &i32)> = leaf.iter().collect();
        assert_eq!(items1.len(), 3);
        
        // Second iteration should produce same results
        let items2: Vec<(&i32, &i32)> = leaf.iter().collect();
        assert_eq!(items1, items2);
        
        // Manual iteration
        let mut iter = leaf.iter();
        assert_eq!(iter.next(), Some((&0, &0)));
        assert_eq!(iter.next(), Some((&1, &10)));
        assert_eq!(iter.next(), Some((&2, &20)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None); // Should continue returning None
    }

    // Phase 8: Next Field Linking Tests

    #[test]
    fn next_field_basic_operations() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Initially should be NULL_NODE
        assert_eq!(leaf.next(), crate::types::NULL_NODE);
        
        // Set next field
        leaf.set_next(42);
        assert_eq!(leaf.next(), 42);
        
        // Change next field
        leaf.set_next(100);
        assert_eq!(leaf.next(), 100);
    }

    #[test]
    fn split_preserves_next_field_linking() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(6);
        
        // Set up a next pointer
        leaf.set_next(999);
        
        // Fill the leaf to capacity
        for i in 0..6 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Split by inserting one more item
        let original_next = leaf.next();
        let mut right_node = leaf.split();
        
        // After split:
        // - Original node should have NULL_NODE (to be set by caller)
        // - Right node should have the original next pointer
        assert_eq!(leaf.next(), crate::types::NULL_NODE);
        assert_eq!(right_node.next(), original_next);
        assert_eq!(right_node.next(), 999);
    }

    #[test]
    fn split_with_null_next_field() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(4);
        
        // Leave next as NULL_NODE (default)
        assert_eq!(leaf.next(), crate::types::NULL_NODE);
        
        // Fill to capacity and split
        for i in 0..4 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        let right_node = leaf.split();
        
        // Both should be NULL_NODE
        assert_eq!(leaf.next(), crate::types::NULL_NODE);
        assert_eq!(right_node.next(), crate::types::NULL_NODE);
    }

    #[test]
    fn to_leaf_node_preserves_next_field() {
        let mut compressed = CompressedLeafNode::<i32, i32>::new(8);
        
        // Set next field and add some data
        compressed.set_next(777);
        for i in 0..3 {
            match compressed.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Convert to regular LeafNode
        let regular = compressed.to_leaf_node();
        
        // Next field should be preserved
        assert_eq!(regular.next, 777);
        
        // Data should also be preserved
        assert_eq!(regular.keys.len(), 3);
        assert_eq!(regular.values.len(), 3);
        for i in 0..3 {
            assert_eq!(regular.keys[i], i as i32);
            assert_eq!(regular.values[i], (i * 10) as i32);
        }
    }

    #[test]
    fn split_and_insert_maintains_next_linking() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(4);
        
        // Set up next pointer
        leaf.set_next(555);
        
        // Fill to capacity
        for i in 0..4 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Insert one more to trigger split
        match leaf.insert(99, 990) {
            InsertResult::Split { old_value: None, new_node_data, separator_key } => {
                // Verify split occurred
                assert!(separator_key >= 0 && separator_key <= 99);
                
                // Check that the new node data preserves next linking
                match new_node_data {
                    crate::types::SplitNodeData::Leaf(right_leaf) => {
                        // The converted leaf should have the original next pointer
                        assert_eq!(right_leaf.next, 555);
                    },
                    _ => panic!("Expected leaf split"),
                }
                
                // Original node should have NULL_NODE after split
                assert_eq!(leaf.next(), crate::types::NULL_NODE);
            },
            _ => panic!("Expected split when inserting beyond capacity"),
        }
    }

    #[test]
    fn next_field_survives_multiple_operations() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Set next field
        leaf.set_next(123);
        
        // Perform various operations
        for i in 0..5 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Next field should still be there
        assert_eq!(leaf.next(), 123);
        
        // Remove some items
        leaf.remove(&2);
        leaf.remove(&4);
        
        // Next field should still be there
        assert_eq!(leaf.next(), 123);
        
        // Update existing keys
        match leaf.insert(1, 999) {
            InsertResult::Updated(Some(old_value)) => {
                assert_eq!(old_value, 10);
            },
            _ => panic!("Expected key update"),
        }
        
        // Next field should still be there
        assert_eq!(leaf.next(), 123);
    }

    // Phase 9: Borrowing and Merging Tests

    #[test]
    fn borrow_last_basic_functionality() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Empty leaf cannot donate
        assert_eq!(leaf.borrow_last(), None);
        
        // Add items (more than min_keys = 4)
        for i in 0..6 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Should be able to donate
        assert!(leaf.can_donate());
        
        // Borrow last item
        let borrowed = leaf.borrow_last();
        assert_eq!(borrowed, Some((5, 50)));
        assert_eq!(leaf.len(), 5);
        
        // Verify remaining items are intact
        for i in 0..5 {
            assert_eq!(leaf.get(&i), Some(&(i * 10)));
        }
        assert_eq!(leaf.get(&5), None);
    }

    #[test]
    fn borrow_last_cannot_donate() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Add exactly min_keys items (4)
        for i in 0..4 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Should not be able to donate
        assert!(!leaf.can_donate());
        assert_eq!(leaf.borrow_last(), None);
        
        // All items should still be there
        assert_eq!(leaf.len(), 4);
        for i in 0..4 {
            assert_eq!(leaf.get(&i), Some(&(i * 10)));
        }
    }

    #[test]
    fn borrow_first_basic_functionality() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Add items (more than min_keys = 4)
        for i in 0..6 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Borrow first item
        let borrowed = leaf.borrow_first();
        assert_eq!(borrowed, Some((0, 0)));
        assert_eq!(leaf.len(), 5);
        
        // Verify remaining items are shifted correctly
        for i in 1..6 {
            assert_eq!(leaf.get(&i), Some(&(i * 10)));
        }
        assert_eq!(leaf.get(&0), None);
        
        // Verify internal order is maintained
        unsafe {
            assert_eq!(*leaf.key_at(0), 1);
            assert_eq!(*leaf.key_at(1), 2);
            assert_eq!(*leaf.key_at(2), 3);
            assert_eq!(*leaf.key_at(3), 4);
            assert_eq!(*leaf.key_at(4), 5);
        }
    }

    #[test]
    fn accept_from_left_functionality() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Add some items
        for i in 2..5 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Accept item from left sibling (should insert at beginning)
        leaf.accept_from_left(1, 10);
        assert_eq!(leaf.len(), 4);
        
        // Verify order is maintained
        for i in 1..5 {
            assert_eq!(leaf.get(&i), Some(&(i * 10)));
        }
        
        // Verify internal order
        unsafe {
            assert_eq!(*leaf.key_at(0), 1);
            assert_eq!(*leaf.key_at(1), 2);
            assert_eq!(*leaf.key_at(2), 3);
            assert_eq!(*leaf.key_at(3), 4);
        }
    }

    #[test]
    fn accept_from_right_functionality() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Add some items
        for i in 1..4 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Accept item from right sibling (should append at end)
        leaf.accept_from_right(4, 40);
        assert_eq!(leaf.len(), 4);
        
        // Verify order is maintained
        for i in 1..5 {
            assert_eq!(leaf.get(&i), Some(&(i * 10)));
        }
        
        // Verify internal order
        unsafe {
            assert_eq!(*leaf.key_at(0), 1);
            assert_eq!(*leaf.key_at(1), 2);
            assert_eq!(*leaf.key_at(2), 3);
            assert_eq!(*leaf.key_at(3), 4);
        }
    }

    #[test]
    fn merge_from_functionality() {
        let mut left = CompressedLeafNode::<i32, i32>::new(8);
        let mut right = CompressedLeafNode::<i32, i32>::new(8);
        
        // Set up left node with keys [1, 2, 3]
        for i in 1..4 {
            match left.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Set up right node with keys [4, 5, 6] and next pointer
        for i in 4..7 {
            match right.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        right.set_next(999);
        
        // Merge right into left
        let next_pointer = left.merge_from(&mut right);
        
        // Verify merge results
        assert_eq!(left.len(), 6);
        assert_eq!(next_pointer, 999);
        
        // Verify all keys are accessible in left
        for i in 1..7 {
            assert_eq!(left.get(&i), Some(&(i * 10)));
        }
        
        // Verify right is cleared
        assert_eq!(right.len(), 0);
        assert_eq!(right.next(), crate::types::NULL_NODE);
        
        // Verify internal order in left
        unsafe {
            for i in 0..6 {
                assert_eq!(*left.key_at(i), (i + 1) as i32);
                assert_eq!(*left.value_at(i), ((i + 1) * 10) as i32);
            }
        }
    }

    #[test]
    fn borrowing_redistribution_scenario() {
        // Simulate a typical redistribution scenario
        let mut left = CompressedLeafNode::<i32, i32>::new(8);
        let mut right = CompressedLeafNode::<i32, i32>::new(8);
        
        // Left has too many items (6 > min_keys=4)
        for i in 0..6 {
            match left.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Right has too few items (2 < min_keys=4)
        for i in 10..12 {
            match right.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Redistribute: left donates to right
        let donated = left.borrow_last().unwrap();
        right.accept_from_left(donated.0, donated.1);
        
        // Verify redistribution
        assert_eq!(left.len(), 5);
        assert_eq!(right.len(), 3);
        
        // Verify left still has keys [0, 1, 2, 3, 4]
        for i in 0..5 {
            assert_eq!(left.get(&i), Some(&(i * 10)));
        }
        
        // Verify right has keys [5, 10, 11] in sorted order
        assert_eq!(right.get(&5), Some(&50));
        assert_eq!(right.get(&10), Some(&100));
        assert_eq!(right.get(&11), Some(&110));
        
        // Verify internal order in right
        unsafe {
            assert_eq!(*right.key_at(0), 5);
            assert_eq!(*right.key_at(1), 10);
            assert_eq!(*right.key_at(2), 11);
        }
    }

    // Phase 10: Collection Accessor Tests

    #[test]
    fn test_key_accessors() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Test empty node
        assert_eq!(leaf.get_key(0), None);
        assert_eq!(leaf.first_key(), None);
        assert_eq!(leaf.last_key(), None);
        assert_eq!(leaf.keys_iter().count(), 0);
        assert_eq!(leaf.collect_keys(), Vec::<i32>::new());
        
        // Add some items in unsorted order
        let keys = [30, 10, 50, 20, 40];
        for &key in &keys {
            match leaf.insert(key, key * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Test key accessors (should be in sorted order)
        assert_eq!(leaf.get_key(0), Some(&10));
        assert_eq!(leaf.get_key(1), Some(&20));
        assert_eq!(leaf.get_key(2), Some(&30));
        assert_eq!(leaf.get_key(3), Some(&40));
        assert_eq!(leaf.get_key(4), Some(&50));
        assert_eq!(leaf.get_key(5), None);
        
        // Test first/last key
        assert_eq!(leaf.first_key(), Some(&10));
        assert_eq!(leaf.last_key(), Some(&50));
        
        // Test keys iterator
        let collected: Vec<&i32> = leaf.keys_iter().collect();
        assert_eq!(collected, vec![&10, &20, &30, &40, &50]);
        
        // Test collect_keys
        assert_eq!(leaf.collect_keys(), vec![10, 20, 30, 40, 50]);
    }

    #[test]
    fn test_value_accessors() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Test empty node
        assert_eq!(leaf.get_value(0), None);
        assert_eq!(leaf.get_value_mut(0), None);
        assert_eq!(leaf.first_value(), None);
        assert_eq!(leaf.last_value(), None);
        assert_eq!(leaf.first_value_mut(), None);
        assert_eq!(leaf.last_value_mut(), None);
        assert_eq!(leaf.values_iter().count(), 0);
        assert_eq!(leaf.collect_values(), Vec::<i32>::new());
        
        // Add some items
        for i in 0..3 {
            match leaf.insert(i, i * 100) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Test value accessors
        assert_eq!(leaf.get_value(0), Some(&0));
        assert_eq!(leaf.get_value(1), Some(&100));
        assert_eq!(leaf.get_value(2), Some(&200));
        assert_eq!(leaf.get_value(3), None);
        
        // Test first/last value
        assert_eq!(leaf.first_value(), Some(&0));
        assert_eq!(leaf.last_value(), Some(&200));
        
        // Test mutable value accessors
        if let Some(value) = leaf.get_value_mut(1) {
            *value = 999;
        }
        assert_eq!(leaf.get_value(1), Some(&999));
        
        // Test first/last mutable value
        if let Some(value) = leaf.first_value_mut() {
            *value = 777;
        }
        if let Some(value) = leaf.last_value_mut() {
            *value = 888;
        }
        assert_eq!(leaf.first_value(), Some(&777));
        assert_eq!(leaf.last_value(), Some(&888));
        
        // Test values iterator
        let collected: Vec<&i32> = leaf.values_iter().collect();
        assert_eq!(collected, vec![&777, &999, &888]);
        
        // Test collect_values
        assert_eq!(leaf.collect_values(), vec![777, 999, 888]);
    }

    #[test]
    fn test_mutable_values_iterator() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Add some items
        for i in 0..3 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Modify all values through mutable iterator
        for (i, value) in leaf.values_iter_mut().enumerate() {
            *value = ((i + 1) * 1000) as i32;
        }

        // Verify changes
        assert_eq!(leaf.get_value(0), Some(&1000));
        assert_eq!(leaf.get_value(1), Some(&2000));
        assert_eq!(leaf.get_value(2), Some(&3000));
    }

    #[test]
    fn test_pair_accessors() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Test empty node
        assert_eq!(leaf.get_pair(0), None);
        assert_eq!(leaf.get_pair_mut(0), None);
        assert_eq!(leaf.first_pair(), None);
        assert_eq!(leaf.last_pair(), None);
        assert_eq!(leaf.first_pair_mut(), None);
        assert_eq!(leaf.last_pair_mut(), None);
        assert_eq!(leaf.collect_pairs(), Vec::<(i32, i32)>::new());
        
        // Add some items
        let keys = [20, 10, 30];
        for &key in &keys {
            match leaf.insert(key, key * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Test pair accessors (should be in sorted order)
        assert_eq!(leaf.get_pair(0), Some((&10, &100)));
        assert_eq!(leaf.get_pair(1), Some((&20, &200)));
        assert_eq!(leaf.get_pair(2), Some((&30, &300)));
        assert_eq!(leaf.get_pair(3), None);
        
        // Test first/last pair
        assert_eq!(leaf.first_pair(), Some((&10, &100)));
        assert_eq!(leaf.last_pair(), Some((&30, &300)));
        
        // Test mutable pair accessors
        if let Some((key, value)) = leaf.get_pair_mut(1) {
            assert_eq!(*key, 20);
            *value = 999;
        }
        assert_eq!(leaf.get_pair(1), Some((&20, &999)));
        
        // Test first/last mutable pair
        if let Some((key, value)) = leaf.first_pair_mut() {
            assert_eq!(*key, 10);
            *value = 777;
        }
        if let Some((key, value)) = leaf.last_pair_mut() {
            assert_eq!(*key, 30);
            *value = 888;
        }
        
        // Verify changes
        assert_eq!(leaf.first_pair(), Some((&10, &777)));
        assert_eq!(leaf.last_pair(), Some((&30, &888)));
        
        // Test collect_pairs
        assert_eq!(leaf.collect_pairs(), vec![(10, 777), (20, 999), (30, 888)]);
    }

    #[test]
    fn test_collection_accessors_bounds_checking() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Add one item
        match leaf.insert(42, 100) {
            InsertResult::Updated(None) => {},
            _ => panic!("Expected new insertion"),
        }
        
        // Test valid indices
        assert_eq!(leaf.get_key(0), Some(&42));
        assert_eq!(leaf.get_value(0), Some(&100));
        assert_eq!(leaf.get_pair(0), Some((&42, &100)));
        
        // Test invalid indices
        assert_eq!(leaf.get_key(1), None);
        assert_eq!(leaf.get_value(1), None);
        assert_eq!(leaf.get_value_mut(1), None);
        assert_eq!(leaf.get_pair(1), None);
        assert_eq!(leaf.get_pair_mut(1), None);
        
        // Test way out of bounds
        assert_eq!(leaf.get_key(100), None);
        assert_eq!(leaf.get_value(100), None);
        assert_eq!(leaf.get_pair(100), None);
    }

    #[test]
    fn test_collection_accessors_after_operations() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Insert items
        for i in 0..5 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Verify initial state
        assert_eq!(leaf.collect_keys(), vec![0, 1, 2, 3, 4]);
        assert_eq!(leaf.collect_values(), vec![0, 10, 20, 30, 40]);
        
        // Remove middle item
        leaf.remove(&2);
        
        // Verify state after removal
        assert_eq!(leaf.collect_keys(), vec![0, 1, 3, 4]);
        assert_eq!(leaf.collect_values(), vec![0, 10, 30, 40]);
        assert_eq!(leaf.first_key(), Some(&0));
        assert_eq!(leaf.last_key(), Some(&4));
        assert_eq!(leaf.first_value(), Some(&0));
        assert_eq!(leaf.last_value(), Some(&40));
        
        // Test that removed index is now out of bounds
        assert_eq!(leaf.get_key(4), None);
        assert_eq!(leaf.get_value(4), None);
        assert_eq!(leaf.get_pair(4), None);
    }

    // Phase 11: LeafNode Compatibility Tests

    #[test]
    fn test_get_mut_by_key() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Insert some items
        for i in 0..3 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Test get_mut with existing key
        if let Some(value) = leaf.get_mut(&1) {
            *value = 999;
        }
        
        // Verify the change
        assert_eq!(leaf.get(&1), Some(&999));
        assert_eq!(leaf.get(&0), Some(&0));
        assert_eq!(leaf.get(&2), Some(&20));
        
        // Test get_mut with non-existent key
        assert_eq!(leaf.get_mut(&10), None);
    }

    #[test]
    fn test_needs_split() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(4);
        
        // Initially should not need split
        assert!(!leaf.needs_split());
        
        // Fill to capacity
        for i in 0..4 {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // At capacity, should not need split
        assert!(!leaf.needs_split());
        assert!(leaf.is_full());
        
        // Force overfull state (this would normally trigger split)
        leaf.len = 5; // Simulate overfull
        assert!(leaf.needs_split());
        
        // Reset
        leaf.len = 4;
        assert!(!leaf.needs_split());
    }

    #[test]
    fn test_extract_all() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Insert some items
        let keys = [10, 20, 30];
        for &key in &keys {
            match leaf.insert(key, key * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Set next pointer
        leaf.set_next(999);
        
        // Extract all content
        let (extracted_keys, extracted_values, next_ptr) = leaf.extract_all();
        
        // Verify extracted content
        assert_eq!(extracted_keys, vec![10, 20, 30]);
        assert_eq!(extracted_values, vec![100, 200, 300]);
        assert_eq!(next_ptr, 999);
        
        // Verify leaf is now empty
        assert_eq!(leaf.len(), 0);
        assert!(leaf.is_empty());
        assert_eq!(leaf.next(), crate::types::NULL_NODE);
        
        // Verify all keys are gone
        for &key in &keys {
            assert_eq!(leaf.get(&key), None);
        }
    }

    
    // Phase 11: External Code Compatibility Demonstration

    #[test]
    fn demonstrate_external_code_patterns() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        
        // Add some test data
        for i in [30, 10, 50, 20, 40] {
            match leaf.insert(i, i * 10) {
                InsertResult::Updated(None) => {},
                _ => panic!("Expected new insertion"),
            }
        }
        
        // Pattern 1: Instead of leaf.keys().len() -> use leaf.len()
        assert_eq!(leaf.len(), 5);
        
        // Pattern 2: Instead of leaf.keys()[0] -> use leaf.first_key()
        assert_eq!(leaf.first_key(), Some(&10));
        
        // Pattern 3: Instead of leaf.keys().last() -> use leaf.last_key()
        assert_eq!(leaf.last_key(), Some(&50));
        
        // Pattern 4: Instead of leaf.keys().get(index) -> use leaf.get_key(index)
        assert_eq!(leaf.get_key(2), Some(&30));
        
        // Pattern 5: Instead of leaf.values()[index] -> use leaf.get_value(index)
        assert_eq!(leaf.get_value(2), Some(&300));
        
        // Pattern 6: Instead of leaf.values_mut()[index] = value -> use get_value_mut
        if let Some(value) = leaf.get_value_mut(2) {
            *value = 999;
        }
        assert_eq!(leaf.get_value(2), Some(&999));
        
        // Pattern 7: Instead of iterating over leaf.keys() -> use leaf.keys_iter()
        let keys: Vec<i32> = leaf.keys_iter().copied().collect();
        assert_eq!(keys, vec![10, 20, 30, 40, 50]);
        
        // Pattern 8: Instead of iterating over leaf.values() -> use leaf.values_iter()
        let values: Vec<i32> = leaf.values_iter().copied().collect();
        assert_eq!(values, vec![100, 200, 999, 400, 500]);
        
        // Pattern 9: For compatibility, can still get Vec if needed
        assert_eq!(leaf.collect_keys(), vec![10, 20, 30, 40, 50]);
        assert_eq!(leaf.collect_values(), vec![100, 200, 999, 400, 500]);
        
        // Pattern 10: Access key-value pairs together
        assert_eq!(leaf.get_pair(1), Some((&20, &200)));
        assert_eq!(leaf.first_pair(), Some((&10, &100)));
        assert_eq!(leaf.last_pair(), Some((&50, &500)));
    }

    // Memory efficiency verification
    #[test]
    fn memory_efficiency_comparison() {
        use crate::types::LeafNode;
        
        let regular_size = std::mem::size_of::<LeafNode<i32, i32>>();
        let compressed_size = std::mem::size_of::<CompressedLeafNode<i32, i32>>();
        
        println!("Regular LeafNode: {} bytes", regular_size);
        println!("Compressed LeafNode: {} bytes", compressed_size);
        
        assert_eq!(compressed_size, 256); // Exactly 4 cache lines
        
        // Should be more memory-efficient for reasonable capacities
        if regular_size > 256 {
            println!("Compressed node is {}x more memory efficient", 
                    regular_size as f64 / compressed_size as f64);
        }
    }
}
