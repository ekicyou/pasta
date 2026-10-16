# Requirements Document

## Project Description (Input)
**誰が困っているか**: ゴーストの作者（と、作者を手伝う AI・自動検査）。辞書（`.pasta`）の書き間違いを、SSP でゴーストを起動するまで確かめられない。

**今の状況**: 配布物を作る道具 `pasta_check` は `release`・`help`・`version` しか持たず、辞書を読まない。構文が壊れた辞書でも nar ができてしまう。辞書を起動せずに確かめたい場面は、マニュアルの自動検査・hello-pasta の段階辞書のテスト・使い捨てのクレートなどの手作りの代用品で埋めており、scene-anchor-link・failure-output-unification も「読み込む前に見つける仕組みはこのコマンドで」と先送りしている。バックログの「モジュール名の衝突」「サニタイズ後のシーン名の衝突の警告」、scene-name-alias が範囲の外に置いた「pasta_check に pasta.toml を読ませる」も受け皿が無い。

**何を変えるか**: `pasta_check` にサブコマンドを 1 つ足し、ゴーストのフォルダを受け取って辞書を全部読み、構文エラー・モジュール名の重なり・照合用の名前の重なり・飛び先の無い参照（読むだけで決まる範囲）を検査する。問題があればファイル・行・理由を示し、誤りが 1 つでもあれば失敗の終了コードで終わる。使い方をマニュアルとスキルに載せる。詳細は `brief.md` を参照。

## Introduction

ゴーストの作者が、SSP を起動せずに、`pasta_check` のコマンド 1 つでゴーストの辞書（`.pasta`）を検査できるようにする。検査は、ゴーストを読み込んだときに `pasta.dll` が失敗する書き方（構文エラー・`pasta.toml` の誤りなど）を先に見つけることと、読み込みは通るが作者の意図とずれやすい書き方（名前の重なり・飛び先の無い参照）を知らせることの 2 つを受け持つ。見つけたものは「誤り」と「注意」に分け、誤りが 1 つでもあれば失敗の終了コードで終わる（作成ツールは問題があれば止める）。同じ検査を VSCode 拡張にも載せ、作者が辞書を保存するたびに、同じ誤りと注意をエディタの診断として示す。想定していない書き方を意図の推測で救う規則は足さず、読むだけでは決まらないものは決まらないものとして扱う（書いたとおりに読む）。

各要件の注記「確定（OQ-n）」は、要件ディスカッション（2026-10-16）で確定した事項である。確定した事項の一覧は末尾にある。

### 用語

| 用語 | 意味 |
| ---- | ---- |
| 検査コマンド | この spec で `pasta_check` に足すサブコマンド `validate`（OQ-9） |
| ゴーストのフォルダ | `release` の `--target` と同じ、`ghost/master` を含むゴーストの開発フォルダ |
| 設置ディレクトリ | ゴーストのフォルダの `ghost/master`。`pasta.toml` と辞書の場所の基準 |
| 辞書ファイル | `pasta.toml` の `[loader]` `pasta_patterns` に一致し、`pasta.dll` が読み込む `.pasta` ファイル |
| 辞書モジュール | 辞書ファイルと、`pasta_patterns` から作られる `.lua` のパターンに一致する `.lua` ファイル。どちらもファイル名からモジュール名が作られる |
| 照合用の名前 | シーン名の文字・数字・`_` 以外の文字を `_` に置き換えた名前（マニュアル「シーン名の照合」） |
| 誤り | 検査コマンドが失敗の終了コードで終わる原因になる問題 |
| 注意 | 報告はするが、終了コードを失敗にしない問題 |
| 静的な飛び先 | Call のシーン名（`＞名前`）・文字列リテラル（`＞「名前」`）、選択肢行のジャンプ先（`＠？名前`）。書いた名前だけで検索キーが決まるもの |
| 動的な飛び先 | 変数・関数呼び出し・連結・算術など、実行するまで検索キーが決まらない Call のターゲット |

## Boundary Context

