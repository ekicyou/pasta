# ギャップ分析: scene-count

## Summary
- **Feature**: `scene-count`
- **Discovery Scope**: Simple Addition（今の検索の部品に、数だけを返す口と組み込みの関数を足す）
- **調べた時点**: 2026-11-03（ブランチ `claude/kiro-start-scene-count-73eb1a`、main `96b75cbf` の上。`brief.md` の再測定は `c575febd`。行数はこのブランチで測り直した）
- **Key Findings**:
  - 数えるのに要る部品は全部ある。前方一致は `SceneTable::collect_scene_candidates("", key)`（`&self` なので巡回の状態に触らない）、完全一致はその候補を通し番号を外した名前で絞れば数えられる。`scene_table.rs` は触らずに済む（同じウェーブの約束どおり）。
  - 別名の置き換えと照合用の名前への揃えは、`search/context.rs` の `search_scene_by`（グローバルの枝）の 2 行（`scene_aliases.resolve(name)` → `SceneRegistry::sanitize_name`）を同じ順で通せば揃う。
  - DSL からの呼び方は 5 段の検索の 4 段目（`GLOBAL` の完全一致）で、置き場の `pasta/global.lua`（29 行）に足すだけで `＠シーン数（…）`・`＞シーン数（…）`・Lua の `GLOBAL.scene_count(act, …)` が全部動く。
  - 辞書の検査は `builtins.rs` の `GLOBAL_NAMES` に 4 つ足す。実行時の `GLOBAL` と一致を見るテスト（`validate_parity_test.rs` 689 行）があるので、足し忘れはテストで落ちる。
  - `brief.md` の Current State の「`＠…（…）` で呼ぶと無い関数として注意が出る」は今の検査と合わない。検査が見るのは `＞` の Call と選択肢の飛び先だけで、`＠名前（…）` の関数名は見ない（`references.rs` 30〜60 行）。4 つの名前を足して変わるのは、`＞シーン数` の `call-target-missing` が出なくなることと、`＊シーン数` に `scene-shadowed-by-builtin` が出ることの 2 つ。

## Requirement-to-Asset Map

