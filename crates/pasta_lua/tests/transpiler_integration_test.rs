//! Integration tests for pasta_lua transpiler.
//!
//! Tests that verify the transpiler generates Lua code matching the reference implementation.

use pasta_core::parse_str;
use pasta_core::parser::{ActorScope, FileItem, GlobalSceneScope};
use pasta_lua::{LuaTranspiler, TranspilerConfig};

/// Normalize line endings to LF (\n) for cross-platform comparison.
/// This handles the case where Git's autocrlf setting converts LF to CRLF on Windows.
fn normalize_line_endings(s: &str) -> String {
    s.replace("\r\n", "\n").replace("\r", "\n")
}

/// Helper to get global scene scopes from PastaFile
fn get_global_scene_scopes(file: &pasta_core::parser::PastaFile) -> Vec<&GlobalSceneScope> {
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

/// Helper to get actor scopes from PastaFile
fn get_actor_scopes(file: &pasta_core::parser::PastaFile) -> Vec<&ActorScope> {
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

/// Sample Pasta source for testing
/// アクター定義では ＠ (word_marker) を使用して表情を定義
const SAMPLE_PASTA: &str = r#"
％さくら
　＠通常：\s[0]
　＠照れ：\s[1]

％うにゅう
　＠通常：\s[10]

＊メイン
　　さくら：こんにちは。
　　うにゅう：やふぅ。

　・自己紹介
　　　さくら：私はさくらです。
　　　うにゅう：ワイはうにゅうや。
"#;

#[test]
fn test_transpile_sample_pasta_header() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    // Parse the sample - returns PastaFile directly
    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();

    let result = transpiler.transpile(&file, &mut output);
    assert!(result.is_ok());

    let lua_code = String::from_utf8(output).unwrap();

    // Verify header
    assert!(
        lua_code.contains("local PASTA = require \"pasta\""),
        "Missing PASTA require statement"
    );
}

#[test]
fn test_transpile_sample_pasta_actors() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify actor definitions (Requirement 3a)
    assert!(
        lua_code.contains("PASTA.create_actor(\"さくら\")"),
        "Missing actor さくら"
    );
    assert!(
        lua_code.contains("PASTA.create_actor(\"うにゅう\")"),
        "Missing actor うにゅう"
    );

    // Verify actor attributes with StringLiteralizer (Requirement 2)
    assert!(
        lua_code.contains("ACTOR.通常 = [=[\\s[0]]=]"),
        "Missing さくら.通常 attribute with long string format"
    );
    assert!(
        lua_code.contains("ACTOR.照れ = [=[\\s[1]]=]"),
        "Missing さくら.照れ attribute"
    );
    assert!(
        lua_code.contains("ACTOR.通常 = [=[\\s[10]]=]"),
        "Missing うにゅう.通常 attribute"
    );
}

#[test]
fn test_transpile_sample_pasta_scenes() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify scene definition (Requirement 3b)
    // Counter is now assigned by Lua runtime, not transpiler (Requirement 8.5)
    assert!(
        lua_code.contains("PASTA.create_scene(\"メイン\")"),
        "Missing scene メイン"
    );

    // Verify entry point function (Requirement 3c)
    assert!(
        lua_code.contains("function SCENE.__start__(act, ...)"),
        "Missing __start__ function"
    );

    // Verify session initialization
    assert!(
        lua_code.contains("local args = { ... }"),
        "Missing args initialization"
    );
    assert!(
        lua_code.contains("local save, var = act:init_scene(SCENE)"),
        "Missing session initialization"
    );

    // Verify local scene function (Requirement 3c)
    assert!(
        lua_code.contains("function SCENE.__自己紹介_"),
        "Missing 自己紹介 local scene function"
    );
}

#[test]
fn test_transpile_sample_pasta_actions() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify talk actions (Requirement 3d)
    assert!(
        lua_code.contains("act.さくら:talk(\"こんにちは。\")"),
        "Missing さくら talk action"
    );
    assert!(
        lua_code.contains("act.うにゅう:talk(\"やふぅ。\")"),
        "Missing うにゅう talk action"
    );
}

