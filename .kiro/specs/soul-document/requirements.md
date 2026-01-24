# Requirements Document

## Introduction

本仕様は、pastaプロジェクトの「あるべき姿」を定義するソウルドキュメントを整備し、その実装状態を検証するテスト体系を構築することを目的とする。ソウルドキュメントはプロジェクトのビジョン・設計思想・コアバリューを明文化し、テスト群はそれらが実現されているかを証明する。

## Project Description (Input)

pastaのあるべき姿、こんな実装を目標にしている、そういういわゆる「ソウルドキュメント」を整理したい。また、「ソウルドキュメント」に対して、現在、何ができていて、何ができていないかを証明するためのテスト群を整理する。

---

## Requirements

### Requirement 1: ソウルドキュメント体系の定義

**Objective:** As a pasta開発者, I want プロジェクトのビジョン・設計思想・コアバリューを体系的に整理したソウルドキュメント, so that 開発の方向性が明確になり、意思決定の基準が一貫する

#### Acceptance Criteria

1. The soul-document shall ビジョン・ミッションステートメントを「パスタのように絡み合う会話」というコア概念として定義する
2. The soul-document shall 以下のコアバリューを明文化する：
   - 日本語フレンドリー（全角キーワード対応）
   - UNICODE識別子（日本語シーン名・変数名サポート）
   - yield型エンジン（継続出力、チェイントーク対応）
   - 宣言的フロー（Call/Jumpによる制御）
3. The soul-document shall 行指向文法の設計原則を明示する
4. The soul-document shall 前方一致によるランダムジャンプ/単語選択の設計思想を記述する
5. The soul-document shall UI独立性（Wait/Syncはマーカーのみ）の原則を定義する

---

### Requirement 2: コア機能の実装状態を証明するテスト群

**Objective:** As a pasta開発者, I want ソウルドキュメントで定義された各機能の実装状態を検証するテスト群, so that 何ができていて何ができていないかが客観的に証明される

#### Acceptance Criteria

1. When テストスイートを実行した場合, the test-suite shall 各コア機能の実装状態を「完了」「部分完了」「未実装」で分類する
2. The test-suite shall 以下の文法機能に対するテストを含む：
   - グローバルシーン定義（`＊`）
   - ローカルシーン定義（`・`）
   - アクション行（アクター：発言）
   - 変数スコープ（ローカル`＄`、グローバル`＄＊`、システム`＄＊＊`）
   - Call文（`＞`）による制御フロー
   - 単語定義・参照（`＠`）
   - 属性定義（`＆`）
   - コメント行（`＃`）
3. The test-suite shall 前方一致検索（シーン・単語）の動作を検証するテストを含む
4. The test-suite shall 重複シーン・重複単語のランダム選択動作を検証するテストを含む
5. The test-suite shall Luaトランスパイル結果の正当性を検証するテストを含む

---

### Requirement 3: テスト結果と機能マッピングレポート

**Objective:** As a pasta開発者, I want テスト結果からソウルドキュメントの機能実装状況を可視化するレポート, so that プロジェクトの進捗状況が一目で把握できる

#### Acceptance Criteria

1. When テストを実行した場合, the report shall 各コア機能と対応するテストファイル・テストケースの対応表を出力する
2. The report shall 実装状態を以下のカテゴリで表示する：
   - ✅ 完了：テストが存在し、すべてパス
   - 🔶 部分完了：テストが存在し、一部パス
   - ⚠️ 未検証：機能は存在するがテストなし
   - ❌ 未実装：機能自体が未実装
3. The report shall Phase別（Phase 0〜4）の進捗サマリーを含む
4. If 新しいテストが追加された場合, then the report shall 自動的に機能マッピングを更新する

---

### Requirement 4: 既存テスト資産の整理・分類

**Objective:** As a pasta開発者, I want 既存のテストファイルをソウルドキュメントの機能分類に従って整理, so that テストの網羅性と重複が把握できる

#### Acceptance Criteria

1. The test-organization shall 既存テストを以下のカテゴリに分類する：
   - Parser層テスト（文法解析）
   - Registry層テスト（シーン/単語テーブル）
   - Transpiler層テスト（Lua変換）
   - Runtime層テスト（実行エンジン）
   - 統合テスト（E2E）
2. The test-organization shall 各テストファイルが検証する機能を明示するドキュメントを作成する
3. Where テストが複数機能をまたがる場合, the test-organization shall 主要機能と副次機能を区別して記録する
4. The test-organization shall テスト未カバーの機能リストを生成する

---

### Requirement 5: ソウルドキュメントとテストの整合性維持

**Objective:** As a pasta開発者, I want ソウルドキュメントの変更がテスト要件に反映される仕組み, so that ドキュメントと実装の乖離を防止できる

#### Acceptance Criteria

1. When ソウルドキュメントに新機能が追加された場合, the workflow shall 対応するテスト要件を生成する
2. The workflow shall ソウルドキュメントの各セクションとテストファイルの対応を維持するチェックリストを提供する
3. If ソウルドキュメントから機能が削除された場合, then the workflow shall 対応するテストの廃止検討をアラートする
4. The workflow shall 定期的なソウルドキュメント・テスト整合性レビューの手順を定義する

---

### Requirement 7: 未テスト領域の実装

**Objective:** As a pasta開発者, I want TEST_COVERAGE.mdで特定した未テスト領域に対するテストを実装, so that SOUL.mdで定義した「あるべき姿」が証明される

#### Acceptance Criteria

1. The test-implementation shall TEST_COVERAGE.md Section 4で特定した以下の未テスト領域に対するテストを作成する：
   - コメント行（＃）の明示的パーステスト
   - 属性定義（＆）の継承動作テスト
   - 変数スコープ（Local/Global/System）の完全テスト
   - 単語ランダム選択の統計的検証テスト
   - エラーメッセージの具体性検証テスト
2. The test-implementation shall **Runtime E2Eテスト**を作成し、以下の実行時動作を検証する：
   - **シーン辞書の前方一致検索とランダム選択**（finalize_scene()後の実際の動作）
   - **単語辞書のランダム選択と置換**（実行時の単語解決）
   - **アクター単語辞書のスコープ解決**（アクター別の単語置換）
   - Pastaスクリプト→トランスパイル→Lua実行→シーン/単語選択の完全フロー
3. When 新規テストを追加した場合, the test-implementation shall TEST_COVERAGE.mdのマッピングを更新する
4. The test-implementation shall SOUL.md Section 5.6（Phase 0完了基準）の未達成項目に貢献する：
   - pasta_shiori 100%パス（該当する場合）
   - 未テスト領域の特定と受容判断
5. Where テスト実装が困難な領域がある場合, the test-implementation shall 「受容理由」をドキュメント化し、Phase 1以降への延期を判断する

---

### Requirement 6: 現行Phase 0課題との連携

**Objective:** As a pasta開発者, I want ソウルドキュメントがPhase 0再構築の指針となる構成, so that 設計見直し作業が効率化される

#### Acceptance Criteria

1. The soul-document shall Phase 0の主要課題（DSL文法曖昧性、トランスパイル品質、シーンテーブル設計）に対する「あるべき姿」を定義する
2. The test-suite shall Phase 0課題の解決を検証するリグレッションテストを含む
3. While Phase 0再構築が進行中の間, the soul-document shall 「現状」と「目標」の差分を明示する
4. The soul-document shall 過去完了仕様（31件）の知見を設計原則として反映する
