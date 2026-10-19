---
name: kiro-complete
description: 'Kiro仕様駆動開発のSpec完了ワークフローを実行する。未解決問題の棚卸（原則その場で解決・spec の外に手が届いても直す・他specと重なるものはbriefへ申し送り・裁定や spec 1 本規模の例外だけ/kiro-discoveryで起票）→DoDゲート検証→コミット→completedフォルダ移動→spec.json更新→参照パス修正→ロードマップ更新→スキルドキュメント同期→最終コミット→PR作成→squashマージまでを中断なく完遂する。Use when: 実装完了を承認する, 承認してください, 完了を承認, spec承認, approve implementation, kiro承認完了。DO NOT USE when: 実装が完了したのみ（承認の明示がない場合）、タスクが終わっただけ'
argument-hint: <feature-name>
---

# Kiro Spec 完了ワークフロー

## 発動条件（必須）

> **⚠️ このスキルは開発者の明示的「承認」がある場合にのみ発動する。**

### ✅ 発動する（承認の明示がある）
- 「実装完了を**承認**します」「**承認**してください」
- 「このspecを**承認**する」「**approve**」「kiro **承認**完了」

### ❌ 発動しない（承認の明示がない）
- 「実装が完了した」「タスクが全部終わった」
- 「spec完了」「アーカイブしてほしい」などの曖昧な表現のみ
- AIが自律的に「完了したと判断」した場合

### 承認が不明瞭なとき
発動を迷う場合は **発動しない**。必要なら開発者に確認する:
> 「実装完了の承認をいただけますか？承認いただいた場合、完了ワークフローを実行します。」

---

## いつ使うか
- 開発者が上記「発動する」に該当する承認を**明示的に**宣言したとき
- tasks.md の全タスクが `[x]` 完了している状態で使用
- 設計文書リフレッシュが完了した後

## 権威的ソース

> **このスキルは `.kiro/steering/workflow.md` に従う。**
> 完了基準（DoD）、コミット規約、ドキュメント更新チェックリスト、スキルドキュメント更新判定はすべて workflow.md を参照すること。
> このスキルは workflow.md の「実装完了時アクション」を**自動オーケストレーション**するものであり、ルールの複製は行わない。

## 哲学
- **中断せず一連で完遂する** — 全ステップを止めずに実行
- **VSCodeの変更ファイル確定挙動を回避** — spec.json編集は移動後に行う
- **ステアリングが正** — DoD・コミット規約・禁止事項はすべて workflow.md に従う
- **繰り返し仕様は移動しない** — `review-improvement-loop` 等は常に `.kiro/specs/` 直下に留まる
- **未解決問題を取りこぼさない** — アーカイブした後では拾い漏れに気付けない。DoD ゲートより前に棚卸しし、原則その場で直し、例外に当たるものだけ申し送り・起票する（冒頭ステップ）

## 前提条件
- `.kiro/specs/{feature}/tasks.md` の全タスクが完了
- 設計文書と最終実装の整合確認済み

## 例外: 繰り返し仕様

`review-improvement-loop` のような繰り返し実行型仕様は `completed/` に**移動しない**。

判定基準:
- spec.json や requirements.md に「繰り返し」「repeatable」「定期実行」等の記述がある
- `/kiro-spec-impl` のたびにタスクがリセットされる設計

繰り返し仕様の場合:
1. 冒頭ステップ（棚卸）・ステップ1（DoD検証）・ステップ2（コミット）のみ実行
2. ステップ3〜5（移動・パス更新・ロードマップ）をスキップ（`completed/` へは移動しない）
3. ステップ8（リモート同期＝PR ベース）を実行
4. tasks.md のチェックボックスをリセット（全 `[x]` → `[ ]`）

---

## 手順

### 前置きステップ: マージの机（areka-impl-watch）

マージをリポジトリごとに 1 本ずつ通すため、冒頭ステップより前に、並走するセッションが共有するコンソールアプリ `areka-impl-watch.exe` のマージの机の番を取る。完了の手順は roadmap.md・台帳の数・生成した報告を触るので、手順の全部が机の番の中に入る。机の状態の正本はアプリの状態で、ここに書くのはその呼び方の約束である。アプリと手順書は areka のリポジトリにあり（手順書は `C:\home\maz\git\areka\doc\impl-watch.md`）、pasta は置かれた実行ファイルを呼ぶだけである。

- **机のコマンドを呼ぶのは主の会話だけ**。サブエージェント（テスト・文書の並行レーンなど）には `areka-impl-watch.exe` を走らせない（1 つのセッションが 1 つの参加者）。
- **1 つの作業木で机を使うセッションは 1 つ**。同じ作業木の 2 つのセッションは識別が同じになり、互いの見張りを外し合う。

#### 机の呼び方の約束

- 受け継いだ環境変数をそのまま使う。値を設定・変更・書き替えしない。`/c/…` の形へ直さない・`cd` しない・コマンドの前へ `AREKA_IMPL_WATCH_HOME=…` を置かない。
- 机の段の中で実行ファイルをビルド・コピー・置き換えしない。置き場所の下のファイルを手で書き替え・消さない。ほかのセッションの見張りや待ちのプロセスを止めない（推測で `kill` しない）。`clear`（全部）を呼ばない。
- 見回り・時間を決めた繰り返し・定期の `status` をしない。ふだんの流れでは `status` も `status.md` も読まない。セッションを起こすのは待ちのコマンドが終わったときだけ。
- 机について開発者へ伝えるのは、1 つの出来事につき 1 行。

**机の前置き（1 回の Bash・前景）**: 前置きステップの初めに 1 回呼ぶ。`/kiro-impl` の机の段と同じ字で書く。

