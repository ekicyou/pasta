# Gap Analysis: shiori-event-test-framework

## 要件→既存資産マッピング

### Requirement 1: SHIORIリクエスト時刻制御

| 技術ニーズ                       | 既存資産                                                                                                  | ギャップ                                                                                                    |
| -------------------------------- | --------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| `X-Pasta-Time` ヘッダーのパース  | PEG文法の `key_other` ルール → `dic["X-Pasta-Time"]` に自動格納                                           | **なし** — 文法変更不要                                                                                     |
| RFC 3339 日時文字列のパース      | `time` クレート `0.3` 使用中。ただし `parsing` feature **未有効**（現在は `local-offset` のみ）           | **Missing** — `parsing` + `formatting` feature 追加が必要                                                   |
| 固定時刻での `req.date` 生成     | `lua_date_from(lua, dt: OffsetDateTime)` が public で存在                                                 | **なし** — そのまま利用可能                                                                                 |
| `parse_request()` での時刻上書き | `parse_request()` は `lua_date(lua)?` で `req.date` を先にセットし、`parse1()` 後に返却。上書きフックなし | **Missing** — `parse1()` 後に `dic["X-Pasta-Time"]` を検出し `lua_date_from()` で上書きするコード追加が必要 |

**変更影響分析**:
- `parse_request()` への変更は **4行程度の追加**（dic取得→ヘッダー検出→パース→date上書き）
- `time` crateのfeature追加はワークスペースレベル `Cargo.toml` の1行変更（`features = ["local-offset", "parsing"]`）
- `lua_date_from()` は既に分離済みのため、新規関数不要
- 後方互換性: ヘッダーなしの場合は従来パスを通るため **影響なし**

### Requirement 2: Luaモック一括注入ライブラリ

| 技術ニーズ                    | 既存資産                                                            | ギャップ                             |
| ----------------------------- | ------------------------------------------------------------------- | ------------------------------------ |
| `@pasta_persistence` スタブ   | `common/mod.rs` L42-49 にインラインモック（`load→{}`, `save→true`） | **Constraint** — 3箇所に重複実装あり |
| `@pasta_search` スタブ        | `common/mod.rs` L51-60 にメタテーブルモック（全メソッド→nil）       | 同上                                 |
| `@pasta_sakura_script` スタブ | `common/mod.rs` L62-73 にパススルーモック（`talk_to_script→text`）  | 同上                                 |
| `@pasta_config` スタブ        | `virtual_event_config_test.rs` にインラインモック                   | **Missing** — 共通化されていない     |
| `@pasta_log` スタブ           | `lua_unittest_runner.rs` で実体登録（`log::register()`）            | **Missing** — Luaスタブが存在しない  |
| 一括インストール関数          | なし                                                                | **Missing**                          |
| リセット関数                  | 各lua_specが `package.loaded[...] = nil` を個別実行                 | **Missing** — 共通化されていない     |

**配置先分析**:
- `scriptlibs/lua_test/` に既存の `test.lua`, `expect.lua` が存在 — テストフレームワークの論理的配置場所
- `pasta_scripts/` は本番ランタイムコード — テスト専用モックの配置先としては **不適切**
- **推奨**: `scriptlibs/lua_test/mocks.lua` として配置

**Rust側モック重複**:
- `pasta_lua/tests/common/mod.rs` の `create_runtime_with_pasta_path()` — Luaインライン文字列でモック
- `pasta_lua/tests/common/e2e_helpers.rs` の `create_runtime_with_finalize()` — Rust実体モジュールを登録
- `pasta_lua/tests/lua_unittest_runner.rs` — Rust実体モジュールを登録
- この3箇所の重複は **Lua側モックライブラリ導入で部分的に解消可能**（Luaインライン文字列モックを `mocks.lua` の `require` に置換）

### Requirement 3: SHIORIレスポンス構造化検証

| 技術ニーズ                         | 既存資産                                       | ギャップ    |
| ---------------------------------- | ---------------------------------------------- | ----------- |
| レスポンスステータスコードのパース | なし（テスト側は `contains("200 OK")` で検証） | **Missing** |
| Valueフィールドの抽出              | なし（テスト側は `contains("Value:")` で検証） | **Missing** |
| ヘッダーのキー・値ペア分解         | なし                                           | **Missing** |
| パースエラーハンドリング           | なし                                           | **Missing** |

