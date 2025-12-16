# Grammar Tests - Deleted

**削除日**: 2025-12-13  
**削除理由**:

1. **pest文法の構造変化**
   - 旧: 個別のRule（`speech_line`, `call_stmt`, `jump_stmt`）
   - 新: 統合されたRule（`label_body_line`）
   - 多くのRuleが単独ではパース不可能

2. **テストの目的と代替**
   - 目的: pest文法の単体テスト
   - 代替: 統合テスト（`test_comprehensive_control_flow_transpile`）で文法を網羅的にテスト
   - パーサーの正確性は既存の56+テストで保証済み

3. **保守コスト**
   - 現在の文法構造に合わせた書き直しが必要
   - 統合テストで同等の保証が得られる

**残存する文法テスト**:
- `grammar_diagnostic.rs`: 低レベルのpest文法要素（引数リスト、識別子等）のテスト（有効）

元ファイル: grammar_tests.rs（再作成版）
