# Implementation Plan

- [x] 1. 最初の関門: SSP が `updates.txt` の日付の欄を比べに使わないことを確かめる
  - 公開中の v0.6.2 の置き場所を写し、`updates.txt` の各行の日付の欄だけを書き換えて、ローカルの HTTP サーバーに置く
  - v0.6.2 の `hello-pasta.nar` を入れた、別ポートの複製した SSP（bootunlock・standalone）の `homeurl` をそのローカルの置き場所に向け、更新確認をする
  - SSP の更新のログが `No Update` で、ファイルを 1 本も取っていないことを確かめ、ログの抜き書きを `research.md` の「実機確認」の節に記録する
  - `No Update` にならず、日付の違うファイルを取り直したら、ここで作業を止めて開発者に報告する（作りが成り立たないため、後のタスクに進まない）
  - _Dispatch: standard — 実機の SSP を手で動かして判定する確認_
  - _Requirements: 1.6, 10.2, 10.5_

- [x] 2. 置き場所の部品（nar の側の規則と組み立ての台本）
- [x] 2.1 nar の側を選ぶ規則と `nar-only` 命令を足す
  - 用意: `gh release download v0.6.2 --pattern hello-pasta.nar` で v0.6.2 の nar をスクラッチに取り、`git fetch origin refs/tags/v0.6.2:refs/tags/v0.6.2` でタグを手元に置く（2.3・5.2 も使う）
  - タグのツリーを相対の名前の集合として読む処理を足す。読めないタグでは、git の文を添えた誤りになる
  - nar のファイルのうち、タグのツリーに無く、最後の部品が除外の 3 つの名前（`update_files.rs` の `EXCLUDED_FILES` と同じ）に当たらないものだけを選ぶ規則を足す。ファイル名の一覧は持たない
  - `nar-only` 命令は、nar を今の照合に通し、選んだファイルを nar のバイトのまま出力先に書き、書いた名前を 1 行ずつ出す。照合の問題・git の失敗・選んだものが 0 本・出力先が空でない、で終了コード 1
  - 自己テストに、一時の git リポジトリとタグを使う項目を足す: ツリーに在るファイル（main で消したものを含む）が残らない、どの階層の `updates.txt`・`updates2.dau`・`developer_options.txt` も残らない、書いたバイトが nar と同じ、nar に `updates.txt` が無いと何も書かずに失敗する、無いタグで git の文が出る
  - `node book/tools/ghost-site/ghost-site-test.mjs` が全部通る。v0.6.2 の nar と `refs/tags/v0.6.2` で `nar-only` を動かすと、`ghost/master/pasta.dll`・`ghost/master/THIRD_PARTY_LICENSES.txt`・`ghost/master/scripts/README.md` の 3 本だけが出る
  - _Depends: 1_
  - _Dispatch: standard — 規則と CLI の新しい処理を書き、自己テストを足す_
  - _Requirements: 1.2, 1.3, 1.4, 1.5, 1.6, 2.6, 7.1_

- [x] 2.2 公開後の見比べを「nar の側の行」の一致に変える
  - `verify-live` に必須の `--tree` を足す。比べる行は、nar の `updates.txt` のうち 2.1 の規則で選ばれたファイルの行だけにする
  - 実 URL の `updates.txt` に、比べる行が全部、同じパス・md5・大きさで載れば一致とする。日付の欄と、比べる行の外の行は見ない
  - 取り直しの回数・間隔・HTTP 200 以外や解釈できないときの取り直しは今のまま。比べる行が 0 本なら、取らずに失敗する
  - `--full` は、一致の後に実 URL の全部の行のファイルを 1 回ずつ取り、その行の md5・大きさと照合する
  - 成功の文を「`OK updates.txt: <n> 回目で nar の側の <k> 行が一致`」にする（`publish-ghost-site.ps1` が回数を読む「<n> 回目で」は保つ）
  - 自己テストで、日付の欄とリポジトリの側の行が違っても一致・md5 か大きさの違いや行の欠けは上限で失敗・比べる行 0 本で取らずに失敗・`--tree` 無しで使い方の誤り、が確かめられ、`ghost-site-test.mjs` が全部通る
  - _Depends: 2.1_
  - _Dispatch: standard — 既存の見比べの判定の中身を書き替える_
  - _Requirements: 6.1, 6.2, 6.3_

