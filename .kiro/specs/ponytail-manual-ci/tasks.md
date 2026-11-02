# Implementation Plan

- [x] 1. 配るスキル pasta-check の 6 行の書き換えの許可を、着手の最初に開発者に求める
  - 設計の diff の 6 行（`SKILL.md` の 3 行・`nar-spec.md` の 1 行・`updates-txt-spec.md` の 2 行）を、直した後の行と並べて開発者に示す
  - `.claude/skills` の編集は分類器が止めるので、許可の言葉を得てから 6.1 で書き換える。止められても回避策は探さない
  - 完了の姿: 開発者の許可（または変える指示）が会話に残っている（2026-11-03 に「そのとおり書き換えて」で許可）
  - _Requirements: 2.6_

- [x] 2. 共有の部品（LinkCheckModule）
- [x] 2.1 目次の章の並びを共有にし、リンクの検査の穴を 2 つ塞ぐ
  - 目次（`SUMMARY.md`）から章のパスを目次の順・重複なしで返す共有の関数を足す。先頭の `./` とアンカーを外す読み方は、本文の検査の今の正規表現にそろえる
  - 内部設計の章が引くリポジトリのパスの前置きに `editors/` を足す
  - 章の壊れたリンクの検出を、コードフェンスを隠した本文に対して行う
  - 公開マニュアルの URL の注記から「生成器と同値（import しない）」を消す
  - リンクの検査の自己テストに、フェンスの中の壊れたリンクを報告しない項目と、章の並びの関数が `./`・アンカーを外し重複を消す項目を足す
  - 完了の姿: リンクの検査が今の木で 0 件のまま通り、その自己テストが新しい 2 項目を含めて全部 PASS する
  - _Requirements: 1.4, 7.3, 9.1, 10.8_

- [x] 3. 本文の検査・生成器・章の数
- [x] 3.1 (P) 本文の検査のフェンスの判定を共有の部品に寄せる
  - フェンスを外す処理を共有の `maskFences` に替え、フェンスの中身を取る処理をその結果の上に作り直す（返す形は今と同じ）
  - 4 つのバッククォートの外側のフェンスの中に 3 つのバッククォートを書いた見本で、後ろの見出しがフェンスの外に残る自己確認（N-fence）を足す
  - Markdown のファイルの一覧の写しを消して共有の関数を使い、目次の読みを 2.1 の関数にする
  - 外から使われない `REPO_ROOT` の export を外し、注記の「全 64 章」を「全章」にする
  - 完了の姿: 本文の検査が N-fence を含めて全部 PASS し、C-sections 13・I-sections 8 の件数が変わらない
  - _Boundary: ContentVerify_
  - _Depends: 2.1_
  - _Requirements: 7.2, 9.4, 10.8_

- [x] 3.2 (P) 生成器の URL と区切りの規則を 1 か所にする
  - 生成器の公開マニュアルの URL を共有の部品の値から取り、外から使われない export 2 つ（`GENERATED_MARK`・`FIX_COMMAND`）を外す
  - 台詞部品の集計（`--stats`）の区切りの写しを消し、CLI の枝の中だけで生成器の区切りの関数を `import().then()` で取る（トップレベルの await は使わない）
  - 構造の無い章は導入・締めを数えない扱いを今と同じにし、生成器の「写しも合わせる」の注記を消す
  - 完了の姿: `gen-skill-refs --check` が通り、`talk.mjs --stats` が終了コード 0 で統計を出し、生成器と台詞部品の自己テスト（J-7〜J-9 を含む）が通る
  - _Boundary: SkillRefsAndTalk_
  - _Requirements: 5.5, 10.8_

- [x] 3.3 章の数の決め打ちを目次との突き合わせにする
  - 生成器の自己テスト K-10 を、`book/src` のファイルの集合と目次の章の集合の突き合わせにし、差を名前で示して失敗させる
  - 台詞部品の自己テスト J-9 の 64 を、目次の章の数にする
  - スクリプトの自己テストの T 系の 64 を目次の章の数にし、T-syntax の章の名前の集合と目次の差を名前で示す項目を足す
  - 章を足しても変わらない数（I-structure の 10 など）は変えない
  - 完了の姿: 3 つの自己テストに数字の 64 が無く、今の章で全部 PASS する。目次に無いファイルを 1 つ置くと K-10 が名前を示して落ちる
  - J-9 は 3.2 で直した `talk.mjs --stats` の出力を数えるので、3.2 の後に行う
  - _Boundary: ChapterCountTests_
  - _Depends: 2.1, 3.2_
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 4. 検索（SearchTools）
- [x] 4.1 索引のモジュールに検索の補助を 1 つだけ置く
  - 索引の `search_options` で検索する関数・全文を集める関数・語を含む節の URL を返す関数の 3 つを export する（検索の設定は書き写さない）
  - 完了の姿: 3 つの関数が export され、今の索引の自己テストがそのまま通る
  - _Requirements: 5.6, 5.7_

