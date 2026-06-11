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
    // Span は Copy（Copy: Clone なので Clone 実装も保証される）
    let span = Span::new(1, 1, 1, 1, 0, 1);
    let _span2 = span;

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

// ============================================================================
// word-multi-key: 複数キー単語定義のパーステスト
// ============================================================================

/// 単一キーの従来形式が names.len() == 1 を返す（後方互換性）
#[test]
fn test_parse_single_key_word_definition() {
    let source = "＠女性：水無灯里、アリス・キャロル\n";
    let file = parse_str(source, "test.pasta").expect("パース成功すべし");

    let word = match &file.items[0] {
        FileItem::GlobalWord(w) => w,
        other => panic!("GlobalWord を期待, got: {:?}", other),
    };
    assert_eq!(word.names.len(), 1);
    assert_eq!(word.name(), "女性");
    assert_eq!(word.words, vec!["水無灯里", "アリス・キャロル"]);
}

/// 2キー指定のパーステスト
#[test]
fn test_parse_two_key_word_definition() {
    let source = "＠女性、水の妖精：水無灯里、アリス・キャロル\n";
    let file = parse_str(source, "test.pasta").expect("パース成功すべし");

    let word = match &file.items[0] {
        FileItem::GlobalWord(w) => w,
        other => panic!("GlobalWord を期待, got: {:?}", other),
    };
    assert_eq!(word.names.len(), 2);
    assert_eq!(word.names[0], "女性");
    assert_eq!(word.names[1], "水の妖精");
    assert_eq!(word.name(), "女性");
    assert_eq!(word.words, vec!["水無灯里", "アリス・キャロル"]);
}

/// 3キー以上のパーステスト
#[test]
fn test_parse_three_key_word_definition() {
    let source = "＠人物、女性、水の妖精：水無灯里、アリス・キャロル\n";
    let file = parse_str(source, "test.pasta").expect("パース成功すべし");

    let word = match &file.items[0] {
        FileItem::GlobalWord(w) => w,
        other => panic!("GlobalWord を期待, got: {:?}", other),
    };
    assert_eq!(word.names.len(), 3);
    assert_eq!(word.names[0], "人物");
    assert_eq!(word.names[1], "女性");
    assert_eq!(word.names[2], "水の妖精");
    assert_eq!(word.words, vec!["水無灯里", "アリス・キャロル"]);
}

/// 半角カンマでのキー区切り
#[test]
fn test_parse_multi_key_half_width_comma() {
    let source = "＠人物,女性：水無灯里\n";
    let file = parse_str(source, "test.pasta").expect("パース成功すべし");

    let word = match &file.items[0] {
        FileItem::GlobalWord(w) => w,
        other => panic!("GlobalWord を期待, got: {:?}", other),
    };
    assert_eq!(word.names.len(), 2);
    assert_eq!(word.names[0], "人物");
    assert_eq!(word.names[1], "女性");
}

/// シーンスコープ内での複数キー単語定義
#[test]
fn test_parse_multi_key_in_scene_scope() {
    let source = "＊メイン\n　＠場所、地名：東京、大阪\n　　さくら：テスト。\n";
    let file = parse_str(source, "test.pasta").expect("パース成功すべし");

    let scene = match &file.items[0] {
        FileItem::GlobalSceneScope(s) => s,
        other => panic!("GlobalSceneScope を期待, got: {:?}", other),
    };
    assert_eq!(scene.words.len(), 1);
    assert_eq!(scene.words[0].names.len(), 2);
    assert_eq!(scene.words[0].names[0], "場所");
    assert_eq!(scene.words[0].names[1], "地名");
    assert_eq!(scene.words[0].words, vec!["東京", "大阪"]);
}

/// アクター辞書内での複数キー単語定義
#[test]
fn test_parse_multi_key_in_actor_scope() {
    let source = "％さくら\n　＠通常、普通：\\s[0]、\\s[1]\n";
    let file = parse_str(source, "test.pasta").expect("パース成功すべし");

    let actor = match &file.items[0] {
        FileItem::ActorScope(a) => a,
        other => panic!("ActorScope を期待, got: {:?}", other),
    };
    assert_eq!(actor.words.len(), 1);
    assert_eq!(actor.words[0].names.len(), 2);
    assert_eq!(actor.words[0].names[0], "通常");
    assert_eq!(actor.words[0].names[1], "普通");
}
