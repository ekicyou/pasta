# Brief: scene-exact-match

> **ステータス**: 未着手（2026-10-21 の `/kiro-discovery`「PastaDSL機能ヒアリング」で `event-scene-exact-match` として起票し、同日に開発者の決定で完全一致の仕組み一式に広げて改名した。ghost_dev の「emo2 開発」「ぱすた開発」セッションからの聞き取り）。版は未定（開発者の判断）。要件定義は Opus。

## Problem

シーンを探す方法が前方一致しかない。そのため、作者が呼びたくないシーンまで候補に入る。

- emo2（ghost_dev）で、`＊OnInstallComplete` と `＊OnInstallCompleteAll` を両方書いたところ、`OnInstallComplete` の候補に `＊OnInstallCompleteAll` が混ざった。`OnInstallCompleteAll` のシーンは Reference を見て 204 を返すので、約半数のインストール完了で何も言わなくなった。
  - 作者は `OnInstallCompleteAll` を `REG`（完全一致）へ逃がした。
  - 両ゴーストの作者は、シーン名を足すたびに、全部の名前の前方一致の衝突を総当たりで調べている。
- 窓際のぱすたさん（ghost_dev）では、`OnMouseEnterAll`／`OnMouseLeaveAll` が前方一致で `＊OnMouseEnter`／`＊OnMouseLeave` に入った。そのためシーンが見かけの約 2 倍動いていた（`chain-talk-interrupt` の brief の Problem）。
- 同じ形の組は SHIORI イベントに多い。`OnMouseClick`／`OnMouseClickEx`、`OnMouseDoubleClick`／`OnMouseDoubleClickEx`、`OnChoiceSelect`／`OnChoiceSelectEx` などである。
- ノベルでも同じことが起きる。`＞第1章` は `第10章` にも当たる（`novel-talk-flow` の brief の Current State）。

## Current State

2026-10-21（main `71e18f42`）に確かめた。

- **完全一致の部品はもうある**:
  - シーン表は、シーン名から関数の実体の ID の並びを引く。関数の実体には `_1` などが付く。名前と実体は別に管理されている。
  - `SceneTable::resolve_scene_id_exact`（`crates/pasta_core/src/registry/scene_table.rs` L251）は、名前が同じ候補だけを集める。その後は前方一致と同じ `select_from_cache` に渡し、シャッフル＆順次消費で 1 つ選ぶ。
  - 候補の控えのキーは、前方一致と別に持つ（`SceneCacheKey` の `exact`）。
  - `SearchContext::search_scene_exact`（`crates/pasta_lua/src/search/context.rs` L105）は、ローカルシーンとグローバルシーンを引ける。`SCENE.co_exec_exact`（`scene.lua` L227-238）もある。
  - これを使っているのは、デバッグのキックと、撫で・つつきのショートカットの 1〜8 段目だけである。
- **DSL の Call**: `call_scene = { call_marker ~ (id | call_target_expr) ~ s ~ args? }`、`call_marker = _{ gt }`（`crates/pasta_dsl/src/parser/grammar.pest` L52・L170）。名前は `＞` で始められないので、今は `＞＞` が構文エラーになる。
- **イベントの受け口**: `REG` にハンドラが無いイベントは、`SCENE.co_exec(act, act.req.id, nil, nil)`（`pasta_scripts/pasta/shiori/event/init.lua` L179）で前方一致で探す。マニュアル `book/src/lua/shiori-events.md` L635 は「`＊OnBoot朝` のように、イベント名で始まる名前のシーンも候補になる」と書く。作者はこれで `＊OnTalk_ギャル` のような変化を書いている。
- `novel-talk-flow` の brief には「完全一致の場面選びの切り替え」が残っている。切り替えの仕組みはまだ無い。

## Desired Outcome

- `＞＞シーン名` と書くと、そのシーンを完全一致で呼ぶ。同じ名前のシーンが複数あれば、今までどおりシャッフル＆順次消費で 1 つ選ぶ。
- `pasta.toml` の切り替えで、ゴースト全体のシーン探しを完全一致にできる。既定は今の前方一致のままにする。
- 既定の前方一致のままのゴーストでも、SHIORI イベントの受け口が別のイベントのシーンを拾わない。

```
＊OnInstallComplete
　＞＞インストール完了      ＃ 「インストール完了」だけ。「インストール完了All」は呼ばない
```

## Approach

2026-10-21 に開発者が承認した。

1. **構文**: `＞＞シーン名`。ほかの印と同じく、半角の `>>` も受ける。動的な Call にも引数にも効く。
   ```
   ＞＞季節9月
   ＞＞「季節」＆＄月
   ＞＞＄次の場面
   ＞＞紹介（「ぱすた」、3）
   ```
