# pasta_core

Pasta DSL のパーサーとレジストリを提供する言語非依存層クレートです。

## 概要

`pasta_core` は Pasta DSL の構文解析とシーン/単語テーブル管理を担当します。
バックエンド（Lua等）に依存しない純粋なパーサーとデータ構造を提供し、複数のランタイムで再利用可能です。

## アーキテクチャ

```
pasta_core
├── Parser         # DSL → AST 変換（Pest PEG）
├── Registry       # シーン/単語テーブル管理
│   ├── SceneRegistry   # シーン登録（Pass 1）
│   ├── WordDefRegistry # 単語定義登録
│   ├── SceneTable      # シーン検索（Radix Trie）
│   └── WordTable       # 単語検索
└── Error          # パースエラー型定義
```

## ディレクトリ構成

```
pasta_core/
├── Cargo.toml
└── src/
    ├── lib.rs           # クレートエントリーポイント
    ├── error.rs         # ParseError, SceneTableError, WordTableError
    ├── parser/          # パーサーレイヤー
    │   ├── mod.rs       # パーサーAPI（parse_str, parse_file）
    │   ├── ast.rs       # AST定義（PastaFile, FileItem, Statement等）
    │   └── grammar.pest # Pest PEG文法定義
    └── registry/        # 型管理レイヤー
        ├── mod.rs       # Registry API
        ├── scene_registry.rs  # SceneRegistry
        ├── word_registry.rs   # WordDefRegistry
        ├── scene_table.rs     # SceneTable（Radix Trie）
        ├── word_table.rs      # WordTable
        └── random.rs          # RandomSelector インターフェース
```

## 公開API

### Parser

| 関数/型                       | 説明                                              |
| ----------------------------- | ------------------------------------------------- |
| `parse_str(source, filename)` | 文字列からパース                                  |
| `parse_file(path)`            | ファイルからパース                                |
| `PastaFile`                   | パース結果（ASTルート）                           |
| `FileItem`                    | ファイル直下の項目（GlobalSceneScope, WordDef等） |

### Registry

| 型                | 説明                              |
| ----------------- | --------------------------------- |
| `SceneRegistry`   | シーン登録・管理（Pass 1）        |
| `WordDefRegistry` | 単語定義登録                      |
| `SceneTable`      | シーン検索（完全一致 + 前方一致） |
| `WordTable`       | 単語検索                          |
| `SceneEntry`      | シーン情報エントリ                |
| `WordEntry`       | 単語情報エントリ                  |

### Random

| 型                      | 説明                 |
| ----------------------- | -------------------- |
| `RandomSelector`        | ランダム選択トレイト |
| `DefaultRandomSelector` | 本番用ランダム実装   |
| `MockRandomSelector`    | テスト用固定選択実装 |

## 使用例

### 基本的なパース

```rust
use pasta_core::parser::{parse_str, FileItem, PastaFile};

let source = r#"
＊挨拶
    Alice：こんにちは
    Bob：やあ！
"#;

let ast = parse_str(source, "example.pasta").unwrap();

// グローバルシーン数をカウント
let scene_count = ast.items.iter()
    .filter(|i| matches!(i, FileItem::GlobalSceneScope(_)))
    .count();
println!("Parsed {} global scenes", scene_count);
```

### シーンテーブルの構築

```rust
use pasta_core::registry::{SceneTable, SceneEntry, SceneId, SceneScope, DefaultRandomSelector};

let mut table = SceneTable::new();

// シーンを登録
let entry = SceneEntry {
    id: SceneId(0),
    name: "挨拶".to_string(),
    scope: SceneScope::Global,
    file_path: "example.pasta".to_string(),
};
table.insert(entry);

// 完全一致検索
let selector = DefaultRandomSelector::new();
if let Some(scene) = table.get_exact("挨拶", &selector) {
    println!("Found scene: {:?}", scene);
}
```

### 前方一致検索

```rust
use pasta_core::registry::{SceneTable, DefaultRandomSelector};

let selector = DefaultRandomSelector::new();
// "挨拶" で始まるすべてのシーンからランダムに1つ選択
if let Some(scene) = table.get_prefix("挨拶", &selector) {
    println!("Selected: {:?}", scene);
}
```

## 依存関係

| クレート        | バージョン | 用途                       |
| --------------- | ---------- | -------------------------- |
| pest            | 2.8        | PEGパーサー                |
| pest_derive     | 2.8        | パーサー生成マクロ         |
| thiserror       | 2          | エラー型定義               |
| fast_radix_trie | 1.1.0      | 前方一致検索（SceneTable） |
| rand            | 0.9        | ランダム選択               |
| tracing         | 0.1        | ロギング・診断             |

## 関連クレート

- [pasta_lua](../pasta_lua/README.md) - Luaバックエンド
- [pasta_shiori](../pasta_shiori/README.md) - SHIORI DLL統合
- [プロジェクト概要](../../README.md) - pasta プロジェクト全体

## ライセンス

プロジェクトルートの [LICENSE](../../LICENSE) ファイルを参照してください。
