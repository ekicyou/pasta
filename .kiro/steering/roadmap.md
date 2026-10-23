# Roadmap

## 概要

pasta は、日本語 DSL（Pasta DSL）で書いた辞書を Lua へトランスパイルし、組込 LuaJIT で実行する「伺か」の SHIORI（`pasta.dll`）と、その周辺ツール（`pasta_check`・LSP・VSCode 拡張・利用者マニュアル）からなる。Phase 1〜11 で、プロパティアクセス・監査・マニュアル・ソースレベルデバッグ・アクターモデル駆動・配布物の形・現行実装の不具合の一掃までを完了した。2026-10-09 までに、リリースの CI 化、シーン名の別名、式の中の値なしの扱い、マニュアルの着せ替え、段階ごとに学べる hello-pasta の辞書も入った（下の「完了フェーズ」）。2026-10-11 までに、hello-pasta の新しいシェル、9 段目の選択肢の表示の直し、マニュアルの見出しへのリンクの検査も入った。2026-10-15 に、リリースの本文を見どころの紹介にした。2026-10-17 までに、撫で・つつきの反応のショートカット、実行時の失敗の出力の一本化、SSP を起動しない辞書の検査、シェルの説明の章、印刷用のページの廃止、結合テストの古いランタイムの写しの撤去も入った。同じ日に、止まる実行時エラーのバルーン表示と入門ガイドの絵 16 枚が入り、v0.4.0（主題「入門者にやさしく」）を出した。2026-10-18 に、辞書の検査の端のケース、入門ガイドと SSP のずれ（SSP の仮想の時刻を時報に効かせる）、窓際のぱすたさん（ghost_dev）からの要望の 2 本（アクターごとの表示の装い・縦書きのバルーンの列の扱い）が入り、v0.4.1 を出した。2026-10-19 までに、ゴーストから DLL の関数を LuaJIT の FFI で呼ぶ仕組み（`[loader] ffi`）、デバッグのポートが取れないときにデバッグなしで起動を続ける直し、pasta.dll のアイコンが入った。2026-10-23 までに、`pasta_check` を GitHub Release から単体で落とせるようにする直し（`pasta_check.zip`）、シーンからランダムトークと時報を止めたり再開したりする関数（`自動トーク停止`／`自動トーク再開`）、MCP から名前か位置で指名したシーンを再生するイベント（`OnPastaScene`・`OnPastaSceneAt`）が入った。同じ日に、外から `＞名前` の呼び出しとグローバル変数の代入ができるイベント（`OnPastaCallFunction`・`OnPastaSetSaveVariable`）、`REG:register` で 1 つのイベントにハンドラを重ねる仕組み、台詞を出さないイベントでチェイントークの続きが消えない直し、台詞の自動ウェイトと budoux の改行を書記素クラスタ単位にする直しも入った。

現在の主題は 4 つある。

- **0.4.2: 致命的でない不具合**。はじめの範囲の 7 本（2026-10-18・2026-10-19・2026-10-20・2026-10-22 開発者の決定。`lua-cmodule-robustness`・`pasta-check-release-asset`・`ontalk-suppression`・`scene-play-event`・`external-control-events`・`reg-handler-chain`・`chain-talk-interrupt`）と、0.4.1 の後に届いた不具合の `talk-grapheme-clusters`（組み合わせの絵文字が自動ウェイトで割れる）と、2026-10-23 に起票した `pasta-skills-release-asset`（pasta のスキル 3 本を `pasta-skills.zip` にしてリリースに添える。開発者の依頼）は、2026-10-23 までに全部完了した。残りは 0.4.2 のリリースである（下の「0.4.2 の範囲」）。
- **0.4.3: shiori4 の SHIORI3 互換モード・おすすめメニュー・ponytail 監査の指摘の修正**（2026-10-23 開発者の決定）。0.4.2 から移した `shiori4-v3-portable`（まだ実施できないので、0.4.3 の最後）、版の決まっていなかった `recommend-menu-scenes`、ponytail 監査の指摘を直す 10 本（`ponytail-*`。2026-10-22 に決めた「版に縛らない隙間の spec」をやめた）を入れる（下の「0.4.3 の範囲」と「ponytail 監査の指摘の修正」）。2026-10-21 に起票した `scene-exact-match`・`scene-pick-modes` と、2026-10-23 に起票した `pasta-dll-multiarch` は、版がまだ決まっていない。
- **Phase 13: areka でノベルゲーム（0.5.0）**。2026-10-10 の discovery で、合格ライン・通り道・台本の書き方・巻き戻しの方式を決め、spec を起こした（2026-10-17 の時点で 11 本）。属性セマンティクス（シーン属性の保持と読み出し・Call の属性フィルター）も、この道筋に入った。
- **Phase 11 の残り**: 文中のシーンリンク（`scene-anchor-link`）。どの版に入れるかは、まだ決めていない（下の棚卸の「開発者の判断が要るもの」）。

Phase 12（初心者向けの入門ガイド）は、2026-10-17 の入門ガイドの絵で完了した（「完了フェーズ」）。

このほか、リリース手順（spec `release-workflow`）の CI に合わせた書き換えと、CI での初回のリリース（v0.3.8）は済んだ（2026-10-10）。リリース手順は `release` スキル（`/release`）へ移し、spec `release-workflow` は完了にした（2026-10-11）。

**優先順位（開発者の方針）**: 0.4.0（主題「入門者にやさしく」）は 2026-10-17 に、0.4.1（致命的でない不具合と、窓際のぱすたさんからの要望の 2 本）は 2026-10-18 に出した。次の節目は 0.4.2（致命的でない不具合。2026-10-18）、その次の 0.5.0 の目標は、areka でノベルゲームが作れるようになること（2026-10-10）。0.4.2 の後は 0.4.3（shiori4 の SHIORI3 互換モード・おすすめメニュー・ponytail 監査の指摘の修正。2026-10-23）。ponytail 監査の指摘の修正は、2026-10-22 に版に縛らない隙間の spec にしたが、2026-10-23 に 0.4.3 に入れると決め直した。ウェーブは、今のゴールの範囲の spec だけで組む（2026-10-16。ponytail の spec の例外は、0.4.3 に入れたので要らなくなった）。致命的な不具合は、見つかりしだい割り込ませる。致命的でない不具合は、次の 0.4.x に入れるかを開発者が決める（2026-10-17 に 0.4.1 へ回した方針の続き。2026-10-18 に `lua-cmodule-robustness` を 0.4.2 に入れた）。

## 運用ルール

- `## Specs (dependency order)` は未完了の spec 専用（`/kiro-spec-batch`・`/kiro-spec-status` が読む）。完了したら `[x]` にし、次の棚卸で「完了フェーズ」へ畳む。
- Dependencies は論理的な依存に加え、**同じソースファイル（関数・領域）を触る spec を同じウェーブに置かない**ための順序を含む。同じウェーブの spec は、互いのソースを触らずに並走できる。マニュアルの同じページの別の節を触ることと、Lua のテストの一覧（`crates/pasta_lua/tests/lua_specs/init.lua`）に 1 行ずつ足すことは許し、後から入る側が rebase で合わせる（テストの一覧は 2026-10-22 の棚卸で足した）。
- 個々の経緯・決定は各 spec（`.kiro/specs/completed/<name>/`）と git 履歴が持つ。ここには要約だけを置く。

## 前提として確定している方針

後続の spec が前提にするもの。詳細は各完了 spec とマニュアルの内部設計パートにある。

- **マニュアルが権威**: mdBook（`book/`）が利用者向け情報と内部設計の唯一の権威。挙動を変えたら同じ変更でマニュアルを直し、`node book/tools/gen-skill-refs.mjs` でスキル `references/` を再生成する（CI の `--check`・`link-check.mjs` が鮮度とリンクを見る）。
- **現行実装を正とする**: 旧仕様・吸収台帳の記述は材料であり規範ではない。
- **デバッグ**: Rust ホスト型 DAP バックエンドを `pasta_lua` に内蔵。単一 DLL・opt-in・127.0.0.1 固定。ポートが取れなければ、警告を出してデバッグなしで起動を続ける。
- **FFI**: pasta.toml の `[loader] ffi = true`（既定は無効）で LuaJIT の FFI を開く。`ffi.load` は `pasta_lua` の `runtime/ffi_load/` の置き換えが受け、深いパス・ANSI に無い文字のフォルダ・隣の依存 DLL でも DLL を読む。C モジュールの `require` は使えない。
- **エンジン**: コアは宿主非依存でアクタースレッドに pin。シーンキックは即時再生のみ。
- **LuaJIT 2.1**: `coroutine.close`・`<close>` が無い。中断したコルーチンは参照を外して GC に任せる。
- **シーン名の別名**: pasta.toml の `[scene.alias]` で定義する。未定義なら `会話 → OnTalk` の 1 件が既定。完全一致の 1 段だけで、キー正規化の前段で登録・検索の両側に効く。
- **式の中の値なし**: 式の中の nil は、算術なら 0、連結なら空文字列とみなし、ログを出さない。
- **リリース**: タグ `vX.Y.Z` の push で `.github/workflows/release.yml` が検証・ビルド・公開まで行う。ビルドした成果物は git で追跡しない。シェルの画像は追跡を続ける。配布物は `pasta.dll.zip`・`hello-pasta.nar`・VSIX・`pasta_check.zip`・`pasta-skills.zip` の 5 つで、GitHub Release には、それにリリースの全件の一覧を足した 6 つを添える（`pasta-skills.zip` は 0.4.2 で `pasta-skills-release-asset` が足した）。
- **hello-pasta の辞書**: 1 段 1 ファイル（`dic/01-boot.pasta`〜`12-lua.pasta`）で、段階表は `crates/pasta_sample_ghost/STAGES.md`。全段が起動することをテストが確かめる。
- **入門ガイド**: Claudia が全編を語る（`getting-started` に限った執筆規約の例外）。段の章と段階辞書を逐語で照合する（`book/tools/tutorial-check.mjs`）。辞書を変える spec は、同じ変更で入門の章の作例を直す。絵は `book/SCREENSHOTS.md` の手順で撮り直す。
- **マニュアルの見た目**: mdBook のまま「Claudia のマニュアル」。台詞は顔アイコン付きの部品で書く（記法は `book/AUTHORING.md`）。印刷用のページ（`print.html`）は作らず、章の印刷はテーマが受け持つ。
- **実行時の失敗**: シーンが生きている間の失敗は、`pasta/failure.lua` の `FAILURE.report` が、その場所に `【種類：対象（説明）】` の表記とログ 1 行を出す。切り替えは pasta.toml の `[logging] balloon_warning`。新しい種類は `FAILURE.LABEL` に足す。スクリプトが止まるエラーは `pasta/shiori/stop.lua` が受け、`【スクリプトのエラー：…】`・`【時間切れ：…】` の表記を `200 OK` と `X-Error-Reason` の行で返す（`balloon_warning = false` なら、これまでどおり 500）。
- **辞書の検査**: `pasta_check validate` が pasta.dll と同じ規則で辞書を検査し、`pasta_check release` の段 4 の関門と VSCode 拡張の保存のときの診断にもなる。検査を足すときは報告の種類 `FindingKind` に足す。
- **マニュアルの章の数**: 章を足す spec は、章の数を決め打ちする 3 つの検査（`book/tools/verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`）を合わせる（2026-10-23 の時点で 63 章）。
- **辞書の検査の順**: 飛び先などの検査（`pasta_check` の `validate/references.rs`）は、読めない辞書が無いとき（`skip_reasons` が空のとき）だけ走る。検査を足す spec は、この順を保つ。
- **アクターの装い**: アクターごとの表示の装いは pasta.toml の `[actor."名前"].font` に書き、タグ列（`\f[…]`・`\f[default]`）は `pasta_core` の `ActorFont` が作る。出し入れは `pasta/shiori/actor_style.lua` が受け持つ。1 つのスポットを複数のアクターで共有できる。
- **SSP の仮想の時刻**: `pasta/shiori/clock.lua` が受け持つ。SSP 2.9.18 の TimeMachine は pasta.dll の読む OS の時計まで仮想にするので、時刻の飛びで次の正時を決め直し、間隔は飛びを差し引いて数える。
- **budoux の列**: budoux は、列を動かすタグ（`\n`・`\n[…]`・`\c`・`\_l[…]`）の所で数え直し、列の字数を吹き出しごとに持ち越す（1 つの応答の中だけ）。
- **自動トークの停止**: `GLOBAL.pause_auto_talk`／`resume_auto_talk`（別名 `自動トーク停止`／`自動トーク再開`）。印は `STORE.auto_talk_paused` で、セッションの間だけ効く。止めている間は `virtual_dispatcher.lua` が OnTalk・OnHour・止まっている場面の続き（キックと `OnPastaScene` の 2 ビート目からも）を出さない。1 ビート目は流す。止めている間にシーンが `yield` で続きを作ると、警告して再開する（`act.lua` の `ACT_IMPL.yield`）。
- **シーンを再生するイベント**: `OnPastaScene`（大域・ローカルの名前の完全一致）と `OnPastaSceneAt`（ファイルと行）を `pasta/shiori/event/scene_play.lua` が受ける。デバッグの有効は設定の値で判断し、ポートは要らない（`@pasta_debug`）。見つからないときは `FAILURE.report` の表記を出す。外からのイベントの口を足す spec は、この置き方に揃える。
- **外からのイベント**: `OnPastaCallFunction`（`＞名前` の呼び出し）と `OnPastaSetSaveVariable`（グローバル変数の代入。値は JSON、`null` で消す）を `pasta/shiori/event/external_control.lua` が受ける。いつでも有効。見つからないときは `OnPastaScene` と同じ失敗の表記を出す。
- **ハンドラの重ね**: `REG:register(id, fn)` で 1 つのイベントにハンドラを重ねる（`pasta/shiori/event/register.lua`。`REG` のメタテーブルの関数で、表のキーではない）。新しく登録したものから呼び、`nil`・`false` なら次へ、`true` なら 204、文字列・コルーチンならその応答にする。全部が見送ったら `REG[id]`、それも無ければイベントと同じ名前のシーン。ハンドラごとに新しい act を渡す。
- **チェイントークの続き**: 台詞（`talk`・`choice`）を出さなかったシーンは、待っている続き（`STORE.co_scene`）に触れない（判定は `sakura_builder.lua` の `BUILDER.speaks`）。台詞を出したシーンが割り込むと続きを捨て、`＊チェイントーク中断` があれば、割り込んだシーンの台詞をその台詞で置き換える。キック・`OnPastaScene`・`OnPastaSceneAt`・エラーは、`STORE.co_preempt` の印で黙って捨てる。外からクリックやイベントを受ける口を足す spec は、どちらの扱いにするかを決める。
- **台詞の 1 文字**: 自動ウェイト・`chars_*` の判定・budoux の改行は、拡張書記素クラスタを 1 文字として数える（`unicode-segmentation`。`sakura_script/tokenizer.rs`・`line_breaker.rs`）。

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
| —（0.4.1） | 辞書の検査の端のケースと代用品の置き換え、SSP の仮想の時刻を時報に効かせる（入門ガイドと SSP のずれ）、アクターごとの表示の装い（`[actor."名前"].font`）、縦書きのバルーンの列の扱い（budoux の数え直しと列の持ち越し） | pasta-check-validate-edges, tutorial-ssp-gaps, actor-display-style, vertical-balloon-lines |
| —（0.4.2） | ゴーストから DLL の関数を LuaJIT の FFI で呼ぶ（`[loader] ffi`・`ffi.load` の置き換え。深いパス・ANSI に無い文字のフォルダ・隣の依存 DLL）、`pasta_check` を GitHub Release から単体で配る（`pasta_check.zip`）、シーンから自動トークを止める・再開する（`自動トーク停止`／`自動トーク再開`）、MCP から指名したシーンを再生する（`OnPastaScene`・`OnPastaSceneAt`）、外から `＞名前` を呼ぶ・変数を書く（`OnPastaCallFunction`・`OnPastaSetSaveVariable`）、`REG:register` でハンドラを重ねる、台詞を出さないイベントでチェイントークの続きを捨てない（割り込みには `＊チェイントーク中断`）、台詞の自動ウェイトと改行を書記素クラスタ単位にする | lua-cmodule-robustness, pasta-check-release-asset, ontalk-suppression, scene-play-event, external-control-events, reg-handler-chain, chain-talk-interrupt, talk-grapheme-clusters |

