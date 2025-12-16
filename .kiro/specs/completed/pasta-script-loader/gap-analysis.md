# Gap Analysis: pasta-script-loader

## 分析概要

**スコープ**: Pastaスクリプトエンジンへのディレクトリベース読み込み機能追加  
**課題**: 現在は文字列ベース（`PastaEngine::new(script: &str)`）のみ対応、ディレクトリ構造からの自動読み込みが未実装  
**推奨アプローチ**: Option B - 新規コンポーネント作成（`PastaEngine::from_directory`）+ 既存コンポーネント拡張  
**見積り**: M (3-7日) - 新規ディレクトリローダー + エラー型拡張 + テストフィクスチャ作成  
**リスク**: Low - 既存パターン活用、明確な仕様、Rune API理解済み

---

## 1. Current State Investigation

### 既存アセット

#### 主要モジュール構成

```
crates/pasta/src/
├── lib.rs              - 公開API定義
├── engine.rs           - PastaEngine実装（文字列ベースのみ）
├── parser/
│   └── mod.rs          - parse_str(), parse_file() 提供
├── cache.rs            - ParseCache（グローバル静的キャッシュ）
├── error.rs            - PastaError型（7種類）
├── runtime/
│   ├── labels.rs       - LabelTable（ラベル管理、ランダム選択）
│   ├── random.rs       - RandomSelector trait
│   └── variables.rs    - VariableManager
├── transpiler/         - Pasta AST → Rune変換
└── stdlib/             - Rune組み込み関数

crates/pasta/examples/
└── scripts/            - サンプルスクリプト（6ファイル、.pasta形式）

crates/pasta/tests/
├── engine_integration_test.rs
├── parser_tests.rs
├── sakura_script_tests.rs
└── ... (16テストファイル、ディレクトリローダー未実装)
```

#### 既存の関連機能

**1. PastaEngine（`engine.rs`）**
- **現在の初期化**: `PastaEngine::new(script: &str)` - 単一文字列のみ
- **内部処理フロー**:
  1. `parse_str()` でAST生成
  2. `Transpiler::transpile()` でRune変換
  3. `register_labels()` でラベルテーブル構築
  4. Rune `Sources` に追加してコンパイル
- **ラベル登録ロジック**: 同名ラベルに自動連番付与（`挨拶_0`, `挨拶_1`）
- **グローバルキャッシュ**: `PARSE_CACHE: OnceLock<ParseCache>` - スクリプト内容ハッシュでキャッシュ

**2. Parser（`parser/mod.rs`）**
- **`parse_file(path: &Path)`**: ファイル読み込み済み実装（単一ファイル）
- **`parse_str(source: &str, filename: &str)`**: 文字列パース
- **AST構造**: `PastaFile { path, labels, span }`

**3. Error Handling（`error.rs`）**
- **既存エラー型**:
  - `ParseError { file, line, column, message }`
  - `LabelNotFound { label }`
  - `FunctionNotFound { name }`
  - `NameConflict { name, existing_kind }`
  - `RuneCompileError(String)`
  - `RuneRuntimeError(String)`
  - `IoError(#[from] std::io::Error)`
  - `PestError(String)`

**4. LabelTable（`runtime/labels.rs`）**
- **機能**: ラベル登録、属性フィルタリング、ランダム選択
- **register()**: LabelInfo追加（name, scope, attributes, fn_name, parent）
- **find_label()**: 名前 + 属性フィルタで検索、複数一致時はランダム選択

**5. Rune統合**
- **Sources API**: `rune::Sources::insert()` でRuneモジュール追加
- **main.rune**: エントリーポイントファイル（areka-P0-script-engine規約）
- **mod解決**: Runeコンパイラに委譲（エンジンは`main.rune`のみ追加）

### アーキテクチャパターン

**1. レイヤードアーキテクチャ**（`steering/structure.md`より）
- **Parser層**: DSL → AST
- **Transpiler層**: AST → Rune
- **Runtime層**: Rune VM実行
- **IR出力層**: ScriptEvent生成

