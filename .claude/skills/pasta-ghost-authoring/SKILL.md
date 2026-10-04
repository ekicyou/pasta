---
name: pasta-ghost-authoring
description: >-
  Pasta DSL文法リファレンスと辞書制作パターン集。ゴースト（「伺か」デスクトップマスコット）の
  辞書ファイル（.pasta）を作成・編集する際に、自然言語の指示からPasta DSLコードへの変換を
  サポートする。
  USE FOR: pasta, Pasta DSL, .pasta, ゴースト, 辞書, トーク作成, シーン作成,
  アクション行, 単語定義, 変数, イベントハンドラ, ランダムトーク, アクター辞書,
  さくらスクリプト, 伺か, ukagaka, ghost authoring, dictionary file,
  talk creation, scene definition, pasta script, pasta code generation,
  時報, OnHour, 時報変数, 日時変数, date variables, hour variables.
  DO NOT USE FOR: pasta料理, cooking pasta, Pasta DSLパーサー開発,
  pasta_dsl crate, pasta_lua crate, pasta_core crate, Rustクレート実装,
  SHIORIプロトコル実装, Luaランタイム開発, pasta言語仕様の設計変更.
metadata:
  author: ekicyou
  version: "1.7.1"
---

# Pasta Ghost Authoring Skill

## §1 Purpose & Prerequisites

**目的**: 自然言語の指示（「こんなトークを作って」等）からPasta DSLコードを正確に生成するサポートを提供する。

**対象**: ゴースト（「伺か」デスクトップマスコット）の辞書ファイル（`.pasta`）の作成・編集。

**前提条件**: ゴーストプロジェクトが既に存在すること（`pasta.toml`、`descript.txt`、`dic/` ディレクトリが揃っている）。

**役割分離**: 本スキルはLLMによるコード生成に特化する。Pasta DSL言語仕様の設計判断やパーサー実装には関与しない。
- `references/`（詳細リファレンス）と `SKILL.md`（要約＋パターン集）の2層構成
- 文法・`pasta.toml` の挙動の権威は pasta 利用者マニュアル（<https://ekicyou.github.io/pasta/>）であり、`references/` の生成ファイルはその写しである

**自己完結性**: 必要な文法ルールはすべて `references/` に内包している。永続化メカニズムや Lua ランタイムの実装詳細については `pasta-lua-coding` スキルへのクロスリファレンスを含む。

### references 一覧

`references/` の各ファイルの区分（マニュアルから生成／スキル手書き）を次の表に示す。

- **生成ファイルは編集しない**。修正はマニュアルの該当章で行い、pasta リポジトリで再生成する。生成ファイルと手書きファイルが同じ事実を扱う場合は、生成ファイルの記述を正とする。
- この `SKILL.md` 本文の要約・早見表（§2 のマーカー一覧表、§3〜§5 の要約など）は、参照先を選ぶための**非規範の要約**である。食い違う場合は生成ファイルが正である。
- このスキルを別の場所へ持ち出して使っている場合、持ち出し先を更新するときは `references/` を丸ごと置き換える（旧名のファイルを残さない）。

