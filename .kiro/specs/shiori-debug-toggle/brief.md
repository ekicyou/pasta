# Brief: shiori-debug-toggle

## Problem

ゴーストの作者（開発者）が、マニュアルの「デバッグの有効化」を読んで、手元でデバッグを入れたり切ったりしたい。今の口は 2 つだけで、どちらも手間がかかる。

- `pasta.toml` の `[debug] enabled = true`: 配布する設定ファイルを書き換えることになり、戻し忘れるとデバッグを入れたまま配ってしまう。
- 環境変数 `PASTA_DEBUG`: ベースウェアを起動し直さないと効かない。

VSCode 拡張と pasta-recipe は「SHIORI をリロード」を呼べるが、そのときにデバッグを入れる手段がない。デバッグが無効なゴーストにシーンの再生などを頼むと、「pasta.toml に [debug] enabled = true を書いてリロードしてください」（`editors/shared/ghostFiles.ts` の `DEBUG_HINT`）と案内され、作者がファイルを手で書き換えに行く。

## Current State

- 有効かどうかは、SHIORI の読み込みのたびに 1 回だけ、`crates/pasta_lua/src/debug/config.rs` の `resolve_enabled` が決める。順は `PASTA_DEBUG` ＞ `[debug] enabled` ＞ false。呼ぶのはローダの Phase 5.5（`crates/pasta_lua/src/loader/mod.rs`）で、シーンマップを作るかどうかもここで決まる。Lua のランタイムはまだ起きていない。
- グローバル変数（`＄＊名前`）は、シーンからは `SAVE` として見え、SHIORI を下ろすとき（ランタイムの Drop）に保存され、次の読み込みで戻る。`profile/` の中に保存されるので配布物には入らない。
- シーンマップ（`SourceMap`）は Phase 5.5 で作り、ランタイムを作るときに渡している（`runtime/factory.rs`）。そのため今は、Lua が起きる前に有効かどうかが決まっている。
- `OnPastaSetSaveVariable`（`pasta/shiori/event/external_control.lua`）は、外から保存変数を書く。デバッグが無効でも効く。型は Reference2 の `number`・`string`・`boolean`・`delete`。pasta-recipe は `editors/shared/dumpVariables.ts` の `setSaveVariable` から使っている。
- リロードは `editors/shared/reloadShiori.ts` の `reloadAndWait`（MCP の `reload` → pasta.log の読み込み完了の印を待つ）。デバッグが有効なら、読み込みのときに pasta.log に `Debug enabled: development events are allowed` が 1 行出る。
- マニュアルは `book/src/debug/index.md` の「デバッグの有効化」節（pasta.toml・環境変数・優先順位・確かめ方）。

## Desired Outcome

- 保存変数 `＄＊デバッグ` に `true` を入れておくと、次の読み込みからデバッグが有効になる。`false` なら、`pasta.toml` で有効にしていても無効になる。英語名の `＄＊debug` も同じに効く（英語圏の作者向け。議案 6）。
- VSCode 拡張から「SHIORI デバッグを有効にする」「SHIORI デバッグを無効にする」を選ぶと、保存変数を書いてリロードし、効いたかを結果に出す。
- pasta-recipe では、リロードの隣のトグルのボタンで同じことができる。
- デバッグが無効なときの案内（`DEBUG_HINT`）は、ファイルの書き換えでなくトグルを案内する。
- マニュアルの「デバッグの有効化」に、保存変数とトグルの使い方が載る。

## Approach

今ある部品をつなぐ。新しく作るのは、読み込みのときにグローバル変数 `＄＊デバッグ` で有効かどうかを決める所だけ。値は、シーンから見える `＄＊デバッグ` と同じものを使う。どのファイルにどう保存されているか（書式・gzip）には触れない。

```text
「SHIORI デバッグを有効にする」
  → OnPastaSetSaveVariable（デバッグ, true, boolean）   ← 今ある
  → reload（下ろすときにグローバル変数が保存される）     ← 今ある
  → 読み込みで ＄＊デバッグ を見て有効かどうかを決める   ← 新しい
  → pasta.log の「Debug enabled …」の行で効いたかを見る ← 今ある
```

**設計の議題に残すこと**: 今は Lua が起きる前（Phase 5.5）に有効かどうかを決め、シーンマップをランタイムを作るときに渡している。`＄＊デバッグ` を見るには、判断を Lua が起きた後へ移すことになる。シーンマップを作る時期と渡し方、`@pasta_debug` の有効の読み方を、無効なときに余計なものを作らないという制約を保って設計で決める。

