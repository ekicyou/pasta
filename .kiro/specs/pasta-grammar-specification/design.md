# 設計ドキュメント

## プロジェクト説明（入力）

Pasta DSL の文法仕様を、現在の実装（pest定義）と乖離から修正し直す必要があります。特に：
- さくらスクリプトは**字句のみ認識**（非解釈）、**半角限定**
- ブラケット内での `\]` **許容**
- **Jump 文（`？`）廃止**、Call（`＞`）へ統一

この破壊的変更を層単位の段階化で実装し、リグレッション発生箇所を特定しやすくします。

---

## 設計概要

### 全体戦略: 層単位の段階的実装 + テスト層別化

1. **Phase 0（Pre-Implementation Preparation）**: テスト層別化・グリーン確認・commit
   - テストファイルを層別命名規則に統一
   - 全テストが通ることを確認し、git に保存
   - リグレッション検出の基盤を確立

2. **Phase 1（Parser 層）**: pest + AST を新仕様へ統一
   - pest.pest の全ルール修正（さくら・Jump）
   - AST 型修正（Jump 削除）
   - Parser テストが 100% パス

3. **Phase 2（Transpiler 層）**: Parser 出力に対応
   - Statement::Jump 削除
   - pasta::jump() 関数削除
   - Transpiler テストが 100% パス

4. **Phase 3（Runtime/Tests 層）**: 統合テスト・ドキュメント修正
   - テスト・フィクスチャ置換（`？` → `＞`、全角 → 半角）
   - GRAMMAR.md 改訂
   - 全テストが 100% パス

---

## Phase 0: Pre-Implementation Preparation

### 目的
リグレッション発生箇所を層別に特定しやすくするため、テストファイルを命名規則で統一し、変更前の「グリーン状態」を git で保存。

### 実施内容

#### 0.1 テスト層別化・リネーム

**命名規則**:
```
pasta_parser_<name>_test.rs          — Parser 層テスト
pasta_transpiler_<name>_test.rs      — Transpiler 層テスト
pasta_engine_<name>_test.rs          — Engine/Runtime 層テスト
pasta_integration_<name>_test.rs     — 統合テスト
pasta_debug_<name>_test.rs           — デバッグ用（オプション）
```

**リネーム対象ファイル（約38ファイル）** → [test-hierarchy-plan.md](test-hierarchy-plan.md) に詳細記載

#### 0.2 事前グリーン確認

```bash
cargo test --all 2>&1 | tee .kiro/specs/pasta-grammar-specification/test-baseline.log
# 全テスト "test result: ok" を確認
```

#### 0.3 Commit（グリーン状態の記録）

```bash
git add tests/*.rs
git commit -m "Refactor: Organize test files by layer hierarchy for regression tracking (Phase 0)"
```

### 成果物
- リネーム済みテストファイル
- test-baseline.log
- Git commit ID（グリーン状態の参照点）

### 検証方法
- `cargo test --all` が 100% パス
- `git log --oneline | head -1` でコミット確認

---

## Phase 1: Parser 層修正

### 目的
pest 定義と AST 型を正規仕様（grammar-specification.md）に統一。Parser テストが 100% パス。

### 修正項目一覧（grammar-specification.md 基準）

| ID | カテゴリ | 仕様箇所 | 修正内容 |
|----|---------|---------|---------|
| A1 | Sakura | 7.2 | `sakura_escape` 半角のみ |
| A2 | Sakura | 7.3 | `sakura_command` 簡素化（5パターン → 単純形） |
| A3 | Sakura | 7.3 | `bracket_content` に `\]` 許容 |
| A4 | Sakura | 7.4 | `sakura_bracket_open/close` 半角のみ |
| A5 | Sakura | 7.4 | `sakura_letter` ルール削除（不要） |
| A6 | Sakura | 7.4 | `sakura_digit` ルール削除（不要） |
| A7 | Sakura | 7.4 | `sakura_underscore` ルール削除（不要） |
| B1 | Jump | 2.4 | `jump_marker` ルール削除 |
| B2 | Jump | — | `jump_content` ルール削除 |
| B3 | Jump | — | `label_body_line` から jump 分岐削除 |
| B4 | Jump | — | `local_label_body_line` から jump 分岐削除 |
| C1 | text_part | 6.3 | `dollar_marker` を除外対象に追加 |
| D1 | 行継続 | 6.5.3 | 継続行内空行の対応（設計判断） |

