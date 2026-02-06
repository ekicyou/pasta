# 実装タスク: alpha04-sample-ghost

## タスク概要

hello-pasta サンプルゴーストの完全実装。SSPにインストールして即座に動作する完全な配布物として、pasta.dll を使った実働するゴーストを `crates/pasta_sample_ghost/ghosts/hello-pasta/` に完成させる。

**憲法的目的**: pasta.dll を使った**実働する**サンプルゴースト配布物を作ること

---

## 実装タスク

### 1. クレート基盤構築

- [x] 1.1 (P) pasta_sample_ghost クレート初期化
  - `crates/pasta_sample_ghost/` ディレクトリ構造確認・整備
  - Cargo.toml の依存関係確認（image 0.25, imageproc 0.25）
  - workspace members への登録確認
  - pasta_lua からの責務分離確認（依存なし）
  - _Requirements: 1.3, 1.4_

- [x] 1.2 (P) 公開 API 定義
  - `src/lib.rs` に `generate_ghost()` 関数シグネチャ実装
  - `GhostConfig` 構造体定義（name, version, sakura_name, kero_name, craftman, shiori, homeurl）
  - `GhostError` エラー型定義（ImageError, IoError, ConfigError）
  - 各サブモジュールの公開（image_generator, config_templates, scripts）
  - _Requirements: 1.1, 1.2_

- [x] 1.3 テンプレートディレクトリ整備
  - `ghosts/hello-pasta/` 静的ファイル構造確認
  - install.txt, readme.txt の存在確認
  - ghost/master/, shell/master/ サブディレクトリ構造確認
  - dic/ 空ディレクトリ（ビルド時生成用）確認
  - _Requirements: 1.1, 1.5_

---

### 2. シェル画像生成機能

- [x] 2.1 (P) ピクトグラム描画ロジック
  - Character enum 定義（Sakura: 赤 #DC3545, Kero: 青 #007BFF）
  - 128×256 ピクセル透過 PNG 生成関数
  - 3頭身比率の定数定義（HEAD_RADIUS: 42px, 頭部中心: 47px）
  - 頭部（塗りつぶし円）描画
  - 胴体描画（女の子: ○+△、男の子: ○+▽）
  - _Requirements: 6.1, 6.2_

- [x] 2.2 (P) 表情バリエーション実装
  - Expression enum 定義（9種: Smile, Normal, Shy, Surprised, Crying, Confused, Sparkle, Sleepy, Angry）
  - 表情描画関数（線の太さ 3px、目の間隔 36px）
  - 各表情パターン実装（`^ ^`, `- -`, `> <`, `o o`, `; ;`, `@ @`, `* *`, `= =`, `# #`）
  - フォント不使用（CI再現性確保）
  - _Requirements: 6.3, 6.5_

- [x] 2.3 サーフェス一括生成
  - `generate_surfaces()` 関数実装
  - sakura サーフェス生成（surface0.png 〜 surface8.png）
  - kero サーフェス生成（surface10.png 〜 surface18.png）
  - PNG ファイル書き込み（外部依存なし）
  - _Requirements: 6.1, 6.4_

---

### 3. 設定ファイル生成機能

- [x] 3.1 (P) ukadoc 設定ファイルテンプレート
  - `generate_install_txt()` 実装（charset, type, name, directory）
  - `generate_ghost_descript_txt()` 実装（charset, type, shiori, sakura.name, kero.name）
  - `generate_shell_descript_txt()` 実装（charset, type, name, balloon offset）
  - `generate_surfaces_txt()` 実装（SERIKO 形式サーフェス定義）
  - UTF-8 BOM なしエンコーディング
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 3.2 (P) pasta.toml 設定テンプレート
  - `generate_pasta_toml()` 実装
  - [package] セクション（教育的コメント付き）
  - [loader] セクション（pasta_patterns, lua_search_paths, transpiled_output_dir）
  - [ghost] セクション（random_talk_interval）
  - [persistence] セクション（data_dir）
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [x] 3.3 ディレクトリ構造生成統合
  - `generate_structure()` 関数実装
  - ghost/master/, shell/master/, dic/ サブディレクトリ作成
  - 全設定ファイルの一括出力
  - _Requirements: 1.1, 1.5_

---

### 4. pasta DSL スクリプト実装

