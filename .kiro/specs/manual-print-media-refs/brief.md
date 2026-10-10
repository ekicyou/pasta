# Brief: manual-print-media-refs

起点: 2026-10-10 の棚卸でバックログの「print.html の動画の参照切れ」から起票した。

## Problem

マニュアルの印刷用のページ（全章を 1 枚にまとめた `print.html`）で、「開発支援アクション」の章の実演動画が再生できない。章のページでは再生できる。

検査ツールは、この 1 件を例外として見逃している。そのため自動検査は合格のままで、壊れていることが表に出ない。同じ書き方の動画を別の章に足しても、検査は気づかない。

## Current State

2026-10-10 の main（`add05022`）を、ソースと検査ツールを読んで確かめた。手元でのビルドはしていない。

- **章の書き方**: `book/src/debug/dev-actions.md` 19〜21 行が、生の HTML で動画を置いている。19 行が `<video … src="media/dev-actions-demo.mp4">`、20 行が再生できない環境向けの代わりのリンク `<a href="media/dev-actions-demo.mp4">` である。
- **動画の実体**: `book/src/debug/media/dev-actions-demo.mp4`（約 3.1 MB）。出力では `debug/media/` に置かれる。
- **壊れ方**: 章のページ（`debug/dev-actions.html`）では、`media/…` が `debug/media/…` を指すので再生できる。`print.html` は出力の根にあるので、同じ `media/…` が根の `media/` を指し、そこに動画は無い。
- **検査ツールの例外**: 出力の検査（`book/tools/verify-static.mjs` 245〜250 行）は、`print.html` を読むときだけ、`<video …>` と `<source …>` の開きタグ（`<` から最初の `>` まで）を消してから参照を集める。消えるのはこの 2 種類のタグの中の `src` だけである。
- **代わりのリンクは壊れていない**: `<video>` の中に書いた `<a href>` は、上の例外では消えない。`print.html` の中でも参照として検査され、合格している。
  - 例外のコメント（247〜248 行）は、「mdBook は `<a href>` を章の場所に合わせて書き換えるが、`<video>`・`<source>` の `src` は書き換えない」と書いている。
  - 確かめ方: 検査ツールのコードを読んで、`<a href>` が検査の対象に残ることを確かめた。あわせて、main の `add05022` に対するマニュアルの自動検査が成功していることを見た。手元でビルドした `print.html` は見ていない。
- **ロードマップの記述の直し**: バックログの項目は「動画と代わりのリンクの両方が `print.html` で `debug/media/` を指せない」「mdBook は生の HTML の属性を書き換えない」と書いているが、どちらも半分だけ正しい。壊れているのは `<video>`（と `<source>`）の `src` だけである。
- **ほかの生の HTML**: マニュアルのソース（`book/src/`）で `<video>` を使うのは、この 1 か所だけである。`<source>`・`<audio>` は無い。
- **ビルドの流れ**: マニュアルの自動検査（`.github/workflows/manual.yml` 86〜105 行）は、ビルドの後に出力を 3 回書き換える。コードの着色（93〜94 行の `book/tools/highlight/highlight-html.mjs`）、台詞の部品への変換（99〜100 行の `book/tools/talk/talk-html.mjs`）、検索の索引の作り直し（104〜105 行）の順である。出力の検査は 140〜141 行で、ビルド済みの出力を見る（`--no-build --self-test`）。
- **検査ツールが自分でビルドする場合**: `verify-static.mjs` を `--no-build` なしで動かすと、自分でビルドし、台詞の変換も自分で行う（177〜209 行の `ensureBuilt`。30 行で `talk-html.mjs` の `transformDir` を取り込んでいる）。
- **検査ツールの自己テスト**: `verify-static.mjs` の自己テスト（383〜528 行）に、`print.html` の動画の参照を扱う場合は無い。
- **着せ替え前の版**: `classic/` は、固定のコミットのソースとその版のツールで作る（`manual.yml` 59 行・165〜179 行）。同じ不具合を持つ。出力の検査は `classic/` を見ない（`verify-static.mjs` 96〜111 行）。
- **公開の場所**: `book/book.toml` 13 行が `site-url = "/pasta/"` を指定している。リンクは相対のまま出し、`file://` での閲覧と両立させている（同 11〜12 行のコメント）。

## Desired Outcome

