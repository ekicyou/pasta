# Implementation Gap Analysis: pasta-label-resolution-runtime

| 項目 | 内容 |
|------|------|
| **Feature Name** | pasta-label-resolution-runtime |
| **Analysis Date** | 2025-12-14 |
| **Analyzer** | GitHub Copilot |
| **Parent Spec** | pasta-declarative-control-flow (completed) |
| **Priority** | P1 |

---

## 分析サマリー

**スコープ:** Pasta DSLの実行時ラベル解決機能（前方一致検索、属性フィルタリング、ランダム選択、キャッシュベース消化）を実装し、Rust側の `LabelTable::resolve_label_id()` とRune側の `pasta_stdlib::select_label_to_id()` を統合する。

**主要な課題:**
- 現在の `LabelTable::find_label()` は完全一致検索（HashMap）のみをサポート、前方一致検索が未実装
- 履歴管理が配列インデックスベースで脆弱（フィルタ変動や候補順序変更に非対応）
- Rune ↔ Rust の型変換（HashMap、Object型）が未定義

**推奨アプローチ:** 
- **Option A（推奨）:** 既存の `LabelTable` を拡張し、`resolve_label_id()` メソッドを追加。HashMap + フルスキャンで前方一致検索を実装し、履歴管理をラベルIDベースに変更。
- **実装難易度:** M (3-7日) - 既存パターンの拡張、適度な複雑度
- **リスク:** Low - 既存アーキテクチャとの整合性が高く、明確な統合ポイントあり

---

## 1. Current State Investigation

### 1.1 既存アセット

#### 核心モジュール

| モジュール | パス | 責務 | 再利用可能性 |
|-----------|------|------|------------|
| **LabelTable** | `crates/pasta/src/runtime/labels.rs` | ラベル管理、完全一致検索、属性フィルタリング、ランダム選択 | ✅ 高 - 拡張ベース |
| **RandomSelector** | `crates/pasta/src/runtime/random.rs` | ランダム選択の抽象化、MockRandomSelector | ✅ 完全再利用 |
| **LabelRegistry** | `crates/pasta/src/transpiler/label_registry.rs` | トランスパイル時のラベル収集とID割り当て | ✅ 完全再利用 |
| **PastaStdlib** | `crates/pasta/src/stdlib/mod.rs` | Rune標準ライブラリ、`select_label_to_id()` スタブ実装 | ✅ 拡張必要 |
| **PastaError** | `crates/pasta/src/error.rs` | エラー型定義 | ✅ 新規エラー追加 |

#### データ構造

**LabelTable（既存）:**
```rust
pub struct LabelTable {
    labels: HashMap<String, Vec<LabelInfo>>,  // ⚠️ 完全一致のみ
    history: HashMap<String, Vec<usize>>,     // ⚠️ インデックス記録（問題あり）
    random_selector: Box<dyn RandomSelector>,
}
```

**LabelInfo（既存）:**
```rust
pub struct LabelInfo {
    pub name: String,              // DSL上のラベル名
    pub scope: LabelScope,
    pub attributes: HashMap<String, String>,
    pub fn_name: String,           // Rune関数名（"会話_1::__start__"）
    pub parent: Option<String>,
}
```

**重要:** 現在の `LabelInfo` には `id` フィールドが存在しない。`LabelRegistry::LabelInfo` には `id: i64` フィールドがあるが、`runtime::LabelInfo` には欠落している。本実装では `runtime::LabelInfo` に `pub id: usize` フィールドを追加する。

#### アーキテクチャパターン

1. **トレイトベースの抽象化**: `RandomSelector` トレイトでテスタビリティを確保
2. **2パストランスパイラー**: Pass 1でラベル収集 → Pass 2で `mod pasta {}` 生成
3. **Rust ↔ Rune ブリッジ**: `create_module()` でRune関数を登録
4. **エラー処理**: `thiserror::Error` を使用した構造化エラー
5. **テストパターン**: `MockRandomSelector` で決定論的テスト、`tests/common/` で共通ユーティリティ

### 1.2 統合ポイント

#### 呼び出しフロー

```
Rune: label_selector(label, filters)
  ↓
Rune: pasta_stdlib::select_label_to_id(label, filters)  ← スタブ実装
  ↓
Rust: LabelTable::resolve_label_id(label, filters)      ← 未実装
  ↓
Rust: 前方一致検索 (fn_path.starts_with(search_key))
  ↓
Rust: フィルタリング (属性マッチ + "::__start__"サフィックスフィルタ)
  ↓
Rust: ランダム選択 (RandomSelector::select_index)
  ↓
Rust: 履歴記録 (history[search_key].push(selected.id))
  ↓
Return: ラベルID (i64)
  ↓
Rune: match id { ... } → 関数ポインタ取得 → 実行
```

