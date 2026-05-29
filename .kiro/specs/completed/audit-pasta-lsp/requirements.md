# 要件定義書

## 導入

pasta_lsp は tower-lsp ベースの LSP サーバーラッパーであり、Pasta DSL ファイル（`*.pasta`）に対してセマンティックトークンハイライトとパースエラー診断を提供する。約400行の比較的小規模なクレートだが、ネットワーク経由の JSON-RPC リクエスト処理と WASM 対応コード（`wasm-bindgen`, `js-sys`）を含む。

本監査では、JSON-RPC 入力処理の安全性検証、WASM 境界の安全性確認、デッドコード除去、冗長表現削減を実施する。既存テスト全パスと外部振る舞い（LSP レスポンス）不変を前提条件とする。

## 境界コンテキスト

- **対象**: `crates/pasta_lsp/src/` 配下の全ファイル（`lib.rs`, `server.rs`, `document.rs`, `error.rs`, `transport.rs`, `analysis/`）
- **対象外**: tower-lsp クレートの内部実装、VS Code 拡張（`editors/vscode/`）の変更、pasta_dsl パーサーの変更、LSP プロトコル仕様の変更、新しい LSP 機能の追加
- **隣接する期待**: pasta_dsl パーサーはアップストリーム依存であり、本監査ではその API を消費するのみ。pasta_dsl 側の脆弱性は `audit-pasta-dsl` の責務である

## 要件

### 要件 1: JSON-RPC 入力処理の安全性

**目的:** LSP サーバー運用者として、ネットワーク経由で受信する JSON-RPC リクエストが安全に処理されることを確認し、不正な入力による予期しない動作を防止したい

#### 受入基準

1. When LSP サーバーが不正な URI を含む `didOpen` リクエストを受信した場合, the pasta_lsp shall パニックせずにリクエストを処理する
2. When LSP サーバーが空文字列のドキュメントテキストを含む `didOpen` リクエストを受信した場合, the pasta_lsp shall 正常にドキュメントを登録し解析結果を返す
3. When LSP サーバーが範囲外の行番号・列番号を含む `didChange` リクエストを受信した場合, the pasta_lsp shall パニックせずにリクエストを処理する
4. When LSP サーバーが `didOpen` 前のドキュメントに対する `didChange` を受信した場合, the pasta_lsp shall パニックせずにリクエストを無視する
5. When `RwLock` の `write()` がポイズニングした場合, the pasta_lsp shall `unwrap()` によるパニックではなく安全に処理する
6. The pasta_lsp shall すべての既存テストがパスする状態を維持する

### 要件 2: WASM 境界の安全性

**目的:** VS Code 拡張開発者として、WASM ビルドで公開されるエントリポイントが安全であり、不正な JavaScript 入力がクラッシュを引き起こさないことを確認したい

#### 受入基準

1. When WASM エントリポイント `wasm_analyze()` が空文字列を受け取った場合, the pasta_lsp shall 有効な空の解析結果を返す
2. When WASM エントリポイントが不正なUTF-8を含む入力を受け取った場合, the pasta_lsp shall パニックせずにエラーを返す
3. The pasta_lsp shall `serde-wasm-bindgen` によるシリアライズが型安全であることを維持する
4. The pasta_lsp shall WASM ターゲット（`cfg(target_arch = "wasm32")`）のコンパイル互換性を維持する

### 要件 3: パーサーパニック耐性

**目的:** LSP サーバー運用者として、アップストリームパーサー（pasta_dsl）のパニックがサーバー全体をクラッシュさせないことを確認したい

#### 受入基準

1. When `AnalysisEngine::analyze()` 内で `pasta_dsl::parse_str()` がパニックした場合, the pasta_lsp shall `catch_unwind` でパニックを捕捉しエラーログを出力する
2. When パーサーパニックが発生した場合, the pasta_lsp shall 他のドキュメントの処理を継続する
3. The pasta_lsp shall `catch_unwind` の使用が `AssertUnwindSafe` ラッパーの安全性前提を文書化する

### 要件 4: デッドコード除去

**目的:** メンテナとして、未使用のコード・型・インポートが除去され、コードベースの保守性が向上することを確認したい

#### 受入基準

1. The pasta_lsp shall 未使用の `pub` アイテム（関数、型、定数）が存在しない状態にする
2. The pasta_lsp shall 未使用のインポート（`use` 文）が存在しない状態にする
3. The pasta_lsp shall `LangServerError` 型の各バリアントが実際に使用されていることを確認する
4. When デッドコードが検出された場合, the pasta_lsp shall 外部公開 API を変更せずにそれを除去する

### 要件 5: 冗長表現の削減

**目的:** メンテナとして、不必要に冗長な式・パターン・変換が簡素化され、コードの可読性と保守性が向上することを確認したい

#### 受入基準

1. The pasta_lsp shall 不要な `.clone()` 呼び出しを除去する
2. The pasta_lsp shall 冗長なパターンマッチ（ `if let Some(x) = ... { if let Some(y) = ...` のネスト等）を簡素化する
3. The pasta_lsp shall 重複するコード変換パターン（transport.rs の WASM 型変換等）を簡素化する
4. The pasta_lsp shall 外部振る舞い（LSP レスポンス内容）を変更しない
5. The pasta_lsp shall 既存テスト 11 ファイルすべてがパスする状態を維持する