**2. ECSアーキテクチャ**（wintfクレートとの連携）
- **責務分離**: `pasta` = スクリプトロジック、`wintf` = UIレンダリング
- **独立性**: pastaクレートはUIに依存しない純粋なスクリプトエンジン

**3. キャッシング戦略**
- **グローバル静的キャッシュ**: `PARSE_CACHE` - FNV-1aハッシュでスクリプト内容をキャッシュ
- **共有**: 全PastaEngineインスタンス間で共有
- **注意**: pasta-engine-independenceスペック（進行中）で変更予定

### 統合ポイント

**1. areka-P0-script-engineとの依存関係**
- **ファイル構成規約**（requirements.md L690-760）:
  - `dic/` ディレクトリ: .pastaファイル配置（再帰探索）
  - `main.rune`: Runeエントリーポイント（必須）
  - Runeモジュール: `main.rune`から`mod`/`use`で明示的インポート
- **Convention over Configuration**: 設定ファイル不要、ディレクトリ構造で規約定義

**2. エラーログ出力**
- **pasta_errors.log**: スクリプトルートディレクトリに出力
- **内容**: ファイルパス・行番号・列番号・エラー詳細

**3. Runeモジュールシステム**
- **エンジンの責務**: `main.rune`のみ`Sources`に追加
- **Runeの責務**: `mod`文解析、依存ファイル探索、循環依存検出

---

## 2. Requirements Feasibility Analysis

### 技術要件マップ

| 要件 | 必要な機能 | 既存実装 | Gap |
|------|-----------|---------|-----|
| **Req 1**: ディレクトリ初期化 | 絶対パス検証、fail-fast検証 | `parse_file()`: IoError対応 | ✅ 拡張可能 - ディレクトリ検証追加 |
| **Req 2**: ファイル配置規則 | `dic/`再帰探索、`main.rune`検出 | `parse_file()`: 単一ファイル | ❌ **Missing** - ディレクトリ走査ロジック |
| **Req 3**: スクリプトローディング | 複数ファイルパース、エラー収集、Rune統合 | `parse_str()`, `Sources::insert()` | ✅ 拡張可能 - ループ処理で統合 |
| **Req 4**: ラベル名前空間 | 同名ラベル連番、ランダム選択 | `register_labels()`: 実装済み | ✅ **Exists** - そのまま利用可能 |
| **Req 5**: テストフィクスチャ | `tests/fixtures/test-project/` | `examples/scripts/`: 別構造 | ❌ **Missing** - 新規作成必要 |
| **Req 6**: 統合テスト | ディレクトリローダーテスト | `sakura_script_tests.rs`: 文字列ベース | ❌ **Missing** - 新規テスト作成 |
| **Req 7**: エラーハンドリング | 10エラー型 | 7エラー型 | ⚠️ **Constraint** - 3エラー型追加必要 |
| **Req 8**: パフォーマンス | 既存キャッシュ利用 | `PARSE_CACHE`: 実装済み | ✅ **Exists** - そのまま利用可能 |
| **Req 9**: API設計 | `from_directory()`, `list_labels()`, `reload_directory()` | `new()`: 実装済み | ❌ **Missing** - 新規メソッド追加 |

### ギャップ分析

#### Missing Capabilities

**1. ディレクトリ走査ロジック**
- **必要な機能**:
  - `dic/` ディレクトリの再帰探索（`std::fs::read_dir` + 再帰）
  - `.pasta` ファイルフィルタリング（大文字小文字不問）
  - `_` プレフィックス・隠しファイル除外
- **現状**: `parse_file()` は単一ファイルのみ対応
- **複雑度**: Low - 標準ライブラリで実装可能

**2. 複数ファイルエラー収集**
- **必要な機能**:
  - 全ファイルパース試行、個別エラー収集
  - `pasta_errors.log` 生成（スクリプトルートに出力）
  - `PastaError::MultipleParseErrors` への集約
