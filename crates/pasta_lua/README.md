# pasta_lua

Pasta DSL を Lua にトランスパイルし、Lua VM 上で実行するためのクレートです。

## 概要

`pasta_lua` は Pasta DSL の Lua バックエンド実装を提供します。主な機能：

- Pasta DSL → Lua ソースコードへのトランスパイル
- Lua VM ホスティング（mlua ベース）
- モジュール解決とパッケージパス管理
- SHIORI/3.0 プロトコル統合サポート

ゴースト作者向けの使い方とコントリビュータ向けの内部設計は、[pasta マニュアル](https://ekicyou.github.io/pasta/) にまとめています。

## アーキテクチャ

起動時は `PastaLoader` が `pasta.toml` を読み、同梱のフレームワークスクリプトを自己展開し、辞書ファイルを検出します。更新された `.pasta` だけを `pasta_dsl` でパースして `LuaTranspiler` で Lua コードへ変換し（結果はキャッシュ）、`PastaLuaRuntime` が Lua VM を構築して組み込みモジュールを登録します。実行時は `pasta_core` のレジストリで辞書を確定し、SHIORI のイベントからシーン関数をコルーチンとして実行して、蓄積したトークンをさくらスクリプトに組み立てます。DLL としての SHIORI の入口は `pasta_shiori` が担います。

詳細は [内部設計: 内部設計の概要](https://ekicyou.github.io/pasta/internals/index.html#全体像) を参照してください。章ごとの対象ソース範囲は [章と対象ソース範囲](https://ekicyou.github.io/pasta/internals/index.html#章と対象ソース範囲) にあります。

## ディレクトリ構成

`pasta_lua` はゴーストの `ghost/master/`（SHIORI の `load_dir`）を基準ディレクトリとして扱います。主な構成要素は次のとおりです。

- `pasta.toml` — 設定ファイル（必須）
- `dic/` — Pasta DSL の辞書（既定の検出パターンは `dic/**/*.pasta`。同じ場所の `.lua` もトランスパイルせずに読み込まれる）
- `scripts/` — ゴースト作者の Lua スクリプト（同梱スクリプトより優先される）
- `scriptlibs/` — 外部 Lua ライブラリ
- `profile/pasta/` — 実行時に作られる領域（同梱スクリプトの自己展開先、トランスパイル結果のキャッシュ、保存データ、ログ）

ゴーストのフォルダ構成は [最初のゴーストを作る](https://ekicyou.github.io/pasta/getting-started/first-ghost.html#ゴーストのフォルダ構成)、辞書ファイルの検出パターンと `.lua` の扱いは [pasta_patterns](https://ekicyou.github.io/pasta/reference/pasta-toml.html#pasta_patterns) を参照してください。検出とモジュール名の生成の仕組みは [内部設計: ファイル検出](https://ekicyou.github.io/pasta/internals/loader.html#ファイル検出)・[モジュール名の生成](https://ekicyou.github.io/pasta/internals/loader.html#モジュール名の生成) にあります。

## 設定ファイル（pasta.toml）

SHIORI として起動するために必須なのは `[actor]`（1 つ以上）だけで、他のセクションは省略すると既定値が使われます。

```toml
[actor."女の子"]
spot = 0

[actor."男の子"]
spot = 1
```

全セクション・全フィールドの分類と既定値は [pasta.toml リファレンス](https://ekicyou.github.io/pasta/reference/pasta-toml.html) を参照してください。`[actor]` の設定が Lua 側のアクターになる仕組みは [内部設計: @pasta_config からの初期化](https://ekicyou.github.io/pasta/internals/internal-modules.html#pasta_config-からの初期化) にあります。

## Lua モジュール検索パスと起動

`require` は既定で `profile/pasta/save/lua` → `scripts` → `profile/pasta/pasta_scripts` → `profile/pasta/cache/lua` → `scriptlibs` の順にモジュールを探します（`[loader] lua_search_paths` で変更可）。`package.path` は UTF-8 として解釈され、設置パスの長さや ANSI コードページに無い文字に左右されません（ホスト側の制約は起動シーケンスの章を参照）。起動時には `main`・`pasta.shiori.entry`・`pasta.scene_dic` を順に読み込み、いずれかの失敗で起動を中止します。

詳細は [起動シーケンスとモジュール解決](https://ekicyou.github.io/pasta/reference/startup.html) を参照してください。searcher の実装は [内部設計: require の解決（searcher）](https://ekicyou.github.io/pasta/internals/loader.html#require-の解決searcher) にあります。

## 組み込みモジュール

ランタイムは `@pasta_search`・`@pasta_persistence`・`@pasta_config`・`@pasta_sakura_script`・`@enc`・`@pasta_log` を登録し、mlua-stdlib の `@json`・`@yaml`・`@regex`・`@assertions`・`@testing` を既定で有効にします（`@env` は既定で無効）。各モジュールの API は [公開モジュール API](https://ekicyou.github.io/pasta/lua/modules/index.html) を参照してください。

## 使用方法

### 基本的な使用法

```rust
use pasta_lua::PastaLoader;

// ベースディレクトリから起動
let runtime = PastaLoader::load("path/to/ghost/master/")?;

// Lua コードを実行
let result = runtime.exec("return 1 + 1")?;
```

### カスタム設定での起動

```rust
use pasta_lua::{PastaLoader, RuntimeConfig};

// デフォルト設定（std_all + assertions, testing, regex, json, yaml）
let config = RuntimeConfig::new();
let runtime = PastaLoader::load_with_config("path/to/base", config)?;

// 全機能有効（std_all_unsafe + @env モジュール含む）
let config = RuntimeConfig::full();
let runtime = PastaLoader::load_with_config("path/to/base", config)?;

// 最小構成（std_all のみ、mlua-stdlib モジュールなし）
let config = RuntimeConfig::minimal();
let runtime = PastaLoader::load_with_config("path/to/base", config)?;

// カスタム構成
let config = RuntimeConfig::from_libs(vec![
    "std_all".into(),
    "regex".into(),
    "-std_io".into(),  // io を除外
]);
let runtime = PastaLoader::load_with_config("path/to/base", config)?;
```

### トランスパイラー単独使用

```rust
use pasta_lua::{LuaTranspiler, PastaLuaRuntime};

let transpiler = LuaTranspiler::default();
let mut output = Vec::new();

// Pasta AST をトランスパイル
let context = transpiler.transpile(&pasta_file, &mut output)?;
let lua_code = String::from_utf8(output)?;

// ランタイムを作成して実行
let runtime = PastaLuaRuntime::new(context)?;
runtime.exec(&lua_code)?;
```

## SHIORI 統合

SHIORI のエントリ（`pasta.shiori.entry`）はフレームワークスクリプトとして同梱され、起動時に読み込まれます。ゴースト作者は `REG` にイベントハンドラを登録し、ハンドラはさくらスクリプトの文字列（応答の `Value`）を返します。SHIORI/3.0 の応答文字列への組み立てはエンジンが `RES`（`pasta.shiori.res`）で行います。

- イベントハンドラと `RES`: [SHIORI イベントとハンドラ](https://ekicyou.github.io/pasta/lua/shiori-events.html)
- イベント配送の内部: [内部設計: Lua 側の SHIORI エントリとイベント配送](https://ekicyou.github.io/pasta/internals/shiori.html#lua-側の-shiori-エントリとイベント配送)
- トークンからさくらスクリプトへの組み立て: [内部設計: さくらスクリプトの組立](https://ekicyou.github.io/pasta/internals/talk-output.html#さくらスクリプトの組立)

## 関連クレート

- [`pasta_dsl`](https://crates.io/crates/pasta_dsl) - DSLパーサー、AST
- [`pasta_core`](https://crates.io/crates/pasta_core) - レジストリ（シーン/単語テーブル）
- [`pasta_shiori`](https://crates.io/crates/pasta_shiori) - SHIORI DLL ラッパー
- [プロジェクト概要](https://github.com/ekicyou/pasta) - pasta プロジェクト全体

## ライセンス

プロジェクトルートの [LICENSE](https://github.com/ekicyou/pasta/blob/main/LICENSE) ファイルを参照してください。
