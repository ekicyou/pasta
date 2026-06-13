//! 部分パース機能のユニットテスト (Task 2.5)
//!
//! - Phase 1成功時の完全AST返却テスト
//! - Phase 2スコープ境界分割の正確性テスト
//! - Phase 3行単位フォールバックの正確性テスト
//! - 全角/半角マーカー両対応テスト
//! - PartialParseError生成テスト

use pasta_dsl::parser::Rule;
use pasta_dsl::partial::{PartialParseResult, infer_rule_from_line, parse_str_partial};

// ============================================================================
// Phase 1: Full Parse Success
// ============================================================================

#[test]
fn test_phase1_full_parse_success_returns_complete_ast() {
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result: PartialParseResult = parse_str_partial(source);
    assert!(result.errors.is_empty(), "Phase 1 成功時にエラーなし");
    assert!(!result.items.is_empty(), "Phase 1 成功時にアイテムあり");
}

#[test]
fn test_phase1_multiple_global_scenes() {
    let source = "＊挨拶\n  Alice：こんにちは\n＊別れ\n  Bob：さようなら\n";
    let result = parse_str_partial(source);
    assert!(result.errors.is_empty());
    // 2つのグローバルシーンが返されるはず
    assert!(result.items.len() >= 2, "複数グローバルシーンが返される");
}

// ============================================================================
// Phase 2: Scope Boundary Split
// ============================================================================

#[test]
fn test_phase2_mixed_valid_and_invalid_scopes() {
    // 1つ目のシーンは正常、2つ目のシーンは構文エラー含む
    let source = "＊挨拶\n  Alice：こんにちは\n\n＊壊れたシーン\nこれは構文エラー行ですわ!!!\n";
    let result = parse_str_partial(source);
    // 正常シーンのアイテムか、エラーが返る
    assert!(
        !result.items.is_empty() || !result.errors.is_empty(),
        "混在ソースで何らかの結果が返る"
    );
}

// ============================================================================
// Phase 3: Line-by-Line Fallback
// ============================================================================

#[test]
fn test_phase3_error_lines_generate_partial_errors() {
    // 完全に不正なソース
    let source = "不正な行1\n不正な行2\n";
    let result = parse_str_partial(source);
    // line-by-line fallback でエラーが生成されるはず
    assert!(!result.errors.is_empty(), "Phase 3 でエラー生成");
}

#[test]
fn test_phase3_error_line_numbers_are_1_based() {
    let source = "不正な行\n";
    let result = parse_str_partial(source);
    for err in &result.errors {
        assert!(err.line >= 1, "行番号は1-based: got {}", err.line);
    }
}

#[test]
fn test_phase3_error_message_is_nonempty() {
    let source = "不正な行\n";
    let result = parse_str_partial(source);
    for err in &result.errors {
        assert!(!err.message.is_empty(), "エラーメッセージは空でない");
    }
}

// ============================================================================
// infer_rule_from_line: 全角/半角マーカー両対応
// ============================================================================

#[test]
fn test_infer_rule_global_scene_fullwidth() {
    let rule = infer_rule_from_line("＊挨拶");
    assert_eq!(rule, Some(Rule::global_scene_scope));
}

#[test]
fn test_infer_rule_global_scene_halfwidth() {
    let rule = infer_rule_from_line("*greeting");
    assert_eq!(rule, Some(Rule::global_scene_scope));
}

#[test]
fn test_infer_rule_local_scene_fullwidth() {
    let rule = infer_rule_from_line("・ローカル");
    assert_eq!(rule, Some(Rule::local_scene_line));
}

#[test]
fn test_infer_rule_local_scene_halfwidth() {
    let rule = infer_rule_from_line("-local");
    assert_eq!(rule, Some(Rule::local_scene_line));
}

#[test]
fn test_infer_rule_file_attr_fullwidth() {
    let rule = infer_rule_from_line("＆attr=value");
    assert_eq!(rule, Some(Rule::file_attr_line));
}

#[test]
fn test_infer_rule_file_attr_halfwidth() {
    let rule = infer_rule_from_line("&attr=value");
    assert_eq!(rule, Some(Rule::file_attr_line));
}

#[test]
fn test_infer_rule_word_fullwidth() {
    let rule = infer_rule_from_line("＠単語");
    assert_eq!(rule, Some(Rule::file_word_line));
}

#[test]
fn test_infer_rule_word_halfwidth() {
    let rule = infer_rule_from_line("@word");
    assert_eq!(rule, Some(Rule::file_word_line));
}

#[test]
fn test_infer_rule_actor_fullwidth() {
    let rule = infer_rule_from_line("％アクター");
    assert_eq!(rule, Some(Rule::actor_scope));
}

#[test]
fn test_infer_rule_actor_halfwidth() {
    let rule = infer_rule_from_line("%actor");
    assert_eq!(rule, Some(Rule::actor_scope));
}

#[test]
fn test_infer_rule_var_set_fullwidth() {
    let rule = infer_rule_from_line("＄変数=値");
    assert_eq!(rule, Some(Rule::var_set_line));
}

#[test]
fn test_infer_rule_var_set_halfwidth() {
    let rule = infer_rule_from_line("$var=value");
    assert_eq!(rule, Some(Rule::var_set_line));
}

#[test]
fn test_infer_rule_call_fullwidth() {
    let rule = infer_rule_from_line("＞呼び出し先");
    assert_eq!(rule, Some(Rule::call_scene_line));
}

#[test]
fn test_infer_rule_call_halfwidth() {
    let rule = infer_rule_from_line(">target");
    assert_eq!(rule, Some(Rule::call_scene_line));
}

