# Roadmap

## 概要

pasta は、日本語 DSL（Pasta DSL）で書いた辞書を Lua へトランスパイルし、組込 LuaJIT で実行する「伺か」の SHIORI（`pasta.dll`）と、その周辺ツール（`pasta_check`・LSP・VSCode 拡張・利用者マニュアル）からなる。Phase 1〜11 で、プロパティアクセス・監査・マニュアル・ソースレベルデバッグ・アクターモデル駆動・配布物の形・現行実装の不具合の一掃までを完了した。2026-10-09 までに、リリースの CI 化、シーン名の別名、式の中の値なしの扱い、マニュアルの着せ替え、段階ごとに学べる hello-pasta の辞書も入った（下の「完了フェーズ」）。2026-10-11 までに、hello-pasta の新しいシェル、9 段目の選択肢の表示の直し、マニュアルの見出しへのリンクの検査も入った。2026-10-15 に、リリースの本文を見どころの紹介にした。2026-10-17 までに、撫で・つつきの反応のショートカット、実行時の失敗の出力の一本化、SSP を起動しない辞書の検査、シェルの説明の章、印刷用のページの廃止、結合テストの古いランタイムの写しの撤去も入った。同じ日に、止まる実行時エラーのバルーン表示と入門ガイドの絵 16 枚が入り、v0.4.0（主題「入門者にやさしく」）を出した。

現在の主題は 3 つある。

- **0.4.1: 致命的でない不具合**。0.4.0 に入れずに回した不具合を直す。今の範囲は、辞書の検査の端のケース（`pasta-check-validate-edges`）と、入門ガイドを SSP で確かめたときに見つかったずれ（`tutorial-ssp-gaps`。2026-10-17 開発者の決定で足した）の 2 本に、窓際のぱすたさん（ghost_dev）からの要望の 2 本（アクターごとの表示の装い `actor-display-style`・縦書きのバルーンの列の扱い `vertical-balloon-lines`。2026-10-18 開発者の決定）を足した 4 本である（下の「0.4.1 の範囲」）。
- **Phase 13: areka でノベルゲーム（0.5.0）**。2026-10-10 の discovery で、合格ライン・通り道・台本の書き方・巻き戻しの方式を決め、spec を起こした（2026-10-17 の時点で 11 本）。属性セマンティクス（シーン属性の保持と読み出し・Call の属性フィルター）も、この道筋に入った。
- **Phase 11 の残り**: 文中のシーンリンク（`scene-anchor-link`）。どの版に入れるかは、まだ決めていない（下の棚卸の「開発者の判断が要るもの」）。

Phase 12（初心者向けの入門ガイド）は、2026-10-17 の入門ガイドの絵で完了した（「完了フェーズ」）。

このほか、リリース手順（spec `release-workflow`）の CI に合わせた書き換えと、CI での初回のリリース（v0.3.8）は済んだ（2026-10-10）。リリース手順は `release` スキル（`/release`）へ移し、spec `release-workflow` は完了にした（2026-10-11）。

**優先順位（開発者の方針）**: 0.4.0（主題「入門者にやさしく」）は 2026-10-17 に出した。次の節目は 0.4.1（致命的でない不具合と、窓際のぱすたさんからの要望の 2 本。2026-10-18）で、その次の 0.5.0 の目標は、areka でノベルゲームが作れるようになること（2026-10-10）。ウェーブは、今のゴールの範囲の spec だけで組む（2026-10-16）。致命的な不具合は、見つかりしだい割り込ませる。致命的でない不具合は 0.4.1 に回す（2026-10-17）。

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
- **入門ガイド**: Claudia が全編を語る（`getting-started` に限った執筆規約の例外）。段の章と段階辞書を逐語で照合する（`book/tools/tutorial-check.mjs`）。辞書を変える spec は、同じ変更で入門の章の作例を直す。絵は `book/SCREENSHOTS.md` の手順で撮り直す。
- **マニュアルの見た目**: mdBook のまま「Claudia のマニュアル」。台詞は顔アイコン付きの部品で書く（記法は `book/AUTHORING.md`）。印刷用のページ（`print.html`）は作らず、章の印刷はテーマが受け持つ。
- **実行時の失敗**: シーンが生きている間の失敗は、`pasta/failure.lua` の `FAILURE.report` が、その場所に `【種類：対象（説明）】` の表記とログ 1 行を出す。切り替えは pasta.toml の `[logging] balloon_warning`。新しい種類は `FAILURE.LABEL` に足す。スクリプトが止まるエラーは `pasta/shiori/stop.lua` が受け、`【スクリプトのエラー：…】`・`【時間切れ：…】` の表記を `200 OK` と `X-Error-Reason` の行で返す（`balloon_warning = false` なら、これまでどおり 500）。
- **辞書の検査**: `pasta_check validate` が pasta.dll と同じ規則で辞書を検査し、`pasta_check release` の段 4 の関門と VSCode 拡張の保存のときの診断にもなる。検査を足すときは報告の種類 `FindingKind` に足す。
- **マニュアルの章の数**: 章を足す spec は、章の数を決め打ちする 3 つの検査（`book/tools/verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`）を合わせる（2026-10-17 の時点で 63 章）。

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
| 12（シェル） | hello-pasta の立ち絵をイラストにして当たり判定を足した。9 段目の選択肢を 1 行に 1 つずつ出す（設定 `choice_newlines`）。マニュアルのリンク検証が見出しの実在まで見る | hello-pasta-shell-art, choice-line-layout, manual-link-anchor-check |
| 11 | 実行時の失敗の出力の一本化（`【種類：対象（説明）】`・`pasta/failure.lua`・`[logging] balloon_warning`）、結合テストの古いランタイムの写しの撤去、止まるエラーのバルーン表示（`pasta/shiori/stop.lua`・`200 OK` と `X-Error-Reason`） | failure-output-unification, shiori-test-support-runtime, runtime-error-balloon |
| 12（仕上げ） | 撫で・つつきの反応を決まった名前のシーンで書けるようにし、入門の 8・9 段目を書き直した。マニュアルにシェルを自分の絵にする章を足した。入門ガイドに実機の絵 16 枚を載せ、撮り直しの手順（`book/SCREENSHOTS.md`）を置いた | touch-poke-shortcuts, manual-shell-guide, getting-started-screenshots |
| — | SSP を起動しない辞書の検査（`pasta_check validate`・`release` の段 4 の関門・VSCode 拡張の保存のときの診断） | pasta-check-dic-validate |
| — | 印刷用ページの動画の参照の直しと、印刷用ページの廃止（章の印刷はテーマの `@media print`） | manual-print-media-refs, manual-print-page-removal |
| — | リリースの CI 化（タグの push で crates.io・Marketplace・GitHub Release へ公開。認証は OIDC、成果物の git 追跡を解除） | release-ci |
| — | リリース手順（版の決定 → 版の更新の PR → リリースタグの push → リリース CI の結果の確認）。リリース CI に合わせて書き換え、CI での初回のリリース（v0.3.8）を済ませ、手順を `release` スキル（`.claude/skills/release/`）へ移した | release-workflow |
| — | リリースの本文を、見どころの紹介（Claudia の口調）にし、全件の一覧を Assets に添える | release-notes-highlights |
| — | 動的単語参照 `＠＄`（旧文法仕様の将来項目 B2） | dynamic-word-reference |

ロードマップを置く前の初期開発（DSL・トランスパイラ・ランタイム・SHIORI・alpha リリースなど）の spec も `.kiro/specs/completed/` にある。

