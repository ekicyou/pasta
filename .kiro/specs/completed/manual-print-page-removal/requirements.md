# Requirements Document

## Project Description (Input)
マニュアルの印刷用のページ（全章を 1 枚につないだ `print.html`）は、開くとブラウザが固まる。`book/book.toml` に `[output.html.print] enable = false` を書いて mdBook に `print.html` と右上の印刷のボタンを作らせず、`print.html` のためだけにあった `print-media-refs` の道具・自己テスト・CI の段・`AUTHORING.md` の一覧の行を消し、`verify-static.mjs` と steering を `print.html` の無い出力に合わせる。章のページは今までどおりブラウザの印刷で 1 章ずつ刷れる。詳細は brief.md を参照。

## Introduction

公開しているマニュアルの読者が、右上の印刷のボタンを押すか `print.html` を開くと、ブラウザが固まる。`print.html` は全章（今は 60 章・約 1 万 5 千行・印刷で 700 ページ超）を 1 枚につないだページで、読み込みが終わると自分で印刷の画面を開くためである。1 章だけ刷りたい読者は、章のページでブラウザの印刷を使えば足りる。

本 spec は、新版と旧版（`classic/`）の両方のマニュアルから全章の印刷用のページと印刷のボタンを無くし、`print.html` のためだけにあった道具・CI の段・手引きの行・説明を取り除き、出力の検査を `print.html` の無い出力に合わせる。完了した `manual-print-media-refs`（PR#120）が足したものを取り下げる spec である。

brief の「決めること」は、次のように決めた（要件の議論で覆してよい）。

- **旧版の `print.html`**: 旧版でも作らせない（要件 2）。旧版も開くと固まるためである。作った後に消す案は、旧版の章のページに行き先の無い印刷のボタンが残るので採らない。固定のコミットの原稿とツールは変えず、生成のときの設定だけで止める（手元の mdBook v0.5.4 で、生成の時に設定を上書きすれば `print.html` もボタンも作られないことを確かめた）。
  - 例: 旧版の `https://ekicyou.github.io/pasta/classic/print.html` は無くなり、旧版の章のページの右上にも印刷のボタンが出ない。旧版の章の本文・テーマ・検索は今までどおり。
  - 引き換え: 旧版の章のページをブラウザで刷ると、目次とメニューも一緒に刷られる（下の「印刷用の規則」の理由による。旧版は固定のコミットのまま触らないので、規則を足せない）。旧版は見比べのための固定の版なので、章の印刷の見た目は保証しない（要件 2.3）。
- **印刷用の規則（2026-10-16 の設計で分かった）**: mdBook は、全章の印刷用のページを止めると、章のページへ読み込む印刷用の規則（目次・メニュー・前後のボタンを隠し、本文を紙幅に広げる）も一緒に読み込まなくなる（手元の v0.5.4 で、章のページから印刷用の CSS の参照と `css/print-*.css` が消えることを確かめた）。要件 1.3 を保つため、新版ではその規則をテーマに持たせる（Boundary Context の In scope）。
  - 例: 新版の「文法」の章をブラウザで刷ると、今までどおり左の目次も上のメニューも前後の矢印も出ず、本文だけが紙幅いっぱいに刷られる。
- **古い URL**: `/pasta/print.html` は転送せず、見つからないページのままにする（要件 1.4）。転送すると転送用の `print.html` が出力に残り、「`print.html` が無いこと」と両立しない。読者には mdBook の 404 のページ（目次と検索がある）が出る。
- **作られていないことの検査**: 入れる（要件 4.3〜4.5）。設定が外れて `print.html` が戻ると、開くと固まるページが黙って公開されるためである。
- **`print.html` を名前で扱う箇所**: 「`print.html` に読み込まれる」と事実を述べている説明（設定とテーマとツールの先頭の説明）は直す（要件 3.6）。自前のフィクスチャとして `print.html` という名前を使う自己テストと、`print.html` を読まないことを確かめる自己テストは、`print.html` の無い出力でも正しいので直さない（Boundary Context）。

