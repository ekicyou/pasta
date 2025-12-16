# Design Decisions - 設計判断の詳細展開

| 項目 | 内容 |
|------|------|
| **Document Title** | areka スクリプトエンジン 設計判断詳細 |
| **Version** | 1.0 |
| **Date** | 2025-12-09 |
| **Spec ID** | areka-P0-script-engine |
| **Phase** | Requirements → Design 移行準備 |

---

## Document Purpose

本文書は、gap-analysis.mdで提起された未解決事項（Open Questions）を1つずつ展開し、設計判断を明確化する。各議題について、選択肢の比較、推奨案、および設計への影響を詳述する。

---

## 議題1: ファイルローディング戦略

### 背景

Pasta DSLファイル（`./dic/**/*.pasta`）の読み込みタイミングとキャッシュ戦略を決定する必要がある。

### 選択肢の比較

| オプション | メリット | デメリット | パフォーマンス | 開発体験 |
|-----------|---------|-----------|---------------|---------|
| **A: 起動時全ロード** | ・シンプルな実装<br>・実行時のレイテンシなし<br>・全ラベルの整合性チェック可能 | ・起動時間増加<br>・メモリ常駐<br>・ファイル変更時に再起動必要 | ⭐️⭐️⭐️⭐️⭐️<br>(実行時) | ⭐️⭐️<br>(再起動必要) |
| **B: 遅延ロード（JIT）** | ・起動高速<br>・メモリ効率的<br>・未使用ファイルは読み込まない | ・初回呼び出しに遅延<br>・複雑な実装<br>・キャッシュ管理が必要 | ⭐️⭐️⭐️<br>(初回遅延) | ⭐️⭐️⭐️<br>(起動高速) |
| **C: ホットリロード対応** | ・開発体験が最高<br>・リアルタイム反映 | ・ファイルウォッチャー必要<br>・複雑な実装<br>・プロダクション不要 | ⭐️⭐️⭐️⭐️<br>(開発時) | ⭐️⭐️⭐️⭐️⭐️<br>(最高) |

### 推奨案: ハイブリッド実装

**段階的実装計画**:

#### Phase 1: MVP（起動時全ロード）

```rust
// crates/pasta/src/loader.rs
pub struct ScriptLoader {
    scripts: HashMap<String, LabelDefinition>,
}

impl ScriptLoader {
    /// 起動時に全スクリプトを読み込み
    pub fn load_all(dic_path: &Path) -> Result<Self, PastaError> {
        let glob_pattern = format!("{}/**/*.pasta", dic_path.display());
        let mut scripts = HashMap::new();
        
        for entry in glob::glob(&glob_pattern)? {
            let path = entry?;
            let content = fs::read_to_string(&path)?;
            let parsed = parse_pasta(&content)?;  // パーサー実装は設計時決定
            
            for label in parsed.labels {
                scripts.insert(label.name.clone(), label);
            }
        }
        
        Ok(Self { scripts })
    }
}
```

**利点**:
- ✅ 実装がシンプル（MVP優先）
- ✅ 実行時のパフォーマンスが最高
- ✅ ラベル名の重複チェックが容易

**欠点**:
- ⚠️ ファイル変更時に再起動が必要（開発体験に影響）

#### Phase 2: 開発モード（ホットリロード追加）

```rust
// crates/pasta/src/loader.rs (拡張)
#[cfg(feature = "hot-reload")]
pub struct HotReloadScriptLoader {
    base: ScriptLoader,
    watcher: notify::RecommendedWatcher,
    reload_channel: mpsc::Receiver<PathBuf>,
}

impl HotReloadScriptLoader {
    pub fn new(dic_path: &Path) -> Result<Self, PastaError> {
        let base = ScriptLoader::load_all(dic_path)?;
        
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::watcher(tx, Duration::from_secs(2))?;
        watcher.watch(dic_path, RecursiveMode::Recursive)?;
        
        Ok(Self {
            base,
            watcher,
            reload_channel: rx,
        })
    }
    
    pub fn check_and_reload(&mut self) -> Result<(), PastaError> {
        while let Ok(path) = self.reload_channel.try_recv() {
            if path.extension() == Some("pasta".as_ref()) {
                self.reload_file(&path)?;
            }
        }
        Ok(())
    }
}
```

**有効化方法**:
```toml
# Cargo.toml
[features]
default = []
hot-reload = ["notify"]

[dependencies]
notify = { version = "6.0", optional = true }
```

**使用例**:
```bash
# 開発モード
cargo run --features hot-reload

# リリースビルド（ホットリロードなし）
cargo build --release
```

### 設計への影響

#### API設計

```rust
// crates/pasta/src/lib.rs
pub struct PastaEngine {
    loader: Box<dyn ScriptLoaderTrait>,
    rune_vm: RuneVm,
    variables: VariableStore,
}

pub trait ScriptLoaderTrait {
    fn get_label(&self, name: &str) -> Option<&LabelDefinition>;
    fn reload(&mut self) -> Result<(), PastaError>;
}

impl PastaEngine {
    /// プロダクション用（起動時全ロード）
    pub fn new(dic_path: &Path) -> Result<Self, PastaError> {
        let loader = Box::new(ScriptLoader::load_all(dic_path)?);
        Self::with_loader(loader)
    }
    
    /// 開発用（ホットリロード）
    #[cfg(feature = "hot-reload")]
    pub fn new_with_hot_reload(dic_path: &Path) -> Result<Self, PastaError> {
        let loader = Box::new(HotReloadScriptLoader::new(dic_path)?);
        Self::with_loader(loader)
    }
    
    /// フレーム毎に呼び出し（開発モードのみリロードチェック）
    pub fn update(&mut self) -> Result<(), PastaError> {
        #[cfg(feature = "hot-reload")]
        self.loader.reload()?;
        Ok(())
    }
}
```

