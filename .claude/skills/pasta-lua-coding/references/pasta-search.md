<!-- GENERATED FROM PASTA MANUAL - DO NOT EDIT -->
<!-- このファイルは pasta マニュアル「@pasta_search」（https://ekicyou.github.io/pasta/lua/modules/pasta-search.html）から自動生成されたものです。手で編集しないでください。修正はマニュアルの該当章で行い、pasta リポジトリで再生成してください。 -->

# @pasta_search

`@pasta_search` は、シーンと単語を前方一致で検索するモジュールである。ランタイムが持つ検索コンテキスト（シーン表と単語表）をユーザーデータとして公開し、メソッドを `:` で呼ぶ。

```lua
local SEARCH = require "@pasta_search"
```

DSL の単語参照（`act:word(name)`）、Call（`＞`）のシーン解決、ランダムトーク（OnTalk）のシーン選択などは、内部でこのモジュールを使う。シーン関数の中では、これらを経由するのが普通で、`@pasta_search` を直接呼ぶのは、検索結果を Lua で調べたい場合とテストで選択を固定したい場合である。

## 利用できる時期

検索対象は、シーン辞書の読み込みの最後に `require("pasta").finalize_scene()` が実行された時点で確定する。`finalize_scene()` はランタイムが自動で呼ぶため、シーン関数・イベントハンドラの実行時には検索できる。

`finalize_scene()` より前（`main.lua` や、シーン辞書の読み込み中に実行される Lua ブロックのトップレベル）でも `require` と呼び出しはエラーにならないが、検索対象が確定していないため、結果は使えない。

## 検索の範囲

どのメソッドも、第 2 引数 `global_scene_name` の有無で検索の範囲が決まる。

| 第 2 引数 | 検索する範囲 |
| ---- | ---- |
| 省略（`nil`） | グローバルシーン（`search_scene`）・グローバル単語（`search_word`）だけ |
| グローバルシーン名を指定 | 指定したグローバルシーンのローカルシーン・ローカル単語だけ |

