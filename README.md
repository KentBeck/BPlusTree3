# BPlusTree

High-performance B+ tree implementations for **Rust** and **Python**, designed for efficient range queries and sequential access patterns.

## 🚀 **Dual-Language Implementation**

This project provides **complete, optimized B+ tree implementations** in both languages:

- **🦀 [Rust Implementation](./rust/)** - Zero-cost abstractions, arena-based memory management
- **🐍 [Python Implementation](./python/)** - Competitive with SortedDict, optimized for specific use cases

## 📊 **Performance Highlights**

### **Rust Implementation**

- **32-68% faster range scans** than std::BTreeMap (1.5-2.8x throughput)
- **23-68% faster GET operations** across all dataset sizes
- **2-22% faster insertions** with excellent scaling
- **Trade-off: 34% slower deletes** in optimized scenarios

### **Python Implementation**

- **Up to 2.5x faster** than SortedDict for partial range scans
- **1.4x faster** for medium range queries
- **Excellent scaling** for large dataset iteration

## 🎯 **Choose Your Implementation**

| Use Case                          | Rust                      | Python                        |
| --------------------------------- | ------------------------- | ----------------------------- |
| **Systems programming**           | ✅ Primary choice         | ❌                            |
| **High-performance applications** | ✅ Zero-cost abstractions | ⚠️ Good for specific patterns |
| **Database engines**              | ✅ Full control           | ⚠️ Limited                    |
| **Data analytics**                | ✅ Fast                   | ✅ Great for range queries    |
| **Rapid prototyping**             | ⚠️ Learning curve         | ✅ Easy integration           |
| **Existing Python codebase**      | ❌                        | ✅ Drop-in replacement        |

## 🚀 **Quick Start**

### Rust

```rust
use bplustree::BPlusTreeMap;

let mut tree = BPlusTreeMap::new(16).unwrap();
tree.insert(1, "one");
tree.insert(2, "two");

// Range queries with Rust syntax!
for (key, value) in tree.range(1..=2) {
    println!("{}: {}", key, value);
}
```

### Python

```python
from bplustree import BPlusTree

tree = BPlusTree(capacity=128)
tree[1] = "one"
tree[2] = "two"

# Range queries
for key, value in tree.range(1, 2):
    print(f"{key}: {value}")
```

## 📖 **Documentation**

- **📚 [Technical Documentation](./rust/docs/)** - Architecture, algorithms, benchmarks
- **🦀 [Rust Documentation](./rust/README.md)** - Rust-specific usage and examples
- **🐍 [Python Documentation](./python/README.md)** - Python-specific usage and examples

## Performance Characteristics

**BPlusTreeMap demonstrates significant performance advantages in range operations and read-heavy workloads compared to Rust's standard BTreeMap.** Comprehensive benchmarking across dataset sizes from 1K to 10M entries reveals that BPlusTreeMap consistently outperforms BTreeMap in range scans by 32-68%, delivering 1.5-2.8x higher throughput (67K-212K vs 44K-83K items/ms). GET operations show similarly strong advantages, with BPlusTreeMap performing 23-68% faster across all scales, making it particularly well-suited for read-heavy applications and analytical workloads.

**Insert performance is competitive to superior, with BPlusTreeMap showing 2-22% faster insertion speeds depending on dataset size and configuration.** The implementation scales exceptionally well, with larger datasets (>1M entries) showing the most pronounced advantages. However, delete operations represent the primary trade-off, with BPlusTreeMap performing 34% slower in optimized scenarios and 1.7-10.5x slower depending on capacity configuration, particularly at high capacities (1024+ elements per node).

**Capacity configuration is critical for optimal performance.** The B+ tree implementation allows tuning of node capacity, with optimal settings varying by use case: capacity 64-128 for datasets under 10K entries, 128-256 for medium datasets (10K-100K), and 256-512 for large datasets (100K-1M+). Proper configuration can achieve near-optimal performance across all operations, while misconfiguration (particularly high capacities with delete-heavy workloads) can significantly impact performance.

**BPlusTreeMap is recommended for range-heavy workloads (>20% range scans), read-heavy applications (>60% gets), large dataset analytics, and mixed workloads with light-to-moderate delete operations (<15% deletes).** Standard BTreeMap remains preferable for delete-heavy workloads, small datasets with unknown access patterns, or applications requiring zero configuration. The performance characteristics make BPlusTreeMap particularly valuable for database-like applications, time-series analysis, and any scenario where range queries and sequential access patterns dominate.

## 🏗️ **Architecture**

Both implementations share core design principles:

- **Arena-based memory management** for efficiency
- **Linked leaf nodes** for fast sequential access
- **Hybrid navigation** combining tree traversal + linked list iteration
- **Optimized rebalancing** with reduced duplicate lookups
- **Comprehensive testing** including adversarial test patterns

## 🛠️ **Development**

### Rust Development

```bash
cd rust/
cargo test --features testing
cargo bench
```

### Python Development

```bash
cd python/
pip install -e .
python -m pytest tests/
```

### Cross-Language Benchmarking

```bash
python scripts/analyze_benchmarks.py
```

## 🤝 **Contributing**

This project follows **Test-Driven Development** and **Tidy First** principles:

1. **Write tests first** - All features start with failing tests
2. **Small, focused commits** - Separate structural and behavioral changes
3. **Comprehensive validation** - Both implementations tested against reference implementations
4. **Performance awareness** - All changes benchmarked for performance impact

## 📄 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 **Links**

- **[GitHub Repository](https://github.com/KentBeck/BPlusTree3)**
- **[Rust Crate](https://crates.io/crates/bplustree)** _(coming soon)_
- **[Python Package](https://pypi.org/project/bplustree/)** _(coming soon)_

---

> Built with ❤️ following Kent Beck's **Test-Driven Development** methodology.
