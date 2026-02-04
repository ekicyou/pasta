# Research & Design Decisions: refine-talk-conversion

---
**Purpose**: 調査結果と設計判断の根拠を記録する。

---

## Summary
- **Feature**: `refine-talk-conversion`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  - `@pasta_sakura_script`モジュールは実装済みで、`talk_to_script`関数が利用可能
  - `token.actor`でactorオブジェクトが既に利用可能（`group_by_actor()`経由）
  - pasta.toml `[actor."名前"]`セクションでキャラクター固有のウェイト設定が自動適用される

## Research Log

### 既存実装の調査

- **Context**: sakura_builder.luaの現行実装と置換対象の特定
- **Sources Consulted**: 
  - [sakura_builder.lua](../../crates/pasta_lua/scripts/pasta/shiori/sakura_builder.lua)
  - [gap-analysis.md](./gap-analysis.md)
- **Findings**:
  - `escape_sakura`関数は行12-17で定義（単純なエスケープ処理のみ）
  - 行96で`escape_sakura(inner.text)`が呼び出されている
  - 行78で`local actor = token.actor`として既にactorオブジェクトが利用可能
- **Implications**: 変更は最小限で済む（require追加、呼び出し置換、関数削除の3点）

### @pasta_sakura_scriptモジュールの確認

- **Context**: 置換先となるモジュールのAPI確認
- **Sources Consulted**: 
  - [mod.rs](../../crates/pasta_lua/src/sakura_script/mod.rs) 行100-130
  - [LUA_API.md](../../crates/pasta_lua/LUA_API.md)
- **Findings**:
  - `talk_to_script(actor, text)` - actorテーブルとテキストを受け取る
  - `resolve_wait_values`関数（行111-125）がactorからウェイト値を読み取る
  - actor.`script_wait_*`フィールドから直接読み取り、なければデフォルト値にフォールバック
  - 3段階フォールバック: actor → config → ハードコード値
- **Implications**: actor.talkサブテーブル不要。actorテーブル直下のフィールドを参照

### actorオブジェクトの構造調査

- **Context**: token.actorの内容確認
- **Sources Consulted**: 
  - [act.lua](../../crates/pasta_lua/scripts/pasta/shiori/act.lua)（group_by_actor関数）
  - [store.lua](../../crates/pasta_lua/scripts/pasta/store.lua) 行76-79
  - [actor.lua](../../crates/pasta_lua/scripts/pasta/actor.lua) 行151-160
- **Findings**:
  - `STORE.actors = CONFIG.actor`で参照共有
  - pasta.toml `[actor."名前"]`の設定が自動的にactorオブジェクトに含まれる
  - `script_wait_normal`, `script_wait_period`等のフィールドを直接設定可能
- **Implications**: 追加実装不要。pasta.tomlで設定すればtalk_to_scriptが自動適用

### テスト影響調査

- **Context**: 期待値変更が必要なテストの特定
- **Sources Consulted**: 
  - sakura_builder_test.lua（24テスト）
- **Findings**:
  - 現在のテストは`escape_sakura`ベースの出力を期待
  - 変更後はウェイトタグ（`\_w[ms]`）が挿入される形式に
  - 出力形式が変わるため全テストケースの期待値更新が必要
- **Implications**: テスト期待値更新を実装スコープに含める（Requirement 6）

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 最小変更 | sakura_builder.luaのみ変更 | 変更範囲最小、既存パターン維持 | なし | 推奨 |
| テスト駆動 | テスト更新先行 | 回帰リスク低減 | 工数増加 | 不要 |
| 機能フラグ | 新旧動作切り替え | ロールバック容易 | 複雑性増加 | 過剰 |

## Design Decisions

### Decision: 最小変更アプローチの採用

- **Context**: sakura_builder.luaのトーク変換処理をtalk_to_scriptに置き換える
- **Alternatives Considered**:
  1. 最小変更 — sakura_builder.luaのみ変更
  2. テスト駆動 — テスト更新を先行
  3. 機能フラグ — 新旧動作切り替え可能
- **Selected Approach**: 最小変更アプローチ
- **Rationale**: 
  - 既存の`@pasta_sakura_script`モジュールが完全に実装済み
  - 変更箇所が明確で限定的
  - テストカバレッジがあり回帰リスクが低い
- **Trade-offs**: 
  - ✅ 変更範囲最小
  - ✅ 既存パターン維持
  - ❌ テスト期待値の更新が必要（スコープ内）
- **Follow-up**: テスト期待値更新を同一PRで実施

### Decision: actorオブジェクトをそのまま使用

- **Context**: talk_to_scriptに渡すactorパラメーターの形式
- **Alternatives Considered**:
  1. token.actorをそのまま渡す
  2. actor.talkサブテーブルを構築して渡す
- **Selected Approach**: token.actorをそのまま渡す
- **Rationale**:
  - `resolve_wait_values`がactorテーブル直下の`script_wait_*`フィールドを読み取る設計
  - pasta.toml `[actor."名前"]`で設定したフィールドがそのまま適用される
  - 追加の変換処理が不要
- **Trade-offs**:
  - ✅ シンプル
  - ✅ 既存設定機構を活用
  - ✅ 追加実装不要
- **Follow-up**: なし

## Risks & Mitigations

- **テスト期待値の更新漏れ** — 変更影響のあるテストを網羅的に確認し更新する
- **ウェイトタグによる出力長増加** — さくらスクリプト仕様の範囲内であり問題なし
- **既存ゴーストの動作変更** — 会話テンポの向上であり、破壊的変更ではない

## References

- [LUA_API.md](../../crates/pasta_lua/LUA_API.md) — @pasta_sakura_scriptモジュールAPI仕様
- [gap-analysis.md](./gap-analysis.md) — 実装ギャップ分析
- [requirements.md](./requirements.md) — 要件定義
