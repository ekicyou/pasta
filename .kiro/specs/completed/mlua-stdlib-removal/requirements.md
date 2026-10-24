# Requirements Document

## Project Description (Input)
pasta は Rust のライブラリ mlua-stdlib（0.1.0）の 6 つのモジュール（`@json`・`@yaml`・`@regex`・`@assertions`・`@testing`・`@env`）を Lua に組み込んでいるが、窓際のぱすたさん・emo2・hello-pasta のどれも 6 つを使っていない。mlua-stdlib は `mlua = "^0.11"` の 0.1.0 しか無く、pasta が mlua 0.12 へ上がるのを止めている。pasta 自身は `OnPastaSetSaveVariable` の値を読むためだけに `@json` を使っており、既定のシリアライズを JSON にしたのは開発者に確かめずに決めた不当な決定だった。作者が任意の TOML の文字列を読む部品も無い。

この spec では mlua-stdlib を丸ごと外し、TOML の文字列を Lua の表に読む `@toml`（`decode` だけ。既定で有効）を足し、`OnPastaSetSaveVariable` を JSON の値でなく Reference2 の型（`number`・`string`・`boolean`・`delete`。省いたら `string`）で受ける形に改める。マニュアルとスキルを今の姿に直す（詳しくは brief.md）。

## Introduction

この spec は、作者に見える Lua の部品を入れ替え、外から変数を書き込むイベントの値の受け方を改める。

- mlua-stdlib の 6 つのモジュール（`@json`・`@yaml`・`@regex`・`@assertions`・`@testing`・`@env`）は無くなる。どのゴーストも使っていない。文字列を調べるのは Lua の標準の `string.match`・`find`・`gsub` で足りる。
- 新しく `@toml` を持つ。TOML の文字列を Lua の表に読む `decode` だけを持つ。
- `OnPastaSetSaveVariable` は、Reference1 を JSON として読むのをやめる。Reference1 は届いた文字列をそのまま値に使い、型は Reference2 で受ける。

```lua
local toml = require "@toml"

local t = toml.decode('name = "ぱすた"\nlevel = 3\ntags = ["a", "b"]')
-- t.name == "ぱすた"、t.level == 3、t.tags[1] == "a"

local bad, err = toml.decode("name = ")
-- bad == nil、err は読めなかった理由の文字列
```

```
raise_event(event = "OnPastaSetSaveVariable", references = ["pasta_talk_interval_min", "60", "number"])
  → ＄＊pasta_talk_interval_min ＝ 60（数値）
raise_event(event = "OnPastaSetSaveVariable", references = ["会員番号", "01"])
  → ＄＊会員番号 ＝ "01"（型を省いたので文字列。引用符で包まない）
raise_event(event = "OnPastaSetSaveVariable", references = ["初回済み", "true", "boolean"])
  → ＄＊初回済み ＝ true
raise_event(event = "OnPastaSetSaveVariable", references = ["会員番号", "", "delete"])
  → ＄＊会員番号 を消す
```

0.4.2 で出した `OnPastaSetSaveVariable` の形（JSON の値）とは互換が無い。作者に見える部品を消す変更とあわせて、リリースノートの「動きが変わる変更」で知らせる。

主な利用者は 3 人である。

- **ゴーストの作者**: `scripts/` などの Lua で、組み込みのモジュールを `require` して使う人。
- **外から試験・操作する人**: SSP の MCP の `raise_event` などで `OnPastaSetSaveVariable` を送る人（AI エージェントを含む）。
- **pasta の開発者**: 依存を管理し、mlua 0.12 へ上げる次の spec（`mlua-0-12-upgrade`）を始めたい人。

## Boundary Context

- **In scope**
  - mlua-stdlib への依存と、6 つのモジュールの撤去（Rust の側の設定のモジュール名・`env` の警告・誤りの文を含む）
  - `@toml`（`decode` だけ）
  - `OnPastaSetSaveVariable` の値と型の受け方と、その失敗の出方
  - 自動テスト（消えたモジュールを使っていた既存のテストの書き直しを含む）
  - マニュアル、スキル `pasta-lua-coding`（`SKILL.md` と、マニュアルから作る references）、`crates/pasta_lua/README.md`、`tech.md`
