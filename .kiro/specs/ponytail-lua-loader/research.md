# ギャップ分析: ponytail-lua-loader

## Summary

- **Feature**: `ponytail-lua-loader`
- **Discovery Scope**: Extension（既存の `loader/`・`runtime/` の直しと撤去。新しい部品は無い）
- **測った所**: ワークツリーの HEAD `839e07b8`（main の v0.4.4 `6aabf087` に、book の雑記の 2 コミットを足したもの。持ち場のソースは main と同じ）。2026-11-01。
- **Key Findings**:
  - 監査の 2〜8 番は、mlua 0.12（`dfb23703`）・`toml-serialization`（`e221ebf3`）・`mlua-stdlib-removal`（`49fd1f9d`）・`shiori4-v3-portable`（`40c33532`）の後も、**すべて残っている**。行番号も brief の 2026-10-24 の再測定のとおり（下の表）。mlua 0.12 が持ち場に入れた変更は、`mlua::String` → `mlua::LuaString` の読み替えだけで、指摘の中身は変わらない。
  - 監査のやり直しで、同じ領域に **新しい指摘が 6 つ** 増えた（本番で作られない `LoaderError` の種類・働かない旧 `scene_dic.lua` の片付け・`.pasta` のキャッシュ書き込みの失敗で古い結果が動く・大文字と小文字だけ違うモジュール名のキャッシュの衝突・`lua_search_paths` の `;`・`?`・直すべきマニュアルの頁の追加）。
  - brief が挙げたマニュアルの行番号のうち、`internals/shiori.md` の 3 か所は今の行と合わない（下の「マニュアル」）。
  - 衝突の規則は、`pasta_check` の `module_name_of` とローダーの `CacheManager::source_to_module_name` が、別々の実装をパリティのテスト（`crates/pasta_check/tests/validate_parity_test.rs:98-122`）で揃えている。ローダーが `source_to_module_name` の結果で衝突を見れば、`pasta_check` と同じ組み合わせになる。
  - 規模は M・危険は低い。どの直しも既存の型の中で閉じる。いちばん広く波及するのは、古い構築の道のテストの移し替え（`runtime_api_test.rs` の `from_loader` 15 か所・`load_scene_dic` 11 か所）。

## 監査のやり直し（2026-10-19 の控えとの突き合わせ）

`/ponytail:ponytail-audit` の観点（使われない抽象・実装が 1 つの継ぎ目・テストだけが使う道・データを失う端・黙って捨てる誤り）で、`crates/pasta_lua/src` の `loader/`・`runtime/`・`presentation/`・`encoding/` と `lib.rs` を読み直した。プラグインのスキルは呼ばず、コードを読んで確かめた。cargo test と clippy は走らせていない。「使われない」の判断は、`crates/` 全体の grep による。

### 控えの指摘の今

| # | 指摘 | 今 | 今の場所 |
| - | ---- | -- | -------- |
| 1 | 読めないセーブを空で上書き | **範囲外**（`toml-serialization` PR #200 で直った） | `runtime/persistence.rs`（697 行に書き直された） |
| 2 | キャッシュの判定が `source_mtime > cache_mtime` | 残る | `loader/cache.rs:153`（`needs_transpile` 137-167） |
| 3 | 同じモジュール名の辞書が後勝ちで消える | 残る | `loader/process.rs:77-97`（`.pasta`↔`.lua` だけ見る）・`process.rs:129`・`:212`（`module_names` に重複が入る）・`cache.rs:187` の `save_cache`（同じキャッシュのパスへ上書き） |
| 4 | finalize 前の `@pasta_search` が今回変換したファイルだけを見る | 残る | `loader/process.rs:182`（`merge_from`）→ `loader/mod.rs:241-249` → `runtime/mod.rs` の `with_config_and_source_map`（`crate::search::register` に渡す） |
| 5 | パスの `;` で `package.path` が割れる | 残る | `loader/context.rs:101-116`（`generate_package_path`）・`runtime/searcher.rs:79-87`（`candidate_paths` が `;` で割り、`?` を全部置き換える） |
| 6 | `presentation/` と `RendererInjection` | 残る | `presentation/mod.rs`（292 行）・`marker.rs`（126 行）・`runtime/renderer_injection.rs`（213 行）・`runtime/module_registry.rs:156`・`runtime/factory.rs:196-207`・`runtime/mod.rs:39` の再輸出。`pasta_shiori` などほかのクレートからの参照は 0 |
| 7 | テストだけが使う古い構築の道 | 残る | `runtime/factory.rs:46`（`from_loader`）・`:149`（`_scene_dic_path`）・`:284`（`load_scene_dic`）・`loader/mod.rs:337`（`TranspileResult`）・`lib.rs` の再輸出。使うのは `tests/runtime/runtime_api_test.rs`（`from_loader` 15・`load_scene_dic` 11・`TranspileResult` 4）と `tests/runtime/encoding_test.rs`（`from_loader` 4） |
| 8a | 孤立したキャッシュの警告が 2 回 | 残る | `cache.rs:414-420`（常に、件数 1 行＋各ファイル）と `loader/mod.rs:193-198`（`debug_mode` のときに各ファイルをもう一度） |
| 8b | `Encoder` は実装が 1 つ・`Encoding::OEM` はテストだけ | 残る | `encoding/mod.rs`。`Encoder` を `use` するのは `runtime/enc.rs:23`・`runtime/ffi_load/resolve.rs:6`・`tests/ffi/compare_test.rs:12`・`lib.rs:54`。`pasta_shiori` は自前の `Encoder`・`Encoding::OEM`（`src/util/hglobal/enc.rs`）を持ち、こちらに依存しない |

