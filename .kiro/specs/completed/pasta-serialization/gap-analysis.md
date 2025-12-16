# Gap Analysis: pasta-serialization

## Analysis Summary

**スコープ**: PastaEngineに永続化ディレクトリパス管理機能を追加し、Runeスクリプトからのパスアクセスを実現

**主要課題**:
- PastaEngine構造体への`Option<PathBuf>`フィールド追加と初期化時のパス検証
- トランスパイラによるRuneラベル関数シグネチャ変更（`pub fn label_name(ctx)`生成）
- Rune VM実行時のコンテキスト引数渡し（`vm.execute(hash, context)`）
- `tempfile`クレート導入とテスト基盤整備

**推奨アプローチ**: **Option C (Hybrid)** - エンジン拡張（コンストラクタ、フィールド、実行ロジック）と新規テストファイル作成を組み合わせ

**実装難易度**: **Medium (3-5 days)** - 既存パターンに沿った拡張だが、トランスパイラとVM実行の統合に注意が必要

**リスク**: **Low** - 明確な設計方針（`pasta-engine-independence`遵守）、既存APIへの影響なし、テストカバレッジ拡充で検証可能

---

## 1. Current State Investigation

### 1.1 Domain Assets Inventory

#### Core Components

**PastaEngine** (`crates/pasta/src/engine.rs`):
- **構造体定義** (Line 61-68):
  ```rust
  pub struct PastaEngine {
      unit: Arc<rune::Unit>,
      runtime: Arc<rune::runtime::RuntimeContext>,
      label_table: LabelTable,
  }
  ```
- **コンストラクタ** (Line 85-166):
  - `PastaEngine::new(script: &str)` - スクリプト文字列からエンジン作成
  - `PastaEngine::with_random_selector(script, random_selector)` - カスタム乱数選択器対応
  - パース→トランスパイル→Runeコンパイルの3段階処理
  - グローバルキャッシュ使用（`pasta-engine-independence`で削除予定）

- **実行メソッド** (Line 270-280):
  ```rust
  let mut vm = Vm::new(self.runtime.clone(), self.unit.clone());
  let hash = rune::Hash::type_hash(&[fn_name.as_str()]);
  let execution = vm.execute(hash, ()).map_err(|e| PastaError::VmError(e))?;
  ```
  - 現状: `vm.execute(hash, ())` - 空タプルを引数として渡す
  - **Gap**: コンテキスト引数（永続化パス含む）を渡す機構が存在しない

**Transpiler** (`crates/pasta/src/transpiler/mod.rs`):
- **ラベル→Rune関数変換** (Line 155):
  ```rust
  output.push_str(&format!("pub fn {}() {{\n", fn_name));
  ```
  - 現状: 引数なしの関数シグネチャ生成
  - **Gap**: `pub fn label_name(ctx)` 形式への変更が必要

**Error Types** (`crates/pasta/src/error.rs`):
- `PastaError` enum: ParseError, LabelNotFound, RuneRuntimeError等
- **Gap**: 永続化ディレクトリ関連エラー（ディレクトリ不在、パス解決失敗）の型が未定義

**Testing Infrastructure** (`crates/pasta/tests/`):
- 既存テストファイル: `engine_integration_test.rs`, `parser_tests.rs`, `stdlib_integration_test.rs`等
- テストは`cargo test`で実行可能
- **Gap**: 永続化パス管理に関するテストファイルが存在しない

#### Dependencies

**Cargo.toml** (`crates/pasta/Cargo.toml`):
```toml
[dependencies]
rune = "0.14"
thiserror = "2"
tracing = "0.1"
# ... (他の依存関係)

[dev-dependencies]
# (現在空)
```
- **Gap**: `tempfile` クレートが`[dev-dependencies]`に未追加

### 1.2 Architecture Patterns & Conventions

**レイヤー構成**:
- **Parser Layer**: Pasta DSL → AST
- **Transpiler Layer**: AST → Rune Source
- **Runtime Layer**: Rune Source → Execution (Vm)

