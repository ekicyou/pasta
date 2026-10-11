# CI での初回のリリースの記録

- 版: v0.3.8
- リリース CI の実行: <https://github.com/ekicyou/pasta/actions/runs/38025598142>
- 完了した attempt: 1（7 つの公開先がすべて `published`）
- 全 job の再実行: attempt 2 で、7 つの公開先がすべて `skipped`
- クレートごとの公開の所要時間（秒）: pasta_core 18 / pasta_dsl 9 / pasta_lua 84 / pasta_shiori 29 / pasta_check 9
- 認証の期限（1800 秒）に対する余裕の最小: 1716 秒
- 後片付け（開発者が実施。完了の連絡: 2026-10-10）
  - [x] 5 クレートの「Trusted Publishing のみ」の有効化（crates.io の API で、5 つとも `trustpub_only` が `true` であることを確かめた）
  - [x] `CARGO_REGISTRY_TOKEN` の失効（開発者の連絡による。開発機のユーザー環境変数が消えたことを確かめた）
  - [x] `VSCE_PAT` の失効（`vsce verify-pat` が 401 を返すことと、開発機のユーザー環境変数が消えたことを確かめた）

## 初回で確かめたこと

`design.md`「CI での初回のリリース（Opus）で確かめること」の 6 つの結果。

| # | 確かめること | 結果 |
|---|--------------|------|
| 1 | タグで起動した実行の `headBranch` がタグ名、`headSha` がタグのコミットになる | 合格 |
| 2 | 公開の step のログに `status=...` の行があり、`line` の行として取れる。`gate` の中の job の名前が `gate / ...` の形で出る | 合格 |
| 3 | 失敗した job の再実行の後の読み方 | 未確認（失敗が起きなかった） |
| 4 | 全 job の再実行で、最新の attempt の `line` が 7 行・すべて `skipped` になる | 合格 |
| 5 | `gh pr merge --squash --subject` でできたコミットの件名が `chore(release): vX.Y.Z` を含む | 合格（件名は `chore(release): v0.3.8` だけで、PR の番号は付かない） |
| 6 | 実行の約束のとおりに進む | 合格（サブエージェント・レビュー・`/kiro-validate-impl` は走らず、コミットは版の更新と本記録だけ）。ただし下の食い違い 1 がある |

## 手順との食い違い

1 と 2 は、本記録の後の PR で `design.md` と手順書（`.github/release-ci-setup.md`）に反映した（2026-10-10）。3 と 4 は事実の記録で、反映するものは無い。

1. **段 5 手順 3（統合）**: エージェントが 2 行を `;` と `echo` でつないで 1 回で実行したところ、許可の規則（`Bash(gh pr merge:*)`）に合わず、許可の判定に拒否された。`gh pr merge` を 1 行だけで実行し直すと統合された。その直後の `gh pr view {PR} --json state,mergeCommit` が同じ理由で拒否され、`MERGED <SHA>` の出力は取れなかった。統合は `git fetch origin main` と `git log origin/main` で確かめ、開発者の指示で段 5 手順 5 から続けた。
   - 直す案: コマンドは 1 回の呼び出しに 1 行ずつ実行すると明記する。統合の成否を `git log origin/main -1 --format=%H -F --grep="chore(release): v{V}"` でも判定できるようにする。
2. **手順書 10 節（`VSCE_PAT` の失効）**: 「User settings」は、組織の中に入らないと出ない。組織名は `https://aex.dev.azure.com/me` で調べ、`https://dev.azure.com/{organization}/_usersSettings/tokens` を開く。全組織向けの PAT は、「Access scope」の絞り込みを「All accessible organizations」にしないと一覧に出ない。
3. **開発機の回線**: Azure DevOps のホスト（`dev.azure.com` など）は、開発機の回線では IPv6 だと接続がリセットされる（IPv4 は通る）。画面の操作は別の回線から行った。リリース CI には関係しない。
4. **`VSCE_PAT` の状態**: 開発機の値は `vsce verify-pat` で 401 になった。その後に、開発者から「削除した」との連絡を受けた。使えない状態であることは確かめたが、401 になった時点で失効の操作が済んでいたのか、それより前から無効だったのかは確かめていない。
