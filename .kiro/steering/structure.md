# Project Structure Steering

## ディレクトリ構造

```
pasta/                        # Cargo ワークスペースルート（Pure Virtual Workspace）
├── Cargo.toml               # ワークスペース設定のみ（[package] セクションなし）
├── crates/                  # クレート群
│   ├── pasta_dsl/           # DSLパーサー層
│   │   ├── Cargo.toml       # pasta_dsl設定
│   │   ├── tests/           # 外部化テスト（Phase A）
│   │   └── src/
│   │       ├── lib.rs       # クレートエントリーポイント
│   │       ├── error.rs     # ParseError, ParseErrorInfo, ParseResult
│   │       └── parser/      # パーサーレイヤー（PEG → AST変換）
│   │           ├── mod.rs   # パーサーAPI公開・エントリーポイント
│   │           ├── parse_scene.rs    # シーン解析サブモジュール
│   │           ├── parse_action.rs   # アクション解析サブモジュール
│   │           ├── parse_elements.rs # 要素解析サブモジュール
│   │           ├── ast/              # AST型定義（ディレクトリモジュール）
│   │           │   ├── mod.rs        # 全型の pub use re-export
│   │           │   ├── span.rs       # Span型定義
│   │           │   ├── scene.rs      # シーン関連AST型
│   │           │   ├── action.rs     # アクション関連AST型
│   │           │   └── cue.rs        # キューコマンドAST型（CueCommandNode等）
│   │           └── grammar.pest # Pest文法定義
│   ├── pasta_core/          # 言語非依存層（レジストリ）
│   │   ├── Cargo.toml       # pasta_core設定
│   │   ├── tests/           # 外部化テスト（Phase A）
│   │   │   └── word_table_test.rs # 単語テーブルテスト
│   │   └── src/
│   │       ├── lib.rs       # クレートエントリーポイント
│   │       ├── error.rs     # SceneTableError, WordTableError
│   │       └── registry/    # 型管理レイヤー（独立）
│   │           ├── mod.rs   # Registry API
│   │           ├── scene_registry.rs  # SceneRegistry - シーン管理
│   │           ├── word_registry.rs   # WordDefRegistry - 単語辞書
│   │           ├── scene_table.rs     # SceneTable - シーン検索
│   │           ├── scene_table_candidate_tests.rs      # シーンテーブルテスト（候補収集/解決・#[path]パターン）
│   │           ├── scene_table_resolve_filter_tests.rs # シーンテーブルテスト（解決境界/フィルタ/アクセサ・#[path]パターン）
│   │           ├── scene_types.rs     # シーンID・スコープ・情報型定義
│   │           ├── word_table.rs      # WordTable - 単語検索
│   │           └── random.rs          # RandomSelector - ランダム選択
│   └── pasta_lua/           # Lua言語バックエンド層
│       ├── Cargo.toml       # pasta_lua設定（pasta_core依存）
│       ├── build.rs         # pasta_scriptsを決定論zip化→OUT_DIR、MD5をcargo:rustc-envで公開
│       ├── build_zip.rs     # build.rsとテスト共有の決定論zipパッカー（純粋関数）
│       ├── scripts/         # ユーザーカスタムLuaスクリプト（profile/pasta/pasta_scriptsより優先）
│       ├── pasta_scripts/   # 標準ランタイムLuaスクリプト（埋め込みzipのソース正本。main.lua, pasta/等）
│       ├── src/
│       │   ├── lib.rs       # クレートエントリーポイント
│       │   ├── config.rs    # 設定管理
│       │   ├── code_gen/    # Luaコード生成（ディレクトリモジュール）
│       │   │   ├── mod.rs          # コード生成エントリーポイント
│       │   │   ├── scope_gen.rs    # スコープ生成（分割impl）
│       │   │   ├── element_gen.rs  # 要素生成（分割impl）
│       │   │   └── source_map.rs   # .pastaソースマップ記録シーム（SourceMapSink/PastaPos）
│       │   ├── debug/      # VSCode Luaデバッグバックエンド（DAP・既定無効/ゼロコスト・SHIORI非依存）
│       │   │   ├── mod.rs          # DebugConfig/enable()/DebugHandle/DebugError・有効化ゲート
│       │   │   ├── config.rs / enable.rs / error.rs / handle.rs # 設定・有効化・エラー・ハンドル（フラットモジュール）
│       │   │   ├── types.rs        # 共有DTO（SessionCommand/Event・FrameInfo・ThreadId等）
│       │   │   ├── breakpoints.rs  # BreakpointSet（Arc<Mutex>共有）
│       │   │   ├── hook.rs         # set_global_hook＋jit.off・LineHookシーム
│       │   │   ├── inspect.rs      # FrameInspector（mlua::ffi・コルーチンstate走査）
│       │   │   ├── source_mode.rs  # ソース表示モード（.pasta/Lua ビュー切替）
│       │   │   ├── session/        # 停止状態機械（mod/anchor/stepping/stop_loop に分解）
│       │   │   ├── dap/            # DAP最小サブセット（codec/decode/encode/pending/resolver・serde_json手書き）
│       │   │   ├── transport/      # TCP＋Content-Lengthフレーミング（mod/framing・I/O専用）
│       │   │   ├── wiring/         # transport↔dap↔session↔hook結線（mod/bridge/inbound/resolver）
│       │   │   └── source_map/     # .pasta本番ソースマップ（mod/sidecar・正規化キー・任意サイドカー）
│       │   │   # 注: 上記に対応する src 内テスト（*_tests.rs）・共有ヘルパ（*_test_support.rs）が多数併置（後述「src/ 内テスト配置方針」）
│       │   ├── context.rs   # トランスパイルコンテキスト
│       │   ├── error.rs     # エラー型
│       │   ├── encoding/    # エンコーディング処理（プラットフォーム別分割）
│       │   │   ├── mod.rs           # エンコーディングAPI
│       │   │   ├── windows.rs       # Windows固有（Shift_JIS等）
│       │   │   └── unix.rs          # Unix固有
│       │   ├── loader/      # スクリプトローダー（ディレクトリモジュール）
│       │   │   ├── mod.rs           # ローダーAPI
│       │   │   ├── cache.rs         # キャッシュ管理
│       │   │   ├── config.rs        # ローダー設定
│       │   │   ├── context.rs       # ローダーコンテキスト
│       │   │   ├── discovery.rs     # スクリプト検出
│       │   │   ├── extract.rs       # 起動時自己展開（内蔵zip解凍・MD5マーカー比較・準アトミック展開）
│       │   │   └── error.rs         # ローダーエラー型
│       │   ├── logging/     # ロギング設定
│       │   ├── normalize.rs # 正規化ユーティリティ
│       │   ├── runtime/     # ランタイムレイヤー
│       │   │   ├── mod.rs              # ランタイムコア
│       │   │   ├── runtime_config.rs   # ランタイム設定構造体
│       │   │   ├── module_registry.rs  # モジュール登録関数群（分割impl）
│       │   │   ├── enc.rs              # ランタイムエンコーディング
│       │   │   ├── persistence.rs      # 永続化
│       │   │   ├── finalize.rs         # ファイナライズ処理
│       │   │   ├── renderer_injection.rs # さくらレンダラのアダプタ注入シーム（既定=SHIORIさくら・バイト不変）
│       │   │   └── log.rs              # ランタイムログ
│       │   ├── sakura_script/ # さくらスクリプト処理
│       │   │   ├── mod.rs           # さくらスクリプトAPI
│       │   │   ├── tokenizer.rs     # トークナイザー
│       │   │   └── wait_inserter.rs # ウェイト挿入
│       │   ├── presentation/  # 宿主非依存 presentation マーカー契約（talk/actor切替/wait/choice・拡張可能）
│       │   │   ├── mod.rs           # PresentationEvent 型体系・RenderBoundary
│       │   │   └── marker.rs        # 最小マーカー集合の型表現
│       │   ├── search/       # 検索機能（Rust/Lua間バインディング）
│       │   │   ├── mod.rs           # 検索API
│       │   │   ├── context.rs       # 検索コンテキスト
│       │   │   └── error.rs         # 検索エラー型
│       │   ├── string_literalizer.rs # 文字列リテラル化
│       │   └── transpiler.rs # トランスパイラーエントリーポイント
│       └── tests/           # pasta_lua統合テスト（機能ドメイン別サブディレクトリ化済み）
│           ├── transpiler/          # トランスパイラ関連テスト（8 files）
│           │   ├── main.rs          # エントリーポイント（#[path] で common 参照）
│           │   ├── basic_test.rs
│           │   ├── comparison_test.rs
│           │   ├── scene_test.rs
│           │   ├── snapshot_test.rs
│           │   ├── actor_word_dictionary_test.rs
│           │   ├── fallback_search_integration_test.rs
│           │   ├── code_generator_test.rs
│           │   ├── cue_command_passthrough_test.rs
│           │   └── snapshots/       # insta スナップショット
│           ├── loader/              # ローダー関連テスト（6 files）
│           │   ├── main.rs
│           │   ├── cache_test.rs
│           │   ├── config_test.rs
│           │   ├── lifecycle_test.rs
│           │   ├── startup_test.rs
│           │   ├── config_actors_initialization_test.rs
│           │   └── lua_passthrough_test.rs
│           ├── shiori/              # SHIORI関連テスト（5 files）
│           │   ├── main.rs
│           │   ├── event_dispatch_test.rs
│           │   ├── event_handler_test.rs
│           │   ├── res_test.rs
│           │   ├── virtual_event_config_test.rs
│           │   └── virtual_event_dispatch_test.rs
│           ├── runtime/             # ランタイム関連テスト（8 files）
│           │   ├── main.rs
│           │   ├── finalize_scene_test.rs
│           │   ├── scene_test.rs
│           │   ├── syntax_test.rs
│           │   ├── unit_test.rs
│           │   ├── persistence_integration_test.rs
│           │   ├── encoding_test.rs
│           │   ├── stdlib_modules_test.rs
│           │   └── stdlib_regex_test.rs
│           ├── log/                 # ログ関連テスト（3 files）
│           │   ├── main.rs
│           │   ├── integration_test.rs
│           │   ├── module_test.rs
│           │   └── stack_level_test.rs
│           ├── sakura_script/       # SakuraScript関連テスト（2 files）
│           │   ├── main.rs
│           │   ├── basic_test.rs
│           │   └── output_test.rs
│           ├── search/              # 検索関連テスト（2 files）
│           │   ├── main.rs
│           │   ├── scene_search_test.rs
│           │   └── module_test.rs
│           ├── common/              # テスト共通ユーティリティ
│           │   ├── mod.rs
│           │   └── e2e_helpers.rs
│           ├── fixtures/            # テスト用Pastaスクリプト
│           ├── lua_specs/           # Lua単体テスト仕様
│           ├── lua_unittest_runner.rs  # Lua単体テストランナー（命名例外）
│           ├── japanese_identifier_test.rs  # Lua基盤テスト
│           └── ucid_test.rs         # Lua基盤テスト
│   ├── pasta_lsp/           # LSP実装層
│   │   ├── Cargo.toml       # pasta_lsp設定（tower-lsp, pasta_dsl依存）
│   │   ├── README.md        # クレート概要
│   │   ├── tests/           # 外部化テスト（Phase A）
│   │   └── src/
│   │       ├── lib.rs       # クレートエントリーポイント
│   │       ├── analysis/    # 解析エンジン（ディレクトリモジュール）
│   │       │   ├── mod.rs          # 解析API・全公開型 re-export
│   │       │   ├── token_types.rs  # トークン型定義
│   │       │   ├── visit_scope.rs  # スコープ走査ビジター（旧 visitors.rs を分解）
│   │       │   ├── visit_action.rs # アクション走査ビジター
│   │       │   ├── visit_expr.rs   # 式走査ビジター
│   │       │   └── text_utils.rs   # テキストユーティリティ
│   │       ├── document.rs  # ドキュメント状態管理
│   │       ├── error.rs     # LangServerError型定義
│   │       ├── server.rs    # PastaLangServer (tower-lsp trait実装)
│   │       └── transport.rs # WASM/Nativeプラットフォーム抽象化
│   ├── pasta_check/         # リリースCLIツール
│   │   ├── Cargo.toml       # pasta_check設定（lexopt, md5, zip, pasta_lua依存）
│   │   └── src/
│   │       ├── main.rs          # CLIエントリーポイント
│   │       ├── release.rs       # リリースビルドオーケストレーション
│   │       ├── update_files.rs  # 更新ファイル（updates.txt）生成
│   │       ├── nar.rs           # NAR（ZIP）アーカイブ作成
│   │       └── copy.rs          # ファイルコピーユーティリティ
│   └── pasta_sample_ghost/  # サンプルゴースト「hello-pasta」（publish=false）
│       ├── Cargo.toml       # 画像生成・配布物作成用依存
│       ├── README.md        # クレート概要
│       ├── RELEASE.md       # リリース手順
│       ├── release.ps1      # ビルド＋配布パッケージ生成スクリプト
│       ├── release.bat      # release.ps1のバッチラッパー
│       ├── build.rs         # ビルドスクリプト
│       ├── src/
│       │   ├── lib.rs              # 公開API（画像＋surfaces.txt生成）
│       │   ├── main.rs             # 配布物生成CLIエントリーポイント
│       │   ├── image_generator.rs  # ピクトグラム画像生成
│       │   ├── config_templates.rs # surfaces.txt生成
│       │   └── scripts.rs          # ghosts/hello-pasta 辞書(.pasta)の検証テスト
│       ├── ghosts/           # サンプルゴースト本体（SSOT・配布物一式）
│       │   └── hello-pasta/  # 手書きSSOT(descript/pasta.toml/dic/install)＋生成物(dll/画像)
│       └── tests/            # 統合テスト・配布ファイル構成検証
├── benches/                  # ベンチマークコード
├── editors/                  # エディタ拡張
│   └── vscode/              # VSCode拡張（TypeScript + WASM統合）
│       ├── package.json     # 拡張マニフェスト
│       ├── tsconfig.json    # TypeScript設定
│       ├── src/             # TypeScriptソース
│       │   ├── extension.ts            # アクティベーション・エントリポイント
│       │   ├── wasmBridge.ts           # WASMモジュールブリッジ
│       │   ├── semanticTokensProvider.ts # セマンティックトークン提供
│       │   ├── diagnosticsManager.ts   # 診断情報管理
│       │   ├── documentSync.ts         # ドキュメント同期（200msデバウンス）
│       │   └── test/                   # テスト
│       ├── syntaxes/         # TextMate文法（全角/半角マーカー対応）
│       ├── scripts/          # ビルドスクリプト（WASM等）
│       └── wasm/             # pasta_lsp WASMバイナリ（ビルド生成物）
├── book/                     # 利用者マニュアル（mdBook・GitHub Pages 公開）
│   ├── book.toml            # mdBook 設定（language=ja, 検索有効, site-url）
│   ├── package.json         # book ツールの npm 依存（vscode-textmate/oniguruma/jsdom・lockfile コミット・node_modules 非コミット）
│   ├── src/                 # 章ソース（grammar/lua/getting-started/reference）
│   ├── theme/head.hbs       # 日本語 bigram 検索 tokenizer ＋ pasta ハイライト中和の override
│   ├── tools/               # build-time Node（bigram 索引再生成・drift-check・pasta 構文ハイライト 等）
│   ├── manual-sources.toml  # ドリフト検出マッピング（doc/spec 由来追跡）
│   └── book/                # mdbook build 生成物（.gitignore 済み・CI で再生成）
├── .kiro/                    # Kiro Spec-Driven設定
│   ├── steering/            # ステアリング規約
│   ├── settings/            # テンプレート・ルール
│   └── specs/               # 仕様管理
│       ├── completed/       # 完了仕様（アーカイブ）
│       └── <spec-name>/     # 進行中仕様
├── .vscode/                 # VS Code 設定
├── .github/                 # GitHub Actions, PR テンプレート
├── README.md                # プロジェクト概要
├── GRAMMAR.md               # Pasta DSL文法リファレンス
├─ doc/spec/                # 言語仕様書（章別分割）
├── LICENSE                  # ライセンス
└── CLAUDE.md                # AI開発支援（プロジェクト指示・Kiro ワークフロー・コマンド一覧）
```

