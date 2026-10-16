# ギャップ分析: pasta-check-dic-validate

- **実施日**: 2026-10-16（worktree `claude/pasta-check-dic-validate-3abbef`、main `3071114a` の上）
- **対象**: `requirements.md`（Requirement 1〜10・未決事項 OQ-1〜9）と、今のコードベースの差
- **分析の方針**: 情報と選択肢を示し、決定は要件ディスカッションと設計に回す

## 1. 分析の概要

- **足りないもの**: `pasta_check` には辞書を読む部品が 1 つも無い（依存は `lexopt`・`md5`・`zip` だけ）。検査コマンドの入口・辞書の集め方・`pasta.toml` を読む処理・全体の検査・結果の出力は、すべて新しく作る。
- **使える部品**: 構文は `pasta_dsl`（`parse_str` が `pasta.dll` と同じ最初の 1 件、`parse_str_partial` が複数件）、名前の規則とシーンの前方一致の検索は `pasta_core`（`SceneRegistry::sanitize_name`・`SceneAliasTable`・`SceneTable::collect_scene_candidates`）にある。どちらも LuaJIT を含まない。
- **最大の分かれ道は「`pasta.dll` と同じ規則」をどう保つか**: モジュール名（`cache.rs`）・`pasta.toml` の別名表の読み方（`loader/config/mod.rs`）・辞書の集め方（`discovery.rs`）・シーンの登録名の作り方（`code_gen/scope_gen.rs`）は `pasta_lua` の中にあり、`pasta_lua` は LuaJIT をビルドする。写すと規則が二重になり、依存すると `cargo install pasta_check` が重くなる。さらに今のウェーブの約束で `loader/config/` は触れない。
- **読むだけでは「無い」と言い切れない**: Call の 5 段のうち 1・3・4 段は Lua で決まり、Lua ブロックの `function SCENE.名前` は 2・5 段の検索にも載る。見本の hello-pasta でも、静的な Call の飛び先 `transfer_req_to_var`（9 か所）は act のメソッドで、辞書の中にシーンは無い。飛び先の検査を誤りにすると見本のゴーストの `release` が止まる。
- **推奨の方向（決定ではない）**: 案 C（ハイブリッド）。検査は `pasta_check` の中に書き、依存は `pasta_dsl`・`pasta_core`・`toml`（と `glob`）にとどめる。`pasta_lua` の規則と同じ結果になることは、`pasta_check` の dev-dependency に `pasta_lua` を置いた一致テストで守る（dev-dependency は `cargo install` でビルドされない）。

## 2. brief の行番号の照合（2026-10-10 `add05022` → 今の main `3071114a`）

| brief の参照 | 今 | 判定 |
| ------------ | -- | ---- |
| `crates/pasta_check/src/main.rs` 16〜20 行（`Release`・`Help`・`Version`） | 同じ。全 319 行、テストは 146〜319 行 | 一致 |
| `crates/pasta_check/Cargo.toml` 18〜26 行（`lexopt`・`md5`・`zip`） | 同じ。全 29 行、dev は `tempfile` だけ | 一致 |
| `scene-anchor-link/brief.md` 52 行 | 同じ（Out の節） | 一致 |
| `failure-output-unification/brief.md` 45・66 行 | 同じ。spec は brief.md だけ（要件・設計は未作成） | 一致 |
| `.github/workflows/manual.yml` 128〜135 行の「構文を検証するサブコマンドが無いため」 | 位置は同じ（コメント 128〜133・手順 134〜135）。文言は「pasta_check CLI に .pasta 構文検証サブコマンドは無い（release のみ）ため」（129 行） | 文言のずれ |
| `tutorial_stages_test.rs` 363〜445 行 | 同じ。`GLOBAL_KEYS` 363〜365、`E2E_PREFIXES`（`Kick`・`Gate`）367〜368、`global_scene_names` 370〜397、前方一致の検査 405〜445 | 一致 |
| `loader/cache.rs` 238〜252 行 `source_to_module_name` | 同じ。249 行で `.replace(['.', '-'], "_")` | 一致 |
| `loader/process.rs` 77〜97 行 | 同じ（`.pasta` と `.lua` の重なりだけを見る） | 一致 |
| `scene_registry.rs` 244〜246 行 `sanitize_name` | 同じ | 一致 |
| `scene_registry.rs` 223〜227 行（`会話·A` と `会話_A` の説明） | 223〜227 行は非公開の `increment_counter`（228 行）の説明。`sanitize_name` 自身の説明は 235〜243 行 | 位置の意味のずれ（内容は brief のとおり） |
| `loader/config/mod.rs` 112 行 `parse_scene_aliases`・266〜297 行 `LoaderConfig` | 同じ。`parse_scene_aliases` は非公開、既定値は 299〜300 行 | 一致 |
| `book/src/reference/pasta-toml.md` 349 行「pasta_check は別名表を読まない」 | **377 行**へ移動（全 568 行。540 行ではない） | ずれ |
| `act.lua` 313〜322 行（5 段） | 313〜322 行は説明のコメント。本体 `ACT_IMPL.find_act_handler` は 332〜373 行 | 一致（補足） |
| `grammar.pest` 169〜170 行 | 同じ。ターゲットは変数に限らず任意の式（`call_target_expr = { expr }`）。AST は `CallTarget::{Static, Dynamic}`（`ast/action.rs` 95〜100 行） | 一致（補足） |
| `book/src/grammar/call-jump.md` 121 行 | 121 行は見出し「前方一致によるターゲット解決」、例は 125〜135 行 | 一致 |
| `pasta_lsp`（`pasta_dsl` だけに依存・診断は構文だけ・`analysis/mod.rs` 57〜77 行） | 同じ | 一致 |
| `book/src/SUMMARY.md`（67 行・47 章） | **80 行・60 章**。`pasta_check` は目次に無い | ずれ |
| `release.rs` 13〜50 行（5 段） | 同じ | 一致 |
| `parser/mod.rs` 142 行 `parse_file`・`partial.rs` 144 行 `parse_str_partial` | 同じ | 一致 |
| 章の数 47 を決め打ちした検査（`verify-scripts-test.mjs` 72 行、`talk/talk-test.mjs` 453〜454 行） | **数は 60**。`verify-scripts-test.mjs` は 78・81 行（72 行は部ごとの 13 章の検査）、`talk-test.mjs` 453〜454 行。ほかに `gen-skill-refs-test.mjs` 599 行・`link-check-test.mjs` 1193〜1224 行も 60 | ずれ（数と場所。対象のファイルも 2 つ増えた） |
| `release.yml` 295 行（`pasta_check` の公開に `-DependsOn` が無い） | 同じ（295〜299 行、299 行）。`pasta_dsl` 269・`pasta_lua` 279・`pasta_shiori` 289 行には `-DependsOn` がある | 一致 |
| `crates/pasta_sample_ghost/release.ps1` 181〜189 行が `pasta_check release` を呼ぶ | **162〜170 行**（呼び出しは 167 行）。181〜189 行は `pasta.dll.zip` の段 | ずれ |
| `tech.md` 71〜74 行・`structure.md` 189 行 | 同じ | 一致 |
| `cli_test.rs` 223 行・README 123 行・スキル 153 行 | スキルは 154 行。ほかは同じ | 軽微 |
| `gen-skill-refs.mjs` 50 行（`pasta-toml.md` が生成元） | 同じ。生成先は `pasta-ghost-authoring` のスキル | 一致 |

