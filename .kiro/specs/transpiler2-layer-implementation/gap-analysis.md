# Implementation Gap Analysis: transpiler2-layer-implementation

## Analysis Date
2025-12-23

## ✅ Dependency Resolution: parser2-filescope-bug-fix

**Status**: ✅ **RESOLVED** - parser2のFileScope複数出現バグは修正済みです。

**Fixed Issue**: parser2は `file = ( file_scope | global_scene_scope )*` 文法仕様に準拠し、複数の`file_scope`を順序を保って処理できるようになりました。

**Implementation**: `PastaFile.items: Vec<FileItem>` 構造により、file_scopeとglobal_scene_scopeの出現順序が保持されます。

**Enabled Requirements**:
- Requirement 11: FileScope Attribute Inheritance（Pass1での順次処理が可能）
- Requirement 15: FileScope Words Registration（全file_scope wordsが保持される）

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

5. **Fixture Strategy** ✅ **RESOLVED (議題2)**
   - **Decision**: parser2 test fixtures (`tests/fixtures/parser2/*.pasta`, `comprehensive_control_flow2.pasta`) を流用
   - **Rationale**: parser2で既にテスト済み、重複を避ける
   - **Approach**: transpiler固有機能（変数スコープ、call処理）で5-10個の新規fixtureのみ追加

### New Features (parser1→parser2 AST Changes) - **Critical Gaps**

6. **FileScope Attribute Inheritance (Req 11)** 🚨 **NEW PROCESSING REQUIRED**
   - **Gap**: parser1には`FileScope`自体が存在しない → 旧transpilerはfile-level attributesを処理不可
   - **parser2 Structure**: `PastaFile { file_scope: FileScope { attrs, words }, global_scenes }`
   - **Required Implementation**:
     - `file_scope.attrs`を解析してHashMap<String, String>に変換
     - グローバルシーン登録時に、file-level attrsとシーンattrsをmerge
     - Merge rule: シーンレベル属性が優先（同一キーの場合上書き）
   - **Example**:
     ```pasta
     ＆天気：晴れ     # file-level
     ＆季節：冬       # file-level
     ＊会話＆時間：夜＆季節：夏  # scene-level
     ```
     → シーン「会話」最終属性: `{天気: "晴れ", 時間: "夜", 季節: "夏"}`
   - →Design で attribute merge strategyを詳細設計

7. **Scene Attributes Processing (Req 12)** 🚨 **NEW PROCESSING REQUIRED**
   - **Gap**: 旧transpiler `transpile_attributes_to_map()` は常に空HashMap `#{}` を返す（P0スコープ外として未実装）
   - **Code Reference**: `src/transpiler/mod.rs:558` - "P0: filters are not used, always return empty map"
   - **parser2 Structure**: `GlobalSceneScope.attrs: Vec<Attr>`, `LocalSceneScope.attrs: Vec<Attr>`
   - **Required Implementation**:
     - `GlobalSceneScope.attrs` / `LocalSceneScope.attrs`を解析
     - 属性値（文字列リテラル、エスケープシーケンス）を正しく処理
     - SceneRegistry.register_global/register_localに渡す
   - →Design で attribute conversion logicを実装

8. **CodeBlock Embedding (Req 13)** 🚨 **NEW PROCESSING REQUIRED**
   - **Gap**: parser1には`code_blocks`機能が存在しない → 旧transpilerはRune codeブロックを処理不可
   - **parser2 Structure**: 
     ```rust
     GlobalSceneScope { code_blocks: Vec<CodeBlock>, ... }
     LocalSceneScope { code_blocks: Vec<CodeBlock>, ... }
     ```
   - **Required Implementation**:
     - `GlobalSceneScope.code_blocks`をグローバルモジュールレベルに出力
     - `LocalSceneScope.code_blocks`をローカルシーン関数内に出力
     - 出力位置の制御（statements/itemsとの順序）
     - code_blocks内容をそのまま出力（構文検証はRune VMに委譲）
   - →Design で code block placement strategyを決定

