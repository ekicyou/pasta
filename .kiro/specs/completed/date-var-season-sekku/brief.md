# Brief: date-var-season-sekku

## Problem

ghost_dev の「窓際のぱすたさん」で季節トークを設計している作者（pasta の開発者と同じ人。2026-10-31）が、季節別のシーンへ Lua なしで振り分けたい。今は月を季節に直すのに、複数キーの単語（`＠9月の季節、10月の季節、11月の季節：秋`）と動的単語参照（`＄季節＝＠＄季節キー`）を辞書に書く必要がある。作者の言葉は「transfer_date_to_var に『＄季節、＄節句』が欲しいですねえ。春夏秋冬くらいは知りたいかも」。方針は「レベルによる情報開示などを Lua を使わずに実現したい」（2026-10-29）。

## Current State

`＞transfer_date_to_var`（`crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua`。時報シーンでは自動で呼ばれる）が入れる日時変数は、`＄年 ＄月 ＄日 ＄時 ＄分 ＄秒 ＄時１２ ＄曜日` と、英語名の数値版（`＄year`〜`＄wday`・`＄week`）だけ。季節と節句は無い。

## Desired Outcome

```pasta
＊OnMouseDoubleClick
　＞transfer_date_to_var
　＞「季節_」＆＄季節
　＞「節句_」＆＄節句

＊季節_秋_衣替え
　女の子：＠通常　そろそろ衣替えの季節だね。

＊節句_七夕
　女の子：＠通常　今日は七夕。お願いごと、した？

＊節句_なし
```

## Approach（2026-10-31 開発者の承認）

- `＄季節`：気象の区分。3〜5 月 `春`、6〜8 月 `夏`、9〜11 月 `秋`、12〜2 月 `冬`。暦（立春など）は年で日がずれ、表か天文計算が要るので採らない。暦で分けたいゴーストは `＄月`・`＄日` で書ける。
- `＄節句`：五節句のその日だけ `人日`（1/7）・`上巳`（3/3）・`端午`（5/5）・`七夕`（7/7）・`重陽`（9/9）。ほかの日は `なし`。空文字列にしないのは、`＞「節句_」＆＄節句` が `節句_` の前方一致になり、平日にも節句のシーンが選ばれてしまうため。期間は持たない（`＄月`・`＄日` で書ける）。
- 英語名・数値の版（`＄season` など）は足さない。Lua で分けるなら `var.month` で足りる。
- 今の日時変数と同じ日付（`CLOCK.date`）から作るので、SSP の仮想時刻と `X-Pasta-Time` に従う。テストで確かめる。
- 名前の衝突は、マニュアルの注意文に `＄季節`・`＄節句` を足すだけにする。転記はローカル変数の上書きで、今の `＄日`・`＄時` と同じ扱い。

## Scope

- **In**: `act.lua` の `transfer_date_to_var`、Lua のテスト（`shiori_act_test.lua`）、マニュアル `book/src/grammar/variables.md`（日時変数の表・注意文・例）、スキルの `references/variables.md`（`book/tools/gen-skill-refs.mjs` で生成）。
- **Out**: 二十四節気・雑節・祝日、節句の期間、英語名の版、季節の区切りの設定。

## Boundary Candidates

- 日時変数の転記（`act.lua`）だけ。

## Out of Boundary

- イベントのシーンを選ぶ前の自動転記（`scene-condition-attributes` が持つ）。

## Upstream / Downstream

- **Upstream**: `transfer_date_to_var`・`pasta.shiori.clock`（仮想時刻）。
- **Downstream**: 窓際のぱすたさんの季節トーク。`scene-condition-attributes` の条件（`＆＄季節＝「秋」`）。

## Existing Spec Touchpoints

- **Extends**: 完了 spec `onhour-date-var-transfer`・`tutorial-ssp-gaps`（仮想の日時）。
- **Adjacent**: `scene-condition-attributes`（自動転記で `＄季節`・`＄節句` もイベントのシーン全部に入るようになる。同 brief へ申し送り済み）。

## Constraints

- 緊急（2026-10-31 開発者「ロードマップは緊急で、次のマイナーバージョンを差し込む感じで」「ホントに緊急なので、このセッションで実装完了＆リリースまで進めます」）。0.4.4 として差し込み、ponytail を 0.4.5 へ繰り下げる。
- 規模が小さいので、要件・設計・タスクの文書を作らず、この brief を正本として 1 セッションで実装・リリースする（開発者の指示）。
