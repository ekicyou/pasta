# Implementation Plan

> 順は design.md の Implementation Order のとおり。読み込みの終わりの行（要件 10）を先にし、キックの Rust → キックの Lua → DAP の撤去 → 整え → テストの足し → マニュアル → 関門の順にする。どのタスクの終わりでも `cargo build --workspace` と、そのタスクが触ったテストが通る。cargo の前に環境変数 `NoDefaultCurrentDirectoryInExePath` を外す。同じウェーブの約束で `act.lua`・`scene.lua`・`search/`・`code_gen/element_gen.rs` には触らない。

- [ ] 1. 読み込みの終わりの行を、ログの設定によらず書く
- [x] 1.1 読み込みの終わりの行の target を決め、ログのフィルタがいつもその target の info を通すようにする
  - target の名前 `pasta::load_entry` を定数にし、`pasta_lua::logging` から外のクレートにも使える形で出す
  - フィルタの組み立ての最後に、その target の info を通す指示を足す（`PASTA_LOG`・`[logging]`・失敗のときの既定のどの枝でも）
  - フィルタの組み立てを、環境変数を読む所と、読んだ `PASTA_LOG` の値を引数で受けて組み立てる所に分ける（単体テストが環境変数に触らないため）
  - `tracing_init.rs` の単体テストで、`warn`・`PASTA_LOG` の値・作者の `pasta::load_entry=off` のどれでもその target の info が通り、ほかの target の info は通らないことを確かめる
  - 完了の姿: フィルタの単体テストが通り、ほかの target の絞り方の既存テストも通る
  - _Requirements: 10.2, 10.4_

- [x] 1.2 SHIORI の読み込みの入口 2 つが、その target で終わりの行を書くようにする
  - `windows.rs`・`v3.rs` の `SHIORI load entry completed` の行に target を付ける。文と `entry`・`loaded` の欄は変えない
  - `ffi_loadu_test.rs` に結合テストを足す: `[logging] level = "warn"` のゴーストと `filter = "pasta_lua=error"` のゴーストで、`pasta.log` に `SHIORI load entry completed`・`entry=`・`loaded=true` があり、`Startup sequence completed` などほかの info が無い。辞書の壊れたゴーストでは `loaded=false` の行がある
  - v3 の入口の同じ確かめを、新しい `crates/pasta_shiori/tests/v3_load_entry_log_test.rs` に 1 本足す（v3 の入口の読み込みのログを見るテストは今は無い）
  - 完了の姿: 2 つの入口の結合テストが通り、`pasta.log` に絞った設定でも終わりの行が 1 行ある
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 9.2_

- [ ] 2. キックの経路を撤去する（Rust）
- [x] 2.1 pasta_shiori のアクターと SHIORI からキックを外す
  - アクターのキックのメッセージとその腕、キックの送り先を作る関数、`PastaShiori` のキックの送り先の欄・設定の口・キックの関数と説明を消す
  - アクターのスレッドを起こす関数の引数からキックの送り先を外し（3 つにする）、スレッドは既定の `PastaShiori`、読み込みは既定の `RuntimeConfig` を使う
  - その関数を呼ぶテスト（キックと関係の無いものも）の引数を直す。アクターの単体テストのキックの分・テストのハーネスのキックの腕・`shiori_lifecycle_tests.rs` の `test_default_has_no_kick_sink`・`actor_thread_vm_test.rs` のキックの説明を消す
  - キックだけを確かめるテスト 6 本（`actor_kick_test`・`kick_unused_byte_invariant_test`・`scene_kick_*_e2e_test` の 4 本）を消す
  - 完了の姿: `pasta_shiori` にキックの型と関数が無く、`cargo test -p pasta_shiori` が通る
  - _Requirements: 2.1, 9.1_

