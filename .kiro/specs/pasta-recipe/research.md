# ギャップ分析: pasta-recipe

- **日付**: 2026-11-12（ブランチ `claude/pasta-recipe-b8469b`、base `2b2ba4f4`）
- **対象**: `requirements.md`（要件 1〜14）
- **種別**: 新しいアプリ（Complex Integration。新しいツールチェーン＋既存の解析器・検査の核・リリース CI へのつなぎ込み）

## 要約

- **解析と検査の核はそのまま使える**。`pasta_lsp::analysis::AnalysisEngine::analyze(&str) -> AnalysisResult`（`tokens`・`diagnostics`・`scenes`。`serde::Serialize` 付き）と、`pasta_check::validate::{select_dictionary_files, validate_ghost}` が公開されている。`wasm_*` は `cfg(target_arch = "wasm32")` の中だけなので、ネイティブの Tauri のコマンドは結果を `serde_json` でそのまま返せる。
- **欠けている部品の筆頭はゴーストのファイル集め**。`collect`・`walk`（`crates/pasta_check/src/validate_cmd.rs` 75 行〜。`pub(crate)`・バイナリの側）はライブラリから呼べず、拡張は同じ処理を TS（`ghostValidator.ts` 321〜369 行）で持っている。アプリで 3 本目を書くか、ライブラリへ移すかが要件ディスカッションの議題（OPEN QUESTION 3）。
- **UI の部品は全部が新規**（`editors/recipe/` は無い）。Monaco・shiki・Tauri の依存はリポジトリに 1 つも無い。VS Code の外で TextMate 文法を使う先例は `book/tools/highlight/`（`vscode-textmate`＋`vscode-oniguruma`、Lua の文法は MIT の写し `book/tools/highlight/grammars/lua.tmLanguage.json`）。
- **リリース CI のつなぎ込みは、既存の型の写しで済むが触る所が多い**。`release.yml`（build の作成と中身の検査・アップロード）・`github-release.ps1`（72 行の添付の名前の並び）・`release-scripts-test.ps1`（項目 10・11 の「6 つ」）・`.claude/skills/release/scripts/release-state.ps1`（84 行）・steering の `tech.md`。日々の `build.yml` には npm を使う job が無く、フロントのテストのための job が新しく要る。
- **第三者の表示が新しい難所**。ルートの `about.toml` の `accepted` に MPL-2.0 が無い（Tauri の依存の木に入る見込み）。exe に埋め込む JS（Monaco・shiki・文法）の表示は `cargo about` では作れない（OPEN QUESTION 7）。

## 1. 今の資産（Current State）

