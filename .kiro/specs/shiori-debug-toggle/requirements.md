# Requirements Document

## Project Description (Input)

ゴーストの作者（開発者）が、マニュアルの「デバッグの有効化」を読んで、手元でデバッグを入れたり切ったりしたい。今の口は 2 つだけで、どちらも手間がかかる。`pasta.toml` の `[debug] enabled = true` は配る設定ファイルを書き換えることになり、戻し忘れるとデバッグを入れたまま配ってしまう。環境変数 `PASTA_DEBUG` は、ベースウェアを起動し直さないと効かない。

VSCode 拡張と pasta-recipe は「SHIORI をリロード」を呼べるが、そのときにデバッグを入れる手段がない。デバッグが無効なゴーストにシーンの再生などを頼むと、「pasta.toml に [debug] enabled = true を書いてリロードしてください」と案内され、作者がファイルを手で書き換えに行く。

そこで、保存変数 `＄＊デバッグ`（英語名 `＄＊debug`）に真偽値を入れておくと次の読み込みから有効・無効が決まるようにし、VSCode 拡張と pasta-recipe に、保存変数を書いてリロードし、効いたかを出すトグルを足す。無効なときの案内はトグルを指し、マニュアルの「デバッグの有効化」に保存変数とトグルの使い方を載せる。詳しい経緯と決定は `brief.md` を参照。

## Introduction

この spec は、ゴーストの作者が配る設定ファイルに触れずに、手元でデバッグを入れたり切ったりできるようにする。

```text
VSCode「SHIORI デバッグを有効にする」／pasta-recipe のトグルのボタン
  → ＄＊デバッグ ＝ true（真偽値）を書く
  → SHIORI をリロード
  → 読み込みで ＄＊デバッグ を見て、デバッグが有効になる
  → 結果に「デバッグは有効です」と出る
```

有効かどうかの優先の順は、次の 4 段になる。

```text
PASTA_DEBUG ＞ ＄＊デバッグ／＄＊debug ＞ pasta.toml の [debug] enabled ＞ 無効
```

`＄＊デバッグ` と `＄＊debug` の組み合わせごとの決まり方は、次のとおり。

| `＄＊デバッグ` | `＄＊debug` | この段の結果 |
| -------------- | ----------- | ------------ |
| `true` | （何でも） | 有効 |
| `false` | （何でも） | 無効 |
| 無い・真偽値でない | `true` | 有効 |
| 無い・真偽値でない | `false` | 無効 |
| 無い・真偽値でない | 無い・真偽値でない | 書いていないのと同じ（`pasta.toml` の段へ） |

主な利用者は、VSCode 拡張か pasta-recipe でゴーストを書き、▶ や保存変数の表示を使いたいゴーストの作者（英語圏の作者を含む）である。

## Boundary Context

- **In scope**
  - 読み込みのときに `＄＊デバッグ` と `＄＊debug` で有効かどうかを決めること、優先の順を 4 段にすること
  - 真偽値でない値の扱いと警告
  - VSCode 拡張と pasta-recipe で共通のトグルの流れ（保存変数を書く → リロード → 状態を出す）
  - VSCode 拡張の 2 つのコマンドと、pasta-recipe のトグルのボタン
  - デバッグが無効なときの案内の文
  - マニュアル（デバッグの部と、入門のトグルに触れる箇所）と、スキルの `references/` の作り直し
  - 自動テスト
- **Out of scope**
  - `PASTA_DEBUG` と `[debug] enabled` の読み方の変更
  - 動いている最中にデバッグを入れたり切ったりすること（読み込みをせずに切り替えること）
  - 保存変数の名前を `pasta.toml` で変える口
  - DSL に真偽値の書き方を足すこと
  - グローバル変数の保存の仕組み（ファイルの書式・場所・保存する時期）の変更
  - デバッグのイベント（`OnPastaSceneAt`・`OnPastaDumpVariables` など）の中身
- **Adjacent expectations**
  - **外から保存変数を書くイベント**（完了 spec `external-control-events`）: `OnPastaSetSaveVariable` を今のまま使う。デバッグが無効でも効き、`boolean` の型で真偽値を書ける、という今の振る舞いを前提にする。振る舞いは変えない。
  - **グローバル変数の保存**: SHIORI を下ろすときにグローバル変数が保存され、次の読み込みで戻る、という今の振る舞いを前提にする。保存先はゴーストの `profile/` で、配布物には入らない。
  - **リロードと読み込みの結果の要約**（完了 spec `vscode-mcp-client`・`pasta-recipe-dev-actions`）: リロードの頼み方、読み込みの終わりの待ち方、誤り・警告の要約は、今の「SHIORI をリロード」と同じものを使う。
  - **巻き戻し**（未完了 spec `novel-checkpoint-rollback`）: ロードと巻き戻しで戻るのは名前が「スロット」か `slot_` で始まるグローバル変数だけなので、`＄＊デバッグ`・`＄＊debug` は戻らない側に入る。この spec はそれを変えない。