**検索キー例:**
- `JumpTarget::Global("会話")` → `"会話"` → `"会話_1::__start__"`, `"会話_2::__start__"` にマッチ
- `JumpTarget::Local("選択肢")` → `"選択肢"` → 親ラベルコンテキスト内で検索
- `JumpTarget::LongJump{"会話", "選択肢"}` → `"会話::選択肢"` → `"会話_1::選択肢_1"` にマッチ

#### 現在のスタブ実装

```rust
// crates/pasta/src/stdlib/mod.rs (L56-67)
fn select_label_to_id(_label: String, _filters: rune::runtime::Value) -> i64 {
    // P0: Always return 1 for basic testing
    1
}
```

**統合課題:**
- Runeの `Value` 型から Rust の `HashMap<String, String>` への変換が必要
- `LabelTable` への参照を `select_label_to_id()` に渡す仕組みが未定義

### 1.3 命名規則と慣例

- **Rust型**: `PascalCase` (構造体、列挙型、トレイト)
- **Rust関数**: `snake_case`
- **Runeモジュール**: `snake_case` (`pasta_stdlib`)
- **エラー型**: `PastaError::<Variant>` 形式
- **テスト**: `test_<機能>` または `test_<ケース>`
- **モックオブジェクト**: `Mock<Original>` (例: `MockRandomSelector`)

---

## 2. Requirements Feasibility Analysis

### 2.1 要件からの技術ニーズ

| 要件 | 技術ニーズ | 既存実装 | Gap |
|------|-----------|---------|-----|
| **Req 1: 前方一致検索** | `fn_path` のプレフィックスマッチング | `HashMap::get()` 完全一致のみ | ❌ Missing |
| **Req 2: 属性フィルタリング** | `LabelInfo::attributes` のANDフィルタ | ✅ `find_label()` で実装済み | ✅ Reuse |
| **Req 3: ランダム選択** | `RandomSelector::select_index()` | ✅ 実装済み | ✅ Reuse |
| **Req 4: キャッシュベース消化** | ラベルIDベース履歴管理 | ⚠️ インデックスベース履歴 | ⚠️ Constraint |
| **Req 5: Rust ↔ Rune ブリッジ** | Rune Value → HashMap 変換、`LabelTable` 参照渡し | ⚠️ スタブのみ | ❌ Missing |
| **Req 6: Registry → Table 変換** | `LabelRegistry` → `LabelTable` 変換時にIDフィールド追加 | ⚠️ IDフィールド欠落 | ⚠️ Constraint |

### 2.2 ギャップと制約

#### Gap 1: 前方一致検索の未実装

**現状:**
```rust
// labels.rs L94-105
pub fn find_label(&mut self, name: &str, ...) -> Result<String, PastaError> {
    let candidates = self.labels.get(name)?;  // ← HashMap::get() 完全一致のみ
    // ...
}
```

**必要な機能:**
- `fn_path` が検索キーで始まるすべての `LabelInfo` を抽出
- 例: 検索キー `"会話"` → `"会話_1::__start__"`, `"会話_2::__start__"` にマッチ
- グローバルラベル: `"::__start__"` で終わるものをフィルタ

**採用決定: Trie prefix index + Vec storage with multi-phase search**
```rust
use fast_radix_trie::RadixMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LabelId(usize);  // Vec index

pub struct LabelTable {
    labels: Vec<LabelInfo>,  // ID-based storage
<<<<<<< HEAD
    prefix_index: RadixMap<Vec<LabelId>>,  // fn_path → [LabelId]
    cache: HashMap<String, CachedSelection>,  // search_key → shuffled IDs
=======
    prefix_index: RadixMap<Vec<LabelId>>,  // fn_name → [LabelId]
    cache: HashMap<CacheKey, CachedSelection>,  // (search_key, filters) → shuffled IDs
>>>>>>> 934c1a65c625dc28e18ef5f7bc5699b1f5036e98
    random_selector: Box<dyn RandomSelector>,
    shuffle_enabled: bool,
}

// Cache key includes both search_key and filters
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    search_key: String,
    filters: Vec<(String, String)>,  // Sorted for consistent hashing
}

struct CachedSelection {
    candidates: Vec<LabelId>,  // Shuffled on first access
    next_index: usize,
    history: Vec<LabelId>,
}

// Multi-phase search:
// Phase 1: Trie prefix search O(M) - M is search_key length
let candidate_ids: Vec<LabelId> = self.prefix_index
    .common_prefixes(search_key.as_bytes())
    .flat_map(|(_key, ids)| ids.iter().copied())
    .collect();

// Phase 2: Filter by secondary criteria
let filtered_ids: Vec<LabelId> = candidate_ids
    .into_iter()
    .filter(|&id| self.matches_filters(id, filters))
    .collect();

// Phase 3: Get or create cache (shuffle once if enabled)
let cached = self.cache.entry(search_key.to_string())
    .or_insert_with(|| {
        let mut ids = filtered_ids.clone();
        if self.shuffle_enabled {
            self.random_selector.shuffle(&mut ids);
        }
        CachedSelection {
            candidates: ids,
            next_index: 0,
            history: Vec::new(),
        }
    });

// Phase 4: Sequential selection from cache
let selected_id = cached.candidates[cached.next_index];
cached.next_index += 1;
cached.history.push(selected_id);
```

