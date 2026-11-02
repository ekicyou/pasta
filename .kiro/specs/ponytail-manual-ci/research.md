# ギャップ分析: ponytail-manual-ci

## Summary

- **Feature**: `ponytail-manual-ci`
- **Discovery Scope**: Extension（今ある CI と検査ツールを直す。新しい仕組みは足さない）
- **測った時点**: main `ae04a28b`（2026-11-02）。brief の再測定（`491cc23c`）の後に、章の数を 64 にそろえる直し（`e9847974`）が入っている。
- **Key Findings**:
  - 控えの 11 件のうち、10 件は残っている。B10（Node 20）は `manual.yml` だけ直った（PR #187）。`verify-content.mjs` の「全 62 章」の注記も直った。
  - 監査のやり直しで、新しい指摘が 17 件見つかった（N1〜N17）。大きいのは、本文の検査がコードフェンスを取り違えること（N1）、検索のテストがブラウザと違う設定で検索すること（N2）、配布物を作る job の action と道具が固定されていないこと（N7・N8）である。
  - マニュアルの軽い検査 3 つは、`book/node_modules` も `book/book` も無いワークツリーで、どれも 1 秒未満で通った（`gen-skill-refs --check` 0.2 秒・`link-check` 0.6 秒・`tutorial-check` 0.1 秒）。cargo の要らない小さな job に載せられる。
  - 直しはどれも小さい。消す行が足す行より多い。
  - 設計（2026-11-02）で、Research Needed の 1〜3 を `gh api` で解き、4 は mdBook の生成と索引まで開発機で確かめた。検索の包含の検査が OR の設定では成り立たないこと（N2 の直しが少し増える）と、Pages の environment が main だけを許すこと（公開の条件を直す）が新しく分かった。末尾の「設計フェーズの調査」を参照。

## 監査の条件

- **領域**: `book/tools` の全部の `.mjs`（`highlight/`・`talk/`・`bigram-index/`・`bigram-index/poc/` を含む）、`book/theme/head.hbs` と `book/theme/claudia.js` の JS、`.github/workflows/*.yml`（4 つ）、`.github/scripts/release/*.ps1`（6 つ）。
- **想定した負荷**: 開発者 1 人のリポジトリ。CI は PR と main への push のたびに走る。リリースはタグの push で月に数回。性能より、検査の抜けと公開の安全を重く見た。
- **走らせたもの**: `gen-skill-refs-test`（291 件 ok）・`talk-test`（149）・`verify-scripts-test`（20）・`talk-html-test`（44）・`tutorial-check-test`（132）・`gen-skill-refs --check`・`link-check`・`tutorial-check`。全部通った。
- **走らせていないもの**: jsdom と vscode-textmate が要るテスト（`theme-*`・`highlight-*`・`neutralizer-test`）、`verify-static`・`verify-search`（`book/node_modules` が無いため。spec の欠陥ではない）。`release-scripts-test.ps1` も走らせていない（読んだだけ）。
- **読み方**: `ponytail:ponytail-audit` の観点（不具合・危険・重複・使われないコード・テストの無い危ない処理）で読んだ。`book/tools` と `book/theme` は手分けして読み、主な指摘はこちらで行を確かめた。

## 控えとの突き合わせ（2026-10-19 の 11 件）

| # | 控えの指摘 | 今 | 今の場所 |
|---|------------|----|----------|
| B1 | マニュアルの検査が、Rust の PR とリリースの関門で走らない | **残る**（3 つ → 2 つ） | `manual.yml` L28-47 の path filter。`link-check.mjs` L385-415（`checkInternalsPaths`）。内部設計の章は今、**414 か所・200 種類**のパスを引く（控えは 392）。リリースでは `gen-skill-refs --check` が `release.yml` L120-121（build の job）に入ったので、関門で走らないのは `link-check`・`tutorial-check` の 2 つ |
| B2 | Pages の権限が広すぎる | **残る** | `manual.yml` L56-59（ワークフロー全体に `pages: write`・`id-token: write`）、L191-197（Setup Pages。出力は誰も使わず、`enablement: true` のコメントは誤り）。`tech.md` L210 も同じ権限を書いている |
| B3 | 公開の資格を持つ job で、第三者の action が可変のタグ | **残る**（行は同じ） | `release.yml` L279（`dtolnay/rust-toolchain@stable`）・L282（`Swatinem/rust-cache@v2`）・L288・298・308・318・328（`rust-lang/crates-io-auth-action@v1`）・L430（`Azure/login@v3`）、`release-setup-check.yml` L43（`Azure/login@v3`） |
| B4 | 重複したビルド | **残る** | `manual.yml` L129-144（Rust の用意・cache・`cargo run -p pasta_check -- validate`）。`crates/pasta_check/tests/validate_cli_test.rs` L221 の `test_validate_hello_pasta_clean` が、同じ hello-pasta を終了コード 0 で確かめる |
| B5 | 文法の変更で、ハイライトのテストも再公開も走らない | **残る** | `manual.yml` L28-47 に `editors/vscode/syntaxes/**` が無い。着色は `highlight/highlight-html.mjs` L39 で `editors/vscode/syntaxes/pasta.tmLanguage.json` を読む |
| B6 | `head.hbs` の neutralizer の写しを誰も比べない | **残る** | `head.hbs` L198（BEGIN の印）〜L287（END の印）。印を読むコードは無い。tokenizer の写しは `verify-search.mjs` L350-407 が比べる（作ったページから印の間を取り出し、`vm` で動かして 15 の見本と検索の結果を比べる。写しが逐語でないため動きで比べている） |
| B7 | `highlight-html.mjs` が色を 1 つも付けなくても exit 0 | **残る** | `highlight/highlight-html.mjs` L255-269。`blocksColored` はログに出すだけ。後の検査（`verify-static` など）も `hljs-` の span を見ない |
| B8 | bigram の PoC と fixture が `build-index-test` と重複 | **残る**（補足あり） | `bigram-index/poc-test.mjs`（183 行）・`bigram-index/poc/`（5 ファイル）。`build-index-test.mjs` の RED は L222-233 |
| B9 | `talk.mjs` が `chapterRegions` の写しを持つ | **残る**（行が動いた） | `talk/talk.mjs` L266-275（L266-267 に「循環を避けて複製」のコメント）。元は `gen-skill-refs.mjs`（L121 から写しを指す） |
| B10 | Node 20 は寿命が切れている | **一部直った** | `manual.yml` L75 は PR #187 で 22。残りは `release.yml` L25（`NODE_VERSION: '20'`）・`release-setup-check.yml` L35 |
| B11 | CI の mdBook 0.5.3 と、テストの前提の 0.5.4 がずれる | **残る** | `manual.yml` L92（`0.5.3`）、`tech.md` L223。テストの前提は `talk/talk-html-test.mjs` L11・`theme-menu-test.mjs` L17-18・`theme-sidebar-test.mjs` L14-15・`theme-sidebar-early-test.mjs` L6（0.5.4）。手元の mdBook も 0.5.4。注記の 0.5.3 は `head.hbs` L11・L13・`highlight/highlight-html-test.mjs` L84 にも残る |

