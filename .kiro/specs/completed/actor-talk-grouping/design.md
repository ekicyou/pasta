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
│  │  │  └── BUILDER.build_grouped(grouped, config) に渡す           │  │  │
│  │  └─────────────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                              │                                          │
│                              ▼                                          │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │              pasta.shiori.sakura_builder                          │  │
│  │  ┌─────────────────────────────────────────────────────────────┐  │  │
│  │  │  BUILDER.build_grouped(grouped_tokens, config)               │  │  │
│  │  │  ├── type="spot"       → actor_spots更新                     │  │  │
│  │  │  ├── type="clear_spot" → 状態リセット                        │  │  │
│  │  │  └── type="actor"      → 内部tokens[]を処理しスクリプト生成  │  │  │
│  │  └─────────────────────────────────────────────────────────────┘  │  │
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
| アクター行動 | `talk` (pasta DSL由来) | 対象 | `type="actor"`内に格納 |

**注**: `surface`, `wait`, `newline`, `clear`, `sakura_script`は`talk`経由でさくらスクリプトとして発行されるため、グループ化処理では単独で発生しない。

### 設計原則

1. **責務分離**: グループ化は`pasta.act`、さくらスクリプト生成は`pasta.shiori.sakura_builder`
2. **後方互換性**: 最終出力は完全一致
3. **純粋関数**: グループ化・統合関数は副作用なし
4. **段階的拡張**: 将来のフィルター機能追加を考慮した設計
5. **並行開発戦略**: `build_grouped()`で並行開発 → 同等性確認後に置き換え

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
            -- 未知のトークン型（将来の拡張やバグ検出用）
            -- 現在の設計では発生しないが、デバッグ容易性のため警告を出力
            if current_actor_token then
                -- アクターグループ内に追加（アクター行動として扱う）
                table.insert(current_actor_token.tokens, token)
            else
                -- アクターグループ外の未知トークン: エラーログ推奨
                -- 実装時にtracing等でログ出力を検討
            end
        end
    end

    return result
end
```

**設計根拠**:
- `spot`, `clear_spot`は独立トークンとして維持（R2-1）
- `talk.actor`の変化でグループ分割（R2-2, R2-3）
- elseブロック: 将来の拡張性確保とデバッグ容易性のため、未知のトークン型を処理（R7-1）

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

### Component 4: BUILDER.build_grouped() 新規追加

**責務**: グループ化されたトークン配列（`grouped_token[]`）を直接処理してさくらスクリプトを生成

**所属**: `pasta/shiori/sakura_builder.lua`

**対応要件**: R4

**実装戦略**: 既存の`BUILDER.build()`は維持しつつ、新規関数`BUILDER.build_grouped()`を追加。Phase 3で置き換え。

```lua
--- グループ化トークン配列をさくらスクリプト文字列に変換
--- @param grouped_tokens grouped_token[] グループ化されたトークン配列
--- @param config BuildConfig|nil 設定
--- @return string さくらスクリプト文字列
function BUILDER.build_grouped(grouped_tokens, config)
    config = config or {}
    local spot_newlines = config.spot_newlines or 1.5
    local buffer = {}

    -- ビルダー内部状態（build()呼び出しごとにリセット）
    local actor_spots = {} -- {[actor_name]: spot_id} actor位置マップ
    local last_actor = nil -- 最後に発言したActor
    local last_spot = nil  -- 最後のスポットID

    for _, token in ipairs(grouped_tokens) do
        local t = token.type

        if t == "spot" then
            -- spotトークン処理: actor_spots[actor.name] = spot
            if token.actor and token.actor.name then
                actor_spots[token.actor.name] = token.spot
            end

        elseif t == "clear_spot" then
            -- clear_spotトークン処理: 状態リセット
            actor_spots = {}
            last_actor = nil
            last_spot = nil

        elseif t == "actor" then
            -- actorトークン処理: グループ内のトークンを順次処理
            local actor = token.actor
            local actor_name = actor and actor.name

            -- アクター切り替え検出
            if actor and last_actor ~= actor then
                local spot = actor_spots[actor_name] or 0

                -- spot変更時に段落区切り改行を出力
                if last_spot ~= nil and last_spot ~= spot then
                    local percent = math.floor(spot_newlines * 100)
                    table.insert(buffer, string.format("\\n[%d]", percent))
                end

                table.insert(buffer, spot_to_tag(spot))
                last_actor = actor
                last_spot = spot
            end

            -- グループ内トークンを順次処理
            for _, inner in ipairs(token.tokens) do
                local inner_type = inner.type

                if inner_type == "talk" then
                    table.insert(buffer, escape_sakura(inner.text))
                elseif inner_type == "surface" then
                    table.insert(buffer, string.format("\\s[%s]", tostring(inner.id)))
                elseif inner_type == "wait" then
                    table.insert(buffer, string.format("\\w[%d]", inner.ms))
                elseif inner_type == "newline" then
                    for _ = 1, inner.n do
                        table.insert(buffer, "\\n")
                    end
                elseif inner_type == "clear" then
                    table.insert(buffer, "\\c")
                elseif inner_type == "sakura_script" then
                    table.insert(buffer, inner.text)
                end
            end
        end
    end

    return table.concat(buffer) .. "\\e"
