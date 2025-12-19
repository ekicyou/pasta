# Implementation Gap Analysis Report (Re-run)

## Document Information
- **Feature**: pasta-grammar-specification
- **Analysis Date**: 2025-12-19
- **Language**: ja
- **Current State**: 仕様側で「さくらスクリプトは字句のみ認識（非解釈）」「半角バックスラッシュ限定」「ブラケット内の `\]` 許容」を確定。Jump（`？`）は廃止して Call（`＞`）へ統一方針。

---

## 1. 現状把握（コード資産・構造）

- 文法実装: [src/parser/pasta.pest](src/parser/pasta.pest)
  - `sakura_escape = { "\\" | "＼" }`（全角バックスラッシュも許容）
  - `sakura_bracket_open/close = { "["|"［" }, { "]"|"］" }`（全角角括弧も許容）
  - ブラケット内容は `(!sakura_bracket_close ~ ANY)*` のため、`\]` を「内容」としては扱えず、`]` を必ずクローズとして扱う（エスケープ考慮なし）
  - `jump_marker = { "？" | "?" }`（Jump文をサポート）
- トランスパイラ: [src/transpiler/mod.rs](src/transpiler/mod.rs)
  - `Statement::Jump` 分岐、`pasta::jump()` ラッパーや `JumpTarget` 系ユーティリティが存在
- テスト群: [tests](tests)
  - Jump前提のテスト（例: [tests/phase3_test.rs](tests/phase3_test.rs), [tests/comprehensive_control_flow_test.rs](tests/comprehensive_control_flow_test.rs)）
  - さくらスクリプトの全角記号混在テスト（例: [tests/sakura_script_tests.rs](tests/sakura_script_tests.rs), [tests/engine_integration_test.rs](tests/engine_integration_test.rs)）