### B8 の補足

- `poc-test.mjs` は CI で**走っている**。`manual.yml` L159-167 の自己テストの段が、`find book/tools -name '*-test.mjs'` で 16 本を全部拾うため。中で `poc/` に `mdbook build` をかける。
- PoC の RED は、mdBook の振る舞いを確かめるもので、pasta のコードを確かめていない。GREEN は `build-index-test` と重なる。
- ただし `poc-test.mjs` L144-179 の約 22 の検査（null・数・1 文字の漢字・全角 → 半角・半角カナ・長音・CJK 拡張 A・拡張 B）は、`tokenize.mjs` を直に確かめる**唯一の単体テスト**である。消す前に `build-index-test` へ移す。
- `build-index-test` の RED（L222-233）は、CI では正しい理由で失敗しない。その時点の索引はもう bigram なので、既定の分かち書きで「ファレ」を引くと、別の理由で 0 件になる。

### 2026-11-02 の再測定の項目の確認

- `gen-skill-refs --check` が `release.yml` の build の job（L120-121）にある。**そのとおり**。
- `pasta-check` のスキルが `link-check.mjs` L280 の `CHECKED_SKILLS`（2 つだけ）にも、`manual.yml` の path filter にも無い。**そのとおり**（N17）。
- `verify-content.mjs` L23 の「全 62 章」。**直った**（`e9847974` で「全 64 章」）。
- `manual.yml` L191-197 の Setup Pages の注記の誤り。**そのとおり**（B2）。
- 章の数を決め打ちする自己テスト 3 つ。**残る**（値が 63 → 64 に変わっただけ）。`gen-skill-refs-test.mjs` L600・`talk/talk-test.mjs` L453-454・`verify-scripts-test.mjs` L81（`T-syntax`・`T-intro`・`T-outro` の 64）。下の N16 を参照。

## 新しい指摘（2026-11-02 の監査のやり直し）

重い順に並べる。「直し」は一番小さい案である。決めるのは設計。

### 直すべき（Should fix）

**N1. 本文の検査が、4 つのバッククォートのフェンスを取り違える**（`verify-content.mjs` L53-66）
- **何か**: `extractCodeFences` と `stripCodeFences` が、素朴な正規表現 `/```[^\n]*\n[\s\S]*?```/` でフェンスを探す。
- **問題**: 外側のフェンスが 4 つのバッククォートだと、中の 3 つのバッククォートで閉じたとみなす。`grammar/block-structure.md` では、本物の H2 が 7 つ（選択肢行〜インデントの判定）フェンスの中とみなされて消え、コードの 18 行が文章として残る。`lua/patterns.md` で 5 つ、`lua/modules/pasta-log.md` で 1 つ。口調の検査（D-voice L339・D-codevoice L348）・見出しの検査（C-sections L275・I-sections L541）が、違う範囲を見ている。今通るのは運である。
- **直し**: 同じファイルが L35 で import 済みの `maskFences` を使う。約 −10 行・+8 行。

**N2. 検索のテストが、ブラウザと違う設定で検索する**（`verify-search.mjs` L96-104・`bigram-index/build-index-test.mjs` L151-159・`bigram-index/poc-test.mjs` L56-64）
- **何か**: 3 つのテストが `SEARCH_OPTIONS = { bool: 'AND', … }` を書き写し、「searcher.js と同じ」とコメントする。
- **問題**: 本物の検索は索引の `search_options` を使い、その値は `bool: 'OR'` である（作った searcher と索引で確かめた。`build-index.mjs` L134 が元の値を引き継ぐ）。テストは AND を確かめ、読者は OR で検索する。OR でだけ出る崩れを、テストは見ない。
- **直し**: 索引の `search_options` を渡す。約 −27 行・+3 行。写しが 3 つ消える。

**N3. 検索のテストの補助関数が 3 つずつある**（同じ 3 ファイル。`searchUrls`・`searchResults`・`corpusOf`）
- B8（PoC を消す）と N2 で大半が消える。残るものは `build-index.mjs` から export して 1 つにする。

**N4. 作ったページに neutralizer が入っていることを、誰も確かめない**（`verify-search.mjs` L353-356 は tokenizer の印だけを見る）
- B6 の比べる検査に、作ったページに印があることの確認を足す（+2 行）。
- B6 そのものは、写しが逐語（字下げを除く）なので、行を trim して空行を除いた文字列を比べれば足りる。`function isPastaElement` から END までが、両方で 1,811 文字で一致することを確かめた。約 8 行。作ったページが要らないので、軽い検査の側にも置ける。

**N5. 内部設計の章のパスの検査が、`editors/` を見ない**（`link-check.mjs` L392 の `REPO_PATH_PREFIXES`）
- **問題**: 内部設計の章は `editors/vscode/syntaxes/pasta.tmLanguage.json`（2 か所）と `editors/vscode/src/test/tmGrammar.test.ts`（1 か所。`talk-output.md` L350・351・365）を引く。どちらも今は在るが、名前を変えても検査は落ちない。
- **直し**: 並びに `'editors/'` を足す。1 語。

