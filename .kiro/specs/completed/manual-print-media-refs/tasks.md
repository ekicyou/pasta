# Implementation Plan

- [x] 1. 印刷用のページのメディアの参照を直すツール
- [x] 1.1 `print.html` のメディア要素の参照を章の場所から直す書き換えの本体を作る
  - 対象の表（`video` の `src`・`poster`、`audio`・`source`・`track` の `src`）の属性を、属性名の前が空白のときだけ拾う（`data-src` などは拾わない）。空・`#…`・スキーム付き・`//…`・`/…` の値は直さず、実在も見ない
  - `print.html` を読んだ直後に対象の数を数え、0 件なら `toc.html` も章のページも読まずに何も書かず成功する。出力のフォルダ・`print.html` が無ければ、無いもののパスを示して失敗する
  - `print.html` の `<main>` の中を改ページの印で分け、`toc.html` の章のリンク（`.html` で終わり、スキーム・`//`・`#`・`?` を含まず、`/` で始まらないもの）を出現の順に対応させる。部分の数と章の数が違えば失敗にする
  - 章ごとに、部分の対象の並び（要素・属性・値）が章のページの `<main>` の並びと一致するかを確かめる。`print.html` の値が未修正の値か、直した後の値のどちらかなら一致とみなす（2 回目の実行で何も変わらない）
  - 値は章のフォルダと値のパスの部分をつないで正規化し、`?`・`#` の残りを保つ。直した後のパスが出力に無い・出力の根の外へ出るときは失敗にする
  - 問題は全件集めて 1 回で投げ、そのときは何も書かない。問題が無く、置き換えた文字列が元と違うときだけ `print.html` に 1 回書く。対象の属性の値のほかは 1 バイトも変えない
  - 確かめ用の出力は、`mdbook build book` の後に台詞の変換（`talk-html.mjs book/book`）を当てて作る（このタスクの中で作る。3.1 を待たない）
  - 完了の状態: その出力に対して呼ぶと `print.html` の `media/dev-actions-demo.mp4` が `debug/media/dev-actions-demo.mp4` になり、`{ targets: 1, fixed: 1 }` を返し、2 回目は `fixed: 0` で `print.html` が変わらない
  - _Requirements: 1.1, 1.4, 1.5, 1.6, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 6.2_

- [x] 1.2 書き換えをコマンドとして実行できるようにする
  - 出力のフォルダを引数に取り、既定は `book/book`（台詞の変換のツールと同じ形）
  - 成功で exit 0 と `print-media-refs: fixed N media reference(s) in print.html (M target(s))` の 1 行（0 件でも同じ形）、失敗で exit 1 と標準エラーに `print-media-refs failed: …` と問題の全件
  - 完了の状態: `node book/tools/print-media-refs.mjs book/book` が件数の 1 行を出し、存在しないフォルダを渡すと exit 1 で無いフォルダを表示する
  - _Requirements: 2.5, 2.6, 2.8_

- [x] 1.3 書き換えの自己テストを作る
  - 台詞の変換の自己テストと同じ型（依存ゼロの確かめ・一時フォルダ・最後に終了コード）で、mdBook 0.5.4 の形を写した最小の章のページ・`toc.html`・`print.html` を組み立てる
  - 深さの違う章（根・1 段下・2 段下）、`poster`・`source`・`track`・`audio`、`../` を含む値の正規化、同じ名前の動画を持つ 2 つの章がそれぞれの章のファイルへ直ること
  - 対象外（`<a href>`・`<img src>`・外部 URL・`#…`・`/…`・`data:`・本文・ほかのファイル）が 1 バイトも変わらないこと、`?`・`#` 付きの値と付け直された `id` があっても直ること
  - 対象 0 件で成功（`toc.html` が無くても）、直せない 5 つの場合（写し先なし・根の外・数の違い・並びの違い・章のページなし）で全ファイルが変わらず文面に章と参照が出ること、出力のフォルダ・`print.html` が無いと失敗すること、2 回目で `fixed 0`、CLI の終了コードと出力
  - 完了の状態: `node book/tools/print-media-refs-test.mjs` が全件合格で exit 0 になり、名前が `*-test.mjs` なので CI の自己テストの段が定義を変えずに拾う
  - _Requirements: 2.9, 2.10_

