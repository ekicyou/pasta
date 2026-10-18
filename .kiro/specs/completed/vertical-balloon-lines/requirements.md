# Requirements Document

## Project Description (Input)
窓際のぱすたさん（ghost_dev）は、SSP の縦書きのバルーン（`vertical,1`）を使う。縦書きでは改行が「次の列」になる。右の枠（台詞）から左の枠の 1 列目の頭へは `\_l[-295,0]` で飛ぶ。困っていることは 2 つある。1 つ目は budoux の改行の幅で、明示的な改行は `\n[180]` で出しているのに、budoux が入れる改行は普通の `\n` なので、折り返した列だけ間隔が詰まる。2 つ目は budoux の数え方で、budoux は自分で入れた改行しか知らず、台詞の中の `\n`・`\n[…]`・`\c`・`\_l[…]` で列が替わっても、前の列の字数を引き継いで数え続ける。作者が書いた `\n`・`\c` の後も同じで、横書きのゴーストでも起きている今の不具合である。ゴースト側は今、budoux を止めて SSP の字数による折り返しに任せているので、語の途中で折り返す（例「空｜調」）。budoux が入れる改行の幅を倍率で指定でき、列を動かすタグの所で budoux が数え直し、選択肢の後の改行の幅も倍率で指定できるようにする。方式は「budoux が台詞の中のタグを見て数え直す」（2026-10-18 開発者の承認）。既定の設定のままなら、今の出力を変えない（数え直しの不具合の直しを除く）。詳細は `brief.md` を参照。

## Introduction

窓際のぱすたさんの縦書きのバルーンでは、1 列の字数と列の間隔が決まっている（ゴースト側の実測。2026-10-18・暫定）。

- 字送り 20px、列の間隔 36px（`\n[180]`）、1 列の上限は 20 字。
- 右の枠は 3 列で、各 20 字。左の枠（`\_l` で飛んだ先）は 4 列で、1 列目が 20 字、2〜4 列目が 14 字。
- 字数で書くと「20,20,20 →（`\_l`）→ 20,14,14,14 → 超えたら `\c`」。budoux の幅は全角を 2 と数えるので、その倍になる。

書きたいのは、次の形である。

```toml
[ghost]
spot_newlines = 1.8

[actor."ぱすた"]
spot = 0
budoux = [40]
budoux_newlines = 1.8   # → \n[180]。既定は 1.0（今のまま \n）
```

```pasta
＊ポエム_出来損ないのパスタ
　ぱすた：長い長い、記憶の糸。生まれてから、続いてる一本の長い記憶。＠左
　：そう、きっと、一本道。いつか来る終わりの日まで。
```

```text
長い長い、記憶の糸。生まれてから、\n[180]続いてる一本の長い記憶。\_l[-295,0]そう、きっと、一本道。いつか来る終わりの\n[180]日まで。
```

（`\_l` の所で、列の頭から数え直す。どこで折り返すかは budoux の語の区切りで決まるので、この位置は例である。ウェイトのタグは省いた。）

今の動き（2026-10-18 にコードを読んで確かめた。根拠は `research.md`）:

| 改行の出どころ | 今、数え直すか |
| -------------- | -------------- |
| 台詞の中に書いた `\n`・`\n[180]`・`\c`・`\_l[…]` | しない（不具合） |
| 単語・関数が返して台詞につながった `\n[180]` など | しない（同上） |
| pasta が台詞のトークンの境目に出す区切り（`spot_newlines`・`choice_newlines`） | 結果として、する（ただし、幅の並びは先頭に戻る） |
| budoux が自分で入れる改行 | する |
| 台詞の途中に表情などのトークンを挟んだとき | まだ同じ列なのに、数え直す（逆向きの粗さ） |

本書の読者は 3 者いる。「ゴースト作者」は pasta.toml で budoux を設定する人（窓際のぱすたさんを含む）、「既存のゴースト作者」は budoux を使わない、または今の設定のまま使っている人、「開発者」は pasta を直す人である。

