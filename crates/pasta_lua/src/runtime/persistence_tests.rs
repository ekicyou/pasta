use super::*;
use tempfile::TempDir;

fn create_test_config(temp_dir: &TempDir, obfuscate: bool) -> (PersistenceConfig, PathBuf) {
    let file_name = if obfuscate { "save.dat" } else { "save.toml" };
    let config = PersistenceConfig {
        obfuscate,
        file_path: file_name.to_string(),
        debug_mode: true,
    };
    let base_dir = temp_dir.path().to_path_buf();
    (config, base_dir)
}

/// TOML の文字列から文書を作る。
fn doc(src: &str) -> toml::Table {
    toml::from_str(src).unwrap()
}

#[test]
fn test_description_is_toml() {
    let temp_dir = TempDir::new().unwrap();
    let (config, base_dir) = create_test_config(&temp_dir, false);
    let lua = Lua::new();
    let module = register(&lua, &config, &base_dir).unwrap();
    let desc: String = module.get("_DESCRIPTION").unwrap();
    assert_eq!(desc, "Persistent data storage (TOML/gzip)");
}

#[test]
fn test_save_load_toml() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("save.toml");

    let data = doc(r#"
player_name = "Alice"
play_count = 42
[flags]
tutorial_complete = true
"#);

    save_to_file(&data, &file_path, false).unwrap();

    // 素の TOML の文字列として読める
    let text = fs::read_to_string(&file_path).unwrap();
    assert_eq!(toml::from_str::<toml::Table>(&text).unwrap(), data);

    let loaded = load_from_file(&file_path).unwrap();
    assert_eq!(loaded, data);
}

#[test]
fn test_save_load_obfuscated() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("save.dat");

    let data = doc(r#"
player_name = "Bob"
inventory = ["sword", "shield"]
"#);

    // Save with obfuscation
    save_to_file(&data, &file_path, true).unwrap();

    // Verify file starts with gzip magic
    let raw = fs::read(&file_path).unwrap();
    assert!(raw.len() >= 2);
    assert_eq!(raw[0], GZIP_MAGIC[0]);
    assert_eq!(raw[1], GZIP_MAGIC[1]);

    // Load
    let loaded = load_from_file(&file_path).unwrap();
    assert_eq!(loaded, data);
}

#[test]
fn test_load_nonexistent_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("nonexistent.toml");

    let result = load_from_file(&file_path);
    assert!(matches!(result, Err(PersistenceError::IoError(_))));
}

#[test]
fn test_load_corrupted_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("corrupted.toml");

    // 古い JSON も TOML としては読めない
    fs::write(&file_path, r#"{ "invalid": "json" }"#).unwrap();

    let result = load_from_file(&file_path);
    assert!(matches!(result, Err(PersistenceError::TomlParseError(_))));
}

#[test]
fn test_auto_detect_format() {
    let temp_dir = TempDir::new().unwrap();

    let data = doc(r#"key = "value""#);

    // obfuscate に関係なく、どちらの形も読める
    let plain_path = temp_dir.path().join("test.toml");
    save_to_file(&data, &plain_path, false).unwrap();
    let gzip_path = temp_dir.path().join("test.dat");
    save_to_file(&data, &gzip_path, true).unwrap();

    assert_eq!(load_from_file(&plain_path).unwrap(), data);
    assert_eq!(load_from_file(&gzip_path).unwrap(), data);
}

#[test]
fn test_atomic_write_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir
        .path()
        .join("subdir")
        .join("nested")
        .join("save.toml");

    let data = doc("test = true");

    // Should create directories automatically
    save_to_file(&data, &file_path, false).unwrap();

    assert!(file_path.exists());
    // 一時ファイルは名前の置き換えで残らない
    assert!(!file_path.with_extension("tmp").exists());
    let loaded = load_from_file(&file_path).unwrap();
    assert_eq!(loaded, data);
}

#[test]
fn test_atomic_write_failure_removes_temp_file() {
    let temp_dir = TempDir::new().unwrap();
    // 保存先にフォルダがあると rename が失敗する
    let file_path = temp_dir.path().join("save.toml");
    fs::create_dir(&file_path).unwrap();

    let result = save_to_file(&doc("a = 1"), &file_path, false);
    assert!(matches!(result, Err(PersistenceError::IoError(_))));
    assert!(!file_path.with_extension("tmp").exists());
    assert!(file_path.is_dir());
}

#[test]
fn test_nested_table_serialization() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("nested.toml");

    let data = doc(r#"
[level1.level2.level3]
value = 123
array = [1, 2, 3]
bool = true
string = "nested"
"#);

    save_to_file(&data, &file_path, false).unwrap();
    let loaded = load_from_file(&file_path).unwrap();
    assert_eq!(loaded, data);
}

#[test]
fn test_empty_file_returns_empty_object() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty.toml");

    // Create empty file
    fs::write(&file_path, "").unwrap();

    let loaded = load_from_file(&file_path).unwrap();
    assert!(loaded.is_empty());
}

#[test]
fn test_lua_module_load() {
    let temp_dir = TempDir::new().unwrap();
    let (config, base_dir) = create_test_config(&temp_dir, false);

    let lua = Lua::new();
    let module = register(&lua, &config, &base_dir).unwrap();

    // Load should return empty table when file doesn't exist
    let load_fn: mlua::Function = module.get("load").unwrap();
    let result: Table = load_fn.call(()).unwrap();

    // Should be an empty table
    assert_eq!(result.len().unwrap(), 0);
}