#[test]
fn test_transpile_do_end_scope_separation() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Normalize line endings for reliable pattern matching
    let lua_code = normalize_line_endings(&lua_code);

    // Verify do...end scope separation (Requirement 1)
    // Count do/end pairs
    let do_count = lua_code.matches("\ndo\n").count() + lua_code.matches("do\n").count();
    let end_count = lua_code.matches("\nend\n").count() + lua_code.matches("end\n").count();

    // Should have at least 3 do...end blocks (2 actors + 1 scene)
    assert!(
        do_count >= 3,
        "Expected at least 3 do blocks, found {}",
        do_count
    );
    assert!(
        end_count >= do_count,
        "Unbalanced do/end blocks: {} do, {} end",
        do_count,
        end_count
    );
}

#[test]
fn test_string_literalizer_in_transpile() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    // Test with SakuraScript that needs long string format
    let pasta = r#"
％さくら
　＠通常：\s[0]
　＠照れ：\s[1]

＊メイン
　　さくら：テスト
"#;

    let file = parse_str(pasta, "test.pasta").unwrap();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify long string format for SakuraScript with brackets
    assert!(
        lua_code.contains("[=[\\s[0]]=]"),
        "SakuraScript should use [=[...]=] format due to brackets"
    );
}

#[test]
fn test_transpile_config_no_comments() {
    let config = TranspilerConfig::without_comments();
    let transpiler = LuaTranspiler::new(config);
    let mut output = Vec::new();

    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Config without comments should still produce valid code
    assert!(lua_code.contains("PASTA.create_actor"));
    assert!(lua_code.contains("PASTA.create_scene"));
}

/// Test case for the reference implementation sample.pasta
#[test]
fn test_transpile_reference_sample_structure() {
    let sample_pasta = include_str!("fixtures/sample.pasta");

    let file = parse_str(sample_pasta, "sample.pasta").unwrap();
    let actors = get_actor_scopes(&file);
    let scenes = get_global_scene_scopes(&file);

    // Verify the sample parses correctly
    assert_eq!(actors.len(), 2, "Expected 2 actors");
    assert_eq!(scenes.len(), 2, "Expected 2 global scenes");

    // Transpile
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify key structural elements
    assert!(
        lua_code.contains("PASTA.create_actor(\"さくら\")"),
        "Missing actor さくら"
    );
    assert!(
        lua_code.contains("PASTA.create_actor(\"うにゅう\")"),
        "Missing actor うにゅう"
    );
    // Counter is now assigned by Lua runtime, not transpiler (Requirement 8.5)
    assert!(
        lua_code.contains("PASTA.create_scene(\"メイン\")"),
        "Missing scene メイン"
    );
    // Each unique scene name gets counter at runtime
    assert!(
        lua_code.contains("PASTA.create_scene(\"会話分岐\")"),
        "Missing scene 会話分岐"
    );
}

