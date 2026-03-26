# Research & Design Decisions: ontalk-block-condition

## Summary
- **Feature**: `ontalk-block-condition`
- **Discovery Scope**: Extension（既存モジュールの条件強化）
- **Key Findings**:
  - `has_status()` のプレーンfindは現行SSP Status語彙では安全（衝突パターンなし）
  - `dispatch()` 入口に一括ガードを追加し、`check_hour()`/`check_talk()` の個別チェックを削除する方針が最小変更
  - テストは既存 Lua BDD フレームワーク (`lua_test`) で全9キーワードを網羅可能

## Research Log

### SSP Status ヘッダの仕様確認
- **Context**: ブロック対象キーワードの完全性確認
- **Sources Consulted**: https://ssp.shillest.net/ukadoc/manual/spec_shiori3.html
- **Findings**:
  - Status値: `talking`, `choosing`, `online`, `opening(種類)`, `passive`, `induction`, `timecritical`, `nouserbreak`, `minimizing`, `balloon(ID群)`
  - カンマ区切りで複合値が送信される（例: `"choosing,balloon(0=0)"`）
  - `balloon(...)` はブロック不要（情報のみ、動作制約なし）
- **Implications**: 9キーワードでSSP拡張Status値を網羅。`balloon` は除外が妥当

### has_status() のプレーンfind安全性
- **Context**: `string:find(keyword, 1, true)` による部分一致の衝突リスク
- **Findings**:
  - `has_status("nouserbreak", "user")` → `true`（理論上の衝突）
  - ただしSSP Status語彙に `user` 単体キーワードは存在しない
  - 検査対象9キーワード同士の包含関係: なし（`online` ⊄ `opening` 等）
  - `opening(...)` は `opening` で部分一致検出可能
- **Implications**: 現行語彙では安全。ワード境界マッチ強化は不要（YAGNI）

### 既存テスト構造の確認
- **Context**: テスト追加の統合ポイント特定
- **Findings**:
  - Lua BDD: `crates/pasta_lua/tests/lua_specs/virtual_dispatcher_spec.lua` — talking/choosing のブロックテスト既存
  - Rust統合: `crates/pasta_lua/tests/shiori/virtual_event_dispatch_test.rs` — モジュールロード・基本発火テスト
  - テスト共通ヘルパー: `create_mock_act()` でstatus付きモックact生成可能
- **Implications**: Lua BDDテストへの追加が最も効率的。Rust統合テストは変更不要

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: dispatch入口集約 | dispatch()入口でBLOCKED_STATUSESをループ判定。check_*から個別チェック削除 | 変更量最小、一元管理達成 | check_*直接呼び出し時にガードなし | 推奨。直接呼び出しはドキュメントで対応 |
| B: 共通ガード関数 | is_blocked()関数をdispatch()とcheck_*両方に配置 | 直接呼び出しでも安全 | 通常パスで二重判定（影響は無視可能） | Req 2「個別チェック廃止」と微妙に矛盾 |

## Design Decisions

### Decision: Option A（dispatch入口集約）を採用
- **Context**: Req 1（集約ガード）とReq 2（個別チェック廃止）を同時に満たすアプローチ選択
- **Alternatives Considered**:
  1. Option A — dispatch()入口のみにガード配置
  2. Option B — dispatch() + check_*の両方にガード配置
- **Selected Approach**: Option A
- **Rationale**:
  - Req 2 の意図（重複排除・一元管理）を直接的に達成
  - `check_hour()`/`check_talk()` は `dispatch()` 経由で呼ばれる設計であり、直接呼び出しは想定外
  - 変更量最小（1ファイル + テスト + ドキュメント）
- **Trade-offs**: check_*を直接呼ぶとガードバイパス可能（ドキュメントで「dispatch()経由で使うこと」を明記）
- **Follow-up**: テストで dispatch() 経由のブロック動作を網羅的に検証

### Decision: BLOCKED_STATUSESをモジュールローカルテーブルとする
- **Context**: カスタマイズ可能性の要件が不要と確定（ディスカッションで決定）
- **Selected Approach**: `local BLOCKED_STATUSES = { ... }` としてモジュールローカルに宣言
- **Rationale**: `M.blocked_statuses` として公開する必要がなく、ローカルテーブルの方がカプセル化が良い
- **Trade-offs**: 将来カスタマイズが必要になった場合は `M.` への昇格が必要（低コスト変更）

## Risks & Mitigations
- **check_*直接呼び出し時のガードバイパス** — shiori-handlers.md に「dispatch()経由で使用すること」を明記。テスト内の直接呼び出しは意図的なので問題なし
- **has_status()の部分一致衝突（将来）** — SSP語彙に新キーワードが追加された場合のみリスク。発生時にワード境界マッチへ移行すれば対応可能

## References
- [SSP SHIORI/3.0 Status 仕様](https://ssp.shillest.net/ukadoc/manual/spec_shiori3.html) — Status ヘッダ値の公式リファレンス
- `crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` — 対象モジュール
- `crates/pasta_lua/tests/lua_specs/virtual_dispatcher_spec.lua` — 既存Lua BDDテスト
