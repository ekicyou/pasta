# 設計の検証: hello-pasta-book-sync

- 日付: 2026-11-19
- 対象: `design.md`（`requirements.md`・`research.md`・steering と照らした）
- 進め方: `kiro-validate-design` の手順（分析 → 重大な問題 → 良い点 → GO/NO-GO）。対話なしで判定した。

## まとめ

設計は、既存の道具をつなぐだけの小さな形である。重ねる・検査する・`updates.txt` を作るを `pasta_check release` 1 回に任せ、照合と展開は今の `ghost-site.mjs build` に任せる。要件との対応はほぼ全部が埋まっている。

ただし、古い run の見分け（要件 5.4）の理屈に 1 つ穴がある。直しは `manual.yml` の 1 行で済む。ほかの 2 つは、要件と設計の食い違いの扱いと、確かめる順番の問題である。

## コードで確かめたこと

- `pasta_check release` の引数は `--target`・`--release`・`--nar`・`--copy`（0 回以上）である（`crates/pasta_check/src/main.rs`）。段の並びは「写す → `--copy` を重ねる → `ghost/master` を検査 → `updates.txt` → nar」である（`release.rs`）。設計の頼り方と合う。
- 引数の関門（`release_args.rs`）は、`--release` が `--target`・`--copy` と同じ・含む・中にある、を断る。設計の `<tmp>/repo`・`<tmp>/nar`・`<tmp>/merged`・`<tmp>/merged.nar` は兄弟なので通る。
- 除外の名前は `update_files.rs` の `EXCLUDED_FILES`（`updates2.dau`・`updates.txt`・`developer_options.txt`）で、設計の 3 つと同じ。
- `pasta_check` の既定の機能は `cli` で、バイナリは `pasta_check`。依存に Lua は無い。`cargo install` で入る。
- `manual.yml` の concurrency は `pull_request` だけを別の組にし、それ以外（`push`・`workflow_dispatch`）は、ブランチの手動実行も含めて全部 `pages` の組である。
- `publish-ghost-site.ps1` は、run の結論が `cancelled` でも見比べへ進む。「後の run が同じ最新 Release で作る」という前提である。
- `publish-ghost-site` のジョブはタグで checkout するので、`<タグ>:<パス>` の `git ls-tree` は手元で読める。

## 重大な問題

### 問題 1: ブランチの手動実行が、main の待ちの run を取り消す（要件 5.4）

**懸念**: 設計は「起動の条件のファイルを変えたコミットの run は、この run の後に走るか、すでに終わっている」を前提に、古い run の公開を飛ばす。しかし今の concurrency は、ブランチでの `workflow_dispatch` も `pages` の組に入れる。GitHub は組ごとに待ちの run を 1 つだけ残し、新しく来た run が古い待ちを取り消す。

**影響**: 次の順で、main の最新の中身が公開されないまま残る。

1. main の run A（コミット a）が走っている。
2. 辞書を変えたコミット b が入り、run B が待つ。
3. 誰かがブランチで `manual.yml` を手で起動する（リリースの予行の `-Ref` など）。B が取り消される。
4. A は「b で起動の条件のファイルが変わった」として公開を飛ばす。ブランチの run は公開しない。

置き場所もマニュアルも、次に起動の条件に当たるコミットまで a より前のままになる。要件 5.4 に反する。リリース CI の起動した run が同じ形で取り消されると、`publish-ghost-site.ps1` の「後の run が作る」という前提も崩れる。

**提案**: `pages` の組を main だけにする。例: `group: ${{ github.ref == 'refs/heads/main' && 'pages' || format('manual-{0}-{1}', github.event_name, github.ref) }}`。設計の「System Flows」の前提の文と、ManualWorkflow の責務に、この 1 行を足す。持ち場は `manual.yml` で、この仕様の中である。

**根拠**: `manual.yml` の `concurrency`。GitHub Actions の concurrency の決まり（待ちは組に 1 つ・新しいものが古い待ちを取り消す）。research 11.5。

### 問題 2: リリースの直後の間は、要件 5.1 の「shall」に反する

**懸念**: 設計は、Release が最新になってから、リリース CI の起動した run が公開を終えるまでの間を「残す危険」とした（Open Questions の議題 2）。この間、置き場所の nar の側は 1 つ前の Release のままである。新しい nar を入れた利用者がこの間に更新確認をすると、md5 が違うので古い `pasta.dll` を受け取る。要件 5.1 は「その nar より古いファイルを返さない」と書き、例外を持たない。

