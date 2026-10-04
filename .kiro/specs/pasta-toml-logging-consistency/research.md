# ギャップ分析: pasta-toml-logging-consistency

- 実施日: 2026-10-04
- 対象: `requirements.md`（要件 1〜7、前提 A1〜A10）と現行 main（8033b4d7 時点のワークツリー）
- 方法: コード読解（cargo の実行なし）。`from_libs` の失敗原因はサブエージェントがコード読解で特定した

## 1. 分析の要約

- **U26・U32 の撤去は機械的で小さい**。消すのは型・アクセサ・変換・再エクスポート・テスト・サンプル 1 行・マニュアル数か所。serde は未知のキーを無視するため（`deny_unknown_fields` 無し）、既存のゴーストは読み込める。破壊的なのは `pasta_lua` の公開 API（`LuaConfig`・`PastaConfig::lua()`・`From<LuaConfig>`・`LoggingConfig::rotation_days` フィールド）である。
- **`from_libs(["std_string"])` の失敗原因は確定した**。`package` が無いと、`runtime/mod.rs:194` から呼ぶ `search::register` の `lua.globals().get::<Table>("package")`（`search/mod.rs:75`）が nil→table の変換エラーになる。`package` はほかにも `@pasta_log` の登録・`package.path` の設定・searcher で無条件に必要。既存の `ConfigError`（`UnknownLibrary`）を拡張して事前に拒否するのが最も小さい（`search/` は並走条件で触れない）。
- **ログの破棄は brief が挙げた範囲より広い**。FFI 入口スレッド（`request`・`unload`・`load` の無視）に加え、(1) アクタースレッドのメッセージループの観測ログ（`actor/thread.rs`）、(2) `PastaShiori::drop` の「Unregistered logger」、(3) **ランタイム破棄時の永続化保存の失敗ログ**（`runtime/lifecycle.rs:60`）も捨てられている。(3) はロガーの登録解除の後、ガードの外で出るためである。
- **入口スレッドのログは 2 案が実現可能**。(a) ガードを張る箇所を増やす（変更箇所が散る。アクタースレッドの観測ログまで拾うには `actor/` にも手を入れる）、(b) `RoutingWriter` のフォールバック（`logging/registry.rs` の 1 か所で全スレッドを拾える。複数ロガー時は捨てる）。終了処理のログは、どちらの案でも `PastaShiori::drop` の順序（登録解除をランタイム破棄の後へ）を直す必要がある。
- **不正な `file_path`** は、SHIORI 経由なら段階 1 の既定ロガーが残るため「既定ファイルへ書き続ける」が既に実挙動である。直すのは warn の文言（現行は「logging disabled」と誤った内容）と、マニュアル 2 か所（`reference/pasta-toml.md:420` 付近・`reference/startup.md:132`）である。

## 2. 現状の調査

### 2.1 設定（U26・U32）

| 資産 | 場所 | 現状 |
| ---- | ---- | ---- |
| `LuaConfig`・`default_libs` | `crates/pasta_lua/src/loader/config/sections.rs:122-184` | `[lua] libs` の型。`default_libs()` もここで定義 |
| `PastaConfig::lua()` | `loader/config/mod.rs:173` | 呼び出し元はテストだけ |
| `From<LuaConfig> for RuntimeConfig` | `runtime/runtime_config.rs:321-329` | 呼び出し元はテストだけ。`use crate::loader::{LuaConfig, default_libs}`（9 行） |
| 再エクスポート | `lib.rs:58-59`・`loader/mod.rs:36-37` | `LuaConfig`・`default_libs` を公開 |
| ローダ・SHIORI の構成 | `loader/mod.rs:78`（`RuntimeConfig::new()`）・`pasta_shiori/src/shiori.rs:121` | `[lua]` を読まない |
| `LoggingConfig::rotation_days` | `sections.rs:24-25`・`default_rotation_days`（67 行） | どこからも読まれない。`logger.rs:65` は `Rotation::NEVER` 固定 |
| テスト | `tests/loader/config_test.rs:219-316`（rotation_days）・`403-469`（LuaConfig）、`tests/runtime/unit_test.rs:7,189-` （From<LuaConfig>）、`src/logging/logger.rs` のテスト 4 か所（175・191・247・266 行で `rotation_days: 7`）、`tests/loader/config_sections_test.rs:72`（型不一致のテストが `rotation_days = "fourteen"` を使う） | 撤去に伴い削除・書き換え |
| サンプル | `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/pasta.toml:31` | `rotation_days = 7` |
| リリース成果物 | `release/hello-pasta/ghost/master/pasta.toml:31` | サンプルと同一内容。リリースビルドの出力（`chore(release): build hello-pasta v0.2.4`）で、次のリリースで再生成される |
| マニュアル | `reference/pasta-toml.md:61`（一覧表）・`152-153`（テンプレートのコメント）・`430-435`（`[lua]` 節）、`internals/logging-encoding.md:39,131`、`internals/loader.md:38`（`LuaConfig` への言及） | 撤去に合わせて更新 |
| README | `crates/pasta_lua/README.md:90` | `RuntimeConfig::from_libs` の例。残す API なので変更不要 |

