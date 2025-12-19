# Research & Design Decisions

## Summary
- **Feature**: `pasta-grammar-specification`
- **Discovery Scope**: Extension（既存パーサー・トランスパイラーの破壊的変更を伴う拡張）
- **Key Findings**:
  1. Sakura スクリプトは「字句のみ認識、非解釈」で確定。半角 `\` + ASCII トークン + 非ネスト `[...]`（`\]` エスケープ許容）
  2. Jump マーカー（`？`）は廃止、Call（`＞`）へ統一
  3. 全角文字（`＼` `［］`）は pest 定義から完全削除

---

## Research Log

### ukadoc さくらスクリプトのエスケープ規則
- **Context**: Pasta パーサーでさくらスクリプトをどこまで解釈すべきかの調査
- **Sources Consulted**: 
  - https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html#notes_escape
- **Findings**:
  - `\\` でバックスラッシュをリテラル出力
  - `\%` で環境変数タグの `%` をリテラル出力
  - `\]` で角括弧引数内の `]` をリテラル出力（`\q[...]`, `\![...]` 等）
  - 複数引数の第2引数以降で `,` を含めるには `"..."` で囲む
  - `"` を含めるには `""` と二重にする
- **Implications**: Pasta パーサーは角括弧内容を解釈せず、`\]` のみ字句的に許容すれば十分

### Jump と Call のセマンティクス差異
- **Context**: Jump マーカー（`？`）を廃止すべきか、維持すべきか
- **Sources Consulted**: 
  - grammar-specification.md 4章
  - gap-analysis-2025-12-19.md
- **Findings**:
  - Jump と Call にセマンティクス上の差異なし（同一動作）
  - DSL レベルで区別する必要性がない
  - トランスパイラでは両方とも同じ Rune コード生成
- **Implications**: Jump 廃止は DSL 整理として妥当。破壊的変更だが MVP 前段階なので許容範囲

### 全角文字サポートの範囲
- **Context**: Sakura エスケープで全角 `＼` `［］` をサポートすべきか
- **Sources Consulted**: 
  - ukadoc（さくらスクリプトエスケープ）
  - grammar-specification.md 11.16
- **Findings**:
  - ukadoc では全角バックスラッシュのエスケープ定義なし
  - さくらスクリプトは ASCII コマンド前提
  - 全角括弧 `［］` もコマンド構文として定義なし
- **Implications**: 全角文字は pest 定義から削除（Case A 決定）

### テスト層別化と Cargo 互換性
- **Context**: テストファイルリネームが Cargo テスト実行に影響するか
- **Sources Consulted**: 
  - Cargo ドキュメント（tests/ ディレクトリ自動検出）
- **Findings**:
  - Cargo は `tests/**/*.rs` を自動検出・テスト対象化
  - 命名規則（`pasta_parser_*_test.rs` 等）は検出を阻害しない
  - tests/common モジュール参照も影響なし
- **Implications**: Phase 0 テスト層別化は安全に実行可能

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 層単位段階化 | Parser → Transpiler → Runtime/Tests の順で修正 | リグレッション発生箇所特定容易 | 各フェーズ間で依存関係 | 採用決定 |
| 一括修正 | 全層同時修正 | 修正完了が早い可能性 | リグレッション発生時の原因特定困難 | 却下 |
| 機能単位段階化 | Sakura → Jump → 全角の順で修正 | 機能ごとに完結 | 同一ファイルを複数回修正、混乱の元 | 却下 |

---

## Design Decisions

### Decision: Sakura コマンド簡素化（案 B）
- **Context**: pest 定義の詳細5パターンを維持するか簡素化するか
- **Alternatives Considered**:
  1. 案 A — 詳細5パターン維持、半角・`\]` 対応のみ追加
  2. 案 B — 完全簡素化（ASCIIトークン + 非ネスト `[...]` + `\]` 許容）
- **Selected Approach**: 案 B（完全簡素化）
- **Rationale**: 
  - 仕様「Sakura は字句のみ認識、非解釈」に忠実
  - 詳細パターン区別は実装上不要
  - 未知トークンを通すことで拡張性確保
- **Trade-offs**: 既存の詳細パターンテストは削除が必要
- **Follow-up**: Phase 1 で pest ルール修正

### Decision: Jump マーカー廃止（案 A）
- **Context**: Jump（`？`）を維持するか廃止するか
- **Alternatives Considered**:
  1. 案 A — Jump 廃止、Call（`＞`）へ統一
  2. 案 B — Jump 維持
- **Selected Approach**: 案 A（Jump 廃止）
- **Rationale**: 
  - MVP 未達段階での積極的な破壊的変更
  - セマンティクス上の差異なし
  - DSL 整理・保守性向上
- **Trade-offs**: Jump 依存テスト全削除が必要
- **Follow-up**: Phase 1-3 で Jump 関連コード・テスト削除

### Decision: 全角文字完全削除（Case A）
- **Context**: Sakura エスケープで全角 `＼` `［］` をサポートするか
- **Alternatives Considered**:
  1. Case A — 全角完全削除（pest で reject）
  2. Case B — 全角許容・非推奨（grammar では半角例のみ）
- **Selected Approach**: Case A（全角完全削除）
- **Rationale**: 
  - ukadoc に全角エスケープ定義なし
  - さくらスクリプトは ASCII コマンド前提
- **Trade-offs**: 既存全角スクリプトは半角への移行必須
- **Follow-up**: Phase 1 で pest ルール修正、Phase 3 でテスト置換

---

## Risks & Mitigations
- **Phase 1 ast.rs 型変更の波及** — Transpiler で compiler error として即座に検出
- **Jump 削除漏れ** — grep で Jump 関連コード検索、チェックリスト活用
- **テスト置換ミス** — Phase 0 の test-baseline.log と比較
- **全角テスト削除漏れ** — `grep -r "＼" tests/` で確認

---

## References
- [ukadoc - さくらスクリプトのエスケープ](https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html#notes_escape)
- grammar-specification.md（本仕様の文法仕様書）
- gap-analysis-2025-12-19.md（層別ギャップ分析）
- test-hierarchy-plan.md（テスト層別化計画）
