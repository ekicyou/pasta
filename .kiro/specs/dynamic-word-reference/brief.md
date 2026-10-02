# Brief: dynamic-word-reference

> **ステータス**: 未着手（`manual-ssot-authority` で起票・2026-10-01）。優先度は低い（旧仕様の言葉で「忘れていなければいつか実装する」レベル）。着手するときは `/kiro-start dynamic-word-reference` で開始する。

## Problem

旧文法仕様（`doc/spec/` ch02・ch10・ch12 §12.7）は、変数の値を単語名として参照する動的単語参照 `＠＄変数名` を「文法定義済み・実装は将来予定」として定義していたが、パーサに規則が無く、書くとパースエラーになる。

`manual-ssot-authority` で旧文法仕様を廃止し、マニュアルを文法の唯一の権威とした。マニュアル `grammar/words.md` の「動的単語参照（将来変更あり）」節は未実装機能の予告のため削除した。本 brief はその内容を将来仕様として引き継ぐ（吸収台帳の仕分け表 B2）。

## Current State

実装照合は `manual-ssot-authority` の吸収台帳（`.kiro/specs/completed/manual-ssot-authority/absorption-ledger.md`「将来仕様の仕分け表」ch12 L31 の B2 行）による。

- `grammar.pest` の単語参照は `word_ref = { word_marker ~ id ~ s }` で、`＠＄` を受ける規則が無い。
- 実測: `さくら：[＠＄x]`・`＄y＝＠＄x` はパースエラー（`expected id`）。
- 旧 §12.7 の「未実装期間は無視または警告ログで通知」の方針も実装されていない（現行はパースエラーで辞書のロードが止まる）。
- 単語参照 `＠名前` は Lua の `act:word("名前")`（アクター付きの行では `act.アクター:word("名前")`）に変換される（マニュアル `grammar/words.md`・`grammar/variables.md`）。動的な名前で単語を引くことは、Lua 側で `act:word(…)` に変数の値を渡せば現行でも書ける見込み（代替手段として成立するかは要件フェーズで確かめる）。

## 吸収元の内容（旧文法仕様・マニュアル旧版の記録）

旧文法仕様は `manual-ssot-authority` で削除される。要件化の材料として、動的単語参照に関する記述をここに残す（要旨。原文は git 履歴の `doc/spec/02-markers.md`・`10-words.md`・`12-future.md`、`book/src/grammar/words.md` の 2.3 以前の版）。

- **ch02 マーカー表**: 「単語参照（動的）｜`＠＄var_name`｜変数値を単語名として間接参照」。
- **ch10 動的単語参照**: `＠＄var_name` 形式で、変数値を単語名として間接参照できる。`＄var_name` で取得した値を単語名として `＠` の単語検索を実行する。実装状態は「文法定義済み、実装は将来予定」。
- **ch10 制限（v1）**: 多段階参照（`＠＠word`・`＠＠＠word` など）は非対応。`＠＠` はリテラルの「＠」1 文字を埋め込むエスケープとしてだけ使う。多段解決が必要な場合は関数呼び出し（例: `＠resolve（word：…）`）で代替する。
- **ch12 §12.7 実装スケジュール**: 文法予約のみ。現仕様の完全動作確認が完了するまで着手しない。優先度は低い。未実装期間の扱いは、無視または警告ログで通知する方針を設計で検討する。
- **マニュアル旧版 `grammar/words.md`「動的単語参照（将来変更あり）」**: `＠＄var_name` 形式で変数値を単語名として間接参照する構文が定義されている。`＄var_name` で得た値を単語名として検索する。「文法定義済みだが、実装は将来予定」と注記していた。

## Desired Outcome

- `＠＄変数名` が文法として受理され、変数の値を単語名として単語検索（静的な `＠名前` と同じスコープ解決）を行う。
- 変数が未代入・空・該当する単語が無いときの挙動が決まり、静的な単語参照の扱い（空文字と警告）と揃っている。
- マニュアル（`grammar/words.md`・`grammar/markers.md` ほか）が新しい構文を書き、スキル `references/` は `gen-skill-refs.mjs` で再生成されている。
- 実装しないと決めた場合は、その判断と代替手段（Lua の `act:word`）をマニュアルに書き、本 brief を閉じる。

## Approach

要件フェーズで次を決める（未決定）。

- 実装するか、Lua の代替で足りるとして見送るか。
- 構文の範囲: アクション行・変数代入の右辺（`＄y＝＠＄x`）・アクター付きの行（アクター単語の A1 からの探索）での扱い。変数のスコープ修飾子（`＄＊`・`＄％` など）との組み合わせ。
- `＠＄` と既存のエスケープ（`＠＠`・`＄＄`）、関数呼び出し `＠名前（）`・`＠＊名前（）` との字句上の区別。
- 多段参照を非対応のままにするか（旧 ch10 制限 v1）。

実装の足がかり: トランスパイラが `act:word(var.x)` のように変数の値を `word` に渡す Lua を出せば、ランタイムの単語検索はそのまま使える。

## Scope

- **In**:
  - `＠＄変数名` の文法・パーサ・トランスパイラ対応
  - 未代入・該当なしの挙動
  - マニュアル章の更新とスキル `references/` の再生成
  - VSCode 拡張の TextMate 文法（ハイライト）の追従
- **Out**:
  - 単語検索アルゴリズム自体の変更
  - 属性によるフィルター（`scene-attribute-semantics`）
  - 多段階参照

## Boundary Candidates

- 文法（`grammar.pest`）とパーサ（`pasta_dsl`）
- トランスパイラ（`pasta_lua` code_gen）
- マニュアル・生成スキル・ハイライト文法

## Out of Boundary

- マニュアル権威化とスキル生成の仕組み（`manual-ssot-authority` が提供）

## Upstream / Downstream

- **Upstream**: `manual-ssot-authority`（旧仕様の廃止と本 brief の起票）
- **Downstream**: なし

## Existing Spec Touchpoints

- **Adjacent**: `manual-ssot-authority`（吸収台帳の仕分け表 B2）、`property-dsl-extension`（`＄％` スコープ修飾子。変数参照の字句と競合しないこと）、`pasta-manual-syntax-highlight`

## Constraints

- マニュアルが文法の唯一の権威。挙動を変えたらマニュアルを同じ変更で更新し、`node book/tools/gen-skill-refs.mjs` で生成スキルを再生成する（`--check` が CI で鮮度を見る）。
- 現行で `＠＄` はパースエラーのため、受理しても既存の辞書の挙動は変わらない。
- 現行実装を正として設計する。旧仕様の記述は材料であり規範ではない。
