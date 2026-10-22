# 辞書を検査する（pasta_check validate）

> 【したり顔】辞書の書き間違い探しは、SSP を起動する前に済ませてしまうのが賢いやり方ですわ。コマンド 1 つで、ゴーストの辞書をまるごと調べ上げますのよ。

> 【アンソニー】起動して黙り込まれてから首をひねるより、ずっと早いということでございますね。

> 【にっこり】ええ。何がおかしいのか、どのファイルの何行目なのかまで、きちんと教えてもらえますの。読み方と直し方を、わたくしと一緒に押さえてまいりましょう。

---

このページでは、ゴーストの辞書（`.pasta`）と `pasta.toml` を SSP を起動せずに検査する `pasta_check validate` の使い方と、出てくる報告の意味・直し方を扱う。同じ検査は VSCode 拡張にも入っていて、辞書を保存するたびに結果がエディタの診断として出る（[エディタの診断](#エディタの診断)）。

検査が受け持つのは次の 2 つである。

- ゴーストを読み込んだときに `pasta.dll` が失敗する書き方（構文エラー・`pasta.toml` の誤りなど）を先に見つける。
- 読み込みは通るが、作者の意図とずれやすい書き方（名前の重なり・飛び先の無い参照）を知らせる。

## 使い方

`pasta_check` は、GitHub の Release から `pasta_check.zip` を落として入れる。最新版は次の固定のリンクで落とせる。

<https://github.com/ekicyou/pasta/releases/latest/download/pasta_check.zip>

落とした zip を好きなフォルダに展開する。中身は `pasta_check.exe` と `THIRD_PARTY_LICENSES.txt` で、`pasta_check.exe` は 64 ビット版の Windows 向けである。`pasta_check.exe` には署名が無いため、初めて起動するときに Windows の SmartScreen の警告が出ることがある。

展開した `pasta_check.exe` は、パスで指定して呼べる。たとえば `C:\tools\pasta_check` に展開したなら次のとおり。

```powershell
C:\tools\pasta_check\pasta_check.exe validate --target ghosts\hello-pasta
```

Rust を使っている人は、`cargo install pasta_check` でも入れられる。

```sh
cargo install pasta_check
```

ここから先の例は `pasta_check` と書く。zip から入れたときは、展開した `pasta_check.exe` のパスに読み替える。

ゴーストの開発フォルダ（`ghost/master` を含むフォルダ）を `--target` に渡す。

```sh
pasta_check validate --target ghosts/hello-pasta
```

- `--target` には、`ghost/master` そのものではなく、`ghost/master` を含むフォルダを渡す。`ghost/master` を渡すと「`ghost/master` がありません」の誤りで終わる。
- 検査はファイルを読むだけで、ファイルを作らない・書き換えない・消さない。
- `pasta.toml` と辞書の場所は、`ghost/master` を基準に読む。`ghost/master` の最上位の `profile` フォルダは見ない。
- SSP の起動も、ネットワークへの接続も要らない。

見本のゴースト hello-pasta を検査すると、問題が無いので、まとめの行だけが出る。

```text
検査した辞書ファイル 12 個: 誤り 0 件、注意 0 件、飛び先を検査しなかった Call 1 件
```

## 出力の読み方

見つけた問題は 1 件につき 1 行で、標準出力に出る。

```text
ghost/master/dic/a-b.pasta: 誤り: モジュール名 pasta.scene.a_b が ghost/master/dic/a_b.pasta と同じです。どちらかの辞書の内容が失われます
ghost/master/dic/a_b.pasta: 誤り: モジュール名 pasta.scene.a_b が ghost/master/dic/a-b.pasta と同じです。どちらかの辞書の内容が失われます
ghost/master/dic/c.pasta:2:3: 誤り: アクターの無い継続行です。前の行にアクター行が必要です
ghost/master/dic/d.pasta:2:4: 誤り: 構文エラー: expected id
ghost/master/pasta.toml:5: 誤り: [scene.alias] の別名「会話」が OnBoot・OnTalk に重複しています
ghost/master/pasta.toml:6: 誤り: [scene.alias] の別名「会話」が OnBoot・OnTalk に重複しています
構文エラーと pasta.toml の誤りがあるため、飛び先の検査をしていません。
検査した辞書ファイル 4 個: 誤り 6 件、注意 0 件
```

- 1 行の形は `パス:行:列: 誤り|注意: 理由` である。パスはゴーストの開発フォルダからの相対パスで、`ghost/master/` から始まる。行と列は、取れないときは省く（ファイル全体・`pasta.toml` 全体についての報告）。列は 1 文字を 1 と数える（全角文字も 1）。
- 構文エラーの行と列は、構文が読めなくなった位置である。理由の `expected …` は、その位置に来られる書き方である。
- 並び順はファイルのパスの順、同じファイルの中では行・列の順で、何度走らせても同じ順になる。
- 最後の行は、検査した辞書ファイルの数・誤りの数・注意の数をまとめた行である。最後の項「飛び先を検査しなかった Call」は、読むだけでは飛び先が決まらない Call の数である（[読むだけでは決まらない範囲](#読むだけでは決まらない範囲)）。
- 次のどれかが 1 つでもあると、飛び先の検査をまるごと省く。検査した辞書のシーンが、`pasta.dll` が読み込むシーンと同じとは言えないためである。
  - 構文エラー（`syntax`）
  - 読めない辞書ファイル（`file-not-utf8`・`file-unreadable`）
  - `pasta.toml` の誤り（`config-syntax`・`config-loader-type`・`config-scene-alias`・`config-pattern-invalid`、`pasta.toml` の `file-unreadable`）。`pasta.toml` が無いこと（`config-missing`）と、装いの誤り（`config-actor-font`）は入らない
- 省いたときは、まとめの行の前に、理由を並べた 1 行を出す（「構文エラーがあるため、飛び先の検査をしていません。」「読めない辞書ファイルと pasta.toml の誤りがあるため、飛び先の検査をしていません。」など）。このときまとめの行に最後の項は出ない。

## 誤りと注意

報告は「誤り」と「注意」の 2 種類に分かれる。どの報告がどちらになるかは報告の種類ごとに決まっていて、オプションや状況では変わらない。

| 分類 | 何を報告するか | 終了コード |
| ---- | -------------- | ---------- |
| 誤り | `pasta.dll` がゴーストの読み込みに失敗するもの。または、`pasta.dll` が知らせずに辞書の内容を失うもの | 1 件でもあれば失敗（1） |
| 注意 | 決まった動きで読み込まれるが、書いた人の意図とずれやすいもの | 成功（0）のまま |

注意は、意図どおりならそのままでよい。たとえば Call の飛び先を Lua で定義しているなら、「Call の飛び先が辞書のシーンにありません」は直さなくてよい。

## 検査の項目

検査は次の順に進む。

1. **`pasta.toml`**: TOML として読めるか、`[loader]` のキーの型、`[scene.alias]` の別名表、`[actor."名前"]` の `font`（装い）を確かめる（[pasta.toml リファレンス](../reference/pasta-toml.md)）。
2. **辞書の集め方**: `[loader]` の [`pasta_patterns`](../reference/pasta-toml.md#pasta_patterns) に一致する `.pasta` を辞書ファイルとして集める。`pasta_patterns` の `.pasta` を `.lua` に替えたパターンに一致する `.lua` を辞書モジュールとして集める。集め方は `pasta.dll` と同じである。`pasta.toml` が無いときも、既定の `dic/**/*.pasta` で集めて検査を続ける。
3. **ファイル**: 辞書ファイルが UTF-8 として読めるか、使えないファイル名でないか、辞書のパターンに一致するフォルダが無いかを確かめる。
4. **モジュール名の重なり**: 辞書ファイルと辞書モジュールのファイル名から作られるモジュール名が重ならないかを確かめる。
5. **構文**: 辞書ファイルを 1 つずつ構文として読む。構文としては読めても Lua への変換で失敗する 3 つの書き方も、ここで見つける。
6. **シーン名の重なり**: [照合用の名前](../grammar/call-jump.md#シーン名の照合)に揃えると同じになるシーン名と、組み込みの飛び先と同じ名前のシーンを見つける。
7. **飛び先**: 静的な飛び先の Call（`＞名前`・`＞「名前」`）と選択肢行のジャンプ先（`＠？名前`）が、辞書のシーンか組み込みの飛び先に見つかるかを確かめる。探し方は実行時の検索（[スコープ解決アルゴリズム](../grammar/call-jump.md#スコープ解決アルゴリズム)）と同じで、別名表（[シーン名の別名](../grammar/call-jump.md#シーン名の別名)）による置き換えと前方一致も同じように効く。

## 報告の意味と直し方

診断の code は、VSCode の問題パネルに出る報告の種類の名前である（[エディタの診断](#エディタの診断)）。コマンドの出力には code は出ず、理由の文で見分ける。

### pasta.toml と辞書の集め方

| code | 分類 | 意味 | 直し方 |
| ---- | ---- | ---- | ------ |
| `config-missing` | 誤り | `ghost/master` に `pasta.toml` が無い。`pasta.dll` は `pasta.toml` が無いとゴーストを読み込めない | `ghost/master` に `pasta.toml` を置く |
| `config-syntax` | 誤り | `pasta.toml` が TOML として読めない。理由に TOML の説明が続き、行が付く | 示された行の TOML の書き方を直す |
| `config-loader-type` | 誤り | `[loader]` が表でない、または `pasta_patterns`・`lua_search_paths`（文字列の配列）・`transpiled_output_dir`（文字列）の型が違う | 示されたキーを決まった型で書く |
| `config-scene-alias` | 誤り | `[scene.alias]` の別名表が正しくない。同じ別名が 2 つ以上の名前にある（重複）、空の名前、別名を別名で置き換える（連鎖）、型の違いのどれか | 理由に示された別名・名前を直す。重複は関わる行のすべてに出る |
| `config-actor-font` | 誤り | `[actor."名前"]` の `font`（[装い](../reference/pasta-toml.md#装いfont)）が正しくない。12 個に無い属性名、値が文字列・整数・真偽値でない、空の文字列、値に `]` か `\` が含まれる、`font` が表でない、のどれか。`pasta.dll` はゴーストを読み込めない。行は属性の値の行（表でないときは `font` の値の行） | 理由に示された属性と値を直す |
| `config-pattern-invalid` | 誤り | `pasta_patterns` の要素がパターンとして読めない | パターンの書き方を直す |
| `pattern-ignored` | 注意 | `pasta_patterns` の要素が `..`・ドライブ名・先頭の `/` を含むため使われない。`pasta.dll` もこのパターンを使わない | `ghost/master` の中を指すパターンにする |
| `pattern-not-pasta` | 注意 | `pasta_patterns` の要素が `.pasta` で終わらないため、辞書モジュール（`.lua`）を探すのに使われない | `.pasta` で終わるパターンにする |
| `no-dictionary-files` | 注意 | `pasta_patterns` に一致する辞書ファイルが 1 つも無い | パターンと辞書ファイルの置き場所を合わせる |

`pasta.toml` についての報告と、パターン・辞書の数についての報告は、`ghost/master/pasta.toml` に付く（`pasta.toml` が無いときも同じパス）。

### ファイル

| code | 分類 | 意味 | 直し方 |
| ---- | ---- | ---- | ------ |
| `init-file-name` | 誤り | 辞書ファイル・辞書モジュールの名前が `init.pasta`・`init.lua` である。`pasta.dll` はゴーストを読み込めない | ファイルの名前を変える |
| `file-not-utf8` | 誤り | ファイルが UTF-8 として読めない | UTF-8 で保存し直す |
| `file-unreadable` | 誤り | ファイルが読めない。理由に OS の説明が続く | ファイルのアクセス権や、ほかのプログラムが開いていないかを確かめる |
| `folder-matches-pattern` | 誤り | フォルダの名前が辞書のパターン（`pasta_patterns` と、そこから作る `.lua` のパターン）に一致している（`dic/x.pasta/` など）。`pasta.dll` はこのフォルダを辞書として読もうとして失敗し、ゴーストを読み込めない。中身の無いフォルダも同じ。フォルダの中のファイルは、いつもどおり辞書として集めて検査する | フォルダの名前を変える、または `pasta_patterns` を直す |

### 構文と Lua への変換

| code | 分類 | 意味 | 直し方 |
| ---- | ---- | ---- | ------ |
| `syntax` | 誤り | 構文エラー。理由に、読めなくなった位置に来られる書き方（`expected …`）が続く | 示された位置の書き方を [文法リファレンス](../grammar/index.md) に合わせる |
| `transpile-arg-assign` | 誤り | シーンの引数（`＄０` など）に代入している | 引数への代入をやめる。今の文法では `＄０＝…` は構文エラー（`syntax`）として報告される |
| `transpile-property-in-expr` | 誤り | プロパティ参照（`＄％名前`）を式の中で使っている。使えるのは、変数への代入の右辺にそのまま置くときと、台詞の中だけである | 理由の文のとおり、先に変数へ代入してから式で使う |
| `transpile-continuation-without-actor` | 誤り | 継続行（`：` で始まる行）の前に、同じローカルシーンのアクター行が無い。前のローカルシーンのアクター行は引き継がない | 継続行の前にアクター行を置く |

`transpile-` で始まる 3 つは、構文としては読めるが、`pasta.dll` が Lua へ変換するときに失敗する書き方である。`pasta.dll` は最初の 1 つで読み込みをやめるが、検査はファイルの中の該当箇所をすべて報告する。

### モジュール名

辞書ファイルと辞書モジュールは、ファイル名からモジュール名が作られる。作り方は次のとおりで、`pasta.dll` と同じである。

- パスの先頭の `dic/` を外し、拡張子を外す。
- `.` と `-` を `_` に、フォルダの区切りを `.` に替える。
- 前に `pasta.scene.` を付ける（`dic/a-b.pasta` → `pasta.scene.a_b`）。

| code | 分類 | 意味 | 直し方 |
| ---- | ---- | ---- | ------ |
| `module-pasta-collision` | 誤り | 2 つ以上の辞書ファイルのモジュール名が同じ。`pasta.dll` は知らせずに、どちらかの辞書の内容を失う | どちらかのファイルの名前を変える |
| `module-lua-collision` | 誤り | 2 つ以上の辞書モジュール（`.lua`）のモジュール名が同じ。どちらかの `.lua` が読み込まれない | どちらかのファイルの名前を変える |
| `module-lua-shadowed` | 注意 | 辞書モジュール（`.lua`）のモジュール名が辞書ファイル（`.pasta`）と同じため、その `.lua` は読み込まれない | `.lua` を読み込ませたいなら、ファイルの名前を変える |

### シーン名

| code | 分類 | 意味 | 直し方 |
| ---- | ---- | ---- | ------ |
| `global-scene-collision` | 注意 | 名前の違うグローバルシーンが、[照合用の名前](../grammar/call-jump.md#シーン名の照合)に揃えると同じになる。どちらも同じ名前のシーンとして選ばれる。理由に、相手のシーンの名前と位置が出る | 別のシーンのつもりなら、照合用の名前が違うように名前を変える |
| `local-scene-collision` | 注意 | 同じグローバルシーンの中のローカルシーンについて、同じことが起きている | 同上 |
| `scene-shadowed-by-builtin` | 注意 | グローバルシーンの名前が組み込みの飛び先と同じため、`＞名前` ではそのシーンでなく組み込みのほうが呼ばれる | シーンの名前を変える |

組み込みの飛び先は、次の名前である。

- `GLOBAL` の関数: `yield`・`チェイントーク`・`close_ghost`・`ゴースト終了`・`pause_auto_talk`・`自動トーク停止`・`resume_auto_talk`・`自動トーク再開`
- act のメソッド: `init_scene`・`talk`・`sakura_script`・`raw_script`・`surface`・`wait`・`newline`・`clear`・`choice`・`choice_timeout`・`find_act_handler`・`find_handler`・`word`・`expr_fn`・`expr_fn_var`・`actor_proxy`・`global_fn`・`build`・`yield`・`find_scene`・`call`・`call_key`・`restore_scene`・`call_restore`・`failure`・`set_spot`・`clear_spot`
- SHIORI 用の act のメソッド: `build`・`transfer_date_to_var`・`transfer_req_to_var`・`set_property`・`get_property`

### 飛び先

| code | 分類 | 意味 | 直し方 |
| ---- | ---- | ---- | ------ |
| `call-target-missing` | 注意 | Call の飛び先が、辞書のシーンにも組み込みの飛び先にも見つからない。実行すると Call は失敗する（[Call が失敗したとき](../grammar/call-jump.md#call-が失敗したとき)） | 名前の書き間違いなら直す。Lua で定義した飛び先なら、そのままでよい |
| `choice-target-missing` | 注意 | 選択肢行のジャンプ先が、辞書のシーンに見つからない | 名前の書き間違いなら直す。Lua で定義・振り分けているなら、そのままでよい |

## 読むだけでは決まらない範囲

検査は辞書を読むだけで、実行はしない。そのため、次のものは確かめない。

- **動的な飛び先の Call**: 変数・関数呼び出し・連結・算術など、実行するまで検索キーが決まらない Call（`＞＄行き先` など）は、飛び先を探さず、数だけをまとめの行の「飛び先を検査しなかった Call」に出す。文字列リテラルだけの `＞「名前」` は、書いた名前で決まるので静的な飛び先として探す。
- **Lua の中身**: `scripts/` の Lua・辞書の Lua ブロック・辞書モジュール（`.lua`）の中身は読まない。Lua で定義したシーンや関数は飛び先の候補にならないので、それを呼ぶ Call は注意として報告される。飛び先の報告が注意なのはこのためである。
- **`On`・`script:` で始まる選択肢のジャンプ先**: ベースウェア（SSP）が別の意味で使うので探さない。
- **単語参照の参照先**: `＠単語` の単語が定義されているかは確かめない。
- **構文エラー・読めない辞書ファイル・`pasta.toml` の誤りがあるとき**: 飛び先の検査をまるごと省く（[出力の読み方](#出力の読み方)）。案内の行に出た理由を直してから、もう一度検査する。

注意が出ないことは、実行して失敗しないことの保証ではない。実行してからの失敗の見え方は [Call が失敗したとき](../grammar/call-jump.md#call-が失敗したとき) を、ゴーストが起動しないときは [起動シーケンスとモジュール解決](../reference/startup.md) を参照する。

## 終了コード

| 終了コード | いつ |
| ---------- | ---- |
| 0 | 誤りが無い（注意はあってもよい） |
| 1 | 誤りが 1 件以上ある |
| 1 | 使い方の誤り（`--target` が無い・`validate` に無いオプションを渡したなど）、または `--target` のフォルダが無い・フォルダでない・`ghost/master` を含まない |

使い方とフォルダの誤りは、`Error: ` で始まる理由を標準エラー出力に出す（使い方の誤りでは、続けて使い方を出す）。CI などでは、終了コードが 0 かどうかで合否を判定できる。

## release との関係

`pasta_check release`（配布フォルダと nar を作るコマンド）は、この検査を関門にしている。`release` は 6 段で進み、4 段目で、上書きコピー（`--copy`）を当てた後の配布フォルダの辞書を `validate` と同じ規則で検査する。

```text
[1/6] Preparing release folder...
[2/6] Copying target files...
[3/6] Applying overlay copies...
[4/6] Validating dictionaries...
[5/6] Generating update files...
[6/6] Creating NAR archive...
```

- 4 段目は `validate` と同じ形で問題とまとめの行を出す。
- 誤りが 1 件でもあると、`Error: dictionary validation found N error(s); update files and NAR were not created` を出して、更新ファイル（`updates.txt`）と nar を作らずに失敗の終了コードで終わる。
- 1 段目で `--nar` の位置にある前回の nar を消しているので、関門で止まったときは、新しい nar も前回の nar も残らない。壊れた辞書の nar や、古い nar を最新と取り違えて配ることはない。
- 注意だけのときは、注意を出したうえで 5 段目に進む。
- 検査を省くオプションは無い。

`release` の使い方の詳細は、[pasta_check の README](https://github.com/ekicyou/pasta/blob/main/crates/pasta_check/README.md) を参照する。

## エディタの診断

pasta VSCode 拡張（導入は [VSCode 接続と拡張導入](./vscode-setup.md)）は、同じ検査を拡張の中で走らせ、結果をエディタの診断として出す。`pasta_check` を入れていなくても動く。

- **検査する時**: `ghost/master` の配下の `.pasta` を保存したとき、`ghost/master/pasta.toml` を保存したとき、`ghost/master` の配下に `.pasta`・`.lua` を作った・消した・名前を変えたとき。そのファイルを含むゴーストの全体を検査する。`ghost`・`master`・`pasta.toml` の名前と拡張子 `.pasta`・`.lua` は、大文字・小文字を区別しない（`Ghost/Master/Pasta.toml` も同じ）。祖先に `ghost/master` が 2 つ以上あるときは、いちばん近いものをゴーストとする。続けて保存したときは、少し待ってからまとめて 1 回検査する。
- **検査しない時**: 入力の途中（保存する前）と、拡張を起動した直後。`ghost/master` の配下に無いファイルを保存したときも、ゴースト全体の検査はしない。
- **読む内容**: ディスクに保存された内容を読む。保存していない編集は、次に保存したときに検査に入る。
- **表示**: 誤りは Error、注意は Warning の重さで、該当するファイル・行に出る。開いていないファイルや `pasta.toml` についての報告も、問題パネルに出る。報告の出どころ（source）は `pasta validate` で、code は[報告の意味と直し方](#報告の意味と直し方)の表のものである。理由の文はコマンドの出力と同じである。
- **消える時**: 直して保存し、やり直した検査で報告されなくなった問題は、診断から消える。
- **構文エラーの重複**: 開いている `.pasta` には、入力のたびに更新する構文の診断が別に出ている。同じ構文エラーを二重に出さないよう、開いているファイルの構文エラーはゴースト全体の検査からは出さない。
- **検査できないとき**: ゴースト全体の検査が失敗したときは、そのゴーストの診断を消し、理由を拡張の出力チャンネルに出す。拡張の WASM が読み込めていないときは、ゴースト全体の検査をしない。

エディタの診断とコマンドの結果は、同じ規則・同じ文言である。CI や `release` の前に、手元で `pasta_check validate` を走らせて確かめることもできる。

---

> 【目閉じ】書き間違いは、誰にでもございますの。大事なのは、ゴーストを配る前に見つけ出すことですわ。報告を一つずつ読めば、直す場所はおのずと分かりますのよ。

> 【アンソニー】保存のたびに教えてもらえるのでしたら、見落としようもございませんね。

> 【照れ】……べ、別に、あなたが困らないように用意したわけではありませんわよ。
