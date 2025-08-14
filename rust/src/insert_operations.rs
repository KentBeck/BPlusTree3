//! INSERT operations for BPlusTreeMap.
//!
//! This module contains all the insertion operations for the B+ tree, including
//! key-value insertion, node splitting, tree growth, and helper methods for
//! managing the tree structure during insertions.

use crate::error::{BPlusTreeError, BTreeResult, ModifyResult};
use crate::types::{BPlusTreeMap, NodeRef, LeafNode, BranchNode, NodeId, InsertResult, SplitNodeData};
use std::marker::PhantomData;

impl<K: Ord + Clone, V: Clone> BPlusTreeMap<K, V> {
    /// Allocate a new leaf node in the arena and return its ID.
    pub fn allocate_leaf(&mut self, leaf: LeafNode<K, V>) -> NodeId {
        self.leaf_arena.allocate(leaf)
    }

    /// Allocate a new branch node in the arena and return its ID.
    pub fn allocate_branch(&mut self, branch: BranchNode<K, V>) -> NodeId {
        self.branch_arena.allocate(branch)
    }

    /// Create a new root node when the current root splits.
    /// New roots are the only BranchNodes allowed to remain underfull.
    pub fn new_root(&mut self, new_node: NodeRef<K, V>, separator_key: K) -> BranchNode<K, V> {
        let mut new_root = BranchNode::new(self.capacity);
        new_root.keys.push(separator_key);

        // Move the current root to be the left child
        // Use a dummy NodeRef with NULL_NODE to avoid arena allocation
        let dummy = NodeRef::Leaf(crate::types::NULL_NODE, PhantomData);
        let old_root = std::mem::replace(&mut self.root, dummy);

        new_root.children.push(old_root);
        new_root.children.push(new_node);

        new_root
    }

    // insert method will be moved here once all supporting methods are ready
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_operations_module_exists() {
        // Just a placeholder test to ensure the module compiles
        assert!(true);
    }
}