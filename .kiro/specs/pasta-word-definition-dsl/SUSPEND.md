# Design Review Progress

## Status: SUSPENDED

設計検証中にRune Call/Jump文のyield伝搬問題が発覚したため、本仕様は一時中断。

## Completed Work

### Issue 1: Rune関数実装詳細 ✅
- **決定**: Option A（Rust stdlib実装）
- **理由**: ユニットテスト容易性、Pastaコア機能の責任範囲
- **実装**: `pasta_stdlib::search_word_dict` 関数
- **テスト要件**: Runeからの関数呼び出し検証を含む
- **更新**: design.md に実装方針を追加済み

### Issue 2: Trie→Rune受け渡し方法 ⏸️
- **ブロック理由**: PastaContext設計とCall/Jump文のyield伝搬問題が前提条件
- **議論内容**:
  - `ctx` 引数の設計（actor, scope, save）
  - Trieのグローバル保持方式
  - ローカルスコープの概念整理
- **保留事項**: 
  - Transpiler修正（Call/Jump文にctx引数を渡す）
  - PastaContextの構造定義
  - Trie保持方式の最終決定

### Issue 3: ParsePhase状態管理 ✅
- **実装**: design.md に追加済み
- **内容**: ParsePhase enum（Declaration/Execution）、Parser状態管理メソッド
- **根拠**: gap-analysis.md 方式Cの決定事項を反映

## Blocking Issue

**pasta-yield-propagation** 仕様を先に解決する必要あり：
- Runeはネストした関数内のyieldを透過的に返さない
- Call文（`＞label`）で呼び出し先のyieldが失われる
- Jump文（`－label`）も同様の問題の可能性
- PastaContext設計とTranspiler修正の前提条件

## Next Steps (After Unblocking)

1. pasta-yield-propagationの要件定義・設計・実装
2. 本仕様に戻り、Issue 2を完了
3. design.mdの最終更新とコミット
4. 設計承認後、タスク生成へ

---

**Suspended**: 2025-12-11
**Reason**: Dependency on pasta-yield-propagation
