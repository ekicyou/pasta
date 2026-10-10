# Roadmap

## 概要

pasta は、日本語 DSL（Pasta DSL）で書いた辞書を Lua へトランスパイルし、組込 LuaJIT で実行する「伺か」の SHIORI（`pasta.dll`）と、その周辺ツール（`pasta_check`・LSP・VSCode 拡張・利用者マニュアル）からなる。Phase 1〜11 で、プロパティアクセス・監査・マニュアル・ソースレベルデバッグ・アクターモデル駆動・配布物の形・現行実装の不具合の一掃までを完了した。2026-10-09 までに、リリースの CI 化、シーン名の別名、式の中の値なしの扱い、マニュアルの着せ替え、段階ごとに学べる hello-pasta の辞書も入った（下の「完了フェーズ」）。

現在の主題は 2 つある。

- **Phase 11 の残り: 属性セマンティクスと失敗の出力**。シーン属性の保持と読み出し、実行時の失敗の出力の一本化、Call の属性フィルター、文中のシーンリンク、スクリプトが止まるエラーのバルーン表示が残る。
- **Phase 12: 初心者向けの入門ガイド**。段階辞書・マニュアルの着せ替え・入門ガイドの本文は済んだ。残るのは、hello-pasta の新しいシェル、スクリーンショット、本文の実機確認で見つかった見え方の不具合と検査の穴。

このほか、リリース手順（常駐 spec `release-workflow`）の CI に合わせた書き換えと、CI での初回のリリース（v0.3.8）は済んだ（2026-10-10）。

**優先順位（開発者の方針 2026-10-10）**: 次の節目は 0.4.0 である（下の「0.4.0 までの道筋」）。そこに入れた spec を先に進め、ほかは 0.4.0 の後に回す（トークンの予算のため）。致命的な不具合は、見つかりしだい割り込ませる。

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

- `release-workflow` — crates.io・VSIX・ゴーストのリリース手順（版の決定 → 版の更新の PR → リリースタグの push → リリース CI の結果の確認。CI に合わせた書き換えと、CI での初回のリリース（v0.3.8）は済んだ。下の「リリース」）
- `review-improvement-loop` — レビュー領域 × 7 次元の改善ループ

### 却下（2026-10-01）

- `pasta-sstp-live-output`（SSTP/`\![raise]` による push 出力）— 目的はシーンキックで達成済み。新しい動機が生じたら、その動機から新規に起票する。
- `pasta-authoring-window`（`*.pasta` 編集・プレビュー専用ウィンドウ）— VSCode で実現済み。VSCode 非依存のオーサリング環境が必要になったら再検討する。

### 人手の確認が残っている項目

- `sakura-script-newline` の Task 5.1（実機 SSP での目視確認）— SSP を操作するツール（SSP MCP）で実機の確認ができるようになった（2026-10-09 に `hello-pasta-tutorial-stages` が使った）。`getting-started-screenshots` の実機の確認のついでに済ませるかを、同 spec の要件で決める。
- `pasta-check-bundled-balloon` を v0.3.7 で公開したことの、emo2 開発セッションへの連絡 — 実施の有無はリポジトリから確認できない。
- `getting-started-story-guide` の公開後の確認 4 点（2026-10-10。実装では、SSP を `--ghost` で一時起動して全 13 段をたどった。次の操作は試していない）— (1) 準備の章: ゴーストのフォルダを SSP の `ghost/` の下へ置いて切り替え、立ち絵が無い状態でタスクバーの SSP のアイコンからメニューを開いて終了できるか（最初に確かめる。違っていると読者が 1 段目へ進めない） (2) 目次: 実際のブラウザーで幅を狭めて開き、項目をクリックしても開いたままか (3) 7 段目: メニューから emo2 そのもの（`えも？？`）と切り替えて挨拶が出るか (4) 13 段目: フォルダのドロップで `.nar` ができるか（読者の `profile/` が入るかも見る）。`getting-started-screenshots` の実機の作業のついでに済ませるかを、同 spec の要件で決める。

