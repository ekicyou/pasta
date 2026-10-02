# 吸収台帳

吸収元（`OPTIMIZATION.md`、スキル `internal-modules.md`、クレート README 4 本）の全節について、行き先と実装照合を記録し、内容の欠落がゼロであることを節単位で示す。本台帳は spec 成果物であり、完了後は spec と共に `completed/` へ移る（リポジトリ現行文書としての写しにはならない）。

## 記入規則

- 行は吸収元の見出し 1 つ（コードブロック外の H1〜H4。H1 行はファイル冒頭の見出し前後の記述を表す）に対応する。見出しの下にある本文・表・コード例はその行に含まれる。
- 列:
  - **吸収元（ファイル#節）**: リポジトリルートからのパスと、元の見出し（`#` の数で階層を示す）。
  - **担当**: 処置を埋めるタスク番号（`tasks.md`）。「A（→B）」は A が内部設計章への収録を処置し、B が README 側の処置（縮約・削除・置換・存置）を確定することを示す。「A／B」は 2 章で分担する行で、章間の線引きは 4.11 が見直す。
  - **処置**: 次のいずれか。収録先 `章#節`／既存収録済み `章#節`／README 存置／除外。
  - **理由**: 「README 存置」「除外」の場合に必須（10.6, 7.7）。
  - **実装照合**: 照合したソース位置。照合が要らない行は「照合不要」と理由。
  - **訂正**: 旧記述と現行実装が食い違った場合の要旨（実装を正とする）。
- 並行タスク（`(P)`）は自分の担当行だけを編集する。`OPTIMIZATION.md` の既知の食い違い候補は `design.md`（AbsorptionLedger 節）の表にあり、担当タスクが実装と再照合して「訂正」列に記録する。
- 完了条件: 全行の「処置」が埋まり、「README 存置」「除外」には理由がある（6.1 で確認する）。

## 吸収元の節

### `OPTIMIZATION.md`

| 吸収元（ファイル#節） | 担当 | 処置 | 理由 | 実装照合 | 訂正 |
|---|---|---|---|---|---|
| `OPTIMIZATION.md` # Transpiler Optimization Reference | 4.1 | 収録先 `internals/transpiler.md#生成時最適化` |  | 照合不要（文書の導入文と最終更新日・ステータスの記載のみ。最適化の本体は各節の行で照合） | 「最終更新」「ステータス: Phase 0完了」は文書の管理情報であり引き継がない |
| `OPTIMIZATION.md` ## 1. 最適化の概要 | 4.1 | 収録先 `internals/transpiler.md#生成時最適化` |  | `crates/pasta_lua/src/code_gen/scope_gen.rs` `generate_local_scene_items`、`crates/pasta_lua/src/code_gen/element_gen.rs` `generate_call_scene`・`generate_continue_action`、`crates/pasta_lua/src/string_literalizer.rs` | 「アクター最適化（連続発言のアクター切替最小化）」は現行実装に存在しない。`last_actor` は継続行の話者引継ぎ（構文上の要請）として収録。「ロングブラケット記法」は `"…"` とロングブラケットの選択として収録 |
| `OPTIMIZATION.md` ## 2. 末尾呼び出し最適化 (Tail Call Optimization) | 4.1 | 収録先 `internals/transpiler.md#末尾呼び出し` |  | `crates/pasta_lua/src/code_gen/scope_gen.rs` `generate_local_scene_items`（`is_callable_item`・`is_tail_call`）、`crates/pasta_lua/src/code_gen/element_gen.rs` `generate_call_scene` | 参照実装 `code_generator.rs` は存在しない（現行は `crates/pasta_lua/src/code_gen/`） |
| `OPTIMIZATION.md` ### 2.1 概要 | 4.1 | 収録先 `internals/transpiler.md#末尾呼び出し` |  | `crates/pasta_lua/pasta_scripts/pasta/act.lua` `ACT_IMPL.call`（`return handler(self, ...)` の末尾位置呼び出し） | 効果は「無限再帰を可能にする」ではなく、生成側の `return` と `act:call` 内の末尾位置呼び出しが連鎖してスタックが深くならない、として実装に即して記述 |
| `OPTIMIZATION.md` ### 2.2 適用条件 | 4.1 | 収録先 `internals/transpiler.md#末尾呼び出し` |  | `crates/pasta_lua/src/code_gen/scope_gen.rs` `generate_local_scene_items`（`last_is_callable && index == last_index`） | なし（最後の項目が Call のときだけ `return` を前置する点は一致。暗黙の開始ブロックと名前付きローカルシーンの両方に適用されることを補記） |
| `OPTIMIZATION.md` ### 2.3 コード例 | 4.1 | 収録先 `internals/transpiler.md#生成される-lua-コードの形` |  | `crates/pasta_lua/src/code_gen/scope_gen.rs` `generate_local_scene`、`crates/pasta_lua/tests/fixtures/sample.expected.lua` | なし（`function SCENE.__start__(act, ...)`・`act:init_scene(SCENE)`・`return act:call(...)` の形は一致。収録例は現行の空行配置で書き直した） |
| `OPTIMIZATION.md` ### 2.4 TCO非適用ケース | 4.1 | 収録先 `internals/transpiler.md#末尾呼び出し` |  | `crates/pasta_lua/src/code_gen/scope_gen.rs` `generate_local_scene_items` | なし |
| `OPTIMIZATION.md` ## 3. アクター最適化 | 4.1 | 収録先 `internals/transpiler.md#継続行の話者引継ぎ` |  | `crates/pasta_lua/src/code_gen/scope_gen.rs` `generate_local_scene_items`（`last_actor`）、`crates/pasta_lua/src/code_gen/element_gen.rs` `generate_action_line`・`generate_continue_action` | 最適化ではない。継続行の話者を決めるための構文上の処理として収録 |
| `OPTIMIZATION.md` ### 3.1 概要 | 4.1 | 収録先 `internals/transpiler.md#継続行の話者引継ぎ` |  | `crates/pasta_lua/src/code_gen/element_gen.rs` `generate_action` | 「同じアクターの連続発言でコンテキストを保持して効率化」はしていない。アクションごとに常に `act.アクター:…` の文を生成する。連続発言のまとめは実行時のトーク組立（`talk-output.md`）が担う |
| `OPTIMIZATION.md` ### 3.2 実装 | 4.1 | 収録先 `internals/transpiler.md#継続行の話者引継ぎ` |  | `crates/pasta_lua/src/code_gen/scope_gen.rs` `generate_local_scene_items`、`crates/pasta_lua/src/code_gen/element_gen.rs` `generate_continue_action` | `last_actor` は「必要な場合のみアクターを切り替える」ためではなく、継続行のアクターを決めるための状態。ローカルシーンごとに空から始まり、先行アクション行が無ければ `TranspileError::InvalidContinuation` |
| `OPTIMIZATION.md` ## 4. 文字列リテラル最適化 | 4.1 | 収録先 `internals/transpiler.md#文字列リテラルの表記選択` |  | `crates/pasta_lua/src/string_literalizer.rs` `StringLiteralizer::literalize_with_span` | なし（節全体の位置づけ。個別の訂正は 4.1〜4.3 の行） |
| `OPTIMIZATION.md` ### 4.1 概要 | 4.1 | 収録先 `internals/transpiler.md#文字列リテラルの表記選択` |  | `crates/pasta_lua/src/string_literalizer.rs` `literalize_with_span`・`contains_danger_pattern` | `[=[ ... ]=]` 固定ではない。`=` の数は 0 から試して内容に閉じ括弧の前半が現れない最小の数（最大 10、超過で `TranspileError::StringLiteralError`） |
| `OPTIMIZATION.md` ### 4.2 適用条件 | 4.1 | 収録先 `internals/transpiler.md#文字列リテラルの表記選択` |  | `crates/pasta_lua/src/string_literalizer.rs` `needs_long_string` | ロングブラケットを使うのは `\` か `"` を含む場合だけ。改行・Unicode 文字を含むことは条件ではない（非 ASCII 文字だけなら `"…"`） |
| `OPTIMIZATION.md` ### 4.3 例 | 4.1 | 収録先 `internals/transpiler.md#文字列リテラルの表記選択` |  | `crates/pasta_lua/src/code_gen/scope_gen.rs` `generate_actor`、`crates/pasta_lua/tests/fixtures/sample.expected.lua` | なし（`ACTOR:create_word("通常"):entry([=[\s[0]]=])` は現行の生成形と一致） |
| `OPTIMIZATION.md` ## 5. 今後の最適化候補 | 5.4 |  |  |  |  |
| `OPTIMIZATION.md` ## 6. ビルドプロファイル最適化 | 4.5 | 収録先 `internals/shiori.md#dll-のビルド構成` |  | ルートの `Cargo.toml` `[profile.release]`（`opt-level = "z"`・`lto = true`・`codegen-units = 1`・`panic = "abort"`・`strip = true`）、`.cargo/config.toml`、`crates/pasta_shiori/build.rs`、`crates/pasta_shiori/src/windows.rs`（`catch_unwind`） | なし（5 キーの値は一致）。補記: プロファイルは pasta.dll 専用ではなくワークスペース全体に効く。`panic = "abort"` のためリリースでは FFI 入口の `catch_unwind` が効かない（dev・test だけの保険）ことを併記。静的 CRT（`.cargo/config.toml`）とバージョン情報の埋め込み（`build.rs`）を DLL のビルド構成として併記 |
| `OPTIMIZATION.md` ## 7. 関連ドキュメント | 4.1 | 収録先 `internals/transpiler.md#ソースの所在` |  | `book/src/grammar/index.md`・`crates/pasta_lua/tests/fixtures/tail_call_optimization.pasta` は実在。`code_generator.rs` は不在 | `code_generator.rs`（末尾の「参照実装 code_generator.rs#L320-L460」を含む）は存在しない。現行の所在 `crates/pasta_lua/src/code_gen/` と TCO の用例を「ソースの所在」に記載。文法章は本文から `grammar/index.md` へリンク |