## Requirements

### Requirement 1: 保存変数による有効・無効の決定

**Objective:** As a ゴーストの作者, I want 保存変数に真偽値を入れておくだけでデバッグを入れたり切ったりしたい, so that 配る `pasta.toml` を書き換えずに手元で ▶ を使える

#### Acceptance Criteria

1. When SHIORI を読み込み、`PASTA_DEBUG` が設定されていない（または読めない値である）とき、and `＄＊デバッグ` が真偽値の `true` である, the pasta shall `pasta.toml` の `[debug] enabled` の値にかかわらずデバッグを有効にする。
2. When SHIORI を読み込み、`PASTA_DEBUG` が設定されていないとき、and `＄＊デバッグ` が真偽値の `false` である, the pasta shall `pasta.toml` で `[debug] enabled = true` と書いてあってもデバッグを無効にする。
3. When SHIORI を読み込み、`PASTA_DEBUG` が設定されていないとき、and `＄＊デバッグ` が無いか真偽値でなく、`＄＊debug` が真偽値である, the pasta shall `＄＊debug` の値で有効・無効を決める。
4. When `＄＊デバッグ` と `＄＊debug` がどちらも真偽値で、値が食い違う, the pasta shall `＄＊デバッグ` の値で決める。
5. When SHIORI を読み込み、`PASTA_DEBUG` が設定されていないとき、and `＄＊デバッグ` と `＄＊debug` のどちらも無いか真偽値でない, the pasta shall 今までどおり `pasta.toml` の `[debug] enabled` で決め、それも無ければ無効にする。
6. When SHIORI を読み込み、`PASTA_DEBUG` が読める値で設定されている, the pasta shall `PASTA_DEBUG` の値で決め、保存変数と `pasta.toml` の値には左右されない。
7. The pasta shall 優先の順を `PASTA_DEBUG` ＞ `＄＊デバッグ`／`＄＊debug` ＞ `pasta.toml` の `[debug] enabled` ＞ 無効、の 4 段とする。
8. The pasta shall 判断に `＄＊デバッグ` と `＄＊debug` の 2 つの名前だけを見て、ほかの名前（`Debug`・`DEBUG`・`dbg` など）は見ない。
9. The pasta shall 判断に、シーンから `＄＊デバッグ`（Lua では `SAVE.デバッグ`）として見える値と同じもの、つまり前に SHIORI を下ろしたときに保存されて読み込みで戻る値を使う。

### Requirement 2: 真偽値でない値の扱い

**Objective:** As a ゴーストの作者, I want 真偽値でない値を入れてしまったときに気づきたい, so that `"true"` という文字列などを入れて効かない理由が分からないまま困らない

#### Acceptance Criteria

1. If 判断に使う順で見た `＄＊デバッグ` に真偽値でない値（`"true"` という文字列・数の `1` など）が入っている, the pasta shall その値を書いていないのと同じに扱い、`＄＊デバッグ` という名前と、真偽値でないため読まなかったことを示す警告を pasta.log に 1 行出す。〔仮定: 「判断に使う順で見た」＝ `PASTA_DEBUG` で決まったときは保存変数を見ないので警告しない、`＄＊デバッグ` が真偽値で決まったときは `＄＊debug` を見ないので警告しない。OPEN QUESTION 1〕
2. If 判断に使う順で見た `＄＊debug` に真偽値でない値が入っている, the pasta shall その値を書いていないのと同じに扱い、`＄＊debug` という名前を示す警告を pasta.log に 1 行出す。
3. While 変数が無い（値が入っていない）, the pasta shall 警告を出さない。
4. The pasta shall 真偽値でない値があっても、SHIORI の読み込みを失敗にしない。

### Requirement 3: 判断の時期と、無効のときの変わらなさ

**Objective:** As a ゴーストの作者, I want デバッグの有効・無効が読み込みのときに 1 回だけ決まってほしい, so that `pasta.toml` を書き換えたときと同じ感覚で扱え、無効のゴーストの動きが変わらない

#### Acceptance Criteria

