# Design Document: pasta_search_module

## Overview

**Purpose**: Rust 側のシーン辞書・単語辞書検索機能を mlua バインディングで Lua 側に公開し、`act:word()`, `PROXY:word()`, `act:call()` から呼び出せるようにする。

**Users**: Lua スクリプト開発者（pasta.act, pasta.actor モジュール利用者）

**Impact**: pasta_lua crate に新規 `search` モジュールを追加。pasta_core の SceneTable/WordTable を Lua 環境から利用可能にする。

### Goals

- `@pasta_search` モジュールとして Lua に検索 API を公開
- シーン検索・単語検索の段階的フォールバック戦略を実装
- 複数 Lua ランタイムインスタンスで独立した検索コンテキストを維持
- テスト時の決定的選択を Lua 側から制御可能にする

### Non-Goals

- Lua 側モジュール（pasta.act, pasta.actor）の実装（別仕様）
- Level 1/2 検索（アクター field, SCENE field）- Lua 側で処理
- SceneRegistry/WordDefRegistry の構築ロジック（pasta_core で既存）

---

## Architecture

### Existing Architecture Analysis

**現行システム**:
- **pasta_core**: SceneTable, WordTable, RandomSelector が完全実装済み
- **pasta_lua**: code_generator.rs で Lua コード生成、TranspileContext でレジストリ管理
- **mlua-stdlib**: loader/register パターンの参照実装

**尊重すべきパターン**:
- mlua-stdlib の loader() + register() パターン
- pasta_core の Registry → Table 変換フロー
- Rust 2024 edition のエラーハンドリング慣習

### Architecture Pattern & Boundary Map

```mermaid
graph TB
    subgraph Application
        App[Application Layer]
    end
    
    subgraph pasta_lua
        Runtime[PastaLuaRuntime]
        Transpiler[LuaTranspiler]
        TransCtx[TranspileContext]
        SearchModule[@pasta_search Module]
        SearchContext[SearchContext UserData]
        Loader[loader / register]
    end
    
    subgraph Lua側
        LuaVM[Lua VM]
        LuaScript[Lua Script]
        PastaAct[pasta.act]
        PastaActor[pasta.actor]
    end
    
    subgraph pasta_core
        SceneTable[SceneTable]
        WordTable[WordTable]
        RandomSelector[RandomSelector]
    end
    
    App --> Transpiler
    Transpiler --> TransCtx
    App --> Runtime
    TransCtx --> Runtime
    Runtime --> LuaVM
    Runtime --> Loader
    Loader --> SearchContext
    
    LuaVM --> LuaScript
    LuaScript --> PastaAct
    LuaScript --> PastaActor
    PastaAct --> SearchModule
    PastaActor --> SearchModule
    SearchModule --> SearchContext
    SearchContext --> SceneTable
    SearchContext --> WordTable
    SceneTable --> RandomSelector
    WordTable --> RandomSelector
```

**Architecture Integration**:
- **Selected pattern**: PastaLuaRuntime による Lua VM ホスティング + UserData ラッピング
- **Domain boundaries**: pasta_lua が mlua 依存を持ち、pasta_core は言語非依存を維持
- **Existing patterns preserved**: loader/register パターン、Result<T> エラーハンドリング
- **New components**: PastaLuaRuntime、SearchContext UserData、search モジュール
- **Steering compliance**: 複数インスタンス対応、Static 変数禁止

### Technology Stack

| Layer | Choice / Version | Role in Feature | Notes |
|-------|------------------|-----------------|-------|
| Binding | mlua 0.11 | Lua ↔ Rust バインディング | 既存依存 |
| Runtime | pasta_core | SceneTable, WordTable, RandomSelector | 検索ロジック提供 |
| Language | Rust 2024 | 実装言語 | 型安全性 |
| Testing | mlua + cargo test | 統合テスト | 複数インスタンス検証 |

---

## System Flows

### シーン検索フロー（Requirement 2）

