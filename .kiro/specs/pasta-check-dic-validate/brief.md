# Brief: pasta-check-dic-validate

起点: 2026-10-10 の棚卸でバックログの「`.pasta` を検査するコマンド」から起票した。同じバックログの「モジュール名の衝突」「サニタイズ後のシーン名の衝突の警告」と、完了した scene-name-alias が範囲の外に置いた「pasta_check に pasta.toml を読ませる」を、検査の項目として引き取る。

## Problem

ゴーストの作者は、辞書（`.pasta`）の書き間違いを、SSP でゴーストを起動するまで確かめられない。配布物を作る道具 `pasta_check` は辞書を読まないので、構文が壊れた辞書でも nar ができてしまう。

辞書を起動せずに確かめたい場面は何度も出ていて、そのたびに手作りの代用品で埋めている。後から入る spec も「読み込む前に見つける仕組みは、このコマンドで」と先送りしている。受け皿が無いまま、先送りと代用品だけが増えている。

## Current State

2026-10-10 の main（`add05022`）をコードで確かめた。

- **コマンドの入口**: `pasta_check` の入口（`crates/pasta_check/src/main.rs` 16〜20 行）が受け付けるのは、`Release`・`Help`・`Version` の 3 つだけである。依存する部品は `lexopt`・`md5`・`zip` の 3 つ（`crates/pasta_check/Cargo.toml` 18〜26 行）で、辞書を読む部品には依存していない。
- **先送りしている spec**:
  - scene-anchor-link は、飛び先の無いリンクを読み込みのときに見つける仕事を、選択肢・Call とまとめてこのコマンドに任せている（`.kiro/specs/scene-anchor-link/brief.md` 52 行、`.kiro/steering/roadmap.md` の「文中のシーンリンク」）。
  - failure-output-unification は、実行する前に見つける仕組みを範囲の外に置き、`pasta_check` の検証・リリースとの関係を論点に残している（`.kiro/specs/failure-output-unification/brief.md` 45 行・66 行）。
- **代用品**:
  - マニュアルの自動検査（`.github/workflows/manual.yml` 128〜135 行）は、「構文を検証するサブコマンドが無いため」と書いたうえで、hello-pasta を本物のランタイムで読み込むテスト（`cargo test -p pasta_sample_ghost`）を構文の検証の代わりにしている。
  - hello-pasta の段階辞書のテスト（`crates/pasta_sample_ghost/tests/tutorial_stages_test.rs` 363〜445 行）は、行頭の `＊名前` を自前で拾う関数（370〜397 行）と、名前どうしの前方一致の重なりを調べる検査（405〜445 行）を手で書いている。
  - 完了した dsl-codegen-runtime-safety は、マニュアルの作例を確かめるために使い捨てのクレートを作った（`.kiro/specs/completed/dsl-codegen-runtime-safety/tasks.md` 114 行）。
- **案内だけが先にある**: `pasta_check` のスキルの説明（`.claude/skills/pasta-check/SKILL.md` 3 行）は、「将来的なテスト・検証コマンドを含む」「ゴースト検証」とすでに書いている。
- **引き取る 3 件の現状**:
  - モジュール名: 辞書のファイル名からモジュール名を作る処理（`crates/pasta_lua/src/loader/cache.rs` 238〜252 行の `source_to_module_name`）は、`.` と `-` をどちらも `_` に置き換える。`a-b.pasta`・`a.b.pasta`・`a_b.pasta` は同じモジュール名になる。読み込みの処理（`crates/pasta_lua/src/loader/process.rs` 77〜97 行）が見ているのは `.pasta` と `.lua` の重なりだけで、`.pasta` どうしの重なりは見ていない。
  - 照合用の名前: シーン名を照合用の名前に揃える処理（`crates/pasta_core/src/registry/scene_registry.rs` 244〜246 行の `sanitize_name`）は、文字・数字・`_` 以外を `_` に置き換える。`会話·A` と `会話_A` は同じ名前になる（同じファイルの 223〜227 行の説明）。重なっても知らせる仕組みは無い。
  - 別名表: 別名表の型は `crates/pasta_core/src/registry/scene_alias.rs` にあり、pasta.toml の `[scene.alias]` を読む処理は `crates/pasta_lua/src/loader/config/mod.rs`（112 行の `parse_scene_aliases`）にある。マニュアル（`book/src/reference/pasta-toml.md` 349 行）は「pasta_check は別名表を読まない」と書いている。