#[test]
fn test_lua_module_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let (config, base_dir) = create_test_config(&temp_dir, false);

    let lua = Lua::new();
    let module = register(&lua, &config, &base_dir).unwrap();

    // Create test data
    let data: Table = lua.create_table().unwrap();
    data.set("name", "Test").unwrap();
    data.set("count", 42).unwrap();

    // Save
    let save_fn: mlua::Function = module.get("save").unwrap();
    let (ok, err): (Option<bool>, Option<String>) = save_fn.call(data.clone()).unwrap();
    assert_eq!(ok, Some(true));
    assert!(err.is_none());

    // Load
    let load_fn: mlua::Function = module.get("load").unwrap();
    let result: Table = load_fn.call(()).unwrap();

    // Verify data
    let name: String = result.get("name").unwrap();
    let count: i32 = result.get("count").unwrap();
    assert_eq!(name, "Test");
    assert_eq!(count, 42);
}

#[test]
fn test_register_rejects_directory_traversal() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().to_path_buf();

    let config = PersistenceConfig {
        obfuscate: false,
        file_path: "../../etc/malicious.json".to_string(),
        debug_mode: false,
    };

    let lua = Lua::new();
    let result = register(&lua, &config, &base_dir);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("directory traversal"),
        "Expected traversal error, got: {}",
        err_msg
    );
}

#[test]
fn test_register_rejects_rooted_path() {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().to_path_buf();

    // "/" prefix has RootDir component on all platforms
    let config = PersistenceConfig {
        obfuscate: false,
        file_path: "/etc/passwd".to_string(),
        debug_mode: false,
    };

    let lua = Lua::new();
    let result = register(&lua, &config, &base_dir);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("directory traversal"),
        "Expected traversal error, got: {}",
        err_msg
    );
}

// ---------------------------------------------------------------------------
// SaveCodec（Lua の値 → TOML の値）
// ---------------------------------------------------------------------------

/// Lua のコードが返す表を外すモードで変換する。外したキーは道筋の順に並べる。
fn codec_doc(code: &str) -> (toml::Table, Vec<(String, String)>) {
    let lua = Lua::new();
    let root: Table = lua.load(code).eval().unwrap();
    let (doc, dropped) = lua_table_to_toml_document(&lua, &root);
    let mut dropped: Vec<_> = dropped.into_iter().map(|k| (k.path, k.reason)).collect();
    dropped.sort();
    (doc, dropped)
}

/// Lua のコードが返す値を厳しいモードで変換する。
fn codec_strict(code: &str, max_depth: usize) -> Result<toml::Value, String> {
    let lua = Lua::new();
    let value: Value = lua.load(code).eval().unwrap();
    lua_value_to_toml_strict(&lua, &value, max_depth)
}

/// 根を 1 段目として n 段の入れ子を作る Lua の関数。
const NEST: &str = r#"
local function nest(n)
    local root = {}
    local t = root
    for _ = 2, n do
        local c = {}
        t.k = c
        t = c
    end
    t.leaf = 1
    return root
end
"#;

fn dropped(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(p, r)| (p.to_string(), r.to_string()))
        .collect()
}

#[test]
fn test_codec_writable_values() {
    let (doc, dropped) = codec_doc(
        r#"return {
            s = "あ\n\t", b = true, i = 42, f = 1.5, nan = 0/0, inf = 1/0, ninf = -1/0,
            e = {}, arr = { 1, "x", { k = 1 } }, t = { a = 1 },
        }"#,
    );
    assert!(dropped.is_empty(), "{dropped:?}");
    assert_eq!(doc["s"], toml::Value::String("あ\n\t".into()));
    assert_eq!(doc["b"], toml::Value::Boolean(true));
    assert_eq!(doc["i"], toml::Value::Integer(42));
    assert_eq!(doc["f"], toml::Value::Float(1.5));
    assert!(doc["nan"].as_float().unwrap().is_nan());
    assert_eq!(doc["inf"], toml::Value::Float(f64::INFINITY));
    assert_eq!(doc["ninf"], toml::Value::Float(f64::NEG_INFINITY));
    assert_eq!(doc["e"], toml::Value::Table(toml::Table::new()));
    let arr = doc["arr"].as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0], toml::Value::Integer(1));
    assert_eq!(arr[1], toml::Value::String("x".into()));
    assert_eq!(arr[2]["k"], toml::Value::Integer(1));
    assert_eq!(doc["t"]["a"], toml::Value::Integer(1));
    // 戻りの表は必ず書ける
    toml::to_string(&doc).unwrap();
}

#[test]
fn test_codec_array_marked_empty_table_is_empty_array() {
    let lua = Lua::new();
    let root = lua.create_table().unwrap();
    let marked = lua.create_table().unwrap();
    marked.set_metatable(Some(lua.array_metatable())).unwrap();
    root.set("list", marked).unwrap();
    root.set("plain", lua.create_table().unwrap()).unwrap();
    let (doc, dropped) = lua_table_to_toml_document(&lua, &root);
    assert!(dropped.is_empty());
    assert_eq!(doc["list"], toml::Value::Array(vec![]));
    assert_eq!(doc["plain"], toml::Value::Table(toml::Table::new()));
}

