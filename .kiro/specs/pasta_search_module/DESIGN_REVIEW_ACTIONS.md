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

### 【議題3】段階的フォールバック戦略の責任分岐 ⚠️ 重大な不一致発見

**課題**: design.md が示す「段階的フォールバック」（ローカル → グローバル）の実装責任が不明確です。

**🔴 重大な発見**:

厳密な調査の結果、**要件と pasta_core 実装に重大な不一致** があることが判明しました。

#### 要件定義の期待（フォールバック戦略）

**Requirement 2.2-2.5**:
> 2.2: global_scene_name が指定された場合、段階的フォールバック戦略で検索を実行（ローカル → グローバル）
> 2.3: ローカルシーンで結果が見つかった場合、**そこから選択**
> 2.4: ローカルシーン検索結果が**０件の場合**、グローバルシーンで検索を**再実行**

つまり：ローカルで見つかったら**ローカルのみ**から選択、なければグローバルへフォールバック

#### pasta_core の実装（マージ戦略）

**scene_table.rs L372-392**:
```rust
// Step 1: Local search with :{module_name}:{prefix} pattern
candidates.extend(local_ids);  // ローカル結果を追加

// Step 2: Global search with {prefix} pattern
candidates.extend(global_ids); // グローバル結果も追加

// Step 3: Return merged candidate list
Ok(candidates)  // 両方をマージして返す
```

**テストケース `test_collect_scene_candidates_local_and_global_merge`**:
> "Test: Both local and global candidates should be merged"
> `candidates.len() == 2` （ローカル + グローバル両方が含まれる）

#### 不一致の影響

| シナリオ | 要件の期待 | 実装の動作 |
|----------|-----------|-----------|
| ローカル2件 + グローバル3件 | ローカル2件から選択 | 5件全部から選択 |
| ローカル0件 + グローバル3件 | グローバル3件から選択 | グローバル3件から選択 ✅ |

#### 選択肢

**選択肢 A**: 要件を修正（マージ戦略を採用）
- pasta_core の既存実装を尊重
- 要件を「ローカル + グローバルをマージして選択」に変更
- **利点**: 実装変更不要
- **欠点**: 要件の意図と異なる可能性

**選択肢 B**: pasta_core を修正（フォールバック戦略を実装）
- 新しいメソッド `collect_scene_candidates_fallback()` を追加
- または既存メソッドにフラグを追加
- **利点**: 要件通りの動作
- **欠点**: pasta_core への修正が必要

**選択肢 C**: SearchContext レイヤーでフォールバック実装
- pasta_core はマージのまま維持
- SearchContext が2段階で呼び出し（ローカル検索 → 0件ならグローバル検索）
- **利点**: pasta_core 変更なし、要件通りの動作
- **欠点**: 検索ロジックの分散

**Status**: ⏳ ユーザー確認待ち

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
