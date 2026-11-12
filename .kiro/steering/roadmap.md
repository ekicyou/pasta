# Roadmap

## 概要

pasta は、日本語 DSL（Pasta DSL）で書いた辞書を Lua へトランスパイルし、組込 LuaJIT で実行する「伺か」の SHIORI（`pasta.dll`）と、その周辺ツール（`pasta_check`・LSP・VSCode 拡張・利用者マニュアル）からなる。Phase 1〜11 で、プロパティアクセス・監査・マニュアル・ソースレベルデバッグ・アクターモデル駆動・配布物の形・現行実装の不具合の一掃までを完了した。2026-10-09 までに、リリースの CI 化、シーン名の別名、式の中の値なしの扱い、マニュアルの着せ替え、段階ごとに学べる hello-pasta の辞書も入った（下の「完了フェーズ」）。2026-10-11 までに、hello-pasta の新しいシェル、9 段目の選択肢の表示の直し、マニュアルの見出しへのリンクの検査も入った。2026-10-15 に、リリースの本文を見どころの紹介にした。2026-10-17 までに、撫で・つつきの反応のショートカット、実行時の失敗の出力の一本化、SSP を起動しない辞書の検査、シェルの説明の章、印刷用のページの廃止、結合テストの古いランタイムの写しの撤去も入った。同じ日に、止まる実行時エラーのバルーン表示と入門ガイドの絵 16 枚が入り、v0.4.0（主題「入門者にやさしく」）を出した。2026-10-18 に、辞書の検査の端のケース、入門ガイドと SSP のずれ（SSP の仮想の時刻を時報に効かせる）、窓際のぱすたさん（ghost_dev）からの要望の 2 本（アクターごとの表示の装い・縦書きのバルーンの列の扱い）が入り、v0.4.1 を出した。2026-10-19 までに、ゴーストから DLL の関数を LuaJIT の FFI で呼ぶ仕組み（`[loader] ffi`）、デバッグのポートが取れないときにデバッグなしで起動を続ける直し、pasta.dll のアイコンが入った。2026-10-23 までに、`pasta_check` を GitHub Release から単体で落とせるようにする直し（`pasta_check.zip`）、シーンからランダムトークと時報を止めたり再開したりする関数（`自動トーク停止`／`自動トーク再開`）、MCP から名前か位置で指名したシーンを再生するイベント（`OnPastaScene`・`OnPastaSceneAt`）が入った。同じ日に、外から `＞名前` の呼び出しとグローバル変数の代入ができるイベント（`OnPastaCallFunction`・`OnPastaSetSaveVariable`）、`REG:register` で 1 つのイベントにハンドラを重ねる仕組み、台詞を出さないイベントでチェイントークの続きが消えない直し、台詞の自動ウェイトと budoux の改行を書記素クラスタ単位にする直しも入った。同じ日に、pasta のスキル 3 本を `pasta-skills.zip` にしてリリースに添える直しが入り、v0.4.2 を出した。2026-10-24 に依存を更新した（PR#187）。同じ日に、0.4.3 の 4 本（セーブ・ログの表・サイドカーの TOML 化、mlua-stdlib の撤去と `@toml`、おすすめメニューをシーンで書く仕組み、`pasta_check release` が作者のフォルダを消す不具合の直し）と mlua 0.12 が入り、2026-10-25 に shiori4 の SHIORI3 互換モード（Windows 以外でも動く）が入って、v0.4.3 を出した。2026-10-31 に、日時変数の `＄季節`・`＄節句` を緊急の v0.4.4 として出した。2026-11-02 に、ponytail の 1 本目（`ponytail-lua-loader`）が入った。同じ日に 0.4.5 を廃止し、ponytail の残りを 0.5.0 へまとめた（開発者の決定）。2026-11-03 までに、ponytail の残りの 7 本（DSL のパーサ・シーン表と単語表・Lua のトランスパイラ・Lua のランタイム・エディタの道具・マニュアルの CI・SHIORI）が入り、ponytail は全部済んだ。同じ日に、シーンの属性を書いた順・型のまま実行時まで運んで Lua から読む仕組み（`SCENE.attrs`）、シーン名にぶら下がるシーンの数を読む関数（`シーン数`・`シーン数完全一致`）、DAP とキックの撤去（開発用のイベントとシーンマップに縮めた）、VSCode 拡張の開発の道具の SSP・areka の MCP への移行が入った。

現在の主題は次のとおり。

- **0.5.0: ponytail 監査の指摘の修正と属性関係の整備**（2026-10-29・2026-11-02 開発者の決定）。ponytail の 9 本（`ponytail-check-release` を除く。2026-11-02 に `ponytail-lua-loader` が入り、残りは 8 本。同じ日に `ponytail-lua-debug` を閉じて 7 本）と、属性の 4 本（`scene-attribute-store` → `scene-condition-attributes` → `scene-pick-modes`、`scene-count`）。2026-11-02 に 0.4.5 を廃止して、ponytail を 0.5.0 に入れた（開発者「0.4.5を廃止し、すべて0.5.0にしてください」「ウェーブについてはなるべくウェーブ数が少なくなるように」）。同じ日に、VSCode 拡張の開発の道具を DAP から SSP・areka の MCP へ移す 2 本（`vscode-mcp-client`・`debug-mcp-runtime`）を足した（開発者「0.5.0にしてもウェーブ数変わらなさそう」）。2026-11-03 に ponytail が全部入り、同じ日に属性の 2 本（`scene-attribute-store`・`scene-count`）と MCP の 2 本も入って、残りは属性の 2 本（`scene-condition-attributes` → `scene-pick-modes`）。spec のウェーブ 2 つとリリースで出す見込み（下の「0.5.0 の範囲」と棚卸）。
- **版がまだ決まっていないもの**: 2026-10-21 に起票した `scene-exact-match`、2026-10-23 に起票した `pasta-dll-multiarch`、2026-10-24 に起票した `vsce-4-upgrade`・`pest-2-9-4-upgrade`（下の「依存の更新」）。まとめは下の「版がまだ決まっていないもの」にある。
- **Phase 13: areka でノベルゲーム（0.6.0。2026-10-29 に 0.5.0 から順延）**。2026-10-10 の discovery で、合格ライン・通り道・台本の書き方・巻き戻しの方式を決め、spec を起こした（2026-10-17 の時点で 11 本）。属性の 2 本（`scene-attribute-store`・`scene-condition-attributes`）は、2026-10-29 に 0.5.0 へ移した。
- **Phase 11 の残り**: 文中のシーンリンク（`scene-anchor-link`）。どの版に入れるかは、まだ決めていない（下の棚卸の「開発者の判断が要るもの」）。

Phase 12（初心者向けの入門ガイド）は、2026-10-17 の入門ガイドの絵で完了した（「完了フェーズ」）。

このほか、リリース手順（spec `release-workflow`）の CI に合わせた書き換えと、CI での初回のリリース（v0.3.8）は済んだ（2026-10-10）。リリース手順は `release` スキル（`/release`）へ移し、spec `release-workflow` は完了にした（2026-10-11）。

