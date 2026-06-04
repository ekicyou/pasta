# Implementation Plan

<!-- 本仕様は繰り返し実行型オペレーション仕様です。
     /kiro-impl release-workflow を実行するたびに全タスクが初期化され、
     新たなリリース作業として実行されます。仕様は completed になりません。(Req 9.1-9.3)

     実行モデル: Resource-Aware Staged Concurrency（design.md 参照）
       Stage A (Task 1-3): ローカル・直列（R1 cargo ロック + R2 ワークツリー 排他）
       Stage B (Task 4):   ネットワーク・並行3トラック（X∥Y∥Z、R2 不変）
       Stage C (Task 5):   タグ・プッシュ（Track X 成功が前提 / R2 排他）
       Stage D (Task 6):   GitHub Release（Stage C + Track Z 完了が前提）

     (P) マーカーは「並行実行可能」を示す。R1/R2 を共有する Stage A 内タスクは
     真の並行ができないため (P) を付けない。Stage B の3トラックのみ並行可能。 -->

## Task 1: 事前検証（Stage A / 直列）

- [ ] 1. 事前検証
- [ ] 1.1 GitHub CLI の認証状態を確認する
  - `gh auth status` を実行し ekicyou アカウントで認証済みであることを確認する
  - 未認証の場合: `gh auth login` の実行を案内し認証完了を待つ
  - 完了条件: `gh auth status` が認証済みアカウントを表示する
- [ ] 1.2 リリースバージョン番号を決定する
  - 指定があればそのバージョンを使用する
  - 指定がない場合: Cargo.toml / package.json / git タグ / crates.io / GitHub Releases / VSCode Marketplace の全ソースを調査し最大バージョンを確認する
  - 最大バージョンの PATCH を +1 した値を提案し承認を求める。拒否時は希望バージョンの入力を求め semver 形式（X.Y.Z）を検証する
  - 確定バージョンが全ソースに対して重複していないことを確認する
  - 完了条件: バージョン `vX.Y.Z` が承認され、全ソースで重複がないことが確認された
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7_
- [ ] 1.3 ワークツリーを整理する
  - `git status --porcelain` で未コミット変更の有無を確認する
  - 変更がある場合: `git add -A; git commit -m "chore(release): prepare release vX.Y.Z"`
  - 完了条件: `git status --porcelain` が空出力（ワークツリーがクリーン）
  - _Requirements: 1.8, 1.9_
- [ ] 1.4 全テストを実行してリリース前の健全性を確認する
  - `cargo test --all` を実行する
  - 失敗時: エラー内容を報告してリリース作業を中止する
  - 完了条件: `cargo test --all` がゼロ終了コードで完了する
  - _Requirements: 1.10, 1.11_

## Task 2: バージョン更新とビルド検証（Stage A / 直列）

- [ ] 2. バージョン更新とビルド検証
- [ ] 2.1 Cargo.toml と package.json のバージョンを更新する
  - `[workspace.package].version` を新バージョンへ更新する
  - `[workspace.dependencies]` の5クレート（pasta_core, pasta_dsl, pasta_lua, pasta_shiori, pasta_check）の `version` を更新する（計6箇所）
  - `editors/vscode/package.json` の `version` を同期する
  - 完了条件: Cargo.toml の6箇所と package.json の version すべてに新バージョンが反映されている
  - _Requirements: 2.1, 2.2, 2.3_
- [ ] 2.2 ビルド検証を行いバージョン更新をコミットする
  - `cargo build --workspace` を実行してビルド成功を確認する
  - 失敗時: `git restore Cargo.toml editors/vscode/package.json` でロールバックしエラーを報告する（破壊的 Git 操作は禁止、ファイル単位復元のみ）
  - 成功時: `git add Cargo.toml editors/vscode/package.json; git commit -m "chore(release): bump version to vX.Y.Z"`
  - 完了条件: `cargo build --workspace` が成功し、バージョン更新コミットが git ログに記録されている
  - _Requirements: 2.4, 2.5, 2.6_
  - _Depends: 2.1_

## Task 3: ローカル成果物のビルド（Stage A / 直列・R1+R2 共有のため非並行）

