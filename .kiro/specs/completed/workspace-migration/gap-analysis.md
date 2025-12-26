# Implementation Gap Analysis: workspace-migration

## 分析概要

**スコープ**: 単一クレート構成（`pasta`）からCargoワークスペース＋2クレート構成（`pasta_core`, `pasta_rune`）への移行

**主な課題**:
- 既存の依存関係グラフ（parser/registry → transpiler → runtime → engine）を維持しつつクレート境界を設定
- registryをparserと同一クレート（pasta_core）に配置し、共有層として機能させる
- 28個の統合テストファイルとインポートパスを新構成に適合

**推奨アプローチ**: Option B（新規クレート作成）＋段階的移行戦略

**実装リスク**: Medium（パターン確立、インポートパス一括更新、テスト継続性保証が必要）

---

## 1. Current State Investigation

### ディレクトリ構造とモジュール配置

```
src/
├── lib.rs                    # クレートエントリー、公開API定義
├── parser/                   # パーサー層（Pest文法、AST）
│   ├── mod.rs
│   ├── ast.rs
│   └── grammar.pest
├── registry/                 # 共有レジストリ（AST非依存）★pasta_coreへ
│   ├── mod.rs
│   ├── scene_registry.rs
│   └── word_registry.rs
├── transpiler/               # トランスパイラ層（AST→Rune）
│   ├── mod.rs
│   ├── code_generator.rs
│   ├── context.rs
│   └── error.rs
├── runtime/                  # ランタイム層（Rune VM実行）
│   ├── mod.rs
│   ├── generator.rs
│   ├── variables.rs
│   ├── scene.rs
│   ├── words.rs
│   └── random.rs
├── stdlib/                   # Pasta標準ライブラリ
├── engine.rs                 # 統合API
├── cache.rs                  # パースキャッシュ
├── loader.rs                 # ディレクトリローダー
├── error.rs                  # エラー型
└── ir/                       # IR出力型（ScriptEvent）
```

### 依存関係グラフ（既存）

```
engine → cache, loader, transpiler, runtime, ir
  ↓
transpiler → parser, registry
  ↓
runtime → registry, ir, error
  ↓
parser → error
  ↓
registry (独立層、外部依存なし) ★
```

**重要な発見**:
- **parser**: `crate::error::PastaError`のみに依存
- **registry**: クレート内部依存ゼロ（AST非依存設計）★parserと同一クレートに配置可能
- **transpiler**: `parser`のAST型と`registry`に依存
- **runtime**: `registry`と`error`に依存、parserには依存しない

**新構成での依存関係グラフ**:

```
pasta_rune (engine, transpiler, runtime, stdlib, cache, loader, error, ir)
  ↓
pasta_core (parser, registry) ★
```

- **pasta_core**: parser ⇄ registry（同一クレート内、相互独立）
- **pasta_rune**: transpilerとruntimeの両方がpasta_core::registryに依存

### テストファイル構成

- **統合テスト**: 28ファイル（`tests/*_test.rs`）
- **フィクスチャ**: `tests/fixtures/*.pasta`
- **共通ユーティリティ**: `tests/common/mod.rs`

**インポートパターン（変更前）**:
```rust
use pasta::PastaEngine;
use pasta::ir::{ContentPart, ScriptEvent};
use pasta::parser::{self, parse_str};
use pasta::transpiler::Transpiler2;
use pasta::registry::{SceneRegistry, WordDefRegistry};
```

**インポートパターン（変更後）**:
```rust
use pasta_rune::PastaEngine;
use pasta_rune::ir::{ContentPart, ScriptEvent};
use pasta_core::parser::{self, parse_str};
use pasta_rune::transpiler::Transpiler2;
use pasta_core::registry::{SceneRegistry, WordDefRegistry}; // ★pasta_coreから
```

### 命名規約とパターン

- **モジュール**: スネークケース（`scene_registry`, `code_generator`）
- **公開API**: `lib.rs`で`pub use`による再エクスポート
- **テストファイル**: `<feature>_test.rs`形式
- **エラー処理**: `Result<T, PastaError>`統一