- [x] 4.0 アクター辞書（actors.pasta）
  - ％女の子 アクター定義（全9表情：通常、笑顔、照れ、驚き、泣き、困惑、キラキラ、眠い、怒り）
  - ％男の子 アクター定義（全9表情）
  - surface0-8, surface10-18 への対応マッピング
  - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

- [x] 4.1 起動・終了トーク（boot.pasta）
  - 単語定義（＠起動挨拶、＠終了挨拶）
  - OnFirstBoot シーン実装（初回起動メッセージ）
  - OnBoot シーン実装（複数パターン）
  - OnClose シーン実装（複数パターン）
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 4.2 (P) ダブルクリック反応（click.pasta）
  - OnMouseDoubleClick シーン実装（7種以上）
  - ランダム選択による反応多様性
  - シンプルな pasta DSL のみの実装（アクター辞書は actors.pasta に委譲）
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 4.3 (P) ランダムトーク（talk.pasta - OnTalk）
  - 単語定義（＠雑談）
  - OnTalk シーン実装（6種以上）
  - sakura と kero の掛け合いトーク（アクター辞書は actors.pasta に委譲）
  - _Requirements: 4.1, 4.2, 4.3_

- [x] 4.4 (P) 時報（talk.pasta - OnHour）
  - OnHour シーン実装（3種以上）
  - `＄時１２` 変数参照（12時間表記）
  - onhour-date-var-transfer 仕様準拠確認
  - _Requirements: 5.1, 5.2, 5.3_

- [x] 4.5 スクリプト生成統合
  - `generate_scripts()` 関数実装
  - ACTORS_PASTA, BOOT_PASTA, TALK_PASTA, CLICK_PASTA 定数定義
  - dic/ ディレクトリへの一括出力（actors.pasta 含む）
  - _Requirements: 1.1, 11.4_

---

### 5. 配布物生成統合

- [x] 5.1 generate_ghost() 統合 API
  - config_templates::generate_structure() 呼び出し
  - image_generator::generate_surfaces() 呼び出し
  - scripts::generate_scripts() 呼び出し
  - エラーハンドリング統合
  - _Requirements: 1.1, 1.2, 1.5_

- [x] 5.2 generate-surfaces バイナリ
  - src/bin/generate-surfaces.rs 実装
  - コマンドライン引数処理（output-dir）
  - generate_ghost() 呼び出し
  - エラーメッセージ出力
  - _Requirements: 10.2_

---

### 6. 統合テスト実装

- [x] 6.1 テストヘルパー実装
  - tests/common/mod.rs 作成
  - `copy_pasta_shiori_dll()` 実装（target/i686-pc-windows-msvc/release/pasta.dll 検出）
  - DLL 不在時の明確なエラーメッセージ
  - TempDir テスト環境構築ヘルパー
  - _Requirements: 8.1, 8.2, 8.3_

