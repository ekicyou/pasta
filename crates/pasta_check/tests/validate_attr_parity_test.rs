//! 一致テスト: 検査が登録するシーンの属性が、同じゴーストを読み込んだ pasta.dll（pasta_lua）の
//! `SCENE.attrs` と項目ごとに同じであること（scene-attribute-store 7.4）。
//!
//! 値は「Lua の型:Lua の tostring の表記」で比べる（引用した数字の文字列と数を区別する）。

use std::collections::BTreeMap;
use std::fs;

use pasta_check::validate::{FileText, GhostInput, TomlSource, scene_attrs};
use pasta_core::{SceneAttr, SceneAttrValue};
use pasta_lua::loader::PastaLoader;
use tempfile::TempDir;

/// PASTA_DEBUG が環境にあると読み込みが固定ポートを開いて落ちる。main の前（単一スレッド）で外す
#[ctor::ctor(unsafe)]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
        std::env::remove_var("PASTA_LOG");
    }
}

const TOML: &str = "[loader]\npasta_patterns = [\"dic/**/*.pasta\"]\n";

/// 宣言（整数・小数・整数の値の小数・全角の数字・負の数・i64 に収まらない数・15 桁の小数・
/// 引用あり/なし/空の文字列）、
/// 条件（全角と半角の演算子すべて）、フラグ、ファイル属性の受け継ぎと置き換え、名前なしの `＊`、
/// 宣言行に属性のあるローカル、属性の無いシーンとローカル、同じ名前の宣言の重ね書き
const DIC_A: &str = "\
＆背景：教室＆＄＊初回＆重み：1
＊雑談＆背景：屋上＆＄＊経験値≧500＆＄好感度>=10
　＆割合：0.5＆整数小数：2.0＆負：-3＆全角：１２
　＆大：100000000000000000000＆細：0.123456789012345＆長：123456789012345
　さくら：やあ
　・選択肢＆優先：1＆優先：3＆＄x
　さくら：ほい
　・属性なし
　さくら：ふむ
＊＆重み：5＆＄回数＜3＆＄回数≠0
　さくら：よう
＊雑談＆表記：「3」＆空：「」＆素：晴れ＆＄＊季節＝春＆＄気分!=悪い
　＆＄a≦2＆＄b<=1＆＄c<1＆＄d＞0＆＄e>1＆＄f=1
　さくら：二つ目
";
const DIC_B: &str = "＊別\n　さくら：あ\n　・中\n　さくら：い\n";

/// 項目を「キー|演算子|型:値」の 1 行に（無い欄は空）
fn item_of(attr: &SceneAttr) -> String {
    let value = |v: &SceneAttrValue| {
        let ty = match v {
            SceneAttrValue::String(_) => "string",
            _ => "number",
        };
        format!("{ty}:{v}")
    };
    match attr {
        SceneAttr::Decl { key, value: v } => format!("{key}||{}", value(v)),
        SceneAttr::Cond { key, op, value: v } => format!("{key}|{}|{}", op.as_str(), value(v)),
        SceneAttr::Flag { key } => format!("{key}||"),
    }
}

type Attrs = BTreeMap<(String, String), Vec<String>>;

/// 検査の側: CLI の殻と同じく辞書ファイルの中身を渡して scene_attrs を呼ぶ
fn check_side(temp: &TempDir) -> Attrs {
    let input = GhostInput {
        pasta_toml: TomlSource::Text { text: TOML.into() },
        entries: vec![
            "pasta.toml".into(),
            "dic/a.pasta".into(),
            "dic/b.pasta".into(),
        ],
        dirs: vec!["dic".into()],
    };
    let files: BTreeMap<_, _> = ["dic/a.pasta", "dic/b.pasta"]
        .into_iter()
        .map(|p| {
            let text = fs::read_to_string(temp.path().join(p)).unwrap();
            (p.to_string(), FileText::Text { text })
        })
        .collect();
    scene_attrs(&input, &files)
        .into_iter()
        .map(|(k, attrs)| (k, attrs.iter().map(item_of).collect()))
        .collect()
}

