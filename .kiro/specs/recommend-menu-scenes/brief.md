# Brief: recommend-menu-scenes

> **ステータス**: 未着手（2026-10-20 の `/kiro-discovery` で起票）。版は開発者が決める（推奨は 0.4.2）。要件定義は Opus。

## Problem

右クリックメニューの「おすすめ」「ポータル」に、ゴースト作者が Lua を書かずに項目を並べる手段が無い。

- SSP は、メニューを開くと SHIORI に `sakura.recommendsites` などを GET で問い合わせる。pasta はこれに答える口を持たない。答えなければ 204 で、おすすめは空になる。
- 今答えられるのは、Lua の `REG["sakura.recommendsites"]` だけである。それも、マニュアルのどこにも書いていない。
- 窓際のぱすたさん（ghost_dev）は、2026-10-20 に Lua で 15 行ほど書いて、`sakura`・`kero` の両方を返した（`dic/menu.pasta`、未コミット）。「ぱすた開発」セッションがいちばん困ったのは、「`On` で始まらないリソースの ID も `REG` で受けられる」とどこにも書いていないことだった。
- 選んだ後のトークも、生のさくらスクリプトでしか書けない。表情の単語・縦書きの割り付けの単語・pasta のウェイトが使えない。

## Current State

2026-10-20（main `36a0a8db`）に確かめた。

- **SSP の決まり**（UKADOC の SHIORI リソース・イベントの一覧）
  - `sakura.recommendsites`・`kero.recommendsites`・`char*.recommendsites`・`sakura.portalsites`: 値は「項目名 `\1` URL `\1` バナー画像のパス `\1` 選んだときのトーク」を `\2` でつないだもの（`\1`＝バイト値 1、`\2`＝バイト値 2）。トークはさくらスクリプト。URL を `script:` で始めると、そのスクリプトが走る。項目名 `-` は区切り線。
  - ボタンの名前: `sakura.recommendbuttoncaption`・`kero.recommendbuttoncaption`・`sakura.portalbuttoncaption`、`char*.recommendsites.caption`。
  - 選ばれると `OnRecommendsiteChoice` が来る。Reference0＝サイト名、Reference1＝URL、Reference2＝バナー、Reference3＝`recommend`／`portal`、Reference4＝0（sakura）／1（kero）、Reference5＝上から何番目か（0 始まり）。
- **pasta の振り分け**
  - Rust 側（`crates/pasta_shiori/src/shiori.rs` L137・L266）は ID で分けない。すべて `SHIORI.request` → `EVENT.fire`（`pasta_scripts/pasta/shiori/event/init.lua` L215）へ渡す。
  - `EVENT.fire` は `REG[req.id] or EVENT.no_entry`（L232）。`no_entry`（L176-180）は ID と同じ名前のシーンを前方一致で探す（`SCENE.co_exec`）。`sakura.recommendsites` は検索キーの正規化で `sakura_recommendsites` になる（`pasta_core/src/registry/scene_registry.rs` L244-245）。
  - シーンの出力は必ず `act:build()` を通り、`\e` と `\p[…]` が付く（`scene.lua` L186-196、`shiori/sakura_builder.lua` L534-537）。リソースの値にならない。
  - シーンを動かすと、必ず `EVENT.drive` → `set_co_scene` を通り、待っているチェイントークの続きが捨てられる（`chain-talk-interrupt` で、台詞を出さないシーンは触れないように直す）。
  - `REG` のハンドラーが文字列を返すと、包まずに `RES.ok` になる（L245-250）。リソースの値を返せるのは、今はこの道だけである。
  - `recommendsites`・`portalsites`・`OnRecommendsiteChoice` は、コードのどこにも出てこない。
