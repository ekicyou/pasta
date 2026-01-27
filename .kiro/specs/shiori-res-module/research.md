# Research & Design Decisions: shiori-res-module

## Summary
- **Feature**: `shiori-res-module`
- **Discovery Scope**: Simple Addition（単純な新規モジュール追加）
- **Key Findings**:
  - 既存の `pasta.shiori` は空実装であり、純粋な新規追加として実装可能
  - 他のモジュールへの依存なし（ステートレスユーティリティ）
  - 参考実装 `response.lua` のtypo（`Resion` → `Reason`）を修正

## Research Log

### Luaモジュール構造パターン
- **Context**: 既存コードベースのモジュール構造パターンを調査
- **Sources Consulted**: `.kiro/steering/lua-coding.md`, `pasta/word.lua`, `pasta/actor.lua`
- **Findings**:
  - モジュールテーブルは `UPPER_CASE`（例: `RES`）
  - LuaDoc アノテーション必須（`--- @module`）
  - 標準構造: require → モジュールテーブル → ローカル関数 → 公開API → return
  - 外部依存がないモジュールは循環参照の心配なし
- **Implications**: `RES` モジュールは既存パターンに完全準拠可能

### SHIORI/3.0レスポンス形式
- **Context**: プロトコル仕様の確認
- **Sources Consulted**: 参考実装 `response.lua`, 既存 `main.lua`
- **Findings**:
  - ステータスライン: `SHIORI/3.0 {code}\r\n`
  - 標準ヘッダー: Charset, Sender, SecurityLevel（議題1で確定）
  - 追加ヘッダー: key-value形式で追加
  - 終端: 空行 `\r\n`
- **Implications**: 文字列連結による単純な実装で十分

### エラーハンドリングパターン
- **Context**: Defensiveエラーハンドリング方針の実装パターン
- **Sources Consulted**: 議題3決定事項, 既存コードベース
- **Findings**:
  - `dic = dic or {}` パターンで nil を空テーブルに変換
  - 型検証は行わず、Lua標準の振る舞いに任せる
  - オプショナル引数の防御的処理のみ
- **Implications**: シンプルな実装で十分な堅牢性を確保

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: `init.lua` 拡張 | 既存の `pasta.shiori` にレスポンス機能を追加 | ファイル数削減、名前空間統一 | 単一ファイル肥大化、責務混在 | 将来の拡張性が低い |
| **B: 新規 `res.lua`** | `pasta.shiori.res` として独立モジュール作成 | **責務分離、テスト独立性、拡張容易** | ファイル数微増 | **採用** |
| C: ハイブリッド | 新規作成 + 再エクスポート | 柔軟なアクセスパス | 複雑度増加、過剰設計 | 現時点では不要 |

## Design Decisions

### Decision: 新規モジュール `pasta.shiori.res` として作成
- **Context**: レスポンス構築機能の配置場所を決定
- **Alternatives Considered**:
  1. Option A — 既存 `pasta.shiori.init.lua` を拡張
  2. Option B — 新規 `pasta.shiori.res.lua` を作成
  3. Option C — ハイブリッド（新規作成 + 再エクスポート）
- **Selected Approach**: Option B — 新規モジュール作成
- **Rationale**: 
  - 明確な責務分離（Single Responsibility）
  - 将来の `pasta.shiori.req`（リクエスト解析）との並列配置
  - テストの独立性確保
- **Trade-offs**: ファイル数が1つ増加するが、保守性・拡張性の向上が大きい
- **Follow-up**: 実装後に `main.lua` での統合テストを実施

### Decision: 標準ヘッダー3種を常に出力
- **Context**: Charset, Sender, SecurityLevel の出力方針
- **Selected Approach**: 3種すべてを常に出力、`RES.env` で設定可能
- **Rationale**: SHIORI/3.0プロトコルの完全性、既存 `main.lua` との一貫性
- **Trade-offs**: レスポンスサイズ微増だが、プロトコル準拠を優先

### Decision: `RES.env` 直接アクセスパターン
- **Context**: 環境設定の変更API
- **Selected Approach**: setter関数なし、`RES.env.charset = "..."` で直接アクセス
- **Rationale**: Luaらしいシンプルな設計、既存モジュールとの一貫性
- **Trade-offs**: カプセル化は弱いが、ユーティリティとしては十分

### Decision: Defensiveエラーハンドリング
- **Context**: オプショナル引数の処理
- **Selected Approach**: `dic = dic or {}` による防御的処理のみ
- **Rationale**: 実用的なバランス、既存コードベースのパターンと一致
- **Trade-offs**: 型エラーは Lua 標準に任せる

## Risks & Mitigations
- **Risk 1**: 文字列連結のパフォーマンス → **Mitigation**: レスポンスは小規模、最適化不要
- **Risk 2**: typo の残存 → **Mitigation**: `X-Error-Reason`, `X-Warn-Reason` に統一
- **Risk 3**: 統合時の互換性 → **Mitigation**: 既存 `main.lua` との統合テスト実施

## References
- [SHIORI/3.0 Protocol Specification](http://usada.sakura.vg/contents/shiori.html) — SHIORI プロトコル仕様
- `.kiro/steering/lua-coding.md` — Luaコーディング規約
- `.kiro/specs/shiori-res-module/response.lua` — 参考実装
- `.kiro/specs/shiori-res-module/DECISIONS.md` — 要件フェーズ決定事項
