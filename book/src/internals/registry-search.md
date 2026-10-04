# シーン・単語レジストリとシーン検索

ごきげんよう。同じ名前のシーンがいくつもあるとき、pasta はどれを選ぶのか――気になって夜も眠れませんわね？
レジストリへの登録から辞書の確定、前方一致での検索まで、わたくしが手際よくご案内いたします。さあ、参りましょう。

---

この章では、シーン・単語のレジストリと、実行時のシーン検索・単語検索を扱う。

## 目的と責務

レジストリは、シーンと単語を名前で引ける形に集め、検索表へ変換する。検索表は前方一致で候補を集め、同名・同接頭辞の候補から 1 つを乱数に従って選ぶ。

この章が責務を持つのは次の事項である。

- `pasta_core` のレジストリ（`SceneRegistry`・`WordDefRegistry`）と検索表（`SceneTable`・`WordTable`）、乱数の抽象（`RandomSelector`）
- 検索キーの形式と、登録と検索が共有する照合規則（サニタイズ）、RadixMap による前方一致での候補の収集
- 候補が複数あるときの選択（シャッフルと順次消費）と、その状態の持ち方
- ローカル優先の検索順を、Lua 側の検索手順が `@pasta_search` の呼び出しでどう組み立てているか
- 実行時の辞書確定のうち Rust 側の処理（`finalize_scene` による Lua 側の収集とレジストリの再構築）と、Lua 側との受け渡し
- `@pasta_search` の実装（`SearchContext`）