**優先順位（開発者の方針）**: 0.4.0（主題「入門者にやさしく」）は 2026-10-17 に、0.4.1（致命的でない不具合と、窓際のぱすたさんからの要望の 2 本）は 2026-10-18 に出した。0.4.2（致命的でない不具合。2026-10-18 に範囲を決めた）は 2026-10-23 に出した。0.4.3（セーブの TOML 化・mlua 0.12・おすすめメニュー・SHIORI3 互換モード）は 2026-10-25 に、0.4.4（日時変数の `＄季節`・`＄節句`。緊急）は 2026-10-31 に出した。次の節目は 0.5.0（ponytail 監査の指摘の修正と属性関係の整備。2026-11-02 に 0.4.5 を廃止してまとめた）、0.6.0 の目標は、areka でノベルゲームが作れるようになること（2026-10-10。2026-10-29 に 0.5.0 から順延）。ponytail 監査の指摘の修正は、2026-10-22 に版に縛らない隙間の spec にしたが、2026-10-23 に 0.4.3 に入れると決め直した。2026-10-24 に、0.4.3 をセーブの TOML 化（致命的な不具合）と mlua 0.12 の版に組み替え、ponytail は 0.4.5 へ移した（`ponytail-check-release` だけ 0.4.3 に残す）。ウェーブは、今のゴールの範囲の spec だけで組む（2026-10-16）。致命的な不具合は、見つかりしだい割り込ませる。致命的でない不具合は、次の 0.4.x に入れるかを開発者が決める（2026-10-17 に 0.4.1 へ回した方針の続き。2026-10-18 に `lua-cmodule-robustness` を 0.4.2 に入れた）。2026-10-29 に、0.4.5（ponytail）の次の 0.5.0 を「属性関係の整備」にし、ノベルゲームを 0.6.0 へ順延した（開発者の決定。属性は窓際のぱすたさんの「開示の階段」とノベルの分岐の両方の土台で、早めに仕上げておきたい）。

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
- **シーン名の別名**: pasta.toml の `[scene.alias]` で定義する。既定の表は 7 件（`会話 → OnTalk`・おすすめメニューのシーン 3 件・ボタンの名前 3 件）で、書いたキーだけ既定を置き換える（空の配列で消す・見出しだけなら既定のまま・既定とぶつかれば読み込みエラー。2026-10-24 `recommend-menu-scenes`）。完全一致の 1 段だけで、キー正規化の前段で登録・検索の両側に効く。
- **式の中の値なし**: 式の中の nil は、算術なら 0、連結なら空文字列とみなし、ログを出さない。
- **リリース**: タグ `vX.Y.Z` の push で `.github/workflows/release.yml` が検証・ビルド・公開まで行う。ビルドした成果物は git で追跡しない。シェルの画像は追跡を続ける。配布物は `pasta.dll.zip`・`hello-pasta.nar`・VSIX・`pasta_check.zip`・`pasta-skills.zip` の 5 つで、GitHub Release には、それにリリースの全件の一覧を足した 6 つを添える（`pasta-skills.zip` は 0.4.2 で `pasta-skills-release-asset` が足した）。
- **hello-pasta の辞書**: 1 段 1 ファイル（`dic/01-boot.pasta`〜`12-lua.pasta`）で、段階表は `crates/pasta_sample_ghost/STAGES.md`。全段が起動することをテストが確かめる。
- **入門ガイド**: Claudia が全編を語る（`getting-started` に限った執筆規約の例外）。段の章と段階辞書を逐語で照合する（`book/tools/tutorial-check.mjs`）。辞書を変える spec は、同じ変更で入門の章の作例を直す。絵は `book/SCREENSHOTS.md` の手順で撮り直す。
- **マニュアルの見た目**: mdBook のまま「Claudia のマニュアル」。台詞は顔アイコン付きの部品で書く（記法は `book/AUTHORING.md`）。印刷用のページ（`print.html`）は作らず、章の印刷はテーマが受け持つ。
- **実行時の失敗**: シーンが生きている間の失敗は、`pasta/failure.lua` の `FAILURE.report` が、その場所に `【種類：対象（説明）】` の表記とログ 1 行を出す。切り替えは pasta.toml の `[logging] balloon_warning`。新しい種類は `FAILURE.LABEL` に足す。スクリプトが止まるエラーは `pasta/shiori/stop.lua` が受け、`【スクリプトのエラー：…】`・`【時間切れ：…】` の表記を `200 OK` と `X-Error-Reason` の行で返す（`balloon_warning = false` なら、これまでどおり 500）。
- **辞書の検査**: `pasta_check validate` が pasta.dll と同じ規則で辞書を検査し、`pasta_check release` の段 4 の関門と VSCode 拡張の保存のときの診断にもなる。検査を足すときは報告の種類 `FindingKind` に足す。
- **マニュアルの章の数**: 章の総数を数える 3 つの検査（`book/tools/verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`）は、`SUMMARY.md` の章と突き合わせる（2026-11-03 `ponytail-manual-ci`）。章を足す spec が総数を合わせる手間は無い。部ごとの決め打ち（`verify-scripts-test.mjs` の入門の章・内部設計の章の数、`verify-content.mjs` の `INTERNALS_CHAPTERS` など）は残るので、その部に章を足す spec は合わせる。マニュアルの検査は、全部の PR とリリースの関門で走る。
- **解析の上限**: 1 行の括弧の深さは 64 段まで、1 つの式の演算子は 256 個までで、超えると構文の誤り。大きな数も構文の誤りにする（2026-11-02 `ponytail-dsl-parser`）。新しい書き方を足す spec は、この上限の中に置く。
- **エディタの道具**: `pasta_lsp` は VSCode 拡張の WASM の解析器だけ（tower-lsp は撤去）。TextMate の文法は `grammar.pest` を写し、`tmGrammar.test.ts` が照合する。文法を足す spec は、両方を同じ変更で直す（2026-11-03 `ponytail-editor-tooling`）。
- **辞書の検査の順**: 飛び先などの検査（`pasta_check` の `validate/references.rs`）は、読めない辞書が無いとき（`skip_reasons` が空のとき）だけ走る。検査を足す spec は、この順を保つ。
- **アクターの装い**: アクターごとの表示の装いは pasta.toml の `[actor."名前"].font` に書き、タグ列（`\f[…]`・`\f[default]`）は `pasta_core` の `ActorFont` が作る。出し入れは `pasta/shiori/actor_style.lua` が受け持つ。1 つのスポットを複数のアクターで共有できる。
- **SSP の仮想の時刻**: `pasta/shiori/clock.lua` が受け持つ。SSP 2.9.18 の TimeMachine は pasta.dll の読む OS の時計まで仮想にするので、時刻の飛びで次の正時を決め直し、間隔は飛びを差し引いて数える。
- **budoux の列**: budoux は、列を動かすタグ（`\n`・`\n[…]`・`\c`・`\_l[…]`）の所で数え直し、列の字数を吹き出しごとに持ち越す（1 つの応答の中だけ）。
- **自動トークの停止**: `GLOBAL.pause_auto_talk`／`resume_auto_talk`（別名 `自動トーク停止`／`自動トーク再開`）。印は `STORE.auto_talk_paused` で、セッションの間だけ効く。止めている間は `virtual_dispatcher.lua` が OnTalk・OnHour・止まっている場面の続き（キックと `OnPastaScene` の 2 ビート目からも）を出さない。1 ビート目は流す。止めている間にシーンが `yield` で続きを作ると、警告して再開する（`act.lua` の `ACT_IMPL.yield`）。
- **シーンを再生するイベント**: `OnPastaScene`（大域・ローカルの名前の完全一致）と `OnPastaSceneAt`（ファイルと行）を `pasta/shiori/event/scene_play.lua` が受ける。デバッグの有効は設定の値で判断し、ポートは要らない（`@pasta_debug`）。見つからないときは `FAILURE.report` の表記を出す。外からのイベントの口を足す spec は、この置き方に揃える。
- **外からのイベント**: `OnPastaCallFunction`（`＞名前` の呼び出し）と `OnPastaSetSaveVariable`（グローバル変数の代入。値は Reference1 の文字列、型は Reference2 の `number`・`string`・`boolean`・`delete`。省けば `string`）を `pasta/shiori/event/external_control.lua` が受ける。いつでも有効。見つからないときは `OnPastaScene` と同じ失敗の表記を出す。
- **ハンドラの重ね**: `REG:register(id, fn)` で 1 つのイベントにハンドラを重ねる（`pasta/shiori/event/register.lua`。`REG` のメタテーブルの関数で、表のキーではない）。新しく登録したものから呼び、`nil`・`false` なら次へ、`true` なら 204、文字列・コルーチンならその応答にする。全部が見送ったら `REG[id]`、それも無ければイベントと同じ名前のシーン。ハンドラごとに新しい act を渡す。
- **チェイントークの続き**: 台詞（`talk`・`choice`）を出さなかったシーンは、待っている続き（`STORE.co_scene`）に触れない（判定は `sakura_builder.lua` の `BUILDER.speaks`）。台詞を出したシーンが割り込むと続きを捨て、`＊チェイントーク中断` があれば、割り込んだシーンの台詞をその台詞で置き換える。キック・`OnPastaScene`・`OnPastaSceneAt`・エラーは、`STORE.co_preempt` の印で黙って捨てる。外からクリックやイベントを受ける口を足す spec は、どちらの扱いにするかを決める。
- **シリアライズは TOML**: pasta が読み書きする値・ファイル・Lua の部品の書式は TOML にする。JSON は、作者の要望で求められるまで入れない（2026-10-24 開発者の方針）。型つきの値を 1 つ受けるだけなら、書式でなく Reference で型を渡す。DAP・LSP の JSON はプロトコルが決めているので対象外。残っていた JSON は、2026-10-24 に `mlua-stdlib-removal`・`toml-serialization` が追い出した。`@pasta_persistence` のセーブファイルは `save.toml` で、読めないものは `save.toml.broken` へ退避して空から始める。
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
| —（0.4.3） | セーブファイル・`@pasta_log` の表・ソースマップのサイドカーを TOML にする（読めないセーブは `.broken` へ退避）、mlua-stdlib の 6 つのモジュールを外して `@toml` を足す（`OnPastaSetSaveVariable` の型は Reference2）、おすすめメニューを辞書のシーンで書く（`[scene.alias]` は書いたキーだけ置き換える）、`pasta_check release` の危ない引数の組み合わせを段 1 の前に断る、mlua を 0.12 に上げる、shiori4 の SHIORI3 互換モード（`shiori_get_api(3)`・エンジンをインスタンスごとに持つ・Linux・macOS でビルドとテスト）（2026-10-25 リリース） | toml-serialization, mlua-stdlib-removal, recommend-menu-scenes, ponytail-check-release, mlua-0-12-upgrade, shiori4-v3-portable |
| —（0.4.4） | 日時変数の `＄季節`・`＄節句`（緊急。2026-10-31 リリース） | date-var-season-sekku |
| —（0.5.0・途中） | ponytail 監査（2026-10-19。8 つの領域を正しさ・安全・負荷・テスト・速さ・無駄の順に見た）の指摘の修正。ローダのキャッシュと同じモジュール名、解析を辞書の大きさに比例させて括弧と演算子に上限、シーン表・単語表の死んだ API と `fast_radix_trie` の撤去、Lua ブロックを書いたとおりに出す、Lua のランタイムの不具合と写し、構文の誤りを本当の位置に 1 件・tower-lsp の撤去、マニュアルの検査を全部の PR とリリースの関門に、unload なしの FreeLibrary・`Key:` の行（2026-11-02〜11-03。リリースはまだ。`ponytail-check-release` は 0.4.3、`ponytail-lua-debug` は DAP を捨てるので閉じた） | ponytail-lua-loader, ponytail-shiori, ponytail-dsl-parser, ponytail-editor-tooling, ponytail-lua-scripts, ponytail-lua-transpiler, ponytail-core-registry, ponytail-manual-ci |
| —（0.5.0・途中） | シーンの属性を書いた順・型のまま実行時まで運び、Lua から `SCENE.attrs` で読む（宣言・条件・フラグを 1 つの倉に。受け継ぎは `merge_attrs` の 1 か所）。シーン名にぶら下がるシーンの数を読む `シーン数`・`シーン数完全一致`。DAP とキックを撤去し、`[debug] enabled` を開発用のイベント（`OnPastaSceneAt`・`OnPastaDumpVariables`）を許す旗とシーンマップに縮めた。VSCode 拡張の開発の道具を SSP・areka の MCP へ移した | scene-attribute-store, scene-count, debug-mcp-runtime, vscode-mcp-client |
| —（0.4.2） | ゴーストから DLL の関数を LuaJIT の FFI で呼ぶ（`[loader] ffi`・`ffi.load` の置き換え。深いパス・ANSI に無い文字のフォルダ・隣の依存 DLL）、`pasta_check` を GitHub Release から単体で配る（`pasta_check.zip`）、シーンから自動トークを止める・再開する（`自動トーク停止`／`自動トーク再開`）、MCP から指名したシーンを再生する（`OnPastaScene`・`OnPastaSceneAt`）、外から `＞名前` を呼ぶ・変数を書く（`OnPastaCallFunction`・`OnPastaSetSaveVariable`）、`REG:register` でハンドラを重ねる、台詞を出さないイベントでチェイントークの続きを捨てない（割り込みには `＊チェイントーク中断`）、台詞の自動ウェイトと改行を書記素クラスタ単位にする、pasta のスキル 3 本を `pasta-skills.zip` で配る（2026-10-23 リリース） | lua-cmodule-robustness, pasta-check-release-asset, ontalk-suppression, scene-play-event, external-control-events, reg-handler-chain, chain-talk-interrupt, talk-grapheme-clusters, pasta-skills-release-asset |

ロードマップを置く前の初期開発（DSL・トランスパイラ・ランタイム・SHIORI・alpha リリースなど）の spec も `.kiro/specs/completed/` にある。

### 常駐 spec（再実行型・`completed/` へ移さない）

- `review-improvement-loop` — レビュー領域 × 7 次元の改善ループ

### 却下（2026-10-01）

