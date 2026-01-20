# Research & Design Decisions

## Summary
- **Feature**: `shiori-request-parsed-table`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  - `parse_request`関数は既に実装・テスト済みで、そのまま活用可能
  - `call_lua_request`メソッドの変更は約25行の軽微な修正
  - mluaの`Function::call`はLuaテーブルを直接引数に受け付け可能

## Research Log

### mlua Tableパラメータ互換性
- **Context**: `mlua::Function::call`がLua Tableを引数として受け付けられるか確認
- **Sources Consulted**: 既存コードベース（`lua_request.rs`）、mlua APIドキュメント
- **Findings**:
  - `parse_request`関数は`mlua::Table`を返却する
  - `Function::call`メソッドは`IntoLuaMulti`トレイトを実装した任意の型を受け付ける
  - `Table`は`IntoLua`を実装しており、直接引数として渡すことが可能
- **Implications**: 追加のアダプター層は不要、直接呼び出しが可能

### エラーハンドリングパターン
- **Context**: パース失敗時の400 Bad Request生成方法
- **Sources Consulted**: 既存の`MyError`実装（`error.rs`）、`default_204_response`実装
- **Findings**:
  - `MyError::ParseRequest`は既に定義済み
  - `From<ParseError> for MyError`も実装済み
  - 400レスポンス生成は204レスポンスと同様のパターンで実装可能
- **Implications**: 新規のエラー型追加は不要、レスポンス生成ヘルパー関数1つを追加

### Luaフィクスチャ更新影響
- **Context**: テストフィクスチャの更新範囲を特定
- **Sources Consulted**: `shiori_lifecycle/scripts/pasta/shiori/main.lua`
- **Findings**:
  - 変更対象: 1ファイル（`main.lua`）
  - 変更箇所: `function SHIORI.request(request_text)` → `function SHIORI.request(req)`
  - 関数内部でreqテーブルからフィールドを参照するよう更新が必要
  - 既存テストは関数戻り値を検証しており、Lua内部変更で対応可能
- **Implications**: テストコード（Rust側）の変更は不要、Luaフィクスチャのみ更新

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 直接変更 | `call_lua_request`を直接変更 | 最小限の変更、シンプル | なし（未リリースのため） | 採用 |
| 新規メソッド | 別メソッドを追加 | 後方互換性維持 | コード重複 | 不採用（不要） |

## Design Decisions

### Decision: 完全移行方式の採用
- **Context**: 後方互換性 vs クリーンな実装のトレードオフ
- **Alternatives Considered**:
  1. 段階的移行 — 既存・新規の両メソッドを維持
  2. フィーチャーフラグ — ビルド時切り替え
- **Selected Approach**: 完全移行（既存メソッドを直接変更）
- **Rationale**: 未リリースプロジェクトのため、後方互換性不要
- **Trade-offs**: なし（既存ユーザーがいないため）
- **Follow-up**: Luaフィクスチャの更新を実装タスクに含める

### Decision: 400 Bad Requestレスポンス形式
- **Context**: パース失敗時のエラーレスポンス仕様
- **Alternatives Considered**:
  1. 500 Internal Server Error — サーバー側エラーとして扱う
  2. 204 No Content — エラーを隠蔽
- **Selected Approach**: 400 Bad Request + エラーメッセージ
- **Rationale**: クライアント側のリクエスト形式エラーを正確に伝達
- **Trade-offs**: なし
- **Follow-up**: `default_400_response`ヘルパー関数を追加

## Risks & Mitigations
- **Risk 1**: パースエラー時のエラーメッセージ漏洩 — パースエラーの詳細は含めず、汎用メッセージのみ返却
- **Risk 2**: Luaフィクスチャ更新漏れ — 全テスト実行で検出可能

## References
- [mlua Function API](https://docs.rs/mlua/latest/mlua/struct.Function.html) — call メソッドの引数型
- [SHIORI/3.0 Protocol](https://ssp.shillest.net/ukadoc/manual/spec_shiori3.html) — レスポンスステータスコード仕様
