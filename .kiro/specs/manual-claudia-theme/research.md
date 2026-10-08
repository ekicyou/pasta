# Research & Gap Analysis: manual-claudia-theme

生成日: 2026-10-08（要件生成直後のギャップ分析。`/kiro-validate-gap`）
対象: `requirements.md`（Requirement 1〜10）と現行の `book/` 配下（mdBook 0.5.3・`theme/head.hbs`・`book/tools/`）

---

## 1. 現状の調査

### 1.1 マニュアルの構成と資産

| 資産 | 所在 | 現状 |
| ---- | ---- | ---- |
| mdBook 設定 | `book/book.toml` | `default-theme = "light"`・`preferred-dark-theme = "navy"`・`site-url = "/pasta/"`・検索有効・fold level 1・リダイレクト 1 件。`additional-css`/`additional-js` は未使用。独自 CSS は無い |
| テーマ上書き | `book/theme/head.hbs`（269 行） | 2 つのインライン `<script>` のみ。(a) 検索クエリ側の bigram tokenizer（`tokenize.mjs` の逐語コピー。`verify-search.mjs` が `BEGIN/END canonical bigram tokenize` マーカーで切り出して照合）、(b) `hljs` 再ハイライト中和（`neutralizer.mjs` の逐語コピー）。`<link>`・CSS は無い。`index.hbs`・`css/*` の上書きは無い |
| 章 | `book/src/**/*.md` | 47 章（`SUMMARY.md` 掲載。`AUTHORING.md` は非掲載）。画像・HTML タグを含む章は 0。全章が「H1 → 導入（口調）→ `---` → 本文 → `---` → 締め（口調）」の構造 |
| 表紙 | `book/src/introduction.md` | 導入・締めの台詞、関連リンク表、対象バージョン表（`verify-content` F 系が Cargo.toml と照合）、「将来変更あり」注記、歩き方、権威の宣言 |
| 画像 | なし | `book/src` に画像ファイルは存在しない。`verify-static` の許可拡張子に `.png`・`.svg`・`.webp`・`.woff2` 等は既に含まれる |
| ビルド後加工 | `book/tools/highlight/highlight-html.mjs`・`book/tools/bigram-index/build-index.mjs` | `mdbook build` の後に出力 HTML を書き換える Node スクリプト（決定論・冪等）。CI は Node 20・`npm ci`（jsdom・vscode-textmate・vscode-oniguruma） |
| 検査 | `verify-static.mjs`・`verify-search.mjs`・`verify-content.mjs`・`link-check.mjs`・`tutorial-check.mjs`・各 `*-test.mjs` | 依存ゼロ（Node 標準）。CI `manual.yml` が順に実行し、1 件の失敗で公開を止める |
| 生成 | `gen-skill-refs.mjs` | `GENERATION_MAP`（23 章）→ 2 スキルの `references/`。`extractBody` が最初と最後の `---` で導入・締めを捨てる。本文散文（表の行・フェンス・インラインコード除く）に `VOICE_MARKERS` があると `voice-in-body` で失敗 |
| 公開 | `.github/workflows/manual.yml` | `book/**` 変更で起動。PR は検証のみ、main push で Pages 公開 |

### 1.2 既存の規約・慣習

- **口調判定の一本化**: `findVoice`／`VOICE_MARKERS` は `gen-skill-refs.mjs` が持ち、`verify-content.mjs` が import する。新記法の判定も同じ場所に足すのが筋。
- **逐語ミラーの規律**: ブラウザ側ロジック（tokenizer・neutralizer）は Node 側の正準モジュールから `export` を除いた逐語コピーを `head.hbs` に置き、テストが一致を検査する。台詞部品を JS で変換する方式を採る場合、同じ規律が要る。
- **ビルド後加工の型**: `highlight-html.mjs` は「出力 HTML を再帰グロブ → 対象ブロックを置換 → 決定論・冪等・失敗で exit 1」。台詞部品の変換を build-time 加工で行うなら、この型を再利用できる。
- **mdBook のテーマ上書き制約（実測済み）**: 0.5.3 は `theme/searcher.js` を上書き対象に含めない。`head.hbs`・`index.hbs`・`css/*.css`・`fonts/` は対象。
- **静的性**: `verify-static` は本文ページの `href`/`src` の相対参照が出力配下に実在すること、ルート絶対参照（`/pasta/...`）が無いことを検査する。絶対 URL（`https://`）は対象外（Google Fonts の `<link>` は検査に掛からない）。
- **Manual Sync Gate**（`workflow.md` DoD 6）: `book/` に触れる spec は `gen-skill-refs.mjs --check` と `link-check.mjs` の通過が完了条件。

