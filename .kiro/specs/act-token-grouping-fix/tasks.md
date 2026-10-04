# Implementation Plan

- [x] 1. 変更前の基準線を取る
  - 環境変数 `NoDefaultCurrentDirectoryInExePath` を外したうえで、ワークスペース全体のテスト・clippy・luacheck を変更前のソースで走らせ、結果を記録する
  - `pasta_lua` のテストが `sample.generated.lua` を改行コードだけ書き換えた場合は元に戻す
  - Rust 側の E2E（`pasta_lua` の SHIORI テスト・`pasta_shiori`・`pasta_sample_ghost`）から、2 現象（先頭の表示制御・発言の後の `clear_spot`）に当たりそうな期待値の候補を書き出し、タスク 4 の判定材料にする
  - 変更前のソースで全テストが通り、luacheck が `0 warnings / 0 errors` であることを確認済みで、候補の一覧が残っている
  - 準備のタスクであり、要件は後続のタスク 4 で満たす

- [x] 2. グループ化の修正と CT の撤去
- [x] 2.1 グループ化の 2 か所を直す
  - `clear_spot` を最上位に置いたうえで現在のグループを閉じる。`spot` は最上位に置くだけでグループを閉じない現行のままにする
  - 現在のグループが無いときに積まれた表示制御など（`surface`・`wait`・`newline`・`clear`・`choice`・`choice_timeout`）を、アクター未指定のグループを開いてその内側に入れる
  - 「捨てる」と書いた既存のコメントを、アクター未指定として積んだ位置に出す規則の説明に置き換える
  - 組み立てと外見の記録（`sakura_builder`・`appearance`）は変更しない。変更が必要になったら設計に戻る
  - luacheck が `0 warnings / 0 errors` で、グループ化関数の複雑度が上限 15 以内（見込み 13）である
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4_
  - _Boundary: GroupByActor_

- [x] 2.2 (P) CT を撤去する
  - CT のモジュールと、その挙動を固定するテストを削除し、Lua テスト一覧から登録を外す
  - SHIORI 初期化のコメントから CT への言及を外す（挙動は変えない）
  - `crates/` 配下の検索で CT（`ct.lua`・`ct_test`・`require("ct")`）への言及が残っておらず、Lua テスト一覧に CT のテストの登録が無い（スイート全体の成否はタスク 4 で確かめる）
  - _Requirements: 4.1, 4.2, 4.3, 4.4_
  - _Boundary: CtRemoval_

- [x] 3. 新しい挙動をテストで固定する
- [x] 3.1 (P) グループ化の結果のテストを足す
  - 先頭の `wait` の後の発言、発言・`clear_spot`・`spot`・発言、`clear_spot` の後の `wait`、発言の途中の `set_spot` 単独の 4 件で、グループ化の結果の件数・アクター・内側のトークンを検証する
  - 4 件が通り、`set_spot` 単独の件は変更前と同じ結果を期待値にしている
  - _Requirements: 2.1, 2.2, 2.4, 2.5, 6.4_
  - _Boundary: ActGroupingTest_
  - _Depends: 2.1_

- [x] 3.2 (P) さくらスクリプトのバイト比較のテストを足す
  - 設計の表にある 13 件（`yield` 直後の先頭の表示制御、シーン冒頭の全種の表示制御、発言の無い出力、`＞ゴースト終了` 相当、`raw_script` との混在、先頭の `surface` の観測と復旧、`clear_spot` を挟む 4 パターン、`clear_spot` の後の `wait`、`set_spot` 単独）を追加する
  - 各テストで `STORE.actor_spots`・`STORE.appearance` を初期化し、本文に句読点を含めない
  - 実行結果が期待値と食い違ったら、期待値を合わせずに設計の規則のどこと食い違うかを調べ、規則どおりでなければ設計に戻る
  - 13 件がすべて通り、先頭の `surface(5)` がさくらのサーフェスとして記録されないことを検証している
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.6, 1.7, 2.1, 2.2, 2.3, 2.4, 2.5, 5.6, 6.1, 6.2, 6.3, 6.4_
  - _Boundary: ShioriActBytesTest_
  - _Depends: 2.1_

