# Roadmap

## 概要

pasta は、日本語 DSL（Pasta DSL）で書いた辞書を Lua へトランスパイルし、組込 LuaJIT で実行する「伺か」の SHIORI（`pasta.dll`）と、その周辺ツール（`pasta_check`・LSP・VSCode 拡張・利用者マニュアル）からなる。Phase 1〜11 で、プロパティアクセス・監査・マニュアル・ソースレベルデバッグ・アクターモデル駆動・配布物の形・現行実装の不具合の一掃までを完了した。2026-10-09 までに、リリースの CI 化、シーン名の別名、式の中の値なしの扱い、マニュアルの着せ替え、段階ごとに学べる hello-pasta の辞書も入った（下の「完了フェーズ」）。2026-10-11 までに、hello-pasta の新しいシェル、9 段目の選択肢の表示の直し、マニュアルの見出しへのリンクの検査も入った。2026-10-15 に、リリースの本文を見どころの紹介にした。2026-10-17 までに、撫で・つつきの反応のショートカット、実行時の失敗の出力の一本化、SSP を起動しない辞書の検査、シェルの説明の章、印刷用のページの廃止、結合テストの古いランタイムの写しの撤去も入った。同じ日に、止まる実行時エラーのバルーン表示と入門ガイドの絵 16 枚が入り、v0.4.0（主題「入門者にやさしく」）を出した。2026-10-18 に、辞書の検査の端のケース、入門ガイドと SSP のずれ（SSP の仮想の時刻を時報に効かせる）、窓際のぱすたさん（ghost_dev）からの要望の 2 本（アクターごとの表示の装い・縦書きのバルーンの列の扱い）が入り、v0.4.1 を出した。

現在の主題は 4 つある。

- **0.4.2: 致命的でない不具合**。今の範囲は、ゴーストから DLL の関数を LuaJIT の FFI（`ffi.load`）で、深いパス・ANSI に無い文字を含むフォルダでも呼べるようにする `lua-cmodule-robustness`（完了）と、`pasta_check` を GitHub Release から単体で落とせるようにする `pasta-check-release-asset`（2026-10-19 起票）の 2 本である（2026-10-18・2026-10-19 開発者の決定。`lua-cmodule-robustness` は、要件の話し合いで C モジュールの `require` の道から FFI の道に替えた）。0.4.1 の後に届いた、組み合わせの絵文字が自動ウェイトで割れる不具合（`talk-grapheme-clusters`）は、版がまだ決まっていない（推奨は 0.4.2。下の「0.4.2 の範囲」）。
- **0.4.3: ponytail 監査の指摘の修正**。2026-10-19 に、プラグインの `/ponytail:ponytail-audit` をクレートと領域ごとに 8 つ並べて走らせた。その指摘を、ソースを共有しない 10 本の spec（`ponytail-*`）に切った。0.4.2 を挟んで、0.4.3 で着手する（2026-10-19 開発者の決定）。下の「0.4.3 の範囲」。
- **Phase 13: areka でノベルゲーム（0.5.0）**。2026-10-10 の discovery で、合格ライン・通り道・台本の書き方・巻き戻しの方式を決め、spec を起こした（2026-10-17 の時点で 11 本）。属性セマンティクス（シーン属性の保持と読み出し・Call の属性フィルター）も、この道筋に入った。
- **Phase 11 の残り**: 文中のシーンリンク（`scene-anchor-link`）。どの版に入れるかは、まだ決めていない（下の棚卸の「開発者の判断が要るもの」）。

Phase 12（初心者向けの入門ガイド）は、2026-10-17 の入門ガイドの絵で完了した（「完了フェーズ」）。

このほか、リリース手順（spec `release-workflow`）の CI に合わせた書き換えと、CI での初回のリリース（v0.3.8）は済んだ（2026-10-10）。リリース手順は `release` スキル（`/release`）へ移し、spec `release-workflow` は完了にした（2026-10-11）。

