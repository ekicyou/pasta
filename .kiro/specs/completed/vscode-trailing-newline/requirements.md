# Requirements Document

## Project Description (Input)
pasta vscodeの表示だが、pasta DSLにおいて、ファイル末尾に空行が無い場合、すべての行で警告が出る。これは実際のpasta DSL読み込みでもエラーになるのか？pasta DSL読み込み時は自動的に末尾空行を入れてる？もし、ファイル読み込み時に空空行を入れるのがパーサーのデフォルト挙動になっているのなら、pasta_vscode側でも同じ挙動になるようにしてほしい。pasta_luaにおいて実際にエラーになるなら、末尾空行を自動挿入する方がよいかもしれない。挙動確認をお願いします。

## 調査結果

### 根本原因
Pest文法 (`grammar.pest`) の `eol` ルールが `NEWLINE` のみを受理し、EOFを代替として許容しない。
そのため、ファイル末尾に改行がない場合、最終行の行末ルール (`eol` / `or_comment_eol`) がマッチせず、full parseが失敗する。

### 影響範囲
| コンポーネント | 末尾改行の自動追加 | 末尾改行なし時の挙動 |
|---|---|---|
| pasta_dsl (Pest文法) | — | `eol = _{ NEWLINE }` でEOF不許容 → パースエラー |
| pasta_lua loader | なし | `fs::read_to_string()` → そのまま `parse_str()` → エラー、ファイルスキップ |
| pasta_lsp (analysis) | なし | `parse_str()` 失敗 → `parse_str_partial()` フォールバック → 各行も失敗 → 全行エラー |
| VSCode extension | なし | `document.getText()` → そのまま WASM → 全行にDiagnosticエラー表示 |

### パーサーフォールバックの連鎖失敗メカニズム
1. `parse_str()` (full parse) が失敗 — 最終行に `eol` (NEWLINE) がないため
2. `parse_str_partial()` が起動 — スコープ境界で分割
3. 各チャンクを `try_parse_chunk()` で再パース — `current_lines.join("\n")` で結合するが末尾に `\n` を追加しないため同様に失敗
4. 行単位フォールバック — 各行を個別に `parse_str()` するが、やはり末尾改行なしで失敗
5. 結果: すべての非空行がエラーとして報告される

## Introduction
pasta DSLファイルの末尾改行欠落に対する堅牢性を改善する。文法レベルでEOFを行末として許容し、全コンポーネントで一貫した挙動を実現する。

## Requirements

### Requirement 1: 文法レベルでのEOF許容
**Objective:** DSL作者として、ファイル末尾に改行がなくてもパースが成功してほしい。テキストエディタの設定に依存せず安定動作させるため。

#### Acceptance Criteria
1. When pasta DSLファイルの最終行が改行なしで終了する, the pasta_dsl parser shall 最終行を正常にパースし改行ありの場合と同一のASTを生成する
2. When `eol` ルールが行末を評価する, the pasta_dsl parser shall `NEWLINE` またはEOF（入力の終端）のいずれかをマッチさせる
3. The pasta_dsl parser shall 末尾改行の有無にかかわらず、同一内容のファイルに対して同一のAST構造を出力する

### Requirement 2: partial parserのEOF対応
**Objective:** LSP利用者として、末尾改行のないファイルでも正確なセマンティックトークンと最小限のDiagnosticsを得たい。編集中のファイルで不要なエラー表示を避けるため。

#### Acceptance Criteria
1. When 末尾改行のないソースが `parse_str_partial()` に渡される, the partial parser shall full parseと同等の結果を返す（文法修正により自動的に解決される見込み）
2. When チャンク分割後の各チャンクが末尾改行を持たない, the partial parser shall 正常にパースし不要なパースエラーを生成しない

### Requirement 3: pasta_lua loaderの一貫性
**Objective:** ゴースト開発者として、末尾改行のない.pastaファイルも正常にロード・トランスパイルしてほしい。ファイル保存時のエディタ設定を気にせず開発するため。

#### Acceptance Criteria
1. When pasta_lua loaderが末尾改行のない.pastaファイルを読み込む, the loader shall ファイルを正常にパース・トランスパイルする（文法修正により自動的に解決される見込み）
2. If pasta_lua loaderがパースエラーを検出する, the loader shall 末尾改行の欠落以外の真のエラーのみを報告する

### Requirement 4: VSCode拡張のDiagnostics正確性
**Objective:** VSCode利用者として、末尾改行のないファイルで全行にエラーが表示される誤報を解消してほしい。実際のDSL文法エラーだけを確認するため。

#### Acceptance Criteria
1. When 末尾改行のないpasta DSLファイルがVSCodeで開かれる, the VSCode extension shall 不要なDiagnosticsエラーを表示しない
2. When ファイル内容に真のDSL文法エラーが含まれる, the VSCode extension shall 該当箇所のみにエラーDiagnosticsを表示する

### Requirement 5: 既存テストとの互換性
**Objective:** 開発者として、既存のテストスイート（950+テスト）がすべてパスし続けることを保証したい。リグレッション防止のため。

#### Acceptance Criteria
1. The pasta_dsl parser shall 既存の全テストケースを変更なしでパスする
2. When 末尾改行あり/なし両方のテストケースが実行される, the pasta_dsl parser shall どちらも正常にパースする
