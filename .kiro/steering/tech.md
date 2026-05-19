# Technical Steering

## 技術スタック

### 言語・ランタイム
- **Rust 2024 edition**: メインコンパイラ言語
- **LuaJIT 2.1 (mlua 0.11)**: Luaバックエンドスクリプト実行
- **Pest 2.8.6**: PEGパーサー生成器（`pasta.pest`文法定義）

### ワークスペース構成
- **pasta_dsl**: DSLパーサー層（Pest PEG → AST変換）
- **pasta_core**: 言語非依存層（レジストリ）
- **pasta_lua**: Luaバックエンド層（pasta_dsl + pasta_core依存）
- **pasta_shiori**: SHIORI DLLインターフェース層
- **pasta_lsp**: LSP実装層（tower-lsp, WASM/Native対応）
- **pasta_check**: リリースCLIツール（ゴーストパッケージング・NAR生成）
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
- **mlua 0.11**: Lua VMバインディング（LuaJIT 2.1, features: luajit52, vendored, serialize）
- **mlua-stdlib 0.1**: Lua標準拡張ライブラリ（json, regex, yaml）
- **regex 1.x**: 正規表現（さくらスクリプトタグ検出）
- **thiserror 2**: エラー型定義
- **toml 0.9.8**: 設定ファイル管理
- **serde 1 / serde_json 1**: シリアライゼーション
- **glob 0.3**: ファイルパターンマッチ
- **flate2 1.x**: gzip圧縮（キャッシュ等）
- **budoux 0.1.1**: 日本語改行位置推定（BudouX）
- **unicode-width 0.2.2**: Unicode文字幅計算
- **tracing 0.1 / tracing-appender 0.2 / tracing-subscriber 0.3**: ロギング・診断
- **windows-sys 0.61**: Windows API（Shift_JISエンコーディング等、cfg(windows)）
- **luacheck v1.2.0**: 静的解析ツール（scriptlibs/）
- **lua_test**: BDDスタイルテストフレームワーク（scriptlibs/）

**pasta_shiori:**
- **pasta_core, pasta_lua**: 内部依存
- **pest 2.8.6, pest_derive 2.8.6**: SHIORIプロトコルパーサー
- **time 0.3**: 時刻処理
- **tracing 0.1**: ロギング（subscriber/appenderはpasta_luaに移管）
- **thiserror 2**: エラー型定義
- **windows-sys 0.61**: Windows DLL API（cfg(windows)）

**pasta_lsp:**
- **pasta_dsl**: DSLパーサー層
- **tower-lsp 0.20**: LSPサーバーフレームワーク
- **serde 1 / serde_json 1**: シリアライゼーション
- **thiserror 2**: エラー型定義
- **wasm-bindgen, js-sys**: WASM対応（cfg(wasm32)）

**pasta_check:**
- **lexopt 0.3**: CLIパーサー
- **md5 0.8**: ハッシュ計算（更新ファイル生成）
- **zip 8.4**: NAR（ZIP）アーカイブ作成
- **pasta_lua**: 将来のLua単体試験サポート基盤
- **thiserror 2**: エラー型定義

**pasta_sample_ghost:**
- **image 0.25 / imageproc 0.26**: ピクトグラム画像生成
- **thiserror 2**: エラー型定義

### 開発環境
- **tempfile 3**: テスト用一時ファイル生成
- **insta 1.46**: スナップショットテスト（glob機能付き）
- **tracing-test 0.2**: テスト用ログキャプチャ

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
├── pasta_check         # リリースCLI（NAR生成・更新ファイル）
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
| pasta_check        | CLI          | リリースパッケージング・NAR生成 |
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

## Luaランタイムパターン

### scripts/ 優先順位
- `scripts/`（ゴーストカスタム）> `pasta_scripts/`（エンジン同梱）
- `scripts/main.lua` がエントリーポイント。同名ファイルでエンジン動作を上書き可能

### DSL vs Lua 判断基準

| ケース                       | 推奨                      |
| ---------------------------- | ------------------------- |
| 数個の単語定義               | DSL（`＠単語：値1、値2`） |
| 数十〜数百件の単語一括投入   | Lua（`WORD.create_*`）    |
| 基本的なシーン定義           | DSL（`＊シーン名`）       |
| 条件分岐を含む複雑なロジック | Lua（シーン関数）         |
| カスタムSHIORIイベント処理   | Lua（REGテーブル）        |

### Rust組み込みモジュール

| モジュール             | 用途                                    | 注意                       |
| ---------------------- | --------------------------------------- | -------------------------- |
| `@pasta_search`        | シーン・単語検索                        |                            |
| `@pasta_persistence`   | セーブデータ永続化                      |                            |
| `@pasta_config`        | pasta.toml設定読み取り                  | **`pcall` 必須**           |
| `@pasta_sakura_script` | さくらスクリプト変換                    |                            |
| `@enc`                 | UTF-8⇔ANSI変換                          |                            |
| `@pasta_log`           | ロギング（trace/debug/info/warn/error） |                            |
| `@env`                 | 環境変数アクセス                        | **無効**（セキュリティ上） |

### シーン関数定型パターン

```lua
function SCENE.func_name(act)
    local save, var = act:init_scene(SCENE)  -- 必須
    act:talk(act.アクター名.actor, "セリフ")
    act:yield()
end
```

### SHIORIハンドラ登録パターン

```lua
REG.OnBoot = function(req)
    return RES.ok("value0")
end
```

詳細: `.agents/skills/pasta-lua-coding/SKILL.md`
