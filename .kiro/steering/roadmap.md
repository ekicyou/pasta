# Roadmap

## 概要

pasta は、日本語 DSL（Pasta DSL）で書いた辞書を Lua へトランスパイルし、組込 LuaJIT で実行する「伺か」の SHIORI（`pasta.dll`）と、その周辺ツール（`pasta_check`・LSP・VSCode 拡張・利用者マニュアル）からなる。Phase 1〜11 で、プロパティアクセス・監査・マニュアル・ソースレベルデバッグ・アクターモデル駆動・配布物の形・現行実装の不具合の一掃までを完了した。2026-10-09 までに、リリースの CI 化、シーン名の別名、式の中の値なしの扱い、マニュアルの着せ替え、段階ごとに学べる hello-pasta の辞書も入った（下の「完了フェーズ」）。2026-10-11 までに、hello-pasta の新しいシェル、9 段目の選択肢の表示の直し、マニュアルの見出しへのリンクの検査も入った。2026-10-15 に、リリースの本文を見どころの紹介にした。

現在の主題は 3 つある。

- **Phase 11 の残り: 属性セマンティクスと失敗の出力**。シーン属性の保持と読み出し、実行時の失敗の出力の一本化、Call の属性フィルター、文中のシーンリンク、スクリプトが止まるエラーのバルーン表示が残る。
- **Phase 12: 初心者向けの入門ガイド**。段階辞書・マニュアルの着せ替え・入門ガイドの本文・hello-pasta の新しいシェル・9 段目の選択肢の直しは済んだ。残るのは、撫で・つつきの反応のショートカット、シェルの説明の章、スクリーンショット（辞書が無い間の立ち絵は、2026-10-16 に優先度を下げて 0.4.0 から外した）。
- **Phase 13: areka でノベルゲーム（0.5.0）**。2026-10-10 の discovery で、合格ライン・通り道・台本の書き方・巻き戻しの方式を決め、spec を 9 本起こした。0.4.0 を出した後に始める（2026-10-16 から、ウェーブは今のゴールの範囲の spec だけで組む）。

このほか、リリース手順（spec `release-workflow`）の CI に合わせた書き換えと、CI での初回のリリース（v0.3.8）は済んだ（2026-10-10）。リリース手順は `release` スキル（`/release`）へ移し、spec `release-workflow` は完了にした（2026-10-11）。

**優先順位（開発者の方針 2026-10-10）**: 次の節目は 0.4.0 で、主題は「入門者にやさしく」である（下の「0.4.0 までの道筋」）。その次の 0.5.0 の目標は、areka でノベルゲームが作れるようになること。そこに入れた spec を先に進め、ほかは 0.4.0 の後に回す（トークンの予算のため）。致命的な不具合は、見つかりしだい割り込ませる。2026-10-11 に、撫で・つつきの反応のショートカット（`touch-poke-shortcuts`）を高優先度に置いた。0.4.0 の残りの spec より先に始める。

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
| 12（シェル） | hello-pasta の立ち絵をイラストにして当たり判定を足した。9 段目の選択肢を 1 行に 1 つずつ出す（設定 `choice_newlines`）。マニュアルのリンク検証が見出しの実在まで見る | hello-pasta-shell-art, choice-line-layout, manual-link-anchor-check |
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

- `sakura-script-newline` の Task 5.1（実機 SSP での目視確認）— SSP を操作するツール（SSP MCP）で実機の確認ができるようになった（2026-10-09 に `hello-pasta-tutorial-stages` が使った）。`getting-started-screenshots` の実機の確認のついでに済ませるかを、同 spec の要件で決める。
- `hello-pasta-shell-art` が変えたリリース CI の配布物の検査（`.github/workflows/release.yml`。`.nar` にシェルの絵 18 枚・`surfaces.txt`・`descript.txt` があることを求める）は、実 CI ではまだ走っていない（2026-10-11）。次のリリースで、この検査が通ることを確かめる。手元では `release.ps1` の完走と `.nar` の中身（40 エントリ・3.57 MB）を確かめてある。
- `pasta-check-bundled-balloon` を v0.3.7 で公開したことの、emo2 開発セッションへの連絡 — 実施の有無はリポジトリから確認できない。
- `getting-started-story-guide` の公開後の確認 4 点（2026-10-10。実装では、SSP を `--ghost` で一時起動して全 13 段をたどった。次の操作は試していない）— (1) 準備の章: ゴーストのフォルダを SSP の `ghost/` の下へ置いて切り替え、立ち絵が無い状態でタスクバーの SSP のアイコンからメニューを開いて終了できるか（最初に確かめる。違っていると読者が 1 段目へ進めない） (2) 目次: 実際のブラウザーで幅を狭めて開き、項目をクリックしても開いたままか (3) 7 段目: メニューから emo2 そのもの（`えも？？`）と切り替えて挨拶が出るか (4) 13 段目: フォルダのドロップで `.nar` ができるか（読者の `profile/` が入るかも見る）。`getting-started-screenshots` の実機の作業のついでに済ませるかを、同 spec の要件で決める。

## 0.4.0 までの道筋（2026-10-10 開発者の決定）

0.4.0 の主題は「入門者にやさしく」である。これまでは、仕様を満たすことと、潜在的な高機能さを優先してきた。0.4.0 では、初めて辞書を書く人が迷わず進めて、書き間違えたときに原因が分かることを優先する。絵つきの入門ガイドと新しいシェルの hello-pasta に加えて、実行時の失敗の見え方の一本化と、SSP を起動しない辞書の検査までを入れる。失敗の表記が変わる変更を、版の 2 桁目が上がる境目に収める。