### `.claude/skills/pasta-lua-coding/references/internal-modules.md`

| 吸収元（ファイル#節） | 担当 | 処置 | 理由 | 実装照合 | 訂正 |
|---|---|---|---|---|---|
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` # Internal Modules リファレンス | 4.9／4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ## STORE パターン | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### フィールド一覧 | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### reset() | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### 循環参照回避の原則 | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ## ACT オブジェクト | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### init_scene(scene) | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### トーク系メソッド | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### talk(actor, text) | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### raw_script(text) | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### SHIORI固有メソッド（ShioriActのみ） | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### set_property(name, value) | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### get_property(name_or_names [, timeout [, timeout_message]]) | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### 表示制御メソッド | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### スポット操作 | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### set_spot(name, number) | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### clear_spot() | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### 検索・呼び出し | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### word(name) | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### find_handler(mode, key) | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### find_act_handler(mode, key) | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### expr_fn(key, ...) | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### find_scene(key) | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### call(global_scene_name, key, attrs, ...) | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### yield() | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### choice(target, display) | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### choice_timeout(seconds) | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### PROXYパターン | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### PROXY.find_actor_handler(mode, key) | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### PROXY.find_handler(mode, key) | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### PROXY.expr_fn(key, ...) | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ## SCENE モジュール | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### create_scene(base_name, local_name?, scene_func?) | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### search(name, global_scene_name?, attrs?) | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### co_exec(act, name, global_scene_name?, attrs?) | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### DSL→Luaブリッジ | 4.9／4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ## WORD モジュール | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### ファクトリ関数 | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### ビルダーパターン | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` #### entry(...) | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### 大量投入の使用例 | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ## GLOBAL モジュール | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ## SAVE モジュール | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### キー命名規約 | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### ACT経由のアクセス（推奨） | 4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### 直接require | 4.9／4.10 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ## finalize_scene | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### 目的 | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### 呼び出しタイミング | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### 処理フロー | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### シーン収集データ構造 | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### 単語収集データ構造 | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### 上級者向け情報 | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ## ユーティリティモジュール | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### pasta.buf | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ### pasta.lua_version | 4.9 |  |  |  |  |
| `.claude/skills/pasta-lua-coding/references/internal-modules.md` ## 関連リファレンス | 4.9／4.10 |  |  |  |  |

### `crates/pasta_dsl/README.md`

| 吸収元（ファイル#節） | 担当 | 処置 | 理由 | 実装照合 | 訂正 |
|---|---|---|---|---|---|
| `crates/pasta_dsl/README.md` # pasta_dsl | 5.3 |  |  |  |  |
| `crates/pasta_dsl/README.md` ## Purpose | 5.3 |  |  |  |  |
| `crates/pasta_dsl/README.md` ## Features | 5.3 |  |  |  |  |
| `crates/pasta_dsl/README.md` ## Usage | 5.3 |  |  |  |  |
| `crates/pasta_dsl/README.md` ## Public API | 5.3 |  |  |  |  |
| `crates/pasta_dsl/README.md` ### Parse Functions | 5.3 |  |  |  |  |
| `crates/pasta_dsl/README.md` ### AST Types | 5.3 |  |  |  |  |
| `crates/pasta_dsl/README.md` ### Error Types | 5.3 |  |  |  |  |
| `crates/pasta_dsl/README.md` ## Dependencies | 5.3 |  |  |  |  |
| `crates/pasta_dsl/README.md` ## Architecture | 4.1（→5.3） | 収録先 `internals/transpiler.md#パーサと-ast`（ソース構成。テストファイルの一覧は `crates/pasta_dsl/tests/` の所在として「ソースの所在」に示す） |  | `crates/pasta_dsl/src/`（`lib.rs`・`error.rs`・`partial.rs`・`parser/mod.rs`・`parser/parse_scene.rs`・`parser/parse_action.rs`・`parser/parse_elements.rs`・`parser/ast/`・`parser/grammar.pest`）と `crates/pasta_dsl/tests/` の 15 ファイルを照合 | なし（ツリーは現行と一致） |
| `crates/pasta_dsl/README.md` ## License | 5.3 |  |  |  |  |

### `crates/pasta_core/README.md`

| 吸収元（ファイル#節） | 担当 | 処置 | 理由 | 実装照合 | 訂正 |
|---|---|---|---|---|---|
| `crates/pasta_core/README.md` # pasta_core | 5.3 |  |  |  |  |
| `crates/pasta_core/README.md` ## 概要 | 5.3 |  |  |  |  |
| `crates/pasta_core/README.md` ## アーキテクチャ | 4.2（→5.3） | 収録先 `internals/registry-search.md#pasta_core-のレジストリと検索表` |  | `crates/pasta_core/src/lib.rs`・`crates/pasta_core/src/registry/mod.rs`（公開する型）、`crates/pasta_core/src/registry/scene_registry.rs`・`scene_table.rs`・`word_registry.rs`・`word_table.rs`・`random.rs`、`crates/pasta_core/src/error.rs`、`crates/pasta_core/Cargo.toml`（依存） | 「SceneRegistry シーン登録（Pass 1）」は誤り。トランスパイル時の登録（`register_global`・`register_local`）と実行時の辞書確定での再構築（`register_global_raw`）の 2 か所で使われ、検索の権威は後者。`WordTable` も `SceneTable` と同じく RadixMap の前方一致で検索する。ツリーに無い乱数の抽象（`RandomSelector`・`DefaultRandomSelector`・`MockRandomSelector`）を構成要素に加えた |
| `crates/pasta_core/README.md` ## ディレクトリ構成 | 4.2（→5.3） | 収録先 `internals/registry-search.md#pasta_core-のレジストリと検索表`（ファイルごとの役割）・`internals/registry-search.md#ソースの所在`（テストの所在） |  | `crates/pasta_core/src/`（`lib.rs`・`error.rs`・`registry/` の 9 ファイル）と `crates/pasta_core/tests/word_table_test.rs` を照合 | ファイル構成は現行と一致。注記の訂正: `registry/mod.rs` は API 本体ではなく再エクスポートだけを持つ。`random.rs` はインターフェースだけでなく既定の実装とモックの実装を持つ |
| `crates/pasta_core/README.md` ## 公開API | 5.3 |  |  |  |  |
| `crates/pasta_core/README.md` ### Registry | 5.3 |  |  |  |  |
| `crates/pasta_core/README.md` ### Random | 5.3 |  |  |  |  |
| `crates/pasta_core/README.md` ## 使用例 | 5.3 |  |  |  |  |
| `crates/pasta_core/README.md` ### シーンテーブルの構築と検索 | 5.3 |  |  |  |  |
| `crates/pasta_core/README.md` ### 単語テーブルの構築と検索 | 5.3 |  |  |  |  |
| `crates/pasta_core/README.md` ## 依存関係 | 5.3 |  |  |  |  |
| `crates/pasta_core/README.md` ## 関連クレート | 5.3 |  |  |  |  |
| `crates/pasta_core/README.md` ## ライセンス | 5.3 |  |  |  |  |

### `crates/pasta_shiori/README.md`

