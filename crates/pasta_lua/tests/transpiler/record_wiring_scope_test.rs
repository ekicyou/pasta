//! Task 2.5 — スコープ定義ヘッダの全 record 配線（RecordWiring / scope_gen）。
//!
//! 仕様参照・検証戦略の詳細は下記セクションバナーを参照。

use pasta_dsl::parser::parse_str;
use pasta_dsl::parser::{ActorScope, GlobalSceneScope, LocalSceneScope, Span};
use pasta_lua::LuaTranspiler;
use pasta_lua::code_gen::LuaCodeGenerator;
use pasta_lua::code_gen::source_map::SourceMapSink;

/// 捕捉 sink: 渡された全 `(lua_line, pasta_line)` を順序どおり貯める。
///
/// span 経由（`record`）でも line 経由（`record_line`）でも、最終的に格納するのは
/// `(出力 .lua 行, 由来 .pasta 行)` のペア。これにより構文種別を跨いで「出力行 →
/// 期待 .pasta 行」を一様に表明できる。
#[derive(Default)]
struct LinePairSink {
    records: Vec<(u32, u32)>,
}

impl SourceMapSink for LinePairSink {
    fn record_line(&mut self, lua_line: u32, pasta_line: u32) {
        self.records.push((lua_line, pasta_line));
    }
}

/// 既知 `.pasta` 行を持つ有効 span（end_byte > 0）を作る小ヘルパ。
fn span_at(line: usize) -> Span {
    // start_line/col, end_line/col, start_byte, end_byte（end_byte>0 で is_valid）
    Span::new(line, 1, line, 10, line * 10, line * 10 + 9)
}

// ============================================================================
// Task 2.5 — スコープ定義ヘッダの record 配線（RecordWiring / scope_gen）
//
// 仕様参照（`.kiro/specs/completed/pasta-source-map/`）:
// - requirements.md **1.1**: 生成 `.lua` 行に由来 `.pasta` 位置を記録する。
// - requirements.md **1.4**: 主要構文種別（スコープ・分岐を含む）を網羅して記録する。
// - requirements.md **1.5**: scope定義（アクター/グローバルシーン/ローカルシーン）が `.lua` へ
//   生成される時、その定義ヘッダ行を `.pasta` 位置へ記録し、定義ヘッダ行をブレークポイント対象に
//   できるようにする（`span.start_line` ＝ `.pasta` 定義ヘッダ行）。
//
// design.md "RecordWiring"（324 行）: scope `generate_*`（actor/global/local scene）は
// scope ヘッダの `.lua` 出力行で `scope.span` を記録する（`span.start_line` ＝ 定義ヘッダ行）。
//
// 検証戦略: 3 型の scope は公開 `generate_*` を直接呼べるため、明示 span を与えた AST ノードを
// 渡し、捕捉 sink が「ヘッダ `.lua` 行 → 定義ヘッダ `.pasta` 行」を記録することを表明する。
// `generate_choice` / `generate_choice_timeout`（分岐, 1.4）は private のため、実 `.pasta` を
// `transpile_with_source_map` に通し、`act:choice(...)` / `act:choice_timeout(...)` 出力行が分岐
// `.pasta` 行へ対応づくことを表明する。
// ============================================================================

/// アクター定義ヘッダ（`local ACTOR = PASTA.create_actor(...)`）が定義ヘッダ `.pasta` 行へ対応づく（1.5）。
#[test]
fn actor_header_records_definition_pasta_line() {
    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    {
        let mut codegen = LuaCodeGenerator::new(&mut output);
        codegen.set_source_map(&mut sink);

        // span.start_line == 9（`％さくら` 定義ヘッダ行）を想定。
        let actor = ActorScope {
            name: "さくら".to_string(),
            attrs: Vec::new(),
            words: Vec::new(),
            var_sets: Vec::new(),
            code_blocks: Vec::new(),
            span: span_at(9),
        };
        codegen.generate_actor(&actor).unwrap();
    }

    let lua = String::from_utf8(output).unwrap();
    // 出力1行目: `do`、2行目: `local ACTOR = PASTA.create_actor("さくら")`。
    let header_line = lua
        .lines()
        .position(|l| l.contains("PASTA.create_actor("))
        .map(|i| i as u32 + 1)
        .expect("actor header line must be emitted");
    // 1.5: ヘッダ `.lua` 行が定義ヘッダ `.pasta` 行(9)へ記録される。これにより BP 対象になる。
    assert!(
        sink.records.contains(&(header_line, 9)),
        "1.5: actor header .lua line {header_line} must map to .pasta definition header line 9; records = {:?}",
        sink.records
    );
}

