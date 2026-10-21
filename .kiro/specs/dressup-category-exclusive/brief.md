# Brief: dressup-category-exclusive

> **ステータス**: 未着手（2026-10-21 の `/kiro-discovery`「PastaDSL機能ヒアリング」で起票。ghost_dev の「ぱすた開発」「emo2 開発」セッションからの聞き取り）。roadmap のバックログ「着せ替えの復旧の出力の膨らみ」に、作者が困った事例が出たので起票した。版は未定（開発者の判断）。要件定義は Opus。

## Problem

外見の記録が着せ替えのカテゴリを排他と知らないので、作者は同じカテゴリの全部のパーツの OFF を毎回出している。

- 窓際のぱすたさん（ghost_dev）の `scripts/pasta_dress.lua`（236 行）は、表情コードを bind の列に変えるとき、選んだパーツを ON にし、**同じ軸の他のパーツを全部 `,0` で明示 OFF にしている**。作者の説明は「pasta の外見の記録を軸ごとに 1 つに保つため。OFF を出さないと、復旧で別の腕が出る」。
- emo2 の `scripts/murasaki1000.lua` も、紅の 0/1 を毎回明示している。
- 外見として記録させたくない bind（ボールの OFF、ホバーの bind）は、`raw_script` に逃がしている（この件は `cue-command-runtime` へ申し送った）。
- 表情を DSL の単語で書けない理由の 1 つがこれである。単語の値に OFF の列まで並べると、表情 1 つが bind 6〜7 個の長い文字列になる。

## Current State

2026-10-21（main `71e18f42`）に確かめた。

- `shiori/appearance.lua` の `record_bind`（L217 付近）は、アクターの側（`ac.binds[cat]`）に、着けたことのあるパーツの ON をカテゴリごとに全部残す。復旧のたびに、それを記録の順に出し直す（roadmap のバックログ「着せ替えの復旧の出力の膨らみ」）。
- スポットの側は、ON（1）の bind の後に同じカテゴリの他のパーツを「不明」にする（`spot_after_wear`、L201-213）。排他を仮定せず、控えめに扱っている。
- pasta は `descript.txt` を読まない（`book/src/grammar/actor-dictionary.md` L227）。カテゴリが排他か（`multiple` か）、必ず 1 つ選ぶか（`mustselect`）を知らない。
- バックログの記述: 直すなら、着けたときに同じカテゴリのほかの ON を記録から外し、`multiple` のカテゴリだけを宣言で外す。

## Desired Outcome

- 着せ替えのカテゴリは、既定で排他として記録する。パーツを ON にすると、同じカテゴリのほかのパーツの ON は記録から外れる。作者は OFF の列を出さなくてよい。
- 複数のパーツを同時に着けられるカテゴリ（`multiple`）を、作者が宣言できる（`pasta.toml` か、アクター辞書か、`descript.txt` を読むかは要件で決める）。
- 復旧の出力が、カテゴリごとに 1 つ（`multiple` は着けている分）に縮む。
- マニュアル `grammar/actor-dictionary.md` の外見の復旧の節が、排他の扱いと宣言の書き方を書く。

## Approach

要件フェーズで次を決める。

- **`multiple` の知り方**: (a) `pasta.toml` の宣言、(b) シェルの `descript.txt` の `bindgroup` の `multiple` を読む、(c) 両方。シェルが切り替わったときの読み直し。
- **既定を変える影響**: 今の辞書で、排他でないカテゴリを宣言なしで使っている作者の見た目が変わる。破壊的変更の扱いと、マニュアルの書き方。
- **スポットの側の「不明」**: 排他が分かったカテゴリでは、不明にせず確定してよいか。

## Scope

- **In**: 外見の記録と復旧のカテゴリの扱い、`multiple` の宣言、テスト、マニュアル、スキルの `references/` の再生成、`pasta_check validate` の宣言の検査（要件で決める）
- **Out**: 表情コード（軸の表）から bind の列を作る仕組み（ゴースト固有として Lua に残す。両ゴーストの作者も「この仕組みが無いなら Lua のままでよい」と答えた）、`raw_script` 相当の DSL の書き方（`cue-command-runtime`）

## Boundary Candidates

- `shiori/appearance.lua` の記録と復旧
- `multiple` の宣言の読み込み（設定か `descript.txt`）
- マニュアル・生成スキル

## Out of Boundary

- サーフェスの記録（`\s[ID]`）の扱い
- 着せ替えメニュー（SSP の UI）の操作の追従

## Upstream / Downstream

- **Upstream**: なし
- **Downstream**: ゴーストの表情の Lua（`pasta_dress.lua`・`murasaki1000.lua`）から OFF の列が消える。表情を単語で書く道が近づく。

## Existing Spec Touchpoints

- **Extends**: なし（バックログからの起票）
- **Adjacent**: `cue-command-runtime`（`！命令` で出す `\![bind,…]` を外見に記録するかを決める。記録の規則はこの spec が持つ）

## Constraints

- `\![bind-noevent,…]` を使う今の復旧の出し方（SSP 2.8.23 以降）は変えない。
- 規模は 5〜7 タスクの見込み。