```mermaid
sequenceDiagram
    participant Lua as Lua Script
    participant SEARCH as @pasta_search
    participant Ctx as SearchContext
    participant ST as SceneTable
    
    Lua->>SEARCH: SEARCH:search_scene("シーン", "親シーン")
    SEARCH->>Ctx: search_scene(name, global_scene_name)
    
    alt global_scene_name が指定
        Ctx->>ST: resolve_scene_id_unified(parent, name, filters)
        ST-->>Ctx: SceneId or Error
        
        alt ローカル検索成功
            Ctx-->>SEARCH: (global_name, local_name)
        else ローカル検索失敗 → グローバル検索
            Ctx->>ST: resolve_scene_id(name, filters)
            ST-->>Ctx: SceneId or Error
            Ctx-->>SEARCH: (global_name, "__start__") or nil
        end
    else global_scene_name が nil
        Ctx->>ST: resolve_scene_id(name, filters)
        ST-->>Ctx: SceneId or Error
        Ctx-->>SEARCH: (global_name, "__start__") or nil
    end
    
    SEARCH-->>Lua: (global_name, local_name) or nil
```

### Selector 切り替えフロー（Requirement 8）

```mermaid
sequenceDiagram
    participant Lua as Lua Script
    participant SEARCH as @pasta_search
    participant Ctx as SearchContext
    participant ST as SceneTable
    
    Lua->>SEARCH: SEARCH:set_scene_selector(0, 1, 2)
    SEARCH->>Ctx: set_scene_selector(&mut self, [0, 1, 2])
    Ctx->>Ctx: MockRandomSelector::new([0, 1, 2])
    Ctx->>ST: replace_random_selector(mock)
    Ctx-->>SEARCH: Ok(())
    SEARCH-->>Lua: (success)
```

### ランタイム初期化フロー（Requirement 9）

```mermaid
sequenceDiagram
    participant App as Application
    participant Trans as LuaTranspiler
    participant RT as PastaLuaRuntime
    participant Lua as Lua VM
    participant Loader as search::register
    participant Ctx as SearchContext
    
    App->>Trans: transpile(&pasta_file, &mut output)
    Trans-->>App: TranspileContext
    
    App->>RT: PastaLuaRuntime::new(context)
    RT->>Lua: Lua::new()
    Lua-->>RT: lua
    
    RT->>RT: context.scene_registry, context.word_registry
    RT->>Loader: register(&lua, scene_registry, word_registry)
    Loader->>Ctx: SearchContext::new(scene_registry, word_registry)
    Ctx-->>Loader: search_context
    Loader->>Lua: package.loaded["@pasta_search"] = module
    Loader-->>RT: Ok(table)
    
    RT-->>App: Ok(PastaLuaRuntime)
    
    Note over App,Lua: ランタイム使用準備完了
    
    App->>RT: exec(lua_code)
    RT->>Lua: load(script).eval()
    Lua-->>RT: Value
    RT-->>App: Ok(Value)
```

**フロー説明**:
1. **トランスパイル**: `LuaTranspiler::transpile()` で Pasta AST を Lua コードに変換し、`TranspileContext` を取得
2. **ランタイム生成**: `PastaLuaRuntime::new(context)` で Lua VM を初期化
3. **モジュール登録**: 初期化時に `search::register()` を呼び出し、`@pasta_search` を登録（一度のみ）
4. **スクリプト実行**: `exec()` で Lua スクリプトを実行

---

## Requirements Traceability

| Requirement | Summary | Components | Interfaces | Flows |
|-------------|---------|------------|------------|-------|
| 1.1-1.5 | @pasta_search モジュール公開 | SearchModule, Loader | register(), loader() | - |
| 2.1-2.8 | シーン検索 API | SearchContext | search_scene() | シーン検索フロー |
| 3.1-3.8 | 単語検索 API | SearchContext | search_word() | 同様 |
| 4.1-4.8 | mlua バインディング | SearchContext, Loader | UserData impl | - |
| 5.1-5.4 | ランダム選択循環 | SceneTable, WordTable | (pasta_core 既存) | - |
| 6.1-6.3 | エラーハンドリング | SearchContext | Result<T, mlua::Error> | - |
| 7.1-7.3 | パフォーマンス | SearchContext | (設計考慮) | - |
| 8.1-8.10 | Selector 制御 API | SearchContext | set_scene_selector(), set_word_selector() | Selector 切り替えフロー |
| 9.1-9.10 | ランタイム層 | PastaLuaRuntime | new(), exec(), register_module() | ランタイム初期化フロー |

---

## Components and Interfaces

### Component Summary

| Component | Domain/Layer | Intent | Req Coverage | Key Dependencies | Contracts |
|-----------|--------------|--------|--------------|------------------|-----------|
| PastaLuaRuntime | pasta_lua/runtime | Lua VM ホスト構造体 | 9 | TranspileContext (P0), Loader (P0) | Service, State |
| SearchContext | pasta_lua/search | 検索状態管理 UserData | 1-8 | SceneTable (P0), WordTable (P0) | Service, State |
| SearchModule | pasta_lua/search | @pasta_search モジュール登録 | 1, 4 | Loader (P0), mlua (P0) | API |
| Loader | pasta_lua/search | モジュール初期化 | 4 | SearchContext (P0) | Service |

