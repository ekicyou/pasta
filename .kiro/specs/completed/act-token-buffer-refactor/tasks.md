# Implementation Plan

## Task Format Template

Use whichever pattern fits the work breakdown:

### Major task only
- [ ] {{NUMBER}}. {{TASK_DESCRIPTION}}{{PARALLEL_MARK}}
  - {{DETAIL_ITEM_1}} *(Include details only when needed. If the task stands alone, omit bullet items.)*
  - _Requirements: {{REQUIREMENT_IDS}}_

### Major + Sub-task structure
- [ ] {{MAJOR_NUMBER}}. {{MAJOR_TASK_SUMMARY}}
- [ ] {{MAJOR_NUMBER}}.{{SUB_NUMBER}} {{SUB_TASK_DESCRIPTION}}{{SUB_PARALLEL_MARK}}
  - {{DETAIL_ITEM_1}}
  - {{DETAIL_ITEM_2}}
  - _Requirements: {{REQUIREMENT_IDS}}_ *(IDs only; do not add descriptions or parentheses.)*

---

## Tasks

### 1. 親クラス (pasta.act) の拡張

- [x] 1.1 (P) UI操作トークン蓄積メソッドを追加
  - `ACT_IMPL.surface(self, id)` を実装し、`{ type = "surface", id = id }` トークンを蓄積
  - `ACT_IMPL.wait(self, ms)` を実装し、`{ type = "wait", ms = math.max(0, math.floor(ms or 0)) }` トークンを蓄積
  - `ACT_IMPL.newline(self, n)` を実装し、`{ type = "newline", n = n or 1 }` トークンを蓄積
  - `ACT_IMPL.clear(self)` を実装し、`{ type = "clear" }` トークンを蓄積
  - 全メソッドは `return self` でメソッドチェーンを維持
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 1.2 (P) スポット切り替え検出機能を追加
  - `ACT.new()` で `_current_spot = nil` を初期化
  - `ACT_IMPL.talk()` でスポット変更を検出し、`{ type = "spot_switch" }` トークンを `actor` トークン直後に挿入
  - スポット切り替え検出時に `self._current_spot` を新しいスポットIDに更新
  - `talk()` 内のスポットID取得は `actor.spot or 0` を使用
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 1.3 talk後の固定改行を除去
  - `ACT_IMPL.talk()` から固定改行 `\n` の自動挿入を削除
  - 改行は `newline()` 明示呼び出しでのみトークン化されることを確認
  - _Requirements: 5.1, 5.2_

- [x] 1.4 build()メソッドを新設
  - `ACT_IMPL.build(self)` を実装し、`self.token` を取得後 `self.token = {}` でリセット
  - `self.now_actor = nil` および `self._current_spot = nil` をリセット
  - トークン配列を返却
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [x] 1.5 yield()メソッドをbuild()呼び出しに統一
  - `ACT_IMPL.yield()` を `local result = self:build(); coroutine.yield(result); return self` にリファクタリング
  - yield時に `_current_spot` がリセットされることを確認
  - _Requirements: 7.5, 8.1, 8.2, 8.3, 2.5_

- [x] 1.6 end_action()メソッドを削除
  - `ACT_IMPL.end_action()` を完全削除
  - 公開API一覧から `end_action` への言及を削除
  - _Requirements: 11.1, 11.2_

### 2. sakura_builderモジュールの新設

