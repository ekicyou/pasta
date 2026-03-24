# Research & Design Decisions: talk-frequency-persistence

## Summary
- **Feature**: `talk-frequency-persistence`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  - `get_config()` 1関数のみの変更で全要件を充足可能
  - `cached_config` を廃止し毎回 SAVE/toml を読む方式が最適（Option A）
  - SAVE テーブル基盤は完全稼働中、Rust 層の変更不要

## Research Log

### SAVE テーブル永続化基盤
- **Context**: SAVE テーブルへの読み書きパスを確認
- **Sources Consulted**: `pasta/save.lua`, `persistence.rs`, `internal-modules.md`
- **Findings**:
  - `require("pasta.save")` は `@pasta_persistence.load()` 結果のテーブル参照を返す
  - Lua の `require` キャッシュにより、どこから呼んでも同一テーブル参照
  - Drop 時に自動保存（JSON/gzip）
  - テーブルへの直接代入（`save.key = value`）で書き込み完了
- **Implications**: 読み書きのためにRust側の新規API不要

### get_config() キャッシュ機構
- **Context**: 実行時変更反映（Req 2）のためキャッシュ戦略を検討
- **Sources Consulted**: `virtual_dispatcher.lua` 行 18-32
- **Findings**:
  - `cached_config` はモジュールローカル変数、セッション中不変
  - `_reset()` でのみクリア（テスト用）
  - `check_talk()` は `OnSecondChange` 毎秒呼ばれるが、`require` はキャッシュ返却 + テーブルキー参照2回のみのため負荷は微小
- **Implications**: キャッシュ廃止で Req 2 を暗黙的に満たせる

### テスト基盤
- **Context**: 既存テストパターンを確認し、拡張方針を決定
- **Sources Consulted**: `virtual_event_config_test.rs`, `virtual_event_dispatch_test.rs`
- **Findings**:
  - `create_runtime_with_pasta_path()` でランタイム初期化
  - `dispatcher._reset()` で状態分離
  - `dispatcher._get_internal_state().cached_config` で設定値を検査
  - `dispatcher._set_scene_executor()` でモック
  - テスト内で Lua コードを直接実行するパターン（`runtime.exec(r#"..."#)`）
- **Implications**: SAVE テーブルへの事前書き込み + `dispatch()` + 内部状態検証のパターンで拡張可能

### SAVE キー命名規約
- **Context**: 開発者とのディスカッションで決定
- **Sources Consulted**: 要件レビューディスカッション
- **Findings**:
  - エンジン予約キーには `pasta_` プレフィックスを付与
  - ゴースト固有キーは任意命名（プレフィックスなし）
  - 決定キー名: `pasta_talk_interval_min`, `pasta_talk_interval_max`
- **Implications**: `pasta-lua-coding` スキルへの規約追記が必要（Req 4）

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| **A: get_config() 拡張（キャッシュ廃止）** | 毎回 SAVE → toml → default のフォールバックを実行 | 変更1関数、Req 2 自然充足、追加API不要 | 毎秒テーブルルックアップ（微小） | **推奨** |
| B: キャッシュ維持 + invalidate_config() | cached_config 維持、公開関数で無効化 | パフォーマンス最適 | 作者が呼び忘れるリスク、API面積増加 | 不採用 |
| C: メタテーブル監視 | `__newindex` で SAVE 変更を検知 | 完全自動 | 影響範囲大、過度の複雑化 | 不採用 |

## Design Decisions

### Decision: Option A（キャッシュ廃止 + 毎回読み直し）
- **Context**: Req 1（SAVE 読み出し）と Req 2（実行時変更反映）を同時に満たす方式の選定
- **Alternatives Considered**:
  1. Option A — キャッシュ廃止、毎 `check_talk()` 呼び出しで SAVE/toml を読む
  2. Option B — キャッシュ維持 + `invalidate_config()` 公開関数
  3. Option C — メタテーブル `__newindex` 監視
- **Selected Approach**: Option A
- **Rationale**:
  - `require("pasta.save")` は Lua キャッシュ返却で O(1)
  - テーブルキー参照は 2 回（`pasta_talk_interval_min`, `pasta_talk_interval_max`）のみ
  - `@pasta_config` も `require` キャッシュ返却
  - 追加 API 不要でゴースト作者の学習コストゼロ
- **Trade-offs**: 毎秒のルックアップコスト vs API 簡潔性 → ルックアップコスト無視可能
- **Follow-up**: パフォーマンス計測は不要（`require` キャッシュ + テーブルキー O(1) のため）

### Decision: hour_margin は永続化対象外
- **Context**: `hour_margin` も同パターンで永続化可能だが、現要件では対象外
- **Selected Approach**: 対象外（toml のみ）
- **Rationale**: ユーザー操作で変更するニーズがない。同パターンで将来追加可能。

### Decision: _get_internal_state() の返却値変更
- **Context**: キャッシュ廃止により `cached_config` フィールドは不要になる
- **Selected Approach**: `cached_config` フィールドを除去。テストで設定値を検証する場合は `get_config()` を直接呼ぶテスト用関数を追加
- **Rationale**: 内部状態の公開を最小限に保つ

## Risks & Mitigations
- **低リスク**: 毎秒の SAVE テーブル参照パフォーマンス → `require` キャッシュにより実質 O(1)、計測不要
- **低リスク**: 既存テストの破壊 → `cached_config` フィールドは `_get_internal_state()` から除去するが、テスト側も同時更新

## References
- [gap-analysis.md](gap-analysis.md) — 完全なギャップ分析レポート
- [virtual_dispatcher.lua](../../../crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua) — 変更対象ファイル
- [internal-modules.md](../../../.agents/skills/pasta-lua-coding/references/internal-modules.md) — SAVE/ACT API リファレンス
