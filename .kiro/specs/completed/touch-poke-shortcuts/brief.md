# Brief: touch-poke-shortcuts

`/kiro-discovery` で起票（2026-10-11・main `dd1bbb3c`）。開発者の依頼による。**高優先度**（開発者の指示。0.4.0 の残りの spec より先に始める）。ghost_dev の「emo2 開発」セッションに、emo2 の実装の調査を頼み、コードつきの返事をもらった（下の「emo2 の実装」。コードの控えは、同じフォルダの `research-emo2.md`）。

## Problem

辞書を書く人が、撫でられた・つつかれたときの反応を書きたい。今は、次の 2 つが要る。

- **つつき**: `＊OnMouseDoubleClick` を書き、`＞transfer_req_to_var` で `＄ｒ４`（部位の名前）を読む。部位やキャラで台詞を分けるには、Lua で振り分けを書くしかない。入門ガイドの 8 段目は、部位の名前を台詞に埋めるところまでしか教えられない。
- **撫で**: pasta には撫での判定が無い。`OnMouseMove` は大量に届くので、「撫でた」と決める仕組みを作者が Lua で書く。emo2 は 72 行の `touch_detect.lua` を自前で持ち、同じファイルを pasta-in-windows にも複製している。

書きたいのは「頭を撫でられたらこの台詞」である。イベントの名前も Reference の番号も、作者が知りたいことではない。

## Current State

- イベントの振り分けは `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua` の `EVENT.fire` にある。`REG[req.id]` にハンドラがあればそれを呼び、無ければイベントと同じ名前のシーンを探す（`EVENT.no_entry`）。`OnMouseDoubleClick`・`OnMouseMove` にはハンドラが無い。
- 名前を組み立てて段々に探す手本が 2 つある。
  - 時報: `時報HH → OnHourHH → 時報その他 → OnHourOther` の 4 段（`event/virtual_dispatcher.lua` の `check_hour`）。
  - 選択肢: 明示の `＊OnChoiceSelectEx` が先、無ければ選択肢の ID で探す（`event/choice_select.lua`）。
- トーク中・選択肢の表示中などを見分ける関数がある（`virtual_dispatcher.is_blocked`。Status ヘッダーの `talking`・`choosing` などを見る）。使っているのは仮想イベントだけである。
- `OnMouseMove` のリクエストにも Status ヘッダーが付く（SSP の実ログ `crates/pasta_shiori/doc/shiori-sample.log` で、`Status: balloon(0=0)` の付いた `OnMouseMove` を確かめた）。pasta は、これを `act.req.status` として渡している（`crates/pasta_shiori/src/lua_request.rs`）。トーク中かどうかは、撫での判定の中から読める。
- スコープの番号からアクターの名前は引ける。`pasta.toml` の `[actor."名前"]` の `spot` は必須で、読み込み時に `STORE.actor_spots`（名前 → スポット）へ入る（`pasta_scripts/pasta/store.lua`）。実行中は、出力の組み立てがこの表を書き換える。
- シーンの検索は前方一致である（`crates/pasta_lua/src/search/context.rs`・`pasta_core` の `scene_table.rs`）。`さわり反応` で探すと `＊さわり反応Head` にも当たる。
- hello-pasta の当たり判定は `Head`・`Face`・`Body` の 3 つで、女の子も男の子も同じ名前を使う。足元には無い（`STAGES.md`）。
- 8 段目の辞書（`dic/08-touch.pasta`）は `＊OnMouseDoubleClick` を 3 つ持つ。9 段目（`dic/09-choice.pasta`）も `＊OnMouseDoubleClick` を 1 つ持ち、8 段目の 3 つと同じ名前で抽選される。

### emo2 の実装（2026-10-11 に「emo2 開発」から受け取った）

