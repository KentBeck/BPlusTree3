"""
Test the complete dictionary API for BPlusTreeMap.

This module tests all dictionary-like methods to ensure compatibility
with Python's dict interface.
"""

import pytest
from typing import Any, Dict

# Import the BPlusTreeMap from the package (will use C extension if available)
try:
    # Try to import from installed package first
    import bplustree

    BPlusTreeMap = bplustree.BPlusTreeMap
except ImportError:
    # Fall back to local import if package not installed
    import sys
    import os

    sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    import bplustree

    BPlusTreeMap = bplustree.BPlusTreeMap


class TestDictionaryAPI:
    """Test all dictionary-like methods of BPlusTreeMap."""

    def setup_method(self):
        """Set up test fixtures before each test method."""
        self.tree = BPlusTreeMap(capacity=4)
        # Add some initial data
        for i in range(10):
            self.tree[i] = f"value_{i}"

    def test_clear(self):
        """Test the clear() method."""
        # Verify tree has data
        assert len(self.tree) == 10
        assert 5 in self.tree

        # Clear the tree
        self.tree.clear()

        # Verify tree is empty
        assert len(self.tree) == 0
        assert 5 not in self.tree
        assert bool(self.tree) == False

        # Verify we can still add data after clearing
        self.tree[100] = "new_value"
        assert len(self.tree) == 1
        assert self.tree[100] == "new_value"

    def test_get_with_default(self):
        """Test the get() method with default values."""
        # Test existing key
        assert self.tree.get(5) == "value_5"
        assert self.tree.get(5, "default") == "value_5"

        # Test non-existing key with default
        assert self.tree.get(100) is None
        assert self.tree.get(100, "default") == "default"
        assert self.tree.get(100, 42) == 42

        # Test that tree is unchanged
        assert len(self.tree) == 10

    def test_pop_with_key_present(self):
        """Test pop() when key exists."""
        # Pop existing key
        value = self.tree.pop(5)
        assert value == "value_5"

        # Verify key is removed
        assert 5 not in self.tree
        assert len(self.tree) == 9

        # Verify other keys still exist
        assert self.tree[4] == "value_4"
        assert self.tree[6] == "value_6"

    def test_pop_with_key_missing_no_default(self):
        """Test pop() when key doesn't exist and no default."""
        # Should raise KeyError
        with pytest.raises(KeyError, match="100"):
            self.tree.pop(100)

        # Tree should be unchanged
        assert len(self.tree) == 10

    def test_pop_with_key_missing_with_default(self):
        """Test pop() when key doesn't exist but default provided."""
        # Should return default
        assert self.tree.pop(100, "default") == "default"
        assert self.tree.pop(100, None) is None
        assert self.tree.pop(100, 42) == 42

        # Tree should be unchanged
        assert len(self.tree) == 10

    def test_pop_argument_validation(self):
        """Test pop() argument validation."""
        # Too many arguments
        with pytest.raises(TypeError, match="pop expected at most 2 arguments, got 3"):
            self.tree.pop(1, "default", "extra")

    def test_popitem_with_data(self):
        """Test popitem() when tree has data."""
        original_len = len(self.tree)

        # Pop an item
        key, value = self.tree.popitem()

        # Should be the first item (leftmost)
        assert key == 0
        assert value == "value_0"

        # Verify item is removed
        assert len(self.tree) == original_len - 1
        assert key not in self.tree

    def test_popitem_empty_tree(self):
        """Test popitem() when tree is empty."""
        empty_tree = BPlusTreeMap(capacity=4)

        with pytest.raises(KeyError, match="popitem\\(\\): tree is empty"):
            empty_tree.popitem()

    def test_popitem_until_empty(self):
        """Test popping all items until tree is empty."""
        items = []
        while self.tree:
            items.append(self.tree.popitem())

        # Should have popped all items in order
        assert len(items) == 10
        assert items == [(i, f"value_{i}") for i in range(10)]

        # Tree should be empty
        assert len(self.tree) == 0

        # Now popitem should raise KeyError
        with pytest.raises(KeyError):
            self.tree.popitem()

    def test_setdefault_new_key(self):
        """Test setdefault() with new key."""
        # Set default for new key
        result = self.tree.setdefault(100, "new_default")

        assert result == "new_default"
        assert self.tree[100] == "new_default"
        assert len(self.tree) == 11

    def test_setdefault_existing_key(self):
        """Test setdefault() with existing key."""
        # Should return existing value, not default
        result = self.tree.setdefault(5, "should_not_be_used")

        assert result == "value_5"
        assert self.tree[5] == "value_5"  # Value unchanged
        assert len(self.tree) == 10  # Length unchanged

    def test_setdefault_none_default(self):
        """Test setdefault() with None as default."""
        result = self.tree.setdefault(100)

        assert result is None
        assert self.tree[100] is None
        assert len(self.tree) == 11

    def test_update_with_dict(self):
        """Test update() with a dictionary."""
        update_data = {100: "hundred", 101: "hundred_one", 5: "updated_five"}

        self.tree.update(update_data)

        # Check new keys added
        assert self.tree[100] == "hundred"
        assert self.tree[101] == "hundred_one"

        # Check existing key updated
        assert self.tree[5] == "updated_five"

        # Check length
        assert len(self.tree) == 12

    def test_update_with_another_bplustree(self):
        """Test update() with another BPlusTreeMap."""
        other_tree = BPlusTreeMap(capacity=8)
        other_tree[100] = "hundred"
        other_tree[101] = "hundred_one"
        other_tree[5] = "updated_five"

        self.tree.update(other_tree)

        # Check new keys added
        assert self.tree[100] == "hundred"
        assert self.tree[101] == "hundred_one"

        # Check existing key updated
        assert self.tree[5] == "updated_five"

        # Check length
        assert len(self.tree) == 12

    def test_update_with_iterable_of_pairs(self):
        """Test update() with iterable of (key, value) pairs."""
        pairs = [(100, "hundred"), (101, "hundred_one"), (5, "updated_five")]

        self.tree.update(pairs)

        # Check new keys added
        assert self.tree[100] == "hundred"
        assert self.tree[101] == "hundred_one"

        # Check existing key updated
        assert self.tree[5] == "updated_five"

        # Check length
        assert len(self.tree) == 12

    def test_update_with_generator(self):
        """Test update() with a generator of pairs."""

        def pair_generator():
            yield (100, "hundred")
            yield (101, "hundred_one")
            yield (5, "updated_five")

        self.tree.update(pair_generator())

        # Check updates applied
        assert self.tree[100] == "hundred"
        assert self.tree[101] == "hundred_one"
        assert self.tree[5] == "updated_five"

    def test_copy(self):
        """Test copy() method creates a shallow copy."""
        # Create a copy
        copied_tree = self.tree.copy()

        # Should be a different object
        assert copied_tree is not self.tree

        # But should have same capacity and contents
        assert copied_tree.capacity == self.tree.capacity
        assert len(copied_tree) == len(self.tree)

        # Check all key-value pairs
        for key in range(10):
            assert copied_tree[key] == self.tree[key]

        # Modifications to copy shouldn't affect original
        copied_tree[100] = "new_value"
        assert 100 not in self.tree
        assert len(self.tree) == 10

        # Modifications to original shouldn't affect copy
        self.tree[200] = "another_value"
        assert 200 not in copied_tree

    def test_copy_empty_tree(self):
        """Test copy() of empty tree."""
        empty_tree = BPlusTreeMap(capacity=16)
        copied = empty_tree.copy()

        assert len(copied) == 0
        assert copied.capacity == 16
        assert copied is not empty_tree

    def test_dict_compatibility(self):
        """Test that BPlusTreeMap behaves like a standard dict."""
        # Create equivalent dict
        ref_dict = {i: f"value_{i}" for i in range(10)}

        # Test all basic operations match dict behavior
        for key in range(10):
            assert self.tree[key] == ref_dict[key]
            assert (key in self.tree) == (key in ref_dict)

        assert len(self.tree) == len(ref_dict)
        assert bool(self.tree) == bool(ref_dict)

        # Test get() matches dict.get()
        assert self.tree.get(5) == ref_dict.get(5)
        assert self.tree.get(100) == ref_dict.get(100)
        assert self.tree.get(100, "default") == ref_dict.get(100, "default")

        # Test pop() matches dict.pop()
        tree_val = self.tree.pop(5)
        dict_val = ref_dict.pop(5)
        assert tree_val == dict_val

        # Test setdefault() matches dict.setdefault()
        tree_result = self.tree.setdefault(100, "default")
        dict_result = ref_dict.setdefault(100, "default")
        assert tree_result == dict_result

    def test_edge_cases(self):
        """Test edge cases and error conditions."""
        # Test with None values (but comparable keys)
        self.tree[100] = None
        assert self.tree[100] is None
        assert 100 in self.tree

        # Test with various value types
        self.tree[101] = [1, 2, 3]
        self.tree[102] = {"nested": "dict"}
        self.tree[103] = (1, 2, 3)

        assert self.tree[101] == [1, 2, 3]
        assert self.tree[102] == {"nested": "dict"}
        assert self.tree[103] == (1, 2, 3)

        # Test clear after mixed types
        original_len = len(self.tree)
        self.tree.clear()
        assert len(self.tree) == 0
        assert original_len > 10  # We had our original 10 plus 4 new items

    def test_method_chaining_compatibility(self):
        """Test that methods that should return None do so (for chaining compatibility)."""
        # These methods should return None (like dict)
        assert self.tree.clear() is None
        assert self.tree.update({100: "test"}) is None

        # These methods should return values
        assert self.tree.get(100) == "test"
        assert isinstance(self.tree.copy(), BPlusTreeMap)


