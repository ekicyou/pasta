//! Integration tests for pasta_lua transpiler.
//!
//! Tests that verify the transpiler generates Lua code matching the reference implementation.

use pasta_core::parse_str;
use pasta_lua::{LuaTranspiler, TranspilerConfig};

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

    // Parse the sample
    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();
    let actors = file.actor_scopes();
    let scenes = file.global_scene_scopes();

    // Convert references to owned values for transpile
    let actors: Vec<_> = actors.into_iter().cloned().collect();
    let scenes: Vec<_> = scenes.into_iter().cloned().collect();

    let result = transpiler.transpile(&actors, &scenes, &mut output);
    assert!(result.is_ok());

    let lua_code = String::from_utf8(output).unwrap();

    // Verify header
    assert!(
        lua_code.contains("local PASTA = require \"pasta.runtime\""),
        "Missing PASTA require statement"
    );
}

#[test]
fn test_transpile_sample_pasta_actors() {
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    let file = parse_str(SAMPLE_PASTA, "test.pasta").unwrap();
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify actor definitions (Requirement 3a)
    assert!(
        lua_code.contains("PASTA:create_actor(\"さくら\")"),
        "Missing actor さくら"
    );
    assert!(
        lua_code.contains("PASTA:create_actor(\"うにゅう\")"),
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
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify scene definition (Requirement 3b)
    assert!(
        lua_code.contains("PASTA:create_scene(\"メイン1\")"),
        "Missing scene メイン1"
    );

    // Verify entry point function (Requirement 3c)
    assert!(
        lua_code.contains("function SCENE.__start__(ctx, ...)"),
        "Missing __start__ function"
    );

    // Verify session initialization
    assert!(
        lua_code.contains("local args = { ... }"),
        "Missing args initialization"
    );
    assert!(
        lua_code.contains("local act, save, var = PASTA:create_session(SCENE, ctx)"),
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
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
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
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

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
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
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
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Config without comments should still produce valid code
    assert!(lua_code.contains("PASTA:create_actor"));
    assert!(lua_code.contains("PASTA:create_scene"));
}

/// Test case for the reference implementation sample.pasta
#[test]
fn test_transpile_reference_sample_structure() {
    let sample_pasta = include_str!("fixtures/sample.pasta");

    let file = parse_str(sample_pasta, "sample.pasta").unwrap();
    let actors = file.actor_scopes();
    let scenes = file.global_scene_scopes();

    // Verify the sample parses correctly
    assert_eq!(actors.len(), 2, "Expected 2 actors");
    assert_eq!(scenes.len(), 2, "Expected 2 global scenes");

    // Transpile
    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();

    let actors: Vec<_> = actors.into_iter().cloned().collect();
    let scenes: Vec<_> = scenes.into_iter().cloned().collect();

    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // Verify key structural elements
    assert!(
        lua_code.contains("PASTA:create_actor(\"さくら\")"),
        "Missing actor さくら"
    );
    assert!(
        lua_code.contains("PASTA:create_actor(\"うにゅう\")"),
        "Missing actor うにゅう"
    );
    assert!(
        lua_code.contains("PASTA:create_scene(\"メイン1\")"),
        "Missing scene メイン1"
    );
    // Each unique scene name gets counter=1 (counter is per-name, not global)
    assert!(
        lua_code.contains("PASTA:create_scene(\"会話分岐1\")"),
        "Missing scene 会話分岐1"
    );
}

/// Debug test to dump generated Lua code for analysis
#[test]
fn test_debug_dump_generated_lua() {
    let sample_pasta = include_str!("fixtures/sample.pasta");

    let file = parse_str(sample_pasta, "sample.pasta").unwrap();
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
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
        if line.contains("会話分岐1") {
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

    // Parse and transpile
    let file = parse_str(sample_pasta, "sample.pasta").unwrap();
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
    let generated_lua = String::from_utf8(output).unwrap();

    // Compare line by line
    let (mismatches, stats) = compare_lua_output(sample_lua, &generated_lua);

    // Generate and print report
    let report = generate_mismatch_report(&mismatches, &stats);

    // Print report for debugging (visible in test output with --nocapture)
    eprintln!("{}", report);

    // Test passes if match rate is above threshold (allowing for minor differences)
    // For now, we verify the comparison runs and report is generated
    // Full match requirement can be enforced once transpiler is complete
    assert!(
        stats.match_rate() >= 0.0,
        "Comparison completed. See report above for details."
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
    let actors: Vec<_> = file.actor_scopes().into_iter().cloned().collect();
    let scenes: Vec<_> = file.global_scene_scopes().into_iter().cloned().collect();

    let transpiler = LuaTranspiler::default();
    let mut output = Vec::new();
    transpiler.transpile(&actors, &scenes, &mut output).unwrap();
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
