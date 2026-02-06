# Implementation Plan

## タスク概要

本実装は、Pasta DSLのさくらスクリプトタグ文字クラスに5文字（`-`, `+`, `*`, `?`, `&`）を追加し、ukadoc公式タグリストに記載された全さくらスクリプトタグを認識可能にする。4箇所の文字クラス定義を同期的に更新し、テストで検証する。

**変更範囲**: 仕様書2ファイル、Pest文法1ファイル、ランタイムregex1ファイル + テストファイル2件

**並列実行可能性**: 仕様書更新（Task 1）とコード実装（Task 2, 3）は独立しており、並列実行可能。テスト（Task 4, 5）はコード実装完了後に実行。

---

## 実装タスク

- [x] 1 (P). 仕様書の sakura_token 文字クラス定義を5文字追加
- [x] 1.1 (P) doc/spec/07-sakura-script.md の sakura_token 定義を更新
  - `sakura_token ::= [!_a-zA-Z0-9]+` → `sakura_token ::= [!\-+*?&_a-zA-Z0-9]+`
  - 説明文を更新: 「ASCII 英数字 + `_` + `!`」→「ASCII 英数字 + `_` + `!` + `-` + `+` + `*` + `?` + `&`」
  - 同期箇所一覧を追記: `sakura_token` 文字クラス変更時に更新すべき4箇所を明記（§7.3の末尾に追加）
  - _Requirements: 1.1, 1.2, 1.3, 4.3_

- [x] 1.2 (P) GRAMMAR.md の sakura_token 定義を更新
  - L508付近の `sakura_token ::= [!_a-zA-Z0-9]+` → `sakura_token ::= [!\-+*?&_a-zA-Z0-9]+`
  - 07-sakura-script.md との一貫性を保つ
  - _Requirements: 1.1, 1.2, 1.3, 4.1_

- [x] 2 (P). Pest文法定義の sakura_id ルールを5文字追加
- [x] 2.1 (P) grammar.pest の sakura_id ルールを更新
  - L171の `sakura_id = @{ (('a'..'z') | ('A'..'Z') | ('0'..'9') | "_" | "!" )+ }` に5文字を追加
  - 変更後: `sakura_id = @{ (('a'..'z') | ('A'..'Z') | ('0'..'9') | "_" | "!" | "-" | "+" | "*" | "?" | "&" )+ }`
  - Pest文法では各文字を `"x"` リテラルで個別に追加（正規表現の文字クラスとは異なる記法）
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 3 (P). ランタイムregexの SAKURA_TAG_PATTERN を5文字追加
- [x] 3.1 (P) tokenizer.rs の SAKURA_TAG_PATTERN を更新
  - L113の `const SAKURA_TAG_PATTERN: &'static str = r"\\[0-9a-zA-Z_!]+(?:\[[^\]]*\])?";` を更新
  - 変更後: `const SAKURA_TAG_PATTERN: &'static str = r"\\[0-9a-zA-Z_!+*?&-]+(?:\[[^\]]*\])?";`
  - `-` を文字クラス末尾に配置（範囲指定と誤解されないため）
  - `+`, `*`, `?` は文字クラス内でリテラル扱いのためエスケープ不要
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 4. pasta_lua ランタイムレイヤーのトークナイズテストを追加
- [x] 4.1 tokenizer.rs 内テストに5文字タグのテストケースを追加
  - `#[cfg(test)] mod tests` に6テスト関数を追加:
    - `test_tokenize_symbol_tag_hyphen`: `\-` → `TokenKind::SakuraScript`, `text == r"\-"`
    - `test_tokenize_symbol_tag_plus`: `\+` → `TokenKind::SakuraScript`, `text == r"\+"`
    - `test_tokenize_symbol_tag_asterisk`: `\*` → `TokenKind::SakuraScript`, `text == r"\*"`
    - `test_tokenize_symbol_tag_underscore_question`: `\_?` → `TokenKind::SakuraScript`, `text == r"\_?"`
    - `test_tokenize_symbol_tag_ampersand`: `\&[ID]` → `TokenKind::SakuraScript`, `text == r"\&[ID]"`
    - `test_tokenize_symbol_tag_mixed_text`: `こんにちは\-。` → 3要素分割（General×5 + SakuraScript + Period）
  - 既存の `test_tokenize_sakura_script_tag`, `test_tokenize_complex_tag` がリグレッションなく動作することを確認
  - _Requirements: 3.1, 3.2, 3.4, 4.2_