### 決めたこと（brief で承認済み）

- **方式は、budoux が台詞の中のタグを見て数え直す。** `act:budoux_reset()` のような明示の手段は採らない（2026-10-18 開発者の承認）。
- **`budoux_newlines` は、アクター設定（`budoux` の隣）に書く。** 値は倍率で、既定は 1.0（今のまま `\n`）。
- **作者が書いたタグと、単語や関数が返したタグを同じに扱う。**
- **選択肢の後の改行の幅は、`[ghost]` のキーにする。** 既定は 1.0。汎用の機能で、ぱすたは使わない。
- **縦書きのバルーンそのもの・左の枠へ飛ぶ単語（`＠左`）は、ゴースト側が持つ。**

### 要件ディスカッションで決めたこと（2026-10-18 開発者の承認・全議題を推奨案で確定）

brief が開いたままにした論点は、次のとおり決めた。各要件の【A】〜【G】の印が、その決定に当たる。理由と退けた案は `research.md` の「要件ディスカッションへの申し送り」にある。

- **キーの名前**: `budoux_newlines`・`budoux_jump`（アクター設定）、`after_choice_newlines`（`[ghost]`）。
- **値の検査**: 数値でない値は、今の `budoux`・`choice_newlines` と同じ扱いにする。`pasta_check validate` には検査を足さない。
- **【A】数え直すタグ**: `\n`・`\n[…]`（`\n[half]` を含む）・引数の無い `\c`・`\_l[…]` の 4 種。`\x`・`\x[noclear]`・`\_q`・引数つきの `\c[…]`・スコープの切り替え（`\0`・`\1`・`\p[…]` など）では数え直さない。`\x` を外すのは、pasta の選択肢の配置が「引数の無い `\c` だけを吹き出しの消去に数える」と決めていて（`grammar/block-structure.md`）、規則を 1 つにそろえるためである。
- **【B】`\_l` の後の幅**: アクター設定に、飛んだ後に使う別の幅の並び `budoux_jump` を書けるようにする。書かなければ、`\_l` の所で字数だけを 0 に戻し、列の番号は変えない。
- **【C】トークンをまたいだ字数**: 1 回の出力の中では、吹き出しごとに、列の番号・字数・どちらの幅の並びかを持ち越す。pasta が出す区切り（段落・選択肢の前後・`act:newline()`）は次の列へ、`act:clear()` は最初の列へ進める。出力をまたいでは持ち越さない。
- **【D】選択肢の後の改行の幅**: キーの名前は `after_choice_newlines`。選択肢の後の選択肢の前と、選択肢の後の台詞の前の両方に効かせる。
- **【E】倍率 1.0 の出力**: `budoux_newlines`・`after_choice_newlines` が 1.0 のときは、`\n[100]` ではなく今と同じ `\n` を出す。
- **【F】出力の変化の扱い**: 数え直しの直しで、budoux を使うゴーストの出力が変わることを、不具合の直しとして受け入れる。今の動きに戻す切り替えは設けない。変わることはリリースノートとマニュアルに書く。
- **【G】`break_lines(text, widths)`**: 数え直しは同じに効かせる。倍率と飛んだ後の幅の並びは足さない（今のまま `\n` を入れる）。

## Boundary Context

- **In scope**:
  - budoux が入れる改行の幅（アクター設定の `budoux_newlines`）
  - 台詞の中の列を動かすタグの所での budoux の数え直し（作者が書いたもの・単語や関数が返したもの）
  - `\_l` で飛んだ後の幅（【B】）
  - 1 回の出力の中の、トークンをまたいだ字数の持ち越し（【C】）
  - 選択肢の後の改行の幅（`[ghost]` のキー。【D】）
  - 今の出力を変えない範囲と、変わる場所の記録
  - テスト、マニュアル（`reference/pasta-toml.md`・`lua/modules/pasta-sakura-script.md`・`internals/talk-output.md` と、`[ghost]` のキーの一覧を載せる章・選択肢の配置の表を載せる章）、スキル references の再生成
