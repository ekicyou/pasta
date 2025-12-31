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

    // package.path を設定（Lua モジュール解決用）
    let package_path = format!(
        "{}/?.lua;{}/?/init.lua;{}/?.lua;{}/?/init.lua",
        scripts_path.display(),
        scripts_path.display(),
        scriptlibs_path.display(),
        scriptlibs_path.display()
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

    // Lua ユニットテストを実行
    let test_file = workspace_root.join("crates/pasta_lua/tests/lua_specs/transpiler_spec.lua");
    println!("Running Lua tests from: {}", test_file.display());

    let test_code = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    // テスト実行（describe 内で自動実行される）
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
