# updates.txt 仕様

SSP（伺かベースウェア）のネットワーク更新ファイル仕様。
pasta_check の `generate_update_files()` が自動生成する。

## フォーマット

| 項目 | 値 |
|------|-----|
| エンコーディング | Shift_JIS |
| 改行コード | CRLF (`\r\n`) |
| 行フォーマット | `relative/path/to/file,md5hash` |
| MD5 | 32文字小文字16進数 |
| パス区切り | スラッシュ (`/`) |

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
ghost/master/descript.txt,a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4
ghost/master/dic/talk.pasta,b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5
ghost/master/pasta.toml,c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1
shell/master/surface0.png,d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2
install.txt,e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3
```

## updates2.dau

将来用のメタデータファイル。現在は空ファイルとして生成される。

## 実装箇所

- ソース: `crates/pasta_check/src/update_files.rs`
- エンコーディング変換: `encoding_rs` クレート (UTF-8 → Shift_JIS)
- MD5 計算: `md5` クレート

## SSP 参考仕様

- SSP が `updates.txt` を読み取り、ローカルファイルの MD5 と比較
- 不一致のファイルのみダウンロードして更新
- `profile/` や `var/` を除外することでユーザーデータを保護