---

### pasta_lua/runtime Layer

#### PastaLuaRuntime

| Field | Detail |
|-------|--------|
| Intent | Lua VM をホストし、pasta モジュール群を統合する最上位構造体 |
| Requirements | 9.1-9.10 |

**Responsibilities & Constraints**
- mlua の Lua インスタンスを所有・管理
- TranspileContext から SearchContext を生成
- @pasta_search モジュールを初期化時に登録（一度のみ）
- 複数インスタンス生成をサポート（Static 変数禁止）
- 将来の拡張用にモジュール登録メカニズムを提供

**Dependencies**
- Inbound: アプリケーション層 — ランタイム生成・スクリプト実行 (P0)
- Outbound: SearchModule/Loader — @pasta_search 登録 (P0)
- Outbound: TranspileContext — SceneRegistry/WordDefRegistry 取得 (P0)
- External: mlua — Lua VM (P0)

**Contracts**: Service [x] / API [x] / Event [ ] / Batch [ ] / State [x]

##### Service Interface

```rust
use mlua::{Lua, Result as LuaResult, Table, Value};
use crate::context::TranspileContext;

/// Pasta Lua ランタイム - Lua VM と pasta モジュール群を統合
/// 
/// 各インスタンスは独立した Lua VM と検索コンテキストを持つ。
/// 複数インスタンス生成をサポートし、Static 変数を使用しない。
pub struct PastaLuaRuntime {
    lua: Lua,
    // 将来の拡張用（登録済みモジュール追跡など）
}

impl PastaLuaRuntime {
    /// TranspileContext から新しいランタイムを生成
    /// 
    /// # Arguments
    /// * `context` - LuaTranspiler::transpile() の出力
    /// 
    /// # Returns
    /// * `Ok(Self)` - 初期化成功（@pasta_search 登録済み）
    /// * `Err(e)` - Lua VM 初期化または モジュール登録失敗
    /// 
    /// # Example
    /// ```rust,ignore
    /// let transpiler = LuaTranspiler::default();
    /// let context = transpiler.transpile(&pasta_file, &mut output)?;
    /// let runtime = PastaLuaRuntime::new(context)?;
    /// ```
    pub fn new(context: TranspileContext) -> LuaResult<Self> {
        let lua = Lua::new();
        
        // TranspileContext から Registry を取得
        let scene_registry = context.scene_registry;
        let word_registry = context.word_registry;
        
        // @pasta_search モジュールを登録（一度のみ）
        crate::search::register(&lua, scene_registry, word_registry)?;
        
        Ok(Self { lua })
    }
    
    /// Lua スクリプトを実行
    /// 
    /// # Arguments
    /// * `script` - 実行する Lua コード
    /// 
    /// # Returns
    /// * `Ok(Value)` - 実行結果
    /// * `Err(e)` - 実行エラー
    pub fn exec(&self, script: &str) -> LuaResult<Value> {
        self.lua.load(script).eval()
    }
    
    /// Lua スクリプトファイルを実行
    pub fn exec_file(&self, path: &std::path::Path) -> LuaResult<Value> {
        let script = std::fs::read_to_string(path)
            .map_err(|e| mlua::Error::ExternalError(std::sync::Arc::new(e)))?;
        self.exec(&script)
    }
    
    /// 内部 Lua インスタンスへの参照を取得（高度な操作用）
    pub fn lua(&self) -> &Lua {
        &self.lua
    }
    
    /// カスタムモジュールを登録（将来の拡張用）
    /// 
    /// # Arguments
    /// * `name` - モジュール名（@prefix 含む）
    /// * `module` - モジュールテーブル
    pub fn register_module(&self, name: &str, module: Table) -> LuaResult<()> {
        let package: Table = self.lua.globals().get("package")?;
        let loaded: Table = package.get("loaded")?;
        loaded.set(name, module)?;
        Ok(())
    }
}
```

- Preconditions: TranspileContext が有効な SceneRegistry/WordDefRegistry を持つこと
- Postconditions: Lua VM が初期化され、@pasta_search が登録されている
- Invariants: 各インスタンスは独立した状態を持つ

##### State Management

- **State model**: PastaLuaRuntime は Lua インスタンスを所有し、SearchContext は Lua の UserData として管理される
- **Persistence**: なし（インメモリ）
- **Concurrency**: Lua シングルスレッド前提、各ランタイムインスタンスは独立

**Implementation Notes**
- Integration: `LuaTranspiler::transpile()` → `TranspileContext` → `PastaLuaRuntime::new()`
- Validation: TranspileContext の有効性チェック
- Risks: なし（シンプルな所有権モデル）
- **将来の拡張性**:
  - 他の pasta モジュール（pasta.act, pasta.ctx 等）の Rust バインディング追加
  - 複数のトランスパイル済みファイルを1つのランタイムで実行

---

### pasta_lua/search Layer

#### SearchContext

| Field | Detail |
|-------|--------|
| Intent | Lua ランタイムごとの検索状態を管理する UserData |
| Requirements | 1.3, 2.1-2.8, 3.1-3.8, 4.8, 5.1-5.4, 7.1-7.3, 8.1-8.10 |

**Responsibilities & Constraints**
- シーン検索・単語検索の実行
- RandomSelector 状態の保持・切り替え
- 各 Lua インスタンスで独立したインスタンスを維持
- Static 変数禁止

**Dependencies**
- Inbound: PastaLuaRuntime — ランタイム初期化時に生成 (P0)
- Inbound: SearchModule — モジュール登録時に生成 (P0)
- Outbound: SceneTable — シーン検索実行 (P0)
- Outbound: WordTable — 単語検索実行 (P0)
- External: pasta_core::registry — SceneTable, WordTable, RandomSelector (P0)

**Contracts**: Service [x] / API [ ] / Event [ ] / Batch [ ] / State [x]

##### Service Interface

```rust
/// 検索コンテキスト - 各 Lua インスタンスで独立したインスタンスを持つ
pub struct SearchContext {
    scene_table: SceneTable,
    word_table: WordTable,
}

