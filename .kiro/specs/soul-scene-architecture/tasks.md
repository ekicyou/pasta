# Implementation Plan

## Task Overview

本仕様は、SOUL.mdへの新章追加というドキュメント改訂作業です。コード変更は一切ありません。

**総タスク数**: 6メジャータスク、13サブタスク
**推定工数**: 約6-9時間
**並列実行**: 一部タスクが並列実行可能（`(P)`マーク付き）

## Implementation Tasks

### Phase 1: 事前準備と検証

- [x] 1. 既存章参照の影響範囲調査
- [x] 1.1 (P) workspace全体のgrep検索実行
  - 検索パターン: `SOUL.md.*[第5章|Chapter 5|5\.|§5|Section 5]`
  - `.github/`, `.kiro/`, `README.md`, `AGENTS.md`, `crates/*/README.md`等を対象
  - 発見された参照箇所をリスト化
  - **結果**: TEST_COVERAGE.mdとscope-evolution.mdに「Section 5.6」への参照を発見
  - _Requirements: 7_

- [x] 1.2 (P) SPECIFICATION.mdとの用語整合性確認
  - SPECIFICATION.mdから用語定義を抽出（シーン、アクター、アクション、act、talk、call、word、Jump/Call）
  - 新章で使用予定の用語リストと照合
  - 用語の不一致や曖昧性がないことを確認
  - **結果**: すべての用語がSPECIFICATION.mdと一致
  - _Requirements: 7_

### Phase 2: SOUL.md新章の執筆

- [x] 2. 導入セクション「5.0 シーン実行アーキテクチャとは」の執筆
- [x] 2.1 映画撮影メタファーの説明
  - 4要素（シーン＝台本、アクター＝役者、アクション＝演技、act＝カメラ）の関係を説明
  - 各要素の役割と相互作用を記述
  - 映画撮影の流れとpastaの実行フローの対応を明示
  - design.md § Content Specification 5.0の表を参考に執筆
  - **結果**: セクション5.1として完了
  - _Requirements: 1_

- [x] 3. シーン関数の構造説明セクション「5.1 シーン関数の構造」の執筆
- [x] 3.1 DSLからLua関数への変換概要
  - シーン定義がLua関数にトランスパイルされる流れを説明
  - 関数シグネチャ `function SCENE.__シーン名__(act, ...)` の意味を解説
  - `act:init_scene(SCENE)` による初期化とsave/var取得を説明
  - **結果**: セクション5.2として完了
  - _Requirements: 2, 3_

- [x] 3.2 coroutine/yieldパターンの概要説明
  - Luaコルーチンとyieldによる出力ストリームの実現を説明
  - 継続可能な実行モデルの利点を記述
  - sample.generated.luaからコード例を引用
  - **結果**: セクション5.2内で完了
  - _Requirements: 3_

- [x] 4. アクター発話セクション「5.2 アクター発話の実行モデル」の執筆
- [x] 4.1 act.アクター:talk()による発話記録
  - `act.アクター:talk(...)` がactオブジェクトに出力を記録する仕組みを説明
  - アクター選択、サーフェス切替、テキスト出力のシーケンスを記述
  - 表情変更（`\s[表情ID]`）の出力フローを含める
  - sample.generated.luaの発話パターンをコード例として引用
  - **結果**: セクション5.3として完了
  - _Requirements: 4_

- [x] 4.2 (P) 単語展開による動的テキスト生成
  - `act.アクター:word("単語名")` による実行時単語解決を説明
  - ランダム選択メカニズムを記述
  - スコープ解決順序（アクター → シーン → グローバル）を明記
  - **結果**: セクション5.3内で完了
  - _Requirements: 6_

- [x] 5. シーン呼び出しセクション「5.3 シーン呼び出しと制御フロー」の執筆
- [x] 5.1 Callによるサブルーチン呼び出し
  - `act:call(...)` による復帰を伴うシーン呼び出しを説明
  - 実行スタックの動作を記述
  - sample.generated.luaからCallのコード例を引用
  - **結果**: セクション5.4として完了
  - _Requirements: 5_

