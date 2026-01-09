# Gap Analysis Report: pasta_search_module

**初版作成**: 2026-01-09  
**改訂版（複数インスタンス制約統合）**: 2026-01-10  
**対象仕様**: pasta_search_module (Requirement 1-8, 80 Acceptance Criteria)  
**分析言語**: ja（日本語）  
**分析フレームワーク**: `.kiro/settings/rules/gap-analysis.md` に準拠

## Feature
**pasta_search_module** - Rust側検索モジュールのLuaバインディング実装

## Overview

### Analysis Scope
- Requirement検証：pasta_coreのSceneTable/WordTable APIを活用した4つの検索関数公開
- 既存コード：pasta_luaのコードジェネレータ、mlua-stdlibの実装パターン
- **新規制約**: 複数独立Luaランタイムインスタンス対応
- 統合点：RandomSelector状態管理、mlua-stdlib参照実装、インスタンス隔離

### Key Findings

**✅ 大部分の機能要件は既存コンポーネントで満たし可能**
- pasta_coreの SceneTable, WordTable, RandomSelector が完全に実装済み
- mlua-stdlib が豊富なバインディング実装パターンを提供
- pasta_lua が Cargo.toml で既に mlua-stdlib 依存

**⚠️ 新規：複数ランタイムインスタンス制約による設計分岐**
- ❌ Static 変数の排除が必須（複数インスタンス隔離要件）
- ✅ 3つの設計選択肢を documented (UserData/Arc<Mutex<>>/mlua Registry)
- **Decision Pending**: Design フェーズで参照管理パターン決定

**⚠️ 技術的課題**
1. RandomSelector の状態保持：Box<dyn RandomSelector> の Lua state 内保存 + 複数インスタンス隔離
2. SceneTable/WordTable への参照保持：Static禁止下での安全な参照管理
3. mlua-stdlib との実装パターン統一（登録関数、エラーハンドリング）

**🎯 推奨実装戦略**
- **Option A: UserData ラッピング** ← 推奨（複数インスタンス対応）
- mlua-stdlib の `loader()` + `register()` パターンを採用
- pasta_lua/src/search/ ディレクトリに検索モジュール実装
- SearchContext を UserData として各Luaインスタンスで独立管理

---

## Current State Investigation

### 1. pasta_core レジストリレイヤー

#### SceneTable API
```
Location: crates/pasta_core/src/registry/scene_table.rs (791行)

✅ 利用可能な機能:
- from_scene_registry(registry, random_selector) → Self
- resolve_scene_id(search_key, filters) → Result<SceneId>
- キャッシュベース選択：同一キーで循環的に異なる結果を返す
- RadixMap による前方一致検索
```

**要件への適合度**: **100%**
- Requirement 1.2 (前方一致検索) ✅
- Requirement 1.4 (ランダム選択) ✅
- Requirement 5.1-5.3 (循環動作) ✅

#### WordTable API
```
Location: crates/pasta_core/src/registry/word_table.rs (599行)

✅ 利用可能な機能:
- from_word_def_registry(registry, random_selector) → Self
- search_word(module_name, key, _filters) → Result<String>
- collect_word_candidates(module_name, key) → Result<Vec<String>>
- キャッシュベース選択：検索ごとにシャッフル済み単語を返す
- 統一キー形式：
  - ローカル: `:module_name:key`
  - グローバル: `key`
```

**要件への適合度**: **95%**
- Requirement 2.1-2.5 (グローバルシーン指定検索) ✅
- Requirement 3.1-3.5 (グローバル検索) ✅
- 注: Lua側で Level 1/2 検索後、Rust側に委譲される設計

#### RandomSelector トレイト
```
Location: crates/pasta_core/src/registry/random.rs (157行)

✅ 利用可能な実装:
- RandomSelector トレイト：Send + Sync
  - select_index(&mut self, len: usize) → Option<usize>
  - shuffle_usize(&mut self, items: &mut [usize])
- DefaultRandomSelector：本番用（StdRng使用）
- MockRandomSelector：テスト用（決定的選択）
```

**要件への適合度**: **100%**
- Requirement 5.2-5.4 (循環動作、シード初期化) ✅

### 2. mlua-stdlib 実装パターン

#### 複数ランタイムインスタンス制約（新規統合） ⚠️

