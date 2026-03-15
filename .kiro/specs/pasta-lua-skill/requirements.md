# Requirements Document

## Introduction

pasta.dll用のLuaコード実装を助けるコーディングスキル（VS Code Copilot Skill形式）を開発する。

**主な使用シナリオ**: ゴースト開発者が「大量の単語辞書をLuaコードで投入したい」「カスタムイベントハンドラをLuaで書きたい」「永続化データを操作したい」等と指示し、LLMがpasta_luaのランタイムAPI・コーディング規約に準拠したLuaコードを生成する。本スキルはこの**自然言語→pasta_lua準拠Luaコード変換**に必要なAPI知識・規約・パターン集をLLMに提供する。

既存の `steering/lua-coding.md`（Luaコーディング規約）および `crates/pasta_lua/LUA_API.md`（Lua APIリファレンス）を情報ソースとし、LLMがpasta_luaランタイム向けLuaコードを正確に生成するために必要な知識を、スキルファイル内に転記・体系化する。

**情報ソース明記の原則**: スキルに転記する情報は権威的ドキュメント（`LUA_API.md`, `steering/lua-coding.md`）を正とし、転記元をスキル内に明記する。

**姉妹スキルとの関係**: `.agents/skills/pasta-ghost-authoring/SKILL.md`（Pasta DSL文法スキル）がDSL層を担当するのに対し、本スキルはDSLの下位層であるLuaランタイム層を担当する。Pasta DSLのLuaブロック（` ```lua ``` `）内のコード記述や、`scripts/` 配下のカスタムLuaスクリプト開発を支援する。

**配置と使用形態**: 姉妹スキルと同様に、別リポジトリのゴーストディレクトリにコピーして使用することを前提とする。スキルファイルはpastaリポジトリ内の他ドキュメントへの参照に依存せず、必要な情報をすべてスキルフォルダ内に自己完結的に内包しなければならない。配置先は `.agents/skills/pasta-lua-coding/` とし、VS Code GitHub Copilot の skill 機構により自動的にLLMのコンテキストへ注入される。

## Project Description (Input)
仕様llm-grammar-skillに関連したスキルの作成。pasta dll用のLuaコード実装を助けるスキルが欲しい。例えば、大量の単語辞書をLuaコードを使って投入するなど、pasta_luaの知識をサポートするためのSKILL.mdを作成する。

## Requirements

### Requirement 1: スキルファイル構造の定義

**Objective:** LLMコーディングエージェントとして、pasta_luaランタイム向けLuaコーディングサポートスキルが標準的なVS Code Copilot Skill形式（SKILL.md）で定義されていること により、GitHub Copilotが適切なタイミングでスキルを呼び出せるようにしたい。

#### Acceptance Criteria
1. The スキル shall `.agents/skills/pasta-lua-coding/SKILL.md` にファイルを配置する
2. The SKILL.md shall YAML Frontmatter形式で `name`, `description`（USE FOR / DO NOT USE FOR トリガーフレーズを含む）等のスキルメタデータを定義する
3. When 開発者がpasta.dll用Luaコード記述・ランタイムAPI使用・単語辞書一括投入・カスタムイベントハンドラ作成を依頼した場合, the スキル shall 自動的にコンテキストとして提供される
4. The SKILL.md shall スキルの目的（自然言語→pasta_lua準拠Luaコード変換サポート）・対象ドメイン・前提条件を冒頭に明記する
5. The スキルフォルダ shall 別リポジトリにコピーして単体で機能するよう、pastaリポジトリ内の他ファイルへの参照に依存せず、必要な情報をすべてスキルフォルダ内に自己完結的に内包する
6. The SKILL.md shall 姉妹スキル `pasta-ghost-authoring`（DSL層担当）との役割分離を明記する
7. The SKILL.md shall Pasta DSLでは手間がかかるケース（大量の単語投入、構造化データからの変換等）について、Luaでの実装が適切である理由と判断基準を示す

### Requirement 2: Luaコーディング規約の組み込み

**Objective:** LLMとして、pasta_luaランタイムの規約に準拠したLuaコードを生成するために、コーディング規約がスキル内に含まれていること により、スタイルの一貫性があるコードを生成できるようにしたい。

#### Acceptance Criteria
1. The スキル shall 命名規約（snake_case変数、UPPER_CASEモジュールテーブル、`_IMPL`サフィックス）を含む
2. The スキル shall 標準モジュール構造（require → モジュールテーブル → ローカル関数 → 公開関数 → return）を説明する
3. The スキル shall MODULE/MODULE_IMPL分離パターン（モジュールテーブルとクラス実装メタテーブルの分離）を説明する
4. The スキル shall ドット構文によるメソッド定義（明示的self）とコロン構文による呼び出しの規約を説明する
5. The スキル shall 日本語識別子の使用許可範囲（内部変数・関数はOK、公開API・モジュールテーブルはNG）を説明する
6. The スキル shall EmmyLuaアノテーション規約（`@module`, `@class`, `@field`, `@param`, `@return`）を説明する
7. The スキル shall エラーハンドリング規約（ガードクローズ、pcall、nilチェック）を説明する
8. The スキル shall 命名規約・モジュール構造の記述を `steering/lua-coding.md` と一致させる

### Requirement 3: ランタイムAPIリファレンスの組み込み

