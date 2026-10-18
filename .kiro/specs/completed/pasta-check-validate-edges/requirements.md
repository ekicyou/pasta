# Requirements Document

## Project Description (Input)
ゴーストを書く人が `pasta_check validate`（辞書の検査）と VSCode 拡張の検査を使うと、ふつうの辞書では pasta.dll と同じ規則で正しく報告される。けれども 3 つの端では、報告が原因を指さない、または検査が走らない。(1) UTF-8 で読めない辞書・読めない辞書があっても飛び先の検査を省かないので、本当の原因とは別の「飛び先が無い」が並ぶ。(2) 名前が `.pasta` で終わるフォルダは報告しない。(3) 拡張は小文字の `ghost/master` の下だけを検査し、Windows の CLI と範囲がずれる。あわせて、マニュアルの CI の「Tutorial syntax validation」の段と hello-pasta のテスト `scene_names_do_not_prefix_collide` は、検査のコマンドが無かった頃の代用品のままである。この spec で 3 つの端の扱いを決めて直し、報告の表（`book/src/debug/validate.md`）を更新し、代用品を置き換えるかを決めて、決めたなら置き換える（詳しくは brief.md）。

## Introduction

辞書の検査（`pasta_check validate` と、VSCode 拡張のゴースト全体の診断）が、`pasta-check-dic-validate` で残した 3 つの端でも、本当の原因を指す報告を出すようにする。

1. 辞書の中身が分からないとき（読めない辞書・壊れた設定）は、「飛び先が無い」を出さない。出すと、本当の原因とは別の誤った報告が並ぶ。
2. 辞書のパターンに一致するフォルダ（例 `dic/x.pasta/`）を報告する。pasta.dll はそのフォルダを辞書として読もうとして失敗する。
3. エディタと CLI で、検査するゴーストの範囲をそろえる。`Ghost/Master` のように大文字の混じったフォルダでも、エディタで検査が走るようにする。

あわせて、検査のコマンドが無かった頃の代用品（マニュアルの CI の段・hello-pasta のテストの一部）を、`validate` に置き換える。

想定していない書き方を意図の推測で救う規則は足さない。書いたとおりに読み、pasta.dll と同じ結果を報告する。

各要件の注記「確定（OQ-n）」は、brief.md だけでは決まらず、2026-10-17 の要件ディスカッションで決めた事項である（全議題を推奨案で確定）。一覧は末尾の「確定事項」にある。

### 用語

`pasta-check-dic-validate` の requirements.md の用語（検査コマンド・ゴーストのフォルダ・設置ディレクトリ・辞書ファイル・辞書モジュール・誤り・注意・静的な飛び先・動的な飛び先）をそのまま使う。この spec で足す用語は次のとおり。

| 用語 | 意味 |
| ---- | ---- |
| 飛び先の検査 | 静的な飛び先の Call と選択肢行のジャンプ先が辞書のシーンにあるかを確かめる検査（`call-target-missing`・`choice-target-missing` を出す） |
| 案内の行 | 飛び先の検査を省いたときに、まとめの行の前に出す 1 行（今は「構文エラーがあるため、飛び先の検査をしていません。」） |
| 辞書のパターン | `pasta.toml` の `[loader]` `pasta_patterns`（無ければ既定の `dic/**/*.pasta`）と、そこから作られる `.lua` のパターン |
| 検査する側 | `pasta_check validate`、`pasta_check release` の段 4、VSCode 拡張のゴースト全体の診断の 3 つ |

## Boundary Context

- **In scope**:
  - 飛び先の検査を省く条件の見直しと、案内の行の文言（Requirement 1）
  - 辞書のパターンに一致するフォルダの報告（Requirement 2）
  - VSCode 拡張が検査するゴーストの範囲と、検査を起動する条件の大文字・小文字の扱い（Requirement 3）
  - CLI とエディタで同じ報告になること（Requirement 4）
  - 代用品の置き換え（マニュアルの CI の段、hello-pasta のテストの組み込みの名前の部分）と、前方一致の重なりの扱いの裁定（Requirement 5）
  - テストと、pasta.dll と同じ結果であることを守る一致テストの見直し（Requirement 6）
  - 文書（マニュアル `book/src/debug/validate.md`、必要なら README・スキル）（Requirement 7）
