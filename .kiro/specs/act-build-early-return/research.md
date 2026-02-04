# Research & Design Decisions

---
**Purpose**: ACT:build() / SHIORI_ACT:build()の早期リターンパターン実装に関する調査と設計判断を記録

**Usage**: 軽量Discovery（Extension）を実施し、既存パターンとの整合性を確保
---

## Summary
- **Feature**: `act-build-early-return`
- **Discovery Scope**: Extension（既存システムへの拡張）
- **Key Findings**:
  - 既存のbuild()メソッドは`table[]`または`string`を返す前提で設計されており、nil非対応
  - Lua型アノテーション（LuaLS形式）で`|nil`を追加することで型安全性を確保
  - 変更影響範囲は限定的（ACT/SHIORI_ACTの2ファイル + テスト + ドキュメント例）

## Research Log

### 既存型アノテーションパターンの調査
- **Context**: ACT:build()とSHIORI_ACT:build()の型アノテーションをnilサポートに更新する必要があるため、既存パターンを確認
- **Sources Consulted**: 
  - `crates/pasta_lua/scripts/pasta/act.lua` - ACT_IMPL.build()実装（L280-292）
  - `crates/pasta_lua/scripts/pasta/shiori/act.lua` - SHIORI_ACT_IMPL.build()実装（L56-63）
  - `crates/pasta_lua/scripts/pasta/scene.lua` - 既存の`string|nil`パターン（L101）
  - `crates/pasta_lua/scripts/pasta/actor.lua` - 既存の`string|nil`パターン（L123）
- **Findings**:
  - 既存の型アノテーション形式: `--- @return table[]`, `--- @return string`
  - 既にnilを返す可能性がある関数では`string|nil`形式を使用（scene.lua, actor.lua）
  - Luaコーディング規約（`.kiro/steering/lua-coding.md`）でLuaLS形式の型アノテーションを推奨
- **Implications**: 
  - `--- @return table[]|nil`, `--- @return string|nil`形式で型アノテーション更新
  - 既存パターンに完全準拠し、LuaLS型チェックで検出可能

### トークンカウント・リセットロジックの確認
- **Context**: R1で「nilリターン後もself.tokenは空テーブル{}にリセットする」と要求されているため、現在の実装を確認
- **Sources Consulted**: `crates/pasta_lua/scripts/pasta/act.lua` L282-292
- **Findings**:
  - 現在の実装: `local tokens = self.token; self.token = {}`（L283-284）
  - トークン取得と同時にリセットする設計（状態の一貫性維持）
  - グループ化処理前にリセットされるため、早期リターン時も同様にリセット必要
- **Implications**: 
  - 早期リターン時も`local tokens = self.token; self.token = {}; if #tokens == 0 then return nil end`の順序を維持
  - 状態管理の一貫性が保たれる

### BUILDER.build()の空配列処理コスト（軽量調査）
- **Context**: NFR1で「BUILDER.build()スキップによるメモリ削減」を挙げているため、空配列渡し時のコストを確認
- **Sources Consulted**: ギャップ分析（gap-analysis.md）、要件（requirements.md）
- **Findings**:
  - 空配列渡し時でもBUILDER.build()内部でループ処理が実行される可能性がある
  - 早期リターンにより、グループ化（group_by_actor）+ 統合（merge_consecutive_talks）+ BUILDER.build()の3処理をスキップ
  - 詳細なベンチマークは設計段階では不要（実装時のパフォーマンステストで確認）
- **Implications**: 
  - 設計判断: 早期リターンによるパフォーマンス最適化を採用
  - 実装時にAC2でBUILDER.build()呼び出し回数を検証

### 既存テストケースの非nil前提パターン
- **Context**: NFR2で「既存テスト40件以上が非nil前提」とあるため、影響範囲を確認
- **Sources Consulted**: 
  - `crates/pasta_lua/tests/lua_specs/act_test.lua` - ACT:build()テスト（L410-442）
  - `crates/pasta_lua/tests/lua_specs/shiori_act_test.lua` - SHIORI_ACT:build()テスト（L70-380）
- **Findings**:
  - act_test.lua: L438で`local token = act:build(); expect(type(token)):toBe("table")`（非nil前提）
  - shiori_act_test.lua: L70-380で22件のテストケース、全て戻り値がstringであることを前提
  - トークンありケースのテストは引き続きパス（nil返却条件に該当しない）
- **Implications**: 
  - 既存テストは修正不要（トークンありケースのみテスト）
  - 新規テストケース追加必要: 
    - act_test.lua: トークン0件時にnilリターン
    - shiori_act_test.lua: ACT.IMPL.build()がnilを返した場合のnilリターン

## Architecture Pattern Evaluation