- [x] 2.3 置き場所を組み立てる台本を書き、タグのコミットで nar と一致することを確かめる
  - 用意: 最新 Release と同じ版の `pasta_check` を `cargo install pasta_check --version 0.6.2 --locked --root <スクラッチ>` で入れ、`PASTA_CHECK` をそれに向ける（手元の開発版は検査が違いうるので使わない）
  - 引数はコミット・タグ・Release の nar・出力先。リポジトリのルートで動き、`pasta_check` は `PATH` から（環境変数 `PASTA_CHECK` で上書きできる）取る。どの段の失敗も終了コード 1
  - リポジトリの側は、コミットの hello-pasta の配下を `core.autocrlf=true` の `git archive` で取り出し、除外の 3 つの名前を外す。取り出しが 0 本なら失敗する
  - nar の側は 2.1 の `nar-only` で取り出し、`pasta_check release --target <リポジトリの側> --copy <nar の側>` で重ね・検査・`updates.txt` の作り直しを 1 回で行う。最後に、重ねた nar を今の `build` で照合して出力先に置く（入口のページも今のとおり足す）
  - `delete.txt` は作らない。除外の名前は台本と `ghost-site.mjs` の 2 か所に在り、どちらにも「`update_files.rs` の `EXCLUDED_FILES` と同じ」と書く
  - 作り手は `pasta_check release` だけなので、別の作り手との一致のテストは要らないことを台本の注釈に書く
  - 手元で `assemble-site.sh <v0.6.2 のコミット> v0.6.2 <v0.6.2 の nar> <出力先>` を動かすと通り、できた `updates.txt` が公開中の v0.6.2 の `updates.txt` と日付の欄を除いて全部の行で一致する。結果を `research.md` に記録する
  - 公開するコミットを main の先頭にすると、md5 が v0.6.2 と違う行は、`git diff --name-only v0.6.2 HEAD -- crates/pasta_sample_ghost/ghosts/hello-pasta/` のファイルとちょうど同じで、`pasta.dll` の行は同じままである
  - _Depends: 2.1_
  - _Dispatch: standard — 既存の道具をつなぐ新しい台本と実物での照合_
  - _Requirements: 1.1, 1.5, 1.7, 1.8, 2.1, 2.2, 2.3, 2.4, 2.5, 3.3, 7.1, 7.2_

- [x] 3. CI へのつなぎ込み
- [x] 3.1 マニュアル公開 CI の置き場所の段を新しい組み立てに替える
  - 起動の条件の hello-pasta の 2 行を、hello-pasta の配下全部の 1 行に替える（push・pull_request の両方）
  - concurrency の `pages` の組を main の run だけにし、ほかは event と ref ごとの組にする
  - 今の「nar を取る」「置き場所を作る」の 2 段を、最新 Release の tag の特定（`vX.Y.Z` の形でなければ失敗）・その tag の nar の取得とタグの浅い fetch・`pasta_check` のキャッシュの取り戻し・外れたときだけの `cargo install --locked` と直後の保存・`PATH` への追加・2.3 の台本の実行、に替える
  - 権限は全体の `contents: read` のまま。マニュアルの段と `tutorial-check` には触れない。冒頭の注釈を新しい段に合わせる
  - 実装のブランチを origin に push する（spec と設計の段では push していない。PR はまだ作らない）。ブランチで `gh workflow run manual.yml --ref <branch>` を回すと、置き場所の段が通り、公開はされない（Upload・`deploy` が動かない）
  - _Depends: 2.3_
  - _Dispatch: standard — ワークフローの段の組み替えとキャッシュの新しい使い方_
  - _Requirements: 2.6, 2.7, 3.1, 3.2, 3.3, 3.4, 3.5, 4.1, 4.3, 4.4, 8.3, 8.5, 8.7_

