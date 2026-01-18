# Gap Analysis

## スコープと課題の概要

本分析は`shiori-lifecycle-lua-execution-test`の実装ギャップを評価する。現行の`test_full_shiori_lifecycle`は、Lua実行の実証可能性に欠けており、SHIORI関数呼び出しやPasta DSLトランスパイル実行を検証できていない。本分析では、既存テストインフラを活用しつつ、観測可能な副作用を持つテストフィクスチャの設計を提案する。

### 主要な発見
- 既存の`copy_fixture_to_temp`パターンは再利用可能
- `PastaLuaRuntime::lua()`により、LuaグローバルState直接アクセスが可能
- `main.lua`フィクスチャは動的生成が必要（固定応答の問題）
- `tempfile::TempDir`により、ファイルマーカーベースの検証が実現可能

---

## 1. Current State Investigation

### 1.1 Key Files and Modules

| ファイル | 責務 | 再利用性 |
|---------|------|---------|
| `crates/pasta_shiori/src/shiori.rs` | PastaShiori実装、テストコード | **High** - 既存テスト構造をテンプレートとして利用可能 |
| `crates/pasta_lua/tests/fixtures/loader/minimal/` | 最小構成フィクスチャ | **Medium** - 構造は参考になるが、固定応答の問題がある |
| `crates/pasta_lua/scripts/pasta/shiori/main.lua` | SHIORI関数定義 | **Low** - 固定応答で観測可能な副作用なし |
| `crates/pasta_lua/src/loader/mod.rs` | PastaLoader実装 | **High** - 既存APIをそのまま使用 |
| `crates/pasta_lua/src/runtime/mod.rs` | PastaLuaRuntime実装 | **High** - `lua()`メソッドで直接アクセス可能 |

### 1.2 Reusable Components

**テストヘルパー関数（pasta_shiori/src/shiori.rs）:**
```rust
fn copy_fixture_to_temp(fixture_name: &str) -> TempDir
fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()>
```
- 19個のテストで使用中
- scripts/、scriptlibs/自動コピー機能あり
- **再利用推奨**: 新規フィクスチャ用に拡張

**ランタイムアクセスAPI（pasta_lua/src/runtime/mod.rs）:**
```rust
pub fn lua(&self) -> &Lua  // Line 216
pub fn exec(&self, script: &str) -> LuaResult<Value>  // Line 195
```
- Luaグローバル変数への直接アクセスが可能
- **活用推奨**: テスト検証でLua状態を読み取る

**Loaderパターン（pasta_lua/tests/loader_integration_test.rs）:**
- 14テストが`PastaLoader::load()`を使用
- `runtime.exec()`でLuaコード実行を検証
- **参考推奨**: 検証パターンの模倣

### 1.3 Architecture Patterns

| パターン | 実装箇所 | 適用可否 |
|---------|---------|---------|
| Fixture-based testing | `pasta_lua/tests/fixtures/loader/*` | ✅ **必須** - 新規フィクスチャ追加 |
| TempDir isolation | `tempfile::TempDir` (全テストで使用) | ✅ **必須** - ファイルマーカー検証に利用 |
| Lua global access | `runtime.lua().globals()` | ✅ **必須** - 状態検証に利用 |
| Dynamic fixture generation | **未実装** | ⚠️ **新規** - テストコード内でLua生成 |

### 1.4 Conventions

**命名規則:**
- テスト関数: `test_<feature>_<scenario>`
- フィクスチャ: `crates/pasta_lua/tests/fixtures/loader/<name>/`
- フィクスチャ構成: `dic/`（Pastaファイル）+ `scripts/pasta/shiori/main.lua`

**テスト配置:**
- 統合テスト: `#[cfg(test)] mod tests` in `shiori.rs`
- フィクスチャ: `pasta_lua/tests/fixtures/loader/`（共有）

**依存関係:**
- `tempfile = "3"`（既存依存）
- `pasta_lua::mlua`（既存公開API）

---

## 2. Requirements Feasibility Analysis

### 2.1 Technical Needs

| 要件 | 必要な技術要素 | 現状の対応状況 |
|------|--------------|--------------|
| **Req 1**: SHIORI.load実行確認 | Luaグローバル変数設定・読み取り | ✅ `runtime.lua().globals()` |
| **Req 2**: SHIORI.request実行確認 | 動的応答生成、呼び出しカウント | ⚠️ 新規main.luaフィクスチャ必要 |
| **Req 3**: SHIORI.unload実行確認 | ファイルマーカー書き込み | ✅ `std::fs::write` + `TempDir` |
| **Req 4**: Pasta DSL読み込み確認 | シーン呼び出し、トランスパイル検証 | ⚠️ フィクスチャにPastaシーン追加 |
| **Req 5**: テストフィクスチャ整備 | 専用フィクスチャディレクトリ | ✅ パターン確立済み |

### 2.2 Identified Gaps

