# Research & Design Decisions

## Summary
- **Feature**: `act-token-grouping-fix`
- **Discovery Scope**: Extension（既存のグループ化・組み立ての修正と、死蔵モジュールの撤去）
- **Key Findings**:
  - 先頭の表示制御を捨てているのは `group_by_actor`（`act.lua` 58–66 行）の else 分岐だけで、組み立て（`sakura_builder.lua`）はアクター nil のグループ（164 行の `if actor and …`）を既に受け付ける。ただし nil グループに乗せる案 (a) は、立ち絵の復旧（`appearance.lua`）と組み合わせると作者の `\s[N]` を打ち消し得る。案 (b)（次の発言者のグループに入れる）は復旧の「先頭タグ列」規則にそのまま乗り、`act.lua` だけで完結する。
  - `spot`・`clear_spot` の並び替えが**見える**差になるのは実質 `clear_spot` の場合（呼び出し先のグローバルシーンの冒頭の `％` で、同じ発言者の後続の発言が前のグループに吸い込まれ、新しい立ち位置の `\p[N]` が出ない）。`set_spot` 単独では、組み立てが同じ発言者の間で `\p` を出し直さないため出力のバイトはほぼ変わらず、変わるのは後処理（ウェイト挿入・budoux）の区切りだけである。
  - CT（`ct.lua`）を参照するのは `ct_test.lua`・`tests/lua_specs/init.lua`・マニュアル `internals/execution-model.md`・`internals/index.md` の表・`pasta/shiori/init.lua` のコメント 1 行だけ。zip 埋め込みはディレクトリ全体を固めるため、ファイル削除だけで配布物から消える。`internals/index.md` の表はマニュアル CI のパス実在検査の対象で、更新しないと CI が落ちる。

## Research Log

### 1. 先頭の表示制御が捨てられる箇所と、組み立て側の受け口
- **Context**: Requirement 1。brief は案 (a) を「`sakura_builder.lua` が既に nil を受けるため小さい」とした。
- **Sources Consulted**: `crates/pasta_lua/pasta_scripts/pasta/act.lua`（`group_by_actor`・`merge_consecutive_talks`・`build`）、`pasta/shiori/sakura_builder.lua`（`BUILDER.build`・`emit_actor_switch`・`emit_inner_token`）、`pasta/shiori/appearance.lua`（`observe`・`restore`・`leading_tags`）、`pasta/shiori/act.lua`（`SHIORI_ACT_IMPL.build`）。
- **Findings**:
  - `group_by_actor` の else 分岐（`surface`・`wait`・`newline`・`clear`・`choice`・`choice_timeout`）は `current_actor_token == nil` のとき何もしない（コメントで「捨てる」と明記）。
  - `raw_script` は既に「グループがあれば中、無ければ最上位」のハイブリッド。
  - `BUILDER.build` は `last_actor` などをビルドごとに `nil` から始める。アクター nil のグループは切り替えを出さず、現在のスコープ（ビルドの冒頭なら未切り替え＝ベースウェアの既定スコープ `\0`）に内側を出力する。内側の観測は `APPEARANCE.observe(appearance, nil, …)` 相当になり、`surface` を含むと `observe_raw` が `state.spots = {}`（全スポット不明）にする。
  - `APPEARANCE.restore` は、同じアクターが同じスポットで続く（`owners[spot] == name` かつ `last_spots[name] == spot`）とき復旧しない。それ以外は `leading_tags(tokens)`（グループの内側の先頭タグ列。`surface` トークンと `talk` 先頭の `\s[…]`）にサーフェス変更があれば、サーフェスの復旧を抑止する。
  - DSL から先頭の表示制御が生じるのは、選択肢行 `＠？`（`act:choice`）・キューコマンド `!select`（`act:choice_timeout`）・`＞ゴースト終了（ミリ秒）`（`GLOBAL.close_ghost` の `act:wait`）。アクション行の `＠表情` は `talk` に展開されるため影響しない。マニュアル `grammar/block-structure.md` は選択肢行を「その行の位置に出力する」、`grammar/call-jump.md` は `＞ゴースト終了（ミリ秒）` を「`\_w[ミリ秒]\-` を出力する」と書いており、出力の先頭に書くと現行はこれに反する（`\-` は `raw_script` なので出るが、ウェイトと選択肢が消える）。
