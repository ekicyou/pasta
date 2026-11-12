# Brief: scene-anchor-link

## Problem

ゴーストの作者は、台詞の中の語を Wikipedia のリンクのようにしたい。クリックすると、その語のシーンへ飛ぶ。たとえば「今日は五月晴れだね」の「五月晴れ」を押すと、シーン `＊五月晴れ` が始まる。

今の DSL で書けるクリック部品は、行頭の選択肢の行（`　＠？挨拶`）だけである。台詞の途中に `＠？` を書くと、構文エラーになる。さくらスクリプトの `\_a` を手で書けば表示はできる。しかし pasta はクリックのイベント `OnAnchorSelectEx` を受けていないので、シーンへ飛ばない。

## Current State

- 選択肢の行は次の経路で動く。
  - 文法: `choice_line = { pad ~ word_marker ~ question_marker ~ id ~ choice_label? ~ or_comment_eol }`（`crates/pasta_dsl/src/parser/grammar.pest:221-223`）。`local_scene_item` の選択肢の 1 つで、行の頭にしか書けない。
  - コード生成: `scope_gen.rs:343-371` が `act:choice("<target>", "<label or target>")` を出す。
  - 実行時: `act.lua:276-281` がトークン `{type="choice", target, display, scope}` を積む。
  - 出力: `sakura_builder.lua:29-37` が `\![*]\q[display,target,scope]` にする（スコープは大域シーン名。`On`・`script:` で始まる target には付けない）。
  - クリック: `shiori/event/choice_select.lua:41-78` の `REG.OnChoiceSelectEx` が受ける。Reference1 を ID、Reference2 を検索のスコープにして、`SCENE.search` でローカル → 大域の順に探す（前方一致）。見つからなければ 204 を返す。
- 台詞の行の中の要素は `actions`（`grammar.pest:176`）が読む。`＠` の後ろが `id` でないと、どの選択肢にも当たらない。そのため、`今日は＠？五月晴れだね。` はパースエラーになる。
- `\_a`・`OnAnchorSelect(Ex)` を扱う処理は無い。`OnAnchorSelectEx` は `EVENT.no_entry` に落ちる。そこで同名のシーンを探し、無ければ 204 を返す。
- LSP は選択肢の行に意味トークンを付けていない（`crates/pasta_lsp/src/analysis/visit_action.rs:35-37`「将来実装」）。VSCode の文法定義には `？` の規則が無い。台詞の中の `＠？五月晴れだね。` は、`inline-word-ref`（`pasta.tmLanguage.json:193`）が空白までを単語参照として塗る。

## Desired Outcome

- 台詞の中に `＠？シーン名` と書くと、その語がバルーンの中のアンカー（`\_a`）として表示される。クリックすると、そのシーンが再生される。
- 表示名を変えたいときは `＠？シーン名「表示名」` と書ける。
- アンカーはバルーンを選択肢待ちにしない。トークは普通の台詞と同じように流れ、同じように閉じる。
- エディタ（VSCode・LSP）でもリンクとして見分けられる。
- マニュアルとスキルの references に記法が載っている。

## Approach

台詞の要素に、インラインのアンカーを足す。出力はさくらスクリプトの `\_a[ID,スコープ]表示\_a` とし、クリックは `OnAnchorSelectEx` を今の選択肢と同じ振り分けで受ける。

