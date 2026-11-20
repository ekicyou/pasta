# Brief: choice-condition-timeout

2026-11-20 の棚卸（`/kiro-next`・main `58aba27c`）で、`novel-talk-flow` から文法の分を切り出して起票した（経緯は `novel-talk-flow/brief.md` の「2026-11-20 棚卸の分割」。分け方は、`novel-talk-flow/provisional-design.md` の 27 の案のとおり）。ノベルゲーム（Phase 13。版は 0.7.0）の道筋の 1 本で、フェーズ 1（今の SSP で動く）に入る。走り方（`＆再生モード`）を待たない。

## Problem

選択肢には、2 つの足りない所と、1 つの食い違いがある。

- フラグが立っているときだけ出す選択肢を書けない。今の選択肢の行（`＠？飛び先「表示」`）は、飛び先と表示だけである。
- 選択肢が時間切れになったときに、どのシーンへ進むかを、選択肢の側に書けない。
- `！select` の時間だけが秒である。台本に書くほかの時間（ト書き・`＞ゴースト終了（３００）`）と、さくらスクリプトは、ミリ秒である。

## Current State

- 選択肢の行の文法は `crates/pasta_dsl/src/parser/grammar.pest` 236 行の `choice_line`（`pad ~ word_marker ~ question_marker ~ id ~ choice_label? ~ or_comment_eol`）。構文木は `crates/pasta_dsl/src/parser/ast/scene.rs` 195 行の `ChoiceNode`、読むのは `crates/pasta_dsl/src/parser/parse_scene.rs` 311 行の `parse_choice_line`。
- 生成は `crates/pasta_lua/src/code_gen/scope_gen.rs` の `generate_choice`（376 行）と `generate_choice_timeout`（395 行）。`！` 行のうち、実行時に意味を持つのは `！select` だけである。
- 実行時は `crates/pasta_lua/pasta_scripts/pasta/act.lua` の `ACT_IMPL.choice`（339 行）と `ACT_IMPL.choice_timeout`（350 行。引数は秒）。さくらスクリプトにするのは `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` 456〜458 行で、秒を 1000 倍して `\![set,choicetimeout,ミリ秒]` を出す（引数が無ければ 0）。
- 正典（ukadoc）: `\![set,choicetimeout,時間]` の単位はミリ秒で、0 か -1 でタイムアウトしない。時間切れになると `OnChoiceTimeout` が起き、Reference0 は時間切れになったさくらスクリプトである（選択肢を出したシーンは運ばない）。
- pasta の中に、`OnChoiceTimeout` の既定の受け口は無い（`pasta_scripts` の中に名前が出てこない）。
- 選ばれた時の振り分けは `crates/pasta_lua/pasta_scripts/pasta/shiori/event/choice_select.lua`（73 行）。選択肢を出したグローバルシーンのローカル → グローバルの順に探し、条件漏れは止まるエラー、名前が無ければ何も返さない（51〜70 行）。
- シーンの条件の比べ方は、Rust の 1 か所に在る（`crates/pasta_lua/src/search/scene_cond.rs`。`scene-condition-attributes` が入れた）。`scene-condition-attributes` は、選択肢の行そのものを隠さない（そちらの Out）。選択肢の飛び先の候補を選ぶときには、条件が効く。
- 飛び先の検査は `crates/pasta_check/src/validate/references.rs`（72 行で選択肢の飛び先を見る）。
- 秒で書いた所: 見本ゴースト hello-pasta の `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/09-choice.pasta`（4 行目の注記「`!select(秒)` で、選択肢の制限時間を秒で決める。」と、11 行目の `!select(10)`）、入門 9 章 `book/src/getting-started/09-choice.md`（24・31 行の写し、47 行の表「10 秒にする」、95 行の目次）、マニュアルの `！select` の記述（`book/src/grammar/block-structure.md` の「キューコマンド行」・`book/src/lua/script-api.md`・`book/src/lua/patterns.md`・`book/src/internals/talk-output.md`）。

## Desired Outcome

- 選択肢の行に、シーンの宣言と同じ形の条件を書ける。条件を満たさない選択肢は出ない。

```pasta
    ＠？手を振る「手を振ってみる」　＆＄＊スロット訪ねた数≧３
```

- `！select` の 2 つ目の引数で、時間切れの飛び先を書ける。時間はミリ秒で書く。

```pasta
    ＠？屋上へ「屋上へ行く」
    ＠？帰る「まっすぐ帰る」
    ！select（２００００、時間切れ）
```

- 今の `!select(10)` は 10 ミリ秒になる。見本ゴースト・入門・マニュアルは `!select(10000)` に直っていて、リリースノートに変更の前後の書き方が例つきで載る。
- `pasta_check validate`・LSP の色付け・VSCode の文法定義が、新しい書き方に合っている。

## Approach

決定（2026-11-19 の discovery「ノベルゲームの深掘り」と、同じ日の暫定設計の議論）。`novel-talk-flow` の brief の「決定（2026-11-19 の discovery『ノベルゲームの深掘り』。最終形）」の「文法に足す物」と「分岐は 1 つの形」から、この spec の分を写した。どれも開発者の決定なので、要件定義で変えない。骨子は `novel-areka-contract/final-model-2026-11-19.md` の 2.3・2.4 と 8 章の D1。名前は、どれも仮である。

