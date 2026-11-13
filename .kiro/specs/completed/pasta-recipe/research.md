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

### 要件ディスカッションの結論（2026-11-12 開発者の承認・全議題を推奨案で確定）

1. 名前は `pasta-recipe`（exe は `pasta-recipe.exe`）で確定。
2. アプリはレジストリに書かない。関連付け・「送る」は作者が Windows の機能で行い、要件 2.1・2.6 の決まりで開く（マニュアルに書く・要件 14.6）。2 つ目の起動は今の窓へ渡す単一インスタンス（要件 2.11）。複数のパスは最初の 1 つ。
3. 案 C で確定。ファイル集めを `pasta_check` のライブラリの opt-in の機能へ移す（要件 9.12・13.8。特性化の試験を先に置く）。
4. 外の変更は下書きのとおり（VS Code と同じ）。
5. `.pasta`・`.lua` 以外の UTF-8 のテキストは色分けなしで開ける。TOML の色分けは入れない。
6. ツリーからファイル・フォルダの作成と名前の変更をする（要件 4.14〜4.16）。削除はしない（要件 4.17）。
7. Tauri 本体は MIT／Apache-2.0。MPL-2.0 は推移的な依存に入る見込みで、確かめていない。アプリ専用の `about.toml` を持ち、受け入れる並びは設計で `cargo about` を回して出てきたものに合わせる（要件 12.10）。ルートの `about.toml` は変えない。pasta.dll は Tauri に依存しない（ルートの `Cargo.lock` に Tauri は 0 件・`pasta_shiori` は `pasta_lsp`・`pasta_check` に依存しない）。`cargo deny` をアプリの lock にも当てる（要件 13.7）。
8. spec は分けない。設計で 22 タスクを超えたら、リリース CI の部分を `pasta-recipe-release` へ出す。

---

# 設計フェーズの調査（2026-11-13・`/kiro-spec-design pasta-recipe -y`）

## Summary

- **Feature**: `pasta-recipe`
- **Discovery Scope**: New Feature ＋ Complex Integration（full discovery。新しいツールチェーン Tauri・Monaco・shiki と、既存の解析器・検査の核・リリース CI へのつなぎ込み）
- **使ったスキル・規則**: `kiro-spec-design`（`design-principles.md`・`design-discovery-full.md`・`design-synthesis.md`・`design-review-gate.md`）、`kiro-validate-design`（`design-review.md`）、steering の `tech.md`（全部）・`product.md`・`structure.md`（要点）。開発者の常設の方針（パスは `std::path::absolute`・移す前に特性化の試験・ルートに Tauri を入れない・小さく完全に）を当てた。
- **Key Findings**:
  1. **MPL-2.0 は Tauri 2.12 の依存の木に実際に入る（確かめた）**。`x86_64-pc-windows-msvc` の通常の依存 230 個のうち MPL-2.0 は 5 つ（`option-ext`・`cssparser`・`cssparser-macros`・`selectors`・`dtoa-short`）。`option-ext` は `dirs-sys` → `dirs` → `tauri` で実行時にリンクされる。フォルダの見張りに使う `notify` 8.2 は **CC0-1.0**。ほかは MIT・Apache-2.0・Unicode-3.0・Zlib・BSD-3-Clause と、それらを選べる OR の式だけ（`Unlicense OR MIT` など）。
  2. **shiki とセマンティックトークンは、作りの上では重ねられる（ソースで確かめた・実物の画面では確かめていない）**。`@shikijs/monaco` 4.6.0 の `shikiToMonaco` は `setTokensProvider`（行ごとの下地）と `defineTheme`・`setTheme` の差し替えだけで、セマンティックトークンには触らない。Monaco のセマンティックトークンの色はテーマの `rules` を種類の名前で引くので、shiki に渡すテーマに種類の名前の規則を足せば、両方が同じテーマから色を取る。shiki は `injectTo`（VS Code の `contributes.grammars` と同じ）を型で持つので、拡張の注入の文法をそのまま使える。
  3. **静的 CRT は Tauri の既定と同じ向き（ソースで確かめた・ビルドは確かめていない）**。Tauri の `build.windows.staticVCRuntime` は既定で true（`tauri-utils` 2.10.2）。`tauri-build` 2.7.4 は `static_vcruntime::metabuild()` を呼び、そのクレートは `+crt-static` を併せて使うことを勧めている。ルートの `.cargo/config.toml` の `+crt-static` とぶつからない。
  4. **別ワークスペースからの path 依存は通る（確かめた）**。スクラッチの試しのプロジェクト（自前の `[workspace]`）から `crates/pasta_lsp`・`crates/pasta_check`（`default-features = false`）を path で引くと、`version.workspace = true` などの受け継ぎが解け、`pasta_lua` は入らない（ローカルは `pasta_check`・`pasta_core`・`pasta_dsl`・`pasta_lsp` の 4 つだけ）。ただし新しく作った lock は pest を 2.9.6 に解いた（ルートの `Cargo.lock` は 2.9.2 に据え置き。`pest-2-9-4-upgrade` の理由）。**アプリの lock を素直に作ると、拡張の WASM と別の版の pest で解析することになる**。