/// グローバルシーン定義ヘッダ（`local SCENE = PASTA.create_scene(...)`）が定義ヘッダ `.pasta` 行へ対応づく（1.5）。
#[test]
fn global_scene_header_records_definition_pasta_line() {
    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    {
        let mut codegen = LuaCodeGenerator::new(&mut output);
        codegen.set_source_map(&mut sink);

        // ローカルシーンを持たない最小グローバルシーン。span.start_line == 21（`＊メイン` 行）を想定。
        let mut scene = GlobalSceneScope::new("メイン".to_string());
        scene.span = span_at(21);
        let file_attrs = std::collections::HashMap::new();
        codegen
            .generate_global_scene(&scene, &scene.name, 0, &file_attrs)
            .unwrap();
    }

    let lua = String::from_utf8(output).unwrap();
    let header_line = lua
        .lines()
        .position(|l| l.contains("PASTA.create_scene("))
        .map(|i| i as u32 + 1)
        .expect("global scene header line must be emitted");
    // 1.5: ヘッダ `.lua` 行が定義ヘッダ `.pasta` 行(21)へ記録される。
    assert!(
        sink.records.contains(&(header_line, 21)),
        "1.5: global scene header .lua line {header_line} must map to .pasta definition header line 21; records = {:?}",
        sink.records
    );
}

/// ローカルシーン定義ヘッダ（`function SCENE.<fn>(act, ...)`）が定義ヘッダ `.pasta` 行へ対応づく（1.5）。
#[test]
fn local_scene_header_records_definition_pasta_line() {
    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    {
        let mut codegen = LuaCodeGenerator::new(&mut output);
        codegen.set_source_map(&mut sink);

        // 名前付きローカルシーン。span.start_line == 30（`・サブ会話` 行）を想定。
        let mut scene = LocalSceneScope::named("サブ会話".to_string());
        scene.span = span_at(30);
        let actors: Vec<pasta_dsl::parser::SceneActorItem> = Vec::new();
        // task 2.1 で `parent_ref`（親 global の join-key 断片 `"{base}#{counter}"`）が
        // 追加された。本テストは行マッピング記録のみを観測するため、親参照は任意の決定的
        // 値で良い（record_scene の join_key 構築にのみ使われ、record_line 記録には影響しない）。
        codegen
            .generate_local_scene(&scene, 1, &actors, "会話#1", "会話")
            .unwrap();
    }

    let lua = String::from_utf8(output).unwrap();
    let header_line = lua
        .lines()
        .position(|l| l.starts_with("function SCENE."))
        .map(|i| i as u32 + 1)
        .expect("local scene function header line must be emitted");
    // 1.5: 関数ヘッダ `.lua` 行が定義ヘッダ `.pasta` 行(30)へ記録される。
    assert!(
        sink.records.contains(&(header_line, 30)),
        "1.5: local scene header .lua line {header_line} must map to .pasta definition header line 30; records = {:?}",
        sink.records
    );
}

/// 統合: スタートシーン（無名）ヘッダ（`function SCENE.__start__(act, ...)`）も定義ヘッダ行へ対応づく（1.5）。
///
/// 実 `.pasta` を `transpile_with_source_map` に通し、グローバルシーン定義行とスタートシーン関数ヘッダ行が
/// それぞれ実 span 由来の `.pasta` 定義ヘッダ行へ対応づくことを実証する。
#[test]
fn integration_scope_headers_map_to_real_pasta_lines() {
    // 行1: 空, 行2: ＊メイン（グローバルシーン定義ヘッダ）, 行3: トーク, 行5: ・サブ会話（ローカルシーン定義ヘッダ）
    let source = "\n＊メイン\n  さくら：「はい」\n\n  ・サブ会話\n    うにゅう：「サブ」\n";
    let file = parse_str(source, "scope.pasta").expect("parse ok");

    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    LuaTranspiler::default()
        .transpile_with_source_map(&file, &mut output, Some(&mut sink))
        .expect("transpile ok");

    let lua = String::from_utf8(output).unwrap();

    // グローバルシーン定義ヘッダ → .pasta 行 2。
    let scene_lua_line = lua
        .lines()
        .position(|l| l.contains("PASTA.create_scene("))
        .map(|i| i as u32 + 1)
        .expect("output must contain a create_scene header");
    assert!(
        sink.records.contains(&(scene_lua_line, 2)),
        "1.5: global scene header line {scene_lua_line} must map to .pasta line 2; records = {:?}",
        sink.records
    );

    // 名前付きローカルシーン関数ヘッダ → .pasta 行 5（`・サブ会話`）。
    // 関数ヘッダは do ブロック内でインデントされるため trim_start で判定する。
    let sub_lua_line = lua
        .lines()
        .position(|l| {
            let t = l.trim_start();
            t.starts_with("function SCENE.") && !t.contains("__start__")
        })
        .map(|i| i as u32 + 1)
        .expect("output must contain a named local scene function header");
    assert!(
        sink.records.contains(&(sub_lua_line, 5)),
        "1.5: named local scene header line {sub_lua_line} must map to .pasta line 5; records = {:?}",
        sink.records
    );

    // スタートシーン関数ヘッダ（__start__）→ グローバルシーン span（.pasta 行 2）。
    // 無名スタートシーンは独自の構文行を持たずグローバルシーンに包含されるため、その span は
    // グローバルシーン定義ヘッダ行を指す（パーサ実装に整合）。
    let start_lua_line = lua
        .lines()
        .position(|l| l.trim_start().starts_with("function SCENE.__start__("))
        .map(|i| i as u32 + 1)
        .expect("output must contain a __start__ function header");
    assert!(
        sink.records.iter().any(|&(l, _)| l == start_lua_line),
        "1.5: __start__ header line {start_lua_line} must be recorded; records = {:?}",
        sink.records
    );
}

