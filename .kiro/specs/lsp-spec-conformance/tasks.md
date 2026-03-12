# 実装タスクリスト: lsp-spec-conformance

> **生成日**: 2026-03-12
> **要件**: 5 要件（R1〜R5）/ **設計**: 承認済み

---

- [ ] 1. (P) キューコマンド専用トークンタイプを LSP サーバーに追加する
  - TOKEN_TYPES 配列末尾に cueMarker（インデックス 15）と cueCommand（インデックス 16）を追加する
  - token_type 定数モジュールに `CUE_MARKER` と `CUE_COMMAND` 定数を定義する
  - 既存インデックス 0-14 は変更せず末尾追加のみとし、後方互換性を保証する
  - SemanticTokensLegend が新しいトークンタイプを含んで返すことを確認する
  - _Requirements: 2.1, 2.2, 2.3, 5.1, 5.2_

- [ ] 2. VSCode 拡張にキューコマンドの設定を追加する

- [ ] 2.1 (P) package.json にセマンティックトークンタイプとスコープマッピングを登録する
  - semanticTokenTypes に cueMarker（superType: "keyword"）を追記する
  - semanticTokenTypes に cueCommand（superType: "function"）を追記する
  - semanticTokenScopes に cueMarker → `keyword.other.marker.pasta` のマッピングを追記する
  - semanticTokenScopes に cueCommand → `entity.name.function.cue.pasta` のマッピングを追記する
  - _Requirements: 2.4, 5.1, 5.2_

- [ ] 2.2 (P) TextMate 文法にキューコマンド行パターンを追加する
  - repository に cue-command エントリを追加する（正規表現は設計書記載のものを使用、インデント部は `[\s\u3000]*`）
  - cue-arg-string / cue-arg-number / cue-arg-at-ref / cue-arg-ident のサブパターンをrepository に追加する
  - patterns 配列の `call` の直後・`actor` の直前に `cue-command` を挿入する（既存パターンの正規表現は変更しない）
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 5.3, 5.4_

- [ ] 3. キューコマンド行のセマンティックトークン生成を実装する
  - タスク 1 完了後に着手（CUE_MARKER / CUE_COMMAND 定数が必要）

- [ ] 3.1 マーカーとコマンド名のトークン生成を実装する
  - `visit_local_scene_item` 内の CueCommand アーム（行全体 OPERATOR 1 トークン）を `visit_cue_command` への委譲に置き換える
  - `visit_cue_command` を新設し、span テキストから全角（！）・半角（!）マーカーを全角優先で検出して CUE_MARKER トークンを生成する
  - `cue.command` 文字列をテキストスキャンで検出し CUE_COMMAND トークンを生成する
  - _Requirements: 1.1, 1.7, 2.1, 2.2_

- [ ] 3.2 スコープのトークン生成を実装する
  - `cue.scope` が Some の場合、ScopedName.span をそのまま WORD トークンとして emit する（`@` を含む全体を 1 トークン。ActionLine の WordRef と同方針）
  - テキストスキャンは不要（Span 直接利用）
  - _Requirements: 1.2, 1.3_

- [ ] 3.3 引数リストのトークン生成を実装する
  - `cue.args` が非空の場合、開き括弧（`(` / `（`）を検出して OPERATOR トークンを生成する
  - `arg_cursor` による前進スキャン方式（tokenize_args_text と同一方針）で各引数のテキスト範囲を確定する
  - CueArgToken::Ident → CUE_COMMAND、StringLiteral → TALK、Integer/Float → NUMBER（find_number_literal 使用）、AtRef → WORD のトークンをそれぞれ生成する
  - 複数の同値引数（例: `!cmd(1, 1, 1)`）を引数スライスに絞って検索することで衝突なく位置特定できることを確認する
  - 閉じ括弧（`)` / `）`）を検出して OPERATOR トークンを生成する
  - _Requirements: 1.3, 1.4, 1.5, 1.6_

- [ ] 4. 統合テストを追加してキューコマンドのトークン生成を検証する

- [ ] 4.1 キューコマンドの基本 4 形式のトークン生成をテストする
  - `!id` 形式: マーカー + コマンド名の 2 トークンを検証する
  - `!id@scope` 形式: マーカー + コマンド名 + スコープ（@含む全体）の 3 トークンを検証する
  - `!id(args)` 形式: マーカー + コマンド名 + 開き括弧 + 各引数 + 閉じ括弧のトークン列を検証する
  - `!id@scope(args)` 形式: 全構成要素のトークンを検証する
  - 全角マーカー（`！id`）が半角（`!id`）と同一トークンタイプ・同一トークン数を生成することを検証する
  - _Requirements: 4.1, 4.2_

- [ ] 4.2 引数タイプ別のトークン生成をテストする
  - 文字列リテラル引数（`「こんにちは」`形式）→ TALK トークンであることを検証する
  - 整数・浮動小数点引数（`10`, `10.0`）→ NUMBER トークンであることを検証する
  - `@参照` 引数 → WORD トークンであることを検証する
  - _Requirements: 4.1_

- [ ] 4.3 混在ドキュメントと Diagnostics のテストを追加する
  - キューコマンド行とアクション行が混在するシーンで正確なトークン生成を検証する
  - 引数括弧の不一致（`!cmd(unclosed`）で Diagnostics エラーが報告されることを検証する
  - _Requirements: 4.3, 4.4_

- [ ] 4.4 リグレッションがないことを確認する
  - `cargo test -p pasta_lsp` を実行し既存すべてのテストがパスすることを確認する
  - 新規テスト追加後も合計テスト数が正しく増加していることを確認する
  - _Requirements: 4.5, 5.1, 5.2, 5.3, 5.4_
