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
  version: "1.5.0"
---

# Pasta Ghost Authoring Skill

## §1 Purpose & Prerequisites

**目的**: 自然言語の指示（「こんなトークを作って」等）からPasta DSLコードを正確に生成するサポートを提供する。

**対象**: ゴースト（「伺か」デスクトップマスコット）の辞書ファイル（`.pasta`）の作成・編集。

**前提条件**: ゴーストプロジェクトが既に存在すること（`pasta.toml`、`descript.txt`、`dic/` ディレクトリが揃っている）。

**役割分離**: 本スキルはLLMによるコード生成に特化する。Pasta DSL言語仕様の設計判断やパーサー実装には関与しない。
- `references/`（詳細リファレンス）と `SKILL.md`（要約＋パターン集）の2層構成
- SKILL.md と `references/` の記述に矛盾がある場合、`references/` を正とする

**自己完結性**: 必要な文法ルールはすべて `references/` に内包している。永続化メカニズムや Lua ランタイムの実装詳細については `pasta-lua-coding` スキルへのクロスリファレンスを含む。

---

## §2 Quick Reference（マーカー一覧表）

全マーカーは全角・半角の両方を許容する。コード例では全角を使用する。

| マーカー名       | 全角 | 半角 | 用途                         | 使用例                                                                    | リファレンス                                          |
| ---------------- | ---- | ---- | ---------------------------- | ------------------------------------------------------------------------- | ----------------------------------------------------- |
| グローバルシーン | `＊` | `*`  | シーン定義                   | `＊OnBoot`                                                                | [grammar-model.md](references/grammar-model.md)       |
| ローカルシーン   | `・` | `-`  | サブシーン定義               | `・選択肢1`                                                               | [grammar-model.md](references/grammar-model.md)       |
| 単語/関数        | `＠` | `@`  | 単語定義・参照・関数呼び出し | `＠挨拶：こんにちは、やあ` / `＠女性、妖精：水無灯里、アリス`（複数キー） | [words.md](references/words.md)                       |
| 選択肢           | `＠？` | `@?` | 選択肢定義                   | `＠？挨拶「挨拶する」`                                                    | [grammar-model.md](references/grammar-model.md)       |
| 変数             | `＄` | `$`  | 変数宣言・参照               | `＄count＝1` / `＄％prop＝v`                                              | [variables.md](references/variables.md)               |
| Call             | `＞` | `>`  | シーン呼び出し               | `＞次の会話`                                                              | [call-spec.md](references/call-spec.md)               |
| 属性             | `＆` | `&`  | メタデータ                   | `＆author：Alice`                                                         | [grammar-model.md](references/grammar-model.md)       |
| コメント         | `＃` | `#`  | コメント行                   | `＃ メモ`                                                                 | [grammar-model.md](references/grammar-model.md)       |
| アクター辞書     | `％` | `%`  | アクター辞書定義             | `％さくら`                                                                | [actor-dictionary.md](references/actor-dictionary.md) |
| キューコマンド   | `！` | `!`  | 演出キュー                   | `！emote(smile)`                                                          | [grammar-model.md](references/grammar-model.md)       |
| コロン           | `：` | `:`  | キー・値の区切り             | `Alice：こんにちは`                                                       | [grammar-model.md](references/grammar-model.md)       |
| さくらスクリプト | `\`  | `\`  | 表情・タイミング制御         | `\s[0]`                                                                   | [sakura-script.md](references/sakura-script.md)       |

---

## §3 DSL Syntax（構文ルール）

### 3.1 Scenes（シーン定義）

- **グローバルシーン** `＊シーン名`: ファイル全体からアクセス可能。インデントなしで記述
- **ローカルシーン** `・シーン名`: 親グローバルシーン内でのみアクセス可能。インデントありで記述。アクション行は0個以上（空ローカルシーン可）
- **重複シーン**: 同名のグローバルシーンを複数定義すると、実行時にシャッフル＆順次消費方式で選択される（全シーンを一巡するまで同じシーンは再選択されない。詳細は [authoring-patterns.md §6.6](references/authoring-patterns.md#s6-6)）
- **前方一致検索**: `＞挨拶` で「挨拶朝」「挨拶昼」の両方が候補になる

```pasta
＊OnTalk
  女の子：こんにちは！