- [x] 5. pasta_core パーサーレイヤーのパーステストを追加
- [x] 5.1 新規ファイル sakura_symbol_tag_test.rs を作成
  - `crates/pasta_core/tests/sakura_symbol_tag_test.rs` を作成
  - ファイル配置理由: 5文字すべての体系的テストケースを1ファイルに集約し、将来の文字クラス拡張時の参照性を高める
  - 7テスト関数を実装:
    - `test_parse_sakura_hyphen_tag`: `＊test\nAlice：\-` → `Action::SakuraScript` で `script == r"\-"`
    - `test_parse_sakura_plus_tag`: `＊test\nAlice：\+` → `Action::SakuraScript` で `script == r"\+"`
    - `test_parse_sakura_asterisk_tag`: `＊test\nAlice：\*` → `Action::SakuraScript` で `script == r"\*"`
    - `test_parse_sakura_underscore_question_tag`: `＊test\nAlice：\_?` → `Action::SakuraScript` で `script == r"\_?"`
    - `test_parse_sakura_ampersand_tag`: `＊test\nAlice：\&[entity]` → `Action::SakuraScript` で `script == r"\&[entity]"`
    - `test_parse_sakura_symbol_in_mixed_text`: `＊test\nAlice：こんにちは\-。` → talk + sakura_script + talk の3要素分割
    - `test_parse_existing_tags_no_regression`: `＊test\nAlice：\h\s[0]\_w[500]` → 既存タグが正常にパースされる
  - `parse_str()` APIを使用し、`PastaFile` → `FileItem::GlobalSceneScope` → `Action` の検証パターンに準拠
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 4.2_

- [x] 6. 全テスト実行と4箇所の一貫性検証
- [x] 6.1 ワークスペース全体のテストを実行
  - `cargo test --workspace` を実行し、既存テスト340+件すべてがパスすることを確認
  - 新規追加テスト13件（tokenizer内6件 + パーサー7件）がすべてパスすることを確認
  - リグレッションなしを確認（特に `sakura_script_integration_test.rs` の既存644行のテスト群）
  - _Requirements: 2.4, 3.4, 4.1, 4.2_

- [x] 6.2 4箇所の文字クラス同期を手動検証
  - `doc/spec/07-sakura-script.md`: `[!\-+*?&_a-zA-Z0-9]+`
  - `GRAMMAR.md` L508: `[!\-+*?&_a-zA-Z0-9]+`
  - `grammar.pest` L171: `"_" | "!" | "-" | "+" | "*" | "?" | "&"` の全文字が含まれる
  - `tokenizer.rs` L113: `[0-9a-zA-Z_!+*?&-]` (ハイフン末尾)
  - 4箇所で同一の文字セット（`!`, `_`, `-`, `+`, `*`, `?`, `&`, `a-z`, `A-Z`, `0-9`）を許容することを確認
  - _Requirements: 4.1_

- [x] 7. ドキュメント整合性の確認と更新
- [x] 7.1 SOUL.md との整合性確認
  - コアバリュー（日本語フレンドリー、UNICODE識別子、yield型、宣言的フロー）への影響: なし
  - 設計原則（行指向文法、前方一致、UI独立性）への影響: なし
  - 影響なしを確認済みとして記録
  - _Requirements: すべて_

- [x] 7.2 TEST_COVERAGE.md への新規テストマッピング追加
  - 新規テストファイル `sakura_symbol_tag_test.rs`（7テスト）を追加
  - `tokenizer.rs` 内テスト（6テスト）を追加
  - テストカバレッジマップを更新
  - _Requirements: 4.2_

---

## タスク完了基準（DoD）

すべてのタスクが完了した時点で以下を満たすこと：

1. **Spec Gate**: 全要件（1.1-1.3, 2.1-2.4, 3.1-3.4, 4.1-4.3）をタスクでカバー
2. **Test Gate**: `cargo test --workspace` が成功（既存340+件 + 新規13件）
3. **Doc Gate**: 仕様書2件（07-sakura-script.md, GRAMMAR.md）更新済み
4. **Steering Gate**: tech.md（依存関係なし）、structure.md（テストファイル配置）と整合
5. **Soul Gate**: SOUL.md との整合性確認済み（Task 7.1）

---

## 要件カバレッジマトリクス

| Requirement | タスク | 説明 |
|-------------|-------|------|
| 1.1, 1.2, 1.3 | 1.1, 1.2 | 仕様書・文法リファレンスのsakura_token定義更新 |
| 2.1, 2.2, 2.3, 2.4 | 2.1, 5.1 | Pest文法のsakura_id更新とパーステスト |
| 3.1, 3.2, 3.3, 3.4 | 3.1, 4.1 | ランタイムregex更新とトークナイズテスト |
| 4.1 | 6.2 | 4箇所の文字クラス一貫性検証 |
| 4.2 | 4.1, 5.1, 6.1 | テストケースの網羅 |
| 4.3 | 1.1 | 同期箇所一覧のドキュメント化 |

**全要件カバー済み**: 14要件すべて（1.1-1.3, 2.1-2.4, 3.1-3.4, 4.1-4.3）がタスクにマッピング済み
