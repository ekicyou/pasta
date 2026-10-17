# Roadmap

## 概要

pasta は、日本語 DSL（Pasta DSL）で書いた辞書を Lua へトランスパイルし、組込 LuaJIT で実行する「伺か」の SHIORI（`pasta.dll`）と、その周辺ツール（`pasta_check`・LSP・VSCode 拡張・利用者マニュアル）からなる。Phase 1〜11 で、プロパティアクセス・監査・マニュアル・ソースレベルデバッグ・アクターモデル駆動・配布物の形・現行実装の不具合の一掃までを完了した。2026-10-09 までに、リリースの CI 化、シーン名の別名、式の中の値なしの扱い、マニュアルの着せ替え、段階ごとに学べる hello-pasta の辞書も入った（下の「完了フェーズ」）。2026-10-11 までに、hello-pasta の新しいシェル、9 段目の選択肢の表示の直し、マニュアルの見出しへのリンクの検査も入った。2026-10-15 に、リリースの本文を見どころの紹介にした。2026-10-17 までに、撫で・つつきの反応のショートカット、実行時の失敗の出力の一本化、SSP を起動しない辞書の検査、シェルの説明の章、印刷用のページの廃止、結合テストの古いランタイムの写しの撤去も入った。

現在の主題は 3 つある。

- **Phase 11 の残り: 属性セマンティクスと失敗の出力**。シーン属性の保持と読み出し、Call の属性フィルター、文中のシーンリンク、スクリプトが止まるエラーのバルーン表示が残る（失敗の出力の一本化は 2026-10-17 に済んだ）。
- **Phase 12: 初心者向けの入門ガイド**。段階辞書・マニュアルの着せ替え・入門ガイドの本文・hello-pasta の新しいシェル・9 段目の選択肢の直し・撫で・つつきの反応のショートカット・シェルの説明の章は済んだ。残るのはスクリーンショットだけ（辞書が無い間の立ち絵は、2026-10-16 に優先度を下げて 0.4.0 から外した）。
- **Phase 13: areka でノベルゲーム（0.5.0）**。2026-10-10 の discovery で、合格ライン・通り道・台本の書き方・巻き戻しの方式を決め、spec を 9 本起こした。0.4.0 を出した後に始める（2026-10-16 から、ウェーブは今のゴールの範囲の spec だけで組む）。

このほか、リリース手順（spec `release-workflow`）の CI に合わせた書き換えと、CI での初回のリリース（v0.3.8）は済んだ（2026-10-10）。リリース手順は `release` スキル（`/release`）へ移し、spec `release-workflow` は完了にした（2026-10-11）。

**優先順位（開発者の方針 2026-10-10）**: 次の節目は 0.4.0 で、主題は「入門者にやさしく」である（下の「0.4.0 までの道筋」）。その次の 0.5.0 の目標は、areka でノベルゲームが作れるようになること。そこに入れた spec を先に進め、ほかは 0.4.0 の後に回す（トークンの予算のため）。致命的な不具合は、見つかりしだい割り込ませる。2026-10-11 に、撫で・つつきの反応のショートカット（`touch-poke-shortcuts`）を高優先度に置き、2026-10-16 に完了した。

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
- **マニュアルの見た目**: mdBook のまま「Claudia のマニュアル」。台詞は顔アイコン付きの部品で書く（記法は `book/AUTHORING.md`）。印刷用のページ（`print.html`）は作らず、章の印刷はテーマが受け持つ。
- **実行時の失敗**: シーンが生きている間の失敗は、`pasta/failure.lua` の `FAILURE.report` が、その場所に `【種類：対象（説明）】` の表記とログ 1 行を出す。切り替えは pasta.toml の `[logging] balloon_warning`。新しい種類は `FAILURE.LABEL` に足す。
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
| 11 | 実行時の失敗の出力の一本化（`【種類：対象（説明）】`・`pasta/failure.lua`・`[logging] balloon_warning`）、結合テストの古いランタイムの写しの撤去 | failure-output-unification, shiori-test-support-runtime |
| 12（仕上げ） | 撫で・つつきの反応を決まった名前のシーンで書けるようにし、入門の 8・9 段目を書き直した。マニュアルにシェルを自分の絵にする章を足した | touch-poke-shortcuts, manual-shell-guide |
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

- `hello-pasta-shell-art` が変えたリリース CI の配布物の検査（`.github/workflows/release.yml`。`.nar` にシェルの絵 18 枚・`surfaces.txt`・`descript.txt` があることを求める）は、実 CI ではまだ走っていない（2026-10-11）。次のリリース（0.4.0）で、この検査が通ることを確かめる。手元では `release.ps1` の完走と `.nar` の中身（40 エントリ・3.57 MB）を確かめてある。
- `pasta-check-bundled-balloon` を v0.3.7 で公開したことの、emo2 開発セッションへの連絡 — 実施の有無はリポジトリから確認できない。

## 0.4.0 までの道筋（2026-10-10 開発者の決定）

0.4.0 の主題は「入門者にやさしく」である。これまでは、仕様を満たすことと、潜在的な高機能さを優先してきた。0.4.0 では、初めて辞書を書く人が迷わず進めて、書き間違えたときに原因が分かることを優先する。絵つきの入門ガイドと新しいシェルの hello-pasta に加えて、実行時の失敗の見え方の一本化と、SSP を起動しない辞書の検査までを入れる。失敗の表記が変わる変更を、版の 2 桁目が上がる境目に収める。

