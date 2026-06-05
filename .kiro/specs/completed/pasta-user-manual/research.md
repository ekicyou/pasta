# Gap Analysis: pasta-user-manual

> 本ドキュメントは要件（requirements.md）と既存資産の差分を分析し、設計フェーズの判断材料を提供する。最終決定ではなく選択肢と情報の提示が目的。

## 0. 調査メソッドと検証

- 既存ドキュメント資産の棚卸しは Explore サブエージェントで実施し、主要主張は本体で再検証した。
- **検証で判明した訂正**: サブエージェントが「`crates/pasta_lua/LUA_API.md` が存在する」と報告したが、**実在しない**（`**/LUA_API.md` グロブ＝該当なし）。Lua API の再利用母体は**生成済み Markdown ではなく** `.claude/skills/pasta-lua-coding/SKILL.md`（＋ `tech.md` の「Luaランタイムパターン」節）である。以下のマップはこの訂正を反映する。
- 確認済み事実: `GRAMMAR.md`（ルート）存在 / `book.toml` 不在 / CI は `.github/workflows/build.yml`（Cargo ビルド用）のみ存在し docs ビルドなし / `ekicyou/lua55-manual-ja` 参照は brief・requirements・各スキルに存在。

## 1. 現状調査（Current State）

本仕様は「新規の成果物（静的サイト）」を作るものであり、拡張対象の既存コンポーネント（コード）は存在しない。一方、**マニュアルの母体となるコンテンツ資産は豊富に存在する**。

### コンテンツ資産（再利用元）

| 資産 | 所在 | 規模 | 性質 | 再利用度 |
| ---- | ---- | ---- | ---- | -------- |
| 言語仕様書（全12章） | `doc/spec/01〜12-*.md` | 計 ~1,200 行 | 権威的・実装判断向け | ⭐⭐⭐⭐ 流用可（利用者向けに口調変換要、ch4/8/12 は要選別） |
| 文法クイックリファレンス | `GRAMMAR.md`（ルート） | ~829 行 | 既に利用者向け・例文豊富 | ⭐⭐⭐⭐⭐ ほぼそのまま母体化可 |
| Lua API/コーディング参照 | `.claude/skills/pasta-lua-coding/SKILL.md`（＋ `references/`） | 中規模 | AI 向け簡潔参照 | ⭐⭐⭐ 母体化可だが**展開・例示の加筆要** |
| Lua ランタイムパターン | `.kiro/steering/tech.md`「Luaランタイムパターン」節 | 数十行 | モジュール表・定型パターン | ⭐⭐⭐ 補助母体 |
| 辞書制作パターン | `.claude/skills/pasta-ghost-authoring/SKILL.md`（＋ `references/`） | 中規模 | AI 向け辞書制作集 | ⭐⭐⭐ チュートリアル素材 |
| hello-pasta サンプル一式 | `crates/pasta_sample_ghost/`（`dist-src/ghost/master/dic/*.pasta`, `README.md`, `install.txt`） | 完成品 | 実物のゴースト＋手順 | ⭐⭐⭐⭐ チュートリアル実例の母体 |
| 素の Lua 言語リファレンス | 外部（日本語 Lua 5.1/5.2 例: milkpot 版 + LuaJIT 公式 luajit.org） | 外部 | 外部所有 | 参照リンクのみ（取り込まない）。**ランタイム＝LuaJIT 2.1 に合わせ、Lua 5.5 系（`lua55-manual-ja`）は方言不一致のため言語リファレンスに採用しない**（要件ディスカッション#3） |

### ビルド/公開の現状

| 項目 | 現状 | 判定 |
| ---- | ---- | ---- |
| mdBook ツール | `mdbook v0.5.3` 導入済み（cargo bin、実機検証済み） | ✅ 利用可 |
| `book.toml` / `SUMMARY.md` / `src/` | 不在 | ❌ 新規作成 |
| docs ビルド CI | 不在（`build.yml` は Cargo のみ） | ❌ 新規作成（任意） |
| GitHub Pages 設定 | gh-pages/CNAME 等の確認なし | ❌ 新規設定 |
| 配置場所 | Pure Virtual Workspace。`doc/` が既存 | 構造上 `pasta/` 直下に `book/`（または `doc/manual/`）が自然 |

## 2. 要件→資産マップ（Requirement-to-Asset Map）

