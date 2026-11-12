# Brief: pasta-recipe

> **ステータス**: 未着手（2026-11-12 の `/kiro-discovery` で起票。開発者の依頼「pasta DSL スタンドアロンアプリ（仮名）pasta-recipe（VS Code 非依存）。次のリリースにする。最優先事項」）。版は 0.5.3（開発者の承認）。要件定義は Opus（Fable なし）。「pasta-recipe」は仮の名前で、spec 名・exe 名に使う。正式な名前は要件で決める。

## Problem

pasta DSL の編集支援は、今は VS Code 拡張 `ekicyou.pasta-vscode`（v0.5.2）だけが持っている。ゴーストの作者の中には「VS Code を入れたくない」人がいて、その人たちは色も診断も無いテキストエディタで `.pasta` を書いている。書き間違いは SSP で読み込むまで分からない。

## Current State

2026-11-12（main `5c27b869`）に確かめた。

- **解析**: `crates/pasta_lsp` の `AnalysisEngine::analyze` がセマンティックトークンと診断を作る（`transport.rs` の `wasm_analyze` が WASM の入口）。ゴースト全体の検査は `pasta_check` のライブラリ（`default-features = false` ならファイルシステムに触らない）で、`wasm_select_ghost_files`・`wasm_validate_ghost` が入口。拡張はこれを `wasm-pack --target nodejs` で WASM にして、Node の `require` で読んでいる（`editors/vscode/scripts/build-wasm.ps1`・`src/wasmBridge.ts`）。
- **解析結果の形**: `{tokens:[{deltaLine,deltaStartCharacter,length,tokenType,tokenModifiers}], diagnostics:[{range,message,severity}]}`。位置は UTF-16・0 始まり（`transport.rs` のテストが形を固定）。凡例（トークンの種類 17・修飾 3）は TS の側（`semanticTokensProvider.ts` 18〜45 行）にある。
- **着色**: TextMate 文法 `editors/vscode/syntaxes/pasta.tmLanguage.json`。コードブロック（3 個以上のバッククォートの柵）の中身は `pasta-lua-injection.tmLanguage.json` が VS Code 同梱の `source.lua` を差し込んで色を付ける。マニュアル（`book/tools/highlight/`）は同じ文法を `vscode-textmate`＋`vscode-oniguruma` で読んでいる（VS Code の外で文法を使う先例）。
- **シーンの並び**: 今の解析結果には無い。0.5.3 で先に入る `vscode-scene-outline-play` が、`AnalysisResult` に `scenes`（名前・種類・見出しの行・範囲・子）を足す。
- **Lua のファイル**: ランタイムはゴーストの `ghost/master/scripts/` を同梱の `pasta_scripts/` より先に探す。Lua のパーサーや lint はリポジトリに無い（luacheck は CI 専用の同梱）。

## Desired Outcome

- `pasta-recipe.exe` 1 つ（インストーラ無し）を置けば動く。ゴーストのフォルダを exe にドラッグドロップすると、そのゴーストの `ghost/master` を開く。
- 左のツリーに `ghost/master` のフォルダとファイルが出て、`.pasta` のファイルの下にグローバルシーン ＞ ローカルシーン が並ぶ。シーンを押すとその見出しへ飛ぶ。
- `.pasta` を開くと、VS Code 拡張と同じ色（セマンティックトークン）と同じ診断（波線と一覧）が出る。コードブロックの中の Lua にも色が付く。
- `.lua` のファイルも開いて編集・保存できる（色付けまで）。
- ゴースト全体の検査の結果が、VS Code 拡張と同じものとして一覧に出る。
- 最近開いたゴーストのフォルダが残り、次の起動で選び直せる。

## Approach

**Tauri 2（2.12.x に固定）＋ Vite ＋ Monaco Editor 0.57。解析は Rust の側で `pasta_lsp`・`pasta_check` をライブラリとして直接呼ぶ**（discovery で開発者が承認した推奨案）。

