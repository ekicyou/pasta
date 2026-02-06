# Implementation Plan

## Task Overview

本仕様は**計画・組織仕様**であり、成果物は6つの子仕様の立ち上げ（`.kiro/specs/` 配下へのドキュメント生成）である。実際のコード実装は各子仕様で行うため、本タスクリストはドキュメント作成・検証作業のみを対象とする。

---

## Tasks

### Phase A: 子仕様定義準備

- [x] 1. (P) 子仕様命名規則の確定
  - `alpha<連番2桁>-<機能名>` パターンの例示（alpha01-shiori-alpha-events 等）
  - 3カテゴリ（SHIORI基盤、サンプルゴースト、リリース準備）の定義
  - 規模見積もり基準（S/M/L）のガイドライン作成
  - _Requirements: 1.1, 1.2, 1.4_

- [x] 2. (P) 依存関係グラフの文書化
  - Phase A/B/C の定義と各 Phase の完了条件 (DoD)
  - 子仕様間の依存関係マッピング（alpha02 → alpha01, alpha04 → alpha01/02/03, alpha06 → all 等）
  - 並行作業可能な子仕様の明示（alpha05 は独立進行可能等）
  - Mermaid 依存グラフの作成
  - _Requirements: 1.3, 5.1, 5.2_

- [x] 3. (P) 既存仕様との整合性チェック
  - `lua55-manual-consistency` との関係整理（並行可能）
  - `ukagaka-desktop-mascot` との関係整理（サブセット的先行実装）
  - 完了済み仕様からの再利用可能資産特定（EVENT/REG/RES 等）
  - _Requirements: 6.1, 6.2, 6.3_

---

### Phase B: 子仕様立ち上げコマンド準備

- [x] 4. alpha01-shiori-alpha-events の仕様初期化コマンド作成
  - `/kiro-spec-init` コマンド文字列の作成（description 含む）
  - SHIORI EVENT 7種（OnFirstBoot, OnBoot, OnClose, OnGhostChanged, OnSecondChange, OnMinuteChange, OnMouseDoubleClick）のスコープ明示
  - 既存 REG/EVENT 機構の活用方針を description に含める
  - _Requirements: 2.1, 2.2, 2.6, 2.7, 5.3_

- [x] 5. alpha02-virtual-event-dispatcher の仕様初期化コマンド作成
  - `/kiro-spec-init` コマンド文字列の作成
  - OnTalk/OnHour 仮想イベント発行機構のスコープ明示
  - 状態管理（ctx.save.virtual_event.*）、pasta.toml 設定読み込み、req.date 時刻判定を description に含める
  - alpha01 への依存を明示
  - _Requirements: 2.3, 5.3_

- [x] 6. (P) alpha03-shiori-act-sakura の仕様初期化コマンド作成
  - `/kiro-spec-init` コマンド文字列の作成
  - pasta.act 継承、act:talk()/surface()/wait() インターフェース提供を description に含める
  - さくらスクリプトタグ生成（\0, \s[0], \e 等）のスコープ明示
  - _Requirements: 2.4, 2.5, 5.3_

