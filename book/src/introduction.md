# pasta マニュアル

<section class="claudia-hero" aria-label="扉">
<i class="hero-corner hero-corner-tl" aria-hidden="true"></i><i class="hero-corner hero-corner-tr" aria-hidden="true"></i><i class="hero-corner hero-corner-bl" aria-hidden="true"></i><i class="hero-corner hero-corner-br" aria-hidden="true"></i>
<p class="hero-latin">Claudia et Anthony — à votre service</p>
<div class="hero-faces"><img class="hero-face" src="img/claudia/f25.png" alt="" width="84" height="84"><img class="hero-face hero-face-anthony" src="img/claudia/f10.png" alt="" width="84" height="84"></div>

> 【お辞儀】pasta のマニュアルへ、ようこそいらっしゃいまし。わたくし Claudia が、このマニュアルの案内役を務めますわ。

> 【アンソニー】お嬢様にお仕えしております、執事のアンソニーでございます。失礼ながらお嬢様、ゴーストづくりが初めての方でも、読み通せるものでございましょうか。

> 【したり顔】ふふん、心配なさらなくてよろしくてよ。手順から文法、Lua の書き方まで、わたくしが最後まできっちりお付き合いいたしますもの。さあ、肩の力を抜いて、最初の一歩を踏み出しましょう。

<nav class="hero-toc">

- [ダウンロード <small>Download</small>](download.md)
- [入門／チュートリアル <small>Getting started</small>](getting-started/index.md)
- [Pasta DSL 文法 <small>Grammar</small>](grammar/index.md)
- [Lua API／コーディング <small>Lua API</small>](lua/index.md)
- [デバッグ <small>Debugging</small>](debug/index.md)
- [リファレンス <small>Reference</small>](reference/startup.md)
- [内部設計 <small>Internals</small>](internals/index.md)
- [雑記 <small>Notes</small>](notes/cc-sdd.md)

</nav>

</section>

---

**pasta** は、「伺か」のようなデスクトップマスコットを動かすための対話スクリプトエンジンである。
作者が書いた Pasta DSL の辞書を Lua にトランスパイルし、「ゴースト」の頭脳として動作させる。
本マニュアルは、ゴースト作者が pasta で辞書を書けるようになることを目的とした利用者向けガイドである。

Pasta DSL の文法・公開 Lua API・`pasta.toml` の設定について、利用者向けの権威的な記述は本マニュアルである。ほかの文書と記述が食い違う場合は、本マニュアルを正とする。

まず[ダウンロード](download.md)、それから[入門ガイド](getting-started/index.md)へ。

## 関連リンク

| リンク | 内容 |
| ------ | ---- |
| [GitHub リポジトリ（ekicyou/pasta）](https://github.com/ekicyou/pasta) | ソースコード・ビルド方法などの開発者向け情報。[README](https://github.com/ekicyou/pasta/blob/main/README.md) も参照 |

## このマニュアルが対象とするバージョン

| 項目 | 内容 |
| ---- | ---- |
| 対象 pasta バージョン | **v0.6.0** |
| Lua 方言（ランタイム） | **LuaJIT 2.1**（Lua 5.1 系＋有効化された拡張） |
| 対象プラットフォーム | Windows |

本マニュアルは、上表の版の pasta を対象として記述する。対象バージョンはリリースのたびに更新される。
これより古い版の pasta を使っている場合は、[ダウンロード](download.md)のページから
新しい版を入手すること。

ランタイムの Lua 方言は **LuaJIT 2.1** である。これは **Lua 5.1 系**の言語仕様を基礎とし、
LuaJIT 独自の拡張（`goto`/ラベルや一部の 5.2 互換機能など）が有効化されたものである。
そのため、Lua のコードを書く際は **Lua 5.1 系＋ LuaJIT 拡張**の文法・標準ライブラリを前提とすること。
バージョンの離れた **Lua 5.5 等の他バージョンの仕様と混同しない**よう注意する。外部 Lua リファレンスを
参照するときも、LuaJIT 2.1 / Lua 5.1 系に対応した資料を用いること（詳細は[外部リンク集](reference/external-links.md)）。

## 安定機能と「将来変更あり」の区別について

本マニュアルは、現行の版で実装されている文法・API だけを記述する。まだ実装されていない機能や
実装予定の機能は記述しない。一方で pasta は開発が継続しており、現行の挙動のなかには将来の版で
変わり得る箇所もある。そうした箇所は本文中で次のように明示し、安定機能と視覚的・記述的に区別する。

> **⚠ 将来変更あり**
> このように引用ブロックと「将来変更あり」の見出しを付した箇所は、現行の版で記述どおりに動作するが、
> 将来の版で挙動が変わる可能性がある。安定版として依存する前に、最新の情報を確認すること。

注記のない通常の本文は、現行の版で安定して利用できる機能として記述している。

---

> 【クローディア】準備はよろしくて？ どこから読み始めても構いませんけれど、ゴーストづくりが初めてなら、まずは入門ガイドから順にお読みなさいな。

> 【アンソニー】お嬢様は、皆さまが迷わぬようにと、昨晩から目次を何度も見直しておいででした。

> 【照れ怒り】な、なにを言っていますの！ フンッ、別に、あなたのためを思って言っているわけではありませんわよ！ ……さ、さあ、熱く参りましょう！

<p class="claudia-credit">顔アイコン・意匠: <a href="https://github.com/ponapalt/claudia">ponapalt/claudia</a>（Unlicense）／ 参考: <a href="https://ponadocs.shillest.net/claudia/">ponadocs.shillest.net/claudia</a></p>
