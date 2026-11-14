# Technical Steering

## 技術スタック

### 言語・ランタイム
- **Rust 2024 edition**: メインコンパイラ言語
- **LuaJIT 2.1 (mlua 0.12)**: Luaバックエンドスクリプト実行
- **Pest 2.8.6**: PEGパーサー生成器（`grammar.pest`文法定義）

### ワークスペース構成
- **pasta_dsl**: DSLパーサー層（Pest PEG → AST変換）
- **pasta_core**: 言語非依存層（レジストリ）
- **pasta_lua**: Luaバックエンド層（pasta_dsl + pasta_core依存）
- **pasta_shiori**: SHIORI DLLインターフェース層
- **pasta_lsp**: VSCode 拡張に埋め込む WASM の解析器（色・構文の診断・シーンの並び・ゴースト全体の検査。publish=false）
- **pasta_check**: 辞書の検査の核（ライブラリ・ファイルシステム非依存で WASM でも動く）＋CLIツール（`validate`＝辞書の検査、`release`＝検査を関門にしたゴーストパッケージング・NAR生成）
- **pasta_sample_ghost**: サンプルゴースト「hello-pasta」（publish=false, 配布物作成。立ち絵はコミットした素材）

### 主要依存関係

**pasta_dsl:**
- **pest 2.8.6, pest_derive 2.8.6**: PEGパーサー生成器
- **thiserror 2**: エラー型定義

**pasta_core:**
- **thiserror 2**: エラー型定義
- **rand 0.10**: ランダム選択（重複シーン、前方一致候補）

**pasta_lua:**
- **pasta_dsl**: DSLパーサー層
- **pasta_core**: レジストリ層
- **mlua 0.12**: Lua VMバインディング（LuaJIT 2.1, features: luajit52, vendored, serialize）
- **regex 1.x**: 正規表現（さくらスクリプトタグ検出）
- **thiserror 2**: エラー型定義
- **toml 1.1.2**: pasta の既定の書式。設定ファイル（pasta.toml）・セーブデータ（`@pasta_persistence`）の読み書き、`@toml`（`decode`）、`@pasta_log` の表の 1 行の書き出し
- **serde 1**: TOML の型付きの読み書き（serde_json は DAP とともに `pasta_lua` から外れた。2026-11-03 `debug-mcp-runtime`）
- **glob 0.3**: ファイルパターンマッチ
- **flate2 1.x**: gzip圧縮（キャッシュ等）
- **zip 9.0 (deflate)**: pasta_scripts 自己展開（起動時 zip 解凍）・ビルド時の決定論 zip 生成
- **md5 0.8**: ビルド時の基準ダイジェスト算出（pasta_scripts 自己展開のドリフト検知、build/dev-dependency。ランタイムはマーカー文字列比較のみで md5 不使用）
- **budouy 0.2.2**: 日本語改行位置推定（BudouX、vendored-models）
- **unicode-width 0.2.2**: Unicode文字幅計算
- **unicode-segmentation 1.13.3**: 書記素クラスタの切り出し（トークの文字の単位）
- **tracing 0.1 / tracing-appender 0.2 / tracing-subscriber 0.3**: ロギング・診断
- **windows-sys 0.61**: Windows API（Shift_JISエンコーディング等、cfg(windows)・features: Win32_System_LibraryLoader, Win32_System_Diagnostics_Debug（ffi.load の置き換え）。dev-dependency では Win32_Storage_FileSystem, Win32_System_Registry（8.3 名・長いパスの設定の確かめ））
- **cc 1**（dev-dependency）: FFI テスト用の DLL をテスト中にビルドする
- **luacheck v1.2.0**: 静的解析ツール（scriptlibs/）
- **lua_test**: BDDスタイルテストフレームワーク（scriptlibs/）

**pasta_shiori:**
- **pasta_lua**: 内部依存
- **pest 2.8.6, pest_derive 2.8.6**: SHIORIプロトコルパーサー
- **time 0.3**: 時刻処理
- **tracing 0.1**: ロギング（subscriber/appenderはpasta_luaに移管）
- **thiserror 2**: エラー型定義
- **windows-sys 0.61**: Windows DLL API（cfg(windows)・Win32_System_LibraryLoader。Threading は dev-dependencies だけ）
- **flume 0.12**: アクター mailbox（アクタースレッドは同期 `recv()` のループ・応答待ちは `recv_timeout`・`Sender: Send+Sync+Clone`）
- **arc-swap 1**: 旧来の入口の置き場 `static LEGACY` の lock-free スロット（送信パスに Mutex を置かない）
- **shiori4_abi 0.1**: 版 3 の口（`shiori_get_api`）の ABI の検査（バッファの写し・UTF-8・パニックの捕捉）
- **embed-resource 3**（build-dependency）: pasta.dll に VERSIONINFO（ファイル/製品バージョン）とアイコン（`res/pasta.ico`）を埋め込む（Windows ターゲットのみ）