ロードマップを置く前の初期開発（DSL・トランスパイラ・ランタイム・SHIORI・alpha リリースなど）の spec も `.kiro/specs/completed/` にある。

### 常駐 spec（再実行型・`completed/` へ移さない）

- `review-improvement-loop` — レビュー領域 × 7 次元の改善ループ

### 却下（2026-10-01）

- `pasta-sstp-live-output`（SSTP/`\![raise]` による push 出力）— 目的はシーンキックで達成済み。新しい動機が生じたら、その動機から新規に起票する。
- `pasta-authoring-window`（`*.pasta` 編集・プレビュー専用ウィンドウ）— VSCode で実現済み。VSCode 非依存のオーサリング環境が必要になったら再検討する。

## 0.4.2 の範囲（2026-10-18 開発者の決定）

0.4.1 は 2026-10-18 に出した（リリース CI の run 39123004799。全部の job が 1 回目で通った）。辞書の検査の端のケース、入門ガイドと SSP のずれ、窓際のぱすたさん（ghost_dev）からの要望の 2 本（アクターごとの表示の装い・縦書きのバルーンの列の扱い）が入った。範囲と経緯は、git の履歴と各完了 spec にある。

| 順 | spec | 要件定義 | 規模 | 始められる時期 |
| -- | ---- | -------- | ---- | -------------- |
| 1 | lua-cmodule-robustness | Opus（要件は確定） | 10 前後 | 完了（2026-10-18。2026-10-18 のウェーブ） |
| 2 | pasta-check-release-asset | Opus | 7〜8 | 完了（2026-10-22。2026-10-22 のウェーブ） |
| 3 | ontalk-suppression | Opus | 5〜6 | 完了（2026-10-22。2026-10-22 のウェーブ。キックと `OnPastaScene` の続きは止め、1 ビート目は流す） |
| 4 | scene-play-event | Opus | 7〜8 | 完了（2026-10-23。2026-10-22 のウェーブ。MCP の `raise_event` は GET・`SecurityLevel: local` で届くと実機で確かめた） |
| 5 | external-control-events | Opus | 5〜6 | 完了（2026-10-23。2026-10-23 のウェーブ。見つからないときは `scene-play-event` と同じ失敗の表記） |
| 6 | chain-talk-interrupt | Opus | 6〜7 | 完了（2026-10-23。2026-10-23 のウェーブ。中断の台詞は反応の代わりに出す） |
| 7 | reg-handler-chain | Opus | 5〜7 | 完了（2026-10-23。2026-10-23 のウェーブ。`REG` にメタテーブルを付け、鎖のハンドラはハンドラごとに新しい act を受け取る） |
| 8 | talk-grapheme-clusters | Opus | 8〜11 | 完了（2026-10-23。0.4.1 の後に届いた不具合を 0.4.2 に入れた。`unicode-segmentation` を足した） |
| 9 | pasta-skills-release-asset | Opus | 6〜8 | 完了（2026-10-23。2026-10-23 のウェーブ。`release.ps1` の段 6 で作り、リリース CI がスキルの写しの鮮度と zip の中身を確かめてから添える） |
| 10 | 0.4.2 のリリース（`release` スキル） | Opus | — | 上の全部の後 |

- **0.4.2 に入れたもの（2026-10-18 完了）**: `lua-cmodule-robustness`（`lua-require-robustness` の続き）。2026-10-18 の要件の話し合いで、C モジュールの `require` の道（Lua の C API を pasta.dll から書き出す必要があり、pasta のゴーストが 2 体いると別の pasta.dll に結びつく危険があった）をやめた。代わりに、pasta.toml の `[loader] ffi = true`（既定は無効）で FFI を開き、`ffi.load` を包んで、深いパス・ANSI に無い文字を含むフォルダ・隣の依存 DLL でも DLL を読めるようにする。経緯と実測は spec の brief にある。
- **0.4.2 に足したもの（2026-10-19 開発者の決定）**: `pasta-check-release-asset`。Rust を入れていない作者が `pasta_check` を使えるように、x64 の `pasta_check.exe` とライセンス表示を `pasta_check.zip` にして、4 つ目の配布物として GitHub Release に載せる。作るのは `release.ps1` の新しい段で、手元の `release.bat` とリリース CI が同じ一式を作る。`cargo install` の道は残す。署名・x86 版・インストーラーは作らない。隙間の spec の `ponytail-manual-ci` と `release.yml`・`github-release.ps1` を共有するので、こちらが先に入り、`ponytail-manual-ci` が後から合わせる。
- **0.4.2 に足したもの（2026-10-19 開発者の決定）**: `ontalk-suppression`。初回起動トークのように何回かに分けて進むトークを、ランダムトークと時報が崩さないようにする。`GLOBAL` に `pause_auto_talk`／`resume_auto_talk` を置き、日本語の別名 `自動トーク停止`／`自動トーク再開` を付ける（`close_ghost`／`ゴースト終了` の先例どおり。DSL からは `＞自動トーク停止`）。止めている間は OnTalk・OnHour・止まっている場面の続きが始まらない。セッションの間だけ効き、保存しない。再開すると、ランダムトークの間隔を再開した時刻から数え直す。隙間の spec の `ponytail-lua-scripts` と `virtual_dispatcher.lua` を共有するので、こちらが先に入り、あちらが後から合わせる。0.5.0 の `novel-talk-flow` は、ノベル向けの停止にこのスイッチを使い回す。
- **0.4.2 に足したもの（2026-10-19 開発者の決定）**: `scene-play-event`。AI エージェントが SSP の MCP サーバー（`raise_event`）からゴーストをデバッグできるように、指名したシーンを再生する組み込みのイベントを 2 つ足す。`OnPastaScene`（Reference0＝大域、Reference1＝ローカル。書いた名前どおりの完全一致で、番号の `_1` は付けない）と、`OnPastaSceneAt`（Reference0＝ファイル、Reference1＝行。右クリックの実行と同じ引き方）である。1 ビート目をその場で返すので、MCP の結果に台本が載る。続きと中断はキックと同じ。デバッグを有効にしたときだけ動き、配るゴーストの挙動は変えない。`ontalk-suppression` との接点は、止めている間はキックと同じく続き（2 ビート目から）も止まり、1 ビート目は流す、に決まった。デバッグの有効は設定の値で判断する（2026-10-23 完了）。
- **0.4.2 に足したもの（2026-10-19 開発者の決定）**: `external-control-events`。SSP の MCP（`raise_event`）などの外から、イベントを 1 つ送るだけで辞書の `＞XXX` と同じ呼び出し（`OnPastaCallFunction`。Reference0 が名前、Reference1 以降が文字列の引数。探し方はシーンの外の `＞XXX` と同じ）と、グローバル変数の代入（`OnPastaSetSaveVariable`。Reference1 は JSON の値、`null` で消す）ができるようにする。主な使い道は、試験の間だけ `自動トーク停止` を外から呼ぶこと。いつでも有効で、止めたい作者はハンドラを上書きする。値の読み出しは範囲の外。`ontalk-suppression` とはマニュアルの `lua/shiori-events.md` だけを共有し（節は別）、並べられる。`scene-play-event` の後に入る。どちらも「外からのイベントを受けて何かする」口で、既定のハンドラの置き方・`event/init.lua` の `require`・マニュアルの既定ハンドラーの表と見つからないときの出し方を、先に入るあちらに揃える（ハンドラから先の Lua のコードは別物）。2026-10-23 完了。
- **0.4.2 に足したもの（2026-10-20 開発者の決定「次のリリースに含める」）**: `chain-talk-interrupt`。窓際のぱすたさんで、`＞チェイントーク` の後半が出なかった。マウスのシーンが何も出さずに終わる（204）たびに、`set_co_scene` が待っている続きを捨てていた。台詞（`talk`・`choice`）を出さなかったシーンは続きに触れないようにする。台詞を出したシーンが割り込んだら続きは忘れ（再開の仕組みは作らない）、作者が `＊チェイントーク中断`（完全一致）を書いていれば、割り込んだシーンの台詞の代わりに出す（設計の話し合いで「前に添える」から改めた）。キック・`OnPastaScene`・`OnPastaSceneAt`・エラーで止まったシーンは、今どおり捨て、中断の台詞は出さない。`scene-play-event` の後に入り、隙間の spec の `ponytail-lua-scripts`（同じ `set_co_scene` を整理する）より先に入る。2026-10-23 完了。
- **0.4.2 に足したもの（2026-10-22 開発者の決定「次のリリースに含める」）**: `reg-handler-chain`。窓際のぱすたさんは `REG.OnSecondChange` を 3 か所で包み合い、どれが外側になるかが読み込みの順に頼っていた。`REG:register("OnSecondChange", fn)` で重ねて登録できるようにする。新しく登録したものから呼び、`nil`・`false` なら次へ回り、`true` なら止めて 204、文字列・コルーチンなら止めてその応答にする。全部が見送ったら、今どおり `REG[id]`（組み込みの既定か作者の代入）、それも無ければイベントと同じ名前のシーンを呼ぶ。`register` は外す関数を返す。`register` は表のキーにせず、`EVENT.fire` は `rawget` で引く。優先度の数と、全部に見せる見張り役の口は作らない。バックログの「`REG` に 1 つのイベントのハンドラを重ねる仕組み」を引き取った。`EVENT.fire` のある `event/init.lua` を `external-control-events`・`chain-talk-interrupt` と共有するが、持ち場（この spec は `EVENT.fire` のハンドラの引き当て）が重ならないので、2026-10-23 のウェーブで並べた。2026-10-23 完了。
- **0.4.2 に足したもの（2026-10-23 開発者の意向「今やってるマイナーバージョンアップに入れちゃってもよいかな」）**: `shiori4-v3-portable`。2026-10-23 の 3 者会談（pasta・areka・shiori4 の各セッション）で、shiori4 の C ABI の規約とホストとの取り決めを合意した（正本は shiori4 の `abi-conventions`・`host-protocol`）。pasta の対応は 2 段に分ける（開発者の方針）。1 段目のこの spec は、SHIORI4 のためというより「SHIORI/3.0 を Windows 以外で動かす別口」で、`shiori_get_api(3)`（SHIORI3 互換モード。要求と応答は SHIORI/3.0 の全文）を書き出し、アクターの実行器を OS に頼らない形にし、Linux・macOS でビルドとテストを通す。Windows の今の入口は残す。pasta のエンジンは状態をインスタンスごとに持ち、グローバルな状態を避ける（2026-10-23 開発者の決定。グローバルな状態に頼る怪しい不具合を避けるため）。shiori4 の入口で同じプロセスの 2 つ目の `load` を受けるかは、別の判断として要件で決める（areka は SHIORI ごとに helper のプロセスを起こすので、今は来ない）。2 段目の版 4 への本格対応は `shiori4-v4-host`（Rust の土台）と `shiori4-v4-lua-api`（Lua と DSL の口）で、0.5.0 に入れる（2026-10-23 の棚卸で 2 本に分けた）。この spec は、まだ実施できないので、0.4.3 へ移した（2026-10-23 開発者の決定「これは 0.4.3」。下の「0.4.3 の範囲」）。`ponytail-shiori` と `windows.rs`・`actor/` を共有するので、同じウェーブに置かない。
- **0.4.2 に足したもの（2026-10-23 開発者の決定）**: `talk-grapheme-clusters`。台詞の「1 文字」を拡張書記素クラスタにし、組み合わせの絵文字・結合文字の途中に自動ウェイトも budoux の改行も入れない。`chars_*` に組み合わせの絵文字を書ける。半角カナの濁点つき（`ｶﾞ`）が 1 文字になる。`unicode-segmentation` を足した（2026-10-23 完了）。
- **版がまだ決まっていないもの**: `scene-anchor-link`（下の棚卸）。
- **版がまだ決まっていないもの（2026-10-23 起票）**: `pasta-dll-multiarch`。x64 版と arm64 版の pasta.dll をビルドし、32bit 版と同じリリースで出す。見本ゴーストの .nar には、areka の探し方（x64 は `pasta.x64.dll` → `pasta.dll`、arm64 は `pasta.arm64.dll` → `pasta.x64.dll` → `pasta.dll`）に合う名前で同梱する。接尾辞の無い `pasta.dll` は SSP 向けの 32bit 版のまま。`pasta_check` は名前も arch も気にしないので今のまま使える。手を入れるのは `release.ps1`・`.cargo/config.toml`（aarch64 の `crt-static`）・配布物とリリース CI の検査・マニュアル（`[loader] ffi` で作者の DLL の arch が合わないと読めない注意）。aarch64 で LuaJIT がビルドできるかは確かめていない。`shiori4-v3-portable` の後（areka は shiori4 の入口で読むため）。版は開発者が後で決める（2026-10-23）。
- **0.4.2 に足したもの（2026-10-23 開発者の依頼と決定）**: `pasta-skills-release-asset`。ゴーストの作者が自分の Claude に入れられるように、pasta のスキル 3 本（`pasta-ghost-authoring`・`pasta-lua-coding`・`pasta-check`）とリポジトリの `LICENSE` を `pasta-skills.zip` にして、5 つ目の配布物として GitHub Release に載せる（添付は 6 つ）。展開して `.claude/skills/` に置けばそのまま使える形にする。名前に版を付けない。作るのは `release.ps1` の新しい段で、手元の `release.bat` とリリース CI が同じ一式を作る。スキルごとの zip・開発用のスキル（`kiro-*`・`release`）・VSIX への同梱は範囲の外。`release` スキルの添付の数え方も 5/5 に直すので、分類器が止めたら開発者の許可を求める。リリースの席（`release.yml`・`github-release.ps1`）を持つので、0.4.3 の `ponytail-manual-ci` と、版の決まっていない `pasta-dll-multiarch` が後から合わせる。
- **これから見つかる不具合**: 致命的（起動しない・辞書の内容が失われる・配布物が壊れる・リリースが通らない）なら、すぐに割り込ませる。それ以外は、0.4.2 に入れるかを開発者が決める。
- **0.5.0 との関係**: 0.5.0 の先頭（`novel-areka-contract`・`scene-attribute-store`・`novel-variable-scope`）は、0.4.2 の残りの `pasta-skills-release-asset`（リリースの手順と CI だけ）とファイルが重ならない。並べるには、0.5.0 を今のゴールに含める開発者の決定が要る（下の棚卸の「開発者の判断が要るもの」）。

