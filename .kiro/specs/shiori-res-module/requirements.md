# Requirements Document

## Introduction

本ドキュメントは `pasta.shiori.res` モジュールの要件を定義する。SHIORI/3.0プロトコルに準拠したレスポンス文字列を組み立てるためのユーティリティモジュールであり、ステータスコード別の便利関数を提供する。

### 背景

- **対象ファイル**: `crates/pasta_lua/scripts/pasta/shiori/res.lua`
- **参考実装**: `.kiro/specs/shiori-res-module/response.lua`（旧実装）
- **プロトコル**: SHIORI/3.0（伺か用ゴースト通信プロトコル）

### スコープ

**含む**:
- SHIORI/3.0レスポンス文字列の組み立て
- 環境設定（Charset, Sender, SecurityLevel）
- ステータスコード別関数（200, 204, 311, 312, 400, 500）
- 追加ヘッダーの挿入

**含まない**:
- `insert_wait` 依存（ウェイト処理）
- `time_ok`（最終送信時刻の管理）
- `talk` 関数（別モジュールで対応）
- `join` 関数（不要）

---

## Requirements

### Requirement 1: モジュール構造

**Objective:** 開発者として、Luaコーディング規約に準拠したモジュール構造を持つことで、保守性と一貫性を確保したい。

#### Acceptance Criteria

1. The RES module shall export a table named `RES` as the module return value.
2. The RES module shall include LuaDoc annotations (`--- @module pasta.shiori.res`) at the file header.
3. The RES module shall follow the lua-coding steering conventions for module structure (require statements first, module table declaration, local functions, public functions, return).

---

### Requirement 2: 環境設定テーブル

**Objective:** 開発者として、SHIORI レスポンスの共通パラメーターを一元管理できることで、設定変更を容易にしたい。

#### Acceptance Criteria

1. The RES module shall expose `RES.env` table containing default environment settings.
2. The RES module shall provide `RES.env.charset` with default value `"UTF-8"`.
3. The RES module shall provide `RES.env.sender` with default value `"Pasta"`.
4. The RES module shall provide `RES.env.security_level` with default value `"local"`.
5. When `RES.env` values are modified, the RES module shall reflect those changes in subsequent response generation.

---

### Requirement 3: 汎用ビルダー関数

**Objective:** 開発者として、任意のステータスコードとヘッダー辞書からSHIORI/3.0レスポンス文字列を生成できることで、柔軟なレスポンス構築を実現したい。

#### Acceptance Criteria

1. The RES module shall provide `RES.build(code, dic)` function that generates SHIORI/3.0 response string.
2. When `RES.build(code, dic)` is called, the RES module shall output the status line in format `"SHIORI/3.0 " .. code .. "\r\n"`.
3. When `RES.build(code, dic)` is called, the RES module shall output standard headers in the following order:
   - `"Charset: " .. RES.env.charset .. "\r\n"`
   - `"Sender: " .. RES.env.sender .. "\r\n"`
   - `"SecurityLevel: " .. RES.env.security_level .. "\r\n"`
4. When `dic` parameter is a table, the RES module shall append each key-value pair as `key .. ": " .. value .. "\r\n"` after standard headers.
5. When `dic` parameter is nil, the RES module shall generate response without additional headers (standard headers are always included).
6. The RES module shall terminate the response with an empty line (`"\r\n"`).

---

### Requirement 4: 200 OK レスポンス

**Objective:** 開発者として、Valueヘッダー付きの正常レスポンスを簡潔に生成できることで、会話応答処理を効率化したい。

#### Acceptance Criteria

1. The RES module shall provide `RES.ok(value, dic)` function for 200 OK responses.
2. When `RES.ok(value, dic)` is called, the RES module shall set `dic["Value"]` to the provided `value` parameter.
3. When `RES.ok(value)` is called without `dic`, the RES module shall create a new dictionary with only `Value` header.
4. The RES module shall call `RES.build("200 OK", dic)` to generate the final response.

---

### Requirement 5: 204 No Content レスポンス

**Objective:** 開発者として、返すべきデータがない場合の正常終了レスポンスを生成できることで、無応答時の適切なプロトコル準拠を実現したい。

#### Acceptance Criteria

1. The RES module shall provide `RES.no_content(dic)` function for 204 No Content responses.
2. When `RES.no_content()` is called without arguments, the RES module shall generate a valid 204 response.
3. When `RES.no_content(dic)` is called with a dictionary, the RES module shall include the provided headers.

---

### Requirement 6: TEACH関連レスポンス

**Objective:** 開発者として、TEACH リクエストへの応答（情報不足・解釈不能）を適切に返せることで、ユーザー教育機能をサポートしたい。

#### Acceptance Criteria

1. The RES module shall provide `RES.not_enough(dic)` function for 311 Not Enough responses (TEACH情報不足).
2. The RES module shall provide `RES.advice(dic)` function for 312 Advice responses (TEACH解釈不能).
3. When these functions are called, the RES module shall pass the appropriate status code to `RES.build()`.

---

### Requirement 7: エラーレスポンス

**Objective:** 開発者として、リクエスト不備やサーバーエラーを適切に通知できることで、デバッグとエラーハンドリングを容易にしたい。

#### Acceptance Criteria

1. The RES module shall provide `RES.bad_request(dic)` function for 400 Bad Request responses.
2. The RES module shall provide `RES.err(reason, dic)` function for 500 Internal Server Error responses.
3. When `RES.err(reason, dic)` is called, the RES module shall set `dic["X-Error-Reason"]` to the provided `reason`.
4. When `RES.err(reason)` is called without `dic`, the RES module shall create a new dictionary with `X-Error-Reason` header.

---

### Requirement 8: ワーニング付きレスポンス

**Objective:** 開発者として、警告情報付きの204レスポンスを生成できることで、問題を通知しつつ正常終了を示したい。

#### Acceptance Criteria

1. The RES module shall provide `RES.warn(reason, dic)` function for warning responses.
2. When `RES.warn(reason, dic)` is called, the RES module shall set `dic["X-Warn-Reason"]` to the provided `reason`.
3. When `RES.warn(reason)` is called without `dic`, the RES module shall create a new dictionary.
4. The RES module shall return the result of `RES.no_content(dic)` for warning responses.

---

### Requirement 9: エラーハンドリング戦略

**Objective:** 開発者として、オプショナルな引数に対する防御的な処理により、予期しないランタイムエラーを回避したい。

#### Acceptance Criteria

1. The RES module shall apply defensive programming for optional `dic` parameter (e.g., `dic = dic or {}`).
2. When `dic` parameter is `nil`, the RES module shall treat it as an empty table.
3. The RES module shall not perform strict validation (e.g., `assert()`, `error()`) on parameter types, allowing Lua's standard behavior for type errors.
4. The RES module shall handle `nil` values gracefully in all public functions accepting optional parameters.

---

## 参考: SHIORI/3.0 レスポンス形式

```
SHIORI/3.0 200 OK\r\n
Charset: UTF-8\r\n
Value: \0\s[通常]こんにちは。\r\n
\r\n
```

---

## 除外事項（明示的スコープ外）

| 項目 | 理由 |
|------|------|
| `insert_wait` 依存 | 別モジュール（会話処理）で対応 |
| `time_ok`（最終送信時刻） | ステートレスなユーティリティとして設計 |
| `talk` 関数 | 別モジュール（会話処理）で対応 |
| `join` 関数 | 不要（旧実装の残骸） |
| `set_*` 関数群 | `RES.env` テーブルへの直接アクセスで代替（Luaらしいシンプルな設計） |