**優先順位（開発者の方針）**: 0.4.0（主題「入門者にやさしく」）は 2026-10-17 に、0.4.1（致命的でない不具合と、窓際のぱすたさんからの要望の 2 本）は 2026-10-18 に出した。次の節目は 0.4.2（致命的でない不具合。2026-10-18）、その次が 0.4.3（ponytail 監査の指摘の修正。2026-10-19）で、その次の 0.5.0 の目標は、areka でノベルゲームが作れるようになること（2026-10-10）。ウェーブは、今のゴールの範囲の spec だけで組む（2026-10-16）。致命的な不具合は、見つかりしだい割り込ませる。致命的でない不具合は、次の 0.4.x に入れるかを開発者が決める（2026-10-17 に 0.4.1 へ回した方針の続き。2026-10-18 に `lua-cmodule-robustness` を 0.4.2 に入れた）。

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
- **マニュアルの章の数**: 章を足す spec は、章の数を決め打ちする 3 つの検査（`book/tools/verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`）を合わせる（2026-10-18 の時点で 63 章）。
- **辞書の検査の順**: 飛び先などの検査（`pasta_check` の `validate/references.rs`）は、読めない辞書が無いとき（`skip_reasons` が空のとき）だけ走る。検査を足す spec は、この順を保つ。
- **アクターの装い**: アクターごとの表示の装いは pasta.toml の `[actor."名前"].font` に書き、タグ列（`\f[…]`・`\f[default]`）は `pasta_core` の `ActorFont` が作る。出し入れは `pasta/shiori/actor_style.lua` が受け持つ。1 つのスポットを複数のアクターで共有できる。
- **SSP の仮想の時刻**: `pasta/shiori/clock.lua` が受け持つ。SSP 2.9.18 の TimeMachine は pasta.dll の読む OS の時計まで仮想にするので、時刻の飛びで次の正時を決め直し、間隔は飛びを差し引いて数える。
- **budoux の列**: budoux は、列を動かすタグ（`\n`・`\n[…]`・`\c`・`\_l[…]`）の所で数え直し、列の字数を吹き出しごとに持ち越す（1 つの応答の中だけ）。

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
| 2 | pasta-check-release-asset | Opus | 8 前後 | 今すぐ（2026-10-19 起票） |
| 3 | 0.4.2 のリリース（`release` スキル） | Opus | — | 上の全部の後 |

- **0.4.2 に入れたもの**: `lua-cmodule-robustness`（`lua-require-robustness` の続き）。2026-10-18 の要件の話し合いで、C モジュールの `require` の道（Lua の C API を pasta.dll から書き出す必要があり、pasta のゴーストが 2 体いると別の pasta.dll に結びつく危険があった）をやめた。代わりに、pasta.toml の `[loader] ffi = true`（既定は無効）で FFI を開き、`ffi.load` を包んで、深いパス・ANSI に無い文字を含むフォルダ・隣の依存 DLL でも DLL を読めるようにする。経緯と実測は spec の brief にある。
- **0.4.2 に足したもの（2026-10-19 開発者の決定）**: `pasta-check-release-asset`。Rust を入れていない作者が `pasta_check` を使えるように、x64 の `pasta_check.exe` とライセンス表示を `pasta_check.zip` にして、4 つ目の配布物として GitHub Release に載せる。作るのは `release.ps1` の新しい段で、手元の `release.bat` とリリース CI が同じ一式を作る。`cargo install` の道は残す。署名・x86 版・インストーラーは作らない。0.4.3 の `ponytail-manual-ci` と `release.yml`・`github-release.ps1` を共有するので、こちらが先に入り、`ponytail-manual-ci` が後から合わせる。
- **版がまだ決まっていないもの**: `talk-grapheme-clusters`（推奨は 0.4.2。0.4.1 の後に届いた不具合で、致命的でない）。入れるなら、`lua-cmodule-robustness` と同じウェーブに並べられる（ソースを共有しない。ルートの `Cargo.toml` の席は `talk-grapheme-clusters` が持つ。`lua-cmodule-robustness` はテスト用の DLL を作るために `cc` を `pasta_lua` の dev の依存に足すので、`Cargo.lock` の `pasta_lua` の依存の行が 1 つ変わる。`Cargo.lock`・`crates/pasta_lua/Cargo.toml`・`tech.md` の依存の表は、後から入る側が作り直して合わせる）。`scene-anchor-link` も版が決まっていない（下の棚卸）。
- **これから見つかる不具合**: 致命的（起動しない・辞書の内容が失われる・配布物が壊れる・リリースが通らない）なら、すぐに割り込ませる。それ以外は、0.4.2 に入れるかを開発者が決める。
- **0.5.0 との関係**: 0.5.0 の先頭（`novel-areka-contract`・`scene-attribute-store`・`novel-variable-scope`）は、0.4.2 の spec とファイルが重ならない。並べるには、0.5.0 を今のゴールに含める開発者の決定が要る（下の棚卸の「開発者の判断が要るもの」）。

### 0.4.1 の後に届いた不具合（2026-10-18 起票）

v0.4.1 を出した後に、「ぱすた開発」セッション（窓際のぱすたさん）が実機で見つけて知らせてきた 2 件。どちらも致命的でない。spec にしたのは 1 本で、版は開発者が決める（推奨は 0.4.2）。もう 1 件は、表示の誤りが無いと分かり、取り下げた。

- **`talk-grapheme-clusters`（spec）**: 自動ウェイトと budoux の改行が、コードポイントごとに台詞を切る。組み合わせの絵文字（`🙆‍♀️`・`✌️`）の途中に `\_w` が入り、縦書きのバルーンで絵文字が割れて描かれる。`chars_*` の組み合わせの絵文字も一致しない。根は 2 か所にある。`sakura_script/tokenizer.rs` の 1 コードポイントずつの切り出しと、`line_breaker.rs` が自分で持つ切り出し（`tokenize_plain_chars`）である。`wait_inserter.rs` も合わせて、書記素クラスタ単位にする。推奨は `unicode-segmentation` を足すこと（まだ `Cargo.lock` に無い）。`CharSets` と `classify(char)` は `pasta_lua` クレートの公開の API なので、残すかを設計で決める。`sakura_builder.lua` は触らない見込み。
- **着せ替えの復旧が前のパーツを出す件（取り下げ）**: 起票したが（`dressup-restore-stale-parts`）、同じ日に取り下げた（2026-10-18 開発者の決定「ぱすた側が必要としていないなら取り下げ」）。
  - ログの調べ直しで、表示が壊れた実例は無いと分かった。腕が残る症状は推測だったと、報告の側が訂正した。
  - ゴースト側は、復旧の出力が短く済むので、`,0` の明示の OFF を残す。
  - 残る粗さは、バックログの「着せ替えの復旧の出力の膨らみ」に書いた。