---

## 2. Requirements Feasibility Analysis

### 技術要件マッピング

| 要件 | 必要な技術実装 | 現状のギャップ |
|------|---------------|---------------|
| R1: Cargoワークスペース導入 | ルートCargo.toml修正、`[workspace]`定義 | **Missing**: ワークスペース構成未定義 |
| R2: pasta_core分離 | `/crates/pasta_core/`、parser+registry | **Missing**: 独立クレートディレクトリ、★registry統合 |
| R3: pasta_rune分離 | `/crates/pasta_rune/`、pasta_core依存 | **Missing**: 独立クレートディレクトリ |
| R4: workspace.dependencies | 共通依存バージョン一元管理 | **Missing**: workspace.dependencies定義 |
| R5: ディレクトリ移行 | `src/parser/`+`src/registry/` → `/crates/pasta_core/src/` | **Gap**: 手動ファイル移動とCargo.toml作成 |
| R6: テスト継続性 | インポートパス修正、テスト実行成功 | **Constraint**: 28ファイル一括更新必要 |
| R7: ビルド互換性 | `cargo build --workspace`成功 | **Constraint**: 依存関係解決確認必要 |
| R8: ドキュメント更新 | structure.md, tech.md修正 | **Missing**: ワークスペース構成記述 |
| R9: API境界明確化 | `lib.rs`で公開API限定 | **Existing**: 既存パターン維持可能 |
| R10: 後方互換性 | 再エクスポートクレート検討 | **Research Needed**: 必要性判断 |

### 未解決事項と制約

**Research Needed**:
1. **error.rsの配置**:
   - 選択肢A: pasta_coreに配置（parserが使用）
   - 選択肢B: 両クレートで独立定義（ParseError vs PastaError）
   - 選択肢C: pasta_runeに配置しpasta_coreが依存（循環依存リスク）
   - **推奨**: 選択肢B（各クレートで責任範囲に応じたエラー型定義）

2. **registryモジュールの内部構造**:
   - 現在はAST非依存だが、parser/とディレクトリ構造をどう整理するか
   - **推奨**: pasta_core内で`parser/`と`registry/`を並列配置、lib.rsで両方を公開

**制約**:
- Pest文法ファイル（`grammar.pest`）はpasta_core内部に保持
- テストフィクスチャ（`.pasta`ファイル）はワークスペースレベル`/tests/fixtures/`に配置（両クレート共有）
- registryは両クレートから使用されるが、pasta_coreに配置することで単一方向依存を維持

### 複雑性シグナル

- **中程度の複雑性**: ファイル移動とインポートパス更新は機械的だが、依存関係検証が必要
- **低リスク**: registryのAST非依存設計により、parserと同一クレート配置が自然
- **高リスク**: テスト数が多く、一括変更時のリグレッションリスク

---

## 3. Implementation Approach Options

### Option A: Extend Existing Components ❌
**適用不可**: 新規クレート構造の作成であり、既存コンポーネントの拡張では対応不可能

---

### Option B: Create New Components ✅（推奨）

#### 新規作成するコンポーネント

1. **`/crates/pasta_core/`** ★変更点
   - **責任**: Pasta DSL解析（parser）＋共有型管理（registry）
   - **含むモジュール**:
     - `src/parser/mod.rs`, `ast.rs`, `grammar.pest`
     - `src/registry/mod.rs`, `scene_registry.rs`, `word_registry.rs` ★追加
     - `src/error.rs`（パース関連エラーのみ、またはエラー型全体）
   - **依存関係**: `pest`, `pest_derive`, `thiserror`
   - **公開API**: 
     - Parser: `parse_str()`, `parse_file()`, AST型（Statement, Expr, LabelDef等）
     - Registry: `SceneRegistry`, `WordDefRegistry`, `SceneEntry`, `WordEntry` ★追加

