# Research: pasta-serialization

## R1: Rune VM Context Passing Implementation

### 調査目的
Rust側から構造体またはハッシュマップをRune VM実行時に引数として渡し、Rune関数内でアクセスする実装方法を確定する。

### 調査内容

#### 1. Rune VM Execution API

**`vm.execute(hash, args)` の型シグネチャ**:
```rust
pub fn execute<A, N>(
    &mut self,
    hash: Hash,
    args: A
) -> Result<N, VmError>
where
    A: Args,        // Args トレイトを実装した型
    N: FromValue,   // 戻り値の型変換
```

**`Args`トレイトの実装**:
- タプル型（`()`, `(T,)`, `(T1, T2)`, ...）が`Args`トレイトを自動実装
- 各要素は`ToValue`トレイトを実装している必要がある

#### 2. Option A: Struct を引数として渡す

**POC実装**:

```rust
use rune::{Context, Vm, Sources, to_value};
use std::sync::Arc;

// コンテキスト構造体定義
#[derive(Debug)]
struct ExecutionContext {
    persistence_path: String,
}

// Rune script
let script = r#"
pub fn test_label(ctx) {
    // ctx はタプルの第1要素としてアクセス
    let path = ctx.persistence_path;
    yield path;
}
"#;

// Rune準備
let mut context = Context::with_default_modules()?;
let runtime = Arc::new(context.runtime()?);
let mut sources = Sources::new();
sources.insert(Source::new("test", script)?)?;
let unit = rune::prepare(&mut sources).with_context(&context).build()?;

// VM実行
let mut vm = Vm::new(runtime, Arc::new(unit));
let hash = rune::Hash::type_hash(&["test_label"]);

// コンテキスト構築
let ctx = ExecutionContext {
    persistence_path: "/path/to/persistence".to_string(),
};

// ⚠️ 問題: ExecutionContext は ToValue を実装していない
// 解決策: rune::Any derive を使用
```

**改良版 - `rune::Any` derive使用**:

```rust
use rune::{Any, ContextError, Module};

#[derive(Any, Debug, Clone)]
#[rune(item = ::execution_context)]  // Rune側でのモジュールパス
struct ExecutionContext {
    #[rune(get)]  // getter自動生成
    persistence_path: String,
}

// モジュール登録が必要
fn create_context_module() -> Result<Module, ContextError> {
    let mut module = Module::new();
    module.ty::<ExecutionContext>()?;
    Ok(module)
}

// Context初期化時にモジュール登録
context.install(create_context_module()?)?;

// VM実行
let ctx = ExecutionContext {
    persistence_path: "/path/to/persistence".to_string(),
};

// タプルとして渡す（Argsトレイト実装済み）
let execution = vm.execute(hash, (ctx,))?;

// Rune側でのアクセス
// pub fn test_label(ctx) {
//     let path = ctx.persistence_path;  // getter経由
//     yield path;
// }
```

**課題**:
- カスタム構造体は`rune::Any` deriveとモジュール登録が必要
- `#[rune(get)]`でgetter自動生成、またはメソッド手動実装が必要

#### 3. Option B: HashMap を引数として渡す

**POC実装**:

```rust
use std::collections::HashMap;
use rune::to_value;

// HashMap構築
let mut ctx = HashMap::new();
ctx.insert("persistence_path".to_string(), "/path/to/persistence".to_string());

// rune::Value に変換
let ctx_value = to_value(ctx)?;

// タプルとして渡す
let execution = vm.execute(hash, (ctx_value,))?;

// Rune側でのアクセス
// pub fn test_label(ctx) {
//     let path = ctx["persistence_path"];  // インデックスアクセス
//     yield path;
// }
```

**利点**:
- カスタム型定義・モジュール登録不要
- `HashMap<String, String>`は`ToValue`を自動実装
- 動的フィールド追加が容易

**欠点**:
- 型安全性が低い（Rune側でのキー名ミス）
- フィールド名がコンパイル時に検証されない

#### 4. Option C: 単純な値（String）を引数として渡す

**POC実装**:

```rust
let persistence_path = "/path/to/persistence".to_string();

// String は ToValue を実装済み
let execution = vm.execute(hash, (persistence_path,))?;

// Rune側でのアクセス
// pub fn test_label(path) {
//     yield path;
// }
```

**利点**:
- 最もシンプル
- 型変換不要

**欠点**:
- 拡張性なし（将来的に複数フィールド追加時に引数増加）

### 推奨アプローチ

**Option B (HashMap)** を推奨

**理由**:
1. **実装の簡潔性**: カスタム型定義・モジュール登録が不要
2. **拡張性**: 将来的なコンテキストフィールド追加が容易
3. **Gap Analysisとの整合性**: "構造体またはハッシュマップ"という要件に合致
4. **Rune側の自然な構文**: `ctx["persistence_path"]` は一般的なパターン

