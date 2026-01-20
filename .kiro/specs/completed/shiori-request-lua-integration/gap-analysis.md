# Implementation Gap Analysis

## Feature: shiori-request-lua-integration

分析日: 2026-01-20

---

## 1. Current State Investigation

### ファイル・モジュール構成

**既存の関連資産**:

- **`crates/pasta_shiori/src/lua_request.rs`**: 移植済みだが未統合のSHIORI request→Luaテーブル変換コード
- **`crates/pasta_shiori/src/util/parsers/req.rs`**: 既存のSHIORI requestパーサー実装（Pest利用）
  - `ShioriRequest<'a>`構造体とparse関数を提供
  - `MyResult`型を使用
- **`crates/pasta_shiori/src/error.rs`**: 統一エラー型（`MyError`, `MyResult`）
  - `MyError::ParseRequest`が既に存在
  - `From<pasta_lua::mlua::Error>`実装済み
- **`crates/pasta_shiori/src/shiori.rs`**: SHIORI実装のメインロジック
  - `PastaShiori`構造体、`Shiori`トレイト
  - `pasta_lua::mlua::{Function, Table}`を使用中
- **`pasta_lua::mlua`**: mluaクレートの再エクスポート（`pasta_lua/src/lib.rs`）
- **DateTime依存**: なし（**議題1決定**: `time` v0.3.45 + `local-offset` feature採用予定）

### アーキテクチャパターン・制約

| 項目 | パターン |
|------|---------|
| エラー型 | `MyResult<T> = Result<T, MyError>` 統一 |
| mlua使用 | `pasta_lua::mlua::*`経由で再エクスポート利用 |
| パーサー | Pest PEGパーサー（`util::parsers::req`） |
| インポート | 明示的`use`（`prelude`は非推奨） |
| テストファイル | `tests/<feature>_test.rs` |

### 統合サーフェス

- **パーサー**: `util::parsers::req::ShioriRequest::parse(text: &str) -> MyResult<ShioriRequest>`
- **mlua API**: `pasta_lua::mlua::{Lua, Table}` - 既にshiori.rsで利用中
- **エラー変換**: `MyError::from(mlua::Error)` 実装済み

### 規約

| 項目 | 規約 |
|------|------|
| 命名 | スネークケース（Rust標準） |
| インポート | 明示的`use`（`crate::prelude::*`削除） |
| 依存解決 | `pasta_lua::mlua::{...}` 再エクスポート経由 |
| テスト配置 | `tests/`配下 |

---

## 2. Requirements Feasibility Analysis

### 技術要件一覧

| Requirement | 必要な技術要素 | 現状 |
|-------------|---------------|------|
| Req 1: SHIORI→Luaテーブル変換 | パーサー、mlua Table API | ✅ 両方存在 |
| Req 2: 現在時刻Luaテーブル | time crate (v0.3.x) + local-offset, mlua Table API | ⚠️ timeクレート未追加 |
| Req 3: mlua互換性更新 | `<'lua>`ライフタイム削除、pasta_lua::mlua使用 | ✅ 再エクスポート利用可 |
| Req 4: エラー型統合 | MyResult/MyError | ✅ 既存 |
| Req 5: モジュール公開 | lib.rs統合 | ✅ 標準パターン |

### ギャップ・制約

| 項目 | 状態 | 詳細 |
|------|------|------|
| time依存 | **Missing (✅ 議題1決定)** | `lua_date`関数は`time` v0.3.x + `local-offset` featureが必要。pasta_shioriのCargo.tomlに未記載。**選択理由**: 活発なメンテナンス(520M DL, 7日前更新)、安全なローカルオフセット処理(`Result`型)、軽量(17K SLoC)、Rust 2024互換 |
| `crate::prelude::*` | **Constraint** | 現在のコードベースでは非推奨（明示的インポートに変更必要） |
| `shiori3::req`依存 | **Constraint** | 外部クレート依存を削除し、`crate::util::parsers::req`使用 |
| `<'lua>`ライフタイム | **Resolved (✅ 議題2決定)** | 現行mlua (0.11+) では不要。API DOCにも記載なし。全ての`<'lua>`を削除し、`Table<'lua>` → `Table`に変更 |