**背景**:
Requirements 要件定義フェーズで新たに判明した制約：
```
pasta_lua は複数の独立した Lua ランタイムインスタンスをサポートする必要がある
- ❌ Static 変数による SceneTable/WordTable 保持は禁止
- ✅ 各ランタイムインスタンスは独立した SceneTable/WordTable を持つ必要
- ⚠️ スレッドローカル（TLS）でも複数インスタンス対応には不十分
```

**実装上の影響**:
```rust
// ❌ 許されない実装例
static SCENE_TABLE: Lazy<SceneTable> = Lazy::new(|| { ... });
static WORD_TABLE: Lazy<WordTable> = Lazy::new(|| { ... });

// ✅ 要求される実装方式
let lua1 = Lua::new();
let lua2 = Lua::new();
// lua1 と lua2 が異なる SceneTable/WordTable インスタンスを持つ必要
```

**選択肢の評価**:

| 選択肢 | 複数インスタンス対応 | Static 排除 | 実装複雑度 | 推奨度 |
|--------|------------------|----------|---------|--------|
| A: UserData ラッピング | ✅ | ✅ | **L (推奨)** | ⭐⭐⭐ |
| B: Arc<Mutex<>> + Globals | ✅ | ✅ | L | ⭐⭐ |
| C: mlua UserData Registry | ✅ | ✅ | XL | ⭐ |

---

#### モジュール登録パターン（複数インスタンス対応版）

mlua-stdlib の全モジュール（13+）は統一パターンを採用：

```rust
// Pattern A: 単純な関数群（env, assertions）
fn loader(lua: &Lua) -> Result<Table> {
    let t = lua.create_table()?;
    t.set("func1", lua.create_function(func1)?)?;
    t.set("func2", lua.create_function(func2)?)?;
    Ok(t)
}

pub fn register(lua: &Lua, name: Option<&str>) -> Result<Table> {
    let name = name.unwrap_or("@module_name");
    let value = loader(lua)?;
    lua.register_module(name, &value)?;
    Ok(value)
}
```

```rust
// Pattern B: UserData ラッパー + 関数群（http, task, regex, json）
impl UserData for LuaType {
    fn register(registry: &mut UserDataRegistry<Self>) {
        registry.add_function("new", |_, args| { ... })?;
        registry.add_method("method1", |_, this, args| { ... })?;
        // ...
    }
}

fn loader(lua: &Lua) -> Result<Table> {
    let t = lua.create_table()?;
    t.set("Type", lua.create_proxy::<LuaType>()?)?;
    t.set("func", lua.create_function(func)?)?;
    Ok(t)
}

pub fn register(lua: &Lua, name: Option<&str>) -> Result<Table> {
    let name = name.unwrap_or("@module_name");
    let value = loader(lua)?;
    lua.register_module(name, &value)?;
    Ok(value)
}
```

**エラーハンドリングパターン**:
- `lua_try!` マクロ：Result<T> → Ok(T) | Ok(Err(String))
- `opt_param!` マクロ：オプション引数抽出と型変換
- `param!` マクロ：必須引数抽出と型変換

#### 実装の質的特徴

| 特性 | mlua-stdlib | pasta_search に期待される |
|------|-------------|------------------------|
| 引数検証 | 堅牢（複数段階） | 同様の品質 |
| エラー処理 | ErrorContext で詳細情報 | 同様 |
| 非同期対応 | あり（async メソッド） | 不要（同期検索） |
| ユーザーデータ登録 | lua.create_proxy<T>() | 推奨：RandomSelector 保持用 |
| キャッシュ管理 | SceneTable/WordTable 内で実施 | Rust側で完全管理 |

### 3. pasta_lua の現在の構造

#### モジュール構成
```
pasta_lua/src/
├── lib.rs              # 公開API
├── transpiler.rs       # LuaTranspiler（コードジェネレータ呼び出し）
├── code_generator.rs   # Lua AST生成
├── context.rs          # TranspileContext（レジストリ保持）
├── config.rs
├── error.rs
├── string_literalizer.rs
└── normalize.rs
```

**現在のレジストリ処理**:
```rust
// context.rs
pub struct TranspileContext {
    word_registry: WordDefRegistry,
    // ... scene_registry 等
}

// transpiler.rs
context.word_registry.register_global(&word.name, values);
context.register_global_scene(scene);
```

