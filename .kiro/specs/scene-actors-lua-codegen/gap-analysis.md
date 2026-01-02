# 実装ギャップ分析

## 分析サマリー

- **スコープ**: pasta_luaのコード生成器（`code_generator.rs`）に`set_spot`初期化コード追加
- **現状**: `GlobalSceneScope.actors`フィールド利用可能（pasta_coreで完装備）、`generate_local_scene`呼び出し箇所特定済み
- **主要課題**: `generate_local_scene`に親シーンの`actors`情報を渡す設計、`__start__`関数識別ロジック確認
- **推奨**: Option A（既存コンポーネント拡張）- 最小限変更で高い互換性を実現

---

## 1. 現状調査

### 1.1 ドメインアセット

**主要ファイル構成:**
```
pasta_lua/src/
├── code_generator.rs     # Luaコード生成メインモジュール
├── transpiler.rs         # トランスパイラ（code_generatorを呼び出し）
├── context.rs            # トランスパイルコンテキスト
└── tests/
    └── transpiler_integration_test.rs  # 統合テスト
```

**関連コンポーネント（pasta_core）:**
- `pasta_core/parser/ast.rs`:
  - `SceneActorItem` 構造体（line 314-323）: `name: String`, `number: u32`, `span: Span`
  - `GlobalSceneScope` 構造体（line 332-378）: `actors: Vec<SceneActorItem>` フィールド保持

### 1.2 既存パターン・規約

**コード生成フロー:**
1. `transpiler.rs` line 76: `FileItem::GlobalSceneScope(scene)`をマッチ
2. `code_generator.rs` line 150: `generate_global_scene(&scene, counter, ...)`を呼び出し
3. `code_generator.rs` line 184: ローカルシーン反復処理で`generate_local_scene(local_scene, counter)`を呼び出し
4. `code_generator.rs` line 223: `generate_local_scene`内でセッション初期化→アイテム生成

**関数署名:**
```rust
pub fn generate_global_scene(
    &mut self,
    scene: &GlobalSceneScope,      // ← scene.actors アクセス可能
    scene_counter: usize,
    _context: &TranspileContext,
    _file_attrs: &HashMap<String, AttrValue>,
) -> Result<(), TranspileError>

pub fn generate_local_scene(
    &mut self,
    scene: &LocalSceneScope,       // ← LocalSceneScope には actors 情報なし
    counter: usize,
) -> Result<(), TranspileError>
```

**テスト規約:**
- ファイル: `transpiler_integration_test.rs`（line 1+）
- パターン: Pastaソース定義→パース→トランスパイル→Lua検証

### 1.3 統合面

**データフロー:**
- `GlobalSceneScope.actors` ← パース済み（pasta_coreで計算済み）
- `generate_local_scene`呼び出し時点では`LocalSceneScope`のみ渡される
- 親シーン情報（`actors`）は**渡されていない**

**既存処理の配置:**
- セッション初期化: line 241-242
  ```rust
  self.writeln("local args = { ... }")?;
  self.writeln("local act, save, var = PASTA.create_session(SCENE, ctx)")?;
  ```
- 空行追加: line 243
  ```rust
  self.write_blank_line()?;
  ```
- アイテム生成開始: line 246
  ```rust
  self.generate_local_scene_items(&scene.items)?;
  ```

---

## 2. 要件から技術ニーズへ

### 要件1: set_spotコード生成
**技術ニーズ:**
- `GlobalSceneScope.actors`の反復（各`SceneActorItem`）
- 各アイテムについて`act.<name>:set_spot(<number>)`形式Luaコード生成
- **アクティベーション条件**: `actors.is_empty() == false`の場合のみ

**既存キャパシティ:**
✅ Luaコード生成API (`writeln`, `write_indent`等) 利用可能
✅ `SceneActorItem`構造体フィールド（`name`, `number`）利用可能

