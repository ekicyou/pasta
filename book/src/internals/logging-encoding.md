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
| `crates/pasta_lua/src/loader/config/sections.rs` | `LoggingConfig` | `[logging]` の型（`file_path`・`level`・`filter`・`rotation_days`）と、フィルタ文字列への変換 `to_filter_directive` |

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
     │  CURRENT_LOAD_DIR（スレッドローカル）で設置パスを得る
     │  loggers: Mutex<HashMap<設置パス, Arc<PastaLogger>>> を引く
     ▼
RoutingWriter ── 見つかった → PastaLogger::write → NonBlocking → ワーカースレッド → ログファイル
               └ 無い       → 何もせず書いたことにする（捨てる）
```

### エンコーディング

| ファイル | 主な型・関数 | 役割 |
| -------- | ------------ | ---- |
| `crates/pasta_lua/src/encoding/mod.rs` | `Encoder`・`Encoding`・`to_ansi_bytes`・`path_from_lua` | 変換のトレイトと、コードページの種類（`ANSI`・`OEM`）の列挙 |
| `crates/pasta_lua/src/encoding/windows.rs` | `Encoder for Encoding` | Windows の実装。`MultiByteToWideChar`・`WideCharToMultiByte` で変換する |
| `crates/pasta_lua/src/encoding/unix.rs` | `Encoder for Encoding` | Windows 以外の実装。UTF-8 のまま通す |
| `crates/pasta_lua/src/runtime/enc.rs` | `register`・`to_ansi_impl`・`to_utf8_impl` | `@enc` のモジュール表と 2 つの変換関数 |

どちらの OS の実装を使うかは `#[cfg(windows)]` でビルド時に決まる。`Encoder` と `Encoding` はクレートのルートから再エクスポートされる。`to_ansi_bytes` と `path_from_lua` も公開されているが、ワークスペース内に呼び出し元は無い。`Encoding::OEM`（`CP_OEMCP`）を使うのはテストだけである。

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
 6. PastaLogger::new(設置パス, [logging] または既定)
    成功 → 同じ設置パスで登録し直す（段階 1 のロガーを置き換える）
           [logging] があれば update_tracing_filter で FILTER_HANDLE のフィルタを差し替える
    失敗 → warn ログを出し、ロガー無しで続行（登録も、フィルタも変えない）
 7. 作ったロガーの Arc を PastaLuaRuntime へ渡す（段階 6 で構造体に保持）
