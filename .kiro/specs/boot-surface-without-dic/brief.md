# Brief: boot-surface-without-dic

`getting-started-story-guide` の完了時の棚卸で起票（2026-10-10・main `f193c9cb`）。

## Problem

初めてゴーストを作る人は、入門ガイドの準備の章で最小一式を置いて SSP で起動する。辞書（`.pasta`）がまだ 1 枚も無いので、立ち絵も吹き出しも出ない。画面には何も現れず、うまくいったのか失敗したのかを見た目で区別できない。ガイドは「何も表示されないのが正しい」と教え、ログのファイルで確かめさせている。最初の成功体験が「何も起きない」になっている。

## Current State

- 実機の確認（2026-10-10・SSP・v0.3.8 の `pasta.dll`）: 辞書が無いゴーストは、pasta の読み込みには成功する（`profile/pasta/logs/pasta.log` ができる）。`OnBoot` はスクリプトを返さず、サーフェス番号は 2 人とも -1 のまま。立ち絵は、最初の台詞と一緒に出る。タスクバーに SSP のアイコンは出る。
- `OnBoot` の既定の処理は `crates/pasta_lua/pasta_scripts/pasta/shiori/event/boot.lua` にあり、同名のシーンを探すだけである。
- 入門ガイドの `book/src/getting-started/setup.md`「SSP に入れて起動する」は、成功の目印を「何も表示されず、何もしゃべらない」とし、立ち絵が無い間はタスクバーの SSP のアイコンからメニューを開くと案内している。
- 辞書はあるが `＊OnBoot` が無いゴーストでも、同じことが起きる。

## Desired Outcome

決めていない。やる場合は、辞書や `＊OnBoot` が無くても、起動した時点で立ち絵が出る。やらない場合は、その判断と理由が記録される。

## Approach

決めていない。案は、`OnBoot`（と `OnGhostChanged`）の既定の処理が、応えるシーンが無いときに立ち絵だけを出すスクリプトを返すこと。どのサーフェスを出すかは、`pasta.toml` の `[actor]` からは決まらない（表情とサーフェス番号の対応は辞書が持つ）ので、そこが論点になる。

## Scope

- **In**: やるかどうかの判断。やる場合は、応えるシーンが無い起動のときの既定の出力と、入門ガイドの準備の章・うまく起動しないときの節の更新。
- **Out**: シェルの絵（`hello-pasta-shell-art`）。辞書があるときの起動の挨拶。SSP の側の表示の決まり。

## Boundary Candidates

- 起動のイベントの既定の処理（`boot.lua` ほか）。
- 入門ガイドの準備の章の記述。

## Out of Boundary

- 実行時の失敗をバルーンへ出すこと（`failure-output-unification`・`runtime-error-balloon`）。
- `pasta_check` での辞書の検査（`pasta-check-dic-validate`）。

## Upstream / Downstream

- **Upstream**: なし。
- **Downstream**: `getting-started-screenshots`（準備の章に載せられる絵が変わる）。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**: `hello-pasta-shell-art`（サーフェス番号）、`getting-started-screenshots`。完了した `getting-started-story-guide`（準備の章の終わりを「何も表示されない」に直した経緯。`.kiro/specs/completed/getting-started-story-guide/tasks.md` の Implementation Notes（5.3））。

## Constraints

- 辞書を書いた人が出したくない絵を、pasta が勝手に出さない（意図を推測して救済しない）。
- 既に配布されているゴーストの起動の見え方を変えない。

## 2026-10-10 起票時の測定（main f193c9cb）

- **規模**: やる場合で 3〜5 タスク。
- **種別**: 機能（やるかどうかから）。急ぐバグではない。
- **要件定義のモデル**: Fable（やるかどうかを最初に決める）。
- **要件定義の議題**:
  1. やるか。「辞書が無ければ何も出ないのは正しい」とも言える。入門の最初の成功体験のためだけに既定の動きを足す価値があるか。
  2. やるなら、どのサーフェスを出すか（`0`・`10` の決め打ち、`pasta.toml` に書かせる、シェルの既定に任せる）。
  3. 辞書はあるが `＊OnBoot` が無いときも同じに扱うか。
  4. 入門ガイドの準備の章の成功の目印と、タスクバーからの操作の案内を直すか。
