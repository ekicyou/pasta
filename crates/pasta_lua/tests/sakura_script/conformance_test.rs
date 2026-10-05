//! タグの読み取りの適合テスト（paragraph-break-tag-only-talk 要件 10.1・10.4）
//!
//! 1 つの事例の表 `CASES` を、タグの読み取りを持つ 3 か所に通す。
//! - Rust: `Tokenizer::tokenize`（正。`SAKURA_TAG_PATTERN`）
//! - Lua: `pasta.shiori.appearance` の `tag_at`（写し）
//! - DSL: `grammar.pest` のアクション行（写し）
//!
//! 期待値は新しい規則（要件 7〜9）で書く。移行中は、`changes`（読みが変わる事例）の
//! 印が付いた事例を、まだ書き換えていない読み取りでは飛ばす。

use mlua::prelude::*;
use pasta_dsl::parser::{Action, FileItem, LocalSceneItem, parse_str};
use pasta_lua::loader::TalkConfig;
use pasta_lua::sakura_script::tokenizer::{TokenKind, Tokenizer};
use std::path::PathBuf;

/// 単位の種別。連続する通常の文字は 1 つの `Text` にまとめる。
#[derive(Debug, Clone, Copy, PartialEq)]
enum K {
    Tag,
    Escape,
    Literal,
    Text,
}
use K::*;

/// DSL での扱い
#[derive(Debug, Clone, Copy, PartialEq)]
enum Dsl {
    /// 3 つの読み取りと同じ並び
    Same,
    /// パースエラー（要件 10.3）
    ParseError,
}

struct Case {
    input: &'static str,
    units: &'static [(K, &'static str)],
    dsl: Dsl,
    /// 読みが変わる事例（現行の読み取りと期待値が食い違う）
    changes: bool,
}

const fn case(
    input: &'static str,
    units: &'static [(K, &'static str)],
    dsl: Dsl,
    changes: bool,
) -> Case {
    Case {
        input,
        units,
        dsl,
        changes,
    }
}

use Dsl::*;

const CHG: bool = true;
const KEEP: bool = false;