end
```

**設計根拠**:
- `grouped_token[]`を直接処理することで、フラット化処理が不要になる
- 将来のフィルター機能（会話速度調整等）をグループ単位で挿入可能
- `type="actor"`トークン処理時にアクター切り替え検出を行い、既存のさくらスクリプト出力と完全互換を維持

**既存コードとの主な変更点**:
1. 最上位ループが`grouped_token[]`を処理（旧: フラットな`token[]`）
2. `type="actor"`トークンを認識し、内部の`tokens`配列を処理
3. アクター切り替え検出は`type="actor"`トークン処理時に実施
4. 既存の`actor`, `spot_switch`トークン処理は削除（新形式に統合）

---

### Component 5: SHIORI_ACT_IMPL.build() 変更

**責務**: グループ化されたトークンをそのまま`BUILDER.build_grouped()`に渡す

**所属**: `pasta/shiori/act.lua`

**対応要件**: R4

```lua
--- build()オーバーライド: さくらスクリプト生成
--- @param self ShioriAct アクションオブジェクト
--- @return string さくらスクリプト文字列
function SHIORI_ACT_IMPL.build(self)
    -- 親のbuild()でグループ化済みトークン取得
    local grouped = ACT.IMPL.build(self)
    -- sakura_builderで変換（グループ化トークンを直接処理）
    local script = BUILDER.build_grouped(grouped, {
        spot_newlines = self._spot_newlines
    })
    return script