- [x] 6.2 (P) ディレクトリ構造検証テスト
  - `test_directory_structure()` 実装
  - install.txt, descript.txt, pasta.toml 存在確認
  - dic/*.pasta, shell/*.png 存在確認
  - ディレクトリ階層正確性検証
  - _Requirements: 8.4, 8.7_

- [x] 6.3 (P) ukadoc ファイル検証テスト
  - `test_ukadoc_files()` 実装
  - install.txt 必須フィールド検証（type, name, directory）
  - ghost/master/descript.txt 必須フィールド検証（type, shiori, sakura.name, kero.name）
  - shell/master/descript.txt 必須フィールド検証（type, name, balloon offset）
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 6.4 (P) pasta DSL 検証テスト
  - `test_pasta_scripts()` 実装
  - actors.pasta アクター辞書存在確認（％女の子、％男の子）
  - OnFirstBoot, OnBoot, OnClose シーン存在確認
  - OnMouseDoubleClick パターン数確認（7種以上）
  - OnTalk パターン数確認（5種以上）
  - OnHour + `＄時１２` 参照確認
  - boot.pasta, talk.pasta, click.pasta にアクター辞書が**含まれない**ことを確認
  - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 4.1, 4.2, 5.1, 5.2, 11.1, 11.2, 11.3_

- [x] 6.5 (P) シェル画像検証テスト
  - `test_shell_images()` 実装
  - surface0-8, surface10-18 全18画像存在確認
  - 画像サイズ検証（128×256）
  - PNG フォーマット検証
  - _Requirements: 6.1, 6.4_

- [x] 6.6 (P) pasta.toml 検証テスト
  - `test_pasta_toml_content()` 実装
  - [package] セクション確認
  - [loader] lua_search_paths 確認
  - [ghost] random_talk_interval 確認
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [x] 6.7 CI 統合確認
  - `cargo test --workspace` 成功確認
  - 実SSP不要のモック環境完結性確認
  - テスト前提条件（pasta.dll ビルド）ドキュメント化
  - _Requirements: 8.6, 8.7, 8.8_

---

### 7. 配布ビルド自動化

- [x] 7.1 build-ghost.ps1 スクリプト作成
  - scripts/build-ghost.ps1 実装
  - パラメータ定義（-OutputDir）
  - エラーハンドリング（$ErrorActionPreference = "Stop"）
  - 進捗表示（Write-Host）
  - _Requirements: 10.1_

- [x] 7.2 DLL ビルドステップ
  - cargo build --release --target i686-pc-windows-msvc -p pasta_shiori
  - 32bit Windows ターゲット確認
  - ビルド失敗時の適切なエラー処理
  - _Requirements: 10.2_

- [x] 7.3 テンプレート・成果物配置ステップ
  - build.rs が ghosts/hello-pasta/ に .pasta, .png を生成
  - 静的ファイルは ghosts/hello-pasta/ に配置済み
  - ディレクトリ構造維持
  - _Requirements: 10.2_

- [x] 7.4 画像・スクリプト生成ステップ
  - cargo run --package pasta_sample_ghost --bin generate-surfaces
  - 生成先ディレクトリ指定
  - 生成失敗時のエラー処理
  - _Requirements: 10.2_

- [x] 7.5 DLL・Lua ランタイムコピーステップ
  - pasta.dll コピー（target/i686.../release/pasta.dll → dist/.../ghost/master/pasta.dll）
  - Lua ランタイムコピー（crates/pasta_lua/scripts/ → dist/.../ghost/master/scripts/）
  - scriptlibs 除外（テスト用ライブラリは配布に含めない）
  - _Requirements: 10.3, 10.4_

- [x] 7.6 配布物検証ステップ
  - 必須ファイル存在確認（install.txt, descript.txt, pasta.dll, *.pasta, *.png）
  - 不足ファイル警告表示
  - ビルド成功メッセージ
  - _Requirements: 10.5_

---

### 8. ドキュメント整合性確認

- [x] 8.1 SOUL.md 整合性確認
  - pasta DSL の教育的サンプルとしての「学習可能性」確認
  - ukadoc 準拠による「伺か」エコシステム統合確認
  - コアバリュー（日本語フレンドリー、宣言的フロー）との整合性確認
  - _Requirements: 全要件_

- [x] 8.2 ドキュメント更新
  - README.md 更新（クレート概要、使用方法、ビルド手順）
  - TEST_COVERAGE.md 更新（新規テストのマッピング）
  - steering ファイルとの整合性確認
  - _Requirements: 全要件_

---

## 完了基準（DoD）

- [x] **Spec Gate**: requirements, design, tasks すべて承認済み
- [x] **Test Gate**: `cargo test --package pasta_sample_ghost` 成功（24テスト）
- [x] **Build Gate**: `cargo test -p pasta_sample_ghost` 成功、`ghosts/hello-pasta/` 完成
- [x] **Doc Gate**: README.md にクレート概要・使用方法記載
- [x] **Steering Gate**: steering/structure.md, steering/tech.md との整合性確認
- [x] **Soul Gate**: SOUL.md との整合性確認完了

---

## 要件カバレッジ

| 要件ID | 要件名 | 対応タスク |
|--------|--------|-----------|
| 1.1-1.5 | ディレクトリ構成 | 1.1, 1.2, 1.3, 3.3, 5.1 |
| 2.1-2.4 | 起動・終了トーク | 4.1 |
| 3.1-3.4 | ダブルクリック反応 | 4.2 |
| 4.1-4.3 | ランダムトーク | 4.3 |
| 5.1-5.3 | 時報 | 4.4 |
| 6.1-6.5 | シェル素材 | 2.1, 2.2, 2.3 |
| 7.1-7.4 | 設定ファイル（pasta.toml） | 3.2 |
| 8.1-8.8 | テスト要件 | 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7 |
| 9.1-9.4 | ukadoc設定ファイル | 3.1 |
| 10.1-10.6 | 配布ビルド自動化 | 7.1, 7.2, 7.3, 7.4, 7.5, 7.6 |

**全10要件、36個のAcceptance Criteriaをカバー。**
