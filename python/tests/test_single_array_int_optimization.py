"""
Test single array optimization with integer keys/values only.
This minimizes Python object overhead to better measure the array layout impact.
"""

import time
import random
import gc
import sys
import os
from array import array

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


class IntArrayLeafNode:
    """Leaf node using Python array module for more efficient int storage."""

    def __init__(self, capacity: int = 128):
        self.capacity = capacity
        self.num_keys = 0
        # Single array: first half keys, second half values
        # Using array module for more efficient int storage
        self.data = array("q", [0] * (capacity * 2))  # 'q' = signed long long
        self.next = None

    def find_position(self, key: int) -> int:
        """Binary search for key position."""
        left, right = 0, self.num_keys
        while left < right:
            mid = (left + right) // 2
            if self.data[mid] < key:
                left = mid + 1
            else:
                right = mid
        return left

    def insert(self, key: int, value: int) -> bool:
        """Insert key-value pair. Returns True if successful."""
        pos = self.find_position(key)

        # Check if key exists
        if pos < self.num_keys and self.data[pos] == key:
            self.data[self.capacity + pos] = value
            return True

        # Check capacity
        if self.num_keys >= self.capacity:
            return False

        # Shift elements using array slicing (more efficient)
        if pos < self.num_keys:
            # Shift keys
            self.data[pos + 1 : self.num_keys + 1] = self.data[pos : self.num_keys]
            # Shift values
            self.data[self.capacity + pos + 1 : self.capacity + self.num_keys + 1] = (
                self.data[self.capacity + pos : self.capacity + self.num_keys]
            )

        # Insert
        self.data[pos] = key
        self.data[self.capacity + pos] = value
        self.num_keys += 1
        return True

    def lookup(self, key: int) -> int:
        """Lookup value for key. Returns -1 if not found."""
        pos = self.find_position(key)
        if pos < self.num_keys and self.data[pos] == key:
            return self.data[self.capacity + pos]
        return -1


class TwoArrayLeafNode:
    """Traditional two-array leaf node for comparison."""

    def __init__(self, capacity: int = 128):
        self.capacity = capacity
        self.keys = array("q")  # Empty array
        self.values = array("q")  # Empty array
        self.next = None

    def find_position(self, key: int) -> int:
        """Binary search for key position."""
        left, right = 0, len(self.keys)
        while left < right:
            mid = (left + right) // 2
            if self.keys[mid] < key:
                left = mid + 1
            else:
                right = mid
        return left

    def insert(self, key: int, value: int) -> bool:
        """Insert key-value pair. Returns True if successful."""
        pos = self.find_position(key)

        # Check if key exists
        if pos < len(self.keys) and self.keys[pos] == key:
            self.values[pos] = value
            return True

        # Check capacity
        if len(self.keys) >= self.capacity:
            return False

        # Insert
        self.keys.insert(pos, key)
        self.values.insert(pos, value)
        return True

    def lookup(self, key: int) -> int:
        """Lookup value for key. Returns -1 if not found."""
        pos = self.find_position(key)
        if pos < len(self.keys) and self.keys[pos] == key:
            return self.values[pos]
        return -1