end
```

**変更点**:
- `ACT.IMPL.build(self)`の戻り値が`grouped_token[]`に
- `BUILDER.build_grouped()`を使用（Phase 3で`build()`に統合）
- `flatten_grouped_tokens()`は不要（削除）

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

**影響範囲**:
- `ACT_IMPL.build()`を直接呼び出すコード: `pasta/shiori/act.lua`の`SHIORI_ACT_IMPL.build()`内の1箇所のみ
- テストコード: `act:build()`形式で呼び出し（`SHIORI_ACT`インスタンス経由）、影響なし
- 将来の非SHIORIバックエンド: `ACT_IMPL.build()`を継承する場合、戻り値型の変更を認識する必要がある

**継承に関する注意事項**:
`ACT_IMPL.build()`は継承を前提とした内部APIであり、子クラスは戻り値型が`grouped_token[]`に変更されたことを認識する必要があります。現在の実装では`BUILDER.build()`がグループ化トークンを直接処理することで、後方互換性を保証しています。

### ローカル関数（新規）

| 関数 | 所属 | シグネチャ |
|------|------|-----------|
| `group_by_actor` | `pasta/act.lua` | `(tokens) -> grouped_token[]` |
| `merge_consecutive_talks` | `pasta/act.lua` | `(grouped) -> grouped_token[]` |

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
| R4-1 | BUILDER.build()がgrouped_token[]を受け取る | Component 4: BUILDER.build改造 |
| R4-2 | spot/clear_spot/actorを処理 | Component 4: 3分岐処理 |
| R4-3 | actorトークン内のtokens配列を処理 | Component 4: inner loop |
| R4-4 | アクター切り替え検出はactor処理時に実施 | Component 4: last_actor比較 |
| R4-5 | 既存出力と完全互換 | 既存テストで検証 |
| R4-6 | SHIORI_ACT_IMPL.build()はgroupedを直接渡す | Component 5: SHIORI_ACT_IMPL.build |
| R4-7 | 外部API変更なし | Interface Design: 変更なし |
| R5-1 | nil actor独立グループ | group_by_actor: talk_actor比較 |
| R5-2 | 断続的actor別グループ | group_by_actor: 参照比較 |
| R5-3 | 空文字列結合 | merge_consecutive_talks: 文字列結合 |
| R5-4 | talkなしグループ保持 | merge_consecutive_talks: そのまま出力 |
| R6-1 | 既存テストパス | BUILDER.build()互換性保証 |
| R6-2 | 出力完全一致 | BUILDER.build()互換性保証 |
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
| 未知のトークン型 | アクターグループ内なら追加、グループ外ならログ出力（実装時に検討） |

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

#### アクター切り替え検出の境界値テスト（新規）

**目的**: R4-4（アクター切り替え検出）の正確性を保証

```lua
describe("SHIORI_ACT - actor switching detection", function()
    test("spot後の最初のtalkでアクター切り替え検出", function()
        -- Given: spot(B) → talk(A)
        -- Expected: \s\0（Aのスポット）が出力される
    end)
    
    test("clear_spot後の初回talkで状態リセット確認", function()
        -- Given: talk(A) → clear_spot → talk(A)
        -- Expected: clear_spot後のAは新規スポット0として扱われる
    end)
    
    test("actor=nilのtalkでスポット切り替え挙動", function()
        -- Given: talk(A, spot=1) → talk(nil) → talk(A, spot=1)
        -- Expected: nilアクターはスポット0、その後Aはスポット1に戻る
    end)
end)
```

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

## Existing Code Analysis: sakura_builder.lua

### 現状構造分析

**ファイル**: `pasta/shiori/sakura_builder.lua` (122行)

```
sakura_builder.lua
├── escape_sakura(text)      # ローカル関数: エスケープ処理 (再利用可能)
├── spot_to_id(spot)         # ローカル関数: spot値→数値変換 (再利用可能)
├── spot_to_tag(spot_id)     # ローカル関数: スポットタグ生成 (再利用可能)
└── BUILDER.build(tokens, config)  # 公開API: 全面書き換え対象
```

### 既存BUILDER.build()のトークン処理フロー

```
入力: フラットな tokens[] 配列
  │
  ├─── "spot"        → actor_spots[actor.name] = spot
  ├─── "clear_spot"  → 状態リセット
  ├─── "talk"        → アクター切り替え検出 + テキスト出力
  ├─── "actor"       → スポットタグ出力（レガシー形式）★削除対象
  ├─── "spot_switch" → 段落改行出力（レガシー形式）★削除対象
  ├─── "surface"     → \s[id] 出力
  ├─── "wait"        → \w[ms] 出力
  ├─── "newline"     → \n 出力
  ├─── "clear"       → \c 出力
  ├─── "sakura_script" → 生テキスト出力
  └─── "yield"       → 無視
  │
