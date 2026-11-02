# Requirements Document

## Project Description (Input)
`pasta_core` を読む人と直す人が困っている（2026-10-19 の `/ponytail:ponytail-audit` の指摘より）。シーン表・単語表・シーンの登録に、本番では誰も使わない API（`resolve_scene_id`・`find_scene`・`labels_iter`・単語表の `new`・`entries`・`clear_cache` など）、書くだけで読まない欄（`CachedSelection.history`・`SceneEntry.fn_path`）、作られない誤りの型（`DuplicateSceneName`・`DuplicateScenePath`・`NoMoreScenes`・`SceneTableResult`・`WordTableResult`）、決して失敗しないのに `Result` を返す `from_scene_registry`、前方一致の列挙にしか使わない依存 `fast_radix_trie` がある。ゴーストの作者に見える不具合は無い。`crates/pasta_core` の監査の指摘（ギャップ分析で増えたものを含む）を残らず直し、`from_scene_registry` が `Self` を返し、テストは全部通り、シーンと単語の選ばれ方は変わらないようにする。シーンの属性の絞り込みの仕組みは 0.5.0 が使うので残す。

## Introduction

この spec は、2026-10-19 の監査が `crates/pasta_core` で見つけた指摘（brief の Current State の 1〜4）と、2026-11-02 のギャップ分析で同じ領域に増えた指摘を、残らず直す。直しはすべて「挙動を変えない撤去と型の直し」である。ゴーストの作者に見える挙動（`@pasta_search` の結果・シーンと単語の選ばれ方・`pasta_check validate` の報告）は変えない。

要件は「その領域の指摘が残らず直っている」ことと、変えてはいけない挙動の決めごとである。どう直すか（関数・入れ物の形）は設計で決める。

> ギャップ分析で増えた指摘（要件 1.5・2.3・3.3・4.3・4.4・5.6・6・11.6）は、2026-11-02 の要件ディスカッションで、すべて範囲に入れると開発者が決めた（research.md の「要件ディスカッションへの持ち越し」の推奨案どおり）。

## Boundary Context

- **In scope**
  - 監査の 1〜4（シーン表・シーンの登録の使われない API と欄、作られない誤りの型、単語表の使われない API、`fast_radix_trie`）。
  - ギャップ分析で増えた、同じ領域の指摘（要件 1.5・2.3・3.3・4.3・4.4・5.6・6・11.6）。
  - 呼び手を同じ変更で合わせる：`crates/pasta_lua/src/search/context.rs`・`crates/pasta_check/src/validate/scenes.rs`。
  - 要件 3.3・4.4 のために、roadmap の持ち場の表に無いファイルも触る：`crates/pasta_lua/src/search/mod.rs`・`crates/pasta_lua/src/search/error.rs` と、`SearchContext::new`・`with_aliases` を呼ぶテスト（`tests/common/e2e_helpers.rs`・`tests/lua_unittest_runner.rs`・`tests/runtime/scene_identity_format_test.rs`・`tests/search/scene_search_test.rs`）。どれも同じウェーブのほかの spec の持ち場ではない（2026-11-02 に確かめた）。roadmap の持ち場の表は `kiro-complete` で書き足す。
  - 撤去に合わせた文書の直し：`crates/pasta_core/README.md`・`.kiro/steering/tech.md`・`book/src/internals/registry-search.md`。
- **Out of scope**
  - シーンの属性の絞り込みの仕組み（2026-10-19 開発者の決定）。`filter_by_attributes`・`SceneCacheKey.filters`・解決の関数の `filters` の引数・`NoMatchingScene`・`attributes` の欄。0.5.0 の `scene-attribute-store`・`scene-condition-attributes` が使う。
  - `crates/pasta_lua/src/context.rs`（`TranspileContext`。`ponytail-lua-transpiler` の持ち場）。
  - README と `tech.md` の「公開クレートの pub API を予告なく消すことがある」断り書き。`ponytail-lua-loader` が書いた（ルートの `README.md` 88 行・`tech.md` 178 行）。この spec は書かない。
  - 空の単語名の扱い（単語の検索は空の名前を検査しない。マニュアル `internals/registry-search.md` に今の挙動として書かれている）。
