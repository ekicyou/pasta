# SHIORI 層

ごきげんよう。ベースウェアからの呼びかけを受け止め、ゴーストの言葉を返す――それが SHIORI 層の務めですわ。
FFI の境界からアクター、イベント配送、DLL のビルド構成まで、わたくしがきっちり捌いて差し上げます。さあ、参りましょう。

---

この章では、pasta.dll としてベースウェアと通信する SHIORI 層を扱う。利用者から見たイベントとハンドラは [SHIORI イベントとハンドラ](../lua/shiori-events.md) が正である。

## 目的と責務

SHIORI 層は、ベースウェア（SSP など）が呼ぶ DLL の関数でリクエストを受け取り、Lua VM を所有する専用スレッドへ渡し、Lua 側でイベントを振り分けて作った応答を返す。

この章が責務を持つのは次の事項である。

- FFI 境界: エクスポートする関数（`DllMain`・`loadu`・`load`・`request`・`unload`）、HGLOBAL の所有と解放、設置パスとリクエストの文字コード、`catch_unwind` による panic の封じ込め
- リクエスト文字列の解析と、Lua に渡すリクエストの表の組み立て
- アクターランタイム: VM を専用スレッドに固定し、mailbox（単一のキュー）経由でだけ触る仕組み。GET と NOTIFY の受け渡し（marshaling）、起動と再読み込み、終了処理（teardown）
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
ホストのスレッド（FFI 入口）                    アクタースレッド "pasta-actor"
──────────────────────────                      ───────────────────────────────
loadu / load ── spawn_actor ──────────────────→ thread::spawn → block_on(async …)
     │           （構築の完了を待つ）             PastaShiori::load（VM を構築）
     │      ←── (スレッド ID, 成否, DAP アドレス) ─┘
     │           MAILBOX.store(Sender)            while let Ok(msg) = rx.recv_async().await
request ── marshal_request
     │      determine_method（pest で GET/NOTIFY を判定）
     │      GET:    try_send(Get{req, reply}) ───→ shiori.request(生文字列)
     │              recv_timeout(5 秒) ←────────── reply.send(Reply::Value(応答))
     │      NOTIFY: try_send(Notify{req}) → 即 204 → shiori.request(生文字列)（応答は捨てる）
unload ── teardown_actor
            MAILBOX.swap(None)
            send(Stop{done}) ───────────────────→ ループを抜ける → drop(PastaShiori) → drop(rx)
            recv_timeout(5 秒) ←────────────────── done.send(())
            ActorThread::detach（join しない）