- [x] 3.2 古い run の公開を止める見分けの段と、公開の条件を足す
  - main の run だけ、組み立ての後・Upload の前に、見分けの段を足す。今の最新 Release の tag が組み立ての tag と違えば公開しない。main の先頭が自分と同じなら公開する。違えば、自分から先頭までに起動の条件のファイル（`yq` で `on.push.paths` を読み、`:(glob)` を付ける）が変わったときだけ公開しない
  - 公開しないときは理由を `::notice::` と summary に出し、成功で終える。`gh api`・`git fetch` の失敗はジョブの失敗にする
  - 見分けの結果を `build` ジョブの出力にし、Upload と `deploy` の条件を「main かつ公開する」にする。`deploy` は 1 か所のまま
  - ブランチの run で `yq --version` を出して在ることを確かめる。無ければ、起動の条件を Node で読む小さな処理に替える
  - 段の `run:` の本文には `${{ }}` を書かず、組み立ての tag・`GITHUB_SHA`・`GH_TOKEN` などは全部 `env:` から取る（本文をそのまま bash で動かせるようにする）
  - 判定の 4 つの枝（tag が変わった・先頭と同じ・差あり・差なし）を、段の `run:` の本文を一時のファイルに書き出し、手元の一時の git リポジトリで `GITHUB_SHA` などを与え、`releases/latest` の答えは `PATH` の先頭に置いた偽の `gh` で返して動かして確かめ、その結果を `research.md` に記録する
  - _Depends: 3.1_
  - _Dispatch: standard — 設計が判定の形を与えた分岐の実装_
  - _Requirements: 2.7, 5.1, 5.2, 5.3, 5.4, 8.4_

- [x] 3.3 (P) リリース CI の見比べに `--tree` を渡す
  - `publish-ghost-site.ps1` が、`<タグ>:crates/pasta_sample_ghost/ghosts/hello-pasta` を `--tree` として `verify-live` に渡す。hello-pasta のパスはスクリプトの中の決まった文字列にし、`release.yml` は変えない
  - 説明と結果の文を「Release の nar の側の行と一致」に替える。古いタグの判定・起動・待ち・`status`・`reason`・回数・間隔・summary の形は変えない
  - `release-scripts-test.ps1` の項目 22 の引数の照合に `--tree v0.0.2:crates/pasta_sample_ghost/ghosts/hello-pasta` を足し、項目 18〜28 が全部通る
  - _Boundary: PublishGhostSite_
  - _Depends: 2.2_
  - _Dispatch: mechanical — 設計が引数と文の形を与えた追記_
  - _Requirements: 6.1, 6.4, 6.5, 8.1_

- [ ] 4. 手順書と記録
- [ ] 4.1 RELEASE.md を新しい置き場所の作り方に合わせる
  - 置き場所が「main のリポジトリのファイル＋最新 Release の nar にだけ在るファイル」で作られること、出る 2 つのきっかけ（main への push・リリース）を書く
  - リリースの直後の数分の間（nar の側が 1 つ前の Release のままでありうること）を例外として書く
  - 検査で止まったときの読み方と直し方（リリースを出す・辞書を戻す）、Lua を動かさない検査なので止められない違い、見比べの読み方、消えたファイルの注意（今のまま）を書く
  - 手元で `assemble-site.sh` を動かして置き場所を組み立てる手順を書く
  - RELEASE.md に上の項が全部在り、手順のコマンドが 2.3 の台本の引数と一致する
  - _Depends: 2.3_
  - _Dispatch: standard — 設計が項目を与え、文は書き起こす_
  - _Requirements: 4.6, 5.1, 9.1, 9.5_

- [ ] 4.2 (P) release スキル・release-ci-setup.md・前の仕様の記録を合わせる
  - `.claude/skills/release/procedure.md` の結果表の見比べの行を「nar の側の行」に合わせる。編集が拒否されたら、入った分と残りを示して開発者の許可を求める
  - `.github/release-ci-setup.md` の置き場所の作り方の記述を合わせ、設定の変更は要らないことを書く
  - `hello-pasta-network-update` の要件の置き換わる項（1.2・1.4・4.3・4.8・5.1・9.6 など）に、この仕様で置き換わったことを書き足す
  - 3 つのファイルの該当の箇所が新しい作り方を書き、古い「nar の展開」「バイト一致」の記述が残っていない
  - _Boundary: Docs_
  - _Dispatch: mechanical — 設計が直す箇所と中身を与えた書き換え_
  - _Requirements: 9.2, 9.3, 9.4_

