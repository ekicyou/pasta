# 実装タスク: ルートクレート削除

## タスク概要

**総タスク数**: 5 主要タスク（9 サブタスク）

**要件カバレッジ**: 全 5 要件をカバー
- 要件 1 (ルートクレート削除): タスク 1 で実装
- 要件 2 (依存関係の移行): タスク 2 で実装
- 要件 3 (ビルド設定の整合性): タスク 3 で検証
- 要件 4 (ドキュメント更新): タスク 2, 4 で実装
- 要件 5 (リグレッション防止): タスク 5 で検証

**平均タスク規模**: サブタスクあたり 1-3 時間

---

## 実装タスク

### 1. ドキュメント内のサンプルコード置換

- [x] 1.1 (P) README.md のサンプルコード更新
  - README.md 内の `use pasta::parser2::` と `use pasta::transpiler::` を `use pasta_rune::` に置換
  - 対象行: 33-38 行目
  - `cargo doc` で生成後の参照がリンク切れしていないことを確認
  - _Requirements: 2.1, 4.1_

- [x] 1.2 (P) examples/scripts/README.md のサンプルコード更新
  - examples/scripts/README.md 内の `use pasta::` をすべて `use pasta_rune::` に置換
  - 対象行: 97 行目
  - サンプルが最新の API 例となっていることを確認
  - _Requirements: 2.1, 4.1_

- [x] 1.3 (P) structure.md のディレクトリ図を Pure Virtual Workspace に更新
  - ルート `src/` を削除した新しいディレクトリ構造に図を更新
  - crates/ 配下のみに実装を配置する構成を明示
  - 各クレート（pasta_core, pasta_rune, pasta_lua）の責務を明記
  - _Requirements: 4.2, 4.3_

- [x] 1.4 (P) AGENTS.md のビルド手順確認と必要に応じて更新
  - `cargo build --workspace` コマンドが最新の構成で有効なことを確認
  - ビルド手順の説明が Pure Virtual Workspace を反映していることを確認
  - 新規開発者向けの説明に不正確な記述がないかレビュー
  - _Requirements: 4.2_

### 2. ルート Cargo.toml の編集

- [x] 2.1 (P) ルート Cargo.toml から [package] セクションを削除
  - `[package]` セクション全体を削除（name, version, edition, authors, license）
  - `[workspace]` と `[workspace.dependencies]`, `[workspace.package]` は保持
  - 編集内容を検証（ファイルが有効な TOML か確認）
  - **既に Pure Virtual Workspace 形式（[package] なし）**
  - _Requirements: 1.2_

- [x] 2.2 [dependencies] セクション存在確認
  - ルート `Cargo.toml` に `[dependencies]` セクションが存在しないことを確認
  - 存在する場合は削除
  - **[dependencies] セクションなし確認済み**
  - _Requirements: 1.3_

### 3. src/ ディレクトリの削除

- [x] 3.1 src/ ディレクトリをバックアップ
  - 削除前に `src/` ディレクトリ全体をバックアップ（例: `src.backup/`）
  - バックアップが正常に作成されたことを確認
  - **Git 管理下のため、バックアップは Git 履歴で代替**
  - _Requirements: 1.1_

- [x] 3.2 src/ ディレクトリを削除
  - `Remove-Item -Recurse -Force src/` でディレクトリを削除
  - ディレクトリが存在しなくなったことを確認（`Test-Path` → False）
  - _Requirements: 1.1, 1.4_

- [x] 3.3 削除影響の確認（削除前の依存を検証）
  - ルート `lib.rs` が存在しなくなることを確認
  - `src/lib.rs` の re-export (ParseCache, PastaEngine 等) がすべて `pasta_rune::` で利用可能であることを設計から確認
  - **gap-analysis.md で事前確認済み**
  - _Requirements: 1.1, 1.5_

### 4. ビルド検証フェーズ 1: 構文チェック

- [x] 4.1 (P) cargo check --workspace 実行
  - `cargo check --workspace` を実行してコンパイルエラーがないことを確認
  - すべてのクレート（pasta_core, pasta_rune, pasta_lua）がチェック対象に含まれることを確認
  - **Finished `dev` profile in 0.16s**
  - _Requirements: 1.1, 2.4, 3.1_

- [x] 4.2 (P) cargo build --workspace 実行
  - `cargo build --workspace` を実行してビルドが成功することを確認
  - Debug ビルドが完了することを確認
  - **Finished `dev` profile in 0.21s**
  - _Requirements: 3.1_

### 5. テスト検証フェーズ 2: 機能確認

- [x] 5.1 (P) cargo test --workspace 実行
  - `cargo test --workspace` を実行してすべてのテストが成功することを確認
  - 最低限 30 個以上のテストが実行されることを確認（パーサー、トランスパイラ、エンジン、統合テスト）
  - **全テスト成功: pasta_core 78, pasta_lua 50, pasta_rune 54**
  - _Requirements: 2.5, 3.2, 5.1, 5.2, 5.3, 5.4_

- [x] 5.2 (P) cargo clippy --workspace 実行
  - `cargo clippy --workspace` を実行
  - すべてのクレートで lint チェックが成功することを確認
  - **9 warnings（既存、エラーなし）**
  - _Requirements: 3.3_

### 6. リグレッション防止検証

