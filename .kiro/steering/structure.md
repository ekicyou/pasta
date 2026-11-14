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
│   │       ├── error.rs     # ParseError
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
│   │       ├── actor_font.rs # ActorFont - アクターごとの表示の装い（pasta.toml の [actor."名前"].font）からタグ列を作る
│   │       ├── error.rs     # SceneTableError, WordTableError
│   │       └── registry/    # 型管理レイヤー（独立）
│   │           ├── mod.rs   # Registry API
│   │           ├── scene_registry.rs  # SceneRegistry - シーン管理
│   │           ├── scene_alias.rs     # SceneAliasTable - シーン名の別名表（完全一致 1 段）
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
│       │   │   ├── expr_gen.rs     # 式生成（分割impl。値・関数呼び出し・引数・算術/連結の連鎖の組み直し）
│       │   │   └── source_map.rs   # 記録のシーム（SourceMapSink。行の記録とシーンの宣言）
│       │   ├── debug/      # 開発用のイベントの部品（`[debug] enabled` のときだけ働く・SHIORI非依存。DAP は 2026-11-03 に撤去）
│       │   │   ├── mod.rs          # 入口（resolve_enabled・source_map の再輸出）
│       │   │   ├── config.rs       # 有効の解決（PASTA_DEBUG ＞ [debug] enabled ＞ false）
│       │   │   ├── playscene.rs    # OnPastaSceneAt の位置の引き（@pasta_debug.scene_at）
│       │   │   └── source_map/     # シーンマップ（mod/scene_index/scene_join。シーンの宣言の記録と索引）
│       │   │   # 注: 対応する src 内テスト（*_tests.rs）を併置（後述「src/ 内テスト配置方針」）
│       │   ├── context.rs   # トランスパイルコンテキスト
│       │   ├── error.rs     # エラー型
│       │   ├── encoding/    # エンコーディング処理（プラットフォーム別分割）
│       │   │   ├── mod.rs           # エンコーディングAPI
│       │   │   ├── windows.rs       # Windows固有（Shift_JIS等）
│       │   │   └── unix.rs          # Unix固有
│       │   ├── loader/      # スクリプトローダー（ディレクトリモジュール）
│       │   │   ├── mod.rs           # ローダーAPI
│       │   │   ├── cache.rs         # キャッシュ管理
│       │   │   ├── config/          # ローダー設定（mod.rs・sections.rs・actor_font.rs。pasta.toml の [scene.alias]・[actor."名前"].font の読み込みを含む）
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
│       │   │   ├── ffi_load/           # ffi.load の置き換え（Windows のときだけ）
│       │   │   │   ├── mod.rs          # 設置・Lua のラッパ・理由の文言
│       │   │   │   ├── resolve.rs      # 名前を読むパスへ解く純粋な規則
│       │   │   │   └── win32.rs        # unsafe の層（DLL の先読み・同名の確かめ・解放）
│       │   │   └── log.rs              # ランタイムログ
│       │   ├── sakura_script/ # さくらスクリプト処理
│       │   │   ├── mod.rs           # さくらスクリプトAPI
│       │   │   ├── line_breaker.rs  # BudouX による自動改行
│       │   │   ├── tokenizer.rs     # トークナイザー
│       │   │   └── wait_inserter.rs # ウェイト挿入
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
│           ├── shiori/              # SHIORI関連テスト（7 files）
│           │   ├── main.rs
│           │   ├── event_dispatch_test.rs
│           │   ├── event_handler_test.rs
│           │   ├── mouse_event_test.rs
│           │   ├── res_test.rs
│           │   ├── virtual_event_config_resolution_test.rs
│           │   ├── virtual_event_dispatch_test.rs
│           │   └── virtual_event_status_skip_test.rs
│           ├── runtime/             # ランタイム関連テスト（主なもの。全 21 files）
│           │   ├── main.rs
│           │   ├── finalize_scene_test.rs
│           │   ├── scene_test.rs
│           │   ├── syntax_test.rs
│           │   ├── unit_test.rs
│           │   ├── persistence_integration_test.rs
│           │   ├── encoding_test.rs
│           │   └── toml_module_test.rs
│           ├── log/                 # ログ関連テスト（3 files）
│           │   ├── main.rs
│           │   ├── integration_test.rs
│           │   ├── module_test.rs
│           │   └── stack_level_test.rs
│           ├── sakura_script/       # SakuraScript関連テスト（7 files）
│           │   ├── main.rs
│           │   ├── backslash_break_test.rs
│           │   ├── basic_test.rs
│           │   ├── budoux_test.rs
│           │   ├── conformance_test.rs
│           │   ├── edge_case_test.rs
│           │   ├── grapheme_test.rs  # 書記素クラスタ単位のウェイト・改行・幅
│           │   └── output_test.rs
│           ├── search/              # 検索関連テスト（2 files）
│           │   ├── main.rs
│           │   ├── scene_search_test.rs
│           │   └── module_test.rs
│           ├── ffi/                 # ffi.load の置き換えのテスト（テスト中に cc で DLL をビルド）
│           │   ├── main.rs
│           │   ├── dll_fixture.rs   # テスト用 DLL のビルド
│           │   ├── load_test.rs
│           │   ├── config_test.rs
│           │   ├── collision_test.rs
│           │   ├── lifetime_test.rs
│           │   └── compare_test.rs
│           ├── common/              # テスト共通ユーティリティ
│           │   ├── mod.rs
│           │   └── e2e_helpers.rs
│           ├── fixtures/            # テスト用Pastaスクリプト
│           ├── lua_specs/           # Lua単体テスト仕様
│           ├── lua_unittest_runner.rs  # Lua単体テストランナー（命名例外）
│           ├── japanese_identifier_test.rs  # Lua基盤テスト
│           └── ucid_test.rs         # Lua基盤テスト
│   ├── pasta_lsp/           # WASM の解析器（VSCode 拡張に埋め込む・publish=false）
│   │   ├── Cargo.toml       # pasta_lsp設定（pasta_dsl, pasta_check（既定 feature 無し）, serde, wasm-bindgen 依存）
│   │   ├── README.md        # クレート概要
│   │   ├── tests/           # 外部化テスト（Phase A）
│   │   └── src/
│   │       ├── lib.rs       # クレートエントリーポイント
│   │       ├── analysis/    # 解析エンジン（ディレクトリモジュール）
│   │       │   ├── mod.rs          # 解析API・全公開型 re-export
│   │       │   ├── token_types.rs  # トークン型定義
│   │       │   ├── scenes.rs       # シーンの並び（グローバル ＞ 名前のあるローカル・見出しと範囲。着色と同じ構文木から作る）
│   │       │   ├── visit_scope.rs  # スコープ走査ビジター（旧 visitors.rs を分解）
│   │       │   ├── visit_action.rs # アクション走査ビジター
│   │       │   ├── visit_expr.rs   # 式走査ビジター
│   │       │   └── text_utils.rs   # テキストユーティリティ
│   │       └── transport.rs # WASM の入口 3 つ（解析・辞書ファイルの選び出し・ゴースト全体の検査）
│   ├── pasta_check/         # 辞書の検査の核（ライブラリ）＋CLIツール（validate・release）
│   │   ├── Cargo.toml       # pasta_check設定（核: pasta_dsl, pasta_core, toml, serde, glob／feature cli: lexopt, md5, zip）
│   │   ├── tests/           # CLI統合テスト・検査の規則のテスト・pasta_lua との一致テスト（validate_parity_test.rs）
│   │   └── src/
│   │       ├── lib.rs           # ライブラリの入口（validate を公開）
│   │       ├── validate/        # 検査の核（ファイルシステム非依存・WASM 共有。config/discovery/syntax/transpile_rules/module_name/scenes/builtins/references/report）
│   │       ├── main.rs          # CLIエントリーポイント
│   │       ├── validate_cmd.rs  # validate サブコマンド（ghost/master のファイルを集めて核を呼ぶ・release の段 4 も使う）
│   │       ├── release.rs       # リリースビルドオーケストレーション（6 段・段 4 が辞書の検査の関門）
│   │       ├── balloon.rs       # 同梱バルーン判定（install.txt）
│   │       ├── update_files.rs  # 更新ファイル（updates.txt）生成
│   │       ├── nar.rs           # NAR（ZIP）アーカイブ作成
│   │       └── copy.rs          # ファイルコピーユーティリティ
│   └── pasta_sample_ghost/  # サンプルゴースト「hello-pasta」（publish=false）
│       ├── Cargo.toml       # テスト用の依存だけ（絵を描くプログラムは持たない）
│       ├── README.md        # クレート概要
│       ├── RELEASE.md       # リリース手順
│       ├── STAGES.md        # 入門ガイドの段階表（段階 N の辞書 = dic/01〜NN。.nar には入らない）
│       ├── release.bat      # release.ps1 のバッチラッパー（ダブルクリック用）
│       ├── release.ps1      # ビルド＋配布パッケージ生成スクリプト（8 段。段 3 で x64 の pasta_check.zip を作り、段 4 でその exe の release を走らせ、段 6 で pasta-skills.zip を作る。絵を生成する段は無い。手元では動作確認用・リリース CI の build job も呼ぶ）
│       ├── art/             # 立ち絵の生成の記録（README.md）と設定画像 2 枚（.nar には入らない）
│       ├── src/
│       │   ├── lib.rs              # テスト用モジュールの宣言だけ
│       │   └── scripts.rs          # ghosts/hello-pasta 辞書(.pasta)の検証テスト
│       ├── ghosts/           # サンプルゴースト本体（SSOT・配布物一式）
│       │   └── hello-pasta/  # 手書きSSOT(descript/pasta.toml/dic/NN-*.pasta 12 段/install/shell/master の立ち絵 18 枚・surfaces.txt)＋生成物(dll/ライセンス表示/scripts は .gitignore 済み)
│       └── tests/            # 統合テスト・配布ファイル構成検証・全段階の検証（tutorial_stages_test.rs）・立ち絵と当たり判定の検証（shell_assets_test.rs）
├── editors/                  # エディタ拡張
│   ├── shared/              # 拡張とアプリの共有の実行の流れ（vscode・Node・Tauri を読み込まない・"type": "module"）。ghostIO（入出力の面の型）・mcpClient・ghostFiles・reloadShiori・runSceneAtCursor・dumpVariables・devSession（相手の覚えと選び・重なりの止め・見出し・保存の手順）と、偽の入出力 fakeGhostIO による *.test.ts（アプリの npm test が拾う）
│   ├── recipe/              # pasta-recipe（VS Code を使わずに書く Windows（x64・arm64）のアプリ・Tauri 2 ＋ Vite ＋ Monaco。別の Cargo ワークスペース）
│   │   ├── src/             # 画面（TypeScript）。純粋な判断（mapping・sync・tabs・scheduler・theme）は *.test.ts を node --test で
│   │   ├── src-tauri/       # Rust の殻（自前の Cargo.lock・deny.toml・about.toml）。paths（ghost/master の探し方と範囲の門・開発の読み取りの門 dev_path）・files・recent・check・watch・commands・dev（127.0.0.1 の候補のポートへだけ送る POST と、進んでいる操作の master の下だけを読む門）・variables（保存変数の表の行・ローカル変数のインライン表）
│   │   │   └── tests/       # repo_parity_test.rs（lock の版・凡例・版の置き場がルートや拡張と一致）
│   │   ├── scripts/         # package.ps1（`-Arch x64,arm64`。release/pasta-recipe.x64.zip・release/pasta-recipe.arm64.zip を作る。引数なしで両方）・js-licenses.mjs（画面の部品の第三者の表示）
│   │   └── tauri.conf.json  # 版は ../vscode/package.json を指す・CSP・窓
│   └── vscode/              # VSCode拡張（TypeScript + WASM統合）
│       ├── package.json     # 拡張マニフェスト
│       ├── tsconfig.json    # TypeScript設定
│       ├── src/             # TypeScriptソース
│       │   ├── extension.ts            # アクティベーション・エントリポイント
│       │   ├── wasmBridge.ts           # WASMモジュールブリッジ（解析・ゴースト全体の検査の入口と型）
│       │   ├── ghostValidator.ts       # ゴースト全体の検査の診断（保存・ファイルの増減で起動・300ms で束ねる・source "pasta validate"）
│       │   ├── semanticTokensProvider.ts # セマンティックトークン提供
│       │   ├── sceneOutline.ts         # シーンの並びをアウトライン（DocumentSymbol）と見出しの上の「▶ シーンを実行」（CodeLens）に写す（WASM が読めたときだけ登録）
│       │   ├── diagnosticsManager.ts   # 診断情報管理
│       │   ├── documentSync.ts         # ドキュメント同期（200msデバウンス）
│       │   ├── devCommands.ts          # 開発の 3 コマンド（▶ シーンを実行・SHIORI をリロード・保存変数を表示）の入口（引数 (uri, 0 始まりの行) でも呼べる）。相手の確かめと覚え・保存・出力のパネル「Pasta 開発」
│       │   ├── nodeGhostIO.ts          # 共有の入出力の面（GhostIO）の Node の実装（HTTP・fs・path）と、ファイルから上ってゴーストを見つける resolveGhost
│       │   └── test/                   # テスト（mock/vscode.ts と MCP の窓口を真似る fakeMcpServer.ts）
│       ├── syntaxes/         # TextMate文法（全角/半角マーカー対応）
│       ├── scripts/          # ビルドスクリプト（WASM等）
│       └── wasm/             # pasta_lsp WASMバイナリ（ビルド生成物）
├── book/                     # 利用者マニュアル（mdBook・GitHub Pages 公開）
│   ├── book.toml            # mdBook 設定（language=ja, 検索有効, site-url）
│   ├── AUTHORING.md         # マニュアルの執筆規約（第 8 節は入門ガイドの章の決まり・画像の行）
│   ├── SCREENSHOTS.md       # 入門ガイドの絵の撮影の手順の記録（撮る環境・16 枚の一覧・後片付け・撮り直す決まり。手作業で撮る）
│   ├── package.json         # book ツールの npm 依存（vscode-textmate/oniguruma/jsdom・lockfile コミット・node_modules 非コミット）
│   ├── src/                 # 章ソース（grammar/lua/getting-started/reference/debug）
│   │   ├── img/claudia/     # 台詞部品の顔アイコン 16 枚＋LICENSE.txt（ponapalt/claudia・Unlicense）
│   │   ├── img/getting-started/ # 入門ガイドの絵 16 枚（実機の SSP で写した PNG。撮り方は SCREENSHOTS.md）
│   │   └── internals/       # 内部設計パート（コントリビュータ向け・末尾パート）
│   ├── theme/               # head.hbs（日本語 bigram 検索 tokenizer ＋ pasta ハイライト中和の override ＋ Web フォント ＋ 中間の幅で目次を描く前に開き直す claudiaSidebarEarly）・claudia.css（Claudia テーマ: light/navy の配色・部品・台詞・扉・狭幅・印刷）・claudia.js（テーマメニューを Auto/Light/Navy に絞る・中間の幅で目次の開閉を次のページへ持ち越す）
│   ├── tools/               # build-time Node（スキル references 生成 gen-skill-refs・リンク検証 link-check・bigram 索引再生成・pasta 構文ハイライト・台詞部品 talk/（話し手と表情の登録簿 talk.mjs・ビルド後の吹き出し変換 talk-html.mjs）・検証 verify-*・テーマの対比テスト theme-test 等）
│   └── book/                # mdbook build 生成物（.gitignore 済み・CI で再生成）
├── doc/                      # 開発者向けの文書（利用者向けはマニュアル book/）
│   ├── SOUL.md              # プロジェクトの憲法（ビジョン・コアバリュー）
│   ├── TEST_COVERAGE.md     # コア機能とテストの対応台帳
│   └── release-notes/       # 版ごとのリリースノートの見どころ vX.Y.Z.md（GitHub Release の本文・版の更新と同じコミットで書く）と、書き方・内部の作業の決まり README.md
├── .kiro/                    # Kiro Spec-Driven設定
│   ├── steering/            # ステアリング規約
│   ├── settings/            # テンプレート・ルール
│   └── specs/               # 仕様管理
│       ├── completed/       # 完了仕様（アーカイブ）
│       └── <spec-name>/     # 進行中仕様
├── .vscode/                 # VS Code 設定
├── .github/                 # GitHub Actions
│   ├── workflows/
│   │   ├── build.yml        # Rust/WASM ビルド・検査、pasta-recipe の 2 つの zip の作成と検査・arm64 のランナーの試験と煙試験（push/PR・release.yml から workflow_call）
│   │   ├── manual.yml       # 利用者マニュアルの検査と GitHub Pages 公開
│   │   ├── release.yml      # リリース CI（vX.Y.Z タグ契機・OIDC で crates.io/Marketplace へ公開・GitHub Release 作成）
│   │   └── release-setup-check.yml # リリース CI のセットアップ確認（手動起動・公開しない）
│   ├── scripts/release/     # リリース CI の job 補助スクリプト（verify-tag/publish-crate/publish-vsix/release-notes/github-release の .ps1。pasta-recipe の 2 つの zip の検査 verify-recipe-zips.ps1（main の CI の job recipe とリリース CI が呼ぶ）・ツールの導入 install-tools.ps1）と、その検査 release-scripts-test.ps1（build.yml の release-scripts job で実行）
│   └── release-ci-setup.md  # リリース CI の一回限りのセットアップ手順書
├── img/                     # README のロゴ・スクリーンショット
├── release/                 # release.ps1 と editors/recipe/scripts/package.ps1 の出力先（pasta_check.zip・.nar・pasta.dll.zip・pasta-skills.zip・pasta-recipe.x64.zip・pasta-recipe.arm64.zip。.gitignore 済み・コミットしない）
├── Cargo.lock               # 依存の解決結果（コミット対象・リリース CI の build で不変を検査）
├── README.md                # プロジェクト概要
├── LICENSE                  # ライセンス
└── CLAUDE.md                # AI開発支援（プロジェクト指示・Kiro ワークフロー・コマンド一覧）
```

### ルートに置いてよいもの

ルートは Rust のワークスペースとして最小限に保つ。置いてよいファイルは次だけ。

- Cargo が読むもの: `Cargo.toml`・`Cargo.lock`
- 道具がルートに読みに来るもの: `deny.toml`（cargo-deny）・`about.toml`・`about.hbs`・`about-pasta_check.hbs`（cargo-about）・`CLAUDE.md`（Claude Code・cc-sdd の更新でルートに書かれる）・`.gitignore`
- リポジトリの顔: `README.md`・`LICENSE`

それ以外の文書は `doc/`（開発者向け）か `book/`（利用者向け）、スクリプトは使うクレートの中（例: `crates/pasta_sample_ghost/release.bat`）に置く。移設の案内だけのファイル（「〜へ移りました」）はルートに残さず消す。

**注**:
- ルートクレート (`src/`) は削除済み。すべての実装コードは `crates/*/src/` 配下に配置。
- ルートレベルの `tests/` と `examples/` も削除済み（Pure Virtual Workspace移行完了）
- 各クレートは独自の `tests/` ディレクトリを持つことができる（例: pasta_dsl, pasta_core, pasta_lua, pasta_lsp, pasta_sample_ghost, pasta_shiori）
- pasta_sample_ghost のサンプルゴースト hello-pasta は `ghosts/hello-pasta/` に完全な一式として直接配置（SSOT）。テキスト系（descript/pasta.toml/dic/install）とシェルの画像は追跡する正本、DLL・ライセンス表示・`scripts/` は生成物。`release.ps1` は生成物の配置と `pasta_check.zip`・`.nar`・`pasta.dll.zip`・`pasta-skills.zip` のパッケージングを担う（旧 dist-src/robocopy 方式は廃止）。生成物のうち DLL・ライセンス表示・`scripts/` と `release/` はコミットしない。配布物の公開はリリースタグの push で `release.yml` が行う
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
- `pasta_lua/src/debug/` 配下（`playscene_tests.rs`・`source_map/scene_index_tests.rs`・`scene_join_tests.rs`。シーンマップの内部状態を直接検証）

**命名規約（src 内テスト）**:
- テストモジュール本体: `<feature>_tests.rs`（複数形 `_tests`。`tests/` 配下の統合テスト `<feature>_test.rs`（単数形）と区別する）
- 共有テストヘルパ: `<feature>_test_support.rs`（複数の `*_tests.rs` から `#[path]` 等で共有されるモック/ビルダ/フィクスチャ。テスト関数は置かない）

### 文法定義
- Pest文法: `crates/pasta_dsl/src/parser/grammar.pest`

### ファイルサイズ方針（俯瞰可能性）

- **目安上限 600 行**: Rust ソース・テストファイルは 1 ファイルあたり概ね 600 行未満を保つ（`oversized-file-decomposition` で全ファイルを俯瞰可能サイズへ振る舞い不変分解した。2026-10-24 の時点で、1,000 行を超えるのは `pasta_lua` の `sakura_script/line_breaker.rs`（1,155 行。本体は約 310 行で、残りはテスト）と `search/context.rs`（1,017 行。本体は約 437 行で、残りはテスト）、`pasta_check` の `balloon.rs`（1,070 行。後半はテスト）の 3 つ。600 行を超えるファイルは 30 近くある）。
- **分解の方向**: 肥大化したファイルは、フラットな単一ファイル → 同名のディレクトリモジュール（`mod.rs` + 責務別サブモジュール）へ展開する。例: `debug/dap.rs` → `debug/dap/{mod,codec,decode,encode,pending,resolver}.rs`（DAP は 2026-11-03 に撤去した。分け方の例として残す）。
- **不変条件**: 分解はあくまで振る舞い不変（pub API・テスト結果を変えない）。新規実装でこの上限のために責務をまたぐ分割をしない。

## モジュール構成

### ワークスペース構成

```
pasta (workspace)
├── pasta_dsl           # DSLパーサー層（Pest PEG → AST変換）
├── pasta_core          # 言語非依存層（レジストリ）
├── pasta_lua           # Luaバックエンド層（pasta_dsl + pasta_core依存）
├── pasta_shiori        # SHIORI DLLインターフェース層（旧来の入口 windows.rs〔Windows だけ〕と版 3 の入口 v3.rs〔全 OS〕・src/actor/ = インスタンスごとのアクタースレッド・flume mailbox・marshaling・teardown・pasta.dll の参照 module_ref・旧来の置き場 static LEGACY）
├── pasta_lsp           # WASM の解析器（色・構文の診断・シーンの並び・pasta_check の検査の核を WASM で公開）
├── pasta_check         # 辞書の検査の核（ライブラリ・pasta_dsl + pasta_core依存）＋CLI（validate・release）
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
- **Error**: `ParseError`

### 公開API (`pasta_core/lib.rs`)
- **Registry**: `SceneRegistry`, `WordDefRegistry`, `SceneTable`, `WordTable`
- **ActorFont**: `ActorFont`, `FontInput`, `FontValue`, `FONT_ATTRIBUTES`（アクターごとの表示の装い）
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
| SakuraScript | 書記素クラスタ単位のウェイト・改行 | `sakura_script/grapheme_test.rs` |

### テストファイル配置
- `crates/<crate>/tests/<feature>_test.rs`: 統合テスト
- `crates/<crate>/tests/fixtures/*.pasta`: テスト用スクリプト
- `crates/<crate>/tests/common/`: 共通ユーティリティ

**注**: 旧parser/transpiler実装に依存していたテスト21ファイルは削除済み（2024-12-24 legacy-parser-transpiler-cleanup完了）

## ドキュメント構成

| ファイル   | 用途                                             |
| ---------- | ------------------------------------------------ |
| doc/SOUL.md | プロジェクトの憲法（ビジョン・コアバリュー）    |
| doc/TEST_COVERAGE.md | コア機能とテストの対応台帳            |
| README.md  | プロジェクト概要                                 |
| book/src/  | 利用者マニュアル章（文法・公開 Lua API・`pasta.toml` の唯一の権威） |
| book/src/internals/ | 内部設計パート（コントリビュータ向け・権威は実装＋本パート） |
| CLAUDE.md  | AI開発支援（プロジェクト指示・Kiro ワークフロー・コマンド一覧） |

### Kiro仕様管理
- `.kiro/steering/`: 規約・原則
- `.kiro/specs/completed/`: 完了仕様アーカイブ
- `.kiro/specs/<name>/`: 進行中仕様

### コードドキュメント
- `///`: 公開APIドキュメント
- `//!`: モジュール概要
- Doctest: 使用例をドキュメント内に記述
