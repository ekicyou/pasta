# Brief: manual-print-page-removal

起点: 2026-10-16、`manual-print-media-refs` をマージした後の確かめで見つかった。公開サイトの印刷用のページ（`https://ekicyou.github.io/pasta/print.html`）を開くと、ブラウザが固まる。開発者が確かめ、「これは無理です」と判断した。全章を 1 枚にした印刷用のページはやめる、と開発者が決めた（下の Approach）。

## Problem

マニュアルの読者が、右上の印刷のボタンを押すか `print.html` を開くと、ブラウザが固まる。読むことも印刷することもできない。

- mdBook が作る `print.html` は、全章（今は 60 章）を 1 枚につないだページである。手元の出力では約 1 万 5 千行あり、印刷すると 700 ページを超える。
- `print.html` は、読み込みが終わると自分で印刷の画面を開く。末尾に次のスクリプトがある（手元の出力の 15,250〜15,252 行）。
  ```js
  window.addEventListener('load', function() { window.setTimeout(window.print, 100); });
  ```
  巨大なページを描いた直後に印刷のプレビューを作り始めるので、固まる。印刷の画面が開いている間は、ページのスクリプトも止まる（2026-10-16 の確かめで、ページのスクリプトが 2 回とも 45 秒で時間切れになった）。
- 1 章だけ刷りたい読者は、章のページでブラウザの印刷を使えば足りる。章のページは自分では印刷の画面を開かない。mdBook の印刷用の CSS が、目次やボタンを隠す。

## Current State

2026-10-16 の main（`manual-print-media-refs` の PR#120 のマージの後）で、ソースを読んで確かめた。

- **設定**: `book/book.toml` に `[output.html.print]` の節は無い。mdBook の既定で、`print.html` と右上の印刷のボタンが作られる。15 行目のコメントは「全章と print.html に読み込まれる」と書いている。
- **`print.html` のためだけにあるもの**（`manual-print-media-refs` が足した）:
  - `book/tools/print-media-refs.mjs` と自己テスト `book/tools/print-media-refs-test.mjs`。`print.html` のメディア要素の参照を章の場所から直す
  - `.github/workflows/manual.yml` の段「Fix print page media references」（111〜117 行）と、先頭の説明の 11・14・18 行
  - `book/AUTHORING.md` の「CI と同じ順」の一覧の 472 行
  - `book/tools/verify-static.mjs` の `ensureBuilt` からの呼び出し（31 行の import・210〜216 行）、`analyze` のコメント（256〜257 行）、自己テスト (e)（535〜545 行あたり）
  - steering: `.kiro/steering/tech.md` 201 行のパイプラインの段、`.kiro/steering/structure.md` 232 行の `book/tools` の説明
- **`print.html` を前提にしているほかの検査**（`verify-static.mjs`）:
  - 全章と並べて `print.html` を検査するページの一覧に入れている（303 行の説明・417・451・457 行の自己テスト・696〜713 行のテーマ CSS・JS・台詞部品・テーマメニューの検査）
  - 旧版の自己テスト (a2) のフィクスチャが `classic/print.html` を作る（413 行）
  - `print.html` が無くなると、これらは数が合わずに落ちるか、ページの一覧から外す直しが要る
- **旧版（`classic/`）**: `manual.yml` の段「Build classic manual snapshot」は、固定のコミット（`CLASSIC_REF`）の `book/` を旧版のツールでそのまま作る。旧版も `print.html` と印刷のボタンを持ち、同じく固まる。段は 193〜195 行で `classic/print.html` が在ることを確かめている（無ければ失敗）。
- **`print.html` を名前で扱うが、直す必要が無さそうなもの**: `book/tools/talk/talk-html.mjs`（出力の `*.html` をすべて変換する。`print.html` が無くても動く）、`talk-html-test.mjs` の (D)（自前のフィクスチャ）、`link-check-test.mjs` の 1013・1097 行（`print.html` を読まないことの確かめ）、`book/theme/claudia.css`・`claudia.js` の先頭のコメント（「全章と print.html に読み込まれる」）。どれを直すかは要件で決める。
- **完了した spec**: `manual-print-media-refs` の要件 1.3（公開サイトの `print.html` で動画が再生できること）は、マージの後の確かめとして残っていた。本 spec で `print.html` をやめるので、確かめる対象が無くなる。

## Desired Outcome

- 公開する新しい版のマニュアルに `print.html` が無く、右上の印刷のボタンも無い。どのページを開いてもブラウザが固まらない。
- 章のページは、今までどおりブラウザの印刷で 1 章ずつ刷れる。
- `print.html` のためだけにあった道具・段・一覧の行・説明が残っていない。`verify-static.mjs` は `print.html` を前提にせず、全ページの `poster` の検査（`manual-print-media-refs` の要件 3.3）は残す。
- `file://` のオフライン閲覧と、`site-url = "/pasta/"` での公開は、今までどおり両立する。

