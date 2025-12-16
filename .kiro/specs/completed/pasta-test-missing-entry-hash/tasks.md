# Tasks: pasta-test-missing-entry-hash

## 概要

**目的**: MissingEntryHashエラーを修正し、全テストを成功させる

**MVP定義**: 
1. 現在失敗している全テストを成功させる
2. 無効化されたテストを有効化して成功させる
3. コメントアウトや`#[ignore]`などテスト無効化行為を禁止
4. テストファースト原則を遵守

**根本原因**: 
Runeの`Hash::type_hash`に渡すパス形式が間違っている。
- ❌ 現在: `Hash::type_hash(&["module::function"])` (1要素の配列)
- ✅ 正しい: `Hash::type_hash(&["module", "function"])` (2要素の配列)

**解決策**:
`fn_name.split("::")`でパスを分割してからHash計算する。

---

## Phase 0: 事前調査・準備

### Task 0.1: 現在のテスト状況を完全把握 ✅

**目的**: 全テストの状態を正確に把握する

**実施内容**:
```bash
cargo test --package pasta --all-targets 2>&1 > test_status.txt
```

**確認項目**:
1. 失敗しているテスト数
2. 無効化されているテスト（`#[ignore]`）
3. コンパイルエラー

**既知の状況**:
- **concurrent_execution_test.rs:44-45**: 構文エラー（修正済み）
- **無効化テスト**: 3つ発見
  - `end_to_end_simple_test.rs:70`: `#[ignore]` - generator support
  - `engine_two_pass_test.rs:31`: `#[ignore]` - encoding issues
  - `engine_two_pass_test.rs:58`: `#[ignore]` - execution test

**成果物**: test_status.txt

**完了条件**: 全テストの現状が文書化されている

---

## Phase 1: 核心バグ修正（優先度: 最高）

### Task 1.1: engine.rsのHash計算を修正 🎯

**目的**: MissingEntryHashエラーの根本原因を修正

**対象ファイル**: `crates/pasta/src/engine.rs`

**対象メソッド**: `execute_label_with_filters`（508行目付近）

**修正前**:
```rust
pub fn execute_label_with_filters(
    &mut self,
    label_name: &str,
    filters: &HashMap<String, String>,
) -> Result<Vec<ScriptEvent>> {
    let fn_name = self.label_table.find_label(label_name, filters)?;
    let mut vm = Vm::new(self.runtime.clone(), self.unit.clone());
    
    // ❌ 問題のコード
    let hash = rune::Hash::type_hash(&[fn_name.as_str()]);
    
    let context = self.build_execution_context()?;
    let execution = vm
        .execute(hash, (context,))
        .map_err(|e| PastaError::VmError(e))?;
    // ...
}
```

**修正後**:
```rust
pub fn execute_label_with_filters(
    &mut self,
    label_name: &str,
    filters: &HashMap<String, String>,
) -> Result<Vec<ScriptEvent>> {
    let fn_name = self.label_table.find_label(label_name, filters)?;
    let mut vm = Vm::new(self.runtime.clone(), self.unit.clone());
    
    // ✅ 修正: fn_nameを"::"で分割してパスの配列を作る
    // fn_name format: "module_name::function_name"
    // Rune expects: ["module_name", "function_name"]
    let parts: Vec<&str> = fn_name.split("::").collect();
    let hash = rune::Hash::type_hash(&parts);
    
    let context = self.build_execution_context()?;
    let execution = vm
        .execute(hash, (context,))
        .map_err(|e| PastaError::VmError(e))?;
    // ...
}
```

**変更内容**:
1. 508行目の`let hash = rune::Hash::type_hash(&[fn_name.as_str()]);`を削除
2. 以下の3行を追加（コメント含む）:
   ```rust
   // Split fn_name into path components for Rune
   // fn_name format: "module_name::function_name"
   // Rune expects: ["module_name", "function_name"]
   let parts: Vec<&str> = fn_name.split("::").collect();
   let hash = rune::Hash::type_hash(&parts);
   ```

**完了条件**:
- ✅ `cargo build --package pasta` が成功
- ✅ コンパイルエラーなし

**期待される効果**:
- 24個の失敗テストが成功する（MissingEntryHashエラー解消）

