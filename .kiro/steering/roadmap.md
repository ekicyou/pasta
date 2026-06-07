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
- [x] audit-pasta-core -- レジストリ層の脆弱性監査・コード簡素化（~600行）。Dependencies: none
- [x] audit-pasta-dsl -- DSLパーサー層の脆弱性監査・コード簡素化（~2500行）。Dependencies: none
- [x] audit-pasta-lua -- Luaトランスパイラ/ランタイムの脆弱性監査・コード簡素化（~8000行、最大規模）。Dependencies: none
- [x] audit-pasta-shiori -- SHIORI/FFI層の脆弱性監査・unsafe安全性検証（~1500行）。Dependencies: none
- [x] audit-pasta-check -- CLIツールの脆弱性監査・コード簡素化（~500行）。Dependencies: none
- [x] audit-pasta-lsp -- LSPラッパーの脆弱性監査・コード簡素化（~400行）。Dependencies: none
- [x] audit-pasta-sample-ghost -- サンプルゴーストの脆弱性監査・コード簡素化（~300行）。Dependencies: none

### Wave 2（横断的・Wave 1完了後）
- [x] audit-dependency-supply-chain -- 外部依存クレートのセキュリティ・ライセンス・バージョン監査。Dependencies: Wave 1全spec
- [x] audit-workspace-patterns -- クレート横断エラーハンドリング統一・共通パターン抽出。Dependencies: Wave 1全spec

## Phase 4: 利用者マニュアルサイト

pasta ゴースト作者向けの利用者マニュアルを、mdBook で**サーバー不要の静的 HTML+JS サイト**として構築する。
既存の `doc/spec/` Markdown 資産を流用し、文法・Lua API・入門チュートリアルを検索可能な単一サイトに統合する。

### アプローチ決定（Phase 4）
- **採用**: mdBook（Rust 製・cargo bin に導入済み・追加エコシステム依存ゼロ）。`mdbook build` が静的 HTML+JS（クライアント側 elasticlunr 全文検索・`.nojekyll` 同梱）を出力 → GitHub Pages 等にサーバー不要で公開可能（実機検証済み: mdbook v0.5.3）
- **却下**: Sphinx（Python 依存・reST/MyST 設定が重くオーバースペック）／ VitePress・Docusaurus（node_modules ツリー・2 つ目のエコシステム持ち込み）

### Specs (dependency order)
- [x] pasta-user-manual -- mdBook ベースの利用者マニュアルサイト（Pasta DSL 文法 + Lua API/コーディング + 入門チュートリアル）。Dependencies: none
- [x] pasta-manual-syntax-highlight -- マニュアルの *.pasta コードブロックへ VSCode 同等のシンタックスハイライトを追加。VSCode TextMate 文法（SSOT）を build-time 再利用し hljs 互換クラスへ写像、出力は純静的。Dependencies: pasta-user-manual

### 将来仕様（Phase 4 派生・未着手）
- [ ] pasta-runtime-internals-doc -- pasta Lua ランタイムの内部設計・アーキテクチャ解説（2パストランスパイル / yield-resume コルーチン / シーン検索 / ローダ自己展開 / SHIORI 非同期基盤）。読者＝コントリビュータ・実装理解者（利用者マニュアルとは読者層が異なる別境界）。Dependencies: pasta-user-manual
  - 由来: pasta-user-manual の設計ディスカッションで「ランタイム内部設計は本仕様外・将来仕様」と決定（R5 は API 使用法に限定）

## Phase 5: VSCode Lua デバッグ連携

VSCode から pasta（最終的に .pasta ソースレベル）をステップ実行・ブレーク・変数監視できるデバッグ環境を構築する。
組込 LuaJIT（mlua vendored 静的リンク）に対し、**依存を最小化しトランスポート（ソケット）を Rust 側で提供する「プレーン実装」**を採る（luasocket 等の C モジュール .dll に依存しない）。デバッグ基盤は pasta_lua に内蔵し、SHIORI 以外の pasta ホストでも再利用可能にする。