- [x] 2.2 pasta_shiori の DAP のテストとフィクスチャを消す
  - `debug_preservation_test.rs` とそのフィクスチャ、`multi_instance_test.rs` の DAP のキックとリロードの 2 本、テストの共通部品の待ち受けのポートの関数を消す
  - `multi_instance_test.rs` のポートを確かめる 1 本は、9.3 で書き直すまで残す（DAP のポートを見なくなるよう最小に直す）
  - 完了の姿: `pasta_shiori` のテストから DAP のポートと要求を使う箇所が無くなり、テストが通る
  - _Requirements: 9.1_

- [ ] 3. キックの経路を撤去する（Lua）
- [x] 3.1 Lua のキックの部品を消し、時計のイベントの扱いをキックの無い形にする
  - `kick.lua` を消し、`entry.lua` からその require と `SHIORI.kick` を消す
  - 仮想ディスパッチャからキック割り込みの門とキック起動の分岐を消し、Status の抑止の門だけを残す
  - `store.lua` からキックの保留の印 2 つとその reset を消し、チェイントークの中断の説明からキックを外す
  - キックだけの Lua のテスト 8 本と一覧の行を消し、`chain_talk_interrupt_test.lua`（キックの mock を外し、中断を確かめる名前に直す）・`virtual_dispatcher_pause_gate_test.lua`・`actor_proxy_act_delegation_test.lua`・`recommend_menu_test.lua` のキックの行、`scene_alias_search_test.rs` の埋め込みの Lua のキックの分を直す
  - 完了の姿: `crates/` に `SHIORI.kick`・`kick_pending`・`kick_force` が無く、Lua のテストと luacheck が通り、自動トーク・時報・チェイントーク・一時停止・Status の抑止のテストが今のまま通る
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 9.1_

- [ ] 4. DAP を撤去する（pasta_lua）
- [x] 4.1 ランタイムから DAP の結線を外し、「デバッグが有効か」をシーンマップの有無にする
  - ランタイムの待ち受けの持ち物とフック・`jit.off()`・待ち受けを起こす呼び出しを消し、受けたシーンマップをそのまま持つ
  - 待ち受けのアドレスと提示モードの問い合わせを消し、「デバッグが有効か」の問い合わせはシーンマップを持つかを返す
  - `RuntimeConfig` のデバッグとキックの欄・設定の口・そのテストを消す。ファクトリの 2 回目の解決を消し、`@pasta_debug.enabled()` をシーンマップの有無から決める
  - `lib.rs` の DAP の型と関数の再輸出を消す
  - `scene_play_event_test.rs` の「待ち受けを塞いでも」の 1 本を、「有効ならシーンマップがある」の確かめに直す
  - 消した口を使うテストを同じタスクで消す: `tests/runtime/` の `debug_*`・`runtime_toggle_e2e_*` と `main.rs` の行とフィクスチャ 3 つ、`source_map_handoff_test.rs` の提示モードの 1 本（`tests/runtime/` は 1 つのバイナリなので、残すと DAP と関係の無いテストまでビルドできなくなる）
  - 完了の姿: `cargo test -p pasta_lua --no-run` と `cargo build --workspace` が通り、ランタイムに待ち受け・フック・`jit.off()` の呼び出しが無く、`debug_enabled()` と `@pasta_debug.enabled()` が同じ値を返すテストが通る
  - _Requirements: 1.1, 1.2, 7.1, 7.6, 9.1_

- [x] 4.2 DAP の部品とそのテストを消し、`OnPastaSceneAt` の位置の引きをパスだけを受ける形にする
  - `debug/` の DAP・セッション・ブレークポイント・フック・inspect・transport・wiring・型・エラー・ハンドル・キックのファイルと、その単体テストを消す。提示モードのファイルは設定がまだ使うので 4.3 で消す
  - 位置の引き（`playscene.rs`）からキックの組み立て・結果の型・URI の剥がし・パーセントの復号・ドライブの直しと `#![allow(dead_code)]` を消し、パスをゴーストのフォルダに結んで絶対にするだけにする（消すキックのファイルを import しているので同じタスクで直す）
  - 位置の引きの単体テスト: 相対・絶対・UNC のパスで引ける。`file:///…` と `%41` を含むパスは文字のまま扱われる。`scene_identity_index_test.rs` のキックの名前の組み立てを、位置の引きの形の確かめに直す
  - `final_regression_test.rs` の DAP の 2 本を消す。消した e2e の名残り（`tutorial_stages_test.rs` の接頭辞の表など）を、使い手が無くなれば消す
  - 完了の姿: `crates/` に `TcpListener`・`set_hook`・`jit.off`・`KickSink` が無く、`playscene.rs` に `allow(dead_code)` が無く、`cargo test -p pasta_lua` が通る
  - _Requirements: 1.1, 1.2, 1.4, 4.4, 7.1, 7.2, 9.1_

