# Brief: pasta-recipe-dev-actions

> **ステータス**: 未着手（2026-11-12 の `/kiro-discovery` で起票。`pasta-recipe` の依頼のうち、ゴーストを動かす機能を分けた）。版は 0.5.3（開発者の承認）。要件定義は Opus（Fable なし）。

## Problem

`pasta-recipe` は編集はできるが、書いたシーンを鳴らして確かめる手段が無いと、作者は結局 SSP の右クリックや VS Code に戻ることになる。VS Code 拡張には、シーンの実行・SHIORI のリロード・保存変数の表示があるが、VS Code の外では使えない。

また、作者からは次の 2 つも求められている（開発者の依頼）。

- グローバル変数（`＄＊`）を見るだけでなく、その場で書き換えたい。
- シーンを呼ぶときに、ローカル変数を入れてから鳴らしたい。

## Current State

2026-11-12（main `5c27b869`）に確かめた。

- **拡張の実行系**: `editors/vscode/src` の次の 5 つで、どれも `vscode` を import しない。
  - `mcpClient.ts`（199 行）: Node の `http` で `http://127.0.0.1:{port}/api/mcp/v1` へ JSON-RPC を送る。ポート 9801〜9810・9821〜9830 を並べて探す。`Origin` はわざと送らない（areka は loopback 以外の Origin を 403 にする）。
  - `reloadShiori.ts`（140 行）
  - `runSceneAtCursor.ts`（108 行）: `OnPastaSceneAt` に Reference0＝ファイルの絶対パス、Reference1＝行を付けて送る。
  - `dumpVariables.ts`（64 行）: `OnPastaDumpVariables` を送り、`profile/pasta/debug/variables.toml` の更新を待って読む。
  - `ghostFiles.ts`（263 行）: Node の `fs/promises` で `descript.txt`・`pasta.log`・`variables.toml` を扱う。
  - VS Code との糊は `devCommands.ts` だけ。
- **使っている MCP の道具**: `get_property`・`get_log`・`reload`・`raise_event`。
- **保存変数の書き換え**: ランタイムには `OnPastaSetSaveVariable` がある（`pasta_scripts/pasta/shiori/event/external_control.lua`）。Reference0＝名前、Reference1＝値、Reference2＝型（`number`・`string`・`boolean`・`delete`）。メモリの上で書き換え、ファイルへは終了時の自動保存で書く。拡張はこれを使っていない。
- **ローカル変数を入れた実行**: 今は無い。`OnPastaScene`・`OnPastaSceneAt` は引数を取らない。0.5.3 で先に入る `scene-play-locals` が口を足す（2026-11-12 の要件で確定: Reference2 に TOML のインライン表。日本語のキーは引用符で包む `{ "名前" = "ぱすた" }`。値は文字列・数・真偽、日付・時刻は文字列として入る。配列・表は誤り。組み立てる側で TOML として正しく書く）。
- ブラウザの中からは MCP にも保存変数のファイルにも届かない。ネイティブの層（Tauri の Rust の側）が要る。

## Desired Outcome

- `pasta-recipe` のツリーのシーンの ▶（とエディタの中の見出しの ▶）で、そのシーンが VS Code 拡張と同じ流れで鳴る。流れは、保存 → SHIORI の読み込み直し → `OnPastaSceneAt` → 出力の欄に台本、の順。
- 「SHIORI をリロード」がアプリから使える。
- 保存変数の表が出て、値を書き換え・削除できる（`OnPastaSetSaveVariable`）。
- ローカル変数の名前と値を入れてからシーンを鳴らせる（`scene-play-locals` の口）。
- MCP クライアントと実行の流れの実装は 1 本だけで、VS Code 拡張と `pasta-recipe` が同じものを使う。拡張の振る舞いは変わらない。

## Approach

**実行の流れの TS を `editors/shared/` に移し、入出力を小さな面で差し込む**（discovery で開発者が承認した推奨案）。

```ts
interface GhostIO {
  post(url: string, headers: Record<string, string>, body: string): Promise<string>;
  readText(path: string): Promise<string | null>;
  mtime(path: string): Promise<number | null>;
}
// VS Code 拡張 → Node の http・fs/promises で実装
// pasta-recipe → Tauri の invoke で Rust のコマンドへ（Rust の側は reqwest か ureq と std::fs）
```

- 面の名前と関数の数は、移す 5 つのファイルが実際に使う入出力から決める（要件・設計）。上は見込み。
- 拡張は esbuild で、アプリは Vite で、どちらも `editors/shared/` を相対パスで取り込む。npm のワークスペースは作らない。
- MCP は Rust の側から送るので、CORS とローカルネットワークアクセスの許可は関係しない。`Origin` を送らない今の決まりは、Rust の側でも守る（Tauri の plugin-http は使わない）。

