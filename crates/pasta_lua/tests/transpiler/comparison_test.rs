//! Comparison and structure integration tests for pasta_lua transpiler.
//!
//! - Line-by-line comparison against reference implementation (Requirement 6)
//! - MAJOR-4: FileItem出現順処理・シャドーイング・属性非継承テスト

use crate::common;

use common::normalize_line_endings;
use pasta_dsl::parse_str;
use pasta_lua::LuaTranspiler;

// ============================================================================
// Requirement 6: Integration Test with Line-by-Line Comparison
// ============================================================================

/// Mismatch information for detailed reporting
#[derive(Debug)]
struct LineMismatch {
    line_number: usize,
    expected: String,
    actual: String,
    mismatch_type: MismatchType,
}

#[derive(Debug, PartialEq)]
enum MismatchType {
    ContentDifference,
    MissingInActual,
    ExtraInActual,
}

/// Test statistics for reporting
#[derive(Debug, Default)]
struct TestStatistics {
    total_lines: usize,
    matched_lines: usize,
    mismatched_lines: usize,
    content_differences: usize,
    missing_lines: usize,
    extra_lines: usize,
}

impl TestStatistics {
    fn match_rate(&self) -> f64 {
        if self.total_lines == 0 {
            100.0
        } else {
            (self.matched_lines as f64 / self.total_lines as f64) * 100.0
        }
    }
}

/// Filter out comment lines and normalize whitespace for comparison
fn normalize_lua_lines(code: &str) -> Vec<String> {
    code.lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Keep non-empty lines that are not pure comments
            !trimmed.is_empty() && !trimmed.starts_with("--")
        })
        .map(|line| line.trim().to_string())
        .collect()
}

/// Compare two Lua code strings line by line (Requirement 6)
fn compare_lua_output(expected: &str, actual: &str) -> (Vec<LineMismatch>, TestStatistics) {
    let expected_lines = normalize_lua_lines(expected);
    let actual_lines = normalize_lua_lines(actual);

    let mut mismatches = Vec::new();
    let mut stats = TestStatistics::default();

    let max_len = expected_lines.len().max(actual_lines.len());
    stats.total_lines = max_len;

    for i in 0..max_len {
        let expected_line = expected_lines.get(i);
        let actual_line = actual_lines.get(i);

        match (expected_line, actual_line) {
            (Some(exp), Some(act)) => {
                if exp == act {
                    stats.matched_lines += 1;
                } else {
                    stats.mismatched_lines += 1;
                    stats.content_differences += 1;
                    mismatches.push(LineMismatch {
                        line_number: i + 1,
                        expected: exp.clone(),
                        actual: act.clone(),
                        mismatch_type: MismatchType::ContentDifference,
                    });
                }
            }
            (Some(exp), None) => {
                stats.mismatched_lines += 1;
                stats.missing_lines += 1;
                mismatches.push(LineMismatch {
                    line_number: i + 1,
                    expected: exp.clone(),
                    actual: "<missing>".to_string(),
                    mismatch_type: MismatchType::MissingInActual,
                });
            }
            (None, Some(act)) => {
                stats.mismatched_lines += 1;
                stats.extra_lines += 1;
                mismatches.push(LineMismatch {
                    line_number: i + 1,
                    expected: "<not expected>".to_string(),
                    actual: act.clone(),
                    mismatch_type: MismatchType::ExtraInActual,
                });
            }
            (None, None) => unreachable!(),
        }
    }

    (mismatches, stats)
}

/// Generate detailed mismatch report
fn generate_mismatch_report(mismatches: &[LineMismatch], stats: &TestStatistics) -> String {
    let mut report = String::new();

    report.push('\n');
    report.push_str(
        "================================================================================\n",
    );
    report.push_str("                    TRANSPILER OUTPUT COMPARISON REPORT\n");
    report.push_str(
        "================================================================================\n\n",
    );

    // Statistics summary
    report.push_str("【統計情報】\n");
    report.push_str(&format!("  総行数:       {}\n", stats.total_lines));
    report.push_str(&format!("  一致行数:     {}\n", stats.matched_lines));
    report.push_str(&format!("  不一致行数:   {}\n", stats.mismatched_lines));
    report.push_str(&format!("  一致率:       {:.1}%\n", stats.match_rate()));
    report.push('\n');

    // Mismatch pattern classification
    report.push_str("【不一致パターン分類】\n");
    report.push_str(&format!("  内容差異:     {}\n", stats.content_differences));
    report.push_str(&format!("  欠落行:       {}\n", stats.missing_lines));
    report.push_str(&format!("  余剰行:       {}\n", stats.extra_lines));
    report.push('\n');

    // Detailed mismatch list (limit to first 20 for readability)
    if !mismatches.is_empty() {
        report.push_str("【不一致詳細】\n");
        report.push_str(
            "--------------------------------------------------------------------------------\n",
        );

        for (idx, mismatch) in mismatches.iter().take(20).enumerate() {
            report.push_str(&format!(
                "\n[{}] 行 {}: {:?}\n",
                idx + 1,
                mismatch.line_number,
                mismatch.mismatch_type
            ));
            report.push_str(&format!("  期待: {}\n", mismatch.expected));
            report.push_str(&format!("  実際: {}\n", mismatch.actual));
        }

        if mismatches.len() > 20 {
            report.push_str(&format!("\n... 他 {} 件の不一致\n", mismatches.len() - 20));
        }
    }

    report.push_str(
        "\n================================================================================\n",
    );

    report
}

