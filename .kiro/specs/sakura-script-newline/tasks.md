# Implementation Plan: sakura-script-newline

- [x] 1. 既存テストスイートの green 確認とベースライン記録
  - `sakura_builder_test.lua`（`lua_unittest_runner` 経由）、`shiori_act_test.lua`、`loader/startup_test.rs`、および `cargo test`（pasta_lua ワークスペース全体）を実行し、変更前に全スイートが green であることを確認する
  - 現行の先出し順序出力（`\n[N]` が `\p` の直前・離脱側スコープ末尾に出る）を変更前ベースラインとして記録し、以降の期待値更新の比較対象とする
  - Observable: 変更前の全既存テストスイートが green で完走し、先出し順序が現行のテスト期待値として確認できる
  - _Requirements: 7.1_

- [x] 2. 段落区切り改行の完全遅延状態機械を実装し、既存テスト期待値を更新する
  - `sakura_builder.lua` の `text_since_break`（グローバル bool）を `spot_has_text`（スポットID→bool のテーブル）と `pending_break`（単一 bool）へ置換する
  - `emit_actor_switch` をスポット解決（未設定→0＋警告ログの既存フォールバックは維持）と `\p[spot]` 出力のみに縮小し、改行の判定・出力は行わないようにする
  - ビルドループへ以下の状態遷移を実装する: アクター切替時に切替先スポットの has-text で pending をセット（旧 pending は暗黙破棄。先出し版の `last_spot ~= spot`／`last_spot == spot` 抑制ガードは復活させない）／非空 `talk` 出力の直前で pending が真なら `\n[math.floor(spot_newlines*100)]` を1回出力しフラッシュし、同時に該当スポットの has-text を真に設定／`clear`（`\c`）トークン処理時に現在スポットの has-text を偽へリセットし pending を破棄／`clear_spot` トークン処理時に has-text 全体と pending をリセット／ビルドループ終端で未フラッシュの pending を出力せず破棄してから `\e` を付与
  - `emit_inner_token`（talk 以外のトークン変換）、および `spot`／`clear_spot` による `STORE.actor_spots` 更新ロジックには一切手を加えない
  - `sakura_builder_test.lua` の先出し順序を前提とする既存アサーション8箇所（複合シナリオの `\n[150]` 存在確認、spot変更時の `\n[N]` 出力確認、改行キャンセルスイートの回帰ケース、先頭サーフェス手番ケース、さくらスクリプトのみ手番を挟むケース、統合シナリオの `\p[1]Kero speaks` 前改行確認、persist-spot-position の `\n[150]` 存在確認、string-buffer バイト一致テストの `\n[200]` 存在確認）を、完全遅延方式の新しい出力（多くは A→B→A 往復への書き換え、または改行ゼロへの反転）へ更新し、挙動が意図的に反転した箇所には理由を示すコメントを付す
  - Observable: `sakura_builder_test.lua` の全既存テストケースが、新しい完全遅延の出力順序を期待値として green で通過する
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 3.1, 3.2, 3.3, 3.4, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.1, 5.2, 5.3, 6.1, 6.2, 6.6, 7.2_
  - _Boundary: BUILDER.build ビルドループ, emit_actor_switch_

- [x] 3. 完全遅延方式の新規テストケースを追加する
- [x] 3.1 会話終端・往復・スポット共有シナリオのテストケースを追加する
  - A→B で終了するビルドの出力に `\n[` が1つも含まれないことを検証するケースを追加する
  - A→B→A 往復（異なるスポット）で `\p[0]A1\p[1]B1\p[0]\n[150]A2` と等価な順序になることを検証するケースを追加する
  - 同一スポットを共有する2アクターの交代（A→B、spot 0 共有）で `\p[0]A1\p[0]\n[150]B1` と等価な順序になり、先出し版の同一スポット抑制が適用されないことを検証するケースを追加する
  - `spot_newlines = 1.5 → \n[150]` および `spot_newlines = 2.0 → \n[200]` の算出を、A→B→A 往復シナリオで検証するケースを追加する
  - Observable: 追加した4ケースがすべて green で、会話終端・往復・スポット共有それぞれの完全遅延挙動が固定される
  - _Requirements: 2.6, 2.7, 2.8, 3.1, 6.6, 7.3_

- [x] 3.2 保留破棄・フラッシュ位置・クリア系のテストケースを追加する
  - has-text 済みスポットへ戻り surface のみで次のアクター切替が発生した場合、保留が破棄され切替先で改行条件が再評価されることを検証するケースを追加する
  - has-text 済みスポットへ戻り surface/wait のみでビルドが終端した場合、末尾に `\n[` が出力されず `\e` のみで終わることを検証するケースを追加する
  - 戻り手番の先頭に surface を挟んだ場合、`\n[N]` が `\p` 直後ではなく非空 talk の直前に出力されることを検証するケースを追加する
  - 切替グループが surface のみで、同一アクターの後続グループで初めて talk が出力される場合、その talk 直前でフラッシュされることを検証するケースを追加する
  - has-text 済み状態で `clear_spot` を処理した後、以後の切替で保留が発生しないことを検証するケースを追加する
  - has-text 済みスポットへ戻り pending がセットされた状態で `clear`（`\c`）を処理した場合、保留改行が出力されずに破棄され、当該スポットの has-text が偽へリセットされることを検証するケースを追加する
  - Observable: 追加した6ケースがすべて green で、pending の破棄・フラッシュタイミング・クリア系の完全遅延挙動が固定される
  - _Requirements: 1.3, 2.2, 2.3, 3.3, 4.4, 4.6, 7.3_

