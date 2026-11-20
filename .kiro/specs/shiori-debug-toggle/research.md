# ギャップ分析: shiori-debug-toggle

- 対象: `requirements.md`（Requirement 1〜10）と、今の main（base f423f5e9）のコード・マニュアル
- 方法: Grep・Glob・Read によるコードの読み取り。外の依存は増えない見込みなので、Web の調べは行っていない
- 立場: 選択肢と事実を並べる。決めるのは要件のディスカッションと設計

## 1. 要約

- **判断の所は小さい**: 今の判断は `crates/pasta_lua/src/debug/config.rs` の `resolve_enabled`（純粋な `pick(env, file)`）1 か所で、ローダの Phase 5.5（`loader/mod.rs` 228 行）から 1 回だけ呼ばれる。保存ファイルを読む純粋な関数 `runtime/persistence.rs` の `load_from_file(path) -> toml::Table`（gzip の判別込み・退避なし）が pub で既にあるので、**Rust の側で Phase 5.5 の前に保存ファイルを読み、`pick` を 3 段にする**だけで要件 1〜3 を満たせる見込みが高い。Lua の起動順・シーンマップの渡し方・`@pasta_debug` は触らずに済む。
- **道具の側は部品がそろっている**: `setSaveVariable`（`editors/shared/dumpVariables.ts`）・`reloadAndWait`（`reloadShiori.ts`）・`DevSession`（`devSession.ts`）をつなぐ新しい流れ 1 つと、`ReloadOutcome` に「`Debug enabled …` の行を見たか」を足すことが中心。VSCode はコマンド 2 つ、pasta-recipe はツールバーのボタン 1 つ（`editors/recipe/src/main.ts` 89 行のリロードのボタンの隣）。
- **見落としやすい穴**: (1) `Debug enabled …` は普通の info の行なので、`[logging] level = "warn"` や `PASTA_LOG` で消える。読み込みの終わりの印（`pasta::load_entry`）はいつも通るので、道具は「終わったのに行が無い＝無効」と誤って言う。(2) 案内の文は `DEBUG_HINT` のほかに `runSceneAtCursor.ts` 81 行の `NO_SCRIPT_GUIDE` にもある。(3) マニュアルで直す箇所は Requirement 9.6 の一覧より多い（`vscode-setup.md`・`reference/pasta-toml.md`・`lua/shiori-events.md`・`internals/debug.md`・`internals/loader.md`・`grammar/variables.md`）。
- **名前の決まりとの食い違い**: エンジンが読むグローバル変数は `＄＊pasta_` で始める決まりがある（`book/src/grammar/variables.md` 441 行「予約グローバル変数」。前例 `＄＊pasta_talk_interval_min` も「保存変数 ＞ pasta.toml ＞ 既定」の形）。`＄＊デバッグ`・`＄＊debug` はこの外にある。
- **規模と危うさ**: 規模 M（3〜7 日。Rust は S、道具とマニュアルで M）。危うさ Low〜Medium（判断の所は今の型をなぞるだけ。ログの行で状態を読む所に上の穴がある）。

## 2. 今の作り（調べた事実）

### 2.1 有効の判断と、シーンマップ

