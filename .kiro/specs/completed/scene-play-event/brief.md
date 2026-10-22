# Brief: scene-play-event

## Problem

ゴーストの開発者は、AI エージェントに SSP の MCP サーバーを通してゴーストを動かさせ、デバッグさせたい。そのためには、MCP から「このシーンを今すぐ再生して」と頼めなければならない。

MCP の `raise_event` は、SSTP NOTIFY と同じく、SHIORI にイベントを送る。そして返ってきた台本を再生し、ツールの結果にも台本を載せる。ところが、pasta には指名したシーンを再生するイベントが無い。

- イベント名と同じ名前のシーンなら、`EVENT.no_entry` が再生する。しかし前方一致で探すうえに、任意のシーン（とくにローカルシーン）は指名できない。
- シーンを指名して再生する仕組み（`pasta-scene-kick`・`pasta-scene-kick-from-cursor`）はある。しかし入口は VSCode のデバッグの通り道（`pasta/playSceneAt`）だけで、SHIORI のイベントからは届かない。

## Current State

- **キック**
  - Lua の `KICK.install(scene)`（`pasta_scripts/pasta/shiori/event/kick.lua:46-49`）は、文字列を `STORE.kick_pending` に置くだけである。
  - 次の OnSecondChange で、`virtual_dispatcher.dispatch`（`virtual_dispatcher.lua:237-261`）が `KICK.try_dispatch`（`kick.lua:115-161`）を呼び、シーンを再生する。そのとき Status の抑止を 1 回だけ破り、進行中の `co_scene` を差し替える。
  - 文字列の形は 2 つ。大域の登録名（`会話_1`）と、ローカルの合成（`:会話_1:挨拶_1`）である。どちらも `STORE.scenes` を完全一致で引く。登録名には番号が付くので、外から渡す名前には向かない。
  - Lua の側に有効・無効の切り替えは無い。デバッグが無効なときは、Rust の通り道（`KickSink`）が捨てられるだけである。
- **位置からシーンを引く**
  - `SourceMap::scene_at`（`crates/pasta_lua/src/debug/source_map/mod.rs:528`）が `SceneIdentityIndex::scene_at`（`scene_index.rs:123`）を呼ぶ。最も内側のシーンを先に選び、無ければ行より下にある最も近い宣言を選ぶ。どちらも無ければ見つからない。
  - 呼び手は `playscene.rs:71` の `resolve_and_kick` で、`uri_to_pasta_path`（`:100`）でパスを直してから引く。
  - ソースマップと索引は、デバッグを有効にしたときしか作られない（`loader/mod.rs:214-232`・`runtime/factory.rs:241-258`）。
- **書いた名前どおりに引く**
  - `SEARCH:search_scene_exact(name, parent)`（`crates/pasta_lua/src/search/context.rs:105`）は、名前を直して（`sanitize_name`）エイリアスを当て、同じ名前の候補から重ならない順送りで 1 つ選ぶ。番号の `_1` は付けずに引ける。
  - Lua の口は `SCENE.co_exec_exact(name)`（`scene.lua:227`）だけで、大域しか引けない。ローカルシーンを引く口は無い。
- **イベントの振り分け**
  - `EVENT.fire`（`shiori/event/init.lua:215-255`）は、`REG[req.id]` が無ければ `EVENT.no_entry` を呼ぶ。
  - 組み込みのハンドラーは `REG.OnX = REG.OnX or function(act) ... end` の形で置く（`choice_select.lua:41` が先例）。`init.lua:78-81` に `require` を足す。
  - ハンドラーがコルーチンを返すと、`EVENT.drive` が 1 ビート目を返し、残りを `set_co_scene` に置く。続きは OnSecondChange が流す。
  - NOTIFY には、Rust が中身を見ずに 204 を返す（`actor/thread.rs:204-210`）。
- **名前の先例**: `OnPastaCallBack{N}`・`OnPastaClock` がある。`OnPasta<名前>` の形に揃える。
- **`pasta_check validate`**: イベント名の一覧を持たない。この spec では触らない。

## Desired Outcome

- MCP の `raise_event` から、名前か位置でシーンを指名すると、そのシーンがライブ SSP で再生される。ツールの結果に 1 ビート目の台本が載る。

  ```
  raise_event("OnPastaScene",   ["会話"])                     → ＊会話 を再生
  raise_event("OnPastaScene",   ["会話", "挨拶"])             → ＊会話 の中の ・挨拶 を再生
  raise_event("OnPastaSceneAt", ["dic/boot.pasta", "42"])     → その行のシーンを再生
  ```