- `[lua]` を撤去しても、書かれた `[lua]` は他の未知のセクションと同じく `custom_fields` に残り、`@pasta_config` から読める（Lua 側への露出は変わらない）。
- `config_sections_test.rs:72` の型不一致テストは、`rotation_days` を消すと「未知のキー」になり `logging()` が `Some` を返すので、意味を保つには別のキー（`level = 1` など）へ差し替える必要がある。

### 2.2 `from_libs` の失敗（要件 3）

- `runtime_config.rs:202-232` の `to_stdlib()` は std_* の名前だけを StdLib フラグへ変換する。`["std_string"]` は `StdLib::STRING` だけになり、`runtime/mod.rs:174` の `Lua::unsafe_new_with` の時点で `package` が nil になる。
- 最初に失敗するのは `mod.rs:194` → `search/mod.rs:75` の `lua.globals().get::<Table>("package")?`。同じ依存が `module_registry.rs:22`（`set_loaded_module`）・`:47`（`setup_package_path`）・`searcher.rs:119,185`・`exec.rs:143`・`finalize.rs:256` にもある。
- 乱数シード（`mod.rs:178`）は `if let Ok` で守られており math が無くても失敗しない。`tostring` は base に含まれる。
- 既存テストは `to_stdlib` のフラグ計算（`tests/runtime/unit_test.rs:42-81`）と、`["std_all","-std_math"]` で VM が作れること（`runtime_api_test.rs:196`）を見るだけで、`std_package` の無い構成で VM を作るテストは無い。
- ドキュメント（`runtime_config.rs:41-48`・`sections.rs:150-157`）は `std_package` を任意の選択肢として並べ、必須とは書いていない。
- `ConfigError::UnknownLibrary` が既にあり（`runtime_config.rs:250`）、`to_stdlib()` は `Result` を返す。ここで必須ライブラリの欠落も検査すれば、`with_config`・`from_loader*` のどの経路も 1 か所で守れる。

### 2.3 ログの経路（要件 4）

ログの振り分けはスレッドローカルの `CURRENT_LOAD_DIR`（`logging/registry.rs:131-163`）で決まり、`make_writer`（`registry.rs:117-126`）は文脈が無いか未登録なら黙って捨てる。ガードを張るのは `PastaShiori` の `load`（`shiori.rs:107`）・`request`（163）・`kick`（327）・`call_lua_unload`（368）と、FFI の `load_impl`（`windows.rs:198`）だけである。

捨てられているログ（ロガーは生きているのに残らないもの）:

| ログ | 場所 | スレッド | 理由 |
| ---- | ---- | -------- | ---- |
| `request` の null・UTF-8 不正の warn、panic の error | `windows.rs:257-305` | FFI 入口 | ガード無し |
| marshaling の観測ログ（`actor.try_send`・`actor.drop`・`actor.timeout` など） | `actor/marshaling.rs:169-237` | FFI 入口 | ガード無し |
| `load` の「loadu で初期化済みのため無視」warn | `windows.rs:144` | FFI 入口 | ガード無し（ロガーは loadu で登録済み） |
| teardown の観測ログと異常の warn | `actor/teardown.rs:118-168`・`windows.rs:229` | FFI 入口 | ガード無し。加えて正常終了では先にロガーが破棄済み |
| アクターのメッセージループの観測ログ（`actor.spawn`・`actor.recv`・`actor.reply`・`actor.stop`・`actor.done`） | `actor/thread.rs:157-248` | アクター | `PastaShiori` のメソッドの外でガード無し |
| 「Unregistered logger」 | `shiori.rs:61` | アクター（drop） | 登録解除の直後・ガード無し |
| 永続化保存の失敗の error・`debug_mode` の保存ログ | `runtime/lifecycle.rs:49,60` | アクター（drop） | `shiori.rs:68` のランタイム破棄は登録解除（60 行）の後で、ガードも無い。ロガーの `Arc` はランタイムが持つので生きている |
| lifecycle の観測ログ（`actor.spawn`・`kick.drop`） | `actor/lifecycle.rs:93,188,204` | FFI 入口・任意 | ガード無し |

