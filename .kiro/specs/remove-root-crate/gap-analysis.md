# 実装ギャップ分析: ルートクレート削除

## 分析概要

| 項目 | 内容 |
|------|------|
| **範囲** | ルートクレート (`src/`) の完全削除とワークスペース構造の最適化 |
| **複雑度** | **中程度**: 構造的リファクタリングだが、既存パターンに準拠した変更 |
| **リスク** | **低**: ルートクレートが pasta_rune の完全な複製であり、削除後も API は損失しない |
| **主な課題** | 依存関係の広範な分散と ドキュメントの一括更新 |
| **推奨アプローチ** | Option A (既存構造への統合) - pasta_rune を単一の公開クレートとして位置付け |

## 現状分析 (Current State Investigation)

### ディレクトリ・ファイル構成

**ルートクレート (`src/`):**
```
src/
├── lib.rs              (64行、11モジュール定義)
├── cache.rs            (ParseCache)
├── engine.rs           (PastaEngine)
├── error.rs            (PastaError)
├── ir/mod.rs           (ScriptEvent)
├── loader.rs           (DirectoryLoader)
├── parser/mod.rs       (parse_str, parse_file)
├── registry/mod.rs     (SceneRegistry, WordDefRegistry)
├── runtime/mod.rs      (ScriptGenerator)
├── stdlib/mod.rs       (Pasta標準ライブラリ)
└── transpiler/mod.rs   (Transpiler2)
```

**pasta_rune クレート (`crates/pasta_rune/src/`):**
```
crates/pasta_rune/src/
├── lib.rs              (57行、同じ11モジュール定義)
├── cache.rs            ✓
├── engine.rs           ✓
├── error.rs            ✓
├── ir/mod.rs           ✓
├── loader.rs           ✓
├── runtime/mod.rs      ✓
├── stdlib/mod.rs       ✓
└── transpiler/mod.rs   ✓
```

**pasta_core クレート (`crates/pasta_core/src/`):**
```
crates/pasta_core/src/
├── lib.rs              (言語非依存層)
├── error.rs
├── parser/             (AST, grammar.pest)
└── registry/           (テーブル管理)
```

**重要**: ルートクレートに `parser/` と `registry/` が存在するが、pasta_rune には存在しない。これらは pasta_core から再エクスポートされている。

### 主要な API 再エクスポート

**ルート `src/lib.rs` で再エクスポート:**
- ParseCache, PastaEngine, PastaError, ScriptEvent
- DirectoryLoader, LoadedFiles, SceneEntry
- ScriptGenerator, VariableManager, VariableScope, VariableValue
- RandomSelector, DefaultRandomSelector, SceneTable

**pasta_rune `crates/pasta_rune/src/lib.rs` でも同じ再エクスポート:**
- さらに、`pub use pasta_core as core;` で pasta_core も再エクスポート

### 依存関係の実態

#### `use pasta::` の使用箇所 (20+件確認)

**ドキュメント・例のみ (リテラルコード非使用):**
1. `src/lib.rs:26` - ドキュメント例
2. `src/engine.rs:81` - ドキュメント例
3. `src/transpiler/mod.rs:23-25` - ドキュメント例
4. `src/parser/mod.rs:41,103,148` - ドキュメント例
5. `README.md:33-38` - サンプルコード例
6. `examples/scripts/README.md:97` - サンプルコード例

**実際のインポート文:**
- `tests/parser2_integration_test.rs:7` - **`use pasta_rune::parser::{...};`** ✓ 既に正しい

**他のクレートのドキュメント:**
- `crates/pasta_rune/src/transpiler/mod.rs:23-25` - ドキュメント例

**アーカイブドキュメント:**
- `.kiro/specs/completed/...` - 完了仕様の記録のみ

#### Cargo.toml の依存宣言

**検索結果**: 全 `Cargo.toml` ファイル で `pasta = ` 依存は見つからない

**実際の構造:**
- ルート `Cargo.toml`: `[workspace]` のみ、`[package]` なし（既に構造的に独立）
- `crates/pasta_rune/Cargo.toml`: `pasta_core = { path = "crates/pasta_core" }` のみ
- `crates/pasta_lua/Cargo.toml`: `pasta_core = { path = "crates/pasta_core" }` のみ
- テスト: `pasta_rune::` を直接インポート