- [x] 4.2 索引の自己テストを一時のフォルダに向け、PoC の単体の検査を受け取る
  - 頭で公開の出力の elasticlunr と索引を一時のフォルダへ写し、書く検査はそこに対して行い、終わりに消す。前後で公開の出力の索引のバイトが変わらない項目を足す
  - 正しい理由で失敗しない RED の段を消す
  - 書き写した検索の設定と補助を消し、4.1 の 3 つを使う。包含の検査を「親語を含む節は、全部が語中片のヒットに入る」に言い直す
  - PoC のテストが持つ tokenizer の単体の検査（約 25 件）を頭の段に移し、拡張 B の特性の注記を直す
  - 索引の CLI を引数必須にし（無ければ使い方を出して終了コード 1、既定のパスを消す）、同じコミットで自己テストの CLI の項目を「引数なしで exit 1 と使い方」に変える
  - 完了の姿: `mdbook build` の出力に向けて全部 PASS し、走らせた後で公開の出力の索引が変わっていない
  - _Requirements: 5.3, 5.4, 5.6, 7.4_

- [x] 4.3 検索の検査をブラウザと同じ設定にし、公開の出力を書き換えないようにする
  - 書き写した検索の設定と補助を消して 4.1 の 3 つを使い、自己テストの見本の索引に `search_options` を入れる。包含の検査を 4.2 と同じに言い直す
  - `--no-build` で出力が在るときは索引を作り直さない。手元では `--no-build` を付けない旨の注記を 1 行足す
  - 作ったページに neutralizer の BEGIN・END の印があることを確かめる項目を足し、tokenizer の写しを比べる今の検査は残す
  - 検索の検査と静的な検査の目次の読みを 2.1 の関数にする
  - 完了の姿: bigram の索引を作った出力に対して `verify-search --no-build --self-test` と `verify-static` が全部 PASS し、印の項目が含まれる
  - _Depends: 2.1_
  - _Requirements: 5.6, 5.7, 6.2, 6.4, 7.4, 9.1_

- [x] 4.4 通らない分岐と PoC を消す
  - 正準の分かち書きと `head.hbs` の写しから、UTF-16 の 1 単位では起きない範囲の判定の 1 行を同じコミットで消し、注記を「CJK 拡張 A」にする
  - `head.hbs` の注記の mdBook の版を 0.5.4 にする
  - 4.2 の移しの後で、PoC のテストと fixture のフォルダを消す
  - 完了の姿: PoC のファイルが無く、`verify-search` の tokenizer の写しの比べる検査と 4.2 の単体の検査が通る（4.3 と同じく、bigram の索引を作った一時のフォルダの出力に対して走らせる）
  - _Depends: 4.2_
  - _Requirements: 5.2, 7.5, 8.3_

- [x] 5. 写しの検査と着色
- [x] 5.1 (P) 着色が 0 件なら止める
  - 着色の CLI で、`language-pasta` のブロックを 1 つも着色しなかったら理由を stderr に出し、`process.exitCode = 1` にする
  - 着色の自己テストに、`language-pasta` の無い HTML だけのフォルダで CLI が 0 以外で終わる項目を足し、注記の版を 0.5.4 にする
  - 完了の姿: 着色の自己テストが新しい項目を含めて PASS する（jsdom・vscode-textmate が要るので、依存の無いワークツリーでは手元で確かめられず、9.2 の CI で確かめる）
  - _Boundary: HighlightCli_
  - _Requirements: 7.1, 8.3_

- [x] 5.2 (P) neutralizer の写しと正準を比べる
  - `head.hbs` の BEGIN と END の印の間と、正準の関数の本体を、前後の空白と空行を除いた行の並びで比べる項目を足す
  - 違えば最初に違う行の番号と両方の行を示して失敗し、印が無いときも失敗する。わざと 1 行変えた文字列で比べる関数が失敗を返すことも確かめる
  - 完了の姿: 今の 2 つが一致して PASS し、1 行変えた見本で失敗を返す（jsdom が要るので、依存の無いワークツリーでは手元で確かめられず、9.2 の CI で確かめる）
  - _Boundary: MirrorChecks/Neutralizer_
  - _Requirements: 6.1_

