# Brief: manual-claudia-theme

## Problem
pasta のマニュアル（<https://ekicyou.github.io/pasta/>）は mdBook の既定テーマのままである。白地に黒い文字、灰色の引用ブロックだけの見た目で、とても硬いマニュアルに見える。案内役の Claudia の台詞も地の文と同じ見た目で流れてしまい、Claudia が前に出ている感じがしない。初心者を物語で導く入門ガイド（`getting-started-story-guide`）を書いても、見た目が硬いままでは親しみやすさが伝わらない。

## Current State
- `book/book.toml`: mdBook 0.5.3（`manual.yml` で固定）。`default-theme = "light"`、`preferred-dark-theme = "navy"`。独自の CSS は無い。
- `book/theme/head.hbs`（269 行）: 日本語 bigram 検索のクエリ側 tokenizer をインラインで持つ。`verify-search.mjs` が、索引側の `tokenize.mjs` と逐語一致するかを検査している。
- 公開後の加工: `mdbook build` の後に、`pasta` コードブロックの TextMate 着色（`book/tools/highlight`）と、bigram 索引の再生成が走る。
- Claudia の台詞は、各章の導入と締めに普通の段落として書かれている。`book/AUTHORING.md` が「導入（キャラ口調）→ 本体（普通文体）→ 締め（キャラ口調）」を課し、`verify-content.mjs` が散文部の口調マーカーで判定している。
- `book.toml` のコメントによれば、相対リンクの出力を保って `file://` のオフライン閲覧と両立させている。

## Desired Outcome
- マニュアル全体が、親しみやすい「Claudia のマニュアル」に見える。配色・字体・余白・区切り・枠の意匠は、参考サイト（下記）の調子にそろえる。
- Claudia の台詞を、顔アイコン付きの吹き出し部品で書ける。表情（素・照れ・驚き・落胆・高笑い・目閉じ・不機嫌・にっこり・したり顔など）を台詞ごとに選べる。
- 既存の章の導入と締めの台詞も、この部品で表示される。
- マニュアルの表紙（`introduction.md`）に、Claudia の立ち絵と案内のある扉を置く。
- ダークテーマ（`navy` など）でも読める。mdBook のテーマ切り替えを壊さない。
- 検索（bigram）・`pasta` の着色・目次・印刷・`file://` での閲覧・既存の検査ツールが、これまでどおり動く。

## Approach
mdBook のまま着せ替える（2026-10-06 決定。サイト生成器の移行は、検索・着色・検査ツール・公開の流れを作り直すことになるので採らない）。

- **参考の意匠**: ponapalt さんの「悪役令嬢クローディア」紹介ページ <https://ponadocs.shillest.net/claudia/>（ソースは [ponapalt/claudia](https://github.com/ponapalt/claudia) の `site/`）。リポジトリ全体が **Unlicense**（パブリックドメイン）なので、CSS・顔アイコン・立ち絵を流用してよい。出典はマニュアルのどこかに記す（義務ではないが礼儀として）。
- **意匠の要素**（2026-10-06 に参考サイトから読み取った値）:
  - 色: 紙 `#FAF5EA`・紙（濃）`#F0E4CE`・縁 `#E3D2B3`・墨 `#3A2419`・墨（弱）`#6B4E3D`・差し色 `#B4532F`・罫（金）`#B98D4A`・封蝋 `#6D1F2C`
  - 字体: 本文 BIZ UDPMincho、見出し Shippori Mincho B1、欧文の添え字 Cormorant Garamond（斜体）、表や細かい情報 BIZ UDPGothic（いずれも Google Fonts・OFL）
  - 角丸: 紙 14px・カード 10px・部品 4px。影は墨色の淡い 2 段
  - 部品: 角飾り付きの紙の枠（hero）・金の菱形の区切り線・丸い顔アイコン付きの台詞（`talk`）・人物カード・手順の欄・手紙風の囲み
  - 顔アイコンは `site/img/f*.png`、立ち絵は `shell/master/surface*.png`
