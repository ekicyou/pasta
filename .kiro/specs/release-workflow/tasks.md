# Implementation Plan: release-workflow

## タスク概要

本仕様は**オペレーション仕様**であり、コードの新規作成・変更を伴わない。LLM エージェントが `/kiro-spec-impl release-workflow` を実行するたびに、以下のタスクを順次実行してリリース作業を遂行する。

**タスクの特殊性**:
- タスク完了 = リリース1回の実行完了
- 各実行ごとにタスク状態はリセットされる（繰り返し実行型）
- 実装フェーズ = 実際のリリース作業の実行

**全タスク共通の実行条件**:
- 各タスクで発生した失敗がリカバリー不可能（リトライ上限到達・手動介入が必要・前提条件の不整合など）と判断された場合、その時点で**即座にリリース作業を失敗終了**する
- 失敗終了時は「どのタスクで・何が失敗したか・現在のリリース状態（何が完了済みで何が未実行か）」を明示し、開発者に手動対応の指示を行う
- **「既に完了済みのように見える」状態を理由に後続タスクをスキップしてはならない**。前回の部分的な実行が残存している可能性があるため、各タスクは必ず実行する（冪等性を持って実行する）

## タスク一覧

### Phase 0: 前提条件確認

- [ ] 1. GitHub CLI 認証確認
  - `gh auth status` を実行し、認証状態を確認する
  - 未認証の場合は `gh auth login` のガイダンスを開発者に提示し、認証完了を待つ
  - _Requirements: Phase 0 暗黙的前提_

### Phase 1: 事前検証

- [ ] 2. バージョン番号の決定と承認
  - **2-A: 全バージョンソースの並行調査**（サブエージェントを使って以下を同時に収集する）
    - Git タグ: `git tag --sort=-version:refname | Select-Object -First 1` → 最新タグ
    - Cargo.toml: `Select-String -Path "Cargo.toml" -Pattern '^\s*version\s*='` → ワークスペースバージョン
    - package.json: `(Get-Content "editors/vscode/package.json" | ConvertFrom-Json).version`
    - GitHub Releases: `gh release list --repo ekicyou/pasta --limit 1 --json tagName | ConvertFrom-Json | ForEach-Object { $_.tagName }` → 最新リリースタグ
    - crates.io (pasta_core): `(Invoke-RestMethod https://crates.io/api/v1/crates/pasta_core).crate.max_version`
    - crates.io (pasta_dsl): `(Invoke-RestMethod https://crates.io/api/v1/crates/pasta_dsl).crate.max_version`
    - crates.io (pasta_lua): `(Invoke-RestMethod https://crates.io/api/v1/crates/pasta_lua).crate.max_version`
    - crates.io (pasta_shiori): `(Invoke-RestMethod https://crates.io/api/v1/crates/pasta_shiori).crate.max_version`
    - crates.io (pasta_check): `(Invoke-RestMethod https://crates.io/api/v1/crates/pasta_check).crate.max_version`
    - VSCode Marketplace: `npx @vscode/vsce show ekicyou.pasta-vscode 2>&1 | Select-String 'Version:' | ForEach-Object { $_ -replace '.*Version:\s*', '' }`
  - **2-B: 最大バージョンの算出**
    - 上記全ソースから取得した全バージョン文字列（`v` プレフィックス除去後）を比較し、semver ルールで最大値を決定する
    - これを `$CURRENT_VERSION` とする
    - 取得に失敗したソースは「取得失敗（スキップ）」と記録し、残りのソースで最大値を算出する
    - 全ソースの取得結果と `$CURRENT_VERSION` を表形式で開発者に報告する
  - **2-C: 次バージョンの決定**
    - 開発者からバージョン指定がある場合はそれを使用する
    - 指定がない場合は `$CURRENT_VERSION` の PATCH を +1 した値を `$NEXT_VERSION` として提案する
    - 提案形式「全ソース調査の結果、現在の最大バージョンは vX.Y.Z です。vX.Y.(Z+1) に更新します。よろしいですか？」で開発者に確認
    - 拒否された場合は希望バージョンの入力を求める
  - **2-D: 重複チェック**
    - `$NEXT_VERSION` が2-Aで収集したいずれかのソースに**既に存在する場合は作業を即座に中止**する
    - エラーメッセージ例:「vX.Y.Z は既に [ソース名] に存在します。別のバージョン番号を指定してください。」
    - 開発者が別のバージョンを指定した場合は 2-D を再実行する
  - **2-E: 最終確認**
    - `$NEXT_VERSION` が semver 形式（`^[0-9]+\.[0-9]+\.[0-9]+$`）として妥当か検証する
    - 形式エラー時は再入力を求める
    - 確定したバージョンを `$NEW_VERSION` に保存し、以降のタスクで使用する
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_

