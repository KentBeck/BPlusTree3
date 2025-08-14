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