| 要件 | 今ある部品 | ギャップ |
| ---- | ---------- | -------- |
| 1.1〜1.3 前方一致の数・グローバルだけ | `SceneTable::collect_scene_candidates(module_name, prefix)`（`crates/pasta_core/src/registry/scene_table.rs` 372 行）。`module_name` が空なら `:` で始まるローカルのキーを外す。0 件は `Err(SceneNotFound)`、空の名前は `Err(InvalidScene)` | **Missing**: 数だけを返す口（`SearchContext` のメソッドと Lua のメソッド）。`Err(SceneNotFound)` を `0` に読み替える |
| 1.2 `＞名前` と同じ候補 | `＞名前` の 5 段目は `SCENE.search(key, nil)` → `SEARCH:search_scene(name, nil)` → `resolve_scene_id_unified("", key)` → 同じ `collect_scene_candidates`。属性の絞り込み（`filter_by_attributes`）は今は空のフィルタで何も外さない | **Constraint**: `SCENE.search` は見つけた後に Lua の関数（`SCENE.get`）が無いと `nil` にするが、登録の仕組み上いつも関数がある。数は Rust のシーン表の候補の数で定義してよい（設計で一言書く） |
| 1.4・1.5 別名・照合用の名前 | `search_scene_by` のグローバルの枝（`crates/pasta_lua/src/search/context.rs` 161・162 行）: `let alias = self.scene_aliases.resolve(name); let key = SceneRegistry::sanitize_name(alias.unwrap_or(name));` | **Missing**: 同じ 2 行を数の口でも通す。共通の小さな関数に出すか、2 行を写すかは設計で決める |
| 1.6 無ければ 0・警告なし | `search_scene_by` は別名を当てて見つからないと `tracing::warn!("Scene not found (alias applied)")` を出す | **Constraint**: 数の口ではこの警告を出さない（0 は正しい答え）。共通の関数に出すなら、警告は検索の側に残す |
| 2.1〜2.3 完全一致の数 | 完全一致の検索 `resolve_scene_id_exact`（195 行）は非公開の索引 `prefix_index.get(key)` を引く。公開の部品は `collect_scene_candidates` と `get_scene(id)`（349 行）、`SceneRegistry::split_registered_name`（`scene_registry.rs` 239 行） | **Missing**: 前方一致の候補を `split_registered_name(&get_scene(id).name).0 == key` で絞る数え方（`brief.md` の再測定の答え）。索引のキーは `fn_name_to_search_key`（139 行）が `fn_name` の `::` の前を `split_registered_name` したもので、実行時のグローバルシーンは `register_global_raw` が `name` と `fn_name` の前半を同じ文字列にするので一致する |
| 3.1・3.2 名前 4 つ・どこでも使える | `GLOBAL` に英語名と日本語名を置く形（`global.lua` の `yield`／`チェイントーク`、`entry.lua` の `pause_auto_talk`／`自動トーク停止`、`recommend.lua` の `recommend_item`／`おすすめ項目`）。非 ASCII はブラケット記法（luacheck のため） | **Missing**: 関数 2 つと別名 2 つ。置き場は `pasta/global.lua`（同じウェーブの約束） |
| 3.3〜3.5 アクション行・代入・Call | 4 段目で `GLOBAL[key]` が関数なら `(act, 引数…)` で呼ぶ（`act.lua` 345〜400 行の `find_act_handler`、444 行の `call_expr`）。アクション行の戻り値は `ACT_IMPL.talk`（226 行）が `tostring` して積み、`nil` なら黙る | なし（置けば動く） |
| 3.6 Lua から | `GLOBAL` は `require("pasta.global")` の表 | なし |
| 3.7 読み込みの後ならいつでも | `@pasta_search` は読み込みの最後（finalize）に登録される。`find_act_handler` は `pcall(require, "@pasta_search")` で無いときに飛ばす | **Unknown（小）**: `main.lua` の上の段など、finalize より前に呼ばれたときの扱い（エラーにするか 0 か）。要件は「読み込みの済んだ後」に限ったので、前の扱いは設計で決める |
| 4.1 引数の誤りの報告 | `FAILURE.report(act, record)`（`failure.lua` 173 行。act が `nil` ならログだけ）。Call の動的なキーの誤りは `call_key`（`act.lua` 725 行）が `FAILURE.LABEL.BAD_NAME`（「名前にならない値」）・`FAILURE.kind_of(value)`（「値なし」「空文字列」「真偽値」…）で報告する | **Missing**: 数の関数の中の引数の確かめ。種類の名前は `BAD_NAME` を使えば新しい種類を足さずに済む |
| 4.2 誤りのとき値なし | `FAILURE.report` は `nil` を返す。アクション行では `talk` が `nil` を黙って飛ばすので、表記が 2 重にならない | 議題 1 次第（`0` にするなら返す値を変えるだけ） |
| 4.3 数値の引数 | `call_key`（727 行）・`recommend.lua` の `check_text`（31 行）は数値を `tostring` で名前にする | 議題 2 次第 |
| 4.4 余分な引数 | `recommend_item` は `select("#", ...)` を数えて `log.warn` する | 議題 3 次第 |
| 4.5 表記を出さない設定 | `FAILURE.enabled()`（`[logging] balloon_warning`） | なし（`FAILURE.report` を通せば従う） |
| 5.1 次の選びを変えない | `collect_scene_candidates` と `get_scene` は `&self`。巡回の控え（`cache`）を触るのは `select_from_cache` だけ | なし。テストで「数えた後も同じ乱数の並びで同じ順」を確かめる |
| 5.3 属性で数を変えない | 今の検索は空のフィルタを渡す | なし。`scene-condition-attributes` が後で条件を入れても、数の口は候補を集めるだけなので変わらない（その spec に申し送り済み） |
| 5.4 5 段の順を変えない | `＊シーン数` を書いたゴーストは、`＞シーン数` が 4 段目の組み込みに当たる | なし（今の組み込みの名前と同じ）。検査の `scene-shadowed-by-builtin` が注意する |
| 6.1〜6.3 辞書の検査 | `crates/pasta_check/src/validate/builtins.rs`（68 行）の `GLOBAL_NAMES: [&str; 12]` と上の説明（どのファイルが登録するか）。`is_builtin` を `references.rs` 46 行（Call）と `scenes.rs` 70 行（名前の重なり）が使う。一致テストは `validate_parity_test.rs` 684〜702 行 | **Missing**: 4 つの名前（配列の長さ 12 → 16）、説明の行に `シーン数` を足す。`validate_scenes_test.rs` にも組み込みの名前の例がある |
| 7.1 Call の章 | `book/src/grammar/call-jump.md`（703 行）の「特殊な呼び出し」の表（383〜388 行）と、アクション行での書き方の段落（390 行）、4 段目の例（265 行）。おすすめ項目の節（616 行〜）が説明の節の手本 | **Missing**: 表の 2 行と、数え方の節 1 つ |
| 7.2 検査の章 | `book/src/debug/validate.md`（257 行）172 行の一覧 | **Missing**: 4 つの名前 |
| 7.3 Lua の章 | `book/src/lua/script-api.md`（734 行）の `GLOBAL` の表（624〜633 行）と注記（635 行。`entry.lua` の名前は `main.lua` で代入しても元に戻る） | **Missing**: 表の 4 行。`global.lua` に置く名前は `yield` と同じく作者の代入で置き換わるので、注記の対象に入れない（議題 5） |
| 7.4 `@pasta_search` の章 | `book/src/lua/modules/pasta-search.md`（266 行）。メソッドの節は `search_scene`〜`set_scene_selector`。試験用の `set_*_selector` も載っている＝Lua から見えるメソッドは全部載せる慣わし | 議題 4 次第（慣わしに従うなら載せる） |
| 7.5 references | `book/tools/gen-skill-refs.mjs` がマニュアルの章から `.claude/skills/*/references/` を作る。`call-jump.md`・`script-api.md` の写しがある | **Constraint**: `.claude/skills/` への書き込みは分類器が自己改変として拒むことがある（メモリ）。拒まれたら手で回避せず、入った分と残りを開発者に示す |
| 7.x ほかの記述 | `book/src/internals/registry-search.md` 256 行・`internals/shiori.md` 445 行が、ランタイムが登録する `GLOBAL` の名前を列挙している（`shiori.md` は `entry.lua` の分だけ） | **Missing（小）**: `registry-search.md` 256 行の列挙に足すか、列挙をやめて表へのリンクにするかを設計で決める |
| 8.1〜8.3 テスト | Rust: `crates/pasta_lua/tests/search/`（`main.rs` が `module_test`・`scene_search_test` を束ねる）、`tests/scene_alias_search_test.rs`（455 行。別名の検索の試験）。`context.rs` の中の試験は 435 行から。Lua: `tests/lua_specs/*.lua` と一覧 `tests/lua_specs/init.lua`（1 行ずつ足す） | **Constraint**: `context.rs` は 1,009 行で、足す試験は別のファイルに置く（同じウェーブの約束）。`init.lua` の 1 行の追記は他の spec と重なりうるが、行を足すだけなので rebase で合う |

