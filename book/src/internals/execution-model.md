# ランタイム実行モデル

> 【したり顔】シーンが途中で言葉を止め、次のイベントで続きから話し出す――あの不思議な振る舞いの種明かしをいたしますわ。

> 【アンソニー】黙っている間、話の続きはどこにしまってあるのでございましょう。

> 【クローディア】それを追いかけるのが、この章の道のりですの。VM の組み立てからコルーチンの回し方、永続化まで、わたくしについていらっしゃい。

---

この章では、Lua VM の構築と、イベントからシーンを実行するランタイム実行モデルを扱う。

## 目的と責務

ランタイム実行モデルは、Lua VM を構築してモジュールを登録し、SHIORI のイベントごとにシーン関数をコルーチンとして起動・再開して、応答の文字列を作る。

この章が責務を持つのは次の事項である。

- `PastaLuaRuntime` による Lua VM の構築と、Rust 製モジュール（`@pasta_*` など）の登録の順序
- イベントからシーンへのコルーチン実行。コルーチンの作り方、再開のループ（`resume_until_valid`）、継続トーク（チェイントーク）のための `STORE.co_scene` の更新規則、コールバック待ちのコルーチンとの関係
- ランタイム内部モジュール（ACT・STORE・SCENE・WORD・GLOBAL・SAVE）の関係と、実行中にどれが何を持つか
- 永続化（`@pasta_persistence` の実装と、`save` テーブルの読み込み・保存のタイミング）
- Lua 方言（LuaJIT 2.1）がこの実行モデルに課す制約

次の事項は他の章が権威を持ち、この章では再記述しない。