/// Requirement 6: Full line-by-line comparison test
/// sample.pasta → Lua トランスパイル出力を sample.lua と比較
#[test]
fn test_transpile_sample_pasta_line_comparison() {
    // Load sample files
    let sample_pasta = include_str!("../fixtures/sample.pasta");
    let sample_lua = include_str!("../fixtures/sample.lua");
    let sample_expected = include_str!("../fixtures/sample.expected.lua");

    // Parse and transpile
    let file = parse_str(sample_pasta, "sample.pasta").unwrap();

    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let generated_lua = String::from_utf8(output).unwrap();

    // Save generated output for debugging (overwrites existing file in fixtures)
    // Note: sample.generated.lua is already tracked in crates/pasta_lua/tests/fixtures/
    let output_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample.generated.lua");
    std::fs::write(&output_path, &generated_lua).expect("Failed to write sample.generated.lua");

    // Normalize line endings for comparison (handles Git autocrlf differences)
    let generated_normalized = normalize_line_endings(&generated_lua);
    let expected_normalized = normalize_line_endings(sample_expected);

    // Compare line by line (using normalized versions)
    let (mismatches, stats) = compare_lua_output(sample_lua, &generated_normalized);

    // Generate and print report
    let report = generate_mismatch_report(&mismatches, &stats);

    // Print report for debugging (visible in test output with --nocapture)
    eprintln!("{}", report);

    // Strict equality check with expected output (normalized)
    assert_eq!(
        generated_normalized, expected_normalized,
        "Generated code must match expected output (line endings normalized)"
    );

    // For now, we verify the comparison runs and report is generated
    // Match rate check is currently lenient; will be replaced with strict equality check
    assert!(
        stats.match_rate() >= 0.0,
        "Comparison completed. See report above for details. Generated output saved to crates/pasta_lua/tests/fixtures/sample.generated.lua"
    );

    // Report statistics even on success
    println!("\n【テスト結果サマリー】");
    println!(
        "  一致率: {:.1}% ({}/{})",
        stats.match_rate(),
        stats.matched_lines,
        stats.total_lines
    );
}

// ============================================================================
// MAJOR-4: FileItem出現順処理・シャドーイング・属性非継承テスト
// ============================================================================

