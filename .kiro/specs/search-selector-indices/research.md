# Gap Analysis: search-selector-indices

## Summary
- **Feature**: `search-selector-indices`
- **Discovery Scope**: Extension（既存のテスト用セレクタの不具合修正。`pasta_core` の `random.rs` を中心に閉じる）
- **Key Findings**:
  - 原因は `MockRandomSelector::shuffle_usize` が何もしないこと（`crates/pasta_core/src/registry/random.rs` 97 行）。シーン表・単語表は巡の始まりごとに `shuffle_usize` を 1 回だけ呼び、その並びを先頭から順次消費するため、モックの `shuffle_usize` が「指定列に従った並べ替え」をすれば、Requirement 1 の意味（巡の中の順番の指定）を実現できる。単語は `random.rs` だけで足りる。
  - シーンだけ、一巡後の作り直しで `shuffle_usize` に渡るのが前の巡の並べ替え済みの配列で、`SceneId` の昇順も候補の並びの順と一致しない（確認済み）。2 巡目以降に指定列を当てはめ直す（1.5）には `scene_table.rs` の作り直しに数行の変更が要る見込み。
  - 「検索のたびに指定列の値の位置を返す（巡の中でも繰り返しうる）」意味（吸収元の旧記述 `set_word_selector(0, 1, 0)` →「1 番目、2 番目、1 番目」）は、巡の中で重複なく順次消費する表の構造と両立しない。実現するにはシーン表・単語表の選択経路を変える必要があり、現行マニュアルの例 `set_word_selector(0)` →「こんにちは、やあ、こんにちは」と既存テスト 1 件（`test_search_word_deterministic_with_mock_selector`）の期待が変わる。
  - 負の整数は現状 `parse_selector_args` の `i as usize` で巨大な値に化けるだけでエラーにならない。Requirement 2.3（エラーにする）を採るなら `pasta_lua` の `search/context.rs`（API 口）にも手が入る。
  - `MockRandomSelector` を使う既存の `pasta_core` テストは、シャッフル有効のものが指定列 `[0]` か `[]` だけで、提案の意味ではどちらも恒等の並びになるため、期待値は変わらない見込み（要実測）。

## Research Log

### 現行の選択経路（シーン）
- **Context**: Requirement 1.2・1.5・1.6 の実現位置の特定
- **Sources Consulted**: `crates/pasta_core/src/registry/scene_table.rs`（`select_from_cache` 248–300 行付近、`replace_selector` 410 行付近）
- **Findings**:
  - キャッシュのキーは `SceneCacheKey`（親のグローバル名・検索キー・整列したフィルタ）。キーごとに `CachedSelection { candidates, next_index, history }` を持つ。
  - `shuffle_usize` を呼ぶのは 2 か所: キャッシュ作成時（268 行付近）と、一巡して `next_index >= len` になったときの作り直し（286 行付近）。どちらも `shuffle_enabled` が真のときだけ。`shuffle_enabled` は Lua から変えられず既定は真。
  - `shuffle_usize` に渡すのは候補の並び（`collect_scene_candidates`／`iter_prefix` の順に集め、属性フィルタをかけた後）の `SceneId` の値（`usize`）の配列。作り直し時は、前の巡で並べ替えた後の配列を渡す（収集した順ではない）点に注意。
- **Implications**:
  - モックが「渡された配列の位置」で並べ替えると、2 巡目の作り直しでは前の巡の並び（＝並べ替え済み）を基準にしてしまう。Requirement 1.5（次の巡でも指定列を候補の並びに当てはめ直す）を満たすには、(a) モックが位置ではなく値の大小（`SceneId` の昇順）を基準にする、(b) 表側が作り直し時に収集順の配列を渡す、のどちらかが要る。
  - **確認済み**: (a) は成り立たない。`SceneId` は `SceneRegistry::all_scenes()` の登録順（`scene_registry.rs` 205 行）の 0 始まり添字だが、候補の並びは検索キーのバイト列の辞書順（同じキーの中だけ登録順）である。同名シーン 10 個以上（`メイン10` が `メイン2` より先）や、`挨拶朝` を `挨拶昼` より先に定義した場合などで、`SceneId` の昇順と候補の並びの順が食い違う。したがってシーンの 2 巡目以降を正しく当てはめるには (b)（`scene_table.rs` の作り直しで元の並びを渡す）か、モックが最初に受け取った並びを覚える工夫（状態を持つため 1.6 との整合に注意）が要る。

