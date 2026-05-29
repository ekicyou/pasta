# Roadmap

## 概要
SSPのプロパティシステムへのアクセスをpastaゴーストから可能にする拡張。プロパティの読み書きにはSHIORIプロトコルを介した非同期通信が必要であり、特に読み取り（GET）ではトーク合成中のyield/resume基盤の拡張が核心となる。

段階的に、書き込み（簡単）→ 汎用非同期通信基盤 + 読み取り（複雑）→ DSL統合（機械的）の順で進める。

## アプローチ決定
- **採用**: インクリメンタルLayered — 簡単なSETを先に実装し、GETは汎用的な「トーク合成中のSHIORI非同期通信」基盤として設計
- **理由**: コミットの粒度を小さく保ち、SET単体でも即座に有用。GETの非同期基盤はプロパティ以外の `\![get,...]` パターンにも再利用可能
- **却下**: 
  - 2-Spec一括（SET/GET同時実装）— コミットが乱れるリスク
  - DSLファーストのみ — Lua API基盤なしにDSL構文を設計するのは困難

## スコープ
- **対象**: SSPプロパティシステムの全カテゴリ（system, currentghost, ghostlist, activeghostlist, balloonlist, pluginlist, history, rateofuselist）への汎用的な読み書きアクセス
- **対象外**: 
  - `%property[name]` 環境変数展開（`get_property` が上位互換）
  - 個別プロパティの型安全ラッパー（汎用文字列APIで対応）
  - プロパティ値のバリデーション（SSP側の責任）

## 制約
- SHIORIプロトコル 3.0 に準拠
- 既存のyield/resume基盤（`STORE.co_scene`、`resume_until_valid`）との互換性を維持
- LuaJIT 2.1コルーチンモデルの範囲内で実装

## 境界戦略
- **分割理由**: SET（さくらスクリプトタグ発行のみ、同期的）とGET（SHIORI非同期通信、yield/resume拡張）は実装複雑度が大きく異なる。GETの基盤は「トーク中の非同期SHIORI通信」という汎用パターンとして設計し、プロパティ以外でも再利用可能にする
- **共有接点**: 両specとも `act` オブジェクトにメソッドを追加。Spec 2はSpec 1のset_propertyと対になるget_propertyを提供

## Specs (dependency order)
- [x] property-write-helpers -- `act:set_property(name, value)` によるプロパティ書き込み。Dependencies: none
- [x] shiori-event-test-framework -- SHIORIイベントフロー試験基盤（Luaモックライブラリ + X-Pasta-Time時刻注入 + ShioriResponse検証）。Dependencies: none
- [x] shiori-async-talk -- トーク合成中のSHIORI非同期通信基盤 + `act:get_property(name)`。Dependencies: property-write-helpers, shiori-event-test-framework

## Phase 2: DSL統合
- [x] property-dsl-extension -- `＄％` スコープ修飾子によるプロパティアクセスDSL構文（＄％prop.path＝value / ＄var＝＄％prop.path）。既存Lua APIにトランスパイル。Dependencies: property-write-helpers, shiori-async-talk

## Phase 3: 脆弱性監査・コード簡素化

全クレートを対象に、同一仕様（外部振る舞い不変）のまま、脆弱性回避とコード量削減を実施する。
調査対象: メモリ安全性、入力検証、FFI境界、依存クレートサプライチェーン、デッドコード除去、冗長表現削減、アルゴリズム改善。

### Wave 1（全並行・クレート内完結）
- [ ] audit-pasta-core -- レジストリ層の脆弱性監査・コード簡素化（~600行）。Dependencies: none
- [x] audit-pasta-dsl -- DSLパーサー層の脆弱性監査・コード簡素化（~2500行）。Dependencies: none
- [ ] audit-pasta-lua -- Luaトランスパイラ/ランタイムの脆弱性監査・コード簡素化（~8000行、最大規模）。Dependencies: none
- [ ] audit-pasta-shiori -- SHIORI/FFI層の脆弱性監査・unsafe安全性検証（~1500行）。Dependencies: none
- [ ] audit-pasta-check -- CLIツールの脆弱性監査・コード簡素化（~500行）。Dependencies: none
- [ ] audit-pasta-lsp -- LSPラッパーの脆弱性監査・コード簡素化（~400行）。Dependencies: none
- [ ] audit-pasta-sample-ghost -- サンプルゴーストの脆弱性監査・コード簡素化（~300行）。Dependencies: none

### Wave 2（横断的・Wave 1完了後）
- [ ] audit-dependency-supply-chain -- 外部依存クレートのセキュリティ・ライセンス・バージョン監査。Dependencies: Wave 1全spec
- [ ] audit-workspace-patterns -- クレート横断エラーハンドリング統一・共通パターン抽出。Dependencies: Wave 1全spec
