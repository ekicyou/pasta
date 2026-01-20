# Requirements Document

## Introduction

本仕様は、SHIORI.request関数にリクエスト解析済みのLuaテーブルを渡す機能を定義する。現在の実装では生のリクエストテキストが渡されているが、lua_request.rsに実装済みの`parse_request`関数を活用し、解析済みテーブルを`req`パラメータとしてLua側に渡すことで、Luaスクリプト側でのリクエスト解析処理を不要にする。

## Project Description (Input)
lua_request.rsを使って、lua側のSHIORI.requestに、リクエスト解析済みのテーブルをreqとして渡すようにしてください。

## Requirements

### Requirement 1: リクエスト解析テーブルの生成

**Objective:** As a Luaスクリプト開発者, I want SHIORIリクエストが解析済みテーブルとして渡されること, so that Lua側で独自のパース処理を実装する必要がなくなる

#### Acceptance Criteria
1. When SHIORI.requestが呼び出される, the PastaShiori shall `lua_request::parse_request`を使用してリクエストテキストを解析する
2. When リクエスト解析が成功する, the PastaShiori shall 解析結果を`req`テーブルとしてLua側に渡す
3. If リクエスト解析が失敗する, then the PastaShiori shall エラーログを出力し、Luaを呼ばずにRust側で`SHIORI/3.0 400 Bad Request`レスポンスを返す

### Requirement 2: 解析済みテーブル構造

**Objective:** As a Luaスクリプト開発者, I want 解析済みテーブルが一貫した構造を持つこと, so that 予測可能な方法でリクエストデータにアクセスできる

#### Acceptance Criteria
1. The PastaShiori shall 解析済みテーブルに`method`フィールド（"get"または"notify"）を含める
2. The PastaShiori shall 解析済みテーブルに`version`フィールド（SHIORIバージョン番号）を含める
3. The PastaShiori shall 解析済みテーブルに`id`フィールド（イベントID）を含める
4. The PastaShiori shall 解析済みテーブルに`reference`テーブル（Reference0〜n）を含める
5. The PastaShiori shall 解析済みテーブルに`dic`テーブル（全ヘッダーの辞書）を含める
6. Where charsetヘッダーが含まれる, the PastaShiori shall `charset`フィールドを解析済みテーブルに設定する
7. Where base_idヘッダーが含まれる, the PastaShiori shall `base_id`フィールドを解析済みテーブルに設定する
8. Where senderヘッダーが含まれる, the PastaShiori shall `sender`フィールドを解析済みテーブルに設定する

### Requirement 3: SHIORI.request関数シグネチャの変更（破壊的変更）

**Objective:** As a Luaスクリプト開発者, I want SHIORI.request(req)の形式で呼び出されること, so that 解析済みテーブルを直接利用できる

**Note:** 本プロジェクトは未リリースのため、既存Luaスクリプトとの互換性は考慮しない。完全移行により実装をシンプルに保つ。

#### Acceptance Criteria
1. When SHIORI.requestを呼び出す, the PastaShiori shall 第一引数に解析済みテーブル`req`を渡す
2. When SHIORI.requestを呼び出す, the PastaShiori shall 生のリクエストテキストを引数として渡さない
3. The PastaShiori shall 既存のSHIORI.load/SHIORI.unloadの呼び出し方式を変更しない
4. The PastaShiori shall SHIORI.request関数が存在しない場合も204レスポンスを返す既存動作を維持する
5. If SHIORI.request実行中にエラーが発生する, then the PastaShiori shall エラーをログに記録し、MyErrorを返す