### 現行の選択経路（単語）
- **Context**: Requirement 1.1・1.5
- **Sources Consulted**: `crates/pasta_core/src/registry/word_table.rs`（`search_word` 185–232 行付近）
- **Findings**:
  - キャッシュのキーは `WordCacheKey`（スコープ名・検索キー）。キャッシュに残りがあればそれを返し、無ければ（初回・一巡後とも）`collect_word_candidates` で集め直し、`0..len` の添字配列を `shuffle_usize` で並べ替えて新しいキャッシュに置き換える。
  - 単語では `shuffle_usize` に渡すのが常に `0..len`（候補の並びの位置そのもの）なので、モックが値を位置として並べ替えれば巡ごとに正しく当てはめ直せる。
- **Implications**: 単語側は `random.rs` だけで Requirement 1.1・1.4・1.5 を満たせる。シーン側だけ上記の作り直し時の基準の問題がある。

### セレクタの差し替えと Lua API 口
- **Context**: Requirement 2.3・2.4・3.2〜3.4
- **Sources Consulted**: `crates/pasta_lua/src/search/context.rs`（`build_selector` 187 行、`set_scene_selector`・`set_word_selector` 198–212 行、`parse_selector_args` 215–223 行、UserData メソッド 252–273 行）
- **Findings**:
  - 引数ありなら `MockRandomSelector::new(seq)`、無しなら新しい `DefaultRandomSelector` を作り、表の `replace_selector` で差し替える。差し替えはキャッシュを消す（Requirement 3.4 は現行で満たす）。シーン表・単語表は別々のセレクタを持つ（3.2・3.3 は現行で満たす）。
  - `parse_selector_args` は `as_integer()` で整数でない値を `expected integer argument` のエラーにし（2.4 は現行で満たす）、`i as usize` で変換する。負の値は 2 の補数で巨大な `usize` になり、エラーにならない。
  - 1 つのモックを、その表のすべての検索キーが共有する。モックに「指定列のどこまで使ったか」の状態を持たせると、ある検索の結果が他の検索の回数に左右される（Requirement 1.6 に反する）。現行の `select_index` はこの状態（`index`）を持つが、表からは呼ばれていない。
- **Implications**: 1.6 を満たすには、モックの並べ替えを呼び出しごとに指定列の先頭から当てはめる（並べ替えについては状態を持たない）形にする。2.3 を採るなら `parse_selector_args` に負の値の検査を足す。2.1 の「無視」を採れば、負の値を検査しなくても巨大な値として自然に無視される（エラーにしない選択肢もある）。

### 既存テストへの影響
- **Context**: 互換性（Boundary Context の Adjacent expectations）
- **Sources Consulted**: `crates/pasta_core/tests/word_table_test.rs`、`crates/pasta_core/src/registry/scene_table_candidate_tests.rs`・`scene_table_resolve_filter_tests.rs`、`random.rs` の単体テスト、`crates/pasta_lua/tests/search/module_test.rs`、`crates/pasta_lua/tests/runtime/scene_test.rs`、`crates/pasta_lua/src/search/context.rs` のテスト
- **Findings**:
  - `pasta_core` のテストで `MockRandomSelector` を使うものは、シャッフル有効なら指定列 `[0]` か `[]`、それ以外の指定列（`[0,1,2,3]`・`[0..4]`）は `set_shuffle_enabled(false)` 付き。提案の意味では `[0]`・`[]` は恒等の並びになり、期待値は変わらない見込み。
  - `random.rs` の `test_mock_selector_shuffle_usize_is_noop`（指定列 `[0]`、`[3,1,4,1,5]`）は提案の意味でも結果は変わらないが、テスト名と意図（「並べ替えない」契約）が新しい意味と食い違うため置き換えが要る。
  - `pasta_lua`: `set_word_selector(0)` で 東京→大阪（module_test 309–330 行）、`set_word_selector(0, 1, 2)` で `\s[0]`・`\s[100]`・`\s[200]`（scene_test 365–408 行）。提案の意味ではどちらも現行どおり。`module_test` の `test_set_scene_selector`・`test_set_word_selector` はエラーにならないことしか見ておらず、整数が効くことを固定するテストは無い（Requirement 4 は Missing）。
