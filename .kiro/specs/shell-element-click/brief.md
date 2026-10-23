# Brief: shell-element-click

`/kiro-discovery` で起票（2026-10-10・main `88c4bc0e`）。0.5.0「areka でノベルゲームが作れる」の道筋の 7 番目。

## Problem

ノベルゲームのタイトル・セーブ・ロード・設定の画面は、シェルとして組み、pasta の場面が動かす（2026-10-10 の決定）。areka が出すのは部品だけである。その部品の 1 つが、エレメント単位のクリックである。クリックできるエレメントは、絵で出来た選択肢と言える。ところが、pasta には、名前つきのクリックを場面へ届ける道が無い。

```
サーフェス「セーブ画面」
├─ 背景の絵
├─ 枠 1（エレメント。クリックできる。名前は slot1）
│   ├─ 縮小画像（別のサーフェスを階層的に呼ぶ）
│   └─ バルーン（日付と章の名前を書く所）
├─ 枠 2 … 枠 6
└─ 戻るボタン（エレメント。名前は back）
```

## Current State

- 選択肢は、選ばれると `OnChoiceSelectEx` で届く。飛び先を、選択肢を出したグローバルシーンのローカルシーンから先に探し、無ければグローバルシーンを探す（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/choice_select.lua`）。
- 触られたときの反応（`OnMouseDoubleClick` など）は、当たり判定の名前を Reference で受け取り、作者が場面の中で振り分ける（入門ガイドの 8 段目）。

### 2026-10-10 の検証で見つかった穴

- **選択肢の振り分けは、そのままでは使い回せない**: 処理は `REG.OnChoiceSelectEx` の中に直に書いてあり、Reference の番号が決め打ちである（`choice_select.lua` 16〜18・41〜78 行）。場面のコルーチンを作る関数も、そのファイルの中でしか使えない（25〜35 行）。
- **前方一致**: 場面の検索は前方一致である。`slot1` は `slot10` にも当たる。名前が完全に一致する場面を選ぶ部品は、`touch-poke-shortcuts` が入れた（範囲を付けて探せる `@pasta_search` の `SEARCH:search_scene_exact(名前, グローバルシーン名)`、`crates/pasta_lua/src/search/context.rs` 105・302 行と、グローバルシーンだけの `pasta_scripts/pasta/scene.lua` の `SCENE.co_exec_exact`）。この spec は、それを使って中で塞げる。
- **イベントの引数**: Reference は、場面の引数にならない（`shiori/event/init.lua` 226 行）。
- **NOTIFY には台本を返せない**: クリックの通知は GET で送ってもらう必要がある。
- areka は、今は `OnMouseClick` を SHIORI へ送っていない（areka に brief がある: `areka-P0-mouse-click-wheel-events`）。

### 申し送り（2026-10-11 の discovery・touch-poke-shortcuts より）

- 触られたときの反応には、`touch-poke-shortcuts` が既定のハンドラを足す（`REG.OnMouseDoubleClick`・`REG.OnMouseMove`。`＊つっつき反応女の子Head` → `＊つっつき反応Head` → `＊つっつき反応その他` → `＊OnMouseDoubleClick` の順に探す。各段の次に、英字の名前 `OnPoke…` も探す）。それが入った後は、上の「作者が場面の中で振り分ける」は、ショートカットを書かない場合の話になる。
- 下の「触られたときの反応との関係」は、その既定のハンドラの挙動を変えない、と読み替える。

## Desired Outcome

- エレメントがクリックされると、その名前の場面が動く。飛び先の探し方は、選択肢と同じである。
- 作者は、場面の中で Reference を読んで振り分ける処理を書かなくてよい。

```pasta
＊タイトル
    ％タイトル画面

    ・はじめから
        ＞第一章
    ・つづきから
        ＞ロード画面