2. **`/crates/pasta_rune/`**
   - **責任**: トランスパイル、ランタイム、エンジン統合
   - **含むモジュール**:
     - `src/transpiler/`, `runtime/`, `stdlib/`
     - `src/engine.rs`, `cache.rs`, `loader.rs`, `error.rs`, `ir/`
   - **依存関係**: `pasta_core`, `rune`, `thiserror`, `glob`, `tracing`, `rand`, `futures`, `toml`, `fast_radix_trie`
   - **公開API**: `PastaEngine`, `ScriptEvent`, `PastaError`, Runtime関連型

#### 統合ポイント

```rust
// pasta_rune/src/transpiler/mod.rs
use pasta_core::ast::{Statement, Expr, GlobalSceneScope, PastaFile};
use pasta_core::registry::{SceneRegistry, WordDefRegistry}; // ★pasta_coreから
use pasta_core::{parse_str, parse_file};

// pasta_rune/src/runtime/scene.rs
use pasta_core::registry::SceneRegistry; // ★pasta_coreから
```

- **境界**: pasta_coreとpasta_runeの間（AST型とRegistry型をやりとり）
- **データフロー**: pasta_core（AST生成、Registry提供） → pasta_rune（AST消費、Registry使用、Runeコード生成）
- **責任分離**:
  - pasta_core: DSL文法知識、構文解析、シーン・単語型定義
  - pasta_rune: 意味解析、コード生成、実行、ビジネスロジック

#### 設計上の利点（registry統合 + error分離による）★

- ✅ **単一方向依存**: pasta_rune → pasta_core（循環依存なし）
- ✅ **registryの独立性維持**: AST非依存のため、parserと同居しても疎結合
- ✅ **共有型の一元管理**: SceneRegistry/WordDefRegistryをpasta_coreで定義、pasta_runeで使用
- ✅ **ビルド効率**: registryがpasta_coreに含まれるため、registry変更時にpasta_rune全体が再ビルド不要（依存が明確）
- ✅ **言語非依存性**: error分離により、将来Lua等への切り替えが容易（pasta_core不変）
- ✅ **責任境界の明確さ**: 
  - pasta_core: 言語非依存層（ParseError）
  - pasta_rune: Rune言語特化層（PastaError）

#### Trade-offs

- ✅ 明確な責任分離（コア型定義 vs 実行エンジン）
- ✅ pasta_coreの再利用可能性向上（他ツールでAST解析＋Registry使用可能）
- ✅ registryのAST非依存設計を活かした自然な配置
- ❌ 初期セットアップ工数（ディレクトリ作成、Cargo.toml記述）
- ❌ インポートパス変更による既存テスト修正

---

### Option C: Hybrid Approach（検討不要）

本仕様では完全な構造変更が目的であり、段階的移行のハイブリッドは該当しない。ただし、**実装フェーズでの段階的検証**は推奨される：

1. **Phase 1**: ワークスペース構成作成、ファイル移動
2. **Phase 2**: Cargo.toml記述、依存関係定義
3. **Phase 3**: インポートパス修正（parser/registryテスト）
4. **Phase 4**: インポートパス修正（統合テスト）
5. **Phase 5**: ドキュメント更新、最終検証

---

## 4. Requirement-to-Asset Map

| 要件 | 既存アセット | ギャップ | タグ |
|------|-------------|---------|------|
| R1: Cargoワークスペース | `Cargo.toml`（`[workspace]`空） | ワークスペース定義記述 | **Missing** |
| R2: pasta_core分離 | `src/parser/`（3ファイル）＋`src/registry/`（3ファイル）★ | `/crates/pasta_core/`作成、Cargo.toml記述 | **Missing** |
| R3: pasta_rune分離 | `src/`残り全モジュール | `/crates/pasta_rune/`作成、依存関係定義 | **Missing** |
| R4: workspace.dependencies | `Cargo.toml` [dependencies] | `[workspace.dependencies]`移行 | **Missing** |
| R5: ディレクトリ移行 | `src/parser/`, `src/registry/`, `tests/` | ファイル移動、パス修正 | **Gap** |
| R6: テスト継続性 | 28テストファイル | `use pasta::` → `use pasta_core::{parser, registry}`, `use pasta_rune::`修正 | **Constraint** |
| R7: ビルド互換性 | 既存ビルドスクリプトなし | ワークスペースビルド検証 | **Constraint** |
| R8: ドキュメント更新 | `.kiro/steering/structure.md`, `tech.md` | ワークスペース構成追記 | **Missing** |
| R9: API境界明確化 | `src/lib.rs`再エクスポートパターン | 新クレートで同様パターン適用 | **Existing** |
| R10: 後方互換性 | なし（未公開） | 再エクスポートクレート不要と判断 | **Research Needed** → **Decision: 不要** |