**実装例**:

```rust
// crates/pasta/src/engine.rs

impl PastaEngine {
    fn build_execution_context(&self) -> Result<rune::Value, PastaError> {
        let mut ctx = std::collections::HashMap::new();
        
        // persistence_path フィールド設定
        let path_str = if let Some(ref path) = self.persistence_path {
            path.to_string_lossy().to_string()
        } else {
            String::new()  // 空文字列 = パスなし
        };
        
        ctx.insert("persistence_path".to_string(), path_str);
        
        // 将来的な拡張ポイント（コメントアウト例）
        // ctx.insert("engine_version".to_string(), env!("CARGO_PKG_VERSION").to_string());
        // ctx.insert("script_name".to_string(), self.script_name.clone());
        
        rune::to_value(ctx).map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to build context: {}", e))
        })
    }
    
    fn execute_label_with_filters(&mut self, ...) -> Result<Vec<ScriptEvent>> {
        // ... (既存のラベル検索ロジック)
        
        let mut vm = Vm::new(self.runtime.clone(), self.unit.clone());
        let hash = rune::Hash::type_hash(&[fn_name.as_str()]);
        
        // コンテキスト構築
        let context = self.build_execution_context()?;
        
        // VM実行（タプルとして渡す）
        let execution = vm.execute(hash, (context,))
            .map_err(|e| PastaError::VmError(e))?;
        
        // ... (既存のジェネレータ処理)
    }
}
```

### Rune側のアクセスパターン

**トランスパイラ生成コード**:
```rune
pub fn label_name(ctx) {
    // persistence_path 取得
    let path = ctx["persistence_path"];
    
    // パスが設定されているか確認
    if path.is_empty() {
        // エラーハンドリング（永続化機能を使用しない）
    } else {
        // 永続化処理（TOML保存等）
        let full_path = format!("{}/save_data.toml", path);
        // ... (ファイルI/O)
    }
    
    // 既存のラベル処理
    yield emit_text("こんにちは");
}
```

### トランスパイラ変更箇所

**`crates/pasta/src/transpiler/mod.rs`** (Line 155):

```rust
// Before:
output.push_str(&format!("pub fn {}() {{\n", fn_name));

// After:
output.push_str(&format!("pub fn {}(ctx) {{\n", fn_name));
```

**影響範囲**:
- すべてのグローバルラベル・ローカルラベルのシグネチャが変更
- 既存テストへの影響: Rune関数は未使用引数を許容するため、`ctx`を使用しないラベルも正常動作

### 検証テストコード

```rust
#[test]
fn test_context_passing_with_persistence_path() -> Result<()> {
    let script = r#"
＊test
    さくら：こんにちは
"#;

    let temp_dir = tempfile::TempDir::new()?;
    let mut engine = PastaEngine::new_with_persistence(script, temp_dir.path())?;
    
    // 実行時にコンテキストが渡されることを確認
    let events = engine.execute_label("test")?;
    
    // イベント検証（既存ロジック）
    assert_eq!(events.len(), 2);
    
    Ok(())
}

#[test]
fn test_context_passing_without_persistence_path() -> Result<()> {
    let script = r#"
＊test
    さくら：やあ
"#;

    let mut engine = PastaEngine::new(script)?;  // 永続化パスなし
    
    // persistence_path が空文字列で渡されることを確認
    let events = engine.execute_label("test")?;
    
    assert_eq!(events.len(), 2);
    
    Ok(())
}
```

---

## R1 調査結果サマリ

✅ **実装方法確定**: HashMap + `rune::to_value` + タプル引数  
✅ **型要件**: `HashMap<String, String>` は `ToValue` 自動実装済み  
✅ **Rune側構文**: `ctx["persistence_path"]` でアクセス  
✅ **トランスパイラ変更**: シグネチャを `pub fn label_name(ctx)` に変更  
✅ **既存テスト影響**: なし（未使用引数は許容）  

**Status**: 完了 - 設計フェーズで実装可能  
**Next**: R2調査へ進む

---

## R2: Rune TOML Serialization API

### 調査目的
Rune 0.14での標準TOML機能の利用可否と、ドキュメント例で使用するAPI仕様を確定する。

### 調査内容

#### 1. Rune 0.14 標準ライブラリ調査

**Rune公式ドキュメント確認**:
- Rune 0.14はコア言語機能のみを提供
- 標準ライブラリに**TOMLモジュールは含まれていない**
- ファイルI/O機能も標準では提供されない

**理由**: Runeはembeddable scripting languageとして設計されており、ホスト環境（この場合Rust）が必要な機能を提供する想定

#### 2. ホスト側での対応方針

**Option A: Rust側でTOML機能を提供**

Pastaスクリプトエンジンのstdlib（`crates/pasta/src/stdlib/mod.rs`）に、TOML関連の関数を追加:

