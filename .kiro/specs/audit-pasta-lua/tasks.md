# 実装計画

> **Note**: 本specの `_Boundary:_` はモジュール/ディレクトリ単位で記述しています（例: `code_gen` = `src/code_gen/` 配下全ファイル）。pasta_luaのモジュール構成が深いため、ファイル単位ではなくモジュール単位で管理します。

- [x] 1. 基盤: 監査前ベースライン取得
- [x] 1.1 現状メトリクス収集と監査チェックリスト準備
  - pasta_lua/src/ 全ファイルの行数を記録（ベースライン）
  - `cargo test -p pasta_lua` および `cargo test --workspace` が全パスすることを確認
  - `unsafe` ブロック・`unwrap()`・`unreachable!()` の全出現箇所をgrepでリスト化
  - 監査完了時に行数比較・テスト結果比較に使用するベースラインファイルが存在する状態
  - _Requirements: 8.1, 8.2, 8.4_

- [ ] 2. コード生成モジュールの監査
- [x] 2.1 (P) element_gen.rs の複雑度削減
  - 繰り返しwrite!マクロパターンを特定し、共通ヘルパー関数への抽出を検討
  - `unreachable!()` マクロ（VarScope::Property）の到達不能性を検証し、SAFETYコメントまたは適切なエラーハンドリングに置換
  - デッドコード（未使用の分岐、到達しないマッチアーム）を除去
  - 変更後に `cargo test -p pasta_lua` が全パスし、instaスナップショットが一致する状態
  - _Requirements: 3.1, 3.2, 3.4_
  - _Boundary: code_gen_

- [x] 2.2 (P) scope_gen.rs の重複パターン共通化
  - シーン・アクターレベルのスコープ生成で重複するコード構造を特定
  - 共通化可能なパターンをヘルパーに抽出
  - 変更後に `cargo test -p pasta_lua` が全パスし、instaスナップショットが一致する状態
  - _Requirements: 3.3, 3.4_
  - _Boundary: code_gen_

- [ ] 3. ランタイムモジュールの監査（セキュリティ）
- [x] 3.1 unsafe ブロックのSAFETYコメント付与と安全性検証
  - `runtime/mod.rs:101` の `Lua::unsafe_new_with` に `// SAFETY:` コメント付与（StdLibパラメータの妥当性、メモリ安全性の前提条件を記述）
  - `runtime/enc.rs:146` の `Lua::unsafe_new_with` に `// SAFETY:` コメント付与
  - `encoding/windows.rs:112, 168` の Windows FFI呼び出しに `// SAFETY:` コメント付与（バッファサイズ検証、ヌルポインタチェック、戻り値検証の前提条件を記述）
  - 全unsafeブロックにSAFETYコメントが存在することをgrep確認できる状態
  - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - _Boundary: runtime, encoding_

- [ ] 3.2 Lua実行パスの安全性検証
  - `finalize.rs` のハードコードされた `require` 呼び出しにインジェクションリスクがないことを確認し、安全性コメントを付与
  - `runtime/mod.rs` の `exec()` メソッドの入力源を追跡し、外部ユーザー入力の直接注入パスが存在しないことを文書化
  - `lua.load().eval()` の `.unwrap()` 使用がテストコード外に存在しないことを確認（存在する場合は `?` 演算子に置換）
  - 全eval/exec呼び出しに安全性の根拠コメントが存在する状態
  - _Requirements: 2.1, 2.2, 2.4_
  - _Boundary: runtime_

- [ ] 4. ランタイムモジュールの監査（簡素化）
- [ ] 4.1 (P) finalize.rs のレジストリ収集ロジック簡素化
  - ネストテーブル走査の冗長パターンを特定し削減
  - シーン収集とワード収集で重複するテーブル操作を共通化
  - 変更後にランタイムテストが全パスする状態
  - _Requirements: 4.1, 4.4_
  - _Boundary: runtime/finalize_

- [ ] 4.2 (P) persistence.rs のファイルI/Oエラーハンドリング検証
  - ファイル読み書き操作のエラーパスを全て追跡
  - `.unwrap()` が使用されている箇所を `?` 演算子または適切なエラー型に置換
  - ファイルパスの検証が適切であることを確認
  - 変更後にpersistenceテストが全パスする状態
  - _Requirements: 4.2, 4.4_
  - _Boundary: runtime/persistence_

