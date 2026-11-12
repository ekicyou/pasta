# 開発用のイベントとシーンマップ

> 【考え中】動いているゴーストに、外から「このシーンを再生して」「保存変数を見せて」と頼める――便利ですけれど、本番で誰にでも開いていては困りますわね。

> 【アンソニー】そのための門が `[debug] enabled` でございます。閉じていれば、何を頼まれても 204 でお断りいたします。

> 【にっこり】門を開けても、ポートは取らず JIT も止めませんのよ。では、門とシーンマップの仕掛けを分解してお見せいたしますわ。

---

この章では、開発の道具（VSCode の拡張・SSP や areka の MCP で `raise_event` を送る AI エージェント）が使う、開発用のイベントの門と、位置からシーンを引くシーンマップを扱う。あわせて、止まるエラーの場所の決め方を扱う。利用者から見た有効化の手順と開発の道具の使い方は [デバッグ概要](../debug/index.md) 以下の章が正である。

## 目的と責務

デバッグを有効にすると、pasta.dll は開発用のイベント（`OnPastaScene`・`OnPastaSceneAt`・`OnPastaDumpVariables`）を許し、読み込みのときにシーンマップを作る。ポートを取らず、JIT も止めないので、無効のときと同じ速さで動く。

この章が責務を持つのは次の事項である。

- 有効化の判定（`resolve_enabled`）と、ランタイムが有効を持つ形
- 開発用のイベントの門（`pasta.shiori.event.scene_play` の `allowed`）と既定のハンドラ
- 内部モジュール `@pasta_debug` の 4 つの関数（`enabled`・`scene_at`・`dump_variables`・`is_local_var_name`）
- シーンマップ: シーンの宣言の記録の作り方と、位置からシーンを引くシーン identity 索引
- 保存変数の書き出し（`variables.toml`）
- 止まるエラーの場所の決め方（`STOP.location` と `@pasta_chunks` の値）

次の事項は他の章が権威を持ち、この章では再記述しない。

