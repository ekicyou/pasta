# ギャップ分析: ponytail-core-registry

## Summary

- **Feature**: `ponytail-core-registry`
- **Discovery Scope**: Extension（既存の `crates/pasta_core` の撤去と型の直し。新しい部品もファイルも無い）
- **測った所**: ワークツリーの HEAD `ae04a28b`（main と同じ）。2026-11-02。`crates/pasta_core` は、前の測り直し（`8fc9bdff`）の後、`recommend-menu-scenes`（`1fb7ed38`）が `error.rs`・`scene_alias.rs` を変えただけである。
- **基準**: `cargo test -p pasta_core` は 145 件が通る（単体 117・`tests/word_table_test.rs` 27・doctest 1）。`cargo clippy -p pasta_core --all-targets` は警告 0。`cargo fmt --check -p pasta_core` は差分なし。`pasta_lua`・`pasta_check` のテストは、この分析では走らせていない。
- **Key Findings**:
  - 監査の控え（brief の Current State の 1〜4）は、**すべて残っている**。行番号は brief の 2026-11-02 の再測定とほぼ合う。`resolve_scene_id` だけ、範囲が 173〜213 行（brief は 173〜225 行）である。
  - 監査のやり直しで、同じ領域に **新しい指摘が 9 つ** 見つかった。うち 7 つは同じ種類（死んだ API・届かない誤り・使われない切り替え・写しだけの関数）で、範囲に入れる前提で要件に【前提】として書いた。1 つ（N8）は文書で、必ず直す。1 つ（N9）は範囲に入れない。
  - `fast_radix_trie` の `iter_prefix` は「キーで並べた順」を約束している（`map.rs:553`）。標準の `BTreeMap<String, _>` もキーのバイト列の辞書順で並ぶので、候補の順は変わらない見込みである。ただし、**キーをまたぐ候補の順を固めるテストが今は無い**（`scene_table_candidate_tests.rs:578` の補助は並べ替えてから比べる）。入れ物を替える前に、このテストを置く必要がある。
  - brief が挙げていない文書の直しがある。`book/src/internals/registry-search.md` は `RadixMap`・`fast_radix_trie`・`history`・`RandomSelectionFailed`・`set_shuffle_enabled`・`merge_from` を書いている（10 か所ほど）。`tech.md` は 27 行のほかに 161 行（「前方一致Radix Trie」）もある。README の使用例は、消す `resolve_scene_id` と、`from_scene_registry(..).unwrap()` を使っている。
  - 規模は S〜M、危険は低い。どの直しも既存の型の中で閉じる。いちばん手間が要るのは、消す API を使うテストの移し替え（`resolve_scene_id` 19 か所・`SceneTable::new` 12 か所・`set_shuffle_enabled` 24 か所・`RadixMap` を直に作るテスト）。

## 監査のやり直し（2026-10-19 の控えとの突き合わせ）

`/ponytail:ponytail-audit` の方法（地図を作る → 本番の流れを追う → 指摘ごとに具体的な場合を示す → 使われないと言う前に木全体を grep する）で、`crates/pasta_core` の全ファイル（`lib.rs`・`error.rs`・`actor_font.rs`・`registry/` の 9 ファイル・テスト 3 つ）を読み直した。コードは変えていない。

- **想定した負荷**: 1 つのゴーストの 1 つの Lua ランタイムが、イベントごとに数回、シーンと単語を検索する。表は辞書の確定で 1 回作り、作った後は選択の状態のほかは変えない。並行して書くことは無い（`SearchContext` は Lua の `UserData` で、1 つの VM の中だけで使う）。
- **本番の流れ**: トランスパイル（`pasta_lua/src/context.rs` → `SceneRegistry::register_global`・`register_local`、`WordDefRegistry::register_*`） → 辞書の確定（`pasta_lua/src/runtime/finalize.rs:182` → `register_global_raw`） → `search::register` → `SearchContext::with_aliases` → `SceneTable::from_scene_registry`・`WordTable::from_word_def_registry` → Lua の `SEARCH:search_scene`・`search_scene_exact`・`search_local_scene_exact`・`search_word`。`pasta_check validate` は `validate/scenes.rs` で同じ登録とシーン表を作り、`validate/references.rs` で `collect_scene_candidates` を、`validate/recommend.rs` で `sanitize_name` を使う。
- **「使われない」の判断**: `crates/` 全体（`pasta_lua`・`pasta_check`・`pasta_lsp`・`pasta_shiori`・`pasta_dsl`・`pasta_sample_ghost` と、各テスト）を grep した。Lua のスクリプト（`pasta_scripts/`）が呼ぶのは `SearchContext` のメソッドだけで、`pasta_core` を直には呼ばない。

### 控えの指摘の今

