# Technical Design Document

## Overview

**Purpose**: PastaShioriのload関数にpasta_luaエンジン統合を実装し、SHIORI DLLとしてのランタイム初期化機能を提供する。

**Users**: 「伺か」ゴースト開発者がPastaスクリプトエンジンを使用してゴーストを作成する際に、このload実装がエンジン初期化を担当する。

**Impact**: 既存のPastaShiori構造体を拡張し、PastaLoader::load()統合とロギング機能を追加する。

### Goals
- PastaShiori::load()でPastaLuaRuntimeを初期化・保持する
- pasta.toml必須化によるエラー検出の強化
- 各ゴーストインスタンスごとの独立ログファイル出力

### Non-Goals
- request()関数の完全実装（本仕様はload()のみ）
- pasta_runeバックエンドの統合（将来の別仕様）
- ログレベルやフォーマットのカスタマイズ（将来拡張）

---

## Architecture

### Existing Architecture Analysis

**現行のpasta_shiori構造**:
- `RawShiori<T>` — OnceLockシングルトン、Arc<Mutex<Option<T>>>パターンでライフサイクル管理
- `PastaShiori` — Shioriトレイト実装、現在はhinst/load_dirのみ保持
- `MyError` — エラー型、to_shiori_response()でSHIORI 500レスポンス生成

**既存の制約**:
- Windows FFI境界（C ABI互換）
- スレッドセーフティ必須（複数ゴースト同時ロード可能）
- SHIORI規約（load → request → unloadライフサイクル）

### Architecture Pattern & Boundary Map

```mermaid
graph TB
    subgraph pasta_shiori["pasta_shiori (cdylib)"]
        DllMain["DllMain<br/>OnceLock初期化"]
        RawShiori["RawShiori&lt;PastaShiori&gt;<br/>Arc&lt;Mutex&lt;Option&lt;T&gt;&gt;&gt;"]
        PastaShiori["PastaShiori<br/>hinst, load_dir, runtime, logger"]
        MyError["MyError<br/>From&lt;LoaderError&gt;"]
    end

    subgraph pasta_lua["pasta_lua"]
        PastaLoader["PastaLoader::load()<br/>統合API"]
        PastaConfig["PastaConfig<br/>[loader], [logging]"]
        LoggingConfig["LoggingConfig<br/>file_path, rotation_days"]
        PastaLuaRuntime["PastaLuaRuntime<br/>Lua VMホスト"]
        PastaLogger["PastaLogger<br/>ファイル出力・ローテーション"]
    end

    subgraph ukagaka["伺か ベースウェア"]
        Shell["ベースシェル<br/>SHIORI呼び出し"]
    end

    Shell -->|"load(hinst, load_dir)"| DllMain
    DllMain --> RawShiori
    RawShiori --> PastaShiori
    PastaShiori -->|"PastaLoader::load(load_dir)"| PastaLoader
    PastaLoader --> PastaConfig
    PastaConfig --> LoggingConfig
    PastaLoader -->|"初期化"| PastaLuaRuntime
    PastaLoader -->|"ロガー生成"| PastaLogger
    PastaLoader -.->|"エラー変換"| MyError
    PastaShiori -->|"保持"| PastaLuaRuntime
    PastaShiori -->|"保持"| PastaLogger
```

**Architecture Integration**:
- **Selected pattern**: 既存コンポーネント拡張（Option A）
- **Domain boundaries**: pasta_shiori（SHIORI FFI層）、pasta_lua（ランタイム層）
- **Existing patterns preserved**: Arc<Mutex<Option<T>>>、OnceLock、MyError変換
- **New components rationale**: 
  - LoggingConfig: [logging]セクション読み込み
  - PastaLogger: インスタンスごとのファイル出力
- **Steering compliance**: Rust 2024 edition、thiserror/tracing使用

### Technology Stack

| Layer | Choice / Version | Role in Feature | Notes |
|-------|------------------|-----------------|-------|
| Backend | Rust 2024 edition | コア実装 | Cargo.toml `edition = "2024"` |
| Runtime | pasta_lua (mlua 0.11) | Lua VMホスト | PastaLuaRuntime |
| Logging | tracing 0.1, tracing-subscriber 0.3, tracing-appender 0.2 | ログ出力 | 新規依存 |
| Configuration | toml 0.9.8, serde 1 | pasta.toml解析 | 既存依存 |
| FFI | windows-sys 0.59 | Windows DLL | cfg(windows)ゲート |

---

## System Flows

### load()シーケンス

