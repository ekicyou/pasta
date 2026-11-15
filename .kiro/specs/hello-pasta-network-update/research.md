# ギャップ分析: hello-pasta-network-update

> 実施: 2026-11-15（main `8ad60680` ＋ spec 起票コミット `f6013c40` の上）。入力は `requirements.md`（未承認のたたき台）・`brief.md`・steering。コードは変更していない。選択ではなく、選択肢と材料を出す（Information over Decisions）。

## 1. 分析の要約

- **既にあるもの**: (a) `pasta_check release` 段 5 が nar の `updates.txt`（ルートと `ghost/master/` の 2 か所）を作る仕組み。(b) `manual.yml` の「新版の検査の後に `book/book/classic` を足し、1 つのアーティファクトで公開する」形（置き場所を足す手本）。(c) `release.yml` の各ジョブが `status`/`reason` を outputs に書き、`report` が 1 枚の表にする約束。(d) `release` スキルの結果読み取り（`ci-result.ps1`）と判定表。
- **無いもの（大半は新規）**: nar を取って展開し md5・大きさを `updates.txt` と照らす処理、`release.yml` から `manual.yml` を起動して待つジョブ、公開後の実 URL の見比べ、`report` の 8 行目、`pasta_check validate` の `descript.txt` 読み（今の核は辞書と `pasta.toml`・`calendar.csv` しか中身を読まない）、入口の `index.html`、マニュアルの注意の 1 節。
- **有力な形**: 展開と整合の検査は 1 本の自己テスト付きスクリプトにして `manual.yml` から呼ぶ。公開を起こす側は `release.yml` の新しい 1 ジョブが `workflow_dispatch`（`actions: write`）で `main` の `manual.yml` を起動し、`actions: read` で待つ。公開後の見比べも同じスクリプトを URL 向けに使い回せる。
- **最大の不確実性**: (1) SSP が動いている `pasta.dll` を更新で差し替えられるか（実機でしか分からない・完了条件）。(2) GitHub Pages の配信キャッシュ（`Cache-Control: max-age=600` を実測）が、公開後の見比べの待ちの長さを決める。(3) `manual.yml` の `concurrency`（待ちの run は 1 つしか残らず、新しいものが古い待ちを取り消す）が、起動した run を `cancelled` にしうる。
- **規模**: 見積りは全体で **M〜L（約 1〜2 週間の規模）**、リスクは **Medium**（ワークフロー間の起動・待ちと SSP の実機が未確認。コードの量は小さい）。

## 2. 現状の調査

### 2.1 置き場所の候補となる既存資産

| 資産 | 事実（確認した箇所） | この spec への含意 |
|------|---------------------|-------------------|
| `.github/workflows/manual.yml` | `build` ジョブ（`contents: read`）が検査をすべて通してから `classic/` を `book/book/classic` に足し、`upload-pages-artifact`（`./book/book`・`include-hidden-files: true`）。`deploy` ジョブだけが `pages: write`・`id-token: write`・environment `github-pages`。`concurrency: group pages`（PR は別グループ）・`cancel-in-progress: false`。`workflow_dispatch` あり。公開は `github.ref == 'refs/heads/main'` のときだけ（ブランチ・PR では検査まで）。 | 置き場所は `classic/` と同じ位置（検査の後・アップロードの前）に足せる。Req 3.4・3.5・3.6 は既存の形にそのまま乗る。 |
| `manual.yml` の起動条件 | `push`・`pull_request` の `paths` に `release.yml`・hello-pasta の `dic/**`・`calendar.csv` などはあるが、`descript.txt` は無い。 | `homeurl` の変更だけでは起動しない（`release.yml`・`book/**` の変更で動く）。置き場所の中身は Release 由来なので、起動条件に足す必要は薄い（要判断）。 |
| `book/tools/*.mjs` | 依存は jsdom・vscode-textmate・vscode-oniguruma のみ。`*-test.mjs` は `manual.yml` の「Book tools self-tests」が `find` で自動実行する。 | `book/tools/` に置いたスクリプトの自己テストは CI の配線なしで走る。zip 展開のライブラリは無い（`unzip` コマンドか `pwsh` の `Expand-Archive` を使う）。 |
| `.github/workflows/release.yml` | ジョブ: verify → gate → build → publish-crates・publish-vsce → github-release → report。`github-release` だけが `contents: write`。各公開ジョブが `status`/`reason` を outputs と summary に書く。`report` は `needs` の outputs から 1 枚の表を作り、`Row` 関数が「outputs が空なら skipped のときだけ not-run、他は推論しない」。 | 新ジョブ（例 `publish-ghost-site`）を `github-release` の後に足し、`report` の `needs` と表に 1 行足す。status の語彙（published/skipped/failed/not-run）と `reason` の区分は既存に合わせられる。 |
| `book/tools/link-check.mjs` の `checkReleaseAssetLinks`（行 500〜540） | `release.yml` を行ごとに読み、`${{` を含む行を除いた `^\s*release/(\S+)\s*$` に一致する行を添付名とする。 | 新ジョブに、行が `release/<何か>` だけの行を書かない（例: `path:` の値でなく単独行の `release/...`）。`gh release download` で `-D` を使う書き方なら問題ない。Req 11.1 は守りやすい。 |
| `crates/pasta_check/src/update_files.rs` | 行は `file,<相対パス>\x01<md5>\x01size=<n>\x01date=<日時>\x01\r\n`。1 行目 `charset,UTF-8`。除外は `profile`・`var`・`updates2.dau`・`updates.txt`・`developer_options.txt`（ファイル名一致なので**どの深さの `updates.txt` も**リストに載らない）。`ghost/master/updates.txt` は複製として nar に入るが `updates.txt` の行には載らない。 | 整合の検査は「`updates.txt` の各行 → 展開結果」の 1 方向が基本。nar の全エントリと展開結果の一致（Req 1.2）は別の検査になる。`ghost/master/updates.txt`・ルートの `updates.txt`・`install.txt` の扱い（`install.txt` は載る・`updates.txt` は載らない）を検査の仕様に書く必要がある。`date=` は照合の対象外にできる（md5 と大きさだけで足りる）。 |
| `crates/pasta_sample_ghost/release.ps1` | nar を `release/hello-pasta.nar` に作る（段 5 が `pasta_check release`）。 | 展開の元は GitHub Release の nar（CI の `release-assets` アーティファクトとも同じ中身）。 |
| 実際の公開状況（`gh release view`・`curl -I`） | 最新は v0.6.1。添付 8 つ（`hello-pasta.nar` 3,670,302 バイトなど）。`https://ekicyou.github.io/pasta/` は 200 で `Cache-Control: max-age=600`・`Expires` 付き、`https://ekicyou.github.io/pasta/hello-pasta/` は 404。 | 置き場所は未公開で新規。**配信キャッシュが 10 分**あるため、公開後の見比べの待ちは 10 分を超える見込みで設計する（2.3）。 |
| `ghosts/hello-pasta/` | `ghost/master/descript.txt`（`homeurl,https://github.com/ekicyou/pasta`・`name,hello-pasta`）・`install.txt`（`directory,hello-pasta`）・`shell/master/`。`pasta.dll`・`THIRD_PARTY_LICENSES.txt`・`scripts/` は `.gitignore`（生成物）。`bal/` は含まない。 | 要件 OQ1 は現状どちらでも同じ結果。`crates/pasta_sample_ghost/tests/integration_test.rs` 173 行が `homeurl,https://github.com/ekicyou/pasta` を表明しているので、`homeurl` の変更でこのテストの直しが要る（Req 2.1 に付随する見落としやすい 1 件）。 |