| 所 | 事実 |
| -- | ---- |
| `crates/pasta_lua/src/debug/config.rs`（69 行） | `resolve_enabled(file: Option<&DebugFileConfig>) -> bool`。中は `pick(env, file) = env.or(file).unwrap_or(false)`。`parse_env_bool` で `PASTA_DEBUG` を読む。単体テストは `pick` だけを確かめる（環境変数を読まない） |
| `crates/pasta_lua/src/loader/mod.rs` 219〜235 行 | Phase 5.5。`resolve_enabled(config.debug())` が真のときだけ `build_source_map(&pasta_files, &transpiler)` を呼び、`Some(Arc<SourceMap>)` を作る |
| `loader/source_map_build.rs` | シーンマップは `.pasta` を**読み直して** `transpile_with_source_map(.., &mut Vec::new(), Some(sink))` で作る。出力は捨てるので、Phase 4 の `.lua` は有効・無効で変わらない（Req 3.4 は今の作りで保たれている） |
| `loader/mod.rs` 244〜254 行 | `PastaLuaRuntime::from_loader_with_scene_dic(.., source_map)` の後に `if runtime.debug_enabled() { info!("Debug enabled: development events are allowed") }`。その後に `Startup sequence completed` |
| `runtime/exec.rs` 48 行 | `debug_enabled() = self.source_map.is_some()` |
| `runtime/factory.rs` 93〜99 行 | `register_debug_module(lua, debug_enabled, source_map, base_dir)` を `require("main")` より前に呼ぶ。`enabled()` は登録のときの値を閉じ込める |
| `runtime/factory.rs` 136〜160 行 | Step 7。`scene_dic` の後に、`source_map` があるときだけシーンの索引を突き合わせる |
| `pasta_scripts/pasta/shiori/event/scene_play.lua` 21〜28 行 | 開発用のイベントの門。イベントのたびに `pcall(require, "@pasta_debug")` と `enabled()` を見る |
| pasta_shiori | `debug_enabled` を直接は読まない（Lua の門だけ） |

**待ち受けは無い**: DAP は 2026-11-03（`debug-mcp-runtime`）に撤去済みで、pasta の中に TCP の待ち受けは無い（`TcpListener`・`127.0.0.1` の Rust の使い所は 0。`tests/debug_env_ignored_test.rs`・`pasta_shiori/tests/multi_instance_test.rs` は「待ち受けのログが無い」ことを確かめている）。有効にしたときに開くのは開発用のイベント（`OnPastaScene`・`OnPastaSceneAt`・`OnPastaDumpVariables`）だけ。

### 2.2 グローバル変数の保存と読み込み

| 所 | 事実 |
| -- | ---- |
| `code_gen/element_gen.rs` 39 行 | `＄＊名前` は `save.名前` に変わる。`save` は `require("pasta.save")` の表 |
| `pasta_scripts/pasta/save.lua` | 最初の `require` で `@pasta_persistence.load()` を呼んだ表を返す（Phase 5.5 より後。`main` かシーンのモジュールが最初に `require` したとき） |
| `runtime/persistence.rs` `load_impl` → `read_or_quarantine` | ファイルを読み、読めなければ `.broken` へ退避して空の表。退避できなければ `SaveGate::Closed`（そのランタイムでは保存しない） |
| `runtime/persistence.rs` `load_from_file(path)`（pub） | 読むだけ（gzip のマジックで判別して TOML を読む）。退避も印も無い |
| `runtime/persistence.rs` `toml_to_lua` | 根の表のキーはそのまま Lua の文字列キーになる。TOML の `Boolean` は Lua の真偽値、日時は文字列。つまり `SAVE.デバッグ` ＝ TOML の根のキー `デバッグ` |
| `runtime/lifecycle.rs` | `Drop` で `require("pasta.save")` の表を `save_table` で保存する |
| `loader/config/sections.rs` `PersistenceConfig` | 既定の場所 `profile/pasta/save/save.toml`（`obfuscate = true` なら `.dat`）。`effective_file_path()` で決まる。`register` は絶対パスと `..` を断る |
| `pasta_shiori/src/shiori.rs` 67〜76 行・`release_runtime` | リロードでは**古いランタイムを先に落として（保存してから）**新しく読み込む。SSP の `unload` もアクターのスレッドを join してから戻る。よって「書く → リロード → 次の読み込みで読む」は今の順で成り立つ |
| `pasta_check/src/ghost_files.rs` 67〜82 行 | nar には最上位の `profile/` を入れない |

### 2.3 外から保存変数を書く

- `pasta_scripts/pasta/shiori/event/external_control.lua` 97 行〜 `OnPastaSetSaveVariable`: デバッグの有無によらず働く。`SecurityLevel` が external なら断る。型 `boolean` が使える。成功は 204。ゴーストが先に `REG.OnPastaSetSaveVariable` を登録していれば上書きしない。
- 書くのはメモリの上の `SAVE` だけ。ファイルに出るのは次の `Drop`（リロード）。

