"""
Test for iterator modification safety fix.

This test verifies that the modification counter prevents segfaults by
properly detecting when the tree structure changes during iteration.
"""

import pytest
import sys
import os
import gc

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    import bplustree_c

    HAS_C_EXTENSION = True
except ImportError:
    HAS_C_EXTENSION = False


class TestIteratorModificationSafety:
    """Test that iterators are invalidated when tree is modified."""

    def test_iterator_invalidation_on_insertion(self):
        """Test that iterator is invalidated when items are inserted."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=32)

        # Add initial items
        for i in range(10):
            tree[i] = f"value_{i}"

        # Create iterator
        keys_iter = tree.keys()

        # Get first item
        first_key = next(keys_iter)
        assert first_key == 0

        # Modify tree - this should invalidate the iterator
        tree[100] = "new_value"

        # Next call should raise RuntimeError
        with pytest.raises(RuntimeError, match="tree changed size during iteration"):
            next(keys_iter)

    def test_iterator_invalidation_on_deletion(self):
        """Test that iterator is invalidated when items are deleted."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=32)

        # Add initial items
        for i in range(20):
            tree[i] = f"value_{i}"

        # Create iterator
        keys_iter = tree.keys()

        # Get first item
        first_key = next(keys_iter)
        assert first_key == 0

        # Delete an item - this should invalidate the iterator
        del tree[10]

        # Next call should raise RuntimeError
        with pytest.raises(RuntimeError, match="tree changed size during iteration"):
            next(keys_iter)

    def test_iterator_invalidation_on_update(self):
        """Test that iterator is invalidated when existing items are updated."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=32)

        # Add initial items
        for i in range(10):
            tree[i] = f"value_{i}"

        # Create iterator
        keys_iter = tree.keys()

        # Get first item
        first_key = next(keys_iter)
        assert first_key == 0

        # Update existing item - this should invalidate the iterator
        tree[5] = "updated_value"

        # Next call should raise RuntimeError
        with pytest.raises(RuntimeError, match="tree changed size during iteration"):
            next(keys_iter)

    def test_items_iterator_invalidation(self):
        """Test that items() iterator is also invalidated."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=32)

        # Add initial items
        for i in range(10):
            tree[i] = f"value_{i}"

        # Create items iterator
        items_iter = tree.items()

        # Get first item
        first_item = next(items_iter)
        assert first_item == (0, "value_0")

        # Modify tree - this should invalidate the iterator
        tree[100] = "new_value"

        # Next call should raise RuntimeError
        with pytest.raises(RuntimeError, match="tree changed size during iteration"):
            next(items_iter)

    def test_multiple_iterators_invalidation(self):
        """Test that all iterators are invalidated when tree is modified."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=32)

        # Add initial items
        for i in range(10):
            tree[i] = f"value_{i}"

        # Create multiple iterators
        keys_iter1 = tree.keys()
        keys_iter2 = tree.keys()
        items_iter = tree.items()

        # Get first item from each
        assert next(keys_iter1) == 0
        assert next(keys_iter2) == 0
        assert next(items_iter) == (0, "value_0")

        # Modify tree - this should invalidate all iterators
        tree[100] = "new_value"

        # All iterators should now raise RuntimeError
        with pytest.raises(RuntimeError, match="tree changed size during iteration"):
            next(keys_iter1)

        with pytest.raises(RuntimeError, match="tree changed size during iteration"):
            next(keys_iter2)

        with pytest.raises(RuntimeError, match="tree changed size during iteration"):
            next(items_iter)

    def test_iterator_after_tree_modification(self):
        """Test that new iterators work after tree modification."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=32)

        # Add initial items
        for i in range(10):
            tree[i] = f"value_{i}"

        # Create iterator
        old_iter = tree.keys()
        next(old_iter)  # Get first item

        # Modify tree
        tree[100] = "new_value"

        # Old iterator should be invalidated
        with pytest.raises(RuntimeError, match="tree changed size during iteration"):
            next(old_iter)

        # New iterator should work fine
        new_iter = tree.keys()
        keys = list(new_iter)
        assert len(keys) == 11
        assert 0 in keys
        assert 100 in keys

    def test_list_keys_after_heavy_modification(self):
        """Test that list(tree.keys()) works after heavy modification."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=32)

        # Heavy modification pattern that used to cause segfaults
        for round in range(3):
            # Insert batch
            for i in range(round * 100, (round + 1) * 100):
                tree[i] = f"round_{round}_value_{i}"

            # Delete some from previous rounds
            if round > 0:
                for i in range((round - 1) * 100, (round - 1) * 100 + 50):
                    if i in tree:
                        del tree[i]

            # Force garbage collection
            gc.collect()

        # This should not segfault
        keys = list(tree.keys())
        assert len(keys) > 0

        # All keys should be accessible
        for key in keys[:10]:  # Test first 10 keys
            value = tree[key]
            assert value is not None

    def test_iteration_with_structural_changes(self):
        """Test iteration behavior when tree structure changes significantly."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=32)

        # Create a tree that will undergo structural changes
        for i in range(100):
            tree[i] = f"value_{i}"

        # Create iterator
        keys_iter = tree.keys()
        first_key = next(keys_iter)
        assert first_key == 0

        # Cause major structural changes by deleting many items
        # This should trigger node merging and rebalancing
        for i in range(50, 100):
            del tree[i]

        # Iterator should be invalidated
        with pytest.raises(RuntimeError, match="tree changed size during iteration"):
            next(keys_iter)

    def test_concurrent_modification_detection(self):
        """Test detection of concurrent modifications during iteration."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=32)

        # Setup tree
        for i in range(50):
            tree[i] = f"value_{i}"

        # Start iteration
        keys_iter = tree.keys()
        collected_keys = []

        # Collect some keys
        for _ in range(5):
            collected_keys.append(next(keys_iter))

        # Modify the tree
        tree[1000] = "new_value"

        # Further iteration should fail
        with pytest.raises(RuntimeError, match="tree changed size during iteration"):
            next(keys_iter)

        # Verify we got the expected keys before modification
        assert collected_keys == [0, 1, 2, 3, 4]

    def test_no_false_positives(self):
        """Test that iterators don't get falsely invalidated."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=32)

        # Add items
        for i in range(10):
            tree[i] = f"value_{i}"

        # Create iterator
        keys_iter = tree.keys()

        # Iterate through all items without modifying tree
        keys = []
        for key in keys_iter:
            keys.append(key)

        # Should get all keys without error
        assert keys == list(range(10))

    def test_modification_counter_wrapping(self):
        """Test that modification counter handles large numbers of modifications."""
        if not HAS_C_EXTENSION:
            pytest.skip("C extension not available")

        tree = bplustree_c.BPlusTree(capacity=32)

        # Make many modifications to test counter behavior
        for i in range(1000):
            tree[i] = f"value_{i}"
            if i % 100 == 0:
                # Create and invalidate iterator periodically
                keys_iter = tree.keys()
                next(keys_iter)
                tree[i + 10000] = "trigger_invalidation"

                with pytest.raises(
                    RuntimeError, match="tree changed size during iteration"
                ):
                    next(keys_iter)

        # Final iteration should work
        keys = list(tree.keys())
        assert len(keys) > 1000


if __name__ == "__main__":
    # Run the tests
    test = TestIteratorModificationSafety()
    test.test_iterator_invalidation_on_insertion()
    test.test_iterator_invalidation_on_deletion()
    test.test_iterator_invalidation_on_update()
    test.test_items_iterator_invalidation()
    test.test_multiple_iterators_invalidation()
    test.test_iterator_after_tree_modification()
    try:
        test.test_list_keys_after_heavy_modification()
        test.test_iteration_with_structural_changes()
        test.test_concurrent_modification_detection()
        test.test_no_false_positives()
        test.test_modification_counter_wrapping()
        print("✅ All iterator modification safety tests passed")
    except Exception as e:
        print(f"❌ Test failed: {e}")
        import traceback

        traceback.print_exc()
