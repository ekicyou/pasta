//! 入力のたびの構文の診断（要件 1・2.3・11.1）
//!
//! 全体の解析の誤り 1 件を、保存時の検査と同じ文で、本当の位置の点にする。

use std::collections::BTreeMap;

use pasta_check::validate::{FileText, FindingKind, GhostInput, TomlSource, validate_ghost};
use pasta_lsp::analysis::{AnalysisEngine, Diagnostic};

/// 診断がちょうど 1 件で、(行, 桁) の幅の無い点・Error・文が `message` であること
fn assert_one(source: &str, line: u32, character: u32, message: &str) -> Diagnostic {
    let result = AnalysisEngine::analyze(source);
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    let d = result.diagnostics[0].clone();
    assert_eq!(
        (d.range.start.line, d.range.start.character),
        (line, character),
        "{d:?}"
    );
    assert_eq!(d.range.start, d.range.end, "幅の無い点");
    assert_eq!(d.severity, 1);
    assert_eq!(d.message, message);
    d
}

const EXPECTED_ID: &str = "構文エラー: expected id or call_target_expr";

#[test]
fn test_requirement_1_2_five_lines() {
    let source = "＊挨拶\n  さくら：やあ\n  うにゅう：おう\n  ＞＞\n  さくら：ね\n";
    assert_one(source, 3, 3, EXPECTED_ID);
}

#[test]
fn test_crlf() {
    let source = "＊挨拶\r\n  さくら：やあ\r\n  ＞＞\r\n  さくら：ね\r\n";
    assert_one(source, 2, 3, EXPECTED_ID);
}

#[test]
fn test_first_line() {
    assert_one(
        "＞＞\n＊挨拶\n  さくら：やあ\n",
        0,
        0,
        "構文エラー: expected EOI, actor_line, file_scope, or global_scene_start",
    );
}

#[test]
fn test_last_line() {
    assert_one("＊挨拶\n  さくら：やあ\n  ＞＞\n", 2, 3, EXPECTED_ID);
}

#[test]
fn test_last_line_without_newline() {
    assert_one("＊挨拶\n  さくら：やあ\n  ＞＞", 2, 3, EXPECTED_ID);
}

#[test]
fn test_missing_final_newline_is_reported() {
    // 改行を補わない（R3）。保存時と同じく、最後の行の終わりに 1 件
    let d = AnalysisEngine::analyze("＊挨拶\n  さくら：やあ");
    assert_eq!(d.diagnostics.len(), 1, "{:?}", d.diagnostics);
    let p = d.diagnostics[0].range.start;
    assert_eq!((p.line, p.character), (1, 8));
}

#[test]
fn test_surrogate_pair_column() {
    // `𠮷` は UTF-16 で 2 と数える。誤りは 6 文字目（`＝` の後ろ）＝ UTF-16 の 6
    let result = AnalysisEngine::analyze("＊挨拶\n  𠮷𠮷：やあ\n  ＄𠮷＝\n");
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    let p = result.diagnostics[0].range.start;
    assert_eq!((p.line, p.character), (2, 6));
}

#[test]
fn test_unnamed_scene_at_start() {
    assert_one(
        "＊\n  さくら：やあ\n",
        0,
        0,
        "構文エラー: Unnamed global scene at start of file. A named global scene must appear before any unnamed scenes.",
    );
}

#[test]
fn test_valid_source_no_diagnostics() {
    let result = AnalysisEngine::analyze("＊挨拶\n  Alice：こんにちは\n");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_concat_expression_no_diagnostics() {
    // 連結（全角・半角・算術との混在・括弧）を含む正しい式は診断なし。
    let source =
        "＊挨拶\n  ＄ｘ＝「合計」＆＄ｎ＆「個」\n  ＄ｙ＝（1＋2）& 「x」\n  Alice：こんにちは\n";
    let result = AnalysisEngine::analyze(source);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_concat_missing_operand_has_one_diagnostic() {
    // ＆ の右の被演算子が欠けた式は、その行の終わりに 1 件
    let result = AnalysisEngine::analyze("＊挨拶\n  ＄ｘ＝「a」＆\n  Alice：こんにちは\n");
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    let p = result.diagnostics[0].range.start;
    assert_eq!((p.line, p.character), (1, 9));
}

/// 同じ文字を保存時の検査に渡した syntax の報告と、行・桁（UTF-16 に直して）・文が一致する（要件 2.3）
#[test]
fn test_matches_validate_ghost_syntax_finding() {
    let input = GhostInput {
        pasta_toml: TomlSource::Missing,
        entries: vec!["dic/a.pasta".into()],
        dirs: vec!["dic".into()],
    };
    for source in [
        "＊挨拶\n  さくら：やあ\n  うにゅう：おう\n  ＞＞\n  さくら：ね\n",
        "＊挨拶\r\n  さくら：やあ\r\n  ＞＞\r\n  さくら：ね\r\n",
        "＞＞\n＊挨拶\n  さくら：やあ\n",
        "＊挨拶\n  さくら：やあ",
        "＊挨拶\n  𠮷𠮷：やあ\n  ＄𠮷＝\n",
        "＊\n  さくら：やあ\n",
    ] {
        let files = BTreeMap::from([(
            "dic/a.pasta".to_string(),
            FileText::Text {
                text: source.to_string(),
            },
        )]);
        let report = validate_ghost(&input, &files);
        let syntax: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.code == FindingKind::Syntax)
            .collect();
        assert_eq!(syntax.len(), 1, "{source:?}: {:?}", report.findings);
        let f = syntax[0];
        let line = f.line.expect("位置がある") as usize;
        let column = f.column.expect("位置がある") as usize;
        let line_text = source.split('\n').nth(line - 1).unwrap();
        let line_text = line_text.strip_suffix('\r').unwrap_or(line_text);
        let utf16: usize = line_text
            .chars()
            .take(column - 1)
            .map(char::len_utf16)
            .sum();

        let result = AnalysisEngine::analyze(source);
        assert_eq!(result.diagnostics.len(), 1, "{source:?}");
        let d = &result.diagnostics[0];
        assert_eq!(
            (
                d.range.start.line as usize,
                d.range.start.character as usize
            ),
            (line - 1, utf16),
            "{source:?}"
        );
        assert_eq!(d.message, f.message, "{source:?}");
    }
}