- 開発用のイベントの引数・応答・失敗の出方は、利用者向けの [OnPastaScene](../lua/shiori-events.md#onpastascene名前でシーンを再生)・[OnPastaSceneAt](../lua/shiori-events.md#onpastasceneat位置でシーンを再生)・[OnPastaDumpVariables](../lua/shiori-events.md#onpastadumpvariablesグローバル変数を書き出す)・[開発用のイベントの共通の決まり](../lua/shiori-events.md#開発用のイベントの共通の決まり) が正である。`[debug]` の書き方は [pasta.toml リファレンス](../reference/pasta-toml.md) が正である。
- 止まるエラーの表記の形と、場所が出る・出ない場合の一覧は [止まるエラーの表記](../reference/runtime-failures.md#止まるエラーの表記) が正である。表記を作って応答にするまでは [止まるエラーの控えと応答](shiori.md#止まるエラーの控えと応答) で扱う。
- 生成器がシンクへ対応を記録する入口（`transpile_with_source_map`・`SourceMapSink`）は [ソースマップ生成の出入口](transpiler.md#ソースマップ生成の出入口) で扱う。
- ローダの段階構成の中でシーンマップを作る位置（段階 5.5）は [起動の段階構成](loader.md#起動の段階構成) で、`@pasta_chunks` を書き足す searcher は [require の解決（searcher）](loader.md#require-の解決searcher) で扱う。
- VM の構築順序の中で `@pasta_debug` の登録とシーン identity 索引の突合が入る位置は [VM の構築とモジュール登録](execution-model.md#vm-の構築とモジュール登録) で扱う。
- 再生したシーンのコルーチン（`STORE.co_preempt` の印）が中断中の `STORE.co_scene` を捨てる流れは [継続トーク（チェイントーク）と co_scene の更新](execution-model.md#継続トークチェイントークと-co_scene-の更新) で扱う。コルーチンを作る `SCENE.co_exec_exact`・`SCENE.co_exec_registered` は [co_exec_exact](internal-modules.md#co_exec_exact)・[co_exec_registered](internal-modules.md#co_exec_registered) で扱う。
- セーブの変換の規則（書ける値と書けない値）は [@pasta_persistence](../lua/modules/pasta-persistence.md#書ける値と書けない値) が正である。

## 構成要素

### Rust 側のモジュール

| 要素 | 所在 | 役割 |
| ---- | ---- | ---- |
| 入口 | `crates/pasta_lua/src/debug/mod.rs` | `resolve_enabled` の再輸出と、`source_map`・`playscene` のモジュール。`pasta_shiori` を参照しない |
| 有効化の判定 | `crates/pasta_lua/src/debug/config.rs` | `resolve_enabled`（`PASTA_DEBUG` ＞ `[debug] enabled` ＞ false）と、純粋な部分 `pick`・`parse_env_bool` |
| `[debug]` の型 | `crates/pasta_lua/src/loader/config/sections.rs` | `DebugFileConfig { enabled }`（`#[serde(default)]`。既定は false） |
| 位置の引き | `crates/pasta_lua/src/debug/playscene.rs` | `resolve_position`。`OnPastaSceneAt` の位置を登録名の組にする |
| シーンマップ | `crates/pasta_lua/src/debug/source_map/mod.rs` | `SourceMap`（シーンの宣言の記録と索引）・`MapBuilderSink`（シーンだけを控えるシンク）・`canonicalize_pasta_file` |
| シーン identity 索引 | `crates/pasta_lua/src/debug/source_map/scene_index.rs` | `.pasta` の（ファイル, 行）から `(scene_id, parent)` を引く `SceneIdentityIndex` |
| シーンの突合 | `crates/pasta_lua/src/debug/source_map/scene_join.rs` | `build_scene_index`。記録と実行時の登録名を、定義元の `.pasta` ごとに突き合わせる（`join_records`・`rank_globals`） |
| シーンマップの構築 | `crates/pasta_lua/src/loader/source_map_build.rs` | `PastaLoader::build_source_map` |
| 元のファイル | `crates/pasta_lua/src/loader/context.rs` | `LoaderContext::pasta_sources`（モジュール名 → `DicSource::Pasta`・`DicSource::Lua`） |
| 読み込んだファイルの表 | `crates/pasta_lua/src/runtime/searcher.rs` | `register_chunk_table`・`record_chunk`（`@pasta_chunks` を書き足す）と `pasta_file_of`（候補パス → `.pasta`） |
| `@pasta_debug` の登録 | `crates/pasta_lua/src/runtime/module_registry.rs` | `register_debug_module`。`enabled`・`scene_at`・`dump_variables`・`is_local_var_name` を持つ表を `package.loaded` に置く |
| 書き出し | `crates/pasta_lua/src/runtime/persistence.rs` | `dump_table`（セーブと同じ変換でテキストを書く）と `save_to_file`（一時ファイル → `rename`） |
| 突合の呼び出し | `crates/pasta_lua/src/runtime/factory.rs` | `from_loader_with_scene_dic` の最後で `build_scene_index` を呼び、索引を書き込む |

### Lua 側のモジュール

| モジュール | 役割 |
| ---------- | ---- |
| `pasta.shiori.event.scene_play` | 門 `allowed` と、3 つの開発用のイベントの既定のハンドラ |
| `pasta.shiori.stop` | `STOP.location`。止まるエラーの場所を `@pasta_chunks` の値と行の対応から決める |

## 処理とデータの流れ

### 有効化の判定

有効化の判定は `resolve_enabled(config.debug())` の 1 回だけで、ローダが段階 5.5 で呼ぶ。読む環境変数は `PASTA_DEBUG` だけである。値は `parse_env_bool` で読み、`1`・`true`・`yes`・`on` を真、`0`・`false`・`no`・`off`・空を偽とし（大小と前後の空白は問わない）、ほかの値は設定されていないのと同じに扱う。環境変数が読めれば `[debug] enabled` より優先し、どちらも無ければ偽である。

- `DebugFileConfig` の欄は `enabled` だけで、`deny_unknown_fields` を付けない。`[debug]` に残った古いキー（`port`・`present_as`・`source_map_sidecar`）は、`pasta.toml` のほかの節の知らないキーと同じく黙って読み飛ばされる。
- SHIORI の読み込み（リロードを含む）のたびに、ローダが `pasta.toml` を読み直して判定する。
- 判定が真のときだけ、ローダはシーンマップを作り、`Some(Arc<SourceMap>)` をランタイムに渡す。ランタイムの `source_map: Option<Arc<SourceMap>>` の有無が「デバッグが有効か」の唯一の値であり、別の真偽値は持たない。`PastaLuaRuntime::debug_enabled()` は `source_map.is_some()` を返し、`@pasta_debug.enabled()` も同じ値を返す。
- ローダを通らない VM（`PastaLuaRuntime::new`・`with_config`）は、いつも無効である。
- 有効なときは、読み込みの最後（`Startup sequence completed` の直前）に info のログを 1 行出す。

```text
Debug enabled: development events are allowed
```

### 開発用のイベントの門

`pasta.shiori.event.scene_play` は、3 つのイベントの既定のハンドラを `REG.X = REG.X or function …` の形で登録する。ゴーストが先に登録したハンドラは上書きしない。どのハンドラも、最初に門 `allowed(act, event, outcome)` を通る。

```text
allowed(act, event, outcome)
 1. pcall(require, "@pasta_debug") が失敗・表でない・enabled() が偽
      → log.warn("<event>: debug is disabled; <outcome>") → nil
 2. act.req.security_level == "external"
      → log.warn("<event>: SecurityLevel external is refused; <outcome>") → nil
 3. 通れば @pasta_debug の表を返す
```

`outcome` は `OnPastaScene`・`OnPastaSceneAt` が `scene not played`、`OnPastaDumpVariables` が `variables not dumped` である。門が `nil` を返すとハンドラも `nil` を返し、`EVENT.fire` が 204 にする。門は `scene_play.lua` のこの関数だけで、Rust の `@pasta_debug` の関数は有効かを見ない。

### @pasta_debug

`@pasta_debug` は本番の入口（`PastaLuaRuntime::from_loader_with_scene_dic`）だけが登録する。デバッグが無効でも登録し、`enabled()` が偽を返す。関数が閉じ込めるのは、シーンマップ（`Option<Arc<SourceMap>>`）と、ゴーストのフォルダー（`LoaderContext::base_dir`）である。

| 関数 | 振る舞い |
| ---- | -------- |
| `enabled()` | シーンマップを持つかの値 |
| `scene_at(file, line)` | 位置を登録名の組（グローバルシーンの登録名, ローカルシーンの登録名か `__start__`）に引く。見つからなければ `nil` |
| `dump_variables(save)` | `save` の表を `profile/pasta/debug/variables.toml` に書く。返り値は無い |
| `is_local_var_name(s)` | `s` の全体が `＄名前` と書けるローカル変数の名前に当たるかの値（`pasta_dsl::parser::is_id`）。Reference2 のキーの検査に使う |

どの関数もエラーを Lua に上げない（引数の型の誤りを除く。`scene_at` の `file` が文字列か数でないとき、`dump_variables` に表でない値を渡したとき、`is_local_var_name` に文字列か数でない値を渡したとき）。

### 位置でのシーンの再生

`OnPastaScene` は、名前を `SCENE.co_exec_exact` に渡してコルーチンを作る。シーンマップは使わない。`OnPastaSceneAt` は、ファイルと行を `@pasta_debug.scene_at` で登録名の組に引き、`SCENE.co_exec_registered` に渡す。どちらも、得たコルーチンに `STORE.co_preempt` の印を付けて返し、その要求の応答にする。行の文字列から数への変換と検査は、ハンドラが行う（半角数字だけの 1 以上）。Reference2 は、シーンを探す前に局所関数 `read_locals` が `@toml` で読んで検査し、コルーチンを得たときだけ、その値を `act.var` に入れる。

```text
scene_at(file, line)
  line が 1 以上 u32 に収まる整数でない、シーンマップが無い → nil
  resolve_position(map, ghost/master のパス, file, line)
    base_dir.join(file)        相対パスは ghost/master/ に結ぶ。絶対パス（UNC を含む）は join がそのまま返す
    std::path::absolute        失敗なら結んだパスのまま。canonicalize は使わない
    map.scene_at(パス, line)   シーン identity 索引
      グローバル → (scene_id, "__start__")
      ローカル   → (parent, scene_id)
      見つからない → nil
```

- 位置はファイルのパスとしてだけ扱う。`file://` で始まる文字列や `%41` のような並びも、パスの文字のまま扱い、URI として剥がしたり復号したりしない。
- `resolve_position` はファイルを開かない。区切りと Windows の大小は、`SourceMap::scene_at` の中の `canonicalize_pasta_file` がそろえる。
- 行からシーンを選ぶ規則は、後述の `SceneIdentityIndex::scene_at` である。ファイルの最後のシーンはファイルの末尾まで続くため、索引にあるファイルの行は必ずどれかのシーンに当たり、見つからないのは索引に無いファイルのときだけである。

### シーンマップ

#### 構築

シーンマップは、`.pasta` ごとのシーンの宣言の記録（照合の鍵 → 宣言の行）と、突合の後に書くシーン identity 索引だけを持つ。変換した Lua の行と `.pasta` の行の対応表は持たない。デバッグが有効なとき、ローダは段階 5.5 で `PastaLoader::build_source_map(全 .pasta, トランスパイラ)` を呼ぶ。

```text
build_source_map
 for .pasta ファイル
   読み込み → pasta_dsl::parse_str でパースし直す
   sink = MapBuilderSink::default()
   transpile_with_source_map(…, 捨てる出力, Some(&mut sink))   … 出力と返る LineShift は使わない
   SourceMap::insert_scenes(ファイルのパス, sink.into_scenes())
 → Arc<SourceMap>
```

- 読めない・パースできない・変換できないファイルは、警告（`source-map: skip (…)`）を出して飛ばす。
- 段階 4 の増分処理は更新の無いファイルを変換し直さないため、記録を得るには全部の `.pasta` を変換し直す。使うトランスパイラは段階 4 と同じもの（同じ別名表を持つ）で、記録の鍵の宣言名はキャッシュの `.lua` と一致する。
- `MapBuilderSink` は `record_scene` で（照合の鍵, `span.start_line`）を宣言順に積み、`record_line` の通知は捨てる。宣言の行は `.pasta` の行なので、`LineShift` での付け替えは要らない。
- シンクの有無で出力のバイト列は変わらない。変換した Lua のキャッシュ（`profile/pasta/cache/lua/`）の中身はデバッグの有無で変わらず、記録をディスクに書き出すことも無い。

#### パスの正規化

記録の鍵と問い合わせのパスは、どちらも `canonicalize_pasta_file` を通して突き合わせる。

1. 先頭の `@` を除く。
2. `\` を `/` に置き換える。
3. Windows では小文字にする（Windows 以外では大小を保つ）。

ファイルシステムには問い合わせない。記録の側（段階 5.5 のファイルのパス）と問い合わせの側（`resolve_position` が絶対にしたパス）は、どちらも同じ絶対パスのゴーストのフォルダーから作る（[モジュール検索パス](loader.md#モジュール検索パス)）。

#### シーン identity 索引

`.pasta` の（ファイル, 行）を、実行時のシーンの登録名（`(scene_id, parent)`）に引くには、全ファイルを読み込んだ後でないと決まらない通し番号が要る。グローバルシーンの登録名は、照合用の名前・`_`・通し番号からなり（`メイン_1`。[シーンテーブル](internal-modules.md#シーンテーブル)）、通し番号はシーンモジュールの読み込み時に `create_scene` が全ファイルを通して振る。そこで、変換のときに再現できる照合の鍵を記録し、辞書確定の後に突き合わせる。

- 生成器は `record_scene` で、グローバルシーンを `G:{サニタイズした名前}#{出現順}`、名前付きローカルシーンを `L:{親のサニタイズした名前}#{親の出現順}:{関数名}` として記録する（関数名は `挨拶_1` の形の登録名か `__start__`）。出現順は、その `.pasta` の中でのサニタイズした名前ごとの順（ファイルごとに 1 から）であり、実行時の通し番号とは一致しない。
- 鍵の名前は宣言名（pasta.toml の別名表で置き換えた後の名前。[登録と生成](transpiler.md#登録と生成単一走査)）をサニタイズしたものである。既定の別名表のもとで `＊会話` は `G:OnTalk#{出現順}` と記録され、同じファイルの `＊OnTalk` と出現順を共有する。実行時の登録名も宣言名から作るため、突き合わせは別名を意識せずに成り立つ。
- ランタイムの構築の最後（`pasta.scene_dic` の `require` の後）に、ファクトリが `build_scene_index(lua, map, pasta_file_of)` を呼ぶ。`build_scene_index` は `get_all_scenes()` のシーン表を 1 度走査し、グローバルシーンの登録名ごとに、配下の関数名と定義元の `.pasta` を集める。定義元は、シーンテーブルの関数（`__start__` を優先し、無ければ任意の関数）の `Function::info().source` を、引数の関数で引いたものである。
- ファクトリが渡す関数は、`@` を除いた候補パスで `searcher::pasta_file_of` を引き、返った相対パスをゴーストのフォルダーに結ぶ。`pasta_file_of` は、searcher が `.pasta` のキャッシュを読み込んだときに控えた「候補の全体のパス → `.pasta` のパス」の表を引く。縮めたチャンク名でなく全体のパスで引くので、`@pasta_chunks` のキーの重複に左右されない。引けないシーン（作者の `.lua` で作ったシーン・関数を 1 つも持たないシーン）は突き合わせの対象にしない。
- 突き合わせ（`join_records`）は定義元の `.pasta` ごとに行う。`rank_globals` が、そのファイルの登録名を `SceneRegistry::split_registered_name` で（照合用の名前, 通し番号）に分け、名前ごとに通し番号の昇順で 1 から順位を振る。`G:{名前}#{k}` の記録は、そのファイルの `名前` の k 番目の登録名に突き合わせる。ファイルの中の定義順と実行時の通し番号の大小が一致することだけを使い、ファイルの読み込み順には頼らない。そのため、複数の `.pasta` に同じ名前のグローバルシーンがあっても取り違えない。`＊章` の 11 個目（`章_11`）と `＊章11` の 1 つ目（`章11_1`）も、（`章`, 11）と（`章11`, 1）に分かれて取り違えない。
- `L:` の記録は、親を同じ方法でそのファイルの登録名にし、その配下に同じ関数名のローカルシーンがあるときだけ採る。`__start__` の記録と、突き合わなかった記録は索引に入れない。
- 各シーンの範囲は、同じファイルの宣言の行の昇順で「次の同じレベル以上の宣言の前の行」まで（無ければファイルの末尾まで）とし、`SourceMap::set_scene_index` で 1 度だけ書き込む。突合の失敗は警告だけで、索引なしで起動を続ける。

`SceneIdentityIndex::scene_at(ファイル, 行)` は次の順で引く。

1. 行を範囲に含むシーンのうち、最も内側（ローカル）のもの。
2. 含むシーンが無ければ、その行以降で最も近い宣言のシーン。
3. それも無ければ見つからない。索引が書かれる前や、ファイルが索引に無いときも見つからない。

### 保存変数の書き出し

`OnPastaDumpVariables` の既定のハンドラは、門を通れば `debug.dump_variables(require("pasta.save"))` を呼び、いつも `nil` を返す（喋らずに 204）。引数（Reference）は見ない。

```text
dump_variables(save)
 dump_table(lua, save, ghost/master/profile/pasta/debug/variables.toml)
   lua_table_to_toml_document → 書けないキーごとに warn「key '<キー>' was not saved: <理由>」
   save_to_file(文書, パス, obfuscate = false)
     親のフォルダを作る → variables.tmp に書いて sync_all → rename で置き換える
     どこで失敗しても variables.tmp を消す
 Err なら warn「OnPastaDumpVariables: could not write <パス>: <理由>」（Lua には上げない）
```

- 変換はセーブ（`save.toml`）と同じ `lua_table_to_toml_document` で、外したキーの警告もセーブと同じ関数が出す。`[persistence] obfuscate` によらず、いつもテキストの TOML で書く。
- `dump_table` はセーブの門（`SaveGate`）を見ず、表も変えない。保存変数・`save.toml`・`STORE.co_scene` は変わらない。
- 書き出しは要求の中で同期に終わり、204 はファイルを書き終えた（か、書けずに警告を出した）後に返る。書けなかったときは、最後のファイルを `rename` まで触らないので、前の `variables.toml` が残る。読む側は、送る前と後でファイルの更新時刻を比べて、新しく書かれたかを確かめる。

保存変数が `＄＊好感度＝３`・`＄＊名前＝「まさお」` のときの中身は次のとおりである。

```toml
好感度 = 3
名前 = "まさお"
```

### 止まるエラーの場所

止まるエラーの場所は、デバッグの有無によらず、`@pasta_chunks` の値と、`act` が控えた行の対応（`"<.pasta のファイル>|<lua の行>:<pasta の行>,…"`。生成コードにいつも埋まる）だけで決める。シーンマップは使わず、`pasta.shiori.stop` は `@pasta_debug` を読まない。

`@pasta_chunks` のキーはエラー文の形のチャンク名で、値は searcher が `LoaderContext::pasta_sources` を引いて決める。

| 値 | 意味 |
| -- | ---- |
| 文字列（`scripts\天気.lua`） | ゴーストのフォルダーの中の作者の `.lua`。ゴーストのフォルダーから見たパス |
| 文字列（`dic\道具.lua`） | `dic/` の `.lua` をキャッシュに写したもの。元の `.lua` のパス |
| 表 `{ pasta = "dic\\天気.pasta" }` | その `.pasta` から変換したキャッシュ |
| `false` | 2 つのファイルが同じキーになった（見分けられない） |
| （無い） | 標準ランタイム・ゴーストの外・`pasta_sources` に無いキャッシュ |

- ローダは段階 4 の後に `pasta_sources` を詰める（`.pasta` は `DicSource::Pasta`、`dic/` の `.lua` は `DicSource::Lua`。値は `CacheManager::get_relative_path` の形）。`.pasta` と `.lua` は値の型で見分け、拡張子では見分けない。値のパスの区切りは `\` にそろえる。
- `dic/` の `.lua` の写しは元とバイトで同じなので、エラー文の行がそのまま元の `.lua` の行になる。

```text
STOP.location(chunk, line, chunks, line_map)
 entry = chunks[chunk]
 文字列 → "<entry>:<line>"
 表     → pasta_location(line_map, line, entry.pasta)
            行の対応のファイルと entry.pasta を、/ を \ にそろえて比べる
            違う → nil（実行中のシーンとは別の .pasta の行を取り違えない）
            同じ → 対応に line があれば "<ファイル>:<pasta の行>"、無ければ nil
 ほか   → nil
```

`nil` のときは場所を出さずにエラー文だけを出す。別のグローバルシーンや別のファイルの Lua のブロックの関数の中で止まったときは、デバッグが有効でも場所を出さない。

## 境界の受け渡し

| 境界 | 渡す側 → 受け取る側 | 渡すもの | 所有 |
| ---- | ------------------- | -------- | ---- |
| 設定 → ローダ | `PastaConfig::debug()` → `resolve_enabled` | `Option<DebugFileConfig>` | 判定は読み込みごとに 1 回 |
| `code_gen` → `debug` | 生成器 → `MapBuilderSink` | `record_scene` の呼び出し（`record_line` は捨てる） | 依存は `debug` → `code_gen` の向きで、`SourceMapSink` は `code_gen` にある |
| ローダ → ランタイム | `build_source_map` → `from_loader_with_scene_dic` | `Option<Arc<SourceMap>>` | ランタイムが持ち、`@pasta_debug` の関数が `Arc` を閉じ込める |
| ローダ → searcher | `LoaderContext::pasta_sources` → `register_chunk_table` | モジュール名 → `DicSource` | `ChunkTable`（VM の app data）が持つ |
| searcher → ファクトリ | `pasta_file_of` → `build_scene_index` | 候補パス → `.pasta` の相対パス | ファクトリがゴーストのフォルダーに結ぶ関数で包む。`debug` は `runtime` を参照しない |
| ランタイム → シーンマップ | `build_scene_index` → `SourceMap::set_scene_index` | `SceneIdentityIndex` | 書き込みは 1 回だけ（`OnceLock`） |
| Rust → Lua | `@pasta_debug` → `pasta.shiori.event.scene_play` | 真偽値・登録名の組か `nil`・書き出しの副作用 | シーン関数は Rust を通らない |
| searcher → Lua | `@pasta_chunks` → `pasta.shiori.stop` | チャンク名 → 値の表 | VM が持ち、searcher だけが書き足す |

## 不変条件と制約

- デバッグは opt-in である。無効のとき、ローダはシーンマップを作らず、開発用のイベントはどれも 204 と警告 1 行になる。有効でもポートを取らず JIT も止めないので、デバッグを有効にした 2 体が並んでも互いに影響しない。
- 有効を表す値は、ランタイムがシーンマップを持つかだけである。`debug_enabled()` と `@pasta_debug.enabled()` は食い違わない。
- 変換した Lua のキャッシュは、デバッグの有無で変わらない。シーンマップのための変換し直しは、出力を捨てる。
- `SourceMap` は構築後に変わらない。シーン identity 索引だけが 1 回書き込まれる。
- 止まるエラーの場所の決め方は、デバッグの有無で変わらない。
- `dump_variables` は表を変えず、セーブの門を通らない。書けないときも 204 で、前のファイルを消さない。
- テストの環境: 開発環境で `PASTA_DEBUG=1` が設定されていると、`PastaLoader::load` を呼ぶテストのデバッグが有効になり、「無効なら 204」を確かめるテストなどの結果が変わる。そのため、読み込みを伴う結合テストは `#[ctor]` で `main` の前に `PASTA_DEBUG`・`PASTA_LOG` を消している（`crates/pasta_lua/tests/common/mod.rs` など）。読み込みを伴うテストを足すときは同じ処置が要る。クレート内の単体テストは、環境変数を読まない `pick` を確かめる。

## ソースの所在

- `crates/pasta_lua/src/debug/`
- `crates/pasta_lua/src/loader/source_map_build.rs`
- `crates/pasta_lua/src/runtime/module_registry.rs`
- `crates/pasta_lua/src/runtime/searcher.rs`
- `crates/pasta_lua/src/runtime/persistence.rs`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/event/scene_play.lua`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/stop.lua`

テストは `crates/pasta_lua/src/debug/` の `*_tests.rs`・`crates/pasta_lua/tests/loader_source_map_build_test.rs`・`crates/pasta_lua/tests/scene_identity_index_test.rs`・`crates/pasta_lua/tests/scene_play_event_test.rs`・`crates/pasta_lua/tests/debug_env_ignored_test.rs`・`crates/pasta_lua/tests/lua_specs/scene_play_event_test.lua`・`crates/pasta_lua/tests/lua_specs/stop_test.lua`・`crates/pasta_lua/src/runtime/persistence_tests.rs` にある。

## 経緯

- [pasta-source-map](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-source-map) — `.pasta` と生成 `.lua` の対応の記録（`SourceMapSink`）
- [pasta-scene-kick-from-cursor](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-scene-kick-from-cursor) — シーン identity 索引
- [scene-play-event](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/scene-play-event) — `OnPastaScene`・`OnPastaSceneAt` と `@pasta_debug`
- [runtime-error-balloon](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/runtime-error-balloon) — 止まるエラーの表記と場所
- [debug-mcp-runtime](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/debug-mcp-runtime) — 開発用のイベントの門・シーンマップ・`OnPastaDumpVariables`・止まるエラーの場所の決め方

---

> 【したり顔】門が 1 つ、引き方が 1 つ――仕掛けが小さいほど、壊れる所も少ないのですわ。最後は裏方の要、ロギングとエンコーディングへ参りましょう。

> 【アンソニー】デバッグを切っておけば、何を頼まれても 204 でお断りするだけというのも、心強うございます。
