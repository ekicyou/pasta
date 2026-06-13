# Implementation Plan

- [ ] 1. デバッグ起動ログの実装
- [x] 1.1 待ち受け成功 info ログとバインド失敗 warn ログを enable() に追加
  - 有効ゲート通過後・local_addr 取得後に、実バインドアドレスを含む info ログ（メッセージ `debug backend listening`、`addr` 構造化フィールド）を 1 件出力する
  - Transport::start 失敗時に、試行アドレスと失敗事由を含む warn ログ（メッセージ `debug transport bind failed`、`addr` / `error` フィールド）を 1 件出力し、DebugError::Bind の伝播挙動は変えない（cfg.listen は Option のため Some を分割代入してから Display 出力）
  - 無効時の早期 return 経路には一切手を入れず、ゼロコスト・無言を維持する
  - 観測: デバッグ有効でゴースト起動時、pasta.log に `debug backend listening addr=127.0.0.1:<port>` が 1 件出る／無効時は一切出ない
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 4.1, 4.2, 4.4_
  - _Boundary: enable() ログ点_

- [ ] 2. 検証
- [x] 2.1 ログ出力/非出力のユニットテスト追加
  - 既存 enable テストに `#[traced_test]` を付与し、有効時に `debug backend listening` と取得した実 local_addr 文字列が出力されることを検証（port 0 で要求値ではなく実ポート確定を確認）
  - 無効時に待ち受け関連ログが一切出ないことを logs_contain の否定で検証
  - バインド失敗時に warn（`debug transport bind failed`）が出て info（`debug backend listening`）が出ないことを検証
  - 観測: cargo test で 3 ケースが緑。テストはポート 0 使用・DebugConfig 直接構築で PASTA_DEBUG env に非依存
  - _Requirements: 1.1, 1.2, 1.4, 1.5, 2.1, 2.2, 3.1_
  - _Depends: 1.1_
  - _Boundary: enable() ログ点（#[cfg(test)] mod tests）_

- [x] 2.2 非回帰確認
  - `cargo test --all` が緑であることを確認（既存デバッグ機能挙動の不変・ログ基盤不変・新依存なし）
  - LuaJIT ビルドは環境変数 `NoDefaultCurrentDirectoryInExePath` を外して実行する
  - 観測: `cargo test --all` 全パス、Cargo.toml に新規依存の差分なし
  - _Requirements: 4.1, 4.3, 4.4_
  - _Depends: 1.1, 2.1_
