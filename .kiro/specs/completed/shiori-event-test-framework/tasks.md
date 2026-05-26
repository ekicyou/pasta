# Implementation Plan

## 1. 基盤設定

- [ ] 1. 基盤設定: time crate parsing feature 追加
- [x] 1.1 ワークスペース `Cargo.toml` の `time` 依存に `parsing` feature を追加する
  - `[workspace.dependencies]` の `time` エントリが `features = ["local-offset", "parsing"]` になっていること
  - `cargo check -p pasta_shiori` がコンパイルエラーなく通ること
  - _Requirements: 1.1, 1.3, 1.4_
  - _Boundary: Cargo.toml workspace_

## 2. エラーモデル拡張

- [ ] 2. エラーモデル拡張: 400 レスポンス生成を MyError に統一する
- [x] 2.1 `MyError::InvalidPastaTime { value, reason }` バリアントと `to_shiori_400_response()` メソッドを `error.rs` に追加する
  - `MyError::InvalidPastaTime { value: String, reason: String }` が `thiserror::Error` マクロで `"Invalid X-Pasta-Time header value '{value}': {reason}"` メッセージを持つこと
  - `to_shiori_400_response()` が `SHIORI/3.0 400 Bad Request\r\nCharset: UTF-8\r\nX-ERROR-REASON: {self}\r\n\r\n` 形式の文字列を返すこと
  - `Sender: Pasta` ヘッダーはエラーレスポンスに含まれないこと（設計判断: プロトコルメタデータはディスパッチ層の責務）
  - 既存の `to_shiori_response()`（500用）が変更なしで維持されていること
  - _Requirements: 1.3_
  - _Boundary: pasta_shiori / src/error.rs_
  - _Depends: 1.1_

- [x] 2.2 `PastaShiori::call_lua_request` のパースエラー経路を `MyError::to_shiori_400_response()` に切り替え、`default_400_response()` を削除する
  - `call_lua_request` のパース失敗パスが `e.to_shiori_400_response()` を返すこと
  - `default_400_response()` メソッドが `shiori.rs` から削除されていること
  - `cargo check -p pasta_shiori` がエラーなく通ること
  - _Requirements: 1.3_
  - _Boundary: pasta_shiori / src/shiori.rs_

- [x] 2.3 `shiori_tests.rs` の `test_default_400_response_format()` テストを `MyError::to_shiori_400_response()` ベースに書き換える
  - `test_default_400_response_format()` が `MyError::to_shiori_400_response()` を直接テストする形に変更されていること
  - `cargo test -p pasta_shiori` で当該テストが green になること
  - _Requirements: 1.3, 5.2_
  - _Boundary: pasta_shiori / src/shiori_tests.rs_

## 3. X-Pasta-Time 時刻注入

- [ ] 3. X-Pasta-Time 時刻注入: リクエストパーサーへの決定論的時刻注入
- [x] 3.1 `parse_request()` に `X-Pasta-Time` ヘッダー検出・RFC 3339 パース・`req.date` 上書きロジックを追加する
  - `parse1()` 完了後に `dic["X-Pasta-Time"]` の有無を確認し、有効な RFC 3339 値なら `lua_date_from(lua, dt)?` で `req.date` を上書きすること
  - 不正な値の場合は `tracing::error!` でログ出力し、`Err(MyError::InvalidPastaTime { value, reason })` を返すこと
  - `X-Pasta-Time` ヘッダーがない場合は従来動作（`now_local` ベースの `req.date`）を維持すること
  - 既存の `parse_request()` 関数シグネチャが変更されていないこと（後方互換）
  - `cargo check -p pasta_shiori` がエラーなく通ること
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 5.1_
  - _Boundary: pasta_shiori / src/lua_request.rs_

- [x] 3.2 `lua_request_test.rs` に `X-Pasta-Time` の各シナリオのテストを追加する
  - 有効な RFC 3339 ヘッダー付きリクエスト → `req.date` の `year`/`month`/`day`/`hour` 等がヘッダー値と一致することを確認するテストが存在すること
  - ヘッダー無しリクエスト → 処理が成功し `req.date` が取得できること
  - 不正値ヘッダー付きリクエスト → `MyError::InvalidPastaTime` が返ること
  - タイムゾーンオフセット付き値（例: `+09:00`）→ 変換後フィールドが正確なこと
  - `cargo test -p pasta_shiori` で追加テストを含む全テストが green になること
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  - _Boundary: pasta_shiori / tests/lua_request_test.rs_

## 4. Lua モックライブラリ

- [ ] 4. Lua モックライブラリ: テスト用バックエンドモジュール一括スタブ
- [x] 4.1 (P) `scriptlibs/lua_test/mocks.lua` を作成し、5モジュールのデフォルトスタブと `install()`/`reset()` API を実装する
  - `@pasta_persistence`, `@pasta_search`, `@pasta_sakura_script`, `@pasta_config`, `@pasta_log` の全モジュールにデフォルトスタブが定義されていること
  - `install()` 呼び出し後、`package.loaded["@pasta_persistence"]` 等が nil でないこと
  - `install({ persistence = {...} })` でカスタムスタブが使われること（デフォルトを置換、マージしない）
  - `reset()` 呼び出し後、5モジュールの `package.loaded` エントリが全て `nil` になること
  - `@pasta_search` はメタテーブルキャッチオール方式（`__index` で任意メソッド呼び出し時に `nil` を返す関数）で実装されていること
  - ファイルが `crates/pasta_lua/scriptlibs/lua_test/mocks.lua` として配置されていること
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_
  - _Boundary: pasta_lua / scriptlibs/lua_test/mocks.lua_

