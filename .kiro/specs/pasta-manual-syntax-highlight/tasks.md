# 実装計画 — pasta-manual-syntax-highlight

- [x] 1. Foundation: npm マニフェスト・vendor 文法・テスト基盤
- [x] 1.1 book/ の npm マニフェストとロックファイル整備
  - book/package.json を新設し devDependencies に vscode-textmate ^9 / vscode-oniguruma ^2 / jsdom を追加（editors/vscode 規約に整合）
  - package-lock.json をコミット対象に、node_modules は gitignore
  - book/ で npm ci が成功し node_modules が生成される（観察可能）
  - _Requirements: 4.3, 7.1_
- [x] 1.2 vendor lua TextMate 文法の取得・配置
  - scopeName source.lua の lua.tmLanguage.json（JSON 形式）を book/tools/highlight/grammars/ に読み取り専用 vendor
  - ライセンス（MIT/permissive）と出典を LICENSE/注記として併置
  - source.lua をロード可能な JSON 文法ファイルが配置されライセンスが明記される（観察可能）
  - _Requirements: 1.1, 1.3, 5.1_

- [x] 2. Core: トークナイズと写像
- [x] 2.1 (P) スコープ→hljs クラス写像（6色・文法非依存）
  - TextMate スコープ末尾優先・. 区切り前方一致で両テーマ確実着色の6色クラスへ写像、未マッチは無着色（null）
  - pasta/lua 両スコープを同一テーブルで処理（hljs-symbol/section/name は使わない）
  - 純関数で他コンポーネントに依存しないため 2.2 と並列可（境界 ScopeClassMapper のみ・ファイル非競合）
  - 代表スコープが期待クラス・未マッチが無着色を返すユニットテストが通る（観察可能）
  - _Requirements: 1.2, 1.3, 2.1, 2.2, 2.3, 6.1_
  - _Boundary: ScopeClassMapper_
- [x] 2.2 (P) pasta+lua トークナイザと入れ子 lua の二段トークナイズ
  - oniguruma WASM 初期化後に pasta・lua 文法をロードし、行単位トークナイズで ruleStack を行間引き回し
  - pasta パスで meta.embedded.block.lua.content 区間を検出し lua 文法で再トークナイズして lua スコープへ差し替え
  - 文法は読み取り専用。文法ファイル不在時は例外送出
  - Foundation の 1.2（lua 文法）のみに依存し 2.1 とは別境界ゆえ並列可（境界 PastaTokenizer・ファイル非競合）
  - fixture テキストをトークナイズし入れ子 lua が lua スコープになる・文法不在で例外となるユニットテストが通る（観察可能）
  - _Requirements: 1.1, 1.3, 5.1, 5.2, 5.3, 6.2_
  - _Boundary: PastaTokenizer_
  - _Depends: 1.2_

- [x] 3. Core: HTML 焼き込み後処理
- [x] 3.1 language-pasta ブロックの span 焼き込み後処理
  - book/book 配下 HTML をグロブし language-pasta ブロックのみ抽出、エンティティ復号→トークナイズ→クラス span 付与→再エスケープして in-place 書き換え
  - いずれのスコープにも属さない区間は無着色のまま素通し（プレーン）
  - 非 pasta・無指定ブロックは不変。文法/依存欠落時は非ゼロ終了＋診断
  - fixture HTML の pasta ブロックに hljs span が付与され非 pasta が不変・依存欠落で exit 1 となる（観察可能）
  - _Requirements: 1.1, 1.4, 4.1, 4.3, 6.1, 6.2_
  - _Boundary: HtmlHighlighter_
  - _Depends: 2.1, 2.2_
- [x] 3.2 決定論・冪等の担保
  - ブロックのソーステキストをタグ除去＋実体参照デコードで復元し常に再生成（再実行で二重 span を生まない）
  - 同一入力 HTML に対し同一バイト列を出力
  - 同一入力2回実行で出力バイトが一致し再実行で span が増えない・エンティティ往復が等価となるユニットテストが通る（観察可能）
  - _Requirements: 4.1, 6.4, 6.5_
  - _Boundary: HtmlHighlighter_
  - _Depends: 3.1_

- [x] 4. Core: クライアント再ハイライト中和
- [x] 4.1 (P) 中和ロジック正準モジュールと head.hbs ミラー
  - window.hljs アクセサで highlightBlock/highlightElement をラップし language-pasta 要素をスキップする正準モジュールを実装
  - head.hbs に逐語ミラー＋同期注記をインライン同梱（既存 elasticlunr ブロックと独立共存）
  - 境界 ClientNeutralizer は build-time の 2/3 系と別ファイル・別責務ゆえ Core 内で並列可
  - head.hbs に中和ブロックが入り正準モジュールと一致する（観察可能）
  - _Requirements: 3.1, 3.2, 4.2_
  - _Boundary: ClientNeutralizer_