**レスポンスフォーマット分析**（`res.lua` より）:
```
SHIORI/3.0 {code} {status_text}\r\n
Charset: UTF-8\r\n
Sender: Pasta\r\n
SecurityLevel: local\r\n
{追加ヘッダー}\r\n
\r\n
```
- 改行は CRLF (`\r\n`)
- ヘッダーとボディの区切りは空行（`\r\n\r\n`）
- 標準3ヘッダーは固定順序、追加ヘッダーは順序不定（Lua `pairs()` イテレーション）
- マルチライン値の処理なし（Valueはそのまま単一行）

**実装選択肢**:
- A: 既存のPEG文法 (`req_parser.pest`) を拡張してレスポンスもパース — 重量
- B: Rustの文字列処理で簡易パーサーを実装 — `\r\n\r\n` split + ヘッダー行パースで十分
- C: Lua側でレスポンスパーサーを実装 — Luaテスト用途には良いが pasta_shiori のRustテストでは使えない

### Requirement 4: SHIORIテスト環境セットアップ

| 技術ニーズ                             | 既存資産                                                                 | ギャップ                                                           |
| -------------------------------------- | ------------------------------------------------------------------------ | ------------------------------------------------------------------ |
| フィクスチャ→一時ディレクトリコピー    | `copy_fixture_to_temp()` が pasta_shiori と pasta_lua に**別実装で存在** | **Constraint** — 重複だが統合コスト高（テストutility crateが必要） |
| サンプルゴースト利用                   | `copy_sample_ghost_to_temp()` が pasta_shiori/tests/common に存在        | **なし** — そのまま利用可能                                        |
| PastaShiori の load + request ラッパー | なし                                                                     | **Missing**                                                        |
| TempDir 自動クリーンアップ             | `tempfile::TempDir` を使用中（drop時自動削除）                           | **なし** — パターン確立済み                                        |
| Luaランタイムへの直接アクセス          | `PastaShiori::runtime()` が `Option<&PastaLuaRuntime>` を返す            | **なし** — 既存 public API                                         |

**統合ポイント**:
- `ShioriTestEnv` は `PastaShiori` + `TempDir` をラップするだけで実現可能
- `request()` のラッパーは `PastaShiori::request()` に委譲 + `ShioriResponse::parse()` で構造化

### Requirement 5: 後方互換性

| 技術ニーズ                   | 既存資産                                            | ギャップ                                       |
| ---------------------------- | --------------------------------------------------- | ---------------------------------------------- |
| 既存テスト不変               | 全テスト（950+ passing）                            | **なし** — `X-Pasta-Time` なしは従来パスを通過 |
| `parse_request()` の後方互換 | `parse1()` 後に上書きする形なので既存動作に影響なし | **なし**                                       |

---

## 実装アプローチ選択肢

### Option A: 最小拡張（Extend Existing）

既存ファイルへの変更のみで実現:
- `lua_request.rs` に X-Pasta-Time 処理を追加（4行）
- `scriptlibs/lua_test/mocks.lua` を新規作成（1ファイル）
- `pasta_shiori/tests/common/mod.rs` に `ShioriResponse` 構造体と `ShioriTestEnv` を追加

**Trade-offs**:
- ✅ 変更ファイル数最小（3ファイル）
- ✅ 既存パターンに沿った拡張
- ❌ `tests/common/mod.rs` が肥大化する可能性
- ❌ feature gate なしのため `ShioriResponse` が test binary 以外からはアクセス不能

### Option B: Feature Gate 付き新規モジュール（New Components）

`pasta_shiori` に `test-harness` feature を追加し、テストユーティリティを独立モジュールとして配置:
- `pasta_shiori/src/test_harness.rs` — `#[cfg(feature = "test-harness")]` ゲート
- `pasta_shiori/src/test_harness/response.rs` — ShioriResponse
- `pasta_shiori/src/test_harness/env.rs` — ShioriTestEnv