class TestDictionaryAPILargeDataset:
    """Test dictionary API with larger datasets to ensure performance."""

    def test_large_dataset_operations(self):
        """Test dictionary operations with large dataset."""
        tree = BPlusTreeMap(capacity=32)

        # Insert large dataset
        data = {i: f"value_{i}" for i in range(1000)}
        tree.update(data)

        assert len(tree) == 1000

        # Test copy with large dataset
        copied = tree.copy()
        assert len(copied) == 1000

        # Test clear with large dataset
        tree.clear()
        assert len(tree) == 0
        assert len(copied) == 1000  # Copy should be unaffected


if __name__ == "__main__":
    # Run the tests
    import unittest

    # Convert pytest tests to unittest for standalone running
    suite = unittest.TestSuite()

    # Add test methods manually
    test_instance = TestDictionaryAPI()
    test_instance.setup_method()

    print("Running dictionary API tests...")

    test_methods = [
        "test_clear",
        "test_get_with_default",
        "test_pop_with_key_present",
        "test_pop_with_key_missing_no_default",
        "test_pop_with_key_missing_with_default",
        "test_popitem_with_data",
        "test_popitem_empty_tree",
        "test_setdefault_new_key",
        "test_setdefault_existing_key",
        "test_update_with_dict",
        "test_copy",
    ]

    passed = 0
    failed = 0

    for method_name in test_methods:
        try:
            test_instance.setup_method()  # Reset state
            method = getattr(test_instance, method_name)
            method()
            print(f"✓ {method_name}")
            passed += 1
        except Exception as e:
            print(f"✗ {method_name}: {e}")
            failed += 1

    print(f"\nResults: {passed} passed, {failed} failed")

    if failed == 0:
        print("All dictionary API tests passed!")
    else:
        print(f"Some tests failed. Please check the implementation.")
