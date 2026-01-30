# Requirements Document

## Introduction

本仕様は **pasta アルファバージョンリリース計画**の親仕様である。SHIORI.DLLの完成、サンプルゴーストの作りこみ、最低限のSHIORI EVENTサポートを目的とした複数の子仕様 `alpha01-XXXX` ～ `alpha99-XXXX` シリーズを組織的に立ち上げる計画を定義する。

**本仕様の成果物は子仕様の立ち上げであり、実際の実装コードは生成しない**。

### 背景

- **現在地**: Phase 2（コア機能拡張）- 基盤確立済み
- **pasta_shiori**: SHIORI/3.0 プロトコル実装済み、EVENT モジュール動作確認済み
- **pasta_lua**: Lua 5.5 トランスパイラ・ランタイム動作中
- **テスト状況**: 340+ テスト全パス

### アルファリリース目標

「伺か」ベースウェアで最低限動作するゴーストを完成させ、開発者プレビュー版としてリリースすること。

---

## Requirements

### Requirement 1: 子仕様体系の設計

**Objective:** As a 開発リーダー, I want アルファリリースに必要な作業を子仕様として体系化したい, so that 開発の優先順位と依存関係を明確化できる

#### Acceptance Criteria

1. The alpha-release-planning shall 子仕様命名規則 `alpha<連番2桁>-<機能名>` を定義する（例: `alpha01-shiori-core-events`）
2. The alpha-release-planning shall 子仕様を以下の3カテゴリに分類する:
   - **SHIORI基盤**: SHIORI EVENTのサポート拡充
   - **サンプルゴースト**: 最低限動作するゴーストの実装
   - **リリース準備**: ビルド・配布・ドキュメント整備
3. The alpha-release-planning shall 各子仕様間の依存関係を明示する
4. The alpha-release-planning shall 各子仕様の推定規模（S/M/L）を見積もる

---

### Requirement 2: SHIORI基盤 子仕様群の定義

**Objective:** As a ゴースト開発者, I want 必要最低限のSHIORI EVENTが動作することで, so that ベースウェア上でゴーストとしての基本動作ができる

#### Acceptance Criteria

1. When alpha-release-planning を承認した際, the alpha-release-planning shall 以下のSHIORI EVENTカテゴリをサポート対象として子仕様化する:
   - **ライフサイクル**: OnFirstBoot, OnBoot, OnClose, OnGhostChanged
   - **時刻イベント**: OnMinuteChange（アルファ版では分単位のみ）
   - **ユーザー操作**: OnMouseDoubleClick（キャラクターへのダブルクリック）
2. If 既にイベントハンドラ機構が実装済みの場合, then the 子仕様 shall 既存の `REG` / `EVENT` モジュール機構を活用する設計とする
3. The alpha-release-planning shall 各SHIORI EVENTに対して最低限のスタブ応答仕様を定義する子仕様を立ち上げる
4. Where Referenceパラメータが必要なイベントの場合, the 子仕様 shall Reference0〜Reference7の解析・利用方法を仕様に含める

---

### Requirement 3: サンプルゴースト 子仕様群の定義

**Objective:** As a エンドユーザー, I want 動作するサンプルゴーストがほしい, so that pastaエンジンの動作を体験できる

#### Acceptance Criteria

1. The alpha-release-planning shall サンプルゴースト用の子仕様を立ち上げる
2. The サンプルゴースト子仕様 shall 以下の最低限機能を仕様に含める:
   - 起動時の挨拶トーク（OnFirstBoot / OnBoot）
   - ダブルクリック時の反応（OnMouseDoubleClick）
   - 終了時の挨拶（OnClose）
3. The サンプルゴースト子仕様 shall ゴーストディレクトリ構成（`ghost/master/` 配下）のテンプレートを定義する:
   - `pasta.toml`: 設定ファイル
   - `dic/*.pasta`: Pasta DSLスクリプト
   - `scripts/pasta/shiori/`: Luaエントリーポイント
4. The alpha-release-planning shall シェル（見た目）については記号的なシンプルシェル（男の子・女の子のピクトグラム風PNG画像）を独自作成する方針とする

---

### Requirement 4: リリース準備 子仕様群の定義

**Objective:** As a 配布担当者, I want アルファ版をパッケージ配布できるようにしたい, so that テスターに配布できる

#### Acceptance Criteria

1. The alpha-release-planning shall 以下のリリース準備タスクを子仕様化する:
   - **ビルドCI**: Windows x86/x64 両アーキテクチャでの `pasta.dll` ビルド確認（GitHub Actions）
   - **配布パッケージ**: x86（32bit）版 `pasta.dll` + サンプルゴースト同梱の配布アーカイブ作成
   - **READMEドキュメント**: インストール手順・動作確認方法
2. If GitHub Releases を配布チャネルとする場合, then the 子仕様 shall リリースワークフロー（タグ → ビルド → アップロード）を仕様に含める
3. The alpha-release-planning shall バージョン番号体系（例: `0.1.0-alpha.1`）を決定する

---

### Requirement 5: 子仕様ロードマップの作成

**Objective:** As a プロジェクトマネージャー, I want 子仕様の実行順序と依存関係を可視化したい, so that 並行作業と順次作業を区別できる

#### Acceptance Criteria

1. The alpha-release-planning shall 子仕様を実行順にグループ化する:
   - **Phase A**: SHIORI基盤（依存元、最優先）
   - **Phase B**: サンプルゴースト（SHIORI基盤完了後）
   - **Phase C**: リリース準備（Phase A/B完了後）
2. The alpha-release-planning shall 各Phaseの完了条件（Definition of Done）を定義する
3. While 子仕様ロードマップ策定中, the alpha-release-planning shall 各子仕様の `/kiro-spec-init` コマンド例を提示する

---

### Requirement 6: 既存仕様との整合性確認

**Objective:** As a アーキテクト, I want 既存の進行中仕様と矛盾しないことを確認したい, so that 作業の重複や競合を避けられる

#### Acceptance Criteria

1. The alpha-release-planning shall 以下の既存仕様との関係を整理する:
   - `lua55-manual-consistency`: ドキュメント整備（並行可能）
   - `ukagaka-desktop-mascot`: メタ仕様（本仕様がサブセット的に先行）
2. If 既存仕様と機能的に重複する子仕様を立ち上げる場合, then the alpha-release-planning shall 重複部分の統合または分離方針を明記する
3. The alpha-release-planning shall 完了済み仕様（`.kiro/specs/completed/`）の成果を子仕様で再利用可能な箇所を特定する

---

## Out of Scope

本仕様では以下を **対象外** とする:

- 実際のRust/Luaコードの実装
- SHIORI EVENTの完全なサポート（アルファ版では必要最低限のみ）
- 複雑な会話ロジック（ランダムトークなど）
- SAORI/MAKOTO等の外部連携
- arekaへの投入（Phase 4相当、本仕様の後続）

---

## Glossary

| 用語 | 説明 |
|------|------|
| SHIORI | 伺かゴーストの対話エンジンインターフェース規格 |
| SHIORI EVENT | ベースウェアからSHIORI DLLに送信されるイベント（OnBoot等） |
| ベースウェア | 伺かを動作させるホストアプリケーション（SSP、CROW等） |
| REG | pasta.shiori.event.register モジュール（イベントハンドラ登録） |
| EVENT | pasta.shiori.event モジュール（イベントディスパッチ） |