- **In scope**:
  - `pasta_check` への検査コマンドの追加（引数・終了コード・出力の形）
  - 辞書の集め方（`pasta.toml` の `[loader]` `pasta_patterns`）と、別名表（`[scene.alias]`）を読むこと
  - 検査の項目: 構文エラー、モジュール名の重なり、照合用の名前に揃えた後のシーン名の重なり、静的な飛び先の無い Call・選択肢行
  - `pasta_check release` との関係（`release` が配布する内容を検査し、誤りがあれば nar を作らない。OQ-3）
  - 検査を WASM にビルドできる共有の部品に置き、`pasta_check` と VSCode 拡張の両方から使うこと（OQ-6）
  - VSCode 拡張への診断（ゴースト全体の検査の結果を、保存のたびに表示する。OQ-6）
  - テスト（項目ごとのテストと、コマンドを実際に走らせるテスト、共有の部品の WASM ビルド、VSCode 拡張の診断のテスト）
  - 文書: マニュアルの新しいページ、`reference/pasta-toml.md` の「pasta_check は別名表を読まない」の書き換えとスキル references の再生成、`crates/pasta_check/README.md`、VSCode 拡張の README、スキル `pasta-check`、steering
- **Out of scope**:
  - 実行時の失敗の見せ方（`failure-output-unification` が持つ）
  - `pasta.dll` が読み込みのときに出す警告を足すこと。知らせるのは検査コマンドの側だけ
  - 台詞の中のシーンリンク `＠？シーン名` の記法とその検査（`scene-anchor-link` と、後から入る側が足す）
  - Lua のスクリプト（`scripts/`・辞書の Lua ブロック・辞書モジュールの `.lua`）の中身の検査
  - 単語参照（`＠単語`）の参照先の検査（brief の範囲に無い）
  - VSCode 拡張以外のエディタへの診断と、`pasta_lsp` のネイティブの言語サーバー（tower-lsp の経路）へのゴースト全体の診断の追加（VSCode 拡張は WASM の経路を使う。OQ-6）
  - 入力のたびに更新する、今の構文の診断の挙動を変えること
  - 機械向けの出力の形（JSON など）（OQ-5）
  - 代用品の置き換え（`.github/workflows/manual.yml` の回避、`tutorial_stages_test.rs` の手作りの検査）。コマンドが入った後の後片付けにする
  - `release` の 5 段の中身（同梱バルーン・`updates.txt`・nar の作り方）
  - 文法そのもの・ランタイムの読み込みとシーンの検索の規則の変更
- **Adjacent expectations**:
  - 名前の規則（照合用の名前・別名の置き換え・モジュール名・辞書の集め方）は、同じ版の `pasta.dll` と同じ結果になる。検査コマンドは独自の規則を持たない。
  - 後から入る spec（`scene-anchor-link`・`cue-command-runtime`・`call-attribute-filter`・`novel-talk-flow`）が、検査の項目を足す。この spec は、項目を足しても既存の項目の報告・誤りと注意の区別・終了コードの規則が変わらない作りにしておく。
  - `failure-output-unification` は実行時の失敗を受け持ち、この spec は実行する前に見つける側を受け持つ。実行時の失敗表記・ログの形には依存しない。
  - hello-pasta の配布物（`crates/pasta_sample_ghost/release.ps1`・リリース CI）は `pasta_check release` を通るので、`release` が検査を関門にすると、hello-pasta の辞書が誤りなしで通ることが前提になる。
  - 今のウェーブの約束（`.kiro/steering/roadmap.md`）により、`pasta_lua` の `loader/config/`・`pasta/act.lua`・`actor.lua`・`word.lua` は触らない。`Cargo.lock`・ルートの `Cargo.toml`・`release.yml` の `pasta_check` の公開の段は、この spec だけが触る。共有の部品を WASM にビルドできるようにするため、`pasta_core` の依存（`rand` が引き込む `getrandom`）の扱いを変える場合も、シーンのランダム選択の挙動は変えない。

## Requirements

### Requirement 1: 検査コマンドの起動と入力

**Objective:** As a ゴーストの作者, I want ゴーストのフォルダを指定するだけで辞書を検査したい, so that SSP を起動せずに書き間違いを確かめられる

#### Acceptance Criteria

