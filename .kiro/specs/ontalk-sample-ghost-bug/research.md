# リサーチログ

## Discovery 分類

- **Feature Type**: Extension（既存システムの動作変更）
- **Discovery Level**: Light（新規境界導入なし）

## 拡張ポイント分析

### 改修対象メソッド

`SceneTable::resolve_scene_id` と `SceneTable::resolve_scene_id_unified` の2メソッドが同一のPhase 4パターンを持つ。両者とも以下の分岐で `NoMoreScenes` を返す:

```rust
if cached.next_index >= cached.candidates.len() {
    return Err(SceneTableError::NoMoreScenes { ... });
}
```

### Split Borrow の安全性（検証済み）

循環リセット時に `self.random_selector.shuffle_usize(&mut cached.candidates)` を呼ぶ必要がある。

#### 問題の発見

Phase 3で `self.cache.entry(key).or_insert_with(...)` が返す `&mut CachedSelection` は、`entry()` API が `&mut self` を借用した結果である。この借用は `cached` 変数が生存している間継続するため、Phase 4で `cached` を保持したまま `self.random_selector` にアクセスすることは**できない**。

Phase 3のクロージャ内部では問題ない（クロージャ実行時、`entry()` の借用は未確定のため `self` の他フィールドにアクセス可能）が、Phase 4は異なる。

#### 解決策: Phaseの分割

Phase 3とPhase 4の間で `cached` の借用を一度解放し、Phase 4でリセット判定後に `self.cache.get_mut()` で再取得する：

```rust
// Phase 3: Get or create cache entry
let cache_key = SceneCacheKey::new(module_name, search_key, filters);
let needs_reset = {
    let cached = self.cache.entry(cache_key).or_insert_with(|| { ... });
    cached.next_index >= cached.candidates.len()
};

// Phase 4: Reset if needed (self の借用は解放済み)
if needs_reset {
    let cached = self.cache.get_mut(&cache_key).unwrap();
    cached.next_index = 0;
    cached.history.clear();
    if self.shuffle_enabled {
        self.random_selector.shuffle_usize(&mut cached.candidates);  // ✅ OK
    }
}

// Phase 5: Sequential selection
let cached = self.cache.get_mut(&cache_key).unwrap();
// ...
```

**結論**: 設計上の4行の変更は正しいが、実装時はPhaseを4→5段階に分割する必要がある。

### NoMoreScenes の呼び出しチェーン

```
SceneTable::resolve_scene_id
  → Err(NoMoreScenes)
    → search/context.rs: match NoMoreScenes => Ok(None)
      → Lua: nil（シーン未発見扱い）
        → virtual_dispatcher: トーク不発→次回タイマーで再試行
```

循環リセットにより `NoMoreScenes` は到達不能となるが、上流の変換ロジックは防御的コードとして残す。

## 依存関係チェック

| 依存元 | 依存先 | 方向 | 影響 |
|--------|--------|------|------|
| pasta_lua/search/context.rs | SceneTable::resolve_scene_id | Inbound | 変更なし（インターフェース不変） |
| virtual_dispatcher.lua | @pasta_config | External | R2の設定値変更を自動反映 |
| pasta.toml | pasta_lua config loader | Outbound | 値変更のみ、スキーマ変更なし |

## 統合リスク

### リスク1: 既存テストの破壊

- `test_resolve_scene_id_unified_local_found` が `NoMoreScenes` を期待
- **対応**: アサーション変更（`is_err()` → `is_ok()`）
- **リスクレベル**: Low（テスト内部のみ、プロダクションコード影響なし）

### リスク2: 無限ループの可能性

- 候補数0件の場合に循環リセットが無限ループする可能性
- **分析**: 候補数0件のキャッシュは作成されない（Phase 2で候補検索、Phase 3でキャッシュ作成時に候補があることが前提）
- **リスクレベル**: None（構造的に到達不能）

## 代替案の検討

### Option A: Phase 4 インデックスリセット（採用）

- 最小変更（2メソッド × 4行）
- 既存インターフェース維持
- テスト容易

### Option B: CachedSelection にリセットメソッド追加

- `CachedSelection::reset()` メソッドを追加
- メリット: 責務が明確
- デメリット: private structに対するメソッド追加は過剰設計
- **却下理由**: 4行の変更に対してメソッド抽出は不要

### Option C: キャッシュ自体を破棄して再構築

- `self.cache.remove(&key)` で次回呼び出し時に再構築
- メリット: 完全なリフレッシュ
- デメリット: Phase 1-3の再実行コスト、候補検索の冪等性に依存
- **却下理由**: パフォーマンス不利、不必要な複雑さ