```bash
ID=$(basename "$(git rev-parse --show-toplevel)"); R=$(basename "$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")")
if [ -z "$AREKA_IMPL_WATCH_HOME" ]; then echo "desk: skip no-home"
elif [ ! -f "$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" ]; then echo "desk: skip no-exe"
elif ! [[ "$ID" =~ ^[A-Za-z0-9._-]{1,100}$ && "$R" =~ ^[A-Za-z0-9._-]{1,100}$ ]]; then echo "desk: skip bad-name $ID $R"
else echo "desk: id=$ID repo=$R"; fi
```

- 識別 `<ID>` は作業木の最上位のフォルダの名前（1.1）。作業木の名前から求めるので、Claude のセッションが替わっても同じ値になる（1.2）。
- リポジトリ `<R>` は共通の git フォルダ（`.git`）の親の名前で、どの作業木から呼んでも同じ（1.3）。`origin` の URL は資格情報つきなので使わない。
- 名前 `--name` には spec の名前を渡す（1.4）。`merge` の `--spec` にも同じ名前を渡す。
- 形に合わなければ別の値をこしらえず `skip`（1.5）。
- 出力の `id=`・`repo=` の値を覚え、以後のコマンドへ字のまま書く（Bash の変数はコマンドをまたいで残らないため）。会話の圧縮で忘れたら、前置きをもう 1 度呼ぶ（同じ値が出る）。

**呼ぶ形**（どのコマンドも同じ）:

```bash
"$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" <コマンド> <引数> 2>&1; echo "exit=$?"
```

- `2>&1` で標準エラーの 1 行（`gone:`・`not applied:`・`already running`・断りの文）も同じ出力に入れる。読むのは最後の `exit=<n>` とその前の 1〜2 行だけ。
- **バックグラウンド**（Bash の `run_in_background`）: `watch`・`merge --wait`・`stopped --wait`・`resume`。走らせたら、その種類と実行の番号を会話に覚える（「自分の待ち」）。
- **前景**: `leave`・`merged`・`cancel`、それに 5.7 と訳を調べるときの `status`。
- pasta の完了の手順はマージの机だけを取る。排他テストの机・ひとり占めの机（`request`／`return`）は取らない（2026-10-19 開発者の裁定。`cargo test --all` は机なしで回す）。

#### 終わり方の表（マージの机だけを取る形）

| 終わったもの | 出力 | すること | 要件 |
|---|---|---|---|
| 見張り | `exit=0`・`stop requested by <X>` | 停止要請。各スキルの「止まる切れ目」で `stopped --wait` | 3.1, 3.3 |
| 見張り | `exit=0`・`already stopped; …` | 「止まった」を記録し直さず `resume` をバックグラウンドで | 3.7 |
| 見張り | `exit=1`・`AREKA_IMPL_WATCH_HOME …` | その回の机の段を止めて 1 行（標準エラーの行をそのまま）・最後の報告にも残す | 6.3 |
| 見張り | `exit=1`・`a watch wait for <ID> is already running` | 自分の見張りが走っていれば何もしない。覚えが無ければ、下の「残りの見張りの外し方」の 1 本をバックグラウンドで走らせ、1 行で伝える。抜けたので持っていた申し込みがあれば、その 1 本が終わる前に申し込み直す | 2.3, 2.7, 2.8 |
| 見張り | `exit=1`・上のほか | 直ちに 1 回だけ立て直す。立て直しも同じ文の 1 なら終了コード 1 の表の行に従い、直らなければその回の机の段を止めて最後の報告に残す | 2.6, 6.4 |
| 見張り | `exit=3` | 今の見張り（最後に走らせた見張りの実行）でなければ何もしない。今の見張りで、参加を終えた（実行の終わりの `leave`・`merged`）後なら何もしない。それ以外は 1 行伝えて立て直し、持っていた申し込みを申し込み直す（机を持っていたら各スキルの 4.7・5.8） | 2.9, 2.10 |
| 再開の待ち（`stopped --wait`・`resume`） | `exit=0`・`resumed` | 見張りを立て直してから、止まった所の次へ | 3.5 |
| 再開の待ち | `exit=0`・`stop requested again by <X>` | 作業に戻らず、もう一度 `stopped --wait` | 3.6 |
| 再開の待ち | `exit=3` | 見張りを立て直して続ける | 3.8 |
| 再開の待ち | `exit=1`・`a resume wait for <ID> is already running` | 前の再開の待ちが走っている。何もしない（6.4 の失敗に数えない） | 3.10 |
| `merge --wait` | `exit=1`・`a merge wait for <ID> is already running` | 自分の待ちが走っていれば何もしない。覚えが無ければ `cancel --id <ID>` → 申し込み直す → 1 行（`cancel` はマージの机も外すので、マージの机をまだ持っていない番待ちのときだけ） | 6.4 |
| `merge --wait` | `exit=3`・`not applied: return the test and sole desks before merge`／`not applied: a desk request is waiting; merge after it is granted and returned` | この識別に机の持ち主か机の申し込みが残っている（このスキルは `request` を打たないので、同じ作業木のほかのセッションか残りの記録）。状態を変えずに断られた。`cancel` しない。1 行伝えて開発者の指示を待つ | 6.4 |
| `leave`・`merged`・`cancel` | `exit=1`・下の 2 行のほか | 標準エラーの 1 行をそのまま伝え、手順書 5 節の終了コード 1 の表に従う。直らなければその回の机の段を止めて最後の報告に残す | 6.3, 6.4 |
| どれでも | `exit=1`・`state.lock is busy …` | 間を置かずに 1 回だけ呼び直す（実行ファイルが自分で 10 秒待ってから断るので、スキルの側で待たない）。続けば 6.4 | 6.4 |
| どれでも | `exit=2` | 呼び方の誤り。スキルの文の誤りなので、その回の机の段を止めて最後の報告に残す | 6.4 |

