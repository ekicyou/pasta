# Implementation Plan — pasta-user-manual

> 凡例: `(P)` は直前ピアと並行実行可能。`_Depends:_` は非自明な依存。要件IDは数値のみ。

- [ ] 1. 基盤: mdBook サイト基盤・検索PoC・執筆規約
- [x] 1.1 mdBook プロジェクト雛形と日本語サイト基盤
  - `book.toml`（`language=ja`、検索有効、サブパス公開用 `site-url`、相対パス出力）、`theme/` 既定を用意
  - **全章構成を定義した `src/SUMMARY.md` と、各章のプレースホルダ `.md`（introduction/grammar/lua/getting-started/reference 全章）を用意**し、内容タスクが SUMMARY を編集せず既存ファイルを埋めるだけで済むようにする（並列安全化）
  - コードブロックがテーマ標準ハイライトで判読可能・全章を辿る目次が出る設定
  - 観測可能: `mdbook build` が成功し、`book/` に静的アセットのみのサイトが出力され、トップが日本語（UTF-8）で文字化けなく表示、`file://` で目次ナビが機能する
  - _Requirements: 1.1, 1.2, 1.5, 3.1, 3.2, 3.3_

- [x] 1.2 (P) 日本語 bigram 検索 PoC（最優先・リスク低減）
  - 最小フィクスチャ章で、ビルド後に `searchindex.js` を 2-gram で再生成するスクリプトと、クエリを同一規則で 2-gram 分割する共有 `tokenize` を試作
  - PoC が不成立の場合は劣化フォールバック（見出し・用語の確実ヒット）への切替をここで判断・記録する
  - 観測可能: フィクスチャで「造体」（語中2文字）検索が「構造体」を含むページにヒットし、`file://` でも検索が動く
  - _Requirements: 2.4_
  - _Boundary: Bigram Search_
  - _Depends: 1.1_

- [x] 1.3 (P) 執筆規約（ボイスガイド＋流用/リンク方針）の確立
  - `book/AUTHORING.md`（非公開）に「Claudia 導入 → 普通文体の本体 → 締め」の基準ボイスサンプルと、コード/表/構文定義内にキャラ口調を持ち込まない規則を明文化
  - `doc/spec/` をトランスクルージョンせず書き下ろし＋リンク参照する流用方針を記載
  - 観測可能: AUTHORING.md に各章が準拠できる基準サンプルと流用/リンク方針が揃う
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 8.1_
  - _Boundary: Voice & Tone Guide_
  - _Depends: 1.1_

- [x] 1.4 (P) ドリフト追跡マッピングの骨格
  - `book/manual-sources.toml` に「文法章 → 由来 doc/spec 章・節」の対応と、参照時点のコンテンツハッシュ（版マーカー）を記録する正準ファイルを用意（全文法章ぶんのエントリ枠）
  - 観測可能: manual-sources.toml に全文法章の doc/spec 対応と各 doc/spec 章の版マーカーが記録される
  - _Requirements: 10.1_
  - _Boundary: Drift Detection & Gate_
  - _Depends: 1.1_

- [ ] 2. コア: マニュアル本文の執筆
- [x] 2.1 (P) Pasta DSL 文法リファレンス（全実装網羅・参照型）
  - `GRAMMAR.md`（読みやすさ）＋ `doc/spec/`（完全性）を母体に、全実装済み文法要素（マーカー/ブロック/アクション行/リテラル/変数/単語/アクター辞書/さくらスクリプト/Call-Jump、doc/spec ch02–07・09–11 相当）を網羅執筆。各要素に試せる例、各章末に doc/spec 権威リンク
  - 未実装（ch08 属性）・将来（ch12）は除外または「将来変更あり」注記で区別。入門チュートリアルとは独立した参照型セクションとする
  - 観測可能: `src/grammar/` の各章プレースホルダが全実装済み文法の本文で埋まり、各章末に doc/spec リンクが存在し、manual-sources.toml の対応章と整合する（SUMMARY は 1.1 で確定済み・編集しない）
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 8.4_
  - _Boundary: Content: Grammar_
  - _Depends: 1.3, 1.4_

