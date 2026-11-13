# ギャップ分析: pasta-recipe-dev-actions

- **Feature**: `pasta-recipe-dev-actions`
- **日付**: 2026-11-13（ブランチ `claude/pasta-recipe-dev-actions-0a553d`、base `93fa65c4`＝`pasta-recipe` #255 の後）
- **Discovery Scope**: Extension（既存の拡張の流れを共有へ移し、既存のアプリに画面とネイティブのコマンドを足す）
- **要件**: `requirements.md`（Requirement 1〜13。仮置きは 2026-11-13 の要件ディスカッションで確定）

## 要約

- **移す 5 つは今も `vscode` を import しないが、Node の `http`・`fs`・`path`・`Buffer` を直に使う**。入出力の面は brief の見込み（`post`・`readText`・`mtime` の 3 つ）では足りず、実際に使っているのは「1 回の POST（状態の番号・本文・時間切れ・書く前に切れたかの区別）」「フォルダの一覧」「バイト列の読み取り（ずらして読む）」「大きさ・更新時刻・ファイルか」「パスの組み立て」の 5 種。パスの区切りを変えると拡張の出力の文言（`pasta.log を読めなかった…（C:\…）`）が変わるので、組み立ても面に入れるか、区切りを保つ小さな関数が要る。
- **「相手の探し方」の半分と、出力の見出し・案内の文言は `devCommands.ts`（vscode の糊）の中にある**。覚えと確かめ（`remembered`・`identify`→外れたら `findBasewares`）・同時の止め（`busy`）・見出しの行（`clock()`・`BAD_DOCUMENT`・`NOT_FOUND`…）を共有へ出さないと、Requirement 10.1（相手の探し方・文言も 1 本）を満たせない。選ぶ画面（QuickPick）と保存（`TextDocument.save`）と進み具合（`withProgress`）だけを差し込みにする形になる。
- **アプリ側の型の検査が、移すコードをそのままでは通さない**。`editors/recipe/tsconfig.json` は `verbatimModuleSyntax: true`・`types: ["vite/client"]`（Node の型なし）で、画面の試験は `node --test`（型を外すだけ・拡張子付きの import が要る）。今の 5 つは型と値を混ぜた import（`import { CallResult, callTool }`）と拡張子なしの import なので、共有へ移すときに `import type` と `.ts` 付きの import へ書き直す必要がある。拡張の `tsconfig.json`（`rootDir: "src"`）・`eslint src` は `editors/shared/` を見ない。
- **ネイティブの側は、新しい crate を足さずに済む見込み**。`reqwest 0.13.5` が tauri の依存として `src-tauri/Cargo.lock` に既にあり（TLS なし）、`toml 1`・`pasta_dsl`（`pasta_lsp` の依存。`parser::is_id` が公開）も lock にある。読む範囲は、今の `paths::resolve_rel` が `profile` を断る（pasta-recipe 11.4）ので、開発の操作のための別の門（開いている master の中の決まった 4 か所だけ）が要る。
- 規模は **L**（11〜14 タスク。共有へ移す・面 2、拡張の差し替えと試験 2、ネイティブのコマンド 2、▶・リロード・出力 2、表と書き換え 2、値を入れる窓 1〜2、マニュアル 1）、リスクは **Medium**（拡張の振る舞いを変えずに移すことと、2 つの型の設定を両方通すことが主な危うさ。外の技術は既知）。

## 1. 今の状態

### 1.1 移す 5 つ（`editors/vscode/src`）

| ファイル | 行 | 使う入出力 | 移すときの注意 |
| -------- | -- | ---------- | -------------- |
| `mcpClient.ts` | 199 | `http.request`（`agent: false`・`Content-Length` に `Buffer.byteLength`・`setTimeout`・`finish` の前後で `unreachable`/`noResponse` を分ける） | `rpc` だけが入出力。ほか（`callTool`・`probe`・`identify`・`findBasewares`・`parseLogEntries`・`readRaiseEvent`・`fixEscapes`）は純粋。`ToolArgsMap.raise_event.event` は `'OnPastaSceneAt' \| 'OnPastaDumpVariables'` だけなので `OnPastaSetSaveVariable` を足す。`clientInfo.name` は `'pasta-vscode'` 固定 |
| `reloadShiori.ts` | 140 | `path.join`、`ghostFiles` 経由の読み取り | 純粋。`Timing`・`FlowContext`（`endpoint`・`ghost`・`timing`・`sleep`）が流れの文脈。ここへ入出力の面を 1 つ足すのが素直 |
| `runSceneAtCursor.ts` | 108 | `ghostFiles` 経由 | `runScene(ctx, fsPath, line1)` は Reference2 を送らない。`sceneReferences` は 2 要素の組を返す。Reference2 を付ける引数を足しても、拡張は渡さないので振る舞いは変わらない |
| `dumpVariables.ts` | 64 | `path.join`、`ghostFiles` 経由 | 純粋。返すのは TOML の文字列そのもの（`shown.text`）。表にするのはアプリだけの関心 |
| `ghostFiles.ts` | 263 | `fs.readdir`・`fs.readFile`（バイト列と UTF-8）・`fs.stat`（`size`・`mtimeMs`）・`fs.open`→`handle.stat().isFile()`→`handle.read(buf, 0, n, start)`・`path.relative`/`dirname`/`basename`/`isAbsolute`/`join` | `decodeDescriptName` は `TextDecoder('shift_jis')`（WebView2 の Chromium も Node の full-icu も持つ）。`readNewLines` は `Buffer.lastIndexOf(0x0a)` と `toString('utf8')` → `Uint8Array` と `TextDecoder` で書ける |