### 常駐 spec（再実行型・`completed/` へ移さない）

- `review-improvement-loop` — レビュー領域 × 7 次元の改善ループ

### 却下（2026-10-01）

- `pasta-sstp-live-output`（SSTP/`\![raise]` による push 出力）— 目的はシーンキックで達成済み。新しい動機が生じたら、その動機から新規に起票する。
- `pasta-authoring-window`（`*.pasta` 編集・プレビュー専用ウィンドウ）— VSCode で実現済み。VSCode 非依存のオーサリング環境が必要になったら再検討する。

### 人手の確認が残っている項目

- `pasta-check-bundled-balloon` を v0.3.7 で公開したことの、emo2 開発セッションへの連絡 — 実施の有無はリポジトリから確認できない。

## 0.4.1 の範囲（2026-10-17 開発者の決定）

0.4.0（主題「入門者にやさしく」）は 2026-10-17 に出した。入門ガイドの絵、hello-pasta の新しいシェル、実行時の失敗の出力の一本化、止まるエラーのバルーン表示、SSP を起動しない辞書の検査が入った。範囲と経緯は、git の履歴と各完了 spec にある。

0.4.0 のリリースが遅れ続けたので、**致命的でない不具合は 0.4.0 に入れず、0.4.1 に回した**。

| 順 | spec | 要件定義 | 規模 | 始められる時期 |
| -- | ---- | -------- | ---- | -------------- |
| 1 | pasta-check-validate-edges | Opus | 5〜7 | 今すぐ（2026-10-17 のウェーブ） |
| 1 | tutorial-ssp-gaps | Opus | 7〜11 | 今すぐ（2026-10-17 のウェーブ。2026-10-17 開発者の決定で 0.4.1 に入れた） |
| 1 | actor-display-style | Opus | 8〜12 | 完了（2026-10-18。2026-10-18 開発者の決定で 0.4.1 に入れた） |
| 1 | vertical-balloon-lines | Opus | 8〜12 | 今すぐ（同上。実装の並走の条件は下） |
| 2 | 0.4.1 のリリース（`release` スキル） | Opus | — | 上の全部の後 |

- **0.4.1 に入れたもの**: `pasta-check-validate-edges`（辞書の検査の端のケース 3 つと代用品の置き換え。誤った報告は出すが、検査そのものは働く）。`tutorial-ssp-gaps`（SSP の仮想の時刻を時報に効かせるか・辞書が無いときの起動の立ち絵。どちらも、やるかどうかから決める。入門ガイドと SSP のずれで、0.4.0 の主題の続き）。
- **2026-10-18 に足したもの**（開発者の決定）: 窓際のぱすたさん（ghost_dev）からの要望の 2 本。`actor-display-style`（シェルの無い相手役を、アクターごとの装いで見分ける）と `vertical-balloon-lines`（縦書きのバルーンの列の扱い。budoux の数え直しは、横書きでも起きている不具合の直しを含む）。経緯は下の「窓際のぱすたさんからの要望」。
  - 並走: 2 本は `sakura_builder.lua` の別の関数を触る（`actor-display-style` はアクターの切り替え `emit_actor_switch`、`vertical-balloon-lines` は選択肢どうしの間 `choice_break`）。要件定義と設計は並走できる。実装も並走できるが、`vertical-balloon-lines` がトークンをまたいだ字数の持ち越し（`BUILDER.build` の繰り返しに状態を足す）を採ったら、実装は 1 本ずつにする。マニュアルの `reference/pasta-toml.md` は両方が触る（後から入る側が rebase で合わせる）。
  - 2026-10-17 のウェーブの 2 本とは、ソースが重ならない。ただし `actor-display-style` が装いのキーの書き間違いを `pasta_check validate` に報告させると決めたら、`pasta-check-validate-edges` が main に入ってから実装する。
- **これから見つかる不具合**: 致命的（起動しない・辞書の内容が失われる・配布物が壊れる・リリースが通らない）なら、すぐに割り込ませる。それ以外は 0.4.1 に置く。
- **0.5.0 との関係**: 0.5.0 の先頭の `novel-areka-contract`（文書だけ）と `scene-attribute-store` は、0.4.1 の 2 本とファイルが重ならない。並べるには、0.5.0 を今のゴールに含める開発者の決定が要る（下の棚卸の「開発者の判断が要るもの」）。
- **まだ版が決まっていないもの**: `scene-anchor-link`。
- `tutorial-ssp-gaps` のうち起動の立ち絵の分は、2026-10-16 開発者の決定で優先度を下げた（「何も書かなければ何も表示しない」が既定の動き）。やるかどうかは、要件定義の最初に決める。

### 0.5.0 の目標（2026-10-10 開発者の決定）

別リポジトリで開発中の areka で、ノベルゲームが作れるようになること。pasta の側に何が要るかは、2026-10-10 の discovery で洗い出した。合格ライン・決めたこと・spec の割り方と順番は、下の「Phase 13: areka でノベルゲーム（0.5.0）」にある。

## 棚卸（2026-10-17・main `b4164100`）

前回（2026-10-17・main `e9b321e0`）の後に、前回のウェーブの 2 本（止まるエラーのバルーン表示 `runtime-error-balloon`・入門ガイドの絵 `getting-started-screenshots`）が入り、v0.4.0 を出した（リリース CI の run 39024925990。全部の job が 1 回目で通った）。ほかに、zip を 9.0 に上げた直しと、リリースの本文の直しが入った。未完了は 13 本（常駐の `review-improvement-loop` を除く。この棚卸で 1 本分けた）、完了は 171 本。作業中のブランチは無い（ほかのワークツリーとリモートのブランチは、どれも main に入った後のもの）。前回の記録は git の履歴にある。

今のゴールは 0.4.1 である（0.4.0 を出したので、いちばん近い版）。範囲は `pasta-check-validate-edges` の 1 本だけだった。この棚卸の報告の後に、開発者が `tutorial-ssp-gaps` を足した（2 本）。0.4.0 を出した後の棚卸として、ゴールの後の spec も、0.5.0 の範囲として測り直した。`runtime-error-balloon` が、場面の関数の頭（`act:init_scene` の引数）・`act.lua`・止まるエラーの応答（200 を返す）を変え、Phase 13 の前提が動いたためである。測り直したのは 0.5.0 の土台の 6 本と Phase 11 の残りの 4 本で、合わせて 10 本。遠い 3 本（`novel-save-slots`・`hello-novel-sample`・`manual-novel-guide`）は、前提が変わっていないので、分割の申し送りを足しただけにした。

- `pasta-check-validate-edges`（5〜7）は、zip の直しとも `manual.yml` の直しとも重ならない。0.5.0 の先頭の 2 本とも、ファイルを共有しない。
- `tutorial-ssp-gaps` は、要件定義を Fable から Opus に下げた。SSP の仮想の時刻が SHIORI に届くかは、実機の記録で確かめれば決まり、決め直せる。今の動きは筋が通っているので、種別は不具合でなく機能と文書とした。
- `scene-anchor-link` も Opus に下げた。記法・`\_a` の出力・飛び先が無いときの 204 は、2026-10-08 に決めてある。残る選び方は出力の層の話で、2 つに絞れている。18〜20 タスク。
- `call-attribute-filter`（19〜21）・`scene-attribute-store`（15〜18）・`cue-command-runtime`（16〜20）・`novel-talk-flow`（17〜21）は Fable のまま。どれも、後の spec と作者が上に乗る文法か取り決めで、推奨の無い問いが 3 つ以上絡む。
- `shell-element-click` は、先に要るものが `novel-areka-contract` だけになった。完全一致の場面選びは `touch-poke-shortcuts` が入れてある（`SEARCH:search_scene_exact`・`SCENE.co_exec_exact`）。

