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
- **独立エラー型**: `ParseError`, `ParseErrorInfo` による詳細なパース診断
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
- `parse_str_partial(source)` — 部分パース（3段階フォールバック: 全体→スコープ分割→行単位）

### AST Types

- `PastaFile` — ファイル全体のAST
- `FileItem` — ファイルレベルアイテム（属性、単語定義、シーン、アクター）
- `GlobalSceneScope` — グローバルシーン定義
- `LocalSceneScope` — ローカルシーン定義
- `ActorScope` — アクター定義
- `Action` — アクション（Talk, WordRef, VarRef, FnCall, SakuraScript, Escape）
- `CueCommandNode` / `ChoiceNode` — キューコマンド行・選択肢行
- `Span` — ソース位置情報（行/列 + バイトオフセット）

### Error Types

- `ParseError` — パースエラー（SyntaxError, PestError, IoError, MultipleErrors）
- `ParseErrorInfo` — 個別パースエラー情報
- `ParseResult<T>` — `Result<T, ParseError>` エイリアス
- `PartialParseResult` / `PartialParseError` — 部分パース結果・行単位エラー

## Dependencies

| クレート    | バージョン | 用途                 |
| ----------- | ---------- | -------------------- |
| pest        | 2.8        | PEG文法解析エンジン  |
| pest_derive | 2.8        | ビルド時パーサー生成 |
| thiserror   | 2          | エラー型生成マクロ   |

**注**: レジストリ関連（fast_radix_trie, rand）やLuaランタイム（mlua）への依存は一切ありません。

## Architecture

```
pasta_dsl
├── src/
│   ├── lib.rs               # クレートエントリーポイント
│   ├── error.rs             # ParseError, ParseErrorInfo, ParseResult
│   ├── partial.rs           # 部分パースAPI（parse_str_partial）
│   └── parser/
│       ├── mod.rs           # パーサーAPI（parse_str, parse_file）
│       ├── parse_scene.rs   # シーン解析
│       ├── parse_action.rs  # アクション解析
│       ├── parse_elements.rs # 要素解析
│       ├── ast/             # AST型定義（ディレクトリ）
│       │   ├── mod.rs       # AST公開エントリ
│       │   ├── span.rs      # Span型定義
│       │   ├── scene.rs     # シーン関連AST
│       │   ├── action.rs    # アクション関連AST
│       │   └── cue.rs       # キューコマンド関連AST
│       └── grammar.pest     # Pest文法定義（権威的仕様）
└── tests/
    ├── actor_code_block_test.rs   # アクターコードブロック解析テスト
    ├── ast_test.rs                # AST型テスト
    ├── choice_line_test.rs        # 選択肢行テスト
    ├── cue_cmd_test.rs            # キューコマンド行テスト
    ├── digit_id_var_test.rs       # 全角数字変数テスト
    ├── dynamic_call_test.rs       # 動的コールテスト
    ├── error_api_test.rs          # ParseError APIテスト
    ├── expr_parse_test.rs         # 式ASTパーステスト
    ├── parser_test.rs             # パーサー統合テスト
    ├── partial_parse_test.rs      # 部分パースAPIテスト
    ├── property_scope_test.rs     # プロパティスコープテスト
    ├── sakura_symbol_tag_test.rs  # さくらスクリプト記号タグテスト
    ├── span_byte_offset_test.rs   # バイトオフセットテスト
    └── var_set_none_test.rs       # 式文（$=）テスト
```

## License

MIT OR Apache-2.0
