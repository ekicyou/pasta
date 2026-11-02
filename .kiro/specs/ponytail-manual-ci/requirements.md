# Requirements Document

## Project Description (Input)
開発者とマニュアルの読者が困っている。Rust のファイルの名前を変える PR では、マニュアルが引くパスの検査が走らず、main が後で赤くなって原因の PR が分からない。リリースの関門でも、マニュアルの検査が走らない。Pages のデプロイの権限が、第三者のコードを走らせる job まで広がっている。公開の資格を持つ job が、可変のタグの action を使っている。ほかに、重複したビルドと PoC、誰も比べない写し、色を付けなくても通るハイライト、寿命の切れた Node 20、ずれた mdBook の版がある（2026-10-19 の `/ponytail:ponytail-audit` の 11 の指摘。詳しくは brief.md）。

この領域（`book/tools`、`book/theme` の JS、`.github/workflows`、`.github/scripts/release/*.ps1`）の監査の指摘が、残らず直っている状態にする。マニュアルの検査は全部の push・PR とリリースの関門で走り、Pages の権限は deploy の job だけが持ち、公開の資格を持つ job の action は commit の SHA で固定する。修正は設計以降で行う。

## Introduction

この spec は、マニュアルの CI とリリースの CI の監査の指摘を、残らず直す。

- 監査の領域は 4 つある。`book/tools` の全部の `.mjs`、`book/theme` の JS（`head.hbs`・`claudia.js`）、`.github/workflows/*.yml`、`.github/scripts/release/*.ps1`。
- 指摘の一覧は、`research.md` の「指摘の台帳」にある。2026-10-19 の 11 件（B1〜B11）と、2026-11-02 のギャップ分析で増えたもの（N1〜）である。
- 要件は「台帳の指摘が、残らず直っている」ことを決める。直し方は設計で決める。
- 直した後も、今ある検査の守りは減らさない。

「マニュアルの軽い検査」は次の 3 つを指す。どれも npm の依存が無く、1 秒ほどで終わる。

- スキルの生成物の鮮度の照合（`gen-skill-refs --check`）
- リンクの検査（`link-check`）
- チュートリアルの照合（`tutorial-check`）

## Boundary Context

- **In scope**:
  - 台帳の指摘（B1〜B11 と N1〜）の修正。
  - 指摘を直すのに要る、`tech.md` の CI とマニュアルの行（path filter・権限・mdBook の版）の書き直し。
  - 章の数を決め打ちする自己テスト 3 つ（前提 A。下の Requirement 9）。
- **Out of scope**:
  - マニュアルの本文（`book/src` の章の中身）。
  - Rust のクレート。
  - `editors/vscode` の文法そのもの（`ponytail-editor-tooling` が持つ）。
  - Pages の最初の有効化（手動のまま）。
  - CI の通知や、マージを CI で止める仕組み（PR は CI を待たずにマージする運用のまま）。
- **Adjacent expectations**:
  - `ponytail-editor-tooling` は tmLanguage を変える。この spec の path filter が入れば、文法の変更でマニュアルが作り直される。`tech.md` は別の行を触る。
  - `vsce-4-upgrade` は、この spec が `release.yml`・`release-setup-check.yml` の Node を 22 にするのを待つ。
  - `ponytail-check-release` は `release.yml` を触らない。
  - `pasta-check-release-asset`・`pasta-skills-release-asset` が `release.yml`・`github-release.ps1`・`release-scripts-test.ps1` に入れた行は保つ（Requirement 10）。
  - リリースの実行中は、`release.yml` の変更を main に入れない。

## Requirements

### Requirement 1: マニュアルの軽い検査が、全部の push・PR とリリースの関門で走る

**Objective:** As a 開発者, I want 変えたファイルによらず、マニュアルの軽い検査が毎回走ること, so that Rust のファイルの名前を変えた PR が、その PR の中で赤くなり、原因の PR が分かる。

#### Acceptance Criteria