- **現状**: `parse_str()` は1エラーで即座に停止
- **複雑度**: Low - エラーVec収集 + ログライタ追加

**3. Runeエントリーポイント管理**
- **必要な機能**:
  - `main.rune` 存在確認（必須ファイル）
  - `Sources::insert()` に`main.rune`のみ追加
  - 他の`.rn`ファイルはRuneコンパイラに委譲
- **現状**: 単一Runeソース追加のみ
- **複雑度**: Low - ファイル存在確認 + 既存API利用

**4. テストインフラストラクチャ**
- **必要な機能**:
  - `tests/fixtures/test-project/` ディレクトリ構造
  - `main.rune` + `dic/*.pasta` サンプルファイル
  - 統合テストスイート（9受入基準）
- **現状**: `examples/scripts/` のみ（異なる構造）
- **複雑度**: Low - 静的ファイル作成

#### Constraints

**1. エラー型拡張**
- **追加必要**: 3エラー型
  - `DirectoryNotFound`
  - `DicDirectoryNotFound`
  - `MainRuneNotFound`
  - `NotADirectory`（Req 1-4）
  - `PermissionDenied`（Req 1-5）
  - `MultipleParseErrors`（Req 7-5）
- **影響範囲**: `error.rs` のみ（downstream影響なし）
- **Backward Compatibility**: ✅ 新規エラー追加のみ、既存エラーは不変

**2. ParseCache依存**
- **現状**: グローバル静的キャッシュ（`OnceLock<ParseCache>`）
- **将来変更**: pasta-engine-independenceスペックで変更予定（PastaEngine所有へ）
- **対応策**: 現時点では既存実装を利用、将来スペックで移行

**3. UTF-8エンコーディング**
- **現状**: `std::fs::read_to_string()` - Rust標準動作（無効UTF-8でエラー）
- **要件**: 「Rust標準ライブラリのUTF-8取り扱いルールに準じる」
- **影響**: ✅ 追加実装不要、既存動作で要件満足

#### Research Needed

**1. ファイル探索順序保証**
- **要件**: 「ファイルシステム依存の順序で処理」（順序保証なし）
- **実装**: `read_dir()` はOS依存の順序を返す
- **確認事項**: テスト時の決定性確保手法（ソート不要の確認）

---

## 3. Implementation Approach Options

### Option A: Extend Existing `PastaEngine`

**概要**: `PastaEngine::new()` を拡張し、文字列とディレクトリ両方を受け入れる

**拡張対象ファイル**:
- `engine.rs`: コンストラクタ分岐追加
- `parser/mod.rs`: ディレクトリ走査関数追加

**変更内容**:
```rust
impl PastaEngine {
    pub fn new(input: impl Into<EngineInput>) -> Result<Self> {
        match input.into() {
            EngineInput::String(script) => Self::from_string(script),
            EngineInput::Directory(path) => Self::from_directory(path),
        }
    }
}
```

**Trade-offs**:
- ✅ 単一エントリーポイント（APIシンプル）
- ✅ 既存テストの互換性維持
- ❌ `EngineInput` enum導入で型複雑化
- ❌ エラーハンドリングが分岐（文字列 vs ディレクトリ）
- ❌ ドキュメント混在（2つの使い方を1つのドキュメントで説明）

**互換性評価**: ✅ 既存`new(script: &str)` は`impl Into<EngineInput>`で維持可能

---

### Option B: Create New `from_directory` Constructor（推奨）

**概要**: 新規コンストラクタ`from_directory()` 追加、既存`new()` は不変

**新規コンポーネント**:
1. **`DirectoryLoader`** (新規モジュール `loader.rs`):
   - 責務: ディレクトリ走査、ファイル収集、エラーログ生成
   - API: `load(path: &Path) -> Result<LoadedFiles>`
   - 独立性: `engine.rs` から分離、単独テスト可能

2. **`PastaEngine::from_directory()`** (拡張):
   - `DirectoryLoader` を使用
   - 既存の`register_labels()` ロジック再利用
   - Rune統合は既存フロー活用