## 実装の案

### 案 A: 今の部品を伸ばす（推奨の候補）

- `search/context.rs` の `SearchContext` に、数だけを返すメソッドを 1 つ足す（例 `count_scene(name, exact) -> usize`）。中身は、別名を当てて揃えた名前で `collect_scene_candidates("", key)` を呼び、`Err(SceneNotFound)` を 0 に、`exact` のときは `split_registered_name(&get_scene(id).name).0 == key` で絞って数える。
- 同じファイルの `add_methods` に Lua のメソッドを 1 つ（または `count_scene`・`count_scene_exact` の 2 つ。`search_scene`／`search_scene_exact` の分け方に揃えるなら 2 つ）足す。
- `pasta/global.lua` に `GLOBAL.scene_count`・`GLOBAL.scene_count_exact` と日本語の別名 2 つを足す。引数を確かめ（`FAILURE`）、`require("@pasta_search")` のメソッドを呼ぶ。
- `builtins.rs` に 4 つの名前、マニュアル 3〜4 頁、references の作り直し、テスト（Rust は `tests/search/` に新しいファイル、Lua は `lua_specs/` に新しいファイルと `init.lua` に 1 行）。
- ✅ 触るファイルが少なく、同じウェーブの約束（`global.lua`・`scene_table.rs` に触らない・`context.rs` の外にテスト）に収まる。
- ✅ 数え方が検索と同じ部品を通るので、`＞` との数のずれが起きにくい。
- ❌ 別名と揃えの 2 行が `search_scene_by` と数の口の 2 か所になる（共通の関数に出せば 1 か所。どちらでも数行）。
- ❌ 完全一致の数は、前方一致の候補を絞るので候補が多い名前（`雑`）では余計に回る。シーン数は辞書の規模（数百〜数千）で、呼ぶのはトークの中の 1 回なので問題にならない見込み。

