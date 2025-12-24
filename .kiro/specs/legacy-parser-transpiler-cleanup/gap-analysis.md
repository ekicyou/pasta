# Implementation Gap Analysis

## 分析概要

### スコープ
- 旧 `src/parser/` および `src/transpiler/` ディレクトリの完全削除
- parser2/transpiler2 を parser/transpiler に正規化（「２」を除去）
- ビルド・テスト修正とコミット作業の段階的実施

### 主要な課題
- **ソースコード層**: `src/lib.rs`, `src/cache.rs`, `src/runtime/words.rs`, `src/stdlib/mod.rs` に旧モジュールへの参照が存在
- **テストコード層**: `tests/` 配下の12個のテストファイルが旧parser/transpilerをインポート
- **公開API**: `src/lib.rs` が旧parserの13個の型（`Argument`, `Attribute`, `BinOp`, etc.）を再公開
- **モジュール名正規化**: parser2/transpiler2を段階的にリネーム（中間名を経由）

### 推奨アプローチ
**Option C (Hybrid Approach)** を推奨：
- Phase 1-4: 旧実装削除とビルド復旧（既存コード修正）
- Phase 5-6: モジュール名正規化（新しい名前空間へ移行）
- Phase 7-8: クリーンアップとテスト修正（品質保証）

---

## 1. Current State Investigation

### 1.1 Key Files and Modules

#### 旧実装（削除対象）
| ディレクトリ      | ファイル                         | 説明                                |
| ----------------- | -------------------------------- | ----------------------------------- |
| `src/parser/`     | `mod.rs`, `ast.rs`, `pasta.pest` | 旧パーサー実装（3ファイル）         |
| `src/transpiler/` | `mod.rs`                         | 旧トランスパイラー実装（1ファイル） |

#### 新実装（リネーム対象）
| ディレクトリ       | ファイル                                                | 説明                                |
| ------------------ | ------------------------------------------------------- | ----------------------------------- |
| `src/parser2/`     | `mod.rs`, `ast.rs`, `grammar.pest`                      | 新パーサー実装（3ファイル）         |
| `src/transpiler2/` | `mod.rs`, `code_generator.rs`, `context.rs`, `error.rs` | 新トランスパイラー実装（4ファイル） |

### 1.2 Dependencies and References

#### ソースコード層の参照
| ファイル                        | 参照箇所                                                                         | 影響範囲             |
| ------------------------------- | -------------------------------------------------------------------------------- | -------------------- |
| `src/lib.rs` (Line 47)          | `pub mod parser;`                                                                | クレート公開API      |
| `src/lib.rs` (Line 52)          | `pub mod transpiler;`                                                            | クレート公開API      |
| `src/lib.rs` (Line 61-64)       | `pub use parser::{...13個の型...}`                                               | 公開型の再export     |
| `src/lib.rs` (Line 70)          | `pub use transpiler::{TranspileContext, Transpiler}`                             | 公開型の再export     |
| `src/cache.rs` (Line 96-233)    | `#[cfg(test)] mod tests { use crate::parser::...; use crate::transpiler::...; }` | テストモジュール全体 |
| `src/runtime/words.rs` (Line 8) | `use crate::transpiler::{WordDefRegistry, WordEntry};`                           | 実装コード           |

**注記**: `src/stdlib/mod.rs` は WordTable のみ使用、transpiler への直接参照なし（修正不要）

#### テストコード層の参照（21ファイル削除対象 + 1ファイル修正）

**カテゴリA: 旧parser専用（12ファイル）** - 削除
1. `tests/pasta_parser_debug_test.rs`
2. `tests/pasta_parser_error_test.rs`
3. `tests/pasta_parser_golden_test.rs`
4. `tests/pasta_parser_grammar_diagnostic_test.rs`
5. `tests/pasta_parser_line_types_test.rs`
6. `tests/pasta_parser_main_test.rs`
7. `tests/pasta_parser_pest_debug_test.rs`
8. `tests/pasta_parser_pest_sakura_test.rs`
9. `tests/pasta_parser_phase1_test.rs`
10. `tests/pasta_parser_sakura_debug_test.rs`
11. `tests/pasta_parser_sakura_script_test.rs`
12. `tests/pasta_parser_spec_validation_test.rs`