- 見つからないときは、何が見つからなかったかがツールの結果（バルーン）とログに出る。
- 配るゴースト（デバッグを有効にしていない）では、挙動が何も変わらない。
- マニュアルに 2 つのイベントの節があり、スキルの references にも載っている。

## Approach

2 つの組み込みのイベントを足す。シーンを引いたら、そのシーンのコルーチンをハンドラーから返し、1 ビート目をその場で答える。続きと中断の動きは、今のシーン継続の仕組みに任せる（2026-10-19 開発者の決定。議案をすべて推奨案のまま承認）。

- **イベントは 2 つに分ける**
  - `OnPastaScene`: Reference0 が大域シーンの名前、Reference1 がローカルシーンの名前（省略できる）。
  - `OnPastaSceneAt`: Reference0 がファイル、Reference1 が行。
  - 1 つのイベントに詰めると、`42` が行なのか名前なのか見分けられない。
- **名前は書いたとおりに一致させる**
  - 番号の `_1` は付けずに、`＊会話` なら `会話` と渡す。名前の直し方（`sanitize_name`）と `[scene.alias]` は、ふだんの検索と同じ規則にする。前方一致はしない。
  - 同じ名前のシーンが複数あるときは、ふだんと同じ選び方（重ならない順送り）にする。特定の 1 つを狙うなら、`OnPastaSceneAt` で位置を指す。
  - ローカルシーンは、まず大域を選び、その中から同じ名前のローカルシーンを探す。`search_scene_exact` の `parent` を使い、Lua の口を足す。
