# Implementation Plan

## Task Overview

テキスト系配布ファイル（設定4種＋DSLスクリプト4種）を Rust コード生成から dist-src/ ディレクトリ配置＋robocopy コピー方式に移行する。release.ps1 に robocopy ステップを追加し、実行順序（テキストコピー → 画像生成 → DLL/scripts コピー → finalize）を確立する。

## Tasks

### 1. dist-src ディレクトリ構造の作成

- [x] 1.1 (P) dist-src/ ディレクトリと8ファイルの作成
  - `crates/pasta_sample_ghost/dist-src/` ディレクトリを作成
  - 配布先構造をミラー: `install.txt`, `ghost/master/descript.txt`, `ghost/master/pasta.toml`, `ghost/master/dic/*.pasta` (4ファイル), `shell/master/descript.txt`
  - 各ファイルに GhostConfig::default() の固定値を埋め込んだ最終形を配置
  - 既存の `templates/` ディレクトリと `crates/pasta_sample_ghost/templates/*.template` の4ファイルを削除
  - 移行正確性検証: 既存の `cargo run` 出力と `dist-src/` ファイルの完全一致を確認するテストを追加（移行前のスナップショット比較）
  - _Requirements: 1.1, 1.2, 1.3, 3.1, 3.2, 3.3, 3.4_

### 2. release.ps1 の robocopy ステップ追加

- [x] 2.1 (P) release.ps1 に dist-src → ghosts/hello-pasta の robocopy ステップを追加
  - 新 Step 2 を DLL ビルド直後、画像生成前に配置
  - robocopy コマンド実装: `/E` フラグ使用、サイレントモード (`/NJH`, `/NJS`, `/NDL`, `/NC`, `/NS`, `/NP`) 適用
  - `dist-src/` ディレクトリ不在時のエラーチェックを `Test-Path` で実装、エラーメッセージ表示後 `exit 1`
  - robocopy 終了コード 8 以上を失敗として検出し、エラーメッセージ＋`exit 1` で処理中断
  - Setup Phase のステップ番号を再採番（1: DLL → 2: テキスト → 3: 画像 → 4: DLL/scripts → 5: finalize → 6-9: Release Phase）
  - _Requirements: 2.1, 4.1, 4.2, 4.3, 4.4, 4.5_

### 3. config_templates.rs のテンプレートコード削除

- [x] 3.1 (P) config_templates.rs からテンプレート関連コードを削除
  - `INSTALL_TXT_TEMPLATE`, `GHOST_DESCRIPT_TEMPLATE`, `SHELL_DESCRIPT_TEMPLATE`, `PASTA_TOML_TEMPLATE` 定数を削除
  - `generate_structure()`, `generate_install_txt()`, `generate_ghost_descript()`, `generate_shell_descript()`, `generate_pasta_toml()` 関数を削除
  - `use crate::{GhostConfig, GhostError};` から `GhostConfig` の import を削除
  - `use std::fs;`, `use std::path::Path;` の不要な import を削除
  - `test_install_txt`, `test_ghost_descript`, `test_pasta_toml` テストを削除
  - `test_surfaces_txt` テストを残し、モジュール doc comment を「surfaces.txt 生成」に更新
  - _Requirements: 6.2, 3.1, 3.2, 3.3, 3.4_

### 4. scripts.rs のハードコード定数削除

- [x] 4.1 (P) scripts.rs のハードコード定数と関数を削除
  - `ACTORS_PASTA`, `BOOT_PASTA`, `TALK_PASTA`, `CLICK_PASTA` 定数を削除
  - `generate_scripts()` 関数を削除
  - テストコードに `dist_src_dir()` ヘルパー（`env!("CARGO_MANIFEST_DIR")` ベース）を追加
  - テストコードに `read_pasta_script(name: &str) -> String` ヘルパーを追加（dist-src ファイル読み込み）
  - 6テストを機械的置換: 定数参照 → `read_pasta_script()` 呼び出し（`test_actors_pasta_contains_all_characters`, `test_boot_pasta_contains_events`, `test_talk_pasta_contains_events`, `test_click_pasta_contains_events`, `test_script_expression_names_defined_in_actors`, `test_event_files_do_not_contain_global_actor_dictionary`）
  - _Requirements: 2.2, 5.2_

### 5. lib.rs / generate_ghost() の責務縮小