### 新しく見つけた指摘（同じ領域）

- **N1. 本番で作られない `LoaderError` の種類**（小物・撤去）
  - `LoaderError::Parse`・`LoaderError::parse()`・`parse_with_source()` は、`loader/error.rs` の中（とそのテスト）でしか作られない。`.pasta` の構文の誤りは、`process.rs` が `TranspileFailure` の文字列（`"Parse error: …"`）にして `PartialTranspileError` へまとめる。
  - `LoaderError::Transpile(#[from] TranspileError)` も、`?` で変換される所が無い（`process.rs` は `match` で文字列にする）。テスト `error.rs:424` だけが作る。
  - `GlobPattern`・`GlobTraversal` は `discovery.rs:88-89` の `?` で使われている。消さない。
- **N2. 働かない旧 `scene_dic.lua` の片付け**（小物・撤去）
  - `cache.rs` の `generate_scene_dic` が、キャッシュの直下の古い `scene_dic.lua` を毎回探して消す。版が変わると `prepare_cache_dir` がキャッシュを丸ごと消すので、旧版の置き場所が残ることはない。テスト `tests/loader/cache_test.rs:579`（`test_generate_scene_dic_removes_legacy_location`）だけが通る。
- **N3. `.pasta` のキャッシュの書き込みに失敗すると、古い変換結果が黙って動く**（バグ・2 番と根が同じ）
  - `process.rs` の `.pasta` の道は、`save_cache` の失敗を `warn!` だけにして続ける（「cache write failure is not fatal」）。前のキャッシュがあれば `scene_dic.lua` の `require` がそれを読む。無ければ `module not found` になり、本当の原因（書き込みの失敗）が隠れる。
  - 辞書の `.lua` の道は、同じ失敗を `TranspileFailure` にして読み込みエラーにする。2 つの道で扱いが違う。
  - 2 番の直し（キャッシュに元の更新時刻を写す）を入れると、書き込みや時刻の設定の失敗の扱いを決める必要がある。要件 1.6 に【前提】で書いた。
- **N4. 大文字と小文字だけ違うモジュール名が、Windows でキャッシュのファイルを共有する**（バグの疑い・要確認）
  - 例：`dic/A-b.pasta` → `pasta.scene.A_b`、`dic/a_b.pasta` → `pasta.scene.a_b`。Lua と `pasta_check` には別の名前だが、キャッシュのファイル `pasta/scene/A_b.lua` と `pasta/scene/a_b.lua` は、大文字と小文字を区別しない NTFS では同じファイルになる。後に書いた方が勝ち、片方の辞書の内容が黙って消える（3 番と同じ症状）。
  - `pasta_check` は名前を大文字と小文字を区別して比べるので、これをエラーにしない。
  - 同じフォルダに大文字と小文字だけ違う 2 つのファイルは置けないので、起きるのは `-`・`.` と `_` の違いや、サブフォルダの名前の違いが重なったときだけである。要件には入れていない（持ち越し 4）。
