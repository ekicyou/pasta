---
name: pasta-check
description: 'pasta_check CLIツールのリファレンス。ゴーストリリースビルド（release サブコマンド）、将来的なテスト・検証コマンドを含む。USE FOR: pasta_check, pasta check, ghost release, ゴーストリリース, NAR作成, release.bat, release.ps1, pasta_check release, .nar, リリースビルド, ゴースト配布, updates.txt, build ghost, deploy ghost, publish ghost, リリース手順, pasta_check test, ゴースト検証. DO NOT USE FOR: pasta DSL文法（pasta-ghost-authoringを使用）, Lua API（pasta-lua-codingを使用）, crates.ioパブリッシュ（release-workflow specを参照）.'
argument-hint: 'サブコマンド名（release等）やゴースト名、オプションを指定'
---

# pasta_check — ゴーストリリースツール

pasta_check はゴーストの配布パッケージ（.nar）を作成する CLI ツール。
ゴースト名やパスに依存しない汎用ツールであり、任意のゴーストに対して使える。

## インストール

crates.io から `cargo install` でインストールする。

```bash
cargo install pasta_check
```

インストール後は `pasta_check` コマンドとして使える（`~/.cargo/bin` にパスが通っている前提）。

## pasta_check CLI

### 基本構文

```
pasta_check <command> [options]
pasta_check --help
pasta_check --version
```

### release サブコマンド

ゴースト開発フォルダーからリリースパッケージを作成する。

```
pasta_check release --target <path> --release <path> --nar <path> [--copy <path>]...
```

| オプション | 必須 | 説明 |
|-----------|------|------|
| `--target <path>` | ✅ | ゴースト開発フォルダー（ghost/master 等を含むルート） |
| `--release <path>` | ✅ | リリース出力先フォルダー（毎回クリーンされる） |
| `--nar <path>` | ✅ | 出力 NAR ファイルパス |
| `--copy <path>` | | 上書きコピー元フォルダー（複数指定可、後勝ち） |

#### release の実行フロー（5ステップ）

```
[1/5] Preparing release folder   ← --release を削除して新規作成
[2/5] Copying target files       ← --target → --release に再帰コピー
[3/5] Applying overlay copies    ← --copy（指定があれば）上書きコピー
[4/5] Generating update files    ← updates.txt を自動生成
[5/5] Creating NAR archive       ← --release を ZIP 圧縮して --nar に出力
```

#### 使用例

```powershell
# 最小構成
pasta_check release `
  --target path/to/ghost `
  --release release/my-ghost `
  --nar release/my-ghost.nar

# オーバーレイ付き（ビルド成果物を上書き）
pasta_check release `
  --target path/to/ghost `
  --release release/my-ghost `
  --nar release/my-ghost.nar `
  --copy path/to/build-output `
  --copy path/to/extra-files
```

## リリースワークフロー

ゴーストのフルリリースは 2 フェーズ構成。

```
[Setup Phase]                      [Release Phase]
  1. SHIORI DLL ビルド               4. pasta_check release
  2. ゴースト固有の成果物生成         5. バージョン確認
  3. DLL/スクリプトを開発フォルダーへ  6. リリース案内表示
```

- Setup Phase はゴースト固有の手順（release.ps1 等で実装）
- Release Phase は pasta_check が汎用的に処理

### ディレクトリ構成例

> 以下はサンプルゴースト (hello-pasta) の例。実際のパスはゴーストごとに異なる。

```
workspace/
├── release.bat                          # ラッパースクリプト（任意）
├── release/
│   ├── {ghost-name}/                    # --release 出力先
│   │   ├── ghost/master/                # ゴースト本体
│   │   ├── shell/master/                # シェル（画像等）
│   │   ├── install.txt
│   │   └── updates.txt                  # 自動生成
│   └── {ghost-name}.nar                 # --nar 出力
└── crates/pasta_sample_ghost/
    ├── release.ps1                      # Setup + Release を統合したスクリプト
    └── ghosts/{ghost-name}/             # --target ゴースト開発フォルダー
```

## 技術仕様

SSP 仕様および NAR フォーマットの詳細:

- [updates.txt の仕様](./references/updates-txt-spec.md) — SSP ネットワーク更新ファイル仕様
- [NAR の仕様](./references/nar-spec.md) — NAR (ZIP) パッケージ仕様

## リリース後の手順

NAR 作成後は GitHub Release で配布:

```powershell
gh release create v{VERSION} "release/{ghost-name}.nar" `
  --title "{ghost-name} v{VERSION}" `
  --notes-file release-notes.md
```

リリースノートのテンプレートは各ゴーストの RELEASE.md を参照。

## トラブルシューティング

| 問題 | 原因 | 対処 |
|------|------|------|
| `pasta_shiori build failed` | 32bit ターゲット未インストール | `rustup target add i686-pc-windows-msvc` |
| `pasta.dll not found` | DLL ビルドをスキップしたが未ビルド | DLL ビルドを先に実行 |
| `pasta_check release failed` | パス不正 or ディスク容量 | エラーメッセージの詳細を確認 |
| updates.txt が Shift_JIS でない | pasta_check のバグ | [updates.txt 仕様](./references/updates-txt-spec.md)と照合 |
