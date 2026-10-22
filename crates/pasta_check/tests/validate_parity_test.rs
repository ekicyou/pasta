//! 一致テスト: 検査の核の規則が同じ版の pasta.dll（pasta_lua）と同じであること（9.2〜9.4）。
//!
//! (a) モジュール名 (b) 辞書の選び出し (c) 読み込みの成否 (d) トランスパイルの成否 を照らす。
//! pasta_lua の規則が変わるとここが落ちる（design の Revalidation Triggers）。

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use pasta_check::validate::{
    ACT_METHODS, FileText, FindingKind, GLOBAL_NAMES, GhostInput, Report, SHIORI_ACT_METHODS,
    Severity, TomlSource, module_name_of, pasta_dll_folders, pasta_dll_paths,
    select_dictionary_files, validate_ghost,
};
use pasta_lua::loader::{CacheManager, PastaLoader, discover_files};
use tempfile::TempDir;

/// PASTA_DEBUG が環境にあると読み込みが固定ポートを開いて落ちる。main の前（単一スレッド）で外す
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
        std::env::remove_var("PASTA_LOG");
    }
}

const TOML: &str = "[loader]\npasta_patterns = [\"dic/**/*.pasta\"]\n";
const TALK: &str = "＊挨拶\n  さくら：こんにちは。\n";

/// (ゴーストからの相対パス, 中身)。パスが `/` で終わるものは中身の無いフォルダ
type File<'a> = (&'a str, &'a [u8]);

/// 一時のゴースト（ghost/master に当たるフォルダ）を作る
fn ghost(files: &[File]) -> TempDir {
    let temp = TempDir::new().unwrap();
    for (path, body) in files {
        let full = temp.path().join(path);
        if path.ends_with('/') {
            fs::create_dir_all(full).unwrap();
            continue;
        }
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(full, body).unwrap();
    }
    temp
}

/// CLI の殻と同じ形の入力: 最上位の profile に入らず、相対パスを `/` でつなぐ。
/// 入ったフォルダの相対パスも集める
fn input_of(base: &Path) -> GhostInput {
    fn walk(dir: &Path, prefix: &str, out: &mut Vec<String>, dirs: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().into_owned();
            let rel = format!("{prefix}{name}");
            if entry.file_type().unwrap().is_dir() {
                if rel != "profile" {
                    walk(&entry.path(), &format!("{rel}/"), out, dirs);
                    dirs.push(rel);
                }
            } else {
                out.push(rel);
            }
        }
    }
    let mut entries = Vec::new();
    let mut dirs = Vec::new();
    walk(base, "", &mut entries, &mut dirs);
    let pasta_toml = match fs::read_to_string(base.join("pasta.toml")) {
        Ok(text) => TomlSource::Text { text },
        Err(_) => TomlSource::Missing,
    };
    GhostInput {
        pasta_toml,
        entries,
        dirs,
    }
}

fn check(base: &Path) -> Report {
    let input = input_of(base);
    let files: BTreeMap<_, _> = select_dictionary_files(&input)
        .into_iter()
        .map(|p| {
            let text = match String::from_utf8(fs::read(base.join(&p)).unwrap()) {
                Ok(text) => FileText::Text { text },
                Err(_) => FileText::NotUtf8,
            };
            (p, text)
        })
        .collect();
    validate_ghost(&input, &files)
}

// ---- (a) モジュール名 ----

#[test]
fn module_name_matches_cache_manager() {
    let base = std::env::temp_dir().join("ghost");
    let cache = CacheManager::new(base.clone(), "profile/pasta/cache/lua");
    for rel in [
        "dic/a.pasta",
        "dic/a-b.pasta",
        "dic/a.b.pasta",
        "dic/a_b.pasta",
        "dic/dialog/npc/shop-keeper.lua",
        "dic/x.y/z.pasta",
        "talk.pasta",
        "dic.pasta",
        "dicx/a.pasta",
        "Dic/a.pasta",
        "x/dic/a.pasta",
        "dic/日本語/会話.pasta",
    ] {
        assert_eq!(
            module_name_of(rel),
            cache.source_to_module_name(&base.join(rel)),
            "{rel}"
        );
    }
}

