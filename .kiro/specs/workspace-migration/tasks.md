# タスクドキュメント

## タスク生成ログ

**生成日:** 2025-12-27
**出典:** requirements.md（9 要件）、design.md（13 コンポーネント）
**戦略:** 依存順序付き、並列対応タスクグループ
**言語:** 日本語

## タスク構成

### Phase 1: ワークスペース設定（3 タスク）- 必ず最初に完了
クリティカルパス: ルート設定、ディレクトリ構造、依存関係解決

### Phase 2: pasta_core 移行（5 タスク）- Phase 1 後に開始可
Parser + Registry + エラー型の抽出

### Phase 3: pasta_rune 移行（6 タスク）- Phase 1 後に開始、Phase 2 のインポート依存
Engine + Runtime + Transpiler + エラー リファクタリング

### Phase 4: テスト・ドキュメント（3 タスク）- 最終検証
ビルド、テスト、ドキュメント更新

## 詳細タスク

---

## Phase 1: ワークスペース設定

### タスク 1.1: ルート Cargo.toml - ワークスペース構成
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** なし
**ブロック元:** なし
**見積もり:** 30 分

#### 要件カバレッジ
- 要件 1.1, 1.2, 1.3, 1.4
- 要件 4.1, 4.2, 4.3, 4.4

#### 受け入れ基準
- [ ] ワークスペース構成を `[workspace]` セクションで作成
- [ ] `members = ["crates/*"]` を設定し、/crates/ 配下のすべてのクレートを含む
- [ ] `resolver = "2"` を設定し、依存関係解決の一貫性を確保
- [ ] すべての共有依存関係を含む `[workspace.dependencies]` セクションを作成
- [ ] edition、authors、license を指定した `[workspace.package]` を定義
- [ ] ルート Cargo.toml から `[package]` セクションを削除
- [ ] ルート Cargo.toml がワークスペース専用構成になる

#### 実装詳細
- **ファイル:** `Cargo.toml`（ルート）
- **変更内容:**
  - 既存の `[package]` を `[workspace]` ブロックで置き換え
  - `[workspace.package]` にメタデータを追加
  - `[workspace.dependencies]` にすべての共有依存（pest, thiserror, rune, rand など）を追加
  - design.md の Configuration Structure からバージョン仕様を含める
  - ワークスペース依存関係に `pasta_core = { path = "crates/pasta_core" }` を追加

#### 成果物
- 完全な依存関係定義を持つワークスペース対応 Cargo.toml

---

### タスク 1.2: ディレクトリ構造を作成
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** タスク 1.1
**ブロック元:** なし
**見積もり:** 20 分

#### 要件カバレッジ
- 要件 5.1, 5.2, 5.3, 5.4, 5.5, 5.6

#### 受け入れ基準
- [ ] `/crates/` ディレクトリを作成
- [ ] `/crates/pasta_core/` サブディレクトリを作成
- [ ] `/crates/pasta_rune/` サブディレクトリを作成
- [ ] 各クレート配下に基本的なディレクトリ構造（src/）を作成
- [ ] `/tests/` がワークスペースルートに残ることを確認
- [ ] `/tests/common/` を共有ユーティリティ用に準備（空の状態で作成）

#### 実装詳細
- **作成するディレクトリ:**
  ```
  crates/
  ├── pasta_core/
  │   ├── src/
  │   │   ├── parser/
  │   │   └── registry/
  │   └── Cargo.toml
  └── pasta_rune/
      ├── src/
      │   ├── transpiler/
      │   ├── runtime/
      │   ├── stdlib/
      │   ├── ir/
      │   └── examples/
      └── Cargo.toml
  tests/
  └── common/
  ```

#### 成果物
- design.md のデータモデルセクション通りの完全なディレクトリ階層

---

### タスク 1.3: クレート Cargo.toml ファイルを作成
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** タスク 1.1, タスク 1.2
**ブロック元:** なし
**見積もり:** 30 分

#### 要件カバレッジ
- 要件 2.1, 2.3, 3.1, 3.3