**設計根拠:**
- **Trie prefix index**: O(M)検索性能、ラベル数に依存しない（fast_radix_trie v1.1.0）
- **Vec storage**: ラベル削除なし → Vec indexで十分、高速アクセス
- **ID-based access**: LabelInfoをコピーせず、IDで参照
- **Shuffle strategy**: キャッシュレイヤーでシャッフル（テスト容易性確保）
- **Debug mode**: `shuffle_enabled = false` で決定論的テスト可能

#### Gap 2: ラベルIDフィールドとデータ構造の再設計

**現状:**
- `LabelRegistry::LabelInfo` には `id: i64` フィールドあり
- `LabelTable::LabelInfo` には `id` フィールドなし
- `LabelTable` は `HashMap<String, Vec<LabelInfo>>` で管理（完全一致検索のみ）

**影響:**
- 履歴管理でIDを記録できない（現在は配列インデックスを記録）
- `resolve_label_id()` がIDを返す要件を満たせない
- LabelInfoのコピー/クローンが頻繁に発生（パフォーマンス懸念）

**解決策: ID-based storage with Vec**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LabelId(usize);  // newtype wrapper

pub struct LabelInfo {
    pub id: LabelId,  // ← 追加必須
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub fn_path: String,
    pub parent: Option<String>,
}

pub struct LabelTable {
    labels: Vec<LabelInfo>,  // ID = Vec index
    cache: HashMap<CacheKey, CachedSelection>,  // (search_key, filters) pair
    random_selector: Box<dyn RandomSelector>,
    shuffle_enabled: bool,
}
```

**移行手順:**
1. `runtime::LabelInfo` に `id: LabelId` フィールド追加
2. `from_label_registry()` で `LabelRegistry` → `Vec<LabelInfo>` 変換時にIDを割り当て
   - `id = LabelId(vec_index)` として連番で割り当て
3. `HashMap<String, Vec<LabelInfo>>` → `Vec<LabelInfo>` に変更
4. 検索ロジックを `Vec::iter().filter()` に変更（O(N)走査）
5. キャッシュを `Vec<LabelId>` 型に変更（データ重複排除）

#### Gap 3: キャッシュ管理とシャッフル戦略の未実装

**現状:**
```rust
// labels.rs L143-149
self.history
    .entry(name.to_string())
    .or_insert_with(Vec::new)
    .push(
        candidates.iter().position(|l| l.fn_name == selected.fn_name).unwrap(),
        // ↑ 配列のインデックスを記録（問題あり）
    );
```

**問題:**
- 配列インデックスベースの履歴管理（フィルタ変動に脆弱）
- シャッフルのタイミングが未定義（毎回シャッフル vs 初回のみ）
- デバッグモードでの決定論的テストが不可能

**解決策: CachedSelection 構造体の導入**
```rust
struct CachedSelection {
    candidates: Vec<LabelId>,  // Shuffled on first access
    next_index: usize,         // Sequential selection pointer
    history: Vec<LabelId>,     // Selection history (IDs, not indices)
}

pub struct LabelTable {
    cache: HashMap<CacheKey, CachedSelection>,  // (search_key, filters) → cache entry
    shuffle_enabled: bool,  // Default: true, false for testing
    // ...
}
```

**キャッシュ戦略:**
1. **初回アクセス**: 検索キーに対して候補を列挙 → シャッフル（`shuffle_enabled = true`時） → キャッシュ作成
2. **2回目以降**: キャッシュから順次選択 (`next_index` をインクリメント)
3. **履歴記録**: `history.push(selected_id)` で選択済みIDを記録
4. **デバッグモード**: `shuffle_enabled = false` でVec順序そのまま（決定論的）

**解決済みTODO:**
- ✅ **TODO #3**: 履歴キー生成の一貫性 → search_key単位で CachedSelection を管理、フィルタは検索時に適用
- ✅ **TODO #4**: Rune Value → HashMap変換 → `rune::from_value<T>()` パターン確認済み（persistence.rs参考）
- ✅ **TODO #5**: Arc<Mutex> 統合 → 既存パターンで実装可能

#### Gap 4: Rune ↔ Rust 型変換の未定義 ✅ **解決済み**

**現状:**
```rust
fn select_label_to_id(_label: String, _filters: rune::runtime::Value) -> i64 {
    // filters は rune::runtime::Value 型（未変換）
}
```

**必要な機能:**
- Runeの `Value` 型から Rust の `HashMap<String, String>` への変換
- Rune側で空のHashMapが渡された場合の処理（`Value::Unit` vs `Value::Object(empty)`）

**解決策: `rune::from_value<T>()`パターン使用**

既存の `persistence.rs` モジュールに完璧な参考実装が存在:
```rust
// crates/pasta/src/stdlib/persistence.rs より
fn toml_to_string(data: rune::Value) -> Result<String, String> {
    // Convert Rune Value to HashMap
    let map: std::collections::HashMap<String, rune::Value> = rune::from_value(data)
        .map_err(|e| format!("Failed to convert Rune value to map: {}", e))?;
    
    for (key, value) in map {
        // value もさらに from_value で変換可能
    }
}