**カテゴリB: 旧transpiler専用（7ファイル）** - 削除
13. `tests/pasta_transpiler_actor_assignment_test.rs`
14. `tests/pasta_transpiler_comprehensive_test.rs`
15. `tests/pasta_transpiler_phase3_test.rs`
16. `tests/pasta_transpiler_scene_registry_test.rs`
17. `tests/pasta_transpiler_two_pass_test.rs`
18. `tests/pasta_transpiler_variable_expansion_test.rs`
19. `tests/pasta_transpiler_word_code_gen_test.rs`

**カテゴリC: 旧parser+transpiler統合（2ファイル）** - 削除
20. `tests/pasta_integration_e2e_simple_test.rs`
21. `tests/pasta_engine_rune_compile_test.rs`
22. `tests/pasta_engine_rune_vm_comprehensive_test.rs`

**カテゴリD: Registry参照（1ファイル）** - 修正して残す
- `tests/pasta_stdlib_call_jump_separation_test.rs`
  - `use pasta::transpiler::{SceneRegistry, WordDefRegistry}` → `use pasta::registry::{SceneRegistry, WordDefRegistry}`
  - テスト目的: Call/Jumpがword dictionaryにアクセスしない設計原則の検証
  - 旧parser/transpilerに依存せず、registryモジュールのみ使用

**追加削除対象**:
- `src/cache.rs` (Line 96-233): `#[cfg(test)]` テストモジュール全体（旧parser/transpiler依存）
- `README.md` (Line 26-43): レガシースタックに関する記述を完全削除
  - 削除内容: モジュール状態テーブルの「レガシー」行、レガシースタック使用例、Phase 4 記述
  - 残留内容: 「パーサー/トランスパイラーアーキテクチャ」セクションのタイトルと現行スタック情報のみ

### 1.3 Existing Patterns

#### 命名規則
- **新モジュール**: `parser2`, `transpiler2`（数字サフィックス）
- **正規化後**: `parser`, `transpiler`（サフィックスなし）
- **中間名**: `parser_new`, `transpiler_new`（衝突回避）

#### Registry分離パターン
- `SceneRegistry`, `WordDefRegistry` は既に `src/registry/` に独立モジュール化済み
- 旧transpilerの `SceneRegistry`, `WordDefRegistry` は registry モジュールへの橋渡し型（削除可能）

---

## 2. Requirements Feasibility Analysis

### 2.1 Technical Needs by Requirement

#### Requirement 1: 旧実装ディレクトリの削除
- **必要な操作**: `rm -rf src/parser/`, `rm -rf src/transpiler/`
- **前提条件**: なし（物理削除のみ）
- **リスク**: Low（次ステップでビルドエラーを修正）

#### Requirement 2: ソースコード層のビルド復旧
- **必要な操作**:
  - `src/lib.rs`: `pub mod parser;` と `pub use parser::{...}` を削除
  - `src/lib.rs`: `pub mod transpiler;` と `pub use transpiler::{...}` を削除
  - `src/cache.rs` (Line 96-233): `#[cfg(test)]` モジュール全体を削除（旧parser/transpiler依存テスト）
  - `src/runtime/words.rs` (Line 8): `use crate::transpiler::{WordDefRegistry, WordEntry}` → `use crate::registry::{WordDefRegistry, WordEntry}`
- **検証**: `cargo check` 成功

#### Requirement 3: テストコード層のビルド復旧
- **必要な操作**:
  - 21個のテストファイルを完全削除（カテゴリA: parser 12個、カテゴリB: transpiler 7個、カテゴリC: 統合2個）
  - 1個のテストファイルを修正（カテゴリD: `pasta_stdlib_call_jump_separation_test.rs`）
  - 詳細リストは「1.2 Dependencies and References」参照
- **検証**: `cargo check --all` 成功

#### Requirement 4: テスト実行の復旧
- **必要な操作**:
  - テストファイル削除後、残存テストが正常に実行されることを確認
  - `README.md` (Line 37-43): レガシースタック使用例のコードブロックを削除
- **検証**: `cargo test --all` 成功

#### Requirement 5: モジュール名の正規化
- **必要な操作**:
  1. `mv src/parser2 src/parser_new`（衝突回避）
  2. `mv src/transpiler2 src/transpiler_new`（衝突回避）
  3. `mv src/parser_new src/parser`（最終配置）
  4. `mv src/transpiler_new src/transpiler`（最終配置）
  5. 全ファイルの `use` 文とモジュール宣言を更新
- **検証**: なし（次のRequirementで実施）