#[test]
fn test_codec_unwritable_values_reasons() {
    let (doc, got) = codec_doc(
        r#"return {
            bad_str = "\255",
            hole = { 1, nil, 3 },
            mixed = { 1, 2, x = 3 },
            int_key = { [5] = "x" },
            bool_key = { [true] = 1 },
            float_key = { [1.5] = 1 },
            table_key = { [{}] = 1 },
            fn = function() end,
            co = coroutine.create(function() end),
            ok = 1,
        }"#,
    );
    assert_eq!(
        got,
        dropped(&[
            ("bad_str", "string is not valid UTF-8"),
            ("bool_key", "non-string key (boolean)"),
            ("co", "unsupported value type (thread)"),
            ("float_key", "non-string key (number)"),
            ("fn", "unsupported value type (function)"),
            ("hole", "hole in array"),
            ("int_key", "non-string key (integer)"),
            ("mixed", "table mixes array items and named keys"),
            ("table_key", "non-string key (table)"),
        ])
    );
    assert_eq!(doc.len(), 1);
    assert_eq!(doc["ok"], toml::Value::Integer(1));
}

#[test]
fn test_codec_userdata_values_are_unsupported() {
    let lua = Lua::new();
    let ud = Value::UserData(lua.create_any_userdata(0u8).unwrap());
    assert_eq!(
        lua_value_to_toml_strict(&lua, &ud, SAVE_MAX_DEPTH),
        Err("unsupported value type (userdata)".to_string())
    );
    let light = Value::LightUserData(mlua::LightUserData(std::ptr::null_mut()));
    assert_eq!(
        lua_value_to_toml_strict(&lua, &light, SAVE_MAX_DEPTH),
        Err("unsupported value type (lightuserdata)".to_string())
    );
}

#[test]
fn test_codec_drops_only_nearest_string_key() {
    let (doc, got) = codec_doc(r#"return { flags = { secret = function() end, other = true } }"#);
    assert_eq!(
        got,
        dropped(&[("flags.secret", "unsupported value type (function)")])
    );
    assert_eq!(doc["flags"]["other"], toml::Value::Boolean(true));
    assert!(doc["flags"].get("secret").is_none());
}

#[test]
fn test_codec_drops_key_inside_array_item() {
    let (doc, got) = codec_doc(
        r#"return { inventory = { { name = "剣", f = function() end }, { name = "盾" } } }"#,
    );
    assert_eq!(
        got,
        dropped(&[("inventory[1].f", "unsupported value type (function)")])
    );
    let inv = doc["inventory"].as_array().unwrap();
    assert_eq!(inv.len(), 2);
    assert_eq!(inv[0]["name"], toml::Value::String("剣".into()));
    assert!(inv[0].get("f").is_none());
    assert_eq!(inv[1]["name"], toml::Value::String("盾".into()));
}

#[test]
fn test_codec_drops_whole_array_when_item_unwritable() {
    let (doc, got) =
        codec_doc(r#"return { inventory = { { f = function() end }, function() end }, keep = 1 }"#);
    // 配列の中で先に足した inventory[1].f は一覧に残らない
    assert_eq!(
        got,
        dropped(&[("inventory", "unsupported value type (function)")])
    );
    assert!(doc.get("inventory").is_none());
    assert_eq!(doc["keep"], toml::Value::Integer(1));
}

#[test]
fn test_codec_depth_32_writes_and_33rd_key_is_dropped() {
    let (doc, got) = codec_doc(&format!("{NEST} return nest(32)"));
    assert!(got.is_empty(), "{got:?}");
    let mut t = &doc;
    for _ in 2..=32 {
        t = t["k"].as_table().unwrap();
    }
    assert_eq!(t["leaf"], toml::Value::Integer(1));

    let (_, got) = codec_doc(&format!("{NEST} return nest(33)"));
    let path = vec!["k"; 32].join(".");
    assert_eq!(
        got,
        dropped(&[(path.as_str(), "nested deeper than 32 levels")])
    );
}

#[test]
fn test_codec_cycle_is_recursive_table() {
    let (doc, got) = codec_doc("local t = { a = 1 } t.me = t t.x = { up = t } return t");
    assert_eq!(
        got,
        dropped(&[("me", "recursive table"), ("x.up", "recursive table")])
    );
    assert_eq!(doc["a"], toml::Value::Integer(1));
    assert_eq!(doc["x"], toml::Value::Table(toml::Table::new()));
}

#[test]
fn test_codec_shared_table_is_written_twice() {
    let (doc, got) = codec_doc("local s = { v = 1 } return { a = s, b = s }");
    assert!(got.is_empty(), "{got:?}");
    assert_eq!(doc["a"]["v"], toml::Value::Integer(1));
    assert_eq!(doc["b"]["v"], toml::Value::Integer(1));
}

#[test]
fn test_codec_100000_levels_do_not_overflow() {
    let code = "local root = {} local t = root \
                for _ = 1, 100000 do local c = {} t.k = c t = c end return root";
    let (_, got) = codec_doc(code);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].1, "nested deeper than 32 levels");
    assert_eq!(
        codec_strict(code, 10),
        Err("nested deeper than 10 levels".to_string())
    );
}

