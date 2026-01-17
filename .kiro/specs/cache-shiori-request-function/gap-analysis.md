# Implementation Gap Analysis

## Analysis Summary

- **スコープ**: PastaShiori 構造体における Lua 関数参照のキャッシュ実装（load/request/unload 関数）
- **主要な発見**: mlua::Function は Clone/Send/Sync 可能な通常の struct であり、キャッシュ可能
- **複雑性**: 低 - 既存パターンへの直接的な拡張、冗長コード削減が主な目的
- **推奨アプローチ**: Option A (既存コンポーネント拡張) - PastaShiori struct のフィールドリファクタリング

## 1. Current State Investigation

### 1.1 既存アセット

**主要ファイル**:
- [crates/pasta_shiori/src/shiori.rs](../../../crates/pasta_shiori/src/shiori.rs) (592行) - PastaShiori 実装
- [crates/pasta_shiori/src/error.rs](../../../crates/pasta_shiori/src/error.rs) - エラー型定義

**現在の PastaShiori 構造**:
```rust
pub(crate) struct PastaShiori {
    hinst: isize,
    load_dir: Option<PathBuf>,
    runtime: Option<PastaLuaRuntime>,
    has_shiori_load: bool,      // ← 冗長フラグ
    has_shiori_request: bool,   // ← 冗長フラグ
}
```

**既存の関数取得パターン**:
- `check_shiori_functions()`: 関数存在確認のみ（boolean フラグ設定）
- `request()`: 毎回 `globals.get("SHIORI")` → `shiori_table.get("request")` を実行
- `call_shiori_load()`: 毎回 `globals.get("SHIORI")` → `shiori_table.get("load")` を実行

### 1.2 アーキテクチャパターン

**依存関係**:
- `pasta_lua::mlua::{Function, Table}` - mlua クレートの型を直接使用
- `pasta_lua::PastaLuaRuntime` - Lua ランタイム抽象化
- `tracing` - 構造化ロギング

**テスト戦略**:
- ユニットテスト配置: 同一ファイル内の `#[cfg(test)] mod tests`
- Fixture ベース: `copy_fixture_to_temp()` でテスト環境構築
- カバレッジ: 12テストケース（lifecycle, reload, flags, error handling など）

### 1.3 mlua::Function の調査結果

**重要な確認事項**:
```rust
// mlua/function.rs より
#[derive(Clone, Debug, PartialEq)]
pub struct Function(pub(crate) ValueRef);

// Send/Sync サポート (feature = "send" 時)
#[cfg(feature = "send")]
static_assertions::assert_impl_all!(Function: Send, Sync);
```

**結論**: 
- ✅ `Function` は参照型ではなく、通常の struct
- ✅ `Clone` 実装済み → struct フィールドに保持可能
- ✅ `Send`/`Sync` 実装済み（`send` feature 有効時）→ `PastaShiori::Send/Sync` と互換
- ✅ 内部的には `ValueRef` で参照カウント管理されているため、安全にキャッシュ可能

**誤解の訂正**: 過去の分析で「参照型なのでキャッシュ不可」という意見があったとのことですが、これは誤りです。`Function` は `ValueRef` をラップした通常の struct であり、Rust の所有権システム内で安全にキャッシュできます。

## 2. Requirements Feasibility Analysis

### 2.1 技術的要件マッピング

| 要件 | 現在の状態 | ギャップ | 制約 |
|------|-----------|---------|------|
| Req 1: 関数キャッシュ構造 | boolean フラグのみ | **Missing** - `Option<Function>` フィールド追加必要 | None |
| Req 2: request() 最適化 | 毎回関数取得 | **Missing** - キャッシュ利用ロジック必要 | None |
| Req 3: load() 最適化 | 毎回関数取得 | **Missing** - キャッシュ利用ロジック必要 | None |
| Req 4: unload 関数サポート | 未実装 | **Missing** - Drop impl への追加必要 | None |
| Req 5: フラグ統合 | boolean フラグ使用中 | **Missing** - Option::is_some() への移行必要 | テスト互換性維持 |
| Req 6: テスト互換性 | 12テスト存在 | **Constraint** - 既存テストが動作必須 | 特に has_shiori_* アサーション |

