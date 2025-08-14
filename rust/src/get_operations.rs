//! GET operations for BPlusTreeMap.
//!
//! This module contains all the read operations for the B+ tree, including
//! key lookup, value retrieval, and helper methods for accessing nodes.

use crate::error::{BPlusTreeError, BTreeResult, KeyResult};
use crate::types::{BPlusTreeMap, NodeRef, LeafNode, BranchNode, NodeId, NULL_NODE};

impl<K: Ord + Clone, V: Clone> BPlusTreeMap<K, V> {
    // ============================================================================
    // PUBLIC GET OPERATIONS
    // ============================================================================

    /// Get a reference to the value associated with a key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// A reference to the value if the key exists, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BPlusTreeMap;
    ///
    /// let mut tree = BPlusTreeMap::new(16).unwrap();
    /// tree.insert(1, "one");
    /// assert_eq!(tree.get(&1), Some(&"one"));
    /// assert_eq!(tree.get(&2), None);
    /// ```
    pub fn get(&self, key: &K) -> Option<&V> {
        let node = &self.root;
        self.get_recursive(node, key)
    }

    /// Check if key exists in the tree.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check for existence
    ///
    /// # Returns
    ///
    /// `true` if the key exists, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BPlusTreeMap;
    ///
    /// let mut tree = BPlusTreeMap::new(16).unwrap();
    /// tree.insert(1, "one");
    /// assert!(tree.contains_key(&1));
    /// assert!(!tree.contains_key(&2));
    /// ```
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Get value for a key with default.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    /// * `default` - The default value to return if key is not found
    ///
    /// # Returns
    ///
    /// A reference to the value if the key exists, or the default value.
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BPlusTreeMap;
    ///
    /// let mut tree = BPlusTreeMap::new(16).unwrap();
    /// tree.insert(1, "one");
    /// assert_eq!(tree.get_or_default(&1, &"default"), &"one");
    /// assert_eq!(tree.get_or_default(&2, &"default"), &"default");
    /// ```
    pub fn get_or_default<'a>(&'a self, key: &K, default: &'a V) -> &'a V {
        self.get(key).unwrap_or(default)
    }

    /// Get value for a key, returning an error if the key doesn't exist.
    /// This is equivalent to Python's `tree[key]`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// A reference to the value if the key exists, or a `KeyNotFound` error.
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BPlusTreeMap;
    ///
    /// let mut tree = BPlusTreeMap::new(16).unwrap();
    /// tree.insert(1, "one");
    /// assert_eq!(tree.get_item(&1).unwrap(), &"one");
    /// assert!(tree.get_item(&2).is_err());
    /// ```
    pub fn get_item(&self, key: &K) -> KeyResult<&V> {
        self.get(key).ok_or(BPlusTreeError::KeyNotFound)
    }

