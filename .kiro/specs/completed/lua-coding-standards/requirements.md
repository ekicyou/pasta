# 要件定義書

## はじめに
Pastaプロジェクトの `pasta_lua` クレートにおけるLuaスクリプトのコーディング規約を策定し、AIエージェントが参照可能な形式でステアリング配置する。バグ予防、コード品質向上、一貫性維持を目的とする。

## プロジェクト概要（入力）
Luaスクリプトのコーディングルールを決めておいて、バグがなるべく出ないようにする。AI参照用のコーディング規約を作ってステアリングに配置する。

**追加要件**: 作成したコーディング規約に従い、既存のコードのリファクタリングを行う。

## 要件

### 要件1: コーディング規約ドキュメント作成
**目的:** AI開発エージェントとして、Luaコーディング規約がステアリングに配置されていることで、一貫したコードスタイルでLuaスクリプトを生成・修正できる

#### 受入基準
1. ステアリングシステムは、`.kiro/steering/` ディレクトリに `lua-coding.md` ファイルを提供すること (shall)
2. When AIエージェントが `.kiro/steering/lua-coding.md` を読み込んだとき、ステアリングシステムは命名規約、モジュールパターン、エラーハンドリングを含む包括的なLuaコーディングルールを提供すること (shall)
3. ステアリングシステムは、Pasta固有のLuaパターン（PASTAモジュールAPI、EmmyLuaアノテーション、メタテーブル設計）を文書化すること (shall)

### 要件2: 命名規約
**目的:** 開発者として、明確な命名規約が定義されていることで、コードの可読性と一貫性が向上する

#### 受入基準
1. コーディング規約は、ローカル変数と関数に snake_case を定義すること (shall)
2. コーディング規約は、モジュールファイル名に合わせたモジュールテーブル名に UPPER_CASE を定義すること (shall)（例: `actor.lua` → `ACTOR`）
3. コーディング規約は、内部クラスを含む全てのクラス実装メタテーブルに `_IMPL` サフィックスを定義すること (shall)（例: `ACTOR_IMPL`, `WORD_BUILDER_IMPL`）
4. When プライベートなモジュールメンバーを定義する際、コーディング規約はアンダースコアプレフィックスを要求すること (shall)（例: `_internal_func`）
5. コーディング規約は、ドメイン固有用語に日本語識別子を許可すること (shall)（例: `アクター名`, `シーン`）
6. コーディング規約は、クラスメタテーブルに PascalCase を禁止すること (shall)。全てのクラス実装は `MODULE_NAME_IMPL` パターンを使用すること

### 要件3: モジュール構造規約
**目的:** 開発者として、モジュールの構造パターンが標準化されていることで、循環参照を防ぎ保守性が向上する

#### 受入基準
1. コーディング規約は、各モジュールがファイル名に基づいた UPPER_CASE の単一モジュールテーブルを定義することを要求すること (shall)
2. コーディング規約は、全ての require 文をファイル先頭に配置することを要求すること (shall)
3. コーディング規約は、モジュールが最後にメインテーブルを返すことを要求すること (shall)
4. If 循環依存を引き起こす可能性のあるモジュールを require する場合、then コーディング規約は共有状態に `pasta.store` パターンを使用することを要求すること (shall)
5. コーディング規約は、標準的なモジュール構造テンプレートを提供すること (shall)

### 要件4: クラス設計パターン
**目的:** 開発者として、Rust風の明示的なクラス設計パターンが定義されていることで、クラスとシングルトンの混同を防ぎ、インスタンス生成が明確になる

**設計思想**: このパターンはRustの影響を受けており、実装側での明示性と利用側での利便性を両立する。

