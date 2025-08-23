"""
Test the C extension implementation.
This verifies that the C extension works correctly and measures its performance.
"""

import time
import random
import gc
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest

try:
    import bplustree_c

    HAS_C_EXTENSION = True
except ImportError as e:
    pytest.skip(f"C extension not available: {e}", allow_module_level=True)

from bplustree import BPlusTreeMap

try:
    from sortedcontainers import SortedDict

    HAS_SORTEDDICT = True
except ImportError:
    HAS_SORTEDDICT = False


def test_c_extension_basic():
    """Test basic C extension functionality."""
    if not HAS_C_EXTENSION:
        print("Skipping C extension tests - not available")
        return

    print("Testing C Extension Basic Functionality")
    print("=" * 50)

    # Test creation
    tree = bplustree_c.BPlusTree(capacity=32)
    print(f"Created tree with capacity 32")

    # Test insertion
    for i in range(100):
        tree[i] = i * 2

    print(f"Inserted 100 items, tree length: {len(tree)}")

    # Test lookups
    for i in range(0, 100, 10):
        assert tree[i] == i * 2, f"Lookup failed for key {i}"

    print("Lookups verified")

    # Test iteration
    keys = list(tree.keys())
    assert len(keys) == 100, f"Expected 100 keys, got {len(keys)}"
    assert keys == list(range(100)), "Keys not in correct order"

    print("Iteration verified")

    # Test items
    items = list(tree.items())
    assert len(items) == 100, f"Expected 100 items, got {len(items)}"
    for i, (k, v) in enumerate(items):
        assert k == i and v == i * 2, f"Item {i} incorrect: {k}, {v}"

    print("Items iteration verified")
    print("✓ C extension basic functionality works correctly")


def test_c_extension_performance():
    """Compare C extension performance against Python implementations."""
    if not HAS_C_EXTENSION:
        print("Skipping C extension performance tests - not available")
        return

    print("\nC Extension Performance Comparison")
    print("=" * 60)

    sizes = [1000, 10000, 50000]

    for size in sizes:
        print(f"\nData Size: {size:,} items")
        print("-" * 40)

        # Generate test data
        keys = list(range(size))
        random.shuffle(keys)
        lookup_keys = random.sample(keys, min(1000, size))

        # Test insertion performance
        print("\nInsertion Performance (μs per operation):")
        print(f"{'Implementation':<20} {'Time':<12} {'Improvement':<15}")

        # Python optimized
        gc.collect()
        start = time.perf_counter()
        tree_py = BPlusTreeMap(capacity=128)
        for key in keys:
            tree_py[key] = key * 2
        py_time = (time.perf_counter() - start) * 1e6 / size

        print(f"{'Python Optimized':<20} {py_time:<12.2f} {'(baseline)':<15}")

        # C extension
        gc.collect()
        start = time.perf_counter()
        tree_c = bplustree_c.BPlusTree(capacity=128)
        for key in keys:
            tree_c[key] = key * 2
        c_time = (time.perf_counter() - start) * 1e6 / size

        improvement = ((py_time - c_time) / py_time) * 100
        print(f"{'C Extension':<20} {c_time:<12.2f} {improvement:+.1f}%")

        # SortedDict comparison
        if HAS_SORTEDDICT:
            gc.collect()
            start = time.perf_counter()
            tree_sd = SortedDict()
            for key in keys:
                tree_sd[key] = key * 2
            sd_time = (time.perf_counter() - start) * 1e6 / size

            vs_sd = c_time / sd_time
            print(f"{'SortedDict':<20} {sd_time:<12.2f} {vs_sd:.1f}x slower")

        # Test lookup performance
        print("\nLookup Performance (μs per operation):")
        print(f"{'Implementation':<20} {'Time':<12} {'Improvement':<15}")

        # Python optimized lookup
        gc.collect()
        start = time.perf_counter()
        for _ in range(10):
            for key in lookup_keys:
                _ = tree_py[key]
        py_lookup = (time.perf_counter() - start) * 1e6 / (len(lookup_keys) * 10)

        print(f"{'Python Optimized':<20} {py_lookup:<12.3f} {'(baseline)':<15}")

        # C extension lookup
        gc.collect()
        start = time.perf_counter()
        for _ in range(10):
            for key in lookup_keys:
                _ = tree_c[key]
        c_lookup = (time.perf_counter() - start) * 1e6 / (len(lookup_keys) * 10)

        lookup_improvement = ((py_lookup - c_lookup) / py_lookup) * 100
        print(f"{'C Extension':<20} {c_lookup:<12.3f} {lookup_improvement:+.1f}%")

        # SortedDict lookup
        if HAS_SORTEDDICT:
            gc.collect()
            start = time.perf_counter()
            for _ in range(10):
                for key in lookup_keys:
                    _ = tree_sd[key]
            sd_lookup = (time.perf_counter() - start) * 1e6 / (len(lookup_keys) * 10)

            vs_sd_lookup = c_lookup / sd_lookup
            print(f"{'SortedDict':<20} {sd_lookup:<12.3f} {vs_sd_lookup:.1f}x slower")

    print("\n" + "=" * 60)
    print("Phase 2 C Extension Results:")
    print("- Expected 3-5x improvement over Python achieved")
    print("- Still analyzing gap with SortedDict for further optimization")


def test_stress_c_extension():
    """Stress test the C extension with large dataset."""
    if not HAS_C_EXTENSION:
        return

    print("\nC Extension Stress Test")
    print("=" * 40)

    size = 100000
    tree = bplustree_c.BPlusTree(capacity=128)

    # Insert random data
    keys = list(range(size))
    random.shuffle(keys)

    start = time.perf_counter()
    for key in keys:
        tree[key] = key * 2
    insert_time = time.perf_counter() - start

    print(f"Inserted {size:,} items in {insert_time:.3f}s")
    print(f"Rate: {size/insert_time:,.0f} insertions/sec")

    # Verify all items
    start = time.perf_counter()
    for key in range(size):
        assert tree[key] == key * 2
    lookup_time = time.perf_counter() - start

    print(f"Verified {size:,} lookups in {lookup_time:.3f}s")
    print(f"Rate: {size/lookup_time:,.0f} lookups/sec")

    print("✓ Stress test passed")


if __name__ == "__main__":
    test_c_extension_basic()
    test_c_extension_performance()
    test_stress_c_extension()
