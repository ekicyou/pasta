# Roadmap

## 概要

pasta は、日本語 DSL（Pasta DSL）で書いた辞書を Lua へトランスパイルし、組込 LuaJIT で実行する「伺か」の SHIORI（`pasta.dll`）と、その周辺ツール（`pasta_check`・LSP・VSCode 拡張・利用者マニュアル）からなる。Phase 1〜11 で、プロパティアクセス・監査・マニュアル・ソースレベルデバッグ・アクターモデル駆動・配布物の形・現行実装の不具合の一掃までを完了した（下の「完了フェーズ」）。

現在の主題は **Phase 11 の残り: 属性セマンティクスと失敗の出力**。不具合の一掃（12 本）は 2026-10-05 までに完了した（下の「完了フェーズ」）。残るのは、シーン属性の保持と読み出し、Call の属性フィルター、実行時の失敗の出力の一本化の 3 本と、2026-10-08 に起票したシーン名のエイリアス（`scene-name-alias`。`＊会話` を OnTalk の既定の別名にする）。

並行して **Phase 12: 初心者向けの入門ガイド** を進める（2026-10-06 起票）。入門ガイドを「こんな表現をしたい」を順に叶えていく物語に作り直し、ガイドの間は Claudia が全編を語る。題材の hello-pasta は、段階ごとに起動できる辞書と、fal.ai で作った新しいシェルで育て直す。マニュアルの見た目も、Claudia が前に出る親しみやすいデザインに着せ替える。Phase 11 とは、ソースの持ち場が重ならない（`crates/pasta_sample_ghost` と `book/` だけを触る）。マニュアルの全章に触れるのは、導入と締めの台詞の記法の置き換えだけ。

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
| 11 | 現行実装の不具合の一掃（文字列リテラル・生成コードの安全性・コールバックの再開・検索キー・設定とログ・シーンの内部名・アクタープロキシ・グループ化・セレクター・連結演算子・Call の文脈・段落区切り） | dsl-literal-fixes, dsl-codegen-runtime-safety, callback-resume-unification, scene-search-key-normalization, pasta-toml-logging-consistency, scene-identity-format, actor-proxy-act-delegation, act-token-grouping-fix, search-selector-indices, string-concat-operator, call-execution-correctness, paragraph-break-tag-only-talk |
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

## 棚卸（2026-10-07・main `2cbaf510`）

Phase 11 の不具合 12 本が入ったあとで、未完了の 8 本を現行 main と再照合した（各 brief の末尾に「棚卸の再測定」の節がある）。進行中の spec は無かった。前回（2026-10-04）の棚卸の即時修正の一覧は、git の履歴にある。

### 即時修正（spec なし・本棚卸で実施して閉じた）

- 完了済みの spec を指していたコードのコメントのパスを `.kiro/specs/completed/<name>/` に直した（`pasta-actor-runtime`・`sakura-script-newline`・`pasta-scene-kick`・`shiori-integration-test`・`pasta-vscode-lua-debug`・`pasta-source-map` の 6 spec、13 ファイル）。
- `pasta_sample_ghost` の `main.rs` の案内文が、もう同梱しない `pasta_scripts/` のコピーを書いていたのを直した。
- `steering/product.md` の古い記述（完了件数・現行バージョン・存在しない spec 名・現在地）を直し、進捗の正本がこの roadmap であることを書いた。
- `steering/structure.md` の `sakura_script/` の一覧に `line_breaker.rs` を足した。

### spec へ申し送ったもの（各 brief の再測定の節）

