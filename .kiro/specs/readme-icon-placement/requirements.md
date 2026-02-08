# Requirements Document

## Introduction

`img/pasta.svg` として作成済みのプロジェクトアイコンを `README.md` に配置する。
配置パターンを複数提案し、比較検討の材料を提供する。
GitHubでの表示品質・Markdownの可搬性・プロジェクトのブランディングを考慮する。

## Project Description (Input)
pasta.svgアイコンをつくった。README.mdに良い感じのバランスでアイコンイメージを配置して欲しい。配置について思いつく限り案を出してほしい、比較検討したい。

## Requirements

### Requirement 1: アイコン配置案の提示
**Objective:** As a プロジェクトオーナー, I want README.mdにpasta.svgアイコンを効果的に配置する複数の案を比較検討できること, so that プロジェクトのブランディングと視認性が最適化される

#### Acceptance Criteria
1. The README shall アイコン画像パス `img/pasta.svg` を相対パスで参照する
2. The README shall GitHub上でSVG画像が正常にレンダリングされる形式で記述する
3. The README shall アイコンのサイズを明示的に指定する（width/height属性）

### Requirement 2: 配置パターンの多様性
**Objective:** As a プロジェクトオーナー, I want 異なるレイアウトパターンを比較検討できること, so that プロジェクトに最適な配置を選択できる

#### Acceptance Criteria
1. The requirements document shall 最低5つ以上の配置パターン案を提示する
2. The requirements document shall 各案にメリット・デメリットを記載する
3. The requirements document shall 各案のMarkdownコードサンプルを含める
4. The requirements document shall GitHub/VS Code両方での表示互換性を考慮する

### Requirement 3: 既存構造との調和
**Objective:** As a 開発者/閲覧者, I want アイコン配置が既存のREADME構造を壊さないこと, so that ドキュメントの情報構造が維持される

#### Acceptance Criteria
1. The README shall 既存の見出し階層（h1〜h3）を維持する
2. The README shall Pasta DSLサンプルコードブロックの視認性を損なわない
3. The README shall 説明文（blockquote）との視覚的バランスを保つ
4. If アイコンが読み込めない場合, the README shall altテキストで代替表示する

---

## 配置パターン案（プレビューで比較してください）

---

### 案A: ヘッダー中央配置（タイトル上）｜★★★★☆

> ✅ アイコンが最初に目に入る。OSSの定番。中央揃えで堂々とした印象
> ❌ タイトルとアイコンが分離して見える場合がある

<div align="center">
  <img src="/img/pasta.svg" alt="Pasta logo" width="128">
</div>

# pasta
Memories of pasta twine together—now and then a knot, yet always a delight.

---

### 案B: タイトル横並び（インライン配置）｜★★★☆☆

> ✅ コンパクト。タイトルとアイコンが一体化。スペース効率が良い
> ❌ GitHubではh1内のstyle属性が無視される場合がある。アイコンが小さくなる

# <img src="/img/pasta.svg" alt="Pasta" width="40" style="vertical-align: middle;"> pasta
Memories of pasta twine together—now and then a knot, yet always a delight.

---

### 案C: ヘッダーブロック（アイコン＋タイトル＋サブタイトル中央配置）｜★★★★★【最推奨】

> ✅ 最もリッチなヘッダー。アイコン・タイトル・キャッチコピーが一体。プロフェッショナルな印象
> ❌ ヘッダー部分が大きくなり、コンテンツまでのスクロール量が増える

<div align="center">
  <img src="/img/pasta.svg" alt="Pasta logo" width="128">
  <h1>pasta</h1>
  <p><em>Memories of pasta twine together—now and then a knot, yet always a delight.</em></p>
</div>

---

### 案D: h1維持＋中央アイコンブロック｜★★★★☆

> ✅ h1のMarkdown見出しを維持しつつ、その直下にアイコンブロック。GitHub目次との相性が良い
> ❌ タイトルとアイコンが若干分離する