- **N5. `lua_search_paths` の項目の `;`・`?`**（5 番と根が同じ）
  - `package.path` は `base_dir.join(lua_search_paths の各項目)` から作る（`context.rs:101-116`）。`pasta.toml` の `[loader] lua_search_paths` に `;` か `?` を書くと、ゴーストのフォルダの `;` と同じく割れる。brief は「`load_with_config` の入口で断る」とだけ書いている。入口（設定を読む前）では項目が見えない。要件 4.2 に【前提】で書いた。
  - `transpiled_output_dir` も既定で `lua_search_paths` に含まれる（`profile/pasta/cache/lua`）。作者が別の値にしたときは `lua_search_paths` にも書く運用なので、`lua_search_paths` を見れば足りる。
- **N6. 直すべきマニュアルの頁が brief より多い**（下の「マニュアル」）

### 指摘にしなかったもの（記録）

- `PastaLuaRuntime::new`・`with_config`・`exec_file`・`exec_named`・`debug_source_map`・`debug_source_mode`、`RuntimeConfig::from_libs`・`with_debug` は、本番から呼ばれず、テストだけが使う。ただし、どれも「ランタイムを組み立てて動かす」公開の API で、`runtime_api_test.rs` 以外の多くのテスト（`search/module_test.rs` 20 か所など）が頼っている。抽象の重ね（実装が 1 つの継ぎ目）ではないので、指摘にしない。`exec_file` だけは `runtime_api_test.rs` しか使わないので、テストを移すときに残すかを設計で決める。
- `LoaderContext::new` は、`from_loader` を消すと、テスト（`encoding_test.rs`・`runtime_api_test.rs`）と `context.rs` の単体テストだけが使う形になる。テストを `PastaLoader::load` へ移せば使い手が無くなるので、そのときに一緒に消すかを設計で決める。
- `LoaderError::MetadataError` は、`needs_transpile` の誤りが `process.rs` の `unwrap_or(true)` で飲まれるので、読み込みエラーとしては作者へ届かない。マニュアル（`transpiler.md:336`「更新時刻を取得できない場合もトランスパイルする」）のとおりの挙動で、要件 1.4 で今の挙動として固めた。
- persistence（`persistence.rs`・`lifecycle.rs`・`log.rs`）は `toml-serialization` が書き直した所で、この spec の範囲外。監査の対象からも外した。

## brief の読み方（食い違いの解き方）

- brief の Problem の 1 つ目、Current State の 1 番、Desired Outcome の 1 つ目、Approach の 1 つ目、Scope の `pasta-persistence.md`、Boundary Candidates の「セーブの保護」は、1 番が `toml-serialization` へ移る前の記述である。brief の「申し送り（2026-10-24 の 0.4.3 の組み替えより）」と「2026-10-24 棚卸の再測定」を正として、要件の範囲に入れなかった。
- brief の Out の `context.rs` は `src/context.rs`（`TranspileContext`、`ponytail-lua-transpiler` の持ち場）であり、5 番の `loader/context.rs`（`LoaderContext`）とは別のファイルである。要件の Boundary Context に書き分けた。
- brief が「見つけた穴」に挙げた roadmap の行（`.kiro/steering/roadmap.md` 211 行）は、今は 192 行にあり、「Must fix は `toml-serialization` へ移した」と直っている。

## マニュアル

brief の行番号と今の行の対応（Explore の調べによる）。

| 頁 | brief の行 | 今の場所と中身 |
| -- | ---------- | -------------- |
| `internals/transpiler.md` | :336-337 | :336「ソースの更新時刻がキャッシュより新しければトランスパイルする」（合う）。ほかに :339（`.pasta` と同名の `.lua` を無視）・:382（無効化は版・別名表の指紋・更新時刻だけ）・:138 と :383（段階 1 のレジストリは finalize で置き換わる） |
| `internals/loader.md` | 行の指定なし | :170（同名の `.lua` を警告つきで除く）・:180-202（モジュール名の節。:201 は拡張子だけ違う衝突だけを書き、`.pasta` 同士・`.lua` 同士を書いていない）・:214-225（`package.path`）・:78・:114・:265・:335（`from_loader*`） |
| `reference/startup.md` | 行の指定なし | :36（`?` にモジュール名が入る）・:52-56（`package.path` の UTF-8）。`;`・`?` を含むパスの記述は無い |
| `internals/shiori.md` | L25・L87-88・L444-455 | **合わない**。今は :33（責務の箇条「presentation マーカーとレンダラ注入」）・:106-107（ファイルの表の `presentation/`・`renderer_injection.rs`）・:575-587（節「presentation マーカーとレンダラ注入」）・:627（継ぎ目の表の `register_sakura_script_module`）・:653-654（ソースの一覧） |
| `internals/talk-output.md` | L28・L341 | 合う。どちらも `shiori.md#presentation-マーカーとレンダラ注入` へのリンク |
| `internals/execution-model.md` | :43 | 合う（`from_loader`（旧経路。テストが使う））。ほかに :51（`renderer_injection.rs` の行）・:117-144（構築の順） |
| `.kiro/steering/structure.md` | — | :98（`renderer_injection.rs`）・:109-110（`presentation/`。型の名前が `PresentationEvent`・`RenderBoundary` で、book の `PresentationMarker` と食い違う古い記述） |