試験（`editors/vscode/src/test/`）は、実物の一時フォルダと、`http` で立てる偽の MCP の窓口（`fakeMcpServer.ts` 224 行）に対して流れを通す形。偽の入出力（メモリの上）の試験は今は無い。

### 1.2 VS Code の糊（`devCommands.ts` 209 行）

- `resolveTarget`（`(Uri, line0)` を 1 始まりへ）・`isPasta`/`isGhostSource`・`saveGhostDocuments`（ルートの下の未保存の `.pasta`・`.lua`・`pasta.toml`）・`chooseEndpoint`（覚え・確かめ・探し直し・QuickPick）・`execute`（`busy`・見出しの行・保存の失敗で止める・`withProgress`・通知の誤り/警告の分け `ERROR_KINDS`）。
- 文言の定数（`BAD_DOCUMENT`・`NOT_IN_GHOST`・`BUSY`・`NOT_FOUND`・`NOT_FOUND_CHECKS`・`SAVE_FAILED`）と `clock()` もここ。
- `findGhostMaster` は `ghostValidator.ts`（vscode に依存）にある。アプリでは開いている master が分かっているので要らない。

### 1.3 アプリ（`editors/recipe`）

- **ネイティブ（`src-tauri/src/commands.rs` 480 行）**: `AppState(Mutex<Option<Ghost>>)` が開いている master と見張りを持つ。相対パスのコマンドは `files::*` の中で `paths::resolve_rel` を通し、最初の部品が `profile` なら断る（pasta-recipe 11.4）。重い仕事は `run_blocking`（`spawn_blocking`・パニックを `Err` へ）。HTTP の部品は無い。`Cargo.toml` の依存は tauri・single-instance・dialog・notify・toml・serde・serde_json・pasta_check（fs）・pasta_lsp・windows-sys。
- **画面（`src/`）**: `ipc.ts` がコマンドを 1 対 1 で包む（失敗は `Err(String)` の文）。`main.ts` が開く流れ（`opening` の鎖・`generation`・`switching`）とツールバーの「保存」を持つ。`tree.ts` の `#openMenu` はシーンの行では操作が 0 で何も出さない（ここへ ▶ と「値を入れて実行…」を足す）。ツリーの行の `line` は 1 始まり（`#activate` が Monaco の `lineNumber` にそのまま渡す）。`panels.ts` は `#panels`（高さ 180px の横並びの flex）に「診断」「ゴースト全体の検査」の 2 つの `.panel` を置く。`dialog.ts` は `<dialog>` の showModal（3 択・名前の入力・知らせ）。`editor.ts` は Monaco を作る。CodeLens は `features/register.all` に `codelensController` が入っているので、`registerCodeLensProvider` で足せる（今は使っていない）。
- **CSP**: `connect-src ipc: http://ipc.localhost`。画面からは 127.0.0.1 へ届かない（ネイティブから送るので CSP は変えない）。capabilities は `core:default`・`dialog:allow-open`・`core:window:allow-destroy`（アプリ自身のコマンドは許可の表なしで呼べる今の形）。
- **CI（`build.yml` の `recipe`）**: `npm ci` → `npx tsc --noEmit` → `npm test`（`node --test "src/**/*.test.ts" "scripts/*.test.mjs"`）→ `npm run build` → `cargo test --locked` → clippy → `tauri build --no-bundle`。依存の検査は `cargo deny check (pasta-recipe)`。**拡張の `npm test`（`test:unit`・`test:e2e`）を走らせる job は無い**（手元だけ）。

### 1.4 ランタイムの口（変えない）

