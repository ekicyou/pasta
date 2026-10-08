# 公開モジュール API

> 【クローディア】道具箱を開ける時間ですわ。検索、永続化、設定、ログ、文字コード変換――Pasta ランタイムは、あなたの手間を肩代わりしてくれる頼もしい子たちを、いくつものモジュールとして用意しておりますの。

> 【アンソニー】では本日は、その子たちとの顔合わせでございますね。

> 【お辞儀】ええ、まずは全員の顔と名前を覚えていただきましょう。……皆さま、どうぞよろしくお願いいたしますわ。

---

Pasta ランタイムは Rust 側から Lua VM へモジュール群を公開している。`require` で読み込んで使う。
この章群は、`scripts/` 配下のスクリプト、ランタイム同梱のスクリプト、DSL 内の ` ```lua ``` ` ブロックから
利用できる公開モジュールを、1 モジュール 1 章で扱う。本章はその一覧と、全モジュールに共通する事項をまとめる。

## モジュール一覧

`@` で始まる名前は Rust 組み込みモジュール（ランタイムが提供）、`pasta.` で始まる名前は
ランタイム同梱の Lua モジュールである。この章群は前者の Rust 組み込みモジュールを扱う。
`pasta.` のモジュール（`pasta.word` など）の実用パターンは [scripts/ の記述パターン](../patterns.md) で扱う。

| モジュール | 用途 | 読み込み方 | 章 |
| ---- | ---- | ---- | ---- |
| `@pasta_search` | シーン・単語の前方一致検索 | `require "@pasta_search"` | [@pasta_search](pasta-search.md) |
| `@pasta_persistence` | セーブデータの永続化 | `require "@pasta_persistence"` | [@pasta_persistence](pasta-persistence.md) |
| `@pasta_config` | pasta.toml の設定の読み取り | `pcall(require, "@pasta_config")` | [@pasta_config](pasta-config.md) |
| `@pasta_sakura_script` | さくらスクリプト変換（ウェイト挿入・自動改行） | `require "@pasta_sakura_script"` | [@pasta_sakura_script](pasta-sakura-script.md) |
| `@enc` | UTF-8 ⇔ ANSI の文字コード変換 | `require "@enc"` | [@enc](enc.md) |
| `@pasta_log` | ロギング | `require "@pasta_log"` | [@pasta_log](pasta-log.md) |

加えて、mlua-stdlib の統合モジュール `@json`・`@yaml`・`@regex`・`@assertions`・`@testing` が既定で有効である。
`@env`（環境変数・パスへのアクセス）は既定で無効で、通常のゴーストからは使えない。これらは [mlua-stdlib 統合モジュール](mlua-stdlib.md) で扱う。

## 共通事項

### 対象の Lua 方言

対象方言は **LuaJIT 2.1**（Lua 5.1 系）である。言語の基礎は [Lua の基礎](../basics.md) で扱う。

### 読み込みと利用できる時期

- 一覧のモジュールは、ランタイムの初期化時（`main.lua` を読み込む前）にすべて `package.loaded` へ登録される。`require` はファイルを探さず、登録済みの表（`@pasta_search` はユーザーデータ）を返す。同じ名前を何度 `require` しても同じものが返る。
- `@pasta_search` は、シーン辞書の読み込みの最後に `finalize_scene()` が実行された時点で検索対象が確定する。それより前に呼んだ結果は使えない（[利用できる時期](pasta-search.md#利用できる時期)）。シーン関数やイベントハンドラの実行時には確定している。
- `@pasta_config` は `pcall(require, "@pasta_config")` で読む。ゴーストとして読み込まれたランタイムでは常に登録されているが、ランタイムの外（Lua だけの単体テストなど）でスクリプトを動かすと登録されておらず、素の `require` はエラーになる。ランタイム同梱のスクリプトもこの形で読んでいる（[読み込み方](pasta-config.md#読み込み方)）。

### 呼び出しの形

- `@pasta_search` はメソッドとして `:` で呼ぶ（`SEARCH:search_scene(...)`）。
- その他のモジュールは表の関数として `.` で呼ぶ（`persistence.load()`・`log.info(...)` など）。

### 失敗の返し方

- 呼び出しの失敗を戻り値で返す関数は、成功時に `値, nil`、失敗時に `nil, エラーメッセージ` の 2 値を返す（`@pasta_persistence` の `save`、`@enc` の変換関数）。
- 引数の型が合わない呼び出し（表を渡すべき所に文字列を渡すなど）は、戻り値ではなく Lua のエラーになる（`@enc` の変換関数は例外で、文字列以外を渡しても `nil, エラーメッセージ` を返す）。

### モジュールのメタデータ

`@pasta_persistence`・`@pasta_sakura_script`・`@enc`・`@pasta_log` の表は、`_VERSION`（版の文字列）と `_DESCRIPTION`（説明の文字列）を持つ。

---

> 【したり顔】道具の名前と使い方、ざっと頭に入れていただけたかしら。最初は覚えきれなくても結構ですのよ。必要になったら、この一覧へ戻ってくればよいのですから。さあ、端から順に手に取ってまいりましょう。

> 【アンソニー】皆さまのお顔とお名前は、私も控えに書き留めておきました。
