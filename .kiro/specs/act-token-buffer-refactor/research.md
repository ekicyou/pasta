# Research & Design Decisions

## Summary
- **Feature**: `act-token-buffer-refactor`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  1. 既存の継承チェーン（SHIORI_ACT_IMPL → ACT.IMPL）は維持可能
  2. Luaの多態性（self:build()呼び出し）により、yield()の親クラス統一が可能
  3. sakura_builderモジュールは既存パターン（MODULE/MODULE_IMPL分離）に従う

## Research Log

### 親クラスメソッド呼び出しパターン
- **Context**: 子クラスから親クラスのメソッドを呼び出す方法の確認
- **Sources Consulted**: pasta/shiori/act.lua 既存実装
- **Findings**:
  - `ACT.IMPL.method(self, ...)` パターンで親メソッドを呼び出し可能
  - 既存コード例: `ACT.IMPL.talk(self, actor, text)` （shiori/act.lua:135行）
- **Implications**: build()の親子分離設計が実現可能

### 多態性とyield()統一
- **Context**: yield()を親クラスに統一し、子クラスはbuild()のみオーバーライドする設計の実現可能性
- **Sources Consulted**: Lua言語仕様、pasta/act.lua
- **Findings**:
  - `self:build()` 呼び出しは多態性が働く（子クラスのbuild()が呼ばれる）
  - コロン構文はメタテーブルの`__index`チェーンを辿る
- **Implications**: yield()は親クラスに完全移管可能

### トークン形式の設計
- **Context**: 各トークンタイプの構造決定
- **Sources Consulted**: 要件定義、既存pasta/act.lua
- **Findings**:
  - 既存トークン: `{ type = "actor", actor = actor }`, `{ type = "talk", text = text }`
  - 新規トークン: surface, wait, newline, clear, spot_switch
- **Implications**: 統一されたトークン形式でsakura_builderの実装が容易

### スポット切り替え検出の実装位置
- **Context**: スポット切り替え検出を親クラスで行うか、sakura_builderで行うか
- **Sources Consulted**: 要件定義、設計判断事項
- **Findings**:
  - 要件2: 親クラスでspot_switchトークンを挿入
  - 設計判断: スポットID計算はbuild時（sakura_builder内）
- **Implications**: 親クラスはspot_switch検出とトークン挿入、sakura_builderはスポットタグ生成

## Architecture Pattern Evaluation

| Option                       | Description                            | Strengths            | Risks / Limitations | Notes    |
| ---------------------------- | -------------------------------------- | -------------------- | ------------------- | -------- |
| Option A: _buffer維持        | 既存構造維持、部分修正                 | 変更量少             | 責務混在が残る      | 却下     |
| Option B: sakura_builder分離 | 新モジュールに変換ロジック移動         | 責務明確、テスト容易 | 新規コード量        | **採用** |
| Option C: 親クラス拡張のみ   | sakura_builderなし、子クラスで直接変換 | シンプル             | 再利用性低          | 却下     |

## Design Decisions

### Decision: トークン形式の統一
- **Context**: 全メソッドで一貫したトークン形式が必要
- **Alternatives Considered**:
  1. メソッドごとに異なる形式
  2. 統一されたtype/payloadパターン
- **Selected Approach**: 統一パターン `{ type = "xxx", ... }`
- **Rationale**: sakura_builderでのswitch処理が容易
- **Trade-offs**: やや冗長だが明確性を優先
- **Follow-up**: トークンタイプのドキュメント化

### Decision: build()の責務分離（親子間）
- **Context**: build()でトークン取得とフォーマット変換を分離
- **Alternatives Considered**:
  1. 子クラスでトークン取得とリセットも実装
  2. 親がトークン取得＋リセット、子がフォーマット変換
- **Selected Approach**: Option 2
- **Rationale**: DRY原則、多態性の活用
- **Trade-offs**: 親メソッド呼び出しが必要（`ACT.IMPL.build(self)`）
- **Follow-up**: yield()統一と組み合わせて検証

### Decision: yield()の親クラス統一
- **Context**: yield()の重複実装を排除
- **Alternatives Considered**:
  1. 親子両方でyield()を実装
  2. 親のみでyield()を実装、build()の多態性を活用
- **Selected Approach**: Option 2
- **Rationale**: 単一責務原則、コード重複排除
- **Trade-offs**: 子クラスでyield()の挙動変更が困難になる（ただし不要）
- **Follow-up**: 既存SHIORI_ACT_IMPL.yield()の削除

### Decision: end_action()の削除
- **Context**: end_action()はbuild()と意味が重複し、トランスパイラーからの使用箇所も確認できない
- **Alternatives Considered**:
  1. end_action()を残し、build()を内部呼び出しに変更
  2. end_action()を公開APIから削除
- **Selected Approach**: Option 2
- **Rationale**: 責務重複の排除、APIの簡素化
- **Trade-offs**: 互換性の一部低下（end_action利用者がいる場合に影響）
- **Follow-up**: 互換性注意点を要件に明記

### Decision: \eの付与タイミング
- **Context**: さくらスクリプト終端タグの付与場所
- **Alternatives Considered**:
  1. sakura_builder.build()内で付与
  2. SHIORI_ACT_IMPL.build()で付与
- **Selected Approach**: Option 1（sakura_builder内）
- **Rationale**: 変換ロジックの完全カプセル化
- **Trade-offs**: sakura_builderがさくらスクリプト仕様を知る必要あり
- **Follow-up**: 要件6-4と整合性確認

## Risks & Mitigations
- **Risk 1**: 既存テスト（717行）の大規模修正が必要
  - **Mitigation**: 主要公開APIは維持しつつ、`end_action()`削除の影響範囲を確認
- **Risk 2**: talk()後の固定改行除去による互換性問題
  - **Mitigation**: テストケースで現行動作と新動作を比較検証
- **Risk 3**: pasta_sample_ghostの同期漏れ
  - **Mitigation**: 実装タスクに同期作業を含める

## References
- [pasta/act.lua](../../crates/pasta_lua/scripts/pasta/act.lua) — 親クラス実装
- [pasta/shiori/act.lua](../../crates/pasta_lua/scripts/pasta/shiori/act.lua) — 子クラス実装
- [lua-coding.md](../../.kiro/steering/lua-coding.md) — Luaコーディング規約
