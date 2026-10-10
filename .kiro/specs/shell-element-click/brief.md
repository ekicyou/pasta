# Brief: shell-element-click

`/kiro-discovery` で起票（2026-10-10・main `88c4bc0e`）。0.5.0「areka でノベルゲームが作れる」の道筋の 7 番目。

## Problem

ノベルゲームのタイトル・セーブ・ロード・設定の画面は、シェルとして組み、pasta の場面が動かす（2026-10-10 の決定）。areka が出すのは部品だけである。その部品の 1 つが、エレメント単位のクリックである。クリックできるエレメントは、絵で出来た選択肢と言える。ところが、pasta には、名前つきのクリックを場面へ届ける道が無い。

```
サーフェス「セーブ画面」
├─ 背景の絵
├─ 枠 1（エレメント。クリックできる。名前は slot1）
│   ├─ 縮小画像（別のサーフェスを階層的に呼ぶ）
│   └─ バルーン（日付と章の名前を書く所）
├─ 枠 2 … 枠 6
└─ 戻るボタン（エレメント。名前は back）
```

## Current State

- 選択肢は、選ばれると `OnChoiceSelectEx` で届く。飛び先を、選択肢を出したグローバルシーンのローカルシーンから先に探し、無ければグローバルシーンを探す（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/choice_select.lua`）。
- 触られたときの反応（`OnMouseDoubleClick` など）は、当たり判定の名前を Reference で受け取り、作者が場面の中で振り分ける（入門ガイドの 8 段目）。

### 2026-10-10 の検証で見つかった穴

- **選択肢の振り分けは、そのままでは使い回せない**: 処理は `REG.OnChoiceSelectEx` の中に直に書いてあり、Reference の番号が決め打ちである（`choice_select.lua` 16〜18・41〜78 行）。場面のコルーチンを作る関数も、そのファイルの中でしか使えない（25〜35 行）。
- **前方一致**: 場面の検索は前方一致である。`slot1` は `slot10` にも当たる。完全一致の場面選びは `novel-talk-flow` が入れる。
- **イベントの引数**: Reference は、場面の引数にならない（`shiori/event/init.lua` 226 行）。
- **NOTIFY には台本を返せない**: クリックの通知は GET で送ってもらう必要がある。
- areka は、今は `OnMouseClick` を SHIORI へ送っていない（areka に brief がある: `areka-P0-mouse-click-wheel-events`）。

## Desired Outcome

- エレメントがクリックされると、その名前の場面が動く。飛び先の探し方は、選択肢と同じである。
- 作者は、場面の中で Reference を読んで振り分ける処理を書かなくてよい。

```pasta
＊タイトル
    ％タイトル画面

    ・はじめから
        ＞第一章
    ・つづきから
        ＞ロード画面
```

- シェルの中のバルーンへ、場面から文を書ける。

## Approach

要件定義で決めること。

- **イベントの形**: 名前（エレメントの名前）と、どの画面の上で起きたか（選択肢の 3 番目の引数に当たるもの）を、どう受け取るか。名前は `novel-areka-contract` が決める。
- **振り分けの共有**: 選択肢の振り分けを、関数として取り出して使い回す。文中のシーンリンク（`scene-anchor-link`）も同じ振り分けを使う予定なので、取り出す形を揃える。
- **「どの画面の場面か」の決め方**: 選択肢は、台本に埋めたグローバルシーンの名前で決まる。エレメントのクリックには、それが無い。画面を出した場面を覚えておくか、areka に名前を運んでもらうか。
- **バルーンへ書く**: シェルの中のバルーンは、アクター（スコープ）として書けば足りるかを確かめる。足りなければ、名前を指定して書く口を足す。
- **飛び先が無いとき**: 選択肢と同じく 204 を返す。失敗の見せ方は `failure-output-unification` に乗せる。
- **触られたときの反応との関係**: 今の `OnMouseDoubleClick` の受け方を変えない。

## Scope

- **In**: エレメントのクリックのイベントを、場面へ振り分けること。選択肢の振り分けの取り出し。シェルの中のバルーンへ書く口（要るなら）。テスト、マニュアル、スキル references の再生成。
- **Out**: 画面の見た目と配置（シェル）。エレメントのクリックの検出と通知（areka）。セーブとロード（`novel-save-slots`）。

## Boundary Candidates

- イベントの入口（`shiori/event/` の新しいハンドラ）。
- 振り分けの共通部品（`choice_select.lua` から取り出す）。
- バルーンへ書く口。

## Out of Boundary

- 当たり判定の定義（シェルの `surfaces.txt`）。
- 画面を出す・消すの命令（語彙の話で、`cue-command-runtime` がそのまま流す）。

## Upstream / Downstream

- **Upstream**: `novel-areka-contract`（イベントの名前と Reference の決まり）、`novel-talk-flow`（完全一致の場面選び）。
- **Downstream**: `novel-save-slots`（セーブ画面）、`hello-novel-sample`（タイトル・セーブ・ロード・設定の画面）。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**: `scene-anchor-link`（選択肢の振り分けを共有する。`choice_select.lua` を触る順番に注意）、`.kiro/specs/completed/choice-definition-dsl/`（選択肢の振り分けの元の設計）。

## areka への依頼

- エレメント単位のクリックを、エレメントの名前つきで、GET のイベントとして知らせる。
- シェルを、もう 1 枚上に重ねて出す・消す（システム画面）。
- 重ねた画面が出ている間、下の画面へのクリック送りを止める。

要件が固まったら、取り決めの文書へ書き、areka のセッションへ知らせる。手順は `novel-areka-contract` の brief の「依頼の出し方」。

## Constraints

- 選択肢の挙動を変えない。
- `choice_select.lua` は、`scene-anchor-link` と同じウェーブに置かない。
- 小さい spec である（5〜8 タスクの見当）。規模が膨らむなら、バルーンへ書く口を分ける。
