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

## 2026-10-10 棚卸の再測定（main add05022）

- **前提の変化**: 起票してから初めての再測定。「Current State」に書いた場所は、1 か所を除いて現在のコードと一致する（選択肢の文法は `crates/pasta_dsl/src/parser/grammar.pest` 221〜223 行、`act:choice` は `crates/pasta_lua/pasta_scripts/pasta/act.lua` 276〜281 行、`\q` への変換は `shiori/sakura_builder.lua` 29〜37 行、クリックの受け口は `shiori/event/choice_select.lua` 41〜78 行）。ずれていたのは、選択肢の Lua を出す場所で、`crates/pasta_lua/src/code_gen/scope_gen.rs` の 355〜371 行に動いた。「申し送り（scene-name-alias より）」も合っている（範囲なしの探し直しは `choice_select.lua` 71〜72 行）。
- **伺かの仕様の確認**（SSP の公式の仕様書 UKADOC で確かめた）: `\_a[ID,追加の引数]表示\_a` をクリックすると `OnAnchorSelectEx` が来て、Reference0 が表示、Reference1 が ID、Reference2 以降が追加の引数になる。選択肢の制限時間は掛からず、普通の台詞と同じにバルーンが閉じる。ID が `On` で始まると、その名前のイベントが直接起きる。ここまでは brief のとおり。`script:` で始まる ID を `\_a` がどう扱うかは、確かめられなかった。
- **触るファイル**: `grammar.pest` 272、台詞の要素を読む `crates/pasta_dsl/src/parser/parse_action.rs` 456、構文木 `ast/action.rs` 323、位置をずらす `crates/pasta_dsl/src/partial.rs` 477、台詞の Lua を出す `crates/pasta_lua/src/code_gen/element_gen.rs` 564、`act.lua` 726、`sakura_builder.lua` 288、`choice_select.lua` 80、ウェイトと改行を入れる `crates/pasta_lua/src/sakura_script/`（`mod.rs` 247 ほか）、エディタ向けの色付け `crates/pasta_lsp/src/analysis/visit_action.rs` 431、VSCode の文法定義 `editors/vscode/syntaxes/pasta.tmLanguage.json` 272 と単語参照の枠の表示 `editors/vscode/src/wordRefDecorator.ts`、マニュアルと生成スキル。1,000 行に近いものは無い。
- **規模**: 約 17〜20 タスク（文法と読み取り 3、Lua の生成と `act` の関数 2、`\_a` の出力 2、ウェイトと改行 2、クリックの受け口 2、エディタ 3、マニュアル 2、テスト 2）。hello-pasta への作例まで入れると 20 を超える。
- **先に要るもの**: 機能として要るのは `failure-output-unification` だけで、それも弱い（飛び先が無いときは何も起きない、と決まっている）。`call-attribute-filter` とは同じファイルを触る順番だけの関係なので、どちらを先にしてもよい。ファイルの重なり: `call-attribute-filter`（`grammar.pest`・`parse_action.rs`・`ast/action.rs`・`partial.rs`・`element_gen.rs`・`act.lua`・VSCode の文法定義・`visit_action.rs`）、`failure-output-unification`（`act.lua`）。`scene-attribute-store` とは重ならない。
- **種別**: 機能（台詞の途中の `＠？` は今は構文エラー）。起票のときに開発者が決めたことが 3 つ記録されている（`\_a` にする・記法は `＠？`・飛び先が無ければ何も起きない）。
- **要件定義のモデル**: Fable（新しい書き方。伺かの仕様の読み方、台詞が途中で割れるときのウェイトと改行、クリックと続きのトークの順番を決める）。
- **分割の案**: 分けない。hello-pasta への作例は Scope から外す。hello-pasta の辞書は入門ガイドの段階表と照らし合わせて検査しているので（`crates/pasta_sample_ghost/tests/tutorial_stages_test.rs`）、作例を足すと段階表・`getting-started-story-guide`・`hello-pasta-shell-art` と重なる。
- **見つけた穴・古くなった記述**:
  - `OnAnchorSelectEx` に何も返さないと、SSP は続けて `OnAnchorSelect` を送る（UKADOC）。飛び先が無くて 204 を返すと、この 2 つ目が来る。今の受け口は同じ名前のシーンを探して、無ければ 204 を返すだけなので害は無い。要件に書いておく。
  - VSCode の単語参照の枠（`wordRefDecorator.ts` 6 行の正規表現）は、`＠？名前` も単語参照として囲む。Scope に足す。
  - マニュアルのコードの色付けは、VSCode の文法定義をそのまま読む（`book/tools/highlight/highlight-html.mjs` 39 行）。文法定義を変えると、マニュアルの表示にも効く。
  - アンカーを選択肢と同じ形のトークンにすると、`act.lua` の続いた台詞を 1 つにまとめる処理（85 行の `merge_consecutive_talks`）で、前後の台詞が別々のままになる。Constraints の「折り返しとウェイト」はここで起きる。アンカーを台詞の文字の中に入れる形にするかを、設計で決める。
  - 選択肢のテストは `pasta_lua` 側にある（`crates/pasta_lua/tests/lua_specs/choice_select_test.lua`・`tests/shiori/event_dispatch_test.rs`）。アンカーのテストも同じ場所に置けば、`shiori-test-support-runtime` と重ならない。

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
- 生成コードのシーン関数の先頭は `act:init_scene(SCENE, "書いたグローバル名", "書いたローカル名"|nil, "<.pasta>|<lua>:<pasta>,...")` になった。act は `scene_name`・`local_scene_name`・`line_map`・`scene_names` を控え、`call_restore` は 4 つを戻し、`restore_scene` はグローバル名だけを戻す。`act.lua`・コード生成・`init_scene` の行の数に触るときは、行の対応（`transpiler.rs` の `fill_line_maps`）と止まるエラーの表記を見直す。
- 参照: `.kiro/specs/completed/runtime-error-balloon/design.md` の Revalidation Triggers と `research.md` の「実装の記録」、マニュアル `book/src/reference/runtime-failures.md`「止まるエラーの表記」。