**影響**: 設計のまま実装すると、要件 5.1 を満たさない。完了の判定で食い違いになる。キャッシュが外れた run（新しい版の最初の run は必ず外れる）では、`cargo install` の 1〜3 分だけ間が延びる。

**提案**: 設計ディスカッションで、どちらかに決める。

- 推奨: 要件 5.1 に「リリース CI の公開が終わるまでの間を除く」を足し、手順書に書く。今の `hello-pasta-network-update` にも同じ間がある。塞ぐには `release.yml` の流れを変える必要があり、この仕様の外（`pasta-dll-multiarch` と重なる）である。
- 別の案: 間を塞ぐ設計にする。`release.yml` を触るので、ウェーブの組み方から見直す。

あわせて、間を短くするため、`pasta_check` のキャッシュは組み立ての成否に関わらず残す（`actions/cache/restore` と、入れた直後の `actions/cache/save` に分ける）。`actions/cache` の既定は、ジョブが成功したときだけ残す。要件 4.2 で止まる run が続くと、毎回 `cargo install` をやり直す。

**根拠**: 要件 5.1。design.md「Open Questions / Risks」。`actions/cache` の保存の条件（ジョブの成功のときだけ）。

### 問題 3: `date=` の仮定を確かめるのが最後になっている（要件 10.2）

**懸念**: 作り直した `updates.txt` の日付の欄は、nar の `updates.txt` と必ず違う。設計は「SSP は md5 だけで比べる」と仮定し、実機確認（要件 10.2）で確かめる。実機確認は実装の最後である。

**影響**: もし SSP が日付の欄も見るなら、利用者は辞書の更新のたびに全部のファイル（`pasta.dll` を含む）を取り直す。要件 1.6・1.8・10.2 が全部崩れる。最後に分かると、要件 10.5 で仕様ごとリジェクトになる。直すには、日付の欄を nar の行に合わせる処理が要り、「作り手は `pasta_check release` 1 つ」という設計の柱が変わる。

**提案**: タスクの最初に、短い実機の確かめを置く。公開中の v0.6.2 の置き場所を手元に写し、`updates.txt` の日付の欄だけを書き換えて、ローカルの HTTP サーバーで v0.6.2 を入れた別ポートの SSP に更新確認をさせる。`No Update` になれば仮定は正しい。ならなければ、実装の前に設計へ戻る。

**根拠**: research 11.4。design.md「Open Questions / Risks」の `date=` の項。要件 10.5。

## 良い点

- `updates.txt` の作り手を `pasta_check release` 1 つにした。検査（要件 4）と作り直し（要件 2）が 1 回で済み、要件 2.3 の一致のテストが要らない。v0.6.2 の取り出しで 37 行が日付の欄を除いて一致することも、手元で測ってある。
- 「nar の側」の規則を `ghost-site.mjs` の `narOnlyFiles` 1 か所に置き、置き場所の組み立てとリリース CI の見比べの両方で使う。ファイル名を決め打ちせず、`release.yml` も触らない。`pasta-dll-multiarch` と重ならない。

## 小さな気づき（議題にしなくてよい）

- 要件 8.6 は「キャッシュが効いている間」だけの約束なので、キャッシュが外れたときの `cargo install` の 1〜3 分は違反ではない。キャッシュは 7 日使われないと消える。
- FreshnessGate は、走っている run 自身の `manual.yml` から `paths` を読む。新しいコミットが `manual.yml` を変えても、`manual.yml` が `paths` に在るので差に出る。理屈は合う。
- `git -c core.autocrlf=true archive` の CRLF は Windows の git で測ったものである。ubuntu での一致は、設計どおり実装の PR の run で確かめる。

## 判定

**GO**（条件つき）

- 理由: 形は小さく、既存の道具の使い方はコードと合う。要件との対応も埋まっている。3 つの問題は、どれも設計の形を変えずに直せる。
- 条件: 問題 1 の concurrency の 1 行を設計に入れる。問題 2 は、要件 5.1 の例外として記録するかを設計ディスカッションで決める。問題 3 の確かめを、タスクの最初に置く。
- 次: `/kiro-design-discussion hello-pasta-book-sync` で上の 3 つを議題にし、決まったら `/kiro-spec-tasks hello-pasta-book-sync`。