### 要件2: コード配置
**技術ニーズ:**
- `__start__`関数内でのみ生成（`counter == 0`の場合）
- 配置位置: `PASTA.create_session`行の直後（line 243の空行の前）
- 命名規約: アクター名は`scene.name`が`None`のラッパー

**既存キャパシティ:**
✅ `counter`パラメータで`__start__`関数を識別可能（line 237: `counter == 0` ⟹ `__start__`）
✅ インデント管理機能（`indent()`, `dedent()`）

### 要件3: actorsフィールドアクセス
**技術ニーズ:**
- `generate_global_scene`内で`scene.actors`への参照を保持
- `generate_local_scene`呼び出し時に親シーンの`actors`を渡す
- `generate_local_scene`署名変更: `actors: &[SceneActorItem]`パラメータ追加

**既存キャパシティ:**
✅ `generate_global_scene`で`scene`パラメータ利用可能（line 150-151）
✅ `generate_local_scene`呼び出し箇所を特定済み（line 184）

### 要件4: テスト
**技術ニーズ:**
- パーサーテスト（pasta_coreで完了）
- Luaコード生成テスト（トランスパイル出力検証）
- ケース:
  1. 単一アクター（番号0）
  2. 複数アクター（番号飛び）
  3. アクター宣言なし（`set_spot`なし）
  4. 複数行宣言（複数行継続）

**既存テスト構造:**
✅ `transpiler_integration_test.rs`で同様パターン確認可能
✅ 出力検証パターン確立済み（line 60+: `lua_code.contains()`）

---

## 3. 実装アプローチ

### Option A: 既存コンポーネント拡張 ★ 推奨

**拡張対象:**
- `code_generator.rs`: `generate_local_scene`署名変更 + 実装追加
- `code_generator.rs`: `generate_global_scene`呼び出し部分で`actors`を渡す

**変更内容:**
1. `generate_local_scene`署名を拡張:
   ```rust
   pub fn generate_local_scene(
       &mut self,
       scene: &LocalSceneScope,
       counter: usize,
       actors: &[SceneActorItem],  // NEW
   ) -> Result<(), TranspileError>
   ```

2. `__start__`判定後、セッション初期化後に`set_spot`生成:
   ```rust
   if counter == 0 && !actors.is_empty() {
       for actor in actors {
           self.writeln(&format!("act.{}:set_spot({})", actor.name, actor.number))?;
       }
   }
   ```

3. `generate_global_scene`で呼び出し:
   ```rust
   self.generate_local_scene(local_scene, counter, &scene.actors)?;
   ```

**影響範囲:**
- `generate_local_scene`呼び出し: line 184（1箇所）
- 既存テストの署名: `transpiler.rs`テストヘルパー（複数箇所）

**互換性:** ⚠️ **Breaking Change** - テストコード修正が必要（4-5ファイル）

**複雑性・保守性:**
✅ 単一責任原則維持（`generate_local_scene`は引き続き「ローカルシーン生成」）
✅ 既存インデント/ライティングロジック流用
✅ 認知負荷・ファイルサイズ増加最小限

**トレードオフ:**
- ✅ 最小限変更、実装期間短（S相当）
- ✅ 既存パターン活用、テスト基盤確立
- ❌ `generate_local_scene`署名変更 → テストコード修正4-5箇所

---

### Option B: 新規ヘルパー関数

**新規コンポーネント:**
- `generate_set_spot_initializers(&[SceneActorItem])` → `Result<(), TranspileError>`

**実装戦略:**
- `generate_local_scene`内で呼び出し
- `__start__`判定は`generate_local_scene`内で実施

**影響範囲:**
- 新規関数：1
- `generate_local_scene`修正：実装追加のみ（署名変更なし）
- テストコード修正なし

**互換性:** ✅ **No Breaking Change**

**複雑性・保守性:**
⚠️ 関数分割による微細な管理コスト（新関数が小規模なため相対的にオーバーヘッド）

