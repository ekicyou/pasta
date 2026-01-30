# Research & Design Decisions

## Summary
- **Feature**: `alpha03-shiori-act-sakura`
- **Discovery Scope**: Extension（既存 `pasta.act` モジュールの拡張）
- **Key Findings**:
  - `pasta.act` は `ACT`/`ACT_IMPL` 分離パターンを採用（lua-coding.md 準拠）
  - 現在 `ACT_IMPL` は非公開だが、`ACT.IMPL = ACT_IMPL` で公開することで継承チェーン構築可能
  - さくらスクリプトタグはシンプルな文字列連結で実現可能（外部依存なし）

## Research Log

### pasta.act モジュール構造の調査
- **Context**: 継承元となる `pasta.act` の構造を把握する必要があった
- **Sources Consulted**: `crates/pasta_lua/scripts/pasta/act.lua`
- **Findings**:
  - `ACT` モジュールテーブル + `ACT_IMPL` 実装メタテーブルの分離パターン
  - `ACT_IMPL.__index` はメソッド検索とアクタープロキシ動的生成を担当
  - `ACT.new(ctx)` でインスタンス生成、`setmetatable(obj, ACT_IMPL)` で設定
  - 既存メソッド: `talk`, `sakura_script`, `word`, `yield`, `end_action`, `call`, `set_spot`, `clear_spot`
- **Implications**: `ACT.IMPL` を公開すれば `setmetatable(SHIORI_ACT_IMPL, {__index = ACT.IMPL})` で継承可能

### Lua継承パターンの調査
- **Context**: `ACT_IMPL` が非公開のため、継承方式を検討
- **Sources Consulted**: lua-coding.md、既存モジュール実装
- **Findings**:
  - 方式1: `ACT.new()` の戻り値をベースに構築 → `__index` チェーンが途切れる
  - 方式2: ダミーインスタンス経由でメタテーブル取得 → 複雑
  - 方式3: `ACT.IMPL = ACT_IMPL` で公開 → 最もクリーン
- **Implications**: 方式3を採用。`pasta.act` に1行追加するのみで実現可能

### さくらスクリプト仕様の調査
- **Context**: 生成すべきタグ形式の確認
- **Sources Consulted**: SSP仕様、伺かDeveloper Center
- **Findings**:
  - スコープ切り替え: `\0`（メインキャラ）, `\1`（サブキャラ）, `\p[n]`（任意スコープ）
  - サーフェス: `\s[n]` または `\s[alias]`
  - 待機: `\w[ms]`
  - 改行: `\n`
  - クリア: `\c`
  - 終端: `\e`
  - エスケープ: `\` → `\\`, `%` → `%%`
- **Implications**: すべて文字列連結で実現可能。外部ライブラリ不要

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 継承（メタテーブルチェーン） | `setmetatable(SHIORI_ACT_IMPL, {__index = ACT.IMPL})` | クリーン、親の変更を自動継承 | `ACT.IMPL` 公開が必要 | **採用** |
| 委譲 | 内部に `ACT` インスタンスを保持 | 親を変更不要 | 各メソッドを明示的に委譲、拡張時に手動更新 | 却下 |
| ミックスイン | `ACT_IMPL` のメソッドをコピー | シンプルな `__index` | `ACT_IMPL` 非公開、動的変更に非追従 | 却下 |

## Design Decisions

### Decision: `ACT.IMPL` 公開による継承
- **Context**: `pasta.shiori.act` が `pasta.act` を継承する必要があるが、`ACT_IMPL` は非公開
- **Alternatives Considered**:
  1. 委譲パターン — 各メソッドを明示的にラップ
  2. ミックスイン — メソッドをコピー
  3. `ACT.IMPL` 公開 — 親モジュールに1行追加
- **Selected Approach**: `ACT.IMPL = ACT_IMPL` を `pasta.act` に追加
- **Rationale**: 最小限の変更で継承チェーン構築可能。将来の拡張も自動継承
- **Trade-offs**: 親モジュールの変更が必要だが、1行のみで影響は軽微
- **Follow-up**: `pasta.act` への変更を本仕様のタスクに含める

### Decision: `talk()` オーバーライドによるスコープ自動切り替え
- **Context**: actor 切り替え時にスコープタグと改行を自動挿入したい
- **Alternatives Considered**:
  1. 手動スコープ切り替え（`sakura()`, `kero()` メソッド提供）
  2. `talk()` オーバーライドで自動処理
- **Selected Approach**: `talk(actor, text)` オーバーライド
- **Rationale**: `ActorProxy.spot` 情報を活用し、自然なAPI体験を提供
- **Trade-offs**: `pasta.act` の `talk()` シグネチャと互換だが、内部動作は異なる
- **Follow-up**: テストで actor 切り替え動作を検証

### Decision: 内部バッファ `_buffer` の分離
- **Context**: さくらスクリプト文字列を蓄積するバッファが必要
- **Alternatives Considered**:
  1. `token` バッファを再利用
  2. 新規 `_buffer` フィールドを追加
- **Selected Approach**: `_buffer` を新規追加
- **Rationale**: `token` は `pasta.act` の既存機能用。責務分離を維持
- **Trade-offs**: フィールド増加だが、責務が明確
- **Follow-up**: `reset()` で `_buffer` のみクリア、`token` は維持

## Risks & Mitigations

| リスク | 緩和策 |
|--------|--------|
| `pasta.act` 変更の影響範囲 | 変更は `ACT.IMPL = ACT_IMPL` 1行のみ。既存テストで回帰確認 |
| エスケープ漏れ | テストケースでエッジケース（`\`, `%`, 連続特殊文字）を網羅 |
| `newline(n)` の float 引数 | `math.floor(n)` で整数化、またはバリデーション追加 |

## References

- [pasta.act](crates/pasta_lua/scripts/pasta/act.lua) — 継承元モジュール
- [lua-coding.md](.kiro/steering/lua-coding.md) — Lua コーディング規約
- [SSP仕様](https://ssp.shillest.net/ukadoc/manual/) — さくらスクリプト仕様（外部参照）
