````markdown
# Requirements Document: pasta-label-resolution-runtime

| 項目 | 内容 |
|------|------|
| **Document Title** | Pasta DSL ランタイムラベル解決システム 要件定義書 |
| **Version** | 2.0 |
| **Date** | 2025-12-14 |
| **Parent Spec** | pasta-declarative-control-flow (completed) |
| **Priority** | P1 |
| **Status** | Requirements Refinement |

---

## Introduction

本要件定義書は、Pasta DSLにおける**実行時ラベル解決機能**を定義する。これは、トランスパイラーが生成する `pasta_stdlib::select_label_to_id()` 関数のRust実装であり、宣言的コントロールフロー（call/jump文）の実行時動作を実現する中核機能である。

### Background

親仕様 `pasta-declarative-control-flow` (P0) において、Pasta DSLからRuneコードへのトランスパイラーが実装され、以下の構造が確立された：

```rune
// トランスパイラーが生成するコード
pub mod 会話_1 {
    pub fn __start__(ctx) { /* ... */ }
    pub fn 選択肢_1(ctx) { /* ... */ }
    pub fn 選択肢_2(ctx) { /* ... */ }
}

pub mod pasta {
    pub fn label_selector(label, filters) {
        let id = pasta_stdlib::select_label_to_id(label, filters);
        match id {
            1 => crate::会話_1::__start__,
            2 => crate::会話_1::選択肢_1,
            3 => crate::会話_1::選択肢_2,
            _ => panic!("Unknown label id: {}", id),
        }
    }
}
```

現在の `label_selector()` は**仮実装**であり、常に `id = 1` を返すため、実際のラベル名解決が機能していない。本仕様では、この関数が呼び出す **Rust側のラベル解決エンジン** (`LabelTable::resolve_label_id()`) を実装する。

### Problem Statement

**課題1: 前方一致検索の未実装**

Pasta DSLの設計では、ラベル名は前方一致で解決される：

```pasta
＊会話
    ＞選択肢      # → "会話_1::選択肢_1" または "会話_1::選択肢_2" にマッチ

    ・選択肢
        さくら：選択肢Aです

    ・選択肢
        さくら：選択肢Bです
```

検索キー `"会話_1::選択肢"` に対して、`"会話_1::選択肢_1"` と `"会話_1::選択肢_2"` が候補となるが、現在の `LabelTable` は**完全一致検索**（`HashMap::get()`）のみをサポートしており、前方一致が機能しない。

**課題2: キャッシュベース消化の非効率性**

現在の `LabelTable::find_label()` は `history` に**配列のインデックス**を記録するが、これは以下の問題を引き起こす：

1. **前方一致候補が変動する**: フィルタの違いにより、同じ検索キーでも候補が異なる場合、履歴のインデックスが無効になる
2. **削除・追加に脆弱**: ラベルの追加・削除によりインデックスがずれる

正しい設計では、`history` にはラベルID（`usize`）を記録すべきであり、これにより候補の変動に対しても堅牢性が保たれる。

**課題3: RustとRuneの型システム不整合**

トランスパイラー設計では、`filters` は Rune の `HashMap` として渡されるが、現在の `resolve_label_id()` は Rust の `HashMap<String, String>` を期待している。Rune → Rust の型変換が明示的に定義されていない。

### Scope

**含まれるもの：**

1. **ラベル解決エンジンの実装** (`LabelTable::resolve_label_id()`)
   - 前方一致検索（検索キー → fn_path のプレフィックスマッチ）
   - 属性フィルタリング（`＆time:morning` 等の属性による絞り込み）
   - ランダム選択（`RandomSelector` 統合）
   - キャッシュベース消化（履歴管理による選択肢の順次消化）

2. **Rust ↔ Rune ブリッジの実装** (`PastaApi::create_module()`)
   - `pasta_stdlib::select_label_to_id()` 関数の Rune モジュール登録
   - Rune HashMap → Rust HashMap の型変換

3. **LabelRegistry → LabelTable 変換の実装**
   - トランスパイラーが生成する `LabelRegistry` をランタイム用 `LabelTable` に変換

