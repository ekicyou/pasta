# トランスパイルパイプライン

ごきげんよう。作者の書いた `.pasta` が、どうやって Lua のコードに化けるのか――その錬金術の工程をお見せいたしますわ。
パースから登録、生成、そしてキャッシュまで。一本の流れとして追えば、何も怖くはございませんの。さあ、参りましょう。

---

この章では、`.pasta` 辞書を Lua コードへ変換するトランスパイルパイプラインを扱う。

## 目的と責務

トランスパイルパイプラインは、Pasta DSL のソースをパースして AST を作り、シーン・単語をレジストリへ登録しながら Lua コードを生成する。生成時の最適化（末尾呼び出し・継続行の話者引継ぎ・文字列リテラルの表記選択）と、変換結果を再利用するトランスパイル結果キャッシュもこの章の責務である。トランスパイル時の処理と実行時の辞書確定から成る段階構成の定義も、この章が権威を持つ。

## 構成要素

パイプラインは、pest 文法と AST を持つパーサ（`pasta_dsl`）、トランスパイル文脈とレジストリへの登録を担う `TranspileContext`、要素ごとに Lua コードを組み立てるコード生成部、出力の正規化、文字列リテラル化、トランスパイル結果キャッシュから成る。

## 処理とデータの流れ

ソースはパーサで AST になり、トランスパイラがファイル項目を文書順にたどって、レジストリへの登録と Lua コードの生成を進める。生成されたコードは正規化を経て出力され、キャッシュに保存される。ソースマップの生成側の出入口もこの流れに含まれ、その詳細は [デバッグ基盤とシーンキック](debug.md) で扱う。

## 境界の受け渡し

`pasta_dsl` が AST を所有して `pasta_lua` のトランスパイラへ渡し、トランスパイラは生成した Lua コードをローダへ渡す。シーン・単語の登録情報は、生成コードの実行を通じて実行時の辞書確定へ引き継がれる。

## 不変条件と制約

生成される Lua コードは、ランタイムの Lua 方言（LuaJIT 2.1）で実行できる形でなければならない。キャッシュは、ソースの更新時刻がキャッシュより新しい場合に作り直される。

## ソースの所在

- `crates/pasta_dsl/src/`
- `crates/pasta_lua/src/transpiler.rs`
- `crates/pasta_lua/src/code_gen/`
- `crates/pasta_lua/src/context.rs`
- `crates/pasta_lua/src/normalize.rs`
- `crates/pasta_lua/src/string_literalizer.rs`
- `crates/pasta_lua/src/config.rs`
- `crates/pasta_lua/src/loader/cache.rs`

## 経緯

- [dsl-separation](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/dsl-separation) — DSL パーサと AST の独立クレート化
- [pasta-lua-cache-transpiler](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-lua-cache-transpiler) — トランスパイル結果キャッシュ
- [pasta-transpiler-variable-expansion](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-transpiler-variable-expansion) — 変数展開の生成

---

流れはつかめまして？ フンッ、この程度で目を回すようでは困りますわよ。
次は、登録されたシーンと単語がどう探し出されるのかを見て参りましょう！
