# Brief: hello-pasta-book-sync

> **ステータス**: 未着手（2026-11-19 の `/kiro-discovery` で起票）。CI とスクリプトの spec で、main に入れば効く（リリースを待たない）。要件定義は Opus。

## Problem

hello-pasta のネットワーク更新の中身が、辞書を直しても変わらない。

- 開発者の指示は「ネットワーク更新は book に同期する」である（2026-11-19 の確認。「もともと、ゴーストリリースは book に同期せよ、と指示している」「ゴーストリリースというか、ネットワークアップデートに限った話。nar はリリースタイミングで作ればよい」）。
- 完了済みの `hello-pasta-network-update` は、この「同期」を「book と同じデプロイに載せる」と狭く作った。置き場所の中身は、最新 Release の `hello-pasta.nar` を丸ごと展開したものである。main の辞書を直しても、リリースを出すまで置き場所は変わらない。
- そのため、辞書を直した直後に、実機でネットワーク更新を確かめられない（開発者「変わってもらえないと今のテストができない」）。

## Current State

2026-11-19（main `fcc3ecd0`、最新 Release は v0.6.2）に確かめた。

- **置き場所の作り方**: `manual.yml` の `build` ジョブ（ubuntu）が、`releases/latest/download/hello-pasta.nar` を取り、`ghost-site.mjs build` で展開・照合して `book/book/hello-pasta/` に置く（「Fetch latest hello-pasta.nar」「Build hello-pasta update site」）。
- **走るとき**: `manual.yml` は、main への push のうち `book/**` と、hello-pasta の `dic/**`・`calendar.csv` の変更で走る。リリースのときは、`release.yml` の `publish-ghost-site` ジョブが `manual.yml` を main で起動する。走るときは、すでに開発者の言う 2 つ（下の「Desired Outcome」）に合っている。合っていないのは、置く中身である。
- **置き場所のファイル**: `updates.txt` の一覧は 40 本。そのうち git に入っていない（ビルドで作る）のは、`ghost/master/pasta.dll`・`ghost/master/THIRD_PARTY_LICENSES.txt`・`ghost/master/scripts/README.md` の 3 本だけである。残りの 37 本（辞書・`calendar.csv`・`pasta.toml`・`descript.txt`・`install.txt`・シェルなど）は、リポジトリの `crates/pasta_sample_ghost/ghosts/hello-pasta/` にそのまま在る。
- **起きたこと**: PR #288 で `07-greeting.pasta` に更新イベントの場面を足した。`manual.yml` は走り、マニュアルの 7 章は新しい台詞で公開された。置き場所は v0.6.2 の nar のままで、SSP の更新のログは `No Update` である。
- **`updates.txt` の作り手**: `pasta_check release`（Rust）だけである。`ghost-site.mjs` は読んで照合するだけで、作らない。
- **リリース CI の見比べ**: `publish-ghost-site.ps1` が、実 URL の `updates.txt` と Release の nar の `updates.txt` のバイト一致を確かめる。
- **SSP の更新の動き**（`hello-pasta-network-update` の research）: 手元の `updates.txt` とサーバーのものを比べ、md5 が変わった行のファイルだけを取る。消えたファイルは `delete.txt` が無いと消さない。

## Desired Outcome

**要件はひとつである（開発者 2026-11-19）: 辞書が変わったら、`pasta.dll` が変わったら、ネットワーク更新に出る。**

ネットワーク更新の CI は、次の 2 つをきっかけに出す。

| きっかけ | いつ | 置き場所で変わるもの |
| -------- | ---- | -------------------- |
| ゴーストの辞書が変わった・book が変わった | main への push | 辞書・シェル・設定など、リポジトリに在るファイル |
| `pasta.dll` が変わった | リリース | `pasta.dll` など、ビルドで作るファイル |

- 更新の粒度はファイル単位である（開発者「nar 作成と、ネットワークアップデートでは更新粒度が違う」）。nar は、リリースのときの一式のまとまり。ネットワーク更新は、変わったファイルだけが、変わったときに届く。
- `hello-pasta.nar`（Release の添付）は、今のままリリースのときにだけ作る。
- 開発者は、辞書を直してマージした後、手元の SSP の更新確認で、その変更が届くことを確かめられる。
- Release の nar で入れた直後の利用者が、更新で古い中身に戻されない。

## Approach

**置き場所は、2 つの出どころを重ねて作る。**

```
置き場所のファイル ＝ main のリポジトリに在る hello-pasta のファイル（37 本）
                  ＋ 最新 Release の nar にだけ在るファイル（pasta.dll など 3 本）
```

- `manual.yml` は、今のとおり最新 Release の nar を取る。そこから使うのは、リポジトリに無いファイル（ビルドで作るもの）だけにする。リポジトリに在るファイルは、公開するコミットのものを置く。
- 重ねた結果から `updates.txt` を作り直し、今と同じ照合（全行の md5・大きさ）を通してから公開する。
- 分け方は「リポジトリに在るか」で決める。ファイル名を決め打ちで並べない。`pasta-dll-multiarch` が arch ごとの dll を nar に足しても、そのまま Release の側に入る。
- dll は Release のものをそのまま置くので、リリースの間は md5 が変わらない。辞書を直しても、利用者が dll を取り直すことは無い。
- 走るときは今のままでよい。hello-pasta の配下のうち、今の `paths` に無いもの（シェル・`pasta.toml`・`descript.txt` など）を足す。