- ドキュメント/ステアリング: [GRAMMAR.md](GRAMMAR.md), [AGENTS.md](AGENTS.md), [.kiro/steering/*](.kiro/steering)
  - 仕様更新後の7章（さくら）・11.16（エスケープ/引用）に対し、pest とテストの不一致が発生

---

## 2. 要件対応可否（新決定の反映）

- **さくら（字句のみ・非解釈）**: pest 側は5パターンの詳細なコマンド形を持つが、仕様は「`\\` + ASCIIトークン + 任意の非ネスト`[...]`（`\]`は内容に含められる）」程度の最小字句規則。現状の実装は過剰なパターン化で、非解釈方針と乖離。
- **半角限定**: 仕様は「半角バックスラッシュのみ」「半角角括弧のみ」を推奨・限定。pest は全角も許容しており不一致。テストも全角混在を肯定している。
- **ブラケット内 `\]` 許容**: pest の `(!sakura_bracket_close ~ ANY)*` は直前の `\\` を考慮せず `]` をクローズ扱いするため、`\]` を内容として保持できない。明確なギャップ。
- **Jump 廃止（Call統一）**: pest・トランスパイラ・テストに Jump が広範囲に存在。仕様決定と実装の乖離大。

---

## 3. ギャップ一覧（Requirement-to-Asset Map）

- **Sakura 半角限定**: 実装（pest, tests）が全角も受理 → 仕様に合わせて半角限定へ変更が必要（Missing/Constraint）。
- **Sakura ブラケット内容のエスケープ**: `\]` を内容として許容する字句規則が未実装 → `("\\" ~ "]") | (!"]" ~ ANY)` 形への置換などが必要（Missing）。
- **Sakura 非解釈（字句最小化）**: pest の詳細パターン（digits only 等）を簡素化し、未知トークンも字句的に許容する設計へ（Option要検討）（Constraint）。
- **Jump 廃止**:
  - parser: `jump_marker`, `jump_content` 削除（Missing）。
  - transpiler: `Statement::Jump` 分岐、`pasta::jump()` ランタイム連携削除（Missing）。
  - tests/fixtures: `？` を `＞` へ置換しテストロジック再設計（Missing）。
- **GRAMMAR.md 同期**: 仕様（7章/11.16）の確定に合わせた説明・例の更新（Constraint/Unknown: 旧記述との整合確認要）。

---

## 4. 実装アプローチ（推奨: 層単位の段階的対応）

破壊的変更は**層単位**で段階的に行う必要があります。同一層内での部分的修正は避け、各層を完全に仕様対応させてから次層へ進みます。

- **Phase 1: Parser 層（src/parser/ 全体）の修正**
  - `pasta.pest` 全体の見直し（すべての破壊的変更を一括）:
    - `sakura_escape = { "\\" }` （半角のみ）
    - `sakura_bracket_open/close = { "[" }, { "]" }` （半角のみ）
    - ブラケット内容を `("\\" ~ "]") | (!"]" ~ ANY)*` に差し替え（`\]` 許容）
    - `sakura_command` を「ASCIIトークン + 任意の非ネスト`[...]`」へ簡素化（未知トークン許容）
    - `jump_marker` と `jump_content` を削除
  - AST 型（`ast.rs`）の修正:
    - `Statement` enum から `Jump` 分岐を削除
    - `JumpTarget` enum 不要時は削除
  - **成果物**: 新 pest 定義に対応した新 AST、ビルドが通ること
  - **検証**: Parser の単体テスト（`tests/parser_tests.rs` 等）が新定義で通ること

- **Phase 2: Transpiler 層（src/transpiler/ 全体）の修正**
  - Parser 出力（新 AST）に対応したトランスパイラの修正:
    - `Statement::Jump` 分岐削除
    - `pasta::jump()` ランタイム関数削除
    - `transpile_jump_target*` メソッド削除
    - 既存 `Statement::Call` との統合確認
  - **成果物**: Parser 新定義を受け取り、Rune IR を正しく生成するトランスパイラ
  - **検証**: トランスパイラの単体テスト・IR 出力が新仕様で正しいこと

- **Phase 3: Runtime/Test 層（src/runtime/, tests/ 全体）の修正**
  - テスト・フィクスチャの一括置換:
    - `tests/fixtures/*.pasta` の `？` を `＞` に置換
    - 全角 `＼`・`［］` を使用するテストケースを半角へ統一
    - Jump 検証ロジック削除/置換
  - テストコード修正:
    - `tests/parser_*.rs` 等: Jump 前提の検証を削除
    - `tests/engine_*.rs` 等: Jump 関連の期待を削除
  - GRAMMAR.md 改訂:
    - 7章（さくら）: 字句規則・半角限定・`\]` 対応を明記
    - 11.16: Jump 削除・Call 統一を反映
  - **成果物**: すべてのテストが新仕様で成功
  - **検証**: `cargo test --all` が 100% パス

- **トレードオフ**:
  - ✅ 各層で完全に仕様対応（混乱なし）
  - ✅ 依存関係に従った段階化（下層が完全なら上層は決定的）
  - ✅ 後戻りが最小化
  - ❌ 工数は大きい（複数ファイル・複数層）

---

## 5. 工数・リスク評価

- **Effort（層単位段階化）**
  - Phase 1 Parser 全体: L（5–7日）
  - Phase 2 Transpiler 全体: M（3–5日）
  - Phase 3 Runtime/Tests 全体: L（5–10日）
  - GRAMMAR.md 改訂: M（3–5日）
  - **合計**: 16–32 日

- **Risk**
  - 高: 各層での完全修正が必須（不完全だと次層で動作不可） → 緩和策: Phase 終了時に unit test で検証
  - 中: AST/Transpiler 型変更の波及 → 緩和策: Rust コンパイラが即座に検出
  - 低: 各 Phase は独立して進捗可能（並行作業の余地）

---

## 6. 推奨

- **層単位の段階的対応**を推奨します:
  - Phase 1: Parser 層を完全に新仕様へ対応
  - Phase 2: Transpiler 層を完全に新仕様へ対応
  - Phase 3: Runtime/Tests 層を完全に新仕様へ対応
- 各 Phase の終了時に「ビルド成功」「該当層の単体テスト 100% パス」を確認

---

## 7. 次アクション（設計フェーズへ引き継ぎ）

- **設計フェーズ（`/kiro-spec-design pasta-grammar-specification`）で**:
  - Phase 1 (Parser) の詳細タスク分解: pest ルール単位の修正案、AST 型変更リスト
  - Phase 2 (Transpiler) の詳細タスク分解: 削除対象メソッド・分岐、修正対象関数リスト
  - Phase 3 (Runtime/Tests) の詳細タスク分解: テスト・フィクスチャ置換ポリシー、修正テストファイル一覧
  - GRAMMAR.md 改訂の目次・主要セクション構成案
- **実装フェーズ（`/kiro-spec-impl pasta-grammar-specification`）で**:
  - Phase 1 → Phase 2 → Phase 3 の順に実装、各 Phase 終了時に検証