### 0.5.0 の目標（2026-10-10 開発者の決定）

別リポジトリで開発中の areka で、ノベルゲームが作れるようになること。pasta の側に何が要るかは、2026-10-10 の discovery で洗い出した。合格ライン・決めたこと・spec の割り方と順番は、下の「Phase 13: areka でノベルゲーム（0.5.0）」にある。

## 0.4.3 の範囲（2026-10-19 開発者の決定）

2026-10-19 に、プラグインの `/ponytail:ponytail-audit` を、クレートと領域ごとに 8 つ並べて走らせた。見たのは、正しさ・安全・負荷・テスト・速さ・無駄の順。指摘は約 100 件で、そのうち Must fix は 9 件だった。それを、ソースを共有しない 10 本に切った。0.4.2 を挟んで、0.4.3 で着手する。

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
  - `talk-grapheme-clusters`（`sakura_script/`）とは、ファイルが重ならない。
  - 0.5.0 の 3 本（`novel-areka-contract`・`scene-attribute-store`・`novel-variable-scope`）とも、ファイルが重ならない。
  - `review-improvement-loop`（常駐）が次に回るときは、この 10 本が入った後のコードを基準にする。
- **人手の確認**
  - `ponytail-check-release` は、`.claude/skills/pasta-check/SKILL.md` を直す。分類器が止めるので、その場面で開発者の許可を求める。
  - `ponytail-shiori` の unload の競りと `Key:` の行は、SSP の実機で確かめられれば確かめる。

開始のときの形（0.4.3 で着手するとき。今はウェーブに入れない。ゴールが 0.4.2 のため）:

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

## 棚卸（2026-10-18・main `facd6cf4`）

前回（2026-10-17・main `b4164100`）の後に、0.4.1 の 4 本（辞書の検査の端のケース `pasta-check-validate-edges`・入門ガイドと SSP のずれ `tutorial-ssp-gaps`・アクターごとの表示の装い `actor-display-style`・縦書きのバルーンの列の扱い `vertical-balloon-lines`）が入り、v0.4.1 を出した（リリース CI の run 39123004799。全部の job が 1 回目で通った）。ほかに、シーンのキックの E2E テストの揺れの直しが入り、`lua-cmodule-robustness`（0.4.2）と `talk-grapheme-clusters`（版は未定）を起票した。未完了は 14 本（常駐の `review-improvement-loop` を除く）、完了は 175 本。作業中のブランチは無い（ほかのワークツリーとリモートのブランチは、どれも main に入った後のもの）。前回の記録は git の履歴にある。

今のゴールは 0.4.2 である（0.4.1 を出したので、いちばん近い版）。範囲は `lua-cmodule-robustness` の 1 本。測り直したのは 9 本で、ゴールの中の 1 本、版の決まっていない `talk-grapheme-clusters`、入った 4 本がファイルか前提を動かしたゴールの後の 7 本（`novel-areka-contract`・`cue-command-runtime`・`novel-talk-flow`・`novel-checkpoint-rollback`・`call-attribute-filter`・`scene-anchor-link`・`shell-element-click`）である。前提の動いていない 5 本（`scene-attribute-store`・`novel-variable-scope`・`novel-save-slots`・`hello-novel-sample`・`manual-novel-guide`）は測り直していない。

