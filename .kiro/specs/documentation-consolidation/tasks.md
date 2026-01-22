# Implementation Plan

## Task Overview

**Feature**: documentation-consolidation  
**Total Tasks**: 5 major tasks, 17 sub-tasks  
**Implementation Strategy**: 5-phase sequential execution (Phase 1-5)  
**Estimated Effort**: ~14-20 hours total

---

## Task Breakdown

### Phase 1: 相互参照整備（優先度: D - 最高優先）

- [x] 1. 全ドキュメント間の相互参照リンクを整備し、孤立ドキュメントを解消する
- [x] 1.1 全ルートレベルドキュメント（README.md, AGENTS.md, GRAMMAR.md, SPECIFICATION.md）に相互リンクを追加
  - README.md → GRAMMAR.md, SPECIFICATION.md, AGENTS.md へのリンク追加
  - GRAMMAR.md → SPECIFICATION.md への権威的参照リンク強化（トップ + 各セクション）
  - AGENTS.md → steering/* 全5ファイルへの明示的リンク追加
  - リンク形式: `[ドキュメント名](相対パス)`
  - _Requirements: 6.1, 6.2_

- [x] 1.2 既存クレートREADME（pasta_lua/README.md）からルートREADMEへのバックリンクを追加
  - 「プロジェクト概要」セクションにバックリンク設定
  - 他のドキュメントからの参照リンクを確認
  - _Requirements: 6.1, 6.3_

- [x] 1.3 孤立ドキュメント（他からリンクされていないドキュメント）を特定し解消
  - 全.mdファイルを走査し、リンク元がないファイルをリストアップ
  - 各孤立ドキュメントに対し適切な親ドキュメントからリンク設定
  - 内部ドキュメント（scriptlibs/README.md等）は適切な親からリンク、または現状維持を判断
  - _Requirements: 6.4_

---

### Phase 2: AGENTS.md 再構成（優先度: C）

- [x] 2. AGENTS.md を再構成し、ステアリングファイルとの関係を明示する
- [x] 2.1 ステアリングファイル参照セクションを追加
  - 新規セクション「Steering Files」を作成
  - テーブル形式で全5ファイル（product.md, tech.md, structure.md, grammar.md, workflow.md）の責務とリンクを明示
  - 各ステアリングファイルへのリンク設定
  - _Requirements: 3.2, 5.1_

- [x] 2.2 開発ワークフロー詳細化セクションを追加
  - Kiro仕様駆動開発ワークフローの詳細を追加（既存「Minimal Workflow」セクションを拡充）
  - ステアリング参照順序を明示（AGENTS.md → ステアリング → 仕様）
  - AI参照時の優先順位を明記
  - _Requirements: 5.2_

- [x] 2.3 ステアリングファイルとの相互リンクを設定
  - AGENTS.md → steering/* へのリンク（2.1で実施）
  - steering/*.md 内の適切な箇所に AGENTS.md への参照追加（必要に応じて）
  - ステアリング間の重複を最小化し、各ファイルの責務分離を確認
  - _Requirements: 5.3_

---

### Phase 3: README.md 拡充（優先度: B）

- [x] 3. README.md を拡充し、ドキュメントマップとオンボーディングパスを追加する
- [x] 3.1 ドキュメントマップセクションを追加
  - 新規セクション「ドキュメントマップ」を作成
  - 4階層構造（Level 0-3）を明示
  - 各レベルに属するドキュメントへのリンク設定
  - _Requirements: 2.1, 2.2, 3.1_

- [x] 3.2 オンボーディングパスセクションを追加
  - 新規セクション「オンボーディングパス」を作成
  - 3種類のユーザー向けパス（DSLユーザー、開発者、AI開発支援）を明示
  - 各パスでの推定所要時間を記載（DSLユーザー: 30分、開発者: 2-3時間、AI開発支援: 1時間）
  - 各パスで読むべきドキュメントの順序とリンクを明示
  - _Requirements: 2.3, 7.1, 7.2, 7.3_

- [x] 3.3 クイックスタートセクションを追加
  - 新規セクション「クイックスタート」を作成
  - 前提条件（Rust 2024 edition, cargo）を明示
  - ビルドコマンド、テストコマンドの例を記載
  - 開発環境セットアップの基本手順を明示
  - _Requirements: 3.1, 7.2_

---

### Phase 4: クレートREADME作成（優先度: A - 最低優先）

- [ ] 4. pasta_core と pasta_shiori のREADME.mdを新規作成する
- [ ] 4.1 pasta_core/README.md を新規作成
  - pasta_lua/README.md をテンプレートとして使用
  - 詳細度ガイドライン適用: pasta_lua/README.md同等の詳細度（使用例3-5個、内部構造は概要のみ、組み込みAPI一覧表形式）
  - 以下のセクションを含める: 概要、アーキテクチャ、ディレクトリ構成、公開API、使用例、依存関係、関連クレート
  - パーサー、レジストリAPIの概要を記載
  - ルートREADME、pasta_lua、pasta_shioriへのバックリンク設定
  - _Requirements: 4.1, 4.2, 4.3_

- [ ] 4.2 pasta_shiori/README.md を新規作成
  - pasta_lua/README.md をテンプレートとして使用
  - 詳細度ガイドライン適用: pasta_lua/README.md同等の詳細度（SHIORIプロトコル基本フロー図、主要イベント一覧表、外部仕様への参照リンク）
  - 以下のセクションを含める: 概要、アーキテクチャ、ディレクトリ構成、SHIORIプロトコル、依存関係、関連クレート
  - SHIORI DLL統合の責務を記載
  - ルートREADME、pasta_core、pasta_luaへのバックリンク設定
  - _Requirements: 4.1, 4.2, 4.3_

- [ ] 4.3 ルートREADME.mdのドキュメントマップにクレートREADMEへのリンクを追加
  - 既存のドキュメントマップセクション（3.1で作成）に pasta_core、pasta_shioriへのリンク追加
  - Level 2セクションでの明示的リンク設定確認
  - _Requirements: 4.3_

---

### Phase 5: 保守ガイドライン策定とアーカイブ整理（優先度: なし - 最終タスク）

- [ ] 5. ドキュメント保守ガイドラインを策定し、アーカイブを整理する
- [ ] 5.1 workflow.md にドキュメント保守セクションを追加
  - 新規セクション「ドキュメント保守」を作成
  - ドキュメント更新チェックリストテーブルを追加（変更種別 → 更新対象ドキュメント）
  - 保守責任テーブルを追加（ドキュメント → 更新トリガー）
  - API変更時のドキュメント更新要求ルールを明示
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [ ] 5.2 .kiro/specs/ 配下の完了済み仕様を再評価し、削除対象をリストアップ
  - 全完了済み仕様に対し削除判断フローチャート（design.md記載）を適用
  - COMPLETION_REPORT.md の有無、実装完了記載確認
  - 現在のコードと要件の整合性確認
  - Phase 1-4での参照有無確認
  - 情報の鮮度・参考価値評価
  - 削除候補リスト作成（仕様名と削除理由を明記）
  - _Requirements: 8.5_

- [ ] 5.3 削除候補リストを開発者に提示し、承認後に削除実行
  - 削除候補リストを開発者に提示
  - 開発者承認後、対象仕様をgit操作で削除
  - コミットメッセージに削除理由を記載
  - アーカイブ整理完了をドキュメント化
  - _Requirements: 8.5_

---

## Requirements Coverage

| Requirement        | Tasks         | Coverage Status                                                               |
| ------------------ | ------------- | ----------------------------------------------------------------------------- |
| 1.1, 1.2, 1.3      | 1.1, 1.3      | ✅ Covered                                                                     |
| 2.1, 2.2, 2.3      | 3.1, 3.2      | ✅ Covered                                                                     |
| 3.1                | 3.1, 3.3      | ✅ Covered                                                                     |
| 3.2                | 2.1           | ✅ Covered                                                                     |
| 3.3, 3.4           | -             | ❌ Out of Scope (GRAMMAR.md は現状維持、SPECIFICATION.md は本仕様では更新なし) |
| 4.1, 4.2, 4.3      | 4.1, 4.2, 4.3 | ✅ Covered                                                                     |
| 5.1, 5.2, 5.3      | 2.1, 2.2, 2.3 | ✅ Covered                                                                     |
| 6.1, 6.2, 6.3, 6.4 | 1.1, 1.2, 1.3 | ✅ Covered                                                                     |
| 7.1, 7.2, 7.3      | 3.2, 3.3      | ✅ Covered                                                                     |
| 8.1, 8.2, 8.3, 8.4 | 5.1           | ✅ Covered                                                                     |
| 8.5                | 5.2, 5.3      | ✅ Covered                                                                     |

**Note**: Requirements 3.3, 3.4 (GRAMMAR.md 整理、SPECIFICATION.md 更新) は要件定義で「整理と実装乖離解消のみ、大幅な削減や構造変更は Out of Scope」と明示されているため、本タスクでは扱いません。

---

## Implementation Notes

### Parallel Execution
- 本仕様は**sequential mode**で実行します（`(P)`マーカーなし）
- 理由: 各Phaseが前Phaseの成果物に依存し、ドキュメント整合性を保つため順次実行が必須

### Phase Dependencies
- **Phase 1 → Phase 2**: 相互参照完了後にAGENTS.md再構成
- **Phase 2 → Phase 3**: AGENTS.md更新後にREADME.md拡充
- **Phase 3 → Phase 4**: README.mdのドキュメントマップ完成後にクレートREADME作成
- **Phase 4 → Phase 5**: 全ドキュメント作成完了後に保守ガイドライン策定とアーカイブ整理

### Validation Checkpoints
- **Phase 1完了後**: 孤立ドキュメント0件確認
- **Phase 2完了後**: AGENTS.md-steering整合性確認
- **Phase 3完了後**: オンボーディングパスの順序妥当性確認
- **Phase 4完了後**: クレートREADME網羅率100%確認
- **Phase 5完了後**: アーカイブ整理完了確認

### Quality Gates
- **リンク切れチェック**: 各Phase完了後に手動確認
- **内容整合性チェック**: コード vs ドキュメント照合（特にPhase 4）
- **階層構造チェック**: ドキュメントマップ vs 実ファイル一致確認（Phase 3完了後）

---

## Success Criteria

すべてのタスクが完了し、以下を達成する：

1. ✅ 孤立ドキュメント数: 0件
2. ✅ クレートREADME網羅率: 100%（pasta_core, pasta_lua, pasta_shiori）
3. ✅ ステアリング-AGENTS.md整合性: 完全一致
4. ✅ 重複コンテンツ: 意図的参照のみ（コピペ重複なし）
5. ✅ ドキュメントマップ: 4階層構造完備
6. ✅ オンボーディングパス: 3種類明示
7. ✅ 保守ガイドライン: workflow.mdに追加完了
8. ✅ アーカイブ整理: 削除対象仕様を適切に削除

---

## Estimated Effort

| Phase   | Sub-tasks | Estimated Time |
| ------- | --------- | -------------- |
| Phase 1 | 3         | 2-3 hours      |
| Phase 2 | 3         | 2-3 hours      |
| Phase 3 | 3         | 3-4 hours      |
| Phase 4 | 3         | 4-6 hours      |
| Phase 5 | 3         | 3-4 hours      |

**Total**: 5 major tasks, 17 sub-tasks, ~14-20 hours
