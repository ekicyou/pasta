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

## 2026-10-22 棚卸の再測定（main 6a8dc479）

- **前提の変化**: 2026-10-19 の後、イベントの振り分け（`pasta_scripts/pasta/shiori/event/init.lua` 267 行）と `＞` の解決（`pasta_scripts/pasta/act.lua` の `ACT_IMPL.call` L695）は変わっていない。Current State は今も正しい。
- **触るファイル**: 新しいハンドラのファイル（`pasta/shiori/event/` の下。新規）、`event/init.lua` の読み込みの 1 行（L78〜81）、Lua のテスト（新しいファイルと一覧 `crates/pasta_lua/tests/lua_specs/init.lua` の 1 行）、マニュアル（`lua/shiori-events.md` の既定ハンドラの表と 2 節、`grammar/variables.md`）。失敗の種類を足すなら `pasta_scripts/pasta/failure.lua`。
- **規模**: 5〜6 タスク。
- **先に要るもの**
  - `scene-play-event`: `event/init.lua` の読み込みの所を共有するので、同じウェーブに置けない。働きの上では待つ必要は無い（呼び先が無くても動く）。順は入れ替えてもよく、先に入る側がハンドラの置き方と失敗の出し方の先例になる。
  - `chain-talk-interrupt`: これも `event/init.lua` を共有する（あちらは `set_co_scene`・`EVENT.drive`、こちらは読み込みの 1 行だけ）。厳密には同じウェーブに置けない。読み込みの 1 行を、マニュアルの同じページのように「後から入る側が合わせる」扱いにすれば並べられる。
  - `ontalk-suppression`・`pasta-check-release-asset`: ソースを共有しない。並べられる。
  - 0.4.2 の後に入る作業では、`event/init.lua` を `recommend-menu-scenes`・`shell-element-click`・`scene-exact-match`・`novel-talk-flow`・`novel-checkpoint-rollback`・隙間の `ponytail-lua-scripts` が触る。
- **種別**: 機能（外から `＞XXX` を呼ぶ口と、グローバル変数を書く口。2026-10-19 開発者の決定）。
- **要件定義のモデル**: Opus。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - `ACT_IMPL.call` はハンドラを直接呼ぶ。呼んだシーンが `＞チェイントーク` で区切ると、コルーチンの外での中断になってエラーで止まる。ハンドラは呼び出しをコルーチンで包んで返し、`EVENT.fire` → `EVENT.drive` で回す形にする必要がある。設計で決める。
  - 見つからないとき、`ACT_IMPL.call` は自分で失敗の表記（「シーンが見つからない」。`failure.lua` L19）を積み、警告のログも出す（`act.lua` L716〜）。Approach の「警告のログを残して 204」と食い違う。要件で、この表記に任せるか（推奨。`scene-play-event` の失敗の出し方とも揃う）を決める。
  - MCP の `raise_event` が NOTIFY で送るなら、応答は捨てられる（`crates/pasta_shiori/src/actor/thread.rs` L204〜210）。`＊初回起動` の台本は返らないが、`自動トーク停止` や変数の代入は効く。`scene-play-event` が実機で確かめる結果に合わせて書く。

## 申し送り（ontalk-suppression より、2026-10-22 完了時）

- 呼び先の `GLOBAL` の 4 つの名前（`pause_auto_talk`・`自動トーク停止`・`resume_auto_talk`・`自動トーク再開`）は入った。引数 `act` は `nil` でも動き、出力を積まない。再開は `act.req.date` が取れればその時刻で時計を決め直し、取れなければ時報を初回扱いにする（`.kiro/specs/completed/ontalk-suppression/design.md` の TimerRestart）。外からのイベントの `act` にも `date` があるので、そのまま効く。
- `pasta_check` の組み込みの名前は 8 つになった（`builtins.rs` の `GLOBAL_NAMES`）。

## 申し送り（scene-play-event より、2026-10-22 完了時）

