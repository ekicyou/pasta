# 実装計画

- [ ] 1. JSON-RPC 入力処理の安全性強化
- [x] 1.1 (P) server.rs の RwLock パニックリスク解消
  - `did_open` ハンドラの `self.documents.write().unwrap()` を `let Ok(mut docs) = self.documents.write() else { return; }` に置換
  - `did_change` ハンドラの `self.documents.write().unwrap()` を同様に置換
  - `did_close` ハンドラの `self.documents.write().unwrap()` を同様に置換
  - `semantic_tokens_full` ハンドラの `self.documents.read().unwrap()` を同様に置換
  - `analyze_and_publish` 内の `self.documents.write()` は既に `if let Ok` ガード済み — 確認のみ
  - 修正後、`cargo test -p pasta_lsp` で既存テスト全パスを確認
  - _Requirements: 1.5, 1.6_
  - _Boundary: PastaLangServer_

- [x] 1.2 (P) 入力検証の安全性確認
  - `did_open` で空文字列テキストを受信した場合の動作を既存テストで確認（`AnalysisEngine::analyze("")` が安全に動作すること）
  - `did_change` で未登録ドキュメントへの変更リクエストの動作を確認（`HashMap::get_mut` が `None` を返して安全にスキップ）
  - `line_col_to_offset` が範囲外の行番号・列番号に対して `None` を返すことを確認
  - 不足するカバレッジがあれば該当テストを追加
  - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - _Boundary: DocumentManager_

- [ ] 2. WASM 境界の安全性確認
- [x] 2.1 WASM エントリポイントの入力検証確認
  - `transport.rs` 末尾の `wasm_analyze()` 関数（`cfg(wasm32)` 条件コンパイル）を確認
  - 空文字列入力時に `AnalysisEngine::analyze("")` が空の解析結果を返すパスを確認
  - `serde-wasm-bindgen` によるシリアライズが型安全であること（`WasmAnalysisResult` → `JsValue` 変換）を確認
  - 確認結果をコード内コメントとして記録（WASM ビルドがなくても調査は可能）
  - _Requirements: 2.1, 2.2, 2.3, 2.4_
  - _Boundary: transport.rs_

- [ ] 3. パーサーパニック耐性の確認と文書化
- [x] 3.1 catch_unwind の安全性コメント追加
  - `analysis/mod.rs` の `analyze()` 内の `catch_unwind(AssertUnwindSafe(|| ...))` に安全性の根拠を説明するコメントを追加
  - `AssertUnwindSafe` の使用が安全である理由（`AnalysisEngine::analyze` は副作用のない純粋関数であり、パニック後に不整合状態が残らない）を文書化
  - パニック発生時に `client.log_message` でエラーログが出力されることを確認
  - _Requirements: 3.1, 3.2, 3.3_
  - _Boundary: AnalysisEngine_

- [ ] 4. デッドコード除去
- [x] 4.1 (P) LangServerError 使用状況調査と対応
  - `LangServerError` の3バリアント（`Parse`, `DocumentNotFound`, `Internal`）がクレート内外で使用されているか `grep` で調査
  - `lib.rs` で `pub use error::LangServerError` として公開されていることを確認
  - 未使用バリアントがあり、かつ外部から参照されていなければ除去
  - 外部利用の可能性がある場合は `#[allow(dead_code)]` コメント付きで残置（判断理由を記録）
  - _Requirements: 4.3, 4.4_
  - _Boundary: error.rs_

- [x] 4.2 (P) 未使用インポート・アイテムの除去
  - `cargo clippy -p pasta_lsp` で未使用の `use` 文・アイテムを検出
  - 検出された警告を修正
  - `cargo test -p pasta_lsp` で全テストパスを確認
  - _Requirements: 4.1, 4.2, 4.4_
  - _Boundary: 全ファイル_

- [ ] 5. 冗長表現の削減
- [x] 5.1 (P) document.rs の冗長パターン簡素化
  - `change()` メソッドの `if change.range.is_some() { if let Some(range) = &change.range {` を `if let Some(range) = &change.range { ... } else { ... }` に簡素化
  - 他に冗長なパターンがあれば同様に簡素化
  - `cargo test -p pasta_lsp` で全テストパスを確認
  - _Requirements: 5.2, 5.4, 5.5_
  - _Boundary: DocumentManager_

- [ ] 5.2 (P) server.rs の不要 clone 除去
  - `did_open` の `params.text_document.uri.clone()` と `params.text_document.text.clone()` の必要性を調査
  - 所有権の移動で代替可能な clone を除去
  - `did_change` の `params.content_changes.iter().map(|c| ...)` での `c.text.clone()` の必要性を調査
  - `cargo test -p pasta_lsp` で全テストパスを確認
  - _Requirements: 5.1, 5.4, 5.5_
  - _Boundary: PastaLangServer_

- [ ] 5.3 (P) transport.rs の WASM 型変換簡素化
  - `WasmSemanticToken`、`WasmDiagnostic` への変換を `From` トレイト実装に置き換え
  - `from_analysis()` メソッドを `From` トレイト呼び出しに簡素化
  - JSON シリアライズ出力が変更前と同一であることを確認
  - _Requirements: 5.3, 5.4_
  - _Boundary: transport.rs_

- [ ] 6. 最終検証
- [ ] 6.1 リグレッションテスト実行
  - `cargo test -p pasta_lsp` で全11テストファイルがパスすることを確認
  - `cargo clippy -p pasta_lsp` で新たな警告がないことを確認
  - `cargo build -p pasta_lsp` でネイティブビルドが成功することを確認
  - 全要件の受入基準が満たされていることを最終確認
  - _Requirements: 1.6, 2.4, 4.4, 5.4, 5.5_
  - _Depends: 1.1, 1.2, 2.1, 3.1, 4.1, 4.2, 5.1, 5.2, 5.3_
