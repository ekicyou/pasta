# Requirements Document

## Project Description (Input)
**誰が困っているか**: ゴーストの作者（と、作者を手伝う AI・自動検査）。辞書（`.pasta`）の書き間違いを、SSP でゴーストを起動するまで確かめられない。

**今の状況**: 配布物を作る道具 `pasta_check` は `release`・`help`・`version` しか持たず、辞書を読まない。構文が壊れた辞書でも nar ができてしまう。辞書を起動せずに確かめたい場面は、マニュアルの自動検査・hello-pasta の段階辞書のテスト・使い捨てのクレートなどの手作りの代用品で埋めており、scene-anchor-link・failure-output-unification も「読み込む前に見つける仕組みはこのコマンドで」と先送りしている。バックログの「モジュール名の衝突」「サニタイズ後のシーン名の衝突の警告」、scene-name-alias が範囲の外に置いた「pasta_check に pasta.toml を読ませる」も受け皿が無い。

**何を変えるか**: `pasta_check` にサブコマンドを 1 つ足し、ゴーストのフォルダを受け取って辞書を全部読み、構文エラー・モジュール名の重なり・照合用の名前の重なり・飛び先の無い参照（読むだけで決まる範囲）を検査する。問題があればファイル・行・理由を示し、誤りが 1 つでもあれば失敗の終了コードで終わる。使い方をマニュアルとスキルに載せる。詳細は `brief.md` を参照。

## Introduction

ゴーストの作者が、SSP を起動せずに、`pasta_check` のコマンド 1 つでゴーストの辞書（`.pasta`）を検査できるようにする。検査は、ゴーストを読み込んだときに `pasta.dll` が失敗する書き方（構文エラー・`pasta.toml` の誤りなど）を先に見つけることと、読み込みは通るが作者の意図とずれやすい書き方（名前の重なり・飛び先の無い参照）を知らせることの 2 つを受け持つ。見つけたものは「誤り」と「注意」に分け、誤りが 1 つでもあれば失敗の終了コードで終わる（作成ツールは問題があれば止める）。想定していない書き方を意図の推測で救う規則は足さず、読むだけでは決まらないものは決まらないものとして扱う（書いたとおりに読む）。

この文書の「仮置き」は、brief.md だけでは決まらず、要件ディスカッションで確定する事項である。各仮置きには、対応する未決事項の番号（OQ-n）を付ける。未決事項の一覧は末尾にある。

### 用語

| 用語 | 意味 |
| ---- | ---- |
| 検査コマンド | この spec で `pasta_check` に足すサブコマンド。名前は仮置きで `check`（OQ-9） |
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
  - `pasta_check release` との関係（仮置きでは、`release` が配布する内容を検査し、誤りがあれば nar を作らない。OQ-3）
  - テスト（項目ごとのテストと、コマンドを実際に走らせるテスト）
  - 文書: マニュアルの新しいページ、`reference/pasta-toml.md` の「pasta_check は別名表を読まない」の書き換えとスキル references の再生成、`crates/pasta_check/README.md`、スキル `pasta-check`、steering
- **Out of scope**:
  - 実行時の失敗の見せ方（`failure-output-unification` が持つ）
  - `pasta.dll` が読み込みのときに出す警告を足すこと。知らせるのは検査コマンドの側だけ
  - 台詞の中のシーンリンク `＠？シーン名` の記法とその検査（`scene-anchor-link` と、後から入る側が足す）
  - Lua のスクリプト（`scripts/`・辞書の Lua ブロック・辞書モジュールの `.lua`）の中身の検査
  - 単語参照（`＠単語`）の参照先の検査（brief の範囲に無い）
  - エディタ（LSP・VSCode 拡張）に診断を足すこと（共有できる形にするかは OQ-6）
  - 代用品の置き換え（`.github/workflows/manual.yml` の回避、`tutorial_stages_test.rs` の手作りの検査）。コマンドが入った後の後片付けにする
  - `release` の 5 段の中身（同梱バルーン・`updates.txt`・nar の作り方）
  - 文法そのもの・ランタイムの読み込みとシーンの検索の規則の変更