- `pasta-sstp-live-output`（SSTP/`\![raise]` による push 出力）— 目的はシーンキックで達成済み。新しい動機が生じたら、その動機から新規に起票する。
- `pasta-authoring-window`（`*.pasta` 編集・プレビュー専用ウィンドウ）— VSCode で実現済み。VSCode 非依存のオーサリング環境が必要になったら再検討する。

### 統合して閉じたもの（2026-10-29）

- `call-attribute-filter`（呼ぶ側の `＞シーン＆k＝v`）— 2026-10-29 の discovery（ghost_dev「ぱすた開発」の要件メモ。窓際のぱすたさん「開示の階段」）で、条件は場面の側に書く（`＊雑談_今の部屋　＆＄＊経験値≧500`）と決め、新しい spec `scene-condition-attributes` に統合した（開発者の決定）。2 つは役目が重なる。呼ぶ前にローカル変数を入れれば（`＄気分＝「高」` → `＞告白`、受ける側は `＊告白　＆＄気分＝「高」`）、場面の側の条件だけで呼ぶ場所ごとに絞れるので、呼ぶ側に条件を書く書き方は、ローカル変数では書けない事例が出るまで作らない。旧 brief は git の `1c88e1e5` で読める。この roadmap の古い節に残る `call-attribute-filter` は、`scene-condition-attributes` と読み替える（Call の文法・`parse_action.rs`・`element_gen.rs` の Call は触らない見込みなので、ファイルの重なりは減る）。

### 統合して閉じたもの（2026-11-02）

- `ponytail-lua-debug`（`pasta_lua/src/debug` の監査の指摘。Lua 表示で `.pasta` のブレークポイントが死ぬ・ステップが抜けない・数の表示・DAP の無応答・行ごとの費用・死んだコード）— 2026-11-02 の要件ディスカッションの後に閉じた（開発者の決定）。VSCode デバッガー（DAP）の連携を捨て、SSP・areka の MCP 経由に移す方針が出たためである（セッション「Pasta DSL MCP移行検討」の discovery。版は同じ日に 0.5.1 から 0.5.0 に改めた）。直す対象の大半（`dap/`・`session/`・`transport/`・`wiring/`・`breakpoints`・`hook`・`inspect` など、約 6.2k 行）は、DAP と一緒に消える。残る部品（`source_map/`・`playscene.rs` の `resolve_position`・設定で有効かの判定。約 1.6k 行）への指摘は、DAP 撤去の `debug-mcp-runtime`（0.5.0）が受け継ぐ。受け継ぐ指摘は 6 つある。
  - `playscene.rs`・`scene_index.rs` の `#![allow(dead_code)]`
  - 働きの無い `insert_file`
  - テストだけが使う対応表の問い合わせ
  - `canonicalize_pasta_file`
  - `DebugConfig::resolve` の引数と `from_file`
  - UNC の `file://` でホスト名が消えること

  `book/src/debug/source-level.md` の「切り替えても `.pasta` のブレークポイントは有効」は事実と違うが、`vscode-mcp-client` がその頁ごと消すので、0.5.0 の外へは出ない。`ponytail-editor-tooling` が範囲から外した DAP の側の直し（`host` の廃止など）は、`vscode-mcp-client` が受け継ぐ。この spec の brief は git の `ae04a28b` で読める。

## 0.5.0 の範囲：ponytail 監査の指摘の修正と属性関係の整備（2026-10-29・2026-11-02 開発者の決定）

2026-11-02 に 0.4.5 を廃止し、ponytail 監査の指摘を直す 9 本（`ponytail-check-release` を除く）を 0.5.0 に入れた（開発者「0.4.5を廃止し、すべて0.5.0にしてください」「ウェーブについてはなるべくウェーブ数が少なくなるように」）。0.5.0 は、それと属性関係の整備の 4 本である（2026-10-29 開発者の決定「今存在する 0.5.0（ノベルゲーム）は 0.6.0 に順延し、0.5.0 に属性関係の整備を置く」。`scene-count` は 2026-11-01 に足した）。同じ日に、VSCode 拡張の開発の道具を DAP から SSP・areka の MCP へ移す 2 本（`vscode-mcp-client`・`debug-mcp-runtime`）を足した。ponytail は、2026-11-03 までに 8 本が入り、`ponytail-lua-debug` は DAP を捨てるので閉じた（「完了フェーズ」と「統合して閉じたもの（2026-11-02）」）。シーンの属性を、実行時に意味を持つものにする。窓際のぱすたさんの「開示の階段」（段に届くまで話を出さない）と、0.6.0 のノベルの分岐の両方が、この上に乗る。brief に残る「0.4.5」は、0.5.0 と読み替える。

- **ウェーブの数**: 残りは 2 本。`scene-condition-attributes` → `scene-pick-modes` の順で、候補の袋と控え（`scene_table.rs`）・`search/context.rs`・`scene.lua` を共有するので、同じウェーブに置けない（属性の倉が入っても、袋と控えは倉の外にあるので重なりは消えない。2026-11-03 の 2 回目の棚卸で確かめた）。spec のウェーブ 2 つとリリースで、3 つになる。
- 公開クレートの pub API は「pasta の部品で、外部から使う安定した面ではない」と、README と `tech.md` に書いてある（ponytail の決定。2026-10-19）。

| 順 | spec | 要件定義 | 規模 | 始められる時期 |
| -- | ---- | -------- | ---- | -------------- |
| 12 | scene-condition-attributes | Opus | 11〜15 | **ウェーブ 1（2026-11-03 の 2 回目）**。先に要った `scene-attribute-store`（属性の倉）と `scene-count`（`search/context.rs`）は入った。文法・構文木・運搬・着色は倉の spec が済ませたので、`grammar.pest`・パーサ・LSP・VSCode の文法定義には触らない |
| 13 | scene-pick-modes | Opus | 9〜11 | ウェーブ 2 の見込み。12 番と候補の袋と控え（`scene_table.rs`）・`search/context.rs`・`scene.lua` を共有するので、12 番の後に置く |
| 17 | 0.5.0 のリリース（`release` スキル） | Opus | — | 上の全部の後（ウェーブ 3 の見込み） |

- `scene-exact-match`（`＞＞` の完全一致）は属性ではないので、この版に入れていない。ゴースト側の見立て（2026-10-29）は「入っていれば助かるが必須ではない。入れるなら 0.5.0 の 3 本の後（4 番目）。0.6.0 でも構わない」（emo2 と窓際のぱすたさんは `OnInstallCompleteAll` を DSL のシーンにできず Lua の REG で受けているが、今は動いていて困っていない）。版は判断待ちのままで、0.6.0 の `novel-talk-flow` が前提にするので、遅くとも 0.6.0 に入れる。`scene-condition-attributes`・`scene-count` とは検索の道（`search/context.rs`）を共有するので、同じウェーブに置かない。

## 0.5.1 の範囲：同じ話し手が続く行の改行（2026-11-05 開発者の決定）

v0.5.0 の後の即時の版（開発者「版は即時（0.5.1）にしましょう。早く欲しい」）。窓際のぱすたさんの縦書きの辞書を、ネタ帳と同じ 1 行 1 列の形に戻すための 1 本である。

| 順 | spec | 要件定義 | 規模 | 始められる時期 |
| -- | ---- | -------- | ---- | -------------- |
| 1 | actor-line-newlines | Opus | — | すぐ（Dependencies: none）。`cue-command-runtime` と同じウェーブに置かない（行の頭の生成の場所を共有する） |
| 2 | 0.5.1 のリリース（`release` スキル） | Opus | — | 1 番の後 |

## 0.5.3 の範囲：VS Code を使わないエディタ pasta-recipe（2026-11-12 開発者の決定）

v0.5.2 の次の版で、最優先（開発者「次のリリースにする。最優先事項」）。VS Code を入れたくない作者のために、pasta DSL を編集できる軽いアプリ（仮名 `pasta-recipe`）を出す。0.6.0（ノベル）は動かさない。

- **形**: Tauri 2（2.12.x）＋ Vite ＋ Monaco の、Windows x64 の単一の exe（`tauri build --no-bundle`。WebView2 は OS のもの）。ゴーストのフォルダを exe にドラッグドロップすると、その `ghost/master` を開く。最近開いたフォルダはアプリの置き場に TOML で残す。リリースの成果物に `pasta-recipe.zip` を足す。
- **解析**: Tauri の Rust の側が `pasta_lsp`・`pasta_check` をライブラリとして直接呼ぶ（exe に静的にリンク。WASM は通らない）。`editors/recipe/src-tauri` は別の Cargo ワークスペースにし、Tauri の依存をルートの `Cargo.lock` に混ぜない。
- **ゴーストを動かす機能**: VS Code 拡張の実行の流れ（MCP クライアント・リロード・シーン実行・変数ダンプ）を `editors/shared/` に移して共有し、入出力だけを Node と Tauri で差し込む。保存変数の書き換え（今の `OnPastaSetSaveVariable`）と、ローカル変数を入れたシーンの実行（新しい口）を足す。
- **退けた案**: Electron（100MB を超える）、PWA（127.0.0.1 の MCP に届かない）、Rust ネイティブの GUI（エディタ部品が無い）、フロントで WASM を動かす案（段が増える。開発者「WASM にはこだわらない」）、実行の流れを Rust へ移植する案（2 本になる）。
- **範囲外**: 補完・ホバー・定義へ移動、Lua の構文診断とステップ実行、mac・Linux、インストーラ・署名・自動更新、`.nar` の作成。
- **ウェーブの数**: spec のウェーブ 3 つとリリースで、4 つ。

| 順 | spec | 要件定義 | 規模 | 始められる時期 |
| -- | ---- | -------- | ---- | -------------- |
| 1 | vscode-scene-outline-play | Opus | — | ウェーブ 1。解析結果にシーンの並び（`scenes`）を足す。`pasta-recipe` のツリーがそれを使う（2026-11-09 起票。版は未定だったのを 0.5.3 に入れた） |
| 2 | scene-play-locals | Opus | — | ウェーブ 1。`OnPastaScene`・`OnPastaSceneAt` の Reference2 に TOML のインライン表でローカル変数を渡す口。1 番とは触るファイルが重ならない（こちらは `pasta_lua`） |
| 3 | pasta-recipe | Opus | — | ウェーブ 2。アプリの本体（ツリー・エディタ・着色・診断・ゴースト全体の検査・`.lua`・保存・最近開いたもの・リリース CI・マニュアル）。1 番の後 |
| 4 | pasta-recipe-dev-actions | Opus | — | ウェーブ 3。実行の流れを `editors/shared/` へ移し、アプリにシーンの ▶・リロード・保存変数の表と書き換え・ローカル変数を入れた実行を足す。1 番（`devCommands.ts`・`runSceneAtCursor.ts`）・2 番・3 番の後 |
| 5 | 0.5.3 のリリース（`release` スキル） | Opus | — | 上の全部の後（ウェーブ 4） |

