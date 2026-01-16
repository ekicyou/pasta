# Requirements Document

## Introduction

本仕様は、pasta_shiori クレートにおける SHIORI/3.0 プロトコルの request メソッドの最小実装を定義する。`PastaShiori::request` が Lua スクリプト（main.lua）を経由して「204 No Content」レスポンスを生成し、Rust 側が応答を返すエンドツーエンドの処理フローを実装する。

### 用語定義

| 用語 | 定義 |
|------|------|
| SHIORI/3.0 | 伺かゴースト用スクリプトインターフェース標準プロトコル |
| 204 No Content | リクエスト処理成功、返却値なしを示す SHIORI ステータスコード |
| main.lua | PastaLoader がロードするエントリーポイント Lua スクリプト |
| PastaLuaRuntime | Lua VM を内包し、スクリプト実行を担当するランタイム |

### 参考仕様
- [SHIORI/3.0 仕様書](https://ssp.shillest.net/ukadoc/manual/spec_shiori3.html)

---

## Project Description (Input)
pasta_shioriに対して、最低限のSHIORI.request実装を行う。PastaShiori::requestで、main.luaコードを呼び出して、「何もしない」SHIORI レスポンスを行うコードを実装せよ。なお、PastaLoaderでmain.luaを読み込んでいない場合は、読み込みを行うことも仕様スコープにすること。「PastaShiori::request」が「何もしないレスポンス」を、「lua側で生成して」「Rust側が応答を返す」ところまでに必要な未実装コードすべてがスコープです。

「204 No Content」を返すこと。仕様はこちらを参考。「https://ssp.shillest.net/ukadoc/manual/spec_shiori3.html」

---

## Requirements

### Requirement 1: main.lua エントリーポイント読み込み

**Objective:** ゴースト開発者として、main.lua を起動時に自動ロードしたい。これにより、SHIORI イベント処理のエントリーポイントを Lua 側で定義できる。

#### Acceptance Criteria

1. When PastaLoader がディレクトリを読み込むとき、the PastaLuaRuntime shall `scripts/pasta/shiori/main.lua` をパッケージパスに含めてロードする
2. If main.lua が存在しない場合、the PastaLoader shall エラーを発生させずにデフォルト動作を継続する
3. When main.lua がロードされたとき、the PastaLuaRuntime shall `SHIORI` グローバルテーブルを利用可能にする

---

### Requirement 2: SHIORI.request Lua 関数

**Objective:** ゴースト開発者として、SHIORI リクエストを Lua 側で処理したい。これにより、イベントハンドリングをスクリプトでカスタマイズできる。

#### Acceptance Criteria

1. The `scripts/pasta/shiori/main.lua` shall `SHIORI.request(request_text)` 関数を定義する
2. When SHIORI.request が呼び出されたとき、the SHIORI module shall SHIORI/3.0 形式のレスポンス文字列を返す
3. The SHIORI.request shall 最小実装として "204 No Content" ステータスを含むレスポンスを生成する
4. The SHIORI.request レスポンス shall "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n" 形式に準拠する

---

### Requirement 3: PastaShiori::request Rust 実装

**Objective:** SHIORI DLL 開発者として、Rust から Lua の SHIORI.request を呼び出したい。これにより、プロトコル処理をスクリプト側に委譲できる。

#### Acceptance Criteria

1. When PastaShiori::request が呼び出されたとき、the PastaShiori shall PastaLuaRuntime 経由で Lua の `SHIORI.request` 関数を実行する
2. When SHIORI.request が文字列を返したとき、the PastaShiori::request shall その文字列をそのまま呼び出し元に返す
3. If PastaLuaRuntime が初期化されていない場合、the PastaShiori::request shall `MyError::NotInitialized` エラーを返す
4. If Lua 実行中にエラーが発生した場合、the PastaShiori::request shall 適切なエラー型にラップして返す

---

### Requirement 4: PastaLuaRuntime Lua 関数呼び出し機能

**Objective:** pasta_lua 開発者として、ランタイムから任意の Lua グローバル関数を呼び出したい。これにより、Rust-Lua 間の双方向通信が可能になる。

#### Acceptance Criteria

1. The PastaLuaRuntime shall `call_global` メソッドを提供し、グローバルテーブル上の関数を呼び出せる
2. When call_global が呼び出されたとき、the PastaLuaRuntime shall 指定された引数を Lua 関数に渡す
3. When Lua 関数が値を返したとき、the PastaLuaRuntime shall Rust の `String` 型として結果を返す
4. If 指定された関数が存在しない場合、the PastaLuaRuntime shall 適切なエラーを返す

---

### Requirement 5: SHIORI レスポンスフォーマット

**Objective:** SHIORI プロトコル準拠のため、レスポンスが正しい形式であることを保証したい。

#### Acceptance Criteria

1. The SHIORI response shall "SHIORI/3.0 {status_code} {status_text}" で始まるステータス行を含む
2. The SHIORI response shall "Charset: UTF-8" ヘッダーを含む
3. The SHIORI response shall "Sender: Pasta" ヘッダーを含む
4. The SHIORI response shall 各行を CRLF (`\r\n`) で区切る
5. The SHIORI response shall 空行 (`\r\n\r\n`) でヘッダー終端を示す

---

### Requirement 6: テスト可能性

**Objective:** 品質保証担当者として、SHIORI request 処理が正しく動作することを検証したい。

#### Acceptance Criteria

1. The PastaShiori shall request 処理のユニットテストを提供する
2. When テストが実行されたとき、the test suite shall 204 No Content レスポンスを検証する
3. The test suite shall Lua エラー時の挙動を検証する
4. The test suite shall 既存の load/unload テストとの整合性を維持する
