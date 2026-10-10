# ゴーストの最小一式を置く

> 【クローディア】骨組みを組み上げる番ですわ。あなたのゴーストが住まう器を、これからこしらえますのよ。

> 【アンソニー】お嬢様、器だけでは、まだ口はきけないのでは。

> 【したり顔】ええ、それでよろしいの。黙って立つところまでが、この章の仕上がりですわ。

---

## ゴーストのフォルダ構成

> 【クローディア】ゴーストは、決まった形のフォルダに住んでいますの。会話を司る「中身」と、画面に映る「見た目」が、別々の部屋に分かれていますわ。

```text
my-ghost/
├── install.txt                       インストール情報
├── ghost/
│   └── master/
│       ├── descript.txt              ゴーストの定義
│       ├── pasta.toml                pasta の動作設定
│       ├── pasta.dll                 SHIORI の本体
│       ├── THIRD_PARTY_LICENSES.txt  pasta.dll のライセンス表示
│       └── dic/                      辞書を置く場所（まだ空）
└── shell/
    └── master/                       見た目（シェル）の一式
```

| 置くもの | 役割 | 用意のしかた |
| -------- | ---- | ------------ |
| `install.txt` | このフォルダがゴーストであることと、インストール先を示す | この章で書く |
| `ghost/master/descript.txt` | ゴーストの定義 | この章で書く |
| `ghost/master/pasta.toml` | pasta の動作設定 | この章で書く |
| `ghost/master/pasta.dll` | SHIORI の本体 | hello-pasta から写す |
| `ghost/master/THIRD_PARTY_LICENSES.txt` | `pasta.dll` が含むソフトウェアのライセンス表示 | hello-pasta から写す |
| `shell/master/` | 画面に表示される見た目の一式 | hello-pasta からフォルダごと写す |

- 上の表の 6 つが、1 段目に入る前に置く最小一式である
- `my-ghost` は例の名前である。フォルダの名前は、次の節で自分で決める
- `ghost/master/` — 会話と振る舞いを司る「中身」を置く
- `shell/master/` — 画面に表示される「見た目」を置く
- `ghost/master/dic/` — 辞書（`.pasta` ファイル）を置く場所。この章ではフォルダだけを作り、中は空のままにする
- マスター（master） — ゴーストの標準的な中身・見た目を置く既定のフォルダ名。このガイドでは `master` の 1 つだけを扱う
- シェル — ゴーストの見た目（画像一式）のこと。会話の中身（辞書）とは分かれて管理される

> 【アンソニー】お嬢様、`dic/` が空のままでございますが。

> 【にっこり】それでよろしいのよ。辞書は、次の章から 1 枚ずつ足していきますわ。

## 名前を決める

> 【考え中】さて、最初のお仕事は名付けですわ。あなたのゴーストに、あなただけの名前をお付けなさいまし。

> 【アンソニー】見本と同じ `hello-pasta` では、いけないのでございますか。

> 【不安】それは困りますの。このあと見本の hello-pasta を同じ SSP に入れますから、同じ名前では衝突してしまいますわ。

- ゴーストの名前は、自分で決める
- このガイドは、例の名前として `my-ghost` を使う。この先の章に出てくる `my-ghost` は、自分の決めた名前に読み替える
- `hello-pasta` という名前は使わない。配布版の hello-pasta と同じ SSP に入れると、名前が衝突する
- 作者の名前（`craftman`・`craftmanw`）も、配布版の値を写さずに自分のものにする

| 名前を書く場所 | 例 |
| -------------- | -- |
| ゴーストのフォルダの名前 | `my-ghost` |
| `install.txt` の `name` | `my-ghost` |
| `install.txt` の `directory` | `my-ghost` |
| `ghost/master/descript.txt` の `name` | `my-ghost` |

決めた名前のフォルダを作り、その中に `ghost/master/dic/` と `shell/` のフォルダを作る。

## install.txt を書く

> 【クローディア】まずは表札ですわ。`install.txt` に、ゴーストの名前と、住むフォルダの名前を書きますのよ。

ゴーストのフォルダの直下に `install.txt` を作り、次の内容を UTF-8 で保存する。

```text
charset,UTF-8
type,ghost
name,my-ghost
directory,my-ghost
accept,
```

- `name` — ゴーストの名前。`my-ghost` を自分の決めた名前に置き換える
- `directory` — インストール先のフォルダの名前。ゴーストのフォルダの名前と同じにする
- ほかの行は、そのまま写す

## descript.txt を書く

> 【したり顔】次は名簿ですわ。ゴーストの名前と作者の名前、それから頭脳に `pasta.dll` を使うことを、`descript.txt` に書きますの。

`ghost/master/descript.txt` を作り、次の内容を UTF-8 で保存する。

```text
charset,UTF-8
type,ghost
name,my-ghost
sakura.name,女の子
kero.name,男の子
craftman,your-name
craftmanw,あなたの名前
shiori,pasta.dll
```

- `name` — ゴーストの名前。`my-ghost` を自分の決めた名前に置き換える
- `craftman`・`craftmanw` — 作者の名前。`your-name`・`あなたの名前` を自分のものに置き換える
- `sakura.name`・`kero.name` — 2 人のキャラクターの表示名である
- 辞書の中で呼ぶアクター名は、表示名ではなく `pasta.toml` の `[actor]` で決まる。表示名を変えても、辞書の書き方には影響しない
- `shiori,pasta.dll` — ゴーストの頭脳（SHIORI）として `pasta.dll` を使うという宣言。この行は変えない

> 【アンソニー】`sakura.name` の `女の子` が、辞書の中で呼ぶ名前になるのでございますね。