| 順 | spec | 要件定義 | 規模 | 始められる時期 |
| -- | ---- | -------- | ---- | -------------- |
| 完了 | hello-pasta-shell-art | Fable | 13（実績） | 2026-10-11 に完了 |
| 完了 | manual-link-anchor-check | Opus | 8（実績） | 2026-10-11 に完了 |
| 完了 | choice-line-layout | Opus | 12（実績） | 2026-10-11 に完了 |
| 完了 | release-notes-highlights | Opus | 17（実績） | 2026-10-15 に完了（v0.3.8 のページも見どころの本文と 4 つの添付に直した） |
| 完了 | manual-print-media-refs | Opus | 7（実績） | 2026-10-16 に完了 |
| 完了 | manual-shell-guide | Opus | 8（実績） | 2026-10-16 に完了（リファレンスに章「シェルを自分の絵にする」を足した） |
| 完了 | manual-print-page-removal | Opus | 4（実績） | 2026-10-16 に完了（公開サイトの `print.html` が 404 になったことを 2026-10-17 に確かめた） |
| 完了 | touch-poke-shortcuts | Opus | 18（実績） | 2026-10-16 に完了（2026-10-11 開発者の依頼で足した高優先度の spec） |
| 完了 | shiori-test-support-runtime | Opus | 6（実績） | 2026-10-16 に完了（`runtime-error-balloon` の前段） |
| 完了 | pasta-check-dic-validate | Opus | 26（実績） | 2026-10-17 に完了（`pasta_check validate`・`release` の段 4 の関門・VSCode 拡張の診断・マニュアルの章 `debug/validate.md`） |
| 完了 | failure-output-unification | Opus | 18（実績） | 2026-10-17 に完了（報告の出口 `pasta/failure.lua`・`[logging] balloon_warning`・リファレンスの章「実行時の失敗の出力」。マニュアルは 63 章になった） |
| 1 | runtime-error-balloon | Opus | 7〜10 | 今すぐ（2026-10-17 のウェーブ。2026-10-16 開発者の決定で 0.4.0 に入れた） |
| 1 | getting-started-screenshots | Opus | 6〜9 | 今すぐ（2026-10-17 のウェーブ。`touch-poke-shortcuts` は完了した） |
| 2 | 0.4.0 のリリース（`release` スキル） | Opus | — | 上の全部の後 |

並走するときの約束（「棚卸」のウェーブの約束に足す）:

- `runtime-error-balloon` は、スクリプトが止まるエラーをバルーンに出す。見える失敗の出力を変えるので、`failure-output-unification` と同じ版の境目に収める。表記は `pasta/failure.lua` の出口（`FAILURE.notation`・`FAILURE.format`）と切り替え（`[logging] balloon_warning`）を使い、自分の表記を作らない。`FAILURE.LABEL` に種類を 1 行足すのはよい。
- `getting-started-screenshots` は、6 段目（時報）の撮り方を `baseware-virtual-time` に頼らない。失敗の表記やエラーのバルーンを絵に写すと決めた場合だけ、その 1 枚を `runtime-error-balloon` の後に撮る。
- 0.4.0 のリリースノートの見どころに、失敗の表記が変わったこと（`failure-output-unification`）、止まるエラーがバルーンに出るようになったこと（`runtime-error-balloon`）、SSP を起動しない辞書の検査（`pasta-check-dic-validate`）を書く。

0.4.0 の後（この順）: `scene-attribute-store`（Fable）、`call-attribute-filter`、`scene-anchor-link`、`baseware-virtual-time`。2026-10-17 に起票した `pasta-check-validate-edges` も、0.4.0 に入れると決めなければ、ここに入る。このうち `scene-attribute-store` と `call-attribute-filter` は、0.5.0 の道筋に入った（下の「Phase 13」）。残りの 2 本をどこに挟むかは、0.4.0 を出した後の棚卸で決める。`boot-surface-without-dic` は、2026-10-16 開発者の決定で優先度を下げ、0.4.0 から外した（「何も書かなければ何も表示しない」が既定の動き）。やるかどうかは、0.4.0 を出した後の棚卸で見直す。

### 0.5.0 の目標（2026-10-10 開発者の決定）

別リポジトリで開発中の areka で、ノベルゲームが作れるようになること。pasta の側に何が要るかは、2026-10-10 の discovery で洗い出した。合格ライン・決めたこと・spec の割り方と順番は、下の「Phase 13: areka でノベルゲーム（0.5.0）」にある。

## 棚卸（2026-10-17・main `e9b321e0`）

前回（2026-10-16・main `a5a0c480`）の後に、前回のウェーブの全部と、同じ日に起票した 1 本が入った。結合テストの古いランタイムの写しの撤去（`shiori-test-support-runtime`）、印刷用ページの動画の参照の直し（`manual-print-media-refs`）と印刷用ページの廃止（`manual-print-page-removal`）、シェルの説明の章（`manual-shell-guide`）、撫で・つつきの反応のショートカット（`touch-poke-shortcuts`）、SSP を起動しない辞書の検査（`pasta-check-dic-validate`）、実行時の失敗の出力の一本化（`failure-output-unification`）である。未完了は 17 本（常駐の `review-improvement-loop` を除く。この棚卸で 1 本起票した）、完了は 169 本。作業中のブランチは 1 本も無い（リモートのブランチは、どれも main に入った後のもの）。前回の記録は git の履歴にある。

再測定は、0.4.0 の範囲に残った 2 本（`runtime-error-balloon`・`getting-started-screenshots`）に絞った。ゴールの後の spec は、完了した spec が各 brief へ申し送りを書いてあるので、測り直していない（0.4.0 を出した後の棚卸で、0.5.0 の範囲として測る）。