### 案 B: 数を Lua だけで数える

- `STORE.scenes`（関数の実体の表）を Lua で回し、別名・照合用の名前・前方一致を Lua で作り直す。
- ✅ Rust に手を入れない。
- ❌ `brief.md` の Problem そのもの（規則の作り直し）で、`＞` とずれる。照合用の名前の規則（`is_alphanumeric`）を Lua で同じにするのは難しい。採らない。

### 案 C: 候補の一覧を返す口を足し、数は Lua で数える

- `SEARCH:scene_candidates(name, exact)` で登録名の一覧を返し、`#` で数える。
- ✅ 後で「候補の名前が欲しい」ときに使える。
- ❌ 頼まれていない口（登録名の一覧）を公開することになり、マニュアルで約束する範囲が広がる。今は要らない（YAGNI）。

### 完全一致の数え方の小さな選択（設計で決める）

1. `collect_scene_candidates` の候補を `get_scene(id).name` の通し番号を外して比べる（`brief.md` の再測定の答え。`scene_table.rs` に触らない）。
2. 1 と同じだが、比べるのを `get_scene(id).fn_name` の `::` の前にする（索引のキーを作る `fn_name_to_search_key` と同じ材料なので、`name` と `fn_name` がずれる登録が後で入っても索引と一致する）。
3. `scene_table.rs` に候補を集めるだけの公開の関数を足す（同じウェーブの約束に反するので、このウェーブでは採れない）。

## 実装の大きさとリスク

- **Effort**: S（1〜3 日）。今の部品と書き方をそのまま使い、新しい種類の失敗も依存も足さない。
- **Risk**: Low。数えるのは読み取りだけ（`&self`）で、検索の状態に触らない。広がりは `GLOBAL` に名前が 4 つ増えること（`＊シーン数` を書いたゴーストだけが、`＞シーン数` で組み込みに当たるようになる）。
- **規模の見込み**: 3〜5 タスクのまま（数え方 1、GLOBAL の関数と検査 1、テスト 1、マニュアルと references 1）。

## 設計フェーズへの申し送り

- **推奨の向き**: 案 A。完全一致は小さな選択の 1 か 2。Lua のメソッドを 1 つ（`exact` の引数）にするか 2 つ（`search_scene`／`search_scene_exact` に揃える）にするかを決める。
- **決めること**:
  - 別名と揃えの 2 行を共通の関数に出すか（出すなら、別名を当てて見つからないときの警告は検索の側だけに残す）。
  - 引数の確かめを Lua（`global.lua`）と Rust（`count_scene`）のどちらでするか。Lua で確かめて `FAILURE` で報告するのがほかの組み込みと同じ形。Rust のメソッドに直に空の名前を渡したとき（`InvalidScene`）の扱いも決める（公開してマニュアルに載せると決めたので、マニュアルに書く振る舞いになる）。
  - finalize の前（`@pasta_search` が無いとき）に呼ばれたときの扱い。
  - `registry-search.md` 256 行の列挙をどうするか。
- **確かめること（Research Needed）**:
  - Lua の試験の場（`lua_specs`）で `@pasta_search` を実物にできるか、モックにするか（`global_chaintalk_*`・`auto_talk_pause_test` の作りを見る）。DSL の 4 通りの呼び方の試験は、Rust の統合試験（実物の読み込み）で書くほうが近い見込み。
  - `validate_scenes_test.rs` の組み込みの名前の例が、配列の長さや並びに依存していないか。
- **同じウェーブの約束（守ること）**: 関数は `crates/pasta_lua/pasta_scripts/pasta/global.lua` に置き、`shiori/entry.lua` に置かない（`debug-mcp-runtime` と重ならないため）。`scene_table.rs` に触らない。`search/context.rs` はこの spec の持ち場で、足す試験は別のファイルに置く。`builtins.rs` は版の違う `shiori4-v4-lua-api` も触る（行を足すだけ）。

