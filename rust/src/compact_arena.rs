//! Compact arena implementation using Vec<T> instead of Vec<Option<T>>
//! This eliminates the Option wrapper overhead for better performance

use std::convert::TryFrom;
use std::fmt::Debug;

pub type NodeId = u32;
pub const NULL_NODE: NodeId = u32::MAX;

/// Statistics for a compact arena
#[derive(Debug, Clone, Copy)]
pub struct CompactArenaStats {
    pub total_capacity: usize,
    pub allocated_count: usize,
    pub free_count: usize,
    pub utilization: f64,
    pub fragmentation: f64,
}

/// Compact arena allocator that eliminates Option wrapper overhead
/// Uses Vec<T> with a separate free list and generation tracking
#[derive(Debug)]
pub struct CompactArena<T> {
    /// Direct storage without Option wrapper
    storage: Vec<T>,
    /// Free slot indices for reuse
    free_list: Vec<usize>,
    /// Generation counter for safety (optional)
    generation: u32,
    /// Track which slots are actually allocated
    allocated_mask: Vec<bool>,
}

impl<T> CompactArena<T> {
    /// Create a new empty compact arena
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            free_list: Vec::new(),
            generation: 0,
            allocated_mask: Vec::new(),
        }
    }

    /// Create a new compact arena with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            storage: Vec::with_capacity(capacity),
            free_list: Vec::new(),
            generation: 0,
            allocated_mask: Vec::with_capacity(capacity),
        }
    }

    /// Allocate a new item in the arena and return its ID
    #[inline]
    pub fn allocate(&mut self, item: T) -> NodeId {
        self.generation = self.generation.wrapping_add(1);

        let index = if let Some(free_index) = self.free_list.pop() {
            // Reuse a free slot
            self.storage[free_index] = item;
            self.allocated_mask[free_index] = true;
            free_index
        } else {
            // Allocate new slot
            let index = self.storage.len();
            self.storage.push(item);
            self.allocated_mask.push(true);
            index
        };

        NodeId::try_from(index).expect("Index should fit in NodeId")
    }

    /// Deallocate an item from the arena and return it (requires Default)
    #[inline]
    pub fn deallocate(&mut self, id: NodeId) -> Option<T>
    where
        T: Default,
    {
        if id == NULL_NODE {
            return None;
        }

        let index = usize::try_from(id).ok()?;

        // Check if the slot is actually allocated
        if !self.allocated_mask.get(index).copied().unwrap_or(false) {
            return None;
        }

        // Mark as free
        self.allocated_mask[index] = false;
        self.free_list.push(index);

        // Replace with default and return the old value
        let old_value = std::mem::take(&mut self.storage[index]);
        Some(old_value)
    }

    /// Deallocate without returning the value (for types that don't implement Default)
    pub fn deallocate_no_return(&mut self, id: NodeId) -> bool {
        if id == NULL_NODE {
            return false;
        }

        let index = usize::try_from(id).ok().unwrap_or(usize::MAX);

        // Check if the slot is actually allocated
        if index >= self.allocated_mask.len() || !self.allocated_mask[index] {
            return false;
        }

        // Mark as free
        self.allocated_mask[index] = false;
        self.free_list.push(index);
        true
    }

    /// Get a reference to an item in the arena
    #[inline]
    pub fn get(&self, id: NodeId) -> Option<&T> {
        if id == NULL_NODE {
            return None;
        }

        let index = usize::try_from(id).ok()?;

        // Check bounds and allocation status
        if index < self.storage.len() && self.allocated_mask.get(index).copied().unwrap_or(false) {
            Some(&self.storage[index])
        } else {
            None
        }
    }

    /// Get a mutable reference to an item in the arena
    #[inline]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        if id == NULL_NODE {
            return None;
        }

        let index = usize::try_from(id).ok()?;

        // Check bounds and allocation status
        if index < self.storage.len() && self.allocated_mask.get(index).copied().unwrap_or(false) {
            Some(&mut self.storage[index])
        } else {
            None
        }
    }

    /// Unsafe fast access without bounds checking or allocation verification
    ///
    /// # Safety
    /// Caller must ensure id is valid and allocated
    pub unsafe fn get_unchecked(&self, id: NodeId) -> &T {
        let index = id as usize;
        self.storage.get_unchecked(index)
    }

    /// Unsafe fast mutable access without bounds checking or allocation verification
    ///
    /// # Safety
    /// Caller must ensure id is valid and allocated
    pub unsafe fn get_unchecked_mut(&mut self, id: NodeId) -> &mut T {
        let index = id as usize;
        self.storage.get_unchecked_mut(index)
    }

    /// Check if an ID is valid and allocated
    pub fn contains(&self, id: NodeId) -> bool {
        if id == NULL_NODE {
            return false;
        }

        let index = usize::try_from(id).unwrap_or(usize::MAX);
        index < self.storage.len() && self.allocated_mask.get(index).copied().unwrap_or(false)
    }

    /// Get arena statistics
    pub fn stats(&self) -> CompactArenaStats {
        let total_capacity = self.storage.capacity();
        let allocated_count = self
            .allocated_mask
            .iter()
            .filter(|&&allocated| allocated)
            .count();
        let free_count = self.free_list.len();
        let utilization = if total_capacity > 0 {
            allocated_count as f64 / total_capacity as f64
        } else {
            0.0
        };
        let fragmentation = if allocated_count > 0 {
            free_count as f64 / (allocated_count + free_count) as f64
        } else {
            0.0
        };

        CompactArenaStats {
            total_capacity,
            allocated_count,
            free_count,
            utilization,
            fragmentation,
        }
    }

    /// Compact the arena by removing gaps (expensive operation)
    pub fn compact(&mut self)
    where
        T: Clone,
    {
        let mut new_storage = Vec::with_capacity(self.storage.len());
        let mut new_allocated_mask = Vec::with_capacity(self.allocated_mask.len());
        let mut index_mapping = vec![NULL_NODE; self.storage.len()];

        // Copy allocated items to new storage
        for (old_index, (item, &allocated)) in self
            .storage
            .iter()
            .zip(self.allocated_mask.iter())
            .enumerate()
        {
            if allocated {
                let new_index = new_storage.len();
                new_storage.push(item.clone());
                new_allocated_mask.push(true);
                index_mapping[old_index] = new_index as NodeId;
            }
        }

        self.storage = new_storage;
        self.allocated_mask = new_allocated_mask;
        self.free_list.clear();

        // Note: This breaks existing NodeIds!
        // In a real implementation, you'd need to update all references
    }

    /// Get the number of allocated items
    pub fn len(&self) -> usize {
        self.allocated_mask
            .iter()
            .filter(|&&allocated| allocated)
            .count()
    }

    /// Check if the arena is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the total capacity
    pub fn capacity(&self) -> usize {
        self.storage.capacity()
    }

    /// Clear all items from the arena
    pub fn clear(&mut self) {
        self.storage.clear();
        self.allocated_mask.clear();
        self.free_list.clear();
        self.generation = 0;
    }

    /// Get the number of free slots
    pub fn free_count(&self) -> usize {
        self.free_list.len()
    }

    /// Get the number of allocated items
    pub fn allocated_count(&self) -> usize {
        self.len()
    }

    /// Get the utilization ratio (allocated / total capacity)
    pub fn utilization(&self) -> f64 {
        let stats = self.stats();
        stats.utilization
    }
}

