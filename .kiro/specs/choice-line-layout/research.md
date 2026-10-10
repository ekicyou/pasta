# Research & Design Decisions

## Summary
- **Feature**: `choice-line-layout`
- **Discovery Scope**: Extension（さくらスクリプトの組み立ての既存の状態機械に、選択肢の前後の行の区切りを足す。原因が出力でなければ、見本の辞書と章の修正に切り替わる）
- **Key Findings**:
  - 選択肢の出力は `\![*]\q[表示,ジャンプ先,グローバルシーン名]` だけで、前にも後ろにも、選択肢と選択肢の間にも改行を出さない（`crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` の `choice_to_script`・`inner_token_to_string`）。brief の候補 1（選択肢の前に改行が出ていない）がコードの上では成り立つ。
  - UKADOC は `\q[タイトル,ID]` について「テキストの自動改行を求める場合は `\__q[]` タグを使うこと」と書いている。`\q` の中の文字は折り返されないので、台詞の後ろに続いた選択肢は行の右端で切れる。実機の観察（「をする」が切れる）と合う。
  - 2 つの選択肢の間にも改行が無い。2 つ目が次の行に出たのは、1 つ目が右端を越えた結果とみられ、短い選択肢どうしなら同じ行に並ぶはずである（実機では未確認）。「1 つずつ別の行に」を満たすには、1 つ目の前だけでなく、選択肢ごとに行の区切りが要る。
  - 組み立ての状態機械は「その吹き出しに字を出したか」（`spot_has_text`）を既に持っている。選択肢の前に行の区切りを入れるかの判定は、この状態の上に足せる。
  - 実機のさくらスクリプトは、hello-pasta の `profile/pasta/logs/pasta.log` に出る（`pasta.toml` の `level = "debug"`。`pasta_shiori` は 204 以外の応答を DEBUG で記録する）。この文書を書いた時点では採っていない。
  - emo2（`ghost_dev`）は選択肢行を使わず、台詞の中に `\q[…]`・`\n`・`\_l[…]` を手で書いている。選択肢のトークンだけを直す限り、emo2 の出力は変わらない。

## Research Log

### 選択肢の出力の経路
- **Context**: brief の「1 つ目の選択肢の前に改行を入れているかは調べていない」。
- **Sources Consulted**: `crates/pasta_lua/src/code_gen/scope_gen.rs`（`generate_choice`・`generate_choice_timeout`）、`crates/pasta_lua/pasta_scripts/pasta/act.lua`（`ACT_IMPL.choice`・`group_by_actor`・`merge_consecutive_talks`）、`crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`
- **Findings**:
  - 選択肢行 `＠？ジャンプ先「表示」` は `act:choice("ジャンプ先", "表示")` に変わる。`!select(10)` は `act:choice_timeout(10)` に変わる。どちらも改行を足さない。
  - `act:choice` は `{ type = "choice", target, display, scope }` を積むだけである。グループ化では、選択肢は直前の話し手のグループに入る（グループが無ければ、話し手なしのグループを開く）。
  - 組み立て（`BUILDER.build`）では、選択肢は「S4」の枝（字を出さないトークン）を通り、`\![*]\q[…]` をそのまま出す。`has-text` も、段落の区切りの保留（`pending_break`）も変えない。
  - 台詞の行の終わりに、改行は出ない（文法の規則。改行は作者が `\n` を書く。`book/src/grammar/action-line.md` の「改行」）。
  - コードから導いた 9 段目の出力（ウェイトのタグと表情の番号は省略。`N` は登録の通し番号）:

    ```text
    \p[0]\s[…]ねえ、ユーザーさん。なんの話をしよっか？\![*]\q[おやつの話をする,おやつの話,OnMouseDoubleClick_N]\![*]\q[おでかけの話,おでかけの話,OnMouseDoubleClick_N]\![set,choicetimeout,10000]\e
    ```

  - 既存のテストの期待値も同じ形である。`crates/pasta_shiori/tests/call_execution_correctness_e2e_test.rs` の `\p[0]どれ？\_w[450]\![*]\q[甲,選択肢甲,OnChoiceMenu_1]\![*]\q[乙,選択肢乙,OnChoiceMenu_1]\e` など。
- **Implications**: 候補 1 がコードの上で成り立つ。候補 2（改行は出ているが折り返しと重なる）は、改行が出ていないので成り立たない。候補 3（バルーンの幅）は、台詞の後ろに選択肢が続くかぎり、幅を広げても長い問いかけで再発する。実機の出力で確かめる（Requirement 1）。

