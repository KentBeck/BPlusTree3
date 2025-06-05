//! B+ Tree implementation in Rust with dict-like API.
//!
//! This module provides a B+ tree data structure with a dictionary-like interface,
//! supporting efficient insertion, deletion, lookup, and range queries.

use std::marker::PhantomData;

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
/// use bplustree3::BPlusTreeMap;
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
/// let range: Vec<_> = tree.range(Some(&1), Some(&3)).collect();
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

    // Arena-based allocation for leaf nodes
    /// Arena storage for leaf nodes.
    leaf_arena: Vec<Option<LeafNode<K, V>>>,
    /// Free leaf node IDs available for reuse.
    free_leaf_ids: Vec<NodeId>,

    // Arena-based allocation for branch nodes
    /// Arena storage for branch nodes.
    branch_arena: Vec<Option<BranchNode<K, V>>>,
    /// Free branch node IDs available for reuse.
    free_branch_ids: Vec<NodeId>,
}

/// Node reference that can be either a leaf or branch node
#[derive(Debug, Clone)]
pub enum NodeRef<K, V> {
    Leaf(NodeId, PhantomData<(K, V)>),
    Branch(NodeId, PhantomData<(K, V)>),
}

/// Information about a node's siblings in its parent
#[derive(Debug)]
struct SiblingInfo<K, V> {
    left_sibling: Option<NodeRef<K, V>>,
    right_sibling: Option<NodeRef<K, V>>,
    // Note: separator indices can be added later if needed
}

/// Direction for borrowing operations
#[derive(Debug, Clone, Copy)]
enum BorrowDirection {
    FromLeft,
    FromRight,
}

/// Direction for merge operations
#[derive(Debug, Clone, Copy)]
enum MergeDirection {
    WithLeft,
    WithRight,
}

impl<K, V> SiblingInfo<K, V> {
    fn has_left(&self) -> bool {
        self.left_sibling.is_some()
    }