**注**:
- ルートクレート (`src/`) は削除済み。すべての実装コードは `crates/*/src/` 配下に配置。
- ルートレベルの `tests/` と `examples/` も削除済み（Pure Virtual Workspace移行完了）
- 各クレートは独自の `tests/` ディレクトリを持つことができる（例: pasta_dsl, pasta_core, pasta_lua, pasta_lsp, pasta_sample_ghost, pasta_shiori）
- pasta_sample_ghost のサンプルゴースト hello-pasta は `ghosts/hello-pasta/` に完全な一式として直接配置（SSOT）。テキスト系（descript/pasta.toml/dic/install）は手書き正本、画像・DLL は生成物。`release.ps1` は生成物の配置と `.nar` パッケージングを担う（旧 dist-src/robocopy 方式は廃止）
- `CLAUDE.md` が AI 開発支援の指示（プロジェクト指示・Kiro 仕様駆動ワークフロー・コマンド一覧）を担う

## ファイル命名規則

### ソースファイル
- モジュールエントリー: `mod.rs`
- 単一機能モジュール: `<feature>.rs`（例: `engine.rs`, `cache.rs`）
- サブモジュール: ディレクトリ作成し`mod.rs`配置

### テストファイル
- 統合テスト: `crates/<crate>/tests/<feature>_test.rs`（アンダースコア区切り、単数形）
- フィクスチャ: `crates/<crate>/tests/fixtures/<scenario>.pasta`
- 共通ユーティリティ: `crates/<crate>/tests/common/mod.rs`
- クレート専用テスト: 各クレート配下の `tests/` に配置可能
- **命名例外**: `lua_unittest_runner.rs` — テストランナーはテストとは役割が異なり、`_runner.rs` サフィックスでその役割を明示する