- [x] 4.3 デバッグの設定を「有効か」の 1 つにし、提示モードとサイドカーを消す
  - 設定の解決を、`PASTA_DEBUG` ＞ `[debug] enabled` ＞ false の 1 つの関数にし、引数 8 つの解決とテストだけのファイルからの解決を消す。値の読み方は今のまま。提示モードのファイルを消す
  - `[debug]` の設定の型を `enabled` だけにし、知らないキーを黙って読み飛ばす形にする（既定のポートの関数と再輸出も消す）
  - ローダの Phase 5.5 で解決を 1 回だけ呼び、シーンマップを作る関数にサイドカーの引数を渡さない。サイドカーのファイル・その再輸出・そのテスト（単体と `loader_source_map_build_test.rs` の 2 本）を消す
  - `config_test.rs` の提示モードの分とファイルからの解決のテスト（サイドカーの 2 本を含む）を消す
  - 解決の純粋な部分の単体テスト: 環境変数があれば勝つ・読めない値は無いのと同じ・既定は偽
  - 完了の姿: 古い環境変数・提示モードを読む箇所が無く、`.lua.map` を書く箇所が無く、単体テストと `cargo test -p pasta_lua` が通る
  - _Requirements: 3.1, 3.3, 3.4, 3.8, 7.1, 7.5, 9.1_

- [x] 4.4 DAP だけが使っていた依存を外す
  - ルートと `pasta_lua` の `socket2`、`pasta_lua` の `serde_json`、`pasta_shiori` の dev の `serde_json` を外し、`Cargo.lock` は cargo に作り直させる。ルートの `serde_json` は残す。依存を足さない
  - 完了の姿: `Cargo.toml` に `socket2` が無く、`cargo build --workspace --all-targets` が通る
  - _Requirements: 7.8_

- [ ] 5. 残す部品を整える
- [x] 5.1 (P) 位置からシーンを引く索引の、働きの無い呼び出しと隠す属性を消す
  - 索引の突合の働きの無い `insert_file` の呼び出しと、その上の誤ったコメントを消す
  - 索引の型のファイルの `#![allow(dead_code)]` を外し、使い手の無い項目を消す
  - 完了の姿: clippy が警告 0 で、`scene_identity_index_test.rs` が通る
  - _Boundary: debug::source_map の scene_join・scene_index_
  - _Requirements: 7.2, 7.4_

- [x] 5.2 (P) デバッグが有効なときの起動のログを 1 行出す
  - 読み込みを終えて有効なら、`Startup sequence completed` の直前に info で `Debug enabled: development events are allowed` を出す。読み込みが途中で失敗したときは出さない
  - `scene_play_event_test.rs` の結合テストで、有効なゴーストのログにその 1 行があり、無効なゴーストには無いことを確かめる
  - 完了の姿: そのテストが通る
  - _Boundary: loader_
  - _Requirements: 1.5_

- [x] 5.3 (P) テストの環境の中和から、意味の無くなったポートの行を外す
  - 4 つのクレートの共通部品と個々のテスト（`scene_identity_index_test.rs` を含む。5.1 はこのファイルを走らせるだけで書き換えない）の `#[ctor]` の中和から `PASTA_DEBUG_PORT` の行を外し、`PASTA_DEBUG`・`PASTA_LOG` は残す。説明の「固定ポート」を直す
  - 完了の姿: `crates/` のテストに `PASTA_DEBUG_PORT` を消す行が無く、`PASTA_DEBUG=1` を置いた環境でも `cargo test --workspace` が通る
  - _Boundary: 各クレートのテストの中和_
  - _Requirements: 9.3_

