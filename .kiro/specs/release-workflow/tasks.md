# Implementation Plan

<!-- 本仕様は繰り返し実行型オペレーション仕様です。
     /kiro-impl release-workflow を実行するたびに全タスクが初期化され、
     新たなリリース作業として実行されます。仕様は completed になりません。(Req 9.1-9.3)

     実行モデル: Resource-Aware Staged Concurrency（design.md 参照）
       Stage A (Task 1-3): ローカル・直列（R1 cargo ロック + R2 ワークツリー 排他）
                           ＋ ビルド前に main を非破壊マージで取り込み（自動更新 / Req 10.9）
       Stage B (Task 4):   main 統合（タグ作成ローカル → PR マージコミット方式）＝安全ゲート
       Stage C (Task 5):   公開（crates.io ∥ Marketplace、R2 不変・並行2トラック）
       Stage D (Task 6):   タグ push（公開後）＋ GitHub Release ＋ 完了サマリー
       Task 7:             完遂保証・二段リトライ・Resume・エスカレーション（Stage B-D を統べる横断ポリシー）

     安全順序（Req 8.5）: main 統合 → crates.io 公開 → タグ push → GitHub Release
       （可逆な統合を先・不可逆な公開を後・タグ公開は最後）
     完遂保証（Req 11）: 全ターゲット成功までリリースを「完了」としない。
       失敗しやすい手順は 二段リトライ（短期バックオフ→ScheduleWakeup）で完遂まで粘る。
     (P) は並行実行可能を示す。R1/R2 を共有する Stage A 内タスクは非並行。Stage C の2トラックのみ並行可能。 -->

## Task 1: 事前検証と Resume 検知（Stage A / 直列）

- [ ] 1. 事前検証と Resume 検知
- [ ] 1.1 認証・merge-commit 許可・ワークツリーを確認する
  - `gh auth status` で ekicyou アカウントの認証を確認する（未認証なら `gh auth login` を案内）
  - `gh repo view --json mergeCommitAllowed` が `true` であることを確認する（`false` なら一回限りセットアップ `gh repo edit --enable-merge-commit` の未実施を報告し中止）
  - 現在ブランチが非デフォルトブランチ（ハーネス供給のワークツリー）であることを確認する（`main` 上ならワークツリー上での再実行を促す）
  - 第2段リトライは ScheduleWakeup（同一セッション内待機→再開）で行うため、完遂までセッションを開いておく運用である旨を周知する
  - 完了条件: 認証済み・`mergeCommitAllowed: true`・非デフォルトブランチであることが確認された
  - _Requirements: 8.1, 10.1, 10.3, 11.2, 11.3, 11.4_
- [ ] 1.2 リリースバージョンを決定し Resume を検知する
  - main の現行 Cargo.toml バージョン V を取得し、V が完全公開（全公開クレートが crates.io に存在 かつ タグ `vV` が push 済み かつ GitHub Release が存在）かを確認する
  - **V が完全公開に至っていない場合**: V について Resume Mode へ分岐し、バージョン提案・更新・main 統合（Task 2・4）をスキップして未完了分の続行（Task 5・6）へ進む
  - 完全公開済み（通常の新規リリース）の場合: 指定があればそのバージョン、なければ全ソース（Cargo.toml / package.json / git タグ / crates.io / GitHub Releases / Marketplace）を調査し最大バージョンの PATCH を +1 して提案・承認を求める。拒否時は希望入力を求め semver 形式（X.Y.Z）を検証し、全ソースに重複がないことを確認する
  - 完了条件: バージョン `vX.Y.Z` が確定し、通常リリースか Resume かが判定された
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 9.5_
- [ ] 1.3 ワークツリーを整理する
  - `git status --porcelain` で未コミット変更の有無を確認する
  - 変更があれば `git add -A; git commit -m "chore(release): prepare release vX.Y.Z"`
  - 完了条件: `git status --porcelain` が空出力（ワークツリーがクリーン）
  - _Requirements: 1.8, 1.9, 10.2_