> Task 3.1（ゴースト）と 3.2（VSCode）はともに R1 cargo ロックと R2 ワークツリーを共有するため
> 真の並行実行はできない。両者ともバージョン更新コミット（Task 2）にのみ依存し、crates.io 公開
> （Task 4.1）には依存しない（偽の依存関係の排除 / Req 5.9, 8.6）。実行順序は任意だが直列に行う。

- [ ] 3. ローカル成果物のビルド
- [ ] 3.1 サンプルゴーストをビルドし成果物を確認・コミットする
  - `Push-Location crates/pasta_sample_ghost; PowerShell -ExecutionPolicy Bypass -File release.ps1; Pop-Location` を実行する（ローカルソースから pasta.dll をビルド）
  - 失敗時: エラーを報告してリリース作業を中断する
  - `Test-Path "release/hello-pasta.nar"` と `Test-Path "target/i686-pc-windows-msvc/release/pasta.dll"` で成果物を確認する（いずれか不在なら中断）
  - `Compress-Archive -Path "target/i686-pc-windows-msvc/release/pasta.dll" -DestinationPath "target/i686-pc-windows-msvc/release/pasta.dll.zip" -Force` を実行し `Test-Path` で zip を確認する（失敗時中断）
  - `git add -A; git commit -m "chore(release): build hello-pasta vX.Y.Z"`
  - 完了条件: hello-pasta.nar・pasta.dll・pasta.dll.zip が存在し、ゴーストビルドコミットが git ログに記録されている
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9_
  - _Boundary: Phase 5 GhostBuild（release.ps1 / target/i686-pc-windows-msvc/release/ / release/）_
  - _Depends: 2.2_
- [ ] 3.2 VSCode 拡張をビルドして VSIX を生成する（非クリティカル）
  - `cd editors/vscode; npm install`（失敗時: 警告を記録し継続）
  - `npm run package` を実行する（prepackage の `build:wasm` は R1 cargo ロックを保持するため Stage A で実施）。失敗時: 警告を記録し継続
  - 生成された VSIX パスを `$env:VSIX_PATH` に記録する
  - 完了条件: VSIX（`pasta-vscode-X.Y.Z.vsix`）が生成され `$env:VSIX_PATH` に記録されているか、警告を記録して継続
  - _Requirements: 4.1, 4.2, 4.6_
  - _Boundary: Phase 4a VsixPackage（editors/vscode/）_
  - _Depends: 2.2_

## Task 4: 公開（Stage B / ネットワーク・並行3トラック）

> 前提: Stage A 完了（Task 1-3 / ワークツリークリーン・全成果物生成済み）。Req 8.2。
> 4.1（X）・4.2（Y）・4.3（Z）は R2 を変更せず互いに独立するため並行実行する（Req 8.3）。
> オーケストレーション: Track X の crates.io インデックス待機に Track Y/Z を重ねる。
> バックグラウンド実行可能な環境では Track Y を run_in_background で起動してよい。

- [ ] 4. 公開（並行3トラック）
- [ ] 4.1 (P) crates.io へ依存関係順にクレートを公開する（Track X / クリティカル）
  - 順序固定で公開する: `cargo publish -p pasta_core` → `pasta_dsl` → `pasta_lua` → `pasta_shiori` → `pasta_check`（pasta_check は他 pasta_* に非依存のバイナリ、最後）
  - `pasta_sample_ghost` は `publish = false` のためスキップする
  - 各クレートの公開成功を確認してから次へ進む。各公開後 `Start-Sleep -Seconds 10`（最後のクレートは不要）
  - 失敗時: 段階的バックオフ（1分→2分→…→10分、最大10回）でリトライする
  - リトライ全失敗時: エラーを報告し以降の公開を中断、既公開クレートは残し開発者の指示を待つ。**この場合 Task 5・6 は実行しない（安全順序保証 / Req 8.5）**
  - 完了条件: 5クレートすべてが crates.io に新バージョンで公開された
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_
  - _Boundary: Track X CratesPublish（crates.io）_
  - _Depends: 3.1_ <!-- Stage A 完了ゲート（ワークツリークリーン）。VSIX 生成 3.2 は非クリティカルで crates.io 公開を gate しない -->