- **完全一致の引き方**: `SCENE.co_exec_exact`（`scene.lua` L227-238）がある。撫で・つつきのショートカットが使う。同じ名前のシーンが複数あるときに、ランダムに 1 つを選ぶかは確かめていない（前方一致の引き方では選ぶ）。
- **シーン名の別名**（`pasta_core/src/registry/scene_alias.rs` の `SceneAliasTable`、読み込みは `pasta_lua/src/loader/config/mod.rs` L108-150 の `parse_scene_aliases`）
  - 既定の表は `OnTalk = ["会話"]` の 1 件。
  - pasta.toml に `[scene.alias]` を書くと、既定の表は**丸ごと置き換わる**。見出しだけなら「別名なし」になる（`book/src/reference/pasta-toml.md` L388-397）。
  - 完全一致・1 段だけで、登録と検索の両方に効く。別名の表の指紋（`.scene_alias`）がキャッシュの無効化に使われる。
- **組み込みの呼び出しの先例**: `GLOBAL.close_ghost` と日本語の別名 `GLOBAL["ゴースト終了"]`（`shiori/entry.lua` L18-27）。DSL からは `＞ゴースト終了（1500）`。`pasta_check` は `crates/pasta_check/src/validate/builtins.rs` の `GLOBAL_NAMES` で、組み込みの飛び先として知っている。
- **マニュアル**: リソースの要求は `lua/shiori-events.md` L692 と `reference/runtime-failures.md` L246 で、「失敗の表記を出さない」ことだけが書かれている。おすすめ・ポータル・`OnRecommendsiteChoice` を扱う節は無い。

## Desired Outcome

ゴースト作者は、辞書（DSL）にシーンを書くだけで、おすすめメニューを作れる。pasta.toml は書かなくてよい。

```pasta
＊おすすめメニュー
  ＞おすすめ項目（「カクヨム」、「https://kakuyomu.jp/」、「おすすめ_カクヨム」）
  ＞おすすめ項目（「青空文庫」、「https://www.aozora.gr.jp/」）

＊おすすめメニュー
  ＞おすすめ項目（「小説家になろう」、「https://syosetu.com/」）
  ＞おすすめ区切り
  ＞おすすめ項目（「青空文庫」、「https://www.aozora.gr.jp/」）

＊相方おすすめメニュー
  ＞おすすめ項目（「IPA セキュリティ」、「https://www.ipa.go.jp/security/」）

＊おすすめ_カクヨム
  ぱすた：小説を探しに行くのね。いってらっしゃい。

＠おすすめボタン名：本棚、おすすめ
```

- メニューを開くたびに、2 本の `＊おすすめメニュー` のどちらかの並びが出る（同じ名前のシーンはランダムに選ばれる）。
- カクヨムを選ぶと、`＊おすすめ_カクヨム` が話す。
- メニューを開いても、待っているチェイントークの続きは消えない。
- 別名を 1 つ足したいだけなら、足したいキーだけを `[scene.alias]` に書けばよい。既定の別名は残る。

## Approach

2026-10-20 の話し合いで、開発者が決めた。

- 最初は pasta.toml の表（`[[recommend.sakura]]`）を推したが、開発者が前提から変えた。「シーンにしておくと、同じ名前のエントリがランダムに選ばれ、メニューを開くたびに別のものを出せる。ゴーストの挙動は DSL で書く、という精神にも合う。pasta.toml はとっつきにくい」。
- シーンで書く形への懸念だった 2 点（台詞の加工で `\1`・`\2` や URL が壊れる、メニューを開くたびにチェイントークが消える）は、下の P2 と「動かし方」で避ける。

### 前提（P1〜P6。議案をすべて推奨のまま承認）