---

### Task 1.2: 修正後の基本テスト実行

**目的**: 核心修正が正しく動作することを確認

**実施内容**:
```bash
# ライブラリテスト
cargo test --package pasta --lib

# 統合テスト（失敗していたもの）
cargo test --package pasta --test engine_independence_test
cargo test --package pasta --test concurrent_execution_test
```

**確認項目**:
1. engine_independence_test: 9/9 passing
2. concurrent_execution_test: 7/7 passing
3. lib tests: 50/50 passing

**完了条件**: 上記テストが全て成功

---

## Phase 2: 無効化テストの復旧（優先度: 高）

### Task 2.1: end_to_end_simple_test.rsの調査と修正

**対象ファイル**: `crates/pasta/tests/end_to_end_simple_test.rs:70`

**現状**: 
```rust
#[ignore] // Ignore for now, need to implement generator support
#[test]
fn test_end_to_end_execution() {
    // ...
}
```

**調査事項**:
1. generatorサポートが実装されているか確認
2. `#[ignore]`を削除して実行
3. 失敗する場合、原因を特定

**修正方針**:
- **ケースA**: generatorサポートが実装済み → `#[ignore]`削除、テスト成功
- **ケースB**: generatorサポート未実装 → 本仕様で実装してテスト成功
- **ケースC**: テスト自体が古い → テストを現在のAPIに合わせて修正

**禁止事項**:
- ❌ `#[ignore]`を残したまま完了とする
- ❌ テストをコメントアウトする
- ❌ テストを削除する

**完了条件**: 
- ✅ `#[ignore]`が削除されている
- ✅ テストが成功する

---

### Task 2.2: engine_two_pass_test.rs:31の調査と修正

**対象ファイル**: `crates/pasta/tests/engine_two_pass_test.rs:31`

**現状**:
```rust
#[ignore] // test-project has encoding issues
#[test]
fn test_two_pass_load() {
    // ...
}
```

**調査事項**:
1. encodingの問題が解決されているか確認
2. `#[ignore]`を削除して実行
3. 失敗する場合、encoding問題を修正

**修正方針**:
- **ケースA**: encoding問題が解決済み → `#[ignore]`削除、テスト成功
- **ケースB**: test-projectのファイルがおかしい → ファイルを修正
- **ケースC**: 読み込みコードがおかしい → 読み込みコードを修正

**禁止事項**:
- ❌ `#[ignore]`を残したまま完了とする
- ❌ encoding問題を放置する

**完了条件**:
- ✅ `#[ignore]`が削除されている
- ✅ テストが成功する

---

### Task 2.3: engine_two_pass_test.rs:58の調査と修正

**対象ファイル**: `crates/pasta/tests/engine_two_pass_test.rs:58`

**現状**:
```rust
#[ignore] // Ignore until we can test execution
#[test]
fn test_two_pass_execution() {
    // ...
}
```

**調査事項**:
1. executionテストが可能になっているか確認
2. `#[ignore]`を削除して実行
3. 失敗する場合、原因を特定して修正

**修正方針**:
- **ケースA**: 実行テストが可能 → `#[ignore]`削除、テスト成功
- **ケースB**: エンジンが不完全 → 本仕様で修正してテスト成功
- **ケースC**: テストが古い → テストを現在のAPIに合わせて修正

**禁止事項**:
- ❌ `#[ignore]`を残したまま完了とする
- ❌ 実行テスト不可能のまま放置

**完了条件**:
- ✅ `#[ignore]`が削除されている
- ✅ テストが成功する

---

## Phase 3: 全テスト検証（優先度: 高）

### Task 3.1: 全テストの実行

**目的**: 全てのテストが成功することを確認

**実施内容**:
```bash
cargo test --package pasta --all-targets
```

**確認項目**:
1. 全テストが成功している
2. `#[ignore]`が残っていない（grep確認）
3. コメントアウトされたテストがない

**検証コマンド**:
```bash
# 無効化テストの検索
grep -r "#\[ignore\]" crates/pasta/tests/
grep -r "// *#\[test\]" crates/pasta/tests/
grep -r "/\* *#\[test\]" crates/pasta/tests/

# 結果: 何も出力されないことを確認
```

