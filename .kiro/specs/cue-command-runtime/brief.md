# Brief: cue-command-runtime

`/kiro-discovery` で起票（2026-10-10・main `88c4bc0e`）。0.5.0「areka でノベルゲームが作れる」の道筋の 2 番目。

## Problem

ノベルゲームの台本には、ト書きが要る。効果音を鳴らす、画面を揺らす、人物を登場させる、といった「その瞬間に起きること」である。Pasta DSL には、そのための行（キューコマンド行 `！命令＠対象（引数）`）の構文がもうある。ところが、実行時に意味を持つのは `！select` だけで、ほかの命令は何も出力せずに捨てられる。作者は、台詞の中に `\![…]` を直に書くしかない。

## Current State

- 構文と AST はある。`crates/pasta_dsl/src/parser/ast/cue.rs` の `CueCommandNode`（命令の名前・対象 `ScopedName{actor,name}`・引数の列 `CueArgToken`）。マニュアルは `book/src/grammar/block-structure.md` の「キューコマンド行」。
- コード生成は `crates/pasta_lua/src/code_gen/scope_gen.rs` 340〜345 行で、`select` だけを `act:choice_timeout` にし、ほかを捨てる。`scope_gen_tests.rs` 311〜322 行のテストが、この挙動を固定している。
- 命令の名前と引数には、日本語が書ける（`grammar.pest` 210・218 行）。
- 台詞の中の知らない `\![…]` は、日本語の引数を含めて、そのまま台本に残る。待ちも改行も挟まれず、文字の幅にも数えられない（`sakura_script/tokenizer.rs` 138〜146 行、`wait_inserter.rs` 77〜80 行、`line_breaker.rs` 33〜45 行）。
- `！` 行は、areka の土台（dola）の要請で入った（`.kiro/specs/completed/pasta-cue-dsl-extension/`）。当時の構想は、dola が pasta の AST を直に読んで台本にすることだった。この道は、どちらの側にも実装が無い。

### 2026-10-10 の検証で見つかった穴

- **数字で始まる引数**: 引用しない引数が数字で始まり、数として読めないとき（`01教室`・`001.png`）は、パースエラーになる。数を先に試すためである（`grammar.pest` 216・142 行）。全角の数字も同じ。
- **数の書き方が消える**: `007` は整数の 7、`1.50` は小数の 1.5 になる（`parse_scene.rs` 404〜410 行）。ファイル名の一部として書いた数が変わる。
- **引数に変数を書けない**: `＄x` は、`＄x` という名前の識別子として読まれる。`＠名前` は読めるが、実行時の意味が決まっていない。
- **引数の中の記号**: `]`・`,`・`\` を含む引数は、タグを途中で切る。タグの引数を守る関数は `pasta_scripts/pasta/shiori/act.lua` 201〜212 行にあるが、そのファイルの中でしか使えない。
- `！` 行は、場面の中のインデントした行にだけ書ける。名前・対象・引数の間に空白を置けない。

## Desired Outcome

- `！` 行が、書いた順のまま、台本に `\![…]` として出る。pasta は命令の名前を知らなくてよい。語彙は areka との取り決め（`novel-areka-contract`）で増やせる。
- 日本語の名前で書ける。取り決めの名前との対応は、pasta.toml の別名表が持つ。
- `！select` の挙動は変わらない。
- 上の穴が、直されるか、マニュアルに決まりとして書かれている。
- マニュアル・LSP の色付け・VSCode の文法定義が、新しい挙動に合っている。

## Approach

決定（2026-10-10 の discovery）: **実行時に `\![…]` として流す。** dola が AST を直に読む構想から、こちらへ切り替える。

脚本の 3 層との対応: 柱は `＊` と `＆属性`（`scene-stage-attributes`）、ト書きは `！` 行（この spec）、台詞はアクション行である。

要件定義で決めること。

- **出力の形**: 命令の名前・対象・引数を、タグの中にどう並べるか。対象のアクター名と名前の置き場所。
- **宿主に依らない形**: `act` のトークンとして 1 種類（命令・対象・引数）を足し、さくらスクリプトへの変換は SHIORI 用の組み立て（`sakura_builder.lua`）で行う見込み。
- **別名表**: pasta.toml の表の名前と形。シーン名の別名（`[scene.alias]`）と同じ流儀にするか。既定の表を持つか。
- **知らない命令**: 実行時はそのまま流す。書き間違いは、辞書の検査（`pasta-check-dic-validate`）が語彙の一覧と照らす。その検査を、どちらの spec が足すか。
- **引数**: 数字で始まる引数と数の書き方の穴を、文法で直すか、引用して書く決まりにするか。変数と `＠参照` を引数に書けるようにするか。
- **待つか待たないか**: 演出の完了を待つ指定は、命令の引数で書く。待ち方そのものは areka の仕事である。

## Scope

- **In**: `！` 行のコード生成と実行時の出力。宿主に依らないトークン。別名表と、その設定の読み込み。引数の穴の手当て。テスト、マニュアル、スキル references の再生成、LSP と VSCode の追従。
- **Out**: 語彙そのもの（`novel-areka-contract`）。柱の属性を流すこと（`scene-stage-attributes`）。areka の側の解釈。

## Boundary Candidates

- 文法とパーサ（引数の穴を直す場合）。
- コード生成（`scope_gen.rs`）。
- ランタイム（`act.lua` の新しい入口、`sakura_builder.lua` の変換）。
- 設定（`loader/config/`）と別名表。
- マニュアルとエディタ。

## Out of Boundary

- 台詞の中に直に書いた `\![…]` の扱い（今のまま通す）。
- 選択肢の出力と `！select`。

## Upstream / Downstream

- **Upstream**: `novel-areka-contract`（既定の別名表に入れる名前。仕組みそのものは、取り決めを待たずに作れる）。
- **Downstream**: `scene-stage-attributes`（柱の属性は、場面に入ったときに自動で打たれる `！` 行である）、`novel-talk-flow`、`hello-novel-sample`。

## Existing Spec Touchpoints

- **Extends**: `.kiro/specs/completed/pasta-cue-dsl-extension/`（構文を足した spec。実行時の意味は範囲外だった）。
- **Adjacent**: `failure-output-unification`（`act.lua` と `loader/config/` を触る）、`call-attribute-filter`・`scene-anchor-link`（`act.lua`・`grammar.pest`）、`choice-line-layout`（`sakura_builder.lua`）。

## areka への依頼

この spec から新しく出る依頼は無い見込みである。出力の形（引数の並べ方・引用の決まり）が決まったら、取り決めの文書へ書き、areka のセッションへ知らせる。手順は `novel-areka-contract` の brief の「依頼の出し方」。

## Constraints

- `act.lua` は、1 つのウェーブに 1 つの spec だけが持つ。`failure-output-unification`（0.4.0）の後に置く。
- `！select` と選択肢の出力を変えない。
- 既にある辞書で、`！` 行が「何もしない」ことに頼っているものがあれば、挙動が変わる。見本（`cue.pasta`）とテストの期待値を確かめる。
- マニュアルが文法の唯一の権威である。挙動を変えたら、同じ変更でマニュアルを直し、スキル references を再生成する。