## Boundary Context

- **In scope**:
  - 新版のマニュアルに全章の印刷用のページと印刷のボタンを作らせないこと
  - 旧版のマニュアルに全章の印刷用のページと印刷のボタンを作らせないこと
  - `print.html` のためだけにあった道具（`book/tools/print-media-refs.mjs`）・その自己テスト・CI の段・`book/AUTHORING.md` の一覧の行の削除
  - 出力の検査（`book/tools/verify-static.mjs`）を `print.html` の無い出力に合わせることと、`print.html` と印刷のボタンが無いことの検査
  - CI のパイプラインの説明・steering（`tech.md`・`structure.md`）・設定とテーマとツールの先頭の説明の直し
  - 全章の印刷用のページを止めると mdBook の章のページから無くなる印刷用の規則（目次・メニュー・前後のボタンを隠し、本文を紙幅に広げる）を、新版のテーマに持たせること（今の章の印刷を保つためだけ）
- **Out of scope**:
  - mdBook そのものの変更・上流への報告
  - 章のページの印刷の見た目の作り込み（今までの章の印刷を超える新しい見た目）
  - 旧版の章のページの印刷の見た目（旧版は固定のコミットのまま触らない）
  - 章の本文（`book/src/`）。2026-10-16 の main で `print.html` と印刷のボタンを案内している箇所は無い
  - 完了した `manual-print-media-refs` の spec の文書（記録として残す）
  - `book/tools/talk/talk-html-test.mjs` の (D) のフィクスチャの `print.html`、`book/tools/link-check-test.mjs` の `print.html` を読まないことの確かめ（どちらも `print.html` の無い出力でも正しい）
  - 台詞の変換・着色・検索の索引・リンク検査の各ツールの動き（`print.html` が無くても今までどおり動く）
- **Adjacent expectations**:
  - mdBook（CI は v0.5.3、手元は v0.5.4）が、設定で全章の印刷用のページと印刷のボタンを作らないようにできること
  - 旧版は固定のコミット（`CLASSIC_REF`）の `book/` を、その版のツールで生成する（`manual-claudia-theme` 要件 11）。本 spec はこの決まりを保ち、生成のときの設定だけを足す
  - `pasta-check-dic-validate` は、本 spec が入るまで `manual.yml` を触らない（roadmap の取り決め）

## Requirements

### Requirement 1: 新版のマニュアルに全章の印刷用のページを作らない
**Objective:** As a マニュアルの読者, I want どのページを開いてもブラウザが固まらないこと, so that マニュアルを読み、必要な章を刷れる

#### Acceptance Criteria
1. The マニュアルのビルド shall 新版の出力に全章をつないだ印刷用のページ（`print.html`）を作らない
2. The マニュアルのビルド shall 新版のどの章のページにも、全章の印刷用のページへのボタンやリンクを出さない
3. When 読者が新版の章のページでブラウザの印刷を使う, the 章のページ shall 今までどおり目次とボタンを隠した、その章だけの印刷を出す
4. When 読者が公開サイトの `/pasta/print.html` を開く, the 公開サイト shall 別のページへ転送せず、マニュアルの見つからないページを返す
5. The マニュアルのビルド shall `file://` のオフライン閲覧と、`/pasta/` のサブパスでの公開の両方で、今までどおり全章とその画像・動画・資材を開ける出力を作る

### Requirement 2: 旧版のマニュアルに全章の印刷用のページを作らない
**Objective:** As a 旧版を開く読者, I want 旧版でもブラウザが固まらないこと, so that 着せ替え前の版を安全に見比べられる

