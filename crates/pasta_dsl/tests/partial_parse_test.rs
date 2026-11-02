//! 部分解析のテスト
//!
//! - 区切りは字下げする前の行頭の 1 文字で決める（字下げした行・名前の無い `＊` は区切らない）
//! - 解けない塊は行の範囲（1 始まり・半開区間）だけを `failed_lines` に積む
//! - 解けた塊の span は元のソースの座標に直す

use pasta_dsl::parser::GlobalSceneScope;
use pasta_dsl::partial::{PartialParseResult, parse_str_partial};

fn scenes(result: &PartialParseResult) -> Vec<&GlobalSceneScope> {
    result
        .items
        .iter()
        .filter_map(|it| match it {
            FileItem::GlobalSceneScope(s) => Some(s),
            _ => None,
        })
        .collect()
}

// ============================================================================
// 正しいソース
// ============================================================================

#[test]
fn test_valid_source_returns_complete_ast() {
    let source = "＊挨拶\n  Alice：こんにちは\n";
    let result = parse_str_partial(source);
    assert!(result.failed_lines.is_empty());
    assert!(!result.items.is_empty());
}

#[test]
fn test_valid_multiple_global_scenes() {
    let source = "＊挨拶\n  Alice：こんにちは\n＊別れ\n  Bob：さようなら\n";
    let result = parse_str_partial(source);
    assert!(result.failed_lines.is_empty());
    assert_eq!(scenes(&result).len(), 2);
}

// ============================================================================
// 区切り（要件 3.2）
// ============================================================================

/// 字下げしたアクター指定・単語定義・選択肢・属性の行を含むシーンは割れない
#[test]
fn test_indented_markers_do_not_split_scene() {
    let source = concat!(
        "＊挨拶\n",
        "　＆朝：1\n",
        "　％さくら、うにゅう\n",
        "　＠挨拶：こんにちは\n",
        "　さくら：＠挨拶\n",
        "　＠？はい「はい」\n",
        "＊壊れ\n",
        "　＄＝\n",
    );
    let result = parse_str_partial(source);
    let found = scenes(&result);
    assert_eq!(found.len(), 1, "シーンは 1 つのまま: {:?}", result.items);
    let scene = found[0];
    assert_eq!(scene.name, "挨拶");
    assert_eq!(scene.attrs.len(), 1, "属性はシーンに残る");
    assert_eq!(scene.actors.len(), 2, "アクターはシーンに残る");
    assert_eq!(scene.words.len(), 1, "単語はシーンに残る");
    assert_eq!(result.failed_lines, vec![7..9]);
}

/// 誤りのシーンの行だけが失敗の範囲に入る（要件 3.3）
#[test]
fn test_only_error_scene_lines_fail() {
    let source = concat!(
        "＊挨拶\n",
        "　さくら：こんにちは\n",
        "＊壊れ\n",
        "　＄＝\n",
        "　さくら：まだ壊れたシーン\n",
        "％さくら\n",
        "　＄表情＝0\n",
        "＊別れ\n",
        "　さくら：さようなら\n",
    );
    let result = parse_str_partial(source);
    assert_eq!(result.failed_lines, vec![3..6]);
    let names: Vec<_> = scenes(&result).iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["挨拶", "別れ"]);
    assert!(
        result
            .items
            .iter()
            .any(|it| matches!(it, FileItem::ActorScope(_))),
        "誤りの後ろのアクター辞書は解ける"
    );
}

/// 名前の無い `＊` の行は前のシーンと一緒に解ける（D2）
#[test]
fn test_lone_star_parses_with_previous_scene() {
    let source = concat!(
        "＊挨拶\n",
        "　さくら：A\n",
        "＊　＃続き\n",
        "　さくら：B\n",
        "＊壊れ\n",
        "　＄＝\n",
    );
    let result = parse_str_partial(source);
    let found = scenes(&result);
    assert_eq!(found.len(), 2, "名前付きと続きの 2 つ: {:?}", result.items);
    assert!(found[1].is_continuation);
    assert_eq!(found[1].span.start_line, 3);
    assert_eq!(result.failed_lines, vec![5..7]);
}

/// 改行で終わらない最後の塊も解ける（色を残すために改行を補う）
#[test]
fn test_last_chunk_without_trailing_newline_parses() {
    let source = "＊壊れ\n　＄＝\n＊挨拶\n　さくら：こんにちは";
    let result = parse_str_partial(source);
    assert_eq!(result.failed_lines, vec![1..3]);
    let found = scenes(&result);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].span.start_line, 3);
}

// ============================================================================
// 解けない塊
// ============================================================================

#[test]
fn test_unparsable_chunk_is_one_line_range() {
    let source = "不正な行1\n不正な行2\n";
    let result = parse_str_partial(source);
    assert!(result.items.is_empty());
    assert_eq!(result.failed_lines, vec![1..3]);
}

#[test]
fn test_unparsable_chunk_without_trailing_newline() {
    let result = parse_str_partial("不正な行");
    assert_eq!(result.failed_lines, vec![1..2]);
}

// ============================================================================
// 空・空白だけ
// ============================================================================

#[test]
fn test_partial_parse_empty_source() {
    let result = parse_str_partial("");
    assert!(result.items.is_empty());
    assert!(result.failed_lines.is_empty());
}

#[test]
fn test_partial_parse_whitespace_only() {
    let result = parse_str_partial("   \n  \n");
    assert!(result.failed_lines.is_empty());
}

// ============================================================================
// 塊の span を元のソースの座標に直す
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
fn test_chunk_span_lines_corrected_to_full_source() {
    // 1 行目が不正な塊になり、後ろの塊は別に解かれる。
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
fn test_chunk_span_byte_offsets_extract_correct_source() {
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
fn test_chunk_span_correction_handles_crlf() {
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

#[test]
fn test_chunk_span_correction_for_dynamic_refs() {
    // 部分パース後も動的参照の span（関数呼び出しは引数の span も）がフルソース座標を指す。
    use pasta_dsl::parser::Action;
    let source = "これは不正な冒頭行\n＊挨拶\n  Alice：＠＄x　と＠＄＊f（１）\n";
    let result = parse_str_partial(source);

    let action = first_action_line(&result.items).expect("ActionLine が得られる");
    match &action.actions[..] {
        [
            Action::DynamicWordRef { span: word, .. },
            Action::Talk { .. },
            Action::DynamicFnCall {
                span: call, args, ..
            },
        ] => {
            assert_eq!(word.start_line, 3);
            assert_eq!(word.extract_source(source).unwrap(), "＠＄x　");
            assert_eq!(call.start_line, 3);
            assert_eq!(call.extract_source(source).unwrap(), "＠＄＊f（１）");
            assert_eq!(args.span.extract_source(source).unwrap(), "（１）");
        }
        other => panic!("動的参照 2 つと台詞が得られる: {other:?}"),
    }
}