- `scene-attribute-store` — 宣言行の属性（`＊会話＆k：v`）を文法は受け付けるがパーサが捨てている。マニュアルの `grammar/block-structure.md` の「内部に記録される」はこの部分について誤り。グローバルの属性がローカルシーンへ写る処理、ファイルの属性の効く範囲（書いた位置から後ろか、先頭だけか）も要件で決める。
- `failure-output-unification` — 失敗をさくらスクリプトへ出す仕組みが既に 2 つある（`act:failure` と未登録アクターの表記）。まずこの 2 つを 1 つにする。500 の経路まで広げると 20 タスクを超えるので、その場合は分ける。
- `call-attribute-filter` — 属性が `act:call` の入口で捨てられ、検索まで届かない。シーン表を通らない検索段（現在のシーン・act のメソッド・GLOBAL）で、フィルター付きの Call をどう扱うかを要件で決める。
- `release-ci` — 追跡をやめる生成物の一覧に、`scripts/README.md` が漏れていた。シェルの画像は追跡を続ける。`release-workflow` の手順の書き換えが、次のリリースより前に要る。
- `hello-pasta-tutorial-stages` — 辞書を書き換えると `first-ghost.md` との逐語照合が後の PR で落ちる（Phase 12 の境界戦略）。
- `manual-claudia-theme` — 顔アイコンを Markdown の画像で書くと、スキル references の生成（`gen-skill-refs.mjs`）が止まる。台詞の記法の制約にする。

### 統合・分割・起票

- 分割: なし。20 タスクを超えそうなのは `failure-output-unification`（500 の経路を含める場合）だけで、要件定義で範囲を決めてから判断する。
- 順序の入れ替え: `failure-output-unification` を `call-attribute-filter` の前にした。前者が後者に機能として依存しておらず、`act.lua` を触る順番だけの関係だったため。見つからない Call の失敗表記を、一本化した仕組みに直接載せられる。
- 起票: `shiori-test-support-runtime`（完了 spec の実装メモに残っていた、結合テストが古いランタイムの写しを使う問題）。
- バックログへ: 完了 spec の範囲外に残っていた項目を下の「バックログ」に足した。

### ウェーブ（2026-10-07）

依存の木の先頭は 6 本で、互いのソースが重ならないので全部を同じウェーブに置く。

| spec | 子孫 | 種別 | 要件定義 |
| ---- | ---- | ---- | -------- |
| release-ci | 約 2（`hello-pasta-shell-art`・`getting-started-story-guide`） | 基盤（期限 2026-12-01） | Opus |
| hello-pasta-tutorial-stages | 約 2（`hello-pasta-shell-art`・`getting-started-story-guide`） | 機能 | Fable |
| scene-attribute-store | 約 1（`call-attribute-filter`） | 機能 | Fable |
| failure-output-unification | 約 1（`call-attribute-filter`） | 機能 | Fable |
| manual-claudia-theme | 約 1（`getting-started-story-guide`） | 機能 | Fable |
| shiori-test-support-runtime | 0 | 基盤 | Opus |

同じウェーブでの約束:

- `scene-attribute-store` は、属性を読み出す API を `act.lua` に置かない（`SCENE` 側に置く）。`act.lua` はこのウェーブでは `failure-output-unification` が持つ。
- `hello-pasta-tutorial-stages` は、`release.ps1`・`.gitignore`・`ghost/master/scripts/` を触らない（Lua の段階を作る場合も `scripts/` の外に置く）。`first-ghost.md` は pasta ブロックだけを触り、導入と締めの台詞は `manual-claudia-theme` に任せる。
- `manual-claudia-theme` は `first-ghost.md` の pasta ブロックを触らない。マニュアルの同じページの別の節を、`scene-attribute-store`・`failure-output-unification` と分け合う。スキル references は後から入る側が再生成する。
- `shiori-test-support-runtime` は `crates/pasta_shiori/tests/` だけを触る。

次のウェーブの候補: `call-attribute-filter`（`scene-attribute-store`・`failure-output-unification` の後）、`hello-pasta-shell-art`（`release-ci`・`hello-pasta-tutorial-stages` の後）。`release-workflow` の手順の書き換えは `release-ci` の直後に行う。その次が `getting-started-story-guide`。

### 追加起票（2026-10-08）

- `expr-nil-coercion` — `hello-pasta-tutorial-stages` の設計ディスカッション #1 で、10 段目「覚えていてほしい」の作例 `＄＊回数＝＄＊回数＋１` が現行では初回に数え始めない（未代入の変数は算術で値なし）と分かり、Lua で回避させずに DSL 側を直すと決めた。同 spec の実装着手のゲートになるので、現行ウェーブに加えて早く入れる。要件ディスカッションで、規則を「式の中の nil は算術なら 0・連結なら空文字列」に組み替えた。触るのは `act.lua` の算術・連結の数値化・文字列化（設計しだいで生成コードの形も）で、警告の文言・出口（`failure-output-unification` の持ち場）は触らない。`act.lua` は 1 ウェーブに 1 spec の約束に対する例外として、本 spec を先に入れ、`failure-output-unification` が rebase で合わせる。