// ---- (b) 辞書の選び出し ----

/// pasta.dll（discover_files）が見るパスを、ゴーストからの相対・`/` 区切り・昇順・重複なしで、
/// (ファイル, フォルダ) に分けて。ghost/master そのものは ""
fn dll_discover(base: &Path, patterns: &[String]) -> (Vec<String>, Vec<String>) {
    let (mut files, mut dirs) = (Vec::new(), Vec::new());
    for p in discover_files(base, patterns).unwrap() {
        // `dic/.` の `.` は Path::components に残らない（pasta.dll が読もうとするのも同じフォルダ）
        let rel = p
            .strip_prefix(base)
            .unwrap()
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        if p.is_dir() { &mut dirs } else { &mut files }.push(rel);
    }
    // pasta_lua はパターンごとに glob の順で連結する（並べ替え・重複除去をしない）ので集合で比べる
    for found in [&mut files, &mut dirs] {
        found.sort();
        found.dedup();
    }
    (files, dirs)
}

/// pasta.toml を書き換えながら、核と discover_files の選び出しを照らす
fn assert_discovery_parity(base: &Path, pattern_sets: &[&[&str]]) {
    for patterns in pattern_sets {
        let list = patterns
            .iter()
            .map(|p| format!("{p:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        fs::write(
            base.join("pasta.toml"),
            format!("[loader]\npasta_patterns = [{list}]\n"),
        )
        .unwrap();
        let patterns: Vec<String> = patterns.iter().map(|p| p.to_string()).collect();
        // .lua のパターンは pasta_lua（process.rs）と同じく `.pasta` で終わるものから作る
        let lua_patterns: Vec<String> = patterns
            .iter()
            .filter_map(|p| p.strip_suffix(".pasta").map(|s| format!("{s}.lua")))
            .collect();

        let input = input_of(base);
        let (pasta, lua) = pasta_dll_paths(&input);
        let (dll_pasta, mut dll_dirs) = dll_discover(base, &patterns);
        let (dll_lua, lua_dirs) = dll_discover(base, &lua_patterns);
        assert_eq!(pasta, dll_pasta, ".pasta: {patterns:?}");
        assert_eq!(lua, dll_lua, ".lua: {patterns:?}");
        // フォルダは .pasta と .lua のパターンの和（pasta.dll はどちらで見つけても読もうとして失敗する）
        dll_dirs.extend(lua_dirs);
        dll_dirs.sort();
        dll_dirs.dedup();
        assert_eq!(
            pasta_dll_folders(&input),
            dll_dirs,
            "フォルダ: {patterns:?}"
        );
    }
}

#[test]
fn discovery_matches_discover_files() {
    let temp = ghost(&[
        ("dic/a.pasta", b""),
        ("dic/x/b.pasta", b""),
        ("dic/x/y/c.pasta", b""),
        ("dic/X.PASTA", b""),
        ("dic/d.lua", b""),
        ("dic/x/e.lua", b""),
        ("a.pasta", b""),
        ("dic.pasta", b""),
        ("other/o.pasta", b""),
        ("profile/p.pasta", b""),
        ("profile/q.lua", b""),
        ("notes.txt", b""),
        // 辞書のパターンに一致するフォルダ（中身の無いもの・入れ子・.lua）と、一致しないフォルダ
        ("dic/f.pasta/", b""),
        ("dic/f.lua/", b""),
        ("dic/n.pasta/m.pasta/", b""),
        ("dic/w.txt/", b""),
        ("profile/pf.pasta/", b""),
    ]);
    assert_discovery_parity(
        temp.path(),
        &[
            &["dic/**/*.pasta"],
            &["dic/*.pasta"],
            &["**/*.pasta", "dic/*.pasta"],
            &["d?c/*.pasta"],
            &["dic/**"],
            &["dic/"],
            &["dic/*"],
            &["dic/."],
            &["**"],
            &["*/*.pasta"],
            &[""],
            &["*.txt"],
            &["../*.pasta", "dic/*.pasta"],
            &[],
        ],
    );
}

/// `Dic/` のフォルダ: ワイルドカードの無い成分は Windows のファイルシステムで引かれ、パターンの綴りで見える。
/// 大文字・小文字を区別するファイルシステムでは pasta.dll も一致しないので Windows でだけ照らす
#[cfg(windows)]
#[test]
fn discovery_matches_discover_files_with_capitalized_dic_folder() {
    let temp = ghost(&[
        ("Dic/talk.pasta", b""),
        ("Dic/sub/a.lua", b""),
        ("DIC2/b.pasta", b""),
    ]);
    assert_discovery_parity(
        temp.path(),
        &[&["*/*.pasta"], &["dic/*.pasta"], &["dic/**/*.pasta"]],
    );
    // 最後の pasta.toml（dic/**/*.pasta）のまま、見えるパスがパターンの綴りであること
    let (pasta, lua) = pasta_dll_paths(&input_of(temp.path()));
    assert_eq!(pasta, vec!["dic/talk.pasta"]);
    assert_eq!(lua, vec!["dic/sub/a.lua"]);
}

// ---- (c) 読み込みの成否 ----

/// 読み込みを止める誤り（9.4）。飛び先・シーン・モジュール名の重なりは読み込みを止めない
fn stops_loading(kind: FindingKind) -> bool {
    use FindingKind::*;
    matches!(
        kind,
        ConfigMissing
            | ConfigSyntax
            | ConfigLoaderType
            | ConfigSceneAlias
            | ConfigActorFont
            | ConfigPatternInvalid
            | InitFileName
            | FolderMatchesPattern
            | FileNotUtf8
            | FileUnreadable
            | Syntax
            | TranspileArgAssign
            | TranspilePropertyInExpr
            | TranspileContinuationWithoutActor
    )
}

#[test]
fn load_failure_matches_errors() {
    let toml = TOML.as_bytes();
    let talk = TALK.as_bytes();
    let cases: &[(&str, &[File], bool)] = &[
        // (見本, ファイル, pasta.dll が読み込みに失敗するか)
        (
            "正しい",
            &[("pasta.toml", toml), ("dic/a.pasta", talk)],
            false,
        ),
        (
            "注意だけ（.lua が .pasta に隠れる・飛び先が無い）",
            &[
                ("pasta.toml", toml),
                (
                    "dic/a.pasta",
                    "＊挨拶\n  さくら：はい。\n  ＞無い場面\n".as_bytes(),
                ),
                ("dic/a.lua", b"return {}\n"),
            ],
            false,
        ),
        (
            "別名の表",
            &[
                (
                    "pasta.toml",
                    "[scene.alias]\nOnTalk = [\"会話\", \"雑談\"]\n".as_bytes(),
                ),
                ("dic/a.pasta", "＊雑談\n  さくら：はい。\n".as_bytes()),
            ],
            false,
        ),
        ("pasta.toml が無い", &[("dic/a.pasta", talk)], true),
        (
            "TOML として読めない",
            &[("pasta.toml", b"a = = 1\n"), ("dic/a.pasta", talk)],
            true,
        ),
        (
            "[loader] の型",
            &[
                ("pasta.toml", b"[loader]\npasta_patterns = \"x\"\n"),
                ("dic/a.pasta", talk),
            ],
            true,
        ),
        (
            "[scene.alias] の型",
            &[
                (
                    "pasta.toml",
                    "[scene.alias]\nOnTalk = \"会話\"\n".as_bytes(),
                ),
                ("dic/a.pasta", talk),
            ],
            true,
        ),
        (
            "[scene.alias] の重複",
            &[
                (
                    "pasta.toml",
                    "[scene.alias]\nOnTalk = [\"会話\"]\nOnBoot = [\"会話\"]\n".as_bytes(),
                ),
                ("dic/a.pasta", talk),
            ],
            true,
        ),
        (
            "[scene.alias] の連鎖",
            &[
                (
                    "pasta.toml",
                    "[scene.alias]\nOnTalk = [\"会話\"]\n\"会話\" = [\"雑談\"]\n".as_bytes(),
                ),
                ("dic/a.pasta", talk),
            ],
            true,
        ),
        (
            "init.pasta",
            &[
                ("pasta.toml", toml),
                ("dic/a.pasta", talk),
                ("dic/init.pasta", talk),
            ],
            true,
        ),
        (
            "init.lua",
            &[
                ("pasta.toml", toml),
                ("dic/a.pasta", talk),
                ("dic/init.lua", b"return {}\n"),
            ],
            true,
        ),
        (
            "構文エラー",
            &[
                ("pasta.toml", toml),
                ("dic/a.pasta", "＊\n  ＞＞＞\n".as_bytes()),
            ],
            true,
        ),
        (
            "UTF-8 で読めない",
            &[("pasta.toml", toml), ("dic/a.pasta", b"\x82\xa0\xff\n")],
            true,
        ),
        (
            "中身の無い dic/x.pasta/",
            &[
                ("pasta.toml", toml),
                ("dic/a.pasta", talk),
                ("dic/x.pasta/", b""),
            ],
            true,
        ),
        (
            "中身の無い dic/x.lua/",
            &[
                ("pasta.toml", toml),
                ("dic/a.pasta", talk),
                ("dic/x.lua/", b""),
            ],
            true,
        ),
        (
            "パターンに一致しない dic/sub/",
            &[
                ("pasta.toml", toml),
                ("dic/a.pasta", talk),
                ("dic/sub/", b""),
            ],
            false,
        ),
        // アクターの装い（actor-display-style 7.1〜7.5）
        (
            "正しい装い",
            &[
                (
                    "pasta.toml",
                    "[actor.\"おじさん\"]\nfont = { color = \"90,110,140\", bold = true, height = -2 }\n"
                        .as_bytes(),
                ),
                ("dic/a.pasta", talk),
            ],
            false,
        ),
        (
            "属性の無い装い",
            &[
                ("pasta.toml", "[actor.\"おじさん\"]\nfont = {}\n".as_bytes()),
                ("dic/a.pasta", talk),
            ],
            false,
        ),
        (
            "装いの知らない属性",
            &[
                (
                    "pasta.toml",
                    "[actor.\"おじさん\"]\nfont = { Color = \"red\" }\n".as_bytes(),
                ),
                ("dic/a.pasta", talk),
            ],
            true,
        ),
        (
            "装いの値が浮動小数点数",
            &[
                (
                    "pasta.toml",
                    "[actor.\"おじさん\"]\nfont = { height = 1.5 }\n".as_bytes(),
                ),
                ("dic/a.pasta", talk),
            ],
            true,
        ),
        (
            "装いの値が表",
            &[
                (
                    "pasta.toml",
                    "[actor.\"おじさん\"]\nfont = { color = { r = 1 } }\n".as_bytes(),
                ),
                ("dic/a.pasta", talk),
            ],
            true,
        ),
        (
            "装いの値が空の文字列",
            &[
                (
                    "pasta.toml",
                    "[actor.\"おじさん\"]\nfont = { name = \"\" }\n".as_bytes(),
                ),
                ("dic/a.pasta", talk),
            ],
            true,
        ),
        (
            "装いの値に ]",
            &[
                (
                    "pasta.toml",
                    "[actor.\"おじさん\"]\nfont = { name = \"a]b\" }\n".as_bytes(),
                ),
                ("dic/a.pasta", talk),
            ],
            true,
        ),
        (
            "装いの値に \\",
            &[
                (
                    "pasta.toml",
                    r#"[actor."おじさん"]
font = { name = "a\\b" }
"#
                    .as_bytes(),
                ),
                ("dic/a.pasta", talk),
            ],
            true,
        ),
        (
            "装いが表でない",
            &[
                (
                    "pasta.toml",
                    "[actor.\"おじさん\"]\nfont = \"red\"\n".as_bytes(),
                ),
                ("dic/a.pasta", talk),
            ],
            true,
        ),
        (
            "トランスパイルで失敗する",
            &[
                ("pasta.toml", toml),
                ("dic/a.pasta", "＊場面\n  ：続き\n".as_bytes()),
            ],
            true,
        ),
    ];
    for (name, files, dll_fails) in cases {
        let temp = ghost(files);
        let load_failed = PastaLoader::load(temp.path()).is_err();
        assert_eq!(load_failed, *dll_fails, "見本の前提が違う: {name}");

        let report = check(temp.path());
        let stopping: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Error && stops_loading(f.code))
            .collect();
        assert_eq!(!stopping.is_empty(), load_failed, "{name}: {stopping:?}");
    }
}