## 3. 今のコードの調べ

### 3.1 `pasta_check`

- 入口は `lexopt` の手書きのループ（`main.rs` 76〜144 行）。サブコマンドは `"release"` だけ受け付け、ほかは `UnexpectedValue`。使い方は標準エラー出力（59〜74 行）。
- 終了コードは 0 と 1 だけ。引数の誤りも `release` の失敗も `process::exit(1)`。
- 文言は英語（`Error: …`・`Warning: …`・`[1/5] Preparing release folder...`）。`release` はすでに「警告を出して続ける（`homeurl` が無い）」と「エラーで止まり nar を作らない（`install.txt` の誤り）」を分けている（`README.md` の「同梱バルーン」）。誤りと注意の分け方の前例になる。
- `release` の 5 段（`release.rs` 13〜50 行）: 準備・コピー・上書きコピー（`--copy`）・更新ファイル（警告は止めない）・nar。段 1 で前回の nar を消すので、後の段で止まると `--nar` の位置に nar は残らない。

### 3.2 `pasta.dll` の読み込み（検査が合わせるべき挙動）

- 辞書の集め方: `pasta_patterns`（既定 `dic/**/*.pasta`）から `.lua` のパターンも作る。`..` を含むパターンと `.pasta` で終わらないパターンは警告して飛ばす。`profile/` は除く（`loader/discovery.rs`・`process.rs`、マニュアル `pasta-toml.md` の `pasta_patterns`）。
- ファイル名 `init.pasta`・`init.lua` は読み込みエラー（`process.rs` 68〜75 行）。
- `.pasta` と `.lua` のモジュール名の重なりは、警告して `.lua` を捨てる（77〜97 行）。**`.pasta` どうしの重なりは見ない**。キャッシュの出力先はモジュール名から作るので（`cache.rs` 261〜264 行 `source_to_cache_path`）、後のファイルが前のファイルのキャッシュを上書きする。一方で Rust 側のシーンの登録は両方のファイルからマージする（`process.rs` の `combined_context.merge_from`）。検索の表には載るのに Lua の関数が無いシーンができる。
- 読み込み（`fs::read_to_string`＝UTF-8 必須）・構文（`pasta_dsl::parse_str`＝最初の 1 件）・トランスパイルのどれか 1 つでも失敗すると、全部のファイルを処理した後でゴーストの読み込み全体を失敗にする（`process.rs` 253〜265 行 `partial_transpile`）。
- トランスパイルで初めて分かる誤りは 3 つ: シーンの引数への代入（`element_gen.rs` 81 行）、式の中のプロパティ参照（41 行）、アクター無しの継続行（309 行）。`TranspileError` のほかの型（`UndefinedScene`・`UndefinedWord` など）は今は作られていない。

### 3.3 名前の規則と検索

- `pasta_core`（依存は `thiserror`・`fast_radix_trie`・`rand`。LuaJIT 無し）:
  - `SceneRegistry::sanitize_name`（公開）・`registered_name`・`register_global`・`register_local`。
  - `SceneAliasTable`（`empty`・`builtin_default`・`from_entries`・`resolve`）。
  - `SceneTable::from_scene_registry` と `collect_scene_candidates(module_name, prefix)`（空なら全グローバル、そうでなければそのグローバルの配下のローカルだけ）。重なった名前は誤りにせず、照合用の名前ごとに番号を振る。
- `pasta_lua`（LuaJIT を含む）にしか無い規則:
  - モジュール名 `CacheManager::source_to_module_name`（`cache.rs` 238〜252 行）
  - `pasta.toml` の読み方 `PastaConfig::load`（公開）・`parse_scene_aliases`（非公開、約 50 行）。config のモジュール自体は `serde`・`toml`・`pasta_core` だけを使うが、`LoaderError` が `mlua::Error` を持ち、クレートごと LuaJIT に依存する。
  - AST からシーンを登録する手順 `TranspileContext::register_global_scene`／`register_local_scene`（`context.rs` 41・62 行）、別名の置き換えと登録名（`code_gen/scope_gen.rs` 123・238 行）。
- 実行時にだけ決まるもの（Lua）:
  - 1 段: 実行中のグローバルシーンの `SCENE` テーブル。Lua ブロックの `function SCENE.名前` は、実行時の `finalize_scene` で 2・5 段の検索の表にも載る（マニュアル `internals/registry-search.md` 106 行）。
  - 3 段: act のメソッド（`ACT_IMPL`）と、SHIORI 用の act のメソッド（`SHIORI_ACT_IMPL`。`transfer_req_to_var` は `pasta_scripts/pasta/shiori/act.lua` 165 行）。
  - 4 段: `GLOBAL`（既定は `yield`・`チェイントーク`・`ゴースト終了`・`close_ghost`。`global.lua`）。`scripts/main.lua` などで足せる。
  - `scripts/` から `PASTA.create_scene`（`scene.lua` 129 行）でシーンを足せる。