### 2.2 データモデル変更

**変更前**:
```rust
has_shiori_load: bool,
has_shiori_request: bool,
```

**変更後**:
```rust
load_fn: Option<Function>,
request_fn: Option<Function>,
unload_fn: Option<Function>,
```

**メリット**:
- 関数存在チェックと関数参照を単一フィールドで管理
- 関数ルックアップの削減（パフォーマンス向上）
- コード重複削減（DRY原則）

### 2.3 複雑性シグナル

- **タイプ**: リファクタリング - 既存ロジックの最適化
- **統合**: シンプル - 同一ファイル内で完結
- **外部依存**: なし - mlua API の既知パターン利用
- **パフォーマンス影響**: 正 - 各リクエストでのハッシュテーブルルックアップ削減

## 3. Implementation Approach Options

### Option A: 既存コンポーネント拡張 ⭐ **推奨**

**実装詳細**:
1. **struct フィールド変更**:
   - `has_shiori_load: bool` → 削除
   - `has_shiori_request: bool` → 削除
   - `load_fn: Option<Function>` → 追加
   - `request_fn: Option<Function>` → 追加
   - `unload_fn: Option<Function>` → 追加

2. **check_shiori_functions() リファクタリング**:
   - 関数取得時に `Option<Function>` として保存
   - エラー時は `None` を設定

3. **request() 簡素化**:
   - `self.request_fn` の存在確認のみ（`.is_some()`）
   - キャッシュ済み関数を直接使用

4. **call_shiori_load() 簡素化**:
   - `self.load_fn` を直接使用
   - 関数ルックアップ処理を削除

5. **Drop impl 拡張**:
   - `self.unload_fn` が `Some` の場合、呼び出し
   - エラー発生時はログのみ（伝播しない）

6. **テスト修正**:
   - `has_shiori_load` → `load_fn.is_some()` に変更
   - `has_shiori_request` → `request_fn.is_some()` に変更

**互換性評価**:
- ✅ 外部 API (Shiori trait) 不変
- ✅ 内部ロジック変更のみ
- ✅ 既存テストは最小限の修正で動作

**複雑性とメンテナビリティ**:
- ✅ 単一責任原則維持（関数キャッシュ追加はコア責務の延長）
- ✅ ファイルサイズ: 592行 → ~600行（微増）
- ✅ 認知負荷: 削減（冗長なルックアップ処理削除）

**Trade-offs**:
- ✅ 最小限のコード変更
- ✅ 既存パターンとの整合性維持
- ✅ テストカバレッジ再利用
- ⚠️ 構造体フィールド増加（boolean 2個 → Option<Function> 3個）
  - しかし、各 Function は軽量（ValueRef のみ保持）
  - メモリオーバーヘッドは無視可能

### Option B: 新規コンポーネント作成

**却下理由**:
- 単純な最適化に対して過剰設計
- PastaShiori のコア責務からの切り離しは不自然
- テスト分離によるメリットなし（既存テストで十分）

### Option C: ハイブリッドアプローチ

**却下理由**:
- このスコープには不要
- 段階的移行の必要性なし（互換性破壊なし）

## 4. 技術調査項目

以下の項目は設計フェーズで対応済み:

- ✅ **mlua::Function の所有権セマンティクス**: ValueRef による参照カウント、Clone/Send/Sync 実装確認
- ✅ **Drop 時のエラーハンドリング**: エラー伝播せず、ログ記録のみが SHIORI 仕様に準拠
- ✅ **既存テストの影響範囲**: フラグアサーション箇所のみ修正必要（12テスト中2テスト）

**追加調査不要**: 既存コードベースと mlua ドキュメントで十分な情報確認済み

## 5. Implementation Complexity & Risk

### Effort: **S (1-2 days)**

**理由**:
- 既存パターンの直接的な拡張
- 変更スコープ: 単一ファイル、4メソッド、12テスト中2テストのアサーション修正
- 新規依存なし、新規 API 設計なし

**内訳**:
- 構造体とメソッド変更: 0.5日
- テスト修正と検証: 0.5日
- unload 機能追加とテスト: 0.5日（新規機能）