### 0.4.1 の後に届いた不具合（2026-10-18 起票）

v0.4.1 を出した後に、「ぱすた開発」セッション（窓際のぱすたさん）が実機で見つけて知らせてきた 2 件。どちらも致命的でない。spec にしたのは 1 本で、0.4.2 に入れて 2026-10-23 に完了した。もう 1 件は、表示の誤りが無いと分かり、取り下げた。

- **`talk-grapheme-clusters`（spec。2026-10-23 完了）**: 自動ウェイトと budoux の改行が、コードポイントごとに台詞を切っていた。組み合わせの絵文字（`🙆‍♀️`・`✌️`）の途中に `\_w` が入り、縦書きのバルーンで絵文字が割れて描かれた。`sakura_script/` の `tokenizer.rs`・`line_breaker.rs`・`wait_inserter.rs` を書記素クラスタ単位にし、`unicode-segmentation` を足して直した。
- **着せ替えの復旧が前のパーツを出す件（取り下げ）**: 起票したが（`dressup-restore-stale-parts`）、同じ日に取り下げた（2026-10-18 開発者の決定「ぱすた側が必要としていないなら取り下げ」）。
  - ログの調べ直しで、表示が壊れた実例は無いと分かった。腕が残る症状は推測だったと、報告の側が訂正した。
  - ゴースト側は、復旧の出力が短く済むので、`,0` の明示の OFF を残す。
  - 残る粗さは、バックログの「着せ替えの復旧の出力の膨らみ」に書いた。

### 0.5.0 の目標（2026-10-10 開発者の決定）

別リポジトリで開発中の areka で、ノベルゲームが作れるようになること。pasta の側に何が要るかは、2026-10-10 の discovery で洗い出した。合格ライン・決めたこと・spec の割り方と順番は、下の「Phase 13: areka でノベルゲーム（0.5.0）」にある。

## 0.4.3 の範囲（2026-10-23 開発者の決定）

0.4.2 を出した後の版。2026-10-23 に、0.4.2 から `shiori4-v3-portable` を移し（「まだ実施できない」「これは 0.4.3」）、版の決まっていなかった `recommend-menu-scenes` と、隙間の spec だった ponytail の 10 本を入れた（開発者の決定）。`shiori4-v3-portable` は、始められるようになるまで最後に置く。ほかは 0.4.2 を出した後から始められる。

| 順 | spec | 要件定義 | 規模 | 始められる時期 |
| -- | ---- | -------- | ---- | -------------- |
| 1 | ponytail-lua-loader | Opus | M | 0.4.2 の後（データが失われる Must fix。最初に入れる） |
| 2 | ponytail-check-release | Opus | M | 0.4.2 の後（データが失われる Must fix。最初に入れる。`pasta-check/SKILL.md` の編集は開発者の許可が要る） |
| 3 | recommend-menu-scenes | Opus | 10〜12 | 0.4.2 の後（`ponytail-lua-loader`・`ponytail-lua-scripts`・`ponytail-core-registry` と同じウェーブに置かない） |
| 4 | ponytail-dsl-parser | Opus | M | 0.4.2 の後 |
| 5 | ponytail-core-registry | Opus | S-M | 0.4.2 の後（`Cargo.lock` の席は `ponytail-editor-tooling` が持つ） |
| 6 | ponytail-lua-transpiler | Opus | S-M | 0.4.2 の後 |
| 7 | ponytail-lua-debug | Opus | L | 0.4.2 の後 |
| 8 | ponytail-lua-scripts | Opus | M | 0.4.2 の後 |
| 9 | ponytail-editor-tooling | Opus | L | 0.4.2 の後 |
| 10 | ponytail-manual-ci | Opus | M | `pasta-skills-release-asset` の後（`release.yml`・`github-release.ps1` を共有する） |
| 11 | shiori4-v3-portable | Opus | 14〜18 | 最後（まだ実施できない。2026-10-23 開発者の決定。`pasta_shiori` の `windows.rs`・`actor/`、ルートの `Cargo.toml`・`Cargo.lock`、`build.yml` を触る） |
| 12 | ponytail-shiori | Opus | L | `shiori4-v3-portable` の後（`windows.rs`・`actor/` を共有する。入った後の main でギャップ分析をやり直す） |
| 13 | 0.4.3 のリリース（`release` スキル） | Opus | — | 上の全部の後 |

- **0.4.3 に入れたもの（2026-10-23 開発者の決定）**: `recommend-menu-scenes`。右クリックメニューの「おすすめ」「ポータル」を、辞書のシーンだけで書けるようにする。`＊おすすめメニュー`・`＊相方おすすめメニュー`・`＊ポータルメニュー`（シーン名の別名の既定に足す。完全一致で引く）の中で `＞おすすめ項目（名前、URL、選んだときのシーン、バナー）` を積み、リソースの問い合わせのときにシーンをその場で最後まで動かして `\1`・`\2` の値を返す（チェイントークの続きに触れない）。同じ名前のシーンが複数あれば、メニューを開くたびにランダムに選ばれる。ボタンの名前は決まった名前の単語（`＠おすすめボタン名`）。Lua の `REG` が勝つ。あわせて、`[scene.alias]` を書くと既定の表が丸ごと消える決まりを、書いたキーだけ置き換える決まりに改める（空の配列で既定を消す・見出しだけなら既定のまま・既定とぶつかればエラー）。pasta.toml の表で書く案は、開発者が退けた（「ゴーストの挙動は DSL で書く」「pasta.toml はとっつきにくい」）。先に要る 3 本（`external-control-events`・`chain-talk-interrupt`・`reg-handler-chain`）は 2026-10-23 に入った。`shiori4-v3-portable` とはソースを共有しない。`ponytail-lua-scripts`・`ponytail-core-registry`・`ponytail-lua-loader` とはファイルを共有するので、同じウェーブに置かない。リソースの問い合わせの失敗はログだけにし（失敗の表記を出すと値に混ざる）、選んだ後のシーンは普通の割り込みとして扱う（`STORE.co_preempt` の印を付けない）のが、2026-10-23 の再測定の推奨である。
- **0.4.3 へ移したもの（2026-10-23 開発者の決定）**: `shiori4-v3-portable`。中身と経緯は、上の「0.4.2 の範囲」の箇条にある。
- **0.4.3 に入れたもの（2026-10-23 開発者の決定）**: ponytail の 10 本。中身と決めたことは、下の「ponytail 監査の指摘の修正」にある。
- ウェーブは、上の表の順と、ソースの重なり（下の「ponytail 監査の指摘の修正」の境界の戦略）で組む。ponytail の 10 本は互いにソースを共有しない。

## ponytail 監査の指摘の修正（2026-10-19 起票・2026-10-22 に隙間の spec へ・2026-10-23 開発者の決定で 0.4.3 へ）

2026-10-19 に、プラグインの `/ponytail:ponytail-audit` を、クレートと領域ごとに 8 つ並べて走らせた。見たのは、正しさ・安全・負荷・テスト・速さ・無駄の順。指摘は約 100 件で、そのうち Must fix は 9 件だった。それを、ソースを共有しない 10 本に切った。はじめは専用の版 0.4.3 で着手する決定だったが（2026-10-19）、0.5.0 を待たせないように、版に縛らない「隙間の spec」に置き直した（2026-10-22）。2026-10-23 に、もう一度 0.4.3 に入れると決めた（開発者の決定。上の「0.4.3 の範囲」）。

### 隙間への入れ方（2026-10-22 開発者の決定。2026-10-23 に 0.4.3 へ入れたので、0.5.0 との重なりの見込みのほかは使わない）

