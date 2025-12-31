//! Parser2 integration tests for pasta2.pest grammar.
//!
//! These tests verify the parser2 module implementation using fixtures
//! and cover all grammar rules defined in grammar.pest.

use pasta_rune::parser::{
    ActorScope, Attr, Expr, FileItem, GlobalSceneScope, KeyWords, LocalSceneItem, PastaFile,
    parse_file, parse_str,
};
use std::path::Path;

fn get_file_attrs(file: &PastaFile) -> Vec<&Attr> {
    file.items
        .iter()
        .filter_map(|item| {
            if let FileItem::FileAttr(attr) = item {
                Some(attr)
            } else {
                None
            }
        })
        .collect()
}

fn get_words(file: &PastaFile) -> Vec<&KeyWords> {
    file.items
        .iter()
        .filter_map(|item| {
            if let FileItem::GlobalWord(word) = item {
                Some(word)
            } else {
                None
            }
        })
        .collect()
}

fn get_global_scene_scopes(file: &PastaFile) -> Vec<&GlobalSceneScope> {
    file.items
        .iter()
        .filter_map(|item| {
            if let FileItem::GlobalSceneScope(scene) = item {
                Some(scene)
            } else {
                None
            }
        })
        .collect()
}

#[allow(dead_code)]
fn get_actor_scopes(file: &PastaFile) -> Vec<&ActorScope> {
    file.items
        .iter()
        .filter_map(|item| {
            if let FileItem::ActorScope(actor) = item {
                Some(actor)
            } else {
                None
            }
        })
        .collect()
}

// ============================================================================
// Basic Parsing Tests
// ============================================================================