## 棚卸（2026-11-03・main `b03a1ca5`）

同じ日の 1 回目（main `c575febd`）のウェーブの 4 本が、どれも入った。`scene-count`（PR#230）・`vscode-mcp-client`（PR#231）・`debug-mcp-runtime`（PR#233）・`scene-attribute-store`（PR#234）。ほかに、cc-sdd の記事の絵（PR#232。spec なし）が入った。未完了は 18 本（常駐の `review-improvement-loop` を除く）、完了は 203 本。作業中のブランチは無い（ほかのワークツリーとブランチは、どれも main に入った後のもの）。今のゴールは 0.5.0 である（v0.5.0 のタグはまだ無い）。前回の記録は git の履歴にある。

- 早道を使った。前回のウェーブが全部入り、その後の steering と brief の書き換えは入った 4 本の完了の分だけで、前回が先のウェーブを見込んでいたため。測り直したのは 0.5.0 の残りの 2 本（`scene-condition-attributes`・`scene-pick-modes`）だけで、どちらも brief を書き直した。ゴールの外の 16 本は前回の測定のまま。
- `scene-condition-attributes` は 14〜18 から 11〜15 に軽くなった。属性の文法（`grammar.pest` の `cmp_op`・`attr_cond`・`attr_flag`）・構文木・実行時への運搬・着色は、`scene-attribute-store` がもう入れた。倉は 1 つで、宣言・条件・フラグが同じ並びに入る（Rust は `SceneInfo.attributes`、Lua は `SCENE.attrs`）。受け継ぎの規則は `pasta_dsl` の `merge_attrs` の 1 か所だけ。文字列の写し（`HashMap<String,String>`）は採らなかったので、「写しを消す」約束は無い。残るのは、`scene_table.rs` の `filter_by_attributes`（今は宣言の属性だけを文字列で比べる）を作り直し、条件を評価して袋で絞ること。
- 入る条件は満たした。前回の「自動転記を `virtual_dispatcher.lua` に置くなら `debug-mcp-runtime` の後」は意味が無くなった（キックの分かれ道は消えた。時報の転記は `virtual_dispatcher.lua` に残り、選択肢の飛び先は `choice_select.lua` が `SCENE.search` を直に呼ぶ）。
- `scene-pick-modes` は 9〜11 のまま。倉が入っても、候補の袋と控えは倉の外（`scene_table.rs` の `select_from_cache`・`SceneCacheKey`）にあるので、`scene-condition-attributes` との重なりは消えない。`grammar.pest` の衝突は無くなった（条件の spec は文法に触らなくなった）。単語の重みだけなら並べられるが、9〜11 の spec を割ってもシーンの側が待つので、ウェーブは減らない。0.5.0 は、spec のウェーブ 2 つとリリースで出せる見込み（前回の見込みより 1 つ少ない。今のウェーブが前回の見込みの 2 にあたる）。

### 即時修正（spec なし・本棚卸で実施して閉じた）

- この roadmap — 入った 4 本を「完了フェーズ」の 0.5.0（途中）の行にまとめ、台帳と既存の spec の更新の `[x]` の行を消した。0.5.0 の表から入った 4 本の行を消し、残りの 2 本とリリースの時期を書き直した。入った spec を Dependencies に書いていた行（`cue-command-runtime` の 2 行・`scene-condition-attributes`・`scene-pick-modes`）と、Phase 11 のウェーブ構成、依存の更新の席の `debug-mcp-runtime` の見込みを直した。
- `steering/tech.md` — `pasta_lua` の依存から、撤去した `socket2` の行と、`serde_json`（DAP のメッセージ）・ソースマップのサイドカーの記述を消した。`@pasta_debug` の行に、保存変数の書き出し（`OnPastaDumpVariables`）を足した。
- `steering/structure.md` — `pasta_lua/src/debug/` の木を、今の 4 つ（`mod.rs`・`config.rs`・`playscene.rs`・`source_map/`）に書き直した。`code_gen/source_map.rs` の説明から消えた `PastaPos` を外し、src 内テストの例と、分け方の例（`debug/dap/`）に撤去の断りを足した。
- `steering/product.md` — 完了の数（203）、VSCode 拡張・デバッグ基盤の節に MCP の 2 本の行、現在地（0.5.0 の残りは属性の 2 本）。
- コードとマニュアルの小さな直し: `code_gen/element_gen.rs` 447 行のコメントの「breakpoints」を、実行時のエラーの行に直した（`debug-mcp-runtime` が、そのウェーブの約束で残した 1 語）。`editors/vscode/src/mcpClient.ts`・`src/test/fakeMcpServer.ts` の頭のコメントの spec のパスを `completed/` に直した。`book/src/debug/index.md`・`troubleshooting.md` の「開発用のイベントを許したことを示す info の行」に、`debug-mcp-runtime` が決めた文（`Debug enabled: development events are allowed`）を添えた（`vscode-mcp-client` が仮の言い方で書き、文言が決まったら足すと残していた）。
- 確かめて閉じたもの: README・各クレートの README・マニュアルに、DAP の待ち受けの番号（9276）や提示モードの記述は残っていない。`scene-attribute-store` が属性の端から端の確かめ（辞書から `@pasta_search` まで）を `scene-condition-attributes` へ回した分は、その brief にもう入っている。`debug-mcp-runtime` の申し送り（`spawn_actor_thread_with` の引数と `RuntimeConfig` の欄）は `shiori4-v4-host` の brief に入っている。

### 統合・分割・改名

- なし。20 タスクを超えるものは無い。

### 起票と、バックログに残したもの

- 起票: なし。入った 4 本の申し送りは、どれも先の spec の brief に入っている。バックログは前回のまま。
- `scene-count` の研究の記録が範囲の外とした「`＠名前（…）` の関数名を辞書の検査で確かめる」は、求めた人がまだいないので spec にしない。

### 要件定義のモデル

ウェーブの 1 本（`scene-condition-attributes`）は Opus。書き方と意味は開発者が決めてあり、倉も入った。残る問い（評価の置き場＝Rust の袋に Lua の判定を渡すか Lua で絞るか・自動転記の範囲・`＆優先` の別名）は、どれも推奨を付けられる。

### ウェーブ（2026-11-03）

今のゴール（0.5.0）の先頭の 1 本を、1 つのウェーブにする。`scene-pick-modes` は、候補の袋と控えを共有するので、このウェーブの後に入る。

| spec | 子孫 | 種別 | 規模 | 要件定義 |
| ---- | ---- | ---- | ---- | -------- |
| scene-condition-attributes | 約 9（`scene-pick-modes`、ファイルの順番で `scene-exact-match`・`scene-anchor-link`、0.6.0 の `hello-novel-sample` ほかノベルの数本） | 機能（Lua を使わずに、条件で出す場面を変える書き方が無い。ゴースト側の要望で、書き方と意味は開発者が決めた） | 11〜15 | Opus |

開始コマンド（`/kiro-next` の報告と同じ形。俯瞰の頁が、この塊をコピーのボタンつきで出す）:

```
Wave 1本
Opus
/kiro-start scene-condition-attributes
```

同じウェーブの約束:

- 1 本だけのウェーブなので、ほかの spec との約束は無い。`search/context.rs`（1,060 行。テストを含む）に足すテストは別のファイルに置く。`grammar.pest`・パーサ・`pasta_lsp`・VSCode の文法定義には触らない見込み（倉の spec が済ませた）。

### その先のウェーブ（2026-11-03 の見込み）

見込みで、前のウェーブが入った後の棚卸で確かめてから始める（`/kiro-wave` が始めるのは、上の「ウェーブ」だけ）。0.5.0 は、このウェーブの後に spec のウェーブが 1 つ（2）と、リリースのウェーブ（3）で出せる見込み。

| ウェーブ | spec | 入る条件 | 要件定義 |
| -------- | ---- | -------- | -------- |
| 2 | scene-pick-modes | `scene-condition-attributes` が入った後（外れた札を袋に残す形と `＆優先` を先に決める。`scene_table.rs`・`search/context.rs`・`scene.lua`・`validate/report.rs` を共有する） | Opus |
| 3 | 0.5.0 のリリース | 0.5.0 の spec が全部入った後 | Opus |

```
Wave 2（見込み）1本
Opus
/kiro-start scene-pick-modes
```

```
Wave 3（見込み）リリース（0.5.0 の spec は 0 本）
/release 0.5.0
```

ゴールの後: 版の決まっていないもの: `scene-exact-match`（`scene-condition-attributes`・`scene-pick-modes` と `search/context.rs` を共有する）、`pest-2-9-4-upgrade`、`pasta-dll-multiarch`、`vsce-4-upgrade`、`scene-anchor-link`（`scene-condition-attributes` の後）。0.6.0: `novel-areka-contract`・`novel-variable-scope`・`shiori4-v4-host`（先頭。先に要ったものはどれも入った）、`cue-command-runtime`（`scene-attribute-store` が入ったので、先に要るのは `novel-areka-contract` の語彙の名前）。

### 開発者の判断が要るもの（ウェーブは止めない）

- 前回からの持ち越し（推奨は前回のまま）: `pasta-dll-multiarch` の版（推奨 0.6.0）、`vsce-4-upgrade`・`pest-2-9-4-upgrade` の版（どちらも先に要るものが無くなった。`pest-2-9-4-upgrade` の `Cargo.lock` の相手だった `debug-mcp-runtime` は入った）、`scene-exact-match` の版（遅くとも 0.6.0）、`scene-anchor-link` の版、会えた日数を数える spec を起票するか（`scene-condition-attributes` の brief）、`SOUL.md` §2・§5 の記述（§「Radix Trie 等」の例も、`fast_radix_trie` を外した後は古い）、`steering/structure.md` の作り直し、ghost_dev の emo2 開発のフォルダに残る撮影の保存データとログ。
- `review-improvement-loop`（常駐）が次に回るときは、ponytail の 10 本と MCP の 2 本が入った後のコードを基準にする。

### 人手の確認が残っている項目