- **Adjacent expectations**:
  - 名前の規則（照合用の名前・別名の置き換え・モジュール名・辞書の集め方）は、同じ版の `pasta.dll` と同じ結果になる。検査コマンドは独自の規則を持たない。
  - 後から入る spec（`scene-anchor-link`・`cue-command-runtime`・`call-attribute-filter`・`novel-talk-flow`）が、検査の項目を足す。この spec は、項目を足しても既存の項目の報告・誤りと注意の区別・終了コードの規則が変わらない作りにしておく。
  - `failure-output-unification` は実行時の失敗を受け持ち、この spec は実行する前に見つける側を受け持つ。実行時の失敗表記・ログの形には依存しない。
  - hello-pasta の配布物（`crates/pasta_sample_ghost/release.ps1`・リリース CI）は `pasta_check release` を通るので、`release` が検査を関門にすると、hello-pasta の辞書が誤りなしで通ることが前提になる。
  - 今のウェーブの約束（`.kiro/steering/roadmap.md`）により、`pasta_lua` の `loader/config/`・`pasta/act.lua`・`actor.lua`・`word.lua` は触らない。`Cargo.lock`・ルートの `Cargo.toml`・`release.yml` の `pasta_check` の公開の段は、この spec だけが触る。

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

> 仮置き: コマンド名は `check`、フォルダの指定は `--target`（OQ-9）。

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

> 確定（OQ-7）: 辞書は `pasta_patterns` に従って集め、既定の場所だけを見る方式にはしない。既定の場所だけを見ると、`pasta_patterns` を変えたゴーストで検査と起動の結果が食い違い、Requirement 9.2 を守れないため。`pasta.toml` の項目のうち、読むのは `[loader]` の `pasta_patterns` と `[scene]` だけで、ほかのセクションの検査はこの spec の範囲に入れない。

### Requirement 3: 構文の検査

**Objective:** As a ゴーストの作者, I want すべての辞書ファイルの構文エラーを 1 回の実行でまとめて知りたい, so that 直しては起動する繰り返しを減らせる

#### Acceptance Criteria

1. The pasta_check shall 集めた辞書ファイルをすべて読み、構文エラーのあるファイルが見つかっても、残りのファイルの検査を続ける。
2. If 辞書ファイルに構文エラーがある, the pasta_check shall そのファイルのパス・行・（取れるときは）列・理由を示して、誤りとして報告する。
3. The pasta_check shall 各辞書ファイルについて、少なくとも `pasta.dll` がそのファイルの読み込みで最初に出す構文エラーと同じ位置のエラーを報告する。
4. If 辞書ファイルを UTF-8 として読めない、または読み込みに失敗する, the pasta_check shall そのファイルのパスと理由を示して、誤りとして報告する。
5. Where 構文としては読めるが `pasta.dll` がトランスパイルで失敗する書き方（シーンの引数 `＄０` などへの代入、式の中のプロパティ参照、アクター無しの継続行）を検査する範囲に入れる, the pasta_check shall それらを構文エラーと同じ形で、誤りとして報告する。

> 仮置き: 5 は、依存の重さ（OQ-4）の結論で範囲が決まる。`pasta_lua` に依存しない場合は、この 3 つを検査コマンドの側で同じ規則で判定するか、範囲の外に置くかを設計で決める。1 つのファイルの中で 2 つ目以降の構文エラーをどこまで報告するかは設計で決める（`pasta.dll` は最初の 1 つで止まる）。

### Requirement 4: モジュール名の重なり

**Objective:** As a ゴーストの作者, I want ファイル名の違う辞書が同じモジュール名になっていることを知りたい, so that 片方の辞書が黙って読まれない事故を防げる

#### Acceptance Criteria

1. If 2 つ以上の `.pasta` の辞書ファイルが、`pasta.dll` と同じ規則（`.` と `-` を `_` に置き換えるなど）で同じモジュール名になる, the pasta_check shall 重なった全ファイルのパスとモジュール名を示して、誤りとして報告する。
2. If `.lua` の辞書モジュールが `.pasta` の辞書ファイルと同じモジュール名になる, the pasta_check shall `pasta.dll` が `.lua` を読み込まないことを示して、注意として報告する。
3. If 2 つ以上の `.lua` の辞書モジュールが同じモジュール名になる, the pasta_check shall 重なった全ファイルのパスとモジュール名を示して、誤りとして報告する。
4. The pasta_check shall モジュール名の重なりを、ファイル名の書き方（`a-b.pasta`・`a.b.pasta`・`a_b.pasta`）と置き場所の階層の違いを区別せずに、モジュール名が同じかどうかだけで判定する。

> 仮置き: `.pasta` どうし・`.lua` どうしの重なりは、`pasta.dll` が検出せずにどちらかの内容を失う（キャッシュの出力先とモジュールの読み込みが 1 つに潰れる）ため誤り。`.pasta` と `.lua` の重なりは `pasta.dll` が警告して `.pasta` を優先する決まった挙動のため注意（OQ-1）。

### Requirement 5: シーン名の重なり