- 0.4.2 を出した後から入れる（0.4.3 でも同じ）。0.4.2 の残りの spec（`chain-talk-interrupt`・`external-control-events`・`reg-handler-chain`）が先に入る約束は果たした（3 本とも 2026-10-23 に入った）。完了した 3 本（`ontalk-suppression`・`scene-play-event`・`pasta-check-release-asset`）からの申し送りは、各 ponytail の brief の 2026-10-23 の再測定の節にある。
- 「ゴールを超えた spec はウェーブに入れない」（2026-10-16）の例外にする。同じウェーブの 0.5.0 の spec とソースを共有しないときだけ、ponytail の spec を足す。優先はいつも 0.5.0 で、ponytail の spec のために 0.5.0 の spec を後ろへずらさない。
- できた spec は、そのとき次に出す版に乗る。ponytail の spec を待つための版は作らない。
- データが失われる Must fix（`ponytail-lua-loader` の読めないセーブ、`ponytail-check-release` の作者のフォルダの消去）を持つ 2 本は、最初の空いた所に入れる。
- 0.5.0 の spec とぶつかりそうな相手（見込み。ウェーブを組むたびに確かめる）:
  - `ponytail-dsl-parser`：文法を足す `call-attribute-filter`・`scene-exact-match`・`scene-pick-modes`・`scene-anchor-link`、宣言行の属性を読む `scene-attribute-store`（`parse_scene.rs`。2026-10-23）
  - `ponytail-lua-transpiler`：`code_gen` を触る `cue-command-runtime`・`call-attribute-filter`、`scene-attribute-store`（`scope_gen.rs`・`context.rs`・`transpiler.rs`。2026-10-23）
  - `ponytail-core-registry`：検索を触る `call-attribute-filter`・`scene-exact-match`、`scene-attribute-store`（`scene_registry.rs`。2026-10-23）
  - `ponytail-editor-tooling`：文法を足す spec の LSP・VSCode の着色
  - `ponytail-lua-scripts`：`virtual_dispatcher.lua` を触る `novel-talk-flow`、`scene.lua` を触る `scene-attribute-store`
  - `ponytail-lua-loader`：`runtime/finalize.rs` を触る `scene-attribute-store`（2026-10-23）
  - `ponytail-lua-debug`・`ponytail-shiori`・`ponytail-check-release`・`ponytail-manual-ci`：0.5.0 とはほぼ重ならない見込み（2026-10-23 に、`ponytail-lua-debug`・`ponytail-check-release`・`ponytail-manual-ci` は重ならないと測った）。
  - `ponytail-shiori`：0.4.2 の `shiori4-v3-portable` と `windows.rs`・`actor/` を共有する。あちらが先に入り、入った後の main でギャップ分析をやり直す。0.5.0 の `shiori4-v4-host` とも `actor/` を共有する（2026-10-23）。
  - `ponytail-manual-ci`・`ponytail-check-release`：版の決まっていない `pasta-dll-multiarch` と、`release.yml`・`build.yml`・見本ゴーストのテストを共有する（2026-10-23）。

| spec | 中身（Must fix は太字） | 持つ所 | 規模 | 要件定義 |
| ---- | ---- | ---- | ---- | -------- |
| ponytail-dsl-parser | **解析が辞書の大きさの 2 乗に遅い**・大きな数の字句が化ける・深い括弧でスタックがあふれる・作られない誤りの型 | `pasta_dsl/src/parser/**`・`error.rs`・`lib.rs` | M | Opus |
| ponytail-core-registry | シーン表・単語表の死んだ API・`fast_radix_trie` を `BTreeMap` に・作られない誤りの型 | `pasta_core/**`・`pasta_lua/src/search/context.rs`・`pasta_check/src/validate/scenes.rs` | S-M | Opus |
| ponytail-lua-loader | **読めないセーブを空で上書きして失う**・キャッシュの日付・同じモジュール名が黙って消える・finalize 前の検索・パスの `;`・使われない抽象 | `pasta_lua/src/{loader,runtime,presentation,encoding}/**` | M | Opus |
| ponytail-lua-transpiler | Lua ブロックの中身に字下げが入る・効かない設定・死んだ状態 | `pasta_lua/src/{code_gen/**,normalize,config,context,transpiler}.rs` | S-M | Opus |
| ponytail-lua-debug | **Lua 表示で `.pasta` のブレークポイントが死ぬ**・**ステップが抜けない**・数の表示・DAP の無応答・行ごとの費用 | `pasta_lua/src/debug/**` | L | Opus |
| ponytail-lua-scripts | **`hour_margin` が文字列だと毎秒エラー**・応答の見出しの改行・包む関数の写し・`co_callback`・続きのトークの期限 | `pasta_lua/pasta_scripts/**`・`tests/lua_specs/**` | M | Opus |
| ponytail-shiori | **unload と FreeLibrary の競り**・理由の無い 500・`Key:` の行が黙って 204・x64 の `long`・テストの競り・テストデータ | `pasta_shiori/**` | L | Opus |
| ponytail-check-release | **`release` が作者のフォルダを消す**・自分を入れ子に写す・.nar が自分を含む・効かない守り | `pasta_check/src/{release,copy,nar,update_files}.rs`・`pasta_sample_ghost/tests/**` | M | Opus |
| ponytail-editor-tooling | **1 つの誤りでシーン全体に波線**・箱・行末コメント・Lua のフェンス・tower-lsp の撤去・`host` | `pasta_dsl/src/partial.rs`・`pasta_lsp/**`・`editors/vscode/**`・`pasta_check/src/validate/{syntax,report}.rs` | L | Opus |
| ponytail-manual-ci | マニュアルの検査が Rust の PR とリリースで走らない・Pages の権限・action の SHA 固定・重複したビルド | `.github/workflows/**`・`book/tools/**` | M | Opus |

- **進め方**（2026-10-19 開発者の決定）
  - 各 brief の「Current State」は、2026-10-19 の監査の控え（出発点）である。
  - ギャップ分析で、その領域の `/ponytail:ponytail-audit` をやり直し、控えと突き合わせる。
  - 要件は「その領域の監査の指摘が、残らず直っている」ことにする。
  - 修正は、設計以降で行う。
- **決めたこと**（2026-10-19 開発者の決定）
  - **公開クレートの pub API は消してよい。** README と `tech.md` に「pasta の部品で、外部から使う安定した面ではない」と書く。10 本のうち先に入る 1 本が、1 回だけ書く。
  - **0.5.0 で使い手の決まっているものは消さない。**
    - シーンの属性の絞り込み（`call-attribute-filter` が使う）
    - `TranspileContext` のファイル属性（`scene-attribute-store` が使う）
    - 空の `pasta.areka`（`novel-areka-contract` の置き場）
  - **挙動を変える直しは、各 brief の Desired Outcome のとおりにする。**
    - 読めないセーブは `.broken` に退避する。
    - 知らない DAP 要求には「unsupported」で答える。
    - 大きな数は f64 にする。
    - `Key:` の行を受け付ける。
    - `launch.json` の `host` は廃止する。
    - 同じモジュール名はエラーにする。
  - **`audit-pasta-check` の Req 1.3/1.4 の、働かない守りは消す。**
- **境界の戦略**
  - クレートとディレクトリで切り、どの 2 本もソースのファイルを共有しない。
  - 共有するのは、次のものだけ。
    - `Cargo.lock` とルートの `Cargo.toml`：`ponytail-editor-tooling`（tower-lsp）と `ponytail-core-registry`（fast_radix_trie）。席は editor-tooling が持ち、後から入る側が作り直す。
    - `crates/pasta_lua/src/lib.rs` の再輸出：loader と transpiler。
    - マニュアルの同じ頁の別の節：`internals/shiori.md`・`transpiler.md`・`execution-model.md`・`debug/*.md`。
    - `tech.md`・`structure.md` の別の行。
    - `gen-skill-refs` が作る references。
  - これらは、後から入る側が rebase で合わせる。
- **ほかの spec との関係**
  - 0.5.0 の先頭のうち、`novel-areka-contract` とはファイルが重ならない。`scene-attribute-store` は、`ponytail-dsl-parser`・`ponytail-lua-transpiler`・`ponytail-core-registry`・`ponytail-lua-loader`・`ponytail-lua-scripts` とファイルを共有する（2026-10-23 の再測定。前の「重ならない」は誤り）。`novel-variable-scope` は測り直していない。
  - `review-improvement-loop`（常駐）が次に回るときは、この 10 本が入った後のコードを基準にする。
- **人手の確認**
  - `ponytail-check-release` は、`.claude/skills/pasta-check/SKILL.md` を直す。分類器が止めるので、その場面で開発者の許可を求める。
  - `ponytail-shiori` の unload の競りと `Key:` の行は、SSP の実機で確かめられれば確かめる。

開始のコマンドの形（今はウェーブに入れない。0.4.2 を出した後、0.4.3 のウェーブで、上の「0.4.3 の範囲」の順とソースの重なりで選んだ分だけを並べる）:

```
Wave 10本
Opus
/kiro-start ponytail-dsl-parser
/kiro-start ponytail-core-registry
/kiro-start ponytail-lua-loader
/kiro-start ponytail-lua-transpiler
/kiro-start ponytail-lua-debug
/kiro-start ponytail-lua-scripts
/kiro-start ponytail-shiori
/kiro-start ponytail-check-release
/kiro-start ponytail-editor-tooling
/kiro-start ponytail-manual-ci
```

## 棚卸（2026-10-23・main `82176f19`）

前回（2026-10-23・main `8fc9bdff`）の後に、0.4.2 の 4 本（`external-control-events`・`reg-handler-chain`・`chain-talk-interrupt`・`talk-grapheme-clusters`）と、推したウェーブを別のセッションで一括して始めるスキル `/kiro-wave` が入った。起票は shiori4 の対応の 3 本（`shiori4-v3-portable`・`shiori4-v4-host`・`pasta-dll-multiarch`。PR#178）。この棚卸で `shiori4-v4-host` を 2 本に分けた。未完了は 29 本（常駐の `review-improvement-loop` を除く）、完了は 183 本。作業中のブランチは無い（ほかのワークツリーとブランチは、どれも main に入った後のもの）。前回の記録は git の履歴にある。

今のゴールは 0.4.2 である。棚卸の後の開発者の決定（2026-10-23）で、範囲の残りだった `shiori4-v3-portable` を 0.4.3 へ移し（まだ実施できない）、同じ日の discovery で起票した `pasta-skills-release-asset` を 0.4.2 に入れた。0.4.2 の残りは、この 1 本とリリースである。`recommend-menu-scenes` と ponytail の 10 本は 0.4.3 に入れた。測り直したのは 11 本で、`shiori4-v3-portable` と、版の決まっていない `recommend-menu-scenes`、新しく起票した 2 本（`shiori4-v4-host`・`pasta-dll-multiarch`）、入った 4 本がファイルや前提を動かした 7 本（`ponytail-shiori`・`ponytail-lua-scripts`・`shell-element-click`・`novel-talk-flow`・`cue-command-runtime`・`novel-checkpoint-rollback`・`scene-anchor-link`）である。

- `shiori4-v3-portable` は 14〜18 タスクのまま。shiori4 の C ABI のクレート `shiori4_abi` は crates.io に 0.0.1 が出ていて（依存なし・MIT）、SHIORI3 互換モードの書き出しに足りる。アクターの中で Win32 のメッセージを使う所は無く、待つ所は `actor/thread.rs` の 1 か所だけなので、メッセージ専用の窓（`wintf`）をやめて、どの OS でも普通の受信の繰り返しにするのを推奨する。インスタンスごとに持つ状態の一覧に、キックの受け口が読む `MAILBOX`（`actor/lifecycle.rs`）と、2 つ目の `load` でログの設定が前のインスタンスに漏れる件（ログの絞り込みの持ち手が 1 度しか作られない）が抜けていた。brief に足した。`recommend-menu-scenes` とはソースを共有しない（設定の読み込み `loader/config/` に触らない約束を brief に書いた）。
- `recommend-menu-scenes` は、先に要る 3 本が全部入り、先頭になった（10〜12 タスク）。`REG["sakura.recommendsites"]` は、`REG` のメタテーブルの後も今どおり勝つ。ただし、リソースの問い合わせのハンドラで失敗の表記（`FAILURE.report`）を出すと、表記がリソースの値に混ざる。リソースの失敗はログだけにするのを推奨する。待っているチェイントークがあるとき、選んだ後のシーンの台詞は割り込みになり、`＊チェイントーク中断` に置き換わる。選んだ後のハンドラに `STORE.co_preempt` の印を付けない（付けると黙って捨てる）。
- `shiori4-v4-host` は 22〜28 タスクと測れたので、Rust の土台（この spec）と、Lua と DSL の口（新しい `shiori4-v4-lua-api`）に分けた（下）。
- `pasta-dll-multiarch` は 9〜12 タスク。x64 版を出す前に、SHIORI の入口の型（`windows.rs` の `usize` を C の `long` に、`bool` を `BOOL` に）を直しておく必要がある。`shiori4-v3-portable` か `ponytail-shiori` が直す。aarch64 で LuaJIT がビルドできるかは、まだ確かめていない。
- `ponytail-shiori` は、`shiori4-v3-portable` と `windows.rs`・`actor/` を共有する。あちらの後に入り、入った後の main でギャップ分析をやり直す。`BOOL` の分は `shiori4-v3-portable` に移る見込み。
- チェイントークの新しい決まり（台詞を出した割り込みは続きを捨てて `＊チェイントーク中断` に置き換える）が、0.5.0 の 4 本に議題を足した。`shell-element-click`（エレメントのクリック）と `scene-anchor-link`（アンカーのクリック）は、選択肢のような割り込みにするか、指名の再生のように `STORE.co_preempt` で捨てるかを要件で決める。どちらも、`STORE.last_global_scene` は割り込みの後に中断のシーンの名前になるので、画面やリンクを出したシーンの覚えには使えない。`cue-command-runtime` は、ト書きの命令を台詞と数えない（`BUILDER.speaks`）のを推奨する。`novel-checkpoint-rollback` は、`shiori4-v4-host` が応答の外で台本を送る道（SEND）を足すので、応答の出口だけで控えると取りこぼす。
- `talk-grapheme-clusters` の後、`novel-talk-flow` が `sakura_script/` で直すのは、`\x` で列を動かす所（`line_breaker.rs` の `column_move`）だけになった。`line_breaker.rs` は 1,155 行だが、本体は 310 行までで、残りはファイルの中のテストである。

