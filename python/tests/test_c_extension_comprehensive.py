"""
Comprehensive test suite for C extension to identify and fix all bugs.
"""

import sys
import os
import random

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest

try:
    import bplustree_c

    HAS_C_EXTENSION = True
except ImportError as e:
    pytest.skip(f"C extension not available: {e}", allow_module_level=True)


def test_empty_tree():
    """Test operations on empty tree."""
    print("Testing empty tree...")
    tree = bplustree_c.BPlusTree(capacity=4)

    assert len(tree) == 0, f"Empty tree should have length 0, got {len(tree)}"

    # Test KeyError on empty tree
    try:
        _ = tree[1]
        assert False, "Should raise KeyError on empty tree"
    except KeyError:
        pass

    # Test empty iteration
    keys = list(tree.keys())
    assert keys == [], f"Empty tree keys should be [], got {keys}"

    items = list(tree.items())
    assert items == [], f"Empty tree items should be [], got {items}"

    print("✓ Empty tree tests passed")


def test_single_item():
    """Test tree with single item."""
    print("Testing single item...")
    tree = bplustree_c.BPlusTree(capacity=4)

    tree[42] = 84
    assert len(tree) == 1, f"Single item tree should have length 1, got {len(tree)}"

    assert tree[42] == 84, f"tree[42] should be 84, got {tree[42]}"

    keys = list(tree.keys())
    assert keys == [42], f"Single item keys should be [42], got {keys}"

    items = list(tree.items())
    assert items == [(42, 84)], f"Single item items should be [(42, 84)], got {items}"

    print("✓ Single item tests passed")


def test_sequential_insert_small():
    """Test sequential insertion with small capacity to force splits."""
    print("Testing sequential insertion with capacity 4...")
    tree = bplustree_c.BPlusTree(capacity=4)

    # Insert items that will cause multiple splits
    for i in range(20):
        tree[i] = i * 10
        assert (
            len(tree) == i + 1
        ), f"After inserting {i+1} items, length should be {i+1}, got {len(tree)}"

    # Verify all items
    print("Verifying all items...")
    for i in range(20):
        try:
            value = tree[i]
            expected = i * 10
            assert value == expected, f"tree[{i}] should be {expected}, got {value}"
        except KeyError:
            print(f"ERROR: tree[{i}] not found!")
            # Debug: show what keys are actually in the tree
            keys = list(tree.keys())
            print(f"Available keys: {keys}")
            raise

    # Test iteration
    keys = list(tree.keys())
    expected_keys = list(range(20))
    assert keys == expected_keys, f"Keys should be {expected_keys}, got {keys}"

    print("✓ Sequential insertion tests passed")


def test_random_insert_small():
    """Test random insertion with small capacity."""
    print("Testing random insertion with capacity 4...")
    tree = bplustree_c.BPlusTree(capacity=4)

    keys_to_insert = list(range(20))
    random.shuffle(keys_to_insert)

    inserted_keys = set()
    for i, key in enumerate(keys_to_insert):
        tree[key] = key * 10
        inserted_keys.add(key)
        assert (
            len(tree) == i + 1
        ), f"After inserting {i+1} items, length should be {i+1}, got {len(tree)}"

        # Verify all previously inserted keys still work
        for prev_key in inserted_keys:
            try:
                value = tree[prev_key]
                expected = prev_key * 10
                assert (
                    value == expected
                ), f"After inserting {key}, tree[{prev_key}] should be {expected}, got {value}"
            except KeyError:
                print(f"ERROR: After inserting {key}, tree[{prev_key}] not found!")
                keys = list(tree.keys())
                print(f"Available keys: {sorted(keys)}")
                print(f"Expected keys: {sorted(inserted_keys)}")
                raise

    print("✓ Random insertion tests passed")


def test_duplicate_keys():
    """Test updating existing keys."""
    print("Testing duplicate key updates...")
    tree = bplustree_c.BPlusTree(capacity=4)

    # Insert initial values
    for i in range(10):
        tree[i] = i

    # Update with new values
    for i in range(10):
        tree[i] = i * 100

    # Verify updates
    for i in range(10):
        value = tree[i]
        expected = i * 100
        assert value == expected, f"tree[{i}] should be {expected}, got {value}"

    assert len(tree) == 10, f"Tree should still have 10 items, got {len(tree)}"

    print("✓ Duplicate key tests passed")