    /// Get a mutable reference to the value for a key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// A mutable reference to the value if the key exists, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BPlusTreeMap;
    ///
    /// let mut tree = BPlusTreeMap::new(16).unwrap();
    /// tree.insert(1, "one");
    /// if let Some(value) = tree.get_mut(&1) {
    ///     *value = "ONE";
    /// }
    /// assert_eq!(tree.get(&1), Some(&"ONE"));
    /// ```
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let root = self.root.clone();
        self.get_mut_recursive(&root, key)
    }

    /// Try to get a value, returning detailed error context on failure.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// A reference to the value if the key exists, or a detailed error.
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BPlusTreeMap;
    ///
    /// let mut tree = BPlusTreeMap::new(16).unwrap();
    /// tree.insert(1, "one");
    /// assert!(tree.try_get(&1).is_ok());
    /// assert!(tree.try_get(&2).is_err());
    /// ```
    pub fn try_get(&self, key: &K) -> KeyResult<&V> {
        self.get(key)
            .ok_or(BPlusTreeError::KeyNotFound)
    }

    /// Get multiple keys with detailed error reporting.
    ///
    /// # Arguments
    ///
    /// * `keys` - Slice of keys to look up
    ///
    /// # Returns
    ///
    /// A vector of references to the values if all keys exist, or an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BPlusTreeMap;
    ///
    /// let mut tree = BPlusTreeMap::new(16).unwrap();
    /// tree.insert(1, "one");
    /// tree.insert(2, "two");
    /// 
    /// let values = tree.get_many(&[1, 2]).unwrap();
    /// assert_eq!(values, vec![&"one", &"two"]);
    /// 
    /// assert!(tree.get_many(&[1, 3]).is_err()); // Key 3 doesn't exist
    /// ```
    pub fn get_many(&self, keys: &[K]) -> BTreeResult<Vec<&V>> {
        let mut values = Vec::new();

        for key in keys.iter() {
            match self.get(key) {
                Some(value) => values.push(value),
                None => {
                    return Err(BPlusTreeError::KeyNotFound);
                }
            }
        }

        Ok(values)
    }

    // ============================================================================
    // PRIVATE HELPER METHODS FOR GET OPERATIONS
    // ============================================================================

    /// Recursively search for a key in the tree.
    fn get_recursive<'a>(&'a self, node: &'a NodeRef<K, V>, key: &K) -> Option<&'a V> {
        match node {
            NodeRef::Leaf(id, _) => self.get_leaf(*id).and_then(|leaf| leaf.get(key)),
            NodeRef::Branch(id, _) => self
                .get_branch(*id)
                .and_then(|branch| branch.get_child(key))
                .and_then(|child| self.get_recursive(child, key)),
        }
    }

    /// Get mutable reference recursively.
    fn get_mut_recursive(&mut self, node: &NodeRef<K, V>, key: &K) -> Option<&mut V> {
        match node {
            NodeRef::Leaf(id, _) => self.get_leaf_mut(*id).and_then(|leaf| leaf.get_mut(key)),
            NodeRef::Branch(id, _) => {
                let (_child_index, child_ref) = self.get_child_for_key(*id, key)?;
                self.get_mut_recursive(&child_ref, key)
            }
        }
    }

    /// Helper to get child info for a key in a branch.
    pub fn get_child_for_key(&self, branch_id: NodeId, key: &K) -> Option<(usize, NodeRef<K, V>)> {
        let branch = self.get_branch(branch_id)?;
        let child_index = branch.find_child_index(key);
        branch
            .children
            .get(child_index)
            .cloned()
            .map(|child| (child_index, child))
    }

    // ============================================================================
    // ARENA ACCESS METHODS
    // ============================================================================

    /// Get a reference to a leaf node in the arena.
    pub fn get_leaf(&self, id: NodeId) -> Option<&LeafNode<K, V>> {
        self.leaf_arena.get(id)
    }

    /// Get a mutable reference to a leaf node in the arena.
    pub fn get_leaf_mut(&mut self, id: NodeId) -> Option<&mut LeafNode<K, V>> {
        self.leaf_arena.get_mut(id)
    }

    /// Get the next pointer of a leaf node in the arena.
    pub fn get_leaf_next(&self, id: NodeId) -> Option<NodeId> {
        self.get_leaf(id).and_then(|leaf| {
            if leaf.next == NULL_NODE {
                None
            } else {
                Some(leaf.next)
            }
        })
    }

    /// Get a reference to a branch node in the arena.
    pub fn get_branch(&self, id: NodeId) -> Option<&BranchNode<K, V>> {
        self.branch_arena.get(id)
    }

    /// Get a mutable reference to a branch node in the arena.
    pub fn get_branch_mut(&mut self, id: NodeId) -> Option<&mut BranchNode<K, V>> {
        self.branch_arena.get_mut(id)
    }
}

impl<K: Ord + Clone, V: Clone> LeafNode<K, V> {
    /// Get value for a key from this leaf node.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// A reference to the value if the key exists, `None` otherwise.
    pub fn get(&self, key: &K) -> Option<&V> {
        match self.keys.binary_search(key) {
            Ok(index) => Some(&self.values[index]),
            Err(_) => None,
        }
    }

    /// Get a mutable reference to the value for a key from this leaf node.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// A mutable reference to the value if the key exists, `None` otherwise.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.keys.binary_search(key) {
            Ok(index) => Some(&mut self.values[index]),
            Err(_) => None,
        }
    }
}

impl<K: Ord + Clone, V: Clone> BranchNode<K, V> {
    /// Get the child node for a given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to find the appropriate child for
    ///
    /// # Returns
    ///
    /// A reference to the child node that should contain the key.
    pub fn get_child(&self, key: &K) -> Option<&NodeRef<K, V>> {
        let child_index = self.find_child_index(key);
        if child_index < self.children.len() {
            Some(&self.children[child_index])
        } else {
            None
        }
    }

    /// Get a mutable reference to the child node for a given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to find the appropriate child for
    ///
    /// # Returns
    ///
    /// A mutable reference to the child node that should contain the key.
    pub fn get_child_mut(&mut self, key: &K) -> Option<&mut NodeRef<K, V>> {
        let child_index = self.find_child_index(key);
        if child_index >= self.children.len() {
            return None;
        }
        Some(&mut self.children[child_index])
    }