1. **P1. シーンの名前とリソースの結びつけ**: シーン名の別名（`[scene.alias]`）の既定の表に足す。
   - `"sakura.recommendsites" = ["おすすめメニュー"]`、`"kero.recommendsites" = ["相方おすすめメニュー"]`、`"sakura.portalsites" = ["ポータルメニュー"]`。
   - 名前は、はじめ `おすすめ`・`相方おすすめ`・`ポータル` を推したが、開発者が「短すぎる」として `メニュー` を付けた（2026-10-20）。`＊おすすめ` はふつうのトークの名前として自然すぎて、たまたま重なりやすい。`＊おすすめメニュー` なら、名前を見ただけでメニューの中身と分かる。項目の呼び出し（`＞おすすめ項目`・`＞おすすめ区切り`）と単語（`＠おすすめボタン名`）は、メニューの中でしか使わないので、そのままにする。
   - 名前を変えたい作者は、`[scene.alias]` で変える。`char2.recommendsites` なども、別名を足せば書ける。
   - リソースのシーンは**完全一致**で引く。前方一致だと、`＊おすすめメニュー` で `＊おすすめメニュー_秋` のような別のシーンまで当たる。
2. **P2. 項目の書き方**: 組み込みの呼び出し `＞おすすめ項目（名前、URL、選んだときのシーン、バナー）`。
   - 3 つ目と 4 つ目は省ける。区切り線は `＞おすすめ区切り`。
   - `＞ゴースト終了` と同じく `GLOBAL` に置き、英語の名前と対にする（名前は要件で決める。例 `recommend_item`）。
   - 引数は式なので、単語や変数も使える。
   - 項目は台詞として積まず、リソースの値を作るための別の入れ物に積む。台詞の加工（ウェイト・改行・`\p`）を通らない。
   - 項目を積むシーンの中に台詞を書いても、リソースの値には入れず、捨てる。
3. **P3. 選んだ後のトーク**: `OnRecommendsiteChoice` で、選ばれた項目の 3 つ目の引数のシーンを完全一致で引く。無ければ今までどおり `＊OnRecommendsiteChoice` へ回る。
   - このトークは台詞を出すので、待っているチェイントークは忘れ、`＊チェイントーク中断` が出る（`chain-talk-interrupt` の決まり）。
4. **P4. ボタンの名前などの、一つの値で済むもの**: 決まった名前の単語で書く（例 `＠おすすめボタン名` → `sakura.recommendbuttoncaption`）。単語は候補からランダムに選ばれる。結びつける表は P1 と同じく既定を持ち、作者が変えられるようにする（変え方は要件で決める）。
5. **P5. Lua で書いた人との関係**: Lua の `REG` が勝つ。組み込みの受け手は作者のスクリプトより先に置かれるので、作者が `REG[...]` を書けば上書きされる。窓際のぱすたさんの今の書き方は、そのまま動く。マニュアルに「`On` で始まらないリソースの ID も `REG` で受けられる」と作例付きで書く。
6. **P6. 範囲**: おすすめ・相方おすすめ・ポータルの一覧と、そのボタンの名前まで。`homeurl`・`username`・メニューの文言・`getaistate` は入れない（P4 の仕組みで後から足せる）。

### 動かし方

- リソースの問い合わせのときは、シーンをその場で最後まで動かし、積んだ項目から値（`\1`・`\2` でつないだ文字列）を組み立てて返す。
- シーンの続きの置き場所（`STORE.co_scene`）には触れない。`EVENT.drive`／`set_co_scene` を通さない。
- シーンの中のチェイントーク（`＞チェイントーク`）やコールバック待ち（`get_property`）の扱いは、要件で決める（推奨は「そこで打ち切り、積んだところまでを返す」）。

### 別名の表の読み方を変える（A1〜A5。議案をすべて推奨のまま承認）

既定の別名が 4 つに増えるので、「1 行でも書くと既定が丸ごと消える」今の決まりを直す。

1. **A1. 書いたキーだけ既定を置き換える**。書かないキーの既定は残る。

   ```toml
   [scene.alias]
   OnBoot = ["起動"]                  # 足すだけ。会話・おすすめメニューなどの既定は残る
   OnTalk = ["会話", "おしゃべり"]   # OnTalk の別名だけが替わる
   ```

