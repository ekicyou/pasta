# Brief: ontalk-suppression

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票）。0.4.2 に入れる（2026-10-19 開発者の決定）。要件定義は Opus。

## Problem

初回起動トークのように、何回かに分けて進むトークを書く作者が困る。

- 選択肢やイベントで区切ると、区切りの間はバルーンが空く。その隙にランダムトーク（OnTalk）や時報（OnHour）が始まり、流れが崩れる。
- シーンの側から、それを止める手段が無い。今できるのは、`scripts/main.lua` で `REG.OnSecondChange` を差し替えることだけである。これは素直な DSL の書き方ではない。

## Current State

2026-10-19（main `c4d0380e`）に確かめた。

- **発行の場所**: OnTalk と OnHour は `pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` の `M.dispatch` が出す。呼ぶのは `second_change.lua` の既定の OnSecondChange である。`dispatch` の中の順序は次のとおり。
  1. キックの許可フラグ（`STORE.kick_force`）
  2. Status の抑止（`is_blocked`）
  3. キック（`KICK.try_dispatch`）
  4. 時報（`check_hour`）
  5. ランダムトーク（`check_talk`）。止まっている場面（`STORE.co_scene`）の続きも、ここで返す。
- **今ある抑止**: Status に `talking`・`choosing`・`opening` などがある間だけである（`BLOCKED_STATUSES`。完了 spec `suppress-ontalk-on-choosing`）。区切りの間で Status が空になれば、抑止は効かない。
- **状態**: `next_hour_unix`（`CLOCK.now` で数える）・`seen_epoch`・`next_talk_time`（`CLOCK.steady` で数える）。どれもモジュールの局所変数で、セッションの間だけ生きる。
- **日本語の別名の先例**: `GLOBAL.close_ghost` に `GLOBAL["ゴースト終了"]`（`shiori/entry.lua` L18-27）。`yield` と `チェイントーク` も同じ組である。DSL からは `＞ゴースト終了` と呼べる。`pasta_check` は、これらを `crates/pasta_check/src/validate/builtins.rs` の `GLOBAL_NAMES`（4 つ）で組み込みの飛び先として知っている。
- **act の便利関数**: `ACT_IMPL`（`act.lua`）と `SHIORI_ACT_IMPL`（`shiori/act.lua`）にある。今回はこちらに置かない（Approach）。

## Desired Outcome

シーンから 1 行で、ランダムトークと時報を止めたり、再開したりできる。

```
＊OnFirstBoot
　＞自動トーク停止
　ぱすた：割り込みされないよ！
　＞自動トーク再開
　ぱすた：これ以降、会話が発話するよ！
```

- `＞自動トーク停止` の後は、`＞自動トーク再開` を呼ぶまで、OnTalk・OnHour・止まっている場面の続き（チェイントーク）が始まらない。
- Lua からも同じ関数を呼べる（`GLOBAL.pause_auto_talk(act)`）。
- 再開すると、ランダムトークの間隔は、再開した時刻から数え直される。直後に割り込まれない。

## Approach

2026-10-19 の話し合いで、開発者が決めた（案 A と、議案のすべての推奨）。

1. **置き場所と名前**: `GLOBAL` に置き、日本語の別名を付ける。先例の `close_ghost`／`ゴースト終了` に揃える。
   - 停止：`pause_auto_talk`、別名は `自動トーク停止`。
   - 再開：`resume_auto_talk`、別名は `自動トーク再開`。
   - 「自動トーク」は、タイマーで勝手に始まるもの（ランダムトークと時報）を指す。
   - 却下した名前：`会話停止`／`会話再開`（時報も止めるので、名前より効き目が広くなる）、`割り込み禁止`／`割り込み許可`（つつきやクリックは止めないので誤解を招く）、`会話トーク抑制`／`会話トーク再開`（語が重なり、対になっていない）。
2. **止めるもの**: OnTalk・OnHour・止まっている場面の続き（`check_talk` が返す `STORE.co_scene`）。
   - 止めないもの：デバッグのキック（`KICK.try_dispatch`）、コールバックの時間切れ（`CALLBACK.sweep`）、仮想時刻の読み取り（`CLOCK.take_probe`）、マウスなどほかのイベント。
3. **効いている期間**: そのセッションの間だけ。状態は `STORE` に持ち、`pasta.save` には置かない。`自動トーク再開` を書き忘れても、再起動すれば解ける。
4. **再開した後**:
   - ランダムトークの次の時刻は、再開した時刻から数え直す。
   - 止めている間に過ぎた時報は、後から出さない。次の正時から出る。
