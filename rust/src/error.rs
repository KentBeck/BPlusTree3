//! Error handling and result types for BPlusTreeMap operations.
//!
//! This module provides comprehensive error handling for all B+ tree operations,
//! including specialized error types and result type aliases for better ergonomics.

/// Error type for B+ tree operations.
#[derive(Debug, Clone, PartialEq)]
pub enum BPlusTreeError {
    /// Key not found in the tree.
    KeyNotFound,
    /// Invalid capacity specified.
    InvalidCapacity(String),
    /// Internal data structure integrity violation.
    DataIntegrityError(String),
    /// Arena operation failed.
    ArenaError(String),
    /// Node operation failed.
    NodeError(String),
    /// Tree corruption detected.
    CorruptedTree(String),
    /// Invalid tree state.
    InvalidState(String),
    /// Memory allocation failed.
    AllocationError(String),
}

impl BPlusTreeError {
    /// Create an InvalidCapacity error with context
    pub fn invalid_capacity(capacity: usize, min_required: usize) -> Self {
        Self::InvalidCapacity(format!(
            "Capacity {} is invalid (minimum required: {})",
            capacity, min_required
        ))
    }

    /// Create a DataIntegrityError with context
    pub fn data_integrity(context: &str, details: &str) -> Self {
        Self::DataIntegrityError(format!("{}: {}", context, details))
    }

    /// Create an ArenaError with context
    pub fn arena_error(operation: &str, details: &str) -> Self {
        Self::ArenaError(format!("{} failed: {}", operation, details))
    }

    /// Create a NodeError with context
    pub fn node_error(node_type: &str, node_id: u32, details: &str) -> Self {
        Self::NodeError(format!("{} node {}: {}", node_type, node_id, details))
    }

    /// Create a CorruptedTree error with context
    pub fn corrupted_tree(component: &str, details: &str) -> Self {
        Self::CorruptedTree(format!("{} corruption: {}", component, details))
    }

    /// Create an InvalidState error with context
    pub fn invalid_state(operation: &str, state: &str) -> Self {
        Self::InvalidState(format!("Cannot {} in state: {}", operation, state))
    }

    /// Create an AllocationError with context
    pub fn allocation_error(resource: &str, reason: &str) -> Self {
        Self::AllocationError(format!("Failed to allocate {}: {}", resource, reason))
    }

    /// Check if this error is a capacity error
    pub fn is_capacity_error(&self) -> bool {
        matches!(self, Self::InvalidCapacity(_))
    }

    /// Check if this error is an arena error
    pub fn is_arena_error(&self) -> bool {
        matches!(self, Self::ArenaError(_))
    }
}

impl std::fmt::Display for BPlusTreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BPlusTreeError::KeyNotFound => write!(f, "Key not found in tree"),
            BPlusTreeError::InvalidCapacity(msg) => write!(f, "Invalid capacity: {}", msg),
            BPlusTreeError::DataIntegrityError(msg) => write!(f, "Data integrity error: {}", msg),
            BPlusTreeError::ArenaError(msg) => write!(f, "Arena error: {}", msg),
            BPlusTreeError::NodeError(msg) => write!(f, "Node error: {}", msg),
            BPlusTreeError::CorruptedTree(msg) => write!(f, "Corrupted tree: {}", msg),
            BPlusTreeError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            BPlusTreeError::AllocationError(msg) => write!(f, "Allocation error: {}", msg),
        }
    }
}

impl std::error::Error for BPlusTreeError {}

/// Internal result type for tree operations
pub(crate) type TreeResult<T> = Result<T, BPlusTreeError>;

/// Public result type for tree operations that may fail
pub type BTreeResult<T> = Result<T, BPlusTreeError>;

/// Result type for key lookup operations
pub type KeyResult<T> = Result<T, BPlusTreeError>;

/// Result type for tree modification operations
pub type ModifyResult<T> = Result<T, BPlusTreeError>;

/// Result type for tree construction and validation
pub type InitResult<T> = Result<T, BPlusTreeError>;

/// Result extension trait for improved error handling
pub trait BTreeResultExt<T> {
    /// Convert to a BTreeResult with additional context
    fn with_context(self, context: &str) -> BTreeResult<T>;

    /// Convert to a BTreeResult with operation context
    fn with_operation(self, operation: &str) -> BTreeResult<T>;

    /// Log error and continue with default value
    fn or_default_with_log(self) -> T
    where
        T: Default;
}

impl<T> BTreeResultExt<T> for Result<T, BPlusTreeError> {
    fn with_context(self, context: &str) -> BTreeResult<T> {
        self.map_err(|e| match e {
            BPlusTreeError::KeyNotFound => BPlusTreeError::KeyNotFound,
            BPlusTreeError::InvalidCapacity(msg) => {
                BPlusTreeError::InvalidCapacity(format!("{}: {}", context, msg))
            }
            BPlusTreeError::DataIntegrityError(msg) => {
                BPlusTreeError::data_integrity(context, &msg)
            }
            BPlusTreeError::ArenaError(msg) => BPlusTreeError::arena_error(context, &msg),
            BPlusTreeError::NodeError(msg) => {
                BPlusTreeError::NodeError(format!("{}: {}", context, msg))
            }
            BPlusTreeError::CorruptedTree(msg) => BPlusTreeError::corrupted_tree(context, &msg),
            BPlusTreeError::InvalidState(msg) => BPlusTreeError::invalid_state(context, &msg),
            BPlusTreeError::AllocationError(msg) => BPlusTreeError::allocation_error(context, &msg),
        })
    }

    fn with_operation(self, operation: &str) -> BTreeResult<T> {
        self.with_context(&format!("Operation '{}'", operation))
    }

    fn or_default_with_log(self) -> T
    where
        T: Default,
    {
        match self {
            Ok(value) => value,
            Err(e) => {
                eprintln!("Warning: B+ Tree operation failed, using default: {}", e);
                T::default()
            }
        }
    }
}