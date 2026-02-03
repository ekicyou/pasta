# Implementation Plan

## Task Breakdown

### Phase 1: グループ化機能の実装 ✅ COMPLETED

- [x] 1. pasta.actモジュールのグループ化機能実装
- [x] 1.1 (P) group_by_actor() ローカル関数の実装
  - フラットなトークン配列を受け取り、アクター切り替え境界でグループ化
  - spot/clear_spotは独立トークンとして出力
  - talkトークンのactor変化検出で新しいtype="actor"トークンを開始
  - 未知のトークン型に対するelse分岐を実装（デバッグ容易性のため）
  - トークン順序を保持しながら処理
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 2.4, 2.5, 5.1, 5.2, 7.3_
  - **実装完了**: `scripts/pasta/act.lua` lines 17-55

- [x] 1.2 (P) merge_consecutive_talks() ローカル関数の実装
  - type="actor"トークン内の連続talkトークンを統合
  - アクター行動トークン（surface, wait等）で分離
  - 最初のtalkのactor情報を保持
  - 新規テーブル作成で純粋関数として実装
  - spot/clear_spotトークンはそのまま出力
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 5.3, 5.4, 7.3_
  - **実装完了**: `scripts/pasta/act.lua` lines 60-105

- [x] 1.3 ACT_IMPL.build()の変更実装
  - group_by_actor()とmerge_consecutive_talks()を2段階で呼び出し
  - 戻り値型をtoken[]からgrouped_token[]に変更
  - self.tokenのリセット処理を維持
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4_
  - **実装完了**: `scripts/pasta/act.lua` ACT_IMPL.build()

- [x] 1.4* グループ化機能の単体テスト実装
  - lua_specs/act_grouping_test.luaを新規作成
  - group_by_actor()のテストケース（空配列、単一talk、spot独立、clear_spot独立、actor変更、断続的actor）
  - merge_consecutive_talks()のテストケース（連続talk結合、アクター行動分離、空文字列結合、spot/clear_spot保持）
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 5.1, 5.2, 5.3, 5.4_
  - **実装完了**: `tests/lua_specs/act_grouping_test.lua` (19テスト全パス)
  - **既存テスト対応完了**: `act_test.lua`, `shiori_act_test.lua`, `sakura_builder.lua` をグループ化形式に対応

### Phase 2: sakura_builder並行開発 ✅ COMPLETED (Phase 1で統合実装済み)

- [x] 2. BUILDER.build_grouped() 新規関数の実装
  - **注記**: 設計変更により、既存`BUILDER.build()`を直接グループ化対応に改修
- [x] 2.1 (P) BUILDER.build_grouped()の基本実装
  - grouped_token[]を受け取る新規関数を追加（既存build()は維持）
  - type="spot"処理でactor_spotsマップを更新
  - type="clear_spot"処理で状態をリセット
  - type="actor"処理で内部tokens配列を順次処理
  - アクター切り替え検出とspot_newlines段落改行の実装
  - _Requirements: 4.1, 4.2, 4.3, 4.4_
  - **実装完了**: `sakura_builder.lua` lines 50-123（新形式対応）

- [x] 2.2 (P) type="actor"内部トークンの処理実装
  - talk: escape_sakura()でエスケープして出力
  - surface: \s[id]形式で出力
  - wait: \w[ms]形式で出力
  - newline: \n出力（複数回対応）
  - clear: \c出力
  - sakura_script: 生テキスト出力
  - yield: 無視
  - _Requirements: 4.3, 4.4_
  - **実装完了**: `sakura_builder.lua` lines 103-122

- [x] 2.3 (P) 既存ヘルパー関数の再利用確認
  - escape_sakura()がそのまま使用可能であることを確認
  - spot_to_id()がそのまま使用可能であることを確認
  - spot_to_tag()がそのまま使用可能であることを確認
  - _Requirements: 4.1, 4.2, 4.3_
  - **確認完了**: 全ヘルパー関数がそのまま再利用可能

- [x] 2.4* build_grouped()の同等性確認テスト
  - lua_specs/sakura_builder_grouped_test.luaを新規作成
  - 既存テストシナリオをgrouped形式で再現
  - 既存build()の期待出力と同一であることを確認
  - _Requirements: 4.5, 6.1, 6.2_
  - **実装完了**: `act_grouping_test.lua`で統合テスト、全テストパスで同等性確認済み

### Phase 3: 統合と移行 ✅ COMPLETED

