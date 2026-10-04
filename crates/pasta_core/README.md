# pasta_core

Pasta DSL のレジストリとユーティリティを提供する言語非依存層クレートです。

## 概要

`pasta_core` は Pasta DSL のシーン/単語テーブル管理を担当します。
DSLパーサーは独立クレート `pasta_dsl` に分離されており、
本クレートはバックエンド（Lua等）に依存しない純粋なデータ構造を提供します。

## アーキテクチャ

登録を集めるレジストリ（`SceneRegistry`・`WordDefRegistry`）と、検索する検索表（`SceneTable`・`WordTable`）の 2 層です。`pasta_lua` はトランスパイル時にレジストリへシーンと単語を登録し、実行時の辞書確定で登録し直したレジストリから検索表を作ります。検索表は RadixMap による前方一致で候補を集め、`RandomSelector` でシャッフルした候補を一巡するまで重複なく順に使います。

詳細は [内部設計: シーン・単語レジストリとシーン検索](https://ekicyou.github.io/pasta/internals/registry-search.html#pasta_core-のレジストリと検索表) を参照してください。

## 公開API

> **パーサーAPI**（`parse_str`, `parse_file`, AST型）は [`pasta_dsl`](https://crates.io/crates/pasta_dsl) クレートにあります。

### Registry

| 型                | 説明                              |
| ----------------- | --------------------------------- |
| `SceneRegistry`   | シーン登録・管理（トランスパイル時の登録と辞書確定での再登録） |
| `WordDefRegistry` | 単語定義登録                      |
| `SceneTable`      | シーン検索（前方一致）            |
| `WordTable`       | 単語検索（前方一致）              |
| `SceneEntry`      | シーン情報エントリ（レジストリ側） |
| `SceneInfo`・`SceneId`・`SceneScope` | 検索表側のシーン情報・ID・スコープ |
| `WordEntry`       | 単語情報エントリ                  |
| `WordCacheKey`    | 単語の選択状態のキャッシュキー    |

エラー型は `SceneTableError`・`WordTableError`（`pasta_core::error`。クレート直下にも再エクスポート）です。

### Random

| 型                      | 説明                               |
| ----------------------- | ---------------------------------- |
| `RandomSelector`        | ランダム選択トレイト               |
| `DefaultRandomSelector` | 本番用ランダム実装                 |
| `MockRandomSelector`    | 指定列で巡の順を決めるテスト用実装 |

## 使用例

> **Note**: DSLのパースは [`pasta_dsl`](https://crates.io/crates/pasta_dsl) クレートを使用してください。

### シーンテーブルの構築と検索

```rust
use std::collections::HashMap;
use pasta_core::registry::{SceneRegistry, SceneTable, DefaultRandomSelector};

// トランスパイル時にシーンを登録
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

// 単語定義を登録
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
| fast_radix_trie | 1.2.0      | 前方一致検索（SceneTable・WordTable） |
| rand            | 0.10       | ランダム選択               |

## 関連クレート

- [`pasta_dsl`](https://crates.io/crates/pasta_dsl) - DSLパーサー
- [`pasta_lua`](https://crates.io/crates/pasta_lua) - Luaバックエンド
- [`pasta_shiori`](https://crates.io/crates/pasta_shiori) - SHIORI DLL統合
- [プロジェクト概要](https://github.com/ekicyou/pasta) - pasta プロジェクト全体
- [pasta マニュアル](https://ekicyou.github.io/pasta/) - ゴースト作者向けの使い方とコントリビュータ向けの内部設計

## ライセンス

プロジェクトルートの [LICENSE](https://github.com/ekicyou/pasta/blob/main/LICENSE) ファイルを参照してください。