**スタブ実装状況**:
- Rust側：code_generator.rs で Pass 1/2 実装完了
- Lua側：design.md で API仕様定義済み、実装待ち
- バインディング：未実装（本仕様のスコープ）

#### 統合点

**Requirement 4.2の登録関数署名**:
```rust
pub fn register_search_functions(
    lua: &Lua, 
    scene_table: &SceneTable, 
    word_table: &WordTable
) -> Result<()>
```

Rust側 code_generator → Lua側スクリプトフロー：
1. Pass 2 終了時に SceneRegistry → SceneTable 変換
2. WordDefRegistry → WordTable 変換
3. register_search_functions(lua, scene_table, word_table) 呼び出し
4. Lua globals に `pasta_search_scene`, `pasta_search_word_local`, `pasta_search_word_global` 登録

---

## Requirements Feasibility Analysis

### 技術要件マッピング

| Requirement | 必要な技術 | 既存コンポーネント | Gap | 難易度 |
|--|--|--|--|--|
| 1: シーン検索API | SceneTable 前方一致 | ✅ SceneTable | ❌ なし | 低 |
| 2: 単語検索API (ローカル) | WordTable `:module:key` 検索 | ✅ WordTable | ❌ なし | 低 |
| 3: 単語検索API (グローバル) | WordTable `key` 検索 | ✅ WordTable | ❌ なし | 低 |
| 4: mlua バインディング | Lua関数登録、引数検証 | ⚠️ mlua-stdlib パターン | ⚠️ 実装なし | 中 |
| 5: ランダム選択循環 | RandomSelector 状態保持 | ✅ RandomSelector trait | ⚠️ Lua内保存方法 | 中 |
| 6: エラーハンドリング | Result<T>型、mlua::Error | ✅ mlua + mlua-stdlib パターン | ❌ なし | 低 |
| 7: パフォーマンス | 参照保持、キャッシング | ✅ SceneTable/WordTable キャッシュ済み | ❌ なし | 低 |

### 複雑性分析

#### シンプル（CRUD/アルゴリズム）
- Requirement 1, 2, 3：検索ロジックは pasta_core で完全実装
- 実装作業：Lua関数 → SceneTable/WordTable メソッド呼び出しのみ

#### 中程度（ステートフル）
- Requirement 5：RandomSelector の状態管理
  - 問題：Box<dyn RandomSelector> は Lua UserData として登録不可
  - 解決案：UserData ラッパー型でカプセル化

#### 検索が必要な領域
- **RandomSelector を Lua に安全に公開する方法**
  - 現在：Box<dyn RandomSelector> は trait object
  - Lua側：状態保持が必要（`&mut self`）
  - 実装パターン：mlua-stdlib のサンプルなし（trait object 不在）

---

## Implementation Approach Options (複数インスタンス対応版)

### Option A: UserData ラッピング による状態隔離（推奨） ⭐⭐⭐

**戦略**: 各Luaインスタンスが独立した `SearchContext` UserData を保有

```rust
// Rust側：各インスタンスが独立した状態を管理
pub struct SearchContext {
    scene_table: SceneTable,
    word_table: WordTable,
}

impl mlua::UserData for SearchContext {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("search_scene", |lua, this: &SearchContext, (name, global_scene): (String, Option<String>)| {
            // this.scene_table で検索実行
            Ok((global_name, local_name))
        });
        methods.add_method_mut("set_scene_selector", |lua, this: &mut SearchContext, sequence: Vec<u64>| {
            // this.scene_table.random_selector を切り替え
            Ok(())
        });
    }
}

pub fn loader(lua: &Lua) -> Result<Table> {
    // Luaインスタンスのレジストリから SceneTable/WordTable を取得
    let scene_registry: SceneRegistry = /* ... */;
    let word_registry: WordDefRegistry = /* ... */;
    
    // 各インスタンス用のコンテキスト生成
    let context = SearchContext {
        scene_table: SceneTable::from_scene_registry(scene_registry, Box::new(DefaultRandomSelector::new()))?,
        word_table: WordTable::from_word_def_registry(word_registry, Box::new(DefaultRandomSelector::new()))?,
    };
    
    let table = lua.create_table()?;
    table.set("_context", lua.create_userdata(context)?)?;
    table.set("search_scene", lua.create_function(search_scene_wrapper)?)?;
    Ok(table)
}

pub fn register(lua: &Lua) -> Result<Table> {
    let table = loader(lua)?;
    lua.globals().set("@pasta_search", table.clone())?;
    Ok(table)
}
```