impl SearchContext {
    /// SceneRegistry と WordDefRegistry から SearchContext を生成
    pub fn new(
        scene_registry: SceneRegistry,
        word_registry: WordDefRegistry,
    ) -> Result<Self, SearchError>;
    
    /// シーン検索（段階的フォールバック：ローカル → グローバル）
    /// 
    /// # Arguments
    /// * `name` - 検索プレフィックス
    /// * `global_scene_name` - 親シーン名（None でグローバルのみ検索）
    /// 
    /// # Returns
    /// * `Ok((global_name, local_name))` - 検索成功
    ///   - ローカルシーンで検索成功: (全体シーン名, ローカルシーン名)
    ///   - グローバルシーンで検索成功: (全体シーン名, "__start__")
    /// * `Err(SearchError::NotFound)` - 全検索で候補なし
    /// * `Err(e)` - その他エラー
    pub fn search_scene(
        &mut self,
        name: &str,
        global_scene_name: Option<&str>,
    ) -> Result<(String, String), SearchError>;
    
    /// 単語検索（段階的フォールバック：ローカル → グローバル）
    /// 
    /// # Arguments
    /// * `name` - 検索キー
    /// * `global_scene_name` - 親シーン名（None でグローバルのみ検索）
    /// 
    /// # Returns
    /// * `Ok(word_string)` - 検索成功（文字列を返す）
    /// * `Err(SearchError::NotFound)` - 全検索で候補なし
    /// * `Err(e)` - その他エラー（引数型不正など）
    pub fn search_word(
        &mut self,
        name: &str,
        global_scene_name: Option<&str>,
    ) -> Result<String, SearchError>;
    
    /// シーン用 RandomSelector をリセットまたは切り替え
    /// 
    /// # Arguments
    /// * `sequence` - None でデフォルト、Some で MockRandomSelector
    pub fn set_scene_selector(
        &mut self,
        sequence: Option<Vec<usize>>,
    ) -> Result<(), SearchError>;
    