#[rustfmt::skip]
const CASES: &[Case] = &[
    // 要件 10.4 の 19 件
    case(r"\nHello", &[(Tag, r"\n"), (Text, "Hello")], Same, CHG),
    case(r"\w9OK", &[(Tag, r"\w9"), (Text, "OK")], Same, CHG),
    case(r"\w0", &[(Tag, r"\w"), (Text, "0")], Same, CHG),
    case(r"\s12", &[(Tag, r"\s1"), (Text, "2")], Same, CHG),
    case(r"\_w[100]", &[(Tag, r"\_w[100]")], Same, KEEP),
    case(r"\__w[1]", &[(Tag, r"\__w[1]")], Same, KEEP),
    case(r#"\![raise,X,"a]b"]"#, &[(Tag, r#"\![raise,X,"a]b"]"#)], Same, CHG),
    case(r"\q[a\]b,X]", &[(Tag, r"\q[a\]b,X]")], Same, CHG),
    case(r"\s[0", &[(Tag, r"\s"), (Text, "[0")], Same, KEEP),
    case(r"\\", &[(Escape, r"\\")], Same, KEEP),
    case(r"\%", &[(Escape, r"\%")], Same, CHG),
    case(r"\あ", &[(Text, r"\あ")], ParseError, KEEP),
    case(r"あ\", &[(Text, r"あ\")], ParseError, KEEP),
    case(r"\_?\s[1]\_?", &[(Literal, r"\_?\s[1]\_?")], Same, CHG),
    case(r"\_?abc", &[(Tag, r"\_?"), (Text, "abc")], Same, CHG),
    case(r"\-", &[(Tag, r"\-")], Same, KEEP),
    case(r"\+", &[(Tag, r"\+")], Same, KEEP),
    case(r"\*", &[(Tag, r"\*")], Same, KEEP),
    case(r"\&[amp]", &[(Tag, r"\&[amp]")], Same, KEEP),
    // 設計の TagPattern の表のその他の事例
    case(r"\s3[x]", &[(Tag, r"\s3"), (Text, "[x]")], Same, CHG),
    case(r"\_", &[(Text, r"\_")], ParseError, CHG),
    case(r"\n!?", &[(Tag, r"\n"), (Text, "!?")], Same, CHG),
    case(r"\_?\_?", &[(Literal, r"\_?\_?")], Same, CHG),
    // 引用（引数の先頭の `"` だけが引用を開く）
    case(r#"\q[5"x,OnX]"#, &[(Tag, r#"\q[5"x,OnX]"#)], Same, KEEP),
    case(r#"\![raise,X,a"]b]"#, &[(Tag, r#"\![raise,X,a"]"#), (Text, "b]")], Same, KEEP),
    case(r#"\q["a]b"c,X]z"#, &[(Tag, r#"\q["a]b"c,X]"#), (Text, "z")], Same, CHG),
    case(r#"\q[a\,"b]c"]"#, &[(Tag, r#"\q[a\,"b]"#), (Text, r#"c"]"#)], Same, CHG),
    case(r#"\![a,"b""c]d"]e"#, &[(Tag, r#"\![a,"b""c]d"]"#), (Text, "e")], Same, CHG),
    case(r#"\![call,ghost,"the ""Name"""]"#, &[(Tag, r#"\![call,ghost,"the ""Name"""]"#)], Same, KEEP),
    case(r#"\q["abc,OnX]y"#, &[(Tag, r"\q"), (Text, r#"["abc,OnX]y"#)], Same, CHG),
    case(r#"\![a,"b""]"#, &[(Tag, r"\!"), (Text, r#"[a,"b""]"#)], Same, CHG),
    // `\_?` と `\_!` の混在（同じ印でだけ閉じる）
    case(r"\_?a\_!b\_?", &[(Literal, r"\_?a\_!b\_?")], Same, CHG),
    case(r"\_!a\_?b\_!c", &[(Literal, r"\_!a\_?b\_!"), (Text, "c")], Same, CHG),
    // エスケープ
    case(r"C:\\new", &[(Text, "C:"), (Escape, r"\\"), (Text, "new")], Same, KEEP),
    case(r"100\%です", &[(Text, "100"), (Escape, r"\%"), (Text, "です")], Same, CHG),
    case(r"\\s[0]", &[(Escape, r"\\"), (Text, "s[0]")], Same, KEEP),
    case(r"\\\s[0]", &[(Escape, r"\\"), (Tag, r"\s[0]")], Same, KEEP),
    // 既存のタグ列
    case(
        r"\h\s[0]\_w[500]\![open,inputbox]\-\+\*\_?\&[ID]\n\w8\e",
        &[
            (Tag, r"\h"), (Tag, r"\s[0]"), (Tag, r"\_w[500]"), (Tag, r"\![open,inputbox]"),
            (Tag, r"\-"), (Tag, r"\+"), (Tag, r"\*"), (Tag, r"\_?"), (Tag, r"\&[ID]"),
            (Tag, r"\n"), (Tag, r"\w8"), (Tag, r"\e"),
        ],
        Same,
        KEEP,
    ),
];

type Units = Vec<(K, String)>;

/// 単位を積む。連続する `Text` は 1 つにまとめる。
fn push(units: &mut Units, kind: K, text: &str) {
    if kind == Text
        && let Some((Text, last)) = units.last_mut()
    {
        last.push_str(text);
        return;
    }
    units.push((kind, text.to_string()));
}

/// 期待値を、読み取りが区別できる種別に丸めて返す
fn expected(case: &Case, coarse: impl Fn(K) -> K) -> Units {
    let mut units = Units::new();
    for &(kind, text) in case.units {
        push(&mut units, coarse(kind), text);
    }
    units
}

/// 表の全事例を 1 つの読み取りに通す。`rewritten` が偽なら「読みが変わる事例」を飛ばす。
/// `run` は事例ごとに (期待値, 読み取りの結果) を返す（`None` はパースエラー）。
fn check(reader: &str, rewritten: bool, run: impl Fn(&Case) -> (Option<Units>, Option<Units>)) {
    let mut skipped = 0;
    let mut failures = Vec::new();
    for case in CASES {
        if case.changes && !rewritten {
            skipped += 1;
            continue;
        }
        let (want, got) = run(case);
        if want != got {
            failures.push(format!(
                "  {:?}\n    want {want:?}\n    got  {got:?}",
                case.input
            ));
        }
    }
    println!(
        "conformance[{reader}]: {} run, {skipped} skipped (reading changes)",
        CASES.len() - skipped
    );
    assert!(
        failures.is_empty(),
        "conformance[{reader}] mismatches:\n{}",
        failures.join("\n")
    );
}

/// Rust: トークナイザの `SakuraScript` を単位、それ以外を `Text` として並べる（範囲だけを比べる）
#[test]
fn conformance_rust_tokenizer() {
    let tokenizer = Tokenizer::new(&TalkConfig::default()).unwrap();
    check("rust", true, |case| {
        let mut got = Units::new();
        for token in tokenizer.tokenize(case.input) {
            let kind = if token.kind == TokenKind::SakuraScript {
                Tag
            } else {
                Text
            };
            push(&mut got, kind, &token.text);
        }
        let want = expected(case, |k| if k == Text { Text } else { Tag });
        (Some(want), Some(got))
    });
}

/// Lua: `APPEARANCE.tag_at` を左から回す
#[test]
fn conformance_lua_tag_at() -> LuaResult<()> {
    let lua = Lua::new();
    let scripts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pasta_scripts");
    let package_path = format!("{0}/?.lua;{0}/?/init.lua", scripts.display()).replace('\\', "/");
    lua.globals()
        .get::<LuaTable>("package")?
        .set("path", package_path)?;
    let read: LuaFunction = lua
        .load(
            r#"
            local tag_at = require("pasta.shiori.appearance").tag_at
            return function(s)
                local out, i = {}, 1
                while i <= #s do
                    if s:sub(i, i) ~= "\\" then
                        local j = s:find("\\", i, true) or (#s + 1)
                        out[#out + 1] = { "Text", s:sub(i, j - 1) }
                        i = j
                    else
                        local name, _, next_pos, literal = tag_at(s, i)
                        local kind = (literal and "Literal") or (name and "Tag")
                            or (next_pos == i + 2 and "Escape") or "Text"
                        out[#out + 1] = { kind, s:sub(i, next_pos - 1) }
                        i = next_pos
                    end
                end
                return out
            end
            "#,
        )
        .eval()?;
    check("lua", true, |case| {
        let mut got = Units::new();
        let out: LuaTable = read.call(case.input).unwrap();
        for unit in out.sequence_values::<LuaTable>() {
            let unit = unit.unwrap();
            let kind = match unit.get::<String>(1).unwrap().as_str() {
                "Tag" => Tag,
                "Escape" => Escape,
                "Literal" => Literal,
                _ => Text,
            };
            push(&mut got, kind, &unit.get::<String>(2).unwrap());
        }
        (Some(expected(case, |k| k)), Some(got))
    });
    Ok(())
}

/// DSL: アクション行 1 行をパースし、`SakuraScript` を単位、`Escape` をエスケープ、`Talk` を `Text` として並べる
#[test]
fn conformance_dsl_grammar() {
    check("dsl", false, |case| {
        let want = match case.dsl {
            Same => Some(expected(case, |k| if k == Literal { Tag } else { k })),
            ParseError => None,
        };
        let source = format!("＊t\n　a：{}\n", case.input);
        let got = parse_str(&source, "conformance.pasta").ok().map(|file| {
            let mut got = Units::new();
            for action in action_line(file) {
                match action {
                    Action::SakuraScript { script, .. } => push(&mut got, Tag, &script),
                    Action::Escape { sequence, .. } => push(&mut got, Escape, &sequence),
                    Action::Talk { text, .. } => push(&mut got, Text, &text),
                    other => panic!("unexpected action {other:?} in {:?}", case.input),
                }
            }
            got
        });
        (want, got)
    });
}

/// パース結果から最初のアクション行のアクションを取り出す
fn action_line(file: pasta_dsl::parser::PastaFile) -> Vec<Action> {
    file.items
        .into_iter()
        .find_map(|item| match item {
            FileItem::GlobalSceneScope(scope) => Some(scope),
            _ => None,
        })
        .and_then(|scope| scope.local_scenes.into_iter().next())
        .and_then(|local| {
            local.items.into_iter().find_map(|item| match item {
                LocalSceneItem::ActionLine(line) => Some(line.actions),
                _ => None,
            })
        })
        .expect("アクション行が存在すべし")
}