**N6. 自己テストが、公開する出力を書き換える**（`verify-search.mjs` L90-92・`bigram-index/build-index-test.mjs` L292・L337）
- **問題**: `verify-search` は `--no-build` でも `book/book` の索引を作り直す。`build-index-test` も `book/book` に書く（L292 の `write: true`、L337 の引数なしの CLI）。どちらも `manual.yml` で Pages に上げる前に走る。作り直しが決定論的なので今は害が無いが、テストのせいで公開物が変わりうる。
- **直し**: `--no-build` のときは作り直さない（±2 行）。`build-index-test` は一時のフォルダの写しに書く。

**N7. 配布物を作る job が、可変のタグの第三者の action を使う**（`release.yml` L73 `dtolnay/rust-toolchain@stable`・L78 `Swatinem/rust-cache@v2`）
- **問題**: build の job は資格を持たないが、`pasta.dll`・`pasta_check.exe`・VSIX を作り、それがそのまま公開される。タグを付け替えられると、別のコードで作った配布物が出る。B3 と同じ危険が、公開物の側にある。
- **直し**: B3 と同じく SHA で固定する（前提 D）。関門の `build.yml` は何も配らないので、タグのままでよい。

**N8. ビルドの道具を、中身を確かめずに取って来る**（`release.yml` L89-110）
- **問題**: `wasm-pack` と `cargo-about` を GitHub の release から取り、`--version` の出力だけを確かめる。中身の hash は見ない。`wasm-pack` は VSIX に入る WASM を作り、`cargo-about` は配るライセンスの表を作る。
- **直し**: 2 つの SHA-256 を `env` に置き、`Get-FileHash` で比べる。約 +4 行（前提 D）。

**N9. リリースの補助スクリプトに、誰も使わない差し替えの口がある**（`publish-crate.ps1` L17-18 の `-ApiBase`・`publish-vsix.ps1` L159-160 の `-VsceCommand`）
- **問題**: どちらも「検証用」の口だが、リポジトリのどこからも使われない（`.yml`・`.ps1`・`.md` を grep した）。一方で、「公開済みなら飛ばす」の判定（`publish-crate.ps1` L58-65・L92-96、`publish-vsix.ps1` L204-213・L227）には、テストが 1 つも無い。`release-scripts-test.ps1` が試すのは `release-notes`・`verify-tag`・`highlights-check`・`github-release`（添付の有無だけ）である。
- **直し**: 2 つの案がある（前提 E）。
  - 消す: 2 つの引数を消す。約 −4 行。
  - 使う: `-VsceCommand` に偽の vsce（`pwsh -File` の小さなスクリプト）を渡して、`release-scripts-test.ps1` に「公開済みなら skipped」の項目を足す。ネットワークは要らない。`-ApiBase` は HTTP の偽物が要るので、消すほうが小さい。
- `-DryRun`（3 つのスクリプト）は、各スクリプトの `.EXAMPLE` に手元の確認の使い方が書いてある。指摘にしない。

**N16. 章の数の決め打ちと、手で持つフォルダの並び**（議題。前提 A）
- 自己テスト 3 つの数え方はそれぞれ違う。
  - `gen-skill-refs-test.mjs` L597-600: `book/src` のファイルを数える（SUMMARY.md を除く）。
  - `talk/talk-test.mjs` L453-454: `talk.mjs --stats` の章の行を数える（これもファイルを数える）。
  - `verify-scripts-test.mjs` L81: `verify-content` の出力の行を数える。`verify-content` は、手で持つフォルダの並び `VOICE_DIRS`（L319）と `debug`（L596）から章を選ぶ（下のフォルダは見ない）。
- `SUMMARY.md` の `](x.md)` のリンクは 64 種類で、ファイルの集合と同じだった。同じ正規表現が `verify-search.mjs` L259・`verify-static.mjs` L163・`verify-content.mjs` L163（変形）に 3 回ある。
- **直しの案**: `link-check.mjs` から `SUMMARY.md` の章の並びを返す関数を 1 つ export し、3 つの自己テストで「ファイルの数 ＝ SUMMARY.md の数」と「`verify-content` が見た章の数 ＝ SUMMARY.md の数」を比べる。約 −15 行・+8 行。
- **注意**: 総数の決め打ちだけを消して比べるのをやめると、`VOICE_DIRS` に無いフォルダを足したとき、口調と台詞部品の検査を黙ってすり抜ける。今は 64 ≠ 65 で落ちて気づける。比べることは残す。
- 同じ仲間の決め打ち（`verify-scripts-test.mjs` L62-74 の I-structure 10 など、`talk-test` A-15 の 16、`gen-skill-refs-test` A-5・H-6 の 24）は、章を足しても変わらない数なので、この議題に含めない。

**N17. 配るスキル `pasta-check` が、リンクの検査の外にある**（`link-check.mjs` L280・`manual.yml` L28-47）
- **問題**: `pasta-check` は `pasta-skills.zip` で配る（`release.yml` L167-170）。`SKILL.md` L56 に公開マニュアルの URL（`…/debug/validate.html`）があり、見出しへの相対リンク（L101 `#同梱バルーン`・L205 `#エラー` など）もあるが、どれも検査されない。
- **そのまま足すと**: 自己完結の検査（`crates/` を引かない）で 6 か所落ちる（`SKILL.md` L3・L160・L180、`nar-spec.md` L68、`updates-txt-spec.md` L154-155）。直すにはスキルのファイルを変える（前提 C）。

### あるとよい（Nice to have）

**N10. リンクの検査が、章のコードフェンスの中のリンクも見る**（`link-check.mjs` L156）
- スキルと README の検査はフェンスを隠してから見る（L340・L463）が、章の検査だけ隠さない。今は隠しても隠さなくても 1,457 件で同じ。コードの例に `[x](y.md)` を書くと、壊れたリンクとして報告される。`extractLinks(maskFences(text))` で ±1 行。

**N11. 分かち書きに、決して通らない分岐がある**（`bigram-index/tokenize.mjs` L37・`head.hbs` L78）
- `cp >= 0x20000`（CJK 拡張 B 以降）の判定は、UTF-16 の 1 単位（`text[i]`）を見るループでは起きない。「𠮷野家」は ['𠮷', '野家'] になる（`poc-test.mjs` L176-179 がこれを確かめる）。`review-improvement-loop` 3.63 で負債として記録済み。
- 直し: 両方の写しから 1 行ずつ消す（動きは変わらない）か、両方を code point で回す（約 ±6 行）。写しは同時に直す（B6・N4 の検査が守る）。

