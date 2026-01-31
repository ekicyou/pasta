# Design: alpha03-shiori-act-sakura

> **Status**: Generated  
> **Requirements Version**: 1.0  
> **Last Updated**: 2025-01-20

---

## 1. Overview

### 1.1 Purpose

`pasta.shiori.act` モジュールは、SHIORI サブシステム専用のアクションビルダーを提供する。`pasta.act` を継承し、さくらスクリプトタグ（サーフェス切り替え、待機、改行、クリア等）の生成機能を追加する。

### 1.2 Scope

**In Scope**:
- `pasta.act` の継承（メタテーブルチェーン）
- さくらスクリプトタグ生成メソッド（`surface`, `wait`, `newline`, `clear`）
- `talk()` オーバーライドによるスコープ自動切り替え
- バッファ管理と `build()`/`reset()` API

**Out of Scope**:
- SHIORI プロトコル処理（`pasta.shiori.res` の責務）
- イベントディスパッチ（`pasta.shiori.event` の責務）
- シェル定義やサーフェスエイリアス管理

### 1.3 Background

`pasta.act` は汎用的なアクションオブジェクトを提供するが、さくらスクリプト特有のタグ生成機能を持たない。SHIORI モジュール群では、アクション実行結果をさくらスクリプト形式で出力する必要があり、そのための専用モジュールが必要となった。

### 1.4 Design Goals

| Goal | Description | Priority |
|------|-------------|----------|
| 継承互換 | `pasta.act` のすべてのメソッドを継承し、既存スクリプトとの互換性を維持 | High |
| 直感的API | メソッドチェーンで自然にさくらスクリプトを構築可能 | High |
| 責務分離 | さくらスクリプト生成は本モジュール、プロトコル処理は別モジュール | Medium |
| テスト容易性 | `build()` で文字列出力、単体テストで検証可能 | Medium |

---

## 2. Architecture

### 2.1 High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                    pasta.shiori.act                         │
│  ┌───────────────┐  ┌───────────────────────────────────┐  │
│  │ SHIORI_ACT    │  │ SHIORI_ACT_IMPL                   │  │
│  │  .new(ctx)    │  │  .__index (→ ACT.IMPL fallback)   │  │
│  │  .IMPL        │  │  .talk(actor, text)               │  │
│  │               │  │  .surface(id)                     │  │
│  │               │  │  .wait(ms)                        │  │
│  │               │  │  .newline(n)                      │  │
│  │               │  │  .clear()                         │  │
│  │               │  │  .build()                         │  │
│  │               │  │  .reset()                         │  │
│  └───────────────┘  └───────────────────────────────────┘  │
│                              │                              │
│                              │ setmetatable                 │
│                              ▼                              │
│  ┌───────────────────────────────────────────────────────┐ │
│  │                 pasta.act (ACT.IMPL)                  │ │
│  │   .talk(), .sakura_script(), .word(), .yield(), ...   │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Inheritance Chain

```
SHIORI_ACT_IMPL
    │
    └──▶ __index ──▶ ACT.IMPL (pasta.act)
                          │
                          └──▶ __index ──▶ actor proxy generator
```

**Key Points**:
1. `SHIORI_ACT_IMPL` のメタテーブルに `{__index = ACT.IMPL}` を設定
2. `SHIORI_ACT_IMPL` で定義されたメソッドが優先（オーバーライド）
3. 未定義メソッドは `ACT.IMPL` にフォールバック

### 2.3 Data Flow

```
User Script           SHIORI_ACT              Output
    │                     │                      │
    │  act:talk(sakura, "Hello")                 │
    │  ──────────────────►│                      │
    │                     │ → scope switch?      │
    │                     │ → append \0          │
    │                     │ → append text        │
    │                     │ → append \n          │
    │                     ▼                      │
    │  act:surface(5)     │                      │
    │  ──────────────────►│                      │
    │                     │ → append \s[5]       │
    │                     ▼                      │
    │  act:wait(500)      │                      │
    │  ──────────────────►│                      │
    │                     │ → append \w[500]     │
    │                     ▼                      │
    │  act:build()        │                      │
    │  ──────────────────►│                      │
    │                     │ ────────────────────►│
    │                     │   "\0Hello\n\s[5]\w[500]"
```