1. When main への push か、main への PR が起きたとき, the Build ワークフロー shall 変えたファイルによらず、マニュアルの軽い検査を 3 つとも走らせる。
2. When リリースタグの push でリリースの関門が走るとき, the Release ワークフロー shall 配布物を作る前に、マニュアルの軽い検査を 3 つとも走らせる。
3. If マニュアルの軽い検査のどれかが失敗したとき, the Build ワークフロー shall その実行を失敗にし、失敗した検査の名前と理由をログに出す。
4. If 内部設計の章が引くリポジトリのパスが無くなったとき, the リンクの検査 shall そのパスを壊れたものとして報告する。これは `editors/` の下のパスでも同じである。
5. The Build ワークフロー shall マニュアルの軽い検査を、Rust のビルドの終わりを待たずに走らせる。
6. The Release ワークフロー shall 同じタグの 1 回の実行で、スキルの生成物の鮮度の照合を 2 度走らせない（前提 B。関門に照合が入れば、build の job の照合は重複になる）。

### Requirement 2: マニュアルのパイプラインが、関係するファイルの変更で起動する

**Objective:** As a 開発者, I want マニュアルの出力を変えるファイルを変えたとき、マニュアルのパイプラインが走ること, so that 文法の変更でハイライトのテストが走り、公開のマニュアルも作り直される。

#### Acceptance Criteria

1. When `editors/vscode/syntaxes/` の下のファイルを変える push か PR が起きたとき, the Manual ワークフロー shall 起動し、ハイライトの検査とテストを走らせる。
2. When `editors/vscode/syntaxes/` の下のファイルを変える push が main に入ったとき, the Manual ワークフロー shall マニュアルを作り直して公開する。
3. Where リンクの検査が配るスキル `pasta-check` を対象に含めるとき, the Manual ワークフロー shall `pasta-check` のスキルの変更でも起動する（前提 C）。
4. The Manual ワークフロー shall 起動の条件のコメントと `tech.md` の起動の条件の行を、実際の条件と合わせる。

### Requirement 3: Pages の権限は deploy の job だけが持つ

**Objective:** As a 開発者, I want Pages へ書く権限が、公開する job にだけあること, so that 第三者のコードを走らせる build の job が、Pages の権限を持たない。

#### Acceptance Criteria

1. The Manual ワークフロー shall ワークフロー全体の権限を、リポジトリを読むことだけにする。
2. The Manual ワークフロー shall Pages へ書く権限と OIDC のトークンを作る権限を、deploy の job にだけ与える。
3. While build の job が npm の依存を入れて走るとき, the Manual ワークフロー shall その job に Pages へ書く権限も OIDC のトークンを作る権限も与えない。
4. The Manual ワークフロー shall 出力を誰も使わない step と、効かない設定を持たない。コメントは実際の動きと合う。
5. When main への push でマニュアルを公開するとき, the Manual ワークフロー shall 今と同じ URL に、今と同じ中身（新版と旧版の `classic/`）を公開する。
6. The `tech.md` shall Manual ワークフローの権限の行を、実際の権限と合わせる。

### Requirement 4: 公開の資格を持つ job の action は、commit の SHA で固定する

**Objective:** As a 開発者, I want 公開の資格を持つ job の第三者の action が、動かない版に固定されていること, so that タグを付け替えられても、資格を持つ job で別のコードが走らない。

#### Acceptance Criteria

1. The Release ワークフロー shall crates.io と Marketplace に公開する job で使う第三者の action を、commit の SHA で指す。
2. The Release setup check ワークフロー shall OIDC で Azure に入る job で使う第三者の action を、commit の SHA で指す。
3. The 固定した action の行 shall 元のタグを、行の末尾のコメントに残す。
4. Where 配布物を作る job（Release ワークフローの build の job）も固定の対象に含めるとき, the Release ワークフロー shall その job の第三者の action も commit の SHA で指す（前提 D）。
5. Where 配布物を作る job がビルドの道具をネットから取って来るとき, the Release ワークフロー shall 取って来たファイルが決まった中身であることを確かめ、違えば止める（前提 D）。
6. When 固定した action で公開の job が走るとき, the Release ワークフロー shall 今と同じ公開の結果（status・reason・summary）を出す。