**含まれないもの：**

- トランスパイラーの変更（P0で完了）
- `pasta::call()` / `pasta::jump()` の実装（P0で完了）
- 単語辞書解決 (`WordDictionary`, 別仕様で実装)

---

## Requirements

### Requirement 1: 前方一致ラベル検索

**Objective:** スクリプト作成者として、ラベル名の前方一致による動的解決を行い、ローカルラベルやバリエーション定義を柔軟に参照できるようにする。

#### Context

Pasta DSLでは、以下の検索キー生成規則が定義されている（親仕様より）：

| DSL構文 | 検索キー生成 | 前方一致パターン |
|---------|-------------|-----------------|
| `＞選択肢` | `"親ラベル名_番号::選択肢"` | `"会話_1::選択肢_*"` |
| `＞＊会話` | `"会話"` | `"会話_*::__start__"` |
| `＞＊会話・選択肢` | `"会話::選択肢"` | `"会話_*::選択肢_*"` |

検索キーは**完全修飾名の一部**であり、fn_path（`"会話_1::__start__"`）との前方一致で候補を抽出する。

#### Acceptance Criteria

1. When ラベル解決エンジンが検索キー `"会話"` を受け取る, the LabelTable shall fn_path が `"会話"` で始まるすべてのラベルを前方一致検索で抽出し、`"::__start__"` で終わるラベルのみを候補とする（例: `"crate::会話_1::__start__"`, `"crate::会話_2::__start__"`）
2. When ラベル解決エンジンが検索キー `"会話_1::選択肢"` を受け取る, the LabelTable shall fn_path が `"会話_1::選択肢"` で始まるすべてのラベルを前方一致検索で抽出する（例: `"crate::会話_1::選択肢_1"`, `"crate::会話_1::選択肢_2"`）
3. When 前方一致する候補が存在しない場合, the LabelTable shall `PastaError::LabelNotFound { label: <検索キー> }` エラーを返す
4. When 検索キーが空文字列の場合, the LabelTable shall `PastaError::InvalidLabel` エラーを返す
5. When fn_path に連番が含まれる（`"会話_1"`, `"会話_2"`）場合, the LabelTable shall 連番の違いを無視して前方一致検索を実行する（`"会話"` で `"会話_1"`, `"会話_2"` どちらもマッチ）
   - **注記:** 検索キー生成はトランスパイラーが実施。`JumpTarget::Global("会話")` → `"会話"`, `JumpTarget::LongJump{"会話", "選択肢"}` → `"会話::選択肢"`

### Requirement 2: 属性フィルタリング

**Objective:** スクリプト作成者として、ラベル定義に付与した属性（`＆time:morning`）を使用して、実行時の状況に応じた会話分岐を実現する。

#### Context

Pasta DSLでは、ラベル定義に属性を付与できる：

```pasta
＊会話＆time:morning
    さくら：おはよう！

＊会話＆time:evening
    さくら：こんばんは！
```

call/jump文で属性フィルタを指定すると、条件に合致するラベルのみが候補となる：

```pasta
＊メイン
    ＞＊会話（＆time:morning）    # morning属性のみマッチ
```

属性は `HashMap<String, String>` として管理され、複数指定時はAND条件となる。

#### Acceptance Criteria

1. When ラベル解決エンジンが filters パラメータに `{"time": "morning"}` を受け取る, the LabelTable shall 前方一致候補のうち `attributes["time"] == "morning"` を持つラベルのみを残す
2. When 複数のフィルタが指定される（例: `{"time": "morning", "weather": "sunny"}`）, the LabelTable shall すべてのフィルタ条件を満たすラベルのみを残す（AND条件）
3. When フィルタ適用後の候補が0件になる場合, the LabelTable shall `PastaError::NoMatchingLabel { label: <検索キー>, filters: <適用フィルタ> }` エラーを返す
4. When filters パラメータが空の HashMap の場合, the LabelTable shall フィルタリングをスキップし、前方一致候補をそのまま返す
5. When ラベルが属性を持たず、filters が指定される場合, the LabelTable shall そのラベルを候補から除外する

