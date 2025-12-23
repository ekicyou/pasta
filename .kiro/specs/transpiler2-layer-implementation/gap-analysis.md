# Implementation Gap Analysis: transpiler2-layer-implementation

## Analysis Date
2025-12-23

## Executive Summary

transpiler2実装は**中程度の複雑度（M: 3-7日）**・**中リスク**の機能です。以下の理由から**Option B（新規コンポーネント）を推奨**します：

- **AST型の根本的な差異**: parser2（新3層スコープ）vs. parser（既存フラット構造）→ 共存不可
- **既存パターンの再利用性**: TranspileContext、SceneRegistry、WordDefRegistry は parser2にも適用可能
- **段階的統合**: レガシーtranspilerと完全に独立、段階的置き換え可能
- **明確なレイヤー分離**: tech.md（レイヤードアーキテクチャ）を完全準拠

---

## 1. Current State Investigation

### Key Assets - Existing Transpiler

| Component | Location | Purpose | Size |
|-----------|----------|---------|------|
| **Transpiler** | `src/transpiler/mod.rs` | 2パス変換エンジン | 948行 |
| **SceneRegistry** | `src/transpiler/scene_registry.rs` | シーン登録・ID管理 | 268行 |
| **WordDefRegistry** | `src/transpiler/word_registry.rs` | 単語定義登録 | 207行 |

### Key Assets - Parser2 (Input)

| Component | Location | Purpose |
|-----------|----------|---------|
| **Parser2 AST** | `src/parser2/ast.rs` | 新3層スコープAST（624行） |
| **Parser2 Module** | `src/parser2/mod.rs` | パーサー実装（~200行） |

### Existing Transpiler Patterns

**2パス変換戦略**:
```
Pass 1 (transpile_pass1):
  - Iterate PastaFile.scenes (existing parser)
  - Register in SceneRegistry
  - Register in WordDefRegistry
  - Generate Rune module per scene

Pass 2 (transpile_pass2):
  - Generate __pasta_trans2__::scene_selector()
  - Generate pasta::call() / pasta::jump()
  - Generate ID→function_path mapping
```

**TranspileContext**:
- `local_functions`: Vec<String>
- `global_functions`: Vec<String> (stdlib + user-defined)
- `current_module`: String (scene lookup key)

**Naming Convention**:
- Scene modules: `{scene_name}_{counter}::`（e.g., `会話_1::`）
- Scene start function: `__start__(ctx, args)` (参照: test_combined_code.rn:31)
- Word keys: `"word_name"` (global) or `":module:word_name"` (local)

### Integration Surfaces

| Interface | Usage | Notes |
|-----------|-------|-------|
| `lib.rs` | `pub mod transpiler;` | レガシーtranspiler公開 |
| `error.rs` | `PastaError` enum | 統一エラー型 |
| `ir.rs` | `ScriptEvent` enum | Runtime IR出力 |

---

## 2. Requirements Feasibility Analysis

### Mapping Requirements → Technical Needs

| Req # | Requirement | Technical Need | Feasibility | Gap |
|-------|-------------|-----------------|-------------|-----|
| 1 | Module Independence | New `src/transpiler2/mod.rs` | ✅ Easy | None |
| 2 | AST-to-Rune Gen | Parse 3-layer scope + generate code | ✅ Medium | **Scope handling** |
| 3 | Call Resolution | Scene lookup + random selection | ✅ Easy | SceneRegistry reusable |
| 4 | Symbol Resolution | Phase 1 registration loop | ✅ Easy | Existing pattern |
| 5 | Variable Scope | Local/Global/System distinction | ✅ Medium | **Runtime contract** |
| 6 | Expression Eval | Numeric + string + binary ops | ✅ Medium | **Type system** |
| 7 | Error Handling | TranspileError type | ⚠️ New Type | **Error enum design** |
| 8 | Rune Compatibility | Generate valid Rune 0.14 code | ✅ Medium | Pest error handling |
| 9 | Two-Pass Architecture | Phase 1/Phase 2 separation | ✅ Easy | Existing pattern |
| 10 | Test Coverage | 10 test categories | ✅ Medium | **fixture preparation** |

### Gap Assessment

#### Critical Gaps (Must Research/Design)

1. **Parser2 AST型 vs. Existing Transpiler期待型**
   - Parser2: FileScope / GlobalSceneScope / LocalSceneScope（3層）
   - Existing: PastaFile / SceneDef / SceneDef.local_scenes（2層）
   - **問題**: Scope型変換ロジックの設計が必須

