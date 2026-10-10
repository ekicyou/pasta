# Roadmap

## 概要

pasta は、日本語 DSL（Pasta DSL）で書いた辞書を Lua へトランスパイルし、組込 LuaJIT で実行する「伺か」の SHIORI（`pasta.dll`）と、その周辺ツール（`pasta_check`・LSP・VSCode 拡張・利用者マニュアル）からなる。Phase 1〜11 で、プロパティアクセス・監査・マニュアル・ソースレベルデバッグ・アクターモデル駆動・配布物の形・現行実装の不具合の一掃までを完了した。2026-10-09 までに、リリースの CI 化、シーン名の別名、式の中の値なしの扱い、マニュアルの着せ替え、段階ごとに学べる hello-pasta の辞書も入った（下の「完了フェーズ」）。

現在の主題は 2 つある。

- **Phase 11 の残り: 属性セマンティクスと失敗の出力**。シーン属性の保持と読み出し、実行時の失敗の出力の一本化、Call の属性フィルター、文中のシーンリンク、スクリプトが止まるエラーのバルーン表示が残る。
- **Phase 12: 初心者向けの入門ガイド**。段階辞書とマニュアルの着せ替えは済んだ。残るのは、hello-pasta の新しいシェル、入門ガイドの本文の書き直し、スクリーンショット。

このほか、リリース手順（常駐 spec `release-workflow`）の CI に合わせた書き換えと、CI での初回のリリース（v0.3.8）は済んだ（2026-10-10）。

**優先順位（開発者の方針 2026-10-10）**: トークンの予算のため、当面はマニュアルのサイトの作り直し（入門ガイドの本文）を最優先とし、CI とリリース、急いで直す不具合だけを進める。それ以外は見送る（下の「棚卸」のウェーブ）。

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
- **シーン名の別名**: pasta.toml の `[scene.alias]` で定義する。未定義なら `会話 → OnTalk` の 1 件が既定。完全一致の 1 段だけで、キー正規化の前段で登録・検索の両側に効く。
- **式の中の値なし**: 式の中の nil は、算術なら 0、連結なら空文字列とみなし、ログを出さない。
- **リリース**: タグ `vX.Y.Z` の push で `.github/workflows/release.yml` が検証・ビルド・公開まで行う。ビルドした成果物は git で追跡しない。シェルの画像は追跡を続ける。
- **hello-pasta の辞書**: 1 段 1 ファイル（`dic/01-boot.pasta`〜`12-lua.pasta`）で、段階表は `crates/pasta_sample_ghost/STAGES.md`。全段が起動することをテストが確かめる。
- **マニュアルの見た目**: mdBook のまま「Claudia のマニュアル」。台詞は顔アイコン付きの部品で書く（記法は `book/AUTHORING.md`）。

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
| 11 | 現行実装の不具合の一掃（文字列リテラル・生成コードの安全性・コールバックの再開・検索キー・設定とログ・シーンの内部名・アクタープロキシ・グループ化・セレクター・連結演算子・Call の文脈・段落区切り） | dsl-literal-fixes, dsl-codegen-runtime-safety, callback-resume-unification, scene-search-key-normalization, pasta-toml-logging-consistency, scene-identity-format, actor-proxy-act-delegation, act-token-grouping-fix, search-selector-indices, string-concat-operator, call-execution-correctness, paragraph-break-tag-only-talk |
| 11 | シーン名の別名（`＊会話` を OnTalk の既定の別名にする）、式の中の値なしの扱い（算術なら 0・連結なら空文字列） | scene-name-alias, expr-nil-coercion |
| 12（前半） | 段階ごとに起動できる hello-pasta の辞書と段階表、マニュアルの「Claudia のマニュアル」への着せ替えと台詞の部品 | hello-pasta-tutorial-stages, manual-claudia-theme |
| 12（本文） | 入門ガイドの本文を、段階表に沿った 16 章（入口・準備 2・段の章 13）の物語に書き直した。段の章と辞書の逐語の照合、本文の検査、目次の開閉の持ち越しを足した | getting-started-story-guide |
| — | リリースの CI 化（タグの push で crates.io・Marketplace・GitHub Release へ公開。認証は OIDC、成果物の git 追跡を解除） | release-ci |
| — | 動的単語参照 `＠＄`（旧文法仕様の将来項目 B2） | dynamic-word-reference |

ロードマップを置く前の初期開発（DSL・トランスパイラ・ランタイム・SHIORI・alpha リリースなど）の spec も `.kiro/specs/completed/` にある。

### 常駐 spec（再実行型・`completed/` へ移さない）

- `release-workflow` — crates.io・VSIX・ゴーストのリリース手順（版の決定 → 版の更新の PR → リリースタグの push → リリース CI の結果の確認。CI に合わせた書き換えと、CI での初回のリリース（v0.3.8）は済んだ。下の「リリース手順の書き換え」）
- `review-improvement-loop` — レビュー領域 × 7 次元の改善ループ

### 却下（2026-10-01）

- `pasta-sstp-live-output`（SSTP/`\![raise]` による push 出力）— 目的はシーンキックで達成済み。新しい動機が生じたら、その動機から新規に起票する。
- `pasta-authoring-window`（`*.pasta` 編集・プレビュー専用ウィンドウ）— VSCode で実現済み。VSCode 非依存のオーサリング環境が必要になったら再検討する。

### 人手の確認が残っている項目

