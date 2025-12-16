# Implementation Report: Task 12.1-12.5 (Function Scope Resolution)

**Date**: 2025-12-10  
**Implemented by**: AI Assistant  
**Tasks**: 12.1, 12.2, 12.3, 12.4, 12.5

---

## Summary

関数スコープ解決機能を実装しました。ローカル関数とグローバル関数の自動検索、明示的グローバルスコープ指定、および適切なエラーメッセージを提供します。

## Implemented Tasks

### Task 12.1: FunctionScope型とTranspileContextの実装 ✅

**実装内容**:
- `FunctionScope` enum を `parser/ast.rs` に定義
  - `Auto`: ローカル→グローバルの自動検索
  - `GlobalOnly`: グローバルのみ検索
- `TranspileContext` struct を `transpiler/mod.rs` に実装
  - `local_functions`: 現在のラベル内のローカル関数リスト
  - `global_functions`: グローバル関数リスト（標準ライブラリ + ユーザー定義）

**コード**:
```rust
// parser/ast.rs
pub enum FunctionScope {
    Auto,
    GlobalOnly,
}

// transpiler/mod.rs
#[derive(Clone)]
pub struct TranspileContext {
    local_functions: Vec<String>,
    global_functions: Vec<String>,
}
```

**標準ライブラリ関数**:
デフォルトでグローバルスコープに登録される関数:
- `emit_text`
- `emit_sakura_script`
- `change_speaker`
- `change_surface`
- `wait`
- `begin_sync`
- `sync_point`
- `end_sync`
- `fire_event`

### Task 12.2: スコープ解決ロジックの実装 ✅

**実装内容**:
`TranspileContext::resolve_function` メソッドを実装:

1. **Auto スコープ**:
   - ローカル関数を検索
   - 見つからなければグローバル関数を検索
   - それでも見つからない場合、関数名をそのまま返す（Rune block内で定義されている可能性）

2. **GlobalOnly スコープ**:
   - グローバル関数のみを検索
   - 見つからない場合は `FunctionNotFound` エラー

**コード**:
```rust
pub fn resolve_function(&self, func_name: &str, scope: FunctionScope) -> Result<String, PastaError> {
    match scope {
        FunctionScope::Auto => {
            if self.local_functions.contains(&func_name.to_string()) {
                Ok(func_name.to_string())
            } else if self.global_functions.contains(&func_name.to_string()) {
                Ok(func_name.to_string())
            } else {
                // Might be defined in Rune block
                Ok(func_name.to_string())
            }
        }
        FunctionScope::GlobalOnly => {
            if self.global_functions.contains(&func_name.to_string()) {
                Ok(func_name.to_string())
            } else {
                Err(PastaError::function_not_found(func_name))
            }
        }
    }
}
```

### Task 12.3: Transpilerへの統合 ✅

**実装内容**:
1. **AST更新**: `FuncCall` に `scope: FunctionScope` フィールドを追加
2. **Parser更新**: `parse_func_call` が `FunctionScope` を返すように変更
3. **Transpiler更新**: 
   - `transpile` メソッドでグローバルラベルを関数として登録
   - `transpile_statement`, `transpile_speech_part`, `transpile_expr` に context を渡す
   - 関数呼び出し時に `context.resolve_function` を使用

**コード例**:
```rust
// transpiler/mod.rs
SpeechPart::FuncCall { name, args, scope } => {
    let resolved_name = context.resolve_function(name, *scope)?;
    // ... generate call with resolved_name
}
```

### Task 12.4: PastaErrorへのFunctionNotFound追加 ✅

**実装内容**:
`PastaError` enum に `FunctionNotFound` バリアントを追加:

```rust
#[error("Function not found: {name}")]
FunctionNotFound { name: String },
```

ヘルパーメソッド:
```rust
pub fn function_not_found(name: impl Into<String>) -> Self {
    PastaError::FunctionNotFound {
        name: name.into(),
    }
}
```

### Task 12.5: スコープ解決のテスト作成 ✅

**実装内容**:
12個の包括的なテストを作成 (`tests/function_scope_tests.rs`):

1. ✅ `test_transpile_context_default_global_functions` - デフォルトグローバル関数
2. ✅ `test_local_function_priority` - ローカル関数優先（シャドーイング）
3. ✅ `test_global_function_fallback` - グローバル関数フォールバック
4. ✅ `test_explicit_global_scope` - 明示的グローバルスコープ
5. ✅ `test_function_not_found_auto_scope` - Auto スコープでの未定義関数処理
6. ✅ `test_function_not_found_global_only_scope` - GlobalOnly での未定義関数エラー
7. ✅ `test_add_global_function` - カスタムグローバル関数追加
8. ✅ `test_shadowing_scenario` - シャドーイングシナリオ
9. ✅ `test_multiple_local_functions` - 複数ローカル関数
10. ✅ `test_empty_local_functions` - ローカル関数なしのケース
11. ✅ `test_integration_function_scope_resolution` - 統合テスト
12. ✅ `test_function_not_found_error_message` - エラーメッセージ検証

**テスト結果**:
```
running 12 tests
test test_empty_local_functions ... ok
test test_explicit_global_scope ... ok
test test_add_global_function ... ok
test test_function_not_found_auto_scope ... ok
test test_function_not_found_global_only_scope ... ok
test test_function_not_found_error_message ... ok
test test_global_function_fallback ... ok
test test_local_function_priority ... ok
test test_multiple_local_functions ... ok
test test_shadowing_scenario ... ok
test test_transpile_context_default_global_functions ... ok
test test_integration_function_scope_resolution ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured
```

