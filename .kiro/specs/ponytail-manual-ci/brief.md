# Brief: ponytail-manual-ci

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。版には縛らない。0.4.2 を出した後、0.5.0 の spec が並べられずに空いた所へ入れる（2026-10-22 開発者の決定。2026-10-19 の「0.4.3 に入れる」から改めた）。同じ監査の 10 本と同じウェーブで並走できる。

## 進め方（2026-10-19 開発者の決定）

- 下の「Current State」は、2026-10-19 の監査の控え（出発点）である。
- **ギャップ分析**で、この spec の領域に `/ponytail:ponytail-audit` をやり直す。
  - 領域：`book/tools`、`book/theme` の JS、`.github/workflows`、`.github/scripts/release/*.ps1`（前回は読んでいない）。
- 控えと突き合わせ、違いをギャップ分析の報告に書く。
- 要件は「その領域の監査の指摘が、残らず直っている」ことと、決めごとにする。
- **修正は設計以降**で行う。

## Problem

開発者とマニュアルの読者が困る。

- Rust のファイルの名前を変える PR では、マニュアルが引くパスの検査が走らない。そのため main が後で赤くなり、どの PR が原因かが分からない。
- リリースの関門でも、マニュアルの 3 つの検査が走らない。
- Pages のデプロイの権限が、第三者のコードを走らせる job まで広がっている。
- 公開の資格を持つ job が、可変のタグの action を使っている。

## Current State

2026-10-19 の監査で確かめた。手元の自己テストは全部通る（`link-check-test`・`gen-skill-refs-test`・`tutorial-check-test`・`talk-test`・`theme-test`）。

1. **マニュアルの検査が、Rust の PR とリリースの関門で走らない**（Must fix）
   - 場所：`manual.yml` L26-46 の path filter、`link-check.mjs` L388-414。
   - `book/src/internals` は、392 のリポジトリのパス（たとえば `crates/pasta_core/src/actor_font.rs`）を引いている。
   - ところが、名前を変える PR は緑のまま入る。
   - リリースの関門（`release.yml` が `build.yml` を呼ぶ）でも、次の 3 つは走らない。
     - `gen-skill-refs --check`
     - `link-check`
     - `tutorial-check`
   - 3 つとも npm の依存が無く、1 秒未満で終わる。
2. **Pages の権限が広すぎる**
   - 場所：`manual.yml` L56-59・L191-197。
   - `pages: write`・`id-token: write` が、ワークフロー全体に付いている。そのため、`npm ci`（install scripts）を走らせる build の job にも付く。
   - `configure-pages` の出力は使われていない。`enablement: true` は GITHUB_TOKEN では効かず、コメントも誤っている。
3. **公開の資格を持つ job で、第三者の action が可変のタグを使う**
   - 場所：`release.yml` L244・L247・L253-293・L395、`release-setup-check.yml` L43。
   - 対象：`Swatinem/rust-cache@v2`・`dtolnay/rust-toolchain@stable`・`rust-lang/crates-io-auth-action`・`Azure/login@v3`。
4. **重複したビルド**
   - 場所：`manual.yml` L129-144 の Tutorial syntax validation。
   - Rust と cache を用意し、`pasta_check` をビルドし直す。これは `build.yml` の `validate_cli_test::test_validate_hello_pasta_clean` と重なる。
5. **文法の変更で、ハイライトのテストもマニュアルの再公開も走らない**
   - `editors/vscode/syntaxes/**` が、path filter に入っていない。
6. **`head.hbs` の neutralizer の写しを、誰も比べない**
   - 場所：`head.hbs` L198-287、`neutralizer.mjs`。
   - tokenizer の写しは、`verify-search.mjs` L346-405 が比べている。
7. **`highlight-html.mjs` が、色を 1 つも付けなくても exit 0**
   - 場所：`highlight-html.mjs` L257-269。
8. **bigram の PoC のテストと fixture が、`build-index-test` と重複する**
   - 場所：`poc-test.mjs`・`poc/`（5 ファイル）。
   - `build-index-test` の RED は、CI では意味が無い。
9. **`talk.mjs` が、`gen-skill-refs` の `chapterRegions` の写しを持つ**
   - 場所：`talk.mjs` L262-274。
10. **Node 20 は寿命が切れている**
    - 場所：`manual.yml` L75・`release.yml` L25・`release-setup-check.yml` L35。
