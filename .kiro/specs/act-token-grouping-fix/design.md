# Design Document: act-token-grouping-fix

## Overview

**Purpose**: ACT が積んだトークンをアクターごとのグループにまとめる段（`group_by_actor`）の 2 つの不具合を直し、LuaJIT で機能しない CT（`ct.lua`）を撤去する。

**Users**: Pasta DSL／Lua でトークを書くゴースト制作者と、pasta の保守者。

**Impact**: 変わる出力は次の 2 現象だけである。それ以外のさくらスクリプトは 1 バイトも変えない。

1. 1 回の出力の中で、最初の発言より前（と `clear_spot` の後、発言より前）に積んだ表示制御などが、捨てられずに、スコープ切替タグを付けずに、積んだ位置に出力される。
2. 発言の後に `clear_spot` が積まれたとき、その後の発言が前のグループに混ざらず、新しい立ち位置のスコープ切替タグの後に出力される。

### Goals

- `group_by_actor` の変更だけで要件 1・2 を満たす（組み立て `sakura_builder.lua` と外見 `appearance.lua` は変更しない）。
- CT を配布物・テスト・マニュアルから取り除く。
- 新しい挙動をグループ化の結果とさくらスクリプトのバイト比較で固定する。
- マニュアルを新しい規則に合わせ、スキル `references/` を再生成する。

### Non-Goals

- 先頭の表示制御を次の発言者に結び付ける仕掛け（保留・流し込み・フォールバック）。作者の意図は推測しない。
- `set_spot`（`spot` トークン）でグループを閉じること。`set_spot` の扱いと出力は変えない。
- 段落区切り改行（`sakura-script-newline`）・立ち絵の復旧（`actor-surface-restore`）の規則の変更。
- CT の修正・互換の層。
- グループ化の作り直し・抽象化（関数の切り出しも不要。後述の複雑度の実測による）。

## Boundary Commitments

### This Spec Owns

- `crates/pasta_lua/pasta_scripts/pasta/act.lua` の局所関数 `group_by_actor`（グループ化の規則）とそのコメント。
- `crates/pasta_lua/pasta_scripts/ct.lua`・`crates/pasta_lua/tests/lua_specs/ct_test.lua` の削除と、`tests/lua_specs/init.lua` の登録 1 行の削除。
- `crates/pasta_lua/pasta_scripts/pasta/shiori/init.lua` 17 行のコメントから `ct.lua` への言及を外すこと（コメント 1 行。挙動は変えない。**設計前提 DA2**）。
- グループ化とさくらスクリプトの組み立てを固定する Lua テストの追加（`act_grouping_test.lua`・`shiori_act_test.lua`）。
- マニュアル `book/src/lua/script-api.md`・`book/src/lua/patterns.md`・`book/src/internals/talk-output.md`・`book/src/internals/execution-model.md`・`book/src/internals/index.md` の該当箇所と、生成スキル `.claude/skills/pasta-lua-coding/references/script-api.md` の再生成。

### Out of Boundary

- `pasta/shiori/sakura_builder.lua`・`pasta/shiori/appearance.lua`（変更しない。変更が必要になったら設計に戻る）。
- `act.lua` の `merge_consecutive_talks`・`build`・`set_spot`・`clear_spot`・`init_scene`・`call` と、トークンを積む各メソッド。
- `actor.lua`・`global.lua`・`shiori/entry.lua`（`GLOBAL.close_ghost` を含む）。
- トランスパイラの生成コード（`clear_spot`・`set_spot` を積む位置）。
- `book/src/grammar/call-jump.md`（記述は既に正しい。変更しない。要件 5.6）。
- `release/hello-pasta/`、`.kiro/specs/review-improvement-loop/` などの過去の記録、ロードマップの更新（完了手順が行う）。

### Allowed Dependencies

- `group_by_actor` は、組み立て（`BUILDER.build`）が既に持つ次の契約に依存する。本仕様はこの契約を変えない。
  - 最上位の `type = "actor"` でアクターが `nil` のグループは、スコープ切替タグを出さず、内側を順に出力する。
  - 内側のトークンはアクター `nil` のとき `APPEARANCE.observe(appearance, nil, last_spot, 文字列)` で観測される（アクター未指定の生のさくらスクリプトと同じ規則）。
  - 最上位の `clear_spot` は `last_actor`・`last_spot`・`spot_has_text`・`pending_break` をリセットする。