- **Implications**: 提案の意味（Requirement 1）は後方互換。旧記述の意味（OPEN QUESTION 1 の (b)）を採ると、module_test の 東京→大阪 とマニュアルの `set_word_selector(0)` の例が変わる。

### マニュアル・スキル文書
- **Context**: Requirement 5
- **Sources Consulted**: `book/src/lua/modules/pasta-search.md` 124–156 行、`book/src/internals/registry-search.md` 38 行（構成要素の表の `RandomSelector` 行「モックはシャッフルしない」）・170 行（「渡した整数の値は検索表の選択に影響しない」）、`.claude/skills/pasta-lua-coding/references/pasta-search.md` 121–152 行・`references/testing-lint.md` 122–164 行、`book/tools/link-check-test.mjs` 359 行
- **Findings**:
  - 利用者章は「整数を 1 個以上渡すとシャッフルをやめ、候補を決まった順に 1 つずつ返す」とだけ書き、値の意味を書いていない（manual-ssot-authority の U29 で、争いのない部分だけを書いた結果）。並び順の規則（文字コード順・同名シーン 10 個以上）の記述はそのまま「候補の並び」の定義に使える。
  - 内部設計章は 38 行と 170 行の 2 か所で「使われない」ことを書く。
  - `link-check-test.mjs` が見出し `### set_scene_selector(...) / set_word_selector(...)` のアンカー `set_scene_selector--set_word_selector` を検査し、スキル文書がこのアンカーへリンクしている。見出しを変えるとリンク検査が落ちる。
  - `crates/pasta_core/README.md` 42 行は `MockRandomSelector` を「テスト用固定選択実装」と書く（意味は変えずに済む）。
- **Implications**: 見出しは変えずに本文を書き換える。スキル文書 2 本は spec 完了時の同期で揃える。

### Lua の数値と整数判定
- **Context**: Requirement 2.3・2.4
- **Sources Consulted**: `Cargo.toml`（mlua 0.11、`luajit52`）、`parse_selector_args`
- **Findings**: LuaJIT には整数の型が無く、mlua が整数値の数を `Value::Integer` として渡すかどうかで `1.0` の扱いが決まる。
- **Implications**: **Research Needed**（設計で実測）: LuaJIT 上で `1.0`・`2^53` 超・`-1` を渡したときの `as_integer()` の結果。マニュアルの「小数はエラー」の記述と照らして、`1.0` がエラーか整数扱いかを確定する。

## Requirement-to-Asset Map