**新規ファイル構成**:
```
crates/pasta/src/
├── loader.rs           # 新規: ディレクトリ走査ロジック
│   ├── DirectoryLoader
│   ├── LoadedFiles { pasta_files, main_rune }
│   └── write_error_log()
├── engine.rs           # 拡張: from_directory() 追加
└── error.rs            # 拡張: 3エラー型追加
```

**実装フロー**:
```rust
// 1. DirectoryLoader（新規）
pub struct DirectoryLoader;
impl DirectoryLoader {
    pub fn load(script_root: &Path) -> Result<LoadedFiles> {
        // 1-1. ディレクトリ検証
        validate_directory(script_root)?;
        
        // 1-2. dic/ ディレクトリ走査
        let dic_path = script_root.join("dic");
        let pasta_files = collect_pasta_files(&dic_path)?;
        
        // 1-3. main.rune 検証
        let main_rune = script_root.join("main.rune");
        if !main_rune.exists() {
            return Err(PastaError::MainRuneNotFound);
        }
        
        Ok(LoadedFiles { pasta_files, main_rune })
    }
}

// 2. PastaEngine拡張（既存コード活用）
impl PastaEngine {
    pub fn from_directory(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        
        // 2-1. ディレクトリローダー使用
        let loaded = DirectoryLoader::load(path)?;
        
        // 2-2. 全.pastaファイルパース（エラー収集）
        let (ast_list, errors) = parse_all_pasta_files(&loaded.pasta_files);
        
        // 2-3. エラーログ生成
        if !errors.is_empty() {
            write_error_log(path, &errors)?;
            return Err(PastaError::MultipleParseErrors(errors));
        }
        
        // 2-4. ラベル登録（既存ロジック）
        let mut label_table = LabelTable::new(...);
        for ast in &ast_list {
            Self::register_labels(&mut label_table, &ast.labels, None)?;
        }
        
        // 2-5. Rune統合（既存ロジック + main.rune追加）
        let rune_source = transpile_all(&ast_list)?;
        let mut sources = rune::Sources::new();
        sources.insert(rune::Source::new("entry", rune_source)?)?;
        sources.insert(rune::Source::from_path(&loaded.main_rune)?)?;
        
        // 2-6. コンパイル（既存ロジック）
        let unit = rune::prepare(&mut sources).with_context(&context).build()?;
        
        Ok(Self { unit, runtime, label_table })
    }
}
```

**Trade-offs**:
- ✅ 明確な責務分離（`DirectoryLoader` vs `PastaEngine`）
- ✅ 既存`new()` は完全不変（後方互換性100%）
- ✅ 独立テスト可能（`DirectoryLoader` 単体テスト）
- ✅ ドキュメント明確（文字列 vs ディレクトリで別々のドキュメント）
- ✅ エラーハンドリング明確（ディレクトリ特有エラーを分離）
- ⚠️ 新規ファイル追加（`loader.rs`）- プロジェクトファイル数増加
- ⚠️ 初期実装コスト若干増（Loaderモジュール作成）

**統合ポイント**:
- **既存コード再利用**: `register_labels()`, Runeコンパイルフロー
- **新規コード独立**: ディレクトリ走査、エラーログ生成
- **テスト戦略**: ユニットテスト（`DirectoryLoader`）+ 統合テスト（`from_directory`）

---

### Option C: Hybrid - New Loader + Minimal Engine Changes

**概要**: Option Bの簡略版、`loader.rs` を作成せず`engine.rs` 内にプライベートヘルパー追加

**変更内容**:
```rust
// engine.rs 内にプライベート関数追加
impl PastaEngine {
    pub fn from_directory(path: impl AsRef<Path>) -> Result<Self> {
        let (pasta_files, main_rune) = Self::load_directory_files(path.as_ref())?;
        Self::from_loaded_files(pasta_files, main_rune)
    }
    
    fn load_directory_files(path: &Path) -> Result<(Vec<PathBuf>, PathBuf)> {
        // ディレクトリ走査ロジック（プライベート）
    }
    
    fn from_loaded_files(pasta_files: Vec<PathBuf>, main_rune: PathBuf) -> Result<Self> {
        // 既存ロジック再利用
    }
}
```