```mermaid
sequenceDiagram
    participant Shell as ベースシェル
    participant DllMain
    participant RawShiori
    participant PastaShiori
    participant PastaLoader
    participant PastaConfig
    participant PastaLogger
    participant PastaLuaRuntime

    Shell->>DllMain: load(hinst, load_dir)
    DllMain->>RawShiori: acquire_lock()
    RawShiori->>PastaShiori: load(hinst, load_dir)
    
    PastaShiori->>PastaShiori: validate load_dir exists
    alt load_dir不存在
        PastaShiori-->>Shell: false (DirectoryNotFound)
    end
    
    PastaShiori->>PastaLoader: load(load_dir)
    PastaLoader->>PastaConfig: load(load_dir)
    
    alt pasta.toml不存在
        PastaConfig-->>PastaLoader: Err(ConfigNotFound)
        PastaLoader-->>PastaShiori: Err(LoaderError)
        PastaShiori-->>Shell: false (ConfigNotFound)
    end
    
    PastaConfig-->>PastaLoader: Ok(config)
    
    PastaLoader->>PastaLogger: new(load_dir, logging_config)
    PastaLogger-->>PastaLoader: logger
    
    PastaLoader->>PastaLoader: discover & transpile
    PastaLoader->>PastaLuaRuntime: initialize
    PastaLuaRuntime-->>PastaLoader: runtime
    
    PastaLoader-->>PastaShiori: Ok((runtime, logger))
    PastaShiori->>PastaShiori: store runtime, logger
    PastaShiori-->>Shell: true
```

**Key Decisions**:
- load_dir存在確認はPastaShiori側で実施（FFI境界での早期エラー検出）
- PastaLoader::load()がログローテーション込みの初期化を完結
- エラー発生時はMyError::Load(String)に変換してfalse返却

---

## Requirements Traceability

| Requirement | Summary | Components | Interfaces | Flows |
|-------------|---------|------------|------------|-------|
| 1.1, 1.2, 1.3 | Runtime初期化・保持 | PastaShiori, PastaLoader | Shiori::load() | load()シーケンス |
| 2.1, 2.2, 2.3 | load_dirパス処理 | PastaShiori | PathBuf変換 | load()シーケンス |
| 3.1, 3.2, 3.3, 3.4 | エラーハンドリング | MyError, LoaderError | From<LoaderError> | load()シーケンス |
| 4.1, 4.2 | pasta.toml必須化 | PastaConfig | LoaderError::ConfigNotFound | load()シーケンス |
| 5.1, 5.2 | hinst保持 | PastaShiori | isize field | - |
| 6.1, 6.2, 6.3 | Runtime状態管理 | PastaShiori | Option<PastaLuaRuntime> | - |
| 7.1-7.8 | ロギング機能 | LoggingConfig, PastaLogger | logging()メソッド | load()シーケンス |

---

## Components and Interfaces

| Component | Domain/Layer | Intent | Req Coverage | Key Dependencies | Contracts |
|-----------|--------------|--------|--------------|------------------|-----------|
| PastaShiori | pasta_shiori/SHIORI | SHIORIライフサイクル管理 | 1, 2, 3, 5, 6 | PastaLoader (P0), MyError (P0) | Service |
| MyError | pasta_shiori/Error | エラー型変換 | 3 | LoaderError (P0) | Service |
| PastaConfig | pasta_lua/Config | pasta.toml解析 | 4, 7.2 | toml (P1), serde (P1) | Service |
| LoggingConfig | pasta_lua/Config | [logging]セクション | 7.2, 7.3, 7.4 | serde (P1) | State |
| PastaLogger | pasta_lua/Logging | ファイル出力・ローテーション | 7.1, 7.3-7.8 | tracing-appender (P0) | Service |
| PastaLoader | pasta_lua/Loader | 統合初期化API | 1, 7.1 | PastaConfig (P0), PastaLogger (P0) | Service |

---

### pasta_shiori Layer

#### PastaShiori

| Field | Detail |
|-------|--------|
| Intent | SHIORIライフサイクル管理とpasta_lua統合 |
| Requirements | 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 5.1, 5.2, 6.1, 6.2, 6.3 |

**Responsibilities & Constraints**
- SHIORI load/request/unloadライフサイクルの管理
- PastaLuaRuntimeインスタンスの保持と状態追跡
- FFI境界でのエラー変換とログ出力

**Dependencies**
- Outbound: PastaLoader — ランタイム初期化 (P0)
- Outbound: MyError — エラー変換 (P0)
- External: tracing — ログ出力 (P1)

**Contracts**: Service [x]

##### Service Interface

```rust
pub(crate) trait Shiori {
    fn load<S: AsRef<OsStr>>(&mut self, hinst: isize, load_dir: S) -> MyResult<bool>;
    fn request<S: AsRef<str>>(&mut self, request: S) -> MyResult<String>;
}

#[derive(Default)]
pub(crate) struct PastaShiori {
    hinst: isize,
    load_dir: Option<PathBuf>,
    runtime: Option<PastaLuaRuntime>,
    logger: Option<PastaLogger>,
}
```

