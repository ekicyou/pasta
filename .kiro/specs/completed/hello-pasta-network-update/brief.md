# Brief: hello-pasta-network-update

> **ステータス**: 未着手（2026-11-15 の `/kiro-discovery` で起票）。版は未定（開発者の判断待ち）。要件定義は Opus。

## Problem

見本ゴースト hello-pasta は、SSP のネットワーク更新に対応していない。利用者は新しい版が出るたびに、`hello-pasta.nar` を取り直して入れ直すしかない。

- 開発者は、hello-pasta をネットワーク更新で配りたい。公開の手間はリリース CI に任せたい。
- 更新の置き場所を決めないまま `homeurl` だけ足しても動かない。SSP は `homeurl` の下に `updates.txt` と、`ghost/master/pasta.dll` のようなフォルダ構造のままのファイル一式を探す。

## Current State

2026-11-15（main `8ad60680`、v0.6.1）に確かめた。

- **`homeurl`**: `ghosts/hello-pasta/ghost/master/descript.txt` の `homeurl` は `https://github.com/ekicyou/pasta`（リポジトリのページ）。更新の置き場所ではない。
- **`updates.txt`**: `pasta_check release` の段 5 が作り、`hello-pasta.nar` に封入する（ルートと `ghost/master/updates.txt` の 2 か所。各行は md5 と大きさ。除外は `profile`・`var`・`updates*`・`developer_options.txt`）。同梱バルーンがあれば `bal/updates.txt` も作る。ただし、ゴースト自身の `homeurl` を見る検査は無い（バルーンだけ、`homeurl` が無いと警告する）。
- **配布物**: GitHub Release の添付は平らな名前だけで、フォルダ構造を置けない。`releases/latest/download/<名前>` の固定リンクは `book/src/download.md` が使っているが、更新の置き場所には使えない。
- **Pages**: マニュアルが `https://ekicyou.github.io/pasta/` に出ている（`manual.yml`。`./book/book` を 1 つのアーティファクトとして公開し、`deploy` ジョブだけが `pages: write` を持つ）。Pages は 1 サイトを丸ごと入れ替える方式なので、デプロイする所を `manual.yml` に保つ必要がある。`manual.yml` は旧版マニュアル（`classic/`）を、新版の検査の後に `book/book/classic` へ足す形を既に持つ。
- **リリース CI**: `release.yml` の `github-release` ジョブ（`contents: write`）が、`hello-pasta.nar` を含む配布物を Release に添付する。公開先ごとの結果は `report` ジョブの表にまとめる。
- **マニュアル**: `homeurl`・ネットワーク更新に触れる所は無い（`book/src` を検索して 0 件）。

## Desired Outcome

- 利用者の SSP が、hello-pasta を右クリックの更新確認（ネットワーク更新）で最新版にできる。辞書・`pasta.toml`・シェル・`pasta.dll` が、1 つのリリースの版でそろって入れ替わる。
- 更新の置き場所は、リリースのたびにリリース CI が自動で最新にする。開発者の手作業は要らない。
- 置き場所の中身が、そのリリースの `hello-pasta.nar` と同じであることを、CI が公開前と公開後の両方で確かめる。
- hello-pasta をコピーして自分のゴーストを作る作者が、うっかり hello-pasta の更新で自分の辞書を上書きされない。

## Approach

**GitHub Pages に、nar の中身をそのまま展開して置く。** 置き場所は `https://ekicyou.github.io/pasta/hello-pasta/` とする。

```
https://ekicyou.github.io/pasta/hello-pasta/updates.txt
https://ekicyou.github.io/pasta/hello-pasta/ghost/master/pasta.dll
https://ekicyou.github.io/pasta/hello-pasta/ghost/master/dic/01-boot.pasta
https://ekicyou.github.io/pasta/hello-pasta/shell/master/surface0.png
```

- `ghost/master/descript.txt` の `homeurl` を `https://ekicyou.github.io/pasta/hello-pasta/`（末尾の `/` は必須）にする。
- **リリース CI の流れ**:
  1. タグの push で、`release.yml` が nar を作って Release を公開する（今のまま）。
  2. 新しいジョブが、`manual.yml` を `main` で起動する（`workflow_dispatch`）。デプロイする所を 1 つに保つため、`release.yml` は Pages に直接は出さない。
  3. `manual.yml` が最新 Release の `hello-pasta.nar` を取り、`book/book/hello-pasta/` へ展開する（新版の検査の後・アーティファクトのアップロードの前）。展開した各ファイルの md5 と大きさが、`updates.txt` の行と合うことを確かめ、合わなければ公開を止める。
  4. 公開後、`release.yml` が実際の URL の `updates.txt` を取り、Release 版と見比べる。結果は `report` の表に 1 行増やす。
- 置き場所の中身は nar そのものなので、dll と辞書の版のずれは起きない。将来 `pasta-dll-multiarch` が arch ごとの dll を nar に入れても、そのまま置き場所に出る。
- 展開する範囲（nar 全体か、ゴースト・シェル・ルートのファイルだけで `bal/` を除くか）は、要件定義で決める。

**退けた案**

- **Release の添付だけで配る**: 平らな名前だけで、`ghost/master/pasta.dll` のような構造を置けない。
- **専用ブランチ（履歴を持たない 1 コミット）に置いて raw で配る**: 動くが、CI が git に push する形になる。配信元が Pages より素朴で、マニュアルと置き場所が別になる。
- **main の raw を直接指す**: dll・`updates.txt`・`scripts/` は生成物で git に入れていないため、置けない。

