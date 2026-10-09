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

サンプルゴーストの実体は `ghosts/hello-pasta/` に**完全なゴースト一式として直接配置**されており、これが配布物の Single Source of Truth（SSOT）です。テキスト系ファイル（`descript.txt` / `pasta.toml` / `dic/*.pasta` / `install.txt`）は手書きの正本、画像・DLL は生成物で、いずれも同じツリー内に置かれます。

```
crates/pasta_sample_ghost/
├── src/
│   ├── lib.rs              # 公開API（画像＋surfaces.txt 生成）
│   ├── main.rs             # 配布物生成 CLI（cargo run -p pasta_sample_ghost）
│   ├── image_generator.rs  # ピクトグラム画像生成
│   ├── config_templates.rs # surfaces.txt 生成
│   └── scripts.rs          # ghosts/hello-pasta の辞書(.pasta)を読む検証テスト
├── ghosts/                 # サンプルゴースト本体（SSOT・配布物）
│   └── hello-pasta/        # ゴーストID
│       ├── install.txt
│       ├── ghost/master/   # descript.txt, pasta.toml, dic/*.pasta（手書きSSOT）＋ pasta.dll, THIRD_PARTY_LICENSES.txt, scripts/（生成物・コミットしない）
│       └── shell/master/   # descript.txt（手書き）＋ surfaces.txt, surface*.png（生成物）
├── release.ps1             # ビルド＋セットアップ＋.nar・pasta.dll.zip 作成（動作確認用）
├── build.rs                # ビルドスクリプト
└── tests/
    ├── common/mod.rs                 # テストヘルパー
    ├── dist_src_validation_test.rs   # 配布ファイル構成の検証 ※ファイル名は旧称（dist-src 廃止済み）
    ├── integration_test.rs           # 統合テスト
    └── self_deploy_integration_test.rs # 実 .pasta を PastaLoader で parse/transpile 検証
```

> **注**: かつてテキスト配布ファイルは `dist-src/` に分離し `release.ps1` の robocopy で配布先へコピーする方式でしたが、現在は廃止済みです。テキストファイルは `ghosts/hello-pasta/` に直接置く SSOT 方式に統一されています。

## 使用方法

### セットアップ／リリース（`release.ps1`）

```powershell
# crates/pasta_sample_ghost/ フォルダで PowerShell から実行
# （ビルド＋セットアップ＋リリースパッケージ作成）
.\release.ps1

# DLL ビルドをスキップする場合（既にビルド済みの場合）
.\release.ps1 -SkipDllBuild

# セットアップをスキップしてリリースのみ実行する場合
.\release.ps1 -SkipSetup
```

このスクリプトは以下の 7 ステップを実行します:

1. `pasta_shiori` DLL（32bit Windows）をビルド
2. ゴースト画像を生成（`cargo run` → surface*.png + surfaces.txt）
3. `pasta.dll`・第三者ライセンス表示（`THIRD_PARTY_LICENSES.txt`）・Lua ランタイム（`scripts/`）を `ghosts/hello-pasta/ghost/master/` に配置
4. `pasta_check release` を実行（updates.txt / `release/hello-pasta.nar` 作成）
5. `release/pasta.dll.zip` を作成（`pasta.dll` と `THIRD_PARTY_LICENSES.txt`）
6. バージョン整合チェック
7. リリース手順の表示

**注**: テキスト系配布ファイル（`descript.txt` / `pasta.toml` / `dic/*.pasta` / `install.txt`）は `ghosts/hello-pasta/` に手書きで配置済みのため、コピー工程はありません。`release.ps1` は生成物（画像・DLL・ランタイム）の配置とパッケージングのみを担います。

**注**: `release.ps1` の成果物は手元の動作確認用で、コミットの対象ではありません。手順 3 で `ghost/master/` に置く `pasta.dll`・`THIRD_PARTY_LICENSES.txt`・`scripts/` と、出力先の `release/` は `.gitignore` で無視されます（シェルの画像 `surface*.png`・`surfaces.txt` は従来どおり追跡します）。配布物の公開は、リリースタグ `vX.Y.Z` の push を契機にリリース CI（`.github/workflows/release.yml`）がソースから作り直して行います。手順は [RELEASE.md](RELEASE.md) を参照してください。

