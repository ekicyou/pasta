# Pasta Steering Documentation

このディレクトリには、Pastaプロジェクトの開発指針・規約・現状をまとめたステアリングドキュメントが格納されています。

---

## ステアリングファイル一覧

### 0. [status.md](./status.md) - プロジェクト状態レポート 🔴
**現在のPhase・課題・優先タスク**

- **現在地**: Phase 0（一次設計再構築中）⚠️
- **重大な問題**: 過去の「完了」仕様は実装品質不十分、大規模差し戻し中
- **最優先課題**: Yield伝搬問題（Call/Jump文動作不全）🔴
- Phase 1移行条件、保留中作業、行動指針

**読むべきタイミング**: 🔴 **最初に必ず読む** - プロジェクトの現状を正確に把握するため

---

### 1. [product.md](./product.md) - プロダクトステアリング
**プロジェクトのビジョンと目標**

- プロジェクト概要: Pastaスクリプトエンジンの全体像
- ビジョン: 「パスタのように絡み合う会話の記録」
- 設計思想: 前方一致ランダムジャンプ、yield型エンジン、runeトランスパイル
- コアバリュー: 日本語フレンドリー、UNICODE識別子
- ターゲットユーザー: デスクトップマスコット制作者、ゲームエンジン利用者
- 機能優先順位: ⚠️ Phase 0進行中（一次設計再構築）、基盤未確立

**読むべきタイミング**: プロジェクト全体の方向性を確認したいとき

---

### 2. [tech.md](./tech.md) - 技術ステアリング
**技術スタックとアーキテクチャ原則**

- 技術スタック: Rust 2024, Rune 0.14, Pest 2.8, 主要依存関係
- アーキテクチャ原則:
  - 5層レイヤー分離（Parser, Transpiler, Runtime, Engine, IR）
  - UI層独立性（タイミング・バッファリング・レンダリング制御なし）
  - 宣言的コントロールフロー
  - Yield型実行モデル
  - 2パストランスパイル
- コーディング規約: ファイル命名、識別子、エラーハンドリング、テスト戦略
- 品質基準: カバレッジ、パフォーマンス、セキュリティ
- 依存関係管理: バージョン戦略、ライセンス（MIT OR Apache-2.0）
- デプロイメント: ビルド設定、CI/CD計画

**読むべきタイミング**: 新機能実装前、設計判断時

---

### 3. [structure.md](./structure.md) - プロジェクト構造ステアリング
**ディレクトリ構成とモジュール設計**

- ディレクトリ構造: `src/`, `tests/`, `.kiro/specs/` の詳細構成
- ファイル命名規則: ソース・テスト・文法ファイル
- モジュール構成: レイヤー依存関係、公開API
- テスト構成: 38テストファイルのカテゴリ分類
- ドキュメント構成: README, GRAMMAR, AGENTS, Kiro仕様

**読むべきタイミング**: ファイル配置を決めるとき、モジュール依存を確認するとき

---

### 4. [domain.md](./domain.md) - ドメイン知識ステアリング
**Pastaエンジンのコア概念と実装知見**

- ラベル: グローバル/ローカル、重複ラベル、前方一致検索
- トランスパイル: 2パス方式、モジュール構造化、アクター変数
- Yield伝搬問題: Runeの制約と解決方針
- 変数管理: 3層スコープ（ローカル/グローバル/システム）
- 制御フロー: Call/Jump文、宣言的アプローチ
- IR: ScriptEventイベント型、マーカー型イベント
- さくらスクリプトエスケープ: 透過的処理
- イベントハンドリング: OnXXXラベル規約
- パフォーマンス最適化: キャッシュ、Radix Trie
- areka統合: 分離原則、32子仕様

**読むべきタイミング**: ドメインロジック実装時、アーキテクチャ理解時

---

### 5. [completed-specs.md](./completed-specs.md) - 「完了」扱い仕様一覧 ⚠️
**過去に完了扱いされた仕様（実装品質不十分）**

- ⚠️ 「完了」仕様11件の記録（**再評価必要**）:
  - `pasta-engine-independence`: UI独立性（実装不完全）
  - `pasta-transpiler-pass2-output`: 2パス出力（品質問題）
  - `pasta-declarative-control-flow`: 宣言的制御フロー（動作不全）
  - 他8件
- 再評価必要度の分類（🔴高/🟡中/🟢低）
- **注意**: 手続き上の完了であり、実装品質は保証されていない

**読むべきタイミング**: 過去実装を参照する際、**要件との乖離を意識**しながら

---

### 6. [active-specs.md](./active-specs.md) - 進行中仕様一覧
**現在の開発状況と優先順位（Phase 0完了後に着手）**