1. When 作者が検査コマンドにゴーストのフォルダを指定して実行した, the pasta_check shall そのゴーストの辞書ファイルをすべて検査し、結果を出力する。
2. The pasta_check shall 検査コマンドのゴーストのフォルダの指定を、`release` の `--target` と同じ意味（`ghost/master` を含むゴーストの開発フォルダ）で受け付ける。
3. If ゴーストのフォルダが指定されていない、存在しない、または `ghost/master` を含まない, the pasta_check shall 理由を示して失敗の終了コードで終わり、検査をしない。
4. If 検査コマンドに知らないオプションが渡された, the pasta_check shall 理由と使い方を示して失敗の終了コードで終わる。
5. When 作者が `pasta_check --help` を実行した, the pasta_check shall 検査コマンドの名前・オプション・目的を、`release` と並べて使い方に示す。
6. The pasta_check shall 検査コマンドの実行中に、ゴーストのフォルダの中のファイルを作らず、書き換えず、消さない（トランスパイルのキャッシュも作らない）。
7. The pasta_check shall 検査コマンドの実行に、SSP・`pasta.dll` の起動・ネットワークへの接続を要しない。

> 確定（OQ-9）: 名前は `validate`（`pasta_check check` は名前が重なる）。フォルダの指定は `release` と同じ `--target` で、位置引数にはしない。`ghost/master` そのものを渡されたときは 3 のとおり理由を示して失敗にし、意図を推測して読み替えない。

### Requirement 2: 辞書の集め方と pasta.toml

**Objective:** As a ゴーストの作者, I want `pasta.dll` が実際に読み込む辞書と設定どおりに検査してほしい, so that 検査の結果と起動したときの結果が食い違わない

#### Acceptance Criteria

1. Where 設置ディレクトリに `pasta.toml` がある, the pasta_check shall その `[loader]` `pasta_patterns` に一致する辞書ファイルと、`pasta.dll` と同じ規則で作られる `.lua` のパターンに一致する辞書モジュールを、検査の対象として集める。
2. Where `pasta.toml` が無い、または `[loader]` `pasta_patterns` を書いていない, the pasta_check shall 既定値（`dic/**/*.pasta`）で辞書ファイルを集める。
3. The pasta_check shall 辞書ファイルを集めるときに、`pasta.dll` と同じく `..` を含むパターンを使わず、`profile/` 配下のファイルを対象に含めない。
4. Where `pasta.toml` に `[scene.alias]` がある, the pasta_check shall その別名表を使ってシーン名の宣言と静的な飛び先の検索を判定し、無ければ既定の別名表（`OnTalk = ["会話"]`）を使う。
5. If `pasta.toml` が、`pasta.dll` が読み込みエラーにする書き方（TOML として読めない、`[loader]`・`[scene.alias]` の型や値の誤り、別名の重複・連鎖など）をしている, the pasta_check shall 原因のキー名と、取れるときは行番号を示して、誤りとして報告する。
6. If 集めた辞書モジュールのファイル名が `init.pasta` または `init.lua` である, the pasta_check shall `pasta.dll` が読み込みに失敗することを示して、誤りとして報告する。
7. If `pasta_patterns` に一致する辞書ファイルが 1 つも無い, the pasta_check shall 注意として報告する。
8. If `pasta_patterns` に `..` を含むパターン、または `.pasta` で終わらないパターンがある, the pasta_check shall `pasta.dll` が無視・警告するのと同じく、注意として報告する。
9. The pasta_check shall 検査の結果に、検査した辞書ファイルの数を示す。
10. If 設置ディレクトリに `pasta.toml` が無い, the pasta_check shall 2.2 の既定値で辞書ファイルを集めて検査を続けたうえで、`pasta.dll` がゴーストを読み込めないことを示して、誤りとして報告する。

> 確定（OQ-7）: 辞書は `pasta_patterns` に従って集め、既定の場所だけを見る方式にはしない。既定の場所だけを見ると、`pasta_patterns` を変えたゴーストで検査と起動の結果が食い違い、Requirement 9.2 を守れないため。`pasta.toml` の項目のうち、読むのは `[loader]` の `pasta_patterns` と `[scene]` だけで、ほかのセクションの検査はこの spec の範囲に入れない。

