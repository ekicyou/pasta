# Bug Report: parser2 FileScope Multiple Occurrence

## Bug Summary
**Severity**: 🔴 Critical  
**Component**: parser2 (src/parser2/mod.rs)  
**Grammar Specification**: src/parser2/grammar.pest  
**Discovered**: 2025-12-23 during transpiler2-layer-implementation requirements analysis

---

## Specification vs. Implementation Gap

### Grammar Specification (Correct)
```pest
file = _{ SOI ~ ( file_scope | global_scene_scope )* ~ s ~ EOI }
```
**Intent**: `file_scope` と `global_scene_scope` は**任意の順序で複数回出現可能**

### Current Implementation (Buggy)
```rust
// src/parser2/mod.rs:135-137
Rule::file_scope => {
    file.file_scope = parse_file_scope(pair)?;  // ← BUG: 上書き代入
}
```
**Problem**: 複数の`file_scope`が出現した場合、**最後のfile_scopeのみが保持される**（後勝ち）

---

## Reproduction Example

### Input (Valid Grammar)
```pasta
＆季節：冬
＠天気：晴れ｜曇り

＊シーン1
  Alice：冬のシーンです

＆季節：夏        ← 2回目のfile_scope
＆時間：昼

＊シーン2
  Bob：夏のシーンです
```

### Expected Behavior (Grammar Intent)
- **シーン1**: file-level attrs `{季節: "冬"}` を継承 → 最終: `{季節: "冬"}`
- **シーン2**: file-level attrs `{季節: "夏", 時間: "昼"}` を継承 → 最終: `{季節: "夏", 時間: "昼"}`
- File-level words: `["天気"]` (グローバル登録)

### Actual Behavior (Current Bug)
- `file.file_scope` は**2回目のfile_scopeで上書き**される
- 最終的な `file.file_scope.attrs` = `[{季節: "夏"}, {時間: "昼"}]`
- 1回目の `{季節: "冬"}` と `@天気` は**消失**

**結果**: 
- シーン1は存在しない `{季節: "夏", 時間: "昼"}` を継承（誤り）
- `@天気` 単語定義が失われる

---

## Root Cause Analysis

### AST Structure Problem
```rust
// src/parser2/ast.rs:62-70
pub struct PastaFile {
    pub path: PathBuf,
    pub file_scope: FileScope,  // ← 単一フィールド（複数file_scopeを保持不可）
    pub global_scenes: Vec<GlobalSceneScope>,
    pub span: Span,
}
```

**Design Flaw**: `file_scope` が単一フィールドのため、複数のfile_scopeを**順序を保って保持する機構がない**

### Parser Logic Problem
```rust
// src/parser2/mod.rs:135
Rule::file_scope => {
    file.file_scope = parse_file_scope(pair)?;  // 上書き代入
}
```

**Implementation Flaw**: ループ内で `file.file_scope` を上書きし続けるため、最後のfile_scopeのみが残る

---

## Impact Assessment

### Functional Impact
| 影響範囲 | 深刻度 | 詳細 |
|---------|-------|------|
| **Attribute Inheritance** | 🔴 Critical | file-level attributes継承が正しく動作しない |
| **Word Definitions** | 🔴 Critical | 中間のfile-level word定義が消失 |
| **Transpiler2** | 🔴 Blocker | transpiler2-layer-implementationの前提条件が崩壊 |
| **Spec Compliance** | 🔴 Critical | Grammar.pest仕様違反 |

### User Impact
- **中規模以上のPastaスクリプト**: file_scopeを複数回使用するケースで**データ消失**
- **属性フィルタリング**: シーンごとの属性コンテキスト変更が不可能
- **単語定義**: ファイル途中の単語定義が無視される

---

## Proposed Fix (High-Level)

### Option A: Sequential FileScope Processing (Recommended)
**AST Structure Change**:
```rust
pub struct PastaFile {
    pub path: PathBuf,
    pub items: Vec<FileItem>,  // ← file_scope/global_scene_scopeを順序保持
    pub span: Span,
}

pub enum FileItem {
    FileScope(FileScope),
    GlobalSceneScope(GlobalSceneScope),
}
```

