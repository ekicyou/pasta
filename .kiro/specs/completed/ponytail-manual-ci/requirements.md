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
  - 章の数を決め打ちする自己テスト 3 つ（決定 A。下の Requirement 9）。
  - 配るスキル `pasta-check` の、`crates/` を引く 6 行の書き直し（決定 C。スキルの中身の検査に通すため）。
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
6. The Release ワークフロー shall スキルの生成物の鮮度の照合を、関門（Build ワークフロー）でだけ走らせる。build の job の照合の段は持たない（決定 B。build の job は関門の後に走るので、長いビルドの前の保証は保たれる）。

### Requirement 2: マニュアルのパイプラインが、関係するファイルの変更で起動する

**Objective:** As a 開発者, I want マニュアルの出力を変えるファイルを変えたとき、マニュアルのパイプラインが走ること, so that 文法の変更でハイライトのテストが走り、公開のマニュアルも作り直される。

#### Acceptance Criteria

1. When `editors/vscode/syntaxes/` の下のファイルを変える push か PR が起きたとき, the Manual ワークフロー shall 起動し、ハイライトの検査とテストを走らせる。
2. When `editors/vscode/syntaxes/` の下のファイルを変える push が main に入ったとき, the Manual ワークフロー shall マニュアルを作り直して公開する。
3. When 配るスキル `pasta-check` のファイルを変える push か PR が起きたとき, the Manual ワークフロー shall 起動し、リンクの検査を走らせる（決定 C）。
4. The Manual ワークフロー shall 起動の条件のコメントと `tech.md` の起動の条件の行を、実際の条件と合わせる。
5. The リンクの検査 shall 配るスキル `pasta-check` を、ほかの配るスキルと同じ検査（相対リンク・見出し・公開マニュアルの URL・`crates/` などを引かない自己完結）にかける（決定 C）。
6. The 配るスキル `pasta-check` shall リポジトリの中のパス（`crates/` の下）を引かない。今の 6 行（`SKILL.md` の 3 行・`nar-spec.md` の 1 行・`updates-txt-spec.md` の 2 行）は、公開の URL に変えるか消す（決定 C）。

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
4. The Release ワークフロー shall 配布物を作る job（build の job）の第三者の action も、commit の SHA で指す（決定 D）。
5. If 配布物を作る job がネットから取って来たビルドの道具（`wasm-pack`・`cargo-about`）の SHA-256 が、決めた値と違うとき, the Release ワークフロー shall その job を止める（決定 D）。
6. When 固定した action で公開の job が走るとき, the Release ワークフロー shall 今と同じ公開の結果（status・reason・summary）を出す。
7. The Build ワークフローの action と、GitHub 自身の action（`actions/*`） shall タグのままにする（決定 D。資格も配布物も持たない、または GitHub が持つため）。

### Requirement 5: 重複したビルドと PoC を無くす

**Objective:** As a 開発者, I want 同じことを確かめる検査が 1 つだけであること, so that 片方だけ直して食い違うことが無く、CI の時間も短い。

#### Acceptance Criteria

1. The Manual ワークフロー shall `pasta_check` をビルドし直して hello-pasta を検査することを、しない。hello-pasta の検査は Build ワークフローのテストが受け持つ。
2. The 自己テスト shall bigram の PoC のテストと fixture を持たない。
3. When PoC を消すとき, the 自己テスト shall PoC だけが持っていた tokenizer の単体の検査を、残る自己テストで続けて確かめる。
4. The bigram の索引の自己テスト shall CI で意味のある検査だけを持つ。CI の状態で、正しい理由で失敗しない検査（たとえば、もう bigram の索引に対して既定の分かち書きで 0 件を確かめる検査）を持たない。
5. The 台詞部品の集計 shall 章の導入と締めの区切りを、スキルの生成器と同じ 1 つの規則で決める。写しを持たない。
6. The 検索の自己テスト shall 検索の設定を、索引に入った値から取る。ブラウザの検索と違う設定を書き写して持たない。
7. The 検索の自己テスト shall 索引を引く補助の関数（URL の一覧・結果・本文の集め方）を、1 か所に持つ。テストごとに写しを持たない。

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
6. If 監査の領域のワークフローが使う action が、寿命の切れた Node（20）の実行環境で動く版のとき, the ワークフロー shall その action を Node 22 以降で動く版に上げる（決定 4。どの action が当たるかは設計で調べる）。

### Requirement 9: 章を足しても、自己テストを直さずに済む（決定 A）

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
6. The リリースの補助スクリプト shall 誰も使わない引数を持たない。`publish-crate.ps1` の `-ApiBase` は消す（決定 E）。
7. When CI の変更をブランチで確かめたとき, the Build ワークフローと Manual ワークフロー shall そのブランチの上で成功する。
8. The `book/tools` のスクリプト shall 外から使われない export を持たない。同じ値（公開マニュアルの URL）と同じ処理（Markdown のファイルの一覧）を、2 か所で決めない。
9. When `release-scripts-test.ps1` が `publish-vsix.ps1` に `-VsceCommand` で偽の vsce を渡し、その版が公開済みと答えさせたとき, the `publish-vsix.ps1` shall 公開せずに skipped を出す。この項目はネットワークを使わない（決定 E）。

## 決定（2026-11-02 の要件の議論。全議題を推奨案で確定）

- **決定 A**（議題 1）: 章の数を決め打ちする自己テスト 3 つ（brief の Scope では Out だった）を、この spec で直す。`SUMMARY.md` から数え、ファイルの数と `verify-content` が見た章の数と比べる（Requirement 9）。比べるので、`VOICE_DIRS` に無いフォルダを足したときの守りは減らない。
- **決定 B**（議題 2）: スキルの生成物の鮮度の照合は、関門（Build ワークフロー）にだけ置く。`release.yml` の build の job の照合の段（L119-121）は消す。`pasta-skills-release-asset` の申し送りの「保つ行」から、この段だけを外す（Requirement 1.6）。
- **決定 C**（議題 5）: 配るスキル `pasta-check` を、ほかの配るスキルと同じリンクの検査（自己完結まで）にかけ、`crates/` を引く 6 行を直す（Requirement 2.3・2.5・2.6）。`.claude/skills` の編集は分類器が止めるので、実装のときに開発者の許可を求める。
- **決定 D**（議題 3）: 固定するのは、公開の資格を持つ job と、配布物を作る `release.yml` の build の job の第三者の action と、ビルドの道具 2 つの SHA-256。`build.yml`（`build-unix` を含む）と `actions/*` はタグのまま（Requirement 4）。
- **決定 4**（議題 4）: Node 20 の寿命の始末は、`node-version` の 2 か所と、Node 20 の実行環境で動く action の major の引き上げ（要るなら）の両方を、この spec で行う（Requirement 8.1・8.2・8.6）。
- **決定 E**（議題 6）: `-ApiBase` は消す。`-VsceCommand` は、偽の vsce で「公開済みなら skipped」を確かめる `release-scripts-test.ps1` の項目に使う（Requirement 10.6・10.9）。