#[test]
fn test_codec_root_non_string_keys_dropped_one_by_one() {
    let (doc, got) = codec_doc(r#"return { name = "x", [1] = "a", [true] = 1 }"#);
    assert_eq!(
        got,
        dropped(&[
            ("[1]", "non-string key at root (integer)"),
            ("[true]", "non-string key at root (boolean)"),
        ])
    );
    assert_eq!(doc.len(), 1);
    assert_eq!(doc["name"], toml::Value::String("x".into()));

    // 根を配列として書かない
    let (doc, got) = codec_doc(r#"return { "a", "b" }"#);
    assert!(doc.is_empty());
    assert_eq!(got.len(), 2);
}

#[test]
fn test_codec_invalid_utf8_key_drops_only_that_key() {
    let (doc, got) = codec_doc(r#"return { ["\255k"] = 1, other = { ["\255"] = 2, ok = 3 } }"#);
    assert_eq!(
        got,
        dropped(&[
            ("other.\u{FFFD}", "key is not valid UTF-8"),
            ("\u{FFFD}k", "key is not valid UTF-8"),
        ])
    );
    assert_eq!(doc["other"]["ok"], toml::Value::Integer(3));
}

#[test]
fn test_codec_key_order_does_not_change_document() {
    let (a, _) = codec_doc("local t = {} t.z = 1 t.a = 2 t.m = {} t.m.y = 1 t.m.b = 2 return t");
    let (b, _) = codec_doc("local t = {} t.m = {} t.m.b = 2 t.m.y = 1 t.a = 2 t.z = 1 return t");
    let (sa, sb) = (toml::to_string(&a).unwrap(), toml::to_string(&b).unwrap());
    assert_eq!(sa, sb);
    assert!(
        sa.find("a = 2").unwrap() < sa.find("z = 1").unwrap(),
        "{sa}"
    );
}

#[test]
fn test_codec_strict_mode_fails_whole_value() {
    let ok = codec_strict(r#"return { a = { 1, 2 }, b = "x" }"#, 10).unwrap();
    assert_eq!(ok["a"].as_array().unwrap().len(), 2);
    assert_eq!(
        codec_strict("return { ok = 1, f = function() end }", 10),
        Err("unsupported value type (function)".to_string())
    );
    assert_eq!(
        codec_strict(r#"return { ok = 1, ["\255"] = 1 }"#, 10),
        Err("key is not valid UTF-8".to_string())
    );
    assert_eq!(
        codec_strict("local t = {} t.me = t return t", 10),
        Err("recursive table".to_string())
    );
    // 表でない値もそのまま変換する
    assert_eq!(codec_strict("return 3", 10), Ok(toml::Value::Integer(3)));
    // ログの 10 段（表そのものを 1 段目）
    assert!(codec_strict(&format!("{NEST} return nest(10)"), 10).is_ok());
    assert_eq!(
        codec_strict(&format!("{NEST} return nest(11)"), 10),
        Err("nested deeper than 10 levels".to_string())
    );
}

// ---------------------------------------------------------------------------
// SaveCodec（TOML の値 → Lua の値）と往復
// ---------------------------------------------------------------------------

/// TOML の文書を Lua の表に戻し、もう一度 TOML の文字列にして読み直す。
fn codec_round_trip_doc(lua: &Lua, doc: &toml::Table) -> toml::Table {
    let value = toml_to_lua(lua, &toml::Value::Table(doc.clone())).unwrap();
    let Value::Table(root) = value else {
        panic!("table expected: {value:?}")
    };
    let (back, dropped) = lua_table_to_toml_document(lua, &root);
    assert!(dropped.is_empty(), "{:?}", dropped);
    let text = toml::to_string(&back).unwrap();
    toml::from_str::<toml::Table>(&text).unwrap()
}

/// ファイルを通す往復: Lua の表 → save_to_file → load_from_file → toml_to_lua → TOML の文書。
fn file_round_trip_doc(lua: &Lua, doc: &toml::Table, obfuscate: bool) -> toml::Table {
    let Value::Table(root) = toml_to_lua(lua, &toml::Value::Table(doc.clone())).unwrap() else {
        panic!()
    };
    let (written, dropped) = lua_table_to_toml_document(lua, &root);
    assert!(dropped.is_empty(), "{:?}", dropped);
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("save.toml");
    save_to_file(&written, &path, obfuscate).unwrap();
    let loaded = load_from_file(&path).unwrap();
    let Value::Table(t) = toml_to_lua(lua, &toml::Value::Table(loaded)).unwrap() else {
        panic!()
    };
    let (back, dropped) = lua_table_to_toml_document(lua, &t);
    assert!(dropped.is_empty(), "{:?}", dropped);
    back
}

#[test]
fn test_codec_read_value_types() {
    let lua = Lua::new();
    let v = |t: toml::Value| toml_to_lua(&lua, &t).unwrap();
    #[cfg(target_pointer_width = "64")]
    {
        assert!(matches!(
            v(toml::Value::Integer(i64::MAX)),
            Value::Integer(i64::MAX)
        ));
        assert!(matches!(
            v(toml::Value::Integer(i64::MIN)),
            Value::Integer(i64::MIN)
        ));
    }
    // x86 の Lua の整数は 32 ビット。収まらない整数は浮動小数の数で渡る
    #[cfg(target_pointer_width = "32")]
    {
        assert!(matches!(
            v(toml::Value::Integer(i64::MAX)),
            Value::Number(f) if f == 9223372036854775808.0
        ));
        assert!(matches!(
            v(toml::Value::Integer(i64::MIN)),
            Value::Number(f) if f == -9223372036854775808.0
        ));
    }
    assert!(matches!(v(toml::Value::Float(1.5)), Value::Number(f) if f == 1.5));
    assert!(matches!(
        v(toml::Value::Boolean(true)),
        Value::Boolean(true)
    ));
    match v(toml::Value::String("あ\n\t".into())) {
        Value::String(s) => assert_eq!(s.to_str().unwrap(), "あ\n\t"),
        other => panic!("{other:?}"),
    }
    // 配列は 1 から順の表で、配列の印が付く
    let arr = toml::Value::Array(vec![toml::Value::Integer(7), toml::Value::Integer(8)]);
    let Value::Table(t) = v(arr) else { panic!() };
    assert_eq!(t.raw_get::<i64>(1).unwrap(), 7);
    assert_eq!(t.raw_get::<i64>(2).unwrap(), 8);
    assert_eq!(t.raw_len(), 2);
    assert_eq!(
        t.metatable().unwrap().to_pointer(),
        lua.array_metatable().to_pointer()
    );
    // 表は文字列のキーの表で、印は付かない
    let mut tab = toml::Table::new();
    tab.insert("a".into(), toml::Value::Integer(1));
    let Value::Table(t) = v(toml::Value::Table(tab)) else {
        panic!()
    };
    assert_eq!(t.get::<i64>("a").unwrap(), 1);
    assert!(t.metatable().is_none());
}

#[test]
fn test_codec_round_trip_values() {
    let lua = Lua::new();
    let src = r#"
s = "あいう"
ctl = "a\nb\r\nc\td"
imin = -9223372036854775808
imax = 9223372036854775807
f = 1.5
nz = -0.0
nan = nan
inf = inf
ninf = -inf
big = 9223372036854775808.0
yes = true
no = false
empty_table = {}
empty_array = []
arr = [1, "x", { k = 1 }]
nested = { a = { b = { c = "deep" } } }
"#;
    let doc: toml::Table = toml::from_str(src).unwrap();
    // メモリの中・素のファイル・gzip のファイルの 3 通りで同じ値に戻る
    for back in [
        codec_round_trip_doc(&lua, &doc),
        file_round_trip_doc(&lua, &doc, false),
        file_round_trip_doc(&lua, &doc, true),
    ] {
        assert_eq!(back.len(), doc.len());
        for key in [
            "s",
            "ctl",
            "imin",
            "f",
            "inf",
            "ninf",
            "yes",
            "no",
            "empty_table",
            "empty_array",
            "arr",
            "nested",
        ] {
            assert_eq!(back[key], doc[key], "{key}");
        }
        assert_eq!(back["ctl"].as_str(), Some("a\nb\r\nc\td"));
        assert_eq!(back["empty_array"], toml::Value::Array(vec![]));
        let nz = back["nz"].as_float().unwrap();
        assert_eq!(nz.to_bits(), (-0.0f64).to_bits());
        assert!(back["nan"].as_float().unwrap().is_nan());
        // 表に入った i64::MAX は LuaJIT の倍精度で 2^63 に丸まる（確定した設計判断 6。値そのものの往復は別のテスト）
        assert_eq!(back["imax"], toml::Value::Float(9223372036854775808.0));
        // 2^63 は i64 の外なので浮動小数点数のまま
        assert_eq!(back["big"], toml::Value::Float(9223372036854775808.0));
    }
}

#[test]
fn test_codec_round_trip_i64_extremes_as_values() {
    // LuaJIT の表に入れると 2^53 を超える整数は丸まる（マニュアルに書く）。
    // 値そのものは Integer で戻り、Integer で書き戻る（x86 の i64::MAX だけは下の別の期待）
    let lua = Lua::new();
    let back = |i: i64| {
        let v = toml_to_lua(&lua, &toml::Value::Integer(i)).unwrap();
        lua_value_to_toml_strict(&lua, &v, SAVE_MAX_DEPTH).unwrap()
    };
    assert_eq!(back(i64::MIN), toml::Value::Integer(i64::MIN));
    #[cfg(target_pointer_width = "64")]
    assert_eq!(back(i64::MAX), toml::Value::Integer(i64::MAX));
    // x86 の Lua の値は i64::MAX を整数で持てず 2^63 の浮動小数になり、そのまま浮動小数で書き戻る
    #[cfg(target_pointer_width = "32")]
    assert_eq!(back(i64::MAX), toml::Value::Float(9223372036854775808.0));
    let v = toml_to_lua(&lua, &toml::Value::Float(-0.0)).unwrap();
    let back = lua_value_to_toml_strict(&lua, &v, SAVE_MAX_DEPTH).unwrap();
    assert!(back.as_float().unwrap().is_sign_negative());
}

#[test]
fn test_codec_datetime_comes_back_as_string() {
    let lua = Lua::new();
    let src = r#"
odt = 1979-05-27T07:32:00Z
ldt = 1979-05-27T07:32:00
ld = 1979-05-27
lt = 07:32:00
"#;
    let doc: toml::Table = toml::from_str(src).unwrap();
    let Value::Table(t) = toml_to_lua(&lua, &toml::Value::Table(doc.clone())).unwrap() else {
        panic!()
    };
    assert_eq!(t.get::<String>("odt").unwrap(), "1979-05-27T07:32:00Z");
    assert_eq!(t.get::<String>("ldt").unwrap(), "1979-05-27T07:32:00");
    assert_eq!(t.get::<String>("ld").unwrap(), "1979-05-27");
    assert_eq!(t.get::<String>("lt").unwrap(), "07:32:00");
    // もう一度 TOML にすると文字列として書かれる
    let back = codec_round_trip_doc(&lua, &doc);
    for (key, s) in [
        ("odt", "1979-05-27T07:32:00Z"),
        ("ldt", "1979-05-27T07:32:00"),
        ("ld", "1979-05-27"),
        ("lt", "07:32:00"),
    ] {
        assert_eq!(back[key], toml::Value::String(s.into()), "{key}");
    }
}

// ---------------------------------------------------------------------------
// 読み込みと退避（read_or_quarantine）・SaveGate
// ---------------------------------------------------------------------------

fn gate(lua: &Lua) -> Option<SaveGate> {
    lua.app_data_ref::<SaveGate>().map(|g| g.clone())
}

#[test]
fn test_read_toml_plain_and_gzip() {
    let temp_dir = TempDir::new().unwrap();
    let plain = temp_dir.path().join("save.toml");
    fs::write(&plain, "name = \"Alice\"\n[flags]\ndone = true\n").unwrap();
    let doc = load_from_file(&plain).unwrap();
    assert_eq!(doc["name"].as_str(), Some("Alice"));
    assert_eq!(doc["flags"]["done"].as_bool(), Some(true));

    let gz = temp_dir.path().join("save.dat");
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(b"count = 3\n").unwrap();
    fs::write(&gz, encoder.finish().unwrap()).unwrap();
    assert_eq!(load_from_file(&gz).unwrap()["count"].as_integer(), Some(3));
}

#[tracing_test::traced_test]
#[test]
fn test_read_or_quarantine_broken_toml_moves_to_broken() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("save.toml");
    fs::write(&path, "this is = = not toml").unwrap();

    let lua = Lua::new();
    let doc = read_or_quarantine(&lua, &path);
    assert!(doc.is_empty());
    assert!(!path.exists());
    let broken = temp_dir.path().join("save.toml.broken");
    assert_eq!(fs::read_to_string(&broken).unwrap(), "this is = = not toml");
    assert_eq!(gate(&lua), Some(SaveGate::Open));
    assert!(logs_contain(
        "Unreadable persistence file moved aside, using empty table"
    ));
}

#[test]
fn test_read_or_quarantine_picks_first_free_broken_name() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("save.toml");
    let b0 = temp_dir.path().join("save.toml.broken");
    let b1 = temp_dir.path().join("save.toml.broken.1");
    fs::write(&path, "[[[").unwrap();
    fs::write(&b0, "old0").unwrap();
    fs::write(&b1, "old1").unwrap();

    let lua = Lua::new();
    assert!(read_or_quarantine(&lua, &path).is_empty());
    assert_eq!(
        fs::read_to_string(temp_dir.path().join("save.toml.broken.2")).unwrap(),
        "[[["
    );
    assert_eq!(fs::read_to_string(&b0).unwrap(), "old0");
    assert_eq!(fs::read_to_string(&b1).unwrap(), "old1");
    assert_eq!(gate(&lua), Some(SaveGate::Open));
}

#[tracing_test::traced_test]
#[test]
fn test_read_or_quarantine_missing_file_warns_and_opens() {
    let temp_dir = TempDir::new().unwrap();
    let lua = Lua::new();
    let doc = read_or_quarantine(&lua, &temp_dir.path().join("save.toml"));
    assert!(doc.is_empty());
    assert_eq!(gate(&lua), Some(SaveGate::Open));
    assert!(logs_contain(
        "Persistence file not found, using empty table"
    ));
    // 空のファイルの「警告なし」の確かめ方が効くことを示す
    assert!(logs_contain("WARN"));
}

#[tracing_test::traced_test]
#[test]
fn test_read_or_quarantine_empty_file_is_empty_without_warning() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("save.toml");
    fs::write(&path, "").unwrap();
    let lua = Lua::new();
    assert!(read_or_quarantine(&lua, &path).is_empty());
    assert!(path.exists());
    assert_eq!(gate(&lua), Some(SaveGate::Open));
    assert!(!logs_contain("WARN"));
    assert!(!logs_contain("ERROR"));
}

#[test]
fn test_read_or_quarantine_reads_bom_file() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("save.toml");
    fs::write(&path, "\u{FEFF}name = \"Alice\"\n").unwrap();
    let lua = Lua::new();
    let doc = read_or_quarantine(&lua, &path);
    assert_eq!(doc["name"].as_str(), Some("Alice"));
    assert!(path.exists());
    assert_eq!(gate(&lua), Some(SaveGate::Open));
}

#[test]
fn test_save_gate_closed_is_not_overwritten_by_open() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("save.toml");
    fs::write(&path, "ok = 1\n").unwrap();
    let lua = Lua::new();
    lua.set_app_data(SaveGate::Closed("earlier".into()));
    let doc = read_or_quarantine(&lua, &path);
    assert_eq!(doc["ok"].as_integer(), Some(1));
    assert_eq!(gate(&lua), Some(SaveGate::Closed("earlier".into())));
}