**Parser Logic**:
```rust
for pair in pairs {
    match pair.as_rule() {
        Rule::file_scope => {
            file.items.push(FileItem::FileScope(parse_file_scope(pair)?));
        }
        Rule::global_scene_scope => {
            file.items.push(FileItem::GlobalSceneScope(...));
        }
        ...
    }
}
```

**Processing (transpiler2)**: Pass1で順次処理
```rust
let mut current_file_attrs = HashMap::new();
for item in file.items {
    match item {
        FileItem::FileScope(fs) => {
            current_file_attrs.extend(fs.attrs);  // 累積更新
        }
        FileItem::GlobalSceneScope(scene) => {
            let merged = merge(current_file_attrs.clone(), scene.attrs);
            registry.register_global(scene.name, merged);
        }
    }
}
```

**Advantages**:
- ✅ Grammar仕様に完全準拠
- ✅ 直観的（ファイル記述順に従う）
- ✅ 部分的なコンテキスト変更が可能

**Disadvantages**:
- ❌ AST構造の破壊的変更（既存コード影響あり）

---

### Option B: FileScope Accumulation (Lower Impact)
**AST Structure**:
```rust
pub struct PastaFile {
    pub path: PathBuf,
    pub file_scopes: Vec<FileScope>,  // ← 複数保持
    pub global_scenes: Vec<GlobalSceneScope>,
    pub span: Span,
}
```

**Parser Logic**:
```rust
Rule::file_scope => {
    file.file_scopes.push(parse_file_scope(pair)?);
}
```

**Processing**: transpiler側で順序解決

**Advantages**:
- ✅ 変更範囲が小さい
- ✅ 複数file_scopeを保持可能

**Disadvantages**:
- ❌ file_scopeとglobal_scene_scopeの**交互出現順序が保持されない**
- ❌ transpiler側で順序復元ロジックが必要（困難）

---

## Recommendation

**Preferred**: **Option A (Sequential FileScope Processing)**

**Rationale**:
1. Grammar仕様 `( file_scope | global_scene_scope )*` の意図を正確に実装
2. ファイル記述順に従う直観的な動作
3. transpiler2での処理がシンプル（順次処理）
4. 将来的な拡張性（file_scope内に新要素追加時も対応可能）

**Risk**: AST構造変更により既存コード（特にparser2テスト）への影響あり → 修正コスト中程度

---

## Test Cases Required

### Test 1: Multiple FileScope Attributes
```pasta
＆season：winter

＊Scene1
  Alice：冬です

＆season：summer

＊Scene2
  Bob：夏です
```
**Expected**: Scene1 has `{season: "winter"}`, Scene2 has `{season: "summer"}`

### Test 2: FileScope Words Accumulation
```pasta
＠word1：a｜b

＊Scene1
  Alice：＠word1

＠word2：c｜d

＊Scene2
  Bob：＠word1、＠word2
```
**Expected**: Both `word1` and `word2` are globally registered

### Test 3: Attribute Merge with Override
```pasta
＆season：winter
＆weather：sunny

＊Scene1＆season：spring
  Alice：春、晴れ
```
**Expected**: Scene1 final attrs: `{season: "spring", weather: "sunny"}` (scene priority)

---

## Blocking Dependencies

| Dependent Spec | Status | Reason |
|---------------|--------|--------|
| **transpiler2-layer-implementation** | ⏸️ Blocked | Requirement 11 (FileScope Attribute Inheritance)の前提条件 |

---

## Next Steps

1. ✅ **Create Spec**: parser2-filescope-bug-fix
2. ⏳ **Requirements**: Define precise fix requirements
3. ⏳ **Design**: Choose Option A or B, detail AST changes
4. ⏳ **Implementation**: Modify parser2 AST and parser logic
5. ⏳ **Testing**: Add 3+ test cases for multiple file_scope scenarios
6. ⏳ **Validation**: Update transpiler2-layer-implementation spec with dependency resolution