- **`\q` ではなく `\_a` にした理由（2026-10-08 決定）**: `\q` はトーク全体を選択肢待ちにする（`!select` の制限時間の対象になり、`OnChoiceTimeout` が来る）。台詞の途中に 1 語のリンクを置いただけで吹き出しが止まるのは、「読み流せて、気になれば押せる」リンクに合わない。`\_a` は待たないうえ、表示の文字が普通の文字と同じに折り返す。`OnAnchorSelectEx` の Reference の並び（0 = 表示、1 = ID、2 以降 = 追加の引数）は `OnChoiceSelectEx` と同じなので、今の振り分けをそのまま使える。
- **記法は `＠？` にした（2026-10-08 決定）**: 「＠で始め、空白か改行で終える」という今の規則に揃える（`＠単語　` と同じく、終わりの空白は出力しない）。表示名は、選択肢の行と同じ `「」` で付ける。`「」` を付けた形は、終わりが目に見える形にもなる。`【五月晴れ】` の案は、文法として唐突に見えるので採らなかった。本文の `【】` を予約語にすると今ある辞書の意味が変わる、という点もある。終わりの見えにくさは、エディタの着色で補う。
- **選択肢の行とは衝突しない**: 台詞の行は必ず `名：` か `：` で始まる（`grammar.pest:245-246`）。インラインのアンカーになるのは、`：` より後ろの `＠？` だけである。`　＠？挨拶` だけの行は、今までどおり選択肢の行として読む。
- **飛び先の探し方は選択肢と同じ**: 同じ大域シーンの中のローカルシーンを先に探し、無ければ大域を探す。前方一致で、当たったものから 1 つ選ぶ。Call・選択肢と同じ規則なので、リンクだけの例外は作らない。
- **飛び先が無いとき（2026-10-08 決定）**: 選択肢と同じく、クリックしても何も起きない（204）。失敗の見せ方は `failure-output-unification` の仕組みに乗せる。読み込みのときの警告は、このspecでは足さない。

## Scope

- **In**:
  - 台詞の行（`action_line`・`continue_action_line`）の中の `＠？シーン名` と `＠？シーン名「表示名」` の文法・パーサ・AST（全角と半角の両方。`partial.rs` の span のずらしも含む）
  - コード生成（`element_gen.rs`）と、アンカーのトークンを積む act の API
  - `sakura_builder.lua` の `\_a` の出力（ID・スコープ・表示名のエスケープ。スコープの規則は選択肢と同じ）
  - `OnAnchorSelectEx` の受け口（`choice_select.lua` の振り分けを共有する）。明示の `＊OnAnchorSelectEx` シーンがあれば、選択肢と同じくそちらを先に実行する。
  - アンカーの前後の台詞の、ウェイトの挿入と BudouX の改行の扱い（下の Constraints）
  - LSP の意味トークンと、VSCode の文法定義（`inline-word-ref` との順序、文法のテスト）
  - マニュアル（`grammar/` の該当ページ、`shiori-events.md`・Lua の API のページ）と、スキル references の再生成
  - hello-pasta への作例の追加（要件で要否を決める）
- **Out**:
  - `\q` によるインラインの選択肢（選択肢待ちになるクリック部品）
  - 飛び先の無いリンクの、読み込み時の検出（`pasta_check validate` での検出は In。下の「申し送り（pasta-check-dic-validate より）」）
  - 変数でシーン名を決める形（`＠？＄変数`）
  - `On`・`script:` で始まる ID への直接の発火を、記法として案内すること
  - アンカーの色などの見た目の設定（ゴーストが `\f[anchor.font.color,…]` を書けばよい）
  - 選択肢の行そのものの挙動の変更

## Boundary Candidates

- DSL 層: 文法・パーサ・AST（`pasta_dsl`）
- 生成と実行時: コード生成（`element_gen.rs`）・act の API・`sakura_builder.lua`
- クリックの受け口: `OnAnchorSelectEx` の登録と、選択肢と共有する振り分け
- 編集環境: LSP と VSCode の文法定義
- 文書: マニュアル・スキル references・作例

## Out of Boundary

- 選択肢の行の挙動・`!select` の意味
- シーン検索の規則（前方一致・ローカル優先）そのもの
- 実行時の失敗の出力の仕組み（`failure-output-unification` が持つ）
- Call の属性フィルター（`call-attribute-filter` が持つ。アンカーへの属性フィルターも扱わない）

## Upstream / Downstream

- **Upstream**:
  - 選択肢の仕組み（`act:choice`・`sakura_builder.lua`・`choice_select.lua`）
  - シーン検索（`SCENE.search`）
  - `failure-output-unification`: 飛び先の無いクリックの失敗の見せ方
  - `call-attribute-filter`: 同じソースを触るための順序
