# Design Document - pasta-script-loader

## Overview

**Purpose**: Pastaスクリプトエンジンにディレクトリベースのスクリプト読み込み機能を追加し、areka-P0-script-engine規約（`dic/` + `main.rune`）に準拠した複数ファイル管理を実現する。

**Users**: PastaEngineを使用するアプリケーション開発者が、体系的に整理されたスクリプトディレクトリからエンジンを初期化し、宣言的な会話データ（.pasta）と手続き的なロジック（.rune）を分離して管理できる。

**Impact**: 既存の`PastaEngine::new(script: &str)`は不変のまま、新規コンストラクタ`from_directory()`を追加。ディレクトリ走査・エラー収集・Rune統合の新規コンポーネント導入。

### Goals
- areka-P0-script-engine規約に準拠したディレクトリ構造からのスクリプト読み込み
- 複数.pastaファイルの一括パース・エラー収集・ラベル統合
- main.runeを通じたRuneモジュールシステム連携
- 開発者向けの詳細エラーログ（tracingクレート経由、infoレベル）

### Non-Goals
- 相対パスサポート（絶対パスのみ）
  - **理由**: カレントディレクトリ依存を避け、セキュリティリスク（パストラバーサル）を低減
  - 将来拡張も予定なし（絶対パス運用を強制）
- ファイル探索順序の保証（ファイルシステム依存）
- ホットリロード・スクリプト再読み込み機能
  - **理由**: 状態管理の複雑さを避け、`drop(engine)` → `from_directory()`での再初期化が
            よりシンプルで安全なため。ユースケースも不明確。
- アーカイブ（.zip等）やHTTPソースからの読み込み（将来拡張）

## Architecture

### Existing Architecture Analysis

**現在のPastaEngine初期化フロー**:
```
PastaEngine::new(script: &str)
    ↓
parse_str() → AST
    ↓
Transpiler::transpile() → Rune source
    ↓
register_labels() → LabelTable
    ↓
rune::prepare() → Unit + Runtime
```

**既存コンポーネントで再利用するもの**:
- `parse_str()` / `parse_file()`: ASTパース
- `Transpiler::transpile()`: AST→Rune変換
- `register_labels()`: ラベル登録（同名ラベル連番付与）
- `PARSE_CACHE`: グローバルパースキャッシュ
- `LabelTable`: ラベル管理・ランダム選択

**統合ポイント**:
- Rune `Sources::insert()`: 複数ソース追加
- `std::error::Error`: エラートレイト実装

### Architecture Pattern & Boundary Map

```mermaid
graph TB
    subgraph "Public API"
        API1["PastaEngine::new(script)"]
        API2["PastaEngine::from_directory(path)"]
        API3["PastaEngine::from_directory_with_selector(path, selector)"]
    end
    
    subgraph "Directory Loading (新規)"
        DL[DirectoryLoader]
        LF[LoadedFiles]
        EL[ErrorLogWriter]
    end
    
    subgraph "Core Engine (既存)"
        PE[PastaEngine]
        LT[LabelTable]
        PC[ParseCache]
    end
    
    subgraph "Parser/Transpiler (既存)"
        PS[parse_str / parse_file]
        TR[Transpiler]
    end
    
    subgraph "Rune Integration (既存)"
        RS[rune::Sources]
        RC[rune::Context]
        RU[rune::Unit]
    end
    
    API1 --> PE
    API2 --> DL
    API3 --> DL
    DL --> LF
    DL --> EL
    LF --> PE
    PE --> LT
    PE --> PC
    PE --> PS
    PS --> TR
    TR --> RS
    RS --> RU
    RC --> RU
```

**Architecture Integration**:
- **Selected pattern**: Layered Architecture + Factory Pattern
- **Domain/feature boundaries**: 
  - `loader.rs` = ディレクトリ走査・ファイル収集（I/O境界）
  - `engine.rs` = パース・コンパイル・実行（ビジネスロジック）
  - `error.rs` = エラー型定義（横断関心事）
