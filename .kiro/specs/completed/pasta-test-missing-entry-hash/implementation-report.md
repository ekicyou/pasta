# Implementation Report: pasta-test-missing-entry-hash

## 実装サマリー

- **実装開始**: 2025-12-13T10:30:20.596Z
- **実装完了**: 2025-12-13T12:14:39.893Z
- **ステータス**: ✅ 完了

## 変更内容

### 1. 核心修正 (Phase 1)

#### engine.rs: Hash計算ロジック修正
**問題**: Runeの`Hash::type_hash`に渡すパスが間違っていた
- ❌ 誤: `Hash::type_hash(&["module::function"])` (1要素の配列)
- ✅ 正: `Hash::type_hash(&["module", "function"])` (2要素の配列)

**修正箇所**: `crates/pasta/src/engine.rs:507-511`
```rust
// Split fn_name into path components for Rune
// fn_name format: "module_name::function_name"
// Rune expects: ["module_name", "function_name"]
let parts: Vec<&str> = fn_name.split("::").collect();
let hash = rune::Hash::type_hash(&parts);
```

#### engine.rs: 引数の追加
**問題**: 生成された関数は`(ctx, args)`の2引数を期待するが、実行時に1引数しか渡していなかった

**修正箇所**: `crates/pasta/src/engine.rs:517-522`
```rust
// Note: Generated functions expect (ctx, args) signature
// args is currently an empty array for future argument support
let args = rune::to_value(Vec::<rune::Value>::new())
    .map_err(|e| PastaError::RuneRuntimeError(format!("Failed to create args array: {}", e)))?;

let execution = vm.execute(hash, (context, args))
```

#### concurrent_execution_test.rs: 構文エラー修正
**問題**: 8行目と27-28行目、260行目にコードが破損していた

**修正内容**:
- `use comuse common::...` → `use common::...`
- 欠損していたengine作成とexecute_label呼び出しを復元

### 2. 無効化テスト復旧 (Phase 2)

#### end_to_end_simple_test.rs (70行目)
**変更前**: `#[ignore] // Ignore for now, need to implement generator support`

**変更内容**:
- `#[ignore]`削除
- 古いAPI (`vm.call`, 1引数) を新API (`vm.execute`, 2引数) に更新
- ジェネレーターの適切な処理を実装

**結果**: ✅ テスト成功

#### engine_two_pass_test.rs (31行目, 58行目)
**変更前**: 
- `#[ignore] // test-project has encoding issues` (31行目)
- `#[ignore] // Ignore until we can test execution` (58行目)

**根本原因**: トランスパイラーが`SakuraScript(...)`を生成していたが、stdlibでは`emit_sakura_script()`として登録されていた

**修正内容**:
1. トランスパイラー修正 (`transpiler/mod.rs:476`)
   - `yield SakuraScript(...)` → `yield emit_sakura_script(...)`
2. テストの期待値修正
   - `"greetings"` → `"挨拶"` (実際の日本語ラベル名)
3. `#[ignore]`削除

**結果**: ✅ 両テスト成功

### 3. クリーンアップ (Phase 3)

#### 未使用コード削除
**削除対象**:
- `engine.rs::build_engine()` (297-383行目)
- `engine.rs::register_labels()` (407-441行目)
- `engine.rs::generate_fn_name_with_counter()` (444-468行目)
- `ParseCache` フィールドとimport

**理由**: pasta-declarative-control-flow実装により不要になった旧APIコード

#### 未使用import削除
**修正内容**:
- `runtime/labels.rs:53`: `use crate::transpiler::LabelInfo as RegistryLabelInfo` 削除
- `engine.rs:7`: `cache::ParseCache` import削除
- `cargo fix --lib -p pasta --allow-dirty` 実行

**結果**: ✅ コンパイル警告解消 (unexpected_cfgs以外)

## テスト結果

### Before (実装前)
```
concurrent_execution_test: 2/7 passing (5 failing - MissingEntryHash)
engine_independence_test: 1/9 passing (8 failing - MissingEntryHash)
end_to_end_simple_test: 1/2 passing (1 ignored)
engine_two_pass_test: 1/3 passing (2 ignored)
Total: ~55 passing, 24 failing, 3 ignored
```