- emo2 の SHIORI は pasta.dll である。pasta の標準に乗っているのは、イベントと同じ名前のシーン・`act:transfer_req_to_var()`・`act:call` の前方一致の 3 つだけで、撫での判定は全部自前である。
- **撫での判定**（`project/emo2/ghost/emo2/ghost/master/scripts/touch_detect.lua`。前身の emo-gs の `has_talk.touch` の移植）:
  - 使うイベントは `OnMouseMove` だけ。回数ではなく時間で決める。
  - 同じ「部位:キャラ」の上で 2 秒続いたら 1 回発火する。部位かキャラが変わるか、イベントが 2 秒より長く途切れたら、リセットする。
  - 発火した後は、リセットされるまで再発火しない。状態は `OFF`・`COUNTING`・`FIRED` の 3 つ。
  - 部位が空（当たり判定の外）のイベントは、判定に入れずに 204 を返す。
- **振り分け**（`dic/touch.pasta`）: キーは「部位＋キャラ番号」（`Head0`・`Bust1`）。`＊OnMouseMove` のローカルシーンに `・Head0通常`・`・Head0照れ` などを並べ、`act:call(…, "Head0", {})` の前方一致で 1 つを抽選する。4 キー × 4 通りで 16 シーン。
- **つつき**: 実例が無い。emo2 の `＊OnMouseDoubleClick` は、部位もキャラも見ずにメニューを開く（`dic/menu.pasta`）。
- **フォールバック**: emo2 は 1 段だけ。pasta-in-windows は「代表の部位は個別、ほかは汎用 1 つ」を Lua の `if` で書いている。

**はまりどころ（emo2 の経験）**:

1. 前方一致の衝突。`OnInstallComplete` と `OnInstallCompleteAll` で、約半数が誤って抽選された（ghost_dev `4a2ba33` で直した）。
2. トーク中の抑止を移植で落とした。トーク中に 2 秒撫でると、反応がトークを打ち切る。前身の emo-gs は Status の `talking` を見ていた。
3. `os.time()` は秒単位なので、2 秒のしきい値でも実際の発火は 2〜3 秒後になる。
4. 部位が空のイベントが判定に届かないので、当たり判定の外へ出てもリセットされない。外へ出て 2 秒以内に同じ部位へ戻ると、発火済みのままになる。
5. 状態が 1 つしか無いので、2 人のキャラを行き来すると毎回リセットされる。
6. Lua ブロックの中の誤りが、無言で 204 になる。
7. 身体でない当たり判定。シェルが「閉じる」ボタンにも当たり判定を付けていると、汎用のフォールバックがそこで鳴る（pasta-in-windows は、名前を挙げて除外している）。
8. 部位の名前はシェル任せで、英字（`Head`・`Bust`）も日本語（`頭`・`顔`・`胸右`）も来る。
9. SSP と areka で届くイベントが同じかは、確かめていない。

### 他の SHIORI の先例

里々は、同じことを既定の動作として持つ。`OnMouseDoubleClick` で `＊(Reference3)(Reference4)つつかれ`（例: `＊0Headつつかれ`）、同じ場所への `OnMouseMove` が決めた回数（既定 60）を超えたら `＊(Reference3)(Reference4)なでられ`。2 秒途切れたら回数を戻し、トーク中は反応しない（設定で変えられる）。

## Desired Outcome

- 作者は、決まった名前のシーンを書くだけで、撫で・つつきの反応を書ける。Lua も Reference の番号も要らない。

```pasta
＃ 女の子の頭をつつかれたとき
＊つっつき反応女の子Head
　女の子：＠驚き　ひゃっ！　あたま！？

＃ 誰の頭でも（上に当たらなかったとき）
＊つっつき反応Head
　男の子：＠照れ　ぼ、ぼくの頭？

＃ どれにも当たらなかったとき
＊つっつき反応その他
　女の子：＠通常　なあに？

＃ 撫でも同じ形
＊さわり反応女の子Head
　女の子：＠照れ　えへへ……なでなで？

＊さわり反応その他
　男の子：＠笑顔　ぼくにもやってほしいなあ。
```