def test_key_error():
    """Test KeyError for non-existent keys."""
    print("Testing KeyError for non-existent keys...")
    tree = bplustree_c.BPlusTree(capacity=4)

    # Insert some items
    for i in range(0, 20, 2):  # Even numbers only
        tree[i] = i * 10

    # Test that odd numbers raise KeyError
    for i in range(1, 20, 2):  # Odd numbers
        try:
            _ = tree[i]
            assert False, f"tree[{i}] should raise KeyError"
        except KeyError:
            pass

    print("✓ KeyError tests passed")


def test_iteration_order():
    """Test that iteration maintains sorted order."""
    print("Testing iteration order...")
    tree = bplustree_c.BPlusTree(capacity=4)

    # Insert in random order
    keys_to_insert = list(range(50, 0, -1))  # Reverse order
    for key in keys_to_insert:
        tree[key] = key * 2

    # Check that keys() returns sorted order
    keys = list(tree.keys())
    expected_keys = list(range(1, 51))
    assert (
        keys == expected_keys
    ), f"Keys not in sorted order. Expected {expected_keys[:10]}..., got {keys[:10]}..."

    # Check that items() returns sorted order
    items = list(tree.items())
    for i, (key, value) in enumerate(items):
        expected_key = i + 1
        expected_value = expected_key * 2
        assert (
            key == expected_key and value == expected_value
        ), f"Item {i} should be ({expected_key}, {expected_value}), got ({key}, {value})"

    print("✓ Iteration order tests passed")


def test_large_capacity():
    """Test with larger capacity to ensure it works without frequent splits."""
    print("Testing with large capacity (128)...")
    tree = bplustree_c.BPlusTree(capacity=128)

    # Insert many items
    for i in range(1000):
        tree[i] = i * 3

    # Verify random sample
    for i in range(0, 1000, 100):
        value = tree[i]
        expected = i * 3
        assert value == expected, f"tree[{i}] should be {expected}, got {value}"

    assert len(tree) == 1000, f"Tree should have 1000 items, got {len(tree)}"

    print("✓ Large capacity tests passed")


def test_string_keys():
    """Test with string keys to ensure comparison works correctly."""
    print("Testing string keys...")
    tree = bplustree_c.BPlusTree(capacity=4)

    string_keys = ["apple", "banana", "cherry", "date", "elderberry", "fig", "grape"]
    for key in string_keys:
        tree[key] = len(key)

    # Verify all string keys
    for key in string_keys:
        value = tree[key]
        expected = len(key)
        assert value == expected, f"tree['{key}'] should be {expected}, got {value}"

    # Check sorted order
    keys = list(tree.keys())
    expected_keys = sorted(string_keys)
    assert (
        keys == expected_keys
    ), f"String keys not in sorted order. Expected {expected_keys}, got {keys}"

    print("✓ String key tests passed")


def test_mixed_types():
    """Test with mixed key types (if supported)."""
    print("Testing mixed types...")
    tree = bplustree_c.BPlusTree(capacity=4)

    # This might fail if Python comparison doesn't work between types
    try:
        tree[1] = "one"
        tree["two"] = 2
        tree[3.0] = "three"

        assert tree[1] == "one"
        assert tree["two"] == 2
        assert tree[3.0] == "three"

        print("✓ Mixed type tests passed")
    except Exception as e:
        print(f"Mixed types not supported (expected): {e}")


def run_all_tests():
    """Run all tests and report results."""
    if not HAS_C_EXTENSION:
        print("C extension not available, skipping tests")
        return

    print("Running Comprehensive C Extension Tests")
    print("=" * 50)

    tests = [
        test_empty_tree,
        test_single_item,
        test_sequential_insert_small,
        test_random_insert_small,
        test_duplicate_keys,
        test_key_error,
        test_iteration_order,
        test_large_capacity,
        test_string_keys,
        test_mixed_types,
    ]

    passed = 0
    failed = 0

    for test in tests:
        try:
            test()
            passed += 1
        except Exception as e:
            print(f"✗ {test.__name__} FAILED: {e}")
            failed += 1
            # Continue with other tests

    print("\n" + "=" * 50)
    print(f"Test Results: {passed} passed, {failed} failed")

    if failed == 0:
        print("🎉 All tests passed! C extension is working correctly.")
    else:
        print("🚨 Some tests failed. C extension needs fixes.")

    return failed == 0


if __name__ == "__main__":
    run_all_tests()
