# ギャップ分析レポート: ontalk-sample-ghost-bug

## 1. 現状調査

### 1.1 関連アセット一覧

| カテゴリ | ファイル | 役割 |
|---------|---------|------|
| シーンテーブル | `crates/pasta_core/src/registry/scene_table.rs` | `resolve_scene_id` / `resolve_scene_id_unified` — 候補のキャッシュ消費モデル |
| キャッシュ構造体 | 同上 `CachedSelection` (L77-81) | `candidates`, `next_index`, `history` |
| ランダム選択 | `crates/pasta_core/src/registry/random.rs` | `RandomSelector` トレイト（`shuffle_usize` メソッド含む） |
| エラー型 | `crates/pasta_core/src/error.rs` | `SceneTableError::NoMoreScenes` |
| 検索コンテキスト | `crates/pasta_lua/src/search/context.rs` | `NoMoreScenes` → `Ok(None)` 変換 |
| 仮想ディスパッチャ | `crates/pasta_lua/scripts/pasta/event/virtual_dispatcher.lua` | `check_talk` — OnTalkタイマー制御 |
| サンプルゴースト設定 | `crates/pasta_sample_ghost/ghosts/pasta/pasta.toml` | `talk_interval_min=180`, `talk_interval_max=300` |
| 設定読み込み | `crates/pasta_lua/src/config.rs` | TOML → Lua テーブル登録 |
| シーンモジュール | `crates/pasta_lua/scripts/pasta/scene.lua` | `SCENE.co_exec()` / `SCENE.search()` |

### 1.2 アーキテクチャパターン

- **2レイヤー構成**: `pasta_core`（言語非依存） → `pasta_lua`（Luaバックエンド）
- **エラー変換パターン**: Rust `SceneTableError` → `Ok(None)` → Lua `nil`
- **設定フロー**: `pasta.toml` → Rust `register_config_module` → Lua `@pasta_config` → `virtual_dispatcher.get_config()`
- **テスト配置**: `scene_table.rs` 内にインラインテスト（`#[cfg(test)]` モジュール）

### 1.3 既存の統合ポイント

- `SceneTable` は `RandomSelector` トレイトに依存（DI）
- `DefaultRandomSelector` は `shuffle_usize` を実装済み — サイクルリセット時の再シャッフルに利用可能
- `MockRandomSelector` は `shuffle_usize` が No-Op — テストでの決定的動作を保証
- `virtual_dispatcher` の `next_talk_time` は発火の成否に関わらず再計算される

---

## 2. 要件実現可能性分析

### Requirement 1: シーンキャッシュの循環リセット

| 技術要素 | 状態 | 詳細 |
|----------|------|------|
| `CachedSelection` 構造体 | ✅ 存在 | `next_index`, `candidates`, `history` フィールド |
| `shuffle_enabled` フラグ | ✅ 存在 | `SceneTable` フィールドとして定義済み |
| `RandomSelector::shuffle_usize` | ✅ 存在 | トレイトメソッドとして定義・実装済み |
| サイクルリセットロジック | 🔴 Missing | `NoMoreScenes` エラーを返すのみ、リセットなし |

**修正箇所**: `scene_table.rs` 内の2メソッド（`resolve_scene_id` / `resolve_scene_id_unified`）

**制約**: 既存の `NoMoreScenes` エラーバリアントは外部に公開されている可能性があるため、即座に削除はせず残置が安全。

### Requirement 2: サンプルゴーストの発動間隔

| 技術要素 | 状態 | 詳細 |
|----------|------|------|
| 設定ファイル | ✅ 存在 | `pasta.toml` に `talk_interval_min/max` 定義済み |
| 設定読み込みチェーン | ✅ 動作 | TOML → Rust → Lua 完全パイプライン |
| デフォルト値フォールバック | ✅ 存在 | Lua側 `get_config()` でハードコード（180/300） |

**修正箇所**: `pasta.toml` の2値のみ（180→60, 300→90）

### Requirement 3: 継続性テスト