- 選択肢の振り分け（`shiori/event/choice_select.lua` 70〜71 行）: `SCENE.search(id, 出したシーン) or SCENE.search(id, nil)`。1・3・4 段は無い。見つからなければ 204。`REG.OnChoiceSelectEx` の上書きと `＊OnChoiceSelectEx` のシーンが振り分けより優先される。
- 動的な飛び先の Call の失敗は、実行時に `【Call失敗：…】` を出す（`act.lua` 611〜702 行）。検索キーが nil のときはバルーンに出さず警告だけ。

### 3.4 エディタ・配り方

- `pasta_lsp` の依存は pasta の部品では `pasta_dsl` だけ。WASM にビルドする。診断は `parse_str` が失敗したときの `parse_str_partial` の行単位のエラーだけ。
- `toml` を使うのは今は `pasta_lua` だけ（workspace `1.1.2`）。`glob` も `pasta_lua` だけ。`pasta_check` に `pasta_dsl`・`pasta_core`・`toml`・`glob` を足しても `Cargo.lock` に新しい部品は入らない見込み（ビルドでは確かめていない）。
- `release.yml` の `pasta_check` の公開（295〜299 行）は、内部クレートに依存すると `-DependsOn pasta_dsl,pasta_core` が要る。公開の順は `pasta_check` が最後なので、指定を足すだけで済む。

## 4. 要件と資産の対応表

| 要件 | 使える資産 | ギャップ |
| ---- | ---------- | -------- |
| R1 入口（`--target`・使い方・読むだけ） | `main.rs` の `lexopt` のループ、`ReleaseArgs.target` の意味 | **Missing**: サブコマンドと引数。**Constraint**: 名前（OQ-9） |
| R2 辞書の集め方・`pasta.toml` | `pasta_lua` の `discovery.rs`・`PastaConfig`・`parse_scene_aliases`、`pasta_core::SceneAliasTable` | **Missing**: LuaJIT 無しで同じ結果を出す処理。**Constraint**: `loader/config/` は今のウェーブで触れない（roadmap 161 行）。**Unknown**: `pasta.toml` のほかのセクションの型の誤りまで誤りにするか |
| R3 構文 | `pasta_dsl::parse_str`（最初の 1 件・行・列）、`parse_str_partial`（複数件・列なし・推測の入るフォールバック） | **Missing**: 全ファイルを回して集める処理。**Unknown**: トランスパイルで初めて分かる 3 つ（OQ-4） |
| R4 モジュール名 | `CacheManager::source_to_module_name`（`pasta_lua`） | **Missing**: `.pasta` どうし・`.lua` どうしの重なりの検出（`pasta.dll` にも無い）。規則の写しか共有か |
| R5 シーン名の重なり | `SceneRegistry::sanitize_name`、`SceneAliasTable::resolve`、`scope_gen.rs` の登録の手順 | **Missing**: 書いた名前の違いを覚えたまま照合用の名前でまとめる処理。組み込みの名前の一覧（`GLOBAL` と act のメソッド）。**Unknown**: 一覧を `pasta_scripts` とずれないように保つ方法 |
| R6 飛び先 | `SceneTable::collect_scene_candidates`、AST の `CallTarget::Static`・`ChoiceNode` | **Missing**: AST からの登録と、Call の 2・5 段・選択肢の 2 段の再現。**Constraint**: 1・3・4 段と Lua で足したシーンは読めない（OQ-2） |
| R7 出力・終了コード | `main.rs` の終了コードの扱い、`release` の `Warning:` の前例 | **Missing**: 1 行 1 件の形・まとめの行・決まった順。**Unknown**: 文言の言語（OQ-5） |
| R8 `release` の関門 | `release.rs` の段 3 と段 4 の間 | **Missing**: 段の追加と表示。**Constraint**: 既存の 5 段の中身は変えない。hello-pasta とリリース CI が誤りなしで通る必要 |
| R9 配り方・規則の一致 | `pasta_core`・`pasta_dsl` が LuaJIT 無し | **Constraint**: `pasta_lua` に依存しない場合の規則の二重化。`release.yml` の `-DependsOn` |
| R10 文書 | `pasta-toml.md` 377 行、`gen-skill-refs.mjs`、README、スキル、steering | **Missing**: マニュアルのページ。**Constraint**: 章の数 60 を決め打ちした検査が 4 つのファイル（下の 7 節）。`manual-shell-guide` と同じウェーブで重なる |

## 5. 実装の選択肢

### 案 A: `pasta_check` の中に全部書く（依存は `pasta_dsl`・`pasta_core`・`toml`・`glob`）

- 入口・集め方・検査を `pasta_check` の新しいモジュールに置く。モジュール名の規則・`[scene.alias]` の読み方・辞書の集め方・シーンの登録の手順を `pasta_check` の中に写す。
- ✅ `cargo install` は軽いまま。今のウェーブの約束（`loader/config/` に触れない）を守れる。
- ✅ `pasta_lua` に手を入れない。
- ❌ 規則が 2 か所になり、将来 `pasta_lua` の規則だけ変わると検査の結果が実際の起動と食い違う（R9.2 を守れなくなる）。
- ❌ 組み込みの名前の一覧（`GLOBAL`・act のメソッド）も写しになる。

### 案 B: 規則を下の部品（`pasta_core`）へ移して共有する

- モジュール名の規則・別名表の読み方・（できれば）AST からシーンを登録する手順を `pasta_core` へ移し、`pasta_lua` と `pasta_check` の両方から使う。
- ✅ 規則が 1 か所。LSP（WASM）も将来使える（OQ-6）。
- ❌ `loader/config/` と `code_gen/` に手を入れる。今のウェーブの約束（roadmap 161 行「`loader/config/` は触らない」）にぶつかる。`pasta_core` は `pasta_dsl` に依存していないので、AST を扱う手順は置けない（依存の向きが変わる）。
- ❌ 別名表を TOML から読む処理を `pasta_core` に置くと、`pasta_core` が `toml` に依存する。
- ❌ リファクタリングを先に挟む分、規模が大きい。

### 案 C: ハイブリッド（案 A ＋ 一致テスト）— 推奨の方向