- 英字の名前でも書ける（`＊OnPoke女の子Head`・`＊OnPokeHead`・`＊OnPokeOther`。撫では `OnTouch`）。時報の `時報07`・`OnHour07` と同じ関係である。
- 同じ名前のシーンを複数書けば、今までどおり抽選される。
- ショートカットのシーンを 1 つも書いていないゴーストは、今までと同じに動く（emo2 のメニューと自前の撫で判定が、そのまま動く）。
- 入門ガイドの 8 段目と hello-pasta の辞書が、新しい書き方になる。
- emo2 と pasta-in-windows は、自前の `touch_detect.lua` を捨てられる。

## Approach

**決めたこと（2026-10-11 の discovery・開発者の決定）**

| 論点 | 決定 |
| ---- | ---- |
| 名前 | 撫では `さわり反応`、つつきは `つっつき反応` で始める |
| フォールバック | 時報と同じ決まりにする。部位つきが無ければ `＊さわり反応その他`・`＊つっつき反応その他` を呼ぶ。接頭辞だけの `＊さわり反応` は探さない（前方一致で部位つきのシーンに当たるため） |
| 英字の名前 | 時報（`時報HH → OnHourHH → 時報その他 → OnHourOther`）と同じく、日本語の名前の次に英字の名前を挟む。英字の名前は、撫でが `OnTouch`、つつきが `OnPoke`（開発者が承認した）。後ろに付くもの（アクターの名前・部位）は日本語の名前と同じで、`その他` は `Other` にする |
| キャラの区別 | アクターの名前を部位の前に置く（`＊つっつき反応女の子Head`）。スコープの番号からアクターの名前へは、`pasta.toml` の `[actor]` の `spot` で変える。名前が決まらないときに番号（`＊つっつき反応1Head`）へ落とすかは、要件で決める |
| トーク中の撫で | 反応しない。emo2 が移植で落として、反応がトークを打ち切るようになった抑止を、pasta の側で最初から持つ。Status ヘッダーで見分ける |
| 範囲 | 入門ガイドの該当箇所と hello-pasta の辞書の書き直しまでを含める |

**方式**: Lua のイベント層に、`OnMouseDoubleClick` と `OnMouseMove` の既定のハンドラを足す（`REG.OnMouseDoubleClick`・`REG.OnMouseMove`）。時報と同じく、候補の名前を上から順に `SCENE.co_exec` で探し、最初に見つかったシーンを返す。文法・トランスパイラ・Rust の検索は触らない。

探す順の案（男の子＝スポット 1 の `Head` をつついたとき）:

1. `つっつき反応男の子Head`（アクター＋部位）
2. `OnPoke男の子Head`
3. `つっつき反応Head`（部位だけ）
4. `OnPokeHead`
5. `つっつき反応その他`
6. `OnPokeOther`
7. `OnMouseDoubleClick`（今までどおり、イベントと同じ名前のシーン）

撫では、同じ形で `さわり反応…`・`OnTouch…` を探し、最後に `OnMouseMove` を探す。

- 1〜6 段は、名前の並びが違うので、前方一致でも互いに当たらない。
- 英字の名前は、`OnMouseDoubleClick`・`OnMouseMove` で始めない。7 段目は前方一致で探すので、`＊OnMouseDoubleClickHead` のような名前にすると、7 段目がそれに当たる。時報の `OnHourHH` が安全なのは、`OnHour` だけの名前を探す段が無いからである。
- 7 段目を最後に置くので、ショートカットを書いていないゴーストの挙動は変わらない。選択肢の「明示のシーンが先」とは順が逆になる。ショートカットのほうが細かい指定なので、先に探す。
- 撫では、`OnMouseMove` のたびに判定を回し、「撫でた」と決まったときだけ 1〜6 段を探す。決まらなかったとき・見つからなかったときは、今までどおり `＊OnMouseMove` を探す。

**採らなかった案**

- `[scene.alias]` に足す — 別名は完全一致の 1 段だけで、部位の名前を動的に組み立てられない。
- 文法に糖衣を足す — シーンの名前の規則で足りる。
- 番号を部位の後ろに置く（`Head1`） — `つっつき反応Head` が前方一致で `Head1` に当たる。

