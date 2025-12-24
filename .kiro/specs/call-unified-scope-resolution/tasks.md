# Implementation Tasks

## Overview
**Feature**: `call-unified-scope-resolution`  
**Language**: ja  
**Total Requirements**: 5 (20 acceptance criteria)  
**Updated**: 2025-12-24 (API 名称統一、設計確定版)  
**Status**: ✅ **Completed** (2025-01-05)

---

## Task Breakdown

### 1. SceneTable に統合スコープ検索機能を実装

- [x] 1.1 (P) ローカル＋グローバル統合検索メソッドを実装
  - `collect_scene_candidates(module_name: &str, prefix: &str)` メソッドを追加
  - Step 1: `:module_name:prefix` で前方一致検索（ローカルシーン）
  - Step 2: `prefix` で前方一致検索、`:` 始まりキーを除外（グローバルシーン）
  - Step 3: 両方の `Vec<SceneId>` をマージして返す
  - WordTable の `collect_word_candidates` 実装パターンを完全継承
  - _Requirements: 1.1, 1.2, 3.1, 3.2, 3.3_

- [x] 1.2 (P) SceneCacheKey 構造体を拡張
  - `module_name: String` フィールドを追加（既存の `search_key` と統合）
  - `filters` を `Vec<(String, String)>` 形式で保持（ソート済み）
  - `Hash`, `Eq`, `PartialEq` トレイト実装を更新
  - キャッシュ無効化戦略: 異なる `module_name` では異なるキャッシュキー生成
  - _Requirements: 3.4_

- [x] 1.3 統合スコープ解決メソッドを実装
  - `resolve_scene_id(module_name, search_key, filters)` メソッドを追加
  - 内部で `collect_scene_candidates` を呼び出し
  - キャッシュキーを `SceneCacheKey` で構築
  - フィルター条件を適用してシーンを選択
  - キャッシュベースの順次消費ロジックを維持
  - _Requirements: 1.4, 3.4_

### 2. SceneTable の prefix_index キー形式を WordTable と統一

- [x] 2.1 (P) ローカルシーン登録時のキー変換を実装
  - `from_scene_registry` メソッド内で変換ロジックを追加
  - ローカルシーン: `fn_name` を `::` で分割し `:` で結合、先頭に `:` を付与
  - グローバルシーン: `fn_name` をそのまま使用（`::__start__` をカット）
  - 変換例: `会話_1::選択肢_1` → `:会話_1:選択肢_1`
  - _Requirements: 1.1, 3.1, 3.2_

### 3. stdlib の select_scene_to_id 関数シグネチャを変更

- [x] 3.1 module_name 引数を追加
  - 関数シグネチャを `fn(scene, module_name, filters, scene_table)` に変更
  - Rune VM から渡される `module_name: String` を受け取る
  - `SceneTable::resolve_scene_id` を呼び出すように変更
  - 実装の参考: `word_expansion()` 関数（stdlib/mod.rs の既存実装）
  - _Requirements: 2.2, 3.1_

### 4. Transpiler の Call 文生成コードを更新

- [x] 4.1 CodeGenerator で現在のグローバルモジュール名を取得
  - `generate_call_scene()` メソッドを変更
  - `context.current_module()` で現在のグローバルシーン名を取得
  - _Requirements: 2.1, 2.2_

- [x] 4.2 Call 関数呼び出しに module_name 引数を追加
  - 生成コードを `pasta::call(ctx, "{target}", "{module_name}")` 形式に変更
  - `module_name` を文字列リテラルとして埋め込み
  - 生成 Rune コードで stdlib の `select_scene_to_id(scene, module_name, filters)` を呼び出し
  - _Requirements: 2.2, 2.4_

### 5. SceneTable のユニットテストを実装

- [x] 5.1 (P) collect_scene_candidates メソッドのテスト
  - ローカルシーンのみの候補リスト取得を検証
  - グローバルシーンのみの候補リスト取得を検証
  - ローカル＋グローバル混在の候補リスト取得を検証
  - 前方一致検索の動作を検証
  - 候補なしの場合のエラー処理を検証
  - _Requirements: 1.1, 1.2, 3.1, 3.2, 3.3_

- [x] 5.2 (P) resolve_scene_id メソッドのテスト
  - module_name を渡した場合のローカルシーン解決を検証
  - キャッシュベース選択の動作を検証
  - 属性フィルター適用の動作を検証
  - _Requirements: 1.4, 3.4_

- [x] 5.3 (P) prefix_index キー変換のテスト
  - ローカルシーンが `:parent:local` 形式で登録されることを検証
  - グローバルシーンが正しく登録されることを検証
  - 変換後のキーで前方一致検索が正常動作することを検証
  - _Requirements: 3.1, 3.2_

### 6. Engine 統合テストを実装