2. **TranspileError型定義**
   - 既存: PastaError（解析層で定義）
   - 新規: TranspileError（transpiler2レイヤー）
   - **問題**: 既存PastaErrorと一貫性を保つ設計

3. **Runtime Compatibility**
   - Requirement 8: "Rune 0.14 VM実行可能"
   - **不明確**: transpiler2が生成するRuneコードがExisting Runtime層（`src/runtime/`）で実行可能か

#### Medium Gaps (Design Phase で詳細化)

1. **Expression Type System**
   - Requirement 6: 式の結果を"Data型"として扱う
   - **未決定**: Data型の具体的構造（値 + メタデータ）

2. **Variable Storage Backend**
   - Requirement 5: System変数（`＄＊＊var`）を永続化対象と標識
   - **未実装**: 永続化の具体的メカニズム（Engine層未実装）

3. **Fixture Preparation**
   - Requirement 10: transpiler2専用fixtureを準備
   - **決定**: parser2テスト済みfixtureを流用（`tests/fixtures/parser2/*.pasta`、`comprehensive_control_flow2.pasta`）
   - **追加作業**: transpiler固有機能テストのみ新規fixture作成（推定5-10ファイル）

---

## 3. Implementation Approach Options

### Option A: Extend Existing Transpiler

**Rationale**: 既存transpiler/mod.rsに parser2 AST型対応コードを追加

**Advantages**:
- ✅ ファイル数最小化（mod.rsのみ拡張）
- ✅ 既存TranspileContextを再利用可能
- ✅ Pass 1/Pass 2パターン継承

**Disadvantages**:
- ❌ Parser AST（parser::PastaFile）と Parser2 AST（parser2::PastaFile）の共存：マッチング処理が複雑
- ❌ 既存mod.rsが948行→さらに増加、単一責任原則崩れ
- ❌ 既存テストへのリグレッションリスク（if文・match パターン追加）

**Estimated Effort**: M (3-7日)
**Estimated Risk**: High (既存ロジック変更リスク)

---

### Option B-改改: Create New Transpiler2 + Shared Registry Module ⭐ **RECOMMENDED & APPROVED**

**Rationale**: `src/transpiler2/`を新規作成し、既存transpilerと独立。ただし、SceneRegistry/WordDefRegistry/SceneTable/WordTableは共有モジュール`src/registry/`に統合して再利用。

**Architecture**:
```
src/
├── registry/              # 新規：共有レジストリモジュール
│   ├── mod.rs            # 公開API
│   ├── scene_registry.rs # SceneRegistry（transpilerから移動）
│   ├── word_registry.rs  # WordDefRegistry（transpilerから移動）
│   ├── scene_table.rs    # SceneTable（runtimeから移動）
│   └── word_table.rs     # WordTable（runtimeから移動）
├── transpiler/            # Transpiler struct のみ（registry import）
│   └── mod.rs
├── transpiler2/           # 新規
│   ├── mod.rs            # Transpiler2 struct + public API
│   ├── context.rs        # TranspileContext2（parser2対応）
│   ├── symbol_resolver.rs # Symbol resolution (parser2専用)
│   └── code_generator.rs # AST → Rune code generation
└── runtime/               # Generator/Variables等のみ（registry import）
    ├── mod.rs
    ├── generator.rs
    └── variables.rs
```

**Advantages**:
- ✅ **完全な独立性**: parser/transpiler と parser2/transpiler2 は完全分離
- ✅ **レジストリ共有**: SceneRegistry/WordDefRegistry/SceneTable/WordTableはAST型に依存せず、100%再利用可能
- ✅ **コード重複0**: Registry/Tableの重複実装不要
- ✅ **明確な名前空間**: `pasta::registry::*` として独立管理
- ✅ **リグレッション0**: 既存テストへの影響なし
- ✅ **段階的置き換え**: 将来 `transpiler` 削除時も `registry` は継続使用可能
- ✅ **テスト隔離**: transpiler2テストが既存テストと独立

**Disadvantages**:
- ❌ ファイル移動作業（scene_registry.rs/word_registry.rs/scene_table.rs/word_table.rsの4ファイル）
- ❌ import文の更新（既存transpiler/runtimeコードのuse文修正）

**Estimated Effort**: M (4-5日) - レジストリ移動で1日節約
**Estimated Risk**: Low-Medium (既存Registry完全再利用 → リスク大幅軽減)

---

### Option C: Hybrid - Shared Registry + New Transpiler2

**Rationale**: SceneRegistry/WordDefRegistry を共有しつつ、transpiler2ロジックは独立