- **選択肢の行に、シーンの宣言と同じ形の条件を付ける**（`＠？手を振る「手を振ってみる」　＆＄＊スロット訪ねた数≧３`）。
- **`！select（２００００、時間切れ）` の 2 つ目の引数で、時間切れの飛び先を書く。**
- **`！select` の時間は、ミリ秒に替える**（開発者: 「ミリ秒にそろえる。リリースにちゃんと変更を書けば『ぱすた開発』も『emo2開発』も追従してくれるはず」。骨子 2.4・8 章の D1）。今は秒で、pasta が 1000 倍している。飛び先を足すのと同じ版で替える。**破壊的な変更である。** 一緒に直す物: Lua の `act:choice_timeout`（引数をミリ秒に）・入門 9 章・見本ゴースト hello-pasta の `09-choice.pasta`（`!select(10)` → `!select(10000)`。4 行目の注記も）・マニュアルの `！select` の記述（`grammar/`・`lua/script-api.md`・`lua/patterns.md`）・リリースノート（変更の前後の書き方を、例つきで書く）。ゴーストを作っているセッション（「ぱすた開発」・「emo2 開発」）は、リリースノートを見て追従する。**小さい数を書いた時に知らせる検査は、足さない。**
- `pasta.toml` のトークの間隔（`talk_interval_min` など。秒）は、台本でなく設定なので、今のままにする（骨子 2.4）。
- **イベントは分岐である**（骨子 2.3）。クリックでも、選択肢でも、時間切れでも、イベントでシーンが選ばれたら、そのシーンへ進む。演劇の間の再生位置の動きは `novel-talk-flow` が、着いた時の柱は `scene-stage-attributes` が持つ。この spec は、飛び先のシーンを選んで走らせるところまでを持つ。
- 演劇と設定の間の「`＆再生モード` で候補を絞る」決まりは、選択肢と時間切れの飛び先には当てない（`novel-talk-flow` の「いつもの反応との書き分け」）。この spec は、走り方を知らなくてよい。

暫定設計の案（`novel-talk-flow/provisional-design.md` の R14 と「文法と生成」。決定ではない。要件定義の材料にする）:

- 選択肢の行の条件は、条件とフラグだけを読み、宣言（`＆名前：値`）は構文エラーにする。
- 時間切れの飛び先は、選択肢を出したグローバルシーンのローカル → グローバルの順に探す（選ばれた時の振り分けと同じ）。
- 条件の比べ方は、Rust の 1 か所（`scene_cond.rs`）を、Lua から呼べる口で使う。Lua に書き写さない。

## Scope

- **In**: 選択肢の行の条件（文法・構文木・パーサー・生成・実行時の絞り込み）。`！select` の時間切れの飛び先（生成・実行時・`OnChoiceTimeout` の受け口）。`！select` の時間のミリ秒化（`act:choice_timeout`・組み立て）と、見本ゴースト・入門 9 章・マニュアル・リリースノートの直し。`pasta_check validate`（条件の書き間違い・時間切れの飛び先の検査）と一致テスト。LSP の色付けと VSCode の文法定義。スキル references の再生成。
- **Out**: 走り方（`＆再生モード`）・演劇の間の再生位置・進める・出し直し（`novel-talk-flow`）。着いた時の柱（`scene-stage-attributes`）。ト書きを流すこと（`cue-command-runtime`。`！select` は、あちらが触らない）。シーンの宣言の条件そのもの（`scene-condition-attributes`。入った）。

## Boundary Candidates

- 文法とパーサー（`choice_line`・`ChoiceNode`・`parse_choice_line`）。
- 生成（`generate_choice`・`generate_choice_timeout`）。
- 実行時（`ACT_IMPL.choice`・`ACT_IMPL.choice_timeout`・組み立ての `choice_timeout`・条件を量る口・時間切れの受け口）。
- 検査とエディタ。
- 見本・入門・マニュアル・リリースノート。

## Out of Boundary

- 選択肢の出し方（改行・並べ方。`choice-line-layout` が決めた形のまま）。
- 選ばれた時の振り分けの決まり（探す順・条件漏れの扱いは今のまま）。

## Upstream / Downstream

- **Upstream**: 働きの上では無い（`scene-condition-attributes` の条件の比べ方は入った）。
- **Downstream**: `novel-talk-flow`（時間切れを、演劇の間の分岐として扱う）、`scene-anchor-link`（選択肢の振り分けを取り出すなら、共有する）、`hello-novel-sample`、`manual-novel-guide`。ゴーストを作っているセッション（「ぱすた開発」・「emo2 開発」）は、リリースノートで追従する。

## Existing Spec Touchpoints

- **Extends**: `novel-talk-flow`（切り出し元）。`.kiro/specs/completed/scene-condition-attributes/`（同じ条件の形を、選択肢の行に使う）。
- **Adjacent**: `.kiro/specs/completed/choice-line-layout/`（`sakura_builder.lua` の選択肢の出力）、`scene-anchor-link`（`grammar.pest`・`act.lua`・`choice_select.lua`・`sakura_builder.lua`）、`scene-exact-match`（`act.lua`・`grammar.pest`・VSCode の文法定義）、`cue-command-runtime`（`act.lua`・`grammar.pest`・`scope_gen.rs`）。

