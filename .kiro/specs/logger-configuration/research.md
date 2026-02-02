# Gap Analysis: logger-configuration

## 1. Current State Investigation

### 1.1 Key Files and Modules

| ファイル | 責務 | 現状 |
|---------|------|------|
| `crates/pasta_lua/src/loader/config.rs` | LoggingConfig定義 | `file_path`, `rotation_days`のみ |
| `crates/pasta_shiori/src/windows.rs` | DLL初期化、tracing設定 | フィルタなしでfmt::layer使用 |
| `crates/pasta_shiori/src/shiori.rs` | SHIORI処理、ログ出力 | `debug!`マクロで各種ログ |
| `crates/pasta_lua/src/runtime/persistence.rs` | 永続化ログ | `debug!`/`warn!`混在 |
| `crates/pasta_lua/src/logging/registry.rs` | GlobalLoggerRegistry | MakeWriter実装済み |
| `crates/pasta_lua/src/logging/logger.rs` | PastaLogger | ファイル出力済み |

### 1.2 Existing Patterns and Conventions

**ログ出力パターン:**
```rust
// 現在の使用パターン
use tracing::{debug, info, warn, error};
debug!("message");
debug!(field = value, "message");
```

**init_tracing現状:**
```rust
fn init_tracing() {
    let _ = tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(GlobalLoggerRegistry::instance().clone())
                .with_ansi(false)
                .with_target(true)
                .with_level(true),
        )
        .try_init();
}
```

**重要な制約:**
- `init_tracing()`は`DllMain`の`DLL_PROCESS_ATTACH`時に呼ばれる
- この時点ではpasta.tomlの読み込みが未完了（load_dirが不明）
- グローバルsubscriberは一度しか設定できない（`try_init`）

### 1.3 Dependency Status

**Cargo.toml (workspace):**
```toml
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```
✅ **`env-filter`機能は既に有効** - EnvFilterが使用可能

### 1.4 Log Statements Requiring Level Changes

| 現在の出力 | 現在レベル | 要求レベル | ファイル |
|-----------|-----------|-----------|---------|
| `SHIORI.load function cached` | DEBUG | TRACE | shiori.rs:171 |
| `SHIORI.request function cached` | DEBUG | TRACE | shiori.rs:180 |
| `SHIORI.unload function cached` | DEBUG | TRACE | shiori.rs:189 |
| `SHIORI.load returned true` | DEBUG | TRACE | shiori.rs:244 |
| `Processing SHIORI request` | DEBUG | TRACE | shiori.rs:144 |
| `SHIORI.request completed` | DEBUG | TRACE | shiori.rs:282 |
| `Persistence file not found` | DEBUG | WARN | persistence.rs:170 |
| `SHIORI.unload called successfully` | DEBUG | INFO | shiori.rs:311 |

---

## 2. Requirements Feasibility Analysis

### 2.1 Technical Needs from Requirements

| 要件 | 技術的ニーズ | 既存資産 | ギャップ |
|-----|-------------|---------|---------|
| Req 1: LoggingConfig拡張 | `level`, `filter`フィールド追加 | LoggingConfig構造体あり | **Extend**: フィールド追加のみ |
| Req 2: EnvFilterフィルタリング | tracing_subscriber::filter::EnvFilter | `env-filter`機能有効 | **Missing**: init_tracing統合 |
| Req 3: ログレベル調整 | `debug!` → `trace!`/`info!`/`warn!` | マクロ使用パターン確立 | **Extend**: マクロ変更のみ |
| Req 4: リクエスト/レスポンスログ | 200 OK判定後のログ追加 | call_lua_request内で判定可能 | **Extend**: 条件分岐追加 |
| Req 5: 設定スキーマ | TOML serde対応 | serdeパターン確立 | **Extend**: フィールド追加 |
| Req 6: init_tracing統合 | EnvFilter構築・適用 | init_tracing存在 | **Constraint**: タイミング問題 |
| Req 7: 後方互換性 | デフォルト値維持 | `#[serde(default)]`パターン | **None**: 既存パターンで対応可 |

### 2.2 Critical Constraint: init_tracing Timing

**問題:**
```
DllMain (DLL_PROCESS_ATTACH)
  └─ RawShiori::new()
       └─ init_tracing() ← ここでsubscriber設定
          （load_dir不明、pasta.toml読めない）

load() エントリポイント
  └─ PastaShiori::load()
       └─ PastaLoader::load()
            └─ PastaConfig::load() ← ここでpasta.toml読み込み
```

**解決オプション:**
1. **デフォルトフィルタで初期化** → 実行時変更不可（tracing制約）
2. **環境変数連携** → `PASTA_LOG`環境変数でオーバーライド
3. **reload機能** → `tracing_subscriber::reload`レイヤー使用（複雑）

### 2.3 Complexity Signals

- **Simple**: ログレベル変更（マクロ置換）
- **Simple**: LoggingConfig拡張（serdeフィールド追加）
- **Medium**: EnvFilter構築とinit_tracing統合
- **Medium**: 200 OKログ追加（レスポンス文字列解析）

---

## 3. Implementation Approach Options

### Option A: Extend Existing + Default Filter Strategy

**概要:** pasta.toml読み込み前にデフォルトEnvFilterを適用し、設定ファイルは次回起動時に反映

