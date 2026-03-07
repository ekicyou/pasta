# Technical Steering

## 技術スタック

### 言語・ランタイム
- **Rust 2024 edition**: メインコンパイラ言語
- **Lua 5.5 (mlua 0.11)**: Luaバックエンドスクリプト実行
- **Pest 2.8.6**: PEGパーサー生成器（`pasta.pest`文法定義）

### ワークスペース構成
- **pasta_dsl**: DSLパーサー層（Pest PEG → AST変換）
- **pasta_core**: 言語非依存層（レジストリ）
- **pasta_lua**: Luaバックエンド層（pasta_dsl + pasta_core依存）
- **pasta_shiori**: SHIORI DLLインターフェース層
- **pasta_lsp**: LSP実装層（tower-lsp, WASM/Native対応）
- **pasta_sample_ghost**: サンプルゴースト「hello-pasta」（publish=false, 画像生成・配布物作成）

### 主要依存関係

**pasta_dsl:**
- **pest 2.8.6, pest_derive 2.8.6**: PEGパーサー生成器
- **thiserror 2**: エラー型定義

**pasta_core:**
- **thiserror 2**: エラー型定義
- **fast_radix_trie 1.1.0**: 前方一致シーン検索
- **rand 0.9**: ランダム選択（重複シーン、前方一致候補）
- **tracing 0.1**: ロギング・診断

**pasta_lua:**
- **pasta_dsl**: DSLパーサー層
- **pasta_core**: レジストリ層
- **mlua 0.11**: Lua VMバインディング（Lua 5.5）
- **mlua-stdlib 0.1**: Lua標準拡張ライブラリ（json, regex, yaml）
- **regex 1.x**: 正規表現（さくらスクリプトタグ検出）
- **thiserror 2**: エラー型定義
- **toml 0.9.8**: 設定ファイル管理
- **serde 1 / serde_json 1**: シリアライゼーション
- **glob 0.3**: ファイルパターンマッチ
- **flate2 1.x**: gzip圧縮（キャッシュ等）
- **tracing 0.1 / tracing-appender 0.2 / tracing-subscriber 0.3**: ロギング・診断
- **windows-sys 0.61**: Windows API（Shift_JISエンコーディング等、cfg(windows)）
- **luacheck v1.2.0**: 静的解析ツール（scriptlibs/）
- **lua_test**: BDDスタイルテストフレームワーク（scriptlibs/）

**pasta_shiori:**
- **pasta_core, pasta_lua**: 内部依存
- **pest 2.8.6, pest_derive 2.8.6**: SHIORIプロトコルパーサー
- **time 0.3**: 時刻処理
- **tracing 0.1 / tracing-subscriber 0.3 / tracing-appender 0.2**: ロギング
- **thiserror 2**: エラー型定義
- **windows-sys 0.59**: Windows DLL API（cfg(windows)）

**pasta_lsp:**
- **pasta_dsl**: DSLパーサー層
- **tower-lsp 0.20**: LSPサーバーフレームワーク
- **serde 1 / serde_json 1**: シリアライゼーション
- **thiserror 2**: エラー型定義
- **wasm-bindgen, js-sys**: WASM対応（cfg(wasm32)）

**pasta_sample_ghost:**
- **image 0.25 / imageproc 0.25**: ピクトグラム画像生成
- **md5 0.7**: ハッシュ計算（更新検出）
- **encoding_rs 0.8**: Shift_JISエンコーディング
- **thiserror 2**: エラー型定義

### 開発環境
- **tempfile 3**: テスト用一時ファイル生成

## アーキテクチャ原則

### ワークスペースレイヤー構成
```
pasta (workspace)
├── pasta_dsl           # DSLパーサー層
│   └── Parser          # DSL→AST変換（Pest PEG）
├── pasta_core          # 言語非依存層
│   └── Registry        # シーン/単語テーブル
├── pasta_lua           # Luaバックエンド層
│   ├── Transpiler      # AST→Luaコード
│   ├── Runtime         # Lua VM実行
│   ├── Loader          # スクリプト読み込み・キャッシュ
│   ├── SakuraScript    # さくらスクリプト処理
│   ├── Search          # Rust/Lua間検索バインディング
│   └── Encoding        # プラットフォーム別エンコーディング
├── pasta_shiori        # SHIORI DLLインターフェース
├── pasta_lsp           # LSP実装（WASM/Native）
└── pasta_sample_ghost  # サンプルゴースト生成ツール
```