| # | 指摘 | 今 | 今の場所・使い手 |
| - | ---- | -- | ---------------- |
| 1a | `SceneTable::resolve_scene_id` | 残る | `scene_table.rs:173-213`（説明 173・関数 183）。本番の呼び手 0。テストは `scene_table_candidate_tests.rs` 11 か所・`scene_table_resolve_filter_tests.rs` 8 か所。そのうち、属性の絞り込み（`resolve_filter_tests.rs:52`・`:67`）と順次消費（`candidate_tests.rs:408`・`:458`・`:502`・`:688`）は、残す振る舞いを確かめている |
| 1b | `SceneTable::find_scene` | 残る | `scene_table.rs:505-522`。本番の呼び手 0。テスト `resolve_filter_tests.rs:99`・`:115`。`pasta_check/src/validate/builtins.rs:44` の `"find_scene"` は Lua の `act:find_scene` の名前で、別物 |
| 1c | `CachedSelection.history` | 残る | `scene_table.rs:56`（書くのは `:391`・`:400`・`:419`）。読む所 0 |
| 1d | `SceneTable::labels_iter` | 残る | `scene_table.rs:429-432`。テスト `resolve_filter_tests.rs:138`・`:185`・`:282` だけ |
| 1e | `SceneEntry.fn_path` | 残る | `scene_registry.rs:24`。書くのは 4 か所（`:79`・`:125`・`:165`・`:184`）。読むのは同じファイルのテストだけ（6 か所） |
| 1f | `from_scene_registry` が `Result` | 残る | `scene_table.rs:95-144`（`Ok(Self{..})` だけを返す）。呼び手：`pasta_check/src/validate/scenes.rs:104-106`（`.expect("from_scene_registry は失敗を返さない")`）・`pasta_lua/src/search/context.rs:59-62`（`?`）・テスト 18 か所（`.unwrap()`） |
| 2 | 作られない誤りの種類 | 残る | `error.rs:11`（`SceneTableResult`）・`:35-37`（`DuplicateSceneName`）・`:39-41`（`DuplicateScenePath`）・`:43-48`（`NoMoreScenes`）・`:52`（`WordTableResult`）。どれも作る所 0。`NoMoreScenes` を書くのは `search/context.rs:160`・`:182`・`:224` の match の腕と、文言のテスト（`error.rs:150-163`・`:178-183`）だけ。2 つの別名を使うのは `lib.rs:28-29` の再輸出だけ |
| 3a | `WordTable::new` | 残る | `word_table.rs:53-63`。本番の呼び手 0。テスト `tests/word_table_test.rs:375` だけ |
| 3b | `search_word` の `_filters` | 残る | `word_table.rs:191`。本番の呼び手は `search/context.rs:303` の 1 か所で `&[]`。テスト 27 か所が `&[]` を渡す |
| 3c | `WordTable::entries`・`clear_cache` | 残る | `word_table.rs:241-249`。テスト `word_table_test.rs:29`・`:377`・`:384-400`・`:418` だけ。`pasta_shiori/src/shiori.rs` の `clear_cached_lua_functions` は別物 |
| 3d | `WordCacheKey` の再輸出 | 残る | `word_table.rs:12-29`（欄と `new` も pub）。`registry/mod.rs:30`・`lib.rs:33` で再輸出。クレートの外の使い手 0 |
| 4 | `fast_radix_trie` | 残る | `scene_table.rs:9`・`:64`・`:78`・`:123`・`:129`（`entry`）・`:197`・`:473`・`:492`（`iter_prefix`）・`:273`・`:309`・`:327`（`get`）、`word_table.rs:9`・`:44`・`:58`・`:71`・`:76`・`:115`・`:147`。テストは `RadixMap::new()`・`insert(b"..")` を直に使う（`candidate_tests.rs` の表を手で組むテスト）。`Cargo.lock` で `fast_radix_trie` に頼るのは `pasta_core` だけ（`Cargo.lock:899`）。`memchr` は `fast_radix_trie` のほか 10 のクレートが使うので残る |

### 新しく見つけた指摘（同じ領域）

- **N1. `SceneTable::new` も本番の呼び手が 0**（死んだ API・要件 1.5）
  - `scene_table.rs:75-83`。本番の表はいつも `from_scene_registry` で作る。使うのは `candidate_tests.rs`（9 か所）と `resolve_filter_tests.rs`（3 か所）だけ。brief の 3a（`WordTable::new`）と同じ種類である。
  - テストは、空の表に `labels` と `prefix_index` を手で足す形で使う。`from_scene_registry(SceneRegistry::new(), ..)` で空の表を作れば置き換わる。
- **N2. 並べ替えの切り替え `shuffle_enabled` は、本番では常に有効**（使われない切り替え・要件 6）
  - `scene_table.rs:70`・`:435-437`、`word_table.rs:50`・`:237-239`。本番は `true` のまま変えない。`set_shuffle_enabled(false)` を呼ぶのはテストだけ（`pasta_core` の中で 24 か所）。
  - 指定列が空のモック（`MockRandomSelector::new(vec![])`）は、渡された順を変えない（`random.rs:80-97`。テスト `random.rs:146` で確かめている）。切り替えを切ったときと同じ順になる。巡が変わったときも、どちらも並べ替えないので同じである。2 つのテストの道具が同じことをしている。
  - マニュアル（`registry-search.md:208`）が「Lua からは呼べない（テスト用）」と書いている。
- **N3. `RandomSelectionFailed` は届かない**（届かない誤り・要件 4.3）
  - `select_from_cache`（`scene_table.rs:375-422`）の 2 つの `ok_or(RandomSelectionFailed)`（`:412`・`:417`）は、どちらも起こらない。状態は同じ関数の `:382` で作るので `get_mut` は必ず見つかる。候補は、空なら先に `filter_by_attributes` が `NoMatchingScene` を返す。状態のキーは候補の集まりを決めるもの（モジュール・キー・絞り込み・完全一致か・持ち主のローカル）をすべて含み、作った後の表は変わらないので、同じキーの候補の数は変わらない。巡の終わりでは `next_index` を 0 に戻す。
  - 作る所はあるので brief の 2 と同じではないが、呼び手は届かない誤りのために `Err(e) => Err(SearchError::SceneTableError(e))` の腕を書いている（その腕は `InvalidScene` のためにも要るので、腕は残る）。
