# 実装タスク: parser2 FileScope複数出現バグ修正

## 概要

本タスク一覧は、parser2 モジュールのAST構造再設計と パーサーロジック修正に必要な実装作業を定義します。

- **合計**: 7 大タスク、14 サブタスク
- **要件カバレッジ**: 6 要件、21受入基準 100% カバー
- **平均タスク規模**: 1-3時間/サブタスク
- **並列実行対応**: 複数タスクが並列実行可能

---

## 実装フェーズ

### Phase 1: AST構造定義

- [x] 1. FileItem 列挙型とPastaFile構造体を定義
- [x] 1.1 (P) FileItem列挙型を定義し、3つのバリアント（FileAttr、GlobalWord、GlobalSceneScope）を実装
  - 構造体フィールド: `pub enum FileItem { FileAttr(Attr), GlobalWord(KeyWords), GlobalSceneScope(GlobalSceneScope) }`
  - `#[derive(Debug, Clone)]` を付与
  - ドキュメントコメントで grammar.pest との対応を明記
  - _Requirements: 2.1_

- [x] 1.2 (P) PastaFile 構造体に items フィールドを追加し、旧フィールドを廃止
  - 新フィールド: `pub items: Vec<FileItem>`（記述順序を保持）
  - 廃止フィールド: `file_scope: FileScope` と `global_scenes: Vec<GlobalSceneScope>`
  - PastaFile::new() コンストラクタを items.Vec::new() で初期化
  - _Requirements: 2.2, 2.3, 2.4_

- [x] 1.3 (P) PastaFile にヘルパーメソッドを実装
  - メソッド: `file_attrs() -> Vec<&Attr>` （FileAttr バリアントのみ抽出）
  - メソッド: `words() -> Vec<&KeyWords>` （GlobalWord バリアントのみ抽出）
  - メソッド: `global_scene_scopes() -> Vec<&GlobalSceneScope>` （GlobalSceneScope バリアントのみ抽出）
  - 各メソッドは items イテレータで型別フィルタリングを実装
  - _Requirements: 2.5, 5.2, 5.4_

### Phase 2: パーサーロジック修正

- [x] 2. build_ast 関数を修正し、複数ファイルアイテムの累積処理を実現
- [x] 2.1 (P) build_ast の Rule::file_scope ケースを修正
  - 廃止: `file.file_scope = parse_file_scope(pair)?` の上書き代入
  - 実装: parse_file_scope() から取得した FileScope を分解
  - 実装: attrs を個別に `file.items.push(FileItem::FileAttr(attr))`で追加
  - 実装: words を個別に `file.items.push(FileItem::GlobalWord(word))`で追加
  - _Requirements: 3.1, 3.4, 1.2, 1.3, 1.6_

- [x] 2.2 (P) build_ast の Rule::global_scene_scope ケースを修正
  - 廃止: `file.global_scenes.push(scene)` 操作
  - 実装: `file.items.push(FileItem::GlobalSceneScope(scene))` で items に追加
  - _Requirements: 3.2, 3.4, 1.4_

- [x] 2.3 パーサーループで出現順序を保持確認
  - テスト: 複数 file_scope と global_scene_scope が交互出現する場合、items の順序がファイル記述順と一致することを確認
  - _Requirements: 3.3, 1.5_

### Phase 3: 既存テスト移行

- [x] 3. tests/parser2_integration_test.rs の既存テストを新AST構造に対応させ
- [x] 3.1 test_file_scope_default テストを修正
  - 廃止: `file.file_scope.attrs.is_empty()` アクセス
  - 廃止: `file.global_scenes.is_empty()` アクセス
  - 実装: `file.file_attrs().is_empty()` と `file.global_scene_scopes().is_empty()` で検証
  - _Requirements: 4.4_

- [x] 3.2 test_global_scene_parsing テストを修正
  - 廃止: `file.global_scenes[0]` への直接アクセス
  - 実装: `file.global_scene_scopes()[0]` でアクセス
  - _Requirements: 4.4_

- [x] 3.3 他のテスト4箇所を一括修正（src/parser2/mod.rs 内のテスト）
  - ターゲット: grep 結果から特定された 4箇所のフィールドアクセス
  - 修正パターン: `file.file_scope.*` → `file.file_attrs()` / `file.words()` のいずれか、`file.global_scenes` → `file.global_scene_scopes()`
  - _Requirements: 4.4_

### Phase 4: 新規テスト追加

- [x] 4. 複数ファイルアイテム出現シナリオをカバーする統合テストを追加
- [x] 4.1 複数 file_scope 単純連続パターンのテストを追加
  - テストフィクスチャ: 2つの file_scope を連続記述（attrs 各1個、words 各1個）
  - 検証: items が 4つの FileItem を正確に格納（FileAttr, GlobalWord, FileAttr, GlobalWord の順序）
  - _Requirements: 4.1, 4.2, 4.3, 1.1, 1.2, 1.3, 1.6_

- [x] 4.2 file_scope と global_scene_scope 交互出現パターンのテストを追加
  - テストフィクスチャ: file_scope → global_scene_scope → file_scope → global_scene_scope
  - 検証: items の順序が記述順と完全に一致（FileAttr, GlobalWord, GlobalSceneScope, FileAttr, GlobalWord, GlobalSceneScope）
  - _Requirements: 4.1, 4.2, 4.3, 1.5_

- [x] 4.3 単一バリアント頻出パターンのテストを追加
  - テストフィクスチャ: 複数 global_scene_scope のみ、または複数 file_scope のみ
  - 検証: items の長さと各バリアント型が期待値と一致
  - _Requirements: 4.1, 4.2, 4.3_