- `lua-cmodule-robustness`（10〜13）に、brief に無い大きな穴が見つかった。C のモジュールは普通 `lua51.dll` から Lua の C の API を得るが、pasta.dll は LuaJIT を静的に取り込んでいて `lua51.dll` が無い。そのままでは読み込みが失敗するか、別の `lua51.dll` があると 2 つ目の VM が pasta の状態を触る。要件定義の最初に、渡し方を決める（上の「0.4.2 の範囲」）。テスト用の DLL の作り方も、`Cargo.lock` と CI の手間を決める。
- `talk-grapheme-clusters`（8〜11）は、根が 2 つある（上の「0.4.1 の後に届いた不具合」）。`novel-talk-flow` とは `wait_inserter.rs`・`line_breaker.rs` を共有するので、brief の「同じウェーブに置ける」は誤りだった（再測定の節に書いた）。`line_breaker.rs` は 1,134 行あるが、本体は約 290 行で、残りはテストである。本体を分ける必要は無い。
- `actor-display-style` が、1 つのスポットを複数のアクターで共有する形を正式にした。`novel-talk-flow` の話者名がいっそう要り、差し込む場所（`emit_actor_switch`）もできた。`novel-areka-contract` の「文字の色・大きさ」は、areka に SSP の `\f[…]` を解釈してもらう依頼になる。
- 「出力の組み立ては `\x` を知らない」は半分古くなった。装いの出し入れ（`actor_style.lua`）は `\x` を知っている。段落の区切り・列の持ち越し・外見の復旧・budoux は、まだ知らない（`novel-talk-flow`）。
- `tutorial-ssp-gaps` が入れた時刻の問い合わせ（`clock.lua`）は、暇な `OnSecondChange` の応答に載る。巻き戻しの区切りにしてはいけない応答が 1 つ増えた（`novel-talk-flow`・`novel-checkpoint-rollback` の再測定の節）。
- `cue-command-runtime` は 17〜21 になった（別名表を `pasta_check validate` でも確かめる分。`[actor."名前"].font` が同じ形の前例を作った）。超えるならト書きと柱に分け直す案は、そのまま。
- `novel-talk-flow`（17〜21）を分けるなら、場面の流れの分は Opus でよい（行の ID を持たず、問いが 2 案に絞れて推奨がある）。
- `scene-anchor-link`（18〜20）は、表示名を Lua の側で台詞として通してから `\_a` で包む案なら、Rust の `sakura_script/` を触らずに済み、`talk-grapheme-clusters` と並べられる。

### 即時修正（spec なし・本棚卸で実施して閉じた）

- `steering/product.md` — 完了の数（175）・現行の版（v0.4.1）・現在地と次の版（0.4.2）。
- `steering/tech.md` — zip の版（8.6 → 9.0）。
- `steering/structure.md` — `pasta_core` の `actor_font.rs` と公開 API の `ActorFont`、ローダー設定の `actor_font.rs`、`pasta_lua` のテストの木（`shiori/` 7 本・`sakura_script/` 6 本）、1,000 行を超えるファイル（`line_breaker.rs` 1,134 行・`balloon.rs` 1,070 行。どちらも後半はテスト）。
- コードのコメント — `pasta/shiori/sakura_builder.lua` の `emit_actor_switch` の説明（装いのタグも出す）、`pasta/shiori/event/virtual_dispatcher.lua` の require の説明（`CLOCK` だけは先に読む）。
- brief — `talk-grapheme-clusters` の改行コードを CRLF にそろえた。`call-attribute-filter` の申し送りの 6 行の改行コードも CRLF にそろえた。各 brief の再測定の節に、古くなった行番号と、同じウェーブに置けるかの誤りを書き出した。
- この roadmap — 完了した 4 本を「完了フェーズ」へ畳み、0.4.1 の範囲の節・窓際のぱすたさんの節・作成支援の節・入門ガイドの実機確認の節（どれも完了）を消して、0.4.2 の範囲の表にした。前提に 4 行（辞書の検査の順・アクターの装い・SSP の仮想の時刻・budoux の列）を足した。「辞書の検査に検査を足す spec を `pasta-check-validate-edges` と同じウェーブに置かない」の決まりは、その spec が入ったので、検査のファイルを共有する組の決まりに書き換えた。
- 確かめて閉じたもの: `pasta-check-bundled-balloon` を v0.3.7 で公開したことの emo2 開発セッションへの連絡（人手の確認）は、リポジトリから確かめられず、その後 v0.4.0・v0.4.1 を出して古くなったので閉じた。直接実装の候補だった areka の `doc/PASTA_PROFILE.md` §5 の作例は、`novel-areka-contract` の brief が依頼に含めているので、重複として消した。マニュアルの章の数は 63 のままで、3 つの検査と合っている。

### 統合・分割・改名

- なし。どの spec も 20 タスクを大きく超えない。`cue-command-runtime`（17〜21）と `novel-talk-flow`（17〜21）は、超えたときの分け方の案を brief に持つ。`call-attribute-filter`（19〜21）は 2026-10-04 に一度分けたので、わずかに超えるだけでは分けない。

### 起票と、バックログに残したもの

- 起票: なし（この棚卸の前に、同じ日に `lua-cmodule-robustness` と `talk-grapheme-clusters` を起票してある）。
- バックログに、入った spec の範囲外から 3 行足した（`pasta_check validate` が `[talk]`・`[ghost]` の budoux のキーを見ない件、縦書きの列の扱いの残り、スリープの印が残る件）。「タグの読み取りの制約」は、囲みとエスケープを 1 つのタグとして扱うようになった分を書き直した。
- 残したものは全部そのまま（どれも、動機の事例・使う人・計測が無い）。

### 要件定義のモデル

