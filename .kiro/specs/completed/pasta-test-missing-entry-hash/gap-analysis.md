# Gap Analysis: pasta-test-missing-entry-hash

## 現状

### テスト結果サマリー

```
Total: 00 tests
✅ Passing: 0 tests
❌ Failing: 0 tests
```

### 失敗テスト一覧

**concurrent_execution_test (5/7 failing)**:
- test_thread_safety
- test_multiple_threads_same_script
- test_no_data_races
- test_independent_execution_across_threads
- test_thread_local_cache

**engine_independence_test (8/9 failing)**:
- test_independent_execution
- test_independent_label_execution
- test_engine_with_different_scripts
- test_random_selector_independence
- test_independent_parsing
- test_global_variable_isolation
- test_event_handler_independence
- test_drop_independence

### エラーパターン

```rust
VmError(VmError { 
  error: VmErrorAt { 
    index: 0, 
    kind: MissingEntryHash { hash: 0x... } 
  }
})
```

## ギャップ分析

### 1. ラベルエントリーポイント管理の不整合

**現状**:
- `Transpiler::transpile_with_registry()`でLabelRegistryを生成
- `LabelTable::from_label_registry()`でラベルテーブルに変換
- `fn_name`フィールドをRuneの関数名として使用

**問題**:
- Runeが期待するエントリーポイント名と実際のfn_nameが一致していない可能性
- Hashの計算方法が不明確

**調査必要**:
- [x] fn_nameの生成ロジック確認
- [ ] Runeのエントリーポイント登録確認
- [ ] Hash計算とfn_nameの対応確認

### 2. 複数エンジンインスタンスの挙動

**現状**:
- 各エンジンが独立したunitとlabel_tableを持つ
- `Arc<Unit>`で共有はしているが、label_tableは独立

**問題**:
- 同じscript_dirから複数エンジンを作成すると失敗
- 単一エンジンでは成功する

**調査必要**:
- [ ] unitの初期化タイミング
- [ ] label_tableとunitの同期
- [ ] スレッド間でのunit共有の影響

### 3. テストインフラの完全性

**現状**:
- `create_test_script()`で一意のディレクトリ作成 ✅
- `create_unique_persistence_dir()`で一意の永続化ディレクトリ作成 ✅

**問題**:
- 上記が正しく動作していてもMissingEntryHashが発生

**結論**:
- テストインフラは問題なし
- エンジン本体の問題

## 次のステップ

### Phase 1: 原因特定 (優先度: 最高)

1. **デバッグログ追加**
   - `LabelTable::find_label()`で返すfn_name
   - Runeのunitに登録されているエントリーポイント一覧
   - Hashの計算過程

2. **最小再現テスト作成**
   - 単一ファイルで問題を再現
   - デバッグしやすい形に整理

3. **Runeドキュメント調査**
   - エントリーポイントの登録方法
   - Hashの計算方法
   - 関数名の解決ロジック

### Phase 2: 修正実装 (優先度: 高)

1. **fn_name/fn_pathの修正**
   - 正しいエントリーポイント名を使用
   - 必要に応じてfn_pathを使う

2. **label_tableの修正**
   - Runeが期待する名前を返す
   - 必要に応じて変換ロジック追加

3. **テスト実行**
   - 修正後に全テスト実行
   - 問題が解決したか確認

### Phase 3: 検証・文書化 (優先度: 中)

1. **回帰テスト**
   - 既存のテストが全て成功することを確認
   - パフォーマンス劣化がないか確認

2. **根本原因の文書化**
   - 問題の原因を明確に記述
   - 今後の参考にする

## リスク

### 高リスク

1. **Runeの内部実装依存**
   - Runeのバージョンアップで動作が変わる可能性
   - ドキュメントが不十分

2. **大規模なアーキテクチャ変更が必要**
   - 現在の設計では解決できない可能性
   - リファクタリングに時間がかかる

### 中リスク

1. **他の箇所への影響**
   - 修正により既存の動作テストが壊れる
   - 慎重な修正が必要

## 期待される成果

1. ✅ 全テスト成功（79 tests passing）
2. ✅ MissingEntryHashエラーの根絶
3. ✅ 根本原因の明確化
4. ✅ 今後の保守性向上