// String型への変換例（rune_value_to_toml_value より）
if let Ok(s) = rune::from_value::<String>(value.clone()) {
    // String変換成功
}
```

**確定実装パターン:**
```rust
fn parse_rune_filters(value: rune::Value) -> Result<HashMap<String, String>, String> {
    // Step 1: Value → HashMap<String, rune::Value>
    let rune_map: HashMap<String, rune::Value> = rune::from_value(value)
        .map_err(|e| format!("Filters must be object: {}", e))?;
    
    // Step 2: 各 Value → String に変換
    let mut filters = HashMap::new();
    for (key, val) in rune_map {
        let str_val = rune::from_value::<String>(val)
            .map_err(|e| format!("Filter value for '{}' must be string: {}", key, e))?;
        filters.insert(key, str_val);
    }
    
    Ok(filters)
}

// 空のHashMapケース: rune::from_value() が自動的にエラーを返すため、
// Unit→empty HashMap変換が必要な場合は明示的なmatch処理を追加
```

**空HashMap処理（必要な場合）:**
```rust
// Pastaスクリプト側で `{}` を渡せば自動的に空のHashMapになる
let label_id = select_label_to_id("ch-happy", {});  // Empty filters

// ただし `()` (Unit)を許容する必要がある場合:
fn parse_rune_filters_with_unit(value: rune::Value) -> Result<HashMap<String, String>, String> {
    // Check if it's Unit first
    if rune::from_value::<()>(value.clone()).is_ok() {
        return Ok(HashMap::new());  // Unit → empty HashMap
    }
    
    // Otherwise, parse as HashMap
    parse_rune_filters(value)
}
```

**キー発見:**
- ✅ **`rune::from_value<T>(value)`** が型変換の基本パターン
- ✅ **2段階変換**: `Value` → `HashMap<String, Value>` → `HashMap<String, String>`
- ✅ **エラー処理**: `map_err()` でRune変換エラーを `String` に変換
- ✅ **既存コードベース**: `persistence.rs` に実装済み、`stdlib/mod.rs` にimport済み

**Resolution:** `rune::from_value()` API確認完了。Design Phaseで詳細仕様を決定。

#### Gap 5: LabelTableへの参照渡しとArc<Mutex>統合

**現状:**
- `select_label_to_id()` は引数として `label` と `filters` のみを受け取る
- `LabelTable` への参照がない

**既存パターン:**
```rust
// engine.rs L58
pub struct PastaEngine {
    label_table: LabelTable,  // ← エンジンが所有
    // ...
}
```

**実装決定: Arc<Mutex<LabelTable>> with closure capture**
```rust
// stdlib/mod.rs
pub fn create_module(label_table: Arc<Mutex<LabelTable>>) -> Result<Module, ContextError> {
    let mut module = Module::new();
    
    // Clone Arc for closure
    let label_table_clone = Arc::clone(&label_table);
    
    module.function("select_label_to_id", move |label: String, filters: Value| {
        let mut table = label_table_clone.lock().unwrap();
        // ... resolve_label_id 呼び出し
    })?;
    
    Ok(module)
}
```

**PastaEngineの変更:**
```rust
pub struct PastaEngine {
    label_table: Arc<Mutex<LabelTable>>,  // ← 変更
    // ...
}