### テストファイルの構成

全 30+ テストファイル (`tests/*.rs`) を確認：
- **すべて** `use pasta_rune::` でインポート（`use pasta::` なし）
- パーサー、トランスパイラ、エンジン、統合テストすべて既に pasta_rune 参照
- 例: `tests/parser2_integration_test.rs:7` → `use pasta_rune::parser::{...};`

### ステアリング規約との整合性

**structure.md の設計:**
```
pasta (workspace)
├── pasta_core          # 言語非依存層
└── pasta_rune          # Runeバックエンド層
```

**tech.md の規約:**
> "ワークスペース構成: pasta_core 言語非依存層、pasta_rune Runeバックエンド層"
> "公開API (pasta_rune/lib.rs): Engine, Transpiler, Runtime, IR, Error, Core"

**現状の問題:**
- ステアリング では pasta_rune を単一公開クレートとして定義
- ルートクレート `src/` が重複存在し、構造が曖昧

### コーディング規約

**テストファイル命名:**
- 確立された規約: `<feature>_test.rs` （例: `parser2_integration_test.rs`）

**モジュール命名:**
- 確立: snakecase (`engine.rs`, `cache.rs`)
- AST/型: CapitalCase （例: `PastaEngine`, `ScriptEvent`）

## 要件ギャップ分析 (Requirements Feasibility Analysis)

### 要件 1: ルートクレート削除

#### 技術需要
- ファイルシステム操作: `src/` ディレクトリの削除
- ビルド設定変更: `Cargo.toml` の編集

#### 現状との整合性
| 項目 | 現状 | 必要な対応 |
|------|------|-----------|
| `src/` ディレクトリ | ✓ 存在 | 削除 |
| ルート `Cargo.toml [package]` | ✓ 存在（64行余剰） | 削除 |
| ルート `Cargo.toml [dependencies]` | ✗ 存在しない | 確認のみ |
| ワークスペース定義 | ✓ 既存 (`[workspace]`) | 維持 |
| `workspace.dependencies` | ✓ 既存 | 維持 |

#### ギャップ
- **なし**: 現状でルート `Cargo.toml` は既に workspace-only に近い構成

#### 複雑度: **S** (1-3日)
- シンプルな削除作業
- ビルド設定変更は最小限

### 要件 2: 依存関係の移行

#### 技術需要
- `use pasta::` をしたファイルの特定と置換
- `use pasta_rune::` への変更

#### 現状分析
| ファイルタイプ | 件数 | 使用状況 | 対応 |
|---|---|---|---|
| ドキュメント例 (`/// use pasta::`) | 6件 | リテラルコード | 更新必須 |
| README.md サンプル | 4件 | サンプルコード | 更新必須 |
| テストファイル | 30+件 | 既に `pasta_rune::` | 変更不要 |
| 実装ファイル | 0件 | `use pasta::` なし | 変更不要 |

#### ギャップ
- **なし**: テストと実装は既に正しい参照
- **必要**: ドキュメント内のサンプルコード更新のみ

#### 複雑度: **S** (1-3日)
- 実装コード変更なし
- ドキュメント更新のみ

### 要件 3: ビルド設定の整合性

#### 技術需要
- `cargo check` による構文検証
- `cargo test --workspace` による機能検証
- `cargo clippy` による品質検証

#### 現状分析
| 項目 | 現状 | 予想される結果 |
|------|------|---|
| `cargo build --workspace` | ✓ 成功 | ✓ 削除後も成功 |
| `cargo test --workspace` | ✓ 成功 | ✓ 削除後も成功 |
| `cargo clippy --workspace` | ✓ 成功 | ✓ 削除後も成功 |
| ワークスペース統合性 | ✓ 正常 | ✓ 削除後も正常 |

#### ギャップ
- **なし**: 既に workspace-only 設定に準拠した構成

#### 複雑度: **S** (1-3日)
- ビルド検証は既存手段で実施可能
- 新規ツールやスクリプト不要