**命名規則**:
- ファイル名: `snake_case.rs`
- 構造体: `PascalCase`
- 関数: `snake_case`
- エラー型: `XxxError`

**エラーハンドリング**:
- `Result<T>` alias with `PastaError`
- `thiserror::Error` derive for structured errors

**ロギング**:
- 現状: `#[cfg(debug_assertions)] eprintln!` によるデバッグ出力（Line 102, 109, 127-129）
- **Constraint**: `tracing`クレートが依存関係に含まれているが、コード内で未使用
- **Gap**: 構造化ロギング（`error!`, `info!`, `debug!`）が未実装

**エンジン独立性原則** (`pasta-engine-independence`):
- 各`PastaEngine`インスタンスは完全に独立したデータを所有
- グローバル状態（`static`変数）の禁止（現在はグローバルキャッシュが存在し違反状態）
- 所有権ベースのメモリ管理

### 1.3 Integration Surfaces

**Rune VM Integration**:
- `rune::Vm::new(runtime, unit)` - VM生成
- `vm.execute(hash, args)` - 関数実行
  - `args`は`rune::runtime::Args`トレイトを実装した型（現在は`()`）
  - **Research Needed**: Runeでのstruct/hashmap引数渡しの実装方法

**Transpiler Output**:
- 生成されるRune関数はジェネレータ（`yield`使用）
- 標準ライブラリ関数（`emit_text`, `change_speaker`等）を呼び出し

---

## 2. Requirements Feasibility Analysis

### 2.1 Technical Needs from Requirements

#### Data Models

| Requirement | Data Model | Status |
|-------------|------------|--------|
| Req 1, 4 | `PastaEngine::persistence_path: Option<PathBuf>` | **Missing** - フィールド追加必要 |
| Req 2 | Runeコンテキスト構造体（`persistence_path: String`フィールド含む） | **Missing** - Rune VM引数として構築 |
| Req 7 | エラー型: `PersistenceDirectoryNotFound`, `InvalidPath` | **Missing** - `PastaError`に追加 |

#### APIs & Services

| Requirement | API | Status |
|-------------|-----|--------|
| Req 1 | `PastaEngine::new(script: &str)` → `PastaEngine::new(script, persistence_path: Option<PathBuf>)` | **Missing** - シグネチャ変更 |
| Req 1 | `PastaEngine::with_persistence(script, path, random_selector)` 新規コンストラクタ | **Missing** - 追加必要（既存APIとの互換性保持） |
| Req 2 | `Transpiler::transpile_label` - `pub fn label_name(ctx)`生成 | **Missing** - トランスパイラロジック変更 |
| Req 2 | `PastaEngine::execute_label` - `vm.execute(hash, context)` | **Missing** - 実行ロジック変更 |

#### Business Rules & Validation

| Requirement | Rule | Complexity |
|-------------|------|-----------|
| Req 1.2 | 相対パス→絶対パス変換（`std::fs::canonicalize` or `std::env::current_dir` + `path.join`） | Low |
| Req 1.4 | 永続化ディレクトリの存在確認（`PathBuf::is_dir()`） | Low |
| Req 2.2-2.3 | `persistence_path`フィールドの空文字列/絶対パス設定ロジック | Low |
| Req 3 | テスト時の一時ディレクトリ管理（`tempfile::TempDir`） | Low |

#### Non-Functional Requirements

| Category | Requirement | Status |
|----------|-------------|--------|
| Security | Req 5.3 - パストラバーサル攻撃防止（ドキュメント化） | **Gap** - ドキュメント未作成 |
| Performance | キャッシュの独立性（Req 2） | **Constraint** - `pasta-engine-independence`対応と連動 |
| Reliability | Req 7 - 構造化ロギング（`tracing`使用） | **Gap** - 現在`eprintln!`のみ |
| Testability | Req 6 - 7つのテストシナリオ | **Missing** - テストファイル未作成 |

### 2.2 Gaps & Constraints

#### Missing Capabilities

1. **エンジンフィールド拡張**:
   - `PastaEngine::persistence_path: Option<PathBuf>` 追加
   - 初期化時のパス検証ロジック（存在確認、絶対パス正規化）