2. **A2. 既定の別名を消したいときは、そのキーに空の配列を書く**（`"sakura.recommendsites" = []`）。
3. **A3. 見出しだけの `[scene.alias]` は、既定のまま**（何も書かないのと同じ）。今の「別名なし」から意味を変える。
4. **A4. 書いた別名が、残っている既定の別名とぶつかったら、読み込みエラー**（今の重複と同じ扱い）。例: `OnBoot = ["会話"]` は既定の `OnTalk = ["会話"]` とぶつかる。エラーの文に、相手が既定の表のものであることと、直し方（`OnTalk = []`）を入れる。
5. **A5. これまでのゴーストへの影響は、マニュアルの変更点に書くだけ**。
   - `＊おすすめメニュー` という名前でふつうのシーンを書いていた作者は、そのシーンがおすすめメニューの中身に使われる。`＞おすすめメニュー` で呼ぶ道は、別名が呼ぶ側にも効くので、そのまま動く。困るなら A2 で戻せる。
   - 見出しだけの `[scene.alias]` で別名を消していた作者は、A3 で既定が戻る。A2 で戻せる。

## 要件で決めること

- 書き間違えの検査: `＞おすすめ項目` の引数の数と型、3 つ目のシーンが辞書に無いとき（`pasta_check validate` の報告の種類 `FindingKind` を足すか）、リソースのシーンに台詞を書いたとき（警告するか）。
- 同じ名前のシーンが複数あるとき、完全一致の引き方（`co_exec_exact`）でもランダムに選ぶか。選ばないなら、選ぶようにする。
- 項目を選んだときに SSP が URL を開くかどうか（`OnRecommendsiteChoice` にゴーストが答えた場合も含む）。SSP で確かめる。開くなら、4 つ目のトーク欄は空のまま送る。
- `OnRecommendsiteChoice` で選ばれた項目をどう特定するか（Reference3・4・5 で引くか、名前と URL で引くか）。メニューを開いたときの並びと、選んだときの並びが同じであることをどう保つか（ランダムに選び直すと並びが変わる）。
- バナー画像のパスの基準のフォルダ（SSP にそのまま渡すか）。
- P4 の単語とリソースの結びつけの表の置き場所と、変え方。
- 組み込みの呼び出しの英語の名前。
- 版（推奨は 0.4.2）。

## Scope

- **In**
  - リソースの組み込みの受け手（`sakura.recommendsites`・`kero.recommendsites`・`char*.recommendsites`・`sakura.portalsites` と、そのボタンの名前）。新しいファイルを `pasta/shiori/event/` に置き、`event/init.lua` で `require` する
  - 組み込みの呼び出し `おすすめ項目`・`おすすめ区切り`（`GLOBAL`）と、項目を積む入れ物
  - シーンを続きの置き場所に触れずにその場で最後まで動かす道
  - `OnRecommendsiteChoice` の既定の受け手（選ばれた項目のシーンへ回す）
  - 既定の別名 3 つの追加と、別名の表の読み方の変更（A1〜A5。`pasta_core` の `SceneAliasTable`、`pasta_lua` の `parse_scene_aliases`、エラーの文、info ログ `Scene alias table`）
  - `pasta_check`: 組み込みの名前（`builtins.rs` の `GLOBAL_NAMES`）、別名の表の読み方（`config-scene-alias`）、必要なら新しい報告の種類
  - テスト（Rust の別名の表、Lua の受け手、`pasta_check`）
  - マニュアル（おすすめメニューの節を新しく書く。`lua/shiori-events.md` の既定ハンドラーの表と「`REG` でリソースを受ける」作例、`reference/pasta-toml.md` の `[scene]`、`grammar/call-jump.md` のシーン名の別名、`debug/validate.md`）と、スキルの references の作り直し