- [ ] 6. 保存変数を書き出す `OnPastaDumpVariables`
- [x] 6.1 保存変数の表を、セーブと同じ変換でテキストの TOML に書く関数を足す
  - セーブの門を通さず、表を変えず、`obfuscate` によらずテキストで書く。書けない値はセーブと同じ文言で外して警告する（警告をセーブと共有する）
  - 一時ファイルに書いて置き換える書き方で、書き込み・`sync_all` の失敗でも一時ファイルを消す（`save.toml` にも効く）。親のフォルダを作る
  - 単体テスト: テキストで書く・関数の値を外して警告する・在るファイルを置き換える・フォルダを作る・書けないときは `Err` で前のファイルが残り一時ファイルが残らない
  - 完了の姿: 単体テストとセーブの既存テストが通る
  - _Requirements: 5.1, 5.3, 5.4, 5.5, 5.6_

- [x] 6.2 `@pasta_debug` に書き出しを足し、3 つのイベントの門と既定のハンドラをそろえる
  - `@pasta_debug` に、保存変数の表を `profile/pasta/debug/variables.toml` に書く関数を足す。書けなければ警告を 1 行書き、エラーを Lua に上げない。有効かは見ない
  - `scene_play.lua` の門を、イベント名と後半の文言を受ける形にし、`OnPastaDumpVariables` の既定のハンドラ（先に登録があれば上書きしない・喋らない）を足す。`OnPastaScene`・`OnPastaSceneAt` の引数と応答は変えない。冒頭の説明に 3 つのイベントの共通の決まりを書く
  - 結合テスト: 有効なとき 204 で要件 5 の例のとおりに書く・2 回目で置き換わる・フォルダが無くても作る・`obfuscate = true` でもテキスト・`save.toml` を書かない・続きを変えない。無効・external なら 204 と警告でファイルを書かない
  - 完了の姿: 結合テストと `OnPastaScene`・`OnPastaSceneAt` の既存テストが通る
  - _Requirements: 4.1, 4.2, 4.3, 4.6, 5.1, 5.2, 5.5, 5.6, 9.2_

- [ ] 7. 止まるエラーの場所を正しくする
- [x] 7.1 読み込んだファイルの表に、チャンクの元のファイルを渡し、`.pasta` のパスを引く口を足す
  - ローダの文脈に、辞書のモジュール名 → 元のファイル（`.pasta` か `dic/` の `.lua` か、ゴーストのフォルダから見たパス）の表を足し、Phase 4 の後に詰める
  - 読み込んだファイルの表に、その表と「候補の全体のパス → `.pasta`」の表を持たせる。`@pasta_chunks` の値はこのタスクでは変えない（`stop.lua` と同じタスクで替える）
  - 候補の全体のパスから `.pasta` のパスを返す Rust の口を足す
  - 単体テスト: `.pasta` の候補にだけパスを返し、`dic/` の `.lua` の写し・作者の `.lua`・知らない候補には返さない。`dic/` の `.lua`（複数の行・CRLF を含む）の写しが元とバイトで同じテストを `lua_passthrough_test.rs` に足す
  - 完了の姿: 単体テストと既存の止まるエラーのテストが通り、変換したキャッシュの中身は変わらない（既存のバイト不変のテストが通る）
  - _Requirements: 3.5, 3.7, 6.3_

