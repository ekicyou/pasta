````markdown
# Research & Design Decisions

---
**Purpose**: 末尾再帰最適化機能の設計における調査結果と決定事項を記録

---

## Summary
- **Feature**: `lua_tail_recursion_optimization`
- **Discovery Scope**: Extension（既存システムへの小規模拡張）
- **Key Findings**:
  1. Lua の TCO は `return func()` 形式でのみ有効化される
  2. 現行の `LocalSceneItem` enum では `CallScene` のみが TCO 対象
  3. 末尾判定は `rposition` または `enumerate` + 最終インデックス比較で実現可能

## Research Log

### Lua 末尾呼び出し最適化（TCO）の仕様確認
- **Context**: Lua における TCO の発動条件を確認
- **Sources Consulted**: 
  - [Lua 5.4 Reference Manual - Proper Tail Calls](https://www.lua.org/manual/5.4/manual.html#3.4.10)
  - [Programming in Lua - Proper Tail Calls](https://www.lua.org/pil/6.3.html)
- **Findings**:
  - Lua は「proper tail calls（適切な末尾呼び出し）」をサポート
  - `return func()` の形式でのみ TCO が発動
  - `return func() + 1` や `func(); return` では TCO が無効
  - スタックフレームが再利用され、無限再帰でもスタックオーバーフローしない
- **Implications**: `act:call()` を末尾で呼び出す場合、`return act:call(...)` 形式で生成する必要がある

### LocalSceneItem enum の構造分析
- **Context**: 末尾最適化の対象となるバリアントを特定
- **Sources Consulted**: `crates/pasta_core/src/parser/ast.rs` (L435-L444)
- **Findings**:
  ```rust
  pub enum LocalSceneItem {
      VarSet(VarSet),           // 変数代入 → TCO 非対象
      CallScene(CallScene),     // シーン呼び出し → TCO 対象
      ActionLine(ActionLine),   // アクション行 → TCO 非対象（副作用）
      ContinueAction(ContinueAction), // 継続アクション → TCO 非対象（副作用）
  }
  ```
- **Implications**: 現時点では `CallScene` のみが TCO 対象。将来 `FnCall` などが追加される場合は `matches!` マクロで拡張可能

### 既存の generate_local_scene_items 実装分析
- **Context**: 現行ループ構造と変更ポイントを特定
- **Sources Consulted**: `crates/pasta_lua/src/code_generator.rs` (L288-L312)
- **Findings**:
  - 現行: `for item in items` による単純なイテレーション
  - 末尾判定ロジックなし
  - `generate_call_scene` は `is_tail_call` パラメータを持たない
- **Implications**: 
  - `enumerate` を使用してインデックス追跡が必要
  - または `rposition` で最後の `CallScene` インデックスを事前計算

## Architecture Pattern Evaluation

| Option                  | Description                                  | Strengths                                | Risks / Limitations            | Notes               |
| ----------------------- | -------------------------------------------- | ---------------------------------------- | ------------------------------ | ------------------- |
| **A: 既存拡張（推奨）** | `code_generator.rs` 内に末尾判定ロジック追加 | 局所的変更、迅速な開発、既存パターン踏襲 | 将来複雑化した場合に分離が必要 | Option B で確定済み |
| B: 新規コンポーネント   | `TailCallOptimizer` クラスを新規作成         | 関心の分離、単体テスト容易               | 過剰設計、ファイル数増加       | 現時点では不要      |
| C: ハイブリッド         | 初期は A、複雑化後に B へ移行                | 段階的最適化                             | リファクタリングコスト         | 要件確定のため不要  |

## Design Decisions

### Decision: 末尾判定アルゴリズム
- **Context**: `generate_local_scene_items` で末尾 `CallScene` を検出する方法
- **Alternatives Considered**:
  1. `items.last()` との参照比較 — シンプルだがポインタ比較に依存
  2. `enumerate` + 最終インデックス比較 — 明示的で安全
  3. `rposition` で最後の `CallScene` インデックスを事前計算 — 効率的
- **Selected Approach**: `rposition` による事前計算
- **Rationale**: 
  - 末尾以外の `CallScene` を誤って `return` 化しない（最後の `CallScene` のみ対象）
  - ループ外で一度計算するため、ループ内の条件分岐が単純化
  - コードの意図が明確（「最後の呼び出し可能項目を見つける」）
- **Trade-offs**: 
  - 利点: 安全性、可読性、将来の拡張性
  - 妥協: 追加の走査コスト（ただしリストは小規模なため無視可能）
- **Follow-up**: 実装時に `matches!` マクロを使用し、将来の拡張を容易にする

### Decision: generate_call_scene シグネチャ拡張
- **Context**: 末尾呼び出しフラグをどのように伝達するか
- **Alternatives Considered**:
  1. `is_tail_call: bool` パラメータ追加
  2. 構造体にフラグを保持
  3. 新規メソッド `generate_tail_call_scene` を作成
- **Selected Approach**: `is_tail_call: bool` パラメータ追加
- **Rationale**:
  - 既存の呼び出し箇所は `false` を渡せば互換性維持
  - 単純な条件分岐で `return` プレフィックスを制御可能
  - 新規メソッド作成による重複を回避
- **Trade-offs**:
  - 利点: 最小限の変更、高い互換性
  - 妥協: メソッドシグネチャの変更（ただし内部 API のため影響小）
- **Follow-up**: 既存の呼び出し箇所を `false` で更新

### Decision: 将来拡張性の設計方針
- **Context**: 将来 `LocalSceneItem::FnCall` などが追加される場合の対応
- **Alternatives Considered**:
  1. 汎用的なトレイトベース設計
  2. `matches!` マクロによる条件拡張
  3. 現行の `CallScene` 固定
- **Selected Approach**: `matches!` マクロによる条件拡張（Option B）
- **Rationale**:
  - 現時点では `CallScene` のみが対象（要件確定）
  - `matches!` に条件を追加するだけで将来対応可能
  - 過剰設計を回避しつつ、拡張パスを確保
- **Trade-offs**:
  - 利点: 最小実装、低リスク、拡張コメントで意図明示
  - 妥協: 将来パターン追加時に `matches!` 条件変更が必要（軽微）
- **Follow-up**: コメントで拡張方法を明示

## Risks & Mitigations
- **Risk 1**: 末尾以外の `CallScene` に誤って `return` が付与される
  - **Mitigation**: `rposition` で最後の `CallScene` インデックスのみを対象とし、テストで検証
- **Risk 2**: 既存テストのリグレッション
  - **Mitigation**: 末尾以外の `CallScene` は従来通り `return` なしを維持、既存テストで検証
- **Risk 3**: 将来の `LocalSceneItem` 拡張時に見落とし
  - **Mitigation**: `matches!` マクロ内にコメントで拡張方法を明示

## References
- [Lua 5.4 Reference Manual - Proper Tail Calls](https://www.lua.org/manual/5.4/manual.html#3.4.10)
- [Programming in Lua - Proper Tail Calls](https://www.lua.org/pil/6.3.html)
- gap-analysis.md - 実装アプローチ選択（Option B 確定）

````
