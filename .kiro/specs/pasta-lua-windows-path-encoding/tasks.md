# Implementation Plan

## Task Overview

実装作業を4つのメジャータスク（encoding層、loader層、runtime層、テスト）に分類し、合計10個のサブタスクで構成。すべてのタスクは1-3時間で完了可能な粒度に設計。

---

## Implementation Tasks

### 1. encoding層の拡張
エンコーディング変換機能を追加し、既存のバグ修正を実施する。

- [x] 1.1 (P) to_ansi_bytes関数の実装
  - UTF-8文字列をシステムネイティブエンコーディングのバイト列に変換する関数を追加
  - Windows環境では`Encoding::ANSI.to_bytes()`を使用してANSI変換
  - Unix環境では`s.as_bytes().to_vec()`でUTF-8バイト列をそのまま返す
  - エラー時は`std::io::Error`を返却、panic禁止を徹底
  - `#[cfg(windows)]`と`#[cfg(not(windows))]`で環境別に実装を分岐
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 1.2 (P) path_to_lua関数の廃止
  - バグのある`path_to_lua`関数と関連テストを削除
  - `path_from_lua`は将来使用の可能性があるため維持
  - _Requirements: 2.5_

- [x] 1.3 (P) encoding層のユニットテスト追加
  - ASCIIパス変換テスト（全プラットフォームで入出力同一を確認）
  - 日本語パス変換テスト（Windows専用、`#[cfg(windows)]`でマーク）
  - UTF-8→ANSI→UTF-8ラウンドトリップテストで変換精度を検証
  - エラーハンドリングテスト（無効な入力に対する`std::io::Error`返却確認）
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

---

### 2. loader層の拡張
LoaderContextにバイト列生成機能を追加し、package.path設定用のインターフェースを提供する。

- [x] 2.1 generate_package_path_bytesメソッドの実装
  - 既存の`generate_package_path()`を内部で呼び出し、UTF-8文字列を生成
  - 生成した文字列を`encoding::to_ansi_bytes`で変換してバイト列化
  - セミコロン区切りの`?.lua;?/init.lua`パターン形式を維持
  - エラー時は`std::io::Error`を返却、panic禁止
  - 既存の`generate_package_path()`メソッドは後方互換性のため維持
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 2.2 (P) loader層のユニットテスト追加
  - バイト列生成の正常系テスト（期待されるパターン形式の確認）
  - Windows環境でのANSI変換確認テスト（`#[cfg(windows)]`）
  - Unix環境でのUTF-8パススルー確認テスト
  - `generate_package_path()`の後方互換性テスト（既存動作が維持されることを確認）
  - _Requirements: 3.1, 3.2, 3.3_

---

### 3. runtime層の実装
package.pathのバイト列設定と@encモジュールを実装し、Lua VMへの統合を完成させる。

- [x] 3.1 setup_package_pathメソッドの修正
  - `LoaderContext::generate_package_path_bytes()`を呼び出してバイト列取得
  - `lua.create_string(&bytes)`でバイト列をLua文字列に変換
  - `package.set("path", lua_string)`でpackage.pathを設定
  - `std::io::Error`を`mlua::Error::ExternalError(Arc::new(e))`に変換してエラー伝播
  - tracingでdebugレベルのログ出力（UTF-8として解釈可能な範囲）
  - panic禁止、全てLuaResultで返却
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [x] 3.2 @encモジュールの実装
  - 新規ファイル`src/runtime/enc.rs`を作成
  - `runtime/mod.rs`に`mod enc;`を追加
  - `register(lua: &Lua) -> LuaResult<Table>`関数でモジュールテーブル作成
  - `_VERSION`と`_DESCRIPTION`フィールドをモジュールに追加
  - `to_ansi()`関数: UTF-8 Lua文字列をANSI Lua文字列に変換、`(result, nil)`または`(nil, err)`を返却
  - `to_utf8()`関数: ANSI Lua文字列をUTF-8 Lua文字列に変換、`(result, nil)`または`(nil, err)`を返却
  - `Encoding::ANSI.to_bytes()`と`Encoding::ANSI.to_string()`を使用
  - 変換失敗時はLua標準エラーパターン`(nil, err)`で返却、panic禁止
  - tracingでwarnレベルのエラーログ出力
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 6.1, 6.2, 6.3, 6.4, 7.1, 7.2, 7.5_