補足:

- `DllMain` の detach は、`FreeLibrary` によるもの（`reserved` が null）は `unload()` を呼ぶので `unload` と同じ扱いになる。プロセス終了によるもの（`reserved` が非 null）は何もしない（棚卸の即時修正）。後者ではログを出していないので、保存の対象が無い（前提 A7）。
- teardown の異常のうち **Timeout** は、アクターがまだ `PastaShiori` を破棄していない（ループ中か終了処理の途中）ことが多く、ロガーは登録されたままのことが多い。ただし永続化保存（ランタイム破棄）の途中で止まっている場合は、登録解除の後なので文脈を合わせても届かない。**Disconnected** はアクターが ack 前に `done` を落とした場合で、unwind プロファイルの panic でしか起きない（release は `panic=abort` のためプロセスごと終わる）。そのときロガーは破棄済みの可能性が高い（前提 A6）。
- SSP では、ゴーストごとに別パスの `pasta.dll` を読み込むため、DLL の static（登録簿）は 1 ゴースト専用になる。登録簿に複数のロガーが入るのは、`cargo test` の並列実行や、`pasta_lua` を直接組み込む場合である。
- `PastaLogger` は `tracing_appender::non_blocking` のワーカースレッドで書く。プロセス終了中は他スレッドが止められており、ここから書けない。

### 2.4 不正な `file_path`（要件 5）

- 判定は `PastaLogger::validate_path`（`logging/logger.rs:84-110`）。`base_dir.join(file_path)` が `base_dir` の下にあり、相対パスが文字列として `profile` で始まり、`..` を含まないこと。絶対パスは `join` で置き換わり `strip_prefix` で弾かれる。
- 段階 1.5（`loader/mod.rs:234-261`）で作成に失敗すると、warn「Failed to create instance logger, logging disabled」を出して `Ok(None)` を返し、登録は変えない。
- SHIORI では段階 1（`shiori.rs:100-103`）の既定ロガーが登録されたまま残るので、**以後のログは既定ファイルへ書かれ続ける**。warn 自体も、段階 1.5 はガードの内側なので既定ファイルに届く。つまり「既定ファイルへのフォールバック」は既に実挙動で、warn の文言だけが誤っている（「logging disabled」）。
- `PastaLoader` を直接使う組み込みでは段階 1 が無いので、ロガーは作られない（前提 A8）。
- フィルタは失敗時も反映済み（棚卸の即時修正 `df66d9ac`）。
- マニュアルの記述: `reference/pasta-toml.md:420`「ログファイルが作られない」、`reference/startup.md:132`「ログの初期化自体が拒否される」。どちらも実挙動と違う。スキルの写し `.claude/skills/pasta-ghost-authoring/references/pasta-toml.md:420` も同じ。
- 観察: 文字列の前方一致なので `profiles/x.log`・`profile.log` も通る（前提 A10。本仕様では変えない）。

### 2.5 マニュアルとスキルの生成

- 生成は `book/tools/gen-skill-refs.mjs` の `GENERATION_MAP`。本仕様で触る章のうち `reference/pasta-toml.md`→`pasta-ghost-authoring`、`reference/startup.md`・`lua/modules/mlua-stdlib.md`→`pasta-lua-coding` が写される。`internals/logging-encoding.md`・`internals/shiori.md`・`internals/loader.md` は写されない。
- `lua/modules/mlua-stdlib.md:116`「ゴーストからこのモジュールを有効にする手段は無い」は撤去後も正しい（変更は不要か、`[lua]` 節の撤去に伴う参照の整理程度）。
- 差分検査（drift-check）は改行を LF に正規化して比較する。

## 3. 要件と資産の対応