- `runtime-error-balloon` の brief の予定は、入った失敗の出口と合っている。表記は `pasta/failure.lua` の `FAILURE.notation`・`FAILURE.format` を使い、自分の表記を作らない。切り替えは `[logging] balloon_warning`（既定は出す）。要件の議題が 3 つ増えた（配布したゴーストで既定のまま出てよいか／Lua のエラーの 1 行目にあるファイルの場所と行番号を残すか／出す場所を `RES.err` の中でなく `SHIORI.request` とタイムアウトの出口にする案）。
- `getting-started-screenshots` は先に要るものが無くなった。既定の設定の hello-pasta は失敗の表記を出さないので、撮る絵は `runtime-error-balloon` の前でも後でも同じになる（エラーの絵を載せると決めた場合だけ、その 1 枚を後に撮る）。brief の古い記述（章の名前・画像の数・印刷用ページ・部位名の行）は、再測定の節に書き出した。
- 2 本はソースのファイルもマニュアルのページも共有しない。

### 即時修正（spec なし・本棚卸で実施して閉じた）

- **main のマニュアルの CI が赤かった**（`failure-output-unification` の PR#127 から）。章を足す spec が合わせる 3 つの数のうち、`book/tools/talk/talk-test.mjs` の `J-9`（全 62 章）だけが取り残されていた。63 章に直した。このため、新しい章「実行時の失敗の出力」（`reference/runtime-failures.html`）は、公開サイトでまだ 404 である。この棚卸が main に入れば公開される。
- `.github/workflows/manual.yml` — 「`pasta_check` に構文検証のサブコマンドは無い」という古いコメントを直した（段の中身は変えていない）。
- `steering/tech.md` — 設計哲学の表の「Wait/Sync はマーカーのみ」「全出力は yield」を、今の実装（場面の終わりか `＞yield` までをまとめて 1 本のさくらスクリプトで返す）に直した（下の「Direct Implementation Candidates」から外した）。
- `steering/product.md` — 完了の数（169）・Phase の範囲・最終更新・現在地。`README.md` — `pasta_check` とデバッグの行に、辞書の検査を足した。
- brief — `runtime-error-balloon` の行番号（`event/init.lua` 228 → 229）。`scene-anchor-link` と `novel-talk-flow` に、`pasta-check-dic-validate` からの申し送り（検査の足し方）が無かったので補った。`scene-anchor-link` の Out にあった「飛び先の無いリンクの `pasta_check` での検出はバックログで扱う」を、この spec が `validate` に足す、に直した。
- この roadmap — 完了した 7 本を台帳と各 Phase のウェーブの表から外し、「完了フェーズ」へ畳んだ。済んだ spec を「先に要るもの」から外した（`cue-command-runtime`・`call-attribute-filter`・`runtime-error-balloon`・`getting-started-screenshots`）。0.4.0 の並走の約束を、残る 2 本の分だけにした。
- 確かめて閉じたもの: 公開サイトの `/pasta/print.html` と旧版の `/pasta/classic/print.html` は 404 になった（`manual-print-page-removal` の確かめ）。`manual-print-media-refs` の要件 1.3（公開サイトでの確かめ）は、対象が無くなった。

### 起票と、バックログに残したもの

- **起票** `pasta-check-validate-edges` — `pasta-check-dic-validate` が裁定待ちで残した端のケース 3 つ（読めない辞書があっても飛び先の検査を省かず、原因と別の「飛び先が無い」が並ぶ／名前が `.pasta` で終わるフォルダを報告しない／VSCode 拡張は小文字の `ghost/master` だけを検査する）と、範囲の外にした代用品の置き換え（`manual.yml` の段・`tutorial_stages_test.rs` の前方一致の検査）。4〜7 タスク、Opus、バグと後片付け。0.4.0 に入れるかは開発者が決める（下の「開発者の判断が要るもの」）。
- 分割・統合・改名: なし。
- バックログは全部そのまま残した（どれも、動機の事例・使う人・計測が無い）。完了した spec の範囲外から 2 行足した（辞書の検査の広げ方、areka の撫で・つつき）。

### 要件定義のモデル

Fable の spec は無い。いちばん近いのは `runtime-error-balloon` で、Rust・Lua・SHIORI の応答コードにまたがる。けれども、応答コードの選び方は文法・保存の形・公開の Lua の約束のどれでもなく、SSP の挙動は実機で確かめられる。議題にも推奨案を付けられるので、Opus のままにした。

### ウェーブ（2026-10-17）

0.4.0 の範囲に残った 2 本を、両方とも並べる。どちらも先に要るものが無く、互いのソースもマニュアルのページも重ならない。

| spec | 子孫 | 種別 | 規模 | 要件定義 |
| ---- | ---- | ---- | ---- | -------- |
| runtime-error-balloon | 0（後は 0.4.0 のリリース） | 機能（2026-10-16 開発者の決定で 0.4.0 に入れた） | 7〜10 | Opus |
| getting-started-screenshots | 0（後は 0.4.0 のリリース） | 文書 | 6〜9 | Opus |

開始コマンド（`/kiro-next` の報告と同じ形。俯瞰の頁が、この塊をコピーのボタンつきで出す）:

```
Wave 2本
Opus
/kiro-start runtime-error-balloon
/kiro-start getting-started-screenshots
```

同じウェーブに置くときの約束（上の「0.4.0 までの道筋」の約束に足す）:

- `runtime-error-balloon` が持つのは、`pasta_scripts/pasta/shiori/`（`entry.lua`・`event/callback.lua`。`res.lua`・`event/init.lua` は触らない見込み）、`pasta/failure.lua`（`FAILURE.LABEL` に 1 行足すだけ）、`pasta_shiori` の `error.rs`・`shiori.rs`・`actor/thread.rs` と `tests/`、マニュアルのリファレンスと内部設計の該当ページ。
- `getting-started-screenshots` が持つのは、入門の 16 章の画像の行、`book/src/img/` の新しい画像、撮影の手順の記録、画像の行の書き方によっては `book/tools/tutorial-check.mjs` とそのテスト・`book/AUTHORING.md`。`crates/` と、リファレンス・内部設計のページは触らない。
- どちらも章を足さない見込み。足すなら、3 つの数（`verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`）を 64 に合わせる。
- スキル references は、後から入る側が再生成する。

