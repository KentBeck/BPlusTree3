//! BPlusTree library.

use std::collections::BTreeMap;

#[derive(Debug)]
struct LeafNode<K, V> {
    entries: BTreeMap<K, V>,
}

impl<K: Ord, V> LeafNode<K, V> {
    fn new() -> Self {
        Self { entries: BTreeMap::new() }
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug)]
pub struct BPlusTree<K, V> {
    root: LeafNode<K, V>,
}

impl<K: Ord, V> BPlusTree<K, V> {
    /// Creates an empty `BPlusTree`.
    pub fn new() -> Self {
        Self { root: LeafNode::new() }
    }

    /// Inserts a key-value pair into the tree.
    pub fn insert(&mut self, _key: K, _value: V) -> Option<V> {
        unimplemented!("not yet implemented")
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, _key: &K) -> Option<&V> {
        unimplemented!("not yet implemented")
    }

    /// Removes a key from the tree, returning the value if it existed.
    pub fn remove(&mut self, _key: &K) -> Option<V> {
        unimplemented!("not yet implemented")
    }

    /// Returns the number of elements in the tree.
    pub fn len(&self) -> usize {
        self.root.len()
    }

    /// Returns `true` if the tree contains no elements.
    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }
}

