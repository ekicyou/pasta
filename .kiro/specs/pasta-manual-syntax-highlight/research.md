# Gap Analysis — pasta-manual-syntax-highlight

> 既存コードベース（`book/` mdBook 基盤）と要件（build-time approach C）の溝を分析し、設計フェーズの意思決定材料を提供する。最終決定は行わず、選択肢と研究項目を提示する。

## 1. 分析サマリ

- **アプローチは要件で確定済み**（build-time 焼き込み / approach C）。本分析は「C を既存 `book/tools/` 基盤の中でどう構造化するか」に焦点を当てる。
- **強い再利用資産**: bigram 索引ツール（`book/tools/bigram-index/`）が確立した「mdbook build 後段で `book/book/` 生成物を後処理し、ハッシュ付きファイルをグロブ解決して in-place 書き換え」パターンと、`theme/head.hbs` の「グローバル代入を `defineProperty` で捕捉して挙動を差し替える」実証済みパターンが、本機能の HTML 後処理層・book.js 中和層にそのまま転用できる。
- **最大のギャップ**: `book/tools/` は現状 **npm 依存ゼロ・package.json 無し**（elasticlunr は mdBook 同梱物を `require`）。approach C の `vscode-textmate` + `vscode-oniguruma` 導入は、book/ 配下に初の npm 依存・lockfile・`manual.yml` への `npm ci` 工程追加を要する。
- **中和は必須かつ実現可能**: `book.js`（`book-c22b7243.js:173-202`）は全 `<code>` に無条件で `hljs.highlightBlock()` を適用し言語スキップ分岐が無い。head.hbs アクセサで `hljs.highlightBlock` を book.js 実行前に包み、pasta ブロックを対象外化する手法が最有力（要検証）。
- **テーマ統合は低リスク**: light=`highlight.css`・navy=`tomorrow-night.css` が標準 `hljs-*` クラスを着色。pasta の約 14–16 個のリーフスコープ → 標準 hljs クラスへの有限・安定マッピングで成立する見込み。

## 2. 現状調査（Current State）

### 2.1 ビルド基盤 `book/tools/`
- **構成**: 純 Node 20 ESM スクリプト群。`bigram-index/build-index.mjs`（検索索引再生成）、`drift-check.mjs`、`tutorial-check.mjs`、`verify-*.mjs`。各々 CLI 兼用（`import.meta.url` 判定で直接実行可）。
- **依存方針**: **package.json 無し・npm 依存ゼロ**。外部ライブラリ（elasticlunr）は mdBook 生成物を `createRequire` + `require(path.resolve(...))` で読む。`book/tools/` 初の npm 依存導入は新規パターン。
- **後処理パターン**（`build-index.mjs`）:
  - `resolveHashed(bookOutDir, /regex/)` でハッシュ付きファイルをグロブ解決（固定名禁止）。
  - 生成物を読み→加工→**in-place 上書き**。冪等・決定論的。失敗時 `process.exit(1)` でビルド中断。
  - 本機能の HTML 後処理（`book/book/**/*.html` の pasta ブロック→span 注入）はこの構造の鏡像で実装可能。

### 2.2 テーマ override `theme/head.hbs`
- mdBook v0.5.3 で**確実に効く唯一の override 足場**（`searcher.js` override は v0.5.3 で無視される実測あり、head.hbs はコメントに明記）。
- 既存実装は `window.elasticlunr` に `Object.defineProperty` のアクセサを仕掛け、UMD 代入の瞬間に `tokenizer` を差し替える。**`window.hljs` への同型アクセサで `highlightBlock` を包む中和に転用可能**。
- ブラウザは ESM import 不可ゆえロジックをインライン同梱し、正準ソース（`tokenize.mjs`）と逐語同期する規約。本機能で head.hbs にコードを足す場合も同じ「インライン同梱＋同期注記」規約に従う。

### 2.3 `book.js`（`book-c22b7243.js`）の再ハイライト
- `167-171`: `hljs.configure({ languages: [] })` で**自動言語検出は無効**。
- `173-202`: ヘッダ内 inline code を除く全 `<code>` を収集し、`hljs.highlightBlock(block)` を**無条件適用**（`language-pasta` を除外する分岐は無い）。`highlightBlock` は `textContent` を読み直して `innerHTML` を書き戻すため、事前注入 span を破壊する。
- `204-208`: 全 code に `hljs` クラスを付与（着色用ベース CSS の適用）。中和後も pasta ブロックにこのクラスは付くため、ベース背景＋自前 span の詳細色で成立する。
- **含意**: 中和しない限り file:// でも（book.js は file:// でも走る）色が消える。中和層は本機能の必須 seam。