今のゴール（0.4.2）の `lua-cmodule-robustness` は Opus。いちばん Fable に近いのは、この spec の C の API の渡し方である（pasta.dll から書き出すなら、C のモジュールが上に乗る取り決めになる）。ただ、案は 3 つに絞れていて、どれも Rust の側だけで決まる。Ukagaka の正典の話でも、Rust・Lua・SHIORI をまたぐ時の流れの話でもない。版の決まっていない `talk-grapheme-clusters` も Opus（切り方は書記素クラスタと決まっていて、残る問いは公開の API を残すかどうかの 2 案）。ゴールの後では、`scene-attribute-store`・`novel-variable-scope`・`cue-command-runtime`・`call-attribute-filter`・`novel-talk-flow`（分けるなら出力の分だけ）・`novel-checkpoint-rollback` が Fable である（理由は各 brief の再測定の節）。

### ウェーブ（2026-10-18）

今のゴール（0.4.2）の 1 本を並べる。先に要るものが無く、ほかの未完了の spec とソースを共有しない。

| spec | 子孫 | 種別 | 規模 | 要件定義 |
| ---- | ---- | ---- | ---- | -------- |
| lua-cmodule-robustness | 0（後は 0.4.2 のリリース） | 機能（C の拡張モジュールが読めない。開発者の決定で 0.4.2） | 10〜13 | Opus |

開始コマンド（`/kiro-next` の報告と同じ形。俯瞰の頁が、この塊をコピーのボタンつきで出す）:

```
Wave 1本
Opus
/kiro-start lua-cmodule-robustness
```

同じウェーブに置くときの約束:

- `lua-cmodule-robustness` が持つのは、`crates/pasta_lua/src/runtime/searcher.rs`（622 行。足す分は新しいファイルに分ける）・`runtime/module_registry.rs`、`loader/context.rs`（既定の `cpath` をそこで組むなら）、新しい Windows 専用の C のローダーのファイル、`crates/pasta_lua/Cargo.toml`（`windows-sys` の feature）、`tech.md`、`tests/runtime/` のテスト、マニュアルの `internals/loader.md`・`reference/startup.md` とスキルの references。`sakura_script/` は触らない。
- `talk-grapheme-clusters` を 0.4.2 に入れて並べるなら、`Cargo.lock` とルートの `Cargo.toml` は `talk-grapheme-clusters` が持つ。`lua-cmodule-robustness` は、テスト用の DLL をワークスペースの新しいクレートにしない（`Cargo.lock` が動くため）。`crates/pasta_lua/Cargo.toml` と `tech.md` は別の行なので、後から入る側が合わせる。
- 章を足さない見込み。足すなら、3 つの数（`verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`）を 64 に合わせる。

次（今のゴールの中）: 0.4.2 のリリース（`/release 0.4.2`）。

ゴールの後（0.5.0 の順）: `novel-areka-contract`（先頭・子孫 約 6）、`scene-attribute-store`（先頭・子孫 約 8）、`novel-variable-scope`（先頭・子孫 約 4）。版の決まっていないもの: `talk-grapheme-clusters`（先頭・推奨 0.4.2）、`scene-anchor-link`（`call-attribute-filter` の後）。

### 開発者の判断が要るもの（ウェーブは止めない）

- **`talk-grapheme-clusters` の版**。推奨は 0.4.2（致命的でない不具合で、0.4.1 に回した方針の続き）。入れるなら、ウェーブに足して 2 本（どちらも Opus）にできる。
- **0.5.0 を今のゴールに含めるか**（0.4.2 と並べるか）。推奨は含めること。areka の側が早く動けるよう、`novel-areka-contract` を先に出す方針（2026-10-10）と合う。含めるなら、ウェーブに `novel-areka-contract`（Opus）・`scene-attribute-store`（Fable）・`novel-variable-scope`（Fable）を足せる（`scene-attribute-store` が `partial.rs` と VSCode の文法定義を触らない約束で）。
- **`scene-anchor-link` の版**。推奨は 0.5.0 の後（新しい文法で、ノベルの合格に要らず、文法・`act.lua`・出力の組み立てを 0.5.0 の 5 本と共有する）。
- `pasta_shiori` の `string_to_multibyte`（テスト専用）を直すか消すか。直すなら、`util/hglobal/windows_api.rs` に、65001 ならバイト列をそのまま返す 1 行を足すだけ（`pasta_lua` の `encoding/windows.rs` と同じ）。消すなら、`EncoderCodePage::to_bytes` と 2 本のテストも消す。どちらも小さいので、次の `review-improvement-loop` を待たずに決めてよい。→ 2026-10-19 に、0.4.3 の `ponytail-shiori` で閉じることにした（コードページの層ごと、テストだけの足場を消す）。
- `SOUL.md` §2・§5 の「Wait/Sync はマーカーのみ」「全出力は yield」— 構想の文書なので、直すかどうかは開発者が決める。
- `steering/structure.md` の作り直し（`pasta_shiori` の木と、`pasta_scripts/pasta/` の木が無い。`failure.lua`・`shiori/stop.lua`・`shiori/actor_style.lua`・`shiori/clock.lua` も載っていない。`pasta_lua` のテストの木もほかの所が古い）— `/kiro-steering` で直す。
- ghost_dev の emo2 開発のフォルダに、入門の撮影のときの `profile/` の保存データとログが残っている（`getting-started-screenshots` の実装メモ）。リポジトリの外なので、片付けるかは開発者が決める。