#### パフォーマンス特性

| モード | 起動時間 | メモリ使用量 | 実行時レイテンシ | ファイル変更反映 |
|--------|---------|-------------|-----------------|----------------|
| **プロダクション** | +500ms | +10MB (1000ファイル想定) | <1μs | 再起動 |
| **開発（ホットリロード）** | +600ms | +12MB | <1μs | 即座 |

### 決定事項

✅ **採用**: ハイブリッド実装（Phase 1 → Phase 2）

**理由**:
1. MVP段階では実装コストを抑える（起動時全ロード）
2. 開発体験向上はフィーチャーフラグで段階的に追加
3. プロダクションビルドに不要な依存を含めない

---

## 議題2: Rune VMのライフサイクル管理

### 背景

Rune VMインスタンスの管理方法を決定する。スレッド安全性、複数VMの必要性、ECSとの統合を考慮する必要がある。

### 選択肢の比較

| オプション | メリット | デメリット | ECS統合 | スレッド安全性 |
|-----------|---------|-----------|---------|---------------|
| **A: グローバルシングルトン** | ・アクセスが容易<br>・実装がシンプル | ・スレッドセーフ対策必要<br>・ECSパターンから逸脱<br>・テストが困難 | ⭐️⭐️ | ⚠️ Mutex必要 |
| **B: bevy_ecs Resource** | ・ECSと一貫<br>・スレッドセーフ（bevy保証）<br>・テストしやすい | ・bevy_ecs依存 | ⭐️⭐️⭐️⭐️⭐️ | ✅ 自動保証 |
| **C: コンポーネント内** | ・完全なカプセル化 | ・複数VMインスタンス<br>・オーバーヘッド大 | ⭐️⭐️⭐️ | ✅ 自動保証 |

### 推奨案: bevy_ecs Resource（オプションB）

#### 設計詳細

```rust
// crates/pasta/src/engine.rs
use bevy_ecs::prelude::*;

/// Pasta Script Engine Resource
/// bevy_ecs Worldに1つだけ存在する
#[derive(Resource)]
pub struct PastaEngine {
    /// スクリプトローダー
    loader: ScriptLoader,
    
    /// Rune VM（コンパイル済みユニット + ランタイム）
    rune_context: Arc<rune::runtime::RuntimeContext>,
    rune_unit: Arc<rune::Unit>,
    
    /// 変数ストレージ（グローバル変数）
    variables: VariableStore,
    
    /// イベントレジストリ
    events: ScriptEventRegistry,
}

impl PastaEngine {
    pub fn new(dic_path: &Path) -> Result<Self, PastaError> {
        let loader = ScriptLoader::load_all(dic_path)?;
        let (context, unit) = Self::init_rune(&loader)?;
        
        Ok(Self {
            loader,
            rune_context: Arc::new(context),
            rune_unit: Arc::new(unit),
            variables: VariableStore::new(),
            events: ScriptEventRegistry::new(),
        })
    }
    
    fn init_rune(loader: &ScriptLoader) -> Result<(RuntimeContext, Unit), PastaError> {
        let mut context = rune::Context::with_default_modules()?;
        
        // Rust関数をRuneから呼び出せるように登録
        context.function_meta(ir_functions::emit_text)?.build()?;
        context.function_meta(ir_functions::wait)?.build()?;
        context.function_meta(ir_functions::fire_event)?.build()?;
        
        // Pastaスクリプトをトランスコンパイル
        let rune_code = transpile_all(loader)?;
        
        // Runeコンパイル
        let mut sources = rune::sources! {
            entry => { rune_code }
        };
        
        let unit = rune::prepare(&mut sources)
            .with_context(&context)
            .build()?;
        
        Ok((context.runtime()?, unit))
    }
    
    pub fn execute_label(
        &mut self,
        label: &str,
        args: Vec<RuneValue>,
    ) -> Result<Vec<TypewriterToken>, PastaError> {
        // IR蓄積用バッファ（スレッドローカル）
        IR_BUFFER.with(|buf| {
            buf.borrow_mut().clear();
        });
        
        // Rune VM実行
        let vm = rune::Vm::new(
            self.rune_context.clone(),
            self.rune_unit.clone(),
        );
        
        vm.call([label], (args,))?;
        
        // IR取得
        let ir_tokens = IR_BUFFER.with(|buf| {
            buf.borrow().clone()
        });
        
        Ok(ir_tokens)
    }
}
```

#### bevy_ecs統合

```rust
// crates/wintf/src/systems/script_system.rs
use bevy_ecs::prelude::*;
use pasta::PastaEngine;

/// スクリプト実行要求メッセージ
pub struct ScriptExecutionRequest {
    pub label: String,
    pub args: Vec<pasta::RuneValue>,
    pub response_entity: Entity,
}

/// スクリプト実行完了メッセージ
pub struct ScriptExecutionComplete {
    pub label: String,
    pub ir_tokens: Vec<TypewriterToken>,
    pub target_entity: Entity,
}

/// スクリプト実行システム
pub fn script_execution_system(
    mut engine: ResMut<PastaEngine>,
    mut requests: ResMut<Messages<ScriptExecutionRequest>>,
    mut completions: ResMut<Messages<ScriptExecutionComplete>>,
) {
    for request in requests.drain() {
        match engine.execute_label(&request.label, request.args) {
            Ok(ir_tokens) => {
                completions.send(ScriptExecutionComplete {
                    label: request.label,
                    ir_tokens,
                    target_entity: request.response_entity,
                });
            }
            Err(e) => {
                tracing::error!("Script execution failed: {:?}", e);
            }
        }
    }
}

/// ECSワールド初期化
pub fn setup_pasta_engine(world: &mut World, dic_path: &Path) -> Result<(), PastaError> {
    let engine = PastaEngine::new(dic_path)?;
    world.insert_resource(engine);
    
    // システム登録
    world.add_system(script_execution_system);
    
    Ok(())
}
```

