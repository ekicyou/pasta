# Test Fixtures

このディレクトリには、Pasta DSL トランスパイラーのテスト用ファイルが含まれています。

## ファイル構成

### Comprehensive Control Flow Tests

#### `comprehensive_control_flow.pasta`
包括的なコントロールフローのテストケース。すべての主要機能を含みます：
- グローバルラベル、ローカルラベル
- Call文、Jump文
- 単語定義と展開
- 変数代入
- 発言者切り替え
- Runeブロック

**重要**: このファイルは**同名ラベルを使用していない**ため、P0実装（完全一致検索のみ）で完全にサポート可能です。

#### `comprehensive_control_flow.rn`
**テスト用のクリーンバージョン**。トランスパイラーが実際に出力する形式に厳密一致。
- コメントなし
- トランスパイラーが生成するコードのみ
- `assert_eq!` による厳密比較用

#### `comprehensive_control_flow.annotated.rn`
**人間用の注記付きバージョン**（オリジナル）。
- 詳細な説明コメント付き
- 各セクションの説明
- 統一パターンの注記
- リファレンス実装として保存

**用途**: 
- トランスパイラー実装時の参考
- 期待される動作の理解
- 不正な改変の証跡として保存

### Simple Control Flow Tests

#### `comprehensive_control_flow_simple.pasta`
最もシンプルなテストケース（Task 1.1で作成）。
- グローバルラベルと発言行のみ
- パーサーの基本機能の検証用

#### `comprehensive_control_flow_simple.expected.rn`
simple版の期待される出力（Task 1.2で作成）。

## テスト戦略

### Phase 1（基礎テスト）
`comprehensive_control_flow_simple.pasta` を使用して、基本的なモジュール構造生成を検証。

### Phase 8（最終検証）
`comprehensive_control_flow.pasta` を使用して、すべての機能の統合テストを実施。

**必達条件**: 
```
comprehensive_control_flow.pasta → comprehensive_control_flow.rn
```
トランスパイル結果が期待される出力と厳密一致すること（`assert_eq!`）。

## ファイル管理の注意事項

### 改変の禁止
`comprehensive_control_flow.rn`（クリーンバージョン）は、**テストを通すために内容を変更してはいけません**。

トランスパイラーの出力が期待と異なる場合：
1. ❌ `.rn` ファイルを修正してテストを通す
2. ✅ トランスパイラーを修正して正しい出力を生成する

### 証跡の保存
`comprehensive_control_flow.annotated.rn` は、オリジナルの期待値として保存されています。
- 不正な改変が行われた場合の証拠
- 実装時の参考資料
- リファレンス実装の記録

## P0 vs P1 スコープ

### P0範囲（本タスクリスト）
- 完全一致ラベル解決
- 同名ラベルなし
- `comprehensive_control_flow.pasta` の完全サポート ✅

### P1範囲（別仕様）
- 前方一致検索
- **同名ラベル**のランダム選択
- 属性フィルタリング
- キャッシュベース消化