出力: さくらスクリプト文字列 + "\e"
```

### 既存テスト分析 (sakura_builder_test.lua: 521行)

| テストセクション | 行数 | 影響度 | 対応方針 |
|-----------------|------|-------|---------|
| talk token | 20行 | **要修正** | grouped形式入力に変更 |
| actor token | 50行 | **削除可能** | レガシー形式廃止 |
| spot_switch token | 30行 | **削除可能** | レガシー形式廃止 |
| surface/wait/newline/clear/sakura_script | 60行 | **要修正** | grouped形式入力に変更 |
| \\e終端 | 20行 | 変更なし | 出力形式は同じ |
| 複合シナリオ | 30行 | **要修正** | grouped形式入力に変更 |
| spotトークン処理 | 50行 | **要修正** | grouped形式入力に変更 |
| clear_spotトークン処理 | 50行 | **要修正** | grouped形式入力に変更 |
| talkトークンのactor切り替え検出 | 80行 | **要修正** | grouped形式入力に変更 |
| 統合シナリオ（新トークン構造） | 60行 | **要修正** | grouped形式入力に変更 |

**修正が必要なテストケース**: 約400行（レガシー形式テスト80行は削除可能）

### 変更対象関数の詳細

| 関数 | 行数 | 変更内容 |
|-----|------|---------|
| `escape_sakura(text)` | 5行 | **維持** - そのまま再利用 |
| `spot_to_id(spot)` | 15行 | **維持** - そのまま再利用 |
| `spot_to_tag(spot_id)` | 3行 | **維持** - そのまま再利用 |
| `BUILDER.build(tokens, config)` | 60行 | **全面書き換え** - 新関数に置換 |

---

## Implementation Strategy: Parallel Development

### 戦略: build_grouped() 並行開発 → 同等性確認後に置き換え

```
Phase 1: 新関数追加（既存関数は維持）
  ├── BUILDER.build_grouped(grouped_tokens, config)  # 新規追加
  └── BUILDER.build(tokens, config)                   # 既存維持

Phase 2: 同等性確認
  ├── 新規テスト: build_grouped()のgrouped入力テスト
  └── 統合テスト: SHIORI_ACT_IMPL.build()で新関数使用、既存テスト全パス確認

Phase 3: 移行完了
  ├── BUILDER.build() を build_grouped() で置き換え
  └── build_grouped() を廃止（ build() に統合）
  └── レガシートークン処理（actor, spot_switch）を削除
```

### Phase 1: BUILDER.build_grouped() 新規追加

**新規コード配置** (sakura_builder.lua):

```lua
--- @module pasta.shiori.sakura_builder
--- さくらスクリプトビルダーモジュール

local BUILDER = {}

-- ============================================================================
-- ローカルヘルパー関数（既存・維持）
-- ============================================================================

--- さくらスクリプト用エスケープ処理
local function escape_sakura(text)
    if not text then return "" end
    local escaped = text:gsub("\\", "\\\\")
    escaped = escaped:gsub("%%", "%%%%")
    return escaped
end

--- spotからスポットID番号を決定
local function spot_to_id(spot)
    if spot == "sakura" or spot == 0 then
        return 0
    elseif spot == "kero" or spot == 1 then
        return 1
    elseif type(spot) == "number" then
        return spot
    elseif type(spot) == "string" then
        local n = spot:match("^char(%d+)$")
        if n then return tonumber(n) end
    end
    return 0
end

--- スポットタグを生成
local function spot_to_tag(spot_id)
    return string.format("\\p[%d]", spot_id)
end

-- ============================================================================
-- 新規関数: BUILDER.build_grouped() - grouped_token[]対応
-- ============================================================================

--- グループ化トークン配列をさくらスクリプト文字列に変換
--- @param grouped_tokens grouped_token[] グループ化されたトークン配列
--- @param config BuildConfig|nil 設定
--- @return string さくらスクリプト文字列
function BUILDER.build_grouped(grouped_tokens, config)
    config = config or {}
    local spot_newlines = config.spot_newlines or 1.5
    local buffer = {}

    -- ビルダー内部状態
    local actor_spots = {}
    local last_actor = nil
    local last_spot = nil

    for _, token in ipairs(grouped_tokens) do
        local t = token.type

        if t == "spot" then
            if token.actor and token.actor.name then
                actor_spots[token.actor.name] = token.spot
            end

        elseif t == "clear_spot" then
            actor_spots = {}
            last_actor = nil
            last_spot = nil

        elseif t == "actor" then
            local actor = token.actor
            local actor_name = actor and actor.name

            -- アクター切り替え検出
            if actor and last_actor ~= actor then
                local spot = actor_spots[actor_name] or 0

                if last_spot ~= nil and last_spot ~= spot then
                    local percent = math.floor(spot_newlines * 100)
                    table.insert(buffer, string.format("\\n[%d]", percent))
                end

                table.insert(buffer, spot_to_tag(spot))
                last_actor = actor
                last_spot = spot
            end

            -- グループ内トークンを順次処理
            for _, inner in ipairs(token.tokens) do
                local inner_type = inner.type

                if inner_type == "talk" then
                    table.insert(buffer, escape_sakura(inner.text))
                elseif inner_type == "surface" then
                    table.insert(buffer, string.format("\\s[%s]", tostring(inner.id)))
                elseif inner_type == "wait" then
                    table.insert(buffer, string.format("\\w[%d]", inner.ms))
                elseif inner_type == "newline" then
                    for _ = 1, inner.n do
                        table.insert(buffer, "\\n")
                    end
                elseif inner_type == "clear" then
                    table.insert(buffer, "\\c")
                elseif inner_type == "sakura_script" then
                    table.insert(buffer, inner.text)
                end
                -- yield は無視
            end
        end
    end

    return table.concat(buffer) .. "\\e"