- `OnPastaSceneAt`（`scene_play.lua`）: Reference0＝ファイル、Reference1＝行、Reference2＝TOML のインライン表（`scene-play-locals` で実装済み）。デバッグが要る。位置は `resolve_position` が `base_dir.join(file)` → `std::path::absolute` で引く（相対でも絶対でも通る）。
- `OnPastaDumpVariables`: デバッグが要る。`profile/pasta/debug/variables.toml` に保存変数をテキストの TOML で書く（`persistence::dump_table`。書けない値はキーを外して警告）。
- `OnPastaSetSaveVariable`（`external_control.lua`）: デバッグの有無によらず働く。Reference2 の型（`string`・`number`・`boolean`・`delete`）。成功は 204、失敗は表記（`【数にできない値：…】` など）を返す。名前は区切らない（`a.b` は 1 つの名前）。書き出しはランタイムの終了時（ゴーストの終了・辞書の再読込。`book/src/grammar/variables.md` の「グローバル変数の保存先」）。

## 2. 要件と資産の対応

| 要件 | 使える資産 | ギャップ |
| ---- | ---------- | -------- |
| 1 ▶ で鳴らす | `runScene`・`formatScene`（そのまま）、`tree.ts` のメニュー、`Documents.save`、`analyze` | **Missing**: ツリーのシーンの操作、見出しの CodeLens、保存した中身での見出しの今の行（1.4。タブの解析は書き換えの後おおむね 1 秒遅れる）。**Constraint**: 送るファイルのパスの形（master は `std::path::absolute` 由来。ランタイムも同じ関数）。 |
| 2 リロード | `reloadAndWait`・`formatReload` | **Missing**: ツールバーのボタン（`#toolbar` へ append。開始の画面では出さない）。 |
| 3 相手 | `findBasewares`・`identify`・`CANDIDATE_PORTS` | **Missing**: 覚えと確かめの共有化（今は `devCommands.ts`）、選ぶ画面（2 つ以上のとき）。**Constraint**: `ghost_name` はルートの絶対パス。`descript.txt` の判定は `resolveGhost` の中にあり、ファイルのパスを前提にしている（アプリは master から作る口が要る）。 |
| 4 出力の欄 | `formatReload`/`formatScene`/`formatDump` の `{ lines, notice }`、`.panel` の見た目 | **Missing**: 欄そのもの・見出しの行（`clock()` などは `devCommands.ts`）・誤りと警告の分け（`ERROR_KINDS`）。**Constraint**: 高さ 180px の横並びに 2 つ足すと、1000px 幅で 1 つあたり約 250px（台本の行が折れる）。 |
| 5 重なり | `busy`（`devCommands.ts`）、`main.ts` の `opening` の鎖 | **Missing**: 開発の操作とゴーストの切り替えの順。 |
| 6 表 | `dumpVariables`（文字列を返す）、ネイティブの `toml` | **Missing**: TOML を名前・型・値の行に分けること（画面に TOML の読み手が無い）、表の欄、注記。 |
| 7 書き換え | `callTool('raise_event', …)`・`readRaiseEvent` | **Missing**: `OnPastaSetSaveVariable` の流れ（成功＝`noScript`、失敗＝`script` が表記）、編集・削除の確かめ・追加の入力。 |
| 8 値を入れて実行 | `scene-play-locals` の Reference2、`pasta_dsl::parser::is_id`、`dialog.ts` | **Missing**: 組の入力の窓、インライン表の組み立て（`'…'` のリテラルは `'` を含められない・改行も不可 → 基本の文字列 `"…"` のエスケープか、ネイティブの `toml` で書く）、送る前の弾き。 |
| 9 範囲 | `paths::resolve_rel`（profile を断る） | **Missing**: 開発の操作のための読み取りの門（master の中の `descript.txt`・`pasta.toml`・`pasta.log`（`profile/` の下の設定どおりの場所）・`profile/pasta/debug/variables.toml` だけ）、POST の門（`127.0.0.1` と候補のポートだけ・`Origin` なし）。**Constraint**: pasta-recipe 要件 1.3（つながない）・11.4（profile を読まない）は「この spec の機能では」と範囲を切ってあるので矛盾はしないが、マニュアルの「ネットワークにはつながない」と steering（`tech.md` の「外へつながない CSP」「シーンの実行…は持たない」）は直す。 |
| 10 1 本・拡張は不変 | 5 つのファイル | **Constraint**: 拡張の出力の文言（パスの区切りを含む）・待ちの長さ・`clientInfo` を変えない。`package.json` の `test:unit` が `./src/test/…` を名指し。`vsce-4-upgrade` も `package.json`・`package-lock.json` を触る（同じウェーブに置かない）。 |
| 11 見た目 | `style.css` の `:root` の変数・`.panel` | **Unknown**: 4 つの一覧（診断・検査・出力・変数）を 180px にどう収めるか（タブで切り替えるか、高さを変えるか）。 |
| 12 マニュアル | `book/src/debug/recipe.md`（「まだできないこと」「締めの台詞」）・`dev-actions.md`（拡張） | **Missing**: 「ゴーストを動かす」節。スキルの参照資料に入る章かどうかは `gen-skill-refs` の対象の表で確かめる（Research Needed）。 |
| 13 試験 | 拡張の試験（実物の fs と偽の窓口）、`node --test` | **Missing**: 偽の入出力の試験、ネイティブの門の試験、インライン表の組み立ての試験。**Constraint**: 拡張の試験は CI で走らない。共有の試験をアプリの `npm test` の glob に入れれば CI で走る。 |