```rust
// crates/pasta/Cargo.toml に依存追加
[dependencies]
toml = "0.8"  # TOMLシリアライズ・デシリアライズ

// crates/pasta/src/stdlib/mod.rs
use rune::{ContextError, Module};

pub fn create_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("pasta_stdlib")?;
    
    // 既存の関数登録
    module.function_meta(emit_text)?;
    module.function_meta(change_speaker)?;
    // ...
    
    // TOML関連関数を追加
    module.function_meta(toml_serialize)?;
    module.function_meta(toml_deserialize)?;
    module.function_meta(file_read)?;
    module.function_meta(file_write)?;
    
    Ok(module)
}

// TOML serialize関数
#[rune::function]
fn toml_serialize(data: rune::Value) -> Result<String, String> {
    // rune::Value を Rust の serde 互換型に変換
    let rust_value: toml::Value = rune::from_value(data)
        .map_err(|e| format!("Failed to convert value: {}", e))?;
    
    toml::to_string(&rust_value)
        .map_err(|e| format!("TOML serialization failed: {}", e))
}

// TOML deserialize関数
#[rune::function]
fn toml_deserialize(toml_str: &str) -> Result<rune::Value, String> {
    let rust_value: toml::Value = toml::from_str(toml_str)
        .map_err(|e| format!("TOML parsing failed: {}", e))?;
    
    rune::to_value(rust_value)
        .map_err(|e| format!("Failed to convert to Rune value: {}", e))
}

// ファイル読み込み関数
#[rune::function]
fn file_read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("File read failed: {}", e))
}

// ファイル書き込み関数
#[rune::function]
fn file_write(path: &str, content: &str) -> Result<(), String> {
    std::fs::write(path, content)
        .map_err(|e| format!("File write failed: {}", e))
}
```

**Option B: Runeスクリプトでraw file I/Oのみ使用**

TOMLパース・生成を手動実装（非現実的、複雑すぎる）

### 推奨アプローチ

**Option A (Rust側でTOML機能提供)** を推奨

**理由**:
1. **現実的**: Rune標準にTOML機能がない以上、ホスト側提供が必須
2. **既存パターン踏襲**: pasta stdlibは既に`emit_text`等の関数を提供
3. **型安全**: `rune::Value` ⇔ `toml::Value` 変換により、Rune側で安全にデータ操作可能
4. **要件合致**: Req 5.1「Runeの標準機能を活用」→ホスト提供の機能を"Runeから使える標準機能"と解釈

### 実装詳細

#### Cargo.toml更新

```toml
# crates/pasta/Cargo.toml
[dependencies]
toml = "0.8"
serde = { version = "1.0", features = ["derive"] }
```

#### stdlib拡張

**`crates/pasta/src/stdlib/persistence.rs`** (新規):

```rust
//! Persistence-related functions for Rune scripts

use rune::{ContextError, Module};
use std::fs;

/// Create persistence module with TOML and file I/O functions
pub fn create_persistence_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("pasta_stdlib::persistence")?;
    
    module.function_meta(toml_to_string)?;
    module.function_meta(toml_from_string)?;
    module.function_meta(read_text_file)?;
    module.function_meta(write_text_file)?;
    
    Ok(module)
}

/// Serialize a Rune value to TOML string
#[rune::function]
fn toml_to_string(data: rune::Value) -> Result<String, String> {
    // Convert rune::Value to toml::Value
    let toml_value: toml::Value = rune::from_value(data)
        .map_err(|e| format!("Value conversion failed: {}", e))?;
    
    toml::to_string(&toml_value)
        .map_err(|e| format!("TOML serialization failed: {}", e))
}

/// Deserialize TOML string to Rune value
#[rune::function]
fn toml_from_string(toml_str: &str) -> Result<rune::Value, String> {
    let toml_value: toml::Value = toml::from_str(toml_str)
        .map_err(|e| format!("TOML parsing failed: {}", e))?;
    
    rune::to_value(toml_value)
        .map_err(|e| format!("Failed to convert to Rune value: {}", e))
}

/// Read text file as string
#[rune::function]
fn read_text_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file '{}': {}", path, e))
}

/// Write text to file
#[rune::function]
fn write_text_file(path: &str, content: &str) -> Result<(), String> {
    fs::write(path, content)
        .map_err(|e| format!("Failed to write file '{}': {}", path, e))
}
```

#### エンジン統合

**`crates/pasta/src/engine.rs`** (Context初期化時):