impl PastaEngine {
    pub fn new(script: &str) -> Result<Self, PastaError> {
        let label_table = Arc::new(Mutex::new(LabelTable::from_label_registry(
            &registry,
            Box::new(DefaultRandomSelector),
            true,  // shuffle_enabled
        )?));
        
        let mut context = Context::with_default_modules()?;
        context.install(pasta_stdlib::create_module(Arc::clone(&label_table))?)?;
        // ...
    }
}
```

**全TODO解決済み（Gap 3参照）**

### 2.3 複雑性シグナル

- **アルゴリズム複雑度**: 中程度 - 前方一致検索（線形走査）、フィルタリング（ネストループ）
- **外部統合**: Rune VM統合（既存パターンあり）
- **データ構造変更**: `LabelInfo` へのIDフィールド追加、履歴管理の変更
- **並行性**: `Arc<Mutex<LabelTable>>` でスレッドセーフ性は確保済み

---

## 3. Implementation Approach Options

### Option A: Extend Existing LabelTable（推奨）

**概要:** 既存の `LabelTable` に `resolve_label_id()` メソッドを追加し、`find_label()` と並存させる。

#### 変更対象ファイル

1. **`crates/pasta/src/runtime/labels.rs`**
   - `LabelInfo` に `pub id: usize` フィールド追加
   - `LabelTable::resolve_label_id()` メソッド追加
   - 履歴管理を `Vec<usize>` （ラベルID）に変更
   - 前方一致検索ロジック実装（HashMap + フルスキャン）

2. **`crates/pasta/src/stdlib/mod.rs`**
   - `select_label_to_id()` のスタブを実装版に置き換え
   - `create_module()` で `Arc<Mutex<LabelTable>>` をキャプチャ
   - Rune Value → HashMap 変換処理

3. **`crates/pasta/src/error.rs`**
   - 新規エラー型追加:
     - `PastaError::NoMatchingLabel`
     - `PastaError::InvalidLabel`
     - `PastaError::RandomSelectionFailed`
     - `PastaError::DuplicateLabelName`
     - `PastaError::NoMoreLabels`

4. **`crates/pasta/src/engine.rs`**
   - `PastaEngine::new()` で `LabelTable` を `Arc<Mutex<>>` でラップ
   - `create_module()` に `LabelTable` を渡す

#### 互換性評価

- ✅ **後方互換性**: `find_label()` は変更せず、`resolve_label_id()` を追加するため既存コードは動作継続
- ✅ **インターフェース整合性**: `RandomSelector` トレイトをそのまま使用
- ⚠️ **破壊的変更**: `LabelInfo` 構造体にフィールド追加（デシリアライゼーション時の注意が必要）

#### 複雑性とメンテナビリティ

- **ファイルサイズ**: `labels.rs` は現在314行 → 約450行に増加（許容範囲）
- **単一責務原則**: ラベル解決という単一ドメインに集中（✅ 維持）
- **認知負荷**: `find_label()` と `resolve_label_id()` の2つのエントリーポイント（中程度）

#### Trade-offs

- ✅ 既存のテストコード（`tests/` 配下の `LabelTable` 使用箇所）が継続動作
- ✅ `from_label_registry()` の変更のみで統合可能
- ✅ アーキテクチャ変更なし
- ❌ `LabelInfo` へのIDフィールド追加が破壊的変更の可能性
- ❌ 履歴管理ロジックが2系統存在（`find_label()` はインデックス、`resolve_label_id()` はID）

**実装手順:**
1. `LabelInfo` に `id` 追加、`from_label_registry()` を修正
2. `resolve_label_id()` の骨格実装（前方一致検索なし、常に最初の候補を返す）
3. 前方一致検索ロジック追加（HashMap + フルスキャン）
4. 履歴管理をIDベースに変更
5. Rust ↔ Rune ブリッジ実装
6. テストケース追加

---

### Option B: Create New LabelResolver Component

**概要:** `LabelTable` は変更せず、新しい `LabelResolver` コンポーネントを作成し、前方一致検索専用とする。

#### 責務分離

- **LabelTable**: 既存の完全一致検索、属性フィルタリング、履歴管理（変更なし）
- **LabelResolver**: 前方一致検索、IDベース履歴、`resolve_label_id()` の実装

#### 統合ポイント

```rust
// 新規ファイル: crates/pasta/src/runtime/label_resolver.rs
pub struct LabelResolver {
    labels_by_path: HashMap<String, LabelInfo>,  // fn_path → LabelInfo
    history: HashMap<String, Vec<usize>>,        // 検索キー → ラベルIDリスト
    random_selector: Box<dyn RandomSelector>,
}

impl LabelResolver {
    pub fn new(label_table: &LabelTable, random_selector: Box<dyn RandomSelector>) -> Self {
        // LabelTableから変換
    }
    
    pub fn resolve_label_id(&mut self, label: &str, filters: &HashMap<String, String>) -> Result<usize, PastaError> {
        // 前方一致検索 + フィルタリング + ランダム選択
    }
}
```

#### Trade-offs

- ✅ `LabelTable` を一切変更しない（完全に既存動作を保護）
- ✅ 責務が明確に分離
- ✅ テストが独立
- ❌ 2つのラベル管理コンポーネントが並存（混乱の可能性）
- ❌ `PastaEngine` が2つのコンポーネントを保持する必要
- ❌ `LabelTable` → `LabelResolver` の変換オーバーヘッド

**非推奨理由:**
- ラベル解決は本質的に同一ドメイン（分離する必要性が低い）
- 既存の `LabelTable` が既にフィルタリングやランダム選択を実装済み（重複）

---

### Option C: Hybrid Approach（段階的移行）

**概要:** Phase 1で Option A の実装を行い、Phase 2で `find_label()` を非推奨化し、`resolve_label_id()` に統一する。

#### Phase 1: 並存期間
- `resolve_label_id()` を追加（Option A と同じ）
- `find_label()` は変更せず維持
- 新規コードは `resolve_label_id()` を使用

#### Phase 2: 段階的移行（将来）
- `find_label()` に `#[deprecated]` 属性を追加
- 全ての呼び出し箇所を `resolve_label_id()` に置き換え
- 履歴管理を統一（IDベースのみ）