### 1.3 参考サイトの実態（2026-10-08 取得）

- ソースは `site/index.html` 1 枚（CSS はインライン）と `site/img/`。外部 CSS ファイルは無い。字体は Google Fonts（`BIZ UDPGothic`・`BIZ UDPMincho`・`Cormorant Garamond ital 500`・`Shippori Mincho B1 600/800`）を `<link>` で読み込む。
- **台詞（`.talk`）は画像**: `<div class="talk"><img class="b" src="img/cc-*.png"><img class="f" src="img/f7.png"></div>` の形で、吹き出し本体は PNG（文言は `alt`）、顔は `.f`（72px 円形・`background: var(--paper)`）。**文字の吹き出し部品は参考サイトに存在しない**。マニュアルの台詞部品（本文はテキスト・検索対象）は、配色と顔アイコンの扱いだけを参考にして新規に作る。
- **顔アイコン**: `site/img/f0.png`〜`f9.png`・`f25.png`〜`f28.png`（Claudia 14 種、各 15〜43KB）、`f10.png`・`f11.png`（アンソニー）。番号はシェルの `surfacetable.txt` のサーフェス ID と一致し、表情名が確定できる:
  `0 素／1 照れ／2 驚き／3 不安／4 落胆／5 高笑い／6 目閉じ／7 不機嫌／8 冷笑／9 照れ怒り／25 にっこり／26 したり顔／27 考え中／28 お辞儀（カーテシー）`
- **アンソニー（相方の執事。要件ディスカッションで話し手に追加）**: `surfacetable.txt` の `group,アンソニー` は `10,素`・`11,刮目`（＋ドラッグ中）。参考サイトは `.talk.anthony .f { background: #EEF2FA; }` で顔アイコンの背景だけを青みに変え、台詞は執事の丁寧語（「〜でございます」「お嬢様」）。マニュアルでは文字色の色味（Claudia 赤み・アンソニー青み）を話し手ごとの CSS クラスで切り替える。
- **立ち絵は使わない（議題 2 で決定）**: `site/img/s0.png`（172KB）等の合成済み立ち絵は取り込まない。表紙も顔アイコン 2 枚で構成する。
- **承認済みモックアップ**: `mockup.mjs`（本フォルダ。顔アイコンを取り寄せたディレクトリを渡すと単一 HTML を生成）。CSS 変数の初期値（ライト／navy）、台詞部品の構造（`.talk.claudia`／`.talk.anthony`、左右振り分け、顔 56px、名札は Cormorant 斜体）、扉の構造はここが出発点。
- **ライセンス**: GitHub API で `spdx_id: Unlicense` を確認。
- **意匠の部品**: `.paper.hero`（角飾り `.corner tl/tr/bl/br`）・`.divider`（金の菱形）・`.card`・`.steps`・`.paper.letter`（`.seal` 封蝋）・`.note`・`.caption`・`.latin`（Cormorant 斜体の添え字）。

---

## 2. 要件と資産の対応（ギャップ）