2. **トランスパイラ変更**:
   - ラベル関数シグネチャ: `pub fn label_name()` → `pub fn label_name(ctx)`
   - 全ラベル（グローバル・ローカル）に統一適用

3. **VM実行統合**:
   - コンテキスト構造体/ハッシュマップの構築ロジック
   - `vm.execute(hash, context)` 呼び出しへの変更

4. **エラー型拡張**:
   - `PastaError::PersistenceDirectoryNotFound`
   - `PastaError::InvalidPersistencePath`

5. **ロギング統合**:
   - `tracing`マクロ（`error!`, `info!`, `debug!`）への移行
   - 構造化フィールド（`path`, `operation`）の追加

6. **テスト基盤**:
   - `crates/pasta/tests/persistence_test.rs` 新規作成
   - `tempfile`クレート追加（`[dev-dependencies]`）

7. **ドキュメント**:
   - Runeスクリプト開発者向けガイド（永続化実装例、セキュリティベストプラクティス）

#### Constraints

1. **`pasta-engine-independence`遵守**:
   - 永続化パスはインスタンス所有（`Option<PathBuf>`）
   - グローバルキャッシュの削除と並行して実装する必要あり
   - 複数インスタンスで異なる永続化パスを保持可能

2. **Rune API制約**:
   - `vm.execute(hash, args)` の`args`型要件（`rune::runtime::Args`トレイト実装）
   - **Research Needed**: struct/hashmap引数の推奨パターン（Runeドキュメント確認必要）

3. **既存API互換性**:
   - `PastaEngine::new(script)` は永続化パスなしのデフォルト動作として維持
   - 新規コンストラクタ追加で段階的移行を可能にする

4. **ロギング方針**:
   - `steering/logging.md` に従った構造化ロギング（関数名プレフィックス、フィールド優先）
   - デバッグビルド時の`eprintln!`を`debug!`マクロに置き換え

### 2.3 Unknowns (Research Needed)

1. **Rune VM Context Passing**:
   - Runeで構造体またはハッシュマップを引数として渡す推奨パターン
   - `rune::to_value`でのstruct/hashmapシリアライズ方法
   - Rune側でのフィールドアクセス構文（`ctx.persistence_path` vs `ctx["persistence_path"]`）

2. **Rune TOML Serialization**:
   - Rune 0.14での標準TOML機能の利用可否
   - ドキュメント例で使用するAPI（`toml::to_string`, `toml::from_string`相当）

---

## 3. Implementation Approach Options

### Option A: Extend Existing Components

**適用範囲**: PastaEngine構造体とTranspilerの既存ロジック拡張

#### 変更対象ファイル

**`crates/pasta/src/engine.rs`**:
- **フィールド追加** (Line 61-68):
  ```rust
  pub struct PastaEngine {
      unit: Arc<rune::Unit>,
      runtime: Arc<rune::runtime::RuntimeContext>,
      label_table: LabelTable,
      persistence_path: Option<PathBuf>, // 新規フィールド
  }
  ```

- **コンストラクタ変更** (Line 85-166):
  - `new(script)` - デフォルト動作（`persistence_path: None`）維持
  - `new_with_persistence(script, path)` - 新規コンストラクタ追加
  - パス検証ロジック追加（`validate_persistence_path`ヘルパー関数）

- **実行ロジック変更** (Line 270-280):
  - コンテキスト構築: `build_rune_context(&self) -> rune::Value`
  - `vm.execute(hash, context)` 呼び出しに変更

**`crates/pasta/src/transpiler/mod.rs`**:
- **シグネチャ生成変更** (Line 155):
  ```rust
  // Before: output.push_str(&format!("pub fn {}() {{\n", fn_name));
  // After:
  output.push_str(&format!("pub fn {}(ctx) {{\n", fn_name));
  ```

**`crates/pasta/src/error.rs`**:
- **エラー型追加** (Line 50以降):
  ```rust
  #[error("Persistence directory not found: {path}")]
  PersistenceDirectoryNotFound { path: String },
  
  #[error("Invalid persistence path: {path}")]
  InvalidPersistencePath { path: String },
  ```

