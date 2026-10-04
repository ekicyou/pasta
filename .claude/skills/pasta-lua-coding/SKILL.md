---
name: pasta-lua-coding
description: >-
  pasta.dll Luaランタイム APIリファレンスとコーディング規約。
  ゴーストの scripts/ 配下のカスタムLuaスクリプトや、
  Pasta DSL内のLuaブロック実装を支援する。
  USE FOR: pasta lua, pasta_lua, Lua API, Luaスクリプト, scripts/, pasta_scripts/,
  単語辞書一括投入, WORD.create, イベントハンドラ, REG, RES,
  永続化, @pasta_persistence, save, @pasta_search,
  @pasta_config, @pasta_sakura_script, @enc, @pasta_log,
  ロギング, logging, log, trace, debug, info, warn, error,
  ACT, SCENE, STORE, GLOBAL, SAVE, lua_test, luacheck,
  mocks, lua_test.mocks, モックライブラリ, mock install, mock reset,
  pasta lua coding, pasta runtime API.
  DO NOT USE FOR: Pasta DSL文法, .pastaファイル編集,
  pasta_dsl crate, pasta_core crate, Rustクレート実装,
  汎用Luaプログラミング, SHIORIプロトコル実装.
metadata:
  author: ekicyou
  version: "1.11.1"
---

# Pasta Lua Coding Skill

## §1 Purpose & Prerequisites

**目的**: 自然言語の指示からpasta_luaランタイムに準拠したLuaコードを正確に生成するサポートを提供する。

**対象ドメイン**:
- `scripts/` 配下のカスタムLuaスクリプト（`main.lua` がエントリーポイント、`pasta_scripts/` より優先）
- `pasta_scripts/` 配下の標準ランタイムスクリプト（エンジン同梱）
- Pasta DSL内の ` ```lua ``` ` ブロックで記述するシーン関数

**前提条件**: ゴーストプロジェクトが既に存在すること（`pasta.toml`、`dic/`、`pasta_scripts/` が揃っている）。

**役割分離**: 姉妹スキル `pasta-ghost-authoring` がPasta DSL文法（`.pasta`ファイルの記述）を担当し、本スキルはその下位層であるLuaランタイム層を担当する。DSLの ` ```lua ``` ` ブロック内のコード記述や、`scripts/` 配下のカスタムスクリプト・`pasta_scripts/` 配下のランタイムスクリプト開発を支援する。

**scripts/ フォルダ**: ゴーストディレクトリ直下の `scripts/` にユーザーカスタムLuaスクリプトを配置する。`main.lua` がエントリーポイントとしてpastaランタイムに読み込まれ、シーン関数・単語定義・イベントハンドラ等をセットアップする。`scripts/` は `pasta_scripts/` より優先されるため、同名ファイルでランタイムの動作を上書きできる。

> **`main` のロード失敗は致命**: `main` の読み込みに失敗すると、警告を残して続行するのではなく**起動が中止される**（`load` が失敗を返す）。`scripts/main.lua` を置いて既定の `main.lua` を上書きした場合、そのファイルの構文エラーや実行時エラーはゴーストを起動不能にする。原因は `profile/pasta/logs/pasta.log` の `fatal=true` を含む行に、失敗したモジュール名と根本原因の全文が記録される。詳細は[起動シーケンスとモジュール解決](references/startup.md)を参照。