### Requirement 3: 構文の検査

**Objective:** As a ゴーストの作者, I want すべての辞書ファイルの構文エラーを 1 回の実行でまとめて知りたい, so that 直しては起動する繰り返しを減らせる

#### Acceptance Criteria

1. The pasta_check shall 集めた辞書ファイルをすべて読み、構文エラーのあるファイルが見つかっても、残りのファイルの検査を続ける。
2. If 辞書ファイルに構文エラーがある, the pasta_check shall そのファイルのパス・行・（取れるときは）列・理由を示して、誤りとして報告する。
3. The pasta_check shall 各辞書ファイルについて、少なくとも `pasta.dll` がそのファイルの読み込みで最初に出す構文エラーと同じ位置のエラーを報告する。
4. If 辞書ファイルを UTF-8 として読めない、または読み込みに失敗する, the pasta_check shall そのファイルのパスと理由を示して、誤りとして報告する。
5. If 辞書ファイルに、構文としては読めるが `pasta.dll` がトランスパイルで失敗する書き方（シーンの引数 `＄０` などへの代入、式の中のプロパティ参照、アクター無しの継続行）がある, the pasta_check shall そのファイルのパス・行・理由を示して、構文エラーと同じ形で誤りとして報告する。

> 確定（OQ-4）: 5 の 3 つは、検査を通ったのに起動しないという穴を残さないため範囲に入れ、`pasta_lua` に依存せず構文木の上で `pasta.dll` と同じ規則で判定する（規則の一致はテストで守る。Requirement 9）。1 つのファイルの中で 2 つ目以降の構文エラーをどこまで報告するかは設計で決める（`pasta.dll` は最初の 1 つで止まる）。

### Requirement 4: モジュール名の重なり

**Objective:** As a ゴーストの作者, I want ファイル名の違う辞書が同じモジュール名になっていることを知りたい, so that 片方の辞書が黙って読まれない事故を防げる

#### Acceptance Criteria

1. If 2 つ以上の `.pasta` の辞書ファイルが、`pasta.dll` と同じ規則（`.` と `-` を `_` に置き換えるなど）で同じモジュール名になる, the pasta_check shall 重なった全ファイルのパスとモジュール名を示して、誤りとして報告する。
2. If `.lua` の辞書モジュールが `.pasta` の辞書ファイルと同じモジュール名になる, the pasta_check shall `pasta.dll` が `.lua` を読み込まないことを示して、注意として報告する。
3. If 2 つ以上の `.lua` の辞書モジュールが同じモジュール名になる, the pasta_check shall 重なった全ファイルのパスとモジュール名を示して、誤りとして報告する。
4. The pasta_check shall モジュール名の重なりを、ファイル名の書き方（`a-b.pasta`・`a.b.pasta`・`a_b.pasta`）と置き場所の階層の違いを区別せずに、モジュール名が同じかどうかだけで判定する。

> 確定（OQ-1）: `.pasta` どうし・`.lua` どうしの重なりは、`pasta.dll` が検出せずにどちらかの内容を失う（キャッシュの出力先とモジュールの読み込みが 1 つに潰れる）ため誤り。`.pasta` と `.lua` の重なりは `pasta.dll` が警告して `.pasta` を優先する決まった挙動のため注意。

### Requirement 5: シーン名の重なり

**Objective:** As a ゴーストの作者, I want 書き方の違うシーン名が同じ名前として扱われることや、呼べないシーンがあることを知りたい, so that 意図しないシーンが選ばれる・呼ばれない原因が分かる

#### Acceptance Criteria

