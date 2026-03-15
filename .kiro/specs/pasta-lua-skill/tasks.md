# Implementation Plan

- [ ] 1. SKILL.mdファイルの骨格とメタデータ・目的セクションを準備する
- [ ] 1.1 YAML Frontmatterを作成してスキルのトリガーフレーズを定義する
  - `.agents/skills/pasta-lua-coding/` ディレクトリを作成してSKILL.mdファイルを新規作成する
  - `name: pasta-lua-coding`、`description`（USE FOR / DO NOT USE FOR）、`metadata`（author, version）をスキル標準形式で記述する
  - USE FOR フレーズに：pasta lua, pasta_lua, Lua API, Luaスクリプト, scripts/, 単語辞書一括投入, WORD.create, イベントハンドラ, REG, RES, 永続化, @pasta_persistence, save, @pasta_search, @pasta_config, @pasta_sakura_script, @enc, ACT, SCENE, STORE, GLOBAL, SAVE, lua_test, luacheck, pasta lua coding, pasta runtime API を含める
  - DO NOT USE FOR フレーズに：Pasta DSL文法, .pastaファイル編集, pasta_dsl crate, pasta_core crate, Rustクレート実装, 汎用Luaプログラミング, SHIORIプロトコル実装 を含め、姉妹スキル`pasta-ghost-authoring`との重複がないことを確認する
  - _Requirements: 1.1, 1.2, 1.3_

- [ ] 1.2 §1 Purpose & Prerequisitesセクションを書く
  - スキルの目的（自然言語→pasta_lua準拠Luaコード変換サポート）と対象ドメイン（scripts/配下カスタムLuaスクリプトおよびDSL内Luaブロック）を明記する
  - 姉妹スキル`pasta-ghost-authoring`との役割分離（DSL層 vs Lua層）を明記する
  - `scripts/` フォルダの位置づけ（ゴーストディレクトリ直下、`main.lua`がエントリーポイント）を説明する
  - DSL vs Lua 判断基準表を記載する（数個の単語定義→DSL、数十〜数百件一括投入→Lua、複雑なロジック→Lua、カスタムSHIORIイベント→Luaなど）
  - 自己完結性宣言と権威的情報ソース（lua-coding.md, LUA_API.md）を明記する
  - _Requirements: 1.4, 1.5, 1.6, 1.7, 1.8_

- [ ] 2. §2 クイックリファレンスセクションを作成する
  - Rust組み込みモジュール（`@pasta_*` + `@enc`）のカタログ表を作成する（モジュール名・用途・requireパターン区分）
  - 内部Luaモジュール（`pasta.*`名前空間）のカタログ表を作成する（モジュール名・用途・主要API代表）
  - mlua-stdlib統合モジュール（`@json`, `@yaml`, `@regex`, `@assertions`, `@testing`）の一覧表を作成する（`@env`はデフォルト無効と注記）
  - セクション末尾にDSL→Luaブリッジ基本形コード例（`function SCENE.func_name(act)` / `act:init_scene(SCENE)` / `act:yield()`）を追加する
  - 情報ソース（crates/pasta_lua/LUA_API.md §1）を注記する
  - _Requirements: 1.2, 1.3, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_

- [ ] 3. §3 Luaコーディング規約セクションを作成する
- [ ] 3.1 命名規約・モジュール構造・循環参照回避パターンを書く
  - 命名規約表（snake_case変数・ローカル関数、UPPER_CASEモジュールテーブル・定数、`_`プレフィックスプライベート、`_IMPL`サフィックスクラス実装）を記述する
  - PascalCase禁止の禁止パターンをコード例付きで明記する
  - 日本語識別子の許可範囲（内部変数・GLOBAL登録はOK、公開API・モジュールテーブルはNG）を記述する
  - 標準モジュール構造テンプレート（require → モジュールテーブル → ローカル関数 → 公開関数 → return）をコード例付きで記述する
  - 循環参照回避パターン（STOREに共有データを集約し他モジュールがSTOREをrequire）をコード例付きで記述する
  - 情報ソース（steering/lua-coding.md §1-§2）を注記する
  - _Requirements: 2.1, 2.2, 2.5_

- [ ] 3.2 クラス設計パターン（MODULE/MODULE_IMPL分離・コンストラクタ・継承・禁止パターン）を書く
  - MODULE/MODULE_IMPL分離パターン：モジュールテーブル（公開API）とクラス実装メタテーブル（インスタンスメソッド）の分離をコード例付きで記述する
  - ドット構文によるメソッド定義（明示的self）とコロン構文による呼び出しの規約をテーブル形式で記述する
  - コンストラクタパターン（`setmetatable`でIMPLを設定）とシングルトンパターン（requireキャッシング活用）を簡潔なコード例で記述する
  - 継承パターン（`setmetatable` + `__index`チェーン・`MODULE.IMPL`公開）を要約説明のみで記述する
  - 禁止パターン（`MODULE.instance()`パターン・コロン構文でのメソッド定義）を明記する
  - 情報ソース（steering/lua-coding.md §3）を注記する
  - _Requirements: 2.3, 2.4_