- [ ] 4.3 (P) module_registry.rs の重複パターン共通化
  - モジュール登録関数間の重複するボイラープレートを特定
  - 共通パターンをヘルパー関数またはマクロに抽出
  - 変更後にランタイムテストが全パスする状態
  - _Requirements: 4.3, 4.4_
  - _Boundary: runtime/module_registry_

- [ ] 5. トランスパイラモジュールの監査
- [ ] 5.1 transpiler.rs のフェーズ責務分離と冗長コード削減
  - マルチフェーズトランスパイルの各フェーズの責務を明確化
  - 冗長な中間状態・デッドコードを除去
  - 関数レベルの複雑度を削減（長い関数の分割検討）
  - 変更後にトランスパイラテストおよびinstaスナップショットが全パスする状態
  - _Requirements: 5.1, 5.2, 5.3_
  - _Boundary: transpiler_

- [ ] 6. ローダー・ユーティリティモジュールの監査
- [ ] 6.1 (P) loader/ モジュールのセキュリティ検証と簡素化
  - ファイルパス操作がディレクトリトラバーサルに対して安全であることを検証（`..` や絶対パスの処理確認）
  - discovery.rs のファイル検出パスが意図したディレクトリ外にアクセスしないことを確認
  - cache.rs のキャッシュ管理で不要な複雑性を削減
  - config.rs の設定解析で冗長パターンを削減
  - 変更後にローダーテストが全パスする状態
  - _Requirements: 6.1, 6.4_
  - _Boundary: loader_

- [ ] 6.2 (P) sakura_script/ モジュールのセキュリティ検証と簡素化
  - tokenizer.rs の正規表現パターンにReDoS脆弱性がないことを検証（バックトラッキングの爆発的増加がない正規表現構造を確認）
  - wait_inserter.rs の `unreachable!()` マクロの到達不能性を検証
  - line_breaker.rs の冗長パターンを確認し削減
  - 変更後にさくらスクリプトテストが全パスする状態
  - _Requirements: 6.2, 6.4_
  - _Boundary: sakura_script_

- [ ] 6.3 (P) logging/ およびその他ユーティリティの監査
  - logging/ でユーザーデータや機密情報がログ出力に含まれないことを検証
  - context.rs, normalize.rs, string_literalizer.rs のデッドコードと冗長パターンを確認し削減
  - 変更後に関連テストが全パスする状態
  - _Requirements: 6.3, 6.4_
  - _Boundary: logging, utilities_

- [ ] 7. Luaスクリプト群の安全性調査
- [ ] 7.1 pasta_scripts/ の安全性検証
  - 全.luaファイルでグローバル変数の意図しない汚染がないことを検証（`local` 宣言の網羅性確認）
  - `os.execute`, `io.popen`, `loadstring`, `dofile` 等の危険関数の使用がないことをgrepで確認
  - スクリプト間の依存関係（`require` 呼び出し）を追跡し循環参照がないことを確認
  - luacheckが警告なしでパスする状態
  - _Requirements: 7.1, 7.2, 7.3_
  - _Boundary: lua_scripts_

- [ ] 7.2 scripts/ ユーザースクリプトテンプレートの安全性確認
  - ユーザースクリプトテンプレートに安全でないパターンが含まれないことを確認
  - lua_testフレームワークによるBDDテストが全パスすることを確認
  - _Requirements: 7.2, 7.4_
  - _Boundary: lua_scripts_

- [ ] 8. 統合検証と最終確認
- [ ] 8.1 全体回帰テストと行数比較
  - `cargo test -p pasta_lua` が全パスすることを確認
  - `cargo test --workspace` が全パスすることを確認（pasta_shiori等の下流クレート含む）
  - 監査前後の行数を比較し総行数が削減されていることを確認
  - 全unsafeブロックにSAFETYコメントが存在することを最終確認
  - 全要件の受け入れ基準を充足していることを確認できる状態
  - _Requirements: 8.1, 8.2, 8.3, 8.4_