### 2.4 テーマ CSS と hljs クラス
- `book.js:329-410`: テーマ切替で `#mdbook-highlight-css`（light）/ `#mdbook-tomorrow-night-css`（coal・navy・ayu系のダーク）/ `#mdbook-ayu-highlight-css` を出し分け。
- light=`highlight.css`、navy=`tomorrow-night.css`。両者は標準 hljs クラス（`hljs-comment`/`hljs-keyword`/`hljs-string`/`hljs-title`/`hljs-variable`/`hljs-type`/`hljs-number`/`hljs-built_in`/`hljs-attr` 等）を着色する。**マッピングは標準クラスへ写すことでテーマ CSS をそのまま流用できる**。

### 2.5 SSOT 文法 `editors/vscode/syntaxes/pasta.tmLanguage.json`
- `scopeName: source.pasta`、261 行、JSON 形式（vscode-textmate が直接ロード可能）。
- 主要リーフスコープ（→ 想定 hljs クラスは設計で確定）:

| TextMate スコープ | 構文要素 | 想定 hljs クラス（暫定） |
| --- | --- | --- |
| `comment.line.pasta` | 行コメント（`#`/`＃`） | `hljs-comment` |
| `keyword.other.marker.pasta` | マーカー（＊・&@$>%! 等） | `hljs-keyword` / `hljs-symbol` |
| `keyword.control.scene.pasta` | シーン名（global/local） | `hljs-title` / `hljs-section` |
| `keyword.control.pasta` | 呼び出し（＞） | `hljs-keyword` |
| `entity.other.attribute-name.pasta` | 属性名（＆） | `hljs-attr` |
| `markup.inline.raw.string.pasta` | 単語（＠） | `hljs-string` |
| `variable.other.pasta` / `variable.other.reference.pasta` | 変数定義/参照（＄） | `hljs-variable` |
| `entity.name.class.pasta` | アクター（％） | `hljs-type` / `hljs-title` |
| `entity.name.type.actor.pasta` | アクション行アクター | `hljs-type` |
| `entity.name.function.call.pasta` / `entity.name.function.cue.pasta` | 関数/キュー呼出 | `hljs-built_in` / `hljs-title` |
| `string.other.sakura-script.pasta` | さくらスクリプト | `hljs-meta` / `hljs-string` |
| `constant.character.escape.pasta` | エスケープ | `hljs-literal` / `hljs-subst` |
| `constant.numeric.pasta` | 数値 | `hljs-number` |
| `string.quoted.other.pasta` | 引用文字列 | `hljs-string` |
| `entity.name.tag.pasta` | タグ語 | `hljs-name` / `hljs-tag` |
| `punctuation.*` / `meta.*` | 区切り・コンテナ | 着色なし（プレーン） |

- 有限・安定（約 14–16 リーフ → 約 10 hljs クラス）。マッピング層は決定論的な純関数で実装可能。

### 2.6 公開パイプライン `manual.yml`
- 実行順: `mdbook build book` → `build-index.mjs`（bigram）→ `drift-check.mjs` → `tutorial-check.mjs` → `cargo test -p pasta_sample_ghost` → Pages upload → deploy。
- Node 20 セットアップ済み。**`npm ci` 工程は無い**（追加が必要）。
- 各工程の入出力は独立: bigram は `searchindex-*.js`、drift/tutorial は markdown/実ファイル、本機能は `*.html` を対象。HTML 後処理は他工程と**ファイル非競合**で、mdbook build 後段の任意位置に挿入可能。

## 3. 要件 → 資産マップ（ギャップタグ）

