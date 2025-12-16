# Implementation Tasks: pasta-engine-doctest-fix

## Overview

要件定義が不十分なまま実装された関数群とdoctestを削除する。

**推定総工数**: 1-2時間

---

## Task 1: 不要関数の削除

### 1.1 find_event_handlers() の削除

**Description**: `find_event_handlers()` メソッドとそのdoctestを削除する。

**Location**: `crates/pasta/src/engine.rs` 約Line 450-459

**Requirements**: REQ-1

**Acceptance Criteria**:
- `find_event_handlers()` メソッドが削除されている
- doctest（Line 435-449）が削除されている
- コンパイルエラーが発生しない

---

### 1.2 on_event() の削除

**Description**: `on_event()` メソッドとそのdoctestを削除する。

**Location**: `crates/pasta/src/engine.rs` 約Line 490-509

**Requirements**: REQ-1

**Acceptance Criteria**:
- `on_event()` メソッドが削除されている
- doctest（Line 478-489）が削除されている
- コンパイルエラーが発生しない

---

### 1.3 execute_label_chain() の削除

**Description**: `execute_label_chain()` メソッドとそのdoctestを削除する。

**Location**: `crates/pasta/src/engine.rs` 約Line 566-600

**Requirements**: REQ-1

**Acceptance Criteria**:
- `execute_label_chain()` メソッドが削除されている
- doctest（Line 550-565）が削除されている
- コンパイルエラーが発生しない

---

## Task 2: PastaEngine構造体のdoctest削除

### 2.1 不適切なdoctestの削除

**Description**: `PastaEngine` 構造体の存在しないAPIを前提とするdoctestを削除する。

**Location**: `crates/pasta/src/engine.rs` Line 33-51

**Requirements**: REQ-2

**Acceptance Criteria**:
- 構造体のdoctest例が削除されている
- `cargo test --doc --package pasta` がパスする

---

## Task 3: 関連テストの削除

### 3.1 テストコードの調査

**Description**: 削除した3関数を参照するテストコードを検索する。

**Requirements**: REQ-3

**Commands**:
```powershell
cd crates\pasta
grep -r "find_event_handlers" tests\
grep -r "on_event" tests\
grep -r "execute_label_chain" tests\
```

**Acceptance Criteria**:
- 削除対象のテストファイルリストが作成されている

---

### 3.2 テストコードの削除

**Description**: 該当するテストコードを削除する。

**Requirements**: REQ-3

**Acceptance Criteria**:
- 削除した関数を参照するテストが全て削除されている
- `cargo test --package pasta` がパスする

---

## Task 4: ビルド・テスト検証

### 4.1 ビルド確認

**Description**: 変更後のコードがビルドできることを確認する。

**Requirements**: All

**Commands**:
```powershell
cd crates\pasta
cargo build
cargo build --release
```

**Acceptance Criteria**:
- ビルドが成功する
- 警告が発生しない

---

### 4.2 テスト実行

**Description**: 全テストがパスすることを確認する。

**Requirements**: All

**Commands**:
```powershell
cd crates\pasta
cargo test
cargo test --doc
```

**Acceptance Criteria**:
- 全単体テストがパスする
- 全doctestがパスする（残ったdoctestのみ）

---

## Task Dependencies

```
Task 1 (関数削除)
    ├─> Task 1.1 find_event_handlers()
    ├─> Task 1.2 on_event()
    └─> Task 1.3 execute_label_chain()
         │
         v
Task 2 (doctest削除)
    └─> Task 2.1 PastaEngine doctest
         │
         v
Task 3 (テスト削除)
    ├─> Task 3.1 調査
    └─> Task 3.2 削除
         │
         v
Task 4 (検証)
    ├─> Task 4.1 ビルド
    └─> Task 4.2 テスト
```

---

## Estimated Effort

| Task | Subtasks | Estimated Time |
|------|----------|----------------|
| 1 | 関数削除 | 20分 |
| 2 | doctest削除 | 10分 |
| 3 | テスト削除 | 30分 |
| 4 | 検証 | 10分 |
| **Total** | | **1-2時間** |