### 即時修正（spec なし・本棚卸で実施して閉じた）

- `steering/product.md` — 完了の数（171）・現行の版（v0.4.0）・現在地。「全ファイル < 600 行」を「分解した時点で」にした（`pasta_check` の `balloon.rs` が 1,070 行ある）。
- `steering/tech.md` — 理由つきの時間切れの応答を、今の形（表記を `200 OK` と `X-Error-Reason` で返す。表記を出さない設定なら 500）に直した。
- `steering/structure.md` — 600 行の目安の行に、超えている `balloon.rs` を書いた。
- brief — `pasta-check-validate-edges`（行番号）、`tutorial-ssp-gaps`（測定のモデル）、`novel-areka-contract`（後に続く spec の数・0.4.0 → 0.4.1）、`shell-element-click`（完全一致の場面選びはもうある・先に要るもの）。各 brief の再測定の節に、古くなった行番号を書き出した。
- この roadmap — 完了した 2 本を「完了フェーズ」へ畳み、Phase 12 の節（完了）と 0.4.0 の道筋を消して、0.4.1 の範囲の表にした。入門ガイドの決まりは「前提として確定している方針」へ移した。
- 確かめて閉じたもの: `hello-pasta-shell-art` が変えたリリース CI の配布物の検査は、v0.4.0 のリリースで通った。配布したゴーストで失敗の表記を既定で出すかは、`runtime-error-balloon` の要件で決まった（既定は出す。配る前に `false` にするよう、マニュアルが書く）。マニュアルの章の数は 63 のままで、3 つの検査と合っている。

### 統合・分割・改名

- **分割** `novel-checkpoint-rollback`（21〜25）→ `novel-variable-scope`（8〜10）と `novel-checkpoint-rollback`（13〜15）。変数の保存の範囲（セーブごとか全セーブ共通か・今の `＄＊x` との互換・保存できない値）を前に出した。2 本はファイルを共有しない。`novel-variable-scope` は先に要るものが無く、`novel-talk-flow` を待たずに始められる。どちらも要件定義は Fable（変数の保存の形は、今のゴーストの `＄＊x` が上に乗る。巻き戻しは、Rust の 5 秒の時間切れ・Lua のコルーチン・SHIORI の応答にまたがる時の流れの取り決め）。
- 分けなかったもの: `call-attribute-filter`（19〜21。2026-10-04 に一度分けたので、20 をわずかに超えるだけでは分けない）、`novel-talk-flow`（17〜21。超えたら出力の分と場面の流れの分に分ける案を brief に書いた）、`cue-command-runtime`（16〜20。分けてもファイルが重なり、並走できる分は増えない）、`scene-anchor-link`（18〜20。超えたらエディタの分を出す案を brief に書いた）。

### 起票と、バックログに残したもの

- 起票: なし。
- バックログは全部そのまま残した（どれも、動機の事例・使う人・計測が無い）。完了した spec の範囲外から 2 行足した（止まるエラーの表記の粗さ、バルーンが左端で折り返るときの見え方）。

### 要件定義のモデル

今のゴール（0.4.1）の 2 本はどちらも Opus。`pasta-check-validate-edges` は端のケースの直しで、文法・保存の形・公開の Lua の約束のどれにも触らない。`tutorial-ssp-gaps` は、SSP の仮想の時刻が SHIORI に届くかを実機の記録で確かめれば決まり、決め直せる。ゴールの後では、`scene-attribute-store`・`cue-command-runtime`・`call-attribute-filter`・`novel-talk-flow`・`novel-variable-scope`・`novel-checkpoint-rollback` が Fable である（理由は各 brief の再測定の節）。

### ウェーブ（2026-10-17）

今のゴール（0.4.1）の 2 本を、両方とも並べる（`tutorial-ssp-gaps` は 2026-10-17 開発者の決定で 0.4.1 に入れた）。どちらも先に要るものが無く、互いのソースもマニュアルのページも重ならない。

| spec | 子孫 | 種別 | 規模 | 要件定義 |
| ---- | ---- | ---- | ---- | -------- |
| pasta-check-validate-edges | 0（後は 0.4.1 のリリース） | バグと後片付け | 5〜7 | Opus |
| tutorial-ssp-gaps | 0（後は 0.4.1 のリリース） | 機能と文書（入門ガイドと SSP のずれ） | 7〜11 | Opus |

開始コマンド（`/kiro-next` の報告と同じ形。俯瞰の頁が、この塊をコピーのボタンつきで出す）:

```
Wave 2本
Opus
/kiro-start pasta-check-validate-edges
/kiro-start tutorial-ssp-gaps
```

同じウェーブに置くときの約束:

- `pasta-check-validate-edges` が持つのは、`crates/pasta_check/src/validate/`（`mod.rs`・`report.rs`・`discovery.rs`、フォルダを報告するなら `input.rs`）、`validate_cmd.rs` の集め方、VSCode 拡張の `ghostValidator.ts`（入力の形を変えるなら `wasmBridge.ts` と `pasta_lsp` の受け渡し）、そのテスト、マニュアルの `debug/validate.md`、代用品の置き換えで `manual.yml` と `tutorial_stages_test.rs`。`crates/pasta_lua`・`crates/pasta_shiori` と入門の章は触らない。
- `tutorial-ssp-gaps` が持つのは、`pasta_shiori` の `lua_request.rs`（時刻の受け渡し）、`pasta_scripts/pasta/shiori/event/` の `virtual_dispatcher.lua`・`boot.lua`（外から起こした `OnTalk` の件をやるなら `event/init.lua`。止まるエラーの見張り `STOP.hold_handler` は、処理を呼ぶ直前のまま保つ）、そのテスト、入門の章（`getting-started/06-hour.md`・`setup.md`・`11-jump.md`）と `crates/pasta_sample_ghost/STAGES.md`、撮り直す絵。`crates/pasta_check`・`editors/`・`manual.yml`・`tutorial_stages_test.rs` は触らない。
- どちらも章を足さない見込み。足すなら、3 つの数（`verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`）を 64 に合わせる。スキル references は、後から入る側が再生成する。
- 0.5.0 を今のゴールに含めると決めたら、`novel-areka-contract`（文書だけ。章を足すなら 3 つの数を 64 に）と `scene-attribute-store`（宣言行の属性のパース・`scope_gen.rs`・`scene.lua`・`pasta_core` のシーン登録）を並べられる。どちらも `pasta_check` と `editors/` を触らない。そのときは、`scene-attribute-store` が `partial.rs` と VSCode の文法定義を触らない約束を足す（`novel-variable-scope` を次のウェーブに置くため）。

次（今のゴールの中）: 0.4.1 のリリース（`/release 0.4.1`）。

ゴールの後（0.5.0 の順）: `novel-areka-contract`（先頭・子孫 約 6）、`scene-attribute-store`（先頭・子孫 約 8）、`novel-variable-scope`（先頭・子孫 約 4）、`shell-element-click`（`novel-areka-contract` の後）。版の決まっていないもの: `scene-anchor-link`（`call-attribute-filter` の後）。

### 開発者の判断が要るもの（ウェーブは止めない）