### 人手の確認が残っている項目

- リリースの失敗した job の再実行の読み方は、まだ確かめていない（v0.4.1 も 1 回目で通った。下の「リリース」）。

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
| 版は未定 | scene-anchor-link | 機能 | `call-attribute-filter` と同じ文法・生成・`act.lua`・エディタのファイルに加え、`sakura_builder.lua`、`shiori/event/choice_select.lua`、`sakura_script/`（表示名を Lua の側で台詞として通してから `\_a` で包む案なら触らない。触るなら `talk-grapheme-clusters` と同じウェーブに置かない）、VSCode 拡張の `wordRefDecorator.ts`、`pasta_check validate`（`references.rs`） |

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
  - 出力の組み立て（`pasta_scripts/pasta/shiori/sakura_builder.lua`）は、`cue-command-runtime`・`novel-talk-flow`・`novel-checkpoint-rollback` が触る。`scene-anchor-link` も触る。540 行あり、3 本が足すと 720〜850 行になる。`BUILDER.build` は込み入っているので、クリック待ちや `\x` の処理は、`actor_style.lua` のように別のモジュールに分ける。
  - 台詞の切り出し・自動ウェイト・budoux（`sakura_script/` の `tokenizer.rs`・`wait_inserter.rs`・`line_breaker.rs`）は、`talk-grapheme-clusters` と `novel-talk-flow` が触る。`scene-anchor-link` も、Rust の側を触るなら加わる。
  - 選択肢の振り分け（`shiori/event/choice_select.lua`）は、`shell-element-click` と `scene-anchor-link` が共有する。先に入る側が、振り分けを関数として取り出す。
  - 場面の関数を出すコード生成（`code_gen/scope_gen.rs`）は、`scene-attribute-store`・`cue-command-runtime`（2026-10-17 に `scene-stage-attributes` をまとめた）が触る。この順に置く。場面の関数の頭は `act:init_scene(SCENE, 書いたグローバル名, 書いたローカル名, 行の表)` で、行の表は最後の引数である（`runtime-error-balloon` が入れた。`transpiler.rs` の `fill_line_maps` が後から差し込む）。引数を足す spec は、この形を保つ。
  - `act.lua` は 829 行あり、1,000 行に近い。`cue-command-runtime`・`call-attribute-filter`・`scene-anchor-link`・`novel-talk-flow` が足すので、足す側が分け方を考える。ト書きの新しい入口は、別のファイルに置く。
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
| 12 | 0.5.0 のリリース（`release` スキル） | Opus | — | 上の全部 |

並べられる組み合わせ（2026-10-17 と 2026-10-18 の棚卸で測った）: `novel-areka-contract`・`scene-attribute-store`・`novel-variable-scope` は、0.4.2 の `lua-cmodule-robustness`・`talk-grapheme-clusters` とソースを共有しない。`novel-areka-contract` と `scene-attribute-store` は、互いにも共有しない。`shell-element-click` は `novel-areka-contract` の後、`scene-attribute-store` と並べられる（`scene.lua` を触らない約束で）。`novel-variable-scope` は `scene-attribute-store` と並べられる（`scene-attribute-store` が `partial.rs` と VSCode の文法定義を触らない約束で）。`cue-command-runtime` → `novel-talk-flow` → `novel-checkpoint-rollback` は、`act.lua` と `sakura_builder.lua` を共有するので 1 本ずつ入れる。

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