### 配布物の確認

```powershell
# テストを実行（辞書検証・画像生成等）
cargo test -p pasta_sample_ghost

# 配布物の場所（このフォルダをそのまま SSP にインストール可能）
crates/pasta_sample_ghost/ghosts/hello-pasta/
```

### 手動ビルド手順

```powershell
# 1. pasta_shiori DLL をビルド
cargo build --release --target i686-pc-windows-msvc -p pasta_shiori

# 2. ゴースト一式をコピー
$dist = "dist/hello-pasta"
Copy-Item -Recurse "crates/pasta_sample_ghost/ghosts/hello-pasta" $dist

# 3. DLL をコピー
Copy-Item "target/i686-pc-windows-msvc/release/pasta.dll" "$dist/ghost/master/pasta.dll"

# 4. Lua ランタイムをコピー
Copy-Item -Recurse "crates/pasta_lua/scripts" "$dist/ghost/master/scripts"
```

### ゴースト生成API

`generate_ghost()` は画像ファイル（surface*.png）と surfaces.txt **のみ**を生成します。
テキスト系配布ファイルは `ghosts/hello-pasta/` に手書き配置済みのため、本 API は生成しません。

```rust
use pasta_sample_ghost::generate_ghost;

generate_ghost(Path::new("./output"))?;
```

### テスト実行

```powershell
cargo test -p pasta_sample_ghost
```

## 配布物の構成

`ghosts/hello-pasta/` の構成（凡例: **[SSOT]** = 手書き正本 / **[gen]** = 生成物）:

```
hello-pasta/
├── install.txt                 # [SSOT]
├── ghost/
│   └── master/
│       ├── descript.txt        # [SSOT]
│       ├── pasta.toml          # [SSOT]
│       ├── dic/                # pasta DSL 辞書 [SSOT]（段階ごとに 1 ファイル。STAGES.md 参照）
│       │   ├── 01-boot.pasta     # 1 段: 起動の一言（OnBoot）
│       │   ├── 02-talk.pasta     # 2 段: 2 人の掛け合い（＊会話 ＝ ランダムトーク）
│       │   ├── 03-face.pasta     # 3 段: アクター辞書と表情
│       │   ├── 04-variety.pasta  # 4 段: 同名シーン・単独 ＊ で毎回ちがう話
│       │   ├── 05-words.pasta    # 5 段: 単語の定義と参照
│       │   ├── 06-hour.pasta     # 6 段: 時報
│       │   ├── 07-greeting.pasta # 7 段: 挨拶（OnGhostChanged/Changing・OnFirstBoot・OnClose）
│       │   ├── 08-touch.pasta    # 8 段: ダブルクリックへの反応（＄ｒ４）
│       │   ├── 09-choice.pasta   # 9 段: 選択肢
│       │   ├── 10-save.pasta     # 10 段: 保存される変数（＄＊回数）
│       │   ├── 11-jump.pasta     # 11 段: Call・前方一致・ローカルシーン・チェイントーク
│       │   └── 12-lua.pasta      # 12 段: Lua ブロックの関数を呼ぶ
│       ├── pasta.dll           # [gen] SHIORI DLL（cargo build・コミットしない）
│       ├── THIRD_PARTY_LICENSES.txt # [gen] 第三者ライセンス表示（cargo about・コミットしない）
│       └── scripts/            # [gen] Lua ランタイム（pasta_lua/scripts/ の写し・コミットしない）
└── shell/
    └── master/
        ├── descript.txt        # [SSOT]
        ├── surfaces.txt        # [gen] cargo run（generate_ghost）
        └── surface*.png        # [gen] cargo run（generate_ghost）
```

段階表（各段で教える表現・追加するファイル・検証するイベント）は [`STAGES.md`](STAGES.md) にあります。段階 N の辞書は `01`〜`NN` のファイルの集まりで、12 段目（全ファイル）が配布版の辞書です。

## ライセンス

MIT OR Apache-2.0
