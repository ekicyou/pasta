//! パーサーモジュールの外部テスト
//!
//! Phase A: parser/mod.rs のインラインテストから外部化

use pasta_dsl::ParseError;
use pasta_dsl::parser::*;

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

#[test]
fn test_parse_empty_file() {
    let result = parse_str("", "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    assert!(get_global_scene_scopes(&file).is_empty());
}

#[test]
fn test_parse_simple_global_scene() {
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0].name, "挨拶");
    assert!(!scenes[0].is_continuation);
}

#[test]
fn test_parse_continuation_scene() {
    let source = "＊挨拶\n  Alice：こんにちは\n＊\n  Bob：やあ\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 2);
    assert_eq!(scenes[0].name, "挨拶");
    assert!(!scenes[0].is_continuation);
    assert_eq!(scenes[1].name, "挨拶");
    assert!(scenes[1].is_continuation);
}

#[test]
fn test_parse_unnamed_scene_at_start_error() {
    let source = "＊\n  Alice：こんにちは\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_err());
    if let Err(ParseError::SyntaxError { message, .. }) = result {
        assert!(message.contains("Unnamed global scene at start of file"));
    }
}

#[test]
fn test_parse_file_scope() {
    let source = "&author：テスト\n＊挨拶\n  Alice：こんにちは\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let attrs = get_file_attrs(&file);
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].key, "author");
}

#[test]
fn test_parse_continue_action_line() {
    let source = "＊挨拶\n  Alice：こんにちは\n  ：続きの台詞\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 1);
    let local_scene = &scenes[0].local_scenes[0];
    assert!(local_scene.items.len() >= 2);
    // First item should be ActionLine, second should be ContinueAction
    assert!(matches!(
        local_scene.items[0],
        LocalSceneItem::ActionLine(_)
    ));
    assert!(matches!(
        local_scene.items[1],
        LocalSceneItem::ContinueAction(_)
    ));
}

#[test]
fn test_parse_code_block() {
    let source = "＊挨拶\n  Alice：こんにちは\n```rune\nlet x = 1;\n```\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    assert_eq!(get_global_scene_scopes(&file).len(), 1);
    // Code blocks may be in global or local scope depending on grammar
}

// ========================================================================
// SceneActorItem Tests (scene-actors-ast-support)
// ========================================================================

#[test]
fn test_parse_scene_actors_single() {
    // 単一アクター「さくら」（番号0）
    let source = "＊挨拶\n　％さくら\n  さくら：こんにちは\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0].actors.len(), 1);
    assert_eq!(scenes[0].actors[0].name, "さくら");
    assert_eq!(scenes[0].actors[0].number, 0);
}

#[test]
fn test_parse_scene_actors_with_explicit_number() {
    // 「さくら」（番号0）と「うにゅう＝２」
    let source = "＊挨拶\n　％さくら、うにゅう＝２\n  さくら：こんにちは\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0].actors.len(), 2);
    assert_eq!(scenes[0].actors[0].name, "さくら");
    assert_eq!(scenes[0].actors[0].number, 0);
    assert_eq!(scenes[0].actors[1].name, "うにゅう");
    assert_eq!(scenes[0].actors[1].number, 2);
}

#[test]
fn test_parse_scene_actors_csharp_enum_numbering() {
    // C#のenum採番ルール: さくら=0, うにゅう=2, まりか=3
    let source = "＊挨拶\n　％さくら、うにゅう＝２、まりか\n  さくら：こんにちは\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0].actors.len(), 3);
    assert_eq!(scenes[0].actors[0].name, "さくら");
    assert_eq!(scenes[0].actors[0].number, 0);
    assert_eq!(scenes[0].actors[1].name, "うにゅう");
    assert_eq!(scenes[0].actors[1].number, 2);
    assert_eq!(scenes[0].actors[2].name, "まりか");
    assert_eq!(scenes[0].actors[2].number, 3);
}