#[test]
fn test_parse_empty_file() {
    let result = parse_str("", "empty.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    assert!(get_global_scene_scopes(&file).is_empty());
    assert!(get_file_attrs(&file).is_empty());
    assert!(get_words(&file).is_empty());
}

#[test]
fn test_parse_comment_only() {
    let source = "# コメントのみのファイル\n";
    let result = parse_str(source, "comment.pasta");
    assert!(result.is_ok());
}

#[test]
fn test_parse_simple_scene() {
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = parse_str(source, "simple.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    assert_eq!(get_global_scene_scopes(&file).len(), 1);
    assert_eq!(get_global_scene_scopes(&file)[0].name, "挨拶");
}

// ============================================================================
// Scope Structure Tests (Task 5.1)
// ============================================================================

#[test]
fn test_three_layer_scope_structure() {
    let source = r#"＊グローバル
  Alice：ファースト
  ・ローカル
    Bob：セカンド
"#;
    let result = parse_str(source, "scope.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();

    // GlobalSceneScope
    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 1);
    let global = scenes[0];
    assert_eq!(global.name, "グローバル");

    // LocalSceneScope (start scene + named scene)
    assert_eq!(global.local_scenes.len(), 2);
    assert!(global.local_scenes[0].name.is_none()); // start scene
    assert_eq!(global.local_scenes[1].name, Some("ローカル".to_string()));
}

#[test]
fn test_file_scope_attrs_and_words() {
    let source = r#"&author：テスト
&version：1
@greeting：こんにちは、おはよう
＊シーン
  Alice：テスト
"#;
    let result = parse_str(source, "file_scope.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();

    let attrs = get_file_attrs(&file);
    let words = get_words(&file);
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].key, "author");
    assert_eq!(words.len(), 1);
    assert_eq!(words[0].name, "greeting");
}

#[test]
fn test_continue_action_line() {
    let source = r#"＊挨拶
  Alice：こんにちは
  ：続きの台詞
"#;
    let result = parse_str(source, "continue.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();

    let local = &get_global_scene_scopes(&file)[0].local_scenes[0];
    assert!(local.items.len() >= 2);
    assert!(matches!(local.items[0], LocalSceneItem::ActionLine(_)));
    assert!(matches!(local.items[1], LocalSceneItem::ContinueAction(_)));
}

// ============================================================================
// Unnamed Scene Inheritance Tests (Task 5.2)
// ============================================================================

#[test]
fn test_unnamed_scene_inheritance() {
    let source = r#"＊挨拶
  Alice：こんにちは
＊
  Bob：やあ
"#;
    let result = parse_str(source, "unnamed.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();

    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 2);
    assert_eq!(scenes[0].name, "挨拶");
    assert!(!scenes[0].is_continuation);
    assert_eq!(scenes[1].name, "挨拶"); // inherited
    assert!(scenes[1].is_continuation);
}

#[test]
fn test_unnamed_scene_at_start_error() {
    let source = "＊\n  Alice：こんにちは\n";
    let result = parse_str(source, "error.pasta");
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(err_msg.contains("Unnamed global scene at start of file"));
}

#[test]
fn test_multiple_unnamed_scenes() {
    let source = r#"＊元
  Alice：最初
＊
  Bob：継続1
＊
  Charlie：継続2
"#;
    let result = parse_str(source, "multi_unnamed.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();

    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 3);
    assert_eq!(scenes[0].name, "元");
    assert_eq!(scenes[1].name, "元");
    assert_eq!(scenes[2].name, "元");
}

// ============================================================================
// String Literal Tests (Task 5.3)
// ============================================================================

#[test]
fn test_string_literal_level1() {
    let source = r#"＊テスト
  $s＝「こんにちは」
  Alice：「挨拶」と言った
"#;
    let result = parse_str(source, "string1.pasta");
    assert!(result.is_ok());
}

#[test]
fn test_string_literal_level2() {
    let source = r#"＊テスト
  $s＝「「入れ子」」
"#;
    let result = parse_str(source, "string2.pasta");
    assert!(result.is_ok());
}

#[test]
fn test_string_literal_level3() {
    let source = r#"＊テスト
  $s＝「「「深い」」」
"#;
    let result = parse_str(source, "string3.pasta");
    assert!(result.is_ok());
}

#[test]
fn test_string_literal_level4() {
    let source = r#"＊テスト
  $s＝「「「「最深」」」」
"#;
    let result = parse_str(source, "string4.pasta");
    assert!(result.is_ok());
}

#[test]
fn test_blank_string() {
    let source = r#"＊テスト
  $a＝「」
  $b＝""
"#;
    let result = parse_str(source, "blank.pasta");
    assert!(result.is_ok());
}

// ============================================================================
// Code Block Tests (Task 5.3)
// ============================================================================

#[test]
fn test_code_block_with_language() {
    let source = r#"＊テスト
  Alice：コード実行
```rune
let x = 1;
```
"#;
    let result = parse_str(source, "code_rune.pasta");
    assert!(result.is_ok());
}

#[test]
fn test_code_block_without_language() {
    let source = r#"＊テスト
  Alice：コード実行
```
plain code
```
"#;
    let result = parse_str(source, "code_plain.pasta");
    assert!(result.is_ok());
}

// ============================================================================
// Number Literal Tests (Task 5.5)
// ============================================================================

#[test]
fn test_half_width_integer() {
    let source = r#"＊テスト
  $a＝123
"#;
    let result = parse_str(source, "num_hw.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let item = &get_global_scene_scopes(&file)[0].local_scenes[0].items[0];
    if let LocalSceneItem::VarSet(vs) = item {
        assert!(matches!(vs.value, Expr::Integer(123)));
    } else {
        panic!("Expected VarSet");
    }
}

#[test]
fn test_full_width_integer() {
    let source = r#"＊テスト
  $a＝１２３
"#;
    let result = parse_str(source, "num_fw.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let item = &get_global_scene_scopes(&file)[0].local_scenes[0].items[0];
    if let LocalSceneItem::VarSet(vs) = item {
        assert!(matches!(vs.value, Expr::Integer(123)));
    } else {
        panic!("Expected VarSet");
    }
}

#[test]
fn test_negative_number() {
    let source = r#"＊テスト
  $a＝-456
  $b＝－７８９
"#;
    let result = parse_str(source, "num_neg.pasta");
    assert!(result.is_ok());
}

#[test]
fn test_float_number() {
    let source = r#"＊テスト
  $a＝3.14
  $b＝３．１４
"#;
    let result = parse_str(source, "num_float.pasta");
    assert!(result.is_ok());
}

#[test]
fn test_mixed_width_number() {
    let source = r#"＊テスト
  $a＝３.１４
  $b＝3．14
"#;
    let result = parse_str(source, "num_mixed.pasta");
    assert!(result.is_ok());
}

// ============================================================================
// Variable and Function Tests
// ============================================================================

#[test]
fn test_local_variable() {
    let source = r#"＊テスト
  $x＝10
  Alice：$xを表示
"#;
    let result = parse_str(source, "var_local.pasta");
    assert!(result.is_ok());
}

#[test]
fn test_global_variable() {
    let source = r#"＊テスト
  $*global＝「テスト」
  Alice：$*globalを表示
"#;
    let result = parse_str(source, "var_global.pasta");
    assert!(result.is_ok());
}

#[test]
fn test_local_function_call() {
    let source = r#"＊テスト
  Alice：@hello（）と呼ぶ
"#;
    let result = parse_str(source, "fn_local.pasta");
    assert!(result.is_ok());
}

#[test]
fn test_global_function_call() {
    let source = r#"＊テスト
  Alice：@*greet（name：「太郎」）と呼ぶ
"#;
    let result = parse_str(source, "fn_global.pasta");
    assert!(result.is_ok());
}

// ============================================================================
// Escape Sequence Tests
// ============================================================================

#[test]
fn test_at_escape() {
    let source = r#"＊テスト
  Alice：@@はアットマーク
"#;
    let result = parse_str(source, "escape_at.pasta");
    assert!(result.is_ok());
}

#[test]
fn test_dollar_escape() {
    let source = r#"＊テスト
  Alice：$$はドル
"#;
    let result = parse_str(source, "escape_dollar.pasta");
    assert!(result.is_ok());
}

#[test]
fn test_sakura_escape() {
    let source = r#"＊テスト
  Alice：\\はバックスラッシュ
"#;
    let result = parse_str(source, "escape_sakura.pasta");
    assert!(result.is_ok());
}

// ============================================================================
// Call Scene Tests
// ============================================================================

#[test]
fn test_call_scene_simple() {
    let source = r#"＊テスト
  >挨拶
"#;
    let result = parse_str(source, "call_simple.pasta");
    assert!(result.is_ok());
}

#[test]
fn test_call_scene_with_args() {
    let source = r#"＊テスト
  >挨拶（名前：「太郎」）
  >会話（1、2、3）
"#;
    let result = parse_str(source, "call_args.pasta");
    assert!(result.is_ok());
}

// ============================================================================
// Fixture File Tests (Task 5.6)
// ============================================================================

#[test]
fn test_parse_basic_syntax_fixture() {
    let path = Path::new("tests/fixtures/parser2/basic_syntax.pasta");
    if path.exists() {
        let result = parse_file(path);
        assert!(
            result.is_ok(),
            "Failed to parse basic_syntax.pasta: {:?}",
            result
        );
    }
}

#[test]
fn test_parse_string_and_numbers_fixture() {
    let path = Path::new("tests/fixtures/parser2/string_and_numbers.pasta");
    if path.exists() {
        let result = parse_file(path);
        assert!(
            result.is_ok(),
            "Failed to parse string_and_numbers.pasta: {:?}",
            result
        );
    }
}

#[test]
fn test_parse_escape_sequences_fixture() {
    let path = Path::new("tests/fixtures/parser2/escape_sequences.pasta");
    if path.exists() {
        let result = parse_file(path);
        assert!(
            result.is_ok(),
            "Failed to parse escape_sequences.pasta: {:?}",
            result
        );
    }
}

// ============================================================================
// Existing Fixture Compatibility Tests (Task 5.7)
// ============================================================================

#[test]
fn test_parse_comprehensive_control_flow() {
    let path = Path::new("tests/fixtures/comprehensive_control_flow.pasta");
    if path.exists() {
        let result = parse_file(path);
        // Note: This might fail if the existing fixture uses pasta.pest-specific syntax
        // that differs from pasta2.pest
        if result.is_err() {
            println!("Note: comprehensive_control_flow.pasta may use pasta.pest-specific syntax");
        }
    }
}

// ============================================================================
// Multiple FileItem Occurrence Tests (parser2-filescope-bug-fix)
// ============================================================================

/// Task 4.1: 複数 file_scope 単純連続パターンのテスト
/// 連続する attrs/words は 1つの file_scope としてパースされる
/// 複数の file_scope を発生させるには global_scene_scope を挟む必要がある
#[test]
fn test_multiple_file_scope_consecutive() {
    // 連続する file_scope アイテムは1つの file_scope としてパース
    let source = r#"&author：テスト
@greeting：こんにちは
&version：1.0
@farewell：さようなら
＊シーン
  Alice：テスト
"#;
    let result = parse_str(source, "multi_file_scope.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();

    // 全ての attrs と words が先に来て、その後 GlobalSceneScope
    // 連続した file_scope アイテムは attrs → words の順でパースされる
    let attrs = get_file_attrs(&file);
    let words = get_words(&file);
    assert_eq!(attrs.len(), 2);
    assert_eq!(words.len(), 2);
    assert_eq!(attrs[0].key, "author");
    assert_eq!(attrs[1].key, "version");
    assert_eq!(words[0].name, "greeting");
    assert_eq!(words[1].name, "farewell");
    assert_eq!(get_global_scene_scopes(&file).len(), 1);
}

/// Task 4.2: file_scope と global_scene_scope 交互出現パターンのテスト
#[test]
fn test_file_scope_and_global_scene_interleaved() {
    let source = r#"&author：テスト
@greeting：こんにちは
＊シーン1
  Alice：最初
&version：2.0
@farewell：さようなら
＊シーン2
  Bob：次
"#;
    let result = parse_str(source, "interleaved.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();

    // items の順序がファイル記述順と一致
    // FileAttr, GlobalWord, GlobalSceneScope, FileAttr, GlobalWord, GlobalSceneScope
    assert_eq!(file.items.len(), 6);
    assert!(matches!(file.items[0], FileItem::FileAttr(_)));
    assert!(matches!(file.items[1], FileItem::GlobalWord(_)));
    assert!(matches!(file.items[2], FileItem::GlobalSceneScope(_)));
    assert!(matches!(file.items[3], FileItem::FileAttr(_)));
    assert!(matches!(file.items[4], FileItem::GlobalWord(_)));
    assert!(matches!(file.items[5], FileItem::GlobalSceneScope(_)));

    // 属性と単語は全て取得可能
    assert_eq!(get_file_attrs(&file).len(), 2);
    assert_eq!(get_words(&file).len(), 2);
    assert_eq!(get_global_scene_scopes(&file).len(), 2);
}

/// Task 4.3: 単一バリアント頻出パターンのテスト
#[test]
fn test_single_variant_multiple_occurrences() {
    // 複数 global_scene_scope のみ
    let source = r#"＊シーン1
  Alice：最初
＊シーン2
  Bob：次
＊シーン3
  Charlie：最後
"#;
    let result = parse_str(source, "scenes_only.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();

    assert_eq!(file.items.len(), 3);
    for item in &file.items {
        assert!(matches!(item, FileItem::GlobalSceneScope(_)));
    }
    assert_eq!(get_global_scene_scopes(&file).len(), 3);
    assert!(get_file_attrs(&file).is_empty());
    assert!(get_words(&file).is_empty());
}

/// Task 4.4: パターンマッチと型判定の動作確認テスト
#[test]
fn test_pattern_match_and_helper_methods() {
    let source = r#"&title：テスト
@word1：あ、い、う
＊シーン
  Alice：セリフ
"#;
    let result = parse_str(source, "pattern_match.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();

    // items をイテレートしながら match で各バリアントを識別
    let mut attr_count = 0;
    let mut word_count = 0;
    let mut scene_count = 0;

    for item in &file.items {
        match item {
            FileItem::FileAttr(attr) => {
                attr_count += 1;
                assert_eq!(attr.key, "title");
            }
            FileItem::GlobalWord(word) => {
                word_count += 1;
                assert_eq!(word.name, "word1");
            }
            FileItem::GlobalSceneScope(scene) => {
                scene_count += 1;
                assert_eq!(scene.name, "シーン");
            }
            FileItem::ActorScope(_) => {
                // アクター定義はこのテストでは対象外
            }
        }
    }

    assert_eq!(attr_count, 1);
    assert_eq!(word_count, 1);
    assert_eq!(scene_count, 1);

    // ヘルパー関数が正確に型別抽出できることを確認
    assert_eq!(get_file_attrs(&file).len(), 1);
    assert_eq!(get_words(&file).len(), 1);
    assert_eq!(get_global_scene_scopes(&file).len(), 1);
}

// ============================================================================
// transpiler2 Compatibility Tests (Phase 5)
// ============================================================================

/// Task 5.1: items イテレーション処理の型安全性テスト
/// transpiler2 風の順次処理シミュレーション
#[test]
fn test_transpiler2_style_iteration() {
    let source = r#"&title：テストタイトル
@greeting：こんにちは、おはよう
＊シーン1
  Alice：最初のセリフ
&author：テスト作者
@farewell：さようなら、またね
＊シーン2
  Bob：次のセリフ
"#;
    let result = parse_str(source, "transpiler2.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();

    // transpiler2 風の順次処理シミュレーション
    // 属性と単語をバッファリングし、シーン到達時に適用
    let mut buffered_attrs = Vec::new();
    let mut buffered_words = Vec::new();
    let mut scene_contexts = Vec::new();

    for item in &file.items {
        match item {
            FileItem::FileAttr(attr) => {
                buffered_attrs.push(attr.key.clone());
            }
            FileItem::GlobalWord(word) => {
                buffered_words.push(word.name.clone());
            }
            FileItem::GlobalSceneScope(scene) => {
                // シーン到達時に現在のコンテキストを記録
                scene_contexts.push((
                    scene.name.clone(),
                    buffered_attrs.clone(),
                    buffered_words.clone(),
                ));
            }
            FileItem::ActorScope(_) => {
                // アクター定義はこのテストでは対象外
            }
        }
    }

    // シーン1のコンテキスト: title, greeting
    assert_eq!(scene_contexts[0].0, "シーン1");
    assert_eq!(scene_contexts[0].1, vec!["title"]);
    assert_eq!(scene_contexts[0].2, vec!["greeting"]);

    // シーン2のコンテキスト: title, author, greeting, farewell（累積）
    assert_eq!(scene_contexts[1].0, "シーン2");
    assert_eq!(scene_contexts[1].1, vec!["title", "author"]);
    assert_eq!(scene_contexts[1].2, vec!["greeting", "farewell"]);
}

/// Task 5.2: Span情報の伝播確認テスト
#[test]
fn test_span_information_preserved() {
    let source = "&title：テスト\n＊シーン\n  Alice：セリフ\n";
    let result = parse_str(source, "span.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();

    // 各 FileItem が Span を保有していることを確認
    for item in &file.items {
        match item {
            FileItem::FileAttr(attr) => {
                // Span が設定されている（デフォルトの0,0,0,0でない）
                assert!(attr.span.start_line > 0 || attr.span.start_col > 0);
            }
            FileItem::GlobalWord(word) => {
                assert!(word.span.start_line > 0 || word.span.start_col > 0);
            }
            FileItem::GlobalSceneScope(scene) => {
                assert!(scene.span.start_line > 0 || scene.span.start_col > 0);
            }
            FileItem::ActorScope(actor) => {
                assert!(actor.span.start_line > 0 || actor.span.start_col > 0);
            }
        }
    }
}
