# Design: pasta-test-missing-entry-hash

## 根本原因の特定

### 問題の核心

**Runeのエントリーポイント解決方法とfn_nameの不一致**

#### 現在の実装

```rust
// engine.rs:508
let fn_name = self.label_table.find_label(label_name, filters)?;
// fn_name = "test1_1::__start__"

let hash = rune::Hash::type_hash(&[fn_name.as_str()]);
// hash = Hash::type_hash(&["test1_1::__start__"])
//      = Hash of ["test1_1::__start__"] ← 1要素の配列
```

#### Runeが期待する形式

```rust
// Runeのエントリーポイントは**パスの配列**
vm.execute(["module_name", "function_name"], args)
vm.execute(["test1_1", "__start__"], (ctx,))

// Hashの計算
Hash::type_hash(&["test1_1", "__start__"])
// ↑ 2要素の配列
```

### 証拠

1. **テストコードの使用例**:
```rust
// test_rune_metadata.rs:52
vm.execute(["test_mod", "function_a"], ())
```

2. **生成されたRuneソース**:
```rust
pub mod test1_1 {
    pub fn __start__(ctx, args) {
        // ...
    }
}
```

エントリーポイントは`test1_1::__start__`ではなく、**モジュール`test1_1`の関数`__start__`**。

3. **現在のfn_name**:
```rust
// label_registry.rs:81
let fn_name = format!("{}_{}::__start__", Self::sanitize_name(name), counter);
// → "test1_1::__start__"
```

これを**文字列として**`type_hash`に渡すと、Runeは`["test1_1::__start__"]`というパスを探す。しかし、実際には`["test1_1", "__start__"]`というパスにしか登録されていない。

## 設計方針

### アプローチ1: fn_nameを分割してHashを計算 (採用)

**メリット**:
- 既存のfn_name形式を保持
- label_registryの変更が不要
- 最小限の修正

**実装**:
```rust
// engine.rs
let fn_name = self.label_table.find_label(label_name, filters)?;
// fn_name = "test1_1::__start__"

// Split into path components
let parts: Vec<&str> = fn_name.split("::").collect();
let hash = rune::Hash::type_hash(&parts);
// hash = Hash::type_hash(&["test1_1", "__start__"])
```

### アプローチ2: fn_nameの形式を変更

**デメリット**:
- label_registryの大規模な変更が必要
- fn_pathとfn_nameの区別が曖昧になる
- 影響範囲が大きい

**不採用理由**: 最小限の修正原則に反する

### アプローチ3: 新しいfn_componentsフィールドを追加

**デメリット**:
- データ構造の変更が必要
- メモリ使用量が増加
- 冗長性が高い

**不採用理由**: 過剰設計

## 詳細設計

### 変更対象ファイル

#### 1. `crates/pasta/src/engine.rs`

**変更箇所**: `execute_label_with_filters`メソッド

```rust
pub fn execute_label_with_filters(
    &mut self,
    label_name: &str,
    filters: &HashMap<String, String>,
) -> Result<Vec<ScriptEvent>> {
    // Look up the label
    let fn_name = self.label_table.find_label(label_name, filters)?;

    // Create a new VM for this execution
    let mut vm = Vm::new(self.runtime.clone(), self.unit.clone());

    // Split fn_name into path components for Rune
    // fn_name format: "module_name::function_name"
    // Rune expects: ["module_name", "function_name"]
    let parts: Vec<&str> = fn_name.split("::").collect();
    let hash = rune::Hash::type_hash(&parts);

    // Build execution context
    let context = self.build_execution_context()?;

    // Execute and get a generator
    let execution = vm
        .execute(hash, (context,))
        .map_err(|e| PastaError::VmError(e))?;

    // ... rest of the code
}
```

### ローカルラベルの対応

ローカルラベルは親モジュール内の関数なので、すでに正しい形式になっています：

```rust
// label_registry.rs:128-132
let fn_name = format!(
    "{}_{}::{}_{}",
    Self::sanitize_name(parent_name),
    parent_counter,
    Self::sanitize_name(name),
    counter
);
// 例: "parent_1::local_1"
```

これを分割すると`["parent_1", "local_1"]`となり、正しく解決されます。

### エッジケース

#### ケース1: ネストしたモジュール

