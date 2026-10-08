# hello-pasta 段階表

入門ガイドは「こんな表現をしたい」を 1 段ずつ叶えながら、hello-pasta の辞書を育てていく。この文書は、どの表現をどの順で教えるかを定める**段階表の正本**である。入門ガイドの章立てと作例はこの表に従い、`cargo test -p pasta_sample_ghost`（`tests/tutorial_stages_test.rs`）はこの文書の 2 つの表を読んで全段階を検証する。

段階辞書は hello-pasta の配布辞書 `ghosts/hello-pasta/ghost/master/dic/*.pasta` そのものである。別置きの段階ディレクトリや複製は持たない。

## 段階表

| 段階 | 願い | 新しく覚える表現 | 使う文法要素 | 初めて扱うイベント | 追加するファイル |
|------|------|------------------|--------------|--------------------|------------------|
| 1 | しゃべらせたい | 起動したら一言しゃべる | `＊OnBoot`（[グローバルシーン](../../book/src/grammar/block-structure.md#グローバルシーン)）、[アクション行](../../book/src/grammar/action-line.md#基本構文) `アクター：台詞`、pasta.toml の [`[actor]`](../../book/src/reference/pasta-toml.md#actor名前アクター設定) | `OnBoot` | `01-boot.pasta` |
| 2 | 二人で掛け合いさせたい | 暇なときに 2 人でおしゃべりする。すぐ確かめたいときは `[ghost]` で間隔を短くする（配布版は 45〜75 秒） | `＊会話`（暇なときに pasta が呼ぶシーンの名前。[シーン名の別名](../../book/src/grammar/call-jump.md#シーン名の別名)）、2 人のアクション行、`：` の前の空白で位置をそろえる（[コロンの前後の空白は無視される](../../book/src/grammar/action-line.md#基本構文)）、pasta.toml の [`talk_interval_min`・`talk_interval_max`](../../book/src/reference/pasta-toml.md#ghostゴースト動作) | — | `02-talk.pasta` |
| 3 | 表情を変えたい | 台詞ごとに表情を付ける | [アクター辞書](../../book/src/grammar/actor-dictionary.md#グローバルアクター辞書定義) `％女の子`・`＠表情：\s[n]`、台詞の中の `＠表情`（台詞の途中で続けて変える例を含む） | — | `03-face.pasta` |
| 4 | 毎回ちがうことを言わせたい | 同じ名前のシーンから 1 つ選ばれる | 同名 `＊会話` の繰り返し、単独 `＊`（同名の別シーン。ファイル先頭は不可。[グローバルシーン](../../book/src/grammar/block-structure.md#グローバルシーン)）。別名は完全一致なので `＊会話朝` は別シーン | — | `04-variety.pasta` |
| 5 | 単語でちょこっと変えたい | 単語のランダム選択 | [グローバル単語](../../book/src/grammar/words.md#グローバル単語定義) `＠単語：a、b、c`、台詞の中の `＠単語` | — | `05-words.pasta` |
| 6 | 時刻を知らせたい | 正時に時報 | `＊時報12`・`＊時報その他`（[OnHour](../../book/src/lua/shiori-events.md#onhour) の 4 段の候補）、[日時変数](../../book/src/grammar/variables.md#日時変数) `＄時１２` | — | `06-hour.pasta` |
| 7 | 挨拶したい | ベースウェアのイベントに応える・付加情報を読む・ベースウェアに聞く | シーン名＝イベント名（[シーン関数フォールバック](../../book/src/lua/shiori-events.md#シーン関数フォールバック)）、[`＞transfer_req_to_var`](../../book/src/grammar/variables.md#リクエスト変数reference)、`＄ｒ０`、[プロパティ変数](../../book/src/grammar/variables.md#プロパティ変数) `＄％baseware.name`、OnGhostChanged・OnGhostChanging に応答すると OnBoot・OnClose は来ない、マニュアルの一覧に無いイベントも同名シーンで応答できる（[UKADOC の OnGhostChanging](https://ssp.shillest.net/ukadoc/manual/list_shiori_event.html#OnGhostChanging)） | `OnGhostChanged`・`OnGhostChanging`・`OnFirstBoot`・`OnClose` | `07-greeting.pasta` |
| 8 | 触ったら反応してほしい | 触られた部位で台詞を変える | `＊OnMouseDoubleClick`、[`＞transfer_req_to_var`](../../book/src/grammar/variables.md#リクエスト変数reference)、`＄ｒ４` | `OnMouseDoubleClick` | `08-touch.pasta` |
| 9 | 選ばせたい | 選択肢を出して選ばれた先へ進む | [選択肢行](../../book/src/grammar/block-structure.md#選択肢行) `＠？ジャンプ先「表示」`、[キューコマンド](../../book/src/grammar/block-structure.md#キューコマンド行) `!select(秒)`、選ばれた ID のシーンへの[自動ルーティング](../../book/src/lua/shiori-events.md#onchoiceselectex) | `OnChoiceSelectEx` | `09-choice.pasta` |
| 10 | 覚えていてほしい | 終了しても残る値 | [グローバル変数](../../book/src/grammar/variables.md#グローバル変数) `＄＊回数`、算術の代入 `＄＊回数＝＄＊回数＋１`（未代入の変数は算術で 0 とみなされる。[値が無いときの扱い](../../book/src/grammar/variables.md#値が無いときの扱い)） | — | `10-save.pasta` |
| 11 | 話を続けたい・分岐させたい | 話の続き・ランダムジャンプ | `＞シーン名`（[Call](../../book/src/grammar/call-jump.md)）、同名・[前方一致](../../book/src/grammar/call-jump.md#前方一致によるターゲット解決)の候補からのランダム選択、[ローカルシーン](../../book/src/grammar/block-structure.md#ローカルシーン) `・`、[`＞チェイントーク`](../../book/src/grammar/call-jump.md#チェイントーク)、ローカル優先の[スコープ解決](../../book/src/grammar/call-jump.md#スコープ解決アルゴリズム) | — | `11-jump.pasta` |
| 12 | もっと凝ったことをしたい | Lua の関数を呼ぶ | シーン内の ```` ```lua ```` ブロック（[Lua ブロックの配置](../../book/src/grammar/block-structure.md#lua-ブロックの配置)）、`function SCENE.名前(act)`、`＞＠名前（）`（[条件分岐の実現](../../book/src/grammar/call-jump.md#条件分岐の実現)の書き方） | — | `12-lua.pasta` |
| 13 | 配布したい | `.nar` にする | SSP の NAR 作成機能（6 段目で有効にした開発者用機能の「ディレクトリをドロップした際に更新ファイルや NAR を作成」を ON にしてフォルダをドロップ） | — | — |

- 辞書を持つ段階は 1〜12 段目。13 段目は手順だけの段階で、辞書の差分を持たない（`.nar` にする中身は 12 段目と同じ）。
- 12 段目の辞書（＝`dic/` の全ファイル）が hello-pasta の配布辞書である。
- 「使う文法要素」はマニュアル（`book/src/`）に記載のある書き方だけを指す。7 段目の OnGhostChanged・OnGhostChanging の応答と OnBoot・OnClose の関係はベースウェアの規則で、根拠は UKADOC にある。

## 検証イベント表

| 段階 | イベント | Reference |
|------|----------|-----------|
| 7 | `OnGhostChanged` | `0=むらさき, 2=えも？？` |
| 7 | `OnGhostChanging` | `0=むらさき, 1=manual, 2=えも？？` |
| 7 | `OnFirstBoot` | `0=0` |
| 7 | `OnClose` | `0=user` |
| 8 | `OnMouseDoubleClick` | `3=0, 4=Head` |
| 9 | `OnChoiceSelectEx` | `0=おやつの話をする, 1=おやつの話, 2=OnMouseDoubleClick` |

- 検証は全段階で `OnBoot` を送ったうえで、この表の該当行のイベントを Reference 付きで 1 回ずつ送り、エラーにならずに応答が返ることを確かめる（応答の内容は照合しない）。`OnBoot` はこの表に載せない。
- 載せるのはベースウェアから来る実イベントだけである。仮想イベント（ランダムトーク・時報）は載せない。
- Reference は半角の `番号=値` を `, ` で区切って並べる。無ければ `—` と書く。
- 8 段目の `Head` は仮の部位名である。`hello-pasta-shell-art` が当たり判定の部位名を確定したら、台詞とこの表を合わせる。

## ファイル名の規則

- `NN-name.pasta` の形にする。`NN` は 2 桁ゼロ埋めの段階番号（`01`〜`12`）、区切りは半角ハイフン、`name` は ASCII 小文字の英単語（必要ならハイフンでつなぐ）。
- pasta は `dic/*.pasta` をファイル名の辞書順に読み込むので、読み込み順と段階の順が一致する。
- ASCII に限るのは、読者の環境・`.nar`・URL でのファイル名の扱いを単純にするためである。
- 1 段 1 ファイル。ファイル名の `NN` は、段階表でそのファイルを追加する行の `段階` と一致する。

## 段階 N の辞書の組み立て方

- 段階 N の辞書は、段階 1〜N で追加されるファイルの集まりである（`dic/` のうち `NN` が N 以下のファイル）。
- 読者は前の段階のファイルに手を入れず、新しいファイルを `dic/` に足すだけで次の段階になる。1 段目の `＊OnBoot` は以後の段階で変えない。
- 段階ごとに変わるのは `dic/` の `.pasta` ファイルの集まりだけである。`pasta.toml`・`descript.txt`・`install.txt`・シェル・`scripts/` は全段階で共通。
- 段階表の「追加するファイル」の集まり（`—` を除く）は、`dic/*.pasta` の集まりと一致する。

## 表の書き方の規則

- 2 つの表は、見出し `## 段階表`・`## 検証イベント表` の直後の最初の表である。テストは見出しの行で表を探し、列見出しの名前で列を引く。列見出しの名前を変えるときは、テストと入門ガイドも合わせて直す。
- `段階` は整数で書く。
- セルには `|` を書かない（エスケープした形も含む）。列がずれて表が読めなくなる。

## brief のたたき台からの変更理由

- 要件ディスカッション（議題 5）で、1 段目の `＊OnBoot` を以後の段階で育てない方針にした。どの段階も新しいファイルを足すだけで成立させるためである。
- そのため、2〜4 段目の掛け合い・表情・毎回ちがう台詞の受け皿として、たたき台では 5 段目にあった「暇なときに話しかけてほしい」（ランダムトーク）を 2 段目に繰り上げ、2 段目「二人で掛け合いさせたい」に吸収した。2 段目では `＊会話` を「暇なときに pasta が呼ぶシーンの名前」とだけ説明し、イベント名とシーン名の対応は 7 段目で扱う。
- たたき台の 4 段目に同居していた「単語」を、5 段目「単語でちょこっと変えたい」として独立させた。段階の数は 13 のまま変わらない。
- 設定ファイルを編集する段は独立させない。`pasta.toml` は 1 段目から全段で共通の 1 ファイルで、設定の段は辞書ファイルを持たず、1 段 1 ファイルの組み立て方と検査を崩すためである。トークの間隔はランダムトークを初めて書く 2 段目で意味を持つので、2 段目の「新しく覚える表現」で `[ghost]` の 2 行を短くする書き方を示す。あわせて配布版の間隔を 45〜75 秒（平均 1 分）に縮め、2 段目の `＊会話` を数分待たずに確かめられるようにした。

## 13 段目：配布したい

`.nar` は SSP の NAR 作成機能で作る。

1. 本体設定「一般」で開発者用機能を有効にする（6 段目で済ませている）。
2. 本体設定「開発/その他」の「ディレクトリをドロップした際に更新ファイルや NAR を作成」を ON にする。
3. ゴーストのフォルダを SSP にドロップする。

手順の正本は UKADOC の SSP ヘルプである。

- [開発者向けヘルプ](https://ssp.shillest.net/ukadoc/ssphelp/dev.html)
- [設定：開発/その他](https://ssp.shillest.net/ukadoc/ssphelp/config-dev.html)

## 確かめるための道具

読者が書いた辞書を、イベントや時刻を待たずに確かめるために、SSP の開発用パレット（[UKADOC SSP ヘルプ](https://ssp.shillest.net/ukadoc/ssphelp/dev-palette.html)）を使う。ここには「どの段でどの道具を使うか」だけを書き、操作の説明は入門ガイドに任せる。

- 6 段目: 本体設定「一般」で開発者用機能を有効にする（初出。13 段目の NAR 作成もこの設定を使う）。開発用パレット（`Ctrl+Shift+D`）の「現在時刻の仮想的変更」で、正時を待たずに時報を確かめる。
- 7 段目: 開発用パレットの「スクリプト入力」に `\![raise,イベント名,Reference0,…]` を入れてイベントを起こす（`OnFirstBoot`・`OnGhostChanged` など。切り替えの相手のゴーストが手元に無くても試せる）。`OnClose` は「`\-` タグで終了しない」を ON にして繰り返し試す。
- 読者は 1 段目で初回起動を済ませているので、7 段目で足した `OnFirstBoot` は自然には呼ばれない。`profile/` を消して初期状態に戻す方法は案内しない（10 段目で保存した値も消えるため）。
