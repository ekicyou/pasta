# pasta_sample_ghost

Pasta サンプルゴースト「hello-pasta」の実装クレート。

## 概要

このクレートは、pasta システムの入門者向けサンプルゴーストを提供します。
SHIORI/3.0 プロトコルで動作するミニマルなゴーストとして、インストール直後から動作可能な状態を実現します。

## 特徴

- **自己完結型**: シェル画像を Rust で自動生成（外部素材不要）
- **教育的設計**: pasta.toml に詳細なコメントを付与
- **ukadoc 準拠**: SSP 標準の設定ファイル群を完備
- **pasta DSL のみ**: イベントハンドラを pasta DSL のみで実装

## キャラクター

| キャラ | 一人称 | 口調 | 色 |
|--------|--------|------|-----|
| **女の子** (sakura) | わたし | 標準語、丁寧めでかわいい | ライトブルー (#4A90D9) |
| **男の子** (kero) | ぼく | 標準語、少し生意気 | ライトグリーン (#4AD98A) |

## ディレクトリ構成

```
crates/pasta_sample_ghost/
├── src/
│   ├── lib.rs              # 公開API
│   ├── image_generator.rs  # ピクトグラム画像生成
│   ├── config_templates.rs # 設定ファイルテンプレート
│   └── scripts.rs          # pasta DSL スクリプト
├── tests/
│   ├── common/mod.rs       # テストヘルパー
│   └── integration_test.rs # 統合テスト
└── ghosts/                 # 生成された配布物
    └── hello-pasta/        # ゴーストID
```

## 使用方法

### ゴースト生成

```rust
use pasta_sample_ghost::{generate_ghost, GhostConfig};

let config = GhostConfig::default();
generate_ghost(Path::new("./output"), &config)?;
```

### テスト実行

```powershell
# pasta_shiori DLL を先にビルド
cargo build --release --target i686-pc-windows-msvc -p pasta_shiori

# テスト実行
cargo test -p pasta_sample_ghost
```

## ライセンス

MIT OR Apache-2.0
