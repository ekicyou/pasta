# 調査ログ: ルートクレート削除

## 調査概要

| 項目 | 内容 |
|------|------|
| **実施タイプ** | 軽量ディスカバリー（既存システムの拡張分析） |
| **調査期間** | Gap Analysis フェーズ |
| **調査範囲** | ワークスペース構造、依存関係、ビルド設定 |
| **言語** | ja |

## 主要な発見

### 1. ワークスペース構造の分析

**発見**: ルートクレート (`src/`) が pasta_rune と完全に重複している

- **ルート `src/lib.rs`**: 64行、11モジュール定義
- **pasta_rune `src/lib.rs`**: 57行、同じ11モジュール定義
- **対応ファイル**: cache.rs, engine.rs, error.rs, ir/mod.rs, loader.rs, runtime/mod.rs, stdlib/mod.rs, transpiler/mod.rs（9個）
- **唯一の差異**: ルートに `parser/` と `registry/` が存在（ただしこれは pasta_core から再エクスポート）

**含意**:
- ルートクレートは pasta_rune の単なる複製
- 削除後、API は pasta_rune で継続利用可能
- 二重メンテナンスの必要性なし

### 2. 依存関係の確認

**発見**: 実装コードは既に正しい参照構造

| 依存タイプ | 件数 | 対象 | 対応 |
|---|---|---|---|
| テストの `use pasta_rune::` | 30+件 | `tests/**/*.rs` | 変更不要 |
| ドキュメント例の `use pasta::` | 10件 | README.md, 例 | 更新必須 |
| 実装の `use pasta::` | 0件 | `src/**/*.rs` | 確認のみ |

**含意**:
- テスト全体が既に pasta_rune 参照で保護されている
- ドキュメント更新のみが必要な作業

### 3. ビルド設定の現状

