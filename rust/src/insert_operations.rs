//! INSERT operations for BPlusTreeMap.
//!
//! This module contains all the insertion operations for the B+ tree, including
//! key-value insertion, node splitting, tree growth, and helper methods for
//! managing the tree structure during insertions.

use crate::types::{BPlusTreeMap, BranchNode, InsertResult, NodeId, NodeRef, SplitNodeData};
use std::marker::PhantomData;

impl<K: Ord + Clone, V: Clone> BPlusTreeMap<K, V> {
    // allocate_leaf and allocate_branch methods moved to arena.rs module

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

    /// Insert into a leaf node by ID.
    fn insert_into_leaf(&mut self, leaf_id: NodeId, key: K, value: V) -> InsertResult<K, V> {
        let leaf = match self.get_leaf_mut(leaf_id) {
            Some(leaf) => leaf,
            None => return InsertResult::Updated(None),
        };

        // Do binary search once and use the result throughout
        match leaf.binary_search_keys(&key) {
            Ok(index) => {
                // Key already exists, update the value
                if let Some(old_val) = leaf.get_value_mut(index) {
                    let old_value = std::mem::replace(old_val, value);
                    InsertResult::Updated(Some(old_value))
                } else {
                    InsertResult::Updated(None)
                }
            }
            Err(index) => {
                // Key doesn't exist, need to insert
                // Check if split is needed BEFORE inserting
                if !leaf.is_full() {
                    // Room to insert without splitting
                    leaf.insert_at_index(index, key, value);
                    // Simple insertion - no split needed
                    return InsertResult::Updated(None);
                }

                // Node is full, need to split
                // Don't insert first. That causes the Vecs to overflow.
                // Split the full node
                let mut new_right = leaf.split();
                // Insert into the correct node
                if index <= leaf.keys.len() {
                    leaf.insert_at_index(index, key, value);
                } else {
                    new_right.insert_at_index(index - leaf.keys.len(), key, value);
                }

                // Determine the separator key (first key of right node)
                let separator_key = new_right.first_key().unwrap().clone();

                InsertResult::Split {
                    old_value: None,
                    new_node_data: SplitNodeData::Leaf(new_right),
                    separator_key,
                }
            }
        }
    }

    /// Recursively insert a key with proper arena access.
    pub fn insert_recursive(
        &mut self,
        node: &NodeRef<K, V>,
        key: K,
        value: V,
    ) -> InsertResult<K, V> {
        match node {
            NodeRef::Leaf(id, _) => self.insert_into_leaf(*id, key, value),
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
}

#[cfg(test)]
mod tests {
    // Test module for insert operations

    #[test]
    fn test_insert_operations_module_exists() {
        // Just a placeholder test to ensure the module compiles
        assert!(true);
    }
}
