"""
Minimal test for node split bug - smallest possible failing test.
Following TDD: write the smallest test that replicates the problem.
"""

import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest

try:
    import bplustree_c

    HAS_C_EXTENSION = True
except ImportError as e:
    pytest.skip(f"C extension not available: {e}", allow_module_level=True)


def test_single_node_split_maintains_order():
    """
    SMALLEST POSSIBLE TEST: Single node split must maintain sorted order.
    This test MUST fail until the bug is fixed.
    """
    if not HAS_C_EXTENSION:
        pytest.skip("C extension not available")

    # Create tree with capacity 4 - split will happen after 4 items
    tree = bplustree_c.BPlusTree(capacity=4)

    # Insert exactly enough items to cause ONE split
    for i in range(5):  # 5 items in capacity-4 tree = 1 split
        tree[i] = i * 10

    # After split, iteration MUST return keys in sorted order
    keys = list(tree.keys())

    print(f"Keys after single split: {keys}")
    print(f"Expected: [0, 1, 2, 3, 4]")

    # THE CRITICAL TEST: keys must be sorted
    assert keys == [
        0,
        1,
        2,
        3,
        4,
    ], f"Keys not in sorted order after single node split. Got: {keys}"
    print("✅ PASSED: Keys in correct order after split")


def test_two_splits_maintains_order():
    """
    Second minimal test: Two splits must maintain sorted order.
    """
    if not HAS_C_EXTENSION:
        pytest.skip("C extension not available")

    # Create tree with capacity 4
    tree = bplustree_c.BPlusTree(capacity=4)

    # Insert enough items to cause TWO splits
    for i in range(9):  # Should cause 2 splits
        tree[i] = i * 10

    # Keys must still be sorted
    keys = list(tree.keys())
    expected = list(range(9))

    print(f"Keys after two splits: {keys}")
    print(f"Expected: {expected}")

    assert keys == expected, f"Keys not in sorted order after two splits. Got: {keys}"
    print("✅ PASSED: Keys in correct order after two splits")


if __name__ == "__main__":
    print("Running MINIMAL node split tests...")
    print("=" * 50)

    # Test 1: Single split
    result1 = test_single_node_split_maintains_order()

    # Test 2: Two splits
    result2 = test_two_splits_maintains_order()

    if result1 and result2:
        print("\n🎉 All minimal tests PASSED")
    else:
        print("\n🚨 MINIMAL tests FAILED - must fix before proceeding")
