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

## 2026-11-02 棚卸の再測定（main 491cc23c）

- **前提の変化**: `＞＞` の道（Call の 5 段の探し方 `act.lua` 361 行の `find_act_handler` → `search_dictionary` 331 行 → `SCENE.search` `scene.lua` 150 行）と完全一致の部品（`search_scene_exact` `search/context.rs` 119 行・`search_local_scene_exact` 204 行・`SCENE.co_exec_exact` 230 行・`SCENE.co_exec_registered` 258 行）は、前回（2026-10-23）から変わっていない。0.4.2 の 3 本でイベントの受け口 `shiori/event/init.lua` が 372 行になり、`EVENT.no_entry` は 244〜248 行に動いた。0.4.2 の 3 本・`recommend-menu-scenes`・`ponytail-lua-loader`（`loader/config/` には触れなかった）は完了し、重なりではなくなった。0.5.0 に `scene-count` が入り、`シーン数` を `[scene] match = "exact"` に合わせるかをこの spec で決める（上の申し送り）。
- **前の節から持ち越すこと**: `＞＞` に要るのは 2 段目（実行中のシーンの中）と 5 段目（グローバル）の完全一致で、前からある `search_scene_exact`（親のシーンを渡せばローカルも探せる）で足りる。`search_local_scene_exact` は外から名前の組で指名する形で、`＞＞` には要らない。`OnPastaScene`・`OnPastaSceneAt` の既定のハンドラ（`scene_play.lua`）は常に登録されているので、受け口の「英大文字の続きを外す」規則の例にはならない。撫で・つつきの受け口（`mouse.lua` 143 行付近）の「止めている間は、つつきは 9 段目だけ」も、完全一致のモードでの 9 段目の論点に含める。`OnPastaCallFunction`（`external-control-events`）は「シーンの外の `＞XXX` と同じ探し方」なので、完全一致のモードに従わせるかをこの spec の要件で決める。
- **触るファイル**: 文法 `grammar.pest` 272（`call_scene` 170 行）・`parse_action.rs` 456・`ast/action.rs` 323・`element_gen.rs` 564、`act.lua` 837（1,000 行に近い）、`scene.lua` 266、`shiori/event/init.lua` 372、`virtual_dispatcher.lua` 328（125 行）、`choice_select.lua` 80（43・70〜71 行）、`mouse.lua` 176（120・128 行）、`loader/config/`（`mod.rs` 356・`sections.rs` 358）、英大文字の規則をシーン表で作るなら `scene_table.rs` 539 と `search/context.rs` 1,022（試験は別のファイルに置く）、`visit_action.rs` 431、VSCode の文法定義 273、`validate/references.rs` 135、マニュアル 3 章。
- **規模**: 9〜11 タスクのまま。
- **先に要るもの**: 機能の上では無い。ファイルの重なり（未完了）: 0.5.0 の `scene-condition-attributes`（`act.lua`・`scene.lua`・`search/context.rs`・VSCode の文法定義）・`scene-pick-modes`（`act.lua`、英大文字の規則をシーン表に置く場合は `scene_table.rs`）・`scene-count`（`search/context.rs`）、`scene-anchor-link`（文法・`act.lua`・`element_gen.rs`）、0.6.0 の `cue-command-runtime`（`act.lua`・`loader/config/`）・`novel-variable-scope`（文法・`element_gen.rs`）・`shell-element-click`（`init.lua`・`scene.lua`）、隙間の spec の `ponytail-dsl-parser`・`ponytail-lua-transpiler`・`ponytail-core-registry`・`ponytail-editor-tooling`・`ponytail-lua-scripts`（0.4.5 で先に入る）。
- **種別**: 機能（`＞＞` とモードの設定が無い）。ghost_dev の聞き取りから、開発者が承認して広げた。
- **要件定義のモデル**: Opus。新しい文法と設定の約束ではあるが、書き方と規則は brief で決まっていて、残る問い（検査の知らせ・完全一致のモードでの 9 段目・別名の置き換えとの順・`シーン数` を合わせるか）は互いに絡まず、それぞれ推奨を付けられる。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**:
  - 「Current State」: `resolve_scene_id_exact` は 255 行、`search_scene_exact` は 119 行、`SCENE.co_exec_exact` は 230〜251 行で形も変わった。「使っているのはデバッグのキックと撫で・つつきの 1〜8 段目だけ」は古い。キックは登録名で関数を直に引き、完全一致の検索の今の使い手は撫で・つつき（`mouse.lua` 120 行）と `OnPastaScene`（`scene_play.lua` 69 行）。イベントの受け口は `init.lua` 244 行、マニュアルの「イベント名で始まる名前のシーンも候補」は `lua/shiori-events.md` 1053 行。
  - 「Approach」2 の「今の 5 段」は 4 つしか挙げていない。1 段目の「実行中のシーンの表の完全一致」が抜けている。