- [x] lua-cmodule-robustness -- ゴーストから C の DLL を LuaJIT の FFI で、深いパス・ANSI に無い文字を含むフォルダでも呼べるようにする。pasta.toml の `[loader] ffi = true`（既定は無効）で FFI を開き、`ffi.load` を包んで、ゴーストのフォルダを探し・`LoadLibraryExW` で先に読み・隣の依存 DLL を解き・理由を UTF-8 にする。LuaJIT は改造しない（`lua-require-robustness` の続き。2026-10-18 起票。要件の話し合いで C モジュールの `require` の道から替えた）。0.4.2 に入れる（2026-10-18 開発者の決定）。Dependencies: none
- [ ] pasta-check-release-asset -- `pasta_check` を GitHub Release から単体で落とせるようにする。`release.ps1` に段を足し、x64（crt-static）の `pasta_check.exe` と pasta_check の依存だけのライセンス表示を `pasta_check.zip` にする。リリース CI の build job に x64 のターゲット・中身の検査・upload を足し、`github-release.ps1` の添付を 5 つにする。入手の手順は Release を第一の道にし、`cargo install` は残す（2026-10-19 起票）。0.4.2 に入れる（2026-10-19 開発者の決定）。Dependencies: none
- [ ] talk-grapheme-clusters -- 自動ウェイト・`chars_*` の判定・budoux の改行を、コードポイントでなく書記素クラスタ単位にする。組み合わせの絵文字の途中に `\_w` や改行を入れない（「ぱすた開発」セッションの実機の報告。2026-10-18 起票）。根は `sakura_script/tokenizer.rs` と `line_breaker.rs` の切り出し（`tokenize_plain_chars`）の 2 か所にある。版は未定（推奨 0.4.2）。`wait_inserter.rs`・`line_breaker.rs` を共有する `novel-talk-flow` と同じウェーブに置かない。Dependencies: none
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
- [ ] ponytail-dsl-parser -- 解析を辞書の大きさに比例させる（span の行と桁を pest の行の索引で取る）。大きな数の字句を、どこでも同じ規則（i64、だめなら f64）にする。深い括弧の入れ子を構文の誤りにする。作られない誤りの型と `PastaFile.span` を消す。0.4.3（2026-10-19 起票。ponytail 監査）。Dependencies: none
- [ ] ponytail-core-registry -- シーン表・単語表の死んだ API と誤りの型を消し、`fast_radix_trie` を `BTreeMap` に替える。属性の絞り込みは残す（0.5.0 が使う）。0.4.3。Dependencies: none
- [ ] ponytail-lua-loader -- 読めないセーブを `.broken` に退避する。キャッシュを日付の一致で判定する。同じモジュール名の辞書を読み込みのエラーにする。finalize 前の検索を一定にする。パスの `;`・`?` を断る。`presentation/`・`RendererInjection`・テストだけの構築の道を消す。0.4.3。Dependencies: none
- [ ] ponytail-lua-transpiler -- Lua ブロックの中身を書いたとおりに出す（字下げと空行）。効かない `LineEnding`・`comment_mode` と、死んだ文脈の状態を消す（ファイル属性は残す）。0.4.3。Dependencies: none
- [ ] ponytail-lua-debug -- Lua 表示でも `.pasta` のブレークポイントを効かせる。ステップがコルーチンや切断で抜けなくなるのを直す。数の表示と、知らない DAP 要求への応答（unsupported）を直す。行ごとの費用を下げ、死んだコードを消す。0.4.3。Dependencies: none
- [ ] ponytail-lua-scripts -- 数でない `hour_margin` を既定に戻す。応答の見出しの値の改行を空白にする。続きのトークの期限を呼んだ時刻から数える。包む関数の写しと `co_callback` を消す（`pasta.areka` は残す）。0.4.3。Dependencies: none
- [ ] ponytail-shiori -- unload が、アクターのスレッドが DLL を出るまで（上限つきで）待つ。フォルダが無いときに理由を出す。`Key:` の行を受け付ける。FFI の型を C に合わせ、テストの競りを無くす。テストだけのコードとデータを出荷物から外す。0.4.3。Dependencies: none
- [ ] ponytail-check-release -- `release` の `--target`・`--release`・`--copy`・`--nar` の包含の関係を、ファイルに触れる前に断る。読み取りの誤りを、起きた段で報告する。働かない守りとテストの写しを消す。0.4.3。Dependencies: none
- [ ] ponytail-editor-tooling -- 構文の誤りを本当の位置に 1 件だけ出し、部分解析がシーンを落とさないようにする。箱・行末コメント・Lua のフェンスの色を文法に合わせる。`pasta_lsp` を WASM の解析器だけにする（tower-lsp を撤去）。`host` を廃止する。0.4.3。Dependencies: none
- [ ] ponytail-manual-ci -- マニュアルの 3 つの検査を、全部の push・PR とリリースの関門で走らせる。Pages の権限を deploy に絞る。公開の job の action を SHA で固定する。重複したビルド・PoC・写しを整理する。Node 22・mdBook 0.5.4 に揃える。0.4.3。Dependencies: pasta-check-release-asset（`release.yml`・`github-release.ps1` を共有する）

## Existing Spec Updates

2026-10-10 の discovery で、0.5.0 の道筋に乗せた既存の spec。各 brief の「申し送り（2026-10-10 の discovery・0.5.0 のノベルゲームより）」の節に書いた。

- [ ] scene-attribute-store -- 属性の読み手に、演出（`cue-command-runtime` の柱の分。2026-10-17 に `scene-stage-attributes` をまとめた）が加わる。順番を保てる倉の形、ローカルシーンと名前なしの `＊` への受け継ぎを、要件の材料にする。0.5.0 の先頭に置く。Dependencies: none
- [ ] call-attribute-filter -- フラグによる分岐の本命になる。条件の値に変数を書けること、「それ以外」の場面、数のしきい値、候補が複数残るときの扱いを、要件で決める。Dependencies: scene-attribute-store

## Direct Implementation Candidates