#### 受け入れ基準
- [ ] `/crates/pasta_core/Cargo.toml` をパッケージメタデータで作成
- [ ] pasta_core 依存関係を設定: pest, pest_derive, thiserror, fast_radix_trie, rand, tracing
- [ ] すべての依存関係が `dependency.workspace = true` でワークスペースバージョンを参照
- [ ] `/crates/pasta_rune/Cargo.toml` をパッケージメタデータで作成
- [ ] pasta_rune 依存関係を設定: pasta_core, rune, thiserror, glob, tracing, rand, futures, toml, fast_radix_trie
- [ ] すべての依存関係が `dependency.workspace = true` でワークスペースバージョンを参照
- [ ] 両方が `edition.workspace = true`、`authors.workspace = true`、`license.workspace = true` を使用
- [ ] 両方が `publish = true` を設定

#### 実装詳細
- **pasta_core/Cargo.toml 構造:**
  ```toml
  [package]
  name = "pasta_core"
  version = "0.1.0"
  edition.workspace = true
  authors.workspace = true
  license.workspace = true
  publish = true
  
  [dependencies]
  pest.workspace = true
  pest_derive.workspace = true
  thiserror.workspace = true
  fast_radix_trie.workspace = true
  rand.workspace = true
  tracing.workspace = true
  ```

- **pasta_rune/Cargo.toml 構造:**
  ```toml
  [package]
  name = "pasta_rune"
  version = "0.1.0"
  edition.workspace = true
  authors.workspace = true
  license.workspace = true
  publish = true
  
  [dependencies]
  pasta_core.workspace = true
  rune.workspace = true
  thiserror.workspace = true
  glob.workspace = true
  tracing.workspace = true
  rand.workspace = true
  futures.workspace = true
  toml.workspace = true
  fast_radix_trie.workspace = true
  
  [dev-dependencies]
  tempfile.workspace = true
  ```

#### 成果物
- 両クレート用に適切に構成された 2 つの Cargo.toml ファイル

---

## Phase 2: pasta_core 移行

### タスク 2.1: パーサーモジュールを pasta_core に移動
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** タスク 1.2, タスク 1.3
**ブロック元:** なし
**見積もり:** 45 分

#### 要件カバレッジ
- 要件 2.2, 2.4, 5.1, 9.1

#### 受け入れ基準
- [ ] `src/parser/mod.rs` を `crates/pasta_core/src/parser/mod.rs` に移動
- [ ] `src/parser/ast.rs` を `crates/pasta_core/src/parser/ast.rs` に移動
- [ ] `src/parser/grammar.pest` を `crates/pasta_core/src/parser/grammar.pest` に移動
- [ ] parser/ 内のすべてのインポートが解決される（pest, pest_derive）
- [ ] Rune またはランタイムコンポーネントへの参照がない
- [ ] パーサーモジュールがエクスポート: `pub fn parse_str()`、`pub fn parse_file()`、`pub struct PastaFile` など

#### 実装詳細
- **移動するファイル:**
  - `src/parser/mod.rs` → `crates/pasta_core/src/parser/mod.rs`
  - `src/parser/ast.rs` → `crates/pasta_core/src/parser/ast.rs`
  - `src/parser/grammar.pest` → `crates/pasta_core/src/parser/grammar.pest`

- **インポート変更は不要**（parser は自己完結）
- **検証:** Rune、runtime、engine 依存がない

#### 成果物
- pasta_core で完全に機能するパーサーモジュール

---

### タスク 2.2: レジストリモジュールを pasta_core に移動
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** タスク 2.1
**ブロック元:** なし
**見積もり:** 45 分

#### 要件カバレッジ
- 要件 2.2, 5.1, 9.1

#### 受け入れ基準
- [ ] `src/registry/mod.rs` を `crates/pasta_core/src/registry/mod.rs` に移動
- [ ] `src/registry/scene_registry.rs` を `crates/pasta_core/src/registry/scene_registry.rs` に移動
- [ ] `src/registry/word_registry.rs` を `crates/pasta_core/src/registry/word_registry.rs` に移動
- [ ] registry/ 内のすべてのインポートが解決される（fast_radix_trie, rand）
- [ ] Rune またはエンジンコンポーネントへの参照がない
- [ ] レジストリモジュールがエクスポート: SceneRegistry、WordDefRegistry など

#### 実装詳細
- **移動するファイル:**
  - `src/registry/mod.rs` → `crates/pasta_core/src/registry/mod.rs`
  - `src/registry/scene_registry.rs` → `crates/pasta_core/src/registry/scene_registry.rs`
  - `src/registry/word_registry.rs` → `crates/pasta_core/src/registry/word_registry.rs`

- **インポート変更は不要**（registry は Pass 1 で自己完結）
- **検証:** Rune、engine、runtime 依存がない