＊OnTalk
  女の子：やっほー！
```

> 📖 詳細: [references/grammar-model.md](references/grammar-model.md)

### 3.2 Action Lines（アクション行）

- 構文: `アクター名：発話内容`（インデントあり）
- アクター名を省略すると、直前のアクターが継続する
- インライン要素として `＠単語参照`、`＄変数参照`、さくらスクリプト（`\s[0]`等）を埋め込み可能
- 行継続: インデント付きの次行で発話を継続できる（マーカーで始まる行は継続しない）

```pasta
＊会話
  Alice：こんにちは、＠weather　ですね。\w8
  Bob：＄player_name　さん、元気？
```

#### インライン要素の区切り文字

インライン要素（`＠`、`＄`）の識別子はUnicode識別子（XID_START＋XID_CONTINUE*）として**最長一致**で切り出される。日本語文字（平仮名・カタカナ・漢字）はXID_CONTINUEに含まれるため、空白なしでは後続テキストが識別子に吸収される。

1. **空白区切り** — `＠単語名　テキスト` で単語参照と通常テキストを分離。空白はトークン区切りとして消費され出力に含まれない（空白数は無関係）
2. **最長一致（空白なし）** — `＠天気ですね` は「天気ですね」全体が識別子として吸収される
3. **＠＠エスケープ** — リテラルの「＠」を出力するには `＠＠` と記述

```
❌ ＠地名からおらんようなってもた
✅ ＠地名　からおらんようなってもた

❌ ＄nameさん
✅ ＄name　さん
```

#### ⚠️ よくある間違い

| #   | パターン           | ❌ まちがい                       | ✅ ただしい                         | 理由                                   |
| --- | ------------------ | -------------------------------- | ---------------------------------- | -------------------------------------- |
| a   | ＠空白なし         | `＠地名からおらんようなってもた` | `＠地名　からおらんようなってもた` | 最長一致で全体が識別子に               |
| b   | ＄空白なし         | `＄nameさん`                     | `＄name　さん`                     | 日本語文字もXID_CONTINUEに含まれる     |
| c   | 行継続で行マーカー | 継続行を`＠`で開始               | 継続行はマーカーなしで開始         | マーカーで始まる行は別の行種として解釈 |
| d   | 属性の配置位置     | アクション行→属性行              | シーン定義直後→属性行              | 属性はシーン定義の直後にのみ配置可能   |

> 📖 詳細: [references/action-line.md](references/action-line.md)

### 3.3 Words（単語定義）

- 構文（単一キー）: `＠単語名：値1、値2、値3`（区切りは `、` `，` `,` のいずれか）
- 構文（複数キー）: `＠キー1、キー2、キー3：値1、値2、値3` — 同一値リストを複数のキー名に同時登録
- **グローバル単語**: インデントなしで定義。ファイル全体から参照可能
- **ローカル単語**: インデントありで定義。親シーン内でのみ参照可能
- **複数キー**: キー区切りは全角読点（`、`）・全角コンマ（`，`）・半角カンマ（`,`）のいずれも可。1キーの場合は従来形式と同一（後方互換）。グローバル・ローカル・アクタースコープすべてで有効（詳細は [authoring-patterns.md §6.10](references/authoring-patterns.md#s6-10)）
- 参照時 `＠単語名` で値リストからシャッフル＆順次消費方式で1つ選択される（詳細は [authoring-patterns.md §6.6](references/authoring-patterns.md#s6-6)）
- スコープ解決: ローカル → グローバルの順に前方一致検索

```pasta
＠挨拶：こんにちは、おはよう、やあ
＠女性、水の妖精：水無灯里、アリス・キャロル　＃ 2キー：同一候補を2名称で参照
＊会話
  ＠天気：晴れ、雨、曇り
  Alice：＠挨拶　今日は＠天気　だね。