### ベースウェアの仕様（`\q` と折り返し）
- **Context**: 「をする」が切れ、2 つ目が次の行に出る理由。
- **Sources Consulted**: UKADOC「さくらスクリプトリスト」の `\q[タイトル,ID]`・`\q[タイトル,ID,r2,r3...]`・`\__q[ID,...]`・`\![*]`・`\![set,choicetimeout,時間]`
- **Findings**:
  - `\q[タイトル,ID]`:「テキストの自動改行を求める場合は `\__q[]` タグを使うこと」。
  - `\__q[ID,...]` 〜 `\__q`:「`\q[]` タグと異なり、自動改行も行われる」。範囲の中にさくらスクリプトも挟める。
  - 旧仕様の `\q[ID][タイトル]` は「選択肢は改行される」。今の `\q[タイトル,ID]` は、選択肢ごとの改行をしない。
  - `\![*]` は、バルーンが決めたマーカーを表示する。
  - `\![set,choicetimeout,時間]` は、選択肢より後ろに書いても効く。
- **Implications**: 選択肢ごとの改行は、出す側（pasta か作者）が入れるものである。UKADOC の記述例も、選択肢を `\n` なしで並べている（1 行に並べる前提の例）。2 つ目の選択肢が次の行に出た理由は、UKADOC からは決まらない（下の Research Needed 2）。

### 既にある改行の規則（話し手の切り替え）
- **Context**: 選択肢の前の行の区切りを、どの状態から判定できるか。
- **Sources Consulted**: `sakura_builder.lua`（`spot_has_text`・`pending_break`・`emits_text`）、`book/src/reference/pasta-toml.md` の `spot_newlines`、`.kiro/specs/completed/sakura-script-newline/`
- **Findings**:
  - `spot_has_text[spot]` は「1 回の出力の中で、その吹き出しに字を出したか」を持つ。`\c` と `clear_spot` で偽へ戻る。
  - 字を出すかの判定（`emits_text`）は、字のある `talk` と、字を表示する `sakura_script` だけを数える。`raw_script`・表情・ウェイト・選択肢は数えない。
  - 話し手が切り替わって、字のある吹き出しへ戻るとき、次の字の直前に `\n[150]`（`spot_newlines` × 100）を出す。保留（`pending_break`）は字が来るまで出さない。
  - `BUILDER.build` は、luacheck の複雑度の上限（15）を越えていて（22）、`-- luacheck: ignore 561` と `ponytail:` のコメントで許している。コメントは「分割はリファクタ時に（特性化テスト先行）」と書いている。
- **Implications**: Requirement 2 の 1・3（字の後なら区切る・最初なら区切らない）は `spot_has_text` で判定できる。2・5（選択肢の後）は、吹き出しごとの「選択肢を出した」状態が新しく要る。保留中の `\n[150]` と、選択肢の後の台詞の区切り（Requirement 2 の 5）が重なる場合の扱いを、設計で決める。枝を足すと `build` の複雑度がさらに上がる。

### 既存のテストと文書への波及
- **Context**: 出力を変えたときに一緒に変わるもの。
- **Sources Consulted**: `crates/pasta_lua/tests/lua_specs/`、`crates/pasta_shiori/tests/`、`crates/pasta_lua/tests/`、`crates/pasta_sample_ghost/tests/`、`book/src/`、`book/tools/gen-skill-refs.mjs`
- **Findings**:
  - `crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua` の選択肢のテスト（1612〜2010 行あたり）は、部分一致（`find`）と、2 種類のバッファの出力の一致で見ている。出力に改行が増えても落ちないものが多い。新しい配置のテストは無い。
  - `crates/pasta_lua/tests/lua_specs/shiori_act_test.lua:751` は完全一致で、`\s[5]\_w[500]\n\c\![*]\q[d,t]\![set,choicetimeout,30000]\p[0]A\e` を期待する。話し手なしのグループで `\c` の直後に選択肢を出し、その後で話し手 A が話す形である。【仮定 A2】（最初の選択肢の前は区切らない）なら選択肢の前は変わらない。選択肢の後の `A` は別のグループ（スポット未解決 → 0）なので、【仮定 A3】をどう当てるかで期待値が変わる。
  - `crates/pasta_shiori/tests/call_execution_correctness_e2e_test.rs`（317〜408 行）は、選択肢を含む応答を完全一致で 8 件照合している。台詞の後の選択肢、選択肢の後の選択肢、選択肢の後の台詞（`\q[前,…]Ｂ通過Ａ再開`）の形があり、Requirement 2 の 1・2・5 で全部変わる。
  - `crates/pasta_lua/tests/scene_alias_search_test.rs:231`・`symbol_name_search_test.rs:237` は、`\q[…]` の部分一致である。変わらない。
  - `crates/pasta_sample_ghost/tests/tutorial_stages_test.rs` は、9 段目の `OnChoiceSelectEx` が 200 と空でない値を返すことだけを見る。選択肢を出す側の応答の中身は照合しない。出力の修正では落ちない。
  - マニュアルで選択肢の出力の形を書いている場所: `book/src/grammar/block-structure.md` の「選択肢行」（「…として、その行の位置に出力する」）、`book/src/lua/script-api.md` の `act:choice`、`book/src/lua/shiori-events.md` の `OnChoiceSelectEx`。`block-structure.md` と `script-api.md` は、スキルの参照資料の生成元である（`GENERATION_MAP`）。
  - 入門の 9 段目の章（`book/src/getting-started/09-choice.md`）は、辞書を逐語で写している。「成功の目印」は「続けて選択肢…が出る」で、行の配置には触れていない。