### Requirement 3: ランダム選択

**Objective:** スクリプト作成者として、同名ラベルの複数定義からランダムに選択させることで、会話のバリエーションを自然に表現する。

#### Context

Pasta DSLでは、同名ラベルの複数定義が推奨される：

```pasta
＊挨拶
    さくら：やあ！

＊挨拶
    さくら：こんにちは！

＊挨拶
    さくら：どうも！
```

実行時には、これら3つの候補からランダムに1つが選択される。選択ロジックは `RandomSelector` トレイトに委譲され、テスト時にはモック実装に差し替え可能である。

#### Acceptance Criteria

1. When 前方一致およびフィルタ適用後の候補が複数存在する, the LabelTable shall `RandomSelector::select_index()` を呼び出し、候補の中から1つを選択する
2. When 候補が1つのみの場合, the LabelTable shall ランダム選択をスキップし、その候補を直接返す
3. When `RandomSelector::select_index()` が `None` を返す（選択失敗）場合, the LabelTable shall `PastaError::RandomSelectionFailed` エラーを返す
4. When 同一検索キー・同一フィルタで2回目の呼び出しが発生する, the LabelTable shall 異なる候補を返す（Requirement 4のキャッシュベース消化と連携）

### Requirement 4: キャッシュベース消化

**Objective:** スクリプト作成者として、同一の選択肢を繰り返し呼び出した際に、すべての選択肢を順次消化してからリセットされる挙動を実現する。

#### Context

里々（Satori）の仕様を踏襲し、同じ検索キーでラベルを呼び出す度に異なる候補が選ばれ、全候補が消化されるまでリセットされない。これにより、会話の自然なバリエーション展開が可能になる。

**現在の実装の問題点：**

現在の `LabelTable::find_label()` は `history` に**配列のインデックス**を記録する：

```rust
// 現在の実装（問題あり）
self.history
    .entry(name.to_string())
    .or_insert_with(Vec::new)
    .push(
        candidates
            .iter()
            .position(|l| l.fn_name == selected.fn_name)
            .unwrap(),  // ← 配列のインデックスを記録
    );
```

これは以下のケースで不具合を引き起こす：

1. **フィルタが異なる場合**: 同じ検索キーでも、フィルタにより候補が変わるため、インデックスが無効化される
2. **ラベル追加・削除**: スクリプト変更により候補の順序が変わると、履歴が矛盾する

**正しい設計：**

`history` には **ラベルID（`usize`）** を記録すべきである。これにより、候補リストの変動に対して堅牢性が保たれる。

#### Acceptance Criteria

1. When 同一の検索キーで2回目の呼び出しが発生する, the LabelTable shall `history` に記録されたラベルIDを除外し、未選択の候補から選択する
2. When 未選択の候補が存在しない場合（全消化）, the LabelTable shall `history` をクリアし、再度すべての候補を選択対象とする
3. When フィルタが異なる呼び出しが発生する, the LabelTable shall 検索キーとフィルタの組み合わせを別の履歴として管理する（例: `history` のキーを `format!("{}:{:?}", label, filters)` とする）
4. When 履歴に記録されるのは選択されたラベルのID (`LabelInfo::id`)であり, the LabelTable shall 配列のインデックスではなくIDを使用して履歴管理を行う
5. When 全候補が消化され履歴がクリアされる, the LabelTable shall ログまたはトレース出力を生成し、デバッグ時に動作を確認できるようにする

### Requirement 5: Rust ↔ Rune ブリッジ実装

**Objective:** 開発者として、Runeコードから呼び出される `pasta_stdlib::select_label_to_id()` 関数を実装し、`LabelTable::resolve_label_id()` をRune VMに公開する。

#### Context

トランスパイラーが生成するRuneコード：

```rune
pub mod pasta {
    pub fn label_selector(label, filters) {
        let id = pasta_stdlib::select_label_to_id(label, filters); // ← Rust関数呼び出し
        match id {
            1 => crate::会話_1::__start__,
            _ => panic!("Unknown label id: {}", id),
        }
    }
}
```