- [ ] 3. ワークツリーの整理とテスト実行
  - **3-A: バージョンファイルの整合性検証**（自動コミット前に必ず実行）
    - `git diff HEAD -- Cargo.toml editors/vscode/package.json` で HEAD との差分を確認する
    - バージョンファイルに HEAD との差分がある場合:
      - 作業ツリーのバージョンと HEAD のバージョンをそれぞれ表示する
      - 作業ツリーのバージョンが HEAD より古い場合は **巻き戻し警告** を表示し、`git checkout HEAD -- Cargo.toml editors/vscode/package.json` で HEAD の状態に復元するか開発者に確認する
      - 開発者が復元を選択した場合は `git checkout HEAD --` で復元する
      - 開発者が作業ツリーの値を維持する場合はその旨を記録して続行する
  - **3-B: バージョン一致チェック**
    - `editors/vscode/package.json` の `version` と `Cargo.toml` の `workspace.package.version` を比較する
    - さらに、両者が Task 2 で算出した `$CURRENT_VERSION` と一致するか検証する
    - 不一致の場合は全バージョン値を表示して開発者に確認し、同期方法の指示を仰ぐ
  - **3-C: 未コミット変更の処理**
    - `git status --porcelain` で未コミット変更を確認する
    - 未コミット変更がある場合は `git add -A && git commit -m "chore(release): prepare release vX.Y.Z"` で自動コミットする
  - **3-D: テスト実行**
    - `cargo test --all` を実行し全テストの通過を確認する
    - テスト失敗時はエラー内容を報告し、リリース作業を中止する
  - _Requirements: 1.7, 1.8, 1.9, 1.10_

### Phase 2: バージョン更新

- [ ] 4. Cargo.toml のバージョン一括更新
  - **`<OLD>` の定義**: `<OLD>` は Task 3 完了後のファイルから実際に読み取ったバージョン文字列（= `$CURRENT_VERSION`）を指す。Task 3-B で `$CURRENT_VERSION` との一致が検証済みのため、この値を使用する
  - ルート `Cargo.toml` の以下5箇所を `replace_string_in_file` で更新する:
    - `[workspace.package].version = "<OLD>"` → `version = "<NEW>"`
    - `pasta_core = { path = "crates/pasta_core", version = "<OLD>" }` → `version = "<NEW>"`
    - `pasta_dsl = { path = "crates/pasta_dsl", version = "<OLD>" }` → `version = "<NEW>"`
    - `pasta_lua = { path = "crates/pasta_lua", version = "<OLD>" }` → `version = "<NEW>"`
    - `pasta_shiori = { path = "crates/pasta_shiori", version = "<OLD>" }` → `version = "<NEW>"`
  - `editors/vscode/package.json` の `"version": "<OLD>"` → `"version": "<NEW>"` を `replace_string_in_file` で更新する
  - 更新後、以下のコマンドで実際に書き換えられたことを検証する:
    - `Select-String -Path "Cargo.toml" -Pattern 'version = "<NEW>"'` で `[workspace.package]` のバージョンを確認
    - `Select-String -Path "Cargo.toml" -Pattern '"pasta_core".*version = "<NEW>"'` 等でワークスペース依存バージョンを確認（4クレート分）
    - `Select-String -Path "editors/vscode/package.json" -Pattern '"version": "<NEW>"'` でVSCode拡張バージョンを確認
  - 検証で `<NEW>` が見つからない箇所があれば、エラーを報告して中止する
  - _Requirements: 2.1, 2.2_