### 即時修正（spec なし・本棚卸で実施して閉じた）

- `steering/product.md` — 完了の数（183）。現在地に、0.4.2 のはじめの範囲の 7 本と書記素クラスタの直しが入り、残りが `shiori4-v3-portable` とリリースになったことを書いた。
- この roadmap — 完了した 4 本を「完了フェーズ」の 0.4.2 の行へ畳み、台帳の `[x]` の行を消した。前提に、外からのイベント・`REG:register`・チェイントークの続きの決まり・台詞の 1 文字の 4 行を足した。0.4.2 の表を今の状態にし、`talk-grapheme-clusters` の行を足した。`recommend-menu-scenes`・`ponytail-lua-scripts` の Dependencies から完了した spec を外した（none）。`talk-grapheme-clusters` と同じウェーブに置かない、という、もう要らない約束を消した。`sakura_builder.lua` の行数を 558 に直した。0.4.1 の後に届いた不具合の節の「版は開発者が決める」「まだ `Cargo.lock` に無い」を、決まった形に直した。隙間の spec の節に、`ponytail-shiori` と `shiori4-v3-portable`、`ponytail-manual-ci`・`ponytail-check-release` と `pasta-dll-multiarch` の重なりを足した。
- 完了した spec の調査メモ 2 つ（`completed/chain-talk-interrupt/research.md`・`completed/ontalk-suppression/research.md`）の、`external-control-events` の brief へのパスを `completed/` に直した。
- `ponytail-lua-scripts` の brief に、入った 2 本からの申し送りを 2 行足した（`scene_play.lua` と `external_control.lua` に共通する部品の取り出し、`EVENT.drive` の `_interrupted` の枝が `STORE.co_callback` を戻さない件）。
- 確かめて閉じたもの: マニュアルの章の数は 63 のままで、3 つの検査と合っている。台帳の 29 行と brief のフォルダ 29（常駐を除く）は一致する。完了した spec を指す古いパスは、上の 2 つのほかに無い。`tech.md` は `unicode-segmentation` をもう載せている。

### 統合・分割・改名

- `shiori4-v4-host` を分けた（はじめて分ける。22〜28 タスク）。
  - `shiori4-v4-host`（Rust の土台。11〜13 タスク。Opus）: ホストのテーブルを受ける、アクターのスレッドからの `host->request` を SHIORI のスレッドへ中継する、post と `Tag` の Rust の側、unload での無効化、ホストの `log` への転送、キックの即時化、GET の待ち（5 秒）に往復の分を見込む。
  - `shiori4-v4-lua-api`（新しい。Lua と DSL の口。11〜13 タスク。Fable）: ホストのプロパティの読み出し、シーンからの SEND・NOTIFY、終わりを待って `OnHostComplete` で待っているシーンを再開する書き方、SHIORI3 だけのときの扱い、`pasta_check` の組み込みの名前、マニュアル。`shiori4-v4-host` の後に入る。
- ほかは分けない（20 タスクを超えるものは無い。`cue-command-runtime`・`novel-talk-flow` は前回の分け方の案のまま）。

### 起票と、バックログに残したもの

- 起票: なし（分割の 1 本だけ）。
- バックログに 3 行足した: 外から変数の値を読み出すイベント、捨てたチェイントークの続きの再開と種類ごとの中断の台詞、書記素クラスタの残り（どれも入った spec の範囲の外として残したもの）。
- ほかは全部そのまま（どれも、動機の事例・使う人・計測が無い）。既存の spec の要件で拾えるもの（既定の別名の追加 → `recommend-menu-scenes`、式の木の優先順位 → `ponytail-dsl-parser`、デバッグのポートの奪取と時々落ちる TCP のテスト → `ponytail-lua-debug`、budoux のキーの検査 → `recommend-menu-scenes` か小さな起票）はあるが、動機が無いので移していない。
- 直接実装の候補の「Reference の自動の転記」は、`call-attribute-filter` の要件を待つ（前回のまま）。

### 要件定義のモデル

今のゴール（0.4.2）の残りの `shiori4-v3-portable` は Opus。いちばん Fable に近いのは、同じプロセスの 2 つ目の `load` を受けるかだが、受ける・断るの 2 択で、推奨（受ける）が開発者の決定（状態をインスタンスごとに持つ）と 3 者会談の「できれば受ける」から立つ。インスタンスの状態はエンジンの中の作りで、DSL・保存の形式・公開の Lua の取り決めではない。ABI は shiori4 の正本が決める。ゴールの後の Fable は、前回の 6 本（`scene-attribute-store`・`novel-variable-scope`・`cue-command-runtime`・`call-attribute-filter`・`novel-talk-flow`（分けるなら出力の分だけ）・`novel-checkpoint-rollback`）に、分けた `shiori4-v4-lua-api`（後の spec が上に乗る Lua と DSL の取り決め＋ Rust・Lua・SHIORI をまたぐ待ちと再開の時の流れ）を足した 7 本。

### ウェーブ（2026-10-23）

今のゴール（0.4.2）の中で始められるのは、棚卸の後の discovery で起票して 0.4.2 に入れた `pasta-skills-release-asset` の 1 本だけである（2026-10-23 開発者の決定）。0.4.2 から移した `shiori4-v3-portable` と、`recommend-menu-scenes`・ponytail の 10 本は 0.4.3 なので、ゴールの外として入れていない。

| spec | 子孫 | 種別 | 規模 | 要件定義 |
| ---- | ---- | ---- | ---- | -------- |
| pasta-skills-release-asset | 約 1（`ponytail-manual-ci` が後から合わせる） | 機能（開発者の依頼） | 6〜8 | Opus |

開始コマンド（`/kiro-next` の報告と同じ形。俯瞰の頁が、この塊をコピーのボタンつきで出す。Fable は無し）:

```
Wave 1本
Opus
/kiro-start pasta-skills-release-asset
```

`pasta-skills-release-asset` の持ち場: `crates/pasta_sample_ghost/release.ps1`（新しい段）、`.github/workflows/release.yml`（配布物の検査と upload）、`.github/scripts/release/github-release.ps1`・`release-scripts-test.ps1`、`.claude/skills/release/`（添付の数え方）、マニュアルの `introduction.md`、`crates/pasta_sample_ghost/RELEASE.md`・`README.md`、`tech.md`・`structure.md`。リリースの席はこの spec が持つ。

次（今のゴールの中）: 0.4.2 のリリース（`/release 0.4.2`）。

ゴールの後（0.4.3 の順）: `ponytail-lua-loader`・`ponytail-check-release`（データが失われる Must fix。先頭）、`recommend-menu-scenes`（先頭。`ponytail-lua-loader`・`ponytail-lua-scripts`・`ponytail-core-registry` と同じウェーブに置かない）、ほかの ponytail（先頭。`ponytail-manual-ci` は `pasta-skills-release-asset` の後）、`shiori4-v3-portable`（まだ実施できない。最後）、`ponytail-shiori`（`shiori4-v3-portable` の後）。その後（0.5.0 の順）: `novel-areka-contract`（先頭・子孫 約 6）、`scene-attribute-store`（先頭・子孫 約 9）、`novel-variable-scope`（先頭・子孫 約 4）、`shiori4-v4-host`（`shiori4-v3-portable` の後）。版の決まっていないもの: `scene-exact-match`（先頭・推奨 0.5.0）、`pasta-dll-multiarch`（`shiori4-v3-portable` の後）、`scene-pick-modes`（`scene-attribute-store` の後）、`scene-anchor-link`（`call-attribute-filter` の後）。

### 開発者の判断が要るもの（ウェーブは止めない）

- **`pasta-dll-multiarch` の版**。`shiori4-v3-portable` の後に入る。推奨は 0.5.0（0.4.3 は `shiori4-v3-portable` で締め、aarch64 の LuaJIT の確かめを 0.4.3 の出る時期に乗せない）。
- **`scene-exact-match`・`scene-pick-modes` の版**。推奨は 0.5.0（前回と同じ）。
- **0.5.0 を今のゴールに含めるか**（0.4.2・0.4.3 と並べるか）。今の 0.4.2 の残りは 1 本なので、含めれば、次のウェーブに `novel-areka-contract`（Opus・文書だけ）と `scene-attribute-store`（Fable）を置ける（どちらも `pasta-skills-release-asset` とソースを共有しない）。0.4.3 の ponytail の 5 本とは `scene-attribute-store` がファイルを共有するので、0.4.3 と並べるときは組み方に注意する。
- **`scene-anchor-link` の版**。推奨は 0.5.0 の後（前回と同じ）。
- `SOUL.md` §2・§5 の「Wait/Sync はマーカーのみ」「全出力は yield」— 構想の文書なので、直すかどうかは開発者が決める。
- `steering/structure.md` の作り直し（`pasta_shiori` の木と、`pasta_scripts/pasta/` の木が無い。`event/external_control.lua` と新しい結合テストも載っていない。テストの数も古い）— `/kiro-steering` で直す。数だけ直すと下のファイルの一覧と合わなくなるので、即時修正にしなかった。
- ghost_dev の emo2 開発のフォルダに、入門の撮影のときの `profile/` の保存データとログが残っている。リポジトリの外なので、片付けるかは開発者が決める。

### 人手の確認が残っている項目

- リリースの失敗した job の再実行の読み方は、まだ確かめていない（下の「リリース」）。
- 0.4.2 のリリースで、GitHub Release に `pasta_check.zip` と `pasta-skills.zip` が載って添付が 6 つになること、固定のリンク（`releases/latest/download/pasta_check.zip`・`pasta-skills.zip`）で落ちることを確かめる（`pasta-check-release-asset`・`pasta-skills-release-asset`）。
- 手元の `release.bat`（Windows PowerShell 5.1）で、`pasta_check.zip` の段を含めて通しで走ることを確かめる（`pasta-check-release-asset`）。

## Phase 11 の残り: 属性セマンティクスとシーンリンク

### 境界戦略

- **分割理由**: シーン属性を 2 つに分けた（2026-10-04）。保持・継承・読み出し（`scene-attribute-store`）と、Call の属性フィルター（`call-attribute-filter`）。どちらも 0.5.0 の道筋に入った（下の「Phase 13」）。実行時の失敗の出力の 2 本（`failure-output-unification`・`runtime-error-balloon`）は、2026-10-17 までに完了した。
- **共有接点**: `crates/pasta_lua/pasta_scripts/pasta/act.lua` は 1 ウェーブに 1 spec だけが持つ。`call-attribute-filter`・`scene-anchor-link` が触る。`crates/pasta_lua/src/code_gen/element_gen.rs` も 1 ウェーブに 1 spec だけで、`call-attribute-filter` と `scene-anchor-link` が触る。`scene-attribute-store` と `call-attribute-filter` は、`scene.lua` と `pasta_core` のシーン登録（`scene_registry.rs`・`scene_types.rs`）を共有するので、この順に置く。選択肢の出力（`pasta_scripts/pasta/shiori/sakura_builder.lua`）と選択肢の振り分け（`shiori/event/choice_select.lua`）は、`scene-anchor-link` と Phase 13 の spec が共有する。辞書の検査に検査を足す spec は、`call-attribute-filter`・`scene-anchor-link` が飛び先の検査（`validate/references.rs`）を、`cue-command-runtime`・`novel-talk-flow` が設定の検査（`validate/config.rs`）を共有する。
- **シーン名の別名との関係**: 別名の解決は検索の入口（`search/context.rs`）でキー正規化の前に行う。`scene-attribute-store`・`call-attribute-filter` は、別名を当てた後の名前で属性を扱う。
- **結合テスト**: `pasta_shiori` の結合テストは、自己展開した本物のランタイムで動き、回避を書かない（`shiori-test-support-runtime`）。