- `sakura-script-newline` の Task 5.1（実機 SSP での目視確認）— SSP を操作するツール（SSP MCP）で実機の確認ができるようになった（2026-10-09 に `hello-pasta-tutorial-stages` が使った）。`getting-started-screenshots` の実機の確認のついでに済ませるかを、同 spec の要件で決める。
- `pasta-check-bundled-balloon` を v0.3.7 で公開したことの、emo2 開発セッションへの連絡 — 実施の有無はリポジトリから確認できない。
- `getting-started-story-guide` の公開後の確認 4 点（2026-10-10。実装では、SSP を `--ghost` で一時起動して全 13 段をたどった。次の操作は試していない）— (1) 準備の章: ゴーストのフォルダを SSP の `ghost/` の下へ置いて切り替え、立ち絵が無い状態でタスクバーの SSP のアイコンからメニューを開いて終了できるか（最初に確かめる。違っていると読者が 1 段目へ進めない） (2) 目次: 実際のブラウザーで幅を狭めて開き、項目をクリックしても開いたままか (3) 7 段目: メニューから emo2 そのもの（`えも？？`）と切り替えて挨拶が出るか (4) 13 段目: フォルダのドロップで `.nar` ができるか（読者の `profile/` が入るかも見る）。`getting-started-screenshots` の実機の作業のついでに済ませるかを、同 spec の要件で決める。

## 棚卸（2026-10-10・main `add05022`）

前回（2026-10-07）の後に 5 本が入った（`release-ci`・`scene-name-alias`・`expr-nil-coercion`・`manual-claudia-theme`・`hello-pasta-tutorial-stages`）。未完了の 7 本を現行 main と再照合した（各 brief の末尾に「2026-10-10 棚卸の再測定」の節がある）。進行中の spec は無かった。前回までの棚卸の記録は、git の履歴にある。

### 即時修正（spec なし・本棚卸で実施して閉じた）

- `steering/product.md` — `SOUL.md` へのリンク切れ 2 か所、完了件数（156）、存在しないコマンド名（`/kiro-spec-impl` → `/kiro-impl`）、現在地。
- `steering/structure.md` — シーン名の別名表（`scene_alias.rs`）の追加、ローダー設定がフォルダ（`config/`）になったこと、`pasta_check` の依存の一覧、文法定義ファイルのパス。
- `steering/tech.md` — 文法定義ファイルの名前、公開ポリシー（公開済み・リリース CI が公開する）、マニュアルの CI が起動する条件。
- `steering/workflow.md` — 存在しないコマンド名 2 か所。
- `TEST_COVERAGE.md` — 完了した spec へのリンクを `completed/` に直した。
- `crates/pasta_shiori/tests/shiori_lifecycle_test.rs` — もう存在しない spec フォルダを指すコメントを消した。
- `editors/vscode/scripts/build-wasm.ps1` — 使い方のコメントを今の呼び出し方に直した。
- この roadmap — 完了した 5 本を「完了フェーズ」へ畳んだ。バックログの「print.html の動画の参照切れ」の説明の誤り（壊れているのは動画だけで、代わりのリンクは働いている）は、起票した brief で正した。

### spec へ申し送ったもの（各 brief の再測定・申し送りの節）

- `scene-attribute-store` — 見本の辞書（`sample.pasta`）にファイルの属性があるので、属性を出力すると期待値のファイルが変わる。宣言行の属性と属性行の両方に同じキーがあるときの優先を決める。別名で同じ名前になるシーン（`＊会話` と `＊OnTalk`）があるので、属性は宣言ごとに持つ。
- `failure-output-unification` — 数値化・文字列化の関数（`act.lua` の `ACT.num`・`ACT.str`）と動的な単語キーは `act` を受け取らないので、今のままではバルーンへ出せない。生成コードの形を変えるか、出口の側が「今の act」を持つかを要件で決める。未登録のアクターは、今も 2 回ログに出る。
- `call-attribute-filter` — `pasta_core` の `resolve_scene_id`（本番の呼び出し元が無い公開 API）を消すか残すか。LSP の Call の読み取り（`visit_action.rs`）も触る。フィルターで外れたときも「シーンが見つからない」と出る。イベントの入口（`act:find_scene`・`SCENE.co_exec`）も絞るか。
- `scene-anchor-link` — hello-pasta への作例は Scope から外す（段階表と重なり、20 タスクを超える）。`OnAnchorSelectEx` が何も返さないと SSP は続けて `OnAnchorSelect` を送る。VSCode 拡張の単語参照の枠（`wordRefDecorator.ts`）が `＠？名前` も囲む。
- `hello-pasta-shell-art` — 配布物を作るスクリプト（`release.ps1`）の絵を生成する段を外さないと、リリース CI がコミットした絵を上書きする。8 段目の台詞は当たり判定の部位名をそのまま声に出すので、読んでおかしくない名前にする。配布物の大きさの上限を数字にする。`crates/pasta_sample_ghost/README.md` の `scripts/` の説明が古い。
- `getting-started-story-guide` — 今の入門ガイドの古い説明 4 か所（途中の段階では起動しないことがある・`scripts/` に Lua ランタイムを置く・8 段目の「書き方は変わらない」・画像を自動生成する仕組み）。書き直しで消える。ゴースト自身の切り替えでは `OnGhostChanging` が届かない。
- `shiori-test-support-runtime` — 回避の設定が 4 か所に増えた（`scene-name-alias` の結合テストが写した）。テスト用ライブラリの写し（`tests/support/scriptlibs/`）も古い。別のコピー処理が今の読み込み先に効いているかを確かめる。
- `release-workflow` の書き換え — 下の「リリース手順の書き換え」に項目を足した。
- 次の `review-improvement-loop` — `pasta_shiori` の `string_to_multibyte`（テスト専用）を消すか、3 行の分岐を足すか。

### 統合・分割・起票