discovery の議案（2026-11-19 開発者が推奨案で承認）:

1. **優先の順**: `PASTA_DEBUG` ＞ `＄＊デバッグ` ＞ `[debug] enabled` ＞ false。`pasta.toml` はゴースト全体に書いて配る設定、保存変数は開発者の手元のスイッチ。「無効にする」で pasta.toml で入れたデバッグも切れるように、保存変数を pasta.toml より上に置く。
2. **効く時期**: 次の読み込みから。シーンの中で `＄＊デバッグ` を書き換えても、そのセッションでは変わらない（pasta.toml を書き換えたときと同じ）。
3. **値の型**: 真偽値の `true`・`false` だけを数える。ほかの型（`"true"` という文字列や `1`）は書いていないのと同じにして、警告を 1 行出す。入れる口は pasta-recipe の変数の表、`OnPastaSetSaveVariable` の `boolean`、Lua（`SAVE.デバッグ = true`）。DSL には真偽値の書き方が無い。要件のディスカッションで、もう一度判断する議題に残す。
4. **道具の形**: トグル。VSCode はコマンドを 2 つ（有効にする・無効にする）、pasta-recipe はリロードの隣にボタンを 1 つ。どちらも `setSaveVariable` → `reloadAndWait` の順で呼び、リロードの後の pasta.log の行で状態を決める。流れは `editors/shared/` に 1 つ置き、両方から使う。「リロードのときにデバッグを ON にする」案は採らない（口が 2 つになるだけで、作りはトグルと同じ）。
5. **マニュアル**: 「デバッグの有効化」に「保存変数による有効化」と「道具のトグル」の節を足し、優先順位を 4 段に直す。`node book/tools/gen-skill-refs.mjs` でスキルの `references/` を作り直す。
6. **英語名の別名**（2026-11-19 開発者「英語圏の対応もあるから、`debug` も見るようにしてほしい」。推奨案で記録。要件のディスカッションで確かめる）: 読み込みのときに `＄＊デバッグ` と `＄＊debug`（Lua では `SAVE.デバッグ`・`SAVE.debug`）の両方を見る。決め方は「`デバッグ` が真偽値ならそれ。無ければ `debug` が真偽値ならそれ。どちらも無い（真偽値でない）なら書いていないのと同じ」。優先の順の `＄＊デバッグ` の段は、この 2 つを 1 段として数える（`PASTA_DEBUG` ＞ `＄＊デバッグ`／`＄＊debug` ＞ `[debug] enabled` ＞ false）。
   - **道具が書くのは `デバッグ` だけ**: 道具が `debug` を書いたり消したりしなくても、「無効にする」が手書きの `debug = true` に負けない（`デバッグ` が常に先に効く）。英語名だけ書いた作者の手書きも効く。
   - **両方が食い違うとき**: `デバッグ` が勝つ。前例は `GLOBAL.pause_auto_talk` と `GLOBAL["自動トーク停止"]`（日英の対。`pasta/shiori/entry.lua`）。
   - **警告**: 真偽値でない値の警告（議案 3）は、どちらの名前かを出す。
   - **読む側は 1 か所**: 判断の関数が 2 つの名前を順に見るだけ。ほかの名前（`Debug`・`DEBUG`・`dbg` など）は見ない。
   - **気をつけること（要件で確かめる）**: `debug` は、作者が自分のゴーストの印に使っている名前かもしれない。そのゴーストが `＄＊debug ＝ true` を持つと、pasta のデバッグが入る（開発用のイベントが効き、127.0.0.1 の待ち受けが開く）。真偽値だけを数え、待ち受けは 127.0.0.1 固定なので害は小さいが、マニュアルの「デバッグの有効化」に「`debug` という名前は pasta のものとして読む」と 1 行書く。

## Scope

- **In**:
  - 読み込みのときに `＄＊デバッグ` と英語名の `＄＊debug` で有効かどうかを決める（無い・真偽値でないなら書いていないのと同じ。両方あれば `デバッグ` が勝つ）。優先の順の変更、判断の時期の移し替え、単体テスト（2 つの名前の組み合わせを表で）。
  - `editors/shared/` のトグルの流れ（保存変数を書く → リロード → 状態を返す）とテスト。
  - VSCode 拡張の 2 つのコマンド（`package.json` の登録、日本語の表示名、結果の表示）。
  - pasta-recipe のトグルのボタン（リロードの隣）と、押した後の状態の表示。
  - `DEBUG_HINT` の文の変更。
  - マニュアル（`book/src/debug/index.md` ほか、トグルに触れる `dev-actions.md`・`recipe.md`・`troubleshooting.md`）と、スキルの `references/` の作り直し。