1. The pasta shall 有効かどうかを、SHIORI の読み込みのたびに 1 回だけ決める。
2. When シーンや Lua の中で `＄＊デバッグ` か `＄＊debug` を書き換えた, the pasta shall そのセッションの有効・無効を変えず、次の読み込みから新しい値で決める。
3. When 読み込みで有効と決まった, the pasta shall 今と同じ `Debug enabled: development events are allowed` の 1 行を pasta.log に出す（どの段で有効になったかにかかわらず同じ文）。
4. While 有効かどうかが無効と決まっている, the pasta shall この spec の前と同じく、開発用のシーンの対応表を作らず、辞書から作る `.lua` の中身もこの spec の前と変えない。
5. The pasta shall `＄＊デバッグ`・`＄＊debug` の値を書き換えず、シーンからは今までどおりほかのグローバル変数と同じに読み書きでき、同じ仕組みで保存される。

### Requirement 4: 安全の範囲

**Objective:** As a ゴーストの作者, I want 保存変数で入れたデバッグが、ほかの口で入れたデバッグと同じ働きだけを持ってほしい, so that 新しい口のせいで外へ開くものが増えない

#### Acceptance Criteria

1. When 保存変数でデバッグが有効になった, the pasta shall `PASTA_DEBUG` か `pasta.toml` で有効にしたときと同じ働き（開発用のイベントと、127.0.0.1 だけで受ける待ち受け）だけを有効にする。
2. The pasta shall `OnPastaSetSaveVariable` の振る舞い（いつでも効くこと、型と応答）を変えない。

### Requirement 5: 道具のトグルの共通の流れ

**Objective:** As a ゴーストの作者, I want エディタから 1 回の操作でデバッグを入れたり切ったりし、効いたかを見たい, so that ファイルを手で書き換えに行かなくて済む

#### Acceptance Criteria

1. When 作者が「SHIORI デバッグを有効にする」を選んだ, the 開発の道具 shall `＄＊デバッグ` に真偽値の `true` を書き、続けて SHIORI をリロードし、読み込みの後の有効・無効を結果に出す。
2. When 作者が「SHIORI デバッグを無効にする」を選んだ, the 開発の道具 shall `＄＊デバッグ` に真偽値の `false` を書き（変数を消すのではない）、続けて SHIORI をリロードし、読み込みの後の有効・無効を結果に出す。
3. The 開発の道具 shall 書くのを `＄＊デバッグ` だけにし、`＄＊debug` は書きも消しもしない。
4. When リロードの後の読み込みが終わり、その読み込みで `Debug enabled: development events are allowed` の行が出た, the 開発の道具 shall デバッグが有効になったと結果に出す。
5. When リロードの後の読み込みが終わり、その読み込みで `Debug enabled …` の行が出なかった, the 開発の道具 shall デバッグが無効になったと結果に出す。
6. If 読み込みの後の状態が頼んだ状態と違う, the 開発の道具 shall 効かなかったことと、より優先の環境変数 `PASTA_DEBUG` で決まっている見込みがあることを、知らせとして出す。〔仮定: OPEN QUESTION 4〕
7. If 保存変数の書き込みをゴーストが断ったか、頼めなかった, the 開発の道具 shall リロードをせず、今の「保存変数の書き換え」と同じ理由の文を出す。
8. If リロードを頼めなかったか、読み込みの終わりを確かめられなかったか、読み込みが失敗した, the 開発の道具 shall 今の「SHIORI をリロード」と同じ要約を出し、有効・無効は確かめられなかったと出す。
9. The 開発の道具 shall 結果に、今の「SHIORI をリロード」と同じ読み込みの要約（誤り・警告。Requirement 2 の警告を含む）も出す。
10. When トグルを選んだ, the 開発の道具 shall リロードの前に、今の「SHIORI をリロード」と同じく編集中の未保存の辞書を保存する。〔仮定: OPEN QUESTION 7〕
11. The 開発の道具 shall VSCode 拡張と pasta-recipe で、同じ流れと同じ結果の文を使う。

### Requirement 6: VSCode 拡張のコマンド

**Objective:** As a VSCode でゴーストを書く作者, I want コマンドとしてデバッグを入れたり切ったりしたい, so that ほかの開発のコマンドと同じ手で使える

#### Acceptance Criteria

1. The VSCode 拡張 shall コマンドパレットの「Pasta」の分類に、「SHIORI デバッグを有効にする」と「SHIORI デバッグを無効にする」の 2 つのコマンドを出す。〔仮定: 右クリックのメニューには足さない。OPEN QUESTION 6〕
2. When どちらかのコマンドを実行した, the VSCode 拡張 shall 結果を、ほかの開発のコマンドと同じ出力の欄に、同じ見出しの形で出す。
3. While 別の開発の操作が進んでいる, the VSCode 拡張 shall ほかの開発のコマンドと同じく、操作中であることを理由に断る。

### Requirement 7: pasta-recipe のトグルのボタン