- **Out of scope**:
  - 新しい種類の検査を広げること。前方一致の重なりの注意も足さない（確定（OQ-9））
  - 機械向けの出力の形（JSON など）
  - `release` の段の中身（同梱バルーン・`updates.txt`・nar の作り方）
  - 文法・ランタイムの読み込み・シーンの検索の規則の変更（pasta.dll の振る舞いは変えない）
  - hello-pasta の段階の辞書を読み込んで応答を確かめるテスト（`every_stage_loads_and_responds` など）を `validate` に置き換えること。これは実行時の応答の検査で、`validate` では代わりにならない
- **Adjacent expectations**:
  - 報告の種類を足す・分類を変える・案内の行の文言を変えるのは、`pasta-check-dic-validate` の design.md の Revalidation Triggers に当たる。CI・作者のスクリプト・マニュアル・`wasmBridge.ts` の形の検査に効く。
  - 後から検査を足す spec（`call-attribute-filter`・`scene-anchor-link`・`cue-command-runtime`・`novel-talk-flow`）は、この spec が決めた「飛び先の検査を省く条件」に従う。
  - `novel-talk-flow` は、ノベル向けの設定で場面の名前を完全一致で選ぶ道を入れる。前方一致の重なりの問題（`＊第1章` と `＊第10章`）は、その spec が場面の選び方で受け持つ（確定（OQ-9））。
  - `release` は段 4 で `validate` と同じ規則を使うので、この spec で足す誤り（Requirement 2）は `release` の関門でも止まる。
  - `build.yml` の CI は、どの変更でも `cargo test --all` を走らせ、hello-pasta の段階のテストもそこで走る。

## Requirements

### Requirement 1: 辞書の中身が分からないときは飛び先の検査を省く

**Objective:** As a ゴーストの作者, I want 読めない辞書や壊れた設定があるときに、それに引きずられた「飛び先が無い」を見たくない, so that 本当の原因（読めないファイル・設定の誤り）だけを直せばよいと分かる

#### Acceptance Criteria

1. If いずれかの辞書ファイルを UTF-8 として読めない、または読めない, the pasta_check shall 飛び先の検査をまるごと省き、飛び先の無い Call・選択肢を報告しない。
2. If `pasta.toml` を読めない、TOML として読めない、`[loader]` の型が違う、`pasta_patterns` にパターンとして読めないものがある、または `[scene.alias]`・`[scene]` が誤っている, the pasta_check shall 飛び先の検査をまるごと省き、飛び先の無い Call・選択肢を報告しない。
3. While いずれかの辞書ファイルに構文エラーがある, the pasta_check shall 今までどおり飛び先の検査をまるごと省く。
4. When 飛び先の検査を省いた, the pasta_check shall 案内の行に、省いた理由（構文エラー・読めない辞書ファイル・`pasta.toml` の誤りのうち、当てはまるものすべて）を示す。
5. When 飛び先の検査を省いた, the pasta_check shall まとめの行に「飛び先を検査しなかった Call」の項を出さない（今と同じ）。
6. While 1〜3 の条件に当たる問題が無い, the pasta_check shall ほかの誤り（`pasta.toml` が無い・`init.pasta` などのファイル名・モジュール名の重なり・トランスパイルで失敗する書き方・Requirement 2 のフォルダ）があっても、飛び先を検査する。
7. The pasta_check shall 読めない辞書ファイル・`pasta.toml` の誤りそのものは、今までどおり誤りとして報告する。

> 確定（OQ-1）: 省く範囲は「全体」にする。ファイルごとに省く案は、読めないファイルにどのシーンがあったかが分からないので、どの Call が引きずられたかを決められない（brief の推奨）。
>
> 確定（OQ-2）: 2 の `pasta.toml` の誤りも、1 と根が同じ（辞書の集め方・別名表が分からず、検査した辞書のシーンの集まりが pasta.dll と同じとは言えない）なので、省く条件に入れる。`pasta.toml` が無いこと（`config-missing`）は、既定値で集めるのが pasta.dll の決まりと同じなので入れない。brief は 1 だけを挙げていたが、要件ディスカッションで入れると決めた。
>
> 確定（OQ-11）: 案内の行の文言は、例えば「読めない辞書ファイルがあるため、飛び先の検査をしていません。」「構文エラーと pasta.toml の誤りがあるため、飛び先の検査をしていません。」のように、理由を並べた 1 行にする。

### Requirement 2: 辞書のパターンに一致するフォルダ

**Objective:** As a ゴーストの作者, I want 辞書のパターンに一致する名前のフォルダを作ってしまったときに知りたい, so that 検査を通ったのに pasta.dll が読み込みに失敗する、ということが無くなる

#### Acceptance Criteria