11. **CI の mdBook 0.5.3 と、テストの前提の 0.5.4 がずれている**
    - roadmap のバックログにも既出。場所は `manual.yml` L92・`tech.md:220`。

## Desired Outcome

- マニュアルの 3 つの検査が、全部の push・PR とリリースの関門で走る。
- Pages の権限は、deploy の job だけが持つ。
- 公開の資格を持つ job の action は、commit の SHA で固定する（タグは末尾のコメントに残す）。
- 重複したビルドと PoC が無くなり、写しは検査されるか、1 つに寄せられる。
- 色の付かないハイライトは、CI を落とす。
- Node と mdBook の版が揃う。

## Approach

監査の最小の直しを採る。

- `build.yml` に、ubuntu の job を 1 つ足し、3 つの検査を回す。
- `manual.yml` の権限は、ワークフロー全体を `contents: read` にし、deploy の job にだけ `pages`・`id-token` を付ける。Setup Pages の段は消す。
- 写しの検査は、BEGIN と END の間を比べる 12 行ほどのコードで足りる。

## Scope

- **In**: 上の 1〜11 と、ギャップ分析で増えた同じ領域の指摘。
- **Out**:
  - マニュアルの本文
  - 章の数を決め打ちする 3 つの検査（受け入れ済みの約束）

## Boundary Candidates

- CI の検査の配置（`build.yml`・`manual.yml` の path filter）
- 権限と固定（`manual.yml`・`release.yml`・`release-setup-check.yml`）
- `book/tools` の写しと PoC の整理

## Out of Boundary

- Rust のクレート
- `editors/vscode` の文法そのもの（`ponytail-editor-tooling`）

## Upstream / Downstream

- **Upstream**: GitHub Actions、mdBook
- **Downstream**: GitHub Pages のマニュアル、リリース（`release` スキル）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `ponytail-editor-tooling`（tmLanguage を変える。このspec の path filter が入っていれば、マニュアルが再公開される）
  - `ponytail-check-release`（`release.yml` は触らない）
  - `pasta-check-release-asset`（0.4.2・先に入る）: `release.yml` の build job に x64 のターゲット・`pasta_check.zip` の検査と upload を足し、`github-release.ps1` の添付を 5 つにする。監査のやり直しと修正は、その後の形に合わせる。

## Constraints

- PR は CI を待たずにマージしてよい（関門はローカルの test と clippy）。CI の変更は、途中で `gh workflow run build.yml --ref <branch>` で回して確かめる。draft の PR は分類器が止める。
- Pages の最初の有効化は手動である。この spec では変えない。
- `tech.md` の mdBook の版の行（:220）を直す。`ponytail-editor-tooling` が触る行とは別である。
- ワークツリーに `book/node_modules` が無いことがある。在るかを先に `ls` で確かめる。

## 申し送り（pasta-check-release-asset より）

2026-10-22 に main に入った変更。監査と修正はこの形を前提にする（詳しくは `.kiro/specs/completed/pasta-check-release-asset/design.md` の ReleaseBuildJob・GitHubReleaseAssets・ReleaseScriptsTest の節）。

- `release.yml` の build job: toolchain の targets に `x86_64-pc-windows-msvc` を足し、rust-cache の key を `i686-x86_64-pc-windows-msvc` に変えた。「Verify release assets」に `pasta_check.zip` の中身（ちょうど 2 つ）と `pasta_check.exe --version` ＝ `pasta_check <タグの版>` の検査を足し、upload の path に `release/pasta_check.zip` を足した。verify・gate・公開系の job と `Cargo.lock` の検査は変えていない。
- `github-release.ps1` の添付は 5 つ（配布物 4 つと詳しいリリースノート）。`release-scripts-test.ps1` の項目 10 が、`pasta_check.zip` だけが無いときに gh より前で終了コード 1 になることを確かめる（`github-release.ps1` は今はこのテストで試されている）。
- 見直しの論点: 合わせるときに上の行を保つこと（本 spec の Revalidation Triggers）。

## 2026-10-23 棚卸の再測定（main 8fc9bdff）

