---
name: pasta-lua-coding
description: >-
  pasta.dll Luaランタイム APIリファレンスとコーディング規約。
  ゴーストの scripts/ 配下のカスタムLuaスクリプトや、
  Pasta DSL内のLuaブロック実装を支援する。
  USE FOR: pasta lua, pasta_lua, Lua API, Luaスクリプト, scripts/,
  単語辞書一括投入, WORD.create, イベントハンドラ, REG, RES,
  永続化, @pasta_persistence, save, @pasta_search,
  @pasta_config, @pasta_sakura_script, @enc,
  ACT, SCENE, STORE, GLOBAL, SAVE, lua_test, luacheck,
  pasta lua coding, pasta runtime API.
  DO NOT USE FOR: Pasta DSL文法, .pastaファイル編集,
  pasta_dsl crate, pasta_core crate, Rustクレート実装,
  汎用Luaプログラミング, SHIORIプロトコル実装.
metadata:
  author: ekicyou
  version: "1.0.0"
---

# Pasta Lua Coding Skill

## §1 Purpose & Prerequisites

**目的**: 自然言語の指示からpasta_luaランタイムに準拠したLuaコードを正確に生成するサポートを提供する。

**対象ドメイン**:
- `scripts/` 配下のカスタムLuaスクリプト（`main.lua` がエントリーポイント）
- Pasta DSL内の ` ```lua ``` ` ブロックで記述するシーン関数

**前提条件**: ゴーストプロジェクトが既に存在すること（`pasta.toml`、`dic/`、`scripts/` が揃っている）。

**役割分離**: 姉妹スキル `pasta-ghost-authoring` がPasta DSL文法（`.pasta`ファイルの記述）を担当し、本スキルはその下位層であるLuaランタイム層を担当する。DSLの ` ```lua ``` ` ブロック内のコード記述や、`scripts/` 配下の独自スクリプト開発を支援する。

**scripts/ フォルダ**: ゴーストディレクトリ直下の `scripts/` に独自Luaスクリプトを配置する。`main.lua` がエントリーポイントとしてpastaランタイムに読み込まれ、シーン関数・単語定義・イベントハンドラ等をセットアップする。

### DSL vs Lua 判断基準

| ケース | 推奨 | 理由 |
|--------|------|------|
| 数個の単語定義 | DSL (`＠単語：値1、値2`) | 宣言的で簡潔 |
| 数十〜数百件の単語一括投入 | Lua (`WORD.create_*`) | ループ/外部データ読み込みが必要 |
| 基本的なシーン定義 | DSL (`＊シーン名`) | 可読性が高い |
| 条件分岐を含む複雑なロジック | Lua (シーン関数) | DSLの制御構文は限定的 |
| カスタムSHIORIイベント処理 | Lua (REGテーブル) | DSLではイベントハンドラを直接定義不可 |
| 外部データ（JSON/YAML）の読み込み | Lua (`@json`/`@yaml`) | DSLには外部ファイル操作機能なし |

**自己完結性**: 本スキルは別リポジトリにコピーして単体で機能する。pastaリポジトリ内の他ファイルへの参照に依存しない。

**詳細リファレンス**: `references/` 配下に完全なAPIリファレンスとコーディング規約を配置。必要に応じて `read_file` でロードする。

---

## §2 Quick Reference

### Rust組み込みモジュール

| モジュール | 用途 | require方法 |
|-----------|------|------------|
| `@pasta_search` | シーン・単語検索 | `require` 直接 |
| `@pasta_persistence` | セーブデータ永続化 | `require` 直接 |
| `@pasta_sakura_script` | さくらスクリプト変換 | `require` 直接 |
| `@enc` | UTF-8 ⇔ ANSI変換 | `require` 直接 |
| `@pasta_config` | pasta.toml設定読み取り | `pcall(require, ...)` 保護必須 |

### 内部Luaモジュール（pasta.*名前空間）

| モジュール | 用途 | 主要API |
|-----------|------|--------|
| `pasta.store` | 一元データ管理 | `STORE.actors`, `STORE.scenes`, `STORE.reset()` |
| `pasta.scene` | シーン登録・検索 | `SCENE.create_scene()`, `SCENE.search()` |
| `pasta.word` | 単語ビルダー | `WORD.create_global()`, `WORD.create_local()`, `WORD.create_actor()` |
| `pasta.global` | ユーザー定義関数 | `GLOBAL.関数名 = function(act) ... end` |
| `pasta.save` | 永続化データ | `require("pasta.save")` |
| `pasta.act` | シーン実行コンテキスト | `act:init_scene()`, `act:talk()`, `act:yield()` |
| `pasta.shiori.event.register` | イベントハンドラ登録 | `REG.EventName = function(req) ... end` |
| `pasta.shiori.res` | SHIORIレスポンス | `RES.ok()`, `RES.no_content()` |