| 要件 | 既存資産 | ギャップ | タグ |
| --- | --- | --- | --- |
| R1 色分け表示 | 後処理パターン（bigram）、SSOT tmLanguage | TextMate ロード＋トークナイズ機構が無い（vscode-textmate 新規導入）／HTML 後処理ツール新規 | Missing |
| R2 light/navy 配色 | テーマ CSS（標準 hljs クラス着色）、テーマ切替（book.js） | スコープ→hljs クラス写像が無い／両テーマで着色クラスの網羅確認 | Missing / Unknown |
| R3 再ハイライト中和 | head.hbs アクセサ実証パターン、book.js 構造把握済み | hljs.highlightBlock 包み込みの実装と発火順序保証 | Missing / Unknown |
| R4 静的・オフライン | 純静的出力・file:// 両立の既存基盤 | build-time ツール依存（oniguruma WASM）を成果物に混入させない閉じ込め | Constraint |
| R5 SSOT 単一・drift 回避 | tmLanguage を SSOT として VSCode と共有 | 読み取り専用再利用の徹底（第2文法を作らない設計規律） | Constraint |
| R6 ロバストネス | bigram の「失敗時 exit 1／正常は継続」規約 | 未着色テキスト＝正常 vs ツール例外＝中断 の切り分け実装 | Missing |
| R7 パイプライン統合 | manual.yml（Node 20 済み） | `npm ci` 工程追加／HTML 後処理工程の挿入位置確定 | Missing |
| R8 受け入れ検証（初回一度） | 公開済み GitHub Pages（実 HTML） | 公開 HTML を対象にした一度限りの受け入れ確認手順 | Missing |

## 4. 実装アプローチ選択肢（approach C 内の構造化）

### Option A: 既存 `book/tools/` 後処理パターンの拡張（推奨）
- **構成**: `book/tools/highlight/`（仮）に新規 ESM ツールを追加し、bigram と同型に `book/book/**/*.html` をグロブ→pasta ブロックをデコード→`vscode-textmate` でトークナイズ→hljs クラス span を注入して in-place 書き換え。`theme/head.hbs` に hljs 中和ブロックを追記。`manual.yml` に `npm ci` と本工程を挿入。
- **互換性**: 既存工程とファイル非競合。head.hbs の既存 elasticlunr ブロックと独立に共存可能。
- **トレードオフ**:
  - ✅ 実証済みパターンの鏡像で学習コスト最小・保守一貫
  - ✅ 後処理は他工程と疎結合、挿入位置自由
  - ✅ 出力は純静的（要件 R4 を自然に満たす）
  - ❌ book/ 初の npm 依存を持ち込む（package.json/lockfile/`npm ci`）
  - ❌ head.hbs に中和ロジックが増え、book.js 内部構造への依存が生じる（Revalidation Trigger）

### Option B: mdBook preprocessor 方式
- **構成**: カスタム mdBook preprocessor を実装し、mdbook build 中に pasta ブロックを着色 HTML へ変換。
- **トレードオフ**:
  - ✅ ビルドと一体化し後処理工程が減る
  - ❌ preprocessor 出力後も book.js は再ハイライトするため**中和は依然必須**（B でも head.hbs 中和は不可避）
  - ❌ 既存の「book/book 後処理」思想から外れ、preprocessor プロトコル（stdin/stdout JSON 契約）の新規実装が必要
  - ❌ ハッシュ付き生成物・テーマ CSS との結合確認は結局必要で、A 比で利得が薄い

### Option C: ハイブリッド（後処理 A ＋ 中和を独立 seam として明示管理）
- 実質 A（A は既に head.hbs 中和を含む）。C として扱う価値があるのは「トークナイズ層／マッピング層／HTML 注入層／book.js 中和層」を**疎結合な 4 seam に分離**し、中和層を最も注意を要する結合点として独立検証する構造化方針。
- **トレードオフ**: ✅ 各 seam を個別に検証・差し替え可能、Revalidation Trigger を中和 seam に局所化／❌ 計画の粒度が上がる。

> **推奨**: Option A（= 中和を含む後処理拡張）を基本線とし、内部は Option C の 4 seam 分離で設計する。Option B は中和不可避・利得薄で非推奨。

## 5. 工数・リスク

| 作業領域 | 工数 | リスク | 根拠 |
| --- | --- | --- | --- |
| トークナイズ層（vscode-textmate + oniguruma WASM build-time ロード） | M | Medium | 新規 npm 依存・API 学習・WASM 初期化。文法は SSOT を読むだけで安定 |
| スコープ→hljs クラス マッピング層 | S | Low | 有限・安定な純関数。テーマ CSS 流用で配色定義不要 |
| HTML 後処理層（glob/エンティティ復号・再符号化/span 注入/冪等） | M | Medium | 後処理パターンは実証済みだが HTML エンティティ往復と冪等性に注意 |
| book.js 中和層（head.hbs で hljs.highlightBlock 包み込み） | S | Medium | 実証済みアクセサパターンだが発火順序と hljs 内部依存の検証要 |
| パイプライン結線（package.json・`npm ci`・工程挿入） | S | Low-Medium | Node 20 済み。book/ 初の npm 依存が唯一の新規性 |
| 受け入れ検証（公開 HTML・両テーマ・file://・一度限り） | S | Low | 恒常ゲート不要（要件 R8）。一度の手動確認手順を定義 |
| **総合** | **M（1 週間前後）** | **Medium** | 確立パターンへの統合だが、新規 npm 依存・book.js 内部結合・WASM build-time 化が不確実要素 |