- **位置の引き方は、右クリックの実行と同じにする**
  - ファイルは、ゴーストの `ghost/master/` からの相対パスか、絶対パスで渡す。区切りは `/` と `\` のどちらでもよい。
  - 行はエディタと同じく 1 から数える。
  - 引き方は `SourceMap::scene_at` をそのまま使う（最も内側、無ければ下の最も近い宣言）。
- **1 ビート目はその場で返す**
  - `KICK.install` は使わない。あちらは次の OnSecondChange まで待ち、要求には 204 を返すので、MCP の結果に何も載らない。
  - 進行中の会話は中断し、元には戻さない（キックと同じ）。`EVENT.drive` の `set_co_scene` が差し替える。
  - 続きのビートは OnSecondChange で流れる。MCP の結果に載るのは 1 ビート目だけである。
- **デバッグを有効にしたときだけ使える**
  - pasta.toml の `[debug]` か `PASTA_DEBUG` で有効にしたときだけ、2 つのイベントがシーンを再生する。無効なら 204 を返し、「デバッグが無効なので再生しない」とログに書く。
  - 配るゴーストの動きが変わらない。位置の索引もいつも揃っている。
  - 「名前はいつでも、位置はデバッグのときだけ」の案は採らなかった。片方だけ動くのは中途半端である。
- **見つからないとき**は、失敗を出す仕組み（`failure-output-unification`・`runtime-error-balloon`）で理由を出す。デバッグ中ならバルーンにも出るので、MCP の結果に載る。
  - 例: `OnPastaScene: シーン「会話・挨拶」が見つかりません`
  - 引数が足りない・行が数でない、も同じ道で出す。

## Scope

- **In**
  - `OnPastaScene`・`OnPastaSceneAt` の組み込みのハンドラー（`pasta/shiori/event/` に新しいモジュールを置き、`init.lua` に `require` を足す）
  - ローカルシーンを書いた名前どおりに引く Lua の口（`search_scene_exact` の `parent` を通す）
  - 位置からシーンを引く処理を Lua から呼べるようにすること（`SourceMap::scene_at` と `uri_to_pasta_path` の使い回し。相対パスの受け付け）
  - デバッグが有効かどうかを Lua から知る手段（無ければ足す）
  - 見つからない・引数の誤りを、失敗を出す仕組みに載せること
  - テスト（Lua のテストと、位置から引く Rust のテスト）
  - マニュアル（`book/src/lua/shiori-events.md` の既定ハンドラーの表と 2 つの節、`book/src/internals/debug.md` のシーンキックの節、`book/src/debug/` の再生の節から参照）と、スキルの references の作り直し
- **Out**
  - VSCode の拡張と、SSP の MCP サーバーの側の変更
  - シーンに引数やアクターを渡すこと
  - 登録名（`会話_1`）での指名
  - 前方一致での指名
  - SSTP 以外の経路（SSTP の押し出し、`\![raise]` の作例）
  - デバッグを無効にしたゴーストでの再生

## Boundary Candidates

- イベントのハンドラーと引数の確かめ（`pasta/shiori/event/` の新しいモジュール）
- 名前からシーンを引く Lua の口（`scene.lua`・`search/context.rs`）
- 位置からシーンを引く処理の公開（`debug/source_map`・`playscene.rs` の使い回し）
- マニュアルとスキルの references

## Out of Boundary

- キックの動き（`co_scene` の差し替え・Status の抑止を破る・OnSecondChange の継続）の作り直し。`pasta-scene-kick` が持つ。
- 同じ名前のシーンが別のファイルにあるときの、デバッガの索引と通し番号の制約（ロードマップのバックログ。`scene-identity-format` の範囲外として残したもの）。この spec はそれを受け継ぎ、直さない。
- ランダムトークと時報を止めるスイッチ（`ontalk-suppression`）

## Upstream / Downstream

- **Upstream**
  - シーンキックと位置の索引（完了 spec `pasta-scene-kick`・`pasta-scene-kick-from-cursor`・`scene-identity-format`）
  - 書いた名前どおりの検索とエイリアス（完了 spec `scene-search-key-normalization`・`scene-name-alias`）
  - 失敗を出す仕組み（完了 spec `failure-output-unification`・`runtime-error-balloon`）
  - デバッグの有効・無効（完了 spec `debug-transport-hardening` など。pasta.toml `[debug]`・`PASTA_DEBUG`）
- **Downstream**
  - AI エージェントによるゴーストのデバッグ（SSP の MCP サーバーの `raise_event`）
  - 将来、シーンに引数を渡す口を足すなら、この 2 つのイベントの Reference2 以降が候補になる

## Existing Spec Touchpoints

- **Extends**: なし（新しい境界）
- **Adjacent**
  - `external-control-events`（0.4.2。2026-10-19 起票）: この spec の**後に入る**。外から `＞XXX` を呼ぶ `OnPastaCallFunction` と、グローバル変数を書く `OnPastaSetSaveVariable` を足す。既定のハンドラの置き方・`event/init.lua` の `require`・マニュアルの既定ハンドラーの表を共有し、あちらがこの spec の形に揃える。ハンドラから先は別物である。
  - `ontalk-suppression`（0.4.2）: 自動トークを止めている間に、このイベントのシーンの続き（`co_scene`）を流すかが接点になる。要件で「キックと同じく止めない」と決める案を推奨にする。ファイルは `virtual_dispatcher.lua` を共有しない見込みなので、同じウェーブに並べられる。共有することになったら、あちらが先に入る。
  - `pasta-check-release-asset`（0.4.2）: ファイルを共有しない。並べられる。
  - `ponytail-lua-debug`（ponytail・隙間の spec）: 同じ `crates/pasta_lua/src/debug/` を触る。この spec が先に入り、あちらが後から合わせる。

## Constraints

- 前提: `raise_event`（SSTP NOTIFY）は、SHIORI に **GET** で届く。ツールの説明に「返った台本を再生する」とあるのを根拠にした。仕様書では確かめきれなかったので、設計のときに SSP の実機で確かめる。NOTIFY で届くと分かったら、1 ビート目の返し方を設計で決め直す。
- デバッグを無効にしたときの挙動（全イベント）は、何も変えない。今のテストは、そのまま通ること。
- 作者が Lua で `REG.OnPastaScene` を先に定義していたら、そちらを優先する（`REG.OnX = REG.OnX or ...` の先例どおり）。
- 要件定義は Opus。Fable は不要。
- 規模は 6 タスク前後の見込み（ハンドラーと引数 1、名前の口 1、位置の口 1、失敗の出し方 1、テスト 1、マニュアルとスキル 1）。

## 2026-10-22 棚卸の再測定（main 6a8dc479）

- **前提の変化**
  - 2026-10-19 夜の直し（c4d0380e、`crates/pasta_lua/src/debug/enable.rs`）で、デバッグのポートが取れないとき（デバッグを有効にしたゴーストを同じベースウェアで 2 体起動したとき）は、デバッグなしで起動を続けるようになった。このとき設定ではデバッグが有効なので、位置の索引は作られる（`loader/mod.rs` L214〜232）が、ランタイムの `debug_enabled()`（`runtime/exec.rs` L63。デバッグの持ち手があるか）は偽になる。「デバッグを有効にしたときだけ」を設定で決めるか、待ち受けが動いているかで決めるかを要件で決める。推奨は設定（MCP からの再生にポートは要らず、索引はある）。
  - 名前どおりに引く `SEARCH:search_scene_exact(名前, 親)` は、親（ローカルシーンの範囲）をもう Lua から渡せる（`crates/pasta_lua/src/search/context.rs` L302〜306）。足すのは `scene.lua` の側だけで、Rust の検索は触らない。Lua からデバッグの有効と位置の索引を知る口は、まだ無い。
  - ほかの Current State の行番号（`kick.lua`・`virtual_dispatcher.lua`・`init.lua`・`playscene.rs`・`scene_index.rs`）は今も合っている。
- **触るファイル**: 新しいハンドラのファイル（`pasta_scripts/pasta/shiori/event/` の下。新規）、`event/init.lua`（267 行。L78〜81 に読み込みを 1 行）、`pasta_scripts/pasta/scene.lua`（240 行。`co_exec_exact` に親を通す）、`crates/pasta_lua/src/debug/playscene.rs`（相対パスを `ghost/master/` に結ぶ）、Lua へ口を出す登録（`runtime/factory.rs` 324 行か `runtime/module_registry.rs` 201 行）、失敗の種類を足すなら `pasta_scripts/pasta/failure.lua`、テスト（Lua のテストと一覧 `tests/lua_specs/init.lua`、`debug/playscene_tests.rs`）、マニュアル。1,000 行に近いファイルは無い。
- **規模**: 7〜8 タスク（brief の 6 に、デバッグの有効の決め方と Lua への口の登録で 1〜2 増える）。
- **先に要るもの**: なし。`pasta-check-release-asset`・`ontalk-suppression` と同じウェーブに並べられる。条件は、`virtual_dispatcher.lua`・`shiori/entry.lua`・`store.lua` に触らないこと（`ontalk-suppression` の brief の注意を参照）。後に入るのは `external-control-events`（`event/init.lua`）と `chain-talk-interrupt`（`event/init.lua` と、この作業が足すハンドラのファイル）。0.4.2 の後に入る作業では、`scene.lua` を `scene-exact-match`・`scene-pick-modes`・`scene-attribute-store`・`call-attribute-filter`・`recommend-menu-scenes`・`novel-talk-flow` が、`playscene.rs` とソースの対応表を隙間の `ponytail-lua-debug` が、`runtime/` を隙間の `ponytail-lua-loader` が触る。
- **種別**: 機能（MCP から指名したシーンを再生する口。開発者の求め：AI にゴーストをデバッグさせたい）。
- **要件定義のモデル**: Opus（`raise_event` が GET か NOTIFY かは実機で確かめる項目で、どちらでも決め方の推奨がある）。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - 「続きのビートは OnSecondChange で流れる」は正確でない。続きは、ランダムトークの判定（`virtual_dispatcher.lua` の `check_talk`）が次のトークの時刻に達したときだけ流れる（既定で 3〜5 分おき）。MCP の利用者向けに、2 ビート目からはしばらく後に出ると書く。
  - `uri_to_pasta_path`（`playscene.rs` L100）は、相対パスを SSP の今のフォルダから絶対にする。`ghost/master/` からの相対パスを受けるには、先に結ぶ処理が要る。
  - `ontalk-suppression` との接点（止めている間の続き）は、推奨を「止める」に改めるのがよい。今のコードでは、キックの続きも止まるためである。

## 申し送り（ontalk-suppression より、2026-10-22 完了時）

- 止める門は `virtual_dispatcher.lua` の `dispatch` で、`KICK.try_dispatch` の後・`check_hour` の前に置いた。止めている間もキックの 1 ビート目は出て、2 ビート目からの続き（`check_talk` が返す `STORE.co_scene`）は出ない。続きは捨てずに残る。`OnPastaScene` の続きも `co_scene` を通るなら同じ扱いになる（brief の再測定の推奨「止める」のとおり）。
- 止めている間に再生したシーンが `＞チェイントーク` を使うと、警告が出て止めが解ける（`.kiro/specs/completed/ontalk-suppression/design.md` の決定事項 4。マニュアルには書かない扱い）。
- マニュアル `grammar/call-jump.md` の「自動トーク停止・自動トーク再開」の節は、止めないものに「シーン再生で求めたシーンの最初の出力」を挙げ、`debug/dev-actions.md#シーン再生-シーンを実行` へリンクしている。新しいイベントを足すときは、この行とリンクを合わせて直す。