### アプローチ決定（Phase 5）
- **採用**: Rust ホスト型 DAP バックエンド（LRDB 型）。`std::net::TcpListener` でトランスポート、`serde_json`（既存依存）で DAP 最小サブセットを手書き、`mlua::Lua::set_global_hook` ＋ `jit.off(true,true)` でフック。別スレッド I/O ↔ VM スレッドフックのチャネル分離。
- **理由**: 静的リンク LuaJIT は外部 C モジュール（luasocket/emmy_core/remotedebug）を `require` できない構造的制約があるため、トランスポートを Rust が握ることでこの問題を根本回避。依存最小・サンドボックス（`std_debug` 非露出）維持・将来の非 SHIORI ホスト再利用が同時に成立。構造が一致する実在前例（satoren/LRDB、actboy168/lua-debug の LuaJIT 対応）あり。
- **却下**: 
  - devCAT vscode-lua-debug 完成路線（同梱 vscode-debuggee.lua + luasocket）— C .dll ロード可否が静的リンクで不透明・依存が増える
  - lua-debug(actboy168) / EmmyLua への載せ替え — remotedebug.dll/emmy_core.dll の C モジュール依存が同じ壁
  - MobDebug + ZeroBrane — DAP 非準拠・luasocket 依存

### ゲート方針
- **実装仕様は検証仕様の GO 判定を前提とする**。「実装前に可否判断を完結」のため、検証仕様で唯一の本丸（jit.off ＋ set_global_hook が LuaJIT の動的生成シーンコルーチンでラインフックを撃つか／フック内ブロッキング停止・再開／フック内変数 inspect）を最小 PoC で実証してから実装へ進む。
- 検証仕様は開始時に専用ブランチを切り、検証コードは使い捨て/feature-gate とする。

### Specs (dependency order)
- [x] pasta-lua-debug-feasibility -- Rust ホスト型デバッグ方式の go/no-go を最小 PoC で確定（jit.off + set_global_hook の LuaJIT 実発火・フック内ブロッキング停止/再開・フック内変数 inspect）。**判定 = GO+（R1〜R4 全成立・2026-06-07）**。検証コードは feature `lua-debug-poc`（使い捨て・default 無効）。Dependencies: none
- [x] pasta-vscode-lua-debug -- Rust ホスト型 DAP デバッグバックエンド（std::net + serde_json）で **Lua レベルのデバッグ**（生成 .lua 上で BP/ステップ/変数 inspect・コルーチン inspect・VSCode attach）を本番化＋旧 luasocket 資産撤去＋PoC ハーネス除去（完了条件）。**`.pasta` ソースマップは実現可能性確定（調査＋薄い実証スライス＋設計シーム）まで**を担い、本番化は派生別仕様 pasta-source-map へ分割。Dependencies: pasta-lua-debug-feasibility（**= GO+ 達成済み**・着手可）。**完了 2026-06-08（全8タスク・DoD GO）。`.pasta` ソースマップ実現可能性 = 確定、本番化は pasta-source-map へ申し送り済み**

### 将来仕様（Phase 5 派生・未着手）
- [ ] pasta-source-map（仮称）-- `.pasta`↔生成 .lua ソースマップの**本番実装**（全 generate_* 網羅・本番マップ出力）と、**`.pasta` 座標でのブレークポイント／コールスタックの常時提示**。pasta-vscode-lua-debug が確定した実現可能性ノート・薄い実証スライス・設計シーム（code_gen 接合点／マップ受け渡し IF／DAP source 取り扱い口）を入力として消費し、`.pasta` ソースレベルのデバッグ体験（Phase 5 の最終目標）を完成させる。Dependencies: pasta-vscode-lua-debug
  - 由来: pasta-vscode-lua-debug のギャップ分析で「.pasta ソースマップ本番化は独立した最大級の塊（code_gen 全 generate_* 波及・双方向変換の正確性）」と判断。ユーザー決定（2026-06-07）により分割し、本仕様は Lua レベルのデバッグを出荷コア、.pasta ソースマップは実現可能性確定までを担うと確定
