# Implementation Plan

> 実装の順序は設計の 3 段（第 1 段 manual 側ツールと公開前の関門、第 2 段 `homeurl`・警告・文書、第 3 段 `release.yml` の新ジョブ）に沿う。1 つの PR に入れる（部分出荷はしない）。
> PR は `/kiro-complete` で初めて作るので、PR 上のワークフロー実行はこの実装の完了条件に使わない。ワークフローの確認は、ローカルの `build` の実行と構文検査、および人が push したブランチでの実行（5.2）で行う。
> `.claude/skills/` 配下の編集（6.5）は分類器に止められうる。止まったら入った分と残りを示して許可を求める。
> 実 URL の確認はマージ後にしか行えないため、タスクにせず末尾の「マージ後の確認事項」に置く。

- [x] 1. Foundation: 置き場所の組み立て・照合ツール（GhostSite）
- [x] 1.1 nar（zip）の読み出しと `updates.txt` の解析
  - 標準モジュールだけで、zip の中央ディレクトリを読み、保存・deflate の 2 方式を展開する。ディレクトリのエントリは捨てる
  - 危険な名前（`..`・`\`・先頭 `/`・ドライブ指定・NUL・空の区切り・重複）と非対応の形式（暗号化・zip64・その他の方式）は読み出しの時点で失敗にする
  - `updates.txt` を解析して `(パス, md5, 大きさ)` の一覧にする。`charset` 行と空行は無視し、`date=` は読むが使わず、解釈できない行は失敗にする
  - 自己テストで、テスト内に組んだ zip を使い、正常系・危険な名前・非対応形式・`updates.txt` の CRLF と区切りの扱いを確かめる。完了時は自己テストが通り、マニュアル CI の自己テストの自動実行の枠に入っている
  - _Boundary: GhostSite_
  - _Dispatch: standard — 新規ロジックだが設計が契約と規則を与えている_
  - _Requirements: 3.3, 11.4_

- [x] 1.2 照合と展開（`build` の中核）
  - `updates.txt` の各行を nar のエントリと照合し、`updates.txt` が無い・読めない・行が指すファイルが無い・md5 か大きさが違う、をすべて集めて 1 件 1 行で表示する。`updates.txt` に載らないエントリ（`bal/` 以下・入れ子の `updates.txt` など）は許す
  - 照合に通った一式だけを出力先へ書く（照合の失敗時は何も書かない。空でない出力先は拒否）。`bal/` を含む nar 全体を除外なしで展開する
  - 自己テストで、全行一致・ファイル欠け・md5 違い・大きさ違い・`updates.txt` 無し・入れ子の `updates.txt` が行に無くても通る・複数の問題の同時表示・失敗時に出力先が空のままであることを確かめる。完了時は自己テストが通る
  - _Boundary: GhostSite_
  - _Depends: 1.1_
  - _Dispatch: standard — 設計が規則と契約を与えるが、照合と出力の実装は書き下ろし_
  - _Requirements: 1.2, 1.5, 3.1, 3.2, 3.3, 4.4, 11.4_

- [x] 1.3 入口ページと CLI `build`
  - 入口の静的ページ（案内文とダウンロードのページへのリンクだけ。版や nar の中身に依らない）を用意して出力に足す。nar に入口と同名のファイルがあれば失敗にする。出力の一覧が nar のエントリ＋入口と一致することを事後に確かめる
  - `build --nar <nar> --out <dir> --index <html>` の CLI（終了コード 0/1、問題は標準出力、Actions では注釈も出す）を作る
  - 自己テストで、入口の衝突の失敗・一覧の一致・CLI の終了コードを確かめる。完了時は自己テストが通り、手元の nar を CLI で展開すると nar と同じ一覧＋入口になる
  - _Boundary: GhostSite, 入口ページ_
  - _Depends: 1.2_
  - _Dispatch: standard — 設計が契約を与える_
  - _Requirements: 1.1, 1.2, 7.1, 7.2, 7.3_

- [x] 1.4 実 URL の見比べ（`verify-live`）と CLI
  - 実 URL の `updates.txt` を nar のものとバイト一致で見比べ、一致するまで指定の回数・間隔で取り直す。取得には一意のクエリを付け、HTTP 200 以外とネットワーク失敗は取り直しの対象にする。上限に達したら最後の状態（HTTP 状態・長さ・md5）を表示して失敗にする
  - `--full` で、一覧の全ファイルも実 URL から取って md5・大きさを照合できるようにする（実機確認の 1 回用で、CI の常設にはしない）
  - 自己テストでは、ローカルの HTTP サーバーを使い、初回一致・数回後に一致・上限まで不一致・404・`--full` での 1 ファイル改変の検出を確かめる。完了時は自己テストが通り、CLI の `verify-live` が終了コードで結果を返す
  - _Boundary: GhostSite_
  - _Depends: 1.1_
  - _Dispatch: standard — 取り直しと比較のロジックを書き下ろす_
  - _Requirements: 1.3, 5.1, 5.3, 5.4, 10.4_

- [x] 2. Core: 公開側のワークフロー
- [x] 2.1 マニュアル CI（`manual.yml`）に最新 Release の nar の取得と置き場所の組み立てを足す
  - 旧版マニュアルの足しの後・Pages へのアップロードの前に、最新 Release の `hello-pasta.nar` を取る段と、組み立てる段を足す。取得に失敗（Release・nar が無い・取れない）したら失敗にし、PR・ブランチでも警告にとどめない
  - 起動条件・権限・`deploy` ジョブ・environment は変えない。ヘッダのコメントにパイプラインの 1 行を足す
  - 最新の Release の nar（`homeurl` を足す前の版でもそのまま）を置く。マニュアルだけの再公開でも毎回最新 Release から作り直す
  - 完了時は、ワークフローの構文検査が通り、手元で v0.6.1 の nar（Release の添付。取得は許可を得てから）に `build` を実行して全行が整合して通ること（最初の整合の確認）を `research.md` に記録してある
  - _Boundary: manual.yml_
  - _Depends: 1.3_
  - _Dispatch: standard — 設計に段の位置と中身があるが、既存ワークフローへの組み込みを確かめる_
  - _Requirements: 1.1, 3.4, 3.5, 3.6, 4.2, 4.3, 4.4, 4.6, 4.8, 11.5_

- [x] 2.2 (P) `release.yml` の `build` ジョブに、Release 公開前の nar の関門を足す
  - 作った `hello-pasta.nar` を `build` で一度展開・照合する段を、nar の必須エントリを確かめる既存の段の直後に足す。出力先は捨て、失敗したら `github-release` へ進まない
  - この `build` ジョブは Windows ランナーで動くので、新しい段はジョブの既定のシェルに合わせた書き方（または明示的に bash）にし、ツールのあるフォルダがチェックアウトに含まれていることを確かめる
  - 既存ジョブの権限・outputs・`status`/`reason` の契約は変えない。`release/<名前>` だけの行を書かない（固定リンクの名前の検査を壊さない）
  - 完了時は、手元で壊れた nar（1 バイト変えたもの）が失敗し、正常な nar が通ることを同じコマンドで確かめ、ワークフローの構文検査が通る
  - _Boundary: release.yml build job_
  - _Depends: 1.3_
  - _Dispatch: mechanical — 設計が段の位置と引数を与えている_
  - _Requirements: 3.7, 11.1, 11.2_

- [x] 3. Core: ゴースト側の設定と派生ゴーストの警告
- [x] 3.1 hello-pasta の `homeurl` を置き場所に向ける
  - `descript.txt` の `homeurl` を置き場所（末尾の `/` を含む）にする。これを唯一の正とする
  - `homeurl` を表明している既存の統合テストを新しい値に直す
  - 完了時は、`pasta_check release` で作った nar の `updates.txt` が新しい `descript.txt` の md5・大きさを持ち、既存のテストが通る
  - _Boundary: hello-pasta ghost_
  - _Dispatch: mechanical — 値の差し替えと既存の表明の追従_
  - _Requirements: 2.1, 2.2_

- [x] 3.2 派生ゴーストの `homeurl` の判定（純関数）と検査の種類の追加
  - `descript.txt` の bytes から、`name` が `hello-pasta` でないゴーストの `homeurl` が置き場所を指していれば注意を返す純関数と、置き場所の定数を用意する
  - 読み方: 緩い UTF-8・BOM 除去・キー重複は最初の行・`name` は trim の完全一致（大文字小文字を区別）・`name` 無しは hello-pasta でない扱い。`homeurl` は小文字化・`http(s)://` と末尾 `/` を除いて比べ、配下のパスも指すとみなす（`hello-pasta-foo` は指さない）
  - 検査の結果の種類に `hello-pasta-homeurl` を末尾に足し、重さは注意で固定し、文言を用意する
  - 単体テストで、判定の表（一致・別 URL・大文字小文字・http/https・末尾 `/`・配下・別名・`name` 無し・BOM/CRLF・UTF-8 でない bytes・キー重複）、種類の固定の分類、および定数と `descript.txt` の `homeurl` の一致（3.1 の後なので通る）を確かめる。完了時は単体テストが通る
  - _Boundary: pasta_check validate_
  - _Depends: 3.1_
  - _Dispatch: standard — 判定規則は設計にあるが、純関数と種類の追加を書き下ろす_
  - _Requirements: 8.3, 8.4, 8.5, 8.6_