- **Existing patterns preserved**: 
  - グローバルキャッシュ（`PARSE_CACHE`）
  - ラベル登録フロー（`register_labels()`）
  - Rune統合パターン（`Sources::insert()` → `prepare()` → `build()`）
- **New components rationale**: 
  - `DirectoryLoader`: ディレクトリ走査ロジックを分離し単独テスト可能に
  - `LoadedFiles`: ファイル収集結果の型安全な受け渡し
  - `ErrorLogWriter`: エラーログ出力の責務分離
- **Steering compliance**: 
  - モジュール単位で責務を明確に分離
  - 新規ファイル`loader.rs`はPasta層に追加

**Module Organization**:
- **File Path**: `crates/pasta/src/loader.rs` (初期実装はシングルファイル、500行以下想定)
- **Public API** (`lib.rs` re-export):
  - `pub struct DirectoryLoader` — ディレクトリローダー本体
  - `pub struct LoadedFiles` — ファイル収集結果（将来のカスタムローダー実装で再利用可能）
- **Internal API** (`loader.rs`, crate-visible):
  - `pub(crate) struct ErrorLogWriter` — エラーログ出力（統合テスト・engine.rsから参照可能）
- **Refactoring Path**: 500行超過時は`loader/mod.rs`へ分割検討

**lib.rs Module Declaration and Re-export**:
```rust
// crates/pasta/src/lib.rs

// --- 既存モジュール ---
pub mod error;
mod parser;
mod transpiler;
pub mod runtime;
mod cache;

// --- 新規モジュール ---
mod loader; // ディレクトリローダー

// --- Public API Re-exports ---
pub use error::{PastaError, ParseError, Result}; // ParseError追加
pub use runtime::{LabelInfo, LabelScope, RandomSelector}; // 既存

// 新規public API
pub use loader::{DirectoryLoader, LoadedFiles};

// --- Internal Types (非公開) ---
// ErrorLogWriterはengine.rsから参照するがクレート外には公開しない
pub(crate) use loader::ErrorLogWriter;
```

**Visibility Rationale**:
- `DirectoryLoader`, `LoadedFiles`: public API
  - 外部クレートから独自のローダー実装やテストで使用可能
  - 将来の拡張: カスタムローダー(ZipLoader, HttpLoader等)が`LoadedFiles`を返す設計
- `ErrorLogWriter`: `pub(crate)` (クレート内部可視)
  - engine.rsから参照するため完全privateは不可
  - 統合テストでログ出力検証が必要なため、tests/内からアクセス可能にする
  - 外部クレートへの公開は不要(実装詳細)

### Technology Stack

| Layer | Choice / Version | Role in Feature | Notes |
|-------|------------------|-----------------|-------|
| Core Language | Rust 2021 | 型安全なディレクトリ操作 | `std::fs`, `std::path` |
| Parser | pest 2.8 | .pastaファイルパース | 既存利用 |
| Scripting | rune 0.14 | main.rune統合、mod解決 | 既存利用 |
| File Patterns | glob 0.3 | `dic/**/*.pasta`パターンマッチ | 既存依存 |
| Logging | tracing 0.1 | 警告ログ出力 | 既存利用 |
| Testing | tempfile 3 | テスト用一時ディレクトリ | dev-dependency |

## System Flows

### ディレクトリ初期化フロー