**N12. 目次の開閉の条件が 2 か所にある**（`head.hbs` L317-329・`claudia.js` L82-94）
- どちらも幅 620・1080 と `'visible'` の条件を持ち、誰も比べない。

**N13. 使われない export**（`gen-skill-refs.mjs` L180 `GENERATED_MARK`・L181 `FIX_COMMAND`、`verify-content.mjs` L37 `REPO_ROOT`）
- 外から import するものが無い。`verify-content.mjs` は import した時点で検査が走るので、そもそも import できない。`export` を 3 つ外す。

**N14. 公開マニュアルの URL が 2 か所で決まる**（`link-check.mjs` L424 `MANUAL_URL`・`gen-skill-refs.mjs` L35 `MANUAL_BASE_URL`）
- 「同値（import しない）」のコメントがあるが、`gen-skill-refs` は既に `link-check` から import している。片方をもう片方から取る。

**N15. Markdown のファイルの一覧が 2 つある**（`verify-content.mjs` L132-140 `listMarkdown`・`link-check.mjs` L90-99 `listMarkdownFiles`）
- `verify-content` は `link-check` から import できる。約 −9 行・+1 行。

### 指摘にしなかったもの

- `.github/scripts/release/*.ps1` の `Write-To`・`Complete` が 4 つのスクリプトに似た形である。中身がスクリプトごとに違い（summary の行の形、`Write-Host` と `Write-Output`）、1 つにまとめると共通のファイルが増える。直す得が小さい。
- `verify-scripts-test.mjs` が `verify-content.mjs` をもう一度走らせる。検査のスクリプトそのものを試すためで、重複ではない。
- `manual.yml` が軽い検査 3 つを Build ワークフローと二重に走らせることになる点。Manual ワークフローは公開の前の関門なので、残すほうが安全である（各 1 秒）。設計で確かめる。
- 公開の資格を持つ job の、GitHub 自身の action（`actions/checkout`・`actions/setup-node`・`actions/download-artifact`）。brief の範囲は第三者の action である（前提 D）。

## 指摘の台帳

要件の Requirement 10.1 が見る一覧である。完了のとき、各行を「直した」か「外した（理由つき）」にする。

| 指摘 | 重さ | 要件 |
|------|------|------|
| B1 軽い検査が全部の push・PR と関門で走らない | Must | 1.1-1.3・1.5 |
| B2 Pages の権限・Setup Pages | Should（危険） | 3 |
| B3 資格を持つ job の可変のタグ | Should（危険） | 4.1-4.3・4.6 |
| B4 `pasta_check` のビルドの重複 | Should | 5.1 |
| B5 文法の変更で起動しない | Should | 2.1-2.2 |
| B6 neutralizer の写しを比べない | Should | 6.1 |
| B7 着色 0 件で exit 0 | Should | 7.1 |
| B8 PoC と RED の重複・空回り | Should | 5.2-5.4 |
| B9 `chapterRegions` の写し | Should | 5.5 |
| B10 Node 20（残り 2 か所） | Should | 8.1-8.2 |
| B11 mdBook 0.5.3 と 0.5.4 | Should | 8.3-8.4 |
| N1 フェンスの取り違え | Should（不具合） | 7.2 |
| N2 検索の設定の食い違い | Should | 5.6 |
| N3 検索の補助関数の写し | Should | 5.7 |
| N4 作ったページの neutralizer | Should | 6.2 |
| N5 パスの検査に `editors/` が無い | Should | 1.4 |
| N6 自己テストが公開物を書き換える | Should | 7.4 |
| N7 配布物を作る job の可変のタグ | Should（危険） | 4.4 |
| N8 道具の中身を確かめない | Should（危険） | 4.5 |
| N9 使われない差し替えの口・公開の判定のテストが無い | Should | 10.6 |
| N16 章の数の決め打ち | Should（決定 A） | 9 |
| N17 `pasta-check` がリンクの検査の外 | Should | 2.3・2.5・2.6 |
| N10 フェンスの中のリンク | Nice | 7.3 |
| N11 通らない分岐 | Nice | 7.5 |
| N12 目次の条件の写し | Nice | 6.3 |
| N13 使われない export | Nice | 10.8 |
| N14 URL の 2 か所 | Nice | 10.8 |
| N15 ファイルの一覧の写し | Nice | 10.8 |

規模の見込み: 消す行が約 250（PoC の 183 行と 5 ファイルを含む）、足す行が約 80。

## 要件と今ある物の対応

| 要件 | 今ある物 | ギャップ |
|------|----------|----------|
| 1 軽い検査 | 3 つのスクリプトは依存なしで動く。`build.yml` は全部の push・PR と `workflow_call`（関門）で走る | **Missing**: `build.yml` に node の段が無い。**Constraint**: `build-unix` は ubuntu と macOS の matrix で、cargo の長いビルドを持つ |
| 2 起動の条件 | `manual.yml` L28-47 の path filter（push と PR で同じ並びを 2 回書く） | **Missing**: `editors/vscode/syntaxes/**`・`.claude/skills/pasta-check/**` |
| 3 Pages の権限 | deploy の job（L205-216）は `actions/deploy-pages` だけ | **Missing**: job ごとの `permissions`。**Constraint**: `upload-pages-artifact` は build の job にあり、権限は要らない |
| 4 固定 | なし（全部タグ） | **Missing**: SHA。**Research Needed**: `dtolnay/rust-toolchain` の `@stable` はブランチで、toolchain の既定をブランチ名から決める。SHA で指すと `with: toolchain: stable` が要る |
| 5 重複 | `validate_cli_test.rs` L221 が hello-pasta を守る | PoC の tokenizer の単体の検査を移す先が要る |
| 6 写し | `verify-search.mjs` の tokenizer の比べ方（印の間を取り出す） | **Missing**: neutralizer の比べる検査（約 8 行） |
| 7 空回り | `maskFences`（`link-check.mjs`）が既にある | `verify-content`・`link-check` の章の検査が使っていない |
| 8 版 | `manual.yml` は Node 22 | **Research Needed**: 旧版（`CLASSIC_REF` の `book/`）を 0.5.4 で作れるか |
| 9 章の数 | `SUMMARY.md` の章の並びを読む正規表現が 3 か所にある | 1 つの関数にまとめる先（`link-check.mjs` が共有の部品） |
| 10 守り | `release-scripts-test.ps1` の項目 10・11、`release.yml` L135-235 の検査と upload | 保つ行を壊さないこと |

