# pasta_core

Pasta DSL のレジストリとユーティリティを提供する言語非依存層クレートです。

## 概要

`pasta_core` は Pasta DSL のシーン/単語テーブル管理を担当します。
DSLパーサーは独立クレート `pasta_dsl` に分離されており、
本クレートはバックエンド（Lua等）に依存しない純粋なデータ構造を提供します。

## アーキテクチャ

```
pasta_core
├── Registry       # シーン/単語テーブル管理
│   ├── SceneRegistry   # シーン登録（Pass 1）
│   ├── WordDefRegistry # 単語定義登録
│   ├── SceneTable      # シーン検索（Radix Trie）
│   └── WordTable       # 単語検索
└── Error          # テーブルエラー型定義
```

> **Note**: DSLパーサーは独立クレート [pasta_dsl](../pasta_dsl/README.md) に分離されています。

## ディレクトリ構成

```
pasta_core/
├── Cargo.toml
├── src/
│   ├── lib.rs           # クレートエントリーポイント
│   ├── error.rs         # SceneTableError, WordTableError
│   └── registry/        # 型管理レイヤー
│       ├── mod.rs              # Registry API
│       ├── scene_registry.rs   # SceneRegistry
│       ├── scene_types.rs      # SceneId, SceneScope, SceneInfo（抽出型）
│       ├── word_registry.rs    # WordDefRegistry
│       ├── scene_table.rs      # SceneTable（Radix Trie）
│       ├── scene_table_candidate_tests.rs      # SceneTable テスト（候補収集/解決/#[path]属性）
│       ├── scene_table_resolve_filter_tests.rs # SceneTable テスト（解決境界/属性フィルタ/アクセサ）
│       ├── word_table.rs       # WordTable
│       └── random.rs           # RandomSelector インターフェース
└── tests/
    └── word_table_test.rs   # WordTable 統合テスト
```

> **Note**: パーサーモジュール（parser/, grammar.pest）は独立クレート [pasta_dsl](../pasta_dsl/README.md) に移動されました。

## 公開API

> **パーサーAPI**（`parse_str`, `parse_file`, AST型）は [pasta_dsl](../pasta_dsl/README.md) クレートに移動されました。

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

> **Note**: DSLのパースは [pasta_dsl](../pasta_dsl/README.md) クレートを使用してください。

### シーンテーブルの構築と検索

```rust
use std::collections::HashMap;
use pasta_core::registry::{SceneRegistry, SceneTable, DefaultRandomSelector};

// Pass 1: トランスパイル時にシーンを登録
let mut registry = SceneRegistry::new();
registry.register_global("挨拶", HashMap::new());

// Runtime: SceneTable へ変換して前方一致検索
let mut table = SceneTable::from_scene_registry(
    registry,
    Box::new(DefaultRandomSelector::new()),
).unwrap();

let scene_id = table.resolve_scene_id("挨拶", &HashMap::new()).unwrap();
let scene = table.get_scene(scene_id).unwrap();
println!("Selected: {}", scene.fn_name);
```

### 単語テーブルの構築と検索

```rust
use pasta_core::registry::{WordDefRegistry, WordTable, DefaultRandomSelector};

// Pass 1: 単語定義を登録
let mut registry = WordDefRegistry::new();
registry.register_global("挨拶", vec!["こんにちは".to_string(), "おはよう".to_string()]);

// Runtime: WordTable へ変換してランダム選択（同一キーは循環消費）
let mut table = WordTable::from_word_def_registry(
    registry,
    Box::new(DefaultRandomSelector::new()),
);
let word = table.search_word("", "挨拶", &[]).unwrap();
println!("Selected: {}", word);
```

## 依存関係

| クレート        | バージョン | 用途                       |
| --------------- | ---------- | -------------------------- |
| thiserror       | 2          | エラー型定義               |
| fast_radix_trie | 1.2.0      | 前方一致検索（SceneTable） |
| rand            | 0.10       | ランダム選択               |

## 関連クレート

- [pasta_dsl](../pasta_dsl/README.md) - DSLパーサー
- [pasta_lua](../pasta_lua/README.md) - Luaバックエンド
- [pasta_shiori](../pasta_shiori/README.md) - SHIORI DLL統合
- [プロジェクト概要](../../README.md) - pasta プロジェクト全体

## ライセンス

プロジェクトルートの [LICENSE](../../LICENSE) ファイルを参照してください。