| 順 | spec | 要件定義 | 規模 | 始められる時期 |
| -- | ---- | -------- | ---- | -------------- |
| 完了 | hello-pasta-shell-art | Fable | 13（実績） | 2026-10-11 に完了 |
| 完了 | manual-link-anchor-check | Opus | 8（実績） | 2026-10-11 に完了 |
| 完了 | choice-line-layout | Opus | 12（実績） | 2026-10-11 に完了 |
| 完了 | release-notes-highlights | Opus | 17（実績） | 2026-10-15 に完了（v0.3.8 のページも見どころの本文と 4 つの添付に直した） |
| 完了 | manual-print-media-refs | Opus | 7（実績） | 2026-10-16 に完了（公開サイトの `print.html` は開くと固まると分かり、確かめは `manual-print-page-removal` で対象が無くなる） |
| 並走 | manual-print-page-removal | Opus | 4〜6 | 今すぐ（2026-10-16 起票。公開サイトの `print.html` を開くとブラウザが固まる） |
| 1（高優先度） | touch-poke-shortcuts | Opus | 10〜13 | 今すぐ（2026-10-11 開発者の依頼で足し、高優先度に置いた。残りの spec より先に始める） |
| 2 | failure-output-unification | Opus（2026-10-16 に見直し） | 15〜19 | 今すぐ |
| 3 | pasta-check-dic-validate | Opus（2026-10-16 に見直し） | 10〜14 | 今すぐ（`hello-pasta-shell-art` が完了し、`Cargo.lock` の席が空いた） |
| 4 | manual-shell-guide | Opus | 5〜8 | 今すぐ（`hello-pasta-shell-art` が完了した） |
| 5 | getting-started-screenshots | Opus | 4〜6 | `touch-poke-shortcuts` の後（最後に撮る。`hello-pasta-shell-art` と `choice-line-layout` は完了した） |
| 完了 | shiori-test-support-runtime | Opus | 7（実績） | 2026-10-16 に完了（`runtime-error-balloon` の前段） |
| 7 | runtime-error-balloon | Opus（2026-10-16 に見直し） | 8〜10 | `failure-output-unification` の後（`shiori-test-support-runtime` は 2026-10-16 に完了。2026-10-16 開発者の決定で 0.4.0 に入れた。やると決まった） |
| 8 | 0.4.0 のリリース（`release` スキル） | Opus | — | 上の全部の後 |

並走するときの約束（「棚卸」のウェーブの約束に足す）:

- `failure-output-unification` は、選択肢の出力（`sakura_builder.lua`。立ち位置が決まらないときの警告がある）を、`choice-line-layout` が入るまで触らない（`choice-line-layout` は 2026-10-11 に完了したので、もう触れる）。生成コードの形を変えない約束と、スクリプトが止まるエラーを扱わない線引き（`runtime-error-balloon`）は、そのまま守る。
- `pasta-check-dic-validate` と `manual-shell-guide` は、どちらもマニュアルに章を足すと、目次（`SUMMARY.md`）と章の数を決め打ちする検査が重なる。後から入る側が数を合わせる。
- `getting-started-screenshots` は、6 段目（時報）の撮り方を `baseware-virtual-time` に頼らない。失敗の表記を絵に写すなら、`failure-output-unification` の後に撮る。
- 0.4.0 のリリースノートに、失敗の表記が変わったことを書く。`release-notes-highlights` が入っていれば、本文の見どころに書く。
- `touch-poke-shortcuts` が触るのは、イベント層の新しいファイル（`pasta_scripts/pasta/shiori/event/`。`init.lua` には `require` を 1 行足すだけ）、hello-pasta の 8・9 段目（`dic/08-touch.pasta`・`dic/09-choice.pasta`・`STAGES.md`・`src/scripts.rs`・`tests/tutorial_stages_test.rs`）、入門の章（`08-touch.md`・`09-choice.md`）とリファレンスである。`act.lua`・`sakura_builder.lua`・`choice_select.lua`・文法・トランスパイラを触らない。設定を足す場合は、設定の読み込み（`loader/config/`）が `failure-output-unification` と重なるので、後から入る側が合わせる。`manual-shell-guide` は `08-touch.md` にリンクを足すので、後から入る側が合わせる。`getting-started-screenshots` は、この spec の後に 8・9 段目を撮る。
- `release-notes-highlights` は、`release.yml` の verify と github-release の段、`release` スキルの手順の文書（`.claude/skills/release/`）を触る。`pasta-check-dic-validate` が `release.yml` の `pasta_check` の公開の段を触る場合は、段が別なので、後から入る側が合わせる。
- `runtime-error-balloon`（2026-10-16 に 0.4.0 へ入れた）は、スクリプトが止まるエラーをバルーンに出す。これも見える失敗の出力を変えるので、`failure-output-unification` と同じ版の境目に収める。報告の関数・失敗表記の形・pasta.toml の切り替えは `failure-output-unification` が決め、この spec はそれを使う側になる。`shiori/event/init.lua` を触るので、`touch-poke-shortcuts`（`require` を 1 行足す）とは後から入る側が合わせる。0.4.0 のリリースノートには、止まるエラーがバルーンに出るようになったことも書く。