## Research Log

### Tauri 2.12 の依存の木とライセンス
- **Context**: 要件 12.5・12.10・13.7。要件ディスカッションでは「MPL-2.0 は入る見込み・確かめていない」。
- **Sources Consulted**: スクラッチの試しのプロジェクト（`tauri = "~2.12"`・`tauri-build = "2.7"`・`tauri-plugin-single-instance = "2"`・`tauri-plugin-dialog = "2"`・`notify = "8"`・`toml`・`serde`・`serde_json`）で `cargo metadata --filter-platform x86_64-pc-windows-msvc` と `cargo tree -e normal -i`。解けた版: tauri 2.12.5・tauri-build 2.7.4・wry 0.57.2・tao 0.37.2・webview2-com 0.39.1・windows 0.62.2・windows-sys 0.61.2・tauri-plugin-dialog 2.8.4（rfd 0.17.2・tauri-plugin-fs 2.7.1 を引く）・tauri-plugin-single-instance 2.5.5・notify 8.2.2。
- **Findings**: 上の Key Findings 1。`tauri-build` は `^2.7.0` が要る（`~2.5` では解けない）。数は `cargo metadata` の通常の依存のたどり（proc-macro を含む）で、`cargo about` の数え方と同じとは限らない。
- **Implications**: アプリ専用の `about.toml`・`deny.toml` の受け入れる並びに `MPL-2.0` と `CC0-1.0` を足す。ルートの並びは変えない（要件 12.10）。`cargo about` が実際に出す並びは最初のビルドで確かめ、足りなければ中身を読んでその場で足す（推測で広げない）。

### shiki と Monaco のセマンティックトークン
- **Context**: brief の制約「併用は確かめていない」・要件 6.1〜6.4・6.6。
- **Sources Consulted**: `@shikijs/monaco` 4.6.0 の `dist/index.mjs`（全 153 行）、`@shikijs/types` 4.6.0 の `LanguageRegistration`（`embeddedLangs`・`injectTo`）。npm の版: `monaco-editor` 0.57.0（MIT）・`shiki`／`@shikijs/monaco`／`@shikijs/langs`／`@shikijs/themes` 4.6.0（MIT）・`@tauri-apps/cli` 2.12.4・`@tauri-apps/api` 2.12.4。
- **Findings**:
  - `shikiToMonaco` は **Monaco に登録済みの言語にだけ** 下地の着色を付ける（`monaco.languages.getLanguages()` に無い言語は飛ばす）。`pasta`・`lua` を先に `monaco.languages.register` する。
  - 下地の着色は、shiki が出した色をテーマの規則の名前へ引き戻して Monaco に渡す作り。同じ色の規則が増えても下地の結果は変わらない。
  - 拡張は `package.json` の `semanticTokenScopes` で種類 → TextMate のスコープに写している（17 種類すべて）。**同じ写しで「種類 → そのスコープのテーマの色」の規則を作れば、拡張と同じ部分が同じ種類として塗られる**（要件 6.6）。
