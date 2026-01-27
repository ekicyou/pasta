# Requirements Document

## Introduction

pasta_luaクレートにおいて、Rust側からLua VMに公開されている関数・モジュール群のAPIリファレンスドキュメントを作成する。これにより、Luaスクリプト開発者がPasta標準ライブラリを効率的に利用できるようにする。

## Project Description (Input)

pasta_luaで、rust側からluaに公開されている関数群の説明ドキュメントが欲しい。

## Requirements

### Requirement 1: Luaモジュールカタログ

**Objective:** スクリプト開発者として、Rust側からLuaに公開されているすべてのモジュールを一覧で把握したい。どのモジュールがどの目的で使えるか理解できるように。

#### Acceptance Criteria

1. The documentation shall list all Lua modules registered by pasta_lua (`@pasta_search`, `@pasta_config`, `@pasta_persistence`, `@enc`)
2. When モジュール一覧を参照するとき, the documentation shall display module name, description, and version for each module
3. The documentation shall categorize modules by functionality (検索系、設定系、永続化系、エンコーディング系)

### Requirement 2: @pasta_search モジュールドキュメント

**Objective:** スクリプト開発者として、シーン検索・単語検索APIの使い方を理解したい。適切な検索クエリを構築できるように。

#### Acceptance Criteria

1. The documentation shall document `search_scene(name, global_scene_name?)` method with parameters, return values, and examples
2. The documentation shall document `search_word(name, global_scene_name?)` method with parameters, return values, and examples
3. The documentation shall document `set_scene_selector(...)` and `set_word_selector(...)` methods for deterministic testing
4. Where fallback search strategy is used, the documentation shall explain the local → global search order

### Requirement 3: @pasta_persistence モジュールドキュメント

**Objective:** スクリプト開発者として、永続化APIを使ってセーブデータを管理したい。データの保存・読み込みを確実に行えるように。

#### Acceptance Criteria

1. The documentation shall document `load()` function with return type and error handling behavior
2. The documentation shall document `save(data)` function with parameter type, return values, and error conditions
3. The documentation shall explain gzip compression (obfuscation) option and its configuration via pasta.toml
4. If persistence file is not found, the documentation shall clarify that `load()` returns an empty table

### Requirement 4: @enc モジュールドキュメント

**Objective:** スクリプト開発者として、エンコーディング変換APIを使って文字コード変換を行いたい。Windows環境でのファイルパス処理を正しく行えるように。

#### Acceptance Criteria

1. The documentation shall document `to_ansi(utf8_str)` function with parameters, return format, and error cases
2. The documentation shall document `to_utf8(ansi_str)` function with parameters, return format, and error cases
3. The documentation shall explain the tuple return format `(result, error)` pattern
4. The documentation shall include practical examples for Windows file path handling

### Requirement 5: @pasta_config モジュールドキュメント

**Objective:** スクリプト開発者として、pasta.tomlのカスタムフィールドにLuaからアクセスしたい。設定値を動的に参照できるように。

#### Acceptance Criteria

1. The documentation shall explain that `@pasta_config` is a read-only Lua table
2. The documentation shall document how custom fields from pasta.toml are exposed
3. The documentation shall include examples of accessing nested configuration values

### Requirement 6: finalize_scene 関数ドキュメント

**Objective:** 上級開発者として、finalize_sceneの内部動作を理解したい。カスタムローダーやテスト環境の構築時に活用できるように。

#### Acceptance Criteria

1. The documentation shall document `pasta.finalize_scene()` function's purpose and timing
2. The documentation shall explain scene collection from `pasta.scene` registry
3. The documentation shall explain word collection from `pasta.word` registry
4. The documentation shall clarify that this function constructs and registers `@pasta_search` module

### Requirement 7: mlua-stdlib 統合モジュールドキュメント

**Objective:** スクリプト開発者として、mlua-stdlibから提供される追加モジュールを把握したい。テスト・正規表現・JSON/YAML処理を活用できるように。

#### Acceptance Criteria

1. The documentation shall list mlua-stdlib modules enabled by default (`@assertions`, `@testing`, `@regex`, `@json`, `@yaml`)
2. The documentation shall note that `@env` module is disabled by default for security reasons
3. Where RuntimeConfig is customized, the documentation shall explain how to enable/disable individual modules
4. The documentation shall provide links to mlua-stdlib documentation for detailed API reference

### Requirement 8: ドキュメント形式・配置

**Objective:** プロジェクト管理者として、ドキュメントを適切な場所に配置したい。既存のドキュメント体系と整合させるように。

#### Acceptance Criteria

1. The documentation shall be placed in `crates/pasta_lua/` directory as a Markdown file
2. The documentation shall follow the project's existing documentation style (Japanese, Markdown)
3. The documentation shall include a table of contents for navigation
4. The documentation shall be referenced from `crates/pasta_lua/README.md`
