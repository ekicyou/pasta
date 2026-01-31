# Implementation Plan

## タスク概要

OnSecondChangeベースのOnTalk/OnHour仮想イベント発行機構を実装する。モジュールローカル変数で状態管理、pasta.toml設定読み込み、時刻判定ロジックを含む。

---

## 実装タスク

- [ ] 1. (P) virtual_dispatcher モジュールの実装
- [ ] 1.1 (P) モジュール構造とローカル変数の定義
  - `next_hour_unix`, `next_talk_time`, `cached_config`をモジュールローカル変数として宣言
  - 初期値を設定（0, 0, nil）
  - テスト用関数`_reset()`と`_get_internal_state()`を実装
  - _Requirements: 3.1_

- [ ] 1.2 (P) 設定読み込み機能の実装
  - `@pasta_config`から`[ghost]`セクション読み込み
  - `talk_interval_min`, `talk_interval_max`, `hour_margin`のデフォルト値処理
  - 設定キャッシュ機構
  - _Requirements: 5.1, 5.2, 5.3_

- [ ] 1.3 時刻計算ヘルパー関数の実装
  - 次の正時タイムスタンプ計算（`calculate_next_hour_unix`）
  - 次回トーク時刻計算（`calculate_next_talk_time`）
  - ランダム間隔生成（`math.random`使用）
  - _Requirements: 2.3, 5.3_

- [ ] 1.4 シーン関数実行機能の実装
  - `SCENE.search()`でシーン関数検索
  - `pcall`によるエラーハンドリング
  - エラー時のログ出力（既存EVENT.no_entry()パターン準拠）
  - _Requirements: 1.5, 2.5_

- [ ] 1.5 OnHour判定・発行ロジックの実装
  - 初回起動時の次回正時初期化
  - 正時到達判定（`req.date.unix >= next_hour_unix`）
  - トーク中スキップ（`req.status == "talking"`）
  - 次回正時更新
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 4.1, 4.3_

- [ ] 1.6 OnTalk判定・発行ロジックの実装
  - 初回起動時の次回トーク時刻初期化
  - トーク時刻到達判定
  - 時報前マージンチェック
  - 次回トーク時刻再計算（発行成否に関わらず）
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 4.1, 4.3_

- [ ] 1.7 dispatchメインエントリの実装
  - `req.date`存在チェック
  - OnHour判定（優先実行）
  - OnTalk判定（OnHour発行なしの場合のみ）
  - 戻り値制御（"fired" or nil）
  - _Requirements: 1.1, 2.1, 2.8, 4.1_

- [ ] 2. (P) second_change デフォルトハンドラの実装
- [ ] 2.1 (P) OnSecondChangeハンドラの登録
  - `REG.OnSecondChange`関数定義
  - `virtual_dispatcher.dispatch(req)`呼び出し
  - 戻り値に応じた`RES.no_content()`返却（alpha01仕様）
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [ ] 3. (P) init.lua統合の実装
- [ ] 3.1 (P) second_change.luaのロード追加
  - `require("pasta.shiori.event.second_change")`をinit.luaに追加
  - 既存boot.luaロード後に配置
  - _Requirements: 6.5_

- [ ] 4. テスト実装（Rustユニットテスト）
- [ ] 4.1 (P) virtual_event_dispatcher_test.rsファイル作成
  - テストフィクスチャ準備
  - 共通ヘルパー関数実装
  - _Requirements: 7.1_

- [ ] 4.2 req.date不在時のエラーハンドリングテスト
  - `test_dispatch_without_req_date`実装
  - nil返却の検証
  - _Requirements: 7.4_

- [ ] 4.3 OnHour発行ロジックのテスト
  - `test_onhour_first_run_skip`: 初回起動時スキップ検証
  - `test_onhour_fires_at_hour`: 正時超過時発行検証
  - `test_onhour_priority_over_ontalk`: 優先度検証
  - _Requirements: 7.1, 7.2, 7.3_

- [ ] 4.4 OnTalk発行ロジックのテスト
  - `test_ontalk_interval_check`: interval経過前スキップ検証
  - `test_ontalk_fires_after_interval`: interval経過後発行検証
  - `test_ontalk_hour_margin_skip`: 時報前マージンスキップ検証
  - _Requirements: 7.1, 7.2_

- [ ] 4.5 設定読み込みとトーク中判定のテスト
  - `test_config_default_values`: デフォルト値フォールバック検証
  - `test_skip_when_talking`: `req.status == "talking"`スキップ検証
  - _Requirements: 7.3_

- [ ] 4.6 状態管理のテスト
  - `test_module_state_reset`: Lua VMリロード時の状態リセット検証
  - `_reset()`と`_get_internal_state()`を使用
  - _Requirements: 7.2_

- [ ] 5. Luaユニットテストの実装
- [ ] 5.1 (P) virtual_dispatcher_spec.lua作成
  - lua_testフレームワークでBDDスタイル記述
  - `before_each`で`dispatcher._reset()`呼び出し
  - dispatch, check_hour, check_talkの各機能検証
  - _Requirements: 7.1_

- [ ] 6. 統合テスト・リグレッション検証
- [ ] 6.1 既存shiori_event_testとの整合性確認
  - OnSecondChange統合テストとの競合確認
  - 既存テストケースが全パスすることを検証
  - _Requirements: 7.5_

- [ ] 7. ドキュメント整合性の確認と更新
  - SOUL.md - コアバリュー・設計原則との整合性確認
  - SPECIFICATION.md - 言語仕様の更新（該当なし）
  - GRAMMAR.md - 文法リファレンスの同期（該当なし）
  - TEST_COVERAGE.md - 新規テストのマッピング追加
  - crates/pasta_lua/LUA_API.md - virtual_dispatcherモジュールAPI追記
  - steering/* - 該当領域のステアリング更新確認
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

---

## タスク実行ガイド

### 並列実行可能タスク

`(P)`マークのタスクは並列実行可能：
- 1.1, 1.2: 独立したヘルパー機能
- 2.1: virtual_dispatcher APIのみ依存
- 3.1: init.lua変更のみ
- 4.1, 5.1: テストファイル作成

### 依存関係

- 1.3〜1.7: 1.1, 1.2完了後に実行
- 4.2〜4.6: 4.1完了後、1.1〜1.7完了後に実行
- 6.1: 全実装タスク完了後に実行
- 7: 全テスト合格後に実行

### 検証ポイント

- 各タスク完了後に`cargo test --workspace`を実行
- リグレッション0を維持
- エラーログ出力の動作確認
