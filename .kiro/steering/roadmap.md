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

### 将来仕様（doc/spec 廃止時の申し送り）

manual-ssot-authority で旧文法仕様をマニュアルへ吸収した際に、マニュアルへ収録しなかった将来項目（B＝brief 起票済み、R＝キー行のみ）と、実装照合で見つかった未記載構文のバグ候補を 1 項目 1 行で申し送る。根拠と実装照合は `manual-ssot-authority` の吸収台帳（`.kiro/specs/completed/manual-ssot-authority/absorption-ledger.md`）にある。

- シーン属性のセマンティクス（B1） — 属性によるシーンへのメタデータ付与、ファイルレベル属性の継承と上書き（ローカルシーンには影響しない）、Call の属性フィルター（`＞シーン＆k＝v`・比較演算子・複数条件の結合）。現行は構文の受理と内部の登録表への記録まで — `.kiro/specs/scene-attribute-semantics/brief.md`
- 動的単語参照 `＠＄`（B2） — `＠＄変数名` で変数の値を単語名として参照する。現行はパースエラー。未実装期間の扱い（無視・警告）も未定 — `.kiro/specs/dynamic-word-reference/brief.md`
- シーンのパラメータ（R1） — シーン宣言での名前付きパラメータ。対応予定なし。現行は Call の位置引数とシーン引数 `＄０`… で代替できる — （brief なし）
- アクタースコープ内コードブロックの用途（R2） — アクター固有のイベントハンドラ・状態管理関数としての用途。現行は `lua` ブロックで定義した値・関数がアクター付きの単語参照（A1）から使えるだけで、式の呼び出し・イベントからは届かない — （brief なし）
- Call の戻り値と変数代入（R3） — `＄x＝＞シーン` のような呼び出し結果の代入。DSL では定義せず、ランタイム設計の領域 — （brief なし）
- さくらスクリプトの `\]`・`\%`（R4） — 角括弧内の `\]`（`]` を文字として含める）と `\%` を DSL から書けるようにするか。現行は `\]` を特別扱いせず、`\%` はパースエラー — （brief なし）
- BOM 付きファイルの受理（R5） — UTF-8 の BOM 付き `.pasta` を受理するか。現行はパースエラー — （brief なし）
- 継続行内の空行の糖衣構文（R6） — 行継続の途中の空行を改行として出力する糖衣構文。現行は何も出力しない — （brief なし）

#### 未記載構文のバグ候補