1. **単一の exe**: `tauri build --no-bundle` で、フロントの資産を埋め込んだ exe 1 つを作る。WebView2 は OS のもの（Windows 10 の 1803 以降・Windows 11 は同梱）を使い、同梱しない。`pasta_lsp`・`pasta_check` は exe に静的にリンクされる（WASM は通らない）。
2. **解析は Rust のコマンド**: フロントは `invoke('analyze', { source })` を呼び、Rust の側が `AnalysisEngine::analyze` の結果を、今の WASM と同じ形の JSON で返す。ゴースト全体の検査も Rust の側でファイルを読み、`pasta_check` を呼ぶ。凡例は拡張と食い違わないように 1 か所から取る（置き場所は要件・設計で決める）。
3. **着色の 2 段**: 下地は `@shikijs/monaco` で今の `pasta.tmLanguage.json` を読み、柵の中は shiki 同梱の Lua 文法を差し込む。その上に Monaco の `registerDocumentSemanticTokensProvider`（`'semanticHighlighting.enabled': true`）で `pasta_lsp` のトークンを重ねる。診断は `monaco.editor.setModelMarkers`（行と列を +1 する）。`.lua` のファイルは Monaco 組み込みの Lua の Monarch で色を付ける。
4. **ゴーストの開き方**: 起動の引数（exe へのドラッグドロップは引数 1 つで起動する）か、窓へのドロップか、フォルダを選ぶダイアログか、最近開いたものの一覧から開く。渡されたフォルダからは、`ghost/master` を探して開く（ゴーストのフォルダ・`ghost` のフォルダ・`ghost/master` そのもの、のどれを渡されても同じ `ghost/master` に着く。見つからなければ理由を出して開かない）。
5. **最近開いたもの**: アプリの永続の置き場（`%APPDATA%` の下）に TOML で残す（pasta のシリアライズは TOML の方針。Tauri の plugin-store は JSON なので使わない）。

退けた案:
- Electron（今の Node のコードは無改造で動くが、100MB を超え「軽量」に反する）。
- PWA（127.0.0.1 の MCP に届かない。ローカルネットワークアクセスの許可と CORS の壁）。
- Rust ネイティブの GUI（Monaco と同等のエディタ部品が無い）。
- フロントで WASM（`--target web`）を動かす案（Rust の変更は要らないが、読み込みとビルドの段が 1 つ増え、wasm-pack の手元の不調も抱える。開発者「WASM にはこだわらない」）。

### discovery で決めたこと（2026-11-12 開発者の承認）

1. 包み方は Tauri 2 ＋ Vite ＋ Monaco。
2. 解析は Rust の側（`pasta_lsp`・`pasta_check` を exe に取り込む）。
3. 置き場所は `editors/recipe/`（フロントは `src/`、Rust は `src-tauri/`）。`src-tauri` は**別の Cargo ワークスペース**（自前の `[workspace]` と `Cargo.lock`）にして、`crates/pasta_lsp`・`crates/pasta_check` を path 依存で引く。Tauri の依存（数百クレート）を、ルートの `Cargo.lock`・`deny.toml`・`cargo test --workspace` に混ぜない（#249 のルートの最小化に合わせる）。
4. 単一の exe・ドラッグドロップで `ghost/master` を開く・最近開いたフォルダを残す（開発者の追加の依頼）。
5. 配布は Windows x64 だけ。リリースの成果物に `pasta-recipe.zip`（exe ＋ `THIRD_PARTY_LICENSES.txt`）を足す。インストーラとコード署名は作らない。

## Scope

- **In**:
  - `editors/recipe/`: Tauri 2 のアプリ（Rust のコマンド・Vite・Monaco・worker の設定）。
  - 画面: ファイルとシーンのツリー、タブで複数のファイルを開く、保存（未保存の印・閉じるときの確かめ）、診断の一覧（押すとその行へ）、ゴースト全体の検査の一覧、最近開いたものの一覧。
  - `.pasta` の着色（TextMate ＋ セマンティックトークン）・診断。コードブロックの中の Lua の着色。`.lua` の編集（色付けまで）。
  - ツリーのシーンは `vscode-scene-outline-play` の `scenes` を写すだけ。ゴーストの全部の `.pasta` を解析してツリーを作り、保存したら作り直す。
  - 外でファイルが変わったときの扱い（開いているファイルの読み直しか知らせ。ツリーの作り直し）。決め方は要件で。
  - リリース CI（`.github/workflows/release.yml`・`build.yml`）で exe を作って添える。成果物の検査とリリースの全件の一覧も合わせる。`cargo about` で `THIRD_PARTY_LICENSES.txt` を作る。
  - マニュアル: VS Code を使わない編集の章（入手・起動・開き方・SmartScreen の警告が出ること・WebView2 が要ること）。
  - テスト: Rust のコマンド（`ghost/master` の探し方・最近開いたものの読み書き・解析の JSON の形が WASM の出力と同じであること）。フロントの写し（トークン・診断・シーンの並びから Monaco の形へ）。
