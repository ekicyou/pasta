# コード総合レビュー＆改善ループ 改善内容レポート（2026-06-12）

## サマリー

| 項目 | 値 |
|---|---|
| 実行期間 | 2026-06-10T20:55:19+09:00 開始 〜 2026-06-12 完了 |
| ベースラインコミット | `4027097`（ブランチ: impl/review-improvement-loop） |
| レビュー領域 | 31 領域（Rust 7 クレート〔pasta_lua は 13 サブ領域へ細分化〕・Lua 資産 7・VSCode 拡張 1・book/tools 3・横断 D7 1） |
| セル総数 | 64 |
| セル内訳 | **IMPROVED 64 / NO_CHANGE 0 / SKIPPED 0** |
| Riloop-Cell コミット数 | 64（全セルがコミットを伴う改善で終端） |
| Rust workspace テスト | 1510 passed → **1956 passed / 0 failed**（+446。pasta_sample_ghost の随伴削除 2 件を含む純増） |
| クレート別最終テスト数 | pasta_core 105・pasta_dsl 239・pasta_lua 1240・pasta_shiori 157・pasta_lsp 112・pasta_check 65・pasta_sample_ghost 38 |
| Lua spec スイート | 36 → **45 スイート**（lua_specs 計 598 テスト・runner 経由で workspace に内包） |
| VSCode 拡張テスト | **135 件全緑**（実モジュール直接テスト 38+8 件新設・ドリフト済みコピー型 29 件を統合削減） |
| book/tools テスト | 237 → **379 件**（core 104・highlight 206・bigram-index 69） |
| TEST_COVERAGE.md 台帳合計 | 1050+（陳腐化）→ **2091**（Rust 1956＋VSCode 135）へ全面同期 |
| lint 最終状態 | cargo clippy --all-targets 全クレート警告 0（pasta_lua の deny エラー 2 件含む 117 件超を解消）・luacheck 対象 0 警告/0 エラー・eslint 0（新規導入） |
| 監査最終状態 | cargo audit 脆弱性 0/警告 7→4（残は対応不能 transitive）・cargo deny check 全 ok・cargo machete 未使用依存 0・npm audit vscode 15→1（dev 専用）/book 0 |

全 64 セルが終端状態（matrix.md に PENDING なし）。スキップ（巻き戻し）発生ゼロ・ベースライン RED 発生ゼロで完走した。

---

## ① セル別実施結果（領域・次元・改善内容・コミット）

次元グループ: G1=テスト網羅（D1）/ G2=静的衛生（D4 lint＋D5 デッドコード）/ G3=ハードニング（D3 脆弱性＋D6 パニック経路）/ G4=簡素化（D2）/ G5=文書・依存整合（D7 領域分）。詳細所見は matrix.md の各セル Notes を正本とする。