- **Implications**:
  - 案 (a) を既存の nil 経路に乗せると、(1) 立ち位置 1 の発言者の前に書いた `\s[5]` が `\0` に効く、(2) 直前にそのスポットで話していなかった発言者では、`\p[N]` の直後に復旧の `\s[既定]` が出て作者の `\s[5]` を打ち消す（例 `\s[5]\_w[500]\p[0]\s[0]本文`）。案 (a) を正しくするには組み立てと復旧の両方に手が入り、「小さい」前提が崩れる。
  - 案 (b) では先頭の表示制御が次の発言者のグループの先頭に入るため、組み立ては既存の S4 経路（has-text・pending 不変）で出力し、復旧は `leading_tags` で `surface` を見つけて抑止する。`sakura_builder.lua`・`appearance.lua` の変更は不要の見込み。
  - ただし (b) でも「後に発言が無い」「先に `raw_script` が来る」場合の行き先が要る（Requirement 1.4・前提 A2）。これは既存の nil グループ（または最上位のトークン）として出せば、組み立ての既存の経路で出力できる。`raw_script` を飛び越えて後ろの発言者に付けると、`raw_script` との順序が入れ替わる。

### 2. `spot`・`clear_spot` がグループを閉じないことの影響
- **Context**: Requirement 2。brief は「2 行で閉じる」を想定。
- **Sources Consulted**: `act.lua` 34–35 行・`set_spot`・`clear_spot`、`crates/pasta_lua/src/code_gen/scope_gen.rs` 275–286 行、`sakura_builder.lua` の S5・S6、`book/src/internals/talk-output.md` 229 行。
- **Findings**:
  - トランスパイラは、アクター指定行（`％`）を持つグローバルシーンの `__start__` の冒頭でだけ、`act:clear_spot()` に続けて `act:set_spot(名前, 番号)` を積む。そのため発言の後に `clear_spot` が来るのは、出力の途中で `＞` によって `％` を持つ別のグローバルシーンを呼んだ場合（と Lua から直接呼んだ場合）である。
  - 現行では、例えば `％さくら、うにゅう` のシーンでさくらが話した後、`％うにゅう、さくら`（立ち位置を入れ替え）のシーンを呼んでさくらが話すと、後の発言は前のさくらのグループに入り、`clear_spot`・`set_spot` はそのグループの出力の後に処理される。後の発言は古い立ち位置 `\p[0]` のまま出る。
  - 閉じた場合、`BUILDER.build` の S5 が `last_actor`・`last_spot`・`spot_has_text`・`pending_break` をリセットするため、後の発言の前に `\p[新しい位置]` が出て、段落区切りの判定も持ち越さない。外見状態（`owners`・`last_spots`）は `clear_spot` で消えないため、同じスポットの同じアクターなら復旧タグは出ない。
  - `set_spot` 単独で閉じた場合、同じ発言者が続けば `last_actor` が同じなので `\p` は出ない。出力の差は、`merge_consecutive_talks` がグループをまたいで結合しないため、`talk_to_script`（ウェイト挿入・budoux 改行）が 2 回に分かれることだけである（budoux の行幅は呼び出しごとに数え直す）。
  - 閉じた後、発言より先に表示制御が来た場合の行き先は、何もしないと Requirement 1 の「先頭の表示制御」と同じ扱いになる。`set_spot` の後は直前の発言者に付ける（前提 A4）ため、直前の発言者のグループを開き直す（または `current_actor` を保持して遅延で開く）仕掛けが要る。`clear_spot` の後は先頭扱い（前提 A3）。
- **Implications**: `spot` と `clear_spot` で閉じた後の扱いが異なる。`current_actor_token` と `current_actor` を別々に戻す（`clear_spot` は両方 nil、`set_spot` はグループだけ閉じて `current_actor` を残す）形で表現できる見込み。