### 実施内容

#### 1.1 pasta.pest の修正

##### 1.1.1 Sakura スクリプト関連（A1-A7）

**変更前**:
```pest
sakura_escape = { "\\" | "＼" }
sakura_bracket_open = { "[" | "［" }
sakura_bracket_close = { "]" | "］" }
sakura_letter = { ASCII_ALPHA | '\u{FF41}'..'\u{FF5A}' | '\u{FF21}'..'\u{FF3A}' }
sakura_digit = { ASCII_DIGIT | '\u{FF10}'..'\u{FF19}' }
sakura_underscore = { "_" | "＿" }

sakura_command = @{
    sakura_underscore ~ sakura_letter+ ~ (sakura_bracket_open ~ (!sakura_bracket_close ~ ANY)* ~ sakura_bracket_close)? |
    ("!" | "！" | sakura_letter+) ~ sakura_bracket_open ~ (!sakura_bracket_close ~ ANY)* ~ sakura_bracket_close |
    sakura_letter ~ sakura_digit+ ~ !sakura_letter |
    sakura_letter |
    sakura_digit+
}
```

**変更後（grammar-specification.md 7.3 準拠）**:
```pest
// Sakura スクリプト（7.2-7.4 準拠）
// エスケープは厳密に半角バックスラッシュのみ
sakura_escape = { "\\" }

// 括弧は半角のみ
sakura_bracket_open = { "[" }
sakura_bracket_close = { "]" }

// コマンドの字句構造（7.3 簡略版）
// sakura_token ::= [!_a-zA-Z0-9]+
// bracket_content ::= "[" ~ bracket_chars ~ "]"
// bracket_chars ::= ( "\\]" | [^\]] )*
sakura_command = @{
    sakura_token ~ sakura_bracket_content?
}

sakura_token = @{
    (ASCII_ALPHA | ASCII_DIGIT | "_" | "!")+
}

sakura_bracket_content = {
    sakura_bracket_open ~ sakura_bracket_chars ~ sakura_bracket_close
}

sakura_bracket_chars = @{
    (("\\" ~ "]") | (!"]" ~ ANY))*
}

// 以下のルールは削除（A5-A7）:
// - sakura_letter（全角英字含む → 不要）
// - sakura_digit（全角数字含む → 不要）
// - sakura_underscore（全角アンダースコア含む → 不要）
```

**理由**:
- 仕様 7.2: エスケープは半角 `\` のみ
- 仕様 7.3: `sakura_token = [!_a-zA-Z0-9]+`（ASCII のみ）
- 仕様 7.3: `bracket_chars = ( "\\]" | [^\]] )*`（`\]` 許容）
- 仕様 7.4: 括弧は半角のみ

##### 1.1.2 Jump 削除（B1-B4）

**削除対象ルール**:
```pest
// B1: 削除
jump_marker = { "？" | "?" }

// B2: 削除
jump_content = { jump_target ~ filter_list? ~ arg_list? ~ NEWLINE }
```

**修正対象ルール**:
```pest
// B3: label_body_line から jump 分岐削除
label_body_line = {
    indent ~ (
        comment_marker ~ comment_content ~ NEWLINE |
        rune_start ~ rune_block_content |
        at_marker ~ word_def_content |
        amp_marker ~ attribute_content |
        dollar_marker ~ var_assign_content |
        local_label_marker ~ local_label_content |
        call_marker ~ call_content |
        // jump_marker ~ jump_content |  ← 削除
        speech_line_content
    )
}