#[test]
fn test_save_gate_is_per_lua_state() {
    let temp_dir = TempDir::new().unwrap();
    let lua_a = Lua::new();
    let lua_b = Lua::new();
    read_or_quarantine(&lua_a, &temp_dir.path().join("save.toml"));
    assert_eq!(gate(&lua_a), Some(SaveGate::Open));
    assert_eq!(gate(&lua_b), None);
}

/// 本物の OS の失敗: 共有なしで開いたままの保存先は、読めず移せず、印が Closed になる。
#[cfg(windows)]
#[tracing_test::traced_test]
#[test]
fn test_read_or_quarantine_locked_file_closes_gate() {
    use std::os::windows::fs::OpenOptionsExt;
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("save.toml");
    fs::write(&path, "keep = \"me\"\n").unwrap();
    let lock = fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&path)
        .unwrap();

    let lua = Lua::new();
    assert!(read_or_quarantine(&lua, &path).is_empty());
    assert!(matches!(gate(&lua), Some(SaveGate::Closed(_))));
    assert!(!temp_dir.path().join("save.toml.broken").exists());
    assert!(logs_contain(
        "Unreadable persistence file could not be moved aside; saving is disabled for this runtime"
    ));

    drop(lock);
    assert_eq!(fs::read_to_string(&path).unwrap(), "keep = \"me\"\n");
}

