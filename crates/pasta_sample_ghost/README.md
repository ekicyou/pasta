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
│   ├── lib.rs              # 公開API（画像＋surfaces.txt 生成）
│   ├── image_generator.rs  # ピクトグラム画像生成
│   ├── config_templates.rs # surfaces.txt 生成
│   └── scripts.rs          # テスト用 dist-src ヘルパー
├── dist-src/               # テキスト系配布ファイル（Single Source of Truth）
│   ├── install.txt
│   ├── ghost/master/
│   │   ├── descript.txt
│   │   ├── pasta.toml
│   │   └── dic/
│   │       ├── actors.pasta
│   │       ├── boot.pasta
│   │       ├── talk.pasta
│   │       └── click.pasta
│   └── shell/master/
│       └── descript.txt
├── tests/
│   ├── common/mod.rs            # テストヘルパー
│   ├── dist_src_validation_test.rs # dist-src ディレクトリ検証
│   └── integration_test.rs      # 統合テスト
└── ghosts/                  # 生成された配布物
    └── hello-pasta/         # ゴーストID
```

## 使用方法

### セットアップ（初回のみ）

**pasta.dll と Lua ランタイムを配置**

```powershell
# crates/pasta_sample_ghost/ フォルダで release.bat をダブルクリック
# または PowerShell で実行（ビルド＋セットアップ＋リリースパッケージ作成）
.\release.ps1

# DLL ビルドをスキップする場合（既にビルド済みの場合）
.\release.ps1 -SkipDllBuild

# セットアップをスキップしてリリースのみ実行する場合
.\release.ps1 -SkipSetup
```

このスクリプトは以下を実行します：
1. pasta_shiori.dll (32bit) をビルド
2. dist-src/ のテキストファイルを robocopy でコピー
3. ゴースト画像を生成（surface*.png + surfaces.txt）
4. ghosts/hello-pasta/ghost/master/ に pasta.dll と scripts/ を配置
5. updates2.dau / updates.txt を生成
6. バリデーション＆ .nar パッケージ作成

**注意**: `cargo run` は画像ファイル（surface*.png）と surfaces.txt のみ生成します。テキスト系配布ファイルは `dist-src/` に配置済みで、`release.ps1` の robocopy ステップでコピーされます。

### 配布物の確認

```powershell
# テストを実行（.pasta と .png が自動生成される）
cargo test -p pasta_sample_ghost

# 配布物の場所
crates/pasta_sample_ghost/ghosts/hello-pasta/
```

この `ghosts/hello-pasta/` フォルダをそのまま SSP にインストール可能です。
```

スクリプトは以下を自動実行します:
1. `pasta_shiori.dll` のビルド（32bit Windows）
2. `dist-src/` テキストファイルの robocopy コピー
3. 画像生成（`cargo run`）
4. `pasta.dll` の配置
5. Lua ランタイム (`scripts/`) のコピー

### 手動ビルド手順

```powershell
# 1. pasta_shiori DLL をビルド
cargo build --release --target i686-pc-windows-msvc -p pasta_shiori

# 2. 配布ディレクトリを作成
$dist = "dist/hello-pasta"
Copy-Item -Recurse "crates/pasta_sample_ghost/ghosts/hello-pasta" $dist

# 3. DLL をコピー
Copy-Item "target/i686-pc-windows-msvc/release/pasta.dll" "$dist/ghost/master/pasta.dll"

# 4. Lua ランタイムをコピー
Copy-Item -Recurse "crates/pasta_lua/scripts" "$dist/ghost/master/scripts"
```

### ゴースト生成API

`generate_ghost()` は画像ファイル（surface*.png）と surfaces.txt のみを生成します。
テキスト系配布ファイルは `dist-src/` に配置済みで、`release.ps1` の robocopy でコピーされます。

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

`release.ps1` 完了後の `ghosts/hello-pasta/` の構成:

```
hello-pasta/
├── install.txt                 # ← dist-src/ (robocopy)
├── ghost/
│   └── master/
│       ├── pasta.dll           # SHIORI DLL (cargo build)
│       ├── pasta.toml          # ← dist-src/ (robocopy)
│       ├── descript.txt        # ← dist-src/ (robocopy)
│       ├── dic/                # pasta DSL スクリプト
│       │   ├── actors.pasta    # ← dist-src/ (robocopy)
│       │   ├── boot.pasta      # ← dist-src/ (robocopy)
│       │   ├── talk.pasta      # ← dist-src/ (robocopy)
│       │   └── click.pasta     # ← dist-src/ (robocopy)
│       └── scripts/            # Lua ランタイム（pasta_lua/scripts/）
└── shell/
    └── master/
        ├── descript.txt        # ← dist-src/ (robocopy)
        ├── surfaces.txt        # ← cargo run (generate_ghost)
        └── surface*.png        # ← cargo run (generate_ghost)
```

## ライセンス

MIT OR Apache-2.0