    /// 単語用 RandomSelector をリセットまたは切り替え
    pub fn set_word_selector(
        &mut self,
        sequence: Option<Vec<usize>>,
    ) -> Result<(), SearchError>;
}
```

- Preconditions: SceneRegistry/WordDefRegistry が有効な状態であること
- Postconditions: 検索結果が返される、または nil
- Invariants: RandomSelector 状態は SearchContext 内で完結

##### State Management

- **State model**: SearchContext は SceneTable/WordTable を所有し、各テーブルが内部キャッシュと RandomSelector を持つ
- **Persistence**: なし（インメモリ）
- **Concurrency**: Lua シングルスレッド前提、mlua が exclusive access を保証

**Implementation Notes**
- Integration: TranspileContext から SceneRegistry/WordDefRegistry を受け取り SearchContext を生成
- **Initialization**: pasta_lua ランタイム構造体が初期化時に `loader()` を呼び出す（一度のみ）
- Validation: 引数型チェックは mlua が自動実行
- **段階的フォールバック戦略（確定）**:
  - pasta_core の既存マージ戦略を**フォールバック戦略に変更**する
  - ローカル検索 → 結果あり → ローカルから選択（終了）
  - ローカル検索 → 結果なし → グローバル検索 → グローバルから選択
  - 既存のマージ戦略コード・テストは削除対象
- **エラー処理**: 候補なし → `SceneTableError::SceneNotFound` → SearchContext が mlua::Error に変換
- **MockRandomSelector**: pasta_core で常時公開（`#[cfg(test)]` 削除）

---

#### SearchModule (Loader/Register)

| Field | Detail |
|-------|--------|
| Intent | @pasta_search モジュールを Lua に登録 |
| Requirements | 1.1-1.5, 4.1-4.5 |

**Responsibilities & Constraints**
- loader() で SearchContext UserData を含むテーブル生成
- register() で `@pasta_search` として Lua globals に登録
- 複数回 require で同じインスタンスを返す

**Dependencies**
- Inbound: pasta_lua 初期化フロー — モジュール登録呼び出し (P0)
- Outbound: SearchContext — UserData 生成 (P0)
- External: mlua — モジュール登録 API (P0)

**Contracts**: Service [x] / API [x] / Event [ ] / Batch [ ] / State [ ]

##### Service Interface

```rust
/// @pasta_search モジュールテーブルを生成
/// 
/// SearchContext を UserData として含む Table を返す
pub fn loader(
    lua: &Lua,
    scene_registry: SceneRegistry,
    word_registry: WordDefRegistry,
) -> Result<Table, mlua::Error>;

/// @pasta_search モジュールを Lua globals に登録
pub fn register(
    lua: &Lua,
    scene_registry: SceneRegistry,
    word_registry: WordDefRegistry,
) -> Result<Table, mlua::Error>;
```

#### API Contract (Lua側)

| Method | Signature | Returns | Errors |
|--------|-----------|---------|--------|
| search_scene | `SEARCH:search_scene(name, global_scene_name?)` | `global_name, local_name` | SceneNotFound |
| search_word | `SEARCH:search_word(name, global_scene_name?)` | `string` | WordNotFound |
| set_scene_selector | `SEARCH:set_scene_selector(n1, n2, ...)` | (none) | type error |
| set_word_selector | `SEARCH:set_word_selector(n1, n2, ...)` | (none) | type error |

**Implementation Notes**
- Integration: mlua-stdlib の loader/register パターンに従う
- Validation: 引数なし呼び出しで Selector リセット
- Risks: なし

---

## Data Models

### Domain Model

```mermaid
classDiagram
    class SearchContext {
        +SceneTable scene_table
        +WordTable word_table
        +search_scene(name, global_scene_name)
        +search_word(name, global_scene_name)
        +set_scene_selector(sequence)
        +set_word_selector(sequence)
    }
    
    class SceneTable {
        +Vec~SceneInfo~ labels
        +RadixMap prefix_index
        +HashMap cache
        +Box~dyn RandomSelector~ random_selector
        +resolve_scene_id_unified(module, key, filters)
    }
    
    class WordTable {
        +Vec~WordEntry~ entries
        +RadixMap prefix_index
        +HashMap cached_selections
        +Box~dyn RandomSelector~ random_selector
        +search_word(module, key, filters)
    }
    
    class RandomSelector {
        <<trait>>
        +select_index(len) Option~usize~
        +shuffle_usize(items)
    }
    
    class DefaultRandomSelector {
        +StdRng rng
    }
    
    class MockRandomSelector {
        +Vec~usize~ sequence
        +usize index
    }
    
    SearchContext --> SceneTable
    SearchContext --> WordTable
    SceneTable --> RandomSelector
    WordTable --> RandomSelector
    DefaultRandomSelector ..|> RandomSelector
    MockRandomSelector ..|> RandomSelector
```

**Aggregates**: SearchContext が SceneTable/WordTable を所有
**Invariants**: RandomSelector は各テーブルで独立管理

---

## Error Handling

### Error Strategy