## 実装の方針の選択肢

ここは、一番大きい選択（軽い検査をどこで走らせるか）について並べる。ほかの指摘は、どれも今あるファイルを小さく直すだけである（Option A の形）。

| 案 | 中身 | 良い点 | 悪い点 |
|----|------|--------|--------|
| A: `build-unix` の ubuntu の行に載せる | `build-unix` の先頭に、`if: matrix.os == 'ubuntu-latest'` を付けた node の段を 3 つ足す | job が増えない | matrix の全部の段に条件が要る。Rust の失敗と混ざる。Node の版が runner の既定任せ（または setup-node を足す） |
| B: cargo の要らない小さな job を足す | `build.yml` に、checkout・setup-node（22）・3 つの node の段だけの job を足す（約 12 行） | 1 分以内に落ちて、原因がすぐ分かる。関門でも自動で走る。Rust と無関係 | job が 1 つ増える |
| C: B ＋ `manual.yml` から外す | B に加えて、`manual.yml` の 3 つの段を消す | 二重に走らない | Manual ワークフローの公開が、検査を待たなくなる（deploy は `needs: build` だけ）。公開の関門が弱くなる |

- 推奨は **B**。`manual.yml` の 3 つの段は、公開の前の関門として残す。
- B6 の neutralizer の比べる検査は、作ったページが要らないので、B の job に載せられる（設計で決める）。
- スキルの生成物の鮮度の照合は、B の job が関門で走るので、`release.yml` の build の job の L119-121 は重複になる（前提 B）。

## 規模と危険

- **規模: M（3〜7 日）**。直しはどれも小さいが、4 つのワークフローと約 12 のファイルにまたがる。タスクは 13〜16（章の数の議題を含む）。
- **危険: Medium**。
  - `release.yml` の変更は、リリースのタグを打つまで本番の道で試せない。`build.yml` と `manual.yml` は `gh workflow run --ref` で試せる。
  - `dtolnay/rust-toolchain` を SHA で指すとき、toolchain の入力を足し忘れると build が落ちる。
  - mdBook 0.5.4 で旧版が作れないと、公開が止まる。

## Research Needed（設計で調べる）

1. **固定する SHA**: `dtolnay/rust-toolchain`（`stable` のブランチ。SHA で指すなら `toolchain: stable` を渡す）・`Swatinem/rust-cache` v2・`rust-lang/crates-io-auth-action` v1・`Azure/login` v3 の、今のタグが指す commit。
2. **道具の hash**: `wasm-pack` 0.15.0 と `cargo-about` 0.9.2 の Windows の tar.gz の SHA-256（release のページに載っているか、一度取って測る）。
3. **action の Node 20 の実行環境**: `actions/setup-node@v4`・`actions/upload-artifact@v4`・`actions/download-artifact@v4`・`actions/configure-pages@v5`・`actions/upload-pages-artifact@v3`・`actions/deploy-pages@v4`・`peaceiris/actions-mdbook@v2` は、action そのものが Node 20 で動く版だったはずである。GitHub の runner が Node 20 の action をいつまで動かすかを確かめる。B10（`node-version` の 20）とは別の話で、今は指摘にしていない。直すなら major を上げる。
4. **旧版と mdBook 0.5.4**: `CLASSIC_REF` の `book/` と、その版の `highlight-html.mjs`・`build-index.mjs` が、0.5.4 の出力で動くか。
5. **Setup Pages を消した後の公開**: `upload-pages-artifact` と `deploy-pages` が `configure-pages` の出力なしで公開できるか（使われていないので動くはず。一度 main で確かめる）。

## 設計への申し送り

- 保つ行（申し送りから）: `release.yml` build の job の「Verify release assets」の 5 つの配布物の検査（`pasta-skills.zip` の中身と `git ls-tree` の失敗で止まる L171 を含む）、upload の path（L226-231）、`github-release.ps1` L161 の `$names` の 6 つ、`release-scripts-test.ps1` の項目 10・11。
- `release.yml` の build の job の `gen-skill-refs --check`（L119-121）は、申し送りでは「保つ行」に入っているが、関門に照合を足せば重複になる。どちらに残すかを議論で決める（前提 B）。
- `tech.md` の直す行: L209（Manual ワークフローの起動の条件）・L210（権限）・L223（mdBook の版）。`ponytail-editor-tooling` が触る行とは別である。
- `release.yml` の変更は、リリースの実行中に main へ入れない。同じウェーブに `pasta-dll-multiarch`・`vsce-4-upgrade`・`pest-2-9-4-upgrade` を置かない（brief のとおり）。
- PoC を消すときは、tokenizer の単体の検査（`poc-test.mjs` L144-179）を先に `build-index-test` へ移す。
- 軽い検査の置き場所（上の案 A〜C）は、要件の議論で設計の判断とした。要件は Requirement 1.5（Rust のビルドを待たない）だけを決める。
- 固定する SHA・道具の hash・旧版と mdBook 0.5.4・Setup Pages を消した後の公開は、上の Research Needed のとおり設計で調べる。

## OPEN QUESTIONS の結論（2026-11-02 の要件の議論。全部を推奨案で確定）

1. 章の数の議題は採る（requirements.md の決定 A・Requirement 9）。
2. 軽い検査の置き場所は設計で決める（推奨は案 B のまま）。要件は Requirement 1.5 だけ。
3. スキルの生成物の鮮度の照合は関門にだけ残し、`release.yml` L119-121 を消す（決定 B・Requirement 1.6）。
4. 固定は、公開の資格を持つ job・`release.yml` の build の job の第三者の action・道具 2 つの SHA-256。`build.yml` と `actions/*` はタグのまま（決定 D・Requirement 4）。
5. Node 22 はこの spec で行う（Requirement 8.1・8.2）。
6. `pasta-check` のスキルは、自己完結の検査まで足し、`crates/` を引く 6 行を直す（決定 C・Requirement 2.3・2.5・2.6）。`.claude/skills` の編集は、実装のときに開発者の許可を求める。
7. `-ApiBase` は消し、`-VsceCommand` は「公開済みなら skipped」の項目に使う（決定 E・Requirement 10.6・10.9）。
8. Node 20 の実行環境の action は、設計で調べ、要るならこの spec で major を上げる（決定 4・Requirement 8.6）。