**Trade-offs**:
- ✅ 他クレートの `dev-dependencies` から利用可能
- ✅ 本番バイナリに含まれない
- ❌ ワークスペース初の feature gate パターン導入（前例なし）
- ❌ `src/` 内にテスト専用コードが混在

### Option C: ハイブリッド（推奨）

- `lua_request.rs` への時刻注入は直接変更（Option A と同じ）
- `scriptlibs/lua_test/mocks.lua` は新規Luaファイル（Option A と同じ）
- `ShioriResponse` と `ShioriTestEnv` は `pasta_shiori/tests/common/` 内の **別ファイル** として配置
  - `tests/common/response.rs` — レスポンスパーサー
  - `tests/common/test_env.rs` — テスト環境ラッパー
  - `tests/common/mod.rs` — 既存 + 新モジュールの re-export

**Trade-offs**:
- ✅ 変更が論理的に分離される
- ✅ 既存パターン（`tests/common/`）の自然な拡張
- ✅ feature gate の新パターン導入不要
- ✅ `src/` は本番コードのみ（`lua_request.rs` の4行変更を除く）
- ❌ `tests/common/` の構造体は同一 test binary 内でのみ利用可能（他クレートからは不可）
- ❌ 将来 `shiori-async-talk` が他クレートからテスト環境を使いたい場合、feature gate への移行が必要になる可能性

---

## 工数・リスク評価

| 項目                       | 工数          | リスク | 根拠                                                                   |
| -------------------------- | ------------- | ------ | ---------------------------------------------------------------------- |
| 全体                       | **S** (1–3日) | **低** | 既存パターンの拡張、既知の技術スタック                                 |
| Req 1: 時刻注入            | S (数時間)    | 低     | `lua_date_from()` 既存、PEG文法変更不要、`time` crateのfeature追加のみ |
| Req 2: Luaモックライブラリ | S (数時間)    | 低     | 既存モックパターンの集約、新規Luaファイル1つ                           |
| Req 3: レスポンスパーサー  | S (数時間)    | 低     | CRLF区切りの単純なテキストパース                                       |
| Req 4: テスト環境統合      | S (半日)      | 低     | 既存コンポーネントのラッパー                                           |
| Req 5: 後方互換性          | —             | 低     | 追加的変更のみ                                                         |

---

## 設計フェーズへの推奨事項

### 推奨アプローチ: Option C（ハイブリッド）
- 本番コードへの変更を最小化（`lua_request.rs` の時刻注入のみ）
- テストユーティリティは `tests/common/` 内に論理的に分離配置
- feature gate パターンの導入は見送り（ワークスペース初のパターンとなるため慎重に）

### 設計フェーズで検討すべき項目

1. **`time` crateのfeature追加**: `parsing` feature の追加による依存サイズ・コンパイル時間への影響確認
2. **レスポンスパーサーの実装方式**: Rust文字列処理 vs PEG拡張（推奨: 文字列処理 — PEG拡張は過剰）
3. **`tests/common/` のモジュール構造**: `mod.rs` + `response.rs` + `test_env.rs` の分割粒度
4. **Luaモックライブラリのデフォルトスタブ仕様**: `@pasta_search` のメタテーブルモック（全メソッド→nil）を維持するか、明示的メソッド定義に切り替えるか
5. **将来の他クレートからの利用**: `shiori-async-talk` が `pasta_lua` 側から `ShioriTestEnv` を使う可能性がある場合、feature gate への段階的移行パスを設計に含める

6. **400 Bad Request のエラー詳細返却**: 現在 `default_400_response()` は固定文字列で `X-ERROR-REASON` なし。`to_shiori_response()` は 500 専用。`X-Pasta-Time` 不正値のような 400 エラーでも `X-ERROR-REASON` にエラー詳細を載せるべき。`to_shiori_400_response(&self) -> String` のような詳細付き 400 レスポンス生成メソッドを `MyError` に追加する方向で設計する（`to_shiori_response()` の 400 版）。

### Research Needed
- `time` crate `parsing` feature のバイナリサイズ・コンパイル時間影響（実測推奨）
