# Design Review Actions

## Overview

設計分析レポートの3つの重要懸念を踏まえ、以下の作業を実施します：
- **自明な修正点**: design.md の内容を実装仕様と照合し、誤りを修正
- **議題**: 開発者確認が必要な3つの設計判断について、順序立てて解決

---

## 自明な修正点（実装完了）

### 1.1 ✅ 検索 API の戻り値形式を確認・修正

**発見**:
- `search_word()` は `Result<String, WordTableError>` を返す（`Option` ではない）
- `resolve_scene_id()` は `SceneId` を返すが、SceneInfo は自分で取得する必要がある（`get_scene(id)` メソッド）

**修正内容**:
- design.md: `search_scene()` の戻り値を `Result<Option<...>, SearchError>` から `Result<(String, String), SearchError>` に統一
- design.md: `search_word()` の戻り値を `Result<Option<String>, SearchError>` から `Result<String, SearchError>` に統一
- design.md: SearchContext が `SceneTable::get_scene()` を使用してメタデータを取得することを明記

**Status**: 修正待ち

### 1.2 ✅ SearchContext 初期化の引数を確認・修正

**発見**:
- SceneTable/WordTable は `Box<dyn RandomSelector>` を要求する
- SearchContext は `SceneRegistry` と `WordDefRegistry` から初期化される（これは正しい）

**修正内容**:
- design.md: `SearchContext::new()` の戻り値型を確認（`Result<Self, SearchError>` 正確か）
- design.md: RandomSelector の初期化責任を明記（SearchContext が `DefaultRandomSelector::new()` を使うのか、呼び出し側が渡すのか）

**Status**: 議題2で決定

---

## 議題リスト

### 【議題1】MockRandomSelector 公開化の進め方

**課題**: Requirement 8（テスト用 Selector 制御）を実装するため、`MockRandomSelector` を `#[cfg(test)]` から外す必要があります。

**選択肢**:
- **オプション A**: 常時公開（`pub struct MockRandomSelector`）
  - 利点: テスト・Lua 実装で直接利用可能
  - 欠点: 本番環境で意図しない使用の可能性
- **オプション B**: Feature フラグ（`#[cfg(any(test, feature = "test-selector"))]`）
  - 利点: 本番環境で除外可能、テスト環境では有効
  - 欠点: pasta_lua が feature を有効化する必要がある

**決定の権限**: pasta_core maintainer または本プロジェクトの方針

**設計判断が必要な理由**:
- design.md では「推奨: オプション A」とありますが、プロジェクト方針（本番環境への懸念）によって判断が変わります

**Status**: ⏳ ユーザー確認待ち

---

### 【議題2】複数 Lua インスタンス対応：初期化フロー・所有権構造

**課題**: design.md では「複数の独立した Lua ランタイムインスタンスで各インスタンス用の SearchContext を管理する」と述べられていますが、以下が明確でありません：

1. **loader() の呼び出し者・タイミング**
   - 誰が `loader(lua, scene_registry, word_registry)` を呼ぶのか？
   - TranspileContext? PastaEngine? Lua require フック?
   - 初期化は何度（1回/複数回）？

2. **SearchContext インスタンスの所有権**
   - SearchContext が UserData として Lua に登録された後、Rust 側で参照を保持する必要があるか？
   - mlua がインスタンスをクローン可能にする必要があるか？
   - Arc/Mutex でラップする必要があるか？

3. **複数 Lua ランタイム = 複数 SceneRegistry か？**
   - 同一プロセス内で複数の Lua VM が **独立した** SceneRegistry を保持するのか？
   - それとも **共有** SceneRegistry の複数ビューか？

**背景知識**:
- pasta_lua は現在 TranspileContext が SceneRegistry/WordRegistry を管理しています
- 複数ランタイム = 複数の TranspileContext のシナリオが想定されます

**決定の権限**: パスタプロジェクトの設計方針

**Status**: ⏳ ユーザー確認待ち

---

### 【議題3】段階的フォールバック戦略の責任分岐

**課題**: design.md が示す「段階的フォールバック」（ローカル → グローバル）の実装責任が不明確です。

**現状分析**:
```
設計.md の期待:
  SearchContext.search_scene(name, Some("parent"))
    → SceneTable.resolve_scene_id_unified("parent", name, filters)  # ローカル検索
    → (失敗時) SceneTable.resolve_scene_id(name, filters)           # グローバル検索

実装仕様:
  resolve_scene_id_unified(module, key, filters)
    → collect_scene_candidates(module, key)  # ⚠️ この実装は？
    → フィルタリング・キャッシュ・選択
```

**問題点**:
- `collect_scene_candidates()` メソッドの仕様が確認できていません
- グローバル検索へのフォールバックはメソッド内で行われるのか、呼び出し側で行うのか不明確

**詳細確認項目**:
1. `SceneTable::collect_scene_candidates()` の挙動（引数・戻り値・エラー処理）
2. LocalScene が見つからない場合の返却値（`Err(NotFound)` か、`Ok(empty)` か？）
3. グローバル検索への自動フォールバック機能の有無

**決定の権限**: パスタプロジェクトの既存実装設計

**Status**: ⏳ コード検証 → ユーザー確認待ち

---

## 次のステップ

### 自明な修正点の修正
- [ ] 確認項目1.1 を design.md に反映
- [ ] 確認項目1.2 を design.md に反映
- [ ] すべての修正をコミット

### 議題の解決（ユーザーとの対話）
1. 【議題1】: MockRandomSelector 公開化のオプション選択
2. 【議題2】: 複数ランタイム初期化フローの確認
3. 【議題3】: 段階的フォールバック責任の確認
4. 各議題クローズごとに design.md 更新 + コミット

### 完了後
- `/kiro-spec-tasks pasta_search_module` でタスク生成

---

## 用語・定義

- **SceneId**: shapeInfo の一意識別子（Vec インデックス）
- **SceneInfo**: シーン名・スコープ・属性を含むメタデータ
- **ローカルシーン検索**: 親シーン内のシーンを検索（`:parent_name:key` 形式）
- **グローバルシーン検索**: プロジェクト全体から検索（`key` 形式）
- **段階的フォールバック**: ローカル検索失敗時、グローバル検索へ自動切り替え