#### 成果物
- pasta_core で完全に機能するレジストリモジュール

---

### タスク 2.3: ランタイム検索テーブルを pasta_core に抽出
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** タスク 2.2
**ブロック元:** なし
**見積もり:** 60 分

#### 要件カバレッジ
- 要件 2.2, 5.1, 5.2

#### 受け入れ基準
- [ ] WordTable を `src/runtime/words.rs` から → `crates/pasta_core/src/registry/word_table.rs` に抽出
- [ ] SceneTable を `src/runtime/scene.rs` から → `crates/pasta_core/src/registry/scene_table.rs` に抽出
- [ ] RandomSelector トレイト + DefaultRandomSelector を `src/runtime/random.rs` から → `crates/pasta_core/src/registry/random.rs` に抽出
- [ ] registry/mod.rs を更新して WordTable、SceneTable、RandomSelector を再エクスポート
- [ ] Rune 言語固有の依存関係を削除（trait/type 定義のみ保持）
- [ ] RandomSelector が言語に非依存（trait ベース、Rune 参照なし）

#### 実装詳細
- **作成/抽出するファイル:**
  - `crates/pasta_core/src/registry/word_table.rs`（runtime/words.rs から - 検索/ルックアップロジックのみ）
  - `crates/pasta_core/src/registry/scene_table.rs`（runtime/scene.rs から - 検索/ルックアップロジックのみ）
  - `crates/pasta_core/src/registry/random.rs`（runtime/random.rs から - trait + デフォルト実装）

- **RandomSelector 構造:**
  ```rust
  pub trait RandomSelector {
      fn select<T>(&self, items: &[T]) -> Option<T>;
  }
  
  pub struct DefaultRandomSelector { ... }
  impl RandomSelector for DefaultRandomSelector { ... }
  ```

- **registry/mod.rs 内のインポートを更新:**
  ```rust
  pub use scene_table::SceneTable;
  pub use word_table::WordTable;
  pub use random::{RandomSelector, DefaultRandomSelector, MockRandomSelector};
  ```

#### 成果物
- pasta_core 内の検索テーブルと RandomSelector トレイト（言語に非依存）

---

### タスク 2.4: pasta_core ParseError 型を作成
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** タスク 2.1
**ブロック元:** なし
**見積もり:** 30 分

#### 要件カバレッジ
- 要件 2.2, 2.4, 9.1

#### 受け入れ基準
- [ ] `crates/pasta_core/src/error.rs` を ParseError、ParseErrorInfo、ParseResult で作成
- [ ] ParseError バリアント定義: SyntaxError、PestError、IoError、MultipleErrors
- [ ] エラー報告に file、line、column 情報を含める
- [ ] エラー定義に thiserror クレートを使用
- [ ] lib.rs 経由でエクスポート: `pub mod error;` および `pub use error::{ParseError, ParseErrorInfo, ParseResult};`

#### 実装詳細
- **error.rs 構造（design.md から）:**
  ```rust
  pub type ParseResult<T> = std::result::Result<T, ParseError>;
  
  #[derive(Error, Debug, Clone)]
  pub enum ParseError { ... }
  
  #[derive(Debug, Clone, PartialEq)]
  pub struct ParseErrorInfo { ... }
  ```

#### 成果物
- pasta_core 内の ParseError 型システム

---

### タスク 2.5: pasta_core lib.rs を作成
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** タスク 2.1, タスク 2.2, タスク 2.3, タスク 2.4
**ブロック元:** なし
**見積もり:** 20 分

#### 要件カバレッジ
- 要件 2.1, 9.1

#### 受け入れ基準
- [ ] `crates/pasta_core/src/lib.rs` をモジュール宣言で作成
- [ ] モジュール構造: `pub mod error;`、`pub mod parser;`、`pub mod registry;`
- [ ] 再エクスポート: ParseError、ParseErrorInfo、ParseResult、parse_file、parse_str、PastaFile、SceneEntry、SceneRegistry、WordDefRegistry、WordEntry
- [ ] モジュール documentation コメントを含める

#### 実装詳細
- **lib.rs 構造（design.md から）:**
  ```rust
  //! Pasta Core - Language-independent DSL parsing and registry layer.
  
  pub mod error;
  pub mod parser;
  pub mod registry;
  
  pub use error::{ParseError, ParseErrorInfo, ParseResult};
  pub use parser::{parse_file, parse_str, PastaFile};
  pub use registry::{SceneEntry, SceneRegistry, WordDefRegistry, WordEntry};
  ```