- リリースの失敗した job の再実行の読み方は、まだ確かめていない（下の「リリース」。v0.4.3 の失敗はタグの打ち直しで出したので、再実行は使っていない）。
- 手元の `release.bat`（Windows PowerShell 5.1）で、`pasta_check.zip` と `pasta-skills.zip` の段を含めて通しで走ることを確かめる（`pasta-check-release-asset`・`pasta-skills-release-asset`）。
- v0.4.3 を出したので、3 人以上のシェルで `char*` のおすすめメニュー（Reference4・Reference5）を実機で確かめる（`recommend-menu-scenes` が公開の後に回した）。
- `ponytail-manual-ci` のマージの後の確かめ: Pages の新しい版と `/classic/`、main での `release-setup-check.yml`、次のリリースタグの公開（`upload-artifact` v7 → `download-artifact` v8 の受け渡しは、0.5.0 のリリースで初めて本番の道を通る）。
- `ponytail-shiori` の DllMain の道（ゴーストの切り替えと SSP の終了が今と同じに見えること）は、SSP の実機で確かめられれば確かめる（任意。spec の中では子のプロセスの `FreeLibrary` のテストで確かめた）。
- `vscode-mcp-client` の実機の確かめの残り: 「保存変数を表示」を、`debug-mcp-runtime` の入った pasta.dll で SSP の実機に通す（spec の中の実機の 1 回は、その前の pasta.dll だったので「新しく書き出されませんでした」の案内まで）。areka でも 3 つのコマンドを通す（その日は areka が動いていなかった）。

## Phase 11 の残り: 属性セマンティクスとシーンリンク

### 境界戦略

- **分割理由**: シーン属性を 2 つに分けた（2026-10-04）。保持・継承・読み出し（`scene-attribute-store`）と、Call の属性フィルター（`call-attribute-filter`）。どちらも 0.5.0 の道筋に入った（下の「Phase 13」）。実行時の失敗の出力の 2 本（`failure-output-unification`・`runtime-error-balloon`）は、2026-10-17 までに完了した。
- **共有接点**: `crates/pasta_lua/pasta_scripts/pasta/act.lua` は 1 ウェーブに 1 spec だけが持つ。`call-attribute-filter`・`scene-anchor-link` が触る。`crates/pasta_lua/src/code_gen/element_gen.rs` も 1 ウェーブに 1 spec だけで、`call-attribute-filter` と `scene-anchor-link` が触る。`scene-attribute-store` と `call-attribute-filter` は、`scene.lua` と `pasta_core` のシーン登録（`scene_registry.rs`・`scene_types.rs`）を共有するので、この順に置く。選択肢の出力（`pasta_scripts/pasta/shiori/sakura_builder.lua`）と選択肢の振り分け（`shiori/event/choice_select.lua`）は、`scene-anchor-link` と Phase 13 の spec が共有する。辞書の検査に検査を足す spec は、`call-attribute-filter`・`scene-anchor-link` が飛び先の検査（`validate/references.rs`）を、`cue-command-runtime`・`novel-talk-flow` が設定の検査（`validate/config.rs`）を共有する。
- **シーン名の別名との関係**: 別名の解決は検索の入口（`search/context.rs`）でキー正規化の前に行う。`scene-attribute-store`・`call-attribute-filter` は、別名を当てた後の名前で属性を扱う。
- **結合テスト**: `pasta_shiori` の結合テストは、自己展開した本物のランタイムで動き、回避を書かない（`shiori-test-support-runtime`）。

### ウェーブ構成

| Wave | spec（並走可） | 種別 | ソースの持ち場 |
| ---- | -------------- | ---- | -------------- |
| 0.5.0（ウェーブ 1。2026-11-03 の 2 回目の棚卸） | scene-condition-attributes（2026-10-29 に `call-attribute-filter` を統合） | 機能 | `scene.lua`（`SCENE.search`・`co_exec`）、`act.lua`（検索へ act を運ぶときだけ）、`search/`、`pasta_core` の `scene_table.rs`（`filter_by_attributes`・`select_from_cache`）・要るなら `scene_types.rs`、イベントからシーンを起こす道（選ぶ前の自動転記）、`pasta_check validate`、マニュアル（入門の 7 段目を含む）。属性の文法・構文木・着色は `scene-attribute-store` が入れたので触らない |
| 版は未定 | scene-anchor-link | 機能 | `call-attribute-filter` と同じ文法・生成・`act.lua`・エディタのファイルに加え、`sakura_builder.lua`、`shiori/event/choice_select.lua`、`sakura_script/`（表示名を Lua の側で台詞として通してから `\_a` で包む案なら触らない）、VSCode 拡張の `wordRefDecorator.ts`（2026-11-03 の再測定で、箱が文法から規則を読むようになり、要らない見込み）、`pasta_check validate`（`references.rs`） |

`scene-attribute-store` は 2026-11-03 に入った。Wave 1〜3 の 12 本と、先行させた `scene-name-alias`・`expr-nil-coercion`、失敗の出力の 3 本（`failure-output-unification`・`shiori-test-support-runtime`・`runtime-error-balloon`）は完了した（「完了フェーズ」の Phase 11）。

### 文中のシーンリンク（2026-10-08 起票）

台詞の中の語を Wikipedia のリンクのようにし、クリックでそのシーンへ飛べるようにする（`scene-anchor-link`）。

- **方式**:
  - 記法は `＠？シーン名`（`＠単語　` と同じく、空白か改行で終える）と `＠？シーン名「表示名」`。
  - 出力はさくらスクリプトの `\_a`（アンカー）で、`\q` ではない。`\q` だと、トーク全体が選択肢待ちになるため。
  - クリックは `OnAnchorSelectEx` を、選択肢の振り分け（`choice_select.lua`）と同じ規則で受ける。
  - `【】` で囲む案は、文法として唐突なので採らなかった。
- **飛び先が無いとき**: 選択肢と同じく 204 を返す。失敗の見せ方は `pasta/failure.lua` の出口に乗せる。飛び先の無いリンクの検査は、この spec が `pasta_check validate` に足す（選択肢の飛び先の検査 `validate/references.rs` を使える）。
- **ウェーブ**: `grammar.pest`・`parse_action.rs`・`element_gen.rs`・`act.lua`・VSCode の文法定義を、`call-attribute-filter` と共有する。そのため 1 本ずつ入れる。

## Phase 13: areka でノベルゲーム（0.6.0。2026-10-29 に 0.5.0 から順延）

> **版の読み替え（2026-10-29 開発者の決定）**: ノベルゲームは 0.6.0 へ順延した。この節で「0.5.0」と書いてあるのは、ノベルゲームの版のことで、今は 0.6.0 と読む。0.5.0 は「属性関係の整備」になった（上の「0.5.0 の範囲」）。`scene-attribute-store`・`scene-condition-attributes` は 0.5.0 へ移したので、下の順番の 2 番と 7 番は、0.6.0 の時点では入っている前提で読む。

2026-10-10 の `/kiro-discovery` で起こした（main `88c4bc0e`）。国産 10 系統・欧米 9 系統のノベルゲームのスクリプト仕様を調べ、pasta と areka の現在地と照らした。調査ノートは `.kiro/specs/novel-areka-contract/` の `research-jp-engines.md` と `research-western-engines.md` にある。0.5.0 の目標（2026-10-10 開発者の決定）は、別リポジトリで開発中の areka で、ノベルゲームが作れるようになること。

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
  - 出力の組み立て（`pasta_scripts/pasta/shiori/sakura_builder.lua`）は、`cue-command-runtime`・`novel-talk-flow`・`novel-checkpoint-rollback` が触る。`scene-anchor-link` も触る。557 行あり（2026-11-03）、3 本が足すと 740〜870 行になる。`BUILDER.build` は込み入っているので、クリック待ちや `\x` の処理は、`actor_style.lua` のように別のモジュールに分ける。
  - 台詞の切り出し・自動ウェイト・budoux（`sakura_script/` の `tokenizer.rs`・`wait_inserter.rs`・`line_breaker.rs`）は、`novel-talk-flow` が触る（2026-10-23 の再測定では、`\x` で列を動かす `line_breaker.rs` の `column_move` だけ）。`scene-anchor-link` も、Rust の側を触るなら加わる。`line_breaker.rs` は 1,155 行だが、本体は 310 行までで、残りはファイルの中のテストである。テストを足す spec は `tests/sakura_script/` に置く。
  - 選択肢の振り分け（`shiori/event/choice_select.lua`）は、`shell-element-click` と `scene-anchor-link` が共有する。先に入る側が、振り分けを関数として取り出す。
  - 場面の関数を出すコード生成（`code_gen/scope_gen.rs`）は、`scene-attribute-store`・`cue-command-runtime`（2026-10-17 に `scene-stage-attributes` をまとめた）が触る。この順に置く。場面の関数の頭は `act:init_scene(SCENE, 書いたグローバル名, 書いたローカル名, 行の表)` で、行の表は最後の引数である（`runtime-error-balloon` が入れた。`transpiler.rs` の `fill_line_maps` が後から差し込む）。引数を足す spec は、この形を保つ。
  - `act.lua` は 812 行あり（2026-11-03）、1,000 行に近い。`cue-command-runtime`・`call-attribute-filter`・`scene-anchor-link`・`novel-talk-flow` が足すので、足す側が分け方を考える。ト書きの新しい入口は、別のファイルに置く。
  - 変数の書き方（`grammar.pest`・`code_gen/element_gen.rs` の `save.x`）は、`novel-variable-scope` が触る。文法を触る `call-attribute-filter`・`scene-anchor-link`・`cue-command-runtime` と同じウェーブに置かない。
  - 設定の読み込み（`loader/config/`）は、`cue-command-runtime`（別名表）と `novel-talk-flow`（ノベル向けの切り替え）が触る。
- **検証で分かった制約**（2026-10-10。詳しくは各 brief の「検証で見つかった穴」）:
  - NOTIFY のイベントには台本を返せない。ノベル用のイベントは、すべて GET で送ってもらう。GET の応答は 5 秒以内に返す。
  - 場面の検索は前方一致である（`＞第1章` は `第10章` にも当たる）。ノベル向けの設定では完全一致にする（`novel-talk-flow`）。
  - 出力の組み立ては `\x`（クリック待ち）をほとんど知らない（装いの出し入れ `actor_style.lua` だけは知っている。2026-10-18）。`\x` の後で、段落の区切り・列の持ち越し・サーフェスの復旧・budoux の数え方がずれる（`novel-talk-flow`）。
  - 時刻の問い合わせ（`clock.lua` の `\![get,property,OnPastaClock,…]`）は、暇な `OnSecondChange` の応答に載る。ノベルの吹き出しを読む間にも返りうるので、巻き戻しの区切りにしない（`novel-checkpoint-rollback`。2026-10-18）。
  - 走りかけのコルーチンは控えられない。区切りにできるのは、場面が最後まで走った応答だけである（`novel-checkpoint-rollback`）。
  - 宣言行に書いた属性（`＊名前＆k：v`）は、`scene-attribute-store` で倉に入るようになった（Lua からは `SCENE.attrs` で読む）。

### 順番