0.4.0 の後（この順）: `scene-attribute-store`（Fable）、`call-attribute-filter`、`scene-anchor-link`、`baseware-virtual-time`。このうち `scene-attribute-store` と `call-attribute-filter` は、0.5.0 の道筋に入った（下の「Phase 13」）。残りの 2 本をどこに挟むかは、0.4.0 を出した後の棚卸で決める。`boot-surface-without-dic` は、2026-10-16 開発者の決定で優先度を下げ、0.4.0 から外した（「何も書かなければ何も表示しない」が既定の動き）。やるかどうかは、0.4.0 を出した後の棚卸で見直す。

### 0.5.0 の目標（2026-10-10 開発者の決定）

別リポジトリで開発中の areka で、ノベルゲームが作れるようになること。pasta の側に何が要るかは、2026-10-10 の discovery で洗い出した。合格ライン・決めたこと・spec の割り方と順番は、下の「Phase 13: areka でノベルゲーム（0.5.0）」にある。

## 棚卸（2026-10-16・main `a5a0c480`）

前回（2026-10-10 の 2 回目・main `a59438c6`）の後に、次のものが入った。hello-pasta の新しいシェル（`hello-pasta-shell-art`）、9 段目の選択肢を 1 行に 1 つずつ出す直し（`choice-line-layout`）、リンク検証の見出しの検査（`manual-link-anchor-check`）、リリース手順の `release` スキルへの移し替え（spec `release-workflow` の完了）、リリースの本文の見どころ化（`release-notes-highlights`）、マニュアルの OGP 画像、入門 3 段目の書き足し。brief は 10 本増えた（0.5.0 の 9 本と `touch-poke-shortcuts`）。未完了は 22 本（常駐の `review-improvement-loop` を除く）、完了は 162 本。作業中のブランチは 1 本も無い（リモートは main だけで、ほかのワークツリーは main に入った後のコミットで止まっている）。前回の記録は git の履歴にある。

再測定は、0.4.0 の範囲の 7 本と `novel-areka-contract` の 8 本に絞った（開発者の方針とトークンの予算のため。サブエージェントは使っていない）。

- 前回の後にエンジンのソースが変わったのは、選択肢の出力（`shiori/sakura_builder.lua`・`shiori/act.lua`）、設定の読み込み（`crates/pasta_lua/src/loader/config/`。`choice_newlines` を足した）、リリース CI（`release.yml`・`.github/scripts/release/`）、見本のゴースト（シェルと `src/`）である。`pasta/act.lua`・`actor.lua`・`word.lua`・`shiori/event/`・`pasta_check` は変わっていない。
- 完了した spec は、隣の spec の brief へ申し送りを書いてある（各 brief の 2026-10-11 の節）。読み直して、直す所は無かった。
- `failure-output-unification` の brief が挙げる行番号（`act.lua` 726 行・`act:failure` 696 行・呼び出し元 631 行と 658 行・`sakura_builder.lua` 75 行の立ち位置の警告）は、今の main と一致する。設定の読み込みは、`choice-line-layout` が `choice_newlines` を足した後の形から始める。
- Phase 11 の残りの 4 本と、0.5.0 の 8 本（`novel-areka-contract` を除く）は測り直していない。0.4.0 の後の棚卸で測る。

### 即時修正（spec なし・本棚卸で実施して閉じた）

- この roadmap — 完了した 4 本（`hello-pasta-shell-art`・`choice-line-layout`・`manual-link-anchor-check`・`release-notes-highlights`）の行を台帳から消し、「完了フェーズ」へ畳んだ。済んだ spec を、`pasta-check-dic-validate`・`manual-shell-guide`・`getting-started-screenshots` の「先に要るもの」から外した。`manual-shell-guide` の「優先度は低い」を直した。
- `steering/product.md` — 完了の数（162）と現在地。

### 要件定義のモデルの見直し（2026-10-15 の基準）

Fable を使うのは、後戻りしにくい土台の決定（新しい文法・保存するデータの形・ゴーストを書く人が使う公開の Lua の約束）があり、しかもその未決が深いときだけにする、と基準を厳しくした（2026-10-15）。「後の spec が使う」だけでは土台に当たらない。0.4.0 の表と `novel-areka-contract` を、この基準で見直した。結果、Fable の spec は無い。

- `failure-output-unification` — Opus にした。報告の関数は pasta の中の関数で、後の spec が使っても後から直せる。バルーンの失敗表記は、ゴーストが頼りにする出力ではない。空文字で続ける今の方針を変えるのは、移し方のはっきりした破壊的変更である。生成コードを変えない側（報告の関数が実行中の `act` を覚えておく）という推奨も、brief にもうある。
- `novel-areka-contract` — Opus にした（いちばん近い次点）。areka と取り決める名前は、文法・保存の形・公開の Lua の約束のどれでもない。後の spec が依頼を足す・直す手順も、もう決めてある（「Phase 13」の「areka への依頼の出し方」）。大きな決定（通り道・ト書き・柱・巻き戻しの方式）は discovery で済んでいる。NOTIFY に台本を返せない件も、すべて GET で送ってもらうと決着している。残る議題（文書の置き場所・名前の付け方・依頼の束の分け方・要るかどうかから決める項目）には、推奨案を付けられる。
- `boot-surface-without-dic` — Opus にした。既定の動きを 1 つ足すかどうかの決定で、議題には推奨案を付けられる。
- `pasta-check-dic-validate` — Opus にした。依存の重さ（`pasta_lua` に依存すると `cargo install pasta_check` が LuaJIT をビルドする）・誤りと注意の線引き・リリースの関門は、どれも道具の作りの決定で、文法や保存の形ではない。

### ウェーブ（2026-10-16）