- [x] 2.2 (P) Lua API / コーディングマニュアル
  - 公開モジュール・API（`@pasta_search`/`@pasta_persistence`/`@pasta_config`/`@pasta_sakura_script`/`@enc`/`@pasta_log` 等）、`scripts/` 記述パターン、DSL/Lua 使い分け、各 API の試せる例を執筆。対象方言 LuaJIT 2.1 を明示し、初心者向け Lua 基礎の入口＋外部参照リンクを置く
  - 観測可能: `src/lua/` に API/パターン/使い分け/基礎入口の章が揃い、LuaJIT 2.1 明示と外部参照リンクが存在する
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 8.2_
  - _Boundary: Content: Lua_
  - _Depends: 1.3_

- [x] 2.3 (P) 入門 / チュートリアル
  - hello-pasta（`pasta_sample_ghost/dist-src`）を底本に、ゼロから起動可能な完全な最小ゴースト一式（boot/talk/actors 等）へ至る手順を段階的に執筆。最終成果物は起動可能な最小一式に一致させ、起動しない部分集合では固定しない
  - 前提環境（Windows/SSP・他ベースウェアは概ね動作）、`.pasta` の UTF-8 作成と Shift_JIS 辞書移行注意、初心者向けの専門用語説明を明示
  - 観測可能: `src/getting-started/` に前提環境と、ゼロから起動可能な最小ゴーストに到達する完結した手順が揃う
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6_
  - _Boundary: Content: Getting Started_
  - _Depends: 1.3_

- [x] 2.4 (P) 外部リファレンス・権威リンク章
  - 日本語 Lua 5.1/5.2 リファレンス（milkpot 版）と LuaJIT 公式（luajit.org）への参照リンクを置き、版の離れた lua55 系は言語リファレンスとして不採用と明記。`doc/spec/` を権威ソースとして案内
  - 観測可能: `src/reference/` に外部 Lua リファレンス＋ doc/spec 権威への導線が揃い、lua55 系不採用が明記される
  - _Requirements: 8.2, 8.3, 8.4_
  - _Boundary: Content: References_
  - _Depends: 1.3_

- [x] 2.5 (P) バージョン・方言・環境バナー
  - `src/introduction.md` に対象 pasta バージョン（系列）と LuaJIT 2.1 方言を明示。安定機能を主軸とし、未確定・実装予定・将来変更部を「将来変更あり」注記で視覚的・記述的に区別
  - 観測可能: introduction に対象バージョン・LuaJIT 2.1 方言・流動部の区別注記が読者に見える形で記載される
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 5.5_
  - _Boundary: Version & Currency_
  - _Depends: 1.3_

- [ ] 3. コア: 日本語 bigram 検索の本実装
- [x] 3.1 (P) bigram 索引ビルダ本実装
  - PoC を本実装化。`build-index.mjs` が mdBook 出力の検索インデックスを 2-gram で再生成（mdBook 同梱 elasticlunr を再利用しスキーマ一致）、`tokenize.mjs` を索引・クエリ共有の単一正準モジュールとする。索引サイズを監視し 10MB 警告閾値内を確認
  - 観測可能: `mdbook build` 後にビルダを実行すると `searchindex.js` が bigram 索引へ再生成され、索引サイズが閾値内
  - _Requirements: 2.1, 2.2, 2.3, 2.4_
  - _Boundary: Bigram Search_
  - _Depends: 1.2_

- [x] 3.2 検索クエリの bigram override
  - `theme/searcher.js` を、索引ビルダと同一の `tokenize.mjs` を用いてユーザー入力クエリを 2-gram 分割するよう改変（ロジックをブラウザ側へ注入/同梱）
  - 観測可能: 検索ボックスに日本語の語中2文字を入力すると該当ページが結果一覧に出て選択で遷移し、サーバー通信なしで動作する
  - _Requirements: 2.1, 2.3, 2.4, 2.5_
  - _Boundary: Bigram Search_
  - _Depends: 3.1_

- [ ] 4. コア: ドリフト検出スクリプト
- [x] 4.1 (P) drift-check 実装（マーカー方式）
  - `drift-check.mjs` が、manual-sources.toml の記録ハッシュと `doc/spec/` 現値ハッシュを比較してドリフトを検出（git 差分非依存）、未マップの doc/spec 章・節を警告、マニュアル→doc/spec および外部参照のリンク切れを検出
  - 観測可能: doc/spec 章を改変し対応章を未更新にすると drift-check が非ゼロ終了し、未マップ・リンク切れも一覧で報告される
  - _Requirements: 10.2, 10.4_
  - _Boundary: Drift Detection & Gate_
  - _Depends: 1.4_

