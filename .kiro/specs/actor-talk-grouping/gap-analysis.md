# Gap Analysis: actor-talk-grouping

## Executive Summary

本分析では、`pasta.act`モジュールにおけるアクター切り替え単位でのトークングループ化機能の実装ギャップを評価する。

### 主要発見事項

- ✅ **既存資産が充実**: `act.lua`は純粋関数設計の`build()`を持ち、拡張に適した構造
- ✅ **テストカバレッジ良好**: `sakura_builder_test.lua`に521行のテストが存在し、後方互換性検証の基盤がある
- ⚠️ **中間表現の欠如**: 現在はフラットなトークン配列を返しており、グループ化構造がない
- ✅ **責務分離が明確**: `ACT_IMPL.build()`でグループ化、`SHIORI_ACT_IMPL.build()`でさくらスクリプト生成

### 設計決定（2026-02-03）

- **グループ化の実装箇所**: `pasta.act`モジュールの`ACT_IMPL.build()`
- **後段処理**: `SHIORI_ACT_IMPL.build()`がグループ化されたトークンを処理
- **後方互換性**: 最終出力（さくらスクリプト）は変化なし
- **トークン分類（2026-02-03追記）**:
  - **アクター属性設定**: `spot`, `clear_spot` - グループ化対象外、独立トークンとして維持
  - **アクター行動**: `talk`, `surface`, `wait`, `newline`, `clear`, `sakura_script` - グループ化対象

---

## 1. Current State Investigation

### 1.1 対象ファイル・モジュール

| ファイル | 役割 | 行数 |
|---------|------|------|
| [pasta/act.lua](../../../crates/pasta_lua/scripts/pasta/act.lua) | 基底アクション（トークン蓄積・build） | ~260行 |
| [pasta/shiori/act.lua](../../../crates/pasta_lua/scripts/pasta/shiori/act.lua) | SHIORI専用アクション（build()オーバーライド） | ~130行 |
| [pasta/shiori/sakura_builder.lua](../../../crates/pasta_lua/scripts/pasta/shiori/sakura_builder.lua) | トークン→さくらスクリプト変換 | ~120行 |

### 1.2 既存アーキテクチャパターン

**トークン処理フロー（現状）**:
```
self.token[] → ACT_IMPL.build() → token[] → SHIORI_ACT_IMPL.build() → BUILDER.build() → さくらスクリプト
```

**トークン処理フロー（変更後）**:
```
self.token[] → ACT_IMPL.build() [グループ化] → grouped_token[] → SHIORI_ACT_IMPL.build() → BUILDER.build() → さくらスクリプト
```

**グループ化後のトークン構造**:
```lua
grouped_token[] = [
    { type = "spot", actor = <Actor>, spot = 1 },       -- 独立トークン
    { type = "actor", actor = <Actor>, tokens = [...] }, -- グループ化トークン
    { type = "clear_spot" },                             -- 独立トークン
]
```

**`ACT_IMPL.build()`の現状**:
```lua
function ACT_IMPL.build(self)
    local token = self.token
    self.token = {}
    return token
end
```

### 1.3 既存の規約・パターン

- **純粋関数設計**: 副作用なし、内部状態は関数スコープ
- **MODULE/MODULE_IMPL分離**: クラス設計パターン準拠
- **テスト配置**: `crates/pasta_lua/tests/lua_specs/`
- **命名規約**: `UPPER_CASE`モジュール、`snake_case`ローカル

---

## 2. Requirement-to-Asset Map

| Requirement | 既存資産 | ギャップ |
|------------|---------|---------|
| R1: グループ化後トークン構造定義 | - | **Missing**: 新規データ構造が必要 |
| R2: グループ化ロジック | - | **Missing**: `ACT_IMPL.build()`に追加が必要 |
| R3: 連続talk統合 | - | **Missing**: 新規関数が必要 |
| R4: SHIORI_ACT対応 | `SHIORI_ACT_IMPL.build()` | **Extend**: グループ化トークン処理に対応 |
| R5: エッジケース | - | **Missing**: テストケース追加が必要 |
| R6: 後方互換性 | `sakura_builder_test.lua` (521行) | **OK**: 既存テストで検証可能 |
| R7: 将来拡張準備 | 純粋関数設計 | **OK**: 設計原則に合致 |

---

## 3. Implementation Approach Options

### Option A: ACT_IMPL.build()内に全ロジック埋め込み

**概要**: `ACT_IMPL.build()`内でグループ化・統合処理を直接実装

**変更対象**:
- `pasta/act.lua`のみ