#### Requirement 6: モジュール名正規化後のビルド復旧
- **必要な操作**:
  - `src/lib.rs`: `pub mod parser2;` → `pub mod parser;`
  - `src/lib.rs`: `pub mod transpiler2;` → `pub mod transpiler;`
  - 全ソースファイル: `use crate::parser2::...` → `use crate::parser::...`
  - 全ソースファイル: `use crate::transpiler2::...` → `use crate::transpiler::...`
  - 全テストファイル: `use pasta::parser2::...` → `use pasta::parser::...`
  - 全テストファイル: `use pasta::transpiler2::...` → `use pasta::transpiler::...`
- **検証**: `cargo check` + `cargo test --all` 成功

#### Requirement 7: 未使用テストフィクスチャの削除
- **必要な操作**:
  - `tests/` 配下の `*.rs` 以外のファイルをリストアップ
  - grep/semantic_search でテストコードからの参照を確認
  - 参照されていないファイルを削除
- **検証**: `cargo test --all` 成功

#### Requirement 8: 最終検証とコミット
- **必要な操作**:
  - `cargo check --all` + `cargo test --all` 成功確認
  - `git add -A && git commit` (段階的に実施済みのはず)
- **検証**: すべてのテストが通ること

### 2.2 Identified Gaps and Constraints

#### Gap 1: Registry依存の整理（議題2で確認済み）
- **状態**: `WordDefRegistry`, `WordEntry` は既に `src/registry/` に実装済み
- **旧transpilerの役割**: `pub use crate::registry::{...}` で再export（後方互換）
- **対応**: `src/runtime/words.rs` (Line 8) を `use crate::registry::` に変更
- **対応**: `src/stdlib/mod.rs` は WordTable のみ使用、変更不要
- **制約**: なし

#### Gap 2: テストコードの移行戦略
- **状態**: 旧parser/transpilerのテストは新実装と互換性なし
- **選択肢**:
  - A. テストを無効化（コメントアウト）
  - B. テストを parser2/transpiler2 API に書き換え
  - C. テストファイルを削除
- **決定**: **Option C**（完全削除） ← 議題1で決定
- **理由**: 旧実装依存のテストは保持する意義がなく、parser2用テストは別仕様で追加予定

#### Gap 3: 公開APIの互換性
- **状態**: `src/lib.rs` が旧parserの13個の型を公開中
- **影響**: クレート利用者（areka等）がこれらの型に依存している可能性
- **対応**: parser2 の型を同じ名前で公開（モジュール名正規化後）
- **制約**: 型定義の互換性は保証されない（破壊的変更の可能性）

#### Constraint 1: Gitコミット戦略
- **要件**: 各フェーズ完了後にコミット（Req 3, 4, 6, 8）
- **パターン**: `git add -A && git commit -m "refactor(cleanup): <description>"`
- **タイミング**:
  - Phase 1-3: `cargo check --all` 成功後
  - Phase 4: `cargo test --all` 成功後
  - Phase 5-6: `cargo test --all` 成功後
  - Phase 7-8: `cargo test --all` 成功後

### 2.3 Complexity Signals

| 作業               | 複雑性   | 理由                                      |
| ------------------ | -------- | ----------------------------------------- |
| ディレクトリ削除   | Simple   | 物理削除のみ                              |
| ソースコード修正   | Simple   | 参照削除・置換（4ファイル）               |
| テストコード修正   | Simple   | 参照削除・コメントアウト（12ファイル）    |
| モジュールリネーム | Moderate | 2段階リネーム（衝突回避）+ 全ファイル更新 |
| テスト復旧         | Moderate | API変更に伴うテストロジック調整           |
| 未使用ファイル削除 | Simple   | 参照チェック + 削除                       |

---

## 3. Implementation Approach Options

### Option A: Extend Existing Components ❌ **適用不可**
旧実装を「拡張」する要件ではなく、「削除」する要件のため、このオプションは該当しません。

---

### Option B: Create New Components ❌ **適用不可**
新規コンポーネント作成ではなく、既存コードの削除とリネーム作業のため、該当しません。

---

### Option C: Hybrid Approach ✅ **推奨**

#### 適用理由
- **Phase 1-4**: 旧実装削除（削除作業）
- **Phase 5-6**: モジュール名正規化（リネーム＋既存コード更新）
- **Phase 7-8**: クリーンアップ（削除作業）