| 順 | spec | 要件定義 | 規模 | 先に要るもの |
| -- | ---- | -------- | ---- | ------------ |
| 1 | novel-areka-contract | Opus | 6〜9（文書だけ） | なし。**0.6.0 のゴールの先頭**（areka の側が早く動けるよう、最初に始める） |
| 3 | novel-variable-scope（2026-10-17 に `novel-checkpoint-rollback` から分けた） | Fable | 8〜10 | なし（文法と `element_gen.rs` を触るので、`call-attribute-filter`・`cue-command-runtime` と同じウェーブに置かない） |
| 4 | cue-command-runtime（ト書きと柱。2026-10-17 に `scene-stage-attributes` をまとめた） | Fable | 17〜21（超えるならト書きと柱に分け直す） | `scene-attribute-store`（柱の分。2026-11-03 に入った）。語彙の名前は `novel-areka-contract` から引く |
| 5 | shell-element-click | Opus | 5〜8 | `novel-areka-contract`（完全一致の場面選びは `touch-poke-shortcuts` が入れてある）。`scene-anchor-link` とは `choice_select.lua` を共有する |
| 6 | novel-talk-flow | Fable | 17〜21（超えるなら出力の分（Fable）と場面の流れの分（Opus）に分ける） | `cue-command-runtime`、`novel-areka-contract` |
| 8 | novel-checkpoint-rollback | Fable | 13〜15 | `novel-talk-flow`、`novel-variable-scope` |
| 9 | novel-save-slots | Opus | 8〜12 | `novel-checkpoint-rollback` |
| 10 | hello-novel-sample | Opus | 15〜18 | 2〜9 の全部と、areka の側の実装 |
| 11 | manual-novel-guide | Opus | 8〜12 | `hello-novel-sample` |
| 12 | shiori4-v4-host（2026-10-23 起票。同じ日の棚卸で Rust の土台だけにした） | Opus | 14〜17 | `shiori4-v3-portable`（2026-10-25 に入った）・0.5.0 の `ponytail-shiori`（2026-11-03 に入った）。ファイルの順番で 0.5.0 の `debug-mcp-runtime` の後（`actor/`・`shiori.rs` のキックの道。2026-11-03 に入った）。shiori4 の `abi-conventions`・`host-protocol` は 2026-10-24 の 0.1.0 で crates.io に出た。ノベルの合格には要らないので、ほかの 0.5.0 の spec を待たせない |
| 13 | shiori4-v4-lua-api（2026-10-23 の棚卸で `shiori4-v4-host` から分けた） | Fable | 11〜13 | `shiori4-v4-host` |
| 14 | 0.6.0 のリリース（`release` スキル） | Opus | — | 上の全部 |

2 番の `scene-attribute-store` と 7 番の `scene-condition-attributes` は、2026-10-29 に 0.5.0 へ移した（0.5.0 の 1 番・2 番。2026-11-02 の棚卸で表から外した）。

並べられる組み合わせ（2026-10-17 と 2026-10-18 の棚卸で測った）: `novel-areka-contract`・`scene-attribute-store`・`novel-variable-scope` は、0.4.3 の `shiori4-v3-portable` とソースを共有しない見込み（2026-10-23）。`novel-areka-contract` と `scene-attribute-store` は、互いにも共有しない。`shell-element-click` は `novel-areka-contract` の後、`scene-attribute-store` と並べられる（`scene.lua` を触らない約束で）。`novel-variable-scope` は `scene-attribute-store` と並べられる（`scene-attribute-store` が `partial.rs` と VSCode の文法定義を触らない約束で）。`cue-command-runtime` → `novel-talk-flow` → `novel-checkpoint-rollback` は、`act.lua` と `sakura_builder.lua` を共有するので 1 本ずつ入れる。

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
- 数のしきい値（好感度が 50 以上、など）の書き方 — 2026-10-29 に決めた。場面の側に `＆＄＊好感度≧50` と書く（`scene-condition-attributes`）。

## 版がまだ決まっていないもの

どの版に入れるかを、開発者がまだ決めていない spec（2026-10-24 の棚卸で、出した 0.4.2 の節から移してまとめた）。ウェーブには入れない。推奨は、上の棚卸の「開発者の判断が要るもの」にある。

- `pasta-dll-multiarch`（2026-10-23 起票）: x64 版と arm64 版の pasta.dll をビルドし、32bit 版と同じリリースで出す。見本ゴーストの .nar には、areka の探し方（x64 は `pasta.x64.dll` → `pasta.dll`、arm64 は `pasta.arm64.dll` → `pasta.x64.dll` → `pasta.dll`）に合う名前で同梱する。接尾辞の無い `pasta.dll` は SSP 向けの 32bit 版のまま。`pasta_check` は名前も arch も気にしないので今のまま使える。手を入れるのは `release.ps1`・`.cargo/config.toml`（aarch64 の `crt-static`）・配布物とリリース CI の検査・マニュアル（`[loader] ffi` で作者の DLL の arch が合わないと読めない注意）。aarch64 で LuaJIT がビルドできるかは確かめていない。`shiori4-v3-portable` の後（areka は shiori4 の入口で読むため）。版は開発者が後で決める（2026-10-23）。推奨は 0.5.0（2026-10-24 の棚卸）だったが、2026-10-29 にノベルゲームを 0.6.0 へ順延したので、推奨は 0.6.0。
- `scene-exact-match`（2026-10-21 起票。ghost_dev のヒアリング）: 中身は台帳の行にある。0.6.0 の `novel-talk-flow` が前提にするので、遅くとも 0.6.0。`scene-pick-modes` は 2026-10-29 に 0.5.0 へ入れた。
- `scene-condition-attributes`（2026-10-29 起票。`call-attribute-filter` を統合）: 版は 2026-10-29 に 0.5.0 と決めた（上の「0.5.0 の範囲」）。会えた日数をエンジンが数える別 spec を起票するか（推奨: 起票する）と、その変数の名前も開発者の判断待ち（brief の「決まらなかったこと」）。
- `scene-anchor-link`（2026-10-08 起票）: 中身は上の「Phase 11 の残り」の「文中のシーンリンク」にある。
- `vsce-4-upgrade`・`pest-2-9-4-upgrade`（2026-10-24 起票）: 中身は下の「依存の更新」にある。

## 依存の更新（2026-10-24 起票）

2026-10-24 の `/update`（PR #187）で、互換の範囲の更新と、直しの小さい互換の無い更新（esbuild・ctor・socket2・wintf-winmsg-executor・jsdom）は入れた。直しが大きいか、確かめ方が要る 3 件を spec にした（開発者の依頼）。同じ日に、mlua 0.12 を止めている mlua-stdlib を外す spec と、JSON を追い出す `toml-serialization` を足して 5 本にした。同じ日の開発者の決定で、`toml-serialization`・`mlua-stdlib-removal`・`mlua-0-12-upgrade` を 0.4.3 に入れた（「次のリリースのタイミングで、mlua 上げちゃいたい」）。`vsce-4-upgrade`・`pest-2-9-4-upgrade` は 0.4.3 に入れない（版は未定）。

- **`mlua-stdlib-removal`・`toml-serialization`・`mlua-0-12-upgrade`**: 0.4.3 で入った（「完了フェーズ」）。経緯と決定は各 spec の `.kiro/specs/completed/<name>/` にある。
- **`vsce-4-upgrade`**: `@vscode/vsce` 3.9 → 4.0。Node 22 が要る（`release.yml`・`release-setup-check.yml` は `ponytail-manual-ci` が 2026-11-03 に 22 にした）。`--azure-credential` の公開の道が 4.x でも通るかを、`release-setup-check.yml` で公開の前に確かめる。先に要った `ponytail-manual-ci` は 2026-11-03 に入った。`editors/vscode/package.json` を `vscode-mcp-client` と共有するので、その後に置く。
- **`pest-2-9-4-upgrade`**: pest 2.9.2 → 2.9.4 以降。2.9.4 は `std` で stacker・psm を必ず引き込み、psm が wasm32 のビルドに clang を要るので、手元の `build-wasm.ps1` が落ちる。wasm32 向けの C コンパイラを手元と CI に用意するか、上流に直してもらうかを要件で比べる。先に置いた ponytail の 3 本（`ponytail-dsl-parser`・`ponytail-core-registry`・`ponytail-editor-tooling`）は 2026-11-03 までに入った。上げた後も、括弧の走査が pest の守りより先に走るので、64 段までの辞書の結果は変わらない（`ponytail-dsl-parser` の申し送り）。
- **席**: `pest-2-9-4-upgrade` は `Cargo.lock` とルートの `Cargo.toml` を触る。同じく `Cargo.lock` を触る spec とは、同じウェーブに置かない（0.5.0 の `debug-mcp-runtime` は 2026-11-03 に入り、`socket2` を外した）。`vsce-4-upgrade` はリリースの席（`release.yml`）を触るので、リリースの実行中は main へ入れない。

## リリース（`release` スキル）

リリースの CI 化（`release-ci`）、リリース手順（spec `release-workflow`）の書き換え、CI での初回のリリース（v0.3.8）とその後片付けは、2026-10-10 までにすべて済んだ。タグ `vX.Y.Z` の push だけで公開まで進む。記録は `.kiro/specs/completed/release-workflow/first-ci-release.md` と、同じフォルダの `design.md` にある。手順は 2026-10-11 に `release` スキル（`.claude/skills/release/`）へ移し、spec は完了にした。手順を直すときは、スキルを直す。**残るのは、失敗した job の再実行の読み方の確認である**（v0.3.8 も v0.4.0（2026-10-17、run 39024925990）も v0.4.1（2026-10-18、run 39123004799）も v0.4.2（2026-10-23、run 39963557454）も失敗が起きなかった。v0.4.3 は最初の run 40173976849 が失敗したが、直したコミットでタグを打ち直した新しい run 40174683724 で出したので、job の再実行は使っていない。v0.4.4（run 41209054904）は失敗が起きなかった。失敗が起きたリリースで確かめる）。