| 要件 | 主な資産 | ギャップ | 種別 |
| ---- | -------- | -------- | ---- |
| 1.1–1.2 | `reference/pasta-toml.md`・`mlua-stdlib.md` | `[lua]` の行・節・テンプレートのコメントを消す | Missing |
| 1.3 | `PastaConfig` の `custom_fields` | 既に満たす（serde・custom_fields） | なし |
| 1.4–1.5 | `sections.rs`・`config/mod.rs`・`runtime_config.rs`・`lib.rs`・`loader/mod.rs` | 型・アクセサ・変換・再エクスポートの削除。`default_libs` は残す（定義場所を `sections.rs` に置くかは設計で決める） | Missing / Constraint（公開 API の破壊） |
| 2.1–2.4 | `sections.rs`・サンプル・`internals/logging-encoding.md`・テスト | フィールドと既定関数・サンプル 1 行・マニュアル 2 か所・テストの削除と書き換え | Missing |
| 2.5 | `logger.rs` | 既に満たす | なし |
| 3.1–3.3 | `runtime_config.rs::to_stdlib`・`ConfigError` | 必須ライブラリの検査とエラー型の追加。必須の一覧（`package` のほか、ローダ経由で読み込む `pasta_scripts` が要る `string`・`table` など）は未確定 | Missing / Unknown |
| 4.1, 4.5 | `windows.rs` | FFI 入口のログの振り分け | Missing |
| 4.2 | `actor/thread.rs`（または `logging/registry.rs`） | アクタースレッドのガード外ログの振り分け | Missing |
| 4.3 | `shiori.rs::drop` | 登録解除をランタイム破棄の後へ移し、終了処理全体を振り分けの文脈で包む | Missing |
| 4.4 | `windows.rs::unload`・`actor/teardown.rs` | 異常の warn をロガーが生きている間に届ける方法。Disconnected の扱いが未決 | Missing / Unknown |
| 4.6–4.7 | `logging/registry.rs` | 複数ロガー・未登録時の扱いの明確化 | Constraint |
| 4.8 | `actor/marshaling.rs`・`teardown.rs` | 応答と待ち時間を変えないこと（回帰テストで固定） | Constraint |
| 5.1, 5.3 | 段階 1・1.5 | SHIORI では既に満たす | なし |
| 5.2 | `loader/mod.rs:257` | warn の文言（不正な値・既定ファイルへのフォールバック）を直す | Missing |
| 5.4 | `shiori.rs:83-91`・段階 1.5 | 既に満たす見込み（再読み込みで段階 1 から作り直す）。テストで確認 | Unknown |
| 6.x | マニュアル 5 章・スキル再生成 | 記述の更新 | Missing |
| 7.x | `pasta_lua/tests`・`pasta_shiori/tests` | ログファイルの中身を確かめる E2E は `ffi_loadu_test.rs:190-245` の `read_ghost_logs`/`assert_entry_logged` が先例 | Missing |

## 4. 実装の選択肢

### 4.1 入口・アクタースレッドのログ（要件 4.1・4.2・4.5）

#### 案 A: ガードを張る箇所を増やす（brief の案 (a)）

- ロード時の設置パスを `windows.rs` の static（例: `Mutex<Option<PathBuf>>`・`ArcSwapOption<PathBuf>`）に持ち、`request`・`unload`・`load` の無視の各入口でガードを張る。アクタースレッドはスレッドの入口（`actor/thread.rs` の `block_on` の先頭）で一度ガードを張る。
- ✅ 振り分けの規則（スレッドの文脈で決まる）は今のまま。複数ロガーでも誤配が起きない。
- ❌ 変更箇所が `windows.rs`・`actor/thread.rs`・（lifecycle の観測ログまで拾うなら）`actor/lifecycle.rs` に散る。`actor/` は brief の境界候補に無い。
- ❌ 入口が増えるたびにガードを忘れる危険が残る（今回の不具合の原因そのもの）。
- ❌ `request` は送信パスに Mutex を置かない（R8）ため、static は lock-free な型にする必要がある。

#### 案 B: 文脈の無いログを唯一のロガーへ流す（brief の案 (b)）

- `make_writer` で、文脈が無い（または文脈の設置パスが未登録の）とき、登録簿のロガーがちょうど 1 つならそれへ書き、0 個か 2 個以上なら捨てる。
- ✅ 変更は `logging/registry.rs` の 1 か所。FFI 入口・アクタースレッド・lifecycle・panic ハンドラのどれから出たログも拾える。今後入口が増えても漏れない。
- ✅ SSP の実運用（1 DLL＝1 ゴースト）では必ず 1 個なので、本番のログはすべて残る。
- ❌ 複数ロガーのとき（テストの並列実行・組み込み）は従来どおり捨てる。要件 4.6 の「誤配しない」は満たすが、テストでは「唯一」の条件が他テストの登録に左右されうる（下の調査項目）。
- ❌ デバッグバックエンドのスレッドのログも副次的に残るようになる（前提 A9 で許容するか決める）。
- ❌ 文脈のある（ガードを張った）スレッドが未登録の設置パスを指している場合にフォールバックするかは別判断（再読み込みの合間のログ）。

