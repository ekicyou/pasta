# Requirements Document

## Project Description (Input)
pasta_luaの起動シーケンスにおけるWindows環境でのパス解決問題の修正。mlua付属のluaのrequireパッケージ解決に絶対パスを与えているが、LuaのFilePath解決はUTF-8ではなくANSIであるため、Windows環境下で正しく動作するように設計を見直す必要がある。既存のpasta-lua-startup-sequence仕様で実装された起動シーケンスに存在する問題への対応。

## Introduction
本仕様は、pasta_luaのWindows環境におけるファイルパスエンコーディング問題を解決する。現在の実装では`package.path`にUTF-8文字列を設定しているが、Luaの標準ファイルI/O（`fopen`）はANSIエンコーディング（システムロケール依存）を使用するため、日本語などの非ASCII文字を含むパスで`require`が失敗する。

**重要な発見**:
- Lua文字列は任意のバイト列を格納可能（UTF-8である必要なし）
- `package.path`にANSIエンコードされたバイト列を設定すれば、標準サーチャーがそのまま動作
- **カスタムサーチャー実装は不要** - 既存の`encoding`モジュールを活用し、`package.path`設定を修正するだけで解決可能

**背景**:
- mluaが組み込むLua VMは、ファイルパス解決に標準Cライブラリ（`fopen`等）を使用
- Windows環境では、`fopen`はANSIコードページ（日本語環境ではShift_JIS/CP932）でパスを解釈
- 現在の`encoding::path_to_lua`は`String::from_utf8_lossy`を使用しているが、これがANSIバイト列を破壊

## 設計判断事項

本仕様の実装にあたり、以下の設計判断が必要です。ギャップ分析に基づく推奨案を示します。

### 判断1: encoding/mod.rsの関数設計

**選択肢**:
- A: `path_to_lua_bytes`新規追加、既存`path_to_lua`維持（**推奨**）
- B: `path_to_lua`をバイト列返却に変更（破壊的変更）

**推奨理由**: 後方互換性維持（Requirement 8）

**決定**: (設計フェーズで確定)

### 判断2: @encモジュールの関数セット

**必須**: `to_ansi(utf8_str)`, `to_utf8(ansi_str)`  
**オプション**: `to_oem(utf8_str)`, `from_oem(oem_str)`（コンソールI/O用）

**推奨**: 必須のみ実装（**推奨**）  
**推奨理由**: 要件スコープ最小化、OEMは将来追加可能

**決定**: (設計フェーズで確定)

### 判断3: テスト戦略

**Windows専用テスト**: 日本語パスでのモジュールロードテスト  
**Unix環境**: パススルー動作確認のみ

**CI考慮事項**: Windows環境でのみ日本語パステスト実行

**決定**: (設計フェーズで確定)

---

## Requirements

### Requirement 1: package.pathへのANSIバイト列設定
**Objective:** As a ゴースト開発者, I want 日本語パスを含むディレクトリでもLuaモジュールが正しく読み込まれること, so that Windows環境でパス制限なくスクリプトを配置できる

#### Acceptance Criteria
1. When `package.path`が設定される時, the PastaLuaRuntime shall ANSIエンコードされたバイト列をLua文字列として設定する
2. When Windows環境で動作する場合, the パス文字列 shall UTF-8からANSI（Shift_JIS/CP932等）に変換される
3. When 非Windows環境で動作する場合, the パス文字列 shall UTF-8のまま設定される
4. The Lua標準サーチャー shall 設定されたANSIバイト列パスを使用してファイルを解決する
5. The `require`関数 shall 日本語パスを含むモジュールを正しくロードできる

### Requirement 2: encoding/mod.rsのpath_to_lua修正
**Objective:** As a 開発者, I want 既存のpath_to_lua関数がバイト列を正しく扱うこと, so that Windows環境でのパス変換が正確に動作する

#### Acceptance Criteria
1. The `path_to_lua` shall `String`ではなく`Vec<u8>`を返す（または新関数`path_to_lua_bytes`を追加）
2. When Windows環境で呼ばれた場合, the 関数 shall `Encoding::ANSI.to_bytes(path)`の結果をそのまま返す
3. The 関数 shall `String::from_utf8_lossy`を使用しない（ANSIバイト列が破壊される）
4. When 非Windows環境で呼ばれた場合, the 関数 shall UTF-8バイト列を返す
5. If エンコーディング変換が失敗した場合, the 関数 shall `std::io::Error`を返す

### Requirement 3: LoaderContextのバイト列生成対応
**Objective:** As a PastaLoader, I want LoaderContextが`package.path`用のバイト列を生成できること, so that ANSIエンコードされたパスが正しく設定される

#### Acceptance Criteria
1. The LoaderContext shall `generate_package_path_bytes() -> Result<Vec<u8>>`メソッドを提供する
2. When メソッドが呼ばれた時, the LoaderContext shall 検索パス文字列を結合してから`path_to_lua_bytes`で変換する
3. The 生成されたバイト列 shall セミコロン区切りの`?.lua`および`?/init.lua`パターンを含む
4. The LoaderContext shall 既存の`generate_package_path()`メソッドを維持する（後方互換性）