### 2.2 `pasta_check validate` の現状（Req 8 の土台）

- 検査の核 `crates/pasta_check/src/validate/mod.rs` は**ファイルシステムに触れず**、`GhostInput`（`pasta_toml` の状態・`entries`・`dirs`）と `files`（`select_dictionary_files` が返したパスの中身）だけを受ける。`select_dictionary_files` は辞書ファイルと `ghost/master` 直下の `calendar.csv` だけを返す。**`descript.txt` は `entries` にパスはあるが中身は渡されない**。
- 同じ核を使う殻が 4 つある: CLI（`validate_cmd.rs` → `ghost_files::collect`）、LSP（`crates/pasta_lsp/src/transport.rs`）、VS Code の WASM（`editors/vscode/src/wasmBridge.ts`）、pasta-recipe（`editors/recipe/src-tauri/src/check.rs`）。`select_dictionary_files` の返り値を厳密に表明するテストが多数ある（`validate_config_test.rs` など。例 `vec!["dic/a.pasta"]`）。ここに `descript.txt` を足すと、そのテストと `Stats.dictionary_files`（辞書ファイルの個数の表示）の意味が変わる。
- `FindingKind` は宣言の順が報告の並び（`code` の比較）で、`severity()` が種類ごとの固定の分類（Req 8.5 の「固定の重さ」と同じ作り）。注意の側に `RecommendMenuTalk` などが並ぶ。種類を足す箇所は `report.rs` の enum・`severity()`・文言（`msg`）で、`book/src/debug/validate.md` が種類の一覧を持つ（`validate.md` はスキル生成物の元になるため、足せば `gen-skill-refs.mjs` の再生成が要る）。
- 殻側で既に `descript.txt` の `key,value` を読む実装が 1 つある: `crates/pasta_check/src/balloon.rs` の `read_utf8_kv`（`plan_bundled_balloons`・同梱バルーンの `homeurl` の警告）。UTF-8 だけを前提にし、`charset` が UTF-8 でないものは別扱い（同ファイル 949 行付近のテスト）。
- 終了コード: `ValidateOutcome::{Clean, WarningsOnly, HasErrors}`。注意だけなら 0（Req 8.7 は既存で満たされる）。
- `name` の無い・`charset` が UTF-8 でない `descript.txt` の扱いは未定義。今の `balloon.rs` の流儀は「UTF-8 を要求してエラー」だが、`validate` が派生ゴーストの `descript.txt` を読めずに誤りを出すのは Req 8 の趣旨（注意を足すだけ）と合わない。読めないときは黙って警告しないのが筋（要判断）。

### 2.3 リリース CI・マニュアル CI のつなぎ（Req 4・5・6）

- **`GITHUB_TOKEN` で別のワークフローを起動できるか**: 公式文書は「`GITHUB_TOKEN` が起こしたイベントは新しい run を作らない。ただし `workflow_dispatch` と `repository_dispatch` は常に run を作る」（docs.github.com「GITHUB_TOKEN」）。したがって `release.yml` から `main` の `manual.yml` を `workflow_dispatch` で起動できる。必要な権限は、起動が `actions: write`、run の一覧・待ちが `actions: read`（既存の `github-release` の `contents: write` には足さず、新ジョブにだけ置く。Req 4.5 を満たす形）。**実機のワークフローで確かめていない**ので、実装の早い段で `workflow_dispatch` の小さな試験（別ブランチのテスト用ワークフローではなく、この仕様の PR で `manual.yml` を `workflow_dispatch` した確認）を入れる。
- **起動した run の特定**: `gh` 2.87.0 以降、`gh workflow run` は作った run の URL を出す（API の `return_run_details`・2026-02-19 の changelog）。このリポジトリの手元の `gh` は 2.104.0。Actions のランナーの `gh` の版は未確認（Research Needed）。古ければ「起動前の最新 run の ID を控え、起動後に `createdAt` と `event=workflow_dispatch` と `headBranch=main` で探す」形に落とす。
- **待ち**: `gh run watch <id> --exit-status` か、`gh run view --json status,conclusion` のポーリング。マニュアル CI は mdBook ビルド・旧版の生成・検査を含むので数分かかる。新ジョブの `timeout-minutes` を、起動・待ち・公開後の見比べの合計より長くとる。`release.yml` の他ジョブは 10〜90 分の範囲。
- **`concurrency` の落とし穴**: `manual.yml` は `group: pages`・`cancel-in-progress: false`。GitHub の規則では、実行中の 1 つと待ちの 1 つしか残らず、**新しい待ちが古い待ちを取り消す**。リリース直後に別のマニュアルの変更が `main` へ入ると、起動した run が `cancelled` になりうる。待つ側は `cancelled` を失敗として即決せず、再起動するか、`failed`（reason を区別）として出すかを決める必要がある（設計項目 3）。
- **`environment: github-pages` の制約**: `github-pages` は `main` だけに公開を許す。`release.yml` はタグの ref で動くため、`release.yml` から `manual.yml` を `workflow_call`（再利用ワークフロー）で呼ぶ形にすると、呼ばれた側の environment の ref 判定が呼び出し元（タグ）になる見込みで、環境の許可に弾かれる。要件が「`main` で起動」と書いているのはこのため（確かめたわけではないが、この形は採りにくい）。
- **公開後の見比べの待ち**: `Cache-Control: max-age=600` を実測した（Pages の既存ページ）。デプロイ完了直後でも、新しい中身が CDN から返るまで最大約 10 分かかりうる。URL に一意のクエリ（`?run=<id>`）を付けて取ればキャッシュを避けられる見込みがあるが、**Pages がクエリ付きの取得をキャッシュキーにするか未確認**（Research Needed）。回数・間隔・上限の候補: 30 秒間隔・最大 20 回（約 10 分）＋クエリ付き取得、など。見比べ失敗は `report` の 1 行だけを `failed` にする（Req 5.3・5.5 の仮定）。
- **最新の Release の取得**: 公開リポジトリなので、`gh release download --pattern hello-pasta.nar` か `gh api repos/.../releases/latest` は `contents: read` の既定の `GITHUB_TOKEN`（`GH_TOKEN` に渡す）で足りる見込み。ドラフトは `latest` に出ない。`release.yml` の `github-release.ps1` は「下書き → 添付 → 公開」の順なので、公開前に取りに行くことは無い。**追加の権限は `build` ジョブに要らない見込み**（brief の「足すなら `build` だけ」の前提が不要になる）。
- **古いタグの再実行（Req 6.3）**: `latest` の `tagName` とタグ（`GITHUB_REF_NAME`）を新ジョブの頭で比べれば検出できる。比べる位置は `manual.yml` の中（起動時に `inputs.expected_tag` を渡して照合）か、`release.yml` の新ジョブ（起動前に比べて起動しない）かの二択。後者のほうが「他の run を変えない」点で単純。
- **再実行の経路（Req 6）**: `release.yml` の「Re-run failed jobs」で新ジョブだけが再実行できる（既存ジョブの流儀と同じ）。`manual.yml` の手動実行（`main`）は、`workflow_dispatch` が既にあるので追加の作りは要らない。
- **`manual.yml` の `paths` と release.yml**: `release.yml` の変更でマニュアル CI が起動する（`manual-download-page` の検査のため）。新ジョブの追加もこの起動を起こし、PR では置き場所の組み立て（最新 Release の nar の取得）まで走る。**PR からの取得が通る**こと（ネットワークと `gh` の権限）を確かめる必要がある。