| 要件 | 既存の資産 | ギャップ |
| ---- | ---------- | -------- |
| 1.1・1.3・1.4・1.5（単語） | `WordTable::search_word` が巡ごとに `0..len` を `shuffle_usize` に渡す | **Missing**: モックの `shuffle_usize` が並べ替えない |
| 1.2・1.4（シーン） | `SceneTable::select_from_cache` がキャッシュ作成時に `shuffle_usize` を呼ぶ | **Missing**: 同上 |
| 1.5（シーンの次の巡） | 作り直し時は並べ替え済みの配列を渡す | **Constraint**: 位置で並べ替えると前の巡の並びが基準になる。`SceneId` の昇順は候補の並びの順と一致しない（確認済み）ため、値の昇順で代用できない |
| 1.6 | 表ごとに 1 つのモックを全検索キーで共有 | **Constraint**: モックの並べ替えは呼び出しごとに指定列の先頭から当てはめる必要（状態を持たない） |
| 1.7・1.8 | 現行の結果（恒等の並び） | 提案の意味で維持できる（要実測） |
| 2.1・2.2・2.5 | 現行 `select_index` は `% len` で巡回（表からは未使用） | **Missing**: 範囲外・重複を無視する並べ替え |
| 2.3 | `parse_selector_args` の `i as usize` | **Missing**（採る場合）: 負の値の検査。`pasta_lua` の API 口に触れる |
| 2.4 | `expected integer argument` | 現行で満たす（1.0 の扱いは Research Needed） |
| 3.1 | `DefaultRandomSelector` | 変更不要（変えてはならない） |
| 3.2〜3.5 | 表ごとのセレクタ・`replace_selector` のキャッシュ消去・ランタイム全体で 1 つの `SearchContext` | 現行で満たす |
| 4.1〜4.5 | 「エラーにならない」ことだけを見るテスト | **Missing**: 整数が選択に効くことを固定するテスト（`pasta_core` の単体テストと、Lua からの結合テスト） |
| 5.1〜5.5 | 利用者章 124–156 行、内部設計章 38・170 行 | **Missing**: 整数の意味の記述。内部設計章の「使われない」の除去 |

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: `random.rs` だけで閉じる | モックの `shuffle_usize` が指定列に従って並べ替える（指定した位置を先に、残りを元の順に。範囲外・重複は無視。呼び出しごとに先頭から） | 変更が `random.rs` 1 ファイル（＋負の値を採るなら `context.rs` の数行）。Wave 2 の並走条件（`pasta_core` の他ファイルに触れない）を守る。後方互換 | シーンの次の巡で、渡される配列が前の巡の並び替え済みになる問題（1.5）。位置でなく値の昇順を基準にする等の工夫が要り、`SceneId` の昇順と候補の並びの順が一致しない場合に破綻する | brief.md が優先する方式 |
| B: 表が巡の作り直しで収集順を渡す | A に加え、`scene_table.rs` の作り直し（Phase 4）で収集順の配列（キャッシュ作成時に保存した元の並び）を `shuffle_usize` に渡す | 1.5 をシーンでも確実に満たす。本番の挙動はシャッフルの入力順が変わるだけで分布は同じ | `scene_table.rs` に触れる（brief の Out of Boundary は `scene_registry.rs`・`word_registry.rs` で、`scene_table.rs` は「触れる場合は Wave 2 の他 spec と重ならないことを確かめる」条件付き。roadmap 上 Wave 2 の他 spec は `pasta_core` を触らない）。本番の乱数消費は変わらないが、シャッフルの入力の並びが変わる | 3.1（本番の選択を変えない）の解釈を確認する必要 |
| C: 表が `select_index` を使う | 選択のたびに `select_index` を呼ぶ経路をモック用に追加し、「検索のたびに指定列の値の位置を返す」意味にする | 吸収元の旧記述（`0, 1, 0` →「1・2・1 番目」）の意味を実現できる | 巡の中で重複なく順次消費する構造と別の経路を表に足す。`scene_table.rs`・`word_table.rs` の両方に触れる。`set_*_selector(0)` が「常に先頭」に変わり、マニュアルの例と既存テスト 1 件の期待が変わる | OPEN QUESTION 1 で (b) を採る場合だけ |

## Design Decisions（設計フェーズへの申し送り。決定ではない）

### 整数の意味（OPEN QUESTION 1）
- **Context**: Requirement 1。brief.md は「候補の添字を順に使うのか、シャッフル後の並びを決めるのか」を要件フェーズで決めるとした。
- **Alternatives Considered**:
  1. 巡の中で返す順番の指定（要件ドラフトの前提）— Option A/B で実現。後方互換。
  2. 検索のたびに指定列の次の値の位置を返す — Option C。吸収元の旧記述に合うが互換性が崩れる。
- **Follow-up**: 要件ディスカッションで確定。