### 開発者の判断が要るもの（ウェーブは止めない）

- `pasta_core` の `resolve_scene_id` の整理 — 公開 API なので、消すと semver の破壊的変更になる（バックログ）。
- `hello-pasta-shell-art` のライセンスの書き方・外部の画像ファイルの扱い・生成元の記録 — 要件定義の議題。

## Phase 11 の残り: 属性セマンティクスと失敗の出力

### 境界戦略

- **分割理由**: シーン属性を 2 つに分けた（2026-10-04）。保持・継承・読み出し（`scene-attribute-store`）と、Call の属性フィルター（`call-attribute-filter`）。実行時の失敗の出力の一本化（`failure-output-unification`）は、`call-execution-correctness` が Call 行に入れた失敗表記を土台にする。
- **共有接点**: `crates/pasta_lua/pasta_scripts/pasta/act.lua` は 1 ウェーブに 1 spec だけが持つ。`failure-output-unification` と `call-attribute-filter` が触るので、この順に置く。`scene-attribute-store` と `call-attribute-filter` は `scene.lua` と `pasta_core` のシーン登録を共有するので、この順に置く。`crates/pasta_lua/src/code_gen/element_gen.rs` も 1 ウェーブに 1 spec だけ。
- **シーン名のエイリアス**（2026-10-08 起票）: `scene-name-alias` は `＊会話` を OnTalk の既定の別名にし、pasta.toml で表を定義できるようにする。別名はキー正規化（`sanitize_name`）の前段に置き、登録・検索の両側に効かせる。`pasta_core` のシーン登録を `scene-attribute-store` と、`search/`・`scene_table.rs` を `call-attribute-filter` と共有するが、入門ガイド（Phase 12）が依存するため 2026-10-08 に最優先で先行させた。Wave 4 以降の 2 本は未着手なので、この spec の後に rebase する。hello-pasta の辞書の書き換えは Phase 12 の `hello-pasta-tutorial-stages` に任せる。

### ウェーブ構成

| Wave | spec（並走可） | 種別 | ソースの持ち場 |
| ---- | -------------- | ---- | -------------- |
| 4 | scene-attribute-store | 機能 | 宣言行の属性のパース（`parse_scene.rs`）、`scope_gen.rs`・`context.rs`・`transpiler.rs`、`scene.lua`、`finalize.rs`、`pasta_core` のシーン登録 |
| 4 | failure-output-unification | 機能 | `act.lua`・`actor.lua`・`word.lua` の警告と失敗表記（範囲によっては `shiori/event/`・`res.lua`・`pasta_shiori` の `error.rs`） |
| 5 | call-attribute-filter | 機能 | Call の文法（`grammar.pest`・`parse_action.rs`）、`element_gen.rs`（Call）、`act.lua`（`call`・`find_act_handler`）、`scene.lua`（`SCENE.search`）、`search/`、`pasta_core` の `scene_table.rs`、VSCode の文法定義 |
| 3.5（最優先・先行・2026-10-09 完了） | scene-name-alias | 機能 | pasta.toml の別名表（`config.rs`・`reference/pasta-toml.md`）、`pasta_core` の `sanitize_name` とその呼び出し元（`scope_gen.rs`・`search/context.rs`・`debug/source_map/`）、マニュアル（`block-structure.md`・`call-jump.md`・`shiori-events.md`）とスキル references |

Wave 1〜3 の 12 本は完了した（「完了フェーズ」の Phase 11）。

## Phase 12: 初心者向けの入門ガイド

### 方針（2026-10-06 決定）

