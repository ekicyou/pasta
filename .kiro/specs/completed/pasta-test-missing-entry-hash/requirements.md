# Requirements: pasta-test-missing-entry-hash

## 概要

pastaエンジンのテストで発生している`MissingEntryHash`エラーの原因を調査し、修正する。

## 背景

pasta-declarative-control-flow仕様の実装後、以下のテストが失敗している：

- **concurrent_execution_test**: 5/7 tests failing
- **engine_independence_test**: 8/9 tests failing

### エラー詳細

```
VmError(VmError { 
  error: VmErrorAt { 
    index: 0, 
    kind: MissingEntryHash { hash: 0x... } 
  }, 
  chain: [], 
  stacktrace: [] 
})
```

## 問題の症状

### ✅ 成功しているテスト

- 単一エンジン、単一ラベル実行
- test_send_trait
- test_concurrent_engine_creation
- test_empty_script
- test_error_handling_invalid_label

### ❌ 失敗しているテスト

**共通パターン**:
- 複数のエンジンインスタンスを作成
- または複数のスレッドでエンジンを実行
- ラベルを`execute_label()`で実行しようとすると失敗

**失敗例**:
```rust
// test_thread_safety
let handle1 = thread::spawn(move || {
    let script_dir = create_test_script(script1).expect("...");
    let persistence_dir = create_unique_persistence_dir().expect("...");
    let mut engine = PastaEngine::new(&script_dir, &persistence_dir).expect("...");
    engine.execute_label("test1").expect("..."); // ← MissingEntryHash
});
```

## 調査ポイント

### 1. ラベルテーブルとRuneの不整合

**仮説**: LabelTableとRune VMのunitに登録されたエントリーポイントが一致していない

**確認事項**:
- `LabelTable::from_label_registry()`の実装が正しいか
- `fn_name`と`fn_path`の使い分けが正しいか
- Runeの`Hash`計算方法とラベルテーブルの対応

### 2. 並行実行時の状態管理

**仮説**: Runeのコンパイル結果やunitが複数スレッド/インスタンス間で共有されている

**確認事項**:
- `Arc<Unit>`の使い方は正しいか
- `Runtime`の状態管理は正しいか
- `create_test_script()`でディレクトリが分離されているか

### 3. LabelRegistryの生成タイミング

**仮説**: トランスパイル時とエンジン初期化時でラベル登録順序が異なる

**確認事項**:
- `Transpiler::transpile_with_registry()`の実装
- ラベルIDの採番ロジック
- ラベル名の正規化ロジック

## 要求事項

### 機能要件

1. **MissingEntryHashエラーの根本原因を特定**
   - デバッグログを追加して原因を突き止める
   - ラベルハッシュとエントリーポイントの対応を確認

2. **全テストを成功させる**
   - concurrent_execution_test: 7/7 passing
   - engine_independence_test: 9/9 passing
   - その他の既存テストが壊れないこと

3. **再発防止策**
   - 根本原因に対する恒久的な修正
   - 必要に応じてアーキテクチャを見直し

### 非機能要件

1. **デバッグ性**
   - 問題を再現しやすいテストケース
   - 詳細なエラーメッセージ

2. **保守性**
   - ラベル管理の一貫性を保つ
   - コードの重複を削減

## 制約条件

### 技術的制約

- Rune VMの内部実装は変更できない
- 既存のAPI互換性を保つ
- パフォーマンスを劣化させない

### スコープ外

- 新機能の追加
- パフォーマンス最適化（問題修正に必要な場合を除く）
- ドキュメント整備（最小限のコメント追加のみ）

## 成功基準

1. ✅ `cargo test --package pasta --all-targets`が全て成功
2. ✅ MissingEntryHashエラーが発生しない
3. ✅ 既存の動作テスト（53+ tests）が全て成功
4. ✅ 根本原因が文書化されている

## 関連仕様

- **pasta-declarative-control-flow**: この実装が原因で問題が発生
- 本仕様はその修正を行う

## 備考

### 既知の情報

1. **トランスパイラー修正済み**:
   - actor自動import削除 ✅
   - ctx.actor文字列化 ✅
   - LabelRegistry統合 ✅

2. **テストインフラ修正済み**:
   - `create_test_script()`でdic/main.pasta作成 ✅
   - `create_unique_persistence_dir()`追加 ✅

3. **まだ動作しない**:
   - 複数エンジンインスタンスでのラベル実行
   - スレッド間でのエンジン実行

### デバッグヒント

```rust
// MissingEntryHashが発生する流れ:
// 1. engine.execute_label("test1") 呼び出し
// 2. label_table.find_label("test1", filters) → fn_name取得
// 3. Runtime::call(fn_name) → Runeで実行
// 4. Rune VMがfn_nameのHashを計算
// 5. unitに登録されたエントリーポイントを検索
// 6. 見つからない → MissingEntryHash
```

調査すべきは「unitに何が登録されているか」と「label_tableが何を返しているか」の不一致。
