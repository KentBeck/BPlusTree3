//! Node implementations for BPlusTreeMap.
//!
//! This module contains the complete implementations for LeafNode and BranchNode,
//! including all their methods for insertion, deletion, splitting, merging, and
//! other node-level operations.

use crate::types::{LeafNode, BranchNode, NodeRef, NodeId, NULL_NODE, InsertResult, SplitNodeData};

// ============================================================================
// LEAF NODE IMPLEMENTATION
// ============================================================================

impl<K: Ord + Clone, V: Clone> LeafNode<K, V> {
    // ============================================================================
    // GET OPERATIONS
    // ============================================================================

    /// Get a value by key from this leaf node.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.keys
            .binary_search(key)
            .ok()
            .map(|index| &self.values[index])
    }

    /// Get a mutable reference to a value by key from this leaf node.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.keys
            .binary_search(key)
            .ok()
            .map(|index| &mut self.values[index])
    }

    /// Returns the number of key-value pairs in this leaf.
    pub fn len(&self) -> usize {
        self.keys.len()
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

                // Node is full, need to split
                // Insert first, then split
                self.insert_at_index(index, key, value);

                // Now split the overfull node
                let new_right = self.split();

                // Determine the separator key (first key of right node)
                let separator_key = new_right.keys[0].clone();

                InsertResult::Split {
                    old_value: None,
                    new_node_data: SplitNodeData::Leaf(new_right),
                    separator_key,
                }
            }
        }
    }

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

        // Split the keys and values
        let right_keys = self.keys.split_off(mid);
        let right_values = self.values.split_off(mid);

        // Create the new right node
        let new_right = LeafNode {
            capacity: self.capacity,
            keys: right_keys,
            values: right_values,
            next: self.next, // Right node takes over the next pointer
        };

        // Update the linked list: this node now points to the new right node
        // The new right node will get its ID when allocated in the arena
        // For now, we set next to NULL_NODE and let the caller handle linking
        self.next = NULL_NODE;

        new_right
    }

    // ============================================================================
    // DELETE OPERATIONS
    // ============================================================================

    /// Remove a key-value pair from this leaf node.
    /// Returns the removed value if the key existed, and whether the node is now underfull.
    pub fn remove(&mut self, key: &K) -> (Option<V>, bool) {
        match self.keys.binary_search(key) {
            Ok(index) => {
                let removed_value = self.values.remove(index);
                self.keys.remove(index);
                let is_underfull = self.is_underfull();
                (Some(removed_value), is_underfull)
            }
            Err(_) => (None, false), // Key not found
        }
    }

    // ============================================================================
    // STATUS CHECKS
    // ============================================================================

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

// ============================================================================
// BRANCH NODE IMPLEMENTATION
// ============================================================================

impl<K: Ord + Clone, V: Clone> BranchNode<K, V> {
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

            // Now split the overfull branch
            let (new_right, promoted_key) = self.split_data();
            Some((new_right, promoted_key))
        } else {
            // Room to insert without splitting
            self.keys.insert(child_index, separator_key);
            self.children.insert(child_index + 1, new_child);
            None
        }
    }

    /// Split this branch node, returning the new right node and promoted key.
    pub fn split_data(&mut self) -> (BranchNode<K, V>, K) {
        // For branch nodes, we need to ensure both resulting nodes have at least min_keys
        // The middle key gets promoted, so we need at least min_keys on each side
        let min_keys = self.min_keys();
        let _total_keys = self.keys.len();

        // For branch splits, we promote the middle key, so we need:
        // - Left side: min_keys keys
        // - Middle: 1 key (promoted)
        // - Right side: min_keys keys
        // Total needed: min_keys + 1 + min_keys
        let mid = min_keys;

        // Extract the promoted key
        let promoted_key = self.keys[mid].clone();

        // Split keys and children
        let right_keys = self.keys.split_off(mid + 1); // Skip the promoted key
        let right_children = self.children.split_off(mid + 1);

        // Remove the promoted key from left side
        self.keys.pop(); // Remove the key that was promoted

        // Create the new right branch
        let new_right = BranchNode {
            capacity: self.capacity,
            keys: right_keys,
            children: right_children,
        };

        (new_right, promoted_key)
    }

    // ============================================================================
    // STATUS CHECKS
    // ============================================================================

    /// Returns true if this branch node is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Returns true if this branch node is at capacity.
    pub fn is_full(&self) -> bool {
        self.keys.len() >= self.capacity
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

    /// Find the index of the child that should contain the given key.
    pub fn find_child_index(&self, key: &K) -> usize {
        // Binary search to find the appropriate child
        match self.keys.binary_search(key) {
            Ok(index) => index + 1, // Key found, go to right child
            Err(index) => index,    // Key not found, index is the insertion point
        }
    }

    /// Returns the number of keys in this branch node.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns true if this branch node needs to be split.
    /// We allow one extra key beyond capacity to ensure proper splitting.
    pub fn needs_split(&self) -> bool {
        self.keys.len() > self.capacity
    }

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
            return None;
        }
        Some(&mut self.children[child_index])
    }

    // ============================================================================
    // BORROWING AND MERGING HELPERS
    // ============================================================================

    /// Borrow the last key and child from this branch (used when this is the left sibling)
    pub fn borrow_last(&mut self) -> Option<(K, NodeRef<K, V>)> {
        if self.keys.is_empty() || !self.can_donate() {
            return None;
        }
        let key = self.keys.pop().unwrap();
        let child = self.children.pop().unwrap();
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