**発見**: ルート `Cargo.toml` は既に workspace-only 構成に近い

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
edition = "2024"
authors = [...]
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# 共通依存管理
```

**含意**:
- `[package]` セクションは実質的に不使用
- 実装は `crates/` 配下に完全に移行済み
- 削除は形式的な整理のみ

### 4. テストカバレッジの確認

**発見**: 全テストが既に `pasta_rune::` でインポート

- `parser2_integration_test.rs`: `use pasta_rune::parser::{...};`
- `pasta_transpiler2_*.rs`: `use pasta_rune::*`
- `pasta_engine_*.rs`: `use pasta_rune::*`
- 統合テスト群: すべて `pasta_rune::` 参照

**含意**:
- テスト層はルートクレート削除で影響ゼロ
- リグレッションリスク極低

### 5. ドキュメント参照の調査

**発見**: ドキュメント内に `use pasta::` サンプルコードが散在

**対象ファイル**:
1. `src/lib.rs:26` - ドキュメント例
2. `src/engine.rs:81` - ドキュメント例
3. `src/transpiler/mod.rs:23-25` - ドキュメント例
4. `src/parser/mod.rs:41,103,148` - ドキュメント例
5. `README.md:33-38` - サンプルコード（4件）
6. `examples/scripts/README.md:97` - サンプルコード

**含意**:
- 削除前のドキュメント更新が必須
- または削除後、ドキュメント削除も必要

## アーキテクチャパターン評価

### 選択パターン: Pure Virtual Workspace

**パターン定義**:
```
Workspace Root
├── Cargo.toml (設定のみ、[package]なし)
├── crates/
│   ├── pasta_core    # 言語非依存層
│   ├── pasta_rune    # Rune バックエンド（公開クレート）
│   └── pasta_lua     # Lua バックエンド
└── tests/            # ワークスペースレベルテスト
```

**選択理由**:
1. **設計準拠**: ステアリング規約で規定済み
2. **単一化**: pasta_rune を唯一の公開クレートとして位置付け
3. **保守性**: 重複排除でメンテナンスコスト削減
4. **拡張性**: 新バックエンド追加時も構造が明確

### 既存パターンとの整合性

**既存ワークスペース構成** (`structure.md`):
```
pasta (workspace)
├── pasta_core          # 言語非依存層
└── pasta_rune          # Runeバックエンド層
```

**提案する構成**:
```
pasta (workspace) ← Pure Virtual
├── crates/
│   ├── pasta_core      # 言語非依存層
│   ├── pasta_rune      # Rune バックエンド（公開API）
│   └── pasta_lua       # Lua バックエンド
└── tests/              # 統合テスト
```

**差異**: ルートクレート (`src/`) の削除のみ、その他変更なし

## 設計決定とリスク

### 決定 1: ルートクレート完全削除

**リスク評価**: **低**

**理由**:
- テスト全体 (30+件) が既に `pasta_rune::` 参照
- API は pasta_rune で継続提供
- 機能の喪失ゼロ

**緩和策**:
- `cargo check` で削除直後の構文確認
- `cargo test --workspace` で全機能保証
- `cargo clippy` で品質確認

### 決定 2: ドキュメント内のサンプルコード更新

**リスク評価**: **低**

**更新対象**:
- README.md の例 (4件)
- examples/scripts/README.md (1件)
- src/lib.rs のドキュメント例 (削除時に自動的に消滅)

**更新内容**: `use pasta::` → `use pasta_rune::`

### 決定 3: ワークスペース設定の簡潔化

**リスク評価**: **低**

**変更内容**:
- ルート `Cargo.toml` の `[package]` セクション削除
- `[workspace]` と `[workspace.dependencies]` は保持

**含意**: IDE, CI/CD, ドキュメント生成ツール に対する影響なし

## 統合リスク評価

| 項目 | リスク | 理由 | 緩和策 |
|------|--------|------|--------|
| **API 破壊** | 低 | pasta_rune が全 API 提供 | テスト検証 |
| **ビルド失敗** | 低 | 依存関係は既に正確 | cargo check |
| **テスト失敗** | 低 | テストは既に pasta_rune 参照 | cargo test |
| **ドキュメント漏洩** | 中 | サンプル更新が複数箇所 | 検索置換 + PR確認 |
| **保守性低下** | 低 | 重複排除でむしろ改善 | structure.md 更新 |

**総合リスク**: **低～中** （ドキュメント更新の完全性のみ）

## 検証ストラテジー

### Phase 1: ドキュメント更新前の確認

```sh
# 削除前にサンプルコード対象を特定
grep -r "use pasta::" README.md examples/
```

### Phase 2: 削除と更新の実行

```sh
# 1. ドキュメント更新
# README.md, examples/scripts/README.md, structure.md 更新

# 2. ルート削除
rm -rf src/

# 3. Cargo.toml 編集
# [package] セクション削除
```

### Phase 3: ビルド検証

```sh
cargo check --workspace         # 構文確認
cargo build --workspace         # ビルド確認
cargo test --workspace          # 機能確認
cargo clippy --workspace        # 品質確認
cargo doc --open                # ドキュメント生成確認
```

## 技術スタック影響評価

| 技術 | 現状 | 影響 | 対応 |
|------|------|------|------|
| **Cargo** | Workspace 管理 | 削除で最適化 | 変更なし |
| **Rust** | 2024 edition | 削除で影響なし | 変更なし |
| **Pest** | パーサー生成 | 影響なし（pasta_core） | 変更なし |
| **Rune** | バックエンド VM | 影響なし（pasta_rune） | 変更なし |
| **外部依存** | workspace.dependencies | 管理継続 | 変更なし |

## 結論

**ルートクレート削除は、既存システムの単純な整理作業**

- ✓ テストカバレッジで保護済み
- ✓ 依存関係は既に正しく設定
- ✓ ビルド設定は workspace-only に準拠
- ✗ ドキュメント更新が複数箇所必要（リスク：低）

**推奨**: Option A に従い、軽量実装で完了可能

