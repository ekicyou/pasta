# 実装計画

## タスク概要

本仕様は、`pasta_lua`クレートのLuaコーディング規約を策定し、既存コードをリファクタリングする。全9要件を3フェーズ（ドキュメント作成、リファクタリング、検証）で実装する。

---

## フェーズ1: コーディング規約ドキュメント作成

### - [ ] 1. `.kiro/steering/lua-coding.md` の作成

- [ ] 1.1 (P) 命名規約セクションの作成
  - snake_case（ローカル変数・関数）、UPPER_CASE（モジュールテーブル）の規約を文書化
  - `_IMPL`サフィックス規約（クラス実装メタテーブル）を定義
  - プライベートメンバーのアンダースコアプレフィックス規約を記載
  - 日本語識別子の許可ルールを明記
  - PascalCase禁止ルールを明記
  - コード例とアンチパターンを含める
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

- [ ] 1.2 (P) モジュール構造規約セクションの作成
  - 単一モジュールテーブルパターンを文書化（ファイル名ベース、UPPER_CASE）
  - `require`文の先頭配置ルールを記載
  - モジュールテーブルの末尾返却ルールを記載
  - `pasta.store`循環依存回避パターンを説明
  - 標準モジュール構造テンプレートを提供
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [ ] 1.3 (P) クラス設計パターンセクションの作成
  - MODULE/MODULE_IMPL分離パターンを文書化
  - `MODULE.new(args)`コンストラクタ規約を定義
  - 明示的self + ドット構文（実装側）の規約を記載
  - コロン構文（呼び出し側）の許可ルールを記載
  - `setmetatable`パターンを説明
  - モジュールレベル関数とインスタンスメソッドの分離を明記
  - シングルトンパターン（requireキャッシング）を文書化
  - `MODULE.instance()`アンチパターン禁止を記載
  - シングルトンパターン例とクラスパターンテンプレートを含める
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8_

- [ ] 1.4 (P) EmmyLua型アノテーション規約セクションの作成
  - `@module`アノテーション（ファイル先頭）の規約を記載
  - `@class`アノテーション規約を定義
  - `@param`/`@return`アノテーション（全公開関数）の規約を記載
  - `@field`アノテーション（クラスプロパティ）の規約を記載
  - `|nil`戻り値型アノテーションの規約を記載
  - `@param ... type`可変長引数構文を文書化（`@vararg`禁止）
  - コード例を含める
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

- [ ] 1.5 (P) エラーハンドリング規約セクションの作成
  - nilチェックパターンを文書化
  - guard clause パターンを定義
  - pcall使用パターンを記載
  - サイレントnil返却禁止ルールを明記
  - コード例とアンチパターンを含める
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [ ] 1.6 (P) Pasta固有ランタイム規約セクションの作成
  - PASTAモジュールAPI使用パターンを文書化（`PASTA.create_actor`, `PASTA.create_scene`, `PASTA.create_word`）
  - CTXオブジェクトパターンとライフサイクルを説明
  - ACTオブジェクトパターンを文書化
  - PROXYパターンを説明
  - STOREパターンを文書化
  - 各パターンのコード例を含める
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ] 1.7 (P) テスト・Lint規約セクションの作成
  - lua_testフレームワーク使用パターンを文書化（`describe`, `it`, `expect`）
  - テストファイル命名規約`*_test.lua`を定義
  - describe/itパターンのテスト構造テンプレートを提供
  - luacheck設定と使用方法を文書化
  - `.luacheckrc`設定例を含める
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

### - [ ] 2. `.luacheckrc` 設定ファイルの作成

- [ ] 2.1 luacheck設定ファイルの作成
  - `crates/pasta_lua/.luacheckrc`を作成
  - グローバル変数ホワイトリスト設定（PASTA, ACTOR, SCENE, WORD, ACT, CTX, STORE, GLOBAL）
  - 日本語識別子許可設定（UTF-8対応）
  - 未使用変数警告設定（アンダースコアプレフィックス考慮）
  - 行長制限設定（120文字）
  - luacheck実行コマンドの簡易化検討（エイリアス・スクリプト等）
  - _Requirements: 9.4, 9.5_