| 区分 | 資産 | 場所 | 使い方 |
|------|------|------|--------|
| 解析 | `AnalysisEngine::analyze` | `crates/pasta_lsp/src/analysis/mod.rs` 75・82 行 | そのまま呼ぶ。`AnalysisResult`・`Diagnostic`・`Range`・`Position`・`SceneNode`・`SceneKind` が公開 |
| 凡例 | `TOKEN_TYPES`（17）・`TOKEN_MODIFIERS`（3） | `crates/pasta_lsp/src/analysis/token_types.rs` 8・29 行 | Rust のコマンドで返せば 1 か所になる。拡張の `semanticTokensProvider.ts` 18〜45 行は同じ並びの写し（コメントで「same order」と明記・Rust との突き合わせのテストは無い） |
| 形の固定 | `test_analysis_result_json_shape_stable`・`test_ghost_validate_js_shape` | `crates/pasta_lsp/src/transport.rs` 78・126 行 | `serde_json` で文字列の形を固定している。アプリのコマンドの形の試験（要件 13.4）は同じ `serde_json` の出力を比べれば足りる |
| 検査の核 | `GhostInput`・`TomlSource`・`FileText`・`select_dictionary_files`・`validate_ghost`・`Report` | `crates/pasta_check/src/validate/` | ライブラリ（`default-features = false` でファイルシステム非依存）。`pasta_check` は `publish = true`（crates.io に出ている） |
| ファイル集め | `collect`・`walk`・`read_file` | `crates/pasta_check/src/validate_cmd.rs` 75〜140 行 | `pub(crate)`。最上位の `profile` を飛ばす・シンボリックリンクを飛ばす・`pasta.toml` の読み・UTF-8 の判定 |
| 拡張のファイル集め | `walk`・`readToml`・`findGhostMaster` | `editors/vscode/src/ghostValidator.ts` 21〜30・321〜369 行 | TS の写し。`ignoreBOM: true`（BOM を残す）。300 ms のまとめ・実行中の求めは 1 回だけ再実行・開いているファイルの `syntax` は出さない（要件 9.8・9.9 の手本） |
| 書きながらの診断 | `documentSync.ts`・`diagnosticsManager.ts` | `editors/vscode/src/` | 200 ms のまとめ。重大度は全部 Error（構文だけのため） |
| シーンの並び | `sceneOutline.ts` | `editors/vscode/src/` | global → Namespace、local → Function。選ぶと見出しへ |
| 着色の文法 | `pasta.tmLanguage.json`・`pasta-lua-injection.tmLanguage.json` | `editors/vscode/syntaxes/` | 注入は `meta.embedded.block.lua.content` に `source.lua`（VS Code 同梱）を差す。アプリでは Lua の文法を自前で持つ |
| VS Code の外での文法 | `tokenizer.mjs`・`highlight-html.mjs` | `book/tools/highlight/` | 二段のトークナイズ（pasta の後に Lua の範囲を Lua の文法で）。注入の文法は使っていない |
| 配布の型 | `pasta_check.zip` の作成と検査 | `crates/pasta_sample_ghost/release.ps1` 167〜224 行・`release.yml` 151〜163 行 | 「中身 2 つだけ」「exe の版＝タグの版」の検査の型。`cargo about generate -m … -c about.toml --target x86_64-pc-windows-msvc … about-pasta_check.hbs` |
| 静的 CRT | `.cargo/config.toml` | ルート | `[target.x86_64-pc-windows-msvc] rustflags = ["-Ctarget-feature=+crt-static"]`。`editors/recipe/src-tauri` から上へ探すので効く |
| ルートの release プロファイル | `opt-level = "z"`・`lto`・`codegen-units = 1`・`panic = "abort"`・`strip` | ルートの `Cargo.toml` 78〜83 行 | 別ワークスペースには効かない。アプリの `Cargo.toml` に要るものを書き写す |
| マニュアル | 「デバッグ」の部・`debug/vscode-setup.md` | `book/src/SUMMARY.md` 57 行 | 新しい章の置き場の候補。章の口調（クローディアの前置き）に合わせる |

### 慣習

- 位置は 0 始まり・UTF-16（解析器）、表示は 1 始まり。拡張は検査の結果の列を「コードポイント → UTF-16」に直している（`ghostValidator.ts` 297〜315 行）。`pasta_check` の `Finding` の列はコードポイントなので、アプリでも同じ直しが要る（要件 7.5・9.7）。
- エラーメッセージは日本語の短文（`validate_cmd.rs` の `「{} を読めません: {e}」`）。
- 配布物の検査は「止める」（作成ツールは問題があれば止める方針）。

## 2. 要件と資産の対応