- **Implications**: 出力を直すと、`pasta_shiori` の結合テストの期待値 8 件、マニュアル 2〜3 か所、スキルの参照資料の再生成が付いてくる。ロードマップの同じウェーブの約束（「選択肢の出力とそのテスト」）の文面より広い（要件の未決事項 8）。

### 既存のゴーストへの影響
- **Context**: brief の Constraints「既にあるゴーストの選択肢の見え方が変わる」。
- **Sources Consulted**: このリポジトリの `.pasta` と `book/src/` の作例、`C:\home\maz\git\ghost_dev`（emo2）の `dic/menu.pasta`
- **Findings**:
  - このリポジトリで選択肢行を使う辞書は、hello-pasta の `09-choice.pasta` と、`pasta_shiori` のテスト用の辞書 2 枚（`choice_scope.pasta`・`characterization.pasta`）だけである。選択肢行の直前に作者が改行を書いたものは無い。
  - マニュアルとスキルの作例（`block-structure.md`・`shiori-events.md`・`authoring-patterns.md`）は、どれも「台詞の行 → 選択肢行を並べる → `!select`」の形で、改行を書いていない。作例どおりに書いた人は、9 段目と同じ見え方になっているはずである。
  - emo2 は選択肢行を使わない。台詞の中に `\q[…]\n\q[…]\_l[5em,2lh]\q[…]` と手で書いている。台詞の中の `\q` は `talk` のテキストとして通るので、選択肢のトークンの修正は届かない。
- **Implications**: 出力の修正で見え方が変わるのは、選択肢行か `act:choice` を使うゴーストだけである。知っている範囲では hello-pasta だけで、変わる向きは「直る」側である。作者が改行を自分で足していた場合だけ、空の行が増える（【仮定 A4】）。

## Requirement-to-Asset Map