### 2.4 道具（`editors/`）

| 所 | 事実 |
| -- | ---- |
| `shared/reloadShiori.ts` `reloadAndWait` | pasta.log の読み口を先に開き → `get_log` → `reload` → 印 `SHIORI load entry completed loaded=…` まで読み、ERROR・WARN の行を集める。**info の行は捨てる**（`Debug enabled …` を見ていない）。返す型 `ReloadOutcome` に有効・無効の欄は無い |
| `shared/ghostFiles.ts` 67 行 `DEBUG_HINT` | 「pasta.toml に [debug] enabled = true を書いてリロードしてください」。使うのは `dumpVariables.ts` 60・63 行と `runSceneAtCursor.ts` 96 行 |
| `shared/runSceneAtCursor.ts` 81 行 `NO_SCRIPT_GUIDE` | 「ゴーストはスクリプトを返しませんでした（204）。pasta.toml に [debug] enabled = true を書いて、SHIORI をリロードしてください。」——**brief に挙がっていない 2 つ目の案内** |
| `shared/dumpVariables.ts` `setSaveVariable`・`formatSet` | 結果は `set`・`refusedByGhost`・`noResponse`・`failed` の 4 つ |
| `shared/devSession.ts` `DevSession.execute` | 重なりの止め（`BUSY`）・相手の選び・見出しの行・`saves` なら保存・知らせの分け。新しい操作は `DevCommand` を 1 つ作れば乗る |
| `shared/flows.test.ts`（289 行） | 184・244 行で `notice === DEBUG_HINT`、259 行で `notEqual`。定数を見ているので、文を変えても試験は定数の参照のまま通る |
| `vscode/package.json` 35〜71 行 | コマンド 3 つ（分類 `Pasta`）と、右クリックのメニュー 3 つ |
| `vscode/src/devCommands.ts` `COMMANDS` | 3 つ。コメント「拡張はローカル変数も書き換えも送らない（3 つのコマンドだけ）」。対象の文書（`isGhostSource`）からゴーストを決める |
| `vscode/src/test/devCommands.e2e.test.ts` 95〜100 行 | 「3 つのコマンドと出力の欄 1 つ」を `subscriptions.length === 4` で確かめる（足せば数が変わる）。`contributions.test.ts` は各コマンドを個別に確かめる |
| `recipe/src/main.ts` 89〜92 行 | 「SHIORI をリロード」のボタンはツールバーに `main.ts` が直に作る（`devPanels.ts` ではない） |
| `recipe/src/devActions.ts` | `RELOAD` の `DevCommand`（45〜52 行）、`setGhost`（別のゴーストで表を消す）、`#change`（書き換え → 表の読み直し）。今の状態を外へ知らせる口は無い |

### 2.5 マニュアルとスキルの参照

`[debug] enabled` か `PASTA_DEBUG` に触れる章（`book/src`）:

| 章 | 箇所 | Req 9.6 の一覧に |
| -- | ---- | ---------------- |
| `debug/index.md` | 63〜82 行「デバッグの有効化」（優先は 1 文） | ある（9.1〜9.5） |
| `debug/dev-actions.md` | 13 行の表（3 つの操作）・100・103・137 行（案内の文の引用） | ある |
| `debug/recipe.md` | 21〜30 行（エモ・むらさきの台詞と「2 行を書き足す」）・118 行の表・166 行 | ある |
| `debug/troubleshooting.md` | 18・39〜64 行（案内の文の引用、確かめる順、「環境変数は pasta.toml より優先」） | ある |
| `getting-started/01-boot.md` | 62 行「うまくいかないときは」 | ある |
| `getting-started/14-nar.md` | 36〜45 行「デバッグを切る」（`enabled = false`） | ある |
| `debug/vscode-setup.md` | 24〜29 行（pasta.toml に 2 行）・見出し「3 つのコマンドの出る場所」 | **無い** |
| `reference/pasta-toml.md` | 882 行〜 `[debug]`（「`PASTA_DEBUG` が … `enabled` より優先」） | **無い** |
| `lua/shiori-events.md` | 開発用のイベントの共通の決まり（「`[debug]` か `PASTA_DEBUG` で有効にしたときだけ」） | **無い** |
| `internals/debug.md` | 43・67 行（`resolve_enabled` の 2 段）・243 行（テストの環境） | **無い** |
| `internals/loader.md` | 93 行（5.5 の要否は `[debug]` と `PASTA_DEBUG`） | **無い** |
| `grammar/variables.md` | 441〜456 行「予約グローバル変数（pasta_ で始まる名前）」 | **無い**（載せるかは議題） |