### Risk: **Low**

**理由**:
- **技術的確実性**: mlua::Function のキャッシュ可能性を確認済み
- **統合リスク**: 外部 API 不変、内部リファクタリングのみ
- **パフォーマンスリスク**: 改善のみ（ルックアップ削減）
- **セキュリティリスク**: なし（既存 Lua サンドボックスに依存）
- **テストカバレッジ**: 既存12テストで主要シナリオカバー済み

**潜在的リスク**:
- ⚠️ **Drop 時の unload 呼び出し順序**: runtime drop 前に unload 呼び出し必要
  - **緩和策**: Drop impl 内で明示的に順序制御
- ⚠️ **複数回 load() 時のキャッシュクリア忘れ**: 古い関数参照が残る可能性
  - **緩和策**: 既存の `self.runtime = None` と同時にキャッシュクリア

## 6. 推奨実装戦略

### 推奨: Option A - 既存コンポーネント拡張

**理由**:
1. **最小変更原則**: 既存コードへの影響最小化
2. **明確な価値**: パフォーマンス向上と冗長コード削減
3. **低リスク**: テスト互換性維持、外部 API 不変
4. **シンプル性**: 新規ファイル不要、既存パターン踏襲

### 設計フェーズでの主要決定事項

1. **Drop impl での unload 呼び出し順序**:
   - unload → runtime drop の順序を明示
   - エラーハンドリング: ログのみ、伝播しない

2. **キャッシュクリアタイミング**:
   - reload 時: 新 runtime 初期化前にクリア
   - Drop 時: unload 呼び出し後、runtime と共にクリア

3. **テスト修正範囲**:
   - `test_load_sets_shiori_flags_when_main_lua_exists()`
   - `test_load_flags_false_without_main_lua()`
   - 上記2テストのアサーション修正のみ

4. **新規テスト追加**:
   - `test_unload_called_on_drop()` - unload 関数呼び出し確認
   - `test_unload_error_does_not_panic()` - unload エラー時の resilience

### 実装順序

1. **Phase 1**: struct フィールド変更と check_shiori_functions() リファクタリング
2. **Phase 2**: request() と call_shiori_load() 簡素化
3. **Phase 3**: Drop impl への unload サポート追加
4. **Phase 4**: テスト修正と新規テスト追加
5. **Phase 5**: 統合テストと性能検証

## 7. 要件-アセットマッピング

| 要件 ID | 必要な変更 | 既存アセット | ギャップタグ | 実装アプローチ |
|---------|-----------|-------------|-------------|---------------|
| Req 1 | `Option<Function>` フィールド追加 | PastaShiori struct | **Missing** | フィールド定義変更 |
| Req 2 | request() キャッシュ利用 | request() メソッド | **Missing** | 関数ルックアップ削除 |
| Req 3 | call_shiori_load() キャッシュ利用 | call_shiori_load() メソッド | **Missing** | 関数ルックアップ削除 |
| Req 4 | unload 関数呼び出し | Drop impl | **Missing** | Drop impl 拡張 |
| Req 5 | boolean フラグ削除 | has_shiori_* フィールド | **Missing** | フィールド削除 + Option::is_some() 利用 |
| Req 6 | テスト互換性 | 既存12テスト | **Constraint** | アサーション修正（2テスト） |

## 8. 前提条件と依存関係

**前提条件**:
- ✅ mlua クレート（既存依存）
- ✅ pasta_lua::PastaLuaRuntime（既存依存）
- ✅ テスト環境（fixtures, tempfile）

**外部依存**:
- なし - 既存依存のみ使用

**ブロッカー**:
- なし - 即座に実装可能

## 9. 結論

この機能は **低リスク・高価値** のリファクタリングです。mlua::Function が通常の struct であることを確認し、キャッシュ可能性が保証されました。既存の PastaShiori 構造体への直接的な拡張が最適なアプローチであり、1-2日で実装可能と見積もられます。

**次のアクション**: 設計フェーズ (`/kiro-spec-design cache-shiori-request-function`) に進み、詳細な実装設計を策定してください。
