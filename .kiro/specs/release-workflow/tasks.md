# Implementation Plan: release-workflow

## タスク概要

本仕様は**オペレーション仕様**であり、コードの新規作成・変更を伴わない。LLM エージェントが `/kiro-spec-impl release-workflow` を実行するたびに、以下のタスクを順次実行してリリース作業を遂行する。

**タスクの特殊性**:
- タスク完了 = リリース1回の実行完了
- 各実行ごとにタスク状態はリセットされる（繰り返し実行型）
- 実装フェーズ = 実際のリリース作業の実行

## タスク一覧

### Phase 0: 前提条件確認

- [ ] 1. GitHub CLI 認証確認
  - `gh auth status` を実行し、認証状態を確認する
  - 未認証の場合は `gh auth login` のガイダンスを開発者に提示し、認証完了を待つ
  - _Requirements: Phase 0 暗黙的前提_

### Phase 1: 事前検証

- [ ] 2. バージョン番号の決定と承認
  - 開発者からバージョン指定がある場合はそれを使用する
  - 指定がない場合は `Cargo.toml` の `[workspace.package].version` を読み取り、PATCH を +1 して提案する
  - 提案形式「vX.Y.Z から vX.Y.(Z+1) に更新します。よろしいですか？」で開発者に確認
  - 拒否された場合は希望バージョンの入力を求める
  - semver 形式（`^[0-9]+\.[0-9]+\.[0-9]+$`）として妥当性を検証する
  - 形式エラー時は再入力を求める
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_

- [ ] 3. ワークツリーの整理とテスト実行
  - `git status --porcelain` で未コミット変更を確認する
  - 未コミット変更がある場合は `git add -A && git commit -m "chore(release): prepare release vX.Y.Z"` で自動コミットする
  - `cargo test --all` を実行し全テストの通過を確認する
  - テスト失敗時はエラー内容を報告し、リリース作業を中止する
  - _Requirements: 1.7, 1.8, 1.9, 1.10_

### Phase 2: バージョン更新

- [ ] 4. Cargo.toml のバージョン一括更新
  - ルート `Cargo.toml` の以下4箇所を `replace_string_in_file` で更新する:
    - `[workspace.package].version = "<OLD>"` → `version = "<NEW>"`
    - `pasta_core = { path = "crates/pasta_core", version = "<OLD>" }` → `version = "<NEW>"`
    - `pasta_lua = { path = "crates/pasta_lua", version = "<OLD>" }` → `version = "<NEW>"`
    - `pasta_shiori = { path = "crates/pasta_shiori", version = "<OLD>" }` → `version = "<NEW>"`
  - _Requirements: 2.1, 2.2_

- [ ] 5. ビルド検証とコミット
  - `cargo build --workspace` を実行してビルド成功を確認する
  - ビルド失敗時は `git restore Cargo.toml` でロールバックし、エラーを報告して中止する
  - ビルド成功時は `git add Cargo.toml && git commit -m "chore(release): bump version to vX.Y.Z"` でコミットする
  - _Requirements: 2.3, 2.4, 2.5_

### Phase 3: crates.io 公開

- [ ] 6. 依存関係順での crates.io 公開
  - 以下の順序でクレートを公開する: `pasta_core` → `pasta_lua` → `pasta_shiori`
  - 各クレートに対して `cargo publish -p <crate_name>` を実行する
  - 失敗時は最大2回リトライ（合計3回試行）する
  - 3回失敗した場合はエラーを報告し、既に公開済みのクレートはそのまま残して以降を中断する
  - 各クレート公開後（最後の `pasta_shiori` を除く）に `Start-Sleep -Seconds 10` で待機する
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

### Phase 4: ゴーストビルド

- [ ] 7. サンプルゴーストのビルドと成果物確認
  - `crates/pasta_sample_ghost/` ディレクトリで `PowerShell -ExecutionPolicy Bypass -File release.ps1` を実行する
  - `Test-Path "crates/pasta_sample_ghost/hello-pasta.nar"` で .nar ファイルの生成を確認する
  - `Test-Path "target/i686-pc-windows-msvc/release/pasta.dll"` で DLL の存在を確認する
  - いずれかが存在しない場合はエラー報告し中断する
  - 成功時は `git add -A && git commit -m "chore(release): build hello-pasta vX.Y.Z"` でコミットする
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

### Phase 5: タグとプッシュ

- [ ] 8. Git タグの作成とリモートプッシュ
  - `git tag -l "vX.Y.Z"` で既存タグの競合を確認する
  - 競合がある場合は開発者に「手動で `git tag -d vX.Y.Z` を実行しますか？」と確認する
  - `git tag -a vX.Y.Z -m "Release vX.Y.Z"` でアノテーションタグを作成する
  - `git push origin main --tags` でコミットとタグをリモートにプッシュする
  - プッシュ失敗時はエラー報告し「手動で `git push origin main --tags` を再実行してください」と案内する
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
    ```
    gh release create vX.Y.Z `
      "target/i686-pc-windows-msvc/release/pasta.dll" `
      "crates/pasta_sample_ghost/hello-pasta.nar" `
      --title "pasta vX.Y.Z" `
      --notes-file release-notes-vX.Y.Z.md
    ```
  - `gh` 失敗時はエラー報告と手動手順を案内する
  - 成功時は一時ファイル `release-notes-vX.Y.Z.md` を削除する
  - リリース完了サマリー（バージョン、公開クレート、Release URL）を開発者に報告する
  - _Requirements: 6.4, 6.5, 6.6, 6.7, 6.8, 6.9, 7.4_

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
| 4.1–4.5 | 7 |
| 5.1–5.5 | 8 |
| 6.1–6.3 | 9 |
| 6.4–6.9, 7.4 | 10 |
| 7.1–7.3 | 11 |

全47個の Acceptance Criteria がカバーされています。
