# Requirements Document

## Project Description (Input)

**誰が困っているか**: ゴーストの作者と、開発の道具（`pasta-recipe`、のちに VS Code 拡張も）。

**今の状況**: シーンの中でローカル変数（`＄名前`）の値によって分かれる話を確かめるには、その変数に値を入れる前置きのシーンを別に書くか、辞書を書き換えて鳴らすしかない。開発用のイベント `OnPastaScene`（名前で指名）・`OnPastaSceneAt`（ファイルと行で指名）は、シーンを鳴らせるが、ローカル変数に前もって値を入れる口が無い。`OnPastaCallFunction` は位置で並べた文字列の引数（`＄０`…）を渡せるだけで、ローカル変数には入らない。

**何を変えるか**: `OnPastaScene`・`OnPastaSceneAt` の今の Reference の次（Reference2）に、ローカル変数の名前と値を TOML のインライン表で 1 つ付けて送ると、そのシーンはその変数に値が入った状態で始まる。付けなければ今と同じに振る舞う。書き方の誤り（TOML として読めない・変数の名前にならない・扱えない型）は、今の `OnPastaSceneAt` の誤りと同じ形（バルーンの短い表記とログの警告）で知らせ、シーンは鳴らさない。版は 0.5.3（`pasta-recipe` の版）のウェーブ 1。この口は、のちに `pasta-recipe-dev-actions` が画面から呼ぶ。

## Introduction

開発用のイベント `OnPastaScene`・`OnPastaSceneAt` に、ローカル変数を入れた状態でシーンを始める口を足す。送る側は Reference2 に TOML のインライン表を 1 つ書く。

```text
OnPastaSceneAt  Reference0=C:\…\ghost\master\dic\02-talk.pasta  Reference1=12  Reference2={ "名前" = "ぱすた", "回数" = 3 }
OnPastaScene    Reference0=挨拶  Reference1=朝  Reference2={ "天気" = "雨" }
OnPastaScene    Reference0=挨拶  Reference1=（空）  Reference2={ "天気" = "雨" }   ← グローバルシーンだけを指名
→ シーンの最初の行の時点で ＄名前・＄回数（＄天気）に値が入っている
```

> **決定（2026-11-12 の要件ディスカッション）**: Reference2 は TOML の書き方をそのまま受ける。日本語のキーは引用符で包む（`{ "名前" = "ぱすた" }`）。引用符の無い日本語のキー（`{ 名前 = "ぱすた" }`）は、pasta の使う TOML の読み取り（toml 1.1.8）では読めないので、「表にできない値」の誤り（Requirement 4.1）とする。独自に読み替えない（pasta の書式は TOML の方針）。

## Boundary Context

- **In scope**:
  - `OnPastaScene`・`OnPastaSceneAt` の既定ハンドラが、Reference2 の TOML のインライン表を読み、検査し、シーンを始めるときにローカル変数へ入れること。
  - 誤りの 3 種（TOML のインライン表として読めない・変数の名前にならない・扱えない型）の知らせ方。
  - マニュアル（`SHIORI イベントとハンドラ` の章の `OnPastaScene`・`OnPastaSceneAt` の節と、シーンを再生するイベントの共通の決まり）と、そこから作るスキルの参照資料。
- **Out of scope**:
  - 開発の道具の画面（`pasta-recipe-dev-actions` が持つ）。
  - VS Code 拡張から Reference2 を送ること（今の拡張は Reference2 を送らないまま）。
  - `OnPastaCallFunction` の引数の形の変更、シーン引数（`＄０`・`＄１`…）に値を入れること。
  - グローバル変数（`＄＊名前`）を同じ口で入れること（`OnPastaSetSaveVariable` がある）。
  - `OnPastaSceneAt` の行からシーンを引く規則（シーンマップ）、名前でシーンを探す規則。
  - ローカル変数の名前の決まりそのもの（辞書の `＄名前` の決まりをそのまま使う）。
- **Adjacent expectations**:
  - `pasta-recipe-dev-actions`（0.5.3 のウェーブ 3）は、この要件の Reference2 の書き方どおりに文字列を組み立てて送る。日本語のキーを引用符で包むのは送る側の責め。
  - 今の MCP の利用者（VS Code 拡張の「▶ シーンを実行」、ghost_dev の MCP の使い方）は Reference2 を送らないので、振る舞いは変わらない。
  - 並走する `vscode-scene-outline-play` は `pasta_lsp`・`editors/vscode` を触る。この spec はどちらにも触らない。

## Requirements

### Requirement 1: ローカル変数を入れてシーンを始める

**Objective:** As a ゴーストの作者（と、開発の道具）, I want 指名したシーンを、ローカル変数に値を入れた状態で鳴らしたい, so that 変数の値で分かれる話を、前置きのシーンを書かずに確かめられる

