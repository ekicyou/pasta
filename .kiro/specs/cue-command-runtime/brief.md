# Brief: cue-command-runtime

`/kiro-discovery` で起票（2026-10-10・main `88c4bc0e`）。0.5.0「areka でノベルゲームが作れる」の道筋の 2 番目。

2026-10-17 の `/kiro-discovery` で、`scene-stage-attributes`（柱の属性とアクター指定行を、場面に入ったときに台本へ流す）をこの spec にまとめた。どちらも `grammar.pest`・`parse_scene.rs`・`scope_gen.rs` を触り、同じ `\![…]` の通り道を作る。もともと 1 本ずつ順に入れると決めていた 2 本なので、まとめても並走できる分は減らない。規模は 8〜10 と 6〜8 で、合わせて 14〜18 タスクの見当である。

## Problem

ノベルゲームの台本には、ト書きが要る。効果音を鳴らす、画面を揺らす、人物を登場させる、といった「その瞬間に起きること」である。Pasta DSL には、そのための行（キューコマンド行 `！命令＠対象（引数）`）の構文がもうある。ところが、実行時に意味を持つのは `！select` だけで、ほかの命令は何も出力せずに捨てられる。作者は、台詞の中に `\![…]` を直に書くしかない。

台本には柱も要る。Pasta DSL の根は、脚本の柱の記法である。脚本の柱は「どこで・いつ」を掲げ、その場面の間ずっと成り立つことを決める。ノベルゲームの台本では、背景・曲・誰がいるかがこれに当たる。ところが、今の pasta には、場面そのものの性質を演出として伝える手段が無い。ト書き（`！` 行）で毎回書くことはできるが、ファイル単位の受け継ぎが効かず、場面の頭が命令の列で埋まる。

## Current State

### ト書き（`！` 行）

- 構文と AST はある。`crates/pasta_dsl/src/parser/ast/cue.rs` の `CueCommandNode`（命令の名前・対象 `ScopedName{actor,name}`・引数の列 `CueArgToken`）。マニュアルは `book/src/grammar/block-structure.md` の「キューコマンド行」。
- コード生成は `crates/pasta_lua/src/code_gen/scope_gen.rs` 340〜345 行で、`select` だけを `act:choice_timeout` にし、ほかを捨てる。`scope_gen_tests.rs` 311〜322 行のテストが、この挙動を固定している。
- 命令の名前と引数には、日本語が書ける（`grammar.pest` 210・218 行）。
- 台詞の中の知らない `\![…]` は、日本語の引数を含めて、そのまま台本に残る。待ちも改行も挟まれず、文字の幅にも数えられない（`sakura_script/tokenizer.rs` 138〜146 行、`wait_inserter.rs` 77〜80 行、`line_breaker.rs` 33〜45 行）。
- `！` 行は、areka の土台（dola）の要請で入った（`.kiro/specs/completed/pasta-cue-dsl-extension/`）。当時の構想は、dola が pasta の AST を直に読んで台本にすることだった。この道は、どちらの側にも実装が無い。

### 柱（`＆属性` とアクター指定行）

- 属性 `＆名前：値` は、構文だけが受理される。実行時には、どこにも残らない（`crates/pasta_lua/src/runtime/finalize.rs` 182 行）。保持と読み出しは `scene-attribute-store` が引き受ける。
- アクター指定行 `％さくら、ゆい` は、立ち位置の番号を割り当てる（`act:clear_spot()` と `act:set_spot(name, n)` を生成する）。
- 背景は、キャラと同じくアクターとして扱う（2026-10-10 の決定）。状態はサーフェスで表す。

```pasta
％教室と窓
    ＠朝：\s[10]
    ＠稲光：\s[12]

＊嵐の教室
    ％教室と窓、さくら
    教室と窓：＠稲光
    さくら：＠驚き　きゃっ！
```

### 2026-10-10 の検証で見つかった穴（ト書き）