- [x] 5.3 (P) 目次の開閉の条件を 2 か所で比べる
  - `head.hbs` と `claudia.js` から画面の幅のしきい値の組と `stored !== 'visible'` を取り出し、各 1 回ずつあって数の組が等しいことを確かめる項目を足す
  - 違えば両方の値を示して失敗する
  - 完了の姿: 目次の自己テストが新しい項目を含めて PASS する（jsdom が要るので、依存の無いワークツリーでは 9.2 の CI で確かめる）
  - _Boundary: MirrorChecks/Sidebar_
  - _Requirements: 6.3_

- [ ] 6. 配るスキル pasta-check を検査に入れる
- [x] 6.1 `crates/` を引く 6 行を書き直す
  - 1 で得た許可のとおりに、設計の diff の 6 行を書き換える
  - パスでなく名前で書き直す（例「`pasta_check` クレートの `src/nar.rs`」）
  - 完了の姿: 配るスキル `pasta-check` に `crates/` の語が無い
  - _Depends: 1_
  - _Requirements: 2.6_

- [ ] 6.2 リンクの検査に pasta-check を足す
  - 検査するスキルの一覧に `pasta-check` を足し、リンクの検査の自己テストの期待と注記を 3 スキルにする
  - 完了の姿: リンクの検査が 3 スキルで 0 件で通り、その自己テストが PASS する
  - _Requirements: 2.5_

- [x] 7. (P) リリースの補助スクリプトの使われない口を消し、skipped の判定を試験する
  - `publish-crate.ps1` の `-ApiBase` の引数と注記を消し、スクリプトの中の定数で受ける
  - `release-scripts-test.ps1` の末尾に項目 12 を足す。一時のフォルダの偽の vsce が「公開済み」と答え、`publish-vsix.ps1` が `status=skipped`・終了コード 0 になること。`publish` を呼べば偽の vsce が 1 で終わる
  - 項目 1〜11 と `github-release.ps1` は変えず、冒頭の説明に `publish-vsix.ps1` を足す
  - 完了の姿: `release-scripts-test.ps1` の項目 1〜12 が全部 ok になる（ネットワークを使わない）
  - _Boundary: ReleaseScripts_
  - _Requirements: 10.4, 10.5, 10.6, 10.9_

- [ ] 8. ワークフローと steering
- [x] 8.1 (P) Build ワークフローにマニュアルの軽い検査の job を足す
  - `needs` を持たない job `manual-light`（name `manual-light-checks`）を足し、checkout・Node 22・軽い検査 3 つを `manual.yml` と同じ段の名前で走らせる
  - build の job の `upload-artifact` を v7 に上げる。第三者の action はタグのまま
  - 完了の姿: `actionlint` が 0 件で、`build.yml` に軽い検査 3 つの段を持つ `needs` の無い job がある
  - _Boundary: LightCheckJob_
  - _Requirements: 1.1, 1.2, 1.3, 1.5, 4.7, 8.6_

- [x] 8.2 (P) Manual ワークフローの起動・権限・段を直す
  - push と PR の path filter に `editors/vscode/syntaxes/**` と `.claude/skills/pasta-check/**` を足し、冒頭の注記を実際の条件と流れに合わせる
  - 全体の権限を `contents: read` だけにし、deploy の job にだけ `pages: write`・`id-token: write` を与える
  - `pasta_check` の再ビルドの 3 段と Setup Pages の段を消し、upload と deploy の条件を `github.ref == 'refs/heads/main'` にする
  - mdBook を env の版 0.5.4 で公式の配布物から取る段にし（新版と旧版の両方が使う）、`peaceiris/actions-mdbook` を外す
  - `setup-node` v7・`upload-pages-artifact` v5（`include-hidden-files: true`）・`deploy-pages` v5 に上げる
  - 完了の姿: `actionlint` が 0 件で、build の job に書く権限が無く、Node 20 の major と `peaceiris` が残っていない
  - _Boundary: ManualWorkflow_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3, 3.4, 3.5, 5.1, 8.3, 8.6_

- [x] 8.3 (P) Release setup check を固定し Node 22 にする
  - `Azure/login` を設計の SHA と行末の `# v3` に、`setup-node` を v7・`node-version` を 22 にする
  - 完了の姿: `actionlint` が 0 件で、第三者の action が SHA と行末のタグのコメントだけになっている
  - _Boundary: ReleaseSetupCheck_
  - _Requirements: 4.2, 4.3, 8.2, 8.6_