- [x] 3.3 @encモジュールの登録統合
  - `PastaLuaRuntime::from_loader()`内で`enc::register(&lua)?`を呼び出し
  - `package.loaded["@enc"]`に登録してrequireで取得可能にする
  - 既存の`@pasta_config`モジュール登録パターンを踏襲
  - _Requirements: 5.1, 6.1_

- [x] 3.4 (P) runtime層のユニットテスト追加
  - `@enc.to_ansi()`のUTF-8→ANSI変換テスト（日本語文字列、Windows専用）
  - `@enc.to_utf8()`のANSI→UTF-8変換テスト（ラウンドトリップ確認）
  - エラーハンドリングテスト：型エラー（`enc.to_ansi(123)`）、変換エラー（無効なバイト列）
  - `_VERSION`と`_DESCRIPTION`フィールドの存在確認テスト
  - tracingログ出力テスト（warnレベルエラーログ確認）
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 7.1, 7.2, 7.3, 7.4, 7.5_

---

### 4. 統合テストと検証
全レイヤーの統合動作を検証し、後方互換性とWindows環境での実動作を確認する。

- [x] 4.1 (P) 統合テストファイルの作成
  - `tests/pasta_lua_encoding_test.rs`を新規作成
  - Windows環境での日本語パスrequireテスト（例: `"C:\\ユーザー\\テスト\\scripts\\module.lua"`）
  - `#[cfg_attr(not(windows), ignore)]`でWindows専用テストをマーク
  - テストフィクスチャとして日本語ファイル名のLuaスクリプトを用意
  - require成功確認、モジュールロード確認、エラーが発生しないことを検証
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 4.1, 4.2, 4.3, 4.4_

- [x] 4.2 (P) @encモジュールのE2Eテスト追加
  - Lua VM初期化後に`require "@enc"`でモジュール取得
  - `enc.to_ansi("日本語パス")`でANSI変換、結果が非nilであることを確認
  - `enc.to_utf8(ansi_result)`でUTF-8復元、元の文字列と一致することを確認
  - エラーケースのE2E確認（型エラー、変換エラー）
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8_

- [x]* 4.3 後方互換性テストの実施
  - `cargo test --all`で全既存テストが成功することを確認
  - `PastaLuaRuntime::new()`, `with_config()`, `from_loader()`の既存API動作確認
  - `@pasta_config`モジュールの既存動作確認
  - トランスパイル済みコードのロード動作確認
  - リグレッション検出時は即座に原因調査し修正
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

---

## Implementation Notes

### 環境別実装パターン
```rust
#[cfg(windows)]
pub fn to_ansi_bytes(s: &str) -> std::io::Result<Vec<u8>> {
    Encoding::ANSI.to_bytes(s)
}

#[cfg(not(windows))]
pub fn to_ansi_bytes(s: &str) -> std::io::Result<Vec<u8>> {
    Ok(s.as_bytes().to_vec())
}
```

### @encモジュール登録パターン
```rust
// src/runtime/enc.rs
pub fn register(lua: &Lua) -> LuaResult<Table> {
    let enc = lua.create_table()?;
    enc.set("_VERSION", "0.1.0")?;
    enc.set("_DESCRIPTION", "Encoding conversion (UTF-8 <-> ANSI)")?;
    enc.set("to_ansi", lua.create_function(to_ansi_impl)?)?;
    enc.set("to_utf8", lua.create_function(to_utf8_impl)?)?;
    Ok(enc)
}

// src/runtime/mod.rs (from_loader内)
let enc = enc::register(&lua)?;
globals.set("package", package)?; // 既存
```

### テスト実装ガイドライン
- Windows専用: `#[cfg(windows)]`または`#[cfg_attr(not(windows), ignore)]`
- ユニットテスト: 各モジュールの`#[cfg(test)]`ブロック
- 統合テスト: `tests/pasta_lua_encoding_test.rs`
- CI設定: `windows-latest`と`ubuntu-latest`マトリックス

---

## Quality Gates

すべてのタスク完了時、以下を満たすこと：

1. **Test Gate**: `cargo test --all` が全プラットフォームで成功
2. **Spec Gate**: 全要件（1.1-8.4）がタスクに紐付けられている
3. **Doc Gate**: 公開API（`to_ansi_bytes`, `@enc`モジュール）にdocコメント追加
4. **Steering Gate**: pasta_lua構造規約に準拠（`src/encoding/`, `src/loader/`, `src/runtime/enc.rs`配置）