- **以後のリリース**: 新しい作業ブランチ（ハーネスのワークツリー）で `/release` を実行する（版を指定するときは `/release 0.4.0`）。失敗した job の再実行の読み方（スキルの `SKILL.md`「まだ実地で確かめていない読み方」）を確かめるまでは Opus で実行し、確かめたら Sonnet で実行する（開発者決定 2026-10-10）。
- **席**: リリースの実行中は、`release.ps1`・`release.yml` を触る spec を main へ入れない。
- **リリースの本文の作り直し**（2026-10-11 起票 `release-notes-highlights`）: 今の本文は、コミットの件名を全件並べた一覧で、何が良くなったのかを読み取れない（v0.3.8 は 57 行）。本文を、注目に値するいくつかの内容を Claudia（令嬢）の口調で紹介する文章にし、全件の一覧は Assets に添付する。spec 名は載せない。見どころは、版を上げるコミットでリポジトリのファイルとして書き、リリース CI が本文に使う（開発者の決定。公開の後で書き換える案と、機械だけで短くする案は採らない）。この spec は、`release` スキルの段 4 の「5 ファイルだけ」を変える。見出しには元になった PR の番号を付ける。公開済みの v0.3.8 の 1 本も、同じ形に直す（それより前は触らない）。areka のリポジトリの `areka-P0-release-notes-highlights` と、決定と議題をそろえてある。
- **運用の注意**（一回限りのセットアップで分かったこと）:
  - Azure のリソースを `az` で作る・変えるには MFA が要る。WAM（Windows のサインイン窓）でログインすると `RequestDisallowedByAzure` で弾かれる。`az config set core.enable_broker_on_windows=false` にしてからブラウザーで `az login` し直す。ワークフローのマネージド ID には関係ない。
  - Marketplace の Members の管理には、公式の CLI が無い（vsce・az・gh のどれも扱えない）。画面から行う。
  - setup-check の `verify-pat` が失敗すると、vsce のエラー文にマネージド ID の識別子が含まれ、ジョブのログに出る。秘密の値ではないので直さない（2026-10-10 決定）。

## Specs (dependency order)

- [ ] shiori4-v4-host -- 版 4 に本格対応する Rust の土台。ホストのテーブルを受け、アクタースレッドからの `host->request` を SHIORI のスレッドへ中継する。post（SSTP に揃えた `SEND`＋`Script`・`Option: nobreak`、`NOTIFY`）と `Tag` の Rust の側、unload での無効化、ホストの `log` への転送、キックの即時化（チェイントークの続きの決まりと自動トークの停止を保つ）、Lua への低い入口と「版 4 で読まれた」の印（キックは 0.5.0 の `debug-mcp-runtime` が 2026-11-03 に消したので、キックの即時化は要らなくなる見込み。2026-11-03 の再測定）。取り決めの正本は shiori4 の `abi-conventions`・`host-protocol`（2026-10-23 起票。2 段の 2 段目。同じ日の棚卸で、Lua と DSL の口を `shiori4-v4-lua-api` へ分けた）。0.5.0 に入れる（2026-10-23 開発者の方針。ノベルゲームと同じリリース。2026-10-29 にノベルゲームとともに 0.6.0 へ順延）。`debug-mcp-runtime` と同じウェーブに置かない（`actor/`・`shiori.rs` のキックの道を共有する。あちらは 2026-11-03 に入った）。Dependencies: none（`shiori4-v3-portable`・`ponytail-shiori`・`debug-mcp-runtime` は入った）
- [ ] shiori4-v4-lua-api -- 版 4 のホストの機能を Lua と DSL から使う口。ホストのプロパティの読み出し（今の `act:get_property` と `CALLBACK.pending` の上に作る）、シーンからの SEND・NOTIFY、終わりを待って `OnHostComplete` で待っているシーンを再開する書き方（`REG:register` との並び・チェイントークの `STORE.co_preempt` との関係・待っている間の unload）、SHIORI3 だけのときの扱い、`pasta_check` の組み込みの名前、マニュアル（2026-10-23 の棚卸で `shiori4-v4-host` から分けた）。0.6.0（2026-10-29 にノベルゲームとともに 0.5.0 から順延）。Dependencies: shiori4-v4-host
- [ ] pasta-dll-multiarch -- x64・arm64 版の pasta.dll をビルドし、32bit 版と同じリリースで出す。見本ゴーストの .nar に areka の探し方に合う名前（`pasta.x64.dll`・`pasta.arm64.dll`）で同梱し、接尾辞の無い `pasta.dll` は SSP 向けの 32bit 版のまま残す。`release.ps1`・`.cargo/config.toml`・配布物とリリース CI の検査・マニュアル。aarch64 の LuaJIT のビルドは要確認（2026-10-23 起票。3 者会談の後の areka の相談から）。版は未定。`vsce-4-upgrade`（`release.yml`）・`shiori4-v4-host`（`build.yml`）と同じウェーブに置かない。Dependencies: none（先に要った `shiori4-v3-portable`・`ponytail-manual-ci`・`ponytail-shiori` は入った）
- [ ] novel-areka-contract -- pasta と areka の取り決め（台本に載せる語彙・目印・イベント・設定の値）を 1 つの文書にし、areka への依頼書として出す。コードは書かない。areka に依頼を出すための spec（2026-10-10 の discovery で起票）。Dependencies: none
- [ ] novel-variable-scope -- 変数の保存の範囲を、セーブごとと全セーブ共通の 2 つに分ける書き方。今の `＄＊x` と保存ファイルとの互換、保存できない値の扱い（2026-10-17 の棚卸で `novel-checkpoint-rollback` から分割）。Dependencies: none
- [ ] cue-command-runtime -- キューコマンド行 `！命令＠対象（引数）` を、実行時に `\![…]` として台本へ流す。日本語の名前の別名表、引数の穴（数字で始まる引数など）の手当て。柱に書いた `＆属性` とアクター指定行も、場面に入ったときに台本へ流す（受け継ぎと、Call から戻ったときの打ち直し）。2026-10-17 に `scene-stage-attributes` をまとめた。Dependencies: none（`scene-attribute-store` は入った）
- [ ] shell-element-click -- エレメントのクリック（名前つき）を、選択肢と同じ振り分けで場面へ届ける。シェルとして組んだシステム画面を、場面が動かせるようにする。Dependencies: novel-areka-contract（`scene-anchor-link` と `choice_select.lua` を触る順番に注意）
- [x] scene-condition-attributes -- 場面の側に条件を書いて候補を絞る。`＄` で始まる属性が条件（`＆＄＊経験値≧500`・`＆＄ｒ０＝「changed」`・フラグ `＆＄＊初回待ち`）で、演算子は `≧ ≦ ＞ ＜ ＝ ≠`、右辺は書いた値だけ、値なしは 0 か空文字列、全部落ちたら素通り（204）。外れた札は袋に残して今の巡に混ぜる。`＆優先：数`、イベントのシーンを選ぶ前の自動転記、`pasta_check validate` の注意、trace のログ（2026-10-29 起票。ghost_dev「ぱすた開発」の要件メモ。`call-attribute-filter` を統合して閉じた）。0.5.0 の 2 番（2026-10-29 開発者の決定）。`scene-pick-modes` と同じウェーブに置かない。Dependencies: none（`scene-attribute-store`・`scene-count` は入った）
- [ ] scene-anchor-link -- 台詞の中の `＠？シーン名`（`「表示名」` も付けられる）を、さくらスクリプトのアンカー `\_a` として出す。クリックで、`OnAnchorSelectEx` からそのシーンへ飛ぶ。選択肢の振り分けを共有し、LSP・VSCode の着色とマニュアル、`pasta_check validate` の飛び先の検査まで揃える（2026-10-08 起票）。版は未定。Dependencies: scene-condition-attributes（`call-attribute-filter` を 2026-10-29 に統合。機能の依存は無く、`act.lua`・VSCode の文法定義・検査の段を共有する順番だけ）
- [ ] novel-talk-flow -- ノベル向けの出力に切り替える設定。台詞ごとのクリック待ち、話者名と声と変わらない行 ID、ランダムトークと時報の停止、起動時の場面、「次の場面へ」、完全一致の場面選びの切り替え（2026-10-21 に仕組みを `scene-exact-match` へ移した。こちらは設定を入れるだけ）。Dependencies: novel-areka-contract, cue-command-runtime, scene-exact-match
- [ ] novel-checkpoint-rollback -- 台本に目印の ID を埋め、応答ごとに状態と台本を控える。areka が知らせた ID の時点へ戻す（変数の保存の範囲は 2026-10-17 に `novel-variable-scope` へ分けた）。Dependencies: novel-talk-flow, novel-variable-scope
- [ ] novel-save-slots -- スロットを選ぶセーブとロード。控えをスロットへ読み書きし、スロットの中身（日時・章の名前）を場面から読めるようにする。Dependencies: novel-checkpoint-rollback
- [ ] hello-novel-sample -- 見本のノベル。物語と、タイトル・セーブ・ロード・設定の場面一式とシェル。0.6.0 の合格を areka の実機で確かめる題材。Dependencies: cue-command-runtime, novel-talk-flow, novel-variable-scope, novel-checkpoint-rollback, novel-save-slots, shell-element-click, scene-condition-attributes（areka の側の実装も要る）
- [ ] manual-novel-guide -- マニュアルに、ノベルゲームの作り方の部を足す。見本のノベルを題材に、柱・ト書き・台詞・分岐・システム画面・配布の順に導く。Dependencies: hello-novel-sample
- [ ] scene-exact-match -- 完全一致でシーンを呼ぶ `＞＞シーン名`（同名が複数ならランダム。既存の `search_scene_exact` を使う）と、`pasta.toml` の `[scene] match = "exact"`（`＞`・イベントの受け口・OnTalk・時報・選択肢の飛び先を全部完全一致にする。既定は前方一致）を足す。既定のモードでも、イベントの受け口はイベント ID の直後が英大文字で続く候補（`＊OnInstallCompleteAll` など）を外す。`novel-talk-flow` の完全一致の切り替えを引き取る（2026-10-21 起票。ghost_dev のヒアリング。同日に開発者が承認して `event-scene-exact-match` から広げた）。版は未定。`call-attribute-filter` と同じウェーブに置かない。Dependencies: none
- [x] scene-pick-modes -- 同名シーンの選び方に、重み（`＊OnTalk　＆重み：5`。1 巡の中で重みの数だけ出る）と順番（`＆順番：番号`、別名 `index`。番号の小さい順に出して最初へ戻る。続きはシャッフルと同じくゴーストが終われば消える）を足す。単語の値にも重み（`＠挨拶：こんにちは＠３、やあ`）を足す。不正な値は読み込みエラー。今は複写と Lua で作っている（2026-10-21 起票。ghost_dev のヒアリング。同日に開発者と書き方を決めた）。0.5.0 の 3 番（2026-10-29 開発者の決定）。`＆重み`・`＆順番` だけを先に特別扱いで読まない（属性の早い部分実装にしない）。`scene-condition-attributes` と同じウェーブに置かない。Dependencies: scene-condition-attributes（候補の袋と控え。`scene-attribute-store` は入った）
- [ ] vsce-4-upgrade -- `@vscode/vsce` を 4.x に上げる。リリース CI の Node は `ponytail-manual-ci` が 22 にした。`--azure-credential` の公開の道を `release-setup-check.yml` で確かめる。だめなら 4.x の OIDC の口へ移すかを決める（2026-10-24 起票。`/update` から）。版は未定。`pasta-dll-multiarch`（`release.yml`）と同じウェーブに置かない（`editors/vscode/package.json` を共有した `vscode-mcp-client` は 2026-11-03 に入った）。Dependencies: none（`ponytail-manual-ci` は入った）
- [ ] pest-2-9-4-upgrade -- pest を 2.9.4 以降に上げても、手元と CI の 3 か所で `pasta_lsp` の WASM のビルドが通るようにする（psm が wasm32 に clang を要る）。wasm32 向けの C コンパイラの用意か、上流の直しかを決める（2026-10-24 起票。`/update` から）。版は未定。`Cargo.lock` の席を触る spec と同じウェーブに置かない（0.5.0 の `debug-mcp-runtime` は 2026-11-03 に入った）。Dependencies: none（ファイルの順番で後に置いた ponytail の 3 本は入った）
- [x] actor-line-newlines -- 同じ話し手が続く行（継続行 `　：` と、同じアクターを書き直した行）の前に、`[actor."名前"] line_newlines` の幅の改行 `\n[値×100]` を入れる（未設定なら今のまま）。トランスパイラが行の頭の印を出し、出力側が spot_newlines と同じ遅延方式で次の字の直前に 1 つ入れる。話し手の切り替え・選択肢の前後は今の区切りを優先、行の最初の字より前に改行・`\_l`・`\c` があれば入れない、前の行が改行で終われば入れない、Call をまたいでも出力の並びで見る、budoux の列を 1 つ進める。`[ghost]` の既定値と、改行の後に付ける字下げは範囲外（2026-11-05 起票。ghost_dev「窓際のぱすたさん」の縦書きの辞書をネタ帳と同じ 1 行 1 列の形にするため）。0.5.1（2026-11-05 開発者の決定「早く欲しい」）。`cue-command-runtime` と同じウェーブに置かない（行の頭の生成の場所を共有する）。Dependencies: none
- [x] vscode-scene-outline-play -- VSCode 拡張で、`.pasta` の見出しの行（グローバル・ローカル）の上に CodeLens「▶ シーンを実行」を出し、押せばそのシーンを再生する。アウトライン・パンくず・`Ctrl+Shift+O`・スティッキースクロールに グローバル ＞ ローカル の 2 階層を出す。シーンの並びは `pasta_lsp` の解析結果に足して WASM で渡し、DocumentSymbol と CodeLens はそれを写すだけ。`pasta.runSceneAtCursor` は引数（uri・行）でも呼べるようにし、行からシーンを引くのは今の `OnPastaSceneAt` のまま。独自の設定 `pasta.codeLens.enabled` は作らない（`"[pasta]": { "editor.codeLens": false }` で消せる）（2026-11-09 起票。開発者の依頼）。0.5.3 のウェーブ 1（2026-11-12 開発者の承認。`pasta-recipe` のツリーが `scenes` を使う）。`vsce-4-upgrade`（`editors/vscode/package.json`）・`scene-anchor-link`（`pasta_lsp` の解析）と同じウェーブに置かない。Dependencies: none
- [x] scene-play-locals -- `OnPastaScene`・`OnPastaSceneAt` の Reference2 に TOML のインライン表（`{ 名前 = "ぱすた", 回数 = 3 }`）を付けると、そのシーンがローカル変数に値が入った状態で始まる。付けなければ今のまま。誤りは今の `OnPastaSceneAt` と同じ形で知らせて鳴らさない。マニュアルの 2 つの節（2026-11-12 起票。`pasta-recipe` の discovery）。0.5.3 のウェーブ 1。`cue-command-runtime`・`novel-*`（`scene.lua`）と同じウェーブに置かない。Dependencies: none
- [ ] pasta-recipe -- VS Code を使わない pasta DSL のエディタ（仮名）。Tauri 2 ＋ Vite ＋ Monaco の Windows x64 の単一の exe。ゴーストのフォルダのドラッグドロップで `ghost/master` を開き、最近開いたものを TOML で残す。ファイルとシーンのツリー、`.pasta` の着色（TextMate ＋ セマンティックトークン）と診断、ゴースト全体の検査、コードブロックの Lua の着色と `.lua` の編集、保存。解析は Rust の側で `pasta_lsp`・`pasta_check` を直接呼ぶ。`editors/recipe/`（`src-tauri` は別の Cargo ワークスペース）。リリースの成果物 `pasta-recipe.zip` とマニュアル（2026-11-12 起票。開発者の依頼「最優先事項」）。0.5.3 のウェーブ 2。`pasta-dll-multiarch`・`vsce-4-upgrade`（`release.yml`）と同じウェーブに置かない。Dependencies: vscode-scene-outline-play
- [ ] pasta-recipe-dev-actions -- VS Code 拡張の実行の流れ（MCP クライアント・リロード・シーン実行・変数ダンプ）を `editors/shared/` へ移し、入出力（HTTP の POST・ファイルの読み取り・更新時刻）だけを Node と Tauri で差し込む。拡張の振る舞いは変えない。`pasta-recipe` に、シーンの ▶・リロード・出力の欄・保存変数の表（`OnPastaSetSaveVariable` で書き換え・削除。表は見るだけ）・ローカル変数を入れた実行を足す（2026-11-12 起票。`pasta-recipe` から分けた）。0.5.3 のウェーブ 3。Dependencies: pasta-recipe, scene-play-locals, vscode-scene-outline-play