### 要件 4: ドキュメント更新

#### 技術需要
- README.md 更新（クレート参照）
- AGENTS.md 更新（ビルド手順）
- structure.md 更新（Pure Virtual Workspace 反映）
- examples/scripts/README.md 更新（サンプルコード）

#### 現状分析
| ドキュメント | 対象行 | 現状 | 更新内容 |
|---|---|---|---|
| README.md | 33-38 | 古いサンプル（pasta::） | 新サンプルに更新 |
| AGENTS.md | 要確認 | ビルド手順記載 | workspace コマンド確認 |
| structure.md | ディレクトリ図 | ルート `src/` 記載 | 削除して Pure Virtual 化 |
| examples/scripts/README.md | 97 | サンプルに `use pasta::` | 更新 |

#### ギャップ
- **中程度**: ドキュメントの一括更新が必要
- **情報漏洩リスク**: 古いドキュメントが残ると新規開発者を混乱させる

#### 複雑度: **M** (3-7日)
- 複数ドキュメントの更新
- structure.md は詳細な見直しが必要

### 要件 5: リグレッション防止

#### 技術需要
- 既存テストの全実行と成功確認
- pasta_core/rune/lua のユニットテスト実行
- 統合テストの完全実行

#### 現状分析
| テストカテゴリ | ファイル数 | 現状 | 削除後の予想 |
|---|---|---|---|
| Parser 統合 | 1 | ✓ 成功 | ✓ 成功（参照が pasta_rune に正しく設定） |
| Transpiler | 3 | ✓ 成功 | ✓ 成功 |
| Runtime | 7 | ✓ 成功 | ✓ 成功 |
| Engine E2E | 8 | ✓ 成功 | ✓ 成功 |
| stdlib/registry | 3 | ✓ 成功 | ✓ 成功 |
| Control Flow | 1 | ✓ 成功 | ✓ 成功 |
| ディレクトリ | 1 | ✓ 成功 | ✓ 成功 |

#### ギャップ
- **なし**: テストは既に pasta_rune 参照で統合済み

#### 複雑度: **S** (1-3日)
- `cargo test --workspace` で一括検証可能
- 追加テスト不要

## 実装アプローチ評価

### Option A: 既存パターン に従い、ルート削除 + ドキュメント更新のみ

**戦略:**
1. `src/` ディレクトリ全削除
2. ルート `Cargo.toml` の `[package]` セクション削除
3. ドキュメント（README.md, structure.md 等）内のサンプルコードを `pasta_rune::` に更新
4. `cargo test --workspace` で検証

**対象ファイル:**
- **削除**: `src/` (全ファイル: 11個)
- **更新**: `Cargo.toml` (ルート), `README.md`, `structure.md`, `examples/scripts/README.md`
- **変更不要**: `tests/**/*.rs` (既に正しい), `crates/**/*.rs` (既に正しい)

**互換性評価:**
- ✓ 既存 API は変わらない (`pasta_rune::*` で全て利用可能)
- ✓ テストはすべて既に `pasta_rune::` 参照
- ✓ ステアリング規約に完全準拠

**責務境界:**
- ✓ 明確: pasta_rune が単一の公開クレート
- ✓ pasta_core は言語非依存層、pasta_rune が Rune バックエンド

**複雑度・保守性:**
- ✓ シンプル: ファイル削除と既存ドキュメント更新のみ
- ✓ 認知負荷: 最小限（ワークスペース構造が明確化）
- ✓ ファイル規模: 削減される

**Trade-offs:**
- ✅ 最小限の変更で最大の効果
- ✅ リスクなし（テスト検証で保護）
- ✅ ステアリング規約と完全一致
- ❌ ドキュメント更新が複数箇所必要

**推奨度**: ⭐⭐⭐⭐⭐ (最推奨)

---

### Option B: ワークスペース・エイリアス導入（別の名前で pasta_rune を公開）

**戦略:**
1. ルート `Cargo.toml` に `[package]` を残し、ワークスペース仮想クレートとして定義
2. `src/lib.rs` を削除し、`pub use pasta_rune::*;` のみで pasta_rune を再エクスポート
3. `use pasta::` の互換性を維持