```mermaid
sequenceDiagram
    participant App as Application
    participant PE as PastaEngine
    participant DL as DirectoryLoader
    participant PS as Parser
    participant TR as Transpiler
    participant RS as Rune Sources
    
    App->>PE: from_directory(path)
    PE->>DL: load(path)
    
    Note over DL: ディレクトリ検証
    DL->>DL: validate_directory(path)
    alt 絶対パスでない
        DL-->>PE: Err(NotAbsolutePath)
        PE-->>App: Err
    else ディレクトリ存在しない
        DL-->>PE: Err(DirectoryNotFound)
        PE-->>App: Err
    end
    
    Note over DL: dic/ディレクトリ走査
    DL->>DL: collect_pasta_files(dic/)
    alt dic/不在
        DL-->>PE: Err(DicDirectoryNotFound)
        PE-->>App: Err
    end
    
    Note over DL: main.rune検証
    DL->>DL: check_main_rune()
    alt main.rune不在
        DL-->>PE: Err(MainRuneNotFound)
        PE-->>App: Err
    end
    
    DL-->>PE: Ok(LoadedFiles)
    
    Note over PE: 全.pastaファイルパース
    loop 各.pastaファイル
        PE->>PS: parse_file(path)
        PS-->>PE: Result<AST, Error>
    end
    
    alt パースエラーあり
        PE->>PE: write_error_log()
        PE-->>App: Err(MultipleParseErrors)
    end
    
    Note over PE: トランスパイル・統合
    PE->>TR: transpile_all(asts)
    TR-->>PE: rune_source
    PE->>RS: insert("entry", rune_source)
    PE->>RS: insert(main.rune)
    
    Note over PE: Runeコンパイル
    PE->>RS: prepare().build()
    alt コンパイルエラー
        PE-->>App: Err(RuneCompileError)
    end
    
    PE-->>App: Ok(PastaEngine)
```

## Requirements Traceability

| Requirement | Summary | Components | Interfaces | Flows |
|-------------|---------|------------|------------|-------|
| 1.1-1.6 | ディレクトリ初期化（絶対パス検証、fail-fast） | DirectoryLoader | `validate_directory()` | 初期化フロー Step 1-2 |
| 2.1-2.13 | ファイル配置規則（dic/再帰、main.rune） | DirectoryLoader | `collect_pasta_files()`, `check_main_rune()` | 初期化フロー Step 3-4 |
| 3.1-3.12 | スクリプト読み込み（パース、エラー収集） | PastaEngine | `parse_all_files()`, `write_error_log()` | 初期化フロー Step 5-7 |
| 4.1-4.7 | ラベル名前空間（連番、ランダム選択） | LabelTable | `register_labels()` | 既存フロー（不変） |
| 5.1-5.8 | テストフィクスチャ | tests/fixtures/test-project/ | - | - |
| 6.1-6.8 | 統合テスト | directory_loader_test.rs | - | - |
| 7.1-7.10 | エラーハンドリング | PastaError | エラー型定義 | 全フロー |
| 8.1-8.5 | パフォーマンス（キャッシュ） | ParseCache | `global_cache()` | 初期化フロー Step 5 |
| 9.1-9.6 | API設計 | PastaEngine | Public API | - |

## Components and Interfaces

| Component | Domain/Layer | Intent | Req Coverage | Key Dependencies | Contracts |
|-----------|--------------|--------|--------------|------------------|-----------|
| DirectoryLoader | I/O Layer (新規) | ディレクトリ走査・ファイル収集 | 1, 2 | std::fs, glob (P0) | Service |
| LoadedFiles | I/O Layer (新規) | ファイル収集結果の型安全な受け渡し | 2 | - | State |
| ErrorLogWriter | I/O Layer (新規) | エラーログ出力 | 3.5, 3.6, 7.10 | std::fs (P0) | Service |
| PastaEngine | Core Layer (拡張) | スクリプトコンパイル・実行 | 1, 3, 4, 8, 9 | DirectoryLoader (P0), LabelTable (P0), Rune (P0) | Service, API |
| PastaError | Cross-cutting (拡張) | エラー型定義 | 7 | thiserror (P0) | State |

### I/O Layer

#### DirectoryLoader

| Field | Detail |
|-------|--------|
| Intent | areka規約に準拠したディレクトリ走査とファイル収集 |
| Requirements | 1.1-1.6, 2.1-2.13 |