**完了条件**:
- ✅ `cargo test --package pasta --all-targets` が全て成功
- ✅ 無効化されたテストが存在しない
- ✅ test result: XX passed; 0 failed; 0 ignored

**期待されるテスト数**: 79+ tests passing（無効化テスト復旧により増加）

---

## Phase 4: クリーンアップ（優先度: 中）

### Task 4.1: 未使用コードの削除

**目的**: コンパイラ警告を解消

**対象ファイル**: `crates/pasta/src/engine.rs`

**削除対象**:
```rust
// 未使用メソッド（警告が出ている）
fn build_engine(...) { ... }           // 297行目
fn register_labels(...) { ... }        // 407行目
fn generate_fn_name_with_counter(...) { ... }  // 444行目
```

**完了条件**:
- ✅ `cargo build --package pasta` で警告なし
- ✅ 既存テストが全て成功

---

### Task 4.2: その他の警告解消

**目的**: コードクリーンアップ

**対象**:
```rust
// unused_imports
crates/pasta/src/runtime/labels.rs:53 - use crate::transpiler::LabelInfo

// dead_code
crates/pasta/src/engine.rs:61 - field `cache`

// unused_mut など
各種テストファイルの不要な mut
```

**実施内容**:
```bash
cargo fix --package pasta --all-targets --allow-dirty
```

**完了条件**:
- ✅ `cargo build --package pasta --all-targets` で警告0件
- ✅ 全テストが成功

---

### Task 4.3: フォーマットとlint

**目的**: コード品質の保証

**実施内容**:
```bash
cargo fmt --all
cargo clippy --package pasta -- -D warnings
```

**完了条件**:
- ✅ `cargo fmt --all` が成功
- ✅ `cargo clippy --package pasta` で警告なし

---

## Phase 5: 最終検証・報告（優先度: 高）

### Task 5.1: 最終テスト実行

**目的**: 全ての変更が正しく動作することを最終確認

**実施内容**:
```bash
# クリーンビルド
cargo clean
cargo build --package pasta

# 全テスト
cargo test --package pasta --all-targets

# フォーマット・lint
cargo fmt --all -- --check
cargo clippy --package pasta -- -D warnings
```

**完了条件**:
- ✅ クリーンビルド成功
- ✅ 全テスト成功
- ✅ フォーマットOK
- ✅ clippy警告なし

---

### Task 5.2: 実装レポートの作成

**目的**: 実装内容を文書化

**作成ファイル**: `.kiro/specs/pasta-test-missing-entry-hash/implementation-report.md`

**記載内容**:
```markdown
# Implementation Report: pasta-test-missing-entry-hash

## 実装サマリー

- **実装日時**: YYYY-MM-DDTHH:mm:ss.sssZ
- **ステータス**: ✅ 完了

## 変更内容

### 1. 核心修正
- engine.rs: Hash計算ロジック修正（508行目）

### 2. 無効化テスト復旧
- end_to_end_simple_test.rs: #[ignore]削除、修正内容...
- engine_two_pass_test.rs:31: #[ignore]削除、修正内容...
- engine_two_pass_test.rs:58: #[ignore]削除、修正内容...

### 3. クリーンアップ
- 未使用コード削除
- 警告解消
- フォーマット・lint

## テスト結果

### Before
- Total: XX tests
- Passing: 55 tests
- Failing: 24 tests
- Ignored: 3 tests

### After
- Total: XX tests
- Passing: XX tests (100%)
- Failing: 0 tests
- Ignored: 0 tests

## 根本原因

Runeの`Hash::type_hash`に渡すパスが間違っていた：
- 誤: `&["module::function"]` (1要素)
- 正: `&["module", "function"]` (2要素)

## 検証

✅ 全テスト成功
✅ 無効化テストなし
✅ コンパイル警告なし
✅ clippy警告なし
```

**完了条件**: レポートが作成されている

---

## 作業ガイドライン

### 🚫 禁止事項

1. **テスト無効化の禁止**
   - `#[ignore]`の使用禁止
   - テストのコメントアウト禁止
   - テストの削除禁止（明らかに不要な場合を除く）

2. **問題の先送り禁止**
   - 「後で修正」は許可しない
   - 全ての問題はこの仕様スコープ内で解決
   - スコープ外判定の禁止（テストが動かないのは本仕様の責任）