| 吸収元（ファイル#節） | 担当 | 処置 | 理由 | 実装照合 | 訂正 |
|---|---|---|---|---|---|
| `crates/pasta_shiori/README.md` # pasta_shiori | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ## 概要 | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ## アーキテクチャ | 4.5（→5.3） | 収録先 `internals/shiori.md#構成要素` |  | `crates/pasta_shiori/src/windows.rs`・`crates/pasta_shiori/src/actor/`（`lifecycle.rs`・`mailbox.rs`・`thread.rs`・`marshaling.rs`・`teardown.rs`）・`crates/pasta_shiori/src/shiori.rs`・`crates/pasta_shiori/src/lua_request.rs` | ツリーにアクターランタイム（`actor/`）と初期化入口 `loadu` が無い。現行は FFI 入口が `actor::lifecycle` を通じてメッセージを送り、VM を持つアクタースレッド上の `PastaShiori` が `PastaLoader` と `lua_request` を使う |
| `crates/pasta_shiori/README.md` ## ディレクトリ構成 | 4.5（→5.3） | 収録先 `internals/shiori.md#rust-側pasta_shiori`（ファイルごとの役割）・`internals/shiori.md#ソースの所在`（テストの所在） |  | `crates/pasta_shiori/src/`・`crates/pasta_shiori/build.rs`・`crates/pasta_shiori/tests/` を照合 | ツリーに `src/actor/`（6 ファイル）、`build.rs`、`util/parsers/` の pest 文法（`req_parser.pest`）、`windows_tests.rs`、SHIORI 外部仕様の写し `util/shiori.md` が無い。`lua_request.rs` は「Lua リクエスト処理」ではなく、リクエスト文字列を Lua の表にする解析である |
| `crates/pasta_shiori/README.md` ## SHIORI プロトコル | 4.5（→5.3） | 収録先 `internals/shiori.md#処理とデータの流れ` |  | 照合不要（見出しだけの節。内容は下位の 5 行で照合） | なし |
| `crates/pasta_shiori/README.md` ### プロトコルフロー | 4.5（→5.3） | 収録先 `internals/shiori.md#load-とアクターの起動`・`internals/shiori.md#request-と-getnotify-の振り分け`・`internals/shiori.md#unloaddllmain-と-teardown` |  | `crates/pasta_shiori/src/windows.rs` `load_entry`・`request`・`unload`・`DllMain`、`crates/pasta_shiori/src/actor/lifecycle.rs` `spawn_actor`・`marshal_request`・`teardown_actor` | DLL 名は `shiori32.dll` ではなく `pasta.dll`。FFI の `load` は `(hinst, load_dir)` ではなく `(HGLOBAL, 長さ)` を受け取り、UTF-8 の `loadu` が `load` より優先される。`SHIORI.load` に渡る `hinst` は常に 0 |
| `crates/pasta_shiori/README.md` ### サポートイベント | 4.5（→5.3） | 収録先 `internals/shiori.md#処理とデータの流れ`（FFI の関数）。SHIORI イベントの一覧は既存収録済み `lua/shiori-events.md#主要イベント` |  | `crates/pasta_shiori/src/windows.rs`、`crates/pasta_shiori/src/shiori.rs` `PastaShiori::load`・`Drop` | 表はイベントではなく FFI の関数の一覧である。`pasta.toml` の読み込みとランタイムの起動はアクタースレッド上の `PastaShiori::load`（`PastaLoader::load_with_config`）が行う。`unload` はアクターの終了処理で、VM の破棄時に `SHIORI.unload` と永続化データの保存が走る。`loadu` が欠けている |
| `crates/pasta_shiori/README.md` ### SHIORI/3.0 リクエスト形式 | 4.5（→5.3） | 収録先 `internals/shiori.md#アクタースレッドのメッセージループ`（解析と Lua の表）。表のフィールドは既存収録済み `lua/shiori-events.md#actreq` |  | `crates/pasta_shiori/src/util/parsers/req_parser.pest`、`crates/pasta_shiori/src/lua_request.rs` `parse_request` | なし（例は文法に適合する）。補記: SHIORI/2.x のリクエストも解析し、`X-Pasta-Time` で日時を差し替えられる |
| `crates/pasta_shiori/README.md` ### レスポンス形式 | 4.5（→5.3） | 収録先 `internals/shiori.md#応答文字列の出どころ`。`RES` による応答の組み立ては既存収録済み `lua/shiori-events.md#res` |  | `crates/pasta_lua/pasta_scripts/pasta/shiori/res.lua` `RES.build`、`crates/pasta_shiori/src/actor/marshaling.rs` `default_204`、`crates/pasta_shiori/src/shiori.rs` `default_204_response`、`crates/pasta_shiori/src/error.rs` | 例に `Sender`・`SecurityLevel` が無い。`RES.build` は `Charset`・`Sender`・`SecurityLevel` を常に出す。Rust が作る 204・400・500 はヘッダの組が異なる |
| `crates/pasta_shiori/README.md` ### FFI 境界の安全性 | 4.5（→5.3） | 収録先 `internals/shiori.md#不変条件と制約`・`internals/shiori.md#request-と-getnotify-の振り分け` |  | `crates/pasta_shiori/src/windows.rs`（`catch_unwind`・`ShioriString::capture`）、`crates/pasta_shiori/src/util/hglobal/mod.rs` `alloc_global`、`crates/pasta_shiori/src/lua_request.rs` `parse1`、ルートの `Cargo.toml` `[profile.release]` | `request` の panic は 500 ではなく `default_204`（204）になる。`catch_unwind` はリリース（`panic = "abort"`）では効かず、dev・test だけの保険である。UTF-8 として読めないリクエストも 204 になる。入力 HGLOBAL の解放・確保失敗の明示エラー化・反復の解析は一致 |
| `crates/pasta_shiori/README.md` ## 公開API | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ### PastaShiori | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ### Shiori トレイト | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ## 使用例 | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ### Rust からの利用（テスト用） | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ### ゴーストディレクトリ構成 | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ## 依存関係 | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ### Windows 専用 | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ### 開発用（dev-dependencies） | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ## ビルド | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ### Windows DLL | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ### ライブラリ（テスト用） | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ## 外部仕様参照 | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ## 関連クレート | 5.3 |  |  |  |  |
| `crates/pasta_shiori/README.md` ## ライセンス | 5.3 |  |  |  |  |

### `crates/pasta_lua/README.md`