### 3. CT（`ct.lua`）の撤去範囲
- **Context**: Requirement 4・前提 A6。
- **Sources Consulted**: `git grep`（`ct.lua`・`ct_test`・`require("ct")`）、`book/src/internals/loader.md` 262 行（zip の対象はツリー全体）。
- **Findings**:
  - コード上の利用者は 0。参照は `crates/pasta_lua/tests/lua_specs/ct_test.lua`、`tests/lua_specs/init.lua` 45 行の登録、`book/src/internals/execution-model.md`（20 行の扱う事項、336–344 行の節、382 行のソースの所在）、`book/src/internals/index.md` 84 行（章と対象ソース範囲の表）、`crates/pasta_lua/pasta_scripts/pasta/shiori/init.lua` 17 行のコメント（「3.47 ct.lua と同方針の既知負債」）。
  - `.kiro/specs/review-improvement-loop/` の記録（matrix.md・tasks.md・report）にも登場するが、過去の記録なので変更しない。
  - 埋め込み zip はツリー全体を固めるため、ファイル一覧を持つ Rust 側の登録は無い（`extract_tests.rs` にも `ct` の名前は無い）。
  - スキル `references/` には `ct` の記述が無い。`book/tools/gen-skill-refs.mjs` の生成対象のうち本仕様に関わるのは `lua/script-api.md`（→ `pasta-lua-coding/references/script-api.md`）だけで、`lua/patterns.md`・`internals/talk-output.md`・`internals/execution-model.md`・`internals/index.md` は生成対象外（確認済み）。
- **Implications**: 撤去は削除 2 ファイル＋登録 1 行＋マニュアル 2 章。`internals/index.md` の表を直さないとマニュアル CI のパス実在検査（`internals-path`）が落ちる。`shiori/init.lua` のコメントは Wave 2 の編集範囲の外（設計 D2）。

### 4. 既存テストへの影響
- **Sources Consulted**: `tests/lua_specs/act_grouping_test.lua`・`act_test.lua`・`act_choice_test.lua`・`sakura_builder_test.lua`・`shiori_act_test.lua`・`shiori_entry_test.lua`・`appearance_test.lua`。
- **Findings**:
  - 先頭の表示制御が捨てられることを固定するテストは見当たらない（`act_grouping_test.lua` の spot・clear_spot のテストは spot・clear_spot が先頭にある列で、閉じても結果は同じ）。`act_choice_test.lua`・`shiori_entry_test.lua`（`close_ghost`）は `act.token` の中身を見るだけで `build` を通さない。
  - `sakura_builder_test.lua` はグループ化済みの列を直接与えるため、グループ化の変更の影響を受けない。
  - Rust 側の E2E（`crates/pasta_lua/tests/shiori/`・`crates/pasta_sample_ghost/tests/`）で、出力の先頭に選択肢・ウェイトを置くシーンや、発言の後に `％` を持つシーンを呼ぶシーンがあるかは未確認（**Research Needed**: 全テストを走らせて差分を確認する）。サンプルゴースト `hello-pasta` の `actors.pasta` は `％女の子`・`％男の子` を持つが、各シーンの冒頭での `clear_spot` は先頭なので影響しない。
- **Implications**: 既存テストはほぼそのまま通る見込み。新しい挙動はテストの追加で固定する（Requirement 6）。

## Requirement-to-Asset Map

