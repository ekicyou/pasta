# 要件ドキュメント

## 導入

このドキュメントは、pasta_lua トランスパイラーの出力フォーマッティング正規化に関する要件を定義します。

現在、`test_transpile_sample_pasta_line_comparison` 統合テストは失敗しており、生成される Lua コードが期待値と完全に一致していません。具体的な問題は以下の通りです：

- **末尾空行の差異**: 生成ファイルに不要な空行（2行）がある
- **シーン定義の閉じ括弧位置**: `end` の前に余計な空行が挿入されている
- **行数の不一致**: 期待値 114 行、生成値 116 行（+2 行）

これらの問題は、Lua コード生成パイプラインの終了処理、特にシーンスコープの出力正規化が適切に実装されていないことが根本原因です。

要件は、以下の観点から設計されています：

1. **出力正規化**: コード生成終了時に不要な空行を除去
2. **フォーマッティング一貫性**: すべてのシーン定義で統一されたフォーマット
3. **テスト整合性**: `test_transpile_sample_pasta_line_comparison` テストが確実に合格すること
4. **リグレッション防止**: 既存テスト（sample.lua 互換テスト）への影響排除

## 要件

### 1. 末尾空行の除去
**目標**: トランスパイラーが出力する Lua コードの末尾に不要な空行があってはならない

#### 受け入れ条件
1. When transpiler generates Lua code, the LuaTranspiler shall remove all trailing blank lines from the output
2. When output buffer contains multiple consecutive newlines at the end, the LuaTranspiler shall preserve exactly one newline (EOF marker)
3. While code generation is in progress, the LuaTranspiler shall not add unnecessary line breaks between structural elements

### 2. シーン定義の正規化フォーマット
**目標**: グローバルシーン定義の `end` トークンの配置が一貫性を持つ

#### 受け入れ条件
1. When generating a global scene scope closure, the LuaTranspiler shall generate exactly one newline before the `end` keyword
2. If the last function definition in a scene block ends with a closing brace, the LuaTranspiler shall not insert extra blank lines
3. Where multiple scenes are defined in sequence, the LuaTranspiler shall maintain identical indentation and spacing patterns for all scene closures

### 3. 出力バッファの正規化処理
**目標**: コード生成の最終ステップで出力を正規化し、一貫性を確保する

#### 受け入れ条件
1. When transpile method completes code generation, the LuaTranspiler shall apply a normalization pass to the output buffer
2. The LuaTranspiler shall remove consecutive blank lines, preserving single blank lines where semantically appropriate
3. The LuaTranspiler shall ensure the output matches the expected format defined in `sample.expected.lua`

### 4. テストフィクスチャの一致
**目標**: 生成 Lua コードが期待値と完全に一致する

#### 受け入れ条件
1. When the transpiler processes `sample.pasta`, the generated output shall exactly match `sample.expected.lua` (line-by-line comparison with normalized line endings)
2. The generated line count shall equal the expected line count (114 lines)
3. While running test_transpile_sample_pasta_line_comparison, the assertion `assert_eq!(generated_normalized, expected_normalized)` shall pass

### 5. 既存テストとのリグレッション防止
**目標**: 既存の通過テストが引き続き合格すること

#### 受け入れ条件
1. When running the full test suite (`cargo test --all`), all previously passing tests shall continue to pass
2. If any regression is detected (previously passing test now fails), the implementation shall be revised to maintain backward compatibility
3. The test `test_transpile_sample_pasta_basic_output` and all other sample-related tests shall pass with identical behavior