**Objective:** LLMとして、pasta_luaが提供するRust組み込みモジュールやLuaモジュールのAPIを正確に使用するために、APIリファレンスがスキル内に含まれていること により、正しいAPI呼び出しコードを生成できるようにしたい。

#### Acceptance Criteria
1. The スキル shall `@pasta_search` モジュール（`search_scene`, `search_word`, `set_scene_selector`, `set_word_selector`）のシグネチャ・パラメータ・戻り値を含む
2. The スキル shall `@pasta_persistence` モジュール（`load`, `save`）のシグネチャと使用パターンを含む
3. The スキル shall `@pasta_config` モジュール（pasta.tomlカスタムフィールド読み取り）の使用パターンを含む
4. The スキル shall `@pasta_sakura_script` モジュール（`talk_to_script`）のシグネチャとactor.talkテーブルの設定フィールドを含む
5. The スキル shall `@enc` モジュール（`to_ansi`, `to_utf8`）のシグネチャとエラーハンドリングパターンを含む
6. The スキル shall mlua-stdlib統合モジュール（`@json`, `@yaml`, `@regex`, `@assertions`, `@testing`）の概要と基本使用例を含む
7. The スキル shall Rustネイティブモジュール（`@pasta_*`プレフィックス）のrequire方法（直接require vs pcall保護）の使い分けを説明する
8. The スキル shall Rustネイティブモジュールの API情報を `LUA_API.md` と一致させる

### Requirement 4: pasta内部Luaモジュール構造の組み込み

**Objective:** LLMとして、pastaランタイムの内部Luaモジュール群（`pasta.*`名前空間）の構造と用途を理解するために、モジュール構造がスキル内に含まれていること により、適切なモジュールの選択とAPI呼び出しを行えるようにしたい。

#### Acceptance Criteria
1. The スキル shall STOREパターン（`pasta.store` による一元データ管理、循環参照回避）を説明する
2. The スキル shall ACTオブジェクト（シーン関数の引数、`init_scene`, `talk`, `yield`, `word`, `call`等のメソッド）を説明する
3. The スキル shall SCENEモジュール（`pasta.scene` のシーン登録・検索・`create_scene`・`co_exec`）を説明する
4. The スキル shall WORDモジュール（`pasta.word` のビルダーパターンAPI：`create_global`, `create_local`, `create_actor`、`.entry()` メソッドチェーン）を説明する
5. The スキル shall GLOBALモジュール（`pasta.global` のユーザー定義グローバル関数テーブル）を説明する
6. The スキル shall SAVEモジュール（`pasta.save` の永続化データアクセス、ACT経由の使用パターン）を説明する
7. The スキル shall `pasta.finalize_scene()` の呼び出しタイミングと役割を説明する

### Requirement 5: SHIORIイベントハンドラの Lua実装パターン

**Objective:** LLMとして、開発者がカスタムイベントハンドラをLuaで記述したい場合に、REGテーブル・RESモジュール・仮想ディスパッチャの使用方法がスキルに含まれていること により、正しいSHIORIハンドラコードを生成できるようにしたい。

#### Acceptance Criteria
1. The スキル shall REGテーブル（`pasta.shiori.event.register`）へのイベントハンドラ登録パターンを説明する
2. The スキル shall RESモジュール（`pasta.shiori.res`：`ok`, `ok_with`, `no_content`, `err`）のレスポンス生成パターンを説明する
3. The スキル shall 主要SHIORIイベント（OnBoot, OnFirstBoot, OnClose, OnGhostChanged, OnMouseDoubleClick, OnSecondChange, OnMinuteChange）のreqパラメータ（reference配列）を含む
4. The スキル shall シーン関数フォールバック（REGテーブル未登録時にSCENE.searchで検索）の動作を説明する
5. The スキル shall 仮想ディスパッチャ（OnTalk/OnHour自動発行、`pasta.toml` の `talk_interval_min`/`talk_interval_max` 設定）を説明する

### Requirement 6: Luaブロック統合パターン集

**Objective:** LLMとして、Pasta DSLのLuaブロック（` ```lua ``` `）内でのコード記述パターンがスキルに含まれていること により、DSLとLuaの連携コードを正確に生成できるようにしたい。

#### Acceptance Criteria
1. The スキル shall Luaブロック内関数定義の基本パターン（`function SCENE.func(act) ... end`）を説明する
2. The スキル shall `act:init_scene(SCENE)` による save/var 取得パターンを説明する
3. The スキル shall Luaブロック内からの単語参照（`act:word(name)` 4段階検索）を説明する
4. The スキル shall Luaブロック内からのシーンCall（`act:call(global, key, attrs)`）を説明する
5. The スキル shall GLOBALテーブルへの関数登録パターン（`scripts/main.lua` 等でのカスタム関数定義）を説明する

### Requirement 7: テスト・Lint規約の組み込み

**Objective:** LLMとして、生成したLuaコードのテストコードも併せて生成するために、テストフレームワーク・Lint規約がスキルに含まれていること により、テスト可能なコードを生成できるようにしたい。

#### Acceptance Criteria
1. The スキル shall `lua_test`フレームワーク（`describe`, `test`, `expect`）の基本使用パターンを説明する
2. The スキル shall テストファイル命名規約（`*_test.lua` / `*_spec.lua`）を説明する
3. The スキル shall テストスイート登録パターン（`init.lua` での `specs` テーブル登録）を説明する
4. The スキル shall `set_scene_selector` / `set_word_selector` によるテスト用決定論的選択の使用例を提供する