- **Adjacent expectations**
  - `ponytail-editor-tooling` がルートの `Cargo.toml`・`Cargo.lock` の席を持つ。この spec は `fast_radix_trie` の行を消すだけで、依存を足さない。後から入る側が `Cargo.lock` を作り直す。
  - `ponytail-lua-transpiler` が `TranspileContext::merge_from` を消すと、`SceneRegistry::merge_from`・`WordDefRegistry::merge_from` の使い手はテストだけになる。2 本のうち後から入る側が消す（要件 10）。
  - 0.5.0 の `scene-count` は、`SceneTable::get_scene` と `collect_scene_candidates` と `SceneInfo.name` を読む見込みである。この spec はそれらを残す。

## Requirements

### Requirement 1: シーン表の使われない API と欄を無くす

**Objective:** As a pasta の開発者・読み手, I want シーン表から、本番で誰も呼ばない API と、書くだけで読まない欄が無くなってほしい, so that シーン表を読む手間と、使われない道の保守が減る。

#### Acceptance Criteria

1. The pasta_core crate shall モジュールの文脈を取らないシーンの解決（`SceneTable::resolve_scene_id`）を提供しない。
2. The pasta_core crate shall 関数名を返すシーンの検索（`SceneTable::find_scene`。Lua の `act:find_scene` とは別物）を提供しない。
3. The pasta_core crate shall テストだけが使う全シーンの列挙（`SceneTable::labels_iter`）を提供しない。
4. The pasta_core crate shall 順次消費の状態に、選んだシーンの履歴（`CachedSelection.history`）を持たない。
5. The pasta_core crate shall 本番の呼び手の無い空のシーン表の作り方（`SceneTable::new`）を提供しない（ギャップ分析で増えた指摘。単語表の `new` と同じ種類）。
6. The pasta_core crate shall `SceneTable` の `from_scene_registry`・`resolve_scene_id_unified`・`resolve_scene_id_exact`・`resolve_global_owning_local_exact`・`collect_scene_candidates`・`get_scene`・`replace_selector` を、今と同じ意味で提供し続ける（`from_scene_registry` の返り値の形だけは要件 3 で変える）。

### Requirement 2: シーンの登録の使われない欄と API を無くす

**Objective:** As a pasta の開発者・読み手, I want シーンの登録から、書くだけで読まない欄とテストだけが使う API が無くなってほしい, so that 登録の項目が何に使われるのかを読み違えない。

#### Acceptance Criteria

1. The pasta_core crate shall シーンの登録の項目に、`crate::` を前に付けた関数のパス（`SceneEntry.fn_path`）を持たない。
2. When `register_global`・`register_local`・`register_global_raw` でシーンを登録した, the pasta_core crate shall 項目の `name`・`attributes`・`fn_name`・`parent` と、返す値（ID・通し番号）を、今と同じにする。
3. The pasta_core crate shall テストだけが使う、ID からシーンの登録を引く関数（`SceneRegistry::get_scene`）を提供しない（ギャップ分析で増えた指摘）。項目の ID（`SceneEntry.id`）と、登録の関数が返す ID は残す（返り値の形を変えると、`ponytail-lua-transpiler` の持ち場の `pasta_lua/src/context.rs` に波及するため）。

### Requirement 3: 決して失敗しない作り方は、失敗を返さない

**Objective:** As a pasta の開発者, I want 決して失敗しない検索表の作り方が、失敗の型を返さないでほしい, so that 呼び手が起こらない失敗のために `expect` や `?` を書かずに済み、起こりうる失敗と読み違えない。

#### Acceptance Criteria

1. The pasta_core crate shall シーンの登録からシーン表を作る関数（`SceneTable::from_scene_registry`）の結果を、失敗の型で包まずにシーン表そのものとして返す。
2. The `pasta_check validate` shall シーン表を作る所（`crates/pasta_check/src/validate/scenes.rs` 104〜106 行）で、起こらない失敗のための `expect` を持たない。
3. The pasta_lua crate shall `@pasta_search` の検索の文脈（`SearchContext`）の作り方（`new`・`with_aliases`）の結果を、失敗の型で包まずに返す（ギャップ分析で増えた指摘。作り方が失敗を返すのは `from_scene_registry` のためだけである）。呼び手（`search/mod.rs` の 1 か所とテスト）は同じ変更で合わせる。
4. When ランタイムの構築と辞書の確定で `@pasta_search` を登録する, the pasta_lua crate shall 登録の失敗の扱い（Lua の誤りとして返す）を、今と同じにする。

### Requirement 4: 作られない誤りの型を無くす

**Objective:** As a pasta の開発者, I want どこでも作られない誤りの種類と、誰も使わない型の別名が無くなってほしい, so that 呼び手が起こらない誤りのために match の腕を書かずに済む。

#### Acceptance Criteria

