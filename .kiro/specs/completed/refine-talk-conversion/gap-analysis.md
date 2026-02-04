# Gap Analysis: refine-talk-conversion

## 概要

本分析は、`sakura_builder.lua`のトーク変換処理を`escape_sakura`から`@pasta_sakura_script.talk_to_script`に置き換える要件に対する実装ギャップを評価する。

---

## 1. 現状調査

### 1.1 対象ファイル

| ファイル | パス | 役割 |
|---------|------|------|
| sakura_builder.lua | `crates/pasta_lua/scripts/pasta/shiori/sakura_builder.lua` | トークン→さくらスクリプト変換 |
| mod.rs | `crates/pasta_lua/src/sakura_script/mod.rs` | `@pasta_sakura_script`モジュール実装 |

### 1.2 現在の実装（sakura_builder.lua）

```lua
-- 行12-17: escape_sakura関数定義
local function escape_sakura(text)
    if not text then return "" end
    local escaped = text:gsub("\\", "\\\\")
    escaped = escaped:gsub("%%", "%%%%")
    return escaped
end

-- 行96-97: talkトークン処理
if inner_type == "talk" then
    table.insert(buffer, escape_sakura(inner.text))
```

**問題点**:
- 単純なエスケープ処理のみで、ウェイトタグ（`\_w[ms]`）が挿入されない
- 会話テンポが機械的で不自然

### 1.3 既存の`@pasta_sakura_script`モジュール（実装済み）

```lua
-- 使用パターン
local SAKURA_SCRIPT = require "@pasta_sakura_script"
local result = SAKURA_SCRIPT.talk_to_script(actor, "こんにちは。")
-- 結果: "こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[100]。"
```

**機能**:
- 文字種別に応じたウェイト自動挿入
- actorパラメーターによるキャラクター固有設定
- 既存さくらスクリプトタグの保護
- pasta.tomlからのデフォルト設定読み込み

### 1.4 テストカバレッジ

| テストファイル | カバレッジ | 影響 |
|---------------|-----------|------|
| `sakura_builder_test.lua` | 24テスト（act-token-buffer-refactor） | 期待値更新必須 |
| その他インテグレーションテスト | さくらスクリプト出力を検証 | 期待値更新の可能性 |

**影響範囲**:
- 現在のテストは`escape_sakura`ベースの動作を検証（単純エスケープのみ）
- 変更後は`talk_to_script`のウェイト挿入形式になる
- **すべてのさくらスクリプト出力テストの期待値修正が必要**
- 期待値修正は本仕様のスコープ内とする

---

## 2. 要件実現可能性分析

### 2.1 要件マッピング

| 要件 | 技術的実現性 | ギャップ |
|-----|------------|---------|
| Req 1: escape_sakura→talk_to_script置換 | ✅ 実現可能 | コード変更のみ |
| Req 2: モジュール読み込み追加 | ✅ 実現可能 | 1行追加 |
| Req 3: actor情報受け渡し | ✅ 実現可能 | token.actorで既に利用可能 |
| Req 4: escape_sakura削除 | ✅ 実現可能 | 関数定義削除 |
| Req 5: 互換性維持 | ✅ 実現可能 | さくらスクリプトタグ保護済み |
| Req 6: テスト期待値更新 | ✅ 実現可能 | 期待値修正（スコープ内） |

### 2.2 特記事項

**actorオブジェクトの利用**:
```lua
-- sakura_builder.lua:78
local actor = token.actor  -- ACT:build()経由でグループ化されたactorオブジェクト

-- talk_to_scriptへの渡し方
SAKURA_SCRIPT.talk_to_script(actor, inner.text)
```

**talk_to_scriptの期待するactor形式**（LUA_API.md参照）:
```lua
-- actor.talkサブテーブルがあればキャラクター固有設定を使用
local actor = {
    name = "さくら",
    spot = 0,
    talk = {
        script_wait_default = 50,
        script_wait_period = 100,
        -- ...
    }
}

-- actor.talkがない場合やnilの場合はpasta.tomlデフォルト値を使用
```

**現在のactorオブジェクト構造**（テストより）:
```lua
local actors = {
    sakura = { name = "さくら", spot = 0 },
    kero = { name = "うにゅう", spot = 1 },
}
```

**結論**: 
- 現在のactorオブジェクトには`talk`サブテーブルがない
- `talk_to_script`はactor.talkがnilの場合pasta.tomlデフォルト値にフォールバック（mod.rs:112-116で実装済み）
- キャラクター固有設定を使いたい場合、将来的にactor.talk設定機能を追加可能（本仕様のスコープ外）

---

## 3. 実装アプローチオプション

### Option A: 最小限の変更（推奨）

**戦略**: sakura_builder.luaのみ変更

**変更内容**:
1. ファイル先頭に`require "@pasta_sakura_script"`追加
2. 行96-97: `escape_sakura(inner.text)` → `SAKURA_SCRIPT.talk_to_script(actor, inner.text)`に置換
4. sakura_builder_test.luaおよび関連テストの期待値を更新