- [x] 3.3 CLI の殻に警告を組み込む
  - マスターの `descript.txt` を読み、注意があれば検査の結果に足して並べ直す。ファイルが無い・フォルダである・読み取りに失敗したときは警告を出さず、結果も変えない
  - 検査の核・LSP・WASM・pasta-recipe の入力の契約は変えない。`release` が内部で呼ぶ検査にも自然に載ることを確かめる
  - 派生ゴーストのテストで、注意だけのとき `WarningsOnly`（終了コード 0）、読めないとき `Clean` のまま、`release` で注意が出ても nar は作られることを確かめる。完了時は上記テストが通り、既存の検査テストが緑のまま
  - _Boundary: pasta_check CLI shell_
  - _Depends: 3.2_
  - _Dispatch: standard — 既存の殻への組み込みと並べ直しの確認_
  - _Requirements: 8.3, 8.4, 8.7, 8.8_

- [x] 4. Core: リリース CI の新ジョブ
- [x] 4.1 `publish-ghost-site.ps1` の骨格：最新タグの判定・引数・`-DryRun`
  - タグと `releases/latest` を数字 3 つで比べ、古ければ起動せず `skipped`（reason `stale-tag`）、新しければ取り直し（10 秒×6 回）して、それでも新しくならなければ `failed`（reason `transient`）にする
  - `-Tag`・`-DescriptPath`（`homeurl` は判定の後・見比べの直前にだけ読む）・`-LatestTag`・`-DryRun`・`-Ref`（既定 `main`）と待ちの値の引数を持つ。`status`/`reason` は既存の流儀（`GITHUB_OUTPUT` が無ければ標準出力）で出す
  - `release-scripts-test.ps1` に、古いタグ・`homeurl` の無い古いタグ・新しすぎるタグ・同じタグの 4 項目を足す。完了時は 4 項目が通る
  - _Boundary: publish-ghost-site.ps1_
  - _Dispatch: standard — 判定と出力の約束は設計にあるが、既存スクリプトの流儀に合わせて書く_
  - _Requirements: 6.3, 5.5_