## 6. 設計フェーズへの申し送り（Research Needed）

- **R-1 テーマ着色クラスの網羅確認**: mdBook v0.5.3 同梱 `highlight.css`（light）と `tomorrow-night.css`（navy）が実際に色を割り当てる `hljs-*` クラスの集合を実物で確認し、§2.5 の暫定マッピングが「両テーマで構文要素を判別可能」にする十分条件を満たすか検証する。判別に足りないスコープは近接クラスへ再割当て。なお保証対象は light/navy のみ（要件ディスカッション #3 で確定）。標準 hljs クラスを採用するため rust/coal/ayu でも各テーマ CSS により事実上着色される見込みだが、これらは要件・検証の対象外（保証しない副次効果）。
- **R-2 vscode-textmate / vscode-oniguruma の build-time 利用**: JSON 文法（plist でない）ロード、Registry/onigLib 初期化、Node からの oniguruma WASM 読込（`fs.readFile` of node_modules 内 .wasm）、ライセンス（双方 MIT）と runtime 依存ゼロを確認。トークナイズ結果（行ごとの scope スタック）から「最深スコープ→hljs クラス」を取り出す方針を固める。
- **R-3 book.js 中和の堅牢化**: `window.hljs` アクセサで `highlightBlock` を包む方式が book.js の `codeSnippets` IIFE より確実に先行するか（head 段階での仕込み順序）を検証。代替として mdBook v0.5.3 が `theme/book.js` override を honor するかも実測し、堅牢な方式を確定。pasta ブロックの識別子（`language-pasta` クラス or 専用マーカー）を決める。
- **R-4 HTML エンティティ往復と冪等性**: mdBook のコードブロックエスケープ規則（`&lt; &gt; &amp;` 等）に対し、デコード→トークナイズ→span 付き再エンコードでテキスト等価を保つ方法と、再実行時の二重 span 防止（textContent 基準で常に再生成 等）を確定。
- **R-5 book/ の npm 依存管理**: package.json の配置（`book/` 直下が有力）、lockfile コミット、`manual.yml` への `npm ci` 挿入位置、node_modules を CI インストールに限定（コミットしない）方針、ローカル開発手順（bigram は不要だった npm が必要になる旨の周知）を決める。

## 7. 次のステップ

- 本ギャップ分析を踏まえ `/kiro-spec-design pasta-manual-syntax-highlight` で技術設計へ。
- 設計では §4 推奨（Option A ＋ C の 4 seam 分離）を起点に、§6 の R-1〜R-5 を Design 内で解消する。

---

## 8. 設計フェーズ Discovery 結果（R-1〜R-5 の解消）

Light Discovery（Extension）として実施。外部技術は一次情報（MS 公式 npm/GitHub・highlight.js docs）で確認。

### 8.1 技術スタック確定（R-2 解消）
- **vscode-textmate 9.3.2（MIT・外部 npm 依存ゼロ）**: `parseRawGrammar(content, filePath)` で JSON 文法をロード。**JSON は `filePath` に `*.json` を必ず渡す**（省略すると plist 誤判定の既知バグ）。`Registry({ onigLib, loadGrammar })` → `loadGrammar('source.pasta')` → 行単位で `grammar.tokenizeLine(line, ruleStack)`。戻り `tokens: {startIndex,endIndex,scopes:string[]}`（`scopes` は外→内、**末尾が最特定**）と次行用 `ruleStack`。
- **vscode-oniguruma 2.0.1（MIT）**: `loadWASM(fs.readFileSync('node_modules/vscode-oniguruma/release/onig.wasm').buffer)` を await し `createOnigScanner/createOnigString` を `onigLib` として供給。**WASM は build-time のみ**で動作し、生成 HTML には一切含めない（要件 4.1/4.3 を満たす）。
- **落とし穴**: `tokenizeLine` は必ず 1 行ずつ／改行（`\r`/`\n`）を行文字列に含めない／`ruleStack` を行間で引き回す（lua ブロック等の begin/end が壊れる）。
- **既製ツール非存在**: 「TextMate→highlight.js 互換クラス出力」の確立ツールは無い。Shiki は色を style に焼き込む方式で本用途（クラス焼き込み＋テーマ CSS 流用）に不適。→ vscode-textmate 直接利用の自前変換が定石（Option A を裏づけ）。