### テストサブモジュール化方針

テストファイルが 10 本を超えるクレートについては、機能ドメイン別サブディレクトリに分割する。

**パターン**: `tests/<category>/main.rs` + `#[path = "../common/mod.rs"] mod common;`

- 各サブディレクトリの `main.rs` はエントリーポイント。テスト関数は配置せず、`mod` 宣言のみ記述する
- `common/` を使用するサブモジュール内では `use crate::common;` で参照する
- 3 ファイル未満のドメインは類似ドメインに統合するか、フラット残留とする

### src/ 内テスト配置方針

1. private フィールドへの直接アクセスが構造的に必要なテストは、`#[cfg(test)] #[path = "<name>_tests.rs"] mod tests;` パターンで `src/` 内に配置する
2. 公開 API のみをテストする統合テストは、従来通り `tests/` に外部化する
3. 判断基準: テスト対象の構造体が `pub(crate)` 以下のフィールドを持ち、それらに直接アクセスしなければテストが成立しない場合に限り `src/` 内に配置する

**既存の適用例**:
- `pasta_core/src/registry/scene_table_candidate_tests.rs`・`scene_table_resolve_filter_tests.rs`（SceneTable の labels, prefix_index への直接アクセス）
- `pasta_shiori/src/shiori_lifecycle_tests.rs`・`shiori_request_tests.rs`（ShioriService の cache への直接アクセス）
- `pasta_lua/src/debug/` 配下（`*_tests.rs` 多数。デバッグ状態機械・DAP・wiring の内部状態を直接検証）

