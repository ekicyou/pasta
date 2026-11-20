# 付録：フォルダの中身と文字コード

> 【目閉じ】困ったときの引き出しですわ。ゴーストのフォルダに何が入っていて、どれが何の役目か、ここにしまってありますの。

> 【アンソニー】頭から通して読む章ではない、ということでございますね。

> 【にっこり】ええ。つまずいたときに、要る段だけ開けばよろしいのよ。

---

## 動作環境

- OS は Windows 11、ゴーストを動かすソフトは SSP である。
- SSP は、[SSP の配布サイト](https://ssp.shillest.net/) から入れる。

## ゴーストのフォルダ構成

```text
my-ghost/
├── install.txt
├── ghost/
│   └── master/
│       ├── descript.txt
│       ├── pasta.toml
│       ├── pasta.dll
│       ├── THIRD_PARTY_LICENSES.txt
│       └── dic/
│           └── 01-boot.pasta
└── shell/
    └── master/
```

| 場所 | 役目 |
| ---- | ---- |
| `install.txt` | このフォルダがゴーストであることと、入れる先のフォルダの名前 |
| `ghost/master/` | 会話と振る舞いを司る「中身」を置く |
| `ghost/master/descript.txt` | ゴーストの名前と、作者の名前 |
| `ghost/master/pasta.toml` | pasta の設定。このファイルが無いと、ゴーストは読み込めない |
| `ghost/master/pasta.dll` | ゴーストの頭脳（SHIORI）。辞書を読んで台詞を返す |
| `ghost/master/THIRD_PARTY_LICENSES.txt` | `pasta.dll` が含むソフトウェアのライセンス表示。`pasta.dll` の隣に置き、配るときも一緒に配る |
| `ghost/master/dic/` | 辞書（`.pasta`）を置く。この下の `.pasta` は全部読み込まれる |
| `shell/master/` | 画面に出る「見た目」の一式を置く |

- `ghost/master/profile/` は、pasta が動くと自分で作る。ログ（`profile/pasta/logs/pasta.log`）とセーブの置き場である。
- `ghost/master/calendar.csv` は、[13 段目](13-calendar.md) で足す。
- `ghost/master/scripts/` は、自分で書いた Lua のスクリプトを置く場所で、このガイドでは作らない。

## 設定ファイルの中身

### install.txt

```text
charset,UTF-8
type,ghost
name,my-ghost
directory,my-ghost
accept,
```

- `name` — ゴーストの名前。
- `directory` — 入れる先のフォルダの名前。ゴーストのフォルダの名前と同じにする。

### descript.txt

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

- `name` — ゴーストの名前。
- `sakura.name`・`kero.name` — 2 人の表示名。辞書の中で呼ぶ名前は、`pasta.toml` の `[actor]` が決める。
- `craftman`・`craftmanw` — 作者の名前。
- `shiori,pasta.dll` — 頭脳に `pasta.dll` を使うという宣言。この行は変えない。

### pasta.toml

```toml
[actor."女の子"]
spot = 0

[actor."男の子"]
spot = 1

[debug]
# エディター（pasta-recipe・VS Code）の ▶ でシーンを鳴らせるようにする。自分のゴーストを配布する前に false にする
enabled = true
```

- `[actor."女の子"]` の `女の子` がアクター名である。辞書の中では、この名前でアクターを呼ぶ。
- `spot` — そのアクターの台詞を出す吹き出し。`0` が本体側、`1` が相方側で、アクターごとに必ず書く。
- `[debug]` の `enabled = true` — エディタの「▶ シーンを実行」を働かせる。
- ほかの設定は、書かなければ既定値で動く。全部のキーは [pasta.toml リファレンス](../reference/pasta-toml.md) にある。

## 名前を付け替える

SSP を終了してから、次の 4 か所を自分の決めた名前に書き換える。

| 名前を書く場所 | 今の値 |
| -------------- | ------ |
| ゴーストのフォルダの名前 | `my-ghost` |
| `install.txt` の `name` | `my-ghost` |
| `install.txt` の `directory` | `my-ghost` |
| `ghost/master/descript.txt` の `name` | `my-ghost` |

- `descript.txt` の `craftman`・`craftmanw` も、自分の名前に書き換える。
- `hello-pasta` という名前は使わない。見本の hello-pasta と同じ SSP に入れると、名前がぶつかる。

## シェルの中身

| ファイル | 役割 |
| -------- | ---- |
| `descript.txt` | シェルの定義（名前・作者・吹き出しの位置など） |
| `surfaces.txt` | サーフェス番号と画像の対応 |
| `surface0.png`〜`surface8.png` | 女の子の立ち絵（表情 9 種） |
| `surface10.png`〜`surface18.png` | 男の子の立ち絵（表情 9 種） |

| 女の子の立ち絵（`surface0.png`・笑顔） | 男の子の立ち絵（`surface10.png`・笑顔） |
| --- | --- |
| ![赤いエプロンドレスの女の子が、両腕を下ろして笑っている立ち絵](../img/hello-pasta/surface0.png) | ![白いコック服の男の子が、両腕を下ろして笑っている立ち絵](../img/hello-pasta/surface10.png) |

- サーフェス — シェルが持つ、表情ごとの絵。番号で指す。
- 辞書とシェルの接点は、サーフェス番号だけである。番号に表情の名前を付けるのは、[3 段目](03-face.md) で行う。
- 吹き出し（バルーン）は入っていない。SSP で選ばれているバルーンが使われる。
- 絵の差し替え・表情の追加・当たり判定は、[シェルを自分の絵にする](../reference/shell.md) にある。

## 文字コードは UTF-8

- 辞書（`.pasta`）と設定ファイルは、UTF-8 で保存する。
- pasta-recipe と VS Code で書けば、UTF-8 で保存される。
- `install.txt` と `descript.txt` は、先頭の行に `charset,UTF-8` を書く。
- 辞書の先頭の BOM は読み飛ばされるので、「UTF-8（BOM 付き）」で保存してもよい。
- 文字化けする・辞書が読めないときは、まず保存の文字コードを確かめる。
- 辞書の文字コードの規則は、[文字コード](../grammar/block-structure.md#文字コード) にある。

### Shift_JIS の辞書から移すとき

- 里々や YAYA で書いた辞書は、Shift_JIS で保存されていることが多い。
- Shift_JIS のまま `.pasta` にすると、文字化けするか、辞書が読み込めない。
- エディタの「文字コードを指定して開く」で Shift_JIS として開き、UTF-8 を指定して保存し直す。

> 【アンソニー】お嬢様、昔書きためた辞書も、無駄にはならないのでございますね。

> 【クローディア】ええ。保存し直すひと手間だけで、あの頃の台詞がまたしゃべり出しますわ。

---

> 【クローディア】引き出しの中身は、これで全部ですわ。用が済んだら、元の段へお戻りなさいまし。

> 【アンソニー】またつまずかれましたら、いつでもお開けくださいませ。
