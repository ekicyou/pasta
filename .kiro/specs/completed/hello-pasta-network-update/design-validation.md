# 設計検証レポート: hello-pasta-network-update

- 検証日: 2026-11-15
- 対象: `design.md`（`requirements.md`・`research.md` 9 章・steering・現行リポジトリと突き合わせ）
- 実行形態: 非対話（サブエージェント）。設計・要件・research・spec.json は変更していない

## 1. Review Summary

要件 1〜11 の全項目が設計の部品に対応づいており（Traceability 表に抜けなし）、「Pages へ公開する場所は `manual.yml` の `deploy` 1 か所」「新しい権限は新ジョブだけ」「核（`validate_ghost`）には触れない」という境界が一貫している。現行の `manual.yml`（`concurrency: pages`・`needs: build`・`github-pages` 環境）、`release.yml`（`report` の `Row` 関数・`github-release` の status 契約）、`validate_cmd.rs`（`check_master` が `Report` を受けられる位置にある）、`report.rs`（`Report::new` が件数を数え直す）への記述は、読んで確かめた範囲で実態と合っている。

実装に進める水準だが、運用面で外すと後戻りしにくい穴が 3 つある（下記）。いずれも設計の小さな追記か、タスクへの織り込みで足りる。

## 2. Critical Issues

### Issue 1: nar の整合検査が「Release の公開後」にしか無く、直せない Release が公開サイト全体を止める

- **指摘**: 照合（`checkIntegrity`）は `manual.yml` の `build` ジョブにしか置かれていない。つまり nar が `updates.txt` と合わないことが分かるのは、Release が公開された後になる。`github-release.ps1` は Immutable Releases を前提にしており（`immutable` の reason がある）、公開済み Release の添付は差し替えられない可能性が高い。
- **影響**: 欠陥のある nar が `latest` になると、`manual.yml` は PR・`main` を問わず `build` で落ち続ける（要件 4.4 が「どの実行でも失敗」と定めているため）。マニュアルだけの変更も公開できなくなり、回復は新しい版を切るしかない。設計の「回復の指標」は「問題の Release の取得を直す」と書くが、その手段が無い。
- **根拠**: `.github/workflows/release.yml` の `build` ジョブには、すでに nar の必須エントリを確かめる段（約 160〜167 行）と `actions/setup-node`（Node 22）がある。同じ場所に `ghost-site.mjs build --out $RUNNER_TEMP/...` を足せば、公開前に同じ照合が働く。
- **推奨**: 公開前の関門として、`release.yml` の `build` ジョブ（または `gate`）に、作った `hello-pasta.nar` を `ghost-site.mjs build` で一度展開・照合する段を足す。現設計の「既存ジョブの中身は変えない」（Out of Boundary）とは衝突するので、その 1 行の例外を設計に明記する。あわせて `RELEASE.md` の回復欄に「`latest` の nar が壊れているときの手順」（新版を切る、または壊れた Release を取り下げる）を書く。

### Issue 2: 古いタグの再実行で `skipped`（要件 6.3）にならず `failed`・`publish` になりうる

- **指摘**: 設計は、ジョブの step が「そのタグの `descript.txt` の `homeurl` を読んで `-SiteBase` に渡し、無ければ `failed`・`publish`」としている。一方、古いタグの判定はスクリプトの手順 1 にある。`homeurl` を足す前のタグ（現行の `homeurl` は `https://github.com/ekicyou/pasta`）を再実行すると、step が先に動く。
- **影響**: 要件 6.3「最新ではないタグは `skipped`（最新ではないタグ）。判定は起動の前」を満たさない。古いタグの再実行は、置き場所の対象外のはずが、報告の表に `failed` の行を出してしまう。さらに、`homeurl` が GitHub のリポジトリ URL のままのタグでは、「無い」ではなく「別の URL」を `SiteBase` として見比べに渡す経路ができる。
- **推奨**: 順序を入れ替える。step では `homeurl` を読まず（または読めなくても失敗にせず空で渡し）、スクリプトが古いタグの判定を終えた後、見比べの直前に初めて `SiteBase` を要求する。`-DryRun` の 3 項目に「`homeurl` の無い古いタグ→`skipped`」を足す。`SiteBase` は、Rust の定数と `descript.txt` の一致テストが守る固定値なので、スクリプトに定数として持たせる案もある。

### Issue 3: 唯一の公開経路（`GITHUB_TOKEN` での起動と run の特定）が実リリースまで一度も通らず、完了の判定にも入っていない