- **実装の形**: `book.toml` の `additional-css`・`additional-js` と `theme/` の上書きを使う。`head.hbs` の検索 tokenizer には手を触れない。
- **台詞の部品**: Markdown の中に書く記法（例: HTML の `<div class="claudia" data-face="高笑い">`、または決まった形の引用ブロックを JS／CSS で変換する方式）は設計で決める。次を満たすこと。
  - 生 HTML を書かなくても書ける形が望ましい（作者が書きやすい）
  - `verify-content.mjs` の口調検査と、`gen-skill-refs.mjs` の生成（生成対象章の導入・締めの扱い）が、新しい記法を正しく扱える
  - 検索の索引に台詞の本文が入る
- **本文の字体**: 技術文書を明朝で長く読むと疲れることがある。本文に明朝とゴシックのどちらを使うかは、実際の章（文法・内部設計）で読みやすさを確かめてから要件で決める。コードブロックは等幅のまま。

## Scope
- **In**:
  - サイト全体のテーマ（配色・字体・余白・見出し・区切り・表・引用・注意書き・コードブロックの枠・目次・上のバー）とダーク版
  - Claudia の台詞の部品と、その記法の執筆規約（`AUTHORING.md`）への追記
  - 既存の全章の導入と締めの台詞を、新しい記法へ書き換える
  - 表紙の扉
  - 顔アイコン・立ち絵の取り込みと出典の記載
  - 検査ツール（`verify-content.mjs`・`gen-skill-refs.mjs`・`verify-static.mjs` など）の追従
  - スマートフォン幅での表示
- **Out**:
  - 入門ガイドの本文（`getting-started-story-guide`）
  - 各章の内容の書き換え（台詞の記法の置き換え以外）
  - サイト生成器の移行
  - 検索の仕組み（bigram 索引・tokenizer）の変更
  - hello-pasta のシェル（`hello-pasta-shell-art`）

## Boundary Candidates
- テーマ（CSS・字体・ダーク版・表紙）— 見た目だけ
- 台詞の部品（記法・変換・執筆規約・検査ツールの追従）— 書き手と検査に関わる

## Out of Boundary
- 物語の構成と、ガイドの章の文章（`getting-started-story-guide`）
- Claudia ゴースト本体や ponapalt/claudia への変更

## Upstream / Downstream
- **Upstream**: ponapalt/claudia（Unlicense の素材と意匠）、mdBook 0.5.3 のテーマの仕組み、`book/tools` の検査・生成ツール群
- **Downstream**: `getting-started-story-guide`（ガイドの全編を Claudia が語るときに、台詞の部品を使う）

## Existing Spec Touchpoints
- **Extends**: なし。`pasta-user-manual`（Claudia 令嬢ボイスの規約）・`pasta-manual-syntax-highlight`（着色）・`manual-ssot-authority`（生成と検査）は完了済みで、再オープンしない。本 spec がその上に足す。
- **Adjacent**:
  - `getting-started-story-guide`: `AUTHORING.md` の別の節を触る。後から入る側が rebase で合わせる。
  - Phase 11 の未完了 spec: 文法章の本文を触ることがある。本 spec が触るのは導入・締めの台詞の記法だけ。

## Constraints
- 検索の bigram tokenizer（`head.hbs` のインライン部分）と `verify-search.mjs` の逐語一致を壊さない。
- `file://` のオフライン閲覧を壊さない。Web フォントが読めないときは、システムの字体に落ちて読めること。
- 着色済みの `pasta` コードブロックの見た目（`pasta-manual-syntax-highlight` の配色）が、新しい背景色でも読めること。必要なら着色の配色も合わせる。
- 生成対象章（`GENERATION_MAP`）のスキル `references/` に、台詞の部品の HTML や画像参照が漏れないこと。
- 画像の容量を抑える（顔アイコンは小さく、立ち絵は表紙など限られた所だけ）。
- 完成度を優先する。一部の章だけ新しい見た目にする、といった部分出荷はしない。