---

## 5. Implementation Complexity & Risk

### 全体評価

- **Effort**: **M（Medium, 3-7日）**
  - ディレクトリ構造作成: 0.5日
  - Cargo.toml記述: 0.5日
  - ファイル移動（parser+registry）: 0.5日 ★registry追加
  - インポートパス修正（28テスト + src/）: 2日
  - ビルドエラー解決・検証: 1日
  - ドキュメント更新: 0.5日
  - 最終テスト・検証: 1日

- **Risk**: **Low-Medium** ★リスク低減
  - **既知のパターン**: Cargoワークスペースは標準機能、ガイダンス豊富
  - **明確なモジュール境界**: registry統合により依存関係が単純化（pasta_rune → pasta_core）
  - **リスク要因**:
    - 28テストファイルの一括修正でのタイポリスク
    - registryのAST非依存設計により、配置変更リスクは低い ★
    - Cargo依存解決の予期しない競合（可能性低いが検証必要）

### 要件別詳細

| 要件 | Effort | Risk | 理由 |
|------|--------|------|------|
| R1-R4 | S | Low | Cargo標準機能、テンプレート利用可能 |
| R5 | S | Low | 機械的ファイル移動、Git履歴保持、★registry追加だが独立層 |
| R6 | M | Medium | 28ファイル一括修正、リグレッションテスト必須 |
| R7 | S | Low | 既存ビルド成功前提、依存解決は自動 |
| R8 | S | Low | Markdown文書更新のみ |
| R9 | S | Low | 既存パターン踏襲 |
| R10 | S | Low | 再エクスポート不要と判断、README更新のみ |

---

## 6. Recommendations for Design Phase

### 推奨実装アプローチ

**Option B（新規クレート作成）** を採用し、以下の設計判断を実施：

1. **クレート構成** ★変更
   - `/crates/pasta_core/`: パーサー層＋レジストリ層
   - `/crates/pasta_rune/`: トランスパイラ＋ランタイム＋エンジン層

2. **registryモジュール配置** ★確定
   - `pasta_core`に配置（parserと並列、AST非依存を活かす）
   - ディレクトリ構造: `pasta_core/src/parser/`, `pasta_core/src/registry/`

3. **error.rs配置** ★確定
   - pasta_core: ParseError（パース関連エラー）
   - pasta_rune: PastaError（実行時エラー全般）

4. **テストフィクスチャ配置** ★確定
   - ワークスペースレベル: `/tests/fixtures/*.pasta`（両クレート共有）
   - 共通ユーティリティ: `/tests/common/mod.rs`でパス解決関数等を提供

### 設計フェーズで決定すべき事項

1. **Cargo.toml詳細構成**:
   - `workspace.dependencies`の正確なバージョン指定
   - `workspace.package`での共通メタデータ（authors, license, edition）

2. **公開API境界の詳細** ★確定
   - `pasta_core/src/lib.rs`: `pub mod parser;`, `pub mod registry;`, `pub mod error;`
   - `pasta_rune/src/lib.rs`: 
     - `pub use pasta_core as core;` （コアへの間接公開、`core`という名前で）
     - 言語層API（`pub use engine::PastaEngine;`等）
   - 外部ユーザーは`use pasta_rune::core::parser`で必要に応じてアクセス可能

3. **テスト戦略**:
   - パーサー＋レジストリテストの配置（pasta_core/tests/）
   - 統合テストの配置（pasta_rune/tests/）
   - ワークスペースレベルテストユーティリティ（tests/common/）の設計