| Requirement | 今ある資産 | ギャップ | 種別 |
| ----------- | ---------- | -------- | ---- |
| 1 原因の特定 | `pasta.log`（DEBUG で応答を記録）。UKADOC の `\q` の記述。SSP を操作するツール（SSP MCP） | 実機のさくらスクリプトを採っていない。2 つ目の選択肢だけが次の行に出る理由が決まっていない | Unknown |
| 2 選択肢を 1 行に 1 つ | `sakura_builder.lua` の `spot_has_text`・`emits_text`・`\c` でのリセット | 選択肢の前後に行の区切りを出す処理が無い。「吹き出しに選択肢を出した」状態が無い | Missing |
| 2 の 6（`act:choice` も同じ） | 選択肢行と `act:choice` は同じトークンを積む | 組み立ての側で直せば、両方に効く | —（追加なし） |
| 2 の 7（作者の改行と重ねる） | — | 重ねるだけなら追加なし。省くなら、直前の出力が改行で終わるかの判定が要る | Constraint（仮定しだい） |
| 3 変えない範囲 | 選択肢のタグの中身・エスケープ・`scope` のテスト（`sakura_builder_test.lua`）。選択肢の振り分けのテスト（`choice_select_test.lua`・`pasta_shiori` の結合テスト） | 完全一致の期待値 8 件（`call_execution_correctness_e2e_test.rs`）と 1 件（`shiori_act_test.lua:751`）は、配置の変更に合わせて直す | Constraint |
| 4 見本と入門の 9 段目 | `dic/09-choice.pasta`、`book/src/getting-started/09-choice.md`、`tutorial-check.mjs`、`tutorial_stages_test.rs` | 出力で直すなら、辞書と作例は変えない。「成功の目印」を書き足すかは未決 | —／Unknown |
| 5 マニュアルの記述 | `block-structure.md` の「選択肢行」、`script-api.md` の `act:choice`、`gen-skill-refs.mjs` | 行の配置の説明が無い。スキルの参照資料の再生成が要る（`.claude/skills/` への書き込みは、開発者の許可が要る） | Missing／Constraint |
| 6 実機での確認と記録 | SSP を `--ghost` で起動する手順（`hello-pasta-tutorial-stages`・`getting-started-story-guide` が使った）。SSP MCP | 9 段目の選択肢は、同名の 4 シーンから 1 つ選ばれるので、出るまでダブルクリックを繰り返す。時間切れの確認に 10 秒かかる | Constraint |

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
| ------ | ----------- | --------- | ------------------- | ----- |
| A: 組み立てで行の区切りを足す | `sakura_builder.lua` の状態機械に、選択肢の前（字または選択肢の後）と、選択肢の後の字の前の区切りを足す | 選択肢行と `act:choice` の両方に効く。吹き出しの状態（字を出したか・`\c`）を既に持っている。作者は何も書き足さない。brief の方針（出力で直す）に合う | `build` の複雑度が上がる。完全一致の期待値 9 件とマニュアルを直す。選択肢行を使う既存のゴーストの見え方が変わる | ロードマップの約束の範囲（`sakura_builder.lua` とそのテスト）に収まるのは本体だけ。期待値とマニュアルは未決事項 8 |
| B: 辞書で直す | `09-choice.pasta` の問いかけの終わりに `\n` を書く。章の作例と段階表をそろえる | エンジンを触らない。差分が小さい | brief の方針に反する（辞書の書き方で避ける手順を載せない）。選択肢と選択肢の間は直らない。作例どおりに書いた他の作者は直らない | 原因が出力でないと分かったときだけの退路 |
| C: A に加えて選択肢の形を `\__q` に変える | 選択肢を `\__q[ID,…]表示\__q` で出し、長い表示を自動で折り返させる | 1 行に収まらない選択肢も切れない | マニュアルに書いてある出力の形（`\![*]\q[…]`）が変わる。選択肢のテストをほぼ全部直す。表示の中の文字がさくらスクリプトとして読まれるので、エスケープの規則が変わる。`scene-anchor-link` と持ち場が重なる | 【仮定 A5】で本 spec の外。扱うなら別の spec が向く |
| D: バルーンを広げる | hello-pasta に幅の広いバルーンを付ける、または既定のバルーンを指定する | エンジンを触らない | brief の Out（バルーンの絵）。台詞の後ろに選択肢が続く並びは変わらず、長い問いかけで再発する。他のゴーストは直らない | 採らない見込み |

## Design Decisions

設計で決めること（ここでは決めない。選択肢だけを並べる）。

### Decision: 行の区切りを入れる場所
- **Context**: Requirement 2 の 1〜5。
- **Alternatives Considered**:
  1. 組み立て（`sakura_builder.lua`）— 吹き出しの状態を見て出す。
  2. `act:choice`（`act.lua`）で改行のトークンを積む — 吹き出しの状態を知らないので、最初の選択肢でも改行が出る。ロードマップの約束で `act.lua` は触れない。
  3. 生成（`scope_gen.rs`）で `act:newline()` を足す — `act:choice` を直接呼ぶ Lua には効かない。吹き出しの状態も知らない。
- **Selected Approach**: 未定（1 が有力）。
- **Follow-up**: `build` の複雑度を上げない形（補助関数への切り出し）にするかを決める。切り出すなら、先に今の出力を固定するテスト（特性化テスト）を置く。

### Decision: 区切りのタグと幅
- **Context**: Requirement 2 の 8。
- **Alternatives Considered**:
  1. `\n`（台詞の中の改行と同じ）。
  2. `\n[half]` や `\n[150]` のような幅付き。問いかけと選択肢の間だけ広くする。
- **Selected Approach**: 未定（要件の未決事項 7。下書きは 1）。

### Decision: 選択肢を「字」に数えるか
- **Context**: 選択肢だけを出した吹き出しへ話し手が戻るとき、段落の区切り（`\n[150]`）を出すか。今は選択肢を字に数えない。
- **Alternatives Considered**:
  1. 今のまま数えない。選択肢の後の台詞の区切り（Requirement 2 の 5）だけで分ける。
  2. 選択肢を字に数える。`pasta.toml` の `spot_newlines` の説明（「すでに台詞を出したスポット」）も変わる。
- **Selected Approach**: 未定。
- **Follow-up**: 保留中の `\n[150]` と、選択肢の後の `\n` が同じ場所に重なるときに、どちらを出すか。

