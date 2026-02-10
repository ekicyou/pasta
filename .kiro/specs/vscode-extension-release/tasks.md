# Implementation Plan: vscode-extension-release

## タスク概要

本仕様は **2つの側面** を持つ：
1. **ドキュメント・メタデータ整備**（ワンショット作業）
2. **release-workflow 統合** + **初回リリース実施**（v0.1.3）

全タスクを順次実行し、Marketplace 公開と release-workflow 統合を完了する。

---

## タスク一覧

### Phase 1: ドキュメント・メタデータ整備

- [x] 1. (P) Marketplace 公開用 README のリライト
  - 現在の `editors/vscode/README.md` を Marketplace 向けユーザー視点に全面リライト
  - 概要を冒頭に配置（1〜2文、package.json の description と整合）
  - 主要機能一覧を箇条書き（TextMate 文法、セマンティックトークン、診断情報、フォールバック）
  - スクリーンショット `img/screenshot-syntax-highlight.png` を README に埋め込み
  - 対応 VSCode バージョン要件（`engines.vscode: ^1.85.0`）を記載
  - Pasta DSL の簡潔な紹介とリポジトリリンクを含める
  - MIT ライセンス記載
  - 開発者向け情報（ビルド手順、アーキテクチャ図）は削除または折りたたみ
  - セマンティックトークン一覧テーブルは維持（現在の README から転記）
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9_

- [x] 2. (P) CHANGELOG の新規作成
  - `editors/vscode/CHANGELOG.md` を新規作成
  - Keep a Changelog 形式に準拠（ヘッダー、リンク含む）
  - 初回リリース v0.1.3 のエントリを作成
  - 全実装機能を `Added` カテゴリに列挙（TextMate 文法、WASM LSP、セマンティックトークン、診断情報、デバウンス同期、言語登録）
  - リリース日は実施日に手動更新（YYYY-MM-DD プレースホルダー使用）
  - _Requirements: 2.1, 2.2, 2.3, 2.5_

- [x] 3. (P) package.json メタデータの拡充
  - `editors/vscode/package.json` に以下フィールドを追加:
    - `icon`: `"img/pasta.svg"`（プロジェクトルートの既存ロゴ）
    - `keywords`: `["pasta", "dsl", "ukagaka", "ghost", "scripting"]`
    - `homepage`: `"https://github.com/ekicyou/pasta"`
    - `bugs`: `{"url": "https://github.com/ekicyou/pasta/issues"}`
  - 既存フィールド（`publisher`, `categories`, `repository`）は保持
  - _Requirements: 3.1, 3.2, 3.3, 3.5, 3.6, 3.7, 3.8_

### Phase 2: release-workflow 統合設計の反映

- [x] 4. release-workflow の design.md への Phase 3.5 追加
  - `.kiro/specs/release-workflow/design.md` を編集
  - Architecture Pattern & Boundary Map の Mermaid 図に Phase 3.5 ノードを追加（Phase 3 と Phase 4 の間）
  - System Flows のシーケンス図に Phase 3.5 の手順を追加（npm install → package → vsce publish → VSIX パス保持）
  - Requirements Traceability テーブルに vscode-extension-release 要件（6.1, 6.2, 6.3, 6.5）を追加
  - Components and Interfaces テーブルに Phase 3.5 コンポーネントを追加
  - Phase 3.5 の詳細セクションを Components レイヤーに追加（VsixPackaging の実行手順を転記）
  - _Requirements: 6.1, 6.2_

- [x] 5. release-workflow の tasks.md への Phase 3.5 タスク追加
  - `.kiro/specs/release-workflow/tasks.md` を編集
  - Phase 2 のタスク 4（Cargo.toml バージョン更新）に package.json 更新を追加
  - Phase 1 のタスク 3（ワークツリー整理）に package.json バージョン不一致検知を追加
  - Phase 3 と Phase 4 の間に新規タスク挿入（Phase 3.5: VSCode 拡張公開）:
    - `cd editors/vscode`
    - `npm install`
    - `npm run package`（WASM ビルド + コンパイル + VSIX 生成）
    - VSIX 存在確認（`Test-Path pasta-vscode-X.Y.Z.vsix`）
    - 存在する場合: `vsce publish`、環境変数 `$env:VSIX_PATH` に保持
    - 失敗時: 警告記録、Phase 4 へ継続
  - Phase 6 のタスク 10（GitHub Release 作成）に VSIX アセット条件付き添付を追加
  - _Requirements: 6.1, 6.2, 6.3, 6.5_

- [x] 6. release-workflow の Phase 6 拡張（VSIX アセット添付）
  - `.kiro/specs/release-workflow/tasks.md` のタスク 10 を編集
  - `gh release create` コマンドに VSIX アセット追加の PowerShell コードを追加:
    - `$env:VSIX_PATH` 存在確認
    - 存在する場合のみアセット配列に追加
  - リリースサマリーに Marketplace 公開結果（成功 URL or 警告）を含める
  - _Requirements: 6.4, 6.6_

### Phase 3: 初回リリース実施（v0.1.3）

- [ ] 7. 初回 Marketplace 公開の準備確認
  - WASM ビルド環境の確認（wasm-pack インストール確認）
  - Node.js 依存パッケージの更新（`cd editors/vscode && npm install`）
  - vsce の認証状態確認（`vsce login ekicyou` が成功済みであること）
  - PAT 有効期限確認（2027-02-10 まで有効）
  - _Requirements: 5.3, 6.5_