- [ ] 5. 統合: 公開パイプラインとチュートリアル検証
- [x] 5.1 GitHub Pages 公開ワークフロー
  - `.github/workflows/manual.yml` を新規作成し、`configure-pages` → `mdbook build` → bigram 索引再生成 → drift-check → `upload-pages-artifact` → `deploy-pages` を結線。`book/**` 変更時に起動し既存 `build.yml` と独立。permissions（pages/id-token/contents）設定
  - 観測可能: ワークフローが一連を完走し、公開 URL でトップ表示と日本語検索が機能する
  - _Requirements: 1.3, 1.4, 10.2, 10.4_
  - _Depends: 3.1, 3.2, 4.1_

- [x] 5.2 チュートリアル成果物の構文 lint 結線
  - チュートリアル末の成果物を pasta パーサ／`pasta_check` の構文検証に通す CI ステップを追加し、起動可能な最小セット一式（hello-pasta 由来）との一致を確認
  - 観測可能: CI でチュートリアル末の `.pasta` が構文検証を通過し、hello-pasta 最小一式との一致が確認される
  - _Requirements: 6.2_
  - _Depends: 2.3, 5.1_

- [ ] 6. 統合: 完了承認ゲートへのドリフト統合
- [x] 6.1 workflow.md DoD に Manual Sync Gate を追加
  - `.kiro/steering/workflow.md` の DoD に、`doc/spec/` か `book/` に触れた変更時のみ発火する条件付き「Manual Sync Gate」を1件追加（既存5ゲートの意味は変えない）。ルール本体はここに置く
  - 観測可能: workflow.md DoD に条件付き Manual Sync Gate が記載され、無関係変更ではスキップする旨が明記される
  - _Requirements: 10.3, 10.5_
  - _Boundary: Drift Detection & Gate_
  - _Depends: 4.1_

- [x] 6.2 kiro-spec-complete に drift-check 発火を結線
  - `.claude/skills/kiro-spec-complete/SKILL.md` のステップ1（DoD 検証）に drift-check を発火するオーケストレーションを追加（ルールは複製せず workflow.md を参照）。未解決ドリフトで完了を中断
  - 観測可能: 完了承認フローの DoD 検証で drift-check が発火し、未解決ドリフトがあれば完了が中断、無関係変更ではスキップされる
  - _Requirements: 10.3, 10.5_
  - _Boundary: Drift Detection & Gate_
  - _Depends: 6.1_

- [ ] 7. 検証
- [x] 7.1 (P) 静的出力・オフライン閲覧の検証
  - 出力が静的アセットのみでサーバープロセス無しに表示でき、`file://` で目次ナビ・章間リンク・コードブロック表示が機能することを確認
  - 観測可能: サーバーを起動せず `book/index.html` を開いて全ページ閲覧・ナビ・リンク遷移ができる
  - _Requirements: 1.1, 1.2, 1.5, 3.2, 3.3, 3.4_
  - _Depends: 5.1_

- [x] 7.2 (P) 日本語検索の検証
  - フィクスチャ／実章で「造体」（語中2文字）・「構造体」（3文字）検索が該当ページにヒットし、検索がサーバー通信なしで動作、索引サイズが閾値内であることを確認
  - 観測可能: 語中2文字・3文字いずれの日本語クエリでも該当ページが結果に出る
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Depends: 3.2, 5.1_

- [x] 7.3 (P) ドリフト検出・完了ゲートの検証
  - ドリフト注入（doc/spec 改変＋章未更新）で失敗、未マップ章追加で警告、リンク切れ検出、完了 DoD 実行時の中断、doc/spec・book いずれにも触れない変更でのスキップを確認
  - 観測可能: ドリフト・未マップ・リンク切れの各ケースで期待どおり検出/中断し、無関係変更ではゲートがスキップされる
  - _Requirements: 10.2, 10.3, 10.4, 10.5_
  - _Depends: 4.1, 6.2_

- [x] 7.4 (P) コンテンツ整合・網羅レビュー（編集チェックリスト）
  - 文法章が全実装済み要素を網羅し doc/spec と矛盾しない、Lua 章が公開モジュールを網羅し LuaJIT 2.1 明示、チュートリアルが起動可能な最小ゴーストに到達、導入/締めが Claudia ボイス・本体が普通文体・コード内に口調なし、外部参照が日本語 Lua 5.1/5.2＋LuaJIT 公式で lua55 系を言語リファレンスに案内していない、バージョン・流動部注記を確認
  - 観測可能: チェックリスト全項目が確認済みで、要件 R4〜R9 のコンテンツ受入基準を満たす
  - _Requirements: 4.1, 4.5, 5.1, 5.5, 6.1, 6.2, 7.1, 7.2, 7.4, 8.2, 8.3, 9.1, 9.3, 9.4_
  - _Depends: 2.1, 2.2, 2.3, 2.4, 2.5_