この `pasta_stdlib::select_label_to_id()` は、Rune側からRust側の `LabelTable::resolve_label_id()` を呼び出すブリッジ関数である。

**型変換の課題：**

Runeの `HashMap` は、Rustの `HashMap<String, String>` とは異なる型システムを持つ。Runeの `Object` 型から Rust の `HashMap` への変換が必要である。

#### Acceptance Criteria

1. When `PastaApi::create_module()` が呼ばれる, the PastaApi shall `pasta_stdlib` モジュール配下に `select_label_to_id` 関数を登録する
2. When Runeコードから `select_label_to_id(label, filters)` が呼ばれる, the PastaApi shall Runeの引数を Rust の `&str` および `HashMap<String, String>` に変換する
3. When 型変換が失敗した場合（例: filters が Object 型でない）, the PastaApi shall Rune側で `panic!` を発生させ、スクリプト実行を中断する
4. When `LabelTable::resolve_label_id()` がエラーを返す, the PastaApi shall エラーメッセージを Rune の String として返し、Rune側で `panic!` させる
5. When `LabelTable` が `Arc<Mutex<LabelTable>>` として保持される, the PastaApi shall ロック取得失敗時に適切なエラーメッセージを返す

### Requirement 6: LabelRegistry → LabelTable 変換

**Objective:** 開発者として、トランスパイラーが生成する `LabelRegistry` をランタイム用の `LabelTable` に変換し、ID割り当てとデータ構造の最適化を行う。

#### Context

トランスパイラー（Pass 1）は `LabelRegistry` にラベル情報を蓄積する：

```rust
pub struct LabelRegistry {
    labels: Vec<TranspileLabelInfo>,
    id_counter: usize,
}

pub struct TranspileLabelInfo {
    pub id: usize,
    pub name: String,                  // DSL上のラベル名
    pub attributes: HashMap<String, String>,
    pub fn_path: String,               // "会話_1::__start__"
    pub parent: Option<String>,
}
```

この `LabelRegistry` は、トランスパイル完了後、ランタイムの `LabelTable` に変換される。`LabelTable` は前方一致検索に最適化されたデータ構造（例: Trie）を使用すべきである。

#### Acceptance Criteria

1. When `LabelTable::from_label_registry()` が呼ばれる, the LabelTable shall `LabelRegistry` の全エントリを内部データ構造に変換する
2. When 変換処理が行われる, the LabelTable shall fn_path をキーとした前方一致検索可能なデータ構造（HashMap または Trie）を構築する
3. When 同一の fn_path を持つラベルが存在する場合（通常はありえない）, the LabelTable shall `PastaError::DuplicateLabelPath` エラーを返す
4. When `RandomSelector` インスタンスが渡される, the LabelTable shall それを内部で保持し、ランダム選択時に使用する
5. When 変換完了後の `LabelTable` が `Send` トレイトを実装している, the LabelTable shall マルチスレッド環境での使用を保証する（Rune VMは Send を要求）
   - **注記:** `RandomSelector` トレイトは既に `Send + Sync` 境界を持つため、`LabelTable` は自動的に `Send` を実装する

---

## Technical Context

### 現在の実装状況

**実装済み（P0完了）：**

```rust
// crates/pasta/src/runtime/labels.rs
pub struct LabelTable {
    labels: HashMap<String, Vec<LabelInfo>>,  // ⚠️ 完全一致検索のみ
    history: HashMap<String, Vec<usize>>,     // ⚠️ 配列インデックスを記録（問題あり）
    random_selector: Box<dyn RandomSelector>,
}

impl LabelTable {
    pub fn find_label(
        &mut self,
        name: &str,
        filters: &HashMap<String, String>,
    ) -> Result<String, PastaError> {
        // ⚠️ HashMap::get() による完全一致検索
        let candidates = self.labels.get(name)?;
        
        // フィルタリング
        let matching: Vec<&LabelInfo> = candidates
            .iter()
            .filter(|label| /* ... */)
            .collect();
        
        // ランダム選択（実装済み）
        let selected_idx = self.random_selector.select_index(matching.len())?;
        
        // ⚠️ 配列インデックスを履歴に記録
        self.history
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(/* 配列のインデックス */);
        
        Ok(selected.fn_name.clone())
    }
}
```

