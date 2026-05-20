# Implementation Plan

<!-- 本仕様は繰り返し実行型オペレーション仕様です。
     /kiro-impl release-workflow を実行するたびに全タスクが初期化され、
     新たなリリース作業として実行されます。仕様は completed になりません。(Req 8.1-8.3) -->

## Task 1: 事前確認フェーズ

- [ ] 1. 事前確認フェーズ
- [ ] 1.1 GitHub CLI の認証状態を確認する
  - `gh auth status` を実行し、ekicyou アカウントで認証済みであることを確認する
  - 未認証の場合: `gh auth login` の実行を開発者に案内し、認証完了を待つ
  - 完了条件: `gh auth status` が認証済みアカウントを表示する（Phase 0 暗黙的前提条件）
- [ ] 1.2 リリースバージョン番号を決定する
  - 開発者からバージョン指定がある場合はそのバージョンを使用する
  - 指定がない場合: Cargo.toml、package.json、git タグ、crates.io、GitHub Releases、VSCode Marketplace の全ソースを調査し最大バージョンを確認する
  - 最大バージョンの PATCH を +1 した値を提案バージョンとして開発者に承認を求める
  - 拒否された場合は希望バージョンの入力を求め、semver 形式（X.Y.Z）を検証する
  - 確定バージョンが全ソースに対して重複していないことを確認する
  - 完了条件: リリース対象バージョン `vX.Y.Z` が開発者に承認され、全ソースで重複がないことが確認された
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7_
- [ ] 1.3 ワークツリーを整理する
  - `git status --porcelain` を実行し未コミット変更の有無を確認する
  - 変更がある場合: `git add -A && git commit -m "chore(release): prepare release vX.Y.Z"` を実行する
  - 完了条件: `git status --porcelain` が空出力（ワークツリーがクリーン）
  - _Requirements: 1.8, 1.9_
- [ ] 1.4 全テストを実行してリリース前の健全性を確認する
  - `cargo test --all` を実行する
  - テスト失敗時: エラー内容を報告してリリース作業を中止する
  - 完了条件: `cargo test --all` がゼロ終了コードで完了する
  - _Requirements: 1.10, 1.11_

## Task 2: バージョン更新とビルド検証

- [ ] 2. バージョン更新とビルド検証
- [ ] 2.1 Cargo.toml のバージョンを新バージョンに更新する
  - `replace_string_in_file` を使い `[workspace.package].version` を新バージョンに更新する
  - `[workspace.dependencies]` セクションの5クレート（pasta_core, pasta_dsl, pasta_lua, pasta_shiori, pasta_check）の `version` フィールドを更新する
  - 完了条件: Cargo.toml の6箇所すべてに新バージョンが反映されている
  - _Requirements: 2.1, 2.2_
- [ ] 2.2 package.json のバージョンを新バージョンに同期する
  - `replace_string_in_file` を使い `editors/vscode/package.json` の `version` フィールドを新バージョンに更新する
  - 完了条件: `editors/vscode/package.json` の `"version"` が新バージョンを示している
  - _Requirements: 2.3_
- [ ] 2.3 ビルド検証を行い、バージョン更新をコミットする
  - `cargo build --workspace` を実行してビルド成功を確認する
  - ビルド失敗時: `git restore Cargo.toml editors/vscode/package.json` でロールバックしエラーを報告する
  - ビルド成功時: `git add Cargo.toml editors/vscode/package.json && git commit -m "chore(release): bump version to vX.Y.Z"` を実行する
  - 完了条件: `cargo build --workspace` が成功し、バージョン更新コミットが git ログに記録されている
  - _Requirements: 2.4, 2.5, 2.6_

## Task 3: crates.io へのクレート公開