- **Out of scope**:
  - budoux の禁則（バックログ「budoux の自動改行の禁則」。JIS X 4051 の方針のまま）
  - budoux の語の区切り方（分割のモデル）と、1 語で幅を超える行の扱い
  - 列が足りなくなったときに、budoux が `\c` などで自動で吹き出しを替えること（ゴースト側の `＠改ページ` で書く）
  - アクターごとの表示の装い（`actor-display-style`）
  - 縦書きのバルーンそのもの（ゴースト側の descript）、左の枠へ飛ぶ単語（ゴースト側の `＠左`）
  - SSP の自動折り返し（バルーンの `validrect` などで SSP が折り返す分）、ルビ・縦中横
  - 台詞の中に書いたスコープの切り替え（`\0`・`\1`・`\p[…]`・`\h`・`\u`）・バルーンの切り替え（`\b[…]`）の後の数え方（【A】）
  - `pasta_check validate` での新しいキーの検査（今の `budoux`・`spot_newlines` も検査していないのに合わせる）
- **Adjacent expectations**:
  - ベースウェア（SSP）は、`\n[N]` を「行の高さの N% の改行」として扱い、縦書きのバルーンでは改行を次の列として描く。本 spec はこの動きを前提にし、変えない
  - `actor-display-style` は、出力の組み立て（`sakura_builder.lua`）とマニュアルの `reference/pasta-toml.md` を共有する。【C】を採ったので、実装は 1 本ずつ入れる（ロードマップの約束）
  - スキル references は、後から入る側が再生成する

## Requirements

### Requirement 1: budoux が入れる改行の幅

**Objective:** As a ゴースト作者, I want budoux が入れる改行の幅を倍率で指定したい, so that 縦書きのバルーンで、折り返した列の間隔を、話者の切り替えの列の間隔（`\n[180]`）にそろえられる

#### Acceptance Criteria
1. Where アクター設定に `budoux` と `budoux_newlines` が書かれている, the pasta shall そのアクターの台詞に budoux が入れる改行を `\n[N]` にする。`N` は値を 100 倍して小数点以下を切り捨てた整数とする（`spot_newlines` と同じ換算）。
2. Where `budoux_newlines` が書かれていない, the pasta shall budoux が入れる改行を今と同じ `\n` にする。
3. Where `budoux_newlines` の値が 1.0 である, the pasta shall budoux が入れる改行を `\n` にする（`\n[100]` にしない。【E】）。
4. Where アクター設定に `budoux` が無い、または空の配列である, the pasta shall `budoux_newlines` があっても、そのアクターの台詞に改行を入れない。
5. The pasta shall `budoux_newlines` を、budoux が入れる改行だけに効かせる。作者が台詞に書いた改行・単語や関数が返した改行・pasta が出す段落や選択肢の区切りの幅は変えない。
6. If `budoux_newlines` の値が数値でない, the pasta shall `budoux` の配列に整数にできない要素があるときと同じに、その台詞の変換を失敗させる。
7. The pasta shall `budoux_newlines` を、アクター設定の他のキーと同じに Lua から `@pasta_config` の `actor` で読めるようにする。

### Requirement 2: 列を動かすタグの所での数え直し

**Objective:** As a ゴースト作者, I want 台詞の中で列を動かすタグの所から budoux が数え直してほしい, so that 作者が書いた改行や、左の枠へ飛んだ後で、早すぎる（遅すぎる）改行が入らない

（「列」は、横書きのバルーンでは「行」に当たる。「幅の並び」は、アクター設定の `budoux = [1 列目の幅, 2 列目以降…]` を指す。）