- **Out**:
  - `PASTA_DEBUG` と `[debug] enabled` の読み方の変更。
  - 動いている最中にデバッグを入れたり切ったりすること（シーンマップの作り直し）。
  - 保存変数の名前を pasta.toml で変える口。
  - DSL に真偽値の書き方を足すこと。

## Boundary Candidates

- 読み込みの判断（`pasta_lua` の `debug/config.rs` とローダの Phase 5.5）
- 道具の共通の流れ（`editors/shared/`）
- 道具ごとの UI（`editors/vscode/`・`editors/recipe/`）
- マニュアル（`book/src/debug/`）

## Out of Boundary

- デバッグのイベント（`OnPastaSceneAt`・`OnPastaDumpVariables`）の中身。
- `OnPastaSetSaveVariable` の振る舞い（今のまま使う）。
- グローバル変数の保存の仕組み（書式・場所・書く時期）。

## Upstream / Downstream

- **Upstream**: グローバル変数（`＄＊名前`・`SAVE`）、`OnPastaSetSaveVariable`（`external-control-events`）、`reloadAndWait`（`vscode-mcp-client`）、pasta-recipe の開発の道具（`pasta-recipe-dev-actions`）。
- **Downstream**: 開発の道具でデバッグを前提にする後の機能は、無効なときにトグルを案内できる。

## Existing Spec Touchpoints

- **Extends**: なし（デバッグの有効化を決めた spec・`external-control-events`・`pasta-recipe-dev-actions` はどれも `completed/`）。
- **Adjacent**: `novel-checkpoint-rollback`（状態の巻き戻し）。2026-11-19 の discovery で、ロードと巻き戻しで戻るのは名前が「スロット」か `slot_` で始まるグローバル変数だけと決まった（`novel-variable-scope` は閉じた。roadmap の「統合して閉じたもの（2026-11-19）」）。`＄＊デバッグ` は戻らない側に入る。

## Constraints

- 決めるのは読み込みのときの 1 回だけ、という今の形を保つ。デバッグが無効なら、シーンマップを作らず、生成した `.lua` も変わらないこと。
- グローバル変数の保存の仕組み（ファイルの書式・場所・gzip・壊れたファイルの退避）には手を入れない。
- デバッグの待ち受けは 127.0.0.1 固定・単一 DLL・opt-in のまま。`OnPastaSetSaveVariable` は前からいつでも効くので、外から有効にできることで開く口は増えない。
- `cargo test` は環境変数に左右されない（`PASTA_DEBUG` の `#[ctor]` の中和）。`＄＊デバッグ` で決まることのテストも、置いた場所に左右されないように走らせる。
- マニュアルが権威。挙動と同じ変更でマニュアルとスキルの `references/` を直す。

## 申し送り（2026-11-19 manual-reader-first から）

マニュアルのデバッグの部を書き直した。この spec が直すマニュアルと文言は、次の今の形に合わせる。

- 部の題は「編集ツールとデバッグ」。章は `debug/index.md`「編集ツールとデバッグの概要」・`debug/recipe.md`「pasta-recipe で書く」・`debug/vscode-setup.md`「VS Code で書く」・`debug/dev-actions.md`「▶・リロード・保存変数」。見出し「デバッグの有効化」（`debug/index.md`）は残してあり、優先順位の節は 1 文に畳んだ。案内役は emo2 の 2 人（むらさき・エモ）で、本体にも台詞を置ける。
- ゴーストが返したさくらスクリプトの呼び名を「台本」から「スクリプト」に改めた。エディタの文言（`editors/shared/runSceneAtCursor.ts` の `NO_SCRIPT_GUIDE` ほか）も「スクリプト」になっている。`DEBUG_HINT` を直すときは同じ語にそろえる。
- hello-pasta と入門用のスターター（`pasta-starter.nar`・`my-ghost`）は、`pasta.toml` に `[debug] enabled = true` を入れて配る。入門は、配る前に 14 段目で `false` にする手順である。トグルを足すなら、入門の「▶ が効かないとき」（`getting-started/01-boot.md`・`debug/recipe.md` の「3 手で最初の ▶」）と 14 段目の手順も見直す。
- 実機で分かったこと: 開発者の機械には環境変数 `PASTA_DEBUG=1` が入っているので、`[debug] enabled` だけの効き目は実機で見られなかった。この spec の確かめでは環境変数を外して立てる。