#### Acceptance Criteria

1. When `OnPastaScene` か `OnPastaSceneAt` が、Reference2 に TOML のインライン表を付けて届き、指名したシーンが見つかったとき, the 既定ハンドラ shall 表のキーごとに、同じ名前のローカル変数（`＄キー`）にその値を入れた状態でシーンを始める。
2. The 既定ハンドラ shall 入れた値を、シーンの最初の行の時点から、そのシーンの本文で `＄キー` として読めるようにする。
3. The 既定ハンドラ shall 入れた値を、シーンの最初の行の前にその変数へ代入したときと同じ範囲に届ける（そのシーンから `＞` で呼んだシーン・ローカルシーン、そのシーンの Lua のブロック、そのシーンのチェイントークの続きでも同じ値が読め、後の別のイベントで始まるシーンには届かない）。
4. When シーンの本文が、Reference2 で入れたのと同じローカル変数に代入したとき, the 既定ハンドラ shall 本文の代入を後から勝たせる（Reference2 の値は、最初の値にすぎない）。
5. The 既定ハンドラ shall キーを、辞書の `＄名前` と同じく書いたとおりの名前で入れる（全角・半角、大文字・小文字を揃えない）。
6. When Reference2 が空の表（`{}`）のとき, the 既定ハンドラ shall ローカル変数を足さずに、Reference2 が無いときと同じくシーンを鳴らす。
7. When Reference2 のキーが、pasta がイベントの受け取りで先に入れるローカル変数（`＄r0`〜`＄r9`・`＄ｒ０`〜`＄ｒ９`、日時の変数、`＄req_id`・`＄req_base_id`）と同じ名前のとき, the 既定ハンドラ shall Reference2 の値を勝たせる（シーンの最初の行の前に代入したのと同じ。転記の後に入れる）。

### Requirement 2: 入れられる値の型

**Objective:** As a ゴーストの作者, I want 辞書で使う値の型をそのまま送りたい, so that 文字列・数・真偽で分かれる話を確かめられる

#### Acceptance Criteria

1. When 値が TOML の文字列のとき, the 既定ハンドラ shall その変数に文字列を入れる。
2. When 値が TOML の整数か浮動小数点数のとき, the 既定ハンドラ shall その変数に数を入れる（`3` は辞書で `3` と出て、算術にも使える）。
3. When 値が TOML の真偽値（`true`・`false`）のとき, the 既定ハンドラ shall その変数に真偽値を入れる。
4. When 値が TOML の日付・時刻のとき, the 既定ハンドラ shall その変数に、その日付・時刻を TOML で書いたとおりの文字列（`1979-05-27` など）を入れる（`@toml` の今の変換のまま）。
5. If 値が TOML の配列か表（ドットで区切ったキー `"a".b = 1` で生まれる表を含む）のとき, the 既定ハンドラ shall それを扱えない型の誤り（Requirement 4）とする（辞書の `＄名前` に使い道が無い。要る人が出てから足す）。

### Requirement 3: Reference2 が無いときは今のまま

**Objective:** As a 今の MCP の利用者（VS Code 拡張・AI エージェント）, I want Reference2 を送らないときの振る舞いが変わらないでほしい, so that 今の使い方が壊れない

#### Acceptance Criteria

1. When `OnPastaScene` か `OnPastaSceneAt` が Reference2 無しで届いたとき, the 既定ハンドラ shall 今と同じ応答とログの警告を返す（再生するシーン・失敗の表記・警告の文言を変えない）。
2. When Reference2 が空の文字列のとき, the 既定ハンドラ shall Reference2 が無いときと同じに振る舞う。
3. When `OnPastaScene` の Reference1 が無いか空の文字列で、Reference2 が付いているとき, the 既定ハンドラ shall Reference0 のグローバルシーンを指名したものとして、Reference2 のローカル変数を入れて鳴らす。
4. The 既定ハンドラ shall `OnPastaScene` の Reference1 を、中身にかかわらずローカルシーンの名前として扱い、Reference2 の代わりには読まない（Reference1 に `{ … }` が来ても、その名前のローカルシーンを探す）。

### Requirement 4: 書き方の誤りを知らせて鳴らさない

**Objective:** As a ゴーストの作者（と、開発の道具）, I want Reference2 の書き間違いの原因がすぐ分かってほしい, so that 黙って違う状態で鳴ることなく、直して送り直せる

#### Acceptance Criteria