### ウェーブ構成

| Wave | spec（並走可） | 種別 | ソースの持ち場 |
| ---- | -------------- | ---- | -------------- |
| 0.5.0 の先頭 | scene-attribute-store | 機能 | 宣言行の属性のパース（`parse_scene.rs`・`ast/scene.rs`）、`scope_gen.rs`・`context.rs`・`transpiler.rs`、`scene.lua`、`finalize.rs`、`pasta_core` のシーン登録（`scene_registry.rs`・`scene_types.rs`）、見本の期待値ファイル、マニュアル（`block-structure.md`・`script-api.md`・`registry-search.md`・`transpiler.md`） |
| 0.5.0（`scene-attribute-store` の後） | call-attribute-filter | 機能 | Call の文法（`grammar.pest`・`parse_action.rs`・`ast/action.rs`・`partial.rs`）、`element_gen.rs`（Call）、`act.lua`（`call`・`find_handler`）、`scene.lua`（`SCENE.search`・`co_exec`）、`search/`、`pasta_core` の `scene_table.rs`・`scene_types.rs`、LSP（`visit_action.rs`）、VSCode の文法定義、`pasta_check validate`（候補が複数残る検査を足すなら） |
| 版は未定 | scene-anchor-link | 機能 | `call-attribute-filter` と同じ文法・生成・`act.lua`・エディタのファイルに加え、`sakura_builder.lua`、`shiori/event/choice_select.lua`、`sakura_script/`（表示名を Lua の側で台詞として通してから `\_a` で包む案なら触らない）、VSCode 拡張の `wordRefDecorator.ts`、`pasta_check validate`（`references.rs`） |

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
  - 出力の組み立て（`pasta_scripts/pasta/shiori/sakura_builder.lua`）は、`cue-command-runtime`・`novel-talk-flow`・`novel-checkpoint-rollback` が触る。`scene-anchor-link` も触る。558 行あり（2026-10-23）、3 本が足すと 740〜870 行になる。`BUILDER.build` は込み入っているので、クリック待ちや `\x` の処理は、`actor_style.lua` のように別のモジュールに分ける。
  - 台詞の切り出し・自動ウェイト・budoux（`sakura_script/` の `tokenizer.rs`・`wait_inserter.rs`・`line_breaker.rs`）は、`novel-talk-flow` が触る（2026-10-23 の再測定では、`\x` で列を動かす `line_breaker.rs` の `column_move` だけ）。`scene-anchor-link` も、Rust の側を触るなら加わる。`line_breaker.rs` は 1,155 行だが、本体は 310 行までで、残りはファイルの中のテストである。テストを足す spec は `tests/sakura_script/` に置く。
  - 選択肢の振り分け（`shiori/event/choice_select.lua`）は、`shell-element-click` と `scene-anchor-link` が共有する。先に入る側が、振り分けを関数として取り出す。
  - 場面の関数を出すコード生成（`code_gen/scope_gen.rs`）は、`scene-attribute-store`・`cue-command-runtime`（2026-10-17 に `scene-stage-attributes` をまとめた）が触る。この順に置く。場面の関数の頭は `act:init_scene(SCENE, 書いたグローバル名, 書いたローカル名, 行の表)` で、行の表は最後の引数である（`runtime-error-balloon` が入れた。`transpiler.rs` の `fill_line_maps` が後から差し込む）。引数を足す spec は、この形を保つ。
  - `act.lua` は 837 行あり、1,000 行に近い。`cue-command-runtime`・`call-attribute-filter`・`scene-anchor-link`・`novel-talk-flow` が足すので、足す側が分け方を考える。ト書きの新しい入口は、別のファイルに置く。
  - 変数の書き方（`grammar.pest`・`code_gen/element_gen.rs` の `save.x`）は、`novel-variable-scope` が触る。文法を触る `call-attribute-filter`・`scene-anchor-link`・`cue-command-runtime` と同じウェーブに置かない。
  - 設定の読み込み（`loader/config/`）は、`cue-command-runtime`（別名表）と `novel-talk-flow`（ノベル向けの切り替え）が触る。
- **検証で分かった制約**（2026-10-10。詳しくは各 brief の「検証で見つかった穴」）:
  - NOTIFY のイベントには台本を返せない。ノベル用のイベントは、すべて GET で送ってもらう。GET の応答は 5 秒以内に返す。
  - 場面の検索は前方一致である（`＞第1章` は `第10章` にも当たる）。ノベル向けの設定では完全一致にする（`novel-talk-flow`）。
  - 出力の組み立ては `\x`（クリック待ち）をほとんど知らない（装いの出し入れ `actor_style.lua` だけは知っている。2026-10-18）。`\x` の後で、段落の区切り・列の持ち越し・サーフェスの復旧・budoux の数え方がずれる（`novel-talk-flow`）。
  - 時刻の問い合わせ（`clock.lua` の `\![get,property,OnPastaClock,…]`）は、暇な `OnSecondChange` の応答に載る。ノベルの吹き出しを読む間にも返りうるので、巻き戻しの区切りにしない（`novel-checkpoint-rollback`。2026-10-18）。
  - 走りかけのコルーチンは控えられない。区切りにできるのは、場面が最後まで走った応答だけである（`novel-checkpoint-rollback`）。
  - 宣言行に書いた属性（`＊名前＆k：v`）は、パーサが読み飛ばしている（`scene-attribute-store` が直す）。

### 順番

| 順 | spec | 要件定義 | 規模 | 先に要るもの |
| -- | ---- | -------- | ---- | ------------ |
| 1 | novel-areka-contract | Opus | 6〜9（文書だけ） | なし。**0.5.0 のゴールの先頭**（areka の側が早く動けるよう、最初に始める） |
| 2 | scene-attribute-store（既存） | Fable | 15〜18 | なし（0.5.0 の先頭） |
| 3 | novel-variable-scope（2026-10-17 に `novel-checkpoint-rollback` から分けた） | Fable | 8〜10 | なし（文法と `element_gen.rs` を触るので、`call-attribute-filter`・`cue-command-runtime` と同じウェーブに置かない） |
| 4 | cue-command-runtime（ト書きと柱。2026-10-17 に `scene-stage-attributes` をまとめた） | Fable | 17〜21（超えるならト書きと柱に分け直す） | `scene-attribute-store`（柱の分）。語彙の名前は `novel-areka-contract` から引く |
| 5 | shell-element-click | Opus | 5〜8 | `novel-areka-contract`（完全一致の場面選びは `touch-poke-shortcuts` が入れてある）。`scene-anchor-link` とは `choice_select.lua` を共有する |
| 6 | novel-talk-flow | Fable | 17〜21（超えるなら出力の分（Fable）と場面の流れの分（Opus）に分ける） | `cue-command-runtime`、`novel-areka-contract` |
| 7 | call-attribute-filter（既存） | Fable | 19〜21 | `scene-attribute-store` |
| 8 | novel-checkpoint-rollback | Fable | 13〜15 | `novel-talk-flow`、`novel-variable-scope` |
| 9 | novel-save-slots | Opus | 8〜12 | `novel-checkpoint-rollback` |
| 10 | hello-novel-sample | Opus | 15〜18 | 2〜9 の全部と、areka の側の実装 |
| 11 | manual-novel-guide | Opus | 8〜12 | `hello-novel-sample` |
| 12 | shiori4-v4-host（2026-10-23 起票。同じ日の棚卸で Rust の土台だけにした） | Opus | 11〜13 | `shiori4-v3-portable`、shiori4 の `abi-conventions`・`host-protocol` が crates.io に出ること。ノベルの合格には要らないので、ほかの 0.5.0 の spec を待たせない |
| 13 | shiori4-v4-lua-api（2026-10-23 の棚卸で `shiori4-v4-host` から分けた） | Fable | 11〜13 | `shiori4-v4-host` |
| 14 | 0.5.0 のリリース（`release` スキル） | Opus | — | 上の全部 |

並べられる組み合わせ（2026-10-17 と 2026-10-18 の棚卸で測った）: `novel-areka-contract`・`scene-attribute-store`・`novel-variable-scope` は、0.4.2 の残りの `shiori4-v3-portable` とソースを共有しない見込み（2026-10-23）。`novel-areka-contract` と `scene-attribute-store` は、互いにも共有しない。`shell-element-click` は `novel-areka-contract` の後、`scene-attribute-store` と並べられる（`scene.lua` を触らない約束で）。`novel-variable-scope` は `scene-attribute-store` と並べられる（`scene-attribute-store` が `partial.rs` と VSCode の文法定義を触らない約束で）。`cue-command-runtime` → `novel-talk-flow` → `novel-checkpoint-rollback` は、`act.lua` と `sakura_builder.lua` を共有するので 1 本ずつ入れる。

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

## リリース（`release` スキル）

リリースの CI 化（`release-ci`）、リリース手順（spec `release-workflow`）の書き換え、CI での初回のリリース（v0.3.8）とその後片付けは、2026-10-10 までにすべて済んだ。タグ `vX.Y.Z` の push だけで公開まで進む。記録は `.kiro/specs/completed/release-workflow/first-ci-release.md` と、同じフォルダの `design.md` にある。手順は 2026-10-11 に `release` スキル（`.claude/skills/release/`）へ移し、spec は完了にした。手順を直すときは、スキルを直す。**残るのは、失敗した job の再実行の読み方の確認である**（v0.3.8 も v0.4.0（2026-10-17、run 39024925990）も v0.4.1（2026-10-18、run 39123004799）も失敗が起きなかったので未確認。失敗が起きたリリースで確かめる）。

- **以後のリリース**: 新しい作業ブランチ（ハーネスのワークツリー）で `/release` を実行する（版を指定するときは `/release 0.4.0`）。失敗した job の再実行の読み方（スキルの `SKILL.md`「まだ実地で確かめていない読み方」）を確かめるまでは Opus で実行し、確かめたら Sonnet で実行する（開発者決定 2026-10-10）。
- **席**: リリースの実行中は、`release.ps1`・`release.yml` を触る spec を main へ入れない。
- **リリースの本文の作り直し**（2026-10-11 起票 `release-notes-highlights`）: 今の本文は、コミットの件名を全件並べた一覧で、何が良くなったのかを読み取れない（v0.3.8 は 57 行）。本文を、注目に値するいくつかの内容を Claudia（令嬢）の口調で紹介する文章にし、全件の一覧は Assets に添付する。spec 名は載せない。見どころは、版を上げるコミットでリポジトリのファイルとして書き、リリース CI が本文に使う（開発者の決定。公開の後で書き換える案と、機械だけで短くする案は採らない）。この spec は、`release` スキルの段 4 の「5 ファイルだけ」を変える。見出しには元になった PR の番号を付ける。公開済みの v0.3.8 の 1 本も、同じ形に直す（それより前は触らない）。areka のリポジトリの `areka-P0-release-notes-highlights` と、決定と議題をそろえてある。
- **運用の注意**（一回限りのセットアップで分かったこと）:
  - Azure のリソースを `az` で作る・変えるには MFA が要る。WAM（Windows のサインイン窓）でログインすると `RequestDisallowedByAzure` で弾かれる。`az config set core.enable_broker_on_windows=false` にしてからブラウザーで `az login` し直す。ワークフローのマネージド ID には関係ない。
  - Marketplace の Members の管理には、公式の CLI が無い（vsce・az・gh のどれも扱えない）。画面から行う。
  - setup-check の `verify-pat` が失敗すると、vsce のエラー文にマネージド ID の識別子が含まれ、ジョブのログに出る。秘密の値ではないので直さない（2026-10-10 決定）。

## Specs (dependency order)

