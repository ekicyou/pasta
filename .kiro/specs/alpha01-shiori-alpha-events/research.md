# Research & Design Decisions

## Summary
- **Feature**: alpha01-shiori-alpha-events
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  - 既存 EVENT/REG/RES モジュールが完成度高く、核心部分は実装済み
  - `EVENT.no_entry` をシーン関数フォールバック対応に拡張が主要実装
  - `SCENE.search()` は `@pasta_search` を内部使用し、遅延ロードで安全に統合可能

## Research Log

### 既存イベントディスパッチ機構の調査

- **Context**: Requirement 1, 2 を満たす既存実装の確認
- **Sources Consulted**: 
  - `crates/pasta_lua/scripts/pasta/shiori/event/init.lua`
  - `crates/pasta_lua/scripts/pasta/shiori/event/register.lua`
  - `crates/pasta_lua/tests/shiori_event_test.rs`
- **Findings**:
  - `EVENT.fire(req)`: REG テーブルからハンドラ取得、`xpcall` でエラーハンドリング
  - `EVENT.no_entry(req)`: 未登録イベント用、現在は `RES.no_content()` のみ
  - `REG`: 空テーブル、ハンドラを直接代入（`REG.OnBoot = function(req) ... end`）
  - 470行のテストスイートで基本動作は十分検証済み
- **Implications**: 
  - Requirement 1, 2, 3, 4 の核心は既存実装で充足
  - Requirement 7（シーン関数フォールバック）が唯一の実装追加

### SCENE.search API の調査

- **Context**: Requirement 7 のシーン関数フォールバック統合
- **Sources Consulted**: 
  - `crates/pasta_lua/scripts/pasta/scene.lua`
  - `@pasta_search` Rust モジュール
- **Findings**:
  - `SCENE.search(name, global_scene_name, attrs)`: プレフィックス検索、呼び出し可能なオブジェクト返却
  - 内部で `@pasta_search` を遅延ロード（初期化タイミング問題回避）
  - 検索結果は `setmetatable` で `__call` 付与、直接呼び出し可能
  - 返却: `{ global_name, local_name, func }` または `nil`
- **Implications**: 
  - `SCENE.search(req.id, nil, nil)` でグローバルシーン検索可能
  - 見つかった場合は `result(act, ...)` で呼び出し可能
  - act オブジェクト生成は alpha03 統合で必要

### act オブジェクトとシーン関数の統合

- **Context**: シーン関数呼び出し時のコンテキスト提供
- **Sources Consulted**: 
  - `crates/pasta_lua/scripts/pasta/shiori/act.lua`
- **Findings**:
  - `ACT.new(req)`: 新規 act オブジェクト作成（req から直接初期化）
  - シーン関数は `function SCENE.__name__(act, ...)` シグネチャ
  - act はトークン蓄積、アクタープロキシ、変数管理を担当
  - シーン関数実行後、`act.token` からさくらスクリプト生成（alpha03 で実装）
- **Implications**: 
  - シーン関数フォールバックでは `pasta.shiori.act` を使用
  - alpha03 統合前は `RES.no_content()` フォールバックで十分

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: no_entry 拡張 | EVENT.no_entry にシーン検索追加 | 既存フローに自然統合、最小変更 | no_entry の責務拡大 | **採用**: 単一責任維持可能 |
| B: fire 内分岐 | EVENT.fire 内でシーン検索 | ディスパッチロジック集約 | fire の複雑化 | 不採用: 単一関数肥大化 |
| C: 専用 handler | シーン検索用ハンドラ別途 | 責務分離明確 | 呼び出しフロー複雑化 | 過剰設計 |

## Design Decisions

### Decision: シーン関数フォールバック実装箇所

- **Context**: Requirement 7 を満たすため、REG 未登録時のフォールバック先を決定
- **Alternatives Considered**:
  1. `EVENT.no_entry` 拡張 — シーン検索追加
  2. `EVENT.fire` 内分岐 — ハンドラ取得前にシーン検索
  3. 専用モジュール — `EVENT.scene_fallback` 新設
- **Selected Approach**: Option A（`EVENT.no_entry` 拡張）
- **Rationale**: 
  - 既存の「未登録イベント処理」責務に自然に追加
  - fire のロジックは REG テーブル参照のみに保持
  - テスト追加も no_entry 単位で可能
- **Trade-offs**: 
  - no_entry の責務が「204返却」から「シーン検索 → 204」に拡大
  - ただし、フォールバック戦略として一貫性あり
- **Follow-up**: alpha03 統合時に act オブジェクト生成を追加

### Decision: シーン関数の戻り値処理

- **Context**: シーン関数が返す値の型と処理方法
- **Alternatives Considered**:
  1. 文字列直接返却 — シーン関数が SHIORI レスポンス文字列を返す
  2. act トークン変換 — act.token からさくらスクリプト生成、RES.ok で包む
  3. 戻り値無視 — 204 No Content 固定
- **Selected Approach**: Option 3（alpha01 では戻り値無視、204 No Content）
- **Rationale**: 
  - alpha03（さくらスクリプト組み立て）が未実装
  - act オブジェクト生成・変換ロジックは alpha03 で実装
  - alpha01 はシーン関数「呼び出し」のみに責務限定
- **Trade-offs**: 
  - alpha01 単体ではシーン関数の出力が無視される
  - alpha03 統合後に完全動作
- **Follow-up**: alpha03 完了後に no_entry を再拡張

### Decision: ドキュメント配置

- **Context**: Requirement 8 のドキュメント配置先
- **Selected Approach**: `LUA_API.md` 前方セクション（セクション2）
- **Rationale**: 
  - ゴースト開発の最重要機能として早い位置に配置
  - 既存セクション構成を尊重しつつ、参照頻度が高い位置

## Risks & Mitigations

- **Risk 1**: alpha03 未完了時のシーン関数動作不完全
  - **Mitigation**: alpha01 では「呼び出しのみ」で 204 返却、alpha03 で完全統合
- **Risk 2**: SCENE.search の初期化タイミング問題
  - **Mitigation**: 既存実装で遅延ロード対応済み（`require("@pasta_search")`）
- **Risk 3**: テストカバレッジ不足
  - **Mitigation**: 既存 shiori_event_test.rs 拡張、7種イベント明示テスト追加

## References

- [SHIORI/3.0 プロトコル仕様](http://usada.sakura.vg/contents/shiori.html) — イベント定義、Reference 構造
- [SSP SHIORI Event Reference](http://ssp.shillest.net/ukadoc/manual/list_shiori_event.html) — 各イベントの発火条件、Reference 意味
- `crates/pasta_lua/LUA_API.md` — 既存 Lua API ドキュメント構成