- 上流 `dsl-codegen-runtime-safety`（`act.lua` を先に変更済み。main にマージ済みのコミット 34afab28 の上に積む）。

### Revalidation Triggers

- グループ化トークンの形（`type = "actor"` のグループ・最上位の `spot`・`clear_spot`・`raw_script`）を変える変更。
- `BUILDER.build` がアクター `nil` のグループを扱う規則、または `clear_spot` のリセットの範囲を変える変更。
- `APPEARANCE.observe` のアクター `nil` の扱い（全スポット不明化）を変える変更。
- 下流 `call-execution-correctness`（Wave 3）が `act.lua` を触るとき、本仕様のテスト（要件 6）が通ること。

## Architecture

### Existing Architecture Analysis

出力の経路は `act.token`（フラットな列）→ `group_by_actor` → `merge_consecutive_talks` → `BUILDER.build` → さくらスクリプトである。不具合は 2 つとも `group_by_actor` にある。

- 表示制御など（`surface`・`wait`・`newline`・`clear`・`choice`・`choice_timeout`）は、現在のグループが無いとき捨てられる。
- `clear_spot` は現在のグループを閉じない。そのため、同じ発言者の発言が `clear_spot` の前後にあると、後の発言が前のグループに入り、`clear_spot`（と続く `spot`）の処理がそのグループの出力の後になる。

「組み立ては変更しなくてよい」という見込みを実コードで確認した結果は次のとおりで、すべて成り立つ。

| 確認した点 | 実コード | 結果 |
| ---------- | -------- | ---- |
| アクター `nil` のグループの受け入れ | `sakura_builder.lua` 164 行 `if actor and last_actor ~= actor` | 切替タグ・復旧タグを出さず、内側を順に出力する。`last_actor` は変わらないため、次の発言者のグループで必ず切替タグが出る（要件 1.2・1.7） |
| 内側トークンの観測 | `emit_inner_token` → `APPEARANCE.observe(appearance, nil, last_spot, s)` → `observe_raw` | サーフェス変更・着せ替え・スコープ切替タグを含めば `state.spots = {}`（全スポット不明）にするだけで、アクターの記録は変えない（要件 1.3）。`\_w`・`\n`・`\c`・`\![*]\q[…]`・`\![set,choicetimeout,…]` は分類に当たらず、状態を変えない |
| 開いている `nil` グループと `raw_script` | `group_by_actor` はグループがあれば `raw_script` をその内側に入れる。組み立ては内側の `raw_script` を S4 で `text` のまま出力し、アクター `nil` として観測する | 最上位の `raw_script`（`buffer:put(token.text)`・`observe(appearance, nil, nil, text)`）とバイトも観測も同じ。`nil` グループが現れるのは出力の先頭か `clear_spot` の直後だけで、そこでは `last_spot == nil` であるため、観測の引数も一致する（要件 1.4・1.6） |
| `clear`（S4b）と `last_spot == nil` | 183 行 `if last_spot ~= nil then spot_has_text[last_spot] = false end` | `nil` ガードがあり、`\c` を出力して `pending_break` を偽にするだけである。`pending_break` はこの時点で必ず偽なので、段落区切りの判定は変わらない（要件 3.2） |
| `merge_consecutive_talks` | `type == "actor"` のグループを `actor = token.actor` で写す | アクター `nil` でもそのまま写る。表示制御などは `talk` ではないため結合の対象にならない |
| `clear_spot` でグループを閉じた後 | `BUILDER.build` S5 が状態をリセットする | 次の発言者のグループで `last_actor == nil` のため切替タグが出る（同じ発言者でも出る。要件 2.2）。`pending_break`・`spot_has_text` はリセット済みで、段落区切りの判定を持ち越さない（要件 2.3）。外見状態は `clear_spot` で消えないため、同じスポットの同じアクターなら復旧タグは出ない（要件 3.3） |
| luacheck の複雑度（上限 15） | 変更後の `group_by_actor` をスクラッチの写しで実測 | 13（変更前と同じ）。`spot or clear_spot` の 1 分岐を `spot`／`clear_spot` の 2 分岐に分け、else 分岐の `if` を置き換えるだけで、分岐の数は増えない。関数の切り出しは不要 |
| トークン 0 件 | `ACT_IMPL.build` が `group_by_actor` の前に `nil` を返す | 変更しない（要件 3.4） |