- 本体は案 A。`pasta_check` の dev-dependency に `pasta_lua` を置き、同じ入力に対して `pasta_lua` の規則（`source_to_module_name`・`PastaConfig::load` の別名表と `pasta_patterns`・`discovery`・トランスパイラが作るシーンの登録）と結果が一致することをテストで守る。組み込みの名前の一覧も、`pasta_scripts` の実物と照らすテストを置く。
- ✅ `cargo install` は軽いまま（dev-dependency はビルドされない）。規則のずれは CI のテストで止まる。
- ✅ 今のウェーブの約束を守れる。後で案 B へ移すこともできる。
- ❌ 一致テストの組み立てが要る。`cargo publish` は path だけの dev-dependency を外すので問題ない見込みだが、確かめが要る（Research Needed）。
- ❌ トランスパイルで初めて分かる 3 つの誤りは、写して判定するか範囲の外に置くかを決める必要が残る（OQ-4）。

### 参考: 案 D: `pasta_lua` に依存する

- `PastaConfig::load`・`discovery`・`CacheManager`・`LuaTranspiler` をそのまま使う。トランスパイルの誤りまで `pasta.dll` と完全に同じに判定できる。
- ❌ `cargo install pasta_check` が LuaJIT（C のコンパイラ）をビルドする。公開の順も `pasta_lua` の後になる。R9.1 の仮置きと反対。`LoaderConfig` の `transpiled_output_dir` に書き込まないよう、キャッシュを使わない経路が要る。

## 6. 規模とリスク

- **規模: M（3〜7 日）〜 L 寄り**。新しいモジュール（入口・集め方・構文・モジュール名・シーン名・飛び先・出力）に `release` の段の追加、一致テスト、文書 4 か所と steering。brief の見積もり 10〜14 タスクと合う。トランスパイルの 3 つの誤りを範囲に入れる・機械向けの出力を足すと上の端を超える。
- **規模の見直し（2026-10-16、OQ-6 で VSCode 拡張の診断を範囲に入れた後）: L**。共有の部品への切り出し・WASM の入口・`pasta_core` の WASM ビルド・拡張の側のファイル集めと診断の更新・VSIX のテスト・拡張の README で 5〜7 タスク増え、全体で 15〜20 タスクの見込み。
- **リスク: 中**。技術は既知（`pasta_dsl`・`pasta_core` の再利用）。リスクは規則の一致（案 C のテストで抑える）と、飛び先の検査の注意の多さ（hello-pasta で `transfer_req_to_var` 9 件。組み込みの名前に SHIORI 用の act のメソッドを含めれば 0 件の見込み）、`release` の関門がリリース CI を止める可能性。

## 7. 設計への申し送り（Research Needed）

1. hello-pasta の辞書に試作の検査を当て、誤り 0 件・注意の件数を測る（R9.5・R8 の関門がリリース CI を止めないこと）。`transfer_req_to_var` のような SHIORI 用の act のメソッドを組み込みの名前に含めて 0 件になるか。
2. `cargo publish` が path だけの dev-dependency（`pasta_lua`）を外して通るか（案 C）。
3. 名前を書かない単独の `＊` 行が受け継ぐ名前・暗黙の開始ブロック・別名で宣言したシーンの登録を、AST だけで `scope_gen.rs` と同じに再現できるか。
4. act のメソッドの一覧を `pasta_scripts`（`pasta/act.lua`・`pasta/shiori/act.lua`）とずれないように保つ方法（テストで Lua を読み込んで列挙する・手で持つ一覧と照らす）。
5. `parse_str_partial` のフォールバック（行頭からの推測）が出すエラーを、1 つのファイルの 2 つ目以降の構文エラーとして報告してよいか（推測の入った報告は「書いたとおりに読む」とぶつからないか）。
6. `pasta.toml` の `[loader]`・`[scene]` 以外のセクションの型の誤り（`pasta.dll` は読み込みエラーにする）を、どこまで誤りとして報告するか。
7. 章を足す場合、章の数 60 を決め打ちした検査は `verify-scripts-test.mjs`（78・81 行）・`talk/talk-test.mjs`（453〜454 行）・`gen-skill-refs-test.mjs`（599 行）・`link-check-test.mjs`（1193〜1224 行）の 4 つ。`manual-shell-guide` と後から入る側が数を合わせる。
8. `release` の段の表示（`[n/5]` を `[n/6]` にするか、段 3 と段 4 の間に表示だけ足すか）と、`cli_test.rs` の既存の出力の検査への影響。
9. 要件ディスカッション（2026-10-16）で全議題を推奨案で確定した結果、設計で決めること:
   - 規則を `pasta.dll` と揃える方法は案 C を推奨の方向とする。トランスパイルの 3 つの誤り（`element_gen.rs` 41・81・309 行）の判定も一致テストの対象に入れる。
   - 組み込みの名前の一覧に SHIORI 用の act のメソッド（`pasta_scripts/pasta/shiori/act.lua`）を含める。
   - `pasta.toml` の `[loader]`・`[scene]` 以外のセクションは、`pasta.dll` が読み込みに失敗する誤りを誤りにする方針の中で、どこまで当てはまるかを列挙する（6 と同じ）。
   - 日本語の報告の文言と、マニュアルの「デバッグ」の部に足す章の、各報告の意味と直し方の対応。
10. OQ-6 を「含める」に改めた結果、設計で決めること:
   - 共有の検査の部品の置き場所（新しいクレートか、`pasta_dsl`・`pasta_core` の側か）と、`pasta_check`・`pasta_lsp` からの依存の向き。新しいクレートなら `release.yml` の公開の順と `-DependsOn` も。
   - `pasta_core` を WASM にビルドできるようにする方法（`rand` の feature による切り離しか、`getrandom` の `wasm_js` か）。シーンのランダム選択の挙動は変えない。
   - 拡張の側でのファイルの集め方。`pasta_patterns` の照合は Rust の側に置き、TypeScript に規則を写さない（拡張はファイルの一覧と中身を渡すだけにする、など）。
   - 診断を更新するタイミング（保存・ファイルの増減・`pasta.toml` の保存）と、入力のたびの構文の診断との重複の消し方（Requirement 11.9）。
   - 共有の部品の WASM ビルドを CI で確かめる方法と、VSIX のテスト（`editors/vscode/src/test/`）。

## 8. 未決事項の根拠（要件ディスカッションへ）

### OQ-1 誤りと注意の線引き