// ---------------------------------------------------------------------------
// 保存（save_table）と save() の戻り値
// ---------------------------------------------------------------------------

/// save.toml を保存先にしたモジュールの save を呼ぶ。
fn call_save(lua: &Lua, temp_dir: &TempDir, code: &str) -> (Option<bool>, Option<String>) {
    let (config, base_dir) = create_test_config(temp_dir, false);
    let module = register(lua, &config, &base_dir).unwrap();
    let save_fn: mlua::Function = module.get("save").unwrap();
    let data: Table = lua.load(code).eval().unwrap();
    save_fn.call(data).unwrap()
}

#[test]
fn test_save_gate_closed_save_returns_error_and_writes_nothing() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("save.toml");
    fs::write(&path, "keep = 1\n").unwrap();
    let lua = Lua::new();
    lua.set_app_data(SaveGate::Closed("locked".into()));

    let (ok, err) = call_save(&lua, &temp_dir, "return { a = 1 }");
    assert_eq!(ok, None);
    let err = err.unwrap();
    assert!(err.starts_with("Failed to save: "), "{err}");
    assert!(
        err.contains("Saving is disabled for this runtime: locked"),
        "{err}"
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), "keep = 1\n");
    assert!(!path.with_extension("tmp").exists());
    assert!(matches!(
        save_table(&lua, &lua.create_table().unwrap(), &path, false),
        Err(PersistenceError::SaveDisabled(_))
    ));
}