- [x] 4.2 jsdom による中和ユニットテスト
  - jsdom で book.js の highlightBlock 呼出を模擬し pasta ブロックの事前 span が生存・他言語ブロックは原処理へ委譲を検証
  - 公開サイトを叩く恒常ゲートではなく build-time ユニットテスト
  - jsdom テストが通る（観察可能）
  - _Requirements: 3.1, 3.2_
  - _Boundary: ClientNeutralizer_
  - _Depends: 4.1, 1.1_

- [x] 5. Integration: 公開パイプライン結線
- [x] 5.1 manual.yml にハイライト工程を結線
  - Setup Node 後に npm ci（working-directory book）を追加、mdbook build 直後・bigram 前に highlight 後処理工程を挿入
  - 既存 drift-check/tutorial-check/cargo test ゲートを不変に保ち恒常ハイライトゲートは増やさない
  - ワークフローが npm ci→mdbook build→highlight→bigram→既存ゲートの順で構成され既存ステップが保持される（観察可能）
  - _Requirements: 6.3, 7.1, 7.2, 7.3_
  - _Boundary: PipelineWiring_
  - _Depends: 3.1, 3.2, 4.1_
- [x] 5.2 ローカル統合確認（mdbook build→highlight）
  - 実 book/book を生成しハイライト後処理を実行、pasta ブロックが6色 span で着色され冪等であることを確認
  - light/navy 両テーマ CSS が span を着色し file:// で色が保持されることをローカル確認
  - 実 HTML 出力で pasta 構文が6区分の色で判別可能・再実行で不変となる（観察可能）
  - _Requirements: 1.1, 1.3, 2.1, 2.2, 4.2_
  - _Depends: 5.1_

- [x] 6. Validation: 公開 HTML の受け入れ検証（一度限り）
- [x] 6.1 GitHub Pages 公開 HTML の受け入れ検証
  - 公開された HTML の pasta ブロックに hljs span が存在し book.js 再ハイライト後も保持されることを確認
  - light/navy 両テーマと file:// で各構文要素が相互に判別可能であることを確認
  - 初回受け入れ時のみ実施し恒常ゲート化しない
  - 公開 HTML で6色判別・中和保持・両テーマ・file:// が確認された記録が残る（観察可能）
  - _Requirements: 8.1, 8.2, 8.3, 8.4_
  - _Depends: 5.1_
  - _Done: main マージ（squash f15aaa6）→ manual.yml デプロイ完了後、公開 GitHub Pages HTML で 8.1〜8.4 を実証（GO）。8.1 span実在・8.2 デプロイ済み中和でbook.js再ハイライト後もspan生存（jsdom実機模擬 pre/post一致）・8.3 公開テーマCSS同一ハッシュで6色判別・8.4 恒常ゲート非追加。記録: acceptance-verification.md_

## Implementation Notes

- 全テスト・ツールは `book/tools/highlight/*.mjs`。実行は `node book/tools/highlight/<name>-test.mjs`（依存ゼロ自前 assert ハーネス）。Node 25 では `process.exit()` 即時呼出が libuv の async-handle assertion で落ちるため、テストハーネスは `process.exitCode` 方式（pass=0/fail=1）を採用（既存 tokenizer-test.mjs/highlight-html-test.mjs 等）。
- `book/` の npm 依存はキャレット範囲（`^9`/`^2`/jsdom `^25`）＋ `package-lock.json` コミット＋ `node_modules` は gitignore（editors/vscode 規約整合）。CI/ローカルとも `npm ci`（working-directory `book`）が前提。
- oniguruma WASM は `book/node_modules/vscode-oniguruma/release/onig.wasm` を `fs.readFileSync(...).buffer` で `loadWASM`。build-time のみ・公開成果物非混入。
- 入れ子 lua は pasta 文法を改変せず `meta.embedded.block.lua.content` 区間を vendor lua 文法で二段トークナイズ（ScopeClassMapper は前方一致ゆえ lua スコープも追加コードなしで 6 色へ写る）。
- `book/book` は gitignore。ローカル統合確認・受け入れ検証は再生成成果物に対して実施（コミット対象外）。
- ハイライト着色済み HTML は `book.js` の無条件再ハイライトで破壊されるため、`theme/head.hbs` の hljs 中和（`neutralizer.mjs` の逐語ミラー）が必須。head.hbs を更新したら `neutralizer.mjs` と同期すること（同期注記あり）。
- タスク 6.1（公開 HTML 受け入れ）は GitHub Pages デプロイ後の一度限り手動工程。main マージ→`manual.yml` デプロイ完了後に `acceptance-verification.md` のチェックリストで実施する。