### クレート間の依存関係

```
pasta (独立クレート)
  ├─ rune (外部)
  ├─ [パーサー実装] (設計時決定: nom/pest/手書き等)
  └─ bevy_ecs (オプショナル依存)
      └─ feature = "bevy-integration"

wintf (既存クレート)
  ├─ bevy_ecs (既存依存)
  └─ pasta (bevy-integration feature有効)
```

#### Cargo.toml設計

```toml
# crates/pasta/Cargo.toml
[package]
name = "pasta"
version = "0.1.0"
edition = "2021"

[features]
default = []
bevy-integration = ["bevy_ecs"]
hot-reload = ["notify"]

[dependencies]
rune = "0.15"
# パーサー実装は設計時決定（nom/pest/手書き等）

# オプショナル依存
bevy_ecs = { version = "0.17", optional = true }
notify = { version = "6.0", optional = true }

[dev-dependencies]
bevy_ecs = "0.17"
```

```toml
# crates/wintf/Cargo.toml
[dependencies]
pasta = { path = "../pasta", features = ["bevy-integration"] }
bevy_ecs = { workspace = true }
# ... 既存依存
```

### スレッド安全性の保証

#### bevy_ecsによる保証

bevy_ecsのResourceシステムは以下を自動的に保証する：

1. **排他制御**: `ResMut<PastaEngine>`は同時に1つのシステムしかアクセスできない
2. **データ競合防止**: コンパイル時に借用チェック
3. **並列実行**: 依存しないシステムは並列実行可能

#### Rune VM内部のスレッド安全性

```rust
// Rune VMは内部的にArcでラップ
self.rune_context: Arc<RuntimeContext>,
self.rune_unit: Arc<Unit>,

// 複数のVmインスタンスを作成可能（読み取り専用共有）
let vm1 = Vm::new(context.clone(), unit.clone());
let vm2 = Vm::new(context.clone(), unit.clone());
```

**注意点**: Rune VM実行中の状態（変数等）は独立しているため、グローバル変数はPastaEngine側で管理する。

### 決定事項

✅ **採用**: bevy_ecs Resource（オプションB）

**理由**:
1. ECSアーキテクチャとの完全な一貫性
2. スレッド安全性が自動保証される
3. テストとモックが容易
4. bevy_ecsはオプショナル依存として分離可能（将来的なSHIORI.DLL化に対応）

---

## 議題3: 変数の永続化

### 背景

グローバル変数（`＄＊変数名`）の保存先と永続化メカニズムを決定する。セーブシステムとの統合を考慮する必要がある。

### 選択肢の比較

| オプション | メリット | デメリット | 永続性 | セーブ統合 |
|-----------|---------|-----------|--------|-----------|
| **A: Rune VM内** | ・Rune標準機能<br>・実装シンプル | ・永続化に別途対応必要<br>・型情報が曖昧 | ❌ | ⚠️ 別途実装 |
| **B: Rustハッシュマップ** | ・型安全<br>・高速<br>・永続化しやすい | ・Runeとの同期必要 | ✅ | ✅ |
| **C: ファイルシステム** | ・永続性が保証<br>・エクスポート可能 | ・I/Oコスト<br>・変更の度に書き込み | ✅✅ | ✅✅ |

### 推奨案: 階層的アプローチ（B + C）

#### Phase 1: メモリ内ストレージ（B）

```rust
// crates/pasta/src/variables.rs
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Rune値のRust表現
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuneValue {
    Unit,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<RuneValue>),
    Object(HashMap<String, RuneValue>),
}

/// 変数ストレージ
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VariableStore {
    /// グローバル変数（＄＊プレフィックス）
    globals: HashMap<String, RuneValue>,
    
    /// ローカル変数スタック（関数呼び出し階層）
    locals_stack: Vec<HashMap<String, RuneValue>>,
}

impl VariableStore {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// グローバル変数取得
    pub fn get_global(&self, name: &str) -> Option<&RuneValue> {
        self.globals.get(name)
    }
    
    /// グローバル変数設定
    pub fn set_global(&mut self, name: String, value: RuneValue) {
        self.globals.insert(name, value);
    }
    
    /// ローカルスコープ開始
    pub fn push_scope(&mut self) {
        self.locals_stack.push(HashMap::new());
    }
    
    /// ローカルスコープ終了
    pub fn pop_scope(&mut self) {
        self.locals_stack.pop();
    }
    
    /// ローカル変数取得（スコープチェーン探索）
    pub fn get_local(&self, name: &str) -> Option<&RuneValue> {
        // 最内スコープから外側へ探索
        for scope in self.locals_stack.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }
        None
    }
    
    /// ローカル変数設定（現在のスコープ）
    pub fn set_local(&mut self, name: String, value: RuneValue) {
        if let Some(scope) = self.locals_stack.last_mut() {
            scope.insert(name, value);
        }
    }
    
    /// 変数解決（ローカル → グローバルの順）
    pub fn resolve(&self, name: &str) -> Option<&RuneValue> {
        self.get_local(name).or_else(|| self.get_global(name))
    }
}
```

