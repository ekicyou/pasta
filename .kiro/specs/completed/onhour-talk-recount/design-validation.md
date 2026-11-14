# 設計検証レポート: onhour-talk-recount

- 対象: `.kiro/specs/onhour-talk-recount/design.md`（requirements.md・research.md・steering と突き合わせ）
- 実施: 2026-11-14（kiro-validate-design・非対話）
- 照合したコード: `crates/pasta_lua/pasta_scripts/pasta/shiori/clock.lua`・`event/virtual_dispatcher.lua`・`event/init.lua`・`event/boot.lua`・`event/second_change.lua`・`pasta/scene.lua`、テスト `virtual_dispatcher_spec.lua`（L978〜991）・`virtual_clock_test.lua`（L383〜445）・`event_clock_hook_test.lua`・`reg_handler_chain_test.lua`、マニュアル `book/src/lua/shiori-events.md`（L117〜119・L1436〜1447）・`book/src/internals/shiori.md`（L505・L530）

## 設計レビューの要約

不具合の原因（`check_talk` が `hour_margin` で見送っても `next_talk_time` を変えず、`check_hour` も時報で決め直さない）をコードで確かめた。直し方（`check_hour` が thread を返す直前に 1 行で決め直す）と、スリープ明けの 1 回を `CLOCK.suspend_count()` と `talk_suspend_count` の比較で決める形は、既存の作法（`epoch()`／`seen_epoch` の比較・`REG.X = REG.X or function`・鎖の底）にそのまま乗っており、実装の道筋は明確である。残る懸念は、SSP の実機の振る舞いに頼る前提 2 つ（Reference0 と OnSysSuspend の順番）と、`OnSysSuspend` が来ないときの失敗の向きで、どちらもディスカッションで決めれば足りる。

### 実機の順番での検証（コードを手で追った結果）

| 場合 | 流れ | 結果 |
|------|------|------|
| 正時をまたいだスリープ | `OnSysSuspend`（復帰後・回数 n）→ 最初の秒で `check_hour` が時報を返し、決め直して `talk_suspend_count = n` → 数秒後の `OnSysResume` で `wake_talk` は等しいので何もしない | 時報 1 回だけ。直後のランダムトークは出ない（要件 1.2・1.6・3.4） |
| 長いスリープ・正時をまたがない（溜まったトーク） | `OnSysSuspend`（n）→ 最初の秒で `check_talk` が溜まったトークを返し `talk_suspend_count = n` → `OnSysResume` は何もしない | 1 回だけ（要件 3.4）。最初の秒が Status や `hour_margin` で閉じていても、`wake_talk` は既に過ぎた時刻を「今」にするだけで、出るのは 1 回 |
| 長いスリープ・正時をまたぐが時報のシーンが無い | 最初の秒の `check_hour` は `nil`（`next_hour_unix` だけ進む）→ 同じ `dispatch` の `check_talk` が溜まったトークを出して記録 | 1 回だけ（要件 1.4 と両立） |
| 短いスリープ（何も期限が来ていない） | `OnSysSuspend`（n）→ 最初の秒は何も出ない → `OnSysResume` で `talk_suspend_count < n` なので `next_talk_time = 今` → 次の門の開いた秒で出る | スリープ明けのおしゃべりが 1 回（要件 3.1・3.3） |
| `wake_talk` の後、門が閉じている間に正時 | 門が開いた秒に `check_hour` が先に時報を出し、決め直しが「今」を上書き | 時報が 1 回の代わりになる（要件 3.4 の最後） |
| 停止中のスリープ | `dispatch` の門で止まり、`＞自動トーク再開` の `restart_timers` が `next_talk_time = 0` に戻す | 持ち越さない（要件 3.6） |

`watch_jump` の既存のスリープの扱い（`_suspended` が復帰の最初の秒より前に立つこと）も同じ順番に頼っており、設計の新しい依存は既存の前提を超えない。

## 重大な課題（3 件以内）

🔴 **Critical Issue 1**: Reference0 と `OnSysSuspend` の順番が実機で未確認のまま要件 3.7・3.4 を載せている
**Concern**: 2026-10-18 の実機記録は「`OnSysSuspend` が復帰の最初の秒より前・`OnSysResume` が数秒遅れ」の 1 例（142 秒のスリープ）だけで、利用者が起こした復帰で SSP が送る Reference0（`normal` か `auto` か、両方を順に送るか）は記録が無い。Windows は復帰のたびに自動の復帰（PBT_APMRESUMEAUTOMATIC）を送り、利用者の操作があったときだけ別の通知を足すので、SSP がこれを `auto` に写していれば、ふつうの復帰でもスリープ明けのおしゃべりが一度も出ない。90 秒以下の短いスリープでの順番も確かめていない。
**Impact**: 要件 3.1 の新しい既定の動きが、テストは通るのに実機では黙って効かない（または順番が逆の環境で溜まったトークが 2 回出る）おそれがある。
**Suggestion**: ghost_dev の「ぱすた開発」の実機確認の項目に「利用者の復帰と自動の復帰それぞれで届く `OnSysResume` の回数と Reference0」「短いスリープでの `OnSysSuspend` と最初の秒の順番」を明記し、Testing Strategy に `auto` → `normal` の順で 2 回届く場合（2 回目で出る・3 回目以降は出ない）の結合テストを足す。実機で `auto` しか来ないと分かったら、要件 3.7 をディスカッションで見直す。
**Traceability**: 要件 3.1・3.4・3.7・6.5
**Evidence**: design.md「Event Contract」（Reference0 の列挙）・「前提」1・「既存のテスト（6.7）」の実機の行・`Revalidation Triggers` の 4 項目め