- [ ] 3. crates.io へのクレート公開
- [ ] 3.1 pasta_core を crates.io に公開する
  - `cargo publish -p pasta_core` を実行する
  - 失敗時: 段階的バックオフ（1分→2分→…→10分、最大10回）でリトライする
  - リトライ全失敗時: エラーを報告し以降の公開を中断して開発者の指示を待つ
  - 成功後: `Start-Sleep -Seconds 10` で待機する
  - 完了条件: `cargo publish -p pasta_core` が成功し、crates.io に新バージョンが登録された
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.6_
- [ ] 3.2 pasta_dsl を crates.io に公開する
  - `cargo publish -p pasta_dsl` を実行する
  - 失敗時: 段階的バックオフでリトライする
  - 成功後: `Start-Sleep -Seconds 10` で待機する
  - 完了条件: `cargo publish -p pasta_dsl` が成功する
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.6_
- [ ] 3.3 pasta_lua を crates.io に公開する
  - `cargo publish -p pasta_lua` を実行する
  - 失敗時: 段階的バックオフでリトライする
  - 成功後: `Start-Sleep -Seconds 10` で待機する
  - 完了条件: `cargo publish -p pasta_lua` が成功する
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.6_
- [ ] 3.4 pasta_shiori を crates.io に公開する
  - `cargo publish -p pasta_shiori` を実行する
  - 失敗時: 段階的バックオフでリトライする
  - 成功後: `Start-Sleep -Seconds 10` で待機する
  - 完了条件: `cargo publish -p pasta_shiori` が成功する
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.6_
- [ ] 3.5 pasta_check を crates.io に公開する（最終クレート）
  - `cargo publish -p pasta_check` を実行する（他の pasta_* クレートに依存しないバイナリ、最後に公開）
  - `pasta_sample_ghost` は `publish = false` のためスキップ
  - 失敗時: 段階的バックオフでリトライする
  - 完了条件: `cargo publish -p pasta_check` が成功し、5クレートすべての公開が完了した
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

## Task 4: VSCode 拡張の Marketplace 公開（非クリティカル）

- [ ] 4. VSCode 拡張の Marketplace 公開（非クリティカル）
- [ ] 4.1 VSCode 拡張をビルドしてパッケージングする
  - `cd editors/vscode && npm install` を実行する
  - 失敗時: 警告を記録し Task 5 へ継続する
  - `npm run package` を実行して VSIX ファイルを生成する
  - 失敗時: 警告を記録し Task 5 へ継続する
  - 生成された VSIX ファイルパスを記録する（`$env:VSIX_PATH`）
  - 完了条件: VSIX ファイル（`pasta-vscode-X.Y.Z.vsix`）が生成されているか、警告を記録して次フェーズへ継続
  - _Requirements: 4.1, 4.2, 4.6_
- [ ] 4.2 VSIX を VSCode Marketplace に公開する
  - `vsce publish` を実行する
  - 失敗時: 段階的バックオフ（1分→2分→…→10分、最大10回）でリトライする
  - リトライ全失敗時: 警告を記録し Task 5 へ継続する（非クリティカル）
  - 成功時: Marketplace URL を記録する
  - 完了条件: `vsce publish` が成功して Marketplace URL が記録されているか、警告を記録して次フェーズへ継続
  - _Requirements: 4.3, 4.4, 4.5, 4.7_

## Task 5: サンプルゴーストのビルドと成果物確認

- [ ] 5. サンプルゴーストのビルドと成果物確認
- [ ] 5.1 release.ps1 を実行して成果物を確認する
  - `Push-Location crates/pasta_sample_ghost; PowerShell -ExecutionPolicy Bypass -File release.ps1; Pop-Location` を実行する
  - 失敗時: エラーを報告してリリース作業を中断する
  - `Test-Path "crates/pasta_sample_ghost/hello-pasta.nar"` で .nar ファイルの存在を確認する
  - `Test-Path "target/i686-pc-windows-msvc/release/pasta.dll"` で DLL の存在を確認する
  - いずれかが存在しない場合: エラーを報告してリリース作業を中断する
  - 完了条件: hello-pasta.nar と pasta.dll の両方が存在する
  - _Requirements: 5.1, 5.2, 5.3, 5.4_
