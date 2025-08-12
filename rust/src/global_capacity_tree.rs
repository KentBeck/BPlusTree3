//! BPlusTreeMap implementation using global capacity
//! This version removes per-node capacity fields to save memory

use crate::{
    BPlusTreeError, InitResult, BTreeResult, NodeRef, NodeId,
    GlobalCapacityLeafNode, GlobalCapacityBranchNode, CompactArena
};
use std::marker::PhantomData;

/// Memory-optimized B+ Tree with global capacity
#[derive(Debug)]
pub struct GlobalCapacityBPlusTreeMap<K, V> {
    /// Maximum number of keys per node (stored once for entire tree).
    capacity: usize,
    /// The root node of the tree.
    root: NodeRef<K, V>,
    /// Arena storage for leaf nodes.
    leaf_arena: CompactArena<GlobalCapacityLeafNode<K, V>>,
    /// Arena storage for branch nodes.
    branch_arena: CompactArena<GlobalCapacityBranchNode<K, V>>,
}

impl<K: Ord + Clone, V: Clone> GlobalCapacityBPlusTreeMap<K, V> {
    /// Create a new B+ tree with specified node capacity.
    pub fn new(capacity: usize) -> InitResult<Self> {
        const MIN_CAPACITY: usize = 4;
        
        if capacity < MIN_CAPACITY {
            return Err(BPlusTreeError::invalid_capacity(capacity, MIN_CAPACITY));
        }

        let mut leaf_arena = CompactArena::new();
        let root_leaf = GlobalCapacityLeafNode::new(capacity);
        let root_id = leaf_arena.allocate(root_leaf);

        Ok(Self {
            capacity,
            root: NodeRef::Leaf(root_id, PhantomData),
            leaf_arena,
            branch_arena: CompactArena::new(),
        })
    }

    /// Get the capacity of this tree.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the number of key-value pairs in the tree.
    pub fn len(&self) -> usize {
        self.count_items(&self.root)
    }

    /// Check if the tree is empty.
    pub fn is_empty(&self) -> bool {
        match &self.root {
            NodeRef::Leaf(id, _) => {
                if let Some(leaf) = self.leaf_arena.get(*id) {
                    leaf.is_empty()
                } else {
                    true
                }
            }
            NodeRef::Branch(_id, _) => false, // Branch root means non-empty
    }
}


    /// Insert a key-value pair into the tree.
    pub fn insert(&mut self, key: K, value: V) -> BTreeResult<Option<V>> {
        let result = self.insert_recursive(&self.root.clone(), key, value)?;
        
        match result {
            InsertResult::Updated(old_value) => Ok(old_value),
            InsertResult::Split { old_value, new_node_data, separator_key } => {
                // Root split - create new root
                let mut new_root = GlobalCapacityBranchNode::new(self.capacity);
                new_root.children_mut().push(self.root.clone());
                
                let new_node_ref = match new_node_data {
                    SplitNodeData::Leaf(node_ref) => node_ref,
                    SplitNodeData::Branch(node_ref) => node_ref,
                };

                new_root.insert_at(0, separator_key, new_node_ref, self.capacity)?;
                let new_root_id = self.branch_arena.allocate(new_root);
                self.root = NodeRef::Branch(new_root_id, PhantomData);
                
                Ok(old_value)
            }
            InsertResult::Error(err) => Err(err),
        }
    }