```

- 段階 1 は、段階 1.5 より前（設定の読み込みを含む）のログもファイルに残すためにある。段階 1 のロガーは常に既定の場所 `profile/pasta/logs/pasta.log` に作られるため、`file_path` を変えたゴーストでも、段階 1.5 までのログはこのファイルに書かれる。
- 段階 1.5 で登録を置き換えると、段階 1 のロガーへの参照は登録簿から消え、最後の参照が落ちた時点で破棄される（破棄時にフラッシュする）。
- `PastaLoader` を SHIORI 以外から直接呼ぶ場合（テストなど）は段階 1 が無い。購読者を設置するかどうかは呼び出し側が決める。

ロガーの登録は、`PastaShiori` の `Drop` と再読み込みの前に外される。`PastaLuaRuntime` も同じロガーの `Arc` を持つため、ログファイルが閉じるのはランタイムの破棄の後である（[SHIORI 層](shiori.md#unloaddllmain-と-teardown)）。

### ログフィルタ

フィルタは tracing-subscriber の `EnvFilter` で、`build_filter` が次の順に作る。

1. 環境変数 `PASTA_LOG` が設定されていて解釈できれば、それを使う（未設定と解釈できない値は次へ進む）。
2. `LoggingConfig::to_filter_directive` の文字列（`filter` があればそれ、無ければ `level`。`level` の既定は `"info"`）を解釈できれば、それを使う。
3. どちらも使えなければ、標準エラーに警告を出して `"info"` にする。

段階 1 は既定の設定でこの順をたどり、段階 1.5 は `[logging]` があるときだけ、その設定で作り直したフィルタに差し替える。フィルタは `reload::Layer` で包まれており、購読者を設置し直さずに `FILTER_HANDLE` 経由で中身だけを入れ替える。

### ログ 1 件がファイルに届くまで

1. イベントがフィルタを通ると、fmt レイヤが 1 行に整形する（時刻・レベル・target・メッセージ・フィールド。ANSI の色付けはしない）。
2. fmt レイヤはイベントごとに `GlobalLoggerRegistry::make_writer` を呼ぶ。`make_writer` はそのスレッドの `CURRENT_LOAD_DIR` を読み、登録簿のミューテックスを取ってロガーを引き、`RoutingWriter` を返す。
3. `RoutingWriter` は、ロガーがあれば `PastaLogger::write` へ渡し、無ければ書いたバイト数を返して捨てる。
4. `PastaLogger::write` は、ミューテックスで守った `tracing_appender::non_blocking` の writer に書く。実際のファイル書き込みは、ロガーごとのワーカースレッドが行う。

`CURRENT_LOAD_DIR` を設定するのは `LoadDirGuard` で、ガードの破棄で元の値に戻す（入れ子にできる）。ガードを張るのは、アクタースレッド上の `PastaShiori` の `load`・`request`・`kick`・`unload` の呼び出しの間と、FFI 入口の `load_impl`（`load_entry` から呼ばれる）がロードの完了後に 1 行のログを出す間である（後者の理由は [SHIORI 層](shiori.md#load-とアクターの起動)）。

### ログファイル

`PastaLogger::new(基準ディレクトリ, 設定)` は次の順に進む。

1. `基準ディレクトリ.join(file_path)` をログファイルのパスにする（`file_path` の既定は `"profile/pasta/logs/pasta.log"`）。
2. `validate_path` で検査する。パスが基準ディレクトリの下にあり、基準ディレクトリからの相対パスが文字列として `profile` で始まり、`..` を含まないこと。満たさなければ `PermissionDenied` のエラーにする。
3. 親ディレクトリを作る（`create_dir_all`）。
4. `RollingFileAppender` を `Rotation::NEVER` とファイル名そのものを接頭辞にして作る。日付の接尾辞は付かず、ファイル名は `file_path` のとおりになる。既存のファイルには追記する。
5. `tracing_appender::non_blocking` でワーカースレッドと `WorkerGuard` を作り、`PastaLogger` が保持する。

`PastaLogger` の `Drop` はフラッシュし、続く `WorkerGuard` の破棄がワーカースレッドに残りを書かせる。`LoggingConfig` の `rotation_days` は既定値 7 で読み込まれるが、どこからも読まれず、ログファイルはローテーションされない。

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

Windows の実装は、`Encoding` を `CP_ACP`（`ANSI`）か `CP_OEMCP`（`OEM`）に対応させる。コードページはシステムのロケール設定で決まる。

- `to_string`（コードページ → UTF-8）: `MultiByteToWideChar` を `MB_ERR_INVALID_CHARS` 付きで呼び、UTF-16 を `String::from_utf16` で UTF-8 にする。コードページとして不正なバイト列はエラーになる。
- `to_bytes`（UTF-8 → コードページ）: 文字列を UTF-16 にしてから `WideCharToMultiByte` を `WC_COMPOSITECHECK` 付きで呼ぶ。代替文字を指定せず、代替文字を使ったかどうかを受け取り、使っていたら（コードページで表せない文字があったら）`InvalidInput` のエラーにする。
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
| ローダ → ロギング | `PastaLoader::create_and_register_logger` → `GlobalLoggerRegistry`・`update_tracing_filter` | 段階 1.5 の `Arc<PastaLogger>`、`[logging]` の `LoggingConfig` | 同上 |
| ローダ → ランタイム | `PastaLoader` → `PastaLuaRuntime::from_loader_with_scene_dic` | 段階 1.5 の `Arc<PastaLogger>`（無ければ `None`） | ランタイムは参照を保持するだけで、書き込みには使わない |
| Rust → Lua | `crates/pasta_lua/src/runtime/module_registry.rs` → `package.loaded` | `@pasta_log`・`@enc` のモジュール表 | VM が持つ |
| Lua → Rust（`@pasta_log`） | Lua の呼び出し → tracing | 任意の値 1 つ（2 つ目以降は無視）。呼び出し元の情報は Rust 側がスタックから取る | 文字列化した結果だけがイベントに載る |
| Lua ↔ Rust（`@enc`） | Lua の文字列 ↔ `Encoding` | Lua の文字列のバイト列。UTF-8 かどうかを Lua 側は区別しない | 変換結果は新しい Lua の文字列 |
| ホスト ↔ pasta_shiori | FFI | 上表の文字コードのバイト列（HGLOBAL） | [SHIORI 層](shiori.md#境界の受け渡し) |

## 不変条件と制約

- tracing の購読者はプロセスに 1 つで、最初の `init_tracing_with_reload` だけが設置する。先に別の購読者が設置されていた場合、pasta の購読者は設置されず、`FILTER_HANDLE` も保存されないため、`update_tracing_filter` は何もしない。
- フィルタは購読者の 1 つの fmt レイヤに付いており、プロセス内のすべてのインスタンスで共有される。インスタンスごとに異なるフィルタは持てず、最後に差し替えたフィルタがすべてに効く。
- ログの振り分け先は、イベントを出したスレッドの `CURRENT_LOAD_DIR` だけで決まる。設置パスが設定されていないスレッド、または登録の無い設置パスのスレッドで出たイベントは、エラーにならずに捨てられる。
- 登録簿のキーは設置パスの `PathBuf` で、正規化せずに完全一致で比べる。段階 1 と段階 1.5 は同じ設置パスの値を使う。
- ログファイルのパスは、設置パスからの相対パスが `profile` で始まり `..` を含まないものに限られる。ファイルは追記で、ローテーションも切り詰めもしない。
- 書き込みは `tracing_appender::non_blocking` の既定の設定で、ワーカースレッドへの行のバッファが満ちたときは、呼び出し側を待たせずにその行を捨てる。
- `@pasta_log` と `@enc` は、不正な入力で Lua のエラーを出さない。`@pasta_log` は必ず文字列にして記録し、`@enc` は `(nil, メッセージ)` を返す。
- 文字コードの変換をするのは Windows だけである。Windows 以外では `Encoding` の両方の種類が UTF-8 のまま通す。
- `@enc` の結果は、変換を呼んだ時点のシステムの ANSI コードページに依存する。

## ソースの所在

- `crates/pasta_lua/src/logging/`
- `crates/pasta_lua/src/encoding/`
- `crates/pasta_lua/src/runtime/log.rs`
- `crates/pasta_lua/src/runtime/enc.rs`
- テスト: `crates/pasta_lua/tests/log/`・`crates/pasta_lua/tests/runtime/encoding_test.rs`

本文で参照した、他の章の範囲にあるファイルは次のとおりである。`[logging]` の型は `crates/pasta_lua/src/loader/config/sections.rs`、段階 1.5 は `crates/pasta_lua/src/loader/mod.rs`、段階 1 とガードは `crates/pasta_shiori/src/shiori.rs`、`load_impl` のガードは `crates/pasta_shiori/src/windows.rs`、SHIORI 境界の変換は `crates/pasta_shiori/src/util/hglobal/` にある。

## 経緯

- [lua-logging](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/lua-logging) — `@pasta_log`（Lua からのロギング）
- [logger-configuration](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/logger-configuration) — `[logging]` によるフィルタの設定と差し替え
- [load-error-logging](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/load-error-logging) — 段階 1 のロガーによる起動時のログの記録と、固定のファイル名

---

目立たぬ働きこそ、本当に大切なものですわ。フンッ、わたくしのように、ね。
これで内部設計の旅はひとめぐり。あとはあなたの手で、pasta をもっと良くしてくださいまし！
