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
| **女の子** (sakura) | わたし | 標準語、丁寧めでかわいい | 赤 (#DC3545) |
| **男の子** (kero) | ぼく | 標準語、少し生意気 | 青 (#007BFF) |

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
    └── hello-pasta/        # ゴーストID（テンプレート）
```

## 使用方法

### 配布可能なゴーストをビルド

**推奨: PowerShell スクリプトを使用**

```powershell
# ワークスペースルートで実行
.\scripts\build-ghost.ps1

# 出力: dist/hello-pasta/
```

スクリプトは以下を自動実行します:
1. `pasta_shiori.dll` のビルド（32bit Windows）
2. テンプレートのコピー
3. `pasta.dll` の配置
4. Lua ランタイム (`scripts/`) のコピー

### 手動ビルド手順

```powershell
# 1. pasta_shiori DLL をビルド
cargo build --release --target i686-pc-windows-msvc -p pasta_shiori

# 2. 配布ディレクトリを作成
$dist = "dist/hello-pasta"
Copy-Item -Recurse "crates/pasta_sample_ghost/ghosts/hello-pasta" $dist

# 3. DLL をコピー
Copy-Item "target/i686-pc-windows-msvc/release/pasta_shiori.dll" "$dist/ghost/master/pasta.dll"

# 4. Lua ランタイムをコピー
Copy-Item -Recurse "crates/pasta_lua/scripts" "$dist/ghost/master/scripts"
```

### ゴースト生成API

```rust
use pasta_sample_ghost::{generate_ghost, GhostConfig};

let config = GhostConfig::default();
generate_ghost(Path::new("./output"), &config)?;
```

### テスト実行

```powershell
cargo test -p pasta_sample_ghost
```

## 配布物の構成

ビルド後の `dist/hello-pasta/` の構成:

```
hello-pasta/
├── install.txt
├── readme.txt
├── ghost/
│   └── master/
│       ├── pasta.dll           # SHIORI DLL
│       ├── pasta.toml          # pasta 設定
│       ├── descript.txt        # ゴースト設定
│       ├── dic/                # pasta DSL スクリプト
│       │   ├── boot.pasta
│       │   ├── talk.pasta
│       │   └── click.pasta
│       └── scripts/            # Lua ランタイム（pasta_lua/scripts/）
└── shell/
    └── master/
        ├── descript.txt
        ├── surfaces.txt
        └── surface*.png        # ピクトグラム画像
```

## ライセンス

MIT OR Apache-2.0