- [x] 4.2 起動・run の特定・待ち・公開後の見比べ
  - `-Ref` の `manual.yml` を REST で起動し、応答の `workflow_run_id` を控える。無い・拒否されたときは、起動前の時刻を控えて `gh run list` から起動後の run を特定する（絞り込みのブランチは固定の `main` でなく `-Ref` の値）。特定できなければ `failed`（reason `dispatch`）にする
  - 20 秒おきに run の状態を読み、完了まで待つ（最大 30 分）。`success` と `cancelled` は見比べへ進み、`failure`・`timed_out`・`startup_failure` は `failed`（reason `build`）、待ち切れなければ `failed`（reason `transient`）にする
  - Release の nar を取り（見比べの正本）、`homeurl` を読んで `verify-live` を呼ぶ（30 秒×30 回）。成功で `published`、失敗で `failed`（reason `verify`）。スクリプトの異常終了は `failed`（reason `publish`）
  - ジョブの summary に、タグ・最新タグ・起動した run の URL・見比べの回数を出す
  - 公開済みの Release には触れず、再実行は最初からやり直す冪等な形にする
  - 完了時は、`-DryRun` と各分岐の単体確認（結論の判定表）が通り、実際の起動・待ちは 5.2 のリハーサルで確かめる準備ができている
  - _Boundary: publish-ghost-site.ps1_
  - _Depends: 1.4, 4.1_
  - _Dispatch: standard, effort high — 起動・待ち・取り消しの扱いなど順序と時間の判断を含む_
  - _Requirements: 4.1, 4.7, 5.1, 5.3, 5.4, 6.2_

- [x] 4.3 `release.yml` に新ジョブと `report` の行を足す
  - `publish-ghost-site` ジョブを `github-release` の後に足す。権限は `actions: write` と `contents: read` だけ、`timeout-minutes: 60`、`github-release` が成功したときだけ走る。steps は checkout → Node → スクリプト → 結果の集約（`if: always()`）にする
  - `report` の `needs` に足し、関門（`gateFailed`）には含めない。`Row` に任意のラベル引数を足し、置き場所の行（見出し `GitHub Pages hello-pasta`）を `published`＝今回更新した・`skipped`＝最新ではないタグ・`failed`・`not-run` のラベルで出す。他の行は変えない
  - `release/<名前>` だけの行を書かず、Release の添付の名前・数を変えない
  - 完了時は、`release.yml` の構文検査と、固定リンクの名前の検査が通り、`report` の表に 8 行目が出る形になっている
  - _Boundary: release.yml_
  - _Depends: 2.2, 4.2_
  - _Dispatch: standard — 新ジョブと report の行を既存の契約に合わせて書く_
  - _Requirements: 1.4, 4.1, 4.5, 5.2, 5.5, 5.6, 11.1, 11.2_