**Responsibilities & Constraints**
- スクリプトルートディレクトリの検証（絶対パス、存在、読み取り権限）
- `dic/`サブディレクトリの再帰走査と.pastaファイル収集
- ファイルフィルタリング（`_`プレフィックス、隠しファイル除外）
- `main.rune`存在確認

**Dependencies**
- Outbound: std::fs — ファイルシステム操作 (P0)
- Outbound: glob — パターンマッチ (P1)
- Outbound: tracing — 警告ログ (P2)

**Contracts**: Service [x] / API [ ] / Event [ ] / Batch [ ] / State [ ]

##### Service Interface
```rust
/// ディレクトリローダー（ファイル収集専用）
pub struct DirectoryLoader;

impl DirectoryLoader {
    /// スクリプトディレクトリを読み込み、ファイル一覧を返す
    /// 
    /// # Errors
    /// - `NotAbsolutePath`: 相対パスが指定された場合
    /// - `DirectoryNotFound`: ディレクトリが存在しない場合
    /// - `NotADirectory`: パスがファイルの場合
    /// - `PermissionDenied`: 読み取り権限がない場合
    /// - `DicDirectoryNotFound`: dic/ディレクトリが存在しない場合
    /// - `MainRuneNotFound`: main.runeが存在しない場合
    pub fn load(script_root: &Path) -> Result<LoadedFiles>;
}
```
- Preconditions: `script_root`は絶対パスであること
- Postconditions: `LoadedFiles`に有効なファイルパスが格納される
- Invariants: `main_rune`パスは必ず存在するファイルを指す

#### LoadedFiles

| Field | Detail |
|-------|--------|
| Intent | ディレクトリ走査結果の型安全な受け渡し |
| Requirements | 2.1-2.4 |

**Contracts**: State [x]

##### State Management
```rust
/// ディレクトリ走査結果
pub struct LoadedFiles {
    /// スクリプトルートディレクトリ（絶対パス）
    pub script_root: PathBuf,
    /// 収集された.pastaファイルパス一覧
    pub pasta_files: Vec<PathBuf>,
    /// main.runeファイルパス
    pub main_rune: PathBuf,
}
```
- State model: 不変データ構造（構築後は変更不可）
- Persistence: なし（メモリ内のみ）

#### ErrorLogWriter

| Field | Detail |
|-------|--------|
| Intent | パースエラーのログ出力（tracingクレート経由） |
| Requirements | 3.5, 3.6, 7.10 |

**Contracts**: Service [x]

##### Service Interface
```rust
/// エラーログライター
pub(crate) struct ErrorLogWriter;

impl ErrorLogWriter {
    /// エラー情報をtracing経由でログ出力
    /// 
    /// # Arguments
    /// * `script_root` - スクリプトルートディレクトリ
    /// * `errors` - 出力するエラー一覧
    /// 
    /// # Logging Strategy
    /// - デフォルト: infoレベルで出力
    /// - 各エラーは個別にtracing::info!("パースエラー: {}:{} - {}", file, line, message)
    /// - ファイル出力なし（tracing subscriberがログ先を制御）
    pub fn log(script_root: &Path, errors: &[ParseError]);
}
```

##### Log Output Format (tracing)
```
[INFO] パースエラー: dic/greetings.pasta:10:5 - Expected ':' after speaker name, found '@'
[INFO] パースエラー: dic/events.pasta:25:12 - Undefined label reference: ＊挨拶_削除済み
[INFO] 合計 2 件のパースエラー (script_root: /path/to/project)
```

**Design Rationale**:
- **tracingクレート活用**: 既存のログインフラを再利用
- **ファイル出力削除**: pasta_errors.log生成を廃止、ログsubscriberが出力先を制御
- **基礎実装**: 将来の要件追加に対応できるシンプルな設計
- **pub(crate)**: クレート内部のみ可視（engine.rsから参照）

### Core Layer

#### PastaEngine (拡張)

| Field | Detail |
|-------|--------|
| Intent | Pastaスクリプトのコンパイルと実行 |
| Requirements | 1, 3, 4, 8, 9 |