1. If 設置ディレクトリの配下に、辞書のパターンに一致するフォルダがある（例 既定のパターンで `dic/x.pasta/`、`dic/x.lua/`）, the pasta_check shall そのフォルダのパスと、pasta.dll がゴーストを読み込めないことを示して、誤りとして報告する。
2. The pasta_check shall フォルダが一致するかを、辞書ファイルの選び出しと同じ規則（pasta.dll と同じ照合・`profile/` を除く・シンボリックリンクを含めない）で判定する。
3. The pasta_check shall 中身の無いフォルダも、一致すれば報告する。
4. The pasta_check shall 一致したフォルダの中のファイルを、今までどおり、パターンに一致すれば辞書ファイルとして集めて検査する。
5. The pasta_check shall 一致したフォルダを、検査した辞書ファイルの数に含めない。
6. The pasta_check shall 辞書のパターンに一致しないフォルダ（例 既定のパターンで `dic/x.txt/`、`dic/sub/`）を報告しない。
7. The pasta_check shall このフォルダの報告を、ほかの報告と同じ 1 行の形（行・列は省く）と同じ並び順で出す。

> 確定（OQ-3）: 分類は**誤り**にする。brief は「pasta.dll はフォルダを読まないので、書いた人の思い違いの可能性が高い」として注意を推奨していたが、pasta.dll のコードを読むと、`glob` がフォルダにも一致し、そのフォルダを辞書として読もうとして失敗し、ゴーストの読み込み全体が失敗する（research.md「論点 b」）。`pasta-check-dic-validate` の確定事項 OQ-1（pasta.dll が読み込みに失敗するものは誤り）と要件 9.4 から、誤りになる。また、名前が `.pasta` で終わるフォルダだけでなく、辞書のパターンに一致するフォルダ全部（`.lua` のパターン・`.pasta` で終わらないパターンを含む）を対象にする。同じ原因の別の形を残さないため。
>
> 確定（OQ-4）: フォルダは、CLI と拡張の両方で、検査の核が判定する（パターンの照合を拡張の TypeScript に写さない。`pasta-check-dic-validate` の確定事項 OQ-6 と design）。そのために、核に渡す一覧にフォルダを含める。入力の形をどう変えるかは設計で決める。

### Requirement 3: エディタで検査するゴーストの範囲

**Objective:** As a ゴーストの作者, I want `Ghost/Master` のように大文字の混じったフォルダのゴーストでも、エディタで保存したときに検査してほしい, so that CLI では検査されるのにエディタでは検査されない、という食い違いが無くなる

#### Acceptance Criteria

1. When 作者が VSCode で、祖先に大文字・小文字を問わず `ghost` → `master` の順に並んだフォルダ（例 `Ghost/Master`・`GHOST/master`）を持つ `.pasta` ファイルを保存した, the VSCode 拡張 shall そのゴーストの全体を検査する。
2. When 大文字・小文字を問わない `ghost/master` の配下で `.pasta`・`.lua` ファイルの作成・削除・名前の変更があった, the VSCode 拡張 shall 拡張が対応する VSCode のどの版でも、そのゴーストの検査をやり直す。
3. When 作者が、大文字・小文字を問わない `ghost/master` の直下にある、大文字・小文字を問わず `pasta.toml` という名前のファイルを保存した, the VSCode 拡張 shall そのゴーストの検査をやり直す。
4. When 作者が、大文字・小文字を問わない `ghost/master` の配下で、拡張子が大文字・小文字を問わず `.pasta` のファイルを保存した, the VSCode 拡張 shall そのゴーストの検査をする（辞書ファイルかどうかは、CLI と同じ選び出しで決める）。
5. When 祖先に `ghost/master` が 2 つ以上ある, the VSCode 拡張 shall 今までどおり、大文字・小文字を問わず、いちばん近いものをゴーストとして選ぶ。
6. The VSCode 拡張 shall 同じゴーストのフォルダが違う大文字・小文字の綴りで届いても、1 つのゴーストとして検査し、同じ問題を二重に表示しない。
7. If 保存・作成・削除したファイルの祖先に、大文字・小文字を問わない `ghost/master` が無い, the VSCode 拡張 shall 今までどおりゴースト全体の検査をしない。