- **N4. `SceneRegistry::get_scene` はテストだけ**（死んだ API・要件 2.3）
  - `scene_registry.rs:207-213`。使うのは同じファイルのテスト（14 か所）だけ。`SceneTable::get_scene` は別物で、本番が使う。
  - あわせて、`SceneEntry.id` と、`register_global`・`register_local`・`register_global_raw` が返す ID も、本番は読まない（`pasta_lua/src/transpiler.rs:225` は `let (_, counter)`、`:246` と `finalize.rs:182` は返り値を捨てる）。読むのは `pasta_lua/src/transpiler_tests.rs:170` と `context.rs` のテストだけ。ただし返り値の形を変えると、`ponytail-lua-transpiler` の持ち場の `pasta_lua/src/context.rs`（`register_global_scene` が `(i64, usize)` を返す）に波及する。要件では、ID は残す前提にした。
- **N5. `WordDefRegistry::sanitize_name` は写しだけ**（写しだけの関数・要件 5.6）
  - `word_registry.rs:99-102`。`SceneRegistry::sanitize_name` を呼ぶだけ。呼び手は同じファイルの `register_local`（`:64`）・`register_actor`（`:83`）と、そのテスト（`:224-228`）だけ。マニュアル `registry-search.md:148` が「同じ関数を呼ぶ」と書いている。
- **N6. `SearchContext::new`・`with_aliases` が `Result` を返すのは、`from_scene_registry` のためだけ**（決して失敗しない作り方・要件 3.3）
  - `pasta_lua/src/search/context.rs:45-73`。中で失敗しうるのは `from_scene_registry(..)?` だけである。要件 3.1 を入れると、この 2 つも決して失敗しない `Result` になる。
  - 呼び手：`search/mod.rs:50`（`?`。関数は `create_userdata` のために `LuaResult` を返し続ける）、テストの `tests/common/e2e_helpers.rs:69`・`:130`、`tests/lua_unittest_runner.rs:55`、`tests/runtime/scene_identity_format_test.rs:180`、`tests/search/scene_search_test.rs:487`、`search/context.rs` のテスト（12 か所に `new`・`with_aliases`）。`search/mod.rs` とテストは、roadmap の持ち場の表（`pasta_core/**`・`search/context.rs`・`validate/scenes.rs`）の外である。同じウェーブのほかの spec の持ち場でもない。
- **N7. `SearchError::WordTableError` は作られない**（作られない誤りの包み・要件 4.4）
  - `pasta_lua/src/search/error.rs:15`（`#[from] pasta_core::WordTableError`）。`WordTableError` は `WordNotFound` だけで、`search/context.rs:303-306` の `search_word` は、それを必ず `Ok(None)` に写す（match は網羅で、`?` は無い）。`?` でこの包みへ変える所は無い。
  - `search/error.rs` も持ち場の表の外である。
- **N8. 直すべき文書が brief より多い**（文書・要件 11）
  - `book/src/internals/registry-search.md`：20 行（「RadixMap による前方一致」）・32 行（依存に `fast_radix_trie`）・36 行（`merge_from`）・40 行（`RadixMap<Vec<SceneId>>`）・42 行（`WordCacheKey`・`RadixMap<Vec<usize>>`）・44 行（誤りの一覧に `RandomSelectionFailed`）・130 行（「RadixMap に入れる」）・148 行（`WordDefRegistry::sanitize_name`）・181 行（「RadixMap の `iter_prefix` が列挙する順」）・200 行（`history`）・208 行（`set_shuffle_enabled`）。194 行の `WordCacheKey` は、中の型の名前として書いてよい（公開をやめても型は残る）。
  - `.kiro/steering/tech.md`：27 行（`fast_radix_trie 1.2.0`）と 161 行（品質基準の「シーンO(1)、前方一致Radix Trie」）。
  - `crates/pasta_core/README.md`：13 行（「RadixMap による前方一致」）・32 行（`WordCacheKey` の行）・59-64 行（使用例が `from_scene_registry(..).unwrap()` と、消す `resolve_scene_id` を使う）・83 行（`search_word("", "挨拶", &[])`）・92 行（依存の表）。README は `lib.rs` に取り込まれていないので、使用例は doctest で確かめられていない。
  - `registry-search.md` は `internals/*` なので、`gen-skill-refs.mjs` の生成の元ではない。生成物の作り直しは要らない。
- **N9. 空の単語名はすべての大域の単語に前方一致する**（範囲に入れない）
  - `WordTable::collect_word_candidates("", "")` は、`iter_prefix("")` ですべての大域のキーに一致する。シーンは空の名前を `InvalidScene` にするので、規則が 2 つで違う。
  - ただし、これは決めた挙動として固めてある。テスト `word_table_test.rs:427`（`test_search_word_empty_key_matches_all_global`）があり、マニュアル `registry-search.md:189` が「単語名は空文字列を検査しない」と書く。DSL の道（`act.lua`・`actor.lua`）は、空の変数を `empty variable` の警告で先に止める。直すと単語の選ばれ方が変わり、要件 9 に反する。指摘にしない。

### 指摘にしなかったもの（記録）

