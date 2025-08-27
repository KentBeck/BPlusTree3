//! Construction and initialization logic for BPlusTreeMap and nodes.
//!
//! This module contains all the construction, initialization, and setup logic
//! for the B+ tree and its nodes. This includes capacity validation,
//! arena initialization, and default implementations.

use crate::compact_arena::CompactArena;
use crate::error::{BPlusTreeError, BTreeResult};
use crate::types::{BPlusTreeMap, BranchNode, LeafNode, NodeRef, MIN_CAPACITY, NULL_NODE};
use std::marker::PhantomData;

/// Result type for initialization operations
pub type InitResult<T> = BTreeResult<T>;

/// Default capacity for B+ tree nodes
pub const DEFAULT_CAPACITY: usize = 16;

impl<K, V> BPlusTreeMap<K, V> {
    /// Create a B+ tree with specified node capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of keys per node (minimum 8)
    ///
    /// # Returns
    ///
    /// Returns `Ok(BPlusTreeMap)` if capacity is valid, `Err(BPlusTreeError)` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BPlusTreeMap;
    ///
    /// let tree = BPlusTreeMap::<i32, String>::new(16).unwrap();
    /// assert!(tree.is_empty());
    /// ```
    pub fn new(capacity: usize) -> InitResult<Self> {
        if capacity < MIN_CAPACITY {
            return Err(BPlusTreeError::invalid_capacity(capacity, MIN_CAPACITY));
        }

        // Initialize compact arena with the first leaf at id=0
        let mut leaf_arena = CompactArena::new();
        let root_id = leaf_arena.allocate(LeafNode::new(capacity));

        // Initialize compact branch arena (starts empty)
        let branch_arena = CompactArena::new();

        Ok(Self {
            capacity,
            root: NodeRef::Leaf(root_id, PhantomData),
            leaf_arena,
            branch_arena,
        })
    }

    /// Create a B+ tree with default capacity.
    ///
    /// This is equivalent to calling `new(DEFAULT_CAPACITY)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BPlusTreeMap;
    ///
    /// let tree = BPlusTreeMap::<i32, String>::with_default_capacity().unwrap();
    /// // Tree created with default capacity
    /// ```
    pub fn with_default_capacity() -> InitResult<Self> {
        Self::new(DEFAULT_CAPACITY)
    }

    /// Create an empty B+ tree with specified capacity.
    ///
    /// Unlike `new()`, this creates a completely empty tree with no root node.
    /// This is useful for advanced use cases where you want to build the tree
    /// structure manually.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of keys per node (minimum 8)
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BPlusTreeMap;
    ///
    /// let tree = BPlusTreeMap::<i32, String>::empty(16).unwrap();
    /// // Empty tree created successfully
    /// ```
    pub fn empty(capacity: usize) -> InitResult<Self> {
        if capacity < MIN_CAPACITY {
            return Err(BPlusTreeError::invalid_capacity(capacity, MIN_CAPACITY));
        }

        // For empty tree, we still need a root - create an empty leaf
        let mut leaf_arena = CompactArena::new();
        let root_id = leaf_arena.allocate(LeafNode::new(capacity));

        Ok(Self {
            capacity,
            root: NodeRef::Leaf(root_id, PhantomData),
            leaf_arena,
            branch_arena: CompactArena::new(),
        })
    }
}

impl<K, V> LeafNode<K, V> {
    /// Creates a new leaf node with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of keys this node can hold
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::LeafNode;
    ///
    /// let leaf: LeafNode<i32, String> = LeafNode::new(16);
    /// // Leaf node created successfully
    /// ```
    pub fn new(capacity: usize) -> Self {
        // Pre-allocate to capacity to avoid reallocations during steady-state ops
        Self {
            capacity,
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            next: NULL_NODE,
        }
    }

    /// Creates a new leaf node with default capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::LeafNode;
    ///
    /// let leaf: LeafNode<i32, String> = LeafNode::with_default_capacity();
    /// // Leaf node created with default capacity
    /// ```
    pub fn with_default_capacity() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }

    /// Creates a new leaf node with pre-allocated capacity.
    ///
    /// This pre-allocates the internal vectors to the specified capacity,
    /// which can improve performance when you know the expected size.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of keys this node can hold
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::LeafNode;
    ///
    /// let leaf: LeafNode<i32, String> = LeafNode::with_reserved_capacity(16);
    /// // Leaf node created with reserved capacity
    /// ```
    pub fn with_reserved_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            next: NULL_NODE,
        }
    }
}

impl<K, V> BranchNode<K, V> {
    /// Creates a new branch node with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of keys this node can hold
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BranchNode;
    ///
    /// let branch: BranchNode<i32, String> = BranchNode::new(16);
    /// // Branch node created successfully
    /// ```
    pub fn new(capacity: usize) -> Self {
        // Pre-allocate: keys up to capacity, children up to capacity+1
        Self {
            capacity,
            keys: Vec::with_capacity(capacity),
            children: Vec::with_capacity(capacity + 1),
        }
    }