brief に無く、直す必要がある頁（N6）。

- `lua/modules/pasta-search.md:23`・`lua/modules/index.md:42`：finalize 前は「結果は使えない」。要件 3 を入れると、決まった結果（【前提】見つからない）に書き直す。**`lua/modules/` は `gen-skill-refs.mjs` の生成の元**なので、`.claude/skills/pasta-lua-coding/references/` の生成物も変わる。
- `internals/internal-modules.md:429`・:444：「`@pasta_search` は VM の構築時にも、トランスパイル時のレジストリから一度登録されている」。
- `internals/logging-encoding.md:66-71`（`Encoder`・`Encoding` の表、「`Encoding::OEM` を使うのはテストだけ」）・:219（GetOEMCP）。
- `index.md:89`：SHIORI の章が見張るパスに `presentation/` と `runtime/renderer_injection.rs` がある。
- `debug/validate.md:146-158`：`pasta_check` の衝突の規則とコード。ローダーも同じ組み合わせをエラーにするようになるので、一言足すかを設計で決める（必須ではない）。

`gen-skill-refs.mjs` は、`grammar/*`・`reference/pasta-toml.md`・`reference/runtime-failures.md`・`lua/modules/*` の 24 章から、`.claude/skills/{pasta-ghost-authoring|pasta-lua-coding}/references/` を作る。`internals/*` は対象外。`--check` で古い生成物を見つける。

## Requirement-to-Asset Map

| 要件 | 今ある資産 | ギャップ |
| ---- | ---------- | -------- |
| 1 キャッシュの日付 | `CacheManager::needs_transpile`・`save_cache`・`get_mtime`（`cache.rs`）。`.pasta` と `.lua` の両方の道が `needs_transpile` を使う | **Missing**: 変換のときの元の更新時刻を覚える手段（キャッシュのファイルの更新時刻に写す・別の記録に書く・中身のハッシュ）。**Constraint**: `cache_test.rs:228` の `test_needs_transpile_cache_newer`（キャッシュが新しければ変換しない）は、意味が変わるので書き直しになる。`:205` も見直す。**Missing**: N3（書き込みの失敗の扱い） |
| 2 同じモジュール名 | `discover_all_files`（`process.rs:28-100`）が `.pasta`↔`.lua` を見る。`LoaderError::InvalidFileName` が `init.pasta` を断る形の前例。`source_to_module_name` は `pasta_check` とパリティのテストで揃っている | **Missing**: `.pasta` 同士・`.lua` 同士の重複の検出と、ファイル名つきの新しい `LoaderError` の種類。**Unknown**: N4（大文字と小文字） |
| 3 finalize 前の検索 | `process_incremental` が `TranspileContext` を `merge_from` で集め、`from_loader_with_scene_dic` → `with_config_and_source_map` → `crate::search::register` へ渡す。finalize（`runtime/finalize.rs`）が `@pasta_search` を登録し直す | **Missing**: 渡す中身を常に空にする（`merge_from` をやめる）。`search/context.rs` は範囲外なので触らない。空の表で `search_scene` が nil を返すことは設計で確かめる（**Research Needed**） |
| 4 パスの `;`・`?` | 入口の `DirectoryNotFound` の前例（`loader/mod.rs` の `load_with_config` の先頭）。`LoaderContext::from_config` は `std::path::absolute` を使い 8.3 の形を保つ | **Missing**: 入口での文字の検査と、新しい `LoaderError` の種類。**Constraint**: 入口では設定が読めていないので、`lua_search_paths`（N5）は設定を読んだ後に見る。`searcher.rs` の FFI の道（`lua-cmodule-robustness`）は変えない |
| 5 使われない抽象 | `sakura_script::register` を直に呼べば足りる（`default_sakura_renderer` の中身がそれだけ）。`renderer_injection.rs` のテスト `default_injection_renders_byte_identically_to_direct_register` が、バイト不変の確かめの前例 | **Missing**: 撤去と再輸出の削除。`Encoder` をやめるなら `Encoding` の固有のメソッドへ寄せ、`enc.rs`・`ffi_load/resolve.rs`・`tests/ffi/compare_test.rs` の `use` を直す。N1・N2 の撤去 |
| 6 古い構築の道 | `tests/common/mod.rs` の `copy_fixture_to_temp`（:159）・`create_temp_with_pasta`（:219）と `PastaLoader::load` の組み合わせが、`tests/loader/*.rs` で広く使われている | **Missing**: `runtime_api_test.rs`・`encoding_test.rs` の移し替え。**Constraint**: `PASTA_DEBUG` を読む道を通るテストには中和の守り（`#[ctor]` の既存の守り）を付ける |
| 7 孤立キャッシュの警告 | `find_orphaned_caches`（`cache.rs:397-425`）と `loader/mod.rs:193-198` | **Missing**: 片方の削除。戻り値の `Vec` は `mod.rs` の 2 回目の警告にしか使われていない |
| 8 マニュアル | 上の「マニュアル」の表 | **Missing**: 頁の直しと `gen-skill-refs.mjs` の生成。公開 API の断り書きは、README・`tech.md` にまだ無い（ponytail の 10 本のうち `ponytail-check-release` だけが完了し、それは `pasta_check` の中で pub API を消していない） |
| 9 非退行 | `lua_passthrough_test.rs:185-340`（`.pasta` が `.lua` に勝つ・ハイフンの正規化・`dic` に似たフォルダ）・`cache_test.rs`・`startup_test.rs` | **Constraint**: pasta_lua のテストは `sample.generated.lua` の改行を書き換える。`git checkout --` で戻してコミットに混ぜない |