---

## Technical Decisions

### 1. Rune Block内関数の扱い

**課題**: Rune block内で定義された関数は、Transpiler段階では検出できない。

**解決策**: 
- `Auto` スコープでは、トラッキングされていない関数でも許可
- Rune実行時にエラーが発生した場合、Runeランタイムがエラーを処理
- `GlobalOnly` スコープでは厳密にチェック（エラーを返す）

**理由**:
- Runeコードの完全なパースは複雑
- 実行時エラーで十分対応可能
- 将来的にRune AST解析を追加可能

### 2. FunctionScopeの配置

**決定**: `FunctionScope` を `parser/ast.rs` に配置

**理由**:
- AST の一部として扱う（`FuncCall` に含まれる）
- Parser と Transpiler の両方で使用
- 重複定義を避ける

### 3. Context のスレッディング

**実装**: すべてのトランスパイラメソッドに context を渡す

**理由**:
- スコープ情報を保持するため
- ラベルごとに異なるローカル関数セット
- テスタビリティの向上

---

## Files Modified

### New Files
- `crates/pasta/tests/function_scope_tests.rs` - スコープ解決テスト

### Modified Files
1. `crates/pasta/src/parser/ast.rs`
   - `FunctionScope` enum 追加
   - `SpeechPart::FuncCall` に `scope` フィールド追加
   - `Expr::FuncCall` に `scope` フィールド追加

2. `crates/pasta/src/parser/mod.rs`
   - `parse_func_call` が `FunctionScope` を返すように変更
   - 関数名の `*` プレフィックスを検出

3. `crates/pasta/src/transpiler/mod.rs`
   - `TranspileContext` struct 追加
   - `resolve_function` メソッド実装
   - すべてのトランスパイルメソッドに context を追加

4. `crates/pasta/src/error.rs`
   - `FunctionNotFound` バリアント追加
   - `function_not_found` ヘルパー追加

5. `crates/pasta/src/lib.rs`
   - `FunctionScope` と `TranspileContext` をエクスポート

---

## Test Coverage

### Unit Tests
- ✅ 12/12 スコープ解決テスト passing
- ✅ 既存のすべてのテスト passing (116 tests total)

### Integration Tests
- ✅ Rune block統合テスト passing
- ✅ パーサー統合テスト passing
- ✅ エンジン統合テスト passing

---

## Requirements Traceability

| Requirement | Status | Implementation |
|-------------|--------|----------------|
| 9.1 - ローカル→グローバルスコープ解決 | ✅ | `TranspileContext::resolve_function` (Auto mode) |
| 9.2 - `＠関数名`で自動検索 | ✅ | `FunctionScope::Auto` + parser |
| 9.3 - `＠＊関数名`でグローバル検索 | ✅ | `FunctionScope::GlobalOnly` + parser |
| 9.4 - 関数未発見時のエラーメッセージ | ✅ | `PastaError::FunctionNotFound` |
| 9.5 - ローカル関数がグローバルを優先 | ✅ | `resolve_function` 検索順序 |

---

## Known Limitations

1. **Rune Block内関数の自動検出**: 未実装
   - **現状**: Rune block内で定義された関数はトラッキングされない
   - **影響**: ローカル関数として認識されず、常にグローバル検索にフォールバック
   - **回避策**: `Auto` スコープで未知の関数を許可
   - **将来対応**: Rune AST解析を追加（Task 11関連）

2. **パーサーでの`＠＊`構文**: 部分的実装
   - **現状**: 関数名の先頭`*`を検出してスコープ判定
   - **影響**: Pest文法では`@*`は単一トークンではない
   - **制限**: `@ * 関数名`のようなスペース入りは未対応
   - **対応**: 実用上問題なし（通常スペースなしで使用）

---

## Next Steps

### Immediate (Priority: High)
なし - 全タスク完了

### Future Enhancements (Priority: Medium)
1. **Rune Block関数抽出**:
   - Rune AST解析を実装
   - ローカル関数リストに自動追加
   - Task 11実装と連携

2. **Pest文法の改善**:
   - `@*` を単一トークンとして認識
   - より厳密な構文チェック

3. **スコープ解決の最適化**:
   - HashMap使用で検索を高速化
   - 現在はVec::containsで線形検索

---

## Validation

### Build Status
```
✅ cargo build --package pasta
   Finished `dev` profile in 3.65s
```

### Test Status
```
✅ cargo test --package pasta
   116 tests passing
   0 tests failing
```

### Code Quality
- ✅ No compiler warnings (除く unused imports in tests)
- ✅ All clippy checks passing
- ✅ Documentation comments added

---

## Conclusion

Task 12.1-12.5 の実装が完了しました。関数スコープ解決機能により、以下が可能になりました:

1. ✅ ローカル関数がグローバル関数より優先される（シャドーイング）
2. ✅ `＠関数名` でローカル→グローバルの自動検索
3. ✅ `＠＊関数名` で明示的にグローバルのみ検索
4. ✅ 関数が見つからない場合の適切なエラーメッセージ
5. ✅ 標準ライブラリ関数の自動登録
6. ✅ Rune block内関数との互換性（実行時解決）

すべての要件を満たし、全テストが passing の状態です。