- [ ] 1.4 ブランチ現在性を確保する（ビルド前の自動更新）
  - `git fetch origin {default-branch}` を実行する
  - `origin/{default-branch}` が HEAD の祖先でなければ（main が先行）、`git merge origin/{default-branch}` で非破壊マージにより取り込む（`reset`/`rebase` は使わない）
  - コンフリクト時: `git merge --abort` で復帰し、リリース作業を中止して開発者に解消を求める
  - 完了条件: main が HEAD の祖先（ff 相当）であり、以降のビルド・公開が統合後 main と同一ツリー上で行われることが保証された
  - _Requirements: 10.9_
- [ ] 1.5 全テストを実行してリリース前の健全性を確認する
  - `cargo test --all` を実行する（失敗時はエラーを報告して中止）
  - 完了条件: `cargo test --all` がゼロ終了コードで完了する
  - _Requirements: 1.10, 1.11_

## Task 2: バージョン更新とビルド検証（Stage A / 直列）

> Resume Mode の場合は Task 2 をスキップする（main は既に V へ更新・統合済み）。

- [ ] 2. バージョン更新とビルド検証
- [ ] 2.1 Cargo.toml と package.json のバージョンを更新する
  - `[workspace.package].version` と `[workspace.dependencies]` の5クレート（pasta_core, pasta_dsl, pasta_lua, pasta_shiori, pasta_check）の `version` を新バージョンへ更新する（計6箇所）
  - `editors/vscode/package.json` の `version` を同期する
  - 完了条件: Cargo.toml の6箇所と package.json の version すべてに新バージョンが反映されている
  - _Requirements: 2.1, 2.2, 2.3_
- [ ] 2.2 ビルド検証を行いバージョン更新をコミットする
  - `cargo build --workspace` でビルド成功を確認する
  - 失敗時: `git restore Cargo.toml editors/vscode/package.json` でファイル単位ロールバックしエラーを報告する（破壊的 Git 操作は禁止）
  - 成功時: `git add Cargo.toml editors/vscode/package.json; git commit -m "chore(release): bump version to vX.Y.Z"`
  - 完了条件: `cargo build --workspace` が成功し、バージョン更新コミットが git ログに記録されている
  - _Requirements: 2.4, 2.5, 2.6_
  - _Depends: 2.1_

## Task 3: ローカル成果物のビルド（Stage A / 直列・R1+R2 共有のため非並行）

> Resume Mode かつ新セッションの場合は、Release 添付用に成果物のみ再生成する（バージョンは V で確定済み・再 bump/統合はしない）。
> Task 3.1（ゴースト）と 3.2（VSCode）はともに R1 cargo ロックを共有するため真の並行はできない。両者ともバージョン更新コミット（Task 2）にのみ依存し crates.io 公開には依存しない（偽の依存関係の排除 / Req 5.9, 8.6）。

- [ ] 3. ローカル成果物のビルド
- [ ] 3.1 サンプルゴーストをビルドし成果物を確認・コミットする
  - `Push-Location crates/pasta_sample_ghost; PowerShell -ExecutionPolicy Bypass -File release.ps1; Pop-Location`（ローカルソースから pasta.dll をビルド）。失敗時はエラーを報告し中断
  - `Test-Path "release/hello-pasta.nar"` と `Test-Path "target/i686-pc-windows-msvc/release/pasta.dll"` を確認する（いずれか不在なら中断）
  - `Compress-Archive -Path "target/i686-pc-windows-msvc/release/pasta.dll" -DestinationPath "target/i686-pc-windows-msvc/release/pasta.dll.zip" -Force` → `Test-Path` で確認（失敗時中断）
  - `git add -A; git commit -m "chore(release): build hello-pasta vX.Y.Z"`（このコミットが Stage A HEAD＝タグ対象）
  - 完了条件: hello-pasta.nar・pasta.dll・pasta.dll.zip が存在し、ゴーストビルドコミットが git ログに記録されている
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9, 8.6_
  - _Boundary: Phase 5 GhostBuild（release.ps1 / target/i686-pc-windows-msvc/release/ / release/）_
  - _Depends: 2.2_
