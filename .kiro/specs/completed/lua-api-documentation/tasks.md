# Implementation Plan: lua-api-documentation

## Task Overview

本仕様はドキュメント作成のみであり、コード変更を伴わないため、すべてのタスクは並列実行可能です。

## Tasks

- [x] 1. (P) LUA_API.md基本構造の作成
  - ファイルヘッダー（タイトル、概要）を作成
  - 目次セクションを配置（7つのメインセクションへのリンク）
  - Markdownフォーマットを既存README.mdに合わせる（日本語、GitHub互換）
  - _Requirements: 8.1, 8.2, 8.3_

- [x] 2. (P) モジュールカタログセクションの作成
  - pasta_lua固有モジュール（`@pasta_search`, `@pasta_config`, `@pasta_persistence`, `@enc`）をリスト化
  - mlua-stdlib統合モジュール（`@assertions`, `@testing`, `@regex`, `@json`, `@yaml`, `@env`）をリスト化
  - 各モジュールの名前、カテゴリ、バージョン、説明を表形式で整理
  - 機能別カテゴリ（検索系、設定系、永続化系、エンコーディング系、テスト系、ユーティリティ系）で分類
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 3. (P) @pasta_search モジュールドキュメントの作成
- [x] 3.1 (P) search_scene関数の文書化
  - 関数シグネチャ: `search_scene(name, global_scene_name?)`
  - パラメータ説明（name: 検索プレフィックス、global_scene_name: 親シーン名）
  - 戻り値説明（成功時: `(global_name, local_name)`、失敗時: `nil`）
  - フォールバック検索戦略（local → global）の説明
  - 実用例を3つ以上記載（グローバル検索、ローカル検索、フォールバック発動ケース）
  - _Requirements: 2.1, 2.4_

- [x] 3.2 (P) search_word関数の文書化
  - 関数シグネチャ: `search_word(name, global_scene_name?)`
  - パラメータ説明（name: 単語キー、global_scene_name: 親シーン名）
  - 戻り値説明（成功時: `string`、失敗時: `nil`）
  - フォールバック検索戦略（local → global）の説明
  - 実用例を2つ以上記載（グローバル単語、ローカル単語）
  - _Requirements: 2.2, 2.4_

- [x] 3.3 (P) テストユーティリティ関数の文書化
  - `set_scene_selector(...)` シグネチャと用途（決定論的テスト用）
  - `set_word_selector(...)` シグネチャと用途（決定論的テスト用）
  - 引数なし呼び出しでデフォルトに戻る動作を説明
  - MockRandomSelectorの概念を簡単に説明
  - 実用例（テストシーケンス設定）を記載
  - _Requirements: 2.3_

- [x] 4. (P) @pasta_persistence モジュールドキュメントの作成
- [x] 4.1 (P) load関数の文書化
  - 関数シグネチャ: `load()`
  - 戻り値（テーブルまたは空テーブル）
  - ファイル未存在時の動作（空テーブルを返す）を明記
  - エラーハンドリング（ファイル読み込み失敗時は空テーブル）
  - 実用例（初回起動と2回目以降）を記載
  - _Requirements: 3.1, 3.4_

- [x] 4.2 (P) save関数の文書化
  - 関数シグネチャ: `save(data)`
  - パラメータ説明（data: Luaテーブル）
  - 戻り値パターン（`(true, nil)` 成功、`(nil, error_message)` 失敗）
  - エラー条件（Lua値変換失敗、ファイル書き込み失敗）
  - 実用例（セーブデータの保存）を記載
  - _Requirements: 3.2_

- [x] 4.3 (P) 永続化設定オプションの説明
  - pasta.toml内`[persistence]`セクションの説明
  - `obfuscate`オプション（gzip圧縮有効化）の詳細
  - `file_path`オプション（保存先パス指定）の詳細
  - `debug_mode`オプション（ログ出力有効化）の詳細
  - 設定例を記載
  - _Requirements: 3.3_

- [x] 5. (P) @enc モジュールドキュメントの作成
- [x] 5.1 (P) to_ansi関数の文書化
  - 関数シグネチャ: `to_ansi(utf8_str)`
  - パラメータ説明（utf8_str: UTF-8文字列）
  - 戻り値パターン（`(ansi_string, nil)` 成功、`(nil, error_message)` 失敗）
  - エラーケース（無効なUTF-8入力、ANSI変換失敗）
  - Windowsファイルパス処理の実用例を記載
  - _Requirements: 4.1, 4.3, 4.4_

- [x] 5.2 (P) to_utf8関数の文書化
  - 関数シグネチャ: `to_utf8(ansi_str)`
  - パラメータ説明（ansi_str: ANSIバイト列）
  - 戻り値パターン（`(utf8_string, nil)` 成功、`(nil, error_message)` 失敗）
  - エラーケース（UTF-8変換失敗）
  - 実用例（ANSIファイルパスの読み込み後の変換）を記載
  - _Requirements: 4.2, 4.3_

- [x] 6. (P) @pasta_config モジュールドキュメントの作成
  - モジュールの性質（読み取り専用Luaテーブル）を説明
  - pasta.tomlの`custom_fields`セクションがどのように公開されるかを説明
  - ネストした設定値へのアクセス方法（`config.section.key`形式）を説明
  - 実用例（カスタム設定の読み取り）を2つ以上記載
  - _Requirements: 5.1, 5.2, 5.3_

- [x] 7. (P) pasta.finalize_scene 関数ドキュメントの作成
  - 関数の目的（Lua側レジストリからSearchContextを構築）を説明
  - 呼び出しタイミング（scene_dic.lua読み込み時）を明記
  - 処理フロー（シーン収集 → 単語収集 → @pasta_search登録）を説明
  - `pasta.scene`レジストリからのシーン収集処理を説明
  - `pasta.word`レジストリからの単語収集処理を説明
  - SearchContextの構築と`@pasta_search`モジュール登録を説明
  - 上級開発者向けの注記（カスタムローダー実装時の参考情報）を追加
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [x] 8. (P) mlua-stdlib 統合モジュールドキュメントの作成
  - デフォルト有効なモジュール一覧（`@assertions`, `@testing`, `@regex`, `@json`, `@yaml`）を記載
  - `@env`モジュールがセキュリティ上デフォルト無効であることを明記
  - RuntimeConfigによるモジュール有効化/無効化方法を説明
  - 各モジュールの簡単な用途説明を記載
  - mlua-stdlib公式ドキュメントへのリンクを配置
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [x] 9. (P) README.mdへのリンク追加
  - README.md内に「## API リファレンス」セクションを追加
  - LUA_API.mdへのリンクを配置
  - 簡単な説明文（「Rust側から公開されているLua APIの詳細は...」）を記載
  - 既存セクションとの整合性を確認
  - _Requirements: 8.4_

- [x]* 10. (P) ドキュメント品質検証
  - 全要件のカバレッジをトレーサビリティマトリクスで確認（要件1-8すべて）
  - コード例の文法チェック（Lua REPLで実行可能か確認）
  - 内部リンクの動作確認（目次 → 各セクション）
  - 外部リンクの有効性確認（mlua-stdlibドキュメント）
  - 日本語文法・表記の校正
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3, 3.4, 4.1, 4.2, 4.3, 4.4, 5.1, 5.2, 5.3, 6.1, 6.2, 6.3, 6.4, 7.1, 7.2, 7.3, 7.4, 8.1, 8.2, 8.3, 8.4_
