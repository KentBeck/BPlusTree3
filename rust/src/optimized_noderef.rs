//! Optimized NodeRef implementation to reduce memory footprint
//! Reduces NodeRef size from 16 bytes to 8 bytes by eliminating PhantomData

use std::fmt::Debug;

/// Node ID type for arena-based allocation
pub type NodeId = u32;

/// Special node ID constants
pub const NULL_NODE: NodeId = u32::MAX;

/// Optimized node reference that packs type information into a single u64
/// This eliminates PhantomData overhead and reduces size from 16 bytes to 8 bytes
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptimizedNodeRef(u64);

impl OptimizedNodeRef {
    /// Flag bit to indicate leaf nodes (MSB of u64)
    const LEAF_FLAG: u64 = 1u64 << 63;
    
    /// Mask to extract the actual node ID (lower 32 bits)
    const ID_MASK: u64 = 0x00000000FFFFFFFF;
    
    /// Create a new leaf node reference
    pub fn new_leaf(id: NodeId) -> Self {
        Self(Self::LEAF_FLAG | (id as u64))
    }
    
    /// Create a new branch node reference
    pub fn new_branch(id: NodeId) -> Self {
        Self(id as u64)
    }
    
    /// Get the node ID
    pub fn id(&self) -> NodeId {
        (self.0 & Self::ID_MASK) as NodeId
    }
    
    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        (self.0 & Self::LEAF_FLAG) != 0
    }
    
    /// Check if this is a branch node
    pub fn is_branch(&self) -> bool {
        !self.is_leaf()
    }
    
    /// Create a null reference
    pub fn null() -> Self {
        Self::new_branch(NULL_NODE)
    }
    
    /// Check if this is a null reference
    pub fn is_null(&self) -> bool {
        self.id() == NULL_NODE
    }
}

impl Default for OptimizedNodeRef {
    fn default() -> Self {
        Self::null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;
    
    #[test]
    fn test_size_optimization() {
        // Verify the size reduction
        assert_eq!(mem::size_of::<OptimizedNodeRef>(), 8);
        
        // Compare with original enum size (would be larger due to PhantomData)
        // Original: enum (8 bytes) + NodeId (4 bytes) + PhantomData (0 bytes) + padding = 16 bytes
        // Optimized: u64 = 8 bytes
    }
    
    #[test]
    fn test_leaf_node_ref() {
        let leaf_ref = OptimizedNodeRef::new_leaf(42);
        
        assert!(leaf_ref.is_leaf());
        assert!(!leaf_ref.is_branch());
        assert_eq!(leaf_ref.id(), 42);
        assert!(!leaf_ref.is_null());
    }
    
    #[test]
    fn test_branch_node_ref() {
        let branch_ref = OptimizedNodeRef::new_branch(123);
        
        assert!(!branch_ref.is_leaf());
        assert!(branch_ref.is_branch());
        assert_eq!(branch_ref.id(), 123);
        assert!(!branch_ref.is_null());
    }
    
    #[test]
    fn test_null_reference() {
        let null_ref = OptimizedNodeRef::null();
        
        assert!(null_ref.is_null());
        assert!(null_ref.is_branch()); // NULL_NODE is treated as branch
        assert_eq!(null_ref.id(), NULL_NODE);
    }
    
    #[test]
    fn test_max_node_id() {
        // Test with maximum valid node ID (NULL_NODE - 1)
        let max_id = NULL_NODE - 1;
        
        let leaf_ref = OptimizedNodeRef::new_leaf(max_id);
        assert!(leaf_ref.is_leaf());
        assert_eq!(leaf_ref.id(), max_id);
        
        let branch_ref = OptimizedNodeRef::new_branch(max_id);
        assert!(branch_ref.is_branch());
        assert_eq!(branch_ref.id(), max_id);
    }
    
    #[test]
    fn test_zero_node_id() {
        let leaf_ref = OptimizedNodeRef::new_leaf(0);
        assert!(leaf_ref.is_leaf());
        assert_eq!(leaf_ref.id(), 0);
        
        let branch_ref = OptimizedNodeRef::new_branch(0);
        assert!(branch_ref.is_branch());
        assert_eq!(branch_ref.id(), 0);
    }
    
    #[test]
    fn test_equality() {
        let leaf1 = OptimizedNodeRef::new_leaf(42);
        let leaf2 = OptimizedNodeRef::new_leaf(42);
        let leaf3 = OptimizedNodeRef::new_leaf(43);
        let branch1 = OptimizedNodeRef::new_branch(42);
        
        assert_eq!(leaf1, leaf2);
        assert_ne!(leaf1, leaf3);
        assert_ne!(leaf1, branch1); // Same ID but different type
    }
    
    #[test]
    fn test_debug_format() {
        let leaf_ref = OptimizedNodeRef::new_leaf(42);
        let debug_str = format!("{:?}", leaf_ref);
        
        // Should contain the internal representation
        assert!(debug_str.contains("OptimizedNodeRef"));
    }
    
    #[test]
    fn test_clone_copy() {
        let original = OptimizedNodeRef::new_leaf(42);
        let cloned = original.clone();
        let copied = original;
        
        assert_eq!(original, cloned);
        assert_eq!(original, copied);
    }
}