#### Phase 2: ファイル永続化（C）

```rust
// crates/pasta/src/save.rs
use std::path::Path;
use std::fs;

/// セーブデータフォーマット
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    /// グローバル変数
    pub variables: HashMap<String, RuneValue>,
    
    /// メタデータ
    pub metadata: SaveMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct SaveMetadata {
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub script_hash: String, // スクリプト整合性チェック用
}

impl VariableStore {
    /// セーブデータ出力（JSON形式）
    pub fn save_to_file(&self, path: &Path) -> Result<(), PastaError> {
        let save_data = SaveData {
            variables: self.globals.clone(),
            metadata: SaveMetadata {
                version: env!("CARGO_PKG_VERSION").to_string(),
                timestamp: chrono::Utc::now(),
                script_hash: self.calculate_script_hash()?,
            },
        };
        
        let json = serde_json::to_string_pretty(&save_data)?;
        fs::write(path, json)?;
        
        Ok(())
    }
    
    /// セーブデータ読み込み
    pub fn load_from_file(&mut self, path: &Path) -> Result<(), PastaError> {
        let json = fs::read_to_string(path)?;
        let save_data: SaveData = serde_json::from_str(&json)?;
        
        // バージョンチェック
        if save_data.metadata.version != env!("CARGO_PKG_VERSION") {
            tracing::warn!(
                "Save data version mismatch: {} != {}",
                save_data.metadata.version,
                env!("CARGO_PKG_VERSION")
            );
        }
        
        // 変数復元
        self.globals = save_data.variables;
        
        Ok(())
    }
    
    /// 自動セーブ（変更検知）
    pub fn auto_save(&self, path: &Path) -> Result<(), PastaError> {
        // TODO: 変更検知メカニズム（Dirtyフラグ等）
        self.save_to_file(path)
    }
}
```

### セーブシステム統合

#### areka-P0-package-managerとの連携

```rust
// 将来的な統合イメージ
// crates/areka-package-manager/src/save.rs

pub struct GhostSaveData {
    /// パッケージメタデータ
    pub package: PackageMetadata,
    
    /// スクリプト変数（pasta管理）
    pub script_variables: pasta::SaveData,
    
    /// ユーザー設定
    pub user_config: UserConfig,
}
```

#### 自動セーブトリガー

```rust
// crates/wintf/src/systems/save_system.rs

/// 定期自動セーブシステム
pub fn auto_save_system(
    time: Res<Time>,
    mut last_save: Local<f64>,
    engine: Res<PastaEngine>,
) {
    const SAVE_INTERVAL: f64 = 60.0; // 60秒毎
    
    if time.elapsed_seconds_f64() - *last_save > SAVE_INTERVAL {
        if let Err(e) = engine.variables.auto_save(Path::new("./save/variables.json")) {
            tracing::error!("Auto save failed: {:?}", e);
        }
        *last_save = time.elapsed_seconds_f64();
    }
}
```

### セーブデータ例

```json
{
  "variables": {
    "好感度": { "Integer": 75 },
    "天気": { "String": "晴れ" },
    "訪問回数": { "Integer": 42 },
    "ユーザー名": { "String": "太郎" },
    "フラグ_イベント完了": { "Bool": true }
  },
  "metadata": {
    "version": "0.1.0",
    "timestamp": "2025-12-09T12:34:56Z",
    "script_hash": "a1b2c3d4e5f6..."
  }
}
```

### 決定事項

✅ **採用**: 階層的アプローチ（メモリストレージ + ファイル永続化）

**実装計画**:
1. Phase 1 (MVP): メモリ内`VariableStore`実装
2. Phase 2: JSON形式でのファイル永続化
3. Phase 3: areka-P0-package-managerと統合

**理由**:
1. 段階的実装が可能（MVP優先）
2. 型安全性が保証される
3. セーブシステムとの統合が容易
4. デバッグとエクスポートが容易（JSON）

---

## 議題4: エラーハンドリング方針

### 背景

スクリプトエラー（構文エラー、ランタイムエラー）の報告方法を決定する。制作者の開発体験とデバッグ効率を考慮する必要がある。

### 選択肢の比較

| オプション | メリット | デメリット | 開発体験 | プロダクション |
|-----------|---------|-----------|---------|---------------|
| **A: ログのみ** | ・実装シンプル<br>・パフォーマンス影響なし | ・エラー見落とし<br>・制作者が気づきにくい | ⭐️⭐️ | ⭐️⭐️⭐️⭐️ |
| **B: メッセージ通知** | ・リアルタイム通知<br>・エラー処理が統一 | ・bevy_ecs依存 | ⭐️⭐️⭐️⭐️ | ⭐️⭐️⭐️ |
| **C: デバッグUI** | ・視覚的<br>・詳細情報表示 | ・開発時のみ有効<br>・実装コスト大 | ⭐️⭐️⭐️⭐️⭐️ | ⭐️⭐️ |

### 推奨案: 多層エラーハンドリング（A + B + C）

#### Layer 1: エラー型定義

