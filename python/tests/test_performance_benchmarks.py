"""
Performance benchmark tests for B+ Tree implementation.

These tests verify that performance meets expected thresholds and
can be used for regression detection in CI/CD.
"""

import pytest
import time
import sys
import os
from typing import List, Tuple

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from bplustree import BPlusTreeMap


@pytest.mark.slow
class TestPerformanceBenchmarks:
    """Performance benchmark tests with threshold validation."""

    def test_insertion_performance_small(self):
        """Test insertion performance for small datasets."""
        size = 1000
        tree = BPlusTreeMap(capacity=32)

        start_time = time.perf_counter()
        for i in range(size):
            tree[i] = f"value_{i}"
        elapsed = time.perf_counter() - start_time

        # Should complete in reasonable time (< 0.1 seconds)
        assert elapsed < 0.1, f"Small insertion took {elapsed:.3f}s, expected < 0.1s"

        # Verify all items inserted correctly
        assert len(tree) == size
        assert tree[0] == "value_0"
        assert tree[size - 1] == f"value_{size - 1}"

    def test_insertion_performance_medium(self):
        """Test insertion performance for medium datasets."""
        size = 10000
        tree = BPlusTreeMap(capacity=32)

        start_time = time.perf_counter()
        for i in range(size):
            tree[i] = f"value_{i}"
        elapsed = time.perf_counter() - start_time

        # Should complete in reasonable time (< 1 second)
        assert elapsed < 1.0, f"Medium insertion took {elapsed:.3f}s, expected < 1.0s"

        # Verify correctness
        assert len(tree) == size

        # Check performance metrics
        ops_per_second = size / elapsed
        assert (
            ops_per_second > 5000
        ), f"Insertion rate {ops_per_second:.0f} ops/s, expected > 5000"

    def test_bulk_loading_performance(self):
        """Test bulk loading performance advantage."""
        size = 10000
        data = [(i, f"value_{i}") for i in range(size)]

        # Test bulk loading
        start_time = time.perf_counter()
        tree_bulk = BPlusTreeMap.from_sorted_items(data, capacity=32)
        bulk_time = time.perf_counter() - start_time

        # Test individual insertion
        start_time = time.perf_counter()
        tree_individual = BPlusTreeMap(capacity=32)
        for k, v in data:
            tree_individual[k] = v
        individual_time = time.perf_counter() - start_time

        # Bulk loading should be faster
        speedup = individual_time / bulk_time
        assert speedup > 1.5, f"Bulk loading speedup {speedup:.1f}x, expected > 1.5x"

        # Verify both trees have same content
        assert len(tree_bulk) == len(tree_individual) == size
        for i in range(size):
            assert tree_bulk[i] == tree_individual[i]

    def test_lookup_performance(self):
        """Test lookup performance."""
        size = 10000
        tree = BPlusTreeMap(capacity=32)

        # Populate tree
        for i in range(size):
            tree[i] = f"value_{i}"

        # Test lookup performance
        lookup_count = 10000
        lookup_keys = list(range(0, size, size // lookup_count)) * (
            lookup_count // (size // (size // lookup_count)) + 1
        )
        lookup_keys = lookup_keys[:lookup_count]

        start_time = time.perf_counter()
        for key in lookup_keys:
            _ = tree[key]
        elapsed = time.perf_counter() - start_time

        # Should complete lookups quickly
        assert elapsed < 0.5, f"Lookups took {elapsed:.3f}s, expected < 0.5s"

        # Check lookup rate
        lookups_per_second = lookup_count / elapsed
        assert (
            lookups_per_second > 20000
        ), f"Lookup rate {lookups_per_second:.0f} ops/s, expected > 20000"

    def test_range_query_performance(self):
        """Test range query performance."""
        size = 10000
        tree = BPlusTreeMap(capacity=64)  # Larger capacity for range queries

        # Populate tree
        for i in range(size):
            tree[i] = f"value_{i}"

        # Test range queries of different sizes
        range_sizes = [10, 100, 1000]

        for range_size in range_sizes:
            start_key = size // 2 - range_size // 2
            end_key = start_key + range_size

            start_time = time.perf_counter()
            results = list(tree.range(start_key, end_key))
            elapsed = time.perf_counter() - start_time

            # Verify results
            assert len(results) == range_size

            # Performance threshold depends on range size
            max_time = range_size * 0.001  # 1ms per 1000 items
            assert (
                elapsed < max_time
            ), f"Range query ({range_size} items) took {elapsed:.3f}s, expected < {max_time:.3f}s"

    def test_mixed_workload_performance(self):
        """Test performance with mixed operations."""
        tree = BPlusTreeMap(capacity=32)

        # Initial data
        initial_size = 5000
        for i in range(initial_size):
            tree[i] = f"value_{i}"

        # Mixed workload: 60% lookups, 30% inserts, 10% deletes
        operations = 10000
        lookup_ops = int(operations * 0.6)
        insert_ops = int(operations * 0.3)
        delete_ops = int(operations * 0.1)

        start_time = time.perf_counter()

        # Perform mixed operations
        import random

        # Lookups
        for _ in range(lookup_ops):
            key = random.randint(0, initial_size - 1)
            _ = tree.get(key)

        # Inserts
        for i in range(insert_ops):
            key = initial_size + i
            tree[key] = f"new_value_{key}"

        # Deletes
        for _ in range(delete_ops):
            key = random.randint(0, initial_size - 1)
            try:
                del tree[key]
            except KeyError:
                pass

        elapsed = time.perf_counter() - start_time

        # Should handle mixed workload efficiently
        assert elapsed < 2.0, f"Mixed workload took {elapsed:.3f}s, expected < 2.0s"

        # Check operation rate
        ops_per_second = operations / elapsed
        assert (
            ops_per_second > 5000
        ), f"Mixed workload rate {ops_per_second:.0f} ops/s, expected > 5000"

    def test_capacity_impact_on_performance(self):
        """Test how node capacity affects performance."""
        size = 5000
        capacities = [8, 32, 128]
        insertion_times = {}

        for capacity in capacities:
            tree = BPlusTreeMap(capacity=capacity)

            start_time = time.perf_counter()
            for i in range(size):
                tree[i] = f"value_{i}"
            elapsed = time.perf_counter() - start_time

            insertion_times[capacity] = elapsed

            # Verify correctness
            assert len(tree) == size

        # Higher capacity should generally be faster for this size
        # (fewer node splits and levels)
        assert insertion_times[32] <= insertion_times[8] * 1.5
        assert insertion_times[128] <= insertion_times[32] * 1.2

    def test_memory_efficiency(self):
        """Test memory usage efficiency."""
        try:
            import tracemalloc
        except ImportError:
            pytest.skip("tracemalloc not available")

        size = 10000

        tracemalloc.start()

        tree = BPlusTreeMap(capacity=32)
        for i in range(size):
            tree[i] = f"value_{i}"

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()

        # Memory usage should be reasonable
        memory_per_item = peak / size
        assert (
            memory_per_item < 1000
        ), f"Memory per item {memory_per_item:.0f} bytes, expected < 1000"

        total_mb = peak / 1024 / 1024
        assert total_mb < 50, f"Total memory {total_mb:.1f} MB, expected < 50 MB"

    def test_sequential_vs_random_insertion(self):
        """Test performance difference between sequential and random insertion."""
        size = 5000

        # Sequential insertion
        tree_seq = BPlusTreeMap(capacity=32)
        start_time = time.perf_counter()
        for i in range(size):
            tree_seq[i] = f"value_{i}"
        sequential_time = time.perf_counter() - start_time

        # Random insertion
        import random

        keys = list(range(size))
        random.shuffle(keys)

        tree_rand = BPlusTreeMap(capacity=32)
        start_time = time.perf_counter()
        for k in keys:
            tree_rand[k] = f"value_{k}"
        random_time = time.perf_counter() - start_time

        # Both should complete in reasonable time
        assert (
            sequential_time < 1.0
        ), f"Sequential insertion took {sequential_time:.3f}s"
        assert random_time < 2.0, f"Random insertion took {random_time:.3f}s"

        # Sequential should be faster
        speedup = random_time / sequential_time
        assert speedup > 1.2, f"Sequential speedup {speedup:.1f}x, expected > 1.2x"

        # Both trees should have same content
        assert len(tree_seq) == len(tree_rand) == size
        for i in range(size):
            assert tree_seq[i] == tree_rand[i]

    def test_large_dataset_scalability(self):
        """Test scalability with larger datasets."""
        # Test with progressively larger datasets
        sizes = [1000, 5000, 10000]
        times = []

        for size in sizes:
            tree = BPlusTreeMap(capacity=64)

            start_time = time.perf_counter()
            for i in range(size):
                tree[i] = f"value_{i}"
            elapsed = time.perf_counter() - start_time

            times.append(elapsed)

            # Each size should complete in reasonable time
            max_time = size / 5000  # Should handle at least 5000 ops/sec
            assert (
                elapsed < max_time
            ), f"Size {size} took {elapsed:.3f}s, expected < {max_time:.3f}s"

        # Check that time complexity is reasonable (should be roughly O(n log n))
        # The ratio of times should grow slower than the ratio of sizes
        time_ratio_1_2 = times[1] / times[0]
        size_ratio_1_2 = sizes[1] / sizes[0]

        time_ratio_2_3 = times[2] / times[1]
        size_ratio_2_3 = sizes[2] / sizes[1]

        # Time should grow slower than linear with size
        assert time_ratio_1_2 < size_ratio_1_2 * 1.5
        assert time_ratio_2_3 < size_ratio_2_3 * 1.5

    @pytest.mark.slow
    def test_stress_performance(self):
        """Stress test with intensive operations."""
        tree = BPlusTreeMap(capacity=64)

        # Phase 1: Large insertion
        size = 50000
        start_time = time.perf_counter()
        for i in range(size):
            tree[i] = f"value_{i}"
        insertion_time = time.perf_counter() - start_time

        assert (
            insertion_time < 10.0
        ), f"Large insertion took {insertion_time:.3f}s, expected < 10s"

        # Phase 2: Many lookups
        lookup_count = 100000
        start_time = time.perf_counter()
        import random

        for _ in range(lookup_count):
            key = random.randint(0, size - 1)
            _ = tree[key]
        lookup_time = time.perf_counter() - start_time

        assert lookup_time < 5.0, f"Many lookups took {lookup_time:.3f}s, expected < 5s"

        # Phase 3: Range queries
        start_time = time.perf_counter()
        for i in range(0, size, 1000):
            list(tree.range(i, i + 100))
        range_time = time.perf_counter() - start_time

        assert range_time < 3.0, f"Range queries took {range_time:.3f}s, expected < 3s"

        print(f"Stress test completed:")
        print(f"  Insertion: {insertion_time:.3f}s ({size/insertion_time:.0f} ops/s)")
        print(f"  Lookups: {lookup_time:.3f}s ({lookup_count/lookup_time:.0f} ops/s)")
        print(f"  Ranges: {range_time:.3f}s")


class TestPerformanceRegression:
    """Tests to detect performance regressions."""

    def test_baseline_insertion_performance(self):
        """Baseline test for insertion performance regression detection."""
        size = 10000
        tree = BPlusTreeMap(capacity=32)

        start_time = time.perf_counter()
        for i in range(size):
            tree[i] = f"value_{i}"
        elapsed = time.perf_counter() - start_time

        # Conservative threshold to catch major regressions
        max_time = 2.0  # Should be much faster, but allows for slow CI environments
        assert (
            elapsed < max_time
        ), f"Insertion baseline exceeded: {elapsed:.3f}s > {max_time}s"

        # Store result for comparison (in real CI, this would be persisted)
        ops_per_second = size / elapsed
        assert (
            ops_per_second > 2000
        ), f"Insertion rate too low: {ops_per_second:.0f} ops/s"

    def test_baseline_lookup_performance(self):
        """Baseline test for lookup performance regression detection."""
        size = 10000
        tree = BPlusTreeMap(capacity=32)

        # Populate tree
        for i in range(size):
            tree[i] = f"value_{i}"

        # Test lookups
        lookup_count = 10000
        start_time = time.perf_counter()
        for i in range(lookup_count):
            _ = tree[i % size]
        elapsed = time.perf_counter() - start_time

        # Conservative threshold
        max_time = 1.0
        assert (
            elapsed < max_time
        ), f"Lookup baseline exceeded: {elapsed:.3f}s > {max_time}s"

        ops_per_second = lookup_count / elapsed
        assert ops_per_second > 5000, f"Lookup rate too low: {ops_per_second:.0f} ops/s"

    def test_memory_usage_baseline(self):
        """Baseline test for memory usage regression detection."""
        try:
            import tracemalloc
        except ImportError:
            pytest.skip("tracemalloc not available")

        tracemalloc.start()

        size = 10000
        tree = BPlusTreeMap(capacity=32)
        for i in range(size):
            tree[i] = f"value_{i}"

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()

        # Conservative memory threshold
        max_memory_mb = 100  # Should be much less, but allows for overhead
        memory_mb = peak / 1024 / 1024
        assert (
            memory_mb < max_memory_mb
        ), f"Memory usage baseline exceeded: {memory_mb:.1f} MB > {max_memory_mb} MB"


if __name__ == "__main__":
    # Run performance tests
    pytest.main([__file__, "-v", "-x"])  # Stop on first failure