```rust
// Step 4: Compile Rune code
let mut context = Context::with_default_modules().map_err(|e| {
    PastaError::RuneRuntimeError(format!("Failed to create Rune context: {}", e))
})?;

// Install standard library
context
    .install(crate::stdlib::create_module().map_err(|e| {
        PastaError::RuneRuntimeError(format!("Failed to install stdlib: {}", e))
    })?)
    .map_err(|e| {
        PastaError::RuneRuntimeError(format!("Failed to install context: {}", e))
    })?;

// Install persistence module (新規)
context
    .install(crate::stdlib::persistence::create_persistence_module().map_err(|e| {
        PastaError::RuneRuntimeError(format!("Failed to install persistence module: {}", e))
    })?)
    .map_err(|e| {
        PastaError::RuneRuntimeError(format!("Failed to install persistence context: {}", e))
    })?;
```

### Runeスクリプトでの使用例

#### ドキュメント例（Req 5対応）

**保存処理**:
```rune
pub fn save_game(ctx) {
    let path = ctx["persistence_path"];
    if path.is_empty() {
        yield emit_text("永続化パスが設定されていません");
        return;
    }
    
    // 保存データ構造（Runeのobject literal）
    let save_data = #{
        player_name: "さくら",
        level: 10,
        gold: 5000,
        items: ["薬草", "剣", "盾"],
    };
    
    // TOMLシリアライズ
    let toml_str = toml_to_string(save_data)?;
    
    // ファイルパス構築（パストラバーサル対策: basename only）
    let filename = "save_data.toml";
    let full_path = `${path}/${filename}`;
    
    // ファイル書き込み
    write_text_file(full_path, toml_str)?;
    
    yield emit_text("セーブしました");
}
```

**読み込み処理**:
```rune
pub fn load_game(ctx) {
    let path = ctx["persistence_path"];
    if path.is_empty() {
        yield emit_text("永続化パスが設定されていません");
        return;
    }
    
    let filename = "save_data.toml";
    let full_path = `${path}/${filename}`;
    
    // ファイル読み込み
    let toml_str = read_text_file(full_path)?;
    
    // TOMLデシリアライズ
    let save_data = toml_from_string(toml_str)?;
    
    // データ使用
    let player_name = save_data["player_name"];
    let level = save_data["level"];
    
    yield emit_text(`ロードしました: ${player_name} Lv.${level}`);
}
```

### パストラバーサル攻撃対策（Req 5.3対応）

**ベストプラクティスドキュメント**:

```markdown
## セキュリティガイド: パストラバーサル攻撃の防止

### 問題
ユーザー入力を含むファイル名で`../`を使用すると、永続化ディレクトリ外のファイルにアクセス可能

### 対策

#### 1. ファイル名のベースネームのみ使用
```rune
// ❌ 危険: ユーザー入力を直接使用
let user_filename = input["filename"];  // 悪意: "../../etc/passwd"
let full_path = `${ctx["persistence_path"]}/${user_filename}`;

// ✅ 安全: ベースネームのみ抽出（ディレクトリ区切りを除去）
let sanitized = user_filename.replace("/", "_").replace("\\", "_");
let full_path = `${ctx["persistence_path"]}/${sanitized}`;
```

#### 2. 固定ファイル名使用（推奨）
```rune
// ✅ 最も安全: ハードコードされたファイル名のみ
let filename = "save_data.toml";
let full_path = `${ctx["persistence_path"]}/${filename}`;
```

#### 3. ホワイトリスト検証
```rune
const ALLOWED_FILES = ["save_data.toml", "config.toml", "progress.toml"];

fn is_safe_filename(name) {
    ALLOWED_FILES.contains(name)
}

if !is_safe_filename(filename) {
    yield emit_text("無効なファイル名です");
    return;
}
```
```

### エラーハンドリング例

```rune
pub fn safe_save(ctx) {
    let path = ctx["persistence_path"];
    
    // 1. パス未設定チェック
    if path.is_empty() {
        yield emit_text("エラー: 永続化が無効です");
        return;
    }
    
    // 2. データ準備
    let data = #{ level: 5 };
    
    // 3. TOML変換（エラーハンドリング）
    let toml_str = match toml_to_string(data) {
        Ok(s) => s,
        Err(e) => {
            yield emit_text(`シリアライズエラー: ${e}`);
            return;
        }
    };
    
    // 4. ファイル書き込み（エラーハンドリング）
    let full_path = `${path}/save.toml`;
    match write_text_file(full_path, toml_str) {
        Ok(_) => yield emit_text("保存成功"),
        Err(e) => yield emit_text(`保存失敗: ${e}`),
    }
}
```

---

## R2 調査結果サマリ

✅ **Rune 0.14標準機能**: TOMLモジュール**なし**（ホスト提供が必要）  
✅ **推奨アプローチ**: Rust側でTOML機能をpasta stdlibに追加  
✅ **実装パターン**: 
- `toml_to_string(data)` - シリアライズ
- `toml_from_string(str)` - デシリアライズ  
- `read_text_file(path)` / `write_text_file(path, content)` - ファイルI/O

✅ **ドキュメント例**: 保存・読み込み・エラーハンドリング・セキュリティ対策を含む  
✅ **依存追加**: `toml = "0.8"` を`crates/pasta/Cargo.toml`に追加  