- **User Errors**: 引数型不正 → mlua が自動検出、エラーメッセージ返却
- **Business Logic**: 検索候補なし → `nil` を返す（エラーではない）
- **System Errors**: 内部エラー → mlua::Error にラップして返却

### Error Categories and Responses

| Category | Condition | Response | Req |
|----------|-----------|----------|-----|
| Type Error | 引数が string でない | "expected string argument" | 6.1 |
| Type Error | Selector 引数が integer でない | "expected integer argument" | 8.9 |
| Not Found | 検索候補なし | `nil` を返す | 6.3 |
| Internal | pasta_core エラー | mlua::Error でラップ | 6.2 |

---

## Testing Strategy

### Unit Tests
- PastaLuaRuntime::new() の初期化成功
- PastaLuaRuntime::exec() のスクリプト実行
- SearchContext::search_scene() の段階的フォールバック動作
- SearchContext::search_word() のローカル/グローバル検索
- set_scene_selector() / set_word_selector() の MockSelector 切り替え
- 引数型検証エラー

### Integration Tests
- PastaLuaRuntime から `require "@pasta_search"` + 検索呼び出し
- 複数 PastaLuaRuntime インスタンスでの独立性検証
- MockSelector 設定後の決定的選択動作

### E2E Tests (Lua)
- pasta.act モジュールからの search_word 呼び出し
- pasta.actor モジュールからの search_scene 呼び出し

---

## Optional Sections

### pasta_core 変更要件

**MockRandomSelector の公開化**:

現在、`MockRandomSelector` は `#[cfg(test)]` で限定されている。Requirement 8 の Lua 側 Selector 制御を実現するため、以下の変更が必要：

```rust
// crates/pasta_core/src/registry/random.rs

// 変更前
#[cfg(test)]
pub struct MockRandomSelector { ... }

// 変更後（オプション A: 常時公開）
pub struct MockRandomSelector { ... }

// 変更後（オプション B: feature フラグ）
#[cfg(any(test, feature = "mock-selector"))]
pub struct MockRandomSelector { ... }
```

**推奨**: オプション A（常時公開）— テスト以外での利用シナリオ（Lua テスト）が正当

---

## File Structure

```
pasta_lua/src/
├── lib.rs (修正: pub mod runtime, pub mod search)
├── runtime/
│   ├── mod.rs
│   │   └── pub struct PastaLuaRuntime
│   └── (将来の拡張用)
├── search/
│   ├── mod.rs
│   │   ├── pub struct SearchContext
│   │   ├── impl UserData for SearchContext
│   │   ├── pub fn loader(...)
│   │   └── pub fn register(...)
│   ├── scene_search.rs
│   │   └── search_scene 実装
│   └── word_search.rs
│       └── search_word 実装
└── (既存ファイル: transpiler.rs, code_generator.rs, etc.)
```

---

## UserData Implementation Detail

```rust
impl mlua::UserData for SearchContext {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        // 不変メソッド（&self）
        // 注: search_scene/search_word は &mut self だがキャッシュ更新のため
        //     add_method_mut を使用
        
        methods.add_method_mut("search_scene", |lua, this, (name, global_scene_name): (String, Option<String>)| {
            let (global, local) = this.search_scene(&name, global_scene_name.as_deref())?;
            Ok((global, local).into_lua_multi(lua)?)
        });
        
        methods.add_method_mut("search_word", |lua, this, (name, global_scene_name): (String, Option<String>)| {
            let word = this.search_word(&name, global_scene_name.as_deref())?;
            Ok(word.into_lua(lua)?)
        });
        
        // 可変メソッド（&mut self）
        methods.add_method_mut("set_scene_selector", |lua, this, args: mlua::MultiValue| {
            if args.is_empty() {
                this.set_scene_selector(None)?;
            } else {
                let sequence: Vec<usize> = args.iter()
                    .map(|v| v.as_integer().ok_or_else(|| mlua::Error::RuntimeError("expected integer argument".into())))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .map(|i| i as usize)
                    .collect();
                this.set_scene_selector(Some(sequence))?;
            }
            Ok(())
        });
        
        methods.add_method_mut("set_word_selector", |lua, this, args: mlua::MultiValue| {
            // 同様の実装
            Ok(())
        });
    }
}
```

**Lua 呼び出し形式**:
- `SEARCH:search_scene("シーン", "親シーン")` — メソッド形式
- `SEARCH.search_scene(SEARCH, "シーン", "親シーン")` — 関数形式（メタテーブルで対応）
