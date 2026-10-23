# Implementation Plan

- [x] 1. 下地: 変更前の記録を残す
  - cargo を走らせる前に、環境変数 `NoDefaultCurrentDirectoryInExePath` を外す（LuaJIT のビルドが止まるため）
  - 変更前の `release.ps1` を pwsh から直に呼んで通す（`-ExecutionPolicy Bypass` は使わない）
  - `pasta.dll.zip`・`hello-pasta.nar`・`pasta_check.zip` の中身の名前の一覧と、段の見出しの並び（`[1/7]`〜`[7/7]`）を、`target\pasta-skills-release-asset-baseline\` の `assets-before.txt`（配布物ごとの名前の一覧）と `steps-before.txt`（段の見出し）に書き出す（`target\` は git の外なので、後のタスクの担当も同じ場所で読める）
  - 完了の姿: 変更前の 3 つの配布物の一覧と段の見出しが上の 2 つのファイルにあり、2.3 の比較に使える
  - _Requirements: 2.9_

- [x] 2. リリースビルドに pasta-skills.zip の段を足す
- [x] 2.1 pasta-skills.zip を作って検査する段を足す
  - 段 5（`pasta.dll.zip`）の後に段 6「Creating pasta-skills.zip」を置く。見出しは最初から `[6/8]` で書く（ほかの段の番号の付け直しは 2.2）。リリースの段（3〜8）の中にあり、`-SkipSetup`・`-SkipDllBuild` でも必ず走る。冒頭のパスの設定に `pasta-skills.zip` の場所を足す
  - 古い zip を消す → 配るスキル 3 つのフォルダの追跡しているファイルの一覧を git から取る（失敗なら `ERROR: git ls-files failed`）→ 名前を `/` 区切りで組んで（先頭の `.claude/skills/` を外す）ファイルを書き、最後にルートの `LICENSE` を `LICENSE` の名前で書く → 書く途中の例外では zip を閉じて消して失敗する、の順にする
  - 書いた zip を読み直し、必須の名前（3 つの `SKILL.md` と `LICENSE`）・期待の一覧との過不足のない一致（大文字と小文字を区別）・`\` を含むエントリが無いこと、を検査する。合わなければ `ERROR: pasta-skills.zip content mismatch` と、空でない `Missing:`・`Extra:`・`Bad path:` の行を出し、zip を消して失敗する
  - `Compress-Archive` と `ZipFile.CreateFromDirectory` は使わず、`ZipFile.Open(…, 'Create')` と静的に呼ぶ `ZipFileExtensions.CreateEntryFromFile` で書く。Windows PowerShell 5.1 で動く書き方に限り、関数に括り出さない
  - 完了の姿: 段 6 の行をスクラッチの写しにして pwsh で走らせると `release\pasta-skills.zip` ができ、成功の行 `Created pasta-skills.zip (<N> files: …)` が出る。最上位は 3 つのフォルダと `LICENSE` だけ（通しの実行は 2.3 で行う）
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 2.2, 2.3, 2.4, 2.6, 2.7_

- [x] 2.2 段の並びと表示・release.bat の説明を合わせる
  - 段の見出しを `[1/8]`〜`[8/8]` に。今の段 6・7（版の確認・大きさとまとめ）を段 7・8 にし、中身は変えない。段 3〜5 は触らない
  - 冒頭の説明（作る物に `pasta-skills.zip`、`-SkipSetup` の「run release steps (3-8) only」）と、リリースの段の範囲の注釈（`Release Phase (Steps 3-8)`）を合わせる
  - 頭の表示とまとめの `Check Zip:` の次に `Skills Zip:` の行を足す
  - `release.bat` の冒頭の説明（作る物・段の並び・`-SkipSetup` の段の範囲）を合わせる
  - 完了の姿: 通しの実行で段の番号が 1〜8 で並び、まとめに `pasta-skills.zip` の場所が出る。`release.bat` の説明が 8 段の並びを示す
  - _Requirements: 2.1, 2.5, 2.8, 2.9_

- [x] 2.3 手元のリリースビルドで通しの確かめをする
  - pwsh から `release.ps1` を直に通し、`release\` に 4 つの配布物ができ、まとめに `Skills Zip:` が出ることを確かめる
  - `pasta-skills.zip` を展開し、最上位が 3 つのフォルダと `LICENSE` だけで、各フォルダを `.claude/skills/<名前>/` と `diff -r` して差が無く、`LICENSE` がルートと同じバイト列で、エントリに `\` が無いことを確かめる
  - `pasta.dll.zip`・`hello-pasta.nar`・`pasta_check.zip` の中身の名前の一覧が、1 の記録（`target\pasta-skills-release-asset-baseline\assets-before.txt`）と同じことを確かめる
  - `-SkipSetup` でも段 6 が走り、`pasta-skills.zip` の更新時刻が変わることを確かめる
  - 失敗の道を、段 6 の行のスクラッチの写しで確かめる: (a) 前の zip を置いたうえで存在しないファイルを一覧に混ぜ、書く途中で失敗して zip が残らない、(b) 検査の部分に `Compress-Archive` で作った `\` のエントリの zip を与え、`Missing:`・`Extra:`・`Bad path:` が出て zip が消える
  - 段 6 の行を `powershell.exe -NoProfile -Command -` で走らせ、pwsh 7 の zip とエントリの一覧が同じで `\` が無いこと、5.1 の構文解析を `release.ps1` に当ててエラーが無いことを確かめる
  - `release.bat` での通しの実行は、開発者の確かめに回す（エージェントは `-ExecutionPolicy Bypass` を走らせられない）と報告に書く
  - 完了の姿: 上の確かめがすべて通り、`release\` の 4 つの配布物が 3.1 の確かめに使える形で残っている
  - _Depends: 1_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.8, 2.9_

- [x] 3. リリース CI の配線
- [x] 3.1 (P) リリース CI の build の job で照合し、pasta-skills.zip を検査して渡す
  - 「Setup Node」の直後に「Check generated skill references」の step（`node book/tools/gen-skill-refs.mjs --check`）を、タグの時点の鮮度を保証する理由の注釈付きで足す
  - リリースビルドの段の名前に `pasta-skills.zip` を足し、実行の行は変えない。冒頭の説明を「5 つの配布物」にする
  - 配布物の検査の `pasta_check.zip` の項目の後に、`pasta-skills.zip` の項目を足す: 必須の名前（3 つの `SKILL.md` と `LICENSE`）、タグのコミットの木（`git ls-tree -r --name-only HEAD`）から取った期待の一覧との過不足（大文字と小文字を区別）、`\` を含むエントリ、をそれぞれ `$errors` に積む
  - 受け渡しの path に `release/pasta-skills.zip` を足す。verify・gate・公開系の job、`Cargo.lock` の検査、`build.yml` は変えない
  - 検査の `run` をスクラッチの `.ps1` に写し、2.3 の `release\` で通ること、`kiro-x/SKILL.md` を足した zip・`references/` の 1 つを抜いた zip・`pasta-check/SKILL.md` を抜いた zip で、余計な物・足りない物・無い `SKILL.md` の行が出ることを確かめる（VSIX の項目の NG の行は除いて読む）
  - `node book/tools/gen-skill-refs.mjs --check` がワークツリーで通ること、`actionlint` が `release.yml` にエラーを出さないことを確かめる
  - 完了の姿: 正しい zip では `pasta-skills.zip` の項目が通り、3 通りの壊れた zip ではそれぞれの行が出る。照合の step が build の job の長いビルドの前にある
  - _Boundary: ReleaseBuildJob_
  - _Depends: 2.3_
  - _Requirements: 2.10, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 7.3_

- [x] 3.2 (P) GitHub Release の添付を 6 つにし、欠けのテストを合わせる
  - 添付の名前の一覧に `pasta-skills.zip`（版を含まない名前）を足して 6 つにし、`-AssetDir` の説明・添付の数の注釈・skip の文言・公開の注釈を「6 つ（配布物 5 つと詳しいリリースノート）」に合わせる。有無の検査・下書き → 添付 → 公開・再実行の判定の流れは変えない
  - スクリプトのテストの項目 10 を 6 つの前提に直す（置くファイルに `pasta-skills.zip` を足し、出力に `pasta-skills.zip` が出ないことも検査する）
  - 項目 11 を足す: 別の一時のフォルダに `pasta-skills.zip` のほかの 5 つを置き、終了コード 1・出力に `pasta-skills.zip` がある・出力に `pasta_check.zip` が無い、を検査する
  - 完了の姿: `release-scripts-test.ps1` を走らせると、項目 1〜11 がすべて ok（ネットワークと gh を使わない）
  - _Boundary: GitHubReleaseAssets, ReleaseScriptsTest_
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

- [x] 4. release スキルの添付の数え方を 5/5 にする
  - `.claude/skills/**` の編集が分類器に拒否されたら、回避策を探さずに、入った分と残りを示して開発者の許可を求める
- [x] 4.1 (P) リリースの状態の読み取りを n/5 にする
  - 配布物の配列に `pasta-skills.zip` を足し、表示の `/4` を `/5` に、公開済みの判定を 5 つそろったときだけにする。リリースノートの添付は数えない
  - 0.4.1 に向けて走らせ（gh の認証がある場合）、`添付不足（公開・添付 3/5）` と出ることを確かめる。認証が無ければ、コードの読み合わせで判定が `-eq 5` であることを確かめたと報告する
  - 完了の姿: 状態の読み取りが GitHub Release の添付を `n/5` で示し、5/5 の公開だけを公開済みにする
  - _Boundary: ReleaseStateReader_
  - _Requirements: 5.1, 5.2_

- [x] 4.2 (P) release スキルの手順書の読み方を 5 つの配布物に合わせる
  - 手順書の状態の表の読み方を `5/5`・`n/5` に、配布物の並びに `pasta-skills.zip` を足し、0.4.1 までの版は `添付不足（公開・添付 3/5）` と出ることを書く
  - スキルの見どころの「添付 4/4」を「添付 5/5」に。手で添付する案内（F5）は数を含まないことを確かめ、変えない。公開先の「7/7」は変えない
  - 完了の姿: `release` スキルの文書に `4/4`・`n/4`・`2/4` の言い方が残らず、`7/7` はそのまま
  - _Boundary: ReleaseSkillDocs_
  - _Requirements: 5.3, 5.4, 5.5_

- [x] 5. 文書を合わせる
- [x] 5.1 (P) マニュアルの「はじめに」にスキルの受け取り方を書く
  - 関連リンクの表の「リリースページ」の行の、検査の CLI の後に、`pasta-skills.zip` を固定のリンクで足す
  - 表の後・「このマニュアルが対象とするバージョン」の前に、節「AI にゴーストを書かせるとき」を「である」調で置く: 中身（3 つのスキルとそれぞれの役目）、落とし方（固定のリンクと版ごとのリリースページ）と置き場所（ゴーストの開発フォルダの `.claude/skills/` に展開した木）、版（同じ版を使う・版を上げるときは 3 つのフォルダを消してから展開する理由）
  - claude.ai へのアップロード・スキルごとの zip の作り直し・ほかの AI の道具ごとの置き場所は書かない。章は足さず、台詞の行を含めない
  - `link-check.mjs` と、`manual.yml` が行う検査（台詞部品・章の数・`verify-content` など）を手元で走らせる。`book/node_modules` が無いワークツリーでは、highlight・theme の自己テストは環境の理由で落ちるので、対象外と報告する
  - 完了の姿: `introduction.md` に固定のリンクと新しい節があり、マニュアルの検査がすべて通る
  - _Boundary: ManualSkillsSection_
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 5.2 (P) 配布物の一覧と段の並びを書いた文書を合わせる
  - ルートの `README.md` の直リンクの並びの `pasta_check.zip` の後に、`pasta-skills.zip` の 1 行を足す
  - `crates/pasta_sample_ghost/RELEASE.md`・`README.md` の配布物・build の役目・添付 6 つ・`release.ps1` の 8 段の並び（段 6 を足し、版の確認 7・大きさ 8、「手順 3〜8 は毎回走る」）を合わせる
  - steering の `tech.md`（成果物 5 つ・添付 6 つ）・`structure.md`（`release.ps1` の 8 段・`release/` の中身）を合わせる
  - `pasta-check` スキルの `SKILL.md` の hello-pasta の段の並び（段 6 を足し、版の確認 7・大きさ 8、「段 3・段 5〜8」）、`release/` の木、配布物の並びに `pasta-skills.zip` を足す。ほかの所は変えない。分類器に拒否されたら、入った分と残りを示して開発者の許可を求める
  - 完了の姿: 上の文書に `pasta-skills.zip` があり、配布物 4 つ・添付 5 つ・7 段の古い記述が残らない
  - _Boundary: AssetListDocs_
  - _Depends: 2.2_
  - _Requirements: 7.1, 7.2_

- [x] 6. 全体の確かめ
  - `release-scripts-test.ps1`・`node book/tools/gen-skill-refs.mjs --check`・`link-check.mjs` をまとめて走らせ、すべて通ることを確かめる
  - リポジトリを `4/4`・`n/4`・`/7]`・「添付 5 つ」・「配布物 4 つ」など、GitHub Release の添付と配布物の総数を指す古い言い方で grep し、本仕様の範囲（完了 spec と過去の記録を除く）に残りが無いことを確かめる。「4 つ」は文脈で判断する（`release.ps1` が `release/` に作る物は変更後も 4 つで正しい）
  - `git diff main --stat` で、`build.yml`・gate・段 3〜5 の中身・公開系の job の行を変えていないことを確かめる
  - 完了の姿: 上の検査がすべて通り、変更が File Structure Plan のファイルだけに収まっている
  - _Depends: 3.1, 3.2, 4.1, 4.2, 5.1, 5.2_
  - _Requirements: 2.9, 3.7, 4.6, 5.5, 6.5, 7.1_

## Implementation Notes

- 段の行を `powershell.exe -NoProfile -Command -` に流して 5.1 で走らせるときは、入力の末尾に空行を置く（無いと最後の文が走らない）
- 2.3: pwsh 直の通し・`-SkipSetup`・失敗の道・5.1 の同一性はすべて確認済み。`release.bat`（`-ExecutionPolicy Bypass`）での通しは開発者の確かめに回した
- 完了時にその場で解決: `release.ps1` のまとめの表示で `Skills Zip:` の値だけ 1 桁ずれていたのを、ほかのラベルの桁を合わせてそろえた