- [x] pasta-skills-release-asset -- pasta のスキル 3 本（`pasta-ghost-authoring`・`pasta-lua-coding`・`pasta-check`）とリポジトリの `LICENSE` を `pasta-skills.zip` にして、5 つ目の配布物として GitHub Release に載せる（添付は 6 つ）。`release.ps1` の新しい段で作り、リリース CI が中身を確かめる。`github-release.ps1`・`release` スキルの添付の数え方を合わせる。マニュアルに受け取り方と置き場所を書く（2026-10-23 の discovery で起票。開発者の依頼）。0.4.2 に入れる（2026-10-23 開発者の決定）。`ponytail-manual-ci`・`pasta-dll-multiarch` と同じウェーブに置かない。Dependencies: none
- [ ] recommend-menu-scenes -- 右クリックメニューの「おすすめ」「ポータル」を辞書のシーンだけで書けるようにする。`＊おすすめメニュー`・`＊相方おすすめメニュー`・`＊ポータルメニュー`（別名の既定に足す。完全一致）の中で `＞おすすめ項目（…）`・`＞おすすめ区切り` を積み、リソースの問い合わせでシーンをその場で動かして値を返す（`STORE.co_scene` に触れない）。選んだ後は項目のシーンへ回す。ボタンの名前は決まった名前の単語。Lua の `REG` が勝つ。`[scene.alias]` をキーごとの置き換えに改める（2026-10-20 起票）。0.4.3 に入れる（2026-10-23 開発者の決定）。先に要る 3 本は 2026-10-23 に入った。`shiori4-v3-portable` とはソースを共有しない。`ponytail-lua-loader`・`ponytail-lua-scripts`・`ponytail-core-registry` と同じウェーブに置かない。Dependencies: none
- [ ] shiori4-v3-portable -- pasta.dll が shiori4 の `shiori_get_api(3)`（SHIORI3 互換モード。SHIORI/3.0 の全文を UTF-8 で運ぶ）を書き出し、Windows 以外（Linux・macOS）でもビルドとテストが通るようにする。アクターの実行器を OS に頼らない形にし、Windows だけの依存を `cfg(windows)` に閉じ込める。Windows の今の入口は残す。エンジンは状態をインスタンスごとに持つ（2026-10-23 開発者の決定）。入口で 2 つ目の `load` を受けるかは要件で決める。版 4 は範囲の外（2026-10-23 起票。3 者会談の後、開発者の方針で 2 段に分けた 1 段目）。0.4.2 に入れた（2026-10-23 開発者の意向）が、まだ実施できないので、0.4.3 へ移した（2026-10-23 開発者の決定）。0.4.3 の最後。`ponytail-shiori` と同じウェーブに置かない。Dependencies: none
- [ ] shiori4-v4-host -- 版 4 に本格対応する Rust の土台。ホストのテーブルを受け、アクタースレッドからの `host->request` を SHIORI のスレッドへ中継する。post（SSTP に揃えた `SEND`＋`Script`・`Option: nobreak`、`NOTIFY`）と `Tag` の Rust の側、unload での無効化、ホストの `log` への転送、キックの即時化（チェイントークの続きの決まりと自動トークの停止を保つ）、Lua への低い入口と「版 4 で読まれた」の印。取り決めの正本は shiori4 の `abi-conventions`・`host-protocol`（2026-10-23 起票。2 段の 2 段目。同じ日の棚卸で、Lua と DSL の口を `shiori4-v4-lua-api` へ分けた）。0.5.0 に入れる（2026-10-23 開発者の方針。ノベルゲームと同じリリース）。`ponytail-shiori` と同じウェーブに置かない。Dependencies: shiori4-v3-portable
- [ ] shiori4-v4-lua-api -- 版 4 のホストの機能を Lua と DSL から使う口。ホストのプロパティの読み出し（今の `act:get_property` と `CALLBACK.pending` の上に作る）、シーンからの SEND・NOTIFY、終わりを待って `OnHostComplete` で待っているシーンを再開する書き方（`REG:register` との並び・チェイントークの `STORE.co_preempt` との関係・待っている間の unload）、SHIORI3 だけのときの扱い、`pasta_check` の組み込みの名前、マニュアル（2026-10-23 の棚卸で `shiori4-v4-host` から分けた）。0.5.0。Dependencies: shiori4-v4-host
- [ ] pasta-dll-multiarch -- x64・arm64 版の pasta.dll をビルドし、32bit 版と同じリリースで出す。見本ゴーストの .nar に areka の探し方に合う名前（`pasta.x64.dll`・`pasta.arm64.dll`）で同梱し、接尾辞の無い `pasta.dll` は SSP 向けの 32bit 版のまま残す。`release.ps1`・`.cargo/config.toml`・配布物とリリース CI の検査・マニュアル。aarch64 の LuaJIT のビルドは要確認（2026-10-23 起票。3 者会談の後の areka の相談から）。版は未定。`ponytail-manual-ci`・`ponytail-check-release` と同じウェーブに置かない。Dependencies: shiori4-v3-portable
- [ ] novel-areka-contract -- pasta と areka の取り決め（台本に載せる語彙・目印・イベント・設定の値）を 1 つの文書にし、areka への依頼書として出す。コードは書かない。areka に依頼を出すための spec（2026-10-10 の discovery で起票）。Dependencies: none
- [ ] scene-attribute-store -- シーン属性の実行時の保持・Lua からの読み出し・ファイルレベル属性の継承と上書き・値の型解釈。Dependencies: none
- [ ] novel-variable-scope -- 変数の保存の範囲を、セーブごとと全セーブ共通の 2 つに分ける書き方。今の `＄＊x` と保存ファイルとの互換、保存できない値の扱い（2026-10-17 の棚卸で `novel-checkpoint-rollback` から分割）。Dependencies: none
- [ ] cue-command-runtime -- キューコマンド行 `！命令＠対象（引数）` を、実行時に `\![…]` として台本へ流す。日本語の名前の別名表、引数の穴（数字で始まる引数など）の手当て。柱に書いた `＆属性` とアクター指定行も、場面に入ったときに台本へ流す（受け継ぎと、Call から戻ったときの打ち直し）。2026-10-17 に `scene-stage-attributes` をまとめた。Dependencies: scene-attribute-store
- [ ] shell-element-click -- エレメントのクリック（名前つき）を、選択肢と同じ振り分けで場面へ届ける。シェルとして組んだシステム画面を、場面が動かせるようにする。Dependencies: novel-areka-contract（`scene-anchor-link` と `choice_select.lua` を触る順番に注意）
- [ ] call-attribute-filter -- Call の属性フィルター構文（`＞シーン＆k＝v`・比較演算子・複数条件）と実行時の絞り込み。Dependencies: scene-attribute-store
- [ ] scene-anchor-link -- 台詞の中の `＠？シーン名`（`「表示名」` も付けられる）を、さくらスクリプトのアンカー `\_a` として出す。クリックで、`OnAnchorSelectEx` からそのシーンへ飛ぶ。選択肢の振り分けを共有し、LSP・VSCode の着色とマニュアル、`pasta_check validate` の飛び先の検査まで揃える（2026-10-08 起票）。版は未定。Dependencies: call-attribute-filter
- [ ] novel-talk-flow -- ノベル向けの出力に切り替える設定。台詞ごとのクリック待ち、話者名と声と変わらない行 ID、ランダムトークと時報の停止、起動時の場面、「次の場面へ」、完全一致の場面選びの切り替え（2026-10-21 に仕組みを `scene-exact-match` へ移した。こちらは設定を入れるだけ）。Dependencies: novel-areka-contract, cue-command-runtime, scene-exact-match
- [ ] novel-checkpoint-rollback -- 台本に目印の ID を埋め、応答ごとに状態と台本を控える。areka が知らせた ID の時点へ戻す（変数の保存の範囲は 2026-10-17 に `novel-variable-scope` へ分けた）。Dependencies: novel-talk-flow, novel-variable-scope
- [ ] novel-save-slots -- スロットを選ぶセーブとロード。控えをスロットへ読み書きし、スロットの中身（日時・章の名前）を場面から読めるようにする。Dependencies: novel-checkpoint-rollback
- [ ] hello-novel-sample -- 見本のノベル。物語と、タイトル・セーブ・ロード・設定の場面一式とシェル。0.5.0 の合格を areka の実機で確かめる題材。Dependencies: cue-command-runtime, novel-talk-flow, novel-variable-scope, novel-checkpoint-rollback, novel-save-slots, shell-element-click, call-attribute-filter（areka の側の実装も要る）
- [ ] manual-novel-guide -- マニュアルに、ノベルゲームの作り方の部を足す。見本のノベルを題材に、柱・ト書き・台詞・分岐・システム画面・配布の順に導く。Dependencies: hello-novel-sample
- [ ] ponytail-dsl-parser -- 解析を辞書の大きさに比例させる（span の行と桁を pest の行の索引で取る）。大きな数の字句を、どこでも同じ規則（i64、だめなら f64）にする。深い括弧の入れ子を構文の誤りにする。作られない誤りの型と `PastaFile.span` を消す。0.4.3（2026-10-19 起票。ponytail 監査。2026-10-22 に隙間の spec へ改め、2026-10-23 に 0.4.3 へ戻した）。Dependencies: none
- [ ] ponytail-core-registry -- シーン表・単語表の死んだ API と誤りの型を消し、`fast_radix_trie` を `BTreeMap` に替える。属性の絞り込みは残す（0.5.0 が使う）。0.4.3。Dependencies: none
- [ ] ponytail-lua-loader -- 読めないセーブを `.broken` に退避する。キャッシュを日付の一致で判定する。同じモジュール名の辞書を読み込みのエラーにする。finalize 前の検索を一定にする。パスの `;`・`?` を断る。`presentation/`・`RendererInjection`・テストだけの構築の道を消す。0.4.3。Dependencies: none
- [ ] ponytail-lua-transpiler -- Lua ブロックの中身を書いたとおりに出す（字下げと空行）。効かない `LineEnding`・`comment_mode` と、死んだ文脈の状態を消す（ファイル属性は残す）。0.4.3。Dependencies: none
- [ ] ponytail-lua-debug -- Lua 表示でも `.pasta` のブレークポイントを効かせる。ステップがコルーチンや切断で抜けなくなるのを直す。数の表示と、知らない DAP 要求への応答（unsupported）を直す。行ごとの費用を下げ、死んだコードを消す。0.4.3。Dependencies: none
- [ ] ponytail-lua-scripts -- 数でない `hour_margin` を既定に戻す。応答の見出しの値の改行を空白にする。続きのトークの期限を呼んだ時刻から数える。包む関数の写しと `co_callback` を消す（`pasta.areka` は残す）。0.4.3。`set_co_scene` と `EVENT.fire` を先に直した 0.4.2 の 2 本は入った（2026-10-23）。Dependencies: none
- [ ] ponytail-shiori -- unload が、アクターのスレッドが DLL を出るまで（上限つきで）待つ。フォルダが無いときに理由を出す。`Key:` の行を受け付ける。FFI の型を C に合わせ、テストの競りを無くす。テストだけのコードとデータを出荷物から外す。0.4.3。Dependencies: none
- [ ] ponytail-check-release -- `release` の `--target`・`--release`・`--copy`・`--nar` の包含の関係を、ファイルに触れる前に断る。読み取りの誤りを、起きた段で報告する。働かない守りとテストの写しを消す。0.4.3。Dependencies: none
- [ ] ponytail-editor-tooling -- 構文の誤りを本当の位置に 1 件だけ出し、部分解析がシーンを落とさないようにする。箱・行末コメント・Lua のフェンスの色を文法に合わせる。`pasta_lsp` を WASM の解析器だけにする（tower-lsp を撤去）。`host` を廃止する。0.4.3。Dependencies: none
- [ ] ponytail-manual-ci -- マニュアルの 3 つの検査を、全部の push・PR とリリースの関門で走らせる。Pages の権限を deploy に絞る。公開の job の action を SHA で固定する。重複したビルド・PoC・写しを整理する。Node 22・mdBook 0.5.4 に揃える。0.4.3。`pasta-check-release-asset`（完了）が `release.yml`・`github-release.ps1` に足した段を保つ。Dependencies: none
- [ ] scene-exact-match -- 完全一致でシーンを呼ぶ `＞＞シーン名`（同名が複数ならランダム。既存の `search_scene_exact` を使う）と、`pasta.toml` の `[scene] match = "exact"`（`＞`・イベントの受け口・OnTalk・時報・選択肢の飛び先を全部完全一致にする。既定は前方一致）を足す。既定のモードでも、イベントの受け口はイベント ID の直後が英大文字で続く候補（`＊OnInstallCompleteAll` など）を外す。`novel-talk-flow` の完全一致の切り替えを引き取る（2026-10-21 起票。ghost_dev のヒアリング。同日に開発者が承認して `event-scene-exact-match` から広げた）。版は未定。`call-attribute-filter` と同じウェーブに置かない。Dependencies: none
- [ ] scene-pick-modes -- 同名シーンの選び方に、重み（`＊OnTalk　＆重み：5`。1 巡の中で重みの数だけ出る）と順番（`＆順番：番号`、別名 `index`。番号の小さい順に出して最初へ戻る。最後に出した番号を保存する）を足す。単語の値にも重み（`＠挨拶：こんにちは＠３、やあ`）を足す。不正な値は読み込みエラー。今は複写と Lua で作っている（2026-10-21 起票。ghost_dev のヒアリング。同日に開発者と書き方を決めた）。版は未定。`＆重み`・`＆順番` だけを先に特別扱いで読まない（属性の早い部分実装にしない）。`call-attribute-filter` と同じウェーブに置かない。Dependencies: scene-attribute-store