1. If 書いた名前の違う 2 つ以上のグローバルシーンが、別名を置き換えた後の照合用の名前で同じ名前になる（例: `＊会話·A` と `＊会話_A`）, the pasta_check shall 各シーンの宣言のファイル・行と照合用の名前を示して、注意として報告する。
2. If 同じグローバルシーンの配下で、書いた名前の違う 2 つ以上のローカルシーンが、照合用の名前で同じ名前になる, the pasta_check shall 各シーンの宣言のファイル・行と照合用の名前を示して、注意として報告する。
3. The pasta_check shall 書いた名前がまったく同じシーンを複数定義したこと（同名シーンからの選択）、別名を置き換えた結果として同じ名前になったこと（`＊会話` と `＊OnTalk`）、名前どうしの前方一致の重なり（`＊挨拶` と `＊挨拶朝`）を、重なりとして報告しない。
4. If グローバルシーンの名前が、`pasta.dll` が組み込みで持つ Call の飛び先（`GLOBAL` の関数 `yield`・`チェイントーク`・`ゴースト終了`・`close_ghost`、act のメソッド、SHIORI 用の act のメソッド）と完全に同じである, the pasta_check shall そのシーンが同じ名前の Call では呼ばれないことを示して、注意として報告する。

> 確定（OQ-1・OQ-2）: 照合用の名前の重なりは、`pasta.dll` が同名シーンとして扱う決まった挙動（マニュアル「シーン名の照合」）であり、書いたとおりに読めば誤りではないため注意。4 の組み込みの名前は、検査コマンドと同じ版の `pasta.dll` が持つものに限り、ゴーストの Lua が足した名前は見ない。

### Requirement 6: 飛び先の無い参照

**Objective:** As a ゴーストの作者, I want 書き間違えた Call・選択肢の飛び先を起動する前に知りたい, so that 実行して初めて `【Call失敗：…】` や反応しない選択肢に気付くことが無くなる

#### Acceptance Criteria

1. When 検査コマンドが静的な飛び先の Call を検査する, the pasta_check shall `pasta.dll` と同じ順（その Call を書いたグローバルシーンの配下のローカルシーンの前方一致、次に別名を置き換えた後のグローバルシーンの前方一致）で、辞書に定義されたシーンから飛び先の候補を探す。
2. When 検査コマンドが選択肢行のジャンプ先を検査する, the pasta_check shall `pasta.dll` の既定の振り分けと同じ順（その選択肢行を書いたグローバルシーンの配下のローカルシーンの前方一致、次に別名を置き換えた後のグローバルシーンの前方一致）で、辞書に定義されたシーンから飛び先の候補を探す。
3. If 静的な飛び先の Call に辞書の中の候補が無く、その名前が `pasta.dll` の組み込みの飛び先（`GLOBAL` の関数・act のメソッド・SHIORI 用の act のメソッド）とも一致しない, the pasta_check shall Call 行のファイル・行・検索キーと、Lua で定義していれば問題ないことを示して、注意として報告する。
4. If 選択肢行のジャンプ先に辞書の中の候補が無い, the pasta_check shall 選択肢行のファイル・行・ジャンプ先と、Lua で定義・振り分けていれば問題ないことを示して、注意として報告する。
5. The pasta_check shall `On` または `script:` で始まる選択肢行のジャンプ先を、飛び先の検査の対象にしない。
6. The pasta_check shall 動的な飛び先の Call を、飛び先の検査で誤りにも注意にもしない。
7. The pasta_check shall 検査の結果に、飛び先を検査しなかった参照（動的な飛び先の Call）の数を示す。
8. While いずれかの辞書ファイルに構文エラーがある, the pasta_check shall 飛び先の無い参照を報告せず、構文エラーを直すまで飛び先の検査をしなかったことを示す。
9. The pasta_check shall 辞書の Lua ブロック・辞書モジュールの `.lua`・`scripts/` の中身を、飛び先の候補を探すのに使わない。

> 確定（OQ-1・OQ-2）: Call の飛び先は、辞書の外（Lua ブロックの `SCENE` の関数・`scripts/` で足した `GLOBAL` の関数とシーン）でも決まるため、読むだけでは「無い」と言い切れない。そのため 3・4 は注意にし、検査コマンドは Lua の中身を推測で読まない（9）。選択肢行も、`REG.OnChoiceSelectEx` の上書き・`＊OnChoiceSelectEx` のシーン・Lua で足したシーンで振り分けが変わるため注意にする。動的な飛び先は 1 件ずつは報告せず、数だけを示す（条件分岐の定型が動的な飛び先なので、1 件ずつ出すと多すぎる）。属性で絞る Call（`call-attribute-filter`）が入ったときの扱いは、その spec が足す。