- **Out of scope**
  - mlua 0.12 への更新（`mlua-0-12-upgrade`。この spec の後）
  - JSON・YAML・正規表現・テストの道具の代わりの部品（作者の要望が出たら起票する）
  - `@toml` の `encode`（Lua の値を TOML の文字列にする。要る作者が出てから）
  - `OnPastaCallFunction`（JSON を使っていない。今のまま）
  - セーブファイル・`@pasta_log` の表の出力・ソースマップのサイドカーの JSON（`toml-serialization`）
  - DAP・LSP の JSON（プロトコルが決めている）
  - 0.4.2 の JSON の形の `OnPastaSetSaveVariable` を受け続けること
  - リリースノートのファイルそのもの（`release` スキルが書く。この spec はコミットの件名の `!` と申し送りで知らせる）
- **Adjacent expectations**
  - **`external-control-events`**（完了）: `OnPastaSetSaveVariable` の名前の扱い・`SecurityLevel` の門・作者のハンドラでの止め方・応答・保存の時機は、あちらで決めたまま受け継ぐ。この spec が変えるのは、値と型の受け方と、それに伴う失敗だけである。
  - **`toml-serialization`**（同じウェーブ）: セーブ・ログの表・サイドカーの TOML 化はあちらが持つ。この spec は `@toml` の読む側だけを持つ。`pasta.toml` の撤去済みの `[lua] libs` に `env` を書いても `@env` が使えないことを確かめる既存のテストは、モジュールが無くなっても同じ結果になるので、そのまま通る（手を付けない）。
  - **`recommend-menu-scenes`**（同じウェーブ）: マニュアルの SHIORI イベントの章の既定ハンドラーの表を共有する。この spec は `OnPastaSetSaveVariable` の行だけを直し、後から入る側が合わせる。失敗の種類の数は、この spec では変えない。
  - **`mlua-0-12-upgrade`**（後）: この spec で mlua-stdlib が無くなれば、上流の公開を待たずに始められる。

## Requirements

### Requirement 1: mlua-stdlib の 6 つのモジュールを無くす

**Objective:** As a pasta の開発者, I want pasta が mlua-stdlib に依存しないこと, so that 使われていない部品のために mlua 0.12 への更新が止まらない。

#### Acceptance Criteria

1. The pasta shall `@json`・`@yaml`・`@regex`・`@assertions`・`@testing`・`@env` のどのモジュールも、どのゴーストの Lua にも登録しない（`pcall(require, "@json")` は `false` を返す）。
2. The pasta shall mlua-stdlib に依存せず、mlua-stdlib だけが使っていた依存も持たない（pasta の依存の一覧と、配布物に添える第三者のライセンスの一覧に mlua-stdlib が出ない）。
3. The pasta shall pasta 自身の Lua のスクリプト（`pasta_scripts`）から、6 つのモジュールを読まない。
4. The pasta shall Lua の標準ライブラリの構成（既定で使える `string`・`table`・`math`・`io`・`os`・`package`・`coroutine`・`bit`・`jit`、既定で使えない `debug`・`ffi`）を変えない。
5. Where Rust から `pasta_lua` を使う人が VM に入れるライブラリの名前を並べるとき, the pasta shall 6 つのモジュールの名前を、選べるライブラリとして扱わない（どの名前を並べても 6 つのモジュールは入らない）。
6. If Rust から並べたライブラリの名前に、知らない `std_` の名前があるとき, the pasta shall 今と同じく誤りを返し、その誤りの文に、選べるライブラリとして 6 つのモジュールの名前を並べない。
7. The pasta shall `env` のモジュールを有効にしたときの警告を出さない（警告する相手のモジュールが無い）。`std_debug`・`std_all_unsafe` の警告は今のまま出す。

### Requirement 2: TOML の文字列を読む `@toml`

**Objective:** As a ゴーストの作者, I want TOML で書いた文字列を Lua の表に読む部品, so that pasta の既定の書式の TOML で書いたデータを、自分で読み取り係を書かずに使える。

#### Acceptance Criteria

1. The pasta shall `@toml` を、設定なしで、ほかの組み込みのモジュールと同じ時機から（ランタイムの初期化の時、`main.lua` を読む前から）、ゴーストの Lua で `require "@toml"` できるようにする。
2. When 作者が `toml.decode(文字列)` に TOML として正しい文字列を渡したとき, the `@toml` shall その中身を Lua の表にして返す。
3. The `@toml` shall TOML の値を、`@pasta_config` が pasta.toml の値を表にするときと同じ規則で Lua の値にする（表はキーの文字列の表、配列は 1 から始まる並びの表、文字列は文字列、整数と小数は数値、真偽値は真偽値、日付と時刻は RFC 3339 の形に整えた文字列。`1979-05-27 07:32:00` は `1979-05-27T07:32:00` になる）。
4. If `toml.decode` に渡した文字列が TOML として読めないとき, the `@toml` shall エラーを投げずに `nil` と、読めなかった理由を述べたエラーメッセージの文字列の 2 つを返す。
5. The `@toml` shall `decode` だけを持ち、Lua の値を TOML の文字列にする関数（`encode`）を持たない（`toml.encode` は `nil`）。
6. The `@toml` shall 何度 `require` しても同じ表を返し、`decode` の呼び出しが前の呼び出しの結果に左右されない。
7. If `toml.decode` に文字列でない値を渡したとき, the `@toml` shall 公開モジュールの共通の決まり（引数の型が合わない呼び出しは Lua のエラー）に従い、Lua のエラーにする。