#### 案 C: 併用

- 案 B を基本にし、アクタースレッドだけ入口で一度ガードを張る（アクタースレッドは自分の設置パスを知っている）。FFI 入口は案 B に任せる。
- ✅ アクタースレッドのログは複数ロガー時も正しく振り分けられる。FFI 入口は 1 か所の変更で済む。
- ❌ `actor/thread.rs` にも手が入る。

### 4.2 終了処理のログ（要件 4.3・4.4）

- `PastaShiori::drop` の順序を「ガードを張る → `SHIORI.unload` → キャッシュした関数を捨てる → ランタイムを破棄（永続化保存）→ 登録解除 → 登録解除のログ」に変える。ロガーの `Arc` はランタイムと登録簿の両方が持つので、登録解除のログを登録解除の前に出すか、ガードを保ったまま案 B のフォールバックに頼るかを設計で決める。
- 再読み込みの分岐（`shiori.rs:83-91`）も同じ順序の問題を持つ（FFI 経路では通らないが、組み込みでは通る）。
- teardown の異常の warn（要件 4.4）は次の選択肢がある。
  - (i) FFI 入口で出す（今の位置）。案 A ならガード、案 B ならフォールバックで、ロガーが登録されている場合（Timeout の大半）は残る。Disconnected と「永続化保存中の Timeout」は残らない。
  - (ii) FFI 側がロガーの `Arc` を持ち続け、teardown が終わってから warn を書いて手放す（ロガーの寿命を延ばす）。両方の場合で残るが、ロガーの所有が SHIORI 層へ漏れる。
  - (iii) 異常を `TeardownReport` で返すだけでなく、アクター側で検知できるもの（drop の途中の失敗など）はアクター側でログに出す。

### 4.3 不正な `file_path`（要件 5）

- 案 1（最小）: `create_and_register_logger` の失敗時の warn の文言を直す（不正な値と、既定ファイルへのフォールバックを示す）。挙動は SHIORI では既に要件どおり。組み込みでは「既定ロガーが登録されていればフォールバック、無ければロガー無し」を文言で分ける（登録簿を引けば分かる）。
- 案 2（統一）: 段階 1.5 自身が失敗時に既定の設定で `PastaLogger` を作り直して登録する。SHIORI と組み込みで挙動が揃い、warn も一律に「既定ファイルへ」と書ける（前提 A8 の代替案）。SHIORI の段階 1 とは同じパスのロガーを作り直すことになる（同じファイルへの追記なので害は無いが、ワーカースレッドが一時的に 2 本になる）。

### 4.4 必須ライブラリの検査（要件 3）

- 案 1: `to_stdlib()` で必須フラグ（少なくとも `PACKAGE`）の欠落を検査し、`ConfigError` に新しい種類（例: `MissingRequiredLibrary`）を足して返す。`search/` を触らずに済む。
- 案 2: `to_stdlib()` で `PACKAGE` を常に OR する。作者の `-std_package` を黙って上書きする。ローダ経由の `pasta_scripts` が要る `string`・`table` などの欠落は救えない。
- 案 3: ドキュメントに前提を書くだけ（エラーは分かりにくいまま）。

## 5. 工数とリスク

| 領域 | 工数 | リスク | 理由 |
| ---- | ---- | ------ | ---- |
| U26・U32 の撤去 | S | Low | 削除中心。呼び出し元はテストだけ。公開 API の破壊はリリース時のマイナー更新で吸収 |
| 必須ライブラリの検査 | S | Low〜Medium | 既存の `ConfigError` の拡張。ただし必須の一覧（ローダ経由で要るもの）の確定に調査が要る |
| 入口・アクタースレッドのログ | M | Medium | ログの経路はプロセス全域の状態（購読者・登録簿）で、テストの並列実行と干渉しやすい。送信パスに Mutex を置かない制約（R8）がある |
| 終了処理のログ | S〜M | Medium | `PastaShiori::drop` の順序変更。永続化保存・DAP の片付けの順序（`internals/shiori.md` の記述）との整合が要る |
| 不正な `file_path` | S | Low | 文言とマニュアルの修正が中心 |
| マニュアル・スキル | S | Low | 既存の生成手順と差分検査がある |
| 全体 | M（3〜7 日） | Medium | 個々は小さいが、ログの E2E テストの設計（プロセス全域の状態）が一番の不確実性 |