### 2.4 手順書・スキル（Req 6.4・9）

- `crates/pasta_sample_ghost/RELEASE.md`: Step 3 の job 表（7 行）・report の表の読み方・「失敗したときの回復」。置き場所のジョブの行と、`report` の 8 行目の読み方と、置き場所専用の回復（「Re-run failed jobs」か `manual.yml` を `main` で手動実行）を足す。「リリースの実行中は `release.yml` を `main` へ入れない」は `procedure.md` の末尾に既にあるが、`RELEASE.md` には無い（Req 6.4 の指す注意はここに書き足す）。
- `.claude/skills/release/procedure.md`（662 行）: 段 7（結果の読み取り・「7 つの公開先」の判定が約 6 箇所）・「結果表の作り方」（公開先ごとの step の対応表）・定型コマンド B の出力（`ci-result.ps1`、対象 job を 3 つにハードコード: `publish-crates`・`publish-vsce`・`github-release`）・定型コマンド A（`release-state.ps1`、公開先の状態の照会。置き場所の 404/200 を見るか）。完了の判定「公開済み 7/7」は 8 つに増やすか、置き場所を別扱い（失敗しても Release は公開済み）にするかの判断が要る。
- `.github/release-ci-setup.md`: 節 4 の environment 表に `github-pages` が無い（`release`・`release-setup-check` の 2 つ）。Req 9.3 は「必要な権限」と「`github-pages` は `main` だけ」を書くよう求めるので、この手順書に節を足すことになる（権限は新ジョブのワークフロー側であって Settings の変更ではない見込み。`github-pages` の許可ブランチは既に設定済みで、確認だけ）。
- マニュアル: `book/src/download.md`（サンプルゴーストの節に更新の対応を 1〜2 文）、`book/src/getting-started/setup.md`（「hello-pasta から写す」の節。今の手順は `descript.txt` を自分で書き直し、`pasta.dll`・ライセンス・シェルだけ写すので、`homeurl` が自然に混ざる経路は「フォルダごと複製」した場合に限る。注意の置き場所は OQ4 の 2 か所のうち `setup.md` 側が手順に近い）。`book/src/getting-started/` の他の章は触らない（Req 9.5）。

### 2.5 外部の事実（SSP の仕様。ukadoc で確認）

- `homeurl` は「ネットワーク更新する内容が入ったフォルダまでの URL」。その下に `updates2.dau` か `updates.txt`（`.dau` を扱わないサーバー向けの代用）を置く。`homeurl` の値は、ゴースト側では `descript.txt` に書くか、SHIORI の Resource `homeurl` で返す。
- 更新は「サーバーのファイルでユーザーのファイルを**上書き**する（同期ではない）」。消えたファイルは `delete.txt` で消す。→ 手元で書き換えた辞書・`pasta.toml` は md5 が違えば取り直される前提（実機確認 Req 10.3 で記録）。**要件に `delete.txt` の話は無い**。将来、nar からファイルを消した版を出した場合、利用者の手元に古いファイルが残る。hello-pasta の辞書名の変更（`dic/NN-*.pasta` の改名）は `pasta.toml` の読み込みパターンで二重に読まれうる点に注意。この spec の対象外と明記してよいか要判断。
- バイナリの取り扱い: ukadoc は「テキストファイルに広告が入らない・拡張子がアップロードできる」を求める。Pages は静的配信でどちらも問題ないはずだが、Req 10.4 は実 URL で確かめる。`.nojekyll` は mdBook が出す（`include-hidden-files: true` で保持）ので、`_` で始まる名前などが除かれる心配は今の nar の構成では無い。
- SSP が動いている `pasta.dll` を更新で差し替えられるか: **ドキュメントでは確認できなかった**（Research Needed・実機）。他の dll ゴーストは普通に更新できるので、SSP は SHIORI を unload してから差し替えると推測できるが、pasta は `pasta.dll` が静的 CRT でデバッグの待受など常駐の機能を持つ（steering・memory にあるとおり）。実機で更新確認を行い、`OnUpdateBegin`〜`OnUpdateComplete` の流れと再読み込みを見る。
- 更新イベント: `OnUpdateBegin`・`OnUpdateReady`・`OnUpdateComplete`（`none`/`changed`）・md5 不一致の再取得（`OnUpdate.OnMD5CompareFailure`）などが SHIORI に届く。pasta 側で処理が要るか（Out of scope の「更新を知らせる台詞」とは別に、既定の挙動で壊れないか）は実機で見る。

## 3. 要件と資産の対応表（Requirement-to-Asset Map）

凡例: **Missing**＝コード・設定が無い、**Unknown**＝実機・実測が要る、**Constraint**＝既存の制約。