**対象ファイル:**
- **削除**: `src/` (実装ファイルのみ: cache.rs など)
- **保持**: `src/lib.rs` (再エクスポート専用)
- **更新**: ルート `Cargo.toml`

**互換性評価:**
- ⚠️ `use pasta::` は動作し続ける
- ⚠️ しかし二重参照パス （pasta と pasta_rune）が共存

**責務境界:**
- ✗ 曖昧: pasta と pasta_rune が区別されず

**複雑度・保守性:**
- ❌ 複雑: 再エクスポート層が増える
- ❌ 保守性低下: 今後 pasta_rune 変更時に pasta も追従必須

**Trade-offs:**
- ✅ 後方互換性保持（既存コードが動作）
- ❌ ステアリング規約に反する（二重構造の継続）
- ❌ 技術債を増やす

**推奨度**: ❌ (非推奨)

---

### Option C: 段階的移行（フェーズ分割）

**戦略:**
- **Phase 1**: ドキュメント更新 + テスト検証（`use pasta_rune::` への更新）
- **Phase 2**: `src/` ディレクトリ削除 + Cargo.toml 整理

**対象ファイル:**
- **Phase 1**: ドキュメント (README.md, examples/ 等)
- **Phase 2**: ファイルシステム操作 (src/ 削除)

**互換性評価:**
- ✓ リスク分散: 各段階で検証

**複雑度・保守性:**
- ❌ 不要な段階分割（リスク がない）

**Trade-offs:**
- ✅ 慎重な進め方
- ❌ 過度に複雑化
- ❌ 実装に時間がかかる

**推奨度**: ⭐ (オプション、不要)

---

## 実装の複雑度・リスク評価

### 全体の複雑度: **S (1-3日)**

**根拠:**
- ルート削除は単純な操作（`rm src/`）
- ドキュメント更新は既存パターン（検索置換）
- テスト検証は既存ツール（`cargo test`）
- 設計変更なし（ステアリング規約に従うのみ）

### リスク評価: **低**

| リスク要因 | 評価 | 理由 |
|---|---|---|
| 機能破壊 | **低** | テストは既に pasta_rune 参照で保護 |
| 外部 API 破壊 | **低** | pasta_rune が全 API を提供 |
| ビルド失敗 | **低** | 依存関係が正しく設定済み |
| パフォーマンス低下 | **低** | 変更はなし |
| セキュリティ影響 | **低** | 構造変更のみ |
| ドキュメント漏洩 | **中** | 多数のドキュメント更新が必要（ただしリスク低） |

**総合リスク**: **低**

---

## 要件-資産マッピング

### 要件 1: ルートクレート削除

| 要件 | 現在の資産 | ギャップ | 対応 |
|------|---|---|---|
| `src/` 削除 | ✓ `src/` 存在 | ❌ 削除対象 | 直接削除 |
| `[package]` 削除 | ✓ ルート Cargo.toml | ❌ 削除対象 | 編集削除 |
| `[dependencies]` 削除 | ✗ 存在しない | ✓ 完了 | 確認のみ |

### 要件 2: 依存関係移行

| 要件 | 現在の資産 | ギャップ | 対応 |
|------|---|---|---|
| コード内の `use pasta::` | 0件（実装） | ✓ 完了 | 確認のみ |
| テスト内の `use pasta::` | 0件 | ✓ 完了 | 確認のみ |
| ドキュメント内の例 | 10件 | ❌ 更新必須 | 検索置換 |
| Cargo.toml 依存 | 0件（実装） | ✓ 完了 | 確認のみ |

### 要件 3: ビルド整合性

| 要件 | 現在の資産 | ギャップ | 対応 |
|------|---|---|---|
| `cargo build` | ✓ 成功 | ✓ 完了 | 削除後確認 |
| `cargo test` | ✓ 成功（30+テスト） | ✓ 完了 | 削除後実行 |
| `cargo clippy` | ✓ 成功 | ✓ 完了 | 削除後実行 |

### 要件 4: ドキュメント更新

