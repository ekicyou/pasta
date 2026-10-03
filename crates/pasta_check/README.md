# pasta_check

ゴースト（「伺か」デスクトップマスコット）の配布パッケージ（`.nar`）と
SSP ネットワーク更新ファイル（`updates.txt`）を作成する CLI ツールです。

ゴースト名やパスに依存しない汎用ツールであり、任意のゴーストに対して使えます。

## インストール

```bash
cargo install pasta_check
```

## 使い方

### 基本構文

```
pasta_check <command> [options]
pasta_check --help
pasta_check --version
```

### release サブコマンド

ゴースト開発フォルダーからリリースパッケージを作成します。

```
pasta_check release --target <path> --release <path> --nar <path> [--copy <path>]...
```

| オプション | 必須 | 説明 |
|-----------|------|------|
| `--target <path>` | ✅ | ゴースト開発フォルダー（`ghost/master` 等を含むルート） |
| `--release <path>` | ✅ | リリース出力先フォルダー（毎回クリーンされる） |
| `--nar <path>` | ✅ | 出力 NAR ファイルパス（親ディレクトリは自動作成） |
| `--copy <path>` | | 上書きコピー元フォルダー（複数指定可、後勝ち） |

#### 実行フロー（5ステップ）

```
[1/5] Preparing release folder   ← --release を削除して新規作成、--nar の位置の前回の nar を削除
[2/5] Copying target files       ← --target → --release に再帰コピー
[3/5] Applying overlay copies    ← --copy（指定があれば）上書きコピー
[4/5] Generating update files    ← 同梱バルーンを判定し、ゴースト用・バルーン用の updates.txt を自動生成
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

## 仕様メモ

### updates.txt（SSP ネットワーク更新・Version 3 形式）

- 1 行目: `charset,UTF-8`
- 以降: `file,<path>\x01<md5>\x01size=<bytes>\x01date=<YYYY-MM-DDTHH:MM:SS>\x01` （CRLF 区切り）
- エントリはパスの辞書順でソートされる（決定的出力）
- `ghost/master/` が存在する場合は同内容の `updates.txt` をそこにもコピーする
- 一覧から除外: ディレクトリ `profile/`・`var/`、ファイル `updates2.dau`・`updates.txt`・`developer_options.txt`、同梱バルーンのフォルダ配下

### 同梱バルーン

- 段 4 は `--copy` 上書き後の配布フォルダ直下の `install.txt` を読む。`install.txt` が無ければ同梱バルーンは無いものとする
- `install.txt` があるときは、同梱バルーンの有無にかかわらず UTF-8 で、1 行目（BOM があればその直後）が `charset,UTF-8` でなければならない（文字コードの変換はしない）
- バルーン指定（`balloon[N].source.directory`・`balloon[N].directory`。番号なし → `balloon0` → `balloon1` … の順）が指す配布フォルダ内のフォルダを同梱バルーンとする。置き場所は任意で、階層付きの値は `source.directory` にだけ書ける
- 同梱バルーンのフォルダ配下はゴースト用 `updates.txt` から除き、フォルダ直下にバルーン用 `updates.txt`（パスはバルーンのフォルダが基準・`ghost/master/` への複製なし・除外規則はゴースト用と同じ）を生成して NAR に封入する
- 同梱バルーンのフォルダ直下には `descript.txt` が必要で、`install.txt` と同じ UTF-8 の規則を適用する。`homeurl` が無いか値が空なら警告を出す（NAR は作成する）
- バルーン指定の不正（キーの重複・欠番の後ろの番号、値が空・絶対パス・`..`・`profile`/`var` 等）、フォルダが無い、`ghost/master` や別の同梱バルーンとの入れ子、UTF-8 の規則違反はエラーで止まり、NAR を作らない

### 前回の NAR の削除

- 段 1 で配布フォルダを準備した後に `--nar` の位置の前回の NAR を削除する（`--nar` がフォルダなど削除できないときは段 1 でエラー）
- 段 5 で失敗したときは作りかけの NAR も削除する。削除した後の段でエラーになったときは `--nar` の位置に NAR が残らない（古い NAR を最新と誤って配らないため）

### NAR（ZIP）封入規則

- `profile/` ディレクトリは封入しない（ユーザーデータ・フレームワーク自己展開先の保護）
- シンボリックリンクはコピー・封入ともにスキップする
- ZIP エントリ名の `..` コンポーネントは常時実行時検査で拒否する（パストラバーサル防御）

## ソース構成

```
pasta_check/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI 引数解析（lexopt）・ディスパッチ
│   ├── release.rs       # release サブコマンド（5ステップパイプライン）
│   ├── copy.rs          # 再帰コピー・release フォルダー初期化
│   ├── balloon.rs       # 同梱バルーン判定（install.txt・descript.txt の読み取り）
│   ├── update_files.rs  # updates.txt 生成（MD5・ISO 8601 日時）
│   └── nar.rs           # NAR（ZIP）アーカイブ作成
└── tests/
    └── cli_test.rs      # バイナリ経由の CLI 統合テスト（終了コード・入出力契約）
```

## 依存クレート

| クレート | 用途 |
|---------|------|
| `lexopt` | CLI 引数パーサー |
| `md5` | updates.txt の MD5 ハッシュ（SSP 仕様準拠のファイル変更検出用途・非暗号学的） |
| `zip` | NAR（ZIP）アーカイブ作成 |

## ライセンス

MIT OR Apache-2.0