#### Gap 1: pasta_shiori専用フィクスチャ ⚠️ **Missing**
**現状:** `pasta_shiori`は`pasta_lua/tests/fixtures/loader/minimal/`に依存している
**必要:** `crates/pasta_shiori/tests/fixtures/shiori_lifecycle/`に専用の静的フィクスチャを配置

**実装方針:**
- `scripts/pasta/shiori/main.lua`: 観測可能な副作用を持つSHIORI関数を静的に定義
- `dic/test/lifecycle.pasta`: テスト用Pastaシーンを静的に定義
- `pasta.toml`: 設定ファイル
- フィクスチャは`TempDir`にコピーして使用（テスト分離）

#### Gap 2: Pasta DSL → Lua → SHIORI応答統合パス ⚠️ **Unknown**
**現状:** Pasta DSLがトランスパイルされても、SHIORI応答に含まれるかは未検証
**不明点:**
- トランスパイル済みシーンをLuaから呼び出す方法
- `@pasta_search`モジュールの使用方法
- シーン出力をSHIORI応答に埋め込む方法

**Research Needed**: `@pasta_search` APIとシーン呼び出しパターンの調査

#### Gap 3: ファイルマーカー検証のタイミング制御 ⚠️ **Constraint**
**現状:** `drop(shiori)`後にファイルチェックするが、`TempDir`も同時にドロップされる可能性
**制約:** `TempDir`のライフタイムを明示的に制御する必要がある

**対策:**
```rust
let marker_path = temp.path().join("unload_marker.txt");
drop(shiori);  // unload呼び出し
assert!(marker_path.exists());  // TempDirはスコープ終了まで有効
```

### 2.3 Complexity Signals

| カテゴリ | 複雑性 | 理由 |
|---------|-------|------|
| データモデル | **Simple** | テスト検証のみ、本番データモデル不要 |
| ビジネスロジック | **Simple** | 既存SHIORI実装は変更なし |
| ワークフロー | **Moderate** | 3フェーズ（load/request/unload）の検証シーケンス |
| 外部統合 | **Simple** | ファイルシステムのみ |

---

## 3. Implementation Approach Options

### Option A: Create Integration Test with Static Fixture ✅ **推奨（修正版）**

#### 新規作成ファイル
- **`crates/pasta_shiori/tests/shiori_lifecycle_test.rs`**:
  - インテグレーションテストとして独立配置
  - テスト関数: `test_shiori_lifecycle_lua_execution_verified()`
  - フィクスチャコピーヘルパー: `copy_fixture_to_temp()`を実装

- **`crates/pasta_shiori/tests/fixtures/shiori_lifecycle/`**:
  - `scripts/pasta/shiori/main.lua` - 観測可能な副作用を持つSHIORI関数（静的）
  - `dic/test/lifecycle.pasta` - テスト用Pastaシーン（静的）
  - `pasta.toml` - 設定ファイル（静的）

- **`crates/pasta_shiori/tests/common/mod.rs`** (オプション):
  - 共通ヘルパー関数を配置（複数テストで再利用する場合）

#### 互換性評価
- ✅ **既存テストへの影響なし**: インテグレーションテストとして独立
- ✅ **破壊的変更なし**: 既存コード不変
- ✅ **テストカバレッジ**: E2Eライフサイクル検証に特化

#### 複雑性と保守性
- **認知負荷**: Low - インテグレーションテストとして明確な責務分離
- **単一責任原則**: 維持 - ライフサイクルテストに特化
- **ファイルサイズ**: 新規ファイルのため既存コードへの影響なし

#### Trade-offs
- ✅ 静的フィクスチャで可読性・保守性向上
- ✅ インテグレーションテストとして適切な配置
- ✅ `pasta_shiori`専用フィクスチャで依存関係明確化
- ✅ `src/shiori.rs`の肥大化を回避
- ❌ 新規ファイル追加によるファイル数増加（minimal）

---

### Option B: Create New Test Fixture Directory

#### 新規作成要素
- **`crates/pasta_lua/tests/fixtures/loader/shiori_lifecycle/`**:
  - `dic/test/lifecycle.pasta` - Pastaシーン定義
  - `scripts/pasta/shiori/main.lua` - 観測可能な副作用を持つSHIORI関数
  - `pasta.toml` - 設定ファイル

#### 統合ポイント
- 既存の`copy_fixture_to_temp`で読み込み
- 新規テスト関数から参照

#### 責務境界
- **フィクスチャ責務**: 検証可能な振る舞い定義
- **テストコード責務**: 検証ロジック実装

#### Trade-offs
- ✅ 明確な関心の分離
- ✅ 静的フィクスチャで再利用可能
- ❌ 動的検証（例: hinst値確認）が困難
- ❌ フィクスチャのメンテナンスコスト増

---

### Option C: Hybrid Approach - 動的生成 + 静的フィクスチャ

#### 組み合わせ戦略
1. **静的フィクスチャ**: `dic/test/lifecycle.pasta`（Pasta DSL）
2. **動的生成**: `scripts/pasta/shiori/main.lua`（テストコード内で生成）

