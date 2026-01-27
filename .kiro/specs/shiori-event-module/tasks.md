# Implementation Plan: shiori-event-module

## Task Breakdown

### Phase 1: Core Module Implementation

- [ ] 1. (P) ハンドラ登録テーブルモジュールを実装
- [ ] 1.1 (P) register.lua の基本構造を作成
  - 空のREGテーブルをエクスポート
  - LuaDocアノテーション（@module pasta.shiori.event.register）を追加
  - lua-coding.md の規約に準拠（UPPER_CASE、モジュール構造）
  - _Requirements: 1.1, 1.2, 1.5_

- [ ] 1.2 (P) REGテーブルの型定義とドキュメントを追加
  - @type table<string, fun(req: table): string> アノテーション
  - 使用例をコメントに記載（REG.EventName = function(req) ... end パターン）
  - _Requirements: 1.3, 1.4_

- [ ] 2. イベント振り分けモジュールを実装
- [ ] 2.1 init.lua の基本構造とrequire文を作成
  - pasta.shiori.event.register をREGとしてrequire
  - pasta.shiori.res をRESとしてrequire
  - EVENTテーブルを宣言
  - LuaDocアノテーション（@module pasta.shiori.event）を追加
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [ ] 2.2 デフォルトハンドラ（EVENT.no_entry）を実装
  - RES.no_content() を返すシンプルな実装
  - LuaDocアノテーション（@param, @return）を追加
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 2.3 イベント振り分け関数（EVENT.fire）を実装
  - REG[req.id] でハンドラを検索、未登録時はEVENT.no_entryにフォールバック
  - req.id が nil の場合も EVENT.no_entry にフォールバック（Lua標準挙動）
  - LuaDocアノテーション（@param req table, @return string）を追加
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

- [ ] 2.4 xpcallによるエラーハンドリングを実装
  - ハンドラ実行を xpcall でラップ（debug.traceback をエラーハンドラに指定）
  - 成功時はハンドラの結果をそのまま返却
  - エラー時は traceback の最初の行を抽出（result:match("^[^\n]+")）
  - 抽出したエラーメッセージ（nil の可能性あり）を RES.err() に渡す（nil 防御は RES.err 側で実施）
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

- [ ] 2.5 ハンドラシグネチャのドキュメント化
  - モジュールヘッダーコメントに期待されるハンドラシグネチャを記載（function(req) -> string）
  - reqテーブルの構造をドキュメント化（id, method, version, charset, sender, reference, dic）
  - reqテーブルがハンドラ内で変更されないことを明記（read-only契約）
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 7.1, 7.2, 7.3, 7.4_

- [ ] 2.6 モジュール公開APIの最終確認
  - EVENT.fire と EVENT.no_entry のみが公開関数であることを確認
  - 内部実装詳細が公開されていないことを確認
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [ ] 2.7 使用例ドキュメントを追加
  - ハンドラ登録の例を追加（REG.OnBoot, REG.OnClose など）
  - EVENT.fire(req) の呼び出し例を追加
  - テスト用 req テーブルの最小構造例を追加
  - Rust側統合が別途行われることを明記
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

### Phase 2: Testing

- [ ] 3. Unit テストを実装
- [ ] 3.1 (P) REGモジュールのテストを作成
  - shiori_event_test.rs に create_runtime_with_pasta_path() ヘルパーを追加
  - REGが空テーブルであることを検証（test_reg_module_exports_empty_table）
  - _Requirements: 1.1_

- [ ] 3.2 (P) EVENT.fire のハンドラディスパッチテストを作成
  - 登録済みハンドラが正しく呼び出されることを検証（test_event_fire_dispatches_registered_handler）
  - ハンドラの戻り値がそのまま返されることを検証
  - _Requirements: 4.2, 4.3_

- [ ] 3.3 (P) 未登録イベント処理のテストを作成
  - 未登録イベントで204レスポンスを返すことを検証（test_event_fire_returns_no_content_for_unregistered）
  - req.id=nil の場合も204を返すことを検証（test_event_fire_handles_nil_id）
  - _Requirements: 4.4, 4.5_

- [ ] 3.4 (P) エラーハンドリングのテストを作成
  - ハンドラ内エラーで500レスポンスを返すことを検証（test_event_fire_catches_handler_error）
  - エラーメッセージに改行が含まれないことを検証（test_error_message_no_newline）
  - 空エラーメッセージ時に "Unknown error" にフォールバックすることを検証（test_event_fire_handles_empty_error_message）
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [ ] 4. Integration テストを実装
- [ ] 4.1 RESモジュールとの統合テストを作成
  - EVENT.fire が RES.ok, RES.no_content, RES.err を正しく使用することを検証（test_event_module_with_res_module）
  - _Requirements: 3.2, 5.4_

- [ ] 4.2 ハンドラ登録から呼び出しまでの完全フローテストを作成
  - 複数ハンドラ登録 → EVENT.fire 呼び出し → 正しいハンドラが呼ばれることを検証（test_handler_registration_and_dispatch）
  - _Requirements: 1.3, 4.1, 4.2, 4.3_

### Phase 3: Integration & Validation

- [ ] 5. Rust側統合の準備確認
  - main.lua での EVENT.fire(req) 統合パターンが文書化されていることを確認
  - parse_request() が生成する req テーブル構造が文書と一致することを確認
  - SHIORI.request → EVENT.fire のディスパッチフローが設計通りであることを確認
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 9.3_

---

## Requirements Coverage

| Requirement | Tasks |
|-------------|-------|
| 1.1 - 1.5 | 1.1, 1.2, 3.1 |
| 2.1 - 2.5 | 2.1 |
| 3.1 - 3.3 | 2.2, 4.1 |
| 4.1 - 4.6 | 2.3, 3.2, 3.3, 4.2 |
| 5.1 - 5.6 | 2.4, 3.4, 4.1 |
| 6.1 - 6.4 | 2.5 |
| 7.1 - 7.4 | 2.5, 5 |
| 8.1 - 8.4 | 2.6 |
| 9.1 - 9.4 | 2.7, 5 |

**Total Coverage**: 9 requirements → 18 sub-tasks