#### 受入基準
1. コーディング規約は、モジュールテーブル（`MODULE`）とクラス実装メタテーブル（`MODULE_IMPL`）を分離することを要求すること (shall)
2. コーディング規約は、新しいインスタンスを作成して返すコンストラクタ関数として `MODULE.new(args)` を要求すること (shall)
3. When `_IMPL` でクラスメソッドを定義する際、コーディング規約は暗黙のselfバグを防ぐため、ドット構文で明示的な `self` パラメータを要求すること (shall)（`function MODULE_IMPL.method(self, arg)`）
4. When インスタンスメソッドを呼び出す際、コーディング規約は利便性のためコロン構文（`instance:method(arg)`）を許可すること (shall)。これは自動的に `self` を渡す
5. コーディング規約は、コンストラクタで `setmetatable(obj, { __index = MODULE_IMPL })` パターンを要求すること (shall)
6. コーディング規約は、モジュールレベル関数（`MODULE.func(args)`）をインスタンスメソッドと分離することを要求すること (shall)
7. When シングルトン状態が必要な場合、コーディング規約はモジュールテーブル自体またはモジュールローカル変数を使用することを要求すること (shall)（Luaの `require` キャッシング動作を活用）
8. コーディング規約は、`MODULE.instance()` アンチパターンを禁止すること (shall)。モジュールをシングルトンとして扱うか `pasta.store` パターンを使用すること

#### シングルトンパターン（requireキャッシング経由）
```lua
--- @module pasta.config
--- このモジュールはシングルトン（モジュールテーブル自体が状態を保持）
local CONFIG = {
    debug = false,
    log_level = "info",
}

--- モジュールローカル状態（プライベートシングルトンデータ）
local _cache = {}

--- @param key string
--- @param value any
function CONFIG.set(key, value)
    CONFIG[key] = value
end

--- @param key string
--- @return any
function CONFIG.get(key)
    return CONFIG[key]
end

return CONFIG
-- 使用例: local CONFIG = require("pasta.config")
-- requireキャッシングにより、CONFIGは常に同じインスタンス
```

#### クラスパターンテンプレート
```lua
--- @module pasta.example
local EXAMPLE = {}

--- @class Example
--- @field name string
--- @field value number
local EXAMPLE_IMPL = {}
EXAMPLE_IMPL.__index = EXAMPLE_IMPL

--- 新しいExampleインスタンスを作成
--- @param name string
--- @param value number
--- @return Example
function EXAMPLE.new(name, value)
    local obj = {
        name = name,
        value = value,
    }
    setmetatable(obj, EXAMPLE_IMPL)
    return obj
end

--- インスタンスメソッド（明示的self、実装ではドット構文）
--- @param self Example
--- @param delta number
--- @return number
function EXAMPLE_IMPL.add(self, delta)
    self.value = self.value + delta
    return self.value
end

--- モジュールレベルユーティリティ関数（インスタンスメソッドではない）
--- @param a Example
--- @param b Example
--- @return Example
function EXAMPLE.merge(a, b)
    return EXAMPLE.new(a.name .. b.name, a.value + b.value)
end

return EXAMPLE

-- 使用例:
-- local ex = EXAMPLE.new("test", 10)
-- ex:add(5)  -- 呼び出し時はコロン構文が使える（利便性）
-- -- 以下と同等: EXAMPLE_IMPL.add(ex, 5)
```

### 要件5: EmmyLua型アノテーション規約
**目的:** 開発者として、型アノテーション規約が定義されていることで、IDE補完とドキュメント生成が向上する

#### 受入基準
1. コーディング規約は、各モジュールファイルの先頭に `@module` アノテーションを要求すること (shall)
2. コーディング規約は、クラス的なテーブルに `@class` アノテーションを要求すること (shall)
3. コーディング規約は、全ての公開関数に `@param` と `@return` アノテーションを要求すること (shall)
4. コーディング規約は、クラスプロパティに `@field` アノテーションを要求すること (shall)
5. When 関数がnilを返す可能性がある場合、コーディング規約は戻り値型アノテーションに `|nil` を要求すること (shall)
6. コーディング規約は、可変長引数関数に `@param ... type` 構文を要求すること (shall)（`@vararg` ではない）

### 要件6: エラーハンドリング規約
**目的:** 開発者として、エラーハンドリングパターンが標準化されていることで、デバッグと障害対応が容易になる

#### 受入基準
1. コーディング規約は、テーブル/関数アクセス前のnilチェックパターンを定義すること (shall)
2. When nilの可能性のあるテーブルにアクセスする際、コーディング規約はガード節パターンを要求すること (shall)
3. コーディング規約は、外部/リスクのある操作に pcall 使用パターンを定義すること (shall)
4. コーディング規約は、明示的なドキュメントなしのサイレントnil返却を禁止すること (shall)

### 要件7: Pastaランタイム固有規約
**目的:** 開発者として、Pasta固有のLuaパターンが文書化されていることで、ランタイムとの一貫した統合ができる