// ---- (d) トランスパイルの成否 ----

#[test]
fn transpile_failure_matches_transpile_errors() {
    for body in [
        // 3 つの書き方
        "  ＄x＝＄％p＋1",
        "  ＞場面（＄％p）",
        "  ＞＄％p",
        "  さくら：＠f（＄％p）",
        "  ＄％a＝＄％b",
        "  ・次\n  ＄x＝1\n  ：続き",
        // 許される形
        "  ＄x＝＄％p",
        "  ＄＊g＝＄％p",
        "  さくら：値は＄％pです",
        "  ＄％a＝＄x＋1",
        "  ＄x＝1\n  ：続き",
        "  ＞場面",
    ] {
        let text = format!("＊場面\n  さくら：はい。\n{body}\n");
        let ast = pasta_dsl::parse_str(&text, "dic/a.pasta").expect("構文としては読める");
        let lua_failed = pasta_lua::LuaTranspiler::default()
            .transpile(&ast, &mut Vec::new())
            .is_err();

        let temp = ghost(&[
            ("pasta.toml", TOML.as_bytes()),
            ("dic/a.pasta", text.as_bytes()),
        ]);
        let found = check(temp.path()).findings.into_iter().any(|f| {
            matches!(
                f.code,
                FindingKind::TranspileArgAssign
                    | FindingKind::TranspilePropertyInExpr
                    | FindingKind::TranspileContinuationWithoutActor
            )
        });
        assert_eq!(found, lua_failed, "{body}");
    }
}

