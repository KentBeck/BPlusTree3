use bplustree3::BPlusTree;

#[test]
fn create_empty_tree() {
    let _tree: BPlusTree<i32, i32> = BPlusTree::new();
}

#[test]
fn empty_tree_len_zero() {
    let tree: BPlusTree<i32, i32> = BPlusTree::new();
    assert_eq!(tree.len(), 0);
    assert!(tree.is_empty());
}