```

- シェルの中のバルーンへ、場面から文を書ける。

## Approach

要件定義で決めること。

- **イベントの形**: 名前（エレメントの名前）と、どの画面の上で起きたか（選択肢の 3 番目の引数に当たるもの）を、どう受け取るか。名前は `novel-areka-contract` が決める。
- **振り分けの共有**: 選択肢の振り分けを、関数として取り出して使い回す。文中のシーンリンク（`scene-anchor-link`）も同じ振り分けを使う予定なので、取り出す形を揃える。
- **「どの画面の場面か」の決め方**: 選択肢は、台本に埋めたグローバルシーンの名前で決まる。エレメントのクリックには、それが無い。画面を出した場面を覚えておくか、areka に名前を運んでもらうか。
- **バルーンへ書く**: シェルの中のバルーンは、アクター（スコープ）として書けば足りるかを確かめる。足りなければ、名前を指定して書く口を足す。
- **飛び先が無いとき**: 選択肢と同じく 204 を返す。失敗の見せ方は `failure-output-unification` に乗せる。
- **触られたときの反応との関係**: 今の `OnMouseDoubleClick` の受け方を変えない。

## Scope

- **In**: エレメントのクリックのイベントを、場面へ振り分けること。選択肢の振り分けの取り出し。シェルの中のバルーンへ書く口（要るなら）。テスト、マニュアル、スキル references の再生成。
- **Out**: 画面の見た目と配置（シェル）。エレメントのクリックの検出と通知（areka）。セーブとロード（`novel-save-slots`）。

## Boundary Candidates

- イベントの入口（`shiori/event/` の新しいハンドラ）。
- 振り分けの共通部品（`choice_select.lua` から取り出す）。
- バルーンへ書く口。

## Out of Boundary

- 当たり判定の定義（シェルの `surfaces.txt`）。
- 画面を出す・消すの命令（語彙の話で、`cue-command-runtime` がそのまま流す）。

## Upstream / Downstream

- **Upstream**: `novel-areka-contract`（イベントの名前と Reference の決まり）。完全一致の場面選びは `touch-poke-shortcuts` が入れた部品を使うので、`novel-talk-flow` は待たない（ノベル向けの切り替えと揃えるかは、要件で決める）。
- **Downstream**: `novel-save-slots`（セーブ画面）、`hello-novel-sample`（タイトル・セーブ・ロード・設定の画面）。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**: `scene-anchor-link`（選択肢の振り分けを共有する。`choice_select.lua` を触る順番に注意）、`.kiro/specs/completed/choice-definition-dsl/`（選択肢の振り分けの元の設計）。

## areka への依頼

- エレメント単位のクリックを、エレメントの名前つきで、GET のイベントとして知らせる。
- シェルを、もう 1 枚上に重ねて出す・消す（システム画面）。
- 重ねた画面が出ている間、下の画面へのクリック送りを止める。

要件が固まったら、取り決めの文書へ書き、areka のセッションへ知らせる。手順は `novel-areka-contract` の brief の「依頼の出し方」。

## Constraints

- 選択肢の挙動を変えない。
- `choice_select.lua` は、`scene-anchor-link` と同じウェーブに置かない。
- 小さい spec である（5〜8 タスクの見当）。規模が膨らむなら、バルーンへ書く口を分ける。

## 申し送り（failure-output-unification より、2026-10-17）

- 失敗の見せ方を載せる先ができた。シーンが生きている間の失敗は `pasta/failure.lua` の `FAILURE.report(act, record)` から出す（警告 1 行と `【種類：対象（説明）】` の表記 1 つ）。新しい種類は `FAILURE.LABEL` に足す。表記の形・エスケープ・`[logging] balloon_warning` の切り替えは出口が持つ。
- 応答が表記だけになったシーンは 200 で返る。出さない設定で何も残らないときは 204 になる（`act:build` がすべて空の `raw_script` で nil を返す）。
- 参照: `.kiro/specs/completed/failure-output-unification/design.md`、マニュアル `book/src/reference/runtime-failures.md`。

## 申し送り（runtime-error-balloon より、2026-10-17）

- `FAILURE.LABEL` に `SCRIPT_ERROR = "スクリプトのエラー"`・`TIMEOUT = "時間切れ"` が入った（10 種）。種類を足すときは、hello-pasta のテスト（`tutorial_stages_test.rs`）が `LABEL` を全部読んで見張ることを前提にしてよい。
- `FAILURE.format` は `FAILURE.body`（枠なし・無効化なし）を無効化して【】で囲む形になった（出力は不変）。表記の形を変えるときは `FAILURE.body` も見る。
- 生成コードのシーン関数の先頭は `act:init_scene(SCENE, "書いたグローバル名", "書いたローカル名"|nil, "<.pasta>|<lua>:<pasta>,...")` になった。act は `scene_name`・`local_scene_name`・`line_map`・`scene_names` を控え、`call_restore` は 4 つを戻し、`restore_scene` はグローバル名だけを戻す。`act.lua`・コード生成・`init_scene` の行の数に触るときは、行の対応（`transpiler.rs` の `fill_line_maps`）と止まるエラーの表記を見直す。
- 参照: `.kiro/specs/completed/runtime-error-balloon/design.md` の Revalidation Triggers と `research.md` の「実装の記録」、マニュアル `book/src/reference/runtime-failures.md`「止まるエラーの表記」。

## 2026-10-17 棚卸の再測定（main b4164100）

- **前提の変化**:
  - 名前が完全に一致する場面を選ぶ部品が入った（`touch-poke-shortcuts`）。範囲を付けて探せる `@pasta_search` の `SEARCH:search_scene_exact(名前, グローバルシーン名)`（`crates/pasta_lua/src/search/context.rs` 105・302 行）と、グローバルシーンだけの `SCENE.co_exec_exact`（`pasta_scripts/pasta/scene.lua` 227 行）。`slot1` が `slot10` に当たる穴は、この部品でこの spec の中で塞げる。`novel-talk-flow` への依存は、機能としては無くなった（ノベル向けの切り替えと揃えるかは、要件で決める）。
  - 撫で・つつきの既定の受け口（`shiori/event/mouse.lua`）が、既定のハンドラを `REG.名前 = REG.名前 or 関数` の形で置き、完全一致の部品で場面を探す。新しい受け口の手本になる。選択肢の受け口も同じ形になった（`choice_select.lua` 41 行）。
  - 止まるエラーのバルーン表示（`runtime-error-balloon`）で、受け口のハンドラが Lua のエラーで止まっても、表記が 200 で出る（`shiori/event/init.lua` 227 行の `STOP.hold_handler`）。この spec で手当てする物は無い。
- **触るファイル**: 選択肢の振り分け `shiori/event/choice_select.lua` 80（振り分けを関数として取り出す）、クリックの受け口（新しいファイルなら、読み込みの 1 行を `shiori/event/init.lua` 77〜80 行に足す。`choice_select.lua` の中に置けば `init.lua` は触らない）、範囲つきの完全一致の場面の口を `scene.lua` 240 に足すならそれも、マニュアル。1,000 行に近いものは無い。
- **規模**: 5〜8 タスクのまま（完全一致の部品があるので増えない）。
- **先に要るもの**: `novel-areka-contract`（イベントの名前と Reference の決まり）だけ。`novel-talk-flow` は要らなくなったので、ロードマップの順番の 7 番より前に出せる。ファイルの重なり: `scene-anchor-link`（`choice_select.lua`。先に入る側が振り分けを取り出す）、`scene.lua` に足すなら `scene-attribute-store`・`call-attribute-filter`・`novel-talk-flow`、`init.lua` に読み込みを足すなら `novel-talk-flow`・`novel-checkpoint-rollback`、`cue-command-runtime` が柱を `choice_select.lua` で打つならそれとも。`pasta-check-validate-edges`・`tutorial-ssp-gaps`・`novel-areka-contract` とは重ならない。
- **種別**: 機能（名前つきのクリックを場面へ届ける道が無い）。開発者の 0.5.0 の決定（システム画面はシェルで組む）から。
- **要件定義のモデル**: Opus。イベントの名前は取り決めの文書が決め、振り分けは選択肢の規則を使い回す。「どの画面の場面か」の決め方は 2 案（画面を出した場面を覚える／areka に名前を運んでもらう）に絞れていて、推奨案を付けられる。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**:
  - 「前方一致… 完全一致の場面選びは `novel-talk-flow` が入れる」と「Upstream」の `novel-talk-flow` は古くなった（上の 1 点目）。2026-10-17 に直した。
  - 「Reference は場面の引数にならない（`init.lua` 226 行）」は 228〜232 行に動いた（ハンドラには act だけを渡し、`EVENT.drive(result, act, act)` で始める）。`choice_select.lua` の行番号（16〜18・25〜35・41〜78 行）は合っている。
  - 「失敗の見せ方は `failure-output-unification` に乗せる」は、完了したので `pasta/failure.lua` の `FAILURE.report` と読む（下の申し送りのとおり）。

## 申し送り（tutorial-ssp-gaps より、2026-10-18）

- **イベントの入口に時計が入った**: `EVENT.fire`（`shiori/event/init.lua` 215 行〜）の順が `CALLBACK.try_route` → `CLOCK.observe`（223 行。`OnPastaClock` を受けた回は 204 を返してハンドラへ進まない）→ `create_act` → `STOP.hold_handler`（234 行）→ ハンドラになった。読み込みの行に `pasta.shiori.clock` が足され（75 行）、ファイルは 267 行になった。brief に書いた `init.lua` の行番号はずれているので、要件定義で測り直す（`set_co_scene` は 100 行、ハンドラを `EVENT.drive(result, act, act)` で始める所は 239 行）。
- **時刻の読み方が変わった**: 新しいモジュール `pasta.shiori.clock` が「今」を一手に持つ。時報の判定と日時変数は `CLOCK.now`・`CLOCK.date`（SSP の仮想時刻に従う）、おしゃべりの間隔・コールバックの待ち時間・撫では `CLOCK.steady`（設定し直しなどの時刻の飛びを差し引いた時計）で数える。SSP 2.9.18 の TimeMachine は `act.req.date` 自体も仮想時刻にする。仕組みは本 spec の design.md の末尾「設計の見直し（7.2 の後・2026-10-18）」とマニュアル `lua/shiori-events.md` の「OnPastaClock」「時刻の飛び」。

## 2026-10-18 棚卸の再測定（main facd6cf4）

- **前提の変化**:
  - 選択肢の振り分け `shiori/event/choice_select.lua`（80 行）は変わっていない。行番号（16〜18・25〜35・41〜78・70〜71 行）は合っている。
  - `tutorial-ssp-gaps` が `shiori/event/init.lua` に時計を入れた（267 行。読み込み 75 行、`CLOCK.observe` 223 行、`STOP.hold_handler` 234 行、`EVENT.drive(result, act, act)` 239 行）。`CLOCK.observe` が止めるのは `OnPastaClock` だけ（`shiori/clock.lua` 150 行）なので、クリックの受け口の作りは変わらない。既定のハンドラの読み込みは 78〜81 行に動いた。
  - `shiori/event/mouse.lua`（170 行）は、撫での時刻を `CLOCK.steady` で数える 5 行だけの変化。手本の形（`REG.名前 = REG.名前 or 関数`、`SCENE.co_exec_exact` で探す 120 行）はそのまま。完全一致の部品（`scene.lua` 227 行の `SCENE.co_exec_exact`、`search/context.rs` 105 行の `search_scene_exact`）も変わっていない。
- **触るファイル**: `choice_select.lua`（80。振り分けを関数として取り出す）、受け口を新しいファイルにするなら `init.lua`（267）の 78〜81 行に読み込みを 1 行、範囲つきの完全一致の口を足すなら `scene.lua`（240）、テスト（`crates/pasta_lua/tests/lua_specs/`）、マニュアル（`lua/shiori-events.md`）。`act.lua` は触らない。1,000 行に近いものは無い。
- **規模**: 5〜8 タスクのまま。
- **先に要るもの**: `novel-areka-contract`（イベントの名前と Reference の決まり）だけ。ファイルの重なり: `scene-anchor-link`（`choice_select.lua`。先に入る側が振り分けを取り出す）、`cue-command-runtime`（柱を `choice_select.lua` で打つ場合）、`init.lua` に読み込みを足すなら `novel-talk-flow`・`novel-checkpoint-rollback`、`scene.lua` に足すなら `scene-attribute-store`・`call-attribute-filter`・`novel-talk-flow`。`talk-grapheme-clusters`・`lua-cmodule-robustness` とは重ならない。
- **種別**: 機能（名前つきのクリックを場面へ届ける道が無い）。開発者の 0.5.0 の決定（システム画面はシェルで組む）から。
- **要件定義のモデル**: Opus のまま。イベントの名前は取り決めの文書が決め、振り分けは選択肢の規則を使い回す。「どの画面の場面か」の決め方は 2 案（画面を出した場面を覚える／areka に名前を運んでもらう）に絞れていて、推奨案を付けられる。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**:
  - 2026-10-17 の節の「読み込みの 1 行を `init.lua` 77〜80 行に足す」は、78〜81 行に動いた。
  - `call-attribute-filter` の brief（2026-10-17 の節）は、この spec との重なりを `act.lua` としているが、この spec は `act.lua` を触らない。重なるのは `scene.lua` に口を足す場合だけ。

## 申し送り（external-control-events より、2026-10-23）

- `FAILURE.LABEL` に `NOT_JSON = "JSON として読めない値"`・`NOT_ASSIGNABLE = "代入できない値"` が入り、12 種になった（上の「10 種」は古い）。種類を足すときは `failure_test.lua` の数のテストも合わせて直す。
- 既定のハンドラの読み込みの並び（`pasta/shiori/event/init.lua`）の末尾に、`scene_play` の次の 1 行として `external_control` が入った。読み込みの行を足すときはその後ろになる。

## 2026-10-23 棚卸の再測定（main 82176f19）

- **前提の変化**
  - `choice_select.lua`（80 行）・`mouse.lua`（176 行）・`scene.lua`（266 行）・`search/context.rs` は、前の棚卸から変わっていない。行番号（16〜18・25〜35・41〜78・70〜71 行）は合っている。
  - `event/init.lua` は 371 行になった（`reg-handler-chain`・`chain-talk-interrupt`）。読み込みの並びは 85〜90 行で、末尾は `external_control`。新しい受け口のファイルを置くなら、その後ろに 1 行。ハンドラには今も act だけを渡し、Reference は場面の引数にならない（`EVENT.drive(result, act, act)` は 343 行）。`CLOCK.observe` は 317 行、`STOP.hold_handler` は 327・337 行。
  - `REG` にメタテーブルが付いた（`register.lua`）。既定のハンドラの置き方 `REG.名前 = REG.名前 or 関数` はそのまま使える。作者が `REG:register` で重ねたハンドラは、既定の受け口より先に呼ばれる。
  - チェイントークの決まり（`chain-talk-interrupt`）: クリックで動いた場面が台詞を出すと、待っている続きを捨てる割り込みになり、`＊チェイントーク中断` があれば反応の代わりにそれが出る。システム画面のボタンを、選択肢と同じ割り込みとして扱うか、指名した再生（`STORE.co_preempt` の印。中断の台詞を出さずに捨てる）として扱うかを、要件で決める。
  - 「どの画面の場面か」を覚える案で、`STORE.last_global_scene` は使えない。選択肢の受け口は範囲が無いときにこれを使う（`choice_select.lua` 68 行）が、中断の場面が走るとその名前になる（`chain-talk-interrupt` の design の Risks）。覚えるなら専用の置き場所にする。
- **触るファイル**: `choice_select.lua`（80）、受け口を新しいファイルにするなら `init.lua`（371）に 1 行、範囲つきの完全一致の口を足すなら `scene.lua`（266）、テスト、マニュアル（`lua/shiori-events.md` の既定ハンドラの表 119〜131 行と新しい節）。`act.lua` は触らない。1,000 行に近いものは無い。
- **規模**: 6〜8 タスク（チェイントークとの関係を決めて試す分で、下の端が 1 つ上がる）。
- **先に要るもの**: `novel-areka-contract`（未着手。イベントの名前と Reference の決まり）。ファイルの重なり（未完了）: `ponytail-lua-scripts`（`choice_select.lua` の写しを消す・`init.lua`）、`scene-anchor-link`（`choice_select.lua`）、`scene-exact-match`（`choice_select.lua`・`init.lua`・`scene.lua`）、`recommend-menu-scenes`（`init.lua` の読み込みの行。マニュアルの既定ハンドラの表にも同じく行を足す）、`cue-command-runtime`（柱を `choice_select.lua` で打つ場合）、`novel-talk-flow`・`novel-checkpoint-rollback`（`init.lua`）、`scene.lua` に足すなら `scene-attribute-store`・`call-attribute-filter`・`scene-pick-modes`。
- **種別**: 機能（名前つきのクリックを場面へ届ける道が無い）。開発者の 0.5.0 の決定（システム画面はシェルで組む）から。
- **要件定義のモデル**: Opus のまま（決め方は 2 案に絞れていて推奨を付けられる。チェイントークとの関係も 2 案）。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - 2026-10-18 の節の「`init.lua`（267）の 78〜81 行に読み込みを 1 行」は、85〜90 行の後ろに動いた。
  - `ponytail-lua-scripts` が先に入ると、`create_scene_coroutine`（`choice_select.lua` 25〜35 行）は `SCENE` 側の関数に置き換わる見込み。振り分けの取り出しは、先に入った側の形に合わせる。
