use bplustree::BPlusTreeMap;
use std::collections::BTreeMap;

fn populate_maps(capacity: usize, data: &[i32]) -> (BPlusTreeMap<i32, i32>, BTreeMap<i32, i32>) {
    let mut tree = BPlusTreeMap::new(capacity).unwrap();
    let mut map = BTreeMap::new();
    for &k in data {
        tree.insert(k, k * 10);
        map.insert(k, k * 10);
    }
    (tree, map)
}

#[test]
fn test_range_differential_basic_boundaries() {
    // Use small capacities to force multiple leaves and boundary transitions
    for &cap in &[4_usize, 5, 8] {
        let data: Vec<i32> = (0..20).collect();
        let (tree, map) = populate_maps(cap, &data);

        // Helper to compare results for a range expression
        let assert_same = |lhs: Vec<(i32, i32)>, rhs: Vec<(i32, i32)>, label: &str| {
            assert_eq!(lhs, rhs, "mismatch for range: {} (cap={})", label, cap);
        };

        // Closed-open typical range
        let got: Vec<_> = tree.range(3..7).map(|(k, v)| (*k, *v)).collect();
        let exp: Vec<_> = map.range(3..7).map(|(k, v)| (*k, *v)).collect();
        assert_same(got, exp, "3..7");

        // Closed-closed
        let got: Vec<_> = tree.range(3..=7).map(|(k, v)| (*k, *v)).collect();
        let exp: Vec<_> = map.range(3..=7).map(|(k, v)| (*k, *v)).collect();
        assert_same(got, exp, "3..=7");

        // Open-ended start
        let got: Vec<_> = tree.range(..5).map(|(k, v)| (*k, *v)).collect();
        let exp: Vec<_> = map.range(..5).map(|(k, v)| (*k, *v)).collect();
        assert_same(got, exp, "..5");

        // Open-ended end
        let got: Vec<_> = tree.range(5..).map(|(k, v)| (*k, *v)).collect();
        let exp: Vec<_> = map.range(5..).map(|(k, v)| (*k, *v)).collect();
        assert_same(got, exp, "5..");

        // Full range
        let got: Vec<_> = tree.range(..).map(|(k, v)| (*k, *v)).collect();
        let exp: Vec<_> = map.range(..).map(|(k, v)| (*k, *v)).collect();
        assert_same(got, exp, "..");

        // Singleton ranges
        let got: Vec<_> = tree.range(4..=4).map(|(k, v)| (*k, *v)).collect();
        let exp: Vec<_> = map.range(4..=4).map(|(k, v)| (*k, *v)).collect();
        assert_same(got, exp, "4..=4");

        // Empty by construction
        let got: Vec<_> = tree.range(4..4).map(|(k, v)| (*k, *v)).collect();
        let exp: Vec<_> = map.range(4..4).map(|(k, v)| (*k, *v)).collect();
        assert_same(got, exp, "4..4 (empty)");
    }
}

#[test]
fn test_range_differential_gaps_and_nonexistent_bounds() {
    // Data with gaps to test non-existing bound keys and cross-leaf traversal
    for &cap in &[4_usize, 5, 8] {
        let data = vec![0, 1, 2, 4, 7, 8, 10, 13, 14, 18];
        let (tree, map) = populate_maps(cap, &data);

        let assert_same = |lhs: Vec<(i32, i32)>, rhs: Vec<(i32, i32)>, label: &str| {
            assert_eq!(lhs, rhs, "mismatch for range: {} (cap={})", label, cap);
        };

        // Start/end on non-existent keys (between 2 and 4; between 8 and 10)
        let got: Vec<_> = tree.range(3..9).map(|(k, v)| (*k, *v)).collect();
        let exp: Vec<_> = map.range(3..9).map(|(k, v)| (*k, *v)).collect();
        assert_same(got, exp, "3..9");

        // Inclusive upper bound non-existent
        let got: Vec<_> = tree.range(3..=9).map(|(k, v)| (*k, *v)).collect();
        let exp: Vec<_> = map.range(3..=9).map(|(k, v)| (*k, *v)).collect();
        assert_same(got, exp, "3..=9");

        // Exclusive lower bound non-existent
        let got: Vec<_> = tree.range(3..=4).map(|(k, v)| (*k, *v)).collect();
        let exp: Vec<_> = map.range(3..=4).map(|(k, v)| (*k, *v)).collect();
        assert_same(got, exp, "3..=4");

        // Entirely out-of-range
        let got: Vec<_> = tree.range(100..200).map(|(k, v)| (*k, *v)).collect();
        let exp: Vec<_> = map.range(100..200).map(|(k, v)| (*k, *v)).collect();
        assert_same(got, exp, "100..200 (empty)");

        // Negative lower bound below min
        let got: Vec<_> = tree.range(-5..3).map(|(k, v)| (*k, *v)).collect();
        let exp: Vec<_> = map.range(-5..3).map(|(k, v)| (*k, *v)).collect();
        assert_same(got, exp, "-5..3");

        // Intentionally avoid inverted ranges: std::BTreeMap panics for start > end
    }
}