**Trade-offs**:
- ✅ 新規ファイル不要（`loader.rs` なし）
- ✅ 初期実装コスト低
- ❌ `engine.rs` のファイルサイズ増加（現在1040行 → 1300行程度）
- ❌ 単独テスト困難（プライベート関数）
- ❌ 将来的なリファクタリングで Option B へ移行必要

**適用シーン**: 初期実装でOption Cを選択し、将来Option Bへリファクタリング

---

### 推奨アプローチ: Option B

**理由**:
1. **保守性**: `DirectoryLoader` の責務が明確、単独テスト可能
2. **拡張性**: 将来的に他の入力ソース（アーカイブ、HTTP等）追加時に統一パターン適用可能
3. **既存パターン整合**: `parser/mod.rs`, `transpiler/` 等の既存モジュール構成と一貫
4. **テスタビリティ**: ユニットテスト（Loader）+ 統合テスト（Engine）で多層検証
5. **areka規約準拠**: `dic/` + `main.rune` 規約を専用モジュールで明示的に実装

**実装順序**:
1. Phase 1: `error.rs` 拡張（3エラー型追加）
2. Phase 2: `loader.rs` 作成（ディレクトリ走査 + テスト）
3. Phase 3: `engine.rs` 拡張（`from_directory()` + 統合テスト）
4. Phase 4: テストフィクスチャ作成（`tests/fixtures/test-project/`）

---

## 4. Implementation Complexity & Risk

### 見積り: M (3-7日)

**内訳**:
- **Phase 1**: エラー型拡張（0.5日）
  - `error.rs` に3エラー型追加
  - エラーメッセージテスト
  
- **Phase 2**: DirectoryLoader実装（1.5日）
  - ディレクトリ走査ロジック
  - ファイルフィルタリング
  - エラーログライタ
  - ユニットテスト（6ケース）
  
- **Phase 3**: PastaEngine拡張（2日）
  - `from_directory()` コンストラクタ
  - 複数ファイルパース統合
  - main.rune統合
  - 既存ラベル登録ロジック接続
  
- **Phase 4**: テストフィクスチャ + 統合テスト（2日）
  - `tests/fixtures/test-project/` 作成
  - 9受入基準に対応する統合テスト
  - エラーケーステスト
  
- **Phase 5**: API追加メソッド（1日）
  - `list_labels()`, `list_global_labels()`, `reload_directory()`
  - API統合テスト

**Total**: 7日（バッファ含む、実働は5-6日想定）

### リスク: Low

**低リスク要因**:
1. **既存パターン活用**: `parse_file()`, `register_labels()` 等の確立された実装を再利用
2. **明確な仕様**: areka-P0-script-engineで規約定義済み、曖昧性なし
3. **Rune API理解済み**: `Sources::insert()`, `mod`解決の理解完了（`examples/rune_module_test.rs`で確認済み）
4. **標準ライブラリ依存**: ディレクトリ走査は`std::fs`、外部依存なし
5. **後方互換性**: 既存`new()` 不変、新規APIのみ追加

**潜在リスクと緩和策**:
- **リスク1**: ファイル探索順序がテストで非決定的
  - **緩和**: 要件で順序保証不要を明記、テストでは順序非依存な検証方法を採用
  
- **リスク2**: pasta_errors.logの書き込み権限エラー
  - **緩和**: IoError処理を明示的にハンドリング、`PermissionDenied`エラーで即座に停止
  
- **リスク3**: ParseCache変更（pasta-engine-independenceスペック）
  - **緩和**: 現時点では既存実装利用、将来スペックで移行計画

---

## 5. Recommendations for Design Phase

### 優先設計決定事項

