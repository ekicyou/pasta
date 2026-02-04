use mlua::prelude::*;
use pasta_lua::context::TranspileContext;
use pasta_lua::loader::PersistenceConfig;
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

    // Register @pasta_persistence module (required by pasta.save)
    let temp_dir = std::env::temp_dir();
    let persistence_config = PersistenceConfig::default();
    let persistence_table =
        pasta_lua::runtime::persistence::register(&lua, &persistence_config, &temp_dir)?;
    let package: LuaTable = lua.globals().get("package")?;
    let loaded: LuaTable = package.get("loaded")?;
    loaded.set("@pasta_persistence", persistence_table)?;

    // Register @pasta_search module with empty registries (required by pasta.scene)
    let ctx = TranspileContext::new();
    let search_context =
        pasta_lua::search::SearchContext::new(ctx.scene_registry, ctx.word_registry)
            .expect("Failed to create SearchContext");
    loaded.set("@pasta_search", lua.create_userdata(search_context)?)?;

    // Register @pasta_sakura_script module (required by pasta.shiori.sakura_builder)
    let sakura_module = pasta_lua::sakura_script::register(&lua, None)?;
    loaded.set("@pasta_sakura_script", sakura_module)?;

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