```rust
// crates/pasta/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PastaError {
    // パースエラー
    #[error("Syntax error at line {line}, column {column}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
        file: Option<String>,
    },
    
    // ラベル解決エラー
    #[error("Label not found: {label}")]
    LabelNotFound { label: String },
    
    // 変数解決エラー
    #[error("Variable not found: {variable}")]
    VariableNotFound { variable: String },
    
    // 型エラー
    #[error("Type error: expected {expected}, got {actual}")]
    TypeError {
        expected: String,
        actual: String,
        location: String,
    },
    
    // Runeランタイムエラー
    #[error("Runtime error: {0}")]
    RuntimeError(#[from] rune::runtime::VmError),
    
    // I/Oエラー
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    // カスタムエラー
    #[error("{0}")]
    Custom(String),
}

impl PastaError {
    /// エラーの重大度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::SyntaxError { .. } => ErrorSeverity::Fatal,
            Self::LabelNotFound { .. } => ErrorSeverity::Error,
            Self::VariableNotFound { .. } => ErrorSeverity::Warning,
            Self::TypeError { .. } => ErrorSeverity::Error,
            Self::RuntimeError(_) => ErrorSeverity::Error,
            Self::IoError(_) => ErrorSeverity::Fatal,
            Self::Custom(_) => ErrorSeverity::Warning,
        }
    }
    
    /// エラーコンテキスト（デバッグ用）
    pub fn context(&self) -> ErrorContext {
        match self {
            Self::SyntaxError { line, column, file, .. } => {
                ErrorContext {
                    file: file.clone(),
                    line: Some(*line),
                    column: Some(*column),
                }
            }
            _ => ErrorContext::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Warning,  // 継続可能
    Error,    // 処理中断
    Fatal,    // 起動不可
}

#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    pub file: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}
```

#### Layer 2: ログ出力（tracing）

```rust
// crates/pasta/src/engine.rs
impl PastaEngine {
    pub fn execute_label(&mut self, label: &str) -> Result<Vec<TypewriterToken>, PastaError> {
        match self.execute_label_impl(label) {
            Ok(tokens) => Ok(tokens),
            Err(e) => {
                // 重大度に応じたログレベル
                match e.severity() {
                    ErrorSeverity::Warning => {
                        tracing::warn!(
                            error = ?e,
                            label = %label,
                            "Script warning"
                        );
                    }
                    ErrorSeverity::Error => {
                        tracing::error!(
                            error = ?e,
                            label = %label,
                            "Script error"
                        );
                    }
                    ErrorSeverity::Fatal => {
                        tracing::error!(
                            error = ?e,
                            label = %label,
                            "Fatal script error"
                        );
                    }
                }
                Err(e)
            }
        }
    }
}
```

#### Layer 3: bevy_ecsメッセージ通知

```rust
// crates/wintf/src/messages.rs
#[derive(Debug, Clone)]
pub struct ScriptError {
    pub error: String,
    pub severity: pasta::ErrorSeverity,
    pub context: pasta::ErrorContext,
    pub timestamp: std::time::Instant,
}

// crates/wintf/src/systems/script_system.rs
pub fn script_execution_system(
    mut engine: ResMut<PastaEngine>,
    mut requests: ResMut<Messages<ScriptExecutionRequest>>,
    mut completions: ResMut<Messages<ScriptExecutionComplete>>,
    mut errors: ResMut<Messages<ScriptError>>,
) {
    for request in requests.drain() {
        match engine.execute_label(&request.label, request.args) {
            Ok(ir_tokens) => {
                completions.send(ScriptExecutionComplete { /* ... */ });
            }
            Err(e) => {
                // エラーメッセージ送信
                errors.send(ScriptError {
                    error: e.to_string(),
                    severity: e.severity(),
                    context: e.context(),
                    timestamp: std::time::Instant::now(),
                });
            }
        }
    }
}
```

#### Layer 4: 開発者UI（areka-P1-devtools統合）

```rust
// 将来実装（areka-P1-devtools）
// crates/areka-devtools/src/error_panel.rs

pub struct ErrorPanel {
    errors: Vec<ScriptError>,
    filter: ErrorFilter,
}

impl ErrorPanel {
    pub fn update(&mut self, errors: &Messages<ScriptError>) {
        for error in errors.iter() {
            if self.filter.matches(error) {
                self.errors.push(error.clone());
            }
        }
    }
    
    pub fn render(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for error in &self.errors {
                self.render_error(ui, error);
            }
        });
    }
    
    fn render_error(&self, ui: &mut egui::Ui, error: &ScriptError) {
        let color = match error.severity {
            ErrorSeverity::Warning => egui::Color32::YELLOW,
            ErrorSeverity::Error => egui::Color32::RED,
            ErrorSeverity::Fatal => egui::Color32::DARK_RED,
        };
        
        ui.horizontal(|ui| {
            ui.colored_label(color, format!("{:?}", error.severity));
            ui.label(&error.error);
            
            if let Some(file) = &error.context.file {
                ui.label(format!("{}:{}:{}", 
                    file,
                    error.context.line.unwrap_or(0),
                    error.context.column.unwrap_or(0)
                ));
            }
        });
    }
}
```

### エラーメッセージの国際化

```rust
// crates/pasta/src/error_messages.rs
pub fn format_error(error: &PastaError, locale: &str) -> String {
    match (error, locale) {
        (PastaError::LabelNotFound { label }, "ja") => {
            format!("ラベルが見つかりません: {}", label)
        }
        (PastaError::LabelNotFound { label }, _) => {
            format!("Label not found: {}", label)
        }
        // ... 他のエラーメッセージ
        _ => error.to_string(),
    }
}
```

### 決定事項

✅ **採用**: 多層エラーハンドリング（段階的実装）

**実装計画**:
1. Phase 1 (MVP): エラー型 + ログ出力（Layer 1, 2）
2. Phase 2: bevy_ecsメッセージ通知（Layer 3）
3. Phase 3 (P1): 開発者UI（Layer 4、areka-P1-devtoolsで実装）