#[test]
fn test_infer_rule_comment_fullwidth() {
    let rule = infer_rule_from_line("＃コメント");
    assert_eq!(rule, Some(Rule::or_comment_eol));
}

#[test]
fn test_infer_rule_comment_halfwidth() {
    let rule = infer_rule_from_line("#comment");
    assert_eq!(rule, Some(Rule::or_comment_eol));
}

#[test]
fn test_infer_rule_action_line_with_colon() {
    let rule = infer_rule_from_line("Alice：こんにちは");
    assert_eq!(rule, Some(Rule::action_line));
}

#[test]
fn test_infer_rule_action_line_with_halfwidth_colon() {
    let rule = infer_rule_from_line("Bob:hello");
    assert_eq!(rule, Some(Rule::action_line));
}

#[test]
fn test_infer_rule_empty_line_returns_none() {
    assert_eq!(infer_rule_from_line(""), None);
    assert_eq!(infer_rule_from_line("   "), None);
}

#[test]
fn test_infer_rule_code_block() {
    let rule = infer_rule_from_line("```lua");
    assert_eq!(rule, Some(Rule::code_scope));
}

// ============================================================================
// parse_str_partial: Empty / Edge Cases
// ============================================================================

#[test]
fn test_partial_parse_empty_source() {
    let result = parse_str_partial("");
    assert!(result.items.is_empty());
    assert!(result.errors.is_empty());
}

#[test]
fn test_partial_parse_whitespace_only() {
    let result = parse_str_partial("   \n  \n");
    // Empty lines should be silently skipped
    assert!(result.errors.is_empty());
}

// ============================================================================
// Phase 2: Span position correction (regression for chunk-relative spans)
// ============================================================================

use pasta_dsl::parser::{ActionLine, FileItem, LocalSceneItem};

/// GlobalSceneScope 内の最初の ActionLine を取り出すヘルパー
fn first_action_line(items: &[FileItem]) -> Option<&ActionLine> {
    for item in items {
        if let FileItem::GlobalSceneScope(scene) = item {
            for ls in &scene.local_scenes {
                for it in &ls.items {
                    if let LocalSceneItem::ActionLine(al) = it {
                        return Some(al);
                    }
                }
            }
        }
    }
    None
}

#[test]
fn test_phase2_span_lines_corrected_to_full_source() {
    // 1 行目が不正なため Phase 1 全体パースは失敗し、Phase 2 のチャンク分割に落ちる。
    // 2 つ目以降のスコープの span がチャンク相対のままだと行番号がズレる。
    let source = "これは不正な冒頭行\n＊挨拶\n  Alice：こんにちは\n％さくら\n  ＄表情＝0\n";
    let result = parse_str_partial(source);

    // GlobalSceneScope: ＊挨拶 は元ソース 2 行目、Alice 行は 3 行目
    let scene = result
        .items
        .iter()
        .find_map(|it| match it {
            FileItem::GlobalSceneScope(s) => Some(s),
            _ => None,
        })
        .expect("GlobalSceneScope が部分パースで得られる");
    assert_eq!(scene.span.start_line, 2, "シーン先頭はフルソース2行目");

    let action = first_action_line(&result.items).expect("ActionLine が得られる");
    assert_eq!(action.span.start_line, 3, "アクション行はフルソース3行目");

    // ActorScope: ％さくら は 4 行目、＄表情＝0（var_set）は 5 行目
    let actor = result
        .items
        .iter()
        .find_map(|it| match it {
            FileItem::ActorScope(a) => Some(a),
            _ => None,
        })
        .expect("ActorScope が部分パースで得られる");
    assert_eq!(actor.span.start_line, 4, "アクタースコープ先頭は4行目");
    assert_eq!(
        actor.var_sets[0].span.start_line, 5,
        "2つ目のスコープの var_set は5行目（チャンク相対の2行目ではない）"
    );
}

#[test]
fn test_phase2_span_byte_offsets_extract_correct_source() {
    // バイトオフセットもフルソース座標へ補正されること（extract_source で検証）。
    let source = "これは不正な冒頭行\n＊挨拶\n  Alice：こんにちは\n％さくら\n  ＄表情＝0\n";
    let result = parse_str_partial(source);

    let action = first_action_line(&result.items).expect("ActionLine が得られる");
    let extracted = action
        .span
        .extract_source(source)
        .expect("補正後 span はフルソースの正しい範囲を指す");
    assert!(
        extracted.contains("Alice"),
        "抽出結果に Alice が含まれる: {:?}",
        extracted
    );
    assert!(
        !extracted.contains("挨拶"),
        "抽出結果が前のチャンクへずれていない: {:?}",
        extracted
    );
}

#[test]
fn test_phase2_span_correction_handles_crlf() {
    // CRLF ソース: チャンクが LF 再結合で作られていると 1 バイト/行ずれるが、
    // 元ソースの部分スライスを使うことで extract_source が正しく機能する。
    let source = "bad line\r\n＊scene\r\n  Alice：hi\r\n";
    let result = parse_str_partial(source);

    let action = first_action_line(&result.items).expect("ActionLine が得られる (CRLF)");
    assert_eq!(action.span.start_line, 3, "CRLF でもアクション行は3行目");
    let extracted = action
        .span
        .extract_source(source)
        .expect("CRLF でも extract_source が成功する");
    assert!(
        extracted.contains("Alice"),
        "CRLF: 抽出結果に Alice が含まれる: {:?}",
        extracted
    );
}