- 進行中仕様9件（**Phase 0完了まで保留**）:
  - **P0**: `pasta-yield-propagation` 🔴 Phase 0で対応中
  - **P0**: `pasta-local-rune-calls` - 基盤安定後
  - **P1**: `pasta-word-definition-dsl` - 基盤安定後
  - 他6件
- ⚠️ 現状認識: Phase 0（一次設計再構築中）
- 推奨アクション: **新機能実装より既存問題解決を優先**
- Phase 1移行条件

**読むべきタイミング**: 次のタスクを決めるとき、進捗確認時（Phase 0優先を意識）

---

## クイックスタート

### 🔴 まず最初に（必須）
1. **[status.md](./status.md)**: **現在のプロジェクト状態を把握**
   - Phase 0（一次設計再構築中）であることを認識
   - 最優先課題（Yield伝搬問題）の確認
   - 新機能実装は保留中であることを理解

### Phase 0: 既存問題解決時
1. **[status.md](./status.md)**: Phase 0タスク一覧確認
2. **[completed-specs.md](./completed-specs.md)**: 過去の「完了」仕様を参照（⚠️ 品質不十分）
3. **[domain.md](./domain.md)**: ドメイン概念と実装知見確認
4. **[tech.md](./tech.md)**: アーキテクチャ原則・品質基準確認
5. **要件定義に立ち戻る**: `.kiro/specs/{spec-name}/requirements.md`を精読

### Phase 1以降: 新機能開発（Phase 0完了後のみ）
1. **[status.md](./status.md)**: Phase 1移行条件を確認
2. **[product.md](./product.md)**: ビジョン・目標との整合性確認
3. **[active-specs.md](./active-specs.md)**: 重複作業がないか確認、優先順位確認
4. **[domain.md](./domain.md)**: ドメイン概念理解
5. **[tech.md](./tech.md)**: アーキテクチャ原則・コーディング規約確認
6. **[structure.md](./structure.md)**: ファイル配置・モジュール依存確認

### 仕様レビュー時
1. **[status.md](./status.md)**: 現在のPhaseに適したレビューか確認
2. **[product.md](./product.md)**: コアバリューとの整合性
3. **[tech.md](./tech.md)**: アーキテクチャ原則違反がないか
4. **[domain.md](./domain.md)**: 既存実装パターンとの一貫性
5. **要件との整合性**: requirements.mdとの乖離がないか厳密にチェック

---

## ステアリング更新方針

### 更新タイミング
- 新しい仕様が完了したとき → `completed-specs.md` 更新
- 仕様の状態が変わったとき → `active-specs.md` 更新
- 重要なドメイン知見が得られたとき → `domain.md` 更新
- アーキテクチャ原則が変わったとき → `tech.md` 更新
- プロジェクト目標が更新されたとき → `product.md` 更新
- ディレクトリ構造が大きく変わったとき → `structure.md` 更新

### 更新コマンド
```bash
# ステアリング全体を再解析して更新
/kiro-steering

# 特定のステアリングファイルを更新（手動編集）
# 該当ファイルを直接編集し、commitする
```

---

## 関連ドキュメント

### プロジェクトルート
- **[README.md](../../README.md)**: プロジェクト概要
- **[GRAMMAR.md](../../GRAMMAR.md)**: Pasta DSL文法リファレンス（600行超）
- **[AGENTS.md](../../AGENTS.md)**: AI開発支援情報

### Kiro仕様管理
- **[.kiro/specs/](../specs/)**: 全仕様の詳細（requirements, design, tasks, validation）
- **[.kiro/specs/completed/](../specs/completed/)**: 完了仕様11件のアーカイブ

### ソースコード
- **[src/lib.rs](../../src/lib.rs)**: クレートエントリーポイント、公開API
- **[tests/](../../tests/)**: 38テストファイル、フィクスチャ

---

## AI開発支援への統合

このステアリングディレクトリは、AI Development Life Cycle (AI-DLC) の一部として機能します。

### AI読み込み
AIエージェントは`.kiro/steering/`全体をプロジェクトメモリとして読み込み、開発時の判断基準として使用します。

### Kiro Spec-Driven Development連携
- `/kiro-steering`: ステアリング初期化・更新
- `/kiro-spec-init`: 新仕様作成時、ステアリング参照
- `/kiro-spec-design`: 設計生成時、アーキテクチャ原則適用
- `/kiro-spec-impl`: 実装時、コーディング規約適用
- `/kiro-spec-status`: 進捗確認時、active-specs.md参照

---

**最終更新**: 2025-12-16  
**管理者**: Kiro AI-DLC System  
**プロジェクト**: pasta v0.1.0
