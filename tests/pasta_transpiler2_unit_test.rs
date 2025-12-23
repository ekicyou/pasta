//! Unit tests for transpiler2 module.
//!
//! These tests verify the transpiler2 layer through its public API:
//! - Transpiler2 pass1/pass2 functionality
//! - TranspileContext2 attribute and function resolution
//! - TranspileError patterns
//!
//! Note: CodeGenerator internal methods are tested via mod.rs internal tests.

use pasta::parser2::{self, FnScope, Span};
use pasta::registry::{SceneRegistry, WordDefRegistry};
use pasta::transpiler2::{TranspileContext2, TranspileError, Transpiler2};

// ====================
// TranspileContext2 Tests
// ====================

#[test]
fn test_context_set_current_module() {
    let mut ctx = TranspileContext2::new();

    ctx.set_current_module("test_module".to_string());
    assert_eq!(ctx.current_module(), "test_module");
}

#[test]
fn test_context_resolve_function_local() {
    let mut ctx = TranspileContext2::new();
    ctx.set_local_functions(vec!["helper".to_string()]);
    ctx.set_current_module("test_mod".to_string());

    let result = ctx.resolve_function("helper", FnScope::Local);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "helper");
}

#[test]
fn test_context_resolve_function_global() {
    let mut ctx = TranspileContext2::new();
    ctx.add_global_function("global_fn".to_string());
    ctx.set_current_module("test_mod".to_string());

    // From local scope, global function is accessible without prefix
    let result = ctx.resolve_function("global_fn", FnScope::Local);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "global_fn");
}

#[test]
fn test_context_resolve_function_stdlib() {
    let ctx = TranspileContext2::new();

    // stdlib functions like emit_text should always be accessible
    let result = ctx.resolve_function("emit_text", FnScope::Global);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "emit_text");
}

#[test]
fn test_context_resolve_function_unknown_local_scope() {
    let ctx = TranspileContext2::new();

    // Unknown function from local scope returns as-is (Rune will validate)
    let result = ctx.resolve_function("unknown_func", FnScope::Local);
    assert!(result.is_ok());
}

#[test]
fn test_context_resolve_function_unknown_global_scope() {
    let ctx = TranspileContext2::new();

    // Unknown function from global scope returns error
    let result = ctx.resolve_function("unknown_func", FnScope::Global);
    assert!(result.is_err());
}

#[test]
fn test_context_file_attrs_accumulation() {
    let mut ctx = TranspileContext2::new();

    let attr1 = pasta::parser2::Attr {
        key: "天気".to_string(),
        value: pasta::parser2::AttrValue::AttrString("晴れ".to_string()),
        span: Span::default(),
    };
    let attr2 = pasta::parser2::Attr {
        key: "季節".to_string(),
        value: pasta::parser2::AttrValue::AttrString("夏".to_string()),
        span: Span::default(),
    };

    ctx.accumulate_file_attr(&attr1);
    ctx.accumulate_file_attr(&attr2);

    let attrs = ctx.file_attrs();
    assert_eq!(attrs.len(), 2);
    assert_eq!(
        attrs.get("天気"),
        Some(&pasta::parser2::AttrValue::AttrString("晴れ".to_string()))
    );
}

#[test]
fn test_context_merge_attrs_scene_priority() {
    let mut ctx = TranspileContext2::new();

    // File-level attr
    let file_attr = pasta::parser2::Attr {
        key: "季節".to_string(),
        value: pasta::parser2::AttrValue::AttrString("冬".to_string()),
        span: Span::default(),
    };
    ctx.accumulate_file_attr(&file_attr);

    // Scene-level attrs override file-level
    let scene_attrs = vec![pasta::parser2::Attr {
        key: "季節".to_string(),
        value: pasta::parser2::AttrValue::AttrString("夏".to_string()),
        span: Span::default(),
    }];

    let merged = ctx.merge_attrs(&scene_attrs);
    assert_eq!(
        merged.get("季節"),
        Some(&pasta::parser2::AttrValue::AttrString("夏".to_string()))
    );
}

// ====================
// TranspileError Tests
// ====================

#[test]
fn test_error_invalid_ast() {
    let span = Span::new(10, 5, 10, 15);
    let err = TranspileError::invalid_ast(&span, "unexpected token");

    assert!(err.to_string().contains("10:5"));
    assert!(err.to_string().contains("unexpected token"));
}