| 要件 | 必要な技術的要素 | 既存資産 | ギャップ判定 |
| ---- | ---------------- | -------- | ------------ |
| R1 静的サイト生成（サーバー不要） | 静的サイトジェネレータ＋ビルド構成 | mdBook 導入済み | **Missing**（`book.toml`/構成の新規作成。ツールは充足） |
| R2 クライアントサイド全文検索 | 静的検索インデックス＋日本語トークナイズ | mdBook 標準検索（elasticlunr） | **Unknown**（日本語の検索再現性に懸念＝下記 Research Needed） |
| R3 日本語表示・ナビ・閲覧 | UTF-8 表示・目次・コード表示・相互リンク | mdBook 標準で充足見込み | Low（標準機能で対応可、要確認） |
| R4 Pasta DSL 文法マニュアル | 文法解説コンテンツ | `doc/spec/` + `GRAMMAR.md` | **Constraint**（流用可だが口調変換・権威ソースとの非重複管理） |
| R5 Lua API/コーディング | API・規約・例示コンテンツ | `pasta-lua-coding` スキル + `tech.md` | **Missing/部分**（LUA_API.md は不在。スキルから展開・例示の加筆要） |
| R6 入門/チュートリアル | ゼロから作る手順＋動く例 | hello-pasta 一式 + 各スキル | **Missing**（手順の新規書き下ろし。素材は揃う） |
| R7 執筆方針（Claudia ボイス） | 一貫した語り口・ボイスガイド | 既存の「声付き」ドキュメントは皆無 | **Missing**（全面新規の執筆作業・ボイスガイド確立） |
| R8 既存資産再利用・外部参照 | 流用方針・権威ソース不変・外部リンク | 全資産＋ lua55 別リポ | **Constraint**（二重管理ドリフト防止策が必要） |

## 3. 実装アプローチ選択肢

ここでの「拡張 vs 新規」は、コンポーネント改変ではなく**コンテンツ・ソーシング戦略とサイト配置**の選択として読み替える。

### Option A: 参照リンク主体（既存資産を「拡張」＝薄いサイト）
マニュアルは最小限の導線・索引のみを新規作成し、本文は `doc/spec/` や `GRAMMAR.md` へリンクで誘導する。
- ✅ 二重管理ほぼゼロ、初期工数最小
- ✅ 権威ソースとの矛盾が起きにくい
- ❌ mdBook は外部 Markdown のトランスクルージョンを標準では行えず、リンク飛ばしばかりで「統合された読み物」にならない
- ❌ Claudia ボイス（R7）や利用者向け口調変換（R4）をほぼ実現できない
- **適合度**: 低（R4/R6/R7 と相性が悪い）

### Option B: 全面書き下ろし（新規＝厚いサイト）
`book/src/` に全章を新規執筆し、既存資産は下敷きにしつつ独立した本文として書き切る。
- ✅ Claudia ボイス・利用者向けトーン・構成の完全な自由度（R4〜R7 に最適）
- ✅ 単一サイトで完結した読書体験
- ❌ `doc/spec/` とのコンテンツ重複 → 仕様変更時のドリフトリスク（R8 の二重管理）
- ❌ 工数大
- **適合度**: 中〜高（品質は出るが保守コスト）

### Option C: ハイブリッド（推奨）
- 文法（R4）: `GRAMMAR.md` を主母体に `book/src` へ取り込み、利用者向けに再編。各章末に**「権威的仕様は `doc/spec/` を参照」**の注記＋リンクを置き、`doc/spec/` は権威ソースとして不変（R8）。
- Lua API（R5）: `pasta-lua-coding` スキルを母体に、利用者向けの説明・例示を加筆して新規執筆。
- チュートリアル（R6）: hello-pasta 一式を実例に新規書き下ろし。
- ボイス（R7）: 最初にボイスガイド（基準サンプル）を1つ確立し、導入・繋ぎ・締めに適用。説明本体は普通の文体。
- サイト基盤（R1〜R3）: `book.toml` + `SUMMARY.md` + `src/` を新規作成。検索・日本語は標準機能を検証して採否判断。
- ✅ 品質（ボイス・読み物性）と保守（権威ソース不変・リンク参照）のバランス
- ✅ 段階実装可能（基盤 → 文法 → Lua → チュートリアル → ボイス通し）
- ❌ 「どこを流用しどこをリンクに留めるか」の線引き設計が必要
- **適合度**: 高

## 4. 工数・リスク

| 区分 | 工数 | リスク | 一言根拠 |
| ---- | ---- | ------ | -------- |
| サイト基盤（book.toml/SUMMARY/構成） | S | Low | mdBook 導入済み、定型作業 |
| 文法マニュアル（R4） | M | Low | 母体が高品質（GRAMMAR.md）、口調変換中心 |
| Lua API/コーディング（R5） | M | Medium | LUA_API.md 不在。スキルからの展開・例示加筆が必要 |
| 入門/チュートリアル（R6） | M | Medium | 新規書き下ろし。手順の動作確認が要る |
| Claudia ボイス通し（R7） | M | Medium | 一貫性維持＋技術正確性の両立 |
| 検索・日本語・公開（R2/R3/CI/Pages） | S〜M | **Medium〜High** | 日本語全文検索の再現性が未知（下記） |
| **総合** | **L（1〜2週相当）** | **Medium** | 大半はコンテンツ作業。技術リスクは検索と二重管理ドリフトに集中 |