    /// Creates a new branch node with default capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BranchNode;
    ///
    /// let branch: BranchNode<i32, String> = BranchNode::with_default_capacity();
    /// // Branch node created with default capacity
    /// ```
    pub fn with_default_capacity() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }

    /// Creates a new branch node with pre-allocated capacity.
    ///
    /// This pre-allocates the internal vectors to the specified capacity,
    /// which can improve performance when you know the expected size.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of keys this node can hold
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BranchNode;
    ///
    /// let branch: BranchNode<i32, String> = BranchNode::with_reserved_capacity(16);
    /// // Branch node created with reserved capacity
    /// ```
    pub fn with_reserved_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            keys: Vec::with_capacity(capacity),
            children: Vec::with_capacity(capacity + 1), // Branch nodes have one more child than keys
        }
    }
}

// Default implementations
impl<K: Ord + Clone, V: Clone> Default for BPlusTreeMap<K, V> {
    /// Create a B+ tree with default capacity.
    fn default() -> Self {
        Self::with_default_capacity().unwrap()
    }
}

impl<K, V> Default for LeafNode<K, V> {
    /// Create a leaf node with default capacity.
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

impl<K, V> Default for BranchNode<K, V> {
    /// Create a branch node with default capacity.
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

/// Validation utilities for construction
pub mod validation {
    use super::*;

    /// Validate that a capacity is suitable for B+ tree nodes.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The capacity to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if valid, `Err(BPlusTreeError)` otherwise.
    #[allow(dead_code)]
    pub fn validate_capacity(capacity: usize) -> BTreeResult<()> {
        if capacity < MIN_CAPACITY {
            Err(BPlusTreeError::invalid_capacity(capacity, MIN_CAPACITY))
        } else {
            Ok(())
        }
    }

    /// Get the recommended capacity for a given expected number of elements.
    ///
    /// This uses heuristics to suggest an optimal node capacity based on
    /// the expected tree size.
    ///
    /// # Arguments
    ///
    /// * `expected_elements` - Expected number of elements in the tree
    ///
    /// # Returns
    ///
    /// Recommended capacity (always >= MIN_CAPACITY)
    #[allow(dead_code)]
    pub fn recommended_capacity(expected_elements: usize) -> usize {
        if expected_elements < 100 {
            MIN_CAPACITY
        } else if expected_elements < 10_000 {
            16
        } else if expected_elements < 1_000_000 {
            32
        } else {
            64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btree_construction() {
        let tree = BPlusTreeMap::<i32, String>::new(16).unwrap();
        assert_eq!(tree.capacity, 16);
        // Note: is_empty() and len() methods need to be implemented in the main module
    }

    #[test]
    fn test_btree_invalid_capacity() {
        let result = BPlusTreeMap::<i32, String>::new(2); // Below MIN_CAPACITY (4)
        assert!(result.is_err());
        // Note: is_capacity_error() method needs to be implemented in error module
    }

    #[test]
    fn test_btree_default() {
        let tree = BPlusTreeMap::<i32, String>::default();
        assert_eq!(tree.capacity, DEFAULT_CAPACITY);
    }

    #[test]
    fn test_btree_empty() {
        let tree = BPlusTreeMap::<i32, String>::empty(16).unwrap();
        // Note: is_empty() method needs to be implemented in the main module
        // For now, just check that it was created successfully
        assert_eq!(tree.capacity, 16);
    }

    #[test]
    fn test_leaf_construction() {
        let leaf = LeafNode::<i32, String>::new(16);
        assert_eq!(leaf.capacity, 16);
        assert!(leaf.keys_is_empty());
    }

    #[test]
    fn test_leaf_with_reserved_capacity() {
        let leaf = LeafNode::<i32, String>::with_reserved_capacity(16);
        // Note: We can't directly test Vec capacity without accessing private fields
        assert_eq!(leaf.capacity, 16);
    }

    #[test]
    fn test_branch_construction() {
        let branch = BranchNode::<i32, String>::new(16);
        assert_eq!(branch.capacity, 16);
        assert!(branch.keys.is_empty());
    }

    #[test]
    fn test_validation() {
        assert!(validation::validate_capacity(16).is_ok());
        assert!(validation::validate_capacity(4).is_ok()); // MIN_CAPACITY is 4
        assert!(validation::validate_capacity(2).is_err()); // Below MIN_CAPACITY
    }

    #[test]
    fn test_recommended_capacity() {
        assert_eq!(validation::recommended_capacity(50), MIN_CAPACITY);
        assert_eq!(validation::recommended_capacity(5000), 16);
        assert_eq!(validation::recommended_capacity(500_000), 32);
        assert_eq!(validation::recommended_capacity(5_000_000), 64);
    }
}