#### Acceptance Criteria
1. When budoux を使うアクターの台詞の中に `\n` または `\n[…]` がある, the pasta shall そのタグの後ろの字を、次の列の頭から字数 0 で数え、幅の並びの次の値を使う（最後の値は、それ以降の列で繰り返す。`\_l` で飛んだ後の並びは Requirement 3）。
2. When budoux を使うアクターの台詞の中に引数の無い `\c` がある, the pasta shall そのタグの後ろの字を、最初の列の頭から字数 0 で数え、幅の並びの先頭の値を使う。
3. When budoux を使うアクターの台詞の中に `\_l[…]` がある, the pasta shall そのタグの後ろの字を、字数 0 から数える（使う幅は Requirement 3）。
4. The pasta shall 1〜3 を、作者が台詞に書いたタグ・単語や関数が返して台詞につながったタグ・`act:sakura_script` で積んだテキストの中のタグに、同じに当てる。
5. When 列を動かすタグの直後に語が続く, the pasta shall その語の前に改行を入れない（列の頭に空の列を作らない）。
6. When 列を動かすタグが、budoux が 1 語とした区切りの途中にある, the pasta shall タグの前と後ろを別の列として数え、タグの後ろの部分を新しい列の頭として扱う。
7. The pasta shall 列を動かすタグの前の字を、今と同じ規則（全角を 2・半角を 1 と数え、タグは数えない）で数え、その列の幅を超える語の前に改行を入れる。
8. The pasta shall `\x`・`\x[noclear]`・`\_q`・引数つきの `\c[…]`・ウェイト・表情などの列を動かさないタグでは、数え直さない（【A】）。
9. The pasta shall エスケープした `\\n` と、タグをそのまま表示する囲み（`\_?…\_?`・`\_!…\_!`）の中の `\n`・`\c`・`\_l` を、列を動かすタグとして読まない。

### Requirement 3: `\_l` で飛んだ後の幅

**Objective:** As a ゴースト作者, I want `\_l` で飛んだ先の枠の列の幅を、別の並びで書きたい, so that 左の枠の「20,14,14,14」のように、右の枠と違う列の並びを budoux に守らせられる

#### Acceptance Criteria
1. Where アクター設定に `budoux_jump`（【B】）が書かれている, when 台詞の中に `\_l[…]` がある, the pasta shall そのタグの後ろの列に `budoux_jump` の先頭の値を使い、続く `\n`・`\n[…]` のたびに次の値を使う（最後の値は、それ以降の列で繰り返す）。
2. Where `budoux_jump` が書かれていない、または空の配列である, when 台詞の中に `\_l[…]` がある, the pasta shall 列の番号を変えずに、その列の字数を 0 に戻す（幅の並びは `budoux` のまま、同じ値を使う）。
3. While `budoux_jump` の並びを使っている, when 引数の無い `\c` がある, the pasta shall `budoux` の並びの先頭へ戻る。
4. While `budoux_jump` の並びを使っている, when もう一度 `\_l[…]` がある, the pasta shall `budoux_jump` の並びの先頭から使い直す。
5. The pasta shall `budoux_jump` の値の書き方（半角 1 文字を 1、全角 1 文字を 2 と数える整数の配列）と、整数にできない要素の扱いを、`budoux` と同じにする。
6. Where アクター設定に `budoux` が無い、または空の配列である, the pasta shall `budoux_jump` があっても、そのアクターの台詞に改行を入れない。
7. The pasta shall `budoux = [40]`・`budoux_jump = [40, 28]` の設定で、Introduction の実測（右の枠は 20 字ずつ、`\_l` の後の 1 列目は 20 字、2 列目以降は 14 字）を越えない位置に改行を入れる（1 語で幅を超える列を除く）。

### Requirement 4: トークンをまたいだ字数の持ち越し

**Objective:** As a ゴースト作者, I want 台詞の途中に表情などを挟んでも、同じ列の字数を budoux が引き継いでほしい, so that 表情を替えたところから数え直して列をはみ出し、SSP が語の途中で折り返すことがない