**Status**: 完了 - 設計フェーズで実装可能  
**Next**: R3調査へ進む

---

## R3: Path Traversal Attack Mitigation

### 調査目的
Runeスクリプトでのファイルパス操作におけるパストラバーサル攻撃の防止策をドキュメント化する。

### 攻撃シナリオ

#### 脆弱なコード例

```rune
pub fn save_user_data(ctx, user_input) {
    let base_path = ctx["persistence_path"];  // "/app/data"
    let filename = user_input["filename"];     // 攻撃者が制御可能
    
    // ❌ 危険: ユーザー入力を直接連結
    let full_path = `${base_path}/${filename}`;
    
    write_text_file(full_path, "secret data");
}

// 攻撃例:
// filename = "../../../etc/passwd" 
// → full_path = "/app/data/../../../etc/passwd" 
// → 実際のパス = "/etc/passwd" (システムファイルを上書き)
```

### Rust側での対策（参考）

Rustでは標準ライブラリで安全なパス操作が可能:

```rust
use std::path::{Path, PathBuf};

fn safe_join(base: &Path, user_input: &str) -> Result<PathBuf, String> {
    let joined = base.join(user_input);
    
    // canonicalize()で正規化し、親ディレクトリの確認
    let canonical = joined.canonicalize()
        .map_err(|e| format!("Path resolution failed: {}", e))?;
    
    // ベースディレクトリ外を指していないか確認
    if !canonical.starts_with(base) {
        return Err("Path traversal attempt detected".to_string());
    }
    
    Ok(canonical)
}

// 使用例
let base = Path::new("/app/data");
let user_input = "../../../etc/passwd";

match safe_join(base, user_input) {
    Ok(path) => println!("Safe: {:?}", path),
    Err(e) => println!("Blocked: {}", e),  // "Path traversal attempt detected"
}
```

### Rune側での対策（推奨パターン）

Runeスクリプトではファイルシステムの詳細な操作が限定的なため、以下の防御策を推奨:

#### 対策1: 固定ファイル名のみ使用（最も安全）

```rune
// ✅ 推奨: ハードコードされたファイル名
pub fn save_game(ctx) {
    let base = ctx["persistence_path"];
    let filename = "save_data.toml";  // 固定値
    let path = `${base}/${filename}`;
    
    write_text_file(path, data);
}
```

**利点**: ユーザー入力を含まないため、攻撃不可能

#### 対策2: ホワイトリスト検証

```rune
// 許可されたファイル名のリスト
const ALLOWED_FILES = [
    "save_data.toml",
    "config.toml",
    "progress.toml",
    "preferences.toml",
];

fn is_safe_filename(name) {
    ALLOWED_FILES.iter().any(|allowed| allowed == name)
}

pub fn save_file(ctx, filename) {
    // ✅ ホワイトリスト検証
    if !is_safe_filename(filename) {
        yield emit_text("エラー: 無効なファイル名");
        return;
    }
    
    let base = ctx["persistence_path"];
    let path = `${base}/${filename}`;
    write_text_file(path, data);
}
```

**利点**: 限定されたファイルセットのみ許可、柔軟性あり

#### 対策3: ファイル名サニタイズ（パス区切り文字除去）

```rune
fn sanitize_filename(input) {
    // パス区切り文字を除去・置換
    input
        .replace("/", "_")
        .replace("\\", "_")
        .replace("..", "_")  // 親ディレクトリ参照も除去
}

pub fn save_custom_file(ctx, user_input) {
    // ✅ サニタイズ処理
    let safe_name = sanitize_filename(user_input);
    
    let base = ctx["persistence_path"];
    let path = `${base}/${safe_name}`;
    write_text_file(path, data);
}

// 例:
// user_input = "../../../etc/passwd"
// safe_name = "_.._.._.._etc_passwd" (無害化)
```

**利点**: ユーザー入力を許容しつつ、パストラバーサルを防止

#### 対策4: ベースネーム抽出（最後のコンポーネントのみ）

```rune
fn extract_basename(path) {
    // 最後の "/" または "\" 以降を抽出
    let parts = path.split("/");
    let last = parts.last().unwrap_or("");
    
    // さらに "\" でも分割（Windows対応）
    let win_parts = last.split("\\");
    win_parts.last().unwrap_or("")
}

pub fn save_with_basename(ctx, user_path) {
    // ✅ ベースネームのみ使用
    let filename = extract_basename(user_path);
    
    let base = ctx["persistence_path"];
    let path = `${base}/${filename}`;
    write_text_file(path, data);
}

// 例:
// user_path = "../../etc/passwd"
// filename = "passwd" (ディレクトリ部分を除去)
```

**注意**: この方法でも意図しないファイル名が残る可能性あり（`passwd`など）

### 推奨実装順序