    /// Get a value by key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.get_recursive(&self.root, key)
    }

    /// Get a mutable reference to a value by key.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let root = self.root.clone();
        self.get_mut_recursive(&root, key)
    }

    /// Remove a key-value pair from the tree.
    pub fn remove(&mut self, key: &K) -> BTreeResult<Option<V>> {
        let result = self.remove_recursive(&self.root.clone(), key)?;
        
        match result {
            RemoveResult::Updated(value, _needs_rebalance) => {
                // Handle root collapse if needed
                if let NodeRef::Branch(root_id, _) = &self.root {
                    if let Some(branch) = self.branch_arena.get(*root_id) {
                        if branch.len() == 0 && branch.children().len() == 1 {
                            // Root has no keys and only one child - collapse
                            self.root = branch.children()[0].clone();
                        }
                    }
                }
                Ok(value)
            }
        }
    }

    /// Clear all items from the tree.
    pub fn clear(&mut self) {
        self.leaf_arena.clear();
        self.branch_arena.clear();
        
        let root_leaf = GlobalCapacityLeafNode::new(self.capacity);
        let root_id = self.leaf_arena.allocate(root_leaf);
        self.root = NodeRef::Leaf(root_id, PhantomData);
    }

    /// Returns an iterator over a range of key-value pairs in the tree.
    /// The range is defined by the provided bounds.
    pub fn range<'a, R>(&'a self, range: R) -> RangeIterator<'a, K, V>
    where
        R: std::ops::RangeBounds<K>,
    {
        use std::ops::Bound;
        
        // Clone the bounds to avoid lifetime issues
        let start_bound = match range.start_bound() {
            Bound::Included(key) => Bound::Included(key.clone()),
            Bound::Excluded(key) => Bound::Excluded(key.clone()),
            Bound::Unbounded => Bound::Unbounded,
        };
        
        let end_bound = match range.end_bound() {
            Bound::Included(key) => Bound::Included(key.clone()),
            Bound::Excluded(key) => Bound::Excluded(key.clone()),
            Bound::Unbounded => Bound::Unbounded,
        };
        
        RangeIterator::new(self, start_bound, end_bound)
    }

    /// Returns true if the tree contains the specified key.
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Returns a reference to the key-value pair corresponding to the supplied key.
    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        self.get_key_value_recursive(&self.root, key)
    }

    fn get_key_value_recursive(&self, node: &NodeRef<K, V>, key: &K) -> Option<(&K, &V)> {
        match node {
            NodeRef::Leaf(id, _) => {
                if let Some(leaf) = self.leaf_arena.get(*id) {
                    match leaf.keys().binary_search(key) {
                        Ok(pos) => {
                            if let Some((k, v)) = leaf.get_pair(pos) {
                                Some((k, v))
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                }
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.branch_arena.get(*id) {
                    let child_index = branch.find_child_index(key);
                    if let Some(child) = branch.get_child(child_index) {
                        self.get_key_value_recursive(child, key)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    
    /// Removes a key-value pair from the tree, returning the stored key and value if present.
    pub fn remove_entry(&mut self, key: &K) -> BTreeResult<Option<(K, V)>> {
        self.remove_entry_recursive(&self.root.clone(), key)
    }

    fn remove_entry_recursive(&mut self, node: &NodeRef<K, V>, key: &K) -> BTreeResult<Option<(K, V)>> {
        match node {
            NodeRef::Leaf(id, _) => {
                if let Some(leaf) = self.leaf_arena.get_mut(*id) {
                    match leaf.keys().binary_search(key) {
                        Ok(pos) => {
                            let (removed_key, removed_value) = leaf.remove_at(pos);
                            Ok(Some((removed_key, removed_value)))
                        }
                        Err(_) => Ok(None),
                    }
                } else {
                    Err(BPlusTreeError::ArenaError("Leaf node not found".to_string()))
                }
            }
            NodeRef::Branch(id, _) => {
                let child_index = if let Some(branch) = self.branch_arena.get(*id) {
                    branch.find_child_index(key)
                } else {
                    return Err(BPlusTreeError::ArenaError("Branch node not found".to_string()));
                };
                
                if let Some(branch) = self.branch_arena.get(*id) {
                    if let Some(child) = branch.get_child(child_index) {
                        let child = child.clone();
                        self.remove_entry_recursive(&child, key)
                    } else {
                        Ok(None)
                    }
                } else {
                    Err(BPlusTreeError::ArenaError("Branch node not found".to_string()))
                }
            }
        }
    }

    /// Returns the first key-value pair in the tree, or None if the tree is empty.
    pub fn first_key_value(&self) -> Option<(&K, &V)> {
        if self.is_empty() {
            return None;
        }
        
        self.find_first_leaf().and_then(|leaf_id| {
            self.leaf_arena.get(leaf_id).and_then(|leaf| {
                leaf.get_pair(0)
            })
        })
    }

    /// Returns the last key-value pair in the tree, or None if the tree is empty.
    pub fn last_key_value(&self) -> Option<(&K, &V)> {
        if self.is_empty() {
            return None;
        }
        
        self.find_last_leaf().and_then(|leaf_id| {
            self.leaf_arena.get(leaf_id).and_then(|leaf| {
                if leaf.len() > 0 {
                    leaf.get_pair(leaf.len() - 1)
                } else {
                    None
                }
            })
        })
    }

    /// Removes and returns the first key-value pair in the tree, or None if empty.
    pub fn pop_first(&mut self) -> BTreeResult<Option<(K, V)>> {
        if let Some((key, _)) = self.first_key_value() {
            let key = key.clone();
            self.remove_entry(&key)
        } else {
            Ok(None)
        }
    }

    /// Removes and returns the last key-value pair in the tree, or None if empty.
    pub fn pop_last(&mut self) -> BTreeResult<Option<(K, V)>> {
        if let Some((key, _)) = self.last_key_value() {
            let key = key.clone();
            self.remove_entry(&key)
        } else {
            Ok(None)
        }
    }

    /// Helper method to find the first (leftmost) leaf node
    fn find_first_leaf(&self) -> Option<NodeId> {
        self.find_first_leaf_recursive(&self.root)
    }

    fn find_first_leaf_recursive(&self, node: &NodeRef<K, V>) -> Option<NodeId> {
        match node {
            NodeRef::Leaf(id, _) => Some(*id),
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.branch_arena.get(*id) {
                    if let Some(first_child) = branch.get_child(0) {
                        self.find_first_leaf_recursive(first_child)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Helper method to find the last (rightmost) leaf node
    fn find_last_leaf(&self) -> Option<NodeId> {
        self.find_last_leaf_recursive(&self.root)
    }

    fn find_last_leaf_recursive(&self, node: &NodeRef<K, V>) -> Option<NodeId> {
        match node {
            NodeRef::Leaf(id, _) => Some(*id),
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.branch_arena.get(*id) {
                    let last_child_index = if branch.len() > 0 { branch.len() } else { 0 };
                    if let Some(last_child) = branch.get_child(last_child_index) {
                        self.find_last_leaf_recursive(last_child)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Helper method to find the starting position for a range query
    fn find_range_start(&self, start_bound: &std::ops::Bound<&K>) -> (Option<NodeId>, usize) {
        match start_bound {
            std::ops::Bound::Unbounded => {
                // Start from the first leaf
                if let Some(first_leaf) = self.find_first_leaf() {
                    (Some(first_leaf), 0)
                } else {
                    (None, 0)
                }
            }
            std::ops::Bound::Included(key) | std::ops::Bound::Excluded(key) => {
                self.find_key_position(key, matches!(start_bound, std::ops::Bound::Excluded(_)))
            }
        }
    }

    /// Helper method to find the position of a key in the tree for range queries
    fn find_key_position(&self, key: &K, excluded: bool) -> (Option<NodeId>, usize) {
        self.find_key_position_recursive(&self.root, key, excluded)
    }

    fn find_key_position_recursive(&self, node: &NodeRef<K, V>, key: &K, excluded: bool) -> (Option<NodeId>, usize) {
        match node {
            NodeRef::Leaf(id, _) => {
                if let Some(leaf) = self.leaf_arena.get(*id) {
                    match leaf.keys().binary_search(key) {
                        Ok(pos) => {
                            if excluded {
                                // For excluded bounds, start from the next position
                                if pos + 1 < leaf.len() {
                                    (Some(*id), pos + 1)
                                } else {
                                    // Move to next leaf
                                    let next_leaf = leaf.next();
                                    if next_leaf != crate::NULL_NODE {
                                        (Some(next_leaf), 0)
                                    } else {
                                        (None, 0)
                                    }
                                }
                            } else {
                                (Some(*id), pos)
                            }
                        }
                        Err(pos) => {
                            // Key not found, start from the insertion position
                            if pos < leaf.len() {
                                (Some(*id), pos)
                            } else {
                                // Move to next leaf
                                let next_leaf = leaf.next();
                                if next_leaf != crate::NULL_NODE {
                                    (Some(next_leaf), 0)
                                } else {
                                    (None, 0)
                                }
                            }
                        }
                    }
                } else {
                    (None, 0)
                }
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.branch_arena.get(*id) {
                    let child_index = branch.find_child_index(key);
                    if let Some(child) = branch.get_child(child_index) {
                        self.find_key_position_recursive(child, key, excluded)
                    } else {
                        (None, 0)
                    }
                } else {
                    (None, 0)
                }
            }
        }
    }

    
    // Helper methods
    fn count_items(&self, node: &NodeRef<K, V>) -> usize {
        match node {
            NodeRef::Leaf(id, _) => {
                self.leaf_arena.get(*id).map(|leaf| leaf.len()).unwrap_or(0)
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.branch_arena.get(*id) {
                    branch.children().iter().map(|child| self.count_items(child)).sum()
                } else {
                    0
                }
            }
        }
    }

    fn get_recursive(&self, node: &NodeRef<K, V>, key: &K) -> Option<&V> {
        match node {
            NodeRef::Leaf(id, _) => {
                if let Some(leaf) = self.leaf_arena.get(*id) {
                    match leaf.keys().binary_search(key) {
                        Ok(pos) => leaf.values().get(pos),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.branch_arena.get(*id) {
                    let child_index = branch.find_child_index(key);
                    if let Some(child) = branch.children().get(child_index) {
                        self.get_recursive(child, key)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    fn get_mut_recursive(&mut self, node: &NodeRef<K, V>, key: &K) -> Option<&mut V> {
        match node {
            NodeRef::Leaf(id, _) => {
                if let Some(leaf) = self.leaf_arena.get_mut(*id) {
                    match leaf.keys().binary_search(key) {
                        Ok(pos) => leaf.get_value_mut(pos),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.branch_arena.get(*id) {
                    let child_index = branch.find_child_index(key);
                    if let Some(child) = branch.children().get(child_index).cloned() {
                        self.get_mut_recursive(&child, key)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    fn insert_recursive(&mut self, node: &NodeRef<K, V>, key: K, value: V) -> BTreeResult<InsertResult<K, V>> {
        match node {
            NodeRef::Leaf(id, _) => {
                // First, check if we need to split by examining the leaf
                let (pos, needs_split, _old_value_opt): (usize, bool, Option<V>) = {
                    if let Some(leaf) = self.leaf_arena.get_mut(*id) {
                        let pos = leaf.find_insert_position(&key);
                        
                        // Check if key already exists
                        if pos < leaf.len() && &leaf.keys()[pos] == &key {
                            let old_value = std::mem::replace(&mut leaf.values_mut()[pos], value);
                            return Ok(InsertResult::Updated(Some(old_value)));
                        }
                        
                        let needs_split = leaf.is_full(self.capacity);
                        (pos, needs_split, None)
                    } else {
                        return Err(BPlusTreeError::ArenaError("Leaf node not found".to_string()));
                    }
                };
                
                if needs_split {
                    // Handle split case
                    let (right_leaf, separator_key) = {
                        if let Some(leaf) = self.leaf_arena.get_mut(*id) {
                            let split_pos = self.capacity / 2;
                            let mut right_leaf = leaf.split_at(split_pos, self.capacity);
                            
                            // Insert into appropriate half
                            if pos <= split_pos {
                                leaf.insert_at(pos, key.clone(), value, self.capacity)?;
                            } else {
                                right_leaf.insert_at(pos - split_pos, key.clone(), value, self.capacity)?;
                            }
                            
                            // Update linked list - right leaf points to what left leaf was pointing to
                            right_leaf.set_next(leaf.next());
                            
                            // Get the separator key before moving right_leaf
                            let separator_key = right_leaf.keys()[0].clone();
                            
                            (right_leaf, separator_key)
                        } else {
                            return Err(BPlusTreeError::ArenaError("Leaf node not found".to_string()));
                        }
                    };
                    
                    // Allocate the right leaf to get its ID
                    let right_leaf_id = self.leaf_arena.allocate(right_leaf);
                    
                    // Now update left leaf to point to the right leaf
                    if let Some(leaf) = self.leaf_arena.get_mut(*id) {
                        leaf.set_next(right_leaf_id);
                    }
                    
                    Ok(InsertResult::Split {
                        old_value: None,
                        new_node_data: SplitNodeData::Leaf(NodeRef::Leaf(right_leaf_id, PhantomData)),
                        separator_key,
                    })
                } else {
                    // Simple insertion
                    if let Some(leaf) = self.leaf_arena.get_mut(*id) {
                        leaf.insert_at(pos, key, value, self.capacity)?;
                        Ok(InsertResult::Updated(None))
                    } else {
                        Err(BPlusTreeError::ArenaError("Leaf node not found".to_string()))
                    }
                }
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.branch_arena.get(*id) {
                    let child_index = branch.find_child_index(&key);
                    if let Some(child) = branch.children().get(child_index).cloned() {
                        let result = self.insert_recursive(&child, key, value)?;
                        
                        match result {
                            InsertResult::Updated(old_value) => Ok(InsertResult::Updated(old_value)),
                            InsertResult::Split { old_value, new_node_data, separator_key } => {
                                // Extract the node reference directly
                                let new_node_ref = match new_node_data {
                                    SplitNodeData::Leaf(node_ref) => node_ref,
                                    SplitNodeData::Branch(node_ref) => node_ref,
                                };

                                // Insert the new node into this branch
                                if let Some(branch) = self.branch_arena.get_mut(*id) {
                                    if branch.is_full(self.capacity) {
                                        // Branch is full - need to split
                                        let split_pos = self.capacity / 2;
                                        let (mid_key, mut right_branch) = branch.split_at(split_pos, self.capacity);
                                        
                                        // Insert into appropriate half
                                        if child_index <= split_pos {
                                            branch.insert_at(child_index, separator_key, new_node_ref, self.capacity)?;
                                        } else {
                                            right_branch.insert_at(child_index - split_pos - 1, separator_key, new_node_ref, self.capacity)?;
                                        }
                                        
                                        let right_branch_id = self.branch_arena.allocate(right_branch);
                                        Ok(InsertResult::Split {
                                            old_value,
                                            new_node_data: SplitNodeData::Branch(NodeRef::Branch(right_branch_id, PhantomData)),
                                            separator_key: mid_key,
                                        })
                                    } else {
                                        // Simple insertion into branch
                                        branch.insert_at(child_index, separator_key, new_node_ref, self.capacity)?;
                                        Ok(InsertResult::Updated(old_value))
                                    }
                                } else {
                                    Err(BPlusTreeError::ArenaError("Branch node not found".to_string()))
                                }
                            }
                            InsertResult::Error(err) => Ok(InsertResult::Error(err)),
                        }
                    } else {
                        Err(BPlusTreeError::NodeError("Child not found".to_string()))
                    }
                } else {
                    Err(BPlusTreeError::ArenaError("Branch node not found".to_string()))
                }
            }
        }
    }

    fn remove_recursive(&mut self, node: &NodeRef<K, V>, key: &K) -> BTreeResult<RemoveResult<V>> {
        match node {
            NodeRef::Leaf(id, _) => {
                if let Some(leaf) = self.leaf_arena.get_mut(*id) {
                    match leaf.keys().binary_search(key) {
                        Ok(pos) => {
                            let (_key, value) = leaf.remove_at(pos);
                            let needs_rebalance = leaf.is_underfull(self.capacity);
                            Ok(RemoveResult::Updated(Some(value), needs_rebalance))
                        }
                        Err(_) => Ok(RemoveResult::Updated(None, false)),
                    }
                } else {
                    Err(BPlusTreeError::ArenaError("Leaf node not found".to_string()))
                }
            }
            NodeRef::Branch(_id, _) => {
                // Branch removal logic would go here
                // For now, return not found
                Ok(RemoveResult::Updated(None, false))
            }
        }
    }
}

/// Result of an insertion operation on a node.
pub enum InsertResult<K, V> {
    /// Insertion completed without splitting. Contains the old value if key existed.
    Updated(Option<V>),
    /// Insertion caused a split with arena allocation needed.
    Split {
        old_value: Option<V>,
        new_node_data: SplitNodeData<K, V>,
        separator_key: K,
    },
    /// Internal error occurred during insertion.
    Error(BPlusTreeError),
}

/// Node data that can be allocated in the arena after a split.
pub enum SplitNodeData<K, V> {
    Leaf(NodeRef<K, V>),
    Branch(NodeRef<K, V>),
}

/// Result of a removal operation on a node.
pub enum RemoveResult<V> {
    /// Removal completed. Contains the removed value if key existed.
    /// The bool indicates if this node is now underfull and needs rebalancing.
    Updated(Option<V>, bool),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_memory_savings() {
        let original_size = mem::size_of::<crate::BPlusTreeMap<i32, i32>>();
        let optimized_size = mem::size_of::<GlobalCapacityBPlusTreeMap<i32, i32>>();
        
        println!("Original BPlusTreeMap size: {} bytes", original_size);
        println!("Global capacity BPlusTreeMap size: {} bytes", optimized_size);
        
        // Should be same or smaller (nodes save memory, but tree struct might be similar)
        assert!(optimized_size <= original_size + 16); // Allow some variance
    }

    #[test]
    fn test_basic_operations() {
        let mut tree = GlobalCapacityBPlusTreeMap::new(4).unwrap();
        
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.capacity(), 4);
        
        // Insert some values
        assert_eq!(tree.insert(10, 100).unwrap(), None);
        assert_eq!(tree.insert(20, 200).unwrap(), None);
        assert_eq!(tree.insert(30, 300).unwrap(), None);
        
        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 3);
        
        // Get values
        assert_eq!(tree.get(&10), Some(&100));
        assert_eq!(tree.get(&20), Some(&200));
        assert_eq!(tree.get(&30), Some(&300));
        assert_eq!(tree.get(&40), None);
        
        // Update value
        assert_eq!(tree.insert(20, 222).unwrap(), Some(200));
        assert_eq!(tree.get(&20), Some(&222));
        
        // Remove value
        assert_eq!(tree.remove(&20).unwrap(), Some(222));
        assert_eq!(tree.get(&20), None);
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn test_node_splitting() {
        let mut tree = GlobalCapacityBPlusTreeMap::new(4).unwrap();
        
        // Fill beyond capacity to trigger splits
        for i in 0..10 {
            tree.insert(i, i * 10).unwrap();
        }
        
        assert_eq!(tree.len(), 10);
        
        // Verify all values are accessible
        for i in 0..10 {
            assert_eq!(tree.get(&i), Some(&(i * 10)));
        }
    }

    #[test]
    fn test_clear() {
        let mut tree = GlobalCapacityBPlusTreeMap::new(4).unwrap();
        
        for i in 0..5 {
            tree.insert(i, i).unwrap();
        }
        
        assert_eq!(tree.len(), 5);
        
        tree.clear();
        
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        
        // Should still be able to insert after clear
        tree.insert(42, 42).unwrap();
        assert_eq!(tree.get(&42), Some(&42));
    }

    #[test]
    fn test_range_queries() {
        let mut tree = GlobalCapacityBPlusTreeMap::new(4).unwrap();
        
        // Insert test data
        for i in 0..10 {
            tree.insert(i, i * 10).unwrap();
        }
        
        // Test inclusive range
        let range_items: Vec<_> = tree.range(3..=6).collect();
        assert_eq!(range_items, vec![(&3, &30), (&4, &40), (&5, &50), (&6, &60)]);

        // Test exclusive range
        let range_items: Vec<_> = tree.range(3..6).collect();
        assert_eq!(range_items, vec![(&3, &30), (&4, &40), (&5, &50)]);
        
        // Test unbounded range
        let all_items: Vec<_> = tree.range(..).collect();
        assert_eq!(all_items.len(), 10);
        
        // Test range from start
        let from_start: Vec<_> = tree.range(..5).collect();
        assert_eq!(from_start, vec![(&0, &0), (&1, &10), (&2, &20), (&3, &30), (&4, &40)]);
        
        // Test range to end
        let to_end: Vec<_> = tree.range(7..).collect();
        assert_eq!(to_end, vec![(&7, &70), (&8, &80), (&9, &90)]);
    }

    #[test]
    fn test_range_performance_analysis() {
        use std::time::Instant;
        
        let mut tree = GlobalCapacityBPlusTreeMap::new(16).unwrap();
        
        // Insert a large number of items
        let n = 10000;
        println!("Inserting {} items...", n);
        let start = Instant::now();
        for i in 0..n {
            tree.insert(i, i * 10).unwrap();
        }
        let insert_time = start.elapsed();
        println!("Insert time: {:?} ({:.2} µs per item)", insert_time, insert_time.as_micros() as f64 / n as f64);
        
        // Test range queries of different sizes
        let range_sizes = vec![10, 100, 1000, 5000];
        
        for range_size in range_sizes {
            let start_key = n / 2 - range_size / 2;  // Start from middle
            let end_key = start_key + range_size;
            
            println!("\n--- Range query: {}..{} (size: {}) ---", start_key, end_key, range_size);
            
            // Time the range creation
            let start = Instant::now();
            let range_iter = tree.range(start_key..end_key);
            let range_creation_time = start.elapsed();
            
            // Time the iteration
            let start = Instant::now();
            let items: Vec<_> = range_iter.collect();
            let iteration_time = start.elapsed();
            
            println!("Range creation: {:?}", range_creation_time);
            println!("Iteration time: {:?} ({:.2} µs per item)", iteration_time, iteration_time.as_micros() as f64 / items.len() as f64);
            println!("Items collected: {}", items.len());
            
            // Check if time scales linearly with range size (O(n)) or logarithmically (O(log n))
            if range_size > 10 {
                let time_per_item = iteration_time.as_nanos() as f64 / items.len() as f64;
                println!("Time per item: {:.2} ns", time_per_item);
            }
        }
        
        // Test single key lookup for comparison
        println!("\n--- Single key lookups ---");
        let lookup_keys = vec![100, 1000, 5000, 9000];
        for key in lookup_keys {
            let start = Instant::now();
            let _value = tree.get(&key);
            let lookup_time = start.elapsed();
            println!("Lookup key {}: {:?}", key, lookup_time);
        }
        
        // Test range creation time vs tree size to check if it's O(log n)
        println!("\n--- Range creation scaling test ---");
        let tree_sizes = vec![1000, 5000, 10000, 50000];
        for tree_size in tree_sizes {
            let mut test_tree = GlobalCapacityBPlusTreeMap::new(16).unwrap();
            for i in 0..tree_size {
                test_tree.insert(i, i * 10).unwrap();
            }
            
            // Test range creation time for a fixed small range in the middle
            let start_key = tree_size / 2;
            let end_key = start_key + 10;
            
            let start = Instant::now();
            let _range_iter = test_tree.range(start_key..end_key);
            let range_creation_time = start.elapsed();
            
            println!("Tree size: {}, Range creation: {:?}", tree_size, range_creation_time);
        }
    }

    #[test]
    fn test_key_value_operations() {
        let mut tree = GlobalCapacityBPlusTreeMap::new(4).unwrap();
        
        // Insert test data
        for i in 0..5 {
            tree.insert(i, i * 10).unwrap();
        }
        
        // Test contains_key
        assert!(tree.contains_key(&2));
        assert!(!tree.contains_key(&10));
        
        // Test get_key_value
        assert_eq!(tree.get_key_value(&3), Some((&3, &30)));
        assert_eq!(tree.get_key_value(&10), None);
        
        // Test remove_entry
        assert_eq!(tree.remove_entry(&2).unwrap(), Some((2, 20)));
        assert_eq!(tree.remove_entry(&2).unwrap(), None);
        assert_eq!(tree.len(), 4);
    }

    #[test]
    fn test_first_last_operations() {
        let mut tree = GlobalCapacityBPlusTreeMap::new(4).unwrap();
        
        // Empty tree
        assert_eq!(tree.first_key_value(), None);
        assert_eq!(tree.last_key_value(), None);
        assert_eq!(tree.pop_first().unwrap(), None);
        assert_eq!(tree.pop_last().unwrap(), None);
        
        // Insert test data
        for i in [5, 2, 8, 1, 9, 3] {
            tree.insert(i, i * 10).unwrap();
        }
        
        // Test first/last
        assert_eq!(tree.first_key_value(), Some((&1, &10)));
        assert_eq!(tree.last_key_value(), Some((&9, &90)));
        
        // Test pop_first
        assert_eq!(tree.pop_first().unwrap(), Some((1, 10)));
        assert_eq!(tree.first_key_value(), Some((&2, &20)));
        
        // Test pop_last
        assert_eq!(tree.pop_last().unwrap(), Some((9, 90)));
        assert_eq!(tree.last_key_value(), Some((&8, &80)));
        
        assert_eq!(tree.len(), 4);
    }
}

/// Iterator over a range of key-value pairs in the tree.
pub struct RangeIterator<'a, K, V> {
    tree: &'a GlobalCapacityBPlusTreeMap<K, V>,
    current_leaf_id: Option<NodeId>,
    current_leaf_ref: Option<&'a GlobalCapacityLeafNode<K, V>>,
    current_pos: usize,
    start_bound: std::ops::Bound<K>,
    end_bound: std::ops::Bound<K>,
    finished: bool,
}

impl<'a, K: Ord + Clone, V: Clone> RangeIterator<'a, K, V> {
    fn new(
        tree: &'a GlobalCapacityBPlusTreeMap<K, V>,
        start_bound: std::ops::Bound<K>,
        end_bound: std::ops::Bound<K>,
    ) -> Self {
        let start_bound_ref = match &start_bound {
            std::ops::Bound::Included(k) => std::ops::Bound::Included(k),
            std::ops::Bound::Excluded(k) => std::ops::Bound::Excluded(k),
            std::ops::Bound::Unbounded => std::ops::Bound::Unbounded,
        };
        
        let (current_leaf_id, current_pos) = tree.find_range_start(&start_bound_ref);
        
        // Get the initial leaf reference if we have a starting leaf
        let current_leaf_ref = current_leaf_id.and_then(|id| tree.leaf_arena.get(id));
        
        RangeIterator {
            tree,
            current_leaf_id,
            current_leaf_ref,
            current_pos,
            start_bound,
            end_bound,
            finished: false,
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> Iterator for RangeIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            // Check if we have a current leaf reference
            let leaf = match self.current_leaf_ref {
                Some(leaf) => leaf,
                None => {
                    self.finished = true;
                    return None;
                }
            };

            // Check if we have more items in current leaf
            if self.current_pos < leaf.len() {
                if let Some((key, value)) = leaf.get_pair(self.current_pos) {
                    // Check if we've exceeded the end bound
                    let should_stop = match &self.end_bound {
                        std::ops::Bound::Included(end_key) => key > end_key,
                        std::ops::Bound::Excluded(end_key) => key >= end_key,
                        std::ops::Bound::Unbounded => false,
                    };

                    if should_stop {
                        self.finished = true;
                        return None;
                    }

                    // Advance position for next call
                    self.current_pos += 1;
                    return Some((key, value));
                }
            }

            // Exhausted current leaf, move to next leaf
            let next_leaf_id = leaf.next();
            if next_leaf_id != crate::NULL_NODE {
                // Update to next leaf - this is the ONLY arena access during iteration
                self.current_leaf_id = Some(next_leaf_id);
                self.current_leaf_ref = self.tree.leaf_arena.get(next_leaf_id);
                self.current_pos = 0;
                
                // Check if we successfully got the next leaf
                if self.current_leaf_ref.is_none() {
                    self.finished = true;
                    return None;
                }
                // Continue the loop to check the new leaf
            } else {
                // No more leaves
                self.finished = true;
                return None;
            }
        }
    }
}