スキルの `references/` で作り直しの対象になるのは、少なくとも `.claude/skills/pasta-ghost-authoring/references/pasta-toml.md`（898 行）と `.claude/skills/pasta-lua-coding/references/shiori-events.md`（872 行）。作り手は `node book/tools/gen-skill-refs.mjs`。

## 3. 要件と部品の対応（Gap の印: Missing＝無い / Unknown＝調べが要る / Constraint＝縛り）

| 要件 | 使える部品 | Gap |
| ---- | ---------- | --- |
| 1.1〜1.7 優先の 4 段 | `pick`・`parse_env_bool` | **Missing**: 保存変数の段。`pick(env, save, file)` へ広げる |
| 1.8 2 つの名前だけ | — | **Missing**: `デバッグ` → `debug` の順に見る小さな関数 |
| 1.9 シーンから見える値と同じ | `load_from_file`・`effective_file_path`・`toml_to_lua` | **Constraint**: どこで読むかで「同じ」の意味が変わる（§4 の案 A・B）。`main.lua` が起動中に `SAVE.デバッグ` を書くと、案 A はファイルの値、案 B はその後の値を見る |
| 2.1〜2.4 真偽値でない値の警告 | `tracing::warn!`（Stage 1.5 で pasta.log の logger は登録済み） | **Missing**: 警告の文。**Unknown**: OPEN QUESTION 1（どの順で見たときに警告するか） |
| 3.1・3.2 読み込みに 1 回 | 今の Phase 5.5 | 案 A なら今のまま |
| 3.3 `Debug enabled …` の行 | `loader/mod.rs` 253 行 | 今のまま。ただし §5 の穴 1 |
| 3.4 無効なら作らない・`.lua` 不変 | `build_source_map` の門・別の transpile | 今のまま保たれる（案 A）。案 B では門の位置が動く |
| 3.5 値を書き換えない | — | 読むだけにすれば満たす |
| 4.1 同じ働きだけ | 門は `source_map.is_some()` 1 つ | **Constraint**: 4.1 の「127.0.0.1 だけで受ける待ち受け」は今のコードに無い（§7 の 1） |
| 4.2 `OnPastaSetSaveVariable` を変えない | `external_control.lua` | 触らない |
| 5.1〜5.3 書く → リロード → 状態 | `setSaveVariable`・`reloadAndWait`・`DevSession` | **Missing**: 新しい流れ（例 `editors/shared/debugToggle.ts`）と `ReloadOutcome` の有効・無効の欄 |
| 5.4・5.5 行の有無で状態 | `readNewLines`・`loadCompletion` | **Missing**: 行の見つけ方。**Constraint**: §5 の穴 1（info が消えると誤る） |
| 5.6 頼んだ状態と違う | — | **Missing**: 知らせの文。**Unknown**: OPEN QUESTION 4。理由は `PASTA_DEBUG` のほかにもある（§5 の穴 3） |
| 5.7 書き込みを断られた | `formatSet` | **Unknown**: `noResponse`（届いたか分からない）をどちらに入れるか |
| 5.8・5.9 リロードの要約 | `formatReload` | そのまま使える |
| 5.10 未保存の辞書を保存 | `DevCommand.saves = true` | そのまま |
| 5.11 2 つの道具で同じ | `editors/shared/` | 流れと文を shared に置けば満たす |
| 6.1〜6.3 VSCode のコマンド | `package.json`・`COMMANDS`・`DevSession` の `BUSY` | **Missing**: 2 つの登録。**Unknown**: OPEN QUESTION 6（右クリックのメニュー）。試験の数（`subscriptions.length`）が変わる |
| 7.1〜7.6 recipe のボタン | `main.ts` のツールバー・`DevActions` | **Missing**: 最後に確かめた状態を持つ所と、ボタンの文を変える口（`DevActions` → `main.ts`）。`setGhost` でゴーストが替わったら戻す |
| 8.1・8.2 案内 | `DEBUG_HINT`・`NO_SCRIPT_GUIDE` | **Missing**: 2 つの文の書き換え（brief は `DEBUG_HINT` だけ挙げている） |
| 9.1〜9.7 マニュアル | §2.5 の章・`gen-skill-refs.mjs` | **Missing**: 一覧外の章（§2.5 の「無い」） |
| 10.1 表の試験 | `config.rs` の `pick` の単体テスト | **Missing**: 3 段の表 |
| 10.2 環境と場所に左右されない | `tests/common/mod.rs` の `#[ctor]`（`PASTA_DEBUG`・`PASTA_LOG` を消す）・一時フォルダ | 結合テストを足すなら `common` を使う。8.3 の短い名前のパスは `std::path::absolute` の型にならう |
| 10.3 道具の試験 | `flows.test.ts`・`fakeGhostIO.ts`・`devActions.test.ts`・`devCommands.e2e.test.ts` | **Missing**: 流れとボタンの移り変わりの試験。偽の pasta.log に `Debug enabled …` の行を出す口 |