成り立たない点は無い。補足として、アクター `nil` の `talk`（Lua から `act:talk(nil, …)` を直接呼んだ場合だけ生じる。DSL からは生じない）は、開いている `nil` グループがあればそこに入る。現行でも `nil` の `talk` はアクター `nil` のグループを開くため、出力のバイトは「捨てられていた表示制御が前に出る」以外に変わらない。

### Architecture Pattern & Boundary Map

- **Selected pattern**: 既存の「グループ化 → 組み立て」の 2 段をそのまま使い、グループ化の規則だけを直す。新しいコンポーネント・トークン型・状態は足さない。
- **アクターの無いグループの表し方（要件の D1）**: アクター `nil` の `type = "actor"` グループにする。最上位に表示制御のトークンを直接置く案は、`BUILDER.build` の最上位が `spot`・`clear_spot`・`actor`・`raw_script` しか扱わないため、組み立ての変更が要る。`nil` グループは既存の経路で出力できる。
- **Existing patterns preserved**: `raw_script` のハイブリッド分類（グループがあれば内側、無ければ最上位）、`spot`・`clear_spot` を最上位に置くこと、`merge_consecutive_talks` の結合規則。
- **Steering compliance**: 編集するソースは Wave 2 の並走条件（`act.lua` のグループ化・`ct.lua` とテスト）に収まる。`shiori/init.lua` はコメント 1 行だけで、Wave 2 の他の spec は触らない。

### Technology Stack

| Layer | Choice / Version | Role in Feature | Notes |
|-------|------------------|-----------------|-------|
| Runtime | LuaJIT 2.1（mlua 経由）・Lua スクリプト `pasta_scripts/` | `group_by_actor` の修正、`ct.lua` の削除 | 新しい依存は無い |
| Test | `lua_test`（`tests/lua_specs/`）、`cargo test`、同梱 luacheck | グループ化・バイト比較のテスト、複雑度の検査 | 既存の流儀のまま |
| Docs | mdBook（`book/`）、`book/tools/gen-skill-refs.mjs`・`link-check.mjs` | マニュアルの更新とスキルの再生成 | 生成対象は `lua/script-api.md` だけ |

## File Structure Plan

### Modified Files

- `crates/pasta_lua/pasta_scripts/pasta/act.lua` — `group_by_actor` の 2 か所とコメント（詳細は Components）。
- `crates/pasta_lua/pasta_scripts/pasta/shiori/init.lua` — 17 行のコメントの「（3.47 ct.lua と同方針の既知負債）」を「（既知負債）」にする。
- `crates/pasta_lua/tests/lua_specs/init.lua` — `"ct_test"` の登録行を削除する。
- `crates/pasta_lua/tests/lua_specs/act_grouping_test.lua` — グループ化の結果のテストを追加する。
- `crates/pasta_lua/tests/lua_specs/shiori_act_test.lua` — さくらスクリプトのバイト比較のテストを追加する。
- `book/src/lua/script-api.md` — 172・408・424 行の「出力されない」を新しい規則に書き換える（要件 5.1）。
- `book/src/lua/patterns.md` — 61 行の説明を改める。作例は変えない（要件 5.2）。
- `book/src/internals/talk-output.md` — 103・106 行（グループ化の手順）、109 行（結果の列の説明）、229 行（不変条件）を書き換える（要件 5.3）。
- `book/src/internals/execution-model.md` — 20 行（扱う事項）、338–344 行（CT の節）、382 行（ソースの所在）から CT を削除する（要件 5.4）。
- `book/src/internals/index.md` — 84 行の表から `ct.lua` を削除する（要件 5.4。マニュアル CI のパス実在検査の対象）。
- `.claude/skills/pasta-lua-coding/references/script-api.md` — `node book/tools/gen-skill-refs.mjs` で再生成する（手で編集しない。要件 5.5）。

