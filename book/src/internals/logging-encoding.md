# ロギングとエンコーディング

ごきげんよう。ゴーストの足跡を書き残すロギングと、文字化けからゴーストを守るエンコーディング――地味ですけれど、欠かせない縁の下の力持ちですわ。
その働きぶりを、わたくしがしっかりご紹介いたします。さあ、参りましょう。

---

この章では、ランタイムのロギングとエンコーディングを扱う。利用者から見た `@pasta_log` と `@enc` の使い方は [@pasta_log](../lua/modules/pasta-log.md) と [@enc](../lua/modules/enc.md) が、`[logging]` の設定は [pasta.toml リファレンス](../reference/pasta-toml.md#loggingログ出力) が正である。

## 目的と責務

ロギングは、Rust 側の `tracing` のイベントと Lua 側の `@pasta_log` の呼び出しを、ゴーストごとのログファイルへ書き出す。エンコーディングは、UTF-8 とシステムのコードページの間の文字列変換を提供する。

この章が責務を持つのは次の事項である。

- tracing の購読者（subscriber）の初期化と、ログフィルタの決め方・差し替え
- インスタンスロガー（`PastaLogger`）の作成と、ログファイルの場所・書き込み方
- グローバルなロガーの登録簿（`GlobalLoggerRegistry`）と、スレッドごとの設置パスによるログの振り分け
- `@pasta_log` の実装（値の文字列化・呼び出し元の取得・イベントの発行）
- `crates/pasta_lua/src/encoding/` の OS 別の変換の実装と、`@enc` の実装
- SHIORI 境界で受け渡す文字列の文字コードの一覧

次の事項は他の章が権威を持ち、この章では再記述しない。

- ローダの段階構成の中での段階 1.5 の位置と、失敗を継続扱いにする規則は [ローダ自己展開とモジュール解決](loader.md#処理とデータの流れ) で扱う。
- `@pasta_log`・`@enc` を `package.loaded` に登録する順序は [ランタイム実行モデル](execution-model.md#vm-の構築とモジュール登録) で扱う。
- `loadu`・`load`・`request` の各入口で文字列をどう読み、失敗をどう応答にするかは [SHIORI 層](shiori.md#処理とデータの流れ) で扱う。

## 構成要素

### ロギング

| ファイル | 主な型・関数 | 役割 |
| -------- | ------------ | ---- |
| `crates/pasta_lua/src/logging/tracing_init.rs` | `init_tracing_with_reload`・`update_tracing_filter`・`FILTER_HANDLE` | tracing の購読者を 1 回だけ設置し、ログフィルタを後から差し替える |
| `crates/pasta_lua/src/logging/logger.rs` | `PastaLogger` | 1 つのゴーストのログファイル。パスの検査、非同期の書き込み、破棄時のフラッシュ |
| `crates/pasta_lua/src/logging/registry.rs` | `GlobalLoggerRegistry`・`RoutingWriter`・`CURRENT_LOAD_DIR`・`LoadDirGuard` | 設置パスをキーにロガーを登録する簿と、書き込み先をスレッドローカルの設置パスで選ぶ writer |
| `crates/pasta_lua/src/runtime/log.rs` | `register`・`value_to_string`・`get_caller_info` | `@pasta_log` のモジュール表と 5 つのレベル関数 |
| `crates/pasta_lua/src/loader/config/sections.rs` | `LoggingConfig` | `[logging]` の型（`file_path`・`level`・`filter`）と、フィルタ文字列への変換 `to_filter_directive` |

ロガーを作って登録する側は、`crates/pasta_shiori/src/shiori.rs` の `PastaShiori::load`（段階 1）と `crates/pasta_lua/src/loader/mod.rs` の `PastaLoader::create_and_register_logger`（段階 1.5）である。

```text
tracing のイベント（Rust の info! など、@pasta_log の呼び出し）
   │
   ▼
Registry（プロセスに 1 つ）
 └ fmt レイヤ（ANSI 色なし・target とレベルを出力）
     │  レイヤ単位のフィルタ: reload::Layer<EnvFilter>（FILTER_HANDLE で差し替え）
     ▼
GlobalLoggerRegistry::make_writer（イベントごと）
     │  CURRENT_LOAD_DIR（スレッドローカル）で文脈（設置パス）を得る
     │  resolve: loggers: Mutex<HashMap<設置パス, Arc<PastaLogger>>> を引く（振り分けの 3 規則）
     ▼
RoutingWriter ── 決まった → PastaLogger::write → NonBlocking → ワーカースレッド → ログファイル
               └ 無い     → 何もせず書いたことにする（捨てる）
```

### エンコーディング

| ファイル | 主な型・関数 | 役割 |
| -------- | ------------ | ---- |
| `crates/pasta_lua/src/encoding/mod.rs` | `Encoder`・`Encoding` | 変換のトレイトと、コードページの種類（`ANSI`・`OEM`）の列挙 |
| `crates/pasta_lua/src/encoding/windows.rs` | `Encoder for Encoding` | Windows の実装。`MultiByteToWideChar`・`WideCharToMultiByte` で変換する |
| `crates/pasta_lua/src/encoding/unix.rs` | `Encoder for Encoding` | Windows 以外の実装。UTF-8 のまま通す |
| `crates/pasta_lua/src/runtime/enc.rs` | `register`・`to_ansi_impl`・`to_utf8_impl` | `@enc` のモジュール表と 2 つの変換関数 |

どちらの OS の実装を使うかは `#[cfg(windows)]` でビルド時に決まる。`Encoder` と `Encoding` はクレートのルートから再エクスポートされる。`Encoding::OEM`（`CP_OEMCP`）を使うのはテストだけである。

pasta_shiori は FFI の文字列を読むための変換を、`crates/pasta_shiori/src/util/hglobal/` に別に持つ（`enc.rs` と `windows_api.rs`）。`crates/pasta_lua/src/encoding/windows.rs` と同じく local-encoding-rs に由来する実装で、`crates/pasta_lua/src/encoding/` は使わない。

## 処理とデータの流れ

### ロガーの 2 段階の初期化

SHIORI として読み込まれるとき、ロガーは 2 段階で用意される。段階の番号はコードのコメントとログに合わせている。

```text
段階 1（PastaShiori::load。PastaLoader を呼ぶ前）
 1. 再読み込みなら、前の設置パスのロガーの登録を外す
 2. PastaLogger::new(設置パス, None)      … 既定の file_path で作る。失敗しても続行
    成功 → GlobalLoggerRegistry に 設置パス → ロガー で登録
 3. init_tracing_with_reload(LoggingConfig::default())
      購読者の設置は try_init。プロセスで最初の 1 回だけ成功し、そのときだけ FILTER_HANDLE を保存
 4. LoadDirGuard::new(設置パス)            … 以後このスレッドのログは設置パスのロガーへ

段階 1.5（PastaLoader::load_with_config。pasta.toml を読んだ直後）
 5. config.logging()                       … [logging] が無い・型が合わない → None
    update_tracing_filter([logging] または既定) で FILTER_HANDLE のフィルタを差し替える
 6. PastaLogger::new(設置パス, [logging] または既定)
    成功 → 同じ設置パスで登録し直す（段階 1 のロガーを置き換える）
    失敗 → PastaLogger::new(設置パス, None) で既定のログファイルのロガーを作り直す
           成功 → 同じ設置パスで登録し、その後で warn を 1 件出す
                  （不正と判断した file_path・切り替え先 profile/pasta/logs/pasta.log・失敗の理由）
           失敗 → warn「logging disabled」を出し、ロガー無しで続行（登録は変えない）
 7. 登録したロガーの Arc を PastaLuaRuntime へ渡す（段階 6 で構造体に保持）
```

- 段階 1 は、段階 1.5 より前（設定の読み込みを含む）のログもファイルに残すためにある。段階 1 のロガーは常に既定の場所 `profile/pasta/logs/pasta.log` に作られるため、`file_path` を変えたゴーストでも、段階 1.5 までのログはこのファイルに書かれる。
- 段階 1.5 で登録を置き換えると、段階 1 のロガーへの参照は登録簿から消え、最後の参照が落ちた時点で破棄される（破棄時にフラッシュする）。置き換えられたロガーの破棄は、登録簿のロックを放してから行う。
- 6 の切り替えは、`file_path` が判定（後述の `validate_path`）で不正なときも、ディレクトリを作れないなど別の理由で失敗したときも、理由を区別せずに行う。起動は続き、`level`・`filter` は 5 で反映済みである。warn は既定のロガーを登録した後に出すため、SHIORI 経由でも `PastaLoader` を直接使う組み込みでも、同じ文言で既定のログファイルに書かれる。SHIORI 経由では、段階 1 の既定のロガーを同じファイルの新しい既定のロガーで置き換えることになる。`file_path` を直して再読み込みすれば、段階 1.5 が直したパスのロガーに置き換える。
- `PastaLoader` を SHIORI 以外から直接呼ぶ場合（テストなど）は段階 1 が無い。購読者を設置するかどうかは呼び出し側が決める。

ロガーの登録を外すのは `PastaShiori` の `release_runtime`（`Drop` と再読み込みの分岐から呼ばれる）で、ランタイムを破棄した後の最後の手順である。`PastaLuaRuntime` も同じロガーの `Arc` を持つが、ランタイムを先に破棄するため、登録を外した時点で最後の参照が落ち、フラッシュしてログファイルが閉じる。ランタイムの破棄（永続化データの保存）で出たログは、閉じる前のファイルに書かれる（[SHIORI 層](shiori.md#unloaddllmain-と-teardown)）。

### ログフィルタ

フィルタは tracing-subscriber の `EnvFilter` で、`build_filter` が次の順に作る。

1. 環境変数 `PASTA_LOG` が設定されていて解釈できれば、それを使う（未設定と解釈できない値は次へ進む）。
2. `LoggingConfig::to_filter_directive` の文字列（`filter` があればそれ、無ければ `level`。`level` の既定は `"info"`）を解釈できれば、それを使う。
3. どちらも使えなければ、標準エラーに警告を出して `"info"` にする。

段階 1 は既定の設定でこの順をたどり、段階 1.5 は毎回、`[logging]` の設定（無ければ既定の設定）で作り直したフィルタに差し替える。再読み込みで `[logging]` を消すと、フィルタは既定に戻る。フィルタは `reload::Layer` で包まれており、購読者を設置し直さずに `FILTER_HANDLE` 経由で中身だけを入れ替える。

### ログ 1 件がファイルに届くまで

1. イベントがフィルタを通ると、fmt レイヤが 1 行に整形する（時刻・レベル・target・メッセージ・フィールド。ANSI の色付けはしない）。
2. fmt レイヤはイベントごとに `GlobalLoggerRegistry::make_writer` を呼ぶ。`make_writer` はそのスレッドの `CURRENT_LOAD_DIR` を読み、`resolve` で書き込み先のロガーを決めて（下の 3 規則）、`RoutingWriter` を返す。
3. `RoutingWriter` は、ロガーがあれば `PastaLogger::write` へ渡し、無ければ書いたバイト数を返して捨てる。捨てるときもエラーや panic は起こさない。
4. `PastaLogger::write` は、ミューテックスで守った `tracing_appender::non_blocking` の writer に書く。実際のファイル書き込みは、ロガーごとのワーカースレッドが行う。

### 振り分けの規則

`resolve` は、スレッドの文脈（`CURRENT_LOAD_DIR` の設置パス）と登録簿から、次の 3 規則で書き込み先を決める。

1. 文脈がある → その設置パスのロガーへ書く。その設置パスに登録が無ければ捨てる（他のロガーへは流さない）。
2. 文脈が無く、登録されたロガーがちょうど 1 つ → そのロガーへ書く。
3. それ以外（文脈が無く、登録が 0 個または 2 個以上）→ 捨てる。

規則 1 で他のロガーへ流さないのは、あるゴーストのログを別のゴーストのログファイルに書かないためである。規則 3 で 2 個以上のときに捨てるのも、どのゴーストに属するかを決められないためである。登録簿のミューテックスを持つのは、表を引いて `Arc` を複製する間だけである。`register` が置き換えた古いロガーと `unregister` が外したロガーの破棄（フラッシュとワーカースレッドの終了待ち）は、ロックを放してから行う。

`CURRENT_LOAD_DIR` を設定するのは `LoadDirGuard` で、ガードの破棄で元の値に戻す（入れ子にできる）。文脈を張るのは次の箇所である。

| 箇所 | 張る範囲 |
| ---- | -------- |
| アクタースレッドの入口（`spawn_actor_thread` のスレッド本体の先頭） | スレッドの終わりまで。メッセージループの観測ログ（`actor.spawn`・`actor.recv`・`actor.reply`・`actor.stop`）もこの文脈で出る |
| `PastaShiori` の `load`（段階 1 の登録の後）・`request`・`kick`・`call_lua_unload`・`release_runtime` | 各メソッドを抜けるまで。アクタースレッドの文脈の内側で入れ子になる |
| `PastaLoader::load_with_config`（設置パスの存在を確かめた直後） | 関数を抜けるまで。組み込みで複数のゴーストを読んでも、ローダのログは自分のロガーへ届く |
| FFI 入口の `load_impl`（`load_entry` から呼ばれる） | `spawn_actor` が戻った後、入口名と成否の 1 行を出す間（理由は [SHIORI 層](shiori.md#load-とアクターの起動)） |

文脈を持たないのは、ホストのスレッド（FFI 入口の `request`・`unload`・`load_entry`、`DllMain` の detach から呼ばれる `unload`）と、デバッグバックエンドのスレッドである。これらのログは規則 2 で届く。

### 残るログと捨てるログ

pasta.dll では、登録簿のロガーは 0 個か 1 個である。そのため、ロガーが登録されている間に出たログは、どのスレッドで出たものでもログファイルに残る。捨てるのはロガーが登録されていないときのログだけである。

| ログ | 扱い |
| ---- | ---- |
| ロガーが登録されている間の、アクタースレッド・ローダのログ | 文脈のとおり自分のロガーへ届く（規則 1） |
| ロガーが登録されている間の、FFI 入口・デバッグバックエンドのログ（`request` の warn・panic の error・観測用の debug と trace、`loadu` 済みで無視した `load` の warn、`unload` の teardown の異常の warn など） | 唯一のロガーへ届く（規則 2） |
| 終了処理の間のログ（`SHIORI.unload` の結果、永続化データの保存の失敗、登録解除の通知「Unregistering logger」） | 登録解除の前に出るので届く。登録解除がファイルを閉じる最後の手順である |
| 最初の `load` の段階 1 より前のログ（入口の引数の誤り、設置パスを読めないなど） | 登録が 0 個なので捨てる（規則 3） |
| 終了処理の完了の通知（done ack）の後のログ（アクタースレッドの `actor.done`、FFI 入口の「done ack received」） | 登録解除の後なので捨てる（規則 1・規則 3） |
| 同じプロセスに 2 つ以上のロガーがあるとき（組み込みで複数のゴーストを読む場合）の、文脈の無いログ | どのゴーストのものか決められないので捨てる（規則 3） |

ログを残すために、SHIORI の応答・応答までの待ち時間の上限・`unload` の戻り値は変えない。FFI 入口のスレッドが登録簿のミューテックスを取るのは、ログのイベントがフィルタを通ったときだけで、mailbox の送信の経路には触れない。

### ログファイル

`PastaLogger::new(基準ディレクトリ, 設定)` は次の順に進む。

1. `基準ディレクトリ.join(file_path)` をログファイルのパスにする（`file_path` の既定は `"profile/pasta/logs/pasta.log"`）。
2. `validate_path` で検査する。満たさなければ `PermissionDenied` のエラーにする（段階 1.5 はこのエラーで既定のログファイルへ切り替える）。条件は次のすべてである。
   - パスが基準ディレクトリの下にある（`strip_prefix` で外れるもの、たとえば絶対パスはここで不正になる）。
   - 基準ディレクトリからの相対パスの最初の要素（`Path::components` の最初）が、ちょうど `profile` である（大文字小文字を区別する）。
   - `profile` の後に 1 つ以上の要素が続く。
   - 相対パスが文字列として `..` を含まない。

   `profile/pasta/logs/pasta.log`・`profile/x.log` は正しい。`profile.log`・`profiles/x.log` のように文字列として `profile` で始まっても `profile/` ディレクトリの下にないもの、`profile` だけのもの、`../x.log`・`profile/../x.log`・`logs/x.log` は不正である。
3. 親ディレクトリを作る（`create_dir_all`）。
4. `RollingFileAppender` を `Rotation::NEVER` とファイル名そのものを接頭辞にして作る。日付の接尾辞は付かず、ファイル名は `file_path` のとおりになる。既存のファイルには追記する。
5. `tracing_appender::non_blocking` でワーカースレッドと `WorkerGuard` を作り、`PastaLogger` が保持する。

`PastaLogger` の `Drop` はフラッシュし、続く `WorkerGuard` の破棄がワーカースレッドに残りを書かせる。ログファイルは日付で分けず、古いファイルの削除もしない。

### `@pasta_log` の呼び出し

`register` は `_VERSION`・`_DESCRIPTION` と、`trace`・`debug`・`info`・`warn`・`error` の 5 関数を持つ表を作る。5 関数はマクロ `log_fn!` が生成し、呼ぶ tracing のマクロ（レベル）だけが違う。1 回の呼び出しは次のとおりに進む。

```text
log.info(値)
 1. value_to_string(値)
      nil → ""、真偽値・整数・数値 → 文字列化、文字列 → そのまま（不正な UTF-8 は置換文字）
      表   → table_to_string（下記）
      その他（関数・ユーザーデータ・コルーチン） → Lua の tostring
 2. get_caller_info: inspect_stack(1) で直接の呼び出し元の
      short_src・現在行（取れなければ 0）・関数名（取れなければ ""）を得る
 3. tracing::info!(lua_source, lua_line, lua_fn, "{}", 文字列)
      target は Rust のモジュールパス pasta_lua::runtime::log
```

`table_to_string` は、表を JSON にする前に 2 つの関門を通す。

1. `pairs` で数えたキーと値の組が 1000（`MAX_TABLE_ELEMENTS`）を超えたら `<table: 要素数 elements>` を返す。
2. `table_depth_exceeds` が、明示的なスタックを使った反復で入れ子の深さを調べ、10（`MAX_NESTING_DEPTH`）を超えたら `tostring` の結果を返す。キーが表の場合もたどる。循環参照のある表は深さが増え続けるため、ここで止まる。

関門を通った表は、`deny_recursive_tables` を付けて `serde_json::Value` へ変換してから文字列にする。変換に失敗したとき（関数を含む表など）は `tostring` の結果を返し、`tostring` も失敗したら `"<unconvertible value>"` を返す。深さの関門を変換より前に置くのは、serde の変換が Rust の再帰で表をたどるため、深く入れ子にした表でホストのスタックが溢れるのを防ぐためである。

呼び出し元の段数 1 は、`crates/pasta_lua/tests/log/stack_level_test.rs` で確かめている（段数 0 は Rust の関数自身になる）。レベル関数は値の変換で Lua のエラーを出さず、常に何も返さずに戻る。

### `@enc` の変換

| 関数 | 処理 | 失敗の戻り値（`nil` と次のメッセージ） |
| ---- | ---- | -------------------------------------- |
| `to_ansi(s)` | 文字列でなければ失敗。Lua の文字列のバイト列を UTF-8 として検査し、`Encoding::ANSI.to_bytes` で変換して、結果のバイト列を Lua の文字列にする | `expected string, got 型名`・`invalid UTF-8 input: …`・`ANSI conversion failed: …` |
| `to_utf8(s)` | 文字列でなければ失敗。バイト列をそのまま `Encoding::ANSI.to_string` に渡し、結果を Lua の文字列にする | `expected string, got 型名`・`UTF-8 conversion failed: …` |

どちらも成功時は `(結果, nil)` を、失敗時は `(nil, メッセージ)` を返し、失敗時は warn ログを出す。Lua の文字列を作れないなど mlua 自体のエラーだけが Lua のエラーになる。

### OS 別の変換の実装

Windows の実装は、`Encoding` を `GetACP`（`ANSI`）か `GetOEMCP`（`OEM`）が返す実際のコードページに対応させる。コードページはシステムのロケール設定で決まる。

- `to_string`（コードページ → UTF-8）: `MultiByteToWideChar` を `MB_ERR_INVALID_CHARS` 付きで呼び、UTF-16 を `String::from_utf16` で UTF-8 にする。コードページとして不正なバイト列はエラーになる。
- `to_bytes`（UTF-8 → コードページ）: 文字列を UTF-16 にしてから `WideCharToMultiByte` を `WC_COMPOSITECHECK` 付きで呼ぶ。代替文字を指定せず、代替文字を使ったかどうかを受け取り、使っていたら（コードページで表せない文字があったら）`InvalidInput` のエラーにする。ただし、コードページが UTF-8（65001）のときは API を呼ばずに UTF-8 のバイト列をそのまま返す（65001 では `WC_COMPOSITECHECK` と代替文字の受け取りが許されないため）。
- どちらの API も、1 回目で必要な長さを問い合わせ、確保したバッファに 2 回目で書かせる。入力の長さは `buffer_len_to_i32` で `i32` に収まるか確かめ、2 GiB 以上はエラーにする（`as` で変換すると負の値になり、API が入力を NUL 終端として読み進めるため）。空の入力は API を呼ばずに空を返す。

Windows 以外の実装は、`to_bytes` で UTF-8 のバイト列をそのまま返し、`to_string` で `String::from_utf8` が通ればそのまま返す（不正な UTF-8 は `InvalidData` のエラー）。

### SHIORI 境界での文字コード

ホストと pasta.dll の間で受け渡す文字列の文字コードは次のとおりである。読み方と失敗時の応答の詳細は [SHIORI 層](shiori.md#処理とデータの流れ) で扱う。

| 経路 | 文字コード | 変換 |
| ---- | ---------- | ---- |
| `loadu` の設置パス | UTF-8 | `ShioriString::to_utf8_str` |
| `load` の設置パス | システムの ANSI コードページ（`CP_ACP`） | `ShioriString::to_ansi_str`（pasta_shiori 側の `MultiByteToWideChar`） |
| `request` のリクエスト | UTF-8 として読む（リクエストの `Charset` ヘッダは見ない） | `ShioriString::to_utf8_str` |
| 応答 | UTF-8（Rust の `String` のバイト列をそのまま HGLOBAL に写す） | なし。`Charset` ヘッダは `RES.env.charset`（既定 `"UTF-8"`）と Rust 側の固定の応答の `UTF-8` |

VM の中の文字列（DSL から生成したコード、`scripts/` の Lua ソース、リクエストの表）は UTF-8 である。ランタイムが Lua 側へ渡すファイルパスも UTF-8 のままで、Lua の標準入出力関数（`io.open` など）に ANSI のパスを渡す必要がある場合は、スクリプトが `@enc` で変換する。モジュールの検索はこの変換を要しない（[ローダ自己展開とモジュール解決](loader.md#処理とデータの流れ) の searcher）。

## 境界の受け渡し

| 境界 | 渡す側 → 受ける側 | 渡すもの | 所有 |
| ---- | ----------------- | -------- | ---- |
| SHIORI → ロギング | `PastaShiori::load` → `GlobalLoggerRegistry`・`init_tracing_with_reload` | 段階 1 の `Arc<PastaLogger>`（キーは設置パス）、既定の `LoggingConfig` | 登録簿が `Arc` を持つ。登録を外すのは `PastaShiori` |
| ローダ → ロギング | `PastaLoader::create_and_register_logger` → `GlobalLoggerRegistry`・`update_tracing_filter` | 段階 1.5 の `Arc<PastaLogger>`、`[logging]`（無ければ既定）の `LoggingConfig` | 同上 |
| ローダ → ランタイム | `PastaLoader` → `PastaLuaRuntime::from_loader_with_scene_dic` | 段階 1.5 の `Arc<PastaLogger>`（無ければ `None`） | ランタイムは参照を保持するだけで、書き込みには使わない |
| Rust → Lua | `crates/pasta_lua/src/runtime/module_registry.rs` → `package.loaded` | `@pasta_log`・`@enc` のモジュール表 | VM が持つ |
| Lua → Rust（`@pasta_log`） | Lua の呼び出し → tracing | 任意の値 1 つ（2 つ目以降は無視）。呼び出し元の情報は Rust 側がスタックから取る | 文字列化した結果だけがイベントに載る |
| Lua ↔ Rust（`@enc`） | Lua の文字列 ↔ `Encoding` | Lua の文字列のバイト列。UTF-8 かどうかを Lua 側は区別しない | 変換結果は新しい Lua の文字列 |
| ホスト ↔ pasta_shiori | FFI | 上表の文字コードのバイト列（HGLOBAL） | [SHIORI 層](shiori.md#境界の受け渡し) |

## 不変条件と制約

- tracing の購読者はプロセスに 1 つで、最初の `init_tracing_with_reload` だけが設置する。先に別の購読者が設置されていた場合、pasta の購読者は設置されず、`FILTER_HANDLE` も保存されないため、`update_tracing_filter` は何もしない。
- フィルタは購読者の 1 つの fmt レイヤに付いており、プロセス内のすべてのインスタンスで共有される。インスタンスごとに異なるフィルタは持てず、最後に差し替えたフィルタがすべてに効く。
- ログの振り分け先は、イベントを出したスレッドの `CURRENT_LOAD_DIR` と登録簿の中身だけで決まる（振り分けの 3 規則）。文脈のあるスレッドのログを別の設置パスのロガーへ書くことは無い。捨てるときもエラーにならない。
- 登録簿のキーは設置パスの `PathBuf` で、正規化せずに完全一致で比べる。段階 1 と段階 1.5 は同じ設置パスの値を使う。
- ログファイルのパスは、設置パスからの相対パスが `profile/` ディレクトリの下にあり `..` を含まないものに限られる。満たさない `file_path` は既定のログファイルに切り替わる。ファイルは追記で、ローテーションも切り詰めもしない。
- 書き込みは `tracing_appender::non_blocking` の既定の設定で、ワーカースレッドへの行のバッファが満ちたときは、呼び出し側を待たせずにその行を捨てる。
- `@pasta_log` と `@enc` は、不正な入力で Lua のエラーを出さない。`@pasta_log` は必ず文字列にして記録し、`@enc` は `(nil, メッセージ)` を返す。
- 文字コードの変換をするのは Windows だけである。Windows 以外では `Encoding` の両方の種類が UTF-8 のまま通す。
- `@enc` の結果は、変換を呼んだ時点のシステムの ANSI コードページに依存する。

## ソースの所在

- `crates/pasta_lua/src/logging/`
- `crates/pasta_lua/src/encoding/`
- `crates/pasta_lua/src/runtime/log.rs`
- `crates/pasta_lua/src/runtime/enc.rs`

テストは `crates/pasta_lua/tests/log/`・`crates/pasta_lua/tests/runtime/encoding_test.rs` にある。

本文で参照した、他の章の範囲にあるファイルは次のとおりである。`[logging]` の型は `crates/pasta_lua/src/loader/config/sections.rs`、段階 1.5 は `crates/pasta_lua/src/loader/mod.rs`、段階 1 とガード・`release_runtime` は `crates/pasta_shiori/src/shiori.rs`、アクタースレッドの入口のガードは `crates/pasta_shiori/src/actor/thread.rs`、`load_impl` のガードは `crates/pasta_shiori/src/windows.rs`、SHIORI 境界の変換は `crates/pasta_shiori/src/util/hglobal/` にある。

## 経緯

- [lua-logging](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/lua-logging) — `@pasta_log`（Lua からのロギング）
- [logger-configuration](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/logger-configuration) — `[logging]` によるフィルタの設定と差し替え
- [load-error-logging](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/load-error-logging) — 段階 1 のロガーによる起動時のログの記録と、固定のファイル名
- [pasta-toml-logging-consistency](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-toml-logging-consistency) — 文脈の無いログを唯一のロガーへ書く振り分け、不正な `file_path` の既定ファイルへのフォールバック、`rotation_days` の撤去

---

目立たぬ働きこそ、本当に大切なものですわ。フンッ、わたくしのように、ね。
これで内部設計の旅はひとめぐり。あとはあなたの手で、pasta をもっと良くしてくださいまし！