**トレードオフ**:
- ✅ 変更範囲が最小（実装1ファイル + テスト更新）
- ✅ 既存パターンを維持
- ✅ 既存の`@pasta_sakura_script`モジュールを活用
- ✅ token.actorでactorオブジェクトが利用可能（スコープ内変数として既存）
- ✅ テスト期待値更新をスコープ内として明確化actorオブジェクトが利用可能（スコープ内変数として既存）
- ❌ テストの期待値更新が必要

### Option B: テスト駆動アプローチ

**戦略**: テスト更新を先行し、実装変更を後追い

**変更内容**:
1. sakura_builder_test.luaの期待値をウェイト挿入形式に更新
2. テスト失敗を確認
3. sakura_builder.luaを変更
4. テストパス確認

**トレードオフ**:
- ✅ 回帰リスク低減
- ❌ テスト更新の工数増加

### Option C: 設定可能なハイブリッド

**戦略**: 機能フラグで新旧動作を切り替え可能に

**トレードオフ**:
- ✅ ロールバック容易
- ❌ 複雑性増加（不必要）

---

## 4. 実装複雑性とリスク

### 工数評価: **S（1-3日）**

**理由**:
- 既存パターンの適用（`@pasta_sakura_script`は実装済み）
- 変更範囲が1ファイル
- 明確なインターフェース
- actorオブジェクトはtoken.actorで既に利用可能

### リスク評価: **Low**

**理由**:
- 既存モジュールを活用
- テストカバレッジあり
- 既知の技術スタック
- actorオブジェクト渡しの実装確認済み
- 既存モジュールを活用
- テストカバレッジあり
- 既知の技術スタック

---

## 5. 推奨事項

### 設計フェーズへの推奨

1. **Opt期待値更新を実装タスクに含める（スコープ内）**
3. **actor設定はpasta.toml `[actor.名前]`で既に対応済み
3. **actor.talk構造は将来拡張として設計に明記**

### 実装詳細

**変更箇所1**: モジュール読み込み追加
```lua
-- ファイル先頭（行7付近）
local SAKURA_SCRIPT = require "@pasta_sakura_script"
```

**変更箇所2**: talk_to_script呼び出し
```lua
-- 行96-97（現在）
if inner_type == "talk" then
    table.insert(buffer, escape_sakura(inner.text))

-- 変更後
if inner_type == "talk" then
    table.insert(buffer, SAKURA_SCRIPT.talk_to_script(actor, inner.text))
```

**変更箇所3**: escape_sakura削除
```lua
-- 行12-17を削除
```
**変更箇所4**: テスト期待値更新
- `sakura_builder_test.lua`: ウェイト挿入形式の期待値に更新
- 他のインテグレーションテスト: さくらスクリプト出力変更による期待値修正

### テスト期待値の変更例

**変更前**（escape_sakuraベース）:
```lua
-- エスケープのみ
expect(result:find("Hello")):toBeTruthy()
expect(result:find("path\\\\to\\\\file")):toBeTruthy()
```

**変更後**（talk_to_scriptベース）:
```lua
-- ウェイト挿入形式
expect(result:find("H\\_w%[%d+%]e\\_w%[%d+%]l")):toBeTruthy()  -- パターンマッチ
-- または具体的なウェイト値で検証
```


### 調査不要項目

- `@pasta_sakura_script`モジュールの実装詳細（完了済み仕様
| 6.1 | sakura_builder_test.lua | 期待値更新必要 |
| 6.2 | sakura_builder_test.lua | 既存期待値削除必要 |
| 6.3 | 各インテグレーションテスト | 期待値修正可能性 |: sakura-script-wait）
- ウェイト挿入ロジック（mod.rs, tokenizer.rs, wait_inserter.rsで実装済み）
- actorオブジェクトの渡し方（token.actorで利用可能、確認済み）

---

## 6. 要件-資産マップ

| 要件ID | 資産 | ステータス |
|-------|------|-----------|
| 1.1 | sakura_builder.lua:96-97 | 変更必要 |
| 1.2 | sakura_builder.lua:96 | actor変数利用可 |
| 1.3 | sakura_builder.lua:96-97 | 削除必要 |
| 2.1 | sakura_builder.lua:先頭 | 追加必要 |
| 2.2 | 新規変数 | 追加必要 |
| 3.1 | sakura_builder.lua:78 | 既存（token.actor） |
| 3.2 | sakura_builder.lua:96 | 変更必要 |
| 3.3 | @pasta_sakura_script | 対応済み（nilフォールバック実装済み） |
| 4.1 | sakura_builder.lua:12-17 | 削除必要 |
| 4.2 | sakura_builder.lua:97 | 削除必要 |
| 5.1-5.3 | @pasta_sakura_script | 対応済み |