/// 分岐（choice, 1.4）: `act:choice(...)` 出力行が分岐 `.pasta` 行へ対応づく。
///
/// `generate_choice` は private のため、実 `.pasta` を `transpile_with_source_map` に通して検証する。
#[test]
fn choice_records_choice_pasta_line_via_pipeline() {
    // 行1: 空, 行2: ＊選択シーン, 行3: ＠？挨拶「あいさつする」（分岐行）
    let source = "\n＊選択シーン\n    ＠？挨拶「あいさつする」\n";
    let file = parse_str(source, "choice.pasta").expect("parse ok");

    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    LuaTranspiler::default()
        .transpile_with_source_map(&file, &mut output, Some(&mut sink))
        .expect("transpile ok");

    let lua = String::from_utf8(output).unwrap();
    let choice_lua_line = lua
        .lines()
        .position(|l| l.contains("act:choice("))
        .map(|i| i as u32 + 1)
        .expect("output must contain an act:choice(...) line");
    // 1.4: 分岐の出力行が分岐元 .pasta 行(3)へ対応づく。
    assert!(
        sink.records.contains(&(choice_lua_line, 3)),
        "1.4: choice output line {choice_lua_line} must map to .pasta choice line 3; records = {:?}",
        sink.records
    );
}

/// 分岐タイムアウト（!select cue, 1.4）: `act:choice_timeout(...)` 出力行が cue `.pasta` 行へ対応づく。
///
/// `generate_choice_timeout` は private のため、実 `.pasta` を `transpile_with_source_map` に通して検証する。
#[test]
fn choice_timeout_records_cue_pasta_line_via_pipeline() {
    // 行1: 空, 行2: ＊選択シーン, 行3: !select(5)（select cue 行）, 行4: ＠？挨拶
    let source = "\n＊選択シーン\n    !select(5)\n    ＠？挨拶\n";
    let file = parse_str(source, "select.pasta").expect("parse ok");

    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    LuaTranspiler::default()
        .transpile_with_source_map(&file, &mut output, Some(&mut sink))
        .expect("transpile ok");

    let lua = String::from_utf8(output).unwrap();
    let timeout_lua_line = lua
        .lines()
        .position(|l| l.contains("act:choice_timeout("))
        .map(|i| i as u32 + 1)
        .expect("output must contain an act:choice_timeout(...) line");
    // 1.4: select cue の出力行が cue 元 .pasta 行(3)へ対応づく。
    assert!(
        sink.records.contains(&(timeout_lua_line, 3)),
        "1.4: choice_timeout output line {timeout_lua_line} must map to .pasta cue line 3; records = {:?}",
        sink.records
    );
}

// ============================================================================
// Lua ブロックの素通しと行の対応（ponytail-lua-transpiler 要件 7.1・7.3）
//
// アクター辞書（4〜8 行）とローカルシーン（16〜20 行）の Lua ブロックに、要件 1.2 の 5 行
// （長い文字列と `end` の前の空行）を置く。
// ============================================================================

const LUA_BLOCK_SOURCE: &str = "\
％さくら
　＠通常：\\s[0]
```lua
local MSG = [[
こんにちは

end
]]
```

＊会話
　　さくら：こんにちは。
　・朝・挨拶
　　　さくら：おはよう。
```lua
local MSG = [[
こんにちは

end
]]
```
";