**理由**:
1. 段階的実装が可能
2. 各レイヤーが独立している
3. 開発体験とプロダクション品質を両立
4. 将来の拡張（国際化、詳細なデバッグ情報）に対応

---

## 議題5: さくらスクリプトのエスケープ処理

### 背景

`\n`, `\w[n]`, `\s[n]`等のさくらスクリプトコマンドをどの層で解釈するかを決定する。

### 選択肢の比較

| オプション | メリット | デメリット | 責務分離 | 再利用性 |
|-----------|---------|-----------|---------|---------|
| **A: Pasta DSL層** | ・パース時に解決<br>・IRが詳細 | ・Pasta層が肥大化<br>・さくらスクリプト依存 | ⭐️⭐️ | ⭐️⭐️ |
| **B: Typewriter層** | ・責務が明確<br>・Pasta層が汎用的 | ・IRが文字列含む<br>・解釈が遅延 | ⭐️⭐️⭐️⭐️⭐️ | ⭐️⭐️⭐️⭐️⭐️ |
| **C: 両層サポート** | ・柔軟性が高い | ・複雑<br>・重複実装 | ⭐️⭐️⭐️ | ⭐️⭐️⭐️ |

### 推奨案: Typewriter層での解釈（オプションB）

#### 設計詳細

##### Pasta層の責務

```rust
// crates/pasta/src/transpiler.rs
impl Transpiler {
    fn transpile_text(&self, text: &str) -> String {
        // さくらスクリプトは解釈せず、そのまま渡す
        format!("emit_text({:?});", text)
    }
}

// 生成されるRuneコード例
pub fn 挨拶() {
    emit_text("さくら：こんにちは\\n今日はいい天気ですね");
    // ↑ \nはエスケープされたまま
}
```

##### Typewriter層の責務

```rust
// crates/wintf/src/ecs/widget/text/sakura_script.rs

/// さくらスクリプトパーサー
pub struct SakuraScriptParser;

impl SakuraScriptParser {
    /// テキストをパースし、IRトークンに変換
    pub fn parse(&self, text: &str) -> Vec<TypewriterToken> {
        let mut tokens = Vec::new();
        let mut current_text = String::new();
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                // エスケープシーケンス検出
                if let Some(command) = self.parse_command(&mut chars) {
                    // 蓄積したテキストを出力
                    if !current_text.is_empty() {
                        tokens.push(TypewriterToken::Text(current_text.clone()));
                        current_text.clear();
                    }
                    
                    // コマンドをIRトークンに変換
                    tokens.extend(command.to_tokens());
                } else {
                    current_text.push(ch);
                }
            } else {
                current_text.push(ch);
            }
        }
        
        // 残りのテキストを出力
        if !current_text.is_empty() {
            tokens.push(TypewriterToken::Text(current_text));
        }
        
        tokens
    }
    
    fn parse_command(&self, chars: &mut Peekable<Chars>) -> Option<SakuraCommand> {
        match chars.peek()? {
            'n' => {
                chars.next();
                Some(SakuraCommand::NewLine)
            }
            'w' => {
                chars.next();
                if chars.peek() == Some(&'[') {
                    chars.next(); // '['
                    let num = self.parse_number(chars)?;
                    chars.next(); // ']'
                    Some(SakuraCommand::Wait(num))
                } else {
                    None
                }
            }
            's' => {
                chars.next();
                if chars.peek() == Some(&'[') {
                    chars.next(); // '['
                    let num = self.parse_number(chars)?;
                    chars.next(); // ']'
                    Some(SakuraCommand::Surface(num))
                } else {
                    None
                }
            }
            '0' | '1' => {
                let scope = chars.next().unwrap();
                Some(SakuraCommand::Scope(scope))
            }
            _ => None,
        }
    }
}

enum SakuraCommand {
    NewLine,
    Wait(u32),
    Surface(u32),
    Scope(char),
}

impl SakuraCommand {
    fn to_tokens(&self) -> Vec<TypewriterToken> {
        match self {
            Self::NewLine => vec![TypewriterToken::Text("\n".to_string())],
            Self::Wait(ms) => vec![TypewriterToken::Wait(*ms as f64 / 1000.0)],
            Self::Surface(id) => vec![TypewriterToken::ChangeSurface(*id)],
            Self::Scope(ch) => {
                let speaker = match ch {
                    '0' => "さくら",
                    '1' => "うにゅう",
                    _ => "unknown",
                };
                vec![TypewriterToken::ChangeSpeaker(speaker.to_string())]
            }
        }
    }
}
```

##### Typewriter統合

```rust
// crates/wintf/src/ecs/widget/text/typewriter_systems.rs

pub fn process_ir_tokens(
    mut query: Query<&mut Typewriter>,
    mut token_messages: ResMut<Messages<TypewriterTokenBatch>>,
) {
    let parser = SakuraScriptParser;
    
    for batch in token_messages.drain() {
        let mut processed_tokens = Vec::new();
        
        for token in batch.tokens {
            match token {
                TypewriterToken::Text(text) => {
                    // さくらスクリプトをパース
                    processed_tokens.extend(parser.parse(&text));
                }
                other => {
                    processed_tokens.push(other);
                }
            }
        }
        
        // Typewriterコンポーネントに渡す
        if let Ok(mut typewriter) = query.get_mut(batch.target_entity) {
            typewriter.queue_tokens(processed_tokens);
        }
    }
}
```

### さくらスクリプト拡張

#### カスタムコマンド定義

