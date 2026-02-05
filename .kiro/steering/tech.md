# Technical Steering

## 技術スタック

### 言語・ランタイム
- **Rust 2024 edition**: メインコンパイラ言語
- **Lua 5.5 (mlua 0.11)**: Luaバックエンドスクリプト実行
- **Pest 2.8**: PEGパーサー生成器（`pasta.pest`文法定義）

### ワークスペース構成
- **pasta_core**: 言語非依存層（パーサー、レジストリ）
- **pasta_lua**: Luaバックエンド層（pasta_core依存）
- **pasta_shiori**: SHIORI DLLインターフェース層

### 主要依存関係

**pasta_core:**
- **pest 2.8, pest_derive 2.8**: PEGパーサー生成器
- **thiserror 2**: エラー型定義
- **fast_radix_trie 1.1.0**: 前方一致シーン検索
- **rand 0.9**: ランダム選択（重複シーン、前方一致候補）
- **tracing 0.1**: ロギング・診断

**pasta_lua:**
- **pasta_core**: 言語非依存層
- **mlua 0.11**: Lua VMバインディング（Lua 5.5）
- **regex 1.x**: 正規表現（さくらスクリプトタグ検出）
- **thiserror 2**: エラー型定義
- **toml 0.9.8**: 設定ファイル管理
- **tracing 0.1**: ロギング・診断
- **luacheck v1.2.0**: 静的解析ツール（scriptlibs/）
- **lua_test**: BDDスタイルテストフレームワーク（scriptlibs/）

### 開発環境
- **tempfile 3**: テスト用一時ファイル生成

## アーキテクチャ原則

### ワークスペースレイヤー構成
```
pasta (workspace)
├── pasta_core          # 言語非依存層
│   ├── Parser          # DSL→AST変換
│   └── Registry        # シーン/単語テーブル
├── pasta_lua           # Luaバックエンド層
│   ├── Transpiler      # AST→Luaコード
│   ├── Runtime         # Lua VM実行
│   └── Loader          # スクリプト読み込み・キャッシュ
└── pasta_shiori        # SHIORI DLLインターフェース
```

| クレート     | レイヤー   | 責務                            |
| ------------ | ---------- | ------------------------------- |
| pasta_core   | Parser     | DSL→AST変換                     |
| pasta_core   | Registry   | シーン/単語テーブル管理         |
| pasta_lua    | Transpiler | AST→Luaコード変換               |
| pasta_lua    | Runtime    | Lua VM実行、コルーチン制御      |
| pasta_lua    | Loader     | スクリプト読み込み・キャッシュ  |
| pasta_shiori | SHIORI     | DLLエクスポート、リクエスト処理 |

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
  - マトリックスビルド: x86 (`i686-pc-windows-msvc`) + x64 (`x86_64-pc-windows-msvc`)
  - Rust キャッシュ: `Swatinem/rust-cache@v2`
  - アーティファクト: `pasta-dll-x86`, `pasta-dll-x64`（7日間保持）