**トレードオフ:**
- ✅ テストコード修正なし
- ✅ 責任の明確化（`set_spot`生成を独立）
- ❌ 追加の関数ナビゲーション

---

### Option C: ハイブリッド（Option A + 段階的リファクタ）

**段階:**
1. **Phase 1**: Option B（新規ヘルパー）で実装、テスト合格
2. **Phase 2**: 次の仕様で`generate_local_scene`署名統一（他の初期化が加わる場合）

**トレードオフ:**
- ✅ 初期段階で署名変更リスク回避
- ❌ 将来的な署名統一の手戻し可能性

---

## 4. スコープ外（設計フェーズに延期）

- `set_spot`API側の実装詳細（Luaランタイム）
- `SceneActorItem`の追加情報（スポット名など）
- エラーハンドリング詳細（不正なアクター参照等）

---

## 5. 実装複雑性・リスク評価

### **推奨: Option A**

| 項目 | 評価 | 理由 |
|------|------|------|
| **工数** | **S** (1-3日) | 既存パターン流用、既知の変更点3箇所、テスト修正定型 |
| **リスク** | **Low** | `actors`フィールド既存、署名変更は内部（呼び出し箇所1）、回帰テスト網充実 |

### **代替: Option B**

| 項目 | 評価 | 理由 |
|------|------|------|
| **工数** | **S** (1日) | 関数追加のみ、既存署名維持 |
| **リスク** | **Low** | テストコード修正なし |

---

## 6. 推奨戦略 & 設計フェーズへの指針

### **推奨アプローチ: Option A（既存コンポーネント拡張）**

**理由:**
- 将来の拡張性（他のシーン初期化追加時に署名統一可能）
- 明確なデータフロー（親 → 子への情報流）
- pasta_coreのパターン統合（`GlobalSceneScope`→パーサー層でデータ準備）

### **設計フェーズでの確認事項:**

1. **署名変更範囲の確認**
   - `generate_local_scene`呼び出し: `code_generator.rs` line 184（1箇所のみ）
   - テストコード修正: `transpiler.rs` テストヘルパー（複数箇所特定）

2. **`__start__`判定ロジック**
   - 確認: `counter == 0` ⟺ start scene（確認済み line 237）

3. **テスト設計**
   - パターン: 既存統合テスト（`transpiler_integration_test.rs`）に新ケース追加
   - ケース: 単一/複数アクター、アクターなし、複数行継続

4. **実装順序**
   - **Task 1**: `code_generator.rs` 署名変更 + `set_spot`生成ロジック
   - **Task 2**: `transpiler.rs` 呼び出し部分修正
   - **Task 3**: テストコード修正
   - **Task 4**: 統合テスト追加・検証

---

## 付録: 実装パターン例（Option A）

```rust
// code_generator.rs - generate_local_scene 署名変更
pub fn generate_local_scene(
    &mut self,
    scene: &LocalSceneScope,
    counter: usize,
    actors: &[SceneActorItem],  // NEW: 親シーンのアクター情報
) -> Result<(), TranspileError> {
    // ... 既存コード ...
    
    // Session initialization (Requirement 3c)
    self.writeln("local args = { ... }")?;
    self.writeln("local act, save, var = PASTA.create_session(SCENE, ctx)")?;
    
    // NEW: Generate set_spot initializers for __start__ function only
    if counter == 0 && !actors.is_empty() {
        for actor in actors {
            self.writeln(&format!("act.{}:set_spot({})", actor.name, actor.number))?;
        }
    }
    
    self.write_blank_line()?;
    
    // ... 既存のアイテム生成 ...
}

// code_generator.rs - generate_global_scene の呼び出し部分
for local_scene in &scene.local_scenes {
    let counter = if let Some(ref name) = local_scene.name {
        let count = name_counters.entry(name.clone()).or_insert(0);
        *count += 1;
        *count
    } else {
        0
    };
    self.generate_local_scene(local_scene, counter, &scene.actors)?;  // NEW
}
```
