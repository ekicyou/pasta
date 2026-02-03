# Design Document

## Project Description (Input)

## token→さくらスクリプトbuildの前処理

`pasta.shiori.act`:build()にて、token→さくらスクリプトへのビルドを行っているが、tokenをactor切り替え単位でグループ化する前処理を入れる。

アクター単位で連続したtalkを1つにまとめることで、会話速度調整（文字単位でウェイト）などの後処理フィルター（別仕様）などを投入予定のため。

### フェーズ1: actor切り替え単位でグループ化
### フェーズ2: 連続したtalkを1つにまとめる

---

## High-Level Architecture

### アーキテクチャ概要

本機能は、既存の`pasta.act`モジュールにグループ化機能を追加する「拡張型」アーキテクチャで実装する。

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         シーン関数実行                                   │
│                              │                                          │
│                              ▼                                          │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                     pasta.act                                     │  │
│  │  ┌─────────────────────────────────────────────────────────────┐  │  │
│  │  │  self.token[] (フラット配列)                                 │  │  │
│  │  │  ├── {type:"spot", actor:B, spot:1}       ← アクター属性     │  │  │
│  │  │  ├── {type:"talk", actor:A, text:"今日は"}                   │  │  │
│  │  │  ├── {type:"talk", actor:A, text:"晴れ"}                     │  │  │
│  │  │  ├── {type:"talk", actor:B, text:"明日は"}                   │  │  │
│  │  │  └── {type:"talk", actor:B, text:"雨"}                       │  │  │
│  │  └─────────────────────────────────────────────────────────────┘  │  │
│  │                              │                                    │  │
│  │                              ▼                                    │  │
│  │  ┌─────────────────────────────────────────────────────────────┐  │  │
│  │  │  ACT_IMPL.build()                                            │  │  │
│  │  │  ┌────────────────┐    ┌────────────────────────────┐        │  │  │
│  │  │  │group_by_actor()│ →  │merge_consecutive_talks()   │        │  │  │
│  │  │  └────────────────┘    └────────────────────────────┘        │  │  │
│  │  └─────────────────────────────────────────────────────────────┘  │  │
│  │                              │                                    │  │
│  │                              ▼                                    │  │
│  │  ┌─────────────────────────────────────────────────────────────┐  │  │
│  │  │  grouped_token[] (グループ化済み配列)                        │  │  │
│  │  │  ├── {type:"spot", actor:B, spot:1}       ← 独立トークン     │  │  │
│  │  │  ├── {type:"actor", actor:A, tokens:[...]}                   │  │  │
│  │  │  └── {type:"actor", actor:B, tokens:[...]}                   │  │  │
│  │  └─────────────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                              │                                          │
│                              ▼                                          │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                  pasta.shiori.act                                 │  │
│  │  ┌─────────────────────────────────────────────────────────────┐  │  │
│  │  │  SHIORI_ACT_IMPL.build()                                     │  │  │
│  │  │  ├── grouped_token[] を受け取る                              │  │  │
│  │  │  ├── flatten_grouped_tokens() でフラット化                   │  │  │
│  │  │  └── BUILDER.build(flat_tokens, config) に渡す               │  │  │
│  │  └─────────────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                              │                                          │
│                              ▼                                          │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │              pasta.shiori.sakura_builder                          │  │
│  │              （変更なし - フラットトークンを処理）                 │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                              │                                          │
│                              ▼                                          │
│                     さくらスクリプト文字列                              │
└─────────────────────────────────────────────────────────────────────────┘
```

### トークン分類

| 分類 | トークン | グループ化 | 説明 |
|------|---------|-----------|------|
| アクター属性設定 | `spot`, `clear_spot` | 対象外 | 独立トークンとして維持 |
| アクター行動 | `talk`, `surface`, `wait`, `newline`, `clear`, `sakura_script` | 対象 | `type="actor"`内に格納 |

### 設計原則

1. **責務分離**: グループ化は`pasta.act`、さくらスクリプト生成は`pasta.shiori.act`
2. **後方互換性**: 最終出力は完全一致
3. **純粋関数**: グループ化・統合関数は副作用なし
4. **段階的拡張**: 将来のフィルター機能追加を考慮した設計

---

## Component Design

### Component 1: group_by_actor(tokens) ローカル関数

**責務**: フラットなトークン配列をアクター切り替え境界で分割し、グループ化後トークン配列を生成

**所属**: `pasta/act.lua` 内ローカル関数

**対応要件**: R1, R2

```lua
--- トークン配列をアクター切り替え境界でグループ化
--- @param tokens table[] フラットなトークン配列
--- @return grouped_token[] グループ化されたトークン配列
local function group_by_actor(tokens)
    if not tokens or #tokens == 0 then
        return {}
    end

    local result = {}
    local current_actor_token = nil  -- 現在の type="actor" トークン
    local current_actor = nil        -- 現在のアクター（nilは未設定）

    for _, token in ipairs(tokens) do
        local t = token.type

        -- アクター属性設定トークン: 独立して出力
        if t == "spot" or t == "clear_spot" then
            table.insert(result, token)
        elseif t == "talk" then
            local talk_actor = token.actor
            -- アクター変更検出（最初のtalkまたはアクター変更時）
            if current_actor_token == nil or talk_actor ~= current_actor then
                -- 新しい type="actor" トークンを開始
                current_actor_token = {
                    type = "actor",
                    actor = talk_actor,
                    tokens = {}
                }
                table.insert(result, current_actor_token)
                current_actor = talk_actor
            end
            table.insert(current_actor_token.tokens, token)
        else
            -- その他のアクター行動トークン: 現在の type="actor" に追加
            if current_actor_token == nil then
                -- type="actor" がなければ actor=nil で作成
                current_actor_token = {
                    type = "actor",
                    actor = nil,
                    tokens = {}
                }
                table.insert(result, current_actor_token)
            end
            table.insert(current_actor_token.tokens, token)
        end
    end

    return result