- **前提の変化**
  - pasta_check を単体で配る作業（pasta-check-release-asset）が 2026-10-22 に入った。この spec が先に入るのを待つ理由は無くなった。
  - リリースの流れ `.github/workflows/release.yml` は 589 行になり、公開の資格を持つ job の行番号が 15 行ずれた（ツールの用意 259 行・キャッシュ 262 行・crates.io の認証 268〜308 行・Azure のログイン 410 行）。build の job（73・78 行）にも同じ可変のタグの action があるが、資格を持たない job なので、固定するかはギャップ分析で決める。
  - `github-release.ps1` は添付が 5 つになり、リリースの補助スクリプトのテスト `release-scripts-test.ps1`（項目 10）が試すようになった。このテストは `build.yml` の最後の段（149 行）で走る。
  - `manual.yml`・`release-setup-check.yml`・`book/tools`・`book/theme` は変わっていない。Node 20 は `manual.yml` 75 行・`release.yml` 25 行（共通の値 `NODE_VERSION`）・`release-setup-check.yml` 35 行。mdBook 0.5.3 は `manual.yml` 92 行と `tech.md` 221 行（brief の 220 行から 1 行ずれた）。
- **触るファイル**: `.github/workflows/` の `build.yml`（149 行）・`manual.yml`（216 行）・`release.yml`（589 行）・`release-setup-check.yml`（122 行）、`book/tools/`（`highlight/highlight-html.mjs`・`talk/talk.mjs`・`bigram-index/poc-test.mjs`・`bigram-index/poc/`・`bigram-index/build-index-test.mjs` と、写しを比べる小さな検査）、`book/theme/head.hbs`、`tech.md`。`.github/scripts/release/*.ps1` はギャップ分析の監査の対象（前回は読んでいない）。1,000 行に近いものは無い。
- **規模**: 11〜14 タスク。
- **先に要るもの**
  - 機能の上では無い。Specs の一覧の `Dependencies: pasta-check-release-asset` は、もう満たされている。合わせるときは、build の job の x64 のターゲット・キャッシュのキー `i686-x86_64-pc-windows-msvc`・`pasta_check.zip` の検査と upload の行・添付 5 つを保つ（申し送りのとおり）。
  - 0.4.2 の残りと 0.5.0 の spec は `.github/` を触らない。重なりうるのは `book/tools` だけ。文法定義を変える spec（`call-attribute-filter`・`scene-exact-match`・`scene-anchor-link`）は、マニュアルの色付けのテスト（`book/tools/highlight/` の `tokenizer-test.mjs` など）を合わせることがある。この spec が触るのは `highlight-html.mjs` の終わりの判定（257〜269 行）で、別のファイル。
  - 章を足す spec（`novel-areka-contract` が足すなら）は、章の数を決め打ちする 3 つの検査を直す。この spec の写しの整理（`talk.mjs` の `chapterRegions`）が `gen-skill-refs.mjs`・`gen-skill-refs-test.mjs` に及ぶなら、ぶつかる。
  - 0.4.2 のリリースの間は、`release.yml` を main に入れない（リリースの席）。
- **種別**: 基盤（検査の置き場所・権限・action の版の固定。Rust のファイルの名前を変える PR で main が後から赤くなる穴を塞ぐ）。
- **要件定義のモデル**: Opus。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - 3 番の行番号（L244・L247・L253-293・L395）は、上のとおり 15 行ずれた。
  - `manual.yml` 191〜197 行の Setup Pages の注記（`enablement: true` で Pages が有効になる）は誤り（2 番。直すのは設計以降）。

## 申し送り（pasta-skills-release-asset より）

- `release.yml` の build の job の「Setup Node」の直後に、step「Check generated skill references」（`node book/tools/gen-skill-refs.mjs --check`）を足した。タグの時点で配るスキルの写しが最新であることを、長いビルドの前に保証するためである。関門（gate）に同じ照合を足すときは、この step との重複を整理する（どちらに残すかは本 spec の論点）。
- 配布物は 5 つ（`pasta.dll.zip`・`hello-pasta.nar`・VSIX・`pasta_check.zip`・`pasta-skills.zip`）、GitHub Release の添付は 6 つ（配布物 5 つと詳しいリリースノート）になった。上の申し送りの「添付 5 つ」「配布物 4 つ」は古い。合わせるときは、build の job の `pasta-skills.zip` の検査（「Verify release assets」の項目と、`git ls-tree` の失敗で止まる 1 行）・upload の path の行・`github-release.ps1` の `$names` の 6 つ・`release-scripts-test.ps1` の項目 11 を保つ。
- `release.yml` の行は、build の job の中で 19 行ほど下へずれた。参照先: `.kiro/specs/completed/pasta-skills-release-asset/design.md` の「ReleaseBuildJob」「GitHubReleaseAssets」。