4. **ドキュメント構成**:
   - 各クレートの`README.md`作成
   - ルート`README.md`でワークスペース全体を説明
   - 移行ガイドの記載

5. **examples/ディレクトリの配置**:
   - ワークスペースレベルに保持 or pasta_runeに移動

---

## Summary

### 分析結果サマリー

- **実装アプローチ**: Option B（新規クレート作成）推奨
- **クレート構成変更**: pasta_core（parser + registry）、pasta_rune（transpiler + runtime）★
- **Effort**: M（3-7日）、主にインポートパス一括修正とテスト検証
- **Risk**: Low-Medium（registry統合により依存関係が単純化、技術リスク低減）★
- **主なギャップ**: ワークスペース構成未定義、クレートディレクトリ未作成、28テストファイルのインポートパス修正必要

### 設計上の利点（pasta_core構成）★

1. **単一方向依存**: pasta_rune → pasta_coreのみ（循環依存なし）
2. **registryの独立性維持**: AST非依存設計により、parserと同居しても疎結合
3. **共有型の一元管理**: パーサーとレジストリを統一クレートで提供
4. **明確な責任境界**: コア型定義（pasta_core） vs ビジネスロジック（pasta_rune）

### 次のステップ

ギャップ分析完了。設計フェーズへ進み、以下を詳細化してください：

1. Cargo.tomlの正確な構成（workspace.dependencies, workspace.package）
2. 公開API境界の詳細定義（lib.rsの内容、registry公開範囲）
3. error.rsの最適配置決定（両クレート独立 vs 統一）
4. テストファイル移行戦略の詳細

**次のコマンド**:
```
/kiro-spec-design workspace-migration
```

または、要件を自動承認して即座に設計フェーズへ進む場合：
```
/kiro-spec-design workspace-migration -y
```

---

## 1. Current State Investigation

### ディレクトリ構造とモジュール配置

```
src/
├── lib.rs                    # クレートエントリー、公開API定義
├── parser/                   # パーサー層（Pest文法、AST）
│   ├── mod.rs
│   ├── ast.rs
│   └── grammar.pest
├── transpiler/               # トランスパイラ層（AST→Rune）
│   ├── mod.rs
│   ├── code_generator.rs
│   ├── context.rs
│   └── error.rs
├── registry/                 # 共有レジストリ（AST非依存）
│   ├── mod.rs
│   ├── scene_registry.rs
│   └── word_registry.rs
├── runtime/                  # ランタイム層（Rune VM実行）
│   ├── mod.rs
│   ├── generator.rs
│   ├── variables.rs
│   ├── scene.rs
│   ├── words.rs
│   └── random.rs
├── stdlib/                   # Pasta標準ライブラリ
├── engine.rs                 # 統合API
├── cache.rs                  # パースキャッシュ
├── loader.rs                 # ディレクトリローダー
├── error.rs                  # エラー型
└── ir/                       # IR出力型（ScriptEvent）
```

### 依存関係グラフ（既存）

```
engine → cache, loader, transpiler, runtime, ir
  ↓
transpiler → parser, registry
  ↓
runtime → registry, ir, error
  ↓
parser → error
  ↓
registry (独立層、外部依存なし)
```

**重要な発見**:
- **parser**: `crate::error::PastaError`のみに依存
- **transpiler**: `parser`のAST型と`registry`に依存
- **runtime**: `registry`と`error`に依存、parserには依存しない
- **registry**: クレート内部依存ゼロ（AST非依存設計）

### テストファイル構成

- **統合テスト**: 28ファイル（`tests/*_test.rs`）
- **フィクスチャ**: `tests/fixtures/*.pasta`
- **共通ユーティリティ**: `tests/common/mod.rs`

**インポートパターン**:
```rust
use pasta::PastaEngine;
use pasta::ir::{ContentPart, ScriptEvent};
use pasta::parser::{self, parse_str};
use pasta::transpiler::Transpiler2;
use pasta::registry::{SceneRegistry, WordDefRegistry};
```

### 命名規約とパターン

