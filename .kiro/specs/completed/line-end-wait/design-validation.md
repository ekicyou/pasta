# 設計の検証: line-end-wait

- 日付: 2026-11-26
- 対象: `design.md`（`research.md` 9〜12 章の決定を含む）と `requirements.md`（要件 1〜10）
- 照らしたコード: `crates/pasta_lua/src/sakura_script/{mod.rs,wait_inserter.rs,tokenizer.rs,line_breaker.rs}`・`pasta_scripts/pasta/act.lua`・`pasta_scripts/pasta/shiori/sakura_builder.lua`・`src/loader/config/{mod.rs,sections.rs}`・`crates/pasta_check/src/validate/{config.rs,report.rs}`・`crates/pasta_dsl/src/parser/grammar.pest`・`tests/lua_specs/line_newlines_test.lua`
- 判定: **GO**

## 設計の要約

行末の位置と直後の作者の `\_w[…]` は Lua の組み立てが求め、待ちの規則は Rust の `insert_waits` が持つ形（案 C）で、見える字の定義と待ちの規則がどちらも 1 か所のまま保たれている。行末のキーが無いときの出力（要件 6.1）は、`line_end.any()` が偽なら今の経路だけを通ることと、`join_lines` の早い戻りの 2 つで、作りとして守られている。実装に入れる。ただし、位置の写し替えと `insert_waits` の改修の形の 2 点は、タスクにする前に決めておく。

## コードで確かめたこと