- **Downstream**:
  - `getting-started-story-guide` の章立てで「リンクで話題を広げる」を扱うなら、その材料になる。
  - 将来、単語参照や属性フィルターをアンカーに広げる案（バックログ）。

## Existing Spec Touchpoints

- **Extends**: なし（選択肢の行の spec は完了済み。仕組みを共有するが、行の挙動は変えない）
- **Adjacent**:
  - `call-attribute-filter`: 次のものを共有する。
    - `grammar.pest`（台詞の要素と Call の文法）
    - `parse_action.rs`
    - `element_gen.rs`
    - `act.lua`
    - VSCode の文法定義
  - `failure-output-unification`: `act.lua` と失敗の表記を共有する。
  - `manual-claudia-theme`・`getting-started-story-guide`: マニュアルのページを共有する（同じページの別の節。後から入る側が rebase で合わせる）。

## Constraints

- **ウェーブ**: `call-attribute-filter` の後に置く。`grammar.pest`・`element_gen.rs`・`act.lua` は、1 ウェーブに 1 spec だけが持つ（roadmap の運用ルール）。
- **名前の終わり**: `id` は `XID_CONTINUE` を貪欲に読み、ひらがなも含む。そのため、`＠？五月晴れだね。` の飛び先は `五月晴れだね` になる。名前の終わりは、`＠単語` と同じく空白・改行、またはこの規則が止まる文字である。表示名を付けた形では `「」` が終わりになる。書かれたとおりに解釈し、意図を推測して救う規則は足さない。
- **折り返しとウェイト**:
  - 今の選択肢のトークンは `talk_to_script` を通らない。そのため、アンカーの前後で台詞が別のトークンに割れる。
  - BudouX の行幅の数えは、トークンごとにやり直しになる（`crates/pasta_lua/src/sakura_script/mod.rs:145`）。`\_a…\_a` の中の表示の文字にウェイトが入るか、行幅に数えるかも同じ問題である。
  - 文中のリンクで改行の位置がずれないことを、設計で決める。
- **マニュアルが権威**: 記法を足したら、同じ変更でマニュアルを直し、`node book/tools/gen-skill-refs.mjs` でスキル references を再生成する。
- **現行実装を正とする**: 選択肢の振り分けの現行の挙動（Ex だけを受ける、スコープは Reference2、前方一致）を基準にする。

## 申し送り（scene-name-alias より）

- 選択肢の飛び先は、範囲つきの検索で見つからなければ範囲なしの `SCENE.search(id, nil)` で探し直す（`choice_select.lua`）。この範囲なしの検索には別名表（既定 `OnTalk = ["会話"]`）が効くので、`＠？会話` の飛び先は OnTalk のシーンになる。アンカーで選択肢の振り分けを共有するなら、同じ挙動を引き継ぐ（意図して変えるなら要件で決める）。
- 参照: `.kiro/specs/completed/scene-name-alias/design.md`「SearchAlias」、マニュアル `grammar/call-jump.md#シーン名の別名`。

## 申し送り（choice-line-layout より）

2026-10-11 に `choice-line-layout` が完了し、選択肢の出力のまわりが次のように変わった。詳細は `.kiro/specs/completed/choice-line-layout/design.md` の「出力層」と「Revalidation Triggers」、マニュアルの `book/src/internals/talk-output.md`。

- **変わった事実**: `BUILDER.build`（`sakura_builder.lua`）が、吹き出しごとに「最後に表示したもの」（`spot_tail`。`"text"` か `"choice"`）を持つようになった。選択肢のトークンの前には、台詞の後なら `\n[choice_newlines×100]`、選択肢の後なら `\n` を出す。選択肢の後の台詞の前には `\n` を出す。台詞・`sakura_script` に書いた引数の無い `\c` は、吹き出しの消去として数える。
- **行番号がずれた**: `\q` への変換（`choice_to_script`）と `BUILDER.build` の場所は、上の「前提の変化」「触るファイル」に書いた行から動いている。
- **見直しが要る論点**: アンカーを選択肢と同じ形のトークンにすると、`spot_tail` の分岐を通るかどうかを決めることになる（選択肢として数えれば前後に改行が入り、台詞の途中の 1 語が行を分ける）。アンカーを台詞の文字の中に入れる形にすれば、字の判定（`emits_text`）が `\_a` をどう読むかだけになる。上の「見つけた穴」の `merge_consecutive_talks` の論点と合わせて、設計で決める。
- **複雑度**: `BUILDER.build` の循環的複雑度は 28 になった（`-- luacheck: ignore 561` を付けている。`choice-line-layout` は分割を範囲の外にした）。分岐を足すなら、分割するかどうかも論点になる。