**Objective:** As a ゴーストの作者, I want 書き方の違うシーン名が同じ名前として扱われることや、呼べないシーンがあることを知りたい, so that 意図しないシーンが選ばれる・呼ばれない原因が分かる

#### Acceptance Criteria

1. If 書いた名前の違う 2 つ以上のグローバルシーンが、別名を置き換えた後の照合用の名前で同じ名前になる（例: `＊会話·A` と `＊会話_A`）, the pasta_check shall 各シーンの宣言のファイル・行と照合用の名前を示して、注意として報告する。
2. If 同じグローバルシーンの配下で、書いた名前の違う 2 つ以上のローカルシーンが、照合用の名前で同じ名前になる, the pasta_check shall 各シーンの宣言のファイル・行と照合用の名前を示して、注意として報告する。
3. The pasta_check shall 書いた名前がまったく同じシーンを複数定義したこと（同名シーンからの選択）、別名を置き換えた結果として同じ名前になったこと（`＊会話` と `＊OnTalk`）、名前どうしの前方一致の重なり（`＊挨拶` と `＊挨拶朝`）を、重なりとして報告しない。
4. If グローバルシーンの名前が、`pasta.dll` が組み込みで持つ Call の飛び先（`GLOBAL` の関数 `yield`・`チェイントーク`・`ゴースト終了`・`close_ghost` と、act のメソッド）と完全に同じである, the pasta_check shall そのシーンが同じ名前の Call では呼ばれないことを示して、注意として報告する。

> 仮置き: 照合用の名前の重なりは、`pasta.dll` が同名シーンとして扱う決まった挙動であり、書いたとおりに読めば誤りではないため注意（OQ-1）。4 の組み込みの名前は、検査コマンドと同じ版の `pasta.dll` が持つものに限り、ゴーストの Lua が足した名前は見ない（OQ-2）。

### Requirement 6: 飛び先の無い参照

**Objective:** As a ゴーストの作者, I want 書き間違えた Call・選択肢の飛び先を起動する前に知りたい, so that 実行して初めて `【Call失敗：…】` や反応しない選択肢に気付くことが無くなる

#### Acceptance Criteria

1. When 検査コマンドが静的な飛び先の Call を検査する, the pasta_check shall `pasta.dll` と同じ順（その Call を書いたグローバルシーンの配下のローカルシーンの前方一致、次に別名を置き換えた後のグローバルシーンの前方一致）で、辞書に定義されたシーンから飛び先の候補を探す。
2. When 検査コマンドが選択肢行のジャンプ先を検査する, the pasta_check shall `pasta.dll` の既定の振り分けと同じ順（その選択肢行を書いたグローバルシーンの配下のローカルシーンの前方一致、次に別名を置き換えた後のグローバルシーンの前方一致）で、辞書に定義されたシーンから飛び先の候補を探す。
3. If 静的な飛び先の Call に辞書の中の候補が無く、その名前が `pasta.dll` の組み込みの飛び先（`GLOBAL` の関数・act のメソッド）とも一致しない, the pasta_check shall Call 行のファイル・行・検索キーを示して、注意として報告する。
4. If 選択肢行のジャンプ先に辞書の中の候補が無い, the pasta_check shall 選択肢行のファイル・行・ジャンプ先を示して、注意として報告する。
5. The pasta_check shall `On` または `script:` で始まる選択肢行のジャンプ先を、飛び先の検査の対象にしない。
6. The pasta_check shall 動的な飛び先の Call を、飛び先の検査で誤りにも注意にもしない。
7. The pasta_check shall 検査の結果に、飛び先を検査しなかった参照（動的な飛び先の Call）の数を示す。
8. While いずれかの辞書ファイルに構文エラーがある, the pasta_check shall 飛び先の無い参照を報告せず、構文エラーを直すまで飛び先の検査をしなかったことを示す。

> 仮置き: Call の飛び先は、辞書の外（Lua ブロックの `SCENE` の関数・`scripts/` で足した `GLOBAL` の関数とシーン）でも決まるため、読むだけでは「無い」と言い切れない。そのため 3・4 は注意にし、検査コマンドは Lua の中身を推測で読まない。選択肢行も、`REG.OnChoiceSelectEx` の上書き・`＊OnChoiceSelectEx` のシーン・Lua で足したシーンで振り分けが変わるため注意にする（OQ-1・OQ-2）。動的な飛び先は 1 件ずつは報告せず、数だけを示す（OQ-2）。属性で絞る Call（`call-attribute-filter`）が入ったときの扱いは、その spec が足す。

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

