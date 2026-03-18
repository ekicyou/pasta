# Research & Design Decisions: act-sakura-script-method

## Summary
- **Feature**: `act-sakura-script-method`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  - `talk()` メソッドが3層すべてで完全なテンプレートとして機能する。新規パターン不要
  - `talk_to_script()` はさくらスクリプトタグをパススルーする設計（トークナイザーが `\` 開始のタグを `TokenKind::SakuraScript` として認識し、ウェイト挿入をスキップ）。`sakura_script` トークンも `talk_to_script()` 経由で安全に処理可能
  - SHIORI継承チェーン（`SHIORI_ACT_IMPL → ACT.IMPL`）により、`ACT_IMPL.sakura_script()` を追加すれば `SHIORI_ACT` にも自動的に継承される

## Research Log

### talk_to_script() のさくらスクリプトタグ処理
- **Context**: `sakura_script` トークン（`\n`, `\w9` 等）を `talk_to_script()` に渡して安全か検証
- **Sources Consulted**: `sakura_script/tokenizer.rs`, `sakura_script/wait_inserter.rs`
- **Findings**:
  - `Tokenizer` は `\\[0-9a-zA-Z_!+*?&-]+(?:\[[^\]]*\])?` パターンでさくらスクリプトタグを最優先でマッチ
  - `WaitValues::get_wait()` は `TokenKind::SakuraScript` に対し `None` を返す（ウェイト挿入なし）
  - `insert_waits()` は `SakuraScript` トークンをそのまま出力に追加（パススルー）
- **Implications**: `sakura_script` トークンの `inner.text`（例: `\n`）を `talk_to_script(actor, inner.text)` に渡すと、タグとして認識されそのまま出力される。`talk` トークンと同じ処理パスで正しく動作する

### SHIORI継承チェーンの影響
- **Context**: `ACT_IMPL` への `sakura_script()` 追加が SHIORI レイヤーに波及するか確認
- **Sources Consulted**: `shiori/act.lua`
- **Findings**:
  - `setmetatable(SHIORI_ACT_IMPL, { __index = ACT.IMPL })`（L22）により `ACT.IMPL` のメソッドは自動継承
  - `SHIORI_ACT_IMPL.__index` は `rawget(SHIORI_ACT_IMPL, key)` → `ACT.IMPL.__index(self, key)` の2段検索
  - `build()` のみオーバーライド。トークン蓄積メソッド（`talk`, `raw_script`, `surface` 等）はすべて継承
- **Implications**: `ACT_IMPL.sakura_script()` を追加すれば SHIORI_ACT でも使用可能。SHIORI レイヤーへの変更は不要

### group_by_actor() の既存パターン
- **Context**: `sakura_script` トークンの統合方法を既存コードから分析
- **Sources Consulted**: `act.lua` L18-60
- **Findings**:
  - `talk` 分岐: アクター変更検出 + `type="actor"` グループへの格納（L35-48）
  - `spot` / `clear_spot` 分岐: 独立出力（L32-33）
  - `else` 分岐: `current_actor_token` が存在すればグループに追加、なければ無視（L50-55）
  - `sakura_script` は `talk` と同じアクター情報を持つため、`talk` 分岐に統合すべき
- **Implications**: `elseif t == "talk" or t == "sakura_script" then` で `talk` 分岐を拡張

## Design Decisions

### Decision: `sakura_script` を `talk` と同パターンで3層に追加
- **Context**: `Action::SakuraScript` のアクター紐付け欠落を修正する設計アプローチ
- **Alternatives Considered**:
  1. Option A: 既存コンポーネント拡張（`talk` パターン踏襲）
  2. Option B: `talk()` に統合（ユーザーにより却下）
  3. Option C: ジェネリックトークンメソッド（過度な抽象化）
- **Selected Approach**: Option A
- **Rationale**: `talk()` の実装パターンが3層で完全なテンプレートとして機能。認知負荷ゼロで拡張可能
- **Trade-offs**: 類似コードが増えるが各3-8行程度で許容範囲
- **Follow-up**: なし

### Decision: `group_by_actor()` でアクター切り替え検出に参加（解釈A）
- **Context**: `sakura_script` トークンが `group_by_actor()` でどう処理されるべきか
- **Alternatives Considered**:
  1. 解釈A: `talk` と同等にアクター切り替え検出に参加
  2. 解釈B: `else` 分岐で既存グループに追加のみ
- **Selected Approach**: 解釈A
- **Rationale**: さくらスクリプトは `\s[ID]` 等のサーフェス切り替えなどアクターに影響するコマンドを発行でき、Pasta DSLのアクション行に記述される要素である
- **Trade-offs**: 解釈Bより分岐ロジックが増えるが、堅牢性が向上
- **Follow-up**: なし

### Decision: `merge_consecutive_talks()` は変更不要
- **Context**: `sakura_script` トークンは連続 `talk` と結合すべきか
- **Selected Approach**: 結合しない（現行動作維持）
- **Rationale**: `sakura_script` は制御文字であり、`surface`/`wait`/`newline` と同じ分離トークンとして機能すべき
- **Follow-up**: なし

## Risks & Mitigations
- **リスク: 既存テスト破壊** → 現行フィクスチャにさくらスクリプト使用例がないため影響なし。新規テストで網羅
- **リスク: 継承チェーン不整合** → SHIORI_ACT_IMPLは__index経由でACT.IMPLを継承。追加メソッドは自動継承される

## References
- `doc/spec/07-sakura-script.md` — さくらスクリプト仕様（§7.1: アクション行内のインライン要素）
- `crates/pasta_lua/src/sakura_script/tokenizer.rs` — タグパターン: `\\[0-9a-zA-Z_!+*?&-]+(?:\[[^\]]*\])?`
- `crates/pasta_lua/src/sakura_script/wait_inserter.rs` — `SakuraScript` トークンのパススルー処理