（【C】の決定に基づく。「1 回の出力」は、pasta がベースウェアへ返すさくらスクリプト 1 本を指す。）

#### Acceptance Criteria
1. While 1 回の出力の中で、同じ吹き出しに続けて台詞を出している, when 台詞と台詞の間に、字を表示しない指定（表情の切り替え・ウェイト・`!select` の制限時間）だけがある, the pasta shall 後ろの台詞を、前の台詞の終わりの列・字数・幅の並びを引き継いで数える。
2. When pasta が台詞と台詞の間に区切り（話し手の切り替えの段落の区切り・選択肢の前と後の区切り・`act:newline()` の改行）を出す, the pasta shall 区切りの改行 1 つにつき列を 1 つ進め、次の台詞を新しい列の頭から字数 0 で数える。
3. When `act:clear()` で吹き出しを消す、または `act:clear_spot()` で立ち位置を消す, the pasta shall 次の台詞を最初の列の頭から、`budoux` の並びの先頭の値で数える。
4. The pasta shall 列と字数を、吹き出し（スポット）ごとに別々に持ち、話し手が別の吹き出しへ移って戻ってきたときは、戻った吹き出しの列と字数を引き継ぐ。
5. When 1 回の出力が始まる, the pasta shall どの吹き出しも、最初の列の頭から字数 0 で数える（前の出力から持ち越さない）。
6. When 引き継いだ字数のままでは、後ろの台詞の最初の語が列に収まらない, the pasta shall その語の前に budoux の改行を入れる。
7. While 引き継いだ列の字数が 0 である, the pasta shall 後ろの台詞の最初の語の前に改行を入れない。
8. The pasta shall 選択肢の表示の文字を、budoux の字数に数えない。

### Requirement 5: 選択肢の後の改行の幅

**Objective:** As a ゴースト作者, I want 選択肢の後の改行の幅を倍率で指定したい, so that pasta が自分で出す改行（段落の区切り・選択肢の前・budoux の折り返し・選択肢の後）が、すべて倍率で調整できる

#### Acceptance Criteria
1. Where `pasta.toml` の `[ghost]` に `after_choice_newlines`（【D】）が書かれている, when 同じ吹き出しで選択肢の後に選択肢を出す, the pasta shall 2 つ目の選択肢の直前の区切りを `\n[N]` にする。`N` は値を 100 倍して小数点以下を切り捨てた整数とする。
2. Where `after_choice_newlines` が書かれている, when 同じ吹き出しで選択肢の後に台詞を出す, the pasta shall 台詞の直前の区切りを、1 と同じ `\n[N]` にする（【D】）。
3. Where `after_choice_newlines` が書かれていない、または値が 1.0 である, the pasta shall 1・2 の区切りを今と同じ `\n` にする（【E】）。
4. When 話し手が切り替わって、選択肢を最後に出した吹き出しへ戻り、台詞を出す, the pasta shall 今と同じく `spot_newlines` の区切りだけを出し、`after_choice_newlines` の区切りを重ねない。
5. The pasta shall `after_choice_newlines` の値の換算と、数値でない値の扱いを、`choice_newlines` と同じにする。
6. Where `pasta.toml` に `after_choice_newlines` が書かれていない, the pasta shall `[ghost]` の他のキーと同じに既定値（1.0）を補い、Lua から `@pasta_config` で既定値が読めるようにする。
7. The pasta shall 字の後の選択肢の前の区切り（`choice_newlines`）を変えない。

### Requirement 6: 変えない範囲と、出力が変わる場所

**Objective:** As a 既存のゴースト作者, I want 新しいキーを書かない限り、budoux の不具合の直しのほかは出力が今のままであってほしい, so that pasta を更新しても、台詞の見え方が思わぬ所で変わらない