- **Out**
  - pasta.toml の表でおすすめを書く形（開発者が退けた）
  - `homeurl`・`username`・メニューの文言・`getaistate` などのほかのリソース
  - 時間帯や好感度で並びを変える専用の仕組み（シーンの中の単語・変数・同じ名前のシーンのランダムで足りる）
  - 見本のゴースト（hello-pasta）と入門ガイドへの作例
  - Rust 側（`pasta_shiori`）で ID を振り分けること

## Boundary Candidates

- リソースの受け手と、シーンをその場で動かす道（`pasta/shiori/event/` の新しいファイル）
- 組み込みの呼び出しと項目の入れ物（`shiori/entry.lua` の `GLOBAL`、act）
- `OnRecommendsiteChoice` の受け手
- 別名の表（`pasta_core/src/registry/scene_alias.rs`・`pasta_lua/src/loader/config/mod.rs`）
- 辞書の検査（`pasta_check`）
- マニュアルとスキルの references

## Out of Boundary

- チェイントークの続きを捨てる決まり（`chain-talk-interrupt`）
- 外からのイベントの既定のハンドラーそのもの（`scene-play-event`・`external-control-events`）
- 自動トークの停止（`ontalk-suppression`）

## Upstream / Downstream

- **Upstream**: シーン名の別名（完了 spec `scene-name-alias`）、撫で・つつきの完全一致の引き方（`touch-poke-shortcuts`）、`GLOBAL` の組み込みの呼び出しの先例（`close_ghost`）、`chain-talk-interrupt`（選んだ後のトークがチェイントークをどう扱うか）
- **Downstream**: ほかのリソース（メニューの文言など）を DSL で返すとき、この spec の受け手と P4 の結びつけの表を使い回す

## Existing Spec Touchpoints

- **Extends**: `scene-name-alias`（完了）の別名の表の読み方を変える（A1〜A5）。2026-10-08 の決定（既定 `OnTalk = ["会話"]`、書けば丸ごと置き換え）のうち、「丸ごと置き換え」を「キーごとに置き換え」に改める。
- **Adjacent**
  - `scene-play-event`・`external-control-events`（0.4.2）: 既定のハンドラーの置き方（`pasta/shiori/event/` の新しいファイルと `event/init.lua` の `require`）と、マニュアルの既定ハンドラーの表を共有する。あちらが先に入り、この spec が揃える。
  - `ontalk-suppression`（0.4.2）: `pasta_check` の `builtins.rs` の `GLOBAL_NAMES` と、`shiori/entry.lua` の `GLOBAL` の定義を共有する。あちらが先に入る。
  - `chain-talk-interrupt`（0.4.2）: この spec はシーンの続きの置き場所を通さない道を足す。あちらは `set_co_scene` の決まりを変える。同じ `event/init.lua` を触るので、同じウェーブに置かない。
  - `call-attribute-filter`・`scene-attribute-store`（0.5.0）: 別名の置き換え（`scene_aliases.resolve`）を前提にしている。置き換えの規則（完全一致・1 段）は変えないので、影響は表の作り方だけ。
  - `ponytail-check-release`（ponytail・隙間の spec）: `builtins.rs` を触る。この spec が先に入る。

## Constraints

- 別名の置き換えの規則（完全一致・1 段だけ・グローバルシーン名だけ・登録と検索の両方に効く）は変えない。変えるのは表の作り方だけ。
- 別名の表の指紋（`.scene_alias`）でキャッシュを無効にする仕組みは保つ。既定の表が変わるので、版を上げた最初の読み込みでキャッシュが作り直される。
- 非 ASCII の識別子は、ブラケット記法で定義する（`GLOBAL["おすすめ項目"]`）。
- 規模は 10 タスク前後の見込み（別名の表 2、受け手とその場で動かす道 2、組み込みの呼び出し 1、選んだ後の受け手 1、ボタンの名前 1、pasta_check 1、テスト 1、マニュアルとスキル 1）。

## 申し送り（2026-10-21 の discovery「PastaDSL機能ヒアリング」より）