| 要件 | 既存資産 | ギャップ | 区分 |
| ---- | -------- | -------- | ---- |
| R1 テーマ（ライト） | mdBook 既定 `light` の CSS 変数（`variables.css`）・`general.css`・`chrome.css` | Claudia 配色・字体・角丸・区切り・枠の CSS 一式が無い。`additional-css` で変数と部品を上書きする経路は mdBook が提供済み | Missing |
| R1.3–1.6 字体 | mdBook 同梱の Open Sans／Source Code Pro（`theme/fonts/fonts.css`・woff2 同梱。`verify-static` R3.4 が woff2 の存在を見る） | 日本語 Web フォントの読み込み経路（Google Fonts `<link>` か同梱）が無い。フォールバック字体の指定も無い | Missing／未確定事項 3・8 |
| R1.7 全ページ適用 | `index.hbs` が全章・`print.html`・検索結果に共通 | `additional-css` は全ページに入る。問題なし | — |
| R2 ダーク | mdBook の `.navy`・`.coal`・`.rust`・`.ayu` クラスに CSS 変数が定義済み | テーマごとの Claudia 配色の変数上書きが無い。顔アイコンの縁の処理が無い | Missing |
| R3 台詞部品（表示） | なし。参考サイトの `.talk` は画像方式 | 文字の吹き出し＋円形顔アイコンの部品を新規に作る。表情→画像の対応表が要る（1.3 で確定） | Missing |
| R4 記法 | Markdown のみ。mdBook は生 HTML をそのまま通す | 「HTML を直接書かない」記法と、それを HTML に変える変換が無い。変換の置き場所が設計の中心論点（§3） | Missing |
| R4.4–4.5 執筆規約 | `AUTHORING.md` 第 1〜6 節・チェックリスト | 台詞部品の節と、基準ボイスサンプルの置き換えが無い。`getting-started-story-guide` と別の節を触る（rebase 前提） | Missing |
| R5 全章置き換え | 47 章すべてが「導入 → `---` → 本文 → `---` → 締め」で統一されている | 置き換えは機械的に進められる。表情の付与基準が無い（未確定事項 7） | Constraint |
| R5.4 生成物不変 | `extractBody` が導入・締めを捨てる | 記法が `---` の外側に収まる限り生成物は不変。記法が `---` を含むと `bad-structure` になる（記法設計の制約） | Constraint |
| R6 表紙の扉 | `introduction.md` の冒頭段落と F 系検査 | 扉の HTML（角飾り紙枠・顔アイコン 2 枚・掛け合い）が無い。`introduction.md` は非生成章なので HTML を直接置いてよい（`link-check` は HTML タグのリンクを検査しないため、画像参照は `verify-static` の `src` 実在検査が担う） | Missing |
| R7 素材 | `book/src` に画像なし。`verify-static` は `.png`・`.webp` を許可 | 画像の置き場所（`book/src/img/` など）、縮小・形式（PNG→WebP の可否）、ライセンス文の置き場所が未決 | Missing／未確定事項 5・6 |
| R8.1 検索 | `head.hbs` tokenizer・`verify-search.mjs` | `head.hbs` に追記しても `BEGIN/END` マーカー内を変えなければ照合は通る。tokenizer 本体には触らない | Constraint |
| R8.2 台詞が索引に入る | mdBook の検索索引は章の描画結果からテキストを抽出 | 生 HTML ブロック内のテキストが mdBook 0.5.3 の索引に入るかは **未確認**（§5 Research Needed）。入らない場合はビルド後加工（`build-index.mjs`）側で補う余地がある | Unknown |
| R8.3 着色の可読性 | `scope-map.mjs` が TextMate スコープ→hljs クラスを写像。配色は mdBook の `highlight.css`（light）・`tomorrow-night.css`（navy/coal）・`ayu-highlight.css`。コードブロック背景は `highlight.css` が独自に持つ | 紙色の背景に対する hljs 配色の対比は未検証。`additional-css` で `.hljs` 背景・各クラス色を上書きできる | Unknown |
| R8.4 `file://` | `verify-static` | 追加資材を相対参照にすれば問題なし。ルート絶対参照（`/pasta/img/...`）を書かないこと | Constraint |
| R8.5 外部通信 | 現在は外部通信ゼロ | Google Fonts を採ると初の外部通信（任意）が入る。`preconnect`・`display=swap` でオフライン時に体裁を保つ必要 | Constraint／未確定事項 3 |
| R8.6 印刷 | `print.html` は `index.hbs` の同型 | 台詞部品・扉の印刷用スタイル（`@media print`）が要る | Missing |
| R9 スマートフォン | mdBook の既定ブレークポイント（サイドバー折りたたみ） | 台詞部品・扉・表の狭幅レイアウトが要る | Missing |
| R10 ツール追従 | `findVoice`・`extractBody`・`verify-content` D/G 系・`verify-static` 参照検査・`*-test.mjs` | 記法の構文検査（存在しない表情名・崩れ）が無い。`verify-search` に台詞語の検索検査が無い | Missing |

