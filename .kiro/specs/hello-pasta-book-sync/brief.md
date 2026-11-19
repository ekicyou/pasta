# Brief: hello-pasta-book-sync

> **ステータス**: 未着手（2026-11-19 の `/kiro-discovery` で起票）。CI とスクリプトの spec で、main に入れば効く（リリースを待たない）。要件定義は Opus。

## Problem

hello-pasta のネットワーク更新の中身が、マニュアル（book）と同じ時機に変わらない。

- 開発者の指示は「ネットワーク更新は book に同期する」である（2026-11-19 の確認。「もともと、ゴーストリリースは book に同期せよ、と指示している」「ゴーストリリースというか、ネットワークアップデートに限った話。nar はリリースタイミングで作ればよい」）。
- 完了済みの `hello-pasta-network-update` は、この「同期」を「book と同じデプロイに載せる」と狭く作った。置き場所の中身は、最新 Release の `hello-pasta.nar` を展開したものである。main の辞書を直しても、リリースを出すまで置き場所は変わらない。
- そのため、辞書を直した直後に、実機でネットワーク更新を確かめられない（開発者「変わってもらえないと今のテストができない」）。

## Current State

2026-11-19（main `fcc3ecd0`、最新 Release は v0.6.2）に確かめた。

- **置き場所の作り方**: `manual.yml` の `build` ジョブ（ubuntu）が、`releases/latest/download/hello-pasta.nar` を取り、`ghost-site.mjs build` で展開・照合して `book/book/hello-pasta/` に置く（`manual.yml` の「Fetch latest hello-pasta.nar」「Build hello-pasta update site」）。
- **起動の条件**: `manual.yml` は、hello-pasta では `dic/**` と `calendar.csv` の変更で走る（入門の章との照合のため）。シェル・`pasta.toml`・`descript.txt`・dll のソースの変更では走らない。
- **起きたこと**: PR #288 で `07-greeting.pasta` に更新イベントの場面を足した。`manual.yml` は走り、マニュアルの 7 章は新しい台詞で公開された。置き場所は v0.6.2 の nar のままで、SSP の更新のログは `No Update` である。
- **nar の作り方**: `crates/pasta_sample_ghost/release.ps1` が、x86 の `pasta.dll` を `cargo build --release --target i686-pc-windows-msvc` で作り、`cargo about` でライセンスの一覧を作り、`pasta_check release` で `updates.txt` と nar を作る。Windows のランナーが要る。
- **リリース CI**: `release.yml` の `publish-ghost-site` ジョブが、Release の公開の後に `manual.yml` を main で起動して待ち、実 URL の `updates.txt` を Release の nar のものと見比べる（`publish-ghost-site.ps1`。最新のタグが `vX.Y.Z` の形であることを前提にする）。
- **SSP の更新の動き**（`hello-pasta-network-update` の research）: 手元の `updates.txt` とサーバーのものを比べ、md5 が変わった行のファイルだけを取る。消えたファイルは `delete.txt` が無いと消さない。

## Desired Outcome

**要件はひとつである（開発者 2026-11-19）: 辞書が変わったら、`pasta.dll` が変わったら、ネットワーク更新に出る。**

- main の hello-pasta が変わると、book の公開と同じ run で、ネットワーク更新の置き場所も同じコミットの中身に変わる。リリースを待たない。
- 開発者は、辞書を直してマージした後、手元の SSP の更新確認で、その変更が届くことを確かめられる。
- 更新の粒度はファイル単位である（開発者「nar 作成と、ネットワークアップデートでは更新粒度が違う」）。nar は、リリースのときの一式のまとまり。ネットワーク更新は、変わったファイルだけが、変わったときに届く。中身が変わっていないファイルは、置き場所を作り直しても md5 が変わらず、利用者は取り直さない。
- `hello-pasta.nar`（Release の添付）は、今のままリリースのときにだけ作る。
- 壊れた中身（読み込めない辞書・dll と合わない辞書）を、置き場所に出さない。
- Release の nar で入れた直後の利用者が、更新で古い中身に戻されない。

## Approach

**`manual.yml` が、公開するコミットから置き場所を直接作る。Release の nar は取らない。**

```
main に push（book か hello-pasta が変わる）
  └ manual.yml
      ├ build-ghost（windows）: そのコミットから hello-pasta の配布の中身と updates.txt を作る
      ├ build（ubuntu）: マニュアルを作り、build-ghost の成果物を book/book/hello-pasta/ に足して照合する
      └ deploy: これまでどおり 1 つのサイトとして公開する
```

- 置き場所は「nar を展開したもの」ではなく、「ファイルごとに最新の中身を置いたもの」と考える。置くファイルの集まりと `updates.txt` の形は、`pasta_check release` が nar に入れるものと同じにする（一覧の作り方を 2 つ持たない）。nar そのものは作らない。
- 辞書・シェル・設定のファイルは、リポジトリのファイルそのままなので、変わったコミットでだけ md5 が変わる。ビルドして作るファイル（`pasta.dll`・ライセンスの一覧）は、中身が変わったときだけ md5 が変わるようにする（「設計で確かめること」の 1）。
- `ghost-site.mjs` の展開と照合（`updates.txt` の全行の md5・大きさ）は、そのまま使う。入力が「Release の nar」から「同じ run で作った成果物」に替わる。
- リリースのときも、`release.yml` から `manual.yml` を起動する今の流れは残す。リリースのコミットで置き場所を作り直さないと、Release の nar で入れた利用者が、古い置き場所の中身に戻される。

**退けた案**