- 起動シーケンスの利用者から観測できる挙動（読み込むモジュールの順序・失敗時の扱い）は [起動シーケンスとモジュール解決](../reference/startup.md#2-起動シーケンス) が正である。VM 構築より前の段階（設定読込・自己展開・トランスパイル）と `require` の解決は [ローダ自己展開とモジュール解決](loader.md) で扱う。
- `@pasta_search` の中身と辞書確定（`finalize_scene`）は [シーン・単語レジストリとシーン検索](registry-search.md) で扱う。
- イベントの振り分け（`REG`・シーン関数フォールバック・コールバックのルーティング・仮想イベント）と、VM を所有するスレッドは [SHIORI 層](shiori.md) で扱う。
- 各モジュールのフィールド・関数・データ構造（STORE のフィールド、`finalize_scene` の収集、ACT の初期化など）は [Lua ランタイム内部モジュール](internal-modules.md) で扱う。
- ACT のトークをさくらスクリプトにする処理（`build`）は [トーク出力とアピアランス](talk-output.md) で扱う。
- 利用者から観測できるチェイントークの挙動は [Call / Jump](../grammar/call-jump.md#チェイントーク) が、`@pasta_persistence` の API と `[persistence]` の設定は [@pasta_persistence](../lua/modules/pasta-persistence.md) が正である。

## 構成要素

### Rust 側（PastaLuaRuntime）

`PastaLuaRuntime`（`crates/pasta_lua/src/runtime/mod.rs`）が Lua VM（`mlua::Lua`）を 1 つ所有する。インスタンスごとに VM と `@pasta_search` は独立しており、複数のインスタンスは互いに干渉しない。実装は `impl PastaLuaRuntime` をファイルごとに分けて書いている。

| ファイル | 役割 |
| -------- | ---- |
| `crates/pasta_lua/src/runtime/mod.rs` | 構造体の定義と、VM を作る `new`・`with_config`・`with_config_and_source_map` |
| `crates/pasta_lua/src/runtime/factory.rs` | ローダが使う構築経路 `from_loader_with_scene_dic`（本番）と `from_loader`（旧経路。テストが使う）、起動モジュールの `require` と致命扱い |
| `crates/pasta_lua/src/runtime/module_registry.rs` | `package.path` と searcher の設置、`@pasta_config`・`@enc`・`@pasta_persistence`・`@pasta_sakura_script`・`@pasta_log` の登録（`package.loaded` への格納） |
| `crates/pasta_lua/src/runtime/exec.rs` | 任意の Lua コードの実行（`exec`・`exec_named`・`exec_file`）と、VM・設定・ロガー・デバッグ状態の参照 |
| `crates/pasta_lua/src/runtime/lifecycle.rs` | `Drop` での永続化データの保存 |
| `crates/pasta_lua/src/runtime/persistence.rs` | `@pasta_persistence` の実装（読み込み・保存・形式判定・原子的な書き込み） |
| `crates/pasta_lua/src/runtime/finalize.rs` | `finalize_scene` の Rust 実装（[シーン・単語レジストリとシーン検索](registry-search.md#辞書確定rust-側)） |
| `crates/pasta_lua/src/runtime/runtime_config.rs`・`crates/pasta_lua/src/runtime/searcher.rs` | ライブラリ構成（`RuntimeConfig`）と `require` のファイル解決（[ローダ自己展開とモジュール解決](loader.md)） |
| `crates/pasta_lua/src/runtime/log.rs`・`crates/pasta_lua/src/runtime/enc.rs` | `@pasta_log`・`@enc` の実装（[ロギングとエンコーディング](logging-encoding.md)） |
| `crates/pasta_lua/src/runtime/renderer_injection.rs` | `@pasta_sakura_script` を注入されたレンダラで登録する継ぎ目（[SHIORI 層](shiori.md)） |

`PastaLuaRuntime` は VM のほかに、インスタンスのロガー、`pasta.toml` の設定（`PastaConfig`）、ゴーストのベースディレクトリ、デバッグバックエンドのハンドル、デバッグ用のソースマップを持つ。設定とベースディレクトリは、`Drop` で永続化データを保存するために保持している。

### Lua 側のモジュールの関係

ランタイムの Lua 側は `crates/pasta_lua/pasta_scripts/pasta/` のモジュール群である。中心となるモジュールの `require` の関係を示す（矢印は「require する」）。

```text
生成コード（pasta.scene.…）
  └→ pasta（init.lua。生成コードが呼ぶ公開 API）
        ├→ pasta.actor ──→ pasta.store, pasta.word, pasta.failure
        ├→ pasta.scene ──→ pasta.store, pasta.word, @pasta_search（search の呼び出し時）
        └→ pasta.word  ──→ pasta.store, pasta.failure

pasta.shiori.entry（SHIORI.load / request / unload / kick）
  ├→ pasta.shiori.stop
  └→ pasta.shiori.event（EVENT.fire・EVENT.drive）
        ├→ pasta.shiori.act ──→ pasta.act ──→ pasta.actor, pasta.scene, pasta.global,
        │                                     pasta.store, pasta.word, pasta.failure,
        │                                     pasta.save（ACT.new の呼び出し時）
        ├→ pasta.shiori.event.callback ──→ pasta.store, pasta.shiori.stop,
        │                                  pasta.shiori.event（再開の呼び出し時）
        ├→ pasta.shiori.stop
        └→ pasta.store

pasta.shiori.stop ──→ pasta.failure, pasta.shiori.res（@pasta_chunks・pasta.shiori.act・pasta.store は表記を作る時点で読む）
pasta.store ──→ （@pasta_config だけを pcall で読む。他の pasta.* を require しない）
pasta.save  ──→ @pasta_persistence
pasta.failure ──→ （@pasta_log・pasta.config を報告の時点で読む。他の pasta.* を読み込みの時点では require しない）
```

この図ではログ出力の `@pasta_log` を省いている（`pasta.act`・`pasta.actor`・`pasta.word` などがモジュールの読み込み時に `require` する）。

| モジュール | 実行モデルでの役割 |
| ---------- | ------------------ |
| `pasta`（`crates/pasta_lua/pasta_scripts/pasta/init.lua`） | 生成コードが呼ぶ `create_actor`・`create_scene`・`create_word` の窓口と、Rust が上書きする `finalize_scene` のスタブ |
| `pasta.store` | ランタイムの共有状態の置き場。アクター・シーン・単語の Lua 側の登録表、外見の状態、継続中のコルーチン（`co_scene`）、選択肢とキックのための状態を持つ。他の `pasta.*` を `require` しないため、どのモジュールからも循環なく参照できる |
| `pasta.scene` | シーン関数の登録と、検索結果からシーン関数を引く `SCENE.search`、シーン関数をコルーチンで包む `SCENE.co_exec`・`SCENE.co_exec_exact`・`SCENE.co_exec_registered` |
| `pasta.word`・`pasta.actor` | 単語とアクターの登録。アクターのプロキシ（`act.アクター名`）は `pasta.actor` が作る |
| `pasta.failure` | 実行時の失敗の報告の出口。ログの警告と失敗表記のトークンを作る（[実行時の失敗の出力](../reference/runtime-failures.md)・[失敗の報告（pasta.failure）](internal-modules.md#失敗の報告pastafailure)） |
| `pasta.act`・`pasta.shiori.act` | シーン関数が第 1 引数で受け取る ACT。トークの蓄積、名前の解決（`call`・`word` など）、`yield`。`pasta.shiori.act` は `pasta.act` を継承し、`build` をさくらスクリプトの生成に差し替え、`get_property` などの SHIORI 固有のメソッドを加える |
| `pasta.global` | 利用者が関数を足すグローバル関数の表。`yield` と `チェイントーク` を最初から持つ |
| `pasta.save` | 永続化データの表。`require` された時点で 1 回だけ `@pasta_persistence.load()` を呼び、その結果を返す |
| `pasta.shiori.event` | `EVENT.fire` と `EVENT.drive`。`EVENT.fire` はハンドラの戻り値を応答にする。`EVENT.drive` はシーンコルーチンの再開の手順（割り込みの種類の判定 → 出力が出るまで再開 → 予約の消費 → 種類と台詞の有無による `STORE.co_scene` の更新）と、中断のシーンの流し方（`run_interrupt_scene`）の持ち主で、`EVENT.fire`・`CALLBACK.try_route`・`CALLBACK.sweep` のすべてがこれを使う |
| `pasta.shiori.event.callback` | `get_property` のコールバック待ちのコルーチンを、待っているイベント名ごとに持つ。一致したコールバックの再開（`try_route`）と期限切れの掃引（`sweep`）を行い、再開そのものは `EVENT.drive` に任せる |
| `pasta.shiori.stop` | 止まるエラーの控えと表記付きの応答。`EVENT.drive`・`EVENT.fire` が要求ごとに 1 つずつ控えた act とエラーの値から、`SHIORI.request` と `CALLBACK.sweep` の求めで表記付きの応答を作る（[止まるエラーの控えと応答](shiori.md#止まるエラーの控えと応答)） |

ACT・STORE・PROXY・SCENE・SAVE の内部構造は [Lua ランタイム内部モジュール](internal-modules.md) の [STORE パターン](internal-modules.md#store-パターン)・[ACT の内部](internal-modules.md#act-の内部)・[PROXY パターン](internal-modules.md#proxy-パターン)・[SCENE モジュール](internal-modules.md#scene-モジュール)・[SAVE モジュールの内部](internal-modules.md#save-モジュールの内部) で扱う。

### コルーチンの持ち主

実行中のシーンのコルーチンを持つ場所は次の 3 つだけである。

| 持ち主 | 持つもの | 寿命 |
| ------ | -------- | ---- |
| 再開する側のローカル変数（`EVENT.fire`・`CALLBACK.try_route`・`CALLBACK.sweep`） | ハンドラが返したコルーチン、または `CALLBACK.pending` から外した待機のコルーチン | `EVENT.drive` での再開が終わるまで |
| `STORE.co_scene` | 中断した（`suspended` の）継続待ちのコルーチン 1 つ、または `nil` | 仮想イベントの OnTalk で再開されるか、別のシーンの再開の結果で捨てられるまで（台詞を出した割り込み・指名した再生・エラー。[続きの決まり](#継続トークチェイントークと-co_scene-の更新)） |
| `CALLBACK.pending[イベント名]` | `get_property` のコールバックを待つコルーチン | 該当するコールバックが届くか、タイムアウトで掃引されるまで |

`STORE.co_callback` は、`CALLBACK.consume_staged` がコールバック待ちとして登録したコルーチンを、`EVENT.drive` の直後の `set_co_scene` に知らせるための印である。`set_co_scene` は、渡されたコルーチンがこの印と一致したときだけ `nil` に戻す。`act:get_property` は予約（`stage_pending`）の直後に必ず get タグを出力として中断するため、`consume_staged` が登録するコルーチンは suspended であり、続く `set_co_scene` が印を `nil` に戻す（予約の後、中断の前にエラーで終わった場合は `consume_staged` を呼ばず、`discard_staged` が予約を捨てる）。したがって `EVENT.drive` を抜けた時点で印は常に `nil` である。

## 処理とデータの流れ

### VM の構築とモジュール登録

本番の構築は、ローダ（`crates/pasta_lua/src/loader/mod.rs`）が設定読込・自己展開・トランスパイルを終えた後に `PastaLuaRuntime::from_loader_with_scene_dic` を呼んで行う。順序は次のとおりである。

```text
from_loader_with_scene_dic
 0. pasta.toml の [debug] と環境変数からデバッグ設定を解決し、RuntimeConfig に載せる
 1. with_config_and_source_map
    a. RuntimeConfig の libs を検証して警告（std_debug・std_all_unsafe・env）
       std_package を欠けば ConfigError::MissingRequiredLibrary で失敗（VM を作らない）
    b. libs を mlua の StdLib に変換し、Lua::unsafe_new_with で VM を作る
       math があれば、時刻とプロセス ID から作った種で math.randomseed を呼ぶ
    c. @pasta_search を登録（トランスパイル時の TranspileContext のレジストリと、RuntimeConfig の別名表から）
    d. libs に応じて @assertions・@testing・@env・@regex・@json・@yaml を登録
    e. @pasta_log を登録（libs に依らず常に）
    f. debug::enable を 1 回だけ呼ぶ（無効なら何もしない）
 2. ロガー・PastaConfig・ベースディレクトリを構造体に保持
 3. package.path を設定し、require の searcher を差し替え、@pasta_chunks（読み込んだファイルの表）を置く
 4. @pasta_config・@enc・@pasta_persistence・@pasta_log・@pasta_sakura_script・@pasta_debug を登録
 5. pasta.finalize_scene を Rust の実装で上書き（同じ別名表をクロージャに保持する）
 6. require("main")
 7. require("pasta.shiori.entry")
 8. require("pasta.scene_dic")   … 全シーンモジュールの require と finalize_scene()
 9. （ソースマップを持つときだけ）シーンの identity 索引を突き合わせてソースマップへ格納
```

- 1 の `with_config_and_source_map` は `PastaLuaRuntime::new`・`with_config` の実体でもあり、そこで作った VM は 2 以降の登録を持たない。`@pasta_log` は 1e と 4 の 2 回登録され、2 回目が置き換える。
- 1a の必須ライブラリの検査（`RuntimeConfig::ensure_libs`）は、VM を作るすべての入口がここを通るため 1 か所にある。必須は `std_package` だけである。1c・1e と 3 が `package` 表を無条件に使うためである。`std_string`・`std_table`・`std_math`・`std_os` は検査しない。
- 6〜8 は `require_startup_module` を通る。失敗は `module`・`fatal` 付きのエラーログと、`failed to load startup module '…'` の文脈を付けた `Err` になり、構築全体が失敗する。利用者から見た扱いは [起動シーケンス](../reference/startup.md#2-起動シーケンス) を参照する。
- 8 の `pasta.scene_dic` が `finalize_scene()` を呼ぶと、`@pasta_search` が Lua 側の登録から作り直される（[辞書確定](registry-search.md#辞書確定)）。
- 3 の `@pasta_chunks` は、searcher が読み込んだファイルを書き足す表で、`pasta.shiori.stop` が止まるエラーの表記で場所を示すのにだけ使う（[require の解決（searcher）](loader.md#require-の解決searcher)）。
- 4 の `@pasta_debug` は、0 で解決した設定の `enabled` の値と、1 で保持したソースマップと、ベースディレクトリ（`ghost/master/`）を持つ内部モジュールである（[位置でのシーンの再生](debug.md#位置でのシーンの再生)）。この入口だけが登録するため、`PastaLuaRuntime::new`・`with_config` の VM には無い。
- 1 の後にランタイムがソースマップを持つのは、0 で解決した設定でデバッグが有効なときである。1f の `debug::enable` が待ち受けのポートを取れずに `None` を返しても、ソースマップは持ち続けるため、9 も行われる。無効なときはローダがソースマップを作らず、9 は行われない（[有効化の判定](debug.md#有効化の判定)）。
- 4 の `@pasta_persistence` の登録はファイルを読まない。永続化ファイルを読むのは、後述のとおり `pasta.save` が最初に `require` されたときである。

構築が終わった VM に対して、SHIORI 層は `SHIORI.load`・`SHIORI.request`・`SHIORI.unload` を取り出して保持し、`SHIORI.request` をリクエストごとに 1 回呼ぶ（`crates/pasta_shiori/src/shiori.rs`）。リクエストの文字列は Rust 側で Lua の表に解析されてから渡る。

### イベントからシーンへ

`SHIORI.request(req)`（`crates/pasta_lua/pasta_scripts/pasta/shiori/entry.lua`）から `EVENT.fire(req)` までの経路と、`EVENT.fire` の振り分け（コールバック・`REG:register` の鎖・`REG` の値・シーン関数フォールバック）は [Lua 側の SHIORI エントリとイベント配送](shiori.md#lua-側の-shiori-エントリとイベント配送) で扱う。`EVENT.fire`（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua`）のうち、コルーチンを扱う部分の流れは次のとおりである。

```text
EVENT.fire(req)
 1. CALLBACK.try_route(req)          待っているコールバックなら再開して応答を返す（後述）
    CLOCK.observe(req)               時計の返事（OnPastaClock）を受け取った回は 204
 2. 鎖（REG:register）の写し getmetatable(REG).handlers(req.id) を新しい順に
      act = SHIORI_ACT.new(STORE.actors, req)        ハンドラごとに新しい ACT
      STOP.hold_handler(act)                          渡す act を控える（止まるエラーの表記のため）
      result = fn(act)                                真の値ならここで止める
 3. 全部見送れば（鎖が空でも）底を呼ぶ
      act = SHIORI_ACT.new(STORE.actors, req)
      STOP.hold_handler(act)
      result = (rawget(REG, req.id) or EVENT.no_entry)(act)    底の戻り値は真偽で見ない
 4. result の型で分岐（act は止めたハンドラか底のもの）
    thread  → ok, value = EVENT.drive(result, act, act)   後述の再開の手順
              ok が偽  → error(value)
              ok が真  → return RES.ok(value)        value が nil か "" なら 204
    string  → "SHIORI/" で始まる → return result     応答全体として包まずに返す
              それ以外           → return RES.ok(result)
    その他  → return RES.no_content()                STORE.co_scene は触らない（鎖の true・数・表もここ）
```

- 鎖の写しは呼ぶ前に 1 回だけ取る。ハンドラの中での登録・外しは、次のイベントから効く。見送ったハンドラの act は捨てるので、その台詞は応答に混ざらない。

- 文字列の分岐は、先頭 7 文字が `SHIORI/` と等しいかで判定する。`RES` の関数で作った応答（`RES.err` の 500 など）を返したハンドラは、そのステータスのまま応答になる。
- コルーチンの出力にはこの規則を適用しない。出力は接頭辞にかかわらず常に `RES.ok` の `Value` になる（`CALLBACK.try_route`・`CALLBACK.sweep` の出力も同じ）。

ハンドラがコルーチンを返す経路は、シーン関数フォールバック（`EVENT.no_entry`）、既定の OnBoot・OnChoiceSelectEx・OnMouseDoubleClick・OnMouseMove・OnPastaScene・OnPastaSceneAt・OnPastaCallFunction、OnSecondChange の仮想イベント（OnTalk・OnHour・シーンキック）と、コルーチンを返す利用者のハンドラ（`REG:register` で重ねた鎖のハンドラと `REG` の値）である。振り分けの詳細は [SHIORI 層](shiori.md) で、利用者から見た挙動は [SHIORI イベントとハンドラ](../lua/shiori-events.md#シーン関数フォールバック) で扱う。

### シーンコルーチンの作り方

シーン関数をコルーチンにする箇所は、どれも同じ包み方をする。

```lua
local function wrapped_fn(resumed_act, ...)
    fn(resumed_act, ...)               -- シーン関数本体
    local result = resumed_act:build() -- 残りのトークを組み立てる
    if result ~= nil then
        return result                  -- コルーチンの最後の値になる
    end
end
return coroutine.create(wrapped_fn)
```

| 箇所 | シーン関数の得方 |
| ---- | ---------------- |
| `SCENE.co_exec`（`crates/pasta_lua/pasta_scripts/pasta/scene.lua`） | `act:find_scene(名前)`。`EVENT.no_entry`・既定の OnBoot・仮想イベント・明示的な OnChoiceSelectEx シーン・既定の OnMouseDoubleClick と OnMouseMove（ショートカットが見つからないとき）が使う |
| `SCENE.co_exec_exact`（`crates/pasta_lua/pasta_scripts/pasta/scene.lua`） | `search_scene_exact(名前, nil)` か `search_local_scene_exact(名前, ローカル名)` の結果で `SCENE.get`。完全一致のシーンだけ。既定の OnMouseDoubleClick・OnMouseMove がショートカットの 1〜8 段を探すのに、既定の OnPastaScene が名前で指名したシーンを探すのに使う（[co_exec_exact](internal-modules.md#co_exec_exact)） |
| `SCENE.co_exec_registered`（`crates/pasta_lua/pasta_scripts/pasta/scene.lua`） | 登録名の組で `SCENE.get`。`co_exec_exact` が使うほか、既定の OnPastaSceneAt が位置から引いた組で使う（[co_exec_registered](internal-modules.md#co_exec_registered)） |
| `create_scene_coroutine`（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/choice_select.lua`） | `SCENE.search(選択 ID, 親)`、無ければ `SCENE.search(選択 ID, nil)`。親は Reference2 のグローバルシーン名（既知のグローバルシーンのとき）か `STORE.last_global_scene`（[既定ハンドラ](shiori.md#lua-側の-shiori-エントリとイベント配送)） |
| `wrap_local_func`・`wrap_reload_func`（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/kick.lua`） | シーン表から完全一致で引いたキック対象のシーン関数（`SCENE.get`・`SCENE.get_start`）、または SHIORI 再読み込みのタグだけを出す関数 |

- コルーチンは作るだけで、作った時点ではシーン関数は動かない。最初の `resume` で `EVENT.fire` が（`EVENT.drive` を通して）そのイベントの ACT を渡し、それが `resumed_act` になる。
- シーン関数の戻り値は捨てられる。応答になるのは、途中の `yield` で渡した値か、最後の `build()` の値である。
- Call（`act:call`・`act:call_restore`）で呼んだ先のシーン関数は、同じコルーチンの中の通常の関数呼び出しとして動く。呼び先での中断はコルーチン全体の中断になり、再開すると呼び先の続きから進む。途中の Call（`act:call_restore`）は、再開後に呼び先から戻った時点で `current_scene` を呼ぶ前の値に戻す（[シーン文脈の復元](internal-modules.md#シーン文脈の復元call_restorerestore_scene)）。

### 再開の手順（EVENT.drive）

シーンコルーチンの再開は、呼び出し元が `EVENT.fire`・`CALLBACK.try_route`・`CALLBACK.sweep` のどれでも、`EVENT.drive`（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua`）の 1 つの手順を通る。

```text
EVENT.drive(co, act, ...)
 1. kind = drive_kind(co)                      再開の前に割り込みの種類を決める（後述）
 2. target = FAILURE.act_of(co) or act         target._spoke = nil・target._interrupted = nil
                                               kind が "interrupt" なら target._on_speech = run_interrupt_scene
 3. ok, value = resume_until_valid(co, ...)   ... は最初の resume にだけ渡す
 4. scene_act = FAILURE.act_of(co) or act      target と scene_act の _on_speech を nil に戻す（成否によらず）
 5. ok が偽 → STOP.hold(co, value)             止まったコルーチンの act とエラーの値を控える
             CALLBACK.discard_staged()        中断の前にエラーで終わったシーンの予約を捨てる
             set_co_scene(co, false)          co は dead。STORE.co_scene も空になる
             return false, value
 6. CALLBACK.consume_staged(co, act)           予約があれば待機として登録（後述）
 7. spoke = scene_act._spoke                   印が nil（組み立てていない）なら value が nil でも "" でもないか
 8. scene_act._interrupted が真 → set_co_scene(nil, false)
    それ以外                    → set_co_scene(co, kind == "interrupt" and not spoke)
 9. return true, value
```

- 応答の文字列は作らない。エラーを投げるか、続けるかも決めない。どちらも呼び出し元が決める。5 の `STOP.hold` は控えるだけで、控えから表記付きの応答を作るのは `SHIORI.request` と `CALLBACK.sweep` である（[止まるエラーの控えと応答](shiori.md#止まるエラーの控えと応答)）。
- 1 は再開の前に行う。再開の中で `STORE.co_scene` が変わる前の状態で種類を決めるためである。
- 2 の `target` は、新しく始まるシーンではまだ `FAILURE.act_of(co)` が無いので、最初の `resume` に渡す ACT になる（`EVENT.fire` は `EVENT.drive(result, act, act)` と呼ぶので、シーンが積む ACT と同じ）。4 の `scene_act` は、再開の中で `init_scene` が記録した ACT である。印の意味は [台詞の印と中断のシーン](#台詞の印と中断のシーン) で扱う。
- 4 で欄の関数を外すのは、失敗の後に `STOP.response` が `act:build()` を呼んでも、中断のシーンを動かさないためである。
- 6 から 8 の間には、台詞の印を読むこと（7）のほかに処理を挟まない。予約の消費と継続の更新は必ず組で行われる。6 を先に行うのは、`set_co_scene` が `STORE.co_callback` で待ちに入ったかを見るためである。
- 7 で ACT の印が無いのは、ACT を通らない素のコルーチン（テストやゴーストが Lua で作ったもの）である。出力が空でなければ台詞に数える。
- `act` は、`get_property` で中断したときに待機へ登録する ACT である。`EVENT.fire` ではそのイベントの ACT、`try_route`・`sweep` では待機を登録したときの ACT（`entry.act`）を渡す。
- `resume_until_valid`・`drive_kind`・`set_co_scene`・`run_interrupt_scene` は `init.lua` のローカル関数である。`callback.lua` は `EVENT` を呼び出し時に `require` して `EVENT.drive` を使う（読み込み時に `require` すると循環する）。

### 再開のループ（resume_until_valid）

`resume_until_valid(co, ...)` は、有効な値が得られるかコルーチンが終わるまで `coroutine.resume` を繰り返す。

```text
ok, value = coroutine.resume(co, ...)      初回だけ引数を渡す（引数は呼び出し元ごとに異なる。後述の表）
loop
  ok が偽                       → (false, エラー) を返す
  value が nil でない           → (true, value) を返す
  value が nil で co が dead    → (true, nil) を返す（出力の無いシーン）
  value が nil で co が suspended → ok, value = coroutine.resume(co)（引数なし）
```

- ACT の `build()` はトークが 0 件なら `nil` を返す。そのため、トークを積まずに `act:yield()` した中断（`yield` の直後の `yield` など）は、応答を返さず同じイベントの中で再開される。
- 2 回目以降の `resume` は引数を渡さない。継続したシーンの再開でも、`EVENT.fire` が渡す新しい ACT は `coroutine.yield` の戻り値になるだけで、`ACT_IMPL.yield` はそれを使わない。継続したシーンは、最初の `resume` で受け取った ACT（`act.req` はそのときのイベントのもの）を使い続ける。
- このループは `EVENT.drive` の中で使われる。`EVENT.drive` を呼ぶのは次の 3 箇所であり、最初の `resume` に渡す引数だけが異なる。

| 呼び出し元 | 最初の `resume` の引数 | シーン側で受け取る値 |
| ---------- | ---------------------- | -------------------- |
| `EVENT.fire`（ハンドラが返したコルーチン） | そのイベントの ACT | 新しく始まったシーンではシーン関数の第 1 引数（`resumed_act`）。継続したシーンでは `coroutine.yield` の戻り値になり、使われない |
| `CALLBACK.try_route` | `refs`（Reference を 1 始まりにした配列） | `get_property` の中の `coroutine.yield` の戻り値 |
| `CALLBACK.sweep` | `nil, reason`（`reason` は `on_timeout` が文字列のときだけ。それ以外は `nil`） | 同上。`get_property` は `reason` があれば `error(reason)` する |

### 継続トーク（チェイントーク）と co_scene の更新

`act:yield()`（`ACT_IMPL.yield`）は、それまでのトークを `build()` して `coroutine.yield` に渡す。DSL の `＞yield` は Call の検索で ACT のメソッド `yield` に、`＞チェイントーク` は `GLOBAL["チェイントーク"]`（`GLOBAL.yield` と同じ関数）に解決され、どちらも `act:yield()` になる（[ローカル優先の検索順](registry-search.md#ローカル優先の検索順)）。自動トークを止めている間（`STORE.auto_talk_paused`）に呼ばれたときは、先にログへ警告を 1 行出して `GLOBAL.resume_auto_talk` を呼び、その後は同じく区切る。

`EVENT.drive` は、再開の前に `drive_kind(co)` で割り込みの種類を決め、再開の後に台詞の印を読んで `set_co_scene(co, keep)` で `STORE.co_scene` を更新する。呼び出し元が `EVENT.fire`・`CALLBACK.try_route`・`CALLBACK.sweep` のどれでも、イベントの種類が何でも同じ規則である。利用者から見た決まりは [続きが捨てられるとき](../grammar/call-jump.md#続きが捨てられるとき) と [チェイントーク中断](../grammar/call-jump.md#チェイントーク中断) が正であり、ここではそれを実現する仕組みだけを書く。

```text
drive_kind(co)                       判定の順はこのとおり
  STORE.co_scene が nil         → "none"       待っている続きが無い
  co == STORE.co_scene          → "resume"     待っている続きそのものの再開
  STORE.co_preempt[co] が真     → "preempt"    開発者が指名した再生（キック・シーン再生）
  それ以外                      → "interrupt"  続きを待っている間の割り込み

set_co_scene(co, keep)
 1. co が suspended でなければ co = nil として扱う（dead のコルーチンは残さない）
 2. keep が真なら STORE.co_scene には触れずに終わる。co が STORE.co_callback と同じなら
      印だけ nil に戻す（co は CALLBACK.pending が持つ）。それ以外の co の残りは
      どこにも置かないので捨てられる
 3. co が STORE.co_callback と同じなら（コールバック待ちとして登録済み）
      STORE.co_scene が別のコルーチンなら破棄し、STORE.co_scene = nil、
      STORE.co_callback = nil として終わる
 4. STORE.co_scene と co が同じなら何もしない（継続中のコルーチンがまだ続く）
 5. STORE.co_scene があれば破棄する
 6. STORE.co_scene = co
```

`STORE.co_scene` をランタイムの中で書くのは `set_co_scene` だけである（`STORE.reset` も `nil` に戻す）。`STORE.co_preempt` は、キーを弱い参照にした表（`table<thread, true>`）で、指名した再生のコルーチンに付ける「はっきり捨てる」の印である。付けるのは `kick.lua` の `wrap_local_func`・`wrap_reload_func` が作るコルーチンと、`scene_play.lua` の `OnPastaScene`・`OnPastaSceneAt` が返すコルーチンだけで、読むのは `drive_kind` だけである。印はコルーチンごとに 1 つで消さないため、キックのシーンがコールバック待ちから戻った回も "preempt" になる。キックのシーンが区切って続きになった後は、`co == STORE.co_scene` が先に当たるので "resume" になる。

`EVENT.drive` の 8 は、種類と再開の結果から次のように `STORE.co_scene` を決める。「台詞」は 7 の `spoke`、「中断」は `run_interrupt_scene`（[台詞の印と中断のシーン](#台詞の印と中断のシーン)）である。

| 種類 | 再開の結果 | 呼び出し | `STORE.co_scene` | 中断 |
| ---- | ---------- | -------- | ---------------- | ---- |
| `resume` | エラーでない | `set_co_scene(co, false)` | `co` の結果で置き換える（区切れば `co`、終われば `nil`） | 動かさない |
| `none` | エラーでない | `set_co_scene(co, false)` | 区切れば `co` の残りが続きになる | 動かさない |
| `preempt` | エラーでない | `set_co_scene(co, false)` | 待っている続きを捨てる。区切れば `co` の残りが続きになる | 動かさない |
| `interrupt` | 台詞あり・中断の台詞を出した（`_interrupted`） | `set_co_scene(nil, false)` | `nil`。`co` が区切っていても残りは捨てる | 反応の代わりに出す |
| `interrupt` | 台詞あり・中断が `nil` を返した | `set_co_scene(co, false)` | 待っている続きを捨てる。区切れば `co` の残りが続きになる | 動かしたが出さない（反応をそのまま出す） |
| `interrupt` | 台詞なし・最後まで動いた | `set_co_scene(co, true)` | 待っている続きを残す | 動かさない |
| `interrupt` | 台詞なし・区切った | `set_co_scene(co, true)` | 待っている続きを残す。`co` の残りは捨てる | 動かさない |
| `interrupt` | 台詞なし・コールバック待ちに入った | `set_co_scene(co, true)` | 待っている続きを残す。`co` は `CALLBACK.pending` が持つ | 動かさない |
| どれでも | エラー | `set_co_scene(co, false)` | `co` は dead なので `nil`。待っている続きも捨てる | 動かさない（エラーの前に動いていても応答に載らない） |

この規則から次のことが成り立つ。

- 中断したコルーチンは `STORE.co_scene` に 1 つだけ残る。待っている続きを捨てるのは、待っている続きそのものの再開・指名した再生・台詞を出した割り込み・エラーのどれかである。台詞を出さない割り込み（何も積まないハンドラのシーン、表情の単語・さくらスクリプト・`get_property` の get タグだけを出すシーン）は、待っている続きを残す。捨てた続きは参照を外すだけで、再開されない。
- 中断のシーンを動かすのは、種類が "interrupt" で、組み立てが台詞を見つけたときだけである。続きが無い回・続きの再開・指名した再生・台詞を出さない回では、中断のシーンを探しもしない。
- 継続を再開するのは仮想イベントの OnTalk だけである。`check_talk`（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua`）は、トークの時刻に達したとき `STORE.co_scene` があれば新しいシーンを探さずにそれを返す。外から起こした `OnTalk`（`EVENT.no_entry` が `SCENE.co_exec` で探したシーン）は、ほかのイベントと同じく "interrupt" である。OnHour は OnTalk より先に判定されるが、`check_hour` が `SCENE.co_exec` で探したコルーチンに印は無く、ふつうの割り込みとして扱われる（時報のシーンが台詞を出せば続きを捨てる）。シーンキックは印の付いたコルーチンを返すので "preempt" になる。自動トークを止めている間（`STORE.auto_talk_paused`）は `check_talk` まで進まないため、継続は再開されず、捨てられもしない。
- 文字列か `nil` を返すハンドラ（`REG` の関数など）は `STORE.co_scene` を変えない。
- シーンの実行中にエラーが起きると、そのコルーチンは dead になり、`set_co_scene(co, false)` で既存の継続も破棄される。中断のシーンのエラーも、割り込んだシーンのコルーチンの中で投げ直されるので、同じ枝を通る。エラーは、`EVENT.fire` と `CALLBACK.try_route` の経路では `SHIORI.request` の `xpcall` へ伝わり、`CALLBACK.sweep` では応答の候補になる（後述）。

### 台詞の印と中断のシーン

台詞の有無と中断の流し方は、ACT の欄で `EVENT.drive` と組み立て（`SHIORI_ACT_IMPL.build`、`crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua`）の間を受け渡す。ACT のメソッドは足さない。

| 欄 | 型 | 書く所 | 意味 |
| -- | -- | ------ | ---- |
| `_spoke` | `boolean\|nil` | `EVENT.drive` が再開の前に `nil`、`build` が `self._spoke or 台詞あり` | この再開の間に組み立てた出力に台詞があったか。`nil` は組み立てていない |
| `_on_speech` | `(fun(act): table[]\|nil)\|nil` | `EVENT.drive` が "interrupt" のときに `run_interrupt_scene` を掛け、使う前の `build` と再開の後の `EVENT.drive` が外す | 台詞を見つけたときに呼ぶ、中断の流し方 |
| `_interrupted` | `boolean\|nil` | `EVENT.drive` が再開の前に `nil`、`build` が差し替えたときに `true` | この再開で、中断の台詞を反応の代わりに出したか |

```text
SHIORI_ACT_IMPL.build(self)
 1. token = ACT.IMPL.build(self)                 まとめたトークン列（0 件なら nil）
 2. spoke = token ~= nil and BUILDER.speaks(token)
    self._spoke = self._spoke or spoke           同じ再開の中では一度立てたら落とさない
 3. token が nil → return nil
 4. spoke で self._on_speech があれば
      self._on_speech = nil                      呼ぶ前に外す（中断の組み立てから入れ子に呼ばせない）
      head = on_speech(self)
      head があれば token = head、self._interrupted = true
 5. BUILDER.build(token, …) で 1 回だけ文字列にする
```

- `BUILDER.speaks(groups)`（`sakura_builder.lua`）は、アクターのグループの中に、字が出る `talk`（`emits_text`。組み立ての段落の区切りと同じ判定）か `choice` があるかを返す。表情の単語（字が `\s[5]` だけの `talk`）・空の `talk`・`sakura_script`・`raw_script`（失敗の表記を含む）・`surface`・`wait`・`newline`・`clear`・`choice_timeout`・`spot`・`clear_spot` は数えない。`get_property` が出す get タグだけの回は台詞にならず、待つ前によけた台詞はまだ出していないので数えない。
- 4 で差し替えた割り込んだシーンのトークンは文字列にしないため、その表情・立ち位置は `STORE.appearance`・`STORE.actor_spots` に残らない。応答は中断の台詞だけの 1 本のスクリプトになる。
- 素のコルーチンでも、渡された ACT に積んで `act:build()` するものは、組み立てが印を立てるので、ふつうのシーンと同じに数える。出力を自分で文字列にして返すコルーチンは印が立たず、`EVENT.drive` の 7 が出力の有無で決める。この場合、組み立てを通らないので中断の台詞は付かない。

`run_interrupt_scene(act)` は、割り込んだシーンの組み立ての中（割り込んだシーンのコルーチンの中）で呼ばれ、`＊チェイントーク中断` を入れ子のコルーチンで 1 回だけ動かして、まとめたトークン列を返す。

```text
run_interrupt_scene(act)
 1. ico = SCENE.co_exec_exact("チェイントーク中断")       名前の完全一致。同名が複数ならふつうの選び方で 1 つ
    無ければ return nil
 2. iact = SHIORI_ACT.new(STORE.actors, act.req)         割り込んだシーンの要求を持つ新しい ACT
    iact.build = ACT.IMPL.build                          文字列にせず、まとめたトークン列で受け取る
 3. ok, groups = resume_until_valid(ico, iact)           出力の無い区切りを飛ばし、最初に出力のある区切りまで
 4. waited = CALLBACK.has_staged()
    CALLBACK.discard_staged()                            中断の中の予約で割り込んだシーンを待ちに登録しない
 5. ok が偽 → iact.build = nil                           止まるエラーの表記をふつうの組み立てで作るため
             FAILURE.bind(iact)                          止まった場所を中断のシーンとして控えさせる
             error(groups, 0)                            割り込んだシーンのコルーチンの中で投げ直す
 6. waited → return nil                                  get タグも出さない
 7. return groups                                        nil なら中断の台詞は無い
```

- 名前は固定の文字列 `"チェイントーク中断"` で、設定や別名表の特別扱いは無い。ふつうのシーンとして DSL で書け、`pasta_check` も特別な名前として扱わない。
- 中断のシーンのコルーチンは `EVENT.drive` を通らないため、`STORE.co_scene` にも `CALLBACK.pending` にも入らず、割り込みの判断にも関わらない。区切っても残りは参照を外すだけで捨てる。
- 5 のエラーは `EVENT.drive` の失敗の枝 → `SHIORI.request` の `xpcall` → `STOP.response` の道を通る。表記の止まった場所は `＊チェイントーク中断` になり、中身は中断のシーンが止まるまでに積んだ出力と表記で、割り込んだシーンのその回の出力は載らない。待っている続きは失敗の枝で捨てる。
- `STORE.last_global_scene` は中断のシーンの名前になる。選択肢の検索の親は Reference2 が持つので、戻さない。

### コールバック待ちとの関係

`act:get_property` は、`CALLBACK.stage_pending` で待つイベント名（`OnPastaCallBack{N}`）を予約し、`\![get,property,…]` のタグだけを `build()` して `coroutine.yield` する。

```text
シーン内 act:get_property(…)
  CALLBACK.stage_pending(イベント名, 期限, 理由)
  coroutine.yield(get タグだけのスクリプト)
        ↓ resume_until_valid が値を受け取る
EVENT.drive（どの呼び出し元から再開していても同じ）
  CALLBACK.consume_staged(co, act)   予約があれば pending[イベント名] = {co, act, …}
                                     STORE.co_callback = co
  set_co_scene(co, keep)             co_callback と一致 → co_scene には入れず、印を nil に戻す
                                     keep が偽なら co_scene も nil にする（keep が真なら触れない）
        ↓ 後のリクエスト
EVENT.fire → CALLBACK.try_route(req)
  pending[req.id] があれば外して EVENT.drive(co, entry.act, Reference の配列)
  ok が偽 → error(value)
  ok が真 → return RES.ok(value)     value が nil か "" なら 204
```

- コールバック待ちのコルーチンは `STORE.co_scene` に入らない。待っている間も、別のシーンの継続とは独立に保たれる。
- 待ちに入った回の `STORE.co_scene` は、[続きの決まり](#継続トークチェイントークと-co_scene-の更新)の表のとおりに決まる。get タグは台詞に数えないので、台詞を出さずに待ちに入った割り込みは待っている続きを残す（`keep` が真）。待っている続きそのものが待ちに入ったとき（"resume"）は `STORE.co_scene` を空にし、待っている間に次の自然なおしゃべりが同じコルーチンを再開しないようにする。続きが無い回・指名した再生の回も `keep` は偽なので空になる。
- コールバックで再開したシーンは、戻った回の `EVENT.drive` で、通常のイベントと同じ規則で扱われる。待ちに入った回に続きを残していれば、戻った回の種類は "interrupt" になり、台詞を出せば続きを捨てて中断の台詞を流す。台詞を出さなければ続きは残る。指名した再生のコルーチンは印が残っているので "preempt" になる。出力の無い中断は同じリクエストの中で進み、チェイントークで中断すれば表のとおりに `STORE.co_scene` が決まり（続きになれば、次の自然なおしゃべり（仮想ディスパッチャが起こす OnTalk）の機会に続きが出る）、続けて `get_property` すれば新しい待機として登録される。
- 再開したシーンのエラーは、`pending` にも `STORE.co_scene` にも残らず、`SHIORI.request` が応答にする（表記を出す設定で ID が `On` で始まる要求なら表記付きの `200 OK`、それ以外は 500。[止まるエラーの控えと応答](shiori.md#止まるエラーの控えと応答)）。
- `req.id` と一致する待機が無ければ `try_route` は `nil` を返し、`EVENT.fire` は通常の振り分けに進む。掃引で外した待機に遅れて届いた結果も、この扱いになる。

### 期限切れの掃引（CALLBACK.sweep）

期限を過ぎた待機は、OnSecondChange の既定ハンドラ（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/second_change.lua`）が毎回呼ぶ `CALLBACK.sweep(CLOCK.steady(act.req) or os.time())`（飛びを差し引いた時計。[仮想時刻の時計](shiori.md#仮想時刻の時計pastashioriclock)）が、タイムアウトとして再開する。

```text
CALLBACK.sweep(now)
 1. 集める  pending を走査し、now > timeout_at の待機を一覧にする（この間は再開しない）
 2. 外す    一覧の待機をすべて pending から外す
 3. 並べる  イベント名の番号（OnPastaCallBack{N} の N）の小さい順。番号の無い名前は末尾に名前の文字列順
 4. 再開    一覧の順に、待機ごとに
              on_timeout が文字列なら reason = on_timeout とし、イベント名と理由を警告ログに出す
              ok, value = EVENT.drive(entry.co, entry.act, nil, reason)
              結果から応答の候補を決める（下表）
              まだ応答が無ければ候補を採用し、既にあれば候補を捨てて警告ログを出す
 5. 採用した応答、または nil を返す
```

| `EVENT.drive` の結果 | `on_timeout` | 応答の候補 |
| -------------------- | ------------ | ---------- |
| `ok` が偽 | 文字列 | `STOP.response` が作る時間切れの表記付きの応答（`200 OK`）。表記を出さない設定なら `RES.err(on_timeout)`（500）。理由はどちらもシーン内のエラー文字列ではなく `on_timeout` そのもの |
| `ok` が偽 | 文字列以外 | イベント名を前に付けた警告ログを出し、`STOP.response` が作るスクリプトのエラーの表記付きの応答（`200 OK`）。表記を出さない設定ならなし |
| `ok` が真で出力あり | どちらでも | `RES.ok(出力)` |
| `ok` が真で出力なし（`nil` か `""`） | どちらでも | なし |

- 収集と取り外しを再開より前に済ませるため、再開中に登録された待機（タイムアウトを捕まえて続けて `get_property` したシーンなど）は同じ回には扱われない。走査中の表への挿入も起きない。
- 待機は再開の前に `pending` から外すため、1 つの待機が 2 回処理されることはない。途中の待機がエラーで終わっても、残りの待機の処理を続ける。候補を捨てた待機も `EVENT.drive` を通っているため、予約と継続は規則どおりに更新されている。
- 既定ハンドラは、`sweep` が応答を返せばそれを返し、仮想イベント（OnTalk・OnHour）を出さない。応答は `SHIORI/` で始まる全文なので、`EVENT.fire` が包まずにその回の応答にする。`sweep` が `nil` を返した回だけ `virtual_dispatcher.dispatch` に進む。

利用者から見た挙動は [OnPastaCallBack](../lua/shiori-events.md#onpastacallbackコールバック応答) を参照する。

### 永続化

永続化の対象は `pasta.save` が返す 1 つの Lua の表（`save` テーブル）である。`@pasta_persistence` の関数の振る舞い（読み込みに失敗したときの戻り値、`save()` の戻り値と失敗する条件、保存の形式と書き込み方）と `[persistence]` の設定は [@pasta_persistence](../lua/modules/pasta-persistence.md) が正であり、ここでは実装の所在と、ランタイムのどの時点で読み書きが起きるかだけを書く。

```text
VM の構築      register_persistence_module → persistence::register
                 PersistenceConfig::effective_file_path の相対パスを検証して保存先を決め、
                 保存先・obfuscate・debug_mode を PersistenceState にまとめて
                 load / save のクロージャに持たせ、@pasta_persistence として登録する
初回の require  pasta.save → @pasta_persistence.load()（load_impl → load_from_file）
                 → 返った表が package.loaded["pasta.save"] に入り、以後は同じ表を返す
実行中          ACT.new が act.save に同じ表を入れる。生成コードの save.名前 = 式 や
                 act:init_scene の戻り値で読み書きする
任意の時点      @pasta_persistence.save(表)（save_impl → save_to_file）
ランタイム破棄  Drop → save_persistence_data → save_to_file
```

- `persistence::register` は、保存先が絶対パスであるか、`..`・ルート・ドライブの接頭辞を含むと `RuntimeError` を返す。これは `from_loader_with_scene_dic` の `?` で構築全体の失敗になる。
- `load_from_file` はファイルを読み、空なら空の JSON オブジェクトにする。先頭 2 バイトが gzip のマジック（`1f 8b`）なら `GzDecoder` で展開してから、そうでなければそのまま `serde_json` で解析する。`load_impl` はその結果を Lua の値に変換し、読み込みの失敗と表にならない値を警告ログ付きの空の表に置き換える。
- `save_impl` と `save_persistence_data` は、表を `serde_json::Value` に変換して `save_to_file` に渡す。Lua の表と JSON の変換は mlua の serde 連携（`LuaSerdeExt`）で行う。`save_to_file` は親ディレクトリを作り、`obfuscate` に応じて `GzEncoder` で圧縮するか整形した JSON にして、`Path::with_extension("tmp")` のファイルに書いて `sync_all` し、`fs::rename` で保存先に置き換える。`rename` が失敗したら一時ファイルを消してエラーを返す。
- `Drop` での保存（`crates/pasta_lua/src/runtime/lifecycle.rs`）は、`PastaLuaRuntime` が保持する `PastaConfig` とベースディレクトリから保存先を計算し直し、`require("pasta.save")` を評価した表を書き出す。`require` が失敗した場合（`@pasta_persistence` の登録前など）は保存を省く。変換・書き込みの失敗はエラーログに出し、`Drop` からは伝えない。
- `Drop` での保存は、`PastaLuaRuntime` が破棄されるすべての場合に走る。SHIORI 層では、ゴーストの終了（`SHIORI.unload` を呼んだ後のランタイムの破棄）と、`load` のやり直しで前のランタイムを捨てるときがこれにあたる。起動が途中で失敗して構築中のランタイムが破棄されるときも同じ処理が走る。`unload` を呼ばれずにホストのプロセスが終わったときはランタイムが破棄されないため、保存されない（[SHIORI 層](shiori.md#unloaddllmain-と-teardown)）。
- `Drop` は `pasta.save` を `require` して保存するため、セッション中に一度も読み込まれていなければ、破棄の時点で読み込んでそのまま書き戻す。
- VM を破棄すると、コルーチン・STORE・ACT はすべて失われる。セッションをまたいで残るのは保存した `save` テーブルだけである。

保存先の計算は `PersistenceConfig::effective_file_path`（`crates/pasta_lua/src/loader/config/sections.rs`）にある。

## 境界の受け渡し

| 境界 | 渡す側 → 受ける側 | 渡すもの | 所有 |
| ---- | ----------------- | -------- | ---- |
| Rust → Lua（構築） | `PastaLuaRuntime` → VM | `@pasta_*` などのモジュールの表・ユーザーデータ、`finalize_scene` の関数 | Rust が `Lua` を所有し、モジュールは `package.loaded` に置かれて VM が所有する。`@pasta_persistence` の関数は保存先などの状態（`PersistenceState`）をクロージャに持つ |
| Rust → Lua（起動） | `require_startup_module` → VM | モジュール名（`main`・`pasta.shiori.entry`・`pasta.scene_dic`） | 実行結果の Lua の状態（シーン・単語の登録表、`SHIORI` 表）は VM が所有する |
| `pasta_shiori` → Lua（リクエスト） | `PastaShiori` → `SHIORI.request` | 解析済みのリクエストの表 | `PastaShiori` が `PastaLuaRuntime` と取り出した Lua 関数を所有する。表は呼び出しの間だけ使われ、ACT の `req` として残る |
| Lua → `pasta_shiori`（応答） | `SHIORI.request` → `PastaShiori` | SHIORI 応答の文字列 | 文字列の複製が Rust に渡る |
| Lua 内（シーンの状態） | `EVENT.drive` → `STORE.co_scene`・`CALLBACK.pending` | コルーチンと ACT | Lua だけが所有する。Rust はコルーチンを持たず、再開もしない |
| Lua ↔ Rust（永続化） | `pasta.save` ↔ `@pasta_persistence`、`Drop` ↔ `pasta.save` | `save` テーブル ↔ `serde_json::Value` | 表は Lua が所有する。Rust は変換して書き出すだけで、表を保持しない |

`PastaLuaRuntime` を所有して呼び出すスレッドと、リクエストが届くまでの経路は [SHIORI 層](shiori.md) で扱う。

## 不変条件と制約

- VM は `PastaLuaRuntime` ごとに 1 つであり、1 つのスレッドだけが触る。`Lua::unsafe_new_with` の使用は、ライブラリの指定を検証済みの `RuntimeConfig` から作ることと、単一スレッドで使うことを前提にしている。
- コルーチンを再開するのは `resume_until_valid` だけであり、それを呼ぶのは `EVENT.drive` と、`EVENT.drive` の再開の中で中断のシーンを動かす `run_interrupt_scene` だけである。`EVENT.drive` を呼ぶのは `EVENT.fire`・`CALLBACK.try_route`・`CALLBACK.sweep` だけである。Rust 側はコルーチンを再開しない。
- `STORE.co_scene` は、中断中のコルーチン 1 つか `nil` だけを持ち、ランタイムの中で書くのは `set_co_scene` だけである（`STORE.reset` も `nil` に戻す）。コールバック待ちのコルーチンと中断のシーンのコルーチンは `STORE.co_scene` に入らない。
- 出力の無い中断（`nil` の `yield`）は、どの呼び出し元から再開した場合も同じリクエストの中で飛ばされる。
- 継続したシーンは、最初の `resume` で受け取った ACT を使い続ける。イベントごとに作られる ACT は、新しく始まったシーンにだけ渡る。
- 永続化されるのは `package.loaded["pasta.save"]` の表である。`act.save` に別の表を代入しても、その表は `Drop` では保存されない。

### LuaJIT 2.1 の制約

ランタイムの Lua は、Lua 5.2 互換を有効にした LuaJIT 2.1（mlua の `luajit52`。[Cargo.toml](https://github.com/ekicyou/pasta/blob/main/Cargo.toml) のワークスペース依存）である。実行モデルに関わる制約は次のとおりである。

- `coroutine.close` が無い。`set_co_scene` と `STORE.reset` は `if coroutine.close then` で分岐しているため、LuaJIT では閉じる処理は実行されない。中断中のコルーチンを「破棄する」とは、参照を外して GC に回収させることである。破棄したコルーチンは、`yield` より後のコードを二度と実行しない（後始末のコードも動かない）。
- to-be-closed 変数（`<close>`）が無い。`__close` メタメソッドは自動では呼ばれない。
- コルーチンの中断は Lua の関数の中でだけ行える。Rust の関数（`@pasta_*` の関数など）から呼ばれた Lua の関数の内側で `yield` すると、C 呼び出しの境界をまたぐためエラーになる。
- `_VERSION` は `"Lua 5.1"` を返す。ランタイムの種類と版は `pasta.lua_version` が `jit` の有無で判定する。
- Lua 5.2 互換により `table.unpack` などが使える。生成コードと `pasta.shiori.act` はこれを使う。

## ソースの所在

- `crates/pasta_lua/src/runtime/`
- `crates/pasta_lua/pasta_scripts/pasta/`

本文で参照した主なファイルは次のとおりである。VM の構築は `crates/pasta_lua/src/runtime/mod.rs`・`crates/pasta_lua/src/runtime/factory.rs`・`crates/pasta_lua/src/runtime/module_registry.rs`、永続化は `crates/pasta_lua/src/runtime/persistence.rs`・`crates/pasta_lua/src/runtime/lifecycle.rs`・`crates/pasta_lua/pasta_scripts/pasta/save.lua`、イベントとコルーチンは `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua`・`crates/pasta_lua/pasta_scripts/pasta/shiori/event/callback.lua`・`crates/pasta_lua/pasta_scripts/pasta/scene.lua`・`crates/pasta_lua/pasta_scripts/pasta/act.lua`・`crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua`、台詞の判定は `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`、はっきり捨てる印は `crates/pasta_lua/pasta_scripts/pasta/store.lua`・`crates/pasta_lua/pasta_scripts/pasta/shiori/event/kick.lua`・`crates/pasta_lua/pasta_scripts/pasta/shiori/event/scene_play.lua`、失敗の報告は `crates/pasta_lua/pasta_scripts/pasta/failure.lua` にある。`SHIORI.request` を呼ぶ側は `crates/pasta_shiori/src/shiori.rs`、構築を呼ぶローダは `crates/pasta_lua/src/loader/mod.rs` である。

## 経緯

- [scene-coroutine-execution](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/scene-coroutine-execution) — シーン関数のコルーチン実行と `STORE.co_scene` による継続
- [coroutine-resume-loop](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/coroutine-resume-loop) — `nil` の `yield` を飛ばす再開のループ
- [act-build-early-return](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/act-build-early-return) — トークが 0 件の `build()` が `nil` を返す
- [yield-continuation-token](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/yield-continuation-token) — `＞yield`・`＞チェイントーク` による継続トーク
- [shiori-async-talk](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/shiori-async-talk) — コールバック待ちのコルーチン（`get_property`）
- [store-save-table](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/store-save-table) — 永続変数の表の導入
- [store-save-persistence](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/store-save-persistence) — `pasta.save` と `@pasta_persistence`、`Drop` での保存
- [luajit-migration](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/luajit-migration) — ランタイムの LuaJIT 2.1 への移行
- [ai-friendly-file-split](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/ai-friendly-file-split) — `PastaLuaRuntime` の実装のファイル分割

---

> 【落胆】止まっては動き、動いては止まる――そんな健気なコルーチンなのに、LuaJIT 2.1 には `coroutine.close` がございませんの。閉じてあげることすらできませんのよ……。

> 【アンソニー】参照を外して、GC に静かに引き取ってもらうのでございますね。

> 【お辞儀】おやすみなさいまし、と見送るのも務めのうちですわ。次は、内部モジュールを一つひとつ手に取って確かめて参りましょう。
