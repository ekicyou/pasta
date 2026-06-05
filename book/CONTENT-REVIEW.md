<!--
このファイルは編集レビューの内部記録です。
SUMMARY.md には載せず、mdBook のビルド対象（公開サイト）には含めません。
タスク 7.4「コンテンツ整合・網羅レビュー（編集チェックリスト）」の確認記録。
機械検証は book/tools/verify-content.mjs（node 実行で全項目アサート）が担い、
本ファイルはそれを補完する人手確認項目（doc/spec との非矛盾の最終目視等）を記録する。
-->

# コンテンツ整合・網羅レビュー記録（タスク 7.4）

> 対応要件: Requirement 4, 5, 6, 7, 8, 9（タスク `_Requirements_`: 4.1, 4.5, 5.1, 5.5, 6.1, 6.2, 7.1, 7.2, 7.4, 8.2, 8.3, 9.1, 9.3, 9.4）。
> 関連設計: design.md「Testing Strategy / コンテンツ整合・編集レビュー」。
> 観測可能な完了条件: チェックリスト全項目が確認済みで、要件 R4〜R9 のコンテンツ受入基準を満たす。

本レビューは二段構えで確認する。

1. **機械検証**: `node book/tools/verify-content.mjs` を実行し、実ファイル（`book/src/**`・`book/manual-sources.toml`・`book/tools/*`）を走査して網羅・整合・ボイスを自動アサートする。**exit 0（全 65 項目 PASS）を確認済み。**
2. **人手確認**: スクリプトで自動化しきれない「doc/spec との非矛盾の最終目視」等を本ファイルに列挙し、各項目に確認状況と根拠を記す。

機械検証の各カテゴリ（A〜F）の ID は `verify-content.mjs` の出力 `[ID]` に対応する。

---

## A. 文法網羅（R4.1 / R4.5）

| # | 確認項目 | 状況 | 根拠 |
| - | -------- | ---- | ---- |
| A-1 | grammar 全実装章（index/markers/block-structure/call-jump/literals/action-line/sakura-script/variables/words/actor-dictionary）が存在し本文を持つ | 確認済 | `verify-content` `[A-body:*]` 全 PASS。各章 3.3K〜7K 字の実本文。 |
| A-2 | 各文法章末に doc/spec 権威リンク（GitHub 絶対 URL）がある | 確認済 | `[A-link:*]` 全 PASS。全章が `https://github.com/ekicyou/pasta/blob/main/doc/spec/...` を持つ。 |
| A-3 | `manual-sources.toml` が全文法章（index 除く9章＋index 由来）を doc/spec に対応付け、source が実在する | 確認済 | `[A-toml]`/`[A-toml-src]` PASS。01–07,09–11 の 10 source 全実在。 |
| A-4 | **doc/spec と矛盾しない**（最終目視） | 確認済 | doc/spec 実装済み章は 01–07・09–11。grammar 章はこの 11 章に 1:1 対応。未実装 ch08（属性）・将来 ch12 は「安定機能」として記述せず、`block-structure.md`/`index.md`/`call-jump.md`/`words.md`/`actor-dictionary.md` で **「将来変更あり」** 注記により明示区別（R4.6 整合）。文法要素の説明が doc/spec の定義と齟齬する箇所は目視で検出されず。 |

## B. Lua 網羅（R5.1 / R5.5）

| # | 確認項目 | 状況 | 根拠 |
| - | -------- | ---- | ---- |
| B-1 | 公開モジュール `@pasta_search`/`@pasta_persistence`/`@pasta_config`/`@pasta_sakura_script`/`@enc`/`@pasta_log` が登場 | 確認済 | `[B-mod:*]` 全 PASS。`lua/modules.md` に各モジュール専用セクション＋試せる例。 |
| B-2 | 対象方言 LuaJIT 2.1 を明示 | 確認済 | `[B-luajit]` PASS。`index.md`/`basics.md`/`modules.md`/`patterns.md` で LuaJIT 2.1（Lua 5.1 系）明示。 |
| B-3 | Lua 基礎入口＋外部参照リンク | 確認済 | `[B-basics]` PASS。`basics.md` に基礎文法と外部リファレンス導線。 |

## C. チュートリアル（R6.1 / R6.2）

