# Requirements Document

## Project Description (Input)
lua側からロギング出力したい。lua側から、rustのlog出力へリダイレクトするためのモジュールを公開して欲しい。`@pasta_log`がいいな。trace/debugなど、基本的な出力と、可能なら呼び出し元コードの関数名位が取れればありがたいけど、luaのdebugモジュール有効にしないと無理かな？mluaはコールスタック参照できるなら関数名とか解決して欲しい。

## Introduction

本仕様は、Lua実行環境からRust側の`tracing`ロギングインフラへログ出力をブリッジする`@pasta_log`モジュールを定義する。既存の`@pasta_*`モジュール群（`@pasta_config`, `@pasta_persistence`, `@pasta_sakura_script`, `@pasta_search`）と同じ登録パターンに従い、`package.loaded`経由で`require "@pasta_log"`として利用可能にする。

## Requirements

### Requirement 1: ログレベル別出力関数の提供

**Objective:** Luaスクリプト開発者として、標準的なログレベルでメッセージを出力したい。Rust側の`tracing`ログレベルと一貫した粒度でデバッグ・診断が行えるようにするため。

#### Acceptance Criteria
1. The `@pasta_log` module shall `trace(message)`, `debug(message)`, `info(message)`, `warn(message)`, `error(message)` の5つのログレベル関数を提供する
2. When Luaスクリプトがログ関数を呼び出した場合, the `@pasta_log` module shall Rust側の`tracing`マクロ（`tracing::trace!`, `tracing::debug!`等）へメッセージを転送する
3. The `@pasta_log` module shall 各ログ関数の引数として文字列メッセージを受け付ける
4. If ログ関数にnil以外の非文字列値が渡された場合, the `@pasta_log` module shall `tostring()`で文字列に変換してから出力する
5. If ログ関数にnilまたは引数なしで呼び出された場合, the `@pasta_log` module shall 空文字列として処理し、エラーを発生させない

### Requirement 2: 呼び出し元情報の自動付与

**Objective:** Luaスクリプト開発者として、ログ出力時に呼び出し元のソースファイル名・行番号・関数名を自動的に付与したい。デバッグ時にログの発生箇所を迅速に特定するため。

#### Acceptance Criteria
1. When ログ関数が呼び出された場合, the `@pasta_log` module shall Lua側のコールスタックから呼び出し元のソースファイル名（`source`）と行番号（`currentline`）を取得してログに付与する
2. When ログ関数が呼び出された場合, the `@pasta_log` module shall 可能であれば呼び出し元の関数名（`name`）をログに付与する
3. The `@pasta_log` module shall 呼び出し元情報の取得にLua標準の`debug`ライブラリを必要としない（mlua APIの`Lua::inspect_stack()`を使用しRust側から取得する）
4. If 呼び出し元情報が取得できない場合, the `@pasta_log` module shall メッセージのみを出力し、エラーを発生させない

### Requirement 3: モジュール登録と利用方法

**Objective:** Luaスクリプト開発者として、既存の`@pasta_*`モジュールと同じパターンで`require "@pasta_log"`としてモジュールを利用したい。一貫したAPIスタイルを維持するため。

#### Acceptance Criteria
1. The `@pasta_log` module shall `package.loaded["@pasta_log"]`に登録され、`require "@pasta_log"`で取得可能である
2. The `@pasta_log` module shall ランタイム初期化シーケンス中に他の`@pasta_*`モジュールと同様に登録される
3. The `@pasta_log` module shall `_VERSION`および`_DESCRIPTION`メタデータフィールドを持つ
4. The `@pasta_log` module shall 外部設定（`pasta.toml`等）に依存せず、追加設定なしで動作する

### Requirement 4: Rust-side logging infrastructure への統合

**Objective:** ゴーストインスタンスとして、Luaからのログ出力をRust側の `tracing` インフラに統合し、既存のPastaLogger ルーティング機構を活用して、インスタンスごとのログファイルにLua側のログも含めるようにしたい。

#### Acceptance Criteria
1. The `@pasta_log` module shall Luaからのログ出力を `tracing` マクロ（`tracing::trace!`, `tracing::debug!` 等）経由でRust側のロギングインフラに転送する
2. When ログ関数が呼び出された場合, the `@pasta_log` module shall 呼び出し元情報（Lua側のsource, line, function名）を structured fields `lua_source`, `lua_line`, `lua_fn` として `tracing` イベントに埋め込む
3. The `@pasta_log` module shall トレーシング基盤の既存ルーティング機構（GlobalLoggerRegistry + RoutingWriter）を経由して、インスタンス固有のPastaLoggerに自動的に振り分けられる
4. Where PastaLoggerが設定されていない場合, the `@pasta_log` module shall `tracing` 出力のみが記録され、正常に動作する（ファイルロギング不要）

### Requirement 5: 安全性とサンドボックス整合性

**Objective:** システム管理者として、ログモジュールがLuaサンドボックスのセキュリティポリシーに違反しないことを保証したい。既存のセキュリティモデルを維持するため。

#### Acceptance Criteria
1. The `@pasta_log` module shall Lua標準の`debug`ライブラリの有効/無効に関わらず動作する
2. The `@pasta_log` module shall ファイルシステムへの直接アクセスを行わない（PastaLoggerへの委譲のみ）
3. The `@pasta_log` module shall Rust側のモジュール登録であり、`RuntimeConfig.libs`によるLuaライブラリ制御とは独立して常に利用可能である（`@pasta_persistence`等と同様）