#### 成果物
- 完全な公開 API を備えた pasta_core ライブラリ

---

## Phase 3: pasta_rune 移行

### タスク 3.1: トランスパイラモジュールを pasta_rune に移動
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** タスク 2.5
**ブロック元:** なし
**見積もり:** 45 分

#### 要件カバレッジ
- 要件 3.2, 5.2

#### 受け入れ基準
- [ ] `src/transpiler/mod.rs` を `crates/pasta_rune/src/transpiler/mod.rs` に移動
- [ ] `src/transpiler/code_generator.rs` を `crates/pasta_rune/src/transpiler/code_generator.rs` に移動
- [ ] `src/transpiler/context.rs` を `crates/pasta_rune/src/transpiler/context.rs` に移動
- [ ] `src/transpiler/error.rs` を `crates/pasta_rune/src/transpiler/error.rs` に移動
- [ ] すべてのインポートを更新して `pasta_core::{parser, registry, error}` の代わりにルートクレートを参照
- [ ] トランスパイラモジュールのコンパイルにエラーなし

#### 実装詳細
- **移動するファイル:**
  - `src/transpiler/mod.rs` → `crates/pasta_rune/src/transpiler/mod.rs`
  - `src/transpiler/code_generator.rs` → `crates/pasta_rune/src/transpiler/code_generator.rs`
  - `src/transpiler/context.rs` → `crates/pasta_rune/src/transpiler/context.rs`
  - `src/transpiler/error.rs` → `crates/pasta_rune/src/transpiler/error.rs`

- **インポート更新:**
  - 変更: `use crate::parser::` → `use pasta_core::parser::`
  - 変更: `use crate::registry::` → `use pasta_core::registry::`
  - 変更: `use crate::error::` → transpiler/error.rs または pasta_rune::error からインポート

#### 成果物
- 更新されたインポートで pasta_rune で機能するトランスパイラモジュール

---

### タスク 3.2: ランタイムモジュールを pasta_rune に移動
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** タスク 2.5
**ブロック元:** タスク 2.3（最初に検索テーブルを抽出）
**見積もり:** 45 分

#### 要件カバレッジ
- 要件 3.2, 5.2

#### 受け入れ基準
- [ ] `src/runtime/mod.rs` を `crates/pasta_rune/src/runtime/mod.rs` に移動
- [ ] `src/runtime/generator.rs` を `crates/pasta_rune/src/runtime/generator.rs` に移動
- [ ] `src/runtime/variables.rs` を `crates/pasta_rune/src/runtime/variables.rs` に移動
- [ ] **移動しない**: `scene.rs`、`words.rs`、`random.rs`（すでに pasta_core にある）
- [ ] すべてのインポートを更新して検索テーブル用に `pasta_core::{registry, ...}` を参照
- [ ] ランタイムモジュールが pasta_core から RandomSelector にアクセス可能

#### 実装詳細
- **移動するファイル:**
  - `src/runtime/mod.rs` → `crates/pasta_rune/src/runtime/mod.rs`
  - `src/runtime/generator.rs` → `crates/pasta_rune/src/runtime/generator.rs`
  - `src/runtime/variables.rs` → `crates/pasta_rune/src/runtime/variables.rs`
  - （および scene.rs、words.rs、random.rs 以外のその他の Rune 固有ランタイムファイル）

- **移動しないファイル:**
  - `src/runtime/scene.rs` → タスク 2.3 で既に pasta_core に移動
  - `src/runtime/words.rs` → タスク 2.3 で既に pasta_core に移動
  - `src/runtime/random.rs` → タスク 2.3 で既に pasta_core に移動

- **インポート更新:**
  - 変更: `use crate::registry::` → `use pasta_core::registry::{SceneTable, WordTable, RandomSelector}`

#### 成果物
- 検索テーブルを除いて、更新されたインポートで pasta_rune で機能するランタイムモジュール

---

### タスク 3.3: Engine、Cache、Loader、IR を pasta_rune に移動
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** タスク 2.5
**ブロック元:** タスク 3.1, タスク 3.2
**見積もり:** 45 分

#### 要件カバレッジ
- 要件 3.1, 3.2, 5.2