- [x] 7. alpha04-sample-ghost の仕様初期化コマンド作成
  - `/kiro-spec-init` コマンド文字列の作成
  - 最低限機能（起動挨拶、ダブルクリック反応、終了挨拶、ランダムトーク）を description に含める
  - シェル素材仕様（幅96～128 x 高さ256、透過PNG、女の子 surface0-8 / 男の子 surface10-18）を明示
  - ディレクトリ構成テンプレート（pasta.toml, dic/*.pasta, shell/master/ 等）のスコープを含める
  - alpha01/02/03 への依存を明示
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 5.3_

- [x] 8. (P) alpha05-build-ci の仕様初期化コマンド作成
  - `/kiro-spec-init` コマンド文字列の作成
  - GitHub Actions による x86/x64 DLL ビルド CI のスコープ明示
  - i686-pc-windows-msvc / x86_64-pc-windows-msvc 両ターゲット、cargo test 実行を description に含める
  - _Requirements: 4.1, 5.3_

- [x] 9. alpha06-release-packaging の仕様初期化コマンド作成
  - `/kiro-spec-init` コマンド文字列の作成
  - x86 配布パッケージ（pasta.dll + サンプルゴースト同梱 ZIP）のスコープ明示
  - GitHub Releases タグ駆動アップロード、README ドキュメント（インストール手順・動作確認）を description に含める
  - バージョン体系 `0.1.0-alpha.1` の明示
  - alpha01-05 全依存を明示
  - _Requirements: 4.1, 4.2, 4.3, 5.3_

---

### Phase C: 子仕様立ち上げ実行

- [x] 10. Phase A 子仕様の立ち上げ
- [x] 10.1 alpha01-shiori-alpha-events 立ち上げ
  - Task 4 で作成したコマンドを実行
  - `.kiro/specs/alpha01-shiori-alpha-events/spec.json` 生成確認
  - _Requirements: 2.1, 2.2, 2.6, 2.7_

- [x] 10.2 alpha02-virtual-event-dispatcher 立ち上げ
  - Task 5 で作成したコマンドを実行
  - spec.json の dependencies に alpha01 が記載されていることを確認
  - _Requirements: 2.3_

- [x] 10.3 (P) alpha03-shiori-act-sakura 立ち上げ
  - Task 6 で作成したコマンドを実行
  - `.kiro/specs/alpha03-shiori-act-sakura/spec.json` 生成確認
  - _Requirements: 2.4, 2.5_

- [x] 11. (P) Phase B 子仕様の立ち上げ
  - Task 7 で作成したコマンドを実行（alpha04-sample-ghost）
  - spec.json の dependencies に alpha01/02/03 が記載されていることを確認
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 12. Phase C 子仕様の立ち上げ
- [x] 12.1 (P) alpha05-build-ci 立ち上げ
  - Task 8 で作成したコマンドを実行
  - `.kiro/specs/alpha05-build-ci/spec.json` 生成確認
  - _Requirements: 4.1_

- [x] 12.2 alpha06-release-packaging 立ち上げ
  - Task 9 で作成したコマンドを実行
  - spec.json の dependencies に alpha01-05 が記載されていることを確認
  - _Requirements: 4.2, 4.3_

---

### Phase D: 検証・統合

- [x] 13. 子仕様一覧の生成と検証
  - 6つの子仕様ディレクトリ（alpha01-06）が `.kiro/specs/` に存在することを確認
  - 各 spec.json の feature_name, language, phase, dependencies フィールドを検証
  - 依存グラフが設計通り（Phase A/B/C）であることを確認
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [x] 14. ロードマップドキュメントの最終化
  - Phase A/B/C の定義と完了条件 (DoD) を最終確認
  - 各 Phase の並行作業可能性（alpha01/03 並行、alpha05 独立）を明記
  - `/kiro-spec-init` コマンド一覧の完全性を確認
  - _Requirements: 5.1, 5.2, 5.3_

- [x] 15. ドキュメント整合性の確認と更新
  - SOUL.md - コアバリュー・設計原則との整合性確認
  - SPECIFICATION.md - 言語仕様の更新（該当する場合）
  - GRAMMAR.md - 文法リファレンスの同期（該当する場合）
  - TEST_COVERAGE.md - 新規テストのマッピング追加
  - クレートREADME - API変更の反映（該当する場合）
  - steering/* - 該当領域のステアリング更新
  - _Note: 本仕様は計画・組織仕様のため、コード変更なし。ドキュメント確認のみ実施。_

---

## Requirements Coverage Summary

| Requirement | Task(s) |
|-------------|---------|
| 1.1 | 1, 13 |
| 1.2 | 1, 13 |
| 1.3 | 2, 13 |
| 1.4 | 1, 13 |
| 2.1 | 4, 10.1 |
| 2.2 | 4, 10.1 |
| 2.3 | 5, 10.2 |
| 2.4 | 6, 10.3 |
| 2.5 | 6, 10.3 |
| 2.6 | 4, 10.1 |
| 2.7 | 4, 10.1 |
| 3.1 | 7, 11 |
| 3.2 | 7, 11 |
| 3.3 | 7, 11 |
| 3.4 | 7, 11 |
| 4.1 | 8, 9, 12.1 |
| 4.2 | 9, 12.2 |
| 4.3 | 9, 12.2 |
| 5.1 | 2, 14 |
| 5.2 | 2, 14 |
| 5.3 | 4, 5, 6, 7, 8, 9, 14 |
| 6.1 | 3 |
| 6.2 | 3 |
| 6.3 | 3 |

**全24要件をカバー完了**