- [x] 8.4 (P) Release ワークフローの action を固定し、道具の hash を確かめる
  - `NODE_VERSION` を 22 にし、`wasm-pack`・`cargo-about` の SHA-256 を env に置き、道具の段で展開の前に `Get-FileHash` で比べて違えば `throw` する
  - build・publish-crates・publish-vsce の job の第三者の action（`rust-toolchain`・`rust-cache`・`crates-io-auth-action` 5 か所・`Azure/login`）を設計の SHA と行末のタグのコメントにする。`toolchain: stable` は足さない
  - build の job の「Check generated skill references」の段とその注記を消し、`setup-node` v7・`upload-artifact` v7・`download-artifact` v8 に上げる
  - 保つ行（配布物 5 つの作成と検査・Cargo.lock の確認・upload・公開の job の `run:`・report）は 1 文字も変えない
  - 完了の姿: `actionlint` が 0 件で、`git diff origin/main` の hunk が上の行だけにある
  - _Boundary: ReleaseWorkflow_
  - _Requirements: 1.6, 4.1, 4.3, 4.4, 4.5, 4.6, 4.7, 8.1, 8.5, 8.6, 10.3_

- [ ] 8.5 steering の CI とマニュアルの行を実際に合わせる
  - `tech.md` の設計の 6 か所（build.yml の軽い検査・manual.yml の起動と流れ・権限・release の鮮度の照合・mdBook 0.5.4・リンク検証の 3 スキルと `editors/`）だけを直す
  - 完了の姿: `tech.md` の 6 か所が 8.1〜8.4 と 2.1・6.2 の実際と一致し、ほかの行に差分が無い
  - _Depends: 6.2, 8.1, 8.2, 8.3, 8.4_
  - _Requirements: 1.1, 1.4, 1.6, 2.4, 2.5, 3.6, 8.4_

- [ ] 9. 検証と台帳
- [ ] 9.1 手元で全部の検査とワークフローの確かめを通す
  - 依存の無い自己テストと軽い検査 3 つ・本文の検査・`release-scripts-test.ps1` を走らせる。mdBook の出力が要る `verify-search`・`build-index-test` は一時のフォルダに作った出力で走らせる
  - `actionlint .github/workflows/*.yml` が 0 件。固定した SHA ごとに `gh api repos/<owner>/<repo>/commits/<sha>` が 200 で、行末のタグがその SHA を指す
  - `uses:` の grep で、release と setup check の第三者の action が全部 SHA、`build.yml` と `actions/*` がタグ、Node 20 の major が残っていないことを確かめる。`release.yml` の保つ行に hunk が無いことを確かめる
  - 完了の姿: 上の全部が通り、jsdom・vscode-textmate が要るテストだけが 9.2 に残っている
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.7, 8.6, 10.2, 10.3_

- [ ] 9.2 ブランチの上で Build と Manual のワークフローを回す
  - ブランチの push を開発者に確かめてから、`gh workflow run build.yml --ref <branch>` と `gh workflow run manual.yml --ref <branch>` を走らせる
  - `build.yml` は `manual-light-checks` を含む全部の job が緑。`manual.yml` は build の job が緑（自己テスト全部・新版と旧版の 0.5.4 での生成と着色を含む）で、upload と deploy は飛ぶ
  - マージの後の確かめ（Pages の新版と `/classic/`・main での `release-setup-check.yml`・次のリリースタグの公開の結果）は、kiro-complete への申し送りとして残す
  - 完了の姿: 2 つの実行が緑で、その run の URL が控えてある
  - _Requirements: 1.1, 3.5, 8.5, 10.2, 10.7_

- [ ] 9.3 指摘の台帳を全部「直した」にする
  - `research.md` の台帳の B1〜B11・N1〜N17 の 28 件に、直したタスクとコミットを書き、全部「直した」にする
  - 完了の姿: 台帳に「直した」でない指摘が 1 件も無い
  - _Requirements: 10.1_

## Implementation Notes
- 3.1: `maskFences` が空にした行の連続では空行 1 つを挟む 2 つのフェンスを分けられないため、`extractCodeFences` は閉じの行を `maskFences` に 1 行足して確かめる作りにした。kiro-complete で design.md の ContentVerify の文言を直す。
- 5.2: neutralizer の写しと正準の比べる行は 82 行（design.md の「77 行」は古い数え）。kiro-complete で design.md を 82 に直す。