辞書確定がトランスパイルからどうつながるか（2 段の段階構成）は [トランスパイルパイプライン](transpiler.md#段階構成) が権威を持ち、この章では再記述しない。Lua 側でシーン・単語を収集するテーブルの構造とモジュールの関数は [Lua ランタイム内部モジュール](internal-modules.md#finalize_scene) で扱う。利用者から観測できる検索の挙動（スコープの優先順位・前方一致・シャッフル＆順次消費）は、[単語定義](../grammar/words.md#スコープと優先順位)・[Call / Jump](../grammar/call-jump.md#スコープ解決アルゴリズム)・[アクター辞書](../grammar/actor-dictionary.md#アクタースコープと単語参照の統合)・[@pasta_search](../lua/modules/pasta-search.md) が正である。

## 構成要素

### pasta_core のレジストリと検索表

`pasta_core` は Lua に依存しないレジストリ層である。依存は `thiserror`・`fast_radix_trie`・`rand` だけである。

| 型 | 所在 | 役割 |
| -- | ---- | ---- |
| `SceneRegistry`・`SceneEntry` | `crates/pasta_core/src/registry/scene_registry.rs` | シーンを登録順の `Vec` に集め、1 始まりの ID を振る。`register_global`・`register_local` はトランスパイル時の登録で、同名のグローバルシーンにサニタイズ後の名前ごとのカウンタで番号を振る。`register_global_raw` は辞書確定での登録で、番号付きの名前をそのまま受け取る。`merge_from` はファイルごとのレジストリを統合する。`sanitize_name` は名前の英数字と `_` 以外を `_` に置き換える。登録と検索が共有する照合規則である（[照合規則の共有](#照合規則の共有)） |
| `SceneId`・`SceneScope`・`SceneInfo` | `crates/pasta_core/src/registry/scene_types.rs` | 検索表が持つシーン情報。`SceneId` は `SceneTable` 内の 0 始まりの添字である |
| `SceneTable` | `crates/pasta_core/src/registry/scene_table.rs` | シーンの検索表。`SceneInfo` の `Vec`、前方一致の索引（`RadixMap<Vec<SceneId>>`）、選択状態のキャッシュ、`RandomSelector` を持つ |
| `WordDefRegistry`・`WordEntry` | `crates/pasta_core/src/registry/word_registry.rs` | 単語の定義を、検索キーと値のリストの組（`WordEntry`）として登録順に集める。グローバル・ローカル・アクターの 3 種の登録関数がキーの形式を決める |
| `WordTable`・`WordCacheKey` | `crates/pasta_core/src/registry/word_table.rs` | 単語の検索表。`WordEntry` の `Vec`、前方一致の索引（`RadixMap<Vec<usize>>`）、選択状態のキャッシュ、`RandomSelector` を持つ |
| `RandomSelector`・`DefaultRandomSelector`・`MockRandomSelector` | `crates/pasta_core/src/registry/random.rs` | 乱数の抽象。検索表が使うのは `shuffle_usize` だけである。既定の実装はシステムの乱数で種を決めた `StdRng`、モックはシャッフルしない |
| `SceneTableError`・`WordTableError` | `crates/pasta_core/src/error.rs` | 検索の失敗。シーンは `SceneNotFound`・`NoMatchingScene`・`InvalidScene`・`RandomSelectionFailed` などを返し、単語は `WordNotFound` だけを返す |

レジストリは登録を集める側、検索表は検索する側である。`SceneTable::from_scene_registry`・`WordTable::from_word_def_registry` がレジストリを消費して検索表を作り、作った後の検索表は選択状態のキャッシュと `RandomSelector` 以外を変更しない。

### @pasta_search の実装

| 要素 | 所在 | 役割 |
| ---- | ---- | ---- |
| `register`・`loader` | `crates/pasta_lua/src/search/mod.rs` | 2 つのレジストリから `SearchContext` を作り、Lua のユーザーデータとして `package.loaded["@pasta_search"]` に置く |
| `SearchContext` | `crates/pasta_lua/src/search/context.rs` | `SceneTable` と `WordTable` を 1 組ずつ所有する。Lua へ `search_scene`・`search_word`・`set_scene_selector`・`set_word_selector` の 4 メソッドを公開する。`search_scene` の名前と `search_word` のスコープは、`sanitize_name` でサニタイズしてから検索表に渡す |
| `SearchError` | `crates/pasta_lua/src/search/error.rs` | `pasta_core` のエラーを包み、Lua の `RuntimeError` に変換する |

`SearchContext` は Lua VM ごとに 1 つであり、複数のランタイムの間で共有しない。

### 辞書確定（Rust 側）

`crates/pasta_lua/src/runtime/finalize.rs` が実行時の辞書確定を担う。

| 関数 | 役割 |
| ---- | ---- |
| `register_finalize_scene` | Lua の `pasta` モジュールにあるスタブの `finalize_scene` を、Rust の関数で上書きする |
| `finalize_scene_impl` | 収集・レジストリの構築・`@pasta_search` の登録を順に行い、`true` を返す |
| `collect_scenes` | `pasta.scene` の `get_all_scenes()` から `(グローバル名, ローカル名)` の組を集める |
| `collect_words` | `pasta.word` の `get_all_words()` から、値のリスト 1 つにつき 1 つの `WordCollectionEntry` を作る |
| `build_scene_registry`・`build_word_registry` | 集めた組から `SceneRegistry`・`WordDefRegistry` を作る |

### Lua 側の登録と検索の入口

| モジュール | 所在 | この章に関わる役割 |
| ---------- | ---- | ------------------ |
| `pasta.scene` | `crates/pasta_lua/pasta_scripts/pasta/scene.lua` | 生成コードの `create_scene` でシーン関数を登録し、グローバル名の番号を振る。`SCENE.search` は `search_scene` の結果をシーン関数に結び付ける |
| `pasta.word` | `crates/pasta_lua/pasta_scripts/pasta/word.lua` | 生成コードの `create_word` などで単語の値を登録する |
| `pasta` | `crates/pasta_lua/pasta_scripts/pasta/init.lua` | `finalize_scene` のスタブを持つ（Rust が上書きする） |
| `pasta.act` | `crates/pasta_lua/pasta_scripts/pasta/act.lua` | `find_act_handler` がローカル優先の検索順を組み立てる |
| `pasta.actor` | `crates/pasta_lua/pasta_scripts/pasta/actor.lua` | アクタープロキシの `find_actor_handler` がアクター単語を検索する |

これらのモジュールのフィールド・関数・データ構造は [Lua ランタイム内部モジュール](internal-modules.md#scene-モジュール) で扱う。

## 処理とデータの流れ

### 辞書確定

`@pasta_search` は 2 回登録される。1 回目は Lua VM の構築時（`crates/pasta_lua/src/runtime/mod.rs`）で、トランスパイル時の `TranspileContext` のレジストリから作られる。2 回目がここで述べる辞書確定で、検索の権威はこちらにある。ランタイムの構築（`crates/pasta_lua/src/runtime/factory.rs`）は、VM の構築とモジュールの登録の後に `register_finalize_scene` を呼び、`main`・`pasta.shiori.entry` を読み込んでから `pasta.scene_dic` を読み込む。`scene_dic.lua` は全シーンモジュールを `require` した後に `require("pasta").finalize_scene()` を呼び、ここで Rust の `finalize_scene_impl` が動く。

```text
pasta.scene_dic
 ├ require("pasta.scene.…") × 全モジュール
 │   生成コードの実行 → pasta.scene / pasta.word が Lua 側のテーブルに登録
 └ require("pasta").finalize_scene()          … Rust: finalize_scene_impl
     1. collect_scenes: get_all_scenes() を走査
          {グローバル名: {__global_name__, ローカル名: 関数, …}}
          → __global_name__ を除く全キーを (グローバル名, ローカル名) の組に
          （1 件も無ければ警告ログ）
     2. collect_words: get_all_words() の global / local / actor を走査
          {キー: {{値…}, {値…}}} → 値のリスト 1 つにつき WordCollectionEntry 1 つ
     3. build_scene_registry: グローバル名ごとにまとめ、register_global_raw（属性は空）
        build_word_registry: アクター → register_actor、ローカル → register_local、
                             それ以外 → register_global
     4. search::register: SearchContext::new（両検索表を既定の乱数で構築）
          → package.loaded["@pasta_search"] を置き換える
```

`register_global_raw` は、グローバル名（`メイン1` のように番号を含む）から `メイン1::__start__` を作ってグローバルシーンとして登録し、`__start__` 以外のローカル名ごとに `メイン1::選択肢_1` の形でローカルシーンを登録する。カウンタは使わず、名前を作り直さない。グローバル名の番号は Lua 側の `create_scene` が全モジュールを通して振ったものであり、ローカル名はトランスパイラが生成したシーン関数名（`選択肢_1` の形）である。

収集の失敗（Lua のエラー）は `finalize_scene` の Lua エラーになり、`pasta.scene_dic` の読み込みの失敗として起動を止める。起動モジュールの失敗の扱いは [ローダ自己展開とモジュール解決](loader.md) で扱う。

### 検索キーの形式

検索表は、レジストリの各項目から次の形式の検索キーを作り、RadixMap に入れる。シーンは `SceneTable::from_scene_registry` が関数名（`fn_name`）から変換し、単語は `WordDefRegistry` の登録時にキーが決まる。辞書確定後の例を示す。

| 種類 | 関数名（シーン） | 検索キー | 登録 |
| ---- | ---------------- | -------- | ---- |
| グローバルシーン | `メイン1::__start__` | `メイン1`（`::` より前） | `register_global_raw` |
| ローカルシーン | `メイン1::選択肢_1` | `:メイン1:選択肢_1`（`::` を `:` に置き換え、先頭に `:`） | `register_global_raw` |
| グローバル単語 | — | `挨拶`（単語名） | `register_global` |
| ローカル単語 | — | `:メイン1:場所`（`:`＋サニタイズしたシーン名＋`:`＋単語名） | `register_local` |
| アクター単語 | — | `:__actor_さくら__:通常`（`:__actor_`＋サニタイズしたアクター名＋`__:`＋単語名） | `register_actor` |

ローカルとアクターのキーは `:` で始まり、グローバルのキーは `:` で始まらない。前方一致の範囲は、この先頭の `:` とスコープ名の後ろの `:` で区切られる。たとえば `:メイン1:` で始まるキーに `:メイン10:…` は含まれない。同じ単語キーに `entry` を複数回呼ぶと、同じキーの `WordEntry` が複数でき、索引の同じキーに登録順で並ぶ。

### 照合規則の共有

登録と検索は、同じ照合規則 `SceneRegistry::sanitize_name` を共有する。`sanitize_name` は、Unicode の英字・数字（`char::is_alphanumeric`。かな・漢字を含む）と `_` 以外の文字を 1 文字ずつ `_` に置き換える（`WordDefRegistry::sanitize_name` は同じ関数を呼ぶ）。利用者向けの章が「照合用の名前」と呼ぶものは、この関数でサニタイズした名前である（[シーン名の照合](../grammar/call-jump.md#シーン名の照合)）。

| 側 | サニタイズする名前 | サニタイズする所 |
| -- | ------------------ | ---------------- |
| 登録 | グローバルシーンの基本名、名前付きローカルシーンの名前（`_番号` を付ける前） | 生成器（`crates/pasta_lua/src/code_gen/scope_gen.rs`）。トランスパイル時の `register_global`・`register_local` も同じ |
| 登録 | ローカル単語・アクター単語のスコープ名 | `WordDefRegistry::register_local`・`register_actor` |
| 検索 | `search_scene` の第 1 引数（シーン名） | `SearchContext::search_scene` の入口 |
| 検索 | `search_word` の第 2 引数（スコープ） | `SearchContext::search_word` の入口 |

- サニタイズしないのは、`search_scene` の第 2 引数（親のグローバル名）、単語キー（`search_word` の第 1 引数と、登録の単語名）、`register_global_raw` が受け取る番号付きの名前である。第 2 引数と `register_global_raw` には、サニタイズ済みの基本名から作った登録名（`メイン1`）が渡る。
- サニタイズは何度かけても結果が変わらない。そのため、登録名（`メイン1`）をスコープとして `search_word` に渡しても、照合する名前は変わらない。
- A2 のスコープ名（`"__actor_" .. アクター名 .. "__"`）は元のアクター名から作られるが、`search_word` の入口でサニタイズされて `__actor_`＋サニタイズしたアクター名＋`__` になり、`register_actor` のキーのスコープと一致する。
- サニタイズ後の名前が同じになるグローバルシーン名・アクター名は、同じ名前として扱われる。グローバルシーンは同じ基本名のもとで通し番号が振られて同じ名前の候補になり、アクター単語は同じスコープのキーに入る。

### 前方一致による候補の収集

`SearchContext` の 2 つの検索メソッドは、第 2 引数（親のグローバル名）の有無で検索表の呼び分けを変える。

| 呼び出し | 検索表の処理 | 候補 |
| -------- | ------------ | ---- |
| `search_scene(名前, nil)` | `SceneTable::resolve_scene_id_unified("", サニタイズした名前)` → `collect_scene_candidates` | サニタイズした `名前` で前方一致したキーのうち `:` で始まらないもののシーン |
| `search_scene(名前, 親)` | `SceneTable::resolve_scene_id_unified(親, サニタイズした名前)` → `collect_scene_candidates` | `:親:サニタイズした名前` で前方一致したローカルシーンだけ |
| `search_word(名前, nil)` | `WordTable::search_word("", 名前)` → `collect_word_candidates` | `名前` で前方一致したキーのうち `:` で始まらないものの値すべて |
| `search_word(名前, 親)` | `WordTable::search_word(サニタイズした親, 名前)` → `collect_word_candidates` | `:サニタイズした親:名前` で前方一致したキーの値すべて |

- どの経路も、ローカルに候補が無いときにグローバルへ移ることはない。ローカルからグローバルへの順序は Lua 側の検索手順が組み立てる（後述「ローカル優先の検索順」）。
- 候補は RadixMap の `iter_prefix` が列挙する順に集まる。RadixMap はキーのバイト列の辞書順で列挙する。同じキーに複数の項目があれば、その中は登録順である。
- 単語の候補は項目ではなく値である。前方一致したすべての項目の値のリストを、上の順で 1 本につなげたものが候補になる。
- シーンでは、候補を集めた後に属性フィルタ（`filter_by_attributes`）をかける。`SearchContext` は常に空のフィルタを渡し、辞書確定は属性を空で登録するため、実行時の検索ではフィルタは候補を減らさない。
- シーン名が空文字列なら `InvalidScene` になる。単語名は空文字列を検査しない。

`search_scene` は、選ばれたシーンの関数名を最初の `::` で分け、`(グローバル名, ローカル名)` の 2 値を Lua へ返す。第 2 引数なしの検索ではローカル名を常に `__start__` にする。候補が無い・フィルタで全滅したなどの「見つからない」エラーは、Lua には値を返さない（受け取る側は `nil`）形になる。それ以外のエラー（空の名前など）は `Scene search error: …` の Lua エラーになる。`search_word` は見つかれば値の文字列、見つからなければ何も返さない。

### 候補の選択と乱数

候補が複数ある場合の選択は、シャッフルした候補を先頭から 1 つずつ使う方式（シャッフルと順次消費）である。選択の状態は検索表の中のキャッシュに持つ。

| | シーン（`SceneTable::select_from_cache`） | 単語（`WordTable::search_word`） |
| - | ---------------------------------------- | -------------------------------- |
| キャッシュのキー | `SceneCacheKey`（親のグローバル名・サニタイズした検索キー・整列したフィルタ。第 2 引数なしは親を空文字列） | `WordCacheKey`（サニタイズしたスコープ名・検索キー。第 2 引数なしは空文字列） |
| 初回 | 候補の ID をシャッフルしてキャッシュし、先頭を返す | 候補を集めてシャッフルし、先頭を返して残りをキャッシュする |
| 2 回目以降 | 次の ID を返す | 候補を集め直さず、キャッシュの次の値を返す |
| 一巡した後 | 同じ候補の並びをシャッフルし直して先頭から使う | 候補を集め直してシャッフルし、新しいキャッシュで置き換える |

- 一巡するまで同じ候補は選ばれない。一巡の境目では、前の巡の最後と次の巡の最初が同じになりうる。
- シーンのキャッシュは選んだ ID の履歴（`history`）も記録するが、選択には使わない。
- キャッシュは `SearchContext` の寿命の間、イベントをまたいで保たれる。辞書確定で `SearchContext` が作り直されると消える。
- 第 2 引数あり・なしはキャッシュのキーが異なる。ローカル優先の検索順で同じ名前をローカル・グローバルの順に引いても、2 つの選択状態は独立に進む。

乱数は `RandomSelector` の `shuffle_usize` だけを通して使う。`SearchContext::new` は、シーンと単語にそれぞれ別の `DefaultRandomSelector` を与える。Lua の `set_scene_selector(n1, …)`・`set_word_selector(n1, …)` は、引数があれば `MockRandomSelector`、無ければ新しい `DefaultRandomSelector` を作り、検索表の `replace_selector` で差し替える。差し替えはキャッシュを消す。`MockRandomSelector` の `shuffle_usize` は何もしないため、モックに差し替えた後の候補は収集した順（キーのバイト列の辞書順）のまま使われ、渡した整数の値は検索表の選択に影響しない。検索表にはシャッフルの有無を切り替える `set_shuffle_enabled` もあるが、Lua からは呼べない（テスト用）。

### ローカル優先の検索順

Rust 側は 1 回の呼び出しで 1 つのスコープしか検索しないため、利用者向け章が定める検索順は Lua 側の検索手順が組み立てる。単語参照・Call・式関数の呼び出しは、いずれも `ACT_IMPL.find_act_handler(mode, key, skip_methods)` を通る（`mode` は `"word"`・`"scene"`・`"expr"`。`skip_methods` は後述「動的参照の検索」）。

```text
ACT_IMPL.find_act_handler(mode, key, skip_methods)
  L1  current_scene[key]                             完全一致
  L2  ローカル辞書の前方一致                         @pasta_search があり、実行中のシーンがあるとき
        word        → SEARCH:search_word(key, グローバル名)
        scene/expr  → SCENE.search(key, グローバル名) → .func
  L3  self[key] が関数なら採用                       ACT のメソッド
  L4  GLOBAL[key]                                    完全一致
  L5  グローバル辞書の前方一致                       @pasta_search があるとき
        word        → SEARCH:search_word(key, nil)
        scene/expr  → SCENE.search(key, nil) → .func

PROXY_IMPL.find_handler(mode, key, skip_methods)     act:actor_proxy("アクター"):word(…) など
  A1  actor[key]                                     word モードだけ。完全一致
  A2  SEARCH:search_word(key, "__actor_" .. アクター名 .. "__")
                                                     word モードだけ
  →   act:find_act_handler(mode, key, skip_methods)  L1〜L5 へ委譲
```

- L2 のグローバル名は、実行中のシーンテーブルの `__global_name__`（`メイン1` の形）である。`ACT_IMPL.init_scene` がシーン関数の先頭で `current_scene` を設定する。これがローカルキー `:メイン1:…` の前方一致になる。
- A2 のスコープ名 `__actor_アクター名__` は、`search_word` の入口でサニタイズされてから `:__actor_サニタイズしたアクター名__:キー` の前方一致になり、`register_actor` のキーの形式と対応する（[照合規則の共有](#照合規則の共有)）。
- `@pasta_search` の取得は、`find_act_handler`・`find_actor_handler` とも呼び出しごとの `pcall(require, "@pasta_search")` で行う。
- 見つかった値の後処理（関数なら呼ぶ、それ以外は文字列にする、見つからなければ警告ログ）は `ACT_IMPL.word`・`ACT_IMPL.call`・`ACT_IMPL.expr_fn`・`ACT_IMPL.expr_fn_var` と、PROXY 側の `word`・`expr_fn`・`expr_fn_var` が行う（[名前の解決のメソッド](internal-modules.md#名前の解決のメソッド)）。
- グローバル関数の呼び出し（`＠＊名前（…）`）の生成コード `act:global_fn("名前", …)` は、この検索手順を通らず `GLOBAL[名前]` だけを引く（[生成コード用のメソッド](internal-modules.md#生成コード用のメソッドactor_proxyglobal_fnarith)）。

`ACT_IMPL.find_scene` は第 2 引数のグローバル名を使わず `find_handler("scene", key)` に委ねる。そのため、親を指定してローカルシーンを引く必要がある選択肢イベント（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/choice_select.lua`）と、ローカルシーンを対象とするシーンキック（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/kick.lua`）は、`SCENE.search(名前, 親)` を直接呼んでコルーチンを作る（グローバルシーンのキックは `SCENE.co_exec` を使う）。

### 動的参照の検索

動的参照（`＠＄変数名`・`＠＄変数名（…）`）では、変数の値が検索キーになる。生成コードは `word(値, "変数の経路")`・`expr_fn_var(値, "変数の経路", 引数…)` を呼び、値は `WORD.dynamic_key` が検索キーに変換する（変換規則は [WORD.dynamic_key(value, var_path, via)](../lua/script-api.md#worddynamic_keyvalue-var_path-via)、置き場所は [動的参照のキー](internal-modules.md#動的参照のキーworddynamic_key)）。キーに変換した後は、`skip_methods` を真にして同じ検索手順を通る。

```text
skip_methods が真のとき（動的参照）
  A1  rawget(actor, key)              アクターオブジェクト自身のフィールドだけ
  A2  静的と同じ
  L1  rawget(current_scene, key)      シーンテーブル自身のキー（__global_name__・シーン関数）だけ
  L2  静的と同じ
  L3  探さない
  L4  静的と同じ（GLOBAL[key]）
  L5  静的と同じ
```

- L3 を飛ばすのは、変数の値が ACT のメソッド名（`talk`・`yield` など）と一致しても、そのメソッドを呼ばないためである。L1・A1 を `rawget` で引くのは、シーンテーブルの `SCENE_TABLE_IMPL`・アクターオブジェクトの `ACTOR_IMPL` からメタテーブル経由で継承した `create_word` などに一致させないためである。
- `GLOBAL` はメタテーブルを持たない表として、静的と同じく `GLOBAL[key]` で引く。値が `GLOBAL` のキー（ランタイムが登録する `yield`・`チェイントーク`・`close_ghost`・`ゴースト終了` を含む）と一致すれば、その値が見つかる。
- 前方一致の段（L2・L5・A2）に渡すキーは、静的な参照で同じ名前を書いた場合と同じ文字列である。選択状態のキャッシュのキーも同じになるため、`＠＄x`（値が `挨拶`）と `＠挨拶` は同じスコープでは 1 つの巡回を共有する。

### 検索結果からシーン関数へ

`SCENE.search(name, global_scene_name)` は `@pasta_search` の `search_scene` が返した `(グローバル名, ローカル名)` で Lua 側のシーンテーブルから関数を引き、結果オブジェクトにして返す（手順と結果オブジェクトの形は [search と結果オブジェクト](internal-modules.md#search-と結果オブジェクト)）。Rust 側が返すのは名前だけであり、シーン関数そのものは Rust を通らない。`SCENE.co_exec` は `act:find_scene` で得た関数をコルーチンで包む。コルーチンの実行は [ランタイム実行モデル](execution-model.md) で扱う。

## 境界の受け渡し

| 境界 | 渡す側 → 受ける側 | 渡すもの | 所有 |
| ---- | ----------------- | -------- | ---- |
| Lua → Rust（辞書確定） | `pasta.scene`・`pasta.word` → `collect_scenes`・`collect_words` | `get_all_scenes()`・`get_all_words()` が返す Lua 側のテーブル | テーブルは Lua 側が所有し続ける。Rust は読むだけで変更せず、名前と値を `String` に写して `Vec` に集める。シーン関数（Lua の関数）は写さない |
| 辞書確定 → 検索 | `build_scene_registry`・`build_word_registry` → `SearchContext::new` | `SceneRegistry`・`WordDefRegistry` | 値渡しで消費され、検索表に変換される |
| Rust → Lua（登録） | `search::register` → Lua VM | `SearchContext` のユーザーデータ | Lua VM がユーザーデータを所有し、`package.loaded["@pasta_search"]` から参照される。置き換えられた古いユーザーデータは、Lua から参照されなくなれば回収される |
| Lua → Rust（検索） | `SCENE.search`・`find_act_handler`・`find_actor_handler` → `SearchContext` | 名前と親のグローバル名（文字列） | 検索のたびに `SearchContext` のキャッシュを更新する（可変メソッド） |
| Rust → Lua（検索結果） | `SearchContext` → 呼び出し側 | シーンは `(グローバル名, ローカル名)`、単語は値の文字列 | 文字列の複製を返す。シーン関数への解決は Lua 側（`SCENE.search`）が行う |
| `pasta_core` ↔ `pasta_lua` | レジストリ型と検索表の利用 | `SceneRegistry`・`WordDefRegistry`・`SceneTable`・`WordTable` | `pasta_core` は Lua に依存しない。Lua との接続は `pasta_lua` の `search` と `runtime/finalize.rs` だけが持つ |

## 不変条件と制約

- 検索の権威は辞書確定後の `SearchContext` である。VM の構築から辞書確定までの間（`main.lua`・`entry.lua` の実行中）は、トランスパイル時のレジストリから作った `SearchContext` が登録されており、グローバル名の形式（`メイン_1`）も内容も確定後と異なる（[@pasta_search の利用できる時期](../lua/modules/pasta-search.md#利用できる時期)）。
- 辞書確定は `package.loaded["@pasta_search"]` を新しいユーザーデータで置き換える。それ以前に `require` して保持した参照は古い `SearchContext` を指したままになる。ランタイムの Lua コードは、`SCENE.search` が呼び出し時に、`find_act_handler`・`find_actor_handler` が呼び出しごとに取得し直す。
- Rust 側の検索は、1 回の呼び出しで 1 つのスコープだけを検索し、ローカルからグローバルへ移らない。
- 第 2 引数なしの `search_scene` は、`:` で始まるローカルのキーを除外する。渡した名前の `:` はサニタイズで `_` になるため、`:` で始まる名前を渡しても、ローカルシーンは候補にならない。
- 登録キーと検索に使う名前は、同じ `sanitize_name` でサニタイズされる（[照合規則の共有](#照合規則の共有)）。登録では、グローバルシーンの名前は生成コードがサニタイズした基本名に番号を付けたもの、ローカルシーン名はサニタイズしたローカル名に `_番号` を付けたもの、ローカル単語・アクター単語のスコープ名は登録時にサニタイズされる。検索では、`search_scene` の名前と `search_word` のスコープが入口でサニタイズされる。`search_scene` の第 2 引数と単語キーはサニタイズされない。
- 候補の列挙順はキーのバイト列の辞書順で決まり、`SceneId` の値（辞書確定では `HashMap` の走査順で決まる）には依存しない。
- 検索表は構築後に項目を追加・削除しない。変化するのは選択状態のキャッシュと `RandomSelector` だけである。
- `SearchContext` は Lua VM ごとに 1 つであり、検索表の選択状態はその VM の中だけで共有される。
- 実行時の検索では属性フィルタは効かない（属性は空で登録され、フィルタも空で渡される）。

## ソースの所在

- `crates/pasta_core/src/`
- `crates/pasta_lua/src/search/`
- `crates/pasta_lua/src/runtime/finalize.rs`
- `crates/pasta_lua/pasta_scripts/pasta/scene.lua`
- `crates/pasta_lua/pasta_scripts/pasta/word.lua`

検索の入口として本文で参照した Lua 側の検索手順は `crates/pasta_lua/pasta_scripts/pasta/act.lua`・`crates/pasta_lua/pasta_scripts/pasta/actor.lua` に、`finalize_scene` のスタブは `crates/pasta_lua/pasta_scripts/pasta/init.lua` にあり、辞書確定を呼ぶ順序は `crates/pasta_lua/src/runtime/factory.rs` が決める。テストは `crates/pasta_core/tests/word_table_test.rs`、`SceneTable` のテストは `crates/pasta_core/src/registry/scene_table_candidate_tests.rs`・`crates/pasta_core/src/registry/scene_table_resolve_filter_tests.rs` にある。

## 経緯

- [pasta_search_module](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta_search_module) — `@pasta_search` による検索の公開
- [scene-search-integration](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/scene-search-integration) — `SCENE.search` による Lua 側シーン検索の統合
- [pasta-scene-dictionary-finalization](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-scene-dictionary-finalization) — 実行時の辞書確定（Lua 側の収集から検索表の構築）
- [local-scene-act-call](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/local-scene-act-call) — 辞書確定経路でのローカルシーン名の解決
- [act-word-global-dict-search](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/act-word-global-dict-search) — Rust 側の自動フォールバックの廃止と、スコープ別の検索
- [handler-resolution-fallback](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/handler-resolution-fallback) — `find_handler` による検索順の統一
- [actor-dict-word-shuffle](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/actor-dict-word-shuffle) — アクター単語のシャッフルと順次消費
- [audit-pasta-core](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/audit-pasta-core) — `pasta_core` の監査と簡素化

---

名前ひとつ引くのにも、これだけの段取りがございますのよ。おほほ、奥が深いでしょう？
次は、見つけたシーンがどうやって動き出すのか、実行モデルへ参りましょう！
