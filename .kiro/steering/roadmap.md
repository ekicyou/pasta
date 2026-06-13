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
- [ ] manual-ssot-authority -- マニュアル全体の SSOT/権威化の再編。「mdBook に書く項目は mdBook を権威にする」方針を確立し、文法・Lua 含めた `doc/spec` との並行管理（drift-check 方式）を見直す。Dependencies: pasta-user-manual
  - 由来: pasta-manual-debugging の discovery（2026-06-08）でユーザーが「mdbook に書いてる項目は mdbook を権威にしたい／別仕様で権威化の整理をすべき」と指摘。本仕様外・別仕様として申し送り

### Phase 4 派生（デバッグ利用者ガイド）
- [x] pasta-manual-debugging -- VSCode Lua デバッグ（`.pasta` ソースレベルまで完全網羅）の利用者向けデバッグ章を mdBook マニュアルに追加。有効化／`launch.json`／attach／BP・ステップ・変数 inspect・提示モード切替／構造的制約と緩和策。ルート `DEBUGGING.md` をマニュアルへ統合・最新化しリダイレクト化（mdBook を権威）。Dependencies: pasta-vscode-lua-debug, pasta-source-map, pasta-user-manual
  - discovery 決定（2026-06-08）: DEBUGGING.md = マニュアルに一本化（推奨案）、スコープ = `.pasta` ソースレベルまで完全網羅。brief.md 作成済み（`.kiro/specs/completed/pasta-manual-debugging/brief.md`）。マニュアル全体の権威化再編は manual-ssot-authority へ分離
  - 実装完了 2026-06-08（全8タスク・各独立レビュー APPROVED・機能レベルバリデーション GO・mdbook build/verify-content(G+A〜F)/verify-static/verify-search/drift-check 全緑）。spec 完了フロー未実施

### Phase 5 派生（デバッグ観測性）
- [x] debug-startup-logging -- pasta_lua デバッグバックエンドの `enable()` に「デバッグ有効化・DAP 待ち受け開始（実バインドアドレス `host:port`）」の `info!` 起動ログを追加し、`pasta.log` で起動確認できるようにする。無効時は無言・ゼロコスト維持。Dependencies: pasta-vscode-lua-debug
  - 由来: pasta-manual-debugging 実装後のユーザー検証（2026-06-08）で「pasta.log でデバッグ起動を確認したいが、現状ログが出ない」と判明。デバッグ実装側の観測性ギャップ（pasta-manual-debugging は文書のみ・R8.3 で実装非変更のため境界外）。brief.md 作成済み（`.kiro/specs/debug-startup-logging/brief.md`）。完了後、pasta-manual-debugging のデバッグ章へ「ログ＋ポート確認」手順を小追補可能

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

### Phase 5 派生
- [x] pasta-source-map -- `.pasta`↔生成 .lua ソースマップの**本番実装**（全 generate_* 網羅・本番マップ出力）と、**`.pasta` 座標でのブレークポイント／コールスタックの常時提示**。pasta-vscode-lua-debug が確定した実現可能性ノート・薄い実証スライス・設計シーム（code_gen 接合点／マップ受け渡し IF／DAP source 取り扱い口）を入力として消費し、`.pasta` ソースレベルのデバッグ体験（Phase 5 の最終目標）を完成させる。Dependencies: pasta-vscode-lua-debug（**= 完了済み**）。**完了 2026-06-08（全26サブタスク・各独立レビュー APPROVED・機能レベルバリデーション GO・cargo test --all 緑）。`.pasta` 行 BP／`.pasta` 座標停止・コールスタック／`.pasta` 粒度ステップ（コルーチン跨ぎ含む）／提示モード切替／任意サイドカー出力を実 DAP-over-TCP E2E で実証。OFF 経路バイト不変・既存 Lua デバッグ無回帰**
  - 由来: pasta-vscode-lua-debug のギャップ分析で「.pasta ソースマップ本番化は独立した最大級の塊（code_gen 全 generate_* 波及・双方向変換の正確性）」と判断。ユーザー決定（2026-06-07）により分割し、本仕様は Lua レベルのデバッグを出荷コア、.pasta ソースマップは実現可能性確定までを担うと確定
  - discovery 決定（2026-06-08）: 保持方式 = **メモリ既定＋任意ディスクサイドカー出力**、提示モード = **`.pasta` 既定＋`.pasta`/`.lua` 切替可能**。brief.md 作成済み（`.kiro/specs/pasta-source-map/brief.md`）

### Phase 5 派生（デバッグ UX 修正・2026-06-08）

