//! Basic integration tests for pasta_lua transpiler.
//!
//! Tests that verify the transpiler generates correct Lua code for basic constructs:
//! header, actors, scenes, actions, scope separation, config, and reference patterns.

use crate::common;

use common::normalize_line_endings;
use pasta_dsl::parse_str;
use pasta_dsl::parser::{ActorScope, FileItem, GlobalSceneScope};
use pasta_lua::{LuaTranspiler, SceneAliasTable, TranspilerConfig};

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
    // 書いたシーン名と行の対応を渡す（__start__ はローカル名の代わりに nil・runtime-error-balloon 2.4, 2.5）
    assert!(
        lua_code.contains(r#"local save, var = act:init_scene(SCENE, "メイン", nil, "test.pasta|"#),
        "Missing session initialization: {}",
        lua_code
    );
    assert!(
        lua_code.contains(
            r#"local save, var = act:init_scene(SCENE, "メイン", "自己紹介", "test.pasta|"#
        ),
        "Missing local scene session initialization: {}",
        lua_code
    );

    // Verify local scene function (Requirement 3c)
    assert!(
        lua_code.contains("function SCENE.自己紹介_"),
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
        lua_code.contains("act:actor_proxy(\"さくら\"):talk(\"こんにちは。\")"),
        "Missing さくら talk action"
    );
    assert!(
        lua_code.contains("act:actor_proxy(\"うにゅう\"):talk(\"やふぅ。\")"),
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

/// `init_scene` には、別名で置き換える前・記号を識別子に直す前の、書いたとおりの名前を
/// 文字列で渡す。単独の `＊` 行は受け継いだ名前を渡す（runtime-error-balloon 2.4, 2.7）。
#[test]
fn test_init_scene_receives_written_scene_names() {
    let source = "\
％さくら
　＠通常：\\s[0]

＊会話
　　さくら：こんにちは。
　・朝・挨拶
　　　さくら：おはよう。

＊
　　さくら：続きです。

＊天気・雨
　　さくら：雨です。
";
    let file = parse_str(source, "names.pasta").unwrap();
    let transpiler = LuaTranspiler::new(
        TranspilerConfig::new().with_scene_aliases(SceneAliasTable::builtin_default()),
    );
    let mut output = Vec::new();
    transpiler.transpile(&file, &mut output).unwrap();
    let lua_code = String::from_utf8(output).unwrap();

    // 登録には別名・直した名前が使われる
    assert!(
        lua_code.contains(r#"PASTA.create_scene("OnTalk")"#),
        "{lua_code}"
    );
    assert!(
        lua_code.contains(r#"PASTA.create_scene("天気_雨")"#),
        "{lua_code}"
    );
    assert!(
        lua_code.contains("function SCENE.朝_挨拶_1(act, ...)"),
        "{lua_code}"
    );

    // init_scene には書いた名前（単独の ＊ 行は受け継いだ `会話` を 2 回）。4 つ目は行の対応
    let init_lines: Vec<&str> = lua_code
        .lines()
        .map(str::trim)
        .filter(|l| l.contains("act:init_scene("))
        .map(|l| l.split(", \"names.pasta|").next().unwrap())
        .collect();
    assert_eq!(
        init_lines,
        vec![
            r#"local save, var = act:init_scene(SCENE, "会話", nil"#,
            r#"local save, var = act:init_scene(SCENE, "会話", "朝・挨拶""#,
            r#"local save, var = act:init_scene(SCENE, "会話", nil"#,
            r#"local save, var = act:init_scene(SCENE, "天気・雨", nil"#,
        ],
        "{lua_code}"
    );
}

/// 生成コードの `init_scene` の 4 つ目の引数（1 行の文字列の定数）から行の対応を取り出す。
/// 形は `"<.pasta のファイル>|<.lua の行>:<.pasta の行>,..."`。
fn init_scene_line_maps(lua_code: &str) -> Vec<(String, Vec<(u32, u32)>)> {
    lua_code
        .lines()
        .filter(|l| l.contains("act:init_scene("))
        .map(|l| {
            let start = l.rfind(", \"").expect("4 つ目の引数がある") + 3;
            let end = l.rfind("\")").expect("文字列で閉じる");
            let (file, pairs) = l[start..end].split_once('|').expect("ファイルと行の組");
            let pairs = pairs
                .split(',')
                .filter(|p| !p.is_empty())
                .map(|p| {
                    let (lua, pasta) = p.split_once(':').unwrap();
                    (lua.parse().unwrap(), pasta.parse().unwrap())
                })
                .collect();
            (file.to_string(), pairs)
        })
        .collect()
}

/// `init_scene` の行の対応は、同じ `.pasta` から作ったデバッガの対応表と同じ `.pasta` の行を
/// 指し、シーンの Lua のブロックの行も含む。行は増えない（runtime-error-balloon 2.5, 5.7）。
#[test]
fn test_init_scene_line_map_matches_debugger_source_map() {
    use pasta_lua::debug::source_map::MapBuilderSink;

    let source = "\
％さくら
　＠通常：\\s[0]

＊会話
　　さくら：こんにちは。
　・朝・挨拶
　　　さくら：おはよう。
```lua
function SCENE.壊れた天気(act)
    local data = nil
    return data.x
end
```
";
    let file = parse_str(source, "dic/天気.pasta").unwrap();
    let transpiler = LuaTranspiler::new(
        TranspilerConfig::new().with_scene_aliases(SceneAliasTable::builtin_default()),
    );

    // デバッガを有効にしない読み込みと同じ経路
    let mut plain = Vec::new();
    transpiler.transpile(&file, &mut plain).unwrap();

    // デバッガの対応表（同じ .pasta から作る）
    let mut sink = MapBuilderSink::new("dic/天気.pasta".to_string(), "chunk".to_string());
    let mut out = Vec::new();
    let (_, shift) = transpiler
        .transpile_with_source_map(&file, &mut out, Some(&mut sink))
        .unwrap();
    assert_eq!(plain, out, "対応表を作っても生成コードは変わらない");
    let chunk = sink.finish(&shift);
    let lua_code = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = lua_code.lines().collect();

    // __start__ はローカル名の代わりに nil を置き、4 つ目に行の対応を渡す
    assert!(
        lua_code.contains(r#"act:init_scene(SCENE, "会話", nil, "dic/天気.pasta|"#),
        "{lua_code}"
    );
    assert!(
        lua_code.contains(r#"act:init_scene(SCENE, "会話", "朝・挨拶", "dic/天気.pasta|"#),
        "{lua_code}"
    );

    let maps = init_scene_line_maps(&lua_code);
    assert_eq!(maps.len(), 2, "{lua_code}");
    for (file, pairs) in &maps {
        assert_eq!(file, "dic/天気.pasta");
        assert!(!pairs.is_empty(), "{lua_code}");
        for &(lua_line, pasta_line) in pairs {
            assert_eq!(
                chunk.pasta_for_lua(lua_line).map(|p| p.line),
                Some(pasta_line),
                "lua {lua_line} はデバッガの対応表と同じ .pasta の行を指す\n{lua_code}"
            );
        }
    }

    // 行は増えない: デバッガの対応表の行（normalize の後の最終の行）が生成コードの行と合う
    let line_of = |needle: &str| lines.iter().position(|l| l.contains(needle)).unwrap() as u32 + 1;
    let header = line_of("function SCENE.朝_挨拶_1(act, ...)");
    assert_eq!(chunk.pasta_for_lua(header).map(|p| p.line), Some(6));
    let broken = line_of("return data.x");
    assert_eq!(chunk.pasta_for_lua(broken).map(|p| p.line), Some(11));

    // ローカルシーンの対応は自分のシーン関数の行と Lua のブロックの行を持つ
    assert!(maps[1].1.contains(&(header, 6)), "{lua_code}");
    assert!(maps[1].1.contains(&(broken, 11)), "{lua_code}");
    // __start__ の対応はローカルシーンの関数の行を持たない
    assert!(!maps[0].1.iter().any(|&(l, _)| l == header), "{lua_code}");
}