def benchmark_int_arrays(size: int = 64, iterations: int = 10000):
    """Compare performance of single vs two array layouts."""
    print(f"\nBenchmarking with {size} keys, {iterations} iterations")
    print("-" * 50)

    # Generate test data
    keys = list(range(0, size * 2, 2))  # Even numbers
    random.shuffle(keys)
    lookup_keys = [random.randrange(0, size * 2) for _ in range(100)]

    # Test 1: Sequential insertion
    print("\n1. Sequential Insertion (sorted keys)")

    # Two arrays
    gc.collect()
    start = time.perf_counter()
    for _ in range(iterations):
        node = TwoArrayLeafNode(128)
        for i in range(size):
            node.insert(i, i * 2)
    two_array_seq_time = time.perf_counter() - start

    # Single array
    gc.collect()
    start = time.perf_counter()
    for _ in range(iterations):
        node = IntArrayLeafNode(128)
        for i in range(size):
            node.insert(i, i * 2)
    single_array_seq_time = time.perf_counter() - start

    improvement = (
        (two_array_seq_time - single_array_seq_time) / two_array_seq_time * 100
    )
    print(
        f"Two Arrays:   {two_array_seq_time:.4f}s ({two_array_seq_time/iterations*1e6:.1f} μs/iter)"
    )
    print(
        f"Single Array: {single_array_seq_time:.4f}s ({single_array_seq_time/iterations*1e6:.1f} μs/iter)"
    )
    print(f"Improvement:  {improvement:.1f}%")

    # Test 2: Random insertion
    print("\n2. Random Insertion")

    # Two arrays
    gc.collect()
    start = time.perf_counter()
    for _ in range(iterations):
        node = TwoArrayLeafNode(128)
        for key in keys:
            node.insert(key, key * 2)
    two_array_rand_time = time.perf_counter() - start

    # Single array
    gc.collect()
    start = time.perf_counter()
    for _ in range(iterations):
        node = IntArrayLeafNode(128)
        for key in keys:
            node.insert(key, key * 2)
    single_array_rand_time = time.perf_counter() - start

    improvement = (
        (two_array_rand_time - single_array_rand_time) / two_array_rand_time * 100
    )
    print(
        f"Two Arrays:   {two_array_rand_time:.4f}s ({two_array_rand_time/iterations*1e6:.1f} μs/iter)"
    )
    print(
        f"Single Array: {single_array_rand_time:.4f}s ({single_array_rand_time/iterations*1e6:.1f} μs/iter)"
    )
    print(f"Improvement:  {improvement:.1f}%")

    # Test 3: Lookup performance
    print("\n3. Lookup Performance")

    # Build nodes
    two_array_node = TwoArrayLeafNode(128)
    single_array_node = IntArrayLeafNode(128)
    for key in keys:
        two_array_node.insert(key, key * 2)
        single_array_node.insert(key, key * 2)

    # Two arrays lookup
    gc.collect()
    start = time.perf_counter()
    for _ in range(iterations):
        total = 0
        for key in lookup_keys:
            total += two_array_node.lookup(key)
    two_array_lookup_time = time.perf_counter() - start

    # Single array lookup
    gc.collect()
    start = time.perf_counter()
    for _ in range(iterations):
        total = 0
        for key in lookup_keys:
            total += single_array_node.lookup(key)
    single_array_lookup_time = time.perf_counter() - start

    improvement = (
        (two_array_lookup_time - single_array_lookup_time) / two_array_lookup_time * 100
    )
    print(
        f"Two Arrays:   {two_array_lookup_time:.4f}s ({two_array_lookup_time/iterations*1e6:.1f} μs/iter)"
    )
    print(
        f"Single Array: {single_array_lookup_time:.4f}s ({single_array_lookup_time/iterations*1e6:.1f} μs/iter)"
    )
    print(f"Improvement:  {improvement:.1f}%")

    # Test 4: Sequential scan (cache efficiency)
    print("\n4. Sequential Scan (cache efficiency)")

    # Two arrays scan
    gc.collect()
    start = time.perf_counter()
    for _ in range(iterations):
        total = 0
        for i in range(len(two_array_node.keys)):
            total += two_array_node.keys[i] + two_array_node.values[i]
    two_array_scan_time = time.perf_counter() - start

    # Single array scan
    gc.collect()
    start = time.perf_counter()
    for _ in range(iterations):
        total = 0
        for i in range(single_array_node.num_keys):
            total += (
                single_array_node.data[i]
                + single_array_node.data[single_array_node.capacity + i]
            )
    single_array_scan_time = time.perf_counter() - start

    improvement = (
        (two_array_scan_time - single_array_scan_time) / two_array_scan_time * 100
    )
    print(
        f"Two Arrays:   {two_array_scan_time:.4f}s ({two_array_scan_time/iterations*1e6:.1f} μs/iter)"
    )
    print(
        f"Single Array: {single_array_scan_time:.4f}s ({single_array_scan_time/iterations*1e6:.1f} μs/iter)"
    )
    print(f"Improvement:  {improvement:.1f}%")


def test_single_array_int_optimization():
    """Test integer-only single array optimization."""
    print("Single Array Optimization Test (Integer Keys/Values)")
    print("=" * 60)

    # Test with different node sizes
    for size in [16, 32, 64]:
        benchmark_int_arrays(size, 10000)

    print("\n" + "=" * 60)
    print("Summary: Single array layout impact with integer-only operations")
    print("Note: Real improvement will be more significant in C implementation")


if __name__ == "__main__":
    test_single_array_int_optimization()