- 分割: `getting-started-story-guide` を本文と絵に分けた（22〜24 タスクで 20 を超えた。本文は今始められ、新しいシェルを待つのは絵だけ）。本文は元の名前のまま、絵は `getting-started-screenshots`。
- 分割: `failure-output-unification` から、スクリプトが止まるエラー（500 の応答）のバルーン表示を `runtime-error-balloon` へ出した（含めると 25〜29 タスク）。境目は「シーンがまだ生きていて、`act` に積めるか」。
- 起票: `pasta-check-dic-validate`（バックログの「`.pasta` を検査するコマンド」。`scene-anchor-link`・`failure-output-unification` が読み込み時の検出をここへ任せており、手作りの検査と回避の注記が 3 か所にある）。バックログの「モジュール名の衝突」「サニタイズ後のシーン名の衝突の警告」と、`scene-name-alias` が範囲外にした「`pasta_check` に pasta.toml を読ませる」を検査の項目として引き取る。
- 起票: `manual-print-media-refs`（バックログの「print.html の動画の参照切れ」。コードと CI を変えるので spec にした）。
- 順序: `hello-pasta-shell-art` の先に要る 2 本は完了した。`scene-anchor-link` と `call-attribute-filter` は同じファイルを触る順番だけの関係で、どちらを先にしてもよい（下の「開発者の判断が要るもの」）。
- バックログに残したもの: 文法の将来項目・最適化の候補（動機なし）、デバッグポートの奪取（守りの方針の判断）、式の木の優先順位・デバッガ索引・タグの読み取り・閉じていない引用（利用者なし）、時々落ちる TCP のテスト（新しい発生の記録なし）、BudouX の禁則（実例待ち）。完了 spec に残っていた 3 件を足した。

### ウェーブ（2026-10-10）

依存の木の先頭は 7 本ある。ただし開発者の方針（2026-10-10）で、このウェーブは**マニュアルのサイトの作り直し・CI とリリース・急いで直す不具合**に絞る（トークンの予算のため）。急いで直す不具合は無かった。始めるのは次の 2 つ。

| 対象 | 種別 | 規模 | 要件定義 |
| ---- | ---- | ---- | -------- |
| getting-started-story-guide（本文） | 文書（サイトの作り直し・最優先） | 18〜20 | Fable |
| `release-workflow` の書き換え（常駐 spec。下の「リリース手順の書き換え」） | 基盤（リリース・期限 2026-12-01） | 7〜9 | Opus |

絵の無い入門ガイドを先に公開してよい（開発者決定 2026-10-10）。本文は、新しいシェルとスクリーンショットを待たずに main へ入れる。

見送った先頭（予算が戻ったら、この順に取る。互いのソースは下の約束で重ならない）:

| spec | 子孫 | 種別 | 規模 | 要件定義 |
| ---- | ---- | ---- | ---- | -------- |
| failure-output-unification | 約 3（`call-attribute-filter`・`scene-anchor-link`・`runtime-error-balloon`） | 機能 | 15〜19 | Fable |
| scene-attribute-store | 約 2（`call-attribute-filter`・`scene-anchor-link`） | 機能 | 15〜18 | Fable |
| hello-pasta-shell-art | 約 1（`getting-started-screenshots`） | 機能（開発者の方針） | 12〜14 | Fable |
| shiori-test-support-runtime | 約 1（`runtime-error-balloon`） | 基盤 | 6〜8 | Opus |
| manual-print-media-refs | 0 | バグ | 2〜3 | Opus |

同じウェーブに置くときの約束（見送った spec を後から始めるときも、そのまま使う）:

- `failure-output-unification` は、生成コードの形（`expr_gen.rs`・`element_gen.rs`・見本の期待値ファイル）を変えない。期待値ファイルはこのウェーブでは `scene-attribute-store` が作り直す。要件で生成コードを変えると決めた場合は、`scene-attribute-store` を先に入れて rebase する。
- `scene-attribute-store` は、属性を読み出す API を `act.lua` に置かない（`scene.lua` の `SCENE` 側に置く）。`act.lua` はこのウェーブでは `failure-output-unification` が持つ。
- `hello-pasta-shell-art` は、頭の部位名を今の `Head` のままにし、hello-pasta の辞書（`dic/`）と `book/` を触らない。`Cargo.lock` とルートの `Cargo.toml`（絵を描く部品の取り外し）は、このウェーブではこの spec だけが触る。
- `getting-started-story-guide` は、シェルのファイルの中身を本文に書き写さず、画像を置かない。部位名は段階表（`STAGES.md`）から取る。`.github/workflows/manual.yml` を触らない。
- `manual-print-media-refs` は、`manual.yml` と `book/tools/verify-static.mjs` をこのウェーブで持つ。入門の章と、本文の検査（`verify-content.mjs`・`tutorial-check.mjs`）を触らない。
- `shiori-test-support-runtime` は `crates/pasta_shiori/tests/` だけを触る。Phase 11 の spec が結合テストを足す前に入れるほど、回避の写しが増えない。
- マニュアルの同じページの別の節（`lua/script-api.md` など）は、`scene-attribute-store` と `failure-output-unification` が分け合う。スキル references は後から入る側が再生成する。

見送った先頭の後に置くもの: `pasta-check-dic-validate`（子孫 0。依存を足すと `Cargo.lock` が変わり、このウェーブでは `hello-pasta-shell-art` がその席を持つ。マニュアルに章を足すと、章の数の検査を `getting-started-story-guide` と共有する）。

その次の候補: `call-attribute-filter`（または `scene-anchor-link`。同じファイルを触るので 1 本ずつ）、`runtime-error-balloon`、`pasta-check-dic-validate`、`getting-started-screenshots`。互いのソースは重ならない。

### 開発者の判断が要るもの（ウェーブは止めない）

- 見送った spec をいつ始めるか — 予算の都合で、Phase 11 の残りと `hello-pasta-shell-art`・`shiori-test-support-runtime`・`manual-print-media-refs` を見送った。`shiori-test-support-runtime` は、Phase 11 の spec より先に入れるほど後片付けが減る。
- スクリプトが止まるエラーをバルーンに出すか — `runtime-error-balloon` の要件の最初の議題。要らなければ却下する。
- `call-attribute-filter` と `scene-anchor-link` のどちらを先にするか — 台帳は前者を先にしている。後者を先にすると、`scene-attribute-store` を待たずに始められる。
- リリース手順の書き換えと初回の CI リリースの時期 — `hello-pasta-shell-art` が `release.ps1` を変える前に済ませると、失敗の原因を認証の設定だけに絞れる。期限は 2026-12-01。→ 済んだ（2026-10-10。v0.3.8）。
- `steering/structure.md` の作り直し — `pasta_shiori` の木が無い、`pasta_lua/tests` の一覧が古い、など。`/kiro-steering` で直す。