- [ ] 3.2 VSCode 拡張をビルドして VSIX を生成する
  - `cd editors/vscode; npm install` → `npm run package` を実行する（prepackage の `build:wasm` は R1 を保持するため Stage A で実施）
  - 生成された VSIX パスを `$env:VSIX_PATH` に記録する
  - 失敗時: 一時障害なら Task 7 のリトライへ、非一時障害（ビルドエラー等）なら未完了として原因を報告する。**いずれもリリースを完了済みとしない**（VSIX 未生成のまま完了しない / Req 11）
  - 完了条件: VSIX（`pasta-vscode-X.Y.Z.vsix`）が生成され `$env:VSIX_PATH` に記録されている
  - _Requirements: 4.1, 4.2, 4.6_
  - _Boundary: Phase 4a VsixPackage（editors/vscode/）_
  - _Depends: 2.2_
- [ ] 3.3 コミット履歴からチェンジログを整形する（読み取り専用・先行生成）
  - `git tag -l "v*" --sort=-version:refname` で前回タグを特定する
  - 前回タグがあれば `git log <前回タグ>..HEAD --oneline --no-merges`、なければ（初回）`git log --oneline --no-merges`
  - Conventional Commits で分類・グループ化する（feat/fix/refactor/docs/test/chore）。スコープ `spec` のコミットは除外し、空グループは省略する
  - 整形済みチェンジログを一時ファイル `release-notes-vX.Y.Z.md` に書き出す（compare URL は `<前回タグ>...vX.Y.Z`）
  - 完了条件: `release-notes-vX.Y.Z.md` が作成され整形済みチェンジログを含む
  - _Requirements: 7.1, 7.2, 7.3, 7.9_
  - _Boundary: Phase Z Changelog（git log 読み取り専用 / release-notes-vX.Y.Z.md）_
  - _Depends: 3.1_

## Task 4: main 統合（Stage B / 安全ゲート）

> Resume Mode の場合は Task 4 をスキップする（統合済み）。Stage A 完了（ワークツリークリーン・全成果物生成済み）が前提（Req 8.2）。

- [ ] 4. main 統合（タグ作成 → PR マージコミット）
- [ ] 4.1 最終 ff 検証とアノテーションタグ作成（ローカル）
  - `git fetch origin {default-branch}` で `origin/{default-branch}` が HEAD の祖先であることを再確認する。Task 1.4 後に main が再度先行した稀ケースはリビルドループ回避のため中止し再実行を促す
  - `git tag -l "vX.Y.Z"` で既存タグ競合を確認する（あれば開発者に対応確認。自動削除はしない）
  - `git tag -a vX.Y.Z -m "Release vX.Y.Z"`（作業ブランチ HEAD＝Phase 5 コミットを指す。**push はしない／Stage D で push**）
  - 完了条件: ff 相当が再確認され、ローカルタグ `vX.Y.Z` が作成されている（リモート未反映）
  - _Requirements: 6.1, 6.2, 6.3, 10.6, 10.9_
  - _Boundary: Phase 6 Integrate（git ローカル）_
  - _Depends: 3.1, 3.3_
- [ ] 4.2 PR を作成しマージコミット方式で main へ統合する
  - `gh pr create --base {default-branch} --head <作業ブランチ> --title "release: vX.Y.Z" --body <merge-base..HEAD 履歴＋意図の要約>`
  - `gh pr merge --merge --delete-branch`（**`--squash`/`--rebase` を使わず**コミット SHA を保持。マージ成否は API 結果のみで判定し、`--delete-branch` のローカル削除警告は非致命として無視）
  - 失敗時: 一時障害なら Task 7 のリトライへ、非一時障害（コンフリクト・mergeable でない・権限不足等）なら **force push・履歴書き換え・マージ成功前のブランチ削除を行わず**中断して開発者に解消を求める。**統合成功まで Stage C/D（公開・タグ push・Release）を実行しない**（安全ゲート / Req 8.5, 10.6, 10.7）
  - 完了条件: PR がマージコミット方式で main へマージされ、リリースコミットが SHA 保持のまま main から到達可能になっている
  - _Requirements: 6.4, 8.5, 10.2, 10.3, 10.4, 10.6, 10.7_
  - _Boundary: Phase 6 Integrate（gh pr / GitHub remote）_
  - _Depends: 4.1_

## Task 5: 公開（Stage C / ネットワーク・並行2トラック）

> 前提: Task 4 統合成功（Req 8.2, 10.6）。公開はローカル作業ブランチ（＝統合後 main と同一ツリー）から行うため公開内容は main・タグと一致する（Req 10.9）。
> 5.1（X）と 5.2（Y）は R2 を変更せず独立するため並行実行する（Req 8.3）。各通信は Task 7 の二段リトライ（短期バックオフ→ScheduleWakeup）で完遂まで粘る。

