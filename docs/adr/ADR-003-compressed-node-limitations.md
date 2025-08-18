# ADR-003: Compressed Node Limitations and Future Directions

## Status
Accepted

## Context

During implementation of compressed branch and leaf nodes (`CompressedBranchNode` and `CompressedLeafNode`), we discovered fundamental limitations with the compressed storage approach when dealing with generic key-value types.

### Current Implementation Issues

The compressed nodes store data in fixed-size byte arrays using raw pointer arithmetic:
- `CompressedBranchNode<K, V>` uses `data: [u64; 27]` 
- `CompressedLeafNode<K, V>` uses `data: [u64; 32]`

This approach works for simple `Copy` types but creates critical problems for heap-allocated data:

1. **Memory Manager Invisibility**: When `K` or `V` types contain heap-allocated data (e.g., `String`, `Vec`, `Box`), the memory manager cannot trace references stored within the compressed byte arrays.

2. **Garbage Collection Issues**: References to heap data become invisible to Rust's ownership system, potentially leading to:
   - Use-after-free bugs
   - Memory leaks
   - Double-free errors

3. **Generic Type Constraints**: The compressed format requires `K: Copy` and `V: Copy`, severely limiting the types that can be stored.

### Example Problematic Scenario

```rust
// This would be unsafe with compressed nodes:
let tree = BPlusTree::<String, Vec<u8>>::new(16);
tree.insert("key".to_string(), vec![1, 2, 3, 4]);

// The String and Vec are heap-allocated, but stored as raw bytes
// in the compressed node's fixed array. The memory manager loses
// track of these allocations.
```

## Decision

**We will NOT use compressed nodes for general-purpose B+ tree storage** due to the fundamental incompatibility with Rust's memory management for heap-allocated types.

However, we identify a **viable specialized use case**: Fixed-type trees optimized for specific data patterns.

## Rationale

### Why General Compression Fails
- Rust's ownership model requires visible references for heap-allocated data
- Raw byte storage breaks the ownership chain
- Generic types (`K`, `V`) can be arbitrarily complex with nested heap allocations
- No safe way to serialize/deserialize arbitrary types in fixed byte arrays

### Why Specialized Fixed-Type Trees Could Work

For Facebook graph data storage requirements, we could implement:

```rust
pub struct FixedGraphTree {
    // Fixed key type - no heap allocation
    keys: u64,           // Node IDs, timestamps, etc.
    
    // Variable-sized values - managed separately
    values: Vec<u8>,     // Serialized graph data
}
```

Benefits:
- `u64` keys are `Copy` and fit perfectly in compressed storage
- Variable-sized `Vec<u8>` values can be managed with proper Rust ownership
- No fixed "number of keys" capacity constraint for leaves
- Optimized for graph data patterns (numeric IDs + binary payloads)

## Consequences

### Positive
- **Memory Safety**: Avoid unsafe memory management issues
- **Rust Compatibility**: Work with Rust's ownership model, not against it
- **Specialized Performance**: Fixed-type trees can be highly optimized
- **Clear Boundaries**: Separate concerns between generic trees and specialized storage

### Negative
- **Limited Generality**: Compressed nodes cannot be used for arbitrary `K`, `V` types
- **Code Duplication**: May need separate implementations for different use cases
- **Complexity**: Multiple tree variants increase maintenance burden

## Implementation Notes

### Current Status
- Generic compressed nodes are implemented but should be considered **experimental only**
- All existing tests pass, but usage is limited to `Copy` types
- Performance benefits are significant for supported types

### Future Work
If Facebook graph storage requirements justify the effort:

1. **Implement `FixedGraphTree`**:
   ```rust
   pub struct FixedGraphTree {
       root: Option<FixedGraphNode>,
   }
   
   struct FixedGraphNode {
       keys: [u64; N],           // Fixed-size key array
       values: Vec<Vec<u8>>,     // Variable-sized value storage
       children: [NodeId; N+1],  // Child references
   }
   ```

2. **Variable Capacity Leaves**: Remove fixed capacity constraints to handle varying data sizes efficiently.

3. **Optimized Serialization**: Custom serialization for graph-specific data patterns.

## Alternatives Considered

1. **Smart Pointer Compression**: Store `Rc<K>`, `Arc<V>` in compressed format
   - **Rejected**: Still breaks ownership visibility, adds reference counting overhead

2. **Custom Allocator Integration**: Hook into Rust's allocator to track compressed references
   - **Rejected**: Too complex, fragile, and non-portable

3. **Trait-Based Serialization**: Require `K: Serialize`, `V: Serialize`
   - **Rejected**: Performance overhead, complexity, still doesn't solve ownership issues

## References
- [Rust Ownership Model](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [Memory Safety in Systems Programming](https://www.memorysafety.org/)
- Facebook Graph Storage Requirements (internal documentation)

---

**Date**: 2025-01-17  
**Authors**: Development Team  
**Reviewers**: Architecture Team
