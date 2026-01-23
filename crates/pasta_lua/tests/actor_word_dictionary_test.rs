//! アクター単語辞書トランスパイラテスト
//!
//! タスク2.4: generate_actorの配列形式出力とcode_blocks展開を検証

use pasta_core::parse_str;
use pasta_lua::LuaTranspiler;

/// 複数値が配列形式で正しく出力されることを検証
#[test]
fn test_generate_actor_array_format() {
    let source = r#"％さくら
　＠通常：\s[0]、\s[100]、\s[200]
　＠照れ：\s[1]
"#;

    let ast = parse_str(source, "test.pasta").expect("パース成功すべし");
    let mut output = Vec::new();

    let transpiler = LuaTranspiler::default();
    transpiler
        .transpile(&ast, &mut output)
        .expect("トランスパイル成功すべし");

    let output_str = String::from_utf8(output).expect("UTF-8変換成功すべし");

    // ACTOR定義が存在すること
    assert!(
        output_str.contains(r#"PASTA.create_actor("さくら")"#),
        "アクター作成が含まれること"
    );

    // 対称API形式で出力されること（複数値）
    assert!(
        output_str.contains(r#"ACTOR:create_word("通常"):entry([=[\s[0]]=], [=[\s[100]]=], [=[\s[200]]=])"#),
        "複数値がACTOR:create_word APIで出力されること: {}",
        output_str
    );

    // 単一値も同様のAPI形式で出力されること
    assert!(
        output_str.contains(r#"ACTOR:create_word("照れ"):entry([=[\s[1]]=])"#),
        "単一値もACTOR:create_word APIで出力されること: {}",
        output_str
    );
}

/// code_blocksが正しく展開されることを検証
#[test]
fn test_generate_actor_code_blocks() {
    let source = r#"％さくら
　＠通常：\s[0]
```lua
function ACTOR.時刻(act)
    local hour = os.date("%H")
    return hour < 12 and "おはよう" or "こんにちは"
end
```
"#;

    let ast = parse_str(source, "test.pasta").expect("パース成功すべし");
    let mut output = Vec::new();

    let transpiler = LuaTranspiler::default();
    transpiler
        .transpile(&ast, &mut output)
        .expect("トランスパイル成功すべし");

    let output_str = String::from_utf8(output).expect("UTF-8変換成功すべし");

    // code_blockがdoブロック内に展開されること
    assert!(
        output_str.contains("function ACTOR.時刻(act)"),
        "関数定義が展開されること: {}",
        output_str
    );

    // 関数内容が保持されること
    assert!(
        output_str.contains("os.date"),
        "関数内容が保持されること: {}",
        output_str
    );
}

/// code_blocksがlua以外の言語は無視されることを検証
#[test]
fn test_generate_actor_ignores_non_lua_code_blocks() {
    let source = r#"％さくら
　＠通常：\s[0]
```rust
fn hello() { println!("ignored"); }
```
```lua
function ACTOR.greet(act)
    return "こんにちは"
end
```
"#;

    let ast = parse_str(source, "test.pasta").expect("パース成功すべし");
    let mut output = Vec::new();

    let transpiler = LuaTranspiler::default();
    transpiler
        .transpile(&ast, &mut output)
        .expect("トランスパイル成功すべし");

    let output_str = String::from_utf8(output).expect("UTF-8変換成功すべし");

    // Rustのコードブロックは無視されること
    assert!(
        !output_str.contains("println!"),
        "Rustコードは無視されること: {}",
        output_str
    );

    // Luaのコードブロックは展開されること
    assert!(
        output_str.contains("function ACTOR.greet(act)"),
        "Lua関数は展開されること: {}",
        output_str
    );
}

/// 単一値がACTOR:create_word APIで正しく出力されることを検証
#[test]
fn test_generate_actor_single_value_backward_compatible() {
    let source = r#"％さくら
　＠通常：\s[0]
"#;

    let ast = parse_str(source, "test.pasta").expect("パース成功すべし");
    let mut output = Vec::new();

    let transpiler = LuaTranspiler::default();
    transpiler
        .transpile(&ast, &mut output)
        .expect("トランスパイル成功すべし");

    let output_str = String::from_utf8(output).expect("UTF-8変換成功すべし");

    // 単一値もACTOR:create_word API形式で出力されること
    assert!(
        output_str.contains(r#"ACTOR:create_word("通常"):entry([=[\s[0]]=])"#),
        "単一値がACTOR:create_word APIで出力されること: {}",
        output_str
    );
}