退けた案: 実行の流れを Rust へ移植する案。同じ流れが TS と Rust の 2 本になり、いずれずれる。

### discovery で決めたこと（2026-11-12 開発者の承認）

1. 実行の流れは TS のまま共有し、入出力だけを差し込む。
2. ローカル変数を渡す口は、実行時の別の spec（`scene-play-locals`）が持つ。こちらは入力の画面と、その口を呼ぶことだけを持つ。

## Scope

- **In**:
  - `editors/shared/`: `mcpClient`・`reloadShiori`・`runSceneAtCursor`・`dumpVariables`・`ghostFiles` を移し、入出力の面を差し込めるようにする。
  - `editors/vscode`: 共有のものを Node の実装で使うよう差し替える。コマンド・メニュー・出力パネルの振る舞いは変えない。今のテストは通ったままにする。
  - `editors/recipe`: 入出力の面を Rust のコマンドで実装する。画面は次の 4 つ。
    - シーンの ▶（ツリー・エディタの見出し）
    - リロード
    - 出力の欄（台本・ログ）
    - 保存変数の表（表示・書き換え・削除）
  - ローカル変数を入れて鳴らす画面（名前・値の組を入れる）。`scene-play-locals` の口を呼ぶ。
  - 保存変数の書き換えでは、表（テーブル）の値は書き換えの対象外にする（`OnPastaSetSaveVariable` が扱うのは数値・文字列・真偽だけ）。表は見るだけにする。
  - 書き換えはメモリの上で効き、ファイルへは終了時の自動保存で書かれることを、画面かマニュアルで伝える。
  - マニュアル: `pasta-recipe` の章に、実行・リロード・変数の使い方を足す。
  - テスト: 共有の流れを、偽の入出力で確かめる。拡張とアプリの両方の入出力の実装も確かめる。
- **Out**:
  - ランタイムの変更（`scene-play-locals` が持つ）。
  - VS Code 拡張への新しい機能（保存変数の書き換え・ローカル変数を入れた実行を拡張にも足すかは、別に決める）。
  - `OnPastaScene`（名前で鳴らす）の画面。

## Boundary Candidates

- 共有の流れ（`editors/shared/`）と、入出力の実装（Node と Tauri）。
- 実行の画面（▶・出力の欄）と、変数の画面（保存変数の表・ローカル変数の入力）。

## Out of Boundary

- `pasta-recipe` のエディタ・ツリー・保存（`pasta-recipe` が持つ）。
- `OnPastaSceneAt` の行の解釈と、ローカル変数を受けるランタイムの側。

## Upstream / Downstream

- **Upstream**: `pasta-recipe`（アプリの本体・ツリー）、`scene-play-locals`（ローカル変数を受ける口）、`vscode-scene-outline-play`（2026-11-12 に実装済み。`pasta.runSceneAtCursor` は `executeCommand(id, uri, line0)` の引数を受ける。受け取りは `devCommands.ts` の `resolveTarget`（0 始まりを 1 始まりに直す）に置き、`runSceneAtCursor.ts` の `runScene(ctx, fsPath, line1)` の形は変えていない。シーンの並びは `pasta_lsp::analysis::AnalysisEngine::analyze` の `scenes`（`SceneNode`）を Rust から直接使える。こちらはその後に移す）。
- **Downstream**: なし（拡張にも変数の書き換えを足す話は、ここから生まれうる）。

## Existing Spec Touchpoints

- **Extends**: なし（`vscode-mcp-client` は completed）。
- **Adjacent**: `vscode-scene-outline-play`（`devCommands.ts`・`runSceneAtCursor.ts` を触る。同じウェーブに置かない）。`vsce-4-upgrade`（`editors/vscode/package.json`）。

## Constraints

- VS Code 拡張の振る舞いを変えない（ポートの探し方・`Origin` を送らないこと・ログの待ち方）。
- MCP の相手は SSP と areka の両方。今の拡張が通っている道（`get_property` で baseware を確かめる）をそのまま使う。
- `OnPastaSceneAt`・`OnPastaSetSaveVariable` は、デバッグが有効で、SecurityLevel が external でないときだけ効く。効かないときの知らせ方は、拡張と揃える。

## 申し送り（scene-play-locals より）

2026-11-12 に `scene-play-locals` が実装済み。契約の正本はマニュアル `book/src/lua/shiori-events.md` の `OnPastaScene`・`OnPastaSceneAt` の節と「シーンを再生するイベントの共通の決まり」の「ローカル変数を入れる（Reference2）」。画面を組むときに見直す論点:

- キーは `＄名前` と書ける名前だけ（`"1"`・`"__x__"`・`"a b"` は「名前にならない値」）。判定は `@pasta_debug.is_local_var_name` と同じ決まり（`pasta_dsl::parser::is_id`）なので、入力の欄で先に弾くならこれに揃える。
- 誤りは 3 種（表にできない値・名前にならない値・代入できない値）で、1 件だけを表記とログの警告 1 行で返し、鳴らさない。文言の実例はマニュアルの誤りの表。配列 `[1]` も「表にできない値」。
- グローバルシーンだけを指名して値を入れるときは、`OnPastaScene` の Reference1 を `""` にする。値の文字列は TOML のリテラル文字列 `'…'` で組み立てると `\` と `"` を気にしなくてよい。
- 値はシーンが見つかったときだけ入り、次のイベントには持ち越さない。2^53 を超える整数は丸まる。

## 2026-11-12 棚卸の再測定（main 2381bf52）

- **前提の変化**: 起票の後に `vscode-scene-outline-play`（#252）と `scene-play-locals`（#253）が入り、上の Upstream と申し送りのとおりになっている。移す 5 つ（`mcpClient.ts` 199・`reloadShiori.ts` 140・`runSceneAtCursor.ts` 108・`dumpVariables.ts` 64・`ghostFiles.ts` 263 行）は今も `vscode` を import しない。`runScene(ctx, fileFsPath, line1)` は `runSceneAtCursor.ts` 34 行、受け取りの `resolveTarget` は `devCommands.ts` 50 行（209 行）。新しい `sceneOutline.ts`（60 行）は `vscode` を使う糊なので移さない。`runScene` は Reference2 をまだ送らない（ローカル変数を渡す引数はこの spec が足す）。分け目（ランタイムは `scene-play-locals`、アプリの本体は `pasta-recipe`）はそのまま。
- **触るファイル**: 新しく `editors/shared/`（上の 5 つと、その試験 `editors/vscode/src/test/` の `mcpClient`・`ghostFiles`・`dumpVariables`・`runSceneAtCursor`・`reloadShiori`・`fakeMcpServer`）。既存は `editors/vscode/src/devCommands.ts`、`editors/vscode/package.json`（231 行の `test:unit` などが `./src/test/…` を名指しする）・`tsconfig.json`（`rootDir: "src"`・`include: ["src/**/*"]`）・`eslint src`、`editors/recipe/`（Rust の入出力のコマンドと画面）、マニュアルの `pasta-recipe` の章。1,000 行に近いものは無い。
- **規模**: 11〜14 タスク（共有へ移す・入出力の面 2、拡張の Node の実装と試験 1〜2、Rust の入出力のコマンド 2、▶・リロード・出力の欄 2、保存変数の表と書き換え 2、ローカル変数の入力 1〜2、マニュアル 1）。
- **先に要るもの**: 働きの上では `pasta-recipe`（ほかの 2 本は入った）。ファイルの重なり（未完了）: `pasta-recipe`（`editors/recipe/`・マニュアルの章。順に置くので問題ない）、`vsce-4-upgrade`（`editors/vscode/package.json`・`package-lock.json`）。
- **種別**: 機能（VS Code の外でシーンを鳴らす・保存変数を書き換える・ローカル変数を入れて鳴らす。開発者の依頼）。
- **要件定義のモデル**: Opus。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - 入出力の面の見込み（`post`・`readText`・`mtime` の 3 つ）では足りない。移す 5 つは `fs.readdir`・バイト列の `readFile`（`descript.txt` の名前の読み取り）・`fs.stat` の大きさ・`fs.open` してずらして読む（`pasta.log` の続き）も使う。`reloadShiori.ts`・`dumpVariables.ts`・`ghostFiles.ts` は Node の `path` を import するが、Vite の中には無いので、パスの組み立ても面に入れるか、小さな自前の関数にする。
  - Constraints の「`OnPastaSetSaveVariable` はデバッグが有効なときだけ効く」は誤り。`external_control.lua` 8 行のとおり、デバッグの有無によらず働き、SecurityLevel が external のときだけ何もしない（204）。デバッグが要るのは `OnPastaScene`・`OnPastaSceneAt`・`OnPastaDumpVariables`（`scene_play.lua` 10 行）。

## 申し送り（pasta-recipe の要件ディスカッションより・2026-11-12）

- アプリの名前は `pasta-recipe`（exe は `pasta-recipe.exe`）で確定。
- アプリは単一インスタンス（2 つ目の起動は今の窓へパスを渡す。`pasta-recipe` の要件 2.11）。この spec で別の窓や別の起動の口を足さない。
- ツリーには、ファイル・フォルダの作成と名前の変更の操作が入る（要件 4.14〜4.17。削除は無い）。▶ などのシーンの操作は、その操作と並べて足す。
- ゴースト全体の検査のファイル集めは `pasta_check` のライブラリの opt-in の機能へ移る（要件 9.12）。ゴーストのファイルを Rust で集めるなら、これを使う。
