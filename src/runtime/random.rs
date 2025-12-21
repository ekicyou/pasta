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
#[cfg(test)]
pub struct MockRandomSelector {
    sequence: Vec<usize>, // Sequence of indices to select
    index: usize,         // Current position in sequence
}

#[cfg(test)]
impl MockRandomSelector {
    /// Create a mock selector with a predetermined sequence of indices.
    pub fn new(sequence: Vec<usize>) -> Self {
        Self { sequence, index: 0 }
    }
}

#[cfg(test)]
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
}