- [ ] 5. 公開（並行2トラック）
- [ ] 5.1 (P) crates.io へ依存関係順にクレートを公開する（Track X / クリティカル）
  - 順序固定: `cargo publish -p pasta_core` → `pasta_dsl` → `pasta_lua` → `pasta_shiori` → `pasta_check`（pasta_check は最後）。`pasta_sample_ghost`（`publish = false`）はスキップ
  - 各公開前に crates.io index（`https://crates.io/api/v1/crates/<crate>/<version>`）で公開済みか確認し、済みならスキップ（冪等 / Resume 対応）。`cargo publish` が「already exists」で失敗した場合も成功扱い
  - 各公開後 `Start-Sleep -Seconds 10`（最後は不要）。成功を確認してから次へ
  - 失敗時: Task 7 の二段リトライ。未公開分のみ再試行し、既公開クレートは残す。main 統合状態は保持する（Req 10.8）。**全クレート公開成功まで Stage D を実行しない**
  - 完了条件: 5クレートすべてが crates.io に新バージョンで存在する
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 8.2, 8.3, 10.8_
  - _Boundary: Track X CratesPublish（crates.io）_
  - _Depends: 4.2_
- [ ] 5.2 (P) VSIX を VSCode Marketplace に公開する（Track Y / 隔離・完遂必須）
  - `vsce show <publisher>.<extension> --json` の versions に当該バージョンがあればスキップ（冪等）。なければ `cd editors/vscode; vsce publish`
  - 他トラックをブロックしない（失敗隔離 / Req 8.4）が、**未公開のまま完了としない**（完遂必須 / Req 11）
  - 失敗時: 一時障害なら Task 7 の二段リトライで完遂まで、非一時障害（`VSCE_PAT` 無効等）なら未完了報告
  - 成功時: Marketplace URL を記録する
  - 完了条件: Marketplace に当該バージョンが公開され URL が記録されている
  - _Requirements: 4.3, 4.4, 4.5, 4.7, 8.3, 8.4_
  - _Boundary: Track Y VsixPublish（VSCode Marketplace）_
  - _Depends: 4.2_

## Task 6: タグ push・GitHub Release・完了サマリー（Stage D）

> 前提: Task 5.1（crates.io 全公開）成功。タグ push は公開後に行い、リモートのタグが常に crates.io 公開済みを含意する（議題3）。

- [ ] 6. タグ push・GitHub Release・完了サマリー
- [ ] 6.1 タグをリモートに push する（公開後）
  - `git ls-remote --tags origin vX.Y.Z` で未 push を確認し、未 push なら `git push origin vX.Y.Z`（Task 4.1 で作成済みのローカルタグ ref。Task 4.2 の `--merge` で対象コミットは main から到達可能）
  - 失敗時: Task 7 の二段リトライ。それでも失敗なら手動再実行手順を案内する
  - 完了条件: リモートにタグ `vX.Y.Z` が存在し、main 履歴から到達可能（`git describe` が解決）
  - _Requirements: 6.4, 6.5, 10.5_
  - _Boundary: Phase 7 TagPush（git / GitHub remote）_
  - _Depends: 5.1_
- [ ] 6.2 GitHub Release を作成しアセットを添付する
  - `gh release view vX.Y.Z` で既存を確認する（あればアセット添付の不足のみ補完／冪等）
  - `$env:VSIX_PATH` かつ `Test-Path` なら VSIX をアセットへ追加する（非ブロッキング: 未生成でも dll.zip + .nar で作成し、後刻 Resume で添付補完）
  - `gh release create vX.Y.Z "target/i686-pc-windows-msvc/release/pasta.dll.zip" "release/hello-pasta.nar" [<VSIX>] --title "pasta vX.Y.Z" --notes-file release-notes-vX.Y.Z.md`
  - 失敗時: Task 7 の二段リトライ。成功後 `Remove-Item release-notes-vX.Y.Z.md`
  - 完了条件: `ekicyou/pasta` にリリースページが作成され、アセット（dll.zip + .nar [+ VSIX]）が添付されている
  - _Requirements: 7.4, 7.5, 7.6, 7.7, 7.8_
  - _Boundary: Phase 7 Release（gh CLI）_
  - _Depends: 6.1, 3.3_