---

## 3. 実装アプローチの選択肢

### 3.1 台詞部品の記法と変換（設計の中心論点）

| 案 | 記法の例 | 変換の場所 | 長所 | 短所 |
| -- | -------- | ---------- | ---- | ---- |
| **A. 生 HTML を書く** | `<div class="claudia" data-face="高笑い">…</div>` | 変換不要（mdBook が素通し） | 実装最小。CSS だけで済む | R4.1「HTML を直接書かない」に反する。HTML ブロック内の Markdown は mdBook（pulldown-cmark）で解釈されない（強調・リンクが使えない → R3.5 に反する）。`link-check` が HTML 内リンクを見ない |
| **B. 決まった形の引用ブロック（Markdown のまま）＋ build-time 変換** | `> 【Claudia／高笑い】 さあ、熱く参りましょう！` のような引用ブロック | `mdbook build` 後の Node スクリプト（`highlight-html.mjs` と同型）が、出力 HTML の該当 `<blockquote>` を台詞部品の HTML に置換 | 記法は純 Markdown。本文の強調・リンクは mdBook が描画済み。`verify-content`・`gen-skill-refs`・`link-check` は Markdown を読むので変更最小（口調判定は引用ブロックを散文として既に見ている）。索引は加工後の HTML を `build-index.mjs` が読むなら入る | 変換前の素の表示（`mdbook serve` のプレビュー）は普通の引用ブロックになる。CI の加工ステップが 1 つ増える。`print.html` も加工対象に含める必要 |
| **C. 引用ブロック記法＋クライアント JS 変換** | B と同じ | `additional-js` が読み込み時に `<blockquote>` を書き換える | 加工ステップ不要。プレビューでも見える | JS 無効・読み込み前のちらつき。検索索引は mdBook の抽出に依存（引用ブロック内テキストは入る）。`print.html` の印刷前に JS が走る保証が薄い。逐語ミラー規律のような正準管理が要る |
| **D. mdBook preprocessor（Node コマンド）** | B と同じ、または独自の短い記法 | `book.toml` の `[preprocessor.claudia] command = "node book/tools/..."` が Markdown→Markdown（HTML 埋め込み）を変換 | `mdbook build`・`mdbook serve` の両方で効く。HTML への変換を Markdown 段階で完結 | `mdbook build` 単体が Node に依存する（CI は既に Node あり。開発機も同様）。HTML ブロック内の Markdown 解釈は A と同じ問題（本文を先に HTML へ変換する必要） |

**所見**: B（Markdown 記法＋build-time 変換）が既存の規律（build-time 加工・Markdown ベースの検査）と最も整合する。D は記法と置き場所が同じで、プレビュー性を取るならこちら。A は R3.5・R4.1 に反し、C は印刷・JS 依存の弱点がある。**記法は引用ブロックの先頭に話し手と表情を置く形**が、`extractBody`（引用は散文扱い）・`findVoice`（口調検査に掛かる）・`link-check`（`](` を検査）のすべてと無改修で噛み合う。記法の中に行全体 `---` を含めてはならない。

### 3.2 テーマ（CSS）の載せ方

| 案 | 内容 | 長所 | 短所 |
| -- | ---- | ---- | ---- |
| **A. `additional-css` のみ** | `book/theme/claudia.css`（仮）1 枚で CSS 変数（`:root`・`.light`・`.navy`…）と部品スタイルを上書き | mdBook 更新に強い。差分が 1 ファイルに集まる。全ページ・`print.html` に入る | 既定 CSS の詳細度に負ける箇所は `!important` か詳細度の工夫が要る |
| **B. `theme/css/*.css` を丸ごと上書き** | mdBook の `variables.css`・`general.css`・`chrome.css`・`print.css` をコピーして編集 | 完全な制御 | mdBook のバージョン更新ごとに手動マージ。差分が大きい |
| **C. A ＋必要箇所だけ `index.hbs`** | 基本は A。扉や上部バーに構造の変更が必要なら `index.hbs` を上書き | 構造変更の自由 | `index.hbs` 上書きは mdBook 更新との競合リスク。扉は `introduction.md` 内の HTML で足りる見込み |

