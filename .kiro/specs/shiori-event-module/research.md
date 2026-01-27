# Research & Design Decisions: shiori-event-module

---
**Purpose**: Gap分析と設計議論から得られた発見と決定事項を記録する。

---

## Summary
- **Feature**: `shiori-event-module`
- **Discovery Scope**: Extension（既存`pasta.shiori`システムへの追加）
- **Key Findings**:
  - `pasta.shiori.res` モジュールが完成済みで、レスポンス生成APIをそのまま活用可能
  - Rust側 `lua_request::parse_request()` により `req.id` を含むリクエストテーブルが完備
  - `lua-coding.md` に従ったモジュール構造パターンが確立済み

## Research Log

### Luaモジュール構造パターン
- **Context**: 新規イベントモジュールの構造を既存パターンに合わせる必要があった
- **Sources Consulted**: `lua-coding.md`, `pasta/shiori/res.lua`, `pasta/store.lua`
- **Findings**:
  - モジュールテーブル名は UPPER_CASE（例: `RES`, `EVENT`, `REG`）
  - require文 → テーブル宣言 → ローカル関数 → 公開関数 → return の順序
  - 循環参照回避: `register.lua` は依存ゼロ設計
- **Implications**: `pasta.shiori.res` と同一構造を採用

### SHIORI/3.0プロトコル制約
- **Context**: エラーレスポンスにスタックトレースを含める可否
- **Sources Consulted**: SHIORI/3.0仕様、`res.lua` 実装
- **Findings**:
  - HTTPライクなヘッダー形式で、ヘッダー値に改行を含めると不正なレスポンスになる
  - `X-Error-Reason` は単一行である必要がある
- **Implications**: エラーメッセージの最初の行のみを抽出する設計を採用

### Rust側リクエストテーブル構造
- **Context**: `EVENT.fire(req)` が受け取る `req` テーブルの構造確認
- **Sources Consulted**: `lua_request.rs`, `lua_request_test.rs`
- **Findings**:
  - `req.id` にイベント名（例: `"OnBoot"`）が格納される
  - `req.method`, `req.version`, `req.charset`, `req.sender`, `req.reference` 等が利用可能
  - Rust側で `parse_request()` が呼ばれ、Luaには完成済みテーブルが渡される
- **Implications**: EVENT側でのパース処理は不要、`req.id` でハンドラ振り分け可能

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A. main.lua拡張 | main.lua内に振り分けロジック追加 | ファイル数最小 | 責務混在、テスト困難 | 非採用 |
| B. 新規サブモジュール | event/init.lua + event/register.lua | 責任分離、テスト容易、拡張性 | ファイル数+2 | **採用** |
| C. Hybrid | 段階的導入 | リスク分散 | 複数PR、中途半端な状態 | 不要 |

## Design Decisions

### Decision: main.lua統合はスコープ外
- **Context**: EVENT.fireをmain.lua内で呼び出すかの判断
- **Alternatives Considered**:
  1. Lua側main.luaでEVENT.fire呼び出し
  2. Rust側でEVENT.fire呼び出し（採用）
- **Selected Approach**: Rust側（pasta_shiori）で統合を行う
- **Rationale**: EVENT.fireは独立したツールとして実装し、統合責任をRust側に委譲
- **Trade-offs**: Lua側の完結性は下がるが、Rust側での制御が容易になる
- **Follow-up**: Rust側統合は別仕様で対応

### Decision: req.id=nil防御はLua標準挙動に任せる
- **Context**: req.idがnilの場合の振る舞い
- **Alternatives Considered**:
  1. Lua標準挙動（REG[nil] → nil → no_entry）（採用）
  2. 明示的nilチェック
- **Selected Approach**: `REG[req.id] or EVENT.no_entry` のシンプルな実装
- **Rationale**: Luaの短絡評価を活用、既存res.luaとの一貫性
- **Trade-offs**: 暗黙的だが、Lua開発者には自然な挙動
- **Follow-up**: Rust側がreqテーブルを必ず渡す前提

### Decision: エラーメッセージは最初の行のみ
- **Context**: xpcallでキャッチしたエラーをレスポンスに含める方法
- **Alternatives Considered**:
  1. traceback全体（改行含む）
  2. debug_modeフラグ制御
  3. 最初の行のみ抽出（採用）
- **Selected Approach**: `result:match("^[^\n]+") or "Unknown error"`
- **Rationale**: SHIORI/3.0ヘッダーに改行を含めるとプロトコル違反
- **Trade-offs**: デバッグ情報は限定的だが、プロトコル準拠を優先
- **Follow-up**: 将来的にログファイル出力機能を検討

## Risks & Mitigations
- **req.idがnilの場合** → Lua標準挙動で自然にno_entryへフォールバック
- **ハンドラ内エラー** → xpcallで捕捉し500レスポンスに変換
- **循環参照** → register.luaは依存ゼロ設計で回避

## References
- [lua-coding.md](.kiro/steering/lua-coding.md) — Luaコーディング規約
- [pasta.shiori.res](crates/pasta_lua/scripts/pasta/shiori/res.lua) — レスポンス組み立てモジュール
- [lua_request.rs](crates/pasta_shiori/src/lua_request.rs) — Rust側リクエストパース
- [gap-analysis.md](.kiro/specs/shiori-event-module/gap-analysis.md) — 詳細なギャップ分析
