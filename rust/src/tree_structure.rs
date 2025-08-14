//! Tree structure management operations for BPlusTreeMap.
//!
//! This module contains all tree-level operations that manage the overall structure,
//! including size queries, clearing, node counting, and tree statistics.

use std::marker::PhantomData;
use crate::types::{BPlusTreeMap, NodeRef, NodeId, LeafNode};

// ============================================================================
// TREE STRUCTURE OPERATIONS
// ============================================================================

impl<K: Ord + Clone, V: Clone> BPlusTreeMap<K, V> {
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

    /// Count the number of leaf and branch nodes actually in the tree structure.
    pub fn count_nodes_in_tree(&self) -> (usize, usize) {
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

    // ============================================================================
    // TREE NAVIGATION HELPERS
    // ============================================================================

    /// Get the ID of the first (leftmost) leaf in the tree
    pub fn get_first_leaf_id(&self) -> Option<NodeId> {
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

    /// Deallocate a leaf node from the arena.
    pub fn deallocate_leaf(&mut self, id: NodeId) -> Option<LeafNode<K, V>> {
        self.leaf_arena.deallocate(id)
    }

    /// Deallocate a branch node from the arena.
    pub fn deallocate_branch(&mut self, id: NodeId) -> Option<crate::types::BranchNode<K, V>> {
        self.branch_arena.deallocate(id)
    }

    // ============================================================================
    // CHILD LOOKUP HELPERS
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
    // UNSAFE ARENA ACCESS
    // ============================================================================

    /// Unsafe fast access to leaf node (no bounds checking)
    /// SAFETY: Caller must ensure id is valid and allocated
    pub unsafe fn get_leaf_unchecked(&self, id: NodeId) -> &LeafNode<K, V> {
        self.leaf_arena.get_unchecked(id)
    }

    /// Unsafe fast access to branch node (no bounds checking)
    /// SAFETY: Caller must ensure id is valid and allocated
    pub unsafe fn get_branch_unchecked(&self, id: NodeId) -> &crate::types::BranchNode<K, V> {
        self.branch_arena.get_unchecked(id)
    }
}