### Requirement 7: 結果の出力と終了コード

**Objective:** As a ゴーストの作者（と自動検査）, I want 問題の場所と理由がすぐ分かる形で結果を受け取りたい, so that 直す場所に迷わず、CI などで合否を判定できる

#### Acceptance Criteria

1. The pasta_check shall 見つけた問題を 1 件につき 1 行で、誤りか注意かの区別・ファイルのパス・行（取れるときは列）・理由を含めて出力する。
2. The pasta_check shall ファイルのパスを、ゴーストのフォルダからの相対パスで示す。
3. The pasta_check shall 問題を、ファイルのパスの順、同じファイルの中では行の順に、実行のたびに同じ順で出力する。
4. When 検査が終わった, the pasta_check shall 最後に、検査した辞書ファイルの数・誤りの数・注意の数をまとめた行を出力する。
5. If 誤りが 1 件以上ある, the pasta_check shall 失敗の終了コードで終わる。
6. While 誤りが無い, the pasta_check shall 注意の有無にかかわらず、成功の終了コードで終わる。
7. The pasta_check shall 同じ項目の報告を、誤りと注意のどちらに分類するかを実行のたびに変えない（オプションで誤りと注意を入れ替えない）。
8. The pasta_check shall 検査の報告の種類（誤り・注意）と理由、まとめの行を日本語で出力する。

> 確定（OQ-5）: 出力は人が読む 1 行 1 件の形（`パス:行:列: 誤り|注意: 理由`。エディタや CI でも拾える形）だけにし、JSON などの機械向けの形は付けない。文言は日本語にする（0.4.0 の主題「書き間違えたときに原因が分かる」と、実行時の失敗表記 `【Call失敗：…】`・マニュアルに揃える）。`release` の既存の英語の表示は変えない（Requirement 8.5）。終了コードは 0 と 1 だけで、使い方の誤りと検査の誤りは今の `pasta_check` と同じく 1 に揃える。

### Requirement 8: release との関係

**Objective:** As a ゴーストの作者, I want 壊れた辞書を含む nar を作らないでほしい, so that 起動しないゴーストを配ってしまう事故を防げる

#### Acceptance Criteria

1. When 作者が `pasta_check release` を実行した, the pasta_check shall 配布フォルダに上書きコピー（`--copy`）を当てた後、更新ファイルと nar を作る前に、配布フォルダの辞書を検査コマンドと同じ規則で検査する。
2. If `release` の中の検査で誤りが見つかった, the pasta_check shall 検査コマンドと同じ形で問題を出力し、更新ファイルと nar を作らずに失敗の終了コードで終わる。
3. If `release` の中の検査で注意だけが見つかった, the pasta_check shall 注意を出力したうえで、`release` の残りの段を続ける。
4. The pasta_check shall `release` の中の検査を省くオプションを持たない。
5. The pasta_check shall `release` の既存の段（配布フォルダの準備・コピー・上書きコピー・更新ファイル・nar）の中身と、`release` の既存のオプションの意味を変えない。

> 確定（OQ-3）: `release` は既定で検査を関門にし、省く手段は持たない。作成ツールは問題があれば止める（壊れた配布物を警告だけで通さない）という原則から決まる。段の表示（`[n/5]`）に検査の段を足すかどうか、段 1 で消した前回の nar が残らないことの扱いは設計で決める。

### Requirement 9: 配り方と規則の一致

**Objective:** As a ゴーストの作者, I want 今までどおり `cargo install pasta_check` で手軽に入れたい, and 検査の結果が実際の起動と食い違わないでほしい, so that 道具を信頼して使える

#### Acceptance Criteria

