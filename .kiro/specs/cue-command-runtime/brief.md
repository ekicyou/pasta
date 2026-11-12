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

## 申し送り（2026-10-21 の discovery「PastaDSL機能ヒアリング」より）

ghost_dev の「ぱすた開発」セッションは、**外見として記録させない**生のさくらスクリプトを、Lua の `act:raw_script` で出している。DSL の行に書くと、話者の外見として記録され、復旧で出し直されるからである。

- 例: 窓の「閉じる」ボタンのホバーの `\![bind,閉じる,赤,1/0]`、セキュリティボールの OFF、`\![move,…]`・`\![set,zorder,…]`、初回の演出の `\0\s[1001]`・`\s[1002]`（窓際のぱすたさん `window.pasta`・`ball.pasta`・`tategaki.pasta`・`boot.pasta`）。
- 作者の想像の書き方: 行頭の印で「加工も外見の記録もせずにそのまま出す」と指定する（`！raw \![bind,閉じる,赤,1]`）。
- この spec が `！命令` を `\![…]` として流すとき、その出力を外見に記録するか、話者に結び付けるかを要件で決める。`raw_script` は外見の上ではどのスポットも不明にする（`book/src/grammar/actor-dictionary.md` L217）ので、同じ扱いにするかも含める。
- 作者は「bind を書くと台詞の加工でバルーンに `閉じる,赤,1` と文字で出る」とも報告した。調べた範囲では、行に書いた `\![bind,閉じる,赤,1]` は 1 つのタグとして通り、ウェイトも BudouX の改行も入らない。壊れるのは、単語の値に `「」` で囲まずに書いたとき（`、`・`,` が値の区切りになる）である。再現の手順は残っていない。作者のメモでは、同じ時期に Lua の長い括弧を書き違えて末尾の `]` が欠けていた（SSP が「Unclosed [」と出した）ので、それを取り違えた見込みが高い（2026-10-21「ぱすた開発」の返信）。不具合として扱わない。
- 2026-10-21 に SSP の実機で確かめた（窓際のぱすたさんの複製）。トークの頭で `pasta_layout.lua` が `raw_script` を使って表情を 0000 に戻した。その後に相手役が話し、ぱすたへ戻ったところで、復旧が**前のトークの腕と口**を出し直した。`raw_script` の出力は外見の記録に残らず、アクターの記録には前のトークのパーツが残っているからである。台詞が表情の単語で始まれば上書きされるが、表情の単語の無い台詞では前のトークのパーツが残る。`！命令` の出力を記録しないと決めるなら、同じことが起きる。要件では、この実例を前提に決める。
- カテゴリの排他の扱いは、`dressup-category-exclusive` を実機の確認の後に取り下げたので、roadmap のバックログ「着せ替えの復旧の出力の膨らみ」に残っている。

## 申し送り（recommend-menu-scenes より、2026-10-24）

- 「別名表を `[scene.alias]` と同じ流儀にするか・既定の表を持つか」の参考: `[scene.alias]` は既定の表（7 件）を持ち、書いたキーだけ既定を置き換える形になった（空の配列で既定を消す・見出しだけなら既定のまま・既定とぶつかれば読み込みエラー）。合成の規則は `pasta_core` の `SceneAliasTable::with_written` の 1 か所にあり、pasta.dll と `pasta_check` が同じ規則で読む。
- 参照: `.kiro/specs/completed/recommend-menu-scenes/design.md` の「AliasTable」、マニュアル `grammar/call-jump.md` の「シーン名の別名」。


## 申し送り（2026-10-29 の discovery「シーン属性による候補の絞り込み」より）

- `call-attribute-filter` は閉じて、新しい spec `scene-condition-attributes` に統合した（開発者の決定）。この brief の `call-attribute-filter` は `scene-condition-attributes` と読み替える。
- 中身が変わった。条件は呼ぶ側（`＞シーン＆k＝v`）でなく、場面の側に書く（`＊雑談_今の部屋　＆＄＊経験値≧500`）。
  - 呼ぶ側の Call の文法（`call_scene`・`parse_action.rs`・`element_gen.rs` の Call）は触らない見込みである。
  - 触るのは、属性の文法、`scene.lua`・`act.lua` の検索の道、`search/context.rs`・`scene_table.rs`、イベントからシーンを起こす道（選ぶ前の自動転記）、`pasta_check validate`、LSP と VSCode の文法定義である。
- 参照: `.kiro/specs/completed/scene-condition-attributes/brief.md`。
- この spec への影響: `＄` で始まる属性は条件である。柱から台本へ流す語彙の名前とは取り合わない（`＄` で始まる名前を語彙に入れない）。

## 申し送り（2026-10-29 の版の組み替えより）

- ノベルゲーム（Phase 13）は 0.5.0 から 0.6.0 へ順延した（開発者の決定）。この brief の「0.5.0」は、ノベルゲームの版のことで、今は 0.6.0 と読む。0.5.0 は「属性関係の整備」（`scene-attribute-store`・`scene-condition-attributes`・`scene-pick-modes`）になり、0.4.5（ponytail）の次に入る。
- 柱の属性の土台 `scene-attribute-store` は 0.5.0 で先に入る。その要件は、この spec の柱の属性も読み手として材料にする（`roadmap.md` の「0.5.0 の範囲」）。

## 申し送り（2026-11-03・`scene-attribute-store` の要件ディスカッションより）

- **ローカルシーンは、ファイルの属性と親の属性を受け継がない**（開発者の決定。議題 1。この brief の「要件定義で決めること（柱）」の「受け継ぎ」はこれで決まった）。ローカルシーンの属性は、その宣言行に付記したものだけ。親の柱が要るときは、親の登録名で `SCENE.attrs(親の登録名)` を読み、ローカル自身の `SCENE.attrs(親の登録名, ローカル名)` と合わせる。直に入るローカルシーン（選択肢・`＞ローカル`）で親の柱を打つ位置は、この spec が決める。
- **倉の形**: シーンごとの、書いた順の並び。項目は 3 種で、宣言 `{key, value}`・条件 `{key="＄…", op, value}`・フラグ `{key="＄…"}`。語彙の名前で宣言の項目だけを選べば、条件は自然に外れる（`＄` で始まる名前を語彙に入れない）。ファイル属性は常に 1 行で、後の行は前の行を丸ごと置き換える（累積しない）。並びは「ファイル属性が先頭、続けてシーンの属性」で、同じ名前の宣言は先に書いた位置に 1 つ、値は後出しが正（議題 4。開発者の決定）。柱の順はファイル属性行の順が骨になり、シーン側は値を差し替える。束（ファイル内の場面全部）に効かせたい柱はファイル属性で書く。
- 名前なしの `＊` は名前だけを受け継ぎ、属性は受け継がない（この brief の「要件定義で決めること（柱）」の「受け継ぎ」はこれで決まった）。その代わり `＊＆背景：屋上` のように名前なしの `＊` 行にも属性を付記できる（議題 2。開発者の決定）。

## 申し送り（2026-11-03・`scene-attribute-store` の設計ディスカッションより）

- **`act` から読めるのは実行中のグローバルシーンの属性だけ**: `SCENE.attrs(act.current_scene.__global_name__, nil)`。`act` はローカルの登録名（`選択肢_1`）を持たない（`act.local_scene_name` は書いた名前の `選択肢`）ので、ローカルシーンの属性は検索の結果の登録名（`global_name`・`local_name`）で読む。場面に入るところで柱を打つなら、検索の結果を持っている入口（`act:call` など）で読むことになる。`act.lua` に登録名を持たせるなら、この spec の設計で決める（`scene-attribute-store` は `act.lua` を触らない）。
- 整数の値になる小数（`＆率：1.0`）は整数と同じに扱われる（Lua では 1）。

## 申し送り（actor-line-newlines より、2026-11-06）

- **変わった事実（生成）**: トランスパイラ（`code_gen/element_gen.rs` の `generate_line_marker`）が、アクション行と継続行の 1 行ごとに、その行の呼び出しより前に `act:line()` を 1 行出すようになった（InvalidContinuation の検査の後。ソースマップはその行の位置）。生成 Lua の行番号がずれるので、行番号の重なりに頼るテスト材料は見直すこと。`pasta_check` の act のメソッドの一覧（`validate/builtins.rs` の `ACT_METHODS`）は 28 になった。
- **変わった事実（トークン）**: `act:line()` は `[ghost] line_newlines` が 0 でないときだけ印 `_seam_next` を立てる。次の `talk`・`sakura_script` のトークンが `seams = { 1 }`（字のバイト位置。昇順・1 始まり）を持ち、続く同じ話し手のトークンをまとめるとき（`merge_consecutive_talks`）にも位置を足して保つ。`raw_script` は印を取らない。`act.lua` の局所関数 `take_seam` を参照。
- **変わった事実（出力）**: `sakura_builder.lua` の列の状態に `joinable`・`line_pending` が加わった。継ぎ目の改行（`\n[N]`、N = floor(値×100)）は、トークンの字の中の継ぎ目の位置に `join_lines` が差し込む（`put_break` は通らない）。`next_column`・`first_column` がこの状態を消し、話し手の切り替え（S1）では `break_line` が切り替え先の状態を消す。詳細は `.kiro/specs/completed/actor-line-newlines/design.md` と `book/src/internals/talk-output.md`。
- 見直しが要る論点: この spec で、字を出すトークンを足すなら `take_seam` で継ぎ目を取るか。改行・区切りを出す場所を足すなら、その後に `joinable`・`line_pending` を消すか（`next_column` を通せば消える）。行の頭の生成（`element_gen.rs`）を触るなら、`act:line()` の位置（呼び出しの前）を保つか。

## 2026-11-12 棚卸の再測定（main 2381bf52）

- **前提の変化（柱の倉）**: 統合した属性は書いた順の並びになった（`context.rs` 79 行の `scene_attrs_of`・`pasta_dsl` の `ast/attr_merge.rs` の `merge_attrs`。`HashMap` ではない）。生成は場面の関数の頭の行の後ろに `; PASTA.set_scene_attrs(SCENE, "関数名", {…})` を同じ行で足す（`scope_gen.rs` 112〜125・414 行。行を足すと `init_scene` の行の対応がずれるため）。柱の命令を生成で埋めるなら、同じ決まりに従う。受け継ぎは 2026-11-03 の申し送りで決まった。
- **前提の変化（予約の名前）**: 0.5.0 で、シーンの選び方を変える属性が 6 つになった。`＆優先`/`priority`（`scene-condition-attributes`）と `＆重み`/`weight`・`＆順番`/`index`（`scene-pick-modes`）。名前は `pasta_core/src/registry/scene_types.rs` 158・185・188 行の `PRIORITY_KEYS`・`WEIGHT_KEYS`・`ORDER_KEYS`、値の型の検査は `pasta_check` の `transpile_rules.rs` 127〜140 行。柱から流す語彙には、`＄` で始まる名前に加えて、この 6 つを入れない。語彙の表と照らす検査を足すなら、この 6 つは「柱でない名前」として通す。
- **前提の変化（行の頭）**: `actor-line-newlines` で、アクション行と継続行の 1 行ごとに、呼び出しの前に `act:line()` が出る（`element_gen.rs` 316 行の `generate_line_marker`・`act.lua` 248 行）。`！` 行のトークンが継ぎ目の印を取らない（`raw_script` と同じ）か、取るかを決める。`sakura_builder.lua` は 557 → 690 行、`act.lua` は 812 → 857 行。
- **前の節から持ち越す作りの条件**: 柱の命令は `act:init_scene(…, 行の対応)`（`act.lua` 221 行）の後に置く。途中の Call から戻ったときの打ち直しは `restore_scene`（800 行）か生成コードに足す（`call_restore` 821 行はそれを呼ぶだけ）。ト書きのトークンは `tracking_buffer` の `put` で足し、字にも選択肢にも台詞にも数えず（S4 の枝 `sakura_builder.lua` 663 行〜・`BUILDER.speaks` 542 行）、列を動かさない（`inner_token_to_string` 437 行）。話者の切り替えに出すなら `emit_actor_switch`（72 行）のどこに置くかを決める。中断の台詞の差し替え（`shiori/act.lua` 86〜89 行）で消える柱も扱う。別名表を読み込みで検査するなら `[actor."名前"].font` と同じ 3 か所の形（`pasta_core`・`loader/config/`・`validate/config.rs`）で、`validate_parity_test.rs` で一致を確かめる。アクター指定行の「この場面にいる人」は、スポットの番号でなく名前で伝える。
- **触るファイル**: `grammar.pest` 285（`！` 行の規則は 220〜229 行。単語の重みとはぶつからない）・`parse_scene.rs` 444・`ast/cue.rs` 99・`scope_gen.rs` 498（`！select` だけを生成するのは 357・395〜408 行）・`scope_gen_tests.rs` 690・`transpiler.rs` 292・`context.rs` 295・`act.lua` 857（新しい入口を別のファイルに置くかを設計の最初の議題にする）・`shiori/sakura_builder.lua` 690・`shiori/act.lua` 375・`loader/config/`（`mod.rs` 359・`sections.rs` 324）・`validate/config.rs` 455・`choice_select.lua` 73・VSCode の文法定義 408、見本とスナップショット、マニュアル。`act.lua`・`sakura_builder.lua` は、この spec・`novel-talk-flow`・`novel-checkpoint-rollback`・`scene-anchor-link` が足すと 1,000 行に届きうるので、足す分は別のファイルに置く。テストは `sakura_builder_test.lua`（3,638 行）に足さず、新しいファイルに置く。
- **規模**: 17〜21 のまま（受け継ぎが決まって軽くなった分と、予約の名前を除く分・継ぎ目の印の扱いが増えた分で差し引き同じ）。
- **先に要るもの**: 機能の上ではなし（`scene-attribute-store` は入った。名前は `novel-areka-contract` から引くが、待たずに作れる）。ファイルの重なり（未完了）: 0.6.0 の `novel-talk-flow`（`act.lua`・`sakura_builder.lua`・`loader/config/`・`validate/config.rs`）・`novel-checkpoint-rollback`（`sakura_builder.lua`）・`novel-variable-scope`（`grammar.pest`・`scope_gen.rs`）・`shell-element-click`（柱を `choice_select.lua` で打つ場合）、版の決まっていない `scene-anchor-link`（`grammar.pest`・`act.lua`・`sakura_builder.lua`・`choice_select.lua`・VSCode の文法定義）・`scene-exact-match`（`act.lua`・`choice_select.lua`・`loader/config/`）。0.5.0 の 3 本と `debug-mcp-runtime` は入って外れた。
- **種別**: 機能（`！select` 以外の `！` 行は何も出さずに捨てられ、柱の属性は実行時に台本へ流れない）。開発者の 0.6.0 の目標と、2026-10-17 のまとめの決定から。
- **要件定義のモデル**: Fable のまま。柱とト書きが実行時に何をいつ出すかという DSL の意味で、areka がその `\![…]` の上に作る。受け継ぎは決まったが、「場面に入る」の数え方（グローバルの開始・ローカルへの Call・選択肢・イベント）、生成で埋めるか実行時に読むか、引数の穴の直し方、出力を外見に記録するか（2026-10-21 の申し送り）の 4 つが絡み、推奨案がまだ無い（3 つ以上の絡む問い）。
- **分割の案**: 20 をはっきり超えたら `cue-command-runtime`（ト書き）と `scene-stage-attributes`（柱）に戻す。どちらも `scope_gen.rs`・`act.lua` を触るので 1 本ずつ入れる。21 にわずかに届くだけなら分けない。
- **見つけた穴・古くなった記述**: 「Current State」の「順番が消える（`HashMap`）」は解けた（上の柱の倉）。行番号: `！` 行の生成 → `scope_gen.rs` 357・395〜408 行、`generate_local_scene` → 230 行、命令の名前と引数の規則 210・218 行 → `grammar.pest` 223・229 行、`shiori/act.lua` の `escape_tag_arg` 245 行 → 249 行、`choice_select.lua` の直に入るローカルシーン 60 行 → 70 行（`SCENE.wrap`）。