#### 段階的実装
- **Phase 1**: 動的main.lua生成による基本検証
- **Phase 2**: 必要に応じて静的フィクスチャ追加
- **Phase 3**: リファクタリング（動的 → 静的化）

#### リスク軽減
- テストコード内で全て完結（初期段階）
- 外部ファイル依存を最小化

#### Trade-offs
- ✅ 柔軟性が高い
- ✅ 段階的リファクタリング可能
- ❌ 初期実装が複雑
- ❌ 動的生成コードの保守性

---

## 4. Implementation Complexity & Risk

### 4.1 Effort Estimation

**Size: S (1-3 days)**

**根拠:**
- 既存テストパターン踏襲で学習コスト低
- 新規API不要、既存`runtime.lua()`を活用
- ファイル生成は標準ライブラリ（`std::fs`）のみ
- 検証ロジックは既存テストと同等の複雑性

**内訳:**
- フィクスチャ生成ヘルパー実装: 0.5日
- Lua状態検証テスト実装: 0.5日
- ファイルマーカー検証テスト実装: 0.5日
- Pasta DSL統合検証: 1日（`@pasta_search`調査含む）
- ドキュメント・レビュー: 0.5日

### 4.2 Risk Assessment

**Risk: Low**

**根拠:**
- **技術スタック**: Rust標準ライブラリ + 既存pasta_lua API
- **統合複雑性**: ファイルシステムのみ、外部サービスなし
- **パフォーマンス**: テストコードのみ、本番影響なし
- **セキュリティ**: TempDir分離、権限問題なし
- **アーキテクチャ影響**: テストコード追加のみ、本番コード不変

**潜在リスク:**
- `@pasta_search` APIの理解不足 → **Research Needed**
- ファイルマーカータイミング競合 → **明示的ライフタイム制御で対応**

---

## 5. Recommendations for Design Phase

### 5.1 Preferred Approach

**Option A: Create Integration Test with Static Fixture** を推奨

**理由:**
- インテグレーションテストとして適切な配置
- 静的フィクスチャで可読性・保守性向上
- `pasta_shiori`専用フィクスチャで依存関係明確化
- `src/shiori.rs`の肥大化を回避

### 5.2 Key Decisions

1. **静的フィクスチャを採用**
   - 理由: 可読性・保守性向上、Luaコード/Pastaコードの管理が容易
   - 配置: `crates/pasta_shiori/tests/fixtures/shiori_lifecycle/`

2. **インテグレーションテスト配置**
   - 理由: E2Eライフサイクルテストは統合テストに相応しい
   - 配置: `crates/pasta_shiori/tests/shiori_lifecycle_test.rs`

3. **Luaグローバル変数による状態検証**
   - 理由: `runtime.lua().globals()`で直接アクセス可能
   - 実装: `SHIORI.loaded_hinst`などのマーカー変数を読み取り

4. **ファイルマーカーによるunload検証**
   - 理由: Luaグローバル変数はドロップ後アクセス不可
   - 実装: `temp.path().join("unload_marker.txt")`の存在確認

### 5.3 Research Items

#### Research 1: `@pasta_search`モジュールAPI ⚠️ **High Priority**
**目的:** Pasta DSLシーンをLuaから呼び出す方法を特定

**調査内容:**
- `@pasta_search`モジュールのエクスポート関数
- シーン呼び出しの構文（例: `pasta_search.call_scene("メイン")`）
- 戻り値のフォーマット

**調査方法:**
- `crates/pasta_lua/src/search/mod.rs`を読む
- 既存統合テストの使用例を検索（`grep "@pasta_search"`）

**設計への影響:**
- Requirement 4（Pasta DSL読み込み確認）の実装方法を決定
- `main.lua`フィクスチャのシーン呼び出しコード設計

#### Research 2: Luaエラーハンドリングベストプラクティス ⚠️ **Medium Priority**
**目的:** SHIORI関数内エラーの適切なハンドリング方法

**調査内容:**
- `mlua::Error`の伝播パターン
- `pcall`ラッパーの必要性
- エラーメッセージのロギング方法

**設計への影響:**
- エラーケーステストの実装方法
- ロギング検証の設計

---

## Summary

本ギャップ分析により、以下が明確になった：

1. **既存資産の再利用性**: `copy_fixture_to_temp`、`runtime.lua()`など、既存テストインフラを活用可能
2. **実装ギャップ**: 動的main.lua生成機能、Pasta DSL統合パスの調査が必要
3. **推奨アプローチ**: 既存テストモジュール拡張（Option A）が最小コストで実現可能
4. **リスク**: Low - 既存パターン踏襲、外部依存なし
5. **工数**: S（1-3日） - 既存知識で実装可能

次フェーズでは、`@pasta_search` API調査を優先し、設計ドキュメントで具体的な実装戦略を定義する。