- [x] 4.2 (P) `pasta_lua` のテストスイートに `mocks.lua` の動作検証テストを追加する
  - `install()` → 全5モジュールが `package.loaded` に登録される Lua テストが存在すること
  - `install({ log = {...} })` → カスタムスタブが適用される（デフォルト置換）Lua テストが存在すること
  - `reset()` → 全エントリが `nil` になる Lua テストが存在すること
  - `cargo test -p pasta_lua` で追加テストを含む全テストが green になること
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_
  - _Boundary: pasta_lua / tests_

## 5. ShioriResponse パーサー

- [ ] 5. ShioriResponse パーサー: SHIORI/3.0 レスポンスの構造化分解
- [x] 5.1 (P) `tests/common/response.rs` に `ShioriResponse` 構造体と `parse()` / `header()` / `is_success()` を実装する
  - `ShioriResponse::parse(text)` が `status_code: u16`, `status_text: String`, `headers: HashMap<String,String>`, `value: Option<String>` を正しく分解すること
  - `header(name)` メソッドが大文字小文字を区別せずにヘッダー値を返すこと
  - `is_success()` が `status_code` が 200–299 の場合に `true` を返すこと
  - 不正入力（空文字列、ステータス行なし）で `ShioriResponseError` が返りパニックしないこと
  - `ShioriResponseError` バリアント（`Empty`, `MissingStatusLine`, `InvalidStatusLine`, `InvalidHeaderLine`）が定義されていること
  - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - _Boundary: pasta_shiori / tests/common/response.rs_
  - _Depends: 1.1_

- [x] 5.2 (P) `tests/shiori_response_test.rs` に `ShioriResponse::parse()` の各ケーステストを作成する
  - 200 OK（`Value` あり）のパース確認テスト
  - 204 No Content（`Value` なし → `value == None`）のパース確認テスト
  - 400 / 500 レスポンスのステータスコード・テキストのパース確認テスト
  - `X-Error-Reason` 等の複数カスタムヘッダーが全て保持され `header()` で取得できるテスト
  - 空文字列入力で `ShioriResponseError::Empty` が返るテスト
  - `cargo test -p pasta_shiori` で全テストが green になること
  - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - _Boundary: pasta_shiori / tests/shiori_response_test.rs_

## 6. ShioriTestEnv 統合テスト環境

- [ ] 6. ShioriTestEnv 統合テスト環境: フィクスチャ準備〜SHIORI往復〜検証の一体化
- [x] 6.1 `tests/common/test_env.rs` に `ShioriTestEnv` 構造体と `new` / `request` / `runtime` / `path` を実装する
  - `ShioriTestEnv::new(fixture)` が `copy_fixture_to_temp(fixture)` + `PastaShiori::load()` を実行し、失敗時は panic すること（テスト用途のため許容）
  - `request(&mut self, text)` が SHIORI リクエストを処理し `Result<ShioriResponse, ShioriRequestError>` を返すこと
  - `runtime(&self)` が内部 `PastaLuaRuntime` への参照を返すこと
  - `Drop` 時に `TempDir` が自動削除されること
  - `ShioriRequestError` が `Shiori(#[from] MyError)` と `Parse(#[from] ShioriResponseError)` の2バリアントを持つこと
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_
  - _Boundary: pasta_shiori / tests/common/test_env.rs_
  - _Depends: 2.2, 3.1, 5.1_

- [x] 6.2 `tests/common/mod.rs` を更新し、`response` と `test_env` モジュールを公開する
  - `mod.rs` に `pub mod response; pub mod test_env; pub use response::*; pub use test_env::*;` が追記されていること
  - `cargo check -p pasta_shiori` がエラーなく通ること
  - _Requirements: 4.1, 4.3_
  - _Boundary: pasta_shiori / tests/common/mod.rs_

- [x] 6.3 `tests/shiori_test_env_test.rs` に `ShioriTestEnv` のライフサイクルと end-to-end テストを作成する
  - フィクスチャコピー → load → `request()` → `ShioriResponse` 取得が成功するテスト
  - 同一 `ShioriTestEnv` への複数 `request()` 呼び出しで Lua ランタイム状態（グローバル変数等）が維持されるテスト
  - `runtime()` 経由で Lua グローバル変数を検査するテスト
  - `X-Pasta-Time` ヘッダー付きリクエストで Lua 側ハンドラが固定時刻を観測することを確認する end-to-end テスト
  - 不正な `X-Pasta-Time` 値を含むリクエストで `status_code == 400` かつ `header("X-ERROR-REASON")` に詳細が含まれるテスト
  - `cargo test -p pasta_shiori` で全テストが green になること
  - _Requirements: 1.1, 1.3, 4.1, 4.2, 4.3, 4.4, 4.5_
  - _Boundary: pasta_shiori / tests/shiori_test_env_test.rs_

## 7. 後方互換性検証

- [ ] 7. 後方互換性検証: 既存テスト全パスの確認
- [x] 7.1 変更後のワークスペース全テストを実行し、既存テストが変更なしで成功することを確認する
  - `cargo test --workspace` が全クレートで green になること（新規テストを含む）
  - 新規実装タスクで触れていない既存テストファイルに変更が不要であること
  - _Requirements: 5.1, 5.2_
  - _Boundary: workspace_