- [ ] 6.3 完了判定と完了サマリーを報告する
  - 全ターゲット（crates.io 全クレート・Marketplace・タグ push・GitHub Release）成功を確認する
  - **全完遂時のみ「完了」**として報告する: バージョン `vX.Y.Z`／公開クレート／GitHub Release URL（`https://github.com/ekicyou/pasta/releases/tag/vX.Y.Z`）／Marketplace 結果／各トラック成否
  - 未完了が残る場合は「未完了（再試行待ち）」として残作業・障害分類・次回 ScheduleWakeup 予定を報告する（完了済みと報告しない）
  - 完了条件: 完遂状況に応じた報告（完了 or 未完了）が開発者へ提示されている
  - _Requirements: 9.4, 11.1, 11.5_
  - _Depends: 6.2_

## Task 7: 完遂保証・二段リトライ・Resume オーケストレーション（横断ポリシー）

> Task 4-6 の各外部通信に適用される横断ポリシー。Stage 実行と並行して常時適用し、全ターゲット完遂まで「完了」としない（no half-done）。

- [ ] 7. 完遂保証・リトライ・エスカレーション
- [ ] 7.1 二段リトライを各外部通信に適用する
  - 第1段: 短期バックオフ（1→10分、計約55分）。待機は ScheduleWakeup を基本とし、ごく短い待機（〜1分）のみ Start-Sleep。前景の長時間 sleep は使わない
  - 第2段: 第1段で未完了が残れば ScheduleWakeup（既定 30〜60 分間隔）で同一セッションが待機→再開し、未完了分のみ冪等に続行する。回数・累計時間に固定上限を設けない
  - 適用対象: PR マージ（4.2）・crates.io（5.1）・Marketplace（5.2）・タグ push（6.1）・Release（6.2）
  - 完了条件: 一時障害の各通信が完遂まで自動再試行され、ScheduleWakeup の待機→再開が実際に機能する
  - _Requirements: 8.1, 11.2, 11.3, 11.4, 11.7_
- [ ] 7.2 一時/非一時障害を判別し非一時を即時報告する
  - ビジー/レート制限/タイムアウト/5xx 等は一時障害として第2段リトライへ回す
  - 認証無効・権限不足・ビルドエラー・マージコンフリクト等は非一時障害としてリトライに載せず、原因と必要対応を即時報告し、開発者対応後に Resume で完遂できる状態を保つ
  - 完了条件: 障害が一時/非一時に分類され、非一時は即時に「未完了・要対応」として通知される
  - _Requirements: 11.6_
- [ ] 7.3 定期エスカレーション通知と試行履歴を出す
  - 待機ループ中、5 回ごと または 24 時間経過ごとに開発者へプッシュ通知する（未完遂・継続中・累計試行回数・最終エラー・分類）
  - 試行履歴（累計回数・初回/最終試行時刻・各ターゲット状態）をセッション内で保持する
  - 完了条件: 長時間の待機中も定期通知が発火し、「完遂待ち」と「実質詰み」を開発者が判別できる
  - _Requirements: 11.8_
- [ ] 7.4 完遂保証と Resume 継続を統括する
  - 完遂判定は crates.io / Marketplace / Release / タグの実状態を都度確認して行う（タスク状態に依存しない / 9.3 と両立）
  - 各並行トラックの完了・失敗を個別に検証しサマリーへ反映する（Req 8.7）
  - セッションが完遂前に終了した場合は、次回の手動 `/kiro-impl` 再実行が Task 1.2 の Resume 検知で未完了分から続行する（自律継続の実寿命は ScheduleWakeup ループ／セッション寿命に律速。無人完遂は Non-Goal）
  - 本仕様は `/kiro-impl` 実行のたびにタスク状態を初期化し、`completed` に遷移せず、各実行を前回非依存の独立作業として動作させる
  - 完了条件: 全ターゲット完遂で「完了」が確定し、未完了時はセッション継続またはセッション終了後の手動 Resume で必ず完遂へ収束する
  - _Requirements: 8.7, 9.1, 9.2, 9.3, 9.5, 11.1_
