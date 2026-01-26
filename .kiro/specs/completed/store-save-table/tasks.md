# Implementation Plan: store-save-table

## Task List

### 1. pasta.storeモジュールにsaveフィールドを追加

- [x] 1.1 saveテーブルの初期化とLuaDocアノテーションを追加
  - `@class Store`定義に`@field save table<string, any>`アノテーションを追加
  - `STORE.save = {}`でフィールドを初期化
  - `@type table<string, any>`アノテーションを追加
  - 既存フィールド（actors, scenes等）と同様のパターンで実装
  - _Requirements: 1.1, 1.3_

- [x] 1.2 STORE.reset()にsaveのリセット処理を追加
  - `STORE.reset()`関数内に`STORE.save = {}`を追加
  - 既存のリセット処理（actors, scenes等）と同様のパターンで実装
  - すべてのフィールドが確実にリセットされることを確認
  - _Requirements: 1.2_

### 2. pasta.ctxモジュールにSTORE.save参照を注入

- [x] 2.1 pasta.storeへの依存追加とCTX.newインターフェース更新
  - ファイル先頭に`local STORE = require("pasta.store")`を追加（既存のACT requireの後）
  - `CTX.new(save, actors)`のシグネチャを`CTX.new(actors)`に変更
  - LuaDocアノテーションから`@param save`を削除
  - `ctx.save = STORE.save`で常にSTORE.saveを参照するように変更
  - `save or {}`のロジックを削除し、シンプルな代入に変更
  - 循環参照が発生しないことを確認（CTX→STORE、STORE→なし）
  - _Requirements: 2.1, 2.2, 2.3, 3.2_

### 3. テスト実装

- [x] 3.1 (P) pasta.storeのユニットテスト
  - `STORE.save`が空テーブルで初期化されることを検証
  - `STORE.reset()`呼び出し後に`STORE.save`が空テーブルにリセットされることを検証
  - `STORE.save`に値を設定し、取得できることを検証
  - 他のフィールド（actors, scenes等）のリセットに影響がないことを検証
  - _Requirements: 1.1, 1.2_

- [x] 3.2 (P) pasta.ctxのユニットテスト
  - `CTX.new()`が`STORE.save`への参照を持つことを検証（`ctx.save == STORE.save`）
  - `CTX.new(actors_table)`でactorsが正しく設定されることを検証
  - `CTX.new()`のシグネチャにsaveパラメータが存在しないことを確認
  - _Requirements: 2.1, 2.3_

- [x] 3.3 参照同一性の統合テスト
  - `STORE.save`に値を設定後、`CTX.new()`で作成したctxから同じ値にアクセスできることを検証
  - `ctx.save`に値を設定後、`STORE.save`から同じ値にアクセスできることを検証（参照同一性）
  - 複数のCTXインスタンスが同一のSTORE.saveを共有することを検証
  - _Requirements: 2.1, 3.1, 3.2_

### 4. ドキュメント整合性確認

- [x] 4.1 ドキュメント整合性の確認と更新
  - [x] SOUL.md - コアバリュー・設計原則との整合性確認（変更不要）
  - [x] SPECIFICATION.md - 言語仕様の更新（該当なし - DSL文法変更なし）
  - [x] GRAMMAR.md - 文法リファレンスの同期（該当なし）
  - [x] TEST_COVERAGE.md - 新規テストのマッピング追加（store_save_test.lua追加）
  - [x] クレートREADME - API変更の反映（該当なし - 内部実装変更のみ）
  - [x] steering/* - 該当領域のステアリング更新（変更不要）

## Implementation Notes

- **実装順序**: タスクは1.1 → 1.2 → 2.1 → (3.1, 3.2並列) → 3.3 の順で実行
- **並列実行**: 3.1と3.2は異なるテストファイルを扱うため並列実行可能
- **循環参照**: pasta.storeは他モジュールをrequireしないパターンを維持（3.1）
- **破壊的変更**: `CTX.new()`のインターフェースが変更されるが、未リリースプロジェクトのため影響なし
