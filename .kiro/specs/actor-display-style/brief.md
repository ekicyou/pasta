# Brief: actor-display-style

`/kiro-discovery` で起票（2026-10-18・main `838fbde8`）。ghost_dev の「ぱすた開発」セッション（窓際のぱすたさん）と相談して決めた。0.4.1 に入れる（2026-10-18 開発者の決定）。

## Problem

窓際のぱすたさんには、シェル（立ち絵）の無い相手役がいる（おじさん・男の子・ギャル・妹）。シェルはぱすた（`\0`）の 1 人分しかない。`\1` は「窓」のシェルで、台詞を持たない（2026-10-18 開発者の決定「案C」）。だから、相手役の台詞はぱすたのバルーンに混ぜるしかない。混ぜると、誰の台詞かが絵で分からなくなる。作者は、アクターごとに文字の色などを変えて見分けたい。

ネタ帳は、相手役もアクターとして `おじさん：やあ。` と書いてある（2026-10-17 に全ネタ帳を pasta DSL へ書き換えた）。ところが pasta には、アクターごとの表示の装いを決める設定が無い。そのため、ゴースト側の試作は、アクター辞書の Lua 関数で回避している。

```pasta
＃ 今の回避（ゴースト側の試作。未コミット）
　ぱすた：＠おじさん　やあ、相変わらず薄着だね。
　：＠ぱすた　＠笑顔　やあ、相変わらずエッチですね。
```

この書き方は、ネタ帳のとおりに写せない。ぱすたが単語でおじさんに化ける形で、書いたとおりの意味にならない。

## Current State

- アクターは、pasta.toml の `[actor."名前"]` に `spot = N` で載る。アクターの値は TOML の表そのままで、知らないキーも Lua から読める（`crates/pasta_lua/pasta_scripts/pasta/store.lua` 112〜121 行、マニュアル `reference/pasta-toml.md` 399 行）。
- 出力の組み立ては、アクターが替わると `\p[spot]` と立ち絵の復旧を出す（`pasta_scripts/pasta/shiori/sakura_builder.lua` の `emit_actor_switch`、70〜83 行）。`\p` と台詞の間に、アクターごとのタグを差し込む口は無い。
- **同じスコープで話者が替わったときの改行は、もうある**。替わった先のスポットに字が出ていれば、次の字の直前に `\n[spot_newlines×100]` を出す（`sakura_builder.lua` 310〜318・345〜347 行）。`[ghost] spot_newlines = 1.8` にすると `\n[180]` になる。縦書きのバルーンでは、これで列が替わる。
- シェルの無いアクターは、`surface` を書かなければ立ち絵の復旧を出さない（`pasta_scripts/pasta/shiori/appearance.lua` 388〜391 行）。ぱすたと同じスポットに置いても、ぱすたの立ち絵は崩れない見込み。テストもマニュアルも無い。
- 登録していない名前で書くと、台詞は `\p[0]` に出て、未登録アクターの失敗の表記が付く（`pasta_scripts/pasta/act.lua` 518〜541 行）。
- 作者が台詞に `\f[color,…]` を書けば、そのまま出る（マニュアル `grammar/sakura-script.md` 118 行）。ただし、話者が替わるたびに書き直す必要がある。

## Desired Outcome

ネタ帳のとおりに書いて、アクターごとの装いで出る。

```pasta
＊おじさん_薄着いじり
　おじさん：やあ、相変わらず薄着だね。寒くないの？
　ぱすた：やあ、相変わらずエッチですね。この部屋は空調完璧だから大丈夫。
```

```toml
[ghost]
spot_newlines = 1.8

[actor."おじさん"]
spot = 0
font = { color = "90,110,140" }   # キーの形は要件定義で決める
```

```
\p[0]\f[color,90,110,140]やあ、相変わらず薄着だね。寒くないの？\f[default]\n[180]やあ、相変わらずエッチですね。この部屋は空調完璧だから大丈夫。\e
```

- 1 つのスポットを複数のアクターで共有する形を、正式に支える（テストとマニュアル）。
- ゴースト側の Lua の回避は要らなくなる。

## Approach

`emit_actor_switch` の隣で、アクターの装いのタグを出す。アクターを離れるときに、装いを戻す。要件定義で決めること:

- **何を変えられるか**: SSP の `\f` タグの名前をそのままキーにする（`color`・`name`・`height`・`bold`・`italic` など）か、生のさくらスクリプトの前置きを 1 つ書けるようにするか。推奨は前者（戻すときに `\f[default]` で済み、入門者に書きやすい）。
- **名前を添えるか**（例: 「おじさん」と小さく添える）。この spec に含めるかを、要件定義の最初の議題にする（2026-10-18 開発者は未決）。
- **いつ出すか・いつ戻すか**: スポットが替わったとき、同じスポットで話者が替わったとき、`\c` の後、選択肢の後。SSP がスコープの切り替えで `\f` を保つかを、実機で確かめる。
- **台詞に書いた `\f` との関係**: 作者が台詞の中で書いた `\f` を、装いが上書きしないこと。
- **既定は今の動き**: 装いを書かないアクターの出力は、1 文字も変えない。

