# Requirements & Gap Analysis レビュー結果

**実施日**: 2026-01-10  
**対象**: pasta_search_module (Requirement 1-8, 80 Acceptance Criteria)  
**レビュー範囲**: requirements.md + gap_analysis.md の整合性確認

---

## 1. 自明な修正項目（即座に実施可能）

### 1.1 ✅ 複数ランタイムインスタンス制約の整合性確認
**状態**: 既に requirements.md に「複数ランタイムインスタンス対応」セクションが追加されている  
**アクション**: **修正不要** - 既に整理済み

### 1.2 ✅ mlua-stdlib パターンの参照記載
**状態**: requirements.md の "mlua-stdlib参照を用いた実装方法" が gap_analysis.md と整合している  
**アクション**: **修正不要** - 既に整理済み

### 1.3 ✅ Design Decisions セクションの確認
**状態**: requirements.md に「複数ランタイムインスタンス対応」セクションが追加されている  
**アクション**: **修正不要** - 既に整理済み

---

## 2. 設計判断となる項目（Design フェーズで決定）

### 2.1 Option A/B/C の選択
**決定対象**: Requirement 4, Criteria 6（参照管理パターン）

**現状**:
- gap_analysis.md で 3 つの選択肢を評価済み
- Option A (UserData ラッピング) が推奨

**設計フェーズアクション**:
- Option A を採用することを明示的に決定
- requirements.md に「Design フェーズで Option A を選択」と記載（D記号で）

### 2.2 ✅ 議題 2: Lua側インターフェースデザイン（決定済み）
**決定対象**: SearchContext 公開方式

**検討内容**:
- パターン A: SearchContext を単一 UserData として公開 → `SEARCH:search_scene()` 呼び出し
- パターン B: SceneTable/WordTable を別々 UserData で公開 → `SEARCH.scene:search()` 呼び出し

**決定**: **パターン A を採用**

**理由**:
- シンプルなインターフェース設計
- Lua側でワードテーブル・シーンテーブルをほじくり返す必要なし
- API呼び出し形式が最初の提案と一致（`SEARCH:search_scene(...)`）

**修正内容**:
- requirements.md Requirement 1, Criteria 3 を修正
- gap_analysis.md Option B/C セクションを削除し、Option A に特化

**設計フェーズアクション**:
- メタテーブル設定で `SEARCH:func()` と `SEARCH.func()` の両形式対応
- mlua の `add_method_mut()` API で &mut self を提供（Selector 切り替え時）

---

### 2.3 ✅ Selector 制御実装方法（Requirement 8・決定済み）
**決定対象**: set_scene_selector() / set_word_selector() の可変性制御

**検討内容**:
- 方針 A: Interior Mutability（Arc<Mutex<>>）
- 方針 B: mlua の `add_method_mut()` で &mut self を使用 ← **採用**

**決定**: **方針 B を採用**

**理由**:
- mlua の `add_method_mut()` で UserData への可変参照メソッドが作成可能
- Lua 側から `SEARCH:set_scene_selector(...)` で呼び出し
- Rust 側で SearchContext への exclusive access が自動的に確保される
- Arc<Mutex<>> の overhead や deadlock リスクが不要
- Rust 的な &mut self 設計が活用できる

**実装方針**:
```rust
impl mlua::UserData for SearchContext {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        // 不変メソッド
        methods.add_method("search_scene", |lua, this: &SearchContext, ...| { ... });
        
        // 可変メソッド
        methods.add_method_mut("set_scene_selector", |lua, this: &mut SearchContext, selector_config| {
            this.scene_table.replace_random_selector(...)?;
            Ok(())
        });
    }
}
```

---

## 3. 全議題の決定内容サマリ

| 議題 | 内容 | 決定 | 状態 |
|------|------|------|------|
| 1 | SceneInfo API 設計 | SceneTable/WordTable が直接 SceneInfo/String を返す | ✅ クローズ |
| 2 | Lua側インターフェース | SearchContext を単一 UserData として公開（パターン A） | ✅ クローズ |
| 3 | RandomSelector &mut 制御 | mlua の add_method_mut() で &mut self を使用 | ✅ クローズ |

**次フェーズ**: Design フェーズで上記決定を基に詳細設計を実施

---

## 3. 開発者確認が必要な項目（議題化）

### ⚠️ 議題 1: SceneInfo 復元メカニズム

**問題**:
```
Requirement 2.3, 2.5:
  - search_scene() は (global_name, local_name) タプルを返す必要がある
  - 現在、pasta_core の resolve_scene_id() は SceneId のみを返す

Question: SceneId から (global_name, local_name) をどうやって構築するのか？
```