## 4. 実装の選択肢（判断の時期）

縛り: 無効ならシーンマップを作らない・`.lua` は変わらない・判断は読み込みに 1 回・保存の仕組み（書式・場所・gzip・退避）は触らない。

### 案 A: Rust が Phase 5.5 の前に保存ファイルを読む（今の部品を伸ばす）

- **すること**: Phase 5.5 で `config.persistence().unwrap_or_default().effective_file_path()` から場所を決め、`persistence::load_from_file` で読み、根のキー `デバッグ`・`debug` の `toml::Value::Boolean` だけを拾う。`pick` を `env.or(save).or(file).unwrap_or(false)` にする。読めない・無い・場所が不正なら「書いていない」と同じにし、何もしない（退避と警告は後で Lua の `load` が今までどおり行う）。
- **触る所**: `debug/config.rs`（＋表の単体テスト）・`loader/mod.rs` の 1 行。`factory.rs`・`module_registry.rs`・Lua は触らない。
- **良い点**: 差分が最小。判断が今と同じ 1 か所・同じ時期のまま。シーンマップの作り方・渡し方・`@pasta_debug` の閉じ込め・Step 7 の順が変わらない。`PASTA_DEBUG` で決まるときはファイルを読まずに済む（`env` を先に見る）。Req 1.9 の「前に下ろしたときに保存されて読み込みで戻る値」をそのまま読む。
- **気になる点**:
  - ファイルを 2 回読む（Rust で 1 回、Lua の `pasta.save` で 1 回）。保存ファイルは小さいので費用は小さいが、大きなセーブを持つゴーストでは毎回の読み込みに 1 回分の読みと gzip の展開が増える。
  - 2 回の読みの間にファイルが変わることは、読み込みの中では起きない見込み（同じスレッドで続けて読む）。
  - 壊れたファイルは Rust の側では黙って「書いていない」。その後 Lua が退避して警告するので、知らせは今のまま 1 回。
  - `main.lua` が起動中に `SAVE.デバッグ` を書いても、その読み込みには効かない（次の読み込みから）。Req 3.2 の「次の読み込みから」と同じ向き。
  - 層: `debug/` が `runtime::persistence` を使うことになる（同じクレートの中。`structure.md` の「`debug/` は SHIORI 非依存」は保たれる）。

### 案 B: 判断を Lua が起きた後へ移す