**Lua側の利用**:
```lua
-- Lua instance 1
local SEARCH1 = require "@pasta_search"
local global_name, local_name = SEARCH1.search_scene("シーン名", "グローバル")

-- Lua instance 2 (別インスタンス)
local SEARCH2 = require "@pasta_search"
local word = SEARCH2.search_word("単語", "グローバル")

-- SEARCH1 と SEARCH2 は異なるコンテキストを持つため、
-- RandomSelector の状態が独立している
```

**✅ メリット**:
- 各Luaインスタンスが独立した SceneContext を持つため **複数インスタンス制約を満たす**
- Static 変数を使わない
- Selector 切り替え（`&mut self`）を安全に実装可能
- mlua-stdlib パターン完全準拠
- インスタンス間の state 汚染なし

**❌ デメリット**:
- UserData メカニズムの学習コスト（mlua ドキュメント必須）
- メタテーブル設定で `table.func()` vs `table:func()` を制御する必要あり
- メモリ：各インスタンスが SceneTable/WordTable を複製（共有不可）

**実装複雑度**: **L** (1-2 週間)
- UserData trait implementation
- 関数シグネチャ + エラーハンドリング
- テスト（複数インスタンス並行実行）

**Risk**: **Medium**
- mlua API の学習曲線

---

## Recommended Approach: Option A - 設計決定済み

### 決定内容

**選択アプローチ**: UserData ラッピングによる状態隔離（Option A）

**根拠**:
- 複数Luaランタイムインスタンス対応の要件を完全に満たす
- Static 変数を排除できる
- mlua-stdlib パターンとの完全互換
- マルチスレッド安全性が高い

### Phase 1: UserData 実装詳細（Design フェーズ）

**詳細設計内容**:
1. SearchContext struct の定義（SceneTable, WordTable フィールド）
2. UserData impl の方法論
3. メタテーブル設定（`__index` で `func()` 呼び出しを可能にする）
4. Selector 切り替え時の `&mut self` 制御方法

---

### Phase 2: 実装ファイル構成

```
pasta_lua/src/
├── lib.rs (修正)
│   └── pub mod search
├── search/
│   ├── mod.rs
│   │   ├── pub fn loader(lua: &Lua) -> Result<Table>
│   │   └── pub fn register(lua: &Lua) -> Result<Table>
│   ├── context.rs
│   │   └── pub struct SearchContext
│   ├── scene_search.rs
│   │   └── fn search_scene_impl(...)
│   └── word_search.rs
│       └── fn search_word_impl(...)
```

---

### Phase 3: 複数インスタンステスト戦略

**テストケース**:
```rust
#[test]
fn test_multiple_independent_instances() {
    // Lua インスタンス 1
    let lua1 = Lua::new();
    register(&lua1)?;
    
    // Lua インスタンス 2
    let lua2 = Lua::new();
    register(&lua2)?;
    
    // 各インスタンスの SEARCH は異なるコンテキストを持つことを検証
    let result1 = lua1.load("return require('@pasta_search').search_scene(...)").eval()?;
    let result2 = lua2.load("return require('@pasta_search').search_scene(...)").eval()?;
    
    // 同一キーでも異なる RandomSelector 状態 → 異なる結果
    assert_ne!(result1, result2);  // MockSelector でシーケンス 0, 1 など
}
```

---

## Recommended Implementation Path

### Phase 1: Module Setup

**ファイル構成**:
```
pasta_lua/src/
├── lib.rs (修正)
│   └── pub mod search
├── search/
│   ├── mod.rs
│   │   ├── pub struct SearchContext
│   │   ├── pub fn loader(lua: &Lua) -> Result<Table>
│   │   └── pub fn register(lua: &Lua) -> Result<Table>
│   ├── scene_search.rs (シーン検索実装)
│   └── word_search.rs (単語検索実装)
└── (既存ファイル)
```

**実装ステップ**:
1. `pasta_lua/src/search/mod.rs` 作成：SearchContext struct + loader/register
2. `pasta_lua/src/search/scene_search.rs` 作成：search_scene() 実装
3. `pasta_lua/src/search/word_search.rs` 作成：search_word() 実装
4. `pasta_lua/src/lib.rs` 更新：`pub mod search`