**所見**: A を基本にする。扉は `introduction.md` に HTML を直接書けば `index.hbs` は不要（非生成章・`verify-content` F 系は Markdown の表を見るので共存可）。

### 3.3 Web フォント

| 案 | 内容 | 長所 | 短所 |
| -- | ---- | ---- | ---- |
| **A. Google Fonts `<link>`** | 参考サイトと同じ。`head.hbs` の先頭（tokenizer ブロックの外）または `additional-css` の `@import` | 容量ゼロ。実装が最小 | 初の外部通信。オフラインではフォールバック字体（要件 1.6）。`@import` は描画のブロックが長い |
| **B. 同梱（サブセット化）** | `theme/fonts/` に woff2 を置く | オフラインでも同じ見た目。外部通信ゼロの現状を保つ | 日本語 4 書体のサブセット化が build-time に要る（新規ツール・容量数百 KB〜数 MB）。マニュアルの全文字集合に追従する保守 |
| **C. ハイブリッド** | 見出し（文字数が少ない）だけ同梱、本文は Google Fonts | 見出しの意匠は常に出る | 複雑 |

**所見**: 配信元は未確定事項 3（要件ディスカッション）。A が最小。B は「外部通信ゼロ」を守れるが保守が重い。

**`head.hbs` への追記について（要件ディスカッションで整理）**: brief.md の制約は「検索 tokenizer に手を触れない」であり、ファイル全体の凍結ではない。`verify-search.mjs` は `BEGIN/END canonical bigram tokenize` マーカーで切り出して照合するため、マーカー外（ファイル先頭など）への `<link>` 追記は照合に影響しない。したがって `<link>` 追記と `additional-css` 内 `@import` のどちらを採るかは、描画ブロック時間・`preconnect` の要否で設計が決める（設計判断）。

### 3.4 検査・生成ツールの追従

- **追従の最小集合**（案 B／D の記法を前提）:
  - `gen-skill-refs.mjs`: 変更不要の見込み（記法は導入・締めに閉じ、`extractBody` が捨てる）。R5.4 の「`--check` が再生成なしで通る」が機械的な証明になる。
  - `verify-content.mjs`: D-voice は引用ブロックを散文として見るので通る。追加で「導入・締めが台詞部品の記法で書かれている」（R10.2）と「表情名が一覧にある」（R10.3）の検査を足す。表情一覧は変換スクリプトと同じ 1 か所（export）から import する（口調判定の一本化と同じ規律）。
  - `verify-static.mjs`: 変更不要（`src` 実在検査が画像・CSS を拾う）。
  - `verify-search.mjs`: 台詞にしか出ない語で検索して該当章が返ることを 1 件足す（R10.5）。
  - 新規: 変換スクリプトと `*-test.mjs`（正常系・不正系）。
- `manual.yml` に加工ステップを 1 つ追加（着色の直後・索引再生成の前）。

---

## 4. 工数とリスク

| 領域 | 工数 | リスク | 根拠 |
| ---- | ---- | ------ | ---- |
| テーマ CSS（ライト・navy・印刷・狭幅） | M | Medium | mdBook の CSS 変数の上書きは定石だが、着色（hljs）背景・サイドバー・検索結果など触る面が広い。全ページで崩れが無いことの確認は目視が主体 |
| 台詞部品（記法・変換・CSS） | M | Medium | 変換は `highlight-html.mjs` の型で書ける。索引への入り方（§5）と `print.html` の扱いを設計で確定する必要 |
| 47 章の置き換え | S〜M | Low | 機械的。表情の付与に判断が要る（未確定事項 7）。生成物不変・口調検査の通過で機械確認できる |
| 表紙の扉・素材・出典 | S | Low | 画像の縮小とライセンス文の配置。`file://` と Pages の両方で相対参照が解けること |
| 検査・生成ツールの追従と自己テスト | S〜M | Low | 既存の型（依存ゼロ・自己テスト）に従う |
| **合計** | **L（1〜2 週間）** | **Medium** | 新しい外部技術は無いが、見た目の確認は自動化しにくく、章数（47）と検査面（6 ツール＋CI）が広い |