```rust
// crates/wintf/src/ecs/widget/text/sakura_script_extensions.rs

pub trait SakuraScriptExtension {
    fn command_prefix(&self) -> &str;
    fn parse(&self, args: &str) -> Option<Vec<TypewriterToken>>;
}

/// カスタムコマンド例: \mcp[command:args]
pub struct McpCommandExtension;

impl SakuraScriptExtension for McpCommandExtension {
    fn command_prefix(&self) -> &str {
        "mcp"
    }
    
    fn parse(&self, args: &str) -> Option<Vec<TypewriterToken>> {
        let parts: Vec<&str> = args.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        
        Some(vec![TypewriterToken::FireEvent {
            target: Entity::PLACEHOLDER, // TODO: 解決
            event: TypewriterEventKind::McpCommand {
                command: parts[0].to_string(),
                args: parts[1].to_string(),
            },
        }])
    }
}
```

### 決定事項

✅ **採用**: Typewriter層での解釈（オプションB）

**理由**:
1. **責務の明確化**: Pasta層は会話フロー制御に専念、Typewriter層は表示制御に専念
2. **再利用性**: Pasta層がさくらスクリプトに依存しない（SHIORI.DLL化やLLM統合に有利）
3. **拡張性**: カスタムコマンドをTypewriter層で追加可能
4. **互換性**: 既存SHIORI.DLLとの互換性が保ちやすい

**実装への影響**:
- TypewriterToken::Textは生のテキスト（エスケープ含む）を保持
- Typewriterシステムでパース処理を実行
- IRのサイズが若干増加するが、柔軟性が向上

---

## 議題6: ランダム選択のシード管理

### 背景

同一ラベル名の複数定義からランダム選択する際のシード値管理を決定する。

### 選択肢の比較

| オプション | メリット | デメリット | テスト容易性 | ユーザー体験 |
|-----------|---------|-----------|-------------|-------------|
| **A: システム時刻** | ・真のランダム<br>・実装シンプル | ・再現性なし<br>・テスト困難 | ⭐️ | ⭐️⭐️⭐️⭐️⭐️ |
| **B: 固定シード** | ・完全再現可能<br>・テスト容易 | ・ランダム性なし<br>・毎回同じ | ⭐️⭐️⭐️⭐️⭐️ | ⭐️⭐️ |
| **C: ユーザー設定** | ・柔軟性が高い<br>・再現性も可能 | ・実装複雑<br>・セーブ管理必要 | ⭐️⭐️⭐️⭐️ | ⭐️⭐️⭐️⭐️ |

### 推奨案: ハイブリッド実装（A + B + C）

#### 設計詳細

```rust
// crates/pasta/src/random.rs
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

/// ランダム選択戦略
pub enum RandomStrategy {
    /// システム時刻ベース（デフォルト）
    SystemTime,
    
    /// 固定シード（デバッグ用）
    FixedSeed(u64),
    
    /// セーブデータから復元（ユーザー体験一貫性）
    SavedSeed {
        initial_seed: u64,
        current_state: Vec<u8>, // RNG内部状態
    },
}

/// ランダム選択マネージャー
pub struct RandomSelector {
    strategy: RandomStrategy,
    rng: StdRng,
    
    /// 選択キャッシュ（前方一致キーワード → 未消化リスト）
    cache: HashMap<String, Vec<String>>,
}

impl RandomSelector {
    pub fn new(strategy: RandomStrategy) -> Self {
        let rng = match &strategy {
            RandomStrategy::SystemTime => {
                StdRng::from_entropy()
            }
            RandomStrategy::FixedSeed(seed) => {
                StdRng::seed_from_u64(*seed)
            }
            RandomStrategy::SavedSeed { current_state, .. } => {
                StdRng::from_seed(current_state.as_slice().try_into().unwrap())
            }
        };
        
        Self {
            strategy,
            rng,
            cache: HashMap::new(),
        }
    }
    
    /// ランダム選択（キャッシュ考慮）
    pub fn select<'a>(
        &mut self,
        keyword: &str,
        candidates: &'a [String],
    ) -> Option<&'a String> {
        if candidates.is_empty() {
            return None;
        }
        
        // キャッシュチェック
        let cache_entry = self.cache.entry(keyword.to_string()).or_insert_with(|| {
            // 初回: シャッフルしたリストを作成
            let mut shuffled = candidates.to_vec();
            use rand::seq::SliceRandom;
            shuffled.shuffle(&mut self.rng);
            shuffled
        });
        
        // 先頭要素を消費
        if let Some(selected) = cache_entry.pop() {
            // リストが空になったらキャッシュクリア（次回は再シャッフル）
            if cache_entry.is_empty() {
                self.cache.remove(keyword);
            }
            
            // 候補リストから実際のラベルを検索
            candidates.iter().find(|&c| c == &selected)
        } else {
            None
        }
    }
    
    /// RNG状態の保存（セーブデータ用）
    pub fn save_state(&self) -> Vec<u8> {
        // StdRngの内部状態をシリアライズ
        // 注: 実際にはrune::rand::RngCore::try_fill_bytes等を使用
        // ここでは簡略化
        vec![] // TODO: 実装
    }
    
    /// RNG状態の復元
    pub fn restore_state(&mut self, state: Vec<u8>) {
        // TODO: 実装
    }
}
```

#### 環境変数・設定による制御

