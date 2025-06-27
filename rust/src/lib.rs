//! B+ Tree implementation in Rust with dict-like API.
//!
//! This module provides a B+ tree data structure with a dictionary-like interface,
//! supporting efficient insertion, deletion, lookup, and range queries.

use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds};

// Import our new modules
mod arena;
mod macros;

pub use arena::{Arena, NodeId as ArenaNodeId, NULL_NODE as ARENA_NULL_NODE};

// Constants
const MIN_CAPACITY: usize = 4;

/// Node ID type for arena-based allocation
pub type NodeId = u32;

/// Special node ID constants
pub const NULL_NODE: NodeId = u32::MAX;
pub const ROOT_NODE: NodeId = 0;

/// Error type for B+ tree operations.
#[derive(Debug, Clone, PartialEq)]
pub enum BPlusTreeError {
    /// Key not found in the tree.
    KeyNotFound,
    /// Invalid capacity specified.
    InvalidCapacity(String),
    /// Internal data structure integrity violation.
    DataIntegrityError(String),
}

/// B+ Tree implementation with Rust dict-like API.
///
/// A B+ tree is a self-balancing tree data structure that maintains sorted data
/// and allows searches, sequential access, insertions, and deletions in O(log n).
/// Unlike B trees, all values are stored in leaf nodes, making range queries
/// and sequential access very efficient.
///
/// # Type Parameters
///
/// * `K` - Key type that must implement `Ord + Clone + Debug`
/// * `V` - Value type that must implement `Clone + Debug`
///
/// # Examples
///
/// ```
/// use bplustree::BPlusTreeMap;
///
/// let mut tree = BPlusTreeMap::new(16).unwrap();
/// tree.insert(1, "one");
/// tree.insert(2, "two");
/// tree.insert(3, "three");
///
/// assert_eq!(tree.get(&2), Some(&"two"));
/// assert_eq!(tree.len(), 3);
///
/// // Range queries
/// let range: Vec<_> = tree.items_range(Some(&1), Some(&3)).collect();
/// assert_eq!(range, [(&1, &"one"), (&2, &"two")]);
/// ```
///
/// # Performance Characteristics
///
/// - **Insertion**: O(log n)
/// - **Lookup**: O(log n)
/// - **Deletion**: O(log n)
/// - **Range queries**: O(log n + k) where k is the number of items in range
/// - **Iteration**: O(n)
///
/// # Capacity Guidelines
///
/// - Minimum capacity: 4 (enforced)
/// - Recommended capacity: 16-128 depending on use case
/// - Higher capacity = fewer tree levels but larger nodes
/// - Lower capacity = more tree levels but smaller nodes
#[derive(Debug)]
pub struct BPlusTreeMap<K, V> {
    /// Maximum number of keys per node.
    capacity: usize,
    /// The root node of the tree.
    root: NodeRef<K, V>,

    // Enhanced arena-based allocation using generic Arena<T>
    /// Arena storage for leaf nodes.
    leaf_arena: Arena<LeafNode<K, V>>,
    /// Arena storage for branch nodes.
    branch_arena: Arena<BranchNode<K, V>>,
}

/// Node reference that can be either a leaf or branch node
#[derive(Debug, Clone)]
pub enum NodeRef<K, V> {
    Leaf(NodeId, PhantomData<(K, V)>),
    Branch(NodeId, PhantomData<(K, V)>),
}

impl<K, V> NodeRef<K, V> {
    /// Return the raw node ID.
    pub fn id(&self) -> NodeId {
        match *self {
            NodeRef::Leaf(id, _) => id,
            NodeRef::Branch(id, _) => id,
        }
    }

    /// Returns true if this reference points to a leaf node.
    pub fn is_leaf(&self) -> bool {
        matches!(self, NodeRef::Leaf(_, _))
    }
}

/// Node data that can be allocated in the arena after a split.
pub enum SplitNodeData<K, V> {
    Leaf(LeafNode<K, V>),
    Branch(BranchNode<K, V>),
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

/// Result of a removal operation on a node.
pub enum RemoveResult<V> {
    /// Removal completed. Contains the removed value if key existed.
    /// The bool indicates if this node is now underfull and needs rebalancing.
    Updated(Option<V>, bool),
}

impl<K: Ord + Clone, V: Clone> BPlusTreeMap<K, V> {
    // ============================================================================
    // CONSTRUCTION
    // ============================================================================

    /// Create a B+ tree with specified node capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of keys per node (minimum 4)
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
    pub fn new(capacity: usize) -> Result<Self, BPlusTreeError> {
        if capacity < MIN_CAPACITY {
            return Err(BPlusTreeError::InvalidCapacity(format!(
                "Capacity must be at least {} to maintain B+ tree invariants",
                MIN_CAPACITY
            )));
        }

        // Initialize arena with the first leaf at id=0
        let mut leaf_arena = Arena::new();
        let root_id = leaf_arena.allocate(LeafNode::new(capacity));

        // Initialize branch arena (starts empty)
        let branch_arena = Arena::new();

        Ok(Self {
            capacity,
            root: NodeRef::Leaf(root_id, PhantomData),
            leaf_arena,
            branch_arena,
        })
    }

    // ============================================================================
    // GET OPERATIONS
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
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Get value for a key with default.
    pub fn get_or_default<'a>(&'a self, key: &K, default: &'a V) -> &'a V {
        self.get(key).unwrap_or(default)
    }

    /// Get value for a key, returning an error if the key doesn't exist.
    /// This is equivalent to Python's `tree[key]`.
    pub fn get_item(&self, key: &K) -> Result<&V, BPlusTreeError> {
        self.get(key).ok_or(BPlusTreeError::KeyNotFound)
    }

    /// Get a mutable reference to the value for a key.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let root = self.root.clone();
        self.get_mut_recursive(&root, key)
    }

    // ============================================================================
    // HELPERS FOR GET OPERATIONS
    // ============================================================================

    /// Get mutable reference recursively
    fn get_mut_recursive(&mut self, node: &NodeRef<K, V>, key: &K) -> Option<&mut V> {
        match node {
            NodeRef::Leaf(id, _) => self.get_leaf_mut(*id).and_then(|leaf| leaf.get_mut(key)),
            NodeRef::Branch(id, _) => {
                let (_child_index, child_ref) = self.get_child_for_key(*id, key)?;
                self.get_mut_recursive(&child_ref, key)
            }
        }
    }

    /// Helper to get child info for a key in a branch
    fn get_child_for_key(&self, branch_id: NodeId, key: &K) -> Option<(usize, NodeRef<K, V>)> {
        let branch = self.get_branch(branch_id)?;
        let child_index = branch.find_child_index(key);
        branch
            .children
            .get(child_index)
            .cloned()
            .map(|child| (child_index, child))
    }

    /// Helper to check if a node is underfull
    fn is_node_underfull(&self, node_ref: &NodeRef<K, V>) -> bool {
        match node_ref {
            NodeRef::Leaf(id, _) => self
                .get_leaf(*id)
                .map(|leaf| leaf.is_underfull())
                .unwrap_or(false),
            NodeRef::Branch(id, _) => self
                .get_branch(*id)
                .map(|branch| branch.is_underfull())
                .unwrap_or(false),
        }
    }

    /// Helper to check if a node can donate
    fn can_node_donate(&self, node_ref: &NodeRef<K, V>) -> bool {
        match node_ref {
            NodeRef::Leaf(id, _) => self
                .get_leaf(*id)
                .map(|leaf| leaf.can_donate())
                .unwrap_or(false),
            NodeRef::Branch(id, _) => self
                .get_branch(*id)
                .map(|branch| branch.can_donate())
                .unwrap_or(false),
        }
    }

    /// Get sibling node reference if it exists and types match
    fn get_branch_sibling(
        &self,
        branch_id: NodeId,
        child_index: usize,
        get_left: bool,
    ) -> Option<NodeRef<K, V>> {
        let branch = self.get_branch(branch_id)?;
        let sibling_index = if get_left {
            child_index.checked_sub(1)?
        } else {
            child_index + 1
        };

        match (
            branch.children.get(child_index),
            branch.children.get(sibling_index),
        ) {
            (Some(NodeRef::Branch(_, _)), Some(NodeRef::Branch(_, _))) => {
                branch.children.get(sibling_index).cloned()
            }
            _ => None,
        }
    }

    fn get_recursive<'a>(&'a self, node: &'a NodeRef<K, V>, key: &K) -> Option<&'a V> {
        match node {
            NodeRef::Leaf(id, _) => self.get_leaf(*id).and_then(|leaf| leaf.get(key)),
            NodeRef::Branch(id, _) => self
                .get_branch(*id)
                .and_then(|branch| branch.get_child(key))
                .and_then(|child| self.get_recursive(child, key)),
        }
    }

    // ============================================================================
    // INSERT OPERATIONS
    // ============================================================================

    /// Insert a key-value pair into the tree.
    ///
    /// If the key already exists, the old value is returned and replaced.
    /// If the key is new, `None` is returned.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to insert
    /// * `value` - The value to associate with the key
    ///
    /// # Returns
    ///
    /// The previous value associated with the key, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BPlusTreeMap;
    ///
    /// let mut tree = BPlusTreeMap::new(16).unwrap();
    /// assert_eq!(tree.insert(1, "first"), None);
    /// assert_eq!(tree.insert(1, "second"), Some("first"));
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // Use insert_recursive to handle the insertion
        let result = self.insert_recursive(&self.root.clone(), key, value);

