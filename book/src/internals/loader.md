# ローダ自己展開とモジュール解決

ごきげんよう。pasta.dll が目を覚ましてから、最初のトークを話すまで――その舞台裏の段取りをお見せいたしますわ。
同梱スクリプトの自己展開、`require` の解決、設定読込。手順さえ分かれば、起動はもう謎ではございませんの。さあ、参りましょう。

---

この章では、起動シーケンスの実装、フレームワークスクリプトの自己展開、モジュール解決、設定読込を扱う。利用者から見た起動シーケンスとモジュール検索の挙動は [起動シーケンスとモジュール解決](../reference/startup.md) が正であり、この章はそれを実現する仕組みだけを書く。

## 目的と責務

ローダは、起動シーケンスの各段階を実行し、DLL に埋め込んだフレームワークスクリプトをゴーストディレクトリへ自己展開し、ファイルを検出してモジュール名を生成し、`require` を解決できる状態を整える。設定読込（`pasta.toml` の読込と既定値の補完の仕組み）もこの章の責務である。

## 構成要素

起動シーケンスを進めるローダ、ビルド時に作る埋め込み zip とその展開・版比較、ファイル検出とモジュール名の生成、モジュール検索（`searcher`）、設定読込から成る。

## 処理とデータの流れ

ローダは設定を読み込んだのち、埋め込みスクリプトの版を比較して必要なら展開し、ファイルを検出してトランスパイルへ渡し、モジュール検索を設定して Lua 側のエントリスクリプトを実行する。

## 境界の受け渡し

ビルドスクリプトが埋め込み zip を作って DLL に含め、Rust 側のローダが展開と設定読込を所有する。読み込んだ設定は Rust 側のランタイムと Lua 側のスクリプトへ渡される。

## 不変条件と制約

自己展開は埋め込み版とディスク上の版が異なる場合にだけ行われる。モジュール検索の優先順位は [起動シーケンスとモジュール解決](../reference/startup.md) が定める順序に従う。

## ソースの所在

- `crates/pasta_lua/src/loader/`
- `crates/pasta_lua/build.rs`
- `crates/pasta_lua/build_zip.rs`
- `crates/pasta_lua/src/runtime/searcher.rs`
- `crates/pasta_lua/src/runtime/runtime_config.rs`
- `crates/pasta_lua/pasta_scripts/main.lua`
- `crates/pasta_lua/pasta_scripts/pasta/config.lua`

## 経緯

- [pasta-scripts-self-deploy](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-scripts-self-deploy) — フレームワークスクリプトの自己展開
- [lua-module-path-resolution](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/lua-module-path-resolution) — モジュールパス解決
- [lua-require-robustness](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/lua-require-robustness) — 長パス・非 ANSI パスでの `require`
- [pasta-config-restructure](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-config-restructure) — `pasta.toml` の構成

---

目覚めの儀式も、こうして並べれば整然としたものでしょう？ フンッ、わたくしの説明が分かりやすいおかげですわね。
次は、ベースウェアとの窓口、SHIORI 層へ参りましょう！