- [ ] 5. 確かめ
- [ ] 5.1 ブランチの CI で、組み立て・検査で止まること・時間を確かめる
  - 実装のブランチで `manual.yml` を回し、実物の hello-pasta で置き場所の段が通り、公開されないことを確かめる。`yq` が在ること、キャッシュが外れたときと当たったときの所要を記録する
  - 古い `pasta_check` が誤りにする書き方（`pasta.toml` の未知のキーなど）を入れた一時の変更（コミットしない）で、手元で 0.6.2 の `pasta_check` を使って台本を動かし、段 4 で失敗し、ファイルと理由が出ることを確かめる。注意だけの変更では止まらないことも確かめる。一時の変更は戻す
  - 結果と所要の時間を `research.md` に記録する
  - `release.yml` と `pasta_check` のソースに差分が無いことを `git diff` で確かめる
  - キャッシュが当たった run の増分が 1 分以内であることを記録する
  - _Depends: 3.1, 3.2_
  - _Dispatch: standard — CI の実行と失敗の場合の確かめ_
  - _Requirements: 4.2, 4.4, 4.5, 8.1, 8.2, 8.6_

- [ ] 5.2 実機の SSP で、ローカルの置き場所から main の辞書が届くことを確かめる
  - main の先頭で `assemble-site.sh` を動かした出力をローカルの HTTP サーバーに置き、v0.6.2 を入れた別ポートの SSP で更新確認をする
  - main の辞書に入れ替わり、`OnUpdateComplete`（`changed`）の台詞が出る。更新のログで、取ったファイルが変わったものだけで、`pasta.dll` を取っていないことを確かめる
  - 続けてもう 1 回更新確認をして `No Update` になることを確かめる
  - 結果とログの抜き書きを `research.md` の「実機確認」の節に記録する。通らなければ完了にしない
  - _Depends: 2.3_
  - _Dispatch: standard — 実機の SSP を手で動かして判定する確認_
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

- [ ] 5.3 main へのマージの直後に、公開サイトの実 URL で確かめる
  - マージの後に行うタスク: `/kiro-impl` の自動の割り当てと、マージ前の DoD の関門からは外す。`/kiro-complete` が squash マージの直後に行う
  - `/kiro-complete` の squash マージの直後に走る `manual.yml` の run が通り、置き場所が公開されることと、その所要の時間を確かめる
  - v0.6.2 を入れた別ポートの SSP の `homeurl` を公開サイトに戻して更新確認をし、5.2 と同じ結果（`changed` の台詞・`pasta.dll` を取らない・次は `No Update`）になることを確かめる
  - 結果は、アーカイブ後の `completed/hello-pasta-book-sync/research.md` に、後追いの PR で記録する（main は保護されているため）
  - 通らなければ完了にしない。原因を直す PR を出すか、リジェクトとしてマージを戻す PR（revert）を出し、spec を開き直す
  - _Depends: 5.1, 5.2_
  - _Dispatch: standard — 公開後の実機の確認_
  - _Requirements: 3.1, 8.6, 10.4, 10.5_

## Implementation Notes

- SSP の実機確認で SSP を複製するときは、立てる前に複製の `data/profile/app.dat` から `SelfUpdate.Pending.*` の行を消す（残すと複製が普段の SSP の保留中の自己更新を使って自分を更新し、`temp/cache` の zip を消費する。タスク 1 で発生）。
- 複製の SSP の MCP は普段の SSP が口を持つので届かない。更新の結果はローカルの HTTP サーバーの要求の記録と `pasta.log` の `OnUpdateComplete` の Reference0 で見る。
- 2.2 の後、`publish-ghost-site.ps1` は `--tree` 無しで `verify-live` を呼ぶので失敗し、回数の正規表現 `回目で一致` も新しい文に合わない。3.3 で `--tree` と正規表現（`(\d+) 回目で`）・偽の node の出力文を直す。
- main で「Check freshness」が実際に走るのはマージの後だけ。5.3 でその段の出力（publish=true）も見る。
