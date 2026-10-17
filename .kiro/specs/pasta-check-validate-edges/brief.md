# Brief: pasta-check-validate-edges

2026-10-17 の棚卸で起票した。`pasta-check-dic-validate`（2026-10-17 完了）が「裁定が要るので直していない」として残した端のケース 3 つと、「コマンドが入った後の後片付け」として範囲の外にした代用品の置き換えを拾う（出どころ: `.kiro/specs/completed/pasta-check-dic-validate/tasks.md` の Implementation Notes の最後の項目、`requirements.md` の範囲外の「代用品の置き換え」）。

## Problem

辞書の検査（`pasta_check validate`）は、ふつうの辞書では pasta.dll と同じ規則で正しく報告する。けれども、次の 3 つの端では、報告が原因を指さない、または検査が走らない。

1. UTF-8 で読めない辞書・読めない辞書があっても、飛び先の検査を省かない。そのファイルにあるシーンが「無い」ことになり、そこへの Call・選択肢が、本当の原因（読めないファイル）とは別の「飛び先が無い」の誤りとして並ぶ。
2. 名前が `.pasta` で終わるフォルダは報告しない（検査の核はファイルだけを入力にする）。
3. VSCode 拡張は、小文字の `ghost/master` の下だけを検査する。CLI は Windows で大文字・小文字を区別しないので、`Ghost/Master` のゴーストは CLI では検査され、エディタでは検査されない。

あわせて、マニュアルの CI（`.github/workflows/manual.yml` の「Tutorial syntax validation」の段）と、hello-pasta のテスト（`crates/pasta_sample_ghost/tests/tutorial_stages_test.rs` の `scene_names_do_not_prefix_collide`）は、検査のコマンドが無かった頃の代用品のままである。

## Current State

- 飛び先の検査を省く条件は、構文エラーが 1 つでもあることだけ（`crates/pasta_check/src/validate/mod.rs` 74 行付近。要件 6.8）。読めないファイルは `FileText::Unreadable`・`FileText::NotUtf8`（`validate/input.rs`）として届くが、省く条件に入っていない。
- 検査の核の入力はファイルの一覧で、フォルダは見ない（`validate/discovery.rs`）。
- VSCode 拡張の `findGhostMaster`（`editors/vscode/src/ghostValidator.ts` 18 行付近）は `segments[i] === 'master' && segments[i - 1] === 'ghost'` で探し、見張り（同 70 行付近）も `**/ghost/master/**/*.{pasta,lua}` である。
- `manual.yml` の段は、hello-pasta の実ファイルを本物の読み込みで読むテスト（`cargo test -p pasta_sample_ghost`）を構文の検証の代わりにしている（コメントは 2026-10-17 の棚卸で直した）。
- `scene_names_do_not_prefix_collide` は、グローバルシーン名どうしが前方一致で重ならないこと（シーンの検索は前方一致で、`＊第1章` を呼ぶと `第10章` にも当たる）を確かめる。この検査は `validate` に無いので、そのまま置き換えられない。

## Desired Outcome

- 読めない辞書があるときは、その原因の報告だけが出て、それに引きずられた「飛び先が無い」は出ない（または、引きずられたものと分かる形で出る）。
- `.pasta` で終わるフォルダの扱いが決まり、決めたとおりに報告される。
- エディタと CLI で、検査するゴーストの範囲が同じになる。
- 代用品を `validate` に置き換えるかどうかが決まり、置き換えるなら置き換わっている。

## Approach

- 1 は、飛び先の検査を省く条件に「読めないファイルがある」を足すのが素直な案（要件 6.8 の構文エラーと同じ扱い）。ファイルごとに省く案もあるが、どのシーンがどのファイルにあったかは読めないと分からないので、全体を省く案を推奨にする。
- 2 は、注意として報告する案を推奨にする（pasta.dll はフォルダを読まないので、書いた人の思い違いの可能性が高い）。
- 3 は、拡張の照合を大文字・小文字を区別しない形にそろえる（Windows の CLI に合わせる）。
- 代用品は、`manual.yml` の段を `pasta_check validate` に置き換えるかを要件で決める。前方一致の重なりを `validate` の新しい注意にするか（`novel-talk-flow` の完全一致の場面選びとの関係も見る）は、要件の議題にする。

## Scope

- **In**: 上の 3 つの端の裁定と直し、報告の表（マニュアル `book/src/debug/validate.md`）の更新、代用品の置き換えの裁定と、決めたなら置き換え。
- **Out**: 新しい種類の検査を広げること（前方一致の重なりの注意は、議題で足すと決めた場合だけ In）、機械向けの出力の形（JSON など）、`release` の段の中身。

## Boundary Candidates

- 検査の核（`crates/pasta_check/src/validate/`）と、VSCode 拡張（`editors/vscode/src/ghostValidator.ts`）と、CI・テストの代用品（`manual.yml`・`tutorial_stages_test.rs`）の 3 つ。どれも小さいので 1 本で扱う。

## Out of Boundary

- 文法・ランタイムの読み込みとシーンの検索の規則の変更。

## Upstream / Downstream

- **Upstream**: `pasta-check-dic-validate`（完了）。
- **Downstream**: 検査を足す後の spec（`call-attribute-filter`・`scene-anchor-link`・`novel-talk-flow`）。報告の種類を足すときの決まりは同じ。

## Existing Spec Touchpoints

- `pasta-check-dic-validate` の design.md の Revalidation Triggers（報告の種類・分類・1 行の形を変えるとき）。一致テスト `crates/pasta_check/tests/validate_parity_test.rs` も見直す。