## 設計フェーズの調査（2026-11-02・`/kiro-spec-design -y`）

- **Discovery の種類**: Light（Extension）。新しい依存は無い。外す依存が 1 つ（`peaceiris/actions-mdbook`）。
- **使ったスキル・規則**: `kiro-spec-design`（`design-principles`・`design-discovery-light`・`design-synthesis`・`design-review-gate`）、`ponytail`（一番小さい完全な直し・消す行を優先）。
- **調べ方**: `gh api`（読むだけ）、`actionlint` 1.7.12、開発機の mdBook 0.5.4・Node 26・pwsh 7.6。リポジトリのファイルは変えず、スクラッチのフォルダに写して走らせた。

### Research Needed の答え

**1. 固定する SHA**（`gh api repos/<owner>/<repo>/git/ref/tags/<tag>` と `git/tags/<sha>` で注釈つきのタグをたどった。ブランチは `branches/<name>`）

| action | 参照 | commit | 補足 |
|--------|------|--------|------|
| `dtolnay/rust-toolchain` | `stable`（ブランチ） | `b725eda525362e7cf34af9a7243cac1e3f01a7ec` | 2026-10-16 の commit。この SHA の `action.yml` で `toolchain` の既定が `stable`。SHA で指しても入力は要らない |
| `Swatinem/rust-cache` | `v2`（注釈つきのタグ） | `d1527c8f67ef9af357290475d4af4b0fab8f12e0` | v2.9.3 と同じ |
| `rust-lang/crates-io-auth-action` | `v1`（ブランチ。`v1` のタグは無い） | `3268eb413133e2960fa4ae0e963ca76380c54cd9` | `compare/v1.0.6...v1` が identical |
| `Azure/login` | `v3`（注釈つきのタグ） | `a641126d1b8aa4d1fa005f4f92df94a3a4c4c906` | v3.1.0 と同じ |

- ギャップ分析の「`@stable` を SHA で指すなら `toolchain: stable` が要る」は、`master` の SHA を使うときの話だった。`stable` のブランチの SHA なら既定が `stable` なので要らない。

**2. 道具の hash**（どちらも 2 つの出所で一致した）

| 道具 | SHA-256 | 出所 1 | 出所 2 |
|------|---------|--------|--------|
| `wasm-pack-v0.15.0-x86_64-pc-windows-msvc.tar.gz`（2,290,392 バイト） | `518dc51180c7bc864699c9279b3bc99025bc109123e0249a34ba34d130a509bf` | release の asset の `digest` | スクラッチに取って `sha256sum` で測った（実行していない） |
| `cargo-about-0.9.2-x86_64-pc-windows-msvc.tar.gz`（6,186,529 バイト） | `1c03e5890238562497c2d89a3b75b02560af349c1fc3e713d3284f532a5cd748` | release の asset の `digest` | 公開の `.sha256` のファイル |

- `wasm-pack` は `.sha256` のファイルを出していない。

**3. Node 20 で動く action**（`gh api repos/<repo>/contents/action.yml?ref=<tag>` の `runs.using`）

| action | 今 | runs.using | 最新の major（release） | その runs.using |
|--------|----|-----------|------------------------|-----------------|
| `actions/checkout` | v6 | node24 | — | — |
| `actions/setup-node` | v4 | node20 | v7（v7.1.0・2026-10-08） | node24 |
| `actions/upload-artifact` | v4 | node20 | v7（v7.0.2） | node24 |
| `actions/download-artifact` | v4 | node20 | v8（v8.0.2） | node24 |
| `actions/configure-pages` | v5 | node20 | v6 | node24（使わないので消す） |
| `actions/upload-pages-artifact` | v3 | composite（中で `upload-artifact@v4`） | v5（v5.0.0） | composite（中で `upload-artifact@v7`） |
| `actions/deploy-pages` | v4 | node20 | v5（v5.0.1） | node24 |
| `peaceiris/actions-mdbook` | v2 | node20 | v2.0.0（2024-04-08。新しい release は無い） | node20。`main` は node24 だが release が無い |
| `dtolnay/rust-toolchain` | stable | composite | — | — |
| `Swatinem/rust-cache` | v2 | node24 | — | — |
| `rust-lang/crates-io-auth-action` | v1 | node24 | — | — |
| `Azure/login` | v3 | node24 | — | — |
| `EmbarkStudios/cargo-deny-action` | v2 | docker | — | — |

