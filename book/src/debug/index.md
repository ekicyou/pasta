# 編集ツールとデバッグの概要

<section class="claudia-hero" aria-label="扉">
<i class="hero-corner hero-corner-tl" aria-hidden="true"></i><i class="hero-corner hero-corner-tr" aria-hidden="true"></i><i class="hero-corner hero-corner-bl" aria-hidden="true"></i><i class="hero-corner hero-corner-br" aria-hidden="true"></i>
<p class="hero-latin">Murasaki &amp; Emo — guest hosts</p>
<div class="hero-faces"><img class="hero-face hero-face-murasaki" src="../img/emo2/m1.png" alt="" width="84" height="84"><img class="hero-face hero-face-emo" src="../img/emo2/e0.png" alt="" width="84" height="84"></div>

> 【むらさき：笑顔】主役交代のお知らせや！ この部は、うちらが案内するで！

> 【エモ】ゴースト「えも？？」のエモだよ。隣のにぎやかなのが、むらさき。二人とも pasta で動いてる、現役のゴーストなんだ。

> 【むらさき：照れ怒り】にぎやかは余計や！ ……ほな Claudia はんには、紅茶でも飲んで休んどいてもらお。道具の話は、鳴らされる側のうちらがいちばん詳しいねん。

</section>

> 【むらさき：笑顔】書いた台詞をその場でしゃべらせてもらえるんやて！ うちら、出番めっちゃ増えるやん！

> 【エモ：静観】▶ を押されるたびに、同じ台詞を言わされるんだけどね。

> 【むらさき】ええねん、何回でも言うたる！ 押すんは ▶ ひとつ、保存も読み込み直しも道具がやってくれるんやで。

---

## 書く → ▶ → しゃべる → 直す

```text
 ① 書く ───▶ ② ▶ を押す ───▶ ③ ゴーストがしゃべる
   ▲                                  │
   └──────────── ④ 直す ◀─────────────┘
```

1. エディタで `.pasta` を書く。
2. シーンの見出しの上の「▶ シーンを実行」を押す。
3. SSP で動いているゴーストが、そのシーンをしゃべる。
4. 台詞を直して、もう一度 ▶ を押す。

- 保存と読み込み直しは、▶ を押すたびにエディタが行う。SSP を再起動しなくてよい。
- 起動のときにしか出ないシーンも、再起動せずに ▶ で鳴らせる。
- ベースウェアは SSP でも areka でもよい。

次の動画は、この流れを pasta-recipe の画面で撮ったものである（約 1 分・音声なし）。

<video controls width="100%" preload="metadata" src="media/recipe-dev-actions-demo.mp4">
  お使いの環境では動画を再生できません。<a href="media/recipe-dev-actions-demo.mp4">recipe-dev-actions-demo.mp4</a> を直接開いてご覧ください。
</video>

## 道具を選ぶ

> 【エモ】道具が 2 つあるけど、どっちを使えばいいの。

> 【むらさき：笑顔】迷ったら pasta-recipe や！ 展開して起動するだけやもん。いつも VS Code 使てる人は、拡張を入れたらええで。

| | [pasta-recipe](./recipe.md) | [VS Code](./vscode-setup.md) |
| -- | -- | -- |
| 入れ方 | zip を展開するだけ | VS Code に拡張を入れる |
| 色分け・書き間違いの診断 | ある | ある |
| ▶ シーンを実行 | ある | ある |
| 変数に値を入れて ▶ | ある | 無い |
| 保存変数 | 表で見て、書き換えられる | 見るだけ |

色分けと診断は、どちらも同じ解析器から出る。同じ辞書なら、同じ結果になる。

## デバッグの有効化

▶ と保存変数の表示は、デバッグを有効にしたゴーストでだけ働く。既定では無効である。

### 道具のトグル

デバッグは、道具の操作で有効にも無効にもできる。

| 道具 | 操作 |
| -- | -- |
| pasta-recipe | ツールバーの「SHIORI をリロード」の隣のボタン |
| VS Code | コマンドパレットの「Pasta: SHIORI デバッグを有効にする」「Pasta: SHIORI デバッグを無効にする」 |

道具は保存変数 `＄＊デバッグ` を書き換えてから読み込み直し、有効になったか無効になったかを出力に出す（[SHIORI デバッグを有効にする・無効にする](./dev-actions.md#shiori-デバッグを有効にする無効にする)）。

### 保存変数による有効化

保存変数 `＄＊デバッグ` か `＄＊debug` に `true` を入れると有効、`false` を入れると無効になる。

- 数えるのは真偽値の `true`・`false` だけである。文字列や数は数えず、`pasta.log` に警告を 1 行出す。
- 両方に真偽値があるときは、`＄＊デバッグ` が効く。
- 入れた値は、次の読み込みから効く。
- `デバッグ`・`debug` という名前の保存変数は、pasta のものとして読む。

値を入れる口は 3 つある。

| 口 | 入れ方 |
| -- | ------ |
| pasta-recipe | [保存変数の表](./recipe.md#保存変数の表) で、型を「真偽」にして書き換える・足す |
| SHIORI イベント | [`OnPastaSetSaveVariable`](../lua/shiori-events.md#onpastasetsavevariableグローバル変数に代入) に、型 `boolean` で送る |
| Lua | `SAVE.デバッグ = true` と書く |

保存変数は、既定ではゴーストの `profile/pasta/save/save.toml` に保存される。`profile/` は配布物に入らない。

一度入れた値は、消すまで次の起動からも効き続ける。`pasta.toml` の設定へ戻すには、pasta-recipe の保存変数の表などで `＄＊デバッグ`・`＄＊debug` を消す。

### pasta.toml による有効化

ゴーストの `ghost/master/pasta.toml` に、次の 2 行を書き足す。

```toml
[debug]
enabled = true
```

書き足した後の最初の ▶（か「SHIORI をリロード」）から効く。

### 優先の順

有効か無効かは、読み込みのたびに次の順で決まる。上の段で決まれば、下の段は見ない。

1. 環境変数 `PASTA_DEBUG`（ベースウェアを起動したときの値）
2. 保存変数 `＄＊デバッグ`・`＄＊debug`
3. `pasta.toml` の `[debug] enabled`
4. 無効

環境変数で有効にするときは、ベースウェアを起動する前に次を入れる。

```text
PASTA_DEBUG=1
```

## この部の章

| 章 | 読むとき |
| -- | -------- |
| [pasta-recipe で書く](./recipe.md) | pasta-recipe を入れて、最初の ▶ を押す |
| [VS Code で書く](./vscode-setup.md) | VS Code に拡張を入れて、最初の ▶ を押す |
| [▶・リロード・保存変数](./dev-actions.md) | 5 つの操作が何をして、何を返すかを知りたい |
| [うまく動かないとき](./troubleshooting.md) | ▶ を押してもしゃべらない |
| [辞書を検査する](./validate.md) | SSP を起動せずに、辞書の書き間違いをまとめて探す |

この部の案内役は、pasta で作られたゴースト「[えも？？](https://ekicyou.github.io/ghost_dev/emo2/)」のむらさきとエモである。顔の絵は、フリーシェル「コンフィズリー」（[ゆゆぴか](https://yusyuparo.net/) さん）と「City-Pop'n」（[大槻](http://th88.blog.shinobi.jp/) さん）から作った。

---

> 【むらさき：笑顔】書いて、鳴らして、直して、また鳴らす。これ回りだしたら止まらへんで！

> 【エモ：ジト】鳴らされる側の喉のことも、たまには考えてほしいな。