**1. DirectoryLoader API設計**
- **入力**: `&Path` (絶対パス検証含む)
- **出力**: `Result<LoadedFiles>` 構造体設計
  - `pasta_files: Vec<PathBuf>` - .pastaファイルリスト
  - `main_rune: PathBuf` - main.runeパス
  - メタデータ追加の余地（将来拡張）
  
**2. エラーログフォーマット**
- **ファイル名**: `pasta_errors.log` (固定)
- **配置**: スクリプトルートディレクトリ
- **形式**: 
  ```
  [ERROR] file: dic/greetings.pasta, line: 10, column: 5
      Expected ':' after speaker name, found '@'
  
  [ERROR] file: dic/events.pasta, line: 25, column: 12
      Undefined label: ＊挨拶_削除済み
  ```
- **エンコーディング**: UTF-8
  
**3. テストフィクスチャ構造**
- **パス**: `crates/pasta/tests/fixtures/test-project/`
- **内容**:
  ```
  test-project/
  ├── main.rune              # 最小限の実装
  └── dic/
      ├── greetings.pasta    # 基本会話（5ラベル）
      ├── sakura_script.pasta # さくらスクリプト統合
      ├── variables.pasta    # 変数操作
      ├── special/
      │   └── holiday.pasta  # サブディレクトリ
      └── _ignored.pasta     # スキップ対象
  ```

**4. API設計詳細**
```rust
impl PastaEngine {
    // 既存: 文字列ベース初期化（不変）
    pub fn new(script: &str) -> Result<Self>;
    
    // 新規: ディレクトリベース初期化
    pub fn from_directory(path: impl AsRef<Path>) -> Result<Self>;
    
    // 新規: カスタムRandomSelectorサポート
    pub fn from_directory_with_selector(
        path: impl AsRef<Path>,
        selector: Box<dyn RandomSelector>,
    ) -> Result<Self>;
    
    // 新規: ラベル列挙
    pub fn list_labels(&self) -> Vec<String>;
    pub fn list_global_labels(&self) -> Vec<String>;
    
    // 新規: ディレクトリ再読み込み
    pub fn reload_directory(&mut self) -> Result<()>;
}
```

### Research Items

**1. ファイルシステムテスト戦略** ✅ **議題クローズ**
- **課題**: `read_dir()` 順序がOS依存、テストの決定性確保
- **決定事項**:
  - ✅ **tempfileクレート導入**: `pasta_errors.log`書き込みとテスト隔離のため必須
    - `[dev-dependencies]`に追加: `tempfile = "3"`
    - 使用パターン: `TempDir::new()` でテスト用ディレクトリ作成、フィクスチャコピー
  - ❌ **ファイル名ソート不要**: 要件で順序保証なし（Req 2 AC 8）、テストは順序非依存
  - ❌ **モックFS不要**: 実ファイルシステム（`tests/fixtures/` + `tempfile`）で十分

**2. エラー収集戦略** ✅ **議題クローズ**
- **課題**: 全ファイルパース試行、部分的失敗の処理
- **決定事項**:
  - ✅ **`Vec<Result<T, E>>`パターン採用**: 全ファイルパース試行、partition()でエラー分離
    ```rust
    let results: Vec<Result<PastaFile, PastaError>> = pasta_files.iter().map(parse_file).collect();
    let (asts, errors) = results.into_iter().partition(Result::is_ok);
    if !errors.is_empty() { write_error_log(); return Err(MultipleParseErrors); }
    ```
  - ❌ **スタックトレース不要**: Req 3 AC 6 - ファイルパス・行番号・列番号・エラー詳細のみ
  - ✅ **全ファイルパース方針**: 開発者体験優先（全エラー一括確認）、IoErrorのみ早期停止

**3. Rune Sources統合** ✅ **議題クローズ**
- **課題**: 複数Runeファイル（main.rune + mod）の統合順序
- **決定事項**:
  - ✅ **Sources::insert()順序影響なし**: Runeコンパイラは全ソース収集後に依存解析
    - 推奨順序: トランスパイル済みDSL ("entry") → main.rune（可読性）
    - `rune::prepare(&mut sources).build()` 時点で`mod`解決
  - ✅ **mod解決はRuneコンパイラ責務**: エンジンは`main.rune`のみ追加
    - Runeが`mod`文解析、相対パス探索、循環依存検出を実行
    - エラーは`PastaError::RuneCompileError`でラップ
  - ✅ **main.rune追加方法**: `sources.insert(rune::Source::from_path(&main_rune_path)?)?`