## Constraints

- `act.lua` は、1 つのウェーブに 1 つの spec だけが持つ。`act.lua`・`grammar.pest`・`scope_gen.rs` を共有する `cue-command-runtime` とは、同じウェーブに置かない。`novel-talk-flow` より先に置く（`act.lua`・文法・`scope_gen.rs`・`choice_select.lua` を共有する）。
- 条件を書いていない選択肢の行の生成は、今のまま変えない。
- ミリ秒化のほかに、今のゴーストの出力を変えない。
- 見本の期待値ファイルや、生成の形を固定しているテストの期待値は、同じ変更で直す。
- マニュアルが文法の唯一の権威である。挙動を変えたら、同じ変更でマニュアルを直し、スキル references を再生成する。

## 測定（2026-11-20）

- **触るファイル**: 文法 `crates/pasta_dsl/src/parser/grammar.pest`（285 行。`choice_line` 236 行）・`ast/scene.rs`（235 行）・`parse_scene.rs`（444 行）、生成 `crates/pasta_lua/src/code_gen/scope_gen.rs`（498 行。`generate_choice` 376 行・`generate_choice_timeout` 395 行）と生成のテスト、実行時 `crates/pasta_lua/pasta_scripts/pasta/act.lua`（857 行。`choice` 339 行・`choice_timeout` 350 行）・`shiori/sakura_builder.lua`（690 行。456〜458 行）・`shiori/event/choice_select.lua`（73 行）・時間切れの受け口（新しいファイルと `shiori/event/init.lua` の読み込みの 1 行の見込み）、条件を量る口 `crates/pasta_lua/src/search/context.rs`・`scene_cond.rs`、検査 `crates/pasta_check/src/validate/references.rs`・`report.rs`・`crates/pasta_check/tests/validate_parity_test.rs`、LSP `crates/pasta_lsp/src/analysis/visit_action.rs`、VSCode `editors/vscode/syntaxes/pasta.tmLanguage.json`（`#choice`）、hello-pasta の `dic/09-choice.pasta`、入門 `book/src/getting-started/09-choice.md`、マニュアル（`grammar/block-structure.md`・`lua/script-api.md`・`lua/patterns.md`・`lua/shiori-events.md`・`internals/talk-output.md`）、リリースノート（`/release` が書く `doc/release-notes/v{版}.md` の見どころ）、スキル references。テストは新しいファイルに置く。
- **規模**: 6〜8（文法と生成 2、実行時の条件 1、時間切れの飛び先と受け口 1〜2、ミリ秒化と見本・入門・マニュアル 1、検査とエディタ 1、テスト 1）。
- **先に要るもの**: 働きの上では無い。ファイルの重なり（未完了）: `cue-command-runtime`（`act.lua`・`grammar.pest`・`scope_gen.rs`・VSCode の文法定義）、`novel-talk-flow`（`act.lua`・文法・`scope_gen.rs`・`choice_select.lua`・`sakura_builder.lua`）、`scene-exact-match`（`act.lua`・`grammar.pest`・VSCode の文法定義）、`scene-anchor-link`（`grammar.pest`・`act.lua`・`sakura_builder.lua`・`choice_select.lua`・VSCode の文法定義）、hello-pasta の辞書と入門の章を触る spec。
- **フェーズ**: 1（今の SSP で動く。正典の `\![set,choicetimeout]` と `OnChoiceTimeout` だけを使う）。
- **種別**: 機能（選択肢の条件と、時間切れの飛び先が無い）。ミリ秒化は開発者の指示（2026-11-19）。
- **要件定義のモデル**: Opus（決まりは決まった。残るのは形の選び方で、推奨を付けられる）。
- **要件定義の議題**（6 つ）:
  1. 選択肢の行に書ける物（条件とフラグだけにして、宣言は構文エラーにするか）と、条件の比べ方を Rust の 1 か所から呼ぶ口の形。
  2. 時間切れの飛び先を探す範囲。`OnChoiceTimeout` は選択肢を出したシーンを運ばない（ukadoc）ので、pasta が、選択肢を出したシーンと飛び先を覚えておくか。飛び先の無い時間切れ（2 つ目の引数を書かない・書いた名前のシーンが無い）の扱い。
  3. 選択肢の振り分け（`choice_select.lua`）を、関数として取り出すのを、この spec でするか（時間切れの飛び先が同じ探し方を使う。先に入るので、取り出すならこちらで、`novel-talk-flow` と `scene-anchor-link` が使う）。
  4. `pasta_check validate` に足す物（時間切れの飛び先を、今の飛び先の検査に載せるか・条件の書き間違いを誤りにするか注意にするか）。
  5. 条件を満たす選択肢が 1 つも無い時の動き。
  6. リリースノートの文面を、どこに残して `/release` へ渡すか（見どころのファイルは `/release` の手順で書く）。