end
```

**設計根拠**:
- `spot`, `clear_spot`は独立トークンとして維持（R2-1）
- `talk.actor`の変化でグループ分割（R2-2, R2-3）
- 他のアクター行動は現在のグループに追加（R2-4）

---

### Component 2: merge_consecutive_talks(grouped) ローカル関数

**責務**: `type="actor"`トークン内の連続talkトークンを単一talkに統合

**所属**: `pasta/act.lua` 内ローカル関数

**対応要件**: R3

```lua
--- グループ化トークン内の連続talkトークンを統合
--- @param grouped grouped_token[] グループ化されたトークン配列
--- @return grouped_token[] 統合済みトークン配列
local function merge_consecutive_talks(grouped)
    local result = {}

    for _, token in ipairs(grouped) do
        if token.type == "actor" then
            -- type="actor" トークン内のtalkを統合
            local merged_tokens = {}
            local pending_talk = nil

            for _, inner in ipairs(token.tokens) do
                if inner.type == "talk" then
                    if pending_talk then
                        -- 連続talk: テキスト結合
                        pending_talk.text = pending_talk.text .. inner.text
                    else
                        -- 新規talk開始
                        pending_talk = {
                            type = "talk",
                            actor = inner.actor,
                            text = inner.text
                        }
                    end
                else
                    -- 非talkトークン: pending_talkをフラッシュ
                    if pending_talk then
                        table.insert(merged_tokens, pending_talk)
                        pending_talk = nil
                    end
                    table.insert(merged_tokens, inner)
                end
            end

            -- 最後のpending_talkをフラッシュ
            if pending_talk then
                table.insert(merged_tokens, pending_talk)
            end

            table.insert(result, {
                type = "actor",
                actor = token.actor,
                tokens = merged_tokens
            })
        else
            -- spot, clear_spot はそのまま出力
            table.insert(result, token)
        end
    end

    return result
