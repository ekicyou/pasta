# 内部設計の概要

ごきげんよう。ここから先は、ゴーストを作る方ではなく、pasta そのものを読み・直す方のための舞台裏ですわ。
トランスパイラが何を吐き、ランタイムがどう回り、SHIORI がどう応えるのか――わたくしが順に解き明かして差し上げます。
覚悟はよろしくて？ さあ、エンジンルームの扉を開けて参りましょう。

---

このパートは、pasta ランタイムの内部設計を題材ごとにまとめた解説である。この概要章では、読者・全体像・利用者向け章との関係・対象外の範囲と、各章が対象とするソース範囲を示す。

## この章の読者

このパートの対象読者は、pasta 本体のコードを読み・直すコントリビュータ（実装を担う AI エージェントを含む）である。ゴーストを作成するだけなら、このパートを読む必要はない。ゴースト作者が必要とする情報は、利用者向けパート（入門・文法・Lua・デバッグ・リファレンス）だけで完結する。

## 全体像

pasta ランタイムは、ゴーストの `.pasta` 辞書を Lua コードへ変換し、それを Lua VM で実行して SHIORI 応答を返す。主な流れと担当クレートは次のとおりである。

```text
.pasta 辞書
   │  パース（pasta_dsl）
   ▼
AST
   │  トランスパイル（pasta_lua。登録と生成・キャッシュ）
   ▼
Lua コード
   │  ローダ（pasta_lua。自己展開・モジュール解決・設定読込）
   ▼
Lua VM で実行（pasta_lua。辞書確定・シーンのコルーチン実行）
   │
   ▼
SHIORI 応答（pasta_shiori。pasta.dll）
```

| クレート | 役割 |
| -------- | ---- |
| `pasta_dsl` | Pasta DSL のパーサと AST 定義 |
| `pasta_core` | 言語非依存のシーン・単語レジストリ |
| `pasta_lua` | トランスパイラ、ローダ、Lua ランタイム、トーク出力、デバッグ基盤、ロギングとエンコーディング |
| `pasta_shiori` | SHIORI DLL（`pasta.dll`）としての FFI 境界とアクターランタイム |

各段階の詳細は次の章で扱う。

- [トランスパイルパイプライン](transpiler.md) — パースから Lua コード生成まで
- [シーン・単語レジストリとシーン検索](registry-search.md) — 辞書の登録・確定・検索
- [ランタイム実行モデル](execution-model.md) — Lua VM の構築とシーンのコルーチン実行
- [Lua ランタイム内部モジュール](internal-modules.md) — `pasta.*` 内部モジュールのリファレンス
- [ローダ自己展開とモジュール解決](loader.md) — 起動シーケンスの実装と設定読込
- [SHIORI 層](shiori.md) — FFI 境界・アクター・イベント配送
- [トーク出力とアピアランス](talk-output.md) — さくらスクリプトの組立と後処理
- [デバッグ基盤とシーンキック](debug.md) — DAP バックエンドとソースマップ
- [ロギングとエンコーディング](logging-encoding.md) — ログ出力と文字コード変換

## 利用者向け章との関係

利用者から観測できる挙動（文法・公開 Lua API・`pasta.toml`・起動シーケンスとモジュール解決・デバッグ操作）は、利用者向け章が正である。このパートはそれらの事実を再記述せず、利用者向け章へリンクしたうえで、その挙動を実現する仕組みだけを書く。利用者向け章とこのパートの記述が食い違う場合は、利用者向け章と現行実装を照合し、このパートを直す。

## 対象外

`pasta_lsp`（言語サーバ）・`pasta_check`（リリース用 CLI）・`pasta_sample_ghost`（サンプルゴースト）は、pasta.dll ランタイムの外にあるツールであり、このパートの対象外である。

## 章と対象ソース範囲

各章が対象とするソース範囲を次の表に示す。spec の完了時には、変更したファイルをこの表と照合し、一致した章が実装に追従しているかを確認する。値が `/` で終わるセルはディレクトリ配下のすべてのファイルに、それ以外は記載のファイルだけに一致する。1 つのファイルが複数の章に一致した場合は、一致したすべての章を確認対象とする。