## 6. 設計フェーズへの推奨

- **推奨の方向**: U26・U32 は撤去（前提 A1・A2）。ログは案 B（または C）＋ `PastaShiori::drop` の順序の修正。不正な `file_path` は案 1 か案 2（前提 A8 の結論次第）。必須ライブラリは案 1。
- **設計で決めること**:
  - `default_libs` の定義場所（`LuaConfig` と一緒に消える `sections.rs` に残すか、`runtime_config.rs` へ移して再エクスポートの経路を保つか）。公開パス `pasta_lua::default_libs`・`pasta_lua::loader::default_libs` を維持するか。
  - 案 B のフォールバックの条件（文脈無しのみか、未登録の文脈も含むか）。
  - teardown の異常の warn の届け方（4.2 の (i)〜(iii)）。
  - 必須ライブラリの一覧と、エラーの種類・文言。
- **調査が要る項目（Research Needed）**:
  - ローダ経由（`PastaLoader::load_with_config`）で `pasta_scripts` の読み込みまで通るための最小のライブラリ構成（`std_package` だけで足りるか、`string`・`table`・`coroutine` も要るか）。mlua-stdlib の `register` が `package` 無しで動くか。
  - 案 B を採るとき、`cargo test` の並列実行で「登録簿のロガーがちょうど 1 つ」の条件を E2E テストでどう作るか（`serial` 化、テスト専用プロセス、`#[ctor]` による環境の中和の先例 `c59ee8d` を参照）。
  - `PastaShiori::drop` の順序を変えたとき、永続化保存・DAP バックエンドの片付け・`done` ack の順序の不変条件（`internals/shiori.md` の「unload・DllMain と teardown」）が保たれるか。
  - 撤去した公開 API の利用者が crates.io 上にいるか（0.x のため破壊は許されるが、CHANGELOG での告知の要否）。

## 7. 前提・未決事項への補足

- A3: 必須ライブラリの欠落は、`[lua] libs` の撤去後は組み込み開発者しか踏まない（ゴーストからは構成を変えられなくなる）。範囲に入れても変更は小さい。
- A6: Disconnected は release（`panic=abort`）では実質起きない。実運用で意味があるのは Timeout で、そのうちロガーが既に登録解除されているのは「ランタイム破棄（永続化保存）の途中で止まった」場合に限られる。drop の順序を直せば、この場合もロガーは登録されたままになる。
- A7: プロセス終了中は書き込み用のワーカースレッドが止まっているため、記録しようとしても届く保証が無い。
- A9: 案 B を採るとデバッグバックエンドのログも副次的に残る。brief の Out of scope は「デバッグバックエンドのログ（の挙動を変えること）」と読めるため、副次的に残ることを許すかを確定したい。

---

# 設計フェーズの調査と決定（2026-10-04）

- 種別: 既存システムの拡張（light discovery）。外部の新しい依存は無いため、Web 調査はしていない。
- 方法: コード読解（`crates/pasta_lua`・`crates/pasta_shiori`・`book/`）。cargo の実行なし。

## 8. 6 節「調査が要る項目」の結果

### 8.1 ローダ経由で要る最小のライブラリ構成

- Rust 側が無条件に使うのは `package` だけである（`search::register`・`module_registry.rs` の `set_loaded_module` と `setup_package_path`・`searcher.rs`・`exec.rs`・`finalize.rs`）。
- `math` は無くても VM を作れる。乱数の種の設定は `if let Ok(math)` で守られ、既存テスト `test_runtime_without_math_library_still_builds`（`tests/runtime/runtime_api_test.rs`）がこの挙動を固定している。必須ライブラリの検査を `to_stdlib()` に入れると、`test_to_stdlib_empty_libs`（空の一覧は `StdLib::NONE`）などフラグ計算のテストが壊れるため、検査は `to_stdlib()` の外（`ensure_libs`）に置く。
- `pasta_scripts` の標準ライブラリの使用（コメント行を除く検索）: `string` 22 か所、`table` 34 か所、`math` 10 か所（`act.lua`・`virtual_dispatcher.lua`・`sakura_builder.lua`・`lua_version.lua`）、`os` 2 か所（`shiori/act.lua` の `os.time() + timeout`、`shiori/event/second_change.lua` の `CALLBACK.sweep(os.time())`）、`coroutine` 22 か所、`jit` 2 か所（`lua_version.lua`。引数で受けた表を読むだけ）。`io`・`bit`・`ffi`・`debug` は 0 か所。
- `std_coroutine` は `StdLib::NONE` に対応する（LuaJIT では基本ライブラリに含まれる）。必須の一覧に入れる必要は無い。
- `os`・`math` の使用は呼び出し時で、欠けていても起動は通り、毎秒の OnSecondChange やトークの時点で Lua エラーになる。構成の誤りから遠いエラーになるので、ローダ経由では起動前に止める（設計 D2）。
- mlua-stdlib の `register` が `package` 無しで動くかは未確認（ソースが手元のレジストリに見つからなかった）。`package` は Rust 側の登録で必ず要るため、結論に影響しない。

