# Implementation Plan

## Task Format Template

Use whichever pattern fits the work breakdown:

### Major task only
- [ ] {{NUMBER}}. {{TASK_DESCRIPTION}}{{PARALLEL_MARK}}
  - {{DETAIL_ITEM_1}} *(Include details only when needed. If the task stands alone, omit bullet items.)*
  - _Requirements: {{REQUIREMENT_IDS}}_

### Major + Sub-task structure
- [ ] {{MAJOR_NUMBER}}. {{MAJOR_TASK_SUMMARY}}
- [ ] {{MAJOR_NUMBER}}.{{SUB_NUMBER}} {{SUB_TASK_DESCRIPTION}}{{SUB_PARALLEL_MARK}}
  - {{DETAIL_ITEM_1}}
  - {{DETAIL_ITEM_2}}
  - _Requirements: {{REQUIREMENT_IDS}}_ *(IDs only; do not add descriptions or parentheses.)*

> **Parallel marker**: Append ` (P)` only to tasks that can be executed in parallel. Omit the marker when running in `--sequential` mode.
>
> **Optional test coverage**: When a sub-task is deferrable test work tied to acceptance criteria, mark the checkbox as `- [ ]*` and explain the referenced requirements in the detail bullets.

---

## Implementation Tasks

- [x] 1. release.bat 実装
  - PowerShell スクリプト起動ラッパースクリプトを作成
  - ダブルクリック実行可能な Windows バッチファイルとして実装
  - `release.ps1` を `-ExecutionPolicy Bypass` フラグ付きで起動
  - エラー時は一時停止してメッセージを表示（`pause`）
  - _Requirements: 1.3_

- [x] 2. release.ps1 コアロジック実装
- [x] 2.1 (P) バージョン確認機能を実装
  - ワークスペースルート `Cargo.toml` から `workspace.package.version` を読み取り
  - バージョン文字列を `v<version>` 形式のタグ名に変換（例: `v0.1.1`）
  - バージョン情報を画面に表示（リリース作業のための確認情報）
  - _Requirements: 2.2, 2.3_

- [x] 2.2 (P) 配布物検証機能を実装
  - `ghosts/hello-pasta/` 配下の必須ファイルを `Test-Path` でチェック
  - 検証対象: `ghost/master/pasta.dll`, `pasta.toml`, `descript.txt`, `dic/*.pasta`, `scripts/`, `shell/master/`, `install.txt`, `updates.txt`, `updates2.dau`
  - いずれかのファイルが見つからない場合、エラーメッセージを表示して中断
  - _Requirements: 4.1, 4.2_

- [x] 2.3 .nar 生成機能を実装
  - 一時ディレクトリ作成（`temp_release/`）
  - `robocopy /MIR /XD profile /XF *.bak *.tmp` で `ghosts/hello-pasta/` を一時ディレクトリにコピー
  - `Compress-Archive` で一時ディレクトリを ZIP 圧縮（`hello-pasta.zip`）
  - ZIP ファイルを `.nar` 拡張子にリネーム（`hello-pasta.nar`）
  - `crates/pasta_sample_ghost/` に移動
  - 一時ディレクトリをクリーンアップ（`Remove-Item -Recurse -Force`）
  - _Requirements: 1.1, 1.2_

- [x] 2.4 リリース手順表示機能を実装
  - `.nar` 生成完了メッセージを表示
  - 次のステップガイドを表示:
    - 確認すべきバージョン番号（2.1 で取得した値）
    - `gh release create` コマンド例（タグ名、アセットパス、notes オプション付き）
    - `RELEASE.md` を参照して AI と相談しながらリリースする旨の案内
  - _Requirements: 2.1, 2.4_

- [x] 3. RELEASE.md 作成 (P)
  - AI ガイド付きリリース作業手順書を `crates/pasta_sample_ghost/RELEASE.md` に作成
  - リリースノート本文テンプレートを含める:
    - バージョン番号（プレースホルダー `{VERSION}`）
    - リリース概要（変更点・新機能のセクション）
    - 含まれるコンポーネント（pasta.dll, hello-pasta ゴースト）
    - 必要環境（SSP 2.x 以上、Windows x86）
    - インストール手順（ドラッグ＆ドロップ、手動展開）
    - 動作確認方法（ゴースト切り替え、会話動作確認）
    - 問題報告先（GitHub Issues リンク）
  - `gh release create` コマンド実行例を含める（タグ名、アセット、notes 引数のサンプル）
  - AI と相談しながらリリースする旨の手順を明記
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4_

- [x] 4. release.ps1 統合テスト
  - `setup.bat` を実行して `ghosts/hello-pasta/` を生成
  - `release.bat` をダブルクリック実行し、`hello-pasta.nar` が正常に生成されることを確認
  - `.nar` ファイルを展開（ZIP として）し、以下を検証:
    - `profile/` ディレクトリが含まれていないこと
    - `*.bak`, `*.tmp` ファイルが含まれていないこと
    - `ghost/master/pasta.dll` が存在すること
    - ファイルサイズが妥当であること（数 MB 程度）
  - リリース手順表示の内容が正しいこと（バージョン、コマンド例）
  - _Requirements: 1.1, 1.2, 4.1_

- [ ]* 5. SSP インストール検証
  - 生成した `hello-pasta.nar` を SSP にドラッグ＆ドロップ
  - SSP のゴースト一覧に hello-pasta が表示されることを確認（1.2）
  - ゴーストを切り替えて会話が正常に動作することを確認（3.3）
  - _Requirements: 1.2, 3.3_

---

## Notes

- すべてのタスクは手動実行を前提としており、CI/CD パイプラインは含まない（Out of Scope）
- タスク 5 は SSP のインストール環境が必要なため、開発環境のセットアップが完了している前提
- `release.ps1` の ZIP 圧縮形式が SSP と互換性があるかはタスク 5 で初回検証
- タスク 1〜3 は並列実装可能（ファイル依存なし）。タスク 4〜5 は直列実行必須（成果物検証）