### Requirement 5: 重複したビルドと PoC を無くす

**Objective:** As a 開発者, I want 同じことを確かめる検査が 1 つだけであること, so that 片方だけ直して食い違うことが無く、CI の時間も短い。

#### Acceptance Criteria

1. The Manual ワークフロー shall `pasta_check` をビルドし直して hello-pasta を検査することを、しない。hello-pasta の検査は Build ワークフローのテストが受け持つ。
2. The 自己テスト shall bigram の PoC のテストと fixture を持たない。
3. When PoC を消すとき, the 自己テスト shall PoC だけが持っていた tokenizer の単体の検査を、残る自己テストで続けて確かめる。
4. The bigram の索引の自己テスト shall CI で意味のある検査だけを持つ。CI の状態で、正しい理由で失敗しない検査（たとえば、もう bigram の索引に対して既定の分かち書きで 0 件を確かめる検査）を持たない。
5. The 台詞部品の集計 shall 章の導入と締めの区切りを、スキルの生成器と同じ 1 つの規則で決める。写しを持たない。
6. The 検索の自己テスト shall 検索の設定を、索引に入った値から取る。ブラウザの検索と違う設定を書き写して持たない。

### Requirement 6: 写しは検査される

**Objective:** As a 開発者, I want ページに埋め込んだ写しが、正準のファイルと食い違えば CI が落ちること, so that 写しだけが古くなって、マニュアルのハイライトが黙って崩れることが無い。

#### Acceptance Criteria

1. If `head.hbs` の neutralizer の写しが、正準の `neutralizer.mjs` と食い違うとき, the 自己テスト shall 失敗し、食い違う場所を示す。
2. If 作ったページに neutralizer の写しが入っていないとき, the 自己テスト shall 失敗する。
3. The 目次の開閉の条件（画面の幅のしきい値） shall 1 か所で決まるか、`head.hbs` と `claudia.js` の 2 か所が食い違えば自己テストが失敗する。
4. The 自己テスト shall tokenizer の写しを、今と同じく比べ続ける。

### Requirement 7: 検査が空回りしない

**Objective:** As a 開発者, I want 何も確かめていない検査が、通らないこと, so that 「緑だから大丈夫」が本当になる。

#### Acceptance Criteria

1. If マニュアルの着色が `language-pasta` のブロックを 1 つも着色しなかったとき, the 着色の段 shall 終了コード 0 以外で終わり、Manual ワークフローを止める。
2. The 本文の検査 shall コードフェンスの範囲を、外側のフェンスが 4 つ以上のバッククォートのときも正しく決める。フェンスの外の見出しと文章を、フェンスの中とみなさない。
3. The リンクの検査 shall 章のコードフェンスの中に書いた例のリンクを、リンクとして検査しない。
4. The 自己テスト shall Pages に公開するために作った出力を、書き換えない。
5. The 分かち書き（正準の `tokenize.mjs` と `head.hbs` の写し） shall 決して通らない分岐（UTF-16 の 1 単位では起きない文字の範囲の判定）を持たない。両方を同じに直す。

### Requirement 8: Node と mdBook の版がそろう

**Objective:** As a 開発者, I want CI の Node と mdBook の版が、手元とテストの前提と同じであること, so that 寿命の切れた Node で動かず、テストの前提と違う mdBook で公開しない。

#### Acceptance Criteria

1. The Release ワークフロー shall Node 22 を使う。
2. The Release setup check ワークフロー shall Node 22 を使う。
3. The Manual ワークフロー shall 自己テストが前提にする mdBook の版（0.5.4）で、マニュアルと旧版を作る。
4. The `tech.md` shall mdBook の版の行を、CI の版と合わせる。
5. When Node と mdBook の版を変えた後で Release ワークフローと Manual ワークフローが走るとき, the ワークフロー shall 今と同じ配布物と、今と同じマニュアルを作る。

