# SHIORI 層

> 【したり顔】ベースウェアの呼びかけを受け止め、ゴーストの言葉を返す――それが SHIORI 層の務めですわ。FFI の境界からアクター、イベント配送、DLL のビルド構成まで、きっちり捌いて差し上げます。

> 【アンソニー】VM には専用の部屋をあてがい、取り次ぎは郵便受け一つきり――執事の私には、他人事と思えぬ仕組みでございます。

> 【冷笑】あなたより、よほど律儀な取り次ぎ役ですわよ。届いた順に、一通ずつ必ず捌きますもの。

---

この章では、pasta.dll としてベースウェアと通信する SHIORI 層を扱う。利用者から見たイベントとハンドラは [SHIORI イベントとハンドラ](../lua/shiori-events.md) が正である。

## 目的と責務

SHIORI 層は、ベースウェア（SSP など）が呼ぶライブラリの関数でリクエストを受け取り、Lua VM を所有する専用スレッドへ渡し、Lua 側でイベントを振り分けて作った応答を返す。

入口は 2 つある。どちらも同じ芯の型 `ActorInstance` につながり、同じアクターの処理を通る。

| 入口 | 書き出す関数 | OS | インスタンス |
| ---- | ------------ | -- | ------------ |
| 旧来の SHIORI/3.0 の入口 | `DllMain`・`loadu`・`load`・`request`・`unload`（HGLOBAL で受け渡す） | Windows だけ | プロセスに 1 つ（置き場 `LEGACY`） |
| shiori4 の版 3 | `shiori_get_api`（版 3 のテーブルの `load`・`request`・`notify`・`unload`） | Windows・Linux・macOS | `load` のたびに 1 つ（ハンドルをホストが持つ） |

この章が責務を持つのは次の事項である。

- FFI 境界: 旧来の入口（HGLOBAL の所有と解放、設置パスとリクエストの文字コード、`catch_unwind` による panic の封じ込め）と、版 3 の入口（要求の見分け。ABI の検査は `shiori4_abi` が行う）
- リクエスト文字列の解析と、Lua に渡すリクエストの表の組み立て
- アクターランタイム: インスタンスごとに VM を専用スレッドに固定し、そのインスタンスの mailbox（単一のキュー）経由でだけ触る仕組み。GET と NOTIFY の受け渡し（marshaling）、起動と再読み込み、終了処理（teardown）、ゴーストのディレクトリの占有
- プロセスに 1 つ残す状態と、それぞれを残す理由
- Lua 側の SHIORI エントリ（`SHIORI.load`・`SHIORI.request`・`SHIORI.unload`・`SHIORI.kick`）と、`REG`・既定ハンドラ・シーン関数フォールバックによるイベントの振り分け
- 非同期トーク（`get_property` のコールバック）が SHIORI のリクエストとしてどう往復するか
- 仮想イベントディスパッチャ（OnTalk・OnHour とトーク頻度）
- presentation マーカーとレンダラ注入（`@pasta_sakura_script` の登録の継ぎ目）
- DLL のビルド構成（リリースプロファイル・静的 CRT・バージョン情報）

次の事項は他の章が権威を持ち、この章では再記述しない。