- 立て直し・申し込み直しは、同じ出来事で 2 つの待ちが続けて終わっても、それぞれ 1 回だけ行う。2 度目が `already running` で終わっても害の無いものとして扱う（4.4 と同じ扱い）。
- 終わりの知らせは中身（出力の文）で読む。「自分の見張りか」の覚えに頼るのは `already running` の 2.7 と 2.8 の分かれ目だけ。
- **残りの見張りの外し方**（2.8）: 前のセッションの残りの見張りは、`leave` の後に記録の消えたことを次の見回り（実行ファイルの見回りは 1 秒ごと・`wait.rs` の `POLL`）で見つけて終わるまで、自分の識別の見張りの錠を持ち続ける。直ちに立て直すと、その錠にぶつかって `already running` になる。そこで、外す・待つ・立て直すを 1 本のバックグラウンドの Bash にまとめ、セッションを起こすのは終わりの 1 回だけにする:

```bash
"$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" leave --id <ID> 2>&1; sleep 3; "$AREKA_IMPL_WATCH_HOME/areka-impl-watch.exe" watch --id <ID> --repo <R> --name <feature> 2>&1; echo "exit=$?"
```

  - `sleep 3` は見回りの間隔 1 秒の 3 回分。読むのは最後の `exit=` と見張りの行だけ（`leave` の行は読まない）。この 1 本を「今の見張り」として覚える。
  - この `leave` は参加を終えたことに数えない（参加を終えるのは実行の終わりの `leave` と `merged` だけ）。
  - この 1 本がまた `already running` で終わったら、2.8 をもう 1 度は行わず 6.4 に従う。


#### 会話が覚えるもの
- 会話が覚えるもの: 机の段が `on`（`id`・`repo`）か `off`（訳）／見張りの実行の番号／停止の状態（無し・停止要請を受けた・止まった）／マージの机（無し・待っている・持っている・自分で外した）／参加を終えたか（実行の終わりの `leave`・`merged` の後。2.8 の外し方の `leave` は数えない）。
- 「今の見張り」は最後に走らせた見張り（2.8 の 1 本を含む）の実行の番号。終わった見張りの 3 は、それが今の見張りのときだけ扱う。立て直す前の古い見張りの終わりは読み流す。
- 正本はアプリの状態である。会話の覚えが消えても、前置きと終わり方の表だけで元へ戻れる（失うのは 2.7／2.8 の見分けだけで、そのときは 2.8 に倒す）。

#### 実行ファイルが無いときの案内（`desk: skip no-exe`）

置き方の道具は pasta ではなく areka のリポジトリにあるので、その決まった絶対パスで案内する。

```
pwsh -NoProfile -File C:\home\maz\git\areka\tools\impl-watch-install.ps1
```

既定の枝の最新を取り込んだ areka の作業コピーで走らせる旨を添える。スキルはこのコマンドを案内するだけで、自分では走らせない。

#### 前置きステップの手順

1. **机の前置き**を呼ぶ。`desk: skip no-home|no-exe|bad-name` なら別の値をこしらえず、訳を記録し、開発者へ 1 行（`no-exe` は上の案内のコマンドを添える）伝え、チェックリストと最後の報告に「飛ばした・訳」を残す。この回は机のコマンドを 1 つも呼ばずに冒頭ステップへ進む（机は助けであり、使えないことで完了の手順を止めない）。
   - `no-home`: `机の段を飛ばしました（環境変数 AREKA_IMPL_WATCH_HOME が空）`
   - `no-exe`: `机の段を飛ばしました（置き場所に areka-impl-watch.exe が無い）。置くには: <上の案内のコマンド>`
   - `bad-name`: `机の段を飛ばしました（作業木かリポジトリの名前が識別の形に合わない: <ID> <R>）`
2. **見張り**: このセッションが立てた見張りが走っていなければ、`watch --id <ID> --repo <R> --name <feature>` をバックグラウンドで走らせる（走っていれば重ねて立てない）。ここからステップ8 の後の記録までが参加の区間。
3. **停止要請**: ここまでに見張りの `stop requested by <X>` を受けていたら、直ちに `stopped --id <ID> --wait` をバックグラウンドで走らせる（作業の途中でないので、ここが完了の手順の「止まる切れ目」。番を待つ間に受けたときも同じ）。止まっている間も、次の 4・5 の申し込みと番の待ちだけはしてよい。それ以外の段には入らない。
4. **バグの印**: その spec が不具合の修正（`brief.md` の目的が既存の振る舞いの誤りを直すこと）なら `--bug` を付ける。判断できなければ付けない。
5. **申し込む**: `merge --id <ID> --repo <R> --spec <feature> [--bug] --wait` をバックグラウンドで申し込み、開発者へ「マージの机の番待ち」と 1 行伝えてターンを終える。**番が来るまで、冒頭ステップを含む完了の手順のどの段にも入らない。**
6. **番が来たら**（`granted merge repo=<R>; last: …`）: ステップ0 の 1・2 の決め方で `{remote}`・`{default-branch}` を決め、`git fetch -q {remote} {default-branch}` → `git merge --no-edit {remote}/{default-branch}` で既定の枝を取り込む（ハーネスの同期の道具は資格情報つきの URL を断るので使わない。出力に URL を出さない）。ぶつかったら直してから続ける（番は持ったまま）。`last:` の 1 行を報告に添えて冒頭ステップへ進む。止まっている間に番を受けたら、再開（`resumed`）を待ってから取り込みへ進む。
7. **番の待ち・番の中の 3**:
   - `merge --wait` が `exit=3` で、自分の `cancel` の後でない（申し込みが消えた）: 開発者へ 1 行伝え、見張りを立ててから 5 の形で申し込み直す。自分の `cancel`（下の「中断条件」）の後の 3 は何もしない。
   - 番を持っている間に見張りが `exit=3`（記録が消えた）: ステップ8 の PR のマージへ進まない。見張りを立て直して 5 の形で申し込み直し、番が来てから 6 の形で既定の枝を取り込み直して続ける。知らせが手順の途中に来たら覚えておき、ステップ8 の PR のマージの前で番を持っていることを確かめる（ステップ8 の「マージの机」）。