#### 受け入れ基準
- [ ] `src/engine.rs` を `crates/pasta_rune/src/engine.rs` に移動
- [ ] `src/cache.rs` を `crates/pasta_rune/src/cache.rs` に移動
- [ ] `src/loader.rs` を `crates/pasta_rune/src/loader.rs` に移動
- [ ] `src/ir/` を `crates/pasta_rune/src/ir/` に移動
- [ ] engine.rs 内のすべてのインポートを更新: `use pasta_core::{parser, registry, error}`
- [ ] cache.rs 内のすべてのインポートを更新: `use pasta_core::parser`
- [ ] loader.rs 依存関係が transpiler、runtime、engine に対してローカル（`crate::`）であることを確認

#### 実装詳細
- **移動するファイル:**
  - `src/engine.rs` → `crates/pasta_rune/src/engine.rs`
  - `src/cache.rs` → `crates/pasta_rune/src/cache.rs`
  - `src/loader.rs` → `crates/pasta_rune/src/loader.rs`
  - `src/ir/` → `crates/pasta_rune/src/ir/`

- **engine.rs 内のインポート更新:**
  - 変更: `use crate::parser::` → `use pasta_core::parser::`
  - 変更: `use crate::registry::` → `use pasta_core::registry::`

#### 成果物
- pasta_rune で機能する Engine、Cache、Loader、IR モジュール

---

### タスク 3.4: stdlib モジュールを pasta_rune に移動
**ステータス:** 未開始
**優先度:** P1
**依存:** タスク 3.2
**ブロック元:** なし
**見積もり:** 20 分

#### 要件カバレッジ
- 要件 3.2, 5.2

#### 受け入れ基準
- [ ] `src/stdlib/mod.rs` を `crates/pasta_rune/src/stdlib/mod.rs` に移動
- [ ] `src/stdlib/persistence.rs` を `crates/pasta_rune/src/stdlib/persistence.rs` に移動
- [ ] すべてのインポートが解決される（pasta_core インポートが不要であることを確認）

#### 実装詳細
- **移動するファイル:**
  - `src/stdlib/mod.rs` → `crates/pasta_rune/src/stdlib/mod.rs`
  - `src/stdlib/persistence.rs` → `crates/pasta_rune/src/stdlib/persistence.rs`

#### 成果物
- pasta_rune で機能する stdlib モジュール

---

### タスク 3.5: pasta_rune error.rs をリファクタリング
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** タスク 2.4
**ブロック元:** タスク 3.1
**見積もり:** 30 分

#### 要件カバレッジ
- 要件 3.2, 9.2

#### 受け入れ基準
- [ ] `src/error.rs`（PastaError セクションのみ）を `crates/pasta_rune/src/error.rs` に移動/リファクタリング
- [ ] `#[from] ParseError` 変換を追加: `#[from] pasta_core::error::ParseError`
- [ ] すべてのランタイムエラーバリアント含める: SceneNotFound、RuneCompileError、VmError、IoError
- [ ] エラー定義に thiserror を使用
- [ ] ParseError を `pub use pasta_core::error::ParseError;` 経由でエクスポート
- [ ] 型エイリアス: `pub type Result<T> = std::result::Result<T, PastaError>;`

#### 実装詳細
- **error.rs 構造（design.md から）:**
  ```rust
  use thiserror::Error;
  use pasta_core::error::ParseError;
  
  pub type Result<T> = std::result::Result<T, PastaError>;
  
  #[derive(Error, Debug)]
  pub enum PastaError {
      #[error(transparent)]
      Parse(#[from] ParseError),
      
      #[error("Scene not found: {scene}")]
      SceneNotFound { scene: String },
      
      // ... その他のバリアント
  }
  ```

#### 成果物
- ParseError 統合を備えた PastaError 型システム

---

### タスク 3.6: pasta_rune lib.rs を作成
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** タスク 3.1, タスク 3.2, タスク 3.3, タスク 3.4, タスク 3.5
**ブロック元:** なし
**見積もり:** 20 分

#### 要件カバレッジ
- 要件 3.1, 3.4, 9.2

#### 受け入れ基準
- [ ] `crates/pasta_rune/src/lib.rs` をモジュール宣言で作成
- [ ] モジュール宣言: `pub mod cache;`、`pub mod engine;`、`pub mod error;`、`pub mod ir;`、`mod loader;`、`pub mod runtime;`、`pub mod stdlib;`、`pub mod transpiler;`
- [ ] 再エクスポート: `pub use pasta_core as core;`
- [ ] 再エクスポート: ParseCache、PastaEngine、PastaError、Result、ScriptEvent、ContentPart、DirectoryLoader、LoadedFiles、RandomSelector、SceneTable、ScriptGenerator
- [ ] モジュール documentation コメントを含める
- [ ] `loader` モジュールがプライベート（`mod loader;`、`pub mod` ではない）