/// pasta.dll の側: 登録された全シーン（関数であるもの）について SCENE.attrs を呼ぶ
fn dll_side(temp: &TempDir) -> Attrs {
    let runtime = PastaLoader::load(temp.path()).unwrap();
    let by_key: BTreeMap<String, Vec<String>> = runtime
        .lua()
        .load(
            "local SCENE = require 'pasta.scene'\n\
             local out = {}\n\
             for g, locals in pairs(SCENE.get_all_scenes()) do\n\
               for l, fn in pairs(locals) do\n\
                 if type(fn) == 'function' then\n\
                   local parts = {}\n\
                   for i, item in ipairs(SCENE.attrs(g, l)) do\n\
                     local v = item.value\n\
                     parts[i] = item.key .. '|' .. (item.op or '') .. '|'\n\
                       .. (v == nil and '' or type(v) .. ':' .. tostring(v))\n\
                   end\n\
                   out[g .. '\\t' .. l] = parts\n\
                 end\n\
               end\n\
             end\n\
             return out",
        )
        .eval()
        .unwrap();
    by_key
        .into_iter()
        .map(|(k, v)| {
            let (g, l) = k.split_once('\t').unwrap();
            ((g.to_string(), l.to_string()), v)
        })
        .collect()
}

fn ghost() -> TempDir {
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join("dic")).unwrap();
    fs::write(temp.path().join("pasta.toml"), TOML).unwrap();
    fs::write(temp.path().join("dic/a.pasta"), DIC_A).unwrap();
    fs::write(temp.path().join("dic/b.pasta"), DIC_B).unwrap();
    temp
}

/// 小数の表記の境目（14 桁の丸め・固定表記と指数表記の切り替え・非有限）が LuaJIT の tostring と同じ
#[test]
fn float_notation_matches_luajit_tostring() {
    let temp = ghost();
    let runtime = PastaLoader::load(temp.path()).unwrap();
    let values = [
        0.5,
        -1.5,
        1e20,
        0.123456789012345,
        1e-5,
        0.0001,
        1e14,
        12345678901234.0,
        99999999999999.99,
        1.5e300,
        -2.5e-7,
        -0.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
    ];
    for v in values {
        let lua: String = runtime.lua().load("return tostring(...)").call(v).unwrap();
        assert_eq!(SceneAttrValue::Float(v).to_string(), lua, "{v:?}");
    }
}

#[test]
fn scene_attrs_match_pasta_dll() {
    let temp = ghost();

    let check = check_side(&temp);
    let dll = dll_side(&temp);
    assert_eq!(check, dll);

    // 見本が狙いどおりの形になっていること（両側が同じでも、見本が崩れていれば意味が無い）
    let at = |g: &str, l: &str| &check[&(g.to_string(), l.to_string())];
    assert_eq!(
        at("雑談_1", "__start__"),
        &[
            "背景||string:屋上",
            "＄＊初回||",
            "重み||number:1",
            "＄＊経験値|>=|number:500",
            "＄好感度|>=|number:10",
            "割合||number:0.5",
            "整数小数||number:2",
            "負||number:-3",
            "全角||number:12",
            "大||number:1e+20",
            "細||number:0.12345678901234",
            "長||number:1.2345678901234e+14",
        ]
    );
    // __start__ 以外のローカルにグローバルの属性は出ない
    assert_eq!(at("雑談_1", "選択肢_1"), &["優先||number:3", "＄x||"]);
    assert!(at("雑談_1", "属性なし_1").is_empty());
    assert_eq!(
        at("雑談_2", "__start__"),
        &[
            "背景||string:教室",
            "＄＊初回||",
            "重み||number:5",
            "＄回数|<|number:3",
            "＄回数|!=|number:0",
        ]
    );
    assert_eq!(at("雑談_3", "__start__")[0], "背景||string:教室");
    assert!(at("雑談_3", "__start__").contains(&"表記||string:3".to_string()));
    assert!(at("雑談_3", "__start__").contains(&"空||string:".to_string()));
    assert!(at("別_1", "__start__").is_empty());
    assert!(at("別_1", "中_1").is_empty());
}