- [ ] 5. ビルド検証とコミット
  - `cargo build --workspace` を実行してビルド成功を確認する
  - ビルド失敗時は `git restore Cargo.toml editors/vscode/package.json` でロールバックし、エラーを報告して中止する
  - ビルド成功時は `git add Cargo.toml editors/vscode/package.json && git commit -m "chore(release): bump version to vX.Y.Z"` でコミットする
  - _Requirements: 2.3, 2.4, 2.5_

### Phase 3: crates.io 公開

- [ ] 6. 依存関係順での crates.io 公開
  - 以下の順序でクレートを公開する: `pasta_core` → `pasta_dsl` → `pasta_lua` → `pasta_check` → `pasta_shiori`
  - 各クレートに対して `cargo publish -p <crate_name>` を実行する
  - 失敗時は段階的バックオフでリトライする（待機 1分→2分→...→10分、最大10回）
  - 各リトライ前に `Start-Sleep -Seconds (N * 60)` で待機する（N=1,2,...,10）
  - 10分待機のリトライでも失敗した場合はエラーを報告し、既に公開済みのクレートはそのまま残して以降を中断、開発者の指示を待つ
  - 各クレート公開後（最後の `pasta_shiori` を除く）に `Start-Sleep -Seconds 10` で待機する
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

### Phase 3.5: VSCode 拡張公開

- [ ] 6.5. VSCode 拡張のビルドと Marketplace 公開
  - `cd editors/vscode && npm install` を実行する
    - 失敗時: 警告記録、Phase 4 へ継続
  - `npm run package` を実行する（prepackage: build:wasm + compile、package: vsce package）
    - VSIX 生成失敗時: 警告記録、Phase 4 へ継続
  - `Test-Path "pasta-vscode-X.Y.Z.vsix"` で VSIX 存在確認
  - 存在する場合: `vsce publish` を実行
    - 成功: Marketplace URL を記録
    - 失敗時: 段階的バックオフでリトライ（待機 1分→2分→...→10分、最大10回）
    - 各リトライ前に `Start-Sleep -Seconds (N * 60)` で待機する（N=1,2,...,10）
    - 10分待機のリトライでも失敗した場合: 警告記録、Phase 4 へ継続
  - 環境変数 `$env:VSIX_PATH` に VSIX ファイルパスを保持（Phase 6 で使用）
  - _Requirements: VSX.1–VSX.6_

### Phase 4: ゴーストビルド

- [ ] 7. サンプルゴーストのビルドと成果物確認
  - ワークスペースルートで `PowerShell -ExecutionPolicy Bypass -File crates\pasta_sample_ghost\release.ps1` を実行する
  - `Test-Path "release/hello-pasta.nar"` で .nar ファイルの生成を確認する
  - `Test-Path "target/i686-pc-windows-msvc/release/pasta.dll"` で DLL の存在を確認する
  - いずれかが存在しない場合はエラー報告し中断する
  - DLL 存在確認後、zip 圧縮を実行する:
    ```powershell
    Compress-Archive -Path "target/i686-pc-windows-msvc/release/pasta.dll" `
      -DestinationPath "target/i686-pc-windows-msvc/release/pasta.dll.zip" `
      -Force
    ```
  - `Test-Path "target/i686-pc-windows-msvc/release/pasta.dll.zip"` で zip 確認する
  - zip 圧縮失敗時はエラー報告し中断する
  - 成功時は `git add -A && git commit -m "chore(release): build hello-pasta vX.Y.Z"` でコミットする
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8_

### Phase 5: タグとプッシュ