## 5. Research Needed（設計フェーズへ持ち越す調査項目）

1. **【最重要】日本語全文検索の再現性（R2-4）**: 要件ディスカッションで**合格基準＝「任意の連続2文字以上の日本語語句で、その語句を本文に含むページが検索結果に出る」に確定**（検証可能）。実現手段は design で選定する。候補:
   - **route C: Pagefind 差替**（CJK分かち書き内蔵・静的・GitHub Pages で動作。ただし索引を `fetch` 取得するため `file://` 直開きでは検索不可 → R2-5 で配信前提に限定済み）。最有力。
   - **route B: bigram 前処理**（mdBook 標準 elasticlunr の索引を2文字N-gram化。`file://` でも検索可だが索引肥大・前処理自作）。
   - mdBook 標準のまま（route A）は語中一致不可のため**不採用**。
   設計タスク: route B/C の選定、索引サイズ・ビルド手順・GitHub Pages 配信との整合の検証。
2. **コンテンツ流用の線引き（R8）**: `doc/spec/` の各章について「再編して取り込む」か「リンク参照に留める」かの章単位ポリシー。二重管理ドリフトを防ぐ運用ルール（更新時の同期手順 or single-source 化）。
3. **サイト配置場所**: `book/`（ルート直下）か `doc/manual/` か。既存 `doc/spec/` との関係と CI から見た自然さ。
4. **公開パイプライン（R1/CI）**: 公開先は **GitHub Pages に確定**（R1-3）。残る設計判断は「GitHub Actions による mdBook 自動ビルド→デプロイ」か「手動ビルド＋ push 運用」かの構成選択、および Pages 有効化のリポジトリ設定作業（ブランチ or Actions ソース、`.nojekyll` は mdBook 出力に同梱済み）。
5. **ボイスガイドの確立（R7）**: 「Claudia 令嬢」の基準サンプル（導入文・繋ぎ・締めのテンプレ）を1つ design/実装初期に確定し、以降の一貫性基準とする。
6. **コード例の検証可能性（R6）**: チュートリアルの .pasta 例が実際に動作することを担保する手段（hello-pasta との整合 or 例の実行確認）。

## 6. 設計フェーズへの推奨

- **推奨アプローチ**: Option C（ハイブリッド）。
- **主要意思決定**: ①サイト配置場所、②章単位の「流用 vs リンク」ポリシー、③日本語検索の方針、④公開パイプラインの要否、⑤ボイスガイドの確定タイミング。
- **確定済み制約（要件ディスカッション2巡目）**: 実行環境＝Windows/SSP 主・他補足、コンテンツは UTF-8（R6-5/6）。検索合格基準＝連続2文字以上の日本語部分一致（R2-4）。**オフライン（file://）閲覧を維持（R1-5）するため、ビルドは相対パス構成とすること**（検索は HTTP 配信時のみで file:// では非対象、という棲み分けは許容済み）。
- **持ち越し調査**: 上記 Research Needed の 1〜6（特に 1 の日本語検索を最優先）。
- 工数 L・リスク Medium。技術的不確実性は「日本語検索」と「二重管理ドリフト」の2点に集中しており、それ以外はコンテンツ執筆作業として見通しが立つ。

---

# 設計ディスカバリー（design フェーズ）

## D-1. 日本語検索の実現手段 — route B（bigram）に確定

外部調査の結論。要件 R2-4「任意の連続2文字以上の日本語語句で該当ページがヒット」＋ R1-5「`file://` オフライン閲覧維持」の**両方**を満たすのは **route B（明示的 bigram 索引化）のみ**。

| route | 2文字部分一致 | file:// 動作 | 採否 | 根拠 |
| ----- | ------------- | ------------ | ---- | ---- |
| A. mdBook標準（elasticlunr 既定） | ✕（空白区切り、日本語語中不可） | ◯ | 不採用 | — |
| B. **bigram 索引化**（標準UI維持） | **◎（全2-gram索引）** | **◯（索引は静的js同梱）** | **採用** | pg_bigm と同原理。位置非依存で全2文字連接を索引化 |
| C. Pagefind | ✕（`Intl.Segmenter` の word 粒度＝単語分割。語中・語またぎ取りこぼし。`--fuzzy-cjk` 未実装） | **✕（索引を `fetch` 取得、CORS で file:// 不可）** | 不採用 | [pagefind#987](https://github.com/Pagefind/pagefind/issues/987), [multilingual](https://pagefind.app/docs/multilingual/) |
| B'. lunr-languages 単語分割（`mdbook-search-cjk` 系 fork） | ✕（「構造」はOKだが「構造体」NG の既知欠陥） | ◯ | 不採用 | [dalance記事](https://qiita.com/dalance/items/0a435d66e29f505faf6b) |