## Constraints

- 報告の種類を足す・分類を変えるときは、CI・作者のスクリプト・マニュアル・`wasmBridge.ts` の形の検査に効く。
- `Cargo.lock` は変えない見込み（依存を足さない）。

## 測定（2026-10-17 の棚卸、main e9b321e0）

- 触るファイル: `crates/pasta_check/src/validate/mod.rs`・`discovery.rs`・`report.rs`（種類を足す場合）、`crates/pasta_check/tests/`、`editors/vscode/src/ghostValidator.ts` とそのテスト、`book/src/debug/validate.md`、場合により `.github/workflows/manual.yml`・`crates/pasta_sample_ghost/tests/tutorial_stages_test.rs`。1,000 行に近いファイルは無い。
- 規模: 4〜7 タスク。
- 先に要るもの: なし。ほかの未完了の spec とソースのファイルは重ならない（`runtime-error-balloon`・`getting-started-screenshots` とも重ならない）。
- 種別: バグ（1 は、本当の原因と別の誤りが並ぶ）と後片付け。
- 要件定義のモデル: Opus（道具の報告の裁定で、文法・保存の形・公開の Lua の約束ではない）。
- 議題: 読めないファイルがあるときに省く範囲／`.pasta` で終わるフォルダを注意にするか／前方一致の重なりを注意として足すか／`manual.yml` の段の置き換え。
- 0.4.1 に入れる（2026-10-17 開発者の決定。0.4.0 の遅れを止めるため、致命的でない不具合は 0.4.1 に回す）。0.4.0 を出した直後に始める。

## 2026-10-17 棚卸の再測定（main b4164100）

- **前提の変化**: 前回の測定（main e9b321e0）の後に入った変更は、この spec の触るファイルに届いていない。zip の版上げで変わったのは `crates/pasta_check/src/nar.rs`・`release.rs` のテストの 5 行（入れ物の中の名前の取り出し方）だけで、辞書の検査の核（`crates/pasta_check/src/validate/`）と検査の入口（`validate_cmd.rs`）は変わっていない。マニュアルの CI（`.github/workflows/manual.yml`）の変更は「Tutorial syntax validation」の段のコメント 1 行だけで、段の中身（`cargo test -p pasta_sample_ghost`）はそのまま。「Current State」は今のコードと合う（飛び先の検査を省く条件は `validate/mod.rs` 74 行、ゴーストのフォルダを探す関数の照合は `editors/vscode/src/ghostValidator.ts` 22 行、見張りは 70 行）。
- **触るファイル**: 検査の核の `validate/mod.rs`（128 行）・`report.rs`（379 行。種類を足すとき）・`discovery.rs`（200 行）。フォルダを報告するなら、核の入力の形 `validate/input.rs`（33 行。今はファイルの一覧だけを持つ）と、一覧を集める 2 か所（CLI の `crates/pasta_check/src/validate_cmd.rs` 309 行の `walk`、拡張の `ghostValidator.ts` 314 行の `walk`）。入力の形を変えるなら、拡張と WASM の橋渡し `editors/vscode/src/wasmBridge.ts`（235 行）・`crates/pasta_lsp/src/transport.rs` と、WASM の作り直し。テストは `crates/pasta_check/tests/validate_*.rs`・`editors/vscode/src/test/ghostValidator.test.ts`。マニュアル `book/src/debug/validate.md`。場合により `manual.yml` と `crates/pasta_sample_ghost/tests/tutorial_stages_test.rs`（483 行）。新しいファイルは無い見込み。1,000 行に近いものは無い。
- **規模**: 5〜7 タスク（入力の形を変えてフォルダを報告するなら 7）。
- **先に要るもの**: 無い。0.5.0 の先頭の 2 本とはファイルが重ならない（`novel-areka-contract` は文書だけ。`scene-attribute-store` は文法の読み取り・コード生成・`scene.lua`・`pasta_core` のシーン登録で、`pasta_check` も拡張も触らない）。検査の段の `validate/mod.rs` は、後で検査を足す `call-attribute-filter`・`scene-anchor-link`・`cue-command-runtime`・`novel-talk-flow` も触るので、この 4 本とは同じウェーブに置かない。マニュアル `validate.md` は、同じページの別の節なら並べてよい。
- **種別**: バグ（読めない辞書があると、本当の原因と別の「飛び先が無い」が並ぶ。拡張が `Ghost/Master` のゴーストを検査しない）と後片付け（代用品の置き換え）。
- **要件定義のモデル**: Opus（道具の報告の裁定で、文法・保存の形・公開の Lua の約束ではない）。
- **分割の案**: 分けない。
- **見つけた穴・古くなった記述**:
  - 「Current State」の「54・74 行付近」の 54 行は、検査の関数 `validate_ghost` の始まり（55 行）である。省く条件は 74 行の 1 か所だけ。
  - `.pasta` で終わるフォルダを報告するには、核の入力にフォルダの名前を足すか、一覧を集める 2 か所（CLI と拡張）で別々に報告するかを決める。入力の形を変えると、WASM の作り直しと `wasmBridge.ts` の形の検査に効く（`pasta-check-dic-validate` の design.md の Revalidation Triggers）。
  - 拡張の見張りの書き方（`**/ghost/master/**`）が Windows で大文字・小文字を区別するかは、確かめていない。照合の関数だけを直しても、見張りが `Ghost/Master` の下の保存を拾わなければ検査は起きない。要件で確かめる。