## Implementation Approach Options

### Option A: 既存の部品の中で直す（brief の Approach そのまま）

- キャッシュ：`save_cache` の後でキャッシュのファイルの更新時刻を元の `.pasta` の更新時刻にそろえ（`std::fs::File::set_modified`。edition 2024 なので使える）、`needs_transpile` の比べ方を「等しくなければ変換」にする。
- 衝突：`discover_all_files` で `.pasta` 同士・`.lua` 同士の重複を集め、`LoaderError` の新しい種類で返す（`InvalidFileName` と同じ形）。
- finalize 前の検索：`merge_from` をやめ、空の `TranspileContext` を渡す。
- パス：`load_with_config` の入口でゴーストのフォルダを、設定を読んだ後で `lua_search_paths` を検査する。
- 撤去：`presentation/`・`renderer_injection.rs`・`from_loader`・`load_scene_dic`・`TranspileResult` などを消し、`module_registry.rs` は `sakura_script::register` を直に呼ぶ。
- ✅ 差分が最小。新しいファイルも記録の形式も要らない。
- ✅ キャッシュの判定が 1 か所（`needs_transpile`）で閉じる。
- ❌ 更新時刻の分解能が粗いファイルシステム（FAT32・exFAT）や、時刻の設定を許さない所では、キャッシュの更新時刻が元とずれて**毎回変換し直す**かもしれない（安全な側に倒れるが、起動が遅くなる）。**Research Needed**。

### Option B: 元の更新時刻（か中身のハッシュ）を別の記録に書く

- キャッシュのファイルの更新時刻は触らず、`profile/pasta/cache/` に「元のパス → 変換したときの更新時刻（か中身のハッシュ）」の表を置く。
- ✅ ファイルシステムの時刻の扱いに頼らない。ハッシュなら、日付を保ったまま中身を変えるコピーにも効く。
- ❌ 新しい記録の形式・読み書き・壊れたときの扱いが増える（ponytail の「最小の直し」に反する）。ハッシュは毎回すべての辞書を読む費用がかかる。
- ❌ brief の決定（`set_modified` と `!=`）から外れる。

### Option C: 混ぜる

- 判定は Option A。時刻の設定に失敗したときだけ、そのキャッシュを「次も変換し直す」状態にして（たとえばキャッシュを消して）続ける。
- ✅ Option A の欠点（設定の失敗）を、記録を増やさずに塞げる。
- ❌ 失敗の扱いの分岐が 1 つ増える。N3（書き込みの失敗を読み込みエラーにするか）と合わせて決める必要がある。

衝突・検索・パス・撤去には、Option A の形のほかに現実的な代案が無い。代案の比べ方が意味を持つのは、キャッシュの判定（要件 1）だけである。