```rust
// crates/pasta/src/config.rs

/// Pastaエンジン設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastaConfig {
    /// ランダム選択戦略
    pub random_strategy: RandomStrategyConfig,
    
    /// デバッグモード
    pub debug_mode: bool,
    
    /// ホットリロード
    pub hot_reload: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RandomStrategyConfig {
    #[serde(rename = "system")]
    SystemTime,
    
    #[serde(rename = "fixed")]
    FixedSeed { seed: u64 },
    
    #[serde(rename = "saved")]
    SavedSeed { initial_seed: u64 },
}

impl Default for PastaConfig {
    fn default() -> Self {
        Self {
            random_strategy: if cfg!(debug_assertions) {
                // デバッグビルド: 固定シード（再現性確保）
                RandomStrategyConfig::FixedSeed { seed: 12345 }
            } else {
                // リリースビルド: システム時刻
                RandomStrategyConfig::SystemTime
            },
            debug_mode: cfg!(debug_assertions),
            hot_reload: false,
        }
    }
}

impl PastaConfig {
    /// 環境変数から読み込み
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // PASTA_RANDOM_SEED環境変数
        if let Ok(seed_str) = std::env::var("PASTA_RANDOM_SEED") {
            if let Ok(seed) = seed_str.parse::<u64>() {
                config.random_strategy = RandomStrategyConfig::FixedSeed { seed };
            }
        }
        
        config
    }
    
    /// 設定ファイルから読み込み（TOML）
    pub fn from_file(path: &Path) -> Result<Self, PastaError> {
        let content = fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}
```

#### 設定ファイル例

```toml
# pasta.toml

# ランダム選択戦略
[random_strategy]
type = "fixed"
seed = 12345

# デバッグモード
debug_mode = true

# ホットリロード（開発時のみ）
hot_reload = true
```

#### テスト例

```rust
// crates/pasta/tests/random_selection_test.rs

#[test]
fn test_random_selection_reproducibility() {
    let candidates = vec![
        "挨拶_1".to_string(),
        "挨拶_2".to_string(),
        "挨拶_3".to_string(),
    ];
    
    // 固定シードで2つのセレクターを作成
    let mut selector1 = RandomSelector::new(RandomStrategy::FixedSeed(12345));
    let mut selector2 = RandomSelector::new(RandomStrategy::FixedSeed(12345));
    
    // 同じ順序で選択されることを確認
    for _ in 0..10 {
        let result1 = selector1.select("挨拶", &candidates);
        let result2 = selector2.select("挨拶", &candidates);
        assert_eq!(result1, result2);
    }
}

#[test]
fn test_cache_exhaustion() {
    let candidates = vec![
        "挨拶_1".to_string(),
        "挨拶_2".to_string(),
    ];
    
    let mut selector = RandomSelector::new(RandomStrategy::FixedSeed(12345));
    
    // 候補数分選択
    let first = selector.select("挨拶", &candidates).unwrap();
    let second = selector.select("挨拶", &candidates).unwrap();
    
    // 両方とも異なることを確認（シャッフル済み）
    assert_ne!(first, second);
    
    // キャッシュがクリアされ、再度選択可能
    let third = selector.select("挨拶", &candidates);
    assert!(third.is_some());
}
```

### 決定事項

✅ **採用**: ハイブリッド実装（A + B + C）

**実装計画**:
1. Phase 1 (MVP): システム時刻ベース + 固定シード（デバッグビルド）
2. Phase 2: 環境変数・設定ファイル対応
3. Phase 3: セーブデータとの統合（RNG状態の永続化）

**理由**:
1. **テスト容易性**: デバッグビルドでは固定シード、自動テストが安定
2. **ユーザー体験**: リリースビルドでは真のランダム、飽きない
3. **柔軟性**: ユーザーがシードを指定可能（再現性バグ報告等）

**設定優先度**:
```
環境変数 > 設定ファイル > デフォルト値
```

---

## Summary of Design Decisions

| 議題 | 決定内容 | 実装優先度 | Phase |
|------|---------|-----------|-------|
| **1. ファイルローディング** | ハイブリッド（起動時全ロード + ホットリロード） | P0 (MVP), P1 (ホットリロード) | Phase 1, 2 |
| **2. Rune VMライフサイクル** | bevy_ecs Resource | P0 (MVP) | Phase 1 |
| **3. 変数永続化** | メモリストレージ + JSON永続化 | P0 (メモリ), P1 (永続化) | Phase 1, 2 |
| **4. エラーハンドリング** | 多層（ログ + メッセージ + UI） | P0 (ログ), P1 (メッセージ), P2 (UI) | Phase 1, 2, 3 |
| **5. さくらスクリプト** | Typewriter層での解釈 | P0 (MVP) | Phase 1 |
| **6. ランダムシード** | ハイブリッド（時刻 + 固定 + セーブ） | P0 (時刻・固定), P1 (セーブ) | Phase 1, 2 |

---

## Next Steps

設計フェーズ（`/kiro-spec-design`）で以下を詳細化する：

### Phase 1: MVP実装（P0）

1. ✅ サブクレート`pasta`のプロジェクト構造
2. ✅ パーサー実装手法の選択と文法定義（完全なPasta DSL）
3. ✅ Pasta → Rune トランスコンパイラの設計
4. ✅ Rune VM統合レイヤー
5. ✅ TypewriterToken拡張仕様
6. ✅ bevy_ecsメッセージング統合

### Phase 2: 開発体験向上（P1）

1. ホットリロード機能
2. ファイル永続化
3. エラーメッセージ通知

### Phase 3: デバッグツール（P1-P2）

1. 開発者UI（areka-P1-devtools）
2. RNG状態の永続化
3. 詳細なエラーコンテキスト

---

**文書管理**:
- 作成日: 2025-12-09
- 最終更新: 2025-12-09
- レビュー状態: 初稿完成、設計フェーズへ移行準備完了