- `SceneInfo.id`・`SceneInfo.name`・`SceneInfo.scope`（と型 `SceneScope`）は、本番では読まれない（`scene_table.rs` が読むのは `attributes`・`fn_name`・`parent` だけ）。ただし `SceneInfo` は本番が使う `SceneTable::get_scene` の公開の戻り値で、0.5.0 の `scene-count` が `get_scene(id).name` を読む見込みである（`scene-count/brief.md:114`）。書くだけで読まない内側の欄（`history`・`fn_path`）とは違い、公開の結果の形なので指摘にしない。`scope` は `parent.is_some()` と同じ情報だが、消すかは `scene-attribute-store`（`scene_types.rs` を触る）に任せる。
- `WordEntry.id` は、`merge_from` が消えると常に添字と同じになる。単語表の索引がこの値を使うので、消すなら索引の作り方も変わる。小さな形の違いで、監査の種類（死んだ API・誤りの型・依存）に当たらない。
- `WordTable::collect_word_candidates` はクレートの外から呼ばれない（`search_word` の中とテストだけ）。ただし本番の処理の一部で、死んだ API ではない。公開をやめるだけの直しはテストの置き場を変えるだけなので、指摘にしない。
- `pub mod random` と、`registry` の直下への再輸出の 2 つの道がある。`registry::random::MockRandomSelector` の道を使うのは `pasta_core` のテストだけ。害が無く、直しても読み手の手間はほぼ減らないので、指摘にしない。
- `SceneTable::set_shuffle_enabled` などテスト用の公開メソッドは、2026-06 の `review-improvement-loop`（`matrix.md` 3.2）で「公開クレートの公開 API なので維持」と判断されていた。2026-10-19 の開発者の決定（公開クレートの pub API を消してよい）で、この判断は改められた。N2 はその決定に沿う。
- `SceneAliasTable`・`ActorFont` は、`recommend-menu-scenes` などで最近入った部品で、どの公開の関数にも本番の使い手がある（`pasta_lua/src/config.rs`・`loader/config/`・`search/context.rs`・`pasta_check/src/validate/config.rs`）。指摘は無い。
- `random.rs` は、`MockRandomSelector` が状態を持たない形に整えてあり、指摘は無い。

## brief の読み方（食い違いの解き方）

- brief の Current State 4 と Approach の「`fast_radix_trie` と `memchr` が消える」は、2026-11-02 の再測定のとおり誤りである。`memchr` は `Cargo.lock` に残る。要件 7.2 に書いた。
- brief の 2026-11-02 の再測定の `resolve_scene_id` の範囲（173〜225 行）は、今のコードでは 173〜213 行である（214 行からは `resolve_scene_id_unified`）。
- brief の Current State 3 の「`new`（呼び手 0）」は、本番の呼び手が 0 の意味である。テストが 1 か所使う（`word_table_test.rs:375`）。
- brief の Constraints の「README と `tech.md` の断り書きは、10 本のうち先に入る 1 本が書く」は、`ponytail-lua-loader` が書き終えた（ルートの `README.md:88`・`tech.md:178`）。要件 11.4 で「書かない」とした。
- brief の Out of Boundary の `crates/pasta_lua/src/context.rs` と、roadmap の `merge_from` の約束（後から入る側が消す）は、要件 10 の 2 つの場合に分けて書いた。`TranspileContext::merge_from`（`context.rs:115-126`）は、今の main ではまだ残っている（`ponytail-lua-transpiler` は brief だけで、まだ入っていない）。

## Requirement-to-Asset Map

| 要件 | 今ある資産 | ギャップ |
| ---- | ---------- | -------- |
| 1 シーン表の API と欄 | `SceneTable`（`scene_table.rs`）。`resolve_scene_id_unified("", ..)` が、モジュールの文脈なしの前方一致を本番で受け持つ | **Missing**: 撤去。**Constraint**: `resolve_scene_id` のテストのうち、属性の絞り込み・順次消費・指定列の位置を確かめるものは、`resolve_scene_id_unified` に空のモジュール名で移す（要件 12.3）。`resolve_scene_id` は `:` で始まるキーを大域の検索で引けたが、`unified("", ":x")` は `SceneNotFound` になる。この違いを確かめるテストは消してよい（本番の誰も頼っていない） |
| 2 シーンの登録 | `SceneRegistry`（`scene_registry.rs`） | **Missing**: `fn_path` と 4 か所の書き込み、`get_scene` の撤去。テスト 6 か所の `fn_path` の確かめは、同じ値を持つ `fn_name` の確かめに置き換える |
| 3 失敗しない作り方 | `from_scene_registry` は `Ok` だけを返す | **Missing**: 返り値の型の直しと、呼び手の `?`・`.expect`・`.unwrap()`（テスト 18 か所）の外し。【前提】N6 を入れるなら `SearchContext` の 2 つと、その呼び手 |
| 4 誤りの型 | `SceneTableError`・`WordTableError`（`error.rs`） | **Missing**: 種類と別名の撤去、`lib.rs` の再輸出、`search/context.rs` の 3 つの腕、文言のテスト（`error.rs:150-163`・`:178-183`）。【前提】N3 は `select_from_cache` の 2 つの `ok_or` を、届かない誤りを返さない形に変える。【前提】N7 は `search/error.rs:15` の 1 行 |
| 5 単語表 | `WordTable`（`word_table.rs`） | **Missing**: 撤去と、`search_word` の引数の削除（呼び手 1 か所・テスト 27 か所）。`WordCacheKey` を中の型にする。`word_table_test.rs` は外の結合テストなので、`entries()` を使う確かめ（`:29`・`:418`）は、`collect_word_candidates` か `search_word` の結果で確かめる形に移す |
| 6 並べ替えの切り替え | `MockRandomSelector::new(vec![])` が同じ順を作る | **Missing**: 欄とメソッドの撤去、テスト 24 か所の移し替え。【前提】 |
| 7 前方一致の入れ物 | 標準の `std::collections::BTreeMap`（`scene_alias.rs` がすでに使う） | **Missing**: 索引の型と、`iter_prefix`・`get`・`entry` の置き換え。**Constraint**: キーをまたぐ候補の順を固めるテストが無い（下の Research Needed）。テストの手で組む表（`map.insert(b"..", ..)`）を、新しい入れ物の形に直す |
| 8 属性の絞り込み | `filter_by_attributes`・`SceneCacheKey.filters`・`NoMatchingScene` | **Constraint**: 触らない。絞り込みを確かめるテストが `resolve_scene_id` を使っている（`resolve_filter_tests.rs:52`・`:67`）ので、移すときに観点を失わない |
| 9 選ばれ方 | `pasta_lua` の `tests/search/*`・`lua_specs/*`（`set_scene_selector` の位置で選ぶテスト）、`pasta_check` の `validate_scenes_test.rs`・`validate_parity_test.rs` | **Constraint**: 入れ物を替えたあと、`pasta_lua`・`pasta_check` の全テストで確かめる |
| 10 同じウェーブ | roadmap の約束（`merge_from`・`Cargo.lock`） | **Constraint**: main に入れる時点で `TranspileContext::merge_from` があるかで分かれる。実装の最後（PR の前）に確かめる手順が要る |
| 11 文書 | 上の N8 の表 | **Missing**: 直し。README の使用例は doctest でないので、手で確かめる |
| 12 進め方 | 既存のテスト 3 ファイル（`#[path]` の型で中の欄に触れる） | **Missing**: 候補の順を固めるテスト。**Constraint**: `pasta_lua` のテストは `sample.generated.lua` の改行を書き換える。`git checkout --` で戻す |