#### Acceptance Criteria
1. The 旧版の生成 shall 旧版の出力（`classic/`）に全章の印刷用のページ（`print.html`）を作らない
2. The 旧版の生成 shall 旧版のどの章のページにも、全章の印刷用のページへのボタンやリンクを出さない
3. The 旧版の生成 shall 固定のコミットの原稿とツールを変えずに生成し、旧版の章の本文・テーマ・着色・検索を今までどおりにする（全章の印刷用のページと、それに伴って無くなる章の印刷用の規則は除く）
4. If 旧版の出力に `print.html` がある, the 旧版の生成 shall エラーを表示して止まり、マニュアルを公開しない
5. If 旧版の出力に入口のページ（`index.html`）か検索の索引が無い, the 旧版の生成 shall 今までどおりエラーを表示して止まり、マニュアルを公開しない

### Requirement 3: 印刷用のページのためだけにあったものを残さない
**Objective:** As a マニュアルの保守者, I want 誰も開かないページのための道具や段や説明が残っていないこと, so that 手順と説明が出力の実際と食い違わない

#### Acceptance Criteria
1. The リポジトリ shall 印刷用のページのメディアの参照を直す道具（`book/tools/print-media-refs.mjs`）とその自己テスト（`book/tools/print-media-refs-test.mjs`）を持たない
2. The マニュアルの CI shall 印刷用のページのメディアの参照を直す段を持たず、先頭のパイプラインの説明にもその段と `print.html` の検査を書かない
3. The 手元の静的性の検査のビルド shall CI と同じ順で同じ段を踏み、印刷用のページのメディアの参照を直す段を呼ばない
4. The 著者向けの手引き（`book/AUTHORING.md`）の「CI と同じ順」の一覧 shall 印刷用のページのメディアの参照を直す行を持たない
5. The steering（`tech.md` のパイプライン・`structure.md` の `book/tools` の説明）shall 印刷用のページのメディアの参照を直す段と道具を書かない
6. The 設定・テーマ・ツールの先頭の説明 shall 「`print.html` に読み込まれる」「`print.html` を含む」のように、`print.html` が出力にある前提の説明を書かない

### Requirement 4: 出力の検査を印刷用のページの無い出力に合わせる
**Objective:** As a マニュアルの保守者, I want 検査が印刷用のページの無い出力を正しく見て、印刷用のページが戻ったら気づくこと, so that 固まるページが黙って公開されない

#### Acceptance Criteria
1. The 静的性の検査 shall テーマの CSS・JS の参照、台詞部品、テーマメニューを、目次（`SUMMARY.md`）の全章のページについて今までどおり確かめ、`print.html` を確かめる対象に含めない
2. The 静的性の検査 shall 全ページの `href`・`src`・`poster` の壊れた参照を今までどおり例外なく見つける（`manual-print-media-refs` 要件 3.3 を保つ）
3. If 新版の出力に `print.html` がある, the 静的性の検査 shall 失敗し、`print.html` があることを表示する
4. If 新版の章のページに全章の印刷用のページへのボタンやリンクがある, the 静的性の検査 shall 失敗し、そのページを表示する
5. The 静的性の検査の自己テスト shall 要件 4.3・4.4 の検出と、`poster` の壊れた参照の検出を、故意に壊した出力の写しで確かめ、`print.html` が出力にある前提の自己テストを残さない
6. While 旧版が出力に無い, the 静的性の検査 shall 今までどおり旧版を確かめる対象に含めない

### Requirement 5: 一度の変更で入れ、CI を通す
**Objective:** As a マニュアルの保守者, I want 設定・道具・段・検査が同じ変更で入ること, so that 途中の状態で CI が落ちたり、固まるページが公開されたりしない

#### Acceptance Criteria
1. The 変更 shall 要件 1〜4 を 1 つのプルリクエストで入れ、設定だけ・道具の削除だけが main に入る途中の状態を作らない
2. When 変更のプルリクエストでマニュアルの CI が走る, the マニュアルの CI shall 新版のビルドと検査・ツールの自己テスト・旧版の生成をすべて通す
3. When 手元で静的性の検査（自己テスト付き）を走らせる, the 静的性の検査 shall CI と同じ結果で通る