- 実機（SSP 2.9・MCP）で、`raise_event` は **GET**・`SecurityLevel: local`・`SenderType: external,sstp` で届いた。応答の Value の台本はツールの結果に載る。上の「NOTIFY なら応答は捨てられる」の心配は当たらない。`＊初回起動` の台本も返る前提で書いてよい。
- MCP の `reload` は読み込み直しを頼むだけで、終わるのを待たない。終わる前の `raise_event` は SHIORI に届かず「台本なし」で返り、204 と見分けがつかない。マニュアルの例は「`get_status` が答えてから呼ぶ」とした（`book/src/lua/shiori-events.md` の「MCP からの呼び方」）。
- 既定のハンドラの形は `pasta/shiori/event/scene_play.lua`（`REG.X = REG.X or function`、門を引数より前、失敗は `FAILURE.report` → `act:build()`、成功はコルーチンを返して `EVENT.fire` に任せる）。`event/init.lua` の `require` の並びに 1 行足した。写すときの参照先は `.kiro/specs/completed/scene-play-event/design.md` の ScenePlayHandler。
- `SCENE.co_exec_exact(name, local_name)` は 2 つ目の戻り値（`"global"`・`"local"`）を返すようになった。`＞XXX` の探し方を「シーンと同じ」にするなら、この口と `SEARCH:search_local_scene_exact` を使える。

## 2026-10-23 棚卸の再測定（main 8fc9bdff）

- **前提の変化**: `scene-play-event` と `ontalk-suppression` が入り、待つものは無くなった。先例の形は、シーンを再生する既定のハンドラ（`pasta_scripts/pasta/shiori/event/scene_play.lua`）で決まった。`REG.X = REG.X or function` の形、門は引数を見る前、Reference0 の誤りは「名前にならない値：OnPastaScene Reference0（値なし）」の表記、見つからないときは「シーンが見つからない」の表記とログの警告（`OnPastaScene: scene not found: …`）を出して `act:build()` を返す（表記を出さない設定なら 204）、見つかればコルーチンを返してイベントの振り分け（`EVENT.fire`）に任せる。読み込みの並びはイベントの振り分け `event/init.lua` の L78〜82（最後が `scene_play`）。`＞` の解決 `ACT_IMPL.call` は `act.lua` の L703 に移った（中身は同じ）。
- **触るファイル**: 新しいハンドラのファイル（新規）、`event/init.lua` の読み込みの並びの末尾に 1 行だけ、Lua のテスト（新規と一覧への 1 行）、マニュアル `lua/shiori-events.md`（既定ハンドラの表と「7 つ」の数の文 L69〜79、主要イベントの表 L202〜216、新しい 2 節、必要なら「MCP からの呼び方」L620〜）と `grammar/variables.md`。シーン再生のファイル・`act.lua`・`pasta_check` の組み込みの名前の表は触らない。
- **規模**: 5〜6 タスク。
- **先に要るもの**
  - 働きの上では無い。
  - `chain-talk-interrupt`・`reg-handler-chain` と同じウェーブに置ける。約束は「`event/init.lua` は読み込みの並びの末尾に 1 行足すだけ。マニュアルの既定ハンドラの節は表の行と数の文だけを触り、その下の包み方の段落（L81〜95）とシーン再生の共通の決まりの節（L590〜618）は触らない。後から入る側が rebase で合わせ、生成の references は rebase の後に作り直す」。
  - `chain-talk-interrupt` より先に入ると、外から `自動トーク停止` を呼んだときコルーチンの道を通るので、今の決まりで待っているチェイントークの続きが捨てられる。あちらが入れば、台詞を出さない呼び出しは続きを残す。どちらの順でも壊れはしない。
- **種別**: 機能（外から `＞XXX` を呼ぶ口と、グローバル変数を書く口。2026-10-19 開発者の決定）。
- **要件定義のモデル**: Opus。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - 「警告のログを残して 204」は先例と食い違う。シーン再生と同じく、表記とログの警告を出して `act:build()` を返す形に揃えるのを推奨する。`act:call` に任せると表記の対象が「＞名前」になりイベント名が付かないので、先に `act:find_handler("scene", 名前)` で引いて、無ければイベント名付きの表記を自分で出すかを要件で決める。
  - 「いつでも有効」と、シーン再生の門（`SecurityLevel` が `external` なら断る）を揃えるか決める。MCP は `local` で届く。変数の書き換えや `ゴースト終了` をほかの機械から呼べないように、`external` は断るのを推奨する。
  - JSON の部品（`@json`）は、`pasta.toml` の `[lua]` の `libs` に `-json` と書くと読み込まれない（`crates/pasta_lua/src/runtime/runtime_config.rs` の `should_enable_module`）。`pcall` で引き、無ければ警告にする。`null` を読んだ結果が `nil` か特別な値かも確かめる。
  - `chain-talk-interrupt` の決まりのどれに当てるか（ふつうのイベントと同じか、`OnPastaScene` と同じく「はっきり捨てる」か）を、後から入る側の要件で決める。推奨はふつうのイベントと同じ（台詞を出さない呼び出しは続きを残す）。
  - このファイルの「申し送り（scene-play-event より）」の節だけ改行が LF だったので、CRLF に揃えた。