- [x] 3.3 バイト一致の回帰テストを往復シナリオへ拡張する
  - `\n[200]` を含む A→B→A 往復シナリオを新規に追加し、native バッファ経路と `buf.new_fallback` 経路で出力がバイト一致することを検証する
  - 既存の string-buffer バイト一致テスト（clear_spot 経路を検証するシナリオ）は削除せず、期待値のみ完全遅延方式へ更新して維持する
  - Observable: 新規往復シナリオと既存 clear_spot シナリオの両方で native/fallback のバイト一致が green で確認できる
  - _Requirements: 6.5_

- [x] 4. 統合テストを完全遅延方式へ書き換える
- [x] 4.1 (P) shiori_act_test.lua のスポット変更改行ケースを往復シナリオへ書き換える
  - `act:talk(sakura)` → `act:talk(kero)` → `act:talk(sakura)` の A→B→A 往復で、`\n[150]` が戻り手番（2回目の sakura 発話）の直前にのみ出現することを確認するよう既存ケースを書き換える
  - Observable: `shiori_act_test.lua` の該当ケースが往復シナリオの新期待値で green
  - _Requirements: 7.4_
  - _Boundary: shiori_act_test.lua_
  - _Depends: 2_

- [x] 4.2 (P) startup_test.rs の config 伝搬テストを往復シナリオへ書き換える
  - `test_shiori_act_uses_config_spot_newlines` を A→B→A へ書き換え、`spot_newlines = 2.0` 設定時に戻り手番で `\n[200]` が観測されることを確認し、config 値伝搬の検証を維持する
  - Observable: `startup_test.rs` の該当テストが往復シナリオの新期待値で green
  - _Requirements: 7.4_
  - _Boundary: loader/startup_test.rs_
  - _Depends: 2_

- [ ] 5. 全体回帰確認と実機 SSP 目視検証を実施する
- [ ] 5.1 実機 SSP で段落区切りの見た目を目視検証する
  - 実機 SSP 上で (a) A→B 終了トーク（両バルーンに空行が残らないこと）、(b) A→B→A 往復（異なるスポット、戻り側の段落先頭に約1.5行の区切りが入り修正前と同じ見た目であること）、(c) 同一スポットでの話者交代（段落区切りが入ること）の3項目を再生し確認する
  - Observable: 3項目すべてで意図した見た目（ゴミ改行なし・往復区切り維持・同一スポット区切りが新たに入る）が目視で確認され、チェックリストとして記録される
  - _Requirements: 6.3_

- [ ] 5.2 ワークスペース全体の回帰テストを実行する
  - `cargo test`（pasta_lua ワークスペース全体、`tests/sakura_script/*.rs` を含む既存 Rust 統合テスト一式）を実行し、段落区切り改行の位置変更以外の出力（`talk`/`surface`/`wait`/`newline`/`raw_script`/`choice`/`choice_timeout`/`sakura_script`/`yield` の変換結果、`spot`/`clear_spot` による `actor_spots` 更新、空 `grouped_tokens` に対する `\e` のみの出力）にリグレッションがないことを確認する
  - Observable: `cargo test --workspace` が全件 green で完走し、タスク1で記録したベースラインと比較して段落区切り改行の位置以外に差分がないことが確認できる
  - _Requirements: 6.1, 6.2, 6.4, 6.5, 7.4_

## Implementation Notes

- **検証コマンド**: cargo実行前に `unset NoDefaultCurrentDirectoryInExePath`（未設定だとmlua-sys/LuaJITビルドがexit101で死ぬ）。Luaユニット=`cargo test -p pasta_lua --test lua_unittest_runner`、loader統合=`cargo test -p pasta_lua --test loader`、全体=`cargo test -p pasta_lua`。
- **Task 1 ベースライン（変更前・先出し方式）**: Luaユニット 53スイート全 green、loader 140件 green（`test_shiori_act_uses_config_spot_newlines` の `\n[200]` 含む）。現行は `emit_actor_switch` 内で `allow_break and last_spot ~= nil and last_spot ~= spot` 判定により `\p[spot]` の**直前**（離脱側スコープ末尾）に `\n[N]` を先出しする。完全遅延方式ではこの位置が「切替先スポットの次の非空 talk 直前」へ移動し、A→B 単純シナリオでは改行ゼロになる。