## 要件で決めること

1. **アクターの名前の引き方**: `pasta.toml` の `[actor]` の `spot`（読み込み時の値）で引くか、実行中の表（`STORE.actor_spots`。場面がスポットを入れ替えると変わり、消されることもある）で引くか。同じスポットに 2 人いるとき、そのスポットのアクターがいないとき（SSP は 2 以降も送る）の扱い。番号の段（`つっつき反応1Head`）を足すか。
2. **部位の名前の前方一致**: `つっつき反応胸` は `＊つっつき反応胸右` にも当たる（部位の名前はシェル任せで、頭が同じ名前がありうる）。この spec で完全一致の引き方を持つか、決まりとして書くだけにするか。完全一致の場面選びは `novel-talk-flow` も入れる予定である。根が同じなので、この spec で直す案を先に検討する。
3. **アクターだけの段**: 「男の子のどこか」（`＊つっつき反応男の子その他`）を足すか。
4. **部位が空のとき**: つつきは `＊つっつき反応その他` を呼ぶ見当（今の 8 段目は、当たり判定の外でも反応する）。撫では反応しない見当（emo2 と同じ）。
5. **撫での判定**: 時間で決める（emo2）か、回数で決める（里々）か。既定の値。秒より細かい時計を使うか。当たり判定の外へ出たらリセットするか。キャラごとに状態を持つか。同じ所を撫で続けたときの 2 回目。`pasta.toml` に設定を置くか（置くなら `[ghost]`）。
6. **抑止の細かい所**: トーク中は撫でに反応しない、までは決まっている。残るのは次の 3 つ。
   - 抑止している間のイベントを「撫でていない」と数えるか（emo-gs はそうしていた。こうすると、トークが終わってから改めて決めた秒数だけ撫でる必要がある。撫での反応そのものもトークなので、同じ所を撫で続けたときの 2 回目が、反応の再生が終わった後に自然に出る）。
   - 見分ける語を、仮想イベントと同じ一覧（`talking`・`choosing`・`passive` など 9 つ）にするか、絞るか。トークが終わってバルーンが残っているだけの間（`balloon(0=0)`）は、抑止しない見当。
   - つつきにも効かせるか（つつきは、利用者がはっきり操作しているので、トークを打ち切ってよい見当）。
7. **身体でない当たり判定**: 「閉じる」ボタンなどで `その他` が鳴るのを、どう避けるか（部位つきのシーンで受けて何も返さない、で足りるかを確かめる）。
8. **シーンの中から読めるもの**: 触られた部位とキャラを、`＞transfer_req_to_var` なしで台詞に入れたいか。
9. **8 段目・9 段目の作り直し**: 8 段目で `＄ｒ４` を教え続けるか。9 段目の `＊OnMouseDoubleClick`（選択肢を出すシーン）は、8 段目が `＊つっつき反応その他` を書くと呼ばれなくなる。9 段目の名前を合わせるか、8 段目に `その他` を書かないか。段階表の 9 段目の Reference（`2=OnMouseDoubleClick`）も変わる。
10. **つつきのイベント**: `OnMouseDoubleClick` とする見当（里々と今の 8 段目に合わせる）。`OnMouseClick`（シングル）と `OnMouseWheel` は範囲の外。
11. **`その他`・`Other` という名前の部位**: そういう部位があるシェルでは、部位つきの段とその他の段が同じ名前になる。決まりとして書くだけでよいかを確かめる。

## Scope

- **In**:
  - `OnMouseDoubleClick`・`OnMouseMove` の既定のハンドラと、名前を段々に探す処理。
  - 撫での判定（状態の持ち方・リセット・抑止）。
  - スコープの番号からアクターの名前を引く処理。
  - Lua のテスト、イベントのテスト。
  - hello-pasta: `dic/08-touch.pasta`、連動する `dic/09-choice.pasta`、`STAGES.md`（8・9 段目の行と Reference の表）、`README.md` の辞書の一覧、`src/scripts.rs` と `tests/tutorial_stages_test.rs` の検査。
  - マニュアル: 入門の `08-touch.md` と、連動する `09-choice.md`。リファレンス（`lua/shiori-events.md`、設定を足すなら `reference/pasta-toml.md`）。文法の章からの案内（`grammar/variables.md` ほか、`OnMouseDoubleClick` を例にしている所）。
  - スキル references の再生成（`node book/tools/gen-skill-refs.mjs`）。