end

-- ============================================================================
-- 既存関数: BUILDER.build() - フラットtoken[]対応（Phase 3で削除予定）
-- ============================================================================

function BUILDER.build(tokens, config)
    -- ... 既存コード維持 ...
end

return BUILDER
```

### Phase 2: 同等性確認テスト戦略

**新規テストファイル**: `lua_specs/sakura_builder_grouped_test.lua`

```lua
-- Phase 2 同等性確認テスト
describe("SAKURA_BUILDER.build_grouped - 既存出力との同等性", function()
    test("既存テストシナリオをgrouped形式で再現し同一出力を確認", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local sakura = { name = "さくら" }
        local kero = { name = "うにゅう" }
        
        -- 既存テストの期待出力を grouped 形式入力で再現
        local grouped = {
            { type = "spot", actor = sakura, spot = 0 },
            { type = "spot", actor = kero, spot = 1 },
            { type = "actor", actor = sakura, tokens = {
                { type = "talk", actor = sakura, text = "Sakura speaks" }
            }},
            { type = "actor", actor = kero, tokens = {
                { type = "talk", actor = kero, text = "Kero speaks" }
            }}
        }
        
        local result = BUILDER.build_grouped(grouped, { spot_newlines = 1.5 })
        
        -- 既存テストの期待値と同一であることを確認
        expect(result):toBe("\\p[0]Sakura speaks\\n[150]\\p[1]Kero speaks\\e")
    end)
end)
```

### Phase 3: 移行完了

**SHIORI_ACT_IMPL.build() 変更**:

```lua
function SHIORI_ACT_IMPL.build(self)
    local grouped = ACT.IMPL.build(self)
    -- Phase 3: build_grouped() を使用
    local script = BUILDER.build_grouped(grouped, {
        spot_newlines = self._spot_newlines
    })
    return script
end
```

**削除対象コード** (sakura_builder.lua):
- 旧 `BUILDER.build()` 関数全体（60行）
- `type == "actor"` 処理（レガシー）
- `type == "spot_switch"` 処理（レガシー）

**削除対象テスト** (sakura_builder_test.lua):
- `describe("SAKURA_BUILDER - actor token", ...)` (50行)
- `describe("SAKURA_BUILDER - spot_switch token", ...)` (30行)

---

## Implementation Notes

### 実装順序（改訂版）

1. **Phase 1: 新関数追加**
   - `pasta/act.lua`に`group_by_actor()`追加
   - `pasta/act.lua`に`merge_consecutive_talks()`追加
   - `ACT_IMPL.build()`を変更（grouped_token[]を返す）
   - `pasta/shiori/sakura_builder.lua`に`BUILDER.build_grouped()`追加

2. **Phase 2: 同等性確認**
   - 新規テスト`sakura_builder_grouped_test.lua`作成
   - `SHIORI_ACT_IMPL.build()`を`build_grouped()`使用に変更
   - 既存テスト`sakura_builder_test.lua`全パス確認

3. **Phase 3: 移行完了**
   - `BUILDER.build_grouped()`を`BUILDER.build()`にリネーム
   - 旧`BUILDER.build()`を削除
   - レガシートークン処理を削除
   - レガシーテストケースを削除
   - テストを新形式に書き換え

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
