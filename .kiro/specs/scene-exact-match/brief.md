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

## 申し送り（recommend-menu-scenes より、2026-10-24）

- `[scene.alias]` の読み方が変わった。既定の表は 7 件（`OnTalk = ["会話"]` に、メニューのシーン 3 件 `sakura.recommendsites`・`kero.recommendsites`・`sakura.portalsites` とボタンの名前 3 件）。書いたキーだけ既定を置き換え、空の配列でそのキーの既定を消し、見出しだけなら既定のまま、既定の別名とぶつかれば読み込みエラー（`SceneAliasTable::with_written`、`SceneAliasError::ConflictsWithDefault`・`TargetIsDefaultAlias`）。置き換えの規則（完全一致・1 段・グローバルシーン名だけ・登録と検索の両方）は変わっていない。
- 論点: 完全一致のモードと別名の置き換えの順番を決めるときは、この 7 件の既定を前提にする。メニューのシーンは `SEARCH:search_scene_exact(id)` で ID そのもの（置き換え後の名前）を完全一致で引いている（`pasta/shiori/event/recommend.lua`）。
- 参照: `.kiro/specs/completed/recommend-menu-scenes/design.md` の「AliasTable」「MenuReceiver・InPlaceRun」。


## 申し送り（2026-10-29 の discovery「シーン属性による候補の絞り込み」より）

- `call-attribute-filter` は閉じて、新しい spec `scene-condition-attributes` に統合した（開発者の決定）。この brief の `call-attribute-filter` は `scene-condition-attributes` と読み替える。
- 中身が変わった。条件は呼ぶ側（`＞シーン＆k＝v`）でなく、場面の側に書く（`＊雑談_今の部屋　＆＄＊経験値≧500`）。
  - 呼ぶ側の Call の文法（`call_scene`・`parse_action.rs`・`element_gen.rs` の Call）は触らない見込みである。
  - 触るのは、属性の文法、`scene.lua`・`act.lua` の検索の道、`search/context.rs`・`scene_table.rs`、イベントからシーンを起こす道（選ぶ前の自動転記）、`pasta_check validate`、LSP と VSCode の文法定義である。
- 参照: `.kiro/specs/scene-condition-attributes/brief.md`。
- この spec への影響: `act:call` の 3 つ目の引数 `attrs` を絞り込みのために空けておく約束は、呼ぶ側の条件を作らないので要らなくなった。`＞＞` の完全一致の候補にも、場面の側の条件が効く。

## 申し送り（2026-10-29 の版の組み替えより）

- 0.5.0 は「属性関係の整備」になったが、この spec は属性ではないので入れていない。版は判断待ちのまま。0.6.0（ノベルゲーム。0.5.0 から順延）の `novel-talk-flow` が前提にするので、遅くとも 0.6.0 に入れる。`scene-condition-attributes` とは検索の道を共有するので、同じウェーブに置かない。
- ゴースト側の見立て（2026-10-29、ghost_dev「ぱすた開発」）: 入っていれば助かるが、必須ではない。入れるなら 0.5.0 の属性の 3 本の後（4 番目）、0.6.0 でも構わない。emo2（`scripts/pasta/shiori/event/boot.lua`・`install.pasta` 79 行の注記）と窓際のぱすたさん（`dic/event.pasta` 226〜238 行）は、`OnInstallCompleteAll` を DSL のシーンにできず Lua の REG で受けているが、今は動いていて困っていない。

## 申し送り（2026-11-01 の discovery「シーン数を数える API」より）

- 新しい spec `scene-count` が、`＠シーン数（名前）`（`＞名前` が選ぶ候補の数）と `＠シーン数完全一致（名前）`（`search_scene_exact` が選ぶ候補の数。`＞＞名前` と同じになる約束）を GLOBAL 関数で入れる。どちらもグローバルシーンだけを数え、別名表を当ててから正規化する。
- この spec で決めること: `[scene] match = "exact"` のモードで、`シーン数` も `＞` に合わせて完全一致の数にするか（推奨: する。「`＞` と同じ数」の約束を保つ）。`シーン数完全一致` はモードに関係なく完全一致のまま。
- ファイルの重なり: `scene-count` は `search/context.rs` を触る。同じウェーブに置かない。
- 参照: `.kiro/specs/scene-count/brief.md`。

## 2026-11-03 棚卸の再測定（main c575febd）