**変更対象:**
- `config.rs`: LoggingConfig拡張（`level`, `filter`追加）
- `windows.rs`: init_tracingでデフォルトフィルタ適用
- `shiori.rs`: ログレベル変更（7箇所）
- `persistence.rs`: ログレベル変更（1箇所）

**実装詳細:**
```rust
// config.rs
pub struct LoggingConfig {
    pub file_path: String,
    pub rotation_days: usize,
    #[serde(default = "default_level")]
    pub level: String,  // "error"|"warn"|"info"|"debug"|"trace"
    pub filter: Option<String>,  // EnvFilter directive string
}

// windows.rs
fn init_tracing() {
    use tracing_subscriber::filter::EnvFilter;
    // デフォルトフィルタ（実行時にpasta.tomlから変更不可）
    let filter = EnvFilter::new("debug");
    let _ = tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(GlobalLoggerRegistry::instance().clone())
                .with_filter(filter)
                ...
        )
        .try_init();
}
```

**Trade-offs:**
- ✅ シンプルな実装、既存パターンを踏襲
- ✅ 後方互換性完全維持
- ❌ pasta.toml設定が次回起動まで反映されない
- ❌ ユーザー期待との乖離（設定変更→即時反映を期待）

### Option B: Environment Variable Override

**概要:** `PASTA_LOG`環境変数でEnvFilterを指定可能にし、pasta.tomlはデフォルト値として機能

**変更対象:**
- Option Aと同じ + 環境変数チェック追加

**実装詳細:**
```rust
fn init_tracing() {
    use tracing_subscriber::filter::EnvFilter;
    // 環境変数優先、なければデフォルト
    let filter = EnvFilter::try_from_env("PASTA_LOG")
        .unwrap_or_else(|_| EnvFilter::new("debug"));
    ...
}
```

**Trade-offs:**
- ✅ 開発者がデバッグ時に柔軟にログ制御可能
- ✅ Option Aの利点を維持
- ❌ pasta.toml設定がDLL初期化時に反映されない問題は残る
- ❌ 環境変数設定が必要（ゴースト開発者には敷居が高い）

### Option C: Hybrid with Reload Layer (Research Needed)

**概要:** `tracing_subscriber::reload`レイヤーを使用し、load()時にフィルタを動的更新

**変更対象:**
- `windows.rs`: reloadレイヤー導入
- `shiori.rs`: load()時にフィルタ更新
- 新規: フィルタ更新API

**実装詳細（概念）:**
```rust
use tracing_subscriber::reload;

static FILTER_HANDLE: OnceLock<reload::Handle<EnvFilter, ...>> = OnceLock::new();

fn init_tracing() {
    let filter = EnvFilter::new("debug");
    let (filter_layer, handle) = reload::Layer::new(filter);
    FILTER_HANDLE.set(handle);
    // ...
}

// load()時
fn update_filter(config: &LoggingConfig) {
    if let Some(handle) = FILTER_HANDLE.get() {
        let new_filter = EnvFilter::new(&config.filter());
        handle.modify(|f| *f = new_filter);
    }
}
```

**Trade-offs:**
- ✅ pasta.toml設定が即座に反映
- ✅ ユーザー期待に完全合致
- ❌ 実装複雑度が大幅に増加
- ❌ reloadレイヤーの型制約が厳しい（Research Needed）
- ❌ グローバルstatic管理が煩雑

---

## 4. Implementation Complexity & Risk

### Effort Estimation

| オプション | 工数 | 根拠 |
|-----------|------|------|
| Option A | **S** (1-2日) | 既存パターン拡張のみ、明確なスコープ |
| Option B | **S** (1-2日) | Option A + 環境変数チェック追加 |
| Option C | **M** (3-5日) | reload機構調査・実装、テスト複雑化 |

### Risk Assessment

| オプション | リスク | 根拠 |
|-----------|--------|------|
| Option A | **Low** | 既存パターン、明確な実装パス |
| Option B | **Low** | Option A同等、環境変数は確立技術 |
| Option C | **Medium** | reload APIの型制約が不明、調査必要 |

---

## 5. Recommendations for Design Phase

### 推奨アプローチ

**Option B: Environment Variable Override** を推奨

**理由:**
1. 実装が単純で予測可能
2. 開発者向けデバッグ機能として`PASTA_LOG`環境変数は有用
3. pasta.toml設定はデフォルト値として機能（起動時適用）
4. 将来的にOption C（reload）への移行パスを残せる

### Key Decisions for Design Phase

1. **フィルタ優先順位**: `PASTA_LOG`環境変数 > pasta.toml > ハードコードデフォルト
2. **デフォルトフィルタ文字列**: Requirement 3のレベル調整を反映した静的フィルタ
3. **200 OK判定方法**: レスポンス文字列に`"SHIORI/3.0 200 OK"`が含まれるかチェック

### Research Items to Carry Forward

1. **reload Layer調査** (Optional): 将来的な動的フィルタ変更対応の可能性
2. **ログフォーマット拡張** (Future): timestamp形式、フィールド順序のカスタマイズ

---

## 6. Summary

| 項目 | 状況 |
|------|------|
| **主要ギャップ** | init_tracingタイミング問題（pasta.toml読み込み前に実行） |
| **既存資産** | `env-filter`機能有効、LoggingConfig・init_tracing存在 |
| **推奨アプローチ** | Option B（環境変数オーバーライド） |
| **工数** | S (1-2日) |
| **リスク** | Low |