方針（2026-10-10 の開発者の決定）どおり、0.4.0 の範囲の spec だけを取る。先に要るものが残っていない 6 本を並べる。互いのソースは下の約束で重ならない。最初は `novel-areka-contract`（0.5.0）も入れていたが、2026-10-16 の開発者の決定（今のゴールを超えた spec はウェーブに入れない）で外した。代わりに、同じ日に 0.4.0 へ入った `shiori-test-support-runtime` を入れた。さらに同じ日に、`boot-surface-without-dic` を 0.4.0 から外した（開発者の決定）。

| spec | 子孫 | 種別 | 規模 | 要件定義 |
| ---- | ---- | ---- | ---- | -------- |
| touch-poke-shortcuts | 約 1（`getting-started-screenshots`） | 機能（開発者の依頼・高優先度） | 10〜13 | Opus |
| failure-output-unification | 約 11（Phase 11 の 3 本と、`cue-command-runtime` から先の 0.5.0 の spec） | 機能 | 15〜19 | Opus |
| shiori-test-support-runtime | 約 1（`runtime-error-balloon`） | 基盤 | 6〜8 | Opus |
| pasta-check-dic-validate | 0 | 機能（作成支援） | 10〜14 | Opus |
| manual-shell-guide | 0 | 文書 | 5〜8 | Opus |
| manual-print-media-refs | 0 | バグ（マニュアル・CI） | 2〜3 | Opus |
| manual-print-page-removal | 0 | バグ（マニュアル・CI。2026-10-16 に足した） | 4〜6 | Opus |

同じウェーブに置くときの約束（上の「0.4.0 までの道筋」の約束に足す）:

- 設定の読み込み（`crates/pasta_lua/src/loader/config/` の `sections.rs`・`mod.rs`）は、`failure-output-unification` と `touch-poke-shortcuts` が、設定を足すと決めた場合に触る。どれも自分の設定を足すだけにして、既にある設定（`choice_newlines` など）を書き換えない。後から入る側が rebase で合わせる。
- `pasta/act.lua`・`actor.lua`・`word.lua` は `failure-output-unification` だけが触る。立ち位置の警告（`shiori/sakura_builder.lua` 75 行）を一本化に入れる場合も、`sakura_builder.lua` はこの spec だけが触る。
- `shiori/event/` は、`touch-poke-shortcuts`（新しいファイルと、`init.lua` への `require` 1 行）だけが触る。`failure-output-unification` は `shiori/event/` を触らない。
- hello-pasta の辞書・`STAGES.md`・`src/scripts.rs`・`tests/tutorial_stages_test.rs` は、`touch-poke-shortcuts` だけが触る。
- 入門の章は、別の節なら同じページを触ってよい。`08-touch.md` は `touch-poke-shortcuts`（本文）と `manual-shell-guide`（リンク）。後から入る側が rebase で合わせる。
- `SUMMARY.md` と、章の数を決め打ちする検査（`verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`）は、章を足す `manual-shell-guide` と、章を足す場合の `pasta-check-dic-validate` が触る。後から入る側が数を合わせる。
- `Cargo.lock`・ルートの `Cargo.toml`・`release.yml` の `pasta_check` の公開の段は、`pasta-check-dic-validate` だけが触る。`pasta_lua` のソースに手を入れる場合も、`act.lua`・`actor.lua`・`word.lua`・`loader/config/` は触らない。リリースの実行中は main へ入れない。
- `manual.yml`・`book/tools/verify-static.mjs`・`book/AUTHORING.md` の「CI と同じ順」の一覧は、`manual-print-media-refs` だけが触る。（2026-10-16 に完了し、同じ日に起票した `manual-print-page-removal` が席を引き継いだ。`pasta-check-dic-validate` は、この spec が入るまで `manual.yml` を触らない）
- `crates/pasta_shiori/tests/` は `shiori-test-support-runtime` だけが触る。ほかの 5 本は `pasta_shiori` を触らない。
- スキル references は、後から入る側が再生成する。

選ばなかった先頭: `getting-started-screenshots` は、`touch-poke-shortcuts` の後。`runtime-error-balloon` は先頭ではない（`failure-output-unification` と `shiori-test-support-runtime` の後）。

次のウェーブ: `runtime-error-balloon` と `getting-started-screenshots`。その後に 0.4.0 のリリース（`/release 0.4.0`）。

ゴールの後: `novel-areka-contract`・`scene-attribute-store`・`baseware-virtual-time` ほか（0.4.0 を出した後の棚卸で、0.5.0 のゴールの範囲として組む）。

### 開発者の判断が要るもの（ウェーブは止めない）

- `manual-print-media-refs` は「開発中」としていたが、作業のブランチが見当たらなかったので、ウェーブに入れ直した。既にどこかで進めているなら、そちらを続ける。
- `novel-areka-contract` を 0.4.0 と並べて先に進めたい場合は、「0.4.0 までの道筋」の範囲に入れる決定が要る（入れないなら、0.4.0 の後の先頭）。
- `scene-anchor-link`・`baseware-virtual-time` を 0.5.0 の道筋のどこに挟むか — 0.4.0 を出した後の棚卸で決める。
- `steering/structure.md`・`tech.md` の作り直し — `/kiro-steering` で直す（下の「Direct Implementation Candidates」）。

## Phase 11 の残り: 属性セマンティクスと失敗の出力

### 境界戦略

