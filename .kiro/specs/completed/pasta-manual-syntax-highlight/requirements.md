# Requirements Document

## Introduction

利用者マニュアル（[https://ekicyou.github.io/pasta/](https://ekicyou.github.io/pasta/)、mdBook v0.5.3 製）の `*.pasta` コードブロックは、現在シンタックスハイライト無し（無色）で表示される。mdBook 同梱の highlight.js が `pasta` 言語定義を持たないためで、入門・文法章でコードが読みづらく、VSCode 拡張で得られる色分け体験とのギャップが、読者（ゴースト作者・初学者）の学習効率を損なっている。

本機能は、マニュアルの `*.pasta` コードブロックを VSCode 拡張と同等の色分けで表示する。色付けは **build-time（mdbook build 工程）** で実行し、TextMate 文法（pasta ハイライトの SSOT）を再利用して highlight.js 互換クラスの span を HTML に焼き込む。出力は純静的 HTML/CSS/JS のみとし、`file://` オフライン閲覧を維持し、公開サイトにランタイム依存（WASM 等）を持ち込まない。pasta 文法の正本は TextMate 文法ただ一つに保ち、二重管理（drift）を生まない。

選定アプローチ・検証済み事実・制約の詳細は [brief.md](./brief.md) を参照。

## Boundary Context

- **In scope**:
  - build-time での pasta コードブロック字句解析（TextMate 文法ロード＋トークナイズ）
  - TextMate スコープ → highlight.js 互換クラスへの写像
  - mdBook クライアント側（`book.js`）の無条件再ハイライトに対する中和（事前 span の保持）
  - light/navy 両テーマでの配色（mdBook 既存 hljs テーマ CSS を流用）
  - 公開パイプライン（`manual.yml`）への色付け工程の組み込み（mdbook build 後・bigram 索引再生成と整合する順序）
  - 公開出力 HTML の初回受け入れ検証（一度限り）
- **Out of scope**:
  - VSCode 拡張の TextMate 文法（`editors/vscode/syntaxes/pasta.tmLanguage.json`）の改変・拡張（読み取り再利用のみ）
  - pasta 以外の言語のハイライト改善
  - 公開サイトへのランタイム依存（WASM・フレームワーク等）の持ち込み
  - エディタ/LSP のハイライト改善（VSCode 拡張・`pasta_lsp` の領分）
  - マニュアル本文コンテンツの執筆（`pasta-user-manual` の領分・完了済み）
  - 毎ビルドで走る恒常的なハイライト検証ゲートの新設（検証は初回受け入れ時の一度限り）
  - light/navy 以外のテーマ（rust / coal / ayu）での配色保証・検証（標準 hljs クラス採用により事実上着色される見込みだが、要件・検証の対象外）
- **Adjacent expectations**:
  - **Upstream（依存・読み取り再利用）**: `editors/vscode/syntaxes/pasta.tmLanguage.json`（scopeName `source.pasta`、約 32 スコープ）を pasta ハイライトの SSOT として読み取り再利用する。`pasta-user-manual`（完了・アーカイブ）が確立した mdBook 基盤・`book/tools/` build-time Node パターン・`manual.yml` 公開パイプラインの延長上に構築する。
  - **Revalidation Trigger**: 本機能は mdBook v0.5.3 同梱物（ハッシュ付き `book.js` / highlight.js 実装）に依存する。mdBook を更新した場合、再ハイライト中和とテーマ配色が維持されるかの再検証を要する。

## Requirements

### Requirement 1: pasta コードブロックの色分け表示

**Objective:** ゴースト作者・初学者（読者）として、マニュアルの `*.pasta` コードブロックの各構文要素が、相互に判別できる色分けで表示されてほしい。そうすれば文法を視覚的に把握でき、学習効率が上がる。

> **「同等」の定義（識別性の同等）**: 本要件で言う「VSCode 拡張と同等」は、**各構文要素が相互に判別可能であること（識別性）が同等**であることを指す。配色そのものは mdBook の hljs テーマ CSS（light/navy）に従い、VSCode のテーマ配色と同一色である必要はない（Requirement 2.3 の色焼き込み回避と整合）。

#### Acceptance Criteria

1. When mdbook build 後の出力 HTML に `language-pasta` クラスを持つコードブロックが含まれるとき, the マニュアルビルド shall それらのブロックを TextMate 文法でトークナイズし、各トークンに highlight.js 互換クラスを持つ span を付与する。
2. The マニュアルビルド shall マーカー記号（＊・＠・％ 等）・シーン名・アクター・関数呼び出し・さくらスクリプト・変数・コメントの各構文要素を、相互に判別可能な異なるクラスの span に分類する。
3. When 読者がブラウザで pasta コードブロックを閲覧するとき, the マニュアルサイト shall 各構文要素を相互に判別可能な配色で表示する（配色は mdBook テーマ CSS に従い、VSCode と同一色であることは要しない）。
4. While コードブロックが `language-pasta` 以外（他言語・無指定）であるとき, the マニュアルビルド shall 当該ブロックに pasta 用の色付けを適用せず、mdBook 既定の表示を維持する。

### Requirement 2: light/navy 両テーマでの配色

**Objective:** 読者として、ライトテーマでもダークテーマでも pasta コードが適切に配色されてほしい。そうすればどちらの閲覧環境でもコードが読みやすい。

#### Acceptance Criteria

1. While マニュアルサイトのテーマが light（既定）であるとき, the マニュアルサイト shall pasta コードブロックを当該テーマで判読可能な配色で表示する。
2. While マニュアルサイトのテーマが navy（ダーク）であるとき, the マニュアルサイト shall pasta コードブロックを当該テーマで判読可能な配色で表示する。
3. The マニュアルビルド shall pasta 色付けに mdBook 既存の hljs テーマ CSS を流用し、テーマ別の独自配色定義の新規焼き込み（Shiki 方式の色直書き）を行わない。

### Requirement 3: クライアント側再ハイライトの中和

**Objective:** 読者として、ページを開いた後も pasta コードの色分けが保持されていてほしい。そうすれば一瞬色が付いて消える、無色になる、といった不具合に遭遇しない。

#### Acceptance Criteria

1. When 読者がページを読み込み `book.js` のクライアント側ハイライト処理が実行されるとき, the マニュアルサイト shall pasta コードブロックに焼き込まれた事前 span を破壊せず保持する。
2. While `book.js` が他言語・無指定コードブロックを従来どおりクライアント側ハイライトしている間, the マニュアルサイト shall pasta ブロック以外の既存ハイライト挙動を変更しない。

### Requirement 4: 静的・オフライン制約

**Objective:** プロジェクト運用者として、公開サイトがサーバー不要の純静的成果物のままであってほしい。そうすれば GitHub Pages 公開と `file://` オフライン閲覧が維持され、配布が軽量に保たれる。

#### Acceptance Criteria

1. The マニュアルビルド shall pasta 色付けの成果物を純静的な HTML/CSS/JS のみとして出力し、公開サイトにランタイム WASM・フレームワーク等の実行時依存を持ち込まない。
2. When 読者が公開成果物を `file://` で（ローカル・オフラインで）開くとき, the マニュアルサイト shall pasta コードブロックの色分けを保持して表示する。
3. The マニュアルビルド shall TextMate 文法ロードや字句解析に用いるツール依存（WASM を含む）を build-time に閉じ込め、公開成果物には含めない。

### Requirement 5: SSOT 単一・drift 回避

**Objective:** プロジェクト運用者として、pasta 文法の正本が一つに保たれてほしい。そうすれば VSCode 拡張とマニュアルの色分けがズレず、二重管理の保守負担が生じない。

#### Acceptance Criteria

1. The マニュアルビルド shall pasta ハイライトの語彙的根拠を TextMate 文法（`editors/vscode/syntaxes/pasta.tmLanguage.json`）ただ一つから取得し、highlight.js 用の第 2 文法を新設しない。
2. The マニュアルビルド shall TextMate 文法ファイルを読み取り専用で再利用し、その内容を改変しない。
3. When TextMate 文法が将来更新されるとき, the マニュアルビルド shall 更新後の文法に基づいて pasta 色付けを生成し、追従が一元的に行われる。

### Requirement 6: 色付け処理のロバストネス

**Objective:** プロジェクト運用者として、色付け処理が予期せぬ入力で静かに壊れたり、逆に正常な未着色テキストでビルドを止めたりしないでほしい。また再現可能なビルドと安全な再実行が保証されてほしい。そうすればマニュアル公開が安定する。

#### Acceptance Criteria

1. While TextMate トークナイズ結果に、いずれのスコープにも属さないプレーンテキスト区間が含まれるとき, the マニュアルビルド shall 当該区間を色付けせず素のテキストとして出力する（これは正常動作でありビルドを失敗させない）。
2. If TextMate 文法ファイルの読み込みに失敗する、字句解析ツールが例外を送出する、または必須の build-time 依存が欠落しているとき, then the マニュアルビルド shall 非ゼロ終了で失敗し、原因を示す診断メッセージを出力する。
3. The マニュアルビルド shall 上記以外に、毎ビルドで走る恒常的なハイライト品質検証ゲートを新設しない。
4. When 同一の入力 HTML に対して色付け処理を実行するとき, the マニュアルビルド shall 常に同一の出力（同一バイト列）を生成する（決定論的）。
5. When 既に色付け済みの HTML に対して色付け処理を再実行するとき, the マニュアルビルド shall 二重の span を生成せず、初回実行と同一の結果を維持する（冪等）。

### Requirement 7: 公開パイプラインへの統合

**Objective:** プロジェクト運用者として、色付け工程が既存の公開パイプラインに矛盾なく組み込まれてほしい。そうすれば従来の検索索引・ドリフト検出と整合した形でマニュアルが公開される。

#### Acceptance Criteria

1. The 公開パイプライン（`manual.yml`）shall pasta 色付け工程を mdbook build の後段に実行する。
2. When 色付け工程と日本語 bigram 索引再生成の双方が同一ビルドで実行されるとき, the 公開パイプライン shall 両者が互いの出力を破壊しない順序で実行する。
3. The 公開パイプライン shall pasta 色付け工程の導入後も、既存の drift-check・tutorial-check 等のゲートを従来どおり成立させる。

### Requirement 8: 公開出力 HTML の受け入れ検証（初回・一度限り）

**Objective:** プロジェクト運用者として、実際に GitHub Pages へ公開された HTML で色分けが意図どおり機能していることを一度確認したい。そうすれば本機能が本番環境で成立していることを確証できる。

#### Acceptance Criteria

1. When 本機能の受け入れ検証を行うとき, the 検証作業 shall 実際に GitHub Pages へ出力された（公開された）HTML を対象として、pasta コードブロックに highlight.js 互換クラスの span が付与されていることを確認する。
2. The 検証作業 shall 公開 HTML 上の pasta コードブロックが、`book.js` の再ハイライト実行後も色分けを保持していることを確認する。
3. The 検証作業 shall light テーマと navy テーマの双方、および `file://` での閲覧で、pasta コードブロックの各構文要素が相互に判別可能な配色で表示されていること（識別性の同等）を確認する。
4. The 検証作業 shall 上記確認を本機能の受け入れ時に一度限り実施するものとし、毎ビルドで自動実行する恒常ゲートとしては要求しない。