end
```

**設計根拠**:
- 連続talkのみ結合、アクター行動トークンで分離（R3-1, R3-2）
- 最初のtalkのactor情報を保持（R3-3）
- 新規テーブル作成で元データを変更しない（純粋関数、R7-3）

---

### Component 3: ACT_IMPL.build() 変更

**責務**: グループ化・統合済みトークン配列を返す

**所属**: `pasta/act.lua`

**対応要件**: R1, R2, R3

```lua
--- トークン取得とリセット（グループ化・統合済み）
--- @param self Act アクションオブジェクト
--- @return grouped_token[] グループ化されたトークン配列
function ACT_IMPL.build(self)
    local tokens = self.token
    self.token = {}
    
    -- Phase 1: アクター切り替え境界でグループ化
    local grouped = group_by_actor(tokens)
    
    -- Phase 2: 連続talkを統合
    local merged = merge_consecutive_talks(grouped)
    
    return merged
end
```

**API変更点**:
- 戻り値型: `token[]` → `grouped_token[]`
- 内部処理追加（外部シグネチャは同一）

---

### Component 4: flatten_grouped_tokens(grouped) ローカル関数

**責務**: グループ化トークン配列をフラットなトークン配列に変換

**所属**: `pasta/shiori/act.lua` 内ローカル関数

**対応要件**: R4

```lua
--- グループ化トークン配列をフラットなトークン配列に変換
--- @param grouped grouped_token[] グループ化されたトークン配列
--- @return table[] フラットなトークン配列
local function flatten_grouped_tokens(grouped)
    local flat = {}
    for _, token in ipairs(grouped) do
        if token.type == "actor" then
            -- type="actor" の中身を展開
            for _, inner in ipairs(token.tokens) do
                table.insert(flat, inner)
            end
        else
            -- spot, clear_spot はそのまま追加
            table.insert(flat, token)
        end
    end
    return flat
end
```

**設計根拠**:
- sakura_builderは既存のフラットトークン処理を維持（R4-3）
- 将来的にsakura_builderがグループ対応する際に変更しやすい
- 最小限の変更で後方互換性を保証（R6）

---

### Component 5: SHIORI_ACT_IMPL.build() 変更

**責務**: グループ化されたトークンを処理してさくらスクリプト生成

**所属**: `pasta/shiori/act.lua`

**対応要件**: R4

```lua
--- build()オーバーライド: さくらスクリプト生成
--- @param self ShioriAct アクションオブジェクト
--- @return string さくらスクリプト文字列
function SHIORI_ACT_IMPL.build(self)
    -- 親のbuild()でグループ化済みトークン取得
    local grouped = ACT.IMPL.build(self)
    -- フラット化してsakura_builderに渡す
    local flat_tokens = flatten_grouped_tokens(grouped)
    -- sakura_builderで変換
    local script = BUILDER.build(flat_tokens, {
        spot_newlines = self._spot_newlines
    })
    return script
end
```

**変更点**:
- `ACT.IMPL.build(self)`の戻り値が`grouped_token[]`に
- `flatten_grouped_tokens()`でフラット化後にBUILDER.build()へ

---

## Data Design

### グループ化後トークン構造

```lua
-- グループ化後の出力は3種類のトークンで構成される

-- 1. spotトークン（独立）
{ type = "spot", actor = <Actor>, spot = <number> }

-- 2. clear_spotトークン（独立）
{ type = "clear_spot" }

-- 3. actorトークン（グループ）
{
    type = "actor",
    actor = <Actor|nil>,
    tokens = {
        { type = "talk", actor = <Actor>, text = "今日は晴れでした。" },
        { type = "surface", id = 10 },
        { type = "wait", ms = 500 }
    }
}
```

### actorトークン内のトークン（アクター行動）

```lua
-- talk（統合後）
{ type = "talk", actor = <Actor>, text = "結合されたテキスト" }