3種類の異なる作業（削除・リネーム・参照更新）を段階的に実施するハイブリッドアプローチ。

#### 実装戦略

##### Phase 1-2: 旧実装削除 + ソースコードビルド復旧
1. `rm -rf src/parser/ src/transpiler/`
2. `src/lib.rs` 修正:
   - `pub mod parser;` 削除
   - `pub mod transpiler;` 削除
   - `pub use parser::{...}` 削除（13型）
   - `pub use transpiler::{...}` 削除（2型）
3. `src/runtime/words.rs` 修正 (Line 8):
   - `use crate::transpiler::{WordDefRegistry, WordEntry};` → `use crate::registry::{WordDefRegistry, WordEntry};`
4. `src/cache.rs` 修正 (Line 96-233):
   - `#[cfg(test)]` モジュール全体を削除（旧parser/transpiler依存テスト）
6. **検証**: `cargo check` 成功
7. **コミット**: `git add -A && git commit -m "refactor(cleanup): 旧parser/transpilerディレクトリ削除とソースコード修正"`

##### Phase 3: テストコードビルド復旧
1. 21個のテストファイルを完全削除:
   - カテゴリA: `tests/pasta_parser_*.rs` (12ファイル)
   - カテゴリB: `tests/pasta_transpiler_*.rs` (7ファイル)
   - カテゴリC: `tests/pasta_integration_e2e_simple_test.rs`, `tests/pasta_engine_rune_compile_test.rs`, `tests/pasta_engine_rune_vm_comprehensive_test.rs`
2. 1個のテストファイルを修正:
   - `tests/pasta_stdlib_call_jump_separation_test.rs`: `use pasta::transpiler::` → `use pasta::registry::`
3. `README.md` 修正 (Line 26-43):
   - レガシースタックに関する記述を完全削除
   - モジュール状態テーブルから「レガシー」行を削除
   - レガシースタック使用例のコードブロックを削除
   - Phase 4 「保留」記述を削除
   - 「現行スタック（推奨）」の記述のみ残す
4. `src/cache.rs` 修正 (Line 96-233):
   - `#[cfg(test)]` モジュール全体を削除
5. **検証**: `cargo check --all` 成功
6. **コミット**: `git add -A && git commit -m "refactor(cleanup): 旧parser/transpilerテスト削除とREADME更新"`
3. **検証**: `cargo check --all` 成功
4. **コミット**: `git add -A && git commit -m "refactor(cleanup): 旧parser/transpilerテスト削除とREADME更新"`

##### Phase 4: テスト実行復旧
1. テストファイル削除後、残存テストが正常に実行されることを確認
2. **検証**: `cargo test --all` 成功
3. **コミット**: `git add -A && git commit -m "refactor(cleanup): テスト実行復旧完了"`

##### Phase 5: モジュール名正規化（リネーム）
1. ディレクトリリネーム（2段階）:
   ```bash
   mv src/parser2 src/parser_new
   mv src/transpiler2 src/transpiler_new
   mv src/parser_new src/parser
   mv src/transpiler_new src/transpiler
   ```
2. `src/lib.rs` 修正:
   - `pub mod parser2;` → `pub mod parser;`
   - `pub mod transpiler2;` → `pub mod transpiler;`
3. **注記**: まだビルドは通らない（全参照を次のPhaseで更新）

##### Phase 6: モジュール名正規化後のビルド・テスト復旧
1. 全ソースファイルの参照更新:
   - `use crate::parser2::` → `use crate::parser::`
   - `use crate::transpiler2::` → `use crate::transpiler::`
   - （grep検索で一括置換）
2. 全テストファイルの参照更新:
   - `use pasta::parser2::` → `use pasta::parser::`
   - `use pasta::transpiler2::` → `use pasta::transpiler::`
3. **検証**: `cargo check` 成功
4. **検証**: `cargo test --all` 成功
5. **コミット**: `git add -A && git commit -m "refactor(cleanup): parser2/transpiler2をparser/transpilerに正規化"`

##### Phase 7: 未使用テストフィクスチャ削除
1. `tests/` 配下の `*.rs` 以外をリストアップ:
   ```bash
   find tests -type f ! -name "*.rs"
   ```
2. 各ファイルについてgrep検索で参照確認:
   ```bash
   grep -r "<filename>" tests/
   ```
3. 参照されていないファイルを削除
4. **検証**: `cargo test --all` 成功