## 0.4.0 までの道筋（2026-10-10 開発者の決定）

0.4.0 は「入門が完成し、書き間違えたときに原因が分かる版」とする。絵つきの入門ガイドと新しいシェルの hello-pasta に加えて、実行時の失敗の見え方の一本化と、SSP を起動しない辞書の検査までを入れる。失敗の表記が変わる変更を、版の 2 桁目が上がる境目に収める。

| 順 | spec | 要件定義 | 規模 | 始められる時期 |
| -- | ---- | -------- | ---- | -------------- |
| 開発中 | hello-pasta-shell-art | Fable | 12〜14 | 始めている |
| 開発中 | choice-line-layout、manual-print-media-refs、manual-link-anchor-check | Opus | 各 2〜4 | 始めている |
| 1 | boot-surface-without-dic | Fable | 3〜5（やる場合） | 今すぐ |
| 2 | failure-output-unification | Fable | 15〜19 | 今すぐ |
| 3 | pasta-check-dic-validate | Fable | 10〜14 | `hello-pasta-shell-art` の後（`Cargo.lock` の席） |
| 4 | manual-shell-guide | Opus | 5〜8 | `hello-pasta-shell-art` の後 |
| 5 | getting-started-screenshots | Opus | 4〜6 | `hello-pasta-shell-art`・`choice-line-layout`・`boot-surface-without-dic` の後（最後に撮る） |
| 6 | 0.4.0 のリリース（常駐 spec `release-workflow`） | Opus | — | 上の全部の後 |

並走するときの約束（「棚卸」のウェーブの約束に足す）:

- `boot-surface-without-dic` が触るのは、起動のイベント（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/boot.lua`）と準備の章（`book/src/getting-started/setup.md`）である。pasta.toml に設定を足す場合は、設定の読み込み（`crates/pasta_lua/src/loader/config/`）が `failure-output-unification` と重なるので、後から入る側が合わせる。やらないと決めたら却下し、0.4.0 の条件と `getting-started-screenshots` の前提から外す。
- `failure-output-unification` は、選択肢の出力（`sakura_builder.lua`。立ち位置が決まらないときの警告がある）を、`choice-line-layout` が入るまで触らない。生成コードの形を変えない約束と、スクリプトが止まるエラーを扱わない線引き（`runtime-error-balloon`）は、そのまま守る。
- `pasta-check-dic-validate` と `manual-shell-guide` は、どちらもマニュアルに章を足すと、目次（`SUMMARY.md`）と章の数を決め打ちする検査が重なる。後から入る側が数を合わせる。
- `getting-started-screenshots` は、6 段目（時報）の撮り方を `baseware-virtual-time` に頼らない。失敗の表記を絵に写すなら、`failure-output-unification` の後に撮る。
- 0.4.0 のリリースノートに、失敗の表記が変わったことを書く。

0.4.0 の後（この順）: `scene-attribute-store`（Fable）、`shiori-test-support-runtime`（Opus）、`runtime-error-balloon`、`call-attribute-filter`、`scene-anchor-link`、`baseware-virtual-time`。

## 棚卸（2026-10-10 の 2 回目・main `a59438c6`）

同じ日の 1 回目（main `add05022`）の後に、次のものが入った。入門ガイドの本文（`getting-started-story-guide`）、リリース手順の書き換えと CI での初回のリリース（v0.3.8）、配布物への直接リンクと README の見直し、入門ガイドの節「シェルの中身」。brief が 5 本増えた（`manual-link-anchor-check`・`choice-line-layout`・`baseware-virtual-time`・`boot-surface-without-dic`・`manual-shell-guide`）。未完了は 15 本で、進行中の spec は無い。1 回目の記録は git の履歴にある。

再測定は絞った（トークンの予算のため）。

- 1 回目の後、エンジンのソース（各クレートの `src/` と `pasta_scripts/`）は版の番号のほかに変わっていない。Phase 11 の残り 5 本と `pasta-check-dic-validate`・`shiori-test-support-runtime` は、1 回目の測定（各 brief の「2026-10-10 棚卸の再測定」の節）をそのまま使う。
- マニュアルと見本のゴーストに関わる 8 本は、各 brief の測定と申し送りの節を読み直した。どれも 2026-10-10 の main で書かれていて、直す所は無かった。

### 即時修正（spec なし・本棚卸で実施して閉じた）

- `steering/product.md` — 現行の版（v0.3.8）と現在地。
- この roadmap — 完了した `getting-started-story-guide` の行を消し、済んだ「リリース手順の書き換え」のチェックリストを「リリース」の節へ畳んだ。

### spec へ申し送ったもの

- `manual-print-media-refs` — `.github/workflows/manual.yml` の 114 行目あたりのコメントが、消えた章（`first-ghost.md`）を指している。brief に申し送り済み。
- `getting-started-screenshots` — 先に要るものに `choice-line-layout` を足した（9 段目の絵に、切れた選択肢が写らないようにする）。

### ウェーブ（2026-10-10 の 2 回目）

開発者の方針（2026-10-10 の 2 回目）: **マニュアル・見本のゴースト・CI を最優先**とし、次に致命的な不具合だけを進める。それ以外は見送る（トークンの予算のため）。致命的な不具合は無かった。依存の木の先頭は 9 本あり、そのうち方針に合う 4 本を始める。互いのソースは下の約束で重ならない。

| spec | 子孫 | 種別 | 規模 | 要件定義 |
| ---- | ---- | ---- | ---- | -------- |
| hello-pasta-shell-art | 約 3（`getting-started-screenshots`・`manual-shell-guide`・`pasta-check-dic-validate`） | 機能（見本のゴースト） | 12〜14 | Fable |
| choice-line-layout | 約 1（`getting-started-screenshots`） | バグ（見本のゴースト・入門の 9 段目の見え方） | 2〜4 | Opus |
| manual-print-media-refs | 0 | バグ（マニュアル・CI） | 2〜3 | Opus |
| manual-link-anchor-check | 0 | 検査の穴（マニュアル・CI） | 2〜4 | Opus |

同じウェーブに置くときの約束:

- `hello-pasta-shell-art` は、頭の部位名を今の `Head` のままにする。辞書と `book/` は、部位名の都合で変えるときに限り、8 段目（`dic/08-touch.pasta`・`book/src/getting-started/08-touch.md`）だけを触る。段階表（`STAGES.md`）は、部位名の注記と 8 段目の行だけを触る。`book/tools/` を触らない。`Cargo.lock`・ルートの `Cargo.toml`・`release.ps1` は、このウェーブではこの spec だけが触る。リリースの実行中は、`release.ps1` の変更を main へ入れない。
- `choice-line-layout` が触るのは、選択肢の出力（`sakura_builder.lua` とそのテスト）か、9 段目の辞書（`dic/09-choice.pasta`）・章（`09-choice.md`）・段階表の 9 段目の行・`tests/tutorial_stages_test.rs` のどちらかである。`act.lua`・`element_gen.rs`・シェルのファイル・`tests/integration_test.rs` を触らない。
- `manual-print-media-refs` は、`manual.yml`・`book/tools/verify-static.mjs`・`book/AUTHORING.md` の「CI と同じ順」の一覧を持つ。`link-check.mjs` を触らない。
- `manual-link-anchor-check` が触るのは、`link-check.mjs` とその自己テスト、切れたリンクの行だけである。`manual.yml`・`verify-static.mjs`・`AUTHORING.md` を触らない。
- スキル references は、後から入る側が再生成する。

次点だった `boot-surface-without-dic` と、見送りの先頭だった `failure-output-unification` は、この後の決定で 0.4.0 に入った（上の「0.4.0 までの道筋」）。下の表は棚卸の時点の記録である。

見送った先頭（予算が戻ったら、この順に取る）:

| spec | 子孫 | 種別 | 規模 | 要件定義 |
| ---- | ---- | ---- | ---- | -------- |
| failure-output-unification | 約 3（`call-attribute-filter`・`scene-anchor-link`・`runtime-error-balloon`） | 機能 | 15〜19 | Fable |
| scene-attribute-store | 約 2（`call-attribute-filter`・`scene-anchor-link`） | 機能 | 15〜18 | Fable |
| shiori-test-support-runtime | 約 1（`runtime-error-balloon`） | 基盤 | 6〜8 | Opus |
| baseware-virtual-time | 約 1（`getting-started-screenshots` の 6 段目の撮り方） | 作成支援（調査から） | 調査による | Fable |

見送った spec を後から始めるときの約束（1 回目の棚卸から引き継ぐ）:

- `failure-output-unification` は、生成コードの形（`expr_gen.rs`・`element_gen.rs`・見本の期待値ファイル）を変えない。期待値ファイルは `scene-attribute-store` が作り直す。要件で生成コードを変えると決めた場合は、`scene-attribute-store` を先に入れて rebase する。
- `scene-attribute-store` は、属性を読み出す API を `act.lua` に置かない（`scene.lua` の `SCENE` 側に置く）。`act.lua` は `failure-output-unification` が持つ。
- `shiori-test-support-runtime` は `crates/pasta_shiori/tests/` だけを触る。Phase 11 の spec が結合テストを足す前に入れるほど、回避の写しが増えない。
- マニュアルの同じページの別の節（`lua/script-api.md` など）は、`scene-attribute-store` と `failure-output-unification` が分け合う。

その次の候補: `getting-started-screenshots`（`hello-pasta-shell-art` と `choice-line-layout` の後）、`manual-shell-guide`（`hello-pasta-shell-art` の後。優先度は低い）、`pasta-check-dic-validate`（`Cargo.lock` の席が空いてから）、`call-attribute-filter`（または `scene-anchor-link`。同じファイルを触るので 1 本ずつ）、`runtime-error-balloon`。

### 開発者の判断が要るもの（ウェーブは止めない）

- 辞書が無い間に立ち絵を出すか — `boot-surface-without-dic` の要件の最初の議題。やらないなら却下する。
- スクリプトが止まるエラーをバルーンに出すか — `runtime-error-balloon` の要件の最初の議題。要らなければ却下する。
- `call-attribute-filter` と `scene-anchor-link` のどちらを先にするか — 台帳は前者を先にしている。後者を先にすると、`scene-attribute-store` を待たずに始められる。
- 見送った spec をいつ始めるか — `shiori-test-support-runtime` は、Phase 11 の spec より先に入れるほど後片付けが減る。
- `steering/structure.md` の作り直し — `pasta_shiori` の木が無い、`pasta_lua/tests` の一覧が古い、など。`/kiro-steering` で直す。

## Phase 11 の残り: 属性セマンティクスと失敗の出力

### 境界戦略

- **分割理由**: シーン属性を 2 つに分けた（2026-10-04）。保持・継承・読み出し（`scene-attribute-store`）と、Call の属性フィルター（`call-attribute-filter`）。実行時の失敗の出力も 2 つに分けた（2026-10-10）。シーンが生きている間の失敗の出口の一本化（`failure-output-unification`）と、スクリプトが止まるエラーのバルーン表示（`runtime-error-balloon`）。
- **共有接点**: `crates/pasta_lua/pasta_scripts/pasta/act.lua` は 1 ウェーブに 1 spec だけが持つ。`failure-output-unification`・`call-attribute-filter`・`scene-anchor-link` が触る。`crates/pasta_lua/src/code_gen/element_gen.rs` も 1 ウェーブに 1 spec だけで、`call-attribute-filter` と `scene-anchor-link` が触る。`scene-attribute-store` と `call-attribute-filter` は、`scene.lua` と `pasta_core` のシーン登録（`scene_registry.rs`・`scene_types.rs`）を共有するので、この順に置く。 選択肢の出力（`pasta_scripts/pasta/shiori/sakura_builder.lua`）は `choice-line-layout` と `scene-anchor-link` が触るので、この順に置く。
- **シーン名の別名との関係**: 別名の解決は検索の入口（`search/context.rs`）でキー正規化の前に行う。`scene-attribute-store`・`call-attribute-filter` は、別名を当てた後の名前で属性を扱う。
- **結合テスト**: `pasta_shiori` の結合テストは、`shiori-test-support-runtime` が入るまで古いランタイムの写しへの回避を写して増える。`runtime-error-balloon` は同じ `crates/pasta_shiori/tests/` を触るので、その後に置く。

### ウェーブ構成

| Wave | spec（並走可） | 種別 | ソースの持ち場 |
| ---- | -------------- | ---- | -------------- |
| 4（見送り中） | scene-attribute-store | 機能 | 宣言行の属性のパース（`parse_scene.rs`・`ast/scene.rs`）、`scope_gen.rs`・`context.rs`・`transpiler.rs`、`scene.lua`、`finalize.rs`、`pasta_core` のシーン登録（`scene_registry.rs`・`scene_types.rs`）、見本の期待値ファイル、マニュアル（`block-structure.md`・`script-api.md`・`registry-search.md`・`transpiler.md`） |
| 4（0.4.0 に入れた） | failure-output-unification | 機能 | `act.lua`・`actor.lua`・`word.lua` の警告と失敗表記、pasta.toml の切り替え（`loader/config/`）、マニュアル（`action-line.md`・`call-jump.md`・`script-api.md`・`talk-output.md`・`internal-modules.md`） |
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

- **分割理由**: 持ち場で分けた。シェルの画像（`crates/pasta_sample_ghost` の画像側。`hello-pasta-shell-art`）、スクリーンショット（新しいシェルが要る。`getting-started-screenshots`）、入門の外に置くシェルの説明の章（`manual-shell-guide`）。段階表と段階辞書、台詞の部品、ガイドの本文は完了している。本文の実機確認で見つかった 9 段目の見え方の不具合は、`choice-line-layout` が直す。
- **共有接点**: 段階表（`crates/pasta_sample_ghost/STAGES.md`）と段階辞書（`dic/01-boot.pasta`〜`12-lua.pasta`）を、本文が逐語で照合する（`book/tools/tutorial-check.mjs`）。辞書を変える spec は、同じ変更で入門の章の作例を直す。マニュアルの CI は辞書の変更でも走る。
- **シェルと本文の接点**: 8 段目（触られたときの反応）は、当たり判定の部位名を台詞に出す。部位名は `hello-pasta-shell-art` が要件で決め、本文は段階表から取る。頭は `Head` のまま変えない。
- **リリースとの接点**: `hello-pasta-shell-art` は、配布物を作るスクリプト（`release.ps1`）の絵を生成する段を外す。リリースの実行中は `release.ps1` を変えない。

### ウェーブ構成

| Wave | spec（並走可） | ソースの持ち場 |
| ---- | -------------- | -------------- |
| 2（2026-10-10 のウェーブ） | hello-pasta-shell-art | `crates/pasta_sample_ghost` の絵を描くプログラム（`src/image_generator.rs` ほか。消す）・`Cargo.toml`、ルートの `Cargo.toml`・`Cargo.lock`、`shell/master/`、`README.md`、`release.ps1`、`STAGES.md` の部位名の注記、`tests/integration_test.rs` の画像のテスト、生成の手順と出どころの記録。部位名の都合があるときだけ 8 段目の辞書と章 |
| 2（同上） | choice-line-layout | 選択肢の出力（`crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` とそのテスト）、または 9 段目の辞書（`dic/09-choice.pasta`）・章（`book/src/getting-started/09-choice.md`）・`STAGES.md` の 9 段目の行・`tests/tutorial_stages_test.rs` |
| 3 | getting-started-screenshots | 撮影の手順、画像、各章への画像の行、新しいシェルでの 8 段目の実機の確認。`choice-line-layout` と `boot-surface-without-dic` の後に撮る |
| 4（0.4.0 に入れた） | manual-shell-guide | シェルの説明の新しい章、入門の章（`setup.md`・`03-face.md`・`08-touch.md`）からのリンク、`SUMMARY.md` と章の数を決め打ちする検査 |

Wave 1（`hello-pasta-tutorial-stages`・`manual-claudia-theme`）と、ガイドの本文（`getting-started-story-guide`）は完了した。

## リリース（常駐 spec `release-workflow`）

リリースの CI 化（`release-ci`）、常駐 spec の手順の書き換え、CI での初回のリリース（v0.3.8）とその後片付けは、2026-10-10 までにすべて済んだ。タグ `vX.Y.Z` の push だけで公開まで進む。記録は `.kiro/specs/release-workflow/first-ci-release.md` と、同じフォルダの `design.md` にある。**残るのは、失敗した job の再実行の読み方の確認である**（初回は失敗が起きなかったので未確認。失敗が起きたリリースで確かめる）。

- **以後のリリース**: 新しい作業ブランチ（ハーネスのワークツリー）で `/kiro-impl release-workflow` を実行する。失敗した job の再実行の読み方（`design.md`「CI での初回のリリースで確かめること」の 3）を確かめるまでは Opus で実行し、確かめたら Sonnet で実行する（開発者決定 2026-10-10）。
- **席**: リリースの実行中は、`release.ps1`・`release.yml` を触る spec を main へ入れない。
- **運用の注意**（一回限りのセットアップで分かったこと）:
  - Azure のリソースを `az` で作る・変えるには MFA が要る。WAM（Windows のサインイン窓）でログインすると `RequestDisallowedByAzure` で弾かれる。`az config set core.enable_broker_on_windows=false` にしてからブラウザーで `az login` し直す。ワークフローのマネージド ID には関係ない。
  - Marketplace の Members の管理には、公式の CLI が無い（vsce・az・gh のどれも扱えない）。画面から行う。
  - setup-check の `verify-pat` が失敗すると、vsce のエラー文にマネージド ID の識別子が含まれ、ジョブのログに出る。秘密の値ではないので直さない（2026-10-10 決定）。

## 作成支援とマニュアルの保守（2026-10-10 起票）

- `pasta-check-dic-validate` — SSP を起動せずに、ゴーストの `.pasta` の辞書を検査する `pasta_check` のサブコマンド。飛び先の無い Call・選択肢・文中のシーンリンク、モジュール名の衝突、記号を落とした後のシーン名の衝突などを見る。何をエラーにして何を警告にするか、`pasta_check release` の関門にするか、依存の重さ（`pasta_lua` に依存すると、`cargo install pasta_check` が LuaJIT をビルドする）を要件で決める。依存を足すと `Cargo.lock` が変わるので、`hello-pasta-shell-art` の後に置く。マニュアルに章を足す場合は、章の数を決め打ちする検査（`verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`）と `SUMMARY.md` を、章を足すほかの spec（`manual-shell-guide` など）と共有する。内部クレートに依存させる場合は、公開の手順（`release.yml` の `pasta_check` の公開の段）も触るので、リリースの実行中は入れない。
- `manual-print-media-refs` — マニュアルの印刷用ページ（`print.html`）で、デバッグの章の動画が再生できない。ビルドの後に `print.html` の参照だけを書き換え、検査（`verify-static.mjs`）の例外を外す。
- `manual-shell-guide`（2026-10-10 追加・優先度は低い）— 入門ガイドはシェルを「hello-pasta からフォルダごと写す」と教え、中身の説明は `setup.md` の節「シェルの中身」（ファイルの種類と、辞書との接点がサーフェス番号であること）までである。自分の絵に差し替える所から先を、入門の外に章を置いて書く。急がない。

## 入門ガイドの実機確認で見つかったもの（2026-10-10 起票）

`getting-started-story-guide` の完了時の棚卸で起票した。2 回目の棚卸（2026-10-10）で、`manual-link-anchor-check` と `choice-line-layout` をウェーブに入れた。`boot-surface-without-dic` は次点、`baseware-virtual-time` は見送った。

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
- [ ] manual-print-media-refs -- マニュアルの印刷用ページ（`print.html`）で動画の参照が切れているのを、ビルド後の書き換えで直し、検査の例外を外す（2026-10-10 棚卸で起票）。Dependencies: none
- [ ] pasta-check-dic-validate -- SSP を起動せずに `.pasta` の辞書を検査する `pasta_check` のサブコマンド。飛び先の無い参照・モジュール名とシーン名の衝突などを見る（2026-10-10 棚卸で起票）。Dependencies: hello-pasta-shell-art（`Cargo.lock` を触る順番）
- [ ] call-attribute-filter -- Call の属性フィルター構文（`＞シーン＆k＝v`・比較演算子・複数条件）と実行時の絞り込み。Dependencies: scene-attribute-store, failure-output-unification
- [ ] runtime-error-balloon -- スクリプトが止まる実行時エラー（500 の応答）を、辞書を書く人に見えるようバルーンにも出す。やるかどうかを要件の最初に決める（2026-10-10 の棚卸で `failure-output-unification` から分割）。Dependencies: failure-output-unification, shiori-test-support-runtime
- [ ] scene-anchor-link -- 台詞の中の `＠？シーン名`（`「表示名」` も付けられる）を、さくらスクリプトのアンカー `\_a` として出す。クリックで、`OnAnchorSelectEx` からそのシーンへ飛ぶ。選択肢の振り分けを共有し、LSP・VSCode の着色とマニュアルまで揃える（2026-10-08 起票）。Dependencies: failure-output-unification, call-attribute-filter
- [ ] getting-started-screenshots -- 新しいシェルで、入門ガイドの各章にスクリーンショットを載せ、8 段目（触られたときの反応）を実機で確かめる（2026-10-10 の棚卸で `getting-started-story-guide` から分割）。Dependencies: hello-pasta-shell-art, choice-line-layout（9 段目の絵に切れた選択肢が写らないよう、先に直す）, boot-surface-without-dic（準備の章に写る絵が変わる。却下なら外す）
- [ ] manual-link-anchor-check -- マニュアルのリンク検証（`link-check.mjs`）が、本の中のリンクと公開 URL の見出し（`#…`）の実在も見るようにする（2026-10-10 `getting-started-story-guide` の完了時に起票）。Dependencies: none
- [ ] choice-line-layout -- hello-pasta の 9 段目で、1 つ目の選択肢が問いかけと同じ行に続き、吹き出しの右端で切れて見えるのを直す。原因（選択肢の前の改行・辞書・バルーン）の特定から（2026-10-10 同上）。Dependencies: none（`scene-anchor-link` と `sakura_builder.lua` を触る順番に注意）
- [ ] baseware-virtual-time -- SSP の「現在時刻の仮想的変更」を pasta の時報に効かせ、正時を待たずに確かめられるようにする。SSP が仮想の時刻を SHIORI に伝えているかの調査から。外から起こした `OnTalk` でチェイントークが続かない件も扱う（2026-10-10 同上）。Dependencies: none
- [ ] boot-surface-without-dic -- 辞書や `＊OnBoot` が無いゴーストは起動しても立ち絵が出ない。既定で立ち絵を出すかどうかを、やるかどうかから決める（2026-10-10 同上）。Dependencies: none
- [ ] manual-shell-guide -- マニュアルに、シェル（見た目）の説明を足す。見本のシェルを自分の絵に差し替える・表情を足す・当たり判定を足す・バルーンの扱いを、辞書と噛み合う所を中心に書く。0.4.0 に入れた（2026-10-10 開発者の指示で起票）。Dependencies: hello-pasta-shell-art（新しいシェルと当たり判定を題材にする）

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