- **数字で始まる引数**: 引用しない引数が数字で始まり、数として読めないとき（`01教室`・`001.png`）は、パースエラーになる。数を先に試すためである（`grammar.pest` 216・142 行）。全角の数字も同じ。
- **数の書き方が消える**: `007` は整数の 7、`1.50` は小数の 1.5 になる（`parse_scene.rs` 404〜410 行）。ファイル名の一部として書いた数が変わる。
- **引数に変数を書けない**: `＄x` は、`＄x` という名前の識別子として読まれる。`＠名前` は読めるが、実行時の意味が決まっていない。
- **引数の中の記号**: `]`・`,`・`\` を含む引数は、タグを途中で切る。タグの引数を守る関数は `pasta_scripts/pasta/shiori/act.lua` 201〜212 行にあるが、そのファイルの中でしか使えない。
- `！` 行は、場面の中のインデントした行にだけ書ける。名前・対象・引数の間に空白を置けない。

### 2026-10-10 の検証で見つかった穴（柱）

- **宣言行の属性が捨てられている**: `＊名前＆k：v` と `・名前＆k：v` の属性は、パーサが読み飛ばす（`crates/pasta_dsl/src/parser/parse_scene.rs` 80〜86 行・200〜206 行）。マニュアル（`block-structure.md` の「属性」）は「記録される」と書いている。`scene-attribute-store` へ申し送った。
- **順番が消える**: 統合した属性は順番を持たない表（`HashMap`）に入る（`crates/pasta_lua/src/context.rs` 19・106〜114 行）。書いた順に流すには、順番を持つ形が要る。
- **ローカルシーンには届かない**: `generate_local_scene` は属性を受け取らない（`scope_gen.rs` 230〜236 行）。
- **直に入るローカルシーン**: 選択肢や `＞ローカル` で入るローカルシーンは、親の `__start__` を通らない（`pasta_scripts/pasta/shiori/event/choice_select.lua` 70〜77 行）。親の柱をどこで打つかを決める必要がある。
- **名前なしの `＊`**: 直前のシーンの名前だけを受け継ぎ、属性は受け継がない（`parse_scene.rs` 88〜91 行）。
- **値は 1 つ**: `＆背景：教室、夕方` は、`教室、夕方` という 1 つの文字列になる（`grammar.pest` 164〜166 行）。

## Desired Outcome

- `！` 行が、書いた順のまま、台本に `\![…]` として出る。pasta は命令の名前を知らなくてよい。語彙は areka との取り決め（`novel-areka-contract`）で増やせる。
- 日本語の名前で書ける。取り決めの名前との対応は、pasta.toml の別名表が持つ。
- `！select` の挙動は変わらない。
- ト書きの穴が、直されるか、マニュアルに決まりとして書かれている。
- 柱に書いた属性のうち、演出の語彙にある名前のものが、場面に入ったときに台本へ流れる。

```pasta
＆曲：朝の光

＊教室の朝
    ％教室と窓、さくら、ゆい
    さくら：＠笑顔　おはよう。