1. The pasta_check shall `cargo install pasta_check` で入れるときに、LuaJIT をビルドする必要を持ち込まない。
2. The pasta_check shall 照合用の名前・別名の置き換え・モジュール名・辞書の集め方を、同じ版の `pasta.dll` と同じ規則で判定する。
3. When 同じ版の `pasta.dll` が読み込みに成功するゴーストを検査した, the pasta_check shall 構文エラー・`pasta.toml` の誤り・読み込めない辞書ファイルの名前を、誤りとして報告しない。
4. When 同じ版の `pasta.dll` が構文エラー・Requirement 3.5 に挙げたトランスパイルの誤り・Requirement 2.5 に挙げた `pasta.toml` の誤り・ファイル名の誤り・`pasta.toml` が無いことで読み込みに失敗する辞書を検査した, the pasta_check shall その原因を誤りとして報告する。
5. When 見本のゴースト hello-pasta を検査した, the pasta_check shall 誤りを報告しない。

> 確定（OQ-4）: 依存は構文を読む部品と名前の規則を持つ部品の側にとどめ、Lua への変換まで確かめる依存（LuaJIT を含む部品）は持たない。トランスパイルで失敗する書き方は Requirement 3.5 のとおり検査コマンドの側で判定する。規則を `pasta.dll` と揃える方法（一致テストなど）は設計で決める。

### Requirement 10: 文書

**Objective:** As a ゴーストの作者, I want 検査コマンドの使い方と各報告の意味をマニュアルで調べたい, so that 報告を読んで自分で直せる

#### Acceptance Criteria

1. The documentation shall 利用者マニュアル（`book/src/`）の「デバッグ」の部に、検査コマンドの使い方・検査の項目・誤りと注意の区別・各報告の意味と直し方・読むだけでは決まらない範囲・終了コード・`release` との関係を載せたページを持つ。
2. When 検査コマンドが入った, the documentation shall マニュアル `reference/pasta-toml.md` の「pasta_check は別名表を読まない」という記述を、検査コマンドが別名表と `pasta_patterns` を読むことに合わせて直し、スキル references を再生成する。
3. When 検査コマンドが入った, the documentation shall `crates/pasta_check/README.md` とスキル `pasta-check` に、検査コマンドの使い方と `release` が検査を関門にすることを載せる。
4. When 検査コマンドが入った, the documentation shall steering（`tech.md`・`structure.md` の `pasta_check`・共有の検査の部品・`pasta_lsp`・VSCode 拡張の役割と依存）を実態に合わせる。
5. When マニュアルに章を足した, the documentation shall 目次（`SUMMARY.md`）と、章の数を決め打ちしている検査（今は 60 章）を合わせて直す。
6. When エディタの診断が入った, the documentation shall VSCode 拡張の README（`editors/vscode/README.md`）と、マニュアルの「デバッグ」の部の検査コマンドの章に、エディタで誤りと注意が表示されることと、表示される時（保存のとき）を載せる。

> 確定（OQ-8）: マニュアルには「デバッグ」の部に検査コマンドの章を 1 つ足す（書き間違いに困った作者が開く部のため）。`release` の詳しい説明は README とスキルに残し、章からはリンクだけにする。

### Requirement 11: エディタの診断

**Objective:** As a ゴーストの作者, I want VSCode で辞書を書いているときに、検査コマンドと同じ誤りと注意をその場で見たい, so that SSP を起動せず、コマンドも打たずに、保存するたびに書き間違いに気付ける

#### Acceptance Criteria

1. The 共有の検査の部品 shall WASM（`wasm32-unknown-unknown`）とネイティブの両方にビルドでき、`pasta_check` と VSCode 拡張の両方から使われる。
2. The VSCode 拡張 shall 同じゴーストに対して、検査コマンドと同じ規則で、同じ誤りと注意を、同じ文言で報告する。
3. When 作者が VSCode で `ghost/master` の配下の辞書ファイルを保存した, the VSCode 拡張 shall そのゴーストの全体を検査し、誤りと注意を、該当するファイル・行の診断として表示する。
4. The VSCode 拡張 shall 誤りを Error、注意を Warning の重さで表示する。
5. The VSCode 拡張 shall 開いていない辞書ファイル・`pasta.toml` の誤りと注意も、問題パネルに表示する。
6. When 辞書ファイルの追加・削除・名前の変更、または `pasta.toml` の保存があった, the VSCode 拡張 shall そのゴーストの検査をやり直して診断を更新する。
7. When 前の検査で報告した問題が、やり直した検査で報告されなくなった, the VSCode 拡張 shall その問題の診断を消す。
8. The VSCode 拡張 shall 検査コマンドと同じ辞書ファイルの集合（`pasta.toml` の `pasta_patterns` に従う。Requirement 2）を検査する。
9. The VSCode 拡張 shall 同じ問題を、入力のたびの構文の診断とゴースト全体の検査の診断とで二重に表示しない。
10. If 保存した辞書ファイルが `ghost/master` の配下に無い, the VSCode 拡張 shall ゴースト全体の検査をせず、今の構文の診断だけを続ける。
11. The VSCode 拡張 shall 検査に、`pasta_check` のインストール・SSP の起動・ネットワークへの接続を要しない。
12. While 拡張の WASM が読み込めていない（今の TextMate だけの動作）, the VSCode 拡張 shall ゴースト全体の検査をせず、拡張の起動を失敗させない。