**Responsibilities & Constraints**
- ディレクトリベース初期化（新規）
- 複数.pastaファイルの一括パース・エラー収集（新規）
- AST→Runeトランスパイル
- ラベルテーブル構築
- Runeコンパイル・実行

**Dependencies**
- Inbound: Application — 初期化要求 (P0)
- Outbound: DirectoryLoader — ファイル収集 (P0)
- Outbound: Parser — ASTパース (P0)
- Outbound: Transpiler — Rune変換 (P0)
- Outbound: LabelTable — ラベル管理 (P0)
- Outbound: ParseCache — キャッシュ (P1)
- External: rune — スクリプト実行 (P0)

**Contracts**: Service [x] / API [x]

##### API Contract

| Method | Signature | Description | Errors |
|--------|-----------|-------------|--------|
| `from_directory` | `(path: impl AsRef<Path>) -> Result<Self>` | ディレクトリから初期化 | 1.1-1.6, 2.10-2.11, 3.7-3.8, 3.12, 7.1-7.7 |
| `from_directory_with_selector` | `(path: impl AsRef<Path>, selector: Box<dyn RandomSelector>) -> Result<Self>` | カスタムセレクタで初期化 | 同上 |
| `list_labels` | `(&self) -> Vec<String>` | 全ラベル名列挙 | - |
| `list_global_labels` | `(&self) -> Vec<String>` | グローバルラベルのみ列挙 | - |

##### Struct Definition
```rust
/// Pastaスクリプトエンジン
pub struct PastaEngine {
    /// コンパイル済みRuneユニット
    unit: Arc<rune::Unit>,
    /// Runeランタイムコンテキスト
    runtime: Arc<rune::runtime::RuntimeContext>,
    /// ラベルテーブル（ラベル検索・ランダム選択）
    label_table: LabelTable,
}
```

**Field Rationale**:
- `unit`, `runtime`, `label_table`: 既存フィールド（不変）
- 注: 以前の設計では`script_root: Option<PathBuf>`を含めていたが、
  `reload_directory()`削除により不要となったため削除

##### Service Interface
```rust
impl PastaEngine {
    /// ディレクトリからPastaEngineを初期化
    /// 
    /// # Arguments
    /// * `path` - スクリプトルートディレクトリ（絶対パス）
    /// 
    /// # Errors
    /// - ディレクトリ関連: NotAbsolutePath, DirectoryNotFound, NotADirectory, PermissionDenied
    /// - ファイル関連: DicDirectoryNotFound, MainRuneNotFound
    /// - パース関連: MultipleParseErrors, IoError
    /// - コンパイル関連: RuneCompileError
    pub fn from_directory(path: impl AsRef<Path>) -> Result<Self>;
    
    /// カスタムRandomSelectorでディレクトリから初期化
    pub fn from_directory_with_selector(
        path: impl AsRef<Path>,
        selector: Box<dyn RandomSelector>,
    ) -> Result<Self>;
    
    /// 全ラベル名を列挙（グローバル + ローカル）
    pub fn list_labels(&self) -> Vec<String>;
    
    /// グローバルラベルのみ列挙
    pub fn list_global_labels(&self) -> Vec<String>;
}
```
- Preconditions: `path`は絶対パスで、有効なスクリプトディレクトリ構造を持つ
- Postconditions: 初期化成功時、全.pastaファイルのラベルが登録される
- Invariants: `label_table`は常に有効なラベル情報を保持

**Implementation Notes**
- Integration: `from_directory()`は内部で`DirectoryLoader::load()`を呼び出し
- Validation: `path.is_absolute()`で絶対パスチェック