2. **探し方**: 探す順番は今の 5 段のままにする（ローカルシーン → act のメソッド → `GLOBAL` → グローバルシーン）。比べ方だけを完全一致にする。完全一致の名前が複数あれば、今までどおりランダムに選ぶ。既存の `search_scene_exact` を使う。
3. **モード**: `pasta.toml` の `[scene]` に `match = "exact"` を置く（既定は `"prefix"`）。`"exact"` にすると、次のものが全部完全一致になる。
   - `＞`
   - SHIORI イベントの受け口
   - OnTalk と時報
   - 選択肢の飛び先

   `＞＞` は、モードに関係なくいつも完全一致である。`novel-talk-flow` の「完全一致の切り替え」は、この設定に寄せる。
4. **既定のモードでのイベントの受け口**: イベント ID のすぐ後ろに英大文字が続く候補を外す。
   - 外れる: `＊OnInstallCompleteAll`・`＊OnMouseClickEx`
   - 残る: `＊OnBoot朝`・`＊OnTalk_ギャル`・`＊OnHour12`

   すでに `＊OnTalkMorning` のように英大文字を続けて書いた辞書は、挙動が変わる。破壊的変更として扱い、マニュアルと変更履歴に書く。
5. 要件で決めること:
   - `pasta_check validate` の知らせ。英大文字の続きで外れるシーンを知らせるか、`＞＞` の行き先が無いときに知らせるか。
   - 完全一致のモードで、撫で・つつきの 9 段目（`OnMouseMove` などの前方一致）をどう扱うか。
   - `[scene.alias]` の置き換えとの順番（`recommend-menu-scenes` がキーごとの置き換えに改める）。

## Scope

- **In**:
  - 文法（`＞＞`）、AST、コード生成（完全一致の印を付けた `act:call`）
  - `act.lua` の Call の道（完全一致のときは `search_scene_exact` へ回す）
  - `pasta.toml` の `[scene] match` の読み込み
  - イベントの受け口・OnTalk・時報・選択肢の飛び先へのモードの反映
  - 既定のモードでの英大文字の続きの規則
  - `pasta_check validate`、LSP と VSCode の着色（TextMate 文法）
  - テスト、マニュアル（`grammar/call-jump.md`・`lua/shiori-events.md`・`reference/pasta-toml.md`）、スキルの `references/` の再生成
- **Out**:
  - 単語 `＠` の完全一致。作者が実害は無いと答えた。
  - 選択肢 `＠？` 専用の完全一致の構文。選択肢もモードには従う。
  - 条件で候補を絞る書き方（`call-attribute-filter`）

## Boundary Candidates

- 文法・パーサ・コード生成（`pasta_dsl`・`pasta_lua` の `code_gen`）
- ランタイムの Call とイベントの受け口（`act.lua`・`scene.lua`・`shiori/event/init.lua`・`virtual_dispatcher.lua`・`choice_select.lua`）
- 設定（`loader/config/`）
- 辞書の検査・エディタの道具・マニュアル・生成スキル

## Out of Boundary

- 同名シーンの選び方（重み・巡回。`scene-pick-modes`）
- 完全一致の検索の部品そのもの。もうあるので作り直さない。

## Upstream / Downstream

- **Upstream**: なし
- **Downstream**:
  - `novel-talk-flow`。完全一致の切り替えを、この spec の設定で入れる。
  - ゴースト作者が `REG` へ逃がしていたイベント（emo2 の `OnInstallCompleteAll`）を DSL に戻せる。

## Existing Spec Touchpoints

- **Extends**: `novel-talk-flow` の「完全一致の場面選びの切り替え」を、この spec へ移す。
- **Adjacent**:
  - `call-attribute-filter`。同じ `call_scene` の文法とコード生成を触るので、同じウェーブに置かない。
  - `chain-talk-interrupt`。`event/init.lua` を触る。
  - `scene-play-event`・`external-control-events`。イベントの既定のハンドラを足す。`OnPastaScene` と `OnPastaSceneAt` も英大文字の続きの組なので、規則の例として確かめる。
  - `recommend-menu-scenes`。`[scene.alias]` を改める。

## Constraints

- 既定のモードで `＞` を使う辞書の挙動は変えない。変わるのは、イベントの受け口の英大文字の続きの候補だけである。
- 規模は 7〜9 タスクの見込み。内訳は、文法とコード生成 2、ランタイムの Call 1、モードの設定と受け口への反映 2、英大文字の規則 1、検査とエディタ 1、テストとマニュアル 1〜2。

## 2026-10-23 棚卸の再測定（main 8fc9bdff）