---

## 3. Component Design

### 3.1 SHIORI_ACT Module Table

**責務**: モジュールエントリポイント、コンストラクタ提供

```lua
local SHIORI_ACT = {}
SHIORI_ACT.IMPL = SHIORI_ACT_IMPL  -- 継承用に公開
```

**API**:
- `SHIORI_ACT.new(ctx)`: 新規インスタンス生成

### 3.2 SHIORI_ACT_IMPL Implementation Metatable

**責務**: インスタンスメソッドの実装

**内部状態**:
| Field | Type | Description |
|-------|------|-------------|
| `_buffer` | `table<string>` | さくらスクリプト文字列の蓄積バッファ |
| `_current_scope` | `number or nil` | 現在のスコープ（0=sakura, 1=kero, ...） |

**メソッド一覧**:

| Method | Signature | Description |
|--------|-----------|-------------|
| `talk` | `(self, actor, text) → self` | スコープ切り替え＋テキスト追加 |
| `surface` | `(self, id) → self` | `\s[id]` タグ追加 |
| `wait` | `(self, ms) → self` | `\w[ms]` タグ追加 |
| `newline` | `(self, n?) → self` | `\n` を n 回追加（デフォルト 1） |
| `clear` | `(self) → self` | `\c` タグ追加 |
| `build` | `(self) → string` | バッファを結合して返却 |
| `reset` | `(self) → self` | バッファと状態をクリア |

### 3.3 Method Details

#### 3.3.1 `talk(actor, text)`

**動作フロー**:
1. `actor.spot` を取得（例: `"sakura"`, `"kero"`, `"char2"`）
2. `spot` からスコープ番号を決定:
   - `"sakura"` → `0`
   - `"kero"` → `1`
   - `"char<N>"` → `N`
3. 前回スコープと異なる場合:
   - `_current_scope` 更新
   - スコープタグを追加（`\0`, `\1`, `\p[N]`）
   - 改行タグを追加（`\n`）
4. テキストをエスケープしてバッファに追加
5. 改行タグを追加（`\n`）
6. 親クラスの `talk()` も呼び出し（`token` バッファ用）
7. `self` を返却

**エスケープ処理**:
```lua
local function escape_sakura(text)
    return text:gsub("\\", "\\\\"):gsub("%%", "%%%%")
end
```

#### 3.3.2 `surface(id)`

**動作フロー**:
1. `id` を整数または文字列として受け付け
2. `\s[<id>]` をバッファに追加
3. `self` を返却

#### 3.3.3 `wait(ms)`

**動作フロー**:
1. `ms` をミリ秒として受け付け（整数）
2. `\w[<ms>]` をバッファに追加
3. `self` を返却

#### 3.3.4 `newline(n)`

**動作フロー**:
1. `n` が未指定の場合は `1` をデフォルト
2. `n` 回 `\n` をバッファに追加
3. `self` を返却

**バリデーション**: `n < 1` の場合は何も追加しない

#### 3.3.5 `clear()`

**動作フロー**:
1. `\c` をバッファに追加
2. `self` を返却

#### 3.3.6 `build()`

**動作フロー**:
1. `_buffer` 内のすべての文字列を `table.concat()` で結合
2. 結合結果の末尾に `\e`（スクリプト終端）を追加
3. 完成したさくらスクリプト文字列を返却（バッファはクリアしない）

**実装例**:
```lua
function SHIORI_ACT_IMPL:build()
    local script = table.concat(self._buffer)
    return script .. "\\e"
end
```

#### 3.3.7 `reset()`

**動作フロー**:
1. `_buffer` を空テーブルにリセット
2. `_current_scope` を `nil` にリセット
3. `self` を返却

---

## 4. Interface Definitions

### 4.1 Module Interface

```lua
---@class SHIORI_ACT
---@field IMPL SHIORI_ACT_IMPL
local SHIORI_ACT = {}

---@param ctx Context
---@return SHIORI_ACT_IMPL
function SHIORI_ACT.new(ctx) end
```

### 4.2 Instance Interface