- なし（2026-10-18 の棚卸で、areka の `doc/PASTA_PROFILE.md` §5 の作例の件を消した。`novel-areka-contract` の brief が依頼に含めている）。

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
- タグの読み取りの制約（2026-10-07 棚卸、`paragraph-break-tag-only-talk` の範囲外）— `\nHello` をタグとして読む、引数の中の `\]` を扱わない、BudouX の行幅にエスケープと囲みを数えない。囲み（`\_?…\_?`・`\_!…\_!`）とエスケープ `\\` は、`vertical-balloon-lines` から 1 つのタグとして扱い、列を動かさない（2026-10-18）。上の R4 と一部重なる。
- 閉じていない 1 行の引用（2026-10-07 棚卸、`dsl-literal-fixes` の範囲外）— `＠w：「abc` と、行をまたげる `sakura_body` の規則はそのままにした。
- 辞書の検査の広げ方（2026-10-17 棚卸、`pasta-check-dic-validate` の範囲外）— 単語参照の検査と、Lua の中身の検査。使う人の事例が出たら起票する。
- areka が撫で・つつきのイベント（`OnMouseMove`・`OnMouseDoubleClick` など）を SSP と同じ形で送るか（2026-10-17 棚卸、`touch-poke-shortcuts` の範囲外）— 確かめていない。0.5.0 で `novel-areka-contract` の依頼に要るなら、そこで確かめる。
- 止まるエラーの表記の粗さ（2026-10-17 棚卸、`runtime-error-balloon` の実装メモ「既知の粗さ」）— 同じ場面の中で `restore_scene` の後に止まると（アクターの単語の処理など）、`＊グローバル名` だけが出て、場所とローカル名が出ない。作者が `pcall` で失敗を飲み込み、同じ要求の中で止まると、古い場面の名前が出ることがある。困った事例が出たら起票する。
- 着せ替えの復旧の出力の膨らみ（2026-10-18、`dressup-restore-stale-parts` を取り下げたときの残り）。見た目は壊れない。
  - `shiori/appearance.lua` は、アクターが着けたことのあるパーツの ON を、カテゴリごとに全部記録する。復旧のたびに、それを記録の順に出し直す（複数選択できないカテゴリでは、最後の 1 つが残る）。
  - 次の行の先頭の bind はパーツ単位で省くので、復旧の途中の見た目は古いパーツになる。間にウェイトは入らない。
  - 直すなら、着けたときに同じカテゴリのほかの ON を記録から外し、`multiple` のカテゴリだけを宣言で外す。作者が困った事例が出たら起票する。
- `pasta_check validate` が `[talk]`・`[ghost]` の budoux のキー（`budoux`・`spot_newlines`・`budoux_newlines`・`budoux_jump`・`after_choice_newlines`）を確かめない（2026-10-18 棚卸、`vertical-balloon-lines` の範囲外）。`[actor."名前"].font` は確かめる（`validate/config.rs`）ので、揃っていない。書き間違えた作者の事例が出たら起票する。
- 縦書きの列の扱いの残り（2026-10-18 棚卸、`vertical-balloon-lines` の範囲外）— 列の幅より長い 1 語は分けないので、SSP が自分で折り返した分を数えない。作者が書いたスコープの切り替え（`\0`・`\1`・`\p`・`\h`・`\u`）と `\b[…]` の後で数え直さない。`raw_script`・`act:set_property` は、ひとつだけの `\` の守りの外にある。
- スリープの印が残る件（2026-10-18 棚卸、`tutorial-ssp-gaps` の調査メモ）— `OnSysSuspend` が来て `OnSysResume` が来ず、PC が眠らなかったとき、印は次に立て直すまで残る。害が無いので受け入れた。
- バルーンが画面の左端で折り返るとき・縦に長いときの見え方（2026-10-17 棚卸、`getting-started-screenshots` の調査メモ。`hello-pasta-shell-art` からの持ち越し）— 実機で確かめていない。
- 並列負荷で時々落ちる TCP のテスト（2026-10-07 棚卸、`act-token-grouping-fix`・`scene-identity-format` の実装メモ）— `runtime_toggle_e2e_*`・`hook_panic_*`。
- マニュアルのメニューのテストが持つ mdBook の写し（2026-10-10 棚卸、`manual-claudia-theme` の実装メモ）— `book/tools/theme-menu-test.mjs` は、mdBook 0.5.4 の `book.js` のキー操作の処理を写して持つ。mdBook の版を上げるときに合わせる（CI は 0.5.3 に固定）。
- 配布版の hello-pasta に会話の変化を足すときの指針（2026-10-10 棚卸、`hello-pasta-tutorial-stages` の実装メモ）— emo2 開発からの知見「1 つの場面に 3 通り」は、段階の形を固定した辞書では満たせない。配布版に変化を足すことになったら、その動機から起票する。
- budoux の自動改行の禁則（2026-10-05、ghost_dev「emo2 開発」からの申し送り）— `line_breaker.rs` の `break_lines_impl` は禁則を見ない。BudouX の語の区切りの直前で改行するだけである。
  - 方針は JIS X 4051 どおりとする。リーダー（`‥…`）は行頭に置いてよい。並びの途中では分けない。
  - この方針では、申し送りの実例「イイジャン！／‥‥ええと、」は正しい組版になる。
  - 句読点・`！？`・閉じ括弧の前、開き括弧の後ろでは、BudouX の既定の日本語モデルは区切らない（手元で確認）。
  - BudouX が行頭禁則の字の直前（または行末禁則の字の直後）で区切る実例が出たら、その実例から起票する。その場合は、既存の `[talk]` の `chars_*` を使った追い出しを第一候補とする。
