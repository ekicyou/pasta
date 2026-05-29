# 設計ドキュメント

## 概要

**目的**: pasta_sample_ghostクレート（~300行）の脆弱性監査とコード簡素化を実施し、画像処理・ファイルI/Oの安全性を強化する。

**対象者**: pasta開発者がコード品質改善の恩恵を受ける。

**影響**: 内部実装の安全性向上のみ。外部振る舞い（生成画像、API）は不変。

### ゴール
- 画像処理のピクセル座標計算における境界チェックの安全性確認・改善
- build.rsのパス操作の安全性検証
- デッドコード・冗長コードの除去
- 全既存テストのパス維持

### 非ゴール
- 画像デザインの変更
- 新しい表情やキャラクターの追加
- image/imageprocクレートのバージョンアップ
- ゴーストデータ（ghosts/）の内容変更

## 境界コミットメント

### 本仕様が責任を持つ範囲
- `crates/pasta_sample_ghost/src/` 配下の全5ファイルのコード品質改善
- `crates/pasta_sample_ghost/build.rs` の安全性確認
- 未使用コード・冗長パターンの除去

### 境界外
- `ghosts/hello-pasta/` のコンテンツ変更
- 他クレート（pasta_core, pasta_dsl等）への変更
- image/imageproc依存クレートの更新
- 公開API（`generate_ghost`, `GhostConfig`, `GhostError`）のシグネチャ変更

### 許容される依存
- image 0.25 / imageproc 0.26（既存バージョン維持）
- thiserror（既存バージョン維持）
- 標準ライブラリ（std::fs, std::path, std::env）

### 再検証トリガー
- image/imageprocクレートのメジャーバージョン更新時
- `GhostConfig`や`GhostError`の型定義変更時

## アーキテクチャ

### 既存アーキテクチャ分析

pasta_sample_ghostは以下の5モジュール構成:

| モジュール | 行数(概算) | 責務 |
|-----------|-----------|------|
| `lib.rs` | ~100行 | 公開API（`generate_ghost`）、型定義（`GhostConfig`, `GhostError`） |
| `main.rs` | ~90行 | CLIエントリポイント、引数解析、ファイルカウント |
| `image_generator.rs` | ~450行 | ピクトグラム画像生成（18サーフェス） |
| `config_templates.rs` | ~40行 | surfaces.txt生成 |
| `scripts.rs` | ~130行 | テストのみ（実行コードなし） |
| `build.rs` | ~50行 | ビルドスクリプト（rerun-if-changed、存在チェック） |

### テクノロジースタック

| レイヤー | 選定 / バージョン | 機能内の役割 | 備考 |
|---------|------------------|-------------|------|
| 画像処理 | image 0.25 / imageproc 0.26 | サーフェスPNG生成 | 変更なし |
| エラー処理 | thiserror 2 | GhostError定義 | 変更なし |
| ビルド | Cargo build.rs | rerun-if-changed管理 | 変更なし |

## ファイル構造計画

### 変更対象ファイル

- `crates/pasta_sample_ghost/src/image_generator.rs` — ピクセル座標の境界チェック改善、冗長な描画ヘルパーの簡素化
- `crates/pasta_sample_ghost/src/main.rs` — `walkdir`関数のシグネチャ改善（`&PathBuf` → `&Path`）、デッドコード確認
- `crates/pasta_sample_ghost/src/lib.rs` — `_config`パラメータの使用状況確認、デッドコード確認
- `crates/pasta_sample_ghost/src/config_templates.rs` — 変更不要（既にシンプル）
- `crates/pasta_sample_ghost/src/scripts.rs` — 変更不要（テストコードのみ）
- `crates/pasta_sample_ghost/build.rs` — 安全性確認のみ（現状で安全）

## コンポーネントとインターフェース

| コンポーネント | レイヤー | 意図 | 要件カバレッジ | 主要な変更 |
|--------------|---------|------|-------------|-----------|
| image_generator | 画像処理 | ピクセル座標安全性向上 | 1 | 境界チェック確認・冗長コード削減 |
| main.rs | CLI | シグネチャ改善 | 3, 5 | `&PathBuf` → `&Path` |
| lib.rs | 公開API | デッドコード確認 | 3, 4 | `_config`使用状況確認 |
| build.rs | ビルド | 安全性確認 | 2 | 変更なし（安全性確認のみ） |

### 画像処理レイヤー

#### image_generator.rs 安全性改善

| フィールド | 詳細 |
|-----------|------|
| 意図 | ピクセル座標計算の安全性確認と冗長コード削減 |
| 要件 | 1.1, 1.2, 1.3, 1.4 |

**責務と制約**
- 全ての `put_pixel` 呼び出しは既に `if x < WIDTH && y < HEIGHT` ガードが存在（現状安全）
- `draw_filled_circle_mut` はimageproc内部で境界チェック済み
- 定数（WIDTH=128, HEIGHT=256, HEAD_RADIUS=42等）は固定値であり、実行時オーバーフローリスクなし
- `i32` → `u32` キャスト時に負値チェックが `x < WIDTH` ガード内で暗黙的に保護されている