- [x] 5. Integration: 実機とリハーサルでの確認
- [x] 5.1 実機の SSP で更新が通ることの確認
  - 新しい nar を作業ツリーで作る（`release.ps1` または `pasta_check release`。LuaJIT のビルドがあるため、環境変数 `NoDefaultCurrentDirectoryInExePath` を外してから実行する）
  - 旧版の nar（v0.6.1。Release の添付なので、取得は許可を得てから。2.1 で取った nar があれば使い回す）を、readonly・別ポートの別の SSP に入れ、`homeurl` をローカルの静的サーバーに向ける。新しい nar を `build` で展開して配信する
  - インストール済みの辞書と `pasta.toml` を手で書き換え、ゴーストを動かしたまま（`pasta.dll` を読み込んだまま）更新確認を行う
  - 確認する: `pasta.dll` を含む一式が新しい nar の中身に差し替わる（md5）、書き換えた辞書・`pasta.toml` が上書きされるか、更新後にゴーストが再起動して応答する、`OnUpdate*` の流れで壊れない
  - 結果を `research.md` に記録する。通らなければこの仕様を完了とせず、原因を直すか差し戻す
  - _Boundary: 実機確認_
  - _Depends: 1.3, 3.1_
  - _Dispatch: standard, effort high — 実機での確認と原因の切り分けを含む_
  - _Requirements: 1.3, 2.3, 10.1, 10.2, 10.3, 10.5_

- [x] 5.2 起動経路のリハーサルと `return_run_details` の確認（人の操作を挟む）
  - 手元の `gh` で、`workflow_dispatch` の REST 応答に `workflow_run_id` が含まれるかを確かめる
  - ブランチをリモートへ置く操作（push）は人が行うので、必要になった時点で止めて許可を求める。拒否されたら、飛ばした範囲を `research.md` に記録して先へ進む
  - ブランチが置かれたら、`-Ref <ブランチ>` で起動・run の特定・待ち・結論の判定までをリハーサルする（`deploy` は `main` 限定なので組み立てまで。見比べは行わない）
  - 応答の形と結果を `research.md` に記録し、設計と食い違えば設計を直す。実リリースでの初回の結果は完了後の確認事項と明記する
  - _Boundary: publish-ghost-site.ps1, manual.yml_
  - _Depends: 2.1, 4.3_
  - _Dispatch: standard — 実環境での確認と記録_
  - _Requirements: 4.1, 4.7, 6.2_

- [x] 6. Core: 手順書・マニュアル・スキルの更新（実機の結果の後）
- [x] 6.1 リリース手順書の更新
  - ジョブ表に新ジョブを足し、`report` の置き場所の行の読み方（`published`・`skipped`＋`stale-tag`・`failed`＋reason ごとの次の手）と、回復（失敗した job の再実行、または `manual.yml` を `main` で手動実行）を書く
  - 「リリースの実行中は `release.yml` を `main` へ入れない」、`latest` の nar が壊れていたときの手順（新しい版を切る）、`failed`・`dispatch` のとき（初回の実リリースを含む）の手順を書く
  - `delete.txt` について、ファイルを消す・改名する版を出すときは別途検討する旨を 1 行書く。`homeurl` を足した版の nar を入れた利用者から更新が効く旨と、リリース前の確認（マージ直後の実 URL の確認が通っていること）を書く
  - 完了時は、手順書の記述が設計のエラー表と食い違わない
  - _Boundary: RELEASE.md_
  - _Depends: 4.3_
  - _Dispatch: mechanical — 書く内容が設計にそのまま与えられている_
  - _Requirements: 2.4, 6.1, 6.4, 6.5, 9.1, 9.6, 10.6_

- [x] 6.2 (P) セットアップ手順の更新
  - `release-ci-setup.md` に、置き場所の更新に必要な権限（ジョブ側の `actions: write`・`contents: read`。Settings の変更は要らない）と、`github-pages` の environment が `main` だけに公開を許すこと（変更なし・確認のみ）を書く
  - 完了時は、環境の表に `github-pages` の行があり、権限の節が足されている
  - _Boundary: release-ci-setup.md_
  - _Depends: 4.3_
  - _Dispatch: mechanical — 記述内容を設計が与えている_
  - _Requirements: 9.3_