## Implementation Approach Options

### Option A: 既存のファイルの中で削り、入れ物だけ替える（brief の Approach そのまま）

- シーン表・単語表・登録・誤りの型から、指摘の API と欄を消す。`from_scene_registry` は `Self` を返す。
- 索引は `BTreeMap<String, Vec<_>>` にし、前方一致は `range(prefix.to_string()..).take_while(|(k, _)| k.starts_with(prefix))`、完全一致は `get`、作るときは `entry(..).or_default()` にする。
- テストは今の 3 ファイルのまま、消す API の観点を残る API へ移す。
- ✅ 差分が最小。新しいファイルも抽象も要らない。
- ✅ 1 つの直し = 1 コミットに分けやすい（撤去ごと、入れ物の差し替え、文書）。
- ❌ テストの移し替えが数十か所に散る（`set_shuffle_enabled` 24・`resolve_scene_id` 19・`SceneTable::new` 12・`fn_path` 6・`search_word` の引数 27）。機械的だが量はある。

### Option B: 前方一致の索引を 1 つの小さな型にまとめてから替える

- シーン表と単語表が同じ形の索引（キー → ID の列）を持つので、`PrefixIndex<T>` のような共通の型を作り、2 つの表から使う。
- ✅ 前方一致の規則（`:` で始まるキーを除くなど）を 1 か所で確かめられる。
- ❌ 新しい抽象を足す。今の 2 つの表は ID の型（`SceneId`・`usize`）と、大域の検索で `:` を除く所が少しずつ違い、まとめる得は小さい。監査の方針（削除を優先し、層を足さない）に合わない。

### Option C: A に、入れ物の差し替えを最後に回す順を足す

- 撤去と型の直しを先に終え（テストが減り、入れ物を使う所が少なくなる）、最後に候補の順を固めるテストを置いてから入れ物を替える。
- ✅ 入れ物を替えるときに直すテストが最も少ない。
- ✅ 選ばれ方が変わる危険のある直しが 1 コミットに閉じ、戻しやすい。
- ❌ 候補の順のテストは、入れ物を替える直前でなく最初に置く方が、撤去の間の安全網にもなる（要件 12.1）。順のテストは最初に置き、差し替えだけを最後に回せば両立する。

## Implementation Complexity & Risk

- **Effort: S〜M**（brief の見込み 10〜14 タスク）。どれも既存のパターンの中の削除と型の直し。テストの移し替えの量で M に寄る。
- **Risk: Low**。挙動を変える直しは無い。唯一、選ばれ方が変わりうるのは入れ物の差し替えで、標準の入れ物の順が同じことは仕様で決まっている（下の Research Needed で確かめる）。

## Recommendations for Design Phase

- **推奨**: Option A を、Option C の順で行う。
  1. 候補の順を固めるテストを最初に置く（今の `RadixMap` で通ることを確かめる）。
  2. 撤去と型の直しを 1 つずつ（シーン表 → 登録 → 誤りの型 → 単語表 → 切り替え → 失敗しない作り方）。
  3. 入れ物の差し替え。
  4. 文書（README・`tech.md`・`registry-search.md`・`Cargo.toml` の説明）。
  5. `merge_from` は main に入れる直前に `TranspileContext::merge_from` の有無を見て決める（要件 10）。
- **設計で決めること**
  - `select_from_cache` を、届かない誤りを返さない形にする書き方（N3 を入れる場合）。`entry(..).or_insert_with(..)` の戻りの `&mut` をそのまま使えば、`get_mut` を 2 回引く今の形（借用を分けるための形）も無くなる。
  - `SearchContext::new`・`with_aliases` を直すか（N6）。直すなら呼び手の一覧は上のとおり。
  - テストの手で組む表を、`from_scene_registry` で作る形に寄せるか、中の欄（`labels`・`prefix_index`）を直に組む形のまま新しい入れ物に合わせるか。