- **前提の変化**（前回 2026-11-02・`491cc23c` から）: `ponytail-core-registry` がシーン表の索引を `BTreeMap` に替えた（`scene_table.rs` 539 → 442 行）。完全一致の部品は残っている（`resolve_scene_id_exact` 195 行、控えのキーの `exact` 23 行、範囲つきの `resolve_global_owning_local_exact` 237 行）。`ponytail-lua-scripts` が Lua の検索の道を整えた（`act.lua` 837 → 812 行で `search_dictionary` 328 行・`find_act_handler` 358 行・`act:call` 683 行。`scene.lua` 266 → 261 行で `SCENE.search` 138 行・`SCENE.wrap` 177 行・`SCENE.co_exec` 194 行・`SCENE.co_exec_exact` 240 行・`SCENE.co_exec_registered` 253 行）。`search_scene_exact` は `search/context.rs` 114 行、`search_local_scene_exact` は 197 行。イベントの受け口は `init.lua` 220 行（`EVENT.no_entry` 217 行。ファイルは 347 行）、OnTalk・時報は `virtual_dispatcher.lua` 136 行、撫で・つつきは `mouse.lua` 120・128 行（変わらず）、選択肢の飛び先は `choice_select.lua` 53〜54 行（63 行）。`OnPastaCallFunction` は `act:find_handler("scene", name)`（`external_control.lua` 45 行）で探す。`[scene]` の読み取りは今も `alias` だけ（`loader/config/sections.rs` 337〜342 行）。`＞＞` は今も構文エラー（`call_scene` 170 行・`call_marker` 52 行）。
- **触るファイル**: `grammar.pest` 272・`parse_action.rs` 499・`ast/action.rs` 323・`element_gen.rs` 553、`act.lua` 812、`scene.lua` 261、`init.lua` 347、`virtual_dispatcher.lua` 340、`choice_select.lua` 63、`mouse.lua` 176、`OnPastaCallFunction` をモードに従わせるなら `external_control.lua` 123、`loader/config/`（`mod.rs` 356・`sections.rs` 358）、英大文字の規則をシーン表で作るなら `scene_table.rs` 442 と `search/context.rs` 1,009（1,000 行を超えている。試験は別のファイルに置く）、`visit_action.rs` 463、VSCode の文法定義 366（Call の規則は 176 行）、`validate/references.rs` 135、マニュアル 3 章。
- **規模**: 9〜11 タスクのまま。
- **先に要るもの**: 働きの上では無い。ファイルの重なり（未完了）: `scene-condition-attributes`（`act.lua`・`scene.lua`・`search/context.rs`・`scene_table.rs`・文法・`references.rs`・VSCode の文法定義）、`scene-count`（`search/context.rs`・`references.rs`、規則をシーン表に置くなら `scene_table.rs`）、`scene-attribute-store`（`search/context.rs`・`scene.lua`・`act.lua`）、`scene-pick-modes`（`act.lua`・`scene.lua`・文法）、`scene-anchor-link`（文法・`parse_action.rs`・`element_gen.rs`・`act.lua`・`choice_select.lua`・`references.rs`）、`shell-element-click`（`choice_select.lua`・`init.lua`・`scene.lua`）、`cue-command-runtime`（`act.lua`・`loader/config/`・文法）、`novel-variable-scope`（文法・`element_gen.rs`・`virtual_dispatcher.lua`）、`novel-talk-flow`（`act.lua`・`scene.lua`・`init.lua`・`virtual_dispatcher.lua`・`mouse.lua`・`loader/config/`）、`novel-checkpoint-rollback`（`init.lua`・`virtual_dispatcher.lua`）、`debug-mcp-runtime`（`virtual_dispatcher.lua`・`external_control.lua`・`loader/config/sections.rs`）。隙間の spec の 5 本は入ったので外れた。
- **種別**: 機能（`＞＞` とモードの設定が無い）。ghost_dev の聞き取りから、開発者が承認して広げた。
- **要件定義のモデル**: Opus（前のまま。新しい文法と設定の約束だが、書き方と規則は brief で決まっていて、残る問い（検査の知らせ・完全一致のモードでの 9 段目・別名の置き換えとの順・`シーン数` を合わせるか・`OnPastaCallFunction`）は互いに絡まず、それぞれ推奨を付けられる）。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - 「Current State」の行番号は古い（上の行が正しい）。「使っているのはデバッグのキックと撫で・つつきの 1〜8 段目だけ」も古い。完全一致の検索の今の使い手は、撫で・つつき（`mouse.lua` 120 行）・`OnPastaScene`（`scene_play.lua` 41 行）・チェイントーク中断（`init.lua` 185 行）。マニュアルの「イベント名で始まる名前のシーンも候補」は `lua/shiori-events.md` 1054 行。
  - 「Approach」2 の「今の 5 段」は 4 つしか挙げていない。1 段目の「実行中のシーンの表の完全一致」が抜けている（前のまま）。
  - 前の節から持ち越すこと: `＞＞` に要るのは 2 段目（実行中のシーンの中）と 5 段目（グローバル）の完全一致で、`search_scene_exact`（親のシーンを渡せばローカルも探せる）で足りる。`OnPastaScene`・`OnPastaSceneAt` は既定のハンドラが常にあるので、英大文字の規則の例にならない。撫で・つつきの「止めている間は、つつきは 9 段目だけ」も、完全一致のモードでの 9 段目の論点に含める。
