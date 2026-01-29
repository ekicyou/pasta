# Implementation Plan

## Overview

本実装計画は、pasta_luaのTOML設定にて`[lua]`セクションの`libs`配列でLua標準ライブラリとmlua-stdlibモジュールを統合制御する機能を実装する。既存の`RuntimeConfig`個別フラグを`libs`配列に完全置換し、Cargo風の配列記法と減算記法をサポートする。

## Implementation Tasks

### 1. Config Layer: LuaConfig構造体実装

- [x] 1.1 (P) LuaConfig構造体とデシリアライゼーション実装
  - `loader/config.rs`に`LuaConfig`構造体を追加
  - `libs: Vec<String>`フィールドをserde Deserializeで実装
  - `default_libs()`関数でデフォルト値を返す（`["std_all", "assertions", "testing", "regex", "json", "yaml"]`）
  - `#[serde(default = "default_libs")]`属性でlibs省略時のデフォルト値適用
  - `Default` traitを実装
  - _Requirements: 1.1, 1.2, 5.1_

- [x] 1.2 (P) PastaConfigへのlua()メソッド追加
  - `PastaConfig`に`pub fn lua(&self) -> Option<LuaConfig>`メソッド追加
  - custom_fieldsから`[lua]`セクションを取得し、`LuaConfig`へデシリアライズ
  - 既存の`logging()`, `persistence()`メソッドパターンに準拠
  - _Requirements: 1.1, 5.2_

### 2. Runtime Layer: RuntimeConfig刷新

- [x] 2.1 RuntimeConfig構造体のlibs配列化
  - `RuntimeConfig`の既存フィールド（`enable_std_libs`, `enable_testing`等）を削除
  - `libs: Vec<String>`フィールドに置換
  - `Default::default()`で`LuaConfig::default_libs()`と同等の値を返す
  - `new()`, `full()`, `minimal()`ファクトリメソッドを更新
  - `From<LuaConfig>` trait実装で`LuaConfig`から`RuntimeConfig`への変換
  - _Requirements: 1.1, 1.2, 5.1, 5.3_

- [x] 2.2 StdLib変換ロジック実装（to_stdlib()メソッド）
  - `RuntimeConfig::to_stdlib() -> Result<StdLib, ConfigError>`メソッド実装
  - 加算要素を先に処理し、減算要素を後に処理（順序非依存）
  - `std_`プレフィックス付き要素をStdLibビットフラグに変換
  - `"-"`プレフィックス要素を減算処理
  - 空配列`[]`の場合は`StdLib::NONE`を返す
  - 不明ライブラリ名で`ConfigError::UnknownLibrary`を返す
  - _Requirements: 1.3, 1.4, 1.5, 1.6, 2.1, 2.2, 2.3, 2.4, 2.5, 4.1, 4.3, 4.4_

- [x] 2.3 (P) mlua-stdlibモジュール判定ロジック実装
  - `RuntimeConfig::should_enable_module(&self, module: &str) -> bool`メソッド実装
  - `libs`配列にモジュール名が含まれるか判定
  - 減算記法（`"-module"`）で除外されている場合はfalseを返す
  - _Requirements: 1.4, 1.5, 2.3_

- [x] 2.4 (P) セキュリティ警告ロジック実装
  - `RuntimeConfig::validate_and_warn(&self)`メソッド実装
  - `std_debug`または`std_all_unsafe`検出時に`tracing::warn!`で警告出力
  - `env`モジュール検出時に`tracing::warn!`で警告出力
  - 有効化ライブラリ一覧を`tracing::debug!`で出力
  - 減算により除外されたstd_debug/envは警告対象外
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

### 3. Error Handling: ConfigErrorエラー型実装

- [x] 3.1 (P) ConfigError列挙型実装
  - `error.rs`に`ConfigError`列挙型を追加
  - `UnknownLibrary(String)`バリアントを実装
  - `thiserror::Error`でエラーメッセージ実装（有効ライブラリ名一覧を含める）
  - _Requirements: 4.1, 4.4_

### 4. Runtime Integration: PastaLuaRuntimeへの統合