| 吸収元（ファイル#節） | 担当 | 処置 | 理由 | 実装照合 | 訂正 |
|---|---|---|---|---|---|
| `crates/pasta_lua/README.md` # pasta_lua | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ## 概要 | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ## アーキテクチャ | 4.11（→5.2） |  |  |  |  |
| `crates/pasta_lua/README.md` ## ソースモジュール構成 | 4.11（→5.2） |  |  |  |  |
| `crates/pasta_lua/README.md` ## ディレクトリ構成 | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ### Lua パススルー機能 | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ## 設定ファイル（pasta.toml） | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ### 最小構成（必須の `[actor]` のみ） | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ### [actor.*] セクション | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ### [lua] セクション | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ## Lua モジュール検索パス | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ### 検索優先順位 | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ### UTF-8 契約 | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ### 起動モジュールのロード失敗 | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ## 組み込みモジュール | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ### 使用例 | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ### API リファレンス | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ## 使用方法 | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ### 基本的な使用法 | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ### カスタム設定での起動 | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ### トランスパイラー単独使用 | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ## SHIORI 統合 | 4.5（→5.2） | 収録先 `internals/shiori.md#lua-側の-shiori-エントリとイベント配送` |  | `crates/pasta_lua/pasta_scripts/pasta/shiori/entry.lua`、`crates/pasta_lua/src/runtime/factory.rs`（起動時の `require`） | 利用者が `scripts/pasta/shiori/entry.lua` を置く必要は無い。`pasta.shiori.entry` はフレームワークスクリプトとして同梱され、起動時に必ず読み込まれる（`scripts/` に同名を置けば検索順で置き換わる）。現行の `SHIORI.request` は `EVENT.fire` を `xpcall` で包んでエラーを `RES.err` の 500 にし、`SHIORI.kick` も定義する。README の例は保護の無い旧形 |
| `crates/pasta_lua/README.md` ### pasta.shiori.res モジュール | 4.5（→5.2） | 既存収録済み `lua/shiori-events.md#res`（利用者向け）。Rust が作る応答との違いは収録先 `internals/shiori.md#応答文字列の出どころ` |  | `crates/pasta_lua/pasta_scripts/pasta/shiori/res.lua` | なし（関数・ステータス・`RES.env` の既定値は一致）。補記: `REG` のハンドラは応答全体ではなく `Value` の文字列を返す（`EVENT.fire` が `RES.ok` で包む）。README の `return RES.ok("Hello!")` はハンドラの戻り値としては誤った書き方 |
| `crates/pasta_lua/README.md` ### pasta.shiori.sakura_builder モジュール | 4.6（→5.2） | 収録先 `internals/talk-output.md#さくらスクリプトの組立` |  | `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` `BUILDER.build`、`crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua` `SHIORI_ACT_IMPL.build` | 「純粋関数モジュール」ではない。`BUILDER.build` は渡された `STORE.actor_spots` と `STORE.appearance`（外見状態）をその場で書き換え、`talk`・`sakura_script` を `@pasta_sakura_script` の `talk_to_script` で後処理し、`pasta.shiori.appearance` で外見の観測と復旧を行う |
| `crates/pasta_lua/README.md` #### グループ化トークン形式（推奨） | 4.6（→5.2） | 収録先 `internals/talk-output.md#グループ化トークン` |  | `crates/pasta_lua/pasta_scripts/pasta/act.lua` `ACT_IMPL.build`（局所関数 `group_by_actor`・`merge_consecutive_talks`）、`crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` `BUILDER.build` | 例の結果は誤り。`talk` は `talk_to_script` でウェイトが挿入されるため、既定の `[talk]` では `\p[0]こんにちは！\_w[450]\s[5]\_w[1000]\e` になる（`！` は強調のウェイト 500 − 50。アクターに `surface`・`dressup` や外見の記録があれば `\p[0]` の直後に復旧タグも入る）。「推奨」の形式ではなく、`ACT_IMPL.build` が返す唯一の形式 |
| `crates/pasta_lua/README.md` #### レガシーフラット形式（後方互換） | 4.6（→5.2） | 除外（フラット形式そのもの）。節末のトークンタイプ表は収録先 `internals/talk-output.md#トークンの種類と出力` | フラット形式は現行実装に存在しない。`BUILDER.build` は最上位の `talk`・`surface`・`wait` を無視し、`tokens` を持たない `type = "actor"` は `ipairs(nil)` でエラーになる（後方互換は無い） | `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` `BUILDER.build`・`inner_token_to_string`（Lua 5.4 で README のフラット形式の例を `BUILDER.build` に渡し、エラーになることを確認） | トークンタイプ表の訂正: `talk` の出力は「エスケープ済みテキスト」ではなく `talk_to_script` の後処理（ウェイト挿入・budoux 改行）済みの文字列。`actor`(legacy)・`spot_switch`・`yield` の型は存在しない（未知の型は空文字列）。`sakura_script`・`choice`・`choice_timeout` が欠けている。`raw_script` は最上位とグループの内側の両方に現れる |
| `crates/pasta_lua/README.md` ## ファイル検出パターン | 4.4（→5.2） | 収録先 `internals/loader.md#ファイル検出`（パターンの書き方と既定値は既存収録済み `reference/pasta-toml.md#pasta_patterns`） |  | `crates/pasta_lua/src/loader/discovery.rs` `discover_files`・`is_in_profile_dir`、`crates/pasta_lua/src/loader/process.rs` `discover_all_files`、`crates/pasta_lua/src/loader/config/mod.rs` `default_pasta_patterns` | なし（既定の `dic/**/*.pasta` が `dic/` 直下を含む全階層に一致し、`profile/` 配下を除く点は一致）。補記: `profile/` 配下の除外はパターンに依らず常に行われる。カスタムパターンの例（`dic/*/*.pasta` と `extra/*.pasta`）は利用者向けの例であり、README 側の扱いは 5.2 が確定する |
| `crates/pasta_lua/README.md` ## モジュール名の生成 | 4.4（→5.2） | 収録先 `internals/loader.md#モジュール名の生成` |  | `crates/pasta_lua/src/loader/cache.rs` `source_to_module_name`・`source_to_cache_path`・`strip_component_prefix`、`crates/pasta_lua/src/loader/process.rs` `module_key` | README の表は誤り。`dic/baseware/system.pasta` のモジュール名は `dic_baseware_system` ではなく `pasta.scene.baseware.system`（先頭の `dic` をパス要素の境界でだけ除き、拡張子を除き、区切りを `.`・`-` を `_` にして `pasta.scene.` を前置する）。キャッシュ先 `<キャッシュ>/pasta/scene/baseware/system.lua` との対応も収録 |
| `crates/pasta_lua/README.md` ## 関連クレート | 5.2 |  |  |  |  |
| `crates/pasta_lua/README.md` ## ライセンス | 5.2 |  |  |  |  |

## 付録 A: コメント修正

3.8 に基づき、現行実装と食い違うソースコメントをコメントのみ修正した記録。