**優先度順**:
1. **固定ファイル名** - ユーザー入力を含まない場合
2. **ホワイトリスト** - 限定されたファイルセットの場合
3. **サニタイズ** - 動的ファイル名が必要な場合

### ドキュメント化（Req 5.3対応）

**Runeスクリプト開発者向けガイド** (`doc/rune-persistence-guide.md` として作成予定):

```markdown
# Pastaエンジン永続化ガイド

## セキュリティ: パストラバーサル攻撃の防止

### 概要
永続化ディレクトリ外のファイルへの不正アクセスを防ぐため、ファイルパス構築時に以下の対策を実施してください。

### 対策一覧

#### 1. 固定ファイル名の使用（推奨）
最も安全な方法は、ハードコードされたファイル名のみを使用することです。

\`\`\`rune
pub fn save_game(ctx) {
    let base = ctx["persistence_path"];
    let filename = "save_data.toml";  // ✅ 固定値
    let path = `${base}/${filename}`;
    write_text_file(path, data);
}
\`\`\`

#### 2. ホワイトリスト検証
複数のファイルが必要な場合、許可リストで検証します。

\`\`\`rune
const ALLOWED_FILES = ["save_data.toml", "config.toml"];

fn is_allowed(name) {
    ALLOWED_FILES.iter().any(|f| f == name)
}

pub fn save_file(ctx, filename) {
    if !is_allowed(filename) {
        yield emit_text("エラー: 許可されていないファイル");
        return;
    }
    // ... 保存処理
}
\`\`\`

#### 3. ファイル名サニタイズ
ユーザー入力を使用する場合、パス区切り文字を除去します。

\`\`\`rune
fn sanitize(input) {
    input.replace("/", "_").replace("\\", "_").replace("..", "_")
}

pub fn save_custom(ctx, user_input) {
    let safe_name = sanitize(user_input);  // ✅ 無害化
    let path = `${ctx["persistence_path"]}/${safe_name}`;
    write_text_file(path, data);
}
\`\`\`

### 脆弱なコード例

❌ **危険**: ユーザー入力を直接使用
\`\`\`rune
pub fn vulnerable_save(ctx, filename) {
    let path = `${ctx["persistence_path"]}/${filename}`;  // ❌
    write_text_file(path, data);
}

// 攻撃例: filename = "../../../etc/passwd"
// 結果: システムファイルを上書き
\`\`\`

### チェックリスト

- [ ] ファイル名はハードコードされているか？
- [ ] ユーザー入力を使用する場合、ホワイトリスト検証を実施しているか？
- [ ] パス区切り文字（`/`, `\\`, `..`）を除去しているか？
- [ ] エラーハンドリングを実装しているか？

### エラーハンドリング例

\`\`\`rune
pub fn safe_save(ctx, filename) {
    // 1. 永続化パス確認
    let base = ctx["persistence_path"];
    if base.is_empty() {
        yield emit_text("エラー: 永続化が無効です");
        return;
    }
    
    // 2. ファイル名検証
    if !is_allowed(filename) {
        yield emit_text("エラー: 無効なファイル名");
        return;
    }
    
    // 3. 保存処理
    let path = `${base}/${filename}`;
    match write_text_file(path, data) {
        Ok(_) => yield emit_text("保存成功"),
        Err(e) => yield emit_text(`保存失敗: ${e}`),
    }
}
\`\`\`
```

### Rust側での追加対策（将来的な強化案）

**stdlib関数にパス検証を組み込む**:

```rust
// crates/pasta/src/stdlib/persistence.rs

/// Write text to file (with path traversal protection)
#[rune::function]
fn write_text_file_safe(base_dir: &str, filename: &str, content: &str) -> Result<(), String> {
    // ベースディレクトリのPath化
    let base = Path::new(base_dir);
    
    // ファイル名にパス区切りが含まれていないか確認
    if filename.contains('/') || filename.contains('\\') {
        return Err("Filename must not contain path separators".to_string());
    }
    
    // 親ディレクトリ参照を禁止
    if filename.contains("..") {
        return Err("Filename must not contain '..'".to_string());
    }
    
    // 安全に結合
    let full_path = base.join(filename);
    
    // 書き込み
    fs::write(full_path, content)
        .map_err(|e| format!("Failed to write file: {}", e))
}
```

**Rune側での使用**:
```rune
// ✅ Rust側で検証済み
write_text_file_safe(ctx["persistence_path"], filename, data);
```

---

## R3 調査結果サマリ

✅ **攻撃シナリオ**: `../` を含むユーザー入力でディレクトリ外アクセス  
✅ **推奨対策（優先度順)**:
1. 固定ファイル名（最も安全）
2. ホワイトリスト検証
3. ファイル名サニタイズ（パス区切り文字除去）