### 冒頭ステップ: 未解決問題の棚卸（原則その場で解決・例外だけ既存 spec へ申し送り／`/kiro-discovery` で起票）

DoD ゲートより前に、**実装中に発生した未解決問題のうち未起票のもの**を棚卸しし、**原則このブランチでその場で解決**し、手順 3 の例外に当たるものだけを申し送りまたは起票する。完了してアーカイブした後では拾い漏れに気付けなくなるので、必ず最初に行う（繰り返し仕様でも行う）。その場の修正も Test Gate（ステップ1）の対象になるよう、DoD ゲートより前に済ませる。

1. **未解決問題を洗い出す**。拾う元は次の通り。
   - `tasks.md` の `## Implementation Notes`・`_Blocked:_` の付いたタスク
   - `design.md` の Open Questions / Risks / Revalidation Triggers
   - 実装・レビュー・最終検証（`/kiro-validate-impl`）の報告に残った未対応の指摘・先送り
   - 会話の中で「別 spec で」「後で」と決めたもの
2. **起票済みかを確かめる**。`.kiro/specs/`（`completed/` を含む）の brief.md と `.kiro/steering/roadmap.md` を検索する。本 spec の中で解決済みのものは除く。**「直さない」と裁定済みのものも除く**（触るコードのコメント・roadmap・完了 spec の台帳に取り下げの記録が無いかを確かめる）。
3. **原則としてその場で解決する**。次の**どれかに当たるものだけ**を手順 4・5 へ回す（当たらなければ直す。迷ったら直す）。小さな問題を起票に回す率が高すぎた反省から（2026-10-17 開発者「なるべくその場で解決しクローズ」「多少契約スコープが広がるのは許容」）、**本 spec の境界・要件の外のファイルや別クレートに手が届くことは理由にならない**。
   - 直し方が複数あり、開発者の裁定・要件の追加や改訂が要る（直し方が 1 通りに決まるなら、設計の小さな判断は自分で下して直す）→ 手順 5
   - 進行中の他 spec の担当範囲（brief・roadmap の行）と重なる・その spec のブランチと衝突する → 手順 4
   - 規模が spec 1 本に相当する（目安: このブランチで数時間では終わらない・新しい仕組みや新しい依存の追加が要る）→ 手順 5

   公開 API の小さな変更、触るファイルが十数個・数百行になること、本 spec の要件に無い直しになることは、上に当たらない限り直してよい。ただし、DSL 文法・`pasta.toml` の書式・永続化データの形・ゴーストの `scripts/` が呼ぶ Lua API について、既存のゴーストの書き方や保存済みのデータの意味を変える・読めなくするものは 1 つ目（裁定が要る）に当たる（足すだけで従来どおり動くものは直してよい）。

   解決の作法:
   - バグ修正には、直す前に赤になる決定論テストを 1 本添える（doc・文言だけの修正は不要）。
   - 修正は 1 件 1 コミット（`fix(<feature>): …` 等）で、ステップ1 の前に積む。触るのが本 spec の外なら、コミットの本文にその旨を 1 行書く。コードを変えたときは Test Gate をセッション記録で省略しない。
   - 直しに伴って他のテストが赤になったら、それも直す。着手して上の例外に当たると分かったときだけ、その修正を戻して手順 4・5 へ回す。
   - 解決したものは `tasks.md` の `## Implementation Notes` に「完了時にその場で解決」として 1 行ずつ記録し、PR の body（ステップ8）にも載せる。
4. **担当 spec が既にあるものは、その brief.md へ申し送る**（新しく起票しない）。本 spec で変えた契約・挙動（`design.md` の Revalidation Triggers に当たるもの、後続 spec が前提にしている実行経路・出力・警告の文言など）が、担当 spec の brief.md に反映されていなければ、その brief.md に「申し送り（{feature-name} より）」として追記する。書くのは、変わった事実・見直しが要る論点・参照先（本 spec の design.md の節など）。担当 spec の要件・設計を先回りで決めない（論点として渡すだけ）。担当 spec が要件フェーズ以降に進んでいる（requirements.md がある）場合は、brief.md への追記に加えて開発者に報告する。
   - 手順 3 で他 spec と重なると分かった問題も、新しい spec を立てずにその brief.md へ「件 N」として足す。
   - 申し送り先の spec が要件の段より先に進んでいれば、brief.md に加えて roadmap のその spec の行にも 1 行足す（着地の前に拾われるように）。
5. **担当の無いものを `/kiro-discovery` で起票する**。Skill ツールで `kiro-discovery` を起動する。1 件ずつでも、まとめて 1 回でもよい。
   - 起票する件ごとに、手順 3 のどの例外に当たったかを 1 行で添える（PR の body にも載せる）。言えないものは起票せず手順 3 へ戻して直す。
   - **ハーネスの「別の作業として切り出す」チップ（`spawn_task`）は使わない。** チップは開発者が 1 件ずつ押して指示する手間を生み、roadmap の台帳にも載らない。出してしまっていたら `dismiss_task` で取り下げて起票し直す。