- **ゴースト専用のタグで、hello-pasta の更新だけを出す**: 開発者の指示（book に同期）と別の仕組みになる。タグを打つ手間も残る。
- **今のまま、辞書の変更はリリースで届ける**: 指示に合わない。直した直後に確かめられない。
- **辞書だけ main から取り、dll は最新 Release のものを置く**: dll が変わっても更新に出ない。新しい dll と一緒に入った辞書（13 段目の `＄行事` など）が、古い dll と組になって届く。

## 決まったこと

- `pasta.dll` は、辞書と同じコミットでビルドしたものを置く。main の dll が変われば、更新に出る。
- 起動の条件は、book の公開に加えて、hello-pasta の配下（辞書・シェル・`pasta.toml`・`descript.txt`）と、dll の中身を変えるソースの変更である。
- 検査や組み立てが落ちたら、今と同じく公開ごと止める。欠けた置き場所を出さない。
- リリースの間に届く dll は、最後に出した版の番号を名乗る。見分ける印は足さない。

## 設計で確かめること

開発者の判断は要らない。設計で調べて決める。

1. **dll の md5 が、ソースが同じなら毎回同じになるか**。同じでないと、辞書を 1 行直しただけで、利用者が約 3.6MB の dll を取り直す。2 回ビルドして比べる。違うなら、dll のソースが変わらない間は、前に公開した dll を引き継ぐ。
2. **置き場所を作る前に通す検査**。main は CI を待たずにマージするので、赤いことがある。`pasta_sample_ghost`・`pasta_shiori` のテストと、`release.yml` の `build` が nar に行っている検査のうち、どれを通すか。
3. **dll の中身を変えるソースの範囲**（起動の条件の `paths`）。`crates/pasta_shiori` と、それが依存するクレート・`Cargo.lock`・`.cargo/config.toml`。
4. **リリース CI の見比べの移し替え**。今は「実 URL の `updates.txt` ＝ Release の nar の `updates.txt`」を確かめる。置き場所が main 由来になると一致しないので、`manual.yml` が公開の後に自分で実 URL を確かめ、`release.yml` の側は起動と結果の報告だけにする。
5. **消えたファイル**。`pasta_check release` が `delete.txt` を作るかを調べる。作らないなら、改名・削除したファイルが利用者の手元に残る。扱いを決める。

## Scope

- **In**:
  - `manual.yml`: 置き場所を、公開するコミットから作る（Windows のジョブと成果物の受け渡し）。Release の nar の取得をやめる。起動の条件の見直し。
  - 置き場所を作るスクリプト（`release.ps1` の中身のうち、hello-pasta の配布の中身と `updates.txt` を作る所の使い回し）。
  - `ghost-site.mjs`: 入力の形の変更に要る直し。
  - `release.yml` の `publish-ghost-site`・`publish-ghost-site.ps1`・`report` の行: 見比べの移し替え。自己テスト（`release-scripts-test.ps1`）の追従。
  - 手順書（`crates/pasta_sample_ghost/RELEASE.md`・`release` スキル・`.github/release-ci-setup.md`）と、`hello-pasta-network-update` の要件のうち置き換わる項の記録。
  - 実機確認: v0.6.2 を入れた SSP の更新確認で、main の中身に入れ替わり、`OnUpdateComplete`（`changed`）の台詞が出ること。PR #288 の台詞の、最初の本番の確認を兼ねる。
- **Out**:
  - `hello-pasta.nar`・`pasta-starter.nar` と Release の添付の作り方・時機（リリースのときのまま）。
  - ゴースト専用のリリース・タグ。
  - 作者が自分のゴーストをネットワーク更新に対応させる方法の案内。
  - `pasta_check release` の `updates.txt` の作り方（設計で `delete.txt` を足すと決めたときを除く）。

## Boundary Candidates

- 置き場所の組み立て（Windows のビルドと、配布の中身・`updates.txt` の生成）
- マニュアル公開 CI のつなぎ（ジョブの分け方・成果物の受け渡し・起動の条件・関門）
- リリース CI の後始末（起動は残す・見比べを移す・報告の行）
- 実機確認

## Out of Boundary

- マニュアルの中身と、入門の章と辞書の照合（`tutorial-check`）。
- hello-pasta の台詞・シェル。
- `pasta.dll` の arch ごとの配布（`pasta-dll-multiarch`）。

## Upstream / Downstream

- **Upstream**: `hello-pasta-network-update`（完了。置き場所・`ghost-site.mjs`・`publish-ghost-site`）、`pasta-check-bundled-balloon`（完了。`updates.txt` の作り）、`release-ci`（完了。`release.yml`）。
- **Downstream**: `pasta-dll-multiarch`（nar に arch ごとの dll を足す。置き場所の作り方がこの spec の形になる）。

## Existing Spec Touchpoints

- **Extends**: `hello-pasta-network-update`（完了）。要件 4（置き場所は最新 Release の nar から作る）と要件 5（公開後の見比べ）を置き換える。
- **Adjacent**: `pasta-dll-multiarch`（`release.ps1`・`release.yml` を触る。同じウェーブに置かない）。リリースの実行中は、`release.ps1`・`release.yml` を触る変更を main へ入れない（roadmap「リリース」の席の決まり）。

## Constraints

- Pages は 1 サイトを丸ごと入れ替える。デプロイする所は `manual.yml` の `deploy` ジョブ 1 つに保つ。
- `manual.yml` の全体の権限は `contents: read` のまま。公開の権限は `deploy` ジョブだけが持つ。
- x86 の `pasta.dll` は Windows のランナーでしか作れない。静的 CRT リンクの決まりを変えない。
- マニュアルだけの PR の検証を、長いビルドで遅くしすぎない（Windows のジョブはキャッシュを使い、PR では要るときだけ走らせるかを設計で決める）。
- 置き場所は、すでに利用者の SSP が見ている。切り替えの途中で、欠けた置き場所や、古い中身に戻す置き場所を公開しない。
