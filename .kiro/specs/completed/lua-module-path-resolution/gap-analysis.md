# ギャップ分析: lua-module-path-resolution

## 1. 現状調査

### 1.1 関連ファイル・モジュール構成

| ファイル                                                                                                           | 責務                                                                       |
| ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- |
| [crates/pasta_lua/src/loader/config.rs](crates/pasta_lua/src/loader/config.rs)                                     | `default_lua_search_paths()` 定義、`LoaderConfig`構造体                    |
| [crates/pasta_lua/src/loader/context.rs](crates/pasta_lua/src/loader/context.rs)                                   | `LoaderContext`構造体、`generate_package_path()`メソッド                   |
| [crates/pasta_lua/src/loader/mod.rs](crates/pasta_lua/src/loader/mod.rs)                                           | `PastaLoader::load()`、7フェーズの初期化シーケンス                         |
| [crates/pasta_lua/src/loader/cache.rs](crates/pasta_lua/src/loader/cache.rs)                                       | `CacheManager::generate_scene_dic()`、scene_dic.lua生成                    |
| [crates/pasta_lua/src/runtime/mod.rs](crates/pasta_lua/src/runtime/mod.rs)                                         | `from_loader_with_scene_dic()`、`load_scene_dic()`、`setup_package_path()` |
| [crates/pasta_sample_ghost/templates/pasta.toml.template](crates/pasta_sample_ghost/templates/pasta.toml.template) | サンプルゴースト設定テンプレート                                           |
| [crates/pasta_lua/scripts/](crates/pasta_lua/scripts/)                                                             | pasta標準ランタイム（現在main.luaは存在しない）                            |

### 1.2 現在のLua検索パス設定

```rust
// crates/pasta_lua/src/loader/config.rs L166-172
fn default_lua_search_paths() -> Vec<String> {
    vec![
        "profile/pasta/save/lua".to_string(),
        "scripts".to_string(),
        "profile/pasta/cache/lua".to_string(),
        "scriptlibs".to_string(),
    ]
}
```

**現状**: `user_scripts`ディレクトリは未設定

### 1.3 現在の初期化シーケンス

```
PastaLoader::load() (loader/mod.rs L92-)
├── Phase 1: Load configuration
├── Phase 2: Prepare directories and cache
├── Phase 3: Discover pasta files
├── Phase 4: Incremental transpilation
├── Phase 5: Generate scene_dic.lua
├── Phase 6: Create logger
└── Phase 7: Initialize runtime
     └── PastaLuaRuntime::from_loader_with_scene_dic()
          ├── Setup package.path
          ├── Register modules (@pasta_config, @enc, etc.)
          ├── Load entry.lua (直接ファイル読み込み) ← 問題点
          ├── Register finalize_scene
          └── Load scene_dic.lua (直接ファイル読み込み) ← 問題点
```

### 1.4 現在のファイル読み込みパターン（問題点）

#### entry.lua読み込み（L549-564）
```rust
let entry_lua_path = loader_context
    .base_dir
    .join("scripts/pasta/shiori/entry.lua");
if entry_lua_path.exists() {
    match std::fs::read_to_string(&entry_lua_path) {
        Ok(script) => {
            runtime.lua.load(&script).set_name("entry.lua").exec()?;
        }
        // ...
    }
}
```

#### scene_dic.lua読み込み（L592-600）
```rust
pub fn load_scene_dic(&self, scene_dic_path: &Path) -> LuaResult<()> {
    let script = std::fs::read_to_string(scene_dic_path)
        .map_err(|e| mlua::Error::ExternalError(Arc::new(e)))?;
    self.lua.load(&script).set_name("pasta.scene_dic").exec()?;
    // ...
}
```

**問題**: 両方ともRustで直接ファイルを読み込んでおり、Luaの`package.path`による検索パス解決が使われていない。

### 1.5 既存のrequireパターン

コードベース内で`require`を使用している例：
```rust
// runtime/mod.rs L743
let save_table: Table = match self.lua.load(r#"require("pasta.save")"#).eval() {
```

このパターンは`lua.load()`でLuaコードを実行し、`require()`を呼び出している。

---

## 2. 要件実現可能性分析

### 2.1 要件マップ

| 要件                                    | 現状               | ギャップ                           | 難易度 |
| --------------------------------------- | ------------------ | ---------------------------------- | ------ |
| Req 1: user_scripts検索パス追加         | ❌ 未設定           | `default_lua_search_paths()`修正   | 低     |
| Req 2: require()ベース読み込み統一      | ❌ 直接読み込み     | ヘルパー関数作成＋呼び出し箇所修正 | 中     |
| Req 3: 初期化順序変更（main→scene_dic） | ❌ scene_dic先行    | 初期化シーケンス再構成             | 中     |
| Req 4: デフォルトmain.lua               | ❌ 存在しない       | scripts/main.lua新規作成           | 低     |
| Req 5: pasta_sample_ghost更新           | ❌ 旧設定           | テンプレート＋テスト更新           | 低     |
| Req 6: 後方互換性                       | ✅ 既存動作維持可能 | デフォルト値で吸収                 | 低     |

### 2.2 技術的課題

#### 課題1: requireヘルパー関数の設計