- **モジュール**: スネークケース（`scene_registry`, `code_generator`）
- **公開API**: `lib.rs`で`pub use`による再エクスポート
- **テストファイル**: `<feature>_test.rs`形式
- **エラー処理**: `Result<T, PastaError>`統一

---

## 2. Requirements Feasibility Analysis

### 技術要件マッピング

| 要件 | 必要な技術実装 | 現状のギャップ |
|------|---------------|---------------|
| R1: Cargoワークスペース導入 | ルートCargo.toml修正、`[workspace]`定義 | **Missing**: ワークスペース構成未定義 |
| R2: pasta_parser分離 | `/crates/pasta_parser/`、pest依存関係のみ | **Missing**: 独立クレートディレクトリ |
| R3: pasta_rune分離 | `/crates/pasta_rune/`、pasta_parser依存 | **Missing**: 独立クレートディレクトリ |
| R4: workspace.dependencies | 共通依存バージョン一元管理 | **Missing**: workspace.dependencies定義 |
| R5: ディレクトリ移行 | `src/parser/` → `/crates/pasta_parser/src/` | **Gap**: 手動ファイル移動とCargo.toml作成 |
| R6: テスト継続性 | インポートパス修正、テスト実行成功 | **Constraint**: 28ファイル一括更新必要 |
| R7: ビルド互換性 | `cargo build --workspace`成功 | **Constraint**: 依存関係解決確認必要 |
| R8: ドキュメント更新 | structure.md, tech.md修正 | **Missing**: ワークスペース構成記述 |
| R9: API境界明確化 | `lib.rs`で公開API限定 | **Existing**: 既存パターン維持可能 |
| R10: 後方互換性 | 再エクスポートクレート検討 | **Research Needed**: 必要性判断 |

### 未解決事項と制約

**Research Needed**:
1. **registryモジュールの配置先**:
   - 選択肢A: `pasta_parser`に配置（transpilerが依存）
   - 選択肢B: `pasta_rune`に配置（runtimeも依存）
   - 選択肢C: 独立クレート`pasta_registry`を作成
   - **推奨**: 選択肢B（runtimeとtranspilerの両方がrune側にあるため）

2. **error.rsの配置**:
   - parserとruntimeの両方が依存
   - **推奨**: 両クレートで独立定義、または`pasta_rune`で定義しpasta_parserが依存

3. **後方互換性の必要性**:
   - 現在publish = trueだが、まだcrates.io未公開
   - **推奨**: 破壊的変更を許容し、README.mdに移行ガイドを記載

**制約**:
- Pest文法ファイル（`grammar.pest`）はpasta_parser内部に保持
- テストフィクスチャ（`.pasta`ファイル）はpasta_runeに配置（統合テストが使用）
- 既存の28テストファイルすべてが成功する必要あり

### 複雑性シグナル

- **中程度の複雑性**: ファイル移動とインポートパス更新は機械的だが、依存関係検証が必要
- **低リスク**: 既存のモジュール境界がクレート分割設計と一致している
- **高リスク**: テスト数が多く、一括変更時のリグレッションリスク

---

## 3. Implementation Approach Options

### Option A: Extend Existing Components ❌
**適用不可**: 新規クレート構造の作成であり、既存コンポーネントの拡張では対応不可能

---

### Option B: Create New Components ✅（推奨）

#### 新規作成するコンポーネント

1. **`/crates/pasta_parser/`**
   - **責任**: Pasta DSL解析、AST生成
   - **含むモジュール**:
     - `src/parser/mod.rs`, `ast.rs`, `grammar.pest`
     - `src/error.rs`（パース関連エラーのみ）
   - **依存関係**: `pest`, `pest_derive`, `thiserror`
   - **公開API**: `parse_str()`, `parse_file()`, AST型（Statement, Expr, LabelDef等）