### Requirement 4: runtime/mod.rsのpackage.path設定修正
**Objective:** As a PastaLuaRuntime, I want `package.path`にバイト列を正しく設定できること, so that Lua VMがANSIエンコードされたパスを使用する

#### Acceptance Criteria
1. When `setup_package_path`が呼ばれた時, the Runtime shall `LoaderContext::generate_package_path_bytes()`を使用する
2. The Runtime shall `lua.create_string(&bytes)`でバイト列をLua文字列に変換する
3. The Runtime shall `package.set("path", lua_string)`でpackage.pathを設定する
4. If バイト列生成が失敗した場合, the Runtime shall エラーをLuaResultに変換して返す
5. While tracingが有効な場合, the Runtime shall 設定されたパスをdebugレベルでログ出力する（UTF-8として解釈可能な範囲）

### Requirement 5: @encモジュールによるエンコーディング変換API
**Objective:** As a Luaスクリプト開発者, I want Lua側でUTF-8とANSI間の文字列変換ができること, so that 外部ファイル操作やWindows APIとの連携が容易になる

#### Acceptance Criteria
1. When PastaLuaRuntimeが初期化される時, the Runtime shall `@enc`モジュールを`package.loaded`に登録する
2. The `@enc`モジュール shall `to_ansi(utf8_string)`関数を提供する（入力: UTF-8エンコードされたLua文字列、出力: ANSIエンコードされたLua文字列）
3. The `@enc`モジュール shall `to_utf8(ansi_string)`関数を提供する（入力: ANSIエンコードされたLua文字列、出力: UTF-8エンコードされたLua文字列）
4. The `to_ansi`関数 shall 内部で`Encoding::ANSI.to_bytes()`を使用し、結果のバイト列を`Lua::create_string(&[u8])`でLua文字列化する
5. The `to_utf8`関数 shall Lua文字列から`LuaString::as_bytes()`でバイト列を取得し、`Encoding::ANSI.to_string()`でUTF-8文字列に変換後、`Lua::create_string()`でLua文字列化する
6. When `require "@enc"`が呼ばれた時, the Lua VM shall 登録されたモジュールを返す
7. If 変換が失敗した場合（無効な文字が含まれる等）, the 変換関数 shall nilとエラーメッセージを返す（Lua標準エラーパターン: `result, err = func()`）
8. The 入出力は全てLua文字列型であり、内部的なバイトエンコーディングのみが異なる（UTF-8バイト列 vs ANSIバイト列）

### Requirement 6: 既存encoding/mod.rsの活用
**Objective:** As a 開発者, I want 既存のエンコーディング変換実装を再利用すること, so that コードの重複を避け保守性を維持できる

#### Acceptance Criteria
1. The `@enc`モジュール実装 shall `crate::encoding::Encoding`トレイトを使用する
2. When Windows環境で動作する場合, the エンコーディング変換 shall `encoding/windows.rs`のWindows API実装を使用する
3. When 非Windows環境で動作する場合, the エンコーディング変換 shall `encoding/unix.rs`のパススルー実装を使用する
4. The Windows実装 shall 外部クレートを追加せず、既存の`windows-sys`依存のみを使用する
5. The 実装 shall `Encoding::OEM`は使用せず、将来必要になった場合に`to_oem`/`from_oem`関数として追加可能な設計とする

### Requirement 7: @encモジュールのドキュメントとエラーハンドリング
**Objective:** As a Luaスクリプト開発者, I want エンコーディング変換の使用方法と制限事項を理解できること, so that 正しく機能を活用できる

#### Acceptance Criteria
1. The `@enc`モジュール shall モジュールテーブルに`_VERSION`フィールドを含める
2. The `@enc`モジュール shall モジュールテーブルに`_DESCRIPTION`フィールドを含める（使用方法の説明）
3. When `to_ansi()`に非文字列が渡された場合, the 関数 shall 型エラーを報告する
4. When `to_utf8()`に非文字列が渡された場合, the 関数 shall 型エラーを報告する
5. While tracingが有効な場合, the エンコーディング変換 shall 変換エラーをwarnレベルでログ出力する
6. The ドキュメント shall Lua文字列の内部エンコーディングの違いを明記する（UTF-8 vs ANSI）

### Requirement 8: 後方互換性の維持
**Objective:** As a 既存ユーザー, I want 既存のpasta_luaコードが変更なしで動作すること, so that 移行コストがない

#### Acceptance Criteria
1. The PastaLuaRuntime shall 既存の公開API（`with_config`, `from_loader`など）を維持する
2. The `@pasta_config`モジュール登録 shall 従来通り動作する
3. The トランスパイル済みコードの直接ロード shall 従来通り動作する
4. When 既存のテストが実行された場合, the すべてのテスト shall 変更なしで合格する