- **分割理由**: シーン属性を 2 つに分けた（2026-10-04）。保持・継承・読み出し（`scene-attribute-store`）と、Call の属性フィルター（`call-attribute-filter`）。実行時の失敗の出力も 2 つに分けた（2026-10-10）。シーンが生きている間の失敗の出口の一本化（`failure-output-unification`）と、スクリプトが止まるエラーのバルーン表示（`runtime-error-balloon`）。
- **共有接点**: `crates/pasta_lua/pasta_scripts/pasta/act.lua` は 1 ウェーブに 1 spec だけが持つ。`failure-output-unification`・`call-attribute-filter`・`scene-anchor-link` が触る。`crates/pasta_lua/src/code_gen/element_gen.rs` も 1 ウェーブに 1 spec だけで、`call-attribute-filter` と `scene-anchor-link` が触る。`scene-attribute-store` と `call-attribute-filter` は、`scene.lua` と `pasta_core` のシーン登録（`scene_registry.rs`・`scene_types.rs`）を共有するので、この順に置く。 選択肢の出力（`pasta_scripts/pasta/shiori/sakura_builder.lua`）は `choice-line-layout` と `scene-anchor-link` が触るので、この順に置く。
- **シーン名の別名との関係**: 別名の解決は検索の入口（`search/context.rs`）でキー正規化の前に行う。`scene-attribute-store`・`call-attribute-filter` は、別名を当てた後の名前で属性を扱う。
- **結合テスト**: `pasta_shiori` の結合テストの古いランタイムの写しと回避の設定は、`shiori-test-support-runtime`（2026-10-16 完了）で無くなった。新しい結合テストは、自己展開した本物のランタイムで動き、回避を書かない。`runtime-error-balloon` は同じ `crates/pasta_shiori/tests/` を触るので、その後に置く（満たした）。

### ウェーブ構成

| Wave | spec（並走可） | 種別 | ソースの持ち場 |
| ---- | -------------- | ---- | -------------- |
| 0.4.0 の後の先頭 | scene-attribute-store | 機能 | 宣言行の属性のパース（`parse_scene.rs`・`ast/scene.rs`）、`scope_gen.rs`・`context.rs`・`transpiler.rs`、`scene.lua`、`finalize.rs`、`pasta_core` のシーン登録（`scene_registry.rs`・`scene_types.rs`）、見本の期待値ファイル、マニュアル（`block-structure.md`・`script-api.md`・`registry-search.md`・`transpiler.md`） |
| 2026-10-16 のウェーブ | failure-output-unification | 機能 | `act.lua`・`actor.lua`・`word.lua` の警告と失敗表記、pasta.toml の切り替え（`loader/config/`）、マニュアル（`action-line.md`・`call-jump.md`・`script-api.md`・`talk-output.md`・`internal-modules.md`） |
| 2026-10-16 のウェーブ | shiori-test-support-runtime | 基盤 | `crates/pasta_shiori/tests/`（古いランタイムの写しの撤去・共通部品・回避の設定 4 か所） |
| 5 | call-attribute-filter | 機能 | Call の文法（`grammar.pest`・`parse_action.rs`・`ast/action.rs`・`partial.rs`）、`element_gen.rs`（Call）、`act.lua`（`call`・`find_handler`）、`scene.lua`（`SCENE.search`）、`search/`、`pasta_core` の `scene_table.rs`・`scene_types.rs`、LSP（`visit_action.rs`）、VSCode の文法定義 |
| 次（0.4.0） | runtime-error-balloon | 機能 | `pasta_scripts/pasta/shiori/`（`entry.lua`・`event/init.lua`・`event/callback.lua`・`res.lua`）、`pasta_shiori` の `error.rs`・`shiori.rs`・`actor/thread.rs` と結合テスト。`act.lua` は触らない |
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
- hello-pasta のシェルは fal.ai の画像生成で作り直した（`hello-pasta-shell-art`・完了）。使ったモデルは `openai/gpt-image-2.5/flare/edit` だけで、切り抜きのモデルは使っていない（生成の記録は `crates/pasta_sample_ghost/art/README.md`）。
- マニュアルの見た目は、mdBook のまま「Claudia のマニュアル」に着せ替える（完了）。手本は ponapalt さんの「悪役令嬢クローディア」紹介ページ（<https://ponadocs.shillest.net/claudia/>、ponapalt/claudia、Unlicense）。サイト生成器の移行は採らない。

### 境界戦略

- **分割理由**: 持ち場で分けた。シェルの画像（`crates/pasta_sample_ghost` の画像側。`hello-pasta-shell-art`）、スクリーンショット（新しいシェルが要る。`getting-started-screenshots`）、入門の外に置くシェルの説明の章（`manual-shell-guide`）。段階表と段階辞書、台詞の部品、ガイドの本文は完了している。本文の実機確認で見つかった 9 段目の見え方の不具合は、`choice-line-layout` が直した（2026-10-11）。
- **共有接点**: 段階表（`crates/pasta_sample_ghost/STAGES.md`）と段階辞書（`dic/01-boot.pasta`〜`12-lua.pasta`）を、本文が逐語で照合する（`book/tools/tutorial-check.mjs`）。辞書を変える spec は、同じ変更で入門の章の作例を直す。マニュアルの CI は辞書の変更でも走る。
- **シェルと本文の接点**: 8 段目（触られたときの反応）は、当たり判定の部位名を台詞に出す。部位名は `hello-pasta-shell-art` が要件で決め、本文は段階表から取る。頭は `Head` のまま変えない。
- **リリースとの接点**: `hello-pasta-shell-art` は、配布物を作るスクリプト（`release.ps1`）の絵を生成する段を外す。リリースの実行中は `release.ps1` を変えない。

