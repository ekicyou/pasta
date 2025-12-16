# Technical Steering

## 技術スタック

### 言語・ランタイム
- **Rust 2024 edition**: メインコンパイラ言語
- **Rune 0.14**: バックエンドスクリプトVM（generator機能を使用）
- **Pest 2.8**: PEGパーサー生成器（`pasta.pest`文法定義）

### 主要依存関係
- **thiserror 2**: エラー型定義
- **glob 0.3**: ファイルパターンマッチング（スクリプト読み込み）
- **tracing 0.1**: ロギング・診断
- **rand 0.9**: ランダム選択（重複ラベル、前方一致候補）
- **futures 0.3**: 非同期処理サポート
- **toml 0.9.8**: 設定ファイル管理
- **fast_radix_trie 1.1.0**: 前方一致ラベル検索

### 開発環境
- **tempfile 3**: テスト用一時ファイル生成

## アーキテクチャ原則

### レイヤー分離
Pastaエンジンは以下のレイヤーで構成：

1. **Parser Layer** (`src/parser/`):
   - Pasta DSLをPest文法でパース
   - ASTノード定義（`ast.rs`）
   - 行指向文法の解析

2. **Transpiler Layer** (`src/transpiler/`):
   - Pasta AST → Rune IRコード変換
   - 2パストランスパイル方式
   - ラベル管理・モジュール構造化

3. **Runtime Layer** (`src/runtime/`):
   - Rune VMによる実行
   - Generator機能によるyield型IR出力
   - 変数管理・ラベルテーブル

4. **Engine Layer** (`src/engine.rs`):
   - 上位APIの提供
   - キャッシュ管理
   - ディレクトリスクリプトローダー統合

5. **IR Output** (`src/ir.rs`):
   - `ScriptEvent`列挙型によるイベント出力
   - areka層への標準インターフェース

### 設計哲学

#### UI層からの独立性
- **タイミング制御なし**: Wait/Pause イベントはマーカーのみ、areka側が制御
- **バッファリングなし**: イベントは逐次yield、areka側がバッファ管理
- **同期制御なし**: Sync マーカーのみ、areka側が同期実装
- **レンダリングなし**: UIロジック一切含まず、純粋スクリプトエンジン

これにより**完全なユニットテスト可能性**を実現。

#### 宣言的コントロールフロー
- 命令型制御構文（if/while/for）を持たない
- Call文（`＞label`）、Jump文（`－label`）によるフロー制御
- Runeブロックで複雑なロジックを実装可能

#### Yield型実行モデル
- すべての出力は`yield ScriptEvent`で行う
- Generator継続により中断・再開が自然に実現
- チェイントーク対応

#### 2パストランスパイル
- Pass 1: AST解析、ラベル登録、依存解決
- Pass 2: Runeコード生成、モジュール構造化
- トランスパイル結果はキャッシュ保存、デバッグ参照可能

## コーディング規約

### ファイル命名
- テストファイル: `<feature>_test.rs`（単数形、アンダースコア区切り）
- モジュール: `mod.rs`または単一機能モジュール
- Fixture: `tests/fixtures/<scenario>.pasta`

### 識別子規約
- Rust側: スネークケース（`variable_name`）
- Pasta DSL側: 日本語識別子・UNICODE識別子サポート
- ラベル名: 日本語可、全角記号推奨（`＊挨拶`）

### エラーハンドリング
- `thiserror`による構造化エラー型（`PastaError`）
- `Result<T, PastaError>`での統一
- コンテキスト情報（ファイル名、行番号、カラム位置）の保持

### テスト戦略
- **ユニットテスト**: 各レイヤーごとに独立テスト
- **統合テスト**: `tests/`ディレクトリ配下
- **Fixture駆動**: `tests/fixtures/`内のPastaスクリプトを使用
- **E2Eテスト**: `engine_integration_test.rs`で完全フロー検証
- **Doctest**: 公開API例をドキュメントコメント内に記述

### ドキュメント規約
- 公開API: `///`ドキュメントコメント必須
- モジュールレベル: `//!`で概要記述
- 複雑なロジック: インラインコメントで意図説明
- Grammar: `GRAMMAR.md`でDSL文法全体を文書化

## 品質基準

### テストカバレッジ
- 新機能: 対応するテストケース必須
- リグレッション防止: 既存テストは常にパス
- エッジケース: エラーハンドリング含めテスト

### パフォーマンス
- パース結果キャッシング（`cache.rs`）
- ラベル検索: HashMap O(1)
- 前方一致検索: Radix Trie
- 重複ラベル: 事前グルーピングで高速ランダム選択

### セキュリティ
- サンドボックス: Rune VMのセキュリティ機構に依存
- 入力検証: Pest文法による厳密なパース
- リソース制限: 将来的にRune VM実行時間制限を検討

## 依存関係管理

### バージョン戦略
- セマンティックバージョニング準拠
- Rune 0.14に固定（破壊的変更対応まで）
- 依存ライブラリ: メジャーバージョン指定

### ライセンス
- **MIT OR Apache-2.0**: デュアルライセンス
- 依存関係ライセンス: 互換性確認済み

### 公開ポリシー
- `publish = true` in Cargo.toml
- crates.io公開予定
- API安定化後にリリース

## デプロイメント

### ビルド設定
- `edition = "2024"`
- ワークスペース独立構成（`[workspace]`）
- 最適化: リリースビルドでLTO有効化予定

### テスト実行
```bash
cargo test --all-features
cargo test --release  # パフォーマンステスト
```

### CI/CD（予定）
- GitHub Actions: PR時の自動テスト
- Clippy/Rustfmt チェック
- ドキュメント生成・公開

### SHIORI.DLL統合（将来）
- Windows DLL出力設定
- C FFI ラッパー実装
- 伺かベースウェア互換レイヤー

### areka統合
- `areka`アプリケーションからの動的リンク
- MCP Server統合（`areka-P0-mcp-server`仕様）
- パッケージマネージャ統合（`areka-P0-package-manager`仕様）