### mlua-stdlib統合モジュール

| モジュール | 用途 | デフォルト |
|-----------|------|----------|
| `@json` | JSON encode/decode | ✅ 有効 |
| `@yaml` | YAML encode/decode | ✅ 有効 |
| `@regex` | 正規表現 | ✅ 有効 |
| `@assertions` | アサーション | ✅ 有効 |
| `@testing` | テストフレームワーク | ✅ 有効 |
| `@env` | 環境変数・パスアクセス | ❌ 無効（セキュリティ上） |

### DSL→Luaブリッジ基本形

```lua
-- DSL内 ```lua ブロックから呼ばれるシーン関数の定型
function SCENE.func_name(act)
    local save, var = act:init_scene(SCENE)  -- 必須: save/var を取得
    act:talk(act.さくら.actor, "セリフ")    -- アクター名でトーク
    act:yield()                               -- トークンをyield
end
```

---

## §3 Coding Conventions

命名規約（snake_case/UPPER_CASE/`_IMPL`サフィックス）、`MODULE/MODULE_IMPL`分離によるクラス設計、`STORE`パターンによる循環参照回避、EmmyLua型注釈（`@class`, `@field`, `@param`, `@return`）、エラーハンドリング（ガードクローズ, `pcall`, nilチェック）の規約。

> 📖 詳細: [references/coding-conventions.md](references/coding-conventions.md)

---

## §4 Runtime API

Rust組み込みモジュールの完全APIリファレンス。`@pasta_search`（シーン・単語検索、フォールバック戦略）、`@pasta_persistence`（セーブデータ永続化）、`@pasta_config`（`pcall`必須）、`@pasta_sakura_script`（ウェイトタグ変換）、`@enc`（UTF-8⇔ANSI）。mlua-stdlib（`@json`, `@yaml`, `@regex`等）も含む。

> 📖 詳細: [references/runtime-api.md](references/runtime-api.md)

---

## §5 Internal Modules

pasta.*名前空間の内部Luaモジュール。`STORE`（一元データ管理）、`ACT`（シーン実行コンテキスト、`init_scene`/`talk`/`yield`等）、`SCENE`（シーン登録・検索・コルーチン実行）、`WORD`（ビルダーパターン単語定義）、`GLOBAL`（ユーザー定義関数）、`SAVE`（永続化データ）、`finalize_scene`（検索インデックス構築）。

> 📖 詳細: [references/internal-modules.md](references/internal-modules.md)

---

## §6 SHIORI Handlers

SHIORIイベントハンドラの登録と応答。`REG`テーブルにイベントハンドラを登録し、`RES.ok()`/`RES.no_content()`等でレスポンスを返す。主要イベント（OnBoot, OnClose, OnMouseDoubleClick等）のreference[N]仕様。`OnTalk`/`OnHour`を自動発行する仮想ディスパッチャ。REG未登録時のシーン検索フォールバックチェーン。

> 📖 詳細: [references/shiori-handlers.md](references/shiori-handlers.md)

---

## §7 Testing & Lint

BDD風テストフレームワーク `lua_test`（`describe`/`test`/`expect`マッチャー）。テストファイル規約（`*_test.lua`命名、init.lua登録）。`set_scene_selector`/`set_word_selector`による決定論的テスト。luacheck設定と実行方法。

> 📖 詳細: [references/testing-lint.md](references/testing-lint.md)

---

## References

本スキルの詳細リファレンス。必要に応じて `read_file` でロードすること。

| ファイル | 概要 |
|---------|------|
| [references/runtime-api.md](references/runtime-api.md) | Rust組み込みモジュール（@pasta_search, @pasta_persistence等）の完全API |
| [references/internal-modules.md](references/internal-modules.md) | pasta.*名前空間（STORE, ACT, SCENE, WORD等）の完全リファレンス |
| [references/shiori-handlers.md](references/shiori-handlers.md) | SHIORIイベントハンドラ（REG, RES, イベント一覧, 仮想ディスパッチャ） |
| [references/coding-conventions.md](references/coding-conventions.md) | 命名規約、モジュール構造、クラス設計、型注釈、エラーハンドリング |
| [references/testing-lint.md](references/testing-lint.md) | lua_test, テストファイル規約, 決定論的テスト, luacheck |