```

> 📖 詳細: [references/words.md](references/words.md)

### 3.4 Variables（変数）

- **ローカル変数** `＄変数名`: 一連のシーンが終わるまで有効
- **グローバル変数** `＄＊変数名`: SAVE テーブル経由でセッション間にわたりファイルに永続化される（JSON 保存）
- **プロパティ変数** `＄％prop.path`: SSP共有プロパティの読み書き。SET（`＄％prop＝value`）、GET代入（`＄var＝＄％prop`）、インラインGET（`アクター：＄％prop`）が可能
- 代入: `＄変数名＝値` または `＄変数名：値`（リテラル値・単語参照・変数参照・式・関数呼び出しが使用可能）
- **グローバル関数代入**: `＄result＝＠＊func()` → `GLOBAL.func(act)` の戻り値を代入
- **式文（副作用のみ）**: `＄＝expr` — 戻り値を使わず式を実行するだけ
- 参照: アクション行内で `＄変数名` と記述

```pasta
＊会話
  ＄count＝1
  ＄＊total＝＄＊total ＋ 1
  ＄result＝＠＊globalFunc()     ＃ グローバル関数の戻り値を代入
  ＄＝＠＊logEvent（「起動」）   ＃ 戻り値不要の式文
  ＄％sakura.name＝Alice         ＃ SSP共有プロパティに書き込み
  ＄name＝＄％sakura.name        ＃ SSP共有プロパティを変数に代入
  Alice：＄count　回目の会話だよ。
  Alice：名前は ＄％sakura.name　です。  ＃ インラインでプロパティ参照
```

> 📖 詳細: [references/variables.md](references/variables.md)
> 📖 永続化の詳細・SAVE エンジン予約キー: [references/variables.md](references/variables.md#永続化とsaveテーブル)

### 3.5 Call Statements（Call文）

- 構文: `＞シーン名` — 指定シーンを呼び出し、実行後に復帰
- 動的ターゲット: `＞expr` — 式の評価結果をシーン名として解決（`＞＄変数名` は `var_ref` としての代表ケース）
- 前方一致で候補が複数ある場合はランダム選択
- 特殊Call: `＞ゴースト終了（ミリ秒）` — ゴーストを終了させる
- 特殊Call: `＞チェイントーク` / `＞yield` — シーン出力を分割し、次回 OnTalk トリガーで後半を出力する（[authoring-patterns.md §6.7](references/authoring-patterns.md#s6-7)）

```pasta
＊OnClose
  女の子：またね！
  ＞ゴースト終了（３００）
```

> 📖 詳細: [references/call-spec.md](references/call-spec.md)

### 3.6 Actor Dictionary（アクター辞書）

- **定義**: `％アクター名` でアクター固有の単語辞書を定義する（インデントなし）
- 配下にインデント付きで `＠単語名：値` を記述し、表情等をアクター単位で管理する
- **スコープ指定**: シーン内で `％名前1、名前2` と記述すると、そのシーンでバルーン連動が有効になる
  - SHIORIゴーストでは通常 OnBoot で一度設定して固定する
  - ノベルゲーム用途等ではシーンごとに切り替えることも可能
- 会話行で `アクター名：＠表情名` と記述すると、そのアクターの辞書から優先的に検索される
- アクター辞書に該当単語がない場合、グローバル/ローカル単語辞書にフォールバック
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

さくらスクリプトは必ず半角で記述する（エスケープ文字 `\` は半角バックスラッシュのみ）。

> 📖 詳細: [references/sakura-script.md](references/sakura-script.md)

### 3.8 Lua Code Blocks（Luaブロック）

高度なロジックが必要な場合のエスケープハッチ。辞書制作では Pasta DSL 構文のみで十分なケースが大半。

- グローバルシーン直下に ` ```lua ` 〜 ` ``` ` で囲んで記述（インデント不要）
- **関数定義のみ許可**（変数宣言やステートメントは不可）
- 定義した関数はアクション行内で `＠関数名()` として呼び出せる

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

> 📖 詳細: [references/grammar-model.md](references/grammar-model.md)

### 3.9 Comments & Attributes（コメント・属性）

- **コメント**: `＃` または `#` で始まる行。処理されない。インデントあり・なし両方可
- **属性**: `＆key：value` 形式。シーン定義の直後にのみ配置可能（メタデータ付与）
- 属性はアクション行や変数代入行の後には配置できない