第 2 引数を指定した検索は、ローカルに候補が無くてもグローバルへフォールバックしない（`nil` を返す）。DSL の単語参照や Call で見られる「ローカルを探し、無ければグローバルを探す」順序は、このモジュールではなく act 側の検索手順が、2 回の呼び出し（第 2 引数ありと、なし）で実現している（[単語のスコープと優先順位](https://ekicyou.github.io/pasta/grammar/words.html#スコープと優先順位)、[Call のスコープ解決アルゴリズム](https://ekicyou.github.io/pasta/grammar/call-jump.html#スコープ解決アルゴリズム)）。

## 候補の選び方

前方一致した候補が複数あるとき、DSL の単語参照と同じシャッフル＆順次消費で 1 つを選ぶ（[シャッフル＆順次消費](https://ekicyou.github.io/pasta/grammar/words.html#シャッフル順次消費)）。

- 候補を一巡するまで、同じ候補は選ばれない。一巡すると並べ直して最初から使う。
- 並び順の記録は、検索の範囲（第 2 引数）と検索に使った名前の組ごとに持つ。照合用の名前に揃える引数（`search_scene` の第 1 引数、`search_word` の第 2 引数）は、揃えた後の名前で組を作る。グローバル単語に解決される DSL の `＠挨拶`（Lua の `act:word("挨拶")`）と `SEARCH:search_word("挨拶")` は、同じ記録を進める。
- 記録はゴーストの実行中、イベントをまたいで保たれ、辞書の再読込で消える。

## メソッド

### search_scene(name, global_scene_name?)

シーンを前方一致で検索する。

```lua
SEARCH:search_scene(name, global_scene_name?) -> global_name, local_name | nil
```

| パラメータ | 型 | 必須 | 説明 |
| ---- | ---- | ---- | ---- |
| `name` | string | ✅ | 検索するシーン名（照合用の名前に揃えてから前方一致） |
| `global_scene_name` | string | ❌ | 指定すると、そのグローバルシーンのローカルシーンだけを検索する |

`name` は、DSL の Call と同じく、照合用の名前（名前の中の文字と数字と `_` 以外の文字を `_` に置き換えた名前。[シーン名の照合](https://ekicyou.github.io/pasta/grammar/call-jump.html#シーン名の照合)）に揃えてから、シーン名の照合用の名前と前方一致させる。`"会話・朝"` も `"会話_朝"` も、照合用の名前 `会話_朝` で探す。照合用の名前が同じになるグローバルシーンは区別されず、同じ名前の候補になる。第 2 引数 `global_scene_name` には `global_name`（登録名）を渡す。第 2 引数は照合用の名前に揃えずに使う。

照合の相手は、シーン名の照合用の名前である。登録名（後述の `global_name`・`local_name`）の末尾の `_` と通し番号は照合に使わない。`name` は作者が書くシーン名として扱うため、登録名を渡しても、その登録名のシーンは指さない。たとえば `"メイン_1"` は、1 つ目の `＊メイン` ではなく、照合用の名前が `メイン_1` で始まるシーン（`＊メイン・1` など）を探す。第 2 引数を指定したときの第 1 引数（ローカルシーン名）も同じである。

Lua ブロックで定義したシーン関数（`function SCENE.名前(act)`）も、そのグローバルシーンのローカルシーンとして検索される。照合の相手は関数名の全体である（`SCENE.加算ループ` なら `加算ループ`）。ただし、関数名が `_` と半角数字（`0`〜`9`）で終わるときは、生成されたローカルシーンの登録名と見分けず、最後の `_` と数字を除いた部分が照合の相手になる（`SCENE.step_2` なら `step`）。全角数字で終わる名前は、関数名の全体が照合の相手である。

**戻り値**:

- 見つかったとき: `global_name, local_name` の 2 値。
  - `global_name`: グローバルシーンの登録名。シーン名の照合用の名前の後ろに `_` と、照合用の名前が同じグローバルシーンの通し番号（1 から）を付けた名前である（例: 1 つ目の `＊メイン` は `"メイン_1"`、2 つ目は `"メイン_2"`。1 つ目の `＊会話・朝` は `"会話_朝_1"`）。第 2 引数を指定したときは、第 2 引数と同じ名前が返る。
  - `local_name`: グローバルシーンの本体なら `"__start__"`。ローカルシーンなら、ローカルシーン名の照合用の名前の後ろに `_` と、同じグローバルシーン内の同名ローカルシーンの通し番号（1 から）を付けた名前（例: `"選択肢_1"`。`・選択・A` なら `"選択_A_1"`）。
- 見つからないとき: 何も返さない（受け取った変数は `nil` になる）。

`name` に空文字列を渡すと、Lua のエラー（`Scene search error: Invalid scene name: ''`）になる。

```lua
local SEARCH = require "@pasta_search"

-- グローバルシーンを検索する
local global, local_name = SEARCH:search_scene("メイン")
if global then
    print(global, local_name)  -- "メイン_1", "__start__"
end

-- メイン_1 のローカルシーンだけを検索する
local g, l = SEARCH:search_scene("選択肢", "メイン_1")
if g then
    print(g, l)  -- "メイン_1", "選択肢_1"
end

-- 見つからない場合
local result = SEARCH:search_scene("存在しないシーン")
if not result then
    print("Scene not found")
end
```

### search_word(name, global_scene_name?)

単語を前方一致で検索し、候補の値を 1 つ返す。

```lua
SEARCH:search_word(name, global_scene_name?) -> string | nil
```

| パラメータ | 型 | 必須 | 説明 |
| ---- | ---- | ---- | ---- |
| `name` | string | ✅ | 検索する単語キー（前方一致） |
| `global_scene_name` | string | ❌ | 指定すると、そのグローバルシーンのローカル単語だけを検索する（照合用の名前に揃えてから照合する） |

**戻り値**:

- 見つかったとき: 単語の値（文字列）。
- 見つからないとき: 何も返さない（受け取った変数は `nil` になる）。

第 2 引数には、`search_scene` が返す `global_name`（`"メイン_1"` の形の登録名）を渡す。DSL に書いたシーン名（`"メイン"`）では一致しない。登録名は照合用の名前を元にした名前である（1 つ目の `＊会話・朝` なら `"会話_朝_1"`）。

第 2 引数（検索の範囲）は、照合用の名前に揃えてから照合する。第 1 引数の単語キーは揃えずに、そのまま前方一致させる。

```lua
local SEARCH = require "@pasta_search"

-- グローバル単語を検索する
local word = SEARCH:search_word("挨拶")
if word then print(word) end  -- 例: "こんにちは"

-- メイン_1 のローカル単語だけを検索する
local g = SEARCH:search_scene("メイン")
local reply = SEARCH:search_word("返事", g)
```

### set_scene_selector(...) / set_word_selector(...)

テストのために、候補の選び方を決定論的にする。`set_scene_selector` はシーンの検索、`set_word_selector` は単語の検索に効く。

```lua
SEARCH:set_scene_selector(n1, n2, ...)  -- 決定論的な選択にする
SEARCH:set_scene_selector()             -- 既定（シャッフル）に戻す

SEARCH:set_word_selector(n1, n2, ...)   -- 決定論的な選択にする
SEARCH:set_word_selector()              -- 既定（シャッフル）に戻す
```

| パラメータ | 型 | 説明 |
| ---- | ---- | ---- |
| `n1, n2, ...` | integer | 1 個以上の整数 |

- 整数を 1 個以上渡すと、シャッフルをやめ、候補を決まった順に 1 つずつ返す。一巡すると先頭に戻る。
  - 順序は、候補の検索キーの文字コード順（バイト順）である。検索キーは、単語では単語キー、シーンでは照合の相手（通し番号を除いた名前。[search_scene](#search_scenename-global_scene_name)）である。定義を読み込んだ順ではない。たとえば `＠挨拶朝：朝1、朝2` を `＠挨拶昼：昼1` より先に定義しても、`search_word("挨拶")` は `昼1`・`朝1`・`朝2`・`昼1`…の順になる（「昼」の文字コードが「朝」より小さい）。
  - 同じ単語キーの値どうしは、定義した順に並ぶ。
  - 検索キーが同じシーンどうし（同名シーン）は、通し番号の小さい順に並ぶ。`メイン_2` は `メイン_10` より先になる。
- 引数なしで呼ぶと、既定のシャッフルに戻す。
- どちらの呼び出しも、それまでの並び順の記録を消す。呼んだ後の最初の検索は、候補の先頭（既定に戻した場合は並べ直した先頭）から始まる。
- 整数でない引数（文字列・小数など）を渡すと、Lua のエラー（`expected integer argument`）になる。
- 設定はランタイム全体の検索に効く。DSL の単語参照・Call・ランダムトークのシーン選択も同じ選び方になるため、テストの後は引数なしで呼んで既定に戻す。

```lua
SEARCH:set_scene_selector(0)  -- 同名シーンを通し番号順に選ぶ（メイン_1, メイン_2, ...）
SEARCH:set_word_selector(0)   -- ＠挨拶：こんにちは、やあ → "こんにちは", "やあ", "こんにちは", ...

-- テストの後は既定に戻す
SEARCH:set_scene_selector()
SEARCH:set_word_selector()
```