/// Debug test to dump generated Lua code for analysis
#[test]
fn test_debug_dump_generated_lua() {
    let sample_pasta = include_str!("fixtures/sample.pasta");

    let file = parse_str(sample_pasta, "sample.pasta").unwrap();
    let scenes = get_global_scene_scopes(&file);

    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Debug: Check code block locations in AST
    eprintln!("\n=== CODE BLOCK LOCATIONS ===");
    for (i, scene) in scenes.iter().enumerate() {
        eprintln!(
            "GlobalScene[{}] '{}': {} code_blocks",
            i,
            scene.name,
            scene.code_blocks.len()
        );
        for (j, local) in scene.local_scenes.iter().enumerate() {
            eprintln!(
                "  LocalScene[{}] '{:?}': {} code_blocks",
                j,
                local.name,
                local.code_blocks.len()
            );
        }
    }
    eprintln!("=== END ===\n");

    // Print specific sections for debugging: code block area
    eprintln!("\n=== GENERATED LUA (引数付き呼び出し and code block) ===");
    let mut in_section = false;
    for line in lua_code.lines() {
        if line.contains("引数付き呼び出し") || line.contains("関数") {
            in_section = true;
        }
        if in_section {
            eprintln!("{}", line);
            if line.trim() == "end" && in_section {
                // Count ends to find the right scope boundary
            }
        }
        // Counter now assigned by Lua runtime, so check for base name
        if line.contains("会話分岐") && line.contains("PASTA.create_scene") {
            break;
        }
    }
    eprintln!("=== END ===\n");
}

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

    report.push_str("\n");
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
    report.push_str("\n");

    // Mismatch pattern classification
    report.push_str("【不一致パターン分類】\n");
    report.push_str(&format!("  内容差異:     {}\n", stats.content_differences));
    report.push_str(&format!("  欠落行:       {}\n", stats.missing_lines));
    report.push_str(&format!("  余剰行:       {}\n", stats.extra_lines));
    report.push_str("\n");

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
    let sample_pasta = include_str!("fixtures/sample.pasta");
    let sample_lua = include_str!("fixtures/sample.lua");
    let sample_expected = include_str!("fixtures/sample.expected.lua");

    // Parse and transpile
    let file = parse_str(sample_pasta, "sample.pasta").unwrap();

    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let generated_lua = String::from_utf8(output).unwrap();

    // Save generated output for debugging
    std::fs::write("tests/fixtures/sample.generated.lua", &generated_lua)
        .expect("Failed to write sample.generated.lua");

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
        "Comparison completed. See report above for details. Generated output saved to tests/fixtures/sample.generated.lua"
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

