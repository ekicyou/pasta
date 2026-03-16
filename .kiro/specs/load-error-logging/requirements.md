# Requirements Document

## Project Description (Input)
トランスパイルでエラーが出ているようだが、その記録がログに残らない。SHIORI通信ログでは `X-ERROR-REASON: not initialized error` が返されるが、loadで発生したエラーのメッセージがキャプチャーされず原因がわからない。また、logsフォルダのログも何も出力されていない。エラーハンドリングについて改良し、エラーが発生した時の記録が確実にSHIORI側に出力され続けるようにし、さらにログ出力を確実に行う。また、ログのファイル名が「pasta.log.2026-03-16」などとなっているが、ローリングしないなら単純にpasta.logでよい。

## Introduction

SHIORI `load` フェーズでトランスパイルエラー等が発生した際、エラー情報がSHIORI応答にもログファイルにも記録されず、デバッグが困難な状況がある。本要件は pasta_shiori / pasta_lua のエラーハンドリングとログ出力を改良し、起動失敗時の原因究明を確実にする。

## 現状の問題分析

1. **ロガー初期化タイミングの問題**: `PastaLoader::load()` の成功後にのみ tracing subscriber が初期化される。load 失敗時はロガーが未初期化のため、`error!()` マクロの出力がファイルに記録されない。
2. **load エラー情報のロスト**: `PastaShiori::load()` が `Ok(false)` を返すだけで、実際のエラーメッセージが保存されない。後続の `request()` は `MyError::NotInitialized` を返すが、load 失敗の根本原因（例: トランスパイルエラー）の情報が含まれない。
3. **ログファイル名**: `RollingFileAppender` が `Rotation::DAILY` + `filename_prefix` で `pasta.log.2026-03-16` 形式のファイル名を生成するが、実運用でローリングが不要であれば単純な `pasta.log` が望ましい。

## Requirements

### Requirement 1: load エラーメッセージの SHIORI 応答への伝搬

**Objective:** ゴースト開発者として、load 失敗時の原因をSSPの SHIORI 通信ログで確認したい。これにより、トランスパイルエラー等の問題を即座に特定できる。

#### Acceptance Criteria
1. When `PastaLoader::load()` がエラーを返した場合, pasta_shiori shall エラーメッセージを内部に保持する
2. While `PastaShiori` が load 失敗状態にあるとき, pasta_shiori shall 後続の `request()` の `X-ERROR-REASON` ヘッダにload 時のエラーメッセージを含めた 500 応答を返す
3. The pasta_shiori shall `X-ERROR-REASON` に load 失敗の根本原因（例: `トランスパイル部分失敗: 3件成功, 1件失敗`）を含める
4. When トランスパイルエラーの場合, pasta_shiori shall 失敗したファイル名をエラーメッセージに含める
5. The pasta_shiori shall エラーメッセージを日本語で出力する（既存の `LoaderError` の日本語メッセージを活用する）

### Requirement 2: load 失敗時のログファイル出力保証

**Objective:** ゴースト開発者として、load の成否に関わらずログファイルにエラー情報が記録されていてほしい。起動失敗後にログファイルを確認して原因を調査できるようにする。

#### Acceptance Criteria
1. When `PastaShiori::load()` が開始された直後, pasta_shiori shall ログ出力の初期化を `PastaLoader::load()` の呼び出しより前に行う
2. If `PastaLoader::load()` が失敗した場合, pasta_shiori shall エラーの詳細をログファイルに記録する
3. When load が成功した場合, pasta_shiori shall ログの初期化を二重に行わない（`try_init()` の既存動作を維持する）

### Requirement 3: ログファイル名の簡素化

**Objective:** ゴースト開発者として、ログファイルの場所を直感的に見つけたい。日付サフィックスなしの単純なファイル名とする。

#### Acceptance Criteria
1. The pasta_lua shall ログファイルを `pasta.log` という固定ファイル名で出力する（日付サフィックスなし）
2. When `PastaLogger` が初期化される場合, pasta_lua shall `Rotation::NEVER` を使用し、ファイル名にローリングサフィックスを付与しない

> **Note**: デフォルトパス `profile/pasta/logs/pasta.log` は既存の `default_log_file_path()` で既に設定済みのため、変更不要。

### Requirement 4: トランスパイルエラーの詳細記録

**Objective:** ゴースト開発者として、どの `.pasta` ファイルのトランスパイルが失敗したかをログから特定したい。

#### Acceptance Criteria
1. When トランスパイル処理で個別ファイルのエラーが発生した場合, pasta_lua shall 失敗したファイルパスとエラーメッセージをログに記録する
2. When `PartialTranspileError` が発生した場合, pasta_lua shall 全失敗ファイルの一覧をログに記録する
3. If トランスパイルエラーが発生した場合, pasta_lua shall 失敗情報を呼び出し元に返す（`LoaderError` として伝搬する）

> **設計判断**: 部分失敗時にゴースト起動を継続するか中止するかは設計フェーズで決定する。
