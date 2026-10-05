# Roadmap

## 概要

pasta は、日本語 DSL（Pasta DSL）で書いた辞書を Lua へトランスパイルし、組込 LuaJIT で実行する「伺か」の SHIORI（`pasta.dll`）と、その周辺ツール（`pasta_check`・LSP・VSCode 拡張・利用者マニュアル）からなる。Phase 1〜10 で、プロパティアクセス・監査・マニュアル・ソースレベルデバッグ・アクターモデル駆動・配布物の形までを完了した（下の「完了フェーズ」）。

現在の主題は **Phase 11: 現行実装の不具合の一掃**。マニュアル権威化（`manual-ssot-authority`）と内部設計の執筆（`pasta-runtime-internals-doc`）で現行実装を照合した際に見つかったバグ候補を、2026-10-04 の棚卸で現行 main と再照合し、spec 単位に束ねた。不具合を先に片付け、その後に属性セマンティクス（機能拡張）へ進む。

## 運用ルール

- `## Specs (dependency order)` は未完了の spec 専用（`/kiro-spec-batch`・`/kiro-spec-status` が読む）。完了したら `[x]` にし、次の棚卸で「完了フェーズ」へ畳む。
- Dependencies は論理的な依存に加え、**同じソースファイル（関数・領域）を触る spec を同じウェーブに置かない**ための順序を含む。同じウェーブの spec は、互いのソースを触らずに並走できる。マニュアルの同じページの別の節を触ることは許し、後から入る側が rebase で合わせる。
- 個々の経緯・決定は各 spec（`.kiro/specs/completed/<name>/`）と git 履歴が持つ。ここには要約だけを置く。

## 前提として確定している方針

後続の spec が前提にするもの。詳細は各完了 spec とマニュアルの内部設計パートにある。

- **マニュアルが権威**: mdBook（`book/`）が利用者向け情報と内部設計の唯一の権威。挙動を変えたら同じ変更でマニュアルを直し、`node book/tools/gen-skill-refs.mjs` でスキル `references/` を再生成する（CI の `--check`・`link-check.mjs` が鮮度とリンクを見る）。
- **現行実装を正とする**: 旧仕様・吸収台帳の記述は材料であり規範ではない。
- **デバッグ**: Rust ホスト型 DAP バックエンドを `pasta_lua` に内蔵。単一 DLL・opt-in・127.0.0.1 固定。
- **エンジン**: コアは宿主非依存でアクタースレッドに pin。シーンキックは即時再生のみ。
- **LuaJIT 2.1**: `coroutine.close`・`<close>` が無い。中断したコルーチンは参照を外して GC に任せる。

## 完了フェーズ（要約）

| Phase | 主題 | spec |
| ----- | ---- | ---- |
| 1–2 | SSP プロパティの読み書きと DSL 統合（`act:set_property`・`get_property`・`＄％`） | property-write-helpers, shiori-event-test-framework, shiori-async-talk, property-dsl-extension |
| 3 | 全クレートの脆弱性監査・コード簡素化（外部挙動不変） | audit-pasta-core, audit-pasta-dsl, audit-pasta-lua, audit-pasta-shiori, audit-pasta-check, audit-pasta-lsp, audit-pasta-sample-ghost, audit-dependency-supply-chain, audit-workspace-patterns |
| 4 | 利用者マニュアル（mdBook・静的サイト・GitHub Pages）と権威化・内部設計パート | pasta-user-manual, pasta-manual-syntax-highlight, pasta-manual-debugging, manual-ssot-authority, pasta-runtime-internals-doc |
| 5 | VSCode からの `.pasta` ソースレベルデバッグ | pasta-lua-debug-feasibility, pasta-vscode-lua-debug, pasta-source-map, debug-startup-logging, pasta-debug-break-coalesce, pasta-debug-lua-view-toggle |
| 6 | 移植可能・再実行型のコード総合レビュー＆改善ループ | review-improvement-loop（常駐・初回完走 2026-06-12） |
| 7 | 宿主非依存エンジンコア（アクタースレッド）とシーン再生キック | pasta-actor-feasibility, pasta-actor-runtime, pasta-scene-kick, pasta-scene-kick-from-cursor |
| 8 | バルーン表示品質（段落区切り改行・立ち絵の復旧） | sakura-script-newline, actor-surface-restore |
| 9 | 起動堅牢性（長パス・非 ANSI パスのモジュール解決、ロード失敗の可視化） | lua-require-robustness |
| 10 | 配布物の形（同梱バルーンの `updates.txt`） | pasta-check-bundled-balloon（v0.3.7 で公開） |
| — | 動的単語参照 `＠＄`（旧文法仕様の将来項目 B2） | dynamic-word-reference |