- **0.5.0 を今のゴールに含めるか**（0.4.1 と並べるか）。含めないと、ウェーブは 1 本のままで、0.4.1 を出すまで 0.5.0 は始まらない。推奨は含めること。areka の側が早く動けるよう、`novel-areka-contract` を先に出す方針（2026-10-10）とも合う。含めるなら、ウェーブは `novel-areka-contract`（Opus）と `scene-attribute-store`（Fable）を足した 3 本になる。
- **`scene-anchor-link` の版**。推奨は 0.5.0 の後（新しい文法なので、不具合を直す 0.4.1 に合わない。ノベルの合格に要らず、文法・`act.lua`・出力の組み立てを 0.5.0 の 5 本と共有する。`pasta-check-validate-edges` とは辞書の検査の本体を共有する）。`tutorial-ssp-gaps` は、2026-10-17 開発者の決定で 0.4.1 に入れた。
- `SOUL.md` §2・§5 の「Wait/Sync はマーカーのみ」「全出力は yield」— 構想の文書なので、直すかどうかは開発者が決める。
- `steering/structure.md` の作り直し（`pasta_shiori` の木と、`pasta_scripts/pasta/` の木が無い。`failure.lua`・`shiori/stop.lua` も載っていない）— `/kiro-steering` で直す。
- `pasta_shiori` の `string_to_multibyte`（テスト専用）を直すか消すか — 次の `review-improvement-loop` の実行で決める（バックログのとおり）。
- ghost_dev の emo2 開発のフォルダに、入門の撮影のときの `profile/` の保存データとログが残っている（`getting-started-screenshots` の実装メモ）。リポジトリの外なので、片付けるかは開発者が決める。

## Phase 11 の残り: 属性セマンティクスとシーンリンク

### 境界戦略

- **分割理由**: シーン属性を 2 つに分けた（2026-10-04）。保持・継承・読み出し（`scene-attribute-store`）と、Call の属性フィルター（`call-attribute-filter`）。どちらも 0.5.0 の道筋に入った（下の「Phase 13」）。実行時の失敗の出力の 2 本（`failure-output-unification`・`runtime-error-balloon`）は、2026-10-17 までに完了した。
- **共有接点**: `crates/pasta_lua/pasta_scripts/pasta/act.lua` は 1 ウェーブに 1 spec だけが持つ。`call-attribute-filter`・`scene-anchor-link` が触る。`crates/pasta_lua/src/code_gen/element_gen.rs` も 1 ウェーブに 1 spec だけで、`call-attribute-filter` と `scene-anchor-link` が触る。`scene-attribute-store` と `call-attribute-filter` は、`scene.lua` と `pasta_core` のシーン登録（`scene_registry.rs`・`scene_types.rs`）を共有するので、この順に置く。選択肢の出力（`pasta_scripts/pasta/shiori/sakura_builder.lua`）と選択肢の振り分け（`shiori/event/choice_select.lua`）は、`scene-anchor-link` と Phase 13 の spec が共有する。辞書の検査（`crates/pasta_check/src/validate/mod.rs`）に検査を足す spec（`call-attribute-filter`・`scene-anchor-link`）は、`pasta-check-validate-edges` と同じウェーブに置かない。
- **シーン名の別名との関係**: 別名の解決は検索の入口（`search/context.rs`）でキー正規化の前に行う。`scene-attribute-store`・`call-attribute-filter` は、別名を当てた後の名前で属性を扱う。
- **結合テスト**: `pasta_shiori` の結合テストは、自己展開した本物のランタイムで動き、回避を書かない（`shiori-test-support-runtime`）。

### ウェーブ構成

| Wave | spec（並走可） | 種別 | ソースの持ち場 |
| ---- | -------------- | ---- | -------------- |
| 0.5.0 の先頭 | scene-attribute-store | 機能 | 宣言行の属性のパース（`parse_scene.rs`・`ast/scene.rs`）、`scope_gen.rs`・`context.rs`・`transpiler.rs`、`scene.lua`、`finalize.rs`、`pasta_core` のシーン登録（`scene_registry.rs`・`scene_types.rs`）、見本の期待値ファイル、マニュアル（`block-structure.md`・`script-api.md`・`registry-search.md`・`transpiler.md`） |
| 0.5.0（`scene-attribute-store` の後） | call-attribute-filter | 機能 | Call の文法（`grammar.pest`・`parse_action.rs`・`ast/action.rs`・`partial.rs`）、`element_gen.rs`（Call）、`act.lua`（`call`・`find_handler`）、`scene.lua`（`SCENE.search`・`co_exec`）、`search/`、`pasta_core` の `scene_table.rs`・`scene_types.rs`、LSP（`visit_action.rs`）、VSCode の文法定義、`pasta_check validate`（候補が複数残る検査を足すなら） |
| 版は未定 | scene-anchor-link | 機能 | `call-attribute-filter` と同じ文法・生成・`act.lua`・エディタのファイルに加え、`sakura_builder.lua`、`shiori/event/choice_select.lua`、`sakura_script/`、VSCode 拡張の `wordRefDecorator.ts`、`pasta_check validate`（`references.rs`） |

Wave 1〜3 の 12 本と、先行させた `scene-name-alias`・`expr-nil-coercion`、失敗の出力の 3 本（`failure-output-unification`・`shiori-test-support-runtime`・`runtime-error-balloon`）は完了した（「完了フェーズ」の Phase 11）。

### 文中のシーンリンク（2026-10-08 起票）

台詞の中の語を Wikipedia のリンクのようにし、クリックでそのシーンへ飛べるようにする（`scene-anchor-link`）。

- **方式**:
  - 記法は `＠？シーン名`（`＠単語　` と同じく、空白か改行で終える）と `＠？シーン名「表示名」`。
  - 出力はさくらスクリプトの `\_a`（アンカー）で、`\q` ではない。`\q` だと、トーク全体が選択肢待ちになるため。
  - クリックは `OnAnchorSelectEx` を、選択肢の振り分け（`choice_select.lua`）と同じ規則で受ける。
  - `【】` で囲む案は、文法として唐突なので採らなかった。
- **飛び先が無いとき**: 選択肢と同じく 204 を返す。失敗の見せ方は `pasta/failure.lua` の出口に乗せる。飛び先の無いリンクの検査は、この spec が `pasta_check validate` に足す（選択肢の飛び先の検査 `validate/references.rs` を使える）。
- **ウェーブ**: `grammar.pest`・`parse_action.rs`・`element_gen.rs`・`act.lua`・VSCode の文法定義を、`call-attribute-filter` と共有する。そのため 1 本ずつ入れる。

## Phase 13: areka でノベルゲーム（0.5.0）

2026-10-10 の `/kiro-discovery` で起こした（main `88c4bc0e`）。国産 10 系統・欧米 9 系統のノベルゲームのスクリプト仕様を調べ、pasta と areka の現在地と照らした。調査ノートは `.kiro/specs/novel-areka-contract/` の `research-jp-engines.md` と `research-western-engines.md` にある。

### 決めたこと（開発者の決定）