- 章のページと `print.html` の両方で、動画が再生でき、代わりのリンクも働く。
- `verify-static.mjs` の `print.html` の例外を外しても、検査に合格する。
- 章の本文の変更は、動画の参照だけにとどまる。
- `node book/tools/gen-skill-refs.mjs --check` と `node book/tools/link-check.mjs` に合格する。
- `file://` のオフライン閲覧と、`site-url = "/pasta/"` での公開の両方が、今までどおり働く。

## Approach

ビルドの後で、`print.html` だけを書き換える。`<video>`・`<source>` の `src` を、出力の根から見た場所（`debug/media/dev-actions-demo.mp4`）に直す。章のページとソースは変えない。

- **ほかの案を採らない理由**:
  - 動画を出力の根へ移し、章の側を `../media/` と書く案（ロードマップが挙げた案）: 章のページでは正しくなるが、`print.html` では `../media/` が出力の根の外を指す。mdBook は `<video>` の `src` を書き換えないので、深さの違う 2 つのページの両方で同じ相対パスが正しくなる置き場所は無い。
  - 絶対パス（`/pasta/debug/media/…`）にする案: `file://` で開くと壊れる。出力の検査も、根からの絶対パスを誤りとして落とす（`verify-static.mjs` 133 行・258〜263 行）。
  - 動画を根の `media/` にも写す案: HTML を触らずに済むが、約 3.1 MB を二重に公開する。次点とする。
- **置き場所**: 台詞の変換の直後（`manual.yml` 99〜100 行の後）、出力の検査より前に 1 段を足す。出力を書き換える段がここに並んでいる。`book/tools/` に小さなツールを 1 本と、その自己テストを置く。`*-test.mjs` という名前のテストは、自己テストの段（`manual.yml` 150〜158 行）が自動で拾う。
- **検査ツールの側**:
  - `verify-static.mjs` の例外（247〜249 行）を外す。
  - 自分でビルドする場合（`ensureBuilt`）にも、同じ書き換えを足す。台詞の変換と同じ形にする。足さないと、`--no-build` なしで動かしたときに必ず落ちる。
  - 自己テストに「`print.html` の `<video>` の `src` が壊れていたら見つける」場合を足す。
- **直せないときは止める**: 写し先が見つからない・1 つに決まらないときは、警告で続けずに失敗させる。

## Scope

- **In**:
  - `print.html` の `<video>`・`<source>` の `src` を直す、ビルド後のツールとその自己テスト
  - `.github/workflows/manual.yml` への 1 段の追加
  - `book/tools/verify-static.mjs` の例外の削除、自分でビルドする場合の追従、自己テストの追加
  - `book/src/debug/dev-actions.md` の動画の参照（直す必要があるときだけ）
- **Out**:
  - 着せ替え前の版（`classic/`）の同じ不具合。固定のコミットから作るので直さない
  - mdBook そのものの変更・上流への報告
  - 動画の撮り直し・形式の変更
  - 手元での確かめ方の一覧（`book/AUTHORING.md` 459〜468 行）の書き換え。下の Constraints

## Boundary Candidates

- 出力を書き換えるツール（`print.html` の参照の直し）
- 出力の検査（例外の削除と自己テスト）
- 自動検査の段の追加

## Out of Boundary

- 着色・台詞の変換・検索の索引の各ツール
- 章の本文（動画の参照のほか）
- マニュアルの見た目（テーマ）

## Upstream / Downstream

- Upstream: なし。
- Downstream: なし。以後、章に生の HTML で動画を足しても、出力の検査が `print.html` の側まで見る。

## Existing Spec Touchpoints

- 完了した manual-claudia-theme: 台詞の変換の段と、検査ツールの自己テスト、着せ替え前の版の生成を入れた。このspecは同じ並びに 1 段を足す。
- getting-started-story-guide: `book/AUTHORING.md` と `book/tools/verify-content.mjs`・`tutorial-check.mjs` を持つ。段階表を照合に使う場合は `manual.yml` の対象パスも触る見込みである（同 brief の「触るファイル」）。ウェーブが別なので重ならない。

## Constraints