// B4: local_label_body_line から jump 分岐削除
local_label_body_line = {
    indent ~ (
        comment_marker ~ comment_content ~ NEWLINE |
        rune_start ~ rune_block_content |
        at_marker ~ word_def_content |
        amp_marker ~ attribute_content |
        dollar_marker ~ var_assign_content |
        call_marker ~ call_content |
        // jump_marker ~ jump_content |  ← 削除
        speech_line_content
    )
}
```

##### 1.1.3 text_part バグ修正（C1）

**変更前**:
```pest
text_part = @{ (!(at_marker | sakura_escape | NEWLINE) ~ ANY)+ }
```

**変更後（6.3 インライン要素準拠）**:
```pest
// 6.3: インライン要素（＠、＄、\）で分岐するため、これらを除外
text_part = @{ (!(at_marker | dollar_marker | sakura_escape | NEWLINE) ~ ANY)+ }
```

**理由**: 仕様 6.3 では `＄var_name` が変数参照としてインライン要素。現行 pest では `＄` が text_part に吸収されるバグ。

##### 1.1.4 行継続内空行（D1）

**仕様 6.5.3**: 継続行内のインデントのみの行は改行として解釈

**設計判断**: pest レベルでは現行維持（`continuation_line` は `speech_content` 必須）。空行の改行解釈は **AST 構築時（mod.rs）または Transpiler** で対応。

**理由**:
- pest で空行を許容すると `speech_content` が空になり、構文的に曖昧になる
- AST レベルで `SpeechPart::Newline` を挿入する方が明確

**チェックリスト（1.1）**:
- [ ] A1: `sakura_escape` を `{ "\\" }` に変更
- [ ] A2: `sakura_command` を簡素化（`sakura_token ~ sakura_bracket_content?`）
- [ ] A3: `sakura_bracket_chars` に `\]` 許容ルール追加
- [ ] A4: `sakura_bracket_open/close` を半角のみに変更
- [ ] A5-A7: `sakura_letter`, `sakura_digit`, `sakura_underscore` ルール削除
- [ ] B1: `jump_marker` ルール削除
- [ ] B2: `jump_content` ルール削除
- [ ] B3: `label_body_line` から `jump_marker ~ jump_content` 削除
- [ ] B4: `local_label_body_line` から `jump_marker ~ jump_content` 削除
- [ ] C1: `text_part` に `dollar_marker` 除外追加
- [ ] ビルド確認: `cargo build`

#### 1.2 AST 型（src/parser/ast.rs）の修正

**対象**:
```rust
pub enum Statement {
    Speech { ... },
    VarAssign { ... },
    Call { ... },
    Jump { target: JumpTarget, ... },  // ← 削除対象
    RuneBlock { ... },
    WordDef { ... },
    ...
}

pub enum JumpTarget {
    Local(String),
    Global(String),
    LongJump { global: String, local: String },
    Dynamic(String),
}  // ← Jump がなければ、削除または統合を検討
```

**修正案**:
```rust
pub enum Statement {
    Speech { ... },
    VarAssign { ... },
    Call { ... },  // Jump を統一
    RuneBlock { ... },
    WordDef { ... },
    ...
}

// JumpTarget は Call で使用されている場合は維持
// Jump 専用なら削除
```

**チェックリスト（1.2）**:
- [ ] Statement enum から `Jump` 分岐削除
- [ ] `JumpTarget` enum の使用箇所確認（Call で使用なら維持）
- [ ] 関連する impl ブロック修正
- [ ] コンパイル確認: `cargo build`

#### 1.3 Parser 実装（src/parser/mod.rs）の修正

**対象**: AST 構築ロジックから Jump 関連コード削除

**修正箇所**:
```rust
// 削除対象
Rule::jump_content => { ... }
Rule::jump_marker => { ... }