- [ ] 5.2 DLL を zip 圧縮してビルドコミットを作成する
  - `Compress-Archive -Path "target/i686-pc-windows-msvc/release/pasta.dll" -DestinationPath "target/i686-pc-windows-msvc/release/pasta.dll.zip" -Force` を実行する
  - `Test-Path "target/i686-pc-windows-msvc/release/pasta.dll.zip"` で zip ファイルの存在を確認する
  - 確認失敗時: エラーを報告してリリース作業を中断する
  - `git add -A && git commit -m "chore(release): build hello-pasta vX.Y.Z"` を実行する
  - 完了条件: pasta.dll.zip が存在し、ビルドコミットが git ログに記録されている
  - _Requirements: 5.5, 5.6, 5.7, 5.8_

## Task 6: リリースタグとリモートプッシュ

- [ ] 6. リリースタグとリモートプッシュ
- [ ] 6.1 Git アノテーションタグを作成する
  - `git tag -l "vX.Y.Z"` で既存タグの競合を確認する
  - 既存タグがある場合: 開発者に対応方法を確認する（自動削除はしない）
  - `git tag -a vX.Y.Z -m "Release vX.Y.Z"` を実行する
  - 完了条件: `git tag -l "vX.Y.Z"` でタグが確認できる
  - _Requirements: 6.1, 6.2, 6.3_
- [ ] 6.2 コミットとタグをリモートにプッシュする
  - `git push origin main --tags` を実行する
  - 失敗時: 段階的バックオフでリトライし、それでも失敗なら手動実行手順を案内する
  - 完了条件: `git push origin main --tags` が成功し、GitHub のリモートブランチとタグが更新されている
  - _Requirements: 6.4, 6.5_

## Task 7: GitHub Release 作成と完了報告

- [ ] 7. GitHub Release 作成と完了報告
- [ ] 7.1 コミット履歴からチェンジログを生成する
  - `git tag -l "v*" --sort=-version:refname` で前回タグを特定する
  - 前回タグがある場合: `git log <前回タグ>..vX.Y.Z --oneline --no-merges` でコミット履歴を取得する
  - 前回タグがない場合（初回）: `git log --oneline --no-merges` で全履歴を取得する
  - 取得したコミットを Conventional Commits 形式で分類・グループ化する
  - スコープが `spec` のコミットはチェンジログから除外する
  - グループ別（Features/Bug Fixes/Refactoring/etc.）に見出し配下へ箇条書きで整形する
  - チェンジログを一時ファイル `release-notes-vX.Y.Z.md` に書き出す
  - 完了条件: `release-notes-vX.Y.Z.md` が作成され、整形済みチェンジログが含まれている
  - _Requirements: 7.1, 7.2, 7.3, 7.9_
- [ ] 7.2 GitHub Release を作成しアセットを添付する
  - VSIX ファイルが存在する場合（`$env:VSIX_PATH`）: アセットリストに追加する
  - `gh release create vX.Y.Z "target/i686-pc-windows-msvc/release/pasta.dll.zip" "crates/pasta_sample_ghost/hello-pasta.nar" [<VSIX>] --title "pasta vX.Y.Z" --notes-file release-notes-vX.Y.Z.md` を実行する
  - 失敗時: 段階的バックオフでリトライし、それでも失敗なら手動実行手順を案内する
  - `Remove-Item release-notes-vX.Y.Z.md` で一時ファイルを削除する
  - 完了条件: GitHub の `ekicyou/pasta` リポジトリにリリースページが作成され、アセットが添付されている
  - _Requirements: 7.4, 7.5, 7.6, 7.7, 7.8_
- [ ] 7.3 リリース完了サマリーを報告する
  - 以下の内容を含む完了サマリーを開発者に報告する：
    - リリースバージョン: `vX.Y.Z`
    - 公開クレート: pasta_core, pasta_dsl, pasta_lua, pasta_shiori, pasta_check
    - GitHub Release URL: `https://github.com/ekicyou/pasta/releases/tag/vX.Y.Z`
    - VSCode Marketplace: 公開成功 URL または警告メッセージ（Task 4 の結果を反映）
  - 完了条件: 上記情報を含む完了報告が開発者に提示されている
  - _Requirements: 8.1, 8.2, 8.3, 8.4_