| 要件 | 既存資産 | ギャップ | 種別 |
| ---- | -------- | -------- | ---- |
| 1.1–1.2 先頭の表示制御を次の発言者のスコープへ | `group_by_actor` の else 分岐 | 捨てている。保留して次の発言者のグループの先頭へ入れる仕掛けが無い | Missing |
| 1.3 復旧で打ち消さない | `APPEARANCE.restore` の `leading_tags` | 案 (b) なら既存の規則で満たす。案 (a) なら Missing | Constraint |
| 1.4 発言が無い・`raw_script` が先 | `BUILDER.build` のアクター nil の経路、最上位 `raw_script` | グループ化側で保留分を nil グループ（または最上位）として出す処理が無い | Missing |
| 1.5–1.6 現行規則の維持 | `group_by_actor`・`raw_script` のハイブリッド分類 | なし | — |
| 2.1–2.3 `clear_spot` の後の発言 | `BUILDER.build` S5 のリセット | グループを閉じていない（34–35 行） | Missing |
| 2.4 `clear_spot` の後の表示制御 | 1.x の仕組み | 1.x と同じ仕組みを使う | Missing |
| 2.5–2.6 `set_spot` の後 | `BUILDER.build` S6 | グループを閉じつつ直前の発言者を覚える仕掛けが無い | Missing |
| 3.x 修正対象以外の不変 | 既存のバイト比較テスト | 2 現象以外に差が出ないことの確認（E2E を含む） | Unknown |
| 4.x CT の撤去 | `ct.lua`・`ct_test.lua`・`init.lua` 登録 | 削除のみ。`shiori/init.lua` のコメントは範囲外 | Constraint |
| 5.x マニュアル | `script-api.md` 161–172・408・424 行、`patterns.md` 61 行、`talk-output.md` 101–110・229 行、`execution-model.md`、`internals/index.md` | 記述の更新。`script-api.md` は生成スキル `pasta-lua-coding/references/script-api.md` の元 | Missing |
| 6.x テスト | `act_grouping_test.lua`・`shiori_act_test.lua`・`appearance_test.lua` の流儀 | 新しいテストケースの追加 | Missing |

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: グループ化だけを拡張（案 (b)＋フォールバック） | `group_by_actor` に「保留中の先頭トークン」を持たせ、次の発言で開くグループの先頭に流し込む。発言が来ないまま終わる・`raw_script` が来たら、保留分をアクター nil のグループ（または最上位）として出す。`clear_spot` は `current_actor_token`・`current_actor` を両方戻し、`set_spot` はグループだけ閉じて直前の発言者を残す | 編集が `act.lua` のグループ化の範囲に収まる（Wave 2 の並走条件に合う）。組み立て・復旧・段落区切りの規則に手を入れない | `group_by_actor` の分岐が増え、luacheck の `max_cyclomatic_complexity = 15` を超える恐れ（ローカル関数への切り出しで対処） | 推奨候補 |
| B: 組み立て側で扱う（案 (a) を正しく） | グループ化はアクター nil のグループを作るだけにし、組み立てで nil グループの次の発言者を先読みして `\p[N]` を前倒しする、または復旧の判定に nil グループを含める | グループ化は 2〜3 行の変更で済む | `sakura_builder.lua`（複雑度 22 を ponytail 許容中）と `appearance.lua`（範囲外の `actor-surface-restore` の規則）に手が入る。先読みはビルドの状態機械を複雑にする | 規則の変更が範囲外に及ぶ |
| C: 出力しない規則を維持し、警告する | 捨てる挙動を残し、捨てたときに警告ログを出す。マニュアルの記述を保つ | 出力のバイトが一切変わらない | brief の Desired Outcome は許すが、DSL の選択肢行・`＞ゴースト終了（ミリ秒）` を出力の先頭に書くとマニュアル（文法章）の約束に反したまま残る | 要件ドラフトでは不採用（前提 A1） |

## Design Decisions

### Decision: 先頭の表示制御の行き先（前提 A1・A2）
- **Context**: Requirement 1。brief の案 (a)/(b)。
- **Alternatives Considered**:
  1. 案 (a) — アクター nil のグループとして `\p` の前に出す
  2. 案 (b) — 次の発言者のグループの先頭に入れる。発言が無い・`raw_script` が先なら (a) の形で出す
- **Selected Approach**（要件ドラフトの前提。議題で確定する）: 2。
- **Rationale**: Research Log 1 のとおり、(a) は立ち絵の復旧と組み合わせると作者の `\s` を打ち消し得て、出力の冒頭のスコープ `\0` に効くため立ち位置 1 の発言者で意図と食い違う。(b) は既存の復旧規則にそのまま乗る。
- **Trade-offs**: マニュアルの「表示制御は直前の発言者に入る」に「出力の先頭（と `clear_spot` の後）では次の発言者に入る」という例外が増える。
- **Follow-up**: 先頭の `clear` が次の発言者のスコープで `\c` になること、先頭の `choice` が発言者の `\p[N]` の後に出ることを、テストで確かめる。

### Decision: `set_spot` 単独での後処理の区切り（前提 A5）
- **Context**: Requirement 2.5。グループを閉じると同じ発言者の `talk` の結合が切れる。
- **Alternatives Considered**:
  1. 閉じる（後処理が 2 回に分かれることを許容する）
  2. `set_spot` ではグループを閉じず、並びだけ保つ別の表現にする
- **Selected Approach**（前提）: 1。brief の方針（2 行で閉じる）に沿う。DSL からは `set_spot` 単独が生じないため、影響は Lua から直接 `act:set_spot` を呼ぶ場合に限られる。
- **Follow-up**: 設計でバイト比較テストに含める。