- 仮置き: 誤り＝`pasta.dll` が読み込みに失敗するもの（構文・`pasta.toml`・`init.pasta`）と、`.pasta` どうし・`.lua` どうしのモジュール名の重なり。注意＝照合用の名前の重なり・組み込みの名前と同じシーン・`.pasta` と `.lua` の重なり・飛び先の無い参照。
- 根拠:
  - 読み込みの失敗はゴースト全体が起動しない（`crates/pasta_lua/src/loader/process.rs` 253〜265 行）。
  - `.pasta` どうしの重なりは `pasta.dll` が検出しない（同 77〜97 行）。キャッシュは上書きされる（`crates/pasta_lua/src/loader/cache.rs` 261〜264 行）のに、シーンの登録はマージされる。
  - 照合用の名前の重なりは、マニュアルが決まった挙動として書いている（`book/src/grammar/call-jump.md` の「シーン名の照合」：同名のグローバルシーンを複数定義した場合と同じく 1 つが選ばれる）。
  - 前方一致で候補が複数になるのは書き方そのもの（同 125〜135 行）。
  - `release` には「警告を出して続ける」と「エラーで止まる」の前例がある（`crates/pasta_check/README.md` の「同梱バルーン」）。
- 決めること: 照合用の名前の重なりを誤りに上げるか。`GLOBAL` と同じ名前のシーンを誤りにするか。

### OQ-2 読むだけでどこまで飛び先を決められるか

- 仮置き: 辞書の中のシーン（Call は 2・5 段、選択肢は 2 段とグローバル）と組み込みの名前で探し、無ければ注意。Lua の中身は読まない。動的な飛び先は件数だけを示す。構文エラーがあるときは飛び先の検査をしない。
- 根拠:
  - 1・3・4 段は Lua で決まる（`crates/pasta_lua/pasta_scripts/pasta/act.lua` 313〜322 行・332〜373 行）。
  - Lua ブロックの関数は 2・5 段の検索にも載る（`book/src/internals/registry-search.md` 106 行）。
  - `scripts/` からシーンを足せる（`crates/pasta_lua/pasta_scripts/pasta/scene.lua` 129 行 `create_scene`、`init.lua` 23 行）。
  - hello-pasta の `＞transfer_req_to_var` は SHIORI 用の act のメソッド（`crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua` 165 行）。
  - 動的な飛び先は任意の式（`crates/pasta_dsl/src/parser/grammar.pest` 169〜170 行）。条件分岐の定型が動的な飛び先なので（`call-jump.md` の「条件分岐の実現」）、1 件ずつ注意にすると多くなる。
  - 選択肢は `REG.OnChoiceSelectEx`・`＊OnChoiceSelectEx` で振り分けが変わる（`book/src/lua/shiori-events.md` の OnChoiceSelectEx）。
- 決めること:
  - 選択肢の飛び先なしは、1・3・4 段が無いぶん確かなので誤りに上げるか（ただし `OnChoiceSelectEx` の上書きは読めない）。
  - Lua ブロックの `function SCENE.名前` を読んで候補に数えるか（推測にならない範囲か）。
  - 動的な飛び先の件数を出すか。

### OQ-3 `release` の関門

- 仮置き: 上書きコピーの後・更新ファイルの前に、既定で検査する。誤りがあれば nar を作らない。注意だけなら続ける。省くオプションは持たない。
- 根拠:
  - 原則「作成ツールは問題があれば止める」（brief の Constraints）。
  - `--copy` は配布フォルダを上書きするので、配る中身は段 3 の後で決まる（`crates/pasta_check/src/release.rs` 23〜31 行）。
  - hello-pasta は `crates/pasta_sample_ghost/release.ps1` 162〜170 行で `release` を呼び、リリース CI もそれを使う。
- 決めること: 既定で走らせるか、指定したときだけか。省くオプションを持つか。段の表示。

### OQ-4 依存の重さ

- 仮置き: `pasta_dsl`・`pasta_core`（と `toml`・`glob`）だけ。LuaJIT は持ち込まない。
- 根拠:
  - `pasta_lua` は `mlua`（vendored LuaJIT）に依存する。
  - `pasta.toml` の読み方は `pasta_lua` の中にある（`crates/pasta_lua/src/loader/config/mod.rs` 112 行、非公開）。今のウェーブで `loader/config/` は触れない（`.kiro/steering/roadmap.md` 161 行）。
  - トランスパイルで初めて分かる誤りは 3 つ（`crates/pasta_lua/src/code_gen/element_gen.rs` 41・81・309 行）。
- 決めること:
  - トランスパイルの 3 つを、写して判定するか、範囲の外に置くか、`pasta_lua` に依存するか。
  - 規則の一致を一致テストで守るか（案 C）、下の部品へ移すか（案 B）。

### OQ-5 出力の形と文言

- 仮置き: 1 行 1 件（`パス:行:列: 種類: 理由`）とまとめの行だけ。JSON は付けない。終了コードは 0 と 1。
- 根拠:
  - 今の `pasta_check` は英語の文言で、終了コードは 0 と 1 だけ（`crates/pasta_check/src/main.rs` 34〜57 行、`balloon.rs`）。
  - `pasta.dll` のログも英語（`act:call - handler not found`）。
  - マニュアルとバルーンの失敗表記は日本語（`【Call失敗：…】`）。0.4.0 の主題は「書き間違えたときに原因が分かる」（`.kiro/steering/product.md`）。
- 決めること: 理由の文言を日本語にするか英語にするか。使い方の誤りと検査の誤りの終了コードを分けるか。機械向けの形を今入れるか。

### OQ-6 エディタ（LSP）との共有

- **確定（2026-10-16、個別に議論）**: 含める。検査を WASM にビルドできる共有の部品に置き、`pasta_check` と VSCode 拡張の両方から使う。拡張は保存のたびにゴースト全体を検査して診断を出す（requirements.md Requirement 11）。
- **前の推奨の根拠の訂正**: 「規則を `pasta_core` へ移すとウェーブの約束（`loader/config/` に触らない）にぶつかる」は言い過ぎだった。共有の部品の中に規則を写し、案 C と同じく一致テストで守れば、`loader/config/` には触らない。roadmap の約束にも `pasta_lsp`・`editors/vscode` を触らないという項目は無い。
- **議論のときに調べたこと**:
  - `pasta_core` は今、WASM にビルドできない。`cargo check -p pasta_core --target wasm32-unknown-unknown` は、`rand` が引き込む `getrandom 0.4.3` の `compile_error!`（`wasm_js` の feature が要る）で失敗する。直し方は、`rand` を feature で切り離すか、WASM のときだけ `getrandom` の `wasm_js` を有効にするか。
  - VSCode 拡張は LSP クライアントではなく、開いた文書の本文を WASM の `analyze(text)` に渡しているだけ（`editors/vscode/src/documentSync.ts` 133 行、`wasmBridge.ts`）。ゴースト全体の検査には、拡張の側で `pasta.toml` と辞書を集めて WASM に渡す新しい経路と、開いていないファイルの診断を更新する仕組みが要る。
  - `pasta_lsp` のネイティブの経路（`server.rs`、tower-lsp）は今回の範囲の外。