#### 互換性評価

- **後方互換性**: `PastaEngine::new(script)` のデフォルト動作維持により保証
- **既存テスト影響**: トランスパイラ変更により全ラベル関数が`ctx`引数を受け取るため、既存テストは引数無視で動作（Rune側で未使用引数はエラーにならない）
- **破壊的変更なし**: 新規フィールドは`Option`型、既存コードパスでは`None`として動作

#### 複雑度とメンテナンス性

- **認知負荷**: Medium - エンジン初期化ロジックに条件分岐追加（永続化パスあり/なし）
- **単一責任原則**: 維持 - 永続化パス管理はPastaEngineの責務範囲内
- **ファイルサイズ**: `engine.rs`は現在1040行 → 約100行増加（1140行程度）、許容範囲内

#### Trade-offs

✅ **利点**:
- 最小限のファイル変更（3ファイルのみ）
- 既存アーキテクチャパターンに沿った拡張
- 段階的な機能追加が可能

❌ **欠点**:
- `engine.rs`の肥大化リスク（将来の機能追加時）
- トランスパイラ変更が全ラベル関数に影響

---

### Option B: Create New Components

**適用範囲**: 永続化パス管理を独立したモジュールとして実装

#### 新規作成ファイル

**`crates/pasta/src/persistence.rs`** (新規):
```rust
pub struct PersistenceConfig {
    path: PathBuf,
}

impl PersistenceConfig {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self>;
    pub fn validate(path: &Path) -> Result<()>;
    pub fn to_absolute(path: &Path) -> Result<PathBuf>;
    pub fn as_path(&self) -> &Path;
}
```

**`crates/pasta/src/runtime/context.rs`** (新規):
```rust
pub struct RuneExecutionContext {
    persistence_path: Option<String>,
}

impl RuneExecutionContext {
    pub fn new(persistence_config: Option<&PersistenceConfig>) -> Self;
    pub fn to_rune_value(&self) -> Result<rune::Value>;
}
```

#### 統合ポイント

**PastaEngine統合**:
```rust
pub struct PastaEngine {
    // ... existing fields
    persistence_config: Option<PersistenceConfig>, // 新規フィールド
}

impl PastaEngine {
    fn execute_label_with_filters(&mut self, ...) -> Result<Vec<ScriptEvent>> {
        let context = RuneExecutionContext::new(self.persistence_config.as_ref());
        let context_value = context.to_rune_value()?;
        let execution = vm.execute(hash, (context_value,))?;
        // ...
    }
}
```

#### 責務境界

- **PersistenceConfig**: パス検証、正規化、所有
- **RuneExecutionContext**: Rune VM用の引数構築
- **PastaEngine**: 上記2つのオーケストレーション

#### Trade-offs

✅ **利点**:
- 明確な責務分離（永続化ロジックの独立）
- `engine.rs`の肥大化防止
- テストの独立性向上（`persistence.rs`単体テスト可能）

❌ **欠点**:
- ファイル数増加（2ファイル新規作成）
- インターフェース設計の複雑性（3コンポーネント間の連携）
- 小規模機能に対してオーバーエンジニアリングの可能性

---

### Option C: Hybrid Approach (推奨)

**適用範囲**: エンジン拡張 + 独立したテストファイル作成

#### 実装戦略

**Phase 1: Core Extension** (Option A準拠):
- `engine.rs`: フィールド・コンストラクタ・実行ロジック拡張
- `transpiler/mod.rs`: シグネチャ生成変更
- `error.rs`: エラー型追加

**Phase 2: Testing Infrastructure** (Option B準拠):
- `tests/persistence_test.rs` 新規作成（Req 6の7テストシナリオ）
- `tempfile`クレート追加
- テストフィクスチャ用ディレクトリ構造（`tests/fixtures/persistence/`）

**Phase 3: Documentation & Logging**:
- `tracing`マクロへの段階的移行
- Runeスクリプト開発者向けガイド作成