- **Out**:
  - シーンの実行・SHIORI のリロード・変数の表示と書き換え（`pasta-recipe-dev-actions` が持つ）。
  - 補完・ホバー・定義へ移動・参照の検索・名前の変更。
  - Lua の構文診断とステップ実行（後で入れるなら `full_moon`（MPL-2.0）が候補）。
  - mac・Linux・arm64、インストーラ、コード署名、自動更新。
  - git・ターミナル・拡張機能・複数の窓。
  - `.nar` の作成（`pasta_check release` に任せる）。

## Boundary Candidates

- Rust のコマンド（解析・検査・ファイルの読み書き・ゴーストの探し方・最近開いたもの）と、フロントの写し（Monaco の形にするだけ）。
- エディタの部分（Monaco・着色・診断）と、ツリーの部分（フォルダ・ファイル・シーン）。
- アプリ本体と、リリース CI への組み込み。

## Out of Boundary

- `pasta_lsp` の解析そのものと、`scenes` の取り出し（`vscode-scene-outline-play` が持つ）。
- VS Code 拡張の振る舞い。拡張のファイルは、凡例を 1 か所にする以外では触らない。
- ゴーストを動かす機能（`pasta-recipe-dev-actions`）。

## Upstream / Downstream

- **Upstream**: `vscode-scene-outline-play`（`AnalysisResult.scenes`）。`pasta_lsp`・`pasta_check`。`pasta.tmLanguage.json`。
- **Downstream**: `pasta-recipe-dev-actions`（ツリーの ▶・変数の表をこのアプリに足す）。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**: `vscode-scene-outline-play`（`scenes` の形を受け取る側。形を変えたいときは向こうの spec で）。`pest-2-9-4-upgrade`（`pasta_lsp` のビルド。こちらは WASM を使わないので影響はネイティブのビルドだけ）。`pasta-dll-multiarch`・`vsce-4-upgrade`（`release.yml` を触る。同じウェーブに置かない）。

## Constraints

- 既存の VS Code 拡張を壊さない。拡張のビルド（WASM）と CI はそのまま通ること。
- 解析の JSON は、今の WASM の出力と同じ形（UTF-16・0 始まり）。
- ルートの `.cargo/config.toml` の `+crt-static` は `src-tauri` のビルドにも効く。webview2 との組み合わせで動くかを最初のビルドで確かめる（だめなら Tauri の `staticVCRuntime` に任せる道を要件・設計で決める）。
- Tauri は 2.12.x に固定する（3.0 は alpha）。Monaco 0.53 以降は ESM 前提なので、Vite で worker を `?worker` で読み、オフラインで動かす（CDN を使わない）。
- shiki の着色とセマンティックトークンの色の併用は確かめていない（設計で最初に確かめる）。
- 起動の手段（ドラッグドロップ以外。関連付け・送るメニューなど）は開発者に所論があり、要件で詰める。

## 申し送り（vscode-scene-outline-play より・2026-11-12）

- `scenes` は実装済み。Rust からは `pasta_lsp::analysis::AnalysisEngine::analyze` の `AnalysisResult.scenes: Vec<SceneNode>`（`SceneNode`・`SceneKind` は `pasta_lsp::analysis` から公開）。JSON の欄は `tokens`・`diagnostics`・`scenes` の順、項目は `name`・`kind`（`"global"`／`"local"`）・`heading`・`range`・`children`。行は 0 始まり・列は UTF-16。形は `crates/pasta_lsp/src/transport.rs` の形の試験が文字列で固定している。
- 範囲の式は pasta.dll の行からシーンを引く表と同じ（次の見出しの前の行まで）。シーンの後ろのアクター辞書・単語定義の塊は前のシーンの範囲に入る。構文の誤りのあるシーンは並びに出ず、範囲は出た見出しだけで決まる（詳しくは `completed/vscode-scene-outline-play/design.md` の Open Questions / Risks の Known limit）。
- 見出しの行からの再生は `OnPastaSceneAt` に `ファイルの絶対パス, 1 始まりの行` を渡す今の形のまま。

## 2026-11-12 棚卸の再測定（main 2381bf52）