- **前提の変化**: `scene-play-event` が完全一致の部品を広げた。ただし `＞＞` の道（Call の中の探し方）は、まだどこにも無い。
  - 入ったもの: グローバルとローカルの名前の組で完全一致に探す `search_local_scene_exact`（`crates/pasta_lua/src/search/context.rs` 204 行）と、その裏のシーン表の関数 `resolve_global_owning_local_exact`（`crates/pasta_core/src/registry/scene_table.rs` 297 行）。Lua 側の `SCENE.co_exec_exact` は `(名前, ローカル名)` を受け、見つからなかった側を 2 つ目に返す形になった（`scene.lua` 230〜251 行）。登録名の組から再生する `SCENE.co_exec_registered`（258 行）も増えた。
  - `＞＞` に要るのは、Call の 5 段の探し方（`crates/pasta_lua/pasta_scripts/pasta/act.lua` 361 行の `find_act_handler`）のうち、2 段目（実行中のシーンの中）と 5 段目（グローバル）を完全一致にすることである。どちらも `search_dictionary`（331 行）→ `SCENE.search`（`scene.lua` 150 行）→ `search_scene` を通る。前からある `search_scene_exact`（`context.rs` 119 行）は親のシーンを渡せばローカルも探せるので、これで足りる。新しい `search_local_scene_exact` は「外から名前の組で指名する」ための形で、`＞＞` には要らない。
  - 外から指名したシーンを再生するイベント（`OnPastaScene`・`OnPastaSceneAt`）は、既定のハンドラ（`shiori/event/scene_play.lua` 61・82 行）がデバッグの無効なときも常に登録されている。そのため、イベントの受け口の「英大文字の続きを外す」規則の例にはならない（マニュアル `lua/shiori-events.md` 618 行がすでにそう書いている）。
  - `ontalk-suppression` が、撫で・つつきの受け口（`shiori/event/mouse.lua` 143 行・165 行）に「止めている間は、つつきは 9 段目だけ、撫では決めない」を足した。完全一致のモードで 9 段目をどう扱うかの論点は、この分岐も含めて決める。
- **触るファイル**: 文法 `grammar.pest`（272。`call_scene` 170 行）・Call を読む `parse_action.rs` 456・構文木 `ast/action.rs` 323・Call の Lua を出す `element_gen.rs` 564、`act.lua` 837（1,000 行に近い）、`scene.lua` 266、イベントの受け口 `shiori/event/init.lua` 268（`EVENT.no_entry` 177〜181 行）、ランダムトークと時報 `virtual_dispatcher.lua` 328（125 行）、選択肢の飛び先 `choice_select.lua` 80（43・70〜71 行）、撫で・つつき `mouse.lua` 176、設定の読み込み `loader/config/`（`mod.rs` 343・`sections.rs` 356）、英大文字の規則をシーン表で作るなら `scene_table.rs` 539 と `search/context.rs` 1,017（本体は 437 行まで。試験は別のファイルに置く）、エディタの着色 `visit_action.rs` 431、VSCode の文法定義 272、辞書の検査 `validate/references.rs` 135、マニュアル 3 章。
- **規模**: 9〜11 タスク（brief の 7〜9 に、受け口が 4 か所（イベント・時報とランダムトーク・選択肢・撫で）に散っている分と、`＞＞` の行き先の検査を足した）。
- **先に要るもの**: なし。ファイルの重なり:
  - 0.4.2 の残り: `chain-talk-interrupt`（`act.lua`・`init.lua` の `set_co_scene`・`EVENT.drive`）、`reg-handler-chain`（`init.lua` の `EVENT.fire`。`no_entry` へ落ちる所）、`external-control-events`（`init.lua` の読み込みの 1 行。あちらの `OnPastaCallFunction` は「シーンの外の `＞XXX` と同じ探し方」なので、完全一致のモードに従わせるかをこの spec の要件で決める）、`recommend-menu-scenes`（設定の `[scene.alias]` の読み込み・`init.lua`）。
  - 0.5.0: `call-attribute-filter`（Call の文法・生成・`act.lua`・`scene.lua`・検索。`act:call` の 3 つ目の引数 `attrs` は絞り込みのために空けておく約束にする）、`scene-anchor-link`、`novel-variable-scope`（文法・`element_gen.rs`）、`cue-command-runtime`、`shell-element-click`（`init.lua`・`scene.lua`）、`scene-pick-modes`（英大文字の規則をシーン表に置く場合）。
  - 隙間の spec: `ponytail-dsl-parser`、`ponytail-lua-transpiler`、`ponytail-core-registry`、`ponytail-editor-tooling`、`ponytail-lua-scripts`、`ponytail-lua-loader`（`loader/config/`）。
- **種別**: 機能（`＞＞` とモードの設定が無い）。ghost_dev の聞き取りから、開発者が承認して広げた。
- **要件定義のモデル**: Opus。新しい文法と設定の約束ではあるが、書き方と規則は brief で決まっていて、残る問い（検査の知らせ・完全一致のモードでの 9 段目・別名の置き換えとの順）は互いに絡まず、それぞれ推奨を付けられる。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**:
  - 「Current State」: `resolve_scene_id_exact` は 251 → 255 行、`search_scene_exact` は 105 → 119 行、`SCENE.co_exec_exact` は 227〜238 → 230〜251 行で形も変わった。「使っているのはデバッグのキックと撫で・つつきの 1〜8 段目だけ」は古い。キックは登録名で関数を直に引いていて完全一致の検索を使わない。今の使い手は撫で・つつき（`mouse.lua` 120 行）と `OnPastaScene`（`scene_play.lua` 68 行）。イベントの受け口は `init.lua` 179 → 180 行、マニュアルの「イベント名で始まる名前のシーンも候補」は 635 → 753 行。
  - 「Approach」2 の「今の 5 段」は 4 つしか挙げていない。1 段目の「実行中のシーンの表の完全一致」が抜けている。