- [x] 2. 出力の検査・CI・執筆規約への組み込み
- [x] 2.1 出力の検査が `print.html` のメディア要素と `poster` まで見るようにする
  - `print.html` だけメディア要素の開きタグを消していた例外を外し、ほかのページと同じ規則で参照の実在を確かめる
  - 参照を集める属性に `poster` を足す（`\b` のまま。全ページに効く）
  - 自分でビルドする形では、台詞の変換に続けて書き換えを呼んで件数を表示し、失敗なら `FATAL: print-media-refs の書き換えに失敗しました: …` で exit 1 にする
  - 自己テストに、コピーの `print.html` へ架空の `<video src poster>` を差し込んで 2 件報告される場合と、`grammar/index.html` へ架空の `poster` を差し込んで報告される場合を足し、旧版のフィクスチャに `poster` を足して報告されないことを確かめる。先頭の使い方のコメントを直す
  - 完了の状態: 書き換え済みの出力に `--no-build --self-test` が合格し、書き換え前の出力に `--no-build` を当てると `print.html` の `media/dev-actions-demo.mp4` で失敗する
  - _Boundary: StaticVerifier_
  - _Depends: 1.1, 1.2_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

- [x] 2.2 (P) マニュアルの CI に書き換えの段を足し、説明と起動の条件を直す
  - 「Render talk components」の直後に「Fix print page media references」の段を足し、目的・対象・失敗で公開前に止まること・旧版に使わないことをコメントに書く。旧版の段と自己テストの段は動かさない
  - `push` と `pull_request` の `paths` に `README.md`・`SOUL.md` を足す
  - 先頭の説明を今の段の並びと検査の内容に合わせ、チュートリアルの照合の段の説明を「入門の章 12 枚と同じ名前の辞書の両方向の逐語照合」に直す（段の動作は変えない）
  - 完了の状態: `grep -n` で、段の名前が「Render talk components」→「Fix print page media references」→ 索引の再生成の順に並び、`paths` の 2 か所に `README.md`・`SOUL.md` があることを確かめられる（あれば actionlint か Python の `yaml` でも読めることを見る）
  - _Boundary: ManualWorkflow_
  - _Depends: 1.2_
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7_

- [x] 2.3 (P) 執筆規約の「CI と同じ順」の一覧に書き換えを足す
  - 台詞の変換の行の次に、書き換えのコマンドの行を 1 行、コメントの桁をそろえて足す
  - 完了の状態: `AUTHORING.md` の一覧が CI と同じ並び（台詞の変換の後・出力の検査の前）で書き換えを含む
  - _Boundary: AuthoringList_
  - _Depends: 1.2_
  - _Requirements: 5.1_

- [x] 3. 通しの確かめ
- [x] 3.1 手元で CI と同じ順に出力を作り、全検査を通す
  - `AUTHORING.md` の一覧のとおりに出力を作り（着色の段は `book/node_modules` が無ければ飛ばし、CI で確かめる旨を記す）、書き換えが `fixed 1` を出すことを見る
  - 旧版（`classic/`）の生成は固定のコミットから作り、本 spec で変えないので手元では走らせず、CI で確かめる
  - スキルの生成物の照合・リンク検証・チュートリアルの照合・出力の検査（`--no-build --self-test`）・検索の検査・コンテンツの検査・`book/tools` の全自己テストを実行する
  - `--no-build` なしの出力の検査が、ビルド → 台詞の変換 → 書き換え → 検査の順で合格することを見る
  - `book/src/` に差分が無いこと、章のページ（`debug/dev-actions.html`）の動画の参照が変わっていないことを見る
  - 完了の状態: 上の検査がすべて合格し、`git diff --stat` に `book/src/` が出ない
  - _Requirements: 3.4, 5.2, 6.1, 6.3_

- [x] 3.2 印刷用のページの動画を `file://` で開いて確かめる
  - 3.1 で作った出力の `print.html` を `file://` で開き、「開発支援アクション」の章の動画が再生でき、代わりのリンクから同じ動画が開けることを見る。章のページの動画とリンクも今までどおり働くことを見る
  - 公開サイト（`https://ekicyou.github.io/pasta/print.html`）での同じ確かめは、マージの後の確かめとして申し送る
  - 完了の状態: `file://` で `print.html` の動画の再生と代わりのリンクが働くことを確かめた。要件 1.3 はこのタスクでは満たさず「申し送り（未確認）」として報告に残す
  - _Requirements: 1.2, 1.3, 6.2_

## Implementation Notes
- 1.1: `print.html` の `<main>` の外に対象があれば失敗の問題にする（数えるだけで黙って成功しないため。レビューの指摘）。1.3 の自己テストの「直せない場合」にこの場合も足す
- 3.2: 内蔵ブラウザの枠は `file://` を開けないので、headless Edge（`--allow-file-access-from-files`）で `print.html` と章のページを iframe に読み込む使い捨ての確かめページを出力に置いて調べ、確かめた後に消した。結果は両方とも `currentSrc` が `debug/media/dev-actions-demo.mp4`・`readyState=4`・長さ 66 秒で、代わりのリンクの先も読めた。要件 1.3（公開サイト）はマージ後の確かめとして申し送る（未確認）