次: 0.4.0 のリリース（`/release 0.4.0`）。`pasta-check-validate-edges` を 0.4.0 に入れると決めた場合は、このウェーブに足せる（ほかの 2 本とファイルが重ならない）。

ゴールの後: `novel-areka-contract`・`scene-attribute-store`・`cue-command-runtime`・`baseware-virtual-time`・`boot-surface-without-dic`（どれも先頭。0.4.0 を出した後の棚卸で、0.5.0 の範囲として組む）。

### 開発者の判断が要るもの（ウェーブは止めない）

- `pasta-check-validate-edges` を 0.4.0 に入れるか。主題「書き間違えたときに原因が分かる」に合い、小さく、ほかと重ならない。入れないなら 0.4.0 の後。
- `runtime-error-balloon` の要件で、配布したゴーストで失敗の表記を既定で出すかを決める（今の `balloon_warning` は既定で出す。`pasta_check release` は見ていない）。
- `SOUL.md` §2・§5 の「Wait/Sync はマーカーのみ」「全出力は yield」— 構想の文書なので、直すかどうかは開発者が決める（`tech.md` は直した）。
- `scene-anchor-link`・`baseware-virtual-time`・`boot-surface-without-dic` を 0.5.0 の道筋のどこに挟むか — 0.4.0 を出した後の棚卸で決める。
- `steering/structure.md` の作り直し（`pasta_shiori` の木が無い）— `/kiro-steering` で直す。

## Phase 11 の残り: 属性セマンティクスと失敗の出力

### 境界戦略

- **分割理由**: シーン属性を 2 つに分けた（2026-10-04）。保持・継承・読み出し（`scene-attribute-store`）と、Call の属性フィルター（`call-attribute-filter`）。実行時の失敗の出力も 2 つに分けた（2026-10-10）。シーンが生きている間の失敗の出口の一本化（`failure-output-unification`）と、スクリプトが止まるエラーのバルーン表示（`runtime-error-balloon`）。
- **共有接点**: `crates/pasta_lua/pasta_scripts/pasta/act.lua` は 1 ウェーブに 1 spec だけが持つ。`call-attribute-filter`・`scene-anchor-link` が触る（`failure-output-unification` は 2026-10-17 に完了）。`crates/pasta_lua/src/code_gen/element_gen.rs` も 1 ウェーブに 1 spec だけで、`call-attribute-filter` と `scene-anchor-link` が触る。`scene-attribute-store` と `call-attribute-filter` は、`scene.lua` と `pasta_core` のシーン登録（`scene_registry.rs`・`scene_types.rs`）を共有するので、この順に置く。 選択肢の出力（`pasta_scripts/pasta/shiori/sakura_builder.lua`）は `choice-line-layout` と `scene-anchor-link` が触るので、この順に置く。
- **シーン名の別名との関係**: 別名の解決は検索の入口（`search/context.rs`）でキー正規化の前に行う。`scene-attribute-store`・`call-attribute-filter` は、別名を当てた後の名前で属性を扱う。
- **結合テスト**: `pasta_shiori` の結合テストの古いランタイムの写しと回避の設定は、`shiori-test-support-runtime`（2026-10-16 完了）で無くなった。新しい結合テストは、自己展開した本物のランタイムで動き、回避を書かない。`runtime-error-balloon` は同じ `crates/pasta_shiori/tests/` を触るので、その後に置く（満たした）。

### ウェーブ構成

| Wave | spec（並走可） | 種別 | ソースの持ち場 |
| ---- | -------------- | ---- | -------------- |
| 0.4.0 の後の先頭 | scene-attribute-store | 機能 | 宣言行の属性のパース（`parse_scene.rs`・`ast/scene.rs`）、`scope_gen.rs`・`context.rs`・`transpiler.rs`、`scene.lua`、`finalize.rs`、`pasta_core` のシーン登録（`scene_registry.rs`・`scene_types.rs`）、見本の期待値ファイル、マニュアル（`block-structure.md`・`script-api.md`・`registry-search.md`・`transpiler.md`） |
| 5 | call-attribute-filter | 機能 | Call の文法（`grammar.pest`・`parse_action.rs`・`ast/action.rs`・`partial.rs`）、`element_gen.rs`（Call）、`act.lua`（`call`・`find_handler`）、`scene.lua`（`SCENE.search`）、`search/`、`pasta_core` の `scene_table.rs`・`scene_types.rs`、LSP（`visit_action.rs`）、VSCode の文法定義 |
| 2026-10-17 のウェーブ（0.4.0） | runtime-error-balloon | 機能 | `pasta_scripts/pasta/shiori/`（`entry.lua`・`event/callback.lua`。`res.lua`・`event/init.lua` は触らない見込み）、`pasta/failure.lua`（`FAILURE.LABEL` に 1 行）、`pasta_shiori` の `error.rs`・`shiori.rs`・`actor/thread.rs` と結合テスト。`act.lua` は触らない |
| 6 | scene-anchor-link | 機能 | `call-attribute-filter` と同じ文法・生成・`act.lua`・エディタのファイルに加え、`sakura_builder.lua`、`shiori/event/choice_select.lua`、`sakura_script/`、VSCode 拡張の `wordRefDecorator.ts` |

Wave 1〜3 の 12 本と、先行させた `scene-name-alias`・`expr-nil-coercion`、2026-10-16 のウェーブの `failure-output-unification`・`shiori-test-support-runtime` は完了した（「完了フェーズ」の Phase 11）。

### 文中のシーンリンク（2026-10-08 起票）