#[test]
fn test_parse_scene_actors_fullwidth_number() {
    // 全角数字「＝１０」が正しくパースされることを確認
    let source = "＊挨拶\n　％さくら＝１０\n  さくら：こんにちは\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0].actors.len(), 1);
    assert_eq!(scenes[0].actors[0].name, "さくら");
    assert_eq!(scenes[0].actors[0].number, 10);
}

#[test]
fn test_parse_scene_actors_u32_max_explicit_number_no_panic() {
    // ハードニング回帰テスト: 明示番号 u32::MAX（4294967295）で next_number 計算
    // （n + 1）が整数オーバーフローで panic しない境界を固定する（飽和加算）
    let source = "＊挨拶\n　％さくら＝4294967295\n  さくら：こんにちは\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0].actors.len(), 1);
    assert_eq!(scenes[0].actors[0].number, u32::MAX);
}

#[test]
fn test_parse_scene_actors_u32_max_then_implicit_no_panic() {
    // ハードニング回帰テスト: u32::MAX 指定後の暗黙採番（next_number += 1）が
    // panic せず u32::MAX で飽和する境界を固定する
    let source = "＊挨拶\n　％さくら＝4294967295、うにゅう\n  さくら：こんにちは\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0].actors.len(), 2);
    assert_eq!(scenes[0].actors[0].number, u32::MAX);
    // 飽和後の暗黙採番も u32::MAX に張り付く
    assert_eq!(scenes[0].actors[1].number, u32::MAX);
}

#[test]
fn test_parse_scene_actors_multiple_lines() {
    // 複数行のアクター宣言で番号が行をまたいで引き継がれることを確認
    // 1行目: さくら=0
    // 2行目: うにゅう=5
    // 3行目: まりか=6
    let source = "＊挨拶\n　％さくら\n　％うにゅう＝５\n　％まりか\n  さくら：こんにちは\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0].actors.len(), 3);
    assert_eq!(scenes[0].actors[0].name, "さくら");
    assert_eq!(scenes[0].actors[0].number, 0);
    assert_eq!(scenes[0].actors[1].name, "うにゅう");
    assert_eq!(scenes[0].actors[1].number, 5);
    assert_eq!(scenes[0].actors[2].name, "まりか");
    assert_eq!(scenes[0].actors[2].number, 6);
}

#[test]
fn test_parse_scene_actors_complex_numbering() {
    // 複雑な採番パターン: さくら=0, うにゅう=2, まりか=3, ゆかり=10, あかね=11
    let source =
        "＊挨拶\n　％さくら、うにゅう＝２、まりか、ゆかり＝１０、あかね\n  さくら：こんにちは\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0].actors.len(), 5);
    assert_eq!(scenes[0].actors[0].name, "さくら");
    assert_eq!(scenes[0].actors[0].number, 0);
    assert_eq!(scenes[0].actors[1].name, "うにゅう");
    assert_eq!(scenes[0].actors[1].number, 2);
    assert_eq!(scenes[0].actors[2].name, "まりか");
    assert_eq!(scenes[0].actors[2].number, 3);
    assert_eq!(scenes[0].actors[3].name, "ゆかり");
    assert_eq!(scenes[0].actors[3].number, 10);
    assert_eq!(scenes[0].actors[4].name, "あかね");
    assert_eq!(scenes[0].actors[4].number, 11);
}

#[test]
fn test_parse_scene_actors_span_valid() {
    // SceneActorItemのSpanが有効であることを確認
    let source = "＊挨拶\n　％さくら\n  さくら：こんにちは\n";
    let result = parse_str(source, "test.pasta");
    assert!(result.is_ok());
    let file = result.unwrap();
    let scenes = get_global_scene_scopes(&file);
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0].actors.len(), 1);
    // Spanが有効であることを確認
    assert!(scenes[0].actors[0].span.is_valid());
}