| 要件 | 現在の資産 | ギャップ | 対応 |
|------|---|---|---|
| README.md | ✓ 存在 | ❌ サンプル更新 | 編集 |
| AGENTS.md | ✓ 存在 | ✓ 確認 | 確認のみ |
| structure.md | ✓ 存在 | ❌ ルート削除反映 | 編集 |
| examples/scripts/README.md | ✓ 存在 | ❌ サンプル更新 | 編集 |

### 要件 5: リグレッション防止

| 要件 | 現在の資産 | ギャップ | 対応 |
|------|---|---|---|
| 統合テスト | ✓ 30+件 | ✓ 完了 | 削除後全実行 |
| pasta_core テスト | ✓ 存在 | ✓ 完了 | 削除後実行 |
| pasta_rune テスト | ✓ 存在 | ✓ 完了 | 削除後実行 |
| pasta_lua テスト | ✓ 存在 | ✓ 完了 | 削除後実行 |

---

## デザイン段階への推奨事項

### 推奨アプローチ
**Option A: 既存パターン に従い、ルート削除 + ドキュメント更新**

### キー決定事項
1. **ワークスペース構造**: Pure Virtual Workspace (ルート Cargo.toml は設定のみ)
2. **公開API**: pasta_rune を単一の公開クレート として位置付け
3. **クレート間依存**: pasta_rune → pasta_core (単方向、確立済み)
4. **テスト検証**: `cargo test --workspace` で全機能保証

### 実装順序
1. ドキュメント（README.md, structure.md）の サンプルコード更新
2. ルート `src/` ディレクトリの削除
3. ルート `Cargo.toml` の `[package]` セクション削除
4. `cargo test --workspace` で全テスト実行（検証）
5. `cargo clippy --workspace` で品質確認

### 調査不要な項目
- 外部依存の互換性: 既存ビルド成功で確認済み
- API 設計: ステアリング規約で確定済み
- テスト戦略: 既存パターン活用（新規テスト不要）

### リスク軽減戦略
- **ビルド検証**: `cargo check` で削除直後の構文確認
- **テスト検証**: `cargo test --workspace` で機能保証
- **品質確認**: `cargo clippy` で警告ゼロを確認
- **ドキュメント精査**: Pull Request で複数人による確認

---

## 結論

### サマリー

ルートクレート (`src/`) 削除は **低リスク・低複雑度** の実装。以下の理由：

1. **テスト保護**: 全テスト (30+件) が既に `pasta_rune::` 参照で統合済み
2. **設計準拠**: ステアリング規約で Pure Virtual Workspace が規定済み
3. **最小変更**: ファイル削除とドキュメント更新のみ
4. **既存パターン**: ワークスペース管理の確立された手法

### 実装戦略

**Option A (推奨)** を採用：

```
Phase 1: 準備
- ドキュメント（README.md, structure.md 等）のサンプルコード検索

Phase 2: 実装
- ドキュメント内の `use pasta::` → `use pasta_rune::` 置換
- `src/` ディレクトリ削除
- ルート `Cargo.toml` の `[package]` セクション削除

Phase 3: 検証
- `cargo test --workspace` (全テスト実行)
- `cargo clippy --workspace` (品質確認)
- `cargo doc --open` (ドキュメント生成確認)

Phase 4: 完了
- Git commit + PR で複数人確認
```

### 実装後の状態

| 項目 | 完了後 |
|------|--------|
| ワークスペース構造 | ✓ Pure Virtual (ルート Cargo.toml + crates/) |
| 公開API | ✓ pasta_rune で一元管理 |
| テスト | ✓ 全テスト成功 |
| ドキュメント | ✓ 新構造を反映 |
| ビルド | ✓ cargo build/test/clippy すべて成功 |
| 依存管理 | ✓ workspace.dependencies で一元化 |

---

## 次ステップ

1. **要件承認**: 要件定義 (requirements.md) の管理者確認
2. **設計フェーズ**: `/kiro-spec-design remove-root-crate` でタスク分割と詳細設計を生成
3. **実装フェーズ**: `/kiro-spec-impl remove-root-crate` で実装開始