- [ ] 4. 修正対象以外の出力が変わらないことを確かめる
  - ワークスペース全体のテストを走らせる
  - 既存の期待値に差分が出たら、タスク 1 の候補と照らして 1 件ずつ見る。2 現象（先頭の表示制御・`clear_spot` の後の発言）に当たる差分は、根拠を残して期待値を更新する。当たらない差分が出たら設計に戻る
  - clippy と luacheck を走らせる
  - ワークスペース全体のテスト・clippy・luacheck がすべて通り、既存の期待値の変更は 2 現象に当たるものだけである
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 4.4, 6.5_
  - _Depends: 2.2, 3.1, 3.2_

- [ ] 5. マニュアルを新しい規則に合わせる
- [x] 5.1 (P) 公開 Lua API の章を直す
  - スクリプト API の表示制御・`choice`・`choice_timeout` の節から「発言より前に積むと出力されない」を削除し、アクター未指定として積んだ位置に出る規則と、発言者の表情は発言の中か後に書くことを書く
  - パターン集の「複数トークと表示制御」の説明を「誰のスコープにも付かない」に改め、作例は変えない。回避のための別の書き方は足さない
  - 2 つの章に「出力されない」という記述が残っていない
  - _Requirements: 5.1, 5.2_
  - _Boundary: ManualUpdate（lua/）_

- [ ] 5.2 (P) 内部設計の章を直す
  - トーク出力の章のグループ化の手順・結果の列の説明・不変条件を、`clear_spot` がグループを閉じること、先頭の表示制御をアクター未指定として積んだ位置に出すこと、`spot` がグループを閉じないこととそれで出力が変わらない理由に書き換える
  - 実行モデルの章の扱う事項・CT の節・ソースの所在と、内部設計の索引の表から CT を削除する
  - 呼び出し・ジャンプの章（`＞ゴースト終了`）は変更しない
  - 内部設計の章に「捨てる」「`clear_spot` はグループを閉じない」と CT への言及が残っていない
  - _Requirements: 5.3, 5.4, 5.6_
  - _Boundary: ManualUpdate（internals/）_

- [ ] 5.3 スキルを再生成して検査を通す
  - 生成ツールでスキル `references/` を再生成する（手で編集しない）
  - 鮮度チェック（`node book/tools/gen-skill-refs.mjs --check`）とリンク検証（`node book/tools/link-check.mjs`）が通る
  - マニュアル CI と同じ内容検査（`node book/tools/verify-content.mjs`。内部設計の索引のパス実在検査を含む）が通る
  - リポジトリ全体の検索で、CT への残る言及が `.kiro/specs/` の過去の記録とロードマップだけである
  - _Requirements: 4.1, 5.4, 5.5_
  - _Depends: 2.2, 5.1, 5.2_

## Implementation Notes

- タスク 1 の基準線（273fa925）: `cargo test --workspace` 105 スイート・2385 件すべて合格、`cargo clippy --workspace --all-targets` 警告 0、luacheck `Total: 0 warnings / 0 errors in 84 files`。テスト後に `crates/pasta_lua/profile/pasta/save/save.json` と `tests/fixtures/sample.generated.lua` が書き換わるので `git checkout --` で戻す。
- タスク 1 の候補: 2 現象に当たる既存の期待値は見つからなかった。最も近いのは `act_grouping_test.lua` の「トークン順序を保持する」（`clear_spot` の後は別アクターで、結果は変わらない）と `act_test.lua` の `surface` 単独 `build()`（結果の中身を検証していない）。`shiori_act_test.lua` などにある「先に `talk(…, "")` を積む」回避はそのまま通る。
- Lua テストランナーは最初に落ちたスイートで止まる。修正前の act.lua（273fa925）で RED を確かめるときは、後ろのスイートを一時的に外すか分けて走らせる。
- 並列負荷が高いと Rust の `debug::hook::tests::hook_panic_*` と `runtime_toggle_e2e_basic_test` の TCP テストが不安定に落ちることがある（本仕様と無関係。単独の再実行で通る）。
- タスク 5.1 で、設計の「書き換える内容」に無い `lua/script-api.md`「スポット操作」の箇条も直した。旧記述は `clear_spot` の前後で同じアクターの発言が古い立ち位置で出るという、要件 2.2 で直した不具合の挙動を書いていたため、`set_spot`（不変）と `clear_spot`（新しい立ち位置で出る）の 2 箇条に分けた。
- ビルド環境: C: の空きが少ないと `LNK1318`（PDB）やメモリ確保失敗でビルドが落ちる。`cargo test` は `-j 8` 程度に絞る。