**命名規約（src 内テスト）**:
- テストモジュール本体: `<feature>_tests.rs`（複数形 `_tests`。`tests/` 配下の統合テスト `<feature>_test.rs`（単数形）と区別する）
- 共有テストヘルパ: `<feature>_test_support.rs`（複数の `*_tests.rs` から `#[path]` 等で共有されるモック/ビルダ/フィクスチャ。テスト関数は置かない）

### 文法定義
- Pest文法: `src/parser/pasta.pest`

### ファイルサイズ方針（俯瞰可能性）

- **目安上限 600 行**: Rust ソース・テストファイルは 1 ファイルあたり概ね 600 行未満を保つ（`oversized-file-decomposition` で全ファイルを俯瞰可能サイズへ振る舞い不変分解済み）。
- **分解の方向**: 肥大化したファイルは、フラットな単一ファイル → 同名のディレクトリモジュール（`mod.rs` + 責務別サブモジュール）へ展開する。例: `debug/dap.rs` → `debug/dap/{mod,codec,decode,encode,pending,resolver}.rs`。
- **不変条件**: 分解はあくまで振る舞い不変（pub API・テスト結果を変えない）。新規実装でこの上限のために責務をまたぐ分割をしない。

## モジュール構成

### ワークスペース構成

```
pasta (workspace)
├── pasta_dsl           # DSLパーサー層（Pest PEG → AST変換）
├── pasta_core          # 言語非依存層（レジストリ）
├── pasta_lua           # Luaバックエンド層（pasta_dsl + pasta_core依存）
├── pasta_shiori        # SHIORI DLLインターフェース層（src/actor/ = アクタースレッド・flume mailbox・marshaling・teardown・static MAILBOX 所有）
├── pasta_lsp           # LSP実装層（WASM/Native対応）
└── pasta_sample_ghost  # サンプルゴースト「hello-pasta」（publish=false）
```

