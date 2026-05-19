# NAR パッケージ仕様

NAR (Nanika ARchive) は伺か（Ukagaka）のゴースト配布に使われる ZIP アーカイブ形式。
pasta_check の `create_nar()` が自動生成する。

## フォーマット

| 項目 | 値 |
|------|-----|
| ベース形式 | ZIP |
| 圧縮方式 | Deflate |
| 拡張子 | `.nar` |
| パス区切り | スラッシュ (`/`) |

## 除外ルール

以下はアーカイブに含めない:

| 対象 | 理由 |
|------|------|
| `profile/` ディレクトリ | ユーザー固有データ（配布に含めてはならない） |

## NAR 内部構造の例

```
install.txt
ghost/master/descript.txt
ghost/master/pasta.toml
ghost/master/pasta.dll
ghost/master/dic/boot.pasta
ghost/master/dic/talk.pasta
ghost/master/pasta_scripts/main.lua
ghost/master/pasta_scripts/pasta/init.lua
ghost/master/scripts/README.md
shell/master/descript.txt
shell/master/surface0.png
shell/master/surfaces.txt
updates.txt
```

## インストール動作

1. ユーザーが `.nar` ファイルを SSP にドロップ
2. SSP が ZIP を解凍し、`install.txt` の内容に従ってインストール
3. `ghost/master/` と `shell/master/` がそれぞれ配置される

## 実装箇所

- ソース: `crates/pasta_check/src/nar.rs`
- ZIP 書き込み: `zip` クレート (v8.4, deflate-only feature)
- `profile/` ディレクトリは再帰走査時にスキップ