**from_directory() Implementation Algorithm**:
```rust
pub fn from_directory(path: impl AsRef<Path>) -> Result<Self> {
    let path = path.as_ref();
    
    // Step 1: ディレクトリ検証 (Fail-Fast)
    let loaded = DirectoryLoader::load(path)?; // NotAbsolutePath, DirectoryNotFound, etc.
    
    // Step 2: 全.pastaファイルパース (エラー収集)
    let mut asts = Vec::new();
    let mut parse_errors = Vec::new();
    
    for pasta_file in &loaded.pasta_files {
        match parse_file(pasta_file, pasta_file.to_string_lossy().as_ref()) {
            Ok(ast) => asts.push(ast),
            Err(e) => {
                // ParseError variantのみ収集、他のエラーは即座に返却
                if let Some(parse_err) = Option::<ParseError>::from(&e) {
                    parse_errors.push(parse_err);
                } else {
                    return Err(e); // IoError等はfail-fast
                }
            }
        }
    }
    
    // Step 3: パースエラーがあればログ出力して即座に返却
    if !parse_errors.is_empty() {
        ErrorLogWriter::log(&loaded.script_root, &parse_errors);
        return Err(PastaError::MultipleParseErrors { errors: parse_errors });
    }
    
    // Step 4: 全AST統合とトランスパイル
    let merged_ast = merge_asts(asts); // 全ラベルをマージ
    let rune_source = Transpiler::transpile(&merged_ast)?;
    
    // Step 5: Rune Sources構築
    let mut sources = rune::Sources::new();
    sources.insert(rune::Source::new("entry", rune_source))?;
    sources.insert(rune::Source::from_path(&loaded.main_rune)?)?;
    
    // Step 6: Runeコンパイル (Fail-Fast)
    let mut context = rune::Context::with_default_modules()?;
    let mut diagnostics = rune::Diagnostics::new();
    let unit = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build()?; // RuneCompileError
    
    let runtime = Arc::new(context.runtime()?);
    
    // Step 7: LabelTable構築
    let label_table = LabelTable::from_labels(
        merged_ast.labels,
        Box::new(DefaultRandomSelector::new())
    );
    
    Ok(Self {
        unit: Arc::new(unit),
        runtime,
        label_table,
    })
}
```

**Error Handling Strategy**:
- **Fail-Fast**: ディレクトリ検証(Step 1)、Runeコンパイル(Step 6)
- **Error Collection**: .pastaパース(Step 2-3) - 全ファイル処理後に集約
- **Immediate Return**: IoError, RuneCompileError等の致命的エラー

**merge_asts() Implementation Details**:
```rust
/// 複数ASTを単一ASTに統合
/// 
/// 全ファイルのラベルを1つのVecに結合し、register_labels()で
/// 一括処理することで、同名ラベルの関数名が全ファイル間でユニークになる。
fn merge_asts(asts: Vec<Ast>) -> Ast {
    let mut merged_labels = Vec::new();
    for ast in asts {
        merged_labels.extend(ast.labels);
    }
    Ast { labels: merged_labels }
}
```
- **重要**: ファイルごとに`register_labels()`を呼ぶと、カウンターがリセットされ、
  同名ラベルの関数名が重複する（例: 複数ファイルで`挨拶_0`が生成される）
- **採用理由**: 全ラベル統合後に1回だけ`register_labels()`を呼ぶことで、
  カウンターが連続し、関数名の一意性を保証（`挨拶_0`, `挨拶_1`, `挨拶_2`...）

### Cross-cutting

#### PastaError (拡張)

| Field | Detail |
|-------|--------|
| Intent | Pastaエンジンのエラー型定義 |
| Requirements | 7.1-7.10 |

**Contracts**: State [x]