### Requirement 3: `OnPastaSetSaveVariable` で、型を Reference2 で受けて代入する

**Objective:** As a 外から試験・操作する人, I want 値をそのままの文字列で送り、型は別の Reference で指定すること, so that MCP から引用符を二重に包まずに文字列を送れ、数値・真偽値・消去も迷わず指定できる。

#### Acceptance Criteria

1. When `OnPastaSetSaveVariable` を受け取り、Reference0 に名前があり、Reference2 が無いか空の文字列のとき, the pasta shall Reference1 に届いた文字列を、手を加えずに文字列のまま、その名前のグローバル変数（`＄＊名前`）に代入する（空の Reference2 は型を省いたのと同じ。送る道具が Reference の席を空で埋めても文字列になる）。
2. When Reference2 が `string` のとき, the pasta shall Reference2 が無いときと同じく、Reference1 の文字列をそのまま代入する。
3. The pasta shall 型が文字列のとき、Reference1 を何の書式としても読まない（`01` は文字列の `01`、`"01"` は引用符を含めた文字列、`60` は文字列の `60`、`null` は文字列の `null`、空の文字列は空の文字列になる）。
4. When Reference2 が `number` で、Reference1 が、実行中に届いた値をほかの所で数にするときと同じ範囲（DSL の算術や時計の Reference と同じく Lua の標準の数の読み方。`60`・`-1.5`・前後の空白・`0x10`・`1e3` を含む）で数として読め、無限大・非数にならないとき, the pasta shall Reference1 を数値にして代入する。
5. When Reference2 が `boolean` で、Reference1 が `true` か `false` のとき, the pasta shall それぞれ真偽値の `true`・`false` を代入する。
6. When Reference2 が `delete` のとき, the pasta shall Reference1 の中身（無い・空・どんな文字列でも）を見ずに、その名前のグローバル変数を消し、値の無い状態に戻す（辞書から `＄＊名前` を読むと、値を代入していない変数と同じになる）。
7. The pasta shall 型の名前を、`number`・`string`・`boolean`・`delete` の英小文字の完全一致だけで見分ける（`Number`・`数値` は知らない型の名前になる。空の文字列は 3.1 のとおり型を省いたのと同じ）。
8. When 代入したとき・消したとき, the pasta shall 204 No Content を返す。
9. The pasta shall 名前の扱い（区切らない・予約グローバル変数にも代入できる）、代入した値がその後の `＄＊名前` にすぐ見えること、保存を終了時の自動保存に任せることを、`external-control-events` で決めたとおりに保つ。

### Requirement 4: `OnPastaSetSaveVariable` の誤りの断り方

**Objective:** As a 外から試験・操作する人, I want 型に合わない値や知らない型を送ったとき、何が悪かったかが分かること, so that 送り方を直せて、ゴーストの変数が意図しない値で壊れない。

#### Acceptance Criteria

1. If Reference2 が `number` で、Reference1 が数として読めない（`abc`・空の文字列・無い）か、無限大・非数になる（`inf`・`nan`）とき, the pasta shall 変数を変えずに前の値を残し、種類「数にできない値」の失敗の表記とログの警告を出す。
2. If Reference2 が `boolean` で、Reference1 が `true`・`false` のどちらでもない（`TRUE`・`1`・空の文字列・無い）とき, the pasta shall 変数を変えずに前の値を残し、種類「代入できない値」の失敗の表記とログの警告を出す。
3. If Reference2 が空でなく、`number`・`string`・`boolean`・`delete` のどれでもないとき, the pasta shall 変数を変えずに前の値を残し、種類「型の名前にならない値」で、届いた型の名前を含めた失敗の表記とログの警告を出す。
4. If Reference2 が無い・空・`string` のどれかで、Reference1 が無いとき, the pasta shall 変数を変えずに前の値を残し、種類「代入できない値」の失敗の表記とログの警告を出す（変数を消すのは `delete` を送ったときだけである）。
5. The pasta shall 失敗の表記に、イベント名と変数の名前（`OnPastaSetSaveVariable ＄＊会員番号`）と、届いた値を引用符で包まずにそのまま（値が無ければ「値なし」、空なら「空文字列」）入れ、ログの警告に、イベント名・変数の名前・誤った Reference とその値を書く。
6. The pasta shall 失敗の表記の出方（デバッグの有無によらず出る、`balloon_warning = false` では 204 No Content）を、ほかの実行時の失敗と同じにする。
7. The pasta shall `OnPastaSetSaveVariable` の失敗の種類の名前に「JSON」を含めず、今の「JSON として読めない値」を「型の名前にならない値」に置き換え、「数にできない値」「代入できない値」は今ある種類を使って、失敗の種類の数（12）を増やさない。
8. The pasta shall Reference0 が無い・空のとき、`SecurityLevel` が `external` のときの断り方を、今のまま変えない。

