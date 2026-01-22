# Requirements Document

## Project Description (Input)
今作ってもらった情報などを含め、設定や仕様に関するドキュメントが散在している。一度全体をサルベージして整理し、最新ドキュメント群を構築する。現在はまだ未リリースのプロダクトなので、基本的にはAI参照を含む、開発に必要な情報を中心に整理する。

## Introduction

Pasta プロジェクトのドキュメントは現在、複数の場所に散在しています：

**現状のドキュメント配置**:
- ルートレベル: `README.md`, `GRAMMAR.md`, `SPECIFICATION.md`, `AGENTS.md`
- ステアリング: `.kiro/steering/` 配下の5ファイル
- クレート別: `crates/*/README.md`（一部のみ存在）
- テストフィクスチャ: `tests/fixtures/README.md`
- 内部ドキュメント: `crates/pasta_lua/scripts/README.md`, `scriptlibs/README.md` など

**課題**:
1. ドキュメント間の内容重複と不整合
2. AI開発支援に最適化されていない構造
3. クレート別READMEの欠落
4. 開発に必要な情報へのアクセス困難

本仕様は、これらのドキュメントを整理・統合し、開発者（人間およびAI）にとって参照しやすい構造を構築します。

---

## Requirements

### Requirement 1: ドキュメントインベントリ作成

**Objective:** As a 開発者, I want 全ドキュメントの現状把握, so that 整理・統合の計画を立てられる

#### Acceptance Criteria
1. When ドキュメント調査を実行すると, the Documentation System shall 全ての `.md` ファイルをリストアップする
2. When 各ドキュメントを分析すると, the Documentation System shall 以下を特定する: 対象読者、主要トピック、重複箇所、欠落情報
3. The Documentation System shall 現行ドキュメントマップを作成し、各ファイルの役割と相互関係を明示する

---

### Requirement 2: ドキュメント構造の再設計

**Objective:** As a 開発者（人間/AI）, I want 階層化された一貫性のあるドキュメント構造, so that 必要な情報に迅速にアクセスできる

#### Acceptance Criteria
1. The Documentation System shall 以下の階層構造を定義する:
   - **Level 0 (Entry Point)**: `README.md` - プロジェクト概要と全ドキュメントへのナビゲーション
   - **Level 1 (Core Docs)**: 言語仕様、文法、AI開発支援
   - **Level 2 (Crate Docs)**: 各クレートのREADME
   - **Level 3 (Steering)**: `.kiro/steering/` - AI/仕様駆動開発用コンテキスト
2. When AI開発ツールがプロジェクトを参照すると, the Documentation System shall `AGENTS.md` から全体コンテキストを取得できる構造を提供する
3. The Documentation System shall 各ドキュメントに「このドキュメントの目的」セクションを含め、対象読者を明示する

---

### Requirement 3: ルートレベルドキュメントの整理

**Objective:** As a 新規開発者, I want 明確なエントリーポイント, so that プロジェクトを素早く理解できる

#### Acceptance Criteria
1. The `README.md` shall 以下を含む: プロジェクト概要、クイックスタート、ドキュメントマップ、ライセンス
2. The `AGENTS.md` shall AI開発支援に必要な全情報へのポインタを含み、ステアリングファイルとの関係を明示する
3. The `GRAMMAR.md` shall Pasta DSL の文法リファレンスとして完結し、古い情報を更新して実装との乖離を解消する
   - **Note**: 仕様変更の可能性が高いため、SPECIFICATION.md との重複は許容（現状維持）
   - 作業範囲: 整理と実装乖離解消のみ、大幅な削減や構造変更は Out of Scope
4. The `SPECIFICATION.md` shall 言語の正式仕様書として、実装判断の権威的ソースを提供する

---

### Requirement 4: クレート別READMEの整備

**Objective:** As a クレート開発者, I want 各クレートの独立したドキュメント, so that クレート単位で開発・保守できる

#### Acceptance Criteria
1. The Documentation System shall 以下のクレートにREADME.mdを提供する:
   - `pasta_core/README.md` - パーサー、レジストリのAPI概要
   - `pasta_lua/README.md` - Luaトランスパイラ、ランタイム、ディレクトリ構成（✅ 作成済み）
   - `pasta_shiori/README.md` - SHIORI DLL統合
2. When 各クレートREADMEを参照すると, the Documentation System shall 以下を提供する:
   - クレートの責務と位置づけ
   - 公開API概要
   - 使用例
   - 依存関係