#[test]
fn test_save_without_gate_quarantines_broken_file_first() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("save.toml");
    fs::write(&path, r#"{"old": "json"}"#).unwrap();
    let lua = Lua::new();

    let (ok, err) = call_save(&lua, &temp_dir, r#"return { name = "新" }"#);
    assert_eq!((ok, err), (Some(true), None));
    assert_eq!(
        fs::read_to_string(temp_dir.path().join("save.toml.broken")).unwrap(),
        r#"{"old": "json"}"#
    );
    assert_eq!(load_from_file(&path).unwrap()["name"].as_str(), Some("新"));
    assert_eq!(gate(&lua), Some(SaveGate::Open));
}

#[tracing_test::traced_test]
#[test]
fn test_save_drops_unwritable_key_and_returns_true() {
    let temp_dir = TempDir::new().unwrap();
    let lua = Lua::new();
    let (ok, err) = call_save(
        &lua,
        &temp_dir,
        "return { ok = 1, f = function() end, flags = { secret = function() end, other = true } }",
    );
    assert_eq!((ok, err), (Some(true), None));
    let saved = load_from_file(&temp_dir.path().join("save.toml")).unwrap();
    assert_eq!(saved["ok"].as_integer(), Some(1));
    assert!(saved.get("f").is_none());
    assert_eq!(saved["flags"]["other"].as_bool(), Some(true));
    assert!(saved["flags"].get("secret").is_none());
    assert!(logs_contain(
        "key 'f' was not saved: unsupported value type (function)"
    ));
    assert!(logs_contain(
        "key 'flags.secret' was not saved: unsupported value type (function)"
    ));
}

#[test]
fn test_lua_module_save_obfuscated_and_load_either_form() {
    let temp_dir = TempDir::new().unwrap();
    let lua = Lua::new();
    let (config, base_dir) = create_test_config(&temp_dir, true);
    let module = register(&lua, &config, &base_dir).unwrap();
    let save_fn: mlua::Function = module.get("save").unwrap();
    let data: Table = lua.load("return { n = 7 }").eval().unwrap();
    let (ok, _): (Option<bool>, Option<String>) = save_fn.call(data).unwrap();
    assert_eq!(ok, Some(true));
    let path = temp_dir.path().join("save.dat");
    assert!(fs::read(&path).unwrap().starts_with(&GZIP_MAGIC));

    // obfuscate = true でも素の TOML のファイルを読める
    fs::write(&path, "n = 8\n").unwrap();
    let load_fn: mlua::Function = module.get("load").unwrap();
    let t: Table = load_fn.call(()).unwrap();
    assert_eq!(t.get::<i64>("n").unwrap(), 8);
}

/// 本物の OS の失敗: 共有なしで開いたままの保存先へ save() すると、nil, err で中身が変わらない。
#[cfg(windows)]
#[test]
fn test_save_locked_file_returns_error_and_keeps_content() {
    use std::os::windows::fs::OpenOptionsExt;
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("save.toml");
    fs::write(&path, "keep = \"me\"\n").unwrap();
    let lock = fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&path)
        .unwrap();

    let lua = Lua::new();
    let (ok, err) = call_save(&lua, &temp_dir, "return { a = 1 }");
    assert_eq!(ok, None);
    assert!(err.unwrap().starts_with("Failed to save: "));
    assert!(matches!(gate(&lua), Some(SaveGate::Closed(_))));

    drop(lock);
    assert_eq!(fs::read_to_string(&path).unwrap(), "keep = \"me\"\n");
    assert!(!path.with_extension("tmp").exists());
}