## Phase 11 の残り: 属性セマンティクスと失敗の出力

### 境界戦略

- **分割理由**: シーン属性を 2 つに分けた（2026-10-04）。保持・継承・読み出し（`scene-attribute-store`）と、Call の属性フィルター（`call-attribute-filter`）。実行時の失敗の出力も 2 つに分けた（2026-10-10）。シーンが生きている間の失敗の出口の一本化（`failure-output-unification`）と、スクリプトが止まるエラーのバルーン表示（`runtime-error-balloon`）。
- **共有接点**: `crates/pasta_lua/pasta_scripts/pasta/act.lua` は 1 ウェーブに 1 spec だけが持つ。`failure-output-unification`・`call-attribute-filter`・`scene-anchor-link` が触る。`crates/pasta_lua/src/code_gen/element_gen.rs` も 1 ウェーブに 1 spec だけで、`call-attribute-filter` と `scene-anchor-link` が触る。`scene-attribute-store` と `call-attribute-filter` は、`scene.lua` と `pasta_core` のシーン登録（`scene_registry.rs`・`scene_types.rs`）を共有するので、この順に置く。
- **シーン名の別名との関係**: 別名の解決は検索の入口（`search/context.rs`）でキー正規化の前に行う。`scene-attribute-store`・`call-attribute-filter` は、別名を当てた後の名前で属性を扱う。
- **結合テスト**: `pasta_shiori` の結合テストは、`shiori-test-support-runtime` が入るまで古いランタイムの写しへの回避を写して増える。`runtime-error-balloon` は同じ `crates/pasta_shiori/tests/` を触るので、その後に置く。

### ウェーブ構成

| Wave | spec（並走可） | 種別 | ソースの持ち場 |
| ---- | -------------- | ---- | -------------- |
| 4（見送り中） | scene-attribute-store | 機能 | 宣言行の属性のパース（`parse_scene.rs`・`ast/scene.rs`）、`scope_gen.rs`・`context.rs`・`transpiler.rs`、`scene.lua`、`finalize.rs`、`pasta_core` のシーン登録（`scene_registry.rs`・`scene_types.rs`）、見本の期待値ファイル、マニュアル（`block-structure.md`・`script-api.md`・`registry-search.md`・`transpiler.md`） |
| 4（同上） | failure-output-unification | 機能 | `act.lua`・`actor.lua`・`word.lua` の警告と失敗表記、pasta.toml の切り替え（`loader/config/`）、マニュアル（`action-line.md`・`call-jump.md`・`script-api.md`・`talk-output.md`・`internal-modules.md`） |
| 4（同上） | shiori-test-support-runtime | 基盤 | `crates/pasta_shiori/tests/`（古いランタイムの写しの撤去・共通部品・回避の設定 4 か所） |
| 5 | call-attribute-filter | 機能 | Call の文法（`grammar.pest`・`parse_action.rs`・`ast/action.rs`・`partial.rs`）、`element_gen.rs`（Call）、`act.lua`（`call`・`find_handler`）、`scene.lua`（`SCENE.search`）、`search/`、`pasta_core` の `scene_table.rs`・`scene_types.rs`、LSP（`visit_action.rs`）、VSCode の文法定義 |
| 5 | runtime-error-balloon | 機能 | `pasta_scripts/pasta/shiori/`（`entry.lua`・`event/init.lua`・`event/callback.lua`・`res.lua`）、`pasta_shiori` の `error.rs`・`shiori.rs`・`actor/thread.rs` と結合テスト。`act.lua` は触らない |
| 6 | scene-anchor-link | 機能 | `call-attribute-filter` と同じ文法・生成・`act.lua`・エディタのファイルに加え、`sakura_builder.lua`、`shiori/event/choice_select.lua`、`sakura_script/`、VSCode 拡張の `wordRefDecorator.ts` |

Wave 1〜3 の 12 本と、先行させた `scene-name-alias`・`expr-nil-coercion` は完了した（「完了フェーズ」の Phase 11）。

### 文中のシーンリンク（2026-10-08 起票）

台詞の中の語を Wikipedia のリンクのようにし、クリックでそのシーンへ飛べるようにする（`scene-anchor-link`）。

- **方式**:
  - 記法は `＠？シーン名`（`＠単語　` と同じく、空白か改行で終える）と `＠？シーン名「表示名」`。
  - 出力はさくらスクリプトの `\_a`（アンカー）で、`\q` ではない。`\q` だと、トーク全体が選択肢待ちになるため。
  - クリックは `OnAnchorSelectEx` を、選択肢の振り分け（`choice_select.lua`）と同じ規則で受ける。
  - `【】` で囲む案は、文法として唐突なので採らなかった。
- **飛び先が無いとき**: 選択肢と同じく 204 を返す。失敗の見せ方は `failure-output-unification` に乗せる。読み込み時の検出は `pasta-check-dic-validate` に任せる（記法の検査は、後から入る側が足す）。
- **ウェーブ**: `grammar.pest`・`parse_action.rs`・`element_gen.rs`・`act.lua`・VSCode の文法定義を、`call-attribute-filter` と共有する。そのため 1 本ずつ入れる。

## Phase 12: 初心者向けの入門ガイド

### 方針（2026-10-06 決定）

