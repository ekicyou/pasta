//! Variable management for Pasta scripts.
//!
//! This module provides variable storage and access with support for
//! local, global, and system scopes.

use std::collections::HashMap;

/// Variable scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableScope {
    /// Local to current scene execution.
    Local,
    /// Global across all script execution.
    Global,
    /// System-level variables (persisted across sessions).
    System,
}

/// Variable value type.
#[derive(Debug, Clone, PartialEq)]
pub enum VariableValue {
    /// String value.
    String(String),
    /// Integer value.
    Integer(i64),
    /// Boolean value.
    Boolean(bool),
    /// Float value.
    Float(f64),
}

impl VariableValue {
    /// Convert to string representation.
    pub fn to_string(&self) -> String {
        match self {
            VariableValue::String(s) => s.clone(),
            VariableValue::Integer(i) => i.to_string(),
            VariableValue::Boolean(b) => b.to_string(),
            VariableValue::Float(f) => f.to_string(),
        }
    }

    /// Try to convert to integer.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            VariableValue::Integer(i) => Some(*i),
            VariableValue::Float(f) => Some(*f as i64),
            VariableValue::Boolean(b) => Some(*b as i64),
            VariableValue::String(s) => s.parse().ok(),
        }
    }

    /// Try to convert to boolean.
    pub fn as_boolean(&self) -> bool {
        match self {
            VariableValue::Boolean(b) => *b,
            VariableValue::Integer(i) => *i != 0,
            VariableValue::Float(f) => *f != 0.0,
            VariableValue::String(s) => !s.is_empty() && s != "0" && s != "false",
        }
    }

    /// Try to convert to float.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            VariableValue::Float(f) => Some(*f),
            VariableValue::Integer(i) => Some(*i as f64),
            VariableValue::Boolean(b) => Some(*b as i64 as f64),
            VariableValue::String(s) => s.parse().ok(),
        }
    }
}

impl From<String> for VariableValue {
    fn from(s: String) -> Self {
        VariableValue::String(s)
    }
}

impl From<&str> for VariableValue {
    fn from(s: &str) -> Self {
        VariableValue::String(s.to_string())
    }
}

impl From<i64> for VariableValue {
    fn from(i: i64) -> Self {
        VariableValue::Integer(i)
    }
}

impl From<bool> for VariableValue {
    fn from(b: bool) -> Self {
        VariableValue::Boolean(b)
    }
}

impl From<f64> for VariableValue {
    fn from(f: f64) -> Self {
        VariableValue::Float(f)
    }
}

/// Variable manager for Pasta scripts.
pub struct VariableManager {
    /// Local variables (cleared after scene execution).
    local_vars: HashMap<String, VariableValue>,
    /// Global variables (persist during runtime).
    global_vars: HashMap<String, VariableValue>,
    /// System variables (persist across sessions).
    system_vars: HashMap<String, VariableValue>,
}

impl VariableManager {
    /// Create a new variable manager.
    pub fn new() -> Self {
        Self {
            local_vars: HashMap::new(),
            global_vars: HashMap::new(),
            system_vars: HashMap::new(),
        }
    }

    /// Get a variable value.
    pub fn get(&self, name: &str, scope: VariableScope) -> Option<&VariableValue> {
        match scope {
            VariableScope::Local => self.local_vars.get(name),
            VariableScope::Global => self.global_vars.get(name),
            VariableScope::System => self.system_vars.get(name),
        }
    }

    /// Set a variable value.
    pub fn set(&mut self, name: String, value: VariableValue, scope: VariableScope) {
        match scope {
            VariableScope::Local => {
                self.local_vars.insert(name, value);
            }
            VariableScope::Global => {
                self.global_vars.insert(name, value);
            }
            VariableScope::System => {
                self.system_vars.insert(name, value);
            }
        }
    }

    /// Clear local variables.
    pub fn clear_local(&mut self) {
        self.local_vars.clear();
    }

    /// Clear all variables.
    pub fn clear_all(&mut self) {
        self.local_vars.clear();
        self.global_vars.clear();
        // Note: system_vars are intentionally not cleared
    }

    /// Get all global variables (for persistence).
    pub fn global_vars(&self) -> &HashMap<String, VariableValue> {
        &self.global_vars
    }

    /// Get all system variables (for persistence).
    pub fn system_vars(&self) -> &HashMap<String, VariableValue> {
        &self.system_vars
    }

    /// Load global variables from a map.
    pub fn load_global_vars(&mut self, vars: HashMap<String, VariableValue>) {
        self.global_vars = vars;
    }

    /// Load system variables from a map.
    pub fn load_system_vars(&mut self, vars: HashMap<String, VariableValue>) {
        self.system_vars = vars;
    }
}

impl Default for VariableManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_value_conversions() {
        let str_val = VariableValue::String("42".to_string());
        assert_eq!(str_val.as_integer(), Some(42));
        assert_eq!(str_val.as_float(), Some(42.0));
        assert!(str_val.as_boolean());

        let int_val = VariableValue::Integer(0);
        assert!(!int_val.as_boolean());

        let bool_val = VariableValue::Boolean(true);
        assert_eq!(bool_val.as_integer(), Some(1));
    }

    #[test]
    fn test_variable_manager_basic() {
        let mut mgr = VariableManager::new();

        mgr.set(
            "x".to_string(),
            VariableValue::Integer(42),
            VariableScope::Local,
        );
        mgr.set(
            "y".to_string(),
            VariableValue::String("hello".to_string()),
            VariableScope::Global,
        );

        assert_eq!(
            mgr.get("x", VariableScope::Local),
            Some(&VariableValue::Integer(42))
        );
        assert_eq!(
            mgr.get("y", VariableScope::Global),
            Some(&VariableValue::String("hello".to_string()))
        );
        assert_eq!(mgr.get("z", VariableScope::Local), None);
    }

    #[test]
    fn test_variable_manager_clear_local() {
        let mut mgr = VariableManager::new();

        mgr.set(
            "x".to_string(),
            VariableValue::Integer(1),
            VariableScope::Local,
        );
        mgr.set(
            "y".to_string(),
            VariableValue::Integer(2),
            VariableScope::Global,
        );

        mgr.clear_local();

        assert_eq!(mgr.get("x", VariableScope::Local), None);
        assert_eq!(
            mgr.get("y", VariableScope::Global),
            Some(&VariableValue::Integer(2))
        );
    }

    #[test]
    fn test_variable_manager_scopes() {
        let mut mgr = VariableManager::new();

        mgr.set(
            "var".to_string(),
            VariableValue::Integer(1),
            VariableScope::Local,
        );
        mgr.set(
            "var".to_string(),
            VariableValue::Integer(2),
            VariableScope::Global,
        );
        mgr.set(
            "var".to_string(),
            VariableValue::Integer(3),
            VariableScope::System,
        );

        assert_eq!(
            mgr.get("var", VariableScope::Local),
            Some(&VariableValue::Integer(1))
        );
        assert_eq!(
            mgr.get("var", VariableScope::Global),
            Some(&VariableValue::Integer(2))
        );
        assert_eq!(
            mgr.get("var", VariableScope::System),
            Some(&VariableValue::Integer(3))
        );
    }

    #[test]
    fn test_variable_manager_load() {
        let mut mgr = VariableManager::new();

        let mut globals = HashMap::new();
        globals.insert("x".to_string(), VariableValue::Integer(42));
        mgr.load_global_vars(globals);

        assert_eq!(
            mgr.get("x", VariableScope::Global),
            Some(&VariableValue::Integer(42))
        );
    }
}