**選択肢A**: Luaコード経由で`require()`呼び出し
```rust
fn lua_require(lua: &Lua, module_name: &str) -> LuaResult<Value> {
    lua.load(&format!("return require(\"{}\")", module_name))
       .set_name(&format!("require({})", module_name))
       .eval()
}
```

**選択肢B**: mluaのglobalsから`require`関数を取得して呼び出し
```rust
fn lua_require(lua: &Lua, module_name: &str) -> LuaResult<Value> {
    let require: Function = lua.globals().get("require")?;
    require.call(module_name)
}
```

**推奨**: 選択肢B（より直接的でエラーハンドリングが明確）

#### 課題2: 初期化順序の変更

現在：
1. Register modules
2. Load entry.lua
3. Register finalize_scene
4. Load scene_dic.lua（finalize_scene呼び出し含む）

提案：
1. Register modules
2. Register finalize_scene
3. **require("main")** ← 新規追加
4. Load scene_dic.lua（またはrequire("pasta.scene_dic")）

**注意点**: entry.luaの読み込みは`require("pasta.shiori.entry")`に変更可能だが、現在固定パスを使用しているため影響範囲を検討が必要。

#### 課題3: main.luaが存在しない場合の挙動

Luaの`require()`はモジュールが見つからない場合エラーを返す。
デフォルトの空`scripts/main.lua`が存在すれば、ユーザーが何も配置しなくても動作する。

---

## 3. 実装アプローチ選択肢

### Option A: 最小変更アプローチ（既存コンポーネント拡張）

**変更対象**:
- `config.rs`: `default_lua_search_paths()`に`user_scripts`追加
- `runtime/mod.rs`: `lua_require()`ヘルパー追加、`from_loader_with_scene_dic()`修正
- `scripts/main.lua`: 新規作成
- `templates/pasta.toml.template`: 検索パス更新
- 統合テスト: 検証追加

**Trade-offs**:
- ✅ 変更箇所が明確で限定的
- ✅ 既存テストへの影響最小
- ✅ 後方互換性維持が容易
- ❌ scene_dic.luaは引き続き直接読み込み（パスが動的生成のため）

### Option B: フルrequireアプローチ（新規コンポーネント作成）

**変更対象**:
- Option Aのすべて
- 加えて: scene_dic.luaもrequireベースに変更
- `pasta.scene_dic`モジュールを検索パス内に配置する方式に変更

**Trade-offs**:
- ✅ 完全なLuaルール準拠
- ✅ 一貫性のある設計
- ✅ ユーザーが上書きした場合はユーザー責任（例外を設けない原則）
- ⚠️ scene_dic.luaはキャッシュディレクトリに生成されるが、requireで解決可能

~~### Option C: ハイブリッドアプローチ（推奨）~~

**Option C は廃止**: 例外を設ける理由がないため、Option B（フルrequireアプローチ）を採用。

---

## 4. 実装複雑度・リスク評価

### 見積もり

| 項目       | 値            | 根拠                                   |
| ---------- | ------------- | -------------------------------------- |
| **工数**   | **S** (1-3日) | 既存パターン拡張、限定的な変更箇所     |
| **リスク** | **低**        | 既知の技術、明確なスコープ、テスト可能 |

### リスク項目

1. **エンコーディング**: Windows環境でのパス解決（既存の`generate_package_path_bytes()`で対応済み）
2. **テスト影響**: 既存テストの検索パス確認箇所の更新が必要
3. **user_scriptsディレクトリ不在**: Luaの検索パスに含めても、ディレクトリが存在しない場合は単にスキップされる（問題なし）

---

## 5. 設計フェーズへの推奨事項

### 優先アプローチ: Option B（フルrequireアプローチ）

**設計原則**: Lua検索パス優先順位による上書き可能領域を極限まで広げる。例外は設けない。

### キー決定事項

1. **requireヘルパー関数**: `globals().get("require")`方式を採用
2. **初期化順序**: main.lua → entry.lua → scene_dic.lua（全てrequireベース）
3. **entry.luaの扱い**: `require("pasta.shiori.entry")`に変更（例外なし）
4. **scene_dic.lua**: `require("pasta.scene_dic")`に変更（例外なし）
5. **ユーザー上書き時の責任**: ユーザーが上書きした場合の挙動はユーザー責任

### 次フェーズで検討が必要な項目

- [ ] `lua_require()`ヘルパーのエラーハンドリング詳細設計
- [ ] デフォルトmain.luaのコメント内容（ユーザー向けサンプルコード）
- [ ] 統合テストの具体的なテストケース設計

---

## 6. 要件-資産マッピング（サマリ）

| 要件ID | 既存資産                       | ギャップタグ   | 対応方針                            |
| ------ | ------------------------------ | -------------- | ----------------------------------- |
| Req 1  | `default_lua_search_paths()`   | **Missing**    | パス追加                            |
| Req 2  | `lua.load()`                   | **Constraint** | ヘルパー関数新設、全箇所をrequire化 |
| Req 3  | `from_loader_with_scene_dic()` | **Missing**    | 順序変更                            |
| Req 4  | なし                           | **Missing**    | scripts/main.lua新規                |
| Req 5  | `pasta.toml.template`          | **Missing**    | テンプレート更新                    |
| Req 6  | デフォルト値機構               | **OK**         | 既存機構で対応                      |