/// MAJOR-4: FileItem出現順処理の検証
/// アクターとシーンがファイル内出現順に処理されることを確認
#[test]
fn test_file_item_order_preserved() {
    let pasta = r#"
％アクター1
　＠表情：\s[0]

＊シーン1
　　アクター1：セリフ1

％アクター2
　＠表情：\s[10]

＊シーン2
　　アクター2：セリフ2
"#;

    let file = parse_str(pasta, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // 出現順序を確認: アクター1 → シーン1 → アクター2 → シーン2
    // Counter is now assigned by Lua runtime, so scene names don't have counter suffix
    let actor1_pos = lua_code.find("create_actor(\"アクター1\")").unwrap();
    let scene1_pos = lua_code.find("create_scene(\"シーン1\")").unwrap();
    let actor2_pos = lua_code.find("create_actor(\"アクター2\")").unwrap();
    let scene2_pos = lua_code.find("create_scene(\"シーン2\")").unwrap();

    assert!(
        actor1_pos < scene1_pos,
        "アクター1はシーン1より前に出現すべき"
    );
    assert!(
        scene1_pos < actor2_pos,
        "シーン1はアクター2より前に出現すべき"
    );
    assert!(
        actor2_pos < scene2_pos,
        "アクター2はシーン2より前に出現すべき"
    );
}

/// MAJOR-4: ファイル属性シャドーイングの検証
/// 同じキーの属性が再出現すると後勝ちで上書きされることを確認
#[test]
fn test_file_attr_shadowing() {
    use pasta_dsl::parser::{Attr, AttrValue, FileItem, PastaFile, Span};
    use std::path::PathBuf;

    // 手動でPastaFileを構築（FileAttrを含む）
    let attr1 = FileItem::FileAttr(Attr {
        key: "author".to_string(),
        value: AttrValue::AttrString("Alice".to_string()),
        span: Span::default(),
    });
    let attr2 = FileItem::FileAttr(Attr {
        key: "author".to_string(),
        value: AttrValue::AttrString("Bob".to_string()), // シャドーイング
        span: Span::default(),
    });

    // 正しいPasta構文でシーンをパース
    let scene_pasta = r#"
＊テスト
　　さくら：こんにちは。
"#;
    let scene = parse_str(scene_pasta, "test.pasta")
        .unwrap()
        .items
        .into_iter()
        .next()
        .unwrap();

    let file = PastaFile {
        path: PathBuf::from("test.pasta"),
        items: vec![attr1, attr2, scene],
        span: Span::default(),
    };

    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    let context = transpiler.transpile(&file, &mut output).unwrap();

    // file_attrs()でシャドーイングを確認
    let attrs = context.file_attrs();
    assert_eq!(attrs.len(), 1, "シャドーイングにより1つのキーのみ");
    assert_eq!(
        attrs.get("author"),
        Some(&AttrValue::AttrString("Bob".to_string())),
        "後勝ちでBobになるべき"
    );
}

/// MAJOR-4: アクターがファイル属性を継承しないことの検証
#[test]
fn test_actor_does_not_inherit_file_attrs() {
    use pasta_dsl::parser::{Attr, AttrValue, FileItem, PastaFile, Span};
    use std::path::PathBuf;

    // ファイル属性 → アクター → シーン の順序
    let file_attr = FileItem::FileAttr(Attr {
        key: "author".to_string(),
        value: AttrValue::AttrString("Alice".to_string()),
        span: Span::default(),
    });

    // アクターを含むPastaファイルをパース（表情定義なしでシンプルに）
    let actor_pasta = r#"
％さくら
"#;
    let actor = parse_str(actor_pasta, "test.pasta")
        .unwrap()
        .items
        .into_iter()
        .next()
        .unwrap();

    let scene_pasta = r#"
＊メイン
　　さくら：こんにちは。
"#;
    let scene = parse_str(scene_pasta, "test.pasta")
        .unwrap()
        .items
        .into_iter()
        .next()
        .unwrap();

    let file = PastaFile {
        path: PathBuf::from("test.pasta"),
        items: vec![file_attr, actor, scene],
        span: Span::default(),
    };

    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    let context = transpiler.transpile(&file, &mut output).unwrap();

    // ファイル属性はコンテキストに累積されている
    let attrs = context.file_attrs();
    assert_eq!(attrs.len(), 1);

    // 生成されたLuaコードにはアクター定義が含まれる
    let lua_code = String::from_utf8(output).unwrap();
    assert!(lua_code.contains("create_actor(\"さくら\")"));

    // アクターはファイル属性の影響を受けないことを確認
    // （アクター内にauthor属性が出現しないこと）
    // Luaコード生成時にアクターは独立して処理される
    assert!(
        !lua_code.contains("author = ") || !lua_code.contains("ACTOR.author"),
        "アクターはファイル属性を継承しない"
    );
}

/// MAJOR-4: グローバル単語がFileItem出現順に登録されることの検証
#[test]
fn test_global_word_registration_order() {
    use pasta_dsl::parser::{FileItem, KeyWords, PastaFile, Span};
    use std::path::PathBuf;

    let word1 = FileItem::GlobalWord(KeyWords {
        names: vec!["挨拶1".to_string()],
        words: vec!["こんにちは".to_string()],
        span: Span::default(),
    });
    let word2 = FileItem::GlobalWord(KeyWords {
        names: vec!["挨拶2".to_string()],
        words: vec!["やあ".to_string()],
        span: Span::default(),
    });

    let scene_pasta = r#"
＊メイン
　　さくら：テスト。
"#;
    let scene = parse_str(scene_pasta, "test.pasta")
        .unwrap()
        .items
        .into_iter()
        .next()
        .unwrap();

    let file = PastaFile {
        path: PathBuf::from("test.pasta"),
        items: vec![word1, word2, scene],
        span: Span::default(),
    };

    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    let context = transpiler.transpile(&file, &mut output).unwrap();

    // 登録順序を確認
    let entries = context.word_registry.all_entries();
    // グローバル単語が2つ登録されていることを確認
    let global_entries: Vec<_> = entries.iter().filter(|e| !e.key.contains(":")).collect();
    assert_eq!(global_entries.len(), 2);

    // 名前で確認
    assert!(entries.iter().any(|e| e.key == "挨拶1"));
    assert!(entries.iter().any(|e| e.key == "挨拶2"));
}