| ファイル | 位置 | 修正の要旨 | 担当 |
|---|---|---|---|
| `crates/pasta_core/src/lib.rs` | クレート doc の `registry` の説明行 | 「Pass 1 + Runtime tables」を、トランスパイル時に登録し実行時の確定で作り直すレジストリと実行時テーブル、に修正 | 4.1 |
| `crates/pasta_core/src/registry/mod.rs` | モジュール doc（冒頭・Design の箇条） | 存在しない「transpiler と transpiler2 の共有」「(Pass 1)」を削除し、トランスパイル時の単一走査と実行時の `finalize_scene` の 2 か所で使われることを記述 | 4.1 |
| `crates/pasta_core/src/registry/scene_registry.rs` | モジュール doc | 「two-pass transpiler strategy（Pass 1 / Pass 2: `mod pasta {}` 生成）」を、単一走査での登録と実行時の再構築に修正 | 4.1 |
| `crates/pasta_core/src/registry/word_registry.rs` | モジュール doc・`WordDefRegistry` の doc | 「LabelDef」「during Pass 1」を、トランスパイル時と実行時の確定で収集する、に修正 | 4.1 |
| `crates/pasta_lua/src/code_gen/scope_gen.rs` | `generate_actor`・`generate_global_scene`・`generate_local_scene` の doc の生成例 | 旧生成形（`ACTOR.通常 = { … }`・`create_scene("モジュール名_N")`・`(ctx, ...)`・`PASTA.create_session`・`__シーン名_N__`）を現行の生成形（`ACTOR:create_word(…):entry(…)`・基本名・`(act, ...)`・`act:init_scene(SCENE)`・`シーン名_N`）に修正 | 4.1 |
| `crates/pasta_lua/src/code_gen/mod.rs` | `record_span` の doc（Coverage note） | 「`generate_action` だけが配線済みで残りは後続 spec の範囲外」を、現行の記録対象（スコープ見出し・アクション・変数代入・Call・選択肢・キューコマンド・単語定義、Lua ブロックは行ごと）と `LineShift` による写像に修正 | 4.1 |
| `crates/pasta_lua/src/config.rs` | `TranspilerConfig::comment_mode` の doc | 「ソース行参照を含める」を、現行のコード生成は参照せず出力は変わらない、に修正 | 4.1 |
| `crates/pasta_lua/src/lib.rs` | クレート doc（概要・Example） | 存在しない `pasta_rune` との比較と「Lua 5.3+」を LuaJIT 2.1（`luajit52`）に、旧シグネチャ `transpile(&actors, &scenes, …)` を `transpile(&pasta_file, …)` に修正 | 4.1 |
| `crates/pasta_lua/src/code_gen/element_gen.rs` | `generate_local_word` の doc | 取り込み後に修正（並行 spec dynamic-word-reference が編集中のため未修正）。「Called inside a local scene function, after init_scene」は誤りで、実際はグローバルシーンの `do` ブロック内・関数定義より前に出力される | 4.1 |
| `crates/pasta_dsl/src/parser/mod.rs` | モジュール doc（冒頭・Grammar Authority） | 取り込み後に修正（並行 spec 編集中のため未修正）。`file = ( file_scope \| global_scene_scope )*` に `actor_scope` が欠けている。`grammar.pest` は「手で編集してはならない」とあるが、文法は後続 spec で拡張されている（例: 選択肢行・プロパティ） | 4.1 |
| `crates/pasta_dsl/src/parser/ast/mod.rs` | `PastaFile` の doc と `items` フィールドの doc | 取り込み後に修正（並行 spec 編集中のため未修正）。`file = ( file_scope \| global_scene_scope )*` に `actor_scope` が欠けている（`FileItem` の doc と `build_ast` の doc は正しい） | 4.1 |
| `crates/pasta_core/src/registry/scene_registry.rs` | `SceneEntry` の `attributes`・`fn_path`・`fn_name` の doc、`SceneRegistry` の Design Notes、`name_counters` の doc、3 つの登録関数の `attributes` 引数、`register_global_raw` の `local_names` の例、`sanitize_name` の doc | 「for future P1 filtering」「Full Rune function path」「P0/P1 Implementation」「(P1 feature)」「Rune identifiers」を、属性フィルタ用（辞書確定では常に空）・`fn_name` に `crate::` を付けたもので検索には使わない・カウンタの採番と `register_global_raw`・生成 Lua 識別子と登録キー用、に修正。`local_names` の例の旧形式 `__選択肢_1__` を現行の `選択肢_1` に修正 | 4.2 |
| `crates/pasta_core/src/registry/scene_types.rs` | `SceneInfo::fn_name` の doc | 「Generated function name in Rune code」を `global::local` 形式のシーン関数名に修正 | 4.2 |
| `crates/pasta_core/src/registry/scene_table.rs` | `from_scene_registry` の doc とコメント 2 か所、`resolve_scene_id`・`resolve_scene_id_unified`・`collect_scene_candidates`・`find_scene` の doc | トランスパイル専用という記述を辞書確定でも使うと修正。存在しない `select_label_to_id` への言及と「fn_name の一意性は SceneRegistry が検証する」（検証していない）を削除。「P1 runtime resolution」「2-stage search」「Fallback Strategy（ローカル→グローバル）」を、スコープ 1 つだけの検索でフォールバックしない、に修正し、`resolve_scene_id` がローカルのキーを除外しないことを明記。存在しない `execute_scene()` との互換という記述を、ランタイムの検索では使わない、に修正 | 4.2 |
| `crates/pasta_core/src/registry/word_table.rs` | `collect_word_candidates`・`search_word` の doc | 「Fallback Strategy（ローカル→グローバル）」「2-stage prefix matching」「local + global merge」を、スコープ 1 つだけの検索でフォールバックしない、一致した項目の値をキー順につなげる、に修正 | 4.2 |
| `crates/pasta_core/src/registry/random.rs` | `RandomSelector` の doc | 存在しない「Any 型を使う回避策」を、添字ベースのメソッドだけでオブジェクト安全を保ち、検索表は `shuffle_usize` だけを使う、に修正 | 4.2 |
| `crates/pasta_lua/src/search/mod.rs` | `loader`・`register` の引数の doc | 「from transpilation」を、トランスパイル時または `finalize_scene` の再構築によるレジストリ、に修正 | 4.2 |
| `crates/pasta_lua/pasta_scripts/pasta/act.lua` | `find_act_handler` の doc と `ACT_IMPL.word` の doc | 取り込み後に修正（並行 spec dynamic-word-reference が編集中のため未修正）。「6段階」とあるが検索レベルは L1〜L5 の 5 段。「scene/expr モードは SCENE.search を直接呼び出す（@pasta_search 可用性チェックは package.loaded 参照）」とあるが、実際は全モードとも呼び出しごとの `pcall(require, "@pasta_search")` で取得し、取得できたときだけ L2・L5 を行う | 4.2 |
| `crates/pasta_lua/src/runtime/mod.rs` | `with_config_and_source_map` の `unsafe` の SAFETY コメント（2 項目め） | `validate_and_warn` が警告する対象を「debug/ffi」から、実装どおり「std_debug・std_all_unsafe と `env` モジュール」に修正（`ffi` は警告しない） | 4.3 |
| `crates/pasta_lua/src/runtime/factory.rs` | `from_loader_with_scene_dic` の doc（Initialization Sequence の 2） | 登録する Rust モジュールの列挙に、実際に登録している `@pasta_log` を追加 | 4.3 |
| `crates/pasta_lua/src/runtime/lifecycle.rs` | `save_persistence_data` の doc | 旧名「ctx.save」を、実際に保存する `pasta.save` の表（`require("pasta.save")`）に修正 | 4.3 |
| `crates/pasta_lua/pasta_scripts/pasta/save.lua` | モジュール冒頭のコメント | 「ランタイム起動時に自動ロード」「ctx.saveから参照可能」を、最初に `require` された時点で `@pasta_persistence.load()` により読み込まれ、`act.save`（`act:init_scene` の戻り値）から参照する、に修正 | 4.3 |
| `crates/pasta_lua/pasta_scripts/pasta/global.lua` | モジュール冒頭のコメント | 「単語参照時にL5で検索される」を、単語参照・Call 時に L4（`GLOBAL[key]` の完全一致）で検索される、に修正（L5 はグローバル辞書の前方一致） | 4.3 |
| `crates/pasta_lua/pasta_scripts/ct.lua` | `new` の doc | 「<close>構文で利用します」を、`__close` は Lua 5.4 の `<close>` 向けで、LuaJIT 2.1 ランタイムには `<close>` が無いため自動では呼ばれない（ランタイムは CT を使っていない）、に修正 | 4.3 |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua` | モジュール冒頭の「Rust側統合パターン」、`set_co_scene` の doc | 統合パターンの所在「main.lua」を `pasta.shiori.entry`（実際は `xpcall` で保護して呼ぶ）に修正。`set_co_scene` の doc に、LuaJIT 2.1 には `coroutine.close` が無いため close 分岐は実行されず、破棄は参照を外して GC に任せることを追記 | 4.3 |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua` | モジュール冒頭の「使用例（ハンドラ登録）」 | 例の `return RES.ok(act:build())` は、`EVENT.fire` が返された文字列をさらに `RES.ok` で包むため応答全体が `Value` になる誤った書き方だった。`return act:build()` に直し、不要になった `local RES = require(…)` の行を、文字列を返す理由のコメントに置き換え | 4.3 |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/event/register.lua` | モジュール冒頭の「使用例」 | 同じ理由で、3 つの例の `return RES.ok([[…]])` を `return [[…]]` に直し、`local RES = require(…)` の行を、文字列を返す理由のコメントに置き換え | 4.3 |
| `crates/pasta_lua/src/loader/mod.rs` | `PastaLoader` の doc（段階の列挙） | 欠けていた段階 2.5（フレームワークスクリプトの自己展開）と 5.5（デバッグ有効時のソースマップ構築）を補い、3 の検出対象「dic/*/*.pasta」を `[loader] pasta_patterns`（既定 `dic/**/*.pasta`）と派生する `.lua` のパターンに修正し、4 に `.lua` のコピーを追記 | 4.4 |
| `crates/pasta_lua/src/loader/config/mod.rs` | `LoaderConfig::debug_mode` の doc | 「save transpiled files」を、読み込み時の処理件数と孤立キャッシュのパスをログに出す（キャッシュへの保存は値に依らず行う）、に修正 | 4.4 |
| `crates/pasta_lua/src/runtime/module_registry.rs` | `register_config_module` の doc | 「read-only Lua table」を、普通の（書き換えられる）表、に修正（`toml_to_lua` は保護の無い表を作る） | 4.4 |
| `crates/pasta_lua/src/loader/process.rs` | `module_key` の doc | 「Same derivation as `CacheManager::source_to_module_name`」「the leading `dic` component」を、先頭の `dic` を文字列の接頭辞として除きパス要素の境界を確かめない点で `source_to_module_name` と異なる、に修正（この食い違いの影響は付録 B） | 4.4 |
| `crates/pasta_lua/pasta_scripts/README.md` | 2 段落目の 1 文目 | ソースコメントではなく、埋め込みの zip に含まれて自己展開される文書。「パッケージビルド時に自動的に配布物へコピーされます」を、ビルド時に zip へ固めて pasta.dll に埋め込まれ起動時に `profile/pasta/pasta_scripts/` へ自己展開される、に修正（zip の内容が変わるため `PASTA_SCRIPTS_MD5` も変わる。`pasta_scripts/` の Lua のコメント修正と同じ扱い） | 4.4 |
| `crates/pasta_shiori/src/actor/mod.rs` | モジュール doc（冒頭） | 存在しない `actor_poc/` を「出荷経路へ昇格する先」と現存するように書いていたのを、PoC の足場は撤去済みで本モジュール群が出荷経路である、に修正 | 4.5 |
| `crates/pasta_shiori/Cargo.toml` | `wintf-winmsg-executor` の依存のコメント | 撤去済みの feature `actor-poc` と「default 無効の actor_poc 由来コード」への言及を、PoC の feature は撤去済みで wintf は出荷経路の通常依存、に修正 | 4.5 |
| `crates/pasta_shiori/src/windows_tests.rs` | モジュール doc（統合テストの置き場所） | 撤去済みの feature への言及「`actor-poc` 不要の既定ビルド」を「既定ビルド」に修正 | 4.5 |
| `crates/pasta_shiori/src/actor/mailbox.rs` | モジュール doc（由来・チャンネル決定・task 3.2 のスコープ）、`MailboxRequest`・`Reply` の doc、`ActorMsg::Get`・`Notify` の `req` と `Stop` の `done` の doc、`ActorMsg::stop`・`mailbox` の doc | 存在しない `actor_poc/mailbox.rs` を撤去済みの PoC 実装に、「将来の `static MAILBOX`」を現存する `static MAILBOX` に、done ack の待ち方「`recv()`」を `recv_timeout` に修正。「task 3.2 の placeholder」「本番では `LuaRequestTable` へ置換する」を、生のリクエスト文字列を運びアクタースレッドでテーブル化する現行の形に、`Reply` が運ぶものを「さくらスクリプト」から SHIORI 応答文字列の全体（エラー時は 500 応答）に修正 | 4.5 |
| `crates/pasta_shiori/src/actor/marshaling.rs` | モジュール doc（timeout→204 の箇条・「本タスクのスコープ / 申し送り」・GET タイムアウト閾値の節） | 「`static MAILBOX` の実体は導入しない（task 5.1 まで）」を、出荷経路では `lifecycle::marshal_request` が `MAILBOX` の送信端を渡す、本モジュールの `marshal_request` はテスト用、に修正。「次の OnSecondChange が同コルーチンを resume して回復する」を、タイムアウトした GET の処理は最後まで続き応答は捨てられ VM の状態はその結果どおりに更新される、に修正 | 4.5 |
| `crates/pasta_shiori/src/actor/lifecycle.rs` | `spawn_actor` の doc（戻り値） | 「spawn 自体に失敗した場合は `false`」を、スレッド起動の失敗は `spawn_actor_thread` の `expect` で panic する（unwind プロファイルでは FFI 入口の `catch_unwind` が `false` にする）、に修正。ロードが失敗してもアクターは `MAILBOX` に登録され、以後の GET はランタイム構築の失敗なら理由付きの 500、設置ディレクトリ不在なら原因を含まない 500（`Not initialized error`）、`SHIORI.load` の失敗ならランタイムが残るため通常どおり処理される、を追記 | 4.5 |
| `crates/pasta_shiori/src/actor/thread.rs`・`crates/pasta_shiori/src/actor/teardown.rs`・`crates/pasta_shiori/src/actor/mod.rs` | `spawn_actor_thread` のループ後のコメント 2 か所、`teardown.rs` のモジュール doc（teardown の順序）と `teardown_actor` の doc、`mod.rs` の `teardown` の doc | 「メッセージ専用ウィンドウは block_on 完了時に executor が破棄する」「done ack 受信＝ウィンドウ破棄済み」を、ウィンドウは `wintf-winmsg-executor` の thread_local で、done ack の後、`block_on` が戻ってスレッドが終了するときに破棄される、に修正 | 4.5 |
| `crates/pasta_shiori/src/windows.rs` | モジュール doc（panic 封じ込め） | 「アクタースレッド上の VM panic はアクター側で捕捉され」を、捕捉する処理は無く（unwind プロファイルでは）アクタースレッドだけが終わり reply の drop で 204 になる、に修正 | 4.5 |
| `crates/pasta_shiori/src/shiori.rs` | `PastaShiori::hinst` の doc | 「DLL module handle (for future Windows API integration)」を、`SHIORI.load` に渡す値で FFI 経路では常に 0、に修正 | 4.5 |
| `crates/pasta_lua/src/runtime/renderer_injection.rs` | モジュール doc（目的） | 「SHIORI 宿主時はアダプタがさくらスクリプトレンダラを注入する」を、構築経路（`factory.rs`）は常に既定の `RendererInjection::default` を渡しアダプタから差し替える公開経路は無い、に修正 | 4.5 |
| `crates/pasta_lua/src/runtime/factory.rs` | `from_loader_with_scene_dic` の `@pasta_sakura_script` 登録のコメント | 「アダプタ（pasta_shiori）が別レンダラを注入する場合はこの既定を差し替える」を、現行は常に既定を渡し差し替える経路は無い、に修正 | 4.5 |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/entry.lua` | `SHIORI.load` の `hinst` の doc、`SHIORI.request` の `req.method` の doc | `hinst` に「FFI 経路では常に 0」を追記。`req.method` の値「"GET", "NOTIFY"」を実際の `"get"`・`"notify"` に修正 | 4.5 |
| `crates/pasta_lua/src/sakura_script/mod.rs` | モジュール doc（冒頭・Usage の例） | 「Wait insertion」だけの説明に budoux 改行（アクター表に `budoux` があるとき `\n` を挿入）を追記。1 つ目の例の出力（一般の文字ごとに `\_w[50]`）は `script_wait_normal = 100` のアクター表のときの値であることを明記し、`nil` のアクターの例に既定値での出力 `こんにちは。\_w[950]`（既定の `script_wait_normal = 50` では一般の文字にウェイトが入らない）を追記 | 4.6 |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` | `inner_token_to_string` の doc（`@return`）と末尾のコメント、`BUILDER.build` 内の S4 のコメント、`BuildConfig` の doc | 存在しない `yield` トークンへの言及（「yield は無視」など）を、未知の型は空文字列（出力しない）に修正し、S4 の列挙から `yield` を削除。`BuildConfig` に実装が読む `buffer_factory` を追記 | 4.6 |
| `crates/pasta_lua/pasta_scripts/pasta/act.lua` | `group_by_actor` の「アクター行動トークン」の分岐のコメント | 取り込み後に修正（並行 spec dynamic-word-reference が編集中のため未修正）。対象の列挙「surface, wait, newline, clear」に `choice`・`choice_timeout` が欠けている。「talk より先にアクター行動が来た場合は無視。現在の設計ではこの状況は発生しない」は誤りで、`yield` 直後の `act:surface(…)` など最初の `talk` より前に積まれた場合に起きる（付録 B のバグ候補を参照） | 4.6 |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/event/kick.lua` | モジュール冒頭の doc | 「kick の消費は別タスク」を、`STORE.kick_force` は virtual_dispatcher の `dispatch` 入口が、`STORE.kick_pending` は `KICK.try_dispatch` が消費する、に修正 | 4.7 |
| `crates/pasta_lua/src/debug/mod.rs` | `playscene` モジュール宣言のコメント | 「DAP decode から呼ぶ口は task 4.1 で結線するため、それまで未消費」を、`wiring::inbound` の `pasta/playSceneAt` 処理が呼ぶ、に修正 | 4.7 |
| `crates/pasta_lua/src/debug/playscene.rs` | `#![allow(dead_code)]` 前の NOTE | 「結線は task 4.1。それまで未消費」を、結線済みであり dead_code 許可は結線前の名残、に修正（属性は変更せず） | 4.7 |
| `crates/pasta_lua/src/debug/kick.rs` | モジュール doc の「Wiring (later tasks)」節・`KickSink` の doc | 後続タスクで結線する・`playScene` で呼ぶ、という記述を、`enable` が socket bridge へ渡し `pasta/playSceneAt`・`pasta/reloadShiori` で呼ぶ、に修正 | 4.7 |
| `crates/pasta_lua/src/debug/enable.rs` | `enable` の doc（Scene-kick sink injection）と socket bridge 起動部のコメント | 削除済みの名前指定 `pasta/playScene` を `pasta/playSceneAt`・`pasta/reloadShiori` に修正 | 4.7 |
| `crates/pasta_lua/src/debug/wiring/bridge.rs` | `run_socket_bridge` の引数 `kick_sink` のコメント | 同上（`pasta/playScene` → `pasta/playSceneAt`・`pasta/reloadShiori`） | 4.7 |
| `crates/pasta_lua/src/runtime/mod.rs` | `with_config_and_source_map` の `debug::enable` 呼び出し前のコメント | 「`source_mode == Pasta` のときだけ `Arc` を渡す」を、提示モードに関わらず渡し各消費側が実効モードで判定する、に修正。`pasta/playScene` を `pasta/playSceneAt`・`pasta/reloadShiori` に修正 | 4.7 |
| `crates/pasta_lua/src/debug/session/mod.rs` | `DebugSession` の doc（`Cell`）、`source_map` フィールド・`with_source_map`・`source_map()` の doc | `RunMode` の保持を `Cell` から実際の `RefCell` に修正。マップは「`Pasta` のときだけ `Some`」ではなく、提示モードに関わらず渡され行ごとに実効モードで判定する、に修正 | 4.7 |
| `crates/pasta_lua/src/debug/session/stop_loop.rs` | `stop_loop` の `SetBreakpoints` の腕のコメント | 「応答の振り分けは task 4.1」を、本番では socket bridge が適用・応答してこのコマンドを転送しないため、この腕に届くのは直接送る経路（テストなど）だけ、に修正 | 4.7 |
| `crates/pasta_lua/src/debug/transport/mod.rs` | `Transport::inbound` フィールドの doc、`serve` の accept 後のコメント | `inbound` は `Option` ではないため「無効時は `None`」を、送信端を落とした閉じたチャネル、に修正。「接続後の割り込み可能な poll は後続タスク」を、読み取りタイムアウトと書き込みの `recv_timeout` で割り込み可能にしている、に修正 | 4.7 |
| `crates/pasta_lua/src/debug/dap/mod.rs` | `DapAdapter::source_resolver` フィールドの doc | 「将来の `.pasta` resolver」を、wiring が `pasta_source_resolver` を装着する、に修正 | 4.7 |
| `crates/pasta_lua/src/debug/dap/resolver.rs` | `ResolvedSource`・`SourceResolver` の doc | 「将来の pasta-source-map の resolver」「この層に `.pasta` の写像は無い」を、同じファイルの `pasta_source_resolver` が `.pasta` の位置を返す（写像の本体は `SourceMap`）、に修正 | 4.7 |
| `crates/pasta_lua/src/debug/breakpoints.rs` | モジュール doc（ロックの規律・2 段キー）、`set_breakpoints`・`register` の doc | 存在しない `block_until_command` を `stop_loop` に、「`.pasta` の変換は task 5.3（将来）」を実在の `wiring::resolver::translate_pasta_breakpoints` に修正 | 4.7 |
| `crates/pasta_lua/src/debug/source_map/mod.rs` | `SourceMap::scene_index` の NOTE、`scene_index` モジュール宣言の NOTE | 「参照側への結線は task 2.2 / 3.1。それまで未使用」を、結線済み（構築は `build_scene_index`、参照は `scene_at` → `resolve_and_kick`）、`scene_index()` は本番コードから呼ばれない、に修正 | 4.7 |
| `crates/pasta_lua/src/debug/source_map/scene_index.rs` | `#![allow(dead_code)]` 前の NOTE | 「結線は task 2.2 / 3.1 で行う。結線完了までは未使用」を、結線済みであり dead_code 許可は結線前の名残、に修正（属性は変更せず） | 4.7 |
| `crates/pasta_lua/src/code_gen/source_map.rs` | モジュール doc、`PastaPos`・`SourceMapSink`・`record_scene` の doc | 存在しない `debug/source_map.rs` を `debug/source_map/` に修正。「本番のトランスパイルはすべてシンク無し」を、通常の経路はシンク無しでデバッグ有効時のソースマップ構築だけがシンクを付ける、に修正。「`MapBuilderSink` は `record_scene` を上書きしない」を、上書きしてシーン宣言を集める、に修正 | 4.7 |
| `crates/pasta_lua/src/debug/config.rs` | `resolve`・`from_env` の doc | 存在しない `loader/config.rs` を `loader/config/` に修正 | 4.7 |
| `crates/pasta_lua/src/debug/types.rs` | `LineEvent` の doc | 「task 1.3 でフックが消費する」を、再エクスポートされているが本番のフック・セッションは使わずテストだけが作る、に修正 | 4.7 |