## 申し送り（failure-output-unification より、2026-10-17）

- 失敗の見せ方を載せる先ができた。シーンが生きている間の失敗は `pasta/failure.lua` の `FAILURE.report(act, record)` から出す（警告 1 行と `【種類：対象（説明）】` の表記 1 つ）。新しい種類は `FAILURE.LABEL` に足す。表記の形・エスケープ・`[logging] balloon_warning` の切り替えは出口が持つ。
- 応答が表記だけになったシーンは 200 で返る。出さない設定で何も残らないときは 204 になる（`act:build` がすべて空の `raw_script` で nil を返す）。
- 参照: `.kiro/specs/completed/failure-output-unification/design.md`、マニュアル `book/src/reference/runtime-failures.md`。

## 申し送り（pasta-check-dic-validate より・2026-10-17 の棚卸で補った）

- 辞書の検査は `pasta_check validate`（と `release` の段 4 の関門、VSCode 拡張のゴースト全体の診断）として入った。核は `crates/pasta_check/src/validate/`（ファイルシステムに触れないライブラリ・WASM でも動く）。
- 検査を足すときに触る所: 報告の種類 `FindingKind`（`report.rs`。誤り／注意の分類と kebab-case の code・文言の `msg` を 1 か所で持つ）、検査の段（`validate/mod.rs` の `validate_ghost` が段の順に呼ぶ）、マニュアルの章 `book/src/debug/validate.md` の報告の表。
- 種類の追加・分類・出力の 1 行の形を変えるのは、`pasta-check-dic-validate` の design.md の Revalidation Triggers に当たる（CI・作者のスクリプト・マニュアル・`wasmBridge.ts` の形の検査に効く）。実行時と同じことを守る一致テスト `crates/pasta_check/tests/validate_parity_test.rs` も合わせて見直す。
- `＠？シーン名` の飛び先の検査は、`pasta-check-dic-validate` が範囲の外とした（design.md の Non-Goals）。この spec が `validate` に足す。誤りか注意かは、この spec の要件で決める。

## 申し送り（runtime-error-balloon より、2026-10-17）

- `FAILURE.LABEL` に `SCRIPT_ERROR = "スクリプトのエラー"`・`TIMEOUT = "時間切れ"` が入った（10 種）。種類を足すときは、hello-pasta のテスト（`tutorial_stages_test.rs`）が `LABEL` を全部読んで見張ることを前提にしてよい。
- `FAILURE.format` は `FAILURE.body`（枠なし・無効化なし）を無効化して【】で囲む形になった（出力は不変）。表記の形を変えるときは `FAILURE.body` も見る。
- 生成コードのシーン関数の先頭は `act:init_scene(SCENE, "書いたグローバル名", "書いたローカル名"|nil, "<.pasta>|<lua>:<pasta>,...")` になった。act は `scene_name`・`local_scene_name`・`line_map` を控える（`scene_names` は ponytail-lua-scripts で消した）。`call_restore` と `restore_scene(fn, ...)` は、関数が戻った後に `current_scene`・`scene_name`・`local_scene_name`・`line_map` の 4 つを呼ぶ前の値へ戻す。`act.lua`・コード生成・`init_scene` の行の数に触るときは、行の対応（`transpiler.rs` の `fill_line_maps`）と止まるエラーの表記を見直す。
- 参照: `.kiro/specs/completed/runtime-error-balloon/design.md` の Revalidation Triggers と `research.md` の「実装の記録」、マニュアル `book/src/reference/runtime-failures.md`「止まるエラーの表記」。

## 申し送り（pasta-check-validate-edges より）

