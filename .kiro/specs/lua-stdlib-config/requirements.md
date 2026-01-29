# Requirements Document

## Project Description (Input)
pasta_luaのtomlコンフィグにて、`[lua]`セクションの`stdlib`フィールドにCargo風の配列記法で標準Luaライブラリの使用を制御できる機能を追加。

参考：
- https://docs.rs/mlua/latest/mlua/struct.Lua.html#method.new_with
- https://docs.rs/mlua/latest/mlua/struct.StdLib.html

```toml
[lua]
stdlib = ["all"]  # デフォルト（StdLib::ALL_SAFE相当）
# または
stdlib = ["all_unsafe"]  # debug含む全ライブラリ（StdLib::ALL相当）
# または
stdlib = ["string", "table", "math"]  # 個別指定
# または
stdlib = ["all", "-debug"]  # 減算記法
```

指定しない場合、`["all"]`（StdLib::ALL_SAFE相当）とする。`"all_unsafe"`でdebugを含むすべてのライブラリを有効化可能。また、lua55で指定できるオプションを配列要素として指定可能とする。

## Introduction

本機能は、pasta_luaにおけるLua VMの標準ライブラリ読み込みをTOML設定ファイルにより制御可能にする。これにより、セキュリティ要件やパフォーマンス最適化の観点から、必要な標準ライブラリのみを有効化できる柔軟性を提供する。

## Requirements

### Requirement 1: Cargo風配列記法によるLua標準ライブラリの制御

**Objective:** ゴースト開発者として、Cargo.tomlのfeatures記法と同様の配列形式でLua標準ライブラリを制御したい。これにより、直感的で簡潔な設定が可能になり、セキュリティ要件に応じた柔軟なライブラリ構成を実現できる。

#### Acceptance Criteria

1. When `[lua]`セクションの`stdlib`フィールドが存在する場合, the pasta_lua Runtime shall 指定された配列要素に基づいてLua VMを初期化する
2. When `stdlib`フィールドが省略された場合, the pasta_lua Runtime shall デフォルト値`["all"]`を適用する（`StdLib::ALL_SAFE`相当）
3. The pasta_lua Runtime shall 以下の配列要素をサポートする:
   - `"all"` - 安全なライブラリすべて（`StdLib::ALL_SAFE`、debugを除く）
   - `"all_unsafe"` - debugを含むすべてのライブラリ（`StdLib::ALL`）
   - `"coroutine"` - コルーチンライブラリ
   - `"table"` - テーブル操作ライブラリ
   - `"io"` - I/Oライブラリ
   - `"os"` - OSライブラリ
   - `"string"` - 文字列ライブラリ
   - `"utf8"` - UTF-8ライブラリ
   - `"math"` - 数学ライブラリ
   - `"package"` - パッケージ/モジュールライブラリ
   - `"debug"` - デバッグライブラリ（unsafe）
4. The pasta_lua Runtime shall 複数の個別ライブラリ指定をOR結合（ビット演算`|`）で処理する
5. The pasta_lua Runtime shall 空配列`[]`を`StdLib::NONE`（ライブラリなし）として扱う

### Requirement 2: 減算記法によるライブラリ除外

**Objective:** ゴースト開発者として、特定のライブラリを除外する減算記法を使用したい。これにより、「ほぼすべて有効だが特定のライブラリのみ無効化」といった設定を簡潔に記述できる。

#### Acceptance Criteria

1. When 配列要素が`"-"`で始まる場合, the pasta_lua Runtime shall 該当ライブラリをビット演算`& !(StdLib::XXX)`で除外する
2. The pasta_lua Runtime shall 減算記法として以下をサポートする:
   - `"-debug"` - debugライブラリを除外
   - `"-io"`, `"-os"`, `"-package"` 等、すべてのライブラリに対応
3. The pasta_lua Runtime shall 加算要素を先に処理し、その後減算要素を処理する
4. If `["all", "-debug"]`のように指定された場合, the pasta_lua Runtime shall `StdLib::ALL_SAFE`と同等の結果を生成する

### Requirement 3: debugライブラリ有効化時の警告

**Objective:** ゴースト開発者として、unsafeなdebugライブラリが有効化された際に警告を受け取りたい。これにより、意図しないセキュリティリスクを認識できる。

#### Acceptance Criteria

1. When `"debug"`または`"all_unsafe"`が配列に含まれる場合, the pasta_lua Config shall 警告ログを出力する（tracing::warn: "Debug library enabled - potential security risk"）
2. When `["all", "-debug"]`のようにdebugが明示的に除外された場合, the pasta_lua Config shall 警告を出力しない
3. The pasta_lua Config shall 有効化されるライブラリ一覧をデバッグログに出力する（tracing::debug）

**注記**: mluaにおいて、`StdLib::ALL_SAFE`はdebug以外のすべての標準ライブラリを含む。io, os, packageはsafeライブラリとして扱われる。

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

1. The pasta_lua Runtime shall `[lua]`セクションまたは`stdlib`フィールドが存在しない場合、デフォルト値`["all"]`を適用する（既存動作維持）
2. The pasta_lua Config shall 既存の`TranspilerConfig`とは独立した設定項目として`LuaStdLibConfig`を管理する
3. The pasta_lua Runtime shall 既存の`RuntimeConfig::enable_std_libs`フラグとの整合性を保つ（非推奨化を検討）

## TOML設定例

### パターン1: デフォルト（安全なライブラリすべて）
```toml
[lua]
# 省略時のデフォルト: stdlib = ["all"]
```

### パターン2: 明示的にデフォルト指定
```toml
[lua]
stdlib = ["all"]  # StdLib::ALL_SAFE相当（debugを除く全ライブラリ）
```

### パターン3: 最小構成
```toml
[lua]
stdlib = ["string", "table", "math"]
```

### パターン4: デバッグ有効化（全ライブラリ）
```toml
[lua]
stdlib = ["all_unsafe"]  # StdLib::ALL相当（debug含む）
```

### パターン5: 個別ライブラリの除外（減算記法）
```toml
[lua]
stdlib = ["all", "-os", "-io"]  # OS・IO以外のすべて
```

### パターン6: ライブラリなし
```toml
[lua]
stdlib = []  # StdLib::NONE相当
```

### パターン7: 特定ライブラリの追加
```toml
[lua]
stdlib = ["string", "table", "math", "coroutine", "utf8"]
```

## 技術的制約

- mlua 0.11の`StdLib`フラグを使用
- Lua 5.5（`lua55`フィーチャー）で利用可能なオプションのみサポート
- 設定解析には既存の`toml`クレートを使用
- `StdLib::ALL_SAFE` = 全ライブラリ - debug（io, os, packageはsafeとして含まれる）