#### 段階的実装

1. **最小実装** (Req 1, 2, 4):
   - `PastaEngine`フィールド追加
   - `new_with_persistence`コンストラクタ
   - トランスパイラシグネチャ変更
   - VM実行統合

2. **テスト拡充** (Req 3, 6):
   - `tempfile`統合
   - 7テストシナリオ実装

3. **ロギング・ドキュメント** (Req 5, 7):
   - `tracing`マクロ導入
   - 構造化ロギング
   - Runeスクリプトガイド

#### リスク軽減

- **増分ロールアウト**: Phase 1で基本機能、Phase 2でテスト、Phase 3で運用性向上
- **フィーチャーフラグ不要**: `Option<PathBuf>`により既存コードパスと新機能が自然に共存
- **ロールバック戦略**: Phase単位でコミット分離、各Phaseが独立して動作

#### Trade-offs

✅ **利点**:
- バランスの取れたアプローチ（複雑性 vs 保守性）
- 段階的実装で早期フィードバック獲得
- 既存パターン踏襲 + テスト独立性確保

❌ **欠点**:
- 3段階の計画が必要（単一PRで完結しない）
- Phase間の調整コスト

---

## 4. Implementation Complexity & Risk

### Effort Estimation: **Medium (3-5 days)**

#### 内訳

| Phase | タスク | 見積もり |
|-------|--------|---------|
| Phase 1 | Core Extension (engine, transpiler, error) | 1.5 days |
| Phase 2 | Testing Infrastructure (tests, tempfile) | 1.5 days |
| Phase 3 | Logging & Documentation | 1 day |
| | **合計** | **4 days** |

#### 根拠

- **既存パターン利用**: エンジン拡張は既存コンストラクタパターンを踏襲
- **明確な統合ポイント**: トランスパイラ（1箇所）、VM実行（1箇所）の変更
- **テストフレームワーク確立**: 既存の`cargo test`ベース、追加設定不要
- **ドキュメント化作業**: Runeスクリプトガイドの執筆（既存パターン参考）

### Risk Assessment: **Low**

#### リスク要因と緩和策

| リスク | 深刻度 | 緩和策 |
|--------|--------|--------|
| Rune VM引数渡しの実装詳細不明 | Medium | **Research Phase**: Runeドキュメント・サンプルコード確認、最小実装テスト |
| トランスパイラ変更が既存テストを破壊 | Low | 既存テストは引数無視で動作、Phase 2で全テスト実行確認 |
| `pasta-engine-independence`との競合 | Low | 両機能は独立（キャッシュ削除 vs パス追加）、統合テストで検証 |
| ロギング変更の影響範囲 | Low | `tracing`は既存依存関係、段階的移行（`eprintln!`併用期間あり） |

#### 高リスク項目への対処

**Rune VM Context Passing** (Unknown要素):
- **調査アクション**:
  1. Rune 0.14ドキュメント「Function Arguments」セクション確認
  2. `rune::to_value`でのstruct/hashmap変換例を検索
  3. 最小POC作成（単一フィールドstructをVMに渡すテスト）
- **Fallback Plan**: hashmap引数が複雑な場合、単一String引数として渡し、Rune側で解析

---

## 5. Recommendations for Design Phase

### 5.1 Preferred Approach

**Option C (Hybrid)** を推奨します。

**理由**:
1. **適度な複雑性**: エンジン拡張は最小限、テストは独立ファイルで管理
2. **段階的実装**: 3フェーズに分割することで早期フィードバックと品質保証を両立
3. **既存パターン踏襲**: `pasta-engine-independence`と同様、エンジンフィールド追加による拡張
4. **保守性**: テストの独立性により、将来的なリファクタリングが容易

### 5.2 Key Design Decisions

#### Decision 1: Rune Context Format

**選択肢**:
- A. Struct（型安全、フィールド名明確）
- B. HashMap（柔軟性、動的フィールド追加可能）

**推奨**: **Option A (Struct)** 
- `persistence_path`は固定フィールド、型安全性を優先
- 将来的な拡張（他のコンテキスト情報）もstructフィールド追加で対応

