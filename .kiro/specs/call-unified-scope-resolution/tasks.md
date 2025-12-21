# Implementation Tasks

## Overview
**Feature**: `call-unified-scope-resolution`  
**Language**: ja  
**Total Requirements**: 5 (20 acceptance criteria)

---

## Task Breakdown

### 1. SceneTable に統合スコープ検索機能を実装

- [ ] 1.1 (P) ローカル＋グローバル統合検索メソッドを実装
  - `find_scene_merged(module_name: &str, prefix: &str)` メソッドを追加
  - Step 1: `:module_name:prefix` で前方一致検索（ローカルシーン）
  - Step 2: `prefix` で前方一致検索、`:` 始まりキーを除外（グローバルシーン）
  - Step 3: 両方の `Vec<SceneId>` をマージして返す
  - WordTable の `collect_word_candidates` 実装パターンを流用
  - _Requirements: 1.1, 1.2, 3.1, 3.2, 3.3_

- [ ] 1.2 (P) SceneCacheKey 構造体を拡張
  - `module_name: String` フィールドを追加
  - `filters` を `Vec<(String, String)>` 形式で保持（ソート済み）
  - `Hash`, `Eq`, `PartialEq` トレイト実装を更新
  - _Requirements: 3.4_

- [ ] 1.3 統合スコープ解決メソッドを実装
  - `resolve_scene_id_unified(module_name, search_key, filters)` メソッドを追加
  - 内部で `find_scene_merged` を呼び出し
  - キャッシュキーを `SceneCacheKey` で構築
  - フィルター条件を適用してシーンを選択
  - キャッシュベースの順次消費ロジックを維持
  - _Requirements: 1.4, 3.4_

### 2. SceneTable の prefix_index キー形式を WordTable と統一

- [ ] 2.1 (P) ローカルシーン登録時のキー変換を実装
  - `from_scene_registry` メソッド内で変換ロジックを追加
  - ローカルシーン: `fn_name` を `::` で分割し `:` で結合、先頭に `:` を付与
  - グローバルシーン: `fn_name` をそのまま使用（変更なし）
  - 変換例: `会話_1::選択肢_1` → `:会話_1:選択肢_1`
  - _Requirements: 1.1, 3.1, 3.2_

### 3. stdlib の select_scene_to_id 関数シグネチャを変更

- [ ] 3.1 module_name 引数を追加
  - 関数シグネチャを `fn(scene, module_name, filters, scene_table)` に変更
  - Rune VM から渡される `module_name: String` を受け取る
  - `SceneTable::resolve_scene_id_unified` を呼び出すように変更
  - 空文字列の `module_name` が渡された場合はグローバルのみ検索
  - _Requirements: 2.2, 3.1_

### 4. Transpiler の Call 文生成コードを更新

- [ ] 4.1 Pass 2 で現在のグローバルモジュール名を取得
  - `transpile_statement_pass2_to_writer` 内の Call 処理を変更
  - `context.current_module()` で現在のグローバルシーン名を取得
  - JumpTarget の種類（Local/Global）に関わらず統一処理
  - _Requirements: 2.1, 2.2_

- [ ] 4.2 Call 関数呼び出しに module_name 引数を追加
  - 生成コードを `crate::pasta::call(ctx, "{search_key}", "{module_name}", #{filters}, [args])` 形式に変更
  - `module_name` を文字列リテラルとして埋め込み
  - JumpTarget::Global の `＊` プレフィックスも同じコード生成ルートを使用（非推奨だが互換性維持）
  - _Requirements: 2.2, 2.3_

### 5. SceneTable のユニットテストを実装

- [ ] 5.1 (P) find_scene_merged メソッドのテスト
  - ローカルシーンのみの候補リスト取得を検証
  - グローバルシーンのみの候補リスト取得を検証
  - ローカル＋グローバル混在の候補リスト取得を検証
  - 前方一致検索の動作を検証
  - 候補なしの場合のエラー処理を検証
  - _Requirements: 1.1, 1.2, 3.1, 3.2, 3.3_

- [ ] 5.2 (P) resolve_scene_id_unified メソッドのテスト
  - module_name を渡した場合のローカルシーン解決を検証
  - キャッシュベース選択の動作を検証
  - 属性フィルター適用の動作を検証
  - 空の module_name を渡した場合のグローバルのみ検索を検証
  - _Requirements: 1.4, 3.4_

- [ ] 5.3 (P) prefix_index キー変換のテスト
  - ローカルシーンが `:parent:local` 形式で登録されることを検証
  - グローバルシーンがそのまま登録されることを検証
  - 変換後のキーで前方一致検索が正常動作することを検証
  - _Requirements: 3.1, 3.2_

### 6. Engine 統合テストを実装