**発見事項**
- 現在のピクセル操作は全て安全。各 `put_pixel` 呼び出し前に境界チェックが実施済み
- 描画ヘルパー関数（`draw_thick_horizontal_line`, `draw_thick_vertical_line`等）は同一パターンの繰り返し — 共通化の余地あり
- ただし、可読性と外部振る舞い不変の優先度を考慮し、リスクのある大規模リファクタリングは避ける

### CLIレイヤー

#### main.rs 改善

| フィールド | 詳細 |
|-----------|------|
| 意図 | Rustイディオム準拠とデッドコード確認 |
| 要件 | 3.1, 3.2, 5.1, 5.2, 5.3 |

**改善対象**
- `walkdir(path: &PathBuf)` → `walkdir(path: &Path)`: Rustのイディオムに従い `&Path` を受け取る
- `count_files(dir: &PathBuf)` → `count_files(dir: &Path)`: 同上
- エラーハンドリングは `Box<dyn std::error::Error>` で適切に伝播済み

### ビルドレイヤー

#### build.rs 安全性確認

| フィールド | 詳細 |
|-----------|------|
| 意図 | ファイルI/O操作の安全性検証 |
| 要件 | 2.1, 2.2, 2.3, 2.4 |

**確認結果**
- `CARGO_MANIFEST_DIR` 未設定時は `expect()` でパニック — Cargoが必ず設定するため妥当
- `parent()` の `and_then` + `expect()` チェーン — ワークスペース構造が保証するため妥当
- 外部入力に基づくパス構築なし — パストラバーサルリスクなし
- `ghosts_dir` 不在時は警告のみ — 適切な振る舞い
- **結論: build.rsは現状で安全。変更不要**

## 要件トレーサビリティ

| 要件 | 概要 | コンポーネント | 対応 |
|------|------|-------------|------|
| 1.1 | ピクセル操作の境界安全性 | image_generator.rs | 確認済み（既に安全） |
| 1.2 | 整数オーバーフロー防止 | image_generator.rs | 確認済み（固定定数） |
| 1.3 | 境界チェック実施 | image_generator.rs | 確認済み（全put_pixelにガード） |
| 1.4 | 生成画像の同一性 | image_generator.rs | テストで検証 |
| 2.1 | CARGO_MANIFEST_DIR安全性 | build.rs | 確認済み（変更不要） |
| 2.2 | parent()安全性 | build.rs | 確認済み（変更不要） |
| 2.3 | パストラバーサル防止 | build.rs | 確認済み（外部入力なし） |
| 2.4 | ghosts不在時の警告 | build.rs | 確認済み（変更不要） |
| 3.1 | 未使用関数除去 | 全ファイル | `cargo clippy` で検証 |
| 3.2 | 未使用import除去 | 全ファイル | `cargo clippy` で検証 |
| 3.3 | テスト全パス | 全ファイル | `cargo test` で検証 |
| 3.4 | dead_code警告0件 | 全ファイル | `cargo clippy` で検証 |
| 4.1 | サーフェス画像同一性 | image_generator.rs | テストで検証 |
| 4.2 | surfaces.txt同一性 | config_templates.rs | テストで検証 |
| 4.3 | 公開APIシグネチャ不変 | lib.rs | コードレビューで検証 |
| 4.4 | テスト全パス | 全ファイル | `cargo test` で検証 |
| 5.1 | 書き込み権限エラー | main.rs | エラー伝播確認 |
| 5.2 | 無効パスエラー | main.rs | エラー伝播確認 |
| 5.3 | エラー伝播 | main.rs | Box\<dyn Error\>確認 |

## エラーハンドリング

### エラー戦略

pasta_sample_ghostのエラーハンドリングは既に適切に実装されている:
- `GhostError` enumが `ImageError`, `IoError`, `ConfigError` をラップ
- main.rsは `Box<dyn std::error::Error>` で全エラーを伝播
- build.rsは `expect()` でCargoが保証する環境変数をアンラップ

変更不要。現状のエラーハンドリングは監査基準を満たす。

## テスト戦略

### 単体テスト
1. **既存テスト全パス確認**: `cargo test -p pasta_sample_ghost` — 全既存テストが変更後もパスすること
2. **Clippy警告ゼロ確認**: `cargo clippy -p pasta_sample_ghost` — dead_code含む全警告が0件であること
3. **画像生成同一性**: `test_generate_surface` / `test_sakura_triangle_up_kero_triangle_down` — 生成画像の幾何学的性質が不変

### 統合テスト
1. **ゴースト生成E2E**: `generate_ghost()` が正常にsurface画像とsurfaces.txtを生成すること
2. **スクリプトテスト**: scripts.rs内のテストがghosts/hello-pastaの辞書ファイルを正常に読み込むこと
