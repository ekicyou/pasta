# Brief: external-control-events

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票）。0.4.2 に入れる（2026-10-19 開発者の決定）。要件定義は Opus。

## Problem

SSP の MCP（`raise_event`）などの外部からゴーストを試験・操作する人が困る。

- 辞書の `＞XXX` と同じ呼び出しを、外から 1 回で実行する口が無い。いちばん欲しいのは、試験の間だけ `＞自動トーク停止`／`＞自動トーク再開`（`ontalk-suppression`）を呼んで、ランダムトークと時報の割り込みを止めることである。
- グローバル変数（`＄＊名前`、Lua の `save.名前`）を外から書く口も無い。たとえば、試験の間だけ `＄＊pasta_talk_interval_min` を縮めたい。

## Current State

2026-10-19（main `c1e2d978`）に確かめた。

- **外から呼ぶ今の手段**（どちらもこの用途に合わない）
  - 未登録のイベントは、同じ名前のシーンへ回る（`EVENT.no_entry` → `SCENE.co_exec(act, act.req.id, nil, nil)`。`pasta_scripts/pasta/shiori/event/init.lua`）。呼びたい名前ごとに `＊OnFoo` を 1 つずつ書く必要がある。
  - VSCode のシーンキック（`pasta_scripts/pasta/shiori/event/kick.lua` の `KICK`）はデバッグ専用で、シーン表を完全一致で引くだけである。`GLOBAL` の関数（`自動トーク停止` など）・前方一致・別名表は通らない。
- **`＞XXX` の解決**: `ACT_IMPL.call(self, global_scene_name, key, attrs, ...)`（`pasta_scripts/pasta/act.lua` L695）が `find_handler("scene", key)` で探し、可変長の引数をハンドラに渡す。シーンの外（グローバル）から呼んだときは、グローバルシーンの前方一致・`GLOBAL` の関数・`[scene.alias]` の別名表を通る。
- **既定のハンドラの置き方**: `REG.OnBoot = REG.OnBoot or function(act) ... end` の形で、`event/boot.lua`・`choice_select.lua`・`second_change.lua`・`mouse.lua` に置き、`event/init.lua` L78-81 で `require` する。作者は `REG.On…` を上書きして差し替えられる。
- **イベント名の先例**: `OnPastaCallBack{N}`（コールバック）・`OnPastaClock`（仮想時刻の読み取り）。どちらも `OnPasta` の接頭辞である。
- **グローバル変数**: `＄＊名前` は Lua の `save.名前`。`save` テーブルごと、Lua ランタイムの終了時に `profile/pasta/save/save.json` へ保存される。予約変数（`＄＊pasta_talk_interval_min`／`_max`）は数値でないと無視される（マニュアル `grammar/variables.md`）。
- **JSON**: Lua から `@json`（mlua-stdlib。既定で有効）の `json.decode` が使える。失敗すると `nil, エラーメッセージ` を返す。

## Desired Outcome

外から SHIORI のイベントを 1 つ送るだけで、辞書の `＞XXX` の呼び出しと、グローバル変数の代入ができる。

```
OnPastaCallFunction   Reference0=自動トーク停止
  → 辞書に「＞自動トーク停止」と書いたのと同じ効き目（応答は 204）
OnPastaCallFunction   Reference0=初回起動
  → ＊初回起動 を再生（応答は 200 でさくらスクリプト）
OnPastaCallFunction   Reference0=好感度を足す  Reference1=10
  → GLOBAL["好感度を足す"](act, "10") と同じ（引数は文字列のまま）

OnPastaSetSaveVariable  Reference0=pasta_talk_interval_min  Reference1=60
  → save.pasta_talk_interval_min = 60（数値）
OnPastaSetSaveVariable  Reference0=会員番号  Reference1="01"
  → save.会員番号 = "01"（文字列。数値の 1 に化けない）
OnPastaSetSaveVariable  Reference0=初回済み  Reference1=null
  → save.初回済み を消す（値なしに戻す）
```

## Approach

2026-10-19 の話し合いで、開発者が決めた（議案 1〜17 のすべての推奨）。

1. **spec の置き場所**: 新しい spec。`ontalk-suppression` は「自動トークのスイッチ」、この spec は「外から pasta を操作するイベントの口」で、責任が別である。呼び出しと変数の代入は同じ口なので、1 本にまとめる（最初の名前の案 `call-function-event` から改名）。
2. **既定のハンドラ**: `REG.OnPastaCallFunction`・`REG.OnPastaSetSaveVariable` を、ほかの既定のハンドラと同じ形（`REG.X = REG.X or function`）で、`pasta.shiori.event` の新しいファイルに置く。いつでも有効で、設定は足さない。SSTP の SEND で任意のさくらスクリプト（終了の `\-` も含む）を流せるのと同じ信頼の範囲である。止めたい作者は、ハンドラを上書きすれば止められる。
   - 却下した案: `PASTA_DEBUG` のときだけ有効（MCP の試験を、デバッグを有効にしない配布版で走らせられなくなる）、`pasta.toml` で opt-in（設定が 1 つ増えるだけ）。
3. **`OnPastaCallFunction`**
   - 名前: 利用者の案のまま。`OnPastaCall` は `OnPastaCallBack{N}` と頭が同じで紛らわしいので避けた。
   - Reference0 は呼ぶ名前（`＞` の後ろに書く名前）。Reference1 以降は、そのまま文字列の引数として渡す（`act:call` の可変長の引数）。
   - 探し方は、シーンの外（グローバル）に書いた `＞XXX` と同じ（グローバルシーンの前方一致・`GLOBAL` の関数・`[scene.alias]` の別名）。ローカルシーンは呼べない。完全一致のキックとは、わざと揃えない。
   - 見つからないときは、警告のログ（例 `seam=call_function.unresolved name=…`）を残して 204。エラーの応答にはしない。
   - 結果は普通のイベントと同じ道（`EVENT.fire` → 必要なら `EVENT.drive`）で返す。Status（`talking` など）による抑止はかけない（外から明示的に呼んだため）。