> **モジュール解決はパスの長さ・文字種に依存しない**（詳細: [references/startup.md](references/startup.md#設置パスの長さ文字種に依存しない)）: `require` は設置パスが 260 文字を超えても、ANSIコードページで表現できない文字を含んでいても成功する。ただしこの保証はランタイムが解決するモジュールに限る。`io.open` / `loadfile` / `dofile` / `package.searchpath` はOSのnarrow APIを使うため対象外であり、永続化には `@pasta_persistence` を使うこと。また `package.path` へ独自エントリを追記する場合は**UTF-8の文字列**を使うこと。

### DSL vs Lua 判断基準

| ケース                            | 推奨                     | 理由                                  |
| --------------------------------- | ------------------------ | ------------------------------------- |
| 数個の単語定義                    | DSL (`＠単語：値1、値2`) | 宣言的で簡潔                          |
| 数十〜数百件の単語一括投入        | Lua (`WORD.create_*`)    | ループ/外部データ読み込みが必要       |
| 基本的なシーン定義                | DSL (`＊シーン名`)       | 可読性が高い                          |
| 条件分岐を含む複雑なロジック      | Lua (シーン関数)         | DSLの制御構文は限定的                 |
| カスタムSHIORIイベント処理        | Lua (REGテーブル)        | DSLではイベントハンドラを直接定義不可 |
| 外部データ（JSON/YAML）の読み込み | Lua (`@json`/`@yaml`)    | DSLには外部ファイル操作機能なし       |

**自己完結性**: 本スキルは別リポジトリにコピーして単体で機能する。pastaリポジトリ内の他ファイルへの参照に依存しない。

**詳細リファレンス**: `references/` 配下に完全なAPIリファレンスとコーディング規約を配置。必要に応じて `read_file` でロードする。公開モジュール API・SHIORI イベント・起動シーケンスの権威は pasta 利用者マニュアル（<https://ekicyou.github.io/pasta/>）であり、`references/` の生成ファイルはその写しである。

### references 一覧

`references/` の各ファイルの区分（マニュアルから生成／スキル手書き）を次の表に示す。

- **生成ファイルは編集しない**。修正はマニュアルの該当章で行い、pasta リポジトリで再生成する。生成ファイルと手書きファイルが同じ事実を扱う場合は、生成ファイルの記述を正とする。
- この `SKILL.md` 本文の要約・早見表（§2 のモジュール一覧表、§3〜§7 の要約など）は、参照先を選ぶための**非規範の要約**である。食い違う場合は生成ファイルが正である。
- このスキルを別の場所へ持ち出して使っている場合、持ち出し先を更新するときは `references/` を丸ごと置き換える（旧名のファイルを残さない）。

| ファイル | 区分 | 生成元マニュアル章 | 用途 |
| -------- | ---- | ------------------ | ---- |
| [modules-index.md](references/modules-index.md) | 生成（マニュアルから） | [公開モジュール API](https://ekicyou.github.io/pasta/lua/modules/index.html) | Rust組み込みモジュールの一覧と全モジュール共通の事項（読み込み方・呼び出しの形・失敗の返し方） |
| [pasta-search.md](references/pasta-search.md) | 生成（マニュアルから） | [@pasta_search](https://ekicyou.github.io/pasta/lua/modules/pasta-search.html) | シーン・単語の前方一致検索、テスト用セレクタ |
| [pasta-persistence.md](references/pasta-persistence.md) | 生成（マニュアルから） | [@pasta_persistence](https://ekicyou.github.io/pasta/lua/modules/pasta-persistence.html) | セーブデータの永続化、セーブキーの命名規約 |
| [pasta-config.md](references/pasta-config.md) | 生成（マニュアルから） | [@pasta_config](https://ekicyou.github.io/pasta/lua/modules/pasta-config.html) | `pasta.toml` の設定の読み取り |
| [pasta-sakura-script.md](references/pasta-sakura-script.md) | 生成（マニュアルから） | [@pasta_sakura_script](https://ekicyou.github.io/pasta/lua/modules/pasta-sakura-script.html) | さくらスクリプト変換（ウェイト挿入・自動改行） |
| [enc.md](references/enc.md) | 生成（マニュアルから） | [@enc](https://ekicyou.github.io/pasta/lua/modules/enc.html) | UTF-8 ⇔ ANSI の文字コード変換 |
| [pasta-log.md](references/pasta-log.md) | 生成（マニュアルから） | [@pasta_log](https://ekicyou.github.io/pasta/lua/modules/pasta-log.html) | ロギング（trace/debug/info/warn/error） |
| [mlua-stdlib.md](references/mlua-stdlib.md) | 生成（マニュアルから） | [mlua-stdlib 統合モジュール](https://ekicyou.github.io/pasta/lua/modules/mlua-stdlib.html) | `@json`・`@yaml`・`@regex`・`@assertions`・`@testing`・`@env` と Lua 標準ライブラリの構成 |
| [shiori-events.md](references/shiori-events.md) | 生成（マニュアルから） | [SHIORI イベントとハンドラ](https://ekicyou.github.io/pasta/lua/shiori-events.html) | `REG`・`RES`・主要イベント・シーン関数フォールバック・仮想ディスパッチャ |
| [startup.md](references/startup.md) | 生成（マニュアルから） | [起動シーケンスとモジュール解決](https://ekicyou.github.io/pasta/reference/startup.html) | モジュール検索パス・起動シーケンス・既知の制限・起動しないときの調べ方 |
| [script-api.md](references/script-api.md) | 生成（マニュアルから） | [スクリプト用ランタイム API](https://ekicyou.github.io/pasta/lua/script-api.html) | ACT・WORD・GLOBAL・SAVE のスクリプト向け API |
| [internal-modules.md](references/internal-modules.md) | 生成（マニュアルから） | [Lua ランタイム内部モジュール](https://ekicyou.github.io/pasta/internals/internal-modules.html) | `pasta.*` の内部モジュール（STORE・SCENE・PROXY・`finalize_scene` 等） |
| [coding-conventions.md](references/coding-conventions.md) | 手書き（スキルが権威） | — | 命名規約、モジュール構造、クラス設計、型注釈、エラーハンドリング |
| [testing-lint.md](references/testing-lint.md) | 手書き（スキルが権威） | — | lua_test、テストファイル規約、決定論的テストの手順、luacheck |

---

## §2 Quick Reference

> この節の表は参照先を選ぶための非規範の要約である。食い違う場合は生成ファイル（[modules-index.md](references/modules-index.md)・[shiori-events.md](references/shiori-events.md) ほか）が正。

### Rust組み込みモジュール

| モジュール             | 用途                                    | require方法                    |
| ---------------------- | --------------------------------------- | ------------------------------ |
| `@pasta_search`        | シーン・単語検索                        | `require` 直接                 |
| `@pasta_persistence`   | セーブデータ永続化                      | `require` 直接                 |
| `@pasta_sakura_script` | さくらスクリプト変換                    | `require` 直接                 |
| `@enc`                 | UTF-8 ⇔ ANSI変換                        | `require` 直接                 |
| `@pasta_config`        | pasta.toml設定読み取り                  | `pcall(require, ...)` 保護必須 |
| `@pasta_log`           | ロギング（trace/debug/info/warn/error） | `require` 直接                 |

### 内部Luaモジュール（pasta.*名前空間）

| モジュール                    | 用途                   | 主要API                                                              |
| ----------------------------- | ---------------------- | -------------------------------------------------------------------- |
| `pasta.store`                 | 一元データ管理         | `STORE.actors`, `STORE.scenes`, `STORE.reset()`                      |
| `pasta.scene`                 | シーン登録・検索       | `SCENE.create_scene()`, `SCENE.search()`                             |
| `pasta.word`                  | 単語ビルダー           | `WORD.create_global()`, `WORD.create_local()`, `WORD.create_actor()` |
| `pasta.global`                | ユーザー定義関数       | `GLOBAL.関数名 = function(act) ... end`                              |
| `pasta.save`                  | 永続化データ           | `require("pasta.save")`                                              |
| `pasta.act`                   | シーン実行コンテキスト | `act:init_scene()`, `act:talk()`, `act:yield()`, `act:choice()`, `act:choice_timeout()`, `act:actor_proxy()`, `act:global_fn()`, `act:arith()` |
| `pasta.shiori.event.register` | イベントハンドラ登録   | `REG.EventName = function(act) ... end`                              |
| `pasta.shiori.res`            | SHIORIレスポンス       | `RES.ok()`, `RES.no_content()`                                       |

### mlua-stdlib統合モジュール

| モジュール    | 用途                   | デフォルト               |
| ------------- | ---------------------- | ------------------------ |
| `@json`       | JSON encode/decode     | ✅ 有効                   |
| `@yaml`       | YAML encode/decode     | ✅ 有効                   |
| `@regex`      | 正規表現               | ✅ 有効                   |
| `@assertions` | アサーション           | ✅ 有効                   |
| `@testing`    | テストフレームワーク   | ✅ 有効                   |
| `@env`        | 環境変数・パスアクセス | ❌ 無効（セキュリティ上） |

### DSL→Luaブリッジ基本形

```lua
-- DSL内 ```lua ブロックから呼ばれるシーン関数の定型
function SCENE.func_name(act)
    local save, var = act:init_scene(SCENE)  -- 必須: save/var を取得
    act:talk(act.さくら.actor, "セリフ")    -- アクター名でトーク
end                                           -- 終了時に自動で出力（末尾に yield は書かない）
```

---

## §3 Coding Conventions

命名規約（snake_case/UPPER_CASE/`_IMPL`サフィックス）、`MODULE/MODULE_IMPL`分離によるクラス設計、`STORE`パターンによる循環参照回避、EmmyLua型注釈（`@class`, `@field`, `@param`, `@return`）、エラーハンドリング（ガードクローズ, `pcall`, nilチェック）の規約。

> 📖 詳細: [references/coding-conventions.md](references/coding-conventions.md)

### SAVE キー命名規約

`pasta.save` テーブルのキーは以下の2種類に分類される：

| 種類                 | 命名規則                                | 例                                                   | 備考                                     |
| -------------------- | --------------------------------------- | ---------------------------------------------------- | ---------------------------------------- |
| **エンジン予約キー** | `pasta_` プレフィックス付き             | `pasta_talk_interval_min`, `pasta_talk_interval_max` | pasta エンジン内部で参照・制御されるキー |
| **ゴースト固有キー** | 任意命名（`pasta_` プレフィックス禁止） | `my_flag`, `talk_count`                              | ゴースト作者が自由に使用可能             |

**重要**: `pasta_` プレフィックスはエンジン予約領域。ゴースト固有キーに `pasta_` を使用すると、エンジン動作に意図しない影響を与える可能性がある。

---

## §4 Runtime API

Rust組み込みモジュールの完全APIリファレンス。モジュールの一覧と共通事項は [references/modules-index.md](references/modules-index.md)、各モジュールは 1 モジュール 1 ファイル。`@pasta_search`（シーン・単語の前方一致検索、テスト用セレクタ）、`@pasta_persistence`（セーブデータ永続化）、`@pasta_config`（`pcall`必須）、`@pasta_sakura_script`（ウェイト挿入・自動改行）、`@enc`（UTF-8⇔ANSI）、`@pasta_log`（ロギング、trace/debug/info/warn/error）。mlua-stdlib（`@json`, `@yaml`, `@regex`等）も含む。

> 📖 詳細: [references/modules-index.md](references/modules-index.md)（一覧）／[pasta-search.md](references/pasta-search.md)・[pasta-persistence.md](references/pasta-persistence.md)・[pasta-config.md](references/pasta-config.md)・[pasta-sakura-script.md](references/pasta-sakura-script.md)・[enc.md](references/enc.md)・[pasta-log.md](references/pasta-log.md)・[mlua-stdlib.md](references/mlua-stdlib.md)

---

## §5 Internal Modules

pasta.*名前空間の内部Luaモジュール。`STORE`（一元データ管理）、`ACT`（シーン実行コンテキスト、`init_scene`/`talk`/`yield`等）、`SCENE`（シーン登録・検索・コルーチン実行）、`WORD`（ビルダーパターン単語定義）、`GLOBAL`（ユーザー定義関数）、`SAVE`（永続化データ）、`finalize_scene`（検索インデックス構築）。

> 📖 詳細: [references/internal-modules.md](references/internal-modules.md)（内部）／[script-api.md](references/script-api.md)（スクリプト向け API）

---

## §6 SHIORI Handlers

SHIORIイベントハンドラの登録と応答。`REG`テーブルに `function(act)` 形式のハンドラを登録し、リクエストは `act.req` で読む。ハンドラは `Value` にする文字列（またはシーンのコルーチン、`nil`）を返し、エンジンがそれを応答にする（`RES` はエンジンが応答文字列を作るモジュール）。主要イベント（OnBoot, OnClose, OnMouseDoubleClick, OnChoiceSelectEx等）のReference仕様。`OnTalk`/`OnHour`を自動発行する仮想ディスパッチャ。`OnChoiceSelectEx`選択肢自動ルーティング。REG未登録時のシーン関数フォールバック。

> 📖 詳細: [references/shiori-events.md](references/shiori-events.md)

---

## §7 Testing & Lint

BDD風テストフレームワーク `lua_test`（`describe`/`test`/`expect`マッチャー）。テストファイル規約（`*_test.lua`命名、init.lua登録）。`set_scene_selector`/`set_word_selector`による決定論的テスト。luacheck設定と実行方法。

> 📖 詳細: [references/testing-lint.md](references/testing-lint.md)