## Scope

- **In**:
  - `descript.txt` の `homeurl` を、更新の置き場所に変える。
  - `manual.yml`: 最新 Release の nar を `book/book/hello-pasta/` へ展開する段と、`updates.txt` との整合の検査（失敗で公開中断。PR・ブランチでも検査まで走らせる）。
  - `release.yml`: `github-release` の後に、`manual.yml` を起動して完了を待ち、公開後の URL の `updates.txt` を Release 版と見比べるジョブ。`report` の表への 1 行。必要な権限（`actions`）。
  - 置き場所の入口（`hello-pasta/index.html`）。SSP の「配布元 URL を開く」が `homeurl` を開くとき、404 にならないよう、ダウンロードのページ（`book/src/download.md`）へ案内する。
  - 派生ゴーストの事故の防止: マニュアルに「hello-pasta をコピーしたら `homeurl` を消すか直す」を書く。`pasta_check validate` が、`name` が hello-pasta でないのに `homeurl` が hello-pasta の更新の置き場所を指していれば警告する（`FindingKind` に足す）。
  - リリース手順書（`crates/pasta_sample_ghost/RELEASE.md`・`release` スキル・`.github/release-ci-setup.md`）の更新。
  - 実機の SSP（別ポートの別インスタンス）での確認: ローカルの HTTP サーバーに置き場所を作り、更新確認で `pasta.dll` を含む一式が差し替わること。
- **Out**:
  - 作者が自分のゴーストをネットワーク更新に対応させる方法の案内（別の話。`pasta_check release` が `updates.txt` を作るところまでは既にある）。
  - 同梱バルーンの `homeurl`（バルーン自身の持ち主が決める。`bal/updates.txt` は既存のまま）。
  - hello-pasta の中に、更新を知らせる台詞やメニューを足すこと。
  - dll の arch ごとの配布（`pasta-dll-multiarch`）。

## Boundary Candidates

- 置き場所の組み立てと検査（`manual.yml` の展開の段と整合の検査）
- リリース CI のつなぎ（起動・待ち・公開後の見比べ・report の行）
- ゴースト側の設定（`homeurl`）と、派生ゴーストの事故の防止（マニュアルの注意・`pasta_check validate` の警告）
- 実機確認

## Out of Boundary

- マニュアルの章立て・見た目（このページに触れるのは `download.md` への案内と、派生ゴーストの注意の 1 節だけ）
- `pasta_check release` の `updates.txt` の作り方そのもの（変えない。この spec は出力を使う側）
- Release の添付の名前・数（変えない）

## Upstream / Downstream

- **Upstream**: `release-workflow`・`release-ci`（完了。`release.yml` のジョブ構成）、`manual-download-page`（完了。`download.md` と固定リンクの名前の検査）、`pasta-check-bundled-balloon`（完了。`updates.txt` の作り）
- **Downstream**: `pasta-dll-multiarch`（arch ごとの dll は、nar に入れば置き場所にも出る。この spec は nar の中身に依らない作りにする）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `pasta-dll-multiarch`（未完了）: `release.yml` と `release.ps1` を共有する。同じウェーブに置かない。リリースの実行中は `release.yml` を main へ入れない。
  - `manual-download-page`（完了）: `manual.yml` の起動条件に `release.yml` が入っていて、`release.yml` の変更でマニュアルの CI が動く。固定リンクの名前の検査（`link-check.mjs` の `checkReleaseAssetLinks`）が、`release.yml` の 1 行 1 つの `release/<名前>` の行を読む。新しいジョブの書き方が、この検査を壊さないこと。

## Constraints

- Pages のデプロイは `manual.yml` の 1 か所に保つ（2 つのワークフローが Pages へ出すと、後の方が先の方を消す）。
- `manual.yml` の `deploy` ジョブだけが `pages: write`・`id-token: write` を持つ（ワークフロー全体は `contents: read`）。展開の段が最新 Release を取るのに追加の権限が要るなら、`build` ジョブだけに最小で足す。
- `github-pages` の environment は `main` だけに公開を許す。`release.yml` が起動するのは `main` の `manual.yml`。
- 展開した置き場所は、マニュアルの静的性・検索・コンテンツの検査の対象に入れない（旧版 `classic/` と同じく、検査の後に足す）。
- リリースが出ていない・nar が無いときの扱いは、要件定義で決める（最初のリリースより前は、前の版の nar を置くことになる）。
- 失敗した公開は、再実行で直せる形にする（`release.yml` の再実行、または `manual.yml` の手動実行）。リリース手順書に回復を書く。
- 版が上がるまで、既存の利用者の nar の `homeurl` は古いまま（更新は、`homeurl` を足した版の nar を入れた利用者から効く）。

## 確かめていないこと（要件の前に確かめる）

- SSP が、動いている `pasta.dll` を更新で差し替えられること（dll ゴーストでは普通だが、pasta では未確認）。
- 手元で書き換えた辞書・`pasta.toml` が、更新で上書きされること（SSP は、手元の md5 とサーバーの md5 が違うファイルを取り直すと理解している）。上書きされるなら、利用者向けの注意をどこに書くか。
- Pages が `.dll`・`.pasta`・`.png` などを、そのまま返すこと（静的配信なので問題ないはずだが、実際の URL で確かめる）。
- `GITHUB_TOKEN` で `workflow_dispatch` を起動し、完了を待つのに要る権限（`actions: write`・`actions: read`）。