> 【冷笑】惜しいですわね。それは表示名ですの。辞書の中で呼ぶ名前は、次に書く `pasta.toml` が決めますのよ。

## pasta.toml を書く

> 【クローディア】最後に書くのは `pasta.toml` ですわ。pasta の暮らし方を決める設定ですけれど、起動に欠かせないのは `[actor]` だけですのよ。

`ghost/master/pasta.toml` を作り、次の内容を UTF-8 で保存する。

```toml
[actor."女の子"]
spot = 0

[actor."男の子"]
spot = 1
```

- 起動に必須なのは `[actor]` だけである。ほかのセクションは省略でき、省略すると既定値が使われる
- `pasta.toml` のファイルそのものは必須である。無いと、ゴーストの読み込みは失敗する
- `[actor."女の子"]` の `女の子` がアクター名である。辞書の中では、この名前でアクターを呼ぶ
- アクター名は `女の子`・`男の子` のままにする。この先の章の辞書が、この名前で書かれている
- `spot` — そのアクターの台詞を出すバルーン（吹き出し）を決める。`0` がメイン（sakura 側）、`1` がサブ（kero 側）である
- `spot` には既定値が無い。アクターごとに必ず書く
- 全セクション・全キーの説明は、[pasta.toml リファレンス](../reference/pasta-toml.md) にある

> 【考え中】`spot` は、その子の台詞をどちらの吹き出しに出すかの番号ですわ。`0` がメインの側、`1` がサブの側ですの。

## hello-pasta から写す

> 【クローディア】残りの 3 つは、書くものではなく写すものですわ。完成した見本の hello-pasta を SSP にお招きして、そこから拝借いたしますの。

1. [リリースページ](https://github.com/ekicyou/pasta/releases) から `hello-pasta.nar` をダウンロードする。
2. `hello-pasta.nar` を SSP のウィンドウへドロップして、SSP に入れる。
3. SSP をインストールしたフォルダの中の `ghost/hello-pasta/` を開く。
4. 次の表の 3 つを、自分のゴーストの同じ場所へ写す。

| 写すもの（`ghost/hello-pasta/` の中） | 写す先（自分のゴーストのフォルダの中） |
| ------------------------------------- | -------------------------------------- |
| `ghost/master/pasta.dll` | `ghost/master/pasta.dll` |
| `ghost/master/THIRD_PARTY_LICENSES.txt` | `ghost/master/THIRD_PARTY_LICENSES.txt` |
| `shell/master/`（フォルダごと） | `shell/master/` |

- `THIRD_PARTY_LICENSES.txt` は、`pasta.dll` が含むソフトウェアのライセンス表示である。`pasta.dll` と同じ場所に置き、ゴーストを配布するときも一緒に配る
- シェル（`shell/master/`）は、hello-pasta のものをそのまま使う。中のファイルには手を入れない
- `scripts/` は写さない。自分のゴーストに `scripts/` を作る必要もない（自分で Lua スクリプトを書くときに使う場所である。[前提環境と準備](prerequisites.md)）
- SSP に入れた hello-pasta は、完成版の見本としてそのまま残す

> 【アンソニー】お嬢様、ライセンスの文書まで写すのでございますか。

> 【目閉じ】ええ、大切な礼儀ですわ。`pasta.dll` は、多くのソフトウェアの力を借りていますの。その表示は、いつも `pasta.dll` の隣に置くものですのよ。

## SSP に入れて起動する

> 【にっこり】一式がそろいましたわね。では、あなたのゴーストを SSP に入れて、立たせてみましょう。

1. 自分のゴーストのフォルダを、フォルダごと、SSP をインストールしたフォルダの中の `ghost/` の下へ置く（`ghost/hello-pasta/` の隣に並ぶ）。
2. SSP を終了し、もう一度起動する。
3. メニューから、自分のゴーストに切り替える。

- 成功の目印: 立ち絵が出て、何もしゃべらない
- 辞書がまだ 1 枚も無いので、ゴーストはしゃべらない。これが準備の終わりの状態である
- 次の章 [1 段目：しゃべらせたい](01-boot.md) で、最初の辞書を 1 枚足す

> 【アンソニー】お嬢様、立ってはおりますが、うんともすんとも申しません。

> 【したり顔】それで正解ですわ。辞書が 1 枚も無いのですもの、しゃべる台詞がまだありませんのよ。

## うまく起動しないときは

> 【不安】思ったとおりに立ってくれないときは、慌てずに次の 3 つを確かめなさいまし。

- **文字化けする・辞書が読めない** — ファイルが UTF-8 で保存されているかを確かめる。`install.txt` と `descript.txt` の先頭に `charset,UTF-8` があるかも確かめる
- **何も表示されない** — `ghost/master/` に `pasta.dll` が置かれているか、`ghost/master/descript.txt` に `shiori,pasta.dll` の行があるかを確かめる
- **辞書が反映されない**（1 段目から先で、辞書を足したのにしゃべらないとき） — `ghost/master/dic/` の中に `.pasta` ファイルがあるかを確かめる。`pasta.toml` に `[loader]` を書いていなければ、`dic/` の下の `.pasta` ファイルはすべて読み込まれる（[pasta_patterns](../reference/pasta-toml.md#pasta_patterns) の既定値 `["dic/**/*.pasta"]`）

---

> 【にっこり】器はできあがりましたわね。黙って立っているだけでも、もうあなたのゴーストですのよ。

> 【アンソニー】次の章で、いよいよ最初のひとことでございますね。

> 【高笑い】おほほほ！ ええ、産声を聞きに参りますわよ！