- **Research Needed**
  - **候補の順が同じであることの確かめ**：`fast_radix_trie` 1.2.0 の `iter_prefix` は「キーで並べた順」（`map.rs:553` の説明）で、キーはバイト列である。`String` の順も UTF-8 のバイト列の辞書順なので一致するはずである。設計の最初に、次の組を含むテストで確かめる：ASCII と日本語の混在（`会話`・`会話A`・`会話_朝`）、`:` で始まるキー（`:` は 0x3A で数字より後・英字より前）、あるキーが別のキーの接頭辞になる組（`挨拶` と `挨拶_夜`）、同じキーに複数の ID。
  - 単語表の候補は、一致した項目の値をこの順で 1 本につなぐ。指定列の位置はこの順で読まれるので、単語にも同じテストを置く。

## 要件ディスカッションへの持ち越し

1. **`SceneTable::new` を消すか**（N1・要件 1.5）。推奨：消す。brief の `WordTable::new` と同じ種類で、本番の呼び手が 0。テストは空の登録から作れば足りる。
2. **`SceneRegistry::get_scene` を消し、`SceneEntry.id` と登録の関数が返す ID は残すか**（N4・要件 2.3）。推奨：`get_scene` だけ消し、ID は残す。ID を消すと返り値の形が変わり、`ponytail-lua-transpiler` の持ち場の `context.rs` に波及する。ID は 1 つの整数で、読み手の手間はほぼ増えない。
3. **`RandomSelectionFailed` を消すか**（N3・要件 4.3）。推奨：消す。届かない誤りで、brief の 2（作られない誤りの種類）と同じ「呼び手が起こらない誤りに備える」手間を生む。直しは `select_from_cache` の中で閉じ、マニュアルの 1 語も直る。
4. **`SearchError::WordTableError` を消すか**（N7・要件 4.4）。推奨：消す。作られない包みで、1 行の削除で済む。`search/error.rs` は持ち場の表の外だが、同じウェーブのほかの spec の持ち場でもない。
5. **`WordDefRegistry::sanitize_name` を消すか**（N5・要件 5.6）。推奨：消す。写しだけの関数で、呼び手は同じファイルの中だけ。照合規則が 1 か所（`SceneRegistry::sanitize_name`）にあることが、かえってはっきりする。
6. **並べ替えの切り替え `shuffle_enabled` を消すか**（N2・要件 6）。推奨：消す。本番で常に有効な「使われない切り替え」で、指定列が空のモックと同じことをしている。テストの移し替えは 24 か所で機械的である。消さない場合は、要件 6 とマニュアルの 208 行の直しを外す。
7. **`SearchContext::new`・`with_aliases` を、失敗の型で包まずに返すか**（N6・要件 3.3）。推奨：直す。`from_scene_registry` を直すと、同じ「決して失敗しない `Result`」が 1 段上に移るだけになる。呼び手は `search/mod.rs` の 1 か所とテスト 5 ファイルで、どれも同じウェーブのほかの spec の持ち場ではない。roadmap の持ち場の表（`pasta_core/**`・`search/context.rs`・`validate/scenes.rs`）に `search/mod.rs`・`search/error.rs` とテストを足すことになるので、開発者の了承が要る。
8. **`Cargo.toml` の説明（`description`）を直すか**（要件 11.6）。推奨：直す。crates.io に「DSL parsing」と出るが、パーサーは `pasta_dsl` にある。1 行で済む。
9. **`merge_from` の消し方の確かめの時期**（要件 10）。推奨：実装の最後（PR を作る直前）に、main に `TranspileContext::merge_from` があるかを見て決める。どちらの場合も `context.rs` には触らない。あちらが先に入っていれば、この spec が `pasta_core` の 2 つとテストとマニュアルの 1 文を消す。

**確定（2026-11-02 の要件ディスカッション）**: 1〜8 は推奨案のとおりに確定した（開発者「承認」）。9 は設計フェーズで扱う（PR を作る直前に、main に `TranspileContext::merge_from` が残っているかを見て決める）。

---

# 設計フェーズの調査と決定（2026-11-02・`/kiro-spec-design -y`）

## Summary（設計）