| 要件 | 要るもの | 状態 | 種類 |
|------|----------|------|------|
| 1.1〜1.3 単一 exe・VC++ 不要・オフライン | `tauri build --no-bundle`・資産の埋め込み・`?worker` の Monaco worker | 無い | Missing |
| 1.2 静的 CRT と WebView2 の組み合わせ | `+crt-static` と `webview2-com`・`windows` クレートのリンク | 確かめていない | Unknown（最初のビルドで確かめる。だめなら Tauri の `staticVCRuntime`） |
| 1.4 WebView2 が無いとき | `--no-bundle` では Tauri の WebView2 の導入の仕組み（インストーラ用）が働かない。窓を作る前に WebView2 の有無を見てメッセージ箱を出す処理 | 無い | Missing・Research Needed |
| 1.6 版の表示と一致 | `tauri.conf.json` の `version`／`Cargo.toml` の版を、ワークスペースの版と合わせる仕組み。リリースの版の更新の手順（`release` スキル）が触る版のファイルが増える | 無い | Missing・Constraint（版の更新の手順にも足す） |
| 2 ゴーストを開く | `ghost/master` の探し方（大文字・小文字を無視）。拡張の `findGhostMaster` は「上へたどる」だけ、`pasta_check validate` は「`X/ghost/master`」だけ。3 つ＋上へたどる、は新しい | 無い | Missing（小さい純粋関数＋テスト） |
| 2.1・2.2 起動の引数・窓へのドロップ | Tauri の引数（`std::env::args`）・ドラッグドロップのイベント | 無い | Missing |
| 3 最近開いたもの | `%APPDATA%` の下の TOML の読み書き（`toml` クレートはルートの依存にある・別ワークスペースでは版を自前で書く） | 無い | Missing |
| 4 ツリー | フォルダの一覧（`walk` と同じ範囲）＋各 `.pasta` の `analyze` の `scenes` | 一部（`walk` は `pub(crate)`） | Missing／Constraint |
| 5 タブ・保存 | Monaco のモデルの管理・未保存の印・確かめの窓・改行の種類と BOM を残す保存 | 無い | Missing。BOM: 拡張は BOM を文字として残す。アプリは読みで BOM を外し、保存で戻す方が Monaco に素直（設計） |
| 6 色分け | `@shikijs/monaco` で `pasta.tmLanguage.json`＋Lua の文法、`registerDocumentSemanticTokensProvider` | 無い | Missing・Unknown（shiki の着色とセマンティックトークンの併用は確かめていない。brief の制約） |
| 6.3 凡例の一致 | Rust の `TOKEN_TYPES` を返すコマンド。拡張の TS の写しとの突き合わせのテスト（今は無い） | 一部 | Missing（拡張のファイルを触らずに、アプリの側のテストで Rust と拡張の TS を突き合わせられる） |
| 7 書きながらの診断 | `setModelMarkers`（+1）・一覧の UI・まとめ（拡張は 200 ms） | 無い | Missing |
| 8 `.lua` | Monaco 組み込みの Lua の Monarch | 無い（Monaco に在る） | Missing（設定だけ） |
| 9 ゴースト全体の検査 | ファイル集め＋`validate_ghost`＋列の直し＋まとめと再実行 | 核は在る・集めは `pub(crate)` | Missing／Constraint（OPEN QUESTION 3） |
| 10 外の変更 | フォルダの見張り（`notify` クレートか Tauri の plugin-fs の watch）・自分の保存の除外 | 無い | Missing・Research Needed（どちらを使うか・Windows の大文字・小文字） |
| 11 読み書きの範囲 | コマンドの入口でパスを開いた `ghost/master` の中に限る検査（`std::path::absolute` で正規化。8.3 の短縮名に注意） | 無い | Missing・Constraint（CI の `%TEMP%` が `RUNNER~1` で、`fs::canonicalize` と形の違いが出る既知の罠） |
| 12 リリース | `release.yml` の build に Node と `tauri build` を足す・中身の検査・アップロードの一覧・`github-release.ps1`・`release-scripts-test.ps1`・`release-state.ps1`・`tech.md` | 型は在る | Missing |
| 12.5 第三者の表示 | Rust 側は `cargo about`（`accepted` に MPL-2.0 が要る見込み）。JS 側は npm の依存のライセンスを集める仕組み | 無い | Missing・Research Needed（OPEN QUESTION 7） |
| 13 日々の CI | `build.yml` に Windows の job（Rust のテスト＋npm のテスト＋`tauri build` の確かめ）。今の `build.yml` には npm の job が無い | 無い | Missing |
| 13.2 ルートの検査を変えない | `src-tauri` を自前の `[workspace]`・`Cargo.lock` にし、ルートの `members = ["crates/*"]` に入れない。`deny.toml` は別の lock を見ない | 方針は決まっている | Constraint |
| 14 マニュアル | 新しい章＋`SUMMARY.md`＋`link-check.mjs` | 無い | Missing |

## 3. 実装の道筋の案

### 案 A: 既存を広げる（検査のファイル集めを `pasta_check` のライブラリへ移す）

- `validate_cmd.rs` の `collect`・`walk`・`read_file` を、ライブラリの opt-in の feature（例: `fs`）の下へ移し、`pasta_check validate` とアプリが同じ関数を呼ぶ。拡張の TS（WASM の中ではファイルに触れない）はそのまま。
- ✅ 「同じ結果」（要件 9.5・13.6）が作りで保証される。ツリーの範囲（要件 4.3）も同じ `walk` で揃う。
- ✅ アプリの Rust は薄くなる。
- ❌ 公開クレート `pasta_check` のライブラリの面が増える（「ファイルシステム非依存」の説明に「`fs` の feature を除く」が付く）。WASM の検査（`build.yml` の `cargo check -p pasta_check --lib --no-default-features --target wasm32`）を壊さないよう feature で分ける必要がある。
- ❌ この spec が `crates/pasta_check` を触る（brief の Out of Boundary には入っていないが、境界の線が 1 本増える）。

### 案 B: アプリの中に新しく作る（集めを Rust で書き直す）

- `editors/recipe/src-tauri` に `walk` 相当を書き、`select_dictionary_files`・`validate_ghost` だけを呼ぶ。
- ✅ `pasta_check` を触らない。境界が brief どおり。
- ❌ 同じ集めが Rust のバイナリ・TS の拡張・Rust のアプリの 3 本になり、いずれずれる（`pasta-recipe-dev-actions` の brief が「2 本になりずれる」を理由に退けた型と同じ）。
- ❌ 一致は試験（要件 13.6）でしか守れない。