    /// Find the index of the child that should contain the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to find the child index for
    ///
    /// # Returns
    ///
    /// The index of the child that should contain the key.
    pub fn find_child_index(&self, key: &K) -> usize {
        match self.keys.binary_search(key) {
            Ok(index) => index + 1, // Key found, go to right child
            Err(index) => index,    // Key not found, go to left child at insertion point
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // BPlusTreeMap is already imported from types module

    #[test]
    fn test_basic_get_operations() {
        let mut tree = BPlusTreeMap::new(4).unwrap();
        
        // Test empty tree
        assert_eq!(tree.get(&1), None);
        assert!(!tree.contains_key(&1));
        
        // Insert some values
        tree.insert(1, "one");
        tree.insert(2, "two");
        tree.insert(3, "three");
        
        // Test get operations
        assert_eq!(tree.get(&1), Some(&"one"));
        assert_eq!(tree.get(&2), Some(&"two"));
        assert_eq!(tree.get(&3), Some(&"three"));
        assert_eq!(tree.get(&4), None);
        
        // Test contains_key
        assert!(tree.contains_key(&1));
        assert!(tree.contains_key(&2));
        assert!(tree.contains_key(&3));
        assert!(!tree.contains_key(&4));
    }

    #[test]
    fn test_get_or_default() {
        let mut tree = BPlusTreeMap::new(4).unwrap();
        tree.insert(1, "one");
        
        assert_eq!(tree.get_or_default(&1, &"default"), &"one");
        assert_eq!(tree.get_or_default(&2, &"default"), &"default");
    }

    #[test]
    fn test_get_item() {
        let mut tree = BPlusTreeMap::new(4).unwrap();
        tree.insert(1, "one");
        
        assert_eq!(tree.get_item(&1).unwrap(), &"one");
        assert!(tree.get_item(&2).is_err());
        assert!(matches!(tree.get_item(&2), Err(BPlusTreeError::KeyNotFound)));
    }

    #[test]
    fn test_get_mut() {
        let mut tree = BPlusTreeMap::new(4).unwrap();
        tree.insert(1, "one");
        
        // Test mutable access
        if let Some(value) = tree.get_mut(&1) {
            *value = "ONE";
        }
        assert_eq!(tree.get(&1), Some(&"ONE"));
        
        // Test non-existent key
        assert_eq!(tree.get_mut(&2), None);
    }

    #[test]
    fn test_get_many() {
        let mut tree = BPlusTreeMap::new(4).unwrap();
        tree.insert(1, "one");
        tree.insert(2, "two");
        tree.insert(3, "three");
        
        // Test successful get_many
        let values = tree.get_many(&[1, 2, 3]).unwrap();
        assert_eq!(values, vec![&"one", &"two", &"three"]);
        
        // Test partial failure
        assert!(tree.get_many(&[1, 2, 4]).is_err());
        
        // Test empty slice
        let empty_values = tree.get_many(&[]).unwrap();
        assert!(empty_values.is_empty());
    }

    #[test]
    fn test_try_get() {
        let mut tree = BPlusTreeMap::new(4).unwrap();
        tree.insert(1, "one");
        
        assert!(tree.try_get(&1).is_ok());
        assert_eq!(tree.try_get(&1).unwrap(), &"one");
        assert!(tree.try_get(&2).is_err());
    }

    #[test]
    fn test_leaf_node_get_operations() {
        let mut leaf = LeafNode::new(4);
        
        // Test empty leaf
        assert_eq!(leaf.get(&1), None);
        assert_eq!(leaf.get_mut(&1), None);
        
        // Add some data manually for testing
        leaf.keys.push(1);
        leaf.values.push("one");
        leaf.keys.push(3);
        leaf.values.push("three");
        
        // Test get operations
        assert_eq!(leaf.get(&1), Some(&"one"));
        assert_eq!(leaf.get(&3), Some(&"three"));
        assert_eq!(leaf.get(&2), None);
        
        // Test get_mut
        if let Some(value) = leaf.get_mut(&1) {
            *value = "ONE";
        }
        assert_eq!(leaf.get(&1), Some(&"ONE"));
    }

    #[test]
    fn test_branch_node_operations() {
        use crate::types::NodeRef;
        use std::marker::PhantomData;
        
        let mut branch = BranchNode::<i32, String>::new(4);
        
        // Add some keys and children for testing
        branch.keys.push(5);
        branch.keys.push(10);
        branch.children.push(NodeRef::Leaf(0, PhantomData));
        branch.children.push(NodeRef::Leaf(1, PhantomData));
        branch.children.push(NodeRef::Leaf(2, PhantomData));
        
        // Test find_child_index
        assert_eq!(branch.find_child_index(&3), 0);  // Less than first key
        assert_eq!(branch.find_child_index(&5), 1);  // Equal to first key
        assert_eq!(branch.find_child_index(&7), 1);  // Between keys
        assert_eq!(branch.find_child_index(&10), 2); // Equal to second key
        assert_eq!(branch.find_child_index(&15), 2); // Greater than all keys
        
        // Test get_child
        assert!(branch.get_child(&3).is_some());
        assert!(branch.get_child(&7).is_some());
        assert!(branch.get_child(&15).is_some());
    }
}