- **読むだけで分かること・分からないこと**:
  - Call の飛び先は 5 段で探す（`crates/pasta_lua/pasta_scripts/pasta/act.lua` 313〜322 行）。今のシーンの表、ローカルの辞書の前方一致、`act` の関数、`GLOBAL` の完全一致、グローバルの辞書の前方一致の順である。辞書の外（Lua）で決まる段が 3 つある。
  - Call は変数でも書ける（`＞＄変数`。`crates/pasta_dsl/src/parser/grammar.pest` 169〜170 行）。飛び先は実行するまで決まらない。
  - 前方一致で候補が複数になるのは、pasta の書き方そのものである（`book/src/grammar/call-jump.md` 121 行から）。重なりを一律に誤りにはできない。
- **エディタの側**: 言語機能（LSP）の `pasta_lsp` が依存する pasta の部品は `pasta_dsl` だけで（`crates/pasta_lsp/Cargo.toml`）、出す診断は構文エラーだけである（`crates/pasta_lsp/src/analysis/mod.rs` 57〜77 行）。`pasta_lsp` はブラウザー向けの形式（WASM）にもビルドするので、LuaJIT を含む `pasta_lua` には依存できない。
- **文書**: マニュアルに `pasta_check` のページは無い（`book/src/SUMMARY.md` の 47 章に無い。名前が出るのは `reference/pasta-toml.md` 349 行と `internals/index.md` 76 行だけ）。使い方は `crates/pasta_check/README.md` とスキルが持っている。

## Desired Outcome

- 作者が `pasta_check` のコマンド 1 つで、SSP を起動せずにゴーストの辞書を検査できる。
- 問題があれば、ファイル・行・理由を示して、失敗の終了コードで終わる。
- 次の 4 つが検査の項目に入っている: 構文エラー、モジュール名の重なり、照合用の名前の重なり、飛び先の無い参照（読むだけで決まる範囲）。
- scene-anchor-link と failure-output-unification が先送りした「読み込む前に見つける仕組み」の受け皿になる。
- 使い方がマニュアルとスキルに載っている。

## Approach

起票時の見立て。要件フェーズで確定する。

- **サブコマンドを 1 つ足す**: ゴーストのフォルダを受け取り、辞書を全部読んで、検査の項目を順に当てる。名前（`check`・`validate` など）は要件で決める。
- **検査を 2 層で考える**: 1 つのファイルだけで分かること（構文）と、ゴースト全体を見て分かること（名前の重なり・飛び先）。
- **最初の分かれ道は依存の重さ**: 構文と名前の検査だけなら、構文を読む部品 `pasta_dsl` と、名前の規則を持つ `pasta_core` で足りる。Lua への変換まで確かめるには `pasta_lua` が要り、`cargo install pasta_check` が同梱の LuaJIT をビルドするようになる。
- **問題があれば止める**: 見つけたものを「誤り」と「注意」に分け、誤りが 1 つでもあれば失敗で終わる。壊れた辞書を警告だけで通さない。
- **書いたとおりに読む**: 意図を推測して救う規則は足さない。読むだけでは決まらないもの（変数の Call・Lua で決まる飛び先）は、決まらないものとして扱う。
- **今の `release` は変えない**: 配布物を作る 5 段（`crates/pasta_check/src/release.rs` 13〜50 行）の中身は触らない。`release` の前に検査を挟むかは、要件で決める。

## Scope

- **In**:
  - サブコマンドの追加（引数・終了コード・出力の形）
  - 検査の項目
    - 構文エラー（全部のファイルを読み、見つけた分をまとめて報告する）
    - モジュール名の重なり（バックログの「モジュール名の衝突」）
    - 照合用の名前に揃えた後のシーン名の重なり（バックログの「サニタイズ後のシーン名の衝突の警告」）
    - 飛び先の無い Call・選択肢（読むだけで決まる範囲）
    - pasta.toml を読むこと（scene-name-alias が範囲の外に置いた項目。別名表 `[scene.alias]` と、辞書の場所 `[loader]` の `pasta_patterns`）
  - テスト（モジュールごとのテストと、コマンドを実際に走らせるテスト `crates/pasta_check/tests/cli_test.rs`）
  - 文書: マニュアルの新しいページ、`reference/pasta-toml.md` の「別名表を読まない」の書き換えとスキル references の再生成、`crates/pasta_check/README.md`、スキル `pasta-check`、steering
- **Out**:
  - 実行時の失敗の見せ方（failure-output-unification が持つ）
  - ランタイム（`pasta.dll`）が読み込みのときに出す警告を足すこと。このコマンドの側だけで知らせる
  - 台詞の中のリンク `＠？シーン名` の記法そのもの（scene-anchor-link が持つ。検査の追加は下の Constraints）
  - Lua のスクリプト（`scripts/`）の検査
  - エディタ（LSP・VSCode 拡張）に診断を足すこと。検査を共有できる形にするかだけを要件で決める
  - 代用品の置き換え（`manual.yml` の回避、`tutorial_stages_test.rs` の手作りの検査）。コマンドが入った後の後片付けにする

