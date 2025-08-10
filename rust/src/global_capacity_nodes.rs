//! Node structures without per-node capacity fields
//! This module implements memory-optimized nodes that rely on global capacity

use crate::{NodeId, NULL_NODE, BPlusTreeError, NodeRef};

/// Leaf node without capacity field - saves 8 bytes per node
#[derive(Debug, Clone)]
pub struct GlobalCapacityLeafNode<K, V> {
    /// Sorted list of keys.
    keys: Vec<K>,
    /// List of values corresponding to keys.
    values: Vec<V>,
    /// Next leaf node in the linked list (for range queries).
    next: NodeId,
}

/// Branch node without capacity field - saves 8 bytes per node
#[derive(Debug, Clone)]
pub struct GlobalCapacityBranchNode<K, V> {
    /// Sorted list of separator keys.
    keys: Vec<K>,
    /// List of child nodes (leaves or other branches).
    children: Vec<NodeRef<K, V>>,
}

impl<K: Ord + Clone, V: Clone> GlobalCapacityLeafNode<K, V> {
    /// Creates a new leaf node with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            next: NULL_NODE,
        }
    }

    /// Creates a new leaf node with pre-allocated data.
    pub fn with_data(keys: Vec<K>, values: Vec<V>, next: NodeId) -> Self {
        debug_assert_eq!(keys.len(), values.len());
        Self { keys, values, next }
    }

    /// Get a reference to the keys in this leaf node.
    pub fn keys(&self) -> &Vec<K> {
        &self.keys
    }

    /// Get a reference to the values in this leaf node.
    pub fn values(&self) -> &Vec<V> {
        &self.values
    }

    /// Get the next node ID in the linked list.
    pub fn next(&self) -> NodeId {
        self.next
    }

    /// Set the next node ID in the linked list.
    pub fn set_next(&mut self, next: NodeId) {
        self.next = next;
    }

    /// Get the number of key-value pairs in this node.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Check if the node is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Check if the node is full given the tree's capacity.
    pub fn is_full(&self, capacity: usize) -> bool {
        self.keys.len() >= capacity
    }

    /// Check if the node can accept more items given the tree's capacity.
    pub fn can_insert(&self, capacity: usize) -> bool {
        self.keys.len() < capacity
    }

    /// Check if the node is underfull (less than half capacity).
    pub fn is_underfull(&self, capacity: usize) -> bool {
        self.keys.len() < capacity / 2
    }

    /// Find the position where a key should be inserted.
    pub fn find_insert_position(&self, key: &K) -> usize {
        match self.keys.binary_search(key) {
            Ok(pos) => pos,
            Err(pos) => pos,
        }
    }

    /// Insert a key-value pair at the specified position.
    pub fn insert_at(&mut self, pos: usize, key: K, value: V, capacity: usize) -> Result<(), BPlusTreeError> {
        if self.is_full(capacity) {
            return Err(BPlusTreeError::NodeError("Node is full".to_string()));
        }

        self.keys.insert(pos, key);
        self.values.insert(pos, value);
        Ok(())
    }

    /// Remove a key-value pair at the specified position.
    pub fn remove_at(&mut self, pos: usize) -> (K, V) {
        let key = self.keys.remove(pos);
        let value = self.values.remove(pos);
        (key, value)
    }

    /// Get a key-value pair by index.
    pub fn get_pair(&self, index: usize) -> Option<(&K, &V)> {
        if index < self.keys.len() {
            Some((&self.keys[index], &self.values[index]))
        } else {
            None
        }
    }

    /// Get a mutable reference to a value by index.
    pub fn get_value_mut(&mut self, index: usize) -> Option<&mut V> {
        self.values.get_mut(index)
    }

    /// Get a mutable reference to the values vector.
    pub fn values_mut(&mut self) -> &mut Vec<V> {
        &mut self.values
    }

    /// Split this node at the given position, returning the right half.
    pub fn split_at(&mut self, split_pos: usize, capacity: usize) -> Self {
        let right_keys = self.keys.split_off(split_pos);
        let right_values = self.values.split_off(split_pos);
        
        // Reserve capacity for future insertions
        self.keys.reserve(capacity - self.keys.len());
        self.values.reserve(capacity - self.values.len());
        
        Self::with_data(right_keys, right_values, self.next)
    }

    /// Merge another leaf node into this one.
    pub fn merge_with(&mut self, other: Self, capacity: usize) -> Result<(), BPlusTreeError> {
        if self.keys.len() + other.keys.len() > capacity {
            return Err(BPlusTreeError::NodeError("Merge would exceed capacity".to_string()));
        }

        self.keys.extend(other.keys);
        self.values.extend(other.values);
        self.next = other.next;
        Ok(())
    }

    /// Borrow elements from another leaf node.
    pub fn borrow_from_left(&mut self, left: &mut Self, separator_key: &mut K) {
        if let (Some(key), Some(value)) = (left.keys.pop(), left.values.pop()) {
            self.keys.insert(0, key.clone());
            self.values.insert(0, value);
            *separator_key = key;
        }
    }

    /// Borrow elements from another leaf node.
    pub fn borrow_from_right(&mut self, right: &mut Self, separator_key: &mut K) {
        if !right.keys.is_empty() {
            let key = right.keys.remove(0);
            let value = right.values.remove(0);
            self.keys.push(key);
            self.values.push(value);
            if !right.keys.is_empty() {
                *separator_key = right.keys[0].clone();
            }
        }
    }
}