## Existing Spec Updates

2026-10-10 の discovery で 0.5.0 の道筋に乗せた既存の spec（`scene-attribute-store`）は、2026-11-03 に入った。

2026-10-21 の discovery「PastaDSL機能ヒアリング」（ghost_dev の「emo2 開発」「ぱすた開発」セッションへの聞き取り）で、次の brief に「申し送り（2026-10-21 の discovery「PastaDSL機能ヒアリング」より）」の節を足した。

- [ ] cue-command-runtime -- 外見として記録させない生のさくらスクリプト（今は `act:raw_script`）の実例。`！命令` の出力を外見に記録するかを要件で決める。Dependencies: none（`scene-attribute-store` は入った）

## Direct Implementation Candidates

- なし（最後の 1 件は、2026-10-29 に `scene-condition-attributes` へ取り込んだ）。


## バックログ（brief なし・保留）

動機（作者が困った事例・計測した性能問題）が生じたら、その動機から brief を起票する。

### 文法の将来項目（旧文法仕様の申し送り）

- シーンのパラメータ（R1）— シーン宣言での名前付きパラメータ。Call の位置引数とシーン引数 `＄０`… で代替できる。
- アクタースコープ内コードブロックの用途（R2）— アクター固有のイベントハンドラ・状態管理関数。現行は `lua` ブロックの値・関数がアクター付きの単語参照から使えるだけ。
- Call の戻り値と変数代入（R3）— `＄x＝＞シーン`。DSL では定義せず、ランタイム設計の領域。`＄x＝＠＊関数（）` で代替できる。
- さくらスクリプトの `\]`・`\%`（R4）— 角括弧内の `]` と `\%` を DSL から書けるようにするか。現行は `\]` を特別扱いせず、`\%` はパースエラー。
- 継続行内の空行の糖衣構文（R6）— 行継続の途中の空行を改行として出力する。現行は何も出力しない。
- 単語参照への属性フィルター（`＠単語名＆category＝food`）— 単語に属性を付ける構文が無い。`scene-condition-attributes` の後に検討する。
- 既定の別名の追加（`時報 → OnHour` など。2026-10-10 棚卸、`scene-name-alias` の範囲外）— 既定は `会話 → OnTalk` の 1 件だけにした。

### トランスパイラ最適化の候補

いずれも未実装で、計測された性能問題は無い。現行の生成時最適化はマニュアルの [内部設計: トランスパイルパイプライン](https://ekicyou.github.io/pasta/internals/transpiler.html#生成時最適化) が書く。

- 定数畳み込み — トランスパイル時に定数式を評価する。
- デッドコード削除 — 到達不能コードを生成しない。
- インライン展開 — 小さなローカルシーンを呼び出し元に展開する（ソースマップ・デバッグの忠実さと衝突する）。
- 単語プリフェッチ — 単語辞書を先読みする。

### その他

- `pasta_novel` アダプタ（ノベルゲーム宿主）— ノベルゲーム（0.6.0。2026-10-29 に 0.5.0 から順延）は、SHIORI とさくらスクリプトのまま作ると決めた（2026-10-10。上の「Phase 13」）。SHIORI を通さない宿主が必要になったら、その動機から起票する。Phase 7 の宿主非依存コアと presentation event stream 契約が土台になる。
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
- マニュアルのメニューのテストが持つ mdBook の写し（2026-10-10 棚卸、`manual-claudia-theme` の実装メモ）— `book/tools/theme-menu-test.mjs` は、mdBook 0.5.4 の `book.js` のキー操作の処理を写して持つ。mdBook の版を上げるときに合わせる（CI も 0.5.4。`ponytail-manual-ci` が 2026-11-03 に揃えた）。
- 配布版の hello-pasta に会話の変化を足すときの指針（2026-10-10 棚卸、`hello-pasta-tutorial-stages` の実装メモ）— emo2 開発からの知見「1 つの場面に 3 通り」は、段階の形を固定した辞書では満たせない。配布版に変化を足すことになったら、その動機から起票する。
- pasta.dll が C の実行時ライブラリの入出力の関数を約 170 個書き出している（2026-10-22 棚卸、`lua-cmodule-robustness` の調査の範囲外）— LuaJIT の `msvcbuild.bat` の組み立ての設定（`_CRT_STDIO_INLINE` を `dllexport` にする）から来る。pasta のコードではなく、LuaJIT の組み立ての側にある。害は見つかっていない。ほかの DLL とぶつかる事例が出たら起票する。
- budoux の自動改行の禁則（2026-10-05、ghost_dev「emo2 開発」からの申し送り）— `line_breaker.rs` の `break_lines_impl` は禁則を見ない。BudouX の語の区切りの直前で改行するだけである。
  - 方針は JIS X 4051 どおりとする。リーダー（`‥…`）は行頭に置いてよい。並びの途中では分けない。
  - この方針では、申し送りの実例「イイジャン！／‥‥ええと、」は正しい組版になる。
  - 句読点・`！？`・閉じ括弧の前、開き括弧の後ろでは、BudouX の既定の日本語モデルは区切らない（手元で確認）。
  - BudouX が行頭禁則の字の直前（または行末禁則の字の直後）で区切る実例が出たら、その実例から起票する。その場合は、既存の `[talk]` の `chars_*` を使った追い出しを第一候補とする。
