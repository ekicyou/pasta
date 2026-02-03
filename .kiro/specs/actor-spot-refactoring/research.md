# Research & Design Decisions: actor-spot-refactoring

---
**Purpose**: 責務分離設計の調査結果と設計判断を記録

---

## Summary
- **Feature**: `actor-spot-refactoring`
- **Discovery Scope**: Extension（既存システムへの責務分離リファクタリング）
- **Key Findings**:
  1. 現行act.luaはトークン生成と状態管理の責務が混在（talk()内で`now_actor`/`_current_spot`を管理）
  2. set_spot()およびclear_spot()がトークンを生成せず、直接状態変更している点がtoken-bufferパターンに反する
  3. sakura_builderは現状純粋関数だが、状態管理を受け入れる拡張が必要

## Research Log

### トークン生成層の責務分析
- **Context**: act:talk()が状態管理と切り替え検出を行っている現行設計の評価
- **Sources Consulted**: 
  - [act.lua#L76-83](crates/pasta_lua/scripts/pasta/act.lua#L76-L83)
  - [sakura_builder.lua](crates/pasta_lua/scripts/pasta/shiori/sakura_builder.lua)
- **Findings**:
  - `now_actor`と`_current_spot`はbuild()時にリセット
  - actorトークンはactorオブジェクト参照を持つ（ActorProxy非参照）
  - spot_switchトークンはspot情報を持たず、改行量のみを伝達
- **Implications**: 
  - トークン生成は状態レス化可能
  - ビルダー側で状態追跡を行う設計に移行可能

### set_spot()およびclear_spot()のトークン化検討
- **Context**: 現行set_spot()とclear_spot()はactor.spotを直接変更するのみ
- **Sources Consulted**:
  - [act.lua#L257-268](crates/pasta_lua/scripts/pasta/act.lua#L257-L268)
  - [sample.generated.lua#L28-30](crates/pasta_lua/tests/fixtures/sample.generated.lua#L28-L30)
- **Findings**:
  - `set_spot(name, number)`はactor.spotを直接変更
  - `clear_spot()`は全actorのspotをnilにリセット
  - いずれもトークン生成なし → token-bufferパターンに反する
  - ビルダーはset_spot()/clear_spot()の効果を認識できない
- **Implications**: 
  - `{type="spot", actor=actor, spot=spot}`トークンを生成すべき
  - `{type="clear_spot"}`トークンを生成すべき
  - ビルダーがこれらのトークンを受け取り、内部状態に反映

### ビルダー状態管理設計
- **Context**: sakura_builderに状態管理を導入する設計検討
- **Sources Consulted**:
  - [sakura_builder.lua](crates/pasta_lua/scripts/pasta/shiori/sakura_builder.lua)
- **Findings**:
  - 現行は純粋関数（状態なし）
  - actorトークン処理時にspot_idを抽出し\p[N]を出力
  - spot_switchトークンで\n[N]を出力
- **Implications**: 
  - ビルダーに`actor_spots: {[actor_name]: spot_id}`と`last_actor`状態を追加
  - talkトークン処理時にlast_actorと比較しスポットタグ出力を判断
  - spotトークン処理時にactor_spotsを更新

## Architecture Pattern Evaluation

| Option                  | Description                   | Strengths                    | Risks / Limitations      | Notes    |
| ----------------------- | ----------------------------- | ---------------------------- | ------------------------ | -------- |
| A: 状態レストークン生成 | act層は純粋にトークン生成のみ | テスタビリティ向上、責務明確 | ビルダー実装複雑化       | **採用** |
| B: 現行パターン維持     | act:talk()内で状態管理継続    | 変更量最小                   | 責務混在、拡張困難       | 却下     |
| C: Middleware層追加     | act→middleware→builderの3層化 | 関心分離最大化               | オーバーエンジニアリング | 将来検討 |

## Design Decisions

### Decision: トークン生成層の状態レス化
- **Context**: act:talk()が`now_actor`/`_current_spot`を管理している問題
- **Alternatives Considered**:
  1. 現行維持 — 変更なし
  2. 状態をビルダーに移動 — act層状態レス化
  3. 別ストアに状態分離 — 複雑化
- **Selected Approach**: Option 2 - ビルダーへの状態移動
- **Rationale**: token-bufferパターンの原則に従い、トークン生成と状態管理を明確に分離
- **Trade-offs**: 
  - ✅ テスタビリティ向上（act層が純粋関数に近づく）
  - ✅ 責務の明確化
  - ⚠️ ビルダーの実装が若干複雑化
- **Follow-up**: ビルダーの状態リセットタイミングを明確化

### Decision: 新トークン構造の設計
- **Context**: どのようなトークン構造が最適か
- **Alternatives Considered**:
  1. `{type="spot", actor, spot}` + `{type="talk", actor, text}`
  2. `{type="actor", actor, spot}` + `{type="text", text}`
  3. `{type="utterance", actor, spot, text}` — 複合トークン
- **Selected Approach**: Option 1 - 分離トークン
- **Rationale**: 
  - set_spot()はtalk()とは独立した操作
  - 単一責任の原則に従う
  - talkトークンにactorを含めることでビルダーが切り替え検出可能
- **Trade-offs**: 
  - ✅ 各トークンが単一目的
  - ✅ 既存コードとの互換性（talkトークンのtext属性維持）
  - ⚠️ トークン数が若干増加

### Decision: 設定プロパティ名変更
- **Context**: `spot_switch_newlines` → `spot_newlines`へのリネーム
- **Alternatives Considered**:
  1. 破壊的変更として即時変更
  2. 旧名をフォールバックとしてサポート
  3. 段階的移行（deprecation warning付き）
- **Selected Approach**: Option 1 - 破壊的変更
- **Rationale**: 
  - プロジェクトは開発初期（Phase 2）
  - 影響範囲は`pasta_sample_ghost`のみ
  - 後方互換性維持のコストが利益を上回る
- **Trade-offs**: 
  - ✅ クリーンなAPI
  - ⚠️ 既存設定ファイルの更新必要

## Risks & Mitigations
- **ビルダー状態管理の複雑化** — 明確なインターフェース定義とテストで対応
- **既存テストの大幅修正** — テスト構造は維持し、トークン構造の期待値のみ更新
- **パフォーマンス影響** — トークン数増加は微小、測定可能な影響なし

## References
- [act-token-buffer-refactor spec](../.kiro/specs/completed/act-token-buffer-refactor/) — 元となるtoken-buffer設計
- [gap-analysis.md](./gap-analysis.md) — 影響範囲分析
- [lua-coding.md](../../steering/lua-coding.md) — Luaコーディング規約
