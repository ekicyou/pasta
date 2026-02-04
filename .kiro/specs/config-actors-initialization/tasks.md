# Implementation Plan

## Task Overview
pasta.toml の `[actor.*]` セクションから `CONFIG.actor` を読み込み、Lua ランタイム起動時に `STORE.actors` を自動初期化する機能を実装。STORE.actors = CONFIG.actor の参照共有により、設定ファイルでアクターのプロパティを宣言的に定義可能にする。

---

## Tasks

- [x] 1. CONFIG.actor を STORE.actors に参照共有で初期化
- [x] 1.1 (P) pasta.store モジュールへの初期化ロジック追加
  - `@pasta_config` モジュールを require し、CONFIG.actor を取得
  - `CONFIG.actor` がテーブル型の場合、`STORE.actors = CONFIG.actor` で直接代入（参照共有）
  - `CONFIG.actor` がテーブル型でない（nil、文字列、数値等）場合、`STORE.actors` を空テーブル `{}` のまま維持
  - メタテーブル設定は行わない（pasta.actor モジュールに委譲）
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 1.2 (P) pasta.actor モジュールへのメタテーブル設定ロジック追加
  - モジュール初期化時（`return ACTOR` の前）に `STORE.actors` を走査
  - 各要素（`STORE.actors[name]`）がテーブル型の場合のみ `ACTOR_IMPL` メタテーブルを設定
  - テーブル型でない要素はスキップし、エラーにしない
  - `ACTOR.get_or_create()` の既存動作（メタテーブル設定済みアクターは上書きしない）を維持
  - _Requirements: 2.4, 2.5, 2.6, 4.2_

- [x] 2. テストケース実装
- [x] 2.1 (P) pasta.store 初期化ロジックの単体テスト
  - CONFIG.actor がテーブル型の場合、STORE.actors が同一参照になることを確認
  - CONFIG.actor が nil の場合、STORE.actors が空テーブル `{}` であることを確認
  - CONFIG.actor が非テーブル（文字列、数値等）の場合、STORE.actors が空テーブルであることを確認
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 2.2 (P) pasta.actor メタテーブル設定の単体テスト
  - STORE.actors の各テーブル要素に ACTOR_IMPL メタテーブルが設定されることを確認
  - 非テーブル要素がスキップされ、エラーにならないことを確認
  - _Requirements: 2.4, 2.5, 2.6_

- [x] 2.3 (P) CONFIG.actor と動的追加の共存テスト（統合テスト）
  - pasta.toml に `[actor.さくら]` を定義し、`STORE.actors["さくら"].spot` で値取得できることを確認
  - `ACTOR.get_or_create("さくら")` が CONFIG 由来プロパティを保持したアクターを返すことを確認
  - 動的追加 `STORE.actors["新規"] = {}` と CONFIG 由来アクターが共存することを確認（参照共有の検証）
  - _Requirements: 4.1, 4.2_

- [x]* 2.4 E2E テストの追加（既存テストでカバー済み）
  - SHIORI イベントループ内でアクター参照が正常動作することを確認
  - メタテーブルメソッド（`actor:talk()` 等）が CONFIG 由来アクターでも利用可能なことを確認
  - _Requirements: 2.4, 2.6, 4.2_
  - **注**: 既存の runtime_e2e_test.rs, shiori_event_test.rs 等でE2E動作を検証済み

- [x] 3. ドキュメント整合性の確認と更新
  - SOUL.md - コアバリュー・設計原則との整合性確認（宣言的設定、日本語フレンドリー）
  - TEST_COVERAGE.md - 新規テストのマッピング追加（pasta.store, pasta.actor 初期化テスト）
  - crates/pasta_lua/README.md - CONFIG.actor → STORE.actors 初期化フローの説明追加
  - .kiro/steering/tech.md - 既存技術スタック（Lua 5.5, mlua 0.11）の確認
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 4.1, 4.2_

---

## Notes

**実装上の注意**:
- store.lua は「他の Lua モジュールを require しない」ポリシーだが、`@pasta_config` は Rust 組み込みモジュールのため例外扱い
- CONFIG.actor → STORE.actors は参照共有であり、深いコピーではない（メモリ効率重視）
- ACTOR.get_or_create() は既存の動作を維持（メタテーブル設定済みアクターは上書きしない）
- メタテーブル設定は pasta.actor モジュールの責務であり、pasta.store は行わない

**実装順序**:
- タスク 1.1, 1.2 は並列実行可能（異なるファイルを操作）
- タスク 2.1, 2.2, 2.3 は並列実行可能（独立したテストケース）
- タスク 2.4 は統合テスト完了後に実行（オプショナル）
- タスク 3 は全実装完了後に実行

**想定実装規模**:
- store.lua: 末尾3行追加
- actor.lua: `return ACTOR` 前に5行追加
- 合計: ~8行のコード変更