// ---- (e) 組み込みの名前の一覧 ----

/// 読み込んだランタイムで、表の値が関数であるキーを昇順で（`__index` は除く）
fn function_keys(lua: &pasta_lua::mlua::Lua, table_expr: &str) -> Vec<String> {
    let mut keys: Vec<String> = lua
        .load(format!(
            "local t = {table_expr}\n\
             local out = {{}}\n\
             for k, v in pairs(t) do\n\
               if type(v) == 'function' and k ~= '__index' then out[#out + 1] = k end\n\
             end\n\
             return out"
        ))
        .eval()
        .unwrap();
    keys.sort();
    keys
}

fn sorted(names: &[&str]) -> Vec<String> {
    let mut names: Vec<String> = names.iter().map(|n| n.to_string()).collect();
    names.sort();
    names
}

#[test]
fn builtin_names_match_runtime() {
    let temp = ghost(&[
        ("pasta.toml", TOML.as_bytes()),
        ("dic/a.pasta", TALK.as_bytes()),
    ]);
    let runtime = PastaLoader::load(temp.path()).unwrap();
    let lua = runtime.lua();
    assert_eq!(
        function_keys(lua, "require 'pasta.global'"),
        sorted(&GLOBAL_NAMES),
        "GLOBAL"
    );
    assert_eq!(
        function_keys(lua, "require('pasta.act').IMPL"),
        sorted(&ACT_METHODS),
        "ACT_IMPL"
    );
    assert_eq!(
        function_keys(lua, "require('pasta.shiori.act').IMPL"),
        sorted(&SHIORI_ACT_METHODS),
        "SHIORI_ACT_IMPL"
    );
}

