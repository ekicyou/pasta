# Requirements Document

## Project Description (Input)
pasta_luaのtomlコンフィグにて、[lua]のオプションとして、各標準luaライブラリの使用をするかどうかのフィーチャー指定が出来る機能を追加。

参考：
- https://docs.rs/mlua/latest/mlua/struct.Lua.html#method.new_with
- https://docs.rs/mlua/latest/mlua/struct.StdLib.html

指定しない場合、StdLib::ALL_SAFE 相当とする。unsafeオプションも指定可能。また、lua55で指定できるオプションを指定可とする。

## Introduction

本機能は、pasta_luaにおけるLua VMの標準ライブラリ読み込みをTOML設定ファイルにより制御可能にする。これにより、セキュリティ要件やパフォーマンス最適化の観点から、必要な標準ライブラリのみを有効化できる柔軟性を提供する。

## Requirements

### Requirement 1: TOML設定によるLua標準ライブラリの制御

**Objective:** ゴースト開発者として、TOML設定ファイルでLua標準ライブラリの有効/無効を個別に制御したい。これにより、セキュリティ要件に応じた最小限のライブラリのみを有効化でき、安全なスクリプト実行環境を構築できる。

#### Acceptance Criteria

1. When `[lua.stdlib]`セクションがTOML設定ファイルに存在する場合, the pasta_lua Runtime shall 指定されたライブラリ設定に基づいてLua VMを初期化する
2. When `[lua.stdlib]`セクションが存在しない場合, the pasta_lua Runtime shall `StdLib::ALL_SAFE`相当（安全なライブラリ一式）を有効化する
3. The pasta_lua Runtime shall 以下の個別ライブラリオプションをサポートする:
   - `coroutine` (bool) - コルーチンライブラリ
   - `table` (bool) - テーブル操作ライブラリ
   - `io` (bool) - I/Oライブラリ
   - `os` (bool) - OSライブラリ
   - `string` (bool) - 文字列ライブラリ
   - `utf8` (bool) - UTF-8ライブラリ
   - `math` (bool) - 数学ライブラリ
   - `package` (bool) - パッケージ/モジュールライブラリ
   - `debug` (bool) - デバッグライブラリ（unsafe）

### Requirement 2: プリセット設定のサポート

**Objective:** ゴースト開発者として、プリセット設定を使用して一般的なライブラリセットを簡単に選択したい。これにより、個別設定の手間を省き、推奨構成を素早く適用できる。

#### Acceptance Criteria

1. When `preset = "all_safe"`が指定された場合, the pasta_lua Runtime shall `StdLib::ALL_SAFE`相当のライブラリセットを有効化する
2. When `preset = "all"`が指定された場合, the pasta_lua Runtime shall すべての標準ライブラリ（unsafeを含む）を有効化する
3. When `preset = "none"`が指定された場合, the pasta_lua Runtime shall 標準ライブラリを一切読み込まない
4. When `preset = "minimal"`が指定された場合, the pasta_lua Runtime shall 基本ライブラリ（string, table, math）のみを有効化する
5. While プリセットが指定されている場合, when 個別ライブラリオプションも指定されている場合, the pasta_lua Runtime shall プリセットをベースに個別オプションで上書きする

### Requirement 3: debugライブラリの明示的有効化

**Objective:** ゴースト開発者として、デバッグ目的でdebugライブラリを有効化したい。ただし、セキュリティリスクを理解した上で明示的に許可する必要がある。

#### Acceptance Criteria

1. When `debug = false`または未指定の場合, the pasta_lua Runtime shall debugライブラリを無効化する（`ALL_SAFE`相当）
2. When `debug = true`が指定された場合, the pasta_lua Runtime shall debugライブラリを有効化する
3. When debugライブラリが有効化された場合, the pasta_lua Config shall 警告ログを出力する（tracing::warn）

**注記**: mluaにおいて、`StdLib::ALL_SAFE`はdebug以外のすべての標準ライブラリを含む。io, os, packageはsafeライブラリとして扱われる。

### Requirement 4: 設定バリデーションとエラーハンドリング

**Objective:** ゴースト開発者として、設定ミスを早期に検出したい。これにより、実行時エラーを防ぎ、デバッグ効率を向上できる。

#### Acceptance Criteria

1. If 認識されないライブラリ名が指定された場合, the pasta_lua Config shall 設定読み込み時にエラーを返す
2. If プリセット名が不正な場合, the pasta_lua Config shall 設定読み込み時にエラーを返す
3. When 設定ファイルが正常に読み込まれた場合, the pasta_lua Config shall 有効化されるライブラリ一覧をログ出力する（tracing::debug）
4. The pasta_lua Config shall `LuaStdLibConfig`構造体として設定を保持し、`StdLib`フラグへの変換メソッドを提供する

### Requirement 5: 既存コードとの互換性

**Objective:** 既存のpasta_luaユーザーとして、設定なしでも従来通り動作してほしい。これにより、既存プロジェクトの移行コストを最小化できる。

#### Acceptance Criteria

1. The pasta_lua Runtime shall `[lua.stdlib]`セクションが存在しない場合、既存の動作（`ALL_SAFE`相当）を維持する
2. The pasta_lua Config shall 既存の`TranspilerConfig`とは独立した設定項目として`LuaStdLibConfig`を管理する
3. The pasta_lua Runtime shall `LuaRuntimeConfig`を新設し、ランタイム初期化に必要な全設定を統合管理する

## TOML設定例

```toml
[lua.stdlib]
# プリセット選択（省略時: "all_safe"）
preset = "all_safe"

# 個別ライブラリオプション（プリセットを上書き）
# coroutine = true   # コルーチン
# table = true       # テーブル操作
# io = true          # I/O
# os = true          # OS操作
# string = true      # 文字列操作
# utf8 = true        # UTF-8
# math = true        # 数学関数
# package = true     # パッケージ
# debug = false      # デバッグ（唯一のunsafeライブラリ、デフォルト: false）
```

## 技術的制約

- mlua 0.11の`StdLib`フラグを使用
- Lua 5.5（`lua55`フィーチャー）で利用可能なオプションのみサポート
- 設定解析には既存の`toml`クレートを使用
- `StdLib::ALL_SAFE` = 全ライブラリ - debug（io, os, packageはsafeとして含まれる）
