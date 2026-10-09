# updates.txt 仕様

SSP（伺かベースウェア）のネットワーク更新ファイル仕様。
pasta_check の `generate_update_files()` が自動生成する（release の段 4）。

## フォーマット (Version 3)

| 項目 | 値 |
|------|-----|
| エンコーディング | UTF-8 |
| 改行コード | CRLF (`\r\n`) |
| 1行目 | `charset,UTF-8` |
| 以降の行フォーマット | `file,<path>\x01<md5>\x01size=<bytes>\x01date=<ISO8601>\x01` |
| MD5 | 32文字小文字16進数 |
| パス区切り | スラッシュ (`/`) |
| フィールド区切り | SOH (`\x01`) |
| 行の順 | パスの辞書順 |

> SSP は Version 3 (charset ヘッダー付き) を認識し、UTF-8 として処理する。

md5 と size は配布フォルダ内の実ファイルのバイト列から計算する。pasta_check は `updates.txt` 以外のファイルの内容（改行コードを含む）を変更しない。

## 生成するファイル

| ファイル | 対象 | パスの基準 |
|---------|------|-----------|
| `updates.txt`（配布フォルダ直下） | 配布フォルダ全体から、同梱バルーンのフォルダ配下を除いたもの | 配布フォルダ |
| `ghost/master/updates.txt` | 直下の `updates.txt` の複製（`ghost/master` があるとき） | 配布フォルダ |
| `<同梱バルーンのフォルダ>/updates.txt` | そのバルーンのフォルダ配下 | そのバルーンのフォルダ |

- 対象ファイルが 0 件のときは、その `updates.txt` を生成しない（ゴースト用・バルーン用とも）。
- バルーン用は同梱バルーンのフォルダ直下（`descript.txt` と同じ階層）にだけ書き、`ghost/master` へは複製しない。
- `--target`・`--copy` 由来の `updates.txt` が既にあれば、生成したもので置き換える。

## 除外ルール

ゴースト用・バルーン用のどちらの収集にも同じ規則を適用する。名前で判定し、どの階層にあっても除外する。シンボリックリンクはたどらない。

### 除外ディレクトリ

以下のディレクトリ配下は updates.txt に含めない:

| ディレクトリ | 理由 |
|-------------|------|
| `profile/` | ユーザー固有データ（更新で上書きしてはならない） |
| `var/` | 実行時変数データ |

### 除外ファイル

以下のファイル名は updates.txt に含めない:

| ファイル | 理由 |
|---------|------|
| `updates2.dau` | 旧形式の更新ファイル |
| `updates.txt` | 自分自身 |
| `developer_options.txt` | 開発用オプション |

### 同梱バルーンのフォルダ（ゴースト用だけ）

同梱バルーンのフォルダ配下は、ゴースト用 `updates.txt`（直下・`ghost/master`）に 1 行も載せない。名前ではなく位置で判定するため、同じ名前のフォルダでも別の位置（例: `ghost/master/<同名>/`）にあるものは除外しない。

## 同梱バルーン

ゴーストと一緒に配るバルーン。SSP はインストール時に `install.txt` のバルーン指定で振り分けるため、pasta_check も同じ `install.txt` で判定する。

### 判定方法