- `pasta_check validate` の飛び先の検査は、辞書の中身が分からないとき（構文エラー・読めない辞書ファイル・pasta.toml の誤り）に丸ごと省く形になった。判定は `crates/pasta_check/src/validate/report.rs` の `skip_reasons`（統計の `skipReasons`）で、すべての段の報告がそろった後に 1 回だけ行う（`validate/mod.rs`）。
- 見直しが要る論点: この spec が `validate` に辞書のシーンの集まりに頼る検査を足すなら、`skip_reasons` が空のときだけ走らせるか。辞書の中身が分からないことを表す誤りの種類を足すなら、`skip_reasons` の対応に入れるか。
- 参照: `.kiro/specs/completed/pasta-check-validate-edges/design.md` の「Revalidation Triggers」と「Report（report.rs）」の省く理由の表。

## 申し送り（vertical-balloon-lines より、2026-10-18）

- **出力の組み立てが、吹き出しごとの列の状態を持つようになった**: `sakura_builder.lua` の `BUILDER.build` は、1 回の出力の中でスポットごとに `columns`（`{line, width, jump}`）を持ち、`talk`・`sakura_script` の変換 `SAKURA_SCRIPT.talk_to_script(actor, text, column)` に 3 つ目の引数で渡す。表情・ウェイト・`choice_timeout`・選択肢・`raw_script` は状態を動かさない。`newline` は n 列進め、`clear` は最初の列、`clear_spot` は全スポットの状態を捨てる。
- **区切りは 1 つの関数を通す**: pasta が出す区切り（段落の区切り・選択肢の前後・話者の切り替えの後の `spot_newlines`）は、局所関数 `put_break` で「出して 1 列進める」。改行を出す場所を足す・変えるときは `put_break` を通すこと（通さないと、`tests/lua_specs/sakura_builder_test.lua` の「作例・同値・不変条件」の不変条件のテストが落ちる）。
- **単独の `\` の守り**: 出力バッファ（`tracking_buffer`）が、最後の片が単位にならない `\` で終わるかを覚え、pasta が出すタグの前でエスケープの `\\` にする。新しいタグを出す所を足すときも、このバッファを通す。
- **`[ghost]` に `after_choice_newlines`（既定 1.0）が増えた**（選択肢の後の区切りの幅）。`[ghost]` のキーは 6 つになった（照合テスト・`reference/pasta-toml.md` の表・`book/AUTHORING.md` のキーの数）。
- 見直しが要る論点: この spec が `sakura_builder.lua` に改行や新しいタグを出す所を足すなら、`put_break` と `tracking_buffer` を通すか。字を出すトークンを足すなら、列の状態を `talk_to_script` に渡すか。
- 参照: `.kiro/specs/completed/vertical-balloon-lines/design.md`（SakuraBuilder の節・Revalidation Triggers・決定 7〜9）、マニュアル `book/src/internals/talk-output.md`。

## 申し送り（actor-display-style より、2026-10-18）

- **話者の切り替えで装いのタグが出るようになった**: `sakura_builder.lua` の `emit_actor_switch` は `\p[spot]` → 立ち絵の復旧（`APPEARANCE.restore`）→ 装いの出し入れ（`STYLE.switch`。`pasta/shiori/actor_style.lua`）の順に出す。装いの状態（スポットごとの `used`・`styled` と、出力全体の `any`）は、`BUILDER.build` が 1 回の出力ごとに `STYLE.new()` で作り、`emit_actor_switch` と `emit_inner_token` に渡す。
- **内側のトークンの出力は `STYLE.through` を通る**: 変換後の文字列の中で、引数の無い `\c`・`\x` の直後に話し手の装いのタグを差し込む（SSP 2.9.18 は `\c`・`\x` で文字の装飾を既定に戻すため）。最上位の `raw_script` は通さない。片を `put` と `put_raw` のどちらで足すかは、差し込む前の文字列で決める。
- **装いのあるアクターが出ない出力は 1 文字も変わらない**（`any` が偽の間、`switch` は空文字列を返し、`through` は文字列をそのまま返す）。
- 見直しが要る論点: この spec が次のどれかをするなら、装いの出し入れ（状態表と出し直し）をどう合わせるか。`emit_actor_switch` の出力順や `emit_inner_token` の形を変える。話者の切り替えの位置に新しいタグを出す。内側に `\c`・`\x` を出すトークンを足す。
- 参照: `.kiro/specs/completed/actor-display-style/design.md`（ActorStyle・BuilderHook・Revalidation Triggers）、マニュアル `book/src/internals/talk-output.md` の「装いの出し入れ」。

## 申し送り（external-control-events より、2026-10-23）

- `FAILURE.LABEL` に `NOT_JSON = "JSON として読めない値"`・`NOT_ASSIGNABLE = "代入できない値"` が入り、12 種になった（上の「10 種」は古い）。種類を足すときは `failure_test.lua` の数のテストも合わせて直す。
- 既定のハンドラの読み込みの並び（`pasta/shiori/event/init.lua`）の末尾に、`scene_play` の次の 1 行として `external_control` が入った。読み込みの行を足すときはその後ろになる。

## 申し送り（talk-grapheme-clusters より、2026-10-23 完了時）

- 台詞の「1 文字」が、コードポイントから拡張書記素クラスタ（見た目の 1 文字）に変わった。`sakura_script/tokenizer.rs` の `next_cluster`（2 つ目以降のコードポイントの `\` の手前で切る）が切り出しの正で、`line_breaker.rs` もこれを呼ぶ。
- 自動ウェイトは 1 トークン（クラスタ）に 1 つ（`wait_inserter.rs` の `append_with_wait`。旧名 `append_with_per_char_wait`）。テキストがちょうど `\` のトークンの後ろには入れない。
- budoux の幅は `line_breaker.rs` の `cluster_width`（1 コードポイントは今までどおり・`\r\n` は 0・それ以外は文字列の東アジアの幅）で数え、改行はクラスタの境目にだけ入る。`line_breaker.rs` は 1,155 行、`tokenizer.rs` は 608 行になった。
- 公開 Rust API: `CharSets` の各フィールドは `HashSet<String>`、`CharSets::classify` は `&str`（クラスタ）を取る。Lua からの呼び方（`talk_to_script`・`break_lines`）は変わらない。
- 見直しが要る論点: この spec が待ち・改行・列の数え方に手を入れるなら、上の単位（クラスタ）と関数名を前提にする。参照は `.kiro/specs/completed/talk-grapheme-clusters/design.md` の LineBreaker・WaitInserter の節と `book/src/internals/talk-output.md`。

## 申し送り（mlua-stdlib-removal より・2026-10-24）

- `FAILURE.LABEL` の `NOT_JSON = "JSON として読めない値"` は無くなり、同じ席が `BAD_TYPE = "型の名前にならない値"` になった（`OnPastaSetSaveVariable` の Reference2 の型の名前が知らないもの）。種類の数は 12 のまま。参照先: `.kiro/specs/completed/mlua-stdlib-removal/design.md` の FailureLabels。


## 申し送り（2026-10-29 の discovery「シーン属性による候補の絞り込み」より）

- `call-attribute-filter` は閉じて、新しい spec `scene-condition-attributes` に統合した（開発者の決定）。この brief の `call-attribute-filter` は `scene-condition-attributes` と読み替える。
- 中身が変わった。条件は呼ぶ側（`＞シーン＆k＝v`）でなく、場面の側に書く（`＊雑談_今の部屋　＆＄＊経験値≧500`）。
  - 呼ぶ側の Call の文法（`call_scene`・`parse_action.rs`・`element_gen.rs` の Call）は触らない見込みである。
  - 触るのは、属性の文法、`scene.lua`・`act.lua` の検索の道、`search/context.rs`・`scene_table.rs`、イベントからシーンを起こす道（選ぶ前の自動転記）、`pasta_check validate`、LSP と VSCode の文法定義である。
- 参照: `.kiro/specs/completed/scene-condition-attributes/brief.md`。
- この spec への影響: 「`call-attribute-filter` の後に置く」の理由だった Call の文法・`parse_action.rs`・構文木・`element_gen.rs` の重なりは、無くなる見込みである。残る重なりは `act.lua`・VSCode の文法定義・`visit_action.rs`・検査の段で、同じウェーブに置かない約束は残す。

## 申し送り（actor-line-newlines より、2026-11-06）

- **変わった事実（生成）**: トランスパイラ（`code_gen/element_gen.rs` の `generate_line_marker`）が、アクション行と継続行の 1 行ごとに、その行の呼び出しより前に `act:line()` を 1 行出すようになった（InvalidContinuation の検査の後。ソースマップはその行の位置）。生成 Lua の行番号がずれるので、行番号の重なりに頼るテスト材料は見直すこと。`pasta_check` の act のメソッドの一覧（`validate/builtins.rs` の `ACT_METHODS`）は 28 になった。
- **変わった事実（トークン）**: `act:line()` は `[ghost] line_newlines` が 0 でないときだけ印 `_seam_next` を立てる。次の `talk`・`sakura_script` のトークンが `seams = { 1 }`（字のバイト位置。昇順・1 始まり）を持ち、続く同じ話し手のトークンをまとめるとき（`merge_consecutive_talks`）にも位置を足して保つ。`raw_script` は印を取らない。`act.lua` の局所関数 `take_seam` を参照。
- **変わった事実（出力）**: `sakura_builder.lua` の列の状態に `joinable`・`line_pending` が加わった。継ぎ目の改行（`\n[N]`、N = floor(値×100)）は、トークンの字の中の継ぎ目の位置に `join_lines` が差し込む（`put_break` は通らない）。`next_column`・`first_column` がこの状態を消し、話し手の切り替え（S1）では `break_line` が切り替え先の状態を消す。詳細は `.kiro/specs/completed/actor-line-newlines/design.md` と `book/src/internals/talk-output.md`。
- 見直しが要る論点: この spec で、字を出すトークンを足すなら `take_seam` で継ぎ目を取るか。改行・区切りを出す場所を足すなら、その後に `joinable`・`line_pending` を消すか（`next_column` を通せば消える）。行の頭の生成（`element_gen.rs`）を触るなら、`act:line()` の位置（呼び出しの前）を保つか。

## 2026-11-12 棚卸の再測定（main 2381bf52）

- **前提の変化**（前回 2026-11-03・`c575febd` から）: `scene-condition-attributes`・`scene-pick-modes` が入り（v0.5.0）、順番の待ちは解けた。選択肢の受け口 `choice_select.lua`（73 行）は `SCENE.search(id, scope, act)`（`scene.lua` 182 行）で条件を絞り、名前に合うシーンはあるのに条件に合う候補が無いと、204 でなく止まるエラー `SCENE_UNMET` を出す（56〜64 行）。`FAILURE.LABEL` は 14 種（`SCENE_UNMET`・`NOT_TABLE` が増えた）。`＠数` の重み（`grammar.pest` 155〜160 行）は単語の値（`words`）の中だけの規則で、台詞の要素 `action`（189 行）には入っていない。`talk_word` は今も `＠` で止まるので `：今日は＠？五月晴れ` は今も構文エラーで、アンカーの腕を `action` に足しても `＠数` とはぶつからない。VSCode の文法定義は単語の値の規則（161 行）に `＠数` が入ったが、`inline-word-ref`（330 行）は今も `＠？` を塗らない。v0.5.2 の柵の直しは `pasta-lua-injection.tmLanguage.json` の側で、持ち場に掛からない。`debug-mcp-runtime` が入って `kick.lua` は消え、`STORE.co_preempt` を付けるのは `scene_play.lua`（113・151 行）だけ。`vscode-scene-outline-play` は LSP の `analysis/scenes.rs`・`visit_scope.rs` を足し・直したが、`visit_action.rs` は動いていない。
- **触るファイル**: `grammar.pest` 285（`actions` 190・`word_ref` 192・`choice_line` 236・台詞の行 259〜260 行）・`parse_action.rs` 499・`ast/action.rs` 371・`partial.rs` 335・`element_gen.rs` 570・`act.lua` 857（`merge_consecutive_talks` 108・`take_seam` 238・`act:choice` 339 行。1,000 行に近い）・`sakura_builder.lua` 690（`choice_to_script` 31・`emits_text` 136・`join_lines` 389・`put_break` 423・`emit_inner_token` 487・`BUILDER.speaks` 542・`BUILDER.build` 561 行）・`choice_select.lua` 73・`visit_action.rs` 463（選択肢は 35〜40 行）・`pasta.tmLanguage.json` 409・`validate/references.rs` 197（`CallScene` 35・`Choice` 72 行）・`report.rs` 522・`validate/mod.rs` 266（飛び先の検査は省く理由が無いときだけ走る 122〜125 行。そこへ足せば `skip_reasons` の論点は満ちる）、マニュアルと生成スキル。budoux に手を入れるなら `line_breaker.rs`（1,155 行。本体は 310 行まで）。`wordRefDecorator.ts`（85 行）は文法定義を読むので触らない見込み。受け口を `choice_select.lua` に置けば `event/init.lua` は触らない。
- **規模**: 17〜19 のまま。
- **先に要るもの**: 働きの上では無い。ファイルの重なり（未完了）: `scene-exact-match`（文法・`parse_action.rs`・`element_gen.rs`・`act.lua`・`choice_select.lua`・`references.rs`）、`shell-element-click`（`choice_select.lua`）、`cue-command-runtime`（`grammar.pest`・`act.lua`・`sakura_builder.lua`・`choice_select.lua`）、`novel-talk-flow`（`act.lua`・`sakura_builder.lua`・`line_breaker.rs`）、`novel-checkpoint-rollback`（`sakura_builder.lua`）、`novel-variable-scope`（文法・`parse_action.rs`・`partial.rs`・`element_gen.rs`・VSCode の文法定義）。`pasta-recipe` は `pasta.tmLanguage.json` を読むだけなので重ならない（ここで足す規則は向こうの着色にも効く）。
- **種別**: 機能（台詞の途中の `＠？` は今も構文エラー）。起票のときに開発者が決めたこと 3 つ（`\_a`・`＠？`・飛び先が無ければ 204）は前のまま。
- **要件定義のモデル**: Opus のまま（理由は前のとおり。下の条件漏れの扱いも、選択肢に揃えるか 204 にするかの 2 案）。
- **分割の案**: 分けない。20 を超えたら、エディタの分（`visit_action.rs`・`pasta.tmLanguage.json`・`book/tools/highlight/`）を `scene-anchor-link-editor` として後ろに出す。hello-pasta への作例は Scope から外す（どちらも前のまま）。
- **見つけた穴・古くなった記述**:
  - 新しい議題: 名前はあるが条件に合う候補が無いアンカーのクリックを、選択肢と同じ止まるエラー（`SCENE_UNMET`）にするか、決めたとおり 204 にするか。`＆順番`・`＆重み` は `SCENE.search` の中で効くので、アンカーにもそのまま効く。
  - 古い: Current State の行番号、「`inline-word-ref` が `＠？` を単語参照として塗る」「LSP は意味トークンを付けていない（将来実装）」、Constraints の「`mod.rs:145`」（列は吹き出しごとに持ち越す）、Upstream・Adjacent・Constraints の `call-attribute-filter`（入った `scene-condition-attributes` と読み、順番の待ちは無い）、Adjacent の `failure-output-unification`（完了）。申し送りの「12 種」は 14 種、「読み込みの行は `external_control` の後ろ」は今は末尾の `recommend`（`event/init.lua` 91 行）の後ろ。
  - 前から残るもの: `OnAnchorSelectEx` に 204 を返すと SSP は続けて `OnAnchorSelect` を送る（UKADOC。要件に書く）。`script:` で始まる ID を `\_a` がどう扱うかは確かめていない。マニュアルの色付けは VSCode の文法定義を読む（`book/tools/highlight/highlight-html.mjs` 39 行）。テストは `lua_specs/choice_select_test.lua`・`tests/shiori/event_dispatch_test.rs` に置ける。