#### 受入基準
1. コーディング規約は、PASTAモジュールAPI使用パターンを文書化すること (shall)（`PASTA.create_actor`, `PASTA.create_scene`, `PASTA.create_word`）
2. コーディング規約は、CTX（コンテキスト）オブジェクトパターンとライフサイクルを文書化すること (shall)
3. コーディング規約は、シーン関数用のACT（アクション）オブジェクトパターンを文書化すること (shall)
4. コーディング規約は、アクター・アクション相互作用のためのPROXYパターンを文書化すること (shall)
5. コーディング規約は、共有状態管理のためのSTOREパターンを文書化すること (shall)

### 要件8: 既存コードリファクタリング
**目的:** 開発者として、既存のLuaコードが新しい規約に準拠していることで、コードベース全体の一貫性が保たれる

#### 受入基準
1. When `scripts/pasta/*.lua` をリファクタリングする際、リファクタリングシステムはEmmyLuaアノテーションが完全で一貫していることを保証すること (shall)
2. When リファクタリングする際、リファクタリングシステムは全ての公開関数が適切なドキュメントを持つことを保証すること (shall)
3. When リファクタリングする際、リファクタリングシステムは命名規約に従うことを保証すること (shall)（MODULE, MODULE_IMPLパターン）
4. When クラス的なモジュールをリファクタリングする際、リファクタリングシステムは要件4のクラス設計パターンを適用すること (shall)（MODULEとMODULE_IMPLを分離、実装では明示的self）
5. When メソッド実装をリファクタリングする際、リファクタリングシステムは `_IMPL` 定義内でコロン構文を明示的 `self` パラメータ付きドット構文に変換すること (shall)
6. When メソッド呼び出しをリファクタリングする際、リファクタリングシステムは利便性のためコロン構文（`:`）を許可すること (shall)（呼び出し側）
7. リファクタリングシステムは、既存の動作を保持すること (shall)（機能変更なし）
8. After リファクタリング後、テストシステムは既存の全テストを変更なしでパスすること (shall)

#### リファクタリング対象ファイル
以下のファイルはクラスパターンリファクタリングが必要:

| ファイル | 現在の問題 | 必要な変更 |
|----------|-----------|-----------|
| `act.lua` | ACTはクラス的だがシングルトンとして使用 | ACT/ACT_IMPLを分離、ACT.new()を使用 |
| `actor.lua` | ACTORはモジュールとクラス関数が混在、`WordBuilder`がPascalCase | ACTOR/ACTOR_IMPLを分離、WordBuilder→WORD_BUILDER_IMPL、ActorWordBuilder→ACTOR_WORD_BUILDER_IMPLにリネーム |
| `ctx.lua` | CTXは.new()があるがコロン構文を使用 | 明示的selfでドット構文に変換 |
| `scene.lua` | MOD命名、混在パターン | SCENEにリネーム、構造を明確化 |
| `word.lua` | MOD命名、WordBuilder PascalCaseパターン | MOD→WORD、WordBuilder→WORD_BUILDER_IMPLにリネーム、_IMPLパターンを適用 |
| `store.lua` | シングルトンパターンは正しい | 軽微な命名調整 |
| `global.lua` | シンプルなテーブル、クラスなし | 大きな変更不要 |
| `init.lua` | エントリーポイント、クラスなし | 変更不要 |
| `lua_specs/*_spec.lua` | テストファイルが `*_spec.lua` 命名 | 一貫性のため `*_test.lua` にリネーム |

### 要件9: テストとLint規約
**目的:** 開発者として、Luaテストとlint規約が定義されていることで、コード品質を自動検証できる

#### 受入基準
1. コーディング規約は、lua_testフレームワーク使用パターンを文書化すること (shall)（`expect`, `describe`, `it`）
2. コーディング規約は、テストファイル命名規約を `*_test.lua` と定義すること (shall)（AI検索信頼性のためRust `*_test.rs` と統一）
3. コーディング規約は、テスト構造テンプレートを提供すること (shall)（describe/itパターン）
4. コーディング規約は、luacheck設定と使用方法を文書化すること (shall)（`crates/pasta_lua/scriptlibs/luacheck/` に配置）
5. コーディング規約は、pastaプロジェクトパターン用の `.luacheckrc` 設定テンプレートを提供すること (shall)