##### 新規エラー型
```rust
#[derive(Error, Debug)]
pub enum PastaError {
    // --- 既存エラー（不変）---
    #[error("Parse error at {file}:{line}:{column}: {message}")]
    ParseError { file: String, line: usize, column: usize, message: String },
    
    #[error("Label not found: {label}")]
    LabelNotFound { label: String },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Rune compile error: {0}")]
    RuneCompileError(String),
    
    // ... 他の既存エラー
    
    // --- 新規エラー ---
    /// 指定されたパスが絶対パスでない
    #[error("Path must be absolute: {path}")]
    NotAbsolutePath { path: String },
    
    /// 指定されたディレクトリが存在しない
    #[error("Directory not found: {path}")]
    DirectoryNotFound { path: String },
    
    /// 指定されたパスがディレクトリではない
    #[error("Path is not a directory: {path}")]
    NotADirectory { path: String },
    
    /// ディレクトリの読み取り権限がない
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },
    
    /// dic/ディレクトリが存在しない
    #[error("dic/ directory not found in: {script_root}")]
    DicDirectoryNotFound { script_root: String },
    
    /// main.runeが存在しない
    #[error("main.rune not found in: {script_root}")]
    MainRuneNotFound { script_root: String },
    
    /// 複数のパースエラーが発生
    #[error("Multiple parse errors ({} errors). See pasta_errors.log for details.", .errors.len())]
    MultipleParseErrors { 
        /// パースエラー一覧（ParseError限定で循環参照を防止）
        errors: Vec<ParseError> 
    },
}
```

**Error Type Design Rationale**:
- **`count`フィールド削除**: `.errors.len()`で動的に取得可能、冗長性排除
- **`errors`型を限定**: `Vec<ParseError>`構造体に変更（`Vec<PastaError>`から）
  - 循環参照防止: `MultipleParseErrors`が自身を含むネストを回避
  - 意味的整合性: 複数パースエラー集約は構文エラーのみを対象とすべき
- **`ParseError`構造体定義** (error.rs内、public API):
  ```rust
  /// 個別パースエラー（MultipleParseErrors内で使用）
  #[derive(Debug, Clone, PartialEq)]
  pub struct ParseError {
      pub file: String,
      pub line: usize,
      pub column: usize,
      pub message: String,
  }
  ```
  - **配置**: `crates/pasta/src/error.rs` (PastaError enumと同じファイル)
  - **可視性**: `pub` (テスト・デバッグ時に個別エラー検証が必要なため)
  - **命名の整合性**: `PastaError::ParseError` variantとは異なる型
    - variant: 単一パースエラー（既存、`{file, line, column, message}`フィールド）
    - struct: 複数エラー集約用（新規、MultipleParseErrors内で使用）
  - **変換実装**: `From<PastaError> for ParseError`を実装し、variant→structへの変換を提供
    ```rust
    impl From<&PastaError> for Option<ParseError> {
        fn from(e: &PastaError) -> Self {
            match e {
                PastaError::ParseError { file, line, column, message } => {
                    Some(ParseError {
                        file: file.clone(),
                        line: *line,
                        column: *column,
                        message: message.clone(),
                    })
                }
                _ => None,
            }
        }
    }
    ```
- **`std::error::Error::source()`実装**: 
  - `MultipleParseErrors`では`source()`は`None`を返す
  - 詳細エラー情報は`pasta_errors.log`で確認（個別エラーの列挙は冗長）

```rust
```

## Data Models

### Domain Model

**集約ルート**: `PastaEngine`
- エンティティ: `LabelInfo`（ラベル情報）
- 値オブジェクト: `LoadedFiles`（ファイル収集結果）
- ドメインイベント: なし（同期処理）

**ビジネスルール**:
- 同名グローバルラベルは連番付与（`挨拶_0`, `挨拶_1`）
- ローカルラベルは親グローバルラベルにスコープ限定
- ランタイムの同名ラベル呼び出しはランダム選択

### Logical Data Model

```mermaid
erDiagram
    PastaEngine ||--|| LabelTable : owns
    PastaEngine ||--o| PathBuf : "script_root (optional)"
    LabelTable ||--|{ LabelInfo : contains
    LabelInfo }|--|| LabelScope : has
    LabelInfo }o--o| String : "parent (for local)"
    
    PastaEngine {
        Arc_Unit unit
        Arc_RuntimeContext runtime
        LabelTable label_table
        Option_PathBuf script_root
    }
    
    LabelTable {
        HashMap_Vec_LabelInfo labels
        Box_dyn_RandomSelector random_selector
    }
    
    LabelInfo {
        String name
        LabelScope scope
        HashMap_String_String attributes
        String fn_name
        Option_String parent
    }
```