- 文法リファレンス（`grammar/`）と Lua 章（`lua/`）は、リファレンスのまま残す。入門ガイドが「こんな表現をしたい」を順に叶える物語で導き、各章から詳しい文法へリンクで送り出す。
- ガイドの間は Claudia が全編を語る（説明本体も Claudia の語り）。これは `getting-started` に限った執筆規約の例外とする。コードブロック内・構文定義・コマンド例には口調を持ち込まない。
- 題材は hello-pasta のまま（Claudia ゴーストは別に存在するので、ガイドの題材にはしない）。各章の終わりで読者のゴーストが起動できるようにし、最終章の辞書を hello-pasta と一致させる。
- hello-pasta のシェルは fal.ai の画像生成で作り直す。本線は `qwen-image-edit-2511`、切り抜きは BiRefNet。
- マニュアルの見た目は、mdBook のまま「Claudia のマニュアル」に着せ替える（完了）。手本は ponapalt さんの「悪役令嬢クローディア」紹介ページ（<https://ponadocs.shillest.net/claudia/>、ponapalt/claudia、Unlicense）。サイト生成器の移行は採らない。

### 境界戦略

- **分割理由**: 持ち場で分けた。シェルの画像（`crates/pasta_sample_ghost` の画像側。`hello-pasta-shell-art`）、ガイドの本文（`book/`。`getting-started-story-guide`）、スクリーンショット（新しいシェルと本文の両方が要る。`getting-started-screenshots`）。段階表と段階辞書、台詞の部品は完了している。
- **共有接点**: 段階表（`crates/pasta_sample_ghost/STAGES.md`）と段階辞書（`dic/01-boot.pasta`〜`12-lua.pasta`）を、本文が逐語で照合する（`book/tools/tutorial-check.mjs`）。辞書を変える spec は、同じ変更で入門の章の作例を直す。マニュアルの CI は辞書の変更でも走る。
- **シェルと本文の接点**: 8 段目（触られたときの反応）は、当たり判定の部位名を台詞に出す。部位名は `hello-pasta-shell-art` が要件で決め、本文は段階表から取る。頭は `Head` のまま変えない。
- **リリースとの接点**: `hello-pasta-shell-art` は、配布物を作るスクリプト（`release.ps1`）の絵を生成する段を外す。リリースの実行中は `release.ps1` を変えない。

### ウェーブ構成

| Wave | spec（並走可） | ソースの持ち場 |
| ---- | -------------- | -------------- |
| 2（見送り中） | hello-pasta-shell-art | `crates/pasta_sample_ghost` の絵を描くプログラム（`src/image_generator.rs` ほか。消す）・`Cargo.toml`、ルートの `Cargo.toml`・`Cargo.lock`、`shell/master/`、`README.md`、`release.ps1`、`STAGES.md` の部位名の注記、`tests/integration_test.rs` の画像のテスト、生成の手順と出どころの記録 |
| 2（2026-10-10 のウェーブ・最優先） | getting-started-story-guide（本文） | `book/src/getting-started/`・`SUMMARY.md`・`introduction.md`・`AUTHORING.md`・`book.toml`（転送）、`book/tools/` の `verify-content.mjs`・`tutorial-check.mjs` と自己テスト（章の数を決め打ちする `verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs` を含む）、`crates/pasta_lua/README.md`・`crates/pasta_shiori/README.md` のリンク |
| 3 | getting-started-screenshots | 撮影の手順、画像、各章への画像の行、新しいシェルでの 8 段目の実機の確認 |

Wave 1（`hello-pasta-tutorial-stages`・`manual-claudia-theme`）は完了した。

## リリース手順の書き換え（`release-ci` の後）

リリースの CI 化（`release-ci`）は完了し、タグの push だけで公開まで進む。常駐 spec の旧手順（`release/hello-pasta.nar` を生成してコミットする段・手元でのビルドと公開）は、そのままでは動かないので、下の Existing Spec Updates で書き換えた（2026-10-10 に済んだ。要件・設計・タスクの書き直しと、文書・設定の一回限りの整合）。CI での初回のリリース（v0.3.8）と後片付けも済んだ（2026-10-10。記録は `.kiro/specs/release-workflow/first-ci-release.md`）。記録の「手順との食い違い」も、`design.md` と手順書に反映した（2026-10-10）。**残るのは、失敗した job の再実行の読み方の確認である**（失敗が起きなかったので未確認。起きたリリースで確かめる）。

- **進め方（済んだ）**: brief を足さず、常駐 spec をその場で書き換えた（`/kiro-spec-requirements release-workflow` → `/kiro-design release-workflow` → `/kiro-spec-tasks release-workflow`）。古い `research.md` は書き直し、`gap-analysis.md` は削除した。
- **初回のリリース（済んだ）**: v0.3.8 を Opus で実行した（2026-10-10）。7 つの公開先が最初の実行ですべて `published` になり、全 job の再実行ですべて `skipped` になった。後片付け（5 クレートの「Trusted Publishing のみ」・`CARGO_REGISTRY_TOKEN` と `VSCE_PAT` の失効）も済んだ。
- **以後のリリース**: 新しい作業ブランチ（ハーネスのワークツリー）で `/kiro-impl release-workflow` を実行する。失敗した job の再実行の読み方（`design.md`「CI での初回のリリースで確かめること」の 3）を確かめるまでは Opus で実行し、確かめたら Sonnet で実行する（開発者決定 2026-10-10）。
- **席**: リリースの実行中は、`release.ps1`・`release.yml` を触る spec を main へ入れない。

### Existing Spec Updates