3. **テストファースト原則違反の禁止**
   - テストを無効化してから実装、は禁止
   - テストが失敗している状態で完了としない

### ✅ 許可事項

1. **テストの修正**
   - 古いAPIを使用している場合、新APIに修正OK
   - テストの期待値が間違っている場合、修正OK
   - ただし、テスト意図を変えないこと

2. **実装の追加**
   - テストが要求する機能が未実装なら実装OK
   - 最小限の変更原則を守る

3. **テストの追加**
   - 新しいテストケースの追加OK
   - ただし、既存テストの無効化の代替としては禁止

### 📋 作業チェックリスト

各タスク完了時に確認：

- [ ] Task 0.1: テスト状況把握
- [ ] Task 1.1: Hash計算修正
- [ ] Task 1.2: 基本テスト成功
- [ ] Task 2.1: end_to_end_simple_test復旧
- [ ] Task 2.2: engine_two_pass_test:31復旧
- [ ] Task 2.3: engine_two_pass_test:58復旧
- [ ] Task 3.1: 全テスト成功
- [ ] Task 4.1: 未使用コード削除
- [ ] Task 4.2: 警告解消
- [ ] Task 4.3: フォーマット・lint
- [ ] Task 5.1: 最終検証
- [ ] Task 5.2: レポート作成

### 🎯 MVP達成条件

1. ✅ `cargo test --package pasta --all-targets` が全て成功
2. ✅ MissingEntryHashエラーが0件
3. ✅ `#[ignore]`が0件
4. ✅ コメントアウトされたテストが0件
5. ✅ コンパイル警告が0件
6. ✅ clippy警告が0件

全て達成して初めて本仕様完了とする。

---

## コンテキスト継続のための重要情報

### 根本原因の詳細

**問題**: `engine.rs:508`で、`fn_name`を1要素の配列として`Hash::type_hash`に渡している

**証拠**:
```rust
// 現在のコード（間違い）
let fn_name = "test1_1::__start__";  // find_labelの戻り値
let hash = rune::Hash::type_hash(&[fn_name.as_str()]);
// → Hash::type_hash(&["test1_1::__start__"])  ← 1要素の配列

// Runeが期待する形式
Hash::type_hash(&["test1_1", "__start__"])  ← 2要素の配列
```

**理由**: Runeのエントリーポイントは「モジュール名」と「関数名」の**配列**で解決される。
- 生成されたRuneコード: `pub mod test1_1 { pub fn __start__(...) {...} }`
- 正しいパス: `["test1_1", "__start__"]`
- 間違ったパス: `["test1_1::__start__"]`（このパスは存在しない）

### fn_nameの形式

**グローバルラベル**:
```rust
// label_registry.rs:81
let fn_name = format!("{}_{}::__start__", sanitize_name, counter);
// 例: "test1_1::__start__"
```

**ローカルラベル**:
```rust
// label_registry.rs:128-132
let fn_name = format!("{}_{}::{}_{}",
    sanitize_name(parent), parent_counter,
    sanitize_name(name), counter);
// 例: "parent_1::local_1"
```

いずれも`split("::")`で分割すると2要素になる。

### 既存テストコードの証拠

```rust
// test_rune_metadata.rs:52
vm.execute(["test_mod", "function_a"], ())  // ← 2要素の配列

// simple_rune_test.rs:29
vm.call(rune::Hash::type_hash(&["main"]), ())?  // ← 1要素の配列（トップレベル関数）
```

### 失敗しているテストの共通点

- 複数エンジンインスタンス作成
- スレッド間でのエンジン使用
- `execute_label()`呼び出し時にMissingEntryHash

### 無効化テストの情報

1. **end_to_end_simple_test.rs:70**
   - 理由: "need to implement generator support"
   - 調査必要: generatorサポートの実装状況

2. **engine_two_pass_test.rs:31**
   - 理由: "test-project has encoding issues"
   - 調査必要: encodingの問題解決状況

3. **engine_two_pass_test.rs:58**
   - 理由: "Ignore until we can test execution"
   - 調査必要: executionテストが可能か

これらは全て本仕様で解決する。
