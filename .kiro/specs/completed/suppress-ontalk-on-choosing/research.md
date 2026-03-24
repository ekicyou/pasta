# Research & Design Decisions: suppress-ontalk-on-choosing

## Summary
- **Feature**: suppress-ontalk-on-choosing
- **Discovery Scope**: Extension（既存ガード節パターンの拡張）
- **Key Findings**:
  - `act.req.status` は Lua 側で唯一 `virtual_dispatcher.lua` のみが参照。影響範囲は最小
  - SSP の Status ヘッダーは `talking,choosing,balloon(0=2)` 形式のカンマ区切り複合値。`talking` / `choosing` は他トークンの部分文字列にならない
  - サンプルゴースト（hello-pasta）の `pasta_scripts/` はリリース時コピーであり、ソースは `crates/pasta_lua/pasta_scripts/` が Single Source of Truth

## Research Log

### `act.req.status` の参照箇所
- **Context**: choosing ガード追加の影響範囲を確定するため、`req.status` の参照箇所を全量調査
- **Sources Consulted**: `grep_search` による全ファイル検索
- **Findings**:
  - Lua 側: `virtual_dispatcher.lua` の 2 箇所のみ（L98, L129）
  - Rust 側: `lua_request.rs` L113 で `table.set("status", value)` として生文字列格納
  - `req.rs` のテストは `status: None` のアサーションのみ（Status ヘッダーなしリクエスト）
  - サンプルゴーストの `virtual_dispatcher.lua` はソースのコピー（リリース時更新）
- **Implications**: Option A（`string.find`）で Lua 側のみ変更すれば完結。Rust 側は変更不要

### ガード節の挿入位置分析
- **Context**: choosing ガードの挿入位置がタイマー消費に影響するため、既存コードフローを精査
- **Sources Consulted**: `virtual_dispatcher.lua` L84-168
- **Findings**:
  - `check_hour()`: talking ガードは「正時到達チェック後」かつ「next_hour_unix 更新前」に配置（L98）。choosing ガードも同位置に配置すれば、Req 2.2（正時タイムスタンプ非更新）を自然に満たす
  - `check_talk()`: talking ガードは関数冒頭（L129）。タイマー初期化・到達チェックより前なので、choosing ガードも同位置に配置すれば Req 1.2（タイマー非消費）を自然に満たす
- **Implications**: talking ガードと choosing ガードは同一位置に配置。統合ヘルパーで一括判定可能

### `string.find` の安全性検証
- **Context**: 部分一致検索による誤検出リスクの確認
- **Sources Consulted**: SSP SHIORI/3.0 仕様、shiori-sample.log
- **Findings**:
  - SSP Status トークン一覧: `talking`, `choosing`, `balloon(N=M)`, `teachbox`, `inputbox`
  - `talking` は他トークンの部分文字列にならない（`talking` を含む他トークンは存在しない）
  - `choosing` も同様に一意
  - `string.find(status, keyword, 1, true)` の `true` フラグでパターンマッチを無効化し、プレーンテキスト検索として実行
- **Implications**: 誤検出リスクなし。`string.find` による部分一致で安全

## Design Decisions

### Decision: Status 判定方式 — `string.find` 部分一致（Option A）

- **Context**: カンマ区切り Status ヘッダーから `talking` / `choosing` を検出する方式の選定
- **Alternatives Considered**:
  1. Option A — Lua `string.find()` 部分一致（Lua 側のみ修正）
  2. Option B — Rust 側で Status をカンマ分割し配列化（`act.req.status` の型変更）
  3. Option C — Rust 側でブーリアンフラグ化（`act.req.is_talking` 等）
- **Selected Approach**: Option A
- **Rationale**:
  - 最小変更原則: Lua ファイル 1 本の修正で全要件を充足
  - 後方互換: `act.req.status` の型（string）を維持
  - 既存パターン踏襲: talking ガード節と同構造で一貫性を保持
  - 開発者承認済み（ディスカッション #1）: 「含む」パターン統一を明示的に承認
- **Trade-offs**: Lua 側の文字列操作であり、構造化データとしての型安全性は得られない。ただし SSP Status トークンが一意であるため実用上の問題なし
- **Follow-up**: `has_status` 関数のテストケースで境界値（nil, 空文字列, 単独値, 複合値）を網羅

### Decision: `has_status` ヘルパーのスコープ — モジュールローカル

- **Context**: Status 判定ヘルパー関数の公開範囲
- **Alternatives Considered**:
  1. モジュールローカル関数（`local function has_status`）
  2. 共有ユーティリティモジュール（`pasta.util.status` 等）
- **Selected Approach**: モジュールローカル関数
- **Rationale**: 現時点で `req.status` を参照するのは `virtual_dispatcher.lua` のみ。YAGNI 原則に従い、必要になるまで共有化しない
- **Trade-offs**: 将来的に他モジュールでも Status 判定が必要になった場合、関数の重複が発生する可能性あり。その際に共有化を検討

## Risks & Mitigations
- **Risk**: サンプルゴースト（hello-pasta）の `pasta_scripts/` が古いまま残る → リリースワークフロー（release.ps1）で自動コピーされるため、次回リリース時に自動解消
- **Risk**: 将来の SSP バージョンで `choosing` を部分文字列に含む新トークンが追加される → 極めて低確率。発生時は `string.find` をカンマ分割ベースに移行

## References
- SSP SHIORI/3.0 仕様: https://ssp.shillest.net/ukadoc/manual/spec_shiori3.html
- ギャップ分析レポート: `gap-analysis.md`（本仕様ディレクトリ内）