6. 申し送り・起票した brief.md・roadmap.md の変更はステップ2でまとめてコミットする。**その場で解決した件数・申し送った件数・起票した件数をそれぞれ明示して記録する。0 件なら「0 件」と書く**（黙って飛ばさない）。

### ステップ0: 決定的解決（portable context）

リモート操作で用いる `{remote}` と `{default-branch}` を、**固定優先順序で1回だけ決定的に解決**する。各値はちょうど1つの結果（または明示的なスキップ）に収束させ、推測しない。解決した値は以降のステップ（特にステップ8）で `origin`/`main` のハードコードの代わりに再利用する。この優先順序は `kiro-start` の「Step 0: Resolve portable context」と整合している。

1. **デフォルトリモート（`{remote}`）**: `git remote` を実行し、以下の固定ルールを適用する。
   - `origin` が存在する → `{remote}` = `origin`。
   - そうでなく、リモートがちょうど1つだけ存在する → `{remote}` = そのリモート。
   - それ以外（リモートなし、または `origin` を含まない複数リモート）→ `{remote}` = none。リモート操作はすべてスキップ扱いとする（一度だけ警告する）。

   ```powershell
   $remotes = git remote
   if ($remotes -contains "origin") { $remote = "origin" }
   elseif (@($remotes).Count -eq 1) { $remote = $remotes }
   else { $remote = $null }  # none: リモート操作はスキップ
   ```

2. **デフォルトブランチ（`{default-branch}`）**: 以下の固定優先順序で決定的に解決する。
   - `{remote}` が解決済みなら、`git symbolic-ref --quiet --short refs/remotes/{remote}/HEAD` を読み、先頭の `"{remote}/"` プレフィックスを除去した名前。
   - それが空で、ローカルに `main` ブランチが存在する → `{default-branch}` = `main`。
   - そうでなく、ローカルに `master` ブランチが存在する → `{default-branch}` = `master`。
   - それ以外 → `{default-branch}` = 現在のブランチ。

   ```powershell
   $defaultBranch = $null
   if ($remote) {
     $defaultBranch = git symbolic-ref --quiet --short "refs/remotes/$remote/HEAD"
     if ($defaultBranch) { $defaultBranch = $defaultBranch -replace "^$remote/", "" }
   }
   if (-not $defaultBranch) {
     if (git show-ref --verify --quiet "refs/heads/main") { $defaultBranch = "main" }
     elseif (git show-ref --verify --quiet "refs/heads/master") { $defaultBranch = "master" }
     else { $defaultBranch = git branch --show-current }
   }
   ```

   `{default-branch}` は1つの具体的なブランチ名として確定し、以降で再評価しない。

> **以降のステップは、`origin`/`main` のハードコードではなく、ここで解決した `{remote}` / `{default-branch}` を用いる前提とする。** `{remote}` が none の場合、リモート同期（ステップ8）は安全にスキップし警告する。

### ステップ1: DoD（完了基準）ゲート検証

`.kiro/steering/workflow.md` の「完了基準（DoD）」セクションを読み込み、**workflow.md が定義する全ゲートを順に検証**する。判定ルールの本体は workflow.md（権威）にあり、このスキルは発火・オーケストレーションのみを行いルールを複製しない。

1. **workflow.md を読み込む**（未読の場合）
2. **workflow.md の DoD に列挙された全ゲート**（現状: Spec / Test / Doc / Steering / Soul、および条件付きの **Manual Sync Gate**・**Internals Sync Gate**）を順に検証する。ゲートの構成は workflow.md を正とし、ここで個数や内訳を固定しない。
3. **Test Gate**:
   - まず `session_store_sql` でセッション記録を確認し、直近のターンで `cargo test` が実行され全テスト成功していたか判定する
     ```sql
     SELECT t.content FROM turns t
     JOIN sessions s ON t.session_id = s.id
     WHERE s.id = (SELECT id FROM sessions ORDER BY start_time DESC LIMIT 1)
       AND t.content LIKE '%cargo test%'
       AND t.content LIKE '%test result%'
     ORDER BY t.created_at DESC LIMIT 3
     ```
   - **スキップ可**: セッション記録に `test result: ok` が確認でき、その後にテスト対象コードの変更がない場合、Test Gate をスキップする。スキップ時は完了チェックリストに「(セッション記録により省略)」と注記する
   - **スキップ不可**: セッション記録が見つからない、テスト結果が不明瞭、またはテスト後にコード変更がある場合は実行する:
   ```powershell
   cargo test --all 2>&1 | Select-String "test result:|FAILED|error\["
   ```
4. **Manual Sync Gate（条件付き）**: 判定ルールの本体は workflow.md「完了基準（DoD）> 6. Manual Sync Gate（条件付き）」にある（権威）。ここではその発火だけを行う。
   - **発火条件**: 当該 spec の変更が `book/`・`.claude/skills/pasta-ghost-authoring/`・`.claude/skills/pasta-lua-coding/` のいずれかに**触れる場合のみ**、鮮度チェックとリンク検証を実行する。
     ```powershell
     node book/tools/gen-skill-refs.mjs --check
     node book/tools/link-check.mjs
     ```
   - **中断**: いずれかが**非ゼロ終了**（スキル生成ファイルの不一致 / リンク切れ）した場合は、**ワークフローを中断し開発者に報告**する（下記6の「いずれかのゲート失敗時は中断」と整合）。解消フロー（マニュアル章を正として修正 → 再生成 → コミット → ゲート再実行）は workflow.md を参照。
   - **スキップ**: 当該 spec の変更が上記のいずれにも触れない場合は、このゲートを**スキップ**する（本ゲートは不要）。スキップ時は完了チェックリストに「(無関係変更によりスキップ)」と注記する。