| 論点 | 決定 |
| ---- | ---- |
| 合格ライン | 市販品の体裁。遊ぶ人は、クリック送り・セーブとロード・バックログと巻き戻し・既読スキップ・オート・設定・タイトル画面を使える |
| 通り道 | SHIORI とさくらスクリプトのまま。演出は `\![名前,引数…]` に載せる。pasta 専用の通り道（`pasta_novel` アダプタ・presentation event stream）は作らない |
| 画面 | 複数のシェルの組み合わせで作る。背景・登場人物・トークの出る場所が、それぞれシェル。普通の窓と全画面は areka に依頼する |
| アクター | 背景もトークの場所も、キャラと同じくアクターとして扱う。状態はサーフェス（例: `教室と窓：＠稲光`） |
| 台本の書き方 | 脚本の 3 層に当てる。柱＝`＊` と `＆属性`、ト書き＝`！` 行、台詞＝アクション行。新しい文法は作らない |
| ト書き | `！` 行を、実行時に `\![…]` として流す。dola が AST を直に読む構想から切り替えた。pasta は命令の名前を知らない |
| 柱 | 柱に書いた `＆名前：値` は、場面に入ったときに自動で打たれる `！名前（値）` である。流すのは、語彙の表にある名前だけ |
| フラグによる分岐 | Call の属性の絞り込みで書く。属性の倉は 1 つで、読み手は演出と絞り込みの 2 人。モードの切り替えは作らない |
| バックログと巻き戻し | 一覧は areka が持つ。pasta は台本に目印の ID を埋める。areka は、選ばれた行の直近の ID をイベントで知らせる。pasta は、その時点の状態へ戻し、控えた台本を目印の位置から返す。場面を走り直さない |
| 表示の進み具合に関わる機能 | クリック送り・早送り・オート・既読とスキップは areka が持つ。どこまで表示したかを知っているのは areka だけ |
| システム画面 | areka が出すのは部品だけ（エレメント単位のクリック・シェルの中のバルーン・サーフェスの階層）。タイトル・セーブ・ロード・設定は、シェルとして組み、pasta の場面が動かす。見本を同梱する |
| 0.5.0 の外 | CG 鑑賞・回想・音楽室。全セーブ共通の変数があれば作者が組めるので、専用の仕組みは作らない |

### 境界戦略

- **分割理由**: 持ち場で分けた。areka との取り決め（文書だけ）、ト書きと柱を流す仕組み（2026-10-17 に 1 本にまとめた）、ノベル向けの出力、変数の保存の範囲（2026-10-17 に巻き戻しから分けた）、巻き戻しのための控え、セーブのスロット、エレメントのクリック、見本、入門の文章。pasta の中だけで作れてテストできる物（2〜9 番）と、areka の実装が無いと確かめられない物（10・11 番）を分けた。
- **areka との接点**: 語彙・イベント・設定の値の名前は、`novel-areka-contract` の文書が 1 か所で持つ。後続の spec は、名前をそこから引く。依頼を足す・変えるときは、文書を直してから、areka のセッションへ知らせる。areka のリポジトリを、pasta のセッションから書き換えない。
- **共有接点**:
  - `crates/pasta_lua/pasta_scripts/pasta/act.lua` は、1 つのウェーブに 1 つの spec だけが持つ。`cue-command-runtime`・`novel-talk-flow` が触る。Phase 11 の `call-attribute-filter`・`scene-anchor-link` も触る。
  - 出力の組み立て（`pasta_scripts/pasta/shiori/sakura_builder.lua`）は、`cue-command-runtime`・`novel-talk-flow`・`novel-checkpoint-rollback` が触る。`choice-line-layout`・`scene-anchor-link` も触る。
  - 選択肢の振り分け（`shiori/event/choice_select.lua`）は、`shell-element-click` と `scene-anchor-link` が共有する。先に入る側が、振り分けを関数として取り出す。
  - 場面の関数を出すコード生成（`code_gen/scope_gen.rs`）は、`scene-attribute-store`・`cue-command-runtime`（2026-10-17 に `scene-stage-attributes` をまとめた）が触る。この順に置く。場面の関数の頭は `act:init_scene(SCENE, 書いたグローバル名, 書いたローカル名, 行の表)` で、行の表は最後の引数である（`runtime-error-balloon` が入れた。`transpiler.rs` の `fill_line_maps` が後から差し込む）。引数を足す spec は、この形を保つ。
  - `act.lua` は 829 行あり、1,000 行に近い。`cue-command-runtime`・`call-attribute-filter`・`scene-anchor-link`・`novel-talk-flow` が足すので、足す側が分け方を考える。
  - 変数の書き方（`grammar.pest`・`code_gen/element_gen.rs` の `save.x`）は、`novel-variable-scope` が触る。文法を触る `call-attribute-filter`・`scene-anchor-link`・`cue-command-runtime` と同じウェーブに置かない。
  - 設定の読み込み（`loader/config/`）は、`cue-command-runtime`（別名表）と `novel-talk-flow`（ノベル向けの切り替え）が触る。
- **検証で分かった制約**（2026-10-10。詳しくは各 brief の「検証で見つかった穴」）:
  - NOTIFY のイベントには台本を返せない。ノベル用のイベントは、すべて GET で送ってもらう。GET の応答は 5 秒以内に返す。
  - 場面の検索は前方一致である（`＞第1章` は `第10章` にも当たる）。ノベル向けの設定では完全一致にする（`novel-talk-flow`）。
  - 出力の組み立ては `\x`（クリック待ち）を知らない。`\x` の後で、スコープ・段落の区切り・サーフェスの復旧がずれる（`novel-talk-flow`）。
  - 走りかけのコルーチンは控えられない。区切りにできるのは、場面が最後まで走った応答だけである（`novel-checkpoint-rollback`）。
  - 宣言行に書いた属性（`＊名前＆k：v`）は、パーサが読み飛ばしている（`scene-attribute-store` が直す）。

### 順番

| 順 | spec | 要件定義 | 規模 | 先に要るもの |
| -- | ---- | -------- | ---- | ------------ |
| 1 | novel-areka-contract | Opus | 6〜9（文書だけ） | なし。**0.5.0 のゴールの先頭**（areka の側が早く動けるよう、最初に始める） |
| 2 | scene-attribute-store（既存） | Fable | 15〜18 | なし（0.5.0 の先頭） |
| 3 | novel-variable-scope（2026-10-17 に `novel-checkpoint-rollback` から分けた） | Fable | 8〜10 | なし（文法と `element_gen.rs` を触るので、`call-attribute-filter`・`cue-command-runtime` と同じウェーブに置かない） |
| 4 | cue-command-runtime（ト書きと柱。2026-10-17 に `scene-stage-attributes` をまとめた） | Fable | 16〜20（超えるならト書きと柱に分け直す） | `scene-attribute-store`（柱の分）。語彙の名前は `novel-areka-contract` から引く |
| 5 | shell-element-click | Opus | 5〜8 | `novel-areka-contract`（完全一致の場面選びは `touch-poke-shortcuts` が入れてある）。`scene-anchor-link` とは `choice_select.lua` を共有する |
| 6 | novel-talk-flow | Fable | 17〜21（超えるなら出力の分と場面の流れの分に分ける） | `cue-command-runtime`、`novel-areka-contract` |
| 7 | call-attribute-filter（既存） | Fable | 19〜21 | `scene-attribute-store` |
| 8 | novel-checkpoint-rollback | Fable | 13〜15 | `novel-talk-flow`、`novel-variable-scope` |
| 9 | novel-save-slots | Opus | 8〜12 | `novel-checkpoint-rollback` |
| 10 | hello-novel-sample | Opus | 15〜18 | 2〜9 の全部と、areka の側の実装 |
| 11 | manual-novel-guide | Opus | 8〜12 | `hello-novel-sample` |
| 12 | 0.5.0 のリリース（`release` スキル） | Opus | — | 上の全部 |