- 文法リファレンス（`grammar/`）と Lua 章（`lua/`）は、リファレンスのまま残す。入門ガイドが「こんな表現をしたい」を順に叶える物語で導き、各章から詳しい文法へリンクで送り出す。
- ガイドの間は Claudia が全編を語る（説明本体も Claudia の語り）。これは `getting-started` に限った執筆規約の例外とする。コードブロック内・構文定義・コマンド例には口調を持ち込まない。
- 題材は hello-pasta のまま（Claudia ゴーストは別に存在するので、ガイドの題材にはしない）。各章の終わりで読者のゴーストが起動できるようにし、最終章の辞書を hello-pasta と一致させる。
- hello-pasta のシェルは fal.ai の画像生成で作り直す。本線は `qwen-image-edit-2511`、切り抜きは BiRefNet。
- マニュアルの見た目は、mdBook のまま「Claudia のマニュアル」に着せ替える。手本は ponapalt さんの「悪役令嬢クローディア」紹介ページ（<https://ponadocs.shillest.net/claudia/>、ponapalt/claudia、Unlicense）。Claudia の台詞は顔アイコン付きの吹き出し部品で見せる。サイト生成器の移行は採らない。

### 境界戦略

- **分割理由**: 持ち場で 3 つに分けた。シェルの画像（`crates/pasta_sample_ghost` の画像側）、段階表と段階辞書（同じクレートの辞書とテスト）、ガイドの本文（`book/`）。段階表が固まってから本文を書くので、物語と作例が食い違わない。
- **共有接点**: 段階表と段階辞書の置き場所・形（`hello-pasta-tutorial-stages` → `getting-started-story-guide`）、`tutorial-check.mjs` の照合対象、スクリーンショットに使う絵（`hello-pasta-shell-art` → `getting-started-story-guide`）。
- **`manual-claudia-theme` との接点**: 台詞の部品の記法（`manual-claudia-theme` → `getting-started-story-guide`）。`AUTHORING.md` は両方が別の節を触るので、後から入る側が rebase で合わせる。`getting-started` の既存の台詞の記法の置き換えも `manual-claudia-theme` が行い、ガイドの書き直しはその上に乗る。
- **`hello-pasta-tutorial-stages` との接点**: `hello-pasta-shell-art` と `hello-pasta-tutorial-stages` は、どちらも `crates/pasta_sample_ghost/tests/integration_test.rs` を触る（画像のテストと辞書のテスト）。段階表を先に固めるので、`hello-pasta-tutorial-stages` を先に置く（2026-10-07 棚卸）。
- **`tutorial-check.mjs` の落とし穴**: hello-pasta の `dic/` を書き換えると、`first-ghost.md` の pasta ブロックとの逐語照合が崩れる。マニュアルの CI は `crates/pasta_sample_ghost/ghosts/**` の変更では走らないため、次に `book/` を触る PR で初めて落ちる。`hello-pasta-tutorial-stages` の要件で、`first-ghost.md` の pasta ブロックを同じ spec で合わせるか、CI の対象パスに辞書を足すかを決める。
- **`release-ci` との接点**: `hello-pasta-shell-art` は `release-ci` と同じ `crates/pasta_sample_ghost/release.ps1` を触る。生成物の git 追跡の扱いも共有する（`release-ci` は生成物の追跡をやめ、`hello-pasta-shell-art` は画像を生成物から追跡する素材に変える）。そのため `release-ci` の後に置く。

| Wave | spec（並走可） | ソースの持ち場 |
| ---- | -------------- | -------------- |
| 1 | hello-pasta-tutorial-stages | hello-pasta の `dic/`、段階辞書、`pasta_sample_ghost` のテスト（`integration_test.rs` の辞書のテスト）、`first-ghost.md` の pasta ブロック（同期する場合） |
| 1 | manual-claudia-theme | `book/book.toml`・`book/theme/`（検索の tokenizer 以外）・追加の CSS と JS・画像、全章の導入と締めの台詞の記法、`AUTHORING.md` の台詞の節、検査・生成ツールの追従 |
| 2（`release-ci`・`hello-pasta-tutorial-stages` の後） | hello-pasta-shell-art | `pasta_sample_ghost` の画像生成・`shell/master/`・README・`release.ps1`・`integration_test.rs` の画像のテスト |
| 3 | getting-started-story-guide | `book/src/getting-started/`・`SUMMARY.md`・`introduction.md`・`AUTHORING.md`・`verify-content.mjs`・`tutorial-check.mjs` |