impl<T> Default for CompactArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

// For types that implement Default, we can provide better deallocation
impl<T: Default> CompactArena<T> {
    /// Deallocate and replace with default value
    pub fn deallocate_with_default(&mut self, id: NodeId) -> Option<T> {
        if id == NULL_NODE {
            return None;
        }

        let index = usize::try_from(id).ok()?;

        // Check if the slot is actually allocated
        if !self.allocated_mask.get(index).copied().unwrap_or(false) {
            return None;
        }

        // Mark as free and replace with default
        self.allocated_mask[index] = false;
        self.free_list.push(index);

        let old_value = std::mem::take(&mut self.storage[index]);
        Some(old_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_arena_basic_operations() {
        let mut arena = CompactArena::new();

        // Allocate some items
        let id1 = arena.allocate(42);
        let id2 = arena.allocate(84);
        let id3 = arena.allocate(126);

        // Test retrieval
        assert_eq!(arena.get(id1), Some(&42));
        assert_eq!(arena.get(id2), Some(&84));
        assert_eq!(arena.get(id3), Some(&126));

        // Test contains
        assert!(arena.contains(id1));
        assert!(arena.contains(id2));
        assert!(arena.contains(id3));
        assert!(!arena.contains(NULL_NODE));

        // Test stats
        let stats = arena.stats();
        assert_eq!(stats.allocated_count, 3);
        assert_eq!(stats.free_count, 0);
    }

    #[test]
    fn test_compact_arena_with_default() {
        let mut arena: CompactArena<i32> = CompactArena::new();

        let id1 = arena.allocate(42);
        let id2 = arena.allocate(84);

        // Deallocate with default
        let removed = arena.deallocate_with_default(id1);
        assert_eq!(removed, Some(42));
        assert!(!arena.contains(id1));
        assert!(arena.contains(id2));

        // Reuse the slot
        let id3 = arena.allocate(168);
        assert_eq!(arena.get(id3), Some(&168));

        let stats = arena.stats();
        assert_eq!(stats.allocated_count, 2);
        assert_eq!(stats.free_count, 0); // Should be reused
    }

    #[test]
    fn test_unsafe_access() {
        let mut arena = CompactArena::new();
        let id = arena.allocate(42);

        unsafe {
            assert_eq!(*arena.get_unchecked(id), 42);
            *arena.get_unchecked_mut(id) = 84;
            assert_eq!(*arena.get_unchecked(id), 84);
        }
    }
}

// ============================================================================
// BPLUSTREE ARENA ALLOCATION HELPERS
// ============================================================================

use crate::types::{BPlusTreeMap, BranchNode, LeafNode};

impl<K: Ord + Clone, V: Clone> BPlusTreeMap<K, V> {
    // ============================================================================
    // ARENA ALLOCATION METHODS
    // ============================================================================

    /// Allocate a new leaf node in the arena and return its ID.
    #[inline]
    pub fn allocate_leaf(&mut self, leaf: LeafNode<K, V>) -> NodeId {
        self.leaf_arena.allocate(leaf)
    }

    /// Allocate a new branch node in the arena and return its ID.
    #[inline]
    pub fn allocate_branch(&mut self, branch: BranchNode<K, V>) -> NodeId {
        self.branch_arena.allocate(branch)
    }

    /// Deallocate a leaf node from the arena.
    #[inline]
    pub fn deallocate_leaf(&mut self, id: NodeId) -> Option<LeafNode<K, V>> {
        self.leaf_arena.deallocate(id)
    }

    /// Deallocate a branch node from the arena.
    #[inline]
    pub fn deallocate_branch(&mut self, id: NodeId) -> Option<BranchNode<K, V>> {
        self.branch_arena.deallocate(id)
    }

    // ============================================================================
    // ARENA STATISTICS AND MANAGEMENT
    // ============================================================================

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

    /// Get the number of free branch nodes in the arena.
    pub fn free_branch_count(&self) -> usize {
        self.branch_arena.free_count()
    }

    /// Get the number of allocated branch nodes in the arena.
    pub fn allocated_branch_count(&self) -> usize {
        self.branch_arena.allocated_count()
    }

    /// Get the branch arena utilization ratio.
    pub fn branch_utilization(&self) -> f64 {
        self.branch_arena.utilization()
    }

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

    // ============================================================================
    // UNSAFE ARENA ACCESS
    // ============================================================================

    /// Unsafe fast access to leaf node (no bounds checking)
    ///
    /// # Safety
    /// Caller must ensure id is valid and allocated
    pub unsafe fn get_leaf_unchecked(&self, id: NodeId) -> &LeafNode<K, V> {
        self.leaf_arena.get_unchecked(id)
    }

    /// Unsafe fast access to branch node (no bounds checking)
    ///
    /// # Safety
    /// Caller must ensure id is valid and allocated
    pub unsafe fn get_branch_unchecked(&self, id: NodeId) -> &BranchNode<K, V> {
        self.branch_arena.get_unchecked(id)
    }
}
