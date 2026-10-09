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
updates.txt
ghost/master/descript.txt
ghost/master/pasta.dll
ghost/master/pasta.toml
ghost/master/updates.txt
ghost/master/dic/01-boot.pasta
ghost/master/dic/02-talk.pasta
ghost/master/scripts/README.md
shell/master/descript.txt
shell/master/surface0.png
shell/master/surfaces.txt
```

Lua ランタイム（`pasta_scripts/`）は `pasta.dll` に埋め込まれているため、nar には入らない。

### 同梱バルーンがあるとき

`install.txt` に `balloon.source.directory,balloon/emo2-kakukaku` がある場合。同梱バルーンのフォルダ直下に生成したバルーン用 `updates.txt` も入る（パスは [updates.txt 仕様](./updates-txt-spec.md) を参照）。

```
install.txt
updates.txt
ghost/master/descript.txt
ghost/master/updates.txt
...
shell/master/descript.txt
...
balloon/emo2-kakukaku/descript.txt
balloon/emo2-kakukaku/arrow0.png
balloon/emo2-kakukaku/updates.txt
```

## インストール動作

1. ユーザーが `.nar` ファイルを SSP にドロップ
2. SSP が ZIP を解凍し、`install.txt` の内容に従ってインストール
3. `ghost/master/` と `shell/master/` がそれぞれ配置される
4. 同梱バルーンのフォルダは `install.txt` のバルーン指定に従ってバルーンとしてインストールされる

## 実装箇所

- ソース: `crates/pasta_check/src/nar.rs`
- ZIP 書き込み: `zip` クレート (v8.6, deflate-only feature)
- `profile/` ディレクトリは再帰走査時にスキップ
- 段 1 で配布フォルダの準備の後に `--nar` の位置にある前回の nar を削除する（削除したときは `Removed previous <パス>` を表示）。段 5 で失敗したときは作りかけの nar を削除する。削除した後のどの段で失敗しても `--nar` の位置に nar は残らない