**Architecture**:
```
src/transpiler2/
├── mod.rs                 # Transpiler2 (parser2専用)
├── context.rs             # TranspileContext2
└── code_generator.rs      # Code generation

src/transpiler/             # 既存
├── scene_registry.rs       # 共有 ← transpiler2からもインポート
└── word_registry.rs        # 共有 ← transpiler2からもインポート
```

**Advantages**:
- ✅ コード重複最小化（SceneRegistry は単一実装）
- ✅ ファイル数中程度（3-4新規ファイル）

**Disadvantages**:
- ❌ 設計複雑性増加（共有Registry の parser/parser2 両対応）
- ❌ Registry型が parser AST 前提 → parser2対応に改修必要
- ❌ 将来のlegacy削除時に共有Registry の分離が必須

**Estimated Effort**: M (5-6日)
**Estimated Risk**: Medium-High (共有設計の複雑性)

---

## 4. Recommended Approach: Option B-改改 ✅ **APPROVED**

### Rationale

1. **Specification準拠**: `.kiro/steering/tech.md` - "レイヤー構成...レイヤー分離原則"
2. **マイグレーション安全性**: Requirement 1 - "レガシーとのコンパイルエラーを引き起こさない"
3. **レジストリ再利用**: SceneRegistry/WordDefRegistry/SceneTable/WordTableはAST型に依存せず完全再利用可能
4. **テスト隔離**: Requirement 10の10カテゴリテストが既存テストと独立に実行可能
5. **段階的統合**: parser2完了直後に transpiler2着手可能、将来レガシー削除時にmod transpiler2をpub mod transpilerに置き換え可能

### Key Design Decisions

#### 1. Transpiler2 Module Structure
```rust
// src/transpiler2/mod.rs
pub struct Transpiler2;
impl Transpiler2 {
    pub fn transpile_pass1(
        file: &parser2::PastaFile,
        scene_registry: &mut SceneRegistry2,
        word_registry: &mut WordDefRegistry2,
        writer: &mut dyn Write
    ) -> Result<(), TranspileError> { ... }
    
    pub fn transpile_pass2(
        registry: &SceneRegistry2,
        writer: &mut dyn Write
    ) -> Result<(), TranspileError> { ... }
}

pub fn transpile_str(source: &str) -> Result<String, TranspileError> { ... }
pub fn transpile_file(path: &Path) -> Result<String, TranspileError> { ... }
```

#### 2. TranspileError Type
```rust
// src/error.rs に追加（または transpiler2/error.rs）
#[derive(Error, Debug)]
pub enum TranspileError {
    #[error("Invalid AST at {location}: {message}")]
    InvalidAst { location: String, message: String },
    
    #[error("Undefined symbol: {symbol}")]
    UndefinedSymbol { symbol: String },
    
    #[error("Type mismatch at {location}: expected {expected}, got {got}")]
    TypeMismatch { location: String, expected: String, got: String },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}
```

#### 3. Registry Module Design

**共有レジストリモジュール `src/registry/`**

| Component | Purpose | Reusability |
|-----------|---------|-------------|
| **SceneRegistry** | Transpile時のシーン登録（AST型非依存） | ✅ 100% transpiler/transpiler2共用 |
| **WordDefRegistry** | Transpile時の単語定義登録（AST型非依存） | ✅ 100% transpiler/transpiler2共用 |
| **SceneTable** | Runtime時のシーン検索・選択 | ✅ 100% 既存Runtime層と共用 |
| **WordTable** | Runtime時の単語検索・選択 | ✅ 100% 既存Runtime層と共用 |

**設計**: 既存Registry/Tableを `src/registry/` に移動し、transpiler/transpiler2/runtimeから `use crate::registry::*;` で共用

#### 4. Scope Handling Logic

**Parser2 AST構造** (3層):
```
PastaFile
  ├─ FileScope (attributes, file-level words)
  └─ GlobalSceneScope[] (global scene definitions)
      ├─ GlobalSceneScope.name (scene name)
      ├─ GlobalSceneScope.attrs
      ├─ GlobalSceneScope.words (local words)
      └─ LocalSceneScope[] (nested local scenes)
          ├─ LocalSceneScope.name
          └─ LocalSceneScope.items (actions)
```

**Transpiler2 Phase 1処理**:
```rust
for global_scene in file.global_scenes {
    // 1. Register global scene
    let global_id = registry.register_global(&global_scene.name, ...);
    
    // 2. Register local scenes within this global
    for local_scene in &global_scene.local_scenes {
        let local_id = registry.register_local(
            &global_scene.name,
            &local_scene.name,
            ...
        );
    }
    
    // 3. Generate Rune module for global scene
    generate_global_scene_module(&global_scene, ...)?;
}
```

