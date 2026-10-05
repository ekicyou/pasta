# 統合コミットと PR の告知文面（要件 11.1）

`kiro-complete` は、統合コミットの件名と PR 本文の「破壊的変更」の節に、次の文面をそのまま使う。

## 件名

```
fix(paragraph-break-tag-only-talk)!: タグだけの出力で段落区切りの改行を出さず、タグの読み取りを SSP にそろえる（破壊的変更）
```

## PR 本文の「破壊的変更」の節

```markdown
## 破壊的変更

pasta を更新すると、次の書き方で出力やパース結果が変わる。新しい規則はマニュアルの各章に書いてある。

- 単語の値で、タグの直後に字を続けた書き方（`＠x：\nOK`・`＠x：\w10` など）がパースエラーになる。（[単語定義](https://ekicyou.github.io/pasta/grammar/words.html)・[さくらスクリプト](https://ekicyou.github.io/pasta/grammar/sakura-script.html)）
- タグの直後に ASCII の字や `!`・`?` が続く台詞（`\nHello`・`\w9OK`・`\n!?` など）で、その字が字として読まれ、ウェイトの挿入・BudouX の改行・段落区切りの判定が働く（出力が変わる）。アクション行では、その字が別の `talk` に分かれる。（[さくらスクリプト](https://ekicyou.github.io/pasta/grammar/sakura-script.html)・[@pasta_sakura_script](https://ekicyou.github.io/pasta/lua/modules/pasta-sakura-script.html)）
- `\]` や引用の中の `]` を含む引数が途中で割れなくなる（`\s[a\]b]` は 1 つのタグになる）。引用として読むのは引数の先頭の `"` から始まるものだけになる（アクション行では、変更前は引数の途中の `"…"` も引用として読み、その中の `]` で閉じなかった。変更後は途中の `"` は通常の文字で、最初の `]` で閉じる）。引数の先頭で開いた引用が閉じないとき、その引数は閉じていないものとして読まれる。（[さくらスクリプト](https://ekicyou.github.io/pasta/grammar/sakura-script.html)）
- アクション行で、閉じない `[` や閉じない引用が後ろの行まで読まれなくなる（その行の中で閉じなければ、名前までがタグになる）。（[さくらスクリプト](https://ekicyou.github.io/pasta/grammar/sakura-script.html)）
- `\sN`・`\pN`・`\bN`・`\wN` の直後の `[…]` は引数でなく字として読まれる。`\_`・`\__` だけの並びはタグでなくなる（アクション行ではパースエラー）。（[さくらスクリプト](https://ekicyou.github.io/pasta/grammar/sakura-script.html)）
- `\%` をアクション行に書けるようになる（パースエラーでなくなる）。台詞の中の `\%` の間にウェイトが入らなくなる。（[アクション行](https://ekicyou.github.io/pasta/grammar/action-line.html)・[さくらスクリプト](https://ekicyou.github.io/pasta/grammar/sakura-script.html)）
- 閉じのある囲み（`\_?…\_?`・`\_!…\_!`）の中身に、ウェイトも改行も入らなくなり、外見の観測もされなくなる。アクション行では、中の `＠単語`・`＄変数` が展開されなくなる。（[さくらスクリプト](https://ekicyou.github.io/pasta/grammar/sakura-script.html)・[@pasta_sakura_script](https://ekicyou.github.io/pasta/lua/modules/pasta-sakura-script.html)・[スクリプト API](https://ekicyou.github.io/pasta/lua/script-api.html)）
- タグだけの出力（`\q[…]` だけの `talk` を含む）が台詞に数えられなくなり、段落区切りの改行の出方が変わる。（[pasta.toml リファレンス（spot_newlines）](https://ekicyou.github.io/pasta/reference/pasta-toml.html)）
```

プッシュ・PR 作成・マージは `kiro-complete` が行う（本タスクでは行わない）。