| Option                           | Description                                                 | Strengths                                             | Risks / Limitations                 | Notes                                 |
| -------------------------------- | ----------------------------------------------------------- | ----------------------------------------------------- | ----------------------------------- | ------------------------------------- |
| Option A: 既存関数拡張           | ACT_IMPL.build()とSHIORI_ACT_IMPL.build()に早期リターン追加 | 最小変更、既存パターン踏襲、Single Responsibility維持 | テストリグレッション確認必要        | ✅ 推奨（gap-analysis.mdで評価済み）   |
| Option B: 新規コンポーネント作成 | nil対応版ラッパー（act_nil_safe.lua）作成                   | 後方互換性維持                                        | API 2系統共存、最終的に移行必須     | ❌ 非推奨（gap-analysis.mdで評価済み） |
| Option C: 段階的実装             | Phase 1: 早期リターン、Phase 2: nil処理、Phase 3: テスト    | 段階的動作確認                                        | 実装期間長期化、Phase間不整合リスク | ❌ 非推奨（Option Aと実質同じ）        |

## Design Decisions

### Decision: Option A（既存関数拡張）採用
- **Context**: ACT:build()とSHIORI_ACT:build()にnil早期リターンを導入し、撮影トークン0件時のパフォーマンス最適化を実現
- **Alternatives Considered**:
  1. Option A: 既存関数拡張（3行追加のみ）
  2. Option B: 新規コンポーネント作成（nil対応版ラッパー）
  3. Option C: 段階的実装（フェーズ分割）
- **Selected Approach**: Option A - 既存のACT_IMPL.build()とSHIORI_ACT_IMPL.build()に早期リターンロジックを追加
- **Rationale**: 
  - 最小変更で要件達成（2ファイル、各3行追加）
  - 既存パターンを踏襲（学習コスト低）
  - Single Responsibility Principle維持（早期リターンは責務に含まれる）
  - 実質的な後方互換性リスクなし（build()呼び出しロジック未実装、リスクはテストリグレッションのみ）
- **Trade-offs**: 
  - ✅ Benefits: 実装容易、保守性向上、既存アーキテクチャ準拠
  - ❌ Compromises: 既存テスト40件以上の影響確認必要、型アノテーション更新必須
- **Follow-up**: 実装時にcargo test --workspace実行でリグレッション確認

### Decision: 型アノテーション更新パターン
- **Context**: ACT:build()とSHIORI_ACT:build()の戻り値型をnilサポートに更新
- **Alternatives Considered**:
  1. `--- @return table[]|nil` / `--- @return string|nil`（LuaLS形式）
  2. ドキュメントコメントでnil可能性を記載（型アノテーション変更なし）
- **Selected Approach**: LuaLS形式の型アノテーション更新（`table[]|nil`, `string|nil`）
- **Rationale**: 
  - 既存の`string|nil`パターン（scene.lua, actor.lua）に準拠
  - LuaLS型チェックで検出可能（NFR2要件）
  - Luaコーディング規約（`.kiro/steering/lua-coding.md`）推奨形式
- **Trade-offs**: 
  - ✅ Benefits: 型安全性向上、静的解析サポート、既存パターン準拠
  - ❌ Compromises: 呼び出し元でnil検証が推奨される（将来的な設計ガイドライン）
- **Follow-up**: init.lua:40のドキュメント例を更新し、nil処理パターンを示す

### Decision: nil検証パターンの明示的記述
- **Context**: R2で「ACT.IMPL.build(self)の戻り値がnilであることを明示的に検証する（`== nil`）」と要求
- **Alternatives Considered**:
  1. `if token == nil`（明示的nil検証）
  2. `if not token`（Luaの慣用句、falseとnilを区別しない）
- **Selected Approach**: `if token == nil`（明示的nil検証）
- **Rationale**: 
  - 要件R2で明示的に指定
  - Lua型システムの制約（`.kiro/steering/lua-coding.md`）で`if not value`パターンはfalseとnilを区別できない
  - 意図の明確化（nilのみを検証、falseは許容しない設計）
- **Trade-offs**: 
  - ✅ Benefits: 意図明確、型安全性向上、Lua慣用句の落とし穴回避
  - ❌ Compromises: やや冗長（ただし明示的な方が保守性高い）
- **Follow-up**: 実装コードレビューで`== nil`パターンを確認

## Risks & Mitigations

- **Risk 1: 既存テストケースのリグレッション** → 実装前にcargo test --workspace実行、新規テストケース追加でカバレッジ維持
- **Risk 2: シーン関数でnil非対応のコードがクラッシュ** → ドキュメント（init.lua:40）でnil処理例を示し、将来的な設計ガイドラインとして記載（AC5）
- **Risk 3: 型アノテーション更新漏れ** → ACT_IMPL.build()とSHIORI_ACT_IMPL.build()の両方を確認、AC3でテスト

## References
- [.kiro/specs/act-build-early-return/requirements.md](../requirements.md) - 要件定義書
- [.kiro/specs/act-build-early-return/gap-analysis.md](../gap-analysis.md) - ギャップ分析
- [.kiro/steering/lua-coding.md](../../steering/lua-coding.md) - Luaコーディング規約
- [.kiro/steering/tech.md](../../steering/tech.md) - 技術スタック定義
- [crates/pasta_lua/scripts/pasta/act.lua](../../../crates/pasta_lua/scripts/pasta/act.lua) - ACT実装
- [crates/pasta_lua/scripts/pasta/shiori/act.lua](../../../crates/pasta_lua/scripts/pasta/shiori/act.lua) - SHIORI_ACT実装