並べられる組み合わせ（2026-10-17 の棚卸で測った）: `novel-areka-contract`・`scene-attribute-store` は、互いにも `pasta-check-validate-edges`・`tutorial-ssp-gaps` ともソースを共有しない。`shell-element-click` は `novel-areka-contract` の後、`scene-attribute-store` と並べられる（`scene.lua` を触らない約束で）。`novel-variable-scope` は `scene-attribute-store` と並べられる（`scene-attribute-store` が `partial.rs` と VSCode の文法定義を触らない約束で）。`cue-command-runtime` → `novel-talk-flow` → `novel-checkpoint-rollback` は、`act.lua` と `sakura_builder.lua` を共有するので 1 本ずつ入れる。

### areka の進み具合との関係

- 1〜9 番は、areka を待たずに、pasta だけで作れてテストできる。10・11 番は、areka が画面と音を出せるようになってからでないと、実機で確かめられない。
- areka には、音の層・複数のシェルを 1 つの窓に並べる仕組み・場面転換の演出・スクリプトへ届くキーボード入力が、まだ無い（2026-10-10・areka main `b3650b68`）。areka の側の作業が一番重い。**1 番を一番先に出して、areka の側が早く動けるようにする。**

### areka への依頼の出し方（開発者の指示）

- `novel-areka-contract` の最後のタスクで、areka のリポジトリのセッションへ依頼書を渡す。areka の側の `/kiro-discovery` が、それを areka の brief にする。
- 後続の spec が、要件定義や実装の途中で依頼を足す・変えるときも、同じ手順を踏む。そのとき、areka のセッションを立てて、同時に説明してもよい。各 brief の「areka への依頼」の節が、その入口である。

### spec の要件へ送った未定の項目

- 変数の保存の範囲の書き方（今の `＄＊` をセーブごとにするか、全セーブ共通にするか）— `novel-variable-scope` の要件の最初の議題（2026-10-17 に `novel-checkpoint-rollback` から分けた）。
- 条件つきの選択肢（フラグが立っているときだけ出す）— `novel-talk-flow` の要件で、要るかどうかを決める。
- 背景を柱にどう書くか（アクター指定行に並べるか、属性にするか）— `cue-command-runtime` の要件（柱の分）。
- ノベルを 1 本の配布物としてどう起動するか、動画が要るか — `novel-areka-contract` の要件。
- 数のしきい値（好感度が 50 以上、など）の書き方 — `call-attribute-filter` の要件。

## 窓際のぱすたさんからの要望（2026-10-18 起票・0.4.1）

2026-10-18 の `/kiro-discovery` で、ghost_dev の「ぱすた開発」セッション（窓際のぱすたさん）と相談して起こした。0.4.1 に入れる（2026-10-18 開発者の決定）。

- **ゴースト側の形（開発者の決定「案C」）**: 奥から、`\1` のシェル（窓。台詞を持たない）、`\0` の縦書きバルーン 1 枚（`vertical,1`。SSP 2.8.80 から標準）、`\0` のシェル（ぱすたとセキュリティボール）。右の枠から左の枠へは、同じバルーンの中で `\_l[x,y]` で飛ぶ。シェルの無い相手役（おじさん・男の子・ギャル・妹）は、ぱすたと同じ `\0` のバルーンに出す。
- **`actor-display-style`（spec）**: アクターごとの表示の装い（文字の色など）を pasta.toml で決め、`おじさん：やあ。` のとおりに書いて出せるようにする。1 つのスポットを複数のアクターで共有する形を正式に支える。同じスポットで話者が替わったときの改行は、もうある（`[ghost] spot_newlines = 1.8` で `\n[180]`）ので、足りないのは装いだけ。ゴースト側の今の試作は、アクター辞書の Lua 関数で回避している（`ぱすた：＠おじさん　やあ。`）が、ネタ帳のとおりに写せないので採らない（些細なことに Lua を出さない方針）。`novel-talk-flow`・`cue-command-runtime`・`novel-checkpoint-rollback`・`scene-anchor-link` と `sakura_builder.lua` を共有するので、同じウェーブに置かない。
- **`vertical-balloon-lines`（spec。2026-10-18 に直接実装の候補「budoux の改行の幅」から格上げ）**: budoux が入れる改行の幅（`budoux_newlines`）、列を動かすタグ（`\n`・`\n[…]`・`\c`・`\_l[…]`）の所で budoux が数え直すこと、選択肢どうしの間の改行の幅（`[ghost]` のキー、既定 1.0。汎用の機能。ぱすたは使わない）。budoux は自分で入れた改行しか知らず、作者が書いた `\n`・`\c` の後も前の列の字数を数え続ける（横書きでも起きている今の不具合）。方式は「budoux がタグを見て数え直す」（`act:budoux_reset()` のような明示の手段は採らない）。左の枠の短い列（`\_l` の後の幅）とトークンをまたいだ字数は、要件定義で決める。要件定義は Opus。`actor-display-style` とは `sakura_builder.lua`（選択肢どうしの間）を共有するので、同じウェーブに置かない。
- **左の枠へ飛ぶ `\_l`**: 飛ぶのは単語（ゴースト側の `＠左`）で足りる。budoux の数え直しは `vertical-balloon-lines` が持つ。

## リリース（`release` スキル）

リリースの CI 化（`release-ci`）、リリース手順（spec `release-workflow`）の書き換え、CI での初回のリリース（v0.3.8）とその後片付けは、2026-10-10 までにすべて済んだ。タグ `vX.Y.Z` の push だけで公開まで進む。記録は `.kiro/specs/completed/release-workflow/first-ci-release.md` と、同じフォルダの `design.md` にある。手順は 2026-10-11 に `release` スキル（`.claude/skills/release/`）へ移し、spec は完了にした。手順を直すときは、スキルを直す。**残るのは、失敗した job の再実行の読み方の確認である**（v0.3.8 も v0.4.0（2026-10-17、run 39024925990）も失敗が起きなかったので未確認。失敗が起きたリリースで確かめる）。

- **以後のリリース**: 新しい作業ブランチ（ハーネスのワークツリー）で `/release` を実行する（版を指定するときは `/release 0.4.0`）。失敗した job の再実行の読み方（スキルの `SKILL.md`「まだ実地で確かめていない読み方」）を確かめるまでは Opus で実行し、確かめたら Sonnet で実行する（開発者決定 2026-10-10）。
- **席**: リリースの実行中は、`release.ps1`・`release.yml` を触る spec を main へ入れない。
- **リリースの本文の作り直し**（2026-10-11 起票 `release-notes-highlights`）: 今の本文は、コミットの件名を全件並べた一覧で、何が良くなったのかを読み取れない（v0.3.8 は 57 行）。本文を、注目に値するいくつかの内容を Claudia（令嬢）の口調で紹介する文章にし、全件の一覧は Assets に添付する。spec 名は載せない。見どころは、版を上げるコミットでリポジトリのファイルとして書き、リリース CI が本文に使う（開発者の決定。公開の後で書き換える案と、機械だけで短くする案は採らない）。この spec は、`release` スキルの段 4 の「5 ファイルだけ」を変える。見出しには元になった PR の番号を付ける。公開済みの v0.3.8 の 1 本も、同じ形に直す（それより前は触らない）。areka のリポジトリの `areka-P0-release-notes-highlights` と、決定と議題をそろえてある。
- **運用の注意**（一回限りのセットアップで分かったこと）:
  - Azure のリソースを `az` で作る・変えるには MFA が要る。WAM（Windows のサインイン窓）でログインすると `RequestDisallowedByAzure` で弾かれる。`az config set core.enable_broker_on_windows=false` にしてからブラウザーで `az login` し直す。ワークフローのマネージド ID には関係ない。
  - Marketplace の Members の管理には、公式の CLI が無い（vsce・az・gh のどれも扱えない）。画面から行う。
  - setup-check の `verify-pat` が失敗すると、vsce のエラー文にマネージド ID の識別子が含まれ、ジョブのログに出る。秘密の値ではないので直さない（2026-10-10 決定）。