- [ ] 4.2 (P) VSIX を VSCode Marketplace に公開する（Track Y / 非クリティカル）
  - `cd editors/vscode; vsce publish` を実行する（Track X と並行可能。R1 不要・R2 不変）
  - 失敗時: 段階的バックオフでリトライする
  - リトライ全失敗時: 警告を記録し継続する（失敗隔離。Task 5・6 をブロックしない / Req 8.4）
  - 成功時: Marketplace URL を記録する
  - 完了条件: `vsce publish` が成功し URL が記録されているか、警告を記録して継続している
  - _Requirements: 4.3, 4.4, 4.5, 4.7_
  - _Boundary: Track Y VsixPublish（VSCode Marketplace）_
  - _Depends: 3.2_
- [ ] 4.3 (P) コミット履歴からチェンジログを生成する（Track Z / 準備・読み取り専用）
  - `git tag -l "v*" --sort=-version:refname` で前回タグを特定する
  - 前回タグがある場合: `git log <前回タグ>..HEAD --oneline --no-merges`、ない場合（初回）: `git log --oneline --no-merges`
  - Conventional Commits 形式で分類・グループ化する（feat/fix/refactor/docs/test/chore）。スコープ `spec` のコミットは除外し、空グループは省略する
  - 整形済みチェンジログを一時ファイル `release-notes-vX.Y.Z.md` に書き出す
  - 完了条件: `release-notes-vX.Y.Z.md` が作成され整形済みチェンジログを含む
  - _Requirements: 7.1, 7.2, 7.3, 7.9_
  - _Boundary: Track Z Changelog（git log 読み取り専用 / release-notes-vX.Y.Z.md）_
  - _Depends: 3.1_

## Task 5: リリースタグとリモートプッシュ（Stage C / R2 排他）

- [ ] 5. リリースタグとリモートプッシュ
- [ ] 5.1 Git アノテーションタグを作成しリモートにプッシュする
  - `git tag -l "vX.Y.Z"` で既存タグの競合を確認する。既存があれば開発者に対応を確認する（自動削除はしない）
  - `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
  - `git push origin main --tags`（main 直接 push、workflow.md 準拠）
  - 失敗時: 段階的バックオフでリトライし、それでも失敗なら手動実行手順を案内する
  - 完了条件: `git tag -l "vX.Y.Z"` でタグが確認でき、`git push` が成功してリモートの main とタグが更新されている
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  - _Depends: 4.1_

## Task 6: GitHub Release 作成と完了報告（Stage D）

- [ ] 6. GitHub Release 作成と完了報告
- [ ] 6.1 GitHub Release を作成しアセットを添付する
  - VSIX が存在する場合（`$env:VSIX_PATH` かつ `Test-Path`）アセットリストに追加する（非ブロッキング: 未生成でも dll.zip + .nar で作成）
  - `gh release create vX.Y.Z "target/i686-pc-windows-msvc/release/pasta.dll.zip" "release/hello-pasta.nar" [<VSIX>] --title "pasta vX.Y.Z" --notes-file release-notes-vX.Y.Z.md`
  - 失敗時: 段階的バックオフでリトライし、それでも失敗なら手動実行手順を案内する
  - `Remove-Item release-notes-vX.Y.Z.md` で一時ファイルを削除する
  - 完了条件: `ekicyou/pasta` にリリースページが作成され、アセット（dll.zip + .nar [+ VSIX]）が添付されている
  - _Requirements: 7.4, 7.5, 7.6, 7.7, 7.8_
  - _Depends: 5.1, 4.3_
- [ ] 6.2 リリース完了サマリーを報告する
  - 以下を含む完了サマリーを開発者に報告する：
    - リリースバージョン: `vX.Y.Z`
    - 公開クレート: pasta_core, pasta_dsl, pasta_lua, pasta_shiori, pasta_check
    - GitHub Release URL: `https://github.com/ekicyou/pasta/releases/tag/vX.Y.Z`
    - VSCode Marketplace（Track Y）: 公開成功 URL または警告メッセージ
    - 各並行トラックの成否（Track X / Y / Z）
  - 完了条件: 上記情報を含む完了報告が開発者に提示されている
  - _Requirements: 9.4_ <!-- 9.1-9.3 はワークフロー全体／spec.json レベルの特性であり個別タスクではトレースしない（design Traceability と整合） -->
  - _Depends: 6.1_