## 2026-10-17 棚卸の再測定（main b4164100）

- **前提の変化**: `runtime-error-balloon` が `crates/pasta_lua/pasta_scripts/pasta/act.lua`（28 行増えて 829 行）と、場面の関数を出すコード生成 `crates/pasta_lua/src/code_gen/scope_gen.rs`（`init_scene` に書いた名前と行の対応を渡す）を変えた。生成した Lua の行と `.pasta` の行の対応は、トランスパイラ（`crates/pasta_lua/src/transpiler.rs` の `fill_line_maps`）が、コード生成が書いた行の数から作る。アンカーの Lua を出すときも、行を書き出す関数（`writeln`）を通す（上の申し送り）。選択肢の仕組み（`act:choice`・`\q` の出力・クリックの受け口）の中身は変わっていない。`failure-output-unification` は完了し、出口 `pasta/failure.lua` を使うだけの関係になった。
- **行番号の直し**（「Current State」と 2026-10-10 の節）: 台詞の要素の規則 `actions` は `grammar.pest` 177 行、台詞の行は 246〜247 行。選択肢の Lua を出す関数 `generate_choice` は `scope_gen.rs` 390〜404 行。`act:choice` は `act.lua` 305 行、続いた台詞をまとめる `merge_consecutive_talks` は 94 行。`\q` への変換 `choice_to_script` は `sakura_builder.lua` 30 行、`BUILDER.build` は 266 行。範囲なしの探し直しは `choice_select.lua` 70〜71 行。VSCode の `inline-word-ref` は `pasta.tmLanguage.json` 192 行。
- **触るファイル**: 文法 `crates/pasta_dsl/src/parser/grammar.pest`（272 行）、`parse_action.rs`（456）、構文木 `parser/ast/action.rs`（323）、`partial.rs`（477）、台詞の Lua を出す `crates/pasta_lua/src/code_gen/element_gen.rs`（564）、`act.lua`（829。1,000 行に近づいている）、`shiori/sakura_builder.lua`（383）、`shiori/event/choice_select.lua`（80）、ウェイトと改行の `crates/pasta_lua/src/sakura_script/mod.rs`（247）、エディタの色付け `crates/pasta_lsp/src/analysis/visit_action.rs`（431）、`editors/vscode/syntaxes/pasta.tmLanguage.json`（272）・`editors/vscode/src/wordRefDecorator.ts`（41）、飛び先の検査 `crates/pasta_check/src/validate/`（`references.rs` 135・`report.rs` 379・`mod.rs`）と一致テスト、マニュアルと生成スキル。新しいファイルは無い見込み。
- **規模**: 18〜20 タスク（前回の 17〜20 に、辞書の検査への飛び先の検査 1 を足した。テストは各タスクに含める）。
- **先に要るもの**: 機能として要るものは無い（失敗の出口は入った）。ファイルの重なり: `call-attribute-filter`（文法・`parse_action.rs`・構文木・`partial.rs`・`element_gen.rs`・`act.lua`・VSCode の文法定義・`visit_action.rs`・検査の段）、`scene-attribute-store`（`grammar.pest`・`partial.rs`・`element_gen.rs`・`act.lua`）、`cue-command-runtime`（`grammar.pest`・`act.lua`・`sakura_builder.lua`・`choice_select.lua`・検査の段）、`novel-talk-flow`（`act.lua`・`sakura_builder.lua`・検査の段）、`novel-checkpoint-rollback`（`act.lua`・`sakura_builder.lua`）、`shell-element-click`（`act.lua`・`choice_select.lua`）、`pasta-check-validate-edges`（検査の段の `validate/mod.rs`）。どれとも同じウェーブに置けない。
- **種別**: 機能（台詞の途中の `＠？` は今は構文エラー）。起票のときに開発者が決めたことが 3 つある。
- **要件定義のモデル**: Opus に下げる。後から戻しにくいのは書き方（`＠？名前`・`「表示名」`）と出力（`\_a`）と飛び先が無いときの動き（204）で、どれも 2026-10-08 に開発者が決めてある。伺かの仕様（`OnAnchorSelectEx` の Reference の並び・制限時間が掛からないこと）も UKADOC で確かめてある。残る論点（アンカーを独り立ちのトークにするか台詞の文字に入れるか、それに連なるウェイト・BudouX の行幅・吹き出しの最後の表示による改行）は出力の組み立ての中の話で、2 つの案に絞れていて、後から直せる。
- **分割の案**: 分けない。要件で 20 を超えたら、エディタの分（`visit_action.rs`・`pasta.tmLanguage.json`・`wordRefDecorator.ts` と、文法定義を読むマニュアルの色付けのテスト `book/tools/highlight/`）を `scene-anchor-link-editor` として後ろに出す。そのときは、間の期間、台詞の中の `＠？` が単語参照の色で塗られることを受け入れる。
- **見つけた穴・古くなった記述**:
  - 「Adjacent」の `failure-output-unification`（`act.lua` と失敗の表記を共有）は、完了したので外してよい。
  - 選択肢の飛び先の検査（`validate/references.rs`）は、前方一致と別名まで実行時と同じ規則で探している。アンカーの検査はこれを使える。読む場所は、選択肢の行に加えて台詞の中の要素が 1 つ増える。

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