- **Preconditions**: load_dirはOsStr形式、hinst有効なHINSTANCE
- **Postconditions**: 成功時runtime=Some、失敗時runtime=None
- **Invariants**: load()成功後のみrequest()が有効

**Implementation Notes**
- Integration: PastaLoader::load()呼び出しでランタイムとロガーを取得
- Validation: load_dir存在確認、pasta.toml存在確認（PastaLoader経由）
- Risks: 複数回load()呼び出し時の既存ランタイム解放順序

---

#### MyError

| Field | Detail |
|-------|--------|
| Intent | LoaderErrorからの変換とSHIORIレスポンス生成 |
| Requirements | 3.1, 3.2, 3.3, 3.4 |

**Contracts**: Service [x]

##### Service Interface

```rust
#[derive(Clone, Eq, PartialEq, Debug, Error)]
pub enum MyError {
    #[error("load error: {0}")]
    Load(String),
    
    #[error("not initialized error")]
    NotInitialized,
    
    // ... 他のバリアント
}

impl From<LoaderError> for MyError {
    fn from(e: LoaderError) -> Self {
        MyError::Load(format!("{}", e))
    }
}
```

- **Preconditions**: LoaderErrorが有効
- **Postconditions**: MyError::Load(String)に変換
- **Invariants**: エラーメッセージにデバッグ情報を含む

---

### pasta_lua Layer

#### LoggingConfig

| Field | Detail |
|-------|--------|
| Intent | [logging]セクションのデシリアライズ |
| Requirements | 7.2, 7.3, 7.4 |

**Contracts**: State [x]

##### State Management

```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LoggingConfig {
    /// ログファイルパス（load_dirからの相対パス）
    /// デフォルト: "profile/pasta/logs/pasta.log"
    #[serde(default = "default_file_path")]
    pub file_path: String,
    
    /// ログローテーション日数
    /// デフォルト: 7
    #[serde(default = "default_rotation_days")]
    pub rotation_days: usize,
}

fn default_file_path() -> String {
    "profile/pasta/logs/pasta.log".to_string()
}

fn default_rotation_days() -> usize {
    7
}
```

- **State model**: イミュータブル設定値
- **Persistence**: pasta.tomlから読み込み
- **Concurrency strategy**: Clone可能、スレッドセーフ

---

#### PastaLogger

| Field | Detail |
|-------|--------|
| Intent | インスタンスごとのファイルログ出力とローテーション |
| Requirements | 7.1, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8 |

**Responsibilities & Constraints**
- 各PastaLoaderインスタンスが独立したログファイルを持つ
- profile/pasta/logs/ディレクトリへの自動作成
- N日間のログローテーション

**Dependencies**
- External: tracing-appender — ローテーション処理 (P0)
- External: tracing-subscriber — フォーマット処理 (P1)

**Contracts**: Service [x]

##### Service Interface

```rust
pub struct PastaLogger {
    /// ログファイルの絶対パス
    log_path: PathBuf,
    
    /// ローテーション設定（日数）
    rotation_days: usize,
    
    /// 非同期書き込みガード（Dropで自動フラッシュ）
    _guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

impl PastaLogger {
    /// ロガー生成（[logging]セクション設定に基づく）
    pub fn new(base_dir: &Path, config: Option<&LoggingConfig>) -> Result<Self, LoaderError>;
    
    /// ログ出力が有効かどうか
    pub fn is_enabled(&self) -> bool;
}

impl Drop for PastaLogger {
    fn drop(&mut self) {
        // WorkerGuardのDropでログをフラッシュ
    }
}
```

- **Preconditions**: base_dirが有効なパス
- **Postconditions**: ログディレクトリが作成され、ファイルハンドルが確保
- **Invariants**: profile配下にのみファイルを作成

**Implementation Notes**
- Integration: PastaLoader::load()内で生成、PastaShioriに所有権移転
- Validation: パス正規化、ディレクトリ作成権限確認
- Risks: ファイル書き込み権限エラー時のフォールバック（ログ無効化）

---

#### PastaConfig (拡張)

| Field | Detail |
|-------|--------|
| Intent | [logging]セクションの読み込み追加 |
| Requirements | 7.2 |

**Contracts**: Service [x]

##### Service Interface (追加メソッド)

```rust
impl PastaConfig {
    /// [logging]セクションを取得（存在しない場合None）
    pub fn logging(&self) -> Option<LoggingConfig> {
        self.custom_fields
            .get("logging")
            .and_then(|v| v.clone().try_into().ok())
    }
}
```

- **Preconditions**: PastaConfig::load()成功
- **Postconditions**: [logging]存在時はSome、なければNone
- **Invariants**: custom_fieldsから安全にデシリアライズ

---