| ファイル | 区分 | 生成元マニュアル章 | 用途 |
| -------- | ---- | ------------------ | ---- |
| [grammar-index.md](references/grammar-index.md) | 生成（マニュアルから） | [文法リファレンス概要](https://ekicyou.github.io/pasta/grammar/index.html) | 文法の全体像（ファイル構造の俯瞰・式の基本例・各章への道案内） |
| [markers.md](references/markers.md) | 生成（マニュアルから） | [キーワード・マーカー](https://ekicyou.github.io/pasta/grammar/markers.html) | マーカー・演算子・区切り文字・識別子・空白・インデントの規則 |
| [block-structure.md](references/block-structure.md) | 生成（マニュアルから） | [行とブロック構造](https://ekicyou.github.io/pasta/grammar/block-structure.html) | 行の種類と置ける場所・ブロック構造・Lua ブロックの配置・選択肢行・キューコマンド行・属性・コメント・文字コード |
| [call-jump.md](references/call-jump.md) | 生成（マニュアルから） | [Call / Jump](https://ekicyou.github.io/pasta/grammar/call-jump.html) | Call（`＞`）・スコープ解決・候補の選択・チェイントーク・ゴースト終了・引数リスト |
| [literals.md](references/literals.md) | 生成（マニュアルから） | [リテラル型](https://ekicyou.github.io/pasta/grammar/literals.html) | 文字列・数値リテラル・型変換・引用符エスケープ |
| [action-line.md](references/action-line.md) | 生成（マニュアルから） | [アクション行](https://ekicyou.github.io/pasta/grammar/action-line.html) | アクション行・インライン要素・区切り・行継続・改行 |
| [sakura-script.md](references/sakura-script.md) | 生成（マニュアルから） | [さくらスクリプト](https://ekicyou.github.io/pasta/grammar/sakura-script.html) | さくらスクリプトの字句構造・配置ルール・主要タグ |
| [variables.md](references/variables.md) | 生成（マニュアルから） | [変数・スコープ](https://ekicyou.github.io/pasta/grammar/variables.html) | 変数の種類とスコープ・式・式文・保存先・日時変数などエンジンが値を入れる変数 |
| [words.md](references/words.md) | 生成（マニュアルから） | [単語定義](https://ekicyou.github.io/pasta/grammar/words.html) | 単語定義・参照・シャッフル＆順次消費・スコープと優先順位 |
| [actor-dictionary.md](references/actor-dictionary.md) | 生成（マニュアルから） | [アクター辞書](https://ekicyou.github.io/pasta/grammar/actor-dictionary.html) | アクター辞書・`％` 行による立ち位置・バルーン連携・外見の復旧・コードブロック |
| [pasta-toml.md](references/pasta-toml.md) | 生成（マニュアルから） | [pasta.toml リファレンス](https://ekicyou.github.io/pasta/reference/pasta-toml.html) | `pasta.toml` の全セクション・全キー・既定値 |
| [authoring-patterns.md](references/authoring-patterns.md) | 手書き（スキルが権威） | — | 辞書制作の作例・ファイル分割指針・自然言語→シーン変換の手順 |

---

## §2 Quick Reference（マーカー一覧表）

> この表は参照先を選ぶための非規範の要約である。食い違う場合は生成ファイル（[markers.md](references/markers.md) ほか）が正。

マーカーは全角・半角の両方を許容する（例外: さくらスクリプトの `\` は半角のみ。ローカルシーンの半角は `-` で、全角の `－` は使えない）。コード例では全角を使用する。リテラル（文字列・数値）の書き方は [literals.md](references/literals.md) を参照。

| マーカー名       | 全角   | 半角 | 用途                                 | 使用例                                                                    | リファレンス                                                        |
| ---------------- | ------ | ---- | ------------------------------------ | ------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| グローバルシーン | `＊`   | `*`  | シーン定義                           | `＊OnBoot`                                                                | [block-structure.md](references/block-structure.md#グローバルシーン) |
| ローカルシーン   | `・`   | `-`  | サブシーン定義                       | `・選択肢1`                                                               | [block-structure.md](references/block-structure.md#ローカルシーン)   |
| 単語/関数        | `＠`   | `@`  | 単語定義・参照・関数呼び出し         | `＠挨拶：こんにちは、やあ` / `＠女性、妖精：水無灯里、アリス`（複数キー） | [words.md](references/words.md)                                     |
| 選択肢           | `＠？` | `@?` | 選択肢定義                           | `＠？挨拶「挨拶する」`                                                    | [block-structure.md](references/block-structure.md#選択肢行)         |
| 変数             | `＄`   | `$`  | 変数代入・参照                       | `＄count＝1` / `＄％prop＝「値」`                                         | [variables.md](references/variables.md)                             |
| 代入             | `＝`   | `=`  | 変数代入の区切り（`：` では代入不可） | `＄count＝1`                                                              | [variables.md](references/variables.md#変数の種類とスコープ)         |
| Call             | `＞`   | `>`  | シーン呼び出し                       | `＞次の会話`                                                              | [call-jump.md](references/call-jump.md)                             |
| 属性             | `＆`   | `&`  | メタデータ（処理には反映されない）   | `＆author：Alice`                                                         | [block-structure.md](references/block-structure.md#属性)             |
| コメント         | `＃`   | `#`  | コメント行                           | `＃ メモ`                                                                 | [block-structure.md](references/block-structure.md#コメント)         |
| アクター辞書     | `％`   | `%`  | アクター辞書定義・アクター指定行     | `％さくら`                                                                | [actor-dictionary.md](references/actor-dictionary.md)               |
| キューコマンド   | `！`   | `!`  | 演出指示（`select` だけが有効）      | `！select(30)`                                                            | [block-structure.md](references/block-structure.md#キューコマンド行) |
| コロン           | `：`   | `:`  | キー・値の区切り                     | `Alice：こんにちは`                                                       | [markers.md](references/markers.md#コロン)                          |
| さくらスクリプト | （なし） | `\`  | 表情・タイミング制御                 | `\s[0]`                                                                   | [sakura-script.md](references/sakura-script.md)                     |

---

## §3 DSL Syntax（構文ルール）

以下は非規範の要約。詳細と正確な規則は各節末尾の生成ファイルを参照する。

### 3.1 Scenes（シーン定義）

- **グローバルシーン** `＊シーン名`: 全辞書ファイルから呼び出せる（ファイルをまたいで共通）。インデントなしで記述
- **ローカルシーン** `・シーン名`: 親グローバルシーン内でのみアクセス可能。インデントありで記述。アクション行は0個以上（空ローカルシーン可）
- **重複シーン**: 同名のグローバルシーンを複数定義すると、実行時にシャッフル＆順次消費方式で選択される（全候補を一巡するまで同じシーンは再選択されない。詳細は [call-jump.md](references/call-jump.md#候補の選択)、作例は [authoring-patterns.md §6.6](references/authoring-patterns.md#s6-6)）
- **前方一致検索**: `＞挨拶` で「挨拶朝」「挨拶昼」の両方が候補になる

```pasta
＊OnTalk
  女の子：こんにちは！
＊OnTalk
  女の子：やっほー！
```

> 📖 詳細: [references/block-structure.md](references/block-structure.md#ブロック構造)・全体像は [references/grammar-index.md](references/grammar-index.md)

### 3.2 Action Lines（アクション行）

- 構文: `アクター名：発話内容`（インデントあり）
- インライン要素として `＠単語参照`、`＄変数参照`、さくらスクリプト（`\s[0]`等）を埋め込み可能
- **動的単語参照** `＠＄変数名`: 変数の値を単語キーとして単語を選ぶ（`＠＄＊変数名`・`＠＄０` も可）。`＠＄変数名（引数）` は変数の値を関数名とする動的関数呼び出しになる（詳細は [words.md](references/words.md#動的単語参照)）
- **継続行**: アクター名を書かず、インデントの後に `：` を置いた行は、直前のアクション行と同じアクターの発話として続けて出力される。`：` を置かずテキストだけを書いた行は継続行にならず、パースエラーになる
- アクション行・継続行の行末の `＃` はコメントにならず、台詞として出力される

```pasta
＊会話
  Alice：こんにちは、＠weather　ですね。\w8
  ：今日もよろしくね。
  Bob：＄player_name　さん、元気？
```

#### インライン要素の区切り文字

インライン要素（`＠`、`＄`）の識別子は**最長一致**で切り出される。日本語文字（平仮名・カタカナ・漢字）は識別子に含まれるため、空白なしでは後続テキストが識別子に吸収される。

1. **空白区切り** — `＠単語名　テキスト` で単語参照と通常テキストを分離。空白はトークン区切りとして消費され出力に含まれない（空白数は無関係）
2. **最長一致（空白なし）** — `＠天気ですね` は「天気ですね」全体が識別子として吸収される
3. **＠＠エスケープ** — リテラルの「＠」を出力するには `＠＠` と記述（「＄」は `＄＄`）

```
❌ ＠地名からおらんようなってもた
✅ ＠地名　からおらんようなってもた

❌ ＄nameさん
✅ ＄name　さん
```

#### ⚠️ よくある間違い

| #   | パターン               | ❌ まちがい                                    | ✅ ただしい                                                | 理由                                                   |
| --- | ---------------------- | --------------------------------------------- | --------------------------------------------------------- | ------------------------------------------------------ |
| a   | ＠空白なし             | `＠地名からおらんようなってもた`              | `＠地名　からおらんようなってもた`                        | 最長一致で全体が識別子に                               |
| b   | ＄空白なし             | `＄nameさん`                                  | `＄name　さん`                                            | 日本語文字も識別子に含まれる                           |
| c   | 継続行のコロン省略     | 継続行をテキストだけで開始                    | 継続行はインデント＋`：` で開始                           | `：` の無いテキストだけの行はパースエラー              |
| d   | 属性の配置位置         | アクション行の後・ローカルシーン宣言の次行に属性行 | グローバルシーンの初期部に属性行／ローカルシーンは宣言行に付記（`・名前＆key：value`） | 属性行はグローバルシーンの初期部などにだけ置ける       |
| e   | アクション行の行末コメント | `Alice：こんにちは　＃ 挨拶`              | コメントは別の行に `＃ 挨拶` と書く                       | アクション行の `＃` 以降は台詞として出力される         |

> 📖 詳細: [references/action-line.md](references/action-line.md)

### 3.3 Words（単語定義）

- 構文（単一キー）: `＠単語名：値1、値2、値3`（区切りは `、` `，` `,` のいずれか）
- 構文（複数キー）: `＠キー1、キー2、キー3：値1、値2、値3` — 同一値リストを複数のキー名に同時登録
- **グローバル単語**: インデントなしで定義。全辞書ファイルのすべてのシーンから参照可能
- **ローカル単語**: グローバルシーンの初期部（宣言直後、最初のアクション行より前）にインデントして定義。そのグローバルシーンと配下のローカルシーンからだけ参照可能
- **複数キー**: キー区切りは全角読点（`、`）・全角コンマ（`，`）・半角カンマ（`,`）のいずれも可。1キーの場合は従来形式と同一。グローバル・ローカル・アクタースコープすべてで有効（作例は [authoring-patterns.md §6.10](references/authoring-patterns.md#s6-10)）
- 参照時 `＠単語名` で値リストからシャッフル＆順次消費方式で1つ選択される（詳細は [words.md](references/words.md#シャッフル順次消費)）
- スコープ解決: ローカル → グローバルの順に前方一致検索。ローカル単語に候補があればそれだけを使い、グローバル単語の候補とは合わせない
- 単語定義の行末にコメントを書けるのは、値を `「」` または `"` で囲んだときだけ。引用なしの値の単語定義にコメントを付けるときは別の行に書く（[block-structure.md](references/block-structure.md#コメント)）

```pasta
＠挨拶：こんにちは、おはよう、やあ
＃ 2キー：同一候補を2名称で参照
＠女性、水の妖精：水無灯里、アリス・キャロル
＊会話
  ＠天気：晴れ、雨、曇り
  Alice：＠挨拶　今日は＠天気　だね。
```

> 📖 詳細: [references/words.md](references/words.md)

### 3.4 Variables（変数）

- **ローカル変数** `＄変数名`: 1 回のイベント処理の間だけ有効（Call 先のシーンとも共有）
- **グローバル変数** `＄＊変数名`: Lua の `save` テーブルに入り、ゴーストを終了しても JSON ファイルに保存されて次の起動時に戻る
- **プロパティ変数** `＄％prop.path`: SSP共有プロパティの読み書き。SET（`＄％prop＝「値」`）、GET代入（`＄var＝＄％prop`）、インラインGET（`アクター：＄％prop`）が可能
- 代入: `＄変数名＝値`（代入は `＝` のみ。`＄変数名：値` はパースエラー）。値にはリテラル値・単語参照・変数参照・式・関数呼び出しが使用可能
- **グローバル関数代入**: `＄result＝＠＊func()` → `GLOBAL.func(act)` の戻り値を代入
- **式文（副作用のみ）**: `＄＝expr` — 戻り値を使わず式を実行するだけ
- 参照: アクション行内で `＄変数名` と記述

```pasta
＊会話
  ＄count＝1
  ＄＊total＝＄＊total＋1
  ＄result＝＠＊globalFunc()     ＃ グローバル関数の戻り値を代入
  ＄＝＠＊logEvent（「起動」）   ＃ 戻り値不要の式文
  ＄％sakura.name＝「Alice」     ＃ SSP共有プロパティに書き込み
  ＄name＝＄％sakura.name        ＃ SSP共有プロパティを変数に代入
  Alice：＄count　回目の会話だよ。
  ＃ インラインでプロパティ参照
  Alice：名前は ＄％sakura.name　です。
```

> 📖 詳細: [references/variables.md](references/variables.md)
> 📖 グローバル変数の保存先: [references/variables.md](references/variables.md#グローバル変数の保存先)・エンジン予約の `＄＊pasta_` 変数: [references/variables.md](references/variables.md#予約グローバル変数pasta_-で始まる名前)

### 3.5 Call Statements（Call文）

- 構文: `＞シーン名` — 指定シーンを呼び出し、実行後に復帰
- 動的ターゲット: `＞式` — 式（`＞＄変数名`・`＞＠関数（）`・`＞「文字列」` など）の評価結果を文字列にしてシーン名として解決
- 前方一致で候補が複数ある場合はシャッフル＆順次消費で1つ選ぶ
- 特殊Call: `＞ゴースト終了（ミリ秒）` — ゴーストを終了させる
- 特殊Call: `＞チェイントーク` / `＞yield` — シーン出力を分割し、次回 OnTalk の機会に残りを出力する（詳細は [call-jump.md](references/call-jump.md#チェイントーク)、作例は [authoring-patterns.md §6.7](references/authoring-patterns.md#s6-7)）

```pasta
＊OnClose
  女の子：またね！
  ＞ゴースト終了（３００）
```

> 📖 詳細: [references/call-jump.md](references/call-jump.md)

### 3.6 Actor Dictionary（アクター辞書）

- **定義**: `％アクター名` でアクター固有の単語辞書を定義する（インデントなし）
- 配下にインデント付きで `＠単語名：値` を記述し、表情等をアクター単位で管理する
- **立ち位置の指定**: グローバルシーンの初期部に `％名前1、名前2` と記述すると、並べた順にスポット番号（0, 1, …）を設定する。設定は次に `％` 行が実行されるまでイベントをまたいで保たれる
  - SHIORIゴーストでは通常 OnBoot で一度設定して固定する
  - ノベルゲーム用途等ではシーンごとに切り替えることも可能
- 会話行で `アクター名：＠表情名` と記述すると、そのアクターの辞書から優先的に検索される（`％` 行が無くても働く）
- アクター辞書に該当単語がない場合、ローカル単語辞書 → グローバル単語辞書の順に探される
- 複数値（`＠単語名：値1、値2、値3`）はグローバル/ローカル単語と同じシャッフル＆順次消費方式で選択される

```pasta
％さくら
  ＠通常：\s[0]
  ＠笑顔：\s[1]

＊OnBoot
  ％さくら、うにゅう
  さくら：＠笑顔　おはよう！
```

> 📖 詳細: [references/actor-dictionary.md](references/actor-dictionary.md)

### 3.7 Sakura Script（さくらスクリプト）

アクション行内にインラインで埋め込む `\` から始まるコマンド。Pasta は内容を解釈せずそのまま透過する。

| タグ        | 用途                   | 例          |
| ----------- | ---------------------- | ----------- |
| `\s[ID]`    | 表情変更               | `\s[0]`     |
| `\n`        | 改行                   | 行内改行    |
| `\w数字`    | ウェイト（数字×50ms）  | `\w8`       |
| `\_w[数字]` | ウェイト（ミリ秒指定） | `\_w[1000]` |

さくらスクリプトは必ず半角で記述する（エスケープ文字 `\` は半角バックスラッシュのみ）。アクターの切り替えに伴う `\p[N]` は Pasta が自動で出力する。

> 📖 詳細: [references/sakura-script.md](references/sakura-script.md)

### 3.8 Lua Code Blocks（Luaブロック）

高度なロジックが必要な場合のエスケープハッチ。辞書制作では Pasta DSL 構文のみで十分なケースが大半。

- 3 個以上のバッククォート（` ```lua ` など。識別子は任意）で開き、同じ本数で閉じる。フェンスは**行頭**に置く（インデントするとパースエラー）
- 置ける位置は、グローバルシーンの初期部の後（暗黙の開始ブロックの台詞より前）、または暗黙の開始ブロック・ローカルシーンの末尾
- 内容は検証されず、辞書の読み込み時に一度評価される。関数定義以外の文（`local` 変数の宣言・`require` など）も書ける
- `function SCENE.名前(act)` で定義した関数はアクション行内で `＠関数名()` として呼び出せる

````pasta
＊計算
```lua
function SCENE.calculate(act)
    local save, var = act:init_scene(SCENE)
    return 10 + 20
end
```
  Alice：結果は＠calculate()　です。
````

> 📖 詳細: [references/block-structure.md](references/block-structure.md#lua-ブロックの配置)

### 3.9 Comments & Attributes（コメント・属性）

- **コメント**: `＃` または `#` で始まる行。処理されない。インデントあり・なし両方可。アクション行・継続行の行末に書いた `＃` はコメントにならず台詞として出力される（行末コメントを書ける行は [block-structure.md](references/block-structure.md#コメント) を参照）
- **属性**: `＆key：value` 形式のメタデータ。構文は受理されるが、シーンの選択・出力には反映されない
- 属性行を置けるのは、グローバルシーンの初期部（宣言直後、アクション行などより前）・ファイルレベル・アクター辞書の配下。ローカルシーンには宣言行への付記（`・名前＆key：value`）としてだけ書ける
- アクション行や変数代入行の後に属性行は置けない

```pasta
＃ これはコメント
＊会話
  ＆author：Alice
  ＆genre：comedy
  Alice：こんにちは！
```

> 📖 詳細: [references/block-structure.md](references/block-structure.md#コメント)・[属性](references/block-structure.md#属性)

### 3.10 Choice Lines（選択肢行）

選択肢行は `＠？` マーカーで開始し、プレイヤーに提示する選択肢を宣言的に定義する。

#### 省略形
```text
＠？挨拶
```
ターゲット名がそのまま表示テキストになる。

#### 括弧形
```text
＠？挨拶「挨拶する」
```
「」内が表示テキスト。ターゲット名と異なるラベルを指定できる。

#### 選択肢タイムアウト
```text
!select(30)
```
`!select(秒数)` キューコマンドで選択の制限時間を設定する。

#### 自動ルーティング
選択後は既定の `OnChoiceSelectEx` イベントハンドラが、選択 ID と前方一致するローカルシーンを、直前に実行したグローバルシーン（通常は選択肢を出したシーン）の配下から探して自動実行する。ローカルシーンが無ければ、選択 ID と前方一致するグローバルシーンを探す。`＊OnChoiceSelectEx` という名前のシーンがあれば、そちらが優先される。詳細はマニュアルの [OnChoiceSelectEx](https://ekicyou.github.io/pasta/lua/shiori-events.html#onchoiceselectex) を参照。

#### 使用例
```pasta
＊OnMouseDoubleClick
　％女の子、男の子
　女の子：＠通常　何をしますか？
　＠？挨拶「挨拶する」
　＠？自己紹介
　!select(30)

　・挨拶
　　女の子：＠笑顔　こんにちは！

　・自己紹介
　　女の子：＠通常　私は女の子だよ。
```

> 📖 詳細: [references/block-structure.md](references/block-structure.md#選択肢行)

---

## §4 Project Structure（プロジェクト構造）

| ファイル       | 役割                                             |
| -------------- | ------------------------------------------------ |
| `dic/*.pasta`  | 辞書ファイル（トーク・イベントハンドラ等を記述） |
| `pasta.toml`   | ゴースト設定ファイル                             |
| `descript.txt` | ゴーストメタデータ                               |

### pasta.toml（ゴースト設定）

ゴーストの動作を制御する設定ファイル。主要セクション:

| セクション       | 用途                                                   | 辞書制作者向け重要度 |
| ---------------- | ------------------------------------------------------ | -------------------- |
| `[loader]`       | 辞書ファイルの読み込みパターン                         | ★★★                  |
| `[ghost]`        | トーク間隔・時報マージン等                             | ★★★                  |
| `[actor."名前"]` | バルーン割当・BudouX 自動改行・外見の復旧              | ★★★                  |
| `[talk]`         | ウェイト・禁則処理のカスタマイズ                       | ★★                   |
| `[persistence]`  | 保存ファイルの形式・場所                               | ★                    |
| `[logging]`      | ログ出力の設定                                         | ★                    |
| `[debug]`        | デバッグバックエンド（上級者向け）                     | ★                    |
| `[lua]`          | Lua ライブラリ（上級者向け。通常は書かない）           | ★                    |
| `[package]`      | エンジンプロファイル専用（SHIORI では不要・書いても無視） | —                 |

> 📖 全セクション・全キーの詳細: [references/pasta-toml.md](references/pasta-toml.md)

- `[actor."名前"]` の `spot` はバルーン割り当て（0=sakura側, 1=kero側）
- `pasta_patterns` の既定値 `["dic/**/*.pasta"]` により、`dic/` 配下の全 `.pasta` ファイルが自動的に読み込まれる
- `[actor."名前"]` の名前は辞書で使うアクター名と一致させる（`descript.txt` の `sakura.name` / `kero.name` とそろえると分かりやすい）

### descript.txt（必須フィールド）

| フィールド    | 説明                 | 例            |
| ------------- | -------------------- | ------------- |
| `charset`     | 文字エンコーディング | `UTF-8`       |
| `type`        | リソース種別         | `ghost`       |
| `name`        | ゴースト名           | `hello-pasta` |
| `sakura.name` | メインキャラ名       | `女の子`      |
| `kero.name`   | サブキャラ名         | `男の子`      |
| `shiori`      | SHIORIモジュール     | `pasta.dll`   |

---

## §5 Event Mapping（SHIORIイベントマッピング）

**核心ルール**: `＊イベント名` のグローバルシーンを定義するだけで、対応するイベント発生時に自動実行される（シーン関数フォールバック）。イベントごとの扱いの詳細はマニュアルの [SHIORI イベントとハンドラ](https://ekicyou.github.io/pasta/lua/shiori-events.html) を参照。

| やりたいこと       | シーン名               | 備考                                                                                                                                  |
| ------------------ | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| 起動時の挨拶       | `＊OnBoot`             | 通常起動                                                                                                                              |
| 初回起動の挨拶     | `＊OnFirstBoot`        | 初回のみ                                                                                                                              |
| 終了時の挨拶       | `＊OnClose`            | 末尾に `＞ゴースト終了（ミリ秒）` を付ける                                                                                            |
| ランダムトーク     | `＊OnTalk`             | 仮想イベント。同名複数定義でランダム選択。継続トーク（[authoring-patterns.md §6.7](references/authoring-patterns.md#s6-7)）対応       |
| 汎用時報           | `＊時報その他`         | 仮想イベント。日時変数が自動設定される（[variables.md](references/variables.md#日時変数)、作例は [authoring-patterns.md §6.4](references/authoring-patterns.md#s6-4)）。`＊OnHourOther` も同等 |
| 時刻別時報         | `＊時報{HH}`           | 特定時刻専用（例: `＊時報12` で正午専用）。日時変数が自動設定される。`＊OnHour{HH}` も同等                                            |
| ダブルクリック反応 | `＊OnMouseDoubleClick` | 同名複数定義でランダム選択                                                                                                            |

**仮想イベント**: OnTalk と OnHour は内部タイマーにより自動ディスパッチされる。トーク間隔は `pasta.toml` の `[ghost]` セクションで設定可能。

**OnHour 4段階フォールバック**: 正時に以下の順序でシーンを検索し、最初に見つかったシーンを実行する:
1. `時報{HH}` — 時刻別（例: `時報12` で正午専用）
2. `OnHour{HH}` — 英語時刻別（例: `OnHour12`）
3. `時報その他` — 汎用時報
4. `OnHourOther` — 英語汎用時報

`{HH}` は24時間制0埋め2桁（00〜23）。旧シーン名 `＊OnHour` はフォールバック候補に含まれないため、`＊時報その他` または `＊OnHourOther` に移行が必要。

---

## §6 Authoring Patterns（辞書制作パターン集）

辞書ファイル（`.pasta`）の実践的な記述パターン。

| パターン                  | 内容                           | 代表ファイル   |
| ------------------------- | ------------------------------ | -------------- |
| §6.1 アクター辞書定義     | `％名前` + `＠表情：\s[ID]`    | `actors.pasta` |
| §6.2 イベントハンドラ     | OnBoot / OnFirstBoot / OnClose | `boot.pasta`   |
| §6.3 ランダムトーク       | 同名 `＊OnTalk` の複数定義     | `talk.pasta`   |
| §6.4 時報                 | 4段階フォールバック + 日時変数 | `hour.pasta`   |
| §6.5 クリック反応         | OnMouseDoubleClick             | `click.pasta`  |
| §6.6 シャッフル＆順次消費 | 単語・シーンの選択アルゴリズム | —              |
| §6.7 継続トーク           | `＞チェイントーク` / `＞yield` | `talk.pasta`   |
| §6.8 ファイル分割ガイド   | 責務別ファイル構成             | —              |
| §6.9 自然言語→シーン変換  | LLM 向け変換ワークフロー       | —              |
| §6.10 複数キー単語定義    | `＠キー1、キー2：値` 構文      | —              |
| §6.11 選択肢メニュー      | `＠？` 選択肢定義              | —              |

### 代表パターン: ランダムトーク

同名シーン `＊OnTalk` を複数定義するだけで、シャッフル＆順次消費方式でランダムに選択される。

```pasta
＠雑談：何か用？、暇だなあ...、ねえねえ
＊OnTalk
　％女の子、男の子
　女の子：＠通常　＠雑談
＊OnTalk
　％女の子、男の子
　女の子：＠笑顔　今日はいい天気だね！
　男の子：＠通常　そうだね。
```

> 📖 全パターンの詳細・応用例: [references/authoring-patterns.md](references/authoring-patterns.md)