- [ ] 8. Git タグの作成とリモートプッシュ
  - `git tag -l "vX.Y.Z"` で既存タグの競合を確認する
  - 競合がある場合は開発者に「手動で `git tag -d vX.Y.Z` を実行しますか？」と確認する
  - `git tag -a vX.Y.Z -m "Release vX.Y.Z"` でアノテーションタグを作成する
  - `git push origin main --tags` でコミットとタグをリモートにプッシュする
  - プッシュ失敗時は段階的バックオフでリトライ（待機 1分→2分→...→10分、最大10回）、それでも失敗の場合は「手動で `git push origin main --tags` を再実行してください」と案内する
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

### Phase 6: GitHub Release 作成

- [ ] 9. チェンジログの生成
  - `git tag -l "v*" --sort=-version:refname` で前回リリースタグを取得する
  - 前回タグがある場合は `git log <前回タグ>..vX.Y.Z --oneline --no-merges` でコミット履歴を取得する
  - 前回タグがない場合（初回リリース）は `git log --oneline --no-merges` で全履歴を取得する
  - Conventional Commits 形式（`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`）でコミットを分類する
  - スコープが `spec` のコミット（`chore(spec):`, `docs(spec):` 等）は除外する
  - 各カテゴリを見出し（`### ✨ Features`, `### 🐛 Bug Fixes` 等）配下に箇条書きで整形する
  - チェンジログを一時ファイル `release-notes-vX.Y.Z.md` に書き出す
  - _Requirements: 6.1, 6.2, 6.3_

- [ ] 10. GitHub Release の作成とアセット添付
  - 以下のコマンドで GitHub Release を作成する:
    ```powershell
    $assets = @(
      "target/i686-pc-windows-msvc/release/pasta.dll.zip",
      "release/hello-pasta.nar"
    )
    if ($env:VSIX_PATH -and (Test-Path $env:VSIX_PATH)) {
      $assets += $env:VSIX_PATH
    }

    gh release create vX.Y.Z `
      $assets `
      --title "pasta vX.Y.Z" `
      --notes-file release-notes-vX.Y.Z.md
    ```
  - `gh` 失敗時は段階的バックオフでリトライ（待機 1分→2分→...→10分、最大10回）、それでも失敗の場合はエラー報告と手動手順を案内する
  - 成功時は一時ファイル `release-notes-vX.Y.Z.md` を削除する
  - `Get-ChildItem "editors/vscode" -Filter "*.vsix" | Remove-Item -Force` で VSIX 成果物を全削除する
  - リリース完了サマリー（バージョン、公開クレート、Release URL、Marketplace 公開結果）を開発者に報告する
  - _Requirements: 6.4, 6.5, 6.6, 6.7, 6.8, 6.9, 7.4, VSX.4, VSX.6_

### 最終タスク: ドキュメント整合性確認

- [ ]* 11. ドキュメント整合性の確認と更新
  - 本仕様はオペレーション仕様であり、コード変更を伴わないため、以下の確認は**該当しない**
  - ドキュメント更新が不要であることを確認:
    - SOUL.md: コアバリュー・設計原則に影響なし
    - doc/spec/: 言語仕様変更なし
    - GRAMMAR.md: 文法リファレンス変更なし
    - TEST_COVERAGE.md: 新規テストなし
    - クレートREADME: API変更なし
    - steering/*: ステアリング更新なし
  - _Requirements: 7.1, 7.2, 7.3_

## 繰り返し実行の注意事項

- 各 `/kiro-spec-impl release-workflow` 実行時に全タスク（1〜11）を順次実行する
- タスク完了後、タスク状態はリセットされる（spec.json の `phase` は `ready_for_implementation` を維持）
- 各実行は独立したリリース作業として動作する

## 要件カバレッジ検証

| Requirement | タスク |
|-------------|--------|
| 1.1–1.6 | 2 |
| 1.7–1.10 | 3 |
| 2.1, 2.2 | 4 |
| 2.3–2.5 | 5 |
| 3.1–3.6 | 6 |
| 4.1–4.8 | 7 |
| 5.1–5.5 | 8 |
| 6.1–6.3 | 9 |
| 6.4–6.9, 7.4 | 10 |
| 7.1–7.3 | 11 |

全47個の Acceptance Criteria がカバーされています。
