# Research & Design Decisions: surface-dictionary-sync

## Summary
- **Feature**: `surface-dictionary-sync`
- **Discovery Scope**: Extension（既存システムのデータ修正）
- **Key Findings**:
  - `＠元気` 7箇所は文脈分析により3種の既存表情（笑顔×4、照れ×1、キラキラ×2）に分散置換可能
  - `＠考え` 4箇所は文脈分析により3種の既存表情（通常×2、眠い×1、困惑×1）に分散置換可能
  - 結果としてスクリプト未使用だった3表情（泣き、困惑、キラキラ）のうち2つ（困惑、キラキラ）が使用状態に移行し、表情バリエーションが豊かになる

## Research Log

### セリフ文脈分析: `＠元気` の7箇所

- **Context**: `＠元気` は男の子の「快活な表情」として使われているが、セリフごとに感情ニュアンスが異なる
- **Sources Consulted**: `scripts.rs` 内の BOOT_PASTA / TALK_PASTA / CLICK_PASTA 定数
- **Findings**:
  - **素直な明るさ**（4件）: 自己紹介、推薦、時報リアクション、声かけ → `＠笑顔`
  - **照れ隠し**（1件）: 「しょうがないなあ」ツンデレ承諾 → `＠照れ`
  - **得意気なからかい**（2件）: 「照れてるの？」「ぼくのことが気になる？」 → `＠キラキラ`
- **Implications**: 一律置換ではなくセリフ文脈に応じた分散置換が意味的自然さを確保する

### セリフ文脈分析: `＠考え` の4箇所

- **Context**: `＠考え` は「思索的」な表情として両アクターに使われているが、思考の質が異なる
- **Sources Consulted**: `scripts.rs` 内の TALK_PASTA 定数
- **Findings**:
  - **ぼんやりした迷い**（1件）: 「今日は何しようかな...」 → `＠眠い`（半目のぼーっと感）
  - **軽い困惑**（1件）: 「わかんないや」 → `＠困惑`（答えが出ない感）
  - **静かな思索**（2件）: 哲学的内省、同調コメント → `＠通常`（落ち着いた表情）
- **Implications**: こちらも文脈依存の分散置換が適切。結果として「困惑」「眠い」が使用表情に加わり表情の利用幅が広がる

### 表情使用状況の変化

- **Context**: 置換前後で各表情の使用状況がどう変わるか
- **Findings**:

| 表情名 | 置換前使用数 | 置換後使用数 | 変化 |
|--------|-------------|-------------|------|
| ＠笑顔 | 多数 | +4 | 増加 |
| ＠通常 | 多数 | +2 | 増加 |
| ＠照れ | 少数 | +1 | 増加 |
| ＠驚き | 少数 | ±0 | 変化なし |
| ＠泣き | 0 | 0 | **未使用のまま** |
| ＠困惑 | 0 | +1 | **新規使用** |
| ＠キラキラ | 0 | +2 | **新規使用** |
| ＠眠い | 少数 | +1 | 増加 |
| ＠怒り | 少数 | ±0 | 変化なし |

- **Implications**: 9種中8種が使用状態に。`＠泣き` のみスクリプト未使用だが、辞書定義としては正しい（`Expression::Crying` に対応するサーフェスが存在するため）

## Design Decisions

### Decision: Option B（スクリプト修正アプローチ）の採用

- **Context**: `＠元気`/`＠考え` が `image_generator.rs` に対応しない表情名であり、修正が必要
- **Alternatives Considered**:
  1. Option A — 辞書に元気/考えを追加（`Expression` enum 拡張、サーフェス番号再設計）
  2. Option B — スクリプト側を既存辞書に合わせる（テキスト置換）
  3. Option C — 未使用表情を元気/考えに置換（enum rename）
- **Selected Approach**: Option B — `image_generator.rs` を「憲法」として扱い、スクリプト側のみを修正
- **Rationale**: `image_generator.rs` が生成するサーフェスが Source of Truth。辞書は既に正しい。スクリプトが誤った表情名を使用しているのが根本原因
- **Trade-offs**: 「元気」「考え」という表情名は失われるが、文脈ごとの分散置換により意味的自然さは維持される
- **Follow-up**: 各置換箇所のセリフ↔表情の意味的整合性をレビューで確認

### Decision: 文脈依存の分散置換（一律置換ではない）

- **Context**: `＠元気` を全て `＠笑顔` に一律置換する案もあり得る
- **Selected Approach**: セリフの感情文脈に応じて複数の既存表情に分散置換
- **Rationale**: サンプルゴーストは教育目的であり、表情の使い分けバリエーションを示すことに価値がある。一律置換では未使用表情が3種残り続ける
- **Trade-offs**: 置換作業が若干増えるが、結果の品質が大幅に向上する

## Risks & Mitigations
- 置換先の表情とセリフの意味的ミスマッチ — レビューで全箇所を確認
- 既存テスト破壊 — `cargo test --all` で確認（影響は pasta_sample_ghost 内に閉じる）

## References
- `crates/pasta_sample_ghost/src/image_generator.rs` — Expression enum（9 variants）
- `crates/pasta_sample_ghost/src/scripts.rs` — スクリプト定数（Single Source of Truth）
- `.kiro/specs/surface-dictionary-sync/gap-analysis.md` — 3 Options の詳細比較