---

## フェーズ2: 既存コードリファクタリング（依存関係順）

### - [ ] 3. `store.lua` のリファクタリング

- [ ] 3.1 store.lua の軽微な調整
  - EmmyLua型アノテーションの完全性確認
  - 命名規約準拠確認（既に準拠済みのはず）
  - シングルトンパターンが正しいことを確認
  - _Requirements: 8.1, 8.2, 8.3_

### - [ ] 4. `word.lua` のリファクタリング

- [ ] 4.1 word.lua のモジュール名変更
  - モジュールテーブル名を`MOD`から`WORD`に変更
  - 全ての参照箇所を更新
  - _Requirements: 8.3_

- [ ] 4.2 word.lua のクラスパターンリファクタリング
  - `WordBuilder`クラスを`WORD_BUILDER_IMPL`に変更
  - `WORD`モジュールテーブルと`WORD_BUILDER_IMPL`を分離
  - コロン構文のメソッド定義をドット構文+明示的selfに変換
  - `setmetatable(obj, { __index = WORD_BUILDER_IMPL })`パターンに変更
  - _Requirements: 2.3, 2.6, 4.1, 4.2, 4.3, 4.5, 8.4, 8.5_

- [ ] 4.3 word.lua の型アノテーション修正
  - `@vararg`を`@param ... type`に修正
  - 全公開関数に`@param`/`@return`が完全であることを確認
  - `@class`/`@field`アノテーションを補完
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.6, 8.1_

- [ ] 4.4 word.lua の動作検証
  - 既存テスト実行（`cargo test -p pasta_lua lua_unittest_runner`）
  - luacheck実行で警告がないことを確認
  - _Requirements: 8.7, 8.8_

### - [ ] 5. `actor.lua` のリファクタリング

- [ ] 5.1 actor.lua のクラスパターンリファクタリング
  - `ActorWordBuilder`を`ACTOR_WORD_BUILDER_IMPL`に変更
  - `ACTOR`モジュールテーブルと`ACTOR_IMPL`を分離
  - コロン構文のメソッド定義をドット構文+明示的selfに変換
  - `PROXY`も同様に`PROXY_IMPL`パターンに変更
  - _Requirements: 2.3, 2.6, 4.1, 4.2, 4.3, 4.5, 8.4, 8.5_

- [ ] 5.2 actor.lua の型アノテーション修正
  - `@vararg`を`@param ... type`に修正
  - 全公開関数に`@param`/`@return`が完全であることを確認
  - `@class`/`@field`アノテーションを補完
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.6, 8.1_

- [ ] 5.3 actor.lua の動作検証
  - 既存テスト実行（`actor_word_spec.lua`含む）
  - luacheck実行で警告がないことを確認
  - _Requirements: 8.7, 8.8_

### - [ ] 6. `scene.lua` のリファクタリング

- [ ] 6.1 scene.lua のモジュール名変更
  - モジュールテーブル名を`MOD`から`SCENE`に変更
  - 全ての参照箇所を更新
  - _Requirements: 8.3_

- [ ] 6.2 scene.lua の型アノテーション補完
  - 欠落している公開関数に`@param`/`@return`を追加
  - 一貫性を確認
  - _Requirements: 5.1, 5.3, 8.1, 8.2_

- [ ] 6.3 scene.lua の動作検証
  - 既存テスト実行
  - luacheck実行で警告がないことを確認
  - _Requirements: 8.7, 8.8_

### - [ ] 7. `act.lua` のリファクタリング

- [ ] 7.1 act.lua のクラスパターンリファクタリング
  - `ACT`モジュールテーブルと`ACT_IMPL`を分離
  - コロン構文のメソッド定義をドット構文+明示的selfに変換
  - `__index`メタメソッドを`ACT_IMPL.__index(self, key)`に変換
  - `setmetatable(obj, ACT_IMPL)`パターンに変更
  - _Requirements: 4.1, 4.2, 4.3, 4.5, 8.4, 8.5_

- [ ] 7.2 act.lua の型アノテーション確認
  - 既存の型アノテーションが完全であることを確認
  - 必要に応じて補完
  - _Requirements: 5.1, 5.2, 5.3, 8.1_

