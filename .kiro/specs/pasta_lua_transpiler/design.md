# Technical Design Document

## Overview

**Purpose**: code_generator.rs の出力形式を Act-first アーキテクチャに変更する詳細設計

**Parent Spec**: `pasta_lua_design_refactor`

## Requirements Traceability

| Requirement | Summary | 修正対象メソッド |
|-------------|---------|-----------------|
| 1 | シーン関数シグネチャ | `generate_local_scene()` |
| 2 | init_scene呼び出し | `generate_local_scene()` |
| 3 | スポット管理API | `generate_local_scene()` |
| 4 | create_scene API | `generate_global_scene()` |
| 5 | アクタープロキシ呼び出し | `generate_action_line()` |
| 6 | シーン遷移API | `generate_call_scene()` |
| 7 | テスト互換性 | テストファイル修正 |

## 出力パターン変更

### Before（現状）

```lua
do
    local SCENE = PASTA.create_scene("モジュール名_N")
    
    function SCENE.__start__(ctx, ...)
        local args = { ... }
        PASTA.clear_spot(ctx)
        PASTA.set_spot(ctx, "さくら", 0)
        PASTA.set_spot(ctx, "うにゅう", 1)
        local act, save, var = PASTA.create_session(SCENE, ctx)
        
        act:talk("こんにちは")
    end
end
```

### After（設計準拠）

```lua
do
    local SCENE = PASTA.create_scene("グローバル名", "__start__", nil)
    
    function SCENE.__start__(act, ...)
        local args = { ... }
        act:clear_spot()
        act:set_spot("さくら", 0)
        act:set_spot("うにゅう", 1)
        local save, var = act:init_scene(SCENE)
        
        act.さくら:talk("こんにちは")
    end
    
    -- シーン関数を登録（do...end終了前）
    PASTA.create_scene("グローバル名", "__start__", SCENE.__start__)
end
```

## 修正詳細

### 1. generate_local_scene() の修正

**現状**:
```rust
self.writeln(&format!("function SCENE.{}(ctx, ...)", fn_name))?;
```

**修正後**:
```rust
self.writeln(&format!("function SCENE.{}(act, ...)", fn_name))?;
```

### 2. セッション初期化の修正

**現状**:
```rust
self.writeln("local act, save, var = PASTA.create_session(SCENE, ctx)")?;
```

**修正後**:
```rust
self.writeln("local save, var = act:init_scene(SCENE)")?;
```

### 3. スポット管理の修正

**現状**:
```rust
self.writeln("PASTA.clear_spot(ctx)")?;
self.writeln(&format!(r#"PASTA.set_spot(ctx, "{}", {})"#, actor.name, actor.number))?;
```

**修正後**:
```rust
self.writeln("act:clear_spot()")?;
self.writeln(&format!(r#"act:set_spot("{}", {})"#, actor.name, actor.number))?;
```

### 4. generate_global_scene() の修正

**現状**:
```rust
self.writeln(&format!("local SCENE = PASTA.create_scene(\"{}\")", module_name))?;
```

**修正後（2段階登録）**:

```rust
// Phase 1: SCENEテーブル作成（関数定義前）
self.writeln(&format!("local SCENE = PASTA.create_scene(\"{}\")", global_name))?;

// ... 関数定義 ...

// Phase 2: 各ローカルシーン関数を登録（do...end終了前）
for local_scene in &scene.local_scenes {
    let local_name = /* fn_name */;
    self.writeln(&format!(
        "PASTA.create_scene(\"{}\", \"{}\", SCENE.{})",
        global_name, local_name, local_name
    ))?;
}
```

### 5. アクタープロキシ呼び出しの修正

**現状（ActionLine生成）**:
```rust
// act:talk("テキスト") 形式
self.writeln(&format!("act:talk(\"{}\")", text))?;
```

**修正後**:
```rust
// act.アクター:talk("テキスト") 形式
self.writeln(&format!("act.{}:talk(\"{}\")", actor_name, text))?;
```

### 6. シーン遷移の修正

**現状**:
```rust
// Call/Jumpの生成（現状形式は要確認）
```

**修正後**:
```rust
// act:call({global_name, local_name}, opts, ...) 形式
self.writeln(&format!(
    "act:call({{\"{}\", \"{}\"}}, {{}}, table.unpack(args))",
    global_name, local_name
))?;
```

## 順序制約

シーン関数内の出力順序は以下を維持:

1. `local args = { ... }`
2. `act:clear_spot()` （アクターが定義されている場合）
3. `act:set_spot("name", number)` × N
4. `local save, var = act:init_scene(SCENE)`
5. シーン本体（talk, word, call等）

## テスト戦略

### 既存テストの修正

`transpiler_integration_test.rs` の期待値を新形式に更新:

```rust
// Before
assert!(output.contains("function SCENE.__start__(ctx, ...)"));
assert!(output.contains("PASTA.create_session(SCENE, ctx)"));

// After
assert!(output.contains("function SCENE.__start__(act, ...)"));
assert!(output.contains("act:init_scene(SCENE)"));
```

### 新規テストケース

1. **シグネチャテスト**: act-first パターン検証
2. **init_sceneテスト**: 戻り値 save, var の検証
3. **アクタープロキシテスト**: `act.アクター:talk()` パターン検証
4. **スポット管理テスト**: `act:set_spot()` パターン検証

## 影響範囲

### 修正対象ファイル

| ファイル | 修正内容 |
|---------|---------|
| `code_generator.rs` | 全出力パターン変更 |
| `transpiler_integration_test.rs` | 期待値更新 |
| `lua_specs/*.lua` | テスト用Luaスクリプト更新 |

### 後方互換性

- **破壊的変更**: 既存のトランスパイル出力は動作しなくなる
- **移行**: Luaスケルトン実装と同時にデプロイが必要