- [x] 2.1 sakura_builderモジュール本体を実装
  - `pasta/shiori/sakura_builder.lua` を新規作成
  - `build(tokens, config)` 関数を実装し、トークン配列とconfigを受け取りさくらスクリプト文字列を返却
  - 各トークンタイプの変換ロジックを実装:
    - `talk`: エスケープ済みテキスト
    - `actor`: スポットタグ `\p[n]` (actor.spotから決定)
    - `spot_switch`: 段落区切り改行 `\n[percent]` (config.spot_switch_newlinesから計算)
    - `surface`: サーフェスタグ `\s[id]`
    - `wait`: 待機タグ `\w[ms]`
    - `newline`: 改行タグ `\n` × n回
    - `clear`: クリアタグ `\c`
    - `sakura_script`: そのまま出力（エスケープなし）
    - `yield`: 無視（出力対象外）
  - 出力文字列の末尾に `\e` を付与
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [x] 2.2 sakura_builderのヘルパー関数を実装
  - さくらスクリプトエスケープ処理 `escape_sakura(text)` を実装（`\` → `\\`、`%` → `%%`）
  - スポットID計算関数 `spot_to_id(actor)` を実装（`actor.spot or 0`）
  - スポットタグ生成関数 `spot_to_tag(actor)` を実装（`\p[spot_to_id(actor)]`）
  - _Requirements: 6.5, 6.6_

### 3. 子クラス (pasta.shiori.act) のリファクタリング

- [x] 3.1 トークン蓄積メソッドのオーバーライドを削除
  - `SHIORI_ACT_IMPL.talk()` を削除（親クラスのtalk()を継承）
  - `SHIORI_ACT_IMPL.surface()` を削除
  - `SHIORI_ACT_IMPL.wait()` を削除
  - `SHIORI_ACT_IMPL.newline()` を削除
  - `SHIORI_ACT_IMPL.clear()` を削除
  - `SHIORI_ACT_IMPL.reset()` を削除
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

- [x] 3.2 内部フィールドを削除
  - `_buffer` フィールドを完全削除
  - `_current_spot` フィールドを削除（親クラスが管理）
  - `SHIORI_ACT.new()` で `_spot_switch_newlines` 設定のみを保持
  - _Requirements: 4.1, 4.2, 4.3_

- [x] 3.3 SHIORI_ACT_IMPL.yield()を削除
  - `SHIORI_ACT_IMPL.yield()` メソッドを削除（親クラスのyield()を継承）
  - 親クラスの `yield()` が自動的に `build()` を呼び出すことを確認
  - _Requirements: 8.4, 8.5_

- [x] 3.4 SHIORI_ACT_IMPL.build()を実装
  - `SHIORI_ACT_IMPL.build(self)` を実装
  - `local token = ACT.IMPL.build(self)` で親のbuild()を呼び出しトークン配列を取得
  - `pasta.shiori.sakura_builder.build(token, { spot_switch_newlines = self._spot_switch_newlines })` で変換
  - 既に `\e` は sakura_builder が付与するため、追加の `\e` 付与は不要
  - 変換結果を返却
  - _Requirements: 9.1, 9.2, 9.3_

### 4. 既存APIの互換性検証

- [x] 4.1 公開メソッドシグネチャを検証
  - `talk`, `surface`, `wait`, `newline`, `clear`, `build`, `yield` が利用可能であることを確認（親クラス継承含む）
  - メソッドチェーン（`return self`）が全メソッドで維持されていることを確認
  - _Requirements: 10.1, 10.2_

- [x] 4.2 特殊メソッドの互換性を検証
  - `transfer_date_to_var()` メソッドの動作が変更されていないことを確認
  - アクタープロキシ経由のメソッド呼び出し（`act.sakura:talk("Hello")`）が引き続きサポートされることを確認
  - _Requirements: 10.3, 10.4_

- [x] 4.3 互換性例外の文書化を検証
  - `end_action()` 削除が互換性維持の例外として明記されていることを確認（requirements.md, design.md）
  - _Requirements: 10.5_

### 5. テストスイートの作成

- [x] 5.1 (P) 親クラス (pasta.act) のユニットテスト
  - UI操作メソッド（surface/wait/newline/clear）がトークン配列に正しく蓄積されることを検証
  - スポット切り替え検出ロジックが正しく動作することを検証（`_current_spot` の追跡、spot_switchトークン挿入）
  - talk後の固定改行が除去されていることを検証
  - build()がトークン取得＋リセットを正しく実行することを検証
  - yield()がbuild()を呼び出し、結果をcoroutine.yieldすることを検証
  - end_action()が存在しないことを検証
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 5.1, 5.2, 7.1, 7.2, 7.3, 7.4, 7.5, 8.1, 8.2, 8.3, 11.1, 11.2_

- [x] 5.2 (P) sakura_builderモジュールのユニットテスト
  - 各トークンタイプ（talk/actor/spot_switch/surface/wait/newline/clear/sakura_script/yield）が正しく変換されることを検証
  - さくらスクリプトエスケープ（`\` → `\\`、`%` → `%%`）が正しく動作することを検証
  - 出力文字列の末尾に `\e` が付与されることを検証
  - ヘルパー関数（escape_sakura/spot_to_id/spot_to_tag）の動作を検証
  - configの `spot_switch_newlines` 設定が正しく反映されることを検証
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6_

- [x] 5.3 (P) 子クラス (pasta.shiori.act) のユニットテスト
  - 削除されたメソッド（talk/surface/wait/newline/clear/reset/yield）が存在しないことを検証
  - 親クラスから継承したメソッドが正しく動作することを検証
  - 削除されたフィールド（_buffer/_current_spot）が存在しないことを検証
  - build()が親のbuild()を呼び出し、sakura_builderで変換することを検証
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 4.1, 4.2, 4.3, 8.4, 8.5, 9.1, 9.2, 9.3_

- [x] 5.4 統合テスト: トークン蓄積からさくらスクリプト生成まで
  - アクタープロキシ経由のメソッド呼び出し（`act.sakura:talk("Hello")`）を含むE2Eシナリオを実行
  - スポット切り替えを含む複雑なシナリオでさくらスクリプトが正しく生成されることを検証
  - メソッドチェーン（`act:talk("A"):surface(1):wait(100):newline():clear():yield()`）が正しく動作することを検証
  - transfer_date_to_var()メソッドが引き続き正常に動作することを検証
  - _Requirements: 10.1, 10.2, 10.3, 10.4_

- [x] 5.5* (P) 受け入れ基準カバレッジの確認
  - requirements.md の全11要件（1.1-11.2）に対応するテストが作成されていることを確認
  - 各受け入れ基準がテストケースで検証されていることをTEST_COVERAGE.mdに記録
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 4.1, 4.2, 4.3, 5.1, 5.2, 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 7.1, 7.2, 7.3, 7.4, 7.5, 8.1, 8.2, 8.3, 8.4, 8.5, 9.1, 9.2, 9.3, 10.1, 10.2, 10.3, 10.4, 10.5, 11.1, 11.2_

### 6. ドキュメント整合性の確認と更新

- [x] 6.1 SOUL.mdとの整合性確認
  - コアバリュー（日本語フレンドリー、UNICODE識別子、yield型、宣言的フロー）への影響を確認
  - 設計原則（行指向文法、前方一致、UI独立性）への影響を確認
  - Phase 0完了基準（DoD）への影響を確認

- [x] 6.2 SPECIFICATION.md/GRAMMAR.mdの更新（該当する場合）
  - 言語仕様への影響がないことを確認（本リファクタリングは内部実装のみ）

- [x] 6.3 TEST_COVERAGE.mdの更新
  - 新規テスト（5.1-5.5）のマッピングを追加
  - 削除されたAPIのテストカバレッジ状況を更新

- [x] 6.4 クレートREADME/steering/*の更新（該当する場合）
  - pasta_luaクレートのREADMEにsakura_builderモジュール追加を記載
  - lua-coding.mdの整合性を確認（MODULE/MODULE_IMPL分離パターン準拠）
  - structure.mdのモジュール構成図を更新（sakura_builder追加）

---

## 完了条件

すべてのタスクが完了し、以下が満たされること：

1. **Spec Gate**: 全フェーズ承認済み
2. **Test Gate**: `cargo test --all` 成功（pasta_luaクレートのテスト含む）
3. **Doc Gate**: SOUL.md、TEST_COVERAGE.md、クレートREADME、steering/* の更新完了
4. **Steering Gate**: lua-coding.md、structure.md との整合性確認完了
5. **Soul Gate**: SOUL.mdのコアバリュー・設計原則との整合性確認完了

---

## 注意事項

- **並列実行可能タスク**: `(P)` マーク付きタスクは並列実行可能
- **オプショナルタスク**: `*` マーク付きタスク（5.5）は後回し可能だが、MVP完了前に実施推奨
- **テストファースト**: 実装前にテストケースを明確化し、TDD推奨
- **コミット粒度**: 各メジャータスク（1-6）完了時にコミット推奨