```pasta
＃ これはコメント
＊会話
  ＆author：Alice
  ＆genre：comedy
  Alice：こんにちは！
```

> 📖 詳細: [references/grammar-model.md](references/grammar-model.md)

### 3.10 Choice Lines（選択肢行）

選択肢行は `＠？` マーカーで開始し、プレイヤーに提示する選択肢を宣言的に定義する。

#### 省略形
```pasta
＠？挨拶
```
ターゲット名がそのまま表示テキストになる。

#### 括弧形
```pasta
＠？挨拶「挨拶する」
```
「」内が表示テキスト。ターゲット名と異なるラベルを指定できる。

#### 選択肢タイムアウト
```pasta
!select(30)
```
`!select(秒数)` キューコマンドで選択の制限時間を設定する。

#### 自動ルーティング
選択後は `OnChoiceSelectEx` イベントハンドラが選択IDでシーンを前方一致検索し自動実行する。ローカルシーン → グローバルシーンの順で検索。

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

> 📖 詳細: [references/grammar-model.md](references/grammar-model.md)

---

## §4 Project Structure（プロジェクト構造）

| ファイル       | 役割                                             |
| -------------- | ------------------------------------------------ |
| `dic/*.pasta`  | 辞書ファイル（トーク・イベントハンドラ等を記述） |
| `pasta.toml`   | ゴースト設定ファイル                             |
| `descript.txt` | ゴーストメタデータ                               |

### pasta.toml（ゴースト設定）

ゴーストの動作を制御する設定ファイル。主要セクション:

| セクション       | 用途                             | 辞書制作者向け重要度 |
| ---------------- | -------------------------------- | -------------------- |
| `[loader]`       | 辞書ファイルの読み込みパターン   | ★★★                  |
| `[ghost]`        | トーク間隔・時報マージン等       | ★★★                  |
| `[actor."名前"]` | バルーン割当・BudouX 自動改行    | ★★★                  |
| `[talk]`         | ウェイト・禁則処理のカスタマイズ | ★★                   |
| `[persistence]`  | 保存ファイルの形式・場所         | ★                    |
| `[logging]`      | ログ出力の設定                   | ★                    |
| `[package]`      | パッケージメタデータ             | ★                    |
| `[lua]`          | Lua ライブラリ選択               | ★                    |

> 📖 全セクション・全キーの詳細: [references/pasta-toml.md](references/pasta-toml.md)

- `[actor."名前"]` の `spot` はバルーン割り当て（0=sakura側, 1=kero側）
- `pasta_patterns` により `dic/` 配下の全 `.pasta` ファイルが自動的に読み込まれる
- アクター名は `descript.txt` の `sakura.name` / `kero.name` と一致させる

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

**核心ルール**: `＊イベント名` のグローバルシーンを定義するだけで、対応するイベント発生時に自動実行される（シーン関数フォールバック）。

| やりたいこと       | シーン名               | 備考                                                                                                                                  |
| ------------------ | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| 起動時の挨拶       | `＊OnBoot`             | 通常起動                                                                                                                              |
| 初回起動の挨拶     | `＊OnFirstBoot`        | 初回のみ                                                                                                                              |
| 終了時の挨拶       | `＊OnClose`            | 末尾に `＞ゴースト終了（ミリ秒）` を付ける                                                                                            |
| ランダムトーク     | `＊OnTalk`             | 仮想イベント。同名複数定義でランダム選択。継続トーク（[authoring-patterns.md §6.7](references/authoring-patterns.md#s6-7)）対応       |
| 汎用時報           | `＊時報その他`         | 仮想イベント。日時変数が自動設定される（[authoring-patterns.md §6.4](references/authoring-patterns.md#s6-4)）。`＊OnHourOther` も同等 |
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
