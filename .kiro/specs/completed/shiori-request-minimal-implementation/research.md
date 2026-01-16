# リサーチログ: shiori-request-minimal-implementation

> このドキュメントは Light Discovery プロセスの結果を記録します。

## 1. 拡張ポイント分析

### 1.1 PastaShiori 構造体

**ファイル**: [crates/pasta_shiori/src/shiori.rs](crates/pasta_shiori/src/shiori.rs)

```rust
pub(crate) struct PastaShiori {
    hinst: isize,
    load_dir: Option<PathBuf>,
    runtime: Option<PastaLuaRuntime>,
}
```

**拡張方針**:
- `shiori_load_fn: Option<mlua::Function>` フィールド追加
- `shiori_request_fn: Option<mlua::Function>` フィールド追加
- `Default` derive は維持可能（Option 型のため）

**影響範囲**:
- `load()` メソッド: 関数取得ロジック追加
- `request()` メソッド: TODO スタブを実装に置換
- `Drop` 実装: 変更不要（Option が自動 Drop）

### 1.2 PastaLuaRuntime

**ファイル**: [crates/pasta_lua/src/runtime/mod.rs](crates/pasta_lua/src/runtime/mod.rs)

**関連メソッド**:
- `lua() -> &Lua`: グローバルテーブルアクセス用
- `from_loader()`: 初期化フロー（main.lua ロード追加ポイント）

**観察事項**:
- `from_loader()` の最後で transpiled コードをロード後に main.lua をロード可能
- `exec_file()` メソッドで直接 main.lua を実行可能
- `lua().globals().get::<_, Table>("SHIORI")` で SHIORI テーブル取得パターン確認

### 1.3 PastaLoader

**ファイル**: [crates/pasta_lua/src/loader/mod.rs](crates/pasta_lua/src/loader/mod.rs)

**観察事項**:
- `load()` → `load_with_config()` → `create_runtime()` フロー
- `create_runtime()` 内で `PastaLuaRuntime::from_loader()` 呼び出し
- main.lua ロードは `from_loader()` 内部 or 直後に追加可能

## 2. 依存関係チェック

### 2.1 mlua::Function ライフタイム

**ドキュメント**: [docs.rs/mlua/latest/mlua/struct.Function.html](https://docs.rs/mlua/latest/mlua/struct.Function.html)

**確認事項**:
- `Function<'lua>` は Lua VM へのリファレンス保持
- `PastaLuaRuntime` が生存している限り `Function` も有効
- `PastaShiori.runtime: Option<PastaLuaRuntime>` → `runtime` Drop 時に関数参照も無効化
- **結論**: `Function` は `runtime` と同じ生存期間で安全

### 2.2 mlua::Function::call

**シグネチャ**:
```rust
pub fn call<A, R>(&self, args: A) -> Result<R>
where
    A: IntoLuaMulti,
    R: FromLuaMulti,
```

**使用例**:
```rust
// (hinst: isize, load_dir: String) → bool
let result: bool = shiori_load_fn.call((hinst, load_dir_str))?;

// (request: String) → String  
let response: String = shiori_request_fn.call((req_str,))?;
```

### 2.3 エラー変換

**既存パターン**:
```rust
impl From<pasta_lua::LoaderError> for MyError {
    fn from(error: pasta_lua::LoaderError) -> MyError {
        MyError::Load(format!("{}", error))
    }
}
```

**追加必要**:
```rust
impl From<mlua::Error> for MyError
```

## 3. 統合リスク評価

| リスク項目 | レベル | 理由 |
|-----------|--------|------|
| ライフタイム管理 | Low | runtime と同一生存期間で保証 |
| 関数呼び出しオーバーヘッド | Low | 文字列処理のみ、パフォーマンスクリティカルではない |
| main.lua 不在 | Low | Option 型で None 許容、デフォルト 204 返却 |
| SHIORI テーブル不在 | Low | 警告ログ後デフォルト動作 |
| Lua 実行時エラー | Low | MyError::Script で伝播 |

## 4. 設計決定事項

### 4.1 main.lua ロードタイミング

**決定**: `PastaLoader::load()` 内部で transpiled コードロード後に main.lua をロード

**理由**:
- 責務の集約（Loader がすべての初期化を完了）
- PastaShiori は関数取得のみ担当

### 4.2 関数参照取得タイミング

**決定**: `PastaShiori::load()` 内で `PastaLoader::load()` 完了後に取得

**理由**:
- main.lua ロード後に SHIORI テーブルが存在
- load() 内で一度だけ取得（パフォーマンス最適化）

### 4.3 main.lua 不在時の動作

**決定**: warn ログを出力し、SHIORI 関数参照は None のまま

**理由**:
- 後方互換性維持（既存 fixture は main.lua なし）
- 最小実装の範囲では必須としない

### 4.4 SHIORI.load false 戻り時の動作

**決定**: `PastaShiori::load()` から `Ok(false)` を返却

**理由**:
- SHIORI プロトコル仕様に準拠
- Lua 側で初期化失敗を制御可能

## 5. 参照コード例

### 5.1 グローバルテーブルからの関数取得

**既存パターン（register_module）**:
```rust
pub fn register_module(&self, name: &str, module: Table) -> LuaResult<()> {
    let package: Table = self.lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;
    loaded.set(name, module)?;
    Ok(())
}
```

**SHIORI テーブル取得パターン**:
```rust
let shiori_table: Table = self.runtime.lua().globals().get("SHIORI")?;
let load_fn: Function = shiori_table.get("load")?;
let request_fn: Function = shiori_table.get("request")?;
```

### 5.2 エラーレスポンス生成

**既存パターン（error.rs）**:
```rust
pub fn to_shiori_response(&self) -> String {
    format!(
        "SHIORI/3.0 500 Internal Server Error\r\n\
         Charset: UTF-8\r\n\
         X-ERROR-REASON: {}\r\n\
         \r\n",
        self
    )
}
```