| 設計の主張 | 確かめた結果 |
|---|---|
| `seams` を読むのは `join_lines` だけ | 正しい。`git grep seams` で Rust にも他の Lua にも読む所は無い。`act.lua` の `add_seams`・`take_seam` と `sakura_builder.lua` の `join_lines` だけ |
| 継ぎ目をいつも付けても、`line_newlines` が nil・0 なら出力は変わらない | 正しい。`join_lines` は `ratio == nil or ratio == 0` なら `inner.text` をそのまま返す。直す Lua のテストは `line_newlines_test.lua` の 3 つ（50・57・96〜100 行目の表の丸ごとの比較）だけ |
| `act:line()` はアクション行・継続行だけに出る（3.9） | 正しい。`code_gen/element_gen.rs` の `generate_line_marker` だけが出す |
| `talk_to_script` の呼び出しは組み立ての 1 か所だけ | 正しい。ほかは `scriptlibs/lua_test/mocks.lua` のモック（2 つ目までしか見ない）だけ |
| 辞書に直接書いた `\_w[500]` は別の `sakura_script` のトークンになり、`＠単語` の `\_w` は talk の中 | 正しい。`group_by_actor` は raw_script もまとまりの中に入れる。`merge_consecutive_talks` がつなぐのは talk だけ |
| 句読点の並びはタグで切れる（トークンをまたいで並びが続かない） | 正しい。`insert_waits` は `SakuraScript` で `flush_pending` を呼び、`talk_to_script` は呼び出しの終わりで吐き出す。並びが 2 回の呼び出しをまたぐことは無い |
| `[actor.*.font]` の前例（`pasta.dll` の読み込みのエラーと `pasta_check` の写し） | 正しい。`PastaConfig::parse` の `actor_font::apply`、`pasta_check` の `read_actor_fonts`、`pasta_core::FONT_ATTRIBUTES` |
| Lua の見える字と Rust の待ちを持てる字の境目 | 整合している。Lua の `read_unit` はタグ以外の字を `\` か継ぎ目までの「並び」で 1 単位にするが、その終わりは Rust の `next_cluster`（2 つ目以降のコードポイントの `\` で切る）のクラスタの終わりと一致する。エスケープ・囲み・`\_u` などは Lua では見える字、Rust では `SakuraScript` なので無視（4.6）。単独の `\` は Lua で見える字、Rust で `General` だが、`append_with_wait` と同じく後ろに待ちを入れない規則で揃う |

## 重大な問題（3 つ）

### 🔴 重大な問題 1: `join_lines` の位置の写し替えが、2 つの読み方の単位の境目が一致することに頼っている

- **問題**: 設計は「行末は `read_unit` の単位の終わりにあるので、`join_lines` の同じ単位の切り方で必ず当たる」としている。ただ、`find_line_ends` と `join_lines` は別々のループで単位を読む（継ぎ目の `stop`、「単位の途中に落ちた継ぎ目はその単位の後で達したとみなす」の扱いを 2 か所に書くことになる）。境目が 1 つずれると写し替えが外れ、Rust は「トークンの終わりに当たらない行末」として黙って無視する。
- **影響**: `line_newlines` を書いたゴーストだけで、行末の待ちが黙って行の途中の値に戻る。エラーにならず、`line_newlines` の無い e2e では見つからない。ghost_dev は縦書きで `line_newlines` を使う見込みが高く、いちばん効いてほしい所で外れる。
- **推奨**: 写し替えを単位の一致に頼らない形にする。`join_lines` が差し込む片は、どれも単位の頭（`reach_seam` は継ぎ目、`step_unit` は見える単位の直前）にしか入らないので、「元の位置 `pos` までに差し込んだバイト数」を足せば、どの位置でも写せる（`new_pos = pos + 差し込んだ片の長さの和（差し込んだ位置 ≤ pos のもの）`）。あわせて、行末のキーと `line_newlines = 1.8` と budoux の幅を持つアクターを一緒にした e2e を足す（4.5 の `。\_w[…]\n[180]`、5.1 の辞書に直接書いた道と `＠単語` の道、継続行の頭の `＠左`）。
- **関連する要件**: 3.1・3.8・4.5・5.1・6.2
- **design.md の場所**: Components and Interfaces › LineEndFinder（`join_lines` の契約と「必ず当たる」の注記）、Testing Strategy › 出力の比較の結合テスト

### 🔴 重大な問題 2: `insert_waits` に後戻りの判定を足すやり方が決まっていない

- **問題**: 今の `insert_waits` は、ふつうの字・リーダーの待ちをその場で出す流れの読み方である。行末の規則は、4.4（`す」` の `す` の待ちを、後ろの `」` が行末と分かってから消す）、並びを行末で切る、同じ k の行末を 1 つにする、`author_wait` で自動の待ちを入れない、を含む。どれも先を見ないと決まらない。設計は「行末のトークンの添字を先に求め、ループの中では添字の比べだけにする」と書くだけで、ループの中の分岐の形は決めていない。
- **影響**: 今の分岐（`General`・`Leader`・並び）の中に行末の分岐を混ぜると、行末のキーが無いときの道（6.1）と、効かない行末の道（4.7・5.3）を壊しやすい。いちばん危ない規則が、テストしにくい形になる。
- **推奨**: 先に 1 回読む関数（例: `plan_line_ends(tokens, line_ends, values) -> Vec<LineEndPlan>`。トークンごとに「行の途中の待ちを入れない」「ここで並びを吐き出す（値・自動の待ちなし）」を持つ）に規則 1〜5 をまとめる。`insert_waits` は、計画が空なら今のコードそのものを通り、空でなければ添字で計画を引くだけにする。計画は `line_end.any()` が真で、行末が 1 つ以上あるときだけ作る。単体テストは計画の関数に直接当てる（4.1〜4.7・5.1〜5.3 の表）。
- **関連する要件**: 4.1〜4.7・5.1〜5.3・6.1・6.3
- **design.md の場所**: Components and Interfaces › WaitRules（行末の扱いの規則・Implementation Notes の Risks）

### 🔴 重大な問題 3: 行末の空白をどう扱うかが書かれていない

- **問題**: DSL の `talk`（`grammar.pest` の `talk_word = !(at | dollar | sakura_marker | eol) ~ ANY`）は行の終わりの空白を落とさない。Lua の `has_text`・`read_unit` は空白も見える字に数える。だから `　ぱすた：雨？　`（行の終わりに全角の空白）の行末は「　」（Rust では `General`）になる。`script_wait_strong_eol` だけを書いたゴーストでは、この行の「？」は行の途中の値のまま黙って残る。`script_wait_normal_eol` を書けば、空白の後ろに行末の待ちが入る。
- **影響**: エディタで見えない空白 1 つで、その行だけ行末の待ちが効かない。作者には理由が分からない。ghost_dev には今は該当する行が無い（grep で 0 件）が、作者が増えれば起こる。
- **推奨**: 「想定外の書き方に忖度しない」の方針に合わせ、救う規則は足さない。そのかわり、空白も字として行末になることを `design.md` の行末の判定と、マニュアルの行末の定義（9.2）に 1 行で書き、e2e に例を 1 つ（`雨？　` の行末は「　」）足して、今の振る舞いを固める。
- **関連する要件**: 3.1・9.2・10.2
- **design.md の場所**: Components and Interfaces › LineEndFinder（Responsibilities & Constraints）、マニュアルとスキル › Docs

## 設計の良い所

1. **持ち場の分け方と 6.1 の守り方**: Lua は「どこが行末か・その直後に作者の `\_w` があるか」だけ、Rust は「その行末に何を入れるか」だけを持ち、受け渡しは位置と真偽だけ。見える字の定義（`read_unit`）と待ちの規則（`insert_waits`）がどちらも 1 か所のままで、行末のキーが無いときは新しい道を通らないことを、コード（`seams` を読むのは `join_lines` だけ、モックは 4 つ目の引数を見ない）で確かめられる。
2. **根拠のある調べと抜けの無い対応表**: 辞書に直接書いた `\_w` が別のトークンになること、mlua 0.12 の `get::<i64>` の読み方、budoux がタグを幅 0 で運ぶことを、コードで確かめてから決めている。要件 1.1〜10.3 がすべて部品と契約に対応し、出力の例の表がそのままテストの期待になる。

## 補足（軽微。タスクの段で拾う）

- 変更するファイルの一覧に無い所: `tests/sakura_script/edge_case_test.rs`（`WaitValues` の構造体の書き方と、2 つの引数の `insert_waits` を 5 か所で呼ぶ）、`src/sakura_script/line_breaker.rs` の 835〜844 行目のテストの補助（同じ）、`mod.rs` の `resolve_wait_values` の `WaitValues { … }`。`TalkConfig` の `Default` は手で書いた impl なので、5 つの欄の `None` を足す。
- 4 つ目の引数の形が違うときに設計どおりの文言のエラーを出すには、mlua の引数の型を `Option<Table>` でなく `Value` で受ける（`Option<Table>` だと mlua の決まった文言になる）。
- 直接の呼び出しの既定で `\_w[…]` を見分けるとき、Rust でも名前が `_w` で引数のあるタグだけにする（`\__w[…]`・`\w5` を数えない）。Lua の `author_wait` と同じ条件にし、適合テストで比べる。

## 最終の判定

- **判定**: GO
- **理由**: 既存の作り（継ぎ目・`insert_waits`・`actor_font` の前例・`pasta_check` の写し）に沿っていて、要件はすべて部品と契約に対応している。6.1 も作りで守られている。重大な問題 3 つは、どれも設計の向きを変えない局所的な決め事で、設計のディスカッションで閉じられる。
- **次の段**:
  1. 設計のディスカッション（`/kiro-design-discussion line-end-wait`）で、重大な問題 1〜3 と、`design.md` の議題 1〜7 を閉じる。
  2. 決めたことを `design.md` に入れ、`/kiro-spec-tasks line-end-wait` でタスクを作る。特性化テストを最初のタスクにする（10.1）。`.claude/skills` の書き込みは、開発者の許可が要る段として分ける。