---

## 5. Research Items for Design Phase

### High Priority (Must Research)

1. **Parser2 ActionLine → Rune yield** conversion
   - Parser2 AST での ActionLine 型定義 を確認（ast.rs line ??? ）
   - 既存transpiler での Statement → yield 変換ロジック を参考（mod.rs line ??? ）
   - →Design で "3.2 AST-to-Rune Codegen" セクションを詳細化

2. **TranspileError 統一設計**
   - 既存 PastaError の設計方針 を確認（error.rs）
   - transpiler層でのエラーハンドリング慣例 を確認
   - →Design で error type hierarchy を定義

3. **System Variable Persistence**
   - Engine層での変数永続化機構 を確認（engine.rs）
   - Runtime層での System Variable storage backend を確認
   - →Design で "5 Variable Scope" の実装戦略を詳細化

### Medium Priority (Design で詳細化)

4. **Rune Code Quality**
   - transpiler が生成する Rune コード の例 を test_combined_code.rn から抽出
   - transpiler2 の出力仕様書 を design で定義

5. **Fixture Strategy**
   - parser2 test fixtures (`tests/fixtures/...`) の一覧確認
   - transpiler2 の新 fixtures が必要か判定

---

## 6. Complexity and Risk Assessment

### Effort Estimation

| Phase | Task | Days | Notes |
|-------|------|------|-------|
| **Design** | Architecture + error types + scope logic | 1-2 | Research items解決 |
| **Implementation** | mod.rs + context + registries + codegen | 3-4 | ~800行Rust code |
| **Testing** | 10カテゴリテスト + fixtures | 1-2 | parser2 fixtures流用 |
| **Total** | | **5-8日** | M (medium) |

### Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **AST Mismatch** | Medium | High | Design phase で scope conversion を完全設計 |
| **Rune Codegen Bug** | Medium | High | 生成コードのunit test → Runtime 実行テスト |
| **Symbol Resolution** | Low | Medium | Phase 1 registration ロジックを厳密に仕様化 |
| **Compatibility** | Low | Medium | E2E integration test で既存Runtime 連携確認 |

**Overall Risk: Medium** (新規実装だが、既存パターン踏襲で軽減)

---

## 7. Recommendations for Design Phase

### Preferred Approach: Option B

**設計フェーズでの優先事項**:

1. **TranspileError 型定義**（即座）
   - `src/error.rs` に追加 vs. `src/transpiler2/error.rs` に分離か決定
   - error.rs の既存パターン を踏襲

2. **Scope Conversion Logic**（詳細設計）
   - Parser2 3層 → Rune module structure の完全マッピング
   - local scene の scope rule の明確化（親探索ルール）

3. **Code Generation Templates**（テンプレート化）
   - Global scene module template（既存 transpiler から抽出）
   - Local scene nested function template
   - Symbol resolution code generation

4. **Symbol Table Design**
   - Global シーン名 → Rune function path マッピング
   - Local シーン名 → 親スコープ付きパス マッピング
   - 単語名 → Word function call コード生成

### Next Actions

```
1. Run: /kiro-spec-design transpiler2-layer-implementation
   → Design document で上記4項目を詳細化
   
2. Focus areas:
   - Component diagram (transpiler2 internals)
   - 3-layer scope handling state machine
   - Error handling flow
   - Code generation examples
```

---

## Appendix: Codebase Reference

### Existing Transpiler Analysis

**File: src/transpiler/mod.rs**
- Line 145: `pub fn transpile_pass1<W>()` - Pass 1フローの参考
- Line 189: `fn transpile_global_scene()` - Scene生成の参考パターン
- Line 367: `fn transpile_call_action()` - Call文の変換ロジック

**File: src/transpiler/scene_registry.rs**
- Line 70: `pub fn register_global()` - シーン登録のパターン
- Line 113: `fn sanitize_name()` - 識別子正規化ロジック

**Parser2 AST Reference**
- `src/parser2/ast.rs` Line 62: `pub struct PastaFile` - 新AST型
- `src/parser2/ast.rs` Line 109: `pub struct GlobalSceneScope` - グローバルシーン定義

### Test File Reference

- `tests/pasta_transpiler_two_pass_test.rs` - Pass 1/2 test例
- `tests/pasta_transpiler_comprehensive_test.rs` - 統合テスト例
- `test_combined_code.rn` - 生成Rune code例