- **前提の変化**: 起票の後に `vscode-scene-outline-play`（#252）と `scene-play-locals`（#253）が入った。`AnalysisResult.scenes`（`SceneNode`・`SceneKind`。`pasta_lsp::analysis` から公開・`serde::Serialize` 付き）が在るので、Tauri のコマンドは `AnalysisEngine::analyze` の結果をそのまま返せる（`transport.rs` の `wasm_*` は `cfg(target_arch = "wasm32")` だけ）。Current State の「シーンの並び: 今の解析結果には無い」は古い（上の申し送りが正しい）。ルートの `Cargo.toml` は #249 では変わっておらず（その後の変化は版の数字だけ）、`members = ["crates/*"]` なので、`editors/recipe/src-tauri` を自前の `[workspace]` にする案はそのまま通る。`pasta_lsp` は `workspace = "../.."` を書いた path 依存として引ける（`crate-type` に `rlib` がある）。
- **触るファイル**: 新しく `editors/recipe/`（`src/`・`src-tauri/` の `Cargo.toml`・`Cargo.lock`・`tauri.conf.json`、`package.json`）。既存は `.github/workflows/release.yml`（609 行）・`build.yml`（225 行）、添付を足すので `github-release.ps1`（72 行の名前の並び）・`.claude/skills/release/scripts/release-state.ps1`（84 行）・`release-scripts-test.ps1`（項目 10 の 6 つ）、`.gitignore`、マニュアルの新しい章と `SUMMARY.md`。1,000 行に近いものは無い。
- **規模**: 18〜21 タスク（土台と静的 CRT の確かめ 1、Rust のコマンド 4〜5、Vite・Monaco・worker 1、着色 2、診断 1、ツリー 2、タブ・保存・外の変更 2〜3、開き方と最近開いたもの 2、全体の検査の一覧 1、リリース CI と第三者の表示 2、マニュアル 1）。
- **先に要るもの**: 働きの上では無い（`vscode-scene-outline-play` は入った）。ファイルの重なり（未完了）: `vsce-4-upgrade`（`release.yml`）、`pasta-dll-multiarch`（`release.yml`・`build.yml`・`github-release.ps1`・`release-state.ps1`・`release-scripts-test.ps1`・`.gitignore`）、CI に clang を入れるなら `pest-2-9-4-upgrade`（`build.yml`・`release.yml`）。どれも版が未定でウェーブ 2 に入らないので、今は塞がない。`pasta-recipe-dev-actions` はこの後（`editors/recipe/`・マニュアルの章）。
- **種別**: 機能（VS Code を使わない編集のアプリ。開発者の依頼）。
- **要件定義のモデル**: Opus（形・解析の置き場・ワークスペースは discovery で決まった）。
- **分割の案**: 今は分けない（`pasta-recipe-dev-actions` を一度切り出した）。要件で 22 を超えたら、リリース CI の部分（`release.yml`・`build.yml` のビルドと中身の検査、`github-release.ps1`・`release-state.ps1`・`release-scripts-test.ps1`、`THIRD_PARTY_LICENSES.txt`、マニュアルの入手の節。4 タスクほど）を `pasta-recipe-release` として後ろのウェーブへ出す。
- **見つけた穴・古くなった記述**
  - 凡例は Rust の `pasta_lsp::analysis::TOKEN_TYPES`・`TOKEN_MODIFIERS`（`token_types.rs` 8・29 行の公開の定数）にもある。アプリが Rust のコマンドで凡例を取れば、拡張の `semanticTokensProvider.ts` を触らずに 1 か所にできる。
  - ゴースト全体の検査でファイルを集める処理（`collect`）は `pasta_check` のバイナリの側（`validate_cmd.rs`。`pub(crate)`）にあり、ライブラリから呼べない。拡張は TS の `ghostValidator.ts`（373 行）で集めている。Rust で書き直すか、`pasta_check` のライブラリへ移すかを要件で決める（移すなら `pasta_check` も触る）。
  - 配布物を足すときに合わせる所は、Scope の `release.yml`・`build.yml` だけでなく、`github-release.ps1`・`release-state.ps1`・`release-scripts-test.ps1`・リリースノートもある。添付は 6 → 7 になる。
  - 別のワークスペースには、ルートの `[profile.release]`（`opt-level = "z"`・`lto`・`panic = "abort"`・`strip`）と `[workspace.dependencies]` が効かない。`.cargo/config.toml` の `+crt-static` は、カレントディレクトリから上へ探すので効く。`about.toml` の `accepted` に MPL-2.0 は無く、exe に埋め込む JS（Monaco・shiki）の表示は `cargo about` では作れない。`deny.toml` も別の `Cargo.lock` を見ない。
