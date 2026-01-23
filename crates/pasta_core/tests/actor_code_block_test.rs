//! ActorScope code_blocks パーサーテスト
//!
//! タスク1.3: code_blockを含むactor_scopeが正しくパースされることを検証

use pasta_core::parser::{FileItem, parse_str};

/// code_blockを含むactor_scopeが正しくパースされることを検証
#[test]
fn test_actor_scope_with_code_block() {
    let source = r#"％さくら
　＠通常：\s[0]
```lua
function ACTOR.時刻(act)
    return "朝"
end
```
"#;

    let ast = parse_str(source, "test.pasta").expect("パース成功すべし");

    // ActorScopeが1つあることを確認
    let actors: Vec<_> = ast
        .items
        .iter()
        .filter_map(|item| {
            if let FileItem::ActorScope(actor) = item {
                Some(actor)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(actors.len(), 1, "アクターは1つであるべし");
    let actor = actors[0];

    assert_eq!(actor.name, "さくら");
    assert_eq!(actor.words.len(), 1, "単語定義は1つ");
    assert_eq!(actor.words[0].name, "通常");

    // code_blocksが正しく格納されることを検証
    assert_eq!(actor.code_blocks.len(), 1, "code_blockは1つであるべし");

    let code_block = &actor.code_blocks[0];
    assert_eq!(
        code_block.language,
        Some("lua".to_string()),
        "言語はluaであるべし"
    );
    assert!(
        code_block.content.contains("function ACTOR.時刻"),
        "コード内容が正しいこと"
    );
}

/// 複数のcode_blocksが順序通りに格納されることを検証
#[test]
fn test_actor_scope_with_multiple_code_blocks() {
    let source = r#"％さくら
　＠通常：\s[0]
```lua
function ACTOR.時刻(act)
    return "朝"
end
```
```lua
function ACTOR.天気(act)
    return "晴れ"
end
```
"#;

    let ast = parse_str(source, "test.pasta").expect("パース成功すべし");

    let actors: Vec<_> = ast
        .items
        .iter()
        .filter_map(|item| {
            if let FileItem::ActorScope(actor) = item {
                Some(actor)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(actors.len(), 1);
    let actor = actors[0];

    assert_eq!(
        actor.code_blocks.len(),
        2,
        "code_blockは2つであるべし（順序通り）"
    );
    assert!(actor.code_blocks[0].content.contains("時刻"));
    assert!(actor.code_blocks[1].content.contains("天気"));
}

/// code_blocksがないactor_scopeも正しくパースされることを検証（後方互換性）
#[test]
fn test_actor_scope_without_code_block() {
    let source = r#"％さくら
　＠通常：\s[0]
　＠照れ：\s[1]
"#;

    let ast = parse_str(source, "test.pasta").expect("パース成功すべし");

    let actors: Vec<_> = ast
        .items
        .iter()
        .filter_map(|item| {
            if let FileItem::ActorScope(actor) = item {
                Some(actor)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(actors.len(), 1);
    let actor = actors[0];

    assert_eq!(actor.name, "さくら");
    assert_eq!(actor.words.len(), 2);
    assert_eq!(
        actor.code_blocks.len(),
        0,
        "code_blocksは空であるべし（後方互換）"
    );
}