> **エンジン駆動モデル**: `pasta_lua` の `!Send` Lua VM を、インスタンス（`ActorInstance`）ごとに 1 本のアクタースレッドへ pin し、SHIORI スレッドとは flume mailbox の CH marshaling（GET=block-on-reply／NOTIFY=即204／drop・timeout→204）で通信する。アクタースレッドは OS に頼らない同期 `recv()` のループで回る。入口は 2 つ（旧来の HGLOBAL の `load`・`loadu`・`request`・`unload` は Windows だけ・置き場 `static LEGACY` に 1 つ／shiori4 の版 3 の `shiori_get_api` は全 OS・インスタンスごと）で、同じ芯につながる（`shiori4-v3-portable` で確立）。`unsafe impl Send/Sync` は構造的に撤去済み。さくらスクリプト描画は pasta_lua に物理維持（Lua 集約。`pasta-actor-runtime` で確立）。レンダラを差し替える注入の口（`presentation/`・`RendererInjection`）は、使い手が無いので `ponytail-lua-loader` で撤去した。

**pasta_lsp:**
- **pasta_dsl**: DSLパーサー層
- **pasta_check**（`default-features = false`）: 辞書の検査の核。WASM の入口 `wasm_select_ghost_files`・`wasm_validate_ghost` から呼び、VSCode 拡張のゴースト全体の診断に使う
- **serde 1**（dev: serde_json 1）: シリアライゼーション
- **wasm-bindgen / serde-wasm-bindgen**: WASM の入口（cfg(wasm32)）

**pasta_check:**
- **pasta_dsl / pasta_core**: 検査の核（pasta.dll と同じパーサー・シーン登録）
- **toml 1 / serde 1 / glob 0.3**: 検査の核（pasta.toml の読み取り・入出力の形・`pasta_patterns` の照合）
- **lexopt 0.3**: CLIパーサー（feature `cli`）
- **md5 0.8**: ハッシュ計算（更新ファイル生成・feature `cli`）
- **zip 9.0**: NAR（ZIP）アーカイブ作成（feature `cli`）
- 既定の feature `cli` はバイナリ専用の依存だけを有効にする（ライブラリは WASM でも使うので含めない）。dev-dependency の `pasta_lua`（パス依存・publish で外れる）は pasta.dll との一致テスト用

**pasta_sample_ghost:**
- **png 0.18**（dev-dependency）: 立ち絵の検証テスト（大きさ・透過・表情間の画素一致）

### エディタアプリ pasta-recipe（`editors/recipe/`）
VS Code を使わずに `.pasta` を書く Windows（x64・arm64）のアプリ（arch ごとに exe 1 つ・インストール不要・署名なし）。ルートのワークスペースの外の、別の Cargo ワークスペース（`src-tauri/` が自前の `Cargo.lock` を持つ。ルートの `cargo test --workspace` の対象外）。
- **Tauri 2（~2.12）**: Rust の殻。plugin は `single-instance`（2 つ目の起動を今の窓へ）と `dialog`（フォルダを選ぶ）だけ。画面に fs の許可を与えず、読み書きは Rust のコマンドが開いている `ghost/master` の中だけに限る。外へつながない CSP（開発の操作も Rust の側から送り、画面からは送らない）。WebView2 が無いときはメッセージ箱を出して終了コード 1
- **pasta_lsp / pasta_check（`default-features = false`・feature `fs`）**: パス依存で直接呼ぶ（WASM を通さない）。色・診断・シーンの並び・ゴースト全体の検査は拡張と同じ核から出る。アプリの lock の版はルートの lock と一致させる（`src-tauri/tests` の一致のテスト）
- **notify 8 / toml 1 / windows-sys 0.61**: 見張り（200 ms で束ねる）・最近開いたもの（`%APPDATA%\io.github.ekicyou.pasta-recipe\recent.toml`）・メッセージ箱
- **画面: Vite＋TypeScript（strict）＋Monaco Editor 0.57＋shiki 4**: 拡張の TextMate 文法（読むだけ）で下地を塗り、セマンティックトークンを重ねる。純粋な判断のモジュールは `node --test` で試す
- 版は `tauri.conf.json` から拡張の `package.json` を指す（pasta の版と同じ）。`+crt-static` はルートの `.cargo/config.toml` を受け継ぐ
- **開発の操作**（`pasta-recipe-dev-actions`）: ▶ シーンを実行・値を入れて実行・SHIORI をリロード・保存変数の表示と書き換え。流れは拡張と同じ `editors/shared/`（入出力の面 `GhostIO` を Tauri のコマンドへ写す）。外へ送るのは Rust の `dev_post` だけで、宛先は 127.0.0.1 の候補のポート（9801〜9810・9821〜9830）に固定（代理なし・接続の使い回しなし・送り直しなし・転送に従わない・`Origin` なし）。読むのは進んでいる操作の `ghost/master` の下の決まったファイルだけ。**reqwest 0.13（`default-features = false`）** と `pasta_dsl`（名前の決まり）をパスで使う