## 作成支援とマニュアルの保守

2026-10-10 に起票した 4 本（`pasta-check-dic-validate`・`manual-print-media-refs`・`manual-print-page-removal`・`manual-shell-guide`）は、2026-10-17 までに完了した（「完了フェーズ」）。

- `pasta-check-validate-edges`（2026-10-17 起票）— 辞書の検査の端のケース 3 つ（読めない辞書があっても飛び先の検査を省かない／`.pasta` で終わるフォルダを報告しない／VSCode 拡張は小文字の `ghost/master` だけを見る）と、代用品（`manual.yml` の段・`tutorial_stages_test.rs` の前方一致の検査）の置き換え。0.4.1 に入れる（2026-10-17 開発者の決定）。

## 入門ガイドの実機確認で見つかったもの（2026-10-10 起票）

`getting-started-story-guide` の完了時の棚卸で起票した。`manual-link-anchor-check` と `choice-line-layout` は 2026-10-11 に完了した。`boot-surface-without-dic` は 2026-10-16 に優先度を下げ、0.4.0 から外した。`baseware-virtual-time` は 0.4.0 の後に回した。2026-10-17 に、残った 2 本を `tutorial-ssp-gaps` にまとめ、同じ日に 0.4.1 に入れた（開発者の決定）。

- `tutorial-ssp-gaps`（旧 `baseware-virtual-time`）— SSP の仮想の時刻は pasta に届かない（pasta は OS の時計で正時を判定する）。入門ガイド 6 段目は「次の正時を待つ」と教えている。
- `tutorial-ssp-gaps`（旧 `boot-surface-without-dic`）— 辞書が無い間は何も表示されない。入門ガイドの準備の章は「何も表示されないのが正しい」と教えている。

絵を撮り直すときの決まりは、`getting-started-screenshots` が `tutorial-ssp-gaps` の brief に申し送った（`book/SCREENSHOTS.md`）。

## Specs (dependency order)

- [x] pasta-check-validate-edges -- 辞書の検査（`pasta_check validate`）の端のケース 3 つ（読めない辞書・`.pasta` で終わるフォルダ・VSCode 拡張の大文字と小文字）の裁定と直しと、代用品（`manual.yml` の段・`tutorial_stages_test.rs` の前方一致の検査）の置き換え（2026-10-17 の棚卸で、`pasta-check-dic-validate` の残りから起票）。0.4.1 に入れる（2026-10-17 開発者の決定）。Dependencies: none（2026-10-18 完了）
- [x] tutorial-ssp-gaps -- 入門ガイドを SSP で確かめたときに見つかった、SSP の動きとのずれ 2 つ。SSP の「現在時刻の仮想的変更」を時報に効かせる（外から起こした `OnTalk` でチェイントークが続かない件も）ことと、辞書や `＊OnBoot` が無いときの起動の立ち絵。どちらも、やるかどうかから決める。2026-10-17 に `baseware-virtual-time` と `boot-surface-without-dic` をまとめた。起動の立ち絵の分は、2026-10-16 開発者の決定で優先度を下げた（何も書かなければ何も表示しないのが既定の動き）。0.4.1 に入れる（2026-10-17 開発者の決定）。Dependencies: none（2026-10-18 完了。SSP 2.9.18 の TimeMachine は pasta.dll の読む OS の時計まで仮想時刻にすると分かり、時刻の飛びで次の正時を決め直し、間隔は飛びを差し引いて数える形にした）
- [ ] novel-areka-contract -- pasta と areka の取り決め（台本に載せる語彙・目印・イベント・設定の値）を 1 つの文書にし、areka への依頼書として出す。コードは書かない。areka に依頼を出すための spec（2026-10-10 の discovery で起票）。Dependencies: none
- [ ] scene-attribute-store -- シーン属性の実行時の保持・Lua からの読み出し・ファイルレベル属性の継承と上書き・値の型解釈。Dependencies: none
- [ ] novel-variable-scope -- 変数の保存の範囲を、セーブごとと全セーブ共通の 2 つに分ける書き方。今の `＄＊x` と保存ファイルとの互換、保存できない値の扱い（2026-10-17 の棚卸で `novel-checkpoint-rollback` から分割）。Dependencies: none
- [ ] cue-command-runtime -- キューコマンド行 `！命令＠対象（引数）` を、実行時に `\![…]` として台本へ流す。日本語の名前の別名表、引数の穴（数字で始まる引数など）の手当て。柱に書いた `＆属性` とアクター指定行も、場面に入ったときに台本へ流す（受け継ぎと、Call から戻ったときの打ち直し）。2026-10-17 に `scene-stage-attributes` をまとめた。Dependencies: scene-attribute-store
- [ ] shell-element-click -- エレメントのクリック（名前つき）を、選択肢と同じ振り分けで場面へ届ける。シェルとして組んだシステム画面を、場面が動かせるようにする。Dependencies: novel-areka-contract（`scene-anchor-link` と `choice_select.lua` を触る順番に注意）
- [ ] call-attribute-filter -- Call の属性フィルター構文（`＞シーン＆k＝v`・比較演算子・複数条件）と実行時の絞り込み。Dependencies: scene-attribute-store
- [ ] scene-anchor-link -- 台詞の中の `＠？シーン名`（`「表示名」` も付けられる）を、さくらスクリプトのアンカー `\_a` として出す。クリックで、`OnAnchorSelectEx` からそのシーンへ飛ぶ。選択肢の振り分けを共有し、LSP・VSCode の着色とマニュアル、`pasta_check validate` の飛び先の検査まで揃える（2026-10-08 起票）。版は未定。Dependencies: call-attribute-filter
- [ ] novel-talk-flow -- ノベル向けの出力に切り替える設定。台詞ごとのクリック待ち、話者名と声と変わらない行 ID、ランダムトークと時報の停止、起動時の場面、「次の場面へ」、完全一致の場面選びの切り替え。Dependencies: novel-areka-contract, cue-command-runtime
- [ ] novel-checkpoint-rollback -- 台本に目印の ID を埋め、応答ごとに状態と台本を控える。areka が知らせた ID の時点へ戻す（変数の保存の範囲は 2026-10-17 に `novel-variable-scope` へ分けた）。Dependencies: novel-talk-flow, novel-variable-scope
- [ ] novel-save-slots -- スロットを選ぶセーブとロード。控えをスロットへ読み書きし、スロットの中身（日時・章の名前）を場面から読めるようにする。Dependencies: novel-checkpoint-rollback
- [ ] hello-novel-sample -- 見本のノベル。物語と、タイトル・セーブ・ロード・設定の場面一式とシェル。0.5.0 の合格を areka の実機で確かめる題材。Dependencies: cue-command-runtime, novel-talk-flow, novel-variable-scope, novel-checkpoint-rollback, novel-save-slots, shell-element-click, call-attribute-filter（areka の側の実装も要る）
- [ ] manual-novel-guide -- マニュアルに、ノベルゲームの作り方の部を足す。見本のノベルを題材に、柱・ト書き・台詞・分岐・システム画面・配布の順に導く。Dependencies: hello-novel-sample
- [x] actor-display-style -- アクターごとの表示の装い（文字の色など）を pasta.toml で決め、スポットが替わったときと同じスポットで話者が替わったときに出し入れする。1 つのスポットを複数のアクターで共有する形を正式に支える（窓際のぱすたさんのシェルの無い相手役。2026-10-18 起票）。0.4.1 に入れる。`sakura_builder.lua` を共有する `novel-talk-flow`・`cue-command-runtime`・`novel-checkpoint-rollback`・`scene-anchor-link` と同じウェーブに置かない。Dependencies: none
- [x] vertical-balloon-lines -- 縦書きのバルーンの列の扱い。budoux が入れる改行の幅（`budoux_newlines`）、列を動かすタグ（`\n`・`\n[…]`・`\c`・`\_l[…]`）の所での budoux の数え直し（作者が書いた `\n`・`\c` の後も数え続ける今の不具合の直しを含む）、選択肢どうしの間の改行の幅（窓際のぱすたさん。2026-10-18 起票、直接実装の候補から格上げ）。0.4.1 に入れる。`sakura_builder.lua` を共有する `actor-display-style` と同じウェーブに置かない。Dependencies: none