- [x] release-workflow -- エージェントの手順を「版の決定 → bump の PR のマージ → タグの push → CI の結果確認・失敗した job の再実行」へ縮める。次のものは CI 側へ移すか、要らなくなる。
  - Resume
  - ScheduleWakeup による再試行
  - 公開の 2 トラック
  - ローカルでのビルド
  - main の CI が全部緑かの確認
  - マージコミット方式で統合する理由（タグが指すコミットを main から到達できるようにするため）を、squash でよいか見直す。
  - `release-ci` からの申し送り: CI での初回のリリースと、その前後の一回限りのセットアップは本更新の後の最初のリリースで行う。手順は `.github/release-ci-setup.md` の 6〜10 節。期限は 2026-12-01（global PAT の廃止）より前。
    - 前提（2026-10-08 に済んだ）: 1〜8 節の一回限りのセットアップはすべて完了。
      - Azure（サブスクリプション・予算アラート・マネージド ID・フェデレーション資格情報 2 件）は `az` で読み戻して確認した。
      - GitHub（environment 2 つ・リポジトリ variables 3 つ）は `gh api` で確認した。
      - Marketplace の Members への追加は、setup-check の再実行で `verify-pat` が成功したことで確認した（run 37777674500）。
      - crates.io の Trusted Publisher ×5 は、ユーザーが画面で設定した。setup-check は crates.io を確かめないので、**初回のリリースが最初の実地確認**になる。`publish-crates` が認証で失敗したら、8 節の値（owner・repo・`release.yml`・`release`）を設定画面と照合する。
      - ID の値は書かない。値はリポジトリ variables とユーザーの手元の記録にある。
    - 合格: 3 公開先が `published`、同じ run の再実行ですべて `skipped`。
    - 運用の注意（セットアップで分かったこと）:
      - Azure のリソースを `az` で作る・変えるには MFA が要る。WAM（Windows のサインイン窓）でログインすると `RequestDisallowedByAzure` で弾かれる。`az config set core.enable_broker_on_windows=false` にしてからブラウザーで `az login` し直す。ワークフローのマネージド ID には関係ない。
      - Marketplace の Members の管理には、公式の CLI が無い（vsce・az・gh のどれも扱えない）。画面から行う。
      - 小さな取りこぼし: setup-check の `verify-pat` が失敗すると、vsce のエラー文（`Access Denied: <ID> needs ...`）にマネージド ID の識別子が含まれ、ジョブのログに出る。「profile ID は summary にだけ書く」方針から漏れている。秘密の値ではないので実害は無い。直さない（2026-10-10 決定）。
    - あわせて見直す（`release-ci` の design.md「Out of Boundary」が本更新へ回したもの）: `.claude/settings.json` の公開系コマンドの許可の整理、`build.yml` に `--locked` を足すか、bump 箇所に `package-lock.json` の版を含めること（release.yml の verify は検査しない。`crates/pasta_sample_ghost/RELEASE.md` の bump の一覧にも無い）。→ 済んだ（2026-10-10）: 手元からの公開の許可を外した。`--locked` は足さない。版の更新に `Cargo.lock` と `package-lock.json` を含めた。
    - `VSCE_PAT` の失効（10 節）は、初回のリリースで Marketplace が Entra ID の経路で `published` になったのを確かめてから行う。値が `release-ci` の会話記録に出ているが、前倒しはしない（ユーザー決定 2026-10-08）。
    - 2026-10-10 の棚卸で足したもの:
      - `steering/workflow.md` の「main の CI 全緑」の確認とマージコミットの例外の記述を、書き換えた手順に合わせる。
      - 認証の失敗の案内文が手順書の見出しを「名前の表」と書いている（正しくは「名前の対応表」）。`.github/scripts/release/publish-vsix.ps1` の 1 か所と `release.yml` の 2 か所。
      - 初回の CI リリースは、`hello-pasta-shell-art` が `release.ps1` を変える前に行うのが望ましい。
  - Dependencies: release-ci（完了）

## 作成支援とマニュアルの保守（2026-10-10 起票）

- `pasta-check-dic-validate` — SSP を起動せずに、ゴーストの `.pasta` の辞書を検査する `pasta_check` のサブコマンド。飛び先の無い Call・選択肢・文中のシーンリンク、モジュール名の衝突、記号を落とした後のシーン名の衝突などを見る。何をエラーにして何を警告にするか、`pasta_check release` の関門にするか、依存の重さ（`pasta_lua` に依存すると、`cargo install pasta_check` が LuaJIT をビルドする）を要件で決める。依存を足すと `Cargo.lock` が変わるので、`hello-pasta-shell-art` の後に置く。マニュアルに章を足す場合は、章の数を決め打ちする検査（`verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`）と `SUMMARY.md` を `getting-started-story-guide` と共有するので、その後に置く。内部クレートに依存させる場合は、公開の手順（`release.yml` の `pasta_check` の公開の段）も触るので、リリースの実行中は入れない。
- `manual-print-media-refs` — マニュアルの印刷用ページ（`print.html`）で、デバッグの章の動画が再生できない。ビルドの後に `print.html` の参照だけを書き換え、検査（`verify-static.mjs`）の例外を外す。
- `manual-shell-guide`（2026-10-10 追加・優先度は低い）— 入門ガイドはシェルを「hello-pasta からフォルダごと写す」と教え、中身の説明は `setup.md` の節「シェルの中身」（ファイルの種類と、辞書との接点がサーフェス番号であること）までである。自分の絵に差し替える所から先を、入門の外に章を置いて書く。急がない。

## 入門ガイドの実機確認で見つかったもの（2026-10-10 起票）

`getting-started-story-guide` の完了時の棚卸で起票した。どれも 2026-10-10 のウェーブには入れていない（次の棚卸で順番を決める）。

- `manual-link-anchor-check` — リンク検証は、本の中のリンクの見出しを見ない。入門ガイドは見出しへのリンクを 44 本持つ。見出しを作る関数は `link-check.mjs` にもうある（サイト・CI）。
- `choice-line-layout` — 9 段目の 1 つ目の選択肢「おやつの話をする」が、問いかけと同じ行に続いて右端で切れる。入門の読者が書いたとおりに動かして踏む（バグ・見え方）。`getting-started-screenshots` より先に直すと、撮り直しが要らない。
- `baseware-virtual-time` — SSP の仮想の時刻は pasta に届かない（pasta は OS の時計で正時を判定する）。入門ガイド 6 段目は「次の正時を待つ」と教えている。
- `boot-surface-without-dic` — 辞書が無い間は何も表示されない。入門ガイドの準備の章は「何も表示されないのが正しい」と教えている。