- **Implications**: 最初のタスクで、Monaco の上で両方が出ることを実物で確かめる。だめなときの道: shiki をやめ、`book/tools/highlight/` と同じ `vscode-textmate`＋`vscode-oniguruma` を自前の `setTokensProvider` でつなぐ（テーマと写しはそのまま使える）。

### `.lua` のファイルの色
- **Context**: brief は「`.lua` は Monaco 組み込みの Lua の Monarch」。要件 8.1 は「Lua の色分け」だけ。
- **Findings**: 柵の中の Lua のために shiki の Lua の文法を読むと、`shikiToMonaco` は登録済みの `lua` の言語にもその文法で下地を付ける。Monarch を別に読む理由が無い。Monaco は `edcore.main`（エディタの部品だけ・言語なし）を読めば、Monarch の全言語を同梱しない分 exe も小さくなる。
- **Implications**: `.lua` も柵の中も shiki の Lua の文法 1 つで塗る（brief の Approach からの単純化。要件は満たす）。

### 静的 CRT・WebView2・データの置き場・版
- **Sources Consulted**: `tauri-build` 2.7.4 `src/lib.rs`（`should_static_link_vc_runtime`）、`static_vcruntime` 3.0.0 の文書、`tauri-utils` 2.10.2 `src/config.rs`、`tauri` 2.12.5 `src/manager/webview.rs`・`src/path/desktop.rs`・`src/lib.rs`。
- **Findings**:
  - 上の Key Findings 3。
  - `tauri::webview_version()` が公開されている（`tauri_runtime_wry::webview_version` の再公開）。窓を作る前に WebView2 の有無を確かめられる。新しい依存は要らない。
  - Windows では WebView2 の作業用のデータを、指定が無ければ `app_local_data_dir()`（`%LOCALAPPDATA%\<識別子>`）に置く。`app_config_dir()` は `config_dir()`（`%APPDATA%`）`\<識別子>`。どちらも exe のそばでもゴーストの中でもない（要件 1.7・3.6・11.1）。
  - `tauri.conf.json` の `version` は「semver の文字列か、`version` を持つ `package.json` へのパス」。exe の VERSIONINFO と `package_info().version` はこの値から作られる。
- **Implications**: WebView2 の確かめは `tauri::webview_version()`＋`windows-sys`（既に木にある 0.61）の `MessageBoxW`。版は拡張の `editors/vscode/package.json` を指せる（Design Decisions と OQ-1）。

### 別ワークスペースの path 依存と lock の版
- **Sources Consulted**: スクラッチの試しのプロジェクトで `cargo metadata`（Key Findings 4）。ルートの `Cargo.toml`（`pest = "2.8.6"`）と `pest-2-9-4-upgrade` の brief（lock は 2.9.2 に据え置き）。
- **Implications**: アプリの `Cargo.lock` は、**ルートの `Cargo.lock` を写した上で解いて作る**（cargo は既にある版で足りればそれを使う）。`pasta_lsp`・`pasta_check` が引くクレートの版がルートの lock と同じであることをテストで固定する（要件 13.4 の「同じ中身」を版の上でも守る）。path のクレートの版も lock に書かれるので、リリースの版の更新でアプリの lock も更新が要る（`--locked` のビルドが止まる）。

