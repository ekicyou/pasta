# Research & Design Decisions: onhour-date-var-transfer

---
**Purpose**: OnHour仮想イベント発火時の日時変数転記機能に関する調査結果と設計判断を記録する。
---

## Summary
- **Feature**: `onhour-date-var-transfer`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  1. `execute_scene()` が `act` をシーン関数に渡していない実装ミスを発見・確認
  2. 日本語変数マッピングは `SHIORI_ACT:transfer_date_to_var()` メソッド内で実装
  3. 曜日変換は `wday` 数値から日本語/英語文言へのテーブル変換で実現

## Research Log

### execute_scene への act 引き渡し問題

- **Context**: gap-analysis で `execute_scene()` が act を scene_fn に渡していないことを発見
- **Sources Consulted**: 
  - [virtual_dispatcher.lua](../../../crates/pasta_lua/scripts/pasta/shiori/event/virtual_dispatcher.lua) L68-84
  - [act-req-parameter spec](../completed/act-req-parameter/)
- **Findings**: 
  - 現在の実装: `pcall(scene_fn)` - act なし
  - act-req-parameter 仕様では scene_fn(act) を想定していたが、実装が漏れていた
  - 開発者確認済み: これは実装ミスであり、本仕様で修正する
- **Implications**: 
  - `execute_scene(event_name)` → `execute_scene(event_name, act)` に変更
  - `pcall(scene_fn)` → `pcall(scene_fn, act)` に変更
  - テスト用 `scene_executor(event_name)` → `scene_executor(event_name, act)` に変更

### req.date フィールド構造

- **Context**: 転記対象フィールドの正確な仕様を調査
- **Sources Consulted**: 
  - [lua_request.rs](../../../crates/pasta_shiori/src/lua_request.rs) L12-29 `lua_date_from()` 関数
  - [lua_request_test.rs](../../../crates/pasta_shiori/tests/lua_request_test.rs)
- **Findings**:
  - `unix`: i64 - Unix timestamp
  - `year`: i32 - 年
  - `month`: u8 - 月（1-12）
  - `day`: u8 - 日（1-31）
  - `hour`: u8 - 時（0-23）
  - `min`: u8 - 分（0-59）
  - `sec`: u8 - 秒（0-59）
  - `ns`: u32 - ナノ秒
  - `yday`: u16 - 年内通算日数
  - `ordinal`: u16 - yday のエイリアス
  - `wday`: u8 - 曜日（0=日曜、6=土曜）
  - `num_days_from_sunday`: u8 - wday のエイリアス
- **Implications**: 
  - 転記対象: `year`, `month`, `day`, `hour`, `min`, `sec`, `wday`
  - 除外: `unix`, `ns`, `yday`, `ordinal`, `num_days_from_sunday`

### 日本語変数マッピング設計

- **Context**: 開発者要望により日本語変数名での直接アクセスを実装
- **Sources Consulted**: 開発者との議論
- **Findings**:
  - 日本語変数は**単位付き文字列**として提供（例: "2026年", "9時"）
  - 英語フィールドは**数値型**のまま転記
  - 12時間制表示用に `時１２`（全角数字）変数を追加
  - 曜日は `wday` 数値から日本語/英語文言に変換
- **Implications**: 
  - SHIORI_ACT にメソッド `transfer_date_to_var()` を追加
  - 変換ロジックはメソッド内で完結（将来的な再利用を考慮）

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: execute_scene 拡張 | execute_scene に act を渡す + 転記処理 | 最小限の変更、既存パターン準拠 | なし | **採用** |
| B: 転記専用ヘルパー関数 | 独立したヘルパー関数として実装 | 再利用性高い | ファイル追加の可能性 | SHIORI_ACT メソッドとして吸収 |
| C: Hybrid | A + 日本語変数名マッピング | 完全な日本語対応 | スコープ拡大リスク | **採用**（A の拡張として） |

## Design Decisions

### Decision: 転記関数を SHIORI_ACT メソッドとして実装

- **Context**: 転記ロジックの配置場所の決定
- **Alternatives Considered**:
  1. virtual_dispatcher.lua 内のローカル関数
  2. 独立したモジュール（例: date_transfer.lua）
  3. SHIORI_ACT のメソッド
- **Selected Approach**: SHIORI_ACT のメソッド `transfer_date_to_var()`
- **Rationale**: 
  - act オブジェクトに対する操作なので、メソッドとして自然
  - 将来的に OnTalk 等でも再利用可能
  - steering の MODULE/MODULE_IMPL 分離パターンに準拠
- **Trade-offs**: 
  - ✅ 再利用性が高い
  - ✅ テストが容易（act 単体でテスト可能）
  - ❌ shiori/act.lua の変更が必要

### Decision: 日本語変数は単位付き文字列

- **Context**: 日本語変数の表現形式の決定
- **Alternatives Considered**:
  1. 数値型（英語と同じ）
  2. 単位付き文字列（"2026年", "9時"）
- **Selected Approach**: 単位付き文字列
- **Rationale**: 
  - ゴースト開発者が直接文字列結合でメッセージを構築できる
  - `var.年 .. var.月 .. var.日 .. var.時 .. var.分` → "2026年2月1日9時37分"
- **Trade-offs**: 
  - ✅ 使いやすい（文字列結合で完成）
  - ❌ 数値計算には英語フィールドを使用する必要あり

### Decision: 曜日の日本語/英語変換

- **Context**: wday 数値の表現形式の決定
- **Alternatives Considered**:
  1. 数値のみ（0-6）
  2. 日本語曜日文言（"日曜日", "月曜日", ...）
  3. 英語曜日名（"Sunday", "Monday", ...）
  4. 上記すべて
- **Selected Approach**: すべて提供
  - `var.wday` - 数値（0-6）
  - `var.曜日` - 日本語文言（"日曜日"）
  - `var.week` - 英語名（"Sunday"）
- **Rationale**: ゴースト開発者の多様なユースケースに対応
- **Trade-offs**: 
  - ✅ 柔軟性が高い
  - ❌ 変換テーブル管理が必要

### Decision: 12時間制変数名は `時１２`（全角数字）

- **Context**: 12時間制表示用変数の命名
- **Alternatives Considered**:
  1. `時12`（半角）
  2. `時１２`（全角）
  3. `時刻`
- **Selected Approach**: `時１２`（全角数字）
- **Rationale**: 開発者の明示的な要望
- **Trade-offs**: 
  - ✅ 全角で統一感がある
  - ❌ 入力しにくい可能性

## Risks & Mitigations

- **既存テスト影響** — execute_scene のシグネチャ変更により既存テストの更新が必要
  - 軽減: 変更ファイル数は少ない（virtual_dispatcher_test.lua, virtual_event_dispatcher_test.rs）
- **パフォーマンス影響** — 転記処理の追加によるオーバーヘッド
  - 軽減: 転記はシンプルなテーブルコピー、OnHour は1時間に1回のみ発火
- **OnTalk への影響** — OnTalk でも同様の転記を行うか
  - 軽減: 現時点では OnHour のみ。OnTalk は将来的な拡張として検討

## References

- [lua_request.rs](../../../crates/pasta_shiori/src/lua_request.rs) - req.date 生成仕様
- [virtual_dispatcher.lua](../../../crates/pasta_lua/scripts/pasta/shiori/event/virtual_dispatcher.lua) - 現在の実装
- [shiori/act.lua](../../../crates/pasta_lua/scripts/pasta/shiori/act.lua) - SHIORI_ACT 実装
- [lua-coding.md](../../steering/lua-coding.md) - Lua コーディング規約