## Existing Spec Updates

2026-10-10 の discovery で、0.5.0 の道筋に乗せた既存の spec。各 brief の「申し送り（2026-10-10 の discovery・0.5.0 のノベルゲームより）」の節に書いた。

- [ ] scene-attribute-store -- 属性の読み手に、演出（`cue-command-runtime` の柱の分。2026-10-17 に `scene-stage-attributes` をまとめた）が加わる。順番を保てる倉の形、ローカルシーンと名前なしの `＊` への受け継ぎを、要件の材料にする。0.5.0 の先頭に置く。Dependencies: none
- [ ] call-attribute-filter -- フラグによる分岐の本命になる。条件の値に変数を書けること、「それ以外」の場面、数のしきい値、候補が複数残るときの扱いを、要件で決める。Dependencies: scene-attribute-store

2026-10-21 の discovery「PastaDSL機能ヒアリング」（ghost_dev の「emo2 開発」「ぱすた開発」セッションへの聞き取り）で、次の brief に「申し送り（2026-10-21 の discovery「PastaDSL機能ヒアリング」より）」の節を足した。

- [ ] call-attribute-filter -- イベントの Reference・保存の変数で振り分け、合わなければ 204 にする実例（2 つの値の等しさ・空・値の前方一致・かつ・数の範囲・一度だけ）。両ゴーストでいちばん数の多い要望。Dependencies: scene-attribute-store
- [ ] cue-command-runtime -- 外見として記録させない生のさくらスクリプト（今は `act:raw_script`）の実例。`！命令` の出力を外見に記録するかを要件で決める。Dependencies: scene-attribute-store
- [ ] recommend-menu-scenes -- `REG["sakura.recommendsites"]`・`REG["kero.recommendsites"]` を使っている実例。Dependencies: none
- [ ] scene-attribute-store -- 属性の新しい読み手 `scene-pick-modes`（重み・巡回）。Dependencies: none

## Direct Implementation Candidates

- [ ] イベントのシーンでの Reference の自動の転記 -- SHIORI イベントから動くシーンでは、`＞transfer_req_to_var` を書かなくても `＄ｒ０`〜`＄ｒ９` が入るようにする。今は転記のためだけに Lua を経由する場面がある（emo2 の `OnInstallRefuse`、2026-10-21 のヒアリング）。Lua の変更は数行だが、挙動が変わる。マニュアル `grammar/variables.md` に加え、入門の 7 段目（`getting-started/07-greeting.md` の「書き忘れると `＄ｒ０` は空のままになる」など）と段階辞書 `dic/07-greeting.pasta`・`STAGES.md` の扱いを決める必要がある（2026-10-22 棚卸）。`call-attribute-filter` の要件が Reference を直接書く形にするなら、そちらに含めて閉じる。

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
- デバッグポートの他プロセスによる奪取（即時修正で判明）— 相手が `SO_REUSEADDR` を立てて bind する場合まで防ぐには、Windows の `SO_EXCLUSIVEADDRUSE` が要る。ゴースト同士の二重 bind は防いだ。取れなかった側は、デバッグなしで起動を続ける（2026-10-19）。
- 式の木の優先順位（2026-10-07 棚卸、`dsl-codegen-runtime-safety` の調査）— `pasta_dsl` の `parse_action.rs` の `build_left_assoc_expr` は優先順位なしで木を作り、コード生成（`expr_gen.rs`）が組み直している。AST を直接使う別の利用者が出たら直す。
- 別ファイルの同名シーンのデバッガ索引（2026-10-07 棚卸、`scene-identity-format` の範囲外）— ファイルをまたぐ同名シーンの索引と通し番号は、既存の制約として残した。
- タグの読み取りの制約（2026-10-07 棚卸、`paragraph-break-tag-only-talk` の範囲外）— `\nHello` をタグとして読む、引数の中の `\]` を扱わない、BudouX の行幅にエスケープと囲みを数えない。囲み（`\_?…\_?`・`\_!…\_!`）とエスケープ `\\` は、`vertical-balloon-lines` から 1 つのタグとして扱い、列を動かさない（2026-10-18）。上の R4 と一部重なる。
- 閉じていない 1 行の引用（2026-10-07 棚卸、`dsl-literal-fixes` の範囲外）— `＠w：「abc` と、行をまたげる `sakura_body` の規則はそのままにした。
- 辞書の検査の広げ方（2026-10-17 棚卸、`pasta-check-dic-validate` の範囲外）— 単語参照の検査と、Lua の中身の検査。使う人の事例が出たら起票する。
- areka が撫で・つつきのイベント（`OnMouseMove`・`OnMouseDoubleClick` など）を SSP と同じ形で送るか（2026-10-17 棚卸、`touch-poke-shortcuts` の範囲外）— 確かめていない。0.5.0 で `novel-areka-contract` の依頼に要るなら、そこで確かめる。
- 止まるエラーの表記の粗さ（2026-10-17 棚卸、`runtime-error-balloon` の実装メモ「既知の粗さ」）— 同じ場面の中で `restore_scene` の後に止まると（アクターの単語の処理など）、`＊グローバル名` だけが出て、場所とローカル名が出ない。作者が `pcall` で失敗を飲み込み、同じ要求の中で止まると、古い場面の名前が出ることがある。困った事例が出たら起票する。
- SHIORI の応答に `ErrorLevel`・`ErrorDescription` を載せない件（2026-10-23 棚卸、`scene-play-event` の設計メモ）— pasta の警告が SSP の MCP の `get_log` に出ないので、AI エージェントは pasta のログファイルを読む。MCP からの調べ物で困った事例が出たら起票する。
- 外から変数の値を読み出すイベント（2026-10-23 棚卸、`external-control-events` の範囲の外）— MCP からの試験で値を確かめたい事例が出たら起票する。
- 捨てたチェイントークの続きを後で再開する仕組み（`＊チェイントーク再開` など）と、イベントの種類ごとの中断の台詞（2026-10-23 棚卸、`chain-talk-interrupt` の範囲の外）— 作者の事例が出たら起票する。
- 書記素クラスタの残り（2026-10-23 棚卸、`talk-grapheme-clusters` の範囲の外）— 正規化（NFC・NFKC）をしない。タグの直後の結合文字（`\n` の後の U+0301 など）は、それだけで 1 つのクラスタになる。`talk_to_script` の呼び出しをまたぐクラスタはつながない。困った事例が出たら起票する。
- 「話していない秒が N 秒続いたら」（Reference3 の静かな秒）のイベントの要望（2026-10-21 のヒアリング）— 窓際のぱすたさんは `pose.pasta` で自分で数えている。同じヒアリングで出た「`REG` に 1 つのイベントのハンドラを重ねる仕組み」は、2026-10-22 に `reg-handler-chain` として起票し、2026-10-23 に入った。2 つ目のゴーストで同じ要望が出たら起票する。
- 単語の値の中の単語参照（2026-10-21 のヒアリング）— `＠通常：「\s[1000]＠腕A」` の `＠腕A` は展開されず、文字のまま出る。表情を部品から組む要望の一部だが、表情の 5 軸のコードの表は Lua に残すと両ゴーストの作者が答えた。作者が DSL で書きたいと言い出したら起票する。
- 着せ替えの復旧の出力の膨らみ（2026-10-18、`dressup-restore-stale-parts` を取り下げたときの残り）。見た目は壊れない。2026-10-21 に作者の事例（同じ軸の OFF の列を毎回出している）から `dressup-category-exclusive` を起票したが、同日に SSP の実機で確かめて取り下げた（開発者の決定）。
  - 窓際のぱすたさんの複製（`mustselect` のカテゴリ）で、OFF の列を抜いても、話者が戻ったあとの見た目は腕も目も正しかった。最後に来るのはいつもいちばん新しく着けたパーツである。
  - 出力は膨らむ。話者が戻るたびに `\![bind-noevent,…]` が 14 個ほど並んだ（`うで` だけで 3 つ）。
  - 一瞬のちらつきの見込みがある。次の台詞の頭に表情の単語があると、復旧はその bind と同じパーツを出さない（`leading_explicit_binds` → skip）。そのため復旧の並びが古いパーツで終わり、直後に上書きされる。間にウェイトが無いので、dump には写らなかった。目で見えるという報告が出たら起票する。カテゴリを排他として記録すれば、ちらつきと膨らみが同時に直る。
  - `raw_script` で戻した表情は記録に残らない。そのため、前のトークのパーツが復旧で戻る（`cue-command-runtime` へ申し送った）。
  - `shiori/appearance.lua` は、アクターが着けたことのあるパーツの ON を、カテゴリごとに全部記録する。復旧のたびに、それを記録の順に出し直す（複数選択できないカテゴリでは、最後の 1 つが残る）。
  - 次の行の先頭の bind はパーツ単位で省くので、復旧の途中の見た目は古いパーツになる。間にウェイトは入らない。
  - 直すなら、着けたときに同じカテゴリのほかの ON を記録から外し、`multiple` のカテゴリだけを宣言で外す。作者が困った事例が出たら起票する。
- `pasta_check validate` が `[talk]`・`[ghost]` の budoux のキー（`budoux`・`spot_newlines`・`budoux_newlines`・`budoux_jump`・`after_choice_newlines`）を確かめない（2026-10-18 棚卸、`vertical-balloon-lines` の範囲外）。`[actor."名前"].font` は確かめる（`validate/config.rs`）ので、揃っていない。書き間違えた作者の事例が出たら起票する。
- 縦書きの列の扱いの残り（2026-10-18 棚卸、`vertical-balloon-lines` の範囲外）— 列の幅より長い 1 語は分けないので、SSP が自分で折り返した分を数えない。作者が書いたスコープの切り替え（`\0`・`\1`・`\p`・`\h`・`\u`）と `\b[…]` の後で数え直さない。`raw_script`・`act:set_property` は、ひとつだけの `\` の守りの外にある。
- スリープの印が残る件（2026-10-18 棚卸、`tutorial-ssp-gaps` の調査メモ）— `OnSysSuspend` が来て `OnSysResume` が来ず、PC が眠らなかったとき、印は次に立て直すまで残る。害が無いので受け入れた。
- バルーンが画面の左端で折り返るとき・縦に長いときの見え方（2026-10-17 棚卸、`getting-started-screenshots` の調査メモ。`hello-pasta-shell-art` からの持ち越し）— 実機で確かめていない。
- 並列負荷で時々落ちる TCP のテスト（2026-10-07 棚卸、`act-token-grouping-fix`・`scene-identity-format` の実装メモ）— `runtime_toggle_e2e_*`・`hook_panic_*`。
- マニュアルのメニューのテストが持つ mdBook の写し（2026-10-10 棚卸、`manual-claudia-theme` の実装メモ）— `book/tools/theme-menu-test.mjs` は、mdBook 0.5.4 の `book.js` のキー操作の処理を写して持つ。mdBook の版を上げるときに合わせる（CI は 0.5.3 に固定。隙間の spec の `ponytail-manual-ci` が 0.5.4 に揃える）。
- 配布版の hello-pasta に会話の変化を足すときの指針（2026-10-10 棚卸、`hello-pasta-tutorial-stages` の実装メモ）— emo2 開発からの知見「1 つの場面に 3 通り」は、段階の形を固定した辞書では満たせない。配布版に変化を足すことになったら、その動機から起票する。
- pasta.dll が C の実行時ライブラリの入出力の関数を約 170 個書き出している（2026-10-22 棚卸、`lua-cmodule-robustness` の調査の範囲外）— LuaJIT の `msvcbuild.bat` の組み立ての設定（`_CRT_STDIO_INLINE` を `dllexport` にする）から来る。pasta のコードではなく、LuaJIT の組み立ての側にある。害は見つかっていない。ほかの DLL とぶつかる事例が出たら起票する。
- budoux の自動改行の禁則（2026-10-05、ghost_dev「emo2 開発」からの申し送り）— `line_breaker.rs` の `break_lines_impl` は禁則を見ない。BudouX の語の区切りの直前で改行するだけである。
  - 方針は JIS X 4051 どおりとする。リーダー（`‥…`）は行頭に置いてよい。並びの途中では分けない。
  - この方針では、申し送りの実例「イイジャン！／‥‥ええと、」は正しい組版になる。
  - 句読点・`！？`・閉じ括弧の前、開き括弧の後ろでは、BudouX の既定の日本語モデルは区切らない（手元で確認）。
  - BudouX が行頭禁則の字の直前（または行末禁則の字の直後）で区切る実例が出たら、その実例から起票する。その場合は、既存の `[talk]` の `chars_*` を使った追い出しを第一候補とする。
