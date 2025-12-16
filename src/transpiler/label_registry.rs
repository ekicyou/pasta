//! Label registry for tracking labels and assigning unique IDs during transpilation.
//!
//! This module implements the two-pass transpiler strategy:
//! - Pass 1: Collect all labels from PastaFile(s) and assign unique IDs
//! - Pass 2: Generate `mod pasta {}` with ID→function path mapping

use std::collections::HashMap;

/// Information about a registered label.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelInfo {
    /// Unique numeric ID (starting from 1).
    pub id: i64,

    /// Original label name (without counter suffix).
    pub name: String,

    /// Attributes associated with this label (for future P1 filtering).
    pub attributes: HashMap<String, String>,

    /// Full Rune function path (e.g., "crate::会話_1::__start__").
    pub fn_path: String,

    /// Module/function name without "crate::" prefix (e.g., "会話_1::__start__").
    pub fn_name: String,

    /// Parent label name (for local labels only, None for global labels).
    pub parent: Option<String>,
}

/// Label registry for managing label collection and ID assignment.
///
/// # Design Notes
///
/// - **P0 Implementation**: No duplicate label names, all labels get `_1` suffix
/// - **P1 Implementation**: Handle duplicate names with sequential counters (`_1`, `_2`, ...)
/// - IDs start from 1 and increment sequentially
/// - Each label gets a unique ID even if names are the same
pub struct LabelRegistry {
    /// All registered labels, indexed by ID.
    labels: HashMap<i64, LabelInfo>,

    /// Counter for assigning the next unique ID.
    next_id: i64,

    /// Counter for tracking duplicate label names (name → counter).
    /// P0: Always returns 1 (no duplicates expected).
    /// P1: Increments for each duplicate.
    name_counters: HashMap<String, usize>,
}

impl LabelRegistry {
    /// Create a new label registry.
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
            next_id: 1,
            name_counters: HashMap::new(),
        }
    }

    /// Register a global label.
    ///
    /// # Arguments
    ///
    /// * `name` - Original label name (without scope prefix)
    /// * `attributes` - Attributes for filtering (P1 feature)
    ///
    /// # Returns
    ///
    /// The assigned ID and counter for this label.
    pub fn register_global(
        &mut self,
        name: &str,
        attributes: HashMap<String, String>,
    ) -> (i64, usize) {
        let counter = self.increment_counter(name);
        let id = self.next_id;
        self.next_id += 1;

        let fn_name = format!("{}_{}::__start__", Self::sanitize_name(name), counter);
        let fn_path = format!("crate::{}", fn_name);

        let info = LabelInfo {
            id,
            name: name.to_string(),
            attributes,
            fn_path,
            fn_name,
            parent: None,
        };

        self.labels.insert(id, info);
        (id, counter)
    }

    /// Register a local label.
    ///
    /// # Arguments
    ///
    /// * `name` - Original label name (without scope prefix)
    /// * `parent_name` - Parent global label name
    /// * `parent_counter` - Parent's counter value
    /// * `attributes` - Attributes for filtering (P1 feature)
    ///
    /// # Returns
    ///
    /// The assigned ID and counter for this label.
    pub fn register_local(
        &mut self,
        name: &str,
        parent_name: &str,
        parent_counter: usize,
        attributes: HashMap<String, String>,
    ) -> (i64, usize) {
        // For local labels, the full name includes parent for uniqueness
        let full_name = format!("{}::{}", parent_name, name);
        let counter = self.increment_counter(&full_name);
        let id = self.next_id;
        self.next_id += 1;

        // Local label function path: parent module + local function
        // Format: crate::親_番号::子_番号
        let fn_name = format!(
            "{}_{}::{}_{}",
            Self::sanitize_name(parent_name),
            parent_counter,
            Self::sanitize_name(name),
            counter
        );
        let fn_path = format!("crate::{}", fn_name);

        let info = LabelInfo {
            id,
            name: name.to_string(),
            attributes,
            fn_path,
            fn_name,
            parent: Some(parent_name.to_string()),
        };

        self.labels.insert(id, info);
        (id, counter)
    }

    /// Get all registered labels.
    pub fn all_labels(&self) -> Vec<&LabelInfo> {
        let mut labels: Vec<_> = self.labels.values().collect();
        labels.sort_by_key(|l| l.id);
        labels
    }

    /// Get a label by ID.
    pub fn get_label(&self, id: i64) -> Option<&LabelInfo> {
        self.labels.get(&id)
    }

    /// Iterate over all registered labels.
    pub fn iter(&self) -> impl Iterator<Item = (&i64, &LabelInfo)> {
        self.labels.iter()
    }

    /// Increment the counter for a label name and return the new value.
    fn increment_counter(&mut self, name: &str) -> usize {
        let counter = self.name_counters.entry(name.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Sanitize a label name for use in Rune identifiers.
    fn sanitize_name(name: &str) -> String {
        name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
    }
}

impl Default for LabelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_global_label() {
        let mut registry = LabelRegistry::new();

        let (id1, counter1) = registry.register_global("会話", HashMap::new());
        assert_eq!(id1, 1);
        assert_eq!(counter1, 1);

        let label = registry.get_label(id1).unwrap();
        assert_eq!(label.name, "会話");
        assert_eq!(label.fn_path, "crate::会話_1::__start__");
        assert_eq!(label.parent, None);
    }

    #[test]
    fn test_register_multiple_global_labels() {
        let mut registry = LabelRegistry::new();

        let (id1, _) = registry.register_global("会話", HashMap::new());
        let (id2, _) = registry.register_global("別会話", HashMap::new());

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        let labels = registry.all_labels();
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].name, "会話");
        assert_eq!(labels[1].name, "別会話");
    }

    #[test]
    fn test_register_duplicate_global_labels() {
        let mut registry = LabelRegistry::new();

        let (id1, counter1) = registry.register_global("会話", HashMap::new());
        let (id2, counter2) = registry.register_global("会話", HashMap::new());

        assert_eq!(id1, 1);
        assert_eq!(counter1, 1);
        assert_eq!(id2, 2);
        assert_eq!(counter2, 2);

        let label1 = registry.get_label(id1).unwrap();
        let label2 = registry.get_label(id2).unwrap();

        assert_eq!(label1.fn_path, "crate::会話_1::__start__");
        assert_eq!(label2.fn_path, "crate::会話_2::__start__");
    }

    #[test]
    fn test_register_local_label() {
        let mut registry = LabelRegistry::new();

        // Register parent first
        let (parent_id, parent_counter) = registry.register_global("会話", HashMap::new());

        // Register local label
        let (local_id, local_counter) =
            registry.register_local("選択肢", "会話", parent_counter, HashMap::new());

        assert_eq!(parent_id, 1);
        assert_eq!(local_id, 2);
        assert_eq!(local_counter, 1);

        let local_label = registry.get_label(local_id).unwrap();
        assert_eq!(local_label.name, "選択肢");
        assert_eq!(local_label.parent, Some("会話".to_string()));
        // Local label function is in parent module: crate::親_番号::子名_番号
        assert_eq!(local_label.fn_path, "crate::会話_1::選択肢_1");
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(LabelRegistry::sanitize_name("hello"), "hello");
        assert_eq!(LabelRegistry::sanitize_name("hello-world"), "hello_world");
        assert_eq!(LabelRegistry::sanitize_name("会話"), "会話");
        assert_eq!(LabelRegistry::sanitize_name("＊会話"), "_会話");
    }
}