1. The pasta_core crate shall シーンの検索の誤りに、どこでも作られない種類（`DuplicateSceneName`・`DuplicateScenePath`・`NoMoreScenes`）を持たない。
2. The pasta_core crate shall 誰も使わない結果の型の別名（`SceneTableResult`・`WordTableResult`）を提供しない。
3. The pasta_core crate shall 起こりえない「乱数の選択の失敗」（`RandomSelectionFailed`）を持たず、順次消費でシーンを選ぶ処理は、候補がある限り必ず 1 つを選ぶ（ギャップ分析で増えた指摘。候補は空のときに先に誤りになり、状態はその直前に作られるので、この誤りは作られる道があっても届かない）。
4. The pasta_lua crate shall `@pasta_search` の誤りに、どこでも作られない単語表の誤りの包み（`SearchError::WordTableError`）を持たない（ギャップ分析で増えた指摘。単語表の誤りは「見つからない」だけで、`search_word` は必ずそれを「見つからない」に写す）。
5. When 1 で種類を消した, the pasta_lua crate shall `search/context.rs` の 3 つの match の腕（今の 160・182・224 行）から `NoMoreScenes` を外し、「見つからない」に写す種類を `SceneNotFound` と `NoMatchingScene` にする。
6. The pasta_core crate shall 残る誤りの種類（`SceneNotFound`・`NoMatchingScene`・`InvalidScene`・`WordNotFound`）の文言を、今と同じにする（ログに出る文言は公開の約束である）。

### Requirement 5: 単語表の使われない API を無くす

**Objective:** As a pasta の開発者・読み手, I want 単語表から、本番で誰も呼ばない API と使われない引数が無くなってほしい, so that 単語表を読む手間が減り、「将来のため」の引数を読み違えない。

#### Acceptance Criteria

1. The pasta_core crate shall 本番の呼び手の無い空の単語表の作り方（`WordTable::new`）を提供しない。
2. The pasta_core crate shall 単語の検索（`WordTable::search_word`）に、使われない属性の絞り込みの引数（今の `_filters`。唯一の呼び手が `&[]` を渡す）を持たない。
3. The pasta_core crate shall テストだけが使う、全項目の取り出し（`WordTable::entries`）と選択の状態の消去（`WordTable::clear_cache`）を提供しない。
4. The pasta_core crate shall 単語表の中だけで使う選択の状態のキー（`WordCacheKey`）を、公開の型としてクレートの外へ出さない。
5. The pasta_core crate shall 単語表の `from_word_def_registry`・`collect_word_candidates`・`search_word`・`replace_selector` を、今と同じ意味で提供し続ける。
6. The pasta_core crate shall シーンの照合規則（`SceneRegistry::sanitize_name`）を呼ぶだけの単語の登録の写し（`WordDefRegistry::sanitize_name`）を提供しない（ギャップ分析で増えた指摘。呼び手は `word_registry.rs` の中だけである）。照合規則そのものは変えない。

### Requirement 6: テストだけが使う並べ替えの切り替えを無くす

**Objective:** As a pasta の開発者, I want 本番で常に有効な「並べ替えをするか」の切り替えが無くなってほしい, so that 同じことをする 2 つのテストの道具（切り替えと、指定列が空のモック）を読み比べずに済む。

#### Acceptance Criteria

1. The pasta_core crate shall シーン表と単語表に、並べ替えをしない切り替え（`shuffle_enabled` の欄と `set_shuffle_enabled`）を持たない（ギャップ分析で増えた指摘。本番では常に並べ替え、テストだけが切る）。
2. When 切り替えを使っていたテストを移した, the pasta_core test suite shall 同じ観点（候補を集めた順のまま使う）を、指定列が空のモック（`MockRandomSelector::new(vec![])`。渡された順を変えない）で確かめる。
3. The pasta_core crate shall 並べ替えの抽象（`RandomSelector`・`DefaultRandomSelector`・`MockRandomSelector`）と、Lua からの指定列（`set_scene_selector`・`set_word_selector`）の意味を、今と同じにする。

### Requirement 7: 前方一致の入れ物から外部の依存を外す

**Objective:** As a pasta の開発者, I want 前方一致の列挙のためだけの外部の依存が無くなってほしい, so that 依存の更新と監査の手間が減り、`pasta_core` の依存が少なくなる。

#### Acceptance Criteria