**未実装（本仕様の対象）：**

1. `LabelTable::resolve_label_id()` メソッド
2. 前方一致検索のためのデータ構造変更（Trie または fn_path のイテレーション）
3. ラベルIDベースの履歴管理（現在はインデックス）
4. `PastaApi::create_module()` と `select_label_to_id()` 関数
5. `LabelTable::from_label_registry()` 実装

### データ構造の設計選択

**設計決定: Trie-based prefix index with Vec storage**

```rust
use fast_radix_trie::RadixMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LabelId(usize);  // newtype wrapper for Vec index

pub struct LabelInfo {
    pub id: LabelId,  // ← 必須: ラベルの一意識別子
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub fn_path: String,
    pub parent: Option<String>,
}

// キャッシュキー: (前方一致検索キー, フィルタ条件) のペア
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    search_key: String,
    filters: Vec<(String, String)>,  // Sorted for consistent hashing
}

impl CacheKey {
    fn new(search_key: &str, filters: &HashMap<String, String>) -> Self {
        let mut filter_vec: Vec<_> = filters.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        filter_vec.sort();  // Ensure consistent ordering
        Self {
            search_key: search_key.to_string(),
            filters: filter_vec,
        }
    }
}

pub struct LabelTable {
    labels: Vec<LabelInfo>,  // ID-based storage (index = LabelId)
<<<<<<< HEAD
    prefix_index: RadixMap<Vec<LabelId>>,  // fn_path → [LabelId] for prefix search
    cache: HashMap<String, CachedSelection>,  // search_key → shuffled IDs + history
=======
    prefix_index: RadixMap<Vec<LabelId>>,  // fn_name → [LabelId] for prefix search
    cache: HashMap<CacheKey, CachedSelection>,  // (search_key, filters) → shuffled IDs + history
>>>>>>> 934c1a65c625dc28e18ef5f7bc5699b1f5036e98
    random_selector: Box<dyn RandomSelector>,
    shuffle_enabled: bool,  // Default: true (false for deterministic testing)
}

struct CachedSelection {
    candidates: Vec<LabelId>,  // Shuffled on first access
    next_index: usize,  // Sequential selection from candidates
    history: Vec<LabelId>,  // Selection history for rollback
}

impl LabelTable {
    pub fn resolve_label_id(
        &mut self,
        search_key: &str,
        filters: &HashMap<String, String>,
    ) -> Result<LabelId, PastaError> {
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
        
        // Phase 3: Get or create cache entry with (search_key, filters) as key
        let cache_key = CacheKey::new(search_key, filters);
        let cached = self.cache.entry(cache_key)
            .or_insert_with(|| {
                let mut ids = filtered_ids.clone();
                if self.shuffle_enabled {
                    self.random_selector.shuffle(&mut ids);  // Shuffle once
                }
                CachedSelection {
                    candidates: ids,
                    next_index: 0,
                    history: Vec::new(),
                }
            });
        
        // Phase 4: Sequential selection from cache
        if cached.next_index >= cached.candidates.len() {
            return Err(PastaError::NoMoreLabels { 
                search_key: search_key.to_string(),
                filters: filters.clone(),
            });
        }
        
        let selected_id = cached.candidates[cached.next_index];
        cached.next_index += 1;
        cached.history.push(selected_id);
        
        Ok(selected_id)
    }
    
    pub fn get_label(&self, id: LabelId) -> Option<&LabelInfo> {
        self.labels.get(id.0)
    }
}
```

**設計根拠:**