## 付録 B: ロードマップへの申し送り

ロードマップ（`.kiro/steering/roadmap.md`）へ送ったバグ候補（3.4）と最適化の将来候補（5.3）、およびその他の申し送り事項の記録。

| 項目 | 要旨 | 申し送り先 | 担当 |
|---|---|---|---|
| シーン・アクター名のサニタイズと検索キーの不一致（バグ候補） | 登録キーはサニタイズ済みの名前（`SceneRegistry::sanitize_name`。英数字と `_` 以外を `_` に置換）から作られる（生成コードの `PASTA.create_scene(サニタイズ済み基本名)`・ローカルシーン関数名・`register_actor`）が、検索側は名前をサニタイズせずに渡す（`act:call(…, "名前")`・`actor.lua` の `"__actor_" .. actor.name .. "__"`）。識別子に使えてサニタイズで置換される文字（例: `·` U+00B7。pest の Unicode 表によっては `・` U+30FB も）を含むシーン名・アクター名は、その名前の Call・アクター単語参照で前方一致しない。根拠: `crates/pasta_lua/src/code_gen/scope_gen.rs`、`crates/pasta_lua/src/code_gen/element_gen.rs`、`crates/pasta_core/src/registry/scene_registry.rs`、`crates/pasta_lua/pasta_scripts/pasta/actor.lua` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.2 |
| グローバルシーン検索がローカルのキーを除外しない（バグ候補） | `SearchContext::search_scene(名前, nil)` は `SceneTable::resolve_scene_id` を使い、`:` で始まるローカルのキーを除外しない（`collect_scene_candidates("", …)` は除外する）。`:` で始まる名前（動的な Call の値など）で、任意のグローバルシーンのローカルシーンが候補になりうる（ヒットした場合、`search_scene` はローカル名を `__start__` に置き換えるため、実際に解決されるのはその親グローバルシーンの `__start__`）。根拠: `crates/pasta_lua/src/search/context.rs`、`crates/pasta_core/src/registry/scene_table.rs` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.2 |
| コールバック再開後の継続が失われる（バグ候補） | `CALLBACK.try_route` は待機中のコルーチンを `coroutine.resume` で 1 回だけ再開し、再び `stage_pending` していれば `consume_staged` で再登録するが、`set_co_scene` を呼ばない。`get_property` の後で `act:yield()`（`＞チェイントーク`）したシーンは、`suspended` のままどこからも参照されず、続きが実行されない。また `resume_until_valid` を通らないため、再開後の最初の中断が出力の無い `yield` だと 204 を返してそのまま失われる。根拠: `crates/pasta_lua/pasta_scripts/pasta/shiori/event/callback.lua`、`crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.3 |
| CT（`ct.lua`）が LuaJIT ランタイムで機能しない（既知負債） | `IMPL` に `__index` が無く `obj:defer(…)`・`obj:cancel()` を呼べない（`ct_test.lua` が現行挙動として固定）。`__close` は Lua 5.4 の `<close>` 向けで、LuaJIT 2.1 には `<close>` が無いため自動では呼ばれない。ランタイムの Lua モジュールは CT を使っていない。修正か撤去（zip 出荷物の公開面としての扱いを含む）の判断を要する。根拠: `crates/pasta_lua/pasta_scripts/ct.lua`、`crates/pasta_lua/tests/lua_specs/ct_test.lua` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.3 |
| コールバックのタイムアウト応答が 200 の Value に二重に包まれる（バグ候補） | `CALLBACK.sweep` は期限切れで `on_timeout` が文字列のとき `RES.err(…)`（`SHIORI/3.0 500 …` の応答全体の文字列）を返し、`REG.OnSecondChange` はそれをそのまま返す。`EVENT.fire` は文字列の戻り値を `RES.ok` で包むため、ベースウェアには `Value` に 500 応答の全文を持つ 200 OK が返る。既存のテスト（`crates/pasta_lua/tests/lua_specs/shiori_entry_test.lua`）は `REG.OnSecondChange` を直接呼んで戻り値の文字列を確かめており、`EVENT.fire` を通した応答は検証していない。根拠: `crates/pasta_lua/pasta_scripts/pasta/shiori/event/callback.lua`、`crates/pasta_lua/pasta_scripts/pasta/shiori/event/second_change.lua`、`crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.3 |
| タイムアウト掃引での再開の結果が捨てられ、予約が取り残される（バグ候補） | `CALLBACK.sweep` は `coroutine.resume` の戻り値（エラーと yield した出力）を見ずに捨てる。`on_timeout` が文字列でない静かな経路（`get_property` に文字列でも `nil` でもない `timeout_message`（`false` など）を渡したとき、または `stage_pending` を `nil` で直接呼んだとき）では、再開されたシーンが続けて `get_property` を呼ぶと、`stage_pending` が `_staged` を立てたまま yield し、その出力は捨てられて `_staged` は消費されない。その後、コルーチンを返す次の `EVENT.fire` では、そのシーンが `get_property` を呼べば `stage_pending` が「multiple staging detected」のエラーになり、呼ばなければ `consume_staged` が新しいシーンのコルーチンを古いイベント名で `pending` に登録して `STORE.co_callback` を立てる（シーンが中断していれば `set_co_scene` はそれを `STORE.co_scene` に入れないため、続きは OnTalk で再開されない）。根拠: `crates/pasta_lua/pasta_scripts/pasta/shiori/event/callback.lua`、`crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua`、`crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.3 |
| モジュール名に `.` を含むファイル名でシーンを解決できない（バグ候補） | `source_to_module_name` は拡張子だけを除いて区切りを `.` に置き換えるため、ファイル名やディレクトリ名に `.` を含む `.pasta`・`.lua`（例: `dic/v1.2.pasta`）は `pasta.scene.v1.2` になる。キャッシュ先は `pasta/scene/v1.2.lua` だが、searcher はモジュール名の `.` をすべて区切りに戻して `pasta/scene/v1/2.lua` を探すため見つからず、`pasta.scene_dic` の `require` が失敗して起動が止まる。コード読解に基づく。根拠: `crates/pasta_lua/src/loader/cache.rs`、`crates/pasta_lua/src/runtime/searcher.rs` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.4 |
| 設置パスの glob メタ文字でファイル検出が狂う（バグ候補） | `discover_files` は基準ディレクトリとパターンを連結した文字列をそのまま `glob` に渡し、基準ディレクトリ側をエスケープしない（`glob::Pattern::escape` を使っていない）。設置パスに `[`・`]` を含むと（Windows のパスで使える）、その部分が文字クラスとして解釈され、`.pasta`・`.lua` が 1 件も見つからない（警告だけで起動は続き、シーンが無い状態になる）か、別のディレクトリに一致する。閉じていない `[` はパターンの誤りとして `LoaderError::GlobPattern` になり起動が止まる。コード読解に基づく。根拠: `crates/pasta_lua/src/loader/discovery.rs` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.4 |
| `.pasta` と `.lua` の同名衝突の判定がモジュール名と一致しない（バグ候補） | 衝突の判定に使う `module_key` は先頭の `dic` を文字列の接頭辞として除くが、実際のモジュール名（`source_to_module_name`）とキャッシュ先はパス要素の境界でだけ除く。`dic` で始まる別名のディレクトリをパターンで拾う構成（例: `dicx/a.pasta` と `dic/x/a.lua`、`dicx/a.pasta` と `dic/dicx/a.lua`）では、別モジュールの `.lua` を誤って除くか、同じモジュール名・同じキャッシュ先になる組を見逃して一方のキャッシュが他方で上書きされる。コード読解に基づく。根拠: `crates/pasta_lua/src/loader/process.rs`、`crates/pasta_lua/src/loader/cache.rs` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.4 |
| ランダムトークの間隔が起動ごとに同じ系列になる（バグ候補） | 次のトーク時刻は `virtual_dispatcher.lua` の `math.random(最小間隔, 最大間隔)` で決めるが、ランタイムのどこも `math.randomseed` を呼ばない。LuaJIT 2.1 の `math.random` は固定の初期状態から始まるため、新しい VM ごとに同じ乱数列になり、利用者スクリプトが `math.random` を使わなければ OnTalk の間隔は起動のたびに同じ並びになる（既定の 180〜300 秒では最初の 3 回が 276・264・251 秒）。`mlua::Lua::new()` で `math.random(180,300)` を 3 回呼ぶと 2 つの VM で同じ値になることを確認した。根拠: `crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.5 |
| `unload` を経ないプロセス終了で DllMain の detach が 5 秒待つ（バグ候補） | `DllMain` の `DLL_PROCESS_DETACH` は `unload()` → `teardown_actor` を呼ぶ。ホストが `unload` を呼ばずにプロセスを終了すると、OS がアクタースレッドを先に終わらせるため mailbox の受信側は破棄されないまま残り、`Stop` の送信は成功するが完了の通知は来ず、`TEARDOWN_TIMEOUT`（5 秒）まで待ってから異常として終わる。`PastaShiori` の `Drop` も走らないため、`SHIORI.unload` と永続化データの保存も行われない。通常のホストは `unload` を先に呼ぶため、この経路は `MAILBOX` が空の冪等な no-op になる。コード読解に基づく。根拠: `crates/pasta_shiori/src/windows.rs`、`crates/pasta_shiori/src/actor/lifecycle.rs`、`crates/pasta_shiori/src/actor/teardown.rs` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.5 |
| ACT のグループ化がトークンを捨てる・並べ替える（バグ候補） | `ACT_IMPL.build` の `group_by_actor` は、(1) 最初の `talk`・`sakura_script` より前に積まれた `surface`・`wait`・`newline`・`clear`・`choice`・`choice_timeout` を捨てる。利用者章 `book/src/lua/patterns.md` の例 `act:yield()` の直後の `act:surface(5):wait(500):talk(…)` では `\s[5]` と `\_w[500]` が出力されない。(2) `spot`・`clear_spot` がグループを閉じないため、同じアクターの発言が `act:set_spot`・`act:clear_spot`（`％` 行を持つシーンの呼び出しなど）の前後にあると、後の発言も前のグループに入り、スポットの変更はそのグループの出力の後に処理される（後の発言が変更前のスポットに出る）。Lua 5.4 で `pasta.act` を読み込み、両方の列を `build` して確認した。根拠: `crates/pasta_lua/pasta_scripts/pasta/act.lua`（`group_by_actor`） | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.6 |
| 位置からのキックが前方一致で別のシーンを起動しうる（バグ候補） | `KICK.try_dispatch` は確定済みの実行時 identity を、グローバルなら `SCENE.co_exec(act, "会話1")`（`act:find_scene` → `find_handler` の L5 は `SCENE.search(名前, nil)`）、ローカルなら `SCENE.search("挨拶_1", "会話1")` で引く。どちらも前方一致の検索（`SceneTable::resolve_scene_id`・`collect_scene_candidates`）であり、グローバルの `会話1` は `会話10`〜`会話19` や `会話1日目1` などにも、ローカルの `:会話1:挨拶_1` は `:会話1:挨拶_10` などにも一致し、候補から乱数で 1 つを選ぶ。同名のシーンが 10 個以上あるとき、または名前が別のシーン名の接頭辞になっているとき、カーソル位置と違うシーンが再生されうる。コード読解に基づく。根拠: `crates/pasta_lua/pasta_scripts/pasta/shiori/event/kick.lua`、`crates/pasta_core/src/registry/scene_table.rs` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.7 |
| 末尾が数字のシーン名はシーン identity 索引から漏れる（バグ候補） | `build_scene_index` は実行時のグローバル名（サニタイズした名前と連番を区切りなしで連結した形）の末尾の ASCII 数字をすべて連番とみなして分ける（`split_runtime_global`）。名前自体が数字で終わるシーン（例 `章1` の 1 番目は実行時 `章11`）は `(章, 11)` に分かれ、記録の `G:章1#1` と突き合わず索引に入らない（その配下のローカルシーンも親が引けず入らない）。その範囲にカーソルを置いてキックすると、包含するシーンが無いため `scene_at` の後方フォールバックで下にある別のシーンが選ばれる。コード読解に基づく。根拠: `crates/pasta_lua/src/debug/source_map/scene_join.rs`、`crates/pasta_lua/src/debug/source_map/scene_index.rs` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.7 |
| Windows で同じデバッグポートへの二重 bind が失敗しない（バグ候補） | `Transport::start` は bind の前に `SO_REUSEADDR` を立てる。Windows の `SO_REUSEADDR` は、相手も `SO_REUSEADDR` を立てていれば listen 中のポートへの bind を許す。2 つのゴースト（または終了しきらない旧プロセス）が同じポートでデバッグを有効にすると、2 つ目も bind に成功して `DebugError::Bind` と bind 失敗の警告ログが出ず、クライアントがどちらにつながるかが定まらない。.NET のソケットで `ReuseAddress` を立てた 2 つのソケットが 127.0.0.1 の同じポートで bind と listen に成功し、立てないソケットは失敗することを確認した。根拠: `crates/pasta_lua/src/debug/transport/mod.rs` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.7 |
| C のフレームを挟むと下位フレームの変数が別のフレームのものになる（バグ候補） | `stackTrace` のフレーム `id` は C のフレームを除いた並びの位置（`capture_stack`・`encode_frames`）だが、`variables` は `variablesReference - 1` をそのまま `lua_getstack` のレベルとして使う（`capture_variables`）。`lua_getstack` のレベルは C のフレームも数えるため、間に C のフレーム（`SHIORI.request` の `xpcall` など）があると、それより下のフレームを選んだとき 1 つずれたフレームのローカル変数（C のフレームなら空）が表示される。コード読解に基づく。根拠: `crates/pasta_lua/src/debug/inspect.rs`、`crates/pasta_lua/src/debug/dap/codec.rs`、`crates/pasta_lua/src/debug/session/stop_loop.rs` | `.kiro/steering/roadmap.md` の「内部設計執筆で判明したバグ候補」小節（5.4 で追加） | 4.7 |
| デバッグ接続は 1 回の起動につき 1 回だけ（利用者章に記載なし） | `serve` は最初の接続でリスナーを閉じ、クライアントが切断すると socket bridge が終わってポートを解放する。同じランタイムへ再接続する経路は無く、SHIORI を読み込み直すまで attach できない（SHIORI リロード後の自動再アタッチはこれを前提に動く）。利用者向けのデバッグ章（接続できないときの切り分けを含む）はこの制限を書いていない。根拠: `crates/pasta_lua/src/debug/transport/mod.rs`、`crates/pasta_lua/src/debug/wiring/bridge.rs` | `.kiro/steering/roadmap.md`（利用者章 `book/src/debug/troubleshooting.md` への追記候補） | 4.7 |