4. **`OnPastaSetSaveVariable`**
   - 名前: `OnPastaSetVariable` だとローカル変数（`＄名前`）も書けるように読めるので避けた。
   - Reference0 は変数の名前。`save` の一段目のキーだけを扱う（`a.b` のような入れ子のパスは扱わない）。`pasta_` で始まる予約変数も書ける。
   - Reference1 は JSON の値として読む（`60`・`"お嬢様"`・`true`・`null`）。`null` なら変数を消す。JSON として読めなければ、値を変えずに警告のログを残す。
     - 却下した案: 「数字の形なら数値」（`"01"` が `1` に化けて戻せない）。送るのは MCP などの機械なので、JSON を書くのは苦にならない。
   - ディスクへの保存は、辞書の `＄＊名前＝値` と同じく終了時の自動保存に任せる（イベントのたびに書き出さない）。
   - 応答は、成功でも失敗でも 204。失敗は警告のログに残す。

## Scope

- **In**
  - 2 つの既定のハンドラ（`OnPastaCallFunction`・`OnPastaSetSaveVariable`）と、その読み込み（`event/init.lua`）
  - Lua のテスト（`crates/pasta_lua/tests/lua_specs/`）。呼び出し（シーン・`GLOBAL` の関数・別名・引数・見つからない名前）と、代入（数値・文字列・真偽値・`null`・読めない JSON・名前の欠け）
  - マニュアル（SHIORI イベントの節 `lua/shiori-events.md` に 2 つの節、変数の章 `grammar/variables.md` から代入のイベントへの案内）と、スキルの references の作り直し
- **Out**
  - 値の読み出し（外から今の値を問い合わせること）。欲しくなったら、そのときに起票する
  - 呼び出しの結果を値で返すこと（`GLOBAL` の関数の戻り値を応答に載せるなど）
  - ローカルシーンの呼び出し、ローカル変数・`STORE` の書き換え、入れ子のパス
  - SSP の MCP サーバーの側の変更
  - 有効・無効を切り替える設定

## Boundary Candidates

- 外からの操作のイベントの既定のハンドラ（`pasta.shiori.event` の新しいファイル）
- 呼び出しの解決は既存の `act:call` に任せる（新しい探し方を作らない）
- 変数の代入と値の読み取り（`@json` の `decode`）
- マニュアルとスキルの references

## Out of Boundary

- 自動トークを止めるスイッチそのもの（`ontalk-suppression`）
- VSCode のシーンキック（`KICK`。デバッグ専用で、探し方も別）
- グローバル変数の保存の仕組み（`@pasta_persistence`・`save.json`）

## Upstream / Downstream

- **Upstream**: `scene-play-event`（0.4.2。先に入る。外からのイベントの既定のハンドラの置き方・マニュアルの既定ハンドラーの表・見つからないときの出し方の先例になる）、イベントの振り分け（`pasta.shiori.event` の `EVENT.fire`・`EVENT.drive`・`REG`）、`＞` の呼び出しの解決（`act:call`・`find_handler`・`[scene.alias]`）、`@json`
- **Downstream**: MCP からの試験の手順（`ontalk-suppression` の `自動トーク停止` を外から呼ぶ）。将来の値の読み出しのイベント

## Existing Spec Touchpoints

- **Extends**: なし（新しい境界）
- **Adjacent**
  - `scene-play-event`（0.4.2。2026-10-19 起票）: この spec の**前に入る**（Dependencies に置いた。開発者の指示で、どちらが先でもよいが入口が近いため順番を決めた）。
    - 共有するもの: `pasta/shiori/event/` に新しいモジュールを置く形、`event/init.lua` L78-81 の `require`、マニュアル `lua/shiori-events.md` の既定ハンドラーの表。ハンドラから先の Lua のコードは別物である。
    - 役目の違い: `OnPastaScene` はデバッグのときだけ、書いた名前どおりの完全一致でシーンを再生する（ローカルシーンも指せる）。`OnPastaCallFunction` はいつでも、辞書の `＞XXX` と同じ探し方（前方一致・`GLOBAL` の関数）で呼ぶ。マニュアルでは、2 つの違いが 1 か所で分かるように書く。
    - 要件で決めること: 見つからないときの出し方を、あちらの「失敗を出す仕組み（`failure-output-unification`・`runtime-error-balloon`）」に揃えるか。推奨は揃えること（ログには必ず残り、バルーンに出るのはデバッグのときだけなので、Approach の「エラーの応答にはしない」と矛盾しない）。
  - `ontalk-suppression`（0.4.2）: 主な呼び先（`自動トーク停止`／`自動トーク再開`）を作る。この spec の実装は、それが無くても動く（呼び先が見つからなければ 204）。共有するのはマニュアルの `lua/shiori-events.md` だけで、節は別である。並べられる。後から入る側がマニュアルの差分を合わせる。
  - `pasta-check-release-asset`（0.4.2）: ファイルを共有しない。並べられる。
  - `ponytail-lua-scripts`（ponytail・隙間の spec）: `pasta_scripts/` の既存のファイルを触る。この spec は新しいファイルと `event/init.lua` の `require` の 1 行だけなので、ぶつかりは小さい。

## Constraints

- 既存のイベントの挙動は何も変えない。今のテストは、そのまま通ること。
- 非 ASCII の識別子は、ブラケット記法で扱う（luacheck との互換のため）。
- 規模は 5〜6 タスクの見込み（ハンドラ 2、テスト 1〜2、マニュアルとスキル 1）。