impl<K: Ord + Clone, V: Clone> GlobalCapacityBranchNode<K, V> {
    /// Creates a new branch node with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            keys: Vec::with_capacity(capacity),
            children: Vec::with_capacity(capacity + 1),
        }
    }

    /// Creates a new branch node with pre-allocated data.
    pub fn with_data(keys: Vec<K>, children: Vec<NodeRef<K, V>>) -> Self {
        debug_assert_eq!(keys.len() + 1, children.len());
        Self { keys, children }
    }

    /// Get a reference to the keys in this branch node.
    pub fn keys(&self) -> &Vec<K> {
        &self.keys
    }

    /// Get a reference to the children in this branch node.
    pub fn children(&self) -> &Vec<NodeRef<K, V>> {
        &self.children
    }

    /// Get a mutable reference to the children.
    pub fn children_mut(&mut self) -> &mut Vec<NodeRef<K, V>> {
        &mut self.children
    }

    /// Get the number of keys in this node.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Check if the node is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Check if the node is full given the tree's capacity.
    pub fn is_full(&self, capacity: usize) -> bool {
        self.keys.len() >= capacity
    }

    /// Check if the node can accept more items given the tree's capacity.
    pub fn can_insert(&self, capacity: usize) -> bool {
        self.keys.len() < capacity
    }

    /// Check if the node is underfull (less than half capacity).
    pub fn is_underfull(&self, capacity: usize) -> bool {
        self.keys.len() < capacity / 2
    }

    /// Find the child index for a given key.
    pub fn find_child_index(&self, key: &K) -> usize {
        match self.keys.binary_search(key) {
            Ok(pos) => pos + 1,
            Err(pos) => pos,
        }
    }

    /// Get a child at the specified index.
    pub fn get_child(&self, index: usize) -> Option<&NodeRef<K, V>> {
        self.children.get(index)
    }

    /// Insert a key and child at the specified position.
    pub fn insert_at(&mut self, pos: usize, key: K, child: NodeRef<K, V>, capacity: usize) -> Result<(), BPlusTreeError> {
        if self.is_full(capacity) {
            return Err(BPlusTreeError::NodeError("Node is full".to_string()));
        }

        self.keys.insert(pos, key);
        self.children.insert(pos + 1, child);
        Ok(())
    }

    /// Remove a key and child at the specified position.
    pub fn remove_at(&mut self, pos: usize) -> (K, NodeRef<K, V>) {
        let key = self.keys.remove(pos);
        let child = self.children.remove(pos + 1);
        (key, child)
    }

    /// Split this node at the given position, returning the right half and separator key.
    pub fn split_at(&mut self, split_pos: usize, capacity: usize) -> (K, Self) {
        let separator_key = self.keys.remove(split_pos);
        let right_keys = self.keys.split_off(split_pos);
        let right_children = self.children.split_off(split_pos + 1);
        
        // Reserve capacity for future insertions
        self.keys.reserve(capacity - self.keys.len());
        self.children.reserve(capacity + 1 - self.children.len());
        
        let right_node = Self::with_data(right_keys, right_children);
        (separator_key, right_node)
    }

    /// Merge another branch node into this one with a separator key.
    pub fn merge_with(&mut self, separator: K, other: Self, capacity: usize) -> Result<(), BPlusTreeError> {
        if self.keys.len() + 1 + other.keys.len() > capacity {
            return Err(BPlusTreeError::NodeError("Merge would exceed capacity".to_string()));
        }

        self.keys.push(separator);
        self.keys.extend(other.keys);
        self.children.extend(other.children);
        Ok(())
    }

    /// Borrow elements from left sibling.
    pub fn borrow_from_left(&mut self, left: &mut Self, separator_key: &mut K) {
        if let (Some(key), Some(child)) = (left.keys.pop(), left.children.pop()) {
            self.keys.insert(0, separator_key.clone());
            self.children.insert(0, child);
            *separator_key = key;
        }
    }

    /// Borrow elements from right sibling.
    pub fn borrow_from_right(&mut self, right: &mut Self, separator_key: &mut K) {
        if !right.keys.is_empty() {
            let key = right.keys.remove(0);
            let child = right.children.remove(0);
            self.keys.push(separator_key.clone());
            self.children.push(child);
            *separator_key = key;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaf_node_memory_savings() {
        use std::mem;
        
        // Compare sizes
        let old_size = mem::size_of::<crate::LeafNode<i32, i32>>();
        let new_size = mem::size_of::<GlobalCapacityLeafNode<i32, i32>>();
        
        println!("Old LeafNode size: {} bytes", old_size);
        println!("New LeafNode size: {} bytes", new_size);
        println!("Savings: {} bytes ({:.1}%)", 
                 old_size - new_size,
                 (old_size - new_size) as f64 / old_size as f64 * 100.0);
        
        assert!(new_size < old_size);
        assert_eq!(old_size - new_size, 8); // Should save exactly 8 bytes
    }

    #[test]
    fn test_branch_node_memory_savings() {
        use std::mem;
        
        // Compare sizes
        let old_size = mem::size_of::<crate::BranchNode<i32, i32>>();
        let new_size = mem::size_of::<GlobalCapacityBranchNode<i32, i32>>();
        
        println!("Old BranchNode size: {} bytes", old_size);
        println!("New BranchNode size: {} bytes", new_size);
        println!("Savings: {} bytes ({:.1}%)", 
                 old_size - new_size,
                 (old_size - new_size) as f64 / old_size as f64 * 100.0);
        
        assert!(new_size < old_size);
        assert_eq!(old_size - new_size, 8); // Should save exactly 8 bytes
    }

    #[test]
    fn test_leaf_node_operations() {
        let capacity = 4;
        let mut node = GlobalCapacityLeafNode::new(capacity);
        
        // Test insertion
        assert!(node.can_insert(capacity));
        assert!(!node.is_full(capacity));
        
        node.insert_at(0, 10, 100, capacity).unwrap();
        node.insert_at(1, 20, 200, capacity).unwrap();
        
        assert_eq!(node.len(), 2);
        assert_eq!(node.get_pair(0), Some((&10, &100)));
        assert_eq!(node.get_pair(1), Some((&20, &200)));
        
        // Test capacity limits
        node.insert_at(2, 30, 300, capacity).unwrap();
        node.insert_at(3, 40, 400, capacity).unwrap();
        
        assert!(node.is_full(capacity));
        assert!(!node.can_insert(capacity));
        
        // Should fail to insert when full
        assert!(node.insert_at(4, 50, 500, capacity).is_err());
    }

    #[test]
    fn test_branch_node_operations() {
        let capacity = 4;
        let mut node = GlobalCapacityBranchNode::<i32, i32>::new(capacity);
        
        // Add initial child
        node.children.push(NodeRef::Leaf(1, std::marker::PhantomData));
        
        // Test insertion
        assert!(node.can_insert(capacity));
        assert!(!node.is_full(capacity));
        
        node.insert_at(0, 10, NodeRef::Leaf(2, std::marker::PhantomData), capacity).unwrap();
        node.insert_at(1, 20, NodeRef::Leaf(3, std::marker::PhantomData), capacity).unwrap();
        
        assert_eq!(node.len(), 2);
        assert_eq!(node.children.len(), 3);
        
        // Test capacity limits
        node.insert_at(2, 30, NodeRef::Leaf(4, std::marker::PhantomData), capacity).unwrap();
        node.insert_at(3, 40, NodeRef::Leaf(5, std::marker::PhantomData), capacity).unwrap();
        
        assert!(node.is_full(capacity));
        assert!(!node.can_insert(capacity));
    }

    #[test]
    fn test_leaf_node_split() {
        let capacity = 4;
        let mut node = GlobalCapacityLeafNode::new(capacity);
        
        // Fill the node
        for i in 0..4 {
            node.insert_at(i, i * 10, i * 100, capacity).unwrap();
        }
        
        // Split at position 2
        let right = node.split_at(2, capacity);
        
        assert_eq!(node.len(), 2);
        assert_eq!(right.len(), 2);
        
        assert_eq!(node.get_pair(0), Some((&0, &0)));
        assert_eq!(node.get_pair(1), Some((&10, &100)));
        
        assert_eq!(right.get_pair(0), Some((&20, &200)));
        assert_eq!(right.get_pair(1), Some((&30, &300)));
    }
}