**退けた案**

- **今のまま、辞書の変更はリリースで届ける**: 指示に合わない。直した直後に確かめられない。
- **dll も main のコミットごとにビルドして置く**: dll の粒度はリリースである（開発者の決定）。版の付かない dll を配ることになり、Windows のビルドも `manual.yml` に要る。
- **ゴースト専用のタグで、hello-pasta の更新だけを出す**: 指示と別の仕組みになる。タグを打つ手間も残る。

## 設計で確かめること

1. **`updates.txt` をどこで作るか**。作り手は `pasta_check release`（Rust）だけである。`manual.yml`（ubuntu）で `pasta_check` をビルドして使うか、`ghost-site.mjs` に同じ形の一覧を作る処理を足すか。後者は作り方が 2 つになるので、`pasta_check` の出力と一致することのテストが要る。
2. **新しい dll が要る辞書**。機能と見本の辞書を同じ PR で足すと（13 段目の `＄行事` がそうだった）、次のリリースまでの間、新しい辞書と古い dll の組が置き場所に出る。止める検査を置くか（最新 Release の `pasta_check` で、重ねた結果を検査するなど）、落ちたときにどうするか（公開ごと止める・Release の nar の中身のまま出す）を決める。
3. **リリース CI の見比べ**。重ねて作り直した `updates.txt` は、Release の nar のものとバイトでは一致しないことがある（日付の欄）。リリースのコミットでは中身は同じはずなので、何を比べるか（md5 と大きさの行）を決める。
4. **消えたファイル**。main で改名・削除したファイルは、置き場所の一覧から消えるが、利用者の手元には残る。`delete.txt` の扱いを決める。

## Scope

- **In**:
  - `manual.yml`: 置き場所を「main のファイル＋最新 Release の nar にだけ在るファイル」で作る。起動の条件（`paths`）に hello-pasta の配下を足す。
  - `ghost-site.mjs` とその自己テスト: 重ねる処理と、`updates.txt` の作り直し（設計の 1）。
  - `release.yml` の `publish-ghost-site`・`publish-ghost-site.ps1`・自己テスト: 見比べの直し（設計の 3）。
  - 手順書（`crates/pasta_sample_ghost/RELEASE.md`・`release` スキル・`.github/release-ci-setup.md`）と、`hello-pasta-network-update` の要件のうち置き換わる項の記録。
  - 実機確認: v0.6.2 を入れた SSP の更新確認で、main の辞書に入れ替わり、`OnUpdateComplete`（`changed`）の台詞が出ること。PR #288 の台詞の、最初の本番の確認を兼ねる。
- **Out**:
  - `hello-pasta.nar`・`pasta-starter.nar` と Release の添付の作り方・時機（リリースのときのまま）。
  - `pasta.dll` を main のコミットごとにビルドして配ること。
  - ゴースト専用のリリース・タグ。
  - 作者が自分のゴーストをネットワーク更新に対応させる方法の案内。
  - `pasta_check release` の `updates.txt` の作り方。

## Boundary Candidates

- 置き場所の組み立て（重ねる・`updates.txt` を作り直す・照合する）
- マニュアル公開 CI のつなぎ（起動の条件・新しい dll が要る辞書の検査）
- リリース CI の見比べ
- 実機確認

## Out of Boundary

- マニュアルの中身と、入門の章と辞書の照合（`tutorial-check`）。
- hello-pasta の台詞・シェル。
- `pasta.dll` の arch ごとの配布（`pasta-dll-multiarch`）。

## Upstream / Downstream

- **Upstream**: `hello-pasta-network-update`（完了。置き場所・`ghost-site.mjs`・`publish-ghost-site`）、`pasta-check-bundled-balloon`（完了。`updates.txt` の作り）、`release-ci`（完了。`release.yml`）。
- **Downstream**: `pasta-dll-multiarch`（nar に arch ごとの dll を足す。Release の側のファイルとしてそのまま置かれる）。

## Existing Spec Touchpoints

- **Extends**: `hello-pasta-network-update`（完了）。要件 4（置き場所は最新 Release の nar から作る）と要件 5（公開後の見比べ）を置き換える。
- **Adjacent**: `pasta-dll-multiarch`（`release.yml` を触る。同じウェーブに置かない）。リリースの実行中は、`release.yml` を触る変更を main へ入れない（roadmap「リリース」の席の決まり）。

## Constraints

- Pages は 1 サイトを丸ごと入れ替える。デプロイする所は `manual.yml` の `deploy` ジョブ 1 つに保つ。
- `manual.yml` の全体の権限は `contents: read` のまま。公開の権限は `deploy` ジョブだけが持つ。
- マニュアルだけの PR の検証を遅くしすぎない。
- 置き場所は、すでに利用者の SSP が見ている。欠けた置き場所や、古い中身に戻す置き場所を公開しない。