        match result {
            InsertResult::Updated(old_value) => old_value,
            InsertResult::Error(_error) => {
                // Log the error but maintain API compatibility
                // This should never happen with correct split logic
                eprintln!("BPlusTree internal error during insert - data integrity violation");
                None
            }
            InsertResult::Split {
                old_value,
                new_node_data,
                separator_key,
            } => {
                // Root split - need to create a new root
                let new_node_ref = match new_node_data {
                    SplitNodeData::Leaf(new_leaf_data) => {
                        let new_id = self.allocate_leaf(new_leaf_data);

                        // Update linked list pointers for root leaf split
                        if let Some(leaf) = matches!(&self.root, NodeRef::Leaf(_, _))
                            .then(|| self.root.id())
                            .and_then(|original_id| self.get_leaf_mut(original_id))
                        {
                            leaf.next = new_id;
                        }

                        NodeRef::Leaf(new_id, PhantomData)
                    }
                    SplitNodeData::Branch(new_branch_data) => {
                        let new_id = self.allocate_branch(new_branch_data);
                        NodeRef::Branch(new_id, PhantomData)
                    }
                };

                // Create new root with the split nodes
                let new_root = self.new_root(new_node_ref, separator_key);
                let root_id = self.allocate_branch(new_root);
                self.root = NodeRef::Branch(root_id, PhantomData);

                old_value
            }
        }
    }

    // ============================================================================
    // HELPERS FOR INSERT OPERATIONS
    // ============================================================================

    /// New roots are the only BranchNodes allowed to remain underfull
    fn new_root(&mut self, new_node: NodeRef<K, V>, separator_key: K) -> BranchNode<K, V> {
        let mut new_root = BranchNode::new(self.capacity);
        new_root.keys.push(separator_key);

        // Move the current root to be the left child
        // For arena-based implementation, create a placeholder arena leaf
        let placeholder_id = self.allocate_leaf(LeafNode::new(self.capacity));
        let placeholder = NodeRef::Leaf(placeholder_id, PhantomData);
        let old_root = std::mem::replace(&mut self.root, placeholder);

        new_root.children.push(old_root);
        new_root.children.push(new_node);

        // CRITICAL FIX: Deallocate the placeholder to prevent memory leak
        self.deallocate_leaf(placeholder_id);

        new_root
    }

    /// Recursively insert a key with proper arena access.
    fn insert_recursive(&mut self, node: &NodeRef<K, V>, key: K, value: V) -> InsertResult<K, V> {
        match node {
            NodeRef::Leaf(id, _) => self
                .get_leaf_mut(*id)
                .map_or(InsertResult::Updated(None), |leaf| leaf.insert(key, value)),
            NodeRef::Branch(id, _) => {
                let id = *id;

                // First get child info without mutable borrow
                let (child_index, child_ref) = match self.get_child_for_key(id, &key) {
                    Some(info) => info,
                    None => return InsertResult::Updated(None),
                };

                // Recursively insert
                let child_result = self.insert_recursive(&child_ref, key, value);

                // Handle the result
                match child_result {
                    InsertResult::Updated(old_value) => InsertResult::Updated(old_value),
                    InsertResult::Error(error) => InsertResult::Error(error),
                    InsertResult::Split {
                        old_value,
                        new_node_data,
                        separator_key,
                    } => {
                        // Allocate the new node based on its type
                        let new_node = match new_node_data {
                            SplitNodeData::Leaf(new_leaf_data) => {
                                let new_id = self.allocate_leaf(new_leaf_data);

                                // Update linked list pointers for leaf splits
                                if let NodeRef::Leaf(original_id, _) = child_ref {
                                    // Update the original leaf's next pointer to point to the new leaf
                                    if let Some(original_leaf) = self.get_leaf_mut(original_id) {
                                        original_leaf.next = new_id;
                                    }
                                }

                                NodeRef::Leaf(new_id, PhantomData)
                            }
                            SplitNodeData::Branch(new_branch_data) => {
                                let new_id = self.allocate_branch(new_branch_data);
                                NodeRef::Branch(new_id, PhantomData)
                            }
                        };

                        // Insert into this branch
                        match self.get_branch_mut(id).and_then(|branch| {
                            branch.insert_child_and_split_if_needed(
                                child_index,
                                separator_key,
                                new_node,
                            )
                        }) {
                            Some((new_branch_data, promoted_key)) => {
                                // This branch split too - return raw branch data
                                InsertResult::Split {
                                    old_value,
                                    new_node_data: SplitNodeData::Branch(new_branch_data),
                                    separator_key: promoted_key,
                                }
                            }
                            None => {
                                // No split needed or branch not found
                                InsertResult::Updated(old_value)
                            }
                        }
                    }
                }
            }
        }
    }

    // ============================================================================
    // DELETE OPERATIONS
    // ============================================================================

    /// Remove a key from the tree.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        // Use remove_recursive to handle the removal
        let result = self.remove_recursive(&self.root.clone(), key);

        match result {
            RemoveResult::Updated(removed_value, _root_became_underfull) => {
                // Check if root needs collapsing after removal
                if removed_value.is_some() {
                    self.collapse_root_if_needed();
                }
                removed_value
            }
        }
    }

    /// Remove a key from the tree, returning an error if the key doesn't exist.
    /// This is equivalent to Python's `del tree[key]`.
    pub fn remove_item(&mut self, key: &K) -> Result<V, BPlusTreeError> {
        self.remove(key).ok_or(BPlusTreeError::KeyNotFound)
    }

    // ============================================================================
    // HELPERS FOR DELETE OPERATIONS
    // ============================================================================

    /// Recursively remove a key with proper arena access.
    fn remove_recursive(&mut self, node: &NodeRef<K, V>, key: &K) -> RemoveResult<V> {
        match node {
            NodeRef::Leaf(id, _) => {
                self.get_leaf_mut(*id)
                    .map_or(RemoveResult::Updated(None, false), |leaf| {
                        let removed_value = leaf.remove(key);
                        let is_underfull = leaf.is_underfull();
                        RemoveResult::Updated(removed_value, is_underfull)
                    })
            }
            NodeRef::Branch(id, _) => {
                let id = *id;

                // First get child info without mutable borrow
                let (child_index, child_ref) = match self.get_child_for_key(id, key) {
                    Some(info) => info,
                    None => return RemoveResult::Updated(None, false),
                };

                // Recursively remove
                let child_result = self.remove_recursive(&child_ref, key);

                // Handle the result
                match child_result {
                    RemoveResult::Updated(removed_value, child_became_underfull) => {
                        // If child became underfull, try to rebalance
                        if removed_value.is_some() && child_became_underfull {
                            let _child_still_exists = self.rebalance_child(id, child_index);
                        }

                        // Check if this branch is now underfull after rebalancing
                        let is_underfull =
                            self.is_node_underfull(&NodeRef::Branch(id, PhantomData));
                        RemoveResult::Updated(removed_value, is_underfull)
                    }
                }
            }
        }
    }
    /// Rebalance an underfull child in an arena branch
    fn rebalance_child(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Get information about the child and its siblings
        let (has_left_sibling, has_right_sibling, child_is_leaf) = match self.get_branch(branch_id)
        {
            Some(branch) => {
                let has_left = child_index > 0;
                let has_right = child_index < branch.children.len() - 1;
                let is_leaf = matches!(branch.children[child_index], NodeRef::Leaf(_, _));
                (has_left, has_right, is_leaf)
            }
            None => return false,
        };

        if child_is_leaf {
            // Handle leaf rebalancing
            self.rebalance_leaf_child(branch_id, child_index, has_left_sibling, has_right_sibling)
        } else {
            // Handle branch rebalancing
            self.rebalance_branch_child(branch_id, child_index, has_left_sibling, has_right_sibling)
        }
    }

    /// Rebalance an underfull leaf child
    fn rebalance_leaf_child(
        &mut self,
        branch_id: NodeId,
        child_index: usize,
        has_left_sibling: bool,
        has_right_sibling: bool,
    ) -> bool {
        // Get parent branch once and cache sibling info to avoid multiple arena lookups
        let (left_can_donate, right_can_donate) = match self.get_branch(branch_id) {
            Some(branch) => {
                let left_can_donate = if has_left_sibling && child_index > 0 {
                    branch
                        .children
                        .get(child_index - 1)
                        .map(|sibling| self.can_node_donate(sibling))
                        .unwrap_or(false)
                } else {
                    false
                };

                let right_can_donate = if has_right_sibling {
                    branch
                        .children
                        .get(child_index + 1)
                        .map(|sibling| self.can_node_donate(sibling))
                        .unwrap_or(false)
                } else {
                    false
                };

                (left_can_donate, right_can_donate)
            }
            None => return false,
        };

        // Try borrowing from left sibling first
        if left_can_donate {
            return self.borrow_from_left_leaf(branch_id, child_index);
        }

        // Try borrowing from right sibling
        if right_can_donate {
            return self.borrow_from_right_leaf(branch_id, child_index);
        }

        // Cannot borrow, must merge
        if has_left_sibling {
            return self.merge_with_left_leaf(branch_id, child_index);
        } else if has_right_sibling {
            return self.merge_with_right_leaf(branch_id, child_index);
        }

        // No siblings to merge with - this shouldn't happen
        false
    }

    /// Rebalance an underfull branch child
    fn rebalance_branch_child(
        &mut self,
        branch_id: NodeId,
        child_index: usize,
        has_left_sibling: bool,
        has_right_sibling: bool,
    ) -> bool {
        // Get parent branch once and cache sibling donation and merge capabilities
        let (left_can_donate, right_can_donate, left_can_merge, right_can_merge) =
            match self.get_branch(branch_id) {
                Some(branch) => {
                    let left_can_donate = if has_left_sibling {
                        self.get_branch_sibling(branch_id, child_index, true)
                            .map(|sibling| self.can_node_donate(&sibling))
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    let right_can_donate = if has_right_sibling {
                        self.get_branch_sibling(branch_id, child_index, false)
                            .map(|sibling| self.can_node_donate(&sibling))
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    let left_can_merge = if has_left_sibling {
                        if let (NodeRef::Branch(left_id, _), NodeRef::Branch(child_id, _)) = (
                            &branch.children[child_index - 1],
                            &branch.children[child_index],
                        ) {
                            self.get_branch(*left_id)
                                .zip(self.get_branch(*child_id))
                                .map(|(left, child)| {
                                    left.keys.len() + 1 + child.keys.len() <= self.capacity
                                })
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    let right_can_merge = if has_right_sibling {
                        if let (NodeRef::Branch(child_id, _), NodeRef::Branch(right_id, _)) = (
                            &branch.children[child_index],
                            &branch.children[child_index + 1],
                        ) {
                            self.get_branch(*child_id)
                                .zip(self.get_branch(*right_id))
                                .map(|(child, right)| {
                                    child.keys.len() + 1 + right.keys.len() <= self.capacity
                                })
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    (
                        left_can_donate,
                        right_can_donate,
                        left_can_merge,
                        right_can_merge,
                    )
                }
                None => return false,
            };

        // Try borrowing from left sibling first
        if left_can_donate {
            return self.borrow_from_left_branch(branch_id, child_index);
        }

        // Try borrowing from right sibling
        if right_can_donate {
            return self.borrow_from_right_branch(branch_id, child_index);
        }

        // Try merging with left sibling
        if left_can_merge {
            return self.merge_with_left_branch(branch_id, child_index);
        }

        // Try merging with right sibling
        if right_can_merge {
            return self.merge_with_right_branch(branch_id, child_index);
        }

        // Cannot borrow or merge - leave the node underfull
        // This can happen when siblings are also near minimum capacity
        true
    }

    /// Merge branch with left sibling
    fn merge_with_left_branch(&mut self, parent_id: NodeId, child_index: usize) -> bool {
        // Get the branch IDs and collect all needed info from parent in one access
        let (left_id, child_id, separator_key) = match self.get_branch(parent_id) {
            Some(parent) => {
                match (
                    &parent.children[child_index - 1],
                    &parent.children[child_index],
                ) {
                    (NodeRef::Branch(left, _), NodeRef::Branch(child, _)) => {
                        (*left, *child, parent.keys[child_index - 1].clone())
                    }
                    _ => return false,
                }
            }
            None => return false,
        };

        // Extract all content from child
        let mut child_branch = match self.get_branch_mut(child_id) {
            Some(child_branch) => {
                let mut extracted = BranchNode::new(child_branch.capacity);
                std::mem::swap(&mut extracted.keys, &mut child_branch.keys);
                std::mem::swap(&mut extracted.children, &mut child_branch.children);
                extracted
            }
            None => return false,
        };

        // Merge into left branch
        if let Some(left_branch) = self.get_branch_mut(left_id) {
            left_branch.merge_from(separator_key, &mut child_branch);
        } else {
            return false;
        }

        // Remove child from parent (second and final parent access)
        if let Some(parent) = self.get_branch_mut(parent_id) {
            parent.children.remove(child_index);
            parent.keys.remove(child_index - 1);
        }

        // Deallocate the merged child
        self.deallocate_branch(child_id);

        false // Child was merged away
    }

    /// Merge branch with right sibling
    fn merge_with_right_branch(&mut self, parent_id: NodeId, child_index: usize) -> bool {
        // Get the branch IDs and collect all needed info from parent in one access
        let (child_id, right_id, separator_key) = match self.get_branch(parent_id) {
            Some(parent) => {
                match (
                    &parent.children[child_index],
                    &parent.children[child_index + 1],
                ) {
                    (NodeRef::Branch(child, _), NodeRef::Branch(right, _)) => {
                        (*child, *right, parent.keys[child_index].clone())
                    }
                    _ => return false,
                }
            }
            None => return false,
        };

        // Extract all content from right sibling
        let mut right_branch = match self.get_branch_mut(right_id) {
            Some(right_branch) => {
                let mut extracted = BranchNode::new(right_branch.capacity);
                std::mem::swap(&mut extracted.keys, &mut right_branch.keys);
                std::mem::swap(&mut extracted.children, &mut right_branch.children);
                extracted
            }
            None => return false,
        };

        // Merge into child branch
        if let Some(child_branch) = self.get_branch_mut(child_id) {
            child_branch.merge_from(separator_key, &mut right_branch);
        } else {
            return false;
        }

        // Remove right from parent (second and final parent access)
        if let Some(parent) = self.get_branch_mut(parent_id) {
            parent.children.remove(child_index + 1);
            parent.keys.remove(child_index);
        }

        // Deallocate the merged right sibling
        self.deallocate_branch(right_id);

        true // Child still exists
    }

    /// Borrow from left sibling branch
    fn borrow_from_left_branch(&mut self, parent_id: NodeId, child_index: usize) -> bool {
        // Get the branch IDs and collect all needed info from parent in one access
        let (left_id, child_id, separator_key) = match self.get_branch(parent_id) {
            Some(parent) => {
                match (
                    &parent.children[child_index - 1],
                    &parent.children[child_index],
                ) {
                    (NodeRef::Branch(left, _), NodeRef::Branch(child, _)) => {
                        (*left, *child, parent.keys[child_index - 1].clone())
                    }
                    _ => return false,
                }
            }
            None => return false,
        };

        // Borrow from left branch
        let (moved_key, moved_child) = match self.get_branch_mut(left_id) {
            Some(left_branch) => match left_branch.borrow_last() {
                Some(result) => result,
                None => return false,
            },
            None => return false,
        };

        // Accept into child branch
        let new_separator = if let Some(child_branch) = self.get_branch_mut(child_id) {
            child_branch.accept_from_left(separator_key, moved_key, moved_child)
        } else {
            return false;
        };

        // Update separator in parent (second and final parent access)
        if let Some(parent) = self.get_branch_mut(parent_id) {
            parent.keys[child_index - 1] = new_separator;
        }

        true
    }

    /// Borrow from right sibling branch
    fn borrow_from_right_branch(&mut self, parent_id: NodeId, child_index: usize) -> bool {
        // Get the branch IDs and collect all needed info from parent in one access
        let (child_id, right_id, separator_key) = match self.get_branch(parent_id) {
            Some(parent) => {
                match (
                    &parent.children[child_index],
                    &parent.children[child_index + 1],
                ) {
                    (NodeRef::Branch(child, _), NodeRef::Branch(right, _)) => {
                        (*child, *right, parent.keys[child_index].clone())
                    }
                    _ => return false,
                }
            }
            None => return false,
        };

        // Borrow from right branch
        let (moved_key, moved_child) = match self.get_branch_mut(right_id) {
            Some(right_branch) => match right_branch.borrow_first() {
                Some(result) => result,
                None => return false,
            },
            None => return false,
        };

        // Accept into child branch
        let new_separator = if let Some(child_branch) = self.get_branch_mut(child_id) {
            child_branch.accept_from_right(separator_key, moved_key, moved_child)
        } else {
            return false;
        };

        // Update separator in parent (second and final parent access)
        if let Some(parent) = self.get_branch_mut(parent_id) {
            parent.keys[child_index] = new_separator;
        }

        true
    }

    /// Borrow from left sibling leaf
    fn borrow_from_left_leaf(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Extract leaf IDs from parent in one access (inlined get_adjacent_leaf_ids logic)
        let (left_id, child_id) = match self.get_branch(branch_id) {
            Some(branch) => {
                match (
                    branch.children.get(child_index - 1),
                    branch.children.get(child_index),
                ) {
                    (Some(NodeRef::Leaf(left_id, _)), Some(NodeRef::Leaf(child_id, _))) => {
                        (*left_id, *child_id)
                    }
                    _ => return false,
                }
            }
            None => return false,
        };

        // Borrow from left leaf
        let (key, value) = match self.get_leaf_mut(left_id) {
            Some(left_leaf) => match left_leaf.borrow_last() {
                Some(result) => result,
                None => return false,
            },
            None => return false,
        };

        // Accept into child leaf
        if let Some(child_leaf) = self.get_leaf_mut(child_id) {
            child_leaf.accept_from_left(key.clone(), value);
        } else {
            return false;
        }

        // Update separator in parent (second and final parent access)
        if let Some(branch) = self.get_branch_mut(branch_id) {
            branch.keys[child_index - 1] = key;
        }

        true
    }

    /// Borrow from right sibling leaf
    fn borrow_from_right_leaf(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Extract leaf IDs from parent in one access (inlined get_adjacent_leaf_ids logic)
        let (child_id, right_id) = match self.get_branch(branch_id) {
            Some(branch) => {
                match (
                    branch.children.get(child_index),
                    branch.children.get(child_index + 1),
                ) {
                    (Some(NodeRef::Leaf(child_id, _)), Some(NodeRef::Leaf(right_id, _))) => {
                        (*child_id, *right_id)
                    }
                    _ => return false,
                }
            }
            None => return false,
        };

        // Borrow from right leaf
        let (key, value) = match self.get_leaf_mut(right_id) {
            Some(right_leaf) => match right_leaf.borrow_first() {
                Some(result) => result,
                None => return false,
            },
            None => return false,
        };

        // Accept into child leaf
        if let Some(child_leaf) = self.get_leaf_mut(child_id) {
            child_leaf.accept_from_right(key, value);
        } else {
            return false;
        }

        // Update separator in parent (new first key of right sibling, second parent access)
        let new_separator = self
            .get_leaf(right_id)
            .and_then(|right_leaf| right_leaf.keys.first().cloned());

        if let Some(new_sep) = new_separator {
            if let Some(branch) = self.get_branch_mut(branch_id) {
                branch.keys[child_index] = new_sep;
            }
        }

        true
    }

    /// Merge with left sibling leaf
    fn merge_with_left_leaf(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Extract leaf IDs from parent in one access (inlined get_adjacent_leaf_ids logic)
        let (left_id, child_id) = match self.get_branch(branch_id) {
            Some(branch) => {
                match (
                    branch.children.get(child_index - 1),
                    branch.children.get(child_index),
                ) {
                    (Some(NodeRef::Leaf(left_id, _)), Some(NodeRef::Leaf(child_id, _))) => {
                        (*left_id, *child_id)
                    }
                    _ => return false,
                }
            }
            None => return false,
        };

        // Extract all content from child
        let (mut child_keys, mut child_values, child_next) = match self.get_leaf_mut(child_id) {
            Some(child_leaf) => child_leaf.extract_all(),
            None => return false,
        };

        // Merge into left leaf and update linked list
        if let Some(left_leaf) = self.get_leaf_mut(left_id) {
            left_leaf.keys.append(&mut child_keys);
            left_leaf.values.append(&mut child_values);
            left_leaf.next = child_next;
        } else {
            return false;
        }

        // Remove child from parent (second and final parent access)
        if let Some(branch) = self.get_branch_mut(branch_id) {
            branch.children.remove(child_index);
            branch.keys.remove(child_index - 1);
        }

        // Deallocate the merged child
        self.deallocate_leaf(child_id);

        false // Child was merged away
    }

    /// Merge with right sibling leaf
    fn merge_with_right_leaf(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Extract leaf IDs from parent in one access (inlined get_adjacent_leaf_ids logic)
        let (child_id, right_id) = match self.get_branch(branch_id) {
            Some(branch) => {
                match (
                    branch.children.get(child_index),
                    branch.children.get(child_index + 1),
                ) {
                    (Some(NodeRef::Leaf(child_id, _)), Some(NodeRef::Leaf(right_id, _))) => {
                        (*child_id, *right_id)
                    }
                    _ => return false,
                }
            }
            None => return false,
        };

        // Extract all content from right sibling
        let (mut right_keys, mut right_values, right_next) = match self.get_leaf_mut(right_id) {
            Some(right_leaf) => right_leaf.extract_all(),
            None => return false,
        };

        // Merge into child leaf and update linked list
        if let Some(child_leaf) = self.get_leaf_mut(child_id) {
            child_leaf.keys.append(&mut right_keys);
            child_leaf.values.append(&mut right_values);
            child_leaf.next = right_next;
        } else {
            return false;
        }

        // Remove right from parent (second and final parent access)
        if let Some(branch) = self.get_branch_mut(branch_id) {
            branch.children.remove(child_index + 1);
            branch.keys.remove(child_index);
        }

        // Deallocate the merged right sibling
        self.deallocate_leaf(right_id);

        true // Child still exists
    }

    /// Collapse the root if it's a branch with only one child or no children.
    fn collapse_root_if_needed(&mut self) {
        loop {
            // Capture root ID first to avoid borrowing conflicts
            let root_branch_id = match &self.root {
                NodeRef::Branch(id, _) => Some(*id),
                NodeRef::Leaf(_, _) => None,
            };

            if let Some(branch_id) = root_branch_id {
                if let Some(branch) = self.get_branch(branch_id) {
                    if branch.children.is_empty() {
                        // Root branch has no children, replace with empty arena leaf
                        let empty_id = self.allocate_leaf(LeafNode::new(self.capacity));
                        self.root = NodeRef::Leaf(empty_id, PhantomData);
                        // Deallocate the old root branch to prevent arena leak
                        self.deallocate_branch(branch_id);
                        break;
                    } else if branch.children.len() == 1 {
                        // Root branch has only one child, replace root with that child
                        let new_root = branch.children[0].clone();
                        self.root = new_root;
                        // Deallocate the old root branch to prevent arena leak
                        self.deallocate_branch(branch_id);
                        // Continue the loop in case the new root also needs collapsing
                    } else {
                        // Root branch has multiple children, no collapse needed
                        break;
                    }
                } else {
                    // Missing arena branch, replace with empty arena leaf
                    let empty_id = self.allocate_leaf(LeafNode::new(self.capacity));
                    self.root = NodeRef::Leaf(empty_id, PhantomData);
                    break;
                }
            } else {
                // Already a leaf root, no collapse needed
                break;
            }
        }
    }

    // ============================================================================
    // OTHER API OPERATIONS
    // ============================================================================

    /// Returns the number of elements in the tree.
    pub fn len(&self) -> usize {
        self.len_recursive(&self.root)
    }

    /// Recursively count elements with proper arena access.
    fn len_recursive(&self, node: &NodeRef<K, V>) -> usize {
        match node {
            NodeRef::Leaf(id, _) => self.get_leaf(*id).map(|leaf| leaf.len()).unwrap_or(0),
            NodeRef::Branch(id, _) => self
                .get_branch(*id)
                .map(|branch| {
                    branch
                        .children
                        .iter()
                        .map(|child| self.len_recursive(child))
                        .sum()
                })
                .unwrap_or(0),
        }
    }

    /// Returns true if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if the root is a leaf node.
    pub fn is_leaf_root(&self) -> bool {
        matches!(self.root, NodeRef::Leaf(_, _))
    }

    /// Returns the number of leaf nodes in the tree.
    pub fn leaf_count(&self) -> usize {
        self.leaf_count_recursive(&self.root)
    }

    /// Recursively count leaf nodes with proper arena access.
    fn leaf_count_recursive(&self, node: &NodeRef<K, V>) -> usize {
        match node {
            NodeRef::Leaf(_, _) => 1, // An arena leaf is one leaf node
            NodeRef::Branch(id, _) => self
                .get_branch(*id)
                .map(|branch| {
                    branch
                        .children
                        .iter()
                        .map(|child| self.leaf_count_recursive(child))
                        .sum()
                })
                .unwrap_or(0),
        }
    }

    /// Clear all items from the tree.
    pub fn clear(&mut self) {
        // Clear all arenas and create a new root leaf
        self.leaf_arena.clear();
        self.branch_arena.clear();

        // Create a new root leaf
        let root_leaf = LeafNode::new(self.capacity);
        let root_id = self.leaf_arena.allocate(root_leaf);
        self.root = NodeRef::Leaf(root_id, PhantomData);
    }

    /// Returns an iterator over all key-value pairs in sorted order.
    pub fn items(&self) -> ItemIterator<K, V> {
        ItemIterator::new(self)
    }

    /// Returns an iterator over all keys in sorted order.
    pub fn keys(&self) -> KeyIterator<K, V> {
        KeyIterator::new(self)
    }

    /// Returns an iterator over all values in key order.
    pub fn values(&self) -> ValueIterator<K, V> {
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
        RangeIterator::new(self, start_key, end_key)
    }

    /// Returns an iterator over key-value pairs in a range using Rust's range syntax.
    ///
    /// # Examples
    ///
    /// ```
    /// use bplustree::BPlusTreeMap;
    ///
    /// let mut tree = BPlusTreeMap::new(16).unwrap();
    /// for i in 0..10 {
    ///     tree.insert(i, format!("value{}", i));
    /// }
    ///
    /// // Different range syntaxes
    /// let range1: Vec<_> = tree.range(3..7).map(|(k, v)| (*k, v.clone())).collect();
    /// assert_eq!(range1, vec![(3, "value3".to_string()), (4, "value4".to_string()),
    ///                         (5, "value5".to_string()), (6, "value6".to_string())]);
    ///
    /// let range2: Vec<_> = tree.range(3..=7).map(|(k, v)| (*k, v.clone())).collect();
    /// assert_eq!(range2, vec![(3, "value3".to_string()), (4, "value4".to_string()),
    ///                         (5, "value5".to_string()), (6, "value6".to_string()),
    ///                         (7, "value7".to_string())]);
    ///
    /// let range3: Vec<_> = tree.range(5..).map(|(k, v)| *k).collect();
    /// assert_eq!(range3, vec![5, 6, 7, 8, 9]);
    ///
    /// let range4: Vec<_> = tree.range(..5).map(|(k, v)| *k).collect();
    /// assert_eq!(range4, vec![0, 1, 2, 3, 4]);
    ///
    /// let range5: Vec<_> = tree.range(..).map(|(k, v)| *k).collect();
    /// assert_eq!(range5, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// ```
    pub fn range<R>(&self, range: R) -> RangeIterator<'_, K, V>
    where
        R: RangeBounds<K>,
    {
        // Handle start position and find actual start key/position
        let (start_info, skip_first) = match range.start_bound() {
            Bound::Included(key) => {
                if let Some((leaf_id, index)) = self.find_range_start(key) {
                    (Some((leaf_id, index)), false)
                } else {
                    (None, false)
                }
            }
            Bound::Excluded(key) => {
                if let Some((leaf_id, index)) = self.find_range_start(key) {
                    // We need to check if the first item equals the excluded key
                    (Some((leaf_id, index)), true)
                } else {
                    (None, false)
                }
            }
            Bound::Unbounded => {
                if let Some(first_leaf) = self.get_first_leaf_id() {
                    (Some((first_leaf, 0)), false)
                } else {
                    (None, false)
                }
            }
        };

        // Handle end bound by creating a cloned end key to avoid lifetime issues
        let end_info = match range.end_bound() {
            Bound::Included(key) => Some((key.clone(), true)),
            Bound::Excluded(key) => Some((key.clone(), false)),
            Bound::Unbounded => None,
        };

        RangeIterator::new_with_skip_owned(self, start_info, skip_first, end_info)
    }

    /// Returns the first key-value pair in the tree.
    pub fn first(&self) -> Option<(&K, &V)> {
        self.items().next()
    }

    /// Returns the last key-value pair in the tree.
    pub fn last(&self) -> Option<(&K, &V)> {
        self.items().last()
    }

    // ============================================================================
    // RANGE QUERY OPTIMIZATION HELPERS
    // ============================================================================

    /// Find the leaf node and index where a range should start
    fn find_range_start(&self, start_key: &K) -> Option<(NodeId, usize)> {
        let mut current = &self.root;

        // Navigate down to leaf level
        loop {
            match current {
                NodeRef::Leaf(leaf_id, _) => {
                    if let Some(leaf) = self.get_leaf(*leaf_id) {
                        // Find the first key >= start_key in this leaf
                        let index = leaf
                            .keys
                            .iter()
                            .position(|k| k >= start_key)
                            .unwrap_or(leaf.keys.len());

                        if index < leaf.keys.len() {
                            return Some((*leaf_id, index));
                        } else {
                            // All keys in this leaf are < start_key
                            // Move to next leaf if it exists
                            if leaf.next != NULL_NODE {
                                if let Some(next_leaf) = self.get_leaf(leaf.next) {
                                    if !next_leaf.keys.is_empty() {
                                        return Some((leaf.next, 0));
                                    }
                                }
                            }
                            return None; // No valid start position
                        }
                    }
                    return None;
                }
                NodeRef::Branch(branch_id, _) => {
                    if let Some(branch) = self.get_branch(*branch_id) {
                        // Find the child that could contain start_key
                        let child_index = branch.find_child_index(start_key);

                        if child_index < branch.children.len() {
                            current = &branch.children[child_index];
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
            }
        }
    }

    /// Get the ID of the first (leftmost) leaf in the tree
    fn get_first_leaf_id(&self) -> Option<NodeId> {
        let mut current = &self.root;

        loop {
            match current {
                NodeRef::Leaf(leaf_id, _) => return Some(*leaf_id),
                NodeRef::Branch(branch_id, _) => {
                    if let Some(branch) = self.get_branch(*branch_id) {
                        if !branch.children.is_empty() {
                            current = &branch.children[0];
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
            }
        }
    }

    // ============================================================================
    // ENHANCED ARENA-BASED ALLOCATION FOR LEAF NODES
    // ============================================================================

    /// Allocate a new leaf node in the arena and return its ID.
    pub fn allocate_leaf(&mut self, leaf: LeafNode<K, V>) -> NodeId {
        self.leaf_arena.allocate(leaf)
    }

    /// Deallocate a leaf node from the arena.
    pub fn deallocate_leaf(&mut self, id: NodeId) -> Option<LeafNode<K, V>> {
        self.leaf_arena.deallocate(id)
    }

    /// Get a reference to a leaf node in the arena.
    pub fn get_leaf(&self, id: NodeId) -> Option<&LeafNode<K, V>> {
        self.leaf_arena.get(id)
    }

    /// Get a mutable reference to a leaf node in the arena.
    pub fn get_leaf_mut(&mut self, id: NodeId) -> Option<&mut LeafNode<K, V>> {
        self.leaf_arena.get_mut(id)
    }

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

    /// Set the next pointer of a leaf node in the arena.
    pub fn set_leaf_next(&mut self, id: NodeId, next_id: NodeId) -> bool {
        self.get_leaf_mut(id)
            .map(|leaf| {
                leaf.next = next_id;
                true
            })
            .unwrap_or(false)
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

    // ============================================================================
    // CHILD LOOKUP HELPERS (Phase 2)
    // ============================================================================

    /// Find the child index and `NodeRef` for `key` in the specified branch,
    /// returning `None` if the branch does not exist or index is out of range.
    pub fn find_child(&self, branch_id: NodeId, key: &K) -> Option<(usize, NodeRef<K, V>)> {
        self.get_branch(branch_id).and_then(|branch| {
            let idx = branch.find_child_index(key);
            branch.children.get(idx).cloned().map(|child| (idx, child))
        })
    }

    /// Mutable version of `find_child`.
    pub fn find_child_mut(&mut self, branch_id: NodeId, key: &K) -> Option<(usize, NodeRef<K, V>)> {
        self.get_branch_mut(branch_id).and_then(|branch| {
            let idx = branch.find_child_index(key);
            branch.children.get(idx).cloned().map(|child| (idx, child))
        })
    }

    // ============================================================================
    // ENHANCED ARENA-BASED ALLOCATION FOR BRANCH NODES
    // ============================================================================

    /// Allocate a new branch node in the arena and return its ID.
    pub fn allocate_branch(&mut self, branch: BranchNode<K, V>) -> NodeId {
        self.branch_arena.allocate(branch)
    }

    /// Deallocate a branch node from the arena.
    pub fn deallocate_branch(&mut self, id: NodeId) -> Option<BranchNode<K, V>> {
        self.branch_arena.deallocate(id)
    }

    /// Get a reference to a branch node in the arena.
    pub fn get_branch(&self, id: NodeId) -> Option<&BranchNode<K, V>> {
        self.branch_arena.get(id)
    }

    /// Get a mutable reference to a branch node in the arena.
    pub fn get_branch_mut(&mut self, id: NodeId) -> Option<&mut BranchNode<K, V>> {
        self.branch_arena.get_mut(id)
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

    // ============================================================================
    // OTHER HELPERS (TEST HELPERS)
    // ============================================================================

    /// Check if the tree maintains B+ tree invariants.
    /// Returns true if all invariants are satisfied.
    pub fn check_invariants(&self) -> bool {
        self.check_node_invariants(&self.root, None, None, true)
    }

    /// Check invariants with detailed error reporting.
    pub fn check_invariants_detailed(&self) -> Result<(), String> {
        // First check the tree structure invariants
        if !self.check_node_invariants(&self.root, None, None, true) {
            return Err("Tree invariants violated".to_string());
        }

        // Then check the linked list invariants
        self.check_linked_list_invariants()?;

        // Finally check arena-tree consistency
        self.check_arena_tree_consistency()?;

        Ok(())
    }

    /// Check that the leaf linked list is properly ordered and complete.
    fn check_linked_list_invariants(&self) -> Result<(), String> {
        // Use the iterator to get all keys
        let keys: Vec<&K> = self.keys().collect();

        // Check that keys are sorted
        for i in 1..keys.len() {
            if keys[i - 1] >= keys[i] {
                return Err(format!("Iterator returned unsorted keys at index {}", i));
            }
        }

        // Verify we got the right number of keys
        if keys.len() != self.len() {
            return Err(format!(
                "Iterator returned {} keys but tree has {} items",
                keys.len(),
                self.len()
            ));
        }

        Ok(())
    }

    /// Check arena-tree consistency: verify node counts match between tree and arena.
    fn check_arena_tree_consistency(&self) -> Result<(), String> {
        // Count nodes in the tree structure
        let (tree_leaf_count, tree_branch_count) = self.count_nodes_in_tree();

        // Get arena counts
        let arena_leaf_count = self.allocated_leaf_count();
        let arena_branch_count = self.allocated_branch_count();

        // Check leaf node consistency
        if tree_leaf_count != arena_leaf_count {
            return Err(format!(
                "Leaf count mismatch: {} in tree structure but {} allocated in arena",
                tree_leaf_count, arena_leaf_count
            ));
        }

        // Check branch node consistency
        if tree_branch_count != arena_branch_count {
            return Err(format!(
                "Branch count mismatch: {} in tree structure but {} allocated in arena",
                tree_branch_count, arena_branch_count
            ));
        }

        // Check that all leaf nodes in tree are reachable via linked list
        self.check_leaf_linked_list_completeness()?;

        Ok(())
    }

    /// Count the number of leaf and branch nodes actually in the tree structure.
    fn count_nodes_in_tree(&self) -> (usize, usize) {
        if matches!(self.root, NodeRef::Leaf(_, _)) {
            // Single leaf root
            (1, 0)
        } else {
            self.count_nodes_recursive(&self.root)
        }
    }

    /// Recursively count nodes in the tree.
    fn count_nodes_recursive(&self, node: &NodeRef<K, V>) -> (usize, usize) {
        match node {
            NodeRef::Leaf(_, _) => (1, 0), // Found a leaf
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.get_branch(*id) {
                    let mut total_leaves = 0;
                    let mut total_branches = 1; // Count this branch

                    // Recursively count in all children
                    for child in &branch.children {
                        let (child_leaves, child_branches) = self.count_nodes_recursive(child);
                        total_leaves += child_leaves;
                        total_branches += child_branches;
                    }

                    (total_leaves, total_branches)
                } else {
                    // Invalid branch reference
                    (0, 0)
                }
            }
        }
    }

    /// Check that all leaf nodes in the tree are reachable via the linked list.
    fn check_leaf_linked_list_completeness(&self) -> Result<(), String> {
        // Collect all leaf IDs from the tree structure
        let mut tree_leaf_ids = Vec::new();
        self.collect_leaf_ids(&self.root, &mut tree_leaf_ids);
        tree_leaf_ids.sort();

        // Collect all leaf IDs from the linked list traversal
        let mut linked_list_ids = Vec::new();
        if let Some(first_id) = self.get_first_leaf_id() {
            let mut current_id = Some(first_id);
            while let Some(id) = current_id {
                linked_list_ids.push(id);
                current_id = self.get_leaf(id).and_then(|leaf| {
                    if leaf.next == crate::NULL_NODE {
                        None
                    } else {
                        Some(leaf.next)
                    }
                });
            }
        }
        linked_list_ids.sort();

        // Compare the two lists
        if tree_leaf_ids != linked_list_ids {
            return Err(format!(
                "Linked list incomplete: tree has leaf IDs {:?} but linked list has {:?}",
                tree_leaf_ids, linked_list_ids
            ));
        }

        Ok(())
    }

    /// Collect all leaf node IDs from the tree structure.
    fn collect_leaf_ids(&self, node: &NodeRef<K, V>, ids: &mut Vec<NodeId>) {
        match node {
            NodeRef::Leaf(id, _) => ids.push(*id),
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.get_branch(*id) {
                    for child in &branch.children {
                        self.collect_leaf_ids(child, ids);
                    }
                }
            }
        }
    }

    /// Alias for check_invariants_detailed (for test compatibility).
    pub fn validate(&self) -> Result<(), String> {
        self.check_invariants_detailed()
    }

    /// Returns all key-value pairs as a vector (for testing/debugging).
    pub fn slice(&self) -> Vec<(&K, &V)> {
        self.items().collect()
    }

    /// Returns the sizes of all leaf nodes (for testing/debugging).
    pub fn leaf_sizes(&self) -> Vec<usize> {
        let mut sizes = Vec::new();
        self.collect_leaf_sizes(&self.root, &mut sizes);
        sizes
    }

    /// Prints the node chain for debugging.
    pub fn print_node_chain(&self) {
        println!("Tree structure:");
        self.print_node(&self.root, 0);
    }

    fn collect_leaf_sizes(&self, node: &NodeRef<K, V>, sizes: &mut Vec<usize>) {
        match node {
            NodeRef::Leaf(id, _) => {
                let size = self.get_leaf(*id).map(|leaf| leaf.keys.len()).unwrap_or(0);
                sizes.push(size);
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.get_branch(*id) {
                    for child in &branch.children {
                        self.collect_leaf_sizes(child, sizes);
                    }
                }
                // Missing arena branch contributes no leaf sizes (do nothing)
            }
        }
    }

    fn print_node(&self, node: &NodeRef<K, V>, depth: usize) {
        let indent = "  ".repeat(depth);
        match node {
            NodeRef::Leaf(id, _) => {
                if let Some(leaf) = self.get_leaf(*id) {
                    println!(
                        "{}Leaf[id={}, cap={}]: {} keys",
                        indent,
                        id,
                        leaf.capacity,
                        leaf.keys.len()
                    );
                } else {
                    println!("{}Leaf[id={}]: <missing>", indent, id);
                }
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.get_branch(*id) {
                    println!(
                        "{}Branch[id={}, cap={}]: {} keys, {} children",
                        indent,
                        id,
                        branch.capacity,
                        branch.keys.len(),
                        branch.children.len()
                    );
                    for child in &branch.children {
                        self.print_node(child, depth + 1);
                    }
                } else {
                    println!("{}Branch[id={}]: <missing>", indent, id);
                }
            }
        }
    }

    /// Recursively check invariants for a node and its children.
    fn check_node_invariants(
        &self,
        node: &NodeRef<K, V>,
        min_key: Option<&K>,
        max_key: Option<&K>,
        _is_root: bool,
    ) -> bool {
        match node {
            NodeRef::Leaf(id, _) => {
                if let Some(leaf) = self.get_leaf(*id) {
                    // Check leaf invariants
                    if leaf.keys.len() != leaf.values.len() {
                        return false; // Keys and values must have same length
                    }

                    // Check that keys are sorted
                    for i in 1..leaf.keys.len() {
                        if leaf.keys[i - 1] >= leaf.keys[i] {
                            return false; // Keys must be in ascending order
                        }
                    }

                    // Check capacity constraints
                    if leaf.keys.len() > self.capacity {
                        return false; // Node exceeds capacity
                    }

                    // Check minimum occupancy
                    if !leaf.keys.is_empty() && leaf.is_underfull() {
                        // For root nodes, allow fewer keys only if it's the only node
                        if _is_root {
                            // Root leaf can have any number of keys >= 1
                            // (This is fine for leaf roots)
                        } else {
                            return false; // Non-root leaf is underfull
                        }
                    }

                    // Check key bounds
                    if let Some(min) = min_key {
                        if !leaf.keys.is_empty() && &leaf.keys[0] < min {
                            return false; // First key must be >= min_key
                        }
                    }
                    if let Some(max) = max_key {
                        if !leaf.keys.is_empty() && &leaf.keys[leaf.keys.len() - 1] >= max {
                            return false; // Last key must be < max_key
                        }
                    }

                    true
                } else {
                    false // Missing arena leaf is invalid
                }
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.get_branch(*id) {
                    // Check branch invariants
                    if branch.keys.len() + 1 != branch.children.len() {
                        return false; // Must have one more child than keys
                    }

                    // Check that keys are sorted
                    for i in 1..branch.keys.len() {
                        if branch.keys[i - 1] >= branch.keys[i] {
                            return false; // Keys must be in ascending order
                        }
                    }

                    // Check capacity constraints
                    if branch.keys.len() > self.capacity {
                        return false; // Node exceeds capacity
                    }

                    // Check minimum occupancy
                    if !branch.keys.is_empty() && branch.is_underfull() {
                        if _is_root {
                            // Root branch can have any number of keys >= 1 (as long as it has children)
                            // The only requirement is that keys.len() + 1 == children.len()
                            // This is already checked above, so root branches are always valid
                        } else {
                            return false; // Non-root branch is underfull
                        }
                    }

                    // Check that branch has at least one child
                    if branch.children.is_empty() {
                        return false; // Branch must have at least one child
                    }

                    // Check children recursively
                    for (i, child) in branch.children.iter().enumerate() {
                        let child_min = if i == 0 {
                            min_key
                        } else {
                            Some(&branch.keys[i - 1])
                        };
                        let child_max = if i == branch.keys.len() {
                            max_key
                        } else {
                            Some(&branch.keys[i])
                        };

                        if !self.check_node_invariants(child, child_min, child_max, false) {
                            return false;
                        }
                    }

                    true
                } else {
                    false // Missing arena branch is invalid
                }
            }
        }
    }
}

impl<K: Ord + Clone, V: Clone> Default for BPlusTreeMap<K, V> {
    /// Create a B+ tree with default capacity (16).
    fn default() -> Self {
        Self::new(16).expect("Default capacity should be valid")
    }
}

/// Leaf node containing key-value pairs.
#[derive(Debug, Clone)]
pub struct LeafNode<K, V> {
    /// Maximum number of keys this node can hold.
    capacity: usize,
    /// Sorted list of keys.
    keys: Vec<K>,
    /// List of values corresponding to keys.
    values: Vec<V>,
    /// Next leaf node in the linked list (for range queries).
    next: NodeId,
}

/// Internal (branch) node containing keys and child pointers.
#[derive(Debug, Clone)]
pub struct BranchNode<K, V> {
    /// Maximum number of keys this node can hold.
    capacity: usize,
    /// Sorted list of separator keys.
    keys: Vec<K>,
    /// List of child nodes (leaves or other branches).
    children: Vec<NodeRef<K, V>>,
}

impl<K: Ord + Clone, V: Clone> LeafNode<K, V> {
    // ============================================================================
    // CONSTRUCTION
    // ============================================================================

    /// Creates a new leaf node with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            keys: Vec::new(),
            values: Vec::new(),
            next: NULL_NODE,
        }
    }

    /// Get a reference to the keys in this leaf node.
    pub fn keys(&self) -> &Vec<K> {
        &self.keys
    }

    /// Get a reference to the values in this leaf node.
    pub fn values(&self) -> &Vec<V> {
        &self.values
    }

    /// Get a mutable reference to the values in this leaf node.
    pub fn values_mut(&mut self) -> &mut Vec<V> {
        &mut self.values
    }

    // ============================================================================
    // GET OPERATIONS
    // ============================================================================

    /// Get value for a key from this leaf node.
    pub fn get(&self, key: &K) -> Option<&V> {
        match self.keys.binary_search(key) {
            Ok(index) => Some(&self.values[index]),
            Err(_) => None,
        }
    }

    /// Get a mutable reference to the value for a key from this leaf node.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.keys.binary_search(key) {
            Ok(index) => Some(&mut self.values[index]),
            Err(_) => None,
        }
    }

    // ============================================================================
    // HELPERS FOR GET OPERATIONS
    // ============================================================================
    // (No additional helpers needed for LeafNode get operations)

    // ============================================================================
    // INSERT OPERATIONS
    // ============================================================================

    /// Insert a key-value pair and handle splitting if necessary.
    pub fn insert(&mut self, key: K, value: V) -> InsertResult<K, V> {
        // Do binary search once and use the result throughout
        match self.keys.binary_search(&key) {
            Ok(index) => {
                // Key already exists, update the value
                let old_value = std::mem::replace(&mut self.values[index], value);
                InsertResult::Updated(Some(old_value))
            }
            Err(index) => {
                // Key doesn't exist, need to insert
                // Check if split is needed BEFORE inserting
                if !self.is_full() {
                    // Room to insert without splitting
                    self.insert_at_index(index, key, value);
                    // Simple insertion - no split needed
                    return InsertResult::Updated(None);
                }
                // Leaf is at capacity, split first then insert
                let mut new_leaf_data = self.split();
                let separator_key = new_leaf_data.keys[0].clone();

                // Determine which leaf should receive the new key
                if key < separator_key {
                    // Insert into the current (left) leaf
                    self.insert_at_index(index, key, value);
                } else {
                    // Insert into the new (right) leaf
                    match new_leaf_data.keys.binary_search(&key) {
                        Ok(_) => {
                            // This should never happen with correct split logic
                            // Return error instead of panic to maintain stability
                            return InsertResult::Error(BPlusTreeError::DataIntegrityError(
                                "Key unexpectedly found in new leaf after split".to_string(),
                            ));
                        }
                        Err(new_index) => {
                            new_leaf_data.insert_at_index(new_index, key, value);
                        }
                    }
                }

                // Return the leaf data for arena allocation
                InsertResult::Split {
                    old_value: None,
                    new_node_data: SplitNodeData::Leaf(new_leaf_data),
                    separator_key,
                }
            }
        }
    }

    // ============================================================================
    // HELPERS FOR INSERT OPERATIONS
    // ============================================================================

    /// Insert a key-value pair at the specified index.
    fn insert_at_index(&mut self, index: usize, key: K, value: V) {
        self.keys.insert(index, key);
        self.values.insert(index, value);
    }

    /// Split this leaf node, returning the new right node.
    pub fn split(&mut self) -> LeafNode<K, V> {
        // For B+ trees, we need to ensure both resulting nodes have at least min_keys
        // When splitting a full node (capacity keys), we want to distribute them
        // so that both nodes have at least min_keys
        let min_keys = self.min_keys();
        let total_keys = self.keys.len();

        // Calculate split point for better balance while ensuring both sides have at least min_keys
        // Use a more balanced split: aim for roughly equal distribution
        let mid = total_keys.div_ceil(2); // Round up for odd numbers

        // Ensure the split point respects minimum requirements
        let mid = mid.max(min_keys).min(total_keys - min_keys);

        // Verify this split is valid
        debug_assert!(mid >= min_keys, "Left side would be underfull");
        debug_assert!(
            total_keys - mid >= min_keys,
            "Right side would be underfull"
        );

        // Create new leaf for right half (no Box allocation)
        let mut new_leaf = LeafNode::new(self.capacity);

        // Move right half of keys/values to new leaf
        new_leaf.keys = self.keys.split_off(mid);
        new_leaf.values = self.values.split_off(mid);

        // Maintain the linked list: new leaf inherits our next pointer
        new_leaf.next = self.next;
        // Note: The caller must update self.next to point to the new leaf's ID
        // This can't be done here as we don't know the new leaf's arena ID yet

        new_leaf
    }

    // ============================================================================
    // DELETE OPERATIONS
    // ============================================================================

    /// Remove a key from this leaf node.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        match self.keys.binary_search(key) {
            Ok(index) => {
                self.keys.remove(index);
                Some(self.values.remove(index))
            }
            Err(_) => None,
        }
    }

    // ============================================================================
    // OTHER API OPERATIONS
    // ============================================================================

    /// Returns the number of key-value pairs in this leaf node.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns true if this leaf node is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Returns true if this leaf node is at capacity.
    pub fn is_full(&self) -> bool {
        self.keys.len() >= self.capacity
    }

    /// Returns true if this leaf node needs to be split.
    /// We allow one extra key beyond capacity to ensure proper splitting.
    pub fn needs_split(&self) -> bool {
        self.keys.len() > self.capacity
    }

    /// Returns true if this leaf node is underfull (below minimum occupancy).
    pub fn is_underfull(&self) -> bool {
        self.keys.len() < self.min_keys()
    }

    /// Returns true if this leaf can donate a key to a sibling.
    pub fn can_donate(&self) -> bool {
        self.keys.len() > self.min_keys()
    }

    // ============================================================================
    // OTHER HELPERS
    // ============================================================================

    /// Returns the minimum number of keys this leaf should have.
    pub fn min_keys(&self) -> usize {
        // For leaf nodes, minimum is floor(capacity / 2)
        // Exception: root can have fewer keys
        self.capacity / 2
    }

    // ============================================================================
    // BORROWING AND MERGING HELPERS
    // ============================================================================

    /// Borrow the last key-value pair from this leaf (used when this is the left sibling)
    pub fn borrow_last(&mut self) -> Option<(K, V)> {
        if self.keys.is_empty() || !self.can_donate() {
            return None;
        }
        Some((self.keys.pop().unwrap(), self.values.pop().unwrap()))
    }

    /// Borrow the first key-value pair from this leaf (used when this is the right sibling)
    pub fn borrow_first(&mut self) -> Option<(K, V)> {
        if self.keys.is_empty() || !self.can_donate() {
            return None;
        }
        Some((self.keys.remove(0), self.values.remove(0)))
    }

    /// Accept a borrowed key-value pair at the beginning (from left sibling)
    pub fn accept_from_left(&mut self, key: K, value: V) {
        self.keys.insert(0, key);
        self.values.insert(0, value);
    }

    /// Accept a borrowed key-value pair at the end (from right sibling)
    pub fn accept_from_right(&mut self, key: K, value: V) {
        self.keys.push(key);
        self.values.push(value);
    }

    /// Merge all content from another leaf into this one, returning the other's next pointer
    pub fn merge_from(&mut self, other: &mut LeafNode<K, V>) -> NodeId {
        self.keys.append(&mut other.keys);
        self.values.append(&mut other.values);
        let other_next = other.next;
        other.next = NULL_NODE; // Clear the other's next pointer
        other_next
    }

    /// Extract all content from this leaf (used for merging)
    pub fn extract_all(&mut self) -> (Vec<K>, Vec<V>, NodeId) {
        let keys = std::mem::take(&mut self.keys);
        let values = std::mem::take(&mut self.values);
        let next = self.next;
        self.next = NULL_NODE;
        (keys, values, next)
    }
}

impl<K: Ord + Clone, V: Clone> BranchNode<K, V> {
    // ============================================================================
    // CONSTRUCTION
    // ============================================================================

    /// Creates a new branch node with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            keys: Vec::new(),
            children: Vec::new(),
        }
    }

    // ============================================================================
    // GET OPERATIONS
    // ============================================================================

    /// Get the child node for a given key.
    pub fn get_child(&self, key: &K) -> Option<&NodeRef<K, V>> {
        let child_index = self.find_child_index(key);
        if child_index < self.children.len() {
            Some(&self.children[child_index])
        } else {
            None
        }
    }

    /// Get a mutable reference to the child node for a given key.
    pub fn get_child_mut(&mut self, key: &K) -> Option<&mut NodeRef<K, V>> {
        let child_index = self.find_child_index(key);
        if child_index >= self.children.len() {
            return None; // Invalid child index
        }
        Some(&mut self.children[child_index])
    }

    // ============================================================================
    // HELPERS FOR GET OPERATIONS
    // ============================================================================

    /// Find the child index where the given key should be located.
    pub fn find_child_index(&self, key: &K) -> usize {
        // Binary search to find the appropriate child
        match self.keys.binary_search(key) {
            Ok(index) => index + 1, // Key found, go to right child
            Err(index) => index,    // Key not found, insert position is the child index
        }
    }

    // ============================================================================
    // INSERT OPERATIONS
    // ============================================================================

    /// Insert a separator key and new child into this branch node.
    /// Returns None if no split needed, or Some((new_branch_data, promoted_key)) if split occurred.
    /// The caller should handle arena allocation for the split data.
    pub fn insert_child_and_split_if_needed(
        &mut self,
        child_index: usize,
        separator_key: K,
        new_child: NodeRef<K, V>,
    ) -> Option<(BranchNode<K, V>, K)> {
        // Check if split is needed BEFORE inserting
        if self.is_full() {
            // Branch is at capacity, need to handle split
            // For branches, we MUST insert first because split promotes a key
            // With capacity=4: 4 keys → split needs 5 keys (2 left + 1 promoted + 2 right)
            self.keys.insert(child_index, separator_key);
            self.children.insert(child_index + 1, new_child);
            // Return raw data - caller should allocate through arena
            Some(self.split_data())
        } else {
            // Room to insert without splitting
            self.keys.insert(child_index, separator_key);
            self.children.insert(child_index + 1, new_child);
            None
        }
    }

    // ============================================================================
    // HELPERS FOR INSERT OPERATIONS
    // ============================================================================

    /// Split this branch node, returning the new right node and promoted key.
    pub fn split_data(&mut self) -> (BranchNode<K, V>, K) {
        // For branch nodes, we need to ensure both resulting nodes have at least min_keys
        // The middle key gets promoted, so we need at least min_keys on each side
        let min_keys = self.min_keys();
        let total_keys = self.keys.len();

        // For branch splits, we promote the middle key, so we need:
        // - Left side: min_keys keys
        // - Middle: 1 key (promoted)
        // - Right side: min_keys keys
        // Total needed: min_keys + 1 + min_keys
        let mid = min_keys;

        // Verify this split is valid
        debug_assert!(mid < total_keys, "Not enough keys to promote one");
        debug_assert!(mid >= min_keys, "Left side would be underfull");
        debug_assert!(total_keys - mid > min_keys, "Right side would be underfull");

        // The middle key gets promoted to the parent
        let promoted_key = self.keys[mid].clone();

        let mut right_half = BranchNode::new(self.capacity);
        right_half.keys = self.keys.split_off(mid + 1);
        right_half.children = self.children.split_off(mid + 1);
        self.keys.truncate(mid); // Remove the promoted key from left side

        (right_half, promoted_key)
    }

    // ============================================================================
    // HELPERS FOR DELETE OPERATIONS
    // ============================================================================

    /// Merge this branch with the right sibling using the given separator.
    /// Returns true if merge was successful.
    pub fn merge_with_right(&mut self, mut right: BranchNode<K, V>, separator: K) -> bool {
        // Add the separator key
        self.keys.push(separator);

        // Move all keys and children from right to this node
        self.keys.append(&mut right.keys);
        self.children.append(&mut right.children);

        true
    }

    // ============================================================================
    // OTHER API OPERATIONS
    // ============================================================================

    /// Returns true if this branch node is at capacity.
    pub fn is_full(&self) -> bool {
        self.keys.len() >= self.capacity
    }

    /// Returns true if this branch node needs to be split.
    /// We allow one extra key beyond capacity to ensure proper splitting.
    pub fn needs_split(&self) -> bool {
        self.keys.len() > self.capacity
    }

    /// Returns true if this branch node is underfull (below minimum occupancy).
    pub fn is_underfull(&self) -> bool {
        self.keys.len() < self.min_keys()
    }

    /// Returns true if this branch can donate a key to a sibling.
    pub fn can_donate(&self) -> bool {
        self.keys.len() > self.min_keys()
    }

    // ============================================================================
    // OTHER HELPERS
    // ============================================================================

    /// Returns the minimum number of keys this branch should have.
    pub fn min_keys(&self) -> usize {
        // For branch nodes, minimum is floor(capacity / 2)
        // Exception: root can have fewer keys
        self.capacity / 2
    }

    // ============================================================================
    // BORROWING AND MERGING HELPERS
    // ============================================================================

    /// Borrow the last key and child from this branch (used when this is the left sibling)
    pub fn borrow_last(&mut self) -> Option<(K, NodeRef<K, V>)> {
        if self.keys.is_empty() || !self.can_donate() {
            return None;
        }
        let key = self.keys.pop()?;
        let child = self.children.pop()?;
        Some((key, child))
    }

    /// Borrow the first key and child from this branch (used when this is the right sibling)
    pub fn borrow_first(&mut self) -> Option<(K, NodeRef<K, V>)> {
        if self.keys.is_empty() || !self.can_donate() {
            return None;
        }
        let key = self.keys.remove(0);
        let child = self.children.remove(0);
        Some((key, child))
    }

    /// Accept a borrowed key and child at the beginning (from left sibling)
    /// The separator becomes the first key, and the moved child becomes the first child
    pub fn accept_from_left(
        &mut self,
        separator: K,
        moved_key: K,
        moved_child: NodeRef<K, V>,
    ) -> K {
        self.keys.insert(0, separator);
        self.children.insert(0, moved_child);
        moved_key // Return the new separator for parent
    }

    /// Accept a borrowed key and child at the end (from right sibling)
    /// The separator becomes the last key, and the moved child becomes the last child
    pub fn accept_from_right(
        &mut self,
        separator: K,
        moved_key: K,
        moved_child: NodeRef<K, V>,
    ) -> K {
        self.keys.push(separator);
        self.children.push(moved_child);
        moved_key // Return the new separator for parent
    }

    /// Merge all content from another branch into this one, with separator from parent
    pub fn merge_from(&mut self, separator: K, other: &mut BranchNode<K, V>) {
        // Add separator key from parent
        self.keys.push(separator);
        // Add all keys and children from other
        self.keys.append(&mut other.keys);
        self.children.append(&mut other.children);
    }
}

/// Iterator over key-value pairs in the B+ tree using the leaf linked list.
pub struct ItemIterator<'a, K, V> {
    tree: &'a BPlusTreeMap<K, V>,
    current_leaf_id: Option<NodeId>,
    current_leaf_index: usize,
    end_key: Option<&'a K>,
    end_bound_key: Option<K>,
    end_inclusive: bool,
    finished: bool,
}

impl<'a, K: Ord + Clone, V: Clone> ItemIterator<'a, K, V> {
    fn new(tree: &'a BPlusTreeMap<K, V>) -> Self {
        // Start with the first (leftmost) leaf in the tree
        let leftmost_id = tree.get_first_leaf_id();

        Self {
            tree,
            current_leaf_id: leftmost_id,
            current_leaf_index: 0,
            end_key: None,
            end_bound_key: None,
            end_inclusive: false,
            finished: false,
        }
    }

    /// NEW: Start from specific leaf and index position, optionally bounded by end key
    fn new_from_position(
        tree: &'a BPlusTreeMap<K, V>,
        start_leaf_id: NodeId,
        start_index: usize,
        end_key: Option<&'a K>,
    ) -> Self {
        Self {
            tree,
            current_leaf_id: Some(start_leaf_id),
            current_leaf_index: start_index,
            end_key,
            end_bound_key: None,
            end_inclusive: false,
            finished: false,
        }
    }

    /// Start from specific position with full bound control using owned keys
    fn new_from_position_with_bounds(
        tree: &'a BPlusTreeMap<K, V>,
        start_leaf_id: NodeId,
        start_index: usize,
        end_bound: Bound<&K>,
    ) -> Self {
        let (end_bound_key, end_inclusive) = match end_bound {
            Bound::Included(key) => (Some(key.clone()), true),
            Bound::Excluded(key) => (Some(key.clone()), false),
            Bound::Unbounded => (None, false),
        };

        Self {
            tree,
            current_leaf_id: Some(start_leaf_id),
            current_leaf_index: start_index,
            end_key: None,
            end_bound_key,
            end_inclusive,
            finished: false,
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> Iterator for ItemIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            if let Some(leaf_id) = self.current_leaf_id {
                if let Some(leaf) = self.tree.get_leaf(leaf_id) {
                    // Check if we have more items in the current leaf
                    if self.current_leaf_index < leaf.keys.len() {
                        let key = &leaf.keys[self.current_leaf_index];
                        let value = &leaf.values[self.current_leaf_index];

                        // Check if we've reached the end bound
                        if let Some(end) = self.end_key {
                            if key >= end {
                                self.finished = true;
                                return None;
                            }
                        } else if let Some(ref end) = self.end_bound_key {
                            if self.end_inclusive {
                                if key > end {
                                    self.finished = true;
                                    return None;
                                }
                            } else if key >= end {
                                self.finished = true;
                                return None;
                            }
                        }

                        self.current_leaf_index += 1;
                        return Some((key, value));
                    } else {
                        // Move to next leaf
                        self.current_leaf_id = if leaf.next != NULL_NODE {
                            Some(leaf.next)
                        } else {
                            None
                        };
                        self.current_leaf_index = 0;
                        // Continue loop to try next leaf
                    }
                } else {
                    // Invalid leaf ID
                    self.finished = true;
                    return None;
                }
            } else {
                // No more leaves
                self.finished = true;
                return None;
            }
        }
    }
}

/// Iterator over keys in the B+ tree.
pub struct KeyIterator<'a, K, V> {
    items: ItemIterator<'a, K, V>,
}

impl<'a, K: Ord + Clone, V: Clone> KeyIterator<'a, K, V> {
    fn new(tree: &'a BPlusTreeMap<K, V>) -> Self {
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

/// Iterator over values in the B+ tree.
pub struct ValueIterator<'a, K, V> {
    items: ItemIterator<'a, K, V>,
}

impl<'a, K: Ord + Clone, V: Clone> ValueIterator<'a, K, V> {
    fn new(tree: &'a BPlusTreeMap<K, V>) -> Self {
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

/// Optimized iterator over a range of key-value pairs in the B+ tree.
/// Uses tree navigation to find start, then linked list traversal for efficiency.
pub struct RangeIterator<'a, K, V> {
    iterator: Option<ItemIterator<'a, K, V>>,
    skip_first: bool,
    first_key: Option<K>,
}

impl<'a, K: Ord + Clone, V: Clone> RangeIterator<'a, K, V> {
    fn new(tree: &'a BPlusTreeMap<K, V>, start_key: Option<&K>, end_key: Option<&'a K>) -> Self {
        let iterator = if let Some(start) = start_key {
            // Find the starting position using tree navigation
            if let Some((leaf_id, index)) = tree.find_range_start(start) {
                Some(ItemIterator::new_from_position(
                    tree, leaf_id, index, end_key,
                ))
            } else {
                None // No items in range
            }
        } else {
            // Start from beginning
            tree.get_first_leaf_id()
                .map(|first_leaf| ItemIterator::new_from_position(tree, first_leaf, 0, end_key))
        };

        Self {
            iterator,
            skip_first: false,
            first_key: None,
        }
    }

    fn new_with_skip_owned(
        tree: &'a BPlusTreeMap<K, V>,
        start_info: Option<(NodeId, usize)>,
        skip_first: bool,
        end_info: Option<(K, bool)>, // (end_key, is_inclusive)
    ) -> Self {
        let (iterator, first_key) = if let Some((leaf_id, index)) = start_info {
            // Create iterator with appropriate end bound
            let end_bound = match end_info {
                Some((ref key, true)) => Bound::Included(key),
                Some((ref key, false)) => Bound::Excluded(key),
                None => Bound::Unbounded,
            };

            let iter = ItemIterator::new_from_position_with_bounds(tree, leaf_id, index, end_bound);

            // If we need to skip first, we need to know what key to skip
            let first_key = if skip_first {
                tree.get_leaf(leaf_id)
                    .and_then(|leaf| leaf.keys.get(index))
                    .cloned()
            } else {
                None
            };

            (Some(iter), first_key)
        } else {
            (None, None)
        };

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