## Effort and Risk

- **Effort: M（3〜7 日）**。直しはどれも小さい（各 10〜60 行）が、撤去の波及（再輸出・マニュアル 10 頁ほど・テストの移し替え 30 か所ほど）が広い。brief の見積もり（9〜12 タスク）と合う。
- **Risk: Low**。既存の型の中で閉じ、外のクレート（`pasta_shiori`・`pasta_check`）は撤去する API を使っていない。気をつける所は 2 つ。
  - `test_needs_transpile_cache_newer` のように、今の挙動を前提にしたテストの書き直し。
  - 他の spec との重なり。`ponytail-lua-transpiler`（`lib.rs` の再輸出）、0.5.0 の `scene-attribute-store`（`runtime/finalize.rs`）。この spec は `finalize.rs` に触らずに済む見込み。

## 設計への申し送り

- **推す形**: Option A（brief の Approach）。要件 1.6 と N3 を合わせて、時刻の設定の失敗も「読み込みエラー」か「次も変換し直す」のどちらかに決める（Option C の要素）。
- **決めること**
  - 新しい `LoaderError` の種類の名前と、表示の文（`X-ERROR-REASON` に 1 行で入る。`pasta_check` の文「モジュール名 {module} が {others} と同じです。どちらかの辞書の内容が失われます」に寄せるか）。
  - `;`・`?` の検査をどこで行うか（ゴーストのフォルダは入口、`lua_search_paths` は設定を読んだ後）。
  - `from_loader_with_scene_dic` の引数から `context` 自体を消すか（常に空なら渡す意味が無い）、空を渡すか。消すなら `process_incremental` の戻り値も変わる。
  - `Encoder` をやめた後の `Encoding` の形（固有のメソッドにする。`OEM` を消すと enum の値が 1 つになるので、関数 2 つにするかも含めて）。
  - `LoaderContext::new` と `exec_file` を、テストの移し替えの後も残すか。
- **Research Needed**
  - FAT32・exFAT・ネットワークドライブで `set_modified` と更新時刻の読み戻しが一致するか（毎回変換し直さないか）。
  - 空の登録表の `@pasta_search` で、`search_scene`・`search_word` などが nil を返し、エラーにしないこと（`search/context.rs` は範囲外なので、今の挙動の確かめだけ）。
  - finalize 前の検索の結果を頼りにしているテストやサンプルが無いこと（`tests/fixtures` と `pasta_sample_ghost` の `main.lua` には見当たらなかった）。

## 要件ディスカッションへの持ち越し

1. **finalize 前の検索の結果**（要件 3.2）：常に「見つからない」にする（brief の「merge を消して空にする」）で良いか。代案は「finalize 前に呼ぶとエラーにする」。
2. **`.pasta` のキャッシュの書き込みの失敗**（要件 1.6・N3）：今の「警告だけで続ける」をやめ、読み込みエラーにして良いか。時刻の設定の失敗も同じ扱いにするか。
3. **`lua_search_paths` の `;`・`?`**（要件 4.2・N5）：ゴーストのフォルダと同じく読み込みエラーにして良いか。
4. **大文字と小文字だけ違うモジュール名**（N4）：この spec でローダーのエラーにするか。するなら、`pasta_check` の規則も揃える必要があり、`pasta_check` の側を触る（brief の範囲の外に手が届く）。しないなら、記録だけにする。
5. **`Encoder`・`Encoding::OEM` の直し方**（要件 5.4）：brief の Approach に直し方が無い。`OEM` を消し、`Encoder` の trait をやめる形で良いか。
6. **新しく見つけた撤去**（要件 5.5・5.6、N1・N2）：本番で作られない `LoaderError` の種類と、旧 `scene_dic.lua` の片付けを、この spec で消して良いか。
7. **孤立キャッシュの警告の残し方**（要件 7.2）：常に出る方を残し、`debug_mode` のときの重ねを消す形で良いか。
8. **公開 API の断り書き**（要件 8.5）：README と `tech.md` の断り書きは、まだどの ponytail の spec も書いていない。この spec が最初に main へ入るなら書く、という条件つきで良いか。
9. **マニュアルの範囲の追加**（N6）：`lua/modules/pasta-search.md`・`lua/modules/index.md`・`internal-modules.md`・`logging-encoding.md`・`index.md` を範囲に足して良いか（`lua/modules/` はスキルの参照の生成物も変わる）。
