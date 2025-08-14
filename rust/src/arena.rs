use std::convert::TryFrom;
/// Enhanced generic arena allocator that eliminates all arena duplication
/// This replaces ~160 lines of duplicated arena code with a single generic implementation
use std::fmt::Debug;

pub type NodeId = u32;
pub const NULL_NODE: NodeId = u32::MAX;

/// Statistics for an arena
#[derive(Debug, Clone, Copy)]
pub struct ArenaStats {
    pub total_capacity: usize,
    pub allocated_count: usize,
    pub free_count: usize,
    pub utilization: f64,
    pub fragmentation: f64,
}

/// Generic arena allocator for any node type
/// Eliminates duplication between leaf and branch arena implementations
#[derive(Debug)]
pub struct Arena<T> {
    storage: Vec<Option<T>>,
    free_ids: Vec<NodeId>,
}

impl<T> Arena<T> {
    /// Create a new empty arena
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            free_ids: Vec::new(),
        }
    }

    /// Create a new arena with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            storage: Vec::with_capacity(capacity),
            free_ids: Vec::new(),
        }
    }

    /// Allocate a new item in the arena and return its ID
    pub fn allocate(&mut self, item: T) -> NodeId {
        let id = self.next_id();

        // Extend storage if needed
        let id_usize = usize::try_from(id).expect("NodeId should fit in usize");
        if id_usize >= self.storage.len() {
            self.storage.resize_with(id_usize + 1, || None);
        }

        self.storage[id_usize] = Some(item);
        id
    }

    /// Deallocate an item from the arena and return it
    pub fn deallocate(&mut self, id: NodeId) -> Option<T> {
        if id == NULL_NODE {
            return None;
        }

        let id_usize = usize::try_from(id).ok()?;
        self.storage.get_mut(id_usize)?.take().inspect(|_item| {
            self.free_ids.push(id);
        })
    }

    /// Get a reference to an item in the arena
    pub fn get(&self, id: NodeId) -> Option<&T> {
        if id == NULL_NODE {
            return None;
        }
        let id_usize = usize::try_from(id).ok()?;
        self.storage.get(id_usize)?.as_ref()
    }

    /// Get a mutable reference to an item in the arena
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        if id == NULL_NODE {
            return None;
        }
        let id_usize = usize::try_from(id).ok()?;
        self.storage.get_mut(id_usize)?.as_mut()
    }

    /// Check if an ID is valid and allocated
    pub fn contains(&self, id: NodeId) -> bool {
        if id == NULL_NODE {
            return false;
        }
        let id_usize = usize::try_from(id).unwrap_or(usize::MAX);
        self.storage
            .get(id_usize)
            .is_some_and(|item| item.is_some())
    }

    /// Get the next available ID (from free list or storage length)
    fn next_id(&mut self) -> NodeId {
        self.free_ids.pop().unwrap_or_else(|| {
            u32::try_from(self.storage.len()).expect("Arena size exceeds maximum NodeId capacity")
        })
    }

    // ============================================================================
    // STATISTICS METHODS - Eliminates all arena statistics duplication
    // ============================================================================

    /// Get the number of free (deallocated but reusable) slots
    pub fn free_count(&self) -> usize {
        self.free_ids.len()
    }

    /// Get the number of currently allocated items
    pub fn allocated_count(&self) -> usize {
        self.storage.iter().filter(|item| item.is_some()).count()
    }

    /// Get the total capacity (allocated + free slots)
    pub fn total_capacity(&self) -> usize {
        self.storage.len()
    }

    /// Get the utilization ratio (allocated / total_capacity)
    pub fn utilization(&self) -> f64 {
        if self.storage.is_empty() {
            0.0
        } else {
            self.allocated_count() as f64 / self.total_capacity() as f64
        }
    }

    /// Check if the arena is empty (no allocated items)
    pub fn is_empty(&self) -> bool {
        self.allocated_count() == 0
    }

    /// Get the number of items that can be allocated without growing storage
    pub fn available_capacity(&self) -> usize {
        self.free_count() + (usize::MAX - self.total_capacity()).min(1000) // Reasonable limit
    }

    /// Get fragmentation ratio (free_count / total_capacity)
    pub fn fragmentation(&self) -> f64 {
        if self.storage.is_empty() {
            0.0
        } else {
            self.free_count() as f64 / self.total_capacity() as f64
        }
    }

    /// Get all statistics in a single struct
    pub fn stats(&self) -> ArenaStats {
        ArenaStats {
            total_capacity: self.total_capacity(),
            allocated_count: self.allocated_count(),
            free_count: self.free_count(),
            utilization: self.utilization(),
            fragmentation: self.fragmentation(),
        }
    }

    // ============================================================================
    // MAINTENANCE METHODS
    // ============================================================================

    /// Compact the arena by removing unused slots at the end
    /// This can help reduce memory usage after many deallocations
    pub fn compact(&mut self) {
        // Find the last allocated item
        let mut last_allocated = 0;
        for (i, item) in self.storage.iter().enumerate().rev() {
            if item.is_some() {
                last_allocated = i + 1;
                break;
            }
        }

        // Truncate storage to remove unused slots at the end
        self.storage.truncate(last_allocated);

        // Remove free IDs that are now out of bounds
        self.free_ids
            .retain(|&id| usize::try_from(id).is_ok_and(|id_usize| id_usize < last_allocated));
    }

    /// Clear all items from the arena
    pub fn clear(&mut self) {
        self.storage.clear();
        self.free_ids.clear();
    }

    /// Shrink the arena's capacity to fit current usage
    pub fn shrink_to_fit(&mut self) {
        self.compact();
        self.storage.shrink_to_fit();
        self.free_ids.shrink_to_fit();
    }

    // ============================================================================
    // ITERATION SUPPORT
    // ============================================================================

    /// Iterate over all allocated items with their IDs
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &T)> {
        self.storage.iter().enumerate().filter_map(|(id, item)| {
            let node_id = u32::try_from(id).ok()?;
            item.as_ref().map(|item| (node_id, item))
        })
    }

    /// Iterate over all allocated items mutably with their IDs
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (NodeId, &mut T)> {
        self.storage
            .iter_mut()
            .enumerate()
            .filter_map(|(id, item)| {
                let node_id = u32::try_from(id).ok()?;
                item.as_mut().map(|item| (node_id, item))
            })
    }

    /// Iterate over all allocated items (without IDs)
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.storage.iter().filter_map(|item| item.as_ref())
    }

    /// Iterate over all allocated items mutably (without IDs)
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.storage.iter_mut().filter_map(|item| item.as_mut())
    }

    // ============================================================================
    // DEBUG AND VALIDATION
    // ============================================================================

    /// Validate the internal consistency of the arena
    pub fn validate(&self) -> Result<(), String> {
        // Check that all free IDs are valid and point to None
        for &free_id in &self.free_ids {
            let id_usize = usize::try_from(free_id)
                .map_err(|_| format!("Free ID {} cannot be converted to usize", free_id))?;

            if id_usize >= self.storage.len() {
                return Err(format!("Free ID {} is out of bounds", free_id));
            }
            if self.storage[id_usize].is_some() {
                return Err(format!("Free ID {} points to allocated item", free_id));
            }
        }

        // Check for duplicate free IDs
        let mut sorted_free_ids = self.free_ids.clone();
        sorted_free_ids.sort_unstable();
        for i in 1..sorted_free_ids.len() {
            if sorted_free_ids[i - 1] == sorted_free_ids[i] {
                return Err(format!("Duplicate free ID: {}", sorted_free_ids[i]));
            }
        }

        Ok(())
    }

    /// Get debug information about the arena
    pub fn debug_info(&self) -> ArenaDebugInfo {
        ArenaDebugInfo {
            total_capacity: self.total_capacity(),
            allocated_count: self.allocated_count(),
            free_count: self.free_count(),
            utilization: self.utilization(),
            fragmentation: self.fragmentation(),
            free_ids: self.free_ids.clone(),
        }
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Debug information for arena analysis
#[derive(Debug, Clone)]
pub struct ArenaDebugInfo {
    pub total_capacity: usize,
    pub allocated_count: usize,
    pub free_count: usize,
    pub utilization: f64,
    pub fragmentation: f64,
    pub free_ids: Vec<NodeId>,
}

// ============================================================================
// BPLUSTREE ARENA ALLOCATION HELPERS
// ============================================================================

use crate::types::{BPlusTreeMap, LeafNode, BranchNode};

impl<K: Ord + Clone, V: Clone> BPlusTreeMap<K, V> {
    // ============================================================================
    // ARENA ALLOCATION METHODS
    // ============================================================================

    /// Allocate a new leaf node in the arena and return its ID.
    pub fn allocate_leaf(&mut self, leaf: LeafNode<K, V>) -> NodeId {
        self.leaf_arena.allocate(leaf)
    }

    /// Allocate a new branch node in the arena and return its ID.
    pub fn allocate_branch(&mut self, branch: BranchNode<K, V>) -> NodeId {
        self.branch_arena.allocate(branch)
    }

    /// Deallocate a leaf node from the arena.
    pub fn deallocate_leaf(&mut self, id: NodeId) -> Option<LeafNode<K, V>> {
        self.leaf_arena.deallocate(id)
    }

    /// Deallocate a branch node from the arena.
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
    pub fn leaf_arena_stats(&self) -> crate::compact_arena::CompactArenaStats {
        self.leaf_arena.stats()
    }

    /// Get statistics for the branch node arena.
    pub fn branch_arena_stats(&self) -> crate::compact_arena::CompactArenaStats {
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
    /// SAFETY: Caller must ensure id is valid and allocated
    pub unsafe fn get_leaf_unchecked(&self, id: NodeId) -> &LeafNode<K, V> {
        self.leaf_arena.get_unchecked(id)
    }

    /// Unsafe fast access to branch node (no bounds checking)
    /// SAFETY: Caller must ensure id is valid and allocated
    pub unsafe fn get_branch_unchecked(&self, id: NodeId) -> &BranchNode<K, V> {
        self.branch_arena.get_unchecked(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic_operations() {
        let mut arena: Arena<String> = Arena::new();

        // Test allocation
        let id1 = arena.allocate("first".to_string());
        let id2 = arena.allocate("second".to_string());

        assert_eq!(arena.allocated_count(), 2);
        assert_eq!(arena.total_capacity(), 2);
        assert_eq!(arena.get(id1), Some(&"first".to_string()));
        assert_eq!(arena.get(id2), Some(&"second".to_string()));

        // Test deallocation
        let item = arena.deallocate(id1);
        assert_eq!(item, Some("first".to_string()));
        assert_eq!(arena.allocated_count(), 1);
        assert_eq!(arena.free_count(), 1);
        assert_eq!(arena.get(id1), None);

        // Test ID reuse
        let id3 = arena.allocate("third".to_string());
        assert_eq!(id3, id1); // Should reuse the deallocated ID
        assert_eq!(arena.get(id3), Some(&"third".to_string()));
    }

    #[test]
    fn test_arena_statistics() {
        let mut arena: Arena<i32> = Arena::new();

        // Empty arena
        assert_eq!(arena.utilization(), 0.0);
        assert_eq!(arena.fragmentation(), 0.0);
        assert!(arena.is_empty());

        // Add some items
        let _id1 = arena.allocate(1);
        let id2 = arena.allocate(2);
        let _id3 = arena.allocate(3);

        assert_eq!(arena.utilization(), 1.0); // 3/3 = 100%
        assert_eq!(arena.fragmentation(), 0.0); // No free slots
        assert!(!arena.is_empty());

        // Deallocate one item
        arena.deallocate(id2);
        assert_eq!(arena.allocated_count(), 2);
        assert_eq!(arena.free_count(), 1);
        assert!((arena.utilization() - 0.6667).abs() < 0.001); // 2/3 ≈ 66.67%
        assert!((arena.fragmentation() - 0.3333).abs() < 0.001); // 1/3 ≈ 33.33%
    }

    #[test]
    fn test_arena_iteration() {
        let mut arena: Arena<String> = Arena::new();

        let id1 = arena.allocate("first".to_string());
        let id2 = arena.allocate("second".to_string());
        let id3 = arena.allocate("third".to_string());

        // Deallocate middle item
        arena.deallocate(id2);

        // Test iteration over allocated items
        let items: Vec<_> = arena.values().collect();
        assert_eq!(items.len(), 2);
        assert!(items.contains(&&"first".to_string()));
        assert!(items.contains(&&"third".to_string()));

        // Test iteration with IDs
        let items_with_ids: Vec<_> = arena.iter().collect();
        assert_eq!(items_with_ids.len(), 2);
        assert!(items_with_ids.contains(&(id1, &"first".to_string())));
        assert!(items_with_ids.contains(&(id3, &"third".to_string())));
    }

    #[test]
    fn test_arena_validation() {
        let mut arena: Arena<i32> = Arena::new();

        // Valid arena should pass validation
        assert!(arena.validate().is_ok());

        let id1 = arena.allocate(42);
        let _id2 = arena.allocate(84);
        assert!(arena.validate().is_ok());

        arena.deallocate(id1);
        assert!(arena.validate().is_ok());

        // Test debug info
        let debug_info = arena.debug_info();
        assert_eq!(debug_info.allocated_count, 1);
        assert_eq!(debug_info.free_count, 1);
    }

    #[test]
    fn test_arena_compaction() {
        let mut arena: Arena<i32> = Arena::new();

        // Allocate several items
        let _id1 = arena.allocate(1);
        let _id2 = arena.allocate(2);
        let _id3 = arena.allocate(3);
        let _id4 = arena.allocate(4);
        let _id5 = arena.allocate(5);

        // Deallocate the last few items
        arena.deallocate(_id4);
        arena.deallocate(_id5);

        assert_eq!(arena.total_capacity(), 5);

        // Compact should reduce capacity
        arena.compact();
        assert_eq!(arena.total_capacity(), 3);
        assert_eq!(arena.allocated_count(), 3);
    }
}