# pasta

<div align="center">
  <img src="/img/pasta.svg" alt="Pasta logo" width="96">
  <br>
  <em>Memories of pasta twine together—now and then a knot, yet always a delight.</em>
</div>

---

### 案E: 右寄せフロート（テキスト横配置）｜★★☆☆☆

> ✅ テキスト量が多い場合にスペース効率が良い。ドキュメント的な雰囲気
> ❌ GitHub Markdownの`align`サポートが不安定。レスポンシブで崩れやすい

# pasta

<img src="/img/pasta.svg" alt="Pasta logo" width="96" align="right">

Memories of pasta twine together—now and then a knot, yet always a delight.

<br clear="both">

---

### 案F: 左寄せフロート（テキスト横配置）｜★★☆☆☆

> ✅ 新聞的なレイアウト。アイコンが自然に視線誘導する
> ❌ GitHub上でのフロート挙動が不安定。後続コンテンツの回り込みに注意が必要

<img src="/img/pasta.svg" alt="Pasta logo" width="80" align="left" style="margin-right: 1em;">

# pasta
Memories of pasta twine together—now and then a knot, yet always a delight.

<br clear="both">

---

### 案F2: 左寄せフロート・大アイコン（1.5倍）｜★★★☆☆

> ✅ 案Fの大きめアイコン版。視認性アップ。存在感がありつつテキストとの共存
> ❌ フロートの回り込み範囲が広がる。テキスト領域が狭まる

<img src="/img/pasta.svg" alt="Pasta logo" width="120" align="left" style="margin-right: 1em;">

# pasta
Memories of pasta twine together—now and then a knot, yet always a delight.

<br clear="both">

---

### 案G: 小アイコン＋タイトル行（テーブル利用）｜★★☆☆☆

> ✅ テーブルで確実に横並びを実現。GitHub互換性が高い
> ❌ テーブルのボーダーが見えてしまう（GitHubテーマ依存）。見出しレベルが下がる

| | |
|---|---|
| <img src="/img/pasta.svg" alt="Pasta logo" width="64"> | **pasta** <br> *Memories of pasta twine together—now and then a knot, yet always a delight.* |

---

### 案H: ヒーローセクション（大アイコン＋詳細説明）｜★★★★☆

> ✅ 最大のインパクト。プロジェクトの性格が一目で伝わる。OSSランディングページ風
> ❌ ヘッダーが最も大きくなる。小規模プロジェクトには大げさに見える可能性

<div align="center">
  <img src="/img/pasta.svg" alt="Pasta logo" width="200">
  <h1>pasta</h1>
  <p><strong>対話スクリプト言語 / SHIORI.DLL</strong></p>
  <p><em>Memories of pasta twine together—now and then a knot, yet always a delight.</em></p>
</div>

---

## 比較サマリー

| 案 | スタイル | GitHub互換 | コンパクト | インパクト | 推奨度 |
|----|----------|------------|------------|------------|--------|
| A | 中央・タイトル上 | ◎ | ○ | ○ | ★★★★ |
| B | インライン | △ | ◎ | △ | ★★★ |
| C | 中央ブロック | ◎ | △ | ◎ | ★★★★★ |
| D | h1維持＋中央 | ◎ | ○ | ○ | ★★★★ |
| E | 右フロート | △ | ○ | △ | ★★ |
| F | 左フロート | △ | ○ | △ | ★★ |
| G | テーブル横並び | ○ | ◎ | △ | ★★ |
| H | ヒーロー大型 | ◎ | × | ◎ | ★★★★ |

### 推奨

**案C（ヘッダーブロック）** を最推奨。理由：
- アイコン・タイトル・キャッチコピーが視覚的に一体化
- GitHub Markdownで確実にレンダリングされる `<div align="center">` を使用
- OSSプロジェクトの標準的なパターンで、初見のユーザーに馴染みやすい
- 既存のblockquote説明文をヘッダーに統合でき、重複を削減可能