ロードマップを置く前の初期開発（DSL・トランスパイラ・ランタイム・SHIORI・alpha リリースなど）の spec も `.kiro/specs/completed/` にある。

### 常駐 spec（再実行型・`completed/` へ移さない）

- `release-workflow` — crates.io・VSIX・ゴーストのリリース手順
- `review-improvement-loop` — レビュー領域 × 7 次元の改善ループ

### 却下（2026-10-01）

- `pasta-sstp-live-output`（SSTP/`\![raise]` による push 出力）— 目的はシーンキックで達成済み。新しい動機が生じたら、その動機から新規に起票する。
- `pasta-authoring-window`（`*.pasta` 編集・プレビュー専用ウィンドウ）— VSCode で実現済み。VSCode 非依存のオーサリング環境が必要になったら再検討する。

### 人手の確認が残っている項目

- `sakura-script-newline` の Task 5.1（実機 SSP での目視確認）— 開発者の手動検証待ち。
- `pasta-check-bundled-balloon` を v0.3.7 で公開したことの、emo2 開発セッションへの連絡 — 実施の有無はリポジトリから確認できない。

## 棚卸（2026-10-04）

### 即時修正（spec なし・本棚卸で実施して閉じた）

設計判断を要さず、1 か所の修正とテストで閉じられるもの。各修正は、現行挙動を書いていたマニュアル（利用者章・内部設計章）も同じコミットで直した。

- DSL・検索・選択肢
  - 括弧式の中の演算（U12）— `（1＋2）＊3` の括弧内が最初の項しか残らなかった。
  - BOM 付きの `.pasta`（旧 R5）— 先頭の UTF-8 BOM を読み飛ばす（`pasta_dsl::parse_str`）。
  - 選択肢の自動ルーティングが表示ラベルで探す（U31）— 選択 ID を Reference1 から読む。
  - 選択肢の自動ルーティングの探索範囲（U27）— ローカルに無ければグローバルシーンを探す（`choice-definition-dsl` 要件 3.4 どおり）。
  - グローバルシーン検索がローカルのキーを除外しない — `:` で始まるキーを候補にしない。
- ローダ・ログ
  - 設置パスの glob メタ文字 — 基準ディレクトリをエスケープする。
  - モジュール名に `.` を含むファイル名 — `.` を `_` にし、キャッシュ先をモジュール名から導く。
  - `.pasta` と `.lua` の同名衝突の判定 — 実際のモジュール名で判定する。
  - ログフィルタの再読み込み不整合（の一部）— `[logging]` の無い再読み込みで既定に戻り、不正な `file_path` でも `level`・`filter` が効く。既定ファイルへのフォールバックは `pasta-toml-logging-consistency` が扱う。
- ランタイム・SHIORI・デバッグ
  - ランダムトークの間隔が起動ごとに同じ — VM 作成直後に `math.randomseed` で種を与える。
  - `unload` を経ないプロセス終了での 5 秒停滞 — プロセス終了による detach では teardown しない（その場合 `SHIORI.unload` と保存は走らない。内部設計に記載）。
  - ANSI コードページが UTF-8（65001）の Windows での `@enc.to_ansi` — 65001 では UTF-8 のバイト列をそのまま返す（開発機では不具合を再現できず、API の制約に基づいて修正）。
  - `encoding` の未使用の公開関数 — `to_ansi_bytes`・`path_from_lua` を削除。
  - Windows で同じデバッグポートへの二重 bind — Windows では `SO_REUSEADDR` を立てない。
  - C のフレームを挟むと下位フレームの変数がずれる — 論理レベルを実レベルへ直す処理を 1 か所にまとめる。