| 要件 | 既存の資産 | ギャップ |
|------|-----------|---------|
| 1 置き場所の公開内容 | `manual.yml` の公開の形・`classic/` の手本・nar の構成 | **Missing**: 最新 Release の nar を取り `book/book/hello-pasta/` へ展開する段。**Unknown**: Pages がバイナリ・拡張子をそのまま返す（Req 10.4）。nar に `ghost/master/updates.txt` が入る点（`updates.txt` に載らないファイル）の扱いを仕様に書く |
| 2 `homeurl` | `descript.txt`（現状はリポジトリのページ）・`pasta_check release` の段 5（md5 は自動で変わる） | **Missing**: 値の変更。**付随**: `integration_test.rs:173` の表明の直し。手順書・マニュアルの「既存の利用者は版が上がるまで効かない」の注意 |
| 3 公開前の整合の検査 | md5 は `update_files.rs` の `calculate_md5`（Rust）と同じ規則。`manual.yml` の失敗で公開中断の形 | **Missing**: 検査の本体（`updates.txt` の解析・md5・大きさ・エラー表示）。**Constraint**: PR・ブランチでも検査まで走り公開しない（既存の `if: github.ref == 'refs/heads/main'` で満たせる）。検査の対象外にする（静的性・検索・コンテンツの検査の**後**に足す。`classic/` と同じ位置） |
| 4 公開の経路と自動化 | `release.yml` の `github-release`・`manual.yml` の `workflow_dispatch` | **Missing**: 起動・待ちのジョブ。**Unknown**: `GITHUB_TOKEN` での起動と `actions: read` での待ちの実機（文書上は可）。**Constraint**: `concurrency: pages` の「待ちは 1 つ」。公開を 1 か所に保つ（`manual.yml` の `deploy` だけ） |
| 5 公開後の検証と報告 | `report` の `Row` 関数・各ジョブの `status`/`reason` の約束 | **Missing**: 見比べのジョブ（または新ジョブの後半）・`report` の 1 行。**Unknown**: CDN の遅れ（実測 `max-age=600`）。**Constraint**: `report` の `needs` に足し、他の行は変えない |
| 6 失敗からの回復 | `RELEASE.md` の「失敗したときの回復」・`github-release.ps1` の再実行の冪等性 | **Missing**: 置き場所の更新の再実行の読み方と冪等性（最新の Release から作り直すので冪等）。古いタグの判定 |
| 7 入口 | なし | **Missing**: `index.html`（`manual.yml` のどこかで置く）。置き場所の `book/` 内の元ファイル（git に入れる元）の位置。マニュアルの検査の対象に入らないこと（Req 7.2 と 3.5） |
| 8 派生ゴースト | `balloon.rs` の `read_utf8_kv`・`FindingKind` の固定の重さ・終了コード | **Missing**: `descript.txt` を読む入口（核は中身を受けない。2.2）・種類・文言・判定（正規化）・テスト。マニュアルの注意の 1 節 |
| 9 手順書・マニュアル | `RELEASE.md`・`procedure.md`・`ci-result.ps1`・`release-ci-setup.md`・`download.md`・`setup.md` | **Missing**: 各ファイルの追記。**Constraint**: `.claude/skills/release/` の編集は分類器が拒否するメモあり（`skills-edit-blocked-self-modification`）。実装タスクの手順に「許可を求めて止まる」を織り込む |
| 10 実機 | SSP を別ポートで試す手順（メモ: `--ghost` 絶対パス・readonly・別ポート・TimeMachine） | **Unknown**: dll の差し替え・手元の書き換えの上書き・実 URL のバイナリ。**Constraint**: 通らなければ完了にしない（Req 10.5） |
| 11 既存の保証 | `link-check.mjs` の添付名の読み取り（行ごとの正規表現）・各 `-test.mjs` | **Constraint**: 新ジョブに `release/<名前>` だけの行を書かない。`release.yml` の変更で `manual.yml` が起動し検証される（既存の起動条件で足りる） |

## 4. 実装方式の選択肢

### 4.1 展開・検査のロジックの置き場所（Req 3・5・7）

| 案 | 内容 | 長所 | 短所 |
|----|------|------|------|
| **A. `manual.yml` に直書き（bash）** | `gh release download` → `unzip` → `md5sum`/`stat` で `updates.txt` の行を回す | 新規ファイルなし | 検査ロジックの自己テストが書けない。公開後の見比べと別実装になる。CRLF・`\x01` の扱いが bash では読みにくい |
| **B. `book/tools/` に node スクリプト＋自己テスト** | 例 `book/tools/ghost-site/build.mjs`・`verify.mjs`・`*-test.mjs`。展開は `unzip`、md5 は `node:crypto`、URL の取得は `fetch`。入口は `book/tools/ghost-site/index.html` から複写 | 既存の `book/tools/*-test.mjs` 自動実行の枠に入る。同じ検査関数をローカルのディレクトリと実 URL の両方に使える（Req 3 と 5 を 1 本で）。ツールの配置がマニュアルの CI の流儀と同じ | `book/tools/` にゴースト配布の話が入る（置き場所の spec の都合。マニュアルと同じサイトに載る点で許容できる）。`release.yml` の新ジョブは pwsh がデフォルトだが node も使える（`actions/setup-node`） |
| **C. `.github/scripts/release/` に pwsh スクリプト＋`release-scripts-test.ps1` に項目** | 他のリリース補助と同じ置き場。`release.yml`（pwsh の流儀）に自然 | 新ジョブ側（`release.yml`）と同じ言語。status/reason の書き方の既存の流儀を再利用できる | `manual.yml` の `build` は bash が既定。そこから `pwsh` を呼ぶことになる（ubuntu ランナーに pwsh は入っているが、`manual.yml` に pwsh を使う箇所は今無い）。`release-scripts-test.ps1` は `manual.yml` の PR で走らない（`build.yml` 側か手動） |

→ 材料: 整合の検査は「公開前（ローカル）」と「公開後（URL）」で同じ規則なので、**1 つのコードを共有する**案（B か C）が単純。B は PR の `manual.yml` 検証に自己テストが自動で乗る。C は `release.yml` との距離が近い。

### 4.2 起動・待ちの方式（Req 4）

| 案 | 内容 | 長所 | 短所・不確実性 |
|----|------|------|--------------|
| **A. 新ジョブが `gh workflow run` ＋ `gh run watch`** | `permissions: actions: write, contents: read`（待ちは `actions: read`）。`--ref main`。run は出力された URL から ID を取る（`gh` ≥ 2.87）か、起動前後の差で探す | 要件の想定どおり。`GITHUB_TOKEN` だけで足りる（文書上）。権限は新ジョブにだけ | 起動した run の特定（古い `gh` ではレース）。`concurrency` の取り消し。ランナーの `gh` の版が未確認 |
| **B. `workflow_call` で `manual.yml` を再利用** | `release.yml` に `uses: ./.github/workflows/manual.yml` | 待ちの実装が要らない（needs で足りる）。run の特定の問題が無い | environment `github-pages`（`main` のみ）が呼び出し元のタグの ref で判定される見込み。`manual.yml` の `deploy` ジョブの `permissions`・`concurrency` が呼び出し元に引きずられる。`manual.yml` の単独の起動条件との両立（`workflow_call` を足す）。公開を 1 か所に保つ制約は満たせるが、**環境の許可の確認が要る**（Research Needed）。要件は「`main` で起動」と書いているので別案扱い |
| **C. PAT または GitHub App のトークンで起動** | | 起動の自由度（取り消しに強い設定など） | 秘密情報を足す（`release-ci-setup.md` の「長期の認証情報をどこにも置かない」方針に反する）。要件の「追加の権限は最小」に反する。採りにくい |

→ 材料: A が要件・方針と整合。B は run の特定の問題を消せるが `github-pages` の環境の許可の実機確認が要る（失敗すると手戻りが大きい）。

### 4.3 派生ゴーストの警告の置き場所（Req 8）