5. **Internals Sync Gate（条件付き）**: 判定ルールの本体（照合規則・章ごとの判定・中断・スキップ）は workflow.md「完了基準（DoD）> 7. Internals Sync Gate（条件付き）」にある（権威）。ここではその発火だけを行う。
   - **変更ファイル一覧の取得**: ステップ2のコミットより前に、ステップ0で解決した `{default-branch}` を使って取得する（2 コマンドの出力の和）。
     ```powershell
     git diff --name-only --no-renames $(git merge-base HEAD {default-branch})
     git ls-files --others --exclude-standard
     ```
   - **照合**: 一覧を `book/src/internals/index.md` の「章と対象ソース範囲」表と、workflow.md の照合規則に従って照合する。一致した章ごとに判定結果（更新済み、または更新不要の理由）を完了チェックリストに記録する。
   - **リンク検証**: 一致が 1 件でもあれば、章の更新の有無にかかわらず実行する。
     ```powershell
     node book/tools/link-check.mjs
     ```
   - **中断**: workflow.md の中断条件（判定を満たさない章がある / link-check が非ゼロ終了）に該当した場合は、**ワークフローを中断し開発者に報告**する（下記6と整合）。
   - **スキップ**: 一致が 1 件も無い場合は、このゲートを**スキップ**し、完了チェックリストに「(対象領域外によりスキップ)」と注記する。
6. **いずれかのゲートが失敗した場合**: ワークフローを中断し、開発者に報告

### ステップ2: 未コミットファイルのコミット

実装中の変更をすべてコミットする。コミットメッセージ形式は workflow.md の規約に従う。

```powershell
git add -A
git commit -m "<type>({feature-name}): 実装完了

- 変更の要約（箇条書き）"
```

### ステップ3: completedフォルダへの移動

specディレクトリをcompleted配下へ移動する。

```powershell
New-Item -ItemType Directory -Path ".kiro/specs/completed" -Force | Out-Null
Move-Item ".kiro/specs/{feature-name}" ".kiro/specs/completed/"
```

**重要**: この時点ではspec.jsonを**編集しない**。VSCodeが編集中のファイルを追跡しており、移動前に編集すると移動操作と競合してファイルが元の場所に復活する。

### ステップ4: spec.jsonのステータス更新

**移動完了後に** spec.json を更新する。以下のフィールドを変更:

```json
{
  "phase": "completed",
  "completed_at": "YYYY-MM-DDTHH:MM:SSZ"
}
```

> **注意**: `"status"` フィールドは使用しない。`"phase": "completed"` のみで完了を表す。

### ステップ5: 参照パスの更新

他のspecファイルや親仕様がこのspecを参照している場合、パスを更新する。

1. **参照箇所の検索**:
```powershell
Get-ChildItem ".kiro/specs" -Filter "*.md" -Recurse |
  Where-Object { $_.FullName -notlike "*completed*" } |
  Select-String -Pattern "{feature-name}" |
  Select-Object -ExpandProperty Path | Sort-Object -Unique
```

2. **パスの一括置換**: `.kiro/specs/{feature-name}/` → `.kiro/specs/completed/{feature-name}/`

3. **親仕様への完了マーク**: 親仕様のdesign.mdに完了ステータス（✅）を反映する（該当する場合）

### ステップ6: 追加更新チェック

workflow.md の「実装完了時アクション」セクションおよび「ドキュメント保守」セクションに従い、以下を実施する。

#### 6-1. ロードマップ更新

`.kiro/steering/roadmap.md` を確認し、完了したSpecが「Specs (dependency order)」に記載されているか判定する。

**スコープ判定（優先順位）**:
1. `requirements.md` に明示的なロードマップ項目との紐付け記述がある場合
2. 開発者が直接指示した場合
3. ロードマップの Specs 一覧にこの feature-name が含まれる場合
4. 判断に迷う場合は開発者に確認

**スコープ内の場合**:
- 対応する `- [ ] {feature-name}` を `- [x] {feature-name}` に更新

**スコープ外の場合**: スキップ

#### 6-2. スキルドキュメント更新

workflow.md の「スキルドキュメント更新検討」セクションに従い、変更領域に応じた対象スキル（`pasta-ghost-authoring` / `pasta-lua-coding`）との整合性を確認・更新する。スキップ条件も workflow.md に準拠。

#### 6-3. ステアリング・ドキュメント更新

workflow.md の「ドキュメント保守 > 更新チェックリスト」に従い、該当するドキュメントを更新する。

### ステップ7: 完了最終コミット

移動・ステータス更新・参照パス修正・追加更新をコミットする。

```powershell
git add -A
git commit -m "chore({feature-name}): spec完了・アーカイブ"
```

### ステップ8: リモート同期（PR ベース）

> **権威的ソース**: workflow.md「実装完了時アクション > 3. リモート同期（ブランチ戦略）」に従う。リモート同期は **PR（Pull Request）ベース**であり、これが唯一の手順実体である。このスキルはフローを複製せず、本セクションを参照したうえで以下を実行する。

> **前提**: 本ステップは `origin`/`main` をハードコードせず、ステップ0で解決した `{remote}` / `{default-branch}` を用いる。フィーチャーブランチ／ワークツリーは Claude Code（ハーネス）が供給しており、このスキルは自前でブランチ／ワークツリーを作成・削除しない。1つの feature = 1つのブランチ = 1つの PR とし、完了時に1回だけ PR を作成して squash マージする。**`{default-branch}` への直接 push は一切行わない。**

確認不要。現在のブランチを判定し、以下を中断なく実行する。

```powershell
$branch = git rev-parse --abbrev-ref HEAD   # 現在の作業ブランチ（ハーネス供給）
```