台詞の中の語を Wikipedia のリンクのようにし、クリックでそのシーンへ飛べるようにする（`scene-anchor-link`）。

- **方式**:
  - 記法は `＠？シーン名`（`＠単語　` と同じく、空白か改行で終える）と `＠？シーン名「表示名」`。
  - 出力はさくらスクリプトの `\_a`（アンカー）で、`\q` ではない。`\q` だと、トーク全体が選択肢待ちになるため。
  - クリックは `OnAnchorSelectEx` を、選択肢の振り分け（`choice_select.lua`）と同じ規則で受ける。
  - `【】` で囲む案は、文法として唐突なので採らなかった。
- **飛び先が無いとき**: 選択肢と同じく 204 を返す。失敗の見せ方は `pasta/failure.lua` の出口に乗せる。飛び先の無いリンクの検査は、この spec が `pasta_check validate` に足す（`pasta-check-dic-validate` は範囲の外にした）。
- **ウェーブ**: `grammar.pest`・`parse_action.rs`・`element_gen.rs`・`act.lua`・VSCode の文法定義を、`call-attribute-filter` と共有する。そのため 1 本ずつ入れる。

## Phase 12: 初心者向けの入門ガイド

### 方針（2026-10-06 決定）

- 文法リファレンス（`grammar/`）と Lua 章（`lua/`）は、リファレンスのまま残す。入門ガイドが「こんな表現をしたい」を順に叶える物語で導き、各章から詳しい文法へリンクで送り出す。
- ガイドの間は Claudia が全編を語る（説明本体も Claudia の語り）。これは `getting-started` に限った執筆規約の例外とする。コードブロック内・構文定義・コマンド例には口調を持ち込まない。
- 題材は hello-pasta のまま（Claudia ゴーストは別に存在するので、ガイドの題材にはしない）。各章の終わりで読者のゴーストが起動できるようにし、最終章の辞書を hello-pasta と一致させる。
- hello-pasta のシェルは fal.ai の画像生成で作り直した（`hello-pasta-shell-art`・完了）。使ったモデルは `openai/gpt-image-2.5/flare/edit` だけで、切り抜きのモデルは使っていない（生成の記録は `crates/pasta_sample_ghost/art/README.md`）。
- マニュアルの見た目は、mdBook のまま「Claudia のマニュアル」に着せ替える（完了）。手本は ponapalt さんの「悪役令嬢クローディア」紹介ページ（<https://ponadocs.shillest.net/claudia/>、ponapalt/claudia、Unlicense）。サイト生成器の移行は採らない。

### 境界戦略

- **分割理由**: 持ち場で分けた。シェルの画像（`hello-pasta-shell-art`）、入門の外に置くシェルの説明の章（`manual-shell-guide`）、撫で・つつきのショートカット（`touch-poke-shortcuts`）は完了した。残るのはスクリーンショット（`getting-started-screenshots`）だけ。段階表と段階辞書、台詞の部品、ガイドの本文も完了している。本文の実機確認で見つかった 9 段目の見え方の不具合は、`choice-line-layout` が直した（2026-10-11）。
- **共有接点**: 段階表（`crates/pasta_sample_ghost/STAGES.md`）と段階辞書（`dic/01-boot.pasta`〜`12-lua.pasta`）を、本文が逐語で照合する（`book/tools/tutorial-check.mjs`）。辞書を変える spec は、同じ変更で入門の章の作例を直す。マニュアルの CI は辞書の変更でも走る。
- **シェルと本文の接点**: 8 段目（触られたときの反応）は、当たり判定の部位名を台詞に出す。部位名は `hello-pasta-shell-art` が要件で決め、本文は段階表から取る。頭は `Head` のまま変えない。
- **リリースとの接点**: `hello-pasta-shell-art` は、配布物を作るスクリプト（`release.ps1`）の絵を生成する段を外す。リリースの実行中は `release.ps1` を変えない。

### ウェーブ構成

| Wave | spec（並走可） | ソースの持ち場 |
| ---- | -------------- | -------------- |
| 2026-10-17 のウェーブ（0.4.0） | getting-started-screenshots | 撮影の手順の記録、`book/src/img/` の画像、入門の 16 章の画像の行（書き方によっては `tutorial-check.mjs` と `AUTHORING.md`）、新しいシェルでの 8 段目の実機の確認 |

Wave 1（`hello-pasta-tutorial-stages`・`manual-claudia-theme`）、ガイドの本文（`getting-started-story-guide`）、シェル（`hello-pasta-shell-art`）、9 段目の選択肢の直し（`choice-line-layout`）、撫で・つつきのショートカット（`touch-poke-shortcuts`）、シェルの説明の章（`manual-shell-guide`）は完了した。

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

- **分割理由**: 持ち場で分けた。areka との取り決め（文書だけ）、ト書きを流す仕組み、柱を流す仕組み、ノベル向けの出力、巻き戻しのための控え、セーブのスロット、エレメントのクリック、見本、入門の文章。pasta の中だけで作れてテストできる物（2〜7 番）と、areka の実装が無いと確かめられない物（8・9 番）を分けた。
- **areka との接点**: 語彙・イベント・設定の値の名前は、`novel-areka-contract` の文書が 1 か所で持つ。後続の spec は、名前をそこから引く。依頼を足す・変えるときは、文書を直してから、areka のセッションへ知らせる。areka のリポジトリを、pasta のセッションから書き換えない。
- **共有接点**:
  - `crates/pasta_lua/pasta_scripts/pasta/act.lua` は、1 つのウェーブに 1 つの spec だけが持つ。`cue-command-runtime`・`novel-talk-flow` が触る。Phase 11 の `call-attribute-filter`・`scene-anchor-link` も触る。
  - 出力の組み立て（`pasta_scripts/pasta/shiori/sakura_builder.lua`）は、`cue-command-runtime`・`novel-talk-flow`・`novel-checkpoint-rollback` が触る。`choice-line-layout`・`scene-anchor-link` も触る。
  - 選択肢の振り分け（`shiori/event/choice_select.lua`）は、`shell-element-click` と `scene-anchor-link` が共有する。先に入る側が、振り分けを関数として取り出す。
  - 場面の関数を出すコード生成（`code_gen/scope_gen.rs`）は、`scene-attribute-store`・`cue-command-runtime`・`scene-stage-attributes` が触る。この順に置く。
  - 設定の読み込み（`loader/config/`）は、`cue-command-runtime`（別名表）と `novel-talk-flow`（ノベル向けの切り替え）が触る。