> 確定（OQ-6）: 検査を WASM にビルドできる共有の部品に置き、`pasta_check` と VSCode 拡張が同じ部品を呼ぶ。規則が CLI とエディタの 2 か所に分かれて食い違うことを、最初から防ぐため。表示は保存のときに限り、入力のたびの更新は今の構文の診断に任せる。共有の部品の置き場所（新しいクレートか、既存のクレートか）、`pasta_core` を WASM にビルドできるようにする方法、拡張の側でのファイルの集め方（照合の規則を TypeScript に写さない）は設計で決める。

## 確定事項（要件ディスカッション 2026-10-16）

各項目の根拠（ファイル・行）は `research.md` の「未決事項の根拠」にある。OQ-3・OQ-7 は原則と Requirement 9.2 から決まるため議題にせず確定した。OQ-6 は個別に議論して、推奨案（共有しない）から「含める」に改めた。残りは推奨案で確定した。

- **OQ-1 誤りと注意の線引き**: `pasta.dll` が読み込みに失敗するもの（構文・トランスパイルの 3 つ・`pasta.toml`・`init.pasta`）と、`pasta.dll` が検出せずに内容を失うもの（`.pasta` どうし・`.lua` どうしのモジュール名の重なり）を誤りにする。決まった挙動で動くが意図とずれやすいもの（照合用の名前の重なり・組み込みの名前と同じシーン・`.pasta` と `.lua` の重なり・飛び先の無い参照）を注意にする。
- **OQ-2 読むだけでどこまで飛び先を決められるか**: 辞書の中のシーンと組み込みの名前（`GLOBAL`・act のメソッド・SHIORI 用の act のメソッド）だけで探し、見つからなければ注意。Lua の中身は読まない。動的な飛び先は数だけを示す。
- **OQ-3 `release` の関門**（原則で確定）: 上書きコピーの後・更新ファイルの前に既定で検査し、誤りがあれば nar を作らない。省くオプションは持たない。
- **OQ-4 依存の重さ**: `pasta_dsl`・`pasta_core`（と `toml` など）だけにして LuaJIT を持ち込まない。トランスパイルで初めて分かる 3 つの誤りは、検査コマンドの側で同じ規則で判定して誤りにする。
- **OQ-5 出力の形と文言**: 1 行 1 件の人が読む形とまとめの行だけ。日本語。終了コードは 0 と 1。機械向けの形は付けない。
- **OQ-6 エディタ（LSP）との共有**（個別に議論して確定）: 検査を WASM にビルドできる共有の部品に置き、`pasta_check` と VSCode 拡張の両方から使う。VSCode 拡張は保存のたびにゴースト全体を検査し、誤りと注意を診断として表示する（Requirement 11）。ネイティブの言語サーバーと VSCode 以外のエディタは範囲の外。
- **OQ-7 辞書の集め方**（Requirement 9.2 の帰結で確定）: `pasta.toml` の `pasta_patterns` に従う。
- **OQ-8 マニュアルの章**: 「デバッグ」の部に検査コマンドの章を 1 つ足し、`release` の説明は README とスキルに残す。
- **OQ-9 コマンドの名前と引数**: `pasta_check validate --target <ゴーストのフォルダ>`。