- 利用者章の改訂 — `lua/patterns.md` の作例の誤り（末尾の不要な `act:yield()`・`WORD.create_local` のシーン名・`＠関数名（）` の検索段・`yield` 直後の表示制御）と、同じ作例のある `lua/dsl-vs-lua.md`・pasta-lua-coding スキル・`steering/tech.md`。`reference/pasta-toml.md` の `[loader] debug_mode`（孤立キャッシュの warn）。`debug/troubleshooting.md` の 1 起動 1 接続と `pasta.log` の待ち受けログ（`debug-startup-logging` の申し送りも同時に解消）。

### 統合・分割・改名

- `dynamic-call-nil-guard` → `call-execution-correctness` に統合（同じ `ACT_IMPL.call` と Call のコード生成を触るため）。
- `scene-attribute-semantics` → `scene-attribute-store`（保持・継承・読み出し）と `call-attribute-filter`（Call の属性フィルター）に分割（約 26〜32 タスクで上限を超えるため）。

## Phase 11: 現行実装の不具合の一掃（＋属性セマンティクス）

### 境界戦略

- **分割理由**: 不具合を「根が同じもの」で束ねた。生成コードが存在確認なしに Lua を直接触る（`dsl-codegen-runtime-safety`）、文字列リテラルの文法（`dsl-literal-fixes`）、コールバックが通常の再開経路を通らない（`callback-resume-unification`）、登録キーと検索キーの食い違い（`scene-search-key-normalization`）、ランタイムのシーン名の形式（`scene-identity-format`）など。
- **共有接点**: `crates/pasta_lua/src/code_gen/element_gen.rs` と `crates/pasta_lua/pasta_scripts/pasta/act.lua` を触る spec が多い。これらは 1 ウェーブに 1 spec だけが持つ。トランスパイラのスナップショットを広く変える spec（`dsl-codegen-runtime-safety`・`scene-identity-format`・`call-execution-correctness`）も、別々のウェーブに置く。

### ウェーブ構成

| Wave | spec（並走可） | 種別 | ソースの持ち場 |
| ---- | -------------- | ---- | -------------- |
| 1 | dsl-literal-fixes | バグ（起動不能を含む） | `pasta_dsl` の文法・パーサ、`string_literalizer.rs` |
| 1 | dsl-codegen-runtime-safety | バグ（500） | `element_gen.rs`、`act.lua`、`sakura_script/tokenizer.rs` |
| 1 | callback-resume-unification | バグ（継続の消失・潜在 500） | `pasta_scripts/pasta/shiori/event/`（`choice_select.lua` 以外） |
| 1 | scene-search-key-normalization | バグ | `search/`、`pasta_core` の registry（`random.rs` 以外）、`actor.lua` のアクター単語検索 |
| 1 | pasta-toml-logging-consistency | バグ（設定・ログ） | `pasta_lua` の `loader/config/`・`logging/`、`pasta_shiori` |
| 2 | scene-identity-format | バグ | `scene.lua`、`transpiler.rs`、`debug/source_map/`、`kick.lua` |
| 2 | actor-proxy-act-delegation | バグ（500） | `actor.lua`、`global.lua`、`shiori/entry.lua` |
| 2 | act-token-grouping-fix | バグ | `act.lua`（グループ化）、`sakura_builder.lua`、`ct.lua` |
| 2 | search-selector-indices | バグ（テスト用 API） | `pasta_core` の `random.rs` |
| 2 | string-concat-operator | 機能 | `pasta_dsl` の式の文法（演算子）、`element_gen.rs`（式の Binary）、`act.lua`（算術・連結ヘルパーの領域。グループ化は触らない） |
| 3 | call-execution-correctness | バグ | `element_gen.rs`（Call）、`act.lua`（`init_scene`・`call`） |
| 3 | paragraph-break-tag-only-talk | バグ（表示の空き） | `sakura_builder.lua`（段落区切りの判定）、`appearance.lua`（タグの読み取りの共有だけ） |
| 4 | scene-attribute-store | 機能 | 属性の文法・コード生成・`scene.lua`・`finalize.rs` |
| 5 | call-attribute-filter | 機能 | フィルターの文法・Call のコード生成・検索 |

