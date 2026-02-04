# 実装計画

## タスク一覧

### 1. ACT:build()早期リターン実装

- [ ] 1.1 (P) ACT:build()にトークン0件検証を追加
  - `crates/pasta_lua/scripts/pasta/act.lua`のACT_IMPL.build()メソッドにトークン数検証ロジック追加
  - トークン取得後、`#tokens == 0`条件でnilリターン
  - トークンリセット（`self.token = {}`）は検証前に実施し、状態の一貫性を維持
  - _Requirements: 1.1_

- [ ] 1.2 (P) ACT:build()の型アノテーション更新
  - `crates/pasta_lua/scripts/pasta/act.lua`のACT_IMPL.build()の`@return`アノテーションを`table[]|nil`に変更
  - LuaLS形式（`--- @return table[]|nil`）で型安全性を確保
  - _Requirements: 1.1, 2.2_

### 2. SHIORI_ACT:build()早期リターン実装

- [ ] 2.1 (P) SHIORI_ACT:build()にnil検証を追加
  - `crates/pasta_lua/scripts/pasta/shiori/act.lua`のSHIORI_ACT_IMPL.build()メソッドにnil検証ロジック追加
  - ACT.IMPL.build(self)呼び出し後、`token == nil`条件でnilリターン
  - nil以外の場合は既存のBUILDER.build()処理を継続
  - _Requirements: 1.2_

- [ ] 2.2 (P) SHIORI_ACT:build()の型アノテーション更新
  - `crates/pasta_lua/scripts/pasta/shiori/act.lua`のSHIORI_ACT_IMPL.build()の`@return`アノテーションを`string|nil`に変更
  - LuaLS形式（`--- @return string|nil`）で型安全性を確保
  - _Requirements: 1.2, 2.2_

### 3. テスト実装

- [ ] 3.1 (P) ACT:build() nilリターンテスト追加
  - `crates/pasta_lua/tests/lua_specs/`に新規テストファイル作成（BDDスタイル）
  - トークン0件時にnilを返すテストケース
  - nilリターン後に`self.token`が空テーブルであることを検証
  - トークン1件以上の場合に配列が返されることを検証（既存動作継続確認）
  - _Requirements: 1.1_

- [ ] 3.2 (P) SHIORI_ACT:build() nilリターンテスト追加
  - `crates/pasta_lua/tests/lua_specs/`に新規テストファイル作成（BDDスタイル）
  - ACT.IMPL.build()がnilの場合にnilを返すテストケース
  - ACT.IMPL.build()が配列の場合に文字列を返すテストケース（既存動作継続確認）
  - BUILDER.build()がnilリターン時に呼び出されないことを検証（モックまたは呼び出し回数カウント）
  - _Requirements: 1.2, 2.1_

- [ ] 3.3 既存テストリグレッション確認
  - `cargo test --workspace`実行で全テストパスを確認
  - 既存のact.lua関連テストケース（act_test.lua相当）がパスすることを確認
  - 既存のshiori/act.lua関連テストケース（shiori_act_test.lua相当）がパスすることを確認
  - トークンありケースの既存動作が維持されていることを検証
  - _Requirements: 2.2_

### 4. ドキュメント更新

- [ ] 4.1 (P) init.luaドキュメント例更新
  - `crates/pasta_lua/scripts/pasta/init.lua`のL40ドキュメント例を更新
  - `act:build()`のnil処理パターンを追加（`if script == nil then return RES.no_content() end`）
  - シーン作者向けにnil検証の推奨実装例を提示
  - _Requirements: 1.3_

### 5. 統合検証

- [ ] 5.1 型アノテーション検証
  - LuaLSによる静的型チェック実行
  - ACT_IMPL.build()とSHIORI_ACT_IMPL.build()の両方で`table[]|nil`, `string|nil`型が検出されることを確認
  - 型アノテーション更新漏れがないことを検証
  - _Requirements: 2.2_

- [ ] 5.2 パフォーマンス最適化検証
  - トークン0件時のbuild()実行時間を測定
  - BUILDER.build()呼び出し回数をカウント（トークン0件時は0回）
  - group_by_actor()とmerge_consecutive_talks()のスキップを確認
  - _Requirements: 2.1_

## 要件カバレッジマトリクス

| 要件 | タスク             | 説明                             |
| ---- | ------------------ | -------------------------------- |
| 1.1  | 1.1, 1.2, 3.1      | ACT:build()早期リターン          |
| 1.2  | 2.1, 2.2, 3.2      | SHIORI_ACT:build()早期リターン   |
| 1.3  | 4.1                | 会話未作成検出可能性             |
| 2.1  | 5.2                | パフォーマンス最適化             |
| 2.2  | 1.2, 2.2, 3.3, 5.1 | テストリグレッション対応・型更新 |

## 実装順序の推奨

1. **Phase 1: コア実装** (タスク1.1, 1.2, 2.1, 2.2 - 並列実行可能)
   - ACT:build()とSHIORI_ACT:build()の早期リターンロジック実装
   - 型アノテーション更新

2. **Phase 2: テスト追加** (タスク3.1, 3.2 - 並列実行可能、Phase 1完了後)
   - 新規テストケース追加
   - 既存テスト継続パス確認（タスク3.3）

3. **Phase 3: ドキュメント・検証** (タスク4.1, 5.1, 5.2 - 並列実行可能、Phase 2完了後)
   - ドキュメント更新
   - 型検証・パフォーマンス検証

## 注意事項

- **最小変更原則**: 各ファイルへの変更は3行程度に限定（早期リターン条件追加のみ）
- **Lua コーディング規約準拠**: ドット構文 + 明示的self、LuaLS型アノテーション形式
- **テストフレームワーク**: lua_test（BDD）を使用、`describe`/`test`/`expect`構文
- **リグレッション確認**: 既存テスト40件以上が引き続きパスすることを必須確認