- `EVENT.fire` がハンドラの返したコルーチンを再開し、`STORE.co_scene` とコールバック待ちを更新する規則は [ランタイム実行モデル](execution-model.md#イベントからシーンへ) で扱う。
- VM の構築（`PastaLoader::load_with_config` の内側）は [ローダ自己展開とモジュール解決](loader.md) と [ランタイム実行モデル](execution-model.md#vm-の構築とモジュール登録) で扱う。
- ACT のトークをさくらスクリプトにする処理は [トーク出力とアピアランス](talk-output.md) で扱う。
- シーンキックの要求の発生源（デバッグ通信）と `pasta.shiori.event.kick` の内部は [デバッグ基盤とシーンキック](debug.md) で扱う。
- ログの振り分け（`LoadDirGuard`・`GlobalLoggerRegistry`）と文字コード変換の実装は [ロギングとエンコーディング](logging-encoding.md) で扱う。
- イベントの一覧、ハンドラの戻り値と応答の対応、`act.req` のフィールド、`RES` の関数、仮想ディスパッチャの利用者から見た挙動は [SHIORI イベントとハンドラ](../lua/shiori-events.md) が正である。起動の失敗がどう見えるかは [起動シーケンスとモジュール解決](../reference/startup.md#2-起動シーケンス) が正である。

## 構成要素

### スレッドとデータの経路

```text
ホストのスレッド                                    アクタースレッド "pasta-actor"（インスタンスごとに 1 本）
────────────────                                    ──────────────────────────────────────────
旧来: loadu / load ── lifecycle::spawn_actor ─┐
版 3: load ── PastaV3::load ──────────────────┤
                                              ▼
                      ActorInstance::spawn_with
                        REGISTRY.try_claim(設置パス)   ディレクトリの占有
                        mailbox() と kick の送り先
                        spawn_actor_thread_with ────→ thread::spawn
                        （構築の完了を待つ）            PastaShiori::load（VM を構築）
                              ←── (スレッド ID, 成否, DAP アドレス)
                      旧来: LEGACY.store(インスタンス)    while let Ok(msg) = rx.recv()
                      版 3: PastaV3 がインスタンスを持つ
旧来: request ── marshal_request ── ActorInstance::request
                                     determine_method（pest で GET/NOTIFY を判定）
版 3: request ── classify_request ── GET は get・NOTIFY は notify・ほかは 400
       GET:    try_send(Get{req, reply}) ────────────→ shiori.request(生文字列)
               recv_timeout(5 秒) ←─────────────────── reply.send(Reply::Value(応答))
       NOTIFY: try_send(Notify{req}) → 即 204 ──────→ shiori.request(生文字列)（応答は捨てる）
旧来: unload ── teardown_actor（DllMain は teardown_actor_detached）
版 3: unload ── ActorInstance の Drop
       send(Stop{done}) ─────────────────────────────→ ループを抜ける → drop(PastaShiori) → drop(rx)
       recv_timeout(5 秒) ←──────────────────────────── done.send(())
       ack が届いた: join（DllMain の道だけ join せず手放す）
       届かない:     手放す（warn）

デバッグ通信のスレッド ── そのインスタンスの KickSink → try_send(Kick{scene}) → shiori.kick(scene)
```

VM（`PastaLuaRuntime` を内包する `PastaShiori`）は `!Send` であり、アクタースレッドで作られ、そこで使われ、そこで破棄される。スレッドをまたぐのは、生のリクエスト文字列・応答文字列・シーン名・応答用のチャネルの送信端など `Send` な値と、`Send + Sync` な `ActorInstance` だけである。アクタースレッドは同期の `recv()` だけで待ち、OS のウィンドウやメッセージの仕組みを使わないので、どの OS でも同じ形で動く。

### Rust 側（pasta_shiori）

| ファイル | 役割 |
| -------- | ---- |
| `crates/pasta_shiori/src/windows.rs` | 旧来の入口（`#[cfg(windows)]`）。`loadu`・`load` の共通本体 `load_entry`、`loadu` 済みの印 `LOADU_INITIALIZED`、`unload` と `DllMain` の共通の後始末 `teardown`、各入口の `catch_unwind`、応答の HGLOBAL 化。戻り値は C の `BOOL`（`i32`）、長さは C の `long` |
| `crates/pasta_shiori/src/v3.rs` | 版 3 の入口（全部の OS）。`shiori_get_api` と版 3 のテーブル `API_V3`、`shiori4_abi::Shiori` を実装する `PastaV3`（`ActorInstance` を 1 つ持つ）、応答の開始行から状態の数を読む `status_of` |
| `crates/pasta_shiori/src/util/hglobal/` | `ShioriString`（HGLOBAL の取り込みと解放、`GMEM_FIXED` での確保、UTF-8・ANSI としての読み出し）。ANSI の変換は `MultiByteToWideChar`（`CP_ACP`）による |
| `crates/pasta_shiori/src/actor/instance.rs` | 芯の型 `ActorInstance`（mailbox の送信端・アクタースレッドのハンドル・設置パス・要求の連番）。起動 `spawn`・`spawn_with`（ディレクトリの占有 `ClaimMode`）、`request`・`get`・`notify`・`kick`、後始末 `shutdown`（`JoinPolicy`）と `Drop`、インスタンスごとの kick の送り先 |
| `crates/pasta_shiori/src/actor/lifecycle.rs` | 旧来の入口の置き場 `static LEGACY`（`ArcSwapOption<ActorInstance>`）と、旧来の入口が呼ぶ `spawn_actor`・`marshal_request`・`teardown_actor`・`teardown_actor_detached` |
| `crates/pasta_shiori/src/actor/mailbox.rs` | mailbox を流れるメッセージ `ActorMsg`（`Get`・`Notify`・`Stop`・`Kick`。`#[non_exhaustive]`）、`MailboxRequest`（`seq` と生のリクエスト文字列）、`Reply`、mailbox の生成（flume の unbounded） |
| `crates/pasta_shiori/src/actor/thread.rs` | アクタースレッドの起動 `spawn_actor_thread_with`（失敗を返す）と、同期の `recv()` でメッセージを処理するループ。`ActorThread`（スレッドのハンドル・ロードの成否・DAP の待受アドレス。`join`・`detach`） |
| `crates/pasta_shiori/src/actor/marshaling.rs` | 旧来の入口のメソッドの判定 `determine_method`、版 3 の要求の見分け `classify_request`（`RequestKind`）、`marshal_get`・`marshal_notify`、安全網の応答 `default_204`、GET の待ち時間の上限 `GET_TIMEOUT`（5 秒） |
| `crates/pasta_shiori/src/actor/teardown.rs` | `Stop{done}` を送って完了の通知を待つ `teardown_via_sender` と結果 `TeardownReport`（`acked`・`already_done`・`anomaly`・`joined`）。再読み込みを繰り返して OS のハンドル数を測る `ReloadProbe`（テストが使う） |
| `crates/pasta_shiori/src/shiori.rs` | `Shiori` トレイトと `PastaShiori`。ランタイムの構築、`SHIORI.load`・`SHIORI.request`・`SHIORI.unload` の関数のキャッシュ、`SHIORI.kick` の呼び出し、ディレクトリの占有の札（`set_claim`）、`Drop` での後始末 |
| `crates/pasta_shiori/src/lua_request.rs` | リクエスト文字列を Lua の表にする `parse_request` と、日時の表を作る `lua_date`・`lua_date_from` |
| `crates/pasta_shiori/src/util/parsers/` | SHIORI/3.0 と SHIORI/2.x のリクエストの pest 文法（`crates/pasta_shiori/src/util/parsers/req_parser.pest`）と、そこから生成されるパーサ |
| `crates/pasta_shiori/src/error.rs` | `MyError` と、500 応答・400 応答の文字列を作る `to_shiori_response`・`to_shiori_400_response` |
| `crates/pasta_shiori/build.rs` | pasta.dll に埋め込むバージョン情報（`VERSIONINFO`）の生成 |

`crates/pasta_shiori/src/util/shiori.md` は SHIORI の外部仕様の写しで、実装からは参照されない。

### Rust 側（pasta_lua）

| ファイル | 役割 |
| -------- | ---- |
| `crates/pasta_lua/src/presentation/` | 宿主に依存しない出力の表現 `PresentationMarker`、トークの付随値を引く `TokenFields`、マーカー列を宿主の出力にする境界 `RenderBoundary` |
| `crates/pasta_lua/src/runtime/renderer_injection.rs` | `@pasta_sakura_script` を作る関数を差し替え可能に包む `RendererInjection`、既定のレンダラ `default_sakura_renderer`、`RenderBoundary` の実装 `SakuraRenderBoundary` |

### Lua 側（`crates/pasta_lua/pasta_scripts/pasta/shiori/`）

| モジュール | 役割 |
| ---------- | ---- |
| `pasta.shiori.entry` | グローバルの `SHIORI` 表（`load`・`request`・`kick`・`unload`）。`GLOBAL.close_ghost`（`GLOBAL["ゴースト終了"]`）・`GLOBAL.pause_auto_talk`（`GLOBAL["自動トーク停止"]`）・`GLOBAL.resume_auto_talk`（`GLOBAL["自動トーク再開"]`）も登録する |
| `pasta.shiori.event` | `EVENT.fire`（振り分けとコルーチンの再開）と、ハンドラが無いときの `EVENT.no_entry` |
| `pasta.shiori.event.register` | `REG`。依存を持たない表で、生のキーには代入したハンドラ（鎖の底）だけを置く。鎖はモジュールのローカル変数 `chains` に持ち、読み出しの `REG.register` はメタテーブルの `__index` から登録の口（`REG:register(id, fn)`）を返す。`EVENT.fire` が鎖の写しを取る内部の口は、`__index` でなくメタテーブルの `handlers` 欄（`getmetatable(REG).handlers(id)`）に置き、`REG.handlers` からは見えない。`REG.register` に代入すると、生のキーが先に引かれて登録の口が隠れる（代入を今と同じに働かせるため止めない） |
| `pasta.shiori.event.boot`・`pasta.shiori.event.choice_select`・`pasta.shiori.event.second_change`・`pasta.shiori.event.mouse` | 既定ハンドラ（OnBoot・OnChoiceSelectEx・OnSecondChange・OnMouseDoubleClick・OnMouseMove）を `REG` に登録する。`pasta.shiori.event.mouse` は、つつき・撫でのショートカットの候補の名前・アクター名の解決・撫での判定も持つ |
| `pasta.shiori.event.virtual_dispatcher` | 仮想イベント（OnHour・OnTalk）の判定と発行 |
| `pasta.shiori.clock` | SSP の仮想時刻と OS の時計のずれ・時刻の飛びを持つ時計。合図の判定・仮想時刻の読み取り・ずれを足した時刻と日付・飛びを差し引いた時計 |
| `pasta.shiori.event.callback` | 非同期トークのコールバック待ちの登録・ルーティング・タイムアウトの掃引 |
| `pasta.shiori.event.kick` | シーンキックの保留と起動（[デバッグ基盤とシーンキック](debug.md)） |
| `pasta.shiori.event.scene_play` | 既定ハンドラ（OnPastaScene・OnPastaSceneAt）を `REG` に登録する。デバッグが有効なときだけ、名前か位置で指名したシーンを再生する（[位置でのシーンの再生](debug.md#位置でのシーンの再生)） |
| `pasta.shiori.event.recommend` | 既定ハンドラ（メニューのリソース `sakura.recommendsites` など・ボタンの名前のリソース・OnRecommendsiteChoice）を `REG` に登録し、`＞おすすめ項目`・`＞おすすめ区切り` を `GLOBAL` に置く。メニューのシーンは `EVENT.drive` を通さず、自前のコルーチンで 1 回だけ動かして、続き・持ち越し・ほかの予約に触れない（[おすすめメニュー](../grammar/call-jump.md#おすすめメニュー)） |
| `pasta.shiori.event.external_control` | 既定ハンドラ（OnPastaCallFunction・OnPastaSetSaveVariable）を `REG` に登録する。デバッグの有無によらず、名前で指名したものを `＞名前` と同じ探し方で呼ぶか、グローバル変数に代入する（[OnPastaCallFunction](../lua/shiori-events.md#onpastacallfunction名前を外から呼ぶ)・[OnPastaSetSaveVariable](../lua/shiori-events.md#onpastasetsavevariableグローバル変数に代入)） |
| `pasta.shiori.res` | SHIORI/3.0 の応答文字列を作る `RES` |
| `pasta.shiori.act`・`pasta.shiori.sakura_builder`・`pasta.shiori.appearance`・`pasta.shiori.actor_style` | SHIORI 用の ACT とさくらスクリプトの組立、外見の復旧、装いの出し入れ（[トーク出力とアピアランス](talk-output.md)） |
| `pasta.shiori` | 空の表を返すだけのモジュール。リポジトリ内に `require` する箇所は無いが、展開されるスクリプトの公開面として残している |

## 処理とデータの流れ

### load とアクターの起動

#### 旧来の入口

`loadu` と `load` は、設置パスの文字コードだけが異なる 2 つの入口で、どちらも `load_entry` を通る。`loadu` は UTF-8、`load` はシステムの ANSI コードページ（`CP_ACP`）として設置パスを読む。

```text
load_entry(entry, hdir, len, encoding)
 1. hdir が null → false
 2. ShioriString::capture(hdir, len)   以後のどの経路でも Drop で GlobalFree される
 3. load かつ LOADU_INITIALIZED → 何もせず true（loadu 済みの状態を保つ）
 4. len == 0 → false（C の long の負の長さは 0 に読む）
 5. catch_unwind(load_impl)            panic は false
load_impl
 a. 設置パスを UTF-8（to_utf8_str）か ANSI（to_ansi_str）で読む。失敗 → false
 b. loadu なら LOADU_INITIALIZED = true（ロードが失敗しても立てたままにする）
 c. lifecycle::spawn_actor(0, 設置パス)
 d. 同じ設置パスで LoadDirGuard を張り直し、入口名と成否を 1 行ログに出す
```

- `LOADU_INITIALIZED` をロードの失敗時にも立てるのは、続いて `load` が呼ばれたときに、ANSI で欠落したパスによる別の失敗で `loadu` の失敗の原因を上書きしないためである。印は `unload`（と `DllMain` の FreeLibrary の detach）で下ろす。利用者から見た扱いは [起動シーケンスとモジュール解決](../reference/startup.md#loadu-を呼ばないホストでは非-ansi-の設置パスを扱えない) を参照する。
- `SHIORI.load` に渡す `hinst` は、FFI の経路では常に 0 である（`DllMain` が受け取るモジュールハンドルは保持しない）。
- d でガードを張り直すのは、ログのファイルへの振り分けがスレッドごとの設置パスで決まり、アクタースレッドで張ったガードが FFI 入口のスレッドには及ばないためである（[ロギングとエンコーディング](logging-encoding.md)）。

`spawn_actor` は次の順に動く。

```text
spawn_actor(hinst, load_dir)
 1. 前のインスタンスの設置パスを手元に取る
 2. 置き場の後始末（teardown_actor と同じ。ack が届けば join）  前のインスタンスがあれば終わらせる（再読み込み）
 3. 2 が打ち切り（ack が来ずにスレッドを手放した）で、1 と同じ設置パスなら ClaimMode::TakeOver、
    そうでなければ ClaimMode::Exclusive
 4. ActorInstance::spawn_with(hinst, load_dir, mode)。失敗 → false（置き場は空のまま）
 5. LEGACY.store(Some(Arc::new(インスタンス)))
 6. 成否（インスタンスの loaded）を返す
```

3 の引き継ぎは、手放したスレッドの VM がまだ持つ占有を新しい世代で引き継ぎ、今の SSP での再読み込みの振る舞いを保つ。手放された古い VM が後で片付けを終えても、札の世代が違うので新しい持ち主の項目は消えない（[ディレクトリの占有](#ディレクトリの占有)）。

#### 版 3 の入口

`shiori_get_api(3)` は版 3 のテーブル `API_V3`（プロセスの間ずっと同じ static）を返し、3 以外（4 を含む）には NULL を返す。テーブルの関数は `shiori4_abi` が実装し、`copy_from_slice` の有無・UTF-8 の検査・パニックの捕捉・応答の写しを行ってから、`PastaV3` の `Shiori` の実装を呼ぶ。

```text
PastaV3::load(mode, dir)
 1. mode が V3 でなければ SHIORI_E_UNSUPPORTED
 2. dir が絶対パスでないか、ディレクトリでない → SHIORI_E_INVALID_ARG（ログは出さない）
 3. ActorInstance::spawn(0, dir)          ClaimMode::Exclusive。失敗 → 500（NULL）
 4. 同じ設置パスで LoadDirGuard を張り、入口名 v3 と成否を 1 行ログに出す
 5. インスタンスを返す（辞書・Lua・SHIORI.load の失敗でも返す）
```

2 でカレントディレクトリに頼らないのは、ホストのカレントディレクトリがゴーストのディレクトリと限らないためである。版 3 は再読み込みの印も置き場も持たず、`load` のたびに新しいインスタンスを作る。旧来の入口のインスタンスとは別のインスタンスとして並ぶ。

#### ActorInstance の起動

`ActorInstance::spawn_with` は、どちらの入口からも次の順に動く。

```text
ActorInstance::spawn_with(hinst, load_dir, mode)
 1. 占有: Exclusive → REGISTRY.try_claim(load_dir)。項目が既にあれば
            その設置パスのログに warn を出して Err(DirInUse)
          TakeOver  → REGISTRY.take_over(load_dir)。引き継いだことを warn
 2. (tx, rx) = mailbox()                 このインスタンスの新しいチャネル
 3. kick の送り先を作る                   tx の弱い参照だけを持つ
 4. spawn_actor_thread_with(hinst, load_dir, rx, 送り先, 札)
      スレッド "pasta-actor" を起こし、その上で
        LoadDirGuard を張る（スレッドの終わりまで）
        PastaShiori::with_kick_sink(送り先) に札を渡し、load(hinst, load_dir)
        (スレッド ID, 成否, DAP の待受アドレス) を bounded(1) で送り返す
        メッセージループへ入る
      呼び出し側は送り返しを recv() で待つ   → VM の構築と SHIORI.load が終わるまで戻らない
      送り返しの前にスレッドが終わった      → そのスレッドを join して io::ErrorKind::Other
 5. 4 が失敗 → 既定の PastaLogger を登録してから error を出し、占有を外して
              Err(NotReady)（スレッドが送り返す前に終わった）か Err(Thread)（起こせない）
 6. ActorInstance { tx, スレッドのハンドル, load_dir, seq } を返す
```

5 でロガーをその場で作るのは、VM のロガーがまだ無いときにも、立ち上がれなかった理由をゴーストのログに残すためである。

`PastaShiori::load` はアクタースレッドの上で次を行う。

1. 設置パスを覚え、存在しなければ `false` を返す（エラーの理由は残らない。覚えておくのは、VM の片付けでそのディレクトリの占有を外すため）。
2. 既定の設定でロガーを `GlobalLoggerRegistry` に登録し（占有で作られた項目にロガーが入る）、`init_tracing()` で購読者を入れ（プロセスで最初の 1 回だけ）、`LoadDirGuard` を張る。
3. `RuntimeConfig::new().with_kick_sink(このインスタンスの送り先)` を作り、`PastaLoader::load_with_config` でランタイムを構築する。失敗したらエラーの文字列を `last_load_error` に残して `false` を返す。
4. `SHIORI` 表から `load`・`request`・`unload` の関数を取り出してキャッシュする。リクエストのたびに表を引かないためである。どれも省略でき、無い `load` は成功として扱い（5）、無い `request` には 204 を返し（後述）、無い `unload` は呼ばない。`unload` 以外が無いときは warn ログを出す。
5. `SHIORI.load(hinst, 設置パスの文字列)` を呼ぶ。関数が無ければ成功とみなす。`false` を返すかエラーなら `false`。

ロードが失敗しても、アクタースレッドは起動したままインスタンスに残り（旧来の入口では `LEGACY` に入り、版 3 ではインスタンスを返す）、以後のリクエストを受け付ける。GET の応答は失敗の段階で異なる。3 のランタイムの構築に失敗した場合は、残した理由を 500 応答で返す。1 で設置パスが無かった場合は、理由の無い 500 応答（`Not initialized error`）を返す。5 で `SHIORI.load` が失敗した場合はランタイムが残っているため、リクエストは通常どおり `SHIORI.request` で処理される（[アクタースレッドのメッセージループ](#アクタースレッドのメッセージループ) の `PastaShiori::request`）。

### request と GET・NOTIFY の振り分け

```text
request(req, &mut len)
 1. req が null → null を返し len = 0
 2. ShioriString::capture(req, len)      入力の HGLOBAL は必ず解放される
 3. catch_unwind(request_impl)           panic は default_204
request_impl
 a. UTF-8 として読む。失敗 → default_204
 b. lifecycle::marshal_request(文字列)
 c. 応答を GlobalAlloc(GMEM_FIXED) の領域へ写し、len に長さを入れて返す
    確保に失敗したら null と len = 0
marshal_request
 i.   LEGACY.load_full()。空（load 前・unload 後）→ default_204
 ii.  ActorInstance::request
        determine_method：pest の req 規則で解析し、最初に現れた get / notify を採る
        解析できない → default_204
        GET    → get    （seq を採番して marshal_get）
        NOTIFY → notify （seq を採番して marshal_notify）
```

版 3 の `request`・`notify` は、`classify_request` で要求を見分けてから渡す。

| 見分け（`RequestKind`） | `request` | `notify` |
| ----------------------- | --------- | -------- |
| `Get`（`GET SHIORI/3.0`） | `ActorInstance::get` の応答の全文。戻り値の状態の数は、開始行 `SHIORI/3.0 NNN` の NNN（`status_of`）。読めなければ 500 にして warn を出し、本文はそのまま返す | アクターへ送らず 400 |
| `Notify` | `ActorInstance::notify`（即 204 の全文）。戻り値は 204 | `ActorInstance::notify`。戻り値は 204 |
| `Shiori2`（SHIORI/2.x のヘッダ）・`Unreadable`（今の文法で読めない） | アクターへ送らず、理由を `X-ERROR-REASON` に入れた 400 の全文 | アクターへ送らず 400 |

弾いた要求は、そのインスタンスのログに debug を 1 行出す。旧来の入口は `determine_method` を使い続け、読めない要求に 204 を返す振る舞いを変えない。

- `seq` はインスタンスごとの連番で、ログで順序を追うためだけに使う。
- `marshal_get` は `bounded(1)` の応答用チャネルを作って `ActorMsg::Get` に同梱し、`try_send` で投入してから `recv_timeout(GET_TIMEOUT)` で待つ。値が届けばそれを返し、応答用チャネルが送られずに破棄された（`Disconnected`）か、5 秒を過ぎた（`Timeout`）か、`try_send` に失敗したなら `default_204` を返す。
- `marshal_notify` は `ActorMsg::Notify` を `try_send` し、成否に関わらずすぐ `default_204` を返す。アクターの処理は待たない。
- メソッドの判定は VM に投入する前に、ホストのスレッドで行う。同じリクエスト文字列は、アクタースレッドで `parse_request` がもう一度解析する。
- 旧来の入口の応答の HGLOBAL はホストが解放する。DLL は解放しない。版 3 の応答は `shiori4_abi` がホストの `copy_from_slice` で写す。

### アクタースレッドのメッセージループ

アクタースレッドは、同期の `rx.recv()` だけで待つ単一のループを回す。別のスレッドの `try_send`・`send` が `recv()` を起こす。非同期の実行器・OS のウィンドウ・メッセージの仕組みは使わない。

```text
while let Ok(msg) = rx.recv()
  Get { req, reply }  → shiori.request(&req.raw)
                          Ok(応答)   → reply.send(Reply::Value(応答))
                          Err(e)     → reply.send(Reply::Value(e.to_shiori_response()))  500 応答
  Notify { req }      → shiori.request(&req.raw)（結果は捨てる）
  Kick { scene }      → shiori.kick(&scene)
  Stop { done }       → done を持ってループを抜ける
ループの後
  drop(shiori)        PastaShiori の Drop（後述）
  drop(rx)            以後の Stop の再送が Disconnected になる
  done.send(())       全資源を解放した後に完了を通知する
スレッドが終わる      LoadDirGuard を外す（スレッドの局所の値だけ。待つ所は無い）
```

`PastaShiori::request` は、ランタイムが無ければ `MyError::Load`（ロードの失敗の理由）か `MyError::NotInitialized` を返す。ランタイムがあれば `LoadDirGuard` を張り、次の順で `SHIORI.request` を呼ぶ。

1. キャッシュした `SHIORI.request` が無ければ、Rust で作った 204 応答（`Charset` と `Sender` の 2 ヘッダ）を返す。
2. `lua_request::parse_request` でリクエストを Lua の表にする。失敗したら 400 応答（`to_shiori_400_response`）を返す。
3. `SHIORI.request(表)` を呼び、戻り値の文字列を返す。呼び出しが失敗したら（`SHIORI.request` の `xpcall` の外でのエラー、戻り値を文字列にできないなど）`MyError::Script` を返し、アクターがそれを 500 応答にする。204 以外の応答では、リクエストと応答の全文を debug レベルでログに出す。

`parse_request` が作る表は次のとおりである。フィールドの意味は [act.req](../lua/shiori-events.md#actreq) が正である。

- `reference`・`dic`・`date` は常に作る。`date` は `OffsetDateTime::now_local()` から作り、ヘッダ `X-Pasta-Time`（RFC 3339）があればその日時で作り直す。`X-Pasta-Time` が解析できなければ `MyError::InvalidPastaTime` になり、400 応答になる。
- `Charset`・`ID`・`BaseID`・`Status`・`SecurityLevel`・`Sender` は対応するフィールドに、`ReferenceN` は `reference[N]` に入れる。どのヘッダも `dic[ヘッダ名]` に入る。
- 1 行目が `GET` か `NOTIFY` かで `method` を `"get"`・`"notify"` にする。SHIORI/3.0 は `version = 30`、SHIORI/2.x は 1 行目のイベント名を `id` に、版を `20`〜`29` にする。
- 解析結果の列は再帰ではなく反復で処理する。ヘッダの数はホストが決めるため、再帰ではスタックを使い果たしてホストのプロセスごと落ちうる。

### unload・DllMain と teardown

```text
旧来: unload()                          → teardown("unload", teardown_actor)
旧来: DllMain（FreeLibrary の detach）  → teardown("DllMain", teardown_actor_detached)
teardown(入口名, 窓口)
 1. LOADU_INITIALIZED = false
 2. catch_unwind(窓口)                  異常があれば warn ログ
 3. 常に 1（TRUE）
teardown_actor()          = 置き場の後始末(JoinPolicy::JoinAfterAck, 5 秒)
teardown_actor_detached() = 置き場の後始末(JoinPolicy::Detach, 5 秒)
置き場の後始末(policy, timeout)
 a. インスタンス = LEGACY.swap(None)     以後の送信を断つ。空なら already_done（冪等）
 b. Arc::try_unwrap が成功 → ActorInstance::shutdown_with_timeout(policy, timeout)
    失敗（別のスレッドで要求が走っている） → request_stop（Stop と ack の待ちだけ）。
      最後の参照の Drop でスレッドを手放す
版 3: unload（shiori4_abi が PastaV3 を Drop） → ActorInstance の Drop
      まだ片付けていなければ shutdown と同じ手順を JoinPolicy::JoinAfterAck で行う
ActorInstance の後始末(policy, timeout)
 i.  teardown_via_sender(tx, timeout)
 ii. ack が届き、policy が JoinAfterAck → ActorThread::join（joined = true）
     それ以外                          → detach（手放す）。ack が来なかったなら warn「actor thread abandoned」
teardown_via_sender(tx, timeout)
 - send(Stop{done}) が失敗（受信側がもう無い）→ already_done（何もしない）
 - done を recv_timeout で待つ
     届いた      → acked（正常な終了）
     Timeout     → anomaly（warn ログ）
     Disconnected → anomaly（warn ログ）
```

- 後始末の決まりは、ack を待ち、届いたら join する、である。join するのは ack が届いたときだけで、ack の後にアクタースレッドがするのは受け口を閉じた後の ack の送信とスレッドの局所の値の片付けだけなので、join は上限なしでも止まらない。そのため `unload` が返るとき、アクタースレッドはライブラリのコードを出ている。
- `DllMain` の道だけは join しない（`JoinPolicy::Detach`）。`DllMain` はローダーロックを持ったまま呼ばれ、その中ではスレッドの終わりを待てないためである。ack までは待つので、VM の破棄は済んでいる。
- ack が 5 秒で来なければスレッドを手放す。手放したスレッドの VM が持つディレクトリの占有は、そのスレッドが VM を片付けるまで残る。版 3 から同じディレクトリをすぐ読み直すと断られる。旧来の入口の再読み込みだけは引き継ぐ（[load とアクターの起動](#旧来の入口)）。
- `try_unwrap` が失敗するのは、片付けと同時に別のスレッドで要求が走っているとき（SHIORI の単一スレッドの規約に反する呼び方）だけである。最後の参照の Drop が送る `Stop` は、先の `Stop` でループを抜けたアクターに届かず already_done になるため、その Drop が join で待つことは無い（`DllMain` の道でも待たない）。
- `Stop` は他のメッセージと同じ FIFO を通るため、先に積まれたメッセージを処理し終えてからループを抜ける。
- `PastaShiori` の `Drop` は、キャッシュした `SHIORI.unload` を呼び（`call_lua_unload`）、続けて `release_runtime` でランタイムを手放す。`release_runtime` は次の順に進む。
  1. 設置パスがあれば `LoadDirGuard` を張る（メソッドを抜けるまで保つ）。
  2. キャッシュした Lua 関数を捨てる（VM を参照しているため、ランタイムより先に捨てる）。
  3. ランタイムを破棄する。ここで永続化データが保存され（[永続化](execution-model.md#永続化)）、DAP のバックエンドも VM の一部として片付く。
  4. info「Unregistering logger」を出す。
  5. 占有の札があれば `release_claim(札)`（札の世代が項目の今の世代と同じときだけ項目を消す）、無ければ `unregister(設置パス)` で、ロガーの登録と占有を外す。最後の参照が落ち、フラッシュしてログファイルが閉じる。

  登録を外すのを最後にするのは、`SHIORI.unload` の結果・永続化データの保存の失敗・登録解除の通知を、ファイルを閉じる前に書くためである（[FFI 入口と終了処理のログ](#ffi-入口と終了処理のログ)）。
- 完了の通知は、VM の破棄と mailbox の受信側の破棄が終わってから送る。通知を受け取った時点で、VM・DAP バックエンド・ディレクトリの占有の解放は済んでいる。
- `DllMain` は `DLL_PROCESS_DETACH` のうち `FreeLibrary` によるもの（`lpReserved` が null）で `teardown_actor_detached` の後始末を行い、それ以外の通知では何もしない。プロセスの終了による `DLL_PROCESS_DETACH`（`lpReserved` が null でない）では、他のスレッドが既に止められていてアクターが応答できないため、teardown せずに 1 を返す。そのため、`unload` を呼ばれずにホストのプロセスが終わったときは、`SHIORI.unload` も永続化データの保存も走らない。`DllMain` はローダーロックを持ったまま呼ばれるため、そこでスレッドを起こさない。スレッドは `load` を起点に起こす。`DllMain` が片付けるのは旧来の入口のインスタンスだけで、版 3 のインスタンスはホストの `unload` が片付ける。
- 再読み込み（`unload` の後の `load`、または `load` の再呼び出し）は、`spawn_actor` の 2 で前のインスタンスを終わらせ、新しいスレッド・チャネル・VM を作る。`PastaShiori::load` の中にある、既存のランタイムを捨てて作り直す分岐（info「Releasing existing runtime for reload」→ `release_runtime` → `last_load_error` を消す。`SHIORI.unload` は呼ばない）は、FFI の経路では通らない（インスタンスごとに新しい `PastaShiori` を作るため）。

### FFI 入口と終了処理のログ

ログの振り分けの規則は [ロギングとエンコーディング](logging-encoding.md#振り分けの規則) が書く。この節は、SHIORI の各入口と終了処理のログがその規則でどうなるかを書く。旧来の入口だけを使うホスト（SSP）では、登録簿のロガーは 0 個か 1 個である。版 3 のインスタンスが並ぶと 2 個以上になり、文脈の無いログは捨てる。

| 入口・処理 | スレッドの文脈 | 扱い |
| ---------- | -------------- | ---- |
| アクタースレッド（メッセージループの観測ログ、`PastaShiori` の各メソッドのログ） | スレッドの入口で設置パスの `LoadDirGuard` を張り、スレッドの終わりまで保つ | 自分のロガーへ届く |
| `ActorInstance` の起動と後始末（占有を断った warn・引き継いだ warn・起動の失敗の error・スレッドを手放した warn） | 出す間だけ設置パスの `LoadDirGuard` を張る | そのディレクトリのロガーへ届く。起動の失敗は、その場で既定のロガーを登録してから出す |
| 旧来の `loadu`・`load`（`load_entry`） | 無し。`load_impl` は入口名と成否の 1 行を出す間だけ張り直す | 最初のロード前の引数の誤りや設置パスを読めないときのログは、登録が無いので捨てる。`loadu` 済みで無視した `load` の warn は、登録されたロガーがちょうど 1 つなら届く |
| 旧来の `request` | 無し | 不正なリクエストの warn、境界での panic の error、観測用の debug・trace は、登録されたロガーがちょうど 1 つなら届く。`load` の前・`unload` の後は捨てる |
| 旧来の `unload` | 無し | teardown の異常の warn は後述 |
| `DllMain` の detach（`FreeLibrary`） | 無し | `unload` と同じ（join しないことだけが違う） |
| `DllMain` の detach（プロセスの終了） | — | 何もしないので、ログも出ない。`SHIORI.unload` と永続化データの保存も走らない |
| 版 3 の `load`・`request`・`notify` | 入口の 1 行・弾いた要求の debug・開始行を読めない warn を出す間だけ、インスタンスの設置パスで張る | そのインスタンスのロガーへ届く。絶対パスでない・ディレクトリが無いときは置き場が無いのでログを出さない |

終了処理（`unload`、または `FreeLibrary` による detach）のログは、出る時点で次のように分かれる。

```text
FFI 入口のスレッド: send(Stop{done})
  debug「Stop enqueued」                  … アクターの登録解除より前なら届く
アクタースレッド: Stop を受けてループを抜け、drop(shiori)（PastaShiori の Drop）
  SHIORI.unload・永続化データの保存のログ … 届く
  info「Unregistering logger」            … 届く
  ロガーの登録を外す（ファイルを閉じる）
アクタースレッド: drop(rx) → done.send(())
  debug「actor.done」                     … 捨てる
FFI 入口のスレッド: recv_timeout(5 秒)
  届いた       → debug「done ack received」 … 捨てる。続けて join（DllMain は手放す）
  Timeout      → warn                       … 届く（ロガーはまだ登録されている）
                 warn「actor thread abandoned」… 届く（設置パスの文脈を張って出す）
  Disconnected → warn                       … 登録解除の後なら捨てる
```

- 完了の通知（done ack）の後のログは、そのディレクトリのロガーがもう登録されていないので捨てる。done ack を送る位置は変えないため、通知を受け取った時点で VM・DAP バックエンド・ログファイルの解放が済んでいる、という不変条件は保たれる。
- Timeout のとき、アクターは終了処理の途中で、ロガーは登録されたままである。FFI 入口のスレッドの文脈の無い warn は、ロガーがちょうど 1 つなら届く。Disconnected は巻き戻すプロファイルでの panic でしか起きず、そのときロガーが既に登録を外されていれば warn は捨てる。
- ログを残すために、`unload` の戻り値（常に 1）・待ち時間の上限（5 秒）・応答の内容は変えない。

### ディレクトリの占有

同じゴーストのディレクトリを 2 つのインスタンスが読むと、セーブとログのファイルを取り合う。そのため、生きているインスタンスどうしは同じディレクトリを読まない。占有は、ロガーの登録簿 `GlobalLoggerRegistry`（`REGISTRY`）の項目で表す。項目が在ること＝そのディレクトリが占有されていることで、占有のためにプロセスに 1 つの状態を新しく足さない。

- 1 項目は、ロガー（まだ無いこともある）・量の設定・世代の数を持つ。鍵は設置パスの `PathBuf` で、正規化せずに比べる。
- `try_claim` は項目が無いときだけ「ロガー無し・既定の量」の項目を新しい世代で足し、札（`ClaimToken`。鍵と世代）を返す。項目があれば `None` で、起動は `DirInUse` で断る。
- `take_over` は項目の世代を新しくして札を返す（ロガーと量の設定は引き継ぐ）。使うのは旧来の入口の再読み込みだけである。
- `release_claim` は、札の世代が項目の今の世代と同じときだけ項目を消す。世代の数は表に 1 つの連番で、項目を消しても戻さない。項目ごとの数にすると、消して作り直した項目の世代が古い札と重なり、手放された古い VM が今の持ち主の項目を消してしまうためである。
- 札を持つのは VM（`PastaShiori`）で、`release_runtime` の最後に返す。起動に失敗したときは `ActorInstance::spawn_with` が返す。

旧来の入口の `load` が、版 3 のインスタンスが占有しているディレクトリを指したときは FALSE を返し、アクターを起こさない（以後の GET は 204）。

### プロセスに 1 つ残す状態

インスタンスの状態（VM・アクタースレッド・mailbox・辞書・変数・セーブ・ログの出し先・要求の連番・kick の送り先）は、どれもインスタンスごとに持つ。プロセスに 1 つ残すのは次のものだけである。

| 状態 | 場所 | 残す理由 |
| ---- | ---- | -------- |
| `LEGACY`（`ArcSwapOption<ActorInstance>`） | `crates/pasta_shiori/src/actor/lifecycle.rs` | 旧来の入口は、呼び出し側にインスタンスのハンドルを返せない ABI であるため。旧来の入口の中に閉じ込め、芯・アクター・版 3 の入口は参照しない |
| `LOADU_INITIALIZED`（`AtomicBool`） | `crates/pasta_shiori/src/windows.rs` | 同じく旧来の入口のハンドルの無い ABI のため（`loadu` の後の `load` を何もしないことにする印）。旧来の入口の中に閉じ込める |
| tracing の購読者 | `crates/pasta_lua/src/logging/tracing_init.rs` の `init_tracing` | Rust の tracing の既定の購読者は、プロセスに 1 つの仕組みであるため |
| `REGISTRY`（`GlobalLoggerRegistry`） | `crates/pasta_lua/src/logging/registry.rs` | 1 つの購読者から、ログをゴーストのディレクトリごとに振り分けるため。ディレクトリの占有と世代の連番もここに持つ |
| `UNIQUE_COUNTER`（`AtomicU64`） | `crates/pasta_lua/src/loader/extract.rs` | 自己展開の一時・退避ディレクトリの名前の接尾辞を作る数を振るだけで、振る舞いに効かない |

- 版 3 のテーブル `API_V3` は変わらない定数で、状態ではない。
- ログの文脈 `CURRENT_LOAD_DIR` はスレッドごとの値（`thread_local!`）で、プロセスに 1 つではない。
- DAP の待受のポートの重なりは OS のポートで見分け、プロセスの状態を持たない（後から読んだほうが警告を出してデバッグなしで立ち上がる）。

### ログの量の判定

購読者は `registry().with(fmt::layer().with_writer(GlobalLoggerRegistry).with_filter(RegistryFilter))` の 1 つで、プロセスに 1 つの量の設定は持たない。量の設定（`EnvFilter`）は `REGISTRY` の項目ごとに持ち、`update_tracing_filter(load_dir, config)` がその項目の量の設定を作り直す（[ロギングとエンコーディング](logging-encoding.md#ログフィルタ)）。

- `RegistryFilter::enabled` は、書き込み先を選ぶのと同じ決まりで項目を選び、その項目の量の設定で判定する。スレッドの文脈があればその項目、無ければロガーを持つ項目がちょうど 1 つのときだけその項目である。選べなければ出さない。ほかのゴーストの設定へ逃げない。
- 判定はスレッドの文脈で変わるので、`callsite_enabled` は `Interest::sometimes()` を返し、コールサイトごとに覚えさせない。`max_level_hint` は全部の項目の量の最大（項目が無ければ OFF）で、それより細かいイベントは購読者に届く前に止まる。表を変えたら、ロックを外してから `rebuild_interest_cache()` で作り直す（ロックを持ったまま呼ぶと、`max_level_hint` が同じロックを取って止まる）。
- 1 つの `EnvFilter` を項目ごとに `Filter::enabled` で呼ぶだけなので、スパンの中身で絞る指定（`[span{field=…}]` のような動的な指定）は効かない。ターゲットと量の指定（`warn,pasta_lua=debug` など）は効く。
- 同じ理由で、`tracing::enabled!` はディレクトリごとの断りを映さない。答えは全部の項目の量の最大だけで決まる。実際の書き出しは、選んだ項目の量の設定に従う。

### Lua 側の SHIORI エントリとイベント配送

起動の 7 段目（[VM の構築とモジュール登録](execution-model.md#vm-の構築とモジュール登録)）で `require("pasta.shiori.entry")` が評価される。`pasta.shiori.entry` が `require` するモジュールの評価で、既定ハンドラが `REG` に登録される。どの既定ハンドラも `REG.X = REG.X or ハンドラ` の形で登録し、そのキーが空のときだけ入る。`scripts/main.lua`（先に実行される）でゴーストが登録したハンドラはそのまま残り、辞書の Lua ブロック（後に評価される）で登録したハンドラは既定ハンドラを置き換える。

```text
pasta.shiori.entry
  ├→ pasta.shiori.event
  │     ├→ pasta.shiori.event.register（REG）・pasta.shiori.res・pasta.shiori.act
  │     │  pasta.store・pasta.shiori.event.callback
  │     ├→ pasta.shiori.event.boot           REG.OnBoot を登録
  │     ├→ pasta.shiori.event.choice_select  REG.OnChoiceSelectEx を登録
  │     ├→ pasta.shiori.event.second_change  REG.OnSecondChange を登録
  │     │     └→ pasta.shiori.event.virtual_dispatcher
  │     └→ pasta.shiori.event.mouse          REG.OnMouseDoubleClick・REG.OnMouseMove を登録
  │           └→ pasta.store・pasta.shiori.event.virtual_dispatcher
  ├→ pasta.shiori.res
  ├→ pasta.shiori.event.kick
  └→ pasta.global                            close_ghost・ゴースト終了・pause_auto_talk・自動トーク停止・resume_auto_talk・自動トーク再開 を追加
```

既定ハンドラの上書きがどの時点で効くかは [既定ハンドラと上書き](../lua/shiori-events.md#既定ハンドラと上書き) を参照する。

`SHIORI` の関数は次のとおりである。`SHIORI = SHIORI or {}` で、既にあるグローバルの表に関数を足す。

| 関数 | 動作 |
| ---- | ---- |
| `SHIORI.load(hinst, load_dir)` | `xpcall` の中で何もせず `true` を返す。エラーなら `print` して `false` |
| `SHIORI.request(req)` | `STOP.clear()` で前の要求の控えを空にしてから、`xpcall` の中で `EVENT.fire(req)` を呼ぶ。エラーは、文字列なら最初の 1 行を、そうでなければ `nil` を理由にし、控え（`STOP.take()`・`STOP.take_handler()`）と一緒に `STOP.response` に渡す。表記の応答が返ればそれを、`nil` なら `RES.err` で 500 応答を返す（`nil` の理由は `"Unknown error"` になる。[止まるエラーの控えと応答](#止まるエラーの控えと応答)） |
| `SHIORI.kick(scene)` | `KICK.install(scene)` でキックを保留するだけで、シーンは動かさない |
| `SHIORI.unload()` | `xpcall` の中で何もしない |

`EVENT.fire(req)` の振り分けは次の順である。

1. `CALLBACK.try_route(req)` が応答を返せば、それを返す（待っているコールバックの再開）。`CLOCK.observe(req)` が時計の返事（OnPastaClock）を受け取れば 204 を返す。この 2 つで返した回は、鎖も `REG` の値も呼ばない。
2. 鎖（`REG:register` で重ねたハンドラ）の写しを `getmetatable(REG).handlers(req.id)` で 1 回だけ取り、新しく登録した順に呼ぶ。1 つ呼ぶたびに `act = SHIORI_ACT.new(STORE.actors, req)` を作り直して渡す。真の値を返したらそこで止め、その act と戻り値で 4 へ進む。`nil`・`false` なら次へ回る。写しを取った後の登録・外しは次のイベントから効く。作者から見た規則は [ハンドラを重ねる](../lua/shiori-events.md#ハンドラを重ねるregregister) が正である。
3. 全部見送れば（鎖が空でも）、新しい `act` で底を呼ぶ。底は表の生の値 `rawget(REG, req.id)` で、無ければ `EVENT.no_entry` である。底の戻り値は真偽で見ない。`EVENT.no_entry` は `SCENE.co_exec(act, req.id)` でイベント名のシーンを探し、見つかればそのコルーチンを、無ければ `nil` を返す（`pasta.scene` は循環を避けるため呼び出し時に `require` する）。
4. 戻り値がコルーチンなら `EVENT.drive` で再開し、その出力を `RES.ok` で応答にする（出力は接頭辞にかかわらず常に `Value`）。`SHIORI/` で始まる文字列なら応答全体として包まずに返し、それ以外の文字列なら `RES.ok` で包み、それ以外（鎖の `true`・数・表を含む）なら 204 を返す。コルーチンの扱いは [イベントからシーンへ](execution-model.md#イベントからシーンへ) で扱う。

既定ハンドラの中身は次のとおりである。アクター名の解決（`@pasta_config` の `spot` と `STORE.appearance.owners`）・候補の段・撫での判定の規則は [つつき・撫でのショートカット](../lua/shiori-events.md#つつき撫でのショートカット) が正である。

| ハンドラ | 動作 |
| -------- | ---- |
| `REG.OnBoot` | `SCENE.co_exec(act, act.req.id)`。シーン関数フォールバックと同じ |
| `REG.OnChoiceSelectEx` | まず `SCENE.co_exec(act, "OnChoiceSelectEx")`。無ければ探索範囲の親を決める。`act.req.reference[2]`（`\q` の 3 番目の引数。選択肢を出したグローバルシーンの登録名）が文字列で `SCENE.get_global_table` で引けるグローバルシーンならそれを、そうでなければ `STORE.last_global_scene` を親にする。`act.req.reference[1]`（選択 ID）を `SCENE.search(選択 ID, 親)` で探し、無ければ `SCENE.search(選択 ID, nil)` でグローバルシーンを探し、見つかった関数をコルーチンに包んで返す。`SCENE.co_exec` は親のグローバルシーンを渡せないため、検索を直接呼ぶ |
| `REG.OnMouseDoubleClick` | `act.req.reference[3]`（スコープ番号）からアクター名を、`act.req.reference[4]`（部位）を部位として、ショートカットの 1〜8 段の名前を上から `SCENE.co_exec_exact` で探し、最初に見つかったコルーチンを返す。無ければ `SCENE.co_exec(act, "OnMouseDoubleClick")`。抑止は見ない |
| `REG.OnMouseMove` | 撫での判定（`check_stroke`）を毎回回す。判定には部位・スコープ番号・なめらかな時計の時刻（`CLOCK.steady(act.req)` に `act.req.date.ns` を足した小数の秒）と、`virtual_dispatcher.is_blocked(act.req.status)` を渡す。撫でたと決めた回だけ、アクター名を引いて 1〜8 段を `SCENE.co_exec_exact` で探し、見つかればそのコルーチンを返す。それ以外は `SCENE.co_exec(act, "OnMouseMove")`。判定の状態はモジュールのローカル変数 1 つで、保存しない |
| `REG.OnSecondChange` | `CALLBACK.sweep(CLOCK.steady(act.req) or os.time())` が応答を返せばそれを返し（その回は仮想イベントを出さない）、そうでなければ `virtual_dispatcher.dispatch(act)` がコルーチンを返せばそれを返す。何も出さない回は、Status がブロック対象なら `nil`、そうでなければ `CLOCK.take_probe(act.req)`（予約された仮想時刻の読み取りのスクリプトか `nil`）を返す。`sweep` の応答は、期限切れの待機を再開した結果（出力の 200、止まるエラーの表記の 200、または表記を出さない設定での理由付きタイムアウトの 500）の全文であり、`EVENT.fire` が包まずに返す |

### 非同期トーク

SHIORI はベースウェアが問い合わせるだけの通信であり、シーンの途中で SSP のプロパティを読むには、要求を応答に載せ、値を後のリクエストで受け取るしかない。非同期トークはこれを 1 つのシーンの中の待ち合わせとして実現する。

```text
リクエスト 1（任意のイベント）
  シーン内 act:get_property(名前…)
    CALLBACK.stage_pending("OnPastaCallBack{N}", 期限, 理由)
    応答の Value に \![get,property,OnPastaCallBack{N},名前…] を載せて中断
リクエスト 2（SSP が発行する OnPastaCallBack{N}。値は Reference に入る）
  EVENT.fire → CALLBACK.try_route
    待っているコルーチンを pending から外し、EVENT.drive で Reference の配列を渡して再開し、
    その出力を応答にする
期限切れ（OnSecondChange のたびに確認）
  OnSecondChange の既定ハンドラ → CALLBACK.sweep
    期限切れの待機を集めて pending から外し、番号順に EVENT.drive で (nil, 理由) を渡して再開する
    最初に生まれた応答（出力の 200、表記の 200、または表記を出さない設定での理由付きタイムアウトの 500）をその回の応答にする
```

- アクターランタイムの上では、リクエスト 1 と 2 は別々の mailbox のメッセージとして順に処理される。待っているコルーチンは、その間 VM の `CALLBACK.pending` に保たれる。Rust 側はコルーチンを持たない。
- 再開したシーンは、通常のイベントと同じ再開の手順（`EVENT.drive`）を通る。待ち合わせの状態の遷移（`consume_staged`・`STORE.co_callback`・`set_co_scene` との関係）は [コールバック待ちとの関係](execution-model.md#コールバック待ちとの関係) で、掃引の手順は [期限切れの掃引](execution-model.md#期限切れの掃引callbacksweep) で扱う。利用者から見た挙動は [OnPastaCallBack](../lua/shiori-events.md#onpastacallbackコールバック応答) が正である。

### 止まるエラーの控えと応答

Lua のエラーでシーンやハンドラが止まったとき（止まるエラー）の表記は、`pasta.shiori.stop`（`STOP`）が作る。止まった場所の名前を作るため、エラーが `SHIORI.request` に届くまでに失われる act を、要求ごとに 1 つずつ控えておく。

- `EVENT.drive` は、コルーチンがエラーで終わると、`STOP.hold(co, err)` でそのコルーチンの act（`FAILURE.act_of(co)`）とエラーの値を控えてから `false` を返す。`EVENT.fire` は、`false` を受けたらそのエラーを投げ直す。
- `EVENT.fire` は、ハンドラ（鎖のハンドラ・`REG` の関数・`EVENT.no_entry`）を 1 つ呼ぶ直前ごとに、渡す act を `STOP.hold_handler(act)` で控え直す。シーンの外（ハンドラの中）で止まったときは、止まったハンドラの act に積んだ台詞の後に表記が出る。見送った鎖のハンドラの act は控えが上書きされるので、その台詞は表記に混ざらない。
- `SHIORI.request` は、要求の始めに `STOP.clear()` で両方の控えを空にする。エラーを受けたら `STOP.take()`・`STOP.take_handler()` で控えを取り出し、理由の行（エラー文の最初の行）・要求の ID・ID が `On` で始まるかと一緒に `STOP.response` に渡す。
- `STOP.response` は、控えた act（シーンの失敗の act、無ければハンドラの act）から止まった場所の名前を作り、エラー文の場所を直し、ログに警告を 1 行出す（`FAILURE.notation`）。表記を出す設定で、ID が `On` で始まる要求なら、act の末尾に表記を `raw_script` として積んで組み立てた値を `Value` に、受けた理由を `X-Error-Reason` にした `RES.ok`（`200 OK`）の全文を返す。表記を出さない設定・ID が `On` で始まらない・組み立てに失敗したときは `nil` を返し、`SHIORI.request` はこれまでどおり `RES.err` の 500 を返す。
- 期限切れの掃引（`CALLBACK.sweep`）は、待機ごとの `time_out` で `EVENT.drive` の直後に、成否にかかわらず `STOP.take()` で控えを取り出す（同じ要求の後の止まるエラーに古い控えを混ぜない）。理由付きの待機がエラーで終わると、`timeout_message` を理由と表記の対象にして `STOP.response` に渡し（`【時間切れ：…】`）、`nil` なら `RES.err(理由)` の 500 を候補にする（警告は再開の前に出した 1 行だけで、`STOP.response` は警告を出さない）。理由の無い待機がエラーで終わると、イベント名を前に付けた警告を出し、スクリプトのエラーの表記の応答を候補にする（表記を出さない設定では候補を作らない）。

表記の形・止まった場所の示し方・出る応答と設定ごとの違いは、[止まるエラーの表記](../reference/runtime-failures.md#止まるエラーの表記) が正である。

### 仮想イベントディスパッチャ（OnTalk・OnHour）

OnTalk と OnHour は、`REG.OnSecondChange` の既定ハンドラが毎回呼ぶ `pasta.shiori.event.virtual_dispatcher` が発行する。状態はモジュールのローカル変数 `next_hour_unix`（次の正時）・`seen_epoch`（前に見た時刻の飛びの通し番号）・`next_talk_time`（次のトーク時刻）だけで、`next_hour_unix`・`next_talk_time` は 0 が「未設定」を表す。状態は VM と同じ寿命（`load` から `unload` まで）で、保存されない。

```text
dispatch(act)
 1. act.req.date が無ければ nil
 2. force = STORE.kick_force。真なら false に戻す（1 回だけ使う）
 3. force でなく is_blocked(act.req.status) なら nil
 4. KICK.try_dispatch(act) がコルーチンを返せばそれを返す
 5. STORE.auto_talk_paused が真なら nil
 6. check_hour(act) がコルーチンを返せばそれを返す
 7. check_talk(act) を返す
```

利用者から見た判定の順・ブロック対象のキーワードと照合の仕方・OnHour のシーン候補の順・トーク間隔の決まり方・テスト用の関数は、[仮想ディスパッチャ](../lua/shiori-events.md#仮想ディスパッチャ)（[ブロック対象 Status キーワード](../lua/shiori-events.md#ブロック対象-status-キーワード)・[OnHour](../lua/shiori-events.md#onhour)・[pasta.toml 設定](../lua/shiori-events.md#pastatoml-設定)・[テスト用関数](../lua/shiori-events.md#テスト用関数)）が正である。ここではそれを実現する仕組みだけを書く。

- 2 の `STORE.kick_force` は `KICK.install` が立てる。`dispatch` の入口が読んだ時点で消費するため、ブロックの判定を飛ばすのはキックの直後の 1 回だけである。4 はキックの保留を起動する段で、中身は [デバッグ基盤とシーンキック](debug.md) で扱う。
- `check_hour` の現在時刻は `CLOCK.now(act.req)`（仮想時刻。下の「仮想時刻の時計」）である。`next_hour_unix` が 0 なら次の正時（`unix - unix % 3600 + 3600`）を記録するだけで終わる。`CLOCK.epoch()` が `seen_epoch` と違えば（時刻の飛びがあれば）、`seen_epoch` を更新し、現在時刻の 70 秒前（ローカル定数 `REBASE_GRACE`）から同じ式で次の正時を決め直す。現在時刻がそれに達していれば、同じ式で次の正時を記録し直し、`act:transfer_date_to_var()` で日時変数を転記してからシーンを探す。シーン名の `{HH}` は `CLOCK.date(act.req).hour` から作る。
- `check_talk` は、`next_talk_time` が 0 なら次のトーク時刻を決めるだけで終わる。トーク時刻は `CLOCK.steady(act.req)`（飛びを差し引いた時計）で数え、仮想時刻の設定し直しに左右されない。正時までの残りを `hour_margin` と比べる判定は、`check_hour` が記録した `next_hour_unix` と `CLOCK.now(act.req)` の差を使う（仮想時刻で数える）。この判定で発行を見送るときは `next_talk_time` を変えないため、次の秒にまた判定する。発行するときは、シーンを探す前に `next_talk_time` を決め直す。
- 次のトーク時刻は「現在時刻 + `math.random(最小間隔, 最大間隔)`」で決める。間隔は、決めるたびにローカル関数 `get_config` が `pasta.save` と `@pasta_config` を読み直して求める（キャッシュしない）。`hour_margin` は `[ghost].hour_margin`（無ければ 30）を、切り捨てや下限の補正をせずにそのまま使う。`[ghost]` の既定値の補完は [設定読込](loader.md#設定読込) で扱う。
- ブロック中（3 で終わる間）は `check_hour`・`check_talk` を呼ばないため、`next_hour_unix`・`next_talk_time` は進まない。そのため、ブロックが解けた最初の判定で、過ぎていた正時やトーク時刻の条件が成り立つ。
- 5 の `STORE.auto_talk_paused` は、`GLOBAL.pause_auto_talk`／`resume_auto_talk`（`pasta/shiori/entry.lua`）が書く（[自動トークを止めている間](../lua/shiori-events.md#自動トークを止めている間)）。止めている間もブロック中と同じく時計は進まないが、再開の関数が公開の関数 `restart_timers(act)` を 1 回呼び、時計を決め直す。`next_talk_time` を 0 に戻し、`CLOCK.now(act.req)` が取れれば次の正時を求めて、今の `next_hour_unix` より後のとき、または `CLOCK.epoch()` が `seen_epoch` と違うときだけ置き換えて `seen_epoch` を記録する（取れなければ `next_hour_unix` を 0 にする）。そのため、ブロックと違い、止めている間に過ぎた正時は再開の後にも発行されない。`set_co_scene` を通らないので、中断中の継続は捨てずに残る。
- 返したコルーチンは `EVENT.fire` が `EVENT.drive` で再開し、続きの決まりに従って `STORE.co_scene` を更新する。継続を再開するのは OnTalk だけである（`check_talk` が `STORE.co_scene` を返し、種類は "resume"）。OnHour のシーンは `SCENE.co_exec` で探した印の無いコルーチンなので、中断中の継続があればふつうの割り込み（"interrupt"）になり、台詞を出せば継続を捨て、出さなければ残す。キックのシーンは `KICK.try_dispatch` が返すコルーチンに `STORE.co_preempt` の印が付いているので "preempt" になり、台詞の有無によらず継続を捨てる（[継続トークと co_scene の更新](execution-model.md#継続トークチェイントークと-co_scene-の更新)）。
- 仮想イベントは `REG` を通らない。OnHour・OnTalk で発行するシーンはどれも `SCENE.co_exec` で探す（テスト用の差し替え `scene_executor` が設定されていればそれを使う）。4 のキックは検索を使わず、シーン表のキーとの完全一致でシーンを引く（[キックの保留と起動](debug.md#キックの保留と起動kicklua)）。

#### 仮想時刻の時計（pasta.shiori.clock）

SSP の仮想時刻は SHIORI のリクエストのヘッダに載らない。SSP によっては（2.9.18 の `20261017-5` ビルドで確認）SSP のプロセスの中の OS の時計そのものが仮想時刻になり、`act.req.date` も仮想時刻になるが、そうでない SSP のために `pasta.shiori.clock`（以下 `CLOCK`）が OS の時計とのずれを持つ。どちらの SSP でも Apply・Reset は「今」の飛びになるため、`CLOCK` は飛びも数える。利用者から見た読み取りの手順・返事の扱い・ずれの使われ方は [OnPastaClock](../lua/shiori-events.md#onpastaclock仮想時刻の読み取り) が正である。`CLOCK` は `pasta.shiori.event`・`pasta.shiori.act`・`pasta.shiori.event.second_change`・`pasta.shiori.event.virtual_dispatcher`・`pasta.shiori.event.mouse` から読まれるため、循環参照を避けて `pasta.shiori.event.*`・`pasta.shiori.act`・`pasta.store` を `require` しない。

状態はモジュールのローカル変数 6 つだけである。

| 変数 | 型 | 初期値 | 意味 |
| ---- | -- | ------ | ---- |
| `_offset` | 整数（秒） | `0` | 仮想時刻 − OS の時計。0 は仮想時刻なし |
| `_probe_armed` | 真偽値 | `false` | 仮想時刻の読み取りを予約しているか |
| `_prev_now` | 整数（秒）か `nil` | `nil` | 前の OnSecondChange の `now` |
| `_epoch` | 整数 | `0` | 時刻の飛びの通し番号 |
| `_jump_total` | 整数（秒） | `0` | 飛びの合計（ふだんの 1 秒分を除く） |
| `_resumed` | 真偽値 | `false` | `OnSysResume` の後で、次の 1 回の差を飛びとみなさないか |
| `_suspended` | 真偽値 | `false` | `OnSysSuspend` の後で、次の 1 回の飛びを飛びとみなさないか（飛ばない差では下ろさない） |

状態は保存しない。VM と同じ寿命（`load` から `unload` まで）で、次の `load` で初期値から始まる。

```text
EVENT.fire(req)
 1. CALLBACK.try_route(req)          待っているコールバックの返事ならその応答
 2. CLOCK.observe(req)               true（OnPastaClock の返事を受け取った）なら 204
 3. 鎖（REG:register）を新しい順に   真の値を返したものの応答
 4. 全部見送れば REG[req.id] を呼ぶ  rawget。OnSecondChange では既定ハンドラ（下）

REG.OnSecondChange（既定。鎖の底）
 1. CALLBACK.sweep                   タイムアウトの応答があればそれ
 2. dispatcher.dispatch(act)         シーンがあればそれ
 3. is_blocked(act.req.status) なら nil
 4. CLOCK.take_probe(act.req)        予約があれば読み取りのスクリプト
```

- 合図の判定（`observe`）: 現在時刻 `CLOCK.now(req)` で、`OnMinuteChange` は `now % 60` が 2 秒を超え 58 秒未満なら、`OnHourTimeSignal` は `now % 3600` が 60 秒を超え 3595 秒未満なら、見込みの時機から外れたとして `_probe_armed` を立てる。毎時の合図は遅れを 60 秒、早まりを 5 秒まで許す（トークできるまで待つ遅れを許すため）。幅はローカル定数 `MINUTE_TOLERANCE`・`HOUR_LATE_TOLERANCE`・`HOUR_EARLY_TOLERANCE` である。
- 読み取り（`take_probe`）: 予約があれば消して `\![get,property,OnPastaClock,system.year,system.month,system.day,system.hour,system.minute,system.second]\e` を返す。呼ぶのは OnSecondChange の既定ハンドラだけで、何も発行せず Status がブロック対象でない回に限る。予約が無い回は何も返さないため、毎秒のバルーンには影響しない。
- 返事の受け取り: `id` が `OnPastaClock` で `SecurityLevel` が `external` でなければ、`observe` が返事として消費して true を返す（`REG` へ進まない）。Reference0〜5 を `tonumber` で整数に読み、`os.time`（地方時）で Unix 時刻にし、`act.req.date.unix` との差をずれにする。差の絶対値が 2 秒以下（ローカル定数 `OFFSET_DEADBAND`）なら 0 にする。値が読めなければずれを 0 に戻し、警告ログを出す。ずれが変わったときはデバッグログを出す。
- `X-Pasta-Time`: ヘッダのあるリクエストでは、合図を判定せず、返事を受け取っても無視し、読み取りも返さない。`now` は `req.date.unix` を、`date` は `req.date` をそのまま返す。
- 読み出し: `now(req)` は `req.date.unix + _offset`、`date(req)` はずれを足した時刻から `os.date("*t")` で作った新しい日付の表（`act.req.date` と同じ形）を返す。ずれが 0 なら `req.date` をそのまま返し、`act.req.date` は書き換えない。日時変数の転記（`pasta.shiori.act` の `transfer_date_to_var`）は `CLOCK.date` を読む。`offset()` は `_offset` を返す。
- 時刻の飛び: `observe` は OnSecondChange のたびに `d = now − _prev_now` を求め、`d` が −2 秒より小さいか 90 秒より大きければ（ローカル定数 `JUMP_BACK`・`JUMP_FORWARD`）飛びとして `_epoch` を 1 増やし、`_jump_total` に `d − 1` を足す。`_resumed` か `_suspended` が立っていれば数えずに `_suspended` を下ろす。飛ばない差で `_resumed` が立っていれば `_suspended` も下ろす（スリープせずに戻った場合に印を残さない）。どの場合も最後に `_resumed` を下ろして `_prev_now` を `now` にする。`OnSysSuspend` を見ると `_suspended` を、`OnSysResume` を見ると `_resumed` を立てる（スリープ明けは飛びにしない）。SSP 2.9.18 では、`OnSysSuspend` は復帰してから最初の OnSecondChange より前に届き、`OnSysResume` は最初の OnSecondChange より数秒遅れて届く（2026-10-18 実機）ので、両方を見る。`X-Pasta-Time` 付きのリクエストは飛びを見ず、`_prev_now` も変えない。ずれが変わったとき（読み取りの返事）も、次の OnSecondChange で `now` が飛ぶので同じ判定に入る。
- なめらかな時計: `steady(req)` は `now(req) − _jump_total`（`X-Pasta-Time` があれば `req.date.unix`）を返す。おしゃべりの間隔（`check_talk`）、`act:get_property` の期限と `CALLBACK.sweep` に渡す今（`req.date` が無ければ `os.time()`）、撫での判定の時刻（`pasta.shiori.event.mouse`）はこれで数える。`epoch()` は `_epoch` を返し、ディスパッチャが `seen_epoch` と比べる。飛んでから次の OnSecondChange までの 1 秒足らずの間に始まった期限や撫では、補正前の時計で数える。
- 仮想時刻を使わずに OS の時計を手で 90 秒より進めたか 2 秒より戻したときも、飛びになる（10 時 30 分から 9 時 30 分へ戻すと 10 時に時報が出る。10 時 30 分から 12 時 10 分へ進めると 13 時まで時報が出ず、おしゃべりは残りの間隔を待つ）。
- 見落とし: ずれの変化の秒の端数が毎分の許容幅（±2 秒）に入ると、毎分の合図では気づかない。そのときは仮想の正時の `OnHourTimeSignal` で気づき、決め直しの猶予（70 秒）で時報が数秒遅れて 1 回出る。仮想時刻を戻した（Reset）のを見落とすと、古いずれが最大 1 時間残り、実際の正時の `OnHourTimeSignal` で直る。ずれの変化が 1 時間の倍数に近いと気づけない。

### 応答文字列の出どころ

応答文字列を作る場所は 5 つあり、ヘッダの組が異なる。

| 作る場所 | 応答 | ヘッダ | 使われる場合 |
| -------- | ---- | ------ | ------------ |
| `RES`（Lua） | 200・204・311・312・400・500 | `Charset`・`Sender`・`SecurityLevel` と追加分。500 の理由と、止まるエラーの表記の 200 の理由は `X-Error-Reason` | `EVENT.fire` と `SHIORI.request` のすべての応答 |
| `marshaling::default_204`（Rust） | 204 | `Charset`・`Sender`・`SecurityLevel` | NOTIFY、アクターが居ない、メソッドを判定できない、GET の応答が来ない、panic、UTF-8 として読めない |
| `PastaShiori::default_204_response`（Rust） | 204 | `Charset`・`Sender` | `SHIORI.request` が定義されていない |
| `MyError::to_shiori_response`（Rust） | 500 | `Charset`・`X-ERROR-REASON` | ロードの失敗・未ロード・`SHIORI.request` の呼び出しの失敗 |
| `MyError::to_shiori_400_response`（Rust） | 400 | `Charset`・`X-ERROR-REASON` | リクエストの解析の失敗（`X-Pasta-Time` の誤りを含む） |

- `default_204` のバイト列は、`RES.no_content()` が既定の `RES.env` で作る応答と同じである。
- Rust の 500・400 の理由は `single_line` で 1 行にする。`stack traceback:` の行から後を捨て、残りの行を前後の空白を除いて半角スペースでつなぐ。改行を含まない理由はそのまま使う。

### presentation マーカーとレンダラ注入

`crates/pasta_lua/src/presentation/` は、エンジンの出力をさくらスクリプトのような宿主の形式に依存しない列として表すための型である。

- `PresentationMarker` は `Talk`・`ActorSwitch`・`Wait`・`Choice` と、それ以外を表す `Extension` を持つ（`#[non_exhaustive]`）。`classify` は Lua のトークンの `type` を分類し、`talk`・`sakura_script` を `Talk`、`actor` を `ActorSwitch`、`wait` を `Wait`、`choice` を `Choice`、それ以外を `Extension`（`kind = "passthrough:型名"`）にする。値は `TokenFields` の実装から引く。
- `RenderBoundary` はマーカー列を宿主の出力へ畳み込む境界である。対応するマーカーは `render_known` が文字列を返し、対応しないマーカー（`None`）は `on_unknown` の既定動作（空文字列）で吸収する。`render_stream` は列を順に連結する。

`@pasta_sakura_script` の登録は `RendererInjection` を通る。`RendererInjection` は `fn(&Lua, Option<&TalkConfig>) -> LuaResult<Table>` を 1 つ持つ薄い包みで、`register_sakura_script_module` がそれを呼んで返った表を `package.loaded["@pasta_sakura_script"]` に置く。既定（`RendererInjection::default`）は `default_sakura_renderer`、すなわち `crate::sakura_script::register` である。

現行の実装では、次のとおり出荷経路のバイト列はマーカーを経由しない。

- VM の構築（`from_loader_with_scene_dic`）は常に `RendererInjection::default()` を渡す。`pasta_shiori` から別のレンダラを渡す経路は無い。
- 応答のさくらスクリプトは、VM の中で `pasta.shiori.sakura_builder` と `@pasta_sakura_script` が作る（[トーク出力とアピアランス](talk-output.md)）。Lua のトークを `PresentationMarker` に変換する処理は出荷経路に無く、`classify` と `SakuraRenderBoundary`（`Talk` の本文だけを返す）を使うのはテストだけである。

### DLL のビルド構成

`crates/pasta_shiori/Cargo.toml` はライブラリ名を `pasta`、`crate-type` を `cdylib` と `rlib` にしている。`cdylib` が Windows では pasta.dll、Linux では `libpasta.so`、macOS では `libpasta.dylib` になり、`rlib` は統合テストが Rust から使うためのものである（`crates/pasta_shiori/src/lib.rs` は `shiori_get_api` と、Windows では `load`・`loadu`・`request`・`unload` を再エクスポートし、テストが出荷シンボルそのものを呼べるようにしている）。

- `crates/pasta_shiori/src/windows.rs` と `crates/pasta_shiori/src/util/hglobal/` は `#[cfg(windows)]` で、Windows 以外のビルドは旧来の入口を持たない。
- `crates/pasta_shiori/src/v3.rs` と `crates/pasta_shiori/src/actor/` は全部の OS でコンパイルする。Linux・macOS の共有ライブラリが書き出すのは `shiori_get_api` だけである。
- 依存のうち Windows だけのものは `windows-sys` で、`[target.'cfg(windows)'.dependencies]` に置く。版 3 の ABI は `shiori4_abi`、旧来の入口の置き場は `arc-swap`、mailbox は `flume` が担う。アクターの実行に OS ごとの実行器は使わない。
- CI（`.github/workflows/build.yml`）は、Windows のほかに Linux・macOS でリリースビルドとワークスペースのテストを回し、3 つの OS で `shiori4-testhost` による版 3 の 1 往復を確かめる。

リリースビルドの設定はルートの [Cargo.toml](https://github.com/ekicyou/pasta/blob/main/Cargo.toml) の `[profile.release]` にあり、ワークスペースの全クレートに効く。pasta.dll の大きさを小さくするための設定である。

| キー | 値 | 効果 |
| ---- | -- | ---- |
| `opt-level` | `"z"` | 大きさを優先して最適化する |
| `lto` | `true` | クレートをまたいでリンク時に最適化する |
| `codegen-units` | `1` | コード生成を分割せず、最適化の範囲を広げる |
| `panic` | `"abort"` | panic で巻き戻さず中止する。巻き戻しの表を持たない |
| `strip` | `true` | シンボルを取り除く |

`panic = "abort"` のため、リリースビルドでは panic がホストのプロセスごと中止させる。旧来の入口の `catch_unwind` と、版 3 の入口がパニックを `SHIORI_E_PANIC` で返す `shiori4_abi` の捕捉が効くのは、巻き戻すプロファイル（dev・test）だけである。リリースでは、どちらの入口でもパニックでプロセスが終わる。この扱いは変えない。

ルートの `.cargo/config.toml` は、`i686-pc-windows-msvc` と `x86_64-pc-windows-msvc` の両方に `-Ctarget-feature=+crt-static` を与え、MSVC の CRT を静的にリンクする。pasta.dll は VC++ 再頒布可能パッケージ（`vcruntime140.dll` など）に依存しない。Cargo は `.cargo/config.toml` を作業ディレクトリから上へ探すため、ルートから `cargo build -p pasta_shiori` を実行する CI でも効くよう、クレートの下ではなくルートに置いている。

`crates/pasta_shiori/build.rs` は、`CARGO_PKG_VERSION` などから `VERSIONINFO` の `.rc` を `OUT_DIR` に生成し、`embed_resource::compile_for_cdylib` で pasta.dll にだけ埋め込む。数値のバージョンは「メジャー,マイナー,パッチ,0」で、文字列のバージョンは `CARGO_PKG_VERSION` そのものである。Windows で `rc.exe` が使えず埋め込みに失敗するとビルドを失敗させる（`manifest_required().unwrap()`）。Windows 以外では何もしない。

## 境界の受け渡し

| 境界 | 渡す側 → 受ける側 | 渡すもの | 所有 |
| ---- | ----------------- | -------- | ---- |
| ホスト → pasta.dll（`loadu`・`load`） | ホスト → `load_entry` | 設置パスの HGLOBAL と長さ | 受け取った時点で DLL が所有し、どの経路でも解放する |
| ホスト ↔ pasta.dll（`request`） | ホスト → `request` → ホスト | リクエストの HGLOBAL と長さ（入力）、応答の HGLOBAL と長さ（戻り値と `len`） | 入力は DLL が解放する。応答（`GMEM_FIXED`）はホストが解放する |
| ホスト ↔ 版 3 の入口 | ホスト → `shiori4_abi` → `PastaV3` | 設置パス・要求の UTF-8 の文字列、応答の全文と状態の数 | インスタンスのハンドルはホストが持ち、`unload` で返す。応答はホストの `copy_from_slice` で写す |
| 入口のスレッド → アクタースレッド | `ActorInstance` の `get`・`notify`・`kick`・後始末 → mailbox | `ActorMsg`（生のリクエスト文字列と `seq`、応答用・完了通知用の送信端、シーン名） | メッセージは move される。VM はアクタースレッドから出ない。mailbox の送信端は `ActorInstance` が、受信端はアクタースレッドが持つ。`ActorInstance` は旧来の入口では `LEGACY` が、版 3 では `PastaV3` が持つ |
| アクタースレッド → 入口のスレッド | 応答用チャネル・起動の報告用チャネル | `Reply::Value(応答文字列)`、`(ThreadId, bool, Option<SocketAddr>)` | 文字列と `Send` な値だけが渡る |
| `pasta_shiori` → `pasta_lua`（構築） | `PastaShiori::load` → `PastaLoader::load_with_config` | 設置パスと `RuntimeConfig`（`KickSink` を載せる） | `PastaShiori` が `PastaLuaRuntime`・キャッシュした Lua 関数・占有の札を所有する |
| `pasta_lua` → `pasta_shiori`（キック） | デバッグ通信のスレッド → `KickSink` | `KickRequest { scene }` | クロージャはそのインスタンスの mailbox の送信端の弱い参照だけを持ち、`try_send` で投函する。インスタンスが片付いた後や送れないときは捨てる。DAP がつながっているインスタンスにだけ届く |
| Rust → Lua（リクエスト） | `PastaShiori` → `SHIORI.request` | `parse_request` が作った表 | 表は VM が所有し、ACT の `req` として残る |
| Lua → Rust（応答） | `SHIORI.request` → `PastaShiori` | SHIORI 応答の文字列の全体 | 文字列の複製が Rust に渡る |
| `pasta_lua` のコア → レンダラ | `register_sakura_script_module` → `RendererInjection` | VM と `TalkConfig` | 返った表は `package.loaded` に置かれて VM が所有する |

## 不変条件と制約

- VM（`PastaShiori`・`PastaLuaRuntime`）はアクタースレッドで作られ、使われ、破棄される。`PastaShiori` に `unsafe impl Send`・`Sync` は無く、スレッドをまたぐのは `Send` な値と `Send + Sync` な `ActorInstance` だけである。
- インスタンス 1 つにアクタースレッド 1 本・mailbox 1 本・VM 1 つである。mailbox を消費するのはアクタースレッドの同期 `recv()` のループ 1 つだけで、`select!` は使わない。新しい種類のメッセージは、別のチャネルではなく `ActorMsg` の新しい variant として足す。これにより VM への同時アクセスは構造上起こらず、メッセージは投入順に処理される。NOTIFY はすぐ 204 を返すが、その処理が終わるまで後続の GET は処理されない。
- インスタンスの状態は、ほかのインスタンスへ漏れない。プロセスに 1 つ残す状態は [プロセスに 1 つ残す状態](#プロセスに-1-つ残す状態) に挙げたものだけである。
- 生きているインスタンスどうしは、同じゴーストのディレクトリを読まない（[ディレクトリの占有](#ディレクトリの占有)）。
- 旧来の入口の送信の経路（`marshal_request`）は `LEGACY` の `load_full` だけでインスタンスを得て、ロックを取らない。kick はインスタンスごとの送り先（mailbox の送信端の弱い参照）を通り、`LEGACY` を経由しない。`store`・`swap` するのは起動と終了のときだけである。
- ホストのスレッドを際限なく待たせない。GET の応答待ちと終了の完了待ちはそれぞれ 5 秒で打ち切る。応答を返せない場合はすべて `default_204` を返す。終了の完了待ちの後の join は、ack が届いたときだけ行うので止まらない。
- リクエストの処理のエラー（ロードの失敗・未ロード・`SHIORI.request` の失敗）は 500 応答になり、設定にかかわらずバルーンに何も出ない。204 は応答を返せなかった場合だけの安全網である。`SHIORI.request` の `xpcall` の中で止まったエラーは、表記を出す設定で ID が `On` で始まる要求なら表記の 200、それ以外は `RES.err` の 500 になる。NOTIFY ではどちらもホストへ届かず、`default_204` が返る（[止まるエラーの控えと応答](#止まるエラーの控えと応答)）。
- GET が 5 秒を過ぎて 204 を返しても、アクタースレッドの処理は止めない（LuaJIT の実行は途中で止められない）。その処理は最後まで進み、応答は受け取り手が無いため捨てられる。VM の状態はその処理の結果どおりに更新される。
- 入力の HGLOBAL は DLL が必ず解放し、応答の HGLOBAL はホストが解放する。`GlobalAlloc` の失敗は null のまま使わず、null の応答と長さ 0 にする。
- 旧来の入口は `catch_unwind` で、版 3 の入口は `shiori4_abi` の捕捉（`SHIORI_E_PANIC`）で panic をホストへ伝えない。ただし、これが効くのは巻き戻すビルドだけで、リリースでは `panic = "abort"` のためプロセスが終わる。そのため正常な経路では panic を起こさない書き方をする（`crates/pasta_shiori/src/actor/marshaling.rs` の正常経路に `unwrap`・`expect` が無いことをテストが確かめる。リクエストの解析は再帰しない）。
- `DllMain` ではスレッドを起こさず、スレッドの終わりも待たない。アクタースレッドは `load` を起点に起こす。
- 終了処理は何度呼んでも安全である（`unload` の二重呼び出し、`unload` と `DllMain` の detach の重なり）。ack が届けばスレッドを join し（`DllMain` の道だけ手放す）、届かなければ手放す。完了の通知は VM の破棄と mailbox の受信側の破棄が済んでから送る。
- ロガーの登録を外すのは、ランタイムを手放す処理（`release_runtime`）の最後である。完了の通知の後に出たログは捨てる。
- `default_204` のバイト列は `RES.no_content()` と同じであり、FFI 入口の応答のバイト列はテスト `crates/pasta_shiori/tests/byte_invariant_test.rs` が固定している。版 3 の GET が旧来の入口と同じバイト列を返すことは `crates/pasta_shiori/tests/shiori4_v3_entry_test.rs` が、2 つのインスタンスが混ざらないことは `crates/pasta_shiori/tests/multi_instance_test.rs` が、後始末の決まり（join・手放し）は `crates/pasta_shiori/tests/actor_instance_test.rs` が確かめる。再読み込みを繰り返しても OS のハンドルが増えないことは `crates/pasta_shiori/tests/actor_reload_leak_test.rs` が確かめる。
- 仮想イベントディスパッチャの状態は保存されず、再読み込みのたびに初めからになる。
- 静的 CRT の設定は、ワークスペースのルートから実行したビルドにだけ効く（ルートの `.cargo/config.toml`）。

## ソースの所在

- `crates/pasta_shiori/src/`
- `crates/pasta_shiori/build.rs`
- `crates/pasta_shiori/Cargo.toml`
- `crates/pasta_lua/src/presentation/`
- `crates/pasta_lua/src/runtime/renderer_injection.rs`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/`
- `.cargo/config.toml`

リリースプロファイルはルートの [Cargo.toml](https://github.com/ekicyou/pasta/blob/main/Cargo.toml) にある。テストは `crates/pasta_shiori/tests/`・`crates/pasta_shiori/src/windows_tests.rs`・`crates/pasta_shiori/src/shiori_lifecycle_tests.rs`・`crates/pasta_shiori/src/shiori_request_tests.rs` にある。

本文で参照した主なファイルは次のとおりである。旧来の入口は `crates/pasta_shiori/src/windows.rs`、版 3 の入口は `crates/pasta_shiori/src/v3.rs`、芯の型は `crates/pasta_shiori/src/actor/instance.rs`、アクターランタイムは `crates/pasta_shiori/src/actor/`、ディレクトリの占有とログの量の判定は `crates/pasta_lua/src/logging/registry.rs`・`crates/pasta_lua/src/logging/tracing_init.rs`、リクエストの解析は `crates/pasta_shiori/src/lua_request.rs` と `crates/pasta_shiori/src/util/parsers/req_parser.pest`、Lua 側の振り分けは `crates/pasta_lua/pasta_scripts/pasta/shiori/entry.lua`・`crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua`、仮想イベントは `crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` にある。`@pasta_sakura_script` の登録を呼ぶ側は `crates/pasta_lua/src/runtime/module_registry.rs`・`crates/pasta_lua/src/runtime/factory.rs` である。

## 経緯

- [shiori-entry](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/shiori-entry) — Lua 側の SHIORI エントリ
- [shiori-event-module](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/shiori-event-module) — `EVENT.fire` と `REG` によるイベントの振り分け
- [shiori-res-module](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/shiori-res-module) — 応答文字列を作る `RES`
- [alpha01-shiori-alpha-events](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/alpha01-shiori-alpha-events) — 主要イベントのハンドラの登録
- [alpha02-virtual-event-dispatcher](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/alpha02-virtual-event-dispatcher) — 仮想イベントディスパッチャ
- [onhour-fallback-chain](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/onhour-fallback-chain) — OnHour のシーン候補の順
- [onhour-date-var-transfer](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/onhour-date-var-transfer) — OnHour での日時変数の転記
- [suppress-ontalk-on-choosing](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/suppress-ontalk-on-choosing)・[ontalk-block-condition](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/ontalk-block-condition) — Status による仮想イベントの抑止
- [talk-frequency-persistence](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/talk-frequency-persistence) — トーク頻度の `pasta.save` への保存と読み直し
- [choice-definition-dsl](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/choice-definition-dsl) — OnChoiceSelectEx の既定ハンドラ
- [shiori-async-talk](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/shiori-async-talk) — 非同期トーク（`get_property` のコールバック）
- [audit-pasta-shiori](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/audit-pasta-shiori) — FFI 境界の安全性の点検（HGLOBAL の解放・確保の失敗・反復の解析）
- [pasta-actor-feasibility](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-actor-feasibility)・[pasta-actor-runtime](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-actor-runtime) — アクターランタイム、presentation マーカーとレンダラ注入
- [lua-require-robustness](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/lua-require-robustness) — `loadu`、処理のエラーを 500 応答で返す契約
- [pasta-scene-kick](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-scene-kick) — `ActorMsg::Kick` と `SHIORI.kick`
- [pasta-toml-logging-consistency](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-toml-logging-consistency) — ランタイムの解放を `release_runtime` にまとめ、ロガーの登録解除を最後にする
- [touch-poke-shortcuts](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/touch-poke-shortcuts) — OnMouseDoubleClick・OnMouseMove の既定ハンドラと、既定ハンドラを上書きしない登録（`REG.X = REG.X or …`）

---

> 【お辞儀】呼ばれたら応える――ただそれだけのことに、これほどの備えを欠かさない SHIORI 層には、頭が下がりますわ。次は、応答の中身、トーク出力とアピアランスへ参りましょう。

> 【アンソニー】NOTIFY にはすぐ 204 を返しても、片付くまで次の GET はお待たせする――律儀さも筋金入りでございます。