## Specs (dependency order)

- [x] dsl-literal-fixes -- 改行を含む引用文字列・2 つ目の `""` で生成 Lua が壊れ起動不能になる不具合（U24）、単語値の `「」`・`""` が空にならない（U25）、引用なしの単語値の行末コメント（U06）。Dependencies: none
- [x] dsl-codegen-runtime-safety -- 未定義の `＠＊関数（）`（U18）・未登録アクター（U19）・act のメンバー名と同じアクター名（U20）・数値にできない算術（U22）で 500 になる不具合と、アクション行の `\\`（U08）。生成コードを存在確認付きのヘルパー経由にする。Dependencies: none
- [x] callback-resume-unification -- コールバック再開後の継続の消失、タイムアウト掃引の結果の破棄と予約の残留、タイムアウト応答の二重包み、REG ハンドラの戻り値の二重包み（U23）。コールバックの再開を `EVENT.fire` の再開ループに一本化する。Dependencies: none
- [x] scene-search-key-normalization -- 記号を含むシーン名（U30）・ローカルシーン名・アクター名が、登録キー（サニタイズ済み）と検索キー（元の名前）の食い違いで見つからない不具合。Dependencies: none
- [x] pasta-toml-logging-consistency -- 使われない `[lua] libs`（U26）・`[logging] rotation_days`（U32）の扱い、FFI 入口スレッド（`request`・`unload`・detach）のログの破棄、不正な `file_path` のときの挙動とマニュアルの食い違い。Dependencies: none
- [x] scene-identity-format -- 末尾が数字のシーン名で内部名が重なる不具合（U21）、デバッガのシーン identity の索引漏れ、位置からのキックの前方一致。ランタイム名に区切りを入れ、形式の関数を 1 つにする。Dependencies: scene-search-key-normalization, dsl-codegen-runtime-safety
- [x] actor-proxy-act-delegation -- アクション行の `＠yield`・`＠ゴースト終了`・`＠＄x` などで、ACT を前提とする関数にアクタープロキシが渡って 500 になる不具合。Dependencies: dsl-codegen-runtime-safety, scene-search-key-normalization
- [x] act-token-grouping-fix -- ACT のグループ化が最初の発言より前の表示制御を捨て、スポット変更でグループを閉じない不具合。LuaJIT で機能しない CT（`ct.lua`）の撤去または修正。Dependencies: dsl-codegen-runtime-safety
- [x] search-selector-indices -- `set_scene_selector`・`set_word_selector` の整数が選択に使われない不具合（U29）。Dependencies: scene-search-key-normalization
- [x] string-concat-operator -- 式の文字列連結演算子 `＆`／`&`（算術より低い優先順位・数値は文字列化）。`＋` は数値専用のまま。動的コールのターゲット式での `＆` と `call-attribute-filter` の切り分けを決めて申し送る。`dsl-codegen-runtime-safety` の完成を前提とし、調整は本 spec 側で行う。Dependencies: dsl-codegen-runtime-safety, dsl-literal-fixes
- [ ] call-execution-correctness -- Call から戻った後のシーン文脈が復元されない不具合（U28）と、動的コール `＞式` の値が nil のときの nil ガード（旧 `dynamic-call-nil-guard`）。Dependencies: dsl-codegen-runtime-safety, act-token-grouping-fix, scene-identity-format
- [ ] paragraph-break-tag-only-talk -- タグだけを返す `talk`（表情の単語 `＠通常` など）を `sakura_builder` が字ありと数え、余分な段落区切りの `\n[150]` が出る不具合（2026-10-05 areka「emo2初回起動」からの申し送り）。Dependencies: none
- [ ] scene-attribute-store -- シーン属性の実行時の保持・Lua からの読み出し・ファイルレベル属性の継承と上書き・値の型解釈。Dependencies: dsl-literal-fixes, scene-identity-format, call-execution-correctness
- [ ] call-attribute-filter -- Call の属性フィルター構文（`＞シーン＆k＝v`・比較演算子・複数条件）と実行時の絞り込み。Dependencies: scene-attribute-store, scene-search-key-normalization, call-execution-correctness, string-concat-operator