- **検証で分かった制約**（2026-10-10。詳しくは各 brief の「検証で見つかった穴」）:
  - NOTIFY のイベントには台本を返せない。ノベル用のイベントは、すべて GET で送ってもらう。GET の応答は 5 秒以内に返す。
  - 場面の検索は前方一致である（`＞第1章` は `第10章` にも当たる）。ノベル向けの設定では完全一致にする（`novel-talk-flow`）。
  - 出力の組み立ては `\x`（クリック待ち）を知らない。`\x` の後で、スコープ・段落の区切り・サーフェスの復旧がずれる（`novel-talk-flow`）。
  - 走りかけのコルーチンは控えられない。区切りにできるのは、場面が最後まで走った応答だけである（`novel-checkpoint-rollback`）。
  - 宣言行に書いた属性（`＊名前＆k：v`）は、パーサが読み飛ばしている（`scene-attribute-store` が直す）。

### 順番

| 順 | spec | 要件定義（見当） | 規模（見当） | 先に要るもの |
| -- | ---- | ---------------- | ------------ | ------------ |
| 1 | novel-areka-contract | Opus（2026-10-16 に見直し） | 文書だけ | なし。**0.5.0 のゴールの先頭**（areka の側が早く動けるよう、0.4.0 を出したら最初に始める） |
| 2 | scene-attribute-store（既存） | Fable | 15〜18 | なし（0.4.0 の後の先頭） |
| 3 | cue-command-runtime | Opus | 8〜10 | なし（`failure-output-unification` は 2026-10-17 に完了し、`act.lua` の席は空いた） |
| 4 | scene-stage-attributes | Fable | 6〜8 | `scene-attribute-store`、`cue-command-runtime` |
| 5 | novel-talk-flow | Fable | 12〜15（超えるなら 2 本に分ける） | `cue-command-runtime`、`novel-areka-contract` |
| 6 | call-attribute-filter（既存） | Fable | 17〜20 | `scene-attribute-store` |
| 7 | novel-checkpoint-rollback | Fable | 15〜18（変数の保存の範囲が大きければ分ける） | `novel-talk-flow` |
| 8 | shell-element-click | Opus | 5〜8 | `novel-talk-flow`（完全一致の場面選び） |
| 9 | novel-save-slots | Opus | 8〜12 | `novel-checkpoint-rollback` |
| 10 | hello-novel-sample | Opus | 15〜18 | 2〜9 の全部と、areka の側の実装 |
| 11 | manual-novel-guide | Opus | 8〜12 | `hello-novel-sample` |
| 12 | 0.5.0 のリリース（`release` スキル） | Opus | — | 上の全部 |

どれとどれを同じウェーブに並べられるかは、0.4.0 を出した後の棚卸で、ソースの持ち場を測って決める。

### areka の進み具合との関係

- 1〜9 番は、areka を待たずに、pasta だけで作れてテストできる。10・11 番は、areka が画面と音を出せるようになってからでないと、実機で確かめられない。
- areka には、音の層・複数のシェルを 1 つの窓に並べる仕組み・場面転換の演出・スクリプトへ届くキーボード入力が、まだ無い（2026-10-10・areka main `b3650b68`）。areka の側の作業が一番重い。**1 番を一番先に出して、areka の側が早く動けるようにする。**

### areka への依頼の出し方（開発者の指示）

- `novel-areka-contract` の最後のタスクで、areka のリポジトリのセッションへ依頼書を渡す。areka の側の `/kiro-discovery` が、それを areka の brief にする。
- 後続の spec が、要件定義や実装の途中で依頼を足す・変えるときも、同じ手順を踏む。そのとき、areka のセッションを立てて、同時に説明してもよい。各 brief の「areka への依頼」の節が、その入口である。

### spec の要件へ送った未定の項目

- 変数の保存の範囲の書き方（今の `＄＊` をセーブごとにするか、全セーブ共通にするか）— `novel-checkpoint-rollback` の要件の最初の議題。
- 条件つきの選択肢（フラグが立っているときだけ出す）— `novel-talk-flow` の要件で、要るかどうかを決める。
- 背景を柱にどう書くか（アクター指定行に並べるか、属性にするか）— `scene-stage-attributes` の要件。
- ノベルを 1 本の配布物としてどう起動するか、動画が要るか — `novel-areka-contract` の要件。
- 数のしきい値（好感度が 50 以上、など）の書き方 — `call-attribute-filter` の要件。

## リリース（`release` スキル）

リリースの CI 化（`release-ci`）、リリース手順（spec `release-workflow`）の書き換え、CI での初回のリリース（v0.3.8）とその後片付けは、2026-10-10 までにすべて済んだ。タグ `vX.Y.Z` の push だけで公開まで進む。記録は `.kiro/specs/completed/release-workflow/first-ci-release.md` と、同じフォルダの `design.md` にある。手順は 2026-10-11 に `release` スキル（`.claude/skills/release/`）へ移し、spec は完了にした。手順を直すときは、スキルを直す。**残るのは、失敗した job の再実行の読み方の確認である**（初回は失敗が起きなかったので未確認。失敗が起きたリリースで確かめる）。