### 8.2 スコープ→hljs クラス写像（R-1 解消・実 CSS 色グループ検証済み／設計ディスカッション #1 で是正）
- 実物 CSS の**色グループ**を精査（`highlight-*.css` light / `tomorrow-night-*.css` navy）。「クラスが別」でも「色が同じ」では判別できないため、**両テーマで確実に着色され、かつ色が衝突しないクラス**のみ採用する。
- **実 CSS 色グループ**:
  - light（6色）: gray=comment / red=variable,attr,tag,name / orange=number,built_in,literal,type,params / green=string,symbol / blue=title,section / purple=keyword
  - navy（実質6色）: gray=comment / red=variable,attr,tag / orange=number,built_in,literal,params,constant / green=string,**name** / aqua=title,section / blue=function / purple=keyword
- **判明した不具合（修正済み）**: `hljs-symbol` は navy で**無色**（`.ruby .hljs-symbol` のみ着色）／`hljs-section` と `hljs-title` は両テーマ同色／`hljs-name` は navy で `hljs-string` と同色。→ これらを避け、確実着色6スロット（gray/purple/aqua/orange/green/red）へ集約。
- **確定マッピング（是正後・6色グループ）**（最特定スコープ＝末尾優先、`.` 区切り前方一致）:

| pasta スコープ | hljs クラス | 色（light / navy） |
| --- | --- | --- |
| `comment.line.pasta` | `hljs-comment` | gray / gray |
| `keyword.other.marker.pasta` / `keyword.control.pasta` | `hljs-keyword` | purple / purple |
| `keyword.control.scene.pasta`（シーン名） | `hljs-title` | blue / aqua |
| `entity.name.class.pasta`（アクター ％） | `hljs-title` | blue / aqua |
| `entity.name.type.actor.pasta`（アクション行アクター） | `hljs-title` | blue / aqua |
| `entity.name.function.call.pasta` / `entity.name.function.cue.pasta` | `hljs-built_in` | orange / orange |
| `constant.character.escape.pasta` | `hljs-literal` | orange / orange |
| `constant.numeric.pasta` | `hljs-number` | orange / orange |
| `markup.inline.raw.string.pasta`（単語 ＠） | `hljs-string` | green / green |
| `string.other.sakura-script.pasta` | `hljs-string` | green / green |
| `string.quoted.other.pasta` | `hljs-string` | green / green |
| `variable.other.pasta` / `variable.other.reference.pasta` | `hljs-variable` | red / red |
| `entity.other.attribute-name.pasta` | `hljs-attr` | red / red |
| `entity.name.tag.pasta` | `hljs-tag` | red / red |
| `punctuation.*` / `meta.*` / `source.pasta` | （なし＝プレーン） | — |

- **6色で確実判別**: コメント(gray)／マーカー・呼出(purple)／名前系=シーン・アクター(aqua-blue)／呼出・定数(orange)／文字列系=単語・さくらスクリプト(green)／参照系=変数・属性・タグ(red)。学習上重要な6区分を両テーマで確実に色分け（要件 1.2/1.3/2.1/2.2）。同系要素（シーン⇔アクター、単語⇔さくらスクリプト）は同色グループへ収れん（許容）。

### 8.3 book.js 中和の確定（R-3 解消）
- `book-c22b7243.js`: `codeSnippets()` は**即時実行 IIFE**。`hljs.configure({languages:[]})`（自動検出オフ）後、全 `<code>`（ヘッダ除く）に `hljs.highlightBlock(block)` を無条件適用。未登録言語 `pasta` は plaintext 扱いで innerHTML をエスケープ再生成 → 事前 span 破壊。
- **中和方式**: `theme/head.hbs` に、既存 elasticlunr と同型の `Object.defineProperty(window,'hljs',…)` アクセサを追加。highlight-*.js が `window.hljs` を代入した瞬間に `highlightBlock`（および新名 `highlightElement`）を**ラップ**し、引数要素が `language-pasta` クラスを持つ場合は**原処理を呼ばずスキップ**。head.hbs は `<head>`（最早）で走り、hljs バンドル・book.js はそれ以降に読まれるため発火順序が保証される（elasticlunr で実証済み）。
- **識別子**: 既存 `class="language-pasta"` をそのまま利用（追加マーカー不要）。book.js が後段で付与する `hljs` クラス（ベース CSS）は中和後も付くため、自前 span の色＋ベース背景が成立（要件 3.1/3.2）。
- **回帰テスト（設計ディスカッション #4 で追加）**: 中和ロジックを正準モジュール `neutralizer.mjs` に置き head.hbs はその逐語ミラー（既存 `tokenize.mjs`↔head.hbs と同方式・同期注記）。**jsdom による build-time ユニットテスト**で book.js の `highlightBlock` 模倣→pasta span 生存・他言語委譲を検証。公開サイトを叩く恒常ゲートではなくユニットテストゆえ要件 8.4/6.3 に非抵触。jsdom を book/ devDependency（キャレット範囲）に追加。