## Implementation Complexity & Risk
- **Effort**: S（1–3 日）— 変更は `group_by_actor` の分岐の追加と CT の削除、マニュアル数章。組み立て・復旧は変更しない見込み。
- **Risk**: Low〜Medium — 既存パターンの拡張で範囲は明確。ただし出力のバイトが変わる 2 現象（先頭の表示制御・`clear_spot` を挟む同じ発言者）について、Rust 側の E2E とサンプルゴーストに差が出ないかを全テストで確認する必要がある（Medium の要因）。

## Recommendations for Design Phase
- 推奨: Option A（グループ化だけを拡張。案 (b)＋フォールバック）。`sakura_builder.lua`・`appearance.lua` は変更しない前提で設計し、必要が生じたら Boundary Commitments に明示する。
- 決めること:
  - 保留中の先頭トークンの持ち方と、nil グループ（Requirement 1.4）を最上位のトークンで表すかアクター nil のグループで表すか（組み立ての既存経路はどちらも受ける。nil グループなら内側の観測がアクター nil 扱いで同じ）。
  - `set_spot` の後に直前の発言者のグループを開き直す方法（D1）。
  - `group_by_actor` の複雑度対策（ローカル関数への切り出し）。
  - `shiori/init.lua` のコメントの扱い（D2）。
- **Research Needed**:
  - 全テスト（`cargo test -p pasta_lua`・`-p pasta_shiori`・`-p pasta_sample_ghost`）で、2 現象以外の出力差が無いことの確認。

## Risks & Mitigations
- 既存ゴーストで、出力の先頭の選択肢・ウェイトが「消えていたこと」に依存したトークがあると、出力が増える — 修正対象の現象そのものであり、マニュアルの文法章の約束どおりになる。リリースノートに書く（`release-workflow` の範囲）。
- `％` を持つシーンを発言の後で呼ぶゴーストで、後の発言の前に `\p[N]` が出るようになる — 修正対象の現象そのもの。立ち位置を入れ替えないシーンでは同じ `\p[N]` が出るだけで見た目は変わらない。
- `require("ct")` を書いたゴーストが起動時にエラーになる — `obj:defer` が呼べないため実用の依存は無い。マニュアルから節を削る。

## References
- `.kiro/specs/act-token-grouping-fix/brief.md` — 問題・方針・範囲
- `book/src/internals/talk-output.md` — グループ化・組み立て・復旧の現行仕様
- `book/src/lua/script-api.md#表示制御` — 利用者向けの現行規則
- `book/src/grammar/block-structure.md#選択肢行`・`book/src/grammar/call-jump.md#ゴースト終了` — DSL から生じる先頭の表示制御の約束

## 要件ディスカッションの決定（2026-10-04）

上の分析と推奨のうち、次の 3 点はディスカッションで覆った。設計は本節と requirements.md を正とする。

- **先頭の表示制御は案 (a)（アクター未指定として、積んだ位置にそのまま出す）に決定**。上の「Decision: 先頭の表示制御の行き先」と Option A の「案 (b)＋フォールバック」は採らない。想定外の書き方（発言より前の `＞surface（5）` など）に対して作者の意図を推測しない、という原則による。Research Log 1 で案 (a) の問題とした 2 点（既定のスコープに効く・復旧タグが出る）は、アクター未指定の生のさくらスクリプトと同じ扱いであり、仕様どおりの動作として受け入れる。保留して次の発言者へ流し込む仕掛けと、発言が無い場合のフォールバックは不要になった。
- **`set_spot`（`spot` トークン）ではグループを閉じない。閉じるのは `clear_spot` だけ**。上の「Decision: `set_spot` 単独での後処理の区切り」は採らない。`set_spot` をまたいで同じグループに入るのは切り替えを伴わない同じ発言者の発言だけで、閉じてもスコープ切替タグは変わらず、後処理の区切りが変わる副作用しか無いためである。直前の発言者のグループを開き直す仕掛けは不要になった。
- **CT は撤去で確定**（議題にせず閉じた）。

実装の見込み: `group_by_actor` で、(1) グループが無いときの表示制御などをアクターの無いグループに入れる、(2) `clear_spot` で `current_actor_token`・`current_actor` を戻す。`sakura_builder.lua`・`appearance.lua` は変更しない。
