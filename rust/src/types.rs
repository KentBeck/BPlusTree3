//! Core types and data structures for BPlusTreeMap.
//!
//! This module contains all the fundamental data structures, type definitions,
//! and constants used throughout the B+ tree implementation.

use std::marker::PhantomData;
use crate::compact_arena::CompactArena;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Minimum capacity for any B+ tree node
pub(crate) const MIN_CAPACITY: usize = 4;

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/// Node ID type for arena-based allocation
pub type NodeId = u32;

/// Special node ID constants
pub const NULL_NODE: NodeId = u32::MAX;
pub const ROOT_NODE: NodeId = 0;

// ============================================================================
// CORE DATA STRUCTURES
// ============================================================================

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
    pub(crate) capacity: usize,
    /// The root node of the tree.
    pub(crate) root: NodeRef<K, V>,

    // Compact arena-based allocation for better performance
    /// Compact arena storage for leaf nodes (eliminates Option wrapper overhead).
    pub(crate) leaf_arena: CompactArena<LeafNode<K, V>>,
    /// Compact arena storage for branch nodes (eliminates Option wrapper overhead).
    pub(crate) branch_arena: CompactArena<BranchNode<K, V>>,
    /// Compact arena storage for compressed branch nodes.
    pub(crate) compressed_branch_arena: CompactArena<crate::compressed_branch::CompressedBranchNode<K, V>>,
}

/// Leaf node containing key-value pairs.
#[derive(Debug, Clone)]
pub struct LeafNode<K, V> {
    /// Maximum number of keys this node can hold.
    pub(crate) capacity: usize,
    /// Sorted list of keys.
    pub(crate) keys: Vec<K>,
    /// List of values corresponding to keys.
    pub(crate) values: Vec<V>,
    /// Next leaf node in the linked list (for range queries).
    pub(crate) next: NodeId,
}

// Type aliases for different use cases
/// High-performance leaf node for Copy types (cache-optimized)
pub type FastLeafNode<K, V> = crate::compressed_node::CompressedLeafNode<K, V>;

/// Flexible leaf node for Clone types (compatibility)
pub type FlexibleLeafNode<K, V> = LeafNode<K, V>;

// Conditional type selection based on traits
/// Automatically select the best leaf node type based on trait bounds
pub trait OptimalLeafNode<K, V> {
    type Node;
}

// For Copy types, use CompressedLeafNode
impl<K, V> OptimalLeafNode<K, V> for (K, V)
where
    K: Copy + Ord,
    V: Copy,
{
    type Node = crate::compressed_node::CompressedLeafNode<K, V>;
}

// For non-Copy types, use regular LeafNode  
// (This would require more complex trait bounds, but shows the concept)

/// Internal (branch) node containing keys and child pointers.
#[derive(Debug, Clone)]
pub struct BranchNode<K, V> {
    /// Maximum number of keys this node can hold.
    pub(crate) capacity: usize,
    /// Sorted list of separator keys.
    pub(crate) keys: Vec<K>,
    /// List of child nodes (leaves or other branches).
    pub(crate) children: Vec<NodeRef<K, V>>,
}

// ============================================================================
// ENUMS AND RESULT TYPES
// ============================================================================

/// Node reference that can be either a leaf or branch node
#[derive(Debug, PartialEq, Eq)]
pub enum NodeRef<K, V> {
    Leaf(NodeId, PhantomData<(K, V)>),
    Branch(NodeId, PhantomData<(K, V)>),
}

impl<K, V> Clone for NodeRef<K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V> Copy for NodeRef<K, V> {}

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
    CompressedBranch(crate::compressed_branch::CompressedBranchNode<K, V>),
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
    Error(crate::error::BPlusTreeError),
}

/// Result of a removal operation on a node.
pub enum RemoveResult<V> {
    /// Removal completed. Contains the removed value if key existed.
    /// The bool indicates if this node is now underfull and needs rebalancing.
    Updated(Option<V>, bool),
}
