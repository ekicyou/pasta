# シーン・単語レジストリとシーン検索

ごきげんよう。同じ名前のシーンがいくつもあるとき、pasta はどれを選ぶのか――気になって夜も眠れませんわね？
レジストリへの登録から辞書の確定、前方一致での検索まで、わたくしが手際よくご案内いたします。さあ、参りましょう。

---

この章では、シーン・単語のレジストリと、実行時のシーン検索を扱う。

## 目的と責務

レジストリは、トランスパイルと実行で集めたシーン・単語を保持し、名前による検索を提供する。前方一致の検索、同名候補からの選択と乱数、ローカル優先の検索順、実行時の辞書確定（Rust 側の `finalize`）と Lua 側の収集データの受け渡し、`@pasta_search` の内部がこの章の責務である。

## 構成要素

言語非依存のシーン・単語レジストリ（`pasta_core`）、RadixMap による前方一致の索引、Rust 側で辞書を確定する処理、Lua から検索を呼ぶ `@pasta_search` の実装、Lua 側でシーン・単語を収集するモジュールから成る。

## 処理とデータの流れ

生成コードの実行によって Lua 側にシーン・単語が集まり、辞書確定の処理がそれを Rust 側のレジストリへ渡して索引を作る。以後のシーン検索・単語検索は、この確定済みの索引に対して行われる。Lua 側の収集データ構造は [Lua ランタイム内部モジュール](internal-modules.md) で扱う。

## 境界の受け渡し

Lua 側のモジュールが収集データを所有し、辞書確定の時点で Rust 側へ渡す。確定後の索引は Rust 側が所有し、Lua からは `@pasta_search` を通じて参照する。

## 不変条件と制約

検索は辞書の確定後にだけ行える。同名の候補が複数ある場合の選択は乱数に従い、ローカルシーンはグローバルシーンより先に検索される。

## ソースの所在

- `crates/pasta_core/src/`
- `crates/pasta_lua/src/search/`
- `crates/pasta_lua/src/runtime/finalize.rs`
- `crates/pasta_lua/pasta_scripts/pasta/scene.lua`
- `crates/pasta_lua/pasta_scripts/pasta/word.lua`

## 経緯

- [pasta_search_module](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta_search_module) — 検索モジュール
- [scene-search-integration](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/scene-search-integration) — Lua 側シーン検索の統合
- [pasta-scene-dictionary-finalization](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-scene-dictionary-finalization) — 実行時の辞書確定

---

名前ひとつ引くのにも、これだけの段取りがございますのよ。おほほ、奥が深いでしょう？
次は、見つけたシーンがどうやって動き出すのか、実行モデルへ参りましょう！
