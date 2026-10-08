//! 宣言側の別名置き換え（scene-name-alias task 3）。
//!
//! トランスパイラの設定に別名表を渡すと、グローバルシーンの宣言名が置き換え後の名前になり、
//! 登録・通し番号・単語のモジュール名・生成コードの基本名・突合キー・ローカルの親名の
//! すべてに同じ値が使われることを確かめる（1.2, 1.3, 3.1, 4.1, 4.2, 4.7, 7.1, 8.3）。

use pasta_dsl::parse_str;
use pasta_lua::code_gen::source_map::SourceMapSink;
use pasta_lua::{LuaTranspiler, SceneAliasTable, TranspileContext, TranspilerConfig};

/// 突合キーとシーン行（`record_scene`）と行対応（`record_line`）を記録する。
#[derive(Default)]
struct KeySink {
    scene_keys: Vec<(String, usize)>,
    lines: Vec<(u32, u32)>,
}

impl SourceMapSink for KeySink {
    fn record_line(&mut self, lua_line: u32, pasta_line: u32) {
        self.lines.push((lua_line, pasta_line));
    }

    fn record_scene(&mut self, scene_join_key: &str, span: pasta_dsl::parser::Span) {
        self.scene_keys
            .push((scene_join_key.to_string(), span.start_line));
    }
}

struct Output {
    lua: String,
    context: TranspileContext,
    sink: KeySink,
}

fn transpile(source: &str, aliases: SceneAliasTable) -> Output {
    let file = parse_str(source, "alias.pasta").unwrap();
    let transpiler = LuaTranspiler::new(TranspilerConfig::new().with_scene_aliases(aliases));
    let mut sink = KeySink::default();
    let mut out = Vec::new();
    let context = transpiler
        .transpile_with_sink(&file, &mut out, Some(&mut sink))
        .unwrap();
    Output {
        lua: String::from_utf8(out).unwrap(),
        context,
        sink,
    }
}

fn keys(out: &Output) -> Vec<&str> {
    out.sink
        .scene_keys
        .iter()
        .map(|(k, _)| k.as_str())
        .collect()
}

fn fn_names(out: &Output) -> Vec<&str> {
    out.context
        .scene_registry
        .all_scenes()
        .iter()
        .map(|e| e.fn_name.as_str())
        .collect()
}

const KAIWA: &str = "\
％さくら
　＠通常：\\s[0]

＊会話
　＠話題：天気、季節
　　さくら：こんにちは。
　・挨拶
　　さくら：やあ。
";