### Deleted Files

- `crates/pasta_lua/pasta_scripts/ct.lua`
- `crates/pasta_lua/tests/lua_specs/ct_test.lua`

埋め込み zip は `pasta_scripts/` のツリー全体を固めるため、ファイルの削除だけで配布物から消える（Rust 側にファイル一覧の登録は無い）。

## System Flows

`group_by_actor` の規則（変更は太字の 2 行）。

| トークン | 現在のグループあり | 現在のグループなし |
| -------- | ------------------ | ------------------ |
| `spot` | 最上位に置く。グループは閉じない（変更なし） | 最上位に置く（変更なし） |
| `clear_spot` | **最上位に置き、現在のグループを閉じる（`current_actor_token`・`current_actor` を `nil` に戻す）** | 最上位に置く（閉じるものが無く、結果は変更前と同じ） |
| `talk`・`sakura_script` | アクターが現在のグループと同じなら内側に入れ、違えば新しいグループを開く（変更なし） | 新しいグループを開く（変更なし） |
| `raw_script` | 内側に入れる（変更なし） | 最上位に置く（変更なし） |
| 表示制御など | 内側に入れる（変更なし） | **アクター `nil` のグループを開いて、その内側に入れる** |

`nil` グループが開いているときに発言が来ると、発言のアクターは `nil` ではないため新しいグループが開く。これで「先頭の表示制御 → 発言者の切替タグ → 本文」の順になる。

## Requirements Traceability

| Requirement | Summary | Components | 実現の要点 |
|-------------|---------|------------|------------|
| 1.1 | 先頭の表示制御などを捨てず、切替タグなしで積んだ順に出す | GroupByActor | グループなしの表示制御などを `nil` グループに入れる。組み立ては `nil` グループで切替タグを出さない |
| 1.2 | 後の発言者に結び付けず、切替タグは表示制御の後 | GroupByActor | 発言は `nil` グループに入らず新しいグループを開く |
| 1.3 | 先頭のサーフェス変更はアクター未指定として観測 | GroupByActor（既存の `emit_inner_token`） | アクター `nil` の観測は `observe_raw`。変更なしで満たす |
| 1.4 | 発言が無くても積んだ位置に出す | GroupByActor | `nil` グループは発言を待たない。`raw_script` は開いている `nil` グループの内側に入り、積んだ順のまま出る |
| 1.5 | 発言の後の表示制御は直前の発言者のスコープ | GroupByActor | 「現在のグループあり」の分岐は変更しない |
| 1.6 | `raw_script` は積んだ位置にそのまま | GroupByActor | `raw_script` の分岐は変更しない。内側と最上位で出力・観測が同じ |
| 1.7 | `yield` 直後の `wait(500)` ＋発言で `\_w[500]\p[0]…` | GroupByActor | 1.1・1.2 の帰結。ShioriActBytesTest で固定 |
| 2.1 | `clear_spot` の有無にかかわらず積んだ順 | GroupByActor | `clear_spot` でグループを閉じる |
| 2.2 | `clear_spot` の後の発言は新しい立ち位置の切替タグの後 | GroupByActor（既存の S5） | 閉じた後の発言は新しいグループになり、S5 のリセットで切替タグが出る |
| 2.3 | 段落区切りの判定を持ち越さない | GroupByActor（既存の S5） | S5 が積んだ順のとおりに効く |
| 2.4 | `clear_spot` の後、発言より前の表示制御は 1.x と同じ | GroupByActor | 閉じた後は「現在のグループなし」になり、同じ分岐を通る |
| 2.5 | `set_spot` の扱いは不変 | GroupByActor | `spot` の分岐は変更しない |
| 3.1 | 2 現象以外はバイト不変 | GroupByActor | 変更は「グループなしの表示制御など」と「グループありの `clear_spot`」の 2 分岐だけ |
| 3.2 | 段落区切り改行の規則は不変 | —（`sakura_builder.lua` 無変更） | |
| 3.3 | 立ち絵の復旧の規則は不変 | —（`appearance.lua` 無変更） | |
| 3.4 | トークン 0 件で `nil` | —（`ACT_IMPL.build` 無変更） | |
| 4.1 | 配布スクリプトに `ct` を含めない | CtRemoval | `ct.lua` を削除する |
| 4.2 | `require("ct")` は通常の `require` エラー | CtRemoval | 互換の層を置かない |
| 4.3 | CT のテストと登録を含めない | CtRemoval | `ct_test.lua` と `init.lua` の登録を削除する |
| 4.4 | 撤去後も他のテストの結果は不変 | CtRemoval | 利用者 0 を `git grep` で確認済み。全テストで確かめる |
| 5.1 | `script-api.md` の更新 | ManualUpdate | |
| 5.2 | `patterns.md` の説明の更新（作例は保つ） | ManualUpdate | |
| 5.3 | `talk-output.md` の手順と不変条件の更新 | ManualUpdate | |
| 5.4 | `execution-model.md`・`internals/index.md` から CT を削除 | ManualUpdate | |
| 5.5 | スキルの再生成と 2 つの検査 | ManualUpdate | `gen-skill-refs.mjs`（生成・`--check`）、`link-check.mjs` |
| 5.6 | `grammar/call-jump.md` の記述を保つ | ManualUpdate | 変更しない。ShioriActBytesTest の `close_ghost` のテストで裏付ける |
| 6.1 | 先頭の表示制御のバイト比較 | ShioriActBytesTest | |
| 6.2 | 発言の無い出力・`raw_script` 混在 | ShioriActBytesTest | |
| 6.3 | 先頭の `surface` の観測と復旧 | ShioriActBytesTest | |
| 6.4 | `clear_spot` を挟む出力・`set_spot` 単独 | ActGroupingTest・ShioriActBytesTest | |
| 6.5 | 既存の期待値を変えずに通す | 全テスト | 既存のテストファイルの期待値は編集しない |