- [x] 5.2 (P) Jumpによる末尾呼び出し最適化
  - `return act:call(...)` による末尾呼び出しの仕組みを説明
  - スタック消費なしでシーン遷移する利点を記述
  - 長時間シナリオでのスタックオーバーフロー防止を明記
  - **結果**: セクション5.4として完了
  - _Requirements: 5_

### Phase 3: 既存章の更新と整合性確保

- [x] 6. 既存章の番号繰り下げと整合性確保
- [x] 6.1 章番号の更新
  - 既存の「5. Phase 0」を「6. Phase 0」に変更
  - 既存の「6. 開発哲学」を「7. 開発哲学」に変更
  - 既存の「7. ターゲットユーザー」を「8. ターゲットユーザー」に変更
  - 既存の「8. ロードマップ」を「9. ロードマップ」に変更
  - design.md § Content Specification 5.5の表を参照
  - **結果**: 完了
  - _Requirements: 2_

- [x] 6.2 内部リンクと目次の更新
  - SOUL.md内の章参照リンクを更新（該当する場合）
  - 目次（Table of Contents）の章番号を更新（存在する場合）
  - **結果**: SOUL.md内に内部リンクなし、目次なし
  - _Requirements: 2, 7_

- [x] 6.3 外部参照箇所の更新
  - Task 1.1で発見された外部参照箇所を更新
  - README.md、AGENTS.md、steering/*、crates/*/README.md等の該当箇所を修正
  - 更新がない場合はその旨を記録
  - **結果**: TEST_COVERAGE.mdとscope-evolution.mdの「Section 5.6」→「Section 6.6」に更新
  - _Requirements: 7_

### Phase 4: 最終検証とドキュメント整合性確認

- [x] 7. 品質検証とレビュー
- [x] 7.1 Markdown文法チェック
  - Markdown linterでエラーがないことを確認
  - コードブロックの閉じタグ、リンク形式、見出しレベルの一貫性を検証
  - **結果**: 新章にlintエラーなし（既存部分のエラーはスコープ外）
  - _Requirements: 7_

- [x] 7.2 用語一貫性の最終確認
  - SPECIFICATION.mdとの用語一貫性を再確認
  - Call/Jump表記の統一を検証
  - アーキテクチャ用語（シーン、アクター、act等）の使用一貫性を確認
  - **結果**: 全用語がSPECIFICATION.mdと一致
  - _Requirements: 7_

- [x] 7.3 映画撮影メタファーの完全性検証
  - 4要素（シーン、アクター、アクション、act）がすべて説明されていることを確認
  - メタファーの一貫性を全章を通じて検証
  - **結果**: 4要素すべてが文書化・一貫して使用されていることを確認
  - _Requirements: 1_

## Requirements Coverage Summary

| Requirement | Covered by Tasks |
|-------------|------------------|
| 1 | 2.1, 7.3 |
| 2 | 3.1, 6.1, 6.2 |
| 3 | 3.1, 3.2 |
| 4 | 4.1 |
| 5 | 5.1, 5.2 |
| 6 | 4.2 |
| 7 | 1.1, 1.2, 6.2, 6.3, 7.1, 7.2, 7.3 |

**全要件カバー済み**: ✅

## Parallel Execution Notes

- **Phase 1**: Task 1.1と1.2は完全に独立しており、並列実行可能
- **Phase 2**: Task 4.2（単語展開）とTask 5.2（Jump）は並列実行可能（それぞれ独立したセクション）
- **Phase 3以降**: 順次実行推奨（章番号更新の整合性確保のため）

## Implementation Order Recommendation

1. Phase 1を並列実行（Task 1.1, 1.2）
2. Phase 2を順次実行（Task 2.1 → 3.1 → 3.2 → 4.1）、ただし4.2と5.2は並列可
3. Phase 3を順次実行（Task 6.1 → 6.2 → 6.3）
4. Phase 4を順次実行（Task 7.1 → 7.2 → 7.3）
