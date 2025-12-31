# 要件定義: ルートクレート削除

## 導入

### 背景
pasta ワークスペースは現在、ルートディレクトリに `src/` を持つルートクレートと、`crates/` 配下のサブクレート（pasta_core, pasta_rune, pasta_lua）の二重構造になっている。ステアリング文書（structure.md, tech.md）によれば、本来の設計はワークスペースルートをビルド設定のみとし、すべての実装を `crates/` 配下に配置する方針である。

### 現状分析
- **ルートクレート** (`src/lib.rs`): pasta_rune と完全に重複したモジュール構造を持つ
  - 公開モジュール: cache, engine, error, ir, parser, registry, runtime, stdlib, transpiler
  - 再エクスポート: ParseCache, PastaEngine, PastaError, ScriptEvent 等
- **pasta_core**: 言語非依存層（パーサー、レジストリ）
- **pasta_rune**: Rune バックエンド層（pasta_core 依存）
- **pasta_lua**: Lua バックエンド層（pasta_core 依存）

### 問題点
1. **機能重複**: ルートクレートと pasta_rune が同一のモジュール/API を提供
2. **保守コスト**: 変更時に両方のクレートを更新する必要がある
3. **設計不整合**: ステアリング規約に反したワークスペース構成
4. **依存性混乱**: `use pasta::` と `use pasta_rune::` の使い分けが不明確

### 目標
ルートクレートを削除し、ワークスペース構成をステアリング規約に準拠させる。すべての機能実装を `crates/` 配下のクレートに統合し、ルートは Cargo.toml のみでワークスペース定義を行う Pure Virtual Workspace とする。

## スコープ

### 対象範囲
- ✅ ルート `src/` ディレクトリの削除
- ✅ ルート `Cargo.toml` からの `[package]` セクション削除
- ✅ 既存コードの `use pasta::` 参照を適切なクレート参照に変更
- ✅ テストファイルの依存関係更新
- ✅ ドキュメント（README.md, AGENTS.md 等）の更新

### 対象外
- ❌ pasta_core, pasta_rune, pasta_lua の内部構造変更
- ❌ 新機能の追加
- ❌ パフォーマンス最適化

## 要件

### 要件 1: ルートクレート削除
**目的**: ワークスペースルートから実装コードを削除し、Pure Virtual Workspace 構成にする

#### 受入基準
1. When ルートディレクトリに `src/` が存在する場合、The Refactoring Process shall ディレクトリ全体を削除する
2. When ルート `Cargo.toml` に `[package]` セクションが存在する場合、The Refactoring Process shall セクションを削除し `[workspace]` のみを残す
3. When ルート `Cargo.toml` に `[dependencies]` セクションが存在する場合、The Refactoring Process shall セクションを削除する
4. The Workspace Structure shall ルートに Cargo.toml, README.md, LICENSE, .kiro/, .vscode/, .github/ のみを保持する
5. The Workspace Structure shall すべての Rust コードを `crates/*/src/` 配下に配置する

### 要件 2: 依存関係の移行
**目的**: ルートクレートへの参照を適切なサブクレート参照に置き換える

#### 受入基準
1. When ソースコードに `use pasta::` が含まれる場合、The Refactoring Process shall `use pasta_rune::` または `use pasta_core::` に置き換える
2. When テストコードに `pasta =` 依存が含まれる場合、The Refactoring Process shall `pasta_rune` または `pasta_core` 依存に置き換える
3. When ドキュメントに `pasta` クレート参照が含まれる場合、The Refactoring Process shall 適切なサブクレート名に更新する
4. The Migration Process shall すべての `use` 文の妥当性を `cargo check` で検証する
5. The Migration Process shall すべてのテストが `cargo test --workspace` で成功することを確認する

### 要件 3: ビルド設定の整合性
**目的**: ワークスペース全体のビルドが正常に機能することを保証する

#### 受入基準
1. When `cargo build --workspace` を実行した場合、The Workspace shall エラーなくビルドが完了する
2. When `cargo test --workspace` を実行した場合、The Workspace shall すべてのテストが成功する
3. When `cargo clippy --workspace` を実行した場合、The Workspace shall 警告・エラーがゼロである
4. The Workspace Configuration shall `workspace.dependencies` でサブクレートの共通依存を管理する
5. The Workspace Configuration shall `workspace.package` でメタデータ（edition, authors, license）を共有する

### 要件 4: ドキュメント更新
**目的**: プロジェクトドキュメントを新しいワークスペース構造に合わせて更新する

#### 受入基準
1. When README.md にクレート参照が含まれる場合、The Documentation shall pasta_core, pasta_rune, pasta_lua の正しい構成を説明する
2. When AGENTS.md にビルド手順が含まれる場合、The Documentation shall ワークスペースビルドコマンドを更新する
3. When ステアリング文書（structure.md）にルートクレート記載がある場合、The Documentation shall Pure Virtual Workspace 構成を反映する
4. The Documentation shall サブクレート間の依存関係図を明示する
5. The Documentation shall 各クレートの責務と公開 API を明確に記述する

### 要件 5: リグレッション防止
**目的**: 既存機能が破壊されないことを保証する

#### 受入基準
1. When リファクタリング完了後、The Test Suite shall すべての既存統合テストが成功する
2. When リファクタリング完了後、The Test Suite shall pasta_core の単体テストが成功する
3. When リファクタリング完了後、The Test Suite shall pasta_rune の単体テストが成功する
4. When リファクタリング完了後、The Test Suite shall pasta_lua の単体テストが成功する
5. The Refactoring Process shall 機能追加・変更を含まず、構造変更のみを行う

## 影響分析

### 変更対象ファイル（推定）
- **削除**: `src/**/*.rs` (全ファイル)
- **更新**: `Cargo.toml` (ルート)
- **更新**: `tests/**/*.rs` (use 文変更)
- **更新**: `crates/*/src/**/*.rs` (use pasta:: → use pasta_rune::)
- **更新**: `README.md`, `AGENTS.md`, `.kiro/steering/structure.md`

### 依存関係
- **pasta_core**: 変更なし（言語非依存層）
- **pasta_rune**: `pasta::` 参照を内部で `pasta_rune::` に変更
- **pasta_lua**: 影響なし（pasta 参照なし）
- **外部依存**: 変更なし

### リスク
- **低リスク**: ルートクレートは pasta_rune の完全な重複であり、削除後も機能は pasta_rune で提供される
- **中リスク**: `use pasta::` の一括置換で見落としがある可能性（cargo check で検出可能）
- **低リスク**: テストの依存関係更新漏れ（cargo test で検出可能）

## 成功基準

| 項目 | 基準 |
|------|------|
| **ビルド** | `cargo build --workspace` がエラーなく完了 |
| **テスト** | `cargo test --workspace` で全テスト成功 |
| **Lint** | `cargo clippy --workspace -- -D warnings` が警告ゼロ |
| **構造** | `src/` ディレクトリが存在しない |
| **ドキュメント** | README.md がワークスペース構成を正確に記述 |
| **リグレッション** | すべての既存機能が動作する |

## 非機能要件

### 保守性
- The Workspace Structure shall サブクレート間の責務を明確に分離する
- The Documentation shall 新規開発者がクレート構成を理解できるよう記述する

### 拡張性
- The Workspace Design shall 将来的な新バックエンド追加（例: pasta_python）を想定した構造とする

### 互換性
- The Public API shall pasta_rune の公開 API を変更しない（破壊的変更なし）