デバッグ通信のスレッド ── KickSink → try_send(Kick{scene}) → shiori.kick(scene)
```

VM（`PastaLuaRuntime` を内包する `PastaShiori`）は `!Send` であり、アクタースレッドで作られ、そこで使われ、そこで破棄される。スレッドをまたぐのは、生のリクエスト文字列・応答文字列・シーン名・応答用のチャネルの送信端など `Send` な値だけである。

### Rust 側（pasta_shiori）

| ファイル | 役割 |
| -------- | ---- |
| `crates/pasta_shiori/src/windows.rs` | エクスポートする関数。`loadu`・`load` の共通本体 `load_entry`、`loadu` 済みの印 `LOADU_INITIALIZED`、各入口の `catch_unwind`、応答の HGLOBAL 化 |
| `crates/pasta_shiori/src/util/hglobal/` | `ShioriString`（HGLOBAL の取り込みと解放、`GMEM_FIXED` での確保、UTF-8・ANSI としての読み出し）。ANSI の変換は `MultiByteToWideChar`（`CP_ACP`）による |
| `crates/pasta_shiori/src/actor/lifecycle.rs` | `static MAILBOX`（`ArcSwapOption<Sender<ActorMsg>>`）と `ACTOR_HANDLE`（`Mutex<Option<ActorThread>>`）、FFI 入口が呼ぶ `spawn_actor`・`marshal_request`・`teardown_actor`、キックの投函口 `kick_sink` |
| `crates/pasta_shiori/src/actor/mailbox.rs` | mailbox を流れるメッセージ `ActorMsg`（`Get`・`Notify`・`Stop`・`Kick`。`#[non_exhaustive]`）、`MailboxRequest`（`seq` と生のリクエスト文字列）、`Reply`、mailbox の生成（flume の unbounded） |
| `crates/pasta_shiori/src/actor/thread.rs` | アクタースレッドの起動 `spawn_actor_thread` と、メッセージを処理するループ。`ActorThread`（スレッドのハンドル・ロードの成否・DAP の待受アドレス） |
| `crates/pasta_shiori/src/actor/marshaling.rs` | メソッドの判定 `determine_method`、`marshal_get`・`marshal_notify`、安全網の応答 `default_204`、GET の待ち時間の上限 `GET_TIMEOUT`（5 秒） |
| `crates/pasta_shiori/src/actor/teardown.rs` | `Stop{done}` を送って完了の通知を待つ `teardown_via_sender` と結果 `TeardownReport`。再読み込みを繰り返して OS のハンドル数を測る `ReloadProbe`（テストが使う） |
| `crates/pasta_shiori/src/shiori.rs` | `Shiori` トレイトと `PastaShiori`。ランタイムの構築、`SHIORI.load`・`SHIORI.request`・`SHIORI.unload` の関数のキャッシュ、`SHIORI.kick` の呼び出し、`Drop` での後始末 |
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
| `pasta.shiori.entry` | グローバルの `SHIORI` 表（`load`・`request`・`kick`・`unload`）。`GLOBAL.close_ghost`（`GLOBAL["ゴースト終了"]`）も登録する |
| `pasta.shiori.event` | `EVENT.fire`（振り分けとコルーチンの再開）と、ハンドラが無いときの `EVENT.no_entry` |
| `pasta.shiori.event.register` | `REG`。依存を持たない空の表 |
| `pasta.shiori.event.boot`・`pasta.shiori.event.choice_select`・`pasta.shiori.event.second_change` | 既定ハンドラ（OnBoot・OnChoiceSelectEx・OnSecondChange）を `REG` に登録する |
| `pasta.shiori.event.virtual_dispatcher` | 仮想イベント（OnHour・OnTalk）の判定と発行 |
| `pasta.shiori.event.callback` | 非同期トークのコールバック待ちの登録・ルーティング・タイムアウトの掃引 |
| `pasta.shiori.event.kick` | シーンキックの保留と起動（[デバッグ基盤とシーンキック](debug.md)） |
| `pasta.shiori.res` | SHIORI/3.0 の応答文字列を作る `RES` |
| `pasta.shiori.act`・`pasta.shiori.sakura_builder`・`pasta.shiori.appearance` | SHIORI 用の ACT とさくらスクリプトの組立（[トーク出力とアピアランス](talk-output.md)） |
| `pasta.shiori` | 空の表を返すだけのモジュール。リポジトリ内に `require` する箇所は無いが、展開されるスクリプトの公開面として残している |

## 処理とデータの流れ

### load とアクターの起動

`loadu` と `load` は、設置パスの文字コードだけが異なる 2 つの入口で、どちらも `load_entry` を通る。`loadu` は UTF-8、`load` はシステムの ANSI コードページ（`CP_ACP`）として設置パスを読む。

```text
load_entry(entry, hdir, len, encoding)
 1. hdir が null → false
 2. ShioriString::capture(hdir, len)   以後のどの経路でも Drop で GlobalFree される
 3. load かつ LOADU_INITIALIZED → 何もせず true（loadu 済みの状態を保つ）
 4. len == 0 → false
 5. catch_unwind(load_impl)            panic は false
load_impl
 a. 設置パスを UTF-8（to_utf8_str）か ANSI（to_ansi_str）で読む。失敗 → false
 b. loadu なら LOADU_INITIALIZED = true（ロードが失敗しても立てたままにする）
 c. lifecycle::spawn_actor(0, 設置パス)
 d. 同じ設置パスで LoadDirGuard を張り直し、入口名と成否を 1 行ログに出す
```