### Decision: 話し手なしのグループの選択肢
- **Context**: `shiori_act_test.lua:751` の形（出力の先頭で、話し手なしのまま選択肢を出し、その後で話し手が話す）。話し手なしのグループはスポットが未解決（`nil`）で、後の話し手のスポット 0 と、状態機械の上では別の吹き出しになる。
- **Selected Approach**: 未定。実機では同じ吹き出し（スコープ 0）に出るので、【仮定 A3】を当てるなら、未解決のスポットを 0 と同じに扱うかを決める。

## 規模とリスク

- **Effort**: S（1〜3 日）。既にある状態機械への枝の追加と、テスト・マニュアルの追従である。brief の見積もり（2〜4 タスク）と合う。
- **Risk**: Low〜Medium。直す場所は 1 ファイルで、やり方は既存の改行の規則と同じ型である。一方で、選択肢行を使う既存のゴーストの見え方が変わること、実機でしか確かめられない点（行の配置・マーカーの位置）があること、同じファイルを `scene-anchor-link` も触ることが、Low に留まらない理由である。

## 設計フェーズへの申し送り

- **有力な進め方**: Option A。原因が実機で確かめられた後に進む。
- **決めること**: 上の Design Decisions の 4 件と、要件の未決事項 2〜4・7（どれも Requirement 2 の中身を左右する）。
- **Research Needed**:
  1. 実機のさくらスクリプトの採取（Requirement 1 の 1）。hello-pasta を SSP で動かし、`profile/pasta/logs/pasta.log` から 9 段目の応答を写す。コードから導いた形（上の Research Log）と合うかを見る。
  2. 2 つ目の選択肢だけが次の行に出る理由（Requirement 1 の 4）。SSP は、右端を越えた `\q` の次の文字を次の行へ送るとみられる。短い選択肢 2 つ（例:「はい」「いいえ」）を台詞の後ろに並べたさくらスクリプトを SSP に送り、同じ行に並ぶかを見ると確かめられる。
  3. 選択肢の前に `\n` を入れたときの見え方。マーカー（`\![*]`）が行の先頭に出るか、問いかけと選択肢の行間が詰まりすぎないか。
  4. `\n` で分けた 9 段目の 2 つの選択肢が、SSP の既定のバルーンで 1 行に収まるか（Requirement 4 の 2）。「おやつの話をする」は 8 文字で、問いかけ（20 文字）が 1 行に収まっているので、収まる見込みである。
  5. 選択肢行または `act:choice` を使って配布しているゴーストが、hello-pasta のほかにあるか（要件の未決事項 5）。

## Risks & Mitigations
- 実機で確かめる前に出力を直し、原因が別だった — Requirement 1 を最初のタスクに置く。SSP に手書きのさくらスクリプト（`\n` あり・なし）を送って見比べれば、エンジンを直す前に確かめられる。
- 完全一致の期待値（9 件）の直し漏れ — `pasta_shiori` と `pasta_lua` のテストを両方走らせる。期待値は、新しい規則を手で当てて書く（出力を写すだけにしない）。
- `scene-anchor-link` との衝突（同じ `sakura_builder.lua`） — ロードマップの順番どおり、本 spec を先に入れる。アンカーは台詞の中の `＠？` で、本 spec の選択肢行とは別のトークンである。
- スキルの参照資料の再生成が止まる — `.claude/skills/` への書き込みは開発者の許可が要る。マニュアルを直すタスクで、再生成の前に許可を求める。
- 作者が自分で足した改行と重なって空の行ができる — 変更の記録に書く（Requirement 6 の 4）。知っている範囲で、該当するゴーストは無い。

## References
- [UKADOC さくらスクリプトリスト `\q[タイトル,ID]`](https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html#_q_%E3%82%BF%E3%82%A4%E3%83%88%E3%83%AB%2CID_) — 自動改行は `\__q[]` を使うこと、という記述
- [UKADOC さくらスクリプトリスト `\__q[ID,...]`](https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html#___q_ID%2C..._) — 自動改行をする選択肢の範囲
- [UKADOC さくらスクリプトリスト `\![*]`](https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html#_!_*_) — マーカー
- `.kiro/specs/completed/sakura-script-newline/` — 話し手の切り替えの改行の状態機械（`spot_has_text`・`pending_break`）
- `.kiro/specs/completed/choice-definition-dsl/` — 選択肢行の元の要件（出力は `\![*]\q[…]` を行ごとに出す、までを定めた。行の配置は定めていない）
- `.kiro/steering/roadmap.md` — 同じウェーブの約束（本 spec が触るファイル）と、`scene-anchor-link`・`getting-started-screenshots` との順番
