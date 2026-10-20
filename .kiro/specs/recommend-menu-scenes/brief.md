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
＊おすすめ
  ＞おすすめ項目（「カクヨム」、「https://kakuyomu.jp/」、「おすすめ_カクヨム」）
  ＞おすすめ項目（「青空文庫」、「https://www.aozora.gr.jp/」）

＊おすすめ
  ＞おすすめ項目（「小説家になろう」、「https://syosetu.com/」）
  ＞おすすめ区切り
  ＞おすすめ項目（「青空文庫」、「https://www.aozora.gr.jp/」）

＊相方おすすめ
  ＞おすすめ項目（「IPA セキュリティ」、「https://www.ipa.go.jp/security/」）

＊おすすめ_カクヨム
  ぱすた：小説を探しに行くのね。いってらっしゃい。

＠おすすめボタン名：本棚、おすすめ
```

- メニューを開くたびに、2 本の `＊おすすめ` のどちらかの並びが出る（同じ名前のシーンはランダムに選ばれる）。
- カクヨムを選ぶと、`＊おすすめ_カクヨム` が話す。
- メニューを開いても、待っているチェイントークの続きは消えない。
- 別名を 1 つ足したいだけなら、足したいキーだけを `[scene.alias]` に書けばよい。既定の別名は残る。

## Approach

2026-10-20 の話し合いで、開発者が決めた。

- 最初は pasta.toml の表（`[[recommend.sakura]]`）を推したが、開発者が前提から変えた。「シーンにしておくと、同じ名前のエントリがランダムに選ばれ、メニューを開くたびに別のものを出せる。ゴーストの挙動は DSL で書く、という精神にも合う。pasta.toml はとっつきにくい」。
- シーンで書く形への懸念だった 2 点（台詞の加工で `\1`・`\2` や URL が壊れる、メニューを開くたびにチェイントークが消える）は、下の P2 と「動かし方」で避ける。

### 前提（P1〜P6。議案をすべて推奨のまま承認）

1. **P1. シーンの名前とリソースの結びつけ**: シーン名の別名（`[scene.alias]`）の既定の表に足す。
   - `"sakura.recommendsites" = ["おすすめ"]`、`"kero.recommendsites" = ["相方おすすめ"]`、`"sakura.portalsites" = ["ポータル"]`。
   - 名前を変えたい作者は、`[scene.alias]` で変える。`char2.recommendsites` なども、別名を足せば書ける。
   - リソースのシーンは**完全一致**で引く。前方一致だと、`＊おすすめ` で `＊おすすめ_カクヨム` まで当たる。
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
   OnBoot = ["起動"]                  # 足すだけ。会話・おすすめなどの既定は残る
   OnTalk = ["会話", "おしゃべり"]   # OnTalk の別名だけが替わる
   ```

2. **A2. 既定の別名を消したいときは、そのキーに空の配列を書く**（`"sakura.recommendsites" = []`）。
3. **A3. 見出しだけの `[scene.alias]` は、既定のまま**（何も書かないのと同じ）。今の「別名なし」から意味を変える。
4. **A4. 書いた別名が、残っている既定の別名とぶつかったら、読み込みエラー**（今の重複と同じ扱い）。例: `OnBoot = ["会話"]` は既定の `OnTalk = ["会話"]` とぶつかる。エラーの文に、相手が既定の表のものであることと、直し方（`OnTalk = []`）を入れる。
5. **A5. これまでのゴーストへの影響は、マニュアルの変更点に書くだけ**。
   - `＊おすすめ` という名前でふつうのシーンを書いていた作者は、そのシーンがおすすめメニューに使われる。`＞おすすめ` で呼ぶ道は、別名が呼ぶ側にも効くので、そのまま動く。困るなら A2 で戻せる。
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
  - `ponytail-check-release`（0.4.3）: `builtins.rs` を触る。この spec が先に入る。

## Constraints

- 別名の置き換えの規則（完全一致・1 段だけ・グローバルシーン名だけ・登録と検索の両方に効く）は変えない。変えるのは表の作り方だけ。
- 別名の表の指紋（`.scene_alias`）でキャッシュを無効にする仕組みは保つ。既定の表が変わるので、版を上げた最初の読み込みでキャッシュが作り直される。
- 非 ASCII の識別子は、ブラケット記法で定義する（`GLOBAL["おすすめ項目"]`）。
- 規模は 10 タスク前後の見込み（別名の表 2、受け手とその場で動かす道 2、組み込みの呼び出し 1、選んだ後の受け手 1、ボタンの名前 1、pasta_check 1、テスト 1、マニュアルとスキル 1）。
