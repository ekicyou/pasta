# Research & Design Decisions

---
**Feature**: `pasta-label-continuation`  
**Discovery Scope**: Extension (Parser Layer)  
**Key Findings**:
- パーサー層のみの修正で実現可能（トランスパイラ・ランタイム層への影響なし）
- 既存の文法定義とAST構造を最小限の変更で拡張可能
- 重複シーン管理機構（SceneRegistry）をそのまま再利用できる

---

## Summary
無名「＊」行でシーン名を省略できる構文糖衣を実現するための軽量な拡張機能。パーサー層でファイル内コンテキストを保持し、AST生成時に明示的なシーン名へ正規化することで、後段のトランスパイラ・ランタイム層は従来通り「同名シーンが複数ある」ケースとして扱える。

## Research Log

### Parser Layer Extension Points
- **Context**: パーサー層でどこまで実装可能か、後段への影響範囲を特定
- **Sources Consulted**: 
  - `src/parser/pasta.pest` (文法定義)
  - `src/parser/mod.rs` (parse_str, parse_global_scene関数)
  - `src/parser/ast.rs` (SceneDef構造体)
- **Findings**:
  - `parse_str`関数はファイル全体を一度にパースし、トップレベル要素を順次処理
  - `parse_global_scene`関数は個別のグローバルシーン定義をパースし`SceneDef`を生成
  - 現在は`label_name`が必須（`global_label`文法で要求）
  - ファイル内の処理は線形であり、直近のシーン名をトラッキング可能
- **Implications**:
  - `parse_str`関数内でファイルスコープの`last_global_scene_name: Option<String>`を保持
  - `global_label`文法を`global_label_marker ~ label_name?`に変更してオプション化
  - `parse_global_scene`にコンテキスト（直近シーン名）を渡し、無名時は補完
  - 最初の無名「＊」は`PastaError::ParseError`で早期検出

### Transpiler & Runtime Impact Analysis
- **Context**: 後段レイヤーへの影響を最小化できるか
- **Sources Consulted**:
  - `src/transpiler/scene_registry.rs` (Pass1/Pass2, SceneRegistry)
  - `src/runtime/scene.rs` (SceneTable, 前方一致検索)
  - `.kiro/specs/pasta-label-continuation/gap-analysis.md`
- **Findings**:
  - SceneRegistryは既に同名シーンを連番で登録する仕組みを持つ
  - ランタイムのSceneTableは前方一致検索とランダム選択のみを担当
  - パーサーでAST生成時にシーン名を補完すれば、後段は変更不要
- **Implications**:
  - トランスパイラは「同名シーンが複数ある」通常ケースとして処理
  - ランタイムレイヤーは一切変更不要
  - テスト戦略はパーサー単体テストに集中可能

### Grammar Extension Safety
- **Context**: 文法拡張がPestパーサーに与える影響を検証
- **Sources Consulted**:
  - Pest 2.8公式ドキュメント（PEG文法、オプション記法）
  - `src/parser/pasta.pest`既存文法
- **Findings**:
  - Pestでは`rule?`でオプショナル要素を定義可能
  - 既存の`global_label`はsilent複合ルール（`$`記法）で定義
  - `label_name?`に変更しても、他のルールへの影響はない
  - コメント行、空行、単語定義はトップレベル要素として独立
- **Implications**:
  - 文法変更は後方互換性を維持（既存スクリプトは引き続き有効）
  - 新機能は追加のみで破壊的変更なし
  - パーサーテストで既存・新規両方のケースを網羅

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Parser-Only | パーサー層で無名シーン名を補完し、ASTには明示名のみ格納 | 後段レイヤー無変更、既存重複シーン機構を再利用 | パーサーにファイルスコープの状態が必要 | 推奨：構文糖衣の正規化に最適 |
| Pre-parse Pass | パース前に無名「＊」を明示名に置換する前処理 | パーサーロジックを変更しない | ファイル全体を2回読む、行番号ずれのリスク | 非推奨：パフォーマンス劣化 |
| Hybrid (Transpiler) | パーサーはプレースホルダーを格納、トランスパイラで名前解決 | パーサーは文法の忠実な表現を保持 | トランスパイラに複雑性追加、エラー報告が遅延 | 非推奨：責務分離が不明瞭 |

**選定**: Parser-Only アプローチ（要件との整合性、既存設計への影響最小化）

