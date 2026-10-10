# Brief: hello-novel-sample

`/kiro-discovery` で起票（2026-10-10・main `88c4bc0e`）。0.5.0「areka でノベルゲームが作れる」の道筋の 8 番目。

## Problem

areka が出すのは部品だけで、ノベルゲームのシステム画面（タイトル・セーブ・ロード・設定）は、シェルとして組み、pasta の場面が動かす（2026-10-10 の決定）。出来合いの画面が無いので、作者は、物語を書く前に、これらの画面を一から組むことになる。TyranoScript がテンプレートを配っているのと同じく、写して使える見本が要る。また、0.5.0 の合格（市販品の体裁のノベルゲームが作れる）を確かめる題材も要る。

## Current State

- 見本のゴーストは hello-pasta（`crates/pasta_sample_ghost`）だけである。デスクトップマスコットで、ノベルの見本は無い。
- 配布物を作る道具（`pasta_check release`）は、ゴーストの `.nar` を作る。ノベルを 1 本の配布物としてどう配るかは、決まっていない（`novel-areka-contract` の議題）。
- hello-pasta の絵は、fal.ai の画像生成で作り直している（`hello-pasta-shell-art`）。

## Desired Outcome

- 短いノベルが 1 本ある。最初から最後まで遊べて、0.5.0 の合格ラインの機能を全部使っている。
  - 背景・登場人物・曲・効果音・場面転換の演出
  - 選択肢と、フラグによる分岐
  - クリック送り、セーブとロード、バックログと巻き戻し、既読スキップ、オート、設定、タイトル画面
- システム画面の場面一式と、そのシェルが、物語と分けて置いてある。作者は、それを写して、絵と文を差し替えるだけで自分のノベルを始められる。
- areka の実機で動くことを確かめてある。
- 見本が壊れていないことを、テストが確かめる。

## Approach

要件定義で決めること。

- **置き場所**: `crates/pasta_sample_ghost` に足すか、新しいクレートにするか。hello-pasta と名前がぶつからないようにする。
- **物語の中身と長さ**: 合格ラインの機能を全部通る、最短の筋。分岐は 2 本以上。
- **システム画面の一式**: タイトル・セーブ・ロード・設定。バックログの一覧は areka が持つので、開く命令だけを書く。
- **Lua を使う範囲**: 物語の辞書には Lua を出さない。システム画面の場面に Lua が要るなら、それは DSL の穴である。穴を埋める別の spec を起こすか、見本の中に閉じ込めるかを決める。
- **絵と音の素材**: 作り方（fal.ai）と、出どころ・ライセンスの記録。音の素材の入手先。
- **実機の確認**: areka の、どの版から確かめられるか。areka の側の進み具合に合わせて、確かめられる機能から順に確かめる。
- **配布**: `pasta_check release` で作れるか。作れないなら、何を足すか。

## Scope

- **In**: 見本のノベルの辞書（物語とシステム画面の場面）。シェル（背景・登場人物・トークの場所・システム画面）と音の素材。起動と分岐を確かめるテスト。素材の出どころの記録。areka の実機での確認。
- **Out**: 入門の文章（`manual-novel-guide`）。エンジンの機能（先行する spec）。areka の側の実装。

## Boundary Candidates

- 物語の辞書。
- システム画面の場面一式（写して使う部分）。
- シェルと音の素材。
- テストと配布。

## Out of Boundary

- hello-pasta（デスクトップマスコットの見本）を変えること。
- CG 鑑賞・回想・音楽室。

## Upstream / Downstream

- **Upstream**: `cue-command-runtime`、`scene-stage-attributes`、`novel-talk-flow`、`novel-checkpoint-rollback`、`novel-save-slots`、`shell-element-click`、`call-attribute-filter`（フラグによる分岐）、`novel-areka-contract`（語彙）。areka の側の実装（画面・音・入力）。
- **Downstream**: `manual-novel-guide`（この見本を題材にする）。0.5.0 のリリース。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**: `hello-pasta-shell-art`（絵の作り方と、素材の扱い）、`pasta-check-dic-validate`（辞書の検査を、この見本にも掛ける）、`.kiro/specs/completed/hello-pasta-tutorial-stages/`（段階ごとに起動できる見本の作り方）。

## areka への依頼

この見本を作る途中で、取り決めに足りない物が見つかる見込みである。見つけたら、取り決めの文書を直し、areka のセッションへ知らせる。手順は `novel-areka-contract` の brief の「依頼の出し方」。実機の確認は、areka のセッションと一緒に進める。

## Constraints

- areka の側の実装が無い機能は、実機で確かめられない。この spec の完了は、areka の進み具合に依る。pasta だけで確かめられる所（台本の中身・分岐・セーブの読み書き）は、テストで先に固める。
- ゴースト作りのコツ（表情・掛け合い）を推測で書かない。
- `Cargo.lock` とルートの `Cargo.toml` を触るなら、同じウェーブのほかの spec と席を分ける。
- 規模は 15〜18 タスクの見当で、上限に近い。超えるなら、物語とシステム画面で分ける。