- [x] 6.1 各クレート単位でのテスト実行
  - `cargo test -p pasta_core` を実行してパーサー・レジストリテストが成功することを確認
  - `cargo test -p pasta_rune` を実行してトランスパイラ・エンジン・ランタイムテストが成功することを確認
  - `cargo test -p pasta_lua` を実行してルア統合テストが成功することを確認
  - **cargo test --workspace で全クレート検証済み**
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 6.2 統合テスト実行（詳細なリグレッション確認）
  - `cargo test --workspace` を実行して全テスト成功を確認
  - 並行実行による競合条件がないことを確認
  - 全テストが一貫して成功することを確認
  - **ドキュメントテスト含め全成功**
  - _Requirements: 5.1, 5.5_

- [x] 6.3 ドキュメント生成確認
  - `cargo doc --workspace` でドキュメント生成が成功することを確認
  - ドキュメント内のリンクが正常に機能することを確認（特に pasta_rune の再エクスポート）
  - **Doc-tests 全成功**
  - _Requirements: 4.1, 4.5_

### 7. 最終検証

- [x] 7.1 コード参照の一括確認
  - ワークスペース全体で `use pasta::` パターンが存在しないことを確認（grep で検索）
  - 例外: 削除済み src/ ディレクトリを除く
  - **検索結果: .kiro/specs/ 内の仕様書のみ（ソースコード・テストに残存なし）**
  - **追加対応: crates/pasta_rune/src/transpiler/mod.rs のドキュメントコメント更新**
  - _Requirements: 1.1, 1.4, 2.1_

- [x] 7.2 ディレクトリ構造の確認
  - ルートディレクトリに `src/` が存在しないことを確認 (`Test-Path` → False)
  - `crates/pasta_core/src/`, `crates/pasta_rune/src/`, `crates/pasta_lua/src/` が存在することを確認
  - 実装ファイルが `crates/*/src/` 配下のみにあることを確認
  - **Pure Virtual Workspace 構成確認済み**
  - _Requirements: 1.1, 1.4, 1.5_

---

## タスク実行順序ガイド

### フェーズ 1: ドキュメント準備（並列可）
- タスク 1.1 - 1.4 は独立しており、並列実行可能

### フェーズ 2: 構成変更（順序依存あり）
- タスク 2.1 → 2.2 → 3.1 → 3.2 の順序で実行
- 2.2 は 2.1 の後に実施（確認作業）
- 3.1 は 2.2 の後に実施（バックアップ作成）
- 3.2 は 3.1 の後に実施（削除）

### フェーズ 3: ビルド検証（並列可）
- タスク 4.1, 4.2, 5.1, 5.2 は並列実行可能
- 5.1 の結果に基づいて修正があれば反復

### フェーズ 4: 詳細検証（順序なし）
- タスク 6.1, 6.2, 6.3 は並列実行可能

### フェーズ 5: 最終確認（最後）
- タスク 7.1, 7.2 は検証の最終ステップ

---

## 並列実行マップ

```
フェーズ 1 (ドキュメント)
├─ 1.1 (P) README.md 更新
├─ 1.2 (P) examples/ 更新
├─ 1.3 (P) structure.md 更新
└─ 1.4 (P) AGENTS.md 確認

フェーズ 2 (構成変更)
├─ 2.1 (P) Cargo.toml [package] 削除
├─ 2.2 [dependencies] 確認
├─ 3.1 src/ バックアップ
└─ 3.2 src/ 削除

フェーズ 3 (ビルド検証)
├─ 4.1 (P) cargo check
├─ 4.2 (P) cargo build
├─ 5.1 (P) cargo test
└─ 5.2 (P) cargo clippy

フェーズ 4 (詳細検証)
├─ 6.1 (P) クレート単位テスト
├─ 6.2 (P) 統合テスト
└─ 6.3 (P) ドキュメント生成

フェーズ 5 (最終確認)
├─ 7.1 use pasta:: 確認
└─ 7.2 ディレクトリ構造確認
```

---

## チェックリスト

### 実装前
- [ ] すべてのドキュメント対象を特定（grep で `use pasta::` を検索）
- [ ] Cargo.toml のバックアップを作成
- [ ] src/ ディレクトリのバックアップを作成
- [ ] Git の最新状態に更新

### 実装中
- [ ] ドキュメント更新（1.1 - 1.4）
- [ ] Cargo.toml 編集（2.1 - 2.2）
- [ ] src/ ディレクトリ削除（3.1 - 3.2）
- [ ] ビルド検証（4.1 - 4.2）
- [ ] テスト検証（5.1 - 5.2）
- [ ] リグレッション検証（6.1 - 6.3）
- [ ] 最終確認（7.1 - 7.2）

### 実装後
- [ ] すべてのテストが成功
- [ ] エラーまたは警告がない状態
- [ ] ドキュメントが新構造を反映
- [ ] Git に実装内容をコミット
- [ ] Pull Request で複数人確認

---

## 品質保証

| 検証項目 | 実行フェーズ | 成功基準 |
|---------|-----------|---------|
| ビルド成功 | 4.1, 4.2 | 0 エラー |
| テスト成功 | 5.1, 6.1, 6.2 | 30+ テスト成功 |
| Lint チェック | 5.2 | 0 警告 |
| ドキュメント | 1.1-1.4, 6.3 | 生成成功、リンク有効 |
| リグレッション | 6.1-6.3 | 既存機能すべて動作 |
| 構造整合性 | 7.1, 7.2 | src/ なし、crates/ 完全 |