- [x] 7.2 `@pasta_chunks` の値を替え、止まるエラーの場所を元のファイルを確かめてから決める
  - `@pasta_chunks` の値を、`.pasta` のキャッシュは `{ pasta = <パス> }`、`dic/` の `.lua` の写しは元の `.lua` のパスの文字列、表に無いキャッシュは控えない、にする。パスの区切りは `\` にそろえる。値の 3 つの形と `scripts/` の `.lua` が今のままであることの単体テストを足す
  - `stop.lua` の場所の決め方を、値が文字列ならそのパスと行、表なら行の対応のファイルが同じときだけその行、ほかは場所を出さない、にする。比べる前に区切りをそろえる。引数・`X-Error-Reason`・対象・止まらない失敗の表記は変えない。`@pasta_debug` を読まない
  - `stop_test.lua` に、同じ `.pasta` の行を出す・別の `.pasta` なら番号が重なっても出さない・文字列の値はそのパスと行・`false`・表に無い・対応に無い行は出さない、を足す
  - 完了の姿: 単体テスト・Lua のテスト・luacheck と、今の `.pasta` の位置の結合テスト（`test_alias_scene_lua_block_error_shows_pasta_location`）が通る
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [x] 7.3 止まるエラーの場所の結合テストを足す
  - バルーンのフィクスチャに 2 つ目の `.pasta`（Lua のブロックの `GLOBAL` 関数で止まる）と `dic/道具.lua`（止まる関数）を足す
  - 1 つ目のファイルから 2 つ目の関数を呼んで止まると、場所を出さない（止まった行の番号が実行中のシーンの行の対応に入ることをテストの中で先に確かめる）。`道具.lua` の関数で止まると `dic\道具.lua:<行>:` が出る。今の `.pasta` の位置のテストは残す
  - 完了の姿: `runtime_error_balloon_e2e_test.rs` が通る
  - _Requirements: 6.1, 6.2, 6.3, 9.2_

- [ ] 8. ソースマップをシーンマップに縮める
- [x] 8.1 位置からシーンを引く索引の「チャンク → `.pasta`」を、読み込んだファイルの表から引く
  - 索引の突合が、関数の定義元のチャンクから `.pasta` のパスを返す関数を引数で受けるようにし、ファクトリが 7.1 の口を閉じた関数（ゴーストのフォルダに結んで返す）を渡す
  - 完了の姿: 2 つのファイルの同じ名前のシーンがそれぞれのファイルで引けるテストを含め、`scene_identity_index_test.rs` が通る
  - _Depends: 7.1_
  - _Requirements: 3.5_

- [x] 8.2 シンクとローダの Phase 5.5 を、シーンの宣言だけを控える形にする
  - シーンマップに、1 つの `.pasta` のシーンの記録を入れる口を足す。シンクを、シーンの宣言だけを控え、行の記録を捨てる形にする
  - ローダの Phase 5.5 は記録をシーンマップに入れるだけにする（チャンク名と行の付け替えを外す）。有効なときだけ作り、無効なら作らない
  - 行の引きを確かめていたテスト（`loader_source_map_build_test.rs` の 2 本・`source_map_handoff_test.rs` の 1 本・索引のテストの 2 本）を、シーンの記録と `scene_at` の確かめに直す。キャッシュのバイト不変のテストは残す
  - 完了の姿: 行の対応表がまだ型に在っても誰も書かず、直したテストと `scene_identity_index_test.rs` が通る
  - _Requirements: 3.5, 3.6, 9.1_

- [x] 8.3 シーンマップから行の対応表と逆引きを消す
  - シーンマップの型を、シーンの宣言の記録と索引だけにする。行の対応表・チャンクの表・逆引き・使い手の無い問い合わせ・`PastaPos`・隠す属性を消し、`.pasta` のパスの照合の正規化を 1 つの関数にする。手書きの `Clone` は使い手が無ければ消す
  - 行の対応表と逆引きだけのテスト（単体 3 つ・`chunk_name_validation_test.rs`・索引のテストの 1 本）を消す
  - 完了の姿: `crates/` に `ChunkSourceMap`・`resolve_lua_to_pasta`・`resolve_pasta_to_lua` が無く、`cargo test -p pasta_lua` と clippy が通る
  - _Requirements: 7.2, 7.3, 9.1_

- [x] 8.4 シーンマップの周りの説明を今の形に直す
  - `source_map/mod.rs`・`loader/source_map_build.rs`・`code_gen/source_map.rs`・`transpiler.rs` の説明から、行の対応表・逆引き・`LineShift` での付け替えの話を除き、シーンマップがシーンの宣言だけを控えることを書く。`transpiler.rs` の署名と `fill_line_maps` は変えない
  - 完了の姿: 4 つのファイルの説明に `ChunkSourceMap`・逆引き・`normalize_output` の誤った名前が無く、`cargo doc -p pasta_lua --no-deps` が警告無しで通る
  - _Requirements: 7.7_

- [ ] 9. どのタスクにも属さない結合テストを足す
- [x] 9.1 (P) リロードでの切り替え・古いキー・JIT を確かめる
  - 同じフォルダを `enabled = true` → `false` → `true` で読み直し、`debug_enabled()` と `OnPastaScene` の 200・204 が切り替わる
  - `[debug]` に `port = "x"`・`present_as = "lua"`・`source_map_sidecar = true` を書いても読み込みが通り、有効で、警告が無く、`.lua.map` が無い
  - 有効な VM で `jit.status()` が真
  - 完了の姿: `scene_play_event_test.rs` の足したテストが通る
  - _Boundary: pasta_lua の結合テスト_
  - _Requirements: 1.2, 3.2, 3.3, 3.6, 3.8, 9.2_

- [x] 9.2 (P) 古い環境変数を読まないことを、別のテストのバイナリで確かめる
  - 新しいテストのバイナリで、`PASTA_DEBUG`・`PASTA_LOG` を消し、古い環境変数 3 つを置いてから有効なゴーストを読み込み、`.lua.map` が無く、待ち受けのログが無いことを確かめる。環境を変えるのはこのバイナリだけ
  - 完了の姿: `cargo test -p pasta_lua --test debug_env_ignored_test` が通る
  - _Boundary: pasta_lua の新しいテストのバイナリ_
  - _Requirements: 3.4, 9.2_

- [x] 9.3 (P) デバッグを有効にした 2 体が並ぶことを確かめる
  - `multi_instance_test.rs` のポートの 1 本を、2 体とも `enabled = true` で読み込み、どちらも `OnPastaScene` に 200 で自分のシーンを返し、どちらのログにも有効の info の 1 行があり、待ち受けのログが無い、に直す。ログの量の確かめは残す
  - 完了の姿: `multi_instance_test.rs` が通る
  - _Boundary: pasta_shiori の結合テスト_
  - _Depends: 5.2_
  - _Requirements: 1.1, 1.3, 1.5, 9.2_

- [ ] 10. マニュアルとスキルの references を新しい形に揃える
- [x] 10.1 `internals/debug.md` を「開発用のイベントとシーンマップ」の章に書き直す
  - 門・`@pasta_debug` の 3 つの関数・シーンマップの作り方・止まるエラーの場所の決め方・`variables.toml` を書き、DAP・キック・行フック・提示モード・サイドカーの記述を残さない。`SUMMARY.md` の internals の行の題を合わせる
  - 消える利用者の頁へのリンク（`#初期モードの指定`・`#サイドカー出力任意`・`constraints.md`）を外す
  - 完了の姿: 章が書き直され、`SUMMARY.md` の題が合う
  - _Requirements: 8.1, 8.5_

