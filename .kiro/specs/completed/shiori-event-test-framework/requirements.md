# Requirements Document

## Introduction

SHIORIイベントフロー試験を簡潔に記述するためのテスト基盤を提供する。テスト作成者が（1）時刻依存イベントを決定論的に検証でき、（2）Rustバックエンドモジュールのモック設定を一括で行え、（3）SHIORIレスポンスを構造的に検証できるようにする。

**対象ユーザー**: pasta エンジン開発者（Rust/Luaテスト作成者）

**現状の課題**:
- `pasta_lua` のテストでは `@pasta_persistence`, `@pasta_search`, `@pasta_sakura_script` 等のRustバックエンドモジュールを各テストファイルで手動 `package.loaded` モック設定しており、ボイラープレートが多い
- `pasta_shiori` のテストでは `parse_request()` が常に `now_local()` を呼ぶため時刻を固定できず、OnHour・OnTalk 等の時刻依存イベントのテストが決定論的に書けない
- SHIORIレスポンスの検証が生文字列マッチ（`response.contains("200 OK")`）のみで、ステータスコードやValueフィールドを個別に検証できない
- 上記の結果、`shiori-async-talk` で必要なマルチステップSHIORI往復テストを書く基盤がない

## Boundary Context

- **In scope**: SHIORIリクエストへの時刻注入ヘッダー対応、Luaモック一括注入ライブラリ、SHIORIレスポンスの構造化検証、SHIORIテスト環境の統合セットアップ、各機能の動作検証テスト
- **Out of scope**: 既存テストの新基盤への全面移行、コルーチンステップ制御API（`shiori-async-talk` 側で構築）、`pasta_check` への `test` サブコマンド追加、テスト実行パフォーマンス最適化
- **Adjacent expectations**: Luaモックライブラリは SHIORIプロトコルに依存せず `pasta_lua` 内で独立動作する。`shiori-async-talk` 仕様がマルチステップSHIORI往復テストの基盤として本フレームワークを利用する。

## Requirements

### Requirement 1: SHIORIリクエスト時刻制御

**Objective:** As a テスト作成者, I want SHIORIリクエスト内のカスタムヘッダーで時刻を指定する, so that 時刻依存イベント（OnHour、OnTalk等）のテストを決定論的に実行できる

#### Acceptance Criteria

1. When SHIORIリクエストに `X-Pasta-Time` ヘッダーが含まれ、その値が有効なRFC 3339形式の日時文字列である場合, the リクエストパーサー shall そのヘッダーの日時を `req.date` テーブルに採用し、システムクロックを参照しない
2. When SHIORIリクエストに `X-Pasta-Time` ヘッダーが含まれない場合, the リクエストパーサー shall 従来通りシステムクロックから `req.date` を生成する
3. If `X-Pasta-Time` ヘッダーの値が不正な形式である場合, the リクエストパーサー shall `tracing::error!` で不正値の詳細をログ出力し、`MyError` を返却してリクエスト処理を中断する（SHIORIレスポンスの `X-ERROR-REASON` ヘッダーに不正値の詳細が含まれる）
4. When `X-Pasta-Time` にタイムゾーンオフセット付きの日時が指定された場合, the リクエストパーサー shall そのオフセットを反映した正しい日付・時刻フィールド（year, month, day, hour, min, sec, wday, yday 等）を `req.date` テーブルに設定する
5. The `X-Pasta-Time` ヘッダー shall 既存のSHIORIプロトコルPEG文法の変更なしに、`key_other` として自然にパースされる

### Requirement 2: Luaモック一括注入ライブラリ

**Objective:** As a Luaテスト作成者, I want Rustバックエンドモジュールのモック設定を一括で行う, so that テスト毎の `package.loaded` ボイラープレートを排除しイベントテストに集中できる

#### Acceptance Criteria

