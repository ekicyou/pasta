# 実装タスク

## タスク一覧

- [x] 1. Span 構造体の拡張
- [x] 1.1 (P) Span にバイトオフセットフィールドを追加
  - `start_byte: usize`, `end_byte: usize` フィールドを Span 構造体に追加
  - `Span::new()` コンストラクタを 6 引数（start_line, start_col, end_line, end_col, start_byte, end_byte）に拡張
  - `Default` trait 実装で全フィールドを 0 に初期化
  - _Requirements: 1, 6_
  - _Contracts: Span Service Interface_

- [x] 1.2 (P) From トレイトで Pest 統合
  - `From<&Pair<Rule>> for Span` トレイトを実装
  - `pest_span.start()`, `pest_span.end()` でバイトオフセットを取得
  - `pest_span.start_pos().line_col()`, `pest_span.end_pos().line_col()` で行/列を取得
  - 6 引数の `Span::new()` を呼び出して Span インスタンスを生成
  - _Requirements: 2_
  - _Contracts: From Trait_

- [x] 1.3 (P) ソース参照 API の実装
  - `Span::extract_source(&self, source: &str) -> Result<&str, SpanError>` メソッドを実装
  - バイトオフセット範囲検証（`end_byte > source.len()` でエラー）
  - UTF-8 文字境界検証（`source.is_char_boundary()` でチェック）
  - `Span::is_valid()`, `Span::byte_len()` ヘルパーメソッドを実装
  - SpanError enum（OutOfBounds, InvalidUtf8Boundary, InvalidSpan）を定義
  - _Requirements: 4, 6_
  - _Contracts: Span Service Interface_

- [x] 2. パーサー層のリファクタリング
- [x] 2.1 既存 Span 呼び出し箇所の修正（pasta_core）
  - パーサー層の `Span::new()` 呼び出し（推定 10+ 箇所）を 6 引数版に更新
  - テスト用 Span 生成箇所で `start_byte: 0, end_byte: 0` を指定
  - 実装用 Span 生成箇所で `Span::from(&pair)` を使用
  - `Span::default()` 使用箇所（28 箇所）は変更不要
  - _Requirements: 1, 2, 6_

- [x] 3. Action enum のリファクタリング
- [x] 3.1 Action enum を named fields 化
  - `Action::Talk(String)` → `Action::Talk { text: String, span: Span }` に変更
  - `Action::WordRef(String)` → `Action::WordRef { name: String, span: Span }` に変更
  - `Action::SakuraScript(String)` → `Action::SakuraScript { script: String, span: Span }` に変更
  - `Action::Escape(String)` → `Action::Escape { sequence: String, span: Span }` に変更
  - `Action::VarRef { name, scope }` → `Action::VarRef { name, scope, span }` に変更
  - `Action::FnCall { name, args, scope }` → `Action::FnCall { name, args, scope, span }` に変更
  - _Requirements: 3.2_
  - _Contracts: Action enum definition_

- [x] 3.2 パーサー層での Action 生成修正（pasta_core）
  - Action 生成箇所（推定 20+ 箇所）で named fields 形式に変更
  - `Span::from(&pair)` で Action の span フィールドを設定
  - パターンマッチ箇所でフィールド名を明示的に指定
  - _Requirements: 3.2, 7_

- [x] 4. トランスパイラ層のリファクタリング（pasta_rune）
- [x] 4.1 pasta_rune トランスパイラの Action パターンマッチ修正
  - `CodeGenerator::generate_action()` で Action パターンマッチを named fields 形式に変更（推定 6 箇所）
  - `match action { Action::Talk { text, span } => ... }` 形式に統一
  - span フィールドからソース位置情報を取得可能に
  - _Requirements: 3.2, 7_

- [x] 4.2 pasta_rune テストの Action 生成修正
  - テストコード内の Action 生成箇所（推定 10+ 箇所）を named fields 形式に変更
  - テスト用 Span は `Span::default()` または `Span::new(1, 1, 1, 1, 0, 0)` を使用
  - _Requirements: 3.2, 6_