- `LOADU_INITIALIZED` をロードの失敗時にも立てるのは、続いて `load` が呼ばれたときに、ANSI で欠落したパスによる別の失敗で `loadu` の失敗の原因を上書きしないためである。印は `unload` で下ろす。利用者から見た扱いは [起動シーケンスとモジュール解決](../reference/startup.md#loadu-を呼ばないホストでは非-ansi-の設置パスを扱えない) を参照する。
- `SHIORI.load` に渡す `hinst` は、FFI の経路では常に 0 である（`DllMain` が受け取るモジュールハンドルは保持しない）。
- d でガードを張り直すのは、ログのファイルへの振り分けがスレッドごとの設置パスで決まり、アクタースレッドで張ったガードが FFI 入口のスレッドには及ばないためである（[ロギングとエンコーディング](logging-encoding.md)）。

`spawn_actor` は次の順に動く。

```text
spawn_actor(hinst, load_dir)
 1. teardown_actor()                     前のアクターがあれば終了させる（再読み込み）
 2. (tx, rx) = mailbox()                 新しいチャネル
 3. spawn_actor_thread(hinst, load_dir, rx)
      スレッド "pasta-actor" を起こし、その上の block_on で
        PastaShiori::default().load(hinst, load_dir)
        (スレッド ID, 成否, DAP の待受アドレス) を bounded(1) で送り返す
        メッセージループへ入る
      呼び出し側は送り返しを recv() で待つ   → VM の構築が終わるまで戻らない
 4. MAILBOX.store(Some(Arc::new(tx)))
 5. ACTOR_HANDLE にスレッドのハンドルを保持する
 6. 成否を返す
```

`PastaShiori::load` はアクタースレッドの上で次を行う。

1. 設置パスが存在しなければ `false` を返す（エラーの理由は残らない）。
2. 既定の設定でロガーを `GlobalLoggerRegistry` に登録し、tracing を初期化し、`LoadDirGuard` を張る。
3. `RuntimeConfig::new().with_kick_sink(Some(kick_sink()))` を作り、`PastaLoader::load_with_config` でランタイムを構築する。失敗したらエラーの文字列を `last_load_error` に残して `false` を返す。
4. `SHIORI` 表から `load`・`request`・`unload` の関数を取り出してキャッシュする。リクエストのたびに表を引かないためである。どれも省略でき、無い `load` は成功として扱い（5）、無い `request` には 204 を返し（後述）、無い `unload` は呼ばない。`unload` 以外が無いときは warn ログを出す。
5. `SHIORI.load(hinst, 設置パスの文字列)` を呼ぶ。関数が無ければ成功とみなす。`false` を返すかエラーなら `false`。

ロードが失敗しても、アクタースレッドは起動したまま `MAILBOX` に登録され、以後のリクエストを受け付ける。GET の応答は失敗の段階で異なる。3 のランタイムの構築に失敗した場合は、残した理由を 500 応答で返す。1 で設置パスが無かった場合は、理由の無い 500 応答（`Not initialized error`）を返す。5 で `SHIORI.load` が失敗した場合はランタイムが残っているため、リクエストは通常どおり `SHIORI.request` で処理される（[アクタースレッドのメッセージループ](#アクタースレッドのメッセージループ) の `PastaShiori::request`）。

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
 i.   MAILBOX.load_full()。空（load 前・unload 後）→ default_204
 ii.  seq を採番する（ログで順序を追うための連番）
 iii. determine_method：pest の req 規則で解析し、最初に現れた get / notify を採る
      解析できない → default_204
 iv.  GET    → marshal_get
      NOTIFY → marshal_notify
```

- `marshal_get` は `bounded(1)` の応答用チャネルを作って `ActorMsg::Get` に同梱し、`try_send` で投入してから `recv_timeout(GET_TIMEOUT)` で待つ。値が届けばそれを返し、応答用チャネルが送られずに破棄された（`Disconnected`）か、5 秒を過ぎた（`Timeout`）か、`try_send` に失敗したなら `default_204` を返す。
- `marshal_notify` は `ActorMsg::Notify` を `try_send` し、成否に関わらずすぐ `default_204` を返す。アクターの処理は待たない。
- メソッドの判定は VM に投入する前に、ホストのスレッドで行う。同じリクエスト文字列は、アクタースレッドで `parse_request` がもう一度解析する。
- 応答の HGLOBAL はホストが解放する。DLL は解放しない。

### アクタースレッドのメッセージループ

アクタースレッドは `wintf-winmsg-executor` の `block_on` で、そのスレッドの Windows メッセージループの上で非同期のブロックを動かす。ブロックは `rx.recv_async().await` だけで待ち、別のスレッドの `try_send` が flume の Waker を通じてループを起こす。

```text
while let Ok(msg) = rx.recv_async().await
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
block_on が戻る       スレッドの終了時に、executor のメッセージ専用ウィンドウ（thread_local）が破棄される
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
unload()
 1. LOADU_INITIALIZED = false
 2. catch_unwind(teardown_actor)        異常があれば warn ログ
 3. 常に true
teardown_actor()
 a. tx = MAILBOX.swap(None)             以後の送信を断つ
 b. ACTOR_HANDLE からハンドルを取り出す
 c. tx があれば teardown_via_sender(tx, 5 秒)、無ければ「終了済み」
 d. ハンドルがあれば detach（join しない）
teardown_via_sender(tx, timeout)
 - send(Stop{done}) が失敗（受信側がもう無い）→ already_done（何もしない）
 - done を recv_timeout で待つ
     届いた      → acked（正常な終了）
     Timeout     → anomaly（warn ログ）
     Disconnected → anomaly（warn ログ）
```

- `Stop` は他のメッセージと同じ FIFO を通るため、先に積まれたメッセージを処理し終えてからループを抜ける。
- `PastaShiori` の `Drop` は、キャッシュした `SHIORI.unload` を呼び、ロガーの登録を外し、キャッシュした関数を捨ててからランタイムを破棄する。ランタイムの破棄で永続化データが保存される（[永続化](execution-model.md#永続化)）。DAP のバックエンドも VM の一部としてこのとき片付く。
- 完了の通知は、VM の破棄と mailbox の受信側の破棄が終わってから送る。通知を受け取った時点で、VM と DAP バックエンドの解放は済んでいる。executor のメッセージ専用ウィンドウ（`wintf-winmsg-executor` の thread_local）は、その後 `block_on` が戻ってスレッドが終わるときに破棄される。
- `DllMain` は `DLL_PROCESS_DETACH` で `unload()` を呼び、それ以外の通知では何もしない。`DllMain` はローダーロックを持ったまま呼ばれるため、そこでスレッドを起こさない。スレッドは `load` を起点に起こす。
- 再読み込み（`unload` の後の `load`、または `load` の再呼び出し）は、`spawn_actor` の 1 で前のアクターを終わらせ、新しいスレッド・チャネル・VM を作る。`PastaShiori::load` の中にある、既存のランタイムを捨てて作り直す分岐は、FFI の経路では通らない（アクターごとに新しい `PastaShiori` を作るため）。

### Lua 側の SHIORI エントリとイベント配送

起動の 7 段目（[VM の構築とモジュール登録](execution-model.md#vm-の構築とモジュール登録)）で `require("pasta.shiori.entry")` が評価される。`pasta.shiori.entry` が `require` するモジュールの評価で、既定ハンドラが `REG` に登録される。

```text
pasta.shiori.entry
  ├→ pasta.shiori.event
  │     ├→ pasta.shiori.event.register（REG）・pasta.shiori.res・pasta.shiori.act
  │     │  pasta.store・pasta.shiori.event.callback
  │     ├→ pasta.shiori.event.boot           REG.OnBoot を登録
  │     ├→ pasta.shiori.event.choice_select  REG.OnChoiceSelectEx を登録
  │     └→ pasta.shiori.event.second_change  REG.OnSecondChange を登録
  │           └→ pasta.shiori.event.virtual_dispatcher
  ├→ pasta.shiori.res
  ├→ pasta.shiori.event.kick
  └→ pasta.global                            close_ghost・ゴースト終了 を追加
```

既定ハンドラの上書きがどの時点で効くかは [既定ハンドラと上書き](../lua/shiori-events.md#既定ハンドラと上書き) を参照する。

`SHIORI` の関数は次のとおりである。`SHIORI = SHIORI or {}` で、既にあるグローバルの表に関数を足す。

| 関数 | 動作 |
| ---- | ---- |
| `SHIORI.load(hinst, load_dir)` | `xpcall` の中で何もせず `true` を返す。エラーなら `print` して `false` |
| `SHIORI.request(req)` | `xpcall` の中で `EVENT.fire(req)` を呼ぶ。エラーは、文字列なら最初の 1 行を、そうでなければ `nil` を理由にして `RES.err` で 500 応答にする（`nil` は `"Unknown error"` になる） |
| `SHIORI.kick(scene)` | `KICK.install(scene)` でキックを保留するだけで、シーンは動かさない |
| `SHIORI.unload()` | `xpcall` の中で何もしない |

`EVENT.fire(req)` の振り分けは次の順である。

1. `CALLBACK.try_route(req)` が応答を返せば、それを返す（待っているコールバックの再開）。
2. `act = SHIORI_ACT.new(STORE.actors, req)` を作る。
3. `REG[req.id]` があればそれを、無ければ `EVENT.no_entry` を `act` で呼ぶ。`EVENT.no_entry` は `SCENE.co_exec(act, req.id)` でイベント名のシーンを探し、見つかればそのコルーチンを、無ければ `nil` を返す（`pasta.scene` は循環を避けるため呼び出し時に `require` する）。
4. 戻り値がコルーチンなら再開して応答にし、文字列なら `RES.ok` で包み、それ以外なら 204 を返す。コルーチンの扱いは [イベントからシーンへ](execution-model.md#イベントからシーンへ) で扱う。

既定ハンドラの中身は次のとおりである。

| ハンドラ | 動作 |
| -------- | ---- |
| `REG.OnBoot` | `SCENE.co_exec(act, act.req.id)`。シーン関数フォールバックと同じ |
| `REG.OnChoiceSelectEx` | まず `SCENE.co_exec(act, "OnChoiceSelectEx")`。無ければ `act.req.reference[1]`（選択 ID）を `SCENE.search(選択 ID, STORE.last_global_scene)` で探し、無ければ `SCENE.search(選択 ID, nil)` でグローバルシーンを探し、見つかった関数をコルーチンに包んで返す。`SCENE.co_exec` は親のグローバルシーンを渡せないため、検索を直接呼ぶ |
| `REG.OnSecondChange` | `CALLBACK.sweep(os.time())` が応答を返せばそれを返し、そうでなければ `virtual_dispatcher.dispatch(act)` の結果（コルーチンか `nil`）を返す |

### 非同期トーク

SHIORI はベースウェアが問い合わせるだけの通信であり、シーンの途中で SSP のプロパティを読むには、要求を応答に載せ、値を後のリクエストで受け取るしかない。非同期トークはこれを 1 つのシーンの中の待ち合わせとして実現する。

```text
リクエスト 1（任意のイベント）
  シーン内 act:get_property(名前…)
    CALLBACK.stage_pending("OnPastaCallBack{N}", 期限, 理由)
    応答の Value に \![get,property,OnPastaCallBack{N},名前…] を載せて中断
リクエスト 2（SSP が発行する OnPastaCallBack{N}。値は Reference に入る）
  EVENT.fire → CALLBACK.try_route
    待っているコルーチンを Reference の配列で再開し、その出力を応答にする
期限切れ
  OnSecondChange の既定ハンドラ → CALLBACK.sweep
```

- アクターランタイムの上では、リクエスト 1 と 2 は別々の mailbox のメッセージとして順に処理される。待っているコルーチンは、その間 VM の `CALLBACK.pending` に保たれる。Rust 側はコルーチンを持たない。
- 待ち合わせの状態の遷移（`consume_staged`・`STORE.co_callback`・`set_co_scene` との関係）は [コールバック待ちとの関係](execution-model.md#コールバック待ちとの関係) で扱う。利用者から見た挙動は [OnPastaCallBack](../lua/shiori-events.md#onpastacallbackコールバック応答) が正である。

### 仮想イベントディスパッチャ（OnTalk・OnHour）

OnTalk と OnHour は、`REG.OnSecondChange` の既定ハンドラが毎回呼ぶ `pasta.shiori.event.virtual_dispatcher` が発行する。状態はモジュールのローカル変数 `next_hour_unix`（次の正時）と `next_talk_time`（次のトーク時刻）だけで、どちらも 0 が「未設定」を表す。状態は VM と同じ寿命（`load` から `unload` まで）で、保存されない。

```text
dispatch(act)
 1. act.req.date が無ければ nil
 2. force = STORE.kick_force。真なら false に戻す（1 回だけ使う）
 3. force でなく is_blocked(act.req.status) なら nil
 4. KICK.try_dispatch(act) がコルーチンを返せばそれを返す
 5. check_hour(act) がコルーチンを返せばそれを返す
 6. check_talk(act) を返す
```

利用者から見た判定の順・ブロック対象のキーワードと照合の仕方・OnHour のシーン候補の順・トーク間隔の決まり方・テスト用の関数は、[仮想ディスパッチャ](../lua/shiori-events.md#仮想ディスパッチャ)（[ブロック対象 Status キーワード](../lua/shiori-events.md#ブロック対象-status-キーワード)・[OnHour](../lua/shiori-events.md#onhour)・[pasta.toml 設定](../lua/shiori-events.md#pastatoml-設定)・[テスト用関数](../lua/shiori-events.md#テスト用関数)）が正である。ここではそれを実現する仕組みだけを書く。

- 2 の `STORE.kick_force` は `KICK.install` が立てる。`dispatch` の入口が読んだ時点で消費するため、ブロックの判定を飛ばすのはキックの直後の 1 回だけである。4 はキックの保留を起動する段で、中身は [デバッグ基盤とシーンキック](debug.md) で扱う。
- `check_hour` は、`next_hour_unix` が 0 なら次の正時（`unix - unix % 3600 + 3600`）を記録するだけで終わる。現在時刻がそれに達していれば、同じ式で次の正時を記録し直してからシーンを探す。
- `check_talk` は、`next_talk_time` が 0 なら次のトーク時刻を決めるだけで終わる。正時までの残りを `hour_margin` と比べる判定は、`check_hour` が記録した `next_hour_unix` を使う。この判定で発行を見送るときは `next_talk_time` を変えないため、次の秒にまた判定する。発行するときは、シーンを探す前に `next_talk_time` を決め直す。
- 次のトーク時刻は「現在時刻 + `math.random(最小間隔, 最大間隔)`」で決める。間隔は、決めるたびにローカル関数 `get_config` が `pasta.save` と `@pasta_config` を読み直して求める（キャッシュしない）。`hour_margin` は `[ghost].hour_margin`（無ければ 30）を、切り捨てや下限の補正をせずにそのまま使う。`[ghost]` の既定値の補完は [設定読込](loader.md#設定読込) で扱う。
- ブロック中（3 で終わる間）は `check_hour`・`check_talk` を呼ばないため、`next_hour_unix`・`next_talk_time` は進まない。そのため、ブロックが解けた最初の判定で、過ぎていた正時やトーク時刻の条件が成り立つ。
- 返したコルーチンは `EVENT.fire` が再開し、`set_co_scene` が `STORE.co_scene` を置き換える。OnHour とキックのシーンが中断中の継続を置き換え、OnTalk だけが継続を再開する（`check_talk` が `STORE.co_scene` を返す）のはこのためである（[継続トークと co_scene の更新](execution-model.md#継続トークチェイントークと-co_scene-の更新)）。
- 仮想イベントは `REG` を通らない。発行するシーンはどれも `SCENE.co_exec` で探す（テスト用の差し替え `scene_executor` が設定されていればそれを使う）。

### 応答文字列の出どころ

応答文字列を作る場所は 5 つあり、ヘッダの組が異なる。

| 作る場所 | 応答 | ヘッダ | 使われる場合 |
| -------- | ---- | ------ | ------------ |
| `RES`（Lua） | 200・204・311・312・400・500 | `Charset`・`Sender`・`SecurityLevel` と追加分。500 の理由は `X-Error-Reason` | `EVENT.fire` と `SHIORI.request` のすべての応答 |
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

`crates/pasta_shiori/Cargo.toml` はライブラリ名を `pasta`、`crate-type` を `cdylib` と `rlib` にしている。`cdylib` が pasta.dll になり、`rlib` は統合テストが Rust から使うためのものである（`crates/pasta_shiori/src/lib.rs` は `load`・`loadu`・`request`・`unload` を再エクスポートし、テストが出荷シンボルそのものを呼べるようにしている）。`crates/pasta_shiori/src/windows.rs` と `crates/pasta_shiori/src/util/hglobal/` は `#[cfg(windows)]` で、Windows 以外のビルドは FFI の関数を持たない。

リリースビルドの設定はルートの [Cargo.toml](https://github.com/ekicyou/pasta/blob/main/Cargo.toml) の `[profile.release]` にあり、ワークスペースの全クレートに効く。pasta.dll の大きさを小さくするための設定である。

| キー | 値 | 効果 |
| ---- | -- | ---- |
| `opt-level` | `"z"` | 大きさを優先して最適化する |
| `lto` | `true` | クレートをまたいでリンク時に最適化する |
| `codegen-units` | `1` | コード生成を分割せず、最適化の範囲を広げる |
| `panic` | `"abort"` | panic で巻き戻さず中止する。巻き戻しの表を持たない |
| `strip` | `true` | シンボルを取り除く |

`panic = "abort"` のため、リリースビルドの pasta.dll では panic がホストのプロセスごと中止させる。FFI 入口の `catch_unwind` が効くのは、巻き戻すプロファイル（dev・test）だけである。

ルートの `.cargo/config.toml` は、`i686-pc-windows-msvc` と `x86_64-pc-windows-msvc` の両方に `-Ctarget-feature=+crt-static` を与え、MSVC の CRT を静的にリンクする。pasta.dll は VC++ 再頒布可能パッケージ（`vcruntime140.dll` など）に依存しない。Cargo は `.cargo/config.toml` を作業ディレクトリから上へ探すため、ルートから `cargo build -p pasta_shiori` を実行する CI でも効くよう、クレートの下ではなくルートに置いている。

`crates/pasta_shiori/build.rs` は、`CARGO_PKG_VERSION` などから `VERSIONINFO` の `.rc` を `OUT_DIR` に生成し、`embed_resource::compile_for_cdylib` で pasta.dll にだけ埋め込む。数値のバージョンは「メジャー,マイナー,パッチ,0」で、文字列のバージョンは `CARGO_PKG_VERSION` そのものである。Windows で `rc.exe` が使えず埋め込みに失敗するとビルドを失敗させる（`manifest_required().unwrap()`）。Windows 以外では何もしない。

## 境界の受け渡し

| 境界 | 渡す側 → 受ける側 | 渡すもの | 所有 |
| ---- | ----------------- | -------- | ---- |
| ホスト → pasta.dll（`loadu`・`load`） | ホスト → `load_entry` | 設置パスの HGLOBAL と長さ | 受け取った時点で DLL が所有し、どの経路でも解放する |
| ホスト ↔ pasta.dll（`request`） | ホスト → `request` → ホスト | リクエストの HGLOBAL と長さ（入力）、応答の HGLOBAL と長さ（戻り値と `len`） | 入力は DLL が解放する。応答（`GMEM_FIXED`）はホストが解放する |
| FFI 入口のスレッド → アクタースレッド | `marshal_*`・`teardown_via_sender`・`kick_sink` → mailbox | `ActorMsg`（生のリクエスト文字列と `seq`、応答用・完了通知用の送信端、シーン名） | メッセージは move される。VM はアクタースレッドから出ない。mailbox の送信端は `MAILBOX` が、受信端はアクタースレッドが持つ |
| アクタースレッド → FFI 入口のスレッド | 応答用チャネル・起動の報告用チャネル | `Reply::Value(応答文字列)`、`(ThreadId, bool, Option<SocketAddr>)` | 文字列と `Send` な値だけが渡る |
| `pasta_shiori` → `pasta_lua`（構築） | `PastaShiori::load` → `PastaLoader::load_with_config` | 設置パスと `RuntimeConfig`（`KickSink` を載せる） | `PastaShiori` が `PastaLuaRuntime` とキャッシュした Lua 関数を所有する |
| `pasta_lua` → `pasta_shiori`（キック） | デバッグ通信のスレッド → `KickSink` | `KickRequest { scene }` | クロージャは `static MAILBOX` だけに依存し、`try_send` で投函する。`MAILBOX` が空なら捨てる |
| Rust → Lua（リクエスト） | `PastaShiori` → `SHIORI.request` | `parse_request` が作った表 | 表は VM が所有し、ACT の `req` として残る |
| Lua → Rust（応答） | `SHIORI.request` → `PastaShiori` | SHIORI 応答の文字列の全体 | 文字列の複製が Rust に渡る |
| `pasta_lua` のコア → レンダラ | `register_sakura_script_module` → `RendererInjection` | VM と `TalkConfig` | 返った表は `package.loaded` に置かれて VM が所有する |

## 不変条件と制約

- VM（`PastaShiori`・`PastaLuaRuntime`）はアクタースレッドで作られ、使われ、破棄される。`PastaShiori` に `unsafe impl Send`・`Sync` は無く、スレッドをまたぐのは `Send` な値だけである。
- mailbox は 1 本で、消費するのはアクタースレッドの `recv_async` のループ 1 つだけである。`select!` は使わない。新しい種類のメッセージは、別のチャネルではなく `ActorMsg` の新しい variant として足す。これにより VM への同時アクセスは構造上起こらず、メッセージは投入順に処理される。NOTIFY はすぐ 204 を返すが、その処理が終わるまで後続の GET は処理されない。
- 送信の経路（`marshal_request`・`kick_into_mailbox`）は `MAILBOX` の `load_full` だけで送信端を得て、Mutex を取らない。`ACTOR_HANDLE` の Mutex は起動と終了のときだけ触る。
- ホストのスレッドを際限なく待たせない。GET の応答待ちと終了の完了待ちはそれぞれ 5 秒で打ち切る。応答を返せない場合はすべて `default_204` を返す。
- リクエストの処理のエラー（ロードの失敗・未ロード・`SHIORI.request` の失敗）は 500 応答になる。204 は応答を返せなかった場合だけの安全網である。
- GET が 5 秒を過ぎて 204 を返しても、アクタースレッドの処理は止めない（LuaJIT の実行は途中で止められない）。その処理は最後まで進み、応答は受け取り手が無いため捨てられる。VM の状態はその処理の結果どおりに更新される。
- 入力の HGLOBAL は DLL が必ず解放し、応答の HGLOBAL はホストが解放する。`GlobalAlloc` の失敗は null のまま使わず、null の応答と長さ 0 にする。
- 各 FFI 入口は `catch_unwind` で panic をホストへ伝えない。ただしリリースでは `panic = "abort"` のため、正常な経路では panic を起こさない書き方をする（`crates/pasta_shiori/src/actor/marshaling.rs` の正常経路に `unwrap`・`expect` が無いことをテストが確かめる。リクエストの解析は再帰しない）。
- `DllMain` ではスレッドを起こさない。アクタースレッドは `load` を起点に起こす。
- 終了処理は何度呼んでも安全である（`unload` の二重呼び出し、`unload` と `DllMain` の detach の重なり）。スレッドは join せず detach する。完了の通知は VM の破棄と mailbox の受信側の破棄が済んでから送る。
- `default_204` のバイト列は `RES.no_content()` と同じであり、FFI 入口の応答のバイト列はテスト `crates/pasta_shiori/tests/byte_invariant_test.rs` が固定している。再読み込みを繰り返しても OS のハンドルが増えないことは `crates/pasta_shiori/tests/actor_reload_leak_test.rs` が確かめる。
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

本文で参照した主なファイルは次のとおりである。FFI 入口は `crates/pasta_shiori/src/windows.rs`、アクターランタイムは `crates/pasta_shiori/src/actor/`、リクエストの解析は `crates/pasta_shiori/src/lua_request.rs` と `crates/pasta_shiori/src/util/parsers/req_parser.pest`、Lua 側の振り分けは `crates/pasta_lua/pasta_scripts/pasta/shiori/entry.lua`・`crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua`、仮想イベントは `crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` にある。`@pasta_sakura_script` の登録を呼ぶ側は `crates/pasta_lua/src/runtime/module_registry.rs`・`crates/pasta_lua/src/runtime/factory.rs` である。

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

---

呼ばれたら応える、ただそれだけのことに、これほどの備えが要りますのよ。おほほ、頼もしいでしょう？
次は、応答の中身――トーク出力とアピアランスへ参りましょう！