1. The モックライブラリ shall `@pasta_persistence`, `@pasta_search`, `@pasta_sakura_script`, `@pasta_config`, `@pasta_log` の全Rustバックエンドモジュールに対するデフォルトスタブを提供する
2. When テスト作成者がモックライブラリの一括インストール関数を呼び出した場合, the モックライブラリ shall 全対象モジュールのデフォルトスタブを `package.loaded` に登録する
3. When テスト作成者が特定モジュールのカスタムスタブを指定した場合, the モックライブラリ shall デフォルトスタブの代わりに指定されたスタブを登録する
4. When テスト作成者がモックをリセットした場合, the モックライブラリ shall 対象モジュールの `package.loaded` エントリを初期状態（nil）に戻す
5. The モックライブラリ shall SHIORIプロトコルや `time` クレートに依存せず、`pasta_lua` クレート内で独立して動作する
6. The デフォルトスタブ shall 各モジュールの公開インターフェースと互換性のある最小実装を提供する（`@pasta_persistence` の `load`/`save`、`@pasta_search` の `search_scene`/`search_word`、`@pasta_sakura_script` の `talk_to_script`/`break_lines`、`@pasta_config` の設定テーブル構造、`@pasta_log` の `trace`/`debug`/`info`/`warn`/`error` 関数群）

### Requirement 3: SHIORIレスポンス構造化検証

**Objective:** As a SHIORIテスト作成者, I want SHIORIレスポンス文字列を構造化されたフィールドに分解する, so that ステータスコード・Valueフィールド・ヘッダーを個別に検証できる

#### Acceptance Criteria

1. When 有効なSHIORI/3.0レスポンス文字列が与えられた場合, the レスポンスパーサー shall ステータスコード（整数）、ステータステキスト（文字列）、全ヘッダー（キー・値ペアの集合）、Valueフィールド（文字列）を個別に取得可能な形式に分解する
2. When Valueフィールドを含まないレスポンス（204 No Content等）が与えられた場合, the レスポンスパーサー shall Valueフィールドを空として返す
3. If 不正な形式のレスポンス文字列が与えられた場合, the レスポンスパーサー shall パニックせずに明示的なエラーを返す
4. When レスポンスに複数のカスタムヘッダー（X-Error-Reason等）が含まれる場合, the レスポンスパーサー shall 全ヘッダーを保持し、ヘッダー名による個別取得を可能にする

### Requirement 4: SHIORIテスト環境セットアップ

**Objective:** As a SHIORIテスト作成者, I want フィクスチャの準備・SHIORIロード・リクエスト投入を一体化したテスト環境を使う, so that テストのセットアップを最小限の記述で完了しテストロジックに集中できる

#### Acceptance Criteria

1. When テスト作成者がフィクスチャ名を指定してテスト環境を作成した場合, the テスト環境 shall フィクスチャディレクトリとサポートファイルを一時ディレクトリにコピーし、SHIORIの load を完了した状態を返す
2. When テスト環境が破棄された場合, the テスト環境 shall 一時ディレクトリを自動的にクリーンアップする
3. When テスト作成者がテスト環境に対してSHIORIリクエスト文字列を投入した場合, the テスト環境 shall リクエスト処理結果を構造化レスポンス（Requirement 3 で定義）として返す
4. When テスト作成者が同一テスト環境に複数のSHIORIリクエストを順次投入した場合, the テスト環境 shall 各リクエスト間でLuaランタイムの状態（グローバル変数、コルーチン等）を維持する
5. The テスト環境 shall 内部のLuaランタイムへの直接アクセス手段を提供し、テスト作成者がLuaグローバル変数や内部状態を検査できるようにする

### Requirement 5: 後方互換性

**Objective:** As a 既存テストの維持者, I want テストフレームワーク導入が既存テストに影響しない, so that 安全にフレームワークを追加導入できる

#### Acceptance Criteria

1. The リクエストパーサー shall `X-Pasta-Time` ヘッダーを含まない既存のSHIORIリクエスト処理について、従来と同一の動作を維持する
2. The テストフレームワーク導入後 shall 既存の全テスト（Rustテスト・Luaテスト）が変更なしで成功する