## Implementation Notes
- 1.1: mdBook v0.5.3 は検索成果物を **ハッシュ付きファイル名**（`searchindex-<hash>.js` / `searcher-<hash>.js` / `elasticlunr-<hash>.min.js`）で出力する。検索（bigram）タスク 3.1/3.2 の `build-index.mjs`・`theme/searcher.js` は固定名でなくこのハッシュ付きスキームを前提に実装すること（Bigram Search の「同パス上書き」はハッシュ名の解決が必要）。
- 1.1: 公開リポは `ekicyou/pasta` 想定で `book.toml` の `git-repository-url`・`site-url="/pasta/"` を設定。公開 URL は `https://ekicyou.github.io/pasta/`。公開ワークフロー（5.1）はこれに整合させること。
- 5.2: `pasta_check` CLI は `release` サブコマンドのみで `.pasta` 構文検証コマンドは無い。構文ガードは既存 `cargo test -p pasta_sample_ghost`（`self_deploy_integration_test` が実 `.pasta` を `PastaLoader::load` で parse/transpile）を採用。一致ガードは `book/tools/tutorial-check.mjs`（first-ghost.md の pasta ブロック ↔ ghosts/hello-pasta dic を逐語照合）。**注意: `cargo test -p pasta_sample_ghost` は副作用で `crates/pasta_sample_ghost/README.md` を実ツリーから再生成する**（ローカルでは要 revert。CI の ephemeral checkout では無害）。cargo 実行時は `NoDefaultCurrentDirectoryInExePath` を外す（LuaJIT ビルド対策）。
- 3.2: mdBook v0.5.3 は **`theme/searcher.js` の上書きを無視**する（実機確認: 置いても既定 searcher が byte 同一で出力）。クエリ bigram override は **`book/theme/head.hbs`** に実現（全ページ `<head>` に注入し、elasticlunr ロード前に `Object.defineProperty` で `elasticlunr.tokenizer` を 2-gram に差し替え）。tokenize は `tokenize.mjs` をバイト等価でインライン同梱（ズレ防止コメント必須）。実コンテンツの語中2文字（クリプ/ースト）で RED→GREEN 実証。検証は `mdbook build book` → builder → head.hbs の tokenizer で再現。
- 2.3: hello-pasta の SSOT は **`crates/pasta_sample_ghost/ghosts/hello-pasta/`**（`dist-src/` は現存せず、設計/README の記述は陳腐化）。**5.2 のチュートリアル構文 lint は `ghosts/hello-pasta/ghost/master/dic/*.pasta` を底本に**すること。`crates/pasta_sample_ghost/README.md` の dist-src 記述は本 spec 境界外ゆえ別途修正（spawn task 候補）。
- 2.1: **book 外（`doc/spec/`・`GRAMMAR.md` 等）への参照リンクは GitHub 絶対 URL を使う**こと（`https://github.com/ekicyou/pasta/blob/main/<path>`）。相対 `.md` リンクは mdBook が `.html` に書き換え、book 外はレンダリングされないため GitHub Pages で 404 になる。book 内の章間リンクは相対 `.md` のままで可。後続 2.2/2.4（lua/reference の外部参照）も同様に絶対 URL を使うこと。
- 1.2: PoC 成立（route B bigram で語中2文字ヒットを実証、フォールバック不要）。本実装の正準モジュールは `book/tools/bigram-index/{tokenize.mjs, build-index.mjs}`。3.1 への申し送り: ①`build-index.mjs` の索引リテラル切り出しが `lastIndexOf('))')` 依存で脆い → 本実装ではより堅牢な抽出に。②`build-index.mjs` は索引再構築時に `pipeline.reset()` で bigram トークンを保全、3.2 の searcher.js も検索時に標準 pipeline がトークンを壊さないこと（`elasticlunr.tokenizer` を tokenize へ override）を確認。③索引肥大（10MB 閾値）は本番フルコンテンツで実測。