🔴 **Critical Issue 2**: `OnSysSuspend` を一度も見ていないときの失敗の向きが、この spec の直す不具合と同じ形になる
**Concern**: 前提 2 は `suspend_count() == 0` なら `wake_talk` が「今」にする。`OnSysResume` だけを送るベースウェア（areka など、SSP 以外で `OnSysSuspend` を送るかは未確認）で正時をまたいだスリープから戻ると、最初の秒に時報 → 数秒後の `OnSysResume` で「今」→ 時報の直後にランダムトークが出る。これは要件 1.2（時報から `talk_interval_min` 秒は出さない）に反し、報告された不具合そのものの再現になる。溜まったトークの後にもう 1 回出る場合も同じ。
**Impact**: 直したはずの「時報の直後のランダムトーク」が、`OnSysSuspend` の無い環境で既定の動きとして戻る。
**Suggestion**: 回数 0 のときは「喋らない」側に倒す（`suspend_count() == 0` なら何もしない）か、`OnSysResume` の直前の短い窓に自動トークが出ていれば何もしない、のどちらかをディスカッションで決める。前者なら 1 行で、取りこぼし（おしゃべりが出ないだけ）は今のふつうの間隔に戻るだけで害が小さい。`wake_talk` の単体テストの「`OnSysSuspend` を一度も見ていないときは今にする」の期待もそれに合わせる。
**Traceability**: 要件 1.2・3.1・3.4
**Evidence**: design.md「前提」2・VirtualDispatcher の Postconditions（`CLOCK.suspend_count() > 0` の条件）・Testing Strategy「`wake_talk` の単体」

## 設計の強み

- **既存の仕組みに乗せて足すコードを最小にしている**: 「受けているか」の判定（シーン・`GLOBAL.OnSysResume`・代入・鎖が真の値で止める・条件漏れ）を、鎖の底に既定ハンドラを置くだけで `EVENT.fire` と `SCENE.co_exec` の今の決まりに任せており、要件 4.1〜4.5 に新しい分岐が要らない。`wake_talk` は時刻を動かすだけで門を飛ばさないので、Status・`hour_margin`・停止・チェイントークの続きの決まり（要件 2・3.3・3.5・3.6）がすべて既存のコードのまま効く。
- **追跡と検証の粒度が良い**: 全要件がコンポーネントとテストに対応付けられ、既存テストの期待の変更が `virtual_dispatcher_spec.lua` L987 の 1 か所だけであることまで特定している。数え直しを先のコミット・スリープ明けを後のコミットに分ける段取りも、可逆な小さい段で進める方針に合う。

## 最終判断

**Decision: GO**

**Rationale**: 不具合の根の直し方は正しく、スリープ明けの 3 つの場合（正時をまたぐ・溜まったトーク・短いスリープ）は SSP の実機の順番で二重に喋らないことをコードで確かめた。残る 2 件は構造の作り直しではなく、前提の向き（回数 0 の扱い）と実機確認の項目の追加で閉じられる。

**Next Steps**:
1. 設計ディスカッション（`/kiro-design-discussion onhour-talk-recount`）で課題 1（実機確認の項目と `auto` → `normal` の結合テスト）と課題 2（回数 0 のときの向き）を決め、design.md に反映する。
2. その後 `/kiro-spec-tasks onhour-talk-recount` でタスクを作る。

### 補足（重大ではない気づき）

- `next_talk_time == 0`（読み込み直後の最初の秒の前など）で `wake_talk` が呼ばれると、`check_talk` の「初回は時刻を決めるだけ」の枝を飛ばしてすぐ出る。起こりにくいが、`wake_talk` の Postconditions に「0（未設定）のときは何もしない」を足すと素直になる。
- 依存の閉包を読み直す 10 スイートは、`reload` で REG を空にした後に `pasta.shiori.event` を読み直す形のものがあり、`sys_resume` を並びに足さないと `REG.OnSysResume` が入らないだけで既存の期待は壊れない（設計の記述どおり）。新しいスイートだけは必ず読み直す。