## Components and Interfaces

| Component | Layer | Intent | Req Coverage | Key Dependencies | Contracts |
|-----------|-------|--------|--------------|------------------|-----------|
| GroupByActor | Lua ランタイム（`pasta/act.lua`） | トークン列をグループ化トークンの列にする | 1.1–1.7, 2.1–2.5, 3.1 | `BUILDER.build` の既存の契約（P0） | State |
| CtRemoval | Lua ランタイム・テスト | CT の削除 | 4.1–4.4 | なし | — |
| ActGroupingTest | テスト（`act_grouping_test.lua`） | グループ化の結果を固定する | 6.4 | `pasta.act` | — |
| ShioriActBytesTest | テスト（`shiori_act_test.lua`） | さくらスクリプトをバイト比較で固定する | 6.1–6.4, 5.6 | `pasta.shiori.act`・`STORE` | — |
| ManualUpdate | マニュアル・生成スキル | 記述を新しい規則に合わせる | 5.1–5.6 | `book/tools/` | — |

### Lua ランタイム

#### GroupByActor

| Field | Detail |
|-------|--------|
| Intent | フラットなトークン列を、積んだ順を保ったままグループ化する |
| Requirements | 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1 |

**Responsibilities & Constraints**

- 変更は `group_by_actor` の次の 2 か所だけである。関数の署名・戻り値の形・他の分岐は変えない。

```lua
-- (1) spot と clear_spot の分岐を分け、clear_spot だけグループを閉じる
if t == "spot" then
    table.insert(result, token)
elseif t == "clear_spot" then
    table.insert(result, token)
    current_actor_token = nil
    current_actor = nil
elseif t == "talk" or t == "sakura_script" then
    -- 変更なし

-- (2) 表示制御など: グループが無ければアクター nil のグループを開く
else
    if not current_actor_token then
        current_actor_token = { type = "actor", actor = nil, tokens = {} }
        table.insert(result, current_actor_token)
        current_actor = nil
    end
    table.insert(current_actor_token.tokens, token)
end
```

- 「捨てる」と書いた既存のコメントを、新しい規則（アクター未指定として積んだ位置に出す）の説明に置き換える。
- 不変条件:
  - 結果の列を最上位から内側へ順にたどると、入力のトークンが積んだ順のまま、過不足なく現れる。
  - アクター `nil` のグループが開くのは、出力の先頭か `clear_spot` の後で、発言より前に表示制御などが積まれたときだけである（Lua から `act:talk(nil, …)` を呼んだ場合の既存の挙動を除く）。
  - `spot` トークンはグループを閉じない。