### ウェーブ構成

| Wave | spec（並走可） | ソースの持ち場 |
| ---- | -------------- | -------------- |
| 2026-10-16 のウェーブ | touch-poke-shortcuts | イベント層の新しいファイル（`shiori/event/`）、hello-pasta の 8・9 段目の辞書と `STAGES.md`・`src/scripts.rs`・`tests/tutorial_stages_test.rs`、入門の `08-touch.md`・`09-choice.md` |
| 2026-10-16 のウェーブ | manual-shell-guide | シェルの説明の新しい章、入門の章（`setup.md`・`03-face.md`・`08-touch.md`）からのリンク、`SUMMARY.md` と章の数を決め打ちする検査 |
| 次 | getting-started-screenshots | 撮影の手順、画像、各章への画像の行、新しいシェルでの 8 段目の実機の確認。`touch-poke-shortcuts` の後に撮る |

Wave 1（`hello-pasta-tutorial-stages`・`manual-claudia-theme`）、ガイドの本文（`getting-started-story-guide`）、シェル（`hello-pasta-shell-art`）、9 段目の選択肢の直し（`choice-line-layout`）は完了した。

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
  - `crates/pasta_lua/pasta_scripts/pasta/act.lua` は、1 つのウェーブに 1 つの spec だけが持つ。`cue-command-runtime`・`novel-talk-flow` が触る。Phase 11 の `failure-output-unification`・`call-attribute-filter`・`scene-anchor-link` も触る。
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
| 3 | cue-command-runtime | Opus | 8〜10 | `failure-output-unification`（`act.lua` の席） |
| 4 | scene-stage-attributes | Fable | 6〜8 | `scene-attribute-store`、`cue-command-runtime` |
| 5 | novel-talk-flow | Fable | 12〜15（超えるなら 2 本に分ける） | `cue-command-runtime`、`novel-areka-contract` |
| 6 | call-attribute-filter（既存） | Fable | 17〜20 | `scene-attribute-store`、`failure-output-unification` |
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

## 作成支援とマニュアルの保守（2026-10-10 起票）

- `pasta-check-dic-validate` — SSP を起動せずに、ゴーストの `.pasta` の辞書を検査する `pasta_check` のサブコマンド。飛び先の無い Call・選択肢・文中のシーンリンク、モジュール名の衝突、記号を落とした後のシーン名の衝突などを見る。何をエラーにして何を警告にするか、`pasta_check release` の関門にするか、依存の重さ（`pasta_lua` に依存すると、`cargo install pasta_check` が LuaJIT をビルドする）を要件で決める。依存を足すと `Cargo.lock` が変わる（`hello-pasta-shell-art` が完了し、席は空いた）。マニュアルに章を足す場合は、章の数を決め打ちする検査（`verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`）と `SUMMARY.md` を、章を足すほかの spec（`manual-shell-guide` など）と共有する。内部クレートに依存させる場合は、公開の手順（`release.yml` の `pasta_check` の公開の段）も触るので、リリースの実行中は入れない。
- `manual-print-media-refs` — マニュアルの印刷用ページ（`print.html`）で、デバッグの章の動画が再生できない。ビルドの後に `print.html` の参照だけを書き換え、検査（`verify-static.mjs`）の例外を外す。
- `manual-print-page-removal`（2026-10-16 起票）— 公開サイトの `print.html` を開くとブラウザが固まる（全章をつないだ約 1 万 5 千行のページが、読み込みの後に自分で印刷の画面を開く。700 ページを超える）。`book.toml` で `print.html` を作らせないようにし、`manual-print-media-refs` が足した `print.html` のための道具と段を消す（開発者の決定）。旧版の `print.html` と古い URL の扱いは要件で決める。
- `manual-shell-guide`（2026-10-10 追加・0.4.0 に入れた）— 入門ガイドはシェルを「hello-pasta からフォルダごと写す」と教え、中身の説明は `setup.md` の節「シェルの中身」（ファイルの種類と、辞書との接点がサーフェス番号であること）までである。自分の絵に差し替える所から先を、入門の外に章を置いて書く。

## 入門ガイドの実機確認で見つかったもの（2026-10-10 起票）

`getting-started-story-guide` の完了時の棚卸で起票した。`manual-link-anchor-check` と `choice-line-layout` は 2026-10-11 に完了した。`boot-surface-without-dic` は 2026-10-16 に優先度を下げ、0.4.0 から外した。`baseware-virtual-time` は 0.4.0 の後に回した。

- `baseware-virtual-time` — SSP の仮想の時刻は pasta に届かない（pasta は OS の時計で正時を判定する）。入門ガイド 6 段目は「次の正時を待つ」と教えている。
- `boot-surface-without-dic` — 辞書が無い間は何も表示されない。入門ガイドの準備の章は「何も表示されないのが正しい」と教えている。

隣の spec への申し送り（各 brief の「申し送り（getting-started-story-guide より）」の節）: `getting-started-screenshots`（章の形・画像の行が本文の検査に落ちること・撮るときの制約・開発者の確認 4 点）、`hello-pasta-shell-art`（`first-ghost.md` の読み替え・段階表の訂正・7 段目の表に `＞ゴースト終了` が無いこと）、`manual-print-media-refs`（`manual.yml` の古いコメント・`AUTHORING.md` が空いたこと）。

## Specs (dependency order)

