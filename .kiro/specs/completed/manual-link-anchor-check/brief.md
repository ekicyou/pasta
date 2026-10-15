# Brief: manual-link-anchor-check

`getting-started-story-guide` の完了時の棚卸で起票（2026-10-10・main `f193c9cb`）。

## Problem

マニュアル（`book/src/`）の章どうしのリンクは、見出し（`#…`）まで指していることが多い。リンク検証（`book/tools/link-check.mjs`）は、本の中のリンクについてはリンク先のファイルの実在までしか見ない。見出しの文言を変えると、そこを指すリンクは黙って切れ、CI も気づかない。読者は章の先頭に着地し、目当ての節を自分で探すことになる。

## Current State

- `link-check.mjs` には、見出しから `id` を作る関数（`headingSlug`・`headingSlugs`。重複の付番つき）がもうある。使っているのはスキルの自己完結の検査（`skill-anchor`）だけで、本の中のリンクには使っていない。
- 入門ガイド（16 章）は、章の間の見出しへのリンクを 44 本持つ。`getting-started-story-guide` では、ビルドした出力の `id` を使い捨てのスクリプトで確かめた（切れ 0）。その確かめは CI に残っていない。
- 公開済みクレートの README からの絶対 URL（`…/getting-started/setup.html#ゴーストのフォルダ構成`）は、`link-check.mjs` の別の規則（公開 URL を `book/src` のファイルへ写す）が見ている。ここも見出しまでは見ていない。

## Desired Outcome

- 本の中のリンクが実在しない見出しを指していたら、リンク検証が落ちる。
- クレートの README などから公開 URL で指している見出しも、同じく確かめられる。
- 今のマニュアル（60 章）で、誤検出が 0 件である。

## Approach

`link-check.mjs` の既存の `headingSlugs` を、本の中のリンク（`*.md#…`・`#…`）と公開 URL の写像にも使う。新しいツールや依存は足さない。mdBook が実際に出す `id` と `headingSlugs` の結果が食い違う見出し（記号・全角・重複）が無いかを、ビルドした出力と突き合わせて先に確かめる。

## Scope

- **In**: `book/tools/link-check.mjs` の本の中のリンクと公開 URL の規則に、見出しの実在の検査を足す。自己テスト（`book/tools/link-check-test.mjs`）。見つかった切れリンクの修正。
- **Out**: 外部サイトの見出し。`print.html` の中のリンク。mdBook の `id` の作り方を変えること。

## Boundary Candidates

- 見出しから `id` を作る規則（mdBook の出力と一致させる）。
- 本の中のリンクの検査と、公開 URL の検査。

## Out of Boundary

- `book/tools/verify-static.mjs`・`.github/workflows/manual.yml`（`manual-print-media-refs` がこのウェーブで持つ。`link-check.mjs` は今も CI で走っているので、手順は変えなくてよい）。
- 章の本文の書き直し（切れたリンクの行だけを直す）。

## Upstream / Downstream

- **Upstream**: なし。
- **Downstream**: マニュアルに章を足す spec すべて（`pasta-check-dic-validate`・`scene-anchor-link` など）。見出しを変える spec は、この検査で切れに気づける。

## Existing Spec Touchpoints

- **Extends**: なし（`link-check.mjs` を持つ未完了の spec は無い）。
- **Adjacent**: `manual-print-media-refs`（`manual.yml`・`verify-static.mjs`）、`getting-started-screenshots`（入門の章）。

## Constraints

- DoD の Manual Sync Gate と Internals Sync Gate が `link-check.mjs` を走らせる。誤検出があると、無関係な spec の完了が止まる。
- 依存を足さない（Node の標準だけ）。

## 2026-10-10 起票時の測定（main f193c9cb）

- **規模**: 2〜4 タスク。
- **種別**: 検査の穴（サイト・CI）。
- **要件定義のモデル**: Opus。
- **要件定義の議題**:
  1. `headingSlugs` の結果と mdBook（CI は 0.5.3、手元は 0.5.4）の `id` が食い違う見出しがあるか。あれば、関数を mdBook に合わせるか、ビルドした出力の `id` を読む形にするか。
  2. 台詞部品（`> 【表情】…`）や HTML の `id` など、見出し以外のアンカーを対象にするか。
  3. 公開 URL の見出しも同じ規則で見るか（公開済みクレートの README は、公開した版の時点の見出しを指す）。
