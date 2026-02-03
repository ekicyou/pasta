# Research Log: actor-talk-grouping

## 調査日: 2026-02-03

---

## 1. アーキテクチャ調査

### 1.1 現行トークン処理フロー

```
シーン関数(act)
  → ACT_IMPL.talk()/surface()/wait()/...
  → self.token[]（フラット配列）
  → ACT_IMPL.build()
  → token[]
  → SHIORI_ACT_IMPL.build()
  → BUILDER.build(tokens, config)
  → さくらスクリプト文字列
```

### 1.2 関連モジュール構造

| モジュール | 責務 | 主要メソッド |
|-----------|------|-------------|
| `pasta.act` | トークン蓄積、基底ビルド | `ACT_IMPL.build()` |
| `pasta.shiori.act` | さくらスクリプト生成 | `SHIORI_ACT_IMPL.build()` |
| `pasta.shiori.sakura_builder` | トークン→さくらスクリプト変換 | `BUILDER.build(tokens, config)` |

### 1.3 継承チェーン

```lua
-- pasta/shiori/act.lua
local SHIORI_ACT_IMPL = {}
setmetatable(SHIORI_ACT_IMPL, { __index = ACT.IMPL })
```

SHIORI_ACT_IMPLはACT.IMPLを継承し、`build()`のみをオーバーライド。

---

## 2. トークン構造調査

### 2.1 現行トークンタイプ

```lua
{ type = "talk",         actor = <Actor>, text = <string> }
{ type = "spot",         actor = <Actor>, spot = <number> }
{ type = "surface",      id = <number|string> }
{ type = "wait",         ms = <number> }
{ type = "newline",      n = <number> }
{ type = "clear" }
{ type = "clear_spot" }
{ type = "sakura_script", text = <string> }
```

### 2.2 トークン分類（2026-02-03改訂）

トークンは以下の2カテゴリに分類されますわ：

| カテゴリ | トークンタイプ | グループ化 | 説明 |
|---------|---------------|-----------|------|
| **アクター属性設定** | `spot`, `clear_spot` | ❌ 対象外 | 後続の行動に影響する属性変更 |
| **アクター行動** | `talk`, `surface`, `wait`, `newline`, `clear`, `sakura_script` | ✅ 対象 | アクターの具体的な行動 |

**根拠**:
- `spot`は「次にこのアクターが話すときのスポット」という属性設定
- `talk`は「今このアクターが話す」という即時の行動
- 属性設定はtalkグループ化の妨げにならないよう独立トークンとして維持

### 2.3 アクター参照パターン

- `talk.actor`: 発話者（必須）
- `spot.actor`: スポット変更対象（必須）
- 他のトークン: actorフィールドなし

### 2.4 アクター比較

Luaのテーブル参照比較（`==`）で同一性判定可能。

---

## 3. sakura_builderの内部状態管理

### 3.1 build()内の状態変数

```lua
local actor_spots = {} -- {[actor_name]: spot_id}
local last_actor = nil -- 最後に発言したActor
local last_spot = nil  -- 最後のスポットID
```

### 3.2 アクター切り替え検出ロジック

```lua
if actor and last_actor ~= actor then
    -- actor切り替え検出 → スポットタグ出力
    local spot = actor_spots[actor_name] or 0
    if last_spot ~= nil and last_spot ~= spot then
        -- spot変更時に段落区切り改行
        table.insert(buffer, string.format("\\n[%d]", percent))
    end
    table.insert(buffer, spot_to_tag(spot))
    last_actor = actor
    last_spot = spot
end
```

**発見事項**: アクター切り替え検出は`talk`トークン処理時のみ発生。`spot`トークンは`actor_spots`マップを更新するのみ。

---

## 4. テストカバレッジ分析

### 4.1 既存テストセクション（sakura_builder_test.lua）

| describe | テスト数 | カバー範囲 |
|----------|---------|-----------|
| talk token | 3 | エスケープ処理 |
| actor token | 6 | スポットタグ変換 |
| spot_switch token | 3 | 段落区切り改行 |
| surface token | 3 | サーフェス変換 |
| wait token | 3 | 待機タグ変換 |
| newline token | 2 | 改行タグ変換 |
| clear token | 1 | クリアタグ変換 |
| sakura_script token | 2 | 直接挿入 |
| 統合テスト | 多数 | 複合シナリオ |

### 4.2 後方互換性検証

既存の521行のテストが回帰テストとして機能。グループ化後も全テストがパスすれば後方互換性が保証される。

---

## 5. 設計決定事項

### 5.1 グループ化の実装箇所

**決定**: `pasta.act`モジュールの`ACT_IMPL.build()`

**根拠**:
- 責務分離: トークン構造化は基底レイヤーの責務
- 再利用性: 非SHIORIバックエンドでもグループ化を利用可能
- 拡張性: 将来のフィルター機能追加が容易

### 5.2 SHIORI_ACTでの処理

**決定**: グループをフラット化してsakura_builderに渡す

**根拠**:
- sakura_builderは既にアクター切り替え検出ロジックを持つ
- sakura_builder変更を最小化（後方互換性）
- 将来的にsakura_builderがグループ対応する際に変更しやすい

### 5.3 グループ化トリガー

**決定**: `talk.actor`の変化のみでグループ分割

**根拠**:
- `spot`は属性変更（遅延適用）→ 独立トークンとして出力
- `talk`は実際の行動（即時適用）→ グループ化トリガー
- 論理的にはtalkがアクターの「行動」を表現

### 5.4 出力構造（2026-02-03改訂）

**決定**: 3種類のトークン構造を出力

```lua
grouped_token[] = [
    { type = "spot", actor = <Actor>, spot = 1 },       -- 独立
    { type = "actor", actor = <Actor>, tokens = [...] }, -- グループ
    { type = "clear_spot" },                             -- 独立
]
```

**根拠**:
- `spot`/`clear_spot`はtalkグループ化の妨げにならないよう独立維持
- `type="actor"`でアクター行動をグループ化
- SHIORI_ACT_IMPL.build()でフラット化してsakura_builderに渡す

---

## 6. 実装アプローチ

### 6.1 推奨オプション: Option C

`pasta/act.lua`内に3つのローカル関数を追加:
1. `group_by_actor(tokens)` → grouped_token[]（spot/clear_spot独立、他はtype="actor"グループ）
2. `merge_consecutive_talks(groups)` → grouped_token[]（連続talk統合済み）
3. `flatten_grouped_tokens(groups)` → token[]（SHIORI_ACT用フラット化）

### 6.2 変更ファイル一覧

| ファイル | 変更内容 | 影響度 |
|---------|---------|--------|
| `pasta/act.lua` | ローカル関数追加、build()変更 | 中 |
| `pasta/shiori/act.lua` | build()でflatten_grouped_tokens()使用 | 小 |
| テストファイル | 新規テスト追加 | 中 |

---

## 7. リスク評価

| リスク | 影響度 | 対策 |
|--------|-------|------|
| 既存テスト失敗 | 高 | フラット化で完全互換性維持 |
| パフォーマンス低下 | 低 | O(n)アルゴリズム設計 |
| メモリ増加 | 低 | 一時的なグループ構造のみ |

---

## 8. 未解決事項

- [ ] `merge_consecutive_talks()`のオプション化設計詳細（R7-2対応）
- [ ] 将来のsakura_builderグループ直接対応の設計

---

## 9. 設計改訂履歴

| 日付 | 改訂内容 |
|------|---------|
| 2026-02-03 | 初版作成 |
| 2026-02-03 | トークン分類改訂：spot/clear_spotを独立トークン化、3種類の出力構造に変更 |