## リリースの CI 化（2026-10-06 起票）

タグの push だけでリリースが終わるようにする。

- **方式**: 単一のワークフロー `release.yml` を置く。verify → build → crates.io・Marketplace への公開 → GitHub Release の順に進む。再試行は失敗した job の再実行で行う。下書きを人が承認する 2 段方式と、release-please などの bot は却下した。
- **認証**: 長期のトークンを置かない。crates.io は Trusted Publishing、Marketplace は Entra ID のワークロード ID 連携を使う。
- **成果物**: ビルドした成果物の git 追跡をやめ、CI がタグ時点のソースから作る。
- **期限**: Marketplace の global PAT は 2026-12-01 に廃止されるので、それまでに完了させる。
- **ソースの持ち場**: `.github/workflows/release.yml`、`release.ps1`、`build-wasm.ps1`、`.gitignore`、`release/`、`crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/` の生成物（`pasta.dll`・`THIRD_PARTY_LICENSES.txt`・`scripts/README.md`）。シェルの画像は `hello-pasta-shell-art` が追跡する素材に変えるので、追跡をやめない。Phase 11 の spec とソースが重ならないので、どのウェーブとも並走できる。
- **`release-workflow` の追従**: `release-ci` が入ると、常駐 spec の手順（`release/hello-pasta.nar` を生成してコミットする段）がそのままでは動かない。次のリリースより前に、下の Existing Spec Updates を済ませる。

### Existing Spec Updates

- [ ] release-workflow -- エージェントの手順を「版の決定 → bump の PR のマージ → タグの push → CI の結果確認・失敗した job の再実行」へ縮める。次のものは CI 側へ移すか、要らなくなる。
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
      - 小さな取りこぼし: setup-check の `verify-pat` が失敗すると、vsce のエラー文（`Access Denied: <ID> needs ...`）にマネージド ID の識別子が含まれ、ジョブのログに出る。「profile ID は summary にだけ書く」方針から漏れている。秘密の値ではないので実害は無い。直すかどうかは本更新で決める。
    - あわせて見直す（`release-ci` の design.md「Out of Boundary」が本更新へ回したもの）: `.claude/settings.json` の公開系コマンドの許可の整理、`build.yml` に `--locked` を足すか、bump 箇所に `package-lock.json` の版を含めること（release.yml の verify は検査しない）。
    - `VSCE_PAT` の失効（10 節）は、初回のリリースで Marketplace が Entra ID の経路で `published` になったのを確かめてから行う。値が `release-ci` の会話記録に出ているが、前倒しはしない（ユーザー決定 2026-10-08）。
  - Dependencies: release-ci

## 文中のシーンリンク（2026-10-08 起票）

台詞の中の語を Wikipedia のリンクのようにし、クリックでそのシーンへ飛べるようにする（`scene-anchor-link`）。

- **方式**:
  - 記法は `＠？シーン名`（`＠単語　` と同じく、空白か改行で終える）と `＠？シーン名「表示名」`。
  - 出力はさくらスクリプトの `\_a`（アンカー）で、`\q` ではない。`\q` だと、トーク全体が選択肢待ちになるため。
  - クリックは `OnAnchorSelectEx` を、選択肢の振り分け（`choice_select.lua`）と同じ規則で受ける。
  - `【】` で囲む案は、文法として唐突なので採らなかった。
- **飛び先が無いとき**: 選択肢と同じく 204 を返す。失敗の見せ方は `failure-output-unification` に乗せる。読み込み時の検出は、バックログの「`.pasta` を検査するコマンド」に任せる。
- **ウェーブ**: `grammar.pest`・`parse_action.rs`・`element_gen.rs`・`act.lua`・VSCode の文法定義を、`call-attribute-filter` と共有する。そのため、その後に置く（Phase 11 の Wave 6 相当）。

## Specs (dependency order)