- **Discovery Scope**: Extension（light discovery）。新しい依存は無く、外部の文書の調べは要らない。確かめたのはコードと、cargo の置き場の `fast_radix_trie-1.2.0` のソースだけである。
- **使ったスキルと規則**: `kiro-spec-design`（`rules/design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）、`.kiro/settings/templates/specs/design.md`・`research.md`、steering の `product.md`・`tech.md`・`structure.md`・`roadmap.md`。開発者の方針（削除を優先・層を足さない・特性化テストを先に・1 抽出＝1 検証＝1 コミット・x86 と x64 の両方で走らせる）に沿った。
- **Key Findings**:
  - `RadixMap<V>` は `GenericRadixMap<Vec<u8>, V>`（`map.rs:8`）で、`iter_prefix` は「sorted by key」（`map.rs:553`）。キーはバイト列なので、順はバイト列の辞書順である。`String` の `Ord` も UTF-8 のバイト列の辞書順なので、`BTreeMap<String, _>` の範囲は同じ順になる。
  - 前方一致は `range::<str, _>((Bound::Included(p), Bound::Unbounded)).take_while(|(k, _)| k.starts_with(p))` で O(log n + k) になる。`p` で始まる文字列は `p` 以上の並びの先頭に続けて並ぶ（`p` 以上で `p` で始まらない文字列は、`p` の途中の位置で `p` より大きい文字を持つので、`p` で始まるどの文字列よりも大きい）。
  - 範囲の書き方は、`RangeFrom<&str>` ではなく `Bound` の組を使う（`(Bound<&T>, Bound<&T>)` は `T: ?Sized` で `RangeBounds<T>` を実装する。`str` で引くにはこの形が確かである）。手元のツールチェーン（rustc 1.99.1）には標準ライブラリのソースが無く、`RangeFrom<&str>` が通るかは確かめていない。

## Research Log（設計）

### 行番号と呼び手の数の確かめ（ワークツリー HEAD `208e5101`）

- `scene_table.rs`：`new` 74〜83・`from_scene_registry` 95〜144・`resolve_scene_id` 173〜213・`select_from_cache` 375〜422（`ok_or(RandomSelectionFailed)` 412・417、`history` の書き込み 391・400・419）・`labels_iter` 429〜432・`set_shuffle_enabled` 434〜437・`find_scene` 505〜522。ギャップ分析と一致した。
- `word_table.rs`：`WordCacheKey` 12〜29・`new` 53〜63・`_filters` 191・`set_shuffle_enabled` 236〜239・`entries`・`clear_cache` 241〜249。
- `scene_registry.rs`：`fn_path` 22〜24（書き込み 79・125・165・184）・`get_scene` 207〜213・`merge_from` 271〜289。`word_registry.rs`：`sanitize_name` 99〜102・`merge_from` 104〜114。
- `error.rs`：`SceneTableResult` 11・`RandomSelectionFailed` 31〜33・`DuplicateSceneName` 35〜37・`DuplicateScenePath` 39〜41・`NoMoreScenes` 43〜48・`WordTableResult` 52。消した種類の文言のテスト 145〜162・178〜183。
- `pasta_lua/src/search/context.rs`：`new` 45〜50・`with_aliases` 54〜73（`?` は 62）・`NoMoreScenes` の腕 160・182・224・`search_word` の呼び出し 303。テストの `SearchContext::new`・`with_aliases` は 12 か所（462・562・613・674・739・771・831・840・856・877・929・967）。`search/mod.rs:50`・`search/error.rs:13-15`。
- テストの外の呼び手：`tests/common/e2e_helpers.rs:69`・`:130`（`.expect`）、`tests/lua_unittest_runner.rs:55`（`.expect`）、`tests/runtime/scene_identity_format_test.rs:180`・`tests/search/scene_search_test.rs:487`（`.unwrap()`）。`pasta_check/src/validate/scenes.rs:104-106`。
- `pasta_core` のテストの道具：`resolve_scene_id(` は `candidate_tests.rs` 11・`resolve_filter_tests.rs` 8、`SceneTable::new` は 9＋3、`set_shuffle_enabled` は 5＋6＋13（`word_table_test.rs`）＝24、`search_word(` は `word_table_test.rs` 27、構造体の式で表を組むテストは `candidate_tests.rs` 5・`resolve_filter_tests.rs` 1。
- `set_shuffle_enabled(false)` を呼ぶテストのモックは、どれも並びを変えない列（`[0]`・`[0,1,2,3]`・`[0,1,2,3,4]`）だった。`MockRandomSelector` は先頭から有効な位置を置き、残りを元の順に並べる（`random.rs:80-97`）ので、`[0]` も `[]` も並びを変えない。行を消して `vec![]` にしても順は変わらない。
- `pasta_core` の外で、消す API（`fn_path`・`WordCacheKey`・`set_shuffle_enabled`・`labels_iter`・`resolve_scene_id`・`SceneTable::new`・`WordTable::new`・`WordDefRegistry::sanitize_name`・`SceneRegistry::get_scene`）を使う所は無い。`SceneEntry` を構造体の式で作る所も `pasta_core` の外に無い。`pasta_lua/src/transpiler_tests.rs:170` は `SceneEntry.id` を読む（残すので影響しない）。
- `TranspileContext::merge_from` は、今のワークツリー（main と同じ）の `pasta_lua/src/context.rs:120` にまだある。

### 文書に残る名前

- 要件 11 の 3 つ（`pasta_core/README.md`・`tech.md`・`registry-search.md`）のほかに、`crates/pasta_dsl/README.md:73`（「レジストリ関連（fast_radix_trie, rand）…への依存は一切ありません」）と `SOUL.md:434`（「Radix Trie等の高効率データ構造」）が見つかった。
  - `pasta_dsl/README.md` は要件 7.7（README 全般）にかかるが、要件 11 の一覧と roadmap の持ち場の表に無い。設計は【前提】で「（rand）」に直し、設計ディスカッションで確かめる（OPEN QUESTION 1）。
  - `SOUL.md` は目指す姿の例で、今の実装の説明ではない。要件 7.7 の対象（コードの説明・README・`tech.md`・マニュアル）に入らないので触らない。
- `structure.md` の `error.rs # SceneTableError, WordTableError` と公開 API の一覧は、整理の後も正しい。直さない。

## Architecture Pattern Evaluation（設計）

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A（採る） | 既存のファイルの中で削り、索引の入れ物だけ替える | 差分が最小。新しいファイルも型も無い | テストの移し替えが数十か所に散る（機械的） | C の順で進める |
| B | 前方一致の索引を共通の型にまとめる | 規則を 1 か所で確かめられる | 新しい抽象を足す。2 つの表の ID の型と `:` の扱いが少しずつ違い、得が小さい | 要件ディスカッションで採らないと確定 |
| A の中の小さな共通関数 | 前方一致の 1 行を `registry/mod.rs` などの関数にする | 4 か所の写しが 1 か所になる | 共通の部品が 1 つ増え、`word_table` が別のモジュールに頼る | 採らない。U1・U2 が 4 か所の書き落としを落とす |

## Design Decisions（設計）

### Decision: 候補の順を固めるテスト（持ち越し (a)）

- **Context**: 入れ物を替えると、キーをまたぐ候補の順が変わる危険がある。今は順を固めるテストが無い（`candidate_fn_names` は並べ替えてから比べる）。
- **Selected Approach**: シーン（`scene_table_candidate_tests.rs`）と単語（`tests/word_table_test.rs`）に 1 本ずつ、最初のコミットで置く。登録はわざと辞書順でない順にし、ASCII と日本語の混在（`挨拶1`・`挨拶A`・`Zebra`・`0番`）、接頭辞の組（`挨拶`・`挨拶_夜`）、同じキーに複数の ID（`挨拶` 2 つ）、`:` で始まるキー（ローカル）、範囲の後ろのキー（`雑談`）を入れる。順のまま比べ、指定列の位置で選ぶ確かめも 1 つずつ足す。
- **Rationale**: 今の `RadixMap` で通ることを確かめてから替えれば、替えたコミットで落ちたときにすぐ分かる。`雑談` は `take_while` の書き落としを落とす。
- **Follow-up**: 実装の最初のタスクで、今の `RadixMap` で通ることを確かめる。期待の並びは、キーのバイト列（`1`=0x31・`A`=0x41・`_`=0x5F・`あ`=E3・`挨`=E6・`雑`=E9）から作った。

### Decision: `select_from_cache` の返し方（持ち越し (b)）

- **Context**: 届かない `RandomSelectionFailed` を返す 2 つの `ok_or` を消す（要件 4.3）。
- **Alternatives**: 1) `entry(..).or_insert_with(..)` の `&mut` をそのまま使い、`SceneId` を返す。2) 今の 3 段（作る・戻す・`get_mut`）を残し、`expect` にする。
- **Selected Approach**: 1。欄 `random_selector` を直に使えば、`cache` の借用と分かれる（今の閉包も同じ形で通っている）。候補の位置は添字で読み、不変条件（同じキーの候補の数は変わらない）をコードに 1 行で書く。
- **Rationale**: `get_mut` の引き直しと `SceneCacheKey` の `Clone` が要らなくなり、行が減る。