## 要件ディスカッションへの申し送り

`requirements.md` の「要件ディスカッションで決めること」1〜5 のほかに、次を伝える。

- `brief.md` の Current State の「`builtins.rs` に無い名前を `＠…（…）` で呼ぶと無い関数として注意が出る」は今の検査と合わない（検査は `＞` と選択肢の飛び先だけを見る）。要件 6 は `＞` の Call と名前の重なりだけを約束した。`＠名前（…）` の関数名を検査するのは、この spec の外（今の検査の範囲を広げる別の話）。

---

# 設計フェーズの調査（2026-11-03）

## Summary（設計）
- **Discovery Scope**: Extension（軽い調査。`design-discovery-light.md`）。新しい依存も外部の API も無いので、Web の調査はしない。
- **使った規則とスキル**: `kiro-spec-design` の `design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`。ゴーストの書き方・Lua の口の確かめに、`pasta-ghost-authoring`・`pasta-lua-coding` の references（マニュアルの写し）の該当章だけを見た（`call-jump.md` の特殊な呼び出しの表・`script-api.md` の `GLOBAL` の表）。
- **Key Findings**:
  - Call の生成コードは、呼んだシーンの引数を後ろに足す（`act:call(…, "名前", {}, 引数…, table.unpack(args))`）。引数付きで呼ばれたシーンの中の `＞シーン数（「雑談」）` には、そのシーンの引数も渡る。`＠シーン数（…）`（`expr_fn`）には足さない。余分な引数の警告（4.4）は、Call の形ではシーンの引数でも出る（`＞おすすめ項目` と同じ）。
  - Call の戻り値は捨てられる（`SCENE.wrap` は `fn(act, ...)` の後に `act:build()` を返す。末尾の `return act:call(…)` の数値も使われない）。3.5 は置くだけで満たす。
  - 完全一致の数は、前方一致の候補を `parse_fn_name(fn_name).0` の通し番号を外した名前で絞る。索引のキー（`fn_name_to_search_key`）と同じ材料なので、Lua で通し番号なしに登録したシーン（`register_global_raw("雑談", …)`）でも索引と一致する。
  - 日本語と英語の名前に同じ関数を当てる今の書き方（`GLOBAL["チェイントーク"] = GLOBAL.yield`）では、関数は呼ばれた名前を知らない。4.1（失敗表記に呼んだ関数の名前）を満たすには、名前ごとに関数を作る。
  - 今の `GLOBAL` の組み込みで、引数の誤りを `FAILURE.report` で報告するものは無い。手本はイベントの `FAILURE.bad_reference0`（`BAD_NAME`・`kind_of`・`"<名前>: bad argument: …=" .. quoted`）。

## Research Log（設計）

### Call・式・アクション行での戻り値の扱い
- **Context**: 3.3〜3.5（数を台詞に出す・数値で返す・Call では何も出さない）。
- **Sources Consulted**: `crates/pasta_lua/pasta_scripts/pasta/act.lua`（`ACT_IMPL.talk` 226 行・`call_expr` 444 行・`ACT_IMPL.call` 683 行）、`pasta/scene.lua`（`SCENE.wrap` 177 行）、`tests/fixtures/sample.generated.lua`（`expr_fn` と `call` の生成の形）。
- **Findings**:
  - アクション行の `＠シーン数（…）` は `act:…:talk((act:…:expr_fn("シーン数", …)))` になり、`talk` が `tostring` で積む（LuaJIT の整数の数値は `"5"`）。
  - 変数代入の右辺は `var.n = act:expr_fn(…)` で、数値のまま入る。
  - Call は `handler(self, ...)` を呼んで戻り値を返すが、シーン関数の戻り値は `SCENE.wrap` が捨てる。
  - 引数の `＄x` は `var.x` の生の参照で、値が無くても報告しない（関数の中の報告と重ならない）。
- **Implications**: 関数は数値を返すだけでよい。引数の誤りで `0` を返すと、アクション行では失敗表記の後に `0` が出る（要件ディスカッションの決定 1 の帰結。設計に書く）。

