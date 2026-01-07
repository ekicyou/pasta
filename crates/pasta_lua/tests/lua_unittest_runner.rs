use mlua::prelude::*;
use std::path::PathBuf;

#[test]
fn run_lua_unit_tests() -> LuaResult<()> {
    let lua = Lua::new();

    // プロジェクトルートからの相対パスを設定
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let scripts_path = workspace_root.join("crates/pasta_lua/scripts");
    let scriptlibs_path = workspace_root.join("crates/pasta_lua/scriptlibs");
    let lua_specs_path = workspace_root.join("crates/pasta_lua/tests/lua_specs");

    // package.path を設定（Lua モジュール解決用）
    // lua_specs を追加して、init.lua から各 spec を require できるようにする
    let package_path = format!(
        "{}/?.lua;{}/?/init.lua;{}/?.lua;{}/?/init.lua;{}/?.lua",
        scripts_path.display(),
        scripts_path.display(),
        scriptlibs_path.display(),
        scriptlibs_path.display(),
        lua_specs_path.display()
    );

    lua.load(&format!(
        r#"
        package.path = "{};;" .. package.path
        print("Lua package.path configured:")
        print(package.path)
    "#,
        package_path.replace("\\", "\\\\")
    ))
    .exec()?;

    // Lua ユニットテストを実行（エントリーポイント: init.lua）
    let test_file = lua_specs_path.join("init.lua");
    println!("Running Lua tests from: {}", test_file.display());

    let test_code = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    // テスト実行（init.lua が各 spec を require して実行）
    match lua.load(&test_code).exec() {
        Ok(_) => {
            println!("✅ All Lua tests passed");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Lua test failed: {}", e);
            Err(e)
        }
    }
}