- **すること**: ランタイムをシーンマップ無しで作り、`@pasta_persistence` を登録した後に Rust から `require("pasta.save")` を先に呼んで `SAVE.デバッグ`・`SAVE.debug` を読み、有効ならその場でシーンマップを作って `runtime.source_map` に入れ、`@pasta_debug` を登録する。`pasta_files` と `transpiler` をファクトリへ渡す（作る関数を閉包で渡すなど）。
- **触る所**: `loader/mod.rs`・`runtime/factory.rs`（引数と順）・`module_registry.rs`（`enabled` の閉じ込め方）・`runtime/mod.rs`（`with_config_and_source_map`）・関連の試験（`source_map_handoff_test.rs` ほか）。
- **良い点**: ファイルは 1 回だけ読む。値は文字どおり「シーンから見える `SAVE.デバッグ`」。
- **気になる点**:
  - `pasta.save` を最初に `require` する時期が `main` より前へ動く（退避の警告が出る時期と、`main.lua` が `pasta.save` を差し替える使い方への影響を確かめる必要がある）。
  - `main.lua` の後で読むなら `main.lua` の書き換えが効いてしまい、Req 3.2 の「そのセッションでは変わらない」と食い違う余地がある。`main` の前で読むなら案 A と同じ値になり、得は「1 回読み」だけ。
  - シーンマップを Phase 5.5 から動かすと、内部の章（`internals/loader.md`・`internals/debug.md`）の順の説明も書き直しになる。
  - 未完了の `shiori4-v4-host` が `module_registry.rs`・`factory.rs` に口を足す見込みがあり、ファイルが重なる（brief の 2026-11-20 の再測定）。

### 案 C: 混ぜる（Rust が先に読み、その読みを Lua へ渡す）

- **すること**: 案 A のとおり Phase 5.5 の前に Rust が読み、読んだ `toml::Table` をランタイムへ渡して、`pasta.save` の最初の `load` がそれを使う（2 回目の読みをなくす）。
- **良い点**: 1 回読み・判断は今の時期のまま・値は確実に同じ。
- **気になる点**: `load_impl`／`read_or_quarantine` に「先に読んだものを 1 回だけ使う」道を足すことになり、退避と `SaveGate` の決まり（brief の「保存の仕組みに手を入れない」）に触れる。得は費用の小さな 1 回分だけ。

### 比べ

| | 案 A | 案 B | 案 C |
| - | ---- | ---- | ---- |
| 差分 | 小（2 ファイル＋試験） | 中〜大（4〜5 ファイル＋試験＋内部の章） | 中（persistence に手が入る） |
| 判断の時期 | 今のまま | 動く | 今のまま |
| 無効のとき作らない | 今の門のまま | 門の位置を作り直す | 今の門のまま |
| ファイルの読み | 2 回（`PASTA_DEBUG` で決まれば 1 回） | 1 回 | 1 回 |
| 保存の仕組みへの手 | 無し（読むだけの関数を呼ぶ） | 無し（`require` の時期は動く） | 有り |
| 他 spec とのファイルの重なり | 無し | `shiori4-v4-host` | 無し |

## 5. 見つけた穴と危うさ（設計で扱うこと）