9. **ContinueAction Explicit Processing (Req 14)** 🚨 **SPECIFICATION CHANGE**
   - **Gap**: pasta.pest（旧）では継続行に明示的prefixなし、pasta2.pest（新）では`：`prefixが必須
   - **parser2 Structure**: `LocalSceneItem::ContinueAction(ContinueAction { actions, span })`
   - **Required Implementation**:
     - `ContinueAction`型を認識し、`ActionLine`と別処理
     - 直前の`ActionLine`に連結（同一yield文として出力）
     - 最初のitemがContinueActionの場合、TranspileError::InvalidContinuationを返す
   - →Design で continuation line merge logicを実装

10. **FileScope Words Registration (Req 15)** 🚨 **FIELD LOCATION CHANGE**
    - **Gap**: parser1では`PastaFile.global_words`として単一フィールド、parser2では`PastaFile.file_scope.words`に移動
    - **Code Reference**: 旧transpiler `src/transpiler/mod.rs:156` - `for word_def in &file.global_words { ... }`
    - **Required Implementation**:
      - `file_scope.words`（Vec<KeyWords>）をPhase 1で最初に処理
      - WordDefRegistry.register_globalに登録
      - file_scope.wordsとglobal_scene.wordsの重複チェック（Warningのみ、エラーではない）
    - →Design で word registration orderを明確化

---

## 6. Complexity and Risk Assessment

### Effort Estimation

| Phase | Task | Days | Notes |
|-------|------|------|-------|
| **Design** | Architecture + error types + scope logic + 新機能5項目 | 2-3 | Research items解決 + 新ギャップ設計 |
| **Implementation** | mod.rs + context + registries + codegen + 新機能実装 | 4-6 | ~1000-1200行Rust code (FileScope/CodeBlock/Attributes処理追加) |
| **Testing** | 15カテゴリテスト + fixtures | 2-3 | parser2 fixtures流用 + 新機能テスト追加 |
| **Total** | | **8-12日** | M→L (medium-to-large) |

**変更理由**: 5つの新機能（Req 11-15）追加により、設計・実装・テストすべてのフェーズで工数増加。特にAttribute継承ロジック（Req 11-12）とCodeBlock埋め込み（Req 13）は新規設計が必要。

### Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **AST Mismatch** | Medium | High | Design phase で scope conversion を完全設計 |
| **Rune Codegen Bug** | Medium | High | 生成コードのunit test → Runtime 実行テスト |
| **Symbol Resolution** | Low | Medium | Phase 1 registration ロジックを厳密に仕様化 |
| **Compatibility** | Low | Medium | E2E integration test で既存Runtime 連携確認 |
| **Attribute Merge Logic** 🆕 | Medium | Medium | File-level/scene-level属性mergeルールをテストで網羅検証 |
| **CodeBlock Placement** 🆕 | Low | Medium | Code block出力位置を仕様化、出力Runeコードの構文検証テスト |
| **ContinueAction Continuity** 🆕 | Low | Low | 継続行連結ロジックをunit testで厳密検証 |

**Overall Risk: Medium-High** (新規実装 + 5つの新機能追加でリスク増加、ただし既存パターン踏襲で軽減可能)

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

3. 🚨 CLARIFICATION NEEDED - 議題として検討:
   - **式の型システム**: parser2では Integer/Float を分離したが、Rune出力時の型推論戦略は？
     - parser1 transpiler: Literal::Number(f64) を直接 to_string() で出力
     - parser2 AST: Integer(i64) と Float(f64) を明示的に区別
     - Question: Rune VMでの型推論に委ねる？または明示的に型サフィックス（`42i64`）を付与？
   
   - **変数のスコープ解決**: parser2では VarScope::Local/Global だが、transpiler2での参照方法は？
     - parser1 transpiler: `ctx.local.変数名` / `ctx.global.変数名`
     - parser2 AST: VarScope enum は同じ構造
     - Question: Req 5の「変数参照をRune値として埋め込む」は文字列補間？代入文の右辺？両方？
     - Example clarification needed: `let msg = "Count: $count";` → `format!("Count: {}", ctx.local.count)` なのか？
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