### 失敗の報告の形
- **Context**: 4.1・4.2・4.5・4.6。
- **Sources Consulted**: `pasta/failure.lua`（`LABEL`・`kind_of`・`quoted`・`report`・`notation`・`enabled`）、`shiori/event/external_control.lua`・`scene_play.lua`（`bad_reference0` の使い方）、`shiori/event/recommend.lua`（余分な引数の `log.warn`）。
- **Findings**: `FAILURE.report(act, record)` は、ログに警告を 1 行出し、`[logging] balloon_warning` が真なら表記を `act.token` に積み、act が `nil` ならログだけにする。`failure.lua` は読み込み時に何も require しない（ログと設定は呼んだ時点）ので、`global.lua` から require しても循環しない。
- **Implications**: 新しい種類の名前は足さない（`BAD_NAME`「名前にならない値」を使う）。

### `@pasta_search` を引く時期
- **Context**: 3.7。
- **Sources Consulted**: `crates/pasta_lua/src/search/mod.rs` 81 行（`package.loaded["@pasta_search"]` に置く）、`runtime/finalize.rs`、`pasta/scene.lua` 145・212 行。
- **Findings**: `SCENE.search` は呼んだ時点で `require("@pasta_search")` を素のまま引く（無ければ Lua のエラー）。`find_act_handler` だけが `pcall` で包む。
- **Implications**: 数の関数も呼んだ時点で素の `require` で引く（`SCENE.search` と同じ）。読み込みの前（`main.lua` の上の段）に呼んだときは Lua のエラーになる（要件の範囲の外。仮定として書く）。

### 名前を列挙している場所
- **Context**: 6.3・7.1〜7.5。
- **Sources Consulted**: `おすすめ区切り` を全体から grep した。
- **Findings**: 一覧があるのは `builtins.rs`・`validate_scenes_test.rs`・`book/src/grammar/call-jump.md`・`book/src/debug/validate.md`・`book/src/lua/script-api.md`（`GLOBAL` の表。おすすめの 2 つは今も載っていない）・生成の references・`.claude/skills/pasta-ghost-authoring/SKILL.md`（特殊 Call の手書きの行）・`steering/product.md`。`internals/registry-search.md` 256 行は「…を含む」の例示で、全部の一覧ではない（おすすめの 2 つも無い）。
- **Implications**: 直すのは要件 6・7 が挙げた場所だけにする。`registry-search.md` は例示なので触らない（ギャップ分析の「決めること」の答え）。`SKILL.md` の手書きの行は要件に無く、分類器が書き込みを拒むことがあるので、この spec では触らない（OPEN QUESTION）。

## Architecture Pattern Evaluation（設計）

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 今の部品を伸ばす | `SearchContext` に数のメソッド、`global.lua` に関数 | 触るファイルが少ない・`＞` と同じ部品を通る | 別名と揃えの 2 行が 3 か所目になる | 採る |
| B: Lua だけで数える | `STORE.scenes` を回す | Rust に触らない | 照合用の名前・別名の規則の作り直しで `＞` とずれる | 採らない（brief の Problem そのもの） |
| C: 候補の一覧を返す | 登録名の一覧を公開し `#` で数える | 後で名前が欲しいときに使える | 頼まれていない口を約束する | 採らない（YAGNI） |

## Design Decisions（設計）

### Decision: Rust のメソッドは 2 つ、本体は 1 つ
- **Context**: 1.1・2.1・7.4。
- **Alternatives Considered**:
  1. `count_scene(name, exact)` 1 つ
  2. `count_scene`／`count_scene_exact` の 2 つと、非公開の本体 `count_scene_by`
- **Selected Approach**: 2。`search_scene`／`search_scene_exact`／`search_scene_by` の分け方と同じにする。Lua のメソッドも `count_scene(name)`・`count_scene_exact(name)` の 2 つ。
- **Rationale**: 同じファイルの書き方に揃え、Lua の口で真偽値の引数を約束しない。
- **Trade-offs**: 数行増える。