---

## 6. Requirement-to-Asset Map

| Requirement | 既存Asset | Gap | Implementation Approach |
|-------------|----------|-----|------------------------|
| **Req 1**: ディレクトリ初期化 | `parse_file()` | ディレクトリ検証なし | `DirectoryLoader::validate_directory()` 追加 |
| **Req 2**: ファイル配置規則 | なし | ディレクトリ走査なし | `DirectoryLoader::collect_pasta_files()` 新規実装 |
| **Req 3**: スクリプトローディング | `parse_str()`, `Sources` | 複数ファイル対応なし | `PastaEngine::from_directory()` でループ処理 |
| **Req 4**: ラベル名前空間 | `register_labels()` | ✅ 実装済み | そのまま利用 |
| **Req 5**: テストフィクスチャ | `examples/scripts/` | 構造が異なる | `tests/fixtures/test-project/` 新規作成 |
| **Req 6**: 統合テスト | `sakura_script_tests.rs` | ディレクトリ未対応 | 新規テストファイル作成 |
| **Req 7**: エラーハンドリング | 7エラー型 | 3エラー型不足 | `error.rs` 拡張 |
| **Req 8**: パフォーマンス | `PARSE_CACHE` | ✅ 実装済み | そのまま利用 |
| **Req 9**: API設計 | `new()` | 3メソッド不足 | `engine.rs` に新規メソッド追加 |

**凡例**:
- ✅ **Exists**: 実装済み、そのまま利用可能
- ⚠️ **Constraint**: 制約あり、対応必要
- ❌ **Missing**: 未実装、新規作成必要

---

## 7. Next Steps

### Design Phase へ進むための準備完了事項

✅ **既存コードベースの理解**:
- PastaEngineの初期化フロー把握
- Runeモジュールシステム理解
- ラベル登録・選択メカニズム理解

✅ **アーキテクチャパターン特定**:
- 新規コンポーネント（DirectoryLoader）の責務明確化
- 既存コンポーネント（PastaEngine）の拡張ポイント特定

✅ **実装アプローチ決定**:
- Option B（新規Loader + 既存Engine拡張）推奨
- 理由と根拠を明示

✅ **リスク評価完了**:
- Low Risk - 既存パターン活用、明確な仕様
- 見積り M (3-7日) - 具体的なフェーズ分割

### Design Phase で決定すべき事項

**1. 詳細設計**:
- `DirectoryLoader` の内部構造（関数分割、エラーハンドリング）
- `LoadedFiles` 構造体の詳細フィールド
- `pasta_errors.log` の正確なフォーマット仕様

**2. テスト戦略**:
- ユニットテスト vs 統合テストの境界線
- モックファイルシステムの採用判断
- エラーケースの網羅的リスト作成

**3. マイグレーションパス**:
- 既存コード（`examples/scripts/`）のテストフィクスチャ化
- ドキュメント更新計画（README, GRAMMAR.md）

**4. 実装タスク分解**:
- Phase 1-5の具体的なタスクリスト
- 各タスクの受入基準（Acceptance Criteria）

### コマンド

**次のコマンド（推奨）**:
```bash
/kiro-spec-design pasta-script-loader -y
```

**オプション: 設計前の追加調査**:
- Rune Sourcesの多ファイル統合動作確認（`examples/rune_module_test.rs` 拡張）
- tempfileクレートのテスト戦略検証

---

_このGap Analysisは、要件定義（requirements.md）と既存コードベースの徹底的な調査に基づき、実装可能性とアプローチを評価しました。推奨されるOption Bは、保守性・拡張性・テスタビリティのバランスが最も優れています。_
