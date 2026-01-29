# Requirements Document

## Project Description (Input)
pasta_luaのtomlコンフィグにて、`[lua]`セクションの`libs`フィールドにCargo風の配列記法でLua標準ライブラリ及びmlua-stdlibモジュールの使用を統合制御できる機能を追加。

参考：
- https://docs.rs/mlua/latest/mlua/struct.Lua.html#method.new_with
- https://docs.rs/mlua/latest/mlua/struct.StdLib.html

```toml
[lua]
# Lua標準ライブラリは std_ プレフィックス
libs = ["std_all"]  # デフォルト（StdLib::ALL_SAFE相当）

# mlua-stdlibモジュールも同じ配列で指定
libs = ["std_all", "testing", "regex", "json"]

# 個別指定
libs = ["std_string", "std_table", "std_math", "testing"]

# 減算記法
libs = ["std_all", "testing", "regex", "-env"]
```

指定しない場合、デフォルト値を適用する。`std_all_unsafe`でdebugを含むすべてのLua標準ライブラリを有効化可能。lua55で指定できるオプション及びmlua-stdlib v0.1のモジュールを配列要素として指定可能とする。

## Introduction

本機能は、pasta_luaにおけるLua VMの標準ライブラリ読み込みをTOML設定ファイルにより制御可能にする。これにより、セキュリティ要件やパフォーマンス最適化の観点から、必要な標準ライブラリのみを有効化できる柔軟性を提供する。

## Requirements

### Requirement 1: Cargo風配列記法による統合ライブラリ制御

**Objective:** ゴースト開発者として、Cargo.tomlのfeatures記法と同様の配列形式でLua標準ライブラリとmlua-stdlibモジュールを統合して制御したい。これにより、直感的で簡潔な設定が可能になり、すべてのライブラリ構成を一箇所で管理できる。

#### Acceptance Criteria

1. When `[lua]`セクションの`libs`フィールドが存在する場合, the pasta_lua Runtime shall 指定された配列要素に基づいてLua VMとmlua-stdlibモジュールを初期化する
2. When `libs`フィールドが省略された場合, the pasta_lua Runtime shall デフォルト値を適用する
3. The pasta_lua Runtime shall Lua標準ライブラリとして以下の`std_`プレフィックス付き要素をサポートする:
   - `std_all` - 安全なLua標準ライブラリすべて（`StdLib::ALL_SAFE`、std_debug除く）
   - `std_all_unsafe` - std_debugを含むすべてのLua標準ライブラリ（`StdLib::ALL`）
   - `std_coroutine` - コルーチンライブラリ
   - `std_table` - テーブル操作ライブラリ
   - `std_io` - I/Oライブラリ
   - `std_os` - OSライブラリ
   - `std_string` - 文字列ライブラリ
   - `std_utf8` - UTF-8ライブラリ
   - `std_math` - 数学ライブラリ
   - `std_package` - パッケージ/モジュールライブラリ
   - `std_debug` - デバッグライブラリ（unsafe）
4. The pasta_lua Runtime shall mlua-stdlibモジュールとして以下の要素をサポートする:
   - `assertions` - @assertions（アサーション・検証機能）
   - `testing` - @testing（テストフレームワーク）
   - `env` - @env（環境変数・ファイルシステムアクセス、セキュリティ考慮）
   - `regex` - @regex（正規表現）
   - `json` - @json（JSON エンコード・デコード）
   - `yaml` - @yaml（YAML エンコード・デコード）
5. The pasta_lua Runtime shall 複数のライブラリ指定をOR結合（Lua標準）またはモジュール登録（mlua-stdlib）で処理する
6. The pasta_lua Runtime shall 空配列`[]`を最小構成（ライブラリなし）として扱う

### Requirement 2: 減算記法によるライブラリ除外

**Objective:** ゴースト開発者として、特定のライブラリ・モジュールを除外する減算記法を使用したい。これにより、「ほぼすべて有効だが特定のものだけ無効化」といった設定を簡潔に記述できる。

#### Acceptance Criteria

1. When 配列要素が`"-"`で始まる場合, the pasta_lua Runtime shall 該当ライブラリ・モジュールを除外する
2. The pasta_lua Runtime shall Lua標準ライブラリの減算記法として以下をサポートする:
   - `"-std_debug"` - std_debugライブラリを除外
   - `"-std_io"`, `"-std_os"`, `"-std_package"` 等、すべてのstd_系に対応
3. The pasta_lua Runtime shall mlua-stdlibモジュールの減算記法として以下をサポートする:
   - `"-env"`, `"-testing"`, `"-regex"` 等、すべてのモジュールに対応
4. The pasta_lua Runtime shall 加算要素を先に処理し、その後減算要素を処理する
5. If `["std_all", "-std_debug"]`のように指定された場合, the pasta_lua Runtime shall `StdLib::ALL_SAFE`と同等の結果を生成する

### Requirement 3: セキュリティ関連ライブラリ有効化時の警告

**Objective:** ゴースト開発者として、セキュリティリスクのあるライブラリ・モジュールが有効化された際に警告を受け取りたい。これにより、意図しないセキュリティリスクを認識できる。

#### Acceptance Criteria

