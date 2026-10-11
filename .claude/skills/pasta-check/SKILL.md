---
name: pasta-check
description: 'pasta_check CLIツールのリファレンス。ゴーストリリースビルド（release サブコマンド）、将来的なテスト・検証コマンドを含む。USE FOR: pasta_check, pasta check, ghost release, ゴーストリリース, NAR作成, release.bat, release.ps1, pasta_check release, .nar, リリースビルド, ゴースト配布, updates.txt, build ghost, deploy ghost, publish ghost, リリース手順, pasta_check test, ゴースト検証. DO NOT USE FOR: pasta DSL文法（pasta-ghost-authoringを使用）, Lua API（pasta-lua-codingを使用）, crates.ioパブリッシュ（リリース CI。crates/pasta_sample_ghost/RELEASE.md を参照）.'
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
[1/5] Preparing release folder   ← --release を削除して新規作成、--nar の位置の前回の nar を削除
[2/5] Copying target files       ← --target → --release に再帰コピー
[3/5] Applying overlay copies    ← --copy（指定があれば）上書きコピー
[4/5] Generating update files    ← 同梱バルーンを判定し、ゴースト用・バルーン用の updates.txt を自動生成
[5/5] Creating NAR archive       ← --release を ZIP 圧縮して --nar に出力
```

- 段 1 で配布フォルダを準備した後に前回の nar を消し、段 5 で失敗したときは作りかけの nar も消すため、削除した後の段でエラーになったときは nar が残らない（古い nar を最新と誤って配らない）。
- 段 4 は `--copy` 上書き後の配布フォルダ直下の `install.txt` を読む。`install.txt` があるときは、UTF-8 で 1 行目が `charset,UTF-8` でなければならない（同梱バルーンの有無にかかわらず必須）。`install.txt` が無ければ同梱バルーンは無いものとする。
- `install.txt` のバルーン指定（`balloon[N].source.directory`・`balloon[N].directory`）が指すフォルダを同梱バルーンとし、そのフォルダ配下はゴースト用 `updates.txt` から除き、フォルダ直下にバルーン用 `updates.txt`（パスはバルーンのフォルダが基準）を生成する。
- 判定方法・エラー・警告の詳細は [updates.txt の仕様](./references/updates-txt-spec.md#同梱バルーン) を参照。

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
  1. SHIORI DLL ビルド               3. pasta_check release
  2. DLL/スクリプトを開発フォルダーへ  4. 追加の配布物の作成（hello-pasta は pasta.dll.zip）
                                     5. バージョン確認
                                     6. 大きさの検査とリリース案内表示
```

- 段 3 は pasta_check が汎用的に処理する
- それ以外（Setup Phase と段 4〜6）はゴースト固有の手順（release.ps1 等で実装）
- 段の番号は hello-pasta の `release.ps1` のもの。立ち絵などの素材はコミットしたものをそのまま詰めるので、成果物を生成する段は無い

### ディレクトリ構成例

> 以下はサンプルゴースト (hello-pasta) の例。実際のパスはゴーストごとに異なる。

```
workspace/                               # ワークスペースルート
├── release.bat                          # release.ps1 を呼ぶラッパースクリプト（任意・ワークスペースルートに置く）
├── release/                             # 生成物の出力先（.gitignore で無視・コミットしない）
│   ├── {ghost-name}/                    # --release 出力先
│   │   ├── ghost/master/                # ゴースト本体
│   │   ├── shell/master/                # シェル（画像等）
│   │   ├── balloon/{balloon-name}/      # 同梱バルーンの置き場所の例（install.txt のバルーン指定が指す任意のフォルダ。hello-pasta には無い）
│   │   │   ├── descript.txt
│   │   │   └── updates.txt              # 自動生成（バルーン用）
│   │   ├── install.txt                  # UTF-8・1 行目は charset,UTF-8
│   │   └── updates.txt                  # 自動生成（ゴースト用）
│   ├── {ghost-name}.nar                 # --nar 出力
│   └── pasta.dll.zip                    # 追加の配布物（hello-pasta 固有・release.ps1 の段 4）
└── crates/pasta_sample_ghost/
    ├── release.ps1                      # Setup + Release を統合したスクリプト
    └── ghosts/{ghost-name}/             # --target ゴースト開発フォルダー
```

## 技術仕様

SSP 仕様および NAR フォーマットの詳細:

