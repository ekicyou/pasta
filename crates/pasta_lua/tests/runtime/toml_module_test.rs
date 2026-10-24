//! Integration tests for the `@toml` module (mlua-stdlib-removal 2.1).

use crate::common::{copy_fixture_to_temp, create_empty_context};

use pasta_lua::loader::PastaLoader;
use pasta_lua::{PastaLuaRuntime, RuntimeConfig};
use tempfile::TempDir;

#[test]
fn test_toml_decode_available_in_minimal_config() {
    let runtime =
        PastaLuaRuntime::with_config(create_empty_context(), RuntimeConfig::minimal()).unwrap();

    let result = runtime
        .exec(
            r#"
            local t = require("@toml").decode('a = 1')
            return t.a == 1 and next(t, next(t)) == nil
        "#,
        )
        .unwrap();

    assert_eq!(result.as_boolean(), Some(true));
}

#[test]
fn test_toml_decode_error_contract() {
    let runtime =
        PastaLuaRuntime::with_config(create_empty_context(), RuntimeConfig::minimal()).unwrap();

    let result = runtime
        .exec(
            r#"
            local toml = require "@toml"
            local t1, e1 = toml.decode('a = ')
            local t2, e2 = toml.decode('a = "\255"')
            local ok = pcall(toml.decode, 42)
            local d = toml.decode('d = 1979-05-27 07:32:00').d
            return t1 == nil and #e1 > 0 and t2 == nil and #e2 > 0 and not ok
                and d == '1979-05-27T07:32:00'
        "#,
        )
        .unwrap();

    assert_eq!(result.as_boolean(), Some(true));
}

#[test]
fn test_toml_module_shape_and_independence() {
    let runtime =
        PastaLuaRuntime::with_config(create_empty_context(), RuntimeConfig::minimal()).unwrap();

    let result = runtime
        .exec(
            r#"
            local toml = require "@toml"
            local keys = {}
            for k in pairs(toml) do keys[#keys + 1] = k end
            local a = toml.decode('x = 1')
            local b = toml.decode('y = 2')
            return toml.encode == nil and #keys == 1 and keys[1] == "decode"
                and require "@toml" == toml
                and a.x == 1 and a.y == nil and b.y == 2 and b.x == nil
                and not pcall(toml.decode, nil) and not pcall(toml.decode, {})
        "#,
        )
        .unwrap();

    assert_eq!(result.as_boolean(), Some(true));
}

/// 文字列・整数・小数・真偽値・日付・配列・入れ子の表（pasta.toml の `[sample]` と decode の両方に使う）。
const SAMPLE: &str = r#"s = "ぱすた"
i = 3
f = 1.5
b = true
d = 1979-05-27 07:32:00
arr = ["a", "b"]
nested = { k = 1, deep = { x = "y" } }
"#;

/// `minimal` の一時ゴーストに `[sample]` と `scripts/main.lua` を足して読み込む。
fn load_ghost(main_lua: &str) -> (TempDir, PastaLuaRuntime) {
    let temp = copy_fixture_to_temp("minimal");
    let toml_path = temp.path().join("pasta.toml");
    let base = std::fs::read_to_string(&toml_path).unwrap();
    std::fs::write(&toml_path, format!("{base}\n[sample]\n{SAMPLE}")).unwrap();
    let scripts = temp.path().join("scripts");
    std::fs::create_dir_all(&scripts).unwrap();
    std::fs::write(scripts.join("main.lua"), main_lua).unwrap();
    let runtime = PastaLoader::load(temp.path()).unwrap();
    (temp, runtime)
}

#[test]
fn test_toml_decode_matches_pasta_config() {
    let (_temp, runtime) = load_ghost("");

    let result = runtime
        .exec(&format!(
            r#"
            local function same(a, b)
                if type(a) ~= type(b) then return false end
                if type(a) ~= "table" then return a == b end
                for k, v in pairs(a) do if not same(v, b[k]) then return false end end
                for k in pairs(b) do if a[k] == nil then return false end end
                return true
            end
            local t = require("@toml").decode([==[{SAMPLE}]==])
            return same(t, require("@pasta_config").sample)
                and t.s == "ぱすた" and t.i == 3 and t.f == 1.5 and t.b == true
                and t.d == "1979-05-27T07:32:00" and t.arr[1] == "a" and t.arr[2] == "b"
                and t.nested.k == 1 and t.nested.deep.x == "y"
        "#
        ))
        .unwrap();

    assert_eq!(result.as_boolean(), Some(true));
}

#[test]
fn test_toml_requirable_at_top_of_main_lua() {
    let (_temp, runtime) = load_ghost(
        r#"local toml = require "@toml"
MAIN_TOML_A = toml.decode('a = 1').a
"#,
    );

    let result = runtime.exec("return MAIN_TOML_A").unwrap();

    assert_eq!(result.as_integer(), Some(1));
}