```

- ファイルの属性が、そのファイルの場面へ受け継がれる。1 ファイルを 1 つの場所として書ける。
- 語彙に無い名前の属性は、これまでどおりメタデータのままで、何も出力しない。Call の絞り込み（`call-attribute-filter`）とぶつからない。
- `＞別の場面` から戻ったときの扱いが決まっている。
- アクター指定行が、「この場面にいる人」として台本に伝わる。
- マニュアル・LSP の色付け・VSCode の文法定義が、新しい挙動に合っている。

## Approach

決定（2026-10-10 の discovery）:

- **ト書きは、実行時に `\![…]` として流す。** dola が AST を直に読む構想から、こちらへ切り替える。
- **柱に書いた `＆名前：値` は、場面に入ったときに自動で打たれる `！名前（値）` である。** 語彙は 1 つで、置き場所が 2 つある。柱に書けば「この場面はこう」、ト書きに書けば「今これをやる」。
- 属性の倉は 1 つで、読み手が 2 人いる。演出は場面に入るときに読み、Call の絞り込みは場面を選ぶときに読む。モードの切り替えは作らない。

脚本の 3 層との対応: 柱は `＊` と `＆属性`、ト書きは `！` 行、台詞はアクション行である。この spec は柱とト書きの出力を持つ。

要件定義で決めること（ト書き）。

- **出力の形**: 命令の名前・対象・引数を、タグの中にどう並べるか。対象のアクター名と名前の置き場所。
- **宿主に依らない形**: `act` のトークンとして 1 種類（命令・対象・引数）を足し、さくらスクリプトへの変換は SHIORI 用の組み立て（`sakura_builder.lua`）で行う見込み。
- **別名表**: pasta.toml の表の名前と形。シーン名の別名（`[scene.alias]`）と同じ流儀にするか。既定の表を持つか。
- **知らない命令**: 実行時はそのまま流す。書き間違いは、辞書の検査（`pasta-check-dic-validate`）が語彙の一覧と照らす。その検査を、どちらの spec が足すか。
- **引数**: 数字で始まる引数と数の書き方の穴を、文法で直すか、引用して書く決まりにするか。変数と `＠参照` を引数に書けるようにするか。
- **待つか待たないか**: 演出の完了を待つ指定は、命令の引数で書く。待ち方そのものは areka の仕事である。

要件定義で決めること（柱）。

- **どの属性を流すか**: 語彙の表にある名前だけを流す。表はト書きの別名表と共有する見込み。
- **場面に入るとは**: グローバルシーンの開始、ローカルシーンへの Call、選択肢からの開始、イベントからの開始。それぞれで何を打つか。
- **戻ったとき**: `＞別の場面` から戻ったら、呼び出し元の柱を打ち直す案。同じ値なら areka は何もしない、という取り決めにする。
- **受け継ぎ**: ファイルの属性はグローバルシーンへ。ローカルシーンと名前なしの `＊` へ受け継ぐか（旧仕様は「ローカルシーンには影響しない」だった）。
- **登場人物**: アクター指定行を、台本にどう伝えるか。席の並べ方は areka に任せる。
- **背景を柱にどう書くか**: アクター指定行に並べるか（`％教室と窓、さくら`）、属性にするか（`＆背景：教室`）。開発者の例は前者に近い。
- **実行時に読むか、生成時に埋めるか**: 属性は静的なので、コード生成が場面の関数の頭に命令を埋める手もある。`scene-attribute-store` の実行時の倉を読む手もある。

規模が 20 タスクを超えそうなら、要件定義の中で「ト書き」と「柱」に分け直す。

## Scope

- **In**: `！` 行のコード生成と実行時の出力。宿主に依らないトークン。別名表と、その設定の読み込み。引数の穴の手当て。柱の属性とアクター指定行を、場面に入ったときに台本へ流すこと。受け継ぎと打ち直しの決まり。テスト、マニュアル、スキル references の再生成、LSP と VSCode の追従。
- **Out**: 語彙そのもの（`novel-areka-contract`）。属性の保持・型の解釈・宣言行の属性のパース（`scene-attribute-store`）。Call の絞り込み（`call-attribute-filter`）。areka の側の解釈。

## Boundary Candidates

- 文法とパーサ（引数の穴を直す場合）。
- コード生成（`scope_gen.rs` の `！` 行と場面の関数の頭）と、属性の受け渡し（`transpiler.rs`・`context.rs`）。
- ランタイム（`act.lua` の新しい入口、`sakura_builder.lua` の変換、戻ったときの打ち直し）。
- 設定（`loader/config/`）と別名表。
- アクター指定行の出力。
- マニュアルとエディタ。

## Out of Boundary

- 台詞の中に直に書いた `\![…]` の扱い（今のまま通す）。
- 選択肢の出力と `！select`。
- サーフェスの替わり方そのもの（areka）。
- アクター辞書の単語の引き方（今のまま）。

## Upstream / Downstream

- **Upstream**: `scene-attribute-store`（属性の保持と、宣言行の属性のパース。柱の分に要る）、`novel-areka-contract`（既定の別名表に入れる名前。仕組みそのものは、取り決めを待たずに作れる）。
- **Downstream**: `novel-talk-flow`、`novel-checkpoint-rollback`（区切りの頭に「あるべき画面」が書いてあることを当てにできる）、`hello-novel-sample`、`manual-novel-guide`。

## Existing Spec Touchpoints

- **Extends**: `.kiro/specs/completed/pasta-cue-dsl-extension/`（構文を足した spec。実行時の意味は範囲外だった）。`scene-attribute-store`（属性の読み手が 1 人増える）。
- **Adjacent**: `call-attribute-filter`（同じ属性を、場面を選ぶときに読む。`act.lua`・`grammar.pest` も触る）、`scene-anchor-link`（`act.lua`・`grammar.pest`）、`choice-line-layout`（`sakura_builder.lua`）、`.kiro/specs/completed/actor-surface-restore/`（サーフェスの復旧）。

## areka への依頼

- 同じ値を打ち直されたら何もしない（曲を頭から鳴らし直さない・背景を替え直さない）。
- 「この場面にいる人」を受け取って、席に並べる。
- 出力の形（引数の並べ方・引用の決まり）が決まったら、取り決めの文書へ書き、areka のセッションへ知らせる。

手順は `novel-areka-contract` の brief の「依頼の出し方」。

## Constraints

- `act.lua` は、1 つのウェーブに 1 つの spec だけが持つ。
- `scope_gen.rs` は `scene-attribute-store` も触る。柱の分が属性の倉を使うので、この spec は `scene-attribute-store` の後に置く（2026-10-17 にまとめたため、ト書きの分もその後になる）。
- `！select` と選択肢の出力を変えない。
- 語彙に無い属性は、何も出力しない。既にある辞書の挙動を変えない。
- 既にある辞書で、`！` 行が「何もしない」ことに頼っているものがあれば、挙動が変わる。見本（`cue.pasta`）とテストの期待値を確かめる。生成コードの形を変えるので、見本の期待値ファイルも変わる。
- マニュアルが文法の唯一の権威である。挙動を変えたら、同じ変更でマニュアルを直し、スキル references を再生成する。

## 申し送り（choice-line-layout より）

2026-10-11 に `choice-line-layout` が完了し、`sakura_builder.lua` の `BUILDER.build` が、吹き出しごとに「最後に表示したもの」（`spot_tail`。`"text"` か `"choice"`）を持つようになった。選択肢の前後の改行は、この状態で決まる。詳細は `.kiro/specs/completed/choice-line-layout/design.md` の「出力層」、マニュアルの `book/src/internals/talk-output.md`。

- **見直しが要る論点**: 足すトークンを、台詞にも選択肢にも数えないもの（今の `！select` と同じ扱い）にするかどうか。数えないなら、選択肢の前後の改行は変わらず、上の Constraints の「選択肢の出力を変えない」をそのまま守れる。
- **複雑度**: `BUILDER.build` の循環的複雑度は 28 になった（`-- luacheck: ignore 561` を付けている）。分岐を足すなら、分割するかどうかも論点になる。

## 申し送り（pasta-check-dic-validate より）

- 辞書の検査は `pasta_check validate`（と `release` の段 4 の関門、VSCode 拡張のゴースト全体の診断）として入った。核は `crates/pasta_check/src/validate/`（ファイルシステムに触れないライブラリ・WASM でも動く）。
- 検査を足すときに触る所: 報告の種類 `FindingKind`（`report.rs`。誤り／注意の分類と kebab-case の code・文言の `msg` を 1 か所で持つ）、検査の段（`validate/mod.rs` の `validate_ghost` が段の順に呼ぶ）、マニュアルの章 `book/src/debug/validate.md` の報告の表。
- 種類の追加・分類・出力の 1 行の形を変えるのは本 spec の design.md の Revalidation Triggers に当たる（CI・作者のスクリプト・マニュアル・`wasmBridge.ts` の形の検査に効く）。実行時と同じことを守る一致テスト `crates/pasta_check/tests/validate_parity_test.rs` も合わせて見直す。
- どちらの spec が検査を足すか・誤りか注意かは、この spec の要件で決める論点として渡す（本 spec では決めていない）。

## 申し送り（failure-output-unification より、2026-10-17）

- `act.lua` の組み立てが変わった。話者のトークンは talk・sakura_script と、アクター付きの `raw_script`（未登録アクターの目印）。アクターの無い `raw_script`（失敗表記）は直前の話者の遡りで飛ばされ、直前の話者のバルーンに続けて出る。`act:build` はトークンが 0 件か、すべて空の `raw_script` のとき nil を返す（204）。
- 実行時の失敗は `pasta/failure.lua` の `FAILURE.report(act, record)` から出す。この spec で足す失敗も同じ出口に載せる。
- 参照: `.kiro/specs/completed/failure-output-unification/design.md` の「ActGrouping」、マニュアル `book/src/reference/runtime-failures.md`・`book/src/internals/talk-output.md`。

## 2026-10-17 棚卸の再測定（main b4164100）

- **前提の変化**（止まるエラーのバルーン表示 `runtime-error-balloon` より）:
  - 場面の関数の頭が `act:init_scene(SCENE, 書いたグローバル名, 書いたローカル名, 行の対応)` になった。柱の命令を頭に埋めるなら、その行の後に置き、生成のときに位置を記録する。記録しないと、行の対応（`crates/pasta_lua/src/transpiler.rs` の `fill_line_maps`）に入らず、その行で止まったときの表記に場所が出ない。
  - 途中の Call から戻る `call_restore`（`pasta_scripts/pasta/act.lua` 787〜795 行）は、場面の表に加えて、書いた名前 2 つと行の対応も戻すようになった。「戻ったら呼び出し元の柱を打ち直す」案は、ここか生成コードに足すことになる。止まるエラーの表記がこの 4 つに頼るので、打ち直しの途中で止まったときの名前も確かめる。
- **触るファイル**: 文法 `crates/pasta_dsl/src/parser/grammar.pest` 272（引数の穴を文法で直す場合）、`parse_scene.rs` 426、`ast/cue.rs` 99、`scope_gen.rs` 445 と単体テスト `scope_gen_tests.rs` 533、`transpiler.rs` 339・`context.rs` 454、`act.lua` 829（新しいトークンと `call_restore`）、出力の組み立て `shiori/sakura_builder.lua` 383、タグの引数を守る関数のある `shiori/act.lua` 318、設定の読み込み `crates/pasta_lua/src/loader/config/`（`mod.rs` 329・`sections.rs` 344）、直に入るローカルシーンの柱を打つなら `shiori/event/choice_select.lua` 80、VSCode の文法定義、見本とスナップショット、マニュアル。**`act.lua` は 829 行で 1,000 行に近い**。後に `call-attribute-filter`・`scene-anchor-link`・`novel-talk-flow` も足すので、新しい入口を別のファイルに置くかを設計で決める。
- **規模**: 16〜20 タスク（ト書き 9〜10: 生成 2・トークンと組み立て 2・別名表と設定 2・引数の穴 2・引数を守る関数の共有 1・エディタ 1。柱 6〜7: 場面の頭 2・入り方ごとの決まりと受け継ぎ 2・戻ったときの打ち直し 1〜2・登場人物の行 1。共通: マニュアル 2・テスト 2。語彙の検査を足すなら 1）。上限に近い。
- **先に要るもの**: `scene-attribute-store`（柱の分）。名前は `novel-areka-contract` から引く（仕組みは待たずに作れる）。ファイルの重なり: `scene-attribute-store`（`parse_scene.rs`・`scope_gen.rs`・`transpiler.rs`・`context.rs`）、`call-attribute-filter`（`grammar.pest`・`act.lua`・VSCode の文法定義）、`scene-anchor-link`（`grammar.pest`・`act.lua`・`sakura_builder.lua`・VSCode の文法定義、柱を `choice_select.lua` で打つならそれも）、`novel-talk-flow`（`act.lua`・`sakura_builder.lua`・`loader/config/`）、`novel-checkpoint-rollback`（`sakura_builder.lua`）、`shell-element-click`（柱を `choice_select.lua` で打つ場合）、語彙の検査を足すなら `pasta-check-validate-edges`（`crates/pasta_check/src/validate/mod.rs`・`report.rs`）。`tutorial-ssp-gaps`・`novel-areka-contract` とは重ならない。
- **種別**: 機能（`！select` 以外の `！` 行は、何も出さずに捨てられる）。開発者の 0.5.0 の目標（2026-10-10）と、2026-10-17 のまとめの決定から。
- **要件定義のモデル**: Fable。決めるのは、`！` 行と柱が実行時に何を出すかという文法の意味と、areka がその上に作る `\![…]` の形（別のリポジトリが頼るので取り消しにくい）。「場面に入る」の数え方（グローバルの開始・ローカルへの Call・選択肢・イベント）、戻ったときの打ち直し、ローカルシーンへの受け継ぎ、背景の書き方、引数の穴の直し方の 5 つが互いに絡み、推奨案がまだ無い。
- **分割の案**: 要件で 20 を超えたら、決めてあるとおり 2 本に戻す。`cue-command-runtime`（ト書き: `！` 行の生成・新しいトークン・組み立て・別名表と設定・引数の穴・引数を守る関数）と `scene-stage-attributes`（柱: 場面の頭の命令・入り方ごとの決まり・受け継ぎ・戻ったときの打ち直し・登場人物の行）。どちらも `scope_gen.rs` と `act.lua` を触るので、この順に 1 本ずつ入れる。並走できる分は増えない。
- **見つけた穴・古くなった記述**:
  - 「Current State」の行番号: `！` 行の生成は `scope_gen.rs` 340〜345 行 → 375〜378 行、それを固めるテストは 311〜322 行 → 318〜330 行、`generate_local_scene` は 230〜236 行 → 247〜253 行（書いたグローバル名の引数が増えたが、属性は今も受け取らない）、タグの引数を守る関数は `shiori/act.lua` 201〜212 行 → 205〜216 行。宣言行の属性を読み飛ばす所（`parse_scene.rs` 80〜86・200〜206 行）、順番を持たない表（`context.rs` 19・106〜114 行）、選択肢から直に入る所（`choice_select.lua` 70〜77 行）は合っている。

## 申し送り（pasta-check-validate-edges より）

- `pasta_check validate` の飛び先の検査は、辞書の中身が分からないとき（構文エラー・読めない辞書ファイル・pasta.toml の誤り）に丸ごと省く形になった。判定は `crates/pasta_check/src/validate/report.rs` の `skip_reasons`（統計の `skipReasons`）で、すべての段の報告がそろった後に 1 回だけ行う（`validate/mod.rs`）。
- 見直しが要る論点: この spec が `validate` に辞書のシーンの集まりに頼る検査を足すなら、`skip_reasons` が空のときだけ走らせるか。辞書の中身が分からないことを表す誤りの種類を足すなら、`skip_reasons` の対応に入れるか。
- 参照: `.kiro/specs/completed/pasta-check-validate-edges/design.md` の「Revalidation Triggers」と「Report（report.rs）」の省く理由の表。

## 申し送り（vertical-balloon-lines より、2026-10-18）

- **出力の組み立てが、吹き出しごとの列の状態を持つようになった**: `sakura_builder.lua` の `BUILDER.build` は、1 回の出力の中でスポットごとに `columns`（`{line, width, jump}`）を持ち、`talk`・`sakura_script` の変換 `SAKURA_SCRIPT.talk_to_script(actor, text, column)` に 3 つ目の引数で渡す。表情・ウェイト・`choice_timeout`・選択肢・`raw_script` は状態を動かさない。`newline` は n 列進め、`clear` は最初の列、`clear_spot` は全スポットの状態を捨てる。
- **区切りは 1 つの関数を通す**: pasta が出す区切り（段落の区切り・選択肢の前後・話者の切り替えの後の `spot_newlines`）は、局所関数 `put_break` で「出して 1 列進める」。改行を出す場所を足す・変えるときは `put_break` を通すこと（通さないと、`tests/lua_specs/sakura_builder_test.lua` の「作例・同値・不変条件」の不変条件のテストが落ちる）。
- **単独の `\` の守り**: 出力バッファ（`tracking_buffer`）が、最後の片が単位にならない `\` で終わるかを覚え、pasta が出すタグの前でエスケープの `\\` にする。新しいタグを出す所を足すときも、このバッファを通す。
- **`[ghost]` に `after_choice_newlines`（既定 1.0）が増えた**（選択肢の後の区切りの幅）。`[ghost]` のキーは 6 つになった（照合テスト・`reference/pasta-toml.md` の表・`book/AUTHORING.md` のキーの数）。
- 見直しが要る論点: この spec が `sakura_builder.lua` に改行や新しいタグを出す所を足すなら、`put_break` と `tracking_buffer` を通すか。字を出すトークンを足すなら、列の状態を `talk_to_script` に渡すか。
- 参照: `.kiro/specs/completed/vertical-balloon-lines/design.md`（SakuraBuilder の節・Revalidation Triggers・決定 7〜9）、マニュアル `book/src/internals/talk-output.md`。

## 申し送り（actor-display-style より、2026-10-18）

- **話者の切り替えで装いのタグが出るようになった**: `sakura_builder.lua` の `emit_actor_switch` は `\p[spot]` → 立ち絵の復旧（`APPEARANCE.restore`）→ 装いの出し入れ（`STYLE.switch`。`pasta/shiori/actor_style.lua`）の順に出す。装いの状態（スポットごとの `used`・`styled` と、出力全体の `any`）は、`BUILDER.build` が 1 回の出力ごとに `STYLE.new()` で作り、`emit_actor_switch` と `emit_inner_token` に渡す。
- **内側のトークンの出力は `STYLE.through` を通る**: 変換後の文字列の中で、引数の無い `\c`・`\x` の直後に話し手の装いのタグを差し込む（SSP 2.9.18 は `\c`・`\x` で文字の装飾を既定に戻すため）。最上位の `raw_script` は通さない。片を `put` と `put_raw` のどちらで足すかは、差し込む前の文字列で決める。
- **装いのあるアクターが出ない出力は 1 文字も変わらない**（`any` が偽の間、`switch` は空文字列を返し、`through` は文字列をそのまま返す）。
- 見直しが要る論点: この spec が次のどれかをするなら、装いの出し入れ（状態表と出し直し）をどう合わせるか。`emit_actor_switch` の出力順や `emit_inner_token` の形を変える。話者の切り替えの位置に新しいタグを出す。内側に `\c`・`\x` を出すトークンを足す。
- 参照: `.kiro/specs/completed/actor-display-style/design.md`（ActorStyle・BuilderHook・Revalidation Triggers）、マニュアル `book/src/internals/talk-output.md` の「装いの出し入れ」。