### 開発環境
- **tempfile 3**: テスト用一時ファイル生成
- **insta 1.47.2**: スナップショットテスト（glob機能付き）
- **tracing-test 0.2**: テスト用ログキャプチャ

### サプライチェーン監査
- **cargo-audit**: 脆弱性スキャン（RustSec Advisory Database）
- **cargo-deny 0.19**: ライセンス・脆弱性・ソース監査（deny.toml で定義）

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
├── pasta_lsp           # WASM の解析器（VSCode 拡張に埋め込む）
├── pasta_check         # 辞書の検査の核＋CLI（validate・release＝NAR生成・更新ファイル）
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
| pasta_lsp          | Editor       | WASM の解析器（色・構文の診断・シーンの並び・ゴースト全体の検査） |
| pasta_check        | CLI          | 辞書の検査（核は WASM 共有）・リリースパッケージング・NAR生成 |
| pasta_sample_ghost | Distribution | サンプルゴーストの素材・配布    |

### 設計哲学

| 原則         | 内容                                   |
| ------------ | -------------------------------------- |
| UI独立性     | 待ちと同期の制御は宿主（SSP・areka）に任せる。pasta はさくらスクリプトを書くだけ |
| 宣言的フロー | Call/Jumpで制御、if/while/forなし      |
| まとめて返す | 場面の終わりか `＞yield` までの出力を 1 本のさくらスクリプトにまとめて返す（続きはコルーチンが持つ） |
| 2パス変換    | Pass1: シーン登録、Pass2: コード生成（現行の段階構成は[内部設計: トランスパイルパイプライン](https://ekicyou.github.io/pasta/internals/transpiler.html)を参照） |

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
| 検索性能     | シーンO(1)、前方一致は BTreeMap の範囲（O(log n + k)） |
| セキュリティ | Lua VMサンドボックス依存       |

## 依存関係管理

### バージョン戦略
- セマンティックバージョニング準拠
- 依存ライブラリ: メジャーバージョン指定

### ライセンス
- **MIT**
- 依存関係ライセンス: 互換性確認済み

### 公開ポリシー
- `publish = true` in Cargo.toml（`pasta_sample_ghost` と `pasta_lsp` は `publish = false`。`pasta_lsp` は VSCode 拡張に埋め込むだけで crates.io に出さない）
- crates.io へ公開済み
- 公開はリリースタグ `vX.Y.Z` の push で `release.yml` が Trusted Publishing で行う
- 公開クレートの pub API は予告なく消すことがある

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
- **GitHub Actions**: `.github/workflows/build.yml`（Rust/WASM）
  - push/PR/手動実行トリガー＋`workflow_call`（`release.yml` の gate から同じ検査を呼ぶ）
  - **DLLビルド**: マトリックスビルド: x86 (`i686-pc-windows-msvc`) + x64 (`x86_64-pc-windows-msvc`)。x64 の行で `shiori4-testhost`（`shiori4_host` の example）による版 3 の 1 往復
  - **Linux・macOS（`build-unix`）**: ubuntu-latest・macos-latest で `cargo build --release -p pasta_shiori`（`libpasta.so`・`libpasta.dylib`）→ `cargo test --workspace` → clippy（Linux のみ）→ `shiori4-testhost --entry v3` で版 3 の 1 往復。成果物は upload しない（配布は Windows だけ）
  - **WASMビルド**: `pasta_lsp` を `wasm32-unknown-unknown` ターゲットでビルド（10MBサイズ上限チェック付き）
  - Rust キャッシュ: `Swatinem/rust-cache@v2`
  - アーティファクト: `pasta-dll-x86`, `pasta-dll-x64`（7日間保持）
  - **pasta-recipe（`recipe`）**: windows-latest で `editors/recipe` の `npm ci` → `tsc --noEmit` → `npm test` → `npm run build` → `cargo test --locked`・clippy（`-D warnings`）→ リリース CI と同じスクリプトで 2 つの zip を作る（`install-tools.ps1 -Name cargo-about` → 引数なしの `package.ps1`。arm64 は x64 のランナーからのクロス）→ `verify-recipe-zips.ps1`（版は `editors/vscode/package.json`）→ artifact `pasta-recipe-zips`。arm64 のランナー（`windows-11-arm`）では、`recipe-arm64-test`（`needs` なし。`npm run build` → `cargo test --locked`）と `recipe-arm64-smoke`（`needs: recipe`。arm64 の zip の exe を起動し、30 秒のうちに exe の子の `msedgewebview2.exe` が立てば合格、exe が先に終わるか立たなければ失敗）が走る。依存の検査は `cargo deny check (pasta-recipe)` の step（アプリ専用の `editors/recipe/src-tauri/deny.toml`）
  - **拡張の試験（`vscode-test`）**: ubuntu-latest・Node 22 で `editors/vscode` の `npm ci` → `npm test`。`wasm/` を作らないので、本物の WASM を読む 3 本は `PASTA_VSCODE_NO_WASM=1` で理由を出して外す（手元では走る）
  - **マニュアルの軽い検査（`manual-light`）**: `gen-skill-refs --check`・`link-check`・`tutorial-check` を Rust を待たずに走らせる。関門（release.yml の gate）でも走る
- **GitHub Actions**: `.github/workflows/manual.yml`（利用者マニュアル公開・build.yml と独立）
  - `book/**`・`.github/workflows/manual.yml`・`.github/workflows/release.yml`（添付の名前の照合のため）・`.claude/skills/pasta-ghost-authoring/**`・`.claude/skills/pasta-lua-coding/**`・`.claude/skills/pasta-check/**`・`editors/vscode/syntaxes/**`・`crates/*/README.md`・ルートの `README.md`・`doc/SOUL.md`・`crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/**` の変更時に起動。スキル references 鮮度照合（`gen-skill-refs.mjs --check`）→ npm ci(book) → mdbook build → pasta 構文ハイライト → 台詞部品の変換（`talk/talk-html.mjs`）→ bigram 索引再生成 → リンク検証（`link-check.mjs`）→ tutorial-check → verify-static/search（`--self-test` 付き）/content → ツール自己テスト（`book/tools` の `*-test.mjs` 全件）→ 旧版の生成（固定コミット `CLASSIC_REF` の `book/` を旧版のツールで `book/book/classic/` へ。新版の検査の後・失敗で中断。PR でも実行）→ GitHub Pages デプロイ（main のときだけ。旧版は `https://ekicyou.github.io/pasta/classic/`）
  - 権限: ワークフロー全体は `contents: read`。deploy の job だけ `pages: write`・`id-token: write`。Pages 初回は repo Settings で手動有効化が必要
- **GitHub Actions**: `.github/workflows/release.yml`（リリース CI・release-ci）
  - リリースタグ `vX.Y.Z`（数字 3 つ）の push だけで起動。job は verify（タグ・ワークスペース版・`package.json` 版の一致と main からの到達性・タグの版の見どころのファイル `doc/release-notes/vX.Y.Z.md` の有無）→ gate（`build.yml` を `workflow_call`）→ build → publish-crates／publish-vsce → github-release → report
  - **成果物**: build が CI 上でソースから `pasta.dll.zip`（`pasta.dll`＋`THIRD_PARTY_LICENSES.txt`）・`hello-pasta.nar`・`pasta-vscode-X.Y.Z.vsix`・`pasta_check.zip`（x64 の `pasta_check.exe`＋その依存の `THIRD_PARTY_LICENSES.txt`）・`pasta-skills.zip`（配る pasta のスキル 3 つ＋`LICENSE`。生成ファイルの鮮度は関門（`build.yml` の `manual-light`）が照合し、作った後に中身を検査する）・`pasta-recipe.x64.zip`・`pasta-recipe.arm64.zip`（どちらも `pasta-recipe.exe`＋その arch の exe に入るネイティブの部品と画面の部品の `THIRD_PARTY_LICENSES.txt`。`editors/recipe/scripts/package.ps1` が x64 のランナーで `--target` を付けて 2 回作る。`verify-recipe-zips.ps1` が両方を同じ項目で検査し、問題を全部示してから失敗する: 中身が 2 つだけ・exe の版がタグの版・PE の機械の種類（x64 は `0x8664`、arm64 は `0xAA64`）・表示に `tauri`・`monaco-editor` が在る・30 MB 以下・VC++ の再頒布可能パッケージの DLL（`vcruntime*`・`msvcp*` など）を読まない。UCRT は OS の部品なので数えない）の 7 つを作り、artifact `release-assets` で後続 job に渡す。github-release が、前のリリースタグからのコミットの件名から詳しいリリースノート `release-notes-vX.Y.Z.md`（接頭辞を外し、内部の作業を除く）を作り、GitHub Release（題名 `pasta vX.Y.Z`・本文は見どころのファイル `doc/release-notes/vX.Y.Z.md` に、詳しいリリースノートへのリンクと件数の行を足したもの）を作って、7 つの配布物と詳しいリリースノートの 8 つを添付する。見どころの書き方と内部の作業の決まりは `doc/release-notes/README.md`
  - **認証は OIDC**: crates.io は Trusted Publishing、Marketplace は Azure のマネージド ID（フェデレーション資格情報）。長期のトークン（crates.io の API トークン・Marketplace の PAT）は使わず、開発機（`CARGO_REGISTRY_TOKEN`・`VSCE_PAT`）にも GitHub の secret にも置かない。crates.io の短命トークンは job の中で auth の直後に発行し、直後の publish step だけに渡す。crates.io・Marketplace の公開 job（publish-crates・publish-vsce）は environment `release` で走る
  - 公開先ごとに「公開済みなら飛ばす」冪等な処理で、失敗した job の再実行で続きから回復する。job の補助スクリプトは `.github/scripts/release/*.ps1`（pwsh 7）
  - 一回限りのセットアップ手順は `.github/release-ci-setup.md`、その確認は `.github/workflows/release-setup-check.yml`（手動起動・公開しない）。人が読むリリース手順書は `crates/pasta_sample_ghost/RELEASE.md`。エージェントが実行する手順（版の決定 → 版の更新の PR → リリースタグの push → リリース CI の結果の確認）は `release` スキル（`/release [X.Y.Z]`）にあり、手順を直すときはスキルを直す
  - 手元の `release.ps1`・`npm run package` の成果物は動作確認用で、コミットしない（`release/` とサンプルゴーストの生成物は `.gitignore` 済み）。`Cargo.lock` はコミットする

## 利用者マニュアル（ドキュメントサイト `book/`）

ゴースト作者向けマニュアルは、コードとは独立した **mdBook 静的サイト**として `book/` に構築し GitHub Pages へ公開する（[https://ekicyou.github.io/pasta/](https://ekicyou.github.io/pasta/)）。サーバー不要・`file://` オフライン閲覧両立。

- **mdBook v0.5.4**: Markdown → 静的 HTML/CSS/JS（CI は公式の配布物を取る）。生成物 `book/book/` は `.gitignore` 済み（CI で再生成）
- **日本語 bigram 検索**: build-time Node スクリプト（`book/tools/bigram-index/`）が mdBook 同梱 elasticlunr 索引を 2-gram で再生成。索引ビルダとクエリ側（`theme/head.hbs`）が単一 `tokenize.mjs` を共有（不一致は検索破綻）
- **スキル references 生成**: `book/tools/gen-skill-refs.mjs` が対応表（`GENERATION_MAP`）に従い `book/src/` の章から 2 スキル（`pasta-ghost-authoring`・`pasta-lua-coding`）の `references/` 生成ファイルを書き出す（先頭行に生成マーカー・手編集禁止）。`--check` は書き込まずに鮮度を照合し、古ければ exit 1（**改行 LF 正規化**で比較）
- **リンク検証**: `book/tools/link-check.mjs` がマニュアル章のリンク（章のファイルと見出し `#…` の実在）、3 スキル（`pasta-check` を含む）の相対リンク・アンカー（内部設計の章が引くリポジトリのパスは `editors/` も見る）、読者に届く文書（クレートの README・ルートの `README.md`・`doc/SOUL.md`・3 スキル）にある公開マニュアルの URL と、マニュアルの本文と読者に届く文書の最新版の固定リンク（`releases/latest/download/<名前>`）の名前がリリース CI（`release.yml`）の添付の名前にあることを検証する。見出しの `id` が mdBook の出力と合うことは、自己テスト（`link-check-test.mjs`）が CI でビルドの後に全章を突き合わせる。鮮度照合とあわせて `workflow.md` DoD の条件付き「Manual Sync Gate」と結線し、失敗で完了を中断
- **pasta 構文ハイライト**: build-time Node スクリプト（`book/tools/highlight/`）が VSCode TextMate 文法（`editors/vscode` の SSOT・読み取り再利用）で `language-pasta` ブロックをトークナイズし highlight.js 互換クラスの span を静的 HTML へ焼き込む（決定論・冪等・入れ子 lua は vendor 文法で二段トークナイズ）。`theme/head.hbs` が book.js の無条件再ハイライトを中和し事前 span を保持（正準 `neutralizer.mjs` の逐語ミラー）。`book/` 初の npm devDependency（`vscode-textmate`/`vscode-oniguruma`/`jsdom`・`package-lock.json` コミット・`node_modules` 非コミット・CI は `npm ci`）。WASM は build-time のみで公開成果物にランタイム依存を持ち込まない
- **正の分離**: 利用者向け仕様（文法・公開 Lua API・`pasta.toml`）の唯一の権威はマニュアル（`book/src/`）。スキルの規範部分はマニュアルからの生成物、README は開発者向けの入口
- **内部設計パート**（`book/src/internals/`）: コントリビュータ向け・権威は実装＋本パート

## Luaランタイムパターン

### scripts/ 優先順位
- `scripts/`（ゴーストカスタム）> `profile/pasta/pasta_scripts/`（エンジン標準ランタイム、pasta.dll が起動時に自己展開）
- フレームワークスクリプトは pasta.dll 内蔵 zip から起動時に `profile/pasta/pasta_scripts/` へ自己展開される（`ghost/master/pasta_scripts/` 同梱は廃止）。`.md5` マーカーで版を比較し不一致時のみ準アトミックに再展開。`profile/` 配下のためネットワーク更新（`updates.txt`/`.nar`）の対象外
- `scripts/main.lua` がエントリーポイント。同名ファイルでエンジン動作を上書き可能（検索パスで `scripts` が `profile/pasta/pasta_scripts` より上位）

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
| `@pasta_debug`         | デバッグの有効と、位置からのシーンの引き当て・保存変数の書き出し（内部。`OnPastaScene`・`OnPastaSceneAt`・`OnPastaDumpVariables` の既定ハンドラーが使う） | 本番の入口だけが登録 |
| `@toml`                | TOML の文字列を表に読む（`decode`）     |                            |

### シーン関数定型パターン

```lua
function SCENE.func_name(act)
    local save, var = act:init_scene(SCENE)  -- 必須
    act:talk(act.アクター名.actor, "セリフ")
end
```

### SHIORIハンドラ登録パターン

```lua
REG.OnBoot = function(act)
    return "\\0\\s[0]起動しました。\\e"  -- Value にする文字列を返す（RES で作った応答全体を返すと、包まずにそのまま応答になる）
end
```

### 非同期コールバック（SHIORI通信基盤）

トーク合成中にSSPプロパティを非同期取得するパターン。コルーチン内でのみ使用可能。

```lua
-- act:get_property() はyieldして結果を待つ
local value = act:get_property("baseware.version")
```

- `pasta.shiori.event.callback` モジュールが ID 管理・待機の登録・タイムアウトを担当し、再開は通常のイベントと同じ `EVENT.drive` に任せる
- 応答イベント（`OnPastaCallBack{N}`）で待機中のシーンを再開し、出力が応答になる
- OnSecondChange の既定ハンドラが期限切れの待機を掃引する（理由付きのタイムアウトは `【時間切れ：…】` の表記を `200 OK` ＋ `X-Error-Reason` で返す。`[logging] balloon_warning = false` なら従来どおり `X-Error-Reason` 付きの 500）

詳細: `.claude/skills/pasta-lua-coding/SKILL.md`
