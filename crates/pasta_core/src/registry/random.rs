//! Random selector abstraction for testability.
//!
//! This module provides a trait-based abstraction for random selection,
//! allowing deterministic testing through mock implementations.

use rand::prelude::*;

/// Trait that decides the order of a round (allows mocking in tests).
///
/// SceneTable and WordTable call `shuffle_usize` once at the start of each
/// round, passing an array in candidate order, and then consume the reordered
/// array from its head. The trait has only this method and stays object-safe
/// (`Box<dyn RandomSelector>`).
pub trait RandomSelector: Send + Sync {
    /// Reorder `items` in-place; the result decides the order of the round.
    ///
    /// `items` arrives in candidate order. Implementations must return a
    /// permutation of it (no element added or lost).
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
}

impl Default for DefaultRandomSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomSelector for DefaultRandomSelector {
    fn shuffle_usize(&mut self, items: &mut [usize]) {
        items.shuffle(&mut self.rng);
    }
}

/// Mock selector that decides the order of a round by a specified sequence.
///
/// `shuffle_usize` reads the sequence as positions into the candidate-order
/// array and reorders it accordingly. The sequence is applied from its head
/// on every call; the selector holds only the sequence and keeps no state.
///
/// This selector is always public (not just for tests) to support
/// Lua-side selector control for test scenarios.
pub struct MockRandomSelector {
    sequence: Vec<usize>, // Positions into the candidate-order array
}

impl MockRandomSelector {
    /// Create a mock selector with a predetermined sequence of positions.
    pub fn new(sequence: Vec<usize>) -> Self {
        Self { sequence }
    }
}

impl RandomSelector for MockRandomSelector {
    /// Reorder `items` by the sequence, read as positions into `items`.
    ///
    /// Values at valid positions (in range, first occurrence) come first in
    /// sequence order; the rest follow in their original order. The sequence
    /// is applied from its head on every call and no state is written.
    fn shuffle_usize(&mut self, items: &mut [usize]) {
        let mut used = vec![false; items.len()];
        let mut ordered = Vec::with_capacity(items.len());
        for &n in &self.sequence {
            if n < items.len() && !used[n] {
                used[n] = true;
                ordered.push(items[n]);
            }
        }
        ordered.extend(
            items
                .iter()
                .zip(&used)
                .filter(|(_, u)| !**u)
                .map(|(v, _)| *v),
        );
        items.copy_from_slice(&ordered);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_seed_is_reproducible() {
        // Same seed must produce the same order (documented testing contract)
        let mut a = DefaultRandomSelector::with_seed(42);
        let mut b = DefaultRandomSelector::with_seed(42);
        for _ in 0..5 {
            let mut items_a: Vec<usize> = (0..10).collect();
            let mut items_b = items_a.clone();
            a.shuffle_usize(&mut items_a);
            b.shuffle_usize(&mut items_b);
            assert_eq!(items_a, items_b);
        }
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

    /// Apply a mock sequence to `items` and return the reordered vec.
    fn mock_shuffle(sequence: Vec<usize>, items: &[usize]) -> Vec<usize> {
        let mut selector = MockRandomSelector::new(sequence);
        let mut items = items.to_vec();
        selector.shuffle_usize(&mut items);
        items
    }

    #[test]
    fn test_mock_selector_shuffle_usize_follows_sequence() {
        // items = [a, b, c] = [0, 1, 2]; sequence values are positions
        let abc = [0, 1, 2];
        assert_eq!(mock_shuffle(vec![1], &abc), vec![1, 0, 2]);
        assert_eq!(mock_shuffle(vec![2, 0], &abc), vec![2, 0, 1]);
        // Identity cases
        assert_eq!(mock_shuffle(vec![0], &abc), vec![0, 1, 2]);
        assert_eq!(mock_shuffle(vec![0, 1, 2], &abc), vec![0, 1, 2]);
        assert_eq!(mock_shuffle(vec![], &abc), vec![0, 1, 2]);
        // Out-of-range positions are skipped
        assert_eq!(mock_shuffle(vec![7, 1], &abc), vec![1, 0, 2]);
        // Duplicate positions are skipped
        assert_eq!(mock_shuffle(vec![1, 1, 0], &abc), vec![1, 0, 2]);
        // All ignored -> original order
        assert_eq!(mock_shuffle(vec![5, 9], &abc), vec![0, 1, 2]);
    }

    #[test]
    fn test_mock_selector_shuffle_usize_empty_items() {
        assert_eq!(mock_shuffle(vec![1, 0], &[]), Vec::<usize>::new());
    }

    #[test]
    fn test_mock_selector_shuffle_usize_reads_positions_not_values() {
        assert_eq!(mock_shuffle(vec![2, 0], &[10, 20, 30]), vec![30, 10, 20]);
    }

    #[test]
    fn test_mock_selector_shuffle_usize_restarts_each_call() {
        // Each call applies the sequence from its head, regardless of earlier calls
        let mut selector = MockRandomSelector::new(vec![2, 0]);
        let mut first = vec![10, 20, 30, 40];
        selector.shuffle_usize(&mut first);
        assert_eq!(first, vec![30, 10, 20, 40]);
        let mut second = vec![5, 6, 7];
        selector.shuffle_usize(&mut second);
        assert_eq!(second, vec![7, 5, 6]);
    }
}