- [ ] 3.3 EmmyLua型注釈とエラーハンドリング規約を書く
  - EmmyLuaアノテーション規約：`@module`（ファイル先頭）・`@class`（クラス定義直前）・公開関数への`@param`/`@return`・`@param ...`必須（`@vararg`禁止）をルール列挙形式で記述する
  - エラーハンドリング3パターンをルール箇条書きで列挙する（ガードクローズ：関数先頭での前提条件検証・早期リターン、pcall：外部関数とリスクのある操作に使用、nilチェック：明示的な条件確認）
  - サイレントnil返却禁止の禁止パターンを明記する
  - 情報ソース（steering/lua-coding.md §4-§5）を注記する
  - _Requirements: 2.6, 2.7, 2.8_

- [ ] 4. §4 ランタイムAPIリファレンスセクションを作成する
- [ ] 4.1 @pasta_search・@pasta_persistence・@pasta_configモジュールを書く
  - セクション冒頭にrequire直接 vs pcall保護の使い分けルールを記述する（常時利用可能モジュール: `@pasta_search`, `@pasta_persistence`, `@pasta_sakura_script`, `@enc` → require直接 / オプショナル: `@pasta_config` → pcall保護）
  - `@pasta_search`：`search_scene(name, global?) → global_name, local_name | nil`・`search_word(name, global?) → string | nil`・`set_scene_selector`・`set_word_selector`のシグネチャ、ローカル→グローバルのフォールバック検索戦略、最小使用例3行を記述する
  - `@pasta_persistence`：`load() → table`・`save(data) → true, nil | nil, error_message`のシグネチャ、pasta.toml `[persistence]`設定（obfuscate, file_path）、最小使用例4行を記述する
  - `@pasta_config`：読み取り専用テーブル・TOML構造保持・`[loader]`セクション除外・pcall経由requireが必須な理由、アクセス例2行を記述する
  - 情報ソース（crates/pasta_lua/LUA_API.md §2,§3,§5）を注記する
  - _Requirements: 3.1, 3.2, 3.3, 3.7, 3.8_

- [ ] 4.2 @pasta_sakura_script・@enc・mlua-stdlibモジュールを書く
  - `@pasta_sakura_script`：`talk_to_script(actor, talk) → string`のシグネチャ、actor.talkテーブルのフィールド一覧表（圧縮版）、最小使用例3行を記述する
  - `@enc`：`to_ansi(utf8_str) → ansi_string, nil | nil, error_message`・`to_utf8(ansi_str) → utf8_string, nil | nil, error_message`のシグネチャ、Windows環境のファイルパス処理用途、エラーハンドリングを含む最小使用例3行を記述する
  - mlua-stdlibモジュール：`@json`（json.encode/json.decode）・`@yaml`（yaml.encode/yaml.decode）・`@regex`（regex.new(pattern):find_all(s)）・`@assertions`/`@testing`（テスト用、§7で詳述）・`@env`（デフォルト無効・セキュリティ上の理由）の概要と基本使用例を記述する
  - 情報ソース（crates/pasta_lua/LUA_API.md §4,§6,§8）を注記する
  - _Requirements: 3.4, 3.5, 3.6, 3.8_

- [ ] 5. §5 pasta内部Luaモジュール解説セクションを作成する
- [ ] 5.1 STOREパターンとACTオブジェクトを書く
  - STOREパターン：`pasta.store`の一元データ管理と循環参照回避の役割を説明し、主要フィールド一覧（actors, scenes, global_words, local_words, actor_words, co_scene等）をテーブル形式で記述する。`STORE.reset()`（テスト・再初期化用）を記述する
  - ACTオブジェクト：シーン関数の引数(`function scene(act)`)としての位置づけを説明する
  - **`act:init_scene(SCENE)`の必須定型を特出しで記述する**（シーン関数は必ずこの呼び出しで始まり`save`（永続変数）と`var`（アクション内一時変数）を取得する旨をコード例2〜3行で示す）
  - ACTの主要メソッド一覧テーブル（talk, raw_script, surface, wait, newline, clear, word, call, yield, build）をシグネチャ+1行説明で記述する
  - ACTフィールド（actors, save, app_ctx, var, token, current_scene）を列挙する（reqはShioriAct専用として §6 で説明する旨を注記する）
  - 情報ソース（steering/lua-coding.md §6.3, §6.5）を注記する
  - _Requirements: 4.1, 4.2_