- GitHub の changelog（[Deprecation of Node 20 on GitHub Actions runners](https://github.blog/changelog/2025-09-19-deprecation-of-node-20-on-github-actions-runners/) と、その後の編集）では、2026-06-16 からランナーが Node 24 を既定にし、2026-09-23 に Node 20 を外す予定だった。今日（2026-11-02）は両方の日を過ぎている。外したことが本当に起きたかは確かめていない（前提）。どちらにしても、node20 と書いた action は寿命の切れた実行環境を名指ししているので、上げる。
- 上げる先の破壊的な変更（各 release のノートを読んだ）:
  - `setup-node` v5: `packageManager` があると自動で cache。`book/package.json`・`editors/vscode/package.json` に無いので影響なし。v6 は自動 cache を npm に限っただけ。v7 は ESM 化と出力の追加。
  - `upload-artifact` v5・v6: Node 24。v7: `archive` の入力の追加。
  - `download-artifact` v5: ID で取るときの置き場所の変更（名前で取るので影響なし）。v7: Node 24。v8: hash の不一致を既定でエラー。
  - `upload-pages-artifact` v4: `.` で始まるファイルを既定で外す。v5: `include-hidden-files` の入力を足した。mdBook の出力には `.nojekyll` がある（開発機で確かめた）。
  - `deploy-pages` v5: Node 24。
- `peaceiris/actions-mdbook` は、release の無い `main` の SHA に固定するより、公式の配布物（`rust-lang/mdBook` の v0.5.4 の `mdbook-v0.5.4-x86_64-unknown-linux-gnu.tar.gz`。2026-07-06 公開）を取るほうが、第三者の action が 1 つ減って小さい。

**4. 旧版と mdBook 0.5.4**

- `git archive <CLASSIC_REF> book` をスクラッチに取り出し、`MDBOOK_OUTPUT__HTML__PRINT__ENABLE=false mdbook build`（0.5.4）が通った。旧版の `build-index.mjs` も、その出力で索引を作り直せた（4.54 MB）。
- 旧版の着色（`highlight-html.mjs`）は `vscode-textmate` が要るので、開発機では走らせていない。実装のときに、ブランチの上の `gh workflow run manual.yml --ref <branch>` で確かめる（OPEN QUESTIONS 5）。

**5. Setup Pages を消した後の公開**

- `configure-pages` の出力（`base_url` など）は、どの段も使っていない。`deploy-pages` は、`upload-pages-artifact` が上げた artifact `github-pages` だけを見る。消しても公開は変わらない見込み（前提）。マージの後の main への push で確かめる。

### 設計で新しく分かったこと

- **検索の包含の検査は、ブラウザの設定（OR）では成り立たない**。索引の `search_options` は `{"bool":"OR","expand":true,…}` だった（スクラッチに作った出力で確かめた）。`verify-search.mjs` と `build-index-test.mjs` の写しを OR で走らせると、「語中片のヒットが親語のヒットを包含する」が 8 件と 2 件落ちた。OR では、親語の片（例「レン」）だけを含む節も親語のヒットに入るためである。索引の不具合ではない。検査を「親語を題・本文・パンくずに含む節は、すべて語中片のヒットに入る」に直すと、両方とも全部通った（136 件・47 件）。ギャップ分析の N2 の「約 −27 行・+3 行」は、この検査の書き直しの分だけ増える。
- **`github-pages` の environment は main だけに公開を許す**（`deployment-branch-policies` が `["main"]`）。今の `manual.yml` の条件（`github.event_name != 'pull_request'`）のままブランチで `workflow_dispatch` すると、deploy の job が止められて赤になる。条件を `github.ref == 'refs/heads/main'` にすれば、ブランチでは検査だけが走る（要件 10.7 の確かめ方が緑で終わる）。
- **`pasta-check` を足して落ちるのは、ちょうど 6 行**。`link-check.mjs` の写しの `CHECKED_SKILLS` に `pasta-check` を、`REPO_PATH_PREFIXES` に `editors/` を足して走らせた。スキルの検査で 6 件（全部 `crates/`）、URL の検査で 0 件、内部設計の章のパスで 0 件。見出しへの相対リンク（`#同梱バルーン` など）は通った。
- **禁止の語の検査は URL の中も見る**。公開の GitHub の URL（`…/blob/main/crates/…`）に変えても `crates/` で落ちる。6 行は「`pasta_check` クレートの `src/nar.rs`」のように、`crates/` の語を使わない書き方にする。
- **N1 の直しで落ちる章は無い**。`verify-content.mjs` の写しで、`stripCodeFences` を `maskFences` に、`extractCodeFences` を `maskFences` の結果から作る形にして走らせると、今の章で全部 PASS（401 件）だった。
- **neutralizer の写しは、行ごとの trim と空行の除去で一致する**（77 行）。比べる範囲は「最初の `if (!win) {` から END まで」。
- **偽の vsce は、`-VsceCommand` に `.ps1` のパスを渡すだけで動く**。`& $vsceExe` が `.ps1` をそのまま実行する。開発機で `status=skipped`・終了コード 0 を確かめた。
- `release-scripts-test.ps1` は今、開発機で項目 1〜11 が全部 ok。`actionlint` は今の 4 つのワークフローで 0 件。
- `book.toml` は `create-missing` を決めていない（既定は作る）。目次にだけある章は `mdbook build` が空のファイルとして作るので、ビルドの後の自己テストでは見つからない。ビルドの前の `link-check` が見つける。

## Architecture Pattern Evaluation（軽い検査の置き場所）

| 案 | 中身 | 良い点 | 悪い点 | 判断 |
|----|------|--------|--------|------|
| A | `build-unix` の ubuntu の行に載せる | job が増えない | matrix の全部の段に条件。Rust の失敗と混ざる。Rust の後に走る（1.5 に反する） | 採らない |
| B | `build.yml` に cargo の要らない job を足す。`manual.yml` の段は残す | 1 分以内に落ちる。関門でも走る。公開の関門も保つ | job が 1 つ増える（約 15 行） | **採る** |
| C | B ＋ `manual.yml` の 3 段を消す | 二重に走らない | 公開の deploy が検査を待たなくなる | 採らない |

- 写しの比べる検査（neutralizer・目次の条件）は、軽い job に置かない。写しも正準も `book/**` にあるので、変えれば必ず `manual.yml` が起動する。`neutralizer-test.mjs`（jsdom）と `theme-sidebar-early-test.mjs` に置くと、新しいファイルも段も要らない。

## Design Decisions

### Decision: 軽い検査は `build.yml` の新しい job（案 B）

- **Context**: 要件 1.1〜1.3・1.5。
- **Selected**: job `manual-light`（checkout・setup-node 22・3 つの段）。`needs` なし。
- **Rationale**: Rust を待たず、関門（`release.yml` の gate）でも自動で走る。決定 B の「照合は関門だけ」も、この job で満たす。
- **Trade-offs**: `manual.yml` と二重に走る（各 1 秒）。公開の関門を弱めないほうを採る。

### Decision: 検索の設定は索引から取り、包含の検査を OR に合わせて書き直す

- **Context**: 要件 5.6・5.7（N2・N3）。
- **Alternatives**: 1) 設定の写しを OR に直す（写しが残る） 2) 索引の `search_options` を使い、包含の検査を直す。
- **Selected**: 2。補助関数（`searchIndex`・`corpusOf`・`urlsContaining`）を `build-index.mjs` に 1 つずつ置く。
- **Rationale**: ブラウザと同じ値を、書き写さずに使う。包含の検査は、設定によらず成り立つ形にする。
- **Follow-up**: mdBook が `search_options` を変えたら見直す（design の Revalidation Triggers）。