### Decision: 手で組むテストの表（持ち越し (c)）

- **Context**: `SceneTable::new` を消すと、空の表に手で足すテスト 12 か所と、構造体の式で組む 6 か所の形を決める必要がある。
- **Alternatives**: 1) 中の欄を直に組む形を残し、入れ物の型だけ合わせる。空の表は `from_scene_registry(SceneRegistry::new(), ..)` で作る。2) すべて `from_scene_registry` で作る形に寄せる。
- **Selected Approach**: 1。
- **Rationale**: 差分が最小で、テストの意図（索引の中身そのもの）が変わらない。`candidate_tests.rs:688` は、索引の中の ID が昇順でない表（`[2, 0, 1]`）を使う。登録から作ると、同じキーの中の ID はいつも昇順になり、この表を作れない。

### Decision: `merge_from` の判定の時期（持ち越し (d)）

- **Selected Approach**: PR を作る直前に `git fetch origin main` → `git grep -n "pub fn merge_from" origin/main -- crates/pasta_lua/src/context.rs`。あれば残し、無ければ `main` を取り込んで `Cargo.lock` を作り直したうえで、2 つの `merge_from` とテスト 5 つとマニュアルの一文を消す 1 コミットを足す。どちらでも `context.rs` は触らない。

### Decision: 前方一致は 4 か所にそのまま書く

- **Context**: シーンの大域・ローカル、単語の大域・ローカルの 4 か所が、同じ範囲の書き方になる。
- **Selected Approach**: 共通の関数を作らず、各所に 1 行で書く。
- **Rationale**: 開発者の方針（層を足さない）と、ギャップ分析で Option B を退けた理由に合わせる。書き落としの危険は U1・U2 が落とす。

## Synthesis（設計）

- **Generalization**: 要件 1〜6 は、どれも「本番で使わない面を消す」の同じ形である。共通の仕組みは作らず、移行の順序（Migration Strategy）を 1 本の列にした。
- **Build vs Adopt**: 前方一致の索引は、標準の `BTreeMap` を採る（新しい依存を足さず、外す）。`scene_alias.rs` がすでに使っている。
- **Simplification**: `select_from_cache` の 3 段を 1 段に、`SceneCacheKey` の `Clone` を不要に、2 つの表から切り替えの欄を外す。新しい部品は 0。

## Risks & Mitigations（設計）

- 入れ物の差し替えで候補の順が変わる — U1・U2 を最初に置き、差し替えのコミットで変えずに通す。
- テストの移し替えで観点を落とす — 観点ごとの移し先を設計の Testing Strategy に列挙した。
- `Cargo.lock` が同じウェーブで衝突する — 後から入る側が作り直す（要件 10.5）。
- 0.5.0 の検索の spec が古い索引の形を前提にする — Revalidation Triggers に書いた。`scene-count` の brief はすでに `BTreeMap` を前提にしている。

## References（設計）

- `C:\rust\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\fast_radix_trie-1.2.0\src\map.rs`（8 行の `RadixMap` の定義・553 行の `iter_prefix` の説明）
- Rust 標準ライブラリ `BTreeMap::range`・`str` の `Ord`（バイト列の辞書順）
