# pasta_dsl

**Pasta DSL - Independent DSL parser and AST definitions**

Pasta DSLの独立パーサークレート。Luaバックエンドやレジストリに依存せず、DSL文法解析とAST型定義のみを提供します。

## Purpose

- Pasta DSLスクリプトを解析してAST（抽象構文木）に変換
- 外部開発者がpasta_dslのみを依存に追加してDSL解析を利用可能
- 新しいバックエンド実装（Lua以外）を容易にする

## Features

- **PEG文法パーサー**: Pest 2.8による堅牢な文法駆動パーサー
- **完全なAST型定義**: `PastaFile`, `FileItem`, `GlobalSceneScope`, `Action` 等
- **独立エラー型**: `ParseError`（`SyntaxError`, `IoError`）による位置つきのパース診断
- **ゼロバックエンド依存**: Lua、レジストリ、ランタイムへの依存なし
- **ステートレス**: 複数スレッドから安全に呼び出し可能

## Usage

```rust
use pasta_dsl::parser::{parse_str, FileItem};

let source = "＊挨拶\n  Alice：こんにちは\n";
let ast = parse_str(source, "test.pasta").unwrap();

for item in &ast.items {
    match item {
        FileItem::FileAttr(attr) => println!("Attr: {}", attr.key),
        FileItem::GlobalWord(word) => println!("Word: {}", word.name()),
        FileItem::GlobalSceneScope(scene) => println!("Scene: {}", scene.name),
        FileItem::ActorScope(actor) => println!("Actor: {}", actor.name),
    }
}
```

## Public API

### Parse Functions

- `parse_str(source, filename)` — 文字列からDSLをパース
- `parse_file(path)` — ファイルからDSLをパース
- `parse_str_partial(source)` — 色のための部分パース（字下げする前の行頭の印で塊に分け、塊ごとに解く）

### AST Types

- `PastaFile` — ファイル全体のAST
- `FileItem` — ファイルレベルアイテム（属性、単語定義、シーン、アクター）
- `GlobalSceneScope` — グローバルシーン定義
- `LocalSceneScope` — ローカルシーン定義
- `ActorScope` — アクター定義
- `Action` — アクション（Talk, WordRef, VarRef, FnCall, SakuraScript, Escape, DynamicWordRef, DynamicFnCall）
- `CueCommandNode` / `ChoiceNode` — キューコマンド行・選択肢行
- `Span` — ソース位置情報（行/列 + バイトオフセット）

### Error Types

- `ParseError` — パースエラー（`SyntaxError`: ファイル名・行・列つきの構文の誤り、`IoError`: ファイルの読み込みの誤り）
- `PartialParseResult` — 部分パース結果（解けた塊の AST `items` と解けない塊の行の範囲 `failed_lines`）

## Dependencies

| クレート    | バージョン | 用途                 |
| ----------- | ---------- | -------------------- |
| pest        | 2.8        | PEG文法解析エンジン  |
| pest_derive | 2.8        | ビルド時パーサー生成 |
| thiserror   | 2          | エラー型生成マクロ   |

**注**: レジストリ関連（rand）やLuaランタイム（mlua）への依存は一切ありません。

## Architecture

`parse_str` / `parse_file` parse the source with the Pest grammar (`grammar.pest`, the authoritative syntax) and convert the result, via `parser/parse_scene.rs`, `parser/parse_action.rs` and `parser/parse_elements.rs`, into the AST types under `parser/ast/`. `parse_str_partial` (`partial.rs`) splits the source into chunks at unindented line-head markers and parses each chunk; it is used only for coloring by the editor analyzer (`pasta_lsp`), not by the runtime transpiler. The crate has no runtime state; `pasta_lua` consumes the AST to generate Lua code.

See [Internals: Parser and AST](https://ekicyou.github.io/pasta/internals/transpiler.html#パーサと-ast) for the source layout and how the parser feeds the transpiler (the manual is written in Japanese).

## License

MIT