| 技術要素 | 状態 | 詳細 |
|----------|------|------|
| テストインフラ | ✅ 存在 | `scene_table.rs` 内に `#[cfg(test)]` モジュール |
| `MockRandomSelector` | ✅ 存在 | 決定的テスト用 |
| サイクリングテスト | 🔴 Missing | 候補数超過のテストケースなし |
| 既存テストの競合 | ⚠️ Constraint | `test_resolve_scene_id_unified_local_scope` が `NoMoreScenes` を正として検証 — 修正後にアサーション逆転が必要 |

---

## 3. 実装アプローチ選択肢

### Option A: 既存コンポーネント拡張（推奨）

**対象**: `scene_table.rs` の `resolve_scene_id` / `resolve_scene_id_unified` を直接修正

**変更内容**:
1. `NoMoreScenes` エラーを返す代わりに `next_index` を `0` にリセット
2. `shuffle_enabled` なら `shuffle_usize` で再シャッフル
3. `history` をクリア
4. リセット後に次の候補を返す

**既存テストへの影響**:
- `test_resolve_scene_id_unified_local_scope`: `NoMoreScenes` を期待するアサーションを、成功を期待するように変更
- 新規テスト: サイクリング検証を追加

**トレードオフ**:
- ✅ 最小変更量（`scene_table.rs` の2箇所 + `pasta.toml` の2値）
- ✅ 既存のDIインフラ（`RandomSelector`）をそのまま活用
- ✅ テスト基盤が整備済み
- ❌ `NoMoreScenes` エラーバリアントがデッドコード化する可能性

### Option B: 新コンポーネント作成

**対象**: `CyclicSceneSelector` のような新しい選択戦略を作成

**概要**: Strategy パターンで循環 / 非循環を選択可能にする

**トレードオフ**:
- ✅ 柔軟性が高い（将来的に「一巡で停止」戦略も保持可能）
- ❌ 過剰設計（現時点で「一巡停止」を望むユースケースが不明）
- ❌ 変更量が増加し、リスクも増える

### Option C: ハイブリッドアプローチ

**概要**: Phase 1 で Option A を実装し、将来的に Strategy パターンが必要になった場合に Option B へリファクタリング

**トレードオフ**:
- ✅ 即座にバグ修正
- ✅ 将来の拡張余地を残す
- ❌ Option A で十分な可能性が高く、Phase 2 が不要になりうる

---

## 4. 実装複雑度 & リスク

| 項目 | 評価 | 理由 |
|------|------|------|
| **工数** | **S（1〜3日）** | 既存パターンの拡張、修正箇所が明確かつ限定的 |
| **リスク** | **Low** | 確立済みパターンの拡張、既知の技術スタック、明確なスコープ、最小限の統合影響 |

---

## 5. 要件→アセットマップ

| 要件 | 対象アセット | ギャップ | 変更種別 |
|------|-------------|---------|---------|
| R1: キャッシュ循環 | `scene_table.rs` (`resolve_scene_id`, `resolve_scene_id_unified`) | 🔴 Missing: リセットロジック | 既存メソッド修正（2箇所） |
| R2: 発動間隔 | `pasta.toml` (`talk_interval_min/max`) | 🔴 Missing: 値が不適切 | 設定値変更（2値） |
| R3: テスト | `scene_table.rs` テストモジュール | 🔴 Missing: サイクリングテスト | テスト追加 + 既存テスト修正 |

---

## 6. 設計フェーズへの推奨事項

### 推奨アプローチ: Option A（既存コンポーネント拡張）

**理由**:
- 修正箇所が明確かつ限定的（Rust 2箇所 + TOML 2値）
- 既存のDIインフラ（`RandomSelector`, `shuffle_enabled`）がそのまま利用可能
- 「一巡で停止」を望むユースケースが現時点で存在しない
- テストインフラが整備済みで、追加テスト作成が容易

### 設計フェーズで決定すべき事項

1. **`NoMoreScenes` エラーバリアントの扱い**: 残置 vs 削除 vs `#[allow(dead_code)]`
2. **`history` のクリアタイミング**: リセット時に全クリアか、直前N件を保持するか
3. **既存テストの修正方針**: `NoMoreScenes` を期待するテストの書き換え方法