## Existing Spec Updates

2026-10-10 の discovery で、0.5.0 の道筋に乗せた既存の spec。各 brief の「申し送り（2026-10-10 の discovery・0.5.0 のノベルゲームより）」の節に書いた。

- [ ] scene-attribute-store -- 属性の読み手に、演出（`cue-command-runtime` の柱の分。2026-10-17 に `scene-stage-attributes` をまとめた）が加わる。順番を保てる倉の形、ローカルシーンと名前なしの `＊` への受け継ぎを、要件の材料にする。0.5.0 の先頭に置く。Dependencies: none
- [ ] call-attribute-filter -- フラグによる分岐の本命になる。条件の値に変数を書けること、「それ以外」の場面、数のしきい値、候補が複数残るときの扱いを、要件で決める。Dependencies: scene-attribute-store

## Direct Implementation Candidates

- [ ] areka の `doc/PASTA_PROFILE.md` §5 の作例 -- 手書きの `\w[500]` が、「pasta の上流の欠陥」と誤って記録されている。pasta のエンジンは `\w[` を出さない（2026-10-10 に確かめた）。`novel-areka-contract` の依頼に添えて、areka の側で直してもらう。pasta の側に直す物は無い。

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

- `pasta_novel` アダプタ（ノベルゲーム宿主）— 0.5.0 のノベルゲームは、SHIORI とさくらスクリプトのまま作ると決めた（2026-10-10。上の「Phase 13」）。SHIORI を通さない宿主が必要になったら、その動機から起票する。Phase 7 の宿主非依存コアと presentation event stream 契約が土台になる。
- デバッグポートの他プロセスによる奪取（即時修正で判明）— 相手が `SO_REUSEADDR` を立てて bind する場合まで防ぐには、Windows の `SO_EXCLUSIVEADDRUSE` が要る。ゴースト同士の二重 bind は防いだ。
- `pasta_shiori` の `util/hglobal/windows_api.rs` の `string_to_multibyte`（即時修正で判明）— `@enc` と同じ 65001 で不正になるフラグを渡すが、テストからしか呼ばれない。`pasta_lua` の `encoding/windows.rs` と同じ先頭の分岐（65001 ならバイト列をそのまま返す）を足せば数行で直る。テスト専用なので、消す案もある。次の `review-improvement-loop` の実行で決める。
- 式の木の優先順位（2026-10-07 棚卸、`dsl-codegen-runtime-safety` の調査）— `pasta_dsl` の `parse_action.rs` の `build_left_assoc_expr` は優先順位なしで木を作り、コード生成（`expr_gen.rs`）が組み直している。AST を直接使う別の利用者が出たら直す。
- 別ファイルの同名シーンのデバッガ索引（2026-10-07 棚卸、`scene-identity-format` の範囲外）— ファイルをまたぐ同名シーンの索引と通し番号は、既存の制約として残した。
- タグの読み取りの制約（2026-10-07 棚卸、`paragraph-break-tag-only-talk` の範囲外）— `\nHello` をタグとして読む、`\_?…\_?` の中のタグも読む、引数の中の `\]` を扱わない、BudouX の行幅にエスケープと囲みを数えない。上の R4 と一部重なる。
- 閉じていない 1 行の引用（2026-10-07 棚卸、`dsl-literal-fixes` の範囲外）— `＠w：「abc` と、行をまたげる `sakura_body` の規則はそのままにした。
- 辞書の検査の広げ方（2026-10-17 棚卸、`pasta-check-dic-validate` の範囲外）— 単語参照の検査と、Lua の中身の検査。使う人の事例が出たら起票する。
- areka が撫で・つつきのイベント（`OnMouseMove`・`OnMouseDoubleClick` など）を SSP と同じ形で送るか（2026-10-17 棚卸、`touch-poke-shortcuts` の範囲外）— 確かめていない。0.5.0 で `novel-areka-contract` の依頼に要るなら、そこで確かめる。
- 止まるエラーの表記の粗さ（2026-10-17 棚卸、`runtime-error-balloon` の実装メモ「既知の粗さ」）— 同じ場面の中で `restore_scene` の後に止まると（アクターの単語の処理など）、`＊グローバル名` だけが出て、場所とローカル名が出ない。作者が `pcall` で失敗を飲み込み、同じ要求の中で止まると、古い場面の名前が出ることがある。困った事例が出たら起票する。
- バルーンが画面の左端で折り返るとき・縦に長いときの見え方（2026-10-17 棚卸、`getting-started-screenshots` の調査メモ。`hello-pasta-shell-art` からの持ち越し）— 実機で確かめていない。
- 並列負荷で時々落ちる TCP のテスト（2026-10-07 棚卸、`act-token-grouping-fix`・`scene-identity-format` の実装メモ）— `runtime_toggle_e2e_*`・`hook_panic_*`。
- マニュアルのメニューのテストが持つ mdBook の写し（2026-10-10 棚卸、`manual-claudia-theme` の実装メモ）— `book/tools/theme-menu-test.mjs` は、mdBook 0.5.4 の `book.js` のキー操作の処理を写して持つ。mdBook の版を上げるときに合わせる（CI は 0.5.3 に固定）。
- 配布版の hello-pasta に会話の変化を足すときの指針（2026-10-10 棚卸、`hello-pasta-tutorial-stages` の実装メモ）— emo2 開発からの知見「1 つの場面に 3 通り」は、段階の形を固定した辞書では満たせない。配布版に変化を足すことになったら、その動機から起票する。
- budoux の自動改行の禁則（2026-10-05、ghost_dev「emo2 開発」からの申し送り）— `line_breaker.rs` の `break_lines_impl` は禁則を見ない。BudouX の語の区切りの直前で改行するだけである。
  - 方針は JIS X 4051 どおりとする。リーダー（`‥…`）は行頭に置いてよい。並びの途中では分けない。
  - この方針では、申し送りの実例「イイジャン！／‥‥ええと、」は正しい組版になる。
  - 句読点・`！？`・閉じ括弧の前、開き括弧の後ろでは、BudouX の既定の日本語モデルは区切らない（手元で確認）。
  - BudouX が行頭禁則の字の直前（または行末禁則の字の直後）で区切る実例が出たら、その実例から起票する。その場合は、既存の `[talk]` の `chars_*` を使った追い出しを第一候補とする。