- **ウェーブ**: 2026-10-10 のウェーブで `.github/workflows/manual.yml` と `book/tools/verify-static.mjs` を触るのは、このspecだけである。`book/tools/verify-content.mjs`・`tutorial-check.mjs`・`book/AUTHORING.md`・`book/src/getting-started/` は触らない（getting-started-story-guide が持つ）。
- **2 つの閲覧の形を両立させる**: `file://` のオフライン閲覧と、`site-url = "/pasta/"` での公開。参照は相対パスのまま保つ。
- **章の本文**: 変えるのは動画の参照だけ。
- **問題があれば止める**: 書き換えに失敗したら、公開の前に止める。
- **着せ替え前の版**: 触らない。新しいツールを `classic/` に当てない。

## 2026-10-10 棚卸の測定（main add05022）

- **触るファイル**:
  - `book/tools/` の新しい小さなツール 1 本とその自己テスト（または既存の段への組み込み）。
  - `.github/workflows/manual.yml`（206 行。1 段の追加と、先頭の流れの説明 8〜17 行）。
  - `book/tools/verify-static.mjs`（694 行。例外 247〜249 行、`ensureBuilt` 177〜209 行、自己テスト 383〜528 行）。
  - `book/src/debug/dev-actions.md`（163 行。19〜21 行だけ。触らずに済む見込み）。
  - 1,000 行に近いファイルは無い。
- **規模**: 2〜3 タスク（書き換えのツールと自己テスト 1、検査ツールの例外の削除と自己テスト 1、自動検査の段の追加と通しの確認 1）。
- **先に要るもの**: なし。ほかの未完了の spec とファイルは重ならない。
- **種別**: バグ（印刷用のページで動画が再生できず、検査がそれを見逃している）。
- **要件定義のモデル**: Opus（直し方は 1 つに絞れていて、開発者の判断の分かれ道が少ない）。
- **要件定義の議題**:
  1. 書き換えが、参照の属する章の場所をどう知るか。`print.html` の中に章の境目が残っているかを、実際の出力で確かめる。残っていなければ、出力の中から同じ末尾のパスを持つファイルを探し、1 つに決まるときだけ直す形にする。
  2. 新しい小さなツールにするか、既存の段（台詞の変換など）に組み込むか。新しい段を足すと、`book/AUTHORING.md` 459〜468 行の「CI と同じ順」の手順の一覧が 1 行足りなくなる。`AUTHORING.md` はこのウェーブでは触れないので、getting-started-story-guide へ申し送るか、既存の段に組み込んで一覧を変えずに済ませるかを決める。
  3. 直す対象を `<video>`・`<source>` の `src` に限るか、`poster`・`<audio>`・`<track>` まで広げるか。今あるのは `<video>` の 1 件だけである。
  4. 直す対象が 1 件も無いときに、何もせずに成功とするか。動画を将来はずしたときに、ツールが理由なく落ちないようにする。
  5. 次点の案（動画を根の `media/` にも写す）と比べて、書き換えを採ることの確認。
- **見つけた穴・古くなった記述**:
  - ロードマップのバックログの項目は、代わりのリンクも壊れていると書いているが、壊れているのは `<video>` の `src` だけである（上の Current State）。
  - `verify-static.mjs` の例外のコメント（248 行）は「印刷ビューで動画は用途外」と書いている。このspecはその判断を改め、`print.html` でも動画が働くことを条件にする。
  - `book/src/introduction.md` 6 行にも生の HTML の `<img>` があるが、出力の根にあるページなので、`print.html` と同じ場所から参照が解決する。対象にしない。

## 申し送り（getting-started-story-guide より・2026-10-10）

本文の spec が完了した。

- **`manual.yml` のコメントが古い**: 114 行目あたりの「`book/src/getting-started/first-ghost.md` の ```pasta 成果物ブロックが…」は、もう合わない。`first-ghost.md` は消え、`tutorial-check.mjs` は段の章 12 枚（`01-boot.md`〜`12-lua.md`）を、同じ名前の辞書とそれぞれ照合する。`manual.yml` はこのウェーブでこの spec が持つので、触るついでにコメントを直す（手順そのものは変えなくてよい）。
- **`book/AUTHORING.md` はもう空いている**: 議題 2 の「`AUTHORING.md` はこのウェーブでは触れない」は、本文の spec の完了で外れた。行番号はずれた（第 8 節が足された。「CI と同じ順」の記述は 463 行目あたり）。
- **`link-check.mjs` は、本の中のリンクの見出し（`#…`）を見ない**: 別に起票した。この spec の範囲ではない。