### Phase 2: UserData ラッパー型実装

```rust
// pasta_lua/src/search/mod.rs
use mlua::{Lua, Result, Table, UserData, UserDataMethods, UserDataRegistry};
use pasta_core::registry::{SceneTable, WordTable};

pub struct SearchContext {
    scene_table: SceneTable,
    word_table: WordTable,
}

impl UserData for SearchContext {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // search_scene, search_word, set_scene_selector, set_word_selector を登録
        methods.add_function("search_scene", search_scene_impl)?;
        methods.add_function("search_word", search_word_impl)?;
        methods.add_method_mut("set_scene_selector", set_scene_selector_impl)?;
        methods.add_method_mut("set_word_selector", set_word_selector_impl)?;
    }
}

pub fn loader(lua: &Lua) -> Result<Table> {
    // 各Luaインスタンス用の独立した SearchContext 生成
    let context = SearchContext {
        scene_table: /* from transpile registry */,
        word_table: /* from transpile registry */,
    };
    
    let table = lua.create_table()?;
    let userdata = lua.create_userdata(context)?;
    
    // Lua側で SEARCH.search_scene(...) のように呼び出すため
    // メタテーブルで __index を設定
    let methods = lua.create_table()?;
    methods.set("search_scene", lua.create_function(
        |lua, (_: AnyUserData, name: String)| {
            // UserData から SearchContext を取得して検索実行
        }
    )?)?;
    
    table.set("_context", userdata)?;
    Ok(table)
}

pub fn register(lua: &Lua) -> Result<Table> {
    let table = loader(lua)?;
    lua.globals().set("@pasta_search", table.clone())?;
    Ok(table)
}
```

### Phase 3: 検索関数実装

```rust
// pasta_lua/src/search/scene_search.rs
fn search_scene_impl(
    lua: &Lua,
    this: &SearchContext,
    (name, global_scene_name): (String, Option<String>),
) -> Result<Option<(String, String)>> {
    // Requirement 2 の段階的フォールバック実装
    // ...
}

// pasta_lua/src/search/word_search.rs
fn search_word_impl(
    lua: &Lua,
    this: &SearchContext,
    (name, global_scene_name): (String, Option<String>),
) -> Result<Option<String>> {
    // Requirement 3 の段階的フォールバック実装
    // ...
}
```

### Phase 4: Selector 制御 API（Requirement 8）

```rust
fn set_scene_selector_impl(
    lua: &Lua,
    this: &mut SearchContext,
    sequence: mlua::MultiValue,
) -> Result<()> {
    // Requirement 8: MockRandomSelector に切り替え
    if sequence.is_empty() {
        // デフォルトに戻す
        this.scene_table = SceneTable::new(Box::new(DefaultRandomSelector::new()));
    } else {
        // MockRandomSelector にセット
        let indices: Vec<usize> = sequence.iter()
            .map(|v| v.as_integer().ok_or(...))
            .collect::<Result<_>>()?;
        this.scene_table = SceneTable::new(Box::new(MockRandomSelector::new(indices)));
    }
    Ok(())
}
```

### Phase 5: テスト戦略

**単一インスタンステスト**:
```rust
#[test]
fn test_search_scene() {
    let lua = Lua::new();
    search::register(&lua)?;
    
    let result: (String, String) = lua.load(
        "local SEARCH = require('@pasta_search'); return SEARCH:search_scene('シーン', 'グローバル')"
    ).eval()?;
    
    assert_eq!(result.0, "expected_global");
}
```

**複数インスタンステスト**:
```rust
#[test]
fn test_multiple_instances_independent() {
    let lua1 = Lua::new();
    let lua2 = Lua::new();
    
    search::register(&lua1)?;
    search::register(&lua2)?;
    
    // lua1 と lua2 は異なる SearchContext を持つため
    // RandomSelector の状態が独立している
    
    // MockSelector で検証
    lua1.load("...set_scene_selector(0, 1, 2)...").eval()?;
    lua2.load("...set_scene_selector(3, 2, 1)...").eval()?;
    
    // 異なる結果が返される
}
```

---

## Design Phase Decision Points

### 1. SearchContext の初期化フロー

**Question**: transpile フロー中に SceneRegistry/WordDefRegistry → SearchContext をどのタイミングで生成するか？

