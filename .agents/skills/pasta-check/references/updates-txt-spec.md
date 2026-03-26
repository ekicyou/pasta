# updates.txt 仕様

SSP（伺かベースウェア）のネットワーク更新ファイル仕様。
pasta_check の `generate_update_files()` が自動生成する。

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

> SSP は Version 3 (charset ヘッダー付き) を認識し、UTF-8 として処理する。

## 除外ルール

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
| `updates.txt` | 自分自身 |
| `updates2.dau` | 更新メタファイル |
| `developer_options.txt` | 開発用オプション |

## 出力例

```
charset,UTF-8
file,ghost/master/descript.txt\x01a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4\x01size=1234\x01date=2026-03-26T12:00:00\x01
file,ghost/master/dic/talk.pasta\x01b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5\x01size=5678\x01date=2026-03-26T12:00:00\x01
file,shell/master/surface0.png\x01d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2\x01size=90123\x01date=2026-03-26T12:00:00\x01
file,install.txt\x01e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3\x01size=456\x01date=2026-03-26T12:00:00\x01
```

## updates2.dau

メタデータファイル。SOH 区切り形式:

```
<path>\x01<md5>\x01size=<bytes>\x01
```

## 実装箇所

- ソース: `crates/pasta_check/src/update_files.rs`
- MD5 計算: `md5` クレート

## SSP 参考仕様

- SSP が `updates.txt` を読み取り、ローカルファイルの MD5 と比較
- 不一致のファイルのみダウンロードして更新
- `profile/` や `var/` を除外することでユーザーデータを保護