隣の spec への申し送り（各 brief の「申し送り（getting-started-story-guide より）」の節）: `getting-started-screenshots`（章の形・画像の行が本文の検査に落ちること・撮るときの制約・開発者の確認 4 点）、`hello-pasta-shell-art`（`first-ghost.md` の読み替え・段階表の訂正・7 段目の表に `＞ゴースト終了` が無いこと）、`manual-print-media-refs`（`manual.yml` の古いコメント・`AUTHORING.md` が空いたこと）。

## Specs (dependency order)

- [ ] scene-attribute-store -- シーン属性の実行時の保持・Lua からの読み出し・ファイルレベル属性の継承と上書き・値の型解釈。Dependencies: none
- [ ] failure-output-unification -- シーンが生きている間の実行時の失敗（未定義の参照・見つからない Call など）を、ログとさくらスクリプトの両方へ 1 つの仕組みから出す。既にある 2 つの失敗表記を 1 つにし、`act.lua`・`actor.lua`・`word.lua` の警告を載せ替える（2026-10-10 の棚卸で、スクリプトが止まるエラーを `runtime-error-balloon` へ分けた）。Dependencies: none
- [ ] shiori-test-support-runtime -- `pasta_shiori` の結合テストがコピーして使う古いランタイムの写し（`tests/support/scripts/`）を撤去し、本物のランタイムだけで動かす。回避用の `pasta.toml` の設定とコメントを外す（2026-10-07 棚卸で起票）。Dependencies: none
- [ ] hello-pasta-shell-art -- hello-pasta の女の子・男の子の立ち絵を、fal.ai で作ったイラスト（表情 9 種ずつ・透過 PNG・表情間でずれない）に置き換え、生成物から素材の扱いに切り替える。当たり判定を足し、絵を描くプログラムと配布スクリプトの生成の段を外す。Dependencies: none
- [x] getting-started-story-guide -- 入門ガイドの本文を段階表に沿った物語に書き直し、全編を Claudia が語る（執筆規約に `getting-started` の例外を足す）。段階辞書との逐語照合を章ごとに合わせる（2026-10-10 の棚卸で、スクリーンショットを `getting-started-screenshots` へ分けた）。Dependencies: none
- [ ] manual-print-media-refs -- マニュアルの印刷用ページ（`print.html`）で動画の参照が切れているのを、ビルド後の書き換えで直し、検査の例外を外す（2026-10-10 棚卸で起票）。Dependencies: none
- [ ] pasta-check-dic-validate -- SSP を起動せずに `.pasta` の辞書を検査する `pasta_check` のサブコマンド。飛び先の無い参照・モジュール名とシーン名の衝突などを見る（2026-10-10 棚卸で起票）。Dependencies: hello-pasta-shell-art（`Cargo.lock` を触る順番）, getting-started-story-guide（マニュアルに章を足す場合。章の数を決め打ちする検査と `SUMMARY.md` を共有する）
- [ ] call-attribute-filter -- Call の属性フィルター構文（`＞シーン＆k＝v`・比較演算子・複数条件）と実行時の絞り込み。Dependencies: scene-attribute-store, failure-output-unification
- [ ] runtime-error-balloon -- スクリプトが止まる実行時エラー（500 の応答）を、辞書を書く人に見えるようバルーンにも出す。やるかどうかを要件の最初に決める（2026-10-10 の棚卸で `failure-output-unification` から分割）。Dependencies: failure-output-unification, shiori-test-support-runtime
- [ ] scene-anchor-link -- 台詞の中の `＠？シーン名`（`「表示名」` も付けられる）を、さくらスクリプトのアンカー `\_a` として出す。クリックで、`OnAnchorSelectEx` からそのシーンへ飛ぶ。選択肢の振り分けを共有し、LSP・VSCode の着色とマニュアルまで揃える（2026-10-08 起票）。Dependencies: failure-output-unification, call-attribute-filter
- [ ] getting-started-screenshots -- 新しいシェルで、入門ガイドの各章にスクリーンショットを載せ、8 段目（触られたときの反応）を実機で確かめる（2026-10-10 の棚卸で `getting-started-story-guide` から分割）。Dependencies: hello-pasta-shell-art, getting-started-story-guide
- [ ] manual-link-anchor-check -- マニュアルのリンク検証（`link-check.mjs`）が、本の中のリンクと公開 URL の見出し（`#…`）の実在も見るようにする（2026-10-10 `getting-started-story-guide` の完了時に起票）。Dependencies: none
- [ ] choice-line-layout -- hello-pasta の 9 段目で、1 つ目の選択肢が問いかけと同じ行に続き、吹き出しの右端で切れて見えるのを直す。原因（選択肢の前の改行・辞書・バルーン）の特定から（2026-10-10 同上）。Dependencies: none（`scene-anchor-link` と `sakura_builder.lua` を触る順番に注意）
- [ ] baseware-virtual-time -- SSP の「現在時刻の仮想的変更」を pasta の時報に効かせ、正時を待たずに確かめられるようにする。SSP が仮想の時刻を SHIORI に伝えているかの調査から。外から起こした `OnTalk` でチェイントークが続かない件も扱う（2026-10-10 同上）。Dependencies: none
- [ ] boot-surface-without-dic -- 辞書や `＊OnBoot` が無いゴーストは起動しても立ち絵が出ない。既定で立ち絵を出すかどうかを、やるかどうかから決める（2026-10-10 同上）。Dependencies: none
- [ ] manual-shell-guide -- マニュアルに、シェル（見た目）の説明を足す。見本のシェルを自分の絵に差し替える・表情を足す・当たり判定を足す・バルーンの扱いを、辞書と噛み合う所を中心に書く。優先度は低い（2026-10-10 開発者の指示で起票）。Dependencies: hello-pasta-shell-art（新しいシェルと当たり判定を題材にする）

