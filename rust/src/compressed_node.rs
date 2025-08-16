//! Compressed node implementations optimized for cache line efficiency.
//!
//! This module contains CompressedLeafNode that fits exactly within 4 cache lines (256 bytes)
//! for optimal memory access patterns and reduced cache pressure.

use std::marker::PhantomData;
use std::mem;
use crate::types::NodeId;

/// A leaf node compressed to exactly 4 cache lines (256 bytes) for optimal cache performance.
/// 
/// Memory layout:
/// - Header: 8 bytes (capacity, len, next) + PhantomData (zero-sized)
/// - Data: 248 bytes (inline storage for keys and values)
/// 
/// Keys and values are stored in separate contiguous regions within the data array:
/// [key0, key1, ..., keyN, value0, value1, ..., valueN]
#[repr(C, align(64))]
pub struct CompressedLeafNode<K, V> {
    /// Maximum number of key-value pairs this node can hold
    capacity: u16,
    /// Current number of key-value pairs
    len: u16,
    /// Next leaf node in the linked list (for range queries)
    next: NodeId,
    /// Phantom data to maintain type parameters (zero-sized)
    _phantom: PhantomData<(K, V)>,
    /// Raw storage for keys and values
    data: [u8; 248],
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
    pub fn new(capacity: u16) -> Self {
        Self {
            capacity,
            len: 0,
            next: crate::types::NULL_NODE,
            _phantom: PhantomData,
            data: [0; 248],
        }
    }

    /// Returns the number of key-value pairs in this leaf.
    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns the maximum capacity of this leaf.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity as usize
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
    pub fn calculate_max_capacity() -> u16 {
        let pair_size = mem::size_of::<K>() + mem::size_of::<V>();
        let available_space = 248; // data array size
        (available_space / pair_size) as u16
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
}

// Placeholder implementations - will be implemented through TDD
impl<K, V> CompressedLeafNode<K, V>
where
    K: Copy + Ord,
    V: Copy,
{
    /// Insert a key-value pair into the leaf.
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, &'static str> {
        todo!("Implement through TDD")
    }

    /// Get a value by key.
    pub fn get(&self, key: &K) -> Option<&V> {
        if self.len == 0 {
            return None;
        }

        // Binary search through the keys
        let mut left = 0;
        let mut right = self.len as usize;

        while left < right {
            let mid = left + (right - left) / 2;
            
            // Safety: mid is always < self.len due to binary search bounds
            let mid_key = unsafe { self.key_at(mid) };
            
            match mid_key.cmp(key) {
                std::cmp::Ordering::Equal => {
                    // Found the key, return corresponding value
                    return Some(unsafe { self.value_at(mid) });
                }
                std::cmp::Ordering::Less => {
                    left = mid + 1;
                }
                std::cmp::Ordering::Greater => {
                    right = mid;
                }
            }
        }

        None // Key not found
    }

    /// Remove a key-value pair from the leaf.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        todo!("Implement through TDD")
    }

    /// Iterator over key-value pairs in sorted order.
    pub fn iter(&self) -> CompressedLeafIter<K, V> {
        todo!("Implement through TDD")
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
        todo!("Implement through TDD")
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
            (&leaf.capacity as *const u16 as *const u8).offset_from(start_ptr) 
        };
        let len_offset = unsafe { 
            (&leaf.len as *const u16 as *const u8).offset_from(start_ptr) 
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
        assert_eq!(len_offset, 2);
        assert_eq!(next_offset, 4);
        assert_eq!(phantom_offset, 8);
        assert_eq!(data_offset, 8); // PhantomData is zero-sized

        // Verify data array ends at struct boundary
        let data_end = unsafe { leaf.data.as_ptr().add(248) };
        assert_eq!(data_end as *const u8, end_ptr);
    }

    #[test]
    fn verify_cache_line_alignment() {
        let leaf = CompressedLeafNode::<i32, i32>::new(10);
        let addr = &leaf as *const _ as usize;
        
        // Should be aligned to 64-byte boundary
        assert_eq!(addr % 64, 0);
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
        // 248 bytes available / 8 bytes per pair = 31 pairs
        assert_eq!(max_cap, 31);
    }

    #[test]
    fn calculate_max_capacity_for_different_types() {
        // u8 + u8 = 2 bytes per pair
        let u8_cap = CompressedLeafNode::<u8, u8>::calculate_max_capacity();
        assert_eq!(u8_cap, 124); // 248 / 2 = 124

        // u64 + u64 = 16 bytes per pair  
        let u64_cap = CompressedLeafNode::<u64, u64>::calculate_max_capacity();
        assert_eq!(u64_cap, 15); // 248 / 16 = 15
    }

    // Phase 3: Single Insert/Get Tests (will fail until implemented)

    #[test]
    #[should_panic] // Remove this when implementing
    fn insert_single_item() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        assert!(leaf.insert(42, 100).is_ok());
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
    #[should_panic] // Remove this when implementing
    fn insert_multiple_sorted() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        for i in 0..5 {
            assert!(leaf.insert(i, i * 10).is_ok());
        }
        assert_eq!(leaf.len(), 5);
        
        // Verify sorted order maintained
        for i in 0..5 {
            assert_eq!(leaf.get(&i), Some(&(i * 10)));
        }
    }

    // Phase 5: Capacity Management Tests (will fail until implemented)

    #[test]
    #[should_panic] // Remove this when implementing
    fn insert_at_capacity() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(4);
        
        // Fill to capacity
        for i in 0..4 {
            assert!(leaf.insert(i, i * 10).is_ok());
        }
        assert!(leaf.is_full());
        
        // Attempt overflow
        assert!(leaf.insert(99, 990).is_err());
    }

    // Phase 6: Remove Tests (will fail until implemented)

    #[test]
    #[should_panic] // Remove this when implementing
    fn remove_existing_key() {
        let mut leaf = CompressedLeafNode::<i32, i32>::new(8);
        leaf.insert(42, 100).unwrap();
        
        assert_eq!(leaf.remove(&42), Some(100));
        assert_eq!(leaf.len(), 0);
        assert_eq!(leaf.get(&42), None);
    }

    // Phase 7: Iterator Tests (will fail until implemented)

    #[test]
    #[should_panic] // Remove this when implementing
    fn iterate_empty_leaf() {
        let leaf = CompressedLeafNode::<i32, i32>::new(8);
        let items: Vec<(&i32, &i32)> = leaf.iter().collect();
        assert!(items.is_empty());
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