- 前の仮置き（参考）: この spec ではエディタに診断を足さず、共有も要件にしない。
- 根拠:
  - `pasta_lsp` は `pasta_dsl` だけに依存し、WASM にビルドする（`crates/pasta_lsp/Cargo.toml`）。
  - 診断は構文だけ（`crates/pasta_lsp/src/analysis/mod.rs` 57〜77 行）。
  - LSP は 1 つの文書ずつ解析し、ゴースト全体を見ていない。
- 決めること: 検査の規則を WASM にビルドできる部品に置くことを要件にするか（案 B 寄りになる）。

### OQ-7 辞書の集め方

- 仮置き: `pasta_patterns` に従う。
- 根拠:
  - 既定は `dic/**/*.pasta`（`crates/pasta_lua/src/loader/config/mod.rs` 266〜300 行）。
  - `.lua` のパターンも作られる。`..`・`profile/` の扱いもある（`book/src/reference/pasta-toml.md` 207〜230 行）。
  - 既定の場所だけを見ると、`pasta_patterns` を変えたゴーストで、検査の結果と起動の結果が食い違う。
- 決めること: `pasta_patterns` に従うことの確認。

### OQ-8 マニュアルの章

- 仮置き: 検査コマンドの章を 1 つ足す。`release` の説明は README とスキルに残す。
- 根拠:
  - マニュアルに `pasta_check` のページは無い（`book/src/SUMMARY.md` 80 行・60 章）。
  - 名前が出るのは `book/src/reference/pasta-toml.md` 377 行と `book/src/internals/index.md` 76 行だけ。
  - 章を足すと、章の数を決め打ちした検査が 4 つのファイルにある（7 節の 7）。`manual-shell-guide` と重なる（roadmap 160 行）。
- 決めること: 章を足すか。`release` の説明も章へ移すか。章の置き場所（「リファレンス」の部か）。

### OQ-9 コマンドの名前と引数

- 仮置き: `pasta_check check --target <path>`。
- 根拠:
  - `release` は `--target` でゴーストの開発フォルダを受ける（`crates/pasta_check/src/main.rs` 22〜32 行・README）。
  - brief は `check`・`validate` などを候補にしている。
- 決めること:
  - 名前（`check` は道具の名前と重なる、`validate`・`lint` など）。
  - `ghost/master` そのものも受け付けるか。
  - 引数を位置引数にするか。

---

# 設計フェーズの調査（2026-10-16）

## Summary

- **Feature**: `pasta-check-dic-validate`
- **Discovery Scope**: Extension（既存の `pasta_check`・`pasta_lsp`・VSCode 拡張への追加。軽い調査とコードの照合。`getrandom` だけ実際にビルドして確かめた）
- **使ったスキル・規則**: `kiro-spec-design`（`design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）、テンプレート `design.md`・`research.md`、steering `product.md`・`tech.md`・`structure.md`・`roadmap.md`。
- **Key Findings**:
  - `getrandom 0.4.3` は wasm32-unknown-unknown で feature `wasm_js` だけを有効にすれば通る（cfg の指定は要らない）。`pasta_core`・`pasta_dsl`・`toml`・`glob` をまとめて `cargo check --target wasm32-unknown-unknown` で確かめ、`wasm_js` 無しでは今と同じ `compile_error!` になることも確かめた。`pasta_core` は触らずに済む。
  - `SceneTable::collect_scene_candidates` は乱数を使わない。乱数源を呼ぶのは `DefaultRandomSelector::new()` だけなので、検査は `MockRandomSelector::new(vec![])` を渡せば実行中に `crypto.getRandomValues` を呼ばない（VSCode 1.85 の拡張ホストの Node 18 で `globalThis.crypto` が無くても困らない）。
  - 新しいクレートを公開すると、release CI が Trusted Publishing だけで動いているため、初回の公開を手で行い Trusted Publisher を登録する手間と、`release.yml` の 5 か所（auth と publish の段、outputs、結果の環境変数、結果の表 2 つ）・`release-setup-check.yml`・`.github/release-ci-setup.md` の変更が要る。`pasta_check` にライブラリを足せば `-DependsOn` の 1 か所で済む。

## Research Log

### WASM のビルド（`getrandom`）

- **Context**: 申し送り 10。`pasta_core` が `rand` → `getrandom 0.4.3` を引き込み wasm32 で失敗する。
- **Sources Consulted**: `getrandom-0.4.3/src/backends.rs` 170〜181 行（`wasm_js` の feature だけで分岐）、`rand-0.10.3/Cargo.toml`（`sys_rng` → `getrandom/sys_rng`）、`crates/pasta_core/src/registry/random.rs` 31 行、スクラッチの一時のクレートでの `cargo check --target wasm32-unknown-unknown --offline`。
- **Findings**: `[target.'cfg(all(target_arch = "wasm32", target_os = "unknown"))'.dependencies] getrandom = { version = "0.4", features = ["wasm_js"] }` で通る。`Cargo.lock` の `getrandom 0.4.3` はすでに `wasm-bindgen`・`js-sys` を依存に持つ（`fastrand` 経由）ので新しいパッケージは入らない見込み。
- **Implications**: 設定は検査の部品（`pasta_check`）の wasm32 の依存に置く。`pasta_core` の feature で `rand` を切り離す案より差分が小さく、ネイティブのビルドとランダム選択は変わらない。

### `pasta.dll` の規則の所在（写す対象）

- **Sources Consulted**: `pasta_lua/src/loader/discovery.rs` 15・59〜133 行、`process.rs` 28〜100・105〜271 行、`cache.rs` 238〜282 行、`loader/config/mod.rs` 55〜103・266〜319 行、`sections.rs` 327〜335 行、`transpiler.rs` 208〜236 行、`context.rs` 134 行、`code_gen/element_gen.rs` 41・81・309 行、`string_literalizer.rs` 41〜70 行、`search/context.rs` 91〜157 行、`pasta_scripts/pasta/act.lua` 332〜379 行、`global.lua` 21・27 行、`shiori/entry.lua` 17・26 行、`shiori/act.lua`、`shiori/event/choice_select.lua` 46〜77 行、`shiori/sakura_builder.lua` 31〜35 行、`pasta_dsl/src/parser/parse_scene.rs` 88〜101 行。
- **Findings**:
  - 辞書の集め方: パターンごとに `glob::glob(エスケープした基準 + パターン)`（既定の `MatchOptions`）。`..`・ルート・接頭辞を含むパターンは警告して飛ばす。基準の外・シンボリックリンク・`profile/` は除く。並べ替えと重複の除去は無い。`.lua` のパターンは `.pasta` の末尾を置き換えて作り、`.pasta` で終わらないパターンは `.lua` の検索だけ飛ばす。パターンとして読めないのは致命的な誤り。
  - `init.pasta`・`init.lua` は読み込みの誤り。`.pasta` と `.lua` の重なりは `.lua` を捨てて警告、`.pasta` どうしは見ない。
  - `pasta.toml` は**無いと読み込みに失敗する**（`ConfigNotFound`。マニュアル `pasta-toml.md` 17 行も「ファイル自体は必須」）。厳しく読むのは TOML の構文・`[loader]` の型（4 キー、知らないキーは無視）・`[scene]` と `[scene.alias]` の型と別名の規則だけ。ほかのセクションは型が違っても黙って既定値になる。→ 申し送り 6 は「`[loader]`・`[scene]` だけ」で閉じる。
  - シーンの登録: 別名は宣言の名前に 1 段だけ・完全一致で当てる。通し番号は照合用の名前ごと。単独の `＊` は parser が前の名前を入れる（ファイルの先頭なら構文エラー）。名前の無い開始のブロックはローカルとして登録しない。→ 申し送り 3 は AST だけで再現できる。
  - 実行時の検索: ローカルは照合用の名前に揃えたキーで `:親の登録名:キー` を前方一致（別名なし）、グローバルは別名を置き換えてから照合用の名前で前方一致。1・3・4 段は書いた名前のまま完全一致。
  - トランスパイルの誤りは要件の 3 つのほかに `StringLiteralError`（`\` か `"` を含み `]]`〜`]==========]` の 11 通りを全部含む文字列）がある。現実には書かれない。
  - 組み込みの飛び先: `GLOBAL` は `yield`・`チェイントーク`・`close_ghost`・`ゴースト終了` の 4 つ。act は `ACT_IMPL` の関数 27 個、SHIORI act は `SHIORI_ACT_IMPL` の関数 5 個（`transfer_req_to_var` を含む）。一覧を列挙する既存のテストは無い。
  - 選択肢の `On`・`script:` の除外は `choice_select.lua` ではなく `sakura_builder.lua` の `choice_to_script`（範囲の引数を付けない。SSP がイベントやスクリプトとして扱う）。