1. **`Debug enabled …` の行が消えると道具が誤る**: この行は `pasta_lua::loader` の info。`[logging] level = "warn"`・`filter`・環境変数 `PASTA_LOG` で落ちる。読み込みの終わりの印は `pasta::load_entry` の target で、`tracing_init.rs` が「どの枝でも info を通す」ので落ちない。すると `reloadAndWait` は `finished = true` で、行が無いので「無効になった」と言ってしまう（Req 5.5 の文のまま作ると起きる）。手当ての候補: (a) 印の行（`SHIORI load entry completed`。`pasta_shiori/src/windows.rs` 189 行・`v3.rs` 66 行）に `debug=true|false` の欄を足して読む、(b) `Debug enabled …` を `pasta::load_entry` の target で出す、(c) 行が無いときは「確かめられなかった」と出す。(a)(b) は Req 3.3（同じ文の 1 行）とは両立する（文は変えず、欄か target だけ）。
2. **案内の文は 2 つある**: `DEBUG_HINT` のほかに `NO_SCRIPT_GUIDE`（`runSceneAtCursor.ts` 81 行）が pasta.toml の書き換えを言っている。204 の理由はデバッグが無効のほかにもある（`troubleshooting.md`「スクリプトが返らない（204）」）。トグルを案内しても「ほかの理由は …」の道は残す必要がある。
3. **「頼んだ状態と違う」の理由は `PASTA_DEBUG` だけではない**: ほかに、(i) 保存できない状態（`SaveGate::Closed`。読めないセーブを退避できなかった）、(ii) ゴーストが `OnPastaSetSaveVariable` を自分の関数で差し替えている、(iii) ゴーストの Lua・シーンが下ろすまでに `SAVE.デバッグ` を書き戻す、(iv) 穴 1 のログの設定。知らせの文で `PASTA_DEBUG` だけを名指しすると外れることがある（OPEN QUESTION 4）。
4. **入門の 14 段目との組み合わせ**: トグルで一度 `＄＊デバッグ = true` を入れると、手元では `pasta.toml` の `enabled = false` が効かなくなる（保存変数が上）。配る nar には `profile/` が入らないので配った先は無効のまま（`ghost_files.rs`）だが、作者の手元で「切ったのに ▶ が効く」に見える。逆に「無効にする」で `false` を入れると、後で `pasta.toml` を `true` にしても手元では効かない。道具には「変数を消して pasta.toml に戻す」口が無い（Req 5.2 は消さずに `false`）。マニュアル（Req 9.5）だけで足りるか、3 つ目の操作が要るかは議題。
5. **保存の場所は設定で変えられる**: `[persistence] file_path` は `profile/` の外（例 `save.toml`）にもできる（断るのは絶対パスと `..` だけ）。その場合は保存ファイルが nar に入るので、Req 9.5 の「配布物に入らない」は既定の場所での話になる。
6. **recipe のボタンの状態を変える操作**: ▶・「値を入れて実行」も読み込み直す（`runScene` が先に `reloadAndWait` を呼ぶ）。Req 7.4 はトグルと「SHIORI をリロード」だけを挙げている。▶ の後もボタンを合わせるかは決めておく。
7. **VSCode の拡張が書き換えを送るようになる**: `devCommands.ts` の「拡張は書き換えを送らない」は `vscode-mcp-client` の決めごと。トグルはその例外になる（`OnPastaSetSaveVariable` は前からいつでも効くので、開く口は増えない）。コメントと内部の章の言い方を直す。
8. **名前の決まり**: `grammar/variables.md` は「エンジンが読むグローバル変数は `＄＊pasta_` で始まる。自分の変数に使わない」と決めている。`＄＊デバッグ`・`＄＊debug` は作者の変数と同じ名前の空間にある（brief は `debug` の衝突に触れたが、`デバッグ` も同じ）。予約の表に載せるか、マニュアルの「デバッグの有効化」の 1 行（Req 9.4）だけにするかは議題。

## 6. 規模と危うさ

- **規模: M（3〜7 日）** — Rust の判断は案 A なら S（関数 1 つと表の試験・結合テスト 1〜2 本）。道具は shared の流れ 1 つと 2 つの UI と試験で S〜M。マニュアルは §2.5 の 12 章ほどとスキルの参照の作り直しで M。
- **危うさ: Low〜Medium** — 判断の所は今の型（`pick` と、`＄＊pasta_talk_interval_*` の「保存変数 ＞ pasta.toml ＞ 既定」の前例）をなぞる。ログの行で状態を読む所（穴 1）と、手元の状態が残ること（穴 4）が利用者の混乱につながりうる。

## 7. 要件の側で気づいたこと（requirements.md は直していない）