### シーンの次の巡の基準
- **Context**: Requirement 1.5（シーン）
- **Alternatives Considered**: (a) モックが値の昇順を基準に並べ替える（`random.rs` だけ）、(b) 表が作り直しで収集順を渡す（Option B）、(c) モックが最初に受け取った配列を値の集合ごとに覚える（状態を持つ。1.6 との整合に注意）
- **Follow-up**: (a) は `SceneId` の昇順と候補の並びの順が食い違うため不可（確認済み）。(b) が最小で確実。(b) は本番でもシャッフルの入力の並びが「前の巡の並び」から「元の並び」に変わるが、一様なシャッフルなので選ばれ方の分布と乱数の消費回数は変わらない。これが Requirement 3.1（本番の選択を変えない）に反しないかを設計で明記する。

### 要件ディスカッションの結果（2026-10-04）
- 整数の意味は「巡の中で返す順番の指定」に確定（Option C は不採用）。範囲外・重複は無視、負の値は API 口でエラー、0 始まり。
- シーンの 2 巡目以降も Requirement 1.5 を満たす（Option B を採る）。**設計への候補**: `select_from_cache` は呼び出しのたびに候補の並びの順の `filtered_ids` を受け取っているので、Phase 4 の作り直しを `cached.candidates` からではなく `filtered_ids` から行えば、元の並びを別に保存せずに済む（Phase 3 の作成と同じ処理に寄せられる）。乱数種固定のシーンのテスト（`test_resolve_scene_id_cycling_reshuffles`）は順序を固定しておらず、候補の集合だけを見ている。
- LuaJIT では `1.0` と `1` は同じ値で区別できない。設計では `1.5`・`-1`・大きな整数の `as_integer()` の結果だけを実測する。

## Implementation Complexity & Risk
- **Effort**: S（1–3 日）— 変更は `random.rs` の 1 メソッドとその単体テスト、結合テスト数件、マニュアル 2 章（＋スキル文書 2 本の同期）。表側に触れても数行。
- **Risk**: Low — 本番の選択（`DefaultRandomSelector`）に触れず、既存テストの期待は維持できる見込み。唯一の不確定要素はシーンの次の巡の基準（上記）で、Medium に上がるのは Option B/C で表に触れる場合。

## Recommendations for Design Phase
- 単語は Option A（`random.rs` だけ）で足りる。シーンは 2 巡目の作り直しで並べ替え済みの配列が渡るため、A だけでは Requirement 1.5 を満たせない。現実的なのは Option B（`scene_table.rs` の作り直しで元の並びを渡す数行の変更）。brief.md の「`random.rs` だけで閉じる方式を優先」とは、シーンについて部分的に食い違うため、要件ディスカッションで確認する（requirements.md の Open Question 7）。
- 範囲外の扱い（無視／剰余）・重複の扱い・負の値（エラー／無視）・0 始まりか 1 始まりかは要件ディスカッションの結果に従う。剰余を採る場合は、候補数の違う検索をまたいで同じ指定列を使ったときの結果をマニュアルに例で示す。
- テストは 2 層: `pasta_core` の `random.rs` 単体テスト（並べ替えの規則）と、Lua から `@pasta_search` を呼ぶ結合テスト（2 番目の候補・使い切り後・次の巡・既定への復帰）。`test_mock_selector_shuffle_usize_is_noop` は新しい意味のテストに置き換える。
- マニュアルは見出し（アンカー）を変えずに本文を書き換え、内部設計章 38・170 行を同じ意味に揃える。

### Research Needed
1. （解決済み）候補の並びの順と `SceneId` の昇順は一致しない。
2. LuaJIT＋mlua 0.11 で `1.0`・`-1`・大きな整数を渡したときの `as_integer()` の結果。

## References
- `.kiro/specs/completed/manual-ssot-authority/absorption-ledger.md` 685 行（U29。吸収元 runtime-api L101「選択インデックスのシーケンス（0 始まり）」・例 `set_word_selector(0, 1, 0)` →「1 番目、2 番目、1 番目」）
- `.kiro/specs/completed/pasta_search_module/requirements.md` Requirement 8（8.7「与えられたシーケンス順に確定的に選択」・8.8「末尾に達したら先頭にループ」）
- `.kiro/steering/roadmap.md` Wave 構成（Wave 2 の持ち場: `pasta_core` の `random.rs`）