**Options**:
- A) Transpiler.transpile() の返り値に SearchContext を含める
- B) 別途 init_search_context(scene_registry, word_registry) を呼び出す
- C) Lua globals への登録時に遅延初期化

**Recommendation**: Option B（明示的、責任分離）

### 2. メタテーブル設定による Lua 側インターフェース

**Question**: `SEARCH.search_scene()` vs `SEARCH:search_scene()` どちらを実装するか？

**Options**:
- A) `SEARCH.search_scene()`: グローバル関数（UserData と別）
- B) `SEARCH:search_scene()`: UserData メソッド
- C) 両方対応

**Recommendation**: Option B（mlua-stdlib パターン）で統一

### 3. RandomSelector の trait object 可変性

**Question**: Selector 切り替え（Requirement 8）で `&mut self` を安全に保証するか？

**Options**:
- A) UserData の `add_method_mut` で `&mut self` を提供
- B) Interior Mutability（RefCell）を SearchContext 内に使用
- C) setter で全体を置き換え

**Recommendation**: Option A（mlua サポート、最もシンプル）

---

## Research Needed

### 1. mlua UserData メタテーブル設定

**確認対象**:
- UserData にメタテーブルを設定して __index を制御する方法
- `methods.add_function()` vs `methods.add_method()` の違い

**影響度**: High（Lua側インターフェースに直結）

### 2. SceneInfo 復元メカニズム

**確認対象**:
```rust
// pasta_core から SceneId → SceneInfo を復元できるか？
pub fn get_scene_info(&self, id: SceneId) -> Option<&SceneInfo> { ... }
```

**影響度**: Medium（Requirement 2.3 の (global_name, local_name) 返却に必須）

### 3. pasta_lua コード生成フロー との統合

**確認対象**:
- Transpiler.transpile() から SceneRegistry/WordDefRegistry を取得可能か？
- TranspileContext に search 関連の初期化フックを追加するか？

**影響度**: High（実装フローに直結）

---

## Conclusion & Risk Assessment

### 実装可能性

| Requirement | ギャップ | 実装可能性 | Design 決定 |
|-----------|---------|----------|----------|
| 1-3: 基本検索 | Low | ✅ 十分 | なし |
| 4: mlua バインディング | Medium | ✅ 十分（Option A 選択） | **参照管理パターン** |
| 5: ランダム循環 | Low | ✅ 自動 | なし |
| 6: エラーハンドリング | Medium | ✅ 十分 | なし |
| 7: パフォーマンス | Low | ✅ 設計時に確認 | なし |
| 8: Selector 制御 | Medium | ✅ 十分（Option A で `&mut self` 可能） | **実装順序** |

### 推奨スケジュール

| Phase | 期間 | タスク | Risk |
|-------|------|--------|------|
| Design | 1-2 日 | UserData パターン決定、Lua フロー設計 | Low |
| Phase 1 | 3-4 日 | 基本検索実装（Req 1-3） | Low |
| Phase 2 | 2-3 日 | Selector 制御（Req 8） | Low |
| Phase 3 | 1-2 日 | テスト + 複数インスタンス検証 | Medium |
| **Total** | **7-11 日** | **本実装完了** | **Low-Medium** |

### 重要な設計制約

**複数ランタイムインスタンス対応**:
- ✅ Option A (UserData) で完全対応
- ✅ Static 変数排除の要件を満たす
- ✅ 各インスタンスが独立した RandomSelector 状態を保持

**mlua-stdlib パターン準拠**:
- ✅ loader() + register() パターン採用
- ✅ UserData + 関数ハイブリッド実装
- ✅ エラーハンドリングで mlua::Error 利用

---

## Next: Design Phase Action Items

### 優先度 1（Block）
1. [ ] SearchContext 構造体の詳細設計
2. [ ] UserData add_methods() の実装方針確定
3. [ ] Transpiler との統合ポイント明確化

### 優先度 2（Should）
1. [ ] mlua-stdlib ドキュメント確認（メタテーブル設定）
2. [ ] 複数インスタンステストの詳細シナリオ
3. [ ] エラーメッセージの国際化対応

### 優先度 3（Nice to Have）
1. [ ] ベンチマーク測定設計
2. [ ] Lua リファレンス ドキュメント案作成
3. [ ] ci/ スクリプト更新