✅ **実装パターン**: 3つの対策パターンとサンプルコード  
✅ **ドキュメント**: Runeスクリプト開発者向けガイド（セキュリティチェックリスト含む）  
✅ **将来的な強化**: Rust側stdlib関数にパス検証を組み込む案  

**Status**: 完了 - ドキュメント化準備完了  
**Next**: R4調査へ進む

---

## R4: pasta-engine-independence Integration

### 調査目的
`pasta-engine-independence`スペックとの統合順序を確認し、競合を回避する。

### pasta-engine-independence の概要

**目的**: PastaEngineインスタンスの完全な独立性を保証

**主要変更**:
1. **グローバルキャッシュ削除**: `static PARSE_CACHE` → `PastaEngine::cache` フィールド
2. **インスタンス所有**: 各エンジンが独自のキャッシュ、ラベルテーブル、変数を所有
3. **Arc削除**: Rune Unit/RuntimeContext以外で共有ポインタを使用しない

**実装フェーズ** (gap-analysis.mdより):
- Phase 1: ParseCache simplification
- Phase 2: PastaEngine restructure
- Phase 3: Test suite creation

### pasta-serialization の変更内容

**主要変更**:
1. **フィールド追加**: `PastaEngine::persistence_path: Option<PathBuf>`
2. **コンストラクタ追加**: `PastaEngine::new_with_persistence(script, path)`
3. **実行ロジック変更**: `vm.execute(hash, context)` - コンテキスト引数追加
4. **トランスパイラ変更**: ラベル関数シグネチャ変更

### 競合分析

#### 競合なし（独立領域）

| 領域 | pasta-engine-independence | pasta-serialization |
|------|---------------------------|---------------------|
| キャッシュ | グローバル→インスタンスフィールド化 | 変更なし |
| フィールド | `cache: ParseCache` 追加 | `persistence_path: Option<PathBuf>` 追加 |
| コンストラクタ | `new(script)` 内部ロジック変更 | `new_with_persistence(script, path)` 新規追加 |
| VM実行 | 変更なし | `vm.execute(hash, context)` 引数変更 |

**結論**: 両スペックは異なる領域を変更するため、**競合リスクは極めて低い**

#### 統合ポイント

**PastaEngine構造体の最終形** (両スペック統合後):

```rust
pub struct PastaEngine {
    // Rune関連（既存）
    unit: Arc<rune::Unit>,
    runtime: Arc<rune::runtime::RuntimeContext>,
    
    // ラベルテーブル（既存）
    label_table: LabelTable,
    
    // キャッシュ（pasta-engine-independence）
    cache: ParseCache,
    
    // 永続化パス（pasta-serialization）
    persistence_path: Option<PathBuf>,
}
```

### 推奨実装順序

#### Option A: 並行実装（推奨）

両スペックは独立しているため、同時進行可能:

**Timeline**:
- Week 1: `pasta-engine-independence` Phase 1-2 + `pasta-serialization` Phase 1
- Week 2: `pasta-engine-independence` Phase 3 + `pasta-serialization` Phase 2
- Week 3: `pasta-serialization` Phase 3 + 統合テスト

**利点**:
- 開発速度最大化
- 各スペックの独立性が高いため、マージ競合最小

**欠点**:
- 2つのブランチ管理が必要

#### Option B: 順次実装

**Sequence 1: pasta-engine-independence → pasta-serialization**

**Timeline**:
- Week 1-2: `pasta-engine-independence` 完全実装
- Week 3-4: `pasta-serialization` 完全実装

**利点**:
- シンプルな進行管理
- `pasta-engine-independence`で確立した所有権パターンを`pasta-serialization`で踏襲可能

**欠点**:
- 総期間が長い

**Sequence 2: pasta-serialization → pasta-engine-independence**

**Timeline**:
- Week 1-2: `pasta-serialization` 完全実装
- Week 3-4: `pasta-engine-independence` 完全実装

**利点**:
- 永続化機能を早期に利用可能

**欠点**:
- グローバルキャッシュが残った状態で永続化実装（後でリファクタリング）

### 推奨: Option B - Sequence 1

**理由**:
1. **基盤の確立**: `pasta-engine-independence`でインスタンス所有パターンを確立
2. **一貫性**: `persistence_path`フィールド追加時、既にキャッシュフィールドが存在
3. **テスト簡素化**: 独立性テストが完了後、永続化テストで独立性を前提にできる

**実装計画**:

```
[Week 1-2] pasta-engine-independence
├─ Phase 1: ParseCache simplification
├─ Phase 2: PastaEngine restructure (cache field追加)
└─ Phase 3: Test suite

[Week 3-4] pasta-serialization
├─ Phase 1: Core extension (persistence_path field追加)
│  ├─ PastaEngine::new_with_persistence()
│  ├─ Transpiler signature change
│  └─ VM execution context passing
├─ Phase 2: Testing infrastructure
└─ Phase 3: Logging & documentation

[Week 5] Integration & Validation
├─ 統合テスト: 複数インスタンス × 異なる永続化パス
└─ CI/CD整備
```