- [updates.txt の仕様](./references/updates-txt-spec.md) — SSP ネットワーク更新ファイル仕様
- [NAR の仕様](./references/nar-spec.md) — NAR (ZIP) パッケージ仕様

## リリース後の手順

`pasta_check release` が作った `.nar` をどう配布するかは、ゴーストごとに決める（各ゴーストの RELEASE.md 等を参照）。

### pasta リポジトリ（hello-pasta）の場合

- 手元で `release.bat`・`release.ps1`（中で `pasta_check release` を呼ぶ）を実行して `release/` にできる成果物は、動作確認用である。コミットしない（`release/` は `.gitignore` で無視）。
- 配布物（`pasta.dll.zip`・`hello-pasta.nar`・VSIX）の公開は、リリースタグ `vX.Y.Z` の push を契機にリリース CI（`.github/workflows/release.yml`）が行う。CI はタグのソースから配布物を作り直し、GitHub Release（題名 `pasta vX.Y.Z`・リリースノートはコミット履歴から生成）を作って添付する。
- 手元で `gh release create` を実行しない。手順と失敗したときの回復は `crates/pasta_sample_ghost/RELEASE.md` を参照。

## トラブルシューティング

| 問題 | 原因 | 対処 |
|------|------|------|
| `pasta_shiori build failed` | 32bit ターゲット未インストール | `rustup target add i686-pc-windows-msvc` |
| `pasta.dll not found` | DLL ビルドをスキップしたが未ビルド | DLL ビルドを先に実行 |
| `pasta_check release failed` | パス不正 or ディスク容量 | エラーメッセージの詳細を確認 |
| `Warning: bundled balloon "<フォルダ>": descript.txt has no homeurl` | 同梱バルーンの `descript.txt` に `homeurl` が無いか値が空（nar は作成される） | バルーンをネットワーク更新させるなら `descript.txt` に `homeurl` を書く |
| `Error: install.txt is not UTF-8: ...` | `install.txt` が UTF-8 でない、または 1 行目が `charset,UTF-8` でない | `install.txt` を UTF-8 で保存し、1 行目を `charset,UTF-8` にする |
| `Error: <フォルダ>/descript.txt is not UTF-8: ...` | 同梱バルーンの `descript.txt` が同上 | `descript.txt` を UTF-8 で保存し、1 行目を `charset,UTF-8` にする |
| `Error: install.txt: invalid value for <キー>: ...` | バルーン指定の値が不正（括弧内が理由: 空・`directory` に区切り・絶対パス・`..`・`profile`/`var`） | 配布フォルダからの相対パスで指定する（階層付きの値は `source.directory` だけに書ける） |
| `Error: install.txt: duplicate key ...`／`... is never read by the baseware ...` | バルーン指定のキーの重複・番号の先頭の 0・欠番の後ろの番号 | キーを 1 行ずつ、番号は `balloon0` から欠番なく書く |
| `Error: install.txt: bundled balloon folder "<フォルダ>" (<キー>) does not exist in the release folder` | 指定したフォルダが `--copy` 上書き後の配布フォルダに無い（同名のファイル・シンボリックリンクも不可） | フォルダを配布物に含めるか、指定を実在のフォルダに合わせる |
| `Error: bundled balloon "<フォルダ>": descript.txt not found` | 同梱バルーンのフォルダ直下に `descript.txt` が無い | バルーンのフォルダ直下に `descript.txt` を置く |
| `Error: file name is not valid Unicode: <パス>` | 配布フォルダ内のファイル・フォルダ名が Unicode として正しくない（名前を化けさせて封入しないため止める） | そのファイル・フォルダの名前を正しい名前に付け直す |
| `Error: install.txt: bundled balloon folder "<フォルダ>" (<キー>) overlaps with ...` | 指定したフォルダが `ghost/master` や別の同梱バルーンと同じ・上位・配下 | 互いに入れ子にならないフォルダを指定する |
| updates.txt が仕様（UTF-8・1 行目 `charset,UTF-8`）と違う | pasta_check のバグ | [updates.txt 仕様](./references/updates-txt-spec.md)と照合 |

`Error:` で始まるものは終了コード 1 で終了し、nar を作成しない。メッセージと不正な値の理由の一覧は [updates.txt 仕様のエラー](./references/updates-txt-spec.md#エラー) を参照。