// Statement::Jump 構築コード削除
```

**チェックリスト（1.3）**:
- [ ] `Rule::jump_content` 処理削除
- [ ] `Rule::jump_marker` 処理削除
- [ ] `Statement::Jump` 構築コード削除
- [ ] コンパイル確認: `cargo build`

#### 1.4 Parser テスト修正

**対象ファイル** (`tests/pasta_parser_*.rs` or 現行命名):
- Jump 検証ロジック削除
- 全角 Sakura テストケース削除
- `text_part` テストケース追加（`＄` が正しく分離されるか）

**チェックリスト（1.4）**:
- [ ] Jump を使用するテストケース削除
- [ ] 全角 `＼` `［］` テストケース削除
- [ ] `＄` 変数参照が text_part に吸収されないテスト追加
- [ ] `cargo test` で Parser 関連テストが 100% パス

### 成果物
- 修正済み pasta.pest
- 修正済み ast.rs
- 修正済み Parser テスト
- `git commit` 記録（Phase 1 完了）

### 検証方法
```bash
cargo test pasta_parser_ --all
# test result: ok を確認
```

---

## Phase 2: Transpiler 層修正

### 目的
新 Parser AST（Jump 削除）に対応する Transpiler を実装。Transpiler テストが 100% パス。

### 実施内容

#### 2.1 src/transpiler/mod.rs の修正

**対象コード**:
```rust
// 削除対象
Statement::Jump {
    target,
    filters,
    args,
} => {
    let search_key = Self::transpile_jump_target_to_search_key(target);
    writeln!(
        writer,
        "        for a in crate::pasta::jump(ctx, \"{}\", {}, []) {{ yield a; }}",
        search_key, ...
    )?;
}

// 削除対象メソッド
fn transpile_jump_target(target: &JumpTarget) -> String { ... }
fn transpile_jump_target_to_search_key(target: &JumpTarget) -> String { ... }

