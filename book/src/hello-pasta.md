# サンプルゴースト hello-pasta

> 【女の子：笑顔】見習いコンビのご挨拶です！ いらっしゃいませ、hello-pasta の紹介ページへようこそ。本日のおすすめは、わたしです。

> 【男の子：困惑】……それ、ここでも言うんだ。

> 【女の子：キラキラ】だって、看板娘だもん。わたしたちのこと、ちゃんと紹介するね！

---

<section class="claudia-hero hello-hero" aria-label="ダウンロード">
<i class="hero-corner hero-corner-tl" aria-hidden="true"></i><i class="hero-corner hero-corner-tr" aria-hidden="true"></i><i class="hero-corner hero-corner-bl" aria-hidden="true"></i><i class="hero-corner hero-corner-br" aria-hidden="true"></i>

[![赤いエプロンドレスの女の子が、両腕を下ろして笑っている立ち絵](img/hello-pasta/surface0.png)![白いコック服の男の子が、両腕を下ろして笑っている立ち絵](img/hello-pasta/surface10.png)](https://github.com/ekicyou/pasta/releases/latest/download/hello-pasta.nar)

[hello-pasta をダウンロード（.nar）](https://github.com/ekicyou/pasta/releases/latest/download/hello-pasta.nar)

ダウンロードするか、リンクを SSP へ直接ドロップする。ネットワーク更新に対応している。

</section>

## どんなゴースト

hello-pasta は、pasta で動く見本のゴーストである。パスタ屋の見習いの 2 人が、デスクトップでおしゃべりをする。

| 女の子 | 男の子 |
| --- | --- |
| 給仕の見習い。おやつとまかないが大好き | 料理人の見習い。世話焼きで、落ち着いて言い返す |

> 【男の子】ぼくが厨房で、こっちが給仕。……のはずなんだけど、厨房によく来るんだよね。

> 【女の子：照れ】味見だよ、味見。

## 入れる

1. SSP を用意する（[SSP の配布サイト](https://ssp.shillest.net/)）。
2. このページの上のボタンを、SSP に出ているゴーストの上へドロップする。ダウンロードした `hello-pasta.nar` をドロップしてもよい。
3. ゴーストを右クリックし、メニューの「ゴースト切り替え」から hello-pasta を選ぶ。

- 成功の目印: 2 人が出てきて、吹き出しであいさつする。
- 過去の版は、[ダウンロード](download.md#過去の版) のページから辿れる。

> 【女の子：キラキラ】ご来店、お待ちしてます！ 置いておくだけで、勝手におしゃべりするよ。

> 【男の子：困惑】勝手に、は言い方。

## できること

| できること | 台詞の例 | 書き方が分かる章 |
| ---------- | -------- | ---------------- |
| ひとりでにおしゃべりする | 「ねえ、まかない、まだ？」「さっき食べたよね。ぼくの分まで。」 | [2 段目](getting-started/02-talk.md) |
| 表情を変える | 「えっ、つまみ食い、見てたの？」「味見で、お皿は空にならないよ。」 | [3 段目](getting-started/03-face.md) |
| 毎正時に時刻を知らせる | 「ラストオーダーのお時間です！」「……毎時間、ラストオーダーなの？」 | [6 段目](getting-started/06-hour.md) |
| つつく・撫でると反応する | 「呼び鈴じゃないよ、わたしの頭だよ。」「いい音がしたね。」 | [8 段目](getting-started/08-touch.md) |
| 選択肢を出す | 「いらっしゃいませ。ご注文は、お決まりですか？」 | [9 段目](getting-started/09-choice.md) |
| 終了しても覚えている | 「つまみ食い、通算2回目です！」「……その記録、伸ばさなくていいよ。」 | [10 段目](getting-started/10-save.md) |
| 季節の行事で話が変わる | 「年越しパスタ、できたよ。」「……そばじゃなくて？」 | [13 段目](getting-started/13-calendar.md) |

> 【男の子：笑顔】どれも、辞書っていうテキストファイルに書いてあるんだ。書き方は、入門ガイドが 1 つずつ教えてくれるよ。

## 台詞を書き換えて遊ぶ

hello-pasta の台詞は、エディタで書き換えて、その場で鳴らせる。手順は [まず会ってみる](getting-started/hello.md) にある。

> 【男の子】厨房は見学自由だよ。1 行書き換えて ▶ を押すと、そのとおりにしゃべる。

> 【女の子：驚き】えっ、わたしの台詞、書き換えられちゃうの？

> 【女の子：照れ】……へんなこと、言わせないでね。

## 更新

- hello-pasta は、SSP のネットワーク更新で新しい版になる。
- 手で書き換えた辞書は、更新で元に戻ることがある。
- 自分のゴーストは、[自分のゴーストを始める](getting-started/setup.md) で別に作る。

## 作った人とライセンス

| 項目 | 中身 |
| ---- | ---- |
| 作者 | ekicyou（どっとステーション駅長） |
| 頭脳（SHIORI） | `pasta.dll`（[pasta](https://github.com/ekicyou/pasta)）。同梱のソフトウェアのライセンスは、`ghost/master/THIRD_PARTY_LICENSES.txt` にある |
| 辞書と設定 | pasta のリポジトリと同じ MIT License |
| 絵 | 画像生成 AI の出力から作った。Unlicense（[見本の絵のライセンス](reference/shell.md#見本の絵のライセンス)） |

---

> 【女の子：笑顔】またのご来店を、お待ちしております！

> 【男の子】……来てもらうのは、きみのデスクトップなんだけどね。