- **以後のリリース**: 新しい作業ブランチ（ハーネスのワークツリー）で `/release` を実行する（版を指定するときは `/release 0.4.0`）。失敗した job の再実行の読み方（スキルの `SKILL.md`「まだ実地で確かめていない読み方」）を確かめるまでは Opus で実行し、確かめたら Sonnet で実行する（開発者決定 2026-10-10）。
- **席**: リリースの実行中は、`release.ps1`・`release.yml` を触る spec を main へ入れない。
- **リリースの本文の作り直し**（2026-10-11 起票 `release-notes-highlights`）: 今の本文は、コミットの件名を全件並べた一覧で、何が良くなったのかを読み取れない（v0.3.8 は 57 行）。本文を、注目に値するいくつかの内容を Claudia（令嬢）の口調で紹介する文章にし、全件の一覧は Assets に添付する。spec 名は載せない。見どころは、版を上げるコミットでリポジトリのファイルとして書き、リリース CI が本文に使う（開発者の決定。公開の後で書き換える案と、機械だけで短くする案は採らない）。この spec は、`release` スキルの段 4 の「5 ファイルだけ」を変える。見出しには元になった PR の番号を付ける。公開済みの v0.3.8 の 1 本も、同じ形に直す（それより前は触らない）。areka のリポジトリの `areka-P0-release-notes-highlights` と、決定と議題をそろえてある。
- **運用の注意**（一回限りのセットアップで分かったこと）:
  - Azure のリソースを `az` で作る・変えるには MFA が要る。WAM（Windows のサインイン窓）でログインすると `RequestDisallowedByAzure` で弾かれる。`az config set core.enable_broker_on_windows=false` にしてからブラウザーで `az login` し直す。ワークフローのマネージド ID には関係ない。
  - Marketplace の Members の管理には、公式の CLI が無い（vsce・az・gh のどれも扱えない）。画面から行う。
  - setup-check の `verify-pat` が失敗すると、vsce のエラー文にマネージド ID の識別子が含まれ、ジョブのログに出る。秘密の値ではないので直さない（2026-10-10 決定）。

## 作成支援とマニュアルの保守

2026-10-10 に起票した 4 本（`pasta-check-dic-validate`・`manual-print-media-refs`・`manual-print-page-removal`・`manual-shell-guide`）は、2026-10-17 までに完了した（「完了フェーズ」）。

- `pasta-check-validate-edges`（2026-10-17 起票）— 辞書の検査の端のケース 3 つ（読めない辞書があっても飛び先の検査を省かない／`.pasta` で終わるフォルダを報告しない／VSCode 拡張は小文字の `ghost/master` だけを見る）と、代用品（`manual.yml` の段・`tutorial_stages_test.rs` の前方一致の検査）の置き換え。0.4.0 に入れるかは開発者が決める。

## 入門ガイドの実機確認で見つかったもの（2026-10-10 起票）

`getting-started-story-guide` の完了時の棚卸で起票した。`manual-link-anchor-check` と `choice-line-layout` は 2026-10-11 に完了した。`boot-surface-without-dic` は 2026-10-16 に優先度を下げ、0.4.0 から外した。`baseware-virtual-time` は 0.4.0 の後に回した。

- `baseware-virtual-time` — SSP の仮想の時刻は pasta に届かない（pasta は OS の時計で正時を判定する）。入門ガイド 6 段目は「次の正時を待つ」と教えている。
- `boot-surface-without-dic` — 辞書が無い間は何も表示されない。入門ガイドの準備の章は「何も表示されないのが正しい」と教えている。

隣の spec への申し送り: `getting-started-screenshots` の brief の「申し送り（getting-started-story-guide より）」の節（章の形・画像の行が本文の検査に落ちること・撮るときの制約・開発者の確認 4 点）。

## Specs (dependency order)