| 案 | 内容 | 長所 | 短所 |
|----|------|------|------|
| **A. 殻（CLI）だけ** | `validate_cmd::check_master` が `descript.txt` を読み、`validate_ghost` の報告に `Finding` を足して `Report::new` で並べ直す | 核・LSP・WASM・recipe・`select_dictionary_files` のテスト群に触れない。変更が小さい。Req 8 は「`pasta_check validate`」だけを指す | LSP・VS Code・pasta-recipe では出ない（エディタで気づけない）。`FindingKind` を足すので核の enum は触る |
| **B. 核に入れる**（`select_dictionary_files` とは別の `select_descript_file` を足し、4 つの殻すべてが渡す） | 核が `descript.txt` の中身も受ける | 全部の殻で警告が出る | 4 つの殻（Rust 3・TS 1）の変更と、`GhostInput`/`FileText` の入力の契約の変更。WASM の型（`pasta_lsp_wasm.d.ts`）とそのテストに波及。要件の範囲を超える |
| **C. `release` 側（`pasta_check release`）で警告** | 作者がリリースビルドするとき | 事故の入口（配布する直前）で気づく | Req 8.3 は `validate` を指す。`release` は `validate` を内部で呼ぶ（`run_on_master`）ので、A の実装は release にも自然に載る |

→ 材料: A が最小で、要件の範囲と一致し、`release` にも波及する。B は将来の拡張の余地。

### 4.4 全体の構成案

- **案 X（推奨の材料）**: 4.1 の B（node スクリプト＋自己テスト、`manual.yml` の `classic/` の隣に段を足す）＋ 4.2 の A（新ジョブが dispatch・待ち・URL の見比べを同じ検査関数で行い、`report` に 1 行）＋ 4.3 の A。新規ファイルは、スクリプト 2〜3 本とその自己テスト・入口の `index.html`・`FindingKind` の 1 種類程度。
- **案 Y**: 4.1 の C（pwsh）で寄せ、リリースの補助スクリプトの流儀に統一。`manual.yml` から pwsh を呼ぶ。
- **案 Z（段階的）**: 先に `manual.yml` 側（展開・検査・入口）と `homeurl`・警告・マニュアルを入れて Pages へ出し（手動実行で確かめる）、次に `release.yml` の自動化と `report` の行を足す。実機確認（Req 10）は第 1 段で行える。「部分的な出荷はしない」方針（メモ）に照らすと、**同じ spec・同じ PR で両段を入れる**前提で、実装の順序の案としてのみ使う。

## 5. 複雑さとリスク

- **工数: M〜L**。新規コードの量は小さい（スクリプト・種類 1 つ・ワークフローの 1 ジョブ）が、(a) ワークフロー間の起動と待ち・(b) `report`・`release` スキルの判定表の更新の波及・(c) 実機の SSP の確認、で手数が多い。
- **リスク: Medium**。
  - 高めの要素: SSP の dll 差し替え（実機・完了条件）。起動した run の取り消しと特定。Pages の CDN の遅れ。
  - 低めの要素: 展開・md5 の検査（既存の `updates.txt` の形が明確）。入口のページ。警告（殻だけなら小さい）。

## 6. Research Needed（設計フェーズへの持ち越し）

1. SSP が動いている `pasta.dll` を更新で差し替えられるか、差し替え後に再読み込みされるか（実機。`ssp-ghost-test-launch` の手順で、ローカルの HTTP サーバー＋別ポートの別 SSP）。
2. 手元で書き換えた辞書・`pasta.toml` が更新で上書きされること（Req 10.3。`updates.txt` の md5 と手元の md5 の突き合わせの挙動）。
3. Pages のバイナリ（`.dll`・`.png`）・`.pasta`・`.csv`・`.toml` の配信が無変換か（Content-Type・圧縮・改行）。実際の URL に最初に載せた時点で確かめる（Req 10.4）。
4. Pages の配信キャッシュ（`max-age=600`）に対し、一意のクエリ付きの取得が新しい中身を返すか。見比べの回数・間隔・上限の実測。
5. Actions のランナーの `gh` の版（`gh workflow run` が run の URL を出すのは 2.87.0 以降）。出さない場合の特定の方法。
6. `GITHUB_TOKEN`（`actions: write`）での `workflow_dispatch` の起動と、`actions: read` での待ちが、このリポジトリで通ること（文書上は可。最初の PR で試す）。リポジトリの Settings の「Workflow permissions」の既定が影響しないか。
7. `manual.yml` の `concurrency: pages` で、起動した run が別の run の待ちに取り消される条件の実際の挙動。
8. 再利用ワークフロー（`workflow_call`）でも `github-pages` の環境の許可が呼び出し元の ref で判定されるか（案 B を捨てる根拠の確認。必須ではない）。
9. v0.6.1 の nar を展開して `updates.txt` の全行が整合すること（`pasta_check release` 自身の検査は `update_files.rs` の単体テストと `release.rs` の統合テストにあるが、**実物の nar を展開した検査は未実施**）。検査スクリプトの最初の試験データにできる。

## 7. 設計判断の項目（要件ディスカッションの材料）