#### Trade-offs

- ✅ 段階的な移行でリスク低減
- ✅ 既存コードの動作保証
- ✅ 将来的に単一のAPIに統一可能
- ❌ Phase 2の実装が不確実（技術的負債の可能性）
- ❌ 移行期間中は2つのAPIが並存（ドキュメント負荷）

---

## 4. Recommended Approach & Key Decisions

### 推奨: Vec-based ID Storage with Multi-phase Search

**理由:**
1. **最適なデータ構造**: ラベル削除なし → Vec indexで十分、メモリ効率最高
2. **パフォーマンス**: Vec iterationは最速（prefix-match検索で頻繁に使用）
3. **ID-based access**: LabelInfoのコピー/クローン排除、IDで参照
4. **テスト容易性**: shuffle_enabled フラグで決定論的テスト可能

### 主要な設計決定

#### Decision 1: データ構造 - Trie prefix index + Vec storage

**選択: `RadixMap<Vec<LabelId>>` (fast_radix_trie) + `Vec<LabelInfo>` with `LabelId(usize)` newtype**

**理由:**
- **Trie prefix index**: O(M)検索（Mは検索キー長）、ラベル数に依存しない
- **Vec storage**: ラベル削除なし → Vec indexで十分（シンプル、高速）
- **分離設計**: Trieは検索インデックス、Vecがデータ本体（責務分離）
- **Trie value**: `Vec<LabelId>` - 同一プレフィックスの全ID保持

**fast_radix_trieを選ぶ理由:**
- **最新メンテナンス**: v1.1.0 (2025年12月3日push)、rust-version 1.85.0対応
- **メモリ効率**: ベンチマークで他のTrie実装より高速&省メモリ
- **API設計**: `common_prefixes()` メソッドが前方一致検索に最適
- **ライセンス**: MIT (問題なし)

**他の実装を選ばない理由:**
- **qp_trie**: crates.ioに存在しない
- **radix_trie**: v0.3.0 (2025年9月16日push)、古め、APIが複雑
- **Vec全件走査**: O(N)は非効率（ラベル数増加で線形劣化）

#### Decision 2: 検索アルゴリズム - Multi-phase Search with Trie

**選択: 4フェーズ検索戦略**

```rust
// Phase 1: Trie prefix search O(M) - M is search_key length
let candidate_ids: Vec<LabelId> = self.prefix_index
    .common_prefixes(search_key.as_bytes())
    .flat_map(|(_key, ids)| ids.iter().copied())
    .collect();

// Phase 2: Filter by secondary criteria
let filtered_ids: Vec<LabelId> = candidate_ids
    .into_iter()
    .filter(|&id| self.matches_filters(id, filters))
    .collect();

// Phase 3: Get or create cache (shuffle once if enabled)
let cached = self.cache.entry(search_key.to_string())
    .or_insert_with(|| {
        let mut ids = filtered_ids.clone();
        if self.shuffle_enabled {
            self.random_selector.shuffle(&mut ids);
        }
        CachedSelection { candidates: ids, next_index: 0, history: Vec::new() }
    });

// Phase 4: Sequential selection from cache
let selected_id = cached.candidates[cached.next_index];
cached.next_index += 1;
cached.history.push(selected_id);
```

**理由:**
- **Phase 1**: Trie prefix search O(M) - 検索キー長のみ依存、ラベル数に依存しない
- **Phase 2**: フィルタリング（e.g., "::選択肢"を含む）
- **Phase 3**: キャッシュ作成とシャッフル（初回のみ、決定論的テスト可能）
- **Phase 4**: 順次選択（履歴管理、ロールバック対応）

**パフォーマンス:**
- ラベル数1000でも O(M) - 定数時間に近い
- メモリトレードオフ: Trieインデックス分のオーバーヘッド（許容範囲）

#### Decision 3: キャッシュ管理 - CachedSelection構造体

**選択: search_key単位でキャッシュ、シャッフルは初回のみ**

```rust
struct CachedSelection {
    candidates: Vec<LabelId>,  // Shuffled on first access
    next_index: usize,         // Sequential selection pointer
    history: Vec<LabelId>,     // Selection history (IDs, not indices)
}
```

**理由:**
- **初回シャッフル**: ランダム性と決定論の両立（shuffle_enabledフラグで制御）
- **順次選択**: next_indexで進行、全候補消化後にNoMoreLabelsエラー
- **履歴記録**: LabelIdベースで安定（インデックスではない）
- **デバッグモード**: shuffle_enabled = false でVec順序そのまま

**代替案を選ばない理由:**
- ~~毎回シャッフル~~: 決定論的テスト不可、パフォーマンス懸念
- ~~インデックスベース履歴~~: フィルタ変動に脆弱、候補順序変更で無効化

#### Decision 4: LabelTable参照の渡し方

**選択: `create_module()` で `Arc<Mutex<LabelTable>>` をキャプチャ**