- [ ] runtime-error-balloon -- スクリプトが止まる実行時エラー（500 の応答）を、辞書を書く人に見えるようバルーンにも出す。表記は `pasta/failure.lua` の出口を使う。2026-10-16 開発者の決定で、やると決めて 0.4.0 に入れた（2026-10-10 の棚卸で `failure-output-unification` から分割）。Dependencies: none（`failure-output-unification`・`shiori-test-support-runtime` は完了）
- [ ] getting-started-screenshots -- 新しいシェルで、入門ガイドの各章にスクリーンショットを載せ、8 段目（触られたときの反応）を実機で確かめる（2026-10-10 の棚卸で `getting-started-story-guide` から分割）。Dependencies: none（`touch-poke-shortcuts` は完了）
- [ ] pasta-check-validate-edges -- 辞書の検査（`pasta_check validate`）の端のケース 3 つ（読めない辞書・`.pasta` で終わるフォルダ・VSCode 拡張の大文字と小文字）の裁定と直しと、代用品（`manual.yml` の段・`tutorial_stages_test.rs` の前方一致の検査）の置き換え（2026-10-17 の棚卸で、`pasta-check-dic-validate` の残りから起票）。Dependencies: none
- [ ] scene-attribute-store -- シーン属性の実行時の保持・Lua からの読み出し・ファイルレベル属性の継承と上書き・値の型解釈。Dependencies: none
- [ ] call-attribute-filter -- Call の属性フィルター構文（`＞シーン＆k＝v`・比較演算子・複数条件）と実行時の絞り込み。Dependencies: scene-attribute-store
- [ ] scene-anchor-link -- 台詞の中の `＠？シーン名`（`「表示名」` も付けられる）を、さくらスクリプトのアンカー `\_a` として出す。クリックで、`OnAnchorSelectEx` からそのシーンへ飛ぶ。選択肢の振り分けを共有し、LSP・VSCode の着色とマニュアル、`pasta_check validate` の飛び先の検査まで揃える（2026-10-08 起票）。Dependencies: call-attribute-filter
- [ ] baseware-virtual-time -- SSP の「現在時刻の仮想的変更」を pasta の時報に効かせ、正時を待たずに確かめられるようにする。SSP が仮想の時刻を SHIORI に伝えているかの調査から。外から起こした `OnTalk` でチェイントークが続かない件も扱う（2026-10-10 同上）。Dependencies: none
- [ ] boot-surface-without-dic -- 辞書や `＊OnBoot` が無いゴーストは起動しても立ち絵が出ない。既定で立ち絵を出すかどうかを、やるかどうかから決める（2026-10-10 同上）。2026-10-16 開発者の決定で優先度を下げ、0.4.0 から外した（何も書かなければ何も表示しないのが既定の動き）。Dependencies: none
- [ ] novel-areka-contract -- pasta と areka の取り決め（台本に載せる語彙・目印・イベント・設定の値）を 1 つの文書にし、areka への依頼書として出す。コードは書かない。areka に依頼を出すための spec（2026-10-10 の discovery で起票。以下 9 本は同じ）。Dependencies: none
- [ ] cue-command-runtime -- キューコマンド行 `！命令＠対象（引数）` を、実行時に `\![…]` として台本へ流す。日本語の名前の別名表、引数の穴（数字で始まる引数など）の手当て。Dependencies: none（`failure-output-unification` は完了し、`act.lua` の席は空いた）
- [ ] scene-stage-attributes -- 柱に書いた `＆属性` とアクター指定行を、場面に入ったときに台本へ流す。受け継ぎと、Call から戻ったときの打ち直し。Dependencies: scene-attribute-store, cue-command-runtime
- [ ] novel-talk-flow -- ノベル向けの出力に切り替える設定。台詞ごとのクリック待ち、話者名と声と変わらない行 ID、ランダムトークと時報の停止、起動時の場面、「次の場面へ」、完全一致の場面選び。Dependencies: novel-areka-contract, cue-command-runtime
- [ ] novel-checkpoint-rollback -- 台本に目印の ID を埋め、応答ごとに状態と台本を控える。areka が知らせた ID の時点へ戻す。変数の保存の範囲を、セーブごとと全セーブ共通の 2 つに分ける。Dependencies: novel-talk-flow
- [ ] novel-save-slots -- スロットを選ぶセーブとロード。控えをスロットへ読み書きし、スロットの中身（日時・章の名前）を場面から読めるようにする。Dependencies: novel-checkpoint-rollback
- [ ] shell-element-click -- エレメントのクリック（名前つき）を、選択肢と同じ振り分けで場面へ届ける。シェルとして組んだシステム画面を、場面が動かせるようにする。Dependencies: novel-areka-contract, novel-talk-flow（`scene-anchor-link` と `choice_select.lua` を触る順番に注意）
- [ ] hello-novel-sample -- 見本のノベル。物語と、タイトル・セーブ・ロード・設定の場面一式とシェル。0.5.0 の合格を areka の実機で確かめる題材。Dependencies: cue-command-runtime, scene-stage-attributes, novel-talk-flow, novel-checkpoint-rollback, novel-save-slots, shell-element-click, call-attribute-filter（areka の側の実装も要る）
- [ ] manual-novel-guide -- マニュアルに、ノベルゲームの作り方の部を足す。見本のノベルを題材に、柱・ト書き・台詞・分岐・システム画面・配布の順に導く。Dependencies: hello-novel-sample

## Existing Spec Updates

2026-10-10 の discovery で、0.5.0 の道筋に乗せた既存の spec。各 brief の「申し送り（2026-10-10 の discovery・0.5.0 のノベルゲームより）」の節に書いた。

- [ ] scene-attribute-store -- 属性の読み手に、演出（`scene-stage-attributes`）が加わる。順番を保てる倉の形、ローカルシーンと名前なしの `＊` への受け継ぎを、要件の材料にする。0.4.0 の後の先頭に置く。Dependencies: none
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
- 並列負荷で時々落ちる TCP のテスト（2026-10-07 棚卸、`act-token-grouping-fix`・`scene-identity-format` の実装メモ）— `runtime_toggle_e2e_*`・`hook_panic_*`。
- マニュアルのメニューのテストが持つ mdBook の写し（2026-10-10 棚卸、`manual-claudia-theme` の実装メモ）— `book/tools/theme-menu-test.mjs` は、mdBook 0.5.4 の `book.js` のキー操作の処理を写して持つ。mdBook の版を上げるときに合わせる（CI は 0.5.3 に固定）。
- 配布版の hello-pasta に会話の変化を足すときの指針（2026-10-10 棚卸、`hello-pasta-tutorial-stages` の実装メモ）— emo2 開発からの知見「1 つの場面に 3 通り」は、段階の形を固定した辞書では満たせない。配布版に変化を足すことになったら、その動機から起票する。
- budoux の自動改行の禁則（2026-10-05、ghost_dev「emo2 開発」からの申し送り）— `line_breaker.rs` の `break_lines_impl` は禁則を見ない。BudouX の語の区切りの直前で改行するだけである。
  - 方針は JIS X 4051 どおりとする。リーダー（`‥…`）は行頭に置いてよい。並びの途中では分けない。
  - この方針では、申し送りの実例「イイジャン！／‥‥ええと、」は正しい組版になる。
  - 句読点・`！？`・閉じ括弧の前、開き括弧の後ろでは、BudouX の既定の日本語モデルは区切らない（手元で確認）。
  - BudouX が行頭禁則の字の直前（または行末禁則の字の直後）で区切る実例が出たら、その実例から起票する。その場合は、既存の `[talk]` の `chars_*` を使った追い出しを第一候補とする。
