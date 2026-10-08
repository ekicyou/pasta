# ローダ自己展開とモジュール解決

> 【目閉じ】目を覚ました pasta.dll が最初のトークを話すまでには、きちんと決まった段取りがございますの。設定読込、同梱スクリプトの自己展開、`require` の解決――その舞台裏を順にお見せいたしますわ。

> 【アンソニー】荷ほどきと申しますと、同梱のスクリプトを起動のたびに広げ直すのでございますか。

> 【高笑い】毎回ではございませんわ！ 展開済みの版が同梱のものと同じなら、ほどきもせずに素通りいたしますのよ！

---

この章では、起動シーケンスの実装、フレームワークスクリプトの自己展開、ファイル検出とモジュール名の生成、モジュール解決、設定読込を扱う。

## 目的と責務

ローダ（`PastaLoader`）は、ゴーストの設置ディレクトリ（以下、基準ディレクトリ）を受け取り、Lua VM が最初のリクエストに応答できる状態の `PastaLuaRuntime` を返すまでの段階を順に実行する。

この章が責務を持つのは次の事項である。

- 起動シーケンスの段階構成と、各段階の失敗が致命か継続か
- 設定読込（`pasta.toml` の読込、`[loader]` とそれ以外のセクションの分離、既定値の補完）
- フレームワークスクリプトの埋め込み（ビルド時の zip 化）と、起動時の自己展開・版比較
- `.pasta`・`.lua` のファイル検出と、モジュール名の生成
- モジュール検索パス（`package.path`）の組立と、`require` のファイル解決を担う searcher の置換

利用者から見た起動シーケンス・モジュール検索パスの優先順位・失敗時の示され方は [起動シーケンスとモジュール解決](../reference/startup.md) が正であり、この章はそれを実現する仕組みだけを書く。`pasta.toml` の各キーの意味と既定値は [pasta.toml リファレンス](../reference/pasta-toml.md) が正である。

ほかの章へ委ねる事項は次のとおりである。