**Contracts**: State [x]

##### State Management

- 状態は関数内の局所変数 `current_actor_token`・`current_actor` だけで、ビルドごとに `nil` から始まる。新しい状態は足さない。

**Implementation Notes**

- Validation: luacheck（`Total: 0 warnings / 0 errors`。`group_by_actor` の複雑度は 13 の見込み）、`cargo test -p pasta_lua`、`cargo test --workspace`（事前に環境変数 `NoDefaultCurrentDirectoryInExePath` を外す）。
- Risks: Rust 側の E2E（`crates/pasta_lua/tests/shiori/`・`crates/pasta_shiori`・`crates/pasta_sample_ghost`）に、2 現象に当たるシーンを固定した期待値があるかは設計フェーズで実行して確かめていない（設計フェーズではソースを変更しないため）。実装の最初に全テストを走らせ、差分が出た期待値が 2 現象に当たるものだけであることを 1 件ずつ確かめる。2 現象に当たらない差分が出たら設計に戻る。

#### CtRemoval

| Field | Detail |
|-------|--------|
| Intent | CT のモジュール・テスト・登録・コメントの言及を取り除く |
| Requirements | 4.1, 4.2, 4.3, 4.4 |

**Implementation Notes**

- `ct.lua`・`ct_test.lua` を削除し、`tests/lua_specs/init.lua` の `"ct_test"` の行を削除する。`shiori/init.lua` のコメントから `ct.lua` の言及を外す。
- Validation: `git grep -n "ct\.lua\|ct_test\|require(\"ct\")"` で、残る言及が `.kiro/specs/` の過去の記録と `.kiro/steering/roadmap.md` だけであることを確かめる。`require("ct")` がエラーになること（要件 4.2）はファイルが無いことで満たすため、専用のテストは足さない（**設計前提 DA3**）。

### マニュアル

#### ManualUpdate

| Field | Detail |
|-------|--------|
| Intent | マニュアルの記述を新しい規則に合わせ、スキルを再生成する |
| Requirements | 5.1, 5.2, 5.3, 5.4, 5.5, 5.6 |

**書き換える内容**

- `lua/script-api.md` 表示制御の節: 「出力されない」を削除し、「1 回の出力の中で、まだ発言を積んでいないうちに積んだ表示制御は、どのアクターにも結び付かず、スコープ切替タグを付けずに、積んだ位置にそのまま出力される。発言者の表情は、発言の中か発言の後に書く」に改める。`choice`・`choice_timeout` の節（408・424 行）も同じ規則を指すように改める。
- `lua/patterns.md` 61 行: 「出力されない」を「誰のスコープにも付かない」に改める。作例（`yield` の直後に先に `talk` を積む形）は変えない。回避のための別の書き方は足さない。
- `internals/talk-output.md`:
  - グループ化の手順: `spot` は「そのまま結果に置く。グループは閉じない」、`clear_spot` は「そのまま結果に置き、現在のグループを閉じる」、表示制御などは「グループがあればその `tokens` に入れ、無ければアクター `nil` のグループを開いて入れる」。
  - 不変条件（229 行）: 「出力に含めない」「`clear_spot` はグループを閉じない」を削除し、先頭の表示制御はアクター未指定として積んだ位置に出ること、`clear_spot` がグループを閉じること、`spot` はグループを閉じないこととそれで出力が変わらない理由（`spot` をまたいで同じグループに入るのは切り替えを伴わない同じ発言者の発言だけである）を書く。
- `internals/execution-model.md`・`internals/index.md`: CT の記述を削除する。
- スキル: `node book/tools/gen-skill-refs.mjs` で再生成し、`node book/tools/gen-skill-refs.mjs --check` と `node book/tools/link-check.mjs` を通す。`execution-model.md` の CT の節の見出しを指すリンクが他の章に無いことは、リンク検証で確かめる。

## Data Models

グループ化トークンの形は変えない。変わるのは、アクター `nil` のグループが現れる場面が増えることだけである。