/// lua_int: 64 ビットの整数として読む決まり（x86・x64 で同じ結果）。
#[test]
fn test_lua_int_rule() {
    let p63 = 9223372036854775808.0_f64; // 2^63
    assert_eq!(lua_int(&Value::Integer(7)), Some(7));
    assert_eq!(lua_int(&Value::Number(3000000000.0)), Some(3000000000));
    assert_eq!(lua_int(&Value::Number(-3000000000.0)), Some(-3000000000));
    assert_eq!(lua_int(&Value::Number(-p63)), Some(i64::MIN));
    assert_eq!(lua_int(&Value::Number(1.0)), Some(1));
    // 2^63 は i64 に飽和させない
    assert_eq!(lua_int(&Value::Number(p63)), None);
    assert_eq!(lua_int(&Value::Number(-p63 * 2.0)), None);
    assert_eq!(lua_int(&Value::Number(-0.0)), None);
    assert_eq!(lua_int(&Value::Number(0.5)), None);
    assert_eq!(lua_int(&Value::Number(f64::NAN)), None);
    assert_eq!(lua_int(&Value::Number(f64::INFINITY)), None);
    assert_eq!(lua_int(&Value::Number(f64::NEG_INFINITY)), None);
    assert_eq!(lua_int(&Value::Boolean(true)), None);
}

/// TOML の整数が Lua の整数に収まらなければ浮動小数の数で渡す（切り詰めない）。
#[test]
fn test_toml_to_lua_integer_never_truncates() {
    let lua = Lua::new();
    let v = toml_to_lua(&lua, &toml::Value::Integer(3000000000)).unwrap();
    assert_eq!(lua_int(&v), Some(3000000000));
    let v = toml_to_lua(&lua, &toml::Value::Integer(5)).unwrap();
    assert!(matches!(v, Value::Integer(5)));
}

/// Lua から作った表を書くと、x86・x64 で同じ TOML の文字になる（Data Models の表）。
#[test]
fn test_codec_integer_text_same_on_x86_and_x64() {
    let (doc, dropped) = codec_doc(
        r#"return {
            big = 3000000000, neg = -3000000000, m63 = -2^63, p63 = 2^63,
            nz = -0.0, one = 1.0, arr = { 3000000000, 2 },
        }"#,
    );
    assert!(dropped.is_empty(), "{dropped:?}");
    let text = toml::to_string(&doc).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    lines.sort();
    assert_eq!(
        lines,
        [
            "arr = [3000000000, 2]",
            "big = 3000000000",
            "m63 = -9223372036854775808",
            "neg = -3000000000",
            "nz = -0.0",
            "one = 1",
            "p63 = 9223372036854776000.0",
        ]
    );
}

/// 32 ビットを超える整数のキーの表は、x86・x64 で同じ理由で外れる。
#[test]
fn test_codec_big_integer_keys_are_integer_keys() {
    let (doc, got) =
        codec_doc(r#"return { pos = { [3000000000] = 1 }, neg = { [-3000000000] = 1 }, ok = 1 }"#);
    assert_eq!(
        got,
        dropped(&[
            ("neg", "non-string key (integer)"),
            ("pos", "non-string key (integer)"),
        ])
    );
    assert_eq!(doc.len(), 1);
}

/// TOML の `x = 3000000000` を読むと Lua の数 3000000000 になる。
#[test]
fn test_read_big_integer_is_lua_number() {
    let lua = Lua::new();
    let doc: toml::Table = toml::from_str("x = 3000000000").unwrap();
    let t = toml_to_lua(&lua, &toml::Value::Table(doc)).unwrap();
    lua.globals().set("t", t).unwrap();
    let ok: bool = lua
        .load("return t.x == 3000000000 and t.x + 1 == 3000000001")
        .eval()
        .unwrap();
    assert!(ok);
}

/// 呼ばれたら誤りを起こす `__pairs` を持つ表（3.7）。
const PAIRS_TRAP: &str = r#"local function trap() return setmetatable({ a = 1 }, { __pairs = function() error("called") end }) end"#;

#[test]
fn test_codec_does_not_call_pairs_metamethod() {
    // 3.7: セーブとログの表は `__pairs` を呼ばず、生の中身だけを読む
    let (root, dropped) = codec_doc(&format!("{PAIRS_TRAP} return trap()"));
    assert_eq!(root, doc("a = 1"));
    assert!(dropped.is_empty(), "{dropped:?}");

    let (nested, dropped) = codec_doc(&format!("{PAIRS_TRAP} return {{ t = trap() }}"));
    assert_eq!(nested, doc("t = { a = 1 }"));
    assert!(dropped.is_empty(), "{dropped:?}");

    let log = codec_strict(&format!("{PAIRS_TRAP} return trap()"), SAVE_MAX_DEPTH).unwrap();
    assert_eq!(log, toml::Value::Table(doc("a = 1")));
}