1. **Req 4.1 の「127.0.0.1 だけで受ける待ち受け」は今のコードに無い**: DAP の撤去（2026-11-03）で待ち受けは無くなっており、試験は「待ち受けのログが無い」ことを確かめている。有効で開くのは開発用のイベントだけ。brief の「気をつけること」と Constraints の同じ言い方も古い。
2. **Req 5.4・5.5 はログの設定で誤る**（§5 の穴 1）: 「行が出なかった＝無効」を、行が落ちうる設定でどう扱うかが要件に無い。
3. **Req 8.1 の対象の文は 2 つ**（§5 の穴 2）: `NO_SCRIPT_GUIDE` も含めるか、明記が要る。
4. **Req 9.6 の章の一覧が足りない**（§2.5 の「無い」の行）: `vscode-setup.md`（2 行の書き足し・「3 つのコマンドの出る場所」）、`reference/pasta-toml.md`（`[debug]` の優先の 1 文）、`lua/shiori-events.md`（共通の決まり）、`internals/debug.md`・`internals/loader.md`（判断の段）。`dev-actions.md` の「3 つの操作」の表と `index.md` の章の表の「3 つの操作」の数も変わる。
5. **Req 5.7 と `noResponse`**: `setSaveVariable` の結果の `noResponse`（届いたか分からない）が「断った・頼めなかった」に入るのか、リロードへ進むのかが決まっていない。
6. **Req 7.4 と ▶**（§5 の穴 6）: ▶ と「値を入れて実行」の読み込み直しの後の扱いが無い。
7. **Req 1.9 と `main.lua`**: 「シーンから見える値と同じ」と「前に下ろしたときに保存された値」は、`main.lua` が起動中に書くと分かれる。どちらを正とするか（案 A は後者）。
8. **Req 9.5 の「配布物に入らない」**（§5 の穴 5）: 既定の場所の話であることの断りが要るか。
9. **「pasta.toml に戻す」口**（§5 の穴 4）: Req 9.5 は「変数を消せばよい」と書くが、道具には消す操作が無い（recipe の変数の表からは消せる。VSCode からは消せない）。
10. **名前の決まり**（§5 の穴 8）: 予約グローバル変数（`pasta_` で始まる）の決まりと、`grammar/variables.md` の表に載せるかが要件に無い。

## 8. 設計への申し送り

- **推す方向（決定ではない）**: 案 A。差分が最小で、判断の時期・シーンマップの門・Lua の起動順を変えずに要件 1〜3 を満たせる。案 B・C の得（1 回読み）は小さい。
- **設計で決めること**:
  1. 判断の時期と保存ファイルの読み方（案 A／B／C）。
  2. 真偽値でない値の警告の文と、警告する条件（OPEN QUESTION 1）。
  3. 道具が状態を読む手段（`Debug enabled …` の行／印の行の欄／target）と、行が見えないときの扱い（§5 の穴 1）。
  4. `ReloadOutcome` に足す欄の形と、それを使う操作（トグル・「SHIORI をリロード」・▶）。
  5. 「頼んだ状態と違う」の知らせの文（理由の挙げ方。§5 の穴 3）。
  6. `noResponse` の扱い。
  7. recipe のボタンの状態の持ち場（`DevActions` が持ち、`main.ts` へ知らせる口）と、ゴーストの替わりで戻す所（`setGhost`）。
  8. VSCode の右クリックのメニューに足すか（OPEN QUESTION 6）。
  9. 案内の文（`DEBUG_HINT`・`NO_SCRIPT_GUIDE`）の新しい文。
  10. （要件のディスカッションで決着）予約グローバル変数の表に `＄＊デバッグ`・`＄＊debug` を載せる（Req 9.8）。「pasta.toml に戻す」操作は足さない（Req 5.12。戻すときは recipe の変数の表で消す）。
  11. `devCommands.ts` の「拡張は書き換えを送らない」の言い方と、それを書いた内部の章を、トグルを例外として直す（§5 の穴 7）。
- **調べが要ること（Research Needed）**:
  - `[logging]` の `level`・`filter`・`PASTA_LOG` を変えたとき、`Debug enabled …` が pasta.log に残るかを実機かテストで確かめる。
  - 開発機の `PASTA_DEBUG=1` を外した状態での実機の確かめ（brief の申し送り。外さないと「無効にする」が必ず効かない）。
  - `tests/common/mod.rs` の `#[ctor]` を使う結合テストで、一時フォルダの `profile/pasta/save/save.toml`（と `obfuscate = true` の `.dat`）に `デバッグ = true` を置いて読み込みを確かめる形。