1. The pasta_core crate shall 前方一致の索引に、外部のクレート（`fast_radix_trie`）を使わず、標準ライブラリだけで同じ検索をする。
2. The change shall `crates/pasta_core/Cargo.toml` とルートの `Cargo.toml` から `fast_radix_trie` の行を消し、`Cargo.lock` から `fast_radix_trie` の項目を消す。依存を足さない。`memchr` はほかのクレートが使うので `Cargo.lock` に残る。
3. When 前方一致で候補を集める, the pasta_core crate shall 候補の順を、今と同じキーのバイト列の辞書順にし、同じキーの中は登録順にする。
4. When 完全一致で候補を引く（`resolve_scene_id_exact`・`resolve_global_owning_local_exact`）, the pasta_core crate shall 今と同じく、キーと等しい項目だけを候補にする。
5. While モジュールの文脈が空である, the pasta_core crate shall 今と同じく、`:` で始まるキー（ローカル）を前方一致の候補に含めない。
6. The pasta_core crate shall 前方一致の検索の手間を、今と同じ程度（索引を引く手間に、一致した項目の数を足した程度）に保つ。全項目をなめる検索にしない。
7. The change shall コードの説明・README・`tech.md`・マニュアルに、「RadixMap」「Radix Trie」と `fast_radix_trie` の記述を残さない（要件 11）。

### Requirement 8: 属性の絞り込みの仕組みを残す

**Objective:** As a 0.5.0 の属性の spec の作り手, I want シーンの属性の絞り込みの仕組みが、この整理のあとも今の形のまま残ってほしい, so that `scene-attribute-store`・`scene-condition-attributes` が、それを土台に使える。

#### Acceptance Criteria

1. The pasta_core crate shall 属性による候補の絞り込み（`filter_by_attributes`）と、絞り込みに合う候補が無いときの誤り（`NoMatchingScene`）を残す。
2. The pasta_core crate shall シーンの解決の関数（`resolve_scene_id_unified`・`resolve_scene_id_exact`・`resolve_global_owning_local_exact`）の `filters` の引数と、選択の状態のキーの絞り込みの欄（`SceneCacheKey.filters`）を残す。
3. The pasta_core crate shall シーンの登録と検索表のシーン情報の属性の欄（`SceneEntry.attributes`・`SceneInfo.attributes`）を残す。
4. When 属性つきのシーンを `filters` つきで解決する, the pasta_core crate shall 今と同じく、すべての絞り込みに合う候補だけから選び、絞り込みの違う解決どうしの選択の状態を混ぜない。

### Requirement 9: シーンと単語の選ばれ方と、作者に見える結果を変えない

**Objective:** As a ゴーストの作者, I want この整理の前と後で、シーンと単語の選ばれ方と、検索の結果が変わらないでほしい, so that 同じ辞書が同じようにトークし続ける。

#### Acceptance Criteria

1. The `@pasta_search` module shall `search_scene`・`search_scene_exact`・`search_local_scene_exact`・`search_word` の結果（返す名前の組・値、見つからないときに何も返さないこと、空のシーン名で Lua の誤りになること）を、今と同じにする。
2. When 同じ名前を続けて検索する, the pasta_core crate shall 今と同じく、シャッフルした候補を一巡するまで重複なく順に使い、一巡したら並べ直す。
3. When Lua から指定列（`set_scene_selector`・`set_word_selector`）を与えた, the pasta_core crate shall 今と同じ候補の並びの位置で、同じシーンと単語を選ぶ。
4. The pasta_core crate shall 前方一致と完全一致、第 2 引数のあり・なし、`search_local_scene_exact` のローカル名の違いで、選択の状態を今と同じく別々に持つ。
5. The `pasta_check validate` shall 同じ辞書に対して、今と同じ報告（シーンの参照の検査・名前の衝突の注意）を出す。

### Requirement 10: 同じウェーブの spec との境界を守る

**Objective:** As a 同じウェーブの spec を並べて進める開発者, I want この spec が、ほかの spec の持ち場のファイルに触らず、共有するファイルでは決めた分だけを変えてほしい, so that どの順で main に入っても、後から入る側が main を取り込んで合わせるだけで済む。

#### Acceptance Criteria