-- その他アクター行動（変更なし）
{ type = "surface",      id = <number|string> }
{ type = "wait",         ms = <number> }
{ type = "newline",      n = <number> }
{ type = "clear" }
{ type = "sakura_script", text = <string> }
```

---

## Interface Design

### 外部API（変更なし）

| API | シグネチャ | 変更 |
|-----|-----------|------|
| `ACT.new(actors)` | `(table) -> Act` | 変更なし |
| `SHIORI_ACT.new(actors, req)` | `(table, table?) -> ShioriAct` | 変更なし |
| `SHIORI_ACT_IMPL.build(self)` | `(ShioriAct) -> string` | 変更なし |

### 内部API（変更あり）

| API | 変更前 | 変更後 |
|-----|--------|--------|
| `ACT_IMPL.build(self)` | `-> token[]` | `-> grouped_token[]` |

### ローカル関数（新規）

| 関数 | 所属 | シグネチャ |
|------|------|-----------|
| `group_by_actor` | `pasta/act.lua` | `(tokens) -> grouped_token[]` |
| `merge_consecutive_talks` | `pasta/act.lua` | `(grouped) -> grouped_token[]` |
| `flatten_grouped_tokens` | `pasta/shiori/act.lua` | `(grouped) -> token[]` |

---

## Acceptance Criteria Traceability

| Requirement | Acceptance Criteria | 設計対応 |
|-------------|---------------------|----------|
| R1-1 | 3種類のトークン型を返す | Data Design: グループ化後トークン構造 |
| R1-2 | 空配列で空配列を返す | group_by_actor: 最初のチェック |
| R1-3 | talkのみで単一actorトークン | group_by_actor: ロジック |
| R2-1 | spot/clear_spotは独立トークン | group_by_actor: 条件分岐 |
| R2-2 | actor変更で新actorトークン | group_by_actor: talk処理分岐 |
| R2-3 | 同一actorで同一actorトークン | group_by_actor: talk処理分岐 |
| R2-4 | アクター行動はactorトークン内に追加 | group_by_actor: else分岐 |
| R2-5 | 順序保持 | group_by_actor: ipairs順次処理 |
| R3-1 | 連続talk結合 | merge_consecutive_talks: pending_talk |
| R3-2 | アクター行動トークンで分離 | merge_consecutive_talks: else分岐 |
| R3-3 | 最初のactor保持 | merge_consecutive_talks: 最初のtalk保存 |
| R3-4 | アクター行動トークン保持 | merge_consecutive_talks: table.insert |
| R4-1 | grouped_token受け取り | SHIORI_ACT_IMPL.build: ACT.IMPL.build呼び出し |
| R4-2 | フラット化 | flatten_grouped_tokens + BUILDER.build |
| R4-3 | 出力互換 | 既存テストで検証 |
| R4-4 | 外部API変更なし | Interface Design: 変更なし |
| R5-1 | nil actor独立グループ | group_by_actor: talk_actor比較 |
| R5-2 | 断続的actor別グループ | group_by_actor: 参照比較 |
| R5-3 | 空文字列結合 | merge_consecutive_talks: 文字列結合 |
| R5-4 | talkなしグループ保持 | merge_consecutive_talks: そのまま出力 |
| R5-5 | 最初が非talkでもactor=nil作成 | group_by_actor: else分岐 |
| R6-1 | 既存テストパス | flatten_grouped_tokens保証 |
| R6-2 | 出力完全一致 | flatten_grouped_tokens保証 |
| R6-3 | 外部API変更なし | Interface Design |
| R7-1 | フィルター適用可能設計 | type="actor"構造 |
| R7-2 | 統合無効化可能 | 将来対応（現状は常に有効） |
| R7-3 | 純粋関数 | ローカル関数設計 |

---

## Error Handling

### エラーケース

| ケース | 処理 |
|--------|------|
| tokens = nil | 空配列を返す |
| tokens = {} | 空配列を返す |
| token.actor = nil | nilとして比較（別グループ扱い） |
| token.text = nil | ""として扱う |

### 検証ポリシー

- 入力検証は最小限（Luaの動的型付けを活用）
- nil/空配列は早期リターン
- 型エラーはLuaランタイムに委任

---

## Testing Strategy

### 単体テスト

#### group_by_actor テスト（新規）

```lua
describe("ACT - group_by_actor", function()
    test("空配列で空配列を返す", function()
        -- 検証: result = {}
    end)
    
    test("単一talkで単一actorトークン", function()
        -- 検証: #result == 1, result[1].type == "actor"
    end)
    
    test("spotは独立トークンとして出力", function()
        -- 検証: result[1].type == "spot"
    end)
    
    test("clear_spotは独立トークンとして出力", function()
        -- 検証: result[1].type == "clear_spot"
    end)
    
    test("actor変更で新actorトークン", function()
        -- 検証: #result == 2
    end)
    
    test("断続的actorは別actorトークン", function()
        -- 検証: A→B→A で3つのactorトークン
    end)
end)
```

#### merge_consecutive_talks テスト（新規）

```lua
describe("ACT - merge_consecutive_talks", function()
    test("連続talkを結合", function()
        -- 検証: text == "今日は晴れ"
    end)
    
    test("アクター行動トークンで分離", function()
        -- 検証: 2つのtalkトークン
    end)
    
    test("空文字列も結合に含む", function()
        -- 検証: text == "今日は晴れ"
    end)
    
    test("spot/clear_spotはそのまま出力", function()
        -- 検証: 独立トークンが維持される
    end)
end)
```

### 統合テスト

#### 後方互換性テスト

- 既存の`sakura_builder_test.lua`（521行）を全パス
- 新規テストなしで後方互換性を検証

### テストファイル

| ファイル | 役割 |
|---------|------|
| `lua_specs/act_grouping_test.lua` | グループ化・統合単体テスト（新規） |
| `lua_specs/sakura_builder_test.lua` | 後方互換性テスト（既存） |

---

## Performance Considerations

### 計算量

| 処理 | 計算量 |
|------|--------|
| group_by_actor | O(n) - 単一パス |
| merge_consecutive_talks | O(n) - 単一パス |
| flatten_grouped_tokens | O(n) - 単一パス |

### メモリ

- 一時的なgrouped_token配列を作成
- 元のtoken配列サイズ + グループメタデータ（定数）
- 実用上の影響は最小限

---

## Future Extensibility

### 将来のフィルター機能対応

```lua
-- 将来の拡張例: 会話速度フィルター
local function apply_speech_speed_filter(grouped)
    for _, token in ipairs(grouped) do
        if token.type == "actor" then
            for _, inner in ipairs(token.tokens) do
                if inner.type == "talk" then
                    -- 文字単位でウェイト挿入
                    inner.text = insert_character_waits(inner.text)
                end
            end
        end
    end
    return grouped