| セル | 領域 | 次元 | 改善内容（要約） | コミット |
|---|---|---|---|---|
| 3.1 | pasta_core | G1 | テスト 38 件追加（67→105）— scene_registry/word_registry/scene_table/word_table/random/error の未テスト公開挙動を網羅 | `157c2cb` |
| 3.2 | pasta_core | G2-G5 | clippy 1 件解消・属性フィルタ重複ブロックを helper 抽出・README 旧 API 例/依存表是正・台帳同期（G3 はハードニング対象なしを証拠付きで確認） | `3caa55d` |
| 3.3 | pasta_dsl | G1 | テスト 30 件追加（206→236）— ParseError API・二項演算式 AST を初網羅。paren_expr 既知バグを発見（挙動保存により未修正・④参照） | `2013476` |
| 3.4 | pasta_dsl | G2+G3 | clippy 3 件解消・パニック 2 経路ハードニング（actors 採番オーバーフローの saturating_add 化・逆転 Span の OutOfBounds エラー化）回帰 3 件 | `451533e` |
| 3.5 | pasta_dsl | G4+G5 | 等価簡素化 6 件（bin_op_from_rule 抽出・build_left_assoc_expr 書換等）・README/台帳同期 | `bd2dae2` |
| 3.6 | pasta_lua-root | G1 | ルート層テスト 22 件追加（error/string_literalizer/config/context/transpiler の未到達分岐） | `43f62f6` |
| 3.7 | pasta_lua-root | G2-G5 | 境界内 clippy 21→0・依存全数確認・CrLf 設定の実挙動 doc 正直化・README ツリー同期（G3 はプロダクション 0 件確認） | `17140ef` |
| 3.8 | pasta_lua/code_gen | G1 | テスト 31 件追加（out_line 会計契約・element_gen/scope_gen の生成形状） | `3987868` |
| 3.9 | pasta_lua/code_gen | G2-G5 | clippy 2 件解消・expr_to_string 等 5 箇所重複一掃（-19 行）・doc 正直化・台帳同期 | `80c1481` |
| 3.10 | pasta_lua/enc-log-search | G1 | テスト 38 件追加 — Win32 変換正常/異常系・logging traversal 分岐・search コンテキスト Rust 直接層を初網羅 | `7d68a81` |
| 3.11 | pasta_lua/enc-log-search | G2-G5 | Win32 FFI 長キャスト 4 箇所を buffer_len_to_i32 ガード化（回帰 2）・or-pattern/parse_selector_args 集約・search doc 正直化 | `b4cd702` |
| 3.12 | pasta_lua/loader-core | G1 | テスト 27 件追加（discovery traversal ガード・error Display 網羅・package_path 生成） | `71cca99` |
| 3.13 | pasta_lua/loader-core | G2-G5 | Windows ジャンクション skip の挙動固定回帰テスト・module_key 重複抽出・虚偽 doc 是正 | `d3e3c96` |
| 3.14 | pasta_lua/loader-io | G1 | テスト 21 件追加（cache traversal/orphan 検出・config 厳格パース・md5 マーカー trim） | `36f310d` |
| 3.15 | pasta_lua/loader-io | G2-G5 | save_cache の字句比較トラバーサル素通り修正＋dic 接頭辞誤剥離修正（回帰 4）・clippy 5→0 | `ed3bf30` |
| 3.16 | pasta_lua/runtime | G1 | 統合テスト 28 件追加（exec 系公開 API・persistence 劣化動作・encoding エラー 3 分岐を初到達） | `80cc7ce` |
| 3.17 | pasta_lua/runtime | G2+G3 | log.rs の Lua 由来ホストスタックオーバーフロー abort を深さ事前ゲートで修正（回帰 2）・tests clippy 6→0 | `1aef35f` |
| 3.18 | pasta_lua/runtime | G4+G5 | 5 レベルログ関数を log_fn! マクロ統合（69→29 行）・README 虚偽記載（std_utf8）是正・深さ閾値の境界ペアテスト 2 件 | `4b45f00` |
| 3.19 | pasta_lua/sakura_script | G1 | エッジケース 21 件追加（tokenizer 孤立 `\`・classify 優先順位・wait_inserter flush 経路） | `6efed6a` |
| 3.20 | pasta_lua/sakura_script | G2-G5 | clippy 14→0・到達可能パニックゼロを 1,412 行調査で確認・table_to_widths 抽出・doc 補記 | `0a9acc9` |
| 3.21 | pasta_lua/debug-core | G1 | テスト 14 件追加（read_frame エラー 4 分岐・bind 失敗・DTO 往復） | `1f7c915` |
| 3.22 | pasta_lua/debug-core | G2-G5 | never_loop deny 解消・read_frame Content-Length 16MiB 上限ガード（DoS 対策・回帰 1）・skeleton 期 doc 同期 | `b090436` |
| 3.23 | pasta_lua/debug-dap | G1 | 文書化済み劣化経路へテスト 10 件追加（不正引数スキップ・poisoned lock fail-safe） | `839414b` |
| 3.24 | pasta_lua/debug-dap | G2-G5 | frameId 加算オーバーフローを saturating_add 化（回帰 1）・decode_request 集約（-35 行）・デッド関数 encode_scope 除去 | `2dddf21` |
| 3.25 | pasta_lua/debug-hook-inspect | G1 | テスト 10 件追加（panic 原因記録の優先順位 3 段・FrameInfo・範囲外 frame_level） | `64bfc99` |
| 3.26 | pasta_lua/debug-hook-inspect | G2-G5 | clippy 9→0・debug_assert 前の lua_settop 復元順序是正（release 挙動同一）・staged 期 doc 同期 | `cfc568f` |
| 3.27 | pasta_lua/debug-session | G1 | テスト 12 件追加（stop_loop 9 アームのルーティング・step_should_stop 判定行列） | `8ba43d6` |
| 3.28 | pasta_lua/debug-session | G2+G3 | クレート最後の never_loop deny を解消（--all-targets フル実行可能化）・variables_reference saturating_add 化（回帰 1） | `726a20f` |
| 3.29 | pasta_lua/debug-session | G4+G5 | pasta ゲート重複再計算の等価集約・陳腐化 doc 4 箇所同期 | `a39e22d` |
| 3.30 | pasta_lua/debug-source_map | G1 | テスト 11 件追加（空状態系・canonicalize・サイドカー I/O エラー経路） | `994873e` |
| 3.31 | pasta_lua/debug-source_map | G2-G5 | clippy 3→0・insert_chunk 再投入時の stale 逆引き残留を防御修正（回帰 1）・未来時制 doc 6 箇所同期 | `99f6da8` |
| 3.32 | pasta_lua/debug-wiring | G1 | テスト 13 件追加（bridge ライフサイクル 3 終端条件・poisoned adapter 耐性・切断時 flush） | `92ad69a` |
| 3.33 | pasta_lua/debug-wiring | G2+G3 | wiring clippy 5→0 → **pasta_lua クレート全体 clippy --all-targets 警告 0 達成**・G3 到達可能パニック 0 を全数調査で確認 | `a2b9fe1` |
| 3.34 | pasta_lua/debug-wiring | G4+G5 | attach_pasta_resolver の単一 lock サイト化・staged 期 doc 4 箇所同期 | `bea013f` |
| 3.35 | pasta_shiori | G1 | テスト 28 件追加（120→148）— 完全未テストだった windows.rs FFI 層を MockShiori でインプロセス検証・error/hglobal/lua_request 初到達 | `e59940f` |
| 3.36 | pasta_shiori | G2 | clippy 12→0（--fix 機械適用）・stale allow 除去・未使用 pub ゼロを grep 実証 | `6ceb404` |
| 3.37 | pasta_shiori | G3 | **FFI/SHIORI 境界の脆弱性 6 件ハードニング**（②参照・全件 RED 実証付き）テスト +9（148→157） | `81dfe7f` |
| 3.38 | pasta_shiori | G4 | cache_lua_functions の 3 連同型 match 集約（-26 行）等 4 件・cargo fmt 正規化 | `d3a9e53` |
| 3.39 | pasta_shiori | G5 | README 依存表/アーキ図/公開 API 同期・「FFI 境界の安全性」節新設・台帳同期・SJIS テストのロケール非依存化 | `fae5efe` |
| 3.40 | pasta_lsp | G1 | テスト 18 件追加（92→110）— 完全未テストだった変数代入（var_set）トークン生成 約 340 行を網羅 | `dffbb01` |
| 3.41 | pasta_lsp | G2-G5 | 逆転 range のスライスパニックを no-op 化（回帰 1）＋敵対コーパス 40 種の no-panic 恒久化・visitors 3 連重複集約・serde_json を dev へ移動・README/台帳同期 | `e008091` |
| 3.42 | pasta_check | G1 | テスト 32 件追加（31→63）— CLI 終了コード契約・うるう年分岐・MD5・NAR 往復 | `145cd82` |
| 3.43 | pasta_check | G2-G5 | NAR の ZIP「..」検査を release ビルドでも常時有効化（debug_assert→実行時エラー・回帰 2）・README 新設（7 クレート中唯一不在だった） | `a02c57f` |
| 3.44 | pasta_sample_ghost | G1 | テスト 13 件追加（27→40）— CLI 契約・9 表情の描画相互相違・再実行冪等性 | `392c5b0` |
| 3.45 | pasta_sample_ghost | G2-G5 | publish=false 確認の上で未使用 GhostConfig/ConfigError/_config 引数を除去（随伴テスト 2 件削除・根拠記録済）・同型ループ統合（出力 19 ファイルのバイト一致証明）・release.bat 幽霊参照是正（G3 は NO_CHANGE） | `298c240` |
| 3.46 | lua-pasta_scripts-core | G1 | lua_specs 8 スイート 81 テスト追加（ct/config/scene_registry/word_builder/actor/act 等の未到達網羅）。ct.lua の利用不能バグ発見（④参照） | `ef44bab` |
| 3.47 | lua-pasta_scripts-core | G2-G5 | **luacheck CLI を dev 導入**（⑤参照）し境界 14 ファイル 0 警告化・find_act_handler 重複抽出（複雑度 19→15 以下）・ct.lua を既知負債として記録維持（G3 は NO_CHANGE） | `e63b920` |
| 3.48 | lua-pasta_scripts-shiori | G1 | lua_specs 15 件追加（SHIORI.entry・xpcall 境界・close_ghost・EVENT.fire×CALLBACK 統合・sweep 分岐の初到達） | `f5db29b` |
| 3.49 | lua-pasta_scripts-shiori | G2-G5 | try_route の reference 欠落 nil 添字ガード（回帰 1）・sweep ヘッダーを X-Error-Reason 正準綴りへ統一（回帰 2）・複雑度超過 3 件解消・doc 同期 6 件 | `9179282` |
| 3.50 | lua_specs-act | G2+G4+G5 | act 系 10 ファイル luacheck 0/0 化・重複セットアップの等価 helper 統合（G1/G3 は N/A — テスト資産） | `048546e` |
| 3.51 | lua_specs-runtime | G2+G4+G5 | runtime 系 luacheck 26 警告→0・同型 setup の helper 統合・放棄済み設計試行の死コード除去 | `5b31c31` |
| 3.52 | lua_specs-property-global | G2+G4+G5 | property/global 系 luacheck 5→0・helper 統合（diff -152 行・テスト 74 件不変） | `b5d5cb4` |
| 3.53 | lua_specs-persist-dispatch | G2+G4+G5 | persist/dispatch 系 luacheck 22→0・virtual_dispatcher_spec の helper 統合（-50 行・テスト 101 件不変） | `c7c654a` |
| 3.54 | lua_specs-sakura-shiori | G2+G4+G5 | sakura_builder/shiori_act の同型プレリュード統合（-115 行・expect 行は全件バイト一致） | `5267e38` |
| 3.55 | vscode-extension | G1 | 休眠モック初配線による実モジュール直接テスト 38 件新設・錆びた期待値（tmGrammar 等）のベースライン RED を修復（計 152 件） | `363c557` |
| 3.56 | vscode-extension | G2+G3 | **eslint＋typescript-eslint を導入**（⑤参照）し 37 エラー→0・resolvePort 検証ガード（回帰 5）・wasm 境界の型ガード（回帰 3） | `a22e70a` |
| 3.57 | vscode-extension | G4+G5 | ドリフト済みコピー型テスト 29 件を実モジュールテストへ統合（カバレッジ純増・計 135 件）・README トークン 17 種同期 | `c49853f` |
| 3.58 | book-tools-core | G1 | drift-check/tutorial-check へ 38 件追加・verify 系スモークテスト新設（49→97） | `e19a71a` |
| 3.59 | book-tools-core | G2+G3 | drift-check リンク検証の isWithinRoot パストラバーサルガード（回帰 5）・tutorial-check の単独 CR 正規化対称化（回帰 2）・デッドコード除去（lint は N/A — 基盤なし） | `38ddcec` |
| 3.60 | book-tools-core | G4+G5 | verify-static の自前 tmpdir 実装を os.tmpdir() へ置換・恒等冗長判定の削除・ヘッダ doc 9 本照合 | `8f6f56d` |
| 3.61 | book-tools-highlight | G1 | 4 スイートへ未到達分岐テスト 50 件追加（152→202 — neutralizer フォールバック・tokenizer 未終端ブロック等） | `ba12857` |
| 3.62 | book-tools-highlight | G2-G5 | 同型 clamp 関数統合（-20 行）・junction 走査境界の固定テスト・未消費 default export 除去・PASTA_BLOCK_RE doc 正直化 | `c3dbda4` |
| 3.63 | book-tools-bigram-index | G1-G5 | **Windows で CLI 既定パスが壊れる欠陥を修正**（fileURLToPath 化・回帰 1）＋テスト 33 件追加（36→69）・デッドコード 2 件除去・doc 正直化 | `ec3e2fc` |
| 3.64 | global（横断 D7） | D7 | imageproc 0.26.2 更新（RUSTSEC 3 件解消）・cargo-deny/cargo-machete 導入＆全 ok・npm audit fix 14 件・TEST_COVERAGE.md 全面同期（合計 2091）・drift-check/tutorial-check OK | `5391322` |

---

## ② 許容した挙動変化（攻撃面ハードニング）と境界回帰テスト

R3.2/R3.3/R3.6/R3.7 に基づき、不正入力・攻撃面に限って挙動変化を許容した全件。**正常系（妥当な入力）の外部観測挙動は全件で厳密保存**（各セルのコミット前全体検証 GREEN＋insta スナップショット不変が証拠）。各件とも変化の境界を固定する回帰テストを追加済み。

### Rust クレート

1. **pasta_dsl — actors 採番の整数オーバーフロー**（3.4 / `451533e`）: `＝4294967295` 入力で debug ビルド panic → saturating_add 化。境界回帰 2 件（u32::MAX 明示番号・MAX 後の暗黙採番の飽和）。
2. **pasta_dsl — 逆転 Span のスライス panic**（3.4 / `451533e`）: start_byte>end_byte の Span で extract_source が panic → OutOfBounds エラー化。回帰 1 件。
3. **pasta_lua/encoding — Win32 FFI 長キャストの負値ラップ**（3.11 / `b4cd702`）: `len as _` 4 箇所が 2GiB 超で負値化（境界外読取の恐れ）→ buffer_len_to_i32 ガード（Err(InvalidInput)）。回帰 2 件（0/1/i32::MAX 受理・i32::MAX+1/u32::MAX 拒否）。
4. **pasta_lua/loader-io — save_cache のパストラバーサル素通り**（3.15 / `ed3bf30`）: 字句比較ガードを `dic/../../../evil.pasta` が素通りし cache_dir 外へ書込可能（RED 実証）→ ParentDir コンポーネント検査へ強化。回帰 `test_save_cache_rejects_parent_traversal_in_relative_source`。
5. **pasta_lua/loader-io — dic 接頭辞の非コンポーネント境界剥離**（3.15 / `ed3bf30`）: `dictionary.pasta`→`pasta.scene.tionary` 誤変換 → strip_component_prefix 化。回帰 3 件。正常系（dic/ 配下）は既存 28 テストで不変確認。
6. **pasta_lua/runtime — log の Lua 由来ホストスタックオーバーフロー abort**（3.17・3.18 / `1aef35f`・`4b45f00`）: 深い入れ子テーブルのログ出力で STATUS_STACK_OVERFLOW（SHIORI 境界越し abort＝R3.7）→ 深さ事前ゲート＋tostring フォールバック。回帰 2 件（20 万深入れ子・自己参照循環）＋深さ 10/11 境界ペア 2 件（tests/runtime/runtime_api_test.rs）。
7. **pasta_lua/debug-core — Content-Length 無制限アロケーション DoS**（3.22 / `b090436`）: TCP デバッガクライアント制御長の vec 事前確保 → 16MiB 上限で InvalidData 化。回帰 1 件（上限ちょうど非発火・上限+1 拒否）。
8. **pasta_lua/debug-dap・debug-session — frameId 加算オーバーフロー**（3.24・3.28 / `2dddf21`・`726a20f`）: 信頼境界外 frameId=u32::MAX で debug ビルド panic → saturating_add(1) 化。回帰各 1 件（RED「attempt to add with overflow」実証）。
9. **pasta_lua/debug-source_map — 同一チャンク名再投入の stale 逆引き残留**（3.31 / `99f6da8`）: 公開 API 経由の再投入で旧エントリが逆引きに残留 → contains_key ゲートで retain 掃除（正常系＝loader の 1 chunk 1 回投入は厳密不変）。回帰 1 件。
10. **pasta_shiori — FFI/SHIORI 境界 6 件**（3.37 / `81dfe7f`・全件 RED 実証・回帰 9 件）:
    - parse1 のヘッダ数比例再帰 → 反復化（Reference 5 万件で 0xc00000fd を実証していた SSP 巻き添え abort を解消。tests/lua_request_test.rs stack_safety_tests）
    - RawShiori::{load,request,unload} へ catch_unwind 導入（FFI 境界越えパニック＝UB を SHIORI エラー契約へ縮退: load→false / request→500 / unload→true）
    - extern ガードパス 3 経路の HGLOBAL リーク解放（戻り値不変・GMEM_MOVEABLE プローブで must-free を固定）
    - hglobal GlobalAlloc null 未チェック（null が from_raw_parts_mut へ流入する UB）→ alloc_global ヘルパー＋Err 化
    - windows_api.rs の長キャスト 4 箇所を buffer_len_to_i32 ガード化（pasta_lua 3.11 と同型）
    - 毒化 Mutex/未初期化 early-return の残存リーク 3 経路を capture-before-lock 再順序化で解消
11. **pasta_lsp — 逆転 range のスライス panic**（3.41 / `e008091`）: LSP 契約違反の start>end range で replace_range が panic → no-op 化。回帰 `test_incremental_change_inverted_range_is_noop`＋敵対コーパス 40 種の no-panic 契約を `tests/analyze_robustness_test.rs` で恒久化。
12. **pasta_check — NAR の ZIP パストラバーサル検査が release で無効**（3.43 / `a02c57f`）: `..` 検査が debug_assert のみで release ビルド素通り（RED 実証）→ ensure_no_parent_component で常時実行時エラー化。回帰 2 件（コンポーネント単位拒否・`..foo` 等の部分一致は受理）。

### Lua / TypeScript / JS 資産

13. **pasta_scripts/shiori — try_route の reference 欠落 nil 添字**（3.49 / `9179282`）: reference 欠落 req で待機コルーチンが孤児化（RED 実証）→ `req.reference or {}` ガード。回帰 1 件。
14. **pasta_scripts/shiori — sweep タイムアウト 500 のヘッダー正準化**（3.49 / `9179282`）: X-ERROR-REASON/X-Error-Reason の綴り不一致を res.lua 正準（RES.err 経路）へ統一・空文字列 on_timeout の正規化。回帰 2 件（plain find による厳密ケーシング固定）。X-*-Reason はベースウェア非消費の診断ヘッダーで正常系影響なし。
15. **vscode — デバッグポート検証**（3.56 / `a22e70a`）: `Number(config.port)` 素通しで NaN/0/-1/65536/12.5/true 等が記述子化 → resolvePort ガード（整数 1..65535・違反は既定 9276 へフォールバック）。回帰 5 件（debugAdapterFactory.test.ts・RED 4 件実証）。
16. **vscode — wasm 境界の型ガード**（3.56 / `a22e70a`）: `as WasmAnalysisResult` の盲信キャスト → isWasmAnalysisResult 検証で不正形を既存の「WASM analysis failed」例外経路へ即時収束（従来は遠隔 TypeError）。回帰 3 件（vscodeModules.test.ts）。
17. **book/tools/drift-check — リンク検証のパストラバーサル**（3.59 / `38ddcec`）: `..` を含むリンクが repoRoot 外実在ファイルへ解決されると素通り（リポジトリ外の存在プローブが可能・RED 実証）→ isWithinRoot ガードで実在有無にかかわらず broken 報告。回帰 5 件（drift-check-test B-14 系）。
18. **book/tools/tutorial-check — 単独 CR 正規化の非対称**（3.59 / `38ddcec`）: 旧 Mac 改行で drift-check と比較結果が割れる → `/\r\n?/g` へ対称化。回帰 2 件。
19. **book/tools/bigram-index — Windows での CLI 既定パス欠陥**（3.63 / `ec3e2fc`）: `new URL().pathname` 直接 resolve で `C:\C:\...` ENOENT exit 1（RED 実証）→ fileURLToPath 化（POSIX/CI は同一挙動・明示引数経路は不変）。回帰 1 件。

補足（挙動変化なしの安全是正）: 3.26 の debug_assert 発火順序是正（lua_settop 復元先行）は release 挙動同一・debug ビルドの不変条件違反時のみの差異。3.49 の get_property は二重不正呼出時のエラー選択順のみ変化（単一不正系・正常系は不変）。

---

## ③ スキップ一覧と理由

**スキップなし。** 全 64 セルがデバッグ非収束による巻き戻し（SKIPPED）に至ることなく終端した。kiro-debug の 2 ラウンド上限到達・`_Blocked:` 残置・巻き戻し手順の発動はいずれも発生していない（matrix.md に SKIPPED 行ゼロ・全コミットの porcelain クリーン推移が証拠）。

なお実行学習メモに記録のとおり、G2-G5 セルのサブエージェント応答切れが 2 件（3.5・3.11）発生したが、いずれも完了再派遣で回収済み（作業健在・スキップ非該当）。

---

## ④ 確認済み（改善不要）一覧

**セル単位の NO_CHANGE はゼロ**（全 64 セルが改善コミットを伴う IMPROVED で終端）。したがって R7.5 の「NO_CHANGE セル一覧」は空である。ただし複数次元を束ねたセル内で**次元単位の「点検の結果改善不要」**が証拠付きで記録されており、改善実施分と区別して以下に列挙する。

### 次元単位の確認済み（改善不要）記録

| セル | 領域 | 次元 | 点検結果（証拠は matrix.md Notes） |
|---|---|---|---|
| 3.2 | pasta_core | G3 | プロダクションの unwrap/expect 0 件・到達不能不変条件のみ → ハードニング変更 0 件 |
| 3.7 | pasta_lua-root | G3/G4 | パニック経路 0 件・過剰複雑化なし → 変更 0 件 |
| 3.9 | pasta_lua/code_gen | G3 | unwrap/expect/添字 0 件（grep 全数調査）→ 変更 0 件 |
| 3.20 | pasta_lua/sakura_script | G3 | 1,412 行調査で到達可能パニック経路ゼロ → 変更 0 件 |
| 3.33 | pasta_lua/debug-wiring | G3 | 本番部に panic 系構文 0 件・毒化/チャネル全処理済 → ハードニング対象なし |
| 3.43 | pasta_check | G4 | 線形 5 ステップパイプライン・重複/過剰抽象なし → 是正 0 件 |
| 3.45 | pasta_sample_ghost | G3 | **NO_CHANGE**（攻撃面: ローカル生成 CLI のみ・symlink スキップ既存・unsafe なし） |
| 3.47 | lua-pasta_scripts-core | G3 | **NO_CHANGE**（FFI/再帰/非有界アロケーションなし・エラー経路は mlua 保護実行で Result 化済） |
| 3.62 | book-tools-highlight | G3 | ハードニング挙動変更 0 件（junction 非追従を実験実証し挙動固定テストのみ追加） |
| 3.50〜3.54 | lua_specs 5 領域 | G1/G3 | **N/A**（テスト資産自体 — 網羅性所掌は対応ソースの G1 セル・FFI/攻撃面なし） |
| 3.59/3.62/3.63 | book/tools 3 領域 | D4 lint | **N/A**（lint 基盤なし — book/package.json に lint スクリプト・eslint 設定とも不存在を新鮮確認・導入せず） |
| 3.60 | book-tools-core | G5（book/tools README） | tools/ 直下 9 本のヘッダ doc が実装と一致 → 是正なし |

### 要注意 — 既知負債として記録した「改善不要・未修正」判断

T1 ベースラインで負債が確認された領域（clippy 117 件・pasta_lua expect 346 箇所）の lint・パニック経路はループ内で全数処理済み（全クレート clippy 0 達成）のため、design.md が想定した「既知負債領域の NO_CHANGE 要注意」該当セルは存在しない。一方、**点検で発見しつつ挙動保存制約・境界制約により意図的に未修正とした既知負債**は以下のとおり。次回ループまたは個別フォローアップの入力とすること。

1. **pasta_dsl paren_expr バグ**（3.3 発見・3.4/3.5/3.40 で維持確認）: `parse_action.rs` の paren_expr 分岐が括弧内の最初の term のみ AST 化 — `（１＋２）＊３` の `＋２` が無音欠落。挙動保存により未修正・特性化テストで現挙動を固定済み（上流修正時に期待値更新）。**開発者フォローアップ要**。
2. **pasta_dsl 深い括弧ネストのスタックオーバーフロー DoS**（3.4 申し送り）: 深さ 1000 で STATUS_STACK_OVERFLOW を実証。pest 2.x に深さ制限 API がなく本ループの制約下では修正困難。悪意ある辞書ファイルの攻撃面として**開発者フォローアップ推奨**。
3. **ct.lua 死蔵モジュール**（3.46 発見・3.47 記録）: IMPL.__index 未設定で obj:defer/cancel 呼出不能＋`<close>` LuaJIT 非対応＝事実上利用不能。zip 出荷物の外部公開面につき保守的維持・現行挙動を ct_test.lua で文書化済み。
4. **shiori/init.lua 等の死蔵モジュール**（3.49）: リポジトリ内 require 0 件。zip 出荷物の公開面として維持し、維持根拠をファイル内へ明文化済み。
5. **callback.sweep の _staged 未消費残留**（3.49）: on_timeout=nil 経路で resume 先が再 stage_pending すると残留（現行コンシューマでは非到達・修正は構造変更要）。
6. **res.lua RES.build の pairs 順序非決定**（3.48/3.49）: プロトコル無害と判断し記録のみ。
7. **frozen-hljs strict TypeError**（3.61/3.62）: neutralizer の凍結 hljs 代入時 TypeError は head.hbs インラインミラー（境界外）の同期必須につき既知負債として維持（実環境 hljs は非凍結・preventExtensions 耐性はテスト固定済）。
8. **isCjk 拡張 B 到達不能分岐**（3.63）: コード単位走査により CJK 拡張 B（0x20000-）分岐へ到達せず 1 語トークン化。head.hbs ミラーと同一規則で索引/クエリ整合しており実害なし・特性テスト固定済み。
9. **X-ERROR-REASON の Rust 側旧綴り**（3.49 学習メモ）: `pasta_shiori/src/error.rs:83,94` は旧綴りのまま（パーサーは大文字小文字非区別で実害なし）。リポジトリ全体統一は本ループ外。
10. **cargo audit 残 4 警告**（3.64）: core2 0.4.0（unmaintained＋yanked）・paste 1.0.15（unmaintained）・rand 0.9.2（unsound）— 全て pasta_sample_ghost の画像処理 transitive（rav1e/imageproc 経由）で修正版なし＝対応不能と判断し記録。
11. **esbuild <=0.24.2 moderate**（3.56/3.64）: 修正は ^0.25/0.28 メジャー昇格要・dev ビルドツール限定につき見送り。将来の依存更新サイクルへ。
12. **vscode CHANGELOG.md の 0.2.x 欠落**（3.57）: package.json 0.2.2 と乖離。遡及記載は履歴捏造リスクにつき未実施 — release-workflow で要検討。
13. **lua_specs/README.md の陳腐化**（3.50〜3.54）: `*_spec.lua`/transpiler_spec の実在しない記載。3.64 の境界外につき Task 5（ドキュメント整合）所掌へ申し送り。
14. **workspace 全体の rustfmt 乖離**（3.24/3.33/3.38）: pasta_shiori・pasta_lsp は本ループで fmt クリーン化済みだが、pasta_lua 等に既存ドリフト残存（プロジェクトに fmt ゲートなし）。
15. **公開 API 面の未使用 pub**（3.2/3.7/3.13/3.31/3.41 等で反復確認）: crates.io 公開クレート（publish=true）の workspace 内未使用 pub シンボルは semver-major 破壊回避のため全維持（将来メジャーバージョン時の整理候補として各 Notes に記録）。

---

## ⑤ 導入したツールと環境変更の記録

### dev 環境への導入（リポジトリ非変更 — D-6 ポリシー）

| ツール | バージョン | 導入セル | 経路・備考 |
|---|---|---|---|
| Lua（luacheck 前提） | 5.4.6 | 3.47 | winget `DEVCOM.Lua`（lua.exe＋luarocks） |
| luacheck | 1.2.0 | 3.47 | MSVC vcvars64 下で `luarocks install luacheck`（argparse 0.7.2・luafilesystem 1.9.0・`%APPDATA%\luarocks` 配下）。`--config crates/pasta_lua/.luacheckrc` で運用 |
| cargo-deny | 0.19.8 | 3.64 | `cargo install`。`cargo deny check` = advisories/bans/licenses/sources 全 ok（deny.toml 変更不要） |
| cargo-machete | 0.9.2 | 3.64 | `cargo install`。未使用依存 0（全 workspace） |

既存確認済みツール（Task 1 プリフライト）: cargo 1.96.0 / clippy 0.1.96 / cargo-audit 0.22.2 / npm 11.16.0 / node v25.0.0 / wasm-pack（VSIX 用・導入済み）。

### リポジトリへの変更を伴う導入・更新

| 変更 | セル | 内容 |
|---|---|---|
| eslint＋typescript-eslint | 3.56 | eslint@10.4.1・typescript-eslint@8.61.0 を editors/vscode の devDependencies へ追加・`eslint.config.mjs` 新設（tseslint recommended・テストのみ no-explicit-any off）。実行不能だった `npm run lint` スクリプトを実効化し初回 37 エラー→0 |
| npm audit fix | 3.64 | editors/vscode の package-lock.json のみ更新（package.json 不変）で 15 件中 14 件解消（undici/uuid/ajv/lodash/markdown-it/minimatch 等）。非破壊確認: npm test 135/135 全緑 |
| package.json scripts.test:unit | 3.55/3.57 | テストバンドルの追加・削除に伴う配線のみ更新（contributes/main/activationEvents 等の動作定義は不変） |

### 依存更新（dev 環境のみ — Cargo.lock は .gitignore 対象）

- **imageproc 0.26.1 → 0.26.2**（3.64・`cargo update -p imageproc`・semver 互換）: unsound 3 件（RUSTSEC-2026-0115/0116/0117）を解消。Cargo.lock はリポジトリ非追跡のため修正は dev 環境のみ・新規クローンは 0.26.2 を自動解決。更新後の全体検証 1956 passed/0 failed を新鮮確認。
- pasta_lsp の serde_json を [dependencies]→[dev-dependencies] へ移動（3.41・使用が #[cfg(test)] のみと実証・downstream フットプリント削減）。

### 環境制約の無害化（Task 1 確定・全セルへ伝搬）

本実行時点で 3 変数とも設定中であることを実測確認し、全 cargo コマンドへ以下をコマンド文字列としてインライン適用した（シェル状態非持続のためセッション unset に依存しない）:

```
env -u NoDefaultCurrentDirectoryInExePath -u PASTA_DEBUG -u PASTA_DEBUG_PORT cargo ...
```

- `NoDefaultCurrentDirectoryInExePath=1`: mlua-sys/LuaJIT ビルドが exit 101 で失敗する既知制約（R4.7）
- `PASTA_DEBUG=1`・`PASTA_DEBUG_PORT=9276`: DAP ポート競合による偽 RED（86 テスト失敗）防止

npm/node コマンドには cargo 制約は影響しないため適用外。

---

*生成: Task 4（レポート集約）2026-06-12。入力: matrix.md（64 セル全件終端・PENDING 0）・`git log --grep "Riloop-Cell:" 4027097..HEAD`（64 コミット）・tasks.md Implementation Notes。本レポートは reports/ への追記型アーティファクトであり、次回ループ実行時の退避対象。*