**現状分析**:
- SceneTable.labels は Vec<SceneInfo> で ID-indexed
- SceneInfo には name, parent, fn_name が含まれている
- しかし、labels は private（public API なし）

**必要な確認**:
1. SceneTable に `get_scene_info(id: SceneId) -> Option<&SceneInfo>` メソッドを追加すべきか？
2. または、resolve_scene_id() の代わりに `resolve_scene_info()` を追加すべきか？
3. (global_name, local_name) への変換ロジックはどこに書くのか？

**提案**:
- pasta_core に public method を追加：`pub fn get_scene_info(&self, id: SceneId) -> Option<&SceneInfo>`
- または、SceneTable に `search_scene(search_key) -> Result<(String, String)>` メソッドを追加

---

### ⚠️ 議題 2: Lua 側インターフェース設計

**問題**:
```
Requirement 1.2, 1.3:
  - Lua側で SEARCH.search_scene(name, global_scene_name) のようなインターフェース
  - UserData メソッドを使う場合、SEARCH:search_scene() になる可能性

Question: Lua での呼び出し形式をどう設定するか？
```

**現状分析**:
- mlua-stdlib の http モジュール等は UserData + メタテーブルで `module.func()` 形式を実現している
- メタテーブルの `__index` を利用して実装可能

**必要な確認**:
1. `SEARCH.search_scene()` と `SEARCH:search_scene()` の実装パターンはどれを選ぶのか？
2. メタテーブル設定による `__index` オーバーライドは要件仕様書に明記すべきか？

**提案**:
- requirements.md の Requirement 1 に「Lua 側呼び出し形式は `SEARCH.search_scene()` で統一」と明記
- gap_analysis.md のメタテーブル設定パターンを Design フェーズでの実装ガイドとする

---

### ⚠️ 議題 3: RandomSelector の &mut self 制御

**問題**:
```
Requirement 8 (Criteria 4, 6):
  - set_scene_selector(n1, n2, ...) で MockRandomSelector に切り替える
  - Box<dyn RandomSelector> の `&mut self` メソッドをどうやって Lua から安全に呼び出すか？

Question: Rust 側での RandomSelector 切り替え実装をどうするか？
```

**現状分析**:
- SceneTable.random_selector は `Box<dyn RandomSelector>` で所有
- mlua の `add_method_mut()` は `&mut self` をサポート
- SearchContext を UserData として wrap すれば、`&mut self` を提供できる

**必要な確認**:
1. SearchContext 内で RandomSelector を `&mut self` で切り替える実装は実現可能か？
2. または、Interior Mutability (RefCell) を使うべきか？
3. MockRandomSelector の初期化（`new(vec![...])` の Lua 側呼び出し）をどうするか？

**提案**:
- UserData の `add_method_mut("set_scene_selector", ...)` で実装
- 整数の可変長引数を Vec<usize> に変換して MockRandomSelector::new() に渡す

---

## 4. コミット戦略

### Phase A: 自明な修正（完了済み）
- ✅ gap_analysis.md 完成・コミット済み

### Phase B: 開発者確認・議題化

#### ✅ 議題 1: SceneInfo 復元 API（クローズ - 2026-01-10）

**決定内容**:
- SceneTable の `resolve_scene_id()` ではなく、`resolve_scene() -> Result<&SceneInfo>` で直接返すべき
- WordTable も同様に直接値を返すメソッルに統一
- 理由：2 段階取得は非効率。pasta_core で既に SceneInfo を確定しているので直接返すべき

**修正済み**:
- requirements.md Requirement 4, Criteria 6 を「SceneTable/WordTable が SceneInfo/String を直接返すことを期待」に更新

**次**: 議題 2 へ進行

---

#### ⏳ 議題 2: Lua 側インターフェース設計（開発者確認待ち）
#### ⏳ 議題 3: RandomSelector &mut self 制御（開発者確認待ち）

---

## 5. 現在のステータス

| 項目 | 状態 | 次ステップ |
|------|------|----------|
| Gap Analysis 完成 | ✅ コミット済み | - |
| Requirements 整合性 | ✅ OK | - |
| 自明な修正 | ✅ 完了 | - |
| **議題 1: SceneInfo 復元** | ⏳ 開発者確認待ち | **→ 次に議論開始** |
| 議題 2, 3 | ⏳ キュー待ち | 議題 1 終了後に進行 |
| Design フェーズ | ⏳ 全議題クローズ後 | 最終確認コマンド提示 |