> 確定（OQ-5）: 大文字・小文字は、どの OS でも区別しない。pasta.dll と SSP は Windows で動き、ゴーストは Windows の規則で読まれるため。Linux・macOS の CLI は OS のファイルシステムに従うので、大文字・小文字を区別するファイルシステムの上では、CLI が `Ghost/Master` を「`ghost/master` がありません」とし、エディタは検査する、というずれが残る（CLI は直さない）。
>
> 確定（OQ-6）: 2 の「どの版でも」は、拡張の `engines.vscode`（今は `^1.85.0`）の範囲を指す。VSCode のファイルの見張りは、1.106 までは見張りのパターンを大文字・小文字を区別して照らし、1.107 からは大文字・小文字を区別しないファイルシステムの上で区別しない（research.md「論点 e」）。見張りの書き方（パターンを広げて拡張の側で絞る・`engines` を上げるなど）は設計で決める。

### Requirement 4: CLI とエディタで同じ報告にする

**Objective:** As a ゴーストの作者, I want CLI とエディタで、Requirement 1・2 の報告が同じであってほしい, so that どちらで確かめても同じ直し方になる

#### Acceptance Criteria

1. The 検査する側 shall 同じゴーストに対して、Requirement 1 の飛び先の検査を省く判定と、Requirement 2 のフォルダの報告を、同じ規則・同じ文言で出す。
2. When VSCode 拡張が Requirement 2 のフォルダの誤りを受け取った, the VSCode 拡張 shall そのフォルダに付けて、誤り（Error）として問題パネルに表示する。
3. When 拡張が前の検査で表示したフォルダの誤りが、やり直した検査で報告されなくなった, the VSCode 拡張 shall その診断を消す。
4. When 作者が `pasta_check release` を実行し、配布フォルダに Requirement 2 のフォルダがある, the pasta_check shall 段 4 の関門でほかの誤りと同じく止まり、更新ファイルと nar を作らない。

### Requirement 5: 代用品の置き換えと前方一致の重なり

**Objective:** As a pasta の開発者, I want 検査のコマンドが無かった頃の代用品を、本物の検査に置き換えたい, so that 同じことを確かめる仕組みが 2 つに分かれて、片方だけが古くなることを防げる

#### Acceptance Criteria

1. The マニュアルの CI shall 「Tutorial syntax validation」の段で、hello-pasta（`crates/pasta_sample_ghost/ghosts/hello-pasta`）を `pasta_check validate` で検査し、誤りがあればマニュアルの公開を止める。
2. The マニュアルの CI shall その段のコメントを、`validate` で検査していることに合わせて直す。
3. The pasta_check のテスト shall hello-pasta を検査して、誤りも注意も 0 件であることを確かめる。
4. The 検査コマンド shall グローバルシーンの名前どうしの前方一致の重なり（`＊挨拶` と `＊挨拶朝`、`＊第1章` と `＊第10章`）を、今までどおり報告しない。
5. The hello-pasta のテスト shall 組み込みの飛び先と同じ名前のシーンが無いことを、手書きの名前の一覧で確かめず、Requirement 5.3 の検査（`scene-shadowed-by-builtin` が出ないこと）に任せる。
6. The hello-pasta のテスト shall 入門ガイドの段階辞書の決まり（新しいシーン名を既存のシーン名で始めない・`pasta_shiori` の e2e が足すシーンの接頭辞 `Kick`・`Gate` で始めない）を、今までどおり確かめる。

> 確定（OQ-7）: マニュアルの CI の段は `validate` に置き換える。段の目的は「チュートリアルの辞書の構文が正しいこと」で、それを確かめる道具が `validate` である。LuaJIT のビルドも要らない。段階の辞書を読み込んで応答を確かめるテストは、`build.yml` の `cargo test --all` で、どの変更でも走る。
>
> 確定（OQ-8）: マニュアルの CI は終了コード（誤り）だけで止め、注意 0 件は 3 のテスト（`build.yml` で走る）で守る。注意の扱いをオプションで変える仕組みは足さない。
>
> 確定（OQ-9）: 前方一致の重なりを新しい注意にしない。`pasta-check-dic-validate` の要件 5.3 で「報告しない」と確定しており、入門ガイドの 11 段目は前方一致を機能として教えている（`＞挨拶` が `＊挨拶朝`・`＊挨拶夜` を候補にする）。書いたとおりに動く使い方に注意を出すと、正しい辞書に注意が並ぶ。`＊第1章` と `＊第10章` の問題は、`novel-talk-flow` の完全一致の場面選びが受け持つ。
>
> 確定（OQ-10）: `scene_names_do_not_prefix_collide` は、そのまま置き換えられない。前方一致と `Kick`・`Gate` の接頭辞の検査は hello-pasta の段階辞書の決まり（`hello-pasta-tutorial-stages` の要件 2.5・5.3）で、一般の検査ではないので残す。組み込みの名前の手書きの一覧（`GLOBAL_KEYS`。act のメソッドが入っていない）だけを外し、`validate` の検査に任せる。