3. The Documentation System shall 各クレートREADMEからルートREADMEへのバックリンクを含める

---

### Requirement 5: ステアリングドキュメントの最適化

**Objective:** As a AI開発ツール, I want 構造化されたプロジェクトコンテキスト, so that 仕様駆動開発を正確に支援できる

#### Acceptance Criteria
1. The `.kiro/steering/` shall 以下のファイルを維持する:
   - `product.md` - プロダクトビジョン、フェーズ、優先順位
   - `tech.md` - 技術スタック、依存関係、アーキテクチャ原則
   - `structure.md` - ディレクトリ構造、命名規則、モジュール構成
   - `grammar.md` - DSL文法の要約と権威的仕様への参照
   - `workflow.md` - 開発ワークフロー、完了基準
2. When ステアリングファイルを更新すると, the Documentation System shall AGENTS.md との整合性を検証する
3. The Documentation System shall ステアリング間の重複を最小化し、各ファイルの責務を明確に分離する

---

### Requirement 6: ドキュメント相互参照の整備

**Objective:** As a ドキュメント読者, I want 関連情報へのリンク, so that 必要な情報を辿って参照できる

#### Acceptance Criteria
1. The Documentation System shall 全ドキュメント間に適切なクロスリファレンスを設定する
2. When 権威的仕様を参照すると, the Documentation System shall 明示的に `SPECIFICATION.md` へリンクする
3. If ドキュメントが古い情報を含む場合, the Documentation System shall 最新情報への参照または更新を行う
4. The Documentation System shall 孤立したドキュメント（他からリンクされていない）を特定し解消する

---

### Requirement 7: 開発者オンボーディングパスの定義

**Objective:** As a 新規貢献者, I want 明確な学習パス, so that 効率的にプロジェクトに参加できる

#### Acceptance Criteria
1. The `README.md` shall 以下のオンボーディングパスを提供する:
   - **ユーザー向け**: DSL文法 → サンプル → クイックスタート
   - **開発者向け**: アーキテクチャ → クレート構造 → コーディング規約
   - **AI開発支援向け**: AGENTS.md → ステアリング → 仕様
2. When 開発環境をセットアップすると, the Documentation System shall 必要なツールとコマンドを明示する
3. The Documentation System shall 各パスで読むべきドキュメントの順序を明示する

---

### Requirement 8: ドキュメント保守ガイドラインの策定

**Objective:** As a プロジェクトメンテナ, I want ドキュメント更新ルール, so that ドキュメントの品質を維持できる

#### Acceptance Criteria
1. The Documentation System shall ドキュメント更新時のチェックリストを提供する
2. When コード変更がAPIに影響すると, the Documentation System shall 対応するドキュメント更新を要求する（ワークフロードキュメントに記載）
3. The Documentation System shall 各ドキュメントに最終更新日または関連仕様へのリンクを含める
4. The Documentation System shall 「このドキュメントの保守責任」を明示する（どの仕様変更時に更新が必要か）

#### 8.5 アーカイブ整理タスク
1. **実施タイミング**: 全ドキュメント作成フェーズ完了後、実装の最終タスクとして実施
2. **整理対象**: `.kiro/specs/` 配下の完了済み仕様から、以下を削除
   - 古い仕様で現在の実装とかけ離れたもの
   - 今後のドキュメント作成時に混乱を招く恐れのある旧仕様
3. **目的**: 仕様作成時のブレを防ぎ、ファイル数を適正化
4. **判断基準**: Phase 1-4 でのドキュメント作成中に参照した情報の鮮度・relevance に基づき削除対象を選定
5. **手順**:
   - Phase 1-4 完了後、`.kiro/specs/` の全仕様を再評価
   - 実装とかけ離れた旧仕様をリストアップ
   - 削除実行前の最終確認
   - 削除実行とコミット

---

## Out of Scope

以下は本仕様の対象外とします：

1. **ユーザー向けチュートリアルの新規作成** - 開発者向け情報の整理が優先
2. **多言語対応** - 現時点では日本語のみ
3. **自動生成ドキュメント（rustdoc）の整備** - 別仕様で対応

---

## Success Metrics

| 指標                         | 目標                             |
| ---------------------------- | -------------------------------- |
| 孤立ドキュメント数           | 0件                              |
| クレートREADME網羅率         | 100%                             |
| ステアリング-AGENTS.md整合性 | 完全一致                         |
| 重複コンテンツ               | 意図的参照のみ（コピペ重複なし） |