- 配布フォルダ直下の `install.txt`（`--copy` の上書き後）だけを読む。サブフォルダの `install.txt`（バルーン自身のものなど）は使わない。`install.txt` が無ければ同梱バルーンは無いものとする。
- キーは大文字小文字を区別せず、値の前後の空白は無視する。
- バルーン指定は `balloon` → `balloon0` → `balloon1` → … の順に探す。番号付きは、指定の無い最初の番号で打ち切る。番号なしと番号付きは併存できる。
- 各指定では `<接頭辞>.source.directory` の行があればその値（空でも）を、無ければ `<接頭辞>.directory` の値を同梱バルーンのフォルダとする。
- 値は配布フォルダからの相対パス。`source.directory` だけは `/`・`\` 区切りの階層付きの値（例: `extra\bal1`）を書ける。要素の `.` と空の要素は無視する。
- 値と実在のフォルダ名が大文字小文字だけ違い、ファイルシステムが同じフォルダとみなす場合は、実在の名前で扱う。
- 複数の指定が同じフォルダを指す場合は 1 つの同梱バルーンとして扱う。

### UTF-8 の確認

pasta_check は UTF-8 の `install.txt`・`descript.txt` だけを扱い、文字コードを変換しない。

- 配布フォルダ直下の `install.txt` は、同梱バルーンの有無にかかわらず、1 行目（BOM があればその直後）が `charset,UTF-8` でなければならない（大文字小文字・前後の空白は問わない）。
- 同梱バルーンのフォルダ直下の `descript.txt` にも同じ規則を適用する。

### エラー

次の場合は `Error: ` に続けてメッセージを表示し、終了コード 1 で終了する。段 5 に進まないため nar は作成しない。最初に見つかった 1 件だけを表示する。

| 条件 | メッセージ |
|------|-----------|
| UTF-8 として正しくないバイト列がある | `install.txt is not UTF-8: contains invalid UTF-8 byte sequence` |
| 1 行目が `charset,UTF-8` でない | `install.txt is not UTF-8: the first line must be "charset,UTF-8" (pasta_check supports UTF-8 only)` |
| バルーン指定のキーが複数行ある | `install.txt: duplicate key <キー>` |
| 番号の先頭に 0 がある（例: `balloon01`） | `install.txt: <キー> is never read by the baseware (numbers must not have leading zeros)` |
| 欠番より後ろの番号の指定がある | `install.txt: <キー> is never read by the baseware because balloon<欠番> is missing (numbered entries must not have gaps)` |
| 値が不正 | `install.txt: invalid value for <キー>: "<値>" (<理由>)` |
| フォルダが無い（同名のファイル・シンボリックリンクを含む） | `install.txt: bundled balloon folder "<フォルダ>" (<キー>) does not exist in the release folder` |
| `ghost/master` と重なる | `install.txt: bundled balloon folder "<フォルダ>" (<キー>) overlaps with ghost/master` |
| 別の同梱バルーンと重なる | `install.txt: bundled balloon folder "<フォルダ>" (<キー>) overlaps with bundled balloon folder "<フォルダ>" (<キー>)` |
| バルーンに `descript.txt` が無い | `bundled balloon "<フォルダ>": descript.txt not found` |

- `<キー>` は小文字で表示する。`descript.txt` の UTF-8 エラーでは先頭の `install.txt` が `<フォルダ>/descript.txt` になる。
- 不正な値の `<理由>`（表の上から順に判定し、最初に当てはまったものを表示する。例: `balloon.directory,/foo`・`balloon.directory,C:\x` は `path separator is not allowed in directory`）:

| 理由 | 条件 |
|------|------|
| `empty` | 値が空 |
| `path separator is not allowed in directory` | `directory` の値に `/`・`\` がある |
| `absolute path` | 値が `/`・`\` で始まる（ルート・UNC）、または `:` を含む（ドライブ名） |
| `parent directory reference` | 要素に `..` がある |
| `empty` | `.` と空の要素を除くと何も残らない（例: `.`・`./`） |
| `excluded folder (profile/var)` | 要素に `profile`・`var` がある（大文字小文字は問わない） |

- 重なりは、同じフォルダ・上位・配下を指す。`ghost/master` との比較（`ghost` も重なる）は大文字小文字を区別しない。

### 警告

同梱バルーンの `descript.txt` に `homeurl` の行が無いか値が空のとき、stderr に次の警告を表示する。生成物と終了コード（0）は警告が無い場合と同じ。

```
Warning: bundled balloon "<フォルダ>": descript.txt has no homeurl; the balloon cannot be network-updated
```

### 進捗表示

段 4 では、生成したバルーン用 `updates.txt` ごとにフォルダと登録件数を表示する。

```
[4/5] Generating update files...
  Generated updates.txt (52 entries)
  Generated balloon/emo2-kakukaku/updates.txt (20 entries)
```

## 出力例

ゴースト用（配布フォルダ直下）:

```
charset,UTF-8
file,ghost/master/descript.txt\x01a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4\x01size=1234\x01date=2026-03-26T12:00:00\x01
file,ghost/master/dic/02-talk.pasta\x01b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5\x01size=5678\x01date=2026-03-26T12:00:00\x01
file,install.txt\x01e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2\x01size=456\x01date=2026-03-26T12:00:00\x01
file,shell/master/surface0.png\x01d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1\x01size=90123\x01date=2026-03-26T12:00:00\x01
```

バルーン用（`balloon/emo2-kakukaku/updates.txt`。パスはバルーンのフォルダが基準）:

```
charset,UTF-8
file,arrow0.png\x01c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6\x01size=345\x01date=2026-03-26T12:00:00\x01
file,descript.txt\x01f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3\x01size=789\x01date=2026-03-26T12:00:00\x01
```

## 実装箇所

- 生成: `crates/pasta_check/src/update_files.rs`
- 同梱バルーンの判定: `crates/pasta_check/src/balloon.rs`
- MD5 計算: `md5` クレート

## SSP 参考仕様

- SSP が `updates.txt` を読み取り、ローカルファイルの MD5 と比較
- 不一致のファイルのみダウンロードして更新
- `profile/` や `var/` を除外することでユーザーデータを保護
- バルーンの `updates.txt` はバルーンのフォルダ直下に置き、バルーンのネットワーク更新には `descript.txt` の `homeurl` が要る
