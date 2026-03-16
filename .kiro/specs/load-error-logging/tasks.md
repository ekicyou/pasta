# Implementation Plan

- [ ] 1. (P) ログファイル名を pasta.log に固定する
  - `RollingFileAppender` の rotation を `Rotation::NEVER` に変更する
  - `max_log_files()` 呼び出しを削除する（`Rotation::NEVER` ではローテーション不要）
  - タスク 2・3 と並列実行可能（変更対象ファイルが独立している）
  - _Requirements: 3.1, 3.2_

- [ ] 2. (P) tracing_init モジュールを作成する
- [ ] 2.1 reload::Layer で subscriber を初期化する API を実装する
  - `tracing-subscriber` に `"reload"` feature を追加する（ワークスペース Cargo.toml 変更）
  - 新規ファイル `crates/pasta_lua/src/logging/tracing_init.rs` を作成する
  - `OnceLock<FilterHandle>` を定義して handle を保持する機構を実装する
  - `init_tracing_with_reload(config)` を実装する — `reload::Layer::new()` で subscriber を構築し handle を OnceLock に保存する
  - タスク 1・3 と並列実行可能（新規ファイル作成のため競合なし）
  - _Requirements: 2.1, 2.3_

- [ ] 2.2 フィルター動的更新 API を実装してエクスポートする
  - `update_tracing_filter(config)` を実装する — OnceLock から handle を取得し EnvFilter を差し替える
  - `logging/mod.rs` と `lib.rs` に `pub use` エクスポートを追加する
  - 2.1 の完了が前提（`FilterHandle` 型と `FILTER_HANDLE` が必要）
  - _Requirements: 2.1, 2.3_

- [ ] 3. (P) トランスパイル失敗でロードを中止する
- [ ] 3.1 (P) PartialTranspileError の表示にファイルパスを含める
  - `format_failure_paths()` ヘルパー関数を `loader/error.rs` に追加する
  - `LoaderError::PartialTranspileError` の `#[error(...)]` を変更しヘルパー経由でファイルパスを含める
  - 結果例: `トランスパイル部分失敗: 3件成功, 1件失敗 [dic/talk.pasta]`
  - タスク 1・2 と並列実行可能; タスク 3.2・3.3 とも並列実行可能（変更ファイルが独立）
  - _Requirements: 1.4_

- [ ] 3.2 .lua ファイル処理失敗を failures に収集する
  - `process_incremental()` の .lua 読み込み失敗ブランチに `failures.push(TranspileFailure {...})` を追加する
  - .lua キャッシュ書き込み失敗ブランチにも同様に `failures.push()` を追加する
  - 既存の `warn!(...)` を `error!(...)` に昇格させる
  - _Requirements: 4.1, 4.2_

- [ ] 3.3 failures 非空時に Err を返してロードを中止する
  - `process_incremental()` の失敗レポートブロックを `Err(LoaderError::PartialTranspileError { ... })` を返す実装に変更する
  - 全失敗ファイルを `error!()` でログに記録してから `Err` を返す
  - 3.2 の完了が前提（同関数内の変更のため）
  - _Requirements: 4.1, 4.2, 4.3_

- [ ] 4. PastaLoader に Stage 1.5 を組み込む
  - `create_and_register_logger()` メソッドを実装する — `PastaLogger::new()` + `GlobalLoggerRegistry::register()` + `update_tracing_filter()` を一括実行する
  - `load_with_config()` の Phase 1 完了直後に Stage 1.5 として呼び出しを挿入する
  - 既存の `create_logger()` を `create_and_register_logger()` に置き換え、Phase 6 の記述を削除する
  - タスク 2 と タスク 3 の完了が前提（`update_tracing_filter` が必要; `loader/mod.rs` の競合回避）
  - _Requirements: 2.1_

- [ ] 5. PastaShiori にエラー保持と早期初期化を実装する
- [ ] 5.1 load エラーを保持して request() に伝搬する
  - `PastaShiori` 構造体に `last_load_error: Option<String>` フィールドを追加する
  - `load()` の `Err(e)` ブランチで `last_load_error = Some(format!("{}", e))` を設定する
  - reload 時に `last_load_error = None` でリセットする処理を追加する
  - `request()` の runtime チェックを `ok_or_else(|| ...)` に変更し `last_load_error` に応じて `MyError::Load` または `MyError::NotInitialized` を返す
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [ ] 5.2 load() 先頭に Stage 1 早期ロガー初期化を実装する
  - `PastaLoader::load()` 呼び出し前に `PastaLogger::new()` + `GlobalLoggerRegistry::register()` + `init_tracing_with_reload()` を追加する
  - 既存の `init_tracing_with_config()` 関数定義を削除する（`pasta_lua::init_tracing_with_reload` に置き換え済みのため）
  - load 成功後の `runtime.logger()` → `GlobalLoggerRegistry::register()` 呼び出しブロックを削除する（Stage 1.5 で実施済みのため）
  - タスク 2 と タスク 4 の完了が前提（`init_tracing_with_reload` が必要）
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] 6. テストを更新する
- [ ] 6.1 (P) PastaLogger のテストを Rotation::NEVER に対応させる
  - `Rotation::DAILY` → `Rotation::NEVER` 変更による既存テストの失敗を修正する
  - ログファイル名が日付サフィックスなし（`pasta.log`）で生成されることを確認する
  - タスク 6.2 と並列実行可能（テスト対象モジュールが異なる）
  - _Requirements: 3.1, 3.2_

- [ ] 6.2 (P) process_incremental のエラー伝搬テストを更新する
  - 部分失敗時に `Ok` ではなく `Err(PartialTranspileError)` が返ることをアサートするテストを追加する
  - `.lua` ファイル失敗ケースが `failures` に収集されることを確認するテストを追加する
  - `PartialTranspileError` の Display にファイルパスが含まれることを確認する
  - タスク 6.1 と並列実行可能（テスト対象クレート・ファイルが異なる）
  - _Requirements: 1.4, 4.1, 4.2, 4.3_

- [ ] 6.3 PastaShiori の load 失敗 → request エラー伝搬を確認する
  - `last_load_error` が設定された状態で `request()` が `MyError::Load(msg)` を返すことをアサートする
  - `runtime = None` かつ `last_load_error = None` の場合は `MyError::NotInitialized` が返ることを確認する
  - タスク 5 の完了が前提
  - _Requirements: 1.1, 1.2, 2.1, 2.2_

- [ ] 7. ドキュメント整合性の確認と更新
  - SOUL.md — コアバリュー・設計原則との整合性確認（本機能はエラーハンドリング改良のため影響軽微）
  - doc/spec/ — 言語仕様の更新（該当なし: エラーハンドリング変更のみ）
  - GRAMMAR.md — 文法リファレンスの同期（該当なし）
  - TEST_COVERAGE.md — 新規テスト（6.1, 6.2, 6.3）のマッピング追加
  - クレート README（pasta_lua, pasta_shiori）— `init_tracing_with_reload` / `update_tracing_filter` の API 変更を反映
  - steering/tech.md — tracing-subscriber `"reload"` feature 追加を記録（必要に応じて）