1. **展開・検査のロジックの置き場所と言語**（node＋自己テスト／pwsh／bash 直書き。4.1）。公開前と公開後の検査を同じコードにするか。
2. **起動の方式**（`workflow_dispatch`＋`gh run watch`／`workflow_call`／別のトークン。4.2）と、起動した run の特定の方法（`gh` の版に依存するか）。
3. **起動した `manual.yml` の run が `cancelled` になったときの扱い**（`concurrency` の待ち 1 つ制限）: 失敗にするか・再起動するか・待ちの時間の上限は。
4. **古いタグの判定の位置**（`release.yml` の新ジョブで起動前に判定／`manual.yml` に期待するタグを渡して照合。Req 6.3）と、`report` の表に出す status の語（`skipped` を流用するか・新しい語か）。
5. **公開後の見比べの範囲**: `updates.txt` 1 本だけ（Req 5.1 の文言どおり）か、`updates.txt` の全行を HTTPS で取って md5 まで照らすか（Req 10.4 の無変換の確認を CI に常設できる。dll 3 MB 超の取得が毎回ある）。待ちの回数・間隔・上限（Pages の `max-age=600` を踏まえる）と、クエリ付きの取得を使うか。
6. **新ジョブの構成**: 1 ジョブ（起動・待ち・見比べ）か 2 ジョブか。`timeout-minutes`。`report` の `needs` の追加と、`gateFailed` の判定に含めるか。`Row` の outputs の名前。
7. **`release` スキルの完了の判定**: 「公開済み 7/7」を 8 つに増やすか、置き場所を別枠（失敗しても Release は公開済みで、再実行で直す）にするか。`ci-result.ps1`（job 名の固定）・`release-state.ps1`（公開先の状態の照会に置き場所の 404/200 を足すか）の更新範囲。
8. **入口の `index.html` の元ファイルの置き場所**（git のどこに置くか・どの段で複写するか）と、内容（静的な案内文＋リンク／自動で移す。要件の仮定は静的な案内）。nar に将来 `index.html` が入った場合の衝突の扱い。
9. **整合の検査の仕様の細部**: `ghost/master/updates.txt`（nar にあり、`updates.txt` の行に載らない）・`install.txt`（載る）・ルートの `updates.txt` の扱い。`date=` を照合しないこと。パスの安全性（`..` や絶対パスの行を弾くか。nar は自前の成果物だが、展開先の外へ書かない防御は必要）。nar の zip エントリ名の文字コードとフォルダのエントリの扱い。
10. **`pasta_check validate` の警告の置き場所**（殻だけ／核＋4 つの殻。4.3）。`name` が無い・`charset` が UTF-8 でない・`descript.txt` が無い／読めないときに、誤りでなく黙って警告しないとするか。判定の正規化（`http`/`https`・大文字小文字・末尾の `/`・配下のパス。OQ7）。置き場所の URL の定数の持ち方（`descript.txt`・Rust・ワークフロー・スクリプトの 4 か所に同じ文字列。1 か所から取るか、複数の一致テストで守るか）。`book/src/debug/validate.md` の種類の一覧とスキル生成物の再生成。
11. **`delete.txt` の扱い**: nar から消えたファイルを利用者の手元から消す仕組み（`delete.txt`）は要件に無い。この spec の対象外と明記するか、手順書に「ファイルを消す版を出すときは別途検討」と書くか。
12. **`homeurl` と派生の事故の追加対策**の要否: SHIORI Resource `homeurl` で返す方式の検討はしないでよいか（`descript.txt` で足りる。要件は `descript.txt` 前提）。
13. **PR の `manual.yml` が最新 Release の nar を取る**ことの是非: 取得に失敗した（ネットワーク・Release の一時的な不整合）ときに、マニュアルだけの PR の検証が落ちる。要件は失敗にする（Req 3.4・4.4）が、PR のときだけは取得の失敗を警告にとどめるか。
14. **`manual.yml` の起動条件**に `hello-pasta` の `descript.txt`・`updates` 関連のパスを足すか（置き場所の中身は Release 由来なので足さなくてもよい。足すと `homeurl` を変える PR で組み立ての検査が走る）。
15. **この spec が入る版**（要件 OQ9）。同時に、`pasta-dll-multiarch` と `release.yml`・`release.ps1` を共有するため同じウェーブに置かない（brief の既存の注意）。

## 8. 設計フェーズへの推奨（情報として）

- 優先して確かめる順: (1) `workflow_dispatch` の起動・待ちの小さな試験、(2) v0.6.1 の nar を展開して検査スクリプトの試作、(3) 実機の SSP の dll 差し替え。(3) は完了条件なので、設計の早い段で見通しを立てる。
- 案 X（node＋自己テスト、`workflow_dispatch`、警告は殻だけ）が、要件・既存の流儀・変更量の釣り合いが取れる見込み。ただし 4.2 の A の「起動した run の特定」と、`concurrency` の取り消しの扱いは設計で決める必要がある。
- 触れるファイルの見込み: `.github/workflows/manual.yml`・`release.yml`、新規スクリプトと入口、`crates/pasta_check/src/validate/report.rs`・`validate_cmd.rs`（＋テスト）、`crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/descript.txt`・`tests/integration_test.rs`・`RELEASE.md`、`.claude/skills/release/procedure.md`・`scripts/ci-result.ps1`、`.github/release-ci-setup.md`、`book/src/download.md`・`getting-started/setup.md`・`debug/validate.md`（スキル生成物の再生成を伴う）、`.claude/skills/pasta-check/`（種類の一覧があれば）。

## 9. 設計フェーズの判断（2026-11-15・`/kiro-design`）

> 入力は `requirements.md`（決定事項 10 件を含む）・本書 1〜8・steering・現行の `manual.yml`/`release.yml`/`pasta_check`。探索の種類は **Light（拡張）**。外部の事実は 2 件だけ読み取り専用で確かめた（9.1）。決定は `design.md` が正本で、ここは理由と退けた案の記録。

### 9.1 追加の調査（外部の事実）

| 調べたこと | 結果 | 出典 | 設計への含意 |
|------------|------|------|--------------|
| `workflow_dispatch` の REST が run の ID を返すか | `return_run_details=true` を付けると 200 で返す（付けなければ従来の 204）。`gh` は 2.87.0 以降が `gh workflow run` で run の URL を出す。REST の文書のレスポンス欄には `workflow_run_id`・`run_url`・`html_url` | GitHub changelog 2026-02-19・REST「Create a workflow dispatch event」 | 起動は `gh` の版に依らない REST（`gh api`）を主にし、応答に ID が無いときだけ `gh run list` で起動後の最新 run を探す。文書にパラメーター名が載っていない点は、最初の実リリースで確かめる（Research Needed） |
| ジョブの権限の包含 | `actions: write` は `actions: read` を含む。ジョブの `permissions` に書いた権限以外は `none` になる | GitHub Actions の権限の規則（既知） | 新ジョブは `actions: write` と `contents: read` の 2 つだけで足りる |
| 既存コードの確認 | nar は Rust の `zip` クレートが作り（ファイルのエントリだけ・deflate・`/` 区切り）、`updates.txt` は CRLF・`0x01` 区切り。`validate_cmd::check_master` は `Report` を受けた位置で足せる。`FindingKind` の追加で触るのは `report.rs`・テスト・`validate.md` だけ（`recommend-menu-talk` の足し方を確認）。`validate` の種類名は WASM・LSP の型に列挙されていない | 本リポジトリ | 読み出し側は最小の zip 解釈で足りる。警告は核の入力を触らず殻だけに足せる |

### 9.2 合成（Design Synthesis）

- **一般化**: 公開前の照合（nar のエントリ）と公開後の見比べ（URL）と実機確認の全ファイル確認（10.4）は、すべて「`updates.txt` の行を、ある一覧（nar のエントリ・URL）と照らす」問題の変種。`checkIntegrity`・`verifyLive`（`--full`）を同じ `parseUpdates` の上に作り、実装は 1 ファイルにした。実装の範囲は要件の分だけ（`bal/updates.txt` の照合は足さない）。
- **作るか・採るか**: zip の読み出しは、npm の zip ライブラリ（`yauzl`・`fflate` など）を足さず、約 60 行の自前実装にした。理由: 読む対象は自前の CI が作る deflate の nar だけ、ライブラリを足すと `release.yml` の job で `npm ci` が要る、`unzip` コマンドは手元の Windows の Git Bash に無く自己テストが手元で動かない。md5・inflate・fetch は Node の標準で足りる。Pages への公開・Release の取得・workflow の起動は GitHub の機能をそのまま使う（作らない）。
- **単純化**: (a) `workflow_call` での再利用（案 B）は、`github-pages` の environment の許可が呼び出し元（タグ）の ref で判定される見込みで採らない。(b) `manual.yml` に `expected_tag` の入力を足す案は、手動実行の経路を増やすだけなので採らない（古いタグの判定は release 側で起動の前に行い、見比べが最終の判定者になる）。(c) 警告のために核の入力の契約（`GhostInput`・`FileText`）を広げる案は、4 つの殻に波及するので採らない（純関数＋CLI の殻）。(d) 起動した run の特定のための専用の仕組みは作らない（REST の応答の ID を使い、無ければ 1 度だけ一覧から探す）。