## 3. 入出力の面（移す 5 つが実際に使うもの）

| 種類 | 今の呼び方 | 面に要る形（見込み） |
| ---- | ---------- | -------------------- |
| POST | `http.request` 1 回・`agent: false`・`setTimeout` | `post(port, body, timeoutMs)` → `{ status, body }` か、`unreachable`（書く前に切れた）／`noResponse`（書いた後に切れた・時間切れ）の区別。宛先は `127.0.0.1`・パスは `/api/mcp/v1` に固定し、URL を渡さない（門を狭くする） |
| 一覧 | `fs.readdir(master)` | `list(dir)` → 名前の配列か undefined（`descript.txt` を大文字・小文字を問わず探すため） |
| バイト列 | `fs.readFile`（descript）・`fs.open`+`read`（pasta.log の続き） | `read(path, offset?)` → `{ bytes, size }` か undefined（ファイルでない・無い） |
| 状態 | `fs.stat`（`size`・`mtimeMs`）・`isFile()` | `stat(path)` → `{ size, mtime, isFile }` か undefined |
| UTF-8 | `fs.readFile(p, 'utf8')` | `read` と `TextDecoder` で足りる |
| パス | `path.join`・`relative`・`dirname`・`basename`・`isAbsolute`・`sep` | `join` を面に入れる（拡張は `path.join`、アプリは `\`）か、master の区切りを引き継ぐ小さな関数。`resolveGhost` の上り（`relative`）は拡張の糊に残す手もある |

時計は今も `sleep` の差し込みで、時刻は `Date.now()`（`probe` の締め切り）だけ。両方の環境にある。

## 4. 実装の選択肢

### Option A: 拡張のファイルを `editors/shared/` へそのまま移し、`import` を直すだけ（入出力は `FlowContext` に 1 つの面を足す）

- 5 つを `editors/shared/` へ移し、Node の直の呼び出しを `ctx.io.*` へ置き換える。`mcpClient` の `rpc` は `io.post` を呼ぶ。拡張は `editors/vscode/src/nodeGhostIO.ts`（`http`・`fs/promises`・`path`）を作って `FlowContext` に入れる。アプリは `editors/recipe/src/tauriGhostIO.ts`（`invoke`）とネイティブの 4〜5 個のコマンド。
- `devCommands.ts` の覚え・確かめ・`busy`・見出しの行・文言を、`editors/shared/devSession.ts` のような 1 つにまとめ、選ぶ画面・保存・進み具合・書き出し先を差し込みにする。
- ✅ 1 本になる範囲が Requirement 10.1 と一致。試験は今の拡張の試験が Node の実装の試験として残り、偽の入出力の試験を足す。
- ❌ `import type`・`.ts` 付きの import に書き直すので、拡張側の `tsconfig.json`（`rootDir`・拡張子付き import の許可）・`eslint` の対象・`package.json` の試験の束ね方も触る。

### Option B: 流れだけ共有し、相手の覚えと文言は拡張とアプリで別に持つ

- 共有は 5 つのファイルだけ。`devCommands.ts` はそのまま。アプリは同じ文言を自分で書く。
- ✅ 拡張の糊に触れない（`devCommands.e2e.test.ts` も無傷）。
- ❌ 相手の探し方の後半と文言が 2 本になり、Requirement 10.1 を満たさない（brief の「いずれずれる」を一部で残す）。

### Option C: 流れを共有し、TOML の読み書きと名前の判定はネイティブに置く（A か B の上に重ねる）

- `variables.toml` の行分け（`toml::Table` → 名前・型・値）、インライン表の組み立て（`toml::Value` の `to_string`）、名前の判定（`pasta_dsl::parser::is_id`）をネイティブのコマンドにする。
- ✅ ランタイムと同じ決まり・同じ TOML の書き手。画面に TOML の部品を足さない（npm の依存が増えない）。エスケープの抜けが起きにくい。
- ❌ 画面の 1 回の操作ごとに IPC の往復が増える（小さい）。試験はネイティブ（`cargo test`）側になる（Requirement 13.2 はそれで満たせる）。

**HTTP の送り手の候補（ネイティブ）**: (1) `reqwest 0.13`（既に lock にある・`default-features = false` で TLS なし・`Client::builder().no_proxy()`・接続の失敗と時間切れを見分けられる）、(2) `ureq`（新しい crate・`deny.toml` と第三者の表示の更新が要る）、(3) `std::net::TcpStream` で HTTP/1.1 を手で書く（依存なし・chunked の応答を読む手間）。いずれも `Origin` は付けない。

## 5. 規模とリスク

- **Effort: L** — 移すこと自体は機械的だが、2 つの型の設定・2 つの試験の束ね方・ネイティブのコマンドと門・4 つの画面・マニュアルが重なる。
- **Risk: Medium** — 外の技術は既知（Tauri のコマンド・reqwest・Monaco の CodeLens）。危うさは、拡張の振る舞い（文言・パスの区切り・待ち）を変えずに移すことと、拡張の試験が CI に無いこと。

## 6. 設計への申し送り

### 推奨の向き（決めるのは設計）

- Option A ＋ C。共有の面は「POST・一覧・読む・状態・join」の 5 つを目安に、流れの文脈（`FlowContext`）に入れる。
- ネイティブの門は、今の `resolve_rel`（profile を断る）と別の小さな関数にし、開いている master の下の決まった場所だけを通す。POST は宛先とポートを Rust 側で固定する（画面に URL を渡させない）。
- 共有の偽の入出力の試験は、アプリの `npm test`（`node --test`）の glob で拾える場所に置くと CI で走る。拡張の試験は今の `test:unit` の束ね方に共有の場所を足す。
- 下の欄は、4 つの一覧を横に並べるより、見出しの帯で切り替える形のほうが 1000×680 に収まりやすい（Research Needed）。

### Research Needed

1. `reqwest 0.13` を `default-features = false` で足したとき、`Cargo.lock` に新しい crate が増えないか（tauri が有効にしている features と合うか）。増えるなら `deny.toml`・`about.toml` を確かめる。
2. 拡張の `tsconfig.json`（`module: commonjs`・`rootDir: "src"`・宣言を出す）で、`editors/shared/` の `.ts` 付きの import をどう許すか（`allowImportingTsExtensions` は `noEmit` が要る。`rewriteRelativeImportExtensions`、または `tsc` を拡張では走らせていないので `rootDir` を外すだけで足りるか）。`eslint src` の対象に `../shared` を入れるか。
3. 画面の `tsc --noEmit`（TypeScript 7.0.2）と `node --test`（Node 22）で、共有のファイルが両方通る書き方（`import type`・拡張子・`Uint8Array`/`TextDecoder` だけを使う）。
4. Vite が `editors/recipe` の外（`../shared`）を開発の窓で配れるか（`server.fs.allow`）。`editor.ts` が既に `../../vscode/syntaxes/*.json` を読んでいるので、同じ形で通る見込み。
5. 保存した中身での見出しの今の行（Requirement 1.4）を、保存の後にもう一度解析して取るか、保存の版とタブの解析の版を照らすか。ツリーの行の鍵は `rel|名前|n`（同じ名前の n 番目）。
6. `variables.toml` の値の型の見分け（TOML の整数・浮動小数点数・日付（読み込みで文字列化されているはず）・入れ子の表・配列）と、`"a.b"` のような引用符付きのキーの名前をそのまま `OnPastaSetSaveVariable` の Reference0 へ返せるか。
7. マニュアルの `debug/recipe.md` が、スキルの参照資料（`gen-skill-refs`）の生成の対象に入っているか。
8. 送るファイルのパスの綴り（ドライブ文字の大小・8.3 の短い名前）が、ランタイムのシーンマップの照合と合うか。拡張は VS Code の `fsPath` を送り、アプリは `std::path::absolute` 由来の master を送る（CI の `RUNNER~1` の前例）。
9. 操作の途中で別のゴーストを開いたとき（要件 5.2）: 開き直しを操作の終わりまで待たせるか、操作が始まったときの master を持ち続けてネイティブの門をその master で通すか。待たせると最長で数十秒（読み込みの終わり 20 秒＋頼み 15 秒）止まる。`main.ts` の `opening` の鎖・`generation` との組み方もあわせて決める。

### 隣の spec

- `vsce-4-upgrade`: `editors/vscode/package.json`・`package-lock.json` が重なる。同じウェーブに置かない。
- `pasta-recipe`（completed）: 要件 1.3・11.4 は「この spec の機能では」と切ってあるので書き換えない。完了のときに steering（`tech.md` の pasta-recipe の節・`structure.md` の `editors/`）を直す。

## 7. 設計の調べ（2026-11-13・`/kiro-design`）

- **Discovery Scope**: Extension（light discovery。外の新しい部品は無いので、Web の調べはせず、手元のコードと cargo の置き場の crate のソースで確かめた）
- **使ったスキル・決まり**: `kiro-spec-design` の `design-principles`・`design-discovery-light`・`design-synthesis`・`design-review-gate`。ponytail（最小の完全な変更）。

### 7.1 Research Needed の答え

| # | 問い | 答え | 設計への効き |
| - | ---- | ---- | ------------ |
| 1 | reqwest を足して crate が増えるか | `Cargo.lock` の `reqwest 0.13.5` は tauri の **Android・iOS 向けだけ**の依存（`tauri-2.12.5/Cargo.toml` の `[target.'cfg(any(target_os = "android", …))'.dependencies.reqwest]`・`default-features = false`・`json`・`stream`）。lock の包みは増えないが、Windows では reqwest と hyper・hyper-util・tower・http-body-util などが**新しく compile される**。ライセンスは MIT・Apache-2.0（`deny.toml`・`about.toml` の許可の表に入っている）。`cargo about` は `--target x86_64-pc-windows-msvc` で自動で拾う。reqwest 0.13.5 には `ClientBuilder::retry`・`retry::never()`・`no_proxy`・`http1_only`・`pool_max_idle_per_host`・`Error::is_connect`・`is_timeout` がある | reqwest を `default-features = false` で直接の依存に足す（OPEN QUESTION 3） |
| 2 | 拡張の `tsconfig.json` で `.ts` 付きの import をどう許すか | 拡張は `tsc` を走らせない（`compile` は esbuild・`lint` は型を見ない `typescript-eslint` の recommended）。`out/` は esbuild の出力。`tsconfig.json` は手元のエディタの型の表示にだけ効く | `noEmit`・`allowImportingTsExtensions` を足し、`rootDir`・`outDir`・`declaration*` を外し、`include` に `../shared/**/*`。`eslint src` は `../shared` を見ない（基準の外のファイルは無視される）。共有の型はアプリの `tsc --noEmit` が見る |
| 3 | 画面の `tsc` と `node --test` の両方を通す書き方 | アプリは `verbatimModuleSyntax`・`allowImportingTsExtensions`・`types: ["vite/client"]`（vite の型が node の型を引く）。`node --test` は型を外すだけ | 共有は `import type` と `.ts` 付きの相対 import・enum なし・`Uint8Array`/`TextDecoder` だけ |
| 4 | Vite が `editors/recipe` の外を配れるか | `editor.ts` が既に `../../vscode/syntaxes/*.json`・`../../vscode/package.json` を読んでいる（許された所から import されたファイルは配られる）。本番は束ねるので関係ない | `server.fs.allow` は触らない |
| 5 | 保存した中身での見出しの今の行 | タブの解析は書き換えから 200ms ＋往復遅れる。ツリーの行の鍵は `rel|名前|n` | 保存の後に、保存した中身（タブの `diskText`・無ければディスク）を解析し直し、`ScenePath`（名前と同じ名前の中の順）で引く。鍵の文字列は戻さず、`TreeRow` に `scene` を足す |
| 6 | `variables.toml` の型 | `persistence.rs` の `lua_value_to_toml_strict`: String・Integer（整数の Lua の数）・Float（それ以外）・Boolean・Array（列）・Table。Datetime は書かない。最上位のキーが保存変数。`external_control.lua` は名前を区切らない（`require("pasta.save")[name] = value`） | 型は 4 つ（数＝整数と浮動小数点数、表＝配列と表）。`"a.b"` のキーはそのまま Reference0 に返す |
| 7 | マニュアル `debug/recipe.md` がスキルの参照資料の対象か | `book/tools/gen-skill-refs.mjs` の `GENERATION_MAP`（24 件）に `debug/` の章は無い | 照合は変わらない。リンクの検証だけ |
| 8 | 送るファイルのパスの綴り | `playscene.rs` の `resolve_position` は `std::path::absolute`（canonicalize しない）で、区切りと Windows の大小は `SourceMap::scene_at` が揃える。アプリの master も `std::path::absolute` 由来（ディスクの綴りのまま） | 絶対パスを `\` で結んで送る。8.3 の短い名前の違いは拡張と同じ危うさで、増えない |
| 9 | 途中のゴーストの切り替え | 待たせると最長で数十秒止まる。`main.ts` の `opening` の鎖は開く流れだけ | 待たない。操作は始めの `GhostTarget` を持ち、門は今の master だけを通す。操作はアプリ全体で 1 つずつ（OPEN QUESTION 4） |

ほかに確かめたこと:

- `node --test "src/**/*.test.ts" "../shared/*.test.ts"` は、`package.json` の無いフォルダの `.ts`（型を外す・`import` で ESM と見る）も含めて通った（手元の Node 26。CI は Node 22 なので実装で確かめる。通らなければ `editors/shared/package.json` に `{"type":"module"}` を置く）。
- Monaco 0.57 に `languages.registerCodeLensProvider`・`editor.registerCommand` と、`features/register.all` に入る `codelens.showLensesInCurrentLine`（「Show CodeLens Commands For Current Line」）がある。
- 開始の画面の間は `style.css` の `body:has(#start:not([hidden])) > :not(#start, …)` がツールバーごと隠すので、リロードのボタンは何もしなくても出ない（2.5）。
- `is_local_var_name` は `pasta_dsl::parser::is_id`（pest の `id = !(reserved_id) ~ identifier`）。TS で写すとずれるので Rust で呼ぶ。`pasta_dsl` は lock にある（`pasta_lsp`・`pasta_check` の依存）。
- `OnPastaSetSaveVariable` の成功は `return nil`（204）、失敗は `FAILURE.reply` の表記。SecurityLevel が external のときも 204 なので、成功と見分けられない（表を読み直して確かめる）。
- 拡張の試験は CI で走らない（`build.yml` に拡張の `npm test` の job は無い。`release.yml` は `vsce package` だけ）。`.vsix` は esbuild の束ねなので `../shared` を中に入れなくてよい（`.vscodeignore` は `src/**`・`**/*.ts` を外す）。

### 7.2 合成（generalization・build vs adopt・simplification）

- **一般化**: 6 つの操作（▶・値を入れて実行・リロード・表示・書き換え・削除と追加）は、どれも「相手を決める → 見出し → （保存）→ 流れ → 報告」の同じ手順。拡張の `execute` の後半を `DevSession.execute(cmd, ghost)` として共有し、操作は `DevCommand { title, saves, run }` を足すだけにした。書き換え・削除・追加は 1 つの `setSaveVariable(ctx, name, change)` にまとめた。
- **採る・作る**: HTTP は reqwest を採る（lock にある・接続の失敗と時間切れを見分けられる・chunked の応答を自分で読まなくてよい）。TOML は `toml` crate（ランタイムと同じ）、名前は `pasta_dsl::parser::is_id`（ランタイムと同じ）。窓は `dialog.ts` の `open` を広げて使う（新しい窓の部品を作らない）。下の欄は `.panel` の見た目をそのまま使う。
- **削ったもの**: 共有の側の `dirname`・`basename`・`relative`（ルートは Rust・拡張が渡す）、拡張の古いファイルの写し（re-export）、アプリの `clientInfo` の名前の切り替え、画面の TOML の部品、拡張の CI の job、`OnPastaScene` の画面、表の欄を開いたときの自動の読み込み。

### 7.3 設計の決定

#### 決定: 入出力の面は 5 つ（post・list・stat・read・join）
- **Context**: 移す 5 つが使う Node の入出力（2 章の 3 の表）。
- **Alternatives**: (1) brief の 3 つ（post・readText・mtime）、(2) 5 つ、(3) Node の `fs` の形を写す。
- **Selected**: (2)。`readText`・`mtimeOf`・`openLogCursor`・`readNewLines`・`decodeDescriptName` は共有の中で 5 つの上に書く。`join` を面に入れ、拡張は `path.join` のまま（出力の文言のパスの区切りが変わらない）。
- **Trade-offs**: 関数に `io` を渡すので、拡張の試験の呼び方が変わる（確かめることは変わらない）。

#### 決定: 相手の選び方と文言は `devSession.ts` で共有
- **Context**: 要件 10.1（相手の探し方・出力の行と案内の文言も 1 本）。
- **Selected**: `devCommands.ts` の `chooseEndpoint`・`execute` の後半・文言の定数・`clock()`・`ERROR_KINDS` を移す。vscode に固有のもの（対象の文書・`accepts`・`BAD_DOCUMENT`・`NOT_IN_GHOST`・QuickPick・保存・進み具合・通知）は `DevHost` で差し込む。
- **Follow-up**: `devCommands.e2e.test.ts` を変えずに通すことで、拡張の振る舞いが変わらないことを確かめる。

#### 決定: Rust の読み取りの門は「直下の descript.txt・pasta.toml と profile の下のファイル」
- **Context**: 要件 9.4（読むのは 4 つ）と 2.4（`pasta.log` の場所は `pasta.toml` の設定で `profile/` の下のどこでもよい）。
- **Alternatives**: (1) `profile/` の下のファイル全部（logger.rs の場所の決まりをそのまま門にする）、(2) Rust でも `pasta.toml` を読んで `pasta.log` の場所を決める（TS の読み方とずれると読めなくなる）、(3) 画面が渡す場所を信じる。
- **Selected**: (1)。共有の流れが読むのは 4 つだけで、門はその場所の決まりと同じ幅。
- **Trade-offs**: 画面から求めれば `profile/pasta/save/` の保存も読める（読むだけ・開いているゴーストの中）。要件 9.4 の書き方を、門の形に合わせて直した（2026-11-13 設計ディスカッション）。

#### 決定: 途中で別のゴーストを開いても待たせず、操作は前のゴーストのまま最後まで走らせる
- **Context**: 要件 5.2・Research Needed 9。2026-11-13 設計ディスカッション（議題 2・開発者の承認）。
- **Alternatives**: (1) 門が今の master だけを通す（前のゴーストの操作は読み取りを断られ、約 20 秒の後に紛らわしい文で終わる）、(2) 開き直しを操作の終わりまで待たせる（最長 35 秒ほど何も起きない）、(3) 切り替えを見分けて打ち切る（頼んだ再生の結果を捨てる）、(4) 門が「進んでいる操作の master」を通す。
- **Selected**: (4)。操作はアプリ全体で 1 つずつなので、覚えは 1 つ。`dev_begin` がそのとき開いている master を覚え、`dev_end` が消す。表へ当てる前にゴーストの鍵を比べる。
- **Trade-offs**: コマンドが 2 つ増える。前のゴーストの操作が終わるまで、新しいゴーストの開発の操作は始まらない。

#### 決定: 拡張の試験を CI の job に足す
- **Context**: 要件 13.3・10.2。2026-11-13 設計ディスカッション（議題 3・開発者の承認）。この spec は、拡張の試験が守っているコードそのものを移す。
- **Selected**: `build.yml` に job `vscode-test`（`editors/vscode` で `npm ci` → `npm test`）を足す。e2e も偽の `vscode` で Node だけで走る。
- **Follow-up**: ランナーで `test:grammar` などが走るかは実装のときに確かめる。

#### 決定: 自明として閉じたもの（2026-11-13 設計ディスカッション）
- HTTP の送り手は reqwest（lock に既にある。chunked・SSE の応答を手で読まない）。
- `clientInfo.name` は `io.clientName`（拡張 `pasta-vscode`・アプリ `pasta-recipe`）。
- `readText` は BOM を外す（BOM 付きの `pasta.toml` の `[logging]` の読み落としを、共有の 1 本で直す。要件 10.2 に但し書き）。
- 数の読み方は i64 か有限の f64。保存の後にシーンが見つからなければ送らずに止める（1.4）。

#### 決定: 下の欄は 4 つの切り替え
- **Context**: 要件 11.1（1000×680 でエディタを残す）・11.2（一覧と同じ見た目）。180px の横並びに 4 つ置くと 1 つ約 250px。
- **Selected**: 高さ 180px のまま、見出しの帯に 4 つのボタンを置き、中身を 1 つずつ見せる。開発の操作は出力へ切り替える（焦点は動かさない）。
- **Trade-offs**: 今の「診断と検査を同時に見る」形が変わる（OPEN QUESTION 1）。

#### 決定: 表の読み直しは同じ操作の中で
- **Context**: 要件 6.9・7.6。
- **Selected**: ▶・値を入れて実行（鳴らしたとき）と書き換え・削除・追加の `run` の終わりで `dumpVariables(ctx)` を呼ぶ。見出しは 1 つ・相手の確かめも 1 回。
- **Trade-offs**: 表の読み直しの「書き出されなかった」も、その操作の行と知らせに混ざる。

## 8. 設計のリスクと手当て

- 拡張の振る舞いを移すときにずらす — `devCommands.e2e` と 5 つの試験を変えずに（呼び方だけ直して）通す。共有の流れの試験をアプリの CI で走らせる。
- reqwest で Windows の exe が大きくなる・compile が増える — `package.ps1` の 30 MB の上限の検査がそのまま見張る。
- CI の Node 22 で `../shared` の `.ts` が読めない — 手元で通ることは確かめた。通らなければ `editors/shared/package.json`（`"type": "module"`）。
- CodeLens はマウスの操作 — キーボードはツリーのメニューと Monaco の「Show CodeLens Commands For Current Line」で届く（実機で確かめる）。
