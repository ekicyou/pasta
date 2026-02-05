# Requirements Document

## Project Description (Input)
（仕様以外の）全公開ドキュメントの実装状況進捗評価及び、現在実装へのフィードバックを網羅的に行う。追加で必要となるドキュメントが判明したら追加する。既存ドキュメントについて重複などが生じている場合は、統合・削除も視野に検討する。

## Introduction

本仕様は、pastaプロジェクトにおける公開ドキュメント（SPECIFICATION.mdを除く）の実装状況を網羅的に評価し、現在の実装との整合性を検証するためのものです。ドキュメントの重複・欠落を特定し、追加・統合・削除のアクションを決定します。

### 除外対象

- **SPECIFICATION.md**: 言語仕様書（別途仕様管理対象）
  - ただし、steering/grammar.mdの「権威的参照」としての整合性検証には使用する

### 対象ドキュメント一覧

| カテゴリ    | ドキュメント                 | 役割                                          |
| ----------- | ---------------------------- | --------------------------------------------- |
| **Level 0** | README.md                    | プロジェクト概要・エントリーポイント          |
| **Level 1** | SOUL.md                      | プロジェクトの憲法（ビジョン・設計原則）      |
| **Level 1** | GRAMMAR.md                   | DSL文法マニュアル（人間向け・読みやすさ優先） |
| **Level 1** | AGENTS.md                    | AI開発支援ドキュメント                        |
| **Level 2** | pasta_core/README.md         | パーサー・レジストリAPI                       |
| **Level 2** | pasta_lua/README.md          | Luaトランスパイラ・ランタイム                 |
| **Level 2** | pasta_lua/LUA_API.md         | Lua APIリファレンス                           |
| **Level 2** | pasta_shiori/README.md       | SHIORI DLLインターフェース                    |
| **Level 2** | pasta_sample_ghost/README.md | サンプルゴースト                              |
| **Level 2** | TEST_COVERAGE.md             | テストカバレッジマップ（品質管理）            |
| **Level 2** | OPTIMIZATION.md              | トランスパイラ最適化リファレンス（品質管理）  |
| **Level 2** | SCENE_TABLE_REVIEW.md        | シーンテーブル設計レビュー（品質管理）        |
| **Level 3** | .kiro/steering/*.md          | AI向けプロジェクトルール（完全性優先）        |

### 議題ディスカッション決定事項（2026-02-05）

| 議題 | 決定内容                                                                                    |
| ---- | ------------------------------------------------------------------------------------------- |
| #1   | steering/grammar.md: AI向け完全参照（SPECIFICATION.md準拠）、GRAMMAR.md: 人間向けマニュアル |
| #2   | CHANGELOG.md: Phase 4（外部公開準備時）に初版作成                                           |
| #3   | CONTRIBUTING.md: 実際のPR発生まで延期（YAGNI原則）                                          |
| #4   | SOUL.md Level 2: 品質管理ドキュメント（TEST_COVERAGE.md等）を追加                           |

---

## Requirements

### Requirement 1: ドキュメント実装整合性評価

**Objective:** 開発者として、各ドキュメントの記述内容が現在の実装と整合していることを確認し、乖離がある場合は修正アクションを特定したい。

#### Acceptance Criteria

1. When 各ドキュメントをレビューするとき, the レビューアは ドキュメントに記載された機能・API・構造が実装と一致しているか検証すること
2. If ドキュメントと実装に乖離がある場合, then the レビューアは 乖離内容と推奨修正アクション（ドキュメント更新 or 実装更新）を特定すること
3. While 評価を実施する間, the レビューアは 各ドキュメントの評価結果を「✅ 整合」「⚠️ 軽微な乖離」「❌ 重大な乖離」で分類すること
4. The レビューアは 全対象ドキュメントについて評価を完了し、結果を一覧化すること

---

### Requirement 2: ドキュメント重複・冗長性検出

**Objective:** 開発者として、複数ドキュメント間で重複している内容を特定し、情報の一元化により保守性を向上させたい。

#### Acceptance Criteria

1. When 複数ドキュメントを横断レビューするとき, the レビューアは 同一または類似の情報が複数箇所に記載されている箇所を特定すること
2. If 重複コンテンツが検出された場合, then the レビューアは 「正規の情報源（Single Source of Truth）」となるべきドキュメントを決定し、他箇所からは参照リンクへの置換を推奨すること
3. The レビューアは 特にsteering/*.md ⇔ ルートドキュメント間の重複を重点的に検証すること
4. When steering/grammar.md をレビューするとき, the レビューアは GRAMMAR.md（人間向け）との役割分離が明確であることを検証し、AI向け完全参照としてSPECIFICATION.md準拠であることを確認すること

---

### Requirement 3: 欠落ドキュメントの特定

**Objective:** 開発者として、現在の実装に対して不足しているドキュメントを特定し、追加すべきドキュメントを明確化したい。

#### Acceptance Criteria

1. When 実装済み機能を棚卸しするとき, the レビューアは 対応するドキュメントが存在しない機能・APIを特定すること
2. If ドキュメント化されていない重要機能がある場合, then the レビューアは 新規ドキュメント作成の推奨と、そのスコープ・配置場所を提案すること
3. Where ドキュメントヒエラルキー（SOUL.md定義）に従う場合, the レビューアは 新規ドキュメントの適切なレベル（Level 0-3）を決定すること

---

### Requirement 4: ドキュメントヒエラルキー整合性

**Objective:** 開発者として、SOUL.mdで定義されたドキュメントヒエラルキーと実際のドキュメント構成が整合していることを確認したい。

#### Acceptance Criteria

1. The レビューアは SOUL.mdのドキュメントヒエラルキー定義と実際のドキュメント構成を比較検証すること
2. If ヒエラルキー定義に記載されているが存在しないドキュメントがある場合, then the レビューアは 欠落として報告すること
3. If 存在するがヒエラルキー定義に含まれていないドキュメントがある場合, then the レビューアは 追加登録または削除を推奨すること
4. The レビューアは Level 2を「実装層ドキュメント」として再定義し、クレートREADMEと品質管理ドキュメント（TEST_COVERAGE.md, OPTIMIZATION.md, SCENE_TABLE_REVIEW.md）を含めることを検証すること

---

### Requirement 5: クレートREADME整合性

**Objective:** 開発者として、各クレートのREADME.mdが実際の公開APIおよびディレクトリ構造と整合していることを確認したい。

#### Acceptance Criteria

1. When pasta_core/README.md をレビューするとき, the レビューアは 記載されたAPI（parse_str, parse_file等）およびディレクトリ構造が実装と一致することを検証すること
2. When pasta_lua/README.md をレビューするとき, the レビューアは ディレクトリ構成、pasta.toml設定オプション、[actor.*]セクション説明が実装と一致することを検証すること
3. When pasta_lua/LUA_API.md をレビューするとき, the レビューアは 記載されたLuaモジュール（@pasta_search, @pasta_persistence等）が実際に公開されていることを検証すること
4. When pasta_shiori/README.md をレビューするとき, the レビューアは SHIORIプロトコルフローおよびAPIが実装と一致することを検証すること
5. When pasta_sample_ghost/README.md をレビューするとき, the レビューアは セットアップ手順が実際に動作することを検証すること

---

### Requirement 6: 品質管理ドキュメント整合性

**Objective:** 開発者として、TEST_COVERAGE.md、OPTIMIZATION.md、SCENE_TABLE_REVIEW.mdの内容が現在の実装状況を正確に反映していることを確認したい。

#### Acceptance Criteria

1. When TEST_COVERAGE.md をレビューするとき, the レビューアは 記載されたテスト数・パス状況が `cargo test --workspace` の実行結果と一致することを検証すること
2. When OPTIMIZATION.md をレビューするとき, the レビューアは 記載された最適化（TCO、アクター最適化等）が実際のcode_generator.rsに実装されていることを検証すること
3. When SCENE_TABLE_REVIEW.md をレビューするとき, the レビューアは 記載されたアーキテクチャがscene_table.rs、scene.luaの実装と一致することを検証すること

---

### Requirement 7: Steeringドキュメント整合性

**Objective:** 開発者として、.kiro/steering/配下のステアリングドキュメントが現在のプロジェクト状況を正確に反映していることを確認したい。

#### Acceptance Criteria

1. When product.md をレビューするとき, the レビューアは Phase進捗、完了仕様数、品質指標が実際の状況と一致することを検証すること
2. When tech.md をレビューするとき, the レビューアは 技術スタック（Rust edition、mlua version等）およびワークスペース構成が実際のCargo.tomlと一致することを検証すること
3. When structure.md をレビューするとき, the レビューアは ディレクトリ構造記載が実際のワークスペースと一致することを検証すること
4. When workflow.md をレビューするとき, the レビューアは コマンド一覧およびDoDが最新であることを検証すること
5. When grammar.md をレビューするとき, the レビューアは AI向け完全参照としてSPECIFICATION.md準拠であること、Runeブロック例がLuaブロック例に更新されていることを検証すること

---

### Requirement 8: アクションプラン策定

**Objective:** 開発者として、評価結果に基づいた具体的なアクションプラン（追加・更新・統合・削除）を策定したい。

#### Acceptance Criteria

1. The レビューアは 評価結果を以下のカテゴリで分類すること：
   - **追加**: 新規作成が必要なドキュメント
   - **更新**: 既存ドキュメントの修正が必要な箇所
   - **統合**: 複数ドキュメントの統合が推奨される箇所
   - **削除**: 不要または廃止すべきドキュメント
2. When アクションを優先度付けするとき, the レビューアは 開発への影響度（High/Medium/Low）を付与すること
3. The レビューアは 各アクションに対して具体的な実施内容を記載すること
