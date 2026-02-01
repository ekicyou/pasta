# 伺か ネットワーク更新ファイル仕様書

## 概要

伺か（SSP）のネットワーク更新機能で使用されるファイルリスト形式の仕様。
更新対象ファイルのパス、MD5ハッシュ、ファイルサイズを記録し、クライアント側での整合性検証に使用される。

## ファイル形式

### updates2.dau（バイナリ形式）

SSPが生成・使用するネイティブ形式。

### updates.txt（テキスト形式）

人間が読みやすい代替形式。SSPは両形式をサポートする。

---

## updates2.dau 詳細仕様

### エンコーディング

- 文字コード: **Shift_JIS**（またはCP932）
- 改行コード: **CRLF** (`0x0D 0x0A`)

### レコード構造

各行（レコード）は以下の形式：

```
<filepath><SOH><md5hash><SOH>size=<filesize><SOH><CRLF>
```

### フィールド区切り文字

| 名称 | バイト値 | 説明 |
|------|----------|------|
| SOH (Start of Heading) | `0x01` | フィールド区切り文字 |
| CRLF | `0x0D 0x0A` | レコード（行）区切り |

### フィールド定義

| # | フィールド名 | 形式 | 説明 |
|---|-------------|------|------|
| 1 | filepath | 文字列 | 相対ファイルパス（`/`区切り） |
| 2 | md5hash | 32文字hex（小文字） | ファイルのMD5ハッシュ値 |
| 3 | size | `size=`プレフィクス + 数値 | ファイルサイズ（バイト単位） |

### バイナリ構造例

```
65 6d 6f 2d 6b 61 6b 75 6b 61 6b 75 2f 61 72 72 6f 77 30 2e 70 6e 67  "emo-kakukaku/arrow0.png"
01                                                                      SOH（区切り）
30 36 37 37 64 35 62 30 35 38 39 65 36 64 38 33 33 33 39 37 31 38 38 64 33 30 63 30 35 66 37 63  "0677d5b0589e6d833397188d30c05f7c"
01                                                                      SOH（区切り）
73 69 7a 65 3d 35 35 31                                                "size=551"
01                                                                      SOH（区切り）
0d 0a                                                                   CRLF（改行）
```

### 特殊値

| 用途 | md5hashフィールドの値 | 説明 |
|------|----------------------|------|
| ファイル削除 | `remove` | 次回更新時にクライアント側でファイルを削除 |

---

## updates.txt 詳細仕様

### エンコーディング

- 文字コード: **Shift_JIS** または **UTF-8**
- 改行コード: **CRLF** (`0x0D 0x0A`) または **LF** (`0x0A`)

### レコード構造

各行は以下の形式：

```
file,<filepath><md5hash>size=<filesize>
```

### フィールド定義

| # | フィールド名 | 形式 | 説明 |
|---|-------------|------|------|
| 1 | prefix | 文字列 | 固定値 `file,` |
| 2 | filepath | 文字列 | 相対ファイルパス（`/`区切り） |
| 3 | md5hash | 32文字hex（小文字） | ファイルのMD5ハッシュ値（区切りなしで連結） |
| 4 | size | `size=`プレフィクス + 数値 | ファイルサイズ（バイト単位） |

**注意**: updates.txtでは filepath と md5hash の間に区切り文字がない（直接連結）。

### 例

```
file,emo-kakukaku/arrow0.png0677d5b0589e6d833397188d30c05f7csize=551
file,ghost/master/emo.dlle6ddcfb1db4ecad33341e5e13e0b4153size=949248
```

---

## 共通仕様

### パス表記

- ディレクトリ区切り: **スラッシュ** (`/`)
- 相対パス: ゴースト/シェル/バルーン等のルートディレクトリからの相対パス
- 大文字小文字: 保持される（ただしWindows環境では大文字小文字を区別しない）

### MD5ハッシュ

- 形式: 32文字の16進数文字列
- 大文字小文字: **小文字**
- 計算対象: ファイルのバイナリ内容全体

### ファイルサイズ

- 単位: バイト
- 形式: 10進数整数

---

## ファイル優先順位

SSPは以下の優先順位でファイルを検索する：

1. `updates2.dau` （優先）
2. `updates.txt` （updates2.dauが存在しない場合）

---

## 実装上の注意

### 生成時

1. すべての対象ファイルを列挙
2. 各ファイルのMD5ハッシュを計算
3. 各ファイルのサイズを取得
4. 指定フォーマットでレコードを出力
5. **FTPアップロード時は必ずバイナリモードを使用**（アスキーモードは不可）

### 検証時

1. ファイルリストを読み込み
2. 各ファイルの存在確認
3. MD5ハッシュの再計算と比較
4. サイズの比較（オプション）
5. 不一致があれば更新対象としてマーク

### 除外対象（一般的）

以下のファイル/ディレクトリは通常除外される：

- `profile/` ディレクトリ（ユーザーデータ）
- `var/` ディレクトリ（変数保存）
- `updates2.dau` 自身
- `updates.txt` 自身
- `developer_options.txt`

---

## BNF記法による形式定義

### updates2.dau

```bnf
<file>      ::= <record>*
<record>    ::= <filepath> <SOH> <md5hash> <SOH> "size=" <filesize> <SOH> <CRLF>
<filepath>  ::= <path_char>+
<path_char> ::= <any character except SOH, CR, LF>
<md5hash>   ::= <hexchar>{32} | "remove"
<hexchar>   ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
              | "a" | "b" | "c" | "d" | "e" | "f"
<filesize>  ::= <digit>+
<digit>     ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
<SOH>       ::= 0x01
<CRLF>      ::= 0x0D 0x0A
```

### updates.txt

```bnf
<file>      ::= <record>*
<record>    ::= "file," <filepath> <md5hash> "size=" <filesize> <newline>
<filepath>  ::= <path_char>+
<path_char> ::= <any character except comma, CR, LF, and not starting md5-like sequence>
<md5hash>   ::= <hexchar>{32} | "remove"
<hexchar>   ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
              | "a" | "b" | "c" | "d" | "e" | "f"
<filesize>  ::= <digit>+
<digit>     ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
<newline>   ::= <CRLF> | <LF>
<CRLF>      ::= 0x0D 0x0A
<LF>        ::= 0x0A
```

---

## 参考情報

- UKADOC: http://ssp.shillest.net/ukadoc/manual/dev_update.html
- SSP公式: https://ssp.shillest.net/

---

## 更新履歴

- 2025-02-01: 実ファイル解析に基づく初版作成