// ---- (f) 静的な Call の検索 ----

#[test]
fn static_call_lookup_matches_runtime() {
    // (書いたグローバルシーン, Call のキー, 実行時に見つかるか)。1 行 1 Call で、行番号から核の判定を引く
    let calls = [
        ("挨拶", "ローカル", true),             // L2 ローカル
        ("挨拶", "ロー", true),                 // L2 ローカルの前方一致
        ("挨拶", "会話", true), // L5 別名を置き換えたグローバル（既定 OnTalk = ["会話"]）
        ("挨拶", "OnTa", true), // L5 グローバルの前方一致
        ("挨拶", "チェイントーク", true), // L4 GLOBAL
        ("挨拶", "ゴースト終了", true), // L4 GLOBAL（shiori/entry.lua）
        ("挨拶", "自動トーク停止", true), // L4 GLOBAL（shiori/entry.lua）
        ("挨拶", "自動トーク", false), // 組み込みに前方一致は無い
        ("挨拶", "call_key", true), // L3 ACT_IMPL
        ("挨拶", "transfer_date_to_var", true), // L3 SHIORI_ACT_IMPL
        ("挨拶", "無い場面", false), // 無い
        ("挨拶", "内", false),  // 別のグローバルシーンの配下のローカルは届かない
        ("挨拶", "ｙｉｅｌｄ", false), // 組み込みは完全一致だけ
        ("OnTalk", "内", true), // L2 ローカル
        ("OnTalk", "挨", true), // L5 グローバルの前方一致
    ];
    let mut text = String::from("＊挨拶\n  さくら：はい。\n");
    let mut lines = Vec::new();
    let mut current = "挨拶";
    for (global, key, _) in calls {
        if global != current {
            text.push_str("  ・ローカル\n    さくら：ローカル。\n＊OnTalk\n  さくら：雑談。\n");
            current = global;
        }
        text.push_str(&format!("  ＞{key}\n"));
        lines.push(text.lines().count() as u32);
    }
    text.push_str("  ・内\n    さくら：内。\n");

    let temp = ghost(&[
        ("pasta.toml", TOML.as_bytes()),
        ("dic/a.pasta", text.as_bytes()),
    ]);
    let report = check(temp.path());
    let runtime = PastaLoader::load(temp.path()).unwrap();
    let lua = runtime.lua();
    // 実行時の Call と同じ: そのグローバルシーンの表を current_scene にした SHIORI 用の act の find_act_handler
    let find = lua
        .load(
            "local global, key = ...\n\
             local SEARCH = require '@pasta_search'\n\
             local registered = SEARCH:search_scene(global, nil)\n\
             local act = require('pasta.shiori.act').new({})\n\
             act.current_scene = require('pasta.scene').get_global_table(registered)\n\
             assert(act.current_scene, global)\n\
             return act:find_act_handler('scene', key) ~= nil",
        )
        .into_function()
        .unwrap();

    for ((global, key, expected), line) in calls.into_iter().zip(lines) {
        let found_at_runtime: bool = find.call((global, key)).unwrap();
        assert_eq!(
            found_at_runtime, expected,
            "見本の前提が違う: {global} ＞{key}"
        );
        let missing = report
            .findings
            .iter()
            .any(|f| f.code == FindingKind::CallTargetMissing && f.line == Some(line));
        assert_eq!(!missing, found_at_runtime, "{global} ＞{key}（{line} 行）");
    }
}