### 案 C: 混ぜる（推奨の見込み）

- 検査のファイル集めは案 A（`pasta_check` のライブラリへ、feature の下で移す）。`pasta_check validate` の振る舞いは変えない（移すだけ。特性化の試験を先に置く）。
- それ以外（ゴーストの探し方・最近開いたもの・読み書きの範囲・見張り）はアプリの中に新しく作る。
- 凡例は Rust の `TOKEN_TYPES` を返すコマンドで 1 か所にし、拡張の TS の写しとの突き合わせはアプリの側のテストで行う（拡張のファイルは触らない）。
- UI はアプリの中に新しく作る（共有は `pasta-recipe-dev-actions` が `editors/shared/` を作るときに考える。今は作らない）。
- ✅ 一致を作りで保証しつつ、触る既存のクレートは `pasta_check` の 1 つだけ。
- ❌ 設計で feature の名前・依存の分け方（`std::fs` だけで足り、新しい依存は要らない見込み）を決める必要がある。

## 4. 規模と危うさ

- **規模: L（1〜2 週間）**。UI は全部新しく、Tauri・Monaco・shiki はリポジトリに初めて入る。Rust のコマンドは既存の核を呼ぶだけで小さい。タスクは brief の見込みどおり 18〜21（土台と静的 CRT・WebView2 の確かめ 1、Rust のコマンド 4〜5、Vite・Monaco・worker 1、着色 2、診断 1、ツリー 2、タブ・保存・外の変更 2〜3、開き方と最近開いたもの 2、全体の検査の一覧 1、リリース CI と第三者の表示 2、日々の CI 1、マニュアル 1）。22 を超えないので分けない見込み（OPEN QUESTION 8）。
- **危うさ: 中〜高**。
  - 高: shiki の着色と Monaco のセマンティックトークンの併用（brief の制約。だめなら下地を Monarch に写すか、セマンティックトークンだけで塗る道へ。設計の最初に確かめる）。
  - 中: 静的 CRT と WebView2 の組み合わせ・WebView2 が無いときの振る舞い・exe の大きさ。
  - 中: 第三者の表示（MPL-2.0 と JS の表示）。
  - 低: 解析・検査の呼び出し（既存の公開の API・形の試験がある）。

## 5. 設計へ持ち越す調べもの（Research Needed）

1. `@shikijs/monaco` の下地の色と `registerDocumentSemanticTokensProvider` の色の重ね方（`'semanticHighlighting.enabled': true` で両方が出るか・テーマの `semanticTokenColors` の当て方）。shiki の Lua の文法を柵の中へ差す方法（注入か、`embeddedLangs`）。
2. `+crt-static` での `tauri build --no-bundle` の成否と、`staticVCRuntime` との関係。exe の大きさの目安（要件 12.4 の上限の値）。
3. `--no-bundle` の exe で WebView2 ランタイムが無いときの振る舞いと、窓を作る前に有無を確かめる方法（`webview2-com` の `GetAvailableCoreWebView2BrowserVersionString` など）。
4. Tauri 2.12 の依存の木のライセンス（MPL-2.0 の有無と数）。JS の依存のライセンスを `THIRD_PARTY_LICENSES.txt` へ足す方法（`license-checker` 系の devDependency か、Vite の出力の注記を集めるか）。
5. フォルダの見張りの道具（`notify` クレートか Tauri の plugin-fs の watch）と、自分の保存の除外・まとめの方法。
6. 版を 1 か所にする方法（`tauri.conf.json` の `version` を `Cargo.toml` から取る設定）と、`release` スキルの版の更新・`pretag-check.ps1` が見る版のファイルに加えるかどうか。
7. `release.yml` の build job への Node（`editors/recipe/package-lock.json` のキャッシュ）と `tauri-cli` の入れ方（`@tauri-apps/cli` の npm の devDependency なら版の固定が lock でできる）。
8. 保存で改行の種類と BOM を残す方法（要件 5.7）。読みで BOM を外して覚え、保存で戻すか、拡張と同じく文字として残すか。Monaco のモデルの `EndOfLineSequence` の当て方。
9. `pasta-recipe.zip` の大きさの上限の値（要件 12.4）。最初のビルドの実測から決める。
10. WebView2 の作業用のデータ（`EBWebView`）の置き場。Tauri の既定（`%LOCALAPPDATA%` の下の識別子のフォルダ）のままで要件 1.7・11.1 を満たすかを確かめる。

## 6. つなぎ込みの注意（Constraints）