### 複雑性シグナル

- **アルゴリズムロジック**: SHIORI requestのフィールドをLuaテーブルに写像（中程度）
- **外部統合**: pest パーサー利用済み、mluaも既存利用（低）
- **検証**: エラーハンドリングパターン既存（低）
  - 議題3決定: time::IndeterminateOffset等のエラーは実装時に適切なMyError variantへ変換（詳細は設計フェーズで決定）

---

## 3. Implementation Approach Options

### Option A: 既存`lua_request.rs`を直接修正・統合

**適用ケース**: 機能が明確で、既存ファイル構造が妥当

**変更対象ファイル**:
- `crates/pasta_shiori/src/lua_request.rs`: インポート修正、ライフタイム削除
- `crates/pasta_shiori/src/lib.rs`: モジュール宣言追加
- `crates/pasta_shiori/Cargo.toml`: time依存追加（lua_date用、v0.3 + local-offset feature）

**互換性評価**:
- ✅ `util::parsers::req`との統合は既存パターンと一致
- ✅ `MyError`/`MyResult`型は既存エラー処理と互換
- ⚠️ `<'lua>`ライフタイム削除後の動作確認が必要

**複雑性・保守性**:
- 現在119行の単一ファイル（小規模）
- 単一責任原則: SHIORI request→Lua変換のみ（適切）
- コード量増加: lib.rsへの1-2行追加のみ（問題なし）

**Trade-offs**:
- ✅ 最小限のファイル変更（1ファイル修正、2ファイル軽微更新）
- ✅ 既存パターン活用で学習コスト低
- ❌ chronoクレートの追加が必要（`lua_date`関数のため）
- ❌ ライフタイム削除の動作確認コスト

**推奨度**: ⭐⭐⭐⭐ （高）

---

### Option B: 新モジュール`util::lua_helpers`作成

**適用ケース**: 将来的なLua変換ユーティリティの拡張を想定

**新規作成**:
- `crates/pasta_shiori/src/util/lua_helpers/mod.rs`: モジュール基盤
- `crates/pasta_shiori/src/util/lua_helpers/request.rs`: SHIORI request変換
- `crates/pasta_shiori/src/util/lua_helpers/datetime.rs`: 日時変換

**統合ポイント**:
- `util/mod.rs`に`pub mod lua_helpers;`追加
- `shiori.rs`から`crate::util::lua_helpers::request::parse_request`使用

**責任境界**:
- `lua_helpers`: Rust型→Luaテーブル変換専用
- 既存`parsers`: テキスト→Rust型パース専用

**Trade-offs**:
- ✅ 明確な責任分離（パース vs 変換）
- ✅ 将来のLuaユーティリティ拡張が容易
- ❌ ファイル数増加（3-4ファイル追加）
- ❌ 初期開発コスト増（ディレクトリ構造設計）

**推奨度**: ⭐⭐ （中）

---

### Option C: ハイブリッド - 最小統合＋段階的リファクタリング

**適用ケース**: 初期実装を迅速化し、後で再構成

**フェーズ1（最小統合）**:
1. `lua_request.rs`を修正・統合（Option A）
2. chronoを条件付きfeatureとして追加（`lua_date`のみ）

**フェーズ2（リファクタリング）**:
- `lua_date`が不要なら削除し、chrono依存も削除
- または将来的に`util::lua_helpers`へ移動

**リスク軽減**:
- フェーズ1で動作確認後、フェーズ2で最適化
- feature flag (`[features] datetime-helpers = ["chrono"]`) でchrono依存を任意化

**Trade-offs**:
- ✅ 段階的実装でリスク分散
- ✅ 初期リリース迅速化
- ❌ 2段階計画が必要
- ❌ リファクタリングフェーズの実施保証なし

**推奨度**: ⭐⭐⭐ （中高）

---

## 4. Implementation Complexity & Risk

### 工数見積もり: **S (1-3日)**

