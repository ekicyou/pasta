# Implementation Plan: shiori-entry

## Task List

- [ ] 1. entry.lua の新規作成
- [ ] 1.1 (P) entry.lua ファイル作成とモジュール構造実装
  - `crates/pasta_lua/scripts/pasta/shiori/entry.lua` を新規作成
  - lua-coding.md 規約に従ったモジュール構造（require文 → テーブル宣言 → 公開関数）
  - `pasta.shiori.event` モジュールを require
  - グローバル `SHIORI` テーブルの初期化（既存テーブルがあれば上書きせず追加）
  - _Requirements: 1.1, 1.2, 1.4_

- [ ] 1.2 (P) SHIORI.load 関数の実装
  - `SHIORI.load(hinst, load_dir)` 関数を実装
  - 引数: hinst (integer), load_dir (string)
  - 戻り値: 常に `true` を返却（現時点では拡張ポイントのみ）
  - 将来の拡張ポイントをコメントで明示（設定ファイル読み込み、セーブデータ復元）
  - _Requirements: 1.3, 2.1, 2.2, 2.3, 2.5_

- [ ] 1.3 (P) SHIORI.request 関数の実装
  - `SHIORI.request(req)` 関数を実装
  - `EVENT.fire(req)` を呼び出してレスポンスを返却
  - LuaDoc で req テーブルの構造を文書化（id, method, version, charset, sender, reference, dic）
  - _Requirements: 1.3, 3.1, 3.2, 3.3, 3.4_

- [ ] 1.4 (P) SHIORI.unload 関数の実装
  - `SHIORI.unload()` 関数を実装
  - 引数なし、戻り値なし（void）
  - 将来の拡張ポイントをコメントで明示（セーブデータ保存、リソース解放）
  - _Requirements: 1.3, 4.1, 4.2, 4.3, 4.4_

- [ ] 2. Rust側ランタイムの変更
- [ ] 2.1 runtime/mod.rs のファイルパス変更（2箇所）
  - `crates/pasta_lua/src/runtime/mod.rs:309` の `main.lua` を `entry.lua` に変更
  - `crates/pasta_lua/src/runtime/mod.rs:378` の `main.lua` を `entry.lua` に変更
  - エラーログメッセージも更新（"main.lua" → "entry.lua"）
  - _Requirements: 5.4, 6.3_

- [ ] 3. 本番コード移行の完了
- [ ] 3.1 main.lua の削除
  - `crates/pasta_lua/scripts/pasta/shiori/main.lua` を削除
  - git commit で明確に記録
  - _Requirements: 5.2_

- [ ] 4. テストフィクスチャの更新
- [ ] 4.1 (P) tests/support/ フィクスチャのリネームと req テーブル対応
  - `crates/pasta_shiori/tests/support/scripts/pasta/shiori/main.lua` を `entry.lua` にリネーム
  - SHIORI.request のシグネチャを reqテーブル対応に変更（request_text → req）
  - ミニマル実装を維持（グローバル値記録のみ）
  - _Requirements: 5.3, 7.3_

- [ ] 4.2 (P) tests/fixtures/shiori_lifecycle/ フィクスチャのリネーム
  - `crates/pasta_shiori/tests/fixtures/shiori_lifecycle/scripts/pasta/shiori/main.lua` を `entry.lua` にリネーム
  - 既に reqテーブル対応済みなので実装変更不要
  - request_count カウンタ動作を維持
  - _Requirements: 5.3, 7.3_

- [ ] 5. 単体テストの実装
- [ ] 5.1 (P) entry.lua 単体テストの作成
  - `crates/pasta_lua/tests/lua_specs/shiori_entry_spec.lua` を新規作成
  - テスト1: require 後にグローバル SHIORI テーブルが存在すること
  - テスト2: SHIORI.load(0, "/path") が true を返すこと
  - テスト3: SHIORI.request(valid_req) が文字列を返すこと
  - テスト4: SHIORI.unload() がエラーなく完了すること
  - _Requirements: 7.1_

- [ ] 6. 統合テストの実行と検証
- [ ] 6.1 既存 SHIORI 統合テストの実行
  - `cargo test --package pasta_shiori` で SHIORI ライフサイクルテスト実行
  - `shiori_lifecycle_test.rs` が通過することを確認
  - `shiori_event_test.rs` が通過することを確認（イベント振り分け動作）
  - _Requirements: 6.2, 7.2_

- [ ] 6.2 全体リグレッションテストの実行
  - `cargo test --all` を実行
  - 全テストが通過することを確認（EVENT, RES モジュールテスト含む）
  - トランスパイラテストへの影響がないことを確認
  - _Requirements: 7.4_

- [ ] 7. ドキュメントの更新
- [ ] 7.1 (P) scripts/README.md の更新
  - `crates/pasta_lua/scripts/README.md` でエントリーポイント名を main.lua → entry.lua に変更
  - SHIORI.load/request/unload の説明を更新
  - _Requirements: 8.1, 8.2_

- [ ] 7.2 (P) steering ドキュメントの確認と更新
  - `.kiro/steering/` 配下で main.lua への参照がないか検索
  - 参照があれば entry.lua に更新
  - 参照がなければスキップ
  - _Requirements: 8.3_

- [ ] 8. ドキュメント整合性の確認と更新
- [ ] 8.1 コアドキュメントとの整合性確認
  - SOUL.md - コアバリュー・設計原則との整合性確認（SHIORI統合は影響なし）
  - SPECIFICATION.md - 言語仕様への影響確認（該当なし）
  - GRAMMAR.md - 文法リファレンス同期確認（該当なし）
  - TEST_COVERAGE.md - 新規テストのマッピング追加
  - クレートREADME - pasta_lua, pasta_shiori の API変更確認
  - _Requirements: 全要件_

---

## Requirements Coverage

| Requirement | Tasks |
|-------------|-------|
| 1.1, 1.2, 1.4 | 1.1 |
| 1.3 | 1.2, 1.3, 1.4 |
| 2.1, 2.2, 2.3, 2.5 | 1.2 |
| 3.1, 3.2, 3.3, 3.4 | 1.3 |
| 4.1, 4.2, 4.3, 4.4 | 1.4 |
| 5.2 | 3.1 |
| 5.3 | 4.1, 4.2 |
| 5.4 | 2.1 |
| 6.2 | 6.1 |
| 6.3 | 2.1 |
| 7.1 | 5.1 |
| 7.2 | 6.1 |
| 7.3 | 4.1, 4.2 |
| 7.4 | 6.2 |
| 8.1, 8.2 | 7.1 |
| 8.3 | 7.2 |
| 全要件 | 8.1 |

---

## Task Progression Strategy

### Phase 1: Core Implementation（並列実行可能）
- タスク 1.1-1.4（entry.lua 作成）は完全に独立して実装可能
- 各関数が独立しているため、4つのサブタスクを並列実行可能

### Phase 2: Runtime Integration
- タスク 2.1（Rust側変更）は Phase 1 完了後に実施
- entry.lua が存在しないと動作確認できない

### Phase 3: Migration Completion
- タスク 3.1（main.lua削除）は Phase 2 完了後
- Rust側が entry.lua を読み込むことを確認してから削除

### Phase 4: Test Infrastructure（並列実行可能）
- タスク 4.1-4.2（テストフィクスチャ）は並列実行可能
- タスク 5.1（単体テスト）も並列で作成可能

### Phase 5: Validation
- タスク 6.1-6.2（統合テスト）で全体動作確認

### Phase 6: Documentation（並列実行可能）
- タスク 7.1-7.2（ドキュメント）は並列実行可能
- タスク 8.1（最終整合性確認）で完了
