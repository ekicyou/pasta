# Implementation Plan

## 実装タスク一覧

- [x] 1. クレートの Cargo.toml とソースファイル骨格を作成する
  - `crates/pasta_check/` ディレクトリを作成し、ワークスペースの `crates/*` パターンに自動包含されることを確認する
  - `lexopt`・`md5`・`encoding_rs`・`zip`（deflate only）・`thiserror`・`pasta_lua` を依存に追加した Cargo.toml を作成する
  - `publish = true`、`description`、ワークスペース共通メタデータ（`version`・`edition`・`authors`・`license`・`repository` 等）を設定する
  - `src/main.rs`（エントリポイント）・`src/release.rs`・`src/copy.rs`・`src/update_files.rs`・`src/nar.rs` の空スタブを配置する
  - `cargo build -p pasta_check` が通ることを確認する
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 9.1, 9.2, 9.3_

- [x] 2. (P) release サブコマンドと全オプションを解析する CLI を実装する
  - `lexopt` を使って `release` サブコマンドを認識し、`Args`・`Command`・`ReleaseArgs` 構造体を定義する
  - `--target`・`--release`・`--nar` を必須オプションとして解析し、未指定時はエラーメッセージと使用方法を標準エラー出力に表示して非ゼロ終了する
  - `--copy` を複数回指定可能なオプションとして受け取り、指定された順序を保持して `Vec<PathBuf>` に蓄積する
  - `--help` および `--version` に対応する
  - `parse_args()` の単体テストで必須オプション未指定・複数 `--copy` を検証する
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

- [x] 3. (P) ディレクトリの再帰コピーとリリースフォルダー初期化を実装する
  - リリースフォルダーが存在すれば削除し、空ディレクトリとして再作成する機能を実装する
  - ソースディレクトリの全ファイルとサブディレクトリを宛先に再帰コピーし、コピーしたファイル数を返す機能を実装する
  - 上書きコピーモードで既存ファイルは上書き・新規ファイルは追加し、ディレクトリ構造を維持する
  - ソースディレクトリの内容が変更されないことを担保する（読み取り専用操作）
  - _Requirements: 3.1, 3.2, 3.3, 3.8_

- [x] 4. (P) SSP 更新ファイル生成機能を移植・実装する
  - `pasta_sample_ghost/src/update_files.rs` の `FileEntry` 構造体・`collect_files()`・`generate_updates2_dau()`・`generate_updates_txt()`・`calculate_md5()` を `pasta_check/src/update_files.rs` に移植する
  - `updates2.dau` を Shift_JIS エンコーディング・CRLF 改行・SOH 区切りフォーマットで生成する
  - `updates.txt` を Shift_JIS エンコーディング・CRLF 改行・カンマ区切りフォーマットで生成する
  - `profile/`・`var/`・`updates2.dau`・`updates.txt`・`developer_options.txt` を除外し、ファイルエントリをパスのアルファベット順でソートする
  - Shift_JIS に変換できない文字は UTF-8 のままフォールバックする
  - 既存テスト 3 件を `pasta_check` 側に移植し、生成出力が変わらないことを検証する
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

- [x] 5. (P) ZIP 形式の NAR アーカイブ作成機能を実装する
  - `zip` クレートの `ZipWriter` を使ってリリースフォルダーの全ファイルを deflate 圧縮し `.nar` ファイルとして出力する
  - ZIP 内のファイルパスはリリースフォルダーからの相対パス（スラッシュ区切り）で格納する
  - `profile/` ディレクトリ配下のファイルを NAR から除外する
  - NAR 出力先の親ディレクトリが存在しない場合は再帰的に作成する
  - 既存の NAR ファイルが存在する場合は上書きし、完了時に NAR ファイルサイズ（バイト）を返す
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 6. release サブコマンドのパイプライン実行ロジックを完成させる
  - `execute_release()` を実装し、フォルダー初期化 → target コピー → --copy 上書き → 更新ファイル生成 → NAR 作成の 5 ステップをシーケンシャルに実行する
  - 各ステップで `[1/5] Preparing release folder...` 形式の進捗メッセージを標準出力に表示する
  - `--copy` オプションが指定された場合、指定順序で各フォルダーの内容を上書きコピーする
  - IO エラーは `?` で早期伝播し、`main()` でエラー内容を標準エラー出力に表示して `process::exit(1)` で終了する
  - `--target` フォルダーの内容が変更されていないことを統合テストで検証する
  - `cargo test -p pasta_check` で全テストが通ることを確認する
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8_

- [x] 7. pasta_sample_ghost からリリース処理を分離する
- [x] 7.1. 更新ファイル生成モジュールと --finalize オプションを削除する
  - `pasta_sample_ghost/src/update_files.rs` モジュールを削除する
  - `pasta_sample_ghost/src/lib.rs` から `pub mod update_files` 宣言と `finalize_ghost()` 関数を削除する
  - `pasta_sample_ghost/src/main.rs` から `--finalize` オプション分岐と `run_finalize_mode()` 関数を削除する
  - `pasta_sample_ghost/Cargo.toml` から `md5` と `encoding_rs` への依存を削除する
  - `cargo test -p pasta_sample_ghost` で画像生成系テスト 10 件が正常に通ることを確認する
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [x] 7.2. (P) dist-src を廃止し辞書ファイルをゴースト開発フォルダーへ統合する
  - `crates/pasta_sample_ghost/dist-src/ghost/` の内容を `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/` に移動する
  - `crates/pasta_sample_ghost/dist-src/shell/` の内容を `crates/pasta_sample_ghost/ghosts/hello-pasta/shell/` に移動する
  - `dist-src/install.txt` を `crates/pasta_sample_ghost/ghosts/hello-pasta/install.txt` に移動する
  - `dist-src/` ディレクトリを削除する
  - `crates/pasta_sample_ghost/hello-pasta.nar` を削除する
  - _Requirements: 6.5, 7.5_

- [x] 8. release.ps1 を簡素化し release.bat をルートに移動する
- [x] 8.1. release.ps1 を 6 ステップ構成と pasta_check 呼び出しに改訂する
  - 旧 Step 2（dist-src robocopy）を削除する
  - DLL/scripts の `ghosts/hello-pasta/` へのコピーを Step 3 に繰り上げる
  - 旧 Step 5（finalize）・Step 7（バリデーション）・Step 8（NAR 作成）を削除し、Step 4 として `pasta_check release --target $GhostDir --release $ReleaseDir --nar $NarFilePath` 呼び出しを追加する
  - `$ReleaseDir`（例: `release/hello-pasta`）と `$NarFilePath`（例: `release/hello-pasta.nar`）変数をスクリプト冒頭に追加する
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [x] 8.2. (P) release.bat をリポジトリルートに移動する
  - リポジトリルートに新しい `release.bat` を作成し、`%~dp0crates\pasta_sample_ghost\release.ps1` を呼び出すように設定する
  - `-SkipSetup`・`-SkipDllBuild` オプションを `release.ps1` にパススルーする
  - 既存の `crates/pasta_sample_ghost/release.bat` を削除する
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [x] 9. release-workflow 仕様に pasta_check のパブリッシュ手順を追加する
  - `release-workflow` 仕様の `cargo publish` 対象クレートリストに `pasta_check` を追加する（`pasta_lua` の後、依存順序を維持）
  - _Requirements: 9.4_