## Boundary Candidates

- コマンドの入口（引数・終了コード・出力の形）
- 辞書の集め方（pasta.toml を読む・既定の場所だけを見る）
- 1 つのファイルの検査（構文）
- ゴースト全体の検査（名前の重なり・飛び先）
- `release` との関係（前に挟むか、別のコマンドのままにするか）
- 文書（マニュアル・README・スキル）

## Out of Boundary

- ランタイムの挙動（読み込み・シーンの検索の規則・失敗の出し方）
- `release` の 5 段の中身（同梱バルーン・`updates.txt`・nar）
- エディタの言語機能の実装
- 文法そのものの変更

## Upstream / Downstream

- **Upstream**:
  - 構文を読む部品（`crates/pasta_dsl/src/parser/mod.rs` 142 行の `parse_file`、途中まで読めた分を返す `crates/pasta_dsl/src/partial.rs` 144 行の `parse_str_partial`）
  - 名前の規則（`pasta_core` の `sanitize_name` と別名表）
  - 完了した scene-name-alias（別名表の形と既定の表）
- **Downstream**:
  - scene-anchor-link: 飛び先の無いリンクの検出
  - failure-output-unification: 実行する前に見つける側の受け皿
  - リリースの手順: 常駐の release-workflow と、hello-pasta の `crates/pasta_sample_ghost/release.ps1`（181〜189 行が `pasta_check release` を呼ぶ）
  - 代用品の後片付け（`manual.yml`・`tutorial_stages_test.rs`）

## Existing Spec Touchpoints

- **Extends**: 完了した pasta-check・pasta-check-bundled-balloon（同じコマンドにサブコマンドを足す）
- **Adjacent**:
  - scene-anchor-link・failure-output-unification（上の先送り）
  - scene-name-alias（完了。「pasta_check は pasta.toml を読まない」という決定を、このspecで改める）
  - call-attribute-filter・scene-attribute-store（属性で絞る Call が入ると、飛び先の検査が属性も見ることになる）
  - getting-started-story-guide（マニュアルに章を足すと、`book/src/SUMMARY.md` と章の数を決め打ちした 2 つのテストが重なる。下の測定）
  - hello-pasta-shell-art（`Cargo.lock`）

## Constraints

- **`Cargo.lock` は 1 ウェーブに 1 spec**: 依存を足すと `Cargo.lock` が変わる。2026-10-10 のウェーブでは hello-pasta-shell-art が `Cargo.lock` を持つので、このspecは次のウェーブから始める。
- **`manual.yml`**: 同じウェーブに `.github/workflows/manual.yml` を触る spec がほかにあるときは、このspecは触らない。
- **`＠？シーン名` の検査**: このspecと scene-anchor-link のうち、後から入る側が足す。
- **問題があれば止める**: 作成ツールは、壊れた成果物を警告だけで通さない。誤りは失敗で終わらせる。
- **書いたとおりに読む**: 想定していない書き方を、意図の推測で救わない。
- **マニュアルが権威**: 挙動を足したら同じ変更でマニュアルを直し、`node book/tools/gen-skill-refs.mjs` でスキル references を再生成する。
- **配り方**: `pasta_check` は crates.io から `cargo install pasta_check` で入れる道具である。依存を増やすと、入れるときのビルドの重さと、公開の順番に効く（下の測定）。

## 2026-10-10 棚卸の測定（main add05022）

- **触るファイル**:
  - `crates/pasta_check/Cargo.toml`（29 行）、`crates/pasta_check/src/main.rs`（319 行）、新しいモジュールとそのテスト、`crates/pasta_check/tests/cli_test.rs`（223 行）、ルートの `Cargo.lock`。
  - `crates/pasta_check/README.md`（123 行）、`.claude/skills/pasta-check/SKILL.md`（153 行）と `references/`（今は `nar-spec.md`・`updates-txt-spec.md` の 2 つ。手で書いていて、生成の対象ではない）。
  - マニュアル: 新しいページ、`book/src/SUMMARY.md`（67 行）、`book/src/reference/pasta-toml.md`（540 行のうち 349 行の 1 文）。`pasta-toml.md` はスキル references の生成元（`book/tools/gen-skill-refs.mjs` 50 行）なので、再生成が要る。
  - 章を足すと、章の数 47 を決め打ちしたテスト（`book/tools/verify-scripts-test.mjs` 72 行、`book/tools/talk/talk-test.mjs` 453〜454 行）も直す。
  - steering: `.kiro/steering/tech.md`（71〜74 行の依存の一覧）、`.kiro/steering/structure.md`（189 行）。
  - 依存を足す場合: `.github/workflows/release.yml` 295 行。`pasta_check` の公開は今、先に公開するクレートの指定（`-DependsOn`）を持たない。公開の順番は `pasta_check` が最後なので、指定を足すだけで済む。
  - 1,000 行に近いファイルは `crates/pasta_check/src/balloon.rs`（1,070 行）だが、触らない。