- [x] 6.3 マニュアルの更新（ダウンロードのページと「hello-pasta から写す」）
  - ダウンロードのページのサンプルゴーストの節に、ネットワーク更新に対応していることと、手元で書き換えた辞書・`pasta.toml` が上書きされうることを 1〜2 文書く。書き方は 5.1 の実機の結果に合わせる
  - 「hello-pasta から写す」の節に、`descript.txt` ごと写したときは `homeurl` を消すか自分の置き場所に直すこと、残すと更新確認で自分の辞書が上書きされうること、`pasta_check validate` が注意を出すことを短く書く
  - 章立て・見た目は変えない。完了時は、マニュアルのビルドと検査（リンク・コンテンツ）が通る
  - _Boundary: book/src_
  - _Depends: 5.1_
  - _Dispatch: standard — 実機の結果を読み取って文面を決める_
  - _Requirements: 2.4, 8.1, 8.2, 9.4, 9.5_

- [x] 6.4 (P) 検査の種類の表の更新とスキル生成物の再生成
  - デバッグの章の検査の種類の表に `hello-pasta-homeurl` の行を足す。スキル生成のスクリプトがこの表を元にしているかを先に確かめ、元にしているときだけ再生成し、そうでなければ表の追加だけにする（6.3 の 2 ファイルとは独立）
  - 完了時は、表に行があり、関連のドリフト検査が通る（再生成した場合は生成物が差分なし）
  - _Boundary: validate.md（と、対象なら pasta-check skill references）_
  - _Depends: 3.2_
  - _Dispatch: mechanical — 表の 1 行追加と生成スクリプトの実行_
  - _Requirements: 8.5_

- [x] 6.5 `release` スキルの更新
  - 結果の読み取りの対象に新ジョブを足し、結果表に置き場所の行（8 行目）を足し、最終報告に含める。完了の判定は結果表の 8 行がすべて `published` か `skipped`、公開先の状態の照会（定型コマンド A）の 7 つは変えない
  - `.claude/skills/` の編集が分類器に止められたら、入った分と残りを示して許可を求める
  - 完了時は、スキルの手順と結果読み取りスクリプトが新ジョブを扱い、スクリプトの自己テストが通る
  - _Boundary: release skill_
  - _Depends: 4.3_
  - _Dispatch: standard — 既存の判定表への追加で、波及の確認が要る_
  - _Requirements: 9.2_

- [x] 7. Validation: 全体の確認
- [x] 7.1 全体の回帰の確認
  - 次をすべて実行して通す: ツールの自己テスト（マニュアル CI の自己テスト一式）、固定リンクの名前の検査、スキル生成物のドリフト検査、`pasta_check` と hello-pasta のテスト、`release-scripts-test.ps1`。整形と静的検査も緑であること
  - `pasta_check release` の `updates.txt` の作り方と Release の添付の名前・数が変わっていないことを確かめる
  - 完了時は、上記がすべて緑で、対象外の領域（検査の核・LSP・WASM・recipe）に差分がない
  - _Depends: 2.1, 2.2, 3.3, 4.3, 5.1, 5.2, 6.1, 6.2, 6.3, 6.4, 6.5_
  - _Dispatch: standard — 横断の確認と失敗の切り分け_
  - _Requirements: 3.5, 8.8, 11.1, 11.2, 11.3, 11.5_

## マージ後の確認事項（タスクではない・`/kiro-complete` の後に人が行う）

- PR のマージ直後の最初の公開（前の版の nar）の後に、実 URL で `verify-live --full` を 1 回実行し、`.dll`・`.pasta`・`.png` などが変換されずに返ることを `research.md` に記録する（Requirements 10.4）。
- 通らなければ次のリリースを出さず、直す PR を先に入れる（Requirements 10.6）。実リリースでの初回の起動結果（置き場所の行）も記録する。

## Implementation Notes

- 5.1: `--option readonly` の SSP は手動のネットワーク更新もしない。更新の実機確認は SSP 本体を Temp の外へ複製し `--option bootunlock,standalone` で立てる（research.md 11.2）。SSP は手元とサーバーの `updates.txt` を比べ、変わった行のファイルだけを取る。手で書き換えたファイルは新しい版でそのファイルが変わらなければ残る。
- 6.1: RELEASE.md の「リリース前の確認」（10.4 の実 URL の確認が通っていること）は、置き場所を足した後の初回のリリースだけに当てはまる。初回が済んだら畳む（完了後の作業）。
