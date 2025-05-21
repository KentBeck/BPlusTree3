//! BPlusTree library.

#[derive(Debug)]
pub struct BPlusTree<K, V, const N: usize = 8> {
    root: LeafNode<K, V, N>,
}

impl<K: Ord, V, const N: usize> Default for BPlusTree<K, V, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct LeafNode<K, V, const N: usize> {
    _keys: [Option<K>; N],
    _values: [Option<V>; N],
    len: usize,
}

impl<K, V, const N: usize> Default for LeafNode<K, V, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, const N: usize> LeafNode<K, V, N> {
    fn new() -> Self {
        Self {
            _keys: std::array::from_fn(|_| None),
            _values: std::array::from_fn(|_| None),
            len: 0,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<K: Ord, V, const N: usize> BPlusTree<K, V, N> {
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