- [ ] 7.3 act.lua の動作検証
  - 既存テスト実行
  - luacheck実行で警告がないことを確認
  - _Requirements: 8.7, 8.8_

### - [ ] 8. `ctx.lua` のリファクタリング

- [ ] 8.1 ctx.lua のクラスパターンリファクタリング
  - `CTX`モジュールテーブルと`CTX_IMPL`を分離
  - コロン構文のメソッド定義をドット構文+明示的selfに変換
  - `CTX.new()`は既に準拠しているため、メソッド定義の構文変換に注力
  - _Requirements: 4.1, 4.2, 4.3, 4.5, 8.4, 8.5_

- [ ] 8.2 ctx.lua の型アノテーション確認
  - 既存の型アノテーションが完全であることを確認
  - 必要に応じて補完
  - _Requirements: 5.1, 5.2, 5.3, 8.1_

- [ ] 8.3 ctx.lua の動作検証
  - 既存テスト実行
  - luacheck実行で警告がないことを確認
  - _Requirements: 8.7, 8.8_

### - [ ] 9. その他ファイルの軽微な調整

- [ ] 9.1 (P) global.lua の調整
  - 型アノテーションの確認
  - 命名規約準拠確認
  - _Requirements: 8.1, 8.2, 8.3_

- [ ] 9.2 (P) init.lua の調整
  - 型アノテーションの確認
  - エントリーポイントとして変更不要であることを確認
  - _Requirements: 8.1, 8.2_

### - [ ] 10. テストファイル命名の統一

- [ ] 10.1 テストファイルのリネーム
  - `lua_specs/actor_word_spec.lua` → `actor_word_test.lua`
  - `lua_specs/transpiler_spec.lua` → `transpiler_test.lua`
  - テストファイル内のコメント・ドキュメントも更新（該当する場合）
  - _Requirements: 9.2_

---

## フェーズ3: 検証と最終確認

### - [ ] 11. 統合テストと品質確認

- [ ] 11.1 全Luaテストの実行
  - `cargo test -p pasta_lua lua_unittest_runner`で全テストを実行
  - リグレッションがないことを確認
  - 全テストがパスすることを確認
  - _Requirements: 8.7, 8.8_

- [ ] 11.2 luacheckによる静的解析
  - 全Luaファイルに対してluacheckを実行
  - 警告・エラーがないことを確認
  - 日本語識別子が正しく許可されていることを確認
  - _Requirements: 9.4, 9.5_

- [ ] 11.3 コーディング規約への準拠確認
  - `.kiro/steering/lua-coding.md`の規約に全ファイルが準拠していることを確認
  - 命名規約、モジュール構造、クラスパターン、型アノテーションの一貫性を検証
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 3.1, 3.2, 3.3, 3.4, 3.5, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

### - [ ] 12. ドキュメント整合性の確認と更新

- [ ] 12.1 SOUL.md との整合性確認
  - コアバリュー・設計原則との整合性を確認
  - Luaコーディング規約がプロジェクトビジョンに沿っていることを検証

- [ ] 12.2 tech.md の更新
  - `pasta_lua`クレート（Luaバックエンド層）をtech.mdの技術スタックに追加
  - Lua 5.x、mlua、EmmyLua、luacheck v1.2.0の情報を記載
  - アーキテクチャ原則に`pasta_lua`の位置づけを追記

- [ ] 12.3 structure.md の確認
  - `pasta_lua/scripts/pasta/`と`scriptlibs/`構造が正しく記載されていることを確認
  - 必要に応じて更新

- [ ] 12.4 TEST_COVERAGE.md の更新
  - Luaテスト（`*_test.lua`）のマッピングを追加
  - lua_testフレームワークの情報を記載

- [ ] 12.5 クレートREADMEの確認
  - `crates/pasta_lua/README.md`がAPI変更を反映していることを確認
  - 必要に応じて更新

- [ ] 12.6 ステアリング整合性の最終確認
  - `.kiro/steering/lua-coding.md`が他のステアリングファイルと整合していることを確認
  - workflow.mdのDoD基準を満たしていることを検証