// 削除対象ランタイム関数定義
pub fn jump(ctx, label, filters, args) { ... }
```

**修正案**:
- `Statement::Jump` 分岐削除
- `transpile_jump_target*` メソッド削除
- `pasta::jump()` ランタイム関数削除

**チェックリスト**:
- [ ] Statement::Jump 分岐削除
- [ ] transpile_jump_target* メソッド削除
- [ ] pasta::jump() 関数削除
- [ ] コンパイル確認: `cargo build`
- [ ] 型推論エラーなし

#### 2.2 Transpiler テスト修正

**対象ファイル** (`pasta_transpiler_*.rs`):
- `pasta_transpiler_two_pass_test.rs`: Jump 前提のテスト削除
- `pasta_transpiler_phase3_test.rs`: Jump ケース削除

**チェックリスト**:
- [ ] Jump 関連テスト削除
- [ ] `cargo test pasta_transpiler_ --all` が 100% パス

### 成果物
- 修正済み transpiler/mod.rs
- 修正済み Transpiler テスト
- Git commit 記録

### 検証方法
```bash
cargo test pasta_transpiler_ --all
# test result: ok を確認
```

---

## Phase 3: Runtime/Tests・ドキュメント修正

### 目的
統合テスト・テストフィクスチャ・ドキュメントを新仕様に更新。全テストが 100% パス。

### 実施内容

#### 3.1 テスト・フィクスチャの置換

**対象ファイル** (`tests/fixtures/`):
- `*.pasta` 内の `？` を `＞` へ置換

**置換コマンド**:
```bash
find tests/fixtures -name "*.pasta" -exec sed -i 's/？/＞/g' {} \;
```

**チェックリスト**:
- [ ] `tests/fixtures/*.pasta` の `？` をすべて `＞` に置換
- [ ] `grep -r "？" tests/fixtures/` で残存確認（なし）

#### 3.2 全角テストケースの削除

**対象ファイル** (`pasta_engine_*.rs`, `pasta_integration_*.rs`):
- 全角 `＼` を使用するテストケース削除
- 全角 `［］` を使用するテストケース削除
- 半角へ統一したテストケースに置換

**例**:
```rust
// 削除対象
さくら：こんにちは＼ｓ［０］

// 置換後
さくら：こんにちは\s[0]
```

**チェックリスト**:
- [ ] `grep -r "＼" tests/` で全角バックスラッシュ削除（パスタスクリプト内のみ許容）
- [ ] `grep -r "［" tests/` で全角括弧削除（パスタスクリプト内のみ許容）

#### 3.3 GRAMMAR.md 改訂

**対象セクション**:

1. **7章（さくらスクリプト）**:
   - 7.1（概要）: 「字句のみ認識、非解釈」を明記
   - 7.3（字句文法）: 「半角 `\` + ASCIIトークン + 任意の非ネスト `[...]`（`\]` 許容）」に統一
   - 7.4（制約）: 「半角バックスラッシュ・半角角括弧のみ」を明記

2. **11章（設計決定）**:
   - 11.6–11.20: 現在の決定状況（✓ 確定項目）
   - 11.16（エスケープ・引用）: `\]` 対応を明記

3. **制御フロー セクション**:
   - Jump 文（`？`）削除
   - Call 文（`＞`）へ統一
   - 既存ドキュメントから Jump 記述削除

**チェックリスト**:
- [ ] 7章に「字句のみ、非解釈」を明記
- [ ] 7.3 に「半角 `\[...]` + `\]` 許容」を具体記述
- [ ] Jump 記述すべて削除
- [ ] Call のみで制御フローを説明

#### 3.4 統合テスト・エンジンテスト修正

**対象ファイル** (`pasta_engine_*.rs`, `pasta_integration_*.rs`):
- Jump 依存テスト削除
- Call のみで制御フローをテスト

**チェックリスト**:
- [ ] `grep -r "Jump" tests/` でコメント検索（意図確認）
- [ ] Jump テストコード削除
- [ ] `cargo test pasta_engine_ --all` が 100% パス
- [ ] `cargo test pasta_integration_ --all` が 100% パス

### 成果物
- 置換済みテストフィクスチャ
- 修正済みテストコード
- 改訂済み GRAMMAR.md
- Git commit 記録

### 検証方法
```bash
cargo test --all
# test result: ok を確認
```

---

## 設計の判断ポイント・トレードオフ

### 1. Sakura コマンドの簡素化レベル

- **案 A**: 現在の詳細5パターンを維持し、半角・`\]` 対応のみ
- **案 B**: 「未知トークン許容」で完全に簡素化（仕様「非解釈」に最も準拠）

**決定**: ✅ **案 B（完全簡素化）を採用**

**理由**:
- 仕様「Sakura は字句のみ認識、非解釈」に忠実
- 詳細5パターン区別は必要ない（実装側も複雑化しない）
- **ただしブラケット内の `\]` エスケープ対応は必須**
- 未知トークンを通すことで、将来の拡張性確保
- テスト修正も最小化（詳細パターン検証ケース削除）

### 2. Jump 削除の最終決定

- **案 A**: Jump 廃止は必須 → Phase 1-3 で Jump 削除を進行
- **案 B**: Jump 維持 → design.md・gap-analysis を修正し、Jump は継続サポート

**決定**: ✅ **案 A（Jump 廃止は必須）**

**理由**:
- MVP 達成前の段階における積極的な破壊的変更対応
- Jump と Call のセマンティクス上の区別がなく、統一してシンプル化
- DSL 文法の整理・保守性向上
- テスト修正規模は大きいが、MVP 達成前だからこそ対応が容易

### 3. テスト修正の優先順序

- **案 A**: Parser → Transpiler → Engine/Integration
- **案 B**: 層別に並行修正

**推奨**: 案 A（依存関係に従い、下層完全化が上層前提）

---

## リスク・緩和策

| リスク | 発生箇所 | 緩和策 |
|-------|--------|------|
| Phase 1 ast.rs 型変更の波及 | Parser 出力の型が変更 | Transpiler で compiler error として即座に検出 |
| Jump 削除漏れ | 複数ファイルに分散 | grep で Jump 関連コード検索、チェックリスト活用 |
| テスト置換ミス（`？` → `＞`） | fixtures が混在状態 | Phase 0 の test-baseline.log と比較 |
| 全角テスト削除漏れ | 複数ファイル | `grep -r "＼" tests/` で確認 |

---

## 実装スケジュール（推定）

| Phase | 工数 | 説明 |
|-------|------|------|
| Phase 0 | 1日 | テスト層別化・グリーン確認・commit |
| Phase 1 | 5–7日 | Parser 層（pest/AST）完全修正 |
| Phase 2 | 3–5日 | Transpiler 層修正 |
| Phase 3 | 5–10日 | Runtime/Tests/GRAMMAR.md 修正 |
| **合計** | **14–23 日** | |

---

## 次ステップ

- 要件承認（requirements.md ✓）
- **本設計の承認**
- Phase 0 実装開始
