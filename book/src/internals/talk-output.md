# トーク出力とアピアランス

ごきげんよう。ゴーストの台詞が、ウェイトや改行の効いたさくらスクリプトに仕立て上がるまで――その仕立ての工程をご覧に入れますわ。
組立、後処理、そして表情と着せ替え。舞台衣装の裏側まで、わたくしがお見せいたします。さあ、参りましょう。

---

この章では、ACT のトークからさくらスクリプトを組み立てる出力処理と、アピアランスの制御を扱う。

## 目的と責務

トーク出力は、ACT に積まれたトークをさくらスクリプトへ組み立て（`sakura_builder`・グループ化トークン）、さくらスクリプトの後処理（トークナイザ・ウェイト挿入・budoux 改行）を施す。アピアランス（サーフェス・着せ替え復旧・スポット）もこの章の責務である。

## 構成要素

さくらスクリプトを組み立てる `sakura_builder`、Rust 側のさくらスクリプト後処理（トークナイザ・ウェイト挿入・budoux 改行）、サーフェスと着せ替えとスポットを扱うアピアランス、SHIORI 用の ACT から成る。

## 処理とデータの流れ

シーンの実行中に ACT へトークが積まれ、応答を返す時点で `sakura_builder` がさくらスクリプトに組み立てる。組立の過程で、トークのテキストにはトークンごとに `@pasta_sakura_script` の後処理（ウェイト挿入・改行）が適用され、話者の切り替え時にはアピアランスの状態に応じた復旧が出力される。

## 境界の受け渡し

Lua 側の ACT と `sakura_builder` がトークの組立を所有し、Rust 側のさくらスクリプト処理が後処理を所有する。後処理は `@pasta_sakura_script` を通じて Lua から呼ばれる。

## 不変条件と制約

後処理は、トークナイザでさくらスクリプトのタグとテキストを分けたうえで、ウェイトと改行を挿入する。

## ソースの所在

- `crates/pasta_lua/src/sakura_script/`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/appearance.lua`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua`

## 経緯

- [sakura-builder-string-buffer](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/sakura-builder-string-buffer) — さくらスクリプト組立の文字列バッファ
- [sakura-script-wait](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/sakura-script-wait) — ウェイト挿入
- [budoux-line-breaker](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/budoux-line-breaker) — budoux 改行
- [actor-surface-restore](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/actor-surface-restore) — 着せ替え復旧

---

台詞ひとつにも、これだけの仕立てが施されておりますのよ。フンッ、見直しまして？
次は、開発者の強い味方、デバッグ基盤へ参りましょう！