### レイヤー分離原則
各レイヤーは上位レイヤーのみに依存：

**pasta_dsl:**
```
parser（AST生成）
  ↓
error（パースエラー）
```

**pasta_core:**
```
registry（シーン/単語テーブル）
  ↓
error（テーブルエラー）
```

**pasta_lua:**
```
loader (スクリプト読み込み)
  ↓
transpiler (AST→Lua)
  ↓
runtime (Lua VM)
  ↓
pasta_dsl（パーサー） + pasta_core（レジストリ）
```

### 公開API (`pasta_dsl/lib.rs`)
- **Parser**: `parse_str()`, `parse_file()`, AST型（PastaFile, Statement, Expr等）
- **Error**: `ParseError`, `ParseErrorInfo`, `ParseResult`

### 公開API (`pasta_core/lib.rs`)
- **Registry**: `SceneRegistry`, `WordDefRegistry`, `SceneTable`, `WordTable`
- **Random**: `RandomSelector`, `DefaultRandomSelector`
- **Error**: `SceneTableError`, `WordTableError`



## テスト構成

| カテゴリ     | 対象                | ファイル例                       |
| ------------ | ------------------- | -------------------------------- |
| Parser       | 文法パース、エラー  | `span_byte_offset_test.rs`       |
| Transpiler   | Lua変換、シーン管理 | `transpiler_integration_test.rs` |
| Runtime      | Lua VM、シーン解決  | `runtime_e2e_test.rs`            |
| Loader       | スクリプト読み込み  | `loader_integration_test.rs`     |
| Registry     | 型管理、独立性      | `scene_search_test.rs`           |
| Control Flow | Call、最適化        | `transpiler_snapshot_test.rs`    |

### テストファイル配置
- `crates/<crate>/tests/<feature>_test.rs`: 統合テスト
- `crates/<crate>/tests/fixtures/*.pasta`: テスト用スクリプト
- `crates/<crate>/tests/common/`: 共通ユーティリティ

**注**: 旧parser/transpiler実装に依存していたテスト21ファイルは削除済み（2024-12-24 legacy-parser-transpiler-cleanup完了）

## ドキュメント構成

| ファイル   | 用途                                             |
| ---------- | ------------------------------------------------ |
| SOUL.md    | プロジェクトの憲法（ビジョン・コアバリュー）     |
| README.md  | プロジェクト概要                                 |
| GRAMMAR.md | DSL文法リファレンス（人間向け）                  |
| doc/spec/  | 言語仕様書（章別）                               |
| CLAUDE.md  | AI開発支援（プロジェクト指示・Kiro ワークフロー・コマンド一覧） |

### Kiro仕様管理
- `.kiro/steering/`: 規約・原則
- `.kiro/specs/completed/`: 完了仕様アーカイブ
- `.kiro/specs/<name>/`: 進行中仕様

### コードドキュメント
- `///`: 公開APIドキュメント
- `//!`: モジュール概要
- Doctest: 使用例をドキュメント内に記述