- [x] 5.1 (P) lib.rs の generate_ghost() を画像＋surfaces.txt のみに縮小
  - `config_templates::generate_structure()` 呼び出しを削除
  - `scripts::generate_scripts()` 呼び出しを削除
  - `shell/master/` ディレクトリ作成は画像生成前に必要なため残す
  - `generate_surfaces_txt()` 呼び出しを維持
  - `GhostConfig` 引数を `_config` にリネーム（未使用だが API 互換性のため保持）
  - `ghost/master/dic/` ディレクトリ作成コードを削除（robocopy で作成済み）
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

### 6. main.rs のコンソール出力更新

- [x] 6.1 (P) main.rs のメッセージを画像生成のみに更新
  - `run_generate_mode()` の出力メッセージを「画像ファイル（surface*.png）および surfaces.txt のみ生成」に更新
  - doc comment の「生成されるファイル」セクションからテキストファイルの記述を削除
  - _Requirements: 6.3_

### 7. integration_test.rs の更新

- [x] 7.1 新規テスト dist_src_validation_test.rs の作成
  - `crates/pasta_sample_ghost/tests/dist_src_validation_test.rs` を作成
  - `test_dist_src_directory_structure()` テストを実装: 8ファイルの存在確認（`fs::metadata` による静的検証、`generate_ghost()` 呼び出し不要）
  - dist-src ファイルリスト: `install.txt`, `ghost/master/descript.txt`, `ghost/master/pasta.toml`, `ghost/master/dic/actors.pasta`, `ghost/master/dic/boot.pasta`, `ghost/master/dic/talk.pasta`, `ghost/master/dic/click.pasta`, `shell/master/descript.txt`
  - _Requirements: 5.1, 5.3_

- [x] 7.2 integration_test.rs のテキスト系テストを dist-src 読み込みに変更
  - `test_pasta_toml_content`: `dist-src/ghost/master/pasta.toml` を `fs::read_to_string` で直接読み込み、検証ロジックは維持
  - `test_ukadoc_files`: `dist-src/ghost/master/descript.txt`, `dist-src/shell/master/descript.txt`, `dist-src/install.txt` を直接読み込み
  - `test_pasta_scripts`: `dist-src/ghost/master/dic/*.pasta` を直接読み込み
  - `test_random_talk_patterns`: ファイル読み込みに変更（`scripts::TALK_PASTA` → `fs::read_to_string("dist-src/ghost/master/dic/talk.pasta")`）
  - `test_hour_chime_patterns`: 同上
  - _Requirements: 5.1, 5.3_

- [x] 7.3 integration_test.rs の画像系テストを分離
  - `test_directory_structure` を削除（タスク 7.1 で dist-src 検証に移行済み）
  - `test_generated_images_structure()` テストを新規作成: `generate_ghost()` 経由で `shell/master/*.png` 18ファイル＋`surfaces.txt` 生成確認（既存の `test_directory_structure` の画像チェック部分を流用）
  - `test_shell_images`, `test_image_dimensions`, `test_expression_variations`: 変更なし（画像生成テストとして継続）
  - _Requirements: 5.1, 5.3_

### 8. ドキュメント整合性の確認と更新

実装完了後、以下のドキュメントとの整合性を確認・更新：

- [x] 8.1 SOUL.md - コアバリュー・設計原則との整合性確認
  - Phase 0 完了基準（DoD）への影響確認（今回は影響なし）
  - 段階的品質向上の原則に沿った小規模リファクタリングであることを検証
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [x] 8.2 doc/spec/ - 言語仕様の更新
  - DSL文法やランタイムへの影響はないため、更新不要であることを確認
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 8.3 TEST_COVERAGE.md - 新規テストのマッピング追加
  - `dist_src_validation_test.rs::test_dist_src_directory_structure` のマッピング追加
  - `integration_test.rs::test_generated_images_structure` のマッピング追加
  - 削除されたテスト（`test_directory_structure`, `test_install_txt`, `test_ghost_descript`, `test_pasta_toml`）のマッピング削除
  - _Requirements: 5.1, 5.2, 5.3_

- [x] 8.4 クレートREADME - API変更の反映
  - `crates/pasta_sample_ghost/README.md` の「配布物生成フロー」セクションを更新（テキスト xcopy → 画像生成の順序を反映）
  - `generate_ghost()` の責務変更（画像＋surfaces.txt のみ）を記載
  - _Requirements: 6.1, 6.2, 6.3_

- [x] 8.5 steering/* - 該当領域のステアリング更新
  - `.kiro/steering/workflow.md`: リリースフロー（release.ps1 の新ステップ）を反映
  - `.kiro/steering/structure.md`: `dist-src/` ディレクトリ構造を追記、`templates/` 削除を反映
  - _Requirements: 1.1, 1.2, 1.3, 4.1, 4.2_