##### Phase 8: 最終検証とコミット
1. **検証**: `cargo check --all` 成功
2. **検証**: `cargo test --all` 成功
3. **コミット**: `git add -A && git commit -m "refactor(cleanup): 未使用テストフィクスチャ削除"`

#### Trade-offs
- ✅ 段階的実施によりリスク低減
- ✅ 各フェーズで検証可能
- ✅ コミット履歴が明確
- ✅ 問題発生時のロールバックが容易
- ❌ 8段階の手順で時間がかかる
- ❌ テストコード削除によりカバレッジ低下（parser2用テストは別仕様で追加）

---

## 4. Implementation Complexity & Risk

### Effort Estimation
- **Size**: **M (3-7 days)**
  - **根拠**: 単純な削除・置換作業だが、12個のテストファイル修正と段階的検証が必要
  - **内訳**:
    - Phase 1-2: 0.5日（ソースコード修正4ファイル）
    - Phase 3-4: 0.5日（テストコード12ファイル削除 + README修正）
    - Phase 5-6: 1日（モジュールリネーム + 全参照更新）
    - Phase 7-8: 0.5日（未使用ファイル削除 + 最終検証）
    - バッファ: 1.5日（予期しないビルドエラー対応）

### Risk Assessment
- **Risk Level**: **Low**
  - **根拠**: 既存パターンに従った削除・置換作業のみ、複雑なロジック変更なし
  - **リスク要因**:
    - ✅ 旧実装への依存は明確（grep検索で完全に特定可能）
    - ✅ parser2/transpiler2 は既に動作実績あり
    - ✅ Registry分離は完了済み（[pasta-test-missing-entry-hash](../.kiro/specs/completed/pasta-test-missing-entry-hash/)）
    - ⚠️ テストコード削除によりカバレッジ低下（リスク: Low - parser2用テストは別仕様で追加予定）
    - ⚠️ 公開API変更によりクレート利用者への影響（リスク: Medium - 別仕様で対処）

---

## 5. Recommendations for Design Phase

### 推奨アプローチ
**Option C (Hybrid Approach)** を採用し、8フェーズに分割して段階的に実施。

### Key Decisions
1. **テストコード戦略**: 旧parser/transpilerテストは完全削除（12ファイル）→ parser2 用テストは別仕様で追加
2. **Registry依存**: `src/runtime/words.rs` と `src/stdlib/mod.rs` を `use crate::registry` に変更
3. **中間リネーム**: `parser_new`/`transpiler_new` を経由してディレクトリ衝突を回避
4. **コミット粒度**: 各Phase完了時にコミット（Phase 2, 4, 6, 8）

### Research Items
以下の項目は設計フェーズで詳細化が必要：

#### Research 1: 未使用テストフィクスチャの特定
- **目的**: `tests/` 配下の `*.rs` 以外のファイルで参照されていないものをリストアップ
- **方法**: `find` + `grep` または semantic_search
- **タイミング**: Phase 7実施前

#### Research 2: 公開API互換性の影響調査
- **目的**: 旧parser型（`Argument`, `Attribute`, etc.）を削除した場合の影響範囲
- **方法**: arekaプロジェクトでの参照確認
- **タイミング**: 本仕様完了後（別仕様として切り出し）
- **Note**: parser2 の型定義は旧parserと **互換性なし**（破壊的変更の可能性）

#### Research 3: README.md サンプルコードの更新方針
- **目的**: 旧APIサンプルを parser2/transpiler2 に更新するか削除するか
- **方法**: 現在のREADME.mdを確認し、サンプルコードの役割を評価
- **タイミング**: Phase 3実施時

---

## 6. Summary

### 分析結果
- **要件の実現可能性**: ✅ **高い**（すべての要件は既存のgrep/置換操作で達成可能）
- **既存パターンとの整合性**: ✅ **高い**（Registry分離パターンを踏襲）
- **実装リスク**: ✅ **Low**（削除・置換のみ、複雑なロジック変更なし）
- **推奨アプローチ**: **Option C (Hybrid)** - 8フェーズ段階的実施

### 次のステップ
1. 本gap分析を確認
2. `/kiro-spec-design legacy-parser-transpiler-cleanup` で詳細設計を生成
3. 設計承認後、`/kiro-spec-tasks legacy-parser-transpiler-cleanup` でタスク分解
4. タスク承認後、`/kiro-spec-impl legacy-parser-transpiler-cleanup` で実装開始