## Approach

`book/book.toml` に次を書き、mdBook に `print.html` と印刷のボタンを作らせない（2026-10-16 開発者の決定。案 A）。

```toml
[output.html.print]
enable = false
```

あわせて、`print.html` のためだけにあった `print-media-refs` の道具・自己テスト・CI の段・`AUTHORING.md` の行を消し、`verify-static.mjs` と steering を `print.html` の無い出力に合わせる。

採らなかった案:
- **印刷のボタンだけ隠して `print.html` を残す**: URL を知っている人が開くと、やはり固まる。誰も開かないページのために、書き換えの段を持ち続けることになる。
- **今のまま残す**: 開くと固まるページが、ボタン 1 つで開く。

## Scope

- **In**:
  - `book/book.toml` の `[output.html.print] enable = false` と、15 行目のコメントの直し
  - `book/tools/print-media-refs.mjs`・`print-media-refs-test.mjs` の削除、`manual.yml` の段と説明の直し、`book/AUTHORING.md` の一覧の行の削除
  - `book/tools/verify-static.mjs` を `print.html` の無い出力に合わせる（`ensureBuilt` の呼び出し・ページの一覧・自己テスト）
  - steering（`tech.md`・`structure.md`）のパイプラインと道具の説明の直し
  - 旧版の `print.html` の扱い（下の「決めること」）
- **Out**:
  - mdBook そのものの変更・上流への報告
  - 章のページの印刷の見た目の作り込み（印刷用の CSS の追加など）
  - 章の本文（`book/src/`）。`print.html` を案内している箇所があれば、直す範囲に入れるかを要件で決める
  - 完了した `manual-print-media-refs` の spec の文書の書き換え（記録として残す）

## 決めること（要件で決める）

- **旧版の `print.html`**: 旧版も開くと固まる。旧版の生成の段で、作った後に `classic/print.html` を消すか（193〜195 行の在ることの確かめも外す）、旧版は固定のコミットのまま触らないか。消すと、旧版の章のページの印刷のボタンが行き先の無いリンクになる。推奨は、要件の議論で両方の具体例を見てから決める。
- **古い URL**: 公開サイトの `/pasta/print.html` は 404 になる。mdBook の `[output.html.redirect]` で `index.html` へ送るか、404 のままにするか。
- **作られていないことの検査**: 将来 `enable = false` が外れて `print.html` が戻ったときに、`verify-static.mjs` が気づくようにするか（「`print.html` が無いこと」を確かめる）。
- **`print.html` を名前で扱う、直す必要の無さそうな箇所**（Current State の最後の項目）を、どこまで直すか。

## Boundary Candidates

- マニュアルの設定（`book/book.toml`）
- ビルド後の段と道具（`manual.yml`・`book/tools/print-media-refs*.mjs`・`book/AUTHORING.md` の一覧）
- 出力の検査（`book/tools/verify-static.mjs`）
- 旧版の生成の段（`manual.yml` の「Build classic manual snapshot」）

## Out of Boundary

- 台詞の変換・着色・検索の索引の各ツール（`print.html` が無くても動く）
- `link-check.mjs`（`print.html` を読まない）
- テーマの見た目

## Upstream / Downstream

- **Upstream**: mdBook 0.5.x の `[output.html.print] enable`（CI は v0.5.3、手元は v0.5.4）。完了した `manual-print-media-refs`（PR#120）が足したものを取り下げる。同 spec の design の Revalidation Triggers が「`book.toml` で `[output.html.print] enable = false` にしたとき（`print.html` が無いので本ツールが失敗する）」と書いている。道具を残したまま設定だけ変えると、CI が落ちる。
- **Downstream**: 無し。

## Existing Spec Touchpoints

- **Extends**: 無し（`manual-print-media-refs` は完了済み。その要件 1.3 の確かめは、本 spec で対象が無くなる）
- **Adjacent**: `manual-shell-guide`・`getting-started-screenshots`（マニュアルの章を触る。本 spec は章の本文を触らない）、`pasta-check-dic-validate`（`manual.yml` を触る場合がある。同じウェーブに置くなら、`manual.yml` はどちらか一方だけが触る）

## Constraints

- **作成ツールは問題があれば止める**: 道具を消す前に設定を変えると CI が落ちる。設定・道具・段・検査を 1 つの変更で入れる。
- **相対パスを保つ**: `file://` と `site-url = "/pasta/"` の両立を崩さない。
- **CI と手元を同じ順にする**: `manual.yml`・`verify-static.mjs` の `ensureBuilt`・`AUTHORING.md` の一覧をそろえる。
- 規模の目安: 4〜6 タスク。要件定義は Opus。
