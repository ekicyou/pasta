# 実装計画

- [ ] 1. エラーメッセージ言語・文体の横断的統一
- [x] 1.1 全クレートの `#[error("...")]` メッセージの言語確認と英語化
  - 全7クレートの `error.rs` / エラー型定義を横断的に `grep` で確認し、日本語メッセージを英語に置換する
  - `pasta_core/src/error.rs` の `WordTableError::WordNotFound` メッセージが英語化されていることを確認（未対応なら修正）
  - エラーメッセージ文体を統一: 文頭大文字、末尾ピリオドなし、コロン区切りでコンテキスト付与
  - `pasta_shiori/src/error.rs` の `MyError::Others` メッセージ `"others error"` → `"Others error"` への文頭大文字統一（Wave 1で除去済みなら確認のみ）
  - 全 `#[error("...")]` に日本語が残存しないことを `grep` で確認完了
  - _Requirements: 1.1, 1.2, 1.3_

- [ ] 2. エラー型定義パターンの一貫性確認
- [x] 2.1 全クレートのエラー型パターン一貫性チェック
  - 全7クレートのエラー型定義が `thiserror::Error` derive マクロを使用していることを確認
  - `#[from]` 自動変換の使用箇所を全クレートで確認し、パターンの一貫性をチェック
  - 未使用エラーバリアントの横断的確認（`cargo clippy --workspace` の `dead_code` 警告）
  - 不一致が発見された場合のみ修正を実施、修正後 `cargo test -p <crate>` でパス確認
  - 全エラー型が一貫したパターンに従っていることの確認結果を記録
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [ ] 3. tracing インポートパターンの統一
- [x] 3.1 ワイルドカードインポートの明示的インポートへの変換
  - `crates/pasta_shiori/src/windows.rs` の `use tracing::*` を、実際に使用されているマクロのみの明示的インポートに変換
  - 変換前に該当ファイルで使用されている tracing マクロを特定（`debug!`, `error!`, `info!`, `trace!`, `warn!` 等）
  - 全クレートで `use tracing::*` が存在しないことを `grep` で最終確認
  - `cargo test -p pasta_shiori` でパス確認
  - _Requirements: 4.1, 4.2, 4.3_
  - _Boundary: pasta_shiori/windows.rs_

- [ ] 4. ファイルI/Oエラーハンドリングパターンの一貫性確認
- [x] 4.1 横断的ファイルI/Oパターンの一貫性チェック
  - `pasta_check` (copy.rs, nar.rs, update_files.rs) のパストラバーサル防御・シンボリックリンクスキップの実装パターンを確認
  - `pasta_lua/src/loader/` のファイル検出パスの安全性パターンを確認
  - `pasta_sample_ghost` のファイルI/Oパターンを確認
  - 各クレート間でパターンが一貫していることを確認（共通化は不要 — 用途が異なる）
  - 不一致が発見された場合のみ修正を実施
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 5. `unwrap()`/`expect()` の横断的最終検証
- [x] 5.1 全クレートのプロダクションコードにおける `unwrap()`/`expect()` 検証
  - `grep -rn '\.unwrap()' crates/*/src/ --include='*.rs'` で全 `unwrap()` 使用箇所を検出
  - テストコード（`#[cfg(test)]`, `tests/`）を除外した上でプロダクションコードの `unwrap()` をリストアップ
  - 発見された `unwrap()` が安全に使用されている（論理的にパニックしない）ケースかを判定
  - 安全でない `unwrap()` が発見された場合、`Result` 伝搬や `.ok_or()` への置換を実施
  - `expect()` の使用が「Cargoが保証する環境変数の取得」等の正当なケースのみであることを確認
  - 全修正後に `cargo test --workspace` でパス確認
  - _Requirements: 5.1, 5.2, 5.3_

- [ ] 6. `pub` 可視性の横断的検証
- [x] 6.1 全クレートの不必要な `pub` の検出と `pub(crate)` 化
  - `cargo clippy --workspace` の可視性関連警告を確認
  - 内部ヘルパー関数・型が不必要に `pub` になっている箇所を特定
  - 下流クレートから使用されていない `pub` 項目を `pub(crate)` に縮小
  - 各 `lib.rs` の re-export が最小限かつ一貫したパターンであることを確認
  - 変更後 `cargo test --workspace` でパス確認（下流クレートのコンパイルエラーがないことを保証）
  - _Requirements: 6.1, 6.2, 6.3_

- [ ] 7. コンパイラ警告ゼロと全テストパスの最終確認
- [x] 7.1 ワークスペース全体の最終品質チェック
  - `cargo clippy --workspace` で警告ゼロを確認、残存警告があれば修正
  - `#[allow(...)]` 属性の使用が正当な理由付きに限定されていることを横断的に確認
  - `cargo test --workspace` で全テストパスを確認
  - CLIツール（pasta_check）の出力が改善前と同一であることを確認
  - 全公開APIシグネチャが変更されていないことを確認
  - _Requirements: 7.1, 7.2, 7.3, 8.1, 8.2, 8.3, 8.4_

## Implementation Notes
- Task 4.1: サブエージェントが境界外のファイルに CRLF/改行変更を加える傾向あり。レビュー前に `git diff --name-only` で境界外変更を revert すること。
- Task 4.1: `cargo clippy --workspace` には pasta_lua/pasta_lsp/pasta_check に既存警告が残存。Task 7.1 で対応予定。