#### 実装詳細
- **lib.rs 構造（design.md から）:**
  ```rust
  //! Pasta Rune - Script engine with Rune language backend.
  
  pub mod cache;
  pub mod engine;
  pub mod error;
  pub mod ir;
  mod loader;
  pub mod runtime;
  pub mod stdlib;
  pub mod transpiler;
  
  pub use pasta_core as core;
  
  pub use cache::ParseCache;
  pub use engine::PastaEngine;
  pub use error::{PastaError, Result};
  pub use ir::{ContentPart, ScriptEvent};
  pub use loader::{DirectoryLoader, LoadedFiles};
  pub use runtime::{
      DefaultRandomSelector, RandomSelector, SceneTable, ScriptGenerator,
      ScriptGeneratorState, VariableManager, VariableScope, VariableValue,
  };
  ```

#### 成果物
- 完全な公開 API を備えた pasta_rune ライブラリ、`core::` 名前空間経由の間接的な pasta_core アクセス

---

## Phase 4: テスト・ドキュメント

### タスク 4.1: 共有テスト共通モジュールを作成
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** タスク 1.2
**ブロック元:** なし
**見積もり:** 20 分

#### 要件カバレッジ
- 要件 6.5, 6.6

#### 受け入れ基準
- [ ] `tests/common/mod.rs` を `fixtures_path()` および `fixture()` ユーティリティ関数で作成
- [ ] 関数がワークスペースレベル `/tests/fixtures/` をいずれのクレートテストコンテキストからでも解決
- [ ] `pasta_core` および `pasta_rune` クレートテストコンテキストの両方を処理
- [ ] design.md のナビゲーションロジックから `CARGO_MANIFEST_DIR` と使用

#### 実装詳細
- **ファイル:** `tests/common/mod.rs`
- **関数:**
  ```rust
  pub fn fixtures_path() -> PathBuf { ... }
  pub fn fixture(name: &str) -> PathBuf { ... }
  ```

#### 成果物
- 共有テストユーティリティモジュール

---

### タスク 4.2: テストインポートを更新してテストを実行
**ステータス:** 未開始
**優先度:** P0（ブロッキング）
**依存:** タスク 3.6, タスク 4.1
**ブロック元:** なし
**見積もり:** 90 分

#### 要件カバレッジ
- 要件 6.1, 6.2, 6.3, 6.4, 6.5

#### 受け入れ基準
- [ ] `tests/` 内のすべてのテストファイルを正しいインポートパスで更新
- [ ] pasta_core 型を使用するテストファイルがインポート: `use pasta_rune::core::{parser, registry, error};`
- [ ] pasta_rune 型を使用するテストファイルがインポート: `use pasta_rune::{engine, runtime, ...};`
- [ ] すべてのフィクスチャパス参照が `common::fixtures_path()` または `common::fixture()` を使用
- [ ] `cargo test --workspace` がすべてのテストを実行し、エラーなく成功
- [ ] リグレッションテストスイート実行成功: `parser`、`transpiler`、`engine`、`integration`

#### 実装詳細
- **テストファイル更新:**
  1. テストファイルに `mod common;` を追加（存在しない場合）
  2. フィクスチャパスを置き換え: `"fixtures/..."` → `common::fixture("...")`
  3. インポートを置き換え: `use crate::parser::` → `use pasta_rune::core::parser::`
  4. インポートを置き換え: `use crate::engine::` → `use pasta_rune::engine::`

- **検証実行:**
  - 実行: `cargo test --workspace`
  - 検証: すべてのテストが成功、コンパイルエラーなし

#### 成果物
- すべてのテストが更新され、成功実行

---

### タスク 4.3: ドキュメントを更新
**ステータス:** 未開始
**優先度:** P1
**依存:** タスク 3.6
**ブロック元:** なし
**見積もり:** 45 分

#### 要件カバレッジ
- 要件 8.1, 8.2, 8.3, 8.4