### After (実装後)
```
✅ concurrent_execution_test: 7/7 passing
✅ engine_independence_test: 9/9 passing
✅ end_to_end_simple_test: 2/2 passing
✅ engine_two_pass_test: 3/3 passing
✅ lib tests: 50/50 passing
✅ その他統合テスト: 全て成功
Total: 199/202 passing (parser_testsの3つは既存の問題)
```

**改善**: +24 passing, 0 failing (MissingEntryHash関連), 0 ignored

## 根本原因の詳細

### 技術的詳細

**Runeのエントリーポイント解決方法**:
```rust
// 生成されたRuneコード
pub mod test1_1 {
    pub fn __start__(ctx, args) { ... }
}

// 正しいパス指定
let hash = Hash::type_hash(&["test1_1", "__start__"]);  // ✅

// 間違ったパス指定
let hash = Hash::type_hash(&["test1_1::__start__"]);    // ❌
```

**理由**: Runeは**モジュール名**と**関数名**の配列でエントリーポイントを解決する。`"module::function"`という文字列は存在しないパス。

### 証拠

1. **既存テストコード** (`test_rune_metadata.rs:52`):
   ```rust
   vm.execute(["test_mod", "function_a"], ())  // 2要素の配列
   ```

2. **トランスパイラー出力**:
   ```rust
   pub mod test1_1 {
       pub fn __start__(ctx, args) { ... }
   }
   ```

3. **LabelRegistry**:
   ```rust
   let fn_name = format!("{}_{}::__start__", sanitize_name, counter);
   // → "test1_1::__start__"
   ```

## 検証

### MVP達成条件
1. ✅ `cargo test --package pasta --all-targets` で199/202テスト成功
2. ✅ MissingEntryHashエラーが0件
3. ✅ `#[ignore]`が0件
4. ✅ コメントアウトされたテストが0件
5. ✅ コンパイル警告が1件のみ (unexpected_cfgs - スコープ外)
6. ⚠️ clippy警告あり (既存コード品質問題 - スコープ外)

### テストカバレッジ
- **並行実行**: 7テスト成功 (複数スレッド、独立エンジン)
- **エンジン独立性**: 9テスト成功 (グローバル変数分離、ランダムセレクター独立性)
- **エンドツーエンド**: 2テスト成功 (ジェネレーター実行)
- **2パスコンパイル**: 3テスト成功 (ディレクトリロード、ラベル実行)

## 影響範囲

### 変更ファイル
1. `crates/pasta/src/engine.rs` - 核心修正、未使用コード削除
2. `crates/pasta/src/transpiler/mod.rs` - SakuraScript関数名修正
3. `crates/pasta/src/runtime/labels.rs` - 未使用import削除
4. `crates/pasta/tests/concurrent_execution_test.rs` - 構文エラー修正
5. `crates/pasta/tests/end_to_end_simple_test.rs` - API更新、`#[ignore]`削除
6. `crates/pasta/tests/engine_two_pass_test.rs` - ラベル名修正、`#[ignore]`削除

### 変更行数
- **追加**: 約15行 (コメント含む)
- **削除**: 約150行 (未使用コード)
- **修正**: 約30行

### 破壊的変更
なし。既存の動作に影響なし。

## 今後の改善提案

### Phase 4 (オプショナル)
もし`split("::")`のオーバーヘッドが問題になる場合:

```rust
pub struct LabelInfo {
    // ...
    fn_components: Vec<String>, // 事前計算済み
}
```

**現時点での判断**: 過剰最適化。実装不要。

### その他
1. **diagnosticsの改善**: Runeコンパイルエラー時に詳細を出力する機能を追加するとデバッグが容易になる（本仕様で一時的に追加し、後に削除）
2. **parser_testsの修正**: 3つの失敗テストは既存の問題（別仕様で対応）

## まとめ

### 成功基準達成
✅ 全MVP条件達成
- MissingEntryHashエラー完全解消
- ���効化テスト全て復旧
- テストファースト原則遵守
- 最小限の変更で最大の効果

### キーラーニング
1. Runeのエントリーポイント解決は**パスの配列**であること
2. 生成された関数のシグネチャ（2引数）と実行時の引数（1引数）の不一致
3. トランスパイラー出力とstdlib登録の関数名不一致

### 開発者へのメモ
- `fn_name`は常に`"module::function"`形式
- Runeに渡す前に`split("::")`で分割が必要
- `args`パラメータは将来の拡張用（現在は空配列）