### 9.3 設計判断（`requirements.md` の「設計フェーズへ回す事項」と本書 7 の対応）

| # | 項目 | 決定（推奨） | 退けた案・理由 |
|---|------|--------------|----------------|
| 1 | 展開・検査のロジックの置き場所と言語（7-1） | Node の 1 ファイル `book/tools/ghost-site/ghost-site.mjs`（ライブラリ＋CLI）と自己テスト。公開前と公開後で同じ `parseUpdates` を共有 | bash 直書き（自己テストが書けない・公開後と別実装）、pwsh（`manual.yml` の `build` は bash 既定で pwsh の使用箇所が無く、`release-scripts-test.ps1` は manual の PR で走らない）。node は `book/tools` の自己テストの自動実行に乗る |
| 2 | 起動と待ちの方式・run の特定（7-2） | 新ジョブが REST の `workflow_dispatch`（`ref=main`・`return_run_details=true`）で起動し、`gh run view` を 20 秒おきに最大 30 分。応答に ID が無いときだけ `gh run list` で起動後の最新 run を探す | `workflow_call`（環境の許可が弾く見込み）、PAT・GitHub App（認証情報を足す。方針に反する） |
| 3 | 起動した run が `cancelled` のとき（7-3） | 失敗にせず、実 URL の見比べへ進む（`manual.yml` の `concurrency` が古い待ちを取り消すのは、後続の run が同じ最新 Release から作るため）。見比べが上限まで合わなければ `failed`・`verify`。`failure`・`timed_out` は `failed`・`build` | `cancelled` を即失敗（実際は正しく公開されるのに赤くなる）、再起動（取り消した run との二重起動で連鎖する） |
| 4 | 古いタグの判定の位置・status の語（7-4） | release 側の新ジョブの頭で、`releases/latest` のタグと数字 3 つで比べる（古い→`skipped`・`stale-tag`。新しい→10 秒×6 回まで取り直して `failed`・`transient`）。`skipped` を流用し、理由を消さず、`Row` に専用のラベルを渡して表に出す | `manual.yml` に期待するタグを渡して照合（手動実行の経路が増える・他の run に影響する） |
| 5 | 公開後の見比べの範囲と値（7-5） | `updates.txt` 1 本のバイト一致。30 秒間隔・最大 30 回（15 分）。一意のクエリ付きで取る。全ファイルは `--full` で実機確認 10.4 の 1 回だけ | 全ファイルを毎回（dll 3 MB 超の取得）。待ちは `max-age=600`（10 分）より長く取り、クエリが効かなくても足りるようにした |
| 6 | 新ジョブの構成（7-6） | 1 ジョブ `publish-ghost-site`（起動・待ち・見比べ）。`timeout-minutes: 60`。`report` の `needs` に足すが `gateFailed` には含めない。outputs は `status`・`reason` | 2 ジョブ（起動と見比べ。`needs` と outputs の受け渡しが増えるだけで、再実行の単位としても 1 つで足りる） |
| 7 | `release` スキルの完了の判定（7-7） | 結果表を 8 行にし、条件 1 を「8 つがすべて `published` か `skipped`」にする。定型コマンド A の 7/7 は変えず、置き場所の状態の照会は A に足さない（CI の見比べを正とする）。`ci-result.ps1` の対象 job に足す | 8/8 に増やして A に置き場所の 404/200 を足す（A の変更が増える）、別枠にして失敗でも完了とする（置き場所の失敗が見逃される）。**OPEN QUESTION 1** |
| 8 | `index.html` の元ファイルと内容（7-8） | `book/tools/ghost-site/index.html`（git に入れる静的ファイル）。`buildSite` が出力へ複写。内容は短い案内文とダウンロードのページへのリンク（絶対 URL）だけ。nar に `index.html` があれば失敗 | `book/src/` に置く（mdBook が章にする）、`manual.yml` 内で生成（検査できない） |
| 9 | 照合の細部（7-9） | ルートの `updates.txt` の行だけを照合。`date=` は見ない。載らないエントリは許す（`bal/`・各階層の `updates.txt`・`updates2.dau`・`developer_options.txt`）。パスは `\`・先頭 `/`・`..`・`:`・空の区切り・NUL・重複を拒否。zip のフォルダのエントリは捨てる。zip は暗号化・zip64・方式が 0・8 以外を拒否 | `ghost/master/updates.txt`（複製）の一致確認とバルーンの `updates.txt` の照合は足さない（SSP が使わない・バルーンの持ち主の範囲）。CRC32 は見ない（md5 の照合が上位にある） |
| 10 | 警告の実装（7-10） | 判定は `crates/pasta_check/src/validate/homeurl.rs` の純関数、読むのは `validate_cmd::check_master`（殻）。`FindingKind::HelloPastaHomeurl`（宣言の末尾・注意）。`descript.txt` は先頭の BOM を除いて `from_utf8_lossy` で読み、読めなくても黙る。URL の定数は Rust 1 つ＋`descript.txt`（SSOT）で、`include_str!` の一致テストで守る。release 側の job は `descript.txt` の `homeurl` を読む。`validate.md` に 1 行足しスキル生成物を再生成 | 核に入れる案（4 つの殻に波及）、`balloon.rs` の `read_utf8_kv` の流用（UTF-8 を要求してエラーにするので、派生ゴーストを誤りにしない趣旨と合わない） |
| 11 | `integration_test.rs:173` の直し | `homeurl,https://ekicyou.github.io/pasta/hello-pasta/` に表明を変える | — |
| 12 | `manual.yml` の起動条件 | 変えない。`descript.txt` は足さない（置き場所の中身は Release 由来）。`release.yml`・`book/**`（`book/tools/ghost-site/**` を含む）の変更では動く | `descript.txt` を足す（`homeurl` の変更で組み立てが走るが、中身が変わらず無駄） |
| 13 | `homeurl` の方式 | `descript.txt` のみ（SHIORI の Resource `homeurl` は使わない） | Resource 方式（要件は `descript.txt` 前提で、SSP の更新確認は `descript.txt` の値で足りる） |
| 14 | PR で最新 nar が取れないとき（7-13） | どの実行でも失敗（要件 4.4）。`curl --retry 3 --retry-all-errors` で一時的な障害を吸収 | PR のときだけ警告（要件が失敗を指定） |
| 15 | 版（7-15） | 要件に書かず、ロードマップで決める。`pasta-dll-multiarch` と同じウェーブに置かない | — |