**メリット**:
- ✅ 変更範囲最小
- ✅ シンプルな実装

**デメリット**:
- ❌ `act.lua`が肥大化
- ❌ テストがact.luaに依存

**Trade-offs**: 短期的には最速だが、保守性に懸念

---

### Option B: 別モジュール（token_grouper.lua）に分離

**概要**: `pasta/token_grouper.lua`を新規作成し、グループ化・統合ロジックを分離

**変更対象**:
- `pasta/token_grouper.lua`（新規）
- `pasta/act.lua`（require追加、呼び出し）

**メリット**:
- ✅ 責務分離が明確
- ✅ 単体テストが容易
- ✅ 将来のフィルター機能追加が容易

**デメリット**:
- ❌ ファイル数増加
- ❌ モジュール間依存が増える

**Trade-offs**: 中程度の作業量だが、将来拡張性が高い

---

### Option C: ACT内ローカル関数として追加（推奨）

**概要**: `pasta/act.lua`内に`group_by_actor()`と`merge_consecutive_talks()`をローカル関数として追加し、`ACT_IMPL.build()`から呼び出す

**変更対象**:
- `pasta/act.lua`のみ（1ファイル）
- `pasta/shiori/act.lua`（グループ対応、フラット化）

**メリット**:
- ✅ 変更範囲最小
- ✅ 内部関数として適切にカプセル化
- ✅ 将来のモジュール分離も容易（リファクタ時に抽出可能）

**デメリット**:
- ❌ ファイルサイズ増加（~60行追加で~320行に）

**Trade-offs**: 最もバランスが取れた選択肢

---

## 4. Implementation Complexity & Risk

### Effort Estimate: **S（1-3日）**

**理由**:
- 既存パターンの踏襲で実装可能
- 新規外部依存なし
- テストフレームワーク整備済み

### Risk Level: **Low**

**理由**:
- 既存の純粋関数設計に適合
- 出力の完全互換性が検証可能（既存テスト521行）
- 影響範囲が`act.lua`と`shiori/act.lua`に限定

---

## 5. Research Items

### 確認不要（既存実装から判明）

- トークン構造（`{type, actor, text, ...}`）
- アクター比較方法（オブジェクト参照比較）
- エスケープ処理（既存関数再利用可能）

### 設計フェーズで検討（解決済み）

- ✅ `SHIORI_ACT_IMPL.build()`でのグループ処理方法 → フラット化
- ✅ `BUILDER.build()`のグループ対応要否 → 不要（フラット化で対応）
- ✅ `merge_consecutive_talks()`のオプション化設計 → 将来対応
- ✅ `spot`, `clear_spot`の扱い → 独立トークンとして維持

---

## 6. Recommendations

### 推奨アプローチ

**Option C（ACT内ローカル関数追加）** を推奨

### 設計フェーズでの決定事項

1. **前処理関数の配置**: `act.lua`内ローカル関数
2. **ACT_IMPL.build()の変更**: グループ化済み`grouped_token[]`を返す
3. **SHIORI_ACT_IMPL.build()の変更**: `type="actor"`をフラット化してBUILDER.build()に渡す
4. **後方互換性**: 既存テストの全パスを確認

### テスト戦略

1. 新規テストセクション: `describe("ACT - actor grouping", ...)`
2. 既存テストは変更なし（回帰テストとして機能）
3. エッジケース: nil actor、断続的actor、空文字列、spot/clear_spot独立性

---

## Appendix: Token Structure Reference

### 入力トークン（フラット配列）

```lua
-- アクター属性設定（グループ化対象外）
{ type = "spot", actor = <Actor>, spot = <number> }
{ type = "clear_spot" }

-- アクター行動（グループ化対象）
{ type = "talk", actor = <Actor>, text = "発話テキスト" }
{ type = "surface", id = <number|string> }
{ type = "wait", ms = <number> }
{ type = "newline", n = <number> }
{ type = "clear" }
{ type = "sakura_script", text = <string> }
```

### 出力トークン（グループ化後）

```lua
-- グループ化後の出力は3種類のトークンで構成される

-- 1. spotトークン（独立）
{ type = "spot", actor = <Actor>, spot = <number> }

-- 2. clear_spotトークン（独立）
{ type = "clear_spot" }

-- 3. actorトークン（グループ）
{
    type = "actor",
    actor = <Actor>,
    tokens = {
        { type = "talk", actor = <Actor>, text = "今日は晴れでした。" },
        { type = "surface", id = 10 },
    }
}
```