## Design Decisions

### Decision: Parser Layer Context Tracking
- **Context**: 無名「＊」行で直近グローバルシーン名を継続利用するため、パーサーがファイル内コンテキストを保持する必要がある
- **Alternatives Considered**:
  1. 関数引数で`last_scene_name: &mut Option<String>`を渡す — 明示的だがシグネチャ変更
  2. `parse_str`内でローカル変数として保持 — 最小限の変更
  3. パーサー構造体にフィールド追加 — 再利用性高いが過剰設計
- **Selected Approach**: `parse_str`関数内でローカル変数`last_global_scene_name: Option<String>`を保持し、`parse_global_scene`呼び出し時に渡す
- **Rationale**: 
  - ファイル単位のパース処理であり、状態の寿命が明確
  - 既存のパーサーAPIシグネチャ（`parse_file`, `parse_str`）を維持
  - パーサー構造体は文法定義に専念、状態管理をミックスしない
- **Trade-offs**: `parse_global_scene`の引数が1つ増えるが、内部関数のため影響範囲は限定的
- **Follow-up**: 実装時に`parse_global_scene`を`parse_global_scene_with_context`として新規作成し、段階的移行を検討

### Decision: Error Reporting Strategy
- **Context**: ファイル先頭の無名「＊」や、名前付きシーン未出現時の無名「＊」を早期検出してエラー報告
- **Alternatives Considered**:
  1. 文法レベルで禁止（Pest制約） — 不可能（前方参照が必要）
  2. パース時にコンテキストを確認 — 推奨（行番号を正確に報告）
  3. トランスパイラで検出 — エラー報告が遅延
- **Selected Approach**: パース時に`last_global_scene_name`を検証し、`None`の場合は`PastaError::ParseError`を返す
- **Rationale**: 
  - 行番号と列番号を正確に報告可能（Pestの`Span`情報を活用）
  - ユーザーは即座にDSLファイルの問題箇所を特定できる
  - パーサーの責務（構文検証）に合致
- **Trade-offs**: パーサーロジックが若干複雑化するが、診断性向上のメリットが大
- **Follow-up**: テストケースで先頭無名「＊」、中間無名「＊」のエラーメッセージ品質を検証

### Decision: AST Normalization (Named Scenes Only)
- **Context**: AST（`SceneDef`）には常に明示的なシーン名を格納し、後段レイヤーから見て無名シーンは存在しない
- **Alternatives Considered**:
  1. ASTにプレースホルダー（`Option<String>`）を格納 — トランスパイラに責務移譲
  2. ASTに常に名前を格納（正規化） — 推奨
- **Selected Approach**: パーサーが無名時にシーン名を補完し、ASTには常に`String`を格納
- **Rationale**:
  - トランスパイラ・ランタイムは既存の重複シーン処理ロジックをそのまま利用
  - AST構造体（`SceneDef`）の変更不要
  - 「構文糖衣」の概念をパーサー層に封じ込め
- **Trade-offs**: パーサーが若干複雑化するが、後段レイヤーの単純性を維持
- **Follow-up**: ASTダンプ（デバッグ出力）で無名「＊」が正しく補完されていることを確認

## Risks & Mitigations
- **Risk 1**: ファイル内の処理順序が変更された場合、コンテキストトラッキングが破綻する可能性  
  **Mitigation**: 現在のPest文法は行単位・トップダウン処理であり、仕様変更がない限りリスクは低い。将来的にパーサー再設計時は要注意。
- **Risk 2**: 無名「＊」行にコメントや他の要素が混在した場合の文法曖昧性  
  **Mitigation**: 要件4でフォーマット制約を明確化（空白のみ許容、コメント不可）。Pest文法で厳密に定義。
- **Risk 3**: エラーメッセージがユーザーに分かりにくい可能性  
  **Mitigation**: 実装時に「直前に名前付きグローバルシーンが存在しません」など、具体的な指示を含むメッセージを用意。

## References
- [Pest 2.8 Documentation](https://pest.rs/) — PEG文法、オプション記法、エラーハンドリング
- `.kiro/steering/tech.md` — パーサー・トランスパイラ・ランタイムのレイヤー構成
- `.kiro/specs/pasta-label-continuation/gap-analysis.md` — 実装オプション比較
- `src/parser/pasta.pest` — 現在の文法定義
- `src/parser/mod.rs` — 現在のパーサー実装