2. **`/crates/pasta_rune/`**
   - **責任**: トランスパイル、ランタイム、エンジン統合
   - **含むモジュール**:
     - `src/transpiler/`, `runtime/`, `registry/`, `stdlib/`
     - `src/engine.rs`, `cache.rs`, `loader.rs`, `error.rs`, `ir/`
   - **依存関係**: `pasta_parser`, `rune`, `thiserror`, `glob`, `tracing`, `rand`, `futures`, `toml`, `fast_radix_trie`
   - **公開API**: `PastaEngine`, `ScriptEvent`, `PastaError`, Runtime関連型

#### 統合ポイント

```rust
// pasta_rune/src/transpiler/mod.rs
use pasta_parser::ast::{Statement, Expr, GlobalSceneScope, PastaFile};
use pasta_parser::{parse_str, parse_file};
```

- **境界**: パーサーとトランスパイラーの間（AST型のみをやりとり）
- **データフロー**: pasta_parser（AST生成） → pasta_rune（AST消費、Runeコード生成）
- **責任分離**:
  - pasta_parser: DSL文法知識、構文解析
  - pasta_rune: 意味解析、コード生成、実行

#### Trade-offs

- ✅ 明確な責任分離（パーサー vs 実行エンジン）
- ✅ pasta_parserの再利用可能性向上（他ツールでもAST解析可能）
- ✅ ビルド時間改善の可能性（パーサー変更時にruneレイヤーが再ビルド不要）
- ❌ 初期セットアップ工数（ディレクトリ作成、Cargo.toml記述）
- ❌ インポートパス変更による既存テスト修正

---

### Option C: Hybrid Approach（検討不要）

本仕様では完全な構造変更が目的であり、段階的移行のハイブリッドは該当しない。ただし、**実装フェーズでの段階的検証**は推奨される：

1. **Phase 1**: ワークスペース構成作成、ファイル移動
2. **Phase 2**: Cargo.toml記述、依存関係定義
3. **Phase 3**: インポートパス修正（パーサー層テスト）
4. **Phase 4**: インポートパス修正（統合テスト）
5. **Phase 5**: ドキュメント更新、最終検証

---

## 4. Requirement-to-Asset Map

| 要件 | 既存アセット | ギャップ | タグ |
|------|-------------|---------|------|
| R1: Cargoワークスペース | `Cargo.toml`（`[workspace]`空） | ワークスペース定義記述 | **Missing** |
| R2: pasta_parser分離 | `src/parser/`（3ファイル） | `/crates/pasta_parser/`作成、Cargo.toml記述 | **Missing** |
| R3: pasta_rune分離 | `src/`残り全モジュール | `/crates/pasta_rune/`作成、依存関係定義 | **Missing** |
| R4: workspace.dependencies | `Cargo.toml` [dependencies] | `[workspace.dependencies]`移行 | **Missing** |
| R5: ディレクトリ移行 | `src/`, `tests/` | ファイル移動、パス修正 | **Gap** |
| R6: テスト継続性 | 28テストファイル | `use pasta::` → `use pasta_parser::`, `use pasta_rune::`修正 | **Constraint** |
| R7: ビルド互換性 | 既存ビルドスクリプトなし | ワークスペースビルド検証 | **Constraint** |
| R8: ドキュメント更新 | `.kiro/steering/structure.md`, `tech.md` | ワークスペース構成追記 | **Missing** |
| R9: API境界明確化 | `src/lib.rs`再エクスポートパターン | 新クレートで同様パターン適用 | **Existing** |
| R10: 後方互換性 | なし（未公開） | 再エクスポートクレート不要と判断 | **Research Needed** → **Decision: 不要** |

---

## 5. Implementation Complexity & Risk

### 全体評価

- **Effort**: **M（Medium, 3-7日）**
  - ディレクトリ構造作成: 0.5日
  - Cargo.toml記述: 0.5日
  - ファイル移動: 0.5日
  - インポートパス修正（28テスト + src/）: 2日
  - ビルドエラー解決・検証: 1日
  - ドキュメント更新: 0.5日
  - 最終テスト・検証: 1日

- **Risk**: **Medium**
  - **既知のパターン**: Cargoワークスペースは標準機能、ガイダンス豊富
  - **明確なモジュール境界**: 既存の依存関係グラフがクレート分割と一致
  - **リスク要因**:
    - 28テストファイルの一括修正でのタイポリスク
    - registryモジュール配置の循環依存リスク（設計で回避可能）
    - Cargo依存解決の予期しない競合（可能性低いが検証必要）