**理由:**
- 既存の `persistence` モジュールと同じパターン（`stdlib/persistence.rs` 参照）
- Runeのシグネチャを変更不要（`select_label_to_id(label, filters)` のまま）

**実装例:**
```rust
// stdlib/mod.rs
pub fn create_module(label_table: Arc<Mutex<LabelTable>>) -> Result<Module, ContextError> {
    let mut module = Module::with_crate("pasta_stdlib")?;
    
    let lt = Arc::clone(&label_table);
    module.function("select_label_to_id", move |label: String, filters: rune::runtime::Value| -> Result<i64, String> {
        // Rune Value → HashMap 変換
        let filters_map = parse_rune_filters(filters)?;
        
        // LabelTableを呼び出し
        let mut table = lt.lock().unwrap();
        let id = table.resolve_label_id(&label, &filters_map)
            .map_err(|e| e.to_string())?;
        
        Ok(id as i64)
    }).build()?;
    
    Ok(module)
}
```

#### Decision 5: エラーハンドリング戦略

**選択: 構造化エラー追加 + Rune側でpanic**

**エラー型追加:**
```rust
// crates/pasta/src/error.rs
#[derive(Error, Debug)]
pub enum PastaError {
    // 既存エラー（変更なし）
    #[error("Label not found: {label}")]
    LabelNotFound { label: String },  // ← 既存（前方一致検索前の基本エラー）
    
    // 新規エラー（本仕様で追加）
    #[error("No matching label for '{label}' with filters {filters:?}")]
    NoMatchingLabel {
        label: String,
        filters: HashMap<String, String>,
    },  // ← フィルタ適用後に候補が0件
    
    #[error("Invalid label name: '{label}'")]
    InvalidLabel { label: String },  // ← 空文字列など不正なラベル名
    
    #[error("Random selection failed")]
    RandomSelectionFailed,  // ← RandomSelector::select_index() が None 返却
    
    #[error("Duplicate label name: {name}")]
    DuplicateLabelName { name: String },  // ← LabelRegistry変換時のfn_name重複検出
    
    #[error("No more labels for '{search_key}' with filters {filters:?}")]
    NoMoreLabels {
        search_key: String,
        filters: HashMap<String, String>,
    },  // ← キャッシュの順次選択で全候補を使い果たした
}
```

**Rune側での処理:**
```rune
// トランスパイラーが生成（変更なし）
let id = pasta_stdlib::select_label_to_id(label, filters);
match id {
    1 => crate::会話_1::__start__,
    _ => { yield pasta_stdlib::Error(`ラベルID ${id} が見つかりませんでした。`); },
}
```

---

## 5. Implementation Complexity & Risk

### Effort: M（3-7日）

**内訳:**
- **Day 1-2**: `LabelInfo` へのIDフィールド追加、`resolve_label_id()` の骨格実装、単体テスト
- **Day 3-4**: 前方一致検索ロジック、履歴管理の変更、追加テストケース
- **Day 5-6**: Rust ↔ Rune ブリッジ実装、Rune Value変換、統合テスト
- **Day 7**: エンドツーエンドテスト、ドキュメント更新、エッジケース対応

**既存パターンの活用:**
- `RandomSelector` トレイト、`MockRandomSelector` をそのまま使用（Day 0.5削減）
- `from_label_registry()` の変更のみで統合（Day 1削減）
- `tests/common/` のユーティリティ再利用（Day 0.5削減）

### Risk: Low

**理由:**
- ✅ **既知技術**: Rust標準ライブラリ（HashMap, Vec）のみ、外部クレート依存なし
- ✅ **明確な統合ポイント**: `from_label_registry()`, `create_module()` の変更箇所が限定的
- ✅ **テスト容易性**: `MockRandomSelector` で決定論的テスト可能
- ✅ **パフォーマンス懸念小**: O(N)の走査コスト、想定ラベル数で問題なし

**潜在的リスクと対策:**

| リスク | 影響 | 対策 |
|--------|------|------|
| Rune Value → HashMap 変換失敗 | Medium | Rune公式ドキュメント調査、fallback処理実装 |
| 前方一致検索の誤動作（連番処理） | Medium | 包括的なユニットテスト、エッジケース網羅 |
| 履歴管理のメモリリーク | Low | 履歴クリア処理の実装、メモリ使用量テスト |
| LabelInfo構造体変更による互換性問題 | Low | デフォルト値設定、移行テスト |

---

## 6. Research Items for Design Phase

### Research 1: Rune Value → HashMap 変換

**調査項目:**
- `rune::runtime::Value::into_object()` の使用方法
- Rune Object から Rust HashMap への変換パターン
- 空のHashMapが渡された場合の扱い（`Value::Unit` か `Value::Object(empty)` か）

**参照:**
- Rune公式ドキュメント: https://rune-rs.github.io/
- 既存の `persistence` モジュール（`stdlib/persistence.rs`）

### Research 2: 前方一致検索の最適化

