# 内部設計の概要

ごきげんよう。ここから先は、ゴーストを作る方ではなく、pasta そのものを読み・直す方のための舞台裏ですわ。
トランスパイラが何を吐き、ランタイムがどう回り、SHIORI がどう応えるのか――わたくしが順に解き明かして差し上げます。
覚悟はよろしくて？ さあ、エンジンルームの扉を開けて参りましょう。

---

このパートは、pasta ランタイム（pasta.dll）の内部設計を題材ごとにまとめた解説である。この概要章では、読者・全体像・利用者向け章との関係・対象外の範囲と、各章が対象とするソース範囲を示す。

## この章の読者

このパートの対象読者は、pasta 本体のコードを読み・直すコントリビュータ（実装を担う AI エージェントを含む）である。ゴーストを作成するだけなら、このパートを読む必要はない。ゴースト作者が必要とする情報は、利用者向けパート（入門・文法・Lua・デバッグ・リファレンス）だけで完結する。

このパートの記述の権威は現行実装である。記述は執筆時点のコードと照合しており、完了済み spec の設計書や旧文書とコードが食い違う箇所はコードに合わせている。未実装の構想と、実装の不備（バグ候補）は書かない。

## 全体像

pasta ランタイムは、ゴーストの `.pasta` 辞書を Lua コードへ変換し、それを Lua VM で実行して SHIORI 応答を返す。データの流れと担当クレートは次のとおりである。

```text
                 .pasta 辞書
                    │  パース（pasta_dsl）
                    ▼
                   AST
                    │  トランスパイル（pasta_lua。登録と生成の単一走査、結果のキャッシュ）
                    ▼
                 Lua コード（キャッシュ内の pasta.scene.* と scene_dic.lua）
                    │  ローダ（pasta_lua。設定読込・自己展開・ファイル検出・モジュール解決）
                    ▼
                 Lua VM で実行（pasta_lua。辞書確定・イベントからシーンのコルーチン実行）
                    │  トークン → さくらスクリプトの組立と後処理（pasta_lua）
                    ▼
                 SHIORI 応答（pasta_shiori。pasta.dll の FFI 境界とアクタースレッド）
```

図はデータが変換される順序を示している。呼び出しの主従はこれと一致しない。起動時（SHIORI の `load`）には、SHIORI 層がローダを呼び、ローダが設定読込・フレームワークスクリプトの自己展開・ファイル検出を行ったうえで、更新された `.pasta` だけをパース・トランスパイルし、VM を構築する。以後の SHIORI の `request` は、VM を所有する専用スレッドへ渡され、Lua 側のイベントの振り分けからシーン関数のコルーチンが実行される。

| クレート | 役割 |
| -------- | ---- |
| `pasta_dsl` | Pasta DSL の pest 文法・パーサと AST 定義 |
| `pasta_core` | 言語に依存しないシーン・単語レジストリと、前方一致の検索表 |
| `pasta_lua` | トランスパイラ、ローダ、Lua VM の構築とランタイムモジュール（`@pasta_*` と `pasta.*`）、トーク出力、デバッグ基盤、ロギングとエンコーディング |
| `pasta_shiori` | SHIORI DLL（`pasta.dll`）としての FFI 境界と、VM を所有するアクターランタイム |

依存の向きは `pasta_shiori` → `pasta_lua` → `pasta_dsl`・`pasta_core` である。`pasta_core` は Lua にも DSL にも依存しない。

各段階の詳細は次の章で扱う。

- [トランスパイルパイプライン](transpiler.md) — パースから Lua コード生成まで。段階構成の定義、生成時最適化、トランスパイル結果キャッシュ
- [シーン・単語レジストリとシーン検索](registry-search.md) — 辞書の登録と実行時の確定、前方一致と重複時の選択、ローカル優先の検索順
- [ランタイム実行モデル](execution-model.md) — Lua VM の構築とモジュール登録、イベントからシーンのコルーチン実行、永続化
- [Lua ランタイム内部モジュール](internal-modules.md) — `pasta.*` 内部モジュールのモジュール単位のリファレンス（STORE・ACT・PROXY・SCENE・SAVE・`finalize_scene`・ユーティリティ）
- [ローダ自己展開とモジュール解決](loader.md) — 起動の段階構成、設定読込、自己展開と版比較、ファイル検出とモジュール名、searcher
- [SHIORI 層](shiori.md) — FFI 境界、アクターランタイム、イベント配送、非同期トーク、仮想イベントディスパッチャ（OnTalk・OnHour）、DLL のビルド構成
- [トーク出力とアピアランス](talk-output.md) — トークンからさくらスクリプトへの組立、ウェイト挿入と budoux 改行、サーフェスと着せ替えの復旧
- [デバッグ基盤とシーンキック](debug.md) — DAP バックエンド、デバッグ通信、セッション、ソースマップ、シーンキック
- [ロギングとエンコーディング](logging-encoding.md) — ログの初期化と出力、`@pasta_log`、`@enc` と OS 別の文字コード変換