/// 既定表のもとで `＊会話` の名前はすべて OnTalk 系になる（1.2, 3.1, 4.1, 8.3）。
#[test]
fn builtin_default_replaces_declared_name_everywhere() {
    let out = transpile(KAIWA, SceneAliasTable::builtin_default());

    // 生成コードの基本名
    assert!(
        out.lua.contains(r#"PASTA.create_scene("OnTalk")"#),
        "{}",
        out.lua
    );
    assert!(!out.lua.contains(r#"create_scene("会話")"#), "{}", out.lua);

    // 突合キー（グローバル・ローカルの親）
    assert_eq!(
        keys(&out),
        vec!["G:OnTalk#1", "L:OnTalk#1:__start__", "L:OnTalk#1:挨拶_1"]
    );

    // 登録名（グローバルとローカルの親）
    assert_eq!(
        fn_names(&out),
        vec!["OnTalk_1::__start__", "OnTalk_1::挨拶_1"]
    );
    let scenes = out.context.scene_registry.all_scenes();
    assert_eq!(scenes[0].name, "OnTalk");
    assert_eq!(scenes[1].parent.as_deref(), Some("OnTalk"));
    // ローカルシーン名は置き換えない（4.7）
    assert_eq!(scenes[1].name, "挨拶");

    // シーン単語のモジュール名
    let word_keys: Vec<&str> = out
        .context
        .word_registry
        .all_entries()
        .iter()
        .map(|e| e.key.as_str())
        .collect();
    assert!(word_keys.contains(&":OnTalk_1:話題"), "{word_keys:?}");
    // アクター単語は置き換えない（4.7）
    assert!(
        word_keys.contains(&":__actor_さくら__:通常"),
        "{word_keys:?}"
    );
}

/// `＊OnTalk`・`＊会話`・単独 `＊`（`会話` を受け継ぐ）は同じ基本名の出現順で採番される（1.3, 4.2, 7.3）。
#[test]
fn alias_real_name_and_continuation_share_numbering() {
    let source = "\
＊OnTalk
　　さくら：いち。
＊会話
　　さくら：に。
＊
　　さくら：さん。
";
    let out = transpile(source, SceneAliasTable::builtin_default());

    let globals: Vec<(&str, usize)> = out
        .sink
        .scene_keys
        .iter()
        .filter(|(k, _)| k.starts_with("G:"))
        .map(|(k, l)| (k.as_str(), *l))
        .collect();
    assert_eq!(
        globals,
        vec![("G:OnTalk#1", 1), ("G:OnTalk#2", 3), ("G:OnTalk#3", 5)]
    );
    assert_eq!(
        fn_names(&out),
        vec![
            "OnTalk_1::__start__",
            "OnTalk_2::__start__",
            "OnTalk_3::__start__"
        ]
    );
    assert_eq!(
        out.lua.matches(r#"PASTA.create_scene("OnTalk")"#).count(),
        3
    );
}

/// `＊会話・朝` とローカル `・会話` は置き換わらない（3.1 完全一致・4.7）。
#[test]
fn non_exact_and_local_names_are_not_replaced() {
    let source = "\
＊会話・朝
　　さくら：おはよう。
＊挨拶
　　さくら：やあ。
　・会話
　　さくら：ローカル。
";
    let out = transpile(source, SceneAliasTable::builtin_default());

    assert!(
        out.lua.contains(r#"PASTA.create_scene("会話_朝")"#),
        "{}",
        out.lua
    );
    assert!(out.lua.contains("function SCENE.会話_1("), "{}", out.lua);
    assert!(!out.lua.contains("OnTalk"), "{}", out.lua);
    assert_eq!(
        fn_names(&out),
        vec![
            "会話_朝_1::__start__",
            "挨拶_1::__start__",
            "挨拶_1::会話_1"
        ]
    );
}

/// ブレークポイント行の対応は名前に依存しない（7.1）。
#[test]
fn line_records_are_independent_of_aliases() {
    let with = transpile(KAIWA, SceneAliasTable::builtin_default());
    let without = transpile(KAIWA, SceneAliasTable::empty());
    assert_eq!(with.sink.lines, without.sink.lines);
}

/// 空の表では従来の出力と同じ（`＊会話` は会話のまま）。
#[test]
fn empty_table_keeps_legacy_output() {
    let out = transpile(KAIWA, SceneAliasTable::empty());
    assert!(
        out.lua.contains(r#"PASTA.create_scene("会話")"#),
        "{}",
        out.lua
    );
    assert_eq!(keys(&out)[0], "G:会話#1");
    assert_eq!(fn_names(&out)[0], "会話_1::__start__");

    // 既定のトランスパイラ（空の表）と sample.pasta の出力が一致する。
    let sample = include_str!("../fixtures/sample.pasta");
    let file = parse_str(sample, "sample.pasta").unwrap();
    let render = |t: LuaTranspiler| {
        let mut out = Vec::new();
        t.transpile(&file, &mut out).unwrap();
        String::from_utf8(out).unwrap()
    };
    let legacy = render(LuaTranspiler::default());
    assert_eq!(
        render(LuaTranspiler::new(
            TranspilerConfig::new().with_scene_aliases(SceneAliasTable::empty())
        )),
        legacy
    );
    // sample.pasta には別名に完全一致する宣言が無いので、既定表でも出力は変わらない。
    assert_eq!(
        render(LuaTranspiler::new(
            TranspilerConfig::new().with_scene_aliases(SceneAliasTable::builtin_default())
        )),
        legacy
    );
}