#### 受け入れ基準
- [ ] `.kiro/steering/structure.md` をワークスペース構造で更新
- [ ] `.kiro/steering/structure.md` に `/crates/` ディレクトリ階層を含める
- [ ] `README.md` をビルド指示で更新: `cargo build --workspace`、`cargo test --workspace`
- [ ] `README.md` にワークスペースアーキテクチャ概要を含める
- [ ] `.kiro/steering/tech.md` をワークスペースアーキテクチャ原則で更新
- [ ] 依存関係図を含める（pasta_rune → pasta_core）

#### 実装詳細
- **更新するファイル:**
  1. `.kiro/steering/structure.md` — ワークスペース構造セクションを追加
  2. `README.md` — ビルドセクションをワークスペースコマンドで更新
  3. `.kiro/steering/tech.md` — ワークスペース原則セクションを追加

- **コンテンツ:**
  - ワークスペース概要
  - design.md データモデルセクションからのディレクトリ構造
  - ビルドコマンド例
  - クレート依存グラフ

#### 成果物
- 新しいワークスペース構造を反映した更新されたドキュメント

---

## タスク依存グラフ

```
Phase 1（順序実行）:
  タスク 1.1（ルート Cargo.toml）
    ↓
  タスク 1.2（ディレクトリ構造）
    ↓
  タスク 1.3（クレート Cargo.toml）

Phase 2（順序実行 - pasta_core）:
  タスク 1.3
    ↓
  タスク 2.1（パーサー）
    ↓
  タスク 2.2（レジストリ）
    ↓
  タスク 2.3（検索テーブル）
    ↓
  タスク 2.4（ParseError）
    ↓
  タスク 2.5（pasta_core lib.rs）

Phase 3（部分並列 - pasta_rune）:
  タスク 2.5
    ├→ タスク 3.1（トランスパイラ）
    │    ↓
    │ タスク 3.5（error.rs）
    │    ↓
    │ タスク 3.6（pasta_rune lib.rs）
    │
    ├→ タスク 3.2（ランタイム）[タスク 2.3 に依存]
    │
    └→ タスク 3.3（Engine/Cache/Loader/IR）
         ↓
       タスク 3.4（stdlib）

Phase 4（最終）:
  タスク 1.2
    ↓
  タスク 4.1（テスト共通）
    ↓
  タスク 3.6
    ↓
  タスク 4.2（テスト更新）
    ↓
  タスク 4.3（ドキュメント）
```

## タスク実行戦略

### 推奨実行順序
1. **Phase 1:** 順序実行（T1.1 → T1.2 → T1.3）
2. **Phase 2:** 順序実行（T2.1 → T2.2 → T2.3 → T2.4 → T2.5）
3. **Phase 3:** T2.5 完了後:
   - **並列バッチ 1:** T3.1, T3.2（独立したファイル移動）
   - **バッチ 1 後の順序実行:** T3.5 → T3.6
   - **並列バッチ 2:** T3.3, T3.4（T3.1, T3.2 後）
4. **Phase 4:** T3.6 完了後: T4.1 → T4.2 → T4.3

### 並列実行に関する注記
- ファイル I/O を共有しないタスクは並列化可能
- インポート更新はすべての移動完了後に同期が必要
- 各フェーズ後に進む前にビルド検証が必須

## 成功基準

### 完了の定義
- [ ] すべてのワークスペースメンバーがエラーなくビルド: `cargo build --workspace`
- [ ] すべてのテストが成功: `cargo test --workspace`
- [ ] 未使用のインポートまたはデッドコード警告がない
- [ ] 公開 API 境界が明確でドキュメント化
- [ ] クレート間に循環依存がない
- [ ] すべての要件受け入れ基準が満たされている
- [ ] ドキュメントが最新

---

## 実装に関する注記

### クリティカルパス項目
1. ルート Cargo.toml 構成（T1.1）
2. pasta_core アセンブリ（T2.1 → T2.5）
3. pasta_rune アセンブリ（T3.1 → T3.6）
4. テスト検証（T4.2）

### リスク緩和
- **リスク:** モジュール間のインポートパス不一致
  - **対策:** すべての移動完了後に体系的に更新
- **リスク:** pasta_core と pasta_rune 間の循環依存
  - **対策:** 設計では pasta_rune が pasta_core にのみ依存することを保証
- **リスク:** 新しい構造でテストフィクスチャが見つからない
  - **対策:** タスク 4.1 が共有ユーティリティ関数を作成

### 検証ステップ
- 各フェーズ: `cargo check --workspace`
- 各フェーズ: `cargo build --workspace`
- 最終: `cargo test --workspace`