5. **入れ子**: 回数は数えない（真偽の 1 つ）。2 回止めても、1 回の再開で解ける。
6. **効く時点**: 関数を呼んだ時点で、すぐに切り替わる（トークンとして積まない）。シーンを組み立てる時に効くので、再開の後の台詞を表示している間に解ける。ただし、表示中は Status の `talking` で止まるため、割り込まれない。

## Scope

- **In**
  - 停止と再開の 2 つの関数と、その日本語の別名
  - `virtual_dispatcher.lua` で、止めている間は時報とランダムトークを判定しない
  - 再開した時点で、次のランダムトークの時刻を決め直す
  - `pasta_check` の組み込みの飛び先の名前（`GLOBAL_NAMES` に 4 つ足す）と、その検査のテスト
  - Lua のテスト（`crates/pasta_lua/tests/lua_specs/`）
  - マニュアル（呼び出しの節 `grammar/call-jump.md`、SHIORI イベントの節 `lua/shiori-events.md`、Lua の API `lua/script-api.md`、検査の組み込みの名前の一覧 `debug/validate.md`）と、スキルの references の作り直し
- **Out**
  - 時間が経つと自動で解ける機能
  - 止まっているかを問い合わせる関数
  - 再起動をまたいで止めておくこと（SAVE への保存）
  - 回数を数える入れ子
  - 見本のゴースト（hello-pasta）の書き換え

## Boundary Candidates

- 仮想イベントの発行（`virtual_dispatcher.lua` の `dispatch`・`check_talk`）と、状態の置き場所（`store.lua`）
- DSL から呼べる関数の登録（`shiori/entry.lua` の `GLOBAL` の定義）
- 辞書の検査が知る組み込みの名前（`pasta_check` の `builtins.rs`）
- マニュアルとスキルの references

## Out of Boundary

- ノベル向けの設定で、ランダムトークと時報を**ずっと**止めること（`novel-talk-flow`。この spec のスイッチを使い回す想定）
- Status による抑止の範囲（`BLOCKED_STATUSES`）を変えること
- `hour_margin` の型の確かめと `get_config` の重さ（隙間の spec の `ponytail-lua-scripts`）

## Upstream / Downstream

- **Upstream**: 仮想イベントの発行（`virtual_dispatcher.lua`。完了 spec `suppress-ontalk-on-choosing`・`pasta-scene-kick`）、`GLOBAL` の別名の先例（`shiori/entry.lua`）、`pasta_check` の組み込みの名前（`pasta-check-dic-validate` で入った表）
- **Downstream**: `novel-talk-flow`（0.5.0）のランダムトークと時報の停止。ノベル向けの設定のときに、このスイッチを常にオンにすれば済む

## Existing Spec Touchpoints

- **Extends**: なし（新しい境界）
- **Adjacent**
  - `ponytail-lua-scripts`（ponytail・隙間の spec）: 同じ `virtual_dispatcher.lua` の `get_config`（`hour_margin` の型、毎秒の読み込み）を触る。同じウェーブに置かない。この spec が先に入り、あちらが後から合わせる。
  - `novel-talk-flow`（0.5.0）: 止める場所が同じ `dispatch` である。あちらはこのスイッチを使い回す（あちらの brief に申し送りを書いた）。
  - `pasta-check-release-asset`（0.4.2）: ファイルを共有しない。並べられる。

## Constraints

- 止めていないとき（既定）の挙動は、何も変えない。今の `dispatch` のテストは、そのまま通ること。
- 非 ASCII の識別子は、ブラケット記法で定義する（`GLOBAL["自動トーク停止"]`。luacheck との互換のため。`entry.lua` の先例どおり）。
- 規模は 5 タスク前後の見込み（関数と状態 1、dispatch の分岐と再開時の決め直し 1、pasta_check 1、テスト 1、マニュアルとスキル 1）。

## 申し送り（2026-10-21 の discovery「PastaDSL機能ヒアリング」より）

- 窓際のぱすたさん（ghost_dev）の初回の演出（`boot.pasta` の `shokai_matsu`）は、`save["初回待ち"]` を立て、`REG.OnSecondChange` を一番外側で包み直して毎秒のイベントを全部止めている。ほかに `pose.pasta`（8 秒静かなら表情を戻す）と `tategaki.pasta`（窓あわせ）も `REG.OnSecondChange` を包んでおり、3 か所の包み合いが読み込みの順に頼っていて壊れやすい、と作者は書いた。
- ランダムトークと時報を止める部分は、この spec の `＞自動トーク停止`／`＞自動トーク再開` で置き換えられる見込みである。ゴースト自身の毎秒のフックまで止める門と、`REG` に複数のハンドラを重ねる仕組みは、この spec の範囲の外（roadmap のバックログに置いた）。