**Design Phaseでの検討事項**:
- Runeでのstruct定義方法（Rust側で定義 or Rune側で定義）
- フィールドアクセス構文の確定（`ctx.persistence_path` vs `ctx["persistence_path"]`）

#### Decision 2: Constructor API Design

**選択肢**:
- A. `new(script, path: Option<PathBuf>)` - 既存APIを変更
- B. `new(script)` + `new_with_persistence(script, path)` - 新規API追加

**推奨**: **Option B**
- 既存コードへの影響ゼロ
- 段階的な移行パスを提供

**Design Phaseでの検討事項**:
- `new_with_persistence`の第2引数型（`PathBuf` or `impl Into<PathBuf>` or `Option<PathBuf>`）
- `with_random_selector`との組み合わせAPI（`new_with_persistence_and_random`）の必要性

#### Decision 3: Logging Strategy

**推奨**: 段階的移行
- **Phase 1**: `eprintln!`を`debug!`/`info!`/`error!`に置き換え
- **Phase 2**: 構造化フィールド追加（`path`, `operation`）
- **Phase 3**: `steering/logging.md`完全準拠（関数名プレフィックス等）

**Design Phaseでの検討事項**:
- ログレベル詳細設計（Req 7の4シナリオごと）
- 構造化フィールドのネーミング統一

### 5.3 Research Items for Design Phase

#### Priority 1 (Critical - 実装前に解決必須)

**R1: Rune VM Context Passing Implementation**
- **調査内容**:
  - `rune::to_value`でのstruct/hashmap変換方法
  - `vm.execute(hash, args)`の`args`型要件詳細
  - Rune側でのコンテキストアクセス構文
- **成果物**: 最小POCコード（structをVMに渡し、Rune関数内で読み取る例）
- **推定時間**: 2-3 hours

**R2: Rune TOML Serialization API**
- **調査内容**:
  - Rune 0.14での標準TOML機能の利用可否
  - ドキュメント例で使用するAPI仕様
- **成果物**: TOMLシリアライズ・デシリアライズのRuneスクリプト例
- **推定時間**: 1-2 hours

#### Priority 2 (Important - 設計時に考慮)

**R3: Path Traversal Attack Mitigation**
- **調査内容**:
  - Rustでのパス正規化ベストプラクティス
  - `../`除去、絶対パス強制の実装パターン
- **成果物**: セキュリティベストプラクティスドキュメント（Runeスクリプト開発者向け）
- **推定時間**: 1-2 hours

**R4: `pasta-engine-independence` Integration**
- **調査内容**:
  - キャッシュ削除の実装スケジュール
  - 永続化パス機能との統合順序
- **成果物**: 統合テスト計画
- **推定時間**: 1 hour

### 5.4 Next Steps

1. **Design Phase Transition**:
   - 本Gap Analysisをレビュー
   - `/kiro-spec-design pasta-serialization` 実行

2. **Research Phase** (Design Phase内):
   - R1, R2の調査を優先実施
   - POCコード作成で実装詳細を確定

3. **Design Document作成**:
   - Option C (Hybrid) に基づいた詳細設計
   - 3 Phaseの実装順序とタスク分解

4. **Implementation Phase**:
   - Phase 1: Core Extension (1.5 days)
   - Phase 2: Testing Infrastructure (1.5 days)
   - Phase 3: Logging & Documentation (1 day)

---

## Document Status

✅ **Analysis Approach**: gap-analysis.md framework準拠  
✅ **Current State Investigation**: PastaEngine, Transpiler, Error, Testing, Dependencies調査完了  
✅ **Requirements Analysis**: 7要件から技術要求を抽出、Gap/Constraint特定  
✅ **Options Evaluation**: Option A/B/C評価、Option C (Hybrid)を推奨  
✅ **Complexity & Risk**: Medium/Low評価、リスク緩和策提示  
✅ **Recommendations**: Design Phaseでの意思決定項目と調査事項を明示  

**Language**: Japanese (ja) - as specified in spec.json