### 8.4 HTML エンティティ往復・冪等性（R-4 解消）
- 実 HTML: `<pre><code class="language-pasta">…</code></pre>`。全角マーカー（＠＊％）は非エスケープ、`<` `>` `&` `"` のみ実体参照化（lua ブロック内に出現し得る）。
- **方針**: 各ブロックの**ソース文字列をタグ除去＋実体参照デコードで復元** → 行分割 → トークナイズ → クラス span 付きで**実体参照を再エスケープ**して emit。
- **冪等性（要件 6.5）**: ソース復元時に既存 `<span>` をタグ除去するため、**再実行は常にプレーンテキストから再生成**＝二重 span を生まない。加えて処理済みは検出して同一結果を保証。
- **決定論（要件 6.4）**: 同一入力 HTML → 同一トークン列 → 同一バイト出力（bigram ツールと同契約）。

### 8.5 npm 依存管理（R-5 解消・設計ディスカッション #3 で既存規約へ整合）
- **先例**: `editors/vscode/package.json` は既に `vscode-textmate@^9.0.0`／`vscode-oniguruma@^2.0.0` を devDependencies に持ち（プロジェクトは既にこの2ライブラリへ依存）、`package-lock.json` をコミット・`node_modules` を gitignore。npm audit ゲートは無し。
- **方針（editors/vscode 規約へ整合）**: `book/package.json` も同じ `^9`/`^2`（キャレット範囲）、`package-lock.json` コミット、`node_modules` 非コミット、CI は `npm ci`。**決定論（6.4）は lockfile＋npm ci で担保**（キャレットでも CI は lockfile から厳密インストール）。完全固定や npm audit ゲートはプロジェクト規約に無く採らない（6.3）。
- `manual.yml`: `Setup Node` 後に `npm ci`（book）を追加 → `mdbook build` → **pasta ハイライト後処理** → bigram → drift → tutorial。ハイライト（`*.html`）と bigram（`searchindex-*.js`）は**対象ファイル非競合**で順序独立だが、可読性のため mdbook build 直後・bigram 前に置く（要件 7.1/7.2/7.3）。
- ローカル開発: bigram は不要だった `npm ci`（book）が本機能で必要になる旨を周知（手順ドキュメント反映）。

### 8.6 Synthesis（採否の確定）
- **Adopt**: vscode-textmate/vscode-oniguruma 直接利用（自前変換）。build-vs-adopt は「文法ロード＋トークナイズ＝adopt、スコープ写像＋HTML 焼き込み＋中和＝build（自前）」。
- **入れ子 lua の着色（設計ディスカッション #2 で採用）**: pasta 文法は `lua-code-block` に `source.lua` を `include` しない（改変不可）。よって vendor lua 文法（MIT/permissive・read-only）を追加ロードし、pasta パスで検出した `meta.embedded.block.lua.content` 区間を**二段トークナイズ**して着色する。pasta SSOT は単一のまま（lua は別言語の独立文法）。ScopeClassMapper は前方一致ゆえ lua スコープもそのまま6色へ写る（追加写像不要）。
- **一般化**: スコープ写像は単一の純関数（`scope-map`）に集約し SSOT を一点化（drift 回避・要件 5）。
- **簡素化**: 追加マーカーを設けず `language-pasta` を識別子に流用。テーマ別配色は CSS 流用で写像を交差集合に閉じ、色焼き込みを完全回避（要件 2.3）。
- **却下**: Option B（mdBook preprocessor）＝中和不可避で利得薄、highlight.js 第2文法新設＝drift（要件 5 違反）。

## 9. 次のステップ（更新）
- 設計確定（design.md）→ `/kiro-spec-tasks` でタスク分解。設計は §8 の確定事項を Boundary Commitments / Components に反映する。