### 9.4 リスクと緩和

| リスク | 緩和 |
|--------|------|
| `GITHUB_TOKEN`（`actions: write`）での起動と `return_run_details` は実リリースで初めて通る | 失敗しても他の公開先は無傷（5.5）。回復は再実行か `manual.yml` の手動実行（6.1）。起動は REST を主・一覧探索を副にして特定の失敗を減らす。組み立て側は PR の `manual.yml` と `gh workflow run --ref <branch>` で事前に確かめる |
| Pages の配信が遅れる・クエリが効かない | 取り直しの上限を 15 分（`max-age=600` より長い）にする |
| `release.yml` の変更が `release/<名前>` だけの行を生み、固定リンクの名前の検査を壊す | 新ジョブにその形の行を書かない。`manual.yml` の `link-check` が PR で検出する |
| 手元の書き換えが上書きされる（利用者の事故） | 実機確認 10.3 の結果でマニュアルの文言を確定する（8.2）。派生ゴーストは警告で守る |
| `.claude/skills/release/` の編集は分類器が拒否することがある | 実装タスクの手順に「入った分と残りを示し、ユーザーに許可を求めて止まる」を織り込む |

### 9.5 追加の Research Needed（実装の早い段で確かめる）

1. `return_run_details=true` の応答に `workflow_run_id` が入ること（REST 文書のパラメーター欄に載っていない）。入らなければ一覧探索の副の経路が主になる。
2. `GITHUB_TOKEN`（ジョブの `permissions: actions: write`）で `manual.yml` を `ref=main` で起動できること（最初の実リリースで確認。それまでは `gh workflow run` の手元実行で組み立て側を確認）。
3. 実機確認 10.1〜10.3（SSP の dll の差し替え・書き換えの上書き・ローカルの `updates.txt` 方式で通ること。`updates2.dau` が無いサーバーで `updates.txt` に落ちること）。
4. 実機確認 10.4（初回の `main` 公開の直後の実 URL で `.dll`・`.pasta`・`.png` が無変換）。
5. v0.6.1 の nar が `updates.txt` の全行と整合すること（PR の `manual.yml` の最初の実行で確認）。

### 9.6 設計ディスカッションへ持ち越す論点（OPEN QUESTIONS）

1. **`release` スキルの完了の判定**（9.3 の 7）: 結果表を 8 行にして条件 1 に含める推奨案でよいか。置き場所の失敗でリリース全体を「未完了」と扱い、再実行の流れに乗せる前提。
2. **実機確認 10.4 の時点**: 公開サイトは `main` からしか公開できないので、実 URL の確認は第 1 段のマージの後になる。推奨案は「マージ直後に `verify-live --full` で確認し、通らなければこの仕様を再び開いて直す（初回の公開は `homeurl` の無い版の nar で、どの利用者にも使われないので無害）」。要件 10.5（通らなければ完了にしない）との整合を、完了の判定（`/kiro-complete` の DoD）に 10.4 の確認を含める形にするか。
3. **`cancelled` の扱い**（9.3 の 3）: 失敗にせず実 URL の見比べに進める推奨案でよいか。
4. **取り直しの値**（9.3 の 5）: 30 秒 × 30 回（15 分）・run の待ち 30 分・最新タグの取り直し 10 秒 × 6 回・ジョブ 60 分でよいか。
5. **警告の読み方**（9.3 の 10）: `descript.txt` を `from_utf8_lossy` で読み、`charset` を見ない・`name` は trim した完全一致（大文字小文字を区別）・キーが重複したら最初の行を採る、でよいか。

## 10. 設計ディスカッションの決定（2026-11-15・全議題を推奨案で確定）

1. **公開前の関門**: `release.yml` の `build` ジョブに、作った nar を `ghost-site.mjs build` で展開・照合する段を足す（既存ジョブへの 1 段の例外。権限は変えない）。理由: 公開済みの Release は差し替えられず、壊れた nar が `latest` になると `manual.yml` が失敗し続けるため（検証の指摘 1）。`RELEASE.md` に回復（新版を切る）を書く。
2. **初回の起動経路のリハーサル**: `publish-ghost-site.ps1` に `-Ref`（既定 `main`）を足し、ブランチでの起動・run の特定・待ち・`conclusion` の判定をリハーサルする。`return_run_details` の応答の形と結果を、実装の中でこの章に追記する。実リリース（本番）の初回の結果は完了後の確認事項（検証の指摘 3）。
3. **実 URL の確認 10.4 の時機**: Pages は `main` 限定なので、マージ直後に行う。マージ直後の公開は前の版の nar で、新しい `homeurl` を持つ利用者がいないので無害。通らなければ次のリリースを出さず修正 PR。10.1〜10.3 はマージ前にローカルで行う。
4. **`release` スキルの完了の判定**: 結果表 8 行がすべて `published` か `skipped`。定型コマンド A は 7 つのまま、置き場所の状態の照会は足さない。
5. **`cancelled` と待ちの値**: `cancelled` は失敗にせず見比べへ進む。見比べ 30 秒×30 回、run の待ち 30 分、最新タグの取り直し 10 秒×6 回、ジョブ上限 60 分。
6. **警告の読み取り**: 緩い UTF-8・`charset` 無視・`name` は trim の完全一致（大文字小文字を区別）・キー重複は最初の行・読めなければ黙る。
7. **適用順**: 3 段（第 1 段 manual 側＋`build` の関門、第 2 段 `homeurl`・警告・文書、第 3 段 `release.yml` の新ジョブ＋`-Ref` リハーサル）を 1 つの PR に入れる。
8. **自明な修正**: 古いタグの再実行で `skipped` になるよう、`homeurl` はスクリプトが古いタグの判定の後・見比べの直前にだけ読む（`-DescriptPath`）。`-DryRun` に「`homeurl` の無い古いタグ」を足す（検証の指摘 2）。

## 11. 実装の記録

### 11.1 実物の nar での最初の照合（2026-11-15・タスク 2.1）

- 対象: v0.6.1 の Release の添付 `hello-pasta.nar`（`latest` の指す版）。3,670,302 バイト・sha256 `824aed71057213250d1a315126d7ae2b9d0d096b9257065af63c9f4ec0739d19`。
- コマンド: `node book/tools/ghost-site/ghost-site.mjs build --nar <手元の nar> --out <スクラッチ>/site21 --index book/tools/ghost-site/index.html`（`manual.yml` の「Build hello-pasta update site」と同じ引数）。
- 結果: 終了コード 0。`updates.txt` の 41 行すべてがエントリと md5・大きさで一致。nar のエントリ 42（ファイル）、出力のファイル 43（＝エントリ＋`index.html`）。
- `manual.yml` の 2 段（Fetch latest hello-pasta.nar・Build hello-pasta update site）は `actionlint` で検査済み（指摘なし）。CI での実際の取得は PR の実行で確かめる。
