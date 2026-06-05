# 文法リファレンス概要

ごきげんよう。わたくし Claudia が、Pasta DSL の文法を隅から隅までご案内いたしますわ。
このセクションは読み物ではなく、辞書のように引いて使う**参照型リファレンス**ですの。
「あの構文、どう書くんだったかしら」と迷ったとき、該当の章をめくればよろしくてよ。
……フンッ、別にあなたが忘れっぽいなんて言っていませんわよ。

---

このセクションは、Pasta DSL の**実装済み文法を網羅する参照型ドキュメント**である。手を動かして覚える入門は「入門ガイド」に譲り、ここでは各文法要素を要素ごとに整理し、試せる具体例と権威的仕様への導線を示す。

## Pasta DSL の文法モデル

Pasta DSL は、里々／さとりにインスパイアされた対話スクリプト記述言語であり、次の基本原則に立つ。

- **行指向文法**: 各行は改行で終わり、行頭の数文字（マーカー）によって行の種類が確定する。唯一の例外は複数行にわたる Lua コードブロックである。
- **全角・半角の両対応**: マーカー・演算子・括弧は全角と半角のいずれでも記述でき、両者は同等に扱われる（例: `＊` と `*`、`（` と `(`）。
- **インデントは有無のみで判定**: 行頭に空白があるか否かでグローバルレベル／下層レベルを区別する。インデントの深さは判定に使われない。
- **式（Expression）のサポート**: 変数代入や関数引数で算術式を記述できる。複雑な処理は Lua ブロックに委ねられる。
- **宣言的言語**: `if` / `while` のような命令型制御構文は持たない。制御フローはシーン定義と Call で表現する。

### ファイル構造の俯瞰

```text
ファイル
├─ グローバル単語定義 (＠)
├─ アクター辞書 (％)
├─ グローバルシーン (＊)
│   ├─ 属性行 (＆)
│   ├─ ローカル単語定義 (＠)
│   ├─ アクション行 (アクター：内容)
│   ├─ 変数代入 (＄)
│   ├─ Call 行 (＞)
│   ├─ 選択肢行 (＠？)
│   ├─ キューコマンド行 (！)
│   ├─ ローカルシーン (・)
│   └─ Lua コードブロック
└─ コメント行 (＃)
```

### 式の基本例

```pasta
＄count＝10 + 5            # 算術式
＄result＝＄a * ＄b         # 変数を含む式
＄nested＝（＄a + ＄b）* 2  # 括弧による優先順位制御
＄＝＠副作用関数（）         # 式文: 結果を代入せず式を評価のみ
```

対応する算術演算子は次のとおり（全角・半角は同等）。

| 種別 | 演算子（全角／半角） |
| ---- | -------------------- |
| 加算 | `+` / `＋`           |
| 減算 | `-` / `－`           |
| 乗算 | `*` / `＊` / `×`     |
| 除算 | `/` / `／` / `÷`     |
| 剰余 | `%` / `％`           |

## このセクションの読み方

各章は「軽い導入 → 普通文体の本体（試せる例つき）→ ひとことの締め」のリズムで構成され、章末には対応する `doc/spec/` の権威的仕様へのリンクを置いている。仕様の厳密な定義が必要になったら、章末リンクをたどること。

| 章 | 扱う内容 | 権威的仕様 |
| -- | -------- | ---------- |
| [キーワード・マーカー](markers.md) | 全マーカーと演算子・区切り文字の一覧 | [doc/spec/02-markers.md](https://github.com/ekicyou/pasta/blob/main/doc/spec/02-markers.md) |
| [行とブロック構造](block-structure.md) | 行種別・グローバル／ローカルブロック・インデント | [doc/spec/03-block-structure.md](https://github.com/ekicyou/pasta/blob/main/doc/spec/03-block-structure.md) |
| [Call / Jump](call-jump.md) | シーン呼び出し・前方一致・スコープ解決 | [doc/spec/04-call-spec.md](https://github.com/ekicyou/pasta/blob/main/doc/spec/04-call-spec.md) |
| [リテラル型](literals.md) | 型変換ルール・文字列・数値・真偽値 | [doc/spec/05-literals.md](https://github.com/ekicyou/pasta/blob/main/doc/spec/05-literals.md) |
| [アクション行](action-line.md) | 発言行・インライン要素・行継続・改行 | [doc/spec/06-action-line.md](https://github.com/ekicyou/pasta/blob/main/doc/spec/06-action-line.md) |
| [さくらスクリプト](sakura-script.md) | `\` で始まるコマンドの字句構造と透過処理 | [doc/spec/07-sakura-script.md](https://github.com/ekicyou/pasta/blob/main/doc/spec/07-sakura-script.md) |
| [変数・スコープ](variables.md) | ローカル／グローバル／プロパティ変数 | [doc/spec/09-variables.md](https://github.com/ekicyou/pasta/blob/main/doc/spec/09-variables.md) |
| [単語定義](words.md) | 単語の定義・参照・前方一致・複数キー | [doc/spec/10-words.md](https://github.com/ekicyou/pasta/blob/main/doc/spec/10-words.md) |
| [アクター辞書](actor-dictionary.md) | `％` によるアクター単位の単語辞書 | [doc/spec/11-actor-dictionary.md](https://github.com/ekicyou/pasta/blob/main/doc/spec/11-actor-dictionary.md) |

## 網羅範囲についての注記

このリファレンスは**実装済みの文法要素のみ**を扱う。次の要素は意図的に除外、または注記つきで扱う。

- **属性（`＆`、doc/spec ch08）**: 構文はパーサーで受理されるが、トランスパイラ・ランタイムでの処理は将来予定である。本セクションでは「行とブロック構造」の中で構文と配置ルールのみ触れ、「将来変更あり」の注記を添える。
- **将来仕様（doc/spec ch12）**: 未確定事項は本セクションの対象外とする。

---

さあ、準備はよろしくて？ お目当ての章へお進みなさいまし。
迷ったらこの概要に戻ってくればよろしくてよ。熱く参りましょう！

> **権威的仕様**: 文法モデル全体の厳密な定義は [doc/spec/01-grammar-model.md](https://github.com/ekicyou/pasta/blob/main/doc/spec/01-grammar-model.md) を参照。各文法要素の権威は、章別に分割された [doc/spec/](https://github.com/ekicyou/pasta/tree/main/doc/spec) 各章が担う。