- [x] 4.4 パターンマッチと型判定の動作確認テストを追加
  - テスト: items をイテレートしながら match で FileItem::FileAttr / GlobalWord / GlobalSceneScope を識別
  - テスト: ヘルパーメソッド file_attrs(), words(), global_scene_scopes() が正確に型別抽出できることを確認
  - _Requirements: 4.5, 5.1_

### Phase 5: transpiler2 互換性確認

- [x] 5. PastaFile.items API が transpiler2 での順次処理に対応していることを確認
- [x] 5.1 items イテレーション処理の型安全性テストを追加
  - テスト: transpiler2 風の順次処理シミュレーション
  - テスト: items.iter() でループし、match で各バリアントを正確に判定・処理
  - _Requirements: 5.1, 5.3_

- [x] 5.2 Span情報の伝播確認テストを追加
  - テスト: items 内の各 FileItem が Span を保有していることを確認
  - テスト: エラーメッセージで行番号・列番号が正確に報告されることを検証
  - _Requirements: 5.2, 6.1_

### Phase 6: ドキュメント・移行ガイド整備

- [x] 6. PastaFile と FileItem に docコメント・移行ガイドを追加
- [x] 6.1 FileItem 列挙型にドキュメントコメントを追加
  - 内容: grammar.pest 対応関係、3つのバリアントの説明、使用例
  - _Requirements: 6.2, 6.3_

- [x] 6.2 PastaFile 構造体と items フィールドにドキュメントコメントを追加
  - 内容: 旧 file_scope / global_scenes フィールドからの移行ガイド、ヘルパーメソッド説明
  - _Requirements: 6.2, 6.4_

- [x] 6.3 src/parser2/mod.rs のモジュールコメントに準拠性を明記
  - 追記: 「parser2 は grammar.pest `file = ( file_scope | global_scene_scope )*` 仕様に完全準拠」
  - _Requirements: 6.3, 6.4_

- [x] 6.4 既存 transpiler（legacy parser 使用）への影響ゼロを確認・文書化
  - 確認: src/transpiler/ が parser2 PastaFile を参照していないことを grep で確認
  - 確認: legacy parser モジュールが独立していることを確認
  - 記載: 仕様書に「parser2 修正は既存 transpiler に影響しない」と明記
  - _Requirements: 6.4_

### Phase 7: 統合テストと品質確認

- [x] 7. 全体統合テストを実行し、リグレッション・品質指標を確認
- [x] 7.1 全 parser2 テストを実行（cargo test --lib parser2）
  - 目標: 新規テスト含め全テスト合格（リグレッション0件）
  - _Requirements: 4.1, 4.2, 4.3_

- [x] 7.2 テストカバレッジが 80% 以上であることを確認
  - 対象: FileItem / PastaFile / build_ast / parse_file_scope
  - ツール: cargo tarpaulin または同等ツール
  - _Requirements: 4.1, 4.4_

- [x] 7.3 破壊的変更対応の完全性を確認
  - 確認: コンパイルエラーで 6箇所すべてのアクセス箇所が検出・修正されたことを確認
  - 確認: コンパイル成功 (cargo build)
  - _Requirements: 2.3_

---

## 要件マッピング検証

| 要件 | タスクカバレッジ |
|------|-----------------|
| 1.1 | 1.1, 4.1, 4.2 |
| 1.2 | 2.1, 4.1, 4.2 |
| 1.3 | 2.1, 4.1, 4.2 |
| 1.4 | 2.2, 4.1, 4.2 |
| 1.5 | 2.3, 4.2 |
| 1.6 | 2.1, 4.1 |
| 2.1 | 1.1 |
| 2.2 | 1.2 |
| 2.3 | 1.2, 7.3 |
| 2.4 | 1.2 |
| 2.5 | 1.3 |
| 3.1 | 2.1 |
| 3.2 | 2.2 |
| 3.3 | 2.3 |
| 3.4 | 2.1, 2.2 |
| 4.1 | 4.1, 4.2, 4.3 |
| 4.2 | 4.1, 4.2, 4.3, 4.4 |
| 4.3 | 4.1, 4.2, 4.3, 4.4 |
| 4.4 | 3.1, 3.2, 3.3, 7.1 |
| 4.5 | 4.4, 5.1 |
| 5.1 | 5.1 |
| 5.2 | 5.2 |
| 5.3 | 5.1 |
| 5.4 | 1.3 |
| 6.1 | 5.2 |
| 6.2 | 6.1, 6.2 |
| 6.3 | 6.3 |
| 6.4 | 6.3, 6.4 |

**カバレッジ**: ✅ 21受入基準 100% カバー、4タスク並列実行可能

---

## 実装開始時のチェックリスト

- [ ] 要件ドキュメント (`requirements.md`) を精読
- [ ] 設計ドキュメント (`design.md`) でコンポーネント図・インターフェース仕様を確認
- [ ] `src/parser2/grammar.pest` 行222の `file` ルール定義を確認
- [ ] 現在のバグ（`src/parser2/mod.rs` 行136）の上書き代入を確認
- [ ] `tests/parser2_integration_test.rs` の既存テスト6箇所を特定

---

## 次のステップ

実装タスク承認後、以下コマンドで最初のタスクを開始：

```bash
/kiro-spec-impl parser2-filescope-bug-fix 1.1
```

または複数タスク一括実行（推奨しません、コンテキスト爆発):

```bash
/kiro-spec-impl parser2-filescope-bug-fix 1.1,1.2,1.3
```

各タスク実行前にコンテキスト履歴をクリアすることを推奨。