```lua
-- group_by_actor / ACT_IMPL.build の戻り値の要素（変更なし）
{ type = "actor", actor = Actor|nil, tokens = InnerToken[] }  -- actor == nil はアクター未指定
{ type = "spot", actor = Actor, spot = integer }
{ type = "clear_spot" }
{ type = "raw_script", text = string }                        -- グループの外の raw_script
```

例（`act:wait(500)`、`act:talk(さくら, "A")` の順）:

```lua
{
    { type = "actor", actor = nil,    tokens = { { type = "wait", ms = 500 } } },
    { type = "actor", actor = さくら, tokens = { { type = "talk", actor = さくら, text = "A" } } },
}
```

## Error Handling

新しいエラー経路は無い。`group_by_actor` は入力を検査せず、未知の型のトークンは従来どおり表示制御などと同じ分岐を通り、組み立てが空文字列として扱う。`require("ct")` は Lua 標準の `module 'ct' not found` のエラーになる。

## Testing Strategy

期待値は、既定サーフェスを持たないアクター（さくら＝立ち位置 0、うにゅう＝立ち位置 1）と、テストごとに初期化した `STORE.actor_spots`・`STORE.appearance` を前提にする。本文は句読点を含めない（ウェイト挿入を避ける）。

### グループ化の結果（`act_grouping_test.lua` に追加）

1. `wait` → `talk(さくら)` で、結果が「アクター `nil` のグループ（`wait`）」「さくらのグループ（`talk`）」の 2 件になる（1.1・1.2）。
2. `talk(さくら, A)` → `clear_spot` → `spot(さくら)` → `talk(さくら, B)` で、結果が「さくらのグループ（A）」「`clear_spot`」「`spot`」「さくらのグループ（B）」の 4 件になる（2.1・2.2）。
3. `talk(さくら, A)` → `clear_spot` → `wait` → `talk(さくら, B)` で、`wait` がアクター `nil` のグループに入る（2.4）。
4. `talk(さくら, A)` → `set_spot(さくら, 1)` → `talk(さくら, B)` で、結果が「さくらのグループ（`talk` 1 件、本文 AB）」「`spot`」の 2 件のままである（2.5。変更前と同じ）。

### さくらスクリプトのバイト比較（`shiori_act_test.lua` に追加）

| # | 積む順 | 期待する出力 | 要件 |
|---|--------|--------------|------|
| 1 | `talk(さくら, "X")`・`build()`（`yield` が呼ぶ区切り）の後に `wait(500)`・`talk(さくら, "A")` | 2 回目の `build()` が `\_w[500]\p[0]A\e` | 1.7, 6.1 |
| 2 | シーンの冒頭で `surface(5)`・`wait(500)`・`newline()`・`clear()`・`choice("t", "d")`・`choice_timeout(30)`・`talk(さくら, "A")` | `\s[5]\_w[500]\n\c\![*]\q[d,t]\![set,choicetimeout,30000]\p[0]A\e` | 1.1, 1.2, 6.1 |
| 3 | `wait(1000)` だけ | `\_w[1000]\e` | 1.4, 6.2 |
| 4 | `GLOBAL.close_ghost(act, 1500)`（`＞ゴースト終了（1500）`） | `\_w[1500]\-\e` | 1.4, 5.6, 6.2 |
| 5 | `raw_script("\![x]")`・`wait(100)`・`raw_script("\![y]")`・`talk(さくら, "A")` | `\![x]\_w[100]\![y]\p[0]A\e` | 1.6, 6.2 |
| 6 | 既定サーフェス 0 を持つさくらで `surface(5)`・`talk(さくら, "A")` | `\s[5]\p[0]\s[0]A\e`。`STORE.appearance.actors["さくら"]` にサーフェス 5 が記録されない | 1.3, 6.3 |
| 7 | 6 の続きの出力で `surface(5)`・`talk(さくら, "B")`（同じアクターが同じスポットで続く） | `\s[5]\p[0]B\e`（継続のため復旧タグは出ない。既存の規則どおり） | 1.3, 6.3 |
| 8 | `talk(さくら, "A")`・`clear_spot`・`set_spot(さくら, 0)`・`set_spot(うにゅう, 1)`・`talk(さくら, "B")` | `\p[0]A\p[0]B\e`（変更前は `\p[0]AB\e`） | 2.2, 6.4 |
| 9 | `talk(さくら, "A")`・`clear_spot`・`set_spot(うにゅう, 0)`・`set_spot(さくら, 1)`・`talk(さくら, "B")`（立ち位置の入れ替え） | `\p[0]A\p[1]B\e`（変更前は `\p[0]AB\e`） | 2.2, 6.4 |
| 10 | `talk(さくら, "A")`・`clear_spot`・`set_spot`（同じ配置）・`talk(うにゅう, "B")`（発言者が変わる） | `\p[0]A\p[1]B\e`（変更前と同じ） | 2.1, 6.4 |
| 11 | `talk(さくら, "A")`・`talk(うにゅう, "B")`・`talk(さくら, "C")`・`clear_spot`・`set_spot`（同じ配置）・`talk(さくら, "D")` | `\p[0]A\p[1]B\p[0]\n[150]C\p[0]D\e`（D の前に `\n[150]` が出ない） | 2.3, 6.4 |
| 12 | `talk(さくら, "A")`・`clear_spot`・`set_spot`（同じ配置）・`wait(300)`・`talk(さくら, "B")` | `\p[0]A\_w[300]\p[0]B\e` | 2.4, 6.4 |
| 13 | `talk(さくら, "A")`・`set_spot(さくら, 1)`・`talk(さくら, "B")` | `\p[0]AB\e`（変更前と同じ） | 2.5, 6.4 |