## Error Handling

### Error Strategy
- **Fail Fast**: ディレクトリ検証は即座にエラー返却（1.3-1.6）
- **Error Collection**: パースエラーは全ファイル処理後に集約（3.4-3.7）
- **Graceful Degradation**: 空の.pastaファイルは警告のみ（2.13）

### Error Categories and Responses

**User Errors (Configuration)**:
- `NotAbsolutePath` → 絶対パスで再指定を案内
- `DirectoryNotFound` → パス確認を案内
- `DicDirectoryNotFound` → ディレクトリ構造の確認を案内
- `MainRuneNotFound` → main.rune作成を案内

**System Errors (I/O)**:
- `IoError` → 即座にエラー返却、リトライは呼び出し元の責務
- `PermissionDenied` → 権限確認を案内

**Business Logic Errors (Parse/Compile)**:
- `MultipleParseErrors` → `pasta_errors.log`参照を案内
- `RuneCompileError` → Runeエラーメッセージをそのまま表示

### Monitoring
- 警告ログ（tracing）: 空の.pastaファイル検出時、.pastaファイル0件時
- エラーログ（ファイル）: `pasta_errors.log` にパースエラー詳細出力

## Testing Strategy

### Unit Tests
- `DirectoryLoader::load()` - 各エラーケース（NotAbsolutePath, DirectoryNotFound等）
- `DirectoryLoader::collect_pasta_files()` - フィルタリング（_prefix, hidden files）
- `ErrorLogWriter::write()` - ログフォーマット検証
- `PastaError` variants - エラーメッセージフォーマット

### Integration Tests
- `from_directory()` 正常系 - テストフィクスチャから初期化成功
- `from_directory()` 複数ファイル - 全ラベル統合確認
- `from_directory()` エラー収集 - 複数パースエラー時のログ出力
- `reload_directory()` - ファイル変更反映
- main.rune統合 - Runeモジュール参照解決

### E2E Tests
- テストフィクスチャ（`tests/fixtures/test-project/`）からの完全初期化
- ラベル実行（`execute_label()`）の動作確認
- エラーケース網羅（Req 6 AC 1-8）

## Security Considerations

- **パストラバーサル防止**: 絶対パス強制により、意図しないディレクトリアクセスを防止
- **権限チェック**: 読み取り権限検証を初期化時に実施（1.5）
- **ログファイル配置**: スクリプトルート内に限定（任意パスへの書き込み防止）

## Performance & Scalability

**Target Metrics**:
- 100ファイル読み込み: < 1秒（コールドスタート）
- キャッシュヒット時: < 100ms（再初期化）

**Optimization**:
- `PARSE_CACHE`による重複パース回避（8.1-8.2）
- 遅延ディレクトリエントリ評価（8.3）
- 最小限のmetadata()呼び出し（8.4）
- デバッグビルドでキャッシュヒット/ミスログ（8.5）

## Supporting References

### テストフィクスチャ構造
```
tests/fixtures/test-project/
├── main.rune              # 最小限のRune実装
└── dic/
    ├── greetings.pasta    # 基本会話（挨拶ラベル×3）
    ├── sakura_script.pasta # さくらスクリプト統合
    ├── variables.pasta    # 変数操作
    ├── special/
    │   └── holiday.pasta  # サブディレクトリテスト
    └── _ignored.pasta     # スキップ対象テスト
```

### 関連ドキュメント
- [research.md](./research.md) - 詳細な調査結果とトレードオフ分析
- [gap-analysis.md](./gap-analysis.md) - 既存コードベースとの差分分析
- [requirements.md](./requirements.md) - EARS形式の要件定義