- [x] 10.2 リファレンスの設定とイベントの頁を直す
  - `reference/pasta-toml.md` の `[debug]` 節を `enabled` だけにし、古いキーを読み飛ばすことと環境変数は `PASTA_DEBUG` だけであることを書く。`#提示モードの切替` へのリンクを外す。`[logging]` 節に、読み込みの終わりの行は `level`・`filter` によらず書くことを 1 行足す
  - `lua/shiori-events.md` に `OnPastaDumpVariables`（引数・書き出す先・応答・失敗）を足し、3 つのイベントの共通の決まりをまとめ、待ち受けと URI の話を除く
  - 完了の姿: 2 つの頁に DAP・ポート・URI の記述が無い
  - _Requirements: 8.2, 8.3, 10.2_

- [ ] 10.3 ほかの頁と `DEBUGGING.md` からキックと DAP の記述を消す
  - `internals/` の 8 頁、`reference/startup.md`、`reference/runtime-failures.md`（場所の表と、別のファイルの行を出さないこと・`dic/` の `.lua` はそのファイルと行で出ること）、`lua/modules/pasta-search.md`・`lua/patterns.md`・`grammar/call-jump.md`・`introduction.md`、ルートの `DEBUGGING.md` を直す。`@pasta_chunks` の値の説明も直す。利用者の章 `book/src/debug/` は触らない
  - 完了の姿: これらの頁に `kick`・DAP・9276 の記述が無く、`book/src/debug/` に差分が無い
  - _Requirements: 8.4, 8.5_