> 仮置き: 出力は人が読む 1 行 1 件の形（`パス:行:列: 種類: 理由` のようにエディタや CI でも拾える形）だけにし、JSON などの機械向けの形は付けない。理由の文言の言語（今の `pasta_check` と `pasta.dll` のログは英語、マニュアルは日本語）は未決（OQ-5）。使い方の誤りと検査の誤りの終了コードは、今の `pasta_check` と同じく 1 に揃える（OQ-5）。

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
4. When 同じ版の `pasta.dll` が構文エラー・Requirement 2.5 に挙げた `pasta.toml` の誤り・ファイル名の誤りで読み込みに失敗する辞書を検査した, the pasta_check shall その原因を誤りとして報告する。
5. When 見本のゴースト hello-pasta を検査した, the pasta_check shall 誤りを報告しない。

> 仮置き: 依存は構文を読む部品と名前の規則を持つ部品の側にとどめ、Lua への変換まで確かめる依存（LuaJIT を含む部品）は持たない（OQ-4）。その場合 4 の「トランスパイルで失敗する書き方」は Requirement 3.5 の結論に従う。

### Requirement 10: 文書

**Objective:** As a ゴーストの作者, I want 検査コマンドの使い方と各報告の意味をマニュアルで調べたい, so that 報告を読んで自分で直せる

#### Acceptance Criteria

1. The documentation shall 利用者マニュアル（`book/src/`）に、検査コマンドの使い方・検査の項目・誤りと注意の区別・各報告の意味と直し方・読むだけでは決まらない範囲・終了コード・`release` との関係を載せたページを持つ。
2. When 検査コマンドが入った, the documentation shall マニュアル `reference/pasta-toml.md` の「pasta_check は別名表を読まない」という記述を、検査コマンドが別名表と `pasta_patterns` を読むことに合わせて直し、スキル references を再生成する。
3. When 検査コマンドが入った, the documentation shall `crates/pasta_check/README.md` とスキル `pasta-check` に、検査コマンドの使い方と `release` が検査を関門にすることを載せる。
4. When 検査コマンドが入った, the documentation shall steering（`tech.md`・`structure.md` の `pasta_check` の役割と依存）を実態に合わせる。
5. When マニュアルに章を足した, the documentation shall 目次（`SUMMARY.md`）と、章の数を決め打ちしている検査（今は 60 章）を合わせて直す。

> 仮置き: マニュアルには検査コマンドの章を 1 つ足し、`release` の詳しい説明は README とスキルに残す（章からはリンクだけ）（OQ-8）。

## 未決事項（要件ディスカッションで確定する）

各項目の詳しい根拠（ファイル・行）は `research.md` の「未決事項の根拠」にある。

- **OQ-1 誤りと注意の線引き**: 仮置きは、`pasta.dll` が読み込みに失敗するもの（構文・`pasta.toml`・`init.pasta`）と、`pasta.dll` が検出せずに内容を失うもの（`.pasta` どうしのモジュール名の重なり）を誤り、決まった挙動で動くが意図とずれやすいもの（照合用の名前の重なり・組み込みの名前と同じシーン・`.pasta` と `.lua` の重なり・飛び先の無い参照）を注意にする。
- **OQ-2 読むだけでどこまで飛び先を決められるか**: 仮置きは、辞書の中のシーンと組み込みの名前だけで探し、見つからなければ注意。Lua の中身は読まない。動的な飛び先は 1 件ずつは報告せず数だけを示す。
- **OQ-3 `release` の関門**（原則で確定）: 上書きコピーの後・更新ファイルの前に既定で検査し、誤りがあれば nar を作らない。省くオプションは持たない。
- **OQ-4 依存の重さ**: 仮置きは、`pasta_dsl`・`pasta_core`（と `toml`）だけにして LuaJIT を持ち込まない。トランスパイルで初めて分かる 3 つの誤りの扱いは未決。
- **OQ-5 出力の形と文言**: 仮置きは、1 行 1 件の人が読む形だけ（機械向けの JSON は付けない）。文言の言語と終了コードの細分は未決。
- **OQ-6 エディタ（LSP）との共有**: 仮置きは、この spec ではエディタに診断を足さず、共有も要件にしない。
- **OQ-7 辞書の集め方**（Requirement 9.2 の帰結で確定）: `pasta.toml` の `pasta_patterns` に従う。
- **OQ-8 マニュアルの章**: 仮置きは、検査コマンドの章を 1 つ足し、`release` の説明は README とスキルに残す。
- **OQ-9 コマンドの名前と引数**: 仮置きは、`pasta_check check --target <path>`。