- `release.yml`・`build.yml`・`github-release.ps1`・`release-state.ps1`・`release-scripts-test.ps1`・`.gitignore` は、未完了の `pasta-dll-multiarch`（と `release.yml` は `vsce-4-upgrade`、CI に clang を入れるなら `pest-2-9-4-upgrade`）と重なる。同じウェーブに置かない（brief の棚卸どおり）。
- `.claude/skills/release/scripts/release-state.ps1` はスキルのフォルダの中。スキルのファイルの書き換えは分類器が止めることがある（SKILL.md の編集の前例）。止まったら入った分と残りを示して開発者に許可を求める。
- 別ワークスペースにはルートの `[profile.release]` と `[workspace.dependencies]` が効かない。`pasta_lsp` は `workspace = "../.."` を持つので、path 依存として引いたときの `version.workspace = true` などの解決を最初のビルドで確かめる（brief では通る見込み）。
- 読み書きの範囲の判定（要件 11）は、`fs::canonicalize` でなく `std::path::absolute` で形を揃える（CI の 8.3 の短縮名の前例）。
- 拡張の WASM のビルド（`npm run build:wasm`）・`build.yml` の `wasm` job は触らない。

## 7. 設計への申し送り（推奨）

- 案 C を基に設計する。最初のタスクで「静的 CRT の exe が起動する」「shiki とセマンティックトークンの色が重なる」の 2 つを確かめ、だめなときの道をその場で選ぶ。
- Rust のコマンドは、`analyze`（`AnalysisResult` をそのまま）・`legend`・`open_ghost`（探し方）・`list_tree`・`read_file`／`write_file`（範囲の検査・改行と BOM）・`validate_ghost`・`recent_list`／`recent_push` を見込みとする。どれも純粋な部分（探し方・範囲の判定・TOML の読み書き・形）をテストできる関数に分ける。
- フロントの写し（トークンの差分の位置 → Monaco、診断の +1、シーン → ツリー、検査の列のコードポイント → UTF-16）は、DOM に依らない関数に分けて Node のテストで確かめる。

## 8. 要件ディスカッションの議題（OPEN QUESTIONS）

1. **正式な名前**: exe 名・窓の題名・マニュアルの章題を何にするか。仮定は `pasta-recipe`（brief は「正式な名前は要件で決める」）。
2. **起動の手段**: ドラッグドロップ・窓へのドロップ・ダイアログ・最近開いたもの、のほかに、関連付け・「送る」メニューなどを入れるか。仮定は「入れない（インストーラ無しなので、Windows の設定に書き込まない）」。あわせて、`ghost/master` の中のフォルダやファイルを渡したときは上へたどってそのゴーストを開き、ファイルならそれも開く（要件 2.6）、複数のパスは最初の 1 つ（要件 2.1）、2 つ目の起動は別の窓として動く（1 つの窓に 1 つのゴースト・同じファイルを 2 つで開いたときの保護はしない）、を仮定にしている。
3. **ゴースト全体の検査のファイル集め**: `pasta_check` のライブラリへ（feature の下で）移して `pasta_check validate` とアプリで共有するか、アプリで書き直すか。仮定は「ライブラリへ移す（案 C）」。移すなら `crates/pasta_check` を触る。
4. **外でファイルが変わったとき**: 未保存の書き換えの無いタブは黙って読み直し、ある タブは知らせて「読み直す／今の中身を残す」を選ばせ、消えたファイルのタブは未保存として残す（要件 10）。これで良いか。
5. **`.pasta`・`.lua` 以外のファイル**: `pasta.toml`・`descript.txt` などの UTF-8 のテキストも、色分けなしで開いて編集できるようにするか（要件 5.2 の仮定は「できる」）。UTF-8 でないもの（Shift_JIS の `descript.txt`・画像）は開かない。
6. **ツリーからのファイル操作**: 新しいファイル・名前の変更・削除をツリーから行えるようにするか。仮定は「しない（エクスプローラーで行い、ツリーは外の変更として追う）」。
7. **第三者の表示と依存の検査**: Tauri の依存に入る見込みの MPL-2.0 を、アプリの第三者の表示の設定でだけ受け入れるか。JS の部品（Monaco・shiki・文法）の表示をどう集めるか。アプリの別の `Cargo.lock` にも `cargo deny` を当てるか。仮定は「アプリ用の設定でだけ MPL-2.0 を受け入れる・JS の表示は npm の依存から集めて同じファイルに足す・`cargo deny` もアプリの lock に当てる」。
8. **spec の分割**: 見込み 18〜21 タスクで 22 を超えないので、リリース CI の部分（`pasta-recipe-release`）を分けない。これで良いか。