ゴースト作者が `scripts/` から呼ぶランタイム API（ACT・WORD・GLOBAL・SAVE）の利用者向けリファレンスは、Lua パートの [スクリプト用ランタイム API](../lua/script-api.md) にある。このパートの [Lua ランタイム内部モジュール](internal-modules.md) は同じモジュールの内部の構造を扱う。

## 利用者向け章との関係

利用者から観測できる挙動（文法・公開 Lua API・`pasta.toml`・起動シーケンスとモジュール解決・デバッグ操作）は、利用者向け章が正である。同じ対象を利用者向け章とこのパートの両方が扱う場合、利用者から観測できる挙動については利用者向け章の記述を正とする。このパートはそれらの事実を再記述せず、利用者向け章へリンクしたうえで、その挙動を実現する仕組みだけを書く。利用者向け章とこのパートの記述が食い違う場合は、利用者向け章と現行実装を照合し、このパートを直す。

リンクの規則は次のとおりである。

- このパートの章は、利用者向け章の該当する見出しへリンクしてよい。利用者向け章からこのパートへの逆リンクは張らない。
- 1 つの事実は 1 つの章だけに書き、ほかの章はリンクで参照する。題材章は仕組みと処理の流れを、[Lua ランタイム内部モジュール](internal-modules.md) は個々のフィールド・関数・データ構造を、[スクリプト用ランタイム API](../lua/script-api.md) は作者から見た API を受け持つ。
- 題材章はソースの所在をリポジトリルートからの相対パスで書く。[Lua ランタイム内部モジュール](internal-modules.md) はスキルへの生成元であるため、リポジトリ内パスを書かず、ソースの所在は [ランタイム実行モデル](execution-model.md#ソースの所在) などの題材章へのリンクで示す。

## 対象外

`pasta_lsp`（言語サーバ）・`pasta_check`（リリース用 CLI）・`pasta_sample_ghost`（サンプルゴースト）は、このパートの対象外である。いずれも pasta.dll に載らず、ランタイムの外で動くツールまたは作例であり、ランタイムの内部設計ではないためである。これらが `pasta_dsl`・`pasta_lua` を利用する場合も、利用される側の仕組みはこのパートの該当章が扱う。

## 章と対象ソース範囲

各章が対象とするソース範囲を次の表に示す。spec の完了時には、変更したファイルをこの表と照合し、一致した章が実装に追従しているかを確認する。値が `/` で終わるセルはディレクトリ配下のすべてのファイルに、それ以外は記載のファイルだけに一致する。1 つのファイルが複数の章に一致した場合は、一致したすべての章を確認対象とする。表の各行は、題材章の「ソースの所在」の箇条書きと一致させる。[Lua ランタイム内部モジュール](internal-modules.md) の行は、同章が扱うモジュールのソースである。

| 章 | 対象ソース範囲 |
| -- | -------------- |
| [トランスパイルパイプライン](transpiler.md) | `crates/pasta_dsl/src/`、`crates/pasta_lua/src/transpiler.rs`、`crates/pasta_lua/src/code_gen/`、`crates/pasta_lua/src/context.rs`、`crates/pasta_lua/src/normalize.rs`、`crates/pasta_lua/src/string_literalizer.rs`、`crates/pasta_lua/src/config.rs`、`crates/pasta_lua/src/error.rs`、`crates/pasta_lua/src/loader/cache.rs`、`crates/pasta_lua/src/loader/process.rs` |
| [シーン・単語レジストリとシーン検索](registry-search.md) | `crates/pasta_core/src/`、`crates/pasta_lua/src/search/`、`crates/pasta_lua/src/runtime/finalize.rs`、`crates/pasta_lua/pasta_scripts/pasta/scene.lua`、`crates/pasta_lua/pasta_scripts/pasta/word.lua` |
| [ランタイム実行モデル](execution-model.md) | `crates/pasta_lua/src/runtime/`、`crates/pasta_lua/pasta_scripts/pasta/` |
| [Lua ランタイム内部モジュール](internal-modules.md) | `crates/pasta_lua/pasta_scripts/pasta/store.lua`、`crates/pasta_lua/pasta_scripts/pasta/act.lua`、`crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua`、`crates/pasta_lua/pasta_scripts/pasta/actor.lua`、`crates/pasta_lua/pasta_scripts/pasta/scene.lua`、`crates/pasta_lua/pasta_scripts/pasta/save.lua`、`crates/pasta_lua/pasta_scripts/pasta/init.lua`、`crates/pasta_lua/pasta_scripts/pasta/word.lua`、`crates/pasta_lua/pasta_scripts/pasta/buf.lua`、`crates/pasta_lua/pasta_scripts/pasta/lua_version.lua` |
| [ローダ自己展開とモジュール解決](loader.md) | `crates/pasta_lua/src/loader/`、`crates/pasta_lua/build.rs`、`crates/pasta_lua/build_zip.rs`、`crates/pasta_lua/src/runtime/searcher.rs`、`crates/pasta_lua/src/runtime/module_registry.rs`、`crates/pasta_lua/src/runtime/runtime_config.rs`、`crates/pasta_lua/pasta_scripts/main.lua`、`crates/pasta_lua/pasta_scripts/pasta/config.lua` |
| [SHIORI 層](shiori.md) | `crates/pasta_shiori/src/`、`crates/pasta_shiori/build.rs`、`crates/pasta_shiori/Cargo.toml`、`crates/pasta_lua/src/presentation/`、`crates/pasta_lua/src/runtime/renderer_injection.rs`、`crates/pasta_lua/pasta_scripts/pasta/shiori/`、`.cargo/config.toml` |
| [トーク出力とアピアランス](talk-output.md) | `crates/pasta_lua/src/sakura_script/`、`crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`、`crates/pasta_lua/pasta_scripts/pasta/shiori/appearance.lua`、`crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua` |
| [デバッグ基盤とシーンキック](debug.md) | `crates/pasta_lua/src/debug/`、`crates/pasta_lua/src/loader/source_map_build.rs`、`crates/pasta_lua/src/code_gen/source_map.rs`、`crates/pasta_lua/pasta_scripts/pasta/shiori/event/kick.lua` |
| [ロギングとエンコーディング](logging-encoding.md) | `crates/pasta_lua/src/logging/`、`crates/pasta_lua/src/encoding/`、`crates/pasta_lua/src/runtime/log.rs`、`crates/pasta_lua/src/runtime/enc.rs` |

範囲が重なる行（例: `crates/pasta_lua/src/runtime/` と `crates/pasta_lua/src/runtime/finalize.rs`）は意図的な重なりである。次のものは、章の記述に影響しない変更で照合が発火しないよう、表に載せない。

- テストだけを置く場所（`crates/pasta_lua/tests/` などの各クレートの `tests/`）。各章は「ソースの所在」の箇条書きの後にテストの所在を文で示す。`src/` の中のテストモジュール（`*_tests.rs` など）は、ディレクトリの範囲に含まれる場合にだけ一致する。
- ルートの [Cargo.toml](https://github.com/ekicyou/pasta/blob/main/Cargo.toml)。リリースプロファイルは [SHIORI 層](shiori.md) が扱うが、依存更新のたびに照合対象にならないよう載せない。`crates/pasta_shiori/Cargo.toml` は、DLL のライブラリ名と `crate-type` を [SHIORI 層](shiori.md) が扱うため載せる。
- `crates/pasta_lua/src/lib.rs`。モジュールの宣言と再エクスポートだけを持ち、どの章も記述の対象にしていない。

---

全体の地図は頭に入りまして？ フンッ、迷子になっても知りませんわよ……と言いたいところですけれど、迷ったらいつでもこの章へ戻っていらっしゃい。
さあ、まずはトランスパイラの心臓部から、熱く参りましょう！