| クレート           | レイヤー     | 責務                            |
| ------------------ | ------------ | ------------------------------- |
| pasta_dsl          | Parser       | DSL→AST変換                     |
| pasta_core         | Registry     | シーン/単語テーブル管理         |
| pasta_lua          | Transpiler   | AST→Luaコード変換               |
| pasta_lua          | Runtime      | Lua VM実行、コルーチン制御      |
| pasta_lua          | Loader       | スクリプト読み込み・キャッシュ  |
| pasta_shiori       | SHIORI       | DLLエクスポート、リクエスト処理 |
| pasta_lsp          | LSP          | 構文ハイライト、診断、WASM対応  |
| pasta_sample_ghost | Distribution | サンプルゴースト画像生成・配布  |

### 設計哲学

| 原則         | 内容                                   |
| ------------ | -------------------------------------- |
| UI独立性     | Wait/Syncはマーカーのみ、areka側で制御 |
| 宣言的フロー | Call/Jumpで制御、if/while/forなし      |
| Yield型      | 全出力はyield、Generator継続           |
| 2パス変換    | Pass1: シーン登録、Pass2: コード生成   |

**結果**: 完全なユニットテスト可能性を実現

## コーディング規約

| 項目           | 規約                    |
| -------------- | ----------------------- |
| テストファイル | `<feature>_test.rs`     |
| Rust識別子     | スネークケース          |
| DSL識別子      | 日本語/UNICODE可        |
| エラー型       | `Result<T, PastaError>` |
| ドキュメント   | `///`で公開API          |

### テスト戦略
- ユニット: レイヤー独立
- 統合: `crates/*/tests/`配下
- Fixture: `crates/*/tests/fixtures/*.pasta`
- Doctest: API例をドキュメント内に

## 品質基準

| 項目         | 基準                           |
| ------------ | ------------------------------ |
| テスト       | 新機能必須、リグレッション防止 |
| キャッシュ   | パース結果をメモリ保持         |
| 検索性能     | シーンO(1)、前方一致Radix Trie |
| セキュリティ | Lua VMサンドボックス依存       |

## 依存関係管理

### バージョン戦略
- セマンティックバージョニング準拠
- 依存ライブラリ: メジャーバージョン指定

### ライセンス
- **MIT OR Apache-2.0**: デュアルライセンス
- 依存関係ライセンス: 互換性確認済み

### 公開ポリシー
- `publish = true` in Cargo.toml
- crates.io公開予定
- API安定化後にリリース

## デプロイメント

```bash
cargo build --workspace     # ワークスペースビルド
cargo test --workspace      # 全テスト
cargo test --release        # リリースビルド
```

### クレート別コマンド
```bash
cargo build -p pasta_core   # pasta_coreビルド
cargo build -p pasta_lua    # pasta_luaビルド
cargo test -p pasta_core    # pasta_coreテスト
cargo test -p pasta_lua     # pasta_luaテスト
```

### 将来計画
- SHIORI.DLL: C FFIラッパー
- areka統合: 動的リンク、MCP Server

### CI/CD
- **GitHub Actions**: `.github/workflows/build.yml`
  - push/PR/手動実行トリガー
  - **DLLビルド**: マトリックスビルド: x86 (`i686-pc-windows-msvc`) + x64 (`x86_64-pc-windows-msvc`)
  - **WASMビルド**: `pasta_lsp` を `wasm32-unknown-unknown` ターゲットでビルド（10MBサイズ上限チェック付き）
  - Rust キャッシュ: `Swatinem/rust-cache@v2`
  - アーティファクト: `pasta-dll-x86`, `pasta-dll-x64`（7日間保持）
