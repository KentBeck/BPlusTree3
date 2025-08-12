co# System Prompt Additions for Code Quality

## Code Quality Standards

NEVER write production code that contains:

1. **panic!() statements in normal operation paths** - always return Result<T, Error>
2. **memory leaks** - every allocation must have corresponding deallocation
3. **data corruption potential** - all state transitions must preserve data integrity
4. **inconsistent error handling patterns** - establish and follow single pattern

ALWAYS:

1. **Write comprehensive tests BEFORE implementing features**
2. **Include invariant validation in data structures**
3. **Use proper bounds checking for numeric conversions**
4. **Document known bugs immediately and fix them before continuing**
5. **Implement proper separation of concerns**
6. **Use static analysis tools (clippy, miri) before considering code complete**

## Development Process Guards

### TESTING REQUIREMENTS:
- Write failing tests first, then implement to make them pass
- Never commit code with #[should_panic] for bugs - fix the bugs
- Include property-based testing for data structures
- Test memory usage patterns, not just functionality
- Validate all edge cases and boundary conditions

### ARCHITECTURE REQUIREMENTS:
- Explicit error handling - no hidden panics or unwraps
- Memory safety - all unsafe code must be justified and audited
- Performance conscious - avoid unnecessary allocations/clones
- API design - consistent patterns across all public interfaces

### REVIEW CHECKPOINTS:

Before marking any code complete, verify:

1. **No compilation warnings**
2. **All tests pass (including stress tests)**
3. **Memory usage is bounded and predictable**
4. **No data corruption potential in any code path**
5. **Error handling is comprehensive and consistent**
6. **Code is modular and maintainable**
7. **Documentation matches implementation**
8. **Performance benchmarks show acceptable results**

## Rust-Specific Quality Standards

### ERROR HANDLING:
- Use Result<T, Error> for all fallible operations
- Define comprehensive error enums with context
- Never use unwrap() in production code paths
- Use ? operator for error propagation
- Provide meaningful error messages

### MEMORY MANAGEMENT:
- Audit all allocations for corresponding deallocations
- Use RAII patterns consistently
- Prefer borrowing over cloning when possible
- Use Cow<T> for conditional cloning
- Test for memory leaks in long-running scenarios

### DATA STRUCTURE INVARIANTS:
- Document all invariants in comments
- Implement runtime validation (behind feature flags)
- Test invariant preservation across all operations
- Use type system to enforce invariants where possible
- Validate state consistency at module boundaries

### MODULE ORGANIZATION:
- Single responsibility per module
- Clear public/private API boundaries
- Comprehensive module documentation
- Logical dependency hierarchy

## Critical Patterns to Avoid

### DANGEROUS PATTERNS:
```rust
// NEVER DO THIS - production panic
panic!("This should never happen");

// NEVER DO THIS - unchecked conversion
let id = size as u32; // Can overflow on 64-bit

// NEVER DO THIS - ignoring errors
some_operation().unwrap();

// NEVER DO THIS - leaking resources
let resource = allocate();
// ... no corresponding deallocation
```

### PREFERRED PATTERNS:
```rust
// DO THIS - proper error handling
fn operation() -> Result<T, MyError> {
    match risky_operation() {
        Ok(value) => Ok(process(value)),
        Err(e) => Err(MyError::from(e)),
    }
}

// DO THIS - safe conversion
let id: u32 = size.try_into()
    .map_err(|_| Error::InvalidSize(size))?;

// DO THIS - explicit error handling
let result = some_operation()
    .map_err(|e| Error::OperationFailed(e))?;

// DO THIS - RAII resource management
struct ResourceManager {
    resource: Resource,
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        self.resource.cleanup();
    }
}
```

## Testing Standards

### COMPREHENSIVE TEST COVERAGE:
- Unit tests for all public functions
- Integration tests for complex interactions
- Property-based tests for data structures
- Stress tests for long-running operations
- Memory leak detection tests
- Edge case and boundary condition tests

### TEST ORGANIZATION:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_operation() {
        // Test typical usage patterns
    }

    #[test]
    fn test_edge_cases() {
        // Test boundary conditions
    }

    #[test]
    fn test_error_conditions() {
        // Test all error paths
    }

    #[test]
    fn test_invariants_preserved() {
        // Verify data structure invariants
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_invariant_always_holds(input in any::<InputType>()) {
            let result = operation(input);
            assert!(check_invariant(&result));
        }
    }
}
```

### MEMORY TESTING:
```rust
#[test]
fn test_no_memory_leaks() {
    let initial_count = get_allocation_count();

    {
        let mut structure = DataStructure::new();
        // Perform operations that allocate/deallocate
        for i in 0..1000 {
            structure.insert(i);
        }
        for i in 0..500 {
            structure.remove(i);
        }
    } // structure dropped here

    let final_count = get_allocation_count();
    assert_eq!(initial_count, final_count, "Memory leak detected");
}
```

## Documentation Standards

### CODE DOCUMENTATION:
- Document all public APIs with examples
- Explain complex algorithms and data structures
- Document invariants and preconditions
- Include safety notes for unsafe code
- Provide usage examples in doc comments

### ERROR DOCUMENTATION:
```rust
/// Inserts a key-value pair into the tree.
///
/// # Arguments
/// * `key` - The key to insert (must implement Ord)
/// * `value` - The value to associate with the key
///
/// # Returns
/// * `Ok(old_value)` if key existed (returns old value)
/// * `Ok(None)` if key was newly inserted
/// * `Err(Error::InvalidKey)` if key violates constraints
///
/// # Examples
/// ```
/// let mut tree = BPlusTree::new(4)?;
/// assert_eq!(tree.insert(1, "value")?, None);
/// assert_eq!(tree.insert(1, "new")?, Some("value"));
/// ```
///
/// # Panics
/// Never panics - all error conditions return Result
///
/// # Safety
/// This function maintains all tree invariants
pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, Error> {
    // Implementation
}
```

This system prompt addition should prevent the types of critical issues identified in the code review by establishing clear quality standards, testing requirements, and architectural principles that must be followed for all code.