### Requirement 6: テストと pasta.dll との一致

**Objective:** As a pasta の開発者, I want この spec の振る舞いをテストで守りたい, so that 後から検査を足したり pasta.dll の規則が変わったりしたときに、ずれに気付ける

#### Acceptance Criteria

1. The pasta_check のテスト shall Requirement 1 の各条件（読めない辞書・UTF-8 で読めない辞書・1.2 の `pasta.toml` の誤り・構文エラー）で、飛び先の無い参照が報告されず、案内の行に理由が出ることを確かめる。
2. The pasta_check のテスト shall Requirement 1.6 の誤りだけがあるときに、飛び先の検査が行われることを確かめる。
3. The 一致テスト shall 辞書のパターンに一致するフォルダがあるゴーストで、pasta.dll の読み込みが失敗し、検査の核がそのフォルダを誤りとして報告することを照らす。
4. The 一致テスト shall 辞書ファイルの選び出しの照らし合わせで、フォルダも比べる（今はファイルだけを比べている）。
5. The VSCode 拡張のテスト shall 大文字・小文字の混じった `ghost/master` の配下の保存・作成・削除・`pasta.toml` の保存で検査が起動し、同じゴーストが二重にならないことを確かめる。
6. The VSCode 拡張のテスト shall フォルダの誤りが問題パネルに出て、直すと消えることを確かめる。

### Requirement 7: 文書

**Objective:** As a ゴーストの作者, I want マニュアルで新しい報告と変わった振る舞いを調べたい, so that 報告を読んで自分で直せる

#### Acceptance Criteria

1. The マニュアル shall `book/src/debug/validate.md` の報告の表に、Requirement 2 のフォルダの報告（分類・意味・直し方）を足す。
2. The マニュアル shall 飛び先の検査を省く条件と案内の行の説明を、Requirement 1 に合わせて直す。
3. The マニュアル shall 「エディタの診断」の節の「検査する時」に、`ghost/master` と `pasta.toml` の名前の大文字・小文字を区別しないことを書く。
4. Where README（`crates/pasta_check/README.md`・`editors/vscode/README.md`）やスキル `pasta-check` が、飛び先の検査を省く条件・報告の種類・検査する時を書いている, the 文書 shall その記述を同じ内容に合わせる。
5. The マニュアル shall 章を足さない（章の数を決め打ちしている検査は変えない）。

## 確定事項（2026-10-17 の要件ディスカッション）

根拠（ファイル・行・確かめた方法）は research.md にある。

- **OQ-1 読めないファイルがあるときに省く範囲**: 確定＝全体を省く（brief の推奨）。
- **OQ-2 `pasta.toml` の誤りも省く条件に入れるか**: 確定＝入れる（`config-missing` は除く）。brief は読めないファイルだけを挙げている。
- **OQ-3 辞書のパターンに一致するフォルダの分類と対象**: 確定＝誤り。対象は `.pasta` で終わる名前に限らず、辞書のパターン（`.lua` を含む）に一致するフォルダ全部。brief の推奨（注意）の前提「pasta.dll はフォルダを読まない」は、コードを読むと当たらない。
- **OQ-4 フォルダをどこで判定するか**: 確定＝検査の核（核に渡す一覧にフォルダを含める）。一覧を集める 2 か所で別々に報告する案は、パターンの照合を TypeScript に写すことになる。入力の形の変え方は設計で決める。
- **OQ-5 大文字・小文字をどの OS でも区別しないか**: 確定＝区別しない（Windows の規則に合わせる）。
- **OQ-6 拡張の見張りが古い VSCode で大文字・小文字を区別すること**: 確定＝`engines` の範囲のどの版でも起動する。書き方は設計で決める。
- **OQ-7 マニュアルの CI の段を `validate` に置き換えるか**: 確定＝置き換える。
- **OQ-8 hello-pasta の注意 0 件をどこで守るか**: 確定＝`pasta_check` のテストで守り、CI の段は終了コードだけで止める。
- **OQ-9 前方一致の重なりを新しい注意にするか**: 確定＝しない。
- **OQ-10 `scene_names_do_not_prefix_collide` を置き換えるか**: 確定＝組み込みの名前の部分だけを外し、hello-pasta の決まりの部分は残す。
- **OQ-11 案内の行の文言**: 確定＝理由を並べた 1 行。