期待値は実装時に実際の出力と突き合わせる。食い違った場合は、期待値を出力に合わせて直すのではなく、本設計の規則のどこと食い違うかを確かめる。

### 回帰の確認

- 既存のテストファイルの期待値を編集せずに、`cargo test --workspace` が通ること（3.1–3.4, 4.4, 6.5）。`pasta_lua` のテストが `sample.generated.lua` を改行コードだけ書き換えた場合は `git checkout --` で戻す。
- luacheck が `Total: 0 warnings / 0 errors` であること。
- `cargo clippy --workspace --all-targets`（Rust は変更しないが、関門として実行する）。
- `node book/tools/gen-skill-refs.mjs --check`・`node book/tools/link-check.mjs`（5.5）。

## 設計前提と未決事項

| ID | 状態 | 内容 |
| -- | ---- | ---- |
| DA1 | 確定（要件 D1） | アクターの無いグループはアクター `nil` の `type = "actor"` グループで表す。複雑度は 13 のままで、関数の切り出しは要らない |
| DA2 | 設計前提（要件 D2） | `pasta/shiori/init.lua` のコメントから `ct.lua` の言及を外す（1 行。挙動は変えない）。削除したファイルを指すコメントを残さないためで、Wave 2 の他の spec はこのファイルを触らない |
| DA3 | 設計前提 | `require("ct")` がエラーになることの専用テストは足さない。ファイルが無いことで満たし、`git grep` で確かめる |
| OQ1 | 未決（確認） | 発言の後で、同じ配置の `％` 行を持つシーンを呼ぶと、同じ発言者の台詞が `\p[0]A\p[0]B` のように切替タグを挟んで続き、台詞の結合（`merge_consecutive_talks`）が切れる。後処理（ウェイト挿入・budoux の行幅）は B で数え直しになり、A と B の間に段落区切りの改行は出ない（要件 2.2・2.3 のとおり）。DSL の想定された書き方で生じる。下記の例を参照 |

OQ1 を生じる Pasta DSL の例（想定された書き方）:

```pasta
＊挨拶
　％さくら、うにゅう
　さくら：こんにちは
　＞続き

＊続き
　％さくら、うにゅう
　さくら：続きだよ
```

変更前の出力は `\p[0]こんにちは続きだよ\e`、変更後は `\p[0]こんにちは\p[0]続きだよ\e` である。見た目は同じバルーンに続けて表示され、変わるのは budoux の行幅の数え直しだけである。要件 2.2・2.3 が決めた挙動であり、設計はこのまま採る。確認したいのは「この副作用を受け入れてよいか」の 1 点である。