- **規模**: 約 10〜14 タスク（入口と引数 1、辞書の集め方と pasta.toml 2、構文 1、モジュール名 1、照合用の名前 1、飛び先 2〜3、出力と終了コード 1、`release` との関係 0〜1、文書 2、テスト 1〜2）。Lua への変換まで確かめる・機械向けの出力も付ける、と決めると上の端を超える。
- **先に要るもの**: 機能としては無い。順番の制約は上の Constraints の 3 つ。マニュアルに章を足す場合は、getting-started-story-guide と同じウェーブに置かない（`SUMMARY.md` と、章の数のテスト 2 つが重なる）。
- **種別**: 機能（辞書を起動せずに検査する手段が無い）。
- **要件定義のモデル**: Fable（開発者の判断の分かれ道が多い。依存の重さ・誤りと注意の線引き・リリースとの関係が、配り方と作者の体験の両方に効く）。
- **要件定義の議題**:
  1. どれを誤りにし、どれを注意にするか。原則は「問題があれば止める」。ただし前方一致で候補が複数になるのは pasta の書き方そのものなので、重なりのうちどれを問題とするかを決める（照合用の名前に揃えて初めて重なるもの、モジュール名の重なり、`GLOBAL` と同じ名前のシーン、など）。
  2. 読むだけでどこまで飛び先を決められるか。前方一致、変数の Call（`＞＄変数`）、Lua で決まる飛び先（今のシーンの表・`act` の関数・`GLOBAL`・`scripts/` で足したシーン）、別名表を踏まえ、「無い」と言い切れる範囲を決める。言い切れないものを黙って通すか、注意として出すかも決める。
  3. `pasta_check release` がこの検査を関門として走らせるか。走らせるなら、誤りがあるとき nar を作らない。既定で走らせるか、指定したときだけかも決める。
  4. 依存の重さ。`pasta_dsl` と `pasta_core` だけにして構文と名前の検査にとどめるか、`pasta_lua` に依存して Lua への変換まで確かめるか。後者は `cargo install pasta_check` が同梱の LuaJIT をビルドするようになる。pasta.toml を読むには、`pasta_lua` の読み込み処理を使うか、`toml` に直接依存して読むかも決める。
  5. 出力の形。人が読む文章だけにするか、エディタや自動検査が読める形（1 行 1 件・JSON など）も付けるか。
  6. VSCode 拡張・エディタの言語機能（LSP）と検査を共有するか。共有するなら、検査の置き場所は WASM にビルドできるクレート（`pasta_dsl`・`pasta_core` の側）になり、`pasta_check` はそれを呼ぶだけになる。
  7. 辞書の集め方。pasta.toml の `[loader]` の `pasta_patterns`（既定は `dic/**/*.pasta`。`crates/pasta_lua/src/loader/config/mod.rs` 266〜297 行）に従うか、既定の場所だけを見るか。
  8. マニュアルに章を足すか。今は `pasta_check` の使い方が README とスキルにしか無い。章を足すなら、`release` の説明も同じ章へ移すか、検査のコマンドだけを書くかを決める。
- **見つけた穴・気を付けること**:
  - 手作りの検査（`tutorial_stages_test.rs`）には、hello-pasta に固有の規則が混ざっている（結合テストが足すシーンの接頭辞 `Kick`・`Gate`）。そのままコマンドの規則にはできない。`GLOBAL` と同じ名前（`yield`・`チェイントーク`・`ゴースト終了`・`close_ghost`。同じファイルの 363〜365 行）は、どのゴーストにも当てはまる。
  - `pasta_dsl`・`pasta_core`・`toml` はどれも、ほかのクレートがすでに使っている。足しても `Cargo.lock` に新しい部品は入らず、`pasta_check` の依存の一覧が変わるだけになる見込みである（ビルドでは確かめていない）。
  - スキル（`.claude/skills/`）の編集は、実装のときに開発者の許可が要ることがある。