- [ ] 6.1 (P) ローカル＋グローバル統合検索の E2E テスト
  - グローバルシーン内から `＞シーン` でローカルシーンを呼び出せることを検証
  - 同じプレフィックスのローカルシーン＋グローバルシーン候補がマージされることを検証
  - ランダム選択の動作を検証（複数回実行して両方の候補が選ばれることを確認）
  - _Requirements: 1.1, 1.2, 1.4, 4.2_

- [ ] 6.2 (P) グローバルプレフィックス構文の互換性テスト
  - `＞＊シーン名` 構文が `＞シーン名` と同等に動作することを検証
  - 警告なしで処理されることを確認
  - _Requirements: 2.3, 4.1_

### 7. 既存テストの回帰確認と更新

- [ ] 7.1 cargo test --all の実行
  - 全テストケースが成功することを確認
  - グローバル候補追加により挙動が変化するテストを特定
  - 変化が正当な場合はテストを更新、コメントで理由を明記
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [ ] 7.2 fixtures ディレクトリの Call 文構文を確認
  - `＞＊` 構文使用箇所を洗い出し（調査済み: 0件）
  - 必要に応じて新しい統合スコープ検索の挙動を示す fixture を追加
  - _Requirements: 4.1, 4.2_

### 8. SPECIFICATION.md の更新

- [ ] 8.1 Section 4 (Call 詳細仕様) を更新
  - パターン1（グローバルシーン参照 `＊シーン名`）を削除
  - パターン2 を「ローカル＋グローバル統合検索」に更新
  - スコープ解決アルゴリズムの詳細説明を追加
  - `＞＊シーン名` 構文を非推奨として明記（互換性のためサポート継続）
  - _Requirements: 2.4, 5.1, 5.3, 5.4_

- [ ] 8.2 Section 10.3 (単語参照) とのクロスリファレンスを追加
  - Call 文と単語検索が同じスコープ解決ルールを使用することを明記
  - 両方のセクションから相互参照リンクを追加
  - _Requirements: 5.2_

---

## Requirements Coverage

| Requirement | Covered By Tasks |
|-------------|------------------|
| 1.1 | 1.1, 2.1, 5.1, 6.1 |
| 1.2 | 1.1, 5.1, 6.1 |
| 1.3 | N/A (既存仕様の再確認のみ) |
| 1.4 | 1.3, 5.2, 6.1 |
| 2.1 | 4.1 |
| 2.2 | 3.1, 4.1, 4.2 |
| 2.3 | 4.2, 6.2 |
| 2.4 | 8.1 |
| 3.1 | 1.1, 2.1, 3.1, 5.1, 5.3 |
| 3.2 | 1.1, 2.1, 5.1, 5.3 |
| 3.3 | 1.1, 5.1 |
| 3.4 | 1.2, 1.3, 5.2 |
| 4.1 | 6.2, 7.1, 7.2 |
| 4.2 | 6.1, 7.1, 7.2 |
| 4.3 | 7.1 |
| 4.4 | 7.1 |
| 5.1 | 8.1 |
| 5.2 | 8.2 |
| 5.3 | 8.1 |
| 5.4 | 8.1 |

---

## Task Dependencies

```
1.1, 1.2, 2.1 (P) ─┐
                   ├─> 1.3 ─> 3.1 ─┐
                   │                │
4.1 ───────────────┴─> 4.2 ─────────┤
                                    │
5.1, 5.2, 5.3 (P) ──────────────────┤
                                    │
6.1, 6.2 (P) ───────────────────────┤
                                    │
                                    ├─> 7.1 ─> 7.2 ─> 8.1, 8.2
```

**並列実行可能タスク**:
- Phase 1: 1.1, 1.2, 2.1 (異なるメソッド・構造体)
- Phase 2: 5.1, 5.2, 5.3 (独立したテストケース)
- Phase 3: 6.1, 6.2 (独立した統合テスト)

**順次実行必須**:
- 1.3 は 1.1, 1.2, 2.1 完了後（依存メソッド使用）
- 3.1 は 1.3 完了後（API 変更依存）
- 4.2 は 4.1 完了後（module_name 取得ロジック依存）
- 7.1 は全実装タスク完了後（回帰テスト）
- 8.1, 8.2 は 7.1 完了後（仕様確定）

---

## Effort Estimates

- **Task 1**: 3-4 hours (SceneTable 拡張、WordTable パターン流用)
- **Task 2**: 1-2 hours (キー変換ロジック追加)
- **Task 3**: 1 hour (関数シグネチャ変更のみ)
- **Task 4**: 2-3 hours (Transpiler コード生成変更、context 取得)
- **Task 5**: 3-4 hours (ユニットテスト実装)
- **Task 6**: 2-3 hours (統合テスト実装)
- **Task 7**: 1-2 hours (回帰確認、fixture 確認)
- **Task 8**: 1-2 hours (SPECIFICATION.md 更新)

**Total**: 14-21 hours (2-3 days)
