"""
Test that ensures NO segfaults occur under any circumstances.
A segfault is always a critical bug that must be fixed.
"""

import pytest
import sys
import os
import random
import gc

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    import bplustree_c

    HAS_C_EXTENSION = True
except ImportError:
    HAS_C_EXTENSION = False


class TestNoSegfaults:
    """Test suite to ensure no segfaults occur."""

    def test_large_sequential_insert(self):
        """Test large sequential insertions that previously caused segfaults."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=128)

        # Insert 10,000 items sequentially
        for i in range(10000):
            tree[i] = i * 2

            # Verify tree is still functional every 1000 items
            if i % 1000 == 0:
                assert len(tree) == i + 1, f"Tree size incorrect at {i}"
                assert tree[i] == i * 2, f"Value incorrect at {i}"

        print(f"✓ Successfully inserted 10,000 sequential items")

    def test_large_random_insert(self):
        """Test large random insertions."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=128)

        # Generate random keys
        keys = list(range(5000))
        random.shuffle(keys)

        # Insert all keys
        for i, key in enumerate(keys):
            tree[key] = key * 2

            # Verify periodically
            if i % 500 == 0:
                assert len(tree) == i + 1, f"Tree size incorrect at insertion {i}"

        # Verify all keys are present
        for key in keys:
            assert tree[key] == key * 2, f"Key {key} not found or has wrong value"

        print(f"✓ Successfully inserted 5,000 random items")

    def test_mixed_operations_large(self):
        """Test mixed insert/lookup/delete operations on large dataset."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=64)

        # Phase 1: Insert large dataset
        keys = list(range(3000))
        random.shuffle(keys)

        for key in keys:
            tree[key] = key * 10

        print(f"Inserted {len(keys)} items")

        # Phase 2: Random lookups
        lookup_keys = random.sample(keys, 1000)
        for key in lookup_keys:
            value = tree[key]
            assert value == key * 10, f"Lookup failed for key {key}"

        print(f"Performed 1000 lookups")

        # Phase 3: Random deletions
        delete_keys = random.sample(keys, 500)
        for key in delete_keys:
            del tree[key]

        print(f"Deleted 500 items")

        # Phase 4: Verify remaining keys
        remaining_keys = [k for k in keys if k not in delete_keys]
        for key in remaining_keys:
            value = tree[key]
            assert value == key * 10, f"Key {key} missing after deletions"

        assert len(tree) == len(remaining_keys), f"Tree size incorrect after deletions"

        print(f"✓ Mixed operations completed successfully")

    def test_stress_with_iterations(self):
        """Stress test with many iterations to catch memory issues."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        for iteration in range(10):
            tree = bplustree_c.BPlusTree(capacity=32)

            # Insert 1000 items
            for i in range(1000):
                tree[i] = i

            # Iterate over all items
            keys = list(tree.keys())
            items = list(tree.items())

            assert len(keys) == 1000, f"Iteration {iteration}: wrong key count"
            assert len(items) == 1000, f"Iteration {iteration}: wrong item count"

            # Delete half
            for i in range(0, 1000, 2):
                del tree[i]

            assert (
                len(tree) == 500
            ), f"Iteration {iteration}: wrong size after deletions"

            # Clean up
            del tree
            gc.collect()

        print(f"✓ Completed 10 stress iterations")

    def test_capacity_edge_cases(self):
        """Test various capacity values that might cause issues."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        capacities = [4, 8, 16, 32, 64, 128, 256, 512, 1024]

        for capacity in capacities:
            tree = bplustree_c.BPlusTree(capacity=capacity)

            # Insert enough items to force multiple splits
            num_items = capacity * 10
            for i in range(num_items):
                tree[i] = i * 2

            # Verify all items
            for i in range(num_items):
                assert tree[i] == i * 2, f"Capacity {capacity}: item {i} incorrect"

            assert len(tree) == num_items, f"Capacity {capacity}: wrong final size"

        print(f"✓ Tested {len(capacities)} different capacities")

    def test_boundary_values(self):
        """Test boundary values that might cause buffer overflows."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=128)

        # Test with very large numbers
        large_values = [
            2**31 - 1,  # Max 32-bit signed int
            2**32 - 1,  # Max 32-bit unsigned int
            2**63 - 1,  # Max 64-bit signed int
        ]

        for i, val in enumerate(large_values):
            tree[val] = i
            assert tree[val] == i, f"Large value {val} failed"

        # Test with negative numbers
        negative_values = [-1, -100, -(2**31)]
        for i, val in enumerate(negative_values):
            tree[val] = i + 1000
            assert tree[val] == i + 1000, f"Negative value {val} failed"

        print(f"✓ Boundary value tests passed")

    def test_memory_pressure(self):
        """Test under memory pressure to catch allocation issues."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        trees = []

        # Create many trees to pressure memory
        for i in range(50):
            tree = bplustree_c.BPlusTree(capacity=64)

            # Fill each tree
            for j in range(200):
                tree[j] = j * i

            trees.append(tree)

        # Verify all trees are still valid
        for i, tree in enumerate(trees):
            assert len(tree) == 200, f"Tree {i} has wrong size"
            assert tree[0] == 0, f"Tree {i} first item wrong"
            assert tree[199] == 199 * i, f"Tree {i} last item wrong"

        print(f"✓ Created and verified {len(trees)} trees under memory pressure")


def test_no_segfaults():
    """Run all segfault prevention tests."""
    if not HAS_C_EXTENSION:
        print("C extension not available, skipping segfault tests")
        pytest.skip("C extension not available")

    test_suite = TestNoSegfaults()

    tests = [
        test_suite.test_large_sequential_insert,
        test_suite.test_large_random_insert,
        test_suite.test_mixed_operations_large,
        test_suite.test_stress_with_iterations,
        test_suite.test_capacity_edge_cases,
        test_suite.test_boundary_values,
        test_suite.test_memory_pressure,
    ]

    print("Running Segfault Prevention Tests")
    print("=" * 50)
    print("⚠️  ANY segfault is a critical bug that must be fixed!")
    print()

    passed = 0
    failed = 0

    for test in tests:
        test_name = test.__name__
        try:
            print(f"Running {test_name}...")
            test()
            print(f"✅ {test_name} PASSED")
            passed += 1
        except Exception as e:
            print(f"❌ {test_name} FAILED: {e}")
            failed += 1
            import traceback

            traceback.print_exc()

    print("\n" + "=" * 50)
    print(f"Segfault Prevention Results: {passed} passed, {failed} failed")

    if failed == 0:
        print("🎉 NO SEGFAULTS! C extension is memory-safe.")
    else:
        print("🚨 CRITICAL: Fix all issues before proceeding!")
        assert (
            False
        ), f"CRITICAL: {failed} segfault tests failed - must fix immediately!"

    # Explicitly assert success
    assert (
        failed == 0
    ), f"CRITICAL: {failed} segfault tests failed - must fix immediately!"


if __name__ == "__main__":
    test_no_segfaults()
