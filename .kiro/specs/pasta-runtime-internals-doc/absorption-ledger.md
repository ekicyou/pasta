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
| `OPTIMIZATION.md` ## 6. ビルドプロファイル最適化 | 4.5 |  |  |  |  |
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
| `crates/pasta_core/README.md` ## アーキテクチャ | 4.2（→5.3） |  |  |  |  |
| `crates/pasta_core/README.md` ## ディレクトリ構成 | 4.2（→5.3） |  |  |  |  |
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
| `crates/pasta_shiori/README.md` ## アーキテクチャ | 4.5（→5.3） |  |  |  |  |
| `crates/pasta_shiori/README.md` ## ディレクトリ構成 | 4.5（→5.3） |  |  |  |  |
| `crates/pasta_shiori/README.md` ## SHIORI プロトコル | 4.5（→5.3） |  |  |  |  |
| `crates/pasta_shiori/README.md` ### プロトコルフロー | 4.5（→5.3） |  |  |  |  |
| `crates/pasta_shiori/README.md` ### サポートイベント | 4.5（→5.3） |  |  |  |  |
| `crates/pasta_shiori/README.md` ### SHIORI/3.0 リクエスト形式 | 4.5（→5.3） |  |  |  |  |
| `crates/pasta_shiori/README.md` ### レスポンス形式 | 4.5（→5.3） |  |  |  |  |
| `crates/pasta_shiori/README.md` ### FFI 境界の安全性 | 4.5（→5.3） |  |  |  |  |
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
| `crates/pasta_lua/README.md` ## SHIORI 統合 | 4.5（→5.2） |  |  |  |  |
| `crates/pasta_lua/README.md` ### pasta.shiori.res モジュール | 4.5（→5.2） |  |  |  |  |
| `crates/pasta_lua/README.md` ### pasta.shiori.sakura_builder モジュール | 4.6（→5.2） |  |  |  |  |
| `crates/pasta_lua/README.md` #### グループ化トークン形式（推奨） | 4.6（→5.2） |  |  |  |  |
| `crates/pasta_lua/README.md` #### レガシーフラット形式（後方互換） | 4.6（→5.2） |  |  |  |  |
| `crates/pasta_lua/README.md` ## ファイル検出パターン | 4.4（→5.2） |  |  |  |  |
| `crates/pasta_lua/README.md` ## モジュール名の生成 | 4.4（→5.2） |  |  |  |  |
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

## 付録 B: ロードマップへの申し送り

ロードマップ（`.kiro/steering/roadmap.md`）へ送ったバグ候補（3.4）と最適化の将来候補（5.3）、およびその他の申し送り事項の記録。

| 項目 | 要旨 | 申し送り先 | 担当 |
|---|---|---|---|
