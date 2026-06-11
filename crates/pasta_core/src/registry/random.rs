//! Random selector abstraction for testability.
//!
//! This module provides a trait-based abstraction for random selection,
//! allowing deterministic testing through mock implementations.

use rand::prelude::*;

/// Trait for random selection (allows mocking in tests).
///
/// Note: Due to Rust's object safety rules, we use Any type for items.
/// This is a workaround to make the trait dyn-safe while still allowing
/// generic selection.
pub trait RandomSelector: Send + Sync {
    /// Select a random index from 0..len.
    fn select_index(&mut self, len: usize) -> Option<usize>;

    /// Shuffle a vec of usize in-place (for scene IDs).
    fn shuffle_usize(&mut self, items: &mut [usize]);
}

/// Default random selector using system entropy.
pub struct DefaultRandomSelector {
    rng: StdRng,
}

impl DefaultRandomSelector {
    /// Create a new random selector with system entropy.
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_seed(rand::rng().random()),
        }
    }

    /// Create a new random selector with a fixed seed (for reproducible testing).
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Select a random element from a slice (convenience method).
    pub fn select<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        items.choose(&mut self.rng)
    }

    /// Shuffle a slice in-place (convenience method).
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        items.shuffle(&mut self.rng);
    }
}

impl Default for DefaultRandomSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomSelector for DefaultRandomSelector {
    fn select_index(&mut self, len: usize) -> Option<usize> {
        if len == 0 {
            None
        } else {
            Some(self.rng.random_range(0..len))
        }
    }

    fn shuffle_usize(&mut self, items: &mut [usize]) {
        items.shuffle(&mut self.rng);
    }
}

/// Mock random selector for deterministic testing.
///
/// This selector is always public (not just for tests) to support
/// Lua-side selector control for test scenarios.
pub struct MockRandomSelector {
    sequence: Vec<usize>, // Sequence of indices to select
    index: usize,         // Current position in sequence
}

impl MockRandomSelector {
    /// Create a mock selector with a predetermined sequence of indices.
    pub fn new(sequence: Vec<usize>) -> Self {
        Self { sequence, index: 0 }
    }
}

impl RandomSelector for MockRandomSelector {
    fn select_index(&mut self, len: usize) -> Option<usize> {
        if len == 0 || self.sequence.is_empty() {
            return None;
        }
        let idx = self.sequence[self.index % self.sequence.len()] % len;
        self.index += 1;
        Some(idx)
    }

    fn shuffle_usize(&mut self, _items: &mut [usize]) {
        // Mock implementation does not shuffle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_selector() {
        let mut selector = DefaultRandomSelector::with_seed(42);
        let items = vec![1, 2, 3, 4, 5];

        // Should select something
        let result = selector.select(&items);
        assert!(result.is_some());
        assert!(items.contains(result.unwrap()));
    }

    #[test]
    fn test_default_selector_empty() {
        let mut selector = DefaultRandomSelector::new();
        let items: Vec<i32> = vec![];

        let result = selector.select(&items);
        assert!(result.is_none());
    }

    #[test]
    fn test_mock_selector() {
        let mut selector = MockRandomSelector::new(vec![0, 2, 1]);

        assert_eq!(selector.select_index(3), Some(0)); // index 0
        assert_eq!(selector.select_index(3), Some(2)); // index 2
        assert_eq!(selector.select_index(3), Some(1)); // index 1
        assert_eq!(selector.select_index(3), Some(0)); // wraps around to index 0
    }

    #[test]
    fn test_mock_selector_empty() {
        let mut selector = MockRandomSelector::new(vec![0]);

        assert_eq!(selector.select_index(0), None);
    }

    #[test]
    fn test_shuffle() {
        let mut selector = DefaultRandomSelector::with_seed(42);
        let mut items = vec![1, 2, 3, 4, 5];
        let original = items.clone();

        selector.shuffle(&mut items);

        // After shuffling, items should still contain all elements
        items.sort();
        assert_eq!(items, original);
    }

    #[test]
    fn test_default_selector_select_index_zero_len() {
        // RandomSelector trait: len == 0 must yield None
        let mut selector = DefaultRandomSelector::with_seed(1);
        assert_eq!(selector.select_index(0), None);
    }

    #[test]
    fn test_default_selector_select_index_in_range() {
        // RandomSelector trait: returned index must always be < len
        let mut selector = DefaultRandomSelector::with_seed(7);
        for _ in 0..100 {
            let idx = selector.select_index(5).expect("len > 0 must yield Some");
            assert!(idx < 5, "index {} out of range", idx);
        }
        // len == 1 must always return 0
        assert_eq!(selector.select_index(1), Some(0));
    }

    #[test]
    fn test_with_seed_is_reproducible() {
        // Same seed must produce the same selection sequence (documented testing contract)
        let mut a = DefaultRandomSelector::with_seed(42);
        let mut b = DefaultRandomSelector::with_seed(42);

        let seq_a: Vec<_> = (0..20).map(|_| a.select_index(10)).collect();
        let seq_b: Vec<_> = (0..20).map(|_| b.select_index(10)).collect();
        assert_eq!(seq_a, seq_b);

        let mut items_a = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut items_b = items_a.clone();
        a.shuffle(&mut items_a);
        b.shuffle(&mut items_b);
        assert_eq!(items_a, items_b);
    }

    #[test]
    fn test_default_selector_shuffle_usize_preserves_elements() {
        // RandomSelector trait shuffle_usize: permutation only, no element loss
        let mut selector = DefaultRandomSelector::with_seed(42);
        let mut items: Vec<usize> = (0..10).collect();
        selector.shuffle_usize(&mut items);
        let mut sorted = items.clone();
        sorted.sort();
        assert_eq!(sorted, (0..10).collect::<Vec<usize>>());
    }

    #[test]
    fn test_mock_selector_empty_sequence_returns_none() {
        // Empty sequence must yield None even when len > 0
        let mut selector = MockRandomSelector::new(vec![]);
        assert_eq!(selector.select_index(5), None);
    }

    #[test]
    fn test_mock_selector_index_wraps_by_len() {
        // Sequence values larger than len are reduced modulo len
        let mut selector = MockRandomSelector::new(vec![7]);
        assert_eq!(selector.select_index(3), Some(1)); // 7 % 3 == 1
    }

    #[test]
    fn test_mock_selector_shuffle_usize_is_noop() {
        // Mock shuffle must not reorder (deterministic testing contract)
        let mut selector = MockRandomSelector::new(vec![0]);
        let mut items = vec![3, 1, 4, 1, 5];
        let original = items.clone();
        selector.shuffle_usize(&mut items);
        assert_eq!(items, original);
    }
}