1. **Trie prefix index:**
   - **検索性能**: O(M) - Mは検索キーの長さ（ラベル数に依存しない）
   - **Trie value**: `Vec<LabelId>` - 同一プレフィックスの全IDを保持
   - **実装**: `fast_radix_trie` (v1.1.0) - メモリ効率最高、最新メンテナンス
   - **構築**: `from_label_registry()` 時に1回だけ構築（不変）
   - **API**: `RadixMap::common_prefixes()` で前方一致検索

2. **Vec storage:**
   - ラベルは削除されない → `Vec<LabelInfo>` で十分
   - `Vec` index = `LabelId` （シンプル、高速アクセス）
   - Trieは検索インデックスのみ、データ本体はVecに格納

3. **ID-based access:**
   - `LabelInfo` をコピー/クローンせず、IDで参照
   - キャッシュは `Vec<LabelId>` を保持（データ重複なし）
   - ランダム選択の対象はIDのリスト（軽量）

4. **Multi-phase search:**
   - Phase 1: Trie prefix search O(M) → 候補ID列挙
   - Phase 2: Filter by secondary criteria (e.g., "::選択肢")
   - Phase 3: Cache manager shuffles (if enabled) and stores
   - Phase 4: Sequential selection from cache with history tracking

5. **Shuffle strategy:**
   - `shuffle_enabled = true` (デフォルト): 初回アクセス時にシャッフル、その後は順次選択
   - `shuffle_enabled = false` (デバッグ): Trie順序そのまま、決定論的テスト可能
   - キャッシュエントリごとに独立してシャッフル実行（(search_key, filters)が異なれば別管理）

**実装アルゴリズム:**
```rust
// 1. LabelRegistry → Vec<LabelInfo> + Trie index 構築
impl LabelTable {
    pub fn from_label_registry(
        registry: &LabelRegistry,
        random_selector: Box<dyn RandomSelector>,
        shuffle_enabled: bool,
    ) -> Result<Self, PastaError> {
        // Step 1: Build Vec storage
        let labels: Vec<LabelInfo> = registry
            .labels
            .iter()
            .enumerate()
            .map(|(idx, (name, trans_label))| {
                LabelInfo {
                    id: LabelId(idx),  // Vec index = ID
                    name: name.clone(),
                    fn_path: trans_label.fn_path.clone(),
                    attributes: trans_label.attributes.clone(),
                    parent: trans_label.parent.clone(),
                }
            })
            .collect();
        
        // Step 2: Build Trie prefix index
        let mut prefix_index = RadixMap::new();
        for label in &labels {
            prefix_index
                .entry(label.fn_path.as_bytes())
                .or_insert_with(Vec::new)
                .push(label.id);
        }
        
        Ok(LabelTable {
            labels,
            prefix_index,
            cache: HashMap::new(),
            random_selector,
            shuffle_enabled,
        })
    }
}

// 2. 検索時にTrie prefix search（O(M) - Mは検索キー長）
// 3. グローバルラベル検索時は "::__start__" で終わるものをフィルタ
// 例: search_key="会話" → RadixMap.common_prefixes("会話") → "会話_1::__start__", "会話_2::__start__" が候補
```

### エラーハンドリング

**新規エラー型の追加：**

