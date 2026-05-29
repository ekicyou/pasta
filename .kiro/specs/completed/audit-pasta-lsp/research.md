# 調査・設計判断記録

## 概要

- **機能**: `audit-pasta-lsp`
- **調査スコープ**: 拡張（既存クレートの監査）
- **主要な発見**:
  - `server.rs` に3箇所の `RwLock::write().unwrap()` と2箇所の `RwLock::read().unwrap()` が存在
  - `document.rs` の `line_col_to_offset` は境界チェック付きで `Option` を返す（安全）
  - `error.rs` の `LangServerError` は現在クレート内で直接使用されていない可能性がある
  - `transport.rs` の WASM 型変換は個別フィールドマッピングで冗長だが、`From` トレイトへの置き換えで簡素化可能

## 調査記録

### RwLock パニックリスク

- **背景**: `server.rs` で `self.documents.write().unwrap()` を3箇所、`self.documents.read().unwrap()` を2箇所使用
- **調査**: `RwLock::write()` は、別スレッドがロック保持中にパニックした場合にポイズニングされ、以降すべての `unwrap()` がパニックする
- **影響**: LSP サーバーは通常シングルスレッド的に動作するため実際のリスクは低いが、防御的プログラミングとして `unwrap()` を排除すべき
- **結論**: `let Ok(docs) = self.documents.write() else { return; }` パターンに置換

### LangServerError 使用状況

- **背景**: `error.rs` で3バリアント（`Parse`, `DocumentNotFound`, `Internal`）を定義
- **調査**: `lib.rs` で `pub use error::LangServerError` として公開エクスポートされているが、クレート内部での使用箇所を確認する必要がある
- **結論**: pub API として公開されているため、外部利用者がいる可能性を考慮して慎重に判断。未使用の場合でも `#[allow(dead_code)]` ではなく除去を検討するが、pub re-export は維持する方向で調査

### document.rs の冗長パターン

- **背景**: `change()` メソッドで `if change.range.is_some() { if let Some(range) = &change.range {` という二重チェックが存在
- **調査**: `is_some()` チェックの直後に `if let Some` で再チェックしており、冗長
- **結論**: `if let Some(range) = &change.range { ... } else { ... }` に簡素化

### transport.rs WASM 型変換

- **背景**: `WasmAnalysisResult::from_analysis()` で `SemanticToken` → `WasmSemanticToken`、`Diagnostic` → `WasmDiagnostic` の変換を個別フィールドで実行
- **調査**: `From` トレイト実装に変換することで可読性向上可能。ただし型数が少ないため効果は限定的
- **結論**: `From` トレイト実装への変換で簡素化（コード量削減は小規模だが可読性向上）

## 設計判断

### 判断: RwLock エラーハンドリング方針

- **背景**: `unwrap()` を除去する際の代替パターン
- **代替案**:
  1. `unwrap_or_else` + ログ出力
  2. `let-else` + 早期リターン
  3. `match` + エラーログ
- **選択**: `let-else` + 早期リターン（Rust 2024 edition の慣用パターン）
- **理由**: 最も簡潔で、LSP ハンドラの `async fn` と相性が良い
- **トレードオフ**: ポイズニング時にサイレントにリクエストを無視するが、LSP クライアント側でタイムアウト検知される

## リスクと緩和策

- **リスク1**: `LangServerError` 除去による下流影響 — pub API として公開されているため、外部利用者の有無を確認してから判断
- **リスク2**: WASM 型変換の簡素化による互換性破壊 — JSON シリアライズ形式の同一性をテストで確認
- **リスク3**: `catch_unwind` コメント追加による不必要な変更 — ドキュメントコメントのみで実装ロジックに触れない