/// Requirement 6: Verify specific code patterns match reference
#[test]
fn test_transpile_reference_code_patterns() {
    let sample_pasta = include_str!("fixtures/sample.pasta");
    let sample_lua = include_str!("fixtures/sample.lua");

    // Parse and transpile
    let file = parse_str(sample_pasta, "sample.pasta").unwrap();

    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let generated_lua = String::from_utf8(output).unwrap();

    // Extract code lines (non-comment) from reference
    let reference_patterns: Vec<&str> = sample_lua
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("--") && !trimmed.starts_with("local PASTA") // Skip require line (may differ)
        })
        .map(|line| line.trim())
        .collect();

    // Check that key patterns from reference exist in generated output
    let generated_normalized = generated_lua.replace("    ", "\t");

    let mut missing_patterns = Vec::new();
    let mut found_count = 0;

    for pattern in &reference_patterns {
        // Normalize pattern for comparison
        let pattern_normalized = pattern.replace("    ", "\t");
        if generated_normalized.contains(&pattern_normalized) || generated_lua.contains(*pattern) {
            found_count += 1;
        } else {
            missing_patterns.push(*pattern);
        }
    }

    // Report missing patterns
    if !missing_patterns.is_empty() {
        eprintln!("\n【参照実装に存在するが生成出力に欠落しているパターン】");
        for (idx, pattern) in missing_patterns.iter().take(10).enumerate() {
            eprintln!("  [{}] {}", idx + 1, pattern);
        }
        if missing_patterns.len() > 10 {
            eprintln!("  ... 他 {} 件", missing_patterns.len() - 10);
        }
    }

    let coverage = (found_count as f64 / reference_patterns.len() as f64) * 100.0;
    println!(
        "\n【パターンカバレッジ】: {:.1}% ({}/{})",
        coverage,
        found_count,
        reference_patterns.len()
    );

    // Assert reasonable coverage
    assert!(
        coverage >= 50.0,
        "Pattern coverage too low: {:.1}%. Expected at least 50%.",
        coverage
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
    use pasta_core::parser::{Attr, AttrValue, FileItem, PastaFile, Span};
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
    use pasta_core::parser::{Attr, AttrValue, FileItem, PastaFile, Span};
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
    use pasta_core::parser::{FileItem, KeyWords, PastaFile, Span};
    use std::path::PathBuf;

    let word1 = FileItem::GlobalWord(KeyWords {
        name: "挨拶1".to_string(),
        words: vec!["こんにちは".to_string()],
        span: Span::default(),
    });
    let word2 = FileItem::GlobalWord(KeyWords {
        name: "挨拶2".to_string(),
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

// ============================================================================
// set_spot Tests (scene-actors-lua-codegen)
// ============================================================================

/// Test 3.1: 複数アクター生成テスト
#[test]
fn test_set_spot_multiple_actors() {
    let source = r#"
＊シーン
　％さくら、うにゅう＝２
　　さくら：こんにちは
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify clear_spot is generated (Requirement 2.1)
    assert!(
        lua_code.contains("act:clear_spot()"),
        "Missing clear_spot call. Generated code:\n{lua_code}"
    );

    // Verify set_spot calls are generated with new format (Requirement 3.1, 3.2)
    assert!(
        lua_code.contains(r#"act:set_spot("さくら", 0)"#),
        "Missing さくら set_spot call. Generated code:\n{lua_code}"
    );
    assert!(
        lua_code.contains(r#"act:set_spot("うにゅう", 2)"#),
        "Missing うにゅう set_spot call. Generated code:\n{lua_code}"
    );

    // Verify order: init_scene -> clear_spot -> set_spot (new order)
    let init_scene_pos = lua_code
        .find("act:init_scene")
        .expect("init_scene not found");
    let clear_spot_pos = lua_code
        .find("act:clear_spot()")
        .expect("clear_spot not found");
    let sakura_pos = lua_code
        .find(r#"act:set_spot("さくら""#)
        .expect("さくら set_spot not found");
    let unyu_pos = lua_code
        .find(r#"act:set_spot("うにゅう""#)
        .expect("うにゅう set_spot not found");

    assert!(
        init_scene_pos < clear_spot_pos,
        "init_scene should come before clear_spot"
    );
    assert!(
        clear_spot_pos < sakura_pos,
        "clear_spot should come before さくら set_spot"
    );
    assert!(
        sakura_pos < unyu_pos,
        "Actor order not preserved: さくら should come before うにゅう"
    );
}

/// Test 3.2: 単一アクター生成テスト
#[test]
fn test_set_spot_single_actor() {
    let source = r#"
＊シーン
　％さくら
　　さくら：こんにちは
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify clear_spot is generated (Requirement 2.1)
    assert!(
        lua_code.contains("act:clear_spot()"),
        "Missing clear_spot call. Generated code:\n{lua_code}"
    );

    // Verify single actor set_spot is generated with new format (Requirement 3.1, 3.2)
    assert!(
        lua_code.contains(r#"act:set_spot("さくら", 0)"#),
        "Missing single actor set_spot call. Generated code:\n{lua_code}"
    );

    // Verify order: init_scene -> clear_spot -> set_spot (new order)
    let init_scene_pos = lua_code
        .find("act:init_scene")
        .expect("init_scene not found");
    let clear_spot_pos = lua_code
        .find("act:clear_spot()")
        .expect("clear_spot not found");
    let set_spot_pos = lua_code
        .find(r#"act:set_spot("さくら""#)
        .expect("set_spot not found");

    assert!(
        init_scene_pos < clear_spot_pos,
        "init_scene should come before clear_spot"
    );
    assert!(
        clear_spot_pos < set_spot_pos,
        "clear_spot should come before set_spot"
    );
}

/// Test 3.3: アクター未定義テスト
#[test]
fn test_set_spot_empty_actors() {
    let source = r#"
＊シーン
　　さくら：こんにちは
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify no set_spot calls are generated (Requirement 2.2)
    assert!(
        !lua_code.contains("set_spot"),
        "set_spot should not be generated when no actors defined. Generated code:\n{lua_code}"
    );

    // Verify no clear_spot call is generated (Requirement 2.2)
    assert!(
        !lua_code.contains("PASTA.clear_spot()"),
        "clear_spot should not be generated when no actors defined. Generated code:\n{lua_code}"
    );
}

/// Test 3.4: 明示的番号付きアクターテスト
#[test]
fn test_set_spot_with_explicit_number() {
    let source = r#"
＊シーン
　％さくら、うにゅう＝２、まりか
　　さくら：こんにちは
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify clear_spot is generated (Requirement 2.1)
    assert!(
        lua_code.contains("act:clear_spot()"),
        "Missing clear_spot call. Generated code:\n{lua_code}"
    );

    // Verify explicit numbers are respected with new format (C# enum rule)
    assert!(
        lua_code.contains(r#"act:set_spot("さくら", 0)"#),
        "Missing さくら set_spot(0). Generated code:\n{lua_code}"
    );
    assert!(
        lua_code.contains(r#"act:set_spot("うにゅう", 2)"#),
        "Missing うにゅう set_spot(2). Generated code:\n{lua_code}"
    );
    assert!(
        lua_code.contains(r#"act:set_spot("まりか", 3)"#),
        "Missing まりか set_spot(3). Generated code:\n{lua_code}"
    );

    // Verify order: init_scene -> clear_spot -> all set_spot (new order)
    let init_scene_pos = lua_code
        .find("act:init_scene")
        .expect("init_scene not found");
    let clear_spot_pos = lua_code
        .find("act:clear_spot()")
        .expect("clear_spot not found");
    let marika_pos = lua_code
        .find(r#"act:set_spot("まりか""#)
        .expect("まりか set_spot not found");

    assert!(
        init_scene_pos < clear_spot_pos,
        "init_scene should come before clear_spot"
    );
    assert!(
        clear_spot_pos < marika_pos,
        "clear_spot should come before last set_spot"
    );
}

// ============================================================================
// Tail Call Optimization (TCO) Tests - Requirement 4
// ============================================================================

/// Test 4.1: 単一の act:call() のみを含むシーン関数から return act:call(...) が生成される
#[test]
fn test_single_call_scene_gets_return() {
    let source = r#"
＊メイン
　　＞シーン2
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify that single call gets return prefix (Requirement 4.1)
    // Counter now assigned by Lua runtime, uses SCENE.__global_name__
    assert!(
        lua_code.contains(r#"return act:call(SCENE.__global_name__, "シーン2""#),
        "Single call should have 'return' prefix for TCO. Generated code:\n{lua_code}"
    );
}

/// Test 4.2: 複数の act:call() を含むシーン関数から、最後の呼び出しのみに return が付与される
#[test]
fn test_multiple_call_scenes_only_last_gets_return() {
    let source = r#"
＊メイン
　　＞シーン1
　　＞シーン2
　　＞シーン3
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify first and second calls do NOT have return (Requirement 4.2)
    // Counter now assigned by Lua runtime, uses SCENE.__global_name__
    assert!(
        lua_code.contains(r#"act:call(SCENE.__global_name__, "シーン1""#),
        "First call should NOT have 'return' prefix. Generated code:\n{lua_code}"
    );
    assert!(
        lua_code.contains(r#"act:call(SCENE.__global_name__, "シーン2""#),
        "Second call should NOT have 'return' prefix. Generated code:\n{lua_code}"
    );

    // Verify the last call HAS return
    assert!(
        lua_code.contains(r#"return act:call(SCENE.__global_name__, "シーン3""#),
        "Last call should have 'return' prefix for TCO. Generated code:\n{lua_code}"
    );

    // Verify first calls do not have return prefix
    // Check that "return act:call" appears exactly once for シーン3
    let return_calls: Vec<_> = lua_code.match_indices("return act:call").collect();
    assert_eq!(
        return_calls.len(),
        1,
        "Only the last call should have 'return'. Found {} return calls. Generated code:\n{lua_code}",
        return_calls.len()
    );
}

/// Test 4.3: act:call() の後に ActionLine が続く場合、return が生成されない
#[test]
fn test_call_scene_followed_by_action_no_return() {
    let source = r#"
％さくら
　＠通常：\s[0]

＊メイン
　　＞シーン2
　　さくら：こんにちは
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify that call does NOT have return when followed by action (Requirement 4.3)
    // Counter now assigned by Lua runtime, uses SCENE.__global_name__
    // The call should appear without return prefix
    assert!(
        lua_code.contains(r#"act:call(SCENE.__global_name__, "シーン2""#),
        "Call should be present. Generated code:\n{lua_code}"
    );

    // Verify return is NOT before this call (it's not the tail)
    assert!(
        !lua_code.contains(r#"return act:call(SCENE.__global_name__, "シーン2""#),
        "Call followed by action should NOT have 'return' prefix. Generated code:\n{lua_code}"
    );

    // Verify the talk action comes after the call
    let call_pos = lua_code
        .find(r#"act:call(SCENE.__global_name__, "シーン2""#)
        .expect("Call not found");
    let talk_pos = lua_code
        .find(r#"act.さくら:talk("#)
        .expect("Talk not found");
    assert!(call_pos < talk_pos, "Call should come before talk action");
}

/// Test 4.4: シーン関数に act:call() が含まれない場合、return が生成されない
#[test]
fn test_no_call_scene_no_return() {
    let source = r#"
％さくら
　＠通常：\s[0]

＊メイン
　　さくら：こんにちは
　　さくら：さようなら
"#;
    let file = parse_str(source, "test.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify no return statements for calls (only function return, not call return)
    // The scene should have talk actions but no return act:call
    assert!(
        !lua_code.contains("return act:call"),
        "No call scene means no 'return act:call' should be generated. Generated code:\n{lua_code}"
    );

    // Verify talk actions are generated correctly
    assert!(
        lua_code.contains(r#"act.さくら:talk("こんにちは")"#),
        "Talk action should be present. Generated code:\n{lua_code}"
    );
    assert!(
        lua_code.contains(r#"act.さくら:talk("さようなら")"#),
        "Second talk action should be present. Generated code:\n{lua_code}"
    );
}

/// Test 4.5: テストフィクスチャを使用した末尾再帰最適化の E2E 検証
#[test]
fn test_tail_call_optimization_fixture() {
    let source = include_str!("../../../tests/fixtures/tail_call_optimization.pasta");
    let file = parse_str(source, "tail_call_optimization.pasta").unwrap();
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Pattern 1: 単一呼び出し - return が付く
    // Counter now assigned by Lua runtime, so uses SCENE.__global_name__
    assert!(
        lua_code.contains(r#"return act:call(SCENE.__global_name__, "シーン2""#),
        "単一呼び出し scene should have return. Generated code:\n{lua_code}"
    );

    // Pattern 2: 複数呼び出し - 最後の呼び出しのみ return
    // シーン1, シーン2 には return なし
    let multiple_call_section = lua_code
        .find("__複数呼び出し")
        .expect("複数呼び出し function not found");
    let next_function = lua_code[multiple_call_section..]
        .find("\n    function SCENE.")
        .map(|pos| multiple_call_section + pos)
        .unwrap_or(lua_code.len());

    let multiple_call_code = &lua_code[multiple_call_section..next_function];

    // シーン3 のみ return
    assert!(
        multiple_call_code.contains(r#"return act:call(SCENE.__global_name__, "シーン3""#),
        "Last call in 複数呼び出し should have return. Section:\n{multiple_call_code}"
    );

    // シーン1, シーン2 は return なし（カウント確認）
    let return_count = multiple_call_code.matches("return act:call").count();
    assert_eq!(
        return_count, 1,
        "Only シーン3 should have return in 複数呼び出し. Found {} return calls. Section:\n{multiple_call_code}",
        return_count
    );

    // Pattern 3: 呼び出し後にアクション - return なし
    let action_after_call_section = lua_code
        .find("__呼び出し後にアクション")
        .expect("呼び出し後にアクション function not found");
    let next_function2 = lua_code[action_after_call_section..]
        .find("\n    function SCENE.")
        .map(|pos| action_after_call_section + pos)
        .unwrap_or(lua_code.len());

    let action_after_call_code = &lua_code[action_after_call_section..next_function2];

    assert!(
        !action_after_call_code.contains("return act:call"),
        "Call followed by action should NOT have return. Section:\n{action_after_call_code}"
    );

    // Pattern 4: 呼び出しなし - return act:call なし
    let no_call_section = lua_code
        .find("__呼び出しなし")
        .expect("呼び出しなし function not found");
    let next_function3 = lua_code[no_call_section..]
        .find("\n    function SCENE.")
        .map(|pos| no_call_section + pos)
        .unwrap_or(lua_code.len());

    let no_call_code = &lua_code[no_call_section..next_function3];

    assert!(
        !no_call_code.contains("return act:call"),
        "No call scene should have no return act:call. Section:\n{no_call_code}"
    );
}