**根拠**:
- 既存パーサー活用で実装パターン明確
- mluaインターフェース既知（shiori.rs使用実績）
- 主作業: インポート修正、ライフタイム削除、モジュール統合
- テスト: 既存`tests/shiori_lifecycle_test.rs`パターン流用可

### リスク評価: **Low**

**根拠**:
- ✅ 確立されたパターン: pest パーサー、mlua Table API
- ✅ 明確なスコープ: request→Luaテーブル変換のみ
- ✅ 統合サーフェス最小: `parse_request`関数1つが主要API
- ⚠️ `<'lua>`ライフタイム削除の動作確認必要（mluaドキュメント参照で解決可能）

**リスク項目**:
1. **chrono依存の追加**: Cargo.toml更新のみ（影響小）
2. **ライフタイム削除**: mluaの借用ルール変更に対応（ドキュメント確認で解決）
3. **`prelude::*`削除**: 明示的インポートへ変更（機械的作業）

---

## 5. Requirements-to-Asset Map

| Requirement | 既存Asset | Gap | Status |
|-------------|-----------|-----|--------|
| **Req 1: SHIORI→Luaテーブル変換** | `util::parsers::req` | インポート修正、ライフタイム削除 | **修正必要** |
| **Req 2: 現在時刻テーブル** | なし | chrono依存追加 | **Missing** |
| **Req 3: mlua互換性更新** | `pasta_lua::mlua` | `<'lua>`削除、インポート変更 | **修正必要** |
| **Req 4: エラー型統合** | `MyError`, `MyResult` | 既存利用 | ✅ **利用可** |
| **Req 5: モジュール公開** | `lib.rs` | `mod lua_request;` 追加 | **追加必要** |

---

## 6. Recommendations for Design Phase

### 推奨アプローチ: **Option A（直接修正・統合）**

**理由**:
1. **最小変更**: 既存`lua_request.rs`の修正のみで要件達成
2. **既存パターン準拠**: `util::parsers::req`活用で一貫性維持
3. **工数最小**: S (1-3日) で実装・テスト完了
4. **リスク低**: 確立されたパターン、明確なスコープ

### 主要設計決定事項

1. **chrono依存の扱い**:
   - `lua_date`関数が実際に使用されるか確認
   - 使用されない場合は関数ごと削除し、chrono依存不要
   - 使用される場合はCargo.tomlに`chrono`追加
   - **Research Needed**: Luaスクリプト側での`lua_date`利用実態調査

2. **ライフタイム記法**:
   - `<'lua>`を削除し、現行mluaの参照借用パターンに変更
   - `&Lua`参照のみでTable生成可能か検証
   - **Research Needed**: mlua 0.11のTable生成APIドキュメント確認

3. **テスト戦略**:
   - `tests/lua_request_test.rs`新規作成
   - `parse_request`のフィールド変換正確性テスト
   - `lua_date`の日時フィールドテスト（使用する場合）

### 次フェーズへの引き継ぎ事項

| 項目 | 内容 |
|------|------|
| **調査事項** | 1. `lua_date`使用実態調査<br>2. mlua 0.11 Table API仕様確認 |
| **設計決定** | 1. chrono依存の要否決定<br>2. ライフタイム記法削除パターン確定 |
| **実装準備** | 1. テストケース設計<br>2. インポート修正方針策定 |

---

## Summary

### 分析サマリー

- **スコープ**: 移植済み`lua_request.rs`の現行環境への適合と統合
- **主要課題**: 
  1. chrono依存の追加要否決定
  2. `<'lua>`ライフタイム削除対応
  3. `prelude`から明示的インポートへ変更
- **推奨アプローチ**: Option A（直接修正）- 最小変更で迅速実装
- **工数**: S (1-3日)、リスク: Low

### 次のステップ

gap分析が完了しましたわ！設計フェーズに進む準備が整いましたわよ。

**設計フェーズへ進む**:
```bash
/kiro-spec-design shiori-request-lua-integration
```

または要件承認と同時に進む場合:
```bash
/kiro-spec-design shiori-request-lua-integration -y
```

---