- [ ] 10.4 スキルの references を作り直し、マニュアルの検査を通す
  - `node book/tools/gen-skill-refs.mjs` で references を作り直す。書き込みが拒まれたら、入った分と残りを示して開発者に頼む
  - リンク・目次から数える章の数・references の drift の検査を通す
  - 完了の姿: マニュアルの検査が全部通る
  - _Requirements: 8.6, 8.7_

- [ ] 11. 品質の関門を通す
- [ ] 11.1 撤去の確かめと、ワークスペースの関門
  - `crates/` に `TcpListener`・`socket2`・`jit.off`・`set_hook`・`KickSink`・`kick_pending`・`SHIORI.kick`・`ChunkSourceMap`・`resolve_lua_to_pasta`・`resolve_pasta_to_lua` が残らないことを grep で確かめる。`act.lua`・`scene.lua`・`search/`・`code_gen/element_gen.rs`・`external_control.lua` に差分が無い
  - `cargo fmt --check`・`cargo clippy --workspace --all-targets`（警告 0）・`cargo test --workspace`・Lua のテスト・luacheck を通す
  - 完了の姿: 全部の関門が通る
  - _Requirements: 1.1, 1.2, 1.4, 2.1, 4.5, 7.1, 7.3, 9.4_

- [ ] 11.2 i686 のビルドとテストを通す
  - `i686-pc-windows-msvc` でビルドとテストを通す。32・64 ビットで期待が違うものが出たら `cfg(target_pointer_width)` で分け、省かない
  - 完了の姿: i686 の `cargo test --workspace` が通る
  - _Requirements: 9.5_

## Implementation Notes
- 1.1: 読み込みの終わりの指示を足すと max_level_hint が info より下がらないため、`tracing::enabled!(INFO)` は層ごとのフィルタでも true になる。ログの絞り込みを確かめるテストは `enabled!` でなく pasta.log の中身か DEBUG で見分ける。
- 2.1: `multi_instance_test::dap_kick_reaches_only_its_own_instance` は 2.1 の後 2.2 で消すまで赤い（キックの経路が無いため）。
- 4.1: `tests/fixtures/debug_break_coalesce.pasta` は `src/debug/wiring_pasta_break_coalesce_e2e.rs` が include_str! で読むため残した。4.2 で wiring と一緒に消す。`loader/source_map_build.rs` の `crate::debug::enable` への doc リンクと `tests/runtime/source_map_handoff_test.rs` 冒頭の「デバッグハンドル」の文言も 4.2/4.3 で直す。
- 4.2: `tests/chunk_name_validation_test.rs`（8.3 で消す予定）は jit.off・set_hook を使っていたため 4.2 で前倒しで消した。
- 6.1: `persistence::dump_table` に一時の `#[allow(dead_code)]`（shortcut: コメント付き）がある。6.2 で `@pasta_debug` に結んだら外す。
- 7.1: `searcher::pasta_file_of` に一時の `expect(dead_code)` がある。8.1 で使ったら外す。`@pasta_chunks` の値のパスはまだ OS の区切りのまま（`\` へのそろえは 7.2）。
- 8.1: `SourceMap::pasta_file_for_chunk` と `chunk_files` は本番から呼ばれなくなった（pub なので警告は出ない）。8.3 で消す。
- 8.2: `line_mapping_roundtrip_unchanged_with_index_present` と `src/debug/source_map_builder_tests.rs` は 8.3 を前倒しして消した（後者の削除は誤って 9.2 のコミット 86cb99d4 に入った）。`build_source_map` は `cache_manager` を受けない。