- 増分トランスパイル（更新判定・キャッシュへの保存・`scene_dic.lua` の生成）: [トランスパイル結果キャッシュ](transpiler.md#トランスパイル結果キャッシュ)
- ローダが渡した材料から VM を組み立てる手順（モジュール登録と起動モジュールの `require`）: [VM の構築とモジュール登録](execution-model.md#vm-の構築とモジュール登録)
- インスタンスロガーの作成とログフィルタ: [ロギングとエンコーディング](logging-encoding.md)
- デバッグ有効時のソースマップ構築: [デバッグ基盤とシーンキック](debug.md)
- SHIORI の `load` からローダを呼ぶまでと、ロード失敗の応答への変換: [SHIORI 層](shiori.md)

## 構成要素

| 要素 | 所在 | 役割 |
| ---- | ---- | ---- |
| `PastaLoader` | `crates/pasta_lua/src/loader/mod.rs` | 起動シーケンスの本体。`load`（既定の `RuntimeConfig`）と `load_with_config`（呼び出し側が渡す `RuntimeConfig`）を持つ。ディレクトリの準備も行う |
| `PastaConfig`・`LoaderConfig` | `crates/pasta_lua/src/loader/config/mod.rs` | `pasta.toml` の読込と解析。`[loader]` を型付きの `LoaderConfig` に、それ以外を `custom_fields`（TOML の表）に分ける。SHIORI 用の既定値を補完する |
| 型付きの設定セクション | `crates/pasta_lua/src/loader/config/sections.rs` | `LoggingConfig`・`PersistenceConfig`・`TalkConfig`・`GhostConfig`・`DebugFileConfig` と各既定値の関数 |
| ビルドスクリプト | `crates/pasta_lua/build.rs` | `crates/pasta_lua/pasta_scripts/` を zip に固めて `OUT_DIR` に書き出し、その MD5 を環境変数 `PASTA_SCRIPTS_MD5` としてコンパイルに渡す |
| 決定論的 zip パッカー | `crates/pasta_lua/build_zip.rs` | ソースツリーから毎回バイト単位で同じ zip を作る純粋な関数。ビルドスクリプトとビルド決定論テスト（`crates/pasta_lua/tests/build_determinism_test.rs`）が共有する |
| 自己展開 | `crates/pasta_lua/src/loader/extract.rs` | `sync_pasta_scripts`。埋め込んだ zip と MD5 を定数として持ち、版を比較して必要なら展開する |
| ファイル検出 | `crates/pasta_lua/src/loader/discovery.rs` | `discover_files`。glob パターンでファイルを集め、基準ディレクトリの外・シンボリックリンク・`profile/` 配下を除く |
| 検出の統合と増分処理 | `crates/pasta_lua/src/loader/process.rs` | `discover_all_files` が `.pasta` と `.lua` を検出し、禁止ファイル名と同名衝突を処理する。増分処理（`process_incremental`）は [トランスパイル結果キャッシュ](transpiler.md#トランスパイル結果キャッシュ) で扱う |
| `CacheManager` | `crates/pasta_lua/src/loader/cache.rs` | モジュール名とキャッシュ先の導出（`source_to_module_name`・`source_to_cache_path`）。版管理と `scene_dic.lua` の生成はトランスパイル章で扱う |
| `LoaderContext` | `crates/pasta_lua/src/loader/context.rs` | ローダからランタイムへ渡す材料（絶対パスにした基準ディレクトリ・検索パス・`custom_fields`）。`package.path` の文字列を組み立てる |
| `LoaderError` | `crates/pasta_lua/src/loader/error.rs` | 起動シーケンスのエラー。設定・入出力・検出・キャッシュ・自己展開・トランスパイルの部分失敗・ランタイムの構築失敗を区別する |
| `setup_package_path` | `crates/pasta_lua/src/runtime/module_registry.rs` | `package.path` を設定し、searcher を置換する。`@pasta_config` の登録（`register_config_module`）も同じファイルにある |
| searcher | `crates/pasta_lua/src/runtime/searcher.rs` | `install_module_searcher`。`package.loaders` の 2 番目（Lua ファイルの searcher）を Rust の実装に置き換える |
| `RuntimeConfig`・`lua_require` | `crates/pasta_lua/src/runtime/runtime_config.rs` | VM に入れる標準ライブラリと mlua-stdlib のモジュール、解決済みのデバッグ設定、キックの注入口を持つ設定。`lua_require` は Rust から Lua の `require` を呼ぶ |
| 既定の `main` | `crates/pasta_lua/pasta_scripts/main.lua` | 空の表を返すだけの利用者初期化スクリプト。自己展開され、`scripts/main.lua` があればそちらが優先される |
| `pasta.config` | `crates/pasta_lua/pasta_scripts/pasta/config.lua` | `@pasta_config` を `pcall(require, …)` で取り込み、`get(section, key, default)` で値か既定値を返す Lua 側のラッパ。`@pasta_config` を取得できない環境（テストなど）では空の表を使う |

## 処理とデータの流れ

### 起動の段階構成

`PastaLoader::load_with_config` は次の順に進む。段階の番号はコードのログ（`Phase N`・`Stage 1.5`）に合わせている。

```text
load_with_config(base_dir, runtime_config)
 0.   基準ディレクトリの存在確認                         … 無ければ DirectoryNotFound（致命）
 1.   PastaConfig::load                                   … pasta.toml の読込と既定値の補完（致命）
 1.5  インスタンスロガーの作成と登録・ログフィルタの更新   … 失敗は既定のログファイルへ切り替えて続行（継続）
      別名表の info ログ（Scene alias table）
 2.   profile/ 配下のディレクトリの作成・キャッシュの版と別名表の確認 … 致命
 2.5  sync_pasta_scripts（フレームワークスクリプトの自己展開） … 失敗は ERROR ログで続行（継続）
 3.   discover_all_files（.pasta と .lua の検出）           … glob の誤り・禁止ファイル名は致命
 4.   process_incremental（トランスパイルと .lua のコピー）  … 1 件でも失敗すれば致命
      孤立キャッシュの検出（警告のみ。削除しない）
 5.   generate_scene_dic（scene_dic.lua の生成）            … 致命
 5.5  （デバッグ有効時だけ）ソースマップの構築               … ファイル単位の失敗は飛ばす
 6.   LoaderContext::from_config → PastaLuaRuntime::from_loader_with_scene_dic … 失敗は Runtime（致命）
```

- 0 の直後に、基準ディレクトリで `LoadDirGuard` を張り、`load_with_config` を抜けるまで保つ。組み込みで複数のゴーストを読んでも、ローダのログは自分のロガーへ届く（[ロギングとエンコーディング](logging-encoding.md#振り分けの規則)）。
- 1.5 で `[logging]` の `file_path` からロガーを作れないとき（`profile/` ディレクトリの下にないなど）は、既定のログファイル `profile/pasta/logs/pasta.log` のロガーに切り替えて登録し、不正と判断した `file_path` を warn に残して続ける。`level`・`filter` は反映される（[ロギングとエンコーディング](logging-encoding.md#ロガーの-2-段階の初期化)）。
- 1 の `PastaConfig::load` が決めた別名表（[設定読込](#設定読込)）は、1.5 でロガーを登録した直後に info ログ `Scene alias table` として出す。`source` は表の出どころ（`BuiltinDefault` は既定の表、`PastaToml` は `[scene.alias]` に書いた表）、`table` は表の中身（`OnTalk <- 会話` の形。空の表は `(empty)`）である。ロガーの登録より後に出すため、ゴーストのログファイルに残る。
- 2 で作るディレクトリは `profile/pasta/save`・`profile/pasta/save/lua`・`profile/pasta/cache` と `[loader] transpiled_output_dir` である。続けて `CacheManager::prepare_cache_dir` がキャッシュの版と別名表の指紋を確認し、どちらかが違えばキャッシュを全破棄する（[トランスパイル結果キャッシュ](transpiler.md#トランスパイル結果キャッシュ)）。
- 2.5 を 3 より前に置くのは、検出とトランスパイルより前、`package.path` を組む 6 より前に、フレームワークスクリプトをディスク上で最新にしておくためである。
- 4 の統計（トランスパイル・省略・失敗・コピーの件数）は、`[loader] debug_mode` が `true` のときだけ info ログに出る。孤立キャッシュは `CacheManager::find_orphaned_caches` が `debug_mode` に関わらず件数と各パスを警告し、`debug_mode` が `true` のときはローダが各パスを重ねて警告する。キャッシュへの保存は `debug_mode` に関わらず行う。
- 4 と 5.5 は、別名表を持たせた同じ 1 つのトランスパイラ（`transpiler_for`）を使う。キャッシュの `.lua` の宣言名と、ソースマップ・デバッグの突合キーの宣言名は、常に同じ別名表から決まる。6 では同じ表を `RuntimeConfig.scene_aliases` に入れて `@pasta_search` へ渡すため、宣言と検索にも同じ表が効く（[シーン・単語レジストリとシーン検索](registry-search.md#辞書確定)）。
- 5.5 の要否は、6 で VM が使うのと同じ判定（`pasta.toml` の `[debug]` と環境変数を `DebugConfig::from_env` で解決した結果）で決める。無効なら何も作らない。
- 6 の中で VM を作り、モジュールを登録し、`main`・`pasta.shiori.entry`・`pasta.scene_dic` を `require` する。その順序と失敗の扱いは [VM の構築とモジュール登録](execution-model.md#vm-の構築とモジュール登録) が書く。

どの段階の `LoaderError` も `load_with_config` の戻り値としてそのまま返り、SHIORI 層がロード失敗として扱う。利用者から見た失敗の示され方は [起動シーケンス](../reference/startup.md#2-起動シーケンス) を参照する。

### 設定読込

`PastaConfig::load` は基準ディレクトリ直下の `pasta.toml` を読む。ファイルが無ければ `ConfigNotFound`、読めなければ `Io`、TOML として解析できなければ `Config` を返し、いずれも起動を止める。

解析（`PastaConfig::parse`）は次の順に行う。

1. 全体を TOML の表として読む。
2. `loader` キーを表から取り除き、`LoaderConfig` へ変換する。各キーは serde の既定値の関数で補われ、型の合わない値は解析エラーになる。`loader` キーが無ければ `LoaderConfig::default()` を使う。
3. `[scene.alias]` を同じ内容から読み直し、有効な別名表（`scene_aliases`）とその出どころ（`scene_alias_source`）を決める。`[scene.alias]` が無ければ既定の表（`OnTalk = ["会話"]`）、あれば書いた表をそのまま使う（既定の表と合わせない）。型の合わない値・空の名前・別名の重複・置き換え先を別名にも書いた表は解析エラーになり、起動を止める（利用者向けの説明は [pasta.toml の `[scene]`](../reference/pasta-toml.md#sceneシーン名)）。
4. 残りの表をすべて `custom_fields` とする。`[scene]` も書いたとおりに残り、既定の別名表は書き足さない。`[loader]` 以外のセクションはこの段階では型付きの値にしない。
5. `apply_shiori_defaults` で SHIORI 用の既定値を補完する。

`apply_shiori_defaults` は `custom_fields` の `[ghost]` に、書かれていないキー（`talk_interval_min`・`talk_interval_max`・`hour_margin`・`spot_newlines`）だけを `GhostConfig::default()` の値で書き足す。`[ghost]` が無ければ表を作って 4 キーとも入れる。書かれている値は上書きしないため、2 回適用しても結果は変わらない。あわせて `[actor]` が表として存在しなければ警告を 1 行出す（起動は止めない）。補完をここ 1 か所でだけ行うので、Rust 側の消費者と Lua 側（`@pasta_config`）は同じ補完後の値を見る。

`[ghost]` 以外のセクションは、使う側が必要になった時点でアクセサ（`logging`・`persistence`・`talk`・`debug`）を呼び、`custom_fields` の該当セクションを型付きの構造体へ変換する。セクションが無いとき、または変換に失敗したときは `None` を返し、使う側が既定値を使う。型の合わない値を含むセクションが丸ごと既定値になるのはこのためである（利用者向けの説明は [値の型が合わないとき](../reference/pasta-toml.md#値の型が合わないとき)）。

| アクセサ | 使う場所 |
| -------- | -------- |
| `logging` | ローダの段階 1.5 |
| `debug` | ローダの段階 5.5 と VM の構築（`from_loader_with_scene_dic`） |
| `persistence` | `@pasta_persistence` の登録と、ランタイムの破棄時の保存 |
| `talk` | `@pasta_sakura_script` の登録 |

`custom_fields` は `LoaderContext` に複製されてランタイムへ渡り、`register_config_module` が Lua の表へ変換して `@pasta_config` として `package.loaded` に置く。変換では TOML の整数・浮動小数は Lua の数値に、日時は文字列になり、`[actor]` の各サブテーブルにはキー名を `name` として書き込む。作る表は普通の Lua の表である。Lua 側のスクリプトは `@pasta_config` を直接 `require` するか、`pasta.config` の `get` を通して読む。`[loader]` は `custom_fields` に含まれないため Lua からは見えない。利用者から見た `@pasta_config` は [@pasta_config](../lua/modules/pasta-config.md) が正である。

### フレームワークスクリプトの埋め込み

フレームワークスクリプト（`crates/pasta_lua/pasta_scripts/` のツリー全体）は、ビルド時に zip へ固めて DLL に埋め込む。

1. `crates/pasta_lua/build.rs` が `build_zip::collect` でツリーをたどり、ディレクトリとファイルのすべてに `cargo:rerun-if-changed` を出す。入れ子のファイルの変更も再ビルドの契機になる。
2. `build_zip::build_deterministic_zip` が zip を作る。エントリ名はルートからの相対パスを `/` 区切りにしたもので、名前順に格納する。更新時刻は固定値、Unix 権限は `0o644`、圧縮は Deflate の固定レベル（6）である。シンボリックリンクは格納しない。同じ内容のツリーからは常に同じバイト列ができる。
3. ビルドスクリプトは zip を `OUT_DIR/pasta_scripts.zip` に書き出し、zip のバイト列の MD5 を `PASTA_SCRIPTS_MD5` として公開する。
4. `crates/pasta_lua/src/loader/extract.rs` が `include_bytes!` で zip を、`env!` で MD5 を、それぞれコンパイル時の定数（`EMBEDDED_ZIP`・`EXPECTED_MD5`）として取り込む。

MD5 はツリーのどのファイルの内容が変わっても変わる。クレートの版とは連動しない（トランスパイル結果キャッシュの版がクレートの版で決まるのと対照的である）。

### 自己展開と版比較

`sync_pasta_scripts` は自己展開先 `profile/pasta/pasta_scripts/` を埋め込みの zip に合わせる。

```text
sync_pasta_scripts(base_dir)
  marker = profile/pasta/pasta_scripts/.md5 を読む
  marker を trim した文字列 == EXPECTED_MD5 ?
    はい   → 何も書かずに Skipped（DEBUG ログ）
    いいえ → deploy（マーカーが無い・読めない場合も含む）
```

版比較はマーカーの文字列と定数の比較だけで行い、展開済みのファイルを読み直してハッシュを取ることはしない。

`deploy` は準アトミックに展開する。一時ディレクトリと退避ディレクトリは、同じボリュームに置くため自己展開先の親（`profile/pasta/`）の下に作る。名前はプロセス ID・時刻（ナノ秒）・プロセス内のカウンタを連結した接尾辞で区別する。

```text
1. profile/pasta/.pasta_scripts.new.<接尾辞>/ に全エントリを展開する（自己展開先にはまだ触れない）
2. 自己展開先があれば profile/pasta/.pasta_scripts.old.<接尾辞>/ へ rename して退避する
3. 一時ディレクトリを自己展開先へ rename する（失敗したら退避を戻す）
4. 退避ディレクトリを削除する（失敗しても無視する）
5. .md5 マーカーに EXPECTED_MD5 を書き込む
```

- 1〜3 のどこで失敗しても、自己展開先を直前の状態に戻し、一時ディレクトリを削除して `LoaderError::SelfDeploy` を返す。
- マーカーは最後に書く。3 と 5 の間で中断した場合はマーカーが古いまま残り、次回の起動で再び展開される。
- 展開では zip のエントリ名を `enclosed_name` で検査し、展開先の外を指す名前を飛ばす。ファイルは解凍した生のバイト列として書き出す。
- 自己展開先はディレクトリごと置き換わる。自己展開先に置かれた、zip に含まれないファイルは、次に展開が起きたときに消える。

段階 2.5 は `SelfDeploy` を含むすべてのエラーを ERROR ログに記録して捨て、ディスク上にある既存のスクリプトのまま起動を続ける。

### ファイル検出

`discover_all_files` は `[loader] pasta_patterns` から `.pasta` と `.lua` を検出する。パターンの書き方と既定値は [pasta_patterns](../reference/pasta-toml.md#pasta_patterns) が正である。

1. `discover_files(base_dir, pasta_patterns)` で `.pasta` を集める。0 件なら警告を出して続ける。
2. `.pasta` で終わる各パターンの末尾を `.lua` に替えたパターンを作る。`.pasta` で終わらないパターンは警告を出して飛ばす。
3. 作ったパターンで `discover_files` を呼び、`.lua` を集める。ここでのエラーは警告にとどめ、`.lua` は 0 件として扱う。0 件でも警告は出さない。
4. `.pasta` と `.lua` のどちらかにファイル名が `init.lua` または `init.pasta` のものがあれば、`LoaderError::InvalidFileName` で起動を止める。
5. `.pasta` のモジュール名（後述の `source_to_module_name`）の集合を作り、同じモジュール名の `.lua` を警告付きで除く。

`discover_files` は各パターンについて次を行う。

- パターンが `..`・ルート・ドライブ接頭辞を含むなら、警告を出してそのパターンを使わない。
- 基準ディレクトリとパターンを連結した文字列を `glob` クレートに渡す。基準ディレクトリの部分は `glob::Pattern::escape` でエスケープするため、設置先のパスに `[`・`]`・`*`・`?` が含まれてもそのまま一致する。パターンの誤りと走査中のエラーは `LoaderError` として返る。
- 一致したパスのうち、基準ディレクトリの外にあるもの、基準ディレクトリからの途中にシンボリックリンク（Windows のジャンクションを含む）を含むもの、メタデータを読めないもの、`profile/` 配下にあるものを除く。

返す一覧はパターンの順・glob の列挙順に並び、重複は取り除かない。

### モジュール名の生成

検出したファイルは、基準ディレクトリからの相対パスで `pasta.scene.*` のモジュール名とキャッシュ先が決まる。`CacheManager` が 2 つを導出する。

| 導出 | 手順 |
| ---- | ---- |
| `source_to_module_name` | 相対パスの先頭の `dic` を除く（パス要素の境界で一致する場合だけ）→ 拡張子を除く → `.`・`-` を `_` に置き換える → `/`・`\` を `.` に置き換える → 先頭に `pasta.scene.` を付ける |
| `source_to_cache_path` | `source_to_module_name` のモジュール名の `.` を `/` に置き換え、末尾に `.lua` を付けて `<キャッシュ>/` の下に置く |

```text
dic/baseware/system.pasta
  モジュール名:  pasta.scene.baseware.system
  キャッシュ先:  <キャッシュ>/pasta/scene/baseware/system.lua
dic/会話.pasta
  モジュール名:  pasta.scene.会話
  キャッシュ先:  <キャッシュ>/pasta/scene/会話.lua
dic/v1.2.pasta
  モジュール名:  pasta.scene.v1_2
  キャッシュ先:  <キャッシュ>/pasta/scene/v1_2.lua
```

- `.lua` も `.pasta` と同じ規則で名前とキャッシュ先が決まる。拡張子が違うだけの同名ファイルは同じモジュール名になるため、検出の手順 5 で `.lua` を除く。
- 非 ASCII の文字はそのまま残す。モジュール名を解決する searcher も UTF-8 のまま扱う（次節）。
- `dic` を除くのは、`dic` がパス要素として完結する場合だけである。`dictionary.pasta` や `dicx/foo.pasta` はそのまま残る。
モジュール名の一覧は増分処理で集められ、`scene_dic.lua` が並べ替えて `require` する（[トランスパイル結果キャッシュ](transpiler.md#トランスパイル結果キャッシュ)）。`require("pasta.scene.baseware.system")` は、次節の searcher が検索パスの `profile/pasta/cache/lua` から `pasta/scene/baseware/system.lua` を見つけることで解決する。searcher はモジュール名の `.` をすべてパス区切りに戻すため、キャッシュ先はモジュール名から同じ規則で導出し、ファイル名やディレクトリ名に含まれる `.` はモジュール名の時点で `_` に置き換えておく。

### モジュール検索パス

`LoaderContext::from_config` は基準ディレクトリを `std::path::absolute` で絶対パスにし、`[loader] lua_search_paths` と `custom_fields` を写す。`canonicalize` を使わないのは、8.3 短縮名やシンボリックリンクを解決すると、`package.path` から決まる実行時のチャンク名と、ローダが元の基準ディレクトリから作るキャッシュのパス・ソースマップのキーが食い違うためである（デバッガのブレークポイントの照合がこの一致に依存する）。Windows では `std::path::absolute` は `.`・`..` を字句的に解決し、`\\?\` の拡張長接頭辞も付けない。

`LoaderContext::generate_package_path` は、検索パスの各要素を基準ディレクトリに連結し、区切りを `/` に正規化して、要素ごとに 2 つのテンプレートを宣言順に並べる。

```text
lua_search_paths = ["scripts", "profile/pasta/pasta_scripts"]
package.path = <基準>/scripts/?.lua;<基準>/scripts/?/init.lua;<基準>/profile/pasta/pasta_scripts/?.lua;<基準>/profile/pasta/pasta_scripts/?/init.lua
```

`setup_package_path` はこの文字列を変換せずに `package.path` へ上書きで設定し、続けて `install_module_searcher` を呼ぶ。`package.cpath` は変えない。呼び出しは VM を作って `@pasta_search` などを登録した直後、`@pasta_config` 以降の登録と最初の `require` より前に 1 回だけ行われる（[VM の構築とモジュール登録](execution-model.md#vm-の構築とモジュール登録) の手順 3）。

### require の解決（searcher）

LuaJIT の `package.loaders` は preload・Lua ファイル・C ライブラリ・C ライブラリのルートの 4 つの searcher を持つ。`install_module_searcher` はこのうち 2 番目（Lua ファイル）だけを Rust の実装に置き換える。`package.loaders` が 4 要素でなければ置き換えずに `Err` を返し、VM の構築を失敗させる（mlua・LuaJIT の更新で並びが変わったときに、別の searcher を黙って上書きしないための検査である）。`package.searchers` は LuaJIT では同じ表なので、両方の名前から置き換えが見える。

置き換えた searcher は、LuaJIT 標準の `searchpath`・`loader_Lua` と次の点で同じに保つ。

- 候補の作り方: モジュール名の `.` を OS の区切り（Windows では `\`）に置き換え、`package.path` を `;` で分けた各テンプレート（空の要素は飛ばす）の `?` をすべてその名前に置き換える。テンプレートの順がそのまま探索順である。
- 最初に開けた候補を採用する。開けなかった理由（不在・権限・不正なパス）は区別しない。
- チャンク名は `@` に候補のパスを付けたもの。
- 未検出なら `\n\tno file '<候補>'` を候補の順に連結した文字列を返し、`require` がほかの searcher の結果と合わせて `module '…' not found:` のエラーにする。
- 開けたが読めない、または構文エラーなら、`error loading module '…' from file '…':` に原因を続けたメッセージで失敗する。構文エラーの本文には mlua が付ける `syntax error: ` の接頭辞を付けない。

標準との違いは 2 点である。

- ファイルを開くのに C の `fopen` ではなく Rust の `std::fs::File` を使う。Windows では Rust の標準ライブラリがワイド文字の API を使い、長い絶対パスには拡張長の接頭辞を内部で付けるため、パスの長さや ANSI コードページに無い文字に左右されない。searcher 自身はパス長を判定せず、接頭辞の付与・短縮名の変換・`canonicalize` もしない。
- `package.path` とモジュール名を UTF-8 として解釈する（不正なバイト列は置換文字に変換され、その候補は開けずに `no file` の行に載る）。

本体は 2 層になっている。Rust 関数 `find_module` は `(ローダ, メッセージ, ロード失敗か)` の組を返すだけにし、エラーの送出は Lua のラッパ（チャンク名 `=pasta_searcher`）が `error(message, 0)` で行う。Rust の `Err` をそのまま返すと `pcall(require, …)` が受け取るエラーが文字列ではなくユーザーデータになり、位置情報の付かない標準の文言とも一致しなくなるためである。

Rust 側で登録するモジュール（`@pasta_config`・`@pasta_search` など）は `package.loaded` に直接置くため、`require` は searcher を通らずにそれを返す。利用者から見た検索の優先順位・候補パスの表記・長パスと非 ANSI パスの保証は [モジュール検索パス](../reference/startup.md#1-モジュール検索パス) が正である。

## 境界の受け渡し

| 境界 | 渡す側 → 受け取る側 | 渡すもの | 所有 |
| ---- | ------------------- | -------- | ---- |
| ビルド → 実行時 | `crates/pasta_lua/build.rs` → `extract.rs` | `OUT_DIR/pasta_scripts.zip` と `PASTA_SCRIPTS_MD5` | 正本はソースツリー。DLL はコンパイル時の定数として 1 版だけを持つ |
| DLL → ゴーストのディレクトリ | 自己展開 → ディスク | `profile/pasta/pasta_scripts/` のファイルと `.md5` マーカー | 自己展開先とその隣の一時・退避ディレクトリだけを操作する。`scripts/` やほかのファイルには触れない |
| SHIORI 層 → ローダ | `pasta_shiori` → `PastaLoader::load_with_config` | 基準ディレクトリと `RuntimeConfig`（キックの注入口を含む） | 基準ディレクトリの決定とロード失敗の応答は SHIORI 層が持つ（[SHIORI 層](shiori.md)） |
| ローダ → ランタイム | `PastaLoader` → `PastaLuaRuntime::from_loader_with_scene_dic` | 統合した `TranspileContext`・`LoaderContext`・`RuntimeConfig`・`PastaConfig`・ロガー・`scene_dic.lua` のパス・ソースマップ | `PastaConfig` はランタイムが保持し、破棄時の保存などで使う |
| Rust → Lua（設定） | `register_config_module` → `@pasta_config` | `custom_fields` を変換した表 | 表は VM が所有する。`pasta.toml` へは書き戻さない |
| Rust → Lua（解決） | `setup_package_path` → `package.path`・`package.loaders[2]` | 検索パスの文字列と searcher | 以後の `require` のファイル解決は Rust の searcher が担い、モジュールの読み込みと実行は Lua が担う |

## 不変条件と制約

- 段階の順序は固定である。自己展開（2.5）は検出（3）とランタイムの構築（6）より前に終わり、`package.path` の設定と searcher の置換は最初の `require` より前に終わる。
- 起動を止めずに警告やエラーのログだけを残す失敗は、ロガーの作成（1.5）、自己展開（2.5）、`.lua` の検出エラー、`.pasta` のキャッシュへの保存、ソースマップの構築（5.5）、シーン identity 索引の突合である。段階表で致命とした失敗は `LoaderError` として起動を止める。
- `.pasta`・`.lua` の読込・パース・トランスパイル・コピーの失敗は、全ファイルを処理し終えてから `PartialTranspileError` にまとめる。
- 自己展開の版比較はマーカーの文字列だけで行う。自己展開先のファイルを直接書き換えても、埋め込みの MD5 が変わるまで元に戻らない。フレームワークスクリプトを差し替える経路は、検索パスで先に来る `scripts/` に同名のファイルを置くことである。
- 自己展開先は `profile/pasta/pasta_scripts` に固定されており、`lua_search_paths` からは独立している。検索パスからこのディレクトリを外すと、展開は行われてもフレームワークスクリプトは解決されない。
- キャッシュの出力先（`transpiled_output_dir`）と検索パスも独立している。`pasta.scene.*` と `pasta.scene_dic` は検索パスを通して解決されるため、出力先を変えるときは検索パスにもそれを含める必要がある。また、検索パスで先に来るディレクトリに同じ相対パスのファイルがあれば、そちらが解決される。
- ファイル検出は `profile/` 配下を常に除く。キャッシュや自己展開先が検出の対象になることはない。
- 設定読込を失敗させるのは、`[loader]` の型の誤りと、不正な `[scene]`／`[scene.alias]` だけである。不正とは、型の合わない値（`scene`・`alias` が表でない、別名が文字列の配列でないなど）・空の名前・別名の重複・置き換え先を別名にも書いた表（連鎖）である。ほかのセクションの型の誤りは、使う時点でそのセクションが既定値になるだけで、エラーにも警告にもならない。
- ランタイムの構築は `std` のワイド文字 API でモジュールを開くため、`require` の解決は設置パスの長さと文字種に依存しない。ゴースト作者のコードが直接呼ぶ `io.open` などは対象外である（[既知の制限](../reference/startup.md#3-既知の制限)）。

## ソースの所在

- `crates/pasta_lua/src/loader/`
- `crates/pasta_lua/build.rs`
- `crates/pasta_lua/build_zip.rs`
- `crates/pasta_lua/src/runtime/searcher.rs`
- `crates/pasta_lua/src/runtime/module_registry.rs`
- `crates/pasta_lua/src/runtime/runtime_config.rs`
- `crates/pasta_lua/pasta_scripts/main.lua`
- `crates/pasta_lua/pasta_scripts/pasta/config.lua`

埋め込みの zip に固める対象は `crates/pasta_lua/pasta_scripts/` のツリー全体である。テストは `crates/pasta_lua/src/loader/extract_tests.rs`・`crates/pasta_lua/src/loader/discovery_tests.rs`・`crates/pasta_lua/src/loader/config_tests.rs`・`crates/pasta_lua/tests/build_determinism_test.rs`・`crates/pasta_lua/tests/loader/` にある。

## 経緯

- [pasta-scripts-self-deploy](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-scripts-self-deploy) — フレームワークスクリプトの埋め込みと自己展開
- [lua-module-path-resolution](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/lua-module-path-resolution) — `require` によるモジュール読み込みと検索パス
- [lua-path-restructure](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/lua-path-restructure) — 検索パスとスクリプト配置の整理
- [lua-require-robustness](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/lua-require-robustness) — 長パス・非 ANSI パスでの `require`（searcher の置換）
- [lua-passthrough](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/lua-passthrough) — 辞書ディレクトリの `.lua` の素通し
- [pasta-config-restructure](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-config-restructure) — `pasta.toml` の構成と既定値の補完
- [pasta-toml-logging-consistency](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-toml-logging-consistency) — `[lua]` の設定型の撤去、段階 1.5 のフォールバック

---

> 【にっこり】目覚めの儀式も、こうして並べれば整然としたものでしょう？

> 【アンソニー】ひとつだけ、段の番号に 1.5 や 2.5 が混じっているのが気になっておりました。

> 【したり顔】あれは、コードのログの番号に合わせてありますの。ログと突き合わせても、迷わずに済みますでしょう？