- **Implications**: 規則は `pasta_check` のライブラリに写し、一致テストで照らす（案 C）。`discover_files` は `pub(crate)` なので、一致テストのために `loader/mod.rs` で `#[doc(hidden)]` の公開を 1 行足す（`loader/config/` ではない）。

### hello-pasta の見積もり（申し送り 1）

- **Sources Consulted**: `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/*.pasta`・`pasta.toml`（`pasta_patterns = ["dic/*.pasta"]`、`[scene.alias]` なし）。
- **Findings**: 静的な Call は `transfer_req_to_var`（7 か所・SHIORI act のメソッド）・`ゴースト終了`（2・GLOBAL）・`チェイントーク`（1・GLOBAL）・`寄り道`（2・ローカル `・寄り道` とグローバル `＊寄り道公園` ほか）。選択肢は `おでかけの話`・`おやつの話`（どちらもグローバルシーンがある）。動的な Call は `＞＠次の話（）` の 1 つ。同じ書いた名前のシーン（`＊会話` 11 個など）は報告しない。
- **Implications**: 誤り 0・注意 0・動的な Call 1 の見込み（試作では確かめていない。実装の最初のテストで確かめる）。`release` の関門がリリース CI を止める見込みは無い。

### VSCode 拡張と CI

- **Sources Consulted**: `crates/pasta_lsp/src/transport.rs` 111〜151 行、`editors/vscode/src/{extension,wasmBridge,documentSync,diagnosticsManager}.ts`、`editors/vscode/package.json`（`activationEvents: onLanguage:pasta`・`engines ^1.85.0`・`test:unit`）、`scripts/build-wasm.ps1`、`.github/workflows/build.yml` 94〜126 行、`release.yml` 225〜335・555〜570 行、`.github/release-ci-setup.md` 161・235 行、`release-setup-check.yml` 119 行。
- **Findings**: 拡張は `wasm-pack --target nodejs` の `pasta_lsp` を `require` する。構文の診断は集合 `pasta` で、文書を閉じると消す。保存の購読は無い。テストは esbuild と node の手書きのテストで、`npm test` を走らせるワークフローは無い。`build.yml` の `wasm` の job は `pasta_lsp` を wasm32 でビルドしている。`pasta_lsp` は release.yml で公開されない。
- **Implications**: WASM の入口は `pasta_lsp` に足し（WASM の成果物を増やさない）、拡張は 2 回呼ぶ（選び出し → 中身を読む → 検査）。重複は「開いている文書の `syntax` は出さない」で消す。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| `pasta_check` にライブラリ（選んだ案） | 核を `pasta_check::validate` に置き、バイナリの依存を feature `cli` に分ける。`pasta_lsp` は `default-features = false` | release.yml は `-DependsOn` だけ・新しい Trusted Publisher 不要・検査の規則が検査の道具のクレートにある | エディタの部品が CLI の名前のクレートに依存する見た目・workspace の依存の行に `default-features = false` が要る | 選んだ（設計ディスカッションで確定） |
| 新しいクレート `pasta_validate` | 核だけのクレートを足す | 層と名前が素直 | 初回の手での公開・Trusted Publisher の登録・release.yml の 5 か所と周辺の文書・roadmap の約束を越える | 代案 |
| `pasta_dsl` の feature | 核を `pasta_dsl::validate` に置く | 公開の順は今のまま（`pasta_dsl` は `pasta_core` の後） | 構文の層が `pasta_core`・`toml`・`glob` に依存し責務がぼやける | 代案 |
| `pasta_lua` に依存（案 D） | `pasta.dll` の規則をそのまま使う | 規則が完全に同じ | LuaJIT をビルド・WASM にできない | 要件（9.1・11.1）で除外 |