## Scope

- **In**: pasta.toml のアクターの装いのキー、出力の組み立てでの出し入れ、スポットの共有の正式な支え（同じスポットで話者が替わるときの改行・立ち絵の復旧を確かめるテスト）、`pasta_check validate` が装いのキーの書き間違いを報告するか（要件で決める）、マニュアル（`reference/pasta-toml.md`・`grammar/actor-dictionary.md`・`reference/shell.md`・`internals/talk-output.md`）、スキル references の再生成。
- **Out**: budoux の改行の幅・数え直しと、選択肢どうしの間の改行の幅（`vertical-balloon-lines`。`sakura_builder.lua` を共有するので同じウェーブに置かない）。縦書きの左の枠へ飛ぶ `\_l`（ゴースト側の単語）。ノベル向けの話者名（`novel-talk-flow`）。

## Boundary Candidates

- 設定の読み出し（アクターの表から装いを取り出して、タグに直す）。
- 出力の組み立ての状態（今の装い・戻す時機）。
- マニュアルの「1 つのスポットを複数のアクターで共有する」節。

## Out of Boundary

- バルーンそのもの（縦書き・既定の文字色・書体は、バルーンの descript で決める。ゴースト側）。
- シェルの無いアクターに、別のバルーンを立てること（案C で `\1` は窓になった）。
- アクター辞書（`％名前`）の文法。

## Upstream / Downstream

- **Upstream**: 出力の組み立て（`sakura_builder.lua`）、立ち絵の復旧（`appearance.lua`）、設定の読み込み（`store.lua`）。
- **Downstream**: 窓際のぱすたさん（ghost_dev）。0.5.0 の `novel-talk-flow`（話者名の出力。装いと話者名が同じ所で出るなら、出し方をそろえる）。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**: `novel-talk-flow`・`cue-command-runtime`・`novel-checkpoint-rollback`（`sakura_builder.lua` を共有する。同じウェーブに置かない）。`scene-anchor-link`（同じく `sakura_builder.lua`）。

## Constraints

- 装いを書かないゴーストの出力を変えない（hello-pasta と既存の期待値ファイルが、そのまま通ること）。
- `act.lua` を触らずに済むなら触らない（1,000 行に近い。roadmap の共有接点）。
- ゴースト側の実機は SSP 2.9.18。縦書きバルーン（`vertical,1`）は SSP 2.8.80 から。

## ゴースト側の材料（2026-10-18 時点）

- 試作のファイル: ghost_dev の `project/pasta-in-windows/ghost/pasta-in-windows/`（`dic/actors.pasta`・`dic/tategaki.pasta`・`pasta_tategaki/descript.txt`。未コミット）。
- 設定の正本: 同じく `project/pasta-in-windows/窓際のぱすたさん/doc/`（`characters.md`・`設定_世界観.md`・`ネタ_*.md`）。
- 相手役に人名は出さない（アクター名は役割の呼び名そのもの）。ユーザーはアクターではない。

## 申し送り（vertical-balloon-lines より、2026-10-18）

- **出力の組み立てが、吹き出しごとの列の状態を持つようになった**: `sakura_builder.lua` の `BUILDER.build` は、1 回の出力の中でスポットごとに `columns`（`{line, width, jump}`）を持ち、`talk`・`sakura_script` の変換 `SAKURA_SCRIPT.talk_to_script(actor, text, column)` に 3 つ目の引数で渡す。表情・ウェイト・`choice_timeout`・選択肢・`raw_script` は状態を動かさない。`newline` は n 列進め、`clear` は最初の列、`clear_spot` は全スポットの状態を捨てる。
- **区切りは 1 つの関数を通す**: pasta が出す区切り（段落の区切り・選択肢の前後・話者の切り替えの後の `spot_newlines`）は、局所関数 `put_break` で「出して 1 列進める」。改行を出す場所を足す・変えるときは `put_break` を通すこと（通さないと、`tests/lua_specs/sakura_builder_test.lua` の「作例・同値・不変条件」の不変条件のテストが落ちる）。
- **単独の `\` の守り**: 出力バッファ（`tracking_buffer`）が、最後の片が単位にならない `\` で終わるかを覚え、pasta が出すタグの前でエスケープの `\\` にする。新しいタグを出す所を足すときも、このバッファを通す。
- **`[ghost]` に `after_choice_newlines`（既定 1.0）が増えた**（選択肢の後の区切りの幅）。`[ghost]` のキーは 6 つになった（照合テスト・`reference/pasta-toml.md` の表・`book/AUTHORING.md` のキーの数）。
- 見直しが要る論点: この spec が `sakura_builder.lua` に改行や新しいタグを出す所を足すなら、`put_break` と `tracking_buffer` を通すか。字を出すトークンを足すなら、列の状態を `talk_to_script` に渡すか。
- 参照: `.kiro/specs/completed/vertical-balloon-lines/design.md`（SakuraBuilder の節・Revalidation Triggers・決定 7〜9）、マニュアル `book/src/internals/talk-output.md`。