1. While この spec を main に入れる時点で、`ponytail-lua-transpiler` が `TranspileContext::merge_from` をまだ消していない, the change shall `SceneRegistry::merge_from`・`WordDefRegistry::merge_from` とそのテストを残し、`crates/pasta_lua/src/context.rs` に触らない（あちらが後から入るときに消す）。
2. While この spec を main に入れる時点で、`ponytail-lua-transpiler` が `TranspileContext::merge_from` をすでに消している, the change shall `SceneRegistry::merge_from`・`WordDefRegistry::merge_from` とそのテストを消し、マニュアルの記述（`internals/registry-search.md` の `merge_from` の一文）も消す。
3. The change shall `crates/pasta_lua/src/context.rs` を、1・2 のどちらの場合も変えない。
4. The change shall ルートの `Cargo.toml`・`Cargo.lock` では、`fast_radix_trie` の行と項目を消すだけにする。
5. If この spec を main に入れる時点で、`ponytail-editor-tooling` などがルートの `Cargo.toml`・`Cargo.lock` を先に変えている, then the change shall main を merge で取り込んで `Cargo.lock` を作り直し（積んだコミットを rebase で書き換えない。2026-11-02 の設計ディスカッションで決めた）、相手が消した行・足した行を保つ。

### Requirement 11: 文書を今の構成に合わせる

**Objective:** As a pasta の開発者・`pasta_core` の利用者, I want README・steering・マニュアルが、整理したあとの API と依存のとおりに書かれていてほしい, so that 書いてあることとコードが食い違わず、消えた API の使用例を写して詰まらない。

#### Acceptance Criteria

1. The `crates/pasta_core/README.md` shall 公開 API の表から `WordCacheKey` の行を消し、使用例を残る API（`from_scene_registry` が失敗を返さない形・`resolve_scene_id_unified`・絞り込みの引数の無い `search_word`）で書き、依存の表から `fast_radix_trie` の行を消し、アーキテクチャの説明の「RadixMap による前方一致」を今の仕組みに書き直す。
2. The `.kiro/steering/tech.md` shall `pasta_core` の依存（27 行）から `fast_radix_trie` を消し、品質基準の表の検索性能（161 行の「前方一致Radix Trie」）を今の仕組みに書き直す。
3. The pasta manual shall `internals/registry-search.md` の、撤去したもの・変えたものの記述を、今の構成に書き直す。少なくとも次の所である。
   - 20・32・40・42・130・181 行の `RadixMap`・`fast_radix_trie`（依存は `thiserror`・`rand` だけになる）
   - 44 行の誤りの一覧（`RandomSelectionFailed` を外す。要件 4.3 を入れる場合）
   - 200 行の `history`
   - 208 行の `set_shuffle_enabled`
   - 148 行の `WordDefRegistry::sanitize_name`
   - 36 行の `merge_from`（要件 10.2 の場合だけ）
4. The change shall README と `tech.md` に、公開の pub API を消すことの断り書きを足さない（`ponytail-lua-loader` が書いた）。
5. Where 直した頁がスキルの参照の生成の元（`grammar/*`・`reference/pasta-toml.md`・`reference/runtime-failures.md`・`lua/modules/*`）である, the change shall `node book/tools/gen-skill-refs.mjs` を走らせ、生成物を同じ変更に入れる（`internals/*` は生成の元ではないので、`registry-search.md` だけなら要らない）。
6. The `crates/pasta_core/Cargo.toml` shall クレートの説明（`description`。crates.io に出る）から、もう持たない「DSL parsing」を消し、レジストリの層であることを書く（ギャップ分析で増えた指摘。パーサーは `pasta_dsl` へ移っている）。

### Requirement 12: 進め方と品質の関門

**Objective:** As a pasta の開発者, I want 整理を、振る舞いを固めてから小さく戻せる手順で進め、決まった関門を通してほしい, so that 撤去のついでに選ばれ方が変わっても、すぐに気づいて 1 つだけ戻せる。

#### Acceptance Criteria

1. When 撤去や型の直しを始める, the change shall その前に、変えてはいけない振る舞い（要件 7.3〜7.5・8.4・9.2〜9.4。とくに前方一致の候補の順と、指定列の位置で選ばれるシーン・単語）を固めるテストを、今のコードに対して先に置き、通ることを確かめる。
2. The change shall 1 つの直しごとに関門を通して確かめ、1 つのコミットにする（どの直しも、そのコミットだけを戻せば元に戻る）。
3. When 消す API を使っていたテストが、残す振る舞いを確かめていた（例：`resolve_scene_id` で順次消費を確かめるテスト）, the change shall その観点を残す API（例：`resolve_scene_id_unified` に空のモジュール名）のテストへ移し、観点を失わない。
4. The change shall `cargo fmt --check`、clippy（警告 0）、`cargo test -p pasta_core -p pasta_lua -p pasta_check` を通る。cargo の前に、環境変数 `NoDefaultCurrentDirectoryInExePath` を外す。
5. If `pasta_lua` のテストが `sample.generated.lua` の改行だけを書き換えた, then the change shall その差分をコミットに入れない。