- [ ] scene-attribute-store -- シーン属性の実行時の保持・Lua からの読み出し・ファイルレベル属性の継承と上書き・値の型解釈。Dependencies: dsl-literal-fixes, scene-identity-format, call-execution-correctness
- [ ] failure-output-unification -- 実行時の失敗（未定義の参照・見つからない Call など）をログとさくらスクリプトの両方へ 1 つの仕組みから出す。`call-execution-correctness` が Call 行に入れる失敗表記を載せ替え、他の失敗へ広げる。Dependencies: none
- [ ] call-attribute-filter -- Call の属性フィルター構文（`＞シーン＆k＝v`・比較演算子・複数条件）と実行時の絞り込み。Dependencies: scene-attribute-store, failure-output-unification
- [x] scene-name-alias -- シーン名のエイリアス表を pasta.toml で持ち、未定義なら「会話 → OnTalk」の 1 件を既定にする。完全一致のみ、キー正規化の前段で登録・検索の両側に効かせる（`＊会話` の宣言も `＞会話` の Call も OnTalk になる）。Dependencies: none（2026-10-08 に最優先で先行。`scene-attribute-store`・`call-attribute-filter` はこの後に rebase する）
- [x] release-ci -- タグ `vX.Y.Z` の push を契機に、GitHub Actions で verify・成果物のビルド・crates.io と Marketplace への公開・GitHub Release までを冪等に行う。認証は OIDC（Trusted Publishing・Entra ID）。成果物の git 追跡を解除し、一回限りのセットアップの手順書を作る。Marketplace の global PAT が廃止される 2026-12-01 より前に完了させる。Dependencies: none
- [ ] hello-pasta-tutorial-stages -- 「こんな表現をしたい」の段階表を確定し、段階ごとに起動できる辞書一式を CI で検証する。hello-pasta の辞書を教材として書き直し、最終段階と一致させる。Dependencies: none
- [x] manual-claudia-theme -- マニュアルを mdBook のまま「Claudia のマニュアル」に着せ替える。配色・字体・枠などの意匠は ponadocs の Claudia 紹介ページ（Unlicense）を手本にし、ダーク版と表紙の扉を用意する。顔アイコン付きの台詞の部品を作り、全章の導入と締めの台詞を書き換える。検索・着色・`file://` 閲覧・検査ツールは壊さない。Dependencies: none
- [ ] hello-pasta-shell-art -- hello-pasta の女の子・男の子の立ち絵を、fal.ai で作ったイラスト（表情 9 種ずつ・透過 PNG・表情間でずれない）に置き換え、生成物から素材の扱いに切り替える。Dependencies: release-ci, hello-pasta-tutorial-stages
- [ ] getting-started-story-guide -- 入門ガイドを段階表に沿った物語に書き直し、全編を Claudia が語る（執筆規約に `getting-started` の例外を足す）。段階辞書との逐語照合と、新しいシェルのスクリーンショットを含む。Dependencies: hello-pasta-tutorial-stages, hello-pasta-shell-art, manual-claudia-theme
- [x] expr-nil-coercion -- 式の中の nil を、算術の文脈なら 0、連結の文脈なら空文字列とみなし、ログを出さない。nil 以外の変換できない値（数字でない文字列・真偽値など）は警告を出して 0・空文字列とみなす。`＄＊回数＝＄＊回数＋１` が初回から数え始める。マニュアルとスキル references を同じ PR で直す（2026-10-08、`hello-pasta-tutorial-stages` の設計ディスカッション #1 から起票。要件ディスカッションで規則を組み替えた）。Dependencies: none
- [ ] shiori-test-support-runtime -- `pasta_shiori` の結合テストがコピーして使う古いランタイムの写し（`tests/support/scripts/`）を撤去し、本物のランタイムだけで動かす。回避用の `pasta.toml` の設定とコメントを外す（2026-10-07 棚卸で起票）。Dependencies: none
- [ ] scene-anchor-link -- 台詞の中の `＠？シーン名`（`「表示名」` も付けられる）を、さくらスクリプトのアンカー `\_a` として出す。クリックで、`OnAnchorSelectEx` からそのシーンへ飛ぶ。選択肢の振り分けを共有し、LSP・VSCode の着色とマニュアルまで揃える（2026-10-08 起票）。Dependencies: failure-output-unification, call-attribute-filter

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
- `pasta_shiori` の `util/hglobal/windows_api.rs` の `string_to_multibyte`（即時修正で判明）— `@enc` と同じ 65001 で不正になるフラグを渡すが、テストからしか呼ばれない。`pasta_lua` の `encoding/windows.rs` と同じ先頭の分岐（65001 ならバイト列をそのまま返す）を足せば数行で直る。テスト専用なので、消す案もある。
- `.pasta` を検査するコマンド（2026-10-07 棚卸、`dsl-codegen-runtime-safety` の申し送り）— `pasta_check` に `.pasta` を検証するサブコマンドが無い。同 spec は使い捨てのクレートで代用した。
- 式の木の優先順位（2026-10-07 棚卸、`dsl-codegen-runtime-safety` の調査）— `pasta_dsl` の `parse_action.rs` の `build_left_assoc_expr` は優先順位なしで木を作り、コード生成（`expr_gen.rs`）が組み直している。AST を直接使う別の利用者が出たら直す。
- 別ファイルの同名シーンのデバッガ索引（2026-10-07 棚卸、`scene-identity-format` の範囲外）— ファイルをまたぐ同名シーンの索引と通し番号は、既存の制約として残した。
- サニタイズ後のシーン名の衝突の警告（2026-10-07 棚卸、`scene-search-key-normalization` の範囲外）— 記号を落とした名前が別のシーンと重なる場合に、読み込み時か `pasta_check` で警告する。上の「モジュール名の衝突」とは別の件。
- タグの読み取りの制約（2026-10-07 棚卸、`paragraph-break-tag-only-talk` の範囲外）— `\nHello` をタグとして読む、`\_?…\_?` の中のタグも読む、引数の中の `\]` を扱わない、BudouX の行幅にエスケープと囲みを数えない。上の R4 と一部重なる。
- 閉じていない 1 行の引用（2026-10-07 棚卸、`dsl-literal-fixes` の範囲外）— `＠w：「abc` と、行をまたげる `sakura_body` の規則はそのままにした。
- 並列負荷で時々落ちる TCP のテスト（2026-10-07 棚卸、`act-token-grouping-fix`・`scene-identity-format` の実装メモ）— `runtime_toggle_e2e_*`・`hook_panic_*`。
- print.html の動画の参照切れ（2026-10-09、`manual-claudia-theme` の完了時の棚卸し・spec なしで直せる小さな修正）— `book/src/debug/dev-actions.md` の生の HTML `<video src="media/dev-actions-demo.mp4">` と代わりの `<a href="media/dev-actions-demo.mp4">` は、章のページ（`debug/dev-actions.html`）では解決するが、出力の根にある `print.html` では `debug/media/` を指せない（mdBook は生の HTML の属性を書き換えない）。着せ替え前の版（`classic/`）にもある既存の不具合。`verify-static.mjs` は `print.html` のこの参照だけを例外として見逃している。
  - 直し方は一通りでない（動画の置き場所を出力の根に移して章側を `../media/` にする・ビルド後の変換で `print.html` の参照を書き換える・絶対パスにするなど）。`file://` のオフライン閲覧と `site-url = "/pasta/"` を両立させる案を選ぶ。
  - 完了の条件: 章のページと `print.html` の両方で動画と代わりのリンクが働く。`verify-static.mjs` の `print.html` の例外を外しても合格する。章の本文の変更は動画の参照だけにとどめ、`gen-skill-refs --check` とリンク検証に合格する。
- budoux の自動改行の禁則（2026-10-05、ghost_dev「emo2 開発」からの申し送り）— `line_breaker.rs` の `break_lines_impl` は禁則を見ない。BudouX の語の区切りの直前で改行するだけである。
  - 方針は JIS X 4051 どおりとする。リーダー（`‥…`）は行頭に置いてよい。並びの途中では分けない。
  - この方針では、申し送りの実例「イイジャン！／‥‥ええと、」は正しい組版になる。
  - 句読点・`！？`・閉じ括弧の前、開き括弧の後ろでは、BudouX の既定の日本語モデルは区切らない（手元で確認）。
  - BudouX が行頭禁則の字の直前（または行末禁則の字の直後）で区切る実例が出たら、その実例から起票する。その場合は、既存の `[talk]` の `chars_*` を使った追い出しを第一候補とする。
