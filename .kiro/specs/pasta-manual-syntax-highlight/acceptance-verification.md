# 受け入れ検証記録 — pasta-manual-syntax-highlight（要件 8 / タスク 6.1）

> 本機能の **GitHub Pages 公開 HTML** に対する初回・一度限りの受け入れ検証記録。
> 恒常ゲート化しない（要件 8.4）。デプロイ後に本書の「検証結果」欄を埋めて完了とする。

## ステータス

- **現状: デプロイ待ち（未実施）**
- 理由: 本機能は `feature/pasta-manual-syntax-highlight` ブランチにあり、`main` 未マージ・GitHub Pages 未反映。
  公開パイプライン `manual.yml` は `main` への push（`book/**` 変更）でのみ発火するため、公開 HTML は
  マージ→デプロイ完了後に初めて本機能を含む。
- 確認（記録時点）: `book/tools/highlight` は `origin/main` に存在しない（未マージ）。公開サイト
  `https://ekicyou.github.io/pasta/` は稼働中だが pasta コードブロックは未着色（本機能が解決する「before」状態）。

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
- [ ] `https://ekicyou.github.io/pasta/` 配下の pasta コードブロックを含むページ（例: `grammar/` 各章・`lua-*`）の
      ページソースで `<code class="language-pasta">` 内に `<span class="hljs-...">` が存在する。
- 結果: （記入欄）

### 8.2 book.js 再ハイライト後も span が保持されている
- [ ] ブラウザでページを開き、読み込み完了後（book.js 実行後）も pasta ブロックの色分けが保持されている
      （一瞬色が付いて消える/無色化しない）。DevTools で `<code class="language-pasta">` 直下に事前 span が
      残存していることを確認。
- 結果: （記入欄）

### 8.3 light/navy 両テーマ＋file:// で各構文要素が相互に判別可能
- [ ] light テーマで 6 区分が相互に判別可能な配色で表示される。
- [ ] navy テーマ（テーマ切替）で 6 区分が相互に判別可能な配色で表示される。
- [ ] 公開成果物を `file://`（ローカル保存・オフライン）で開いても色分けが保持される。
- 結果: （記入欄）

### 8.4 一度限り（恒常ゲート化しない）
- [ ] 本検証は初回受け入れ時のみ実施。`manual.yml` に毎ビルドのハイライト品質検証ゲートを追加していない
      （build-time の生成工程と中和ユニットテストのみ）。
- 結果: （記入欄）

## 署名

- 実施者 / 実施日: （記入欄）
- 総合判定（GO / NO-GO）: （記入欄）