```lua
---@class SHIORI_ACT_IMPL : ACT_IMPL
---@field private _buffer string[]
---@field private _current_scope number?
local SHIORI_ACT_IMPL = {}

---@param actor Actor
---@param text string
---@return self
function SHIORI_ACT_IMPL:talk(actor, text) end

---@param id number|string
---@return self
function SHIORI_ACT_IMPL:surface(id) end

---@param ms number
---@return self
function SHIORI_ACT_IMPL:wait(ms) end

---@param n? number
---@return self
function SHIORI_ACT_IMPL:newline(n) end

---@return self
function SHIORI_ACT_IMPL:clear() end

---@return string
function SHIORI_ACT_IMPL:build() end

---@return self
function SHIORI_ACT_IMPL:reset() end
```

### 4.3 Parent Dependency

**Required Change to `pasta.act`**:
```lua
-- pasta/act.lua の最後に追加
ACT.IMPL = ACT_IMPL
```

---

## 5. Data Models

### 5.1 Instance State

```lua
{
    -- 継承元 (ACT_IMPL) のフィールド
    ctx = <Context>,
    token = <table>,
    
    -- 本モジュール固有のフィールド
    _buffer = {"\\0", "Hello", "\\n", "\\s[5]"},
    _current_scope = 0
}
```

### 5.2 Scope Mapping

| Spot Name | Scope Tag | `_current_scope` |
|-----------|-----------|------------------|
| `"sakura"` | `\0` | `0` |
| `"kero"` | `\1` | `1` |
| `"char2"` | `\p[2]` | `2` |
| `"char<N>"` | `\p[N]` | `N` |

---

## 6. Error Handling
| `build()` 空バッファ | `\e` のみ返却 | 警告なし（正常動作） |

### 6.1 Error Scenarios

| Scenario | Handling | Message |
|----------|----------|---------|
| `talk()` に nil actor | エラー送出 | `"actor is required"` |
| `surface()` に無効な id | そのまま出力 | 警告なし（ゴースト側で処理） |
| `wait()` に負数 | 0 として扱う | 警告なし |
| `newline()` に負数 | 何も追加しない | 警告なし |

### 6.2 Defensive Coding

```lua
function SHIORI_ACT_IMPL:wait(ms)
    ms = math.max(0, math.floor(ms or 0))
    table.insert(self._buffer, string.format("\\w[%d]", ms))
    return self
end
```

---

## 7. Testing Strategy

### 7.1 Test Categories

| Category | Description | Priority |
|----------|-------------|----------|
| 継承検証 | 親メソッドの継承動作確認 | High |
| タグ生成 | 各メソッドの出力検証 | High |
| スコープ切替 | `talk()` のスコープ自動切替 | High |
| エスケープ | 特殊文字のエスケープ処理 | Medium |
| エッジケース | 空文字列、負数、連続呼び出し | Medium |

### 7.2 Test File Location

`crates/pasta_lua/tests/lua_specs/shiori_act_spec.lua`

### 7.3 Test Cases

