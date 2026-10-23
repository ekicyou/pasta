# ギャップ分析: pasta-skills-release-asset

調べた日: 2026-10-23（main 4f830dab の上のワークツリー）。対象は `requirements.md` の R1〜R7。

## 1. 分析のまとめ

- **作りの大半は `pasta_check.zip` の段と検査の写しで済む**。`release.ps1` の段 3（古い zip を消す → 作る → `ZipFile.OpenRead` で中身を検査）と、`release.yml` の「Verify release assets」の `Get-ZipEntries`＋`$errors` の形が、そのまま手本になる。新しい道具は要らない（R2.7）。
- **一番の落とし穴は zip の中のパスの区切り**。手元で確かめたところ、Windows PowerShell 5.1（`release.bat` が呼ぶ `powershell.exe`）の `Compress-Archive` も `[ZipFile]::CreateFromDirectory` も、フォルダを詰めると `a\SKILL.md` のように **`\` で記録する**。PowerShell 7（リリース CI）は `a/SKILL.md`。今の 2 つの zip はフォルダを含まないので表に出なかった。フォルダを含む `pasta-skills.zip` は、エントリの名前を自分で `/` に組んで書く必要がある（R1.5・R2.6）。
- **添付の数は 3 か所で決め打ち**（`github-release.ps1` の `$names` と skip の文言、`release-state.ps1` の配列と「/4」、`procedure.md` の表の読み方）。どれも数行の書き換え。添付の欠けのテストは項目 10 の形をそのまま使える（R4.6）。
- **生成ファイルの鮮度は、リリースの道では誰も見ていない**。`gen-skill-refs --check` は `manual.yml`（path filter 付き・PR を CI を待たずにマージする運用）だけで走り、リリースの関門（`build.yml`）には無い。0.4.3 の `ponytail-manual-ci` の Must fix 1 が、まさにこの穴を関門に足す予定。本仕様で zip に生成ファイルを詰める以上、先に塞ぐかを決める必要がある（OPEN QUESTION 2）。
- **推奨は Option A（今の部品を伸ばす）**。規模は S（1〜3 日）、リスクは低い。`.claude/skills/` の下の編集（`release` スキル・`pasta-check/SKILL.md`）が分類器に止められる見込みなので、その場面の段取りだけ要る。

## 2. 今の状態

### 2.1 配布物を作る所

| 所 | 今の中身 | 本仕様との関係 |
|---|---|---|
| `crates/pasta_sample_ghost/release.ps1`（331 行） | 7 段。段 1 DLL のビルド、段 2 DLL・ライセンス表示・`scripts/` の配置（ここまで setup）、段 3 `pasta_check.zip`（古い zip を消す → x64 のビルド → `Compress-Archive` → 中身がちょうど 2 つかの検査）、段 4 その exe で `pasta_check release`、段 5 `pasta.dll.zip`、段 6 版の確認、段 7 `.nar` の大きさとまとめ | 段を 1 つ足す（8 段になる）。段 3 の L165〜227 が手本。段 3 は `-SkipSetup`・`-SkipDllBuild` でも毎回走る作りなので、同じ扱いにする（R2.5）。まとめ（L312〜321）に 1 行足す（R2.1） |
| ルートの `release.bat` | `powershell.exe -ExecutionPolicy Bypass -File …release.ps1`。冒頭の説明に段の並び（「1-2」「3-7」）と作る物 | 説明を直す（R2.8）。**5.1 で走る**ので、足すコードは 5.1 で動き、かつ `/` 区切りの zip を作る書き方にする |
| `.claude/skills/pasta-*`（追跡 32 ファイル・約 800 KB） | `pasta-ghost-authoring` 14・`pasta-lua-coding` 15・`pasta-check` 3。どれも `SKILL.md`＋`references/*.md`。サブフォルダはこの 2 段だけ | 詰める元。手元の作業ツリーに未追跡のファイルは無い（`git status --ignored` で確かめた） |
| ルートの `LICENSE` | MIT License（Copyright (c) 2025 ekicyou）。作業ツリーでは CRLF | そのまま詰める（R1.4）。改行は checkout の設定（Windows では autocrlf で CRLF）に従うので、手元と CI で同じになる見込み |

**zip の区切りの実験**（2026-10-23、スクラッチで実施）

```text
Windows PowerShell 5.1（Microsoft.PowerShell.Archive 1.0.1.0）
  Compress-Archive -Path a,LICENSE      → a\SKILL.md, a\references\r.md, LICENSE
  [ZipFile]::CreateFromDirectory(src)   → LICENSE, a\SKILL.md, a\references\r.md
PowerShell 7
  Compress-Archive -Path a,LICENSE      → a/SKILL.md, a/references/r.md, LICENSE
```

- どちらもフォルダのエントリ（`a/`）は作らない。
- `\` のエントリは、Windows の Explorer では展開できることが多いが、ZIP の仕様（APPNOTE 4.4.17）は `/` を求め、ほかの展開の道具ではファイル名に `\` を含む 1 つのファイルになる。手元とリリース CI で中身の一覧が食い違う原因にもなる。
- 5.1 でも 7 でも同じ結果を得るには、`[System.IO.Compression.ZipArchive]` を `Create` で開き、`CreateEntryFromFile`（`System.IO.Compression.FileSystem` の拡張）か `CreateEntry`＋ストリームの写しで、エントリの名前を `pasta-check/references/nar-spec.md` のように自分で組む。どちらも 5.1 の .NET Framework 4.x で使える。

### 2.2 リリース CI（`.github/workflows/release.yml`、589 行）

- 冒頭の説明 L3「build（4 つの配布物）」（R7.3）。
- build の job（L58〜214）
  - 「Setup Node」（L112）が既にある（VSIX のため）。`gen-skill-refs.mjs` は npm の依存を持たず（`node:fs`・`node:path` と `book/tools` の中のモジュールだけ）、手元で 0.1 秒で終わる。照合を足すなら道具は増えない。
  - 「Build pasta.dll.zip, hello-pasta.nar and pasta_check.zip」（L120）は `release.ps1` を呼ぶだけ。段の名前に `pasta-skills.zip` を足す。
  - 「Verify release assets」（L131〜198）は `Get-ZipEntries` で一覧を取り、`$errors` に積み、最後にまとめて `::error::` と `exit 1`。`pasta-skills.zip` の検査はこの形で足せる。タグのコミットが checkout 済みなので、期待する一覧は `.claude/skills/<3 つ>/` を辿るか `git ls-files` で作れる（R3.3）。
  - 「Upload release-assets」（L203〜214）の `path` に 1 行足す（R3.6）。`if-no-files-found: error` なので欠けは upload でも止まる。
- verify の job（ubuntu・L29〜53）: タグと版の検査。ここに照合を置けば、関門の長いビルドより前に速く失敗する（OPEN QUESTION 2 の置き場所の候補）。
- 後続の job: publish-vsce は `release-assets/*.vsix` で拾い、publish-crates は配布物を使わない。github-release は `-AssetDir release-assets` を渡すだけで、添付の名前は `github-release.ps1` が持つ。影響は無い。
- `ponytail-manual-ci` が触る行（公開系の job の action の固定・`NODE_VERSION`・Pages の権限）とは重ならない。ただし Must fix 1（関門で `gen-skill-refs --check` が走らない）は、OPEN QUESTION 2 と同じ穴を扱う。

### 2.3 GitHub Release の添付（`.github/scripts/release/github-release.ps1`、132 行）

- L18 の引数の説明、L71〜72 の `$names`（5 つ）、L88〜89 の skip の条件の文言（「5 つ（配布物 4 つと詳しいリリースノート）」）、L103 の注釈「5 つがそろってから公開する」。`$names` に `pasta-skills.zip` を足せば、検査・不足分の添付・公開の順序（R4.3〜R4.5）は今の作りのまま 6 つに広がる。
- 添付の有無の検査（L71〜79）は `gh` を呼ぶ前に行う。欠けていれば終了コード 1 と「添付がそろっていない（…に無い: 名前）」。
- `release-scripts-test.ps1`（256 行）の項目 10（L236〜243）は、4 つだけを置いて `pasta_check.zip` の欠けを確かめる。6 つになっても、`pasta_check.zip` と `pasta-skills.zip` の 2 つが欠けとして出るので**項目 10 は通り続ける**が、題名の「添付 5 つのうち」が古くなり、`pasta-skills.zip` の欠けを試す項目が無い。項目 10 に `pasta-skills.zip` を置く行を足し、項目 11（`pasta-skills.zip` だけが無い）を足すのが素直（R4.6）。このテストは `build.yml` L149 で毎回の CI でも走る。

### 2.4 release スキル

- `scripts/release-state.ps1`（110 行）L84〜87: 配布物 4 つの配列と「/4」、公開済みの判定 `$n -eq 4`（R5.1・R5.2）。
- `procedure.md`（504 行）L471 の状態の表の読み方、L477 の「配布物は … の 4 つで、`pasta_check.zip` を添付していない 0.4.1 までの版は `添付不足（公開・添付 3/4）`」（R5.3）。0.4.2 から 2 つが同時に増えるので、0.4.1 までは `3/5` になる。L396（F5）は「足りない配布物」と書くだけで数を含まない（R5.4 は確かめだけの見込み）。
- `SKILL.md`（169 行）L166「配布物の名前・個数（定型コマンド A の「添付 4/4」の判定）」（R5.4）。
- 公開先の数「7/7」は GitHub Release を 1 つと数えるので変わらない（R5.5）。
- `.claude/skills/release/` の編集は分類器が Self-Modification として止める（brief の Constraints・メモリ）。止められたら、回避策を探さずに、入った分と残りを示して開発者の許可を求める。

### 2.5 文書

| 所 | 今の中身 | 本仕様との関係 |
|---|---|---|
| `book/src/introduction.md`（92 行）L38 | 関連リンクの表の「リリースページ」の行に、配布物 3 つの固定のリンクと VSCode 拡張 | `pasta-skills.zip` を足す（R6.1）。スキルの受け取り方の短い説明の置き場所の第一候補（OPEN QUESTION 3） |
| `book/src/getting-started/prerequisites.md` | 「必要なもの」に `hello-pasta.nar`・`pasta.dll.zip` の入手。台詞（クローディアとアンソニー）で進む入門の章 | 置き場所の第二候補。入門の流れの中で AI を前提にすることになる |
| `book/src/debug/validate.md` | `pasta_check.zip` の入手を書いた先例（L20〜22） | 書き方の手本 |
| マニュアルの検査 | 章の数 63 を 3 つの検査が持つ（`gen-skill-refs-test.mjs` K-10・`verify-scripts-test.mjs` の T-syntax/T-intro/T-outro）。章を足すと 3 か所と `SUMMARY.md` を直す | 既存の章に節を足すなら章の数は変わらない（R6.5）。`introduction.md`・`prerequisites.md` は生成ファイルの元（`GENERATION_MAP`）ではないので、直しても生成ファイルの作り直しは要らない |
| `crates/pasta_sample_ghost/RELEASE.md`（107 行） | L6・L42・L45 の配布物の並びと「配布物 4 つと詳しいリリースノートの 5 つ」、L107 の「7 段」 | 5 つ・6 つ・8 段に（R7.1） |
| `crates/pasta_sample_ghost/README.md`（156 行） | L42 の `release.ps1` の説明、L66〜78 の 7 ステップの一覧と `-SkipSetup` の説明 | 段を足す（R7.1） |
| `.kiro/steering/tech.md` L212 | 成果物 4 つ・添付 5 つ | 5 つ・6 つに（R7.1） |
| `.kiro/steering/structure.md` L226・L280・L291 | 「7 段」と `release/` の中身 | 8 段と `pasta-skills.zip` に（R7.1） |
| ルートの `README.md`（102 行）L41〜46 | 最新版の配布物の固定のリンクの並び（`.nar`・`pasta.dll.zip`・`pasta_check.zip`・VSCode 拡張） | brief の Scope に無いが、同じ並びなので足すのが自然（OPEN QUESTION 4） |
| `.claude/skills/pasta-check/SKILL.md`（186 行）L110〜125・L143〜146・L164 | hello-pasta の `release.ps1` の段の並び（段の番号つき）、`release/` の中身、配布物の並び | brief の Scope に無いが、段が増えると番号が古くなる。しかもこのファイル自体が zip に入る（OPEN QUESTION 4） |

### 2.6 配るスキルの中身（変えないが、知っておくこと）

- 3 つのスキルの中のリンクは、どれもスキルのフォルダの中で閉じている（`references/…`・`../SKILL.md`）。`pasta-ghost-authoring`・`pasta-lua-coding` は `link-check.mjs` の自己完結の検査（`skill-escape`・`skill-forbidden-ref` など。禁止のトークン `book/src`・`crates/` など）で守られている。
- `pasta-check` はその検査の対象外で、リポジトリの中の話（`crates/pasta_sample_ghost/RELEASE.md`・`crates/pasta_check/src/nar.rs` などのパス、hello-pasta の `release.ps1` の段）を文字として含む。リンクではないので壊れはしないが、ゴーストの作者の手元では意味の無い記述になる。中身の書き直しは brief の Out（`ponytail-check-release` が同じファイルの別の所を直す）なので、本仕様では扱わない。設計への申し送りとして残す。
- `SKILL.md` の `name` はどれもフォルダ名と同じで、展開して `.claude/skills/` に置けばそのまま読み込まれる形になっている。

## 3. 要件と資産の対応

| 要件 | 使う資産 | ギャップ |
|---|---|---|
| R1.1〜R1.4 zip の形・LICENSE | `.claude/skills/pasta-*`、ルートの `LICENSE` | **Missing**: zip を作る段が無い |
| R1.5・R2.6 `/` 区切り・5.1 と 7 で同じ | `System.IO.Compression`（5.1 でも使える） | **Constraint**: `Compress-Archive`・`CreateFromDirectory` は 5.1 で `\` を書く。エントリの名前を自分で組む必要がある |
| R1.6 版を示すファイルを足さない | — | 無し（足さないだけ） |
| R2.1〜R2.5・R2.8〜R2.10 段・検査・古い zip・飛ばさない・番号 | `release.ps1` 段 3 の形 | **Missing**: 新しい段。番号の振り直し（段の挿入位置によって 6・7 段目がずれる） |
| R2.7 新しい道具を要さない | PowerShell と .NET の zip | 無し |
| R3.1・R3.2・R3.3・R3.5・R3.6 CI での作成・検査・upload | 「Verify release assets」の `Get-ZipEntries`、upload の `path` | **Missing**: `pasta-skills.zip` の検査と upload の 1 行 |
| R3.4 生成ファイルの照合 | `node book/tools/gen-skill-refs.mjs --check`、build の job の「Setup Node」 | **Constraint**: 今のリリースの道に無い。`ponytail-manual-ci` の Must fix 1 と重なる（OPEN QUESTION 2） |
| R3.7 `build.yml` を変えない | — | 無し（照合を release.yml 側に置けば守れる） |
| R4.1〜R4.5 添付 6 つ | `github-release.ps1` の `$names` | **Missing**: 名前 1 つと文言 |
| R4.6 添付のテスト | `release-scripts-test.ps1` 項目 10 | **Missing**: 項目 11。項目 10 の手直し |
| R5.1〜R5.5 状態の読み取り | `release-state.ps1`・`procedure.md`・`SKILL.md` | **Missing**: 4→5 の書き換え。**Constraint**: 分類器 |
| R6.1〜R6.5 マニュアル | `introduction.md`、`debug/validate.md` の書き方 | **Missing**: 説明の節。**Unknown**: 置き場所（OPEN QUESTION 3） |
| R7.1〜R7.3 文書 | 上の表の各所 | **Missing**: 書き換え。**Unknown**: ルートの README と `pasta-check/SKILL.md` を含めるか（OPEN QUESTION 4） |

## 4. 作り方の選択肢

### Option A: 今の部品を伸ばす（推奨）

- `release.ps1` に段を 1 つ足す（`pasta-skills.zip` を作る・検査する）。`release.yml` の「Verify release assets」に検査を、upload に 1 行を足す。`github-release.ps1` の `$names` に 1 つ足し、テストに 1 項目足す。`release-state.ps1`・`procedure.md` の数を直す。文書を直す。
- ✅ `pasta_check.zip` のときと同じ流れで、読む人が迷わない。ファイルが増えない。
- ✅ 手元とリリース CI が同じ手順（R2.10）。
- ❌ `release.ps1` が 331 行から 50〜70 行ほど増える（1,000 行には遠い）。
- ❌ 検査のコードが `release.ps1` と `release.yml` の 2 か所に並ぶ（今の 2 つの zip と同じ重複）。

### Option B: zip を作る部分を別のスクリプトに出す

- たとえば `crates/pasta_sample_ghost/skills-zip.ps1`（作成と検査）を新しく置き、`release.ps1` の段と `release.yml` の検査の両方から呼ぶ。
- ✅ 検査の重複が無くなる。単体で試せる。
- ❌ 今の 2 つの zip と作りがそろわない。新しいファイルの置き場所と持ち主が増える。
- ❌ `release.yml` の検査は「作った物を別の目で確かめる」意味もあるので、同じコードを呼ぶと二重の確かめにならない。

### Option C: zip の作成は release.ps1、照合は release.yml の verify の job

- Option A に加えて、生成ファイルの照合（R3.4）を、関門より前の verify の job（ubuntu・Node は入っている）に置く。
- ✅ 照合が合わないときに、数十分のビルドの前に止まる。
- ❌ verify の job はタグと版を見る job で、配布物の検査は build の job という分け方から外れる。`ponytail-manual-ci` が関門に照合を足すと、さらにもう 1 か所になる。

## 5. 規模とリスク

- **規模: S（1〜3 日）**。今の段と検査の写しが大半で、新しい仕組みは zip のエントリの名前を自分で組む所だけ。文書の書き換えは 8〜10 か所。
- **リスク: 低**。知っている技術と今の形の延長。注意点は 2 つ: 5.1 の `\` の区切り（実験で確かめ済み・対処が分かっている）と、`.claude/skills/` の下の編集が分類器に止められること（段取りで済む）。本番の確かめ（Release に実際に載ること）は 0.4.2 のリリースまで残る。

## 6. 設計への推奨

- **推奨は Option A**。検査の重複は、今の 2 つの zip と同じ形を保つほうを取る（Option B は 3 つの zip の作りをそろえる別の機会に回せるが、要求が無いので起票しない）。
- 決めること
  1. 新しい段の位置。段 3（`pasta_check.zip`）の後に置くと段 4〜7 がずれ、`pasta-check/SKILL.md`・sample の README・RELEASE.md の番号を全部直す。段 5（`pasta.dll.zip`）の後に置くと版の確認と大きさの検査が 7・8 段目にずれるだけで済む。どちらでも `-SkipSetup` で走る位置（段 3 以降）にある。
  2. zip の書き方。`ZipArchive`＋`CreateEntryFromFile` で、エントリの名前を `<スキル名>/<相対パス>` を `/` で組む。`Compress-Archive` は使わない（5.1 で `\`）。
  3. 詰める元の一覧の取り方。フォルダを辿る（作業ツリーにある物すべて）か、`git ls-files`（追跡している物だけ）か。CI の checkout ではどちらも同じ。手元では、未追跡のファイルを置いたままにすると前者は詰めてしまう。後者は git が要る（手元・CI とも入っている）。検査の期待の一覧も同じ取り方にそろえる。
  4. 検査の厳しさ（OPEN QUESTION 1 の結論に従う）。全ファイルの突き合わせなら、期待の一覧は上の 3 と同じ取り方で作り、`Sort-Object` して比べる（今の 2 つの zip の検査と同じ形）。
  5. 生成ファイルの照合（OPEN QUESTION 2 の結論に従う）の置き場所。build の job の「Verify release assets」の前の 1 step（`node book/tools/gen-skill-refs.mjs --check`）が最小。
  6. `release-scripts-test.ps1` の項目 10 の直し方と項目 11。
- **Research Needed**
  - なし（区切りの問題は実験で確かめた）。設計で `CreateEntryFromFile` を 5.1 で実際に呼んで、`/` のエントリになることを 1 度確かめるとよい。
- **申し送りの候補**（kiro-complete のとき）
  - `ponytail-manual-ci`: 本仕様が release.yml に生成ファイルの照合を足した場合、関門へ照合を足すときに重複をどうするか。release.yml の行番号のずれ。
  - `ponytail-check-release`: `pasta-check/SKILL.md` の段の番号が変わったこと。リポジトリの中の話（`crates/…` のパス）が配布物にそのまま入ること（中身の直しをするなら、この観点も）。
  - `pasta-dll-multiarch`: `release.ps1` の段の並びと、配布物の検査が 1 つ増えたこと。

## 7. 要件ディスカッションへの OPEN QUESTIONS

**2026-10-23 の要件ディスカッションで、5 件すべてを下の推奨案で確定した**（開発者の一括承認）。requirements.md の【仮定】の印は外した。

1. **zip の検査の厳しさ**（brief の議題の候補）。【仮定】全ファイルの突き合わせ（R1.2・R2.2・R3.3）。推奨: 全ファイルの突き合わせ。理由: 期待の一覧はタグのコミットの 3 つのフォルダから数行で作れ、今の 2 つの zip の「ちょうどこの中身」の検査と同じ考え方になる。`SKILL.md` と `LICENSE` だけを見る形では、`references/` の欠け（`SKILL.md` からのリンクが切れたスキルが配られる）と、余計なファイルの混入を見逃す。代わりの案: 最上位の形（3 つのフォルダと `LICENSE` だけ）＋ 3 つの `SKILL.md` の存在だけを見る（brief の Desired Outcome の最小の読み方）。
2. **生成ファイルの照合をリリースの道に足すか**。【仮定】足す（R3.4）。推奨: 足す（build の job に `gen-skill-refs --check` の 1 step）。理由: 本仕様で生成ファイルを利用者に配るようになるのに、今の照合は `manual.yml` だけで、PR を CI を待たずにマージする運用ではタグの時点の鮮度を保証しない。npm の依存が無く 0.1 秒で終わる。`ponytail-manual-ci`（0.4.3）が関門に足す予定の検査と重なるが、0.4.2 で zip を出す前に塞ぐほうが「起票に逃がさない」の方針に合う。重複は申し送りで `ponytail-manual-ci` が整理する。代わりの案: 足さずに `ponytail-manual-ci` に任せる（brief の「CI の `--check` が鮮度を見る」の読みのまま）。
3. **マニュアルのどこに受け取り方を書くか**（brief の議題の候補）。【仮定】`introduction.md` に短い節を足す（R6.2）。推奨: `introduction.md` の「関連リンク」の後に「AI にゴーストを書かせるとき」のような短い節（落とし方・置き場所・同じ版を使うこと）。理由: 読者が最初に開く頁で、配布物の並びと同じ所にあり、章を足さないので章の数の検査 3 か所を触らずに済む。生成ファイルの元でもない。代わりの案: 入門の `prerequisites.md` の「必要なもの」に足す（入門の台詞の流れに AI を持ち込むことになり、入門は AI 無しで進める作り）／新しい章を立てる（章の数の検査と `SUMMARY.md` を直す。内容に対して重い）。
4. **brief の Scope に無い文書を含めるか**（ルートの `README.md`・`release.bat` の説明・`pasta-check/SKILL.md` の段の並びと配布物の並び）。【仮定】含める（R2.8・R7.1・R7.2）。推奨: 含める。理由: どれも同じ配布物の並びや段の番号を書いており、直さないと本仕様の直後から古くなる。先例 `pasta-check-release-asset` も同じ所を直した。`pasta-check/SKILL.md` は zip に入るので、古い段の番号を利用者に配ることにもなる。編集は分類器に止められる見込みなので、その場面で開発者の許可を求める（brief の Constraints と同じ扱い）。代わりの案: `pasta-check/SKILL.md` は `ponytail-check-release`（0.4.3）に申し送る（その間、0.4.2 の zip に古い段の番号が入る）。
5. **版の示し方と更新の案内**。【仮定】zip の中に版を示すファイルは足さず（R1.6）、マニュアルに「使っている pasta と同じ版の zip を使う・版を上げるときは 3 つのフォルダを置き換える」と書く（R6.3）。推奨: そのとおり。理由: brief の Desired Outcome が zip の中身を「次の形だけ」と決めており、版は Release ごとの添付で示せる。上書きで展開すると、新しい版で消えた `references/` のファイルが残るので、置き換えの一言は要る（回避の書き方ではなく、置き場所の説明の一部）。代わりの案: zip の中に `VERSION` などを足す（brief の形から外れる）／案内を書かない（古いファイルが残る）。

## 8. 設計の調査（2026-10-23・kiro-spec-design）

発見の種別は「既存の仕組みの拡張」（light discovery）。外の新しい依存は無いので、Web の調査は行わず、手元のコードと実験で確かめた。

### 8.1 読んだもの

- 規則: `kiro-spec-design` の `design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`、テンプレート `design.md`・`research.md`。
- 先例: 完了 spec `pasta-check-release-asset` の `design.md`（段の足し方・検査の形・文書の直し方・テストの立て方を写した）。
- steering: `tech.md` のリリース CI と `manual.yml` の節、`structure.md` の `release.ps1`・`release/` の行。
- コード: `release.ps1`（全体）、`release.bat`、`release.yml` の build の job（L58〜214）、`github-release.ps1`（全体）、`release-scripts-test.ps1` の項目 10、`release-state.ps1` L75〜110、`procedure.md` L471・L477、`release/SKILL.md` L166、`pasta-check/SKILL.md` L100〜170、`book/src/introduction.md`、ルートの `README.md` L30〜55、`crates/pasta_sample_ghost/README.md` L60〜85。

### 8.2 zip の書き方の実験（Windows PowerShell 5.1 と pwsh 7）

スクラッチで、このワークツリーの実物（3 つのスキルの追跡ファイル 32 個とルートの `LICENSE`）を、同じコードで 5.1 と 7 から zip にした。コードは `powershell.exe -NoProfile -Command -`（標準入力から読ませる。`-ExecutionPolicy Bypass` は使わない）と `pwsh -NoProfile -Command -` に同じものを流した。

```text
Add-Type -AssemblyName System.IO.Compression / System.IO.Compression.FileSystem
$files = git -C <ws> ls-files -- .claude/skills/pasta-ghost-authoring .claude/skills/pasta-lua-coding .claude/skills/pasta-check
$zip = [ZipFile]::Open($out, [ZipArchiveMode]::Create)
foreach ($f in $files) { [ZipFileExtensions]::CreateEntryFromFile($zip, <ws>\$f, $f.Substring('.claude/skills/'.Length)) }
[ZipFileExtensions]::CreateEntryFromFile($zip, <ws>\LICENSE, 'LICENSE')
```

| 版 | ファイル | エントリ | `\` を含む | 最上位 |
|---|---|---|---|---|
| 5.1.26100.9711（Desktop） | 32 | 33 | 0 | LICENSE, pasta-check, pasta-ghost-authoring, pasta-lua-coding |
| 7.6.7（Core） | 32 | 33 | 0 | 同じ |

- 2 つの zip のエントリの一覧（並べ替えた物）は完全に同じ（`diff` で差なし）。
- 5.1 の zip を `unzip` で展開すると、最上位に 3 つのフォルダと `LICENSE` が出て、各フォルダは `.claude/skills/<名前>/` と `diff -r` で差が無く、`LICENSE` もルートと同じバイト列だった。5.1 の `Expand-Archive` でも 33 ファイル・3 つのフォルダに展開された。
- 5.1 では拡張メソッドをインスタンスの形（`$zip.CreateEntryFromFile(...)`）で呼べないので、`[System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile` と静的に呼ぶ。7 でも同じ書き方で動く。
- `git ls-files` の出力は、5.1 でもリポジトリ相対で `/` 区切りのまま得られる（5.1 の .NET Framework には `Path.GetRelativePath` が無いので、フォルダを辿る案ではここを自分で組む必要があった）。
- 追跡ファイルの名前は 32 個すべて ASCII（`grep -P '[^\x20-\x7e]'` で該当なし）。
- 2.1 節の実験（`Compress-Archive`・`CreateFromDirectory` は 5.1 で `\` を書く）と合わせ、**`Compress-Archive` を使わず、`ZipArchive` に自分で組んだ名前で書く**ことに決めた。

### 8.3 そのほかの確かめ

- `node book/tools/gen-skill-refs.mjs --check` は、このワークツリー（`core.autocrlf=true`・CRLF の checkout）で「24 件すべて最新・孤立なし」・終了コード 0・0.1 秒。`checkAll` が LF に正規化して比べるので、windows-latest の build の job でもそのまま使える。npm の依存は無い。
- 今のリリースビルド（`release.ps1` と各 `build.rs`）は git を呼んでいない。ただし手元は開発者のクローン（`RELEASE.md` の手順も git を使う）、CI は `actions/checkout` で、どちらにも git がある。新しく入れる道具ではない（R2.7）。
- `release-scripts-test.ps1` の項目 10 の検査は「出力に `pasta_check.zip` がある」だけなので、6 つにしても通り続ける。「だけが無い」を確かめるには、もう一方の名前が出ないことも見る必要がある。
- `pasta-check` のスキルは `GENERATION_MAP` に無い（生成ファイルを持たない）。照合は対応表のすべてを見るので、本仕様で対象を絞る必要はない。

### 8.4 設計の決定

#### 決定: 新しい段の位置
- **背景**: R2.8 で段の番号と説明を新しい並びに合わせる。番号は `release.ps1`・`release.bat`・`pasta-check/SKILL.md`・sample の README・RELEASE.md に書かれている。
- **候補**: (1) 段 3 の後（段 4〜7 がずれる）、(2) 段 5 の後（版の確認・大きさの 2 段だけがずれる）、(3) リリースの段の先頭（段 3〜7 がすべてずれる）。
- **選んだもの**: (2) 段 5 の後の段 6。
- **理由**: 書き換える番号が最も少ない（`pasta-check/SKILL.md` の段 3・4・5 の記述はそのまま）。どの案でも段 3 以降なので `-SkipSetup` でも走る（R2.5）。段 6 はビルドに依らないので、ビルドより前に置いて早く失敗させる利点は小さい。
- **トレードオフ**: 段 3〜5 が失敗したときは段 6 まで進まず、前の実行の `pasta-skills.zip` が `release\` に残る（今の `pasta.dll.zip` と同じ扱い。R2.4 は段 6 自身の失敗のときの要件）。

#### 決定: zip の書き方
- **候補**: (1) `Compress-Archive`、(2) `ZipFile.CreateFromDirectory`（作業フォルダに写してから）、(3) `ZipArchive`＋`CreateEntryFromFile` で名前を自分で組む、(4) 外部の道具（7-Zip・`tar.exe`）。
- **選んだもの**: (3)。
- **理由**: (1)(2) は 5.1 で `\` を書く（2.1 節の実験）。(4) は新しい道具か、Windows の版に依る道具で、R2.7 に反するか、手元と CI の差の元になる。(3) は 5.1 と 7 で同じ結果になることを 8.2 節で確かめた。

#### 決定: 詰める元の一覧の取り方
- **候補**: (1) フォルダを辿る（`Get-ChildItem -Recurse -File`）、(2) `git ls-files`。
- **選んだもの**: (2)。作る側（`release.ps1`）と CI の検査（`release.yml`）の両方で同じ取り方にする。
- **理由**: 出力が既に `/` 区切りの相対パスで、5.1 で相対パスを組む処理が要らない。「リリースするコミットのファイル」（R1.2・R3.3）に合い、手元の未追跡のファイル（エディタの控えなど）を詰めないので、手元の結果が CI と同じになる。
- **トレードオフ**: git が要る（手元・CI とも既にある）。ASCII でない名前のファイルは引用符付きで出るため、そのファイルが見つからずに段が失敗する（壊れた zip は出ない）。今は該当なし。Revalidation Triggers に挙げた。

#### 決定: 生成ファイルの照合の位置
- **候補**: (1) build の job の「Setup Node」の直後（長いビルドの前）、(2) 「Verify release assets」の直前、(3) verify の job（ギャップ分析の Option C）。
- **選んだもの**: (1)。
- **理由**: 照合はソースだけを見るので、ビルドの前に置けば数十分のビルドを待たずに止まる。build の job の中なので、配布物の検査は build の job という分け方を保つ（(3) を退けた理由）。R3.5 の「公開しない」は、build の job の失敗で後続が needs で止まることで満たす。

#### 決定: 失敗の後片付け
- 段の頭で前の実行の zip を消し（R2.4）、さらに書く途中の例外と検査の失敗でも作りかけの zip を消す。先例の段 3 は検査に落ちた zip を残すが、`pasta-skills.zip` は検査に落ちた物を手元で配る誤りを避けるために消す。

#### 決定: テストの項目
- 項目 10 は置くファイルに `pasta-skills.zip` を足し、「出力に `pasta-skills.zip` が出ない」も見る。項目 11 は別の一時のフォルダで `pasta-skills.zip` だけを欠かし、`pasta_check.zip` が出ないことも見る。2 つの項目を互いに裏返しにして、「だけが無い」を確かめる。

### 8.5 合成（synthesis）

- **一般化**: 3 つの zip を作る共通の関数や、配布物の一覧を 1 か所に持つ表は作らない。要件が求めるのは 1 つの zip の追加で、今の 2 つの zip と同じ「段ごとに書き下す」形を保つほうが、段 3〜5 を変えないこと（R2.9）を確かめやすい。配るスキルの名前だけは段 6 の中で 1 つの配列に持ち、検査と一覧の取得で共用する。
- **作るか採るか**: zip は .NET の標準（`System.IO.Compression`）を採る。PowerShell の `Compress-Archive` は 5.1 の挙動のため退けた。外部の道具は採らない。生成ファイルの照合は既存の `gen-skill-refs.mjs --check` を採る。
- **簡素化**: 別のスクリプト（ギャップ分析の Option B）は作らない。zip の中に版のファイルは足さない（R1.6）。中身のハッシュの検査は足さない（ファイルをそのまま書くので一覧と名前の形の検査で足り、中身の一致はテストの展開と `diff` で確かめる）。手元の `release.ps1` で生成ファイルの照合はしない（Node が要ることになり R2.7 に反する。照合はリリース CI の要件 R3.4 だけ）。

### 8.6 設計ディスカッションへの申し送り

- 上の 5 つの決定（段の位置・zip の書き方・一覧の取り方・照合の位置・失敗の後片付け）は、`design.md` の Decisions / Risks に推奨として書いた。開発者の確認を受ける。
- 設計の検証（kiro-validate-design）の結果は、ディスカッションの議題に入れる。

### 8.7 設計ディスカッションの結論（2026-10-23）

開発者の一括承認で、6 件をすべて推奨案で確定した。上の決定から変えたのは次の 2 つ。

- **CI の検査の期待の一覧**: `git ls-files` から `git ls-tree -r --name-only HEAD` に変えた（検証の指摘 1）。作る側（`release.ps1`）は `git ls-files` のまま。作る側の式を繰り返さず、要件 3.3 の「タグのコミットの一覧」に文字どおり合わせるため。
- **要件 3.4 の言い回し**: 「配布物を検査するとき」を「配布物を作り始める前に」に直した（検証の指摘 2）。照合は「Setup Node」の直後に置き、数十分のビルドの前に止める。

そのほか、zip の中の改行は checkout のまま（Windows では CRLF）とし、手元の zip にコミットしていない変更が入ること・段 3〜5 の失敗で前の zip が残ることは受け入れた（検証の指摘 3。design.md に明記した）。
