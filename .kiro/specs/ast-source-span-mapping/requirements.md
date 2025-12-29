# 要件ドキュメント

## イントロダクション

AST（抽象構文木）ノードから元のソースコードへの正確な参照を確立することは、エラー報告、デバッグ情報表示、IDEのコード補完やハイライト等の高度な機能実装に不可欠です。本仕様では、Pastaパーサーが生成するASTの各ノードに、元ファイル内での正確な位置情報（バイト単位のインデックス）を含める**Span拡張機能**を定義します。

## プロジェクト説明（入力）
parserのASTから、元ソースコードへの参照を得ることができるようにSpanを拡張して欲しい。元ファイルの先頭からのインデックスが必要。

## 要件

### 1. Span構造の拡張

**目的**: パーサーが生成するSpan型に、元ファイル先頭からのバイト単位インデックス情報を追加し、ソースコード参照情報を完全に記録する

#### 受け入れ基準
1. The Parser shall provide absolute byte offsets for start and end positions from the beginning of the source file in the Span structure
2. When parsing a Pasta script, the Parser shall assign a valid Span with byte offsets to every AST node (Statement, Expression, Label definition)
3. If a Span is missing or invalid, the Parser shall return an error indicating incomplete position tracking
4. The Parser shall handle multi-byte UTF-8 characters correctly when calculating absolute byte offsets
5. The Span structure shall include fields for absolute start byte and absolute end byte, distinct from any relative position information

### 2. パーサー層での位置情報追跡

**目的**: Pestパーサーが文法マッチング時に正確な位置情報を取得し、ASTの全ノードにSpan情報を割り当てることを保証する

#### 受け入れ基準
1. When the Parser parses a source file with UTF-8 content, the Parser shall maintain byte-accurate position information from the Pest parser input stream
2. While traversing Pest parse tree nodes, the Parser shall extract byte offset information and attach it to corresponding AST nodes
3. Where nested constructs exist (e.g., scenes containing expressions), the Parser shall propagate accurate span information to child nodes
4. The Parser shall map Pest's span information directly to absolute byte offsets from file start
5. If the Pest parser returns incomplete span information, the Parser shall use line/column information as a fallback to compute byte offsets

### 3. AST型定義への統合

**目的**: デバッグ・トランスパイル対応に必要な粒度でSpan情報をAST全体に統合し、
ソースコード参照と行単位のコメント挿入を可能にする

#### 受け入れ基準
1. The Parser shall include a Span field in Statement enum variants representing scenes, labels, and flow control
2. The Parser shall include a Span field for Action elements (each action within ActionLine)
   to enable line-by-line comment insertion and precise debug stack trace mapping
3. The Parser shall include a Span field in Label definition structures
4. When creating an AST node, the Parser shall always populate the Span field with valid byte offset information
5. The AST definition shall ensure backward compatibility with existing test fixtures and error handling code

### 4. ソースコード参照機能の提供

**目的**: Span情報を利用して、元ソースコードの該当部分を効率的に取得できる公開API を提供する

#### 受け入れ基準
1. The Parser shall provide a function that takes a Span and source text and returns the corresponding source code substring
2. When a substring extraction is requested, the Parser shall use byte offsets to locate and extract the exact character range
3. The Parser shall handle edge cases (start == end, out-of-bounds offsets) gracefully
4. The Parser module shall export utility functions for Span-based source reference queries in the public API
5. Where Span information is invalid or missing, the function shall return an error rather than panicking

### 5. テスト・検証カバレッジ

**目的**: Span拡張機能が正確に動作し、様々なシナリオで位置情報を正しく保持することを検証する

#### 受け入れ基準
1. When parsing Pasta scripts with ASCII characters, the Parser shall produce accurate byte offset Spans
2. When parsing scripts containing multi-byte UTF-8 characters (日本語、絵文字等), the Parser shall compute correct absolute byte offsets
3. When parsing complex nested structures (scenes with conditional expressions), the Parser shall maintain accurate Spans for all nested nodes
4. The test suite shall include fixtures covering ASCII-only, mixed UTF-8, and edge cases (empty lines, long lines, etc.)
5. Where regressions are introduced, existing tests shall fail, ensuring position tracking correctness over time

### 6. 後方互換性とエラーハンドリング

**目的**: Span拡張により既存のパーサー動作が破壊されず、エラー報告機能が向上することを保証する

#### 受け入れ基準
1. The Parser shall not break existing parsing logic or change AST structure compatibility requirements
2. When a parsed file contains syntax errors, the Parser error shall include accurate Span information pointing to the error location
3. If Span attachment fails for any AST node, the Parser shall propagate an error rather than silently creating invalid Spans
4. The error type shall distinguish between parsing errors and span attachment errors
5. All existing error handling tests shall continue to pass after Span integration

### 7. トランスパイル時のコメント生成対応

**目的**: トランスパイラが、生成されたRune/Luaコードの各アクション実行部に
元Pastaソース位置を示すコメントを挿入可能にする

#### 受け入れ基準
1. Each Action shall carry distinct Span information (start_byte, end_byte) 
   to enable the transpiler to map transpiled code lines back to original source
2. ActionLine-level Span information shall enable grouping of related actions 
   and insertion of line-scope source location metadata
3. The transpiler shall have access to Span data allowing generation of comments such as:
   `// @source=scene.pasta:5:12-18 (bytes 87-93)` before/after action execution
4. Nested construct Spans shall be accessible for fine-grained source mapping when needed