- [ ] scene-attribute-store -- シーン属性の実行時の保持・Lua からの読み出し・ファイルレベル属性の継承と上書き・値の型解釈。Dependencies: none
- [ ] failure-output-unification -- シーンが生きている間の実行時の失敗（未定義の参照・見つからない Call など）を、ログとさくらスクリプトの両方へ 1 つの仕組みから出す。既にある 2 つの失敗表記を 1 つにし、`act.lua`・`actor.lua`・`word.lua` の警告を載せ替える（2026-10-10 の棚卸で、スクリプトが止まるエラーを `runtime-error-balloon` へ分けた）。Dependencies: none
- [x] shiori-test-support-runtime -- `pasta_shiori` の結合テストがコピーして使う古いランタイムの写し（`tests/support/scripts/`）を撤去し、本物のランタイムだけで動かす。回避用の `pasta.toml` の設定とコメントを外す（2026-10-07 棚卸で起票）。Dependencies: none
- [x] manual-print-media-refs -- マニュアルの印刷用ページ（`print.html`）で動画の参照が切れているのを、ビルド後の書き換えで直し、検査の例外を外す（2026-10-10 棚卸で起票）。Dependencies: none
- [ ] manual-print-page-removal -- 公開サイトの `print.html` を開くとブラウザが固まるので、`book.toml` で印刷用のページを作らせないようにし、`print-media-refs` の道具と CI の段を消し、`verify-static.mjs` を合わせる（2026-10-16 起票）。Dependencies: none
- [ ] pasta-check-dic-validate -- SSP を起動せずに `.pasta` の辞書を検査する `pasta_check` のサブコマンド。飛び先の無い参照・モジュール名とシーン名の衝突などを見る（2026-10-10 棚卸で起票）。Dependencies: none（`hello-pasta-shell-art` は完了し、`Cargo.lock` の席は空いた）
- [ ] call-attribute-filter -- Call の属性フィルター構文（`＞シーン＆k＝v`・比較演算子・複数条件）と実行時の絞り込み。Dependencies: scene-attribute-store, failure-output-unification
- [ ] runtime-error-balloon -- スクリプトが止まる実行時エラー（500 の応答）を、辞書を書く人に見えるようバルーンにも出す。2026-10-16 開発者の決定で、やると決めて 0.4.0 に入れた（2026-10-10 の棚卸で `failure-output-unification` から分割）。Dependencies: failure-output-unification, shiori-test-support-runtime
- [ ] scene-anchor-link -- 台詞の中の `＠？シーン名`（`「表示名」` も付けられる）を、さくらスクリプトのアンカー `\_a` として出す。クリックで、`OnAnchorSelectEx` からそのシーンへ飛ぶ。選択肢の振り分けを共有し、LSP・VSCode の着色とマニュアルまで揃える（2026-10-08 起票）。Dependencies: failure-output-unification, call-attribute-filter
- [ ] getting-started-screenshots -- 新しいシェルで、入門ガイドの各章にスクリーンショットを載せ、8 段目（触られたときの反応）を実機で確かめる（2026-10-10 の棚卸で `getting-started-story-guide` から分割）。Dependencies: touch-poke-shortcuts（8・9 段目の辞書と章が変わる）
- [ ] baseware-virtual-time -- SSP の「現在時刻の仮想的変更」を pasta の時報に効かせ、正時を待たずに確かめられるようにする。SSP が仮想の時刻を SHIORI に伝えているかの調査から。外から起こした `OnTalk` でチェイントークが続かない件も扱う（2026-10-10 同上）。Dependencies: none
- [ ] boot-surface-without-dic -- 辞書や `＊OnBoot` が無いゴーストは起動しても立ち絵が出ない。既定で立ち絵を出すかどうかを、やるかどうかから決める（2026-10-10 同上）。2026-10-16 開発者の決定で優先度を下げ、0.4.0 から外した（何も書かなければ何も表示しないのが既定の動き）。Dependencies: none
- [ ] manual-shell-guide -- マニュアルに、シェル（見た目）の説明を足す。見本のシェルを自分の絵に差し替える・表情を足す・当たり判定を足す・バルーンの扱いを、辞書と噛み合う所を中心に書く。0.4.0 に入れた（2026-10-10 開発者の指示で起票）。Dependencies: none（題材の新しいシェルと当たり判定は、`hello-pasta-shell-art` で入った）
- [ ] novel-areka-contract -- pasta と areka の取り決め（台本に載せる語彙・目印・イベント・設定の値）を 1 つの文書にし、areka への依頼書として出す。コードは書かない。areka に依頼を出すための spec（2026-10-10 の discovery で起票。以下 9 本は同じ）。Dependencies: none
- [ ] cue-command-runtime -- キューコマンド行 `！命令＠対象（引数）` を、実行時に `\![…]` として台本へ流す。日本語の名前の別名表、引数の穴（数字で始まる引数など）の手当て。Dependencies: failure-output-unification（`act.lua` を触る順番）
- [ ] scene-stage-attributes -- 柱に書いた `＆属性` とアクター指定行を、場面に入ったときに台本へ流す。受け継ぎと、Call から戻ったときの打ち直し。Dependencies: scene-attribute-store, cue-command-runtime
- [ ] novel-talk-flow -- ノベル向けの出力に切り替える設定。台詞ごとのクリック待ち、話者名と声と変わらない行 ID、ランダムトークと時報の停止、起動時の場面、「次の場面へ」、完全一致の場面選び。Dependencies: novel-areka-contract, cue-command-runtime
- [ ] novel-checkpoint-rollback -- 台本に目印の ID を埋め、応答ごとに状態と台本を控える。areka が知らせた ID の時点へ戻す。変数の保存の範囲を、セーブごとと全セーブ共通の 2 つに分ける。Dependencies: novel-talk-flow
- [ ] novel-save-slots -- スロットを選ぶセーブとロード。控えをスロットへ読み書きし、スロットの中身（日時・章の名前）を場面から読めるようにする。Dependencies: novel-checkpoint-rollback
- [ ] shell-element-click -- エレメントのクリック（名前つき）を、選択肢と同じ振り分けで場面へ届ける。シェルとして組んだシステム画面を、場面が動かせるようにする。Dependencies: novel-areka-contract, novel-talk-flow（`scene-anchor-link` と `choice_select.lua` を触る順番に注意）
- [ ] hello-novel-sample -- 見本のノベル。物語と、タイトル・セーブ・ロード・設定の場面一式とシェル。0.5.0 の合格を areka の実機で確かめる題材。Dependencies: cue-command-runtime, scene-stage-attributes, novel-talk-flow, novel-checkpoint-rollback, novel-save-slots, shell-element-click, call-attribute-filter（areka の側の実装も要る）
- [ ] manual-novel-guide -- マニュアルに、ノベルゲームの作り方の部を足す。見本のノベルを題材に、柱・ト書き・台詞・分岐・システム画面・配布の順に導く。Dependencies: hello-novel-sample
- [ ] touch-poke-shortcuts -- 撫で・つつきの反応を、決まった名前のシーン（`＊さわり反応女の子Head` → `＊さわり反応Head` → `＊さわり反応その他`。つつきは `つっつき反応`）で書けるようにする。時報と同じ段々のフォールバックで、各段の次に英字の名前（`OnTouch…`・`OnPoke…`・`…Other`）も探し、最後は今までどおり `＊OnMouseMove`・`＊OnMouseDoubleClick` を呼ぶ。撫での判定（トーク中は反応しない）を pasta が持つ。入門ガイドの 8・9 段目と hello-pasta の辞書を書き直す（2026-10-11 の discovery で起票。emo2 の実装を調べた。**高優先度**）。Dependencies: none（`getting-started-screenshots` は、この後に撮る）

