//! Transpile context for transpiler2.
//!
//! This module provides the TranspileContext2 struct which tracks
//! scope information, accumulated attributes, and function resolution
//! during transpilation.

use super::TranspileError;
use pasta_core::parser::{Attr, AttrValue, FnScope};
use std::collections::HashMap;

/// Transpile context for parser2 AST transpilation.
///
/// Tracks local/global functions, current module, and accumulated
/// file-level attributes.
#[derive(Clone, Debug)]
pub struct TranspileContext2 {
    /// List of local function names defined in the current scene
    local_functions: Vec<String>,
    /// List of global function names (standard library + user-defined)
    global_functions: Vec<String>,
    /// Current module name (sanitized global scene name) for word lookup
    current_module: String,
    /// File-level attributes accumulated from FileAttr items
    file_attrs: HashMap<String, AttrValue>,
}

impl TranspileContext2 {
    /// Create a new transpile context.
    pub fn new() -> Self {
        Self {
            local_functions: Vec::new(),
            global_functions: Self::default_global_functions(),
            current_module: String::new(),
            file_attrs: HashMap::new(),
        }
    }

    /// Get default global functions (standard library).
    fn default_global_functions() -> Vec<String> {
        vec![
            "emit_text".to_string(),
            "emit_sakura_script".to_string(),
            "change_speaker".to_string(),
            "change_surface".to_string(),
            "wait".to_string(),
            "begin_sync".to_string(),
            "sync_point".to_string(),
            "end_sync".to_string(),
            "fire_event".to_string(),
        ]
    }

    /// Set local functions for the current scene scope.
    pub fn set_local_functions(&mut self, functions: Vec<String>) {
        self.local_functions = functions;
    }

    /// Add a global function to the list.
    pub fn add_global_function(&mut self, name: String) {
        if !self.global_functions.contains(&name) {
            self.global_functions.push(name);
        }
    }

    /// Set the current module name for word lookup.
    pub fn set_current_module(&mut self, module_name: String) {
        self.current_module = module_name;
    }

    /// Get the current module name.
    pub fn current_module(&self) -> &str {
        &self.current_module
    }

    /// Accumulate file-level attribute.
    ///
    /// Multiple FileAttr items are processed in order. If the same key
    /// appears multiple times, the later value overwrites the earlier one.
    pub fn accumulate_file_attr(&mut self, attr: &Attr) {
        self.file_attrs.insert(attr.key.clone(), attr.value.clone());
    }

    /// Get accumulated file-level attributes.
    pub fn file_attrs(&self) -> &HashMap<String, AttrValue> {
        &self.file_attrs
    }

    /// Merge scene attributes with file attributes.
    ///
    /// Merge rules:
    /// 1. Start with all keys from file_attrs as the base
    /// 2. Overwrite with each key from scene_attrs (scene takes priority)
    /// 3. Return the merged result as HashMap<String, AttrValue>
    pub fn merge_attrs(&self, scene_attrs: &[Attr]) -> HashMap<String, AttrValue> {
        let mut result = self.file_attrs.clone();

        for attr in scene_attrs {
            result.insert(attr.key.clone(), attr.value.clone());
        }

        result
    }

    /// Resolve function name with scope rules (local→global search).
    ///
    /// For FnScope::Local, searches local functions first, then global.
    /// For FnScope::Global, searches only global functions.
    ///
    /// If the function is not found in tracked scopes, it is still returned
    /// as-is because it might be defined in a Rune block. The Rune runtime
    /// will handle the error if the function truly doesn't exist.
    pub fn resolve_function(
        &self,
        func_name: &str,
        scope: FnScope,
    ) -> Result<String, TranspileError> {
        match scope {
            FnScope::Local => {
                // 1. Search local functions first
                if self.local_functions.contains(&func_name.to_string()) {
                    Ok(func_name.to_string())
                }
                // 2. Search global functions
                else if self.global_functions.contains(&func_name.to_string()) {
                    Ok(func_name.to_string())
                } else {
                    // 3. Function not in tracked scopes, but might be defined in Rune block
                    Ok(func_name.to_string())
                }
            }
            FnScope::Global => {
                // Search global functions only
                if self.global_functions.contains(&func_name.to_string()) {
                    Ok(func_name.to_string())
                } else {
                    // Not in global scope - return error for strict global scope
                    Err(TranspileError::undefined_symbol(format!(
                        "global function: {}",
                        func_name
                    )))
                }
            }
        }
    }
}