end

-- SHIORI_ACT_IMPL.build()での使用
local grouped = ACT.IMPL.build(self)
grouped = apply_speech_speed_filter(grouped)  -- 将来追加
local flat_tokens = flatten_grouped_tokens(grouped)
```

### merge無効化オプション（R7-2）

```lua
-- 将来の拡張: オプション引数
function ACT_IMPL.build(self, options)
    options = options or {}
    local grouped = group_by_actor(self.token)
    if options.merge_talks ~= false then
        grouped = merge_consecutive_talks(grouped)
    end
    self.token = {}
    return grouped
end
```

---

## Dependencies

### 既存依存（変更なし）

- `pasta.actor`: Actorオブジェクト
- `pasta.shiori.sakura_builder`: さくらスクリプト生成

### 新規依存

- なし（ローカル関数のみで実装）

---

## Implementation Notes

### 実装順序

1. `pasta/act.lua`に`group_by_actor()`追加
2. `pasta/act.lua`に`merge_consecutive_talks()`追加
3. `ACT_IMPL.build()`を変更
4. `pasta/shiori/act.lua`に`flatten_grouped_tokens()`追加
5. `SHIORI_ACT_IMPL.build()`を変更
6. テスト作成・実行
7. 既存テスト全パス確認

### コード配置

```lua
-- pasta/act.lua の構造
local ACTOR = require("pasta.actor")
local SCENE = require("pasta.scene")
local GLOBAL = require("pasta.global")

local ACT = {}
local ACT_IMPL = {}

-- ============================================================================
-- グループ化ローカル関数
-- ============================================================================

local function group_by_actor(tokens)
    -- ...
end

local function merge_consecutive_talks(grouped)
    -- ...
end

-- ============================================================================
-- ACT_IMPL メソッド
-- ============================================================================

function ACT_IMPL.build(self)
    -- ...
end

-- 他のメソッド...

return ACT
```