1. If Reference2 が TOML のインライン表として読めないとき（TOML の文法の誤り、引用符の無い日本語のキー、表でない値（`3`・`"x"` など）、表の後ろに余計な文字がある）, the 既定ハンドラ shall シーンを鳴らさず、`【表にできない値：<イベント名> Reference2（"<届いた文字列>"）】` の形の失敗の表記と、Reference2 を届いたとおりに含み、読めなかった理由を添えたログの警告を 1 行出す（ラベルは今の `数にできない値`・`文字列にできない値` と同じ言い方の新設。警告の文言は今の `<イベント名>: bad argument: ReferenceN='…'` の形に揃え、細部は設計で決める）。
2. If Reference2 のキーが、辞書でローカル変数として `＄名前` と書ける名前でないとき（空の文字列、数字だけ（`"1"`・`"１"`。`＄１` はシーン引数）、`__名前__` の形の予約された名前、空白や `-`・`.` などの記号を含む名前）, the 既定ハンドラ shall シーンを鳴らさず、`【名前にならない値：<イベント名> Reference2（"<キー>"）】` の形の失敗の表記と、そのキーを含むログの警告を 1 行出す（ラベルは今の `名前にならない値`）。
3. If Reference2 の値が扱えない型（Requirement 2.5）のとき, the 既定ハンドラ shall シーンを鳴らさず、`【代入できない値：<イベント名> Reference2 ＄<キー>（表）】` の形の失敗の表記と、キーと型を含むログの警告を 1 行出す（ラベルは今の `代入できない値`）。
4. If Reference2 に誤りがあるとき, the 既定ハンドラ shall ローカル変数を 1 つも入れず、チェイントークで待っている続きに触れない（今の、見つからないときや引数の誤りと同じ）。
5. If Reference2 に誤りが複数あるとき, the 既定ハンドラ shall 失敗の表記とログの警告を 1 件だけ出す。
6. The 既定ハンドラ shall 誤りを、Reference0 → Reference1 → Reference2 → シーンの検索 の順に確かめ、最初に見つけた 1 件を知らせる（Reference2 が誤っていてシーンも無いときは、Reference2 の誤りを知らせる。シーンを引く前に、届いた文字列だけで済む検査を済ませる）。
7. While `[logging]` の `balloon_warning = false` のとき, the 既定ハンドラ shall 今の失敗と同じく、表記を出さずに 204 No Content を返し、ログの警告は出す。
8. The 既定ハンドラ shall 失敗の表記の中の、さくらスクリプトとして働く文字（`\`・`%`）を、今の失敗の表記と同じく無効にする。

### Requirement 5: 使える条件は今の 2 つのイベントと同じ

**Objective:** As a ゴーストの配布者, I want ローカル変数を入れる口が、開発のときにだけ開いていてほしい, so that 配布したゴーストが外から操られない

#### Acceptance Criteria

1. While デバッグが無効のとき, the 既定ハンドラ shall Reference2 を読まず、ローカル変数を入れず、今と同じ警告を 1 行出して 204 No Content を返す。
2. While リクエストの `SecurityLevel` が `external` のとき, the 既定ハンドラ shall Reference2 を読まず、ローカル変数を入れず、今と同じ警告を 1 行出して 204 No Content を返す。
3. Where ゴーストが `OnPastaScene`・`OnPastaSceneAt` のハンドラを先に登録しているとき, the 既定ハンドラ shall 今と同じく、そのハンドラを上書きしない。

### Requirement 6: マニュアルとスキルの参照資料

**Objective:** As a ゴーストの作者と AI エージェント, I want Reference2 の書き方と誤りの出方をマニュアルで引きたい, so that MCP の `raise_event` から正しく送れる

#### Acceptance Criteria

1. The マニュアル shall `OnPastaScene`・`OnPastaSceneAt` の節の Reference の表に Reference2（ローカル変数のインライン表。省略できる）を載せ、少なくとも 1 つの送り方の例と、そのシーンで値が読めることを示す。
2. The マニュアル shall 2 つの節の誤りの表に、Requirement 4 の 3 種の誤りを、表記とログの警告の実例付きで載せる。
3. The マニュアル shall シーンを再生するイベントの共通の決まりに、値の型（Requirement 2）・届く範囲（Requirement 1.3〜1.4）・`OnPastaScene` で Reference1 を空にしてグローバルシーンだけを指名する書き方（Requirement 3.3）・日本語のキーは引用符で包む書き方を載せる。
4. When マニュアルを書き換えたとき, the スキルの参照資料 shall マニュアルから作り直され、マニュアルとの照合（`gen-skill-refs --check`）に通る。

### Requirement 7: 動作環境と互換性

**Objective:** As a pasta の開発者, I want 今の動作環境と今のテストを保ちたい, so that 0.5.3 の他の spec と安全に並べられる

#### Acceptance Criteria

1. The 既定ハンドラ shall LuaJIT 2.1 の上で動く。
2. The pasta.dll shall Reference2 を送らない今の `OnPastaScene`・`OnPastaSceneAt` のテストを、書き換えずに通す。
3. The 既定ハンドラ shall 入れた値が読めること・Reference2 が無いと今のままであること・誤りの 3 種を、自動のテストで確かめられるようにする。