### Decision: mdBook は公式の配布物を取る

- **Context**: 要件 8.3・8.6。`peaceiris/actions-mdbook` に Node 24 の release が無い。
- **Alternatives**: 1) `peaceiris` の `main` の SHA に固定 2) 公式の tar.gz を `curl | tar` で取る。
- **Selected**: 2（3 行）。
- **Trade-offs**: SHA-256 は確かめない（決定 D の範囲の外。今の `peaceiris` も確かめていない）。

### Decision: Pages の公開の条件を `github.ref == 'refs/heads/main'` にする

- **Context**: 要件 3.4・10.7。environment は main だけを許す。
- **Selected**: upload と deploy の `if` を置き換える（2 行）。PR は `refs/pull/…` なので今と同じく公開しない。

### Decision: 公開の出力を書き換えない

- **Context**: 要件 7.4（N6）。
- **Selected**: `verify-search` は `--no-build` で作り直さない。`build-index-test` は 2 つのファイルを一時のフォルダへ写して書く。`build-index.mjs` の CLI は引数を必須にする（既定のパスを消すと、テストが `book/book` に書く道が無くなる）。
- **Trade-offs**: 3.63 で足した「引数なしで既定のパスに exit 0」の回帰テストは、「引数なしで exit 1」に置き換わる。既定のパスのコードが無くなるので、守る物も無くなる。

### Decision: 通らない分岐は消す（code point で回さない）

- **Context**: 要件 7.5（N11）。
- **Selected**: 両方の写しから 1 行ずつ消す。動きは変わらない。
- **Rationale**: code point で回すと、拡張 B の分かち書きが変わり、索引と検索を同時に変える必要がある。直す得が小さい。

### Decision: `pasta-check` の 6 行は「クレートの中のパス」の書き方にする

- **Context**: 要件 2.6。URL も `crates/` を含むので使えない。
- **Selected**: 「`pasta_check` クレートの `src/nar.rs`」「pasta リポジトリのサンプルゴーストの `RELEASE.md`」。場所の情報を消さない。

### Synthesis（一般化・採用・簡略化）

- **一般化**: 「`SUMMARY.md` の章の並び」を `summaryChapters` 1 つにする。3 つの章の数のテストと、3 つの読み方（`verify-content`・`verify-search`・`verify-static`）が同じ関数を使う。インターフェースだけを広げ、実装は今の正規表現 1 つ。
- **採用**: 偽の vsce は、今ある `-VsceCommand` の口を使う（新しい口を作らない）。区切りの規則は `chapterRegions` を使う（写しを消す）。フェンスの判定は `maskFences` を使う。
- **簡略化**: 消すもの — PoC（6 ファイル・約 200 行）、`pasta_check` の再ビルドの 3 段、Setup Pages、RED の段、検索の設定と補助関数の写し 3 組、区切りの写し、`listMarkdown`、`-ApiBase`、照合の段、使われない export 3 つ、`peaceiris` の action。足すもの — 軽い job（約 15 行）、比べる検査 2 つ（約 14 行）、偽の vsce の項目（約 10 行）、`summaryChapters`（約 8 行）、hash の照合（約 4 行）。

## Risks & Mitigations（設計）

- 旧版の着色が 0.5.4 で崩れる — ブランチの上の `manual.yml` で公開の前に分かる。崩れたら、旧版の段だけを別の版で作るかを議論する。
- SHA の打ち間違い — `gh api repos/<repo>/commits/<sha>` で 1 つずつ在ることを確かめる。
- `upload-artifact@v7` と `download-artifact@v8` の組み合わせ — 次のリリースで初めて本番の道を通る。落ちたら「失敗した job の再実行」の前に、片方を戻す。
- `release.yml` はリリースの実行中に入れない（brief のとおり）。

## OPEN QUESTIONS（設計。推奨案つき）

1. **`upload-pages-artifact@v5` の `include-hidden-files`** — 推奨: `true` を付ける。理由: 要件 3.5 の「今と同じ中身」を 1 行で保てる（`.nojekyll` が残る）。付けない案は、読者に違いは出ないが中身が変わる。
2. **mdBook の取得を SHA-256 で確かめるか** — 推奨: 確かめない。理由: 決定 D の範囲は release の build の道具 2 つで、マニュアルは配布物でない。今の `peaceiris` も確かめていない。
3. **`dtolnay/rust-toolchain` を SHA で指すとき `toolchain: stable` を明示するか** — 推奨: 明示しない。理由: 固定した commit の `action.yml` の既定が `stable` で、その commit は変わらない。
4. **偽の vsce の項目は「公開済みなら skipped」の 1 つだけか** — 推奨: 1 つだけ。理由: 要件 10.9 が求めるのはこの 1 つ。未公開の道は本物の vsce とネットワークが要る。
5. **旧版の着色を 0.5.4 で確かめる時期** — 推奨: 実装の最後に、ブランチを push して `gh workflow run manual.yml --ref <branch>`。理由: 開発機では `vscode-textmate` を入れない（npm install しない）。mdBook の生成と索引は確かめ済み。
6. **`upload-artifact@v7` と `download-artifact@v8` の組み合わせ** — 推奨: そのまま上げ、次のリリースで確かめる。理由: v4 以降の artifact は major をまたいで読める前提（試していない）。ブランチで release の道は試せない。
7. **`build-index.mjs` の CLI の引数を必須にするか** — 推奨: 必須にする。理由: 呼ぶ側は全部引数を渡している。既定のパスを消すのが、テストが公開物に書かない一番小さい直しである。

## References（設計）

- [Deprecation of Node 20 on GitHub Actions runners](https://github.blog/changelog/2025-09-19-deprecation-of-node-20-on-github-actions-runners/) — Node 24 の既定化と Node 20 を外す日。
- `actions/setup-node`・`upload-artifact`・`download-artifact`・`upload-pages-artifact`・`deploy-pages` の GitHub の release のノート（各 major の x.0.0）— 破壊的な変更。
- `rust-lang/mdBook` の release v0.5.4 — 公式の配布物。
- `rustwasm/wasm-pack` v0.15.0・`EmbarkStudios/cargo-about` 0.9.2 の release の asset の `digest` — 道具の hash。