- [ ] 5.2 SCENE・WORD・GLOBAL・SAVE・finalize_sceneを書く
  - SCENEモジュール：`SCENE.create_scene(base_name, local_name?, scene_func?)`（カウンタ自動採番）・`SCENE.search(name, global_scene_name?, attrs?)`・`SCENE.co_exec(name, global_scene_name?, attrs?)`の3引数シグネチャを記述する。DSL→Luaブリッジとして`function SCENE.func(act) ... end`パターンを説明する
  - WORDモジュール：`WORD.create_global(key)`・`WORD.create_local(scene_name, key)`（2引数）・`WORD.create_actor(actor_name, key)`（2引数）の各シグネチャ、`PASTA.create_word(key)`エイリアス、`builder:entry(...)`メソッドチェーンを記述する。大量投入のループパターン使用例5行を含める
  - GLOBALモジュール：`pasta.global`への`GLOBAL.関数名 = function(act) ... end`登録パターンとDSLからの`＠関数名()`呼び出しを説明する
  - SAVEモジュール：`pasta.save`の永続化データアクセスとACT経由パターン`local save, var = act:init_scene(SCENE)`を説明する。直接requireパターンも記述する
  - `finalize_scene()`：`require("pasta").finalize_scene()`のscene_dic.lua末尾での自動呼び出しと`@pasta_search`モジュール構築への役割を説明する
  - 情報ソース（steering/lua-coding.md §6.1-§6.6 および crates/pasta_lua/LUA_API.md §7）を注記する
  - _Requirements: 4.3, 4.4, 4.5, 4.6, 4.7, 4.8_

- [ ] 6. §6 SHIORIイベントハンドラセクションを作成する
- [ ] 6.1 REGテーブル登録パターンとRESレスポンス生成を書く
  - REGテーブル：`local REG = require("pasta.shiori.event.register")`、`REG.EventName = function(req) ... end`登録パターンを記述する。reqパラメータ（req.id, req.reference[], req.date等のSHIORIリクエスト情報）を説明する。OnBoot登録の最小使用例3行を記述する
  - RESモジュール：`local RES = require("pasta.shiori.res")`、`ok(value)`・`ok_with(headers)`・`no_content()`・`err(message)`の関数一覧テーブルと対応するHTTPステータスコード（200 OK / 204 No Content / 500 Internal Server Error）を記述する
  - 情報ソース（crates/pasta_lua/LUA_API.md §9.2, §9.4）を注記する
  - _Requirements: 5.1, 5.2_

- [ ] 6.2 主要SHIORIイベント一覧・シーン関数フォールバック・仮想ディスパッチャを書く
  - 主要SHIORIイベント表（OnBoot, OnFirstBoot, OnClose, OnGhostChanged, OnMouseDoubleClick, OnSecondChange, OnMinuteChange × req.reference[N]パラメータ説明）を作成する。OnFirstBootの統合使用例5行を含める
  - シーン関数フォールバック：REG未登録時に`SCENE.search`でグローバルシーンを検索する動作、DSLの`＊OnBoot`等が自動的に呼び出される仕組みを説明する
  - 仮想ディスパッチャ：`pasta.shiori.event.virtual_dispatcher`モジュールのOnTalk/OnHour自動発行メカニズム（OnSecondChangeをトリガーとして内部ディスパッチ）、pasta.toml `[ghost]`セクション設定（talk_interval_min, talk_interval_max, hour_margin）を説明する
  - 情報ソース（crates/pasta_lua/LUA_API.md §9.3, §9.5, §9.8）を注記する
  - _Requirements: 5.3, 5.4, 5.5_

- [ ] 7. §7 テスト・Lint規約セクションを作成して最終確認する
- [ ] 7.1 lua_testフレームワーク・テストファイル規約・決定論的テストを書く
  - lua_testフレームワーク：`describe`・`test`・`expect`のrequireパターン（`require("lua_test.test").describe`等）とBDD風テスト構造テンプレート8行、`toBe`・`not_:toBe`等のマッチャーを記述する
  - テストファイル規約：`*_test.lua`/`*_spec.lua`命名パターンと配置ディレクトリを説明する
  - init.luaパターン：`specs`テーブルへのテストスイート登録とpcall実行パターンをコード例付きで記述する
  - 決定論的テスト：`set_scene_selector(...)`/`set_word_selector(...)`によるランダム選択固定をテスト前に行う使用例4行を記述する
  - 情報ソース（steering/lua-coding.md §7）を注記する
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [ ] 7.2 luacheck設定を書いてSKILL.md全体の品質を最終確認する
  - luacheck設定：`.luacheckrc`のglobalsホワイトリスト（PASTA, ACTOR, SCENE, WORD, ACT, CTX, STORE, GLOBAL）・`allow_defined = true`（UTF-8/日本語識別子許可）・`unused_args = false`・`max_line_length = 120`の設定例を記述する。実行コマンド（`lua scriptlibs/luacheck/bin/luacheck.lua scripts/ --config .luacheckrc`）を記述する
  - SKILL.md全体の行数を確認し、目標555行（±10%で490〜610行）の範囲内に収まっているかを検証する。超過時は§3・§4を優先的に圧縮する
  - 全セクションの情報ソース注記（`（情報ソース: ファイルパス）`形式）が揃っているかを確認する
  - 全6要件（Req 1〜6）の40ACがSKILL.md内でカバーされていることを確認する
  - 姉妹スキル`pasta-ghost-authoring`とのUSE FOR/DO NOT USE FORフレーズの重複がないことを確認する
  - _Requirements: 1.5, 2.8, 3.8_