- [x] 6.1 (P) ローカル＋グローバル統合検索の E2E テスト
  - グローバルシーン内から `＞シーン` でローカルシーンを呼び出せることを検証
  - 同じプレフィックスのローカルシーン＋グローバルシーン候補がマージされることを検証
  - ランダム選択の動作を検証（複数回実行して複数候補が選ばれることを確認）
  - _Requirements: 1.1, 1.2, 1.4, 4.2_

### 7. 既存テストの回帰確認と更新

- [x] 7.1 cargo test --all の実行
  - 全テストケースが成功することを確認
  - グローバル候補追加により挙動が変化するテストを特定
  - 変化が正当な場合はテストを更新、コメントで理由を明記
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [x] 7.2 fixtures ディレクトリの確認と機能デモ fixture 追加（オプション）
  - `＞＊` 構文使用箇所を確認（調査済み: 0件）
  - [x] 統合スコープ検索の挙動を示す新しい fixture を追加
  - _Requirements: 4.1, 4.2_

### 8. SPECIFICATION.md の更新

- [x] 8.1 Section 4 (Call 詳細仕様) を更新
  - Call 仕様を「ローカル＋グローバル統合検索」として説明
  - スコープ解決アルゴリズムの詳細説明を追加
  - _Requirements: 2.4, 5.1, 5.3_

- [x] 8.2 Section 10.3 (単語参照) とのクロスリファレンスを追加
  - Call 文と単語検索が同じスコープ解決ルールを使用することを明記
  - 両方のセクションから相互参照リンクを追加
  - _Requirements: 5.2_

---

## Requirements Coverage

| Requirement | Covered By Tasks                       |
| ----------- | -------------------------------------- |
| 1.1         | 1.1, 2.1, 5.1, 6.1                     |
| 1.2         | 1.1, 5.1, 6.1                          |
| 1.3         | （既存仕様の再確認のみ、新機能不要）   |
| 1.4         | 1.3, 5.2, 6.1                          |
| 2.1         | 4.1                                    |
| 2.2         | 3.1, 4.1, 4.2                          |
| 2.3         | （パーサー刷新で非サポート、対応不要） |
| 2.4         | 4.2, 8.1                               |
| 3.1         | 1.1, 2.1, 3.1, 5.1, 5.3                |
| 3.2         | 1.1, 2.1, 5.1, 5.3                     |
| 3.3         | 1.1, 5.1                               |
| 3.4         | 1.2, 1.3, 5.2                          |
| 4.1         | 7.1, 7.2                               |
| 4.2         | 6.1, 7.1, 7.2                          |
| 4.3         | 7.1                                    |
| 4.4         | 7.1                                    |
| 5.1         | 8.1                                    |
| 5.2         | 8.2                                    |
| 5.3         | 8.1                                    |
| 5.4         | （新パーサーで最初から非サポート）     |

---

## Task Dependencies

```
1.1, 1.2, 2.1 (P) ─┐
                   ├─> 1.3 ─> 3.1 ─┐
                   │                │
4.1, 4.2 (P) ──────────────────────┤
                                    │
5.1, 5.2, 5.3 (P) ──────────────────┤
                                    │
6.1 (P) ────────────────────────────┤
                                    │
                                    ├─> 7.1 ─> 7.2 ─> 8.1, 8.2
```

**並列実行可能タスク**:
- Phase 1: 1.1, 1.2, 2.1 (異なるメソッド・構造体)
- Phase 2: 4.1, 4.2 (同一の CodeGenerator メソッド内だが独立した処理)
- Phase 3: 5.1, 5.2, 5.3 (独立したテストケース)
- Phase 4: 6.1 (統合テスト)

**順次実行必須**:
- 1.3 は 1.1, 1.2, 2.1 完了後（依存メソッド使用）
- 3.1 は 1.3 完了後（API 変更依存）
- 7.1 は全実装タスク（1.3, 3.1, 4.2, 5.*, 6.1）完了後（回帰テスト）
- 7.2, 8.1, 8.2 は 7.1 完了後（最終確認・仕様更新）

---

## Effort Estimates

| Task          | Size | Notes                                   |
| ------------- | ---- | --------------------------------------- |
| 1.1, 1.2, 1.3 | 3-4h | SceneTable 拡張、WordTable パターン継承 |
| 2.1           | 1-2h | キー変換ロジック追加                    |
| 3.1           | 1h   | 関数シグネチャ変更、実装参考あり        |
| 4.1, 4.2      | 2-3h | Transpiler コード生成変更               |
| 5.1, 5.2, 5.3 | 3-4h | ユニットテスト実装                      |
| 6.1           | 2-3h | 統合テスト実装                          |
| 7.1, 7.2      | 1-2h | 回帰テスト、fixture 確認                |
| 8.1, 8.2      | 1-2h | SPECIFICATION.md 更新                   |

**Total**: 14-21 hours (2-3 days)