- [ ] 8. VSIX パッケージングとパッケージ内容検証
  - `npm run package` 実行（prepackage で WASM ビルド + compile、package で vsce package）
  - 生成された VSIX ファイルサイズ確認（2MB 以下が目安）
  - `vsce ls` で VSIX 内容物を確認:
    - 7.2 除外対象が含まれていないこと（src/, scripts/, tsconfig.json 等）
    - 7.3 包含対象が含まれていること（out/extension.js, wasm/*.wasm, wasm/*.d.ts 等）
  - SVG アイコン警告の有無を確認し、research.md に記録
  - _Requirements: 5.1, 5.2, 5.5, 7.1, 7.2, 7.3_

- [ ] 9. Marketplace への公開実行
  - `vsce publish` 実行
  - Marketplace URL を記録（https://marketplace.visualstudio.com/items?itemName=ekicyou.pasta-vscode）
  - 公開成功メッセージの確認
  - 失敗時はエラー内容を research.md に記録し、手動対処
  - _Requirements: 5.3, 5.4_

- [ ] 10. Marketplace ページの表示確認
  - Marketplace ページにアクセスし、以下を確認:
    - README が正しく表示されていること（スクリーンショット、機能一覧、リンク）
    - アイコン（pasta.svg）が表示されていること
    - メタデータ（keywords, homepage, bugs）が反映されていること
    - CHANGELOG が表示されていること
  - 問題があれば research.md に記録
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.6, 2.1, 3.2, 3.5, 3.7, 3.8_

- [ ] 11. 拡張機能のインストール確認
  - Marketplace から拡張機能をインストール（または VSIX から直接インストール: `code --install-extension pasta-vscode-0.1.3.vsix`）
  - `.pasta` ファイルを開き、以下を確認:
    - TextMate 文法によるシンタックスハイライトが動作すること
    - セマンティックトークンが正しく適用されること（WASM ロード成功確認）
    - パースエラーが Problems パネルに表示されること
  - 問題があれば research.md に記録
  - _Requirements: 1.3, 1.4_

- [ ] 12. 初回リリース手順の文書化と問題点記録
  - research.md に以下を追記:
    - 初回リリース実施日
    - 発生した問題点（SVG アイコン警告、WASM ビルド時間等）
    - 改善提案（あれば）
  - CHANGELOG の `YYYY-MM-DD` を実施日に更新
  - コミット: `chore(vscode): 初回Marketplace公開完了 v0.1.3`
  - _Requirements: 2.4_

### Phase 4: バージョン同期機構の検証

- [ ] 13. package.json と Cargo.toml のバージョン不一致検知テスト
  - 意図的に package.json のバージョンを変更（例: 0.1.3 → 0.1.2）
  - release-workflow Phase 1 のタスク 3 で不一致が検知されることを確認
  - 元に戻す（0.1.3）
  - _Requirements: 4.4_

- [ ] 14. release-workflow での package.json バージョン更新テスト
  - release-workflow Phase 2 のタスク 4 で package.json が Cargo.toml と同時に更新されることを確認（ドライラン可能であれば実施、不可能であれば次回 v0.1.4 リリース時に検証）
  - semver 形式が維持されることを確認
  - _Requirements: 4.1, 4.2, 4.5_

### 最終タスク: ドキュメント整合性の確認と更新

- [ ] 15. ドキュメント整合性の確認と更新
  - 以下のドキュメントとの整合性を確認・更新:
    - ✅ SOUL.md: コアバリュー・設計原則との整合性確認（本仕様は VSCode 拡張リリースのため影響なし）
    - ❌ doc/spec/: 言語仕様の更新不要
    - ❌ GRAMMAR.md: 文法リファレンスの更新不要
    - ❌ TEST_COVERAGE.md: 新規テストなし（VSCode 拡張のテストは既存）
    - ✅ クレート README: 影響なし（VSCode 拡張のみ）
    - ✅ steering/*: 影響なし（リリースプロセスの拡張のみ）
  - 確認結果を記録
  - _Requirements: 本仕様のメタ要件（ドキュメント保守）_

---

## 要件カバレッジ検証

| Requirement | Tasks |
|-------------|-------|
| 1.1–1.9 | 1, 10 |
| 2.1–2.5 | 2, 10, 12 |
| 3.1–3.8 | 3, 10 |
| 4.1–4.5 | 13, 14 |
| 5.1–5.5 | 7, 8, 9 |
| 6.1–6.6 | 4, 5, 6 |
| 7.1–7.3 | 8 |

全 40 個の Acceptance Criteria がカバーされています。

---

## タスク実行ガイダンス

### 推奨実行順序

1. **Phase 1（タスク 1-3）**: 並列実行可能。ドキュメント・メタデータ整備
2. **Phase 2（タスク 4-6）**: 順次実行。release-workflow への統合設計反映
3. **Phase 3（タスク 7-12）**: 順次実行。初回リリース実施と検証
4. **Phase 4（タスク 13-14）**: 順次実行。バージョン同期機構の検証
5. **最終タスク（タスク 15）**: ドキュメント整合性確認

### Phase 間の依存関係

- Phase 2 は Phase 1 完了後に実施（ドキュメント整備完了が前提）
- Phase 3 は Phase 1 完了後に実施可能（Phase 2 とは独立）
- Phase 4 は Phase 2 完了後に実施（release-workflow 統合が前提）

### コミット戦略

- Phase 1 完了時: `docs(vscode): Marketplace公開用ドキュメント整備完了`
- Phase 2 完了時: `docs(spec): release-workflowにVSCode拡張統合`
- Phase 3 完了時: 各タスクで個別コミット、最後に `chore(vscode): 初回Marketplace公開完了 v0.1.3`
- Phase 4 完了時: `test(release): バージョン同期機構の検証完了`
- タスク 15 完了時: `docs: vscode-extension-release仕様完了`