### 8.2 並列実行の下での E2E テスト

- `tests/` 直下の 1 ファイルは 1 つのテストバイナリ（1 プロセス）になる。登録簿・購読者・フィルタはプロセスごとに独立する。
- 先例が 2 つある。`pasta_shiori/tests/ffi_loadu_test.rs` は FFI の static（`MAILBOX`・`LOADU_INITIALIZED`）を共有するため `#[test]` を 1 本にして直列化している。`pasta_lua/tests/logging_filter_reload_test.rs` はフィルタがプロセスに 1 つのため専用バイナリに 1 本だけ置いている。
- 専用バイナリに `#[test]` を 1 本だけ置き、ゴーストを 1 つずつ順に読めば、登録簿は常に 0 個か 1 個になる。`serial` 用のクレートを足す必要は無い。
- 振り分けの規則そのものは、`registry.rs` の `mod tests` から private な `GlobalLoggerRegistry::new()` で登録簿を作って単体テストできる（プロセス全域の登録簿を使わない）。
- `PASTA_LOG` はフィルタを上書きする（`tracing_init.rs` の `build_filter`）。`tests/common` の `#[ctor]` は `PASTA_DEBUG`・`PASTA_DEBUG_PORT` だけを消しているので、`PASTA_LOG` を足す。
- `unload` は done ack を受けてから戻り、ack は `drop(shiori)` の後に送られる。順序を直した後は、ack の時点でロガーは破棄済み（フラッシュ済み）なので、`unload` の直後にログファイルを読める。

### 8.3 `PastaShiori::drop` の順序と不変条件

- 現行: `SHIORI.unload` → 登録解除 → 登録解除のログ → 関数のキャッシュを捨てる → ランタイム破棄（永続化保存・DAP バックエンドの片付け）。
- 変更後: 文脈を張る → `SHIORI.unload` → 関数のキャッシュを捨てる → ランタイム破棄 → 登録解除のログ → 登録解除。
- `actor/thread.rs` は `drop(shiori)` → `drop(rx)` → done ack の順で、本仕様で変えない。「ack を受け取った時点で VM と DAP バックエンドの解放が済んでいる」（`internals/shiori.md`）は保たれる。
- 順序を直すと、ロガーの最後の `Arc` が `unregister` の中で落ちる。現行の `unregister` は `loggers.remove()` の戻り値をミューテックスを持ったまま破棄するため、`WorkerGuard` のフラッシュ待ちの間、他のスレッドの `make_writer` が待たされる。外したロガーをロックの外で破棄する（`register` の置き換えも同じ）。
- 再読み込みの分岐（`PastaShiori::load` の先頭）も同じ順序の問題を持つ。FFI 経路では通らない（アクターごとに新しい `PastaShiori` を作る）が、`PastaShiori` を直接使うテストでは通る。`Drop` と共通のメソッドにまとめる。

### 8.4 撤去する公開 API の利用者

- リポジトリ内の呼び出し元はテストだけ（`tests/loader/config_test.rs`・`tests/runtime/unit_test.rs`）。
- リポジトリに CHANGELOG ファイルは無い。告知はリリース時に `release-workflow` が扱う（本仕様の範囲外）。

## 9. その他の発見

- `.claude/skills/pasta-ghost-authoring/SKILL.md` の 355 行付近（手書き部分）が `[lua]` を載せている。生成物ではないので、手で消す（要件 6.6）。
- `PastaLoader::load_with_config` は `LoadDirGuard` を張らない。組み込みでは、呼び出し側が張らない限りローダのログに文脈が無い。
- 不正な `file_path` のテストの先例: `logging_filter_reload_test.rs` が `file_path = "../outside.log"` で `PastaLoader::load` が成功することを見ている（フィルタの反映だけを検証）。
- FFI 入口スレッドの観測ログのレベル: `actor.try_send` の成功は trace、失敗・timeout・drop は debug。`teardown: Stop enqueued` と `lifecycle: actor spawned` は debug。既定の `info` では、FFI 入口スレッドは正常時にログを出さない。