- 窓際のぱすたさん（ghost_dev）の `menu.pasta` は、`REG["sakura.recommendsites"]` と `REG["kero.recommendsites"]` が `\1`・`\2` 区切りの文字列を返し、イベントのシーン `＊osusume_site` は Lua の足場だけ（`return nil`）である。この spec の `＊おすすめメニュー`・`＊相方おすすめメニュー` で置き換える実例として、要件の確認に使える。

## 2026-10-23 棚卸の再測定（main 8fc9bdff）

- **前提の変化**
  - `ontalk-suppression`・`scene-play-event` が入り、先に要るものから外れた。`GLOBAL` の組み込みの名前は 8 つ（`pasta_scripts/pasta/shiori/entry.lua` L18〜48、`pasta_check` の `builtins.rs` の名前の表）。足すのはその後ろ。
  - 完全一致の引き方 `SCENE.co_exec_exact`（`scene.lua` L230〜256）は、ローカルの名前を 2 つ目に取り、見つからない側を 2 つ目の戻り値で返す形になった。同じ名前のシーンが複数あると、前方一致と同じく、混ぜてから順に重ならないように 1 つを選ぶ（`pasta_core/src/registry/scene_table.rs` の `resolve_scene_id_exact`）。「要件で決めること」の「完全一致でもランダムに選ぶか」は「選ぶ」で閉じてよい。選ぶたびに次へ進むので、選んだときにシーンを動かし直すと別の並びになる。開いたときの並びを覚えておく論点は残る。
  - 行の位置: イベントの振り分け `EVENT.fire` は `event/init.lua` L216、ハンドラを引く行は L233、イベント名でシーンを探す `EVENT.no_entry` は L177〜181、シーンを包む関数は `scene.lua` L186〜195。マニュアルのリソースの要求は `lua/shiori-events.md` L810 に移った。別名の読み込みは `pasta_lua/src/loader/config/mod.rs` L115。
- **触るファイル**: 新しいハンドラのファイル（新規）、`event/init.lua` の読み込みの並びの末尾に 1 行、`shiori/entry.lua`（153 行）、`pasta_core/src/registry/scene_alias.rs`（284 行）、`pasta_lua/src/loader/config/mod.rs`（343 行）とその試験、`pasta_check` の `builtins.rs`（名前の表 8 → 12）・`validate/config.rs`・試験（名前の一覧と一致の試験）、マニュアル。1,000 行に近いファイルは触らない。
- **規模**: 10〜12 タスク。
- **先に要るもの**
  - 働きの上では `chain-talk-interrupt`（P3 の中断の台詞）と `reg-handler-chain`（P5）。どちらも決まりを書くためで、コードの上では待たない。ファイルは `external-control-events` と読み込みの並び・既定ハンドラの表を共有する。
  - 版が決まるまでウェーブに入れない。0.4.2 に決めたら、「`event/init.lua` は読み込みの並びに 1 行だけ、`act.lua` は触らない（項目は `GLOBAL` の関数が act の欄に積む）、マニュアルの既定ハンドラの表は行を足すだけ」と約束すれば上の 3 本と同じウェーブにも置ける。安全なのは 3 本の後のウェーブ（0.4.2 が 1 ウェーブ延びる）。