- **Out**:
  - `OnMouseClick`・`OnMouseWheel`・押しっぱなし（里々の「ころころ」「ホールド」に当たるもの）。
  - エレメントのクリック（`shell-element-click`）。
  - 文法・トランスパイラの変更。
  - emo2・pasta-in-windows の辞書の載せ替え（ghost_dev の側の作業。公開したら知らせる）。
  - シェルの当たり判定の定義（部位の名前は `Head`・`Face`・`Body` のまま変えない）。

## Boundary Candidates

- 名前を段々に探す処理（イベント・アクター・部位から候補の名前を作り、上から探す）。
- 撫での判定（`OnMouseMove` の流れから「撫でた」を決める。時計を外から渡せる形にして、テストできるようにする）。
- スコープの番号からアクターの名前を引く処理。
- 見本のゴーストと入門の章（辞書と章は逐語で照合される。`book/tools/tutorial-check.mjs`）。

## Out of Boundary

- 選択肢の振り分け（`choice_select.lua`）と、その取り出し（`shell-element-click`・`scene-anchor-link`）。
- 失敗の見せ方（`failure-output-unification`）。飛び先が無いときは、今までどおり 204 を返す。
- 完全一致の場面選びを設定で切り替えること（`novel-talk-flow`）。

## Upstream / Downstream

- **Upstream**: なし。今の main で始められる。
- **Downstream**:
  - `getting-started-screenshots` — 8 段目・9 段目の絵は、この spec の後に撮る。8 段目の実機の確認も、新しい辞書で行う。
  - `manual-shell-guide` — `08-touch.md` からシェルの章へリンクを張る。後から入る側が合わせる。
  - ghost_dev の emo2・pasta-in-windows — 公開したら「emo2 開発」へ知らせる。自前の `touch_detect.lua` を捨てられる。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**:
  - `shell-element-click` — 「触られたときの反応の受け方を変えない」と線を引いている。この spec が受け方を足すので、brief の Current State を読み替える。
  - `runtime-error-balloon`（`event/init.lua` を触る）、`boot-surface-without-dic`（`event/boot.lua` を触る） — この spec は、`event/` に新しいファイルを足し、`init.lua` に `require` を 1 行足す。
  - `failure-output-unification`・`boot-surface-without-dic` — 設定を足す場合、設定の読み込み（`crates/pasta_lua/src/loader/config/`）が重なる。後から入る側が合わせる。
  - `pasta-check-dic-validate` — 接頭辞だけの `＊さわり反応`・`＊つっつき反応`（呼ばれないシーン）を検査で見つけられる。検査は、後から入る側が足す。
  - `.kiro/specs/completed/` の時報の spec（`virtual_dispatcher.lua` の 4 段フォールバック）、`choice-definition-dsl`（選択肢の振り分け）。

## Constraints

- ショートカットのシーンを書いていないゴーストの挙動を変えない。`＊OnMouseDoubleClick`・`＊OnMouseMove` は、今までどおり呼ばれる。
- `OnMouseMove` は大量に届く。判定は軽くし、ログを毎回出さない。
- 状態は Lua のモジュールの中に持つ（リロードで消えてよい）。LuaJIT 2.1 で動くこと。
- 辞書を変えたら、同じ変更で入門の章の作例を直す（逐語の照合に落ちる）。
- 挙動を変えたら、同じ変更でマニュアルを直し、スキル references を再生成する。
- 規模の見当は 10〜13 タスク。部位の名前の完全一致まで持つと、3〜4 タスク増える。
- 要件定義は Opus の見当。