## 10. 設計の統合（Synthesis）

### 一般化

- 要件 4.1・4.4・4.5 は「設置パスを知らないスレッドのログ」という 1 つの問題である。入口ごとにガードを足すのではなく、振り分けの規則に「文脈なし → 唯一のロガー」を足して 1 か所で解く。
- 要件 4.2 と 5.2（組み込み）は「設置パスを知っているのに文脈を張っていない」という 1 つの問題である。知っている側（アクタースレッド・ローダ）が張る。
- 要件 4.3 と再読み込みの順序は同じ処理なので、`release_runtime` の 1 か所にまとめる。

### 作るか、既存を使うか

- 新しい依存は足さない。テストの直列化は、専用バイナリという既存の型で足りる。
- ロガーの寿命を FFI 側へ延ばす案（4.2 節の (ii)）は採らない。順序を直せば、待ち時間切れのときロガーは登録されたままである。

### 簡素化

- `windows.rs` に static を足してガードを張る案（案 A）は採らない。入口が増えるたびに漏れる構造が残る。
- 必須ライブラリを黙って足す案は採らない（前提 A3）。
- 既に登録されている既定ロガーを段階 1.5 で再利用する最適化は採らない。組み込みの再読み込みで前回の設定のロガーが残っている場合に、既定ファイルへ切り替わらないためである。常に既定の設定で作り直す。
- 必須ライブラリの定数は公開しない。rustdoc の文章で示す。

## 11. 設計の決定

| ID | 決定 | 採らなかった案 |
| -- | ---- | -------------- |
| D1 | `default_libs` を `runtime/runtime_config.rs` へ移し、`pasta_lua::default_libs`・`pasta_lua::loader::default_libs` は再エクスポートで保つ | `sections.rs` に残す（設定セクションの型でない関数が孤立し、`runtime` → `loader` の依存が残る）。`loader` の公開パスを消す（破壊が 1 つ増える） |
| D2 | 必須は 2 段。VM の構築は `std_package`、ローダ経由は加えて `std_string`・`std_table`・`std_math`・`std_os`。エラーは `ConfigError::MissingRequiredLibrary(String)`。検査は `RuntimeConfig::ensure_libs`（`to_stdlib()` は変えない） | `to_stdlib()` で検査する（フラグ計算のテストが壊れる）。`math` などを VM の構築の必須にする（`math` 無しの VM を作れる現行の挙動が壊れる）。`package` を黙って足す |
| D3 | 案 C。`GlobalLoggerRegistry::resolve` で「文脈あり → そのロガー（未登録なら捨てる）／文脈なし → 登録がちょうど 1 つならそれ／それ以外は捨てる」。アクタースレッドの入口と `load_with_config` で文脈を張る。`PastaShiori` は `release_runtime` で「ランタイム破棄 → 登録解除のログ → 登録解除」。teardown の異常の warn は今の位置（FFI 入口）のまま、規則で届ける（4.2 節の (i)） | 案 A（入口ごとのガード）。文脈ありで未登録のときも唯一のロガーへ流す（別のゴーストへの誤配になりうる） |
| — | 不正な `file_path`: 4.3 節の案 2（段階 1.5 が既定の設定で作り直して登録する）。warn は登録の後に出す。判定は「最初の要素がちょうど `profile`、その後に要素が続く、`..` を含まない」 | 案 1（文言だけ直す。組み込みでロガーが無くなる） |

## 12. リスク

| リスク | 対処 |
| ------ | ---- |
| ローダ経由の必須ライブラリの一覧が `pasta_scripts` の変更に追随しない | 設計の Revalidation Triggers に挙げる。実装タスクで、構成を 1 つずつ外して起動とリクエストが通るかを実行で確かめる |
| 「文脈なし → 唯一のロガー」が、組み込みで意図しないログを拾う | ロガーが 2 つ以上なら捨てる。1 つのときは、そのプロセスのログはそのゴーストのものとみなせる |
| FFI 入口スレッドが登録簿のミューテックスを取る | 取るのはログのイベントがフィルタを通ったときだけで、保持は表を引く間だけ。mailbox の送信パスは変えない |
| ログの E2E がフラッシュの時点に左右される | ロガーの破棄の後に読む（`unload` の戻りの後） |