### 既存のコードの確かめ
- `crates/pasta_check/src/validate_cmd.rs` の `collect`・`walk`・`read_file` は単体テスト（`test_collect_entries`・`test_collect_dirs`・`test_collect_contents`・`test_collect_skips_symlinks`（unix だけ））を持つ。`walk` の `profile` の判定は大文字・小文字を区別する（`name == "profile"`）。`release.rs` は `run_on_master` を使う。
- `pasta_check::validate::{Report, Finding}` は `serde::Serialize`（`camelCase`・`code` は kebab-case）。`Finding.column` はコードポイントで 1 始まり。
- `pasta_lsp::analysis::{AnalysisEngine, AnalysisResult, SceneNode, SceneKind, TOKEN_TYPES, TOKEN_MODIFIERS}` が公開。`SemanticToken` の JSON の名前は `deltaLine`・`deltaStartCharacter`・`length`・`tokenType`・`tokenModifiers`。`char_column_to_utf16` は公開されていない。
- 拡張の `semanticTokensProvider.ts` 18〜45 行の凡例は Rust と同じ並び（突き合わせのテストは無い）。
- `release.yml` の build は `release.ps1`（dll・nar・`pasta_check.zip`・skills）と VSIX を別の step で作り、「Verify release assets」で中身を検査し、「Check Cargo.lock is unchanged」で lock の不変を確かめる。`github-release.ps1` 72 行に添付 6 つの名前の並び。CI の Node は 22（`node --test` が TypeScript の型を外して走らせられる版）。
- 要件 12.5 と 12.6 が `requirements.md` の同じ行に続けて書かれている（改行の抜け。番号はそのまま 12.6 として扱う）。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 薄い Rust の殻＋素の TypeScript の画面（採用） | Rust のコマンドは既存の核を呼ぶだけ・パスの検査は Rust の入口に 1 か所。画面はフレームワーク無しの TS のモジュール。写しは DOM に依らない関数 | 依存が最少・純粋な関数を `node --test` と `cargo test` で確かめられる・`pasta-recipe-dev-actions` が足し場所を得やすい | 画面の部品（ツリー・タブ・一覧・確かめの窓）を手で書く。どれも小さい | 状態は「開いたゴースト 1 つ＋タブの並び」だけ |
| React などの UI フレームワーク | 部品と状態の管理を任せる | 部品が増えたとき楽 | 依存と第三者の表示が増える・今の規模では要らない | 退けた |
| フロントでファイルを読む（plugin-fs） | 読み書きを JS から行う | Rust のコマンドが減る | 読み書きの範囲（要件 11）の守りが JS の許可の設定に散る・BOM と UTF-8 の判定が JS と Rust に分かれる | 退けた |

## Design Decisions

### Decision: ファイル集めを `pasta_check` のライブラリの `fs` の機能へ移す
- **Context**: 要件 9.12・13.8（要件ディスカッションで案 C に確定）。
- **Selected Approach**: `collect`・`walk`・`read_file` を新しいモジュール `pasta_check::ghost_files`（`#[cfg(feature = "fs")]`）へ移し、既存の単体テストも一緒に移す。`cli` は `fs` を含む。WASM の `--no-default-features` のビルドは `fs` を持たない。移す前に、バイナリの外から `pasta_check validate` の出力を固定する特性化の試験を足す（`profile` の飛ばし・入れ子の `profile`・UTF-8 でないファイル・`pasta.toml` 無し）。
- **Trade-offs**: 公開クレートの面が増える（`fs` の機能の下の関数 3 つと型 1 つ）。`lib.rs` の「ファイルシステムに触れず」の説明を「`fs` の機能を除き」に直す。

### Decision: 着色は shiki＋拡張の文法と写しを読むだけで使い、テーマは light-plus／dark-plus に種類の規則を足す
- **Context**: 要件 6.1〜6.4・6.6・6.7・8.1。
- **Selected Approach**: `editors/vscode/syntaxes/pasta.tmLanguage.json`・`pasta-lua-injection.tmLanguage.json` と `editors/vscode/package.json` の `contributes.semanticTokenScopes` を Vite で読み取りの import（`book/tools/highlight/` と同じ「拡張が正本・読むだけ」）。Lua の文法は `@shikijs/langs` の `lua`。テーマは `@shikijs/themes` の `light-plus`・`dark-plus`（VS Code の既定のテーマ）に、セマンティックトークンの種類ごとの規則（色は写し先のスコープのテーマの色）を足す純粋な関数で作る。明暗は `prefers-color-scheme` に合わせて `setTheme`。
- **Trade-offs**: 拡張のファイルが変わるとアプリの見た目も変わる（それが狙い。Revalidation Triggers に書く）。