1. When `"std_debug"`または`"std_all_unsafe"`が配列に含まれる場合, the pasta_lua Config shall 警告ログを出力する（tracing::warn: "std_debug library enabled - potential security risk"）
2. When `"env"`が配列に含まれる場合, the pasta_lua Config shall 警告ログを出力する（tracing::warn: "env module enabled - filesystem and environment access permitted"）
3. When `["std_all", "-std_debug"]`のようにstd_debugが明示的に除外された場合, the pasta_lua Config shall std_debug関連の警告を出力しない
4. The pasta_lua Config shall 有効化されるライブラリ・モジュール一覧をデバッグログに出力する（tracing::debug）

**注記**: 
- mluaにおいて、`StdLib::ALL_SAFE`はstd_debug以外のすべてのLua標準ライブラリを含む
- mlua-stdlibの`env`モジュールはファイルシステム・環境変数アクセスを提供するためセキュリティ考慮が必要

### Requirement 4: 設定バリデーションとエラーハンドリング

**Objective:** ゴースト開発者として、設定ミスを早期に検出したい。これにより、実行時エラーを防ぎ、デバッグ効率を向上できる。

#### Acceptance Criteria

1. If 認識されないライブラリ名が指定された場合, the pasta_lua Config shall 設定読み込み時にエラーを返す
2. If プリセット名が不正な場合, the pasta_lua Config shall 設定読み込み時にエラーを返す
3. When 設定ファイルが正常に読み込まれた場合, the pasta_lua Config shall 有効化されるライブラリ一覧をログ出力する（tracing::debug）
4. The pasta_lua Config shall `LuaStdLibConfig`構造体として設定を保持し、`StdLib`フラグへの変換メソッドを提供する

### Requirement 4: 設定バリデーションとエラーハンドリング

**Objective:** ゴースト開発者として、設定ミスを早期に検出したい。これにより、実行時エラーを防ぎ、デバッグ効率を向上できる。

#### Acceptance Criteria

1. If 認識されないライブラリ名が配列に含まれる場合, the pasta_lua Config shall 設定読み込み時にエラーを返す
2. If `"all"`と`"all_unsafe"`が同時に指定された場合, the pasta_lua Config shall `"all_unsafe"`を優先する（またはエラー）
3. When 矛盾する設定（例: `["coroutine", "-coroutine"]`）が指定された場合, the pasta_lua Config shall 加算を先に処理し、減算を後に処理する（結果: 無効化）
4. The pasta_lua Config shall `LuaStdLibConfig`構造体として設定を保持し、`to_stdlib() -> StdLib`変換メソッドを提供する

### Requirement 5: 既存コードとの後方互換性

**Objective:** 既存のpasta_luaユーザーとして、設定なしでも従来通り動作してほしい。これにより、既存プロジェクトの移行コストを最小化できる。

#### Acceptance Criteria

1. The pasta_lua Runtime shall `[lua]`セクションまたは`libs`フィールドが存在しない場合、デフォルト値を適用する（既存動作維持）
2. The pasta_lua Config shall 既存の`TranspilerConfig`とは独立した設定項目として`LuaLibConfig`を管理する
3. The pasta_lua Runtime shall 既存の`RuntimeConfig`の個別フラグ（`enable_std_libs`, `enable_testing`等）を`libs`配列で置き換え、段階的に非推奨化する

**デフォルト値の定義**:
```toml
[lua]
# 省略時のデフォルト
libs = ["std_all", "assertions", "testing", "regex", "json", "yaml"]
# 注: env はセキュリティ上の理由からデフォルトでは無効
```

## TOML設定例

### 例1: デフォルト構成（省略時）
```toml
# [lua]セクション自体を省略可能
# デフォルト: ["std_all", "assertions", "testing", "regex", "json", "yaml"]
```

### 例2: すべて有効化（std_debug含む、env含む）
```toml
[lua]
libs = ["std_all_unsafe", "assertions", "testing", "env", "regex", "json", "yaml"]
```

### 例3: 最小構成（ライブラリなし）
```toml
[lua]
libs = []
```

### 例4: 個別指定（必要最小限）
```toml
[lua]
libs = ["std_string", "std_table", "std_math"]
```

### 例5: テスト開発向け（std_debugとtesting）
```toml
[lua]
libs = ["std_all_unsafe", "assertions", "testing", "regex"]
```

### 例6: セキュアな本番環境（std_debugとenv除外）
```toml
[lua]
libs = ["std_all", "assertions", "regex", "json", "yaml"]
# デフォルトと同等（testingのみ追加で除外する場合は "-testing" を追加）
```

### 例7: 減算記法の使用
```toml
[lua]
libs = ["std_all", "assertions", "testing", "regex", "json", "yaml", "-testing", "-env"]
# デフォルトからtestingとenvを除外
```

### 例8: std_all_unsafeからstd_debugのみ除外
```toml
[lua]
libs = ["std_all_unsafe", "-std_debug", "assertions", "testing", "regex", "json"]
# std_all_unsafeは不要。std_allと同等になる
```

## 技術的制約

- mlua 0.11の`StdLib`フラグを使用（Lua標準ライブラリ）
- mlua-stdlib 0.1のモジュール登録APIを使用
- Lua 5.5（`lua55`フィーチャー）で利用可能なオプションのみサポート
- 設定解析には既存の`toml`クレートを使用
- `StdLib::ALL_SAFE` = 全Lua標準ライブラリ - std_debug（std_io, std_os, std_packageはsafeとして含まれる）
- mlua-stdlibの`@assertions`, `@testing`, `@regex`, `@json`, `@yaml`, `@env`はLua VMに動的に登録