## Design Decisions

### Decision: 核はファイルシステムに触れない 2 つの関数

- **Context**: 11.8（TypeScript に規則を写さない）・11.2（CLI とエディタで同じ結果）・WASM にはファイルシステムが無い。
- **Alternatives Considered**:
  1. 核が `GhostFs` の trait でファイルを読む（JS のコールバックが要る）。
  2. 殻がすべてのファイルの中身を渡す（パターンに一致しない UTF-8 でないファイルを誤って報告しうる・`profile/` が重い）。
  3. 選び出しと検査の 2 回に分ける。
- **Selected Approach**: 3。核は状態を持たず、`select_dictionary_files(input)` と `validate_ghost(input, files)`。
- **Rationale**: 殻は「一覧を渡す・言われたファイルを読む」だけになり、照合の規則が Rust にしか無い。
- **Trade-offs**: 設定の解析を 2 回する（安い）。

### Decision: 一般化（後から足す項目）

- **Context**: 後から入る spec が項目を足す（Adjacent expectations）。
- **Selected Approach**: `FindingKind` の列挙と、`severity()` の固定の対応と、検査の関数の並びを 1 か所に置く。trait や登録の仕組みは作らない（実装が 1 つしか無い抽象を避ける）。
- **Rationale**: 項目を足すのは列挙子 1 つと関数 1 つ。分類・形・終了コードは Report が決めるので既存の項目は変わらない。

### Decision: 構文エラーは 1 ファイル 1 件・推測しない

- **Context**: 要件のメモ（設計で決める）と申し送り 5。
- **Selected Approach**: `parse_str` の最初の 1 件だけ。
- **Rationale**: `pasta.dll` と同じ位置（3.3）で、推測（`parse_str_partial` の行頭からの推測・列なし）を含まない。開いている文書では入力のたびの診断が複数件を出す。
- **Follow-up**: 設計ディスカッションで確定（自明な修正）。

### Decision: `pasta.toml` が無いときも誤りにする

- **Context**: Requirement 2.2 と `pasta.dll` の挙動（`ConfigNotFound`）の食い違い。
- **Selected Approach**: 既定値で集めて検査を続け、`config-missing` を誤りにする。
- **Follow-up**: 要件 2.10 を足し 9.4 を書き足して確定（自明な修正）。

### Decision: Build vs Adopt

- 構文・照合用の名前・別名・前方一致の検索は既存の `pasta_dsl`・`pasta_core` を使う（自作しない）。パターンの照合は `glob::Pattern`（`pasta.dll` と同じクレート）。TOML は `toml`（同じ版）。新しい外部の依存は無い（`getrandom` は lock にある版の feature を有効にするだけ）。

### Decision: Simplification

- `.lua` の中身は読まない（パスだけ）。拡張の起動直後の全体検査・解析の結果のキャッシュ・JSON の出力・CI の `npm test` の job は作らない（要件に無い・必要になってから足せる）。

### Decision: パターンの大文字・小文字は Windows の `glob::glob` を写す（設計ディスカッション 2026-10-16）

- **Context**: `pasta.dll` の `discover_files` は `glob::glob` を使う。`glob::glob` はワイルドカードの無い成分をパターンの文字列のまま繋いでファイルシステムで引くので、Windows では `Dic/` のフォルダも `dic/**/*.pasta` に一致し、返すパスの綴りは `dic` になる。ワイルドカードのある成分は既定の `MatchOptions`（大文字・小文字を区別）で比べる。
- **Alternatives Considered**:
  1. 核ですべて区別する（下書き）— `Dic/` で辞書を 1 つも検査せずに成功し、`release` の関門を素通りする。
  2. CLI だけファイルシステムの `glob` を使う — エディタと食い違い 11.2 を守れない。
  3. 核で Windows の振る舞いを写す。
- **Selected Approach**: 3。ワイルドカードの無い成分は区別せず、ある成分は区別する。モジュール名は `pasta.dll` が見るパス（パターンの綴り）から作る。
- **Rationale**: `pasta.dll` は Windows でしか動かないので、Windows の振る舞いが正典。CLI とエディタは同じ核なので結果は揃ったまま。
- **Trade-offs**: 成分ごとの照合を自前で書く（数十行）。区別しない比較は Unicode の小文字化で近似する。

### Decision: 設計ディスカッション（2026-10-16）で推奨案のまま確定したもの

- 共有の部品は `pasta_check` のライブラリ（feature `cli`）。新しいクレートは作らない。
- `release` の段は `[1/6]`〜`[6/6]`、検査は `[4/6] Validating dictionaries...`。
- 文字列リテラルの変換の失敗（4 つ目のトランスパイルの誤り）は範囲に入れない。
- VSIX のテストは `test:unit` に足し、CI の job は足さない。
- 拡張の起動直後には全体の検査をしない・シーン名の重なりは宣言ごとに 1 件・組み込みの名前との衝突は別名を置き換えた後の名前で比べる。

## Risks & Mitigations

- 規則のずれ — 一致テスト（モジュール名・選び出し・`pasta.toml`・トランスパイル・組み込みの名前・検索）。`pasta_lua` を変える spec は `cargo test -p pasta_check` を通す。
- Windows のパターンの大文字・小文字 — 核で `glob::glob` の Windows の振る舞いを写す（下の Decision）。一致テスト (b) を Windows の CI で走らせて守る。
- 拡張の保存のたびの全体検査の速さ — 300ms で束ねる。遅ければ解析の結果を中身で覚える改善を後から。
- `release` の関門がリリース CI を止める — hello-pasta の誤り 0 を CLI のテストで守る。
- スキル（`.claude/skills/pasta-check/SKILL.md`）の編集を分類器が止める — 実装のときに開発者の許可を取る。