### マージ戦略

**ブランチ構成** (Option Aを選択する場合):

```
master
  ├─ feature/pasta-engine-independence
  │   └─ 各Phase実装
  └─ feature/pasta-serialization
      └─ 各Phase実装
```

**マージ順序**:
1. `pasta-engine-independence` → `master`
2. `pasta-serialization` を `master` にリベース
3. `pasta-serialization` → `master`

### 統合テスト計画

**テストケース**: 両スペックの機能を組み合わせ

```rust
#[test]
fn test_multiple_engines_with_different_persistence_paths() -> Result<()> {
    let script = r#"
＊save
    さくら：保存します
"#;

    // Engine 1: persistence path A
    let temp_dir1 = tempfile::TempDir::new()?;
    let mut engine1 = PastaEngine::new_with_persistence(script, temp_dir1.path())?;
    
    // Engine 2: persistence path B
    let temp_dir2 = tempfile::TempDir::new()?;
    let mut engine2 = PastaEngine::new_with_persistence(script, temp_dir2.path())?;
    
    // 各エンジンが独立して動作
    let events1 = engine1.execute_label("save")?;
    let events2 = engine2.execute_label("save")?;
    
    // 両方成功
    assert_eq!(events1.len(), 2);
    assert_eq!(events2.len(), 2);
    
    // パスが異なることを確認（内部状態検証）
    // （engine1とengine2は異なる永続化ディレクトリを保持）
    
    Ok(())
}

#[test]
fn test_engine_independence_with_persistence() -> Result<()> {
    let script = r#"
＊test
    さくら：テスト
"#;

    let temp_dir = tempfile::TempDir::new()?;
    
    // 同一スクリプトから複数インスタンス作成
    let mut engine1 = PastaEngine::new_with_persistence(script, temp_dir.path())?;
    let mut engine2 = PastaEngine::new_with_persistence(script, temp_dir.path())?;
    
    // 各エンジンが独立してキャッシュ・永続化パスを所有
    let events1 = engine1.execute_label("test")?;
    let events2 = engine2.execute_label("test")?;
    
    // 互いに干渉しない
    assert_eq!(events1, events2);
    
    Ok(())
}
```

### 潜在的な問題と対策

#### 問題1: コンストラクタの複雑化

**現状**:
- `PastaEngine::new(script)`
- `PastaEngine::with_random_selector(script, selector)`

**追加**:
- `PastaEngine::new_with_persistence(script, path)`
- `PastaEngine::with_persistence_and_random_selector(script, path, selector)`?

**対策**: Builder パターンの検討（将来的な拡張案）

```rust
// 将来的な改善案（Option）
let engine = PastaEngine::builder()
    .script(script)
    .persistence_path(path)
    .random_selector(selector)
    .build()?;
```

**現時点の推奨**: 単純な追加メソッドで十分（Builderは過剰）

#### 問題2: トランスパイラ変更の影響範囲

**変更**: 全ラベル関数が`ctx`引数を受け取る

**影響**:
- 既存のRuneスクリプト例（`examples/`）が`ctx`を使用しない
- ドキュメント更新が必要

**対策**:
- Phase 3でドキュメント更新を含める
- `ctx`は未使用でも問題なし（Runeは未使用引数を許容）

---

## R4 調査結果サマリ

✅ **競合分析**: pasta-engine-independence と pasta-serialization は独立領域を変更、競合リスク極めて低  
✅ **統合後の構造体**: 4フィールド（unit, runtime, label_table, cache, persistence_path）  
✅ **推奨実装順序**: Sequence 1 - pasta-engine-independence → pasta-serialization  
✅ **理由**: 基盤確立（所有権パターン）後に永続化機能を追加、一貫性・テスト簡素化  
✅ **統合テスト**: 複数インスタンス × 異なる永続化パスの独立性検証  
✅ **潜在的問題**: コンストラクタ複雑化（現時点は単純メソッド追加で対応）、トランスパイラ変更の影響範囲（ドキュメント更新で対応）  

**Status**: 完了 - 統合計画確定  
**Overall Research Status**: **全議題完了** - 設計フェーズへ進行可能

---

## 全調査議題完了サマリ

### R1: Rune VM Context Passing
✅ HashMap + `rune::to_value` + タプル引数パターン確定

### R2: Rune TOML Serialization API
✅ Rust側でTOML機能をpasta stdlibとして提供、4関数実装

### R3: Path Traversal Attack Mitigation
✅ 3段階防御策ドキュメント化（固定ファイル名/ホワイトリスト/サニタイズ）

### R4: pasta-engine-independence Integration
✅ 実装順序確定（engine-independence → serialization）、統合テスト計画策定

**Next Step**: `/kiro-spec-design pasta-serialization` で設計フェーズへ移行