impl Default for TranspileContext2 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pasta_core::parser::Span;

    fn make_attr(key: &str, value: AttrValue) -> Attr {
        Attr {
            key: key.to_string(),
            value,
            span: Span::default(),
        }
    }

    #[test]
    fn test_new_context() {
        let ctx = TranspileContext2::new();
        assert!(ctx.current_module().is_empty());
        assert!(ctx.file_attrs().is_empty());
    }

    #[test]
    fn test_accumulate_file_attr() {
        let mut ctx = TranspileContext2::new();

        let attr1 = make_attr("author", AttrValue::String("test".to_string()));
        let attr2 = make_attr("version", AttrValue::Integer(1));

        ctx.accumulate_file_attr(&attr1);
        ctx.accumulate_file_attr(&attr2);

        assert_eq!(ctx.file_attrs().len(), 2);
        assert!(matches!(
            ctx.file_attrs().get("author"),
            Some(AttrValue::String(s)) if s == "test"
        ));
        assert!(matches!(
            ctx.file_attrs().get("version"),
            Some(AttrValue::Integer(1))
        ));
    }

    #[test]
    fn test_accumulate_file_attr_overwrite() {
        let mut ctx = TranspileContext2::new();

        let attr1 = make_attr("key", AttrValue::String("first".to_string()));
        let attr2 = make_attr("key", AttrValue::String("second".to_string()));

        ctx.accumulate_file_attr(&attr1);
        ctx.accumulate_file_attr(&attr2);

        // Later value should overwrite
        assert_eq!(ctx.file_attrs().len(), 1);
        assert!(matches!(
            ctx.file_attrs().get("key"),
            Some(AttrValue::String(s)) if s == "second"
        ));
    }

    #[test]
    fn test_merge_attrs_scene_priority() {
        let mut ctx = TranspileContext2::new();

        // File-level attributes
        let file_attr = make_attr("common", AttrValue::String("file".to_string()));
        ctx.accumulate_file_attr(&file_attr);

        // Scene-level attributes (should override)
        let scene_attrs = vec![make_attr("common", AttrValue::String("scene".to_string()))];

        let merged = ctx.merge_attrs(&scene_attrs);

        assert!(matches!(
            merged.get("common"),
            Some(AttrValue::String(s)) if s == "scene"
        ));
    }

    #[test]
    fn test_merge_attrs_combines() {
        let mut ctx = TranspileContext2::new();

        let file_attr = make_attr("file_only", AttrValue::Integer(1));
        ctx.accumulate_file_attr(&file_attr);

        let scene_attrs = vec![make_attr("scene_only", AttrValue::Integer(2))];

        let merged = ctx.merge_attrs(&scene_attrs);

        assert_eq!(merged.len(), 2);
        assert!(merged.contains_key("file_only"));
        assert!(merged.contains_key("scene_only"));
    }

    #[test]
    fn test_resolve_function_local() {
        let mut ctx = TranspileContext2::new();
        ctx.set_local_functions(vec!["my_func".to_string()]);

        let result = ctx.resolve_function("my_func", FnScope::Local);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my_func");
    }

    #[test]
    fn test_resolve_function_global_stdlib() {
        let ctx = TranspileContext2::new();

        let result = ctx.resolve_function("emit_text", FnScope::Local);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "emit_text");
    }

    #[test]
    fn test_resolve_function_global_scope_strict() {
        let ctx = TranspileContext2::new();

        // Unknown function with Global scope should fail
        let result = ctx.resolve_function("unknown_func", FnScope::Global);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_function_unknown_local_scope() {
        let ctx = TranspileContext2::new();

        // Unknown function with Local scope is allowed (might be in Rune block)
        let result = ctx.resolve_function("unknown_func", FnScope::Local);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_current_module() {
        let mut ctx = TranspileContext2::new();
        ctx.set_current_module("test_module".to_string());
        assert_eq!(ctx.current_module(), "test_module");
    }
}
