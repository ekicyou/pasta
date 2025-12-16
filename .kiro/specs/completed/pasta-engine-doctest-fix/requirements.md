# Requirements Document: pasta-engine-doctest-fix

| 項目 | 内容 |
|------|------|
| **Document Title** | PastaEngine 不要関数削除 要件定義書 |
| **Version** | 2.0 |
| **Date** | 2025-12-14 |
| **Priority** | P2 (Code Quality) |
| **Status** | Requirements Updated |

---

## Introduction

`crates/pasta/src/engine.rs` に要件定義が不十分なまま実装された関数群とdoctestを削除する。

### Background

**pasta-transpiler-pass2-output** 仕様の実装中に、以下の問題を発見：
1. 要件定義不十分な関数が実装されている
2. doctestが存在しないAPI（文字列ベースの`new(script)`）を前提としている

### Problem Statement

**課題1: 要件定義不十分な実装**

以下の関数が要件定義不十分なまま実装されている：
- `find_event_handlers(event_name)` - イベントハンドラ検索
- `on_event(event_name, params)` - イベント実行
- `execute_label_chain(initial_label, max_depth)` - ラベルチェーン実行

**課題2: 不適切なdoctest**

以下のdoctestが存在しないAPI（`new(script: &str)`）を前提としている：
1. `crates/pasta/src/engine.rs:35` - `PastaEngine` 構造体のドキュメント
2. `crates/pasta/src/engine.rs:435` - `find_event_handlers()` メソッド
3. `crates/pasta/src/engine.rs:478` - `on_event()` メソッド
4. `crates/pasta/src/engine.rs:550` - `execute_label_chain()` メソッド

**課題3: テストの失敗**

```
error[E0061]: this function takes 2 arguments but 1 argument was supplied
```

### Scope

**含まれるもの：**

1. **関数の削除**
   - `find_event_handlers()` - 要件定義不十分
   - `on_event()` - 要件定義不十分
   - `execute_label_chain()` - 要件定義不十分

2. **Doctestの削除**
   - `PastaEngine` 構造体のdoctest（存在しないAPIを前提）
   - 上記3関数のdoctest

3. **関連テストの削除**
   - 削除する3関数のテストコード

**含まれないもの：**

- `execute_label()` - テスト用便利関数として残す（doctestは削除）
- `execute_label_with_filters()` - 内部実装として残す
- APIの変更

---

## Requirements

### Requirement 1: 不要関数の削除

**Objective:** 要件定義が不十分なまま実装された関数を削除する。

#### Acceptance Criteria

1. When `find_event_handlers()` を削除する, the Code shall コンパイルエラーが発生しない
2. When `on_event()` を削除する, the Code shall コンパイルエラーが発生しない
3. When `execute_label_chain()` を削除する, the Code shall コンパイルエラーが発生しない

#### 削除対象

**Location:** `crates/pasta/src/engine.rs`

```rust
// 削除: find_event_handlers()
pub fn find_event_handlers(&self, event_name: &str) -> Vec<String> { ... }

// 削除: on_event()
pub fn on_event(&mut self, event_name: &str, params: HashMap<String, String>) -> Result<Vec<ScriptEvent>> { ... }

// 削除: execute_label_chain()
pub fn execute_label_chain(&mut self, initial_label: &str, max_chain_depth: usize) -> Result<Vec<ScriptEvent>> { ... }
```

---

### Requirement 2: 不適切なDoctestの削除

**Objective:** 存在しないAPI（`new(script: &str)`）を前提とするdoctestを削除する。

#### Acceptance Criteria

1. When `PastaEngine` 構造体のdoctestを削除する, the Doctest shall コンパイルエラーが発生しない
2. When 削除した3関数のdoctestを削除する, the Doctest shall コンパイルエラーが発生しない
3. When `cargo test --doc --package pasta` を実行する, the Test shall 全てパスする

#### 削除対象

**Location:** `crates/pasta/src/engine.rs`

1. Line 35-51: `PastaEngine` 構造体のdoctest
2. Line 435-449: `find_event_handlers()` のdoctest
3. Line 478-489: `on_event()` のdoctest
4. Line 550-565: `execute_label_chain()` のdoctest

---

### Requirement 3: 関連テストの削除

**Objective:** 削除する関数の単体テストを削除する。

#### Acceptance Criteria

1. When 関連テストを削除する, the Test Suite shall コンパイルエラーが発生しない
2. When `cargo test --package pasta` を実行する, the Test Suite shall 全テストがパスする

#### 調査対象

- `crates/pasta/tests/**/*.rs` 内の関連テスト
- `find_event_handlers`, `on_event`, `execute_label_chain` を参照するテストコード

---

## Technical Context

### 削除する関数の実装箇所

**Location:** `crates/pasta/src/engine.rs`

1. **find_event_handlers()** - 約Line 450-459
   - イベント名パターンマッチング
   - ラベル名リストを返す

2. **on_event()** - 約Line 490-509
   - イベントハンドラを検索して実行
   - `execute_label_with_filters()` を内部呼び出し

3. **execute_label_chain()** - 約Line 566-600
   - ラベルチェーン実行ロジック
   - `execute_label()` を繰り返し呼び出し

### 残す関数

- `execute_label()` - テスト用便利関数
- `execute_label_with_filters()` - 内部実装（フィルタ付きラベル実行）

---

## Implementation Notes

### 作業手順

1. **Phase 1: 関数削除**
   - `find_event_handlers()` を削除
   - `on_event()` を削除
   - `execute_label_chain()` を削除

2. **Phase 2: Doctest削除**
   - `PastaEngine` 構造体のdoctest（Line 35-51）を削除
   - 削除した3関数のdoctestも自動的に削除される

3. **Phase 3: 関連テスト調査・削除**
   - `grep -r "find_event_handlers\|on_event\|execute_label_chain" tests/` で検索
   - 該当テストを削除

4. **Phase 4: ビルド確認**
   - `cargo test --doc --package pasta`
   - `cargo test --package pasta`

### 影響範囲

- **変更ファイル**: `crates/pasta/src/engine.rs`, テストファイル複数
- **削除行数**: 約150-200行（関数本体 + doctest + 単体テスト）
- **影響**: 要件定義不十分な機能を削除

---

## Testing Strategy

### 検証項目

| テストケース | 期待結果 |
|-------------|---------|
| `cargo build --package pasta` | ビルド成功 |
| `cargo test --doc --package pasta` | 全doctest pass |
| `cargo test --package pasta` | 全テストpass |
| 削除した関数への参照 | コンパイルエラーなし |

---

## References

- **発見元:** `.kiro/specs/pasta-transpiler-pass2-output/`
- **関連タスク:** `.kiro/specs/completed/areka-P0-script-engine/tasks.md` Task 7（リジェクト対象）
- **ソースコード:** `crates/pasta/src/engine.rs`