| # | 確認項目 | 状況 | 根拠 |
| - | -------- | ---- | ---- |
| C-1 | 前提環境（Windows / SSP）・UTF-8 保存・Shift_JIS 移行注意 | 確認済 | `[C-env]`/`[C-utf8]`/`[C-sjis]` PASS。`prerequisites.md`/`first-ghost.md`/`index.md` に記載。 |
| C-2 | ゼロから起動可能な最小ゴーストに到達する完結手順 | 確認済 | `[C-steps]` PASS。`first-ghost.md` は約 23K 字の段階手順。 |
| C-3 | 最終成果物が**起動可能な最小一式**に一致 | 確認済 | `[C-tutorial-check]` PASS。`tutorial-check.mjs` が `first-ghost.md` の `pasta` ブロックと hello-pasta `dic/*.pasta`（actors/boot/talk/click/choice）の逐語一致を確認。起動可能性は既存 `cargo test -p pasta_sample_ghost` が transitively 担保（5.2 で結線）。 |

## D. ボイス（R7.1 / R7.2 / R7.4）

| # | 確認項目 | 状況 | 根拠 |
| - | -------- | ---- | ---- |
| D-1 | 各章の導入/締めに Claudia 令嬢のキャラ口調がある | 確認済 | `[D-voice:*]` 全 20 章 PASS。各章の散文部（コードフェンス除外）にボイスマーカーを検出。 |
| D-2 | 本体（仕様・構文・手順・API 説明）が普通文体 | 確認済（目視） | `AUTHORING.md` の Do/Don't に準拠。表・本体段落は淡々とした記述で、口調による誤読の余地なし。 |
| D-3 | **コードブロック内に解説ナレーションの口調が無い** | 確認済 | `[D-codevoice]` PASS。`verify-content` は Claudia 固有の地の文マーカー（わたくし/おほほ/フンッ/ごきげんよう Claudia）をコードフェンス内で検査し 0 件。なお `action-line.md` の `.pasta` 作例にあるアクター「ラザニア」の台詞（「〜ですわ」等の丁寧語尾）は**サンプルゴーストの会話内容そのもの**であり、解説者ナレーションの混入ではない（R7.4 が禁じる「正確さの母体への装飾」に該当しない）。この区別を機械検査でも採用済み。 |

## E. 外部参照（R8.2 / R8.3）

| # | 確認項目 | 状況 | 根拠 |
| - | -------- | ---- | ---- |
| E-1 | 日本語 Lua 5.1/5.2 リファレンス（milkpot 版）を絶対 URL で案内 | 確認済 | `[E-lua51]`/`[E-lua52]` PASS。`reference/external-links.md` に `milkpot.sakura.ne.jp/lua/lua51_manual_ja`・`lua52_manual_ja`。 |
| E-2 | LuaJIT 公式（luajit.org）を絶対 URL で案内 | 確認済 | `[E-luajit]` PASS。`luajit.org`/`luajit.org/extensions.html`。 |
| E-3 | **lua55 系を言語リファレンスとして不採用と明記** | 確認済 | `[E-lua55]` PASS。「不採用とする資料」表で lua55 系を「言語リファレンスとして不採用」と理由付きで明記。言語リファレンスへの案内として lua55 系へのリンクは存在しない。 |

## F. バージョン・流動部注記（R9.1 / R9.3 / R9.4）

| # | 確認項目 | 状況 | 根拠 |
| - | -------- | ---- | ---- |
| F-1 | introduction に対象 pasta バージョン系列（v0.2 系列）を明示 | 確認済 | `[F-version]` PASS。`introduction.md` の表と本文で v0.2 系列を明示。 |
| F-2 | LuaJIT 2.1 方言を明示 | 確認済 | `[F-luajit]` PASS。 |
| F-3 | 安定機能と「将来変更あり」の流動部を視覚的・記述的に区別 | 確認済 | `[F-future]` PASS。`introduction.md` に「将来変更あり」凡例。各文法章でも引用ブロック注記で実装済みと将来部を区別。 |

---

## 総括

- 機械検証 `node book/tools/verify-content.mjs`: **exit 0 / 65 項目 PASS / 0 FAIL**（実ファイル走査）。
- 人手確認（A-4 doc/spec 非矛盾、D-2 本体文体、E-3 lua55 不採用の意味的確認）: いずれも整合を確認。
- 結論: 要件 **R4〜R9 のコンテンツ受入基準を全項目で満たす**。

### 重複回避メモ

本レビューは 7.4 の責務（コンテンツの網羅・整合・ボイス）に限定する。静的出力・オフライン閲覧（7.1）、日本語検索（7.2）、ドリフト検出・完了ゲート（7.3）は各専用タスク・スクリプトが担い、本ファイル／`verify-content.mjs` では重複検証しない。