#### PR 可否判定

以下を**すべて満たす**ときのみ PR を作成・マージする（PR 可）:

1. 現在ブランチが `{default-branch}` 以外（非デフォルトブランチ）。
2. ステップ0で解決した `{remote}` が none でない（リモートあり）。
3. `gh` が認証済み（`gh auth status` が成功）。

いずれかが欠ける場合は **PR 不可** とし、下記「フォールバック（PR 不可時）」へ進む。

#### PR 可: PR 作成 → squash マージ → リモートブランチ削除

```powershell
# 1. 現在ブランチを push して PR を作成（base = {default-branch}, head = 現在ブランチ）
gh pr create --base {default-branch} --head $branch --title "<subject>" --body "<body>"

# 2. squash マージ（--squash 固定、--delete-branch でリモートブランチを API 削除）
#    --subject / --body は下記「squash メッセージ生成」に従って供給する
gh pr merge --squash --delete-branch --subject "<subject>" --body "<body>"
```

- **CI の完了を待たない。** PR を作ったらすぐ squash マージする。マージ前の関門はステップ1の Test Gate（ローカルの `cargo test --all`。clippy `-D warnings` も通しておく）であり、CI の緑を見張らない。main が CI で赤くなったら次の作業より先に追いかけの修正で直す。取り消せない公開の前の検査は、リリース CI がタグのコミットで行う（workflow.md「3. リモート同期」）。
- **マージ成否はマージ API の結果のみで判定する。** `gh pr merge` の成否がマージ成否であり、それ以外の警告でマージ成功を覆さない。
- **リモートブランチ削除**: `gh pr merge --delete-branch` が **PR マージ成功後に** API でリモート feature ブランチを削除する。
- **ローカル後始末警告は非致命**: `--delete-branch` のローカル削除試行は、カレントワークツリーでブランチがチェックアウト中のため**ブロックされ警告を出す**ことがある。これは**非致命**でありマージ成功（API 結果）を覆さない。リモートブランチは API により削除済みである。
- **ローカルブランチ／ワークツリーの後始末はハーネスへ委譲**: このスキルは自分のワークツリー／カレントブランチを削除しない（構造的に不可）。ローカルブランチ・ワークツリーの teardown はハーネスがセッション/タスク境界で実施する。

**マージの机**（前置きステップで机の段が `on` のときだけ。呼ぶ形と終わり方の表は前置きステップのもの）:
- **`gh pr merge` の前**: マージの机の番を持っていることを確かめる。番を持つ間に見張りが `exit=3` で終わっていたら、ここで PR のマージへ進まず、前置きステップの 7 に従う（立て直し・申し込み直し・番の後に既定の枝を取り込み直す）。
- **squash マージが成功した後**: `gh pr view <PR 番号> --json mergeCommit -q .mergeCommit.oid` でマージ後のコミットを取り、`merged --id <ID> --pr <PR 番号> --sha <コミット>` を前景で呼ぶ。これで机を空けて参加を終える。繰り返しの spec でも同じ。その後に終わる見張りの 3 は何もしない。
- `merged` が `exit=3`（`not applied: not the merge holder`）: `status` を 1 回だけ呼び、`merge <R>:` の行の `last=` に自分の PR が載っているかを見る。載っていなければ開発者へ 1 行で伝える。

**squash メッセージ生成**（`gh pr merge --squash` の `--subject` / `--body`）:
- 固定文言にせず、**分岐点以降のコミット履歴を要約**して作成する。
- 手順:
  1. `git log --no-merges --pretty=format:"%h %s%n%b" {default-branch}..HEAD`（= `merge-base..HEAD`）で分岐点以降の全コミットを取得
  2. 対象 spec の `requirements.md` / `design.md` のタイトル・概要も参照し意図を補強
  3. 以下の形へ再構成:
     - **subject**（`--subject`）: `<type>({feature-name}): <機能全体を1文で表す要約>`
     - **body**（`--body`）: 主な開発仕様・変更内容を箇条書き（3〜7項目目安）。関連コミットは統合し、`fixup`/typo/WIP 等の些末な履歴は集約・省略。個々のコミット羅列ではなく「何を・なぜ作ったか」の開発単位で再構成する。

#### フォールバック（PR 不可時）

現在ブランチが `{default-branch}` である / `{remote}` が none（リモートなし・オフライン）/ `gh` 未認証 のいずれかの場合:

- **警告を出力**し、PR 作成・push を**スキップ**する。
- ローカルコミットは**そのまま保持**して継続する。
- **`{default-branch}` への直接 push は一切行わない。**
- **マージの机**（机の段が `on` のときだけ）: マージが無いまま手順が終わり、`merged` を呼べる PR が無いので、`leave --id <ID>` を前景で呼んで机を空け、参加を終える。その後に終わる見張り・待ちの 3 は何もしない。

#### 中断条件

PR の**作成またはマージ（API）が失敗**した場合（コンフリクト / mergeable でない / 権限不足等）は、**ブランチを削除せず**処理を中断し開発者へ報告する（復旧可能性を確保するため）。中断するのは**マージ API が失敗したとき**のみであり、`--delete-branch` のローカル削除警告（非致命）とは区別する。

**マージの机**（机の段が `on` のときだけ）: この中断条件か開発者の取りやめで完了の手順を途中でやめると決めた時点で、`cancel --id <ID>` を前景で呼び、申し込みと机を外す（参加は続ける）。直してすぐ続けるだけなら外さない。外した後に終わる `merge --wait` の 3 は何もしない（前置きステップの 7）。続けるときは前置きステップの 5 から申し込み直す。

---

## 完了チェックリスト