### Requirement 9: 章を足しても、自己テストを直さずに済む（前提 A）

**Objective:** As a 開発者, I want 章を足すたびに自己テストの章の数を書き直さずに済むこと, so that 章を足した PR で main が赤くなることが、3 度目に起きない。

#### Acceptance Criteria

1. When `SUMMARY.md` に載せた章を 1 つ足したとき, the 自己テスト shall 書き直さずに通る。
2. If `book/src` の章のファイルが `SUMMARY.md` に載っていないか、`SUMMARY.md` に載った章のファイルが無いとき, the 自己テスト shall 失敗し、そのファイルを示す。
3. If 章が、口調の検査と台詞部品の検査が見るフォルダの外にあるとき, the 自己テスト shall 失敗し、その章を示す。
4. The 自己テスト shall 章の総数を、数字で決め打ちしない。

### Requirement 10: 台帳の指摘が残らず直り、今ある守りは減らない

**Objective:** As a 開発者, I want 監査の台帳の指摘が全部直り、今の検査とリリースの形が保たれること, so that 直した後の CI を、次の監査の出発点にできる。

#### Acceptance Criteria

1. When この spec を完了とするとき, the 指摘の台帳（`research.md`） shall すべての指摘を「直した」か「外した（要件の議論で開発者が決めた理由つき）」にしている。
2. The 監査の領域の全部の自己テスト shall 直した後に通る。`book/node_modules` が無いワークツリーでは、npm の依存が要るテストを、依存を入れた環境か CI で確かめる。
3. The Release ワークフロー shall 配布物 5 つ（`pasta.dll.zip`・`hello-pasta.nar`・VSIX・`pasta_check.zip`・`pasta-skills.zip`）の検査と upload を、今のとおり保つ。`pasta-skills.zip` の中身の検査と、`git ls-tree` の失敗で止まる行も保つ。
4. The `github-release.ps1` shall 添付の名前 6 つ（配布物 5 つと詳しいリリースノート）を、今のとおり保つ。
5. The `release-scripts-test.ps1` shall 項目 10（`pasta_check.zip` だけが無い）と項目 11（`pasta-skills.zip` だけが無い）を保ち、直した後も全部の項目が ok になる。
6. The リリースの補助スクリプト shall 誰も使わない引数を持たない。テストのための差し替えの口を持つなら、それを使うテストがある（前提 E）。
7. When CI の変更をブランチで確かめたとき, the Build ワークフローと Manual ワークフロー shall そのブランチの上で成功する。

## 前提（要件の議論で確かめる）

- **前提 A**: 章の数を決め打ちする自己テスト 3 つ（brief の Scope では Out）を、この spec で直す。2026-11-02 の再測定の推奨（`SUMMARY.md` から数えて比べる）に沿う。採らなければ Requirement 9 を外し、brief の Out に戻す。
- **前提 B**: スキルの生成物の鮮度の照合は、関門（Build ワークフロー）に置き、Release ワークフローの build の job の照合は消す。同じタグのコミットで、関門は build の job より先に必ず走るため。
- **前提 C**: 配るスキル `pasta-check` を、リンクの検査（相対リンク・見出し・公開マニュアルの URL）の対象に足す。スキルの自己完結の検査（`crates/` などを引かない）まで足すかは、スキルのファイル 6 行を直すことになるので、議論で決める。
- **前提 D**: 固定の対象は、公開の資格を持つ job に加えて、配布物を作る Release ワークフローの build の job も含める。Build ワークフロー（資格も配布物も持たない）の action はタグのままにする。GitHub 自身の action（`actions/*`）はタグのままにする。
- **前提 E**: 使われない引数（`publish-crate.ps1` の `-ApiBase`・`publish-vsix.ps1` の `-VsceCommand`）は、テストで使うか消す。どちらにするかは設計で決める。