現在の実装ではネストは2階層まで：
- グローバルラベル: `module::__start__`
- ローカルラベル: `parent_module::local_function`

分割は常に`split("::")`で正しく処理されます。

#### ケース2: 特殊文字を含むラベル名

`sanitize_name`で既に正規化されているため、`"::"`が混入することはありません。

### テスト戦略

#### 1. 単体テスト

既存のテストがそのまま機能することを確認：
- `test_engine_execute_simple_label`
- `test_engine_multiple_labels`
- `test_label_names_api`

#### 2. 統合テスト

失敗しているテストが成功することを確認：
- `concurrent_execution_test`: 7/7 passing
- `engine_independence_test`: 9/9 passing

#### 3. 回帰テスト

全てのpastaテストが成功することを確認：
- `cargo test --package pasta --all-targets`

## 実装計画

### Phase 1: 核心修正 (優先度: 最高)

1. **engine.rsの修正**
   - `execute_label_with_filters`でfn_nameを分割
   - コメントで理由を説明

2. **ビルド確認**
   - `cargo build --package pasta`

3. **基本テスト**
   - `cargo test --package pasta --lib`

### Phase 2: テスト検証 (優先度: 高)

1. **失敗テストの実行**
   - `cargo test --package pasta --test engine_independence_test`
   - `cargo test --package pasta --test concurrent_execution_test`

2. **全テスト実行**
   - `cargo test --package pasta --all-targets`

3. **結果確認**
   - 79/79 tests passing

### Phase 3: クリーンアップ (優先度: 中)

1. **不要なコードの削除**
   - `engine.rs`の`register_labels`メソッド（未使用警告が出ている）
   - `generate_fn_name_with_counter`メソッド（未使用警告が出ている）
   - `build_engine`メソッド（未使用警告が出ている）

2. **cargo fmt**
   - `cargo fmt --all`

3. **cargo clippy**
   - `cargo clippy --package pasta`

## リスク分析

### 高リスク

**なし**

理由: 修正箇所が1メソッドの1行のみで、影響範囲が明確。

### 中リスク

**パフォーマンス影響**

- `split("::")`と`collect()`によるオーバーヘッド
- 1ラベル実行あたり数マイクロ秒程度

**緩和策**: 
- 実行頻度が高い場合、LabelInfoに事前計算した`Vec<String>`を持たせる（Phase 4で検討）

### 低リスク

**他のRuneバージョンへの互換性**

- Runeのエントリーポイント解決方法が変更される可能性
- 現時点では問題なし

## 検証基準

### 必達条件

1. ✅ `cargo test --package pasta --all-targets`が全て成功
2. ✅ MissingEntryHashエラーが発生しない
3. ✅ 既存テスト（55 tests）が全て成功
4. ✅ 新規成功テスト（24 tests）が追加で成功

### 推奨条件

1. ✅ `cargo fmt`が成功
2. ✅ `cargo clippy`が警告なし
3. ✅ 不要コードが削除されている

## 今後の改善

### Phase 4: パフォーマンス最適化（オプション）

もし`split("::")`のオーバーヘッドが問題になる場合：

```rust
// LabelInfo に追加
pub struct LabelInfo {
    // ...
    fn_components: Vec<String>, // 事前計算済み
}

// LabelTable::from_label_registry で設定
let runtime_info = LabelInfo {
    // ...
    fn_components: registry_info.fn_name
        .split("::")
        .map(|s| s.to_string())
        .collect(),
};

// engine.rs で使用
let hash = rune::Hash::type_hash(&fn_components.iter().map(|s| s.as_str()).collect::<Vec<_>>());
```

ただし、現時点では不要（過剰最適化）。

## まとめ

### 根本原因

Runeの`Hash::type_hash`に渡すパスが間違っていた：
- **誤**: `&["module::function"]` (1要素)
- **正**: `&["module", "function"]` (2要素)

### 解決方法

`fn_name`を`split("::")`で分割してからHashを計算する。

### 影響範囲

- 変更ファイル: 1ファイル (engine.rs)
- 変更箇所: 1メソッド (execute_label_with_filters)
- 変更行数: 2-3行

### 期待される結果

- ✅ 全79テスト成功
- ✅ MissingEntryHashエラー解消
- ✅ 並行実行テスト成功
- ✅ 独立実行テスト成功