---

## 5. Research Needed（設計フェーズへ送る調査項目）

1. **mdBook 0.5.3 の検索索引が引用ブロック／生 HTML ブロック内のテキストを含めるか**。案 B（build-time 変換）なら `build-index.mjs` が読むのは mdBook の索引 JSON（`searchindex-*.js` の本文）であり、索引本文は mdBook が Markdown から抽出する。引用ブロックは入る見込みだが、HTML ブロックの扱いは要確認。入らない場合の補完策（`build-index.mjs` で加工後 HTML から本文を再抽出する）の可否も確認する。
2. **`print.html` への build-time 加工**: `highlight-html.mjs` は `**/*.html` を対象にするので `print.html` も加工される。同型で書けば印刷も対象になるが、`print.html` 内の相対パス（画像）が章と同じ基準で解けるかを確認する（`verify-static` が検査する）。
3. **hljs 配色と紙色の対比**: `highlight.css`（light）の `.hljs` 背景 `#f6f7f6` と各クラスの色を Claudia の紙色の上でどう見せるか。`pasta-manual-syntax-highlight` が決めたクラス写像（`scope-map.mjs`）は変えず、色だけ `additional-css` で上書きする前提で、各トークンの対比を確認する。
4. **Google Fonts の読み込み経路**: `head.hbs` 先頭への `<link rel="preconnect">`＋`<link rel="stylesheet">` 追記が `verify-search.mjs` のマーカー切り出しに影響しないことの確認（マーカー外の追記なので影響しない見込み）。`@import` との描画ブロック時間の比較。
5. **画像形式と縮小**: 顔アイコンを 72〜96px 表示に合わせて縮小した PNG と WebP の容量比較（`verify-static` は両方許可）。ダークテーマで縁が浮かないか（アルファの縁取り）。
6. **mdBook の CSS 変数一覧**（0.5.3 の `variables.css`）: 上書きすべき変数（`--bg`・`--fg`・`--sidebar-bg`・`--links`・`--quote-bg`・`--quote-border`・`--table-*`・`--searchresults-*`・`--inline-code-color` 等）の棚卸。
7. **狭幅のブレークポイント**: mdBook の既定（サイドバー折りたたみの幅）と、台詞部品を縦積みにする幅の整合。
8. **表情の付与基準**（未確定事項 7）: 47 章の導入・締めの台詞を一覧し、文言から表情を機械的に割り当てられるかを確認する（例: 「フンッ」→照れ怒り、「おめでとう」→にっこり、「熱く参りましょう」→高笑い）。

---

## 6. 設計フェーズへの推奨

- **推奨の方向**: 台詞部品は「Markdown の引用ブロックに話し手と表情を置く記法」＋「build-time の HTML 変換（`highlight-html.mjs` と同型・決定論・冪等・失敗で exit 1）」（§3.1 案 B）。テーマは `additional-css` 1 枚＋表紙の HTML（§3.2 案 A）。記法と表情一覧は 1 か所（Node モジュール）で定義し、変換・検査・自己テストが import する（口調判定の一本化と同じ規律）。
- **先に確定すべき判断**（要件ディスカッション）: 本文の字体（未確定事項 1）、ダークテーマの範囲（2）、Web フォントの配信元（3）、話し手の拡張性（4）。これらは CSS の量と記法の形を左右する。
- **機械的に証明できる完了条件**（設計で検査に落とす）: `gen-skill-refs.mjs --check` が再生成なしで通る（R5.4）、`verify-content` が全章の台詞部品を確認する（R10.2）、`verify-static` が追加資材の実在を確認する（R10.4）、`verify-search` が台詞語で該当章を返す（R10.5）、CI `manual.yml` が全段通る（R8.7）。
- **部分出荷の禁止**（brief.md 制約）: テーマ・台詞部品・47 章の置き換え・表紙・素材・ツール追従を 1 つの完成形で main に入れる。タスク分割は許すが、公開（main マージ）は完成形で行う。