```rust
// crates/pasta/src/error.rs
#[derive(Debug, thiserror::Error)]
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

---

## Testing Strategy

### Unit Tests

| テストケース | 入力 | 期待される出力 |
|-------------|------|--------------|
| **前方一致検索** | `search_key = "会話"` | `["会話_1::__start__", "会話_2::__start__"]` |
| **ローカルラベル検索** | `search_key = "会話_1::選択肢"` | `["会話_1::選択肢_1", "会話_1::選択肢_2"]` |
| **フィルタ適用** | `filters = {"time": "morning"}` | morning属性のみ残る |
| **ランダム選択** | 複数候補 | `RandomSelector::select_index()` が呼ばれる |
| **キャッシュ消化** | 同一検索キー2回 | 異なるIDが返る |
| **全消化リセット** | 全候補を消化 | 履歴がクリアされる |
| **エラー: 候補なし** | 存在しない検索キー | `PastaError::LabelNotFound` |
| **エラー: フィルタ不一致** | 全候補がフィルタで除外 | `PastaError::NoMatchingLabel` |

### Integration Tests

1. **エンドツーエンド実行テスト:**
   - Pasta DSL → トランスパイル → Rune実行 → ラベル解決 → 正しい関数が呼ばれる

2. **ランダム選択の再現性テスト:**
   - MockRandomSelector でシード固定 → 期待される順序でラベルが選ばれる

3. **マルチスレッド安全性テスト:**
   - `Arc<Mutex<LabelTable>>` を複数スレッドから呼び出し → デッドロックや不整合が発生しない

---

## Implementation Notes

### 実装の優先順位

1. **Phase 1: 基本機能** (必須)
   - HashMap + フルスキャンによる前方一致検索
   - 属性フィルタリング
   - ラベルIDベースの履歴管理

2. **Phase 2: Rune統合** (必須)
   - `PastaApi::create_module()` 実装
   - `select_label_to_id()` ブリッジ関数
   - Rune ↔ Rust 型変換

3. **Phase 3: 最適化** (オプショナル)
   - Trie導入によるパフォーマンス改善
   - 履歴管理のメモリ最適化

### パフォーマンス考慮事項

- **想定ラベル数**: 典型的なスクリプトで100～500ラベル（推定値）
- **許容レイテンシ**: ラベル解決は10ms以下（ユーザー体感に影響しない、推定値）
- **メモリ使用量**: LabelTableは数MB程度（問題なし、推定値）
- **Phase 1 実装**: HashMap + フルスキャン方式を採用（O(N)走査、想定ラベル数で十分）
- **Phase 3 最適化**: ラベル数が1000以上またはレイテンシ10ms超過時にTrie導入を検討

---

## Dependencies

| 依存仕様/クレート | 理由 | 状態 |
|------------------|------|------|
| `pasta-declarative-control-flow` (P0) | トランスパイラー、LabelRegistry | ✅ Completed |
| `rune` (0.14) | Rune VM、モジュール登録、`rune::from_value<T>()` | ✅ 既存依存 |
| `thiserror` | エラー型定義 | ✅ 既存依存 |
| `fast_radix_trie` (1.1.0) | Trie-based prefix index (O(M) search) | ✅ **Phase 1で決定** |

**型変換パターン（`rune::from_value<T>()`）:**
```rust
// crates/pasta/src/stdlib/persistence.rs より（既存実装）
use rune;

// Rune Value → Rust HashMap変換
let map: HashMap<String, rune::Value> = rune::from_value(data)
    .map_err(|e| format!("Failed to convert: {}", e))?;

// Rune Value → String変換
let s: String = rune::from_value(value)
    .map_err(|e| format!("Must be string: {}", e))?;

// select_label_to_id() での使用例:
fn parse_filters(value: rune::Value) -> Result<HashMap<String, String>, String> {
    let rune_map: HashMap<String, rune::Value> = rune::from_value(value)?;
    let mut filters = HashMap::new();
    for (key, val) in rune_map {
        filters.insert(key, rune::from_value::<String>(val)?);
    }
    Ok(filters)
}
```

---

## Future Work

- **パフォーマンス最適化:** Trie導入、履歴のLRUキャッシュ化
- **拡張フィルタ構文:** 正規表現（`＆name:/^さくら.*/`）、範囲指定（`＆score:>50`）
- **デバッグ支援:** ラベル解決のトレースログ、呼び出しグラフの可視化
- **エラーメッセージ改善:** 候補ラベルのサジェスト（Levenshtein距離による類似ラベル提示）

---

## References

- **親仕様:** `.kiro/specs/completed/pasta-declarative-control-flow/`
- **GRAMMAR.md:** `crates/pasta/GRAMMAR.md` (ラベル定義、属性構文)
- **現在の実装:** `crates/pasta/src/runtime/labels.rs`
- **トランスパイラー:** `crates/pasta/src/transpiler/mod.rs`
- **エラー型:** `crates/pasta/src/error.rs`

````