**Objective:** As a pasta-recipe でゴーストを書く作者, I want リロードの隣のボタン 1 つでデバッグを入れたり切ったりしたい, so that 今の状態と切り替えを 1 か所で扱える

#### Acceptance Criteria

1. The pasta-recipe shall 「SHIORI をリロード」の隣に、デバッグのトグルのボタンを 1 つ置く。
2. While 最後に確かめた状態が有効である, the pasta-recipe shall ボタンを「SHIORI デバッグを無効にする」として示し、押すと Requirement 5 の無効にする流れを行う。
3. While 最後に確かめた状態が無効であるか、まだ確かめていない, the pasta-recipe shall ボタンを「SHIORI デバッグを有効にする」として示し、押すと Requirement 5 の有効にする流れを行う。〔仮定: OPEN QUESTION 3〕
4. When トグルか「SHIORI をリロード」で読み込みの後の状態を確かめた, the pasta-recipe shall ボタンの示し方を、確かめた状態に合わせて改める。
5. When 別のゴーストを開いた, the pasta-recipe shall 状態を「まだ確かめていない」に戻す。〔仮定: OPEN QUESTION 3〕
6. When ボタンを押した, the pasta-recipe shall 結果を、ほかの開発の操作と同じ出力の欄に出す。

### Requirement 8: デバッグが無効なときの案内

**Objective:** As a ゴーストの作者, I want デバッグが無効で断られたときに、すぐ使える操作を案内されたい, so that ファイルを書き換えに行かなくて済む

#### Acceptance Criteria

1. When デバッグが無効なゴーストにシーンの再生か保存変数の表示を頼んで断られた, the 開発の道具 shall `pasta.toml` の書き換えでなく、「SHIORI デバッグを有効にする」の操作を案内する。
2. The 開発の道具 shall 案内の文を VSCode 拡張と pasta-recipe で同じにし、ゴーストが返したさくらスクリプトを今のエディタの文言と同じく「スクリプト」と呼ぶ。

### Requirement 9: マニュアルとスキルの参照

**Objective:** As a マニュアルを読むゴーストの作者, I want 保存変数とトグルによる有効化をマニュアルで知りたい, so that 書いてあるとおりに手元でデバッグを入れたり切ったりできる

#### Acceptance Criteria

1. The マニュアル shall 「デバッグの有効化」に、保存変数による有効化（`＄＊デバッグ`・`＄＊debug`、真偽値の `true`・`false` だけを数えること、次の読み込みから効くこと、入れる口として pasta-recipe の変数の表・`OnPastaSetSaveVariable` の `boolean`・Lua の `SAVE.デバッグ = true`）を載せる。
2. The マニュアル shall 「デバッグの有効化」に、道具のトグル（VSCode 拡張の 2 つのコマンドと、pasta-recipe のボタン）の使い方を載せる。
3. The マニュアル shall 優先の順を 4 段（`PASTA_DEBUG` ＞ `＄＊デバッグ`／`＄＊debug` ＞ `[debug] enabled` ＞ 無効）に直す。
4. The マニュアル shall 「`debug` という名前は pasta のものとして読む」ことを 1 行書く。
5. The マニュアル shall 保存変数はゴーストの `profile/` に保存されて配布物に入らないこと、一度入れると消すまで次の起動からも効き続けること、`pasta.toml` の設定へ戻すには変数を消せばよいことを書く。
6. The マニュアル shall トグルに触れる章（デバッグの部の「▶・リロード・保存変数」「pasta-recipe で書く」「うまく動かないとき」、入門の 1 段目の「うまくいかないときは」と 14 段目の「デバッグを切る」）を、トグルと 4 段の優先の順に合わせて直す。〔仮定: OPEN QUESTION 5〕
7. When マニュアルを直した, the マニュアルの作り手 shall 同じ変更で、スキルの `references/` を作り直す。

### Requirement 10: 自動テスト

**Objective:** As a pasta の開発者, I want 新しい決まりがテストで守られていてほしい, so that 後の変更で優先の順や名前の決まりが崩れたら気づける

#### Acceptance Criteria

1. The pasta の自動テスト shall `＄＊デバッグ` と `＄＊debug` のそれぞれが `true`・`false`・無い・真偽値でない、の組み合わせを表で確かめ、`PASTA_DEBUG` と `[debug] enabled` との優先の順も確かめる。
2. The pasta の自動テスト shall 開発機の環境変数 `PASTA_DEBUG` と、テストを置いた場所に左右されずに通る。
3. The 開発の道具の自動テスト shall トグルの流れ（有効にする・無効にする・書き込みを断られた・リロードできない・頼んだ状態と違う）と、pasta-recipe のボタンの示し方の移り変わりを確かめる。
