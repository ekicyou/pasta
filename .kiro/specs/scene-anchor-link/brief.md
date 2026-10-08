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
  - 飛び先の無いリンクの、読み込み時・`pasta_check` での検出（バックログの「`.pasta` を検査するコマンド」で、選択肢・Call とまとめて扱う）
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