- **指摘**: `actions: write` での `workflow_dispatch`、`return_run_details`（REST の文書に未掲載）、起動した run の特定は、設計自身が「最初の実リリースで初めて通る」と認めている（Risks・research 9.5 の 1・2）。一方、DoD（10.5）が見るのは実機確認 10.1〜10.4 で、この経路は含まれない。
- **影響**: 失敗しても他の公開先は無傷で再実行できるのは正しい。ただ、リリースの当日に初めて起動の失敗が見つかると、`release.yml` を `main` へ入れてはいけない注意（6.4）と衝突して手を打ちにくい。しかも「この仕様は完了」として `/kiro-complete` を通した後になる。
- **推奨**: (a) `publish-ghost-site.ps1` に `-Ref`（既定 `main`）と対象ワークフロー名を引数として足し、ブランチで走る `manual.yml` を実際に起動・待ち・完了判定までリハーサルできる形にする（デプロイは `main` 限定なので、組み立てまでの確認になる）。(b) タスクに「手元の `gh` で `return_run_details` の応答を確かめる」「リハーサルの結果を `research.md` に記録する」を入れる。(c) DoD に「実リリースでの初回の起動結果は完了後に確認する」旨を明記し、置き場所の行が `failed`・`dispatch` になったときの扱い（再実行・手動実行）を RELEASE.md に書いておく。

## 3. Design Strengths

1. **公開の 1 か所化と最小権限が崩れていない。** 新しい書き込み権限は `publish-ghost-site` の `actions: write` だけで、Pages への書き込みは既存の `deploy`（`github-pages` が `main` だけを許す）のまま。`release.yml` のタグの ref から直接公開できない制約を逆手に取り、`ref=main` の dispatch を唯一の経路にしている。PAT や GitHub App のような長期の認証情報を足さず、`report` は `needs` の outputs だけを読む既存の流儀を保つので、置き場所の失敗が他の公開先の status に漏れない（要件 5.5）。
2. **照合のロジックを 1 本に束ねている。** `parseUpdates` を共有して、公開前の展開の照合・公開後の URL の見比べ・実機確認の全ファイル確認（`--full`）を同じツールで受け持つ。nar を一覧から読むため `hello-pasta` 固有の名前を持たず、`pasta-dll-multiarch` や `bal/` 同梱が入っても変わらない（要件 11.4）。npm 依存を足さず、自己テストが `manual.yml` の `find` で自動実行される点も既存の流れに乗っている。

## 4. 補足（重大ではないが、タスク化のときに拾う）

- v0.6.1 の nar が `updates.txt` の全行と整合するかは未確認（research 9.5 の 5）。PR の `manual.yml` の最初の実行で分かるので、実装の最初の PR 実行を確認の機会にする。
- `cancelled` を見比べに進める扱いは妥当だが、手で取り消した場合も最大 15 分待ってから `failed`・`verify` になる。許容範囲（設計に明記済み）。
- `FindingKind` の追加で触る箇所は `pasta_check` の中だけ（`FindingKind` を参照するのは `crates/pasta_check/src/validate/*` のみ。他のクレートに列挙は無い）。`#[serde(rename_all = "kebab-case")]` で `hello-pasta-homeurl` になる点は設計の文言と一致する。
- `.claude/skills` の編集が分類器に止められる件は、設計の織り込み済み（タスクに「入った分と残りを示して許可を求める」を入れる）。

## 5. Final Assessment

**Decision: GO（条件つき）**

**Rationale**: 要件との対応・境界・既存の契約（`status`/`reason`・`Row`・`Report::new`・`concurrency: pages`）との整合は取れており、設計の骨格は変えなくてよい。上の 3 点は設計への追記か、タスクへの織り込みで解け、実装の方向を覆さない。Issue 1 と Issue 2 は設計のディスカッションで決め、`design.md` に追記してからタスク生成に進むのを推奨する。Issue 3 は、タスクに「リハーサル」と「`research.md` への記録」を入れる形で足りる。

**Next steps**:
1. 設計ディスカッションで Issue 1（公開前の関門を `release.yml` の `build` に足すか）、Issue 2（`SiteBase` の読み出し順）、Issue 3（リハーサル用の `-Ref` 引数と DoD の扱い）を決める。
2. 決定を `design.md` に反映する。
3. `/kiro-spec-tasks hello-pasta-network-update` でタスクを生成する。