```lua
describe("SHIORI_ACT", function()
    describe("inheritance", function()
        it("inherits ACT.IMPL methods", function()
            local act = SHIORI_ACT.new(ctx)
            -- sakura_script() は ACT_IMPL のメソッド
            act:sakura_script("\\e")
            expect(act.token).to_have_length(1)
        end)
    end)

    describe("talk()", function()
        it("appends scope tag on actor switch", function()
            local act = SHIORI_ACT.new(ctx)
            act:talk(sakura, "Hello")
            act:talk(kero, "Hi")\\e$")
        end)

        it("does not append scope tag on same actor", function()
            local act = SHIORI_ACT.new(ctx)
            act:talk(sakura, "Hello")
            act:talk(sakura, "World")
            expect(act:build()).to_match("\\0Hello\\nWorld\\n\\e$

        it("appends surface tag with alias", function()
            local act = SHIORI_ACT.new(ctx)
            act:surface("smile")
            expect(act:build()).to_eq("\\s[smile]\\e")
        end)
    end)

    describe("wait()", function()
        it("appends wait tag", function()
            local act = SHIORI_ACT.new(ctx)
            act:wait(500)
            expect(act:build()).to_eq("\\w[500]\\e")
        end)
    end)

    describe("newline()", function()
        it("appends single newline by default", function()
            local act = SHIORI_ACT.new(ctx)
            act:newline()
            expect(act:build()).to_eq("\\n\\e")
        end)

        it("appends multiple newlines", function()
            local act = SHIORI_ACT.new(ctx)
            act:newline(3)
            expect(act:build()).to_eq("\\n\\n\\n\\e")
        end)
    end)

    describe("clear()", function()
        it("appends clear tag", function()
            local act = SHIORI_ACT.new(ctx)
            act:clear()
            expect(act:build()).to_eq("\\c\\e)
            act:newline(3)
            expect(act:build()).to_eq("\\n\\n\\n")
        end)
    end)

    describe("clear()", function()
        it("appends clear tag", function()
            local act = SHIORI_ACT.new(ctx)
            act:clear()\\e")  -- 空バッファでも \e は付与
            expect(act:build()).to_eq("\\c")
        end)
    end)

    describe("reset()", function()
        it("clears buffer and scope", function()
            local act = SHIORI_ACT.new(ctx)
            act:talk(sakura, "Hello")
            act:reset()
            expect(act:build()).to_eq("")
        end)
    end)

    describe("escape", function()
        it("escapes backslash", function()
            local act = SHIORI_ACT.new(ctx)
            act:talk(sakura, "path\\to\\file")
            expect(act:build()).to_match("path\\\\to\\\\file")
        end)

        it("escapes percent", function()
            local act = SHIORI_ACT.new(ctx)
            act:talk(sakura, "100%")
            expect(act:build()).to_match("100%%%%")
        end)
    end)
end)
```

---

## 8. Implementation Notes

### 8.1 Prerequisites

1. **`pasta.act` への変更**: `ACT.IMPL = ACT_IMPL` を追加
2. **ファイル作成**: `crates/pasta_lua/scripts/pasta/shiori/act.lua`

### 8.2 Implementation Order

1. `pasta.act` に `ACT.IMPL` 公開を追加
2. `SHIORI_ACT` モジュール骨格を作成
3. `SHIORI_ACT.new()` コンストラクタを実装
4. `reset()`, `build()` を実装（基本的なバッファ操作）
5. `surface()`, `wait()`, `newline()`, `clear()` を実装
6. `talk()` オーバーライドを実装（スコープ切替ロジック含む）
7. テストを作成・実行
8. ドキュメント更新

### 8.3 Code Template

```lua
-- pasta/shiori/act.lua
local ACT = require("pasta.act")

local SHIORI_ACT = {}
local SHIORI_ACT_IMPL = {}

-- 継承チェーン設定
setmetatable(SHIORI_ACT_IMPL, { __index = ACT.IMPL })
SHIORI_ACT_IMPL.__index = SHIORI_ACT_IMPL

-- 公開
SHIORI_ACT.IMPL = SHIORI_ACT_IMPL

function SHIORI_ACT.new(ctx)
    local base = ACT.new(ctx)
    base._buffer = {}
    base._current_scope = nil
    setmetatable(base, SHIORI_ACT_IMPL)
    return base
end

-- ... メソッド実装 ...

return SHIORI_ACT
```

---

## 9. Appendix

### 9.1 Sakura Script Reference

| Tag | Description | Example |
|-----|-------------|---------|
| `\0` | Scope 0 (main character) | `\0Hello` |
| `\1` | Scope 1 (sub character) | `\1Hi` |
| `\p[n]` | Scope n (arbitrary) | `\p[2]` |
| `\s[n]` | Surface ID | `\s[5]` |
| `\w[ms]` | Wait milliseconds | `\w[500]` |
| `\n` | Newline | `\n` |
| `\c` | Clear balloon | `\c` |
| `\e` | End script | `\e` |

### 9.2 Related Modules

| Module | Relationship |
|--------|--------------|
| `pasta.act` | 継承元（親クラス） |
| `pasta.shiori.res` | レスポンス生成（本モジュールの出力を使用） |
| `pasta.shiori.event` | イベントディスパッチ（本モジュールをインスタンス化） |
| `pasta.actor` | アクター情報（`spot` プロパティ使用） |