### Decision: 完全一致は `fn_name` の前半の通し番号を外して比べる
- **Context**: 2.1・2.2。`scene_table.rs` に触らない（同じウェーブの約束）。
- **Alternatives Considered**: `get_scene(id).name` を比べる（brief の再測定の答え）／`fn_name` の `::` の前を比べる（ギャップ分析の小さな選択 2）／`scene_table.rs` に関数を足す（約束に反する）。
- **Selected Approach**: `fn_name` の前半。索引のキーを作る `fn_name_to_search_key` と同じ材料なので、完全一致の検索（`prefix_index.get(key)`）の候補と一致する。
- **Trade-offs**: 前方一致の候補を作ってから絞るので、短い名前では余計に回る。辞書の規模とトークの中の 1 回の呼び出しでは問題にならない。

### Decision: 別名と揃えの 2 行は写す（共通の関数に出さない）
- **Context**: 1.4・1.5・1.6。
- **Selected Approach**: `search_local_scene_exact` と同じく 2 行を写す。共通の関数に出すと `search_scene_by`・`search_local_scene_exact` の書き換えになり、この spec の範囲を超える。数の口は別名を当てて見つからなくても警告を出さない（0 は正しい答え）。
- **Trade-offs**: 規則が変わるとき 3 か所を直す。`scene-exact-match` などが検索の規則を変えるときの見直しの対象に挙げる（Revalidation Triggers）。

### Decision: 名前ごとに関数を作る（同じ関数を 2 つの名前に当てない）
- **Context**: 3.1・4.1（失敗表記に呼んだ関数の名前）。
- **Alternatives Considered**:
  1. 今の書き方どおり `GLOBAL["シーン数"] = GLOBAL.scene_count` にし、表記の名前を日本語の名前に固定する
  2. 名前と使うメソッドから関数を作る小さな関数（`global.lua` の中の `local`）で、4 つの名前それぞれに関数を作る
- **Selected Approach**: 2。動作は対の名前で同じで、表記とログに出る名前だけが呼んだ名前になる。
- **Trade-offs**: `GLOBAL.scene_count ~= GLOBAL["シーン数"]`（同じ関数ではない）。マニュアルの `GLOBAL` の表は「同じ関数」でなく「同じ動作」と書く（OPEN QUESTION）。

### Decision: 引数の確かめは Lua で、Rust のメソッドは空の名前をエラーにする
- **Context**: 4.1〜4.4・7.4。
- **Selected Approach**: 値の種類の確かめ・数値の `0`・余分な引数の警告は `global.lua` の関数で行う（ほかの組み込みと同じ層）。`@pasta_search` のメソッドは文字列を受け、0 件は `0`、空の名前は `search_scene` と同じく Lua のエラーにする（`InvalidScene`）。
- **Rationale**: `@pasta_search` のほかのメソッドと同じ約束にし、作者向けの報告は DSL の口に置く。
- **Trade-offs**: `SEARCH:count_scene("")` を直に呼ぶ Lua の作者はエラーを受ける（マニュアルに書く。OPEN QUESTION）。

## Synthesis（設計）
- **Generalization**: 前方一致と完全一致は、同じ「別名を当て・揃え・グローバルの候補を集め・（完全一致なら）絞って数える」の 2 つの形。本体 1 つに `exact` の切り替えで書く（`search_scene_by` と同じ）。
- **Build vs Adopt**: 候補集め（`collect_scene_candidates`）・名前の分解（`split_registered_name`）・別名（`SceneAliasTable::resolve`）・揃え（`sanitize_name`）・報告（`FAILURE.report`）・名前の一致テスト（`validate_parity_test.rs`）をそのまま使う。新しく作るのは、数えるメソッドと関数を作る小さな関数だけ。
- **Simplification**: 候補の一覧の口・属性の絞り込み・「今出せる数」・前に呼んだときの 0 の扱い・共通の別名の関数は作らない。Lua の試験（`lua_specs`）は足さず、実物の読み込みの Rust の統合試験 1 本にまとめる（DSL の 4 通りの呼び方と `@pasta_search` を同じ場で確かめられる）。

## Risks & Mitigations（設計）
- 引数付きで呼ばれたシーンの中の `＞シーン数（…）` で、余分な引数の警告が出る — `＞おすすめ項目` と同じ動きとして受け、マニュアルに書くかを議論で決める（OPEN QUESTION）。
- 別名と揃えの規則が 3 か所になる — 試験で `＞名前` の候補と数が等しいことを確かめ、ずれを試験で拾う。
- `GLOBAL_NAMES` の足し忘れ — `validate_parity_test.rs` の一致テストが落とす。