- [x] 5. トランスパイラ層のリファクタリング（pasta_lua）
- [x] 5.1 pasta_lua トランスパイラの Action パターンマッチ修正
  - `CodeGenerator::generate_action()` で Action パターンマッチを named fields 形式に変更（推定 6 箇所）
  - `match action { Action::Talk { text, span } => ... }` 形式に統一
  - span フィールドからソース位置情報を取得可能に
  - _Requirements: 3.2, 7_

- [x] 5.2 pasta_lua テストの Action 生成修正
  - テストコード内の Action 生成箇所（推定 10+ 箇所）を named fields 形式に変更
  - テスト用 Span は `Span::default()` または `Span::new(1, 1, 1, 1, 0, 0)` を使用
  - _Requirements: 3.2, 6_

- [x] 6. テストカバレッジの拡充
- [x] 6.1 (P) Span 構造体のユニットテスト
  - `Span::new()` が 6 フィールドを正しく設定することを確認
  - `Span::from(&pair)` が Pest から全情報を抽出することを確認
  - `Span::extract_source()` が正確なスライスを返すことを確認
  - `Span::is_valid()` が default Span を無効と判定することを確認
  - SpanError の各バリアント（OutOfBounds, InvalidUtf8Boundary, InvalidSpan）をテスト
  - _Requirements: 1, 4, 5_

- [x] 6.2 (P) パーサー統合テストの拡張
  - ASCII スクリプトでバイトオフセット検証を追加
  - UTF-8 マルチバイト文字（日本語 "こんにちは"、絵文字 "👋🌍"）でオフセット正確性を確認
  - ネスト構造（シーン内アクション）での Span 伝播確認
  - エッジケース（空行、長行）のテストを追加
  - _Requirements: 2, 5_

- [x] 6.3 (P) Action Span 統合テスト
  - ActionLine から Action の span フィールドにアクセス可能なことを確認
  - 各 Action バリアントが正しい Span を保持していることを確認
  - トランスパイラが Action の span 情報を取得できることを確認
  - _Requirements: 3.2, 7_

- [x] 7. 統合検証とリグレッション防止
- [x] 7.1 既存テストの全件実行と修正
  - `cargo test --all` で全テストスイート実行
  - コンパイルエラーとなったテスト箇所を修正
  - 既存のパーサー・トランスパイラテストが引き続き合格することを確認
  - _Requirements: 6_

- [x] 7.2 (P) エラー報告での Span 情報確認
  - パースエラー時に Span 情報が含まれることを確認
  - トランスパイルエラー時に Span 情報が含まれることを確認
  - エラーメッセージが行/列とバイトオフセットの両方を表示することを確認（オプション）
  - _Requirements: 6_

## タスク実行順序

### フェーズ 1: 基盤拡張（並行可能）
- タスク 1.1, 1.2, 1.3 は独立して実行可能

### フェーズ 2: パーサー層修正
- タスク 2.1（フェーズ 1 完了後）

### フェーズ 3: Action リファクタリング
- タスク 3.1 → 3.2（順次実行）

### フェーズ 4: トランスパイラ層修正（並行可能）
- タスク 4.1, 4.2（pasta_rune）と タスク 5.1, 5.2（pasta_lua）は並行実行可能

### フェーズ 5: テストと検証（並行可能）
- タスク 6.1, 6.2, 6.3 は並行実行可能
- タスク 7.1（全タスク完了後）
- タスク 7.2（並行可能）

## 推定作業時間

- **フェーズ 1**: 3-4 時間
- **フェーズ 2**: 1-2 時間
- **フェーズ 3**: 2-3 時間
- **フェーズ 4**: 4-6 時間（両トランスパイラ）
- **フェーズ 5**: 3-4 時間
- **合計**: 13-19 時間