- **種別**: 機能（窓際のぱすたさんの実例。シーンで書く形は開発者の決定）。
- **要件定義のモデル**: Opus（SSP が URL を開くかは実機で確かめられる。ほかの議題は承認済み）。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - `reg-handler-chain` が入ると、P5 は「`REG[...]` に代入すれば組み込みの受け手を差し替える。`REG:register` で重ねた受け手は先に呼ばれ、`nil` を返せば組み込み（辞書のメニュー）へ回る」と書き直す。
  - シーンを包む関数（`scene.lua` の `wrap_scene`）は最後に `act:build()` を呼び、ゴースト側の組み立て（`shiori/act.lua` の `build`）は立ち位置と表情の持ち越し（`STORE.actor_spots`・`STORE.appearance`）を直接書き換える。メニューのシーンの台詞を捨てるつもりでも、この道を通すと持ち越しが動く。その場で動かす道は、この関数を使わず、組み立てずに捨てる。
  - 自動トークを止めている間にメニューのシーンが `＞チェイントーク` を呼ぶと、区切りの見張り（`act.lua` L664）が自動トークを再開してしまう。打ち切りの扱いを決めるとき、この副作用も避ける。
  - その場で動かす道は `EVENT.drive` を通らないので、止まったシーンの控え（`STOP.hold`）も置かれない。エラーのとき（`On` で始まらない要求は 500 と `X-Error-Reason`）の出方を確かめる。
  - `OnRecommendsiteChoice` の既定の受け手は、項目のシーンが無いとき、今の `EVENT.no_entry` と同じ前方一致で `＊OnRecommendsiteChoice` へ回す（完全一致にしない）。
  - steering の roadmap の Dependencies（`recommend-menu-scenes` の行と Existing Spec Updates の行）に、入った `scene-play-event`・`ontalk-suppression` が残っている。

## 申し送り（reg-handler-chain より、2026-10-23 完了時）

- `REG:register(id, fn)` が入った。`EVENT.fire` は、コールバックと時計の後で、鎖を新しい順に呼び（偽なら次・真なら止める）、全部見送れば `rawget(REG, id) or EVENT.no_entry` を呼ぶ。リソースの ID（`sakura.recommendsites` など）も同じ道を通る（Lua テスト `reg_handler_chain_test.lua` の 2.2 で確かめ済み）。
- P5 の書き直し（上の 2026-10-21 の申し送りの「`REG:register` で重ねた受け手は先に呼ばれ、`nil` を返せば組み込みへ回る」）は、この実装のとおりで成り立つ。組み込みの受け手は `REG[...]` の生の値（鎖の底）に置けばよい。
- 鎖のハンドラはハンドラごとに新しい act を受け取る。その場でシーンを動かす道を `REG` の受け手から呼ぶときも、渡された act を使う。
- 参照: `.kiro/specs/completed/reg-handler-chain/design.md` の「Components and Interfaces > EventFire」、マニュアル `lua/shiori-events.md` の「ハンドラを重ねる（`REG:register`）」。

## 2026-10-23 棚卸の再測定（main 82176f19）

- **前提の変化**
  - 先に要るものの 3 本（`external-control-events`・`chain-talk-interrupt`・`reg-handler-chain`）がすべて入った。この spec は先頭になった。
  - `REG` の勝ち方: `REG` にメタテーブルが付いた（`shiori/event/register.lua` 117 行）。`EVENT.fire`（`event/init.lua` 309 行）は鎖（`REG:register`）を新しい順に呼び、全部見送れば `rawget(REG, id) or EVENT.no_entry`（338 行）を呼ぶ。組み込みの受け手を `REG["sakura.recommendsites"] = REG["sakura.recommendsites"] or 関数` で置けば、窓際のぱすたさんの代入は今どおり勝つ。P5 の「組み込みの受け手は作者のスクリプトより先に置かれる」は不正確で、`main.lua` は既定の登録より先に走り、既定は空のキーにだけ置く（マニュアル `lua/shiori-events.md` 133 行）。「どちらの置き場所でも作者が勝つ」と書き直す。
  - マニュアル `lua/shiori-events.md` 70 行に「`REG:register` のイベント名には `On` で始まらないリソースの ID も書ける」が入った。Problem の「どこにも書いていない」は半分閉じた。`REG[...]` に代入する作例とおすすめの節は、まだ無い。
  - チェイントークの決まり: 続きを捨てるのは、台詞（字のある `talk` か `choice`）を出した割り込み・`STORE.co_preempt` の印の付いた再生・エラーだけ（`event/init.lua` 257〜301 行の `EVENT.drive`）。台詞を出した割り込みは、`＊チェイントーク中断` があれば反応の**代わりに**それを出す。P3 の選んだ後のトークは、チェイントークが待っていると、項目のシーンの台詞でなく中断の台詞が出る。P3 の受け手は `co_preempt` の印を付けない（付けると中断の台詞を出さずに黙って捨てる）。
  - その場で動かす道は、今も自前で要る。`EVENT.drive` を通すと、メニューのシーンに書いた台詞で割り込み扱いになり、続きを捨てる。`act:build()`（`shiori/act.lua` 68 行）は台詞の印 `_spoke` を立て、立ち位置と表情の持ち越しを書き換える。
  - 失敗の出し方: `external-control-events` の手本は `FAILURE.report(act, record)` の後に `act:build()` を返す形（`external_control.lua` 29〜32 行）。`FAILURE.report`（`pasta/failure.lua` 173 行）は ID を見ないので、リソースの受け手がこれをまねると表記がリソースの値に混ざる。リソースの受け手の失敗はログの警告だけにする（マニュアル 1086 行「`On` で始まらない要求では表記を出さない」に合わせる）。`OnRecommendsiteChoice` の受け手は手本の形でよい。
  - 既定ハンドラの表は `lua/shiori-events.md` 119〜131 行（「次の 9 つのイベント」）。行を足し、数を直す。読み込みの並びは `event/init.lua` 85〜90 行で、末尾は `external_control`。
