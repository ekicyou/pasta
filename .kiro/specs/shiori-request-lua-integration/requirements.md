# Requirements Document

## Project Description (Input)
「lua_request.rs」というファイルを他のプロジェクトからコピーした。このファイルを実際に使えるように組み込んでほしい。パーサーなどは存在する。<'lua>などとついているが、現在のmluaでは多分必要ない。現在の環境に合わせて組み込んでほしい。この関数群の目的は、SHIORI requestをパースし、Lua Tableに変換することである。

## Introduction
本仕様は、他プロジェクトから移植された`lua_request.rs`を、pasta_shioriクレートの現行mlua環境に統合することを目的とする。既存の`util::parsers::req`パーサーを活用し、SHIORI requestテキストをLuaテーブルに変換する機能を提供する。

## Requirements

### Requirement 1: SHIORI Request Luaテーブル変換
**Objective:** 開発者として、SHIORI requestテキストをLuaテーブル形式で取得したい。これにより、Luaスクリプト内でSHIORI requestの各フィールドに簡単にアクセスできる。

#### Acceptance Criteria
1. When SHIORI requestテキストが渡されたとき, the lua_request module shall 既存の`util::parsers::req::ShioriRequest`パーサーを使用してパースを行う
2. When パースが成功したとき, the lua_request module shall method, version, id, sender, security_level, charset, status, base_idの各フィールドをLuaテーブルに設定する
3. When パースが成功したとき, the lua_request module shall reference配列（reference0～n）をLuaテーブルの配列フィールドとして設定する
4. When パースが成功したとき, the lua_request module shall 全てのキー・バリューをdicサブテーブルに格納する
5. If パースエラーが発生したとき, then the lua_request module shall MyErrorを返却する

### Requirement 2: 現在時刻Luaテーブル生成
**Objective:** 開発者として、現在時刻情報をLuaテーブル形式で取得したい。これにより、Luaスクリプト内で日時情報を扱える。

#### Acceptance Criteria
1. When lua_date関数が呼び出されたとき, the lua_request module shall year, month, day, hour, min, sec, nsフィールドを含むテーブルを返却する
2. When lua_date関数が呼び出されたとき, the lua_request module shall yday（年間通算日）, wday（曜日）フィールドを含むテーブルを返却する
3. The lua_request module shall `time` crate (v0.3.x) with `local-offset` featureを使用してローカルタイムを取得する
4. The lua_request module shall `OffsetDateTime::now_local()`の`Result`型を`From<time::error::IndeterminateOffset>` trait実装で自動変換し、失敗時はMyError::Scriptを返却する

**Rationale**: `time` crateを`chrono`より選択した理由:
- 活発なメンテナンス: 520M DL, 125バージョン, 7日前更新
- セキュリティ重視設計: ローカルオフセット取得を`Result`で安全に処理
- モダンAPI: Rust 2021+ベストプラクティス準拠
- 軽量: 17K SLoC (chrono 20K SLoC)

### Requirement 3: mlua互換性更新
**Objective:** 開発者として、現行のmlua環境で動作するコードが欲しい。これにより、古いライフタイム記法を排除し、保守性を向上させる。

#### Acceptance Criteria
1. The lua_request module shall `<'lua>`ジェネリックライフタイムを削除し、現行mluaの参照借用モデルに準拠する
   - 関数シグネチャから`<'lua>`を除外: `fn lua_date(lua: &Lua) -> MyResult<Table>`
   - `Table<'lua>` → `Table`に変更（mlua 0.11+ではライフタイム不要）
2. The lua_request module shall `pasta_lua::mlua`から必要な型（Lua, Table等）を再エクスポートして使用する
3. The lua_request module shall 外部クレート`shiori3::req`への依存を削除し、`crate::util::parsers::req`を使用する
4. The lua_request module shall `crate::prelude::*`への依存を削除し、明示的なインポートに置き換える

**Rationale** (議題2決定): 現行mlua (0.11+) では明示的なライフタイム記法が不要になりました。API DOCにも記載がなく、冗長性を排除することでコードの保守性が向上します。

### Requirement 4: エラー型統合
**Objective:** 開発者として、統一されたエラー型で一貫したエラーハンドリングを行いたい。これにより、コードの保守性と可読性が向上する。

#### Acceptance Criteria
1. The lua_request module shall `crate::error::MyResult<T>`を戻り値型として使用する
2. When mlua::Errorが発生したとき, the lua_request module shall MyErrorに変換して返却する
3. When パースエラーが発生したとき, the lua_request module shall 既存のMyError::ParseRequest変換を使用する

### Requirement 5: モジュール公開
**Objective:** 開発者として、lua_requestモジュールをpasta_shioriから利用可能にしたい。これにより、SHIORI実装内でLuaテーブル変換機能を使用できる。

#### Acceptance Criteria
1. The pasta_shiori lib.rs shall lua_requestモジュールを宣言し、必要に応じて公開する
2. When shiori.rs等から呼び出されたとき, the lua_request module shall SHIORI requestをLuaテーブルに変換する機能を提供する