### 要件別詳細

| 要件 | Effort | Risk | 理由 |
|------|--------|------|------|
| R1-R4 | S | Low | Cargo標準機能、テンプレート利用可能 |
| R5 | S | Low | 機械的ファイル移動、Git履歴保持 |
| R6 | M | Medium | 28ファイル一括修正、リグレッションテスト必須 |
| R7 | S | Low | 既存ビルド成功前提、依存解決は自動 |
| R8 | S | Low | Markdown文書更新のみ |
| R9 | S | Low | 既存パターン踏襲 |
| R10 | S | Low | 再エクスポート不要と判断、README更新のみ |

---

## 6. Recommendations for Design Phase

### 推奨実装アプローチ

**Option B（新規クレート作成）** を採用し、以下の設計判断を実施：

1. **クレート構成**:
   - `/crates/pasta_parser/`: パーサー層
   - `/crates/pasta_rune/`: トランスパイラ＋ランタイム＋エンジン層

2. **registryモジュール配置**:
   - `pasta_rune`に配置（transpilerとruntimeの両方が同一クレート内）

3. **error.rs配置**:
   - 両クレートで独立定義：
     - `pasta_parser::error::ParseError`
     - `pasta_rune::error::PastaError`（既存）
   - または`pasta_rune::error`を`pasta_parser`が依存（循環依存回避確認必要）

4. **後方互換性**:
   - 再エクスポートクレート作成せず
   - `README.md`に破壊的変更と移行ガイドを明記

### 設計フェーズで決定すべき事項

1. **Cargo.toml詳細構成**:
   - `workspace.dependencies`の正確なバージョン指定
   - `workspace.package`での共通メタデータ（authors, license, edition）

2. **公開API境界の詳細**:
   - `pasta_parser/src/lib.rs`で公開する型とモジュール
   - `pasta_rune/src/lib.rs`で公開する型とモジュール
   - 内部モジュールの`pub(crate)`適用範囲

3. **テスト戦略**:
   - パーサーテストを`pasta_parser/tests/`に分離するか
   - 統合テストはすべて`pasta_rune/tests/`に配置

4. **ドキュメント構成**:
   - 各クレートの`README.md`作成
   - ルート`README.md`でワークスペース全体を説明

### 設計フェーズで実施する技術調査

以下の項目は設計時に詳細検討：

1. **error.rsの最適配置**:
   - 両クレート独立 vs pasta_runeに統一 vs 新規pasta_commonクレート
   - 各選択肢の循環依存リスク評価

2. **テストフィクスチャ共有方法**:
   - `tests/fixtures/`の配置先（pasta_runeか、ワークスペースルートか）
   - 複数クレートでフィクスチャ共有する場合のパス解決

3. **examples/ディレクトリの扱い**:
   - pasta_runeに移動 vs ワークスペースルートに保持
   - サンプルコードでのインポートパス

---

## Summary

### 分析結果サマリー

- **実装アプローチ**: Option B（新規クレート作成）推奨
- **Effort**: M（3-7日）、主にインポートパス一括修正とテスト検証
- **Risk**: Medium、既存パターンに従うため技術リスクは低いが、テスト数が多くリグレッションリスクあり
- **主なギャップ**: ワークスペース構成未定義、クレートディレクトリ未作成、28テストファイルのインポートパス修正必要

### 次のステップ

ギャップ分析完了。設計フェーズへ進み、以下を詳細化してください：

1. Cargo.tomlの正確な構成（workspace.dependencies, workspace.package）
2. 公開API境界の詳細定義（lib.rsの内容）
3. error.rsとregistryの最適配置決定
4. テストファイル移行戦略の詳細

**次のコマンド**:
```
/kiro-spec-design workspace-migration
```

または、要件を自動承認して即座に設計フェーズへ進む場合：
```
/kiro-spec-design workspace-migration -y
```