**調査項目:**
- ラベル数が1000以上になった場合のパフォーマンス測定
- `prefix_tree` クレートまたは `radix_trie` クレートの評価
- Trie構造導入時のメモリオーバーヘッド

**判断基準:**
- 解決時間が10ms以上かかる場合は最適化を検討
- メモリ使用量が10MB以下であればTrie導入可

### Research 3: 履歴管理のメモリ効率化

**調査項目:**
- 長時間実行時の履歴メモリ使用量の増加傾向
- LRUキャッシュの導入可否
- 履歴の自動クリア戦略（例: N回実行後、またはメモリ閾値超過時）

---

## 7. Test Strategy

### 既存テストパターンの活用

**再利用可能なテストユーティリティ:**
- `tests/common/create_test_script()`: 一時的なスクリプトファイル生成
- `tests/common/create_unique_persistence_dir()`: 独立した永続化ディレクトリ
- `runtime::random::MockRandomSelector`: 決定論的ランダム選択

### 新規テストケース

| テストカテゴリ | テストケース数 | ファイル配置案 |
|---------------|--------------|--------------|
| **ユニットテスト** | 15 | `crates/pasta/src/runtime/labels.rs` (`#[cfg(test)]` セクション) |
| **統合テスト** | 5 | `crates/pasta/tests/label_resolution_test.rs` |
| **エンドツーエンド** | 3 | `crates/pasta/tests/comprehensive_control_flow_test.rs` (既存拡張) |

**ユニットテストケース（抜粋）:**
1. `test_resolve_label_id_forward_match`: 前方一致検索の基本動作
2. `test_resolve_label_id_with_filters`: 属性フィルタリング
3. `test_resolve_label_id_cache_exhaustion`: キャッシュ全消化とリセット
4. `test_resolve_label_id_history_by_filter`: フィルタごとの履歴分離
5. `test_resolve_label_id_empty_label`: 空文字列エラー
6. `test_resolve_label_id_no_candidates`: 候補なしエラー
7. `test_resolve_label_id_filter_no_match`: フィルタ不一致エラー

---

## 8. Requirement-to-Asset Map

| 要件 | 既存アセット | Gap/Constraint | 実装アプローチ |
|------|------------|---------------|--------------|
| **Req 1: 前方一致検索** | `LabelTable::find_label()` (完全一致) | ❌ Missing: 前方一致ロジック | HashMap + フルスキャン実装 |
| **Req 2: 属性フィルタリング** | ✅ `LabelTable::find_label()` のフィルタリングロジック | - | 再利用 |
| **Req 3: ランダム選択** | ✅ `RandomSelector` トレイト、`MockRandomSelector` | - | 再利用 |
| **Req 4: キャッシュベース消化** | `LabelTable::history` (インデックスベース) | ⚠️ Constraint: IDベースに変更必要 | 履歴管理を `Vec<usize>` (ID) に変更 |
| **Req 5: Rust ↔ Rune ブリッジ** | `stdlib::select_label_to_id()` (スタブ) | ❌ Missing: 実装版、型変換 | `create_module()` で `Arc<Mutex<LabelTable>>` キャプチャ |
| **Req 6: Registry → Table 変換** | ✅ `LabelTable::from_label_registry()` | ⚠️ Constraint: IDフィールド欠落 | `LabelInfo` に `id` 追加 |

---

## 9. Summary & Next Steps

### 実装推奨事項

1. **Approach:** Option A（Extend Existing LabelTable）を採用
2. **Priority 1:** `LabelInfo` へのIDフィールド追加、`from_label_registry()` 修正
3. **Priority 2:** `resolve_label_id()` の骨格実装（前方一致なし、単純な最初候補返却）
4. **Priority 3:** 前方一致検索ロジック（HashMap + フルスキャン）
5. **Priority 4:** Rust ↔ Rune ブリッジ実装、型変換

### Design Phase への引き継ぎ

**確定事項:**
- 既存 `LabelTable` の拡張方針
- HashMap + フルスキャンでの前方一致検索
- IDベースの履歴管理
- `create_module()` でのクロージャーキャプチャ

**要研究事項:**
- Rune Value → HashMap の具体的変換コード
- エッジケースの網羅的なテストケース設計
- パフォーマンステストの実施計画

### Design Generation へ進む条件

- ✅ Gap分析完了
- ✅ 実装アプローチ決定
- ✅ 主要な設計決定完了

**次のステップ:**
- `/kiro-spec-design pasta-label-resolution-runtime` でデザインフェーズへ進行
- または、Gap分析の結果を踏まえて要件定義を修正

---

## References

- **親仕様:** `.kiro/specs/completed/pasta-declarative-control-flow/`
- **現在の実装:** `crates/pasta/src/runtime/labels.rs`
- **標準ライブラリ:** `crates/pasta/src/stdlib/mod.rs`
- **エラー型:** `crates/pasta/src/error.rs`
- **テストユーティリティ:** `crates/pasta/tests/common/`