## 2026-10-22 棚卸の再測定（main 6a8dc479）

- **前提の変化**: 2026-10-19 の後、仮想イベントの発行（`virtual_dispatcher.lua` 303 行）、DSL から呼べる関数の登録（`shiori/entry.lua` 132 行。`close_ghost`／`ゴースト終了` は L18〜27 のまま）、組み込みの名前の表（`crates/pasta_check/src/validate/builtins.rs` 53 行）は変わっていない。Current State は今も正しい。
- **触るファイル**: `virtual_dispatcher.lua`（止めている間の判定と、再開したときの次の時刻の決め直し）、`shiori/entry.lua`（`GLOBAL` の 4 つの名前）、`pasta_scripts/pasta/store.lua`（124 行。止めている印と `STORE.reset` での片付け）、`builtins.rs`、`pasta_check` のテスト 2 本（名前を書き写して確かめる `tests/validate_scenes_test.rs` の `builtin_names_are_listed`、実行時との一致を見る `tests/validate_parity_test.rs` の `builtin_names_match_runtime`）、Lua のテスト（新しいファイルと、テストの一覧 `crates/pasta_lua/tests/lua_specs/init.lua` への 1 行）、マニュアル。1,000 行に近いファイルは無い。
- **規模**: 5〜6 タスク。
- **先に要るもの**: なし。`pasta-check-release-asset`・`scene-play-event` とソースのファイルを共有しない（下の注意を守れば）ので、同じウェーブに並べられる。`external-control-events` ともソースを共有しない。`chain-talk-interrupt` とは、あちらが `virtual_dispatcher.lua`・`store.lua` に触らなければ共有しない。マニュアルの `lua/shiori-events.md` はこの 3 本と共有するが、こちらは「仮想ディスパッチャ」の節（L696〜）で別の節。`grammar/call-jump.md` も、こちらは特別な呼び出しの表（L332 あたり）、`chain-talk-interrupt` はチェイントークの節で別。ほかの未完了の作業では `recommend-menu-scenes`（`entry.lua`・`builtins.rs`）、`novel-talk-flow`・`novel-checkpoint-rollback`・`scene-exact-match`（`virtual_dispatcher.lua`）、隙間の `ponytail-lua-scripts`（`virtual_dispatcher.lua`・`store.lua`・`entry.lua`）が、こちらの後に入る。
- **注意（`scene-play-event` との境）**: 今のコードでは、キックしたシーンの 2 回目からの続きも、ランダムトークの判定（`check_talk` の `STORE.co_scene`）から流れる。だから、この作業の決めどおり「止まっている場面の続きを止める」と、キックの続きも止まる（1 回目だけは出る）。`scene-play-event` の brief は「キックと同じく止めない」を推奨にしているが、それには `check_talk` で続きの出どころを見分ける必要があり、`virtual_dispatcher.lua` を共有してしまう。推奨は、この作業の要件で「止めている間は、キックとシーン再生の続きも止める（1 回目は出る）」と決め、`scene-play-event` は `virtual_dispatcher.lua` に触らないこと。
- **種別**: 機能（シーンから自動トークを止める口を足す。2026-10-19 開発者の決定。窓際のぱすたさんの初回の演出が動機）。
- **要件定義のモデル**: Opus（名前と効き目はもう決まっている）。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - 組み込みの名前は `builtins.rs` のほかに、`validate_scenes_test.rs` の `builtin_names_are_listed` が 4 つを書き写して確かめている。Scope の「その検査のテスト」に、この書き写しの直しを入れる。
  - `GLOBAL` の名前を挙げているマニュアルは、Scope の 4 ページのほかに `book/src/internals/shiori.md` L94・L299 と `book/src/internals/registry-search.md` L254 がある。手で書いたスキル `.claude/skills/pasta-ghost-authoring/SKILL.md` L207（特別な呼び出しの一覧）に足すかも決める（直すときは許可が要ることがある）。
  - テストの一覧 `tests/lua_specs/init.lua` には、Lua のテストを足す 0.4.2 の 4 本がそろって 1 行ずつ足す。これをソースの共有とみなすと 4 本とも並べられなくなるので、マニュアルの同じページと同じく「後から入る側が合わせる」扱いにするかを、棚卸で決める。
