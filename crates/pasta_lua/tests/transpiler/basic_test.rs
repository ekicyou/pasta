//! Basic integration tests for pasta_lua transpiler.
//!
//! Tests that verify the transpiler generates correct Lua code for basic constructs:
//! header, actors, scenes, actions, scope separation, config, and reference patterns.

use crate::common;

use common::normalize_line_endings;
use pasta_dsl::parse_str;
use pasta_dsl::parser::{ActorScope, FileItem, GlobalSceneScope};
use pasta_lua::{LuaTranspiler, TranspilerConfig};

/// Helper to get global scene scopes from PastaFile
fn get_global_scene_scopes(file: &pasta_dsl::parser::PastaFile) -> Vec<&GlobalSceneScope> {
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
fn get_actor_scopes(file: &pasta_dsl::parser::PastaFile) -> Vec<&ActorScope> {
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

    // Verify actor attributes with symmetric API: ACTOR:create_word(key):entry(...) (actor-word-dictionary)
    assert!(
        lua_code.contains("ACTOR:create_word(\"通常\"):entry([=[\\s[0]]=])"),
        "Missing さくら.通常 attribute with create_word API"
    );
    assert!(
        lua_code.contains("ACTOR:create_word(\"照れ\"):entry([=[\\s[1]]=])"),
        "Missing さくら.照れ attribute"
    );
    assert!(
        lua_code.contains("ACTOR:create_word(\"通常\"):entry([=[\\s[10]]=])"),
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
    let sample_pasta = include_str!("../fixtures/sample.pasta");

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
    let sample_pasta = include_str!("../fixtures/sample.pasta");

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

/// Requirement 6: Verify specific code patterns match reference
#[test]
fn test_transpile_reference_code_patterns() {
    let sample_pasta = include_str!("../fixtures/sample.pasta");
    let sample_lua = include_str!("../fixtures/sample.lua");

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