- [x] 4.1 PastaLuaRuntimeでのRuntimeConfig使用
  - `PastaLuaRuntime::with_config()`でRuntimeConfigを受け取る
  - `RuntimeConfig::validate_and_warn()`を呼び出す
  - `RuntimeConfig::to_stdlib()?`でStdLibフラグを取得
  - `Lua::unsafe_new_with(std_lib, opts)`でLua VM初期化
  - エラー処理を適切に実装
  - _Requirements: 1.1, 3.1, 3.2, 4.4_

- [x] 4.2 mlua-stdlibモジュール動的登録
  - `RuntimeConfig::should_enable_module()`で各モジュールの有効化判定
  - `assertions`, `testing`, `env`, `regex`, `json`, `yaml`モジュールを条件付き登録
  - 既存のmlua-stdlib登録ロジックを`should_enable_module()`による制御に変更
  - _Requirements: 1.4, 1.5_

### 5. Testing: テストスイート実装

- [x] 5.1 RuntimeConfig単体テスト
  - `to_stdlib()`での各ライブラリ名パーステスト
  - 減算記法処理テスト（`["std_all", "-std_debug"]` → `ALL_SAFE`）
  - 空配列テスト（`[]` → `StdLib::NONE`）
  - 不明ライブラリ名でエラーテスト
  - `should_enable_module()`でのモジュール判定テスト
  - `validate_and_warn()`での警告出力テスト（tracing-subscriber使用）
  - _Requirements: 1.3, 1.4, 1.5, 1.6, 2.1, 2.2, 2.4, 2.5, 3.1, 3.2, 4.1_

- [x] 5.2 LuaConfig統合テスト
  - TOML配列デシリアライゼーションテスト
  - デフォルト値適用テスト（libs省略時）
  - `PastaConfig::lua()`メソッドテスト
  - _Requirements: 1.1, 1.2, 5.1_

- [x] 5.3 E2Eテスト
  - 様々なlibs配列パターンでのpasta.toml読み込みテスト
  - 空配列`libs = []`での最小構成動作確認
  - セキュリティ警告出力検証
  - 後方互換性テスト（`[lua]`セクション省略時）
  - _Requirements: 1.1, 1.2, 1.6, 3.1, 3.2, 5.1_

### 6. Documentation and Migration: ドキュメント整合性確認

- [x] 6.1 ドキュメント整合性の確認と更新
  - SOUL.md - コアバリュー・設計原則との整合性確認
  - SPECIFICATION.md - 言語仕様の更新（該当なし）
  - GRAMMAR.md - 文法リファレンスの同期（該当なし）
  - TEST_COVERAGE.md - 新規テストのマッピング追加
  - `crates/pasta_lua/README.md` - RuntimeConfig API変更の反映
  - `steering/tech.md` - 依存関係・アーキテクチャ更新（該当する場合）
  - _Requirements: 5.1, 5.2, 5.3_

## Implementation Notes

### Task Progression
1. Config Layer（1.1-1.2）: 並列実行可能
2. Runtime Layer（2.1-2.4）: 2.1完了後、2.2-2.4は並列実行可能
3. Error Handling（3.1）: 並列実行可能
4. Runtime Integration（4.1-4.2）: 2.2, 3.1完了後に実行
5. Testing（5.1-5.3）: 実装完了後に実行
6. Documentation（6.1）: 全実装・テスト完了後に実行

### Dependencies
- Task 2.1 → Task 2.2, 2.3, 2.4
- Task 2.2, 3.1 → Task 4.1
- Task 2.3 → Task 4.2
- All implementation → Task 5.1, 5.2, 5.3
- All tests → Task 6.1

### Requirements Coverage
- Requirement 1.1-1.6: Tasks 1.1, 1.2, 2.1, 2.2, 4.1, 4.2, 5.1, 5.2, 5.3
- Requirement 2.1-2.5: Tasks 2.2, 5.1
- Requirement 3.1-3.4: Tasks 2.4, 4.1, 5.1, 5.3
- Requirement 4.1-4.4: Tasks 2.2, 3.1, 4.1, 5.1
- Requirement 5.1-5.3: Tasks 1.1, 1.2, 2.1, 5.2, 5.3, 6.1

全27個のAcceptance Criteriaが18個のタスクでカバーされています。