## Existing Spec Updates

2026-10-10 の discovery で、0.5.0 の道筋に乗せた既存の spec。各 brief の「申し送り（2026-10-10 の discovery・0.5.0 のノベルゲームより）」の節に書いた。

- [ ] scene-attribute-store -- 属性の読み手に、演出（`scene-stage-attributes`）が加わる。順番を保てる倉の形、ローカルシーンと名前なしの `＊` への受け継ぎを、要件の材料にする。0.4.0 の後の先頭に置く。Dependencies: none
- [ ] call-attribute-filter -- フラグによる分岐の本命になる。条件の値に変数を書けること、「それ以外」の場面、数のしきい値、候補が複数残るときの扱いを、要件で決める。Dependencies: scene-attribute-store, failure-output-unification
- [ ] pasta-check-dic-validate -- 範囲は変えない。後から入る spec（`cue-command-runtime`・`call-attribute-filter`・`novel-talk-flow`）が検査を足せる作りにしておく。Dependencies: none

## Direct Implementation Candidates

- [ ] `steering/tech.md` の「設計哲学」の表 -- 「Wait/Sync はマーカーのみ、areka 側で制御」「全出力は yield」と書いているが、実装は、場面の終わりか `＞yield` までの出力をまとめて 1 本のさくらスクリプトにして返す。Sync のマーカーは無い。`/kiro-steering` で直す（`steering/structure.md` の作り直しと一緒でよい）。`SOUL.md` §2・§5 の同じ記述は構想の文書なので、直すかどうかは開発者が決める。
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
- 並列負荷で時々落ちる TCP のテスト（2026-10-07 棚卸、`act-token-grouping-fix`・`scene-identity-format` の実装メモ）— `runtime_toggle_e2e_*`・`hook_panic_*`。
- マニュアルのメニューのテストが持つ mdBook の写し（2026-10-10 棚卸、`manual-claudia-theme` の実装メモ）— `book/tools/theme-menu-test.mjs` は、mdBook 0.5.4 の `book.js` のキー操作の処理を写して持つ。mdBook の版を上げるときに合わせる（CI は 0.5.3 に固定）。
- 配布版の hello-pasta に会話の変化を足すときの指針（2026-10-10 棚卸、`hello-pasta-tutorial-stages` の実装メモ）— emo2 開発からの知見「1 つの場面に 3 通り」は、段階の形を固定した辞書では満たせない。配布版に変化を足すことになったら、その動機から起票する。
- budoux の自動改行の禁則（2026-10-05、ghost_dev「emo2 開発」からの申し送り）— `line_breaker.rs` の `break_lines_impl` は禁則を見ない。BudouX の語の区切りの直前で改行するだけである。
  - 方針は JIS X 4051 どおりとする。リーダー（`‥…`）は行頭に置いてよい。並びの途中では分けない。
  - この方針では、申し送りの実例「イイジャン！／‥‥ええと、」は正しい組版になる。
  - 句読点・`！？`・閉じ括弧の前、開き括弧の後ろでは、BudouX の既定の日本語モデルは区切らない（手元で確認）。
  - BudouX が行頭禁則の字の直前（または行末禁則の字の直後）で区切る実例が出たら、その実例から起票する。その場合は、既存の `[talk]` の `chars_*` を使った追い出しを第一候補とする。