#### Acceptance Criteria
1. Where アクター設定に `budoux` を書いていないゴーストである, the pasta shall どの台詞の出力も今と同じにする（`after_choice_newlines` を書いた場合の選択肢の後の区切りと、pasta が出すタグ（区切り・`act:newline()`・`act:clear()`・話し手の切り替え・終わりの `\e` など）の直前の出力が単独の `\` で終わるときのエスケープ（決定 9）を除く）。
2. Where `budoux` を使い、`budoux_newlines`・`budoux_jump`・`after_choice_newlines` を書いていないゴーストである, the pasta shall 列を動かすタグを含まず、pasta の区切りや字を表示しない指定を間に挟まない台詞の出力を、今と同じにする。
3. Where `budoux` を使うゴーストである, the pasta shall Requirement 2・3・4 の数え直しで出力が変わることを、不具合の直しとして扱い、今の数え方に戻す設定を設けない（【F】）。
4. The pasta shall budoux の語の区切り方、1 語だけで幅を超える列の扱い（無理に割らない）、出力の先頭の語の前に改行を入れないことを、今と同じにする。
5. The pasta shall `@pasta_sakura_script` の `talk_to_script(actor, talk)` を、今と同じ 2 つの引数で呼べるようにし、その呼び方では最初の列の頭から数える。
6. The pasta shall `break_lines(text, widths)` に Requirement 2 の数え直しを同じに当て、改行には今と同じ `\n` を入れる（【G】）。
7. The pasta shall hello-pasta と、既存の期待値ファイル（スナップショット・期待値の文字列）が、budoux を設定していない辞書について、そのまま通ることを保つ。
8. The 本 spec の記録 shall 出力が変わる場所（列を動かすタグの後・表情などを挟んだ台詞・pasta の区切りの後で、幅の並びが先頭に戻らなくなること）を、リリースノートとマニュアルに書く。

### Requirement 7: テスト・マニュアル・スキル references

**Objective:** As a 開発者, I want 新しい動きがテストとマニュアルにそろって入る, so that 後から直す人が、数え方の規則を取り違えずに保てる

#### Acceptance Criteria
1. The pasta shall Requirement 1〜5 の各受け入れ条件を確かめる自動テストを持つ（各タグでの数え直し・飛んだ後の幅の並び・トークンをまたいだ持ち越し・倍率の換算と既定値・選択肢の後の区切り）。
2. The pasta shall 既定の設定（新しいキーを書かない）で出力が変わらないことを確かめる自動テストを持つ。
3. The pasta shall Introduction の作例（`budoux = [40]`・`budoux_newlines = 1.8`・`＠左` が返す `\_l[-295,0]`）を、出力の文字列で照合する自動テストを持つ。
4. The マニュアル shall `reference/pasta-toml.md` に、`budoux_newlines`・`budoux_jump`・`after_choice_newlines` のキー・型・既定値・換算・例を載せ、`[ghost]` の既定値の表とテンプレートを、設定の既定値と照合するテストが通る形にする。
5. The マニュアル shall `lua/modules/pasta-sakura-script.md` に、列を動かすタグでの数え直しの規則と、`break_lines` の動きを載せる。
6. The マニュアル shall `internals/talk-output.md` の「budoux の行の幅は呼び出しごとに 1 行目から数え直す」「テキスト内の `\n` でも数え直さない」という今の記述を、新しい数え方に書き換える。
7. The マニュアル shall `[ghost]` のキーの一覧を載せる他の章（`grammar/block-structure.md` の選択肢の配置の表・`lua/modules/pasta-config.md`・`internals/loader.md`・`internals/internal-modules.md`）を、`after_choice_newlines` に合わせる。
8. The pasta shall スキル references を、マニュアルの章から再生成し、鮮度の照合（`gen-skill-refs.mjs --check`）とリンク検証が通る状態にする。
9. The 本 spec shall 窓際のぱすたさんの実機（SSP 2.9.18・縦書きのバルーン）で、作例が右の枠 3 列・左の枠 4 列の中に収まって表示されることを、ゴースト側に確かめてもらえる形で記録する（実機の確認はゴースト側が行う）。