| 章 | 対象ソース範囲 |
| -- | -------------- |
| [トランスパイルパイプライン](transpiler.md) | `crates/pasta_dsl/src/`、`crates/pasta_lua/src/transpiler.rs`、`crates/pasta_lua/src/code_gen/`、`crates/pasta_lua/src/context.rs`、`crates/pasta_lua/src/normalize.rs`、`crates/pasta_lua/src/string_literalizer.rs`、`crates/pasta_lua/src/config.rs`、`crates/pasta_lua/src/loader/cache.rs` |
| [シーン・単語レジストリとシーン検索](registry-search.md) | `crates/pasta_core/src/`、`crates/pasta_lua/src/search/`、`crates/pasta_lua/src/runtime/finalize.rs`、`crates/pasta_lua/pasta_scripts/pasta/scene.lua`、`crates/pasta_lua/pasta_scripts/pasta/word.lua` |
| [ランタイム実行モデル](execution-model.md) | `crates/pasta_lua/src/runtime/`、`crates/pasta_lua/pasta_scripts/pasta/`、`crates/pasta_lua/pasta_scripts/ct.lua` |
| [Lua ランタイム内部モジュール](internal-modules.md) | `crates/pasta_lua/pasta_scripts/pasta/store.lua`、`crates/pasta_lua/pasta_scripts/pasta/act.lua`、`crates/pasta_lua/pasta_scripts/pasta/scene.lua`、`crates/pasta_lua/pasta_scripts/pasta/word.lua`、`crates/pasta_lua/pasta_scripts/pasta/global.lua`、`crates/pasta_lua/pasta_scripts/pasta/save.lua`、`crates/pasta_lua/pasta_scripts/pasta/buf.lua`、`crates/pasta_lua/pasta_scripts/pasta/lua_version.lua` |
| [ローダ自己展開とモジュール解決](loader.md) | `crates/pasta_lua/src/loader/`、`crates/pasta_lua/build.rs`、`crates/pasta_lua/build_zip.rs`、`crates/pasta_lua/src/runtime/searcher.rs`、`crates/pasta_lua/src/runtime/runtime_config.rs`、`crates/pasta_lua/pasta_scripts/main.lua`、`crates/pasta_lua/pasta_scripts/pasta/config.lua` |
| [SHIORI 層](shiori.md) | `crates/pasta_shiori/src/`、`crates/pasta_shiori/build.rs`、`crates/pasta_lua/src/presentation/`、`crates/pasta_lua/src/runtime/renderer_injection.rs`、`crates/pasta_lua/pasta_scripts/pasta/shiori/`、`.cargo/config.toml` |
| [トーク出力とアピアランス](talk-output.md) | `crates/pasta_lua/src/sakura_script/`、`crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`、`crates/pasta_lua/pasta_scripts/pasta/shiori/appearance.lua`、`crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua` |
| [デバッグ基盤とシーンキック](debug.md) | `crates/pasta_lua/src/debug/`、`crates/pasta_lua/src/loader/source_map_build.rs`、`crates/pasta_lua/src/code_gen/source_map.rs`、`crates/pasta_lua/pasta_scripts/pasta/shiori/event/kick.lua` |
| [ロギングとエンコーディング](logging-encoding.md) | `crates/pasta_lua/src/logging/`、`crates/pasta_lua/src/encoding/`、`crates/pasta_lua/src/runtime/log.rs`、`crates/pasta_lua/src/runtime/enc.rs` |

ディレクトリの範囲が重なる行（例: `crates/pasta_lua/src/runtime/` と `crates/pasta_lua/src/runtime/finalize.rs`）は意図的な重なりである。ルートの [Cargo.toml](https://github.com/ekicyou/pasta/blob/main/Cargo.toml) のリリースプロファイルは [SHIORI 層](shiori.md) が扱うが、依存更新のたびに照合対象にならないよう、この表には載せない。

---

全体の地図は頭に入りまして？ フンッ、迷子になっても知りませんわよ……と言いたいところですけれど、迷ったらいつでもこの章へ戻っていらっしゃい。
さあ、まずはトランスパイラの心臓部から、熱く参りましょう！
