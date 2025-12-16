//! Tests for function scope resolution (Task 12)
//!
//! Tests the following scenarios:
//! - Local function priority (shadowing)
//! - Global function fallback
//! - Explicit global scope with @*
//! - Function not found error

use pasta::{FunctionScope, PastaError, TranspileContext};

#[test]
fn test_transpile_context_default_global_functions() {
    let context = TranspileContext::new();

    // Standard library functions should be in global scope
    assert!(context
        .resolve_function("emit_text", FunctionScope::Auto)
        .is_ok());
    assert!(context
        .resolve_function("change_speaker", FunctionScope::Auto)
        .is_ok());
    assert!(context
        .resolve_function("wait", FunctionScope::Auto)
        .is_ok());
}

#[test]
fn test_local_function_priority() {
    let mut context = TranspileContext::new();

    // Add a local function that shadows a global one
    context.set_local_functions(vec!["emit_text".to_string()]);

    // With Auto scope, local should be found first
    let result = context.resolve_function("emit_text", FunctionScope::Auto);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "emit_text");
}

#[test]
fn test_global_function_fallback() {
    let mut context = TranspileContext::new();

    // Add a local function
    context.set_local_functions(vec!["my_local_func".to_string()]);

    // Global functions should still be accessible when not in local scope
    let result = context.resolve_function("wait", FunctionScope::Auto);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "wait");
}

#[test]
fn test_explicit_global_scope() {
    let mut context = TranspileContext::new();

    // Add a local function that shadows a global one
    context.set_local_functions(vec!["emit_text".to_string()]);

    // With GlobalOnly scope, should skip local and find global
    let result = context.resolve_function("emit_text", FunctionScope::GlobalOnly);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "emit_text");
}

#[test]
fn test_function_not_found_auto_scope() {
    let context = TranspileContext::new();

    // With Auto scope, even non-existent functions are allowed
    // (might be defined in Rune blocks)
    let result = context.resolve_function("nonexistent_func", FunctionScope::Auto);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "nonexistent_func");
}

#[test]
fn test_function_not_found_global_only_scope() {
    let mut context = TranspileContext::new();

    // Add a local function
    context.set_local_functions(vec!["my_local_func".to_string()]);

    // With GlobalOnly scope, local function should not be found
    let result = context.resolve_function("my_local_func", FunctionScope::GlobalOnly);
    assert!(result.is_err());

    match result {
        Err(PastaError::FunctionNotFound { name }) => {
            assert_eq!(name, "my_local_func");
        }
        _ => panic!("Expected FunctionNotFound error"),
    }
}

#[test]
fn test_add_global_function() {
    let mut context = TranspileContext::new();

    // Add a custom global function
    context.add_global_function("custom_global".to_string());

    // Should be accessible via Auto scope
    let result = context.resolve_function("custom_global", FunctionScope::Auto);
    assert!(result.is_ok());

    // Should also be accessible via GlobalOnly scope
    let result = context.resolve_function("custom_global", FunctionScope::GlobalOnly);
    assert!(result.is_ok());
}

#[test]
fn test_shadowing_scenario() {
    let mut context = TranspileContext::new();

    // Add a global function
    context.add_global_function("format_text".to_string());

    // Add a local function with the same name
    context.set_local_functions(vec!["format_text".to_string()]);

    // With Auto scope, local should shadow global
    let result = context.resolve_function("format_text", FunctionScope::Auto);
    assert!(result.is_ok());

    // With GlobalOnly, should access the global version
    let result = context.resolve_function("format_text", FunctionScope::GlobalOnly);
    assert!(result.is_ok());
}

#[test]
fn test_multiple_local_functions() {
    let mut context = TranspileContext::new();

    // Add multiple local functions
    context.set_local_functions(vec![
        "func1".to_string(),
        "func2".to_string(),
        "func3".to_string(),
    ]);

    // All should be resolvable with Auto scope
    assert!(context
        .resolve_function("func1", FunctionScope::Auto)
        .is_ok());
    assert!(context
        .resolve_function("func2", FunctionScope::Auto)
        .is_ok());
    assert!(context
        .resolve_function("func3", FunctionScope::Auto)
        .is_ok());

    // None should be resolvable with GlobalOnly scope
    assert!(context
        .resolve_function("func1", FunctionScope::GlobalOnly)
        .is_err());
    assert!(context
        .resolve_function("func2", FunctionScope::GlobalOnly)
        .is_err());
    assert!(context
        .resolve_function("func3", FunctionScope::GlobalOnly)
        .is_err());
}

#[test]
fn test_empty_local_functions() {
    let mut context = TranspileContext::new();

    // Set empty local functions
    context.set_local_functions(vec![]);

    // Global functions should still be accessible
    assert!(context
        .resolve_function("emit_text", FunctionScope::Auto)
        .is_ok());
    assert!(context
        .resolve_function("wait", FunctionScope::Auto)
        .is_ok());
}

// Integration test: Parse and transpile a script with function calls
#[test]
fn test_integration_function_scope_resolution() {
    use pasta::{parse_str, Transpiler};

    let script = r#"
＊テスト
　　さくら：こんにちは＠笑顔
"#;

    // Parse the script
    let ast = parse_str(script, "test.pasta").expect("Failed to parse script");

    // Transpile
    let rune_code = Transpiler::transpile(&ast).expect("Failed to transpile");

    // Should contain a call to 笑顔 (which should be resolved from standard library)
    assert!(rune_code.contains("笑顔"));

    // The code should compile without FunctionNotFound errors
    assert!(!rune_code.is_empty());
}

// Test error message clarity for GlobalOnly scope
#[test]
fn test_function_not_found_error_message() {
    let context = TranspileContext::new();

    // GlobalOnly scope should error for undefined functions
    let result = context.resolve_function("undefined_function", FunctionScope::GlobalOnly);

    match result {
        Err(PastaError::FunctionNotFound { name }) => {
            assert_eq!(name, "undefined_function");
            // Verify error message format
            let error_string = format!("{}", PastaError::function_not_found("undefined_function"));
            assert!(error_string.contains("Function not found"));
            assert!(error_string.contains("undefined_function"));
        }
        _ => panic!("Expected FunctionNotFound error"),
    }
}