- [x] 3. SHIORI_ACT_IMPL.build()の統合実装
- [x] 3.1 SHIORI_ACT_IMPL.build()をbuild_grouped()使用に変更
  - ACT.IMPL.build(self)でgrouped_token[]を取得
  - BUILDER.build_grouped()に渡してさくらスクリプトを生成
  - spot_newlines設定を正しく渡す
  - _Requirements: 4.6, 4.7, 6.3_
  - **実装完了**: `shiori/act.lua`でグループ化対応済み

- [x] 3.2 既存テストによる後方互換性検証
  - 既存sakura_builder_test.lua（521行）を全実行
  - すべてのテストがパスすることを確認
  - 出力が既存と完全一致することを確認
  - _Requirements: 4.5, 6.1, 6.2_
  - **検証完了**: 17テストスイート全パス

- [x] 3.3* アクター切り替え検出の境界値テスト追加
  - spot後の最初のtalkでアクター切り替え検出を確認
  - clear_spot後の初回talkで状態リセットを確認
  - actor=nilのtalkでスポット切り替え挙動を確認
  - _Requirements: 4.4, 5.1_
  - **実装完了**: `act_grouping_test.lua`に境界値テスト含む（19テスト）

- [x] 3.4 レガシーコードのクリーンアップ（Phase 3最終）
  - sakura_builder.luaからレガシートークン処理を削除（171行→120行、約50行削減）
  - type="actor"レガシー形式（actorオブジェクトのspot参照）処理を削除
  - フラット形式トークン（talk, surface, wait, newline, clear, sakura_script）処理を削除
  - type="spot_switch"レガシートークン処理を削除
  - sakura_builder_test.lua（約520行）を全面書き直し（約420行）
    - レガシーテスト「actor token」（6テスト）を削除
    - レガシーテスト「spot_switch token」（3テスト）を削除
    - 全テストをグループ化形式（type="actor" with tokens[]）に変換
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_
  - **実装完了**: 17テストスイート全パス確認済み

### Phase 4: テストとドキュメント ✅ COMPLETED

- [x] 4. テスト修正とドキュメント更新
- [x] 4.1 sakura_builder_test.luaのテストケース修正
  - grouped形式入力に変更（約400行の修正）
  - 複合シナリオテストの更新
  - 統合シナリオテストの更新
  - _Requirements: 6.1, 6.2_
  - **実装完了**: 既存テストをグループ化形式に対応済み、全パス

- [x] 4.2 ドキュメント整合性の確認と更新
  - SOUL.md - コアバリュー・設計原則との整合性確認（yield型、宣言的フロー、UI独立性への影響なし）
  - TEST_COVERAGE.md - 新規テストのマッピング追加（act_grouping_test.lua追加済み）
  - クレートREADME - API変更の反映（pasta_lua/README.md: グループ化トークン形式の説明追加）
  - steering/tech.md - 実装パターン（2パス変換、Yield型）との整合性確認
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 5.1, 5.2, 5.3, 5.4, 6.1, 6.2, 6.3, 7.1, 7.2, 7.3_
  - **完了**: TEST_COVERAGE.md, pasta_lua/README.md 更新済み

## Task Execution Notes

### 並行実行可能なタスク

以下のタスクは(P)マークがあり、並行実行が可能：

- **Phase 1**: 1.1, 1.2は独立して実装可能
- **Phase 2**: 2.1, 2.2, 2.3はbuild_grouped()の異なる部分を実装するため並行可能
- **Phase 2**: 2.4はPhase 1完了後に実行可能

### 依存関係

- 1.3は1.1と1.2の完了後に実行
- Phase 2全体はPhase 1の完了を待つ必要はない（並行開発戦略）
- 3.1はPhase 1とPhase 2の完了が必要
- 3.2は3.1の完了が必要
- 3.4はすべてのテスト（3.2、3.3）がパスした後に実行

### 実装順序の推奨

1. Phase 1（グループ化機能）を完全に実装・テスト
2. Phase 2（build_grouped）を並行開発として実装
3. Phase 3で統合し、既存テストで検証
4. Phase 4でクリーンアップとドキュメント更新

## Requirements Coverage

全27件の受入基準をカバー：

- **R1** (グループ化後のトークン構造): 1.1, 1.2, 1.3, 1.4, 4.2
- **R2** (アクター切り替え境界): 1.1, 1.3, 1.4, 4.2
- **R3** (連続talk統合): 1.2, 1.3, 1.4, 4.2
- **R4** (sakura_builder対応): 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3, 3.4, 4.1, 4.2
- **R5** (エッジケース): 1.1, 1.2, 1.4, 3.3, 4.2
- **R6** (後方互換性): 2.4, 3.1, 3.2, 4.1, 4.2
- **R7** (将来拡張): 1.1, 1.2, 4.2

すべての要件が実装タスクにマッピングされています。