- 単語定義行の行末コメント（U06） — 引用なしの単語値の後ろの `＃…` がコメントにならず、最後の値に取り込まれる — `manual-ssot-authority` の吸収台帳を参照
- `\\` エスケープ（U08） — アクション行の `\\` が `\` 1 文字になり、直後の文字とさくらスクリプトのタグを作る（`\` の表示に `\\\\` が要る） — `manual-ssot-authority` の吸収台帳を参照
- 括弧式の中の演算（U12） — `（1＋2）＊3` の括弧内が最初の項しか残らず `(1) * 3` になる — `manual-ssot-authority` の吸収台帳を参照
- 未定義の `＠＊関数（）`（U18） — GLOBAL に無い関数の呼び出しが Lua の実行時エラーになり、イベントが 500 になる — `manual-ssot-authority` の吸収台帳を参照
- 未登録アクターのアクション行（U19） — アクター辞書にも `[actor]` にも無い名前のアクション行が実行時エラー（500）になる — `manual-ssot-authority` の吸収台帳を参照
- act のメンバー名と同じアクター名（U20） — `talk`・`word`・`call` などと同名のアクターのアクション行が実行時エラーになる — `manual-ssot-authority` の吸収台帳を参照
- 末尾が数字のシーン名（U21） — 内部名（名前＋通し番号）が別名シーンと重なり、`＞A1` が `A` を選ぶ・同じシーン表を共有することがある — `manual-ssot-authority` の吸収台帳を参照
- 数値にできない被演算子の算術（U22） — 数字でない文字列・未代入の変数・nil を返す関数の算術が実行時エラー（500）になる — `manual-ssot-authority` の吸収台帳を参照
- REG ハンドラの戻り値の二重包み（U23） — `RES.ok(…)` などの応答を返すとさらに `RES.ok` で包まれ、不正な SHIORI 応答になる — `manual-ssot-authority` の吸収台帳を参照
- 改行を含む引用文字列（U24） — 改行入りの文字列値（1 ファイルに `""` を 2 つ以上書いた場合を含む）で生成 Lua が構文エラーになり、ランタイムの初期化全体が失敗する — `manual-ssot-authority` の吸収台帳を参照
- 単語値の `「」`・`""`（U25） — 単語定義の値では空文字列にならず、囲み文字そのものが候補の値になる — `manual-ssot-authority` の吸収台帳を参照
- pasta.toml の `[lua]` セクション（U26） — `[lua] libs` がロード時に読まれず、標準ライブラリ・mlua-stdlib の構成が変わらない — `manual-ssot-authority` の吸収台帳を参照
- 選択肢の自動ルーティングの探索範囲（U27） — 既定の OnChoiceSelectEx が直前のグローバルシーンのローカルシーンだけを探し、グローバルシーンへフォールバックしない — `manual-ssot-authority` の吸収台帳を参照
- 別グローバルシーンへの Call 後のローカル探索（U28） — 呼んで戻ったあとの Call・ローカル単語参照が、呼ばれた側のローカルシーンを探す — `manual-ssot-authority` の吸収台帳を参照
- セレクタの整数値が使われない（U29） — `set_scene_selector`・`set_word_selector` の整数値が選択に使われず、シャッフルが止まるだけ — `manual-ssot-authority` の吸収台帳を参照
- 記号を含むグローバルシーン名（U30） — 登録時に英数字以外が `_` に置換され、元の名前で検索・Call できない — `manual-ssot-authority` の吸収台帳を参照
- 選択肢の自動ルーティングが表示ラベルで探す（U31） — 既定の OnChoiceSelectEx が Reference0（表示テキスト）を選択 ID として読み、表示名付きの選択肢が 204 になる — `manual-ssot-authority` の吸収台帳を参照
- `[logging] rotation_days` が使われない（U32） — 設定しても読まれず、ログファイルがローテーションされない — `manual-ssot-authority` の吸収台帳を参照

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
文法・Lua API・`pasta.toml`・入門チュートリアルを検索可能な単一サイトに統合する。マニュアルは利用者向け情報の唯一の権威であり、スキル `references/` の利用者向け部分はマニュアルから生成する（`gen-skill-refs.mjs`。鮮度とリンクは CI の `--check`・`link-check.mjs` が保証する）。

### アプローチ決定（Phase 4）
- **採用**: mdBook（Rust 製・cargo bin に導入済み・追加エコシステム依存ゼロ）。`mdbook build` が静的 HTML+JS（クライアント側 elasticlunr 全文検索・`.nojekyll` 同梱）を出力 → GitHub Pages 等にサーバー不要で公開可能（実機検証済み: mdbook v0.5.3）
- **却下**: Sphinx（Python 依存・reST/MyST 設定が重くオーバースペック）／ VitePress・Docusaurus（node_modules ツリー・2 つ目のエコシステム持ち込み）

### Specs (dependency order)
- [x] pasta-user-manual -- mdBook ベースの利用者マニュアルサイト（Pasta DSL 文法 + Lua API/コーディング + 入門チュートリアル）。Dependencies: none
- [x] pasta-manual-syntax-highlight -- マニュアルの *.pasta コードブロックへ VSCode 同等のシンタックスハイライトを追加。VSCode TextMate 文法（SSOT）を build-time 再利用し hljs 互換クラスへ写像、出力は純静的。Dependencies: pasta-user-manual

### 将来仕様（Phase 4 派生・未着手）

discovery（2026-10-01）で両仕様を起票。依存順は manual-ssot-authority → pasta-runtime-internals-doc。

- [x] manual-ssot-authority -- マニュアル全体の SSOT/権威化の再編。旧文法仕様ディレクトリを mdBook へ吸収・廃止し、旧乖離検出機構をリンク検証（`link-check.mjs`）と生成物鮮度チェック（`gen-skill-refs.mjs --check`）へ置換、`GRAMMAR.md` 廃止。読者で線引きし、利用者向け（文法・公開 Lua API・`pasta.toml`）は mdBook を権威としてスキル `references/` を mdBook から自動生成（スキルは別リポジトリへ持ち出すため自己完結を維持）、AI 作業手順（作例・規約・テスト/lint）はスキル手書きを権威とする。権威移行と生成切替を 1 spec で一括完了。Dependencies: pasta-user-manual
  - 由来: pasta-manual-debugging の discovery（2026-06-08）でユーザーが「mdbook に書いてる項目は mdbook を権威にしたい／別仕様で権威化の整理をすべき」と指摘。本仕様外・別仕様として申し送り
  - brief.md 作成済み（`.kiro/specs/completed/manual-ssot-authority/brief.md`）
  - 完了（2026-10-02）。吸収台帳は `.kiro/specs/completed/manual-ssot-authority/absorption-ledger.md`
- [ ] pasta-runtime-internals-doc -- pasta ランタイムの内部設計・アーキテクチャ解説（トランスパイル / yield-resume コルーチン / シーン検索 / ローダ自己展開 / SHIORI 非同期・アクター基盤 / デバッグ・ソースマップ / シーンキック）。読者＝コントリビュータ・実装理解者。同一 mdBook の末尾に「内部設計」パートとして置く。`OPTIMIZATION.md` を吸収・廃止、スキル `internal-modules` を mdBook 権威＋生成へ移行、クレート README の内部解説を移設。鮮度維持は kiro-complete DoD の追従確認＋review-improvement-loop 次元⑦の照合。Dependencies: manual-ssot-authority
  - 由来: pasta-user-manual の設計ディスカッションで「ランタイム内部設計は本仕様外・将来仕様」と決定（R5 は API 使用法に限定）
  - brief.md 作成済み（`.kiro/specs/pasta-runtime-internals-doc/brief.md`）

### Phase 4 派生（デバッグ利用者ガイド）
- [x] pasta-manual-debugging -- VSCode Lua デバッグ（`.pasta` ソースレベルまで完全網羅）の利用者向けデバッグ章を mdBook マニュアルに追加。有効化／`launch.json`／attach／BP・ステップ・変数 inspect・提示モード切替／構造的制約と緩和策。ルート `DEBUGGING.md` をマニュアルへ統合・最新化しリダイレクト化（mdBook を権威）。Dependencies: pasta-vscode-lua-debug, pasta-source-map, pasta-user-manual
  - discovery 決定（2026-06-08）: DEBUGGING.md = マニュアルに一本化（推奨案）、スコープ = `.pasta` ソースレベルまで完全網羅。brief.md 作成済み（`.kiro/specs/completed/pasta-manual-debugging/brief.md`）。マニュアル全体の権威化再編は manual-ssot-authority へ分離
  - 実装完了 2026-06-08（全8タスク・各独立レビュー APPROVED・機能レベルバリデーション GO・mdbook build/verify-content(G+A〜F)/verify-static/verify-search/当時の文法乖離チェック（manual-ssot-authority で撤去） 全緑）。spec 完了フロー未実施

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
- [x] review-improvement-loop -- 移植可能・再実行型のコード総合レビュー＆改善ループ（領域自己発見 × 7 次元マトリクス・サブエージェント委譲・破壊検知＋巻き戻し・改善レポート）。Dependencies: none。**初回完走 2026-06-12（64 セル・71 タスク）**。再実行型のため `completed/` へは移動せず `.kiro/specs/review-improvement-loop/` に常駐（`/kiro-impl` で再実行）

## Phase 7: アクターモデル駆動エンジン（独立スレッド化＋シーン再生キック）

### 概要
Phase 5 でデバッガ（DAP バックエンド）を組み込んだ結果、ゴーストのデバッグ／オーサリングに本当に必要なのは「デバッグ位置でのブレーク」ではなく「**任意シーンの再生を今すぐキック**」することだと判明した。しかし現状エンジンは SHIORI スレッドに束縛され、ホストのリクエスト周期（OnSecondChange 毎秒等）が唯一の駆動軸＝エンジンは自前の時計を持たない。

本フェーズは、エンジンコアを**宿主非依存・自前スレッド（アクター）化**し、SHIORI アダプタが pull 契約を FIFO/OnSecondChange でブリッジする。これにより任意シーンキックを実現すると同時に、**将来のノベルゲームエンジン化との整合**（SHIORI=pull と novel=push/常駐を一つのコアで支える）を確保する。

### アプローチ決定（Phase 7・2026-06-21 discovery）
- **採用**: 宿主非依存エンジンコア（`pasta_lua`・`!Send`・executor 非依存・**presentation event stream** 出力）＋ 差し替え可能アダプタ（SHIORI=`wintf_winmsg_executor`／将来 novel）。**PoC 先行（feasibility gate）→ 本番**（`pasta-lua-debug-feasibility` と同型）
- **理由**: SHIORI(pull) とノベルゲーム(push/常駐) という真逆の駆動を一つのコアで支えるには、コアが自前の時計＝自前スレッドを持つしかない。executor 選択をアダプタ層に閉じ込めればコア純度を保てる。構造的未知（block-on-reply 反転・reload teardown・coroutine 生存）は PoC で潰してから本番へ
- **挙動保存**: SHIORI 経路の外部挙動は**バイト不変**（純内部リファクタ）。シーンキックは別チャネルの追加機能。`unsafe impl Send`＋Mutex ハックは VM pin により解消（副産物）
- **却下**:
  - 単一基盤 spec 一気通し — 基盤 refactor が最大級の塊。新機能と混ぜるとコミット粒度とリスクが乱れる
  - VM 据え置き＋スレッドセーフキューのみ — SHIORI 単体なら省けるが、ノベルゲーム整合が崩れ将来コアが二重化
  - SSTP/`\![raise]` ライブ push 出力 — pull 契約と衝突。別境界（将来仕様）へ分離

### 出力機構（確定設計）
作者が VSCode（将来は `*.pasta` 編集ウィンドウ）からキック → pasta.dll 内 **talk FIFO** へ積む → **OnSecondChange** 受信時に drain → SSP がライブ再生。別プレビュー画面は不要（ライブ SSP がプレビューを兼ねる）。
- **抑制**: SSP `Status: talking`（権威は SSP）で gate。会話中は通常 FIFO を消費しない
- **即時フラグ**: `talking` でも問答無用で FIFO 消費 → スクリプト応答 → preempt（中断側の前 `co_scene` は閉じる・自動復帰しない）。デバッグ即時再生用
- **入力 marshaling**: SHIORI event を CH 送受信。GET＝応答 tx 付き（受信側が応答義務）、NOTIFY＝義務なし（即 204）、**drop→204 ガード**（panic/忘れでもハング不能）、GET ブロックは短く（キックは executor で非同期実行）、エンジンは yield して block-wait しない、単一直列キューで順序保存

### スコープ
- **対象**: 宿主非依存化、presentation event stream 契約、さくらスクリプト描画の `pasta_shiori` 移設、アクタースレッド（`wintf_winmsg_executor`）、SHIORI marshaling（CH＋GET/NOTIFY＋drop→204）、talk FIFO＋Status-gated drain＋即時 preempt、VSCode キック、debug backend のアクタークライアント化
- **対象外**: ライブ SSP への SSTP/`\![raise]` push 出力（別境界）、`*.pasta` 編集ウィンドウ（別境界）、`pasta_novel` アダプタ実装（遠い将来）、トーク/応答セマンティクス変更（非同期トーク等）

### 境界戦略
- **分割理由**: PoC（使い捨て・GO/no-go）／基盤 refactor（挙動バイト不変・回帰不変性で検証）／キック機能（新挙動・新テスト）は、ライフサイクルと検証方法が根本的に異なる。キックは基盤の存在を前提に初めて設計可能（依存順の自然な境界）
- **共有接点**: 3 spec とも `pasta_lua` runtime 境界と `pasta_shiori` FFI を触る。**presentation event stream** がコア↔アダプタの縫い目。`debug/` の CH 機構はキックの transport へ一般化
- **再統合の余地**: Spec 2/3 は設計フェーズで縫い目が人工的と判明したら統合可（scope-evolution 準拠）

### Specs (dependency order)
- [x] pasta-actor-feasibility -- 使い捨て PoC（feature-gated）。wintf_winmsg_executor による !Send VM ホスト＋reload teardown／block-on-reply marshaling／drop→204 デッドロック消滅／coroutine 生存／FIFO+OnSecondChange+Status gate+即時 preempt の実 SSP ≤1秒キック配信／GET レイテンシ実測 を **GO/no-go 判定**。Dependencies: none。**完了 2026-06-21（全19サブタスク・各独立レビュー APPROVED・機能レベルバリデーション GO・cargo test --all 緑）。判定 = GO+（高信頼）：R1〜R6 全成立。成果物 = `.kiro/specs/completed/pasta-actor-feasibility/verdict-document.md`。出荷 `pasta.dll` バイト不変（正規化 sha 一致）。後続 `pasta-actor-runtime` への申し送り = LuaJIT に coroutine.close 非搭載（preempt は破棄+GC）・release panic=abort で drop→204 は unwind 前提・GET フォールバック閾値候補 6.68ms。撤去手順は `removal-procedure.md` に確定（使い捨て・本番移行完了時に撤去）**。brief.md（`.kiro/specs/completed/pasta-actor-feasibility/brief.md`）
- [x] pasta-actor-runtime -- 本番基盤 refactor。宿主非依存エンジンコア＋presentation event stream 契約（マーカー）＋さくらスクリプトレンダラのアダプタ注入（VM 内レンダリング維持・Lua は pasta_lua 集約・物理移動なし）＋アクタースレッド/VM pin（wintf block_on）＋flume mailbox（単一 recv・select! なし）＋CH marshaling（GET/NOTIFY/drop→204/timeout→204・GET_TIMEOUT 5s）＋static MAILBOX（ArcSwapOption・lock-free 送信）＋unsafe impl Send/Sync 解消＋debug backend 動作保全。**外部 SHIORI 挙動バイト不変・全既存テスト回帰不変**。Dependencies: pasta-actor-feasibility（=GO）。**完了 2026-06-23（全14サブタスク・各独立レビュー APPROVED・機能レベルバリデーション GO・cargo test --all 緑83グループ・出荷 pasta.dll 正規化 sha 不変）。チャンネルは flume 一本化（cancel 安全弁＝単一recv不変条件）＋arc-swap 追加。NOTIFY/OnSecondChange 204 は SecurityLevel 付きでバイト統一**。brief.md（`.kiro/specs/completed/pasta-actor-runtime/brief.md`）
- [x] pasta-scene-kick -- キック機能本番化。**即時再生オンリー**（設計ディスカッション 2026-06-23 で talk FIFO／Status-gated drain／非即時モードを廃止し、既存 `co_scene` 継続機構の流用＋初回ビートのみ `is_blocked` ワンショット突破＋即時 preempt-and-abort へ改訂）＋VSCode `playScene` キックコマンド＋debug DAP チャネル一般化（custom request）＋debug backend のアクタークライアント化（`KickSink` 汎用 seam でクレート依存方向 `pasta_shiori`→`pasta_lua` 順守）＋ctx 合成は通常トーク再生を流用。Dependencies: pasta-actor-runtime。**完了 2026-06-23（全19サブタスク・17実装コミット・機能レベルバリデーション GO・cargo test --workspace 2139 passed/0 failed・VSCode test:unit 緑・要件28基準全充足・依存方向／境界監査クリーン・キック未使用バイト不変）**。brief.md（`.kiro/specs/completed/pasta-scene-kick/brief.md`）

### 将来境界（Phase 7 派生・未着手）
- ~~pasta-sstp-live-output~~ -- **却下（2026-10-01 discovery）**。ライブ SSP への SSTP/`\![raise]` push 出力経路。実体は Phase 7 discovery で却下されたキック出力の対案で、独立した利用者・用途は一度も記録されていない。本来の目的（VSCode から編集中トークを即喋らせる）は pasta-scene-kick（≤1秒 pull 配信）＋pasta-scene-kick-from-cursor で達成済み。ゴースト自発の非同期出力等の新たな動機が生じたら、その動機から新規に起票する
- ~~pasta-authoring-window~~ -- **却下（2026-10-01 discovery）**。`*.pasta` 編集/プレビュー専用ウィンドウ（executor スレッド同居・`!Send` VM 直接アクセス）。本来の目的（ゴースト辞書のライブ修正）は VSCode（LSP・ハイライト・ソースレベルデバッグ・カーソル位置キック）で実現済みのため当面不要。VSCode 非依存のオーサリング環境が必要になったら再検討する
- `pasta_novel` アダプタ（ノベルゲーム宿主）は遠い将来。本フェーズの宿主非依存コア＋presentation event stream 契約がそれを可能にする土台となる

## Phase 8: バルーン表示品質（さくらスクリプト出力）

さくらスクリプトのバルーン表示に関する出力品質の修正群。

### Specs (dependency order)
- [x] sakura-script-newline -- キャラ切替時の段落区切り改行を eager（`\p` 直前・先出し）から fully-lazy（切替で保留し、再登場スコープの次の一般文字列直前でフラッシュ）へ変更。A→B 終了・同一スポット共有交代・全さくらスクリプト手番のゴミ改行を根絶。Dependencies: none。実装完了・全テスト green（`cargo test --all` 含む）。Task 5.1（実機SSP目視）は開発者による手動検証待ち。

### Phase 8 派生（未着手）
- [x] actor-surface-restore -- 同一スポットを複数アクターが共有して交代する際の、切替先アクターの立ち絵（サーフェスID・着せ替え状態）の復旧。Dependencies: sakura-script-newline
  - 由来: sakura-script-newline の要件ディスカッション議題2（2026-07-18）でユーザーが「同一スコープでキャラが変わる場合、立ち絵の復旧が必要」と指摘。段落区切り改行（string 順序）とサーフェス状態管理（アクター状態機械）は責務が異なるため別仕様へ分離。brief.md 作成済み（`.kiro/specs/completed/actor-surface-restore/brief.md`）

## Phase 9: 起動堅牢性（モジュールロード）

areka 実機検証（2026-09-18）で発覚した、深いフォルダへ設置したゴーストが無言になる不具合への対処。独立した 2 つの欠陥（長パスで require 失敗・ロード失敗の無言化）を 1 spec に統合（両者とも `factory.rs` の起動時ロードを触るため分割するとマージ競合する・discovery 決定）。

### Specs (dependency order)
- [x] lua-require-robustness -- (B) `pasta.shiori.entry` のロード失敗を握りつぶさず既存の 500 + `X-ERROR-REASON` 経路（load-error-logging）へ伝搬 → (A) `package.loaders` へ Rust 実装 searcher を前置し、LuaJIT の narrow fopen（MAX_PATH=260・ANSI コードページ制限）を回避。チャンク名は現行形式と厳密一致（ソースマップ無回帰）。Dependencies: none。brief.md 作成済み（`.kiro/specs/completed/lua-require-robustness/brief.md`）

## Phase 10: 配布物の形（pasta_check release）

areka alpha の実機ラップ（2026-10-02・A3r 項目 8）で発覚した、同梱バルーンのファイルがゴーストの `updates.txt` に載り、ネットワーク更新でゴースト配下へ誤配置される不具合への対処。emo2 開発セッション（ghost_dev）からのブリーフィングで discovery（2026-10-02）。方針は「pasta_check 側で汎用に直す（案 A）」でユーザー決定済み。

### Specs (dependency order)
- [ ] pasta-check-bundled-balloon -- `install.txt` の `balloon.source.directory`／`balloon.directory` で同梱バルーンを判定し、その配下をゴーストの `updates.txt`（ルート・`ghost/master`）から除外、同梱バルーン直下へバルーン用 `updates.txt` を生成して nar に封入。`homeurl` 欠落は警告。同梱バルーンの無いゴーストは date 以外バイト不変・改行変換なし。Dependencies: none。完了後に `release-workflow` で crates.io へ出し、emo2 開発セッションへバージョンを連絡する。brief.md 作成済み（`.kiro/specs/pasta-check-bundled-balloon/brief.md`）
