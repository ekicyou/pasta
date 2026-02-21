//! AST 型定義の外部テスト
//!
//! Phase A: parser/ast.rs のインラインテストから外部化

use pasta_dsl::parser::*;
use std::path::PathBuf;

#[test]
fn test_span_new() {
    let span = Span::new(1, 1, 1, 10, 0, 10);
    assert_eq!(span.start_line, 1);
    assert_eq!(span.start_col, 1);
    assert_eq!(span.end_line, 1);
    assert_eq!(span.end_col, 10);
    assert_eq!(span.start_byte, 0);
    assert_eq!(span.end_byte, 10);
}

#[test]
fn test_span_default() {
    let span = Span::default();
    assert_eq!(span.start_line, 0);
    assert_eq!(span.start_col, 0);
    assert_eq!(span.end_line, 0);
    assert_eq!(span.end_col, 0);
    assert_eq!(span.start_byte, 0);
    assert_eq!(span.end_byte, 0);
}

#[test]
fn test_file_scope_default() {
    let scope = FileScope::default();
    assert!(scope.attrs.is_empty());
    assert!(scope.words.is_empty());
}

#[test]
fn test_pasta_file_new() {
    let file = PastaFile::new(PathBuf::from("test.pasta"));
    assert_eq!(file.path, PathBuf::from("test.pasta"));
    assert!(file.items.is_empty());
}

#[test]
fn test_global_scene_scope_new() {
    let scene = GlobalSceneScope::new("挨拶".to_string());
    assert_eq!(scene.name, "挨拶");
    assert!(!scene.is_continuation);
}

#[test]
fn test_global_scene_scope_continuation() {
    let scene = GlobalSceneScope::continuation("挨拶".to_string());
    assert_eq!(scene.name, "挨拶");
    assert!(scene.is_continuation);
}

#[test]
fn test_local_scene_scope_start() {
    let scene = LocalSceneScope::start();
    assert!(scene.name.is_none());
}

#[test]
fn test_local_scene_scope_named() {
    let scene = LocalSceneScope::named("hello".to_string());
    assert_eq!(scene.name, Some("hello".to_string()));
}

#[test]
fn test_args_empty() {
    let args = Args::empty();
    assert!(args.items.is_empty());
}

#[test]
fn test_var_scope_equality() {
    assert_eq!(VarScope::Local, VarScope::Local);
    assert_ne!(VarScope::Local, VarScope::Global);
}

#[test]
fn test_fn_scope_equality() {
    assert_eq!(FnScope::Local, FnScope::Local);
    assert_ne!(FnScope::Local, FnScope::Global);
}

#[test]
fn test_bin_op_equality() {
    assert_eq!(BinOp::Add, BinOp::Add);
    assert_ne!(BinOp::Add, BinOp::Sub);
}

#[test]
fn test_ast_types_clone() {
    // Test that all AST types implement Clone
    let span = Span::new(1, 1, 1, 1, 0, 1);
    let _span2 = span.clone();

    let file = PastaFile::new(PathBuf::from("test.pasta"));
    let _file2 = file.clone();

    let attr = Attr {
        key: "test".to_string(),
        value: AttrValue::Integer(42),
        span: Span::default(),
    };
    let _attr2 = attr.clone();
}

#[test]
fn test_ast_types_debug() {
    // Test that all AST types implement Debug
    let span = Span::new(1, 1, 1, 1, 0, 1);
    let _ = format!("{:?}", span);

    let file = PastaFile::new(PathBuf::from("test.pasta"));
    let _ = format!("{:?}", file);

    let expr = Expr::Integer(42);
    let _ = format!("{:?}", expr);

    let action = Action::Talk {
        text: "hello".to_string(),
        span: Span::default(),
    };
    let _ = format!("{:?}", action);
}