#### PastaLoader (拡張)

| Field | Detail |
|-------|--------|
| Intent | ロガー初期化をload()シーケンスに統合 |
| Requirements | 7.1 |

**Contracts**: Service [x]

##### Service Interface (シグネチャ変更)

```rust
impl PastaLoader {
    /// ロガー付きでランタイムをロード
    pub fn load(base_dir: impl AsRef<Path>) -> Result<(PastaLuaRuntime, Option<PastaLogger>), LoaderError>;
    
    /// 従来互換API（ロガーなし）
    pub fn load_runtime_only(base_dir: impl AsRef<Path>) -> Result<PastaLuaRuntime, LoaderError>;
}
```

**Implementation Notes**
- Integration: Phase 0.5としてロガー初期化を追加
- Validation: [logging]セクションの有無でロガー生成判断
- Risks: API変更による既存テストへの影響（load_runtime_only()で互換維持）

---

## Data Models

### Domain Model

**Aggregates**:
- `PastaShiori` — SHIORIライフサイクルのルートエンティティ
- `PastaLuaRuntime` — Lua VMライフサイクルのルートエンティティ

**Entities**:
- `PastaLogger` — ログファイルライフサイクル管理

**Value Objects**:
- `LoggingConfig` — 不変の設定値
- `LoaderConfig` — 不変の設定値

### Logical Data Model

**pasta.toml構造**:

```toml
[loader]
pasta_patterns = ["dic/*/*.pasta"]
lua_search_paths = ["scripts", "scripts/pasta"]
transpiled_output_dir = "profile/pasta/cache/lua"
debug_mode = true

[logging]
file_path = "profile/pasta/logs/pasta.log"
rotation_days = 7

# カスタムフィールド（ゴースト固有設定）
ghost_name = "パスタちゃん"
version = "1.0.0"
```

**LoggingConfig → ファイル出力マッピング**:

| 設定 | デフォルト | 説明 |
|------|-----------|------|
| file_path | profile/pasta/logs/pasta.log | load_dirからの相対パス |
| rotation_days | 7 | ログ保持日数（max_log_filesにマッピング） |

---

## Error Handling

### Error Strategy

LoaderErrorからMyErrorへの変換で、SHIORI 500レスポンスを生成。

### Error Categories and Responses

| Error Source | MyError Variant | X-ERROR-REASON | Recovery |
|--------------|-----------------|----------------|----------|
| load_dir不存在 | Load("Directory not found: ...") | パス情報 | ゴースト配置確認 |
| pasta.toml不存在 | Load("Config not found: ...") | パス情報 | pasta.toml作成 |
| pasta.tomlパースエラー | Load("Config error: ...") | 詳細メッセージ | TOML構文修正 |
| トランスパイルエラー | Load("Transpile error: ...") | 詳細メッセージ | スクリプト修正 |
| Luaランタイムエラー | Load("Runtime error: ...") | 詳細メッセージ | スクリプト修正 |
| ログファイル作成失敗 | （エラーにせず続行） | - | ログ無効化 |

### Monitoring

- tracingマクロ（error!, warn!, info!, debug!）でログ出力
- PastaLoggerによるファイル永続化
- X-ERROR-REASONでSHIORIレスポンスにエラー詳細を含める

---

## Testing Strategy

### Unit Tests

1. **From<LoaderError> for MyError変換** — 各LoaderErrorバリアントの変換確認
2. **LoggingConfig::default()** — デフォルト値の確認
3. **PastaConfig::logging()** — [logging]セクション有無の読み込み
4. **PastaLogger::new()** — ディレクトリ作成、ファイルハンドル確保

### Integration Tests

1. **PastaLoader::load() with [logging]** — ロガー付きロード成功
2. **PastaLoader::load() without [logging]** — ロガーなしロード成功
3. **PastaShiori::load() success path** — 完全なload()シーケンス
4. **PastaShiori::load() failure paths** — 各エラーケース（DirectoryNotFound, ConfigNotFound等）

### E2E Tests

1. **SHIORI load → request → unload サイクル** — 完全なライフサイクル
2. **複数インスタンス同時ロード** — 独立したログファイル出力確認

---

## Optional Sections

### Security Considerations

- **ファイルパス検証**: profile/pasta/配下以外への書き込みを防止
- **パストラバーサル防止**: file_path設定値の正規化と検証

### Performance & Scalability

- **ロード時間目標**: 1-3秒以内（既存PastaLoader::load()で達成見込み）
- **非同期ログ書き込み**: tracing_appender::non_blockingで書き込みブロッキング回避

---

## Supporting References

詳細な調査結果は [research.md](research.md) を参照。

- tracingの複数ファイル出力アーキテクチャ調査
- tracing_appenderのローテーション実装詳細
- ログファイル分割アプローチの比較検討