```
- [ ] 前置きステップ: マージの机を申し込み（merge --wait）→ 番（last: を報告に添えた）→ ステップ8 の後に merged（PR 番号とマージ後のコミット）。机が使えずに飛ばしたときは、その訳（環境変数が無い・実行ファイルが無い・値が断られた・失敗の 1 行）を記録
- [ ] 冒頭ステップ: 実装中の未解決問題のうち未起票のものを棚卸し済み——原則その場で解決（spec の外に手が届いても可・バグはテスト付き・1 件 1 コミット・Implementation Notes に記録）、手順 3 の例外に当たるものだけ、担当 spec があればその brief.md へ申し送り、担当が無ければ `/kiro-discovery` で起票（起票は 1 件ごとに当たった例外を添える。解決件数・申し送り件数・起票件数をそれぞれ記録。0 件なら「0 件」）。DoD ゲートより前
- [ ] DoD 全ゲート通過（workflow.md 準拠: Spec/Test/Doc/Steering/Soul）
- [ ] cargo test --all 成功（またはセッション記録により省略）
- [ ] Manual Sync Gate: 条件付き発火（book/・.claude/skills/pasta-ghost-authoring/・.claude/skills/pasta-lua-coding/ に触れる spec のみ `node book/tools/gen-skill-refs.mjs --check` と `node book/tools/link-check.mjs` を実行・非ゼロで中断／無関係変更はスキップ）
- [ ] Internals Sync Gate: 条件付き発火（変更ファイルが book/src/internals/index.md「章と対象ソース範囲」に一致する spec のみ、一致章ごとに「更新済み／更新不要の理由」を記録し `node book/tools/link-check.mjs` を実行・未記録の章または非ゼロで中断／一致なしは「(対象領域外によりスキップ)」と記録。判定ルールは workflow.md Gate 7）
- [ ] 未コミットファイルをコミット済み（ステップ2）
- [ ] completedフォルダへ移動済み（ステップ3）※繰り返し仕様はスキップ
- [ ] spec.json の phase を "completed" に更新済み（ステップ4）※繰り返し仕様はスキップ
- [ ] 参照パス更新済み（ステップ5）※繰り返し仕様はスキップ
- [ ] ロードマップ更新済み（スコープ内の場合）
- [ ] スキルドキュメント同期済み（該当する場合）
- [ ] 完了コミット済み（ステップ7）
- [ ] ステップ0で `{remote}` / `{default-branch}` を決定的解決済み
- [ ] リモート同期完了（ステップ8、PR ベース。解決した `{remote}`/`{default-branch}` を使用）
      - PR 可（非デフォルトブランチ かつ `{remote}` あり かつ `gh` 認証あり）: `gh pr create --base {default-branch} --head <current>` → `gh pr merge --squash --delete-branch --subject … --body …`（メッセージは `merge-base..HEAD` 履歴を要約）。マージ成否は API 結果のみで判定し、`--delete-branch` のローカル削除警告は非致命として継続。リモートブランチは API 削除、ローカルブランチ／ワークツリーはハーネス teardown へ委譲
      - PR 不可（`{default-branch}` 上 / `{remote}` none / `gh` 未認証）: 警告して PR・push スキップ、ローカルコミット保持（`{default-branch}` への直接 push なし）
      - PR 作成／マージ（API）失敗: ブランチを残し中断・報告
```

---

## エラー回避

### VSCode変更確定問題
- **症状**: 移動したファイルが元の場所に復活する
- **対策**: spec.jsonは必ずステップ4（移動後）で編集。移動前に編集しない

### 参照パス更新漏れ
- **症状**: 後続specが旧パスで参照しファイルが見つからない
- **対策**: ステップ5で `Select-String` による網羅的検索を実施

### コミット漏れ
- **症状**: pushしたが変更が反映されていない
- **対策**: 各コミット前に `git status --short` で確認

### テスト失敗時
- **症状**: `cargo test --all` が失敗
- **対策**: ワークフローを中断し開発者に報告。テスト修正後に再実行

### リモート同期関連（ステップ8 PR ベース）

#### PR 作成失敗
- **症状**: `gh pr create` が失敗（既存 PR との衝突 / push 権限不足 / ネットワーク等）
- **対策**: **ブランチを削除せず**中断して開発者へ報告。既存 PR がある場合はその PR を確認して再マージを検討

#### マージ不可（mergeable でない / API 失敗）
- **症状**: `gh pr merge --squash` が失敗（コンフリクトで mergeable=false / 必須チェック未通過 / 権限不足等）
- **対策**: **ブランチを削除せず**中断して開発者へ報告。コンフリクトは GitHub 上または別途解決のうえ再実行。中断判定は**マージ API の結果のみ**で行い、`--delete-branch` のローカル削除警告とは混同しない

#### gh 未認証 / リモートなし（PR 不可）
- **症状**: `gh auth status` が失敗、または `{remote}` が none
- **対策**: 警告を出力し PR・push をスキップ。ローカルコミットは保持して継続する。**`{default-branch}` への直接 push は行わない**

#### `{default-branch}` 上で承認された
- **症状**: 現在ブランチが `{default-branch}`（PR の head に使えない）
- **対策**: 警告を出力し PR・push をスキップ、ローカルコミット保持。通常はハーネス供給の非デフォルトブランチ上で完了する想定

#### `--delete-branch` のローカル削除警告
- **症状**: `gh pr merge --delete-branch` がローカルブランチ削除を試みてブロックされ警告を出す（カレントワークツリーでチェックアウト中のため）
- **対策**: **非致命として無視し継続**。リモートブランチは API で削除済み。ローカルブランチ／ワークツリーの後始末はハーネスのワークツリー teardown に委ねる（このスキルは自分のワークツリーを削除しない／できない）