    fn has_right(&self) -> bool {
        self.right_sibling.is_some()
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
    /// use bplustree3::BPlusTreeMap;
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
        let mut leaf_arena = Vec::new();
        leaf_arena.push(Some(LeafNode::new(capacity))); // First leaf at id=0

        // Initialize branch arena (starts empty)
        let branch_arena = Vec::new();

        Ok(Self {
            capacity,
            root: NodeRef::Leaf(0, PhantomData), // Root points to the arena leaf at id=0
            // Initialize arena storage
            leaf_arena,
            free_leaf_ids: Vec::new(),
            branch_arena,
            free_branch_ids: Vec::new(),
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
    /// use bplustree3::BPlusTreeMap;
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
        match &self.root {
            NodeRef::Leaf(id, _) => {
                let id = *id;
                if let Some(leaf) = self.get_leaf_mut(id) {
                    return leaf.get_mut(key);
                }
                None
            }
            NodeRef::Branch(id, _) => {
                let id = *id;
                self.get_mut_in_branch(id, key)
            }
        }
    }

    /// Get mutable reference in an arena branch
    fn get_mut_in_branch(&mut self, branch_id: NodeId, key: &K) -> Option<&mut V> {
        // Use helper to get child info
        let (_child_index, child_ref) = self.get_child_info(branch_id, key)?;

        // Traverse to child
        match child_ref {
            NodeRef::Leaf(leaf_id, _) => {
                if let Some(leaf) = self.get_leaf_mut(leaf_id) {
                    leaf.get_mut(key)
                } else {
                    None
                }
            }
            NodeRef::Branch(child_branch_id, _) => self.get_mut_in_branch(child_branch_id, key),
        }
    }

    // ============================================================================
    // HELPERS FOR GET OPERATIONS
    // ============================================================================

    /// Get child index and reference for a given key
    fn get_child_info(&self, branch_id: NodeId, key: &K) -> Option<(usize, NodeRef<K, V>)> {
        let branch = self.get_branch(branch_id)?;
        let child_index = branch.find_child_index(key);
        if child_index < branch.children.len() {
            Some((child_index, branch.children[child_index].clone()))
        } else {
            None
        }
    }

    /// Get child at specific index
    fn get_child_at(&self, branch_id: NodeId, index: usize) -> Option<NodeRef<K, V>> {
        self.get_branch(branch_id)
            .and_then(|branch| branch.children.get(index).cloned())
    }

    /// Get comprehensive sibling information for a child
    fn get_sibling_info(&self, parent_id: NodeId, child_index: usize) -> Option<SiblingInfo<K, V>> {
        let parent = self.get_branch(parent_id)?;
        Some(SiblingInfo {
            left_sibling: (child_index > 0).then(|| parent.children[child_index - 1].clone()),
            right_sibling: parent.children.get(child_index + 1).cloned(),
        })
    }

    /// Check if any node type is underfull
    fn is_node_underfull(&self, node_ref: &NodeRef<K, V>) -> bool {
        match node_ref {
            NodeRef::Leaf(id, _) => self.get_leaf(*id).map_or(false, |n| n.is_underfull()),
            NodeRef::Branch(id, _) => self.get_branch(*id).map_or(false, |n| n.is_underfull()),
        }
    }

    /// Check if any node type can donate
    fn can_node_donate(&self, node_ref: &NodeRef<K, V>) -> bool {
        match node_ref {
            NodeRef::Leaf(id, _) => self.get_leaf(*id).map_or(false, |n| n.can_donate()),
            NodeRef::Branch(id, _) => self.get_branch(*id).map_or(false, |n| n.can_donate()),
        }
    }

    /// Get node length (number of keys)
    fn node_len(&self, node_ref: &NodeRef<K, V>) -> usize {
        match node_ref {
            NodeRef::Leaf(id, _) => self.get_leaf(*id).map_or(0, |n| n.keys.len()),
            NodeRef::Branch(id, _) => self.get_branch(*id).map_or(0, |n| n.keys.len()),
        }
    }

    /// Check if two nodes can be merged
    fn can_merge_nodes(&self, left: &NodeRef<K, V>, right: &NodeRef<K, V>) -> bool {
        match (left, right) {
            (NodeRef::Leaf(l_id, _), NodeRef::Leaf(r_id, _)) => {
                let left_len = self.get_leaf(*l_id).map_or(0, |n| n.keys.len());
                let right_len = self.get_leaf(*r_id).map_or(0, |n| n.keys.len());
                left_len + right_len <= self.capacity
            }
            (NodeRef::Branch(l_id, _), NodeRef::Branch(r_id, _)) => {
                let left_len = self.get_branch(*l_id).map_or(0, |n| n.keys.len());
                let right_len = self.get_branch(*r_id).map_or(0, |n| n.keys.len());
                left_len + 1 + right_len <= self.capacity // +1 for separator
            }
            _ => false,
        }
    }

    /// Extract all data from a leaf node
    fn take_leaf_data(&mut self, leaf_id: NodeId) -> Option<(Vec<K>, Vec<V>, NodeId)> {
        self.get_leaf_mut(leaf_id).map(|leaf| {
            (
                std::mem::take(&mut leaf.keys),
                std::mem::take(&mut leaf.values),
                leaf.next,
            )
        })
    }

    /// Extract all data from a branch node
    fn take_branch_data(&mut self, branch_id: NodeId) -> Option<(Vec<K>, Vec<NodeRef<K, V>>)> {
        self.get_branch_mut(branch_id).map(|branch| {
            (
                std::mem::take(&mut branch.keys),
                std::mem::take(&mut branch.children),
            )
        })
    }

    /// Update leaf linked list pointer
    fn update_leaf_link(&mut self, from_id: NodeId, to_id: NodeId) -> bool {
        self.get_leaf_mut(from_id)
            .map(|leaf| { leaf.next = to_id; true })
            .unwrap_or(false)
    }

    /// Execute a function with a leaf node reference, returning a default value if node doesn't exist
    fn with_leaf<T, F>(&self, id: NodeId, f: F) -> Option<T>
    where
        F: FnOnce(&LeafNode<K, V>) -> T,
    {
        self.get_leaf(id).map(f)
    }

    /// Execute a function with a mutable leaf node reference, returning a default value if node doesn't exist
    fn with_leaf_mut<T, F>(&mut self, id: NodeId, f: F) -> Option<T>
    where
        F: FnOnce(&mut LeafNode<K, V>) -> T,
    {
        self.get_leaf_mut(id).map(f)
    }

    /// Execute a function with a branch node reference, returning a default value if node doesn't exist
    fn with_branch<T, F>(&self, id: NodeId, f: F) -> Option<T>
    where
        F: FnOnce(&BranchNode<K, V>) -> T,
    {
        self.get_branch(id).map(f)
    }

    /// Execute a function with a mutable branch node reference, returning a default value if node doesn't exist
    fn with_branch_mut<T, F>(&mut self, id: NodeId, f: F) -> Option<T>
    where
        F: FnOnce(&mut BranchNode<K, V>) -> T,
    {
        self.get_branch_mut(id).map(f)
    }



    /// Try to remove a key from a node, returning the removed value if successful
    fn try_remove_from_node(&mut self, node_ref: &NodeRef<K, V>, key: &K) -> Option<V> {
        match node_ref {
            NodeRef::Leaf(id, _) => {
                self.get_leaf_mut(*id).and_then(|leaf| leaf.remove(key))
            }
            NodeRef::Branch(_id, _) => {
                // Branches don't directly contain values
                None
            }
        }
    }

    /// Extract node IDs from two adjacent children (for branch operations)
    fn get_branch_sibling_ids(&self, parent_id: NodeId, child_index: usize, is_left: bool) -> Option<(NodeId, NodeId, K)> {
        let parent = self.get_branch(parent_id)?;
        if is_left {
            // Left sibling case: (left_id, child_id, separator)
            if child_index > 0 {
                if let (NodeRef::Branch(left, _), NodeRef::Branch(child, _)) = (
                    &parent.children[child_index - 1],
                    &parent.children[child_index],
                ) {
                    Some((*left, *child, parent.keys[child_index - 1].clone()))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            // Right sibling case: (child_id, right_id, separator)
            if child_index + 1 < parent.children.len() {
                if let (NodeRef::Branch(child, _), NodeRef::Branch(right, _)) = (
                    &parent.children[child_index],
                    &parent.children[child_index + 1],
                ) {
                    Some((*child, *right, parent.keys[child_index].clone()))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    /// Extract node IDs from two adjacent children (for leaf operations)
    fn get_leaf_sibling_ids(&self, parent_id: NodeId, child_index: usize, is_left: bool) -> Option<(NodeId, NodeId)> {
        let parent = self.get_branch(parent_id)?;
        if is_left {
            // Left sibling case: (left_id, child_id)
            if child_index > 0 {
                if let (NodeRef::Leaf(left, _), NodeRef::Leaf(child, _)) = (
                    &parent.children[child_index - 1],
                    &parent.children[child_index],
                ) {
                    Some((*left, *child))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            // Right sibling case: (child_id, right_id)
            if child_index + 1 < parent.children.len() {
                if let (NodeRef::Leaf(child, _), NodeRef::Leaf(right, _)) = (
                    &parent.children[child_index],
                    &parent.children[child_index + 1],
                ) {
                    Some((*child, *right))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    fn get_recursive<'a>(&'a self, node: &'a NodeRef<K, V>, key: &K) -> Option<&'a V> {
        match node {
            NodeRef::Leaf(id, _) => {
                if let Some(leaf) = self.get_leaf(*id) {
                    leaf.get(key)
                } else {
                    None
                }
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = self.get_branch(*id) {
                    if let Some(child) = branch.get_child(key) {
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
    /// use bplustree3::BPlusTreeMap;
    ///
    /// let mut tree = BPlusTreeMap::new(16).unwrap();
    /// assert_eq!(tree.insert(1, "first"), None);
    /// assert_eq!(tree.insert(1, "second"), Some("first"));
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let _capacity = self.capacity;

        // Special handling for Leaf root
        if let NodeRef::Leaf(id, _) = &self.root {
            let id = *id;
            if let Some(leaf) = self.get_leaf_mut(id) {
                let result = leaf.insert(key.clone(), value.clone());
                match result {
                    InsertResult::Updated(old_value) => return old_value,
                    InsertResult::Split {
                        old_value,
                        new_node_data,
                        separator_key,
                    } => {
                        match new_node_data {
                            SplitNodeData::Leaf(new_leaf_data) => {
                                // Allocate the new leaf in arena
                                let new_id = self.allocate_leaf(new_leaf_data);

                                // Update linked list pointers for root leaf split
                                self.with_leaf_mut(id, |original_leaf| {
                                    original_leaf.next = new_id;
                                });

                                let new_node_ref = NodeRef::Leaf(new_id, PhantomData);
                                let new_root = self.new_root(new_node_ref, separator_key);
                                // Allocate new root in branch arena
                                let root_id = self.allocate_branch(new_root);
                                self.root = NodeRef::Branch(root_id, PhantomData);
                                return old_value;
                            }
                            SplitNodeData::Branch(_) => {
                                // This should never happen - a leaf can only split into a leaf
                                unreachable!("Leaf node cannot return Branch split data");
                            }
                        }
                    }
                }
            }
        }

        // Special handling for Branch root
        if let NodeRef::Branch(id, _) = &self.root {
            let id = *id;

            // Use helper to get child info
            let (child_index, child_ref) = match self.get_child_info(id, &key) {
                Some(info) => info,
                None => return None, // Invalid branch or child
            };

            // Now handle the insert based on child type
            let child_result = match child_ref {
                NodeRef::Leaf(child_id, _) => {
                    // Handle arena leaf child - need to access leaf arena
                    if let Some(result) = self.with_leaf_mut(child_id, |leaf| leaf.insert(key, value)) {
                        result
                    } else {
                        return None; // Invalid child
                    }
                }
                NodeRef::Branch(_, _) => {
                    // For deeper trees, recursively insert with arena access
                    self.insert_recursive(&child_ref, key, value)
                }
            };

            match child_result {
                InsertResult::Updated(old_value) => return old_value,
                InsertResult::Split {
                    old_value,
                    new_node_data,
                    separator_key,
                } => {
                    // Allocate the new node based on its type
                    let new_node_ref = match new_node_data {
                        SplitNodeData::Leaf(new_leaf_data) => {
                            let new_id = self.allocate_leaf(new_leaf_data);

                            // Update linked list pointers for leaf splits
                            if let NodeRef::Leaf(original_id, _) = child_ref {
                                // Update the original leaf's next pointer to point to the new leaf
                                self.with_leaf_mut(original_id, |original_leaf| {
                                    original_leaf.next = new_id;
                                });
                            }

                            NodeRef::Leaf(new_id, PhantomData)
                        }
                        SplitNodeData::Branch(new_branch_data) => {
                            let new_id = self.allocate_branch(new_branch_data);
                            NodeRef::Branch(new_id, PhantomData)
                        }
                    };

                    // Now update the branch using the proper method
                    if let Some(branch) = self.get_branch_mut(id) {
                        if let Some((new_branch_data, promoted_key)) = branch
                            .insert_child_and_split_if_needed(
                                child_index,
                                separator_key,
                                new_node_ref,
                            )
                        {
                            // Branch split, allocate new branch through arena and create new root
                            let new_branch_id = self.allocate_branch(new_branch_data);
                            let new_branch_ref = NodeRef::Branch(new_branch_id, PhantomData);
                            let new_root = self.new_root(new_branch_ref, promoted_key);
                            let root_id = self.allocate_branch(new_root);
                            self.root = NodeRef::Branch(root_id, PhantomData);
                        }
                        // If no split, the child was inserted successfully
                    }

                    return old_value;
                }
            }
        }

        // No fallback needed - all nodes are arena-based
        None
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
        new_root
    }

    // ============================================================================
    // DELETE OPERATIONS
    // ============================================================================

    /// Remove a key from the tree.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        // Special handling for Leaf root
        if let NodeRef::Leaf(id, _) = &self.root {
            let id = *id;
            return self.with_leaf_mut(id, |leaf| leaf.remove(key)).flatten();
        }

        // Special handling for Branch root
        if let NodeRef::Branch(id, _) = &self.root {
            let removed = self.remove_from_branch(*id, key);

            // Check if root needs collapsing after removal
            if removed.is_some() {
                self.collapse_root_if_needed();
            }

            return removed;
        }

        None
    }

    /// Remove a key from a branch node with rebalancing
    fn remove_from_branch(&mut self, branch_id: NodeId, key: &K) -> Option<V> {
        // Use helper to get child info
        let (child_index, child_ref) = self.get_child_info(branch_id, key)?;

        // Remove from child
        let (removed_value, child_became_underfull) = match child_ref {
            NodeRef::Leaf(_leaf_id, _) => {
                let removed = self.try_remove_from_node(&child_ref, key);

                // Use helper to check if leaf became underfull
                let is_underfull = self.is_node_underfull(&child_ref);

                (removed, is_underfull)
            }
            NodeRef::Branch(child_branch_id, _) => {
                let removed = self.remove_from_branch(child_branch_id, key);

                // Use helper to check if branch became underfull
                let is_underfull = self.is_node_underfull(&child_ref);

                (removed, is_underfull)
            }
        };

        // If child became underfull, try to rebalance
        if removed_value.is_some() && child_became_underfull {
            let _child_still_exists = self.rebalance_child(branch_id, child_index);

            // After rebalancing (which might involve merging), check if the parent branch
            // itself became underfull. However, we don't propagate this up since the
            // caller will check this.
        }

        removed_value
    }

    /// Rebalance an underfull child in an arena branch
    fn rebalance_child(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Get sibling information using helper
        let sibling_info = match self.get_sibling_info(branch_id, child_index) {
            Some(info) => info,
            None => return false,
        };

        // Get child reference to determine type
        let child_ref = match self.get_child_at(branch_id, child_index) {
            Some(child) => child,
            None => return false,
        };

        // Use the new generic rebalancing logic
        self.rebalance_child_generic(branch_id, child_index, &child_ref, &sibling_info)
    }

    /// Generic rebalancing that works for both leaves and branches
    fn rebalance_child_generic(
        &mut self,
        parent_id: NodeId,
        child_index: usize,
        child_ref: &NodeRef<K, V>,
        sibling_info: &SiblingInfo<K, V>,
    ) -> bool {
        // Try borrowing from left sibling
        if sibling_info.has_left() {
            if self.can_node_donate(sibling_info.left_sibling.as_ref().unwrap()) {
                return match child_ref {
                    NodeRef::Leaf(_, _) => self.borrow_from_left_leaf(parent_id, child_index),
                    NodeRef::Branch(_, _) => self.borrow_from_left_branch(parent_id, child_index),
                };
            }
        }

        // Try borrowing from right sibling
        if sibling_info.has_right() {
            if self.can_node_donate(sibling_info.right_sibling.as_ref().unwrap()) {
                return match child_ref {
                    NodeRef::Leaf(_, _) => self.borrow_from_right_leaf(parent_id, child_index),
                    NodeRef::Branch(_, _) => self.borrow_from_right_branch(parent_id, child_index),
                };
            }
        }

        // Must merge - prefer left sibling
        if sibling_info.has_left() {
            match child_ref {
                NodeRef::Leaf(_, _) => self.merge_with_left_leaf(parent_id, child_index),
                NodeRef::Branch(_, _) => self.merge_with_left_branch(parent_id, child_index),
            }
        } else if sibling_info.has_right() {
            match child_ref {
                NodeRef::Leaf(_, _) => self.merge_with_right_leaf(parent_id, child_index),
                NodeRef::Branch(_, _) => self.merge_with_right_branch(parent_id, child_index),
            }
        } else {
            false // No siblings - shouldn't happen
        }
    }



    /// Merge branch with left sibling
    fn merge_with_left_branch(&mut self, parent_id: NodeId, child_index: usize) -> bool {
        // Use helper to get branch IDs
        let (left_id, child_id, separator_key) = match self.get_branch_sibling_ids(parent_id, child_index, true) {
            Some(ids) => ids,
            None => return false,
        };

        // Get the data from child branch using helper
        let (mut child_keys, mut child_children) = match self.take_branch_data(child_id) {
            Some(data) => data,
            None => return false,
        };

        // Merge into left branch
        let merge_success = self.with_branch_mut(left_id, |left_branch| {
            // Add separator key from parent
            left_branch.keys.push(separator_key);
            // Add all keys and children from child
            left_branch.keys.append(&mut child_keys);
            left_branch.children.append(&mut child_children);
        }).is_some();

        if !merge_success {
            return false;
        }

        // Remove child from parent
        self.with_branch_mut(parent_id, |parent| {
            parent.children.remove(child_index);
            parent.keys.remove(child_index - 1);
        });

        // Deallocate the merged child
        self.deallocate_branch(child_id);

        false // Child was merged away
    }

    /// Merge branch with right sibling
    fn merge_with_right_branch(&mut self, parent_id: NodeId, child_index: usize) -> bool {
        // Use helper to get branch IDs
        let (child_id, right_id, separator_key) = match self.get_branch_sibling_ids(parent_id, child_index, false) {
            Some(ids) => ids,
            None => return false,
        };

        // Get the data from right branch using helper
        let (mut right_keys, mut right_children) = match self.take_branch_data(right_id) {
            Some(data) => data,
            None => return false,
        };

        // Merge into child branch
        let merge_success = self.with_branch_mut(child_id, |child_branch| {
            // Add separator key from parent
            child_branch.keys.push(separator_key);
            // Add all keys and children from right
            child_branch.keys.append(&mut right_keys);
            child_branch.children.append(&mut right_children);
        }).is_some();

        if !merge_success {
            return false;
        }

        // Remove right from parent
        self.with_branch_mut(parent_id, |parent| {
            parent.children.remove(child_index + 1);
            parent.keys.remove(child_index);
        });

        // Deallocate the merged right sibling
        self.deallocate_branch(right_id);

        true // Child still exists
    }

    /// Borrow from left sibling branch
    fn borrow_from_left_branch(&mut self, parent_id: NodeId, child_index: usize) -> bool {
        // Use helper to get branch IDs and parent info
        let (left_id, child_id, separator_key) = match self.get_branch_sibling_ids(parent_id, child_index, true) {
            Some(ids) => ids,
            None => return false,
        };

        // Take the last key and child from left sibling
        let (moved_key, moved_child) = match self.with_branch_mut(left_id, |left_branch| {
            if let (Some(k), Some(c)) = (left_branch.keys.pop(), left_branch.children.pop()) {
                Some((k, c))
            } else {
                None
            }
        }) {
            Some(Some(pair)) => pair,
            _ => return false,
        };

        // Insert into child branch at the beginning
        let insert_success = self.with_branch_mut(child_id, |child_branch| {
            // The separator becomes the first key in child
            child_branch.keys.insert(0, separator_key);
            // The moved child becomes the first child
            child_branch.children.insert(0, moved_child);
        }).is_some();

        if !insert_success {
            return false;
        }

        // Update separator in parent (moved_key becomes new separator)
        self.with_branch_mut(parent_id, |parent| {
            parent.keys[child_index - 1] = moved_key;
        });

        true
    }

    /// Borrow from right sibling branch
    fn borrow_from_right_branch(&mut self, parent_id: NodeId, child_index: usize) -> bool {
        // Use helper to get branch IDs and parent info
        let (child_id, right_id, separator_key) = match self.get_branch_sibling_ids(parent_id, child_index, false) {
            Some(ids) => ids,
            None => return false,
        };

        // Take the first key and child from right sibling
        let (moved_key, moved_child) = match self.with_branch_mut(right_id, |right_branch| {
            if !right_branch.keys.is_empty() {
                let k = right_branch.keys.remove(0);
                let c = right_branch.children.remove(0);
                Some((k, c))
            } else {
                None
            }
        }) {
            Some(Some(pair)) => pair,
            _ => return false,
        };

        // Append to child branch
        let append_success = self.with_branch_mut(child_id, |child_branch| {
            // The separator becomes the last key in child
            child_branch.keys.push(separator_key);
            // The moved child becomes the last child
            child_branch.children.push(moved_child);
        }).is_some();

        if !append_success {
            return false;
        }

        // Update separator in parent (moved_key becomes new separator)
        self.with_branch_mut(parent_id, |parent| {
            parent.keys[child_index] = moved_key;
        });

        true
    }

    /// Borrow from left sibling leaf
    fn borrow_from_left_leaf(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Use helper to extract the needed data
        let (left_id, child_id) = match self.get_leaf_sibling_ids(branch_id, child_index, true) {
            Some(ids) => ids,
            None => return false,
        };

        // Move last key-value from left to child
        let (key, value) = match self.with_leaf_mut(left_id, |left_leaf| {
            if let (Some(k), Some(v)) = (left_leaf.keys.pop(), left_leaf.values.pop()) {
                Some((k, v))
            } else {
                None
            }
        }) {
            Some(Some(pair)) => pair,
            _ => return false,
        };

        // Insert into child at beginning
        let insert_success = self.with_leaf_mut(child_id, |child_leaf| {
            child_leaf.keys.insert(0, key.clone());
            child_leaf.values.insert(0, value);
        }).is_some();

        if !insert_success {
            return false;
        }

        // Update separator in parent
        self.with_branch_mut(branch_id, |branch| {
            branch.keys[child_index - 1] = key;
        });

        true
    }

    /// Borrow from right sibling leaf
    fn borrow_from_right_leaf(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Use helper to extract the needed data
        let (child_id, right_id) = match self.get_leaf_sibling_ids(branch_id, child_index, false) {
            Some(ids) => ids,
            None => return false,
        };

        // Move first key-value from right to child
        let (key, value) = match self.with_leaf_mut(right_id, |right_leaf| {
            if !right_leaf.keys.is_empty() {
                Some((right_leaf.keys.remove(0), right_leaf.values.remove(0)))
            } else {
                None
            }
        }) {
            Some(Some(pair)) => pair,
            _ => return false,
        };

        // Append to child
        let append_success = self.with_leaf_mut(child_id, |child_leaf| {
            child_leaf.keys.push(key);
            child_leaf.values.push(value);
        }).is_some();

        if !append_success {
            return false;
        }

        // Update separator in parent
        let new_separator = self.with_leaf(right_id, |right_leaf| {
            if !right_leaf.keys.is_empty() {
                Some(right_leaf.keys[0].clone())
            } else {
                None
            }
        }).unwrap_or(None);

        if let Some(new_sep) = new_separator {
            self.with_branch_mut(branch_id, |branch| {
                branch.keys[child_index] = new_sep;
            });
        }

        true
    }

    /// Merge with left sibling leaf
    fn merge_with_left_leaf(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Use helper to extract the needed data
        let (left_id, child_id) = match self.get_leaf_sibling_ids(branch_id, child_index, true) {
            Some(ids) => ids,
            None => return false,
        };

        // Move all keys and values from child to left, and get child's next pointer
        let (mut keys, mut values, child_next) = match self.take_leaf_data(child_id) {
            Some(data) => data,
            None => return false,
        };

        // Merge the child into the left leaf and update linked list
        let merge_success = self.with_leaf_mut(left_id, |left_leaf| {
            left_leaf.keys.append(&mut keys);
            left_leaf.values.append(&mut values);
            // Update linked list: left leaf's next should point to what child was pointing to
            left_leaf.next = child_next;
        }).is_some();

        if !merge_success {
            return false;
        }

        // Remove child from parent
        self.with_branch_mut(branch_id, |branch| {
            branch.children.remove(child_index);
            branch.keys.remove(child_index - 1);
        });

        // Deallocate the merged child
        self.deallocate_leaf(child_id);

        false // Child was merged away
    }

    /// Merge with right sibling leaf
    fn merge_with_right_leaf(&mut self, branch_id: NodeId, child_index: usize) -> bool {
        // Use helper to extract the needed data
        let (child_id, right_id) = match self.get_leaf_sibling_ids(branch_id, child_index, false) {
            Some(ids) => ids,
            None => return false,
        };

        // Move all keys and values from right to child, and get right's next pointer
        let (mut keys, mut values, right_next) = match self.take_leaf_data(right_id) {
            Some(data) => data,
            None => return false,
        };

        // Merge the right leaf into the left leaf and update linked list
        let merge_success = self.with_leaf_mut(child_id, |child_leaf| {
            child_leaf.keys.append(&mut keys);
            child_leaf.values.append(&mut values);
            // Update linked list: left leaf's next should point to what right was pointing to
            child_leaf.next = right_next;
        }).is_some();

        if !merge_success {
            return false;
        }

        // Remove right from parent
        self.with_branch_mut(branch_id, |branch| {
            branch.children.remove(child_index + 1);
            branch.keys.remove(child_index);
        });

        // Deallocate the merged right sibling
        self.deallocate_leaf(right_id);

        true // Child still exists
    }

    /// Recursively insert a key with proper arena access.
    fn insert_recursive(&mut self, node: &NodeRef<K, V>, key: K, value: V) -> InsertResult<K, V> {
        match node {
            NodeRef::Leaf(id, _) => {
                self.with_leaf_mut(*id, |leaf| leaf.insert(key, value))
                    .unwrap_or(InsertResult::Updated(None))
            }
            NodeRef::Branch(id, _) => {
                let id = *id;

                // Use helper to get child info
                let (child_index, child_ref) = match self.get_child_info(id, &key) {
                    Some(info) => info,
                    None => return InsertResult::Updated(None),
                };

                // Recursively insert
                let child_result = self.insert_recursive(&child_ref, key, value);

                // Handle the result
                match child_result {
                    InsertResult::Updated(old_value) => InsertResult::Updated(old_value),
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
                                    self.with_leaf_mut(original_id, |original_leaf| {
                                        original_leaf.next = new_id;
                                    });
                                }

                                NodeRef::Leaf(new_id, PhantomData)
                            }
                            SplitNodeData::Branch(new_branch_data) => {
                                let new_id = self.allocate_branch(new_branch_data);
                                NodeRef::Branch(new_id, PhantomData)
                            }
                        };

                        // Insert into this branch
                        if let Some(branch) = self.get_branch_mut(id) {
                            if let Some((new_branch_data, promoted_key)) = branch
                                .insert_child_and_split_if_needed(
                                    child_index,
                                    separator_key,
                                    new_node,
                                )
                            {
                                // This branch split too - return raw branch data
                                InsertResult::Split {
                                    old_value,
                                    new_node_data: SplitNodeData::Branch(new_branch_data),
                                    separator_key: promoted_key,
                                }
                            } else {
                                // No split needed
                                InsertResult::Updated(old_value)
                            }
                        } else {
                            InsertResult::Updated(old_value)
                        }
                    }
                }
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

    /// Collapse the root if it's a branch with only one child or no children.
    fn collapse_root_if_needed(&mut self) {
        loop {
            match &self.root {
                NodeRef::Branch(id, _) => {
                    if let Some(branch) = self.get_branch(*id) {
                        if branch.children.is_empty() {
                            // Root branch has no children, replace with empty arena leaf
                            let empty_id = self.allocate_leaf(LeafNode::new(self.capacity));
                            self.root = NodeRef::Leaf(empty_id, PhantomData);
                            break;
                        } else if branch.children.len() == 1 {
                            // Root branch has only one child, replace root with that child
                            let new_root = branch.children[0].clone();
                            self.root = new_root;
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
                }
                NodeRef::Leaf(_, _) => {
                    // Arena leaf root, no collapse needed
                    break;
                }
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
            NodeRef::Leaf(id, _) => {
                self.with_leaf(*id, |leaf| leaf.len()).unwrap_or(0)
            }
            NodeRef::Branch(id, _) => {
                self.with_branch(*id, |branch| {
                    branch
                        .children
                        .iter()
                        .map(|child| self.len_recursive(child))
                        .sum()
                }).unwrap_or(0)
            }
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

    /// Get the number of free leaf IDs in the arena (for testing/debugging).
    pub fn free_leaf_count(&self) -> usize {
        self.free_leaf_ids.len()
    }

    /// Get the number of free branch IDs in the arena (for testing/debugging).
    pub fn free_branch_count(&self) -> usize {
        self.free_branch_ids.len()
    }

    /// Recursively count leaf nodes with proper arena access.
    fn leaf_count_recursive(&self, node: &NodeRef<K, V>) -> usize {
        match node {
            NodeRef::Leaf(_, _) => 1, // An arena leaf is one leaf node
            NodeRef::Branch(id, _) => {
                self.with_branch(*id, |branch| {
                    branch
                        .children
                        .iter()
                        .map(|child| self.leaf_count_recursive(child))
                        .sum()
                }).unwrap_or(0)
            }
        }
    }

    /// Clear all items from the tree.
    pub fn clear(&mut self) {
        // Clear the existing arena leaf at id=0
        self.with_leaf_mut(0, |leaf| {
            leaf.keys.clear();
            leaf.values.clear();
            leaf.next = NULL_NODE;
        });
        // Reset root to point to the cleared arena leaf
        self.root = NodeRef::Leaf(0, PhantomData);
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

    /// Alias for items_range (for compatibility).
    pub fn range<'a>(
        &'a self,
        start_key: Option<&K>,
        end_key: Option<&'a K>,
    ) -> RangeIterator<'a, K, V> {
        self.items_range(start_key, end_key)
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
    // ARENA-BASED ALLOCATION FOR LEAF NODES
    // ============================================================================

    /// Get the next available leaf ID (either from free list or arena length).
    fn next_leaf_id(&mut self) -> NodeId {
        self.free_leaf_ids
            .pop()
            .unwrap_or(self.leaf_arena.len() as NodeId)
    }

    /// Allocate a new leaf node in the arena and return its ID.
    pub fn allocate_leaf(&mut self, leaf: LeafNode<K, V>) -> NodeId {
        let id = self.next_leaf_id();

        // Extend arena if needed
        if id as usize >= self.leaf_arena.len() {
            self.leaf_arena.resize(id as usize + 1, None);
        }

        self.leaf_arena[id as usize] = Some(leaf);
        id
    }

    /// Deallocate a leaf node from the arena.
    pub fn deallocate_leaf(&mut self, id: NodeId) -> Option<LeafNode<K, V>> {
        if (id as usize) < self.leaf_arena.len() {
            if let Some(leaf) = self.leaf_arena[id as usize].take() {
                self.free_leaf_ids.push(id);
                Some(leaf)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get a reference to a leaf node in the arena.
    pub fn get_leaf(&self, id: NodeId) -> Option<&LeafNode<K, V>> {
        if (id as usize) < self.leaf_arena.len() {
            self.leaf_arena[id as usize].as_ref()
        } else {
            None
        }
    }

    /// Get a mutable reference to a leaf node in the arena.
    pub fn get_leaf_mut(&mut self, id: NodeId) -> Option<&mut LeafNode<K, V>> {
        if (id as usize) < self.leaf_arena.len() {
            self.leaf_arena[id as usize].as_mut()
        } else {
            None
        }
    }

    /// Set the next pointer of a leaf node in the arena.
    pub fn set_leaf_next(&mut self, id: NodeId, next_id: NodeId) -> bool {
        self.with_leaf_mut(id, |leaf| {
            leaf.next = next_id;
            true
        }).unwrap_or(false)
    }

    /// Get the next pointer of a leaf node in the arena.
    pub fn get_leaf_next(&self, id: NodeId) -> Option<NodeId> {
        self.with_leaf(id, |leaf| {
            if leaf.next == NULL_NODE {
                None
            } else {
                Some(leaf.next)
            }
        }).unwrap_or(None)
    }

    // ============================================================================
    // ARENA-BASED ALLOCATION FOR BRANCH NODES
    // ============================================================================

    /// Get the next available branch ID (either from free list or arena length).
    fn next_branch_id(&mut self) -> NodeId {
        self.free_branch_ids
            .pop()
            .unwrap_or(self.branch_arena.len() as NodeId)
    }

    /// Allocate a new branch node in the arena and return its ID.
    pub fn allocate_branch(&mut self, branch: BranchNode<K, V>) -> NodeId {
        let id = self.next_branch_id();

        // Extend arena if needed
        if id as usize >= self.branch_arena.len() {
            self.branch_arena.resize(id as usize + 1, None);
        }

        self.branch_arena[id as usize] = Some(branch);
        id
    }

    /// Deallocate a branch node from the arena.
    pub fn deallocate_branch(&mut self, id: NodeId) -> Option<BranchNode<K, V>> {
        if (id as usize) < self.branch_arena.len() {
            if let Some(branch) = self.branch_arena[id as usize].take() {
                self.free_branch_ids.push(id);
                Some(branch)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get a reference to a branch node in the arena.
    pub fn get_branch(&self, id: NodeId) -> Option<&BranchNode<K, V>> {
        if (id as usize) < self.branch_arena.len() {
            self.branch_arena[id as usize].as_ref()
        } else {
            None
        }
    }

    /// Get a mutable reference to a branch node in the arena.
    pub fn get_branch_mut(&mut self, id: NodeId) -> Option<&mut BranchNode<K, V>> {
        if (id as usize) < self.branch_arena.len() {
            self.branch_arena[id as usize].as_mut()
        } else {
            None
        }
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
                let size = self.with_leaf(*id, |leaf| leaf.keys.len()).unwrap_or(0);
                sizes.push(size);
            }
            NodeRef::Branch(id, _) => {
                self.with_branch(*id, |branch| {
                    for child in &branch.children {
                        self.collect_leaf_sizes(child, sizes);
                    }
                });
            }
        }
    }

    fn print_node(&self, node: &NodeRef<K, V>, depth: usize) {
        let indent = "  ".repeat(depth);
        match node {
            NodeRef::Leaf(id, _) => {
                self.with_leaf(*id, |leaf| {
                    println!(
                        "{}Leaf[id={}, cap={}]: {} keys",
                        indent,
                        id,
                        leaf.capacity,
                        leaf.keys.len()
                    );
                }).unwrap_or_else(|| {
                    println!("{}Leaf[id={}]: <missing>", indent, id);
                });
            }
            NodeRef::Branch(id, _) => {
                self.with_branch(*id, |branch| {
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
                }).unwrap_or_else(|| {
                    println!("{}Branch[id={}]: <missing>", indent, id);
                });
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
                if self.is_full() {
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
                            Ok(_) => panic!("Key should not exist in new leaf"),
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
                } else {
                    // Room to insert without splitting
                    self.insert_at_index(index, key, value);
                    // Simple insertion - no split needed
                    InsertResult::Updated(None)
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

        // Calculate split point to ensure both sides have at least min_keys
        // Left side gets min_keys, right side gets the rest
        let mid = min_keys;

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
    /// Split this branch node, returning the new right node data and promoted key.
    /// The arena-allocating code should handle creating the actual NodeRef.
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
        debug_assert!(
            total_keys - mid - 1 >= min_keys,
            "Right side would be underfull"
        );

        // The middle key gets promoted to the parent
        let promoted_key = self.keys[mid].clone();

        // Create new branch for right half (no Box allocation)
        let mut new_branch = BranchNode::new(self.capacity);

        // Move right half of keys to new branch (excluding the promoted key)
        new_branch.keys = self.keys.split_off(mid + 1);
        self.keys.truncate(mid); // Remove the promoted key from left side

        // Move right half of children to new branch
        new_branch.children = self.children.split_off(mid + 1);

        (new_branch, promoted_key)
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
}

/// Iterator over key-value pairs in the B+ tree using the leaf linked list.
pub struct ItemIterator<'a, K, V> {
    tree: &'a BPlusTreeMap<K, V>,
    current_leaf_id: Option<NodeId>,
    current_leaf_index: usize,
}

impl<'a, K: Ord + Clone, V: Clone> ItemIterator<'a, K, V> {
    fn new(tree: &'a BPlusTreeMap<K, V>) -> Self {
        // Start with the first leaf in the arena (leftmost leaf)
        let leftmost_id = if tree.leaf_arena.is_empty() || tree.leaf_arena[0].is_none() {
            None
        } else {
            Some(0)
        };

        Self {
            tree,
            current_leaf_id: leftmost_id,
            current_leaf_index: 0,
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> Iterator for ItemIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(leaf_id) = self.current_leaf_id {
                if let Some(leaf) = self.tree.get_leaf(leaf_id) {
                    // Check if we have more items in the current leaf
                    if self.current_leaf_index < leaf.keys.len() {
                        let key = &leaf.keys[self.current_leaf_index];
                        let value = &leaf.values[self.current_leaf_index];
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
                    return None;
                }
            } else {
                // No more leaves
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

/// Iterator over a range of key-value pairs in the B+ tree.
#[derive(Debug)]
pub struct RangeIterator<'a, K, V> {
    items: Vec<(&'a K, &'a V)>,
    index: usize,
}

impl<'a, K: Ord + Clone, V: Clone> RangeIterator<'a, K, V> {
    fn new(tree: &'a BPlusTreeMap<K, V>, start_key: Option<&K>, end_key: Option<&'a K>) -> Self {
        let mut items = Vec::new();
        Self::collect_range_items(tree, &tree.root, start_key, end_key, &mut items);
        Self { items, index: 0 }
    }

    fn collect_range_items(
        tree: &'a BPlusTreeMap<K, V>,
        node: &'a NodeRef<K, V>,
        start_key: Option<&K>,
        end_key: Option<&K>,
        items: &mut Vec<(&'a K, &'a V)>,
    ) {
        match node {
            NodeRef::Leaf(id, _) => {
                if let Some(leaf) = tree.get_leaf(*id) {
                    for (key, value) in leaf.keys.iter().zip(leaf.values.iter()) {
                        // Early termination if we've passed the end key
                        if let Some(end) = end_key {
                            if key >= end {
                                return; // Stop collecting, we've gone past the range
                            }
                        }

                        // Check if key is after start
                        let after_start = start_key.map_or(true, |start| key >= start);

                        if after_start {
                            items.push((key, value));
                        }
                    }
                }
            }
            NodeRef::Branch(id, _) => {
                if let Some(branch) = tree.get_branch(*id) {
                    for (i, child) in branch.children.iter().enumerate() {
                        // Check if this child could contain keys in our range
                        let child_min = if i == 0 {
                            None
                        } else {
                            Some(&branch.keys[i - 1])
                        };
                        let child_max = if i < branch.keys.len() {
                            Some(&branch.keys[i])
                        } else {
                            None
                        };

                        // Skip this child if it's entirely before our start key
                        if let (Some(start), Some(max)) = (start_key, child_max) {
                            if max <= start {
                                continue; // This child is entirely before our range
                            }
                        }

                        // Skip this child if it's entirely after our end key
                        if let (Some(end), Some(min)) = (end_key, child_min) {
                            if min >= end {
                                return; // This child and all following are after our range
                            }
                        }

                        // This child might contain keys in our range
                        Self::collect_range_items(tree, child, start_key, end_key, items);

                        // Early termination: if we have an end key and this child's max >= end,
                        // we don't need to check further children
                        if let (Some(end), Some(max)) = (end_key, child_max) {
                            if max >= end {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<'a, K: Ord, V> Iterator for RangeIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.items.len() {
            let item = self.items[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
