# Brief: manual-download-page

> **ステータス**: 未着手（2026-11-14 の `/kiro-discovery` で起票。開発者「ダウンロード関係だけ簡潔に説明したページが欲しい」）。版は未定（マニュアルは main への push で Pages に出るので、リリースを待たない）。方針は discovery で推奨案のまま決まった（下の「Approach」）。

## Problem

マニュアルを開いた作者が、どのファイルを落とせばよいかにすぐ辿り着けない。

- 配布物が 7 つに増えた（`pasta.dll.zip`・`hello-pasta.nar`・VSIX・`pasta_check.zip`・`pasta-skills.zip`・`pasta-recipe.x64.zip`・`pasta-recipe.arm64.zip`）。
- トップページの案内は、「関連リンク」の表の 1 つのセルに 7 つのファイル名を詰めた一文で、目的から選べない。
- トップページが回りくどい。「このマニュアルの歩き方」は扉の目次の言い直しで、AI のスキルを展開する手順までトップページにある。

## Current State

2026-11-14（main `0ad6937a`・v0.6.0）に確かめた。

- **目次**（`book/src/SUMMARY.md`）: `[はじめに](introduction.md)` の次がすぐ `# 入門 / チュートリアル` のパートになっている。ダウンロードの章は無い。
- **扉の案内**（`introduction.md` の `<nav class="hero-toc">`）: 入門・文法・Lua・デバッグ・リファレンス・内部設計・雑記の 7 つ。
- **トップページの本文**（`introduction.md`・119 行）: 説明の 3 行、「関連リンク」（配布物 7 つを 1 セルに詰めている）、「AI にゴーストを書かせるとき」（`pasta-skills.zip` の中身と展開の木・版を上げるときの注意）、「このマニュアルが対象とするバージョン」、「安定機能と『将来変更あり』の区別について」、「このマニュアルの歩き方」（扉の目次と重なる）。
- **入手の説明がばらけている**: `getting-started/prerequisites.md`（`hello-pasta.nar`・`pasta.dll.zip`）、`getting-started/setup.md`、`debug/recipe.md`（pasta-recipe の 2 つ）、`debug/validate.md`（`pasta_check.zip`）、`debug/vscode-setup.md`（Marketplace と VSIX）、`README.md` L41-47（配布物の一覧）。
- **トップページを見張る検査**: `book/tools/verify-content.mjs` の F（L409-432）が、`introduction.md` に対象の pasta の版（Cargo.toml と一致）・LuaJIT 2.1 の明示・「将来変更あり」の注記があることを確かめる。`book/tools/theme-test.mjs` は扉のクラス（`hero-toc`・`claudia-credit` など）の CSS を見る。`book/tools/talk/talk-test.mjs` の扉は自前の写しなので、本物の扉の項目の数には縛られない。`book/tools/link-check.mjs` がリンクと見出しのアンカーを見る。

## Desired Outcome

- 目次（サイドバー）の「はじめに」のすぐ下と、扉の案内の先頭に「ダウンロード」がある。
- ダウンロードのページを開けば、「やりたいこと → 取るファイル」が 1 つの表で分かり、そのまま最新版を落とせる。
- トップページが短くなり、「まずダウンロード → 入門ガイド」へすぐ進める。
- 入手の説明が 1 か所にまとまり、ほかの章と README はそのページを指す。

## Approach

discovery の案 A（推奨案のまま開発者が承認）。新しいページ `book/src/download.md` を作り、トップページも短くする。

- 案 B（ページを足すだけでトップページはそのまま）は「回りくどい」が直らないので採らない。
- 案 C（トップページに配布物の表を置く）はトップページが太り、ページを分けたい要望に合わないので採らない。

決めたこと（2026-11-14 開発者「推奨案で書いて」）:

1. **置き場所**: SUMMARY の `[はじめに](introduction.md)` の次に、番号の無い章 `[ダウンロード](download.md)` を置く（パートの見出しの前）。扉の案内の先頭に `- [ダウンロード <small>Download</small>](download.md)` を足す。
2. **ページの形**: 先頭に「やりたいこと → 取るファイル」の表を 1 つ置く。その下にファイルごとの短い説明（中身・置き場所・使っている pasta と版をそろえる注意）を並べる。リンクは `releases/latest/download/<名前>` の固定リンクにし、過去の版はリリースページを指す。Claudia の掛け合いは入れない（簡潔さを優先。Claudia が全編を語る例外は `getting-started` だけ）。

   | やりたいこと | ファイル |
   |---|---|
   | まずゴーストを動かしてみる | `hello-pasta.nar` |
   | 自分のゴーストの `pasta.dll` を差し替える | `pasta.dll.zip` |
   | VS Code を使わずに書く | `pasta-recipe.x64.zip`／`pasta-recipe.arm64.zip` |
   | VS Code で書く | Marketplace（VSIX は手動で入れるとき） |
   | 辞書を検査する・.nar を作る | `pasta_check.zip` |
   | AI に書かせる | `pasta-skills.zip` |

3. **トップページの削り方**: 残すのは、扉・2〜3 行の説明・「まずダウンロード → 入門ガイド」の案内・対象バージョンの表・「将来変更あり」の注記。「AI にゴーストを書かせるとき」の節はダウンロードのページへ移す。「このマニュアルの歩き方」の節と「関連リンク」の配布物のセルは消す（GitHub リポジトリへのリンクは残す）。`verify-content.mjs` の F が見る 3 つ（版・LuaJIT 2.1・将来変更あり）はトップページに残す。
4. **ほかの章の入手の説明**: 各章の手順の流れは残し、「どこで手に入れるか」の説明だけをダウンロードのページ（とその見出しのアンカー）へのリンクに寄せる。README も同じ扱い（GitHub で最初に読まれるので、一覧を残してページへのリンクを足すか、リンクだけにするかは要件で決める）。

## Scope

- **In**:
  - `book/src/download.md` の新設と `SUMMARY.md`・扉の案内への追加
  - `introduction.md` の削り込み（「AI にゴーストを書かせるとき」の移設を含む）
  - 入手の説明の寄せ: `getting-started/prerequisites.md`・`getting-started/setup.md`・`debug/recipe.md`・`debug/validate.md`・`debug/vscode-setup.md`・`README.md`
  - 動かなくなる検査の追従（`verify-content.mjs`・`link-check.mjs` の期待など。要るときだけ）と、`node book/tools/gen-skill-refs.mjs` でのスキルの `references/` の再生成（移した記述がスキルに写っていれば）
- **Out**:
  - 配布物そのものの名前・中身・リリース CI の変更
  - 扉の意匠（CSS・顔アイコン）の作り直し
  - 入門ガイドの本文の書き直し（入手先のリンクの差し替えだけ）
  - 英語版や他言語のページ

## Boundary Candidates

- ダウンロードのページの中身（表と各ファイルの説明）
- トップページの削り込みと目次・扉の入口
- ほかの章・README からの参照の付け替え

## Out of Boundary

- 配布物の形（`pasta-dll-multiarch`・リリースの席が持つ）
- 入門ガイドの段の章の作例と段階辞書の照合（`tutorial-check.mjs` の持ち場。触るのはリンクだけ）

## Upstream / Downstream

- **Upstream**: 今の配布物 7 つ（v0.6.0。`pasta-recipe-multiarch` で recipe が x64・arm64 に分かれた）
- **Downstream**: `pasta-dll-multiarch` が入ると `pasta.x64.dll`・`pasta.arm64.dll`（または `pasta.dll.zip` の形の変更）が増える。そのときは、あちらの spec がこのページの表と説明を直す。今後配布物を足す spec も、このページを直す。

## Existing Spec Touchpoints

- **Extends**: なし（マニュアルの最初の形を作った spec は完了済み）
- **Adjacent**: `pasta-dll-multiarch`（配布物とマニュアルの DLL の説明。同じウェーブに置くなら、ダウンロードのページの節を持ち場分けする）

## Constraints

- マニュアルの執筆規約（`getting-started` 以外は地の文の常体）に従う。
- `verify-content.mjs`・`link-check.mjs`・`theme-test.mjs` などの `book/tools` の検査が通ること。見出しを変えるときは、それを指すアンカーのリンクも直す。
- マニュアルは main への push で Pages に出る。リリースを待たずに出せる。