### Decision: パスはすべて `ghost/master` からの相対の `/` 区切りでやり取りし、Rust の入口で検査する
- **Context**: 要件 11.1〜11.4・2.4〜2.7。
- **Selected Approach**: 画面は絶対パスを持たない（開く操作のときだけ渡す）。Rust の `resolve(master, rel)` が、空・`.`・`..`・`\`・`:`・予約されたデバイス名・末尾の `.` と空白の要素、最上位の `profile`（大文字・小文字を区別しない）、途中のシンボリックリンク（ジャンクションを含む）を拒む。`master` は `std::path::absolute` で 1 度だけ形を揃える（`fs::canonicalize` は使わない）。
- **Rationale**: 守りが 1 か所。要素の検査で `..` を拒むので、正規化に頼らずに外へ出られない。

### Decision: 外の変更は中身の比べで判定する
- **Context**: 要件 10.2〜10.4・10.6。
- **Selected Approach**: タブごとに「最後にディスクと揃えた中身（`diskText`・BOM の有無）」を持つ。見張りの知らせが来たら、開いているタブを全部ディスクから読み直して比べる。同じなら何もしない（自分の保存はここで消える）、無ければ消えた、未保存の書き換えが無ければ読み直す、あれば尋ねる。
- **Rationale**: 時刻や「自分が書いた」の印を持たずに 10.6 を満たす。フォルダごと消されたときに子の知らせが届かなくても漏れない。

### Decision: アプリの版は拡張の `package.json` から取る（仮定・OQ-1）
- **Context**: 要件 1.6・12.3。別ワークスペースなのでルートの版を受け継げない。
- **Alternatives Considered**: (1) アプリの `Cargo.toml` に版を書き、リリースの版の更新に足す（版のファイルが 5 → 6・`verify-tag.ps1`・`pretag-check.ps1` とそのテストも直す）。(2) `tauri.conf.json` の `version` を `../../vscode/package.json` に向ける（新しい版のファイルなし。`verify-tag.ps1` が既にタグとの一致を確かめている）。(3) リリースの CI で `--config` で版を渡す（日々のビルドの版が 0.0.0 になる）。
- **Selected Approach**: (2)。どの案でも、アプリの `Cargo.lock` に path のクレートの版が書かれるので、リリースの版の更新の手順にアプリの lock の更新（`cargo update --workspace --manifest-path editors/recipe/src-tauri/Cargo.toml`）を足す。
- **Follow-up**: `release` スキル（`SKILL.md`）の書き換えは分類器が止めることがある。止まったら入った分と残りを示して開発者に許可を求める。

### Decision: パニックで落とさない
- **Context**: 要件 6.5・5.8（書いた中身を失わない）。ルートの `[profile.release]` は `panic = "abort"`。
- **Selected Approach**: アプリの `[profile.release]` は `panic` を書かない（unwind のまま）。`analyze` は `spawn_blocking` の中で走らせ、パニックは `JoinError` として `Err` で返す（`catch_unwind` を別に書かない）。画面は下地の色を残して書き続けられる。
- **Rationale**: 解析器の思わぬパニックで、未保存の書き換えごと窓が消えるのを防ぐ。

### Decision: JS の第三者の表示は Vite の小さな plugin で集める
- **Context**: 要件 12.5。
- **Selected Approach**: `vite build` のときに、出力に入ったモジュールの属するパッケージ（`node_modules/<名前>`）を集め、各パッケージの `license` の欄と `LICENSE*`・`ThirdPartyNotices*` の本文を 1 つのテキストに書き出す（新しい devDependency なし）。`package.ps1` が `cargo about` の出力の後ろに足して `THIRD_PARTY_LICENSES.txt` にする。拡張の文法（pasta 自身・MIT）は pasta の LICENSE の表示で足りる。
- **Trade-offs**: 30 行ほどの自前の処理。束ねに入ったものだけを数えるので、devDependency を誤って載せない。

## Synthesis

- **一般化**: 開く手段は 5 つ（起動の引数・2 つ目の起動・窓へのドロップ・ダイアログ・最近開いたもの）あるが、どれも「パス 1 つ → `resolve_ghost` → 未保存の確かめ → `open_ghost`」の 1 本の流れに入れる。ツリー・診断の一覧・検査の一覧の押下も「相対パス＋位置 → タブを開いて位置へ」の 1 つの操作。外の変更・自分の保存・作成・名前の変更の後のツリーの作り直しも「ツリーを読み直し、足りないシーンを読む」の 1 つ。
- **作るか使うか**: 解析・検査・ファイル集めは既存の核を使う。単一インスタンス・フォルダのダイアログは Tauri の公式の plugin、見張りは `notify`（plugin-fs の watch も中で `notify` を使う）、ドラッグドロップは Tauri の窓のイベント、WebView2 の確かめは `tauri::webview_version()`。確かめの窓（保存する・保存しない・取り消す）は 3 択と日本語の文言が要るので HTML の小さな窓で作る。
- **単純化**: `.lua` の Monarch をやめる（上記）。最近開いたものの記録は Rust の側の 1 つの TOML。シーンは別のコマンドを作らず `read_text`＋`analyze` で作る。UI フレームワーク・状態管理のライブラリ・JS のファイルの読み書きは使わない。保存の改行は Monaco のモデルの改行に任せ、BOM だけを Rust が外して覚え、保存で戻す。

## Risks & Mitigations

- shiki の下地とセマンティックトークンが実物で重ならない — 最初のタスクで確かめる。だめなら `vscode-textmate`＋`vscode-oniguruma` の自前の下地へ（テーマ・写しはそのまま）。
- アプリの依存に `cargo deny` を当てると、Tauri の木の勧告（unmaintained など）で止まる — アプリの `deny.toml` は `targets = ["x86_64-pc-windows-msvc"]` に絞り、出たものは中身を読んで 1 件ずつ判断する（まとめて無視しない）。
- exe・zip の大きさ — 見積もりは exe 10 MB 前後・zip 5 MB 前後。上限は zip 30 MB（要件 12.4）。最初のビルドで実測して比べる。
- `release.yml`・`build.yml`・`github-release.ps1`・`release-state.ps1`・`release-scripts-test.ps1`・`.gitignore` は `pasta-dll-multiarch` などと重なる — 同じウェーブに置かない（brief の棚卸どおり）。
- 日々の CI の時間（Windows で Tauri のビルド） — `Swatinem/rust-cache` の `workspaces` にアプリを足す。

## References

- `@shikijs/monaco` 4.6.0 `dist/index.mjs`・`@shikijs/types` 4.6.0（npm の tarball をスクラッチに展開して読んだ）
- `tauri` 2.12.5・`tauri-build` 2.7.4・`tauri-utils` 2.10.2・`static_vcruntime` 3.0.0（cargo の registry の展開済みのソース）
- `cargo metadata`／`cargo tree` の結果（スクラッチの試しのプロジェクト。リポジトリには残さない）
- `book/tools/highlight/`（VS Code の外で拡張の文法を読む先例）、`editors/vscode/package.json` の `semanticTokenScopes`

## 設計ディスカッションの結論（2026-11-13 開発者の承認・全議題を推奨案で確定）

1. 版は `tauri.conf.json` から `editors/vscode/package.json` を指す。`release` スキルの版の更新にアプリの lock の `cargo update --workspace` を足す。lock の版の一致の試験（pest の版のずれを捕まえる）は残し、落ちたときに直し方のコマンドを出す。スキルの書き換えはリリース CI と同じタスクで行い、分類器に止められたら開発者の許可を求める。
2. アプリの最初のタスクを関門にする（静的 CRT の `--no-bundle` ビルド・大きさの実測・shiki とセマンティックトークンの色の重なりのスクリーンショット）。合格まで画面のタスクに進まず、だめなら同じタスクの中で代わりの道へ切り替えて報告する。
3. タブの閉じる・開き直す・名前の変更・全部保存の判断を純粋な `tabs.ts` に出し、`node --test` で確かめる（書いた中身を失う所を手作業の確かめだけに任せない）。タスクの数は増やさない。
