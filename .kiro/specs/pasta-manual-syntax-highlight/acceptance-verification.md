# 受け入れ検証記録 — pasta-manual-syntax-highlight（要件 8 / タスク 6.1）

> 本機能の **GitHub Pages 公開 HTML** に対する初回・一度限りの受け入れ検証記録。
> 恒常ゲート化しない（要件 8.4）。デプロイ後に本書の「検証結果」欄を埋めて完了とする。

## ステータス

- **現状: 実施完了・GO（2026-06-06）**
- 経緯: `feature/pasta-manual-syntax-highlight` を squash コミット `f15aaa6` として `main` へ ff マージ・push。
  `manual.yml`（run 27049848861）が `npm ci`(book) → `mdbook build` → **Highlight pasta code blocks** → bigram →
  drift → tutorial → cargo test → Pages deploy を全ステップ成功で完了。`https://ekicyou.github.io/pasta/` へ反映後、
  公開 HTML を対象に本受け入れ検証を一度限り実施し全 AC（8.1〜8.4）を実証した。

## 実施トリガー（前提条件）

1. `feature/pasta-manual-syntax-highlight` を `main` へマージ（または直接 push）。
2. `manual.yml` が成功（`npm ci`(book) → `mdbook build` → **highlight 後処理** → bigram → 既存ゲート → Pages deploy）。
3. `https://ekicyou.github.io/pasta/` に反映後、本検証を一度だけ実施する。

## 事前確信（ローカル統合・タスク 5.2 で実証済み）

公開前検証として、実 `mdbook build book` → highlight 後処理に対し以下を実機確認済み（独立レビューで再現）:

- pasta ブロックに 6 区分の hljs span が付与（build 直後 0 span → 後処理後に着色）。禁止クラス
  `hljs-symbol`/`hljs-section`/`hljs-name` は 0。
- 入れ子 ```lua も二段トークナイズで着色。
- 決定論・冪等（再実行で *.html バイト不変・span 数不変）。
- light（`highlight-*.css`）/ navy（`tomorrow-night-*.css`）両テーマで採用 6 クラスが相互に異なる色:
  - light: gray `#575757` / purple `#9d00ec` / blue `#0030f2` / orange `#b21e00` / green `#008200` / red `#d70025`
  - navy: gray `#969896` / purple `#b294bb` / aqua `#8abeb7` / orange `#de935f` / green `#b5bd68` / red `#cc6666`
- 各ページ `<head>` に hljs 中和スクリプト（`defineProperty(win,'hljs')`・`language-pasta` スキップ）が同梱され、
  既存 elasticlunr ブロックと共存。テーマ CSS は相対 href でリンク（file:// 構成が成立）。

→ 公開 HTML は同一の build-time 後処理＋同一 head.hbs から生成されるため、デプロイ後の受け入れは高確度で成立見込み。

## 受け入れチェックリスト（デプロイ後に実施・各項目に結果を記入）

### 8.1 公開 HTML に hljs span が付与されている
- [x] `https://ekicyou.github.io/pasta/` 配下の pasta コードブロックを含むページの生 HTML（node https 取得）で
      `<code class="language-pasta">` 内に `<span class="hljs-...">` が存在することを確認。
- 結果: **PASS** — 公開ページ実測:
  - `grammar/actor-dictionary.html`: 3 ブロック / 43 span / 6 クラス（built_in, comment, keyword, string, title, variable）
  - `grammar/markers.html`: 4 ブロック / 15 span / 5 クラス（attr, keyword, string, title, variable）
  - `getting-started/first-ghost.html`: 5 ブロック / 286 span / 7 クラス（built_in, comment, keyword, number, string, title, variable）
  - 全ページで禁止クラス `hljs-symbol`/`hljs-section`/`hljs-name` は **0**。入れ子 lua も着色（actor-dictionary）。

### 8.2 book.js 再ハイライト後も span が保持されている
- [x] 公開ページの DOM ＋ `<head>` に同梱された**デプロイ済み中和スクリプト**を jsdom で実行し、book.js の
      無条件 `highlightBlock` 再ハイライトを模擬しても pasta ブロックの事前 span が破壊されず生存することを確認。
- 結果: **PASS** — `grammar/actor-dictionary.html` の3 pasta ブロックで再ハイライト前後の span 数が完全一致
      （pre `19,8,16` → post `19,8,16`）、innerHTML 不変、pasta ブロックは中和済み hljs によりスキップ。
      中和スクリプトは全公開ページ `<head>` に存在し既存 elasticlunr ブロックと共存。
      （中和ロジック正準は jsdom ユニットテスト neutralizer-test.mjs でも検証済み＝タスク4.2）

### 8.3 light/navy 両テーマ＋file:// で各構文要素が相互に判別可能
- [x] 公開テーマ CSS がローカル検証（タスク5.2）と**同一ハッシュ**（バイト等価）であることを確認し、
      6区分が両テーマで相互に異なる色になることを担保。`file://` 構成要素（静的 span・相対 href の CSS・
      インライン中和）が公開 HTML に揃っていることを確認。
- 結果: **PASS** — 公開ページが参照する `highlight-493f70e1.css`（light）/ `tomorrow-night-4c0ae647.css`（navy）は
      タスク5.2 でローカル抽出した 6 色（light: gray#575757/purple#9d00ec/blue#0030f2/orange#b21e00/green#008200/red#d70025、
      navy: gray#969896/purple#b294bb/aqua#8abeb7/orange#de935f/green#b5bd68/red#cc6666）と同一ハッシュ＝同一配色。
      CSS は相対 href・span は静的・中和はインラインのため `file://` でも色保持される構成。

### 8.4 一度限り（恒常ゲート化しない）
- [x] `manual.yml` に毎ビルドのハイライト品質検証ゲートを追加していない（生成2ステップ＝npm ci・highlight のみ・
      タスク5.1 レビュー済み）。中和テストは build-time ユニットテストで公開サイトを叩く恒常ゲートではない。
- 結果: **PASS** — 本受け入れ検証は初回一度限りで実施。恒常ゲート化なし。

## 署名

- 実施者 / 実施日: Claude（kiro-impl 自律モード） / 2026-06-06
- 総合判定: **GO** — 公開 GitHub Pages HTML で 8.1〜8.4 全 AC を実証。本機能は本番環境で成立。