#[test]
fn test_error_undefined_symbol() {
    let err = TranspileError::undefined_symbol("unknown");
    match err {
        TranspileError::UndefinedSymbol { symbol } => {
            assert_eq!(symbol, "unknown");
        }
        _ => panic!("Expected UndefinedSymbol error"),
    }
}

#[test]
fn test_error_invalid_continuation() {
    let span = Span::new(20, 1, 20, 10);
    let err = TranspileError::invalid_continuation(&span);
    assert!(matches!(err, TranspileError::InvalidContinuation { .. }));
}

#[test]
fn test_error_internal() {
    let err = TranspileError::internal("internal problem");
    match err {
        TranspileError::InternalError(message) => {
            assert_eq!(message, "internal problem");
        }
        _ => panic!("Expected InternalError"),
    }
}

// ====================
// Transpiler2 Integration Tests
// ====================

#[test]
fn test_transpile_to_string_empty_file() {
    let file = pasta::parser2::PastaFile::new(std::path::PathBuf::from("test.pasta"));
    let result = Transpiler2::transpile_to_string(&file);
    assert!(result.is_ok());

    let output = result.unwrap();
    // Should contain the selector module even for empty file
    assert!(output.contains("__pasta_trans2__"));
    assert!(output.contains("mod pasta"));
}

#[test]
fn test_transpile_pass1_scene_registration() {
    let source = "＊テスト\n  actor：hello\n";
    let file = parser2::parse_str(source, "test.pasta").unwrap();

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
        .unwrap();

    let scenes = scene_registry.all_scenes();
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0].name, "テスト");
}

#[test]
fn test_transpile_pass1_word_registration() {
    let source = "＠挨拶：hello、hi\n＊シーン\n  actor：test\n";
    let file = parser2::parse_str(source, "test.pasta").unwrap();

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
        .unwrap();

    let entries = word_registry.all_entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].key, "挨拶");
}

#[test]
fn test_transpile_pass2_generates_selector() {
    let source = "＊会話\n  actor：hello\n";
    let file = parser2::parse_str(source, "test.pasta").unwrap();

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
        .unwrap();
    Transpiler2::transpile_pass2(&scene_registry, &mut output).unwrap();

    let result = String::from_utf8(output).unwrap();
    assert!(result.contains("pub mod __pasta_trans2__"));
    assert!(result.contains("pub fn scene_selector"));
    assert!(result.contains("pub mod pasta"));
    assert!(result.contains("pub fn call"));
}

#[test]
fn test_transpile_full_scene() {
    let source = "＊挨拶\n  さくら：「こんにちは」\n";
    let file = parser2::parse_str(source, "test.pasta").unwrap();

    let result = Transpiler2::transpile_to_string(&file).unwrap();

    // Verify module structure
    assert!(result.contains("pub mod 挨拶_1"));
    assert!(result.contains("pub fn __start__"));
    assert!(result.contains("yield change_speaker"));
    assert!(result.contains("yield Talk"));
}

#[test]
fn test_transpile_multiple_scenes() {
    let source = "＊挨拶\n  sakura：hello\n＊会話\n  kero：hi\n";
    let file = parser2::parse_str(source, "test.pasta").unwrap();

    let result = Transpiler2::transpile_to_string(&file).unwrap();

    // Verify both scenes are generated
    assert!(result.contains("pub mod 挨拶_1"));
    assert!(result.contains("pub mod 会話_1"));

    // Verify scene_selector matches both
    assert!(result.contains("1 => crate::挨拶_1::__start__"));
    assert!(result.contains("2 => crate::会話_1::__start__"));
}

#[test]
fn test_transpile_scene_with_file_level_attributes() {
    let source = "＆天気：晴れ\n＊会話\n  sakura：hello\n";
    let file = parser2::parse_str(source, "test.pasta").unwrap();

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
        .unwrap();

    let scenes = scene_registry.all_scenes();
    assert_eq!(scenes.len(), 1);

    // Scene should have file-level attribute inherited
    let scene = &scenes[0];
    assert!(scene.attributes.contains_key("天気"));
}

#[test]
fn test_transpile_global_word_definition() {
    let source = "＠挨拶：こんにちは、おはよう\n＊シーン\n  actor：test\n";
    let file = parser2::parse_str(source, "test.pasta").unwrap();

    let mut scene_registry = SceneRegistry::new();
    let mut word_registry = WordDefRegistry::new();
    let mut output = Vec::new();

    Transpiler2::transpile_pass1(&file, &mut scene_registry, &mut word_registry, &mut output)
        .unwrap();

    let entries = word_registry.all_entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].values, vec!["こんにちは", "おはよう"]);
}