## バックログ（brief なし・保留）

動機（作者が困った事例・計測した性能問題）が生じたら、その動機から brief を起票する。

### 文法の将来項目（旧文法仕様の申し送り）

- シーンのパラメータ（R1）— シーン宣言での名前付きパラメータ。Call の位置引数とシーン引数 `＄０`… で代替できる。
- アクタースコープ内コードブロックの用途（R2）— アクター固有のイベントハンドラ・状態管理関数。現行は `lua` ブロックの値・関数がアクター付きの単語参照から使えるだけ。
- Call の戻り値と変数代入（R3）— `＄x＝＞シーン`。DSL では定義せず、ランタイム設計の領域。`＄x＝＠＊関数（）` で代替できる。
- さくらスクリプトの `\]`・`\%`（R4）— 角括弧内の `]` と `\%` を DSL から書けるようにするか。現行は `\]` を特別扱いせず、`\%` はパースエラー。
- 継続行内の空行の糖衣構文（R6）— 行継続の途中の空行を改行として出力する。現行は何も出力しない。
- 単語参照への属性フィルター（`＠単語名＆category＝food`）— 単語に属性を付ける構文が無い。`call-attribute-filter` の後に検討する。
- 既定の別名の追加（`時報 → OnHour` など。2026-10-10 棚卸、`scene-name-alias` の範囲外）— 既定は `会話 → OnTalk` の 1 件だけにした。

### トランスパイラ最適化の候補

いずれも未実装で、計測された性能問題は無い。現行の生成時最適化はマニュアルの [内部設計: トランスパイルパイプライン](https://ekicyou.github.io/pasta/internals/transpiler.html#生成時最適化) が書く。

- 定数畳み込み — トランスパイル時に定数式を評価する。
- デッドコード削除 — 到達不能コードを生成しない。
- インライン展開 — 小さなローカルシーンを呼び出し元に展開する（ソースマップ・デバッグの忠実さと衝突する）。
- 単語プリフェッチ — 単語辞書を先読みする。

### その他

- `pasta_novel` アダプタ（ノベルゲーム宿主）— 遠い将来。Phase 7 の宿主非依存コアと presentation event stream 契約が土台になる。
- デバッグポートの他プロセスによる奪取（即時修正で判明）— 相手が `SO_REUSEADDR` を立てて bind する場合まで防ぐには、Windows の `SO_EXCLUSIVEADDRUSE` が要る。ゴースト同士の二重 bind は防いだ。
- `pasta_shiori` の `util/hglobal/windows_api.rs` の `string_to_multibyte`（即時修正で判明）— `@enc` と同じ 65001 で不正になるフラグを渡すが、テストからしか呼ばれない。`pasta_lua` の `encoding/windows.rs` と同じ先頭の分岐（65001 ならバイト列をそのまま返す）を足せば数行で直る。テスト専用なので、消す案もある。次の `review-improvement-loop` の実行で決める。
- 式の木の優先順位（2026-10-07 棚卸、`dsl-codegen-runtime-safety` の調査）— `pasta_dsl` の `parse_action.rs` の `build_left_assoc_expr` は優先順位なしで木を作り、コード生成（`expr_gen.rs`）が組み直している。AST を直接使う別の利用者が出たら直す。
- 別ファイルの同名シーンのデバッガ索引（2026-10-07 棚卸、`scene-identity-format` の範囲外）— ファイルをまたぐ同名シーンの索引と通し番号は、既存の制約として残した。
- タグの読み取りの制約（2026-10-07 棚卸、`paragraph-break-tag-only-talk` の範囲外）— `\nHello` をタグとして読む、`\_?…\_?` の中のタグも読む、引数の中の `\]` を扱わない、BudouX の行幅にエスケープと囲みを数えない。上の R4 と一部重なる。
- 閉じていない 1 行の引用（2026-10-07 棚卸、`dsl-literal-fixes` の範囲外）— `＠w：「abc` と、行をまたげる `sakura_body` の規則はそのままにした。
- 並列負荷で時々落ちる TCP のテスト（2026-10-07 棚卸、`act-token-grouping-fix`・`scene-identity-format` の実装メモ）— `runtime_toggle_e2e_*`・`hook_panic_*`。
- マニュアルのメニューのテストが持つ mdBook の写し（2026-10-10 棚卸、`manual-claudia-theme` の実装メモ）— `book/tools/theme-menu-test.mjs` は、mdBook 0.5.4 の `book.js` のキー操作の処理を写して持つ。mdBook の版を上げるときに合わせる（CI は 0.5.3 に固定）。
- 配布版の hello-pasta に会話の変化を足すときの指針（2026-10-10 棚卸、`hello-pasta-tutorial-stages` の実装メモ）— emo2 開発からの知見「1 つの場面に 3 通り」は、段階の形を固定した辞書では満たせない。配布版に変化を足すことになったら、その動機から起票する。
- budoux の自動改行の禁則（2026-10-05、ghost_dev「emo2 開発」からの申し送り）— `line_breaker.rs` の `break_lines_impl` は禁則を見ない。BudouX の語の区切りの直前で改行するだけである。
  - 方針は JIS X 4051 どおりとする。リーダー（`‥…`）は行頭に置いてよい。並びの途中では分けない。
  - この方針では、申し送りの実例「イイジャン！／‥‥ええと、」は正しい組版になる。
  - 句読点・`！？`・閉じ括弧の前、開き括弧の後ろでは、BudouX の既定の日本語モデルは区切らない（手元で確認）。
  - BudouX が行頭禁則の字の直前（または行末禁則の字の直後）で区切る実例が出たら、その実例から起票する。その場合は、既存の `[talk]` の `chars_*` を使った追い出しを第一候補とする。