- **触るファイル**: 新しいハンドラのファイル（新規）、`event/init.lua`（371。読み込みに 1 行）、`shiori/entry.lua`（153）、`pasta_core/src/registry/scene_alias.rs`（284）、`pasta_lua/src/loader/config/mod.rs`（343。`parse_scene_aliases` 115 行）と info ログ（`loader/mod.rs` 125 行）、`pasta_check` の `builtins.rs`（63。名前 8 → 12）・`validate/config.rs`（403）、試験、マニュアル。1,000 行に近いものは無い。
- **規模**: 10〜12 タスクのまま。
- **先に要るもの**: 働きの上では無い。ファイルの重なり（未完了）: `ponytail-lua-scripts`（`init.lua`・`entry.lua`）、`ponytail-core-registry`（`scene_alias.rs`）、`ponytail-lua-loader`（`loader/config/mod.rs`・`loader/mod.rs`）、`shell-element-click`・`scene-anchor-link`・`call-attribute-filter`・`novel-checkpoint-rollback`（`init.lua`）、`novel-talk-flow`（`init.lua`・`entry.lua`・`loader/config/`）、`cue-command-runtime`（`loader/config/`・`validate/config.rs`）、`scene-exact-match`（`init.lua`・`loader/config/`）。`scene.lua` に足すなら `scene-attribute-store`・`scene-pick-modes` も。`shiori4-v3-portable`（`pasta_shiori`・`Cargo.toml`・`pasta_lua` のログ。Lua の振る舞いは持ち場の外）とは共有しない。0.4.2 に入れるなら、あちらと同じウェーブに置ける。
- **種別**: 機能（窓際のぱすたさんの実例。シーンで書く形は開発者の決定）。
- **要件定義のモデル**: Opus（議題は推奨のまま承認済み。SSP が URL を開くかは実機で確かめられる）。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - Adjacent の「`ponytail-check-release` が `builtins.rs` を触る」は誤り。あちらの持ち場は `pasta_check/src/{release,copy,nar,update_files}.rs`。
  - Upstream・Adjacent・Out of Boundary の `scene-play-event`・`external-control-events`・`ontalk-suppression`・`chain-talk-interrupt` は完了した。
  - roadmap の台帳（438・479 行）の Dependencies に、完了した 3 本が残る。「版がまだ決まっていないもの」（119 行）と「開発者の判断が要るもの」（296 行）の「3 本の後に入る・1 ウェーブ延びる」も古い。
