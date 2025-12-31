use mlua::prelude::*;

#[test]
fn test_japanese_identifier_support() -> LuaResult<()> {
    let lua = Lua::new();

    // 日本語識別子のテスト
    let result: String = lua
        .load(
            r#"
        local 変数 = "日本語変数"
        local function 関数(引数)
            return "Hello, " .. 引数
        end
        return 関数(変数)
    "#,
        )
        .eval()?;

    assert_eq!(result, "Hello, 日本語変数");
    Ok(())
}

#[test]
fn test_japanese_function_names() -> LuaResult<()> {
    let lua = Lua::new();

    lua.load(
        r#"
        function 足す(甲, 乙)
            return 甲 + 乙
        end
    "#,
    )
    .exec()?;

    let func: LuaFunction = lua.globals().get("足す")?;
    let result: i32 = func.call((10, 20))?;
    assert_eq!(result, 30);
    Ok(())
}