## バックログ（brief なし・保留）

動機（作者が困った事例・計測した性能問題）が生じたら、その動機から brief を起票する。

### 文法の将来項目（旧文法仕様の申し送り）

- シーンのパラメータ（R1）— シーン宣言での名前付きパラメータ。Call の位置引数とシーン引数 `＄０`… で代替できる。
- アクタースコープ内コードブロックの用途（R2）— アクター固有のイベントハンドラ・状態管理関数。現行は `lua` ブロックの値・関数がアクター付きの単語参照から使えるだけ。
- Call の戻り値と変数代入（R3）— `＄x＝＞シーン`。DSL では定義せず、ランタイム設計の領域。`＄x＝＠＊関数（）` で代替できる。
- さくらスクリプトの `\]`・`\%`（R4）— 角括弧内の `]` と `\%` を DSL から書けるようにするか。現行は `\]` を特別扱いせず、`\%` はパースエラー。
- 継続行内の空行の糖衣構文（R6）— 行継続の途中の空行を改行として出力する。現行は何も出力しない。
- 単語参照への属性フィルター（`＠単語名＆category＝food`）— 単語に属性を付ける構文が無い。`call-attribute-filter` の後に検討する。

### トランスパイラ最適化の候補

いずれも未実装で、計測された性能問題は無い。現行の生成時最適化はマニュアルの [内部設計: トランスパイルパイプライン](https://ekicyou.github.io/pasta/internals/transpiler.html#生成時最適化) が書く。

- 定数畳み込み — トランスパイル時に定数式を評価する。
- デッドコード削除 — 到達不能コードを生成しない。
- インライン展開 — 小さなローカルシーンを呼び出し元に展開する（ソースマップ・デバッグの忠実さと衝突する）。
- 単語プリフェッチ — 単語辞書を先読みする。

### その他

- `pasta_novel` アダプタ（ノベルゲーム宿主）— 遠い将来。Phase 7 の宿主非依存コアと presentation event stream 契約が土台になる。
- モジュール名の衝突（即時修正で判明）— ファイル名の `-`・`.` はどちらも `_` になるため、`a-b.pasta`・`a.b.pasta`・`a_b.pasta` が同じモジュール名になる。`.pasta` 同士の衝突は検出しない（従来から `-` で起きていた）。
- デバッグポートの他プロセスによる奪取（即時修正で判明）— 相手が `SO_REUSEADDR` を立てて bind する場合まで防ぐには、Windows の `SO_EXCLUSIVEADDRUSE` が要る。ゴースト同士の二重 bind は防いだ。
- `pasta_core` の `resolve_scene_id`（即時修正で判明）— 本番の呼び出し元が無くなった公開 API。`find_scene`・`crates/pasta_core/README.md` の例・テストとあわせて整理するかを決める。
- `pasta_shiori` の `util/hglobal/windows_api.rs` の `string_to_multibyte`（即時修正で判明）— `@enc` と同じ 65001 で不正になるフラグを渡すが、テストからしか呼ばれない。
- budoux の自動改行の禁則（2026-10-05、ghost_dev「emo2 開発」からの申し送り）— `line_breaker.rs` の `break_lines_impl` は禁則を見ない。BudouX の語の区切りの直前で改行するだけである。
  - 方針は JIS X 4051 どおりとする。リーダー（`‥…`）は行頭に置いてよい。並びの途中では分けない。
  - この方針では、申し送りの実例「イイジャン！／‥‥ええと、」は正しい組版になる。
  - 句読点・`！？`・閉じ括弧の前、開き括弧の後ろでは、BudouX の既定の日本語モデルは区切らない（手元で確認）。
  - BudouX が行頭禁則の字の直前（または行末禁則の字の直後）で区切る実例が出たら、その実例から起票する。その場合は、既存の `[talk]` の `chars_*` を使った追い出しを第一候補とする。