pasta-source-map 完成後のユーザー実機検証（2026-06-08）で判明した、`.pasta` ソースレベルデバッグの2つの体験ギャップを解消する。両者は責務の縫い目が独立（#1 = ブレーク制御フロー／#2 = 提示レゾルバ＋VSCode UX）で依存関係なし。並行実装可能。

#### 境界戦略（Phase 5 派生 UX）
- **分割理由**: #1 は session/breakpoints の停止制御フローの正しさ（回帰テスト駆動・外部 UI 変更なし）、#2 は提示レゾルバの実行時トグル＋DAP カスタムリクエスト＋VSCode 拡張 UI（UX 駆動）。停止する場所と検証方法が根本的に異なる
- **共有接点**: 両者とも `crates/pasta_lua/src/debug/` を触るが、#1 = `session.rs`/`breakpoints.rs`、#2 = `dap.rs`/`wiring.rs`/`mod.rs`(SourceMode)＋`editors/vscode/` と接触面が分離。`SourceMode::Lua` 時はステップ粒度が `.lua` になり #1 のバグは発生しない（モード直交）

#### Specs (dependency order)
- [x] pasta-debug-break-coalesce -- F5（Continue）で同一 `.pasta` 行から抜け出せず再ブレークする不具合の修正。1つの `.pasta` 行が複数 `.lua` 行へ展開され、対応する全 `.lua` 行へ BP が登録されるため、Continue 後に同 `.pasta` 行を指す次の `.lua` 行で `should_pause()` が即再ヒットする。`.pasta` 行 BP は「`.pasta` 行訪問ごとに1回だけ」発火し、Continue は次の `.pasta` 行まで残りの `.lua` 行を消化（再ブレーク抑制）するよう停止制御へロジック追加。Dependencies: pasta-source-map（完了済み）
- [x] pasta-debug-lua-view-toggle -- `.pasta` 行にブレークを張ったまま、停止時に `.lua` 側コードを提示する「lua 表示モード」を**デバッグ中に実行時トグル**できるようにする。内部の提示モード切替基盤（`SourceMode {Pasta, Lua}`／`pasta_source_resolver`／attach 引数 `sourcePresentation`）は既存。DAP カスタムリクエスト＋VSCode 拡張コマンド/ボタンで `.pasta`⇔`.lua` 提示をセッション中に即切替し、スタックトレース/source 応答へ反映。Dependencies: pasta-source-map（完了済み）
  - discovery 決定（2026-06-08）: 仕様分割 = **2仕様**、問題2の操作性 = **デバッグ中の実行時トグル**（attach 時固定ではなく、DAP カスタムリクエスト＋VSCode UI による即切替）。brief.md 作成済み（`.kiro/specs/completed/pasta-debug-break-coalesce/brief.md`, `.kiro/specs/completed/pasta-debug-lua-view-toggle/brief.md`）

## Phase 6: コード総合レビュー＆改善ループ（移植可能・再実行型）

リポジトリ全域を、**外部観測挙動を変えずに**継続改善する「自己発見ループ型」の再実行可能プロセス仕様。`レビュー領域 × レビュー内容` のマトリクスを実装時にギャップ分析で動的生成し、各セルをサブエージェントへ委譲してループ実行する。別プロジェクトへコピー＋再実行で領域を自動再発見し同等効果を狙う。

### アプローチ決定（Phase 6・2026-06-10 discovery）
- **採用**: 自己発見ループ型（design.md=普遍手順、tasks.md 冒頭にギャップ分析タスク、release-workflow 同様の再実行型 spec・`completed/` へ移動しない）
- **挙動保存**: 正常系厳密保存・攻撃面ハードニングのみ挙動変化許容（境界はテストで明示）
- **レビュー内容 7 次元**: ①テスト網羅性 ②karpathy 簡素化 ③脆弱性対策 ④clippy/lint 徹底 ⑤デッドコード/未使用除去 ⑥パニック経路削減 ⑦ドキュメント/依存整合
- **委譲**: ギャップ分析・各セル改善・自己レビュー・レポート集約をサブエージェントへ。メインはオーケストレーション（ワークリスト・コミット・巻き戻し）に徹する
- **完走保証**: サイクル毎コミット／デバッグ不能セルは直前コミットへ巻き戻して次へ／途中中断・部分出荷禁止／全完走後に改善レポート生成
- **却下**: スキル抽出＋spec ラッパ（2層化）、pasta 具体特化（移植時 tasks 再生成が必要）

### Specs (dependency order)
- [ ] review-improvement-loop -- 移植可能・再実行型のコード総合レビュー＆改善ループ（領域自己発見 × 7 次元マトリクス・サブエージェント委譲・破壊検知＋巻き戻し・改善レポート）。Dependencies: none。brief.md 作成済み（`.kiro/specs/review-improvement-loop/brief.md`）
