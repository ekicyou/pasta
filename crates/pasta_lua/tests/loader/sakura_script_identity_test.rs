//! `@pasta_sakura_script.talk_to_script` の出力のバイト不変の特性化テスト
//!
//! 本番の読み込み（`PastaLoader::load`）で登録されたモジュールと、新しい VM で
//! `sakura_script::register` を直に呼んだモジュールが、同じ入力でバイト単位で
//! 同じ出力を返すことを固める（ponytail-lua-loader Requirement 5.3）。

use crate::common;

use common::copy_fixture_to_temp;
use mlua::{Lua, Table, Value};
use pasta_lua::loader::{PastaConfig, PastaLoader};

/// `SAKURA` を受け取り、改行・ウェイト・アクターの切り替えを含む出力の列を返す
const CASES: &str = r#"
return function(SAKURA)
    local sakura = { spot = 0, script_wait_period = 700 }
    local kero = { spot = 1, script_wait_normal = 30, budoux = { 6 }, budoux_jump = { 6, 4 } }
    local inputs = {
        "こんにちは。",
        "一行目\n二行目、三行目！",
        "えっと…まあ、いいか？",
        "私は今日学校へ行って友達と一緒に勉強をしました",
        "Hello, world. 半角も混ぜる!",
        "\\w5待って\\nから「続き」。",
        "",
    }
    local out = {}
    for _, text in ipairs(inputs) do
        for _, actor in ipairs({ false, sakura, kero, sakura }) do
            out[#out + 1] = SAKURA.talk_to_script(actor or nil, text)
        end
    end
    return out
end
"#;

const PASTA_TOML: &str = r#"[loader]
debug_mode = true

[talk]
script_wait_normal = 40
script_wait_comma = 300
chars_leader = "…"
"#;

fn collect(t: Table) -> Vec<Vec<u8>> {
    t.sequence_values::<mlua::LuaString>()
        .map(|s| s.unwrap().as_bytes().to_vec())
        .collect()
}

#[test]
fn talk_to_script_from_load_is_byte_identical_to_direct_register() {
    let temp = copy_fixture_to_temp("minimal");
    std::fs::write(temp.path().join("pasta.toml"), PASTA_TOML).unwrap();

    let runtime = PastaLoader::load(temp.path()).unwrap();
    let loaded = runtime
        .exec(&format!(
            "local f = (function() {CASES} end)()\nreturn f(require \"@pasta_sakura_script\")"
        ))
        .unwrap();
    let Value::Table(loaded) = loaded else {
        panic!("expected table, got {loaded:?}");
    };

    let talk = PastaConfig::load(temp.path()).unwrap().talk();
    let lua = Lua::new();
    let module = pasta_lua::sakura_script::register(&lua, talk.as_ref()).unwrap();
    let f: mlua::Function = lua.load(CASES).eval().unwrap();
    let direct: Table = f.call(module).unwrap();

    let (loaded, direct) = (collect(loaded), collect(direct));
    assert_eq!(loaded.len(), 28);
    for (i, (a, b)) in loaded.iter().zip(&direct).enumerate() {
        assert_eq!(
            a,
            b,
            "case {i}: {:?} vs {:?}",
            String::from_utf8_lossy(a),
            String::from_utf8_lossy(b)
        );
    }
    assert_eq!(loaded.len(), direct.len());
}