**重要**: 既製の bigram プリプロセッサは**確認できず**（`mdbook-pagefind` は crates.io に不在、`mdbook-search-cjk` は単語分割で要件未達）。→ **自前の bigram 化が必要**＝本 spec の最大の実装リスク。

実装方式（design 決定）: mdBook 標準検索 UI（`elasticlunr.min.js` + `searcher.js`）を維持しつつ、
1. **ビルド後に検索インデックスを bigram で再生成**（`searchindex.js` を 2-gram トークンで作り直す build-time ツール）
2. **クエリ側も bigram トークナイズ**（`theme/searcher.js` でユーザー入力を 2-gram 分割）— 索引とクエリの両側を bigram 化しないと「任意の2文字以上」が成立しないため必須

索引とクエリの bigram 化を**対**で行うのが成立条件。索引肥大（語彙増）はサイズ監視（mdBook の 10MB 警告閾値）で管理。

## D-2. GitHub Pages 公開 — 公式 artifact ワークフロー

`actions/configure-pages` → `mdbook build` → `actions/upload-pages-artifact`(path `./book`) → `actions/deploy-pages`。permissions: `pages: write` / `id-token: write` / `contents: read`。
- 公式 artifact デプロイは **Jekyll 処理なし → `.nojekyll` 不要**。`peaceiris/actions-gh-pages`（gh-pages ブランチ方式）は非標準化。
- サブパス公開（`https://user.github.io/repo/`）時は `site-url = "/repo/"`。**アセットは document-relative リンクで出力**でき、これにより `file://` 閲覧と両立（route B 標準検索だから成立。Pagefind を入れると壊れる→ route B 採用を補強）。
出典: [公式 starter workflow](https://github.com/actions/starter-workflows/blob/main/pages/mdbook.yml), [mdBook CI](https://rust-lang.github.io/mdBook/continuous-integration.html)

## D-3. 設計シンセシス（Generalization / Build-vs-Adopt / Simplification）

- **Adopt**: サイト基盤（mdBook）・検索 UI（elasticlunr/searcher.js）・公開（公式 Pages workflow）は既製を採用。**Build は bigram 層のみ**（索引再生成＋クエリトークナイザ）に最小化。
- **Generalization**: コンテンツ章（文法／Lua／入門）は「Claudia 導入 → 普通文体の本体 → 締め」という共通テンプレートを持つ。ボイスガイド（R7-5）をその単一基準として最初に確立し、全章が準拠する。
- **Simplification（流用 vs リンク＝R8 ポリシー確定）**: `doc/spec/` を**トランスクルージョン（自動同期）しない**。マニュアルは Claudia ボイスで**書き下ろし**、`doc/spec/` を権威ソースとして各章末から**リンク参照**する（R4-3, R8-4）。学習用の言い換えと権威仕様の重複は「意図的な役割分担」として許容し、二重管理は「権威は doc/spec、マニュアルは派生」という一方向の参照で回避。`GRAMMAR.md` は母体として下敷きにするが、マニュアル側で再編する。
- **サイト配置**: `book/`（リポジトリルート直下、既存 `doc/` と並列）。
- **コード例の検証（R6）**: チュートリアルの `.pasta` 例は `crates/pasta_sample_ghost` の hello-pasta 配布物と整合させ、実在する動作サンプルを底本にする。

## D-4. 設計ディスカッション決定（validate-design 後）

| # | 議題 | 決定 |
| - | ---- | ---- |
| 1 | bigram 検索層 | 索引ビルダを build-time Node 化し `searcher.js` と単一 `tokenize.mjs` を共有（別言語二重実装を排除）。PoC を最優先タスク化。PoC 不成立時は「見出し・用語の確実ヒット」へ劣化フォールバック |
| 2 | チュートリアル動作担保 | 末尾 `.pasta` を hello-pasta 実ファイルの部分集合に固定＋`pasta_check`/パーサ構文 lint を CI 化（フル SSP 実行なしの機械的ガード） |
| 3 | doc/spec ドリフト検出 | **スコープ拡大を承認**。章マッピング（`manual-sources.toml`）＋ `drift-check.mjs`、かつ**完了承認フロー（`workflow.md` DoD ＋ `kiro-spec-complete`）にゲート統合**。横断プロセスへの影響を避けるため**条件付き発火**（doc/spec か book に触れた変更のみ）。要件 R10 として追加 |

注: 議題3 は Claudia から「完了フローは横断プロセスゆえ境界はみ出し」の慎重論を提示したうえで、開発者が意図的にスコープ拡大を選択。R10.5（条件付きスキップ）で無関係 spec への波及を抑制する設計とした。