### Requirement 5: マニュアル・スキル・文書を今の姿にする

**Objective:** As a ゴーストの作者, I want マニュアルとスキルが今ある部品とイベントの形を書いていること, so that 無くなったモジュールや古い JSON の送り方を手本にして迷わない。

#### Acceptance Criteria

1. The マニュアル shall mlua-stdlib の章を `@toml` の章に置き換え（目次の章の数は変えない）、`@toml` の章で、読み込み方、`decode` の使い方と値の対応、失敗の返し方、`encode` を持たないことを説明する。
2. The マニュアル shall 公開モジュールの一覧に `@toml` を載せ、6 つのモジュールが既定で有効・無効だという記述をどの頁からも無くす（Lua の章の入口・パターン集・DSL と Lua の使い分け・`@pasta_log` の章・内部の説明の頁を含む）。
3. The マニュアル shall mlua-stdlib の章にあった Lua の標準ライブラリの構成（既定で使えるもの・使えないもの・ゴーストから変えられないこと）の説明を残し、その説明へのリンクがすべて行き先に届くようにする。
4. The マニュアル shall SHIORI イベントの章の `OnPastaSetSaveVariable` の節で、Reference0〜2 の中身、型の名前ごとの代入される値、型を省いたときは文字列になること、失敗の出方の表、MCP からの呼び方の例（数値・型を省いた文字列・消す）を説明し、トークの間隔の変数には `number` を付けて送ることを書く。既定ハンドラーの表の `OnPastaSetSaveVariable` の行も、JSON を読まない説明に直す。
5. The マニュアル shall 作者が JSON・YAML・正規表現の部品を探したときに、それが無いこと（文字列は Lua の標準の文字列の関数で調べる）が分かるようにする。
6. The スキル `pasta-lua-coding` shall `SKILL.md` と、マニュアルから作る references で、6 つのモジュールの代わりに `@toml` を載せ、`OnPastaSetSaveVariable` の新しい形を載せる。消えた章の写しの reference は残さない。
7. The `crates/pasta_lua/README.md` と `tech.md` shall mlua-stdlib を依存・登録するモジュールとして書かない。
8. When マニュアルとスキルを直したとき, the マニュアルの検査 shall 章の数を決め打ちする検査・リンクの検査・マニュアルから references を作る道具の照合（最新でない写しと孤立した写しが無いこと）を、すべて通す。

### Requirement 6: 今の動きを壊さない

**Objective:** As a ゴーストの作者, I want 6 つのモジュールと `OnPastaSetSaveVariable` の値の受け方のほかは何も変わらないこと, so that 今のゴーストがそのまま動く。

#### Acceptance Criteria

1. The pasta shall 6 つのモジュールと `OnPastaSetSaveVariable` の値と型の受け方のほかの、Lua の組み込みのモジュール（`@pasta_search`・`@pasta_persistence`・`@pasta_config`・`@pasta_sakura_script`・`@enc`・`@pasta_log` など）と、すべてのイベントの挙動を変えない。
2. The pasta shall `OnPastaCallFunction` の挙動を変えない。
3. While 6 つのモジュールを使っていないゴースト（hello-pasta を含む）を読み込んでいるとき, the pasta shall この spec の前と同じに読み込み、同じに動かす。
4. The pasta shall `@toml` の `decode` の成功と失敗、`OnPastaSetSaveVariable` の型ごとの代入と消去、各失敗を、自動テストで確かめる。
5. The pasta shall 6 つのモジュールを使って確かめていた既存のテストを、そのモジュールを使わない形に書き直すか、確かめる相手が無くなったものは消し、残るテストをすべて通す。
6. The pasta shall 作者に見える部品を消す変更と、`OnPastaSetSaveVariable` の形を変える変更を、リリースノートの「動きが変わる変更」に載る形（コミットの件名の `!`）で残す。