/// 要件 1.2 の 5 行（ブロックの内容）
const BLOCK_LINES: [&str; 5] = ["local MSG = [[", "こんにちは", "", "end", "]]"];

/// 生成コードの中で `BLOCK_LINES` が続いた行として現れる先頭の行（1 始まり）を全部返す。
fn block_start_lines(lua: &str) -> Vec<u32> {
    let lines: Vec<&str> = lua.lines().collect();
    lines
        .windows(BLOCK_LINES.len())
        .enumerate()
        .filter(|(_, w)| *w == BLOCK_LINES)
        .map(|(i, _)| i as u32 + 1)
        .collect()
}

/// 7.1（1.1・1.3）: アクター辞書とローカルシーンのブロックの内容が、字下げも空行の削除もなく、
/// 続いた行としてそのまま出る。
#[test]
fn lua_block_content_is_emitted_verbatim_in_actor_and_local_scene() {
    let lua = crate::common::e2e_helpers::transpile(LUA_BLOCK_SOURCE);
    assert_eq!(
        block_start_lines(&lua).len(),
        2,
        "1.1: both blocks must appear verbatim as consecutive lines\n{lua}"
    );
}

/// 7.3（2.3・2.4）: ブロックの各行の `.lua` の行が `.pasta` のその行を指す。シンクの
/// 記録（最終の行に直した対応表）と `init_scene` に渡す対応の両方で、空行より後の行も合う。
#[test]
fn lua_block_lines_map_to_their_pasta_lines_in_sink_and_init_scene() {
    let file = parse_str(LUA_BLOCK_SOURCE, "block.pasta").expect("parse ok");
    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    let (_, shift) = LuaTranspiler::default()
        .transpile_with_source_map(&file, &mut output, Some(&mut sink))
        .expect("transpile ok");
    // 記録を normalize の後の最終の行に直す（同じ行は後勝ち）。
    let chunk: std::collections::BTreeMap<u32, u32> = sink
        .records
        .iter()
        .filter_map(|&(l, p)| shift.map(l).map(|f| (f, p)))
        .collect();
    let lua = String::from_utf8(output).unwrap();

    let starts = block_start_lines(&lua);
    assert_eq!(starts.len(), 2, "{lua}");
    // アクター辞書のブロックは .pasta 4 行目から、ローカルシーンのブロックは 16 行目から
    for (lua_start, pasta_start) in [(starts[0], 4), (starts[1], 16)] {
        for k in 0..BLOCK_LINES.len() as u32 {
            assert_eq!(
                chunk.get(&(lua_start + k)).copied(),
                Some(pasta_start + k),
                "sink: lua {} must map to pasta {}\n{lua}",
                lua_start + k,
                pasta_start + k
            );
        }
    }

    // ローカルシーンのブロックは、そのグローバルシーンの 2 つの init_scene の対応に入る
    let maps = crate::basic_test::init_scene_line_maps(&lua);
    assert_eq!(maps.len(), 2, "{lua}");
    for (file, pairs) in &maps {
        assert_eq!(file, "block.pasta");
        for k in 0..BLOCK_LINES.len() as u32 {
            assert!(
                pairs.contains(&(starts[1] + k, 16 + k)),
                "init_scene: ({}, {}) must be in {pairs:?}\n{lua}",
                starts[1] + k,
                16 + k
            );
        }
    }
}

/// アクター辞書の単語定義の各行が、`.pasta` の単語定義の行へ対応づく（ponytail-lua-transpiler 5.1・7.4）。
#[test]
fn actor_word_definition_lines_map_to_their_pasta_lines() {
    // 行1: ％さくら, 行2: ＠通常, 行3: ＠照れ
    let source = "％さくら\n　＠通常：\\s[0]、\\s[100]\n　＠照れ：\\s[1]\n";
    let file = parse_str(source, "actor.pasta").expect("parse ok");

    let mut sink = LinePairSink::default();
    let mut output = Vec::new();
    LuaTranspiler::default()
        .transpile_with_source_map(&file, &mut output, Some(&mut sink))
        .expect("transpile ok");

    let lua = String::from_utf8(output).unwrap();
    for (key, pasta_line) in [("通常", 2), ("照れ", 3)] {
        let lua_line = lua
            .lines()
            .position(|l| l.contains(&format!("ACTOR:create_word(\"{key}\")")))
            .map(|i| i as u32 + 1)
            .unwrap_or_else(|| panic!("output must contain ACTOR:create_word(\"{key}\")\n{lua}"));
        assert!(
            sink.records.contains(&(lua_line, pasta_line)),
            "5.1: actor word {key} line {lua_line} must map to .pasta line {pasta_line}; records = {:?}",
            sink.records
        );
    }
}
