# 吸収台帳: manual-ssot-authority

`doc/spec/`・`GRAMMAR.md`・スキル手書きリファレンス（および分割される `book/src/lua/modules.md`）の全見出しを 1 行ずつ列挙し、マニュアル（`book/src/`）のどこに収録するか、または除外する理由と、実装と照合した位置を記録する。吸収元を削除する前に、この台帳で「失われる規範内容が無い」ことを示す（要件 1.1, 1.2, 1.6, 3.1, 3.2, 3.3, 4.1, 10.6）。

## 凡例

### 収録先列の書き方

| 表記 | 意味 |
| ---- | ---- |
| 既存収録済み: 章#節 | マニュアルの該当節が、実装と矛盾しない形で同じ内容を既に持つ |
| 収録先（追記）: 章#節 | 節は在るが、吸収元にある事実が欠けている。追記する |
| 収録先（訂正）: 章#節 | 節は在るが、記述が実装と食い違う（またはマニュアルの記述が誤り）。実装を正として書き直す |
| 収録先（新設）: 章#節 | 節（または章）が無い。新設して収録する |
| 除外（理由） | マニュアルへ収録しない。理由を併記する |

- 章は `book/src/` からの相対パス。`#` 以降は見出しの文字列（リンクではない）。新設節の見出しは仮称であり、執筆時に変えてよい（変える場合はこの台帳の行も直す）。
- 収録先は design.md「ContentMigration と AbsorptionLedger」の章ごとの収録先表に従う。
- 吸収元列の `L数字` は吸収元ファイルの行番号（2026-10-01 時点・コミット `d2b09be8`）。見出しでない行（本文内リンク・章末フッタ）は `（本文）` `（章末）` と書く。

### 実装照合列の略記

| 略記 | 実体 |
| ---- | ---- |
| `pest:規則名` | `crates/pasta_dsl/src/parser/grammar.pest` の規則 |
| `dsl:パス` | `crates/pasta_dsl/src/` 配下 |
| `gen:elem` / `gen:scope` | `crates/pasta_lua/src/code_gen/element_gen.rs` / `scope_gen.rs` |
| `lua:パス` | `crates/pasta_lua/src/` 配下（Rust） |
| `ps:パス` | `crates/pasta_lua/pasta_scripts/` 配下（Lua ランタイム） |
| `core:パス` | `crates/pasta_core/src/registry/` 配下 |
| `L1`〜`L5` | `ps:pasta/act.lua` `find_act_handler` の検索段（L1 シーン完全一致 → L2 ローカル辞書前方一致 → L3 act メソッド → L4 GLOBAL 完全一致 → L5 グローバル辞書前方一致） |
| `A1` / `A2` | `ps:pasta/actor.lua` `find_actor_handler` の検索段（A1 アクター表の完全一致 → A2 アクター辞書前方一致） |
| 不要（理由） | 実装に対応物が無い記述（章題・索引・例示・ナビゲーション） |

- 備考列の「→1.2」「→1.3」「→1.4」は、それぞれ付録「未記載の実装事実」・「食い違い grep 記録」・「将来仕様の仕分け表」で確定させる項目への短い申し送り。ここでは照合で見つけた要点だけを書く（実装を正とする）。

## 本体

### `doc/spec/01-grammar-model.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 1. 文法モデルの基本原則 | 既存収録済み: grammar/index.md#Pasta DSL の文法モデル | 不要（章題） | — |
| L3 1.1 行指向文法 | 既存収録済み: grammar/index.md#Pasta DSL の文法モデル（行指向・複数行は Lua ブロックのみ） | pest:各 `*_line` 規則と `code_block`（複数行を取るのは `code_block` だけ） | — |
| L10 1.2 ファイル構造（俯瞰） | 収録先（追記）: grammar/index.md#ファイル構造の俯瞰（ファイルレベル属性行、アクター辞書配下の単語・属性・変数代入・Lua ブロック、`＊` 単独行を追加） | pest:`file`・`file_scope`・`actor_scope_item`・`global_scene_scope` | ファイル直下は file_scope／actor_scope／global_scene_scope の任意順の並び |
| L21 1.3 式（Expression）のサポート | 既存収録済み: grammar/index.md#式の基本例、grammar/variables.md#式（Expression）のサポート | pest:`expr`・`term`・`bin` | — |
| L25 式の構文 | 収録先（追記）: grammar/variables.md#式（Expression）のサポート（項は括弧式・関数呼び出し・変数参照・数値・文字列の 5 種。規則名は書かない） | pest:`term` | 吸収元の「pasta2.pest」は旧名（実体は grammar.pest） |
| L34 対応演算子 | 既存収録済み: grammar/index.md#式の基本例、grammar/markers.md#算術演算子 | pest:`bin_op`（`add`・`sub`・`mul`・`div`・`modulo`）、gen:elem `generate_expr_to_buffer`（Lua の `+ - * / %` へ） | — |
| L44 使用例 | 既存収録済み: grammar/index.md#式の基本例 | 不要（例示。規則は L25 行で照合） | — |
| L54 式文（ExprStmt） | 既存収録済み: grammar/variables.md#式文（ExprStmt） | pest:`var_set_none`、gen:elem `generate_var_set` | — |
| L74 複雑な演算 | 既存収録済み: grammar/variables.md#式（Expression）のサポート（複雑な処理は Lua ブロックで関数化） | gen:elem（ローカル関数呼び出しは `act:expr_fn`）、ps:pasta/act.lua `expr_fn`（L1 で `SCENE.関数` を解決） | 例の `＄result＝＠calculate()` は実装どおり |
| （章末）関連章 | 除外（doc/spec 章間ナビゲーション。マニュアルの章間リンクが担う） | 不要（ナビゲーション） | — |

### `doc/spec/02-markers.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 2. キーワード・マーカー定義 | 既存収録済み: grammar/markers.md | 不要（章題） | — |
| L3 2.1 基本要素 | 既存収録済み: grammar/markers.md#基本要素 | 不要（節見出し。下位行で照合） | — |
| L5 改行（NEWLINE） | 既存収録済み: grammar/markers.md#改行（NEWLINE） | pest:`eol = NEWLINE`（pest 組み込み。`\r\n`・`\n`・`\r`） | — |
| L16 空白（WHITE_SPACE 文字クラス） | 収録先（訂正）: grammar/markers.md#空白（WHITE_SPACE）（「Unicode White_Space から改行を除いたもの」ではなく、実装が列挙する文字に限る） | pest:`space_chars`（U+0020・タブ・U+3000・U+00A0・U+1680・U+2000〜U+200A・U+202F・U+205F） | U+000B・U+000C・U+0085・U+2028・U+2029 は空白に含まれない →1.3 |
| L33 コロン（Colon） | 収録先（訂正）: grammar/markers.md#コロン（`：` / `:`）（用途から「変数代入」を削除し、引数の区切りを読点／カンマへ） | pest:`kv_marker`（`key_words`・`key_attr`・`key_expr`・`action_line`・`continue_action_line`・`cue_scoped_ident`）、`set_marker = equals`（`var_set`） | 変数代入のコロン形は受理されない →1.3 |
| L50 識別子（Identifier） | 収録先（追記）: grammar/markers.md#識別子（Identifier）（先頭 `_` 可、`__` で始まり `__` で終わる名前は不可） | pest:`id = !(reserved_id) ~ identifier`、`reserved_id = dunder ~ idn2* ~ dunder` | 数字だけの変数名 `＄０` は `digit_id`（→1.2） |
| L71 インデント（Indent） | 既存収録済み: grammar/markers.md#インデント（Indent）、grammar/block-structure.md#インデントの判定 | pest:`pad = space_chars+`（各インデント行規則の先頭） | — |
| L99 2.2 シーン・マーカー | 既存収録済み: grammar/markers.md#マーカー一覧 | 不要（節見出し） | — |
| L101 グローバルシーン（Global Label Marker） | 既存収録済み: grammar/markers.md#マーカー一覧、grammar/block-structure.md#グローバルシーン | pest:`global_marker`・`global_scene_line` | — |
| L108 ローカルシーン（Local Label Marker） | 既存収録済み: grammar/markers.md#マーカー一覧 | pest:`local_marker`（`・` と半角 `-` のみ） | — |
| L115 アクター辞書（Actor Dictionary Marker） | 既存収録済み: grammar/markers.md#マーカー一覧、grammar/actor-dictionary.md | pest:`actor_marker = modulo`・`actor_line`・`scene_actors_line` | — |
| L124 属性（Attribute Marker） | 収録先（訂正）: grammar/markers.md#マーカー一覧（属性の「処理は将来予定」を「構文は受理されるが処理に反映されない」へ） | pest:`attr_marker`・`attr` | 現行挙動のみ記述（1.3 要件） |
| L133 2.3 変数・関数 | 既存収録済み: grammar/markers.md#マーカー一覧 | 不要（節見出し） | — |
| L135 単語登録・参照・呼び出し（At Marker） | 収録先（訂正）: grammar/words.md#単語の定義・#単語の参照、grammar/variables.md#関数スコープの展開先（単語の登録先は位置で決まる、`＠＄` は削除、関数引数は読点／カンマ区切りで位置引数可） | pest:`word_marker`・`fn_marker`・`key_words`・`word_ref`・`fn_call`・`args` | `＠＄`（→1.4）、空白区切りの名前付き引数（→1.3） |
| L151 変数宣言・代入（Dollar Marker） | 収録先（訂正）: grammar/variables.md#変数の種類とスコープ（代入は `＝` のみ） | pest:`var_set_local`・`var_set_global`・`var_set_property`・`set = set_marker ~ …` | `＄my_var ： 10` は受理されない →1.3 |
| L158 変数スコープ修飾子 | 既存収録済み: grammar/variables.md#グローバル変数 | pest:`var_ref_global`・`var_set_global`、gen:elem `resolve_var_path`（Global → `save.名前`） | — |
| L168 2.4 制御フロー | 既存収録済み: grammar/call-jump.md | 不要（節見出し） | — |
| L170 Call マーカー | 収録先（訂正）: grammar/call-jump.md#Call の基本（構文から `＆filter` を除き、引数リストは任意の括弧） | pest:`call_scene = call_marker ~ (id or call_target_expr) ~ s ~ args?` | フィルターは構文として受理されない →1.4 |
| L187 2.5 音声・会話 | 既存収録済み: grammar/sakura-script.md | 不要（節見出し） | — |
| L189 Sakura スクリプト エスケープ | 既存収録済み: grammar/sakura-script.md#エスケープ文字 | pest:`sakura_marker`（半角 `\` のみ） | — |
| L201 2.6 Lua コードブロック | 収録先（訂正）: grammar/block-structure.md#Lua ブロックの配置（収録先表: Lua ブロック規則 ch02 §2.6） | pest:`code_block`・`code_scope` | — |
| L203 ブロック開始 | 収録先（訂正）: grammar/block-structure.md#Lua ブロックの配置（開始フェンスは行頭の 3 個以上のバッククォート＋任意の識別子。`lua` 限定ではない） | pest:`code_open = PUSH(3 個以上のバッククォート) ~ id? ~ eol`（`pad` なし＝インデント不可） | 食い違い表「Lua ブロックのフェンス」。インデントしたフェンスは受理されない →1.3 |
| L211 ブロック終了 | 収録先（訂正）: grammar/block-structure.md#Lua ブロックの配置（終了は開始と同じ本数、後ろにコメント可。内容は検証されず生成 Lua へそのまま出る） | pest:`code_close = POP ~ or_comment_eol`、gen:elem `generate_code_block`（内容を無変換で出力）、gen:scope `generate_global_scene`（全シーン関数の後に出力）・`generate_actor`（識別子が `lua` のブロックだけ出力） | 「関数定義のみ許可・トランスパイラが検証」は実装に無い →1.3 |
| L258 2.7 演算子 | 既存収録済み: grammar/markers.md#演算子 | 不要（節見出し） | — |
| L262 算術演算子 | 既存収録済み: grammar/markers.md#算術演算子 | pest:`add`・`sub`・`mul`・`div`・`modulo` | — |
| L272 比較演算子 | 除外（現行の式に比較演算子は無い。grammar/markers.md#比較演算子 は削除対象。フィルター用の比較は将来仕様の仕分け表（1.4）で扱う） | pest:`bin_op`（算術 5 種のみ。`gt`・`lt` は式の演算子として使われない） | マニュアルに未実装の構文が載っている →1.3 |
| L285 括弧 | 既存収録済み: grammar/markers.md#括弧 | pest:`lparen`・`rparen` | — |
| L294 2.8 リテラル・文字列 | 既存収録済み: grammar/literals.md | 不要（節見出し） | — |
| L296 日本語文字列 | 収録先（追記）: grammar/literals.md#文字列（String）（`「」` は 1〜4 重で囲める。空文字列 `「」`） | pest:`slfence_ja1`〜`slfence_ja4`・`string_blank` | — |
| L303 英語文字列 | 収録先（訂正）: grammar/literals.md#文字列エスケープ（エスケープ規則は無い。`"` の本数を揃えた多重囲み、空文字列 `""`） | pest:`slfence_en = PUSH("\""+)`・`string_contents` | 食い違い表「文字列エスケープ」 |
| L315 数値リテラル | 既存収録済み: grammar/literals.md#数値（i64 / f64） | pest:`number_literal = sub? ~ digit+ ~ (dot ~ digit+)?`・`digit`（全角数字可）・`dot`（`．` 可） | — |
| L331 2.9 単語値の区切り文字 | 既存収録済み: grammar/markers.md#単語値の区切り文字 | 不要（節見出し） | — |
| L333 区切り文字 | 収録先（追記）: grammar/words.md#グローバル単語定義（末尾カンマ可、引用なしの値は空白を含む） | pest:`comma`・`comma_sep`・`words = word ~ (comma_sep ~ word)* ~ comma_sep?`・`word_nofenced` | 食い違い表「単語値」 |
| L352 2.10 コメント | 既存収録済み: grammar/block-structure.md#コメント | 不要（節見出し） | — |
| L354 コメント行 | 収録先（追記）: grammar/block-structure.md#コメント（行末コメントの可否を行の種類ごとに） | pest:`or_comment_eol`・`blank_line`。`action_line`・`continue_action_line` は `eol` 終端で行末コメント不可 | 末尾 `#` コメント →1.2 |
| L370 2.11 キューコマンド | 収録先（新設）: grammar/block-structure.md#キューコマンド行 | pest:`cue_cmd_line` | — |
| L372 キューコマンドマーカー | 収録先（新設）: grammar/block-structure.md#キューコマンド行（構文・引数型。`!select` だけが Lua を生成し、他は何もしない） | pest:`cue_cmd_marker`・`cue_cmd_name`・`cue_cmd_scope`・`cue_cmd_args`・`cue_arg`、dsl:parser/ast/cue.rs `CueArgToken`、gen:scope `generate_local_scene_items`（`command == "select"`）・`generate_choice_timeout` | 食い違い表「キューコマンド」 |
| L415 2.12 選択肢マーカー | 収録先（新設）: grammar/block-structure.md#選択肢行 | pest:`choice_line` | — |
| L417 選択肢行（Choice Line Marker） | 収録先（新設）: grammar/block-structure.md#選択肢行（構文）、lua/shiori-events.md#OnChoiceSelectEx（ルーティング） | pest:`choice_line`・`choice_label`（`「」` 1 重のみ）、gen:scope `generate_choice`（`act:choice(target, display)`）、ps:pasta/shiori/sakura_builder.lua（`\![*]\q[表示,ID]`） | — |
| （章末）関連章 | 除外（doc/spec 章間ナビゲーション） | 不要（ナビゲーション） | — |

### `doc/spec/03-block-structure.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 3. 行とブロック構造 | 既存収録済み: grammar/block-structure.md | 不要（章題） | — |
| L3 3.1 行（Line）の定義 | 既存収録済み: grammar/block-structure.md 冒頭本文 | pest:各 `*_line` 規則（`eol` または `or_comment_eol` で終わる） | — |
| L14 3.2 行の種類 | 既存収録済み: grammar/block-structure.md#行の種類 | 不要（節見出し） | — |
| L18 インデント不要の行構造（行頭にインデントなし） | 収録先（訂正）: grammar/block-structure.md#インデント不要の行（ファイルレベル属性行・`＊` 単独行を追加、単語定義の例を読点区切りへ） | pest:`file_attr_line`・`file_word_line`・`actor_line`・`global_scene_line`・`global_scene_continue_line`・`code_open` | `＊` 単独行 →1.2 |
| L27 インデントが必要な行構造（行頭にインデントあり） | 収録先（訂正）: grammar/block-structure.md#インデントが必要な行（属性行・ローカル単語定義はグローバルシーン初期部のみ。アクター指定行・継続行・キューコマンド行を追加） | pest:`global_scene_init`・`local_scene_item`・`scene_actors_line` | 食い違い表「属性行の配置」 |
| L41 3.3 ブロック構造 | 既存収録済み: grammar/block-structure.md#ブロック構造 | 不要（節見出し） | — |
| L45 グローバルブロック構造 | 収録先（訂正）: grammar/block-structure.md#ブロック構造（ファイルはファイルスコープ・アクタースコープ・グローバルシーンの任意順の並びで、グローバルシーンは 0 個でもよい） | pest:`file = SOI ~ (file_scope or global_scene_scope or actor_scope)* ~ s ~ EOI` | — |
| L57 グローバルシーンブロック構造 | 収録先（訂正）: grammar/block-structure.md#グローバルシーンの内部構造（初期部＝属性行・単語定義・アクター指定行 → Lua ブロック → 暗黙開始ブロック → ローカルシーン） | pest:`global_scene_scope`・`global_scene_init`・`local_start_scene_scope` | — |
| L78 ローカルブロック構造 | 収録先（訂正）: grammar/block-structure.md#ローカルシーン（属性は宣言行への付記のみで行としては置けない。コンテンツ行に継続行・キューコマンド行を追加） | pest:`local_scene_line`・`scene = id ~ s ~ attrs?`・`local_scene_item` | 食い違い表「属性行の配置」 |
| L99 Lua ブロックの配置 | 収録先（訂正）: grammar/block-structure.md#Lua ブロックの配置（Lua ブロックはロード時に評価される。「`__start__` 内で最初に実行」は誤り） | gen:scope `generate_global_scene`（Lua ブロックはグローバルシーンの do ブロック末尾、全シーン関数定義の後に出力） | 処理順序の記述が実装と不一致 →1.3 |
| L127 例 | 収録先（訂正）: grammar/block-structure.md の例（フェンスは行頭に置く） | pest:`code_open`（`pad` を取らない） | 吸収元の例はフェンスをインデントしている →1.3 |
| L162 3.4 インデント（Indentation） | 既存収録済み: grammar/block-structure.md#インデントの判定 | pest:`pad` | 例の `＠fruits：apple orange`（空白区切り）は誤り |
| （章末）関連章 | 除外（doc/spec 章間ナビゲーション） | 不要（ナビゲーション） | — |

### `doc/spec/04-call-spec.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 4. Call の詳細仕様 | 既存収録済み: grammar/call-jump.md | 不要（章題） | — |
| L3 4.1 Call ターゲットの形式 | 既存収録済み: grammar/call-jump.md#Call ターゲットの形式 | pest:`call_scene` | — |
| L7 パターン1: シーン参照 | 収録先（訂正）: grammar/call-jump.md#Call ターゲットの形式（前方一致は 5 段検索の一部） | pest:`call_scene`（`id` → `CallTarget::Static`）、gen:elem `generate_call_scene`（`act:call(SCENE.__global_name__, "名前", {}, …)`） | — |
| L23 パターン2: 動的ターゲット | 収録先（追記）: grammar/call-jump.md#Call ターゲットの形式（変数に限らず任意の式。値は `tostring` してから検索） | pest:`call_target_expr = expr`、gen:elem `generate_call_scene`（Dynamic → `tostring(式)`） | nil の扱い →1.3（call-spec L112 行参照） |
| L33 4.1.3 前方一致によるターゲット解決 | 既存収録済み: grammar/call-jump.md#前方一致によるターゲット解決 | core:scene_table.rs `collect_scene_candidates` | — |
| L49 4.1.4 スコープ解決アルゴリズム | 収録先（訂正）: grammar/call-jump.md#スコープ解決アルゴリズム（5 段。ローカル候補が 1 つでもあればローカルのみ） | ps:pasta/act.lua `find_act_handler`（L1〜L5）、core:scene_table.rs `collect_scene_candidates`（ローカル優先・統合しない） | 食い違い表「Call 検索」「ローカル／グローバル候補」 |
| L81 4.2 フィルター（属性フィルター） | 除外（構文が受理されない未実装機能。grammar/call-jump.md#フィルター（将来変更あり）は節ごと削除し、将来仕様の仕分け表（1.4）→ scene-attribute-semantics へ） | pest:`call_scene`・`word_ref`（`＆` を受ける規則が無い） | 食い違い表「Call フィルター」 |
| L103 4.3 引数リスト | 収録先（訂正）: grammar/call-jump.md#引数リスト（読点／カンマ区切り、位置引数可。`名前：値` は名前が捨てられ値が位置順に渡る。Call の引数はシーン関数の可変長引数へ） | pest:`args`・`key_arg`・`positional_arg`、gen:elem `generate_args_string`（Keyword の名前を破棄）・`generate_call_scene`（末尾に呼び出し元の `table.unpack(args)` を付加） | 食い違い表「引数」。呼び出し元引数の転送 →1.2 |
| （章末）関連章 | 除外（doc/spec 章間ナビゲーション） | 不要（ナビゲーション） | — |

### `doc/spec/05-literals.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 5. リテラル型 | 既存収録済み: grammar/literals.md | 不要（章題） | — |
| L3 5.1 概要 | 既存収録済み: grammar/literals.md 冒頭本文 | 不要（概要文） | — |
| L7 5.2 型変換ルール | 収録先（訂正）: grammar/literals.md#型変換ルール（真偽値リテラルは無い。型は文脈ごと: 式＝数値（整数・小数）と文字列、属性値＝整数・小数・引用文字列・引用なし文字列、単語値＝常に文字列。引用なしの空白で値が分かれる規則は無い） | pest:`term`・`attr_value`・`words`、dsl:parser/ast/action.rs `AttrValue`（Integer・Float・String・AttrString）、gen:elem `generate_expr_to_buffer` | `true` は式として受理されない →1.3 |
| （章末）関連章 | 除外（doc/spec 章間ナビゲーション） | 不要（ナビゲーション） | — |

### `doc/spec/06-action-line.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 6. アクション行（Action Line） | 既存収録済み: grammar/action-line.md | 不要（章題） | — |
| L3 6.1 基本構文 | 既存収録済み: grammar/action-line.md#基本構文 | pest:`action_line = pad ~ id ~ s ~ kv_marker ~ s ~ actions ~ eol` | — |
| L9 6.2 Actor（アクター） | 収録先（訂正）: grammar/action-line.md#基本構文（アクター名は識別子。任意の文字列ではない） | pest:`action_line`（`id`）、ps:pasta/act.lua `ACT_IMPL.__index`（登録済みアクターだけがプロキシになる） | 未登録アクター名の挙動 →1.2 |
| L21 6.3 Action（アクション） | 既存収録済み: grammar/action-line.md#インライン要素 | 不要（節見出し） | — |
| L25 インライン要素 | 収録先（訂正）: grammar/action-line.md#インライン要素（`＠＄` を削除。`＄＄`・`\\` のエスケープ、`＄％` のプロパティ参照、`＠＊関数()` を追加。引数は読点区切り） | pest:`action`（`at_escape`・`dollar_escape`・`sakura_escape`・`fn_call`・`word_ref`・`var_ref`・`sakura_script`・`talk`）、gen:elem `generate_action` | `＄＄`・`\\` エスケープ →1.2 |
| L53 インライン要素の区切り文字 | 既存収録済み: grammar/action-line.md#インライン要素の区切り | 不要（節見出し） | — |
| L57 空白による区切り | 既存収録済み: grammar/action-line.md#空白による区切り | pest:`word_ref`・`var_ref_local`・`var_ref_global`・`var_ref_property`（末尾の `s` が区切り空白を消費） | 関数呼び出し `fn_call` の後の空白は本文に残る →1.2 |
| L72 空白なしの場合（最長一致） | 既存収録済み: grammar/action-line.md#空白がない場合（最長一致） | pest:`id`（XID_CONTINUE の連続） | — |
| L89 6.4 行継続 | 収録先（訂正）: grammar/action-line.md#行継続（継続行はインデント＋`：` で始まる。先行するアクション行が無いとトランスパイルエラー） | pest:`continue_action_line = pad ~ kv_marker ~ s ~ actions ~ eol`、gen:elem `generate_continue_action`（`invalid_continuation`） | 食い違い表「行継続」 |
| L111 6.5 改行 | 既存収録済み: grammar/action-line.md#改行 | 不要（節見出し） | — |
| L113 6.5.1 概要 | 既存収録済み: grammar/action-line.md#改行 | 不要（概要文） | — |
| L117 6.5.2 正規の改行（Sakura） | 既存収録済み: grammar/action-line.md#改行 | pest:`sakura_script`（`\n` を透過） | — |
| L122 6.5.3 糖衣構文（継続行内の空行） | 収録先（訂正）: grammar/action-line.md#改行（継続中の空行は何も出力しない） | pest:`blank_line`（`local_scene_item` で読み捨て。継続空行の専用規則は無い） | 食い違い表「継続内の空行」 |
| L145 6.5.4 非継続領域の空行 | 既存収録済み: grammar/action-line.md#改行（継続行以外の空行は無視） | pest:`blank_line` | — |
| （章末）関連章 | 除外（doc/spec 章間ナビゲーション） | 不要（ナビゲーション） | — |

### `doc/spec/07-sakura-script.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 7. Sakura スクリプト仕様 | 既存収録済み: grammar/sakura-script.md | 不要（章題） | — |
| L3 7.1 概要 | 既存収録済み: grammar/sakura-script.md 冒頭本文 | pest:`sakura_script`（字句的に切り出して透過） | — |
| L7 7.2 エスケープ文字 | 既存収録済み: grammar/sakura-script.md#エスケープ文字 | pest:`sakura_marker` | — |
| L15 7.3 コマンドの字句構造（簡略版） | 収録先（訂正）: grammar/sakura-script.md#コマンドの字句構造（角括弧は `"…"` で囲んだ部分（内部の `""` は `"`）を除いて最初の `]` で閉じる。`\]` エスケープは無い） | pest:`sakura_args`・`sakura_body`・`sakura_str`、lua:sakura_script/tokenizer.rs `SAKURA_TAG_PATTERN`（角括弧内 `[^\]]*`） | `\]` は実装されていない →1.3 |
| L37 7.4 文字種（簡略） | 既存収録済み: grammar/sakura-script.md#コマンドの字句構造 | pest:`sakura_id`（ASCII 英数・`_ ! - + * ? &`） | — |
| L43 7.4.1 `sakura_token` 文字クラスの同期箇所 | 除外（メンテナ向けの同期手順。同期先 4 箇所のうち doc/spec と GRAMMAR.md は本仕様で廃止され、残る 2 箇所の同期は実装側の関心で利用者向けマニュアルの対象外） | pest:`sakura_id` と lua:sakura_script/tokenizer.rs `SAKURA_TAG_PATTERN` の文字クラスが一致することを確認（`0-9a-zA-Z_!+*?&-`） | 記載の `crates/pasta_core/src/parser/grammar.pest` は旧位置 |
| L54 7.5 使用例 | 既存収録済み: grammar/sakura-script.md#使用例 | 不要（例示。字句は L15 行で照合） | — |
| （章末）関連章 | 除外（doc/spec 章間ナビゲーション） | 不要（ナビゲーション） | — |

### `doc/spec/08-attributes.md`

収録先表で「ch08 の属性行構文・配置ルール（現行挙動として）」が grammar/block-structure.md へ割り当てられている行だけを本体で扱う。セマンティクス（継承・フィルター）の行き先は「将来仕様の仕分け表」。

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 8. 属性（Attribute） | 収録先（訂正）: grammar/block-structure.md#属性（将来変更あり）を現行挙動のみの節へ書き換え | 不要（章題） | — |
| L3 8.1 構文 | 収録先（訂正）: grammar/block-structure.md#属性（`＆名前：値`。1 行に複数並べられる。値は整数・小数・引用文字列・引用なし文字列（`：`・`＆`・`＃` と空白を含まない）） | pest:`attr`・`attrs = attr+`・`key_attr`・`attr_value`・`attr_string` | — |
| L9 8.2 配置ルール | 収録先（訂正）: grammar/block-structure.md#属性（行として置けるのはグローバルシーン初期部・ファイルレベル・アクタースコープ。ローカルシーンとグローバルシーンは宣言行への付記も可） | pest:`global_scene_attr_line`・`file_attr_line`・`actor_scope_item`・`scene = id ~ s ~ attrs?` | 食い違い表「属性行の配置」 |
| L32 8.3 ファイルレベル属性（将来予約） | 収録先（訂正）: grammar/block-structure.md#属性（現行挙動のみ: 構文は受理され、後続グローバルシーンの属性と統合されるが処理には使われない）。継承の意味論は仕分け表（1.4） | lua:transpiler.rs（`accumulate_file_attr`・`merge_attrs`）、gen:scope `generate_global_scene`（`_file_attrs` 未使用）、ps:pasta/act.lua `ACT_IMPL.call`（`attrs` 未使用） | — |
| （章末）関連章 | 除外（doc/spec 章間ナビゲーション） | 不要（ナビゲーション） | — |

### `doc/spec/09-variables.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 9. 変数・スコープ | 既存収録済み: grammar/variables.md | 不要（章題） | — |
| L3 9.1 変数型 | 既存収録済み: grammar/variables.md#変数の種類とスコープ | 不要（節見出し） | — |
| L5 グローバル変数 | 収録先（追記）: grammar/variables.md#グローバル変数（`save.名前` に入り、永続化ファイルへ保存される） | gen:elem `resolve_var_path`（Global → `save.名前`）、lua:runtime/lifecycle.rs（終了時に `save_to_file`） | — |
| L21 ローカル変数 | 収録先（追記）: grammar/variables.md#ローカル変数（`var.名前`。1 回のイベント処理ごとに空から始まる） | gen:elem `resolve_var_path`（Local → `var.名前`）、ps:pasta/act.lua `ACT.new`（`var = {}`） | 「一連のシーン」の範囲 →1.3 |
| L37 プロパティ変数 | 既存収録済み: grammar/variables.md#プロパティ変数 | pest:`var_ref_property`・`var_set_property`・`property_id`、gen:elem `generate_property_set`、ps:pasta/shiori/act.lua `set_property`・`get_property` | `property_id` は `(` `)` も含められる →1.2 |
| L56 使用例 | 既存収録済み: grammar/variables.md#ローカル変数・#グローバル変数 | 不要（例示） | — |
| L65 9.2 変数代入の制約 | 既存収録済み: grammar/variables.md#式（Expression）のサポート | 不要（節見出し） | — |
| L69 許可される値の型 | 収録先（訂正）: grammar/variables.md#式（Expression）のサポート（引数付き呼び出しの例を読点区切りへ） | pest:`set = set_marker ~ s ~ (expr or word_ref)`、gen:elem `generate_var_set`（`word_ref` → `act:word`） | — |
| L81 関数スコープの展開先 | 収録先（訂正）: grammar/variables.md#関数スコープの展開先（`＠func()` は `act:expr_fn("func", …)` で 5 段検索、`＠＊func()` は `GLOBAL.func(act, …)` を直接呼ぶ） | gen:elem `generate_expr_to_buffer`（FnScope::Local → `act:expr_fn`、Global → `GLOBAL.名前(act…)`）、ps:pasta/act.lua `expr_fn` | 未定義の `＠＊関数` の挙動 →1.2 |
| L90 式文（ExprStmt） | 既存収録済み: grammar/variables.md#式文（ExprStmt） | pest:`var_set_none` | — |
| L107 代替方法：Lua 関数を使用 | 収録先（訂正）: grammar/variables.md#式（Expression）のサポート（例は `function SCENE.add(act, x, y)` と `＄result＝＠add（10、20）` の形へ） | ps:pasta/act.lua `find_act_handler`（L1 シーン表・L3 act・L4 GLOBAL を探し、素の Lua グローバル関数は探さない） | 吸収元の `function add(ctx, x, y)` と `：` 代入・空白区切り引数は動かない →1.3 |
| （章末）関連章 | 除外（doc/spec 章間ナビゲーション） | 不要（ナビゲーション） | — |

### `doc/spec/10-words.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 10. 単語定義（Word Definition） | 既存収録済み: grammar/words.md | 不要（章題） | — |
| L3 10.1 グローバル単語定義 | 収録先（訂正）: grammar/words.md#グローバル単語定義（参照範囲は全辞書ファイル共通） | gen:elem `generate_global_word`（`PASTA.create_word`）、ps:pasta/word.lua `create_global` | 「ファイル全体」は実装より狭い →1.3 |
| L11 10.2 ローカル単語定義 | 既存収録済み: grammar/words.md#ローカル単語定義（グローバルシーン初期部のみ） | pest:`global_scene_word_line`（`global_scene_init` にだけ現れる）、gen:elem `generate_local_word` | — |
| L19 10.4 複数キー単語定義 | 既存収録済み: grammar/words.md#複数キー単語定義 | pest:`key_words`・`key_list`、gen:elem `generate_word_definition`（キーごとに 1 行登録） | — |
| L38 10.3 単語参照 | 収録先（訂正）: grammar/words.md#単語の参照・#スコープと優先順位（Lua 側は `act:word("名前")`。ローカル優先で統合しない。アクター付き行はアクター辞書が先。`＠＄` は削除） | ps:pasta/act.lua `ACT_IMPL.word`・`find_act_handler`、ps:pasta/actor.lua `PROXY_IMPL.word`、core:word_table.rs `collect_word_candidates` | `pasta.word_lookup` は存在しない。食い違い表「ローカル／グローバル候補」。`＠＄` →1.4 |
| （章末）関連章 | 除外（doc/spec 章間ナビゲーション） | 不要（ナビゲーション） | — |

### `doc/spec/11-actor-dictionary.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 11. アクター辞書（Actor Dictionary） | 既存収録済み: grammar/actor-dictionary.md | 不要（章題） | — |
| L3 11.1 概要 | 既存収録済み: grammar/actor-dictionary.md 冒頭本文 | pest:`actor_scope` | — |
| L12 11.2 グローバルアクター辞書定義 | 収録先（追記）: grammar/actor-dictionary.md#グローバルアクター辞書定義（配下に置ける行: 単語定義・属性行・変数代入行・Lua ブロック・空行） | pest:`actor_scope_item`、gen:scope `generate_actor`（単語と `lua` ブロックだけを出力） | 配下の属性・変数代入は受理されるが出力されない →1.2 |
| L47 11.3 シーンスコープ内でのアクター指定 | 収録先（訂正）: grammar/actor-dictionary.md#シーンスコープ内でのアクター指定（`％` 行はグローバルシーン初期部に置き、`__start__` 冒頭で立ち位置を設定する。アクター付き単語参照は `％` 行が無くても働く） | pest:`scene_actors_line`・`actors_item`、gen:scope `generate_local_scene`（`act:clear_spot()`＋`act:set_spot(名前, 番号)`）、gen:elem `generate_action`（WordRef → `act.アクター:word`） | 番号付け `名前＝数字` →1.2 |
| L71 11.4 アクタースコープと単語参照の統合 | 収録先（訂正）: grammar/actor-dictionary.md#アクタースコープと単語参照の統合（検索順は A1・A2 の後に L1〜L5）。「実装上の注意」の AST 名は除外（内部実装） | ps:pasta/actor.lua `find_actor_handler`（A1・A2）→ ps:pasta/act.lua `find_act_handler`（L1〜L5）、core:word_table.rs（シャッフル＆順次消費） | — |
| L91 11.5 将来拡張（コードブロック） | 収録先（訂正）: grammar/actor-dictionary.md#コードブロック（将来変更あり）を現行挙動の節へ（識別子 `lua` のブロックはアクターの do ブロック内に出力され、`ACTOR` を使える） | pest:`actor_scope_item`（`code_scope`）、gen:scope `generate_actor`（`language == "lua"` のみ出力）、dsl:../tests/actor_code_block_test.rs | design は「受理されるが処理に反映されない」とするが、実装は `lua` ブロックを出力し `function ACTOR.名前(act)` は A1 で呼ばれる →1.3・1.4 |
| （章末）関連章 | 除外（doc/spec 章間ナビゲーション） | 不要（ナビゲーション） | — |

### `doc/spec/12-future.md`

収録先表が章へ割り当てている (M) 項目だけに収録先を書く。それ以外は「将来仕様の仕分け表」（タスク 1.4）で行き先を確定する。

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 12. 未確定事項・検討中の仕様 | 除外（章全体の行き先は仕分け表（1.4）。下の (M) 行だけを本体で収録先へ割り当てる） | 不要（章題） | — |
| L3 12.2 チェーントーク（DSL非採用） | 収録先（新設）: grammar/call-jump.md#特殊な呼び出し（`＞チェイントーク`・`＞yield` による現行の実現方法で置き換える） | ps:pasta/global.lua（`GLOBAL.yield`・`GLOBAL["チェイントーク"]`）、ps:pasta/act.lua `ACT_IMPL.yield` | — |
| L9 12.4 ローカルシーンのパラメータ（将来検討） | 除外（未実装。仕分け表（1.4）で R） | pest:`scene = id ~ s ~ attrs?`（パラメータ構文なし） | — |
| L15 12.5 フィルター機能の詳細（初期版対応なし） | 除外（構文未受理。仕分け表（1.4）で B） | pest:`call_scene` | — |
| L21 12.6 単語定義の値の型変換ルール（初期版） | 収録先（訂正）: grammar/literals.md（単語値は常に文字列）、grammar/words.md#グローバル単語定義（読点区切り、空白は値に含まれる） | pest:`words`・`word`、gen:elem `generate_word_definition`（全値を文字列リテラル化） | 吸収元の例の空白区切りは誤り |
| L31 12.7 動的単語参照（＠＄var_name）の実装スケジュール | 除外（未実装。仕分け表（1.4）で B） | pest:`word_ref = word_marker ~ id ~ s`（`＠＄` の規則なし） | — |
| L37 12.8 Callの戻り値と変数代入（DSL非定義） | 除外（DSL 範囲外。仕分け表（1.4）で R） | pest:`set`（`＞` を値に取らない） | — |
| L43 12.9 ローカル変数のスコープ詳細（トランスパイラ/ランタイム） | 除外（`var`・`save` に置換済みの旧方針。現行挙動は grammar/variables.md。仕分け表（1.4）で除外） | gen:elem `resolve_var_path`（`var.`・`save.`・`args[n]`） | — |
| L50 12.10 属性値の型解釈 | 収録先（訂正）: grammar/literals.md（属性値は整数・小数・引用文字列・引用なし文字列） | dsl:parser/parse_elements.rs `parse_attr`・`parse_attr_number`、dsl:parser/ast/action.rs `AttrValue` | — |
| L55 12.11 前方一致時の複数候補選択ルール（DSL非定義） | 収録先（新設）: grammar/call-jump.md#スコープ解決アルゴリズム（ローカル優先・シャッフル＆順次消費） | core:scene_table.rs `collect_scene_candidates`、core:random.rs | — |
| L59 12.12 引数リストの値の型解釈 | 収録先（訂正）: grammar/literals.md、grammar/call-jump.md#引数リスト（引数は式。数値・文字列・変数・関数呼び出し・算術） | pest:`arg`・`key_expr`・`positional_arg = expr` | — |
| L64 12.13 行継続のインデント深さ制約 | 収録先（訂正）: grammar/action-line.md#行継続（継続は `：` 始まり。直前のアクション行・継続行のアクターを引き継ぐ。深さは問わない） | pest:`continue_action_line`、gen:elem `generate_continue_action`（`last_actor`） | 食い違い表「行継続」 |
| L69 12.14 コメント行の配置可能位置 | 既存収録済み: grammar/block-structure.md#コメント（行末コメントの訂正は ch02 L354 行） | pest:`blank_line`（全スコープの項目に含まれる） | — |
| L74 12.15 識別子と予約語の制限（DSL外） | 収録先（新設）: grammar/variables.md#Lua 予約語の制約 | gen:elem `resolve_var_path`（`var.名前`・`save.名前` をそのまま出力するため Lua キーワードは Lua の構文エラー） | — |
| L80 12.16 Sakuraスクリプト括弧内のエスケープ（確定） | 収録先（訂正）: grammar/sakura-script.md#角括弧内のエスケープと引用（引用と `""` の規則は透過。`\]` は無い） | pest:`sakura_str`（`"…"` 内の `""`）、`sakura_body` | `\]` →1.3 |
| L109 12.17 ファイルエンコーディングとBOM | 収録先（新設）: grammar/block-structure.md#文字コード | lua:loader/process.rs（`fs::read_to_string` で UTF-8 として読む）→ dsl:lib.rs `parse_str`（BOM を除く処理が見当たらない） | 「BOM は許容」は実測で確定 →1.3 |
| L114 12.18 ファイルレベル属性の継承詳細 | 除外（意味論は未実装。仕分け表（1.4）で B。現行挙動は ch08 L32 行） | lua:transpiler.rs `merge_attrs` | — |
| L119 12.19 空行の配置と解釈 | 収録先（新設）: grammar/block-structure.md#空行（空行はどこでも無視。継続中も何も出力しない） | pest:`blank_line` | 食い違い表「継続内の空行」 |
| L124 12.20 全角・半角混在時の正規化 | 収録先（追記）: grammar/markers.md 冒頭本文（全角と半角は同等。`＠＠`・`＄＄` のエスケープは 2 文字目をそのまま出力） | pest:`at_escape = at{2}`・`dollar_escape`、gen:elem `generate_action`（Escape → 2 文字目を出力） | — |
| （章末）更新履歴・関連章 | 除外（doc/spec の版管理とナビゲーション。履歴は git が保持） | 不要（履歴） | — |

### `doc/spec/README.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 Pasta DSL 言語仕様書 | 除外（doc/spec の索引。ディレクトリごと廃止（2.1）。文法章の索引は grammar/index.md#このセクションの読み方 が担う） | 不要（索引） | — |
| L5 概要 | 除外（doc/spec を権威とする宣言。権威はマニュアルへ移る（1.5）） | 不要（権威の宣言） | — |
| L12 章一覧 | 除外（索引。grammar/index.md の章表が担う） | 不要（索引） | — |
| L31 よくある参照パターン | 除外（索引。grammar/index.md・SUMMARY.md が担う） | 不要（索引） | — |
| L48 関連ドキュメント | 除外（旧権威体系の役割分担。廃止） | 不要（索引） | — |
| L61 参考資料 | 除外（節見出し） | 不要（節見出し） | — |
| L63 外部仕様 | 収録先（追記）: reference/external-links.md（Unicode UAX #31 と ukadoc さくらスクリプト一覧へのリンク） | 不要（外部資料へのリンク） | — |
| L70 ディレクトリ構成 | 除外（doc/spec 自身の構成。廃止） | 不要（索引） | — |

### `GRAMMAR.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 Pasta DSL Grammar Reference | 既存収録済み: grammar/index.md#Pasta DSL の文法モデル（里々／さとりに着想）。冒頭の「権威的仕様書」注記は除外（1.5） | 不要（題目） | — |
| L7 目次 | 除外（目次。SUMMARY.md が担う） | 不要（目次） | — |
| L25 基本構文 | 既存収録済み: grammar/index.md | 不要（節見出し） | — |
| L27 ファイル構造 | 収録先（追記）: grammar/index.md#ファイル構造の俯瞰（訂正内容は doc/spec ch01 L10 行と同じ） | pest:`file` | — |
| L47 マーカー一覧（全角/半角両対応） | 既存収録済み: grammar/markers.md#マーカー一覧（キューコマンドの「dola 側で処理」は「`!select` 以外は何もしない」へ） | pest:各マーカー規則、gen:scope `generate_local_scene_items` | — |
| L62 空白 | 収録先（訂正）: grammar/markers.md#空白（WHITE_SPACE） | pest:`space_chars` | doc/spec ch02 L16 行と同じ |
| L67 インデント | 既存収録済み: grammar/block-structure.md#インデントの判定（一貫性も不要） | pest:`pad` | — |
| L75 シーン定義 | 既存収録済み: grammar/block-structure.md#ブロック構造 | 不要（節見出し） | — |
| L77 グローバルシーン | 既存収録済み: grammar/block-structure.md#グローバルシーン | pest:`global_scene_line` | — |
| L87 ローカルシーン | 既存収録済み: grammar/block-structure.md#ローカルシーン | pest:`local_scene_line` | — |
| L103 シーンの重複（前方一致によるランダム選択） | 収録先（訂正）: grammar/block-structure.md#グローバルシーン、grammar/call-jump.md#前方一致によるターゲット解決（「ランダム」はシャッフル＆順次消費） | core:scene_table.rs `collect_scene_candidates`、core:random.rs | — |
| L123 予約パターン | 既存収録済み: grammar/markers.md#識別子（Identifier） | pest:`reserved_id` | — |
| L129 アクション行（発言文） | 既存収録済み: grammar/action-line.md | 不要（節見出し） | — |
| L131 基本形式 | 既存収録済み: grammar/action-line.md#基本構文 | pest:`action_line` | — |
| L144 アクターの省略 | 既存収録済み: grammar/action-line.md#アクターの省略 | pest:`continue_action_line`、gen:elem `generate_continue_action` | — |
| L158 インライン要素 | 収録先（訂正）: grammar/action-line.md#インライン要素（doc/spec ch06 L25 行と同じ追加・削除） | pest:`action` | — |
| L171 行継続 | 収録先（訂正）: grammar/action-line.md#行継続（GRAMMAR.md の「`：` 必須」が実装どおり。マニュアルのインデント継続の記述を改める） | pest:`continue_action_line` | 食い違い表「行継続」 |
| L188 属性 | 収録先（訂正）: grammar/block-structure.md#属性 | 不要（節見出し） | — |
| L190 属性の設定 | 収録先（訂正）: grammar/block-structure.md#属性（現行挙動のみ） | pest:`global_scene_attr_line` | — |
| L203 配置ルール | 収録先（訂正）: grammar/block-structure.md#属性（例のローカルシーン直下の `＆difficulty` 行は受理されない） | pest:`local_scene_item`（属性行を含まない） | 食い違い表「属性行の配置」 |
| L219 変数 | 既存収録済み: grammar/variables.md | 不要（節見出し） | — |
| L221 変数宣言と代入 | 収録先（訂正）: grammar/variables.md#変数の種類とスコープ（`＄score：100` は受理されない、`true` は値に書けない） | pest:`set_marker`・`term` | →1.3 |
| L232 変数スコープ | 既存収録済み: grammar/variables.md#変数の種類とスコープ | gen:elem `resolve_var_path` | — |
| L240 変数参照 | 既存収録済み: grammar/variables.md#変数参照の空白ルール（未代入は空文字＋警告） | ps:pasta/act.lua `ACT_IMPL.talk`（nil で `act:talk - undefined variable` を警告） | — |
| L254 式（Expression）のサポート | 既存収録済み: grammar/variables.md#式（Expression）のサポート | pest:`expr` | — |
| L268 対応演算子 | 既存収録済み: grammar/markers.md#算術演算子 | pest:`bin_op` | — |
| L284 制御構文 | 既存収録済み: grammar/call-jump.md 冒頭本文（宣言的言語・命令型制御構文なし） | 不要（性質の説明。pest に制御構文の規則は無い） | — |
| L288 Call文（サブルーチン呼び出し） | 既存収録済み: grammar/call-jump.md#Call の基本 | pest:`call_scene_line` | — |
| L302 Callターゲットの形式 | 既存収録済み: grammar/call-jump.md#Call ターゲットの形式 | pest:`call_scene` | — |
| L317 条件分岐の実現 | 既存収録済み: grammar/call-jump.md#条件分岐の実現（例の `＄スコア：75` は `＝` へ訂正） | ps:pasta/act.lua `ACT_IMPL.call`、gen:elem `generate_call_scene`（Dynamic） | `＞＠分岐判定` は関数内で act:call 済みの戻り値（nil → `"nil"`）で再検索する →1.3 |
| L353 チェイントーク（継続トーク） | 収録先（新設）: grammar/call-jump.md#特殊な呼び出し（解決段は「4 段階検索の Level 3」ではなく 5 段の L4（GLOBAL 完全一致）。`GLOBAL.チェイントーク` の再定義で動作を変えられる） | ps:pasta/global.lua、ps:pasta/act.lua `find_act_handler`（L4）・`ACT_IMPL.yield`（`coroutine.yield`） | — |
| L375 単語定義と参照 | 既存収録済み: grammar/words.md | 不要（節見出し） | — |
| L377 概要 | 既存収録済み: grammar/words.md 冒頭本文 | 不要（概要文） | — |
| L381 グローバル単語定義 | 収録先（訂正）: grammar/words.md#グローバル単語定義（参照範囲は全辞書ファイル共通） | gen:elem `generate_global_word` | doc/spec ch10 L3 行と同じ |
| L394 複数キー単語定義 | 既存収録済み: grammar/words.md#複数キー単語定義 | pest:`key_list` | — |
| L410 ローカル単語定義 | 既存収録済み: grammar/words.md#ローカル単語定義 | pest:`global_scene_word_line` | — |
| L422 スコープと優先順位 | 収録先（訂正）: grammar/words.md#スコープと優先順位（統合ではなくローカル優先） | core:word_table.rs `collect_word_candidates` | 食い違い表「ローカル／グローバル候補」 |
| L439 単語の呼び出し（インライン参照） | 収録先（訂正）: grammar/words.md#単語の参照（例の `＠挨拶` はローカル `挨拶朝：Hello` があるためローカル候補だけになる） | core:word_table.rs `collect_word_candidates` | — |
| L457 前方一致検索 | 既存収録済み: grammar/words.md#前方一致検索 | core:word_table.rs | GRAMMAR.md の「全9候補」は 6 候補の誤記 |
| L476 Call文と単語参照の分離 | 収録先（追記）: grammar/words.md#Call との分離（辞書は分離されているが、単語参照も L1・L3・L4 で関数を見つければ呼んで結果を使う） | ps:pasta/act.lua `find_act_handler`（word モードの L2・L5 は単語辞書、scene モードはシーン辞書）、`ACT_IMPL.word`（関数なら呼び出し） | — |
| L491 未定義単語の参照 | 既存収録済み: grammar/words.md#未定義単語の参照 | ps:pasta/act.lua `ACT_IMPL.word`・ps:pasta/actor.lua `PROXY_IMPL.word`（警告して nil → 空出力） | — |
| L500 ＠エスケープ | 既存収録済み: grammar/words.md#＠エスケープ | pest:`at_escape`、gen:elem `generate_action`（Escape） | — |
| L509 引用符エスケープ | 収録先（訂正）: grammar/words.md#引用符エスケープ、grammar/literals.md#単語値での引用符エスケープ（囲みの多重化。`「「セリフ」」` の値は括弧を含まない「セリフ」） | pest:`strfence`（`slfence_ja4`〜`ja1` の順に試す）・`string_contents` | 「実行時に「セリフ」として展開」は誤り →1.3 |
| L522 さくらスクリプト | 既存収録済み: grammar/sakura-script.md 冒頭本文（解釈は areka 層） | pest:`sakura_script` | — |
| L530 字句構造 | 収録先（訂正）: grammar/sakura-script.md#コマンドの字句構造 | pest:`sakura_args` | doc/spec ch07 L15 行と同じ |
| L539 使用例 | 収録先（訂正）: grammar/sakura-script.md#使用例（`\s[a\]b]` の例は `\s[a\]` と本文 `b]` に分かれるので差し替える） | pest:`sakura_body` | →1.3 |
| L554 角括弧内のエスケープ | 収録先（訂正）: grammar/sakura-script.md#角括弧内のエスケープと引用 | pest:`sakura_str` | — |
| L562 Luaコードブロック | 収録先（訂正）: grammar/block-structure.md#Lua ブロックの配置 | pest:`code_block` | — |
| L566 構文 | 収録先（訂正）: grammar/block-structure.md#Lua ブロックの配置（フェンス規則） | pest:`code_open` | 食い違い表「Lua ブロックのフェンス」 |
| L590 制約 | 収録先（訂正）: grammar/block-structure.md#Lua ブロックの配置（「関数定義のみ」は検証されない） | gen:elem `generate_code_block` | →1.3 |
| L596 例 | 既存収録済み: grammar/block-structure.md#Lua ブロックの配置（例） | 不要（例示） | — |
| L617 コメント | 既存収録済み: grammar/block-structure.md#コメント | 不要（節見出し） | — |
| L619 行コメント | 収録先（追記）: grammar/block-structure.md#コメント（行末コメント） | pest:`or_comment_eol` | — |
| L634 リテラル型 | 収録先（訂正）: grammar/literals.md | 不要（節見出し） | — |
| L636 型変換ルール | 収録先（訂正）: grammar/literals.md#型変換ルール | pest:`term`、dsl:parser/ast/action.rs `AttrValue` | doc/spec ch05 L7 行と同じ |
| L648 文字列リテラル | 収録先（訂正）: grammar/literals.md#文字列（String）（代入は `＝`） | pest:`string_literal` | — |
| L662 キューコマンド行 | 収録先（新設）: grammar/block-structure.md#キューコマンド行 | pest:`cue_cmd_line` | — |
| L664 概要 | 収録先（新設）: grammar/block-structure.md#キューコマンド行（`!select` 以外は Lua 生成でスキップ） | gen:scope `generate_local_scene_items` | 食い違い表「キューコマンド」 |
| L670 基本構文 | 収録先（新設）: grammar/block-structure.md#キューコマンド行 | pest:`cue_cmd_line` | — |
| L679 全角/半角対応 | 収録先（新設）: grammar/block-structure.md#キューコマンド行 | pest:`cue_cmd_marker`・`lparen`・`rparen`・`at` | — |
| L690 スコープ指定 | 収録先（新設）: grammar/block-structure.md#キューコマンド行 | pest:`cue_cmd_scope`・`cue_scoped_ident` | — |
| L699 引数 | 収録先（新設）: grammar/block-structure.md#キューコマンド行 | pest:`cue_arg`、dsl:parser/ast/cue.rs `CueArgToken` | — |
| L718 シーン内での使用例 | 収録先（新設）: grammar/block-structure.md#キューコマンド行（例示） | pest:`local_scene_item`（`cue_cmd_line` を含む） | — |
| L735 選択肢定義 | 収録先（新設）: grammar/block-structure.md#選択肢行 | pest:`choice_line` | — |
| L737 概要 | 収録先（新設）: grammar/block-structure.md#選択肢行 | gen:scope `generate_choice` | — |
| L741 基本構文 | 収録先（新設）: grammar/block-structure.md#選択肢行 | pest:`choice_line` | — |
| L743 省略形（表示テキスト＝ターゲット名） | 収録先（新設）: grammar/block-structure.md#選択肢行 | ps:pasta/act.lua `ACT_IMPL.choice`（`display or target`） | — |
| L751 括弧形（表示テキスト指定） | 収録先（新設）: grammar/block-structure.md#選択肢行（表示テキストは `「」` 1 重のみ） | pest:`choice_label = slfence_ja1 ~ …` | — |
| L759 全角/半角対応 | 収録先（新設）: grammar/block-structure.md#選択肢行 | pest:`word_marker`・`question_marker` | — |
| L768 選択肢タイムアウト | 収録先（新設）: grammar/block-structure.md#選択肢行（`!select(秒)`。引数なしは `\![set,choicetimeout,0]`） | gen:scope `generate_choice_timeout`、ps:pasta/shiori/sakura_builder.lua（秒×1000 の ms） | — |
| L778 自動ルーティング | 収録先（新設）: lua/shiori-events.md#OnChoiceSelectEx | ps:pasta/shiori/event/choice_select.lua（明示の `＊OnChoiceSelectEx` 優先 → Reference0 を `STORE.last_global_scene` スコープで前方一致） | — |
| L782 使用例 | 収録先（新設）: grammar/block-structure.md#選択肢行（例示） | 不要（例示） | — |
| L799 エラーハンドリング | 収録先（訂正）: 下位 3 行のとおり各章へ分ける | 不要（節見出し） | — |
| L801 パースエラー | 収録先（追記）: reference/startup.md#4. ゴーストが起動しない・喋らないとき（メッセージ形式 `Parse error: ファイル:行:列: 内容` を追記） | dsl:error.rs（`Parse error: {file}:{line}:{column}: {message}`）、lua:loader/process.rs | GRAMMAR.md の例文 `Expected ':' after speaker name` は現行の文言ではない |
| L809 シーン未発見エラー | 収録先（訂正）: grammar/call-jump.md#スコープ解決アルゴリズム（未発見時は警告ログを出して何もしない） | ps:pasta/act.lua `ACT_IMPL.call`（`act:call - handler not found` を警告し nil） | `LabelNotFound` は現行実装に無い |
| L817 ランタイムエラー | 収録先（訂正）: lua/shiori-events.md#エラーハンドリング（実行時エラーは 500 と `X-Error-Reason`） | ps:pasta/shiori/entry.lua `SHIORI.request`（xpcall → `RES.err`） | `ScriptEvent::Error` は現行実装に無い |
| L823 参考資料 | 除外（doc/spec への権威案内とリポジトリ内リンク（1.5）。スキルへの案内は SKILL.md が担う） | 不要（ナビゲーション） | `.agents/skills` は旧パス |

### `.claude/skills/pasta-ghost-authoring/references/grammar-model.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 文法モデルリファレンス | 除外（スキル内ファイルの題目と自己完結の注記。ファイルは P4 で生成ファイルへ置き換わる） | 不要（題目） | — |
| L8 行指向文法 | 既存収録済み: grammar/index.md#Pasta DSL の文法モデル | pest:各 `*_line` 規則 | — |
| L16 マーカー定義一覧 | 収録先（追記）: grammar/markers.md#マーカー一覧（さくらスクリプト `\` の行を追加。属性の注記を訂正） | pest:各マーカー規則・`sakura_marker` | — |
| L37 空白・改行・コロンの定義 | 既存収録済み: grammar/markers.md#基本要素 | 不要（節見出し） | — |
| L39 改行（NEWLINE） | 既存収録済み: grammar/markers.md#改行（NEWLINE） | pest:`eol` | — |
| L45 空白（WHITE_SPACE 文字クラス） | 収録先（訂正）: grammar/markers.md#空白（WHITE_SPACE）（列挙はこのファイルの記述が実装と一致） | pest:`space_chars` | — |
| L54 コロン | 収録先（訂正）: grammar/markers.md#コロン（`：` / `:`）（「変数代入で使用」を削除） | pest:`set_marker` | — |
| L62 インデント | 既存収録済み: grammar/markers.md#インデント（Indent） | pest:`pad` | — |
| L74 演算子の全角/半角対応表 | 収録先（追記）: grammar/markers.md#算術演算子（`％` がアクター辞書マーカーと同じ文字である注記） | pest:`modulo`・`actor_marker` | — |
| L93 ブロック構造 | 既存収録済み: grammar/block-structure.md#ブロック構造 | 不要（節見出し） | — |
| L95 階層 | 収録先（訂正）: grammar/block-structure.md#ブロック構造（ローカルシーンに属性行は置けない。暗黙開始ブロックには選択肢・キューコマンド・継続行も入る） | pest:`global_scene_scope`・`local_scene_item` | — |
| L112 グローバルシーンブロック | 既存収録済み: grammar/block-structure.md#グローバルシーンの内部構造（訂正は doc/spec ch03 L57 行） | pest:`global_scene_scope` | — |
| L119 暗黙ローカル開始ブロック（`__start__`） | 収録先（追記）: grammar/block-structure.md#グローバルシーンの内部構造（コンテンツ行 0 個でもよい） | pest:`local_start_scene_scope = local_scene_item* ~ code_scope*`、gen:scope `generate_local_scene`（`__start__`） | — |
| L128 Luaブロック配置ルール | 収録先（訂正）: grammar/block-structure.md#Lua ブロックの配置 | pest:`code_scope`、gen:elem `generate_code_block` | →1.3 |
| L148 リテラル型変換ルール | 収録先（訂正）: grammar/literals.md#型変換ルール | pest:`term`、dsl:parser/ast/action.rs `AttrValue` | 真偽値なし →1.3 |
| L165 属性の配置ルール | 収録先（訂正）: grammar/block-structure.md#属性 | pest:`global_scene_attr_line`・`scene` | 食い違い表「属性行の配置」 |
| L184 ファイルレベル属性（将来予約） | 収録先（訂正）: grammar/block-structure.md#属性（現行挙動のみ）。継承は仕分け表（1.4） | lua:transpiler.rs `merge_attrs` | — |
| L197 キューコマンド構文 | 収録先（新設）: grammar/block-structure.md#キューコマンド行 | pest:`cue_cmd_line` | — |
| L206 引数トークン型 | 収録先（新設）: grammar/block-structure.md#キューコマンド行 | pest:`cue_arg`、dsl:parser/ast/cue.rs `CueArgToken` | — |
| L216 全角/半角対応 | 収録先（新設）: grammar/block-structure.md#キューコマンド行 | pest:`cue_cmd_marker`・`lparen`・`at`・`comma_sep`・`colon` | — |
| L220 例 | 収録先（新設）: grammar/block-structure.md#キューコマンド行（例示） | 不要（例示） | — |
| L232 行の種類まとめ | 既存収録済み: grammar/block-structure.md#行の種類 | 不要（節見出し） | — |
| L234 インデント不要 | 収録先（訂正）: grammar/block-structure.md#インデント不要の行（ファイルレベル属性・`＊` 単独行・アクター辞書を追加） | pest:`file_scope`・`actor_line`・`global_scene_continue_line` | — |
| L243 インデント必要 | 収録先（訂正）: grammar/block-structure.md#インデントが必要な行 | pest:`global_scene_init`・`local_scene_item` | — |
| （本文）末尾 `<!-- source: … -->` | 除外（起草元の記録コメント。生成ファイルは生成ヘッダが置き換える） | 不要（コメント） | — |

### `.claude/skills/pasta-ghost-authoring/references/call-spec.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 Call 仕様リファレンス | 除外（スキル内ファイルの題目。ファイルは P4 で削除され call-jump.md の生成物に置き換わる） | 不要（題目） | — |
| L8 基本構文 | 収録先（訂正）: grammar/call-jump.md#Call ターゲットの形式（`＞式` を追加、フィルター付きの行を削除） | pest:`call_scene` | フィルター →1.4 |
| L19 スコープ解決アルゴリズム | 収録先（訂正）: grammar/call-jump.md#スコープ解決アルゴリズム | ps:pasta/act.lua `find_act_handler` | — |
| L23 2段階検索 | 収録先（訂正）: grammar/call-jump.md#スコープ解決アルゴリズム（5 段・ローカル優先） | ps:pasta/act.lua（L1〜L5）、core:scene_table.rs `collect_scene_candidates` | 食い違い表「Call 検索」 |
| L30 前方一致検索 | 既存収録済み: grammar/call-jump.md#前方一致によるターゲット解決 | core:scene_table.rs | — |
| L46 属性フィルター | 除外（構文未受理。仕分け表（1.4）→ scene-attribute-semantics） | pest:`call_scene`・`word_ref` | — |
| L61 特殊 Call | 収録先（新設）: grammar/call-jump.md#特殊な呼び出し | ps:pasta/global.lua、ps:pasta/shiori/entry.lua（`GLOBAL.close_ghost`・`GLOBAL["ゴースト終了"]`） | — |
| L68 チェイントーク（継続トーク） | 収録先（新設）: grammar/call-jump.md#特殊な呼び出し（次回の OnTalk で再開、3 分割以上可、OnBoot などでは後半が出ない） | ps:pasta/global.lua `GLOBAL.yield`、ps:pasta/act.lua `ACT_IMPL.yield`、ps:pasta/shiori/event/virtual_dispatcher.lua `check_talk`（継続確認）、ps:pasta/shiori/event/init.lua `set_co_scene` | — |
| L84 動的ターゲット | 収録先（追記）: grammar/call-jump.md#Call ターゲットの形式 | gen:elem `generate_call_scene`（Dynamic → `tostring(式)`） | — |
| L88 変数参照（代表パターン） | 既存収録済み: grammar/call-jump.md#Call ターゲットの形式（例の `＄target：挨拶朝` は `＝` へ訂正） | pest:`var_ref_local` | — |
| L97 グローバル変数参照 | 収録先（追記）: grammar/call-jump.md#Call ターゲットの形式（`＞＄＊topic`） | pest:`var_ref_global` | — |
| L104 関数呼び出し・式 | 収録先（追記）: grammar/call-jump.md#Call ターゲットの形式（`＞＠get_topic（）`） | pest:`fn_call`、ps:pasta/act.lua `expr_fn` | — |
| L112 nil ガード | 収録先（新設）: grammar/call-jump.md#特殊な呼び出し（任意式の呼び出しと nil の扱い） | ps:pasta/act.lua `ACT_IMPL.call`（`key == nil` で警告して nil）、gen:elem `generate_call_scene`（式を `tostring` して渡す） | 生成コードは `tostring(nil)` で文字列 `"nil"` を渡すため nil ガードに届かず「handler not found」警告になる →1.3 |
| （本文）末尾 `<!-- source: … -->` | 除外（起草元の記録コメント） | 不要（コメント） | — |

### `.claude/skills/pasta-ghost-authoring/references/action-line.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 アクション行リファレンス | 除外（スキル内ファイルの題目。ファイルは P4 で生成物に上書きされる） | 不要（題目） | — |
| L8 基本構文 | 収録先（訂正）: grammar/action-line.md#基本構文・#アクターの省略（アクター名は識別子） | pest:`action_line`・`continue_action_line` | — |
| L20 インライン要素一覧 | 収録先（訂正）: grammar/action-line.md#インライン要素（`＠＄` を削除、`＄％` 等を追加） | pest:`action` | — |
| L38 インライン判定ルール | 収録先（訂正）: grammar/action-line.md#インライン要素の区切り（判定順は `＠＠`・`＄＄`・`\\` → 関数呼び出し → 単語参照 → 変数参照 → さくらスクリプト → テキスト。`＠＄` 段は無い） | pest:`action` の選択順 | — |
| L53 識別子定義（Identifier） | 既存収録済み: grammar/markers.md#識別子（Identifier） | pest:`id` | — |
| L68 インライン要素の区切り文字 | 既存収録済み: grammar/action-line.md#インライン要素の区切り | 不要（節見出し） | — |
| L70 空白による区切り | 既存収録済み: grammar/action-line.md#空白による区切り | pest:`word_ref`・`var_ref_*`（末尾 `s`） | — |
| L80 空白なしの場合（最長一致） | 既存収録済み: grammar/action-line.md#空白がない場合（最長一致） | pest:`id` | — |
| L92 意図しない吸収の例 | 既存収録済み: grammar/action-line.md#空白がない場合（最長一致） | pest:`id`（日本語文字は XID_CONTINUE） | — |
| L106 ＠＠エスケープ | 既存収録済み: grammar/words.md#＠エスケープ | pest:`at_escape` | — |
| L112 行継続（Continuation Line） | 収録先（訂正）: grammar/action-line.md#行継続 | pest:`continue_action_line` | 食い違い表「行継続」 |
| L131 改行セマンティクス | 収録先（訂正）: grammar/action-line.md#改行（継続中の空行は何も出力しない） | pest:`blank_line` | 食い違い表「継続内の空行」 |
| （本文）末尾 `<!-- source: … -->` | 除外（起草元の記録コメント） | 不要（コメント） | — |

### `.claude/skills/pasta-ghost-authoring/references/sakura-script.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 さくらスクリプトリファレンス | 除外（スキル内ファイルの題目） | 不要（題目） | — |
| L8 透過ルール | 既存収録済み: grammar/sakura-script.md 冒頭本文 | pest:`sakura_script` | — |
| L14 エスケープ文字 | 既存収録済み: grammar/sakura-script.md#エスケープ文字 | pest:`sakura_marker` | — |
| L20 タグ一覧 | 収録先（新設）: grammar/sakura-script.md#主要タグ早見 | pest:`sakura_id`（どのタグも字句的に受理。意味はベースウェア仕様） | — |
| L22 よく使用するタグ | 収録先（新設）: grammar/sakura-script.md#主要タグ早見 | pest:`sakura_id`。`\w数字` の 50ms 単位などはベースウェア仕様（ukadoc） | — |
| L31 その他のタグ（透過対応） | 収録先（新設）: grammar/sakura-script.md#主要タグ早見 | pest:`sakura_id`、lua:sakura_script/tokenizer.rs（台詞中のタグを保護） | pasta 自身も立ち位置切替で `\p[ID]` を出す（ps:pasta/shiori/sakura_builder.lua） |
| L44 字句構造 | 収録先（訂正）: grammar/sakura-script.md#コマンドの字句構造（`\]` を削除） | pest:`sakura_args` | →1.3 |
| L59 配置ルール | 収録先（訂正）: grammar/sakura-script.md#配置ルール（単語値にも書ける。アクター辞書が依存） | pest:`word = string_literal or sakura_script or word_nofenced`、lua:sakura_script/tokenizer.rs | 食い違い表「単語定義とさくらスクリプト」 |
| L71 複雑なさくらスクリプトの記述 | 既存収録済み: grammar/sakura-script.md#複雑なケースの方針 | gen:elem `generate_action`（FnCall → `act.アクター:talk((act.アクター:expr_fn(…)))`） | — |
| （本文）末尾 `<!-- source: … -->` | 除外（起草元の記録コメント） | 不要（コメント） | — |

### `.claude/skills/pasta-ghost-authoring/references/variables.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 変数リファレンス | 除外（スキル内ファイルの題目と、別スキルへのクロスリファレンス注記） | 不要（題目） | — |
| L8 変数スコープ | 収録先（追記）: grammar/variables.md#変数の種類とスコープ（グローバル変数は save 経由で JSON に永続化） | gen:elem `resolve_var_path`、lua:runtime/persistence.rs | — |
| L27 代入構文 | 収録先（訂正）: grammar/variables.md#変数の種類とスコープ（コロン形は受理されない） | pest:`set_marker = equals` | →1.3 |
| L36 使用可能な値の型 | 収録先（訂正）: grammar/variables.md#式（Expression）のサポート（`＠＊` は GLOBAL 関数。引数例を読点区切りへ） | gen:elem `generate_expr_to_buffer`、ps:pasta/global.lua | — |
| L52 式文構文（var_set_none） | 既存収録済み: grammar/variables.md#式文（ExprStmt） | pest:`var_set_none` | — |
| L67 式サポート | 既存収録済み: grammar/variables.md#式（Expression）のサポート、grammar/markers.md#算術演算子 | pest:`bin_op` | — |
| L89 変数宣言行 vs 変数参照 | 既存収録済み: grammar/action-line.md#インライン要素（重要注記） | pest:`action`（`var_set` を含まない） | — |
| L100 アクション行内の区切りルール | 既存収録済み: grammar/variables.md#変数参照の空白ルール | pest:`var_ref_*` | — |
| L113 自動設定される日時変数（onhour-date-var-transfer） | 既存収録済み: grammar/variables.md#日時変数 | ps:pasta/shiori/act.lua `transfer_date_to_var`、ps:pasta/shiori/event/virtual_dispatcher.lua `check_hour`（OnHour 発火時に転記） | — |
| L119 数値型（英語キー） | 既存収録済み: grammar/variables.md#日時変数 | ps:pasta/shiori/act.lua `transfer_date_to_var`（`DATE_NUM_FIELDS`） | — |
| L132 文字列型（日本語キー） | 既存収録済み: grammar/variables.md#日時変数 | ps:pasta/shiori/act.lua `transfer_date_to_var`（`DATE_JA_FIELDS`） | — |
| L145 `＄時１２` の変換ルール | 既存収録済み: grammar/variables.md#日時変数 | ps:pasta/shiori/act.lua `to_12hour_format` | — |
| L154 使用上の注意 | 収録先（追記）: grammar/variables.md#日時変数（数値キーは Lua での条件分岐に使う） | ps:pasta/shiori/act.lua（英語キーは数値のまま転記） | — |
| L162 リクエスト変数（Reference）（req-var-expansion） | 収録先（追記）: grammar/variables.md#リクエスト変数（Reference）（`＄０`（シーン引数）と `＄ｒ０` は別物、の注記） | ps:pasta/shiori/act.lua `transfer_req_to_var`、gen:elem `resolve_var_path`（`VarScope::Args` → `args[n+1]`） | `＄０` →1.2 |
| L185 永続化とSAVEテーブル | 収録先（新設）: grammar/variables.md#グローバル変数の保存先（収録先表「`＄＊` の保存先・DSL→Lua 対応表」） | gen:elem `resolve_var_path`、ps:pasta/save.lua | — |
| L187 永続化メカニズム | 収録先（新設）: grammar/variables.md#グローバル変数の保存先（DSL→Lua 対応表、既定の保存先 `profile/pasta/save/save.json`、ランタイム終了時に保存） | lua:runtime/lifecycle.rs（Drop で `save_to_file`）、lua:loader/config/sections.rs `default_persistence_file_path` | 別スキルへの案内（`runtime-api.md`・`internal-modules.md`）は lua/modules/pasta-persistence.md へのリンクに置き換える |
| L205 エンジン予約キー（`pasta_` プレフィックス） | 既存収録済み: grammar/variables.md#予約グローバル変数（pasta_ で始まる名前） | ps:pasta/shiori/event/virtual_dispatcher.lua `get_config`（SAVE → `[ghost]` → 既定、10 秒未満は 10、min>max は max を min に） | — |
| （本文）末尾 `<!-- source: … -->` | 除外（起草元の記録コメント） | 不要（コメント） | — |

### `.claude/skills/pasta-ghost-authoring/references/words.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 単語定義リファレンス | 除外（スキル内ファイルの題目） | 不要（題目） | — |
| L8 単語定義構文 | 既存収録済み: grammar/words.md#単語の定義 | 不要（節見出し） | — |
| L10 単一キー | 既存収録済み: grammar/words.md#グローバル単語定義 | pest:`key_words` | — |
| L16 複数キー | 既存収録済み: grammar/words.md#複数キー単語定義 | pest:`key_list` | — |
| L27 値区切り文字 | 既存収録済み: grammar/markers.md#単語値の区切り文字 | pest:`comma` | — |
| L36 スコープ | 収録先（追記）: grammar/words.md#スコープと優先順位（アクタースコープの行を表に加える） | gen:elem `generate_global_word`・`generate_local_word`、gen:scope `generate_actor` | — |
| L53 単語参照 | 収録先（追記）: grammar/words.md#単語の参照（シャッフル＆順次消費） | core:word_table.rs `search_word`（キャッシュ） | — |
| L65 動的単語参照 | 除外（未実装。grammar/words.md#動的単語参照（将来変更あり）は節ごと削除し、仕分け表（1.4）→ dynamic-word-reference） | pest:`word_ref`（`＠＄` の規則なし） | — |
| L76 スコープ解決アルゴリズム | 収録先（訂正）: grammar/words.md#スコープと優先順位（ローカル優先・統合しない・5 段） | core:word_table.rs `collect_word_candidates`、ps:pasta/act.lua `find_act_handler` | 食い違い表「ローカル／グローバル候補」 |
| （本文）末尾 `<!-- source: … -->` | 除外（起草元の記録コメント） | 不要（コメント） | — |

### `.claude/skills/pasta-ghost-authoring/references/actor-dictionary.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 アクター辞書リファレンス | 除外（スキル内ファイルの題目） | 不要（題目） | — |
| L8 アクター辞書定義 | 既存収録済み: grammar/actor-dictionary.md#グローバルアクター辞書定義 | pest:`actor_scope` | — |
| L20 定義例 | 既存収録済み: grammar/actor-dictionary.md#グローバルアクター辞書定義（例） | 不要（例示） | — |
| L38 シーン内でのスコープ指定 | 収録先（訂正）: grammar/actor-dictionary.md#シーンスコープ内でのアクター指定（立ち位置の設定。OnBoot で一度設定して固定する使い方と、シーンごとの切替） | gen:scope `generate_local_scene`（`set_spot`）、ps:pasta/act.lua `set_spot`・`clear_spot`、ps:pasta/shiori/sakura_builder.lua（`\p[ID]`） | 立ち位置がイベントをまたいで保たれるか →1.3 |
| L55 フォールバック検索順 | 収録先（訂正）: grammar/actor-dictionary.md#アクタースコープと単語参照の統合（3 段ではなく A1・A2・L1〜L5） | ps:pasta/actor.lua `find_actor_handler`、ps:pasta/act.lua `find_act_handler` | — |
| L69 バルーン連動 | 収録先（新設）: grammar/actor-dictionary.md#バルーン連携、reference/pasta-toml.md#spot | ps:pasta/shiori/sakura_builder.lua `spot_to_tag`、lua:runtime/module_registry.rs `inject_actor_names` | — |
| L87 完全なコード例 | 収録先（新設）: grammar/actor-dictionary.md#バルーン連携（例示） | 不要（例示） | — |
| （本文）末尾 `<!-- source: … -->` | 除外（起草元の記録コメント） | 不要（コメント） | — |

### `.claude/skills/pasta-ghost-authoring/references/pasta-toml.md`

既定値は同一行の表形式で書く（`config_defaults_test.rs` が照合する）。マニュアルにはリポジトリ内パスを書かない（5.3）。

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 pasta.toml リファレンス | 収録先（新設）: reference/pasta-toml.md | 不要（題目） | — |
| L8 概要 | 収録先（新設）: reference/pasta-toml.md#概要 | lua:loader/config/mod.rs（serde の default 補完・`apply_shiori_defaults`） | — |
| L15 プロファイルと3分類 | 収録先（新設）: reference/pasta-toml.md#プロファイルと3分類 | lua:loader/config/mod.rs（`[actor]` 不在時の警告 1 回） | — |
| L27 3分類表 | 収録先（新設）: reference/pasta-toml.md#3分類表 | lua:loader/config/sections.rs `default_*`、lua:loader/config/mod.rs `default_pasta_patterns` ほか、crates/pasta_lua/tests/loader/config_defaults_test.rs | 既定値の出典 `crates/pasta_lua/src/loader/config.rs` は旧位置。マニュアルには「実装の既定値（SSOT）」とだけ書く |
| L57 最小テンプレート | 収録先（新設）: reference/pasta-toml.md#最小テンプレート | lua:loader/config/mod.rs（`[actor]` 不在は警告のみで停止しない） | — |
| L78 フルリファレンステンプレート | 収録先（新設）: reference/pasta-toml.md#フルリファレンステンプレート | lua:loader/config/sections.rs・mod.rs の既定値 | — |
| L161 [package] 予約注記 | 収録先（新設）: reference/pasta-toml.md#[package] 予約注記（アンカーは見出しの slug に合わせて直す） | lua:loader/config/mod.rs（`[package]` は custom_fields に残るだけで使われない） | 吸収元の `#package予約注記` は切れたアンカー |
| L178 各セクション詳細 | 収録先（新設）: reference/pasta-toml.md#各セクション詳細 | 不要（節見出し） | — |
| L182 [loader]（ファイル読み込み） | 収録先（新設）: reference/pasta-toml.md#[loader]（ファイル読み込み） | lua:loader/config/mod.rs `LoaderConfig` | — |
| L193 pasta_patterns | 収録先（新設）: reference/pasta-toml.md#pasta_patterns（authoring-patterns §6.8 の自動読み込みを統合） | lua:loader/config/mod.rs `default_pasta_patterns`（`dic/**/*.pasta`） | — |
| L202 lua_search_paths | 収録先（新設）: reference/pasta-toml.md#lua_search_paths（reference/startup.md#1. モジュール検索パス と整合させる） | lua:loader/config/mod.rs `default_lua_search_paths` | — |
| L214 [ghost]（ゴースト動作） | 収録先（新設）: reference/pasta-toml.md#[ghost]（ゴースト動作） | lua:loader/config/sections.rs `GhostConfig` | — |
| L225 talk_interval_min / talk_interval_max | 収録先（新設）: reference/pasta-toml.md#talk_interval_min / talk_interval_max | lua:loader/config/sections.rs `default_talk_interval_min`・`_max`、ps:pasta/shiori/event/virtual_dispatcher.lua `get_config` | — |
| L235 spot_newlines | 収録先（新設）: reference/pasta-toml.md#spot_newlines（`\n[half]` 相当という説明は、実装の `\n[150]`（値×100 の百分率）へ訂正） | lua:loader/config/sections.rs `default_spot_newlines`、ps:pasta/shiori/sakura_builder.lua（`\n[%d]`、`spot_newlines * 100`） | — |
| L246 [actor."名前"]（アクター設定） | 収録先（新設）: reference/pasta-toml.md#[actor."名前"]（アクター設定） | lua:runtime/module_registry.rs `inject_actor_names`、ps:pasta/actor.lua（CONFIG 由来アクター） | — |
| L258 spot | 収録先（新設）: reference/pasta-toml.md#spot | ps:pasta/shiori/sakura_builder.lua `spot_to_tag` | — |
| L270 budoux | 収録先（新設）: reference/pasta-toml.md#budoux | lua:sakura_script/mod.rs `apply_budoux_if_configured`、lua:sakura_script/line_breaker.rs | — |
| L290 default_surface | 収録先（新設）: reference/pasta-toml.md#default_surface（実装の挙動に合わせて記述） | crates 配下で `default_surface` を読むコードが見当たらない（fixture と README のみ） | 「シーン開始時に自動適用」は実装で確認できない →1.3 |
| L306 [talk]（トーク表示制御） | 収録先（新設）: reference/pasta-toml.md#[talk]（トーク表示制御） | lua:loader/config/sections.rs `TalkConfig`、lua:sakura_script/wait_inserter.rs | — |
| L335 [persistence]（永続化） | 収録先（新設）: reference/pasta-toml.md#[persistence]（永続化） | lua:loader/config/sections.rs `PersistenceConfig`、lua:runtime/persistence.rs | — |
| L353 [logging]（ログ出力） | 収録先（新設）: reference/pasta-toml.md#[logging]（ログ出力） | lua:loader/config/sections.rs `LoggingConfig` | — |
| L372 [lua]（Lua ライブラリ）★ 上級者向け | 収録先（新設）: reference/pasta-toml.md#[lua]（Lua ライブラリ）（スキルへの案内は lua/modules/mlua-stdlib.md へのリンクに置き換える） | lua:loader/config/sections.rs `LuaConfig`・`default_libs`、lua:runtime/runtime_config.rs `from_libs` | — |
| L387 [debug]（デバッグバックエンド）★ 上級者向け | 収録先（新設）: reference/pasta-toml.md#[debug]（デバッグバックエンド） | lua:loader/config/sections.rs `DebugFileConfig`・`default_debug_port` | — |
| L407 [package]（パッケージ情報）★ エンジンプロファイル専用 | 収録先（新設）: reference/pasta-toml.md#[package]（パッケージ情報） | lua:loader/config/mod.rs（使われない） | — |

### `.claude/skills/pasta-ghost-authoring/references/authoring-patterns.md`

挙動の事実だけを収録先へ移す。作例・手順は手書きファイルに残る（SkillLayout）。

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 辞書制作パターン集 | 除外（作例集の題目。ファイルは手書きとして残る） | 不要（題目） | — |
| L9 6.1 アクター辞書定義（actors.pasta） | 除外（作例。規範は grammar/actor-dictionary.md が既に持つ） | 不要（作例） | — |
| L27 6.2 イベントハンドラ（boot.pasta） | 挙動事実 `＞ゴースト終了（ms）` → 収録先（新設）: grammar/call-jump.md#特殊な呼び出し。残りは作例として除外 | ps:pasta/shiori/entry.lua `GLOBAL.close_ghost`（ms≥1 なら待ってから `\-`） | — |
| L49 6.3 ランダムトーク（talk.pasta） | 挙動事実（同名 `＊OnTalk` を複数定義するとシャッフル＆順次消費で選ばれる）→ 既存収録済み: grammar/block-structure.md#グローバルシーン、収録先（新設）: lua/shiori-events.md#OnTalk。作例は除外 | ps:pasta/shiori/event/virtual_dispatcher.lua `check_talk`、core:scene_table.rs | — |
| L67 6.4 時報（hour.pasta / talk.pasta） | 挙動事実（4 段フォールバック）→ 収録先（新設）: lua/shiori-events.md#OnHour | ps:pasta/shiori/event/virtual_dispatcher.lua `check_hour`（候補 `時報HH`・`OnHourHH`・`時報その他`・`OnHourOther`） | — |
| L71 自動設定される日時変数 | 既存収録済み: grammar/variables.md#日時変数（収録先表: authoring-patterns §6.4 → variables.md） | ps:pasta/shiori/act.lua `transfer_date_to_var` | — |
| L117 時報パターン例 | 除外（作例） | 不要（作例） | — |
| L133 日時変数を活用した応用例 | 除外（作例） | 不要（作例） | 作例の `％` 行が Lua ブロックの後にあり pest:`global_scene_scope` の順序に反する。作例の修正は手書きファイル更新時に行う |
| （本文）L159 6.5 クリック反応（click.pasta） | 除外（作例。直前の入れ子フェンスの誤りでコードブロック内に埋もれている） | 不要（作例） | — |
| L169 6.6 単語ランダム選択（シャッフル＆順次消費方式） | 収録先（新設）: grammar/words.md#シャッフル＆順次消費（同名シーンへの適用は grammar/call-jump.md#スコープ解決アルゴリズム） | core:word_table.rs `search_word`（キャッシュ）、core:random.rs、core:scene_table.rs | — |
| L205 6.7 継続トーク（チェイントーク） | 収録先（新設）: grammar/call-jump.md#特殊な呼び出し | ps:pasta/global.lua、ps:pasta/shiori/event/virtual_dispatcher.lua `check_talk` | — |
| L238 6.8 ファイル分割ガイド | 挙動事実（`pasta_patterns` による自動読み込み）→ 収録先（新設）: reference/pasta-toml.md#pasta_patterns。分割の指針は除外（手順） | lua:loader/config/mod.rs `default_pasta_patterns` | 本文の `["dic/*.pasta"]` は既定値 `["dic/**/*.pasta"]` と異なる |
| L250 6.9 自然言語→シーン変換指針 | 除外（AI 作業手順。手書きとして残る） | 不要（手順） | — |
| L272 6.10 複数キー単語定義（マルチキー） | 既存収録済み: grammar/words.md#複数キー単語定義。アクター辞書内の例 → 収録先（追記）: grammar/actor-dictionary.md#グローバルアクター辞書定義 | gen:elem `generate_word_definition`、gen:scope `generate_actor`（キーごとに登録） | — |
| L320 6.11 選択肢メニュー（Choice Menu） | 構文部 → 収録先（新設）: grammar/block-structure.md#選択肢行。ルーティング部 → 収録先（新設）: lua/shiori-events.md#OnChoiceSelectEx | gen:scope `generate_choice`、ps:pasta/shiori/sakura_builder.lua（`\![*]\q[表示,ID]`・`\![set,choicetimeout,ms]`）、ps:pasta/shiori/event/choice_select.lua | — |

### `.claude/skills/pasta-lua-coding/references/runtime-api.md`

章名は require 名から `@` を除き `_` を `-` にしたもの（lua/modules/ 配下）。

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 Runtime API リファレンス | 収録先（新設）: lua/modules/index.md（対象範囲の説明） | 不要（題目） | — |
| L8 @pasta_search | 収録先（新設）: lua/modules/pasta-search.md | lua:search/mod.rs `register`、lua:search/context.rs | — |
| L18 search_scene(name, global_scene_name?) | 収録先（新設）: lua/modules/pasta-search.md#search_scene(name, global_scene_name?) | lua:search/context.rs（`search_scene` メソッド） | — |
| L61 search_word(name, global_scene_name?) | 収録先（新設）: lua/modules/pasta-search.md#search_word(name, global_scene_name?) | lua:search/context.rs（`search_word` メソッド） | — |
| L87 set_scene_selector(...) / set_word_selector(...) | 収録先（新設）: lua/modules/pasta-search.md#set_scene_selector(...) / set_word_selector(...)（見出しをこの形で保つ。スキル側アンカー互換） | lua:search/context.rs `set_scene_selector`・`set_word_selector` | — |
| （本文）L114 testing-lint.md#決定論的テスト へのリンク | 除外（スキル内ナビゲーション。SKILL.md と手書き→生成のリンクが担う） | 不要（ナビゲーション） | — |
| L118 @pasta_persistence | 収録先（新設）: lua/modules/pasta-persistence.md | lua:runtime/persistence.rs（`_VERSION`・`_DESCRIPTION`） | — |
| L128 load() | 収録先（新設）: lua/modules/pasta-persistence.md#load() | lua:runtime/persistence.rs `load_impl` | — |
| L149 save(data) | 収録先（新設）: lua/modules/pasta-persistence.md#save(data) | lua:runtime/persistence.rs `save_impl` | — |
| L176 pasta.toml設定 | 収録先（新設）: lua/modules/pasta-persistence.md#pasta.toml 設定（詳細は reference/pasta-toml.md へリンク） | lua:loader/config/sections.rs `PersistenceConfig` | — |
| L195 @pasta_config | 収録先（新設）: lua/modules/pasta-config.md | lua:runtime/module_registry.rs `register_config_module` | — |
| L205 公開されるフィールド | 収録先（新設）: lua/modules/pasta-config.md#公開されるフィールド | lua:loader/config/mod.rs（`[loader]` を取り除いた残りが custom_fields）、lua:runtime/module_registry.rs `toml_to_lua` | — |
| L228 アクセス例 | 収録先（新設）: lua/modules/pasta-config.md#アクセス例 | 不要（例示） | — |
| L245 注意事項 | 収録先（新設）: lua/modules/pasta-config.md#注意事項（型の対応: 整数も Lua の数値、日時は文字列） | lua:runtime/module_registry.rs `toml_to_lua` | 「読み取り専用」は普通のテーブルで強制されていない →1.3 |
| L254 @pasta_sakura_script | 収録先（新設）: lua/modules/pasta-sakura-script.md | lua:sakura_script/mod.rs `register` | — |
| L262 talk_to_script(actor, talk) | 収録先（新設）: lua/modules/pasta-sakura-script.md#talk_to_script(actor, talk) | lua:sakura_script/mod.rs `talk_to_script_impl` | — |
| L273 actor.talkテーブルの全フィールド | 収録先（訂正）: lua/modules/pasta-sakura-script.md#アクター表のウェイト設定（アクター表直下の `script_wait_normal`・`_period`・`_comma`・`_strong`・`_leader`。既定 50・1000・500・500・200） | lua:sakura_script/mod.rs `resolve_wait_values`、lua:sakura_script/wait_inserter.rs `WaitValues`、lua:loader/config/sections.rs `TalkConfig` | 食い違い表「さくらスクリプト変換のウェイト」 |
| L289 動作仕様 | 収録先（訂正）: lua/modules/pasta-sakura-script.md#動作仕様（挿入値は `値 - 50`、連続する句読点は最大値） | lua:sakura_script/wait_inserter.rs `insert_waits`（`max`）、lua:sakura_script/tokenizer.rs | 食い違い表「さくらスクリプト変換のウェイト」 |
| L297 使用例 | 収録先（訂正）: lua/modules/pasta-sakura-script.md#使用例（出力例を実装の値で書き直す） | lua:sakura_script/wait_inserter.rs のテスト（`test_requirement_7_*`） | — |
| L326 pasta.toml での設定 | 収録先（訂正）: lua/modules/pasta-sakura-script.md#pasta.toml での設定（`[talk]` のキー名を実装どおりに） | lua:loader/config/sections.rs `TalkConfig` | — |
| L337 break_lines(text, widths) | 収録先（新設）: lua/modules/pasta-sakura-script.md#break_lines(text, widths) | lua:sakura_script/mod.rs `break_lines_lua_impl`、lua:sakura_script/line_breaker.rs | — |
| L359 pasta.toml での budoux 設定 | 収録先（新設）: lua/modules/pasta-sakura-script.md#pasta.toml での budoux 設定 | lua:sakura_script/mod.rs `apply_budoux_if_configured` | — |
| L370 使用例（直接呼び出し） | 収録先（新設）: lua/modules/pasta-sakura-script.md#使用例（直接呼び出し） | lua:sakura_script/line_breaker.rs | — |
| L399 @enc | 収録先（新設）: lua/modules/enc.md | lua:runtime/enc.rs | — |
| L411 to_ansi(utf8_str) | 収録先（新設）: lua/modules/enc.md#to_ansi(utf8_str) | lua:runtime/enc.rs `to_ansi_impl` | — |
| L427 to_utf8(ansi_str) | 収録先（新設）: lua/modules/enc.md#to_utf8(ansi_str) | lua:runtime/enc.rs `to_utf8_impl` | — |
| L439 戻り値パターン | 収録先（新設）: lua/modules/enc.md#戻り値パターン | lua:runtime/enc.rs | — |
| L464 @pasta_log | 収録先（新設）: lua/modules/pasta-log.md | lua:runtime/log.rs、lua:runtime/mod.rs（libs に関係なく常に登録） | — |
| L476 trace(value) | 収録先（新設）: lua/modules/pasta-log.md#trace(value) | lua:runtime/log.rs `log_trace` | — |
| L488 debug(value) | 収録先（新設）: lua/modules/pasta-log.md#debug(value) | lua:runtime/log.rs `log_debug` | — |
| L500 info(value) | 収録先（新設）: lua/modules/pasta-log.md#info(value) | lua:runtime/log.rs `log_info` | — |
| L512 warn(value) | 収録先（新設）: lua/modules/pasta-log.md#warn(value) | lua:runtime/log.rs `log_warn` | — |
| L524 error(value) | 収録先（新設）: lua/modules/pasta-log.md#error(value) | lua:runtime/log.rs `log_error` | — |
| L536 値の変換規則 | 収録先（新設）: lua/modules/pasta-log.md#値の変換規則 | lua:runtime/log.rs | — |
| L550 構造化ログフィールド | 収録先（新設）: lua/modules/pasta-log.md#構造化ログフィールド | lua:runtime/log.rs（`lua_source`・`lua_line`・`lua_fn`） | — |
| L560 使用例 | 収録先（新設）: lua/modules/pasta-log.md#使用例 | 不要（例示） | — |
| L583 mlua-stdlib 統合モジュール | 収録先（新設）: lua/modules/mlua-stdlib.md | lua:runtime/runtime_config.rs | — |
| L587 デフォルトで有効なモジュール | 収録先（新設）: lua/modules/mlua-stdlib.md#デフォルトで有効なモジュール | lua:loader/config/sections.rs `default_libs`（`std_all`・`assertions`・`testing`・`regex`・`json`・`yaml`） | — |
| L589 @json | 収録先（新設）: lua/modules/mlua-stdlib.md#@json | lua:runtime/runtime_config.rs `should_enable_module` | — |
| L600 @yaml | 収録先（新設）: lua/modules/mlua-stdlib.md#@yaml | lua:runtime/runtime_config.rs | — |
| L611 @regex | 収録先（新設）: lua/modules/mlua-stdlib.md#@regex | lua:runtime/runtime_config.rs | — |
| L622 @assertions | 収録先（新設）: lua/modules/mlua-stdlib.md#@assertions | lua:runtime/runtime_config.rs | — |
| L635 @testing | 収録先（新設）: lua/modules/mlua-stdlib.md#@testing | lua:runtime/runtime_config.rs | — |
| L649 デフォルトで無効なモジュール | 収録先（新設）: lua/modules/mlua-stdlib.md#デフォルトで無効なモジュール | lua:runtime/runtime_config.rs | — |
| L651 @env | 収録先（新設）: lua/modules/mlua-stdlib.md#@env（ゴースト作者向けの有効化手段は pasta.toml `[lua] libs` への `"env"` 追加として書く） | lua:runtime/runtime_config.rs（`"env"`）、lua:loader/config/sections.rs `LuaConfig` | Rust の `RuntimeConfig` コード例は利用者が操作できない →1.3 |
| L681 RuntimeConfig によるモジュール制御 | 収録先（訂正）: lua/modules/mlua-stdlib.md#libs によるモジュール制御（エントリで有効化、`-` 接頭辞で除外）。Rust のコンストラクタ（`new`・`full`・`minimal`）は除外（Rust 埋め込み API で利用者の操作対象外） | lua:runtime/runtime_config.rs `new`・`full`・`minimal`・`from_libs` | — |
| L707 関連リファレンス | 除外（スキル内ナビゲーション。SKILL.md と手書き→生成のリンクが担う） | 不要（ナビゲーション） | — |

### `.claude/skills/pasta-lua-coding/references/shiori-handlers.md`

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 SHIORI Handlers リファレンス | 収録先（新設）: lua/shiori-events.md（冒頭） | 不要（題目） | — |
| L8 REG テーブル登録 | 収録先（新設）: lua/shiori-events.md#REG | ps:pasta/shiori/event/register.lua | — |
| L17 登録パターン | 収録先（訂正）: lua/shiori-events.md#REG（ハンドラは `function(act)`。戻り値は文字列・コルーチン・nil） | ps:pasta/shiori/event/register.lua（シグネチャ注記）、ps:pasta/shiori/event/init.lua `EVENT.fire`（`handler(act)` の戻り値を thread・string・nil で分岐） | 食い違い表「REG ハンドラ」。文字列を返すと `EVENT.fire` が `RES.ok` で包むため、ハンドラが `RES.ok(…)` を返すと二重になる疑い →1.3 |
| L26 req パラメータ | 収録先（訂正）: lua/shiori-events.md#act.req（`act.req.id`・`method`・`version`・`charset`・`sender`・`reference[N]`・`dic`・`date`・`status`） | ps:pasta/shiori/event/register.lua（`act.req` の構造注記）、ps:pasta/shiori/entry.lua `SHIORI.request` 注記、ps:pasta/shiori/event/virtual_dispatcher.lua（`act.req.date`・`act.req.status` を使用） | — |
| L50 RES レスポンス生成 | 収録先（新設）: lua/shiori-events.md#RES | ps:pasta/shiori/res.lua | — |
| L58 API一覧 | 収録先（訂正）: lua/shiori-events.md#RES（`RES.ok(value, dic)`（空値は 204）・`no_content`・`not_enough`・`advice`・`bad_request`・`err`（`X-Error-Reason`）・`warn`（`X-Warn-Reason`）・`build`・`env`。`ok_with` は無い） | ps:pasta/shiori/res.lua | 食い違い表「RES」 |
| L67 使用例 | 収録先（訂正）: lua/shiori-events.md#RES（`ok_with` の例を `RES.ok(value, {Reference0 = …})` へ） | ps:pasta/shiori/res.lua `RES.ok(value, dic)` | — |
| L88 主要SHIORIイベント一覧 | 収録先（新設）: lua/shiori-events.md#主要イベント | 不要（節見出し。Reference の意味はベースウェア仕様（ukadoc）） | — |
| L92 起動・終了系 | 収録先（新設）: lua/shiori-events.md#主要イベント | 不要（節見出し） | — |
| L94 OnFirstBoot — 初回起動 | 収録先（新設）: lua/shiori-events.md#OnFirstBoot（例のハンドラを `function(act)`・`act.req.reference[0]` へ） | ps:pasta/shiori/event/init.lua `EVENT.no_entry`（既定ハンドラなし → 同名シーン） | — |
| L112 OnBoot — 通常起動 | 収録先（新設）: lua/shiori-events.md#OnBoot（既定ハンドラが `＊OnBoot` シーンを実行） | ps:pasta/shiori/event/boot.lua | — |
| L129 OnClose — 終了 | 収録先（新設）: lua/shiori-events.md#OnClose | ps:pasta/shiori/event/init.lua `EVENT.no_entry` | — |
| L147 OnGhostChanged — ゴースト切り替え | 収録先（新設）: lua/shiori-events.md#OnGhostChanged | ps:pasta/shiori/event/init.lua `EVENT.no_entry` | — |
| L163 選択肢系 | 収録先（新設）: lua/shiori-events.md#主要イベント | 不要（節見出し） | — |
| L165 OnChoiceSelectEx — 選択肢選択 | 収録先（新設）: lua/shiori-events.md#OnChoiceSelectEx（authoring-patterns §6.11 のルーティングと統合） | ps:pasta/shiori/event/choice_select.lua（明示シーン優先 → Reference0 で前方一致 → 無ければ 204） | — |
| L189 マウス操作系 | 収録先（新設）: lua/shiori-events.md#主要イベント | 不要（節見出し） | — |
| L191 OnMouseDoubleClick — ダブルクリック | 収録先（新設）: lua/shiori-events.md#OnMouseDoubleClick | ps:pasta/shiori/event/init.lua `EVENT.no_entry` | — |
| L212 時間系 | 収録先（新設）: lua/shiori-events.md#主要イベント | 不要（節見出し） | — |
| L214 OnSecondChange — 毎秒 | 収録先（訂正）: lua/shiori-events.md#OnSecondChange（既定ハンドラが `CALLBACK.sweep` と仮想ディスパッチャを動かす。上書きすると OnTalk・OnHour・コールバックのタイムアウト処理が止まる） | ps:pasta/shiori/event/second_change.lua、ps:pasta/shiori/event/callback.lua `sweep` | 食い違い表「OnSecondChange・コールバック」（`CALLBACK.resume_pending` は無い） |
| L230 OnNotifyCallbackResponse — SSPコールバック応答 | 収録先（訂正）: lua/shiori-events.md#OnNotifyCallbackResponse（応答は `EVENT.fire` 冒頭の `CALLBACK.try_route` が REG より先に処理する。モジュール名は `pasta.shiori.event.callback`） | ps:pasta/shiori/event/init.lua `EVENT.fire`（最初に `CALLBACK.try_route(req)`）、ps:pasta/shiori/event/callback.lua `try_route` | 「event.init で REG に自動登録」「独自ハンドラで機構が止まる」は実装と合わない →1.3 |
| L243 OnMinuteChange — 毎分 | 収録先（新設）: lua/shiori-events.md#OnMinuteChange | ps:pasta/shiori/event/init.lua `EVENT.no_entry` | — |
| L265 シーン関数フォールバック | 収録先（新設）: lua/shiori-events.md#シーン関数フォールバック | ps:pasta/shiori/event/init.lua `EVENT.no_entry`（`SCENE.co_exec(act, req.id)`） | — |
| L269 フォールバックチェーン | 収録先（訂正）: lua/shiori-events.md#シーン関数フォールバック（見つかればコルーチンを実行して `RES.ok(script)`（200）、未発見・出力なしは 204） | ps:pasta/shiori/event/init.lua `EVENT.fire`、ps:pasta/scene.lua `co_exec` | 食い違い表「シーン関数フォールバック」 |
| L283 DSLシーンとの連携 | 収録先（新設）: lua/shiori-events.md#シーン関数フォールバック | ps:pasta/scene.lua `co_exec`（`find_handler("scene")` の 5 段検索） | — |
| L294 エラーハンドリング | 収録先（訂正）: lua/shiori-events.md#エラーハンドリング（保護は `SHIORI.request` の xpcall → `RES.err`（500）。フォールバック専用の pcall は無い） | ps:pasta/shiori/entry.lua `SHIORI.request` | 食い違い表「シーン関数フォールバック」 |
| L308 仮想ディスパッチャ | 収録先（新設）: lua/shiori-events.md#仮想ディスパッチャ | ps:pasta/shiori/event/virtual_dispatcher.lua | — |
| L316 dispatch(act) | 収録先（新設）: lua/shiori-events.md#dispatch(act) | ps:pasta/shiori/event/virtual_dispatcher.lua `M.dispatch` | — |
| L330 ブロック対象 Status キーワード | 収録先（新設）: lua/shiori-events.md#ブロック対象 Status キーワード | ps:pasta/shiori/event/virtual_dispatcher.lua（キーワード一覧・`has_status`） | — |
| L346 is_blocked(status) | 収録先（新設）: lua/shiori-events.md#is_blocked(status) | ps:pasta/shiori/event/virtual_dispatcher.lua `M.is_blocked` | — |
| L370 OnHour — 時報自動発行（4段階フォールバックチェーン） | 収録先（新設）: lua/shiori-events.md#OnHour（authoring-patterns §6.4 と統合） | ps:pasta/shiori/event/virtual_dispatcher.lua `M.check_hour` | — |
| L372 check_hour(act) | 収録先（新設）: lua/shiori-events.md#check_hour(act) | ps:pasta/shiori/event/virtual_dispatcher.lua `M.check_hour`（候補 4 つ、`transfer_date_to_var`） | — |
| L391 OnTalk — ランダムトーク自動発行 | 収録先（新設）: lua/shiori-events.md#OnTalk | ps:pasta/shiori/event/virtual_dispatcher.lua `M.check_talk` | — |
| L393 check_talk(act) | 収録先（新設）: lua/shiori-events.md#check_talk(act)（時報マージン・チェイントーク継続） | ps:pasta/shiori/event/virtual_dispatcher.lua `M.check_talk` | — |
| L401 pasta.toml設定 | 収録先（新設）: lua/shiori-events.md#pasta.toml 設定（reference/pasta-toml.md#[ghost]（ゴースト動作）へリンク） | ps:pasta/shiori/event/virtual_dispatcher.lua `get_config` | — |
| L421 テスト用関数 | 収録先（新設）: lua/shiori-events.md#テスト用関数 | ps:pasta/shiori/event/virtual_dispatcher.lua `M._reset`・`M._get_internal_state`・`M._set_scene_executor` | — |
| L439 関連リファレンス | 除外（スキル内ナビゲーション。SKILL.md と手書き→生成のリンクが担う） | 不要（ナビゲーション） | — |

### `book/src/lua/modules.md`

分割前の現行マニュアル章。各節は lua/modules/ 配下へ移り、旧 URL は `book.toml` のリダイレクトで lua/modules/index.html へ転送する（設計 #13）。

| 吸収元（見出し） | 収録先（章#節）／既存収録済み／除外（理由） | 実装照合 | 備考 |
| ---------------- | ------------------------------------------ | -------- | ---- |
| L1 公開モジュール API | 収録先（新設）: lua/modules/index.md（導入と締めを引き継ぐ） | 不要（章題） | — |
| L14 モジュール一覧 | 収録先（新設）: lua/modules/index.md#モジュール一覧 | lua:runtime/module_registry.rs（各モジュールの登録） | — |
| L35 @pasta_log — ロギング | 収録先（新設）: lua/modules/pasta-log.md | lua:runtime/log.rs | — |
| L60 @pasta_persistence — 永続化 | 収録先（新設）: lua/modules/pasta-persistence.md | lua:runtime/persistence.rs | — |
| L98 @pasta_config — 設定読み取り | 収録先（新設）: lua/modules/pasta-config.md | lua:runtime/module_registry.rs `register_config_module` | — |
| L117 @pasta_search — シーン・単語検索 | 収録先（新設）: lua/modules/pasta-search.md | lua:search/context.rs | — |
| L148 @pasta_sakura_script — さくらスクリプト変換 | 収録先（訂正）: lua/modules/pasta-sakura-script.md（`actor.talk` サブテーブル・`script_wait_default`・句点 100ms の記述を実装へ） | lua:sakura_script/mod.rs `resolve_wait_values`、lua:loader/config/sections.rs `TalkConfig` | 食い違い表「さくらスクリプト変換のウェイト」 |
| L172 @enc — 文字コード変換 | 収録先（新設）: lua/modules/enc.md | lua:runtime/enc.rs | — |
| L197 mlua-stdlib 統合モジュール | 収録先（新設）: lua/modules/mlua-stdlib.md | lua:runtime/runtime_config.rs | — |

## 付録: 未記載の実装事実

吸収元（doc/spec ch01–12・`GRAMMAR.md`・スキルの手書きリファレンス・現行 `book/src`）のどこにも書かれていない、現行実装が受理・処理する利用者向けの構文と挙動を 1 行ずつ挙げる（要件 1.8）。`grammar.pest` を規則ごとに読み、パーサ（`pasta_dsl`）・トランスパイラ（`pasta_lua` の code_gen）・ランタイム（`pasta_scripts`）で挙動を確かめた。本体の備考で「→1.2」とした項目はここで確定させる。

- 判定列は「収録先: 章#節」か「バグ候補（根拠）」のどちらか。根拠 a＝実行時エラー・パニック・不正なさくらスクリプトを生む、b＝吸収元や他の規範記述と矛盾する結果を生む、c＝ソースコメント・テストで意図外と明示されている。どれにも当たらず意図が不明なだけのものは収録先を書く。
- バグ候補はマニュアルに書かず、挙動も直さない（10.5）。`roadmap.md` へのキー行の申し送りは別タスクで行う。本表のバグ候補は 14 行（U06・U08・U12・U18・U19・U20・U21・U22・U23・U24・U25・U26・U27・U28）。
- U23〜U27 は、食い違い grep 記録（1.3）で見つかった挙動のうち、バグ候補かどうかの判定を 1.4 へ申し送ったもの（X19・D07・X18・D03）。吸収元に記述はあるが実装と食い違うため、現行挙動をマニュアルに書くかどうかをここで決める。実測はコミット `2d98dcf4`（`d95e12f2` から crates 配下は変わっていない）で同じ検証プログラムを動かした結果。
- 実装照合列の「実測」は、コミット `6e914384` の実装をスクラッチの検証プログラム（`parse_str` → `LuaTranspiler::transpile` → `PastaLoader::load` → `SHIORI.request`）で動かした結果。検証プログラムはリポジトリに残していない。出力は SHIORI レスポンスの `Value`（さくらスクリプト）または状態コード。

| # | 実装事実（構文と挙動） | 実装照合 | 収録先（章#節）／バグ候補（根拠） | 備考 |
| - | ---------------------- | -------- | -------------------------------- | ---- |
| U01 | `＊` 単独行（名前なしのグローバルシーン宣言）: 同じファイルで直前に宣言したグローバルシーンと同じ名前の、別のグローバルシーンを始める。前のシーンへ統合されるのではなく同名シーンの候補が 1 つ増え、ローカル単語・ローカルシーンは共有しない。ファイル先頭の `＊` 単独行はパースエラー（`Unnamed global scene at start of file…`） | pest:`global_scene_continue_line`・`global_scene_start`、dsl:parser/parse_scene.rs `parse_global_scene_start`（`last_name` を継承、無ければエラー）、gen:scope `generate_global_scene`（`PASTA.create_scene("名前")` をもう一度出す）。実測: `＊OnTest`（台詞 A）の後に `＊`（台詞 B）を置き OnTest を 4 回 → B・A・B・A | 収録先: grammar/block-structure.md#グローバルシーン（行の種類は #インデント不要の行、俯瞰は grammar/index.md#ファイル構造の俯瞰） | 本体 ch03 L18 行の「→1.2」 |
| U02 | `％` 行の番号付け `名前＝数字`: `％さくら、うにゅう＝5、まりか` は 0・5・6。番号を省いた要素は直前の番号＋1（先頭は 0）、番号付きはその値（全角数字可）。同じグローバルシーンの複数の `％` 行は採番を引き継ぐ。末尾の読点・同じ番号の重複も受理。アクター辞書にも pasta.toml にも無いアクター名は、立ち位置の設定が何もせず無視される | pest:`actors`・`actors_item = { id ~ ( s ~ set_marker ~ s ~ digit_id )? }`、dsl:parser/parse_scene.rs `parse_actors_item`（C# の enum 式採番・飽和加算）、gen:scope `generate_local_scene`（`act:set_spot(名前, 番号)`）、ps:pasta/act.lua `set_spot`（未登録アクターは何もしない）。実測: `％さくら、うにゅう＝5、まりか` と `％けろ、さくら＝0、` → `set_spot` が 0・5・6・7・0 | 収録先: grammar/actor-dictionary.md#シーンスコープ内でのアクター指定 | 本体 ch11 L47 行の「→1.2」 |
| U03 | `＄０`（数字だけの変数名）: シーン引数の参照。`＄n` はそのシーン関数が受け取った n＋1 番目の引数（全角数字可）。代入 `＄０＝…` はパースエラー。イベントで直接起動されたシーンでは値が無い（空文字＋警告）。番号は 0〜255 で、256 以上は 0 に丸められ `＄０` と同じになる | pest:`var_id = { id \| digit_id }`・`var_ref_local`（`var_set_local` は `id` だけを取る）、dsl:parser/parse_action.rs `parse_var_ref_local_inner`（`parse::<u8>().unwrap_or(0)`）、gen:elem `resolve_var_path`（`VarScope::Args(n)` → `args[n+1]`）。実測: OnTest の `[＄０]` → `[]`、`＄２５６` → `args[1]`、`＄０＝1` → パースエラー | 収録先: grammar/variables.md#シーン引数（新設。渡す側は grammar/call-jump.md#引数リスト） | 吸収元にはスキル variables.md L179 の「`＄０`（シーン引数）と `＄ｒ０` は別物」という注記しか無く、構文の定義は無い。本体 ch02 L50・スキル variables.md L162 行の「→1.2」 |
| U04 | Call の引数の転送: `＞シーン（引数…）` は、明示した引数の後ろに呼び出し元シーンが受け取った引数をすべて付け足して渡す。引数を書かない `＞シーン` は呼び出し元の引数をそのまま渡す。動的ターゲット（`＞＄x` など）も同じ | gen:elem `generate_call_scene`（`act:call(…, {}, 明示引数, table.unpack(args))`）、ps:pasta/act.lua `ACT_IMPL.call`（`handler(self, ...)`）。実測: OnTest → `＞中継（「a」）` → `＞子（「b」）` で、子の `＄０＄１` が `ba` | 収録先: grammar/call-jump.md#引数リスト | 本体 ch04 L103 行の「→1.2」 |
| U05 | 行末コメント: `#`・`＃` から行末までのコメントを行の後ろに置けるのは、グローバルシーン宣言・`＊` 単独行・ローカルシーン宣言・アクター辞書宣言（`％名前`）・`％` 行・属性行・単語定義行・変数代入行・Call 行・キューコマンド行・選択肢行・Lua ブロックの閉じフェンス。アクション行と継続行では `#` 以降も本文として出力される | pest:`or_comment_eol`（上記の各 `*_line` と `code_close` の終端）、`action_line`・`continue_action_line`（`eol` 終端）、`talk_word`（`#` を除外しない）。実測: `さくら：本文 ＃c4` → `本文 ＃c4` を出力。`＊OnTest ＃c1`・`＄x＝1 ＃c3` は正常に処理 | 収録先: grammar/block-structure.md#コメント | 本体 ch02 L354 行の「→1.2」。単語定義行の値への取り込みは U06 |
| U06 | 単語定義行の行末コメント（引用なしの値）: `＠w：あ、い ＃c` の `＃c` はコメントにならず、最後の値が `い ＃c` になる。値を `「」` で囲めば行末コメントとして扱われる | pest:`file_word_line`・`global_scene_word_line`（`or_comment_eol` 終端でコメントを許す形）と `word_nofenced = @{ (!(comma_sep \| "\r" \| "\n") ~ ANY)+ }`（`#` も行末まで取り込む。同じ引用なしの値でも `attr_string` は `comment_marker` を除外している）。実測: `SCENE:create_word("w"):entry("あ", "い ＃c2")`、`＠w：「あ」 ＃c` → `entry("あ")` | バグ候補（b）: 行規則はコメントを許す形なのに値が先に取り込むため、コメントの規範記述（book block-structure.md L133「`#` から行末までがコメント…どの位置にも置ける」、GRAMMAR.md L621）と矛盾する結果になる | — |
| U07 | `＄＄` エスケープ: アクション行の `＄＄`（`$$`）は `＄` 1 文字（2 文字目）を出力する。`＠＠` と同じく全角・半角の混在（`＠@`・`＄$`）も受理し、2 文字目を出す | pest:`dollar_escape = @{ dollar{2} }`・`at_escape = @{ at{2} }`、gen:elem `generate_action`（`Action::Escape` → 2 文字目を `talk`）。実測: `＄＄ ＠＠ ＠@` → `＄ ＠ @` | 収録先: grammar/action-line.md#インライン要素（`＠＠` の節は grammar/words.md#＠エスケープ） | 本体 ch06 L25 行の「→1.2」 |
| U08 | `\\` エスケープ: アクション行の `\\` は `\` 1 文字の台詞として出力され、さくらスクリプト上で直後の文字とつながってタグになる（`C:\\new` → `C:\new` で `\n` 改行タグ。行末の `\\` は終端の `\e` を `\\e` に変える）。`\` を表示させるには `\\\\` と書く必要がある | pest:`sakura_escape = @{ sakura_marker{2} }`、gen:elem `generate_action`（2 文字目の `\` を `talk` で出力）、ps:pasta/shiori/sakura_builder.lua（台詞中の `\` をエスケープしない）。実測: `C:\\new／\\\\／末尾\\` → `\p[0]C:\new／\\／末尾\\e` | バグ候補（a・b）: 意図しない（不正な）さくらスクリプトを生む。さくらスクリプトの規約「`\` を文字として表示する場合は `\\`」（doc/spec ch12 §12.16 が ukadoc を引用）と矛盾する | 本体 ch06 L25 行の「→1.2」。この判定により、本体 L148 行（doc/spec ch06 L25）の収録先「`＄＄`・`\\` のエスケープ…を追加」のうち `\\` は取り消す。マニュアルに追加するのは `＄＄`（U07）だけで、`\\` のエスケープは書かない |
| U09 | 関数呼び出し直後の空白: アクション行の `＠関数（）`・`＠＊関数（）` の後ろの空白は本文に残る（単語参照 `＠単語`・変数参照 `＄変数` の後ろの空白は区切りとして消える） | pest:`fn_call_local = { fn_marker ~ id ~ args }`（`args` が `)` で終わり末尾の `s` を取らない）、`word_ref`・`var_ref_*`（末尾の `s` が空白を消費）。実測: `[＠f() です][＠w です][＄x です]` → `[F です][です][です]` | 収録先: grammar/action-line.md#空白による区切り | 本体 ch06 L57 行の「→1.2」 |
| U10 | プロパティ名の文字種と式での制限: `＄％` の後のプロパティ名は ASCII 英字で始まり、英数字・`_`・`.`・`(`・`)` が続く（`currentghost.scope(0).name` のような添字を含められる）。全角文字や空白で名前が終わるため、直後に半角 `(` を書くと名前に含まれる（`＄％name(笑)` は `name(` を参照）。プロパティ参照は式の中（演算・関数の引数・Call ターゲット）に書けず、トランスパイルエラー（`Property reference cannot be used in expressions; use variable assignment first`）。代入の右辺に単独で置く `＄x＝＄％prop` だけが使える | pest:`property_id = @{ ASCII_ALPHA ~ ( "_" \| "." \| "(" \| ")" \| ASCII_DIGIT \| ASCII_ALPHA )* }`、gen:elem `resolve_var_path`（Property → `TranspileError::property_in_expression`）・`generate_var_set`（単独参照だけ `act:get_property`）。実測: `＄x＝＄％foo＋1` → トランスパイルエラー、`＄％name(笑)` → `get_property("name(")` | 収録先: grammar/variables.md#プロパティ変数 | 本体 ch09 L37 行の「→1.2」。アクター辞書配下のプロパティ代入は U13 |
| U11 | 演算子の優先順位と負号: 式は Lua の算術としてそのまま評価される。`＊`・`／`・`％` が `＋`・`－` より先で、同順位は左から。`／` は整数どうしでも小数になる（`7／2` → 3.5）。負号は数値リテラルにだけ付けられ、`-＄x` はパースエラー。文字列の連結演算子は無く、`＋` などはすべて Lua の算術になる。数字だけの文字列は数値に変換されて計算される（`「1」＋2` → 3） | pest:`expr = _{ term ~ s ~ bin* }`・`number_literal = @{ sub? ~ digit+ ~ … }`、dsl:parser/parse_action.rs `build_left_assoc_expr`（優先順位なしの左結合の木）、gen:elem `generate_expr_to_buffer`（`+ - * / %` を括弧を付けず平坦に出力するので、Lua の優先順位と型変換で評価される）。実測: `1＋2＊3` → 7、`10－4－3` → 3、`7／2` → 3.5、`＄d＝「1」＋2` → 3、`＄x＝-＄y` → パースエラー | 収録先: grammar/variables.md#式（Expression）のサポート（演算子の一覧は grammar/markers.md#算術演算子） | 括弧は U12。数値にできない被演算子のエラーは U22（バグ候補）で、マニュアルには書かない |
| U12 | 括弧式の中の演算: `（1＋2）＊3` の括弧内は最初の項しか残らず、`(1) * 3`（＝3）になる | dsl:parser/parse_action.rs `try_parse_expr`（`Rule::paren_expr` は最初の項だけを `Expr::Paren` にする）。crates/pasta_lsp/tests/var_set_token_test.rs `test_var_set_paren_expr_rhs_characterization` のコメントが「既知の上流問題…括弧内の最初の term しか AST 化せず」と明記。実測: 生成 Lua が `var.b = (1) * 3`、出力 3 | バグ候補（b・c）: 括弧式を項とする規範記述（doc/spec ch01 §1.3、GRAMMAR.md「式（Expression）のサポート」）と矛盾し、テストのコメントで既知の問題とされている | — |
| U13 | アクター辞書配下の属性行・変数代入行: `％名前` ブロックの下に置いた属性行（`＆…`）・変数代入行（`＄x＝…`・`＄＊x＝…`・`＄＝…`）・プロパティ代入（`＄％p＝…`）は構文として受理されるが、Lua に出力されず何も起きない | pest:`actor_scope_item`（`global_scene_attr_line`・`var_set_line` を含む）、dsl:parser/mod.rs `parse_actor_scope`（`attrs`・`var_sets` に格納し、`var_set_property` は捨てる）、gen:scope `generate_actor`（単語と `lua` ブロックだけを出力）。実測: アクター配下に `＄x＝1` を置いても OnTest の `＄x` は空（204） | 収録先: grammar/actor-dictionary.md#グローバルアクター辞書定義 | 本体 ch11 L12 行の「→1.2」 |
| U14 | 単語値の形: 値は「`「」`・`"` で囲んだ文字列」「さくらスクリプト 1 個（`\s[0]` など）」「それ以外の文字列」のいずれか。囲んだ文字列かさくらスクリプトで始まった値はそこで終わり、続けて文字を書くとパースエラーになる（`＠w：\s[0]こんにちは`・`＠w：「a」b`）。さくらスクリプトを含む値は文字で始める（`こんにちは\s[0]`） | pest:`word = _{ string_literal \| sakura_script \| word_nofenced }`（PEG の順序付き選択で先頭の要素に確定する）。実測: 上の 2 例はパースエラー | 収録先: grammar/words.md#グローバル単語定義（さくらスクリプトの可否は grammar/sakura-script.md#配置ルール） | 食い違い表「単語定義とさくらスクリプト」の訂正で書く条件 |
| U15 | 単語値の末尾の空白: 引用なしの値の行末側の空白は値に残る（`＠w：い  ` は `い  `）。読点・カンマの前後の空白は区切りとして消える | pest:`word_nofenced`（行末の空白も取り込む）、`comma_sep = _{ s ~ comma ~ s }`。実測: `＠w：あ   、い  ` → `entry("あ", "い  ")` | 収録先: grammar/words.md#グローバル単語定義 | — |
| U16 | 同じキー・同じアクターの重複定義: 同じスコープで同じキーの単語定義を複数書く（別の行・別のファイル）と、値はすべて同じ候補に合算される。同じ名前のアクター辞書 `％名前` を複数書いても 1 つのアクターにまとまり、単語が合算される | gen:elem `generate_global_word`（定義ごとに `PASTA.create_word(キー):entry(…)` を出す）、gen:scope `generate_actor`（`PASTA.create_actor` は ps:pasta/actor.lua `get_or_create` で同名を再利用）。実測: `＠g：a`・`＠g：b` と `％さくら` 2 つ（`＠表情：x`／`＠表情：y`）で、4 回の発火に a・b と x・y がどちらも出る | 収録先: grammar/words.md#単語の定義、grammar/actor-dictionary.md#グローバルアクター辞書定義 | — |
| U17 | ファイル末尾の改行: 最終行が改行で終わっていないと、その行がパースエラーになり、ファイル全体が読み込まれない。エディタ連携（pasta_lsp）は解析時に改行を補うため、エディタ上では診断が出ない | pest:各 `*_line` の `eol`・`or_comment_eol`（`NEWLINE` 必須）、lua:loader/process.rs（`read_to_string` の内容をそのまま `parse_str` へ渡す）、crates/pasta_dsl/src/partial.rs L238 と crates/pasta_lsp/tests/analysis_test.rs `test_analyze_without_trailing_newline_matches_with_newline`（改行を補うのは LSP 側だけ）。実測: `＊OnTest` ＋ `  さくら：最後`（末尾改行なし）→ 3:9 でパースエラー | 収録先: grammar/markers.md#改行（NEWLINE） | 「すべての行要素は改行で終わる」（book markers.md L31）から導けるが、ファイル末尾の扱いはどこにも書かれていない。LSP との差は規範記述との矛盾ではない |
| U18 | 未定義の `＠＊関数（）`: GLOBAL に無い関数を `＠＊名前（）` で呼ぶと Lua の実行時エラーになり、そのイベントは 500（`attempt to call field '名前' (a nil value)`）。ローカルの `＠名前（）` なら警告して空文字 | gen:elem `generate_action`・`generate_expr_to_buffer`（`GLOBAL.名前(act, …)` を存在確認なしで呼ぶ）、ps:pasta/act.lua `expr_fn`（ローカル側は warn＋nil）。実測: `前＠＊未定義()後` → 500、`前＠未定義()後` → `前後` | バグ候補（a・b）: 実行時エラーを生む。book variables.md L68「存在しない単語（`＠未定義`）や関数を参照したときも同じく空文字になる」と矛盾する | 本体 ch09 L81 行の「→1.2」 |
| U19 | 未登録アクターのアクション行: アクター辞書（`％名前`）にも pasta.toml `[actor]` にも無い名前でアクション行を書くと、パース・トランスパイルは通るが Lua の実行時エラーになり 500（`attempt to index field 'だれ' (a nil value)`） | gen:elem `generate_action`（`act.アクター:talk(…)`）、ps:pasta/act.lua `ACT_IMPL.__index`（登録アクター以外は nil）。実測: `だれ：こんにちは` → 500 | バグ候補（a）: 実行時エラーを生む | 本体 ch06 L9 行の「→1.2」。正しい使い方「アクション行のアクター名は、アクター辞書の `％名前` か pasta.toml `[actor]` に登録した名前」は収録先: grammar/action-line.md#基本構文 に書く。バグ候補は未登録時に 500 になる挙動だけ。`％` 行の未登録アクターは黙って無視される（U02）のと扱いが揃っていない |
| U20 | act のメンバー名と同じアクター名: アクター名が act のメソッド・フィールド名（`talk`・`word`・`wait`・`call`・`yield`・`clear`・`var`・`save` など）と同じだと、そのアクターのアクション行が実行時エラーになる | ps:pasta/act.lua `ACT_IMPL.__index`（メソッドを先に返し、アクターはその後）・`ACT.new`（`actors`・`save`・`var`・`token` などの実フィールド）、gen:elem `generate_action`（`act.名前:talk`）。実測: `％wait` を定義して `wait：こんにちは` → 500 | バグ候補（a）: 実行時エラーを生む | — |
| U21 | 末尾が数字のシーン名: 同名グローバルシーンの通し番号を名前の末尾に連結して内部名にするため（`A` の 1 個目 → `A1`、`A1` の 1 個目 → `A11`）、末尾が数字のシーン名は別名シーンの内部名と重なる。`＊A1` と `＊A` を定義すると `＞A1` が `A` を選ぶことがある。コード上、番号まで一致すると（`A1` の 1 個目と `A` の 11 個目がともに `A11`）2 つのシーンが同じシーン表を共有し、後の定義が先の関数を上書きする | lua:transpiler.rs `process_global_scene`（`format!("{}{}", sanitize_name(name), counter)`）、ps:pasta/scene.lua `create_scene`（`base_name .. counter`）・`register`（同じ表へ上書き）。実測: `＊A1`（台詞 シーンA1）と `＊A`（台詞 シーンA）で `＞A1` を 6 回 → A1・A・A・A1・A1・A | バグ候補（b）: 前方一致の規範記述（book block-structure.md L51、GRAMMAR.md L120「`＞挨拶` では「挨拶」で始まるすべてのシーンが候補」）と矛盾する（`A` は `A1` で始まらない） | — |
| U22 | 数値にできない被演算子の算術: 数字でない文字列・未代入の変数・nil を返す関数を算術演算子の被演算子にすると、Lua の実行時エラーになり、そのイベントは 500（`attempt to perform arithmetic on …`） | gen:elem `generate_expr_to_buffer`（`+ - * / %` をそのまま出力）・`resolve_var_path`（`var.名前` を nil の確認なしで参照）。実測: `＄z＝「a」＋「b」` → 500、`＄x＝＄未定義＋1` → 500 | バグ候補（a・b）: 受理される式が実行時エラーを生む。未代入の変数は「空文字として展開され、ログに警告」（book variables.md L68、GRAMMAR.md L252）とする規範記述と矛盾する | 算術の通常の挙動（連結演算子なし・数字だけの文字列の数値化）は U11 で収録する。バグ候補はこのエラー挙動だけ |
| U23 | REG ハンドラの戻り値: `REG.イベント名 = function(act) … end` の戻り値は、文字列なら `RES.ok(文字列)`（200・`Value` にその文字列）、シーンのコルーチン（thread）なら再開して得た値を `RES.ok`、nil なら `RES.no_content()`（204）に変換される。ハンドラが `RES.ok(…)`・`RES.no_content()` の結果（応答全体の文字列）を返すと、それがさらに `RES.ok` で包まれ、`Value` に応答全体が入れ子になる | ps:pasta/shiori/event/init.lua `EVENT.fire`（L171〜。文字列・thread・nil の分岐）。同じファイルの使用例 L40 は `return RES.ok(act:build())`、L201 のコメントは「既存互換: 文字列をそのまま返す」。ps:pasta/shiori/event/register.lua の使用例（L27〜L35）も `return RES.ok(…)`。crates/pasta_lua/tests/shiori/event_dispatch_test.rs `test_event_fire_dispatches_registered_handler` は `return RES.ok("test response")` を部分一致（`find`）で検査するため入れ子でも通る。実測: `return RES.ok("hi")` → `Value: SHIORI/3.0 200 OK` に続けて応答全体（内側の `Value: hi` を含む）、`return "raw"` → `Value: raw` | バグ候補（a・b・c）: `Value` に改行入りの応答全体が入り、不正な SHIORI 応答（さくらスクリプトとしても不正）になる。吸収元（スキル shiori-handlers.md L22 ほかの登録パターン、book lua/patterns.md L96〜L104）の `return RES.ok(…)` の書き方と矛盾し、ソースの使用例・コメント・テストは `RES.ok(…)` を返す書き方を意図している | X19 の確定。マニュアル（2.6 lua/shiori-events.md の REG 節）に収録するのは「文字列を返すと 200、nil で 204、シーンのコルーチンも返せる」の部分だけ。`RES.ok(…)`・`RES.no_content()` を返す例は書かない（X19 の訂正対象はこの方針で直す） |
| U24 | 改行を含む引用文字列: `「…」`・`"…"` の文字列は閉じの囲みまで改行をまたいで取り込まれる。改行を含む値は生成 Lua の `"…"` リテラルに改行がそのまま入り、ロード時に Lua の構文エラー（`unfinished string`）になって、ゴーストの Lua ランタイムの初期化全体が失敗する。同じ理由で、1 ファイルに空文字列 `""` を 2 つ以上書くと、最初の `""` が空文字列ではなく `""` 囲みの開始になり、次の `""` までを（改行を含めて）取り込んで同じロード失敗になる | pest:`string_contents = @{ (!PEEK ~ ANY)+ }`（改行を除かない）・`string_literal = _{ string_fenced \| string_blank }`（`slfence_en = _{ PUSH("\""+) }` が `""` を囲みとして先に試す）、dsl:parser/parse_action.rs（`Rule::string_contents` をそのまま `Expr::String`）、lua:string_literalizer.rs `needs_long_string`（`\` と `"` だけを見て、改行を含む値も `"…"` で出力する）。実測: `＄s＝"a`＋改行＋`b"`・`＄s＝「a`＋改行＋`b」` → `load: ERR … unfinished string near '"a'`（`pasta.scene_dic` のロード失敗）。`＄f＝""` と `＄g＝""` の 2 行 → 同じロード失敗（`＄f＝""` が 1 つだけなら空文字列） | バグ候補（a）: パースもトランスパイルも通る構文が、ロード時のエラーでゴースト全体を起動不能にする | D07 の確定（「改行入り `"` 文字列でロード失敗」を `「」` 囲みと `""` の 2 回使用まで広げた）。マニュアルには引用文字列が改行を含められるとは書かない。空文字列の例は `「」` で書く（`""` は 2 つ目でこの挙動に当たる） |
| U25 | 単語値の `「」`・`""`: 単語定義の値に書いた `「」`・`""` は空文字列にならず、その 2 文字（`「」`・`""`）が候補の値になる。変数代入や引数などの式に書いた `「」`・`""` は空文字列になる | pest:`string_blank = @{ "\"\"" \| "「」" }`（`word` も `expr` も `string_literal` 経由で受け付ける）、dsl:parser/parse_elements.rs（単語値は `Rule::string_blank` の字面 `as_str()` をそのまま値にする。属性値の `parse_attr` も同じ）、dsl:parser/parse_action.rs（式では `Rule::string_blank` → `Expr::BlankString`）。実測: `＠w：x、「」` → `entry("x", "「」")`、`＠v：""` → `entry([[""]])`、出力は `「」`・`""`。`＄e＝「」`・`＄f＝""` → `var.e = ""`（空） | バグ候補（b）: 同じ空文字列リテラルが式では空、単語値では囲み文字そのものになり、「外側の `「」` は単語値の区切り」（GRAMMAR.md L517）・「引用符で囲まれた中身が文字列」（doc/spec ch05 L18、book literals.md L45）の規範記述と矛盾する | D07 の確定。マニュアルには単語値の空文字列の書き方を載せない。式の `「」` が空文字列になることは literals.md に書く（`""` は U24 に当たるため例にしない） |
| U26 | pasta.toml の `[lua]` セクション: `[lua] libs` はロード時に読まれず、書いても Lua 標準ライブラリ・mlua-stdlib モジュールの構成は既定のまま変わらない（`"env"` を足しても `@env` は使えず、既定に含まれる `@json` などは外せない） | lua:loader/mod.rs `PastaLoader::load`（`load_with_config(base_dir, RuntimeConfig::new())`）、crates/pasta_shiori/src/shiori.rs（SHIORI のロードも `RuntimeConfig::new()` から作る）、lua:loader/config/mod.rs `PastaConfig::lua`（呼び出し元なし）、lua:runtime/runtime_config.rs `From<LuaConfig> for RuntimeConfig`（ロード経路から呼ばれない）、lua:loader/config/sections.rs `LuaConfig` の doc コメント（`[lua]` セクションで有効にするライブラリを構成すると明記）。実測: `[lua] libs = ["std_all", "env"]` で `require "@env"` は失敗し、`@json` は読める | バグ候補（b・c）: 吸収元（スキル pasta-toml.md L49・L145〜L146 の `[lua]` 既定値と節、runtime-api.md L655「`libs` 配列に `"env"` エントリを追加」）と矛盾し、ソースの doc コメントも `[lua]` が構成を決めると明記している | X18 の確定。マニュアル（2.5・2.7）は `[lua]` が効くとは書かず、`@env` は通常のゴーストから有効にできないことだけを書く（X18 の結論どおり） |
| U27 | 選択肢の自動ルーティングの探索範囲: 既定の `REG.OnChoiceSelectEx` は、明示の `＊OnChoiceSelectEx` シーンが無いとき、選択 ID（Reference0）を直前に実行したグローバルシーンのローカルシーンからだけ前方一致で探す。グローバルシーンへはフォールバックせず、見つからなければ 204。シーンをまだ 1 つも実行していない（`STORE.last_global_scene` が nil の）ときだけグローバルシーンから探す | ps:pasta/shiori/event/choice_select.lua `REG.OnChoiceSelectEx`（`SCENE.search(choice_id, STORE.last_global_scene)`。L56 のコメントは「ローカル→グローバル、3.1/3.4」）、ps:pasta/scene.lua `SCENE.search` → lua:search/context.rs `search_scene`（親シーン名ありは「Local-only search … (no global fallback)」）、ps:pasta/act.lua（L181 で `STORE.last_global_scene` を更新）。完了済み仕様 `.kiro/specs/completed/choice-definition-dsl/requirements.md` 要件 3.4 は「ローカル → グローバルの順で前方一致検索」。実測: OnTest（ローカル「ローカル先」とグローバル「行き先」の選択肢）の後、選択 ID `ローカル先` → 200（`L`）、`行き先` → 204 | バグ候補（b・c）: 完了済み仕様の要件 3.4 と吸収元（スキル SKILL.md L276「ローカルシーン → グローバルシーンの順で検索」、authoring-patterns.md L347）に矛盾し、ソースのコメント（choice_select.lua L56）とも食い違う | D03 の確定。マニュアル（2.6 lua/shiori-events.md の OnChoiceSelectEx 節）に収録するのは「明示の `＊OnChoiceSelectEx` が優先」「選択 ID と同名のローカルシーン（直前のグローバルシーン内）を前方一致で自動実行し、見つからなければ 204」の部分。グローバルシーンへのフォールバックの有無は書かない（D03 の訂正対象 ga/SKILL.md L276・authoring-patterns.md L347 は、フォールバックの順序に触れない記述へ直す） |
| U28 | 別グローバルシーンへの Call 後のローカル探索: `＞` で別のグローバルシーンを呼んで戻ったあと、呼び出し元の後続の Call・ローカル単語参照が呼ばれた側のローカルシーンを探す | ps:pasta/act.lua `init_scene`（`act.current_scene` を上書きし、戻り時に復元しない）・`find_act_handler` L1/L2。実測（2.2 レビュー）: OnA が `＞挨拶`→`＞別グローバル`→`＞挨拶` で `A[LXY]`（2 回目が呼ばれた側の `挨拶Y` に解決） | バグ候補（b）: 「実行中のグローバルシーンのローカルシーンを探す」とする吸収元（doc/spec ch04・スキル call-spec）とマニュアル call-jump.md#スコープ解決アルゴリズムの規範と矛盾する | 2.2 の実装・レビューで発見。マニュアルは通常時の挙動（実行中のグローバルシーン）だけを書く。大タスク 6 で roadmap のバグ候補キー行に含める |

## 食い違い grep 記録

design.md「既知の食い違い（実装が正）」表の各行（D01〜D18）と、台帳本体の備考で「→1.3」と申し送った項目（X01〜X22）について、マニュアル全章・両 `SKILL.md`・スキル references の全ファイルを grep した結果と、照合した実装の位置を記録する（要件 1.6, 3.3）。訂正そのものは各「訂正先」タスクが行う。

- grep 範囲: `book/src/**/*.md`、`.claude/skills/pasta-ghost-authoring/SKILL.md`・`references/*.md`、`.claude/skills/pasta-lua-coding/SKILL.md`・`references/*.md`。コマンドは `grep -arnE --include=*.md -e "パターン" 上記パス`。行番号はコミット `d95e12f2`（main 取り込み後）の時点。`doc/spec/`・`GRAMMAR.md` は撤去されるため記録しない。
- 位置の略記: book の章は `book/src/` からの相対パス。`ga/` は `.claude/skills/pasta-ghost-authoring/`、`lc/` は `.claude/skills/pasta-lua-coding/`。実装照合の略記は凡例「実装照合列の略記」と同じ。
- 判定: **訂正対象**（実装と食い違う誤記）／**追記対象**（誤りではないが実装事実が欠け、収録時に補う）／**正**（実装と一致し訂正不要）／**生成で置換（4.1）**（生成ファイルで置き換わるスキルの規範ファイル。手で直さない）。
- 訂正先: 文法章は 2.1（grammar/index・markers・block-structure）、2.2（call-jump・literals・action-line）、2.3（sakura-script・variables・words・actor-dictionary）。旧 `lua/modules.md` は節ごとに 2.4（一覧・@pasta_search・@pasta_persistence・@pasta_config）／2.5（@pasta_sakura_script・@enc・@pasta_log・mlua-stdlib）。新章は 2.6（lua/shiori-events.md）・2.7（reference/pasta-toml.md）。生成対象外章と reference/startup.md は 2.8。スキルは 4.2（`ga/SKILL.md`）・4.3（`ga/references/authoring-patterns.md`）・4.4（`lc/SKILL.md` と手書き 3 ファイル）。スキルの生成ファイルのうち新章の吸収元になるもの（`ga/references/pasta-toml.md`・`lc/references/runtime-api.md`・`lc/references/shiori-handlers.md` と文法 7 ファイル）は「生成で置換（4.1）」とし、その誤りは移設先章のタスクが移設時に訂正する（訂正先列に併記）。
- 実測: 1.2 と同じスクラッチの検証プログラム（`parse_str` → `LuaTranspiler::transpile` → `PastaLoader::load` → `SHIORI.request`）で、コミット `d95e12f2` の実装を動かした結果。プログラムはリポジトリに置かない。

### D01 行継続

- grep: `継続`
- 実装照合: pest:`continue_action_line = { pad ~ kv_marker ~ s ~ actions ~ eol }`（継続行は `：` 始まり）、gen:elem `generate_continue_action`（先行アクターなし → `TranspileError::InvalidContinuation`）。実測: インデントだけの継続行はパースエラー（`expected EOI, var_set_…`）、`：` 始まりは前行のアクターで連結（`AB`）、先行アクターなしは `Continuation action without actor`。
- 結論: 表どおり（実装が正）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/action-line.md:27 | 節「アクターの省略」（`：` で前の発話を継続）（L29） | 正 | — |
| grammar/action-line.md:91 | 節「行継続」: インデントした行を続けると連結（L93〜L102） | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/index.md:70 | 章一覧「発言行・インライン要素・行継続・改行」 | 正 | — |
| ga/SKILL.md:80 | アクター名を省略すると直前のアクターが継続 | 正 | — |
| ga/SKILL.md:82 | 行継続: インデント付きの次行で発話を継続 | 訂正対象 | 4.2 |
| ga/SKILL.md:112 | よくある間違い c「継続行はマーカーなしで開始」 | 訂正対象 | 4.2 |
| ga/references/action-line.md:112 | 節「行継続」: `INDENT ~ !(statement_marker) ~ content`（L120 の制約を含む） | 生成で置換（4.1） | 4.1 |
| ga/references/action-line.md:16 | アクター名を省略すると直前のアクターが継続 | 生成で置換（4.1） | 4.1 |

（他のヒット `grammar/actor-dictionary.md:73`・`introduction.md:32`・`reference/startup.md:65`・`reference/startup.md:136`・`ga/SKILL.md:349`・`ga/SKILL.md:378`・`ga/references/authoring-patterns.md:205`・`ga/references/authoring-patterns.md:235`・`ga/references/call-spec.md:68`・`lc/references/internal-modules.md:28` は「継続トーク」「処理の継続」等の別概念で、対象外。）

他項目で記録: `grammar/action-line.md:109,119`・`ga/references/action-line.md:134,136`（D02）。

### D02 継続内の空行

- grep: `継続`、`空行`、`糖衣構文`
- 実装照合: pest:`blank_line = _{ or_comment_eol }`（サイレント規則で AST を作らない）、`local_scene_item`（`blank_line` を含む）。実測: `さくら：A`・空行・`：B` → `\p[0]AB`（改行は入らない）。
- 結論: 表どおり（空行は何も出力しない）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/action-line.md:109 | 継続行内の空行は 1 改行（糖衣構文）、連続空行は連続改行（L112〜L119 の例と説明を含む） | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/action-line.md:108 | さくらスクリプトの `\n` は改行 | 正 | — |
| ga/references/action-line.md:134 | 糖衣構文: 行継続領域内の空行は改行（L133〜L136）（L144 の出力例を含む） | 生成で置換（4.1） | 4.1 |

他項目で記録: `grammar/action-line.md:29,91,102`・`grammar/index.md:70`・`ga/SKILL.md:80,82,112`・`ga/references/action-line.md:16,112,120`（D01）。

対象外: `grammar/actor-dictionary.md:73`・`introduction.md:32`・`reference/startup.md:65,136`・`ga/references/authoring-patterns.md:205,235`・`ga/references/call-spec.md:68`・`lc/references/internal-modules.md:28`・`ga/SKILL.md:349,378`（「継続トーク」「処理の継続」など、行継続と別の概念）。

### D03 ローカル／グローバル候補

- grep: `統合|マージ|併合|合算|ローカル.{0,20}グローバル|両方.{0,10}候補`、`フォールバック`
- 実装照合: core:scene_table.rs `collect_scene_candidates`・core:word_table.rs `collect_word_candidates`（親シーン名ありはローカルのみ・フォールバックなし、なしはグローバルのみ）、ps:pasta/act.lua `find_act_handler`（L2 で見つかれば返し、無ければ L3〜L5 へ）、lua:search/context.rs `search_scene`（「no local → global fallback」）。実測: グローバル `＠挨拶：G` とローカル `＠挨拶：L` で 6 回参照 → `LLLLLL`。 選択肢の自動ルーティングは ps:pasta/shiori/event/choice_select.lua `REG.OnChoiceSelectEx`（`SCENE.search(選択ID, STORE.last_global_scene)`）→ ps:pasta/scene.lua `SCENE.search` → `search_scene`（親シーン名ありはローカルのみ）。実測: 選択 ID にローカルシーン名 → 実行（200）、グローバルシーン名 → 204。
- 結論: 表どおり。加えて `@pasta_search` の `search_scene`・`search_word` 自体はフォールバックしない（第 2 引数ありはローカルのみ）。ローカル → グローバルの順は act 側（L2 → L5）が担う。 同じ理由で、OnChoiceSelectEx の自動ルーティングは直前のグローバルシーンのローカルシーンだけを探し、グローバルシーンへはフォールバックしない（ソースのコメント「ローカル→グローバル」と食い違う）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/call-jump.md:68 | スコープ解決 3.「マージ: 両検索結果を結合して候補リストを生成」（L66〜L69 の 4 手順を含む） | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/call-jump.md:31 | シーン参照はローカル／グローバルシーンを前方一致検索 | 正 | — |
| grammar/call-jump.md:57 | `＞挨拶` → 「挨拶朝」「挨拶昼」の両方が候補 | 正 | — |
| grammar/words.md:86 | ローカルとグローバルの同名単語は「マージ」されて候補プールが統合（L88〜L94 の例「どちらかが選ばれる」を含む） | 訂正対象 | 2.3 |
| grammar/actor-dictionary.md:52 | アクター辞書に無ければ「グローバル単語辞書・ローカル単語辞書」へフォールバック（順序がローカル優先と逆）（節見出し L48） | 訂正対象 | 2.3 |
| lua/modules.md:119 | @pasta_search は「フォールバック戦略（ローカル → グローバル）を備える」 | 訂正対象 | 2.4 |
| lua/modules.md:133 | 「ローカル優先検索（第2引数に親グローバルシーン名を指定）」（実際はローカルのみ） | 訂正対象 | 2.4 |
| ga/SKILL.md:66 | 前方一致で「挨拶朝」「挨拶昼」の両方が候補 | 正 | — |
| ga/SKILL.md:125 | スコープ解決: ローカル → グローバルの順に前方一致検索（優先順として読める） | 正 | — |
| ga/SKILL.md:64 | ローカルシーンは親グローバルシーン内でのみアクセス可能 | 正 | — |
| ga/SKILL.md:186 | アクター辞書に該当単語がない場合、グローバル/ローカル単語辞書にフォールバック（順序がローカル優先と逆） | 訂正対象 | 4.2 |
| ga/SKILL.md:276 | OnChoiceSelectEx は選択 ID を「ローカルシーン → グローバルシーンの順」で検索（実際はローカルのみ） | 訂正対象 | 4.2 |
| ga/references/authoring-patterns.md:347 | 選択 ID で「ローカル→グローバルの順に」前方一致検索（実際はローカルのみ） | 訂正対象 | 4.3 |
| lc/SKILL.md:132 | `@pasta_search`（シーン・単語検索、フォールバック戦略） | 訂正対象 | 4.4 |
| ga/references/call-spec.md:27 | 「マージ: 両検索結果を結合」（L25〜L27。L41 の「両方が候補」は正） | 生成で置換（4.1） | 4.1 |
| ga/references/words.md:82 | 「マージ: 両検索結果を結合」（L80〜L82。L90 の「両方が候補」は正） | 生成で置換（4.1） | 4.1 |
| lc/references/runtime-api.md:20 | search_scene は「フォールバック戦略（ローカル → グローバル）」（L37〜L38・L48 も同旨） | 生成で置換（4.1） | 2.4（移設時）・4.1 |
| lc/references/runtime-api.md:63 | search_word は「フォールバック戦略（ローカル → グローバル）」 | 生成で置換（4.1） | 2.4（移設時）・4.1 |
| lc/references/internal-modules.md:271 | `find_act_handler` の L1〜L5 フォールバック順序（L263・L274・L294・L303・L306・L449 を含む） | 正 | — |

他項目で記録: `ga/references/actor-dictionary.md:55`（D04）、`lua/patterns.md:128`・`lua/dsl-vs-lua.md:29`・`getting-started/first-ghost.md:148`・`lc/references/shiori-handlers.md:265,269,297`（D15）、`lc/SKILL.md:148`（D13・X19）。

対象外:
- `debug/vscode-setup.md:12`・`getting-started/first-ghost.md:232`・`lua/modules.md:29,197`・`lc/references/runtime-api.md:583,585`・`lc/references/testing-lint.md:229`・`lc/SKILL.md:87`（「統合」がデバッグ統合・統合テスト・mlua-stdlib 統合などの別の意味）。
- `grammar/block-structure.md:29`・`grammar/index.md:72`・`grammar/variables.md:108,112`・`lc/references/internal-modules.md:606`（変数・関数のスコープ種別の列挙で、候補の選び方に触れない）。
- `lc/references/runtime-api.md:328`・`lc/references/shiori-handlers.md:409,418`・`ga/SKILL.md:313`（設定値のマージ・時報マージン）。
- `getting-started/first-ghost.md:241,249,272`・`ga/references/authoring-patterns.md:69,126`・`lc/references/shiori-handlers.md:4,370,381,389`・`ga/SKILL.md:342,356,362,375`（時報の 4 段フォールバックとイベントのシーン関数フォールバックの紹介で、単語・シーン候補の統合ではない）。
- `grammar/actor-dictionary.md:65,121`・`ga/references/variables.md:216,219`・`debug/source-level.md:61`（外見・設定値の既定値へのフォールバック）。
- `lc/references/internal-modules.md:742`・`lc/references/runtime-api.md:544,546`（文字列化・バッファ実装のフォールバック）。
- `lua/dsl-vs-lua.md:37`（標準フォールバックで扱えないイベントは REG へ、という方針の説明）。

### D04 Call 検索

- grep: `[0-9０-９]\s*段|段階|検索順|フォールバック`、`スコープ解決アルゴリズム`
- 実装照合: ps:pasta/act.lua `ACT_IMPL.call`（`find_handler("scene", key)`）→ `find_act_handler` の L1〜L5、ps:pasta/actor.lua `find_actor_handler`（アクター付きは A1・A2 が先）。実測: Lua ブロックの `function SCENE.f` が単語参照 `＠f` で L1 から呼ばれ `F` を出力。
- 結論: 表どおり（5 段）。加えて、単語参照も L1（シーン表）・L3（act メソッド）・L4（GLOBAL）を引くため、「Call はシーン辞書のみ、単語参照はシーン辞書を参照しない」は辞書（L2・L5）に限った言い方であり、L1・L3・L4 では関数も見つかる。 アクター付きの単語参照（`アクター：＠単語`）は A1 → A2 → L1〜L5 の順で、ローカル（L2）がグローバル（L5）より先。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/call-jump.md:62 | 節「スコープ解決アルゴリズム」: ローカル検索・グローバル検索・マージ・選択の 4 段（L64〜L71） | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/call-jump.md:73 | 「Call はシーン辞書のみ、単語参照はシーン辞書を参照しない」 | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/words.md:81 | 検索順序はローカル単語辞書 → グローバル単語辞書の 2 段（L83〜L84） | 訂正対象 | 2.3 |
| grammar/words.md:104 | 単語参照は単語辞書のみ、Call はシーン辞書のみ | 訂正対象 | 2.3 |
| ga/references/call-spec.md:23 | 「2段階検索」（節見出し L19・L21 を含む） | 生成で置換（4.1） | 4.1 |
| ga/references/words.md:76 | 節「スコープ解決アルゴリズム」（ローカル・グローバル・マージの手順） | 生成で置換（4.1） | 4.1 |
| ga/references/actor-dictionary.md:55 | 節「フォールバック検索順」: アクター辞書 → ローカル単語辞書 → グローバル単語辞書の 3 段（L57〜L61。L1・L3・L4 が無い） | 生成で置換（4.1） | 2.3（移設時）・4.1 |
| lc/references/internal-modules.md:294 | `find_act_handler` の「6段階」（L1〜L5＋nil。ソースコメントと同じ数え方）（L263・L271・L274・L303・L306・L338 を含む） | 正 | — |
| lc/references/internal-modules.md:449 | 完全なフォールバックチェーン A1 → A2 → L1〜L5（L434 の A1・A2 を含む） | 正 | — |

他項目で記録: `grammar/actor-dictionary.md:52`・`grammar/words.md:86`・`lua/modules.md:119`・`lc/references/runtime-api.md:20,37,38,48,63`・`lc/SKILL.md:132`・`ga/SKILL.md:186`（D03）、`lua/patterns.md:128`・`lua/dsl-vs-lua.md:29`・`getting-started/first-ghost.md:148`・`lc/references/shiori-handlers.md:265,269,297`（D15）、`lc/SKILL.md:148`（D13・X19）。

対象外:
- `getting-started/first-ghost.md:241,249,272`・`ga/references/authoring-patterns.md:69,126`・`lc/references/shiori-handlers.md:4,370,381,389`・`ga/SKILL.md:342,356,362,375`（時報の 4 段フォールバックとイベントのシーン関数フォールバックの紹介。Call・単語の検索段ではない）。
- `grammar/actor-dictionary.md:65,121`・`ga/references/variables.md:216,219`・`debug/source-level.md:61`（外見・設定値の既定値へのフォールバック）。
- `lc/references/internal-modules.md:742`・`lc/references/runtime-api.md:544,546`（文字列化・バッファ実装のフォールバック）。
- `grammar/words.md:130`・`ga/references/words.md:94`（`＠＠` の多段階参照は非対応という、エスケープの説明）。
- `getting-started/first-ghost.md:9`・`getting-started/index.md:11`・`reference/startup.md:144`（「段階」が作業・起動の段階の意味）。
- `lua/dsl-vs-lua.md:37`（標準フォールバックで扱えないイベントは REG へ、という方針の説明）。

### D05 Call フィルター `＞シーン＆k＝v`

- grep: `フィルター|フィルタ|＞[^ 　]*＆`
- 実装照合: pest:`call_scene = { call_marker ~ (id | call_target_expr) ~ s ~ args? }`（`＆` を受ける規則が無い）。実測: `＞X＆k＝v` はパースエラー（`expected args`）。
- 結論: 表どおり（構文として受理されない）。節の削除と brief への移送は 1.4 の仕分けに従う。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/call-jump.md:109 | 節「フィルター（将来変更あり）」: 構文が予約され現状は無視（L111〜L119） | 訂正対象・訂正済み（2.2） | 2.2（節削除） |
| ga/references/call-spec.md:14 | `＞シーン名＆key＝value ← 属性フィルター付き` | 生成で置換（4.1） | 4.1 |
| ga/references/call-spec.md:46 | 節「属性フィルター」: 現在は将来予約（無視される）（L52）（L49 の例を含む） | 生成で置換（4.1） | 4.1 |

（`grammar/markers.md:39` のフィルター言及は X04 で扱う。`lc/references/testing-lint.md:237` はテスト名フィルタで対象外。）

対象外: `ga/references/action-line.md:120`（行マーカーの列挙 `＄＠＞＆＊・` がパターンに当たっただけ）。

### D06 引数

- grep: `引数`、`（[a-zA-Z]+：|\([a-zA-Z]+：|名前付き引数|位置引数`
- 実装照合: pest:`args = { lparen ~ s ~ (arg ~ (comma_sep ~ arg)*)? ~ s ~ rparen }`・`arg = _{ key_arg | positional_arg }`、gen:elem `generate_args_string`（`Keyword` は名前を捨てて値だけを位置引数として出す）・`generate_call_scene`（明示引数の後ろに `table.unpack(args)`）。実測: `＠add（x：10　y：20）` はパースエラー、`＠add（x：10、20，30,40）` は `act:expr_fn("add", 10, 20, 30, 40)` になり `10/20/30/40` を出力。
- 結論: 表どおり（読点／カンマ区切り、位置引数可、名前付きは名前が無視される）。加えて `＠greet（time：morning）` のように値が式として不正な場合は関数呼び出しとして解析されず、単語参照 `＠greet`＋本文「（time：morning）」になる（実測: `act.ぱすた:word("greet")` と `talk("（time：morning）")` を生成）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/action-line.md:53 | `＠func（x：10）`「名前付き引数で呼び出し」 | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/action-line.md:60 | 例 `＠greet（time：morning）`（関数呼び出しにならない） | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/call-jump.md:121 | 節「引数リスト」: 名前付き・空白区切り・トランスパイラ以降は対応予定（L123〜L129） | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/variables.md:82 | `＄sum＝＠add（x：10　y：20）`「名前付き引数で呼び出し」 | 訂正対象 | 2.3 |
| grammar/index.md:19 | 関数引数で算術式を記述できる | 正 | — |
| ga/references/action-line.md:33 | `＠func（x：10）`「名前付き引数で呼び出し」 | 生成で置換（4.1） | 4.1 |
| ga/references/variables.md:46 | `＄sum：＠add（x：10　y：20）` | 生成で置換（4.1） | 4.1 |

他項目で記録: `lua/modules.md:133`（D03）。

対象外:
- `grammar/literals.md:9`・`ga/references/grammar-model.md:84,150`（リテラル・式を関数引数に使えるという記述で、区切り・名前付きに触れない。正）。
- `ga/references/variables.md:179`（シーン引数 `＄０`。付録 U03）。
- `ga/references/grammar-model.md:206,223,226,227`（キューコマンドの引数）。
- `debug/source-level.md:60`・`grammar/actor-dictionary.md:120,142`・`grammar/sakura-script.md:57`（DAP の起動引数・bind タグの引数・さくらスクリプトの引数）。
- `lua/patterns.md:83,174`・`lc/references/coding-conventions.md:123,174,281,282,289,293`・`lc/references/internal-modules.md:91,168,200,206,319,338,350,458,517,543,559`・`lc/references/runtime-api.md:545`・`lc/references/shiori-handlers.md:172`・`lc/references/testing-lint.md:57,147,150,169`（Lua 関数・SHIORI Reference の引数で、DSL の引数構文ではない）。

### D07 文字列エスケープ

- grep: `エスケープ|\\n\`|\\\\"`
- 実装照合: pest:`string_fenced = _{ strfence ~ string_contents ~ strclose }`・`strfence`（`「`×4〜×1、`"`＋）・`string_contents = @{ (!PEEK ~ ANY)+ }`・`string_blank`、dsl:parser/parse_action.rs（`Rule::string_contents` をそのまま `Expr::String`、`Rule::string_blank` → `Expr::BlankString`）。実測: `＄s＝"a\nb\"` の値は `a\nb\`（`\n`・`\"` は変換されない）、`＄e＝「」`・`＄f＝""` は空文字列。
- 結論: 表どおり（エスケープ規則なし、囲みの多重化、式の空文字列可）。表に無い差分を 2 点記録する: (1) 単語値では `""`・`「」` は空文字列にならず、その 2 文字が値になる（実測: `＠w：…、""、「」` の候補が `""` と `「」`。dsl:parser/parse_elements.rs が `string_blank` の字面を値にする）。(2) `"` 囲みの文字列は改行を含められ（`string_contents` が改行も取る）、生成 Lua が `"a` 改行 `b"` になってロード時に `unfinished string` で失敗する（実測）。いずれもマニュアルへの収録・バグ候補の判定は付録の担当範囲で扱う（本節は記録のみ）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/literals.md:63 | 節「文字列エスケープ」: `\n` `\\` `\"`（L65〜L71） | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/literals.md:45 | 引用符あり: `「...」` または `"..."`、空白も保持 | 正 | — |
| grammar/sakura-script.md:13 | さくらスクリプトのエスケープ文字は半角 `\` のみ | 正 | — |
| grammar/action-line.md:50 | ＠エスケープ `＠＠` | 正 | — |
| grammar/words.md:128 | ＠エスケープ `＠＠`（節見出し L126・L130 を含む） | 正 | — |
| ga/SKILL.md:212 | さくらスクリプトは半角で記述 | 正 | — |

（`grammar/words.md:137`・`grammar/literals.md:84` の「引用符エスケープ」は X14、`grammar/sakura-script.md:25` 等の `\]` は X08 で扱う。）

他項目で記録: `grammar/markers.md:35`（X01）、`grammar/sakura-script.md:40,47`（X08）、`grammar/action-line.md:108`（D02）。

対象外:
- `getting-started/first-ghost.md:186`・`grammar/action-line.md:54`・`grammar/markers.md:31`・`ga/references/action-line.md:34,133`・`ga/references/sakura-script.md:27`・`ga/references/pasta-toml.md:274`・`lc/references/runtime-api.md:339`・`ga/SKILL.md:208`（さくらスクリプトの `
` 改行タグ・改行コードの説明で、文字列リテラルのエスケープではない）。
- `grammar/sakura-script.md:11,18,57`・`grammar/markers.md:122`・`ga/references/sakura-script.md:14,47,54,73`（さくらスクリプトのエスケープ文字 `` と角括弧の扱い。`]` は X08）。
- `ga/references/action-line.md:29,106`・`ga/references/words.md:95`・`ga/SKILL.md:96`（`＠＠` エスケープ。正）。
- `grammar/actor-dictionary.md:142`・`lc/references/internal-modules.md:168,200`（bind 引数・SSP タグ引数のエスケープ）。
- `ga/SKILL.md:218`（「エスケープハッチ」という比喩）。

### D08 単語値

- grep: `空白区切り|空白で区切|空白区切`、`空白(は|が)?(区切|値)|2 ?つの値|引用符なし|hello world`
- 実装照合: pest:`words = { word ~ ( comma_sep ~ word )* ~ comma_sep? }`・`word_nofenced = @{ (!(comma_sep | "\r" | "\n") ~ ANY)+ }`。実測: `＠w：a b、c d、` の候補は `a b` と `c d`（末尾カンマ可）。
- 結論: 表どおり（読点／カンマ区切り。空白は値に含まれる）。なお、引用なしの値の後ろの `＃` は値に取り込まれる（付録 U06・バグ候補）。`＃` コメントを付けた単語定義の例はこの挙動に当たる。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/literals.md:46 | 引用符なし: 空白は区切り文字で文字列に含まれない | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/literals.md:56 | 空白を含む文字列は必ず引用符、`hello world` は 2 つの値（L58〜L61） | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/words.md:15 | 値は読点・全角コンマ・半角カンマで区切る | 正 | — |
| ga/SKILL.md:119 | 区切りは `、` `，` `,` のいずれか | 正 | — |
| ga/SKILL.md:129 | 例 `＠女性、水の妖精：水無灯里、アリス・キャロル　＃ 2キー…`（U06 により `＃` 以降が値に入る） | 訂正対象 | 4.2 |
| ga/references/authoring-patterns.md:313 | 例 `＠女性：水無灯里、アリス　＃ …`（同上） | 訂正対象 | 4.3 |
| ga/references/grammar-model.md:161 | 引用符なしの空白は区切り文字 | 生成で置換（4.1） | 4.1 |

（`grammar/action-line.md:81`・`grammar/action-line.md:89`・`ga/SKILL.md:94`・`ga/references/action-line.md:82`・`ga/references/action-line.md:100`・`ga/references/variables.md:102` はインライン要素の区切りで正。`grammar/call-jump.md:123` は D06。）

### D09 属性行の配置

- grep: `属性`
- 実装照合: pest:`global_scene_attr_line`（`global_scene_init` と `actor_scope_item` にだけ現れる）、`local_scene_line = { pad ~ local_marker ~ scene ~ or_comment_eol }`・`scene = _{ id ~ s ~ attrs? }`（ローカルは宣言行への付記のみ）、`file_attr_line`（`file_scope`）、gen:scope `generate_global_scene`（`_file_attrs` を受けるが未使用 `#[allow(unused_variables)]`）。実測: ローカルシーン宣言の次行の `＆k：v` はパースエラー、`・L＆k：v`・グローバルシーン初期部の `＆k：v`・ファイル先頭の `＆k：v` は受理され、生成 Lua に現れない。
- 結論: 表どおり。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/block-structure.md:30 | 属性定義: 処理は将来予定 | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/block-structure.md:66 | グローバルシーン内部構造 2.「属性行 0 個以上／処理は将来予定」 | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/block-structure.md:88 | ローカルシーンブロックは宣言行・属性行（0 個以上）… | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/block-structure.md:112 | 節「属性（将来変更あり）」: ローカルシーンの直後にも置ける（L126）、処理は将来予定（L129）（L114〜L129） | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/block-structure.md:148 | インデント判定の例 `  ＆author：Alice` | 正 | — |
| grammar/index.md:29 | 俯瞰図のグローバルシーン配下に属性行 | 正 | — |
| grammar/index.md:80 | 属性は処理が将来予定 | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/markers.md:18 | 属性: メタデータ（処理は将来予定） | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/literals.md:9 | リテラルは属性値などで使用 | 正 | — |
| ga/SKILL.md:50 | 属性 `＆`: メタデータ | 正 | — |
| ga/SKILL.md:113 | よくある間違い d「シーン定義直後→属性行、属性はシーン定義の直後にのみ」（ローカルは宣言行への付記のみ） | 訂正対象 | 4.2 |
| ga/SKILL.md:240 | 属性はシーン定義の直後にのみ配置可能（L241）（節見出し L237） | 訂正対象 | 4.2 |
| ga/references/grammar-model.md:165 | 節「属性の配置ルール」（L168 の規則、L172・L184 のファイルレベル属性を含む） | 生成で置換（4.1） | 4.1 |
| ga/references/grammar-model.md:101 | 構造の俯瞰でローカルシーン配下にも属性行（L101〜L115） | 生成で置換（4.1） | 4.1 |

（`grammar/call-jump.md:111`・`ga/references/call-spec.md:14`・`ga/references/call-spec.md:46`・`ga/references/call-spec.md:52` は D05。）

対象外:
- `ga/references/grammar-model.md:3,28,60,150,248`（章の概要・マーカー表・コロンの用途・型の使用箇所・行種表で、属性行の配置に触れない）。
- `grammar/markers.md:31`（「行属性」という別の語）。
- `lc/references/internal-modules.md:499,512`（Lua API の互換用引数 `attrs`）。

### D10 Lua ブロックのフェンス

- grep: `` ```lua|フェンス|バッククォート|Lua ?ブロック ``、`` バッククォート|```lua ?`|` ``` ` ``
- 実装照合: pest:`code_open = _{ PUSH("`"{3,}) ~ id? ~ eol }`（行頭のみ・`pad` を取らない）・`code_close = _{ POP ~ or_comment_eol }`。実測: ```` ````text ```` で開いた 4 本フェンスはグローバルシーンで受理・出力、インデントした ```` ```lua ```` はパースエラー。
- 結論: 表どおり（3 個以上のバッククォート＋任意識別子）。加えて、フェンスは行頭に置く（インデント不可）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/block-structure.md:22 | Lua ブロックは ```` ``` ```` / ```` ```lua ```` | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/block-structure.md:96 | 例のフェンス ```` ```lua ```` は行頭 | 正 | — |
| grammar/call-jump.md:83 | 例のフェンス ```` ```lua ```` は行頭 | 正 | — |
| ga/SKILL.md:226 | 例のフェンス ```` ```lua ```` は行頭（節見出し L216） | 正 | — |
| ga/references/authoring-patterns.md:144 | 例で Lua ブロックの後に `％` 行（L154）を置く（実測: 14:2 でパースエラー。`％` 行はブロックより前に置く）。Markdown のフェンスもここで食い違う（1.1 の申し送り） | 訂正対象 | 4.3 |
| grammar/actor-dictionary.md:156 | アクタースコープの例でフェンスをインデント（L156〜L161） | 訂正対象 | 2.3 |
| lua/dsl-vs-lua.md:42 | DSL に ```` ```lua ```` ブロックを埋め込める | 正 | — |
| lua/index.md:20 | `.pasta` 中の ```` ```lua ```` ブロック | 正 | — |
| lua/modules.md:10 | DSL 内の ```` ```lua ```` ブロック | 正 | — |
| lua/patterns.md:9 | DSL 内の ```` ```lua ```` ブロック | 正 | — |
| ga/SKILL.md:220 | グローバルシーン直下に ```` ```lua ```` 〜 ```` ``` ````（インデント不要） | 訂正対象 | 4.2 |
| lc/SKILL.md:32 | DSL 内の ```` ```lua ```` ブロック | 正 | — |
| lc/references/internal-modules.md:521 | ```` ```lua ```` ブロックの関数は `function SCENE.func_name(act)` | 正 | — |
| ga/references/grammar-model.md:130 | ```` ```lua ```` 〜 ```` ``` ````（インデント不要。L240 も同旨）（節 L128、L103〜L137 の構造説明を含む） | 生成で置換（4.1） | 4.1 |

他項目で記録: `grammar/block-structure.md:67,90,92`（X05）、`grammar/block-structure.md:105`（X03）。

対象外（Markdown 上の Lua コード例のフェンス開始行、または Lua ブロックへの一般的な言及で、DSL のフェンスの形・位置を述べない）: `grammar/call-jump.md:77`・`grammar/index.md:19`・`grammar/variables.md:88`・`lua/basics.md:21,48,61,76,110,124`・`lua/dsl-vs-lua.md:46,47`・`lua/modules.md:42,64,103,124,153,177,201`・`lua/patterns.md:26,43,57,72,90,142,181`・`ga/references/authoring-patterns.md:142`・`ga/references/sakura-script.md:77`・`ga/references/variables.md:157`・`lc/references/coding-conventions.md:22,34,55,86,106,157,169,189,203,238,261,268,279,291,303,318,333,347,359`・`lc/references/internal-modules.md:12,18,36,42,66,80,93,108,115,131,138,146,152,170,177,202,210,233,243,254,265,285,296,317,330,346,362,367,381,388,397,403,412,427,442,456,471,479,487,496,508,523,537,557,564,573,600,613,642,653,659,670,688,705,717,744,761,773,785`・`lc/references/runtime-api.md:4,12,22,41,65,78,91,105,122,132,143,153,165,199,230,258,264,299,342,372,403,415,431,443,468,480,492,504,516,528,562,593,604,615,626,639,675`・`lc/references/shiori-handlers.md:12,19,39,54,69,102,122,137,156,182,200,224,252,299,312,320,350,358,374,395,423`・`lc/references/testing-lint.md:11,19,30,56,84,93,126,138,149,157,186,258,277`・`lc/SKILL.md:6,36,100,101`。

### D11 キューコマンド

- grep: `キューコマンド|!select|！select|choice_timeout|dola`、`キュー`、`スキップ|コード生成`
- 実装照合: gen:scope `generate_local_scene_items`（`CueCommand` は `cmd.command == "select"` のときだけ `generate_choice_timeout`、他は出力しない）・`generate_choice_timeout`。実測: `!select(10)` → `act:choice_timeout(10)` → `\![set,choicetimeout,10000]`、`!emote(x)` は出力なし。
- 結論: 表どおり。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/markers.md:23 | キューコマンド: 演出キュー（dola 側で処理）（`!select` はランタイムが処理する） | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/index.md:35 | 俯瞰図のキューコマンド行 | 正 | — |
| getting-started/first-ghost.md:358 | `!select(10)` は選択待ちの設定（L339 の例を含む） | 正 | — |
| lua/patterns.md:78 | `act:choice_timeout(30)` | 正 | — |
| ga/SKILL.md:53 | キューコマンド: 演出キュー | 正 | — |
| ga/SKILL.md:273 | `!select(秒数)` で選択の制限時間を設定（L271・L285 の例を含む） | 正 | — |
| lc/SKILL.md:83 | `pasta.act` の主要 API に `act:choice_timeout()` | 正 | — |
| ga/references/grammar-model.md:31 | マーカー表のキューコマンド: 演出キュー | 生成で置換（4.1） | 4.1 |
| ga/references/authoring-patterns.md:322 | `!select(秒数)` でタイムアウト（L331 の例を含む） | 正 | — |
| lc/references/internal-modules.md:393 | `choice_timeout(seconds)`（L400〜L405） | 正 | — |
| ga/references/grammar-model.md:197 | 節「キューコマンド構文」（Lua 生成時の扱いの記述なし） | 生成で置換（4.1） | 4.1 |

他項目で記録: `grammar/markers.md:35`（X01）。

対象外: `ga/references/grammar-model.md:47`・`lc/references/internal-modules.md:311`・`lc/references/shiori-handlers.md:380,409,418`（空白の扱い・前方一致検索・時報の「スキップ」）、`ga/references/authoring-patterns.md:202`・`ga/SKILL.md:30`（LLM による「コード生成」）。

### D12 単語定義とさくらスクリプト

- grep: `単語.{0,15}さくら|さくら.{0,15}単語`、`使えない|使用できない|使用不可|含められない|書けない|不可`
- 実装照合: pest:`word = _{ string_literal | sakura_script | word_nofenced }`、gen:elem `generate_word_definition`。実測: `＠表情：\s[1]、\s[2]` の参照で `\s[1]` を出力（付録 U14 の制約つき）。
- 結論: 表どおり（単語値にさくらスクリプトを書ける）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/sakura-script.md:9 | さくらスクリプトは「アクション行内にインラインで埋め込む」（単語値の記述なし） | 追記対象 | 2.3 |
| ga/SKILL.md:81 | インライン要素としてさくらスクリプトを埋め込み可能（単語値の記述なし） | 正 | — |
| grammar/actor-dictionary.md:33 | アクター辞書の値に複数のさくらスクリプト値を指定できる | 正 | — |
| ga/references/sakura-script.md:62 | 「変数宣言行や単語定義行では使用不可」（L61） | 生成で置換（4.1） | 4.1 |

他項目で記録: `ga/SKILL.md:221`（X03）、`lua/modules.md:218`（X18）。

対象外: `grammar/action-line.md:128`（章末の権威的仕様引用。1.5 で削除）、`grammar/markers.md:51`・`grammar/sakura-script.md:13`・`lua/index.md:30`・`lua/modules.md:31`・`ga/references/variables.md:94`・`lc/references/internal-modules.md:171`・`lc/SKILL.md:52`（予約識別子・全角文字・他バージョンの Lua 資料・モジュールの可用性・アクション行内の代入・プロパティ名・DSL のハンドラ定義についての「不可」で、単語値とさくらスクリプトに触れない）。

### D13 REG ハンドラ

- grep: `function ?\(req\)|function\(req`、`req\.`
- 実装照合: ps:pasta/shiori/event/init.lua `EVENT.fire`（`handler(act)`。冒頭のハンドラシグネチャ注記 `function(act: ShioriAct) -> string`）、crates/pasta_shiori/src/lua_request.rs（`act.req` になる要求表に `id`・`reference`・`status` 等を設定）。戻り値の扱いは X19。
- 結論: 表どおり（`function(act)`、リクエストは `act.req.*`）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| lua/patterns.md:94 | `REG.OnBoot = function(req)`（L95 の `req.reference[0]` を含む） | 訂正対象 | 2.8 |
| lua/patterns.md:99 | `REG.OnClose = function(req)`（L100 を含む） | 訂正対象 | 2.8 |
| lua/patterns.md:108 | 「ハンドラは `req` を受け取る」とフィールド表 `req.id` 等（L110〜L115） | 訂正対象 | 2.8 |
| lua/dsl-vs-lua.md:37 | カスタムイベント処理は REG に登録 | 正 | — |
| lc/SKILL.md:84 | `REG.EventName = function(req) ... end` | 訂正対象 | 4.4 |
| lc/references/shiori-handlers.md:20 | 登録パターン `function(req)`（L103・L123・L138・L157・L201・L225・L253・L300 の例も同じ） | 生成で置換（4.1） | 2.6（移設時）・4.1 |
| lc/references/shiori-handlers.md:26 | 節「req パラメータ」: ハンドラ引数 `req` のフィールド表と `req.reference[N]` の例（L30〜L43） | 生成で置換（4.1） | 2.6（移設時）・4.1 |
| lc/references/shiori-handlers.md:100 | 各イベント節の Reference 表を `req.reference[N]` で表記（L100・L118〜L120・L135・L153・L172・L197・L221・L237・L249）と、例の `req.` 参照（L104・L124・L139・L154・L158・L173・L184・L198・L202・L203・L222・L238・L250・L254・L255）、フォールバックチェーン図の `REG[req.id]`・`SCENE.search(req.id)`（L274・L278） | 生成で置換（4.1） | 2.6（移設時）・4.1 |
| lc/references/shiori-handlers.md:326 | 仮想ディスパッチャ節の `act.req.status`・`act.req.date`（L328・L332・L351・L365・L386） | 正 | — |
| grammar/variables.md:162 | Reference10 以降は Lua から `act.req.reference[10]` で読む | 正 | — |
| ga/references/variables.md:180 | Reference10 以降は `act.req.reference[10]` で読む | 生成で置換（4.1） | 4.1 |

### D14 RES

- grep: `ok_with`、`RES\.`
- 実装照合: ps:pasta/shiori/res.lua（`RES.env`・`build`・`ok(value, dic)`・`no_content`・`not_enough`・`advice`・`bad_request`・`err(reason, dic)`・`warn(reason, dic)`。`ok_with` は無い。`ok` は値が nil／空文字列なら 204）。
- 結論: 表どおり。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| lua/patterns.md:124 | `RES.ok_with(headers)`: 200 OK＋複数ヘッダ | 訂正対象 | 2.8 |
| lua/patterns.md:123 | `RES.ok(value)`: 200 OK＋さくらスクリプト | 正 | — |
| lua/patterns.md:126 | `RES.err(message)`: 500 | 正 | — |
| lua/patterns.md:125 | `RES.no_content()`: 204 No Content | 正 | — |
| lc/references/shiori-handlers.md:63 | API 一覧の `RES.ok_with(headers)`（API 一覧 L62〜L65、L71〜L77 の説明を含む） | 生成で置換（4.1） | 2.6（移設時）・4.1 |
| lc/references/shiori-handlers.md:80 | 使用例 `return RES.ok_with({…})` | 生成で置換（4.1） | 2.6（移設時）・4.1 |

他項目で記録: `lua/patterns.md:96,102,104`・`lc/SKILL.md:85,148`・`lc/references/shiori-handlers.md:22,74,106,108,125,141,143,159,205,207,226,257,259`（X19。ハンドラから `RES.*` を返す例）、`lc/references/shiori-handlers.md:296`（D15）。

### D15 シーン関数フォールバック

- grep: `204|pcall|xpcall`、`フォールバック`
- 実装照合: ps:pasta/shiori/event/init.lua `EVENT.no_entry`（`SCENE.co_exec(act, act.req.id, nil, nil)`）・`EVENT.fire`（thread を resume して `RES.ok(yielded_value)`、nil なら `RES.no_content()`）、ps:pasta/shiori/entry.lua `SHIORI.request`（`xpcall` → `RES.err(result)`）。実測: `＊OnTest` を定義して OnTest → `200 OK`。
- 結論: 表どおり（見つかれば 200、見つからなければ 204。保護は entry.lua の xpcall で、フォールバック専用の pcall は無い）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| lua/patterns.md:128 | 節「REG 未登録時のフォールバック」（応答コードの記述なし） | 正 | — |
| lua/dsl-vs-lua.md:29 | ランタイムのフォールバックが自動で呼び出す | 正 | — |
| getting-started/first-ghost.md:148 | 「シーン関数フォールバック」機能を利用 | 正 | — |
| lc/references/shiori-handlers.md:279 | フォールバックチェーン「見つかった → シーン関数実行 → 204 No Content」（節 L265・L269、L274〜L280 の図を含む。L280 の「見つからない → 204」は正） | 生成で置換（4.1） | 2.6（移設時）・4.1 |
| lc/references/shiori-handlers.md:297 | シーン関数フォールバック時も `pcall` でキャッチ | 生成で置換（4.1） | 2.6（移設時）・4.1 |
| lc/references/shiori-handlers.md:296 | REG ハンドラの例外は `xpcall` でキャッチされ `RES.err()` | 生成で置換（4.1） | 2.6（移設時）・4.1 |
| lc/references/shiori-handlers.md:179 | OnChoiceSelectEx で見つからなければ nil（204） | 生成で置換（4.1） | 4.1 |

（`lua/modules.md:26`・`lua/modules.md:101`・`lua/modules.md:104`・`lc/SKILL.md:71`・`lc/references/internal-modules.md:78`・`lc/references/coding-conventions.md:329` 等の `pcall(require, "@pasta_config")` は別件で正。）

他項目で記録: `lua/modules.md:119`・`lc/references/runtime-api.md:20,37,38,48,63`・`lc/references/internal-modules.md:263,271,274,294,303,306,449`・`lc/SKILL.md:132`・`grammar/actor-dictionary.md:52`・`ga/SKILL.md:186`（D03）、`ga/references/actor-dictionary.md:55`（D04）、`lc/SKILL.md:148`（D13・X19）、`lua/patterns.md:125`・`lc/references/shiori-handlers.md:64`（D14）。

対象外:
- `lc/references/coding-conventions.md:334,406`・`lc/references/internal-modules.md:81,767`・`lc/references/runtime-api.md:200,203,231`・`lc/references/testing-lint.md:52,54,58`・`lc/SKILL.md:113`（`pcall(require, "@pasta_config")` やテスト・規約の `pcall` で、シーン関数フォールバックの保護ではない）。
- `getting-started/first-ghost.md:241,249,272`・`ga/references/authoring-patterns.md:69,126`・`lc/references/shiori-handlers.md:4,370,381,389`・`ga/SKILL.md:342,356,362,375`（時報の 4 段フォールバック、イベントのシーン関数フォールバックの紹介。応答コードと保護に触れない）。
- `grammar/actor-dictionary.md:65,121`・`ga/references/variables.md:216,219`・`debug/source-level.md:61`・`lc/references/internal-modules.md:742`・`lc/references/runtime-api.md:544,546`（既定値・文字列化・バッファのフォールバック）。
- `lua/dsl-vs-lua.md:37`（標準フォールバックで扱えないイベントは REG へ、という方針の説明）。

### D16 OnSecondChange・コールバック

- grep: `resume_pending|CALLBACK\.|sweep`、`OnSecondChange|仮想ディスパッチャ|OnNotifyCallbackResponse|コールバック`
- 実装照合: ps:pasta/shiori/event/second_change.lua（既定の `REG.OnSecondChange` が `CALLBACK.sweep(os.time())` → `dispatcher.dispatch(act)`）、ps:pasta/shiori/event/callback.lua（`next_event_id`・`stage_pending`・`consume_staged`・`try_route`・`sweep`・`reset`。`resume_pending` は無い）、ps:pasta/shiori/event/virtual_dispatcher.lua（OnTalk・OnHour の発行）。
- 結論: 表どおり（`REG.OnSecondChange` を上書きすると OnTalk・OnHour とコールバックのタイムアウト処理が止まる）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| lua/patterns.md:134 | OnTalk・OnHour は仮想ディスパッチャが OnSecondChange を起点に発行（L135） | 正 | — |
| getting-started/first-ghost.md:196 | OnSecondChange → 仮想イベントディスパッチャ → ランダムトーク／時報 | 正 | — |
| lua/dsl-vs-lua.md:29 | 仮想ディスパッチャが自動で呼び出す | 正 | — |
| lc/references/shiori-handlers.md:217 | 「コールバック保留中のコルーチン再開もこのイベント（`CALLBACK.resume_pending()`）」（節 L214・L216） | 生成で置換（4.1） | 2.6（移設時）・4.1 |
| lc/references/shiori-handlers.md:225 | 上書き例 `REG.OnSecondChange = function(req) return RES.no_content() end`（上書きの注意なし） | 生成で置換（4.1） | 2.6（移設時）・4.1 |
| lc/references/shiori-handlers.md:310 | 仮想ディスパッチャは OnSecondChange をトリガーに OnTalk／OnHour を発行（L308・L318 を含む） | 生成で置換（4.1） | 4.1 |
| lc/references/internal-modules.md:190 | `get_property` はコールバック到着まで待機（L193 のトークンバッファ保全を含む） | 正 | — |

（`lc/references/shiori-handlers.md:230` の OnNotifyCallbackResponse は X20。）

他項目で記録: `lc/references/shiori-handlers.md:232,237,241`（X20）、`lc/SKILL.md:148`（D13・X19）。

対象外: `getting-started/first-ghost.md:341,347`（選択肢の飛び先シーンを「コールバックシーン」と呼ぶ例のコメント）、`lc/references/shiori-handlers.md:4,403`・`lc/SKILL.md:170`（章の概要・`[ghost]` 設定の前置き・references 一覧の用途欄）。

### D17 さくらスクリプト変換のウェイト

- grep: `script_wait|ウェイト|talk\.|actor\.talk|\[talk\]`
- 実装照合: lua:sakura_script/mod.rs `resolve_wait_values`（アクター表直下の `script_wait_normal`・`script_wait_period`・`script_wait_comma`・`script_wait_strong`・`script_wait_leader` → `[talk]` → 既定の 3 段）、lua:loader/config/sections.rs `TalkConfig` の `Default`（50・1000・500・500・200）、lua:sakura_script/wait_inserter.rs（挿入値は `値 - 50`、0 以下は挿入しない、連続句読点は `max`）。`chars_no_wait`・`chars_half_wait`・`script_wait_default`・`script_wait_newline`・`script_wait_exclamation` は実装に無い。
- 結論: 表どおり。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| lua/modules.md:156 | `local actor = { talk = {} }  -- talk サブテーブルにウェイト設定` | 訂正対象 | 2.5 |
| lua/modules.md:158 | 結果例 `こ\_w[50]ん…は\_w[100]。`（既定では通常文字に挿入されない） | 訂正対象 | 2.5 |
| lua/modules.md:161 | `actor.talk` の `script_wait_default` = 50ms、`script_wait_period` = 100ms（L162 を含む） | 訂正対象 | 2.5 |
| lua/modules.md:150 | セリフにウェイトタグ `\_w[ms]` を自動挿入 | 正 | — |
| lua/patterns.md:67 | `wait(ms)`（ウェイト） | 正 | — |
| ga/SKILL.md:209 | `\w数字`: 数字×50ms | 正 | — |
| ga/SKILL.md:315 | `[talk]`: ウェイト・禁則処理のカスタマイズ | 正 | — |
| ga/references/pasta-toml.md:120 | `[talk]` の既定 50/1000/500/500/200（L317〜L321 の表も同じ）（L46・L119〜L124・L311〜L321・L332〜L335） | 正 | — |
| lc/references/runtime-api.md:270 | `actor` は `talk` サブテーブルにウェイト設定（L273〜L295 の表・処理説明、L308・L313 の例、L328〜L334 の `[talk]` 例を含む）（L256 の冒頭、L321 のハーフウェイト例、L394 の break_lines 例も含む） | 生成で置換（4.1） | 2.5（移設時）・4.1 |
| lc/references/internal-modules.md:129 | `talk` は `@pasta_sakura_script` でウェイトタグ付きに変換 | 正 | — |

他項目で記録: `lc/SKILL.md:132`（D03）。

対象外: `getting-started/first-ghost.md:28,190,192,195`・`ga/references/authoring-patterns.md:49,67,244`・`ga/SKILL.md:374,378`（ファイル名 `talk.pasta`）、`getting-started/first-ghost.md:399`（`[talk]` セクションは省略可。正）、`grammar/sakura-script.md:37`・`ga/references/sakura-script.md:28,29`・`ga/SKILL.md:210`（さくらスクリプトのウェイトタグ `w`・`_w`）、`lc/references/internal-modules.md:144,229`（`act` の出力メソッドと `wait(ms)`。ウェイト変換の設定値に触れない）。

### D18 pasta.toml の既定値の出典

- grep: `config\.rs|loader/config|config/mod\.rs|sections\.rs`、`crates/|\.rs\b`
- 実装照合: lua:loader/config/mod.rs（`PastaConfig`）・lua:loader/config/sections.rs（各セクションの `Default` と `default_*()`）。`crates/pasta_lua/src/loader/config.rs` は存在しない。
- 結論: 表どおり。マニュアルにはリポジトリ内パスを書かず「実装の既定値（SSOT）」とだけ記す（5.3）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| ga/references/pasta-toml.md:29 | 既定値は「Rust `crates/pasta_lua/src/loader/config.rs` の `Default` 実装・`default_*()` 関数」由来 | 生成で置換（4.1） | 2.7（移設時）・4.1 |
| ga/references/call-spec.md:116 | HTML コメント `source: doc/spec/04-call-spec.md, crates/pasta_dsl/src/parser/grammar.pest` | 生成で置換（4.1） | 4.1 |
| lc/references/testing-lint.md:247 | `crates/pasta_lua/scriptlibs/lua_test/mocks.lua`（既定値の出典ではないが D18 の grep に当たったリポジトリ内パス。SkillLayout でモジュール名表記へ直す対象） | 訂正対象 | 4.4 |

（`getting-started/first-ghost.md:421`・`getting-started/first-ghost.md:433` は GitHub の README への絶対 URL で対象外。）

対象外: `getting-started/first-ghost.md:91,415`・`getting-started/prerequisites.md:34`（GitHub 上のサンプルゴーストへの絶対 URL に `crates/` が含まれるだけ）。

### X01 空白の文字クラス

- grep: `White_Space|WHITE_SPACE|空白文字クラス|Unicode.{0,10}空白`
- 実装照合: pest:`space_chars`（U+0020・タブ・U+3000・U+00A0・U+1680・U+2000〜U+200A・U+202F・U+205F の列挙）。
- 結論: 台帳本体の備考どおり。Unicode White_Space 全体ではなく列挙した文字だけ（U+000B・U+000C・U+0085・U+2028・U+2029 は空白でない）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/markers.md:35 | Unicode の White_Space カテゴリから改行を除いたもの | 訂正対象・訂正済み（2.1） | 2.1 |
| ga/references/grammar-model.md:45 | 空白文字クラスの列挙（L50 で U+00A0 等を列挙） | 生成で置換（4.1） | 4.1 |

### X02 変数代入のコロン形

- grep: `(＄|\$)(＊|\*)?[^ 　＝=（）()]{1,20}(：|:)`
- 実装照合: pest:`set_marker = _{ equals }`・`set = _{ set_marker ~ s ~ ( expr | word_ref ) }`。実測: `＄x：1` はパースエラー（`expected EOI, id, var_set_…`）。
- 結論: 台帳本体の備考どおり（代入は `＝` のみ）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/markers.md:42 | コロンの用途例 `＄var_name：value # 変数代入` | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/call-jump.md:44 | `＄target：挨拶朝` | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/call-jump.md:81 | `＄スコア：75` | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/literals.md:37 | `＄is_active：true`・`＄done：false`（L38） | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/literals.md:50 | `＄greeting：「こんにちは」`（L53 の `＄message："…"` を含む） | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/literals.md:78 | `＄count：10` ほか（L79〜L81） | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/action-line.md:63 | 変数の宣言・代入（`＄var：value`）は変数代入行で行う | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/variables.md:19 | 代入の区切りはコロンでも `＝` でも記述できる（L23〜L25 の例を含む） | 訂正対象 | 2.3 |
| grammar/variables.md:62 | 例 `＄ユーザー：太郎` | 訂正対象 | 2.3 |
| ga/SKILL.md:142 | 代入: `＄変数名＝値` または `＄変数名：値` | 訂正対象 | 4.2 |
| ga/references/variables.md:33 | `＄変数名：値 ← コロン形式` | 生成で置換（4.1） | 4.1 |
| ga/references/variables.md:46 | `＄sum：＠add（…）` | 生成で置換（4.1） | 4.1 |

### X03 Lua ブロックの「関数定義のみ・トランスパイラが検証」

- grep: `関数定義のみ|関数定義だけ|トランスパイラ.{0,10}検証|__start__.{0,20}(最初|先頭)`
- 実装照合: gen:elem `generate_code_block`（内容を 1 行ずつ無変換で出力。検証なし）。実測: Lua ブロック内の `local REG = require(…)` と代入文がロード時に実行され、REG に登録された（X19 の実測）。
- 結論: 台帳本体の備考どおり。関数定義以外の文も書け、検証されない（誤りは Lua のロード時エラーになる）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/block-structure.md:107 | 関数定義のみ許可、変数宣言・トップレベルの文は不可 | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/block-structure.md:108 | トランスパイラ層が検証 | 訂正対象・訂正済み（2.1） | 2.1 |
| ga/SKILL.md:221 | 関数定義のみ許可（変数宣言やステートメントは不可） | 訂正対象 | 4.2 |
| ga/references/grammar-model.md:131 | 関数定義のみ許可、トランスパイラーが検証（L132〜L133） | 生成で置換（4.1） | 4.1 |

### X04 比較演算子

- grep: `比較演算子|＝＝|==|！＝|<=|＜＝`
- 実装照合: pest:`bin_op = _{ add_op | sub_op | mul_op | div_op | modulo_op }`（`gt`・`lt` は `call_marker` 等で使われ、式の演算子ではない）。
- 結論: 台帳本体の備考どおり（式に比較演算子は無い）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/markers.md:39 | 比較・フィルター条件には `＝` `＞` `＜` などの比較演算子を用いる | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/markers.md:85 | 節「比較演算子」の表（L87〜L96） | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/call-jump.md:117 | フィルターは比較演算子を使う（L119 を含む） | 訂正対象・訂正済み（2.2） | 2.2（節削除） |
| lua/basics.md:103 | Lua の等値比較は `==`、非等値は `~=` | 正 | — |

（`ga/references/authoring-patterns.md:148`・`lc/references/*` の `==` は Lua コード内で対象外。）

### X05 Lua ブロックの出力位置と評価時期

- grep: `最初に実行|ロード時|読み込み時に(実行|評価)`、`__start__.{0,20}(最初|先頭)`、`暗黙ローカル開始ブロック`
- 実装照合: gen:scope `generate_global_scene`（グローバルシーンと各ローカルシーンの Lua ブロックを、全シーン関数の定義の後にシーンの `do` ブロック直下へ出力）。実測: グローバルシーン末尾の ```` ````text ```` ブロックが `SCENE.__start__` の後に出力され、ロード時に評価された。
- 結論: 台帳本体の備考どおり。Lua ブロックは `__start__` の中ではなく、ロード時に評価される。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/block-structure.md:67 | `__start__` に Lua ブロック・アクション行…を格納できる | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/block-structure.md:92 | Lua ブロックは `__start__` 内に置かれる | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/block-structure.md:102 | 例のアクターなし行 `    こんにちは`（実測: 9:5 でパースエラー） | 訂正対象・訂正済み（2.1） | 2.1 |
| ga/references/grammar-model.md:128 | 節「Luaブロック配置ルール」 | 生成で置換（4.1） | 4.1 |

### X06 動的 Call の `tostring` と nil ガード

- grep: `動的ターゲット|＞＄|＞＠|tostring|nil ?ガード`
- 実装照合: gen:elem `generate_call_scene`（`Dynamic` → `act:call(SCENE.__global_name__, tostring(式), {}, …)`）、ps:pasta/act.lua `ACT_IMPL.call`（`key == nil` で警告し nil）。実測: 未代入の `＞＄t` は `tostring(var.t)` で `"nil"` を検索し「handler not found」警告、後続行は続行。
- 結論: 台帳本体の備考どおり。DSL からの呼び出しでは nil ガードに届かない（nil ガードは Lua から `act:call` を直接呼ぶ場合だけ）。ターゲットは変数に限らず任意の式。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/call-jump.md:32 | 動的ターゲット `＞＄変数名`（変数のみと読める） | 追記対象・訂正済み（2.2） | 2.2 |
| grammar/call-jump.md:99 | `＞＠分岐判定`（関数内で `act:call` 済み、戻り値 nil → `"nil"` で再検索し警告） | 訂正対象・訂正済み（2.2） | 2.2 |
| ga/references/call-spec.md:86 | 式評価結果を `tostring()` でシーン名に変換 | 生成で置換（4.1） | 4.1 |
| ga/references/call-spec.md:112 | 節「nil ガード」: 式評価結果が nil なら即時リターン（L114） | 生成で置換（4.1）・移設時訂正済み（2.2） | 2.2（移設時）・4.1 |
| lc/references/internal-modules.md:355 | `act:call` の nil ガード（Lua からの直接呼び出しとして正しい） | 正 | — |

### X07 真偽値リテラル

- grep: `真偽|bool|\btrue\b|\bfalse\b`
- 実装照合: pest:`term`（`paren_expr`・`fn_call`・`var_ref`・`number_literal`・`string_literal` のみ）、`attr_value`（数値・引用文字列・引用なし文字列）、dsl:parser/ast/action.rs `AttrValue`。実測: `＄x＝true` はパースエラー（`expected paren_expr, …`）。
- 結論: 台帳本体の備考どおり（真偽値リテラルは無い）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/literals.md:15 | 型変換ルール 1「`true` / `false` → bool」（L24〜L25） | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/literals.md:32 | 節「真偽値（bool）」（L34〜L38） | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/literals.md:3 | 導入「真偽なのか」 | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/variables.md:25 | 例 `＄is_active：true` | 訂正対象 | 2.3 |
| grammar/index.md:69 | 章一覧「型変換ルール・文字列・数値・真偽値」 | 訂正対象・訂正済み（2.1） | 2.1 |
| grammar/block-structure.md:99 | Lua ブロック内の `save.talked = true`（Lua の真偽値） | 正 | — |
| ga/references/grammar-model.md:154 | 型変換表の `true` / `false` → bool | 生成で置換（4.1） | 4.1 |

（`lua/basics.md:36`・`lua/modules.md:71`・`reference/startup.md:122` 等は Lua・TOML・ログの真偽値で対象外。）

### X08 さくらスクリプト角括弧内の `\]`

- grep: `エスケープ`、`\\\]`
- 実装照合: pest:`sakura_args`・`sakura_body = @{ ( sakura_str | (!PEEK ~ ANY) )* }`（`"…"` の外では最初の `]` で閉じる）・`sakura_str`（`"…"` 内の `""`）、lua:sakura_script/tokenizer.rs `SAKURA_TAG_PATTERN`。実測: `さくら：\s[a\]b]` はさくらスクリプト `\s[a\]` と台詞 `b]` に分かれる（出力の文字列は同じでも、`b]` は台詞としてウェイト挿入の対象になる）。
- 結論: 台帳本体の備考どおり（`\]` は特別扱いされない）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/sakura-script.md:21 | 字句規則 `bracket_chars` が `"\]"` を内容文字の選択肢に含める | 訂正対象 | 2.3 |
| grammar/sakura-script.md:25 | 最初の「非エスケープな `]`」で閉じ、`\]` は `]` を表す | 訂正対象 | 2.3 |
| grammar/sakura-script.md:40 | 例「ブラケットエスケープ」`\s[a\]b]`（L41） | 訂正対象 | 2.3 |
| grammar/sakura-script.md:51 | `\]` で `]` を文字として含める | 訂正対象 | 2.3 |
| grammar/sakura-script.md:52 | カンマを含む値は `"` で囲む、`"` は二重にする（L53） | 正 | — |
| ga/references/sakura-script.md:54 | 最初の非エスケープ `]` で閉じる（L44 の字句構造を含む） | 生成で置換（4.1） | 4.1 |
| lc/references/internal-modules.md:168 | `set_property` 等の引数を SSP 規則で `]` → `\]` に自動エスケープ（SSP へ出す文字列の話で、DSL の字句ではない） | 正 | — |

### X09 ローカル変数の有効範囲

- grep: `一連のシーン|イベントごと|イベント処理ごと`
- 実装照合: ps:pasta/act.lua `ACT.new`（`var = {}`）・`ACT_IMPL.init_scene`（`self.var` を返す）・`ACT_IMPL.yield`（再開後も同じ `self` を返す）、ps:pasta/shiori/event/init.lua `create_act`（イベントごとに新しい act）。
- 結論: 台帳本体の申し送り「『一連のシーン』の範囲」は次で確定: `var` は act ごとの空テーブルで、イベント処理ごとに新しく作られる。Call 先のシーンは同じ act を使う。`act:yield()` で中断したシーン（チェイントーク）は次のイベントで再開しても最初の act を使い続けるため、そのシーンのコルーチンが終わるまで値が残る。「一連のシーンが終わるまで」は誤りではないが範囲が曖昧なので、この内容で明確化する。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/variables.md:15 | ローカル変数の有効範囲「一連のシーンが終わるまで」 | 追記対象 | 2.3 |
| grammar/variables.md:30 | 同上 | 追記対象 | 2.3 |
| ga/SKILL.md:139 | 同上 | 正 | — |
| ga/references/variables.md:12 | 同上（L17 を含む） | 生成で置換（4.1） | 4.1 |

### X10 Lua 関数の定義形（`function add(ctx, x, y)`）

- grep: `function [A-Za-z_]+\((ctx|req)|\(ctx`
- 実装照合: ps:pasta/act.lua `find_act_handler`（L1 シーン表・L3 act・L4 GLOBAL を探し、素の Lua グローバル関数は探さない）、gen:elem（ローカル関数呼び出しは `act:expr_fn`）。
- 結論: 台帳本体の備考どおり。対象範囲にヒットなし（`function add(ctx, x, y)` は撤去される doc/spec ch09 にだけある）。マニュアルの例は `function SCENE.add(act, x, y)` の形で書く（grammar/variables.md#式（Expression）のサポート、2.3）。

ヒットなし。

### X11 グローバル単語・グローバルシーンの参照範囲

- grep: `ファイル全体|ファイル内|全辞書|全ファイル`
- 実装照合: gen:elem `generate_global_word`（`PASTA.create_word`）→ ps:pasta/word.lua `WORD.create_global`（`STORE.global_words` は全ファイル共通の 1 つ）、ps:pasta/scene.lua（グローバルシーンも全ファイル共通の登録）。
- 結論: 台帳本体の備考どおり（参照範囲は全辞書ファイル共通）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/words.md:15 | グローバル単語は「ファイル全体から参照できる」 | 訂正対象 | 2.3 |
| grammar/words.md:84 | グローバル単語辞書は「ファイル先頭で定義された単語」 | 訂正対象 | 2.3 |
| grammar/block-structure.md:20 | グローバル単語定義は「ファイル全体で参照可能」 | 訂正対象・訂正済み（2.1） | 2.1 |
| ga/SKILL.md:63 | グローバルシーンは「ファイル全体からアクセス可能」 | 訂正対象 | 4.2 |
| ga/SKILL.md:121 | グローバル単語は「ファイル全体から参照可能」 | 訂正対象 | 4.2 |
| ga/references/authoring-patterns.md:242 | `actors.pasta` は全ファイルで共有 | 正 | — |
| ga/references/words.md:40 | グローバル単語の範囲「ファイル全体」 | 生成で置換（4.1） | 4.1 |
| ga/references/grammar-model.md:186 | ファイルレベル属性はファイル内のグローバルシーンに継承 | 生成で置換（4.1） | 4.1 |

### X12 アクタースコープの Lua ブロック

- grep: `コードブロック|ACTOR\.`、`将来変更あり`
- 実装照合: pest:`actor_scope_item`（`code_scope` を含む）、gen:scope `generate_actor`（識別子が `lua` のブロックだけをアクターの `do` ブロック内に出力し、`ACTOR` を使える）、ps:pasta/actor.lua `find_actor_handler`（A1 でアクター表の `ACTOR.名前` を返す）。実測: `％さくら` 直下の ```` ```lua ```` で `function ACTOR.挨拶(act) return "ACT" end` を定義し、`さくら：＠挨拶` → `ACT`。```` ```text ```` のブロックは出力されない。
- 結論: 台帳本体の備考どおり。design #14 の「受理されるが処理に反映されない」とは食い違い、実装は `lua` ブロックを出力して A1 で到達する（実装が正）。現行挙動の書き方は 1.4 の仕分けで確定する。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/actor-dictionary.md:149 | 節「コードブロック（将来変更あり）」 | 訂正対象 | 2.3 |
| grammar/actor-dictionary.md:151 | 構文は定義されているが使用例がなく将来の拡張として予約 | 訂正対象 | 2.3 |
| grammar/actor-dictionary.md:157 | 例の `function SCENE.on_actor_event(act)`（アクターの `do` ブロックに `SCENE` は無い。フェンスのインデントは D10） | 訂正対象 | 2.3 |
| grammar/actor-dictionary.md:164 | 「文法定義のみが存在し、未実装」 | 訂正対象 | 2.3 |

### X13 ファイルエンコーディングと BOM

- grep: `BOM|エンコーディング|UTF-8|文字コード`
- 実装照合: lua:loader/process.rs（`fs::read_to_string` で UTF-8 として読む）→ dsl:parser/mod.rs `parse_str`（BOM を除く処理なし）、pest:`file = _{ SOI ~ … }`（U+FEFF を受ける規則なし）。実測: 先頭に BOM のある `.pasta` は 1:1 でパースエラー。
- 結論: 台帳本体の申し送り「BOM は許容」は実測で否定された。BOM 付き UTF-8 は読み込めない（BOM なし UTF-8 で保存する）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| getting-started/prerequisites.md:36 | 節「文字コードは必ず UTF-8」（L38〜L50。BOM の記述なし） | 追記対象 | 2.8 |
| getting-started/first-ghost.md:439 | 文字化けするときは UTF-8 で保存されているか確認 | 正 | — |
| lua/basics.md:54 | 辞書・スクリプトはすべて UTF-8 | 正 | — |

（`reference/startup.md:50` 等の `package.path` の UTF-8、`lua/modules.md:172` の `@enc`、`ga/SKILL.md:331` の descript.txt の charset は別件で対象外。）

### X14 囲みの多重化（`「「セリフ」」`）

- grep: `「「`、`引用符エスケープ`
- 実装照合: pest:`strfence`（`slfence_ja4`〜`slfence_ja1` の順に試す）・`slfence_ja2 = _{ "「"{2} ~ PUSH_LITERAL("」」") }`・`string_contents`。実測: `＠w：「「セリフ」」` の値は `セリフ`（括弧を含まない）。
- 結論: 台帳本体の備考どおり。多重の囲みは、中身に `」` を含められるようにするためのもので、値に括弧は残らない。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/literals.md:86 | 引用符を含めたい場合は二重の括弧、`「「セリフ」」 # 実行時に「セリフ」として展開`（L89） | 訂正対象・訂正済み（2.2） | 2.2 |
| grammar/words.md:139 | 単語値に引用符を含めるには二重の括弧（L142・L145「内側の `「」` がリテラルの引用符として保持」） | 訂正対象 | 2.3 |

### X15 `％` 行と立ち位置の持続

- grep: `立ち位置|set_spot|clear_spot|スポット.{0,10}(保持|維持|固定|リセット)`、`％`
- 実装照合: gen:scope `generate_local_scene`（`％` 行 → `act:clear_spot()`＋`act:set_spot(名前, 番号)`）、ps:pasta/act.lua `set_spot`・`clear_spot`（トークンを積むだけ）、ps:pasta/shiori/sakura_builder.lua `BUILDER.build`（`STORE.actor_spots` を直接書き換え）、ps:pasta/store.lua（`STORE.actor_spots` はモジュール状態で、初期値は pasta.toml `[actor]` の `spot`）。
- 結論: 立ち位置は次の `％` 行が実行されるまでイベントをまたいで保たれる（`％` 行が無いシーンは直前の立ち位置で話す）。アクター付き単語参照は `％` 行が無くても働く（ps:pasta/actor.lua）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| grammar/actor-dictionary.md:37 | `％actor1、actor2` で「そのシーン内の会話行でアクター名付き単語参照が有効になる」（立ち位置の設定であり、持続の記述がない） | 訂正対象 | 2.3 |
| grammar/actor-dictionary.md:46 | `actor_name：` で始まる会話行はアクターの単語辞書を優先 | 正 | — |
| ga/references/actor-dictionary.md:38 | `％名前1、名前2` で「そのシーンでバルーン連動が有効」 | 生成で置換（4.1） | 2.3（移設時）・4.1 |
| lc/references/internal-modules.md:239 | `set_spot(name, number)`（L250 の `clear_spot()` を含む） | 正 | — |

### X16 `default_surface`（main 取り込み後の再照合）

- grep: `default_surface`、`surface|dressup`
- 実装照合: ps:pasta/shiori/appearance.lua（局所関数 `default_surface(actor)` が `actor.surface` を読む。`dressup` も同モジュールで読む）、ps:pasta/shiori/sakura_builder.lua `emit_actor_switch`（`\p[spot]` の直後に `APPEARANCE.restore`）。pasta.toml の `default_surface` キーを読むコードは無い。
- 結論: main 取り込み（actor-surface-restore）で `ga/references/pasta-toml.md` の `default_surface` 節は `#### surface / dressup` に置き換わり、実装と一致した（「旧資料の `default_surface` は実装されていない」と注記あり）。台帳本体の該当行（`pasta-toml.md` L290）の「収録先: reference/pasta-toml.md#default_surface」は、2.7 で `surface / dressup` 節として収録する。book に `default_surface` のヒットは無い。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| ga/references/pasta-toml.md:291 | 節「surface / dressup」（既定値、復旧時のみ出力） | 生成で置換（4.1） | 2.7（移設時）・4.1 |
| ga/references/pasta-toml.md:307 | `default_surface` は実装されていない旨の注記 | 生成で置換（4.1） | 2.7（移設時）・4.1 |
| grammar/actor-dictionary.md:63 | 節「同一スポット共有時の外見の復旧」（main 取り込みで追加） | 正 | — |
| grammar/actor-dictionary.md:100 | 節「任意キー `surface`・`dressup`」 | 正 | — |

### X17 `@pasta_config` の「読み取り専用」

- grep: `読み取り専用|read-?only|書き換え`
- 実装照合: lua:runtime/module_registry.rs `register_config_module`（doc コメントは read-only だが、`toml_to_lua` が作る普通のテーブルを `inject_actor_names` で書き換えたうえで登録）。実測: `require("@pasta_config").extra = 1` の後、再度 require した表で `extra` が 1。
- 結論: 台帳本体の備考どおり。読み取り専用は強制されない（書き換えても pasta.toml には反映されない）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| lua/modules.md:113 | 値は読み取り専用 | 訂正対象 | 2.4 |
| lc/references/runtime-api.md:246 | 注意事項「読み取り専用: 値の変更はできない」 | 生成で置換（4.1） | 2.4（移設時）・4.1 |

### X18 `@env` の有効化手段（`[lua] libs` は適用されない）

- grep: `RuntimeConfig`、`\[lua\]|libs`
- 実装照合: lua:loader/mod.rs `PastaLoader::load`（`load_with_config(base_dir, RuntimeConfig::new())`）、crates/pasta_shiori/src/shiori.rs（SHIORI のロードも `RuntimeConfig::new()` を渡す）、lua:runtime/runtime_config.rs（`From<LuaConfig> for RuntimeConfig` はあるがロード経路から呼ばれない）、lua:loader/config/mod.rs `PastaConfig::lua`（呼び出し元なし）。実測: pasta.toml に `[lua] libs = ["std_all", "env"]` を書いても `require "@env"` は失敗し、`@json` は読める（既定のまま）。
- 結論: 台帳本体の備考（「有効化手段は pasta.toml `[lua] libs` への `"env"` 追加として書く」）は実装で否定された。現行実装では pasta.toml の `[lua]` セクションはロード時に読まれず、ゴーストから `@env` を有効にする手段は無い。マニュアル（2.5・2.7）は `[lua] libs` が効くとは書かない。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| lua/modules.md:218 | `@env` はデフォルト無効、Rust 側の `RuntimeConfig` 設定が必要で通常のゴーストからは使えない | 正 | — |
| lua/modules.md:38 | `@pasta_log` は `RuntimeConfig` に関わらず利用可能 | 正 | — |
| lc/references/runtime-api.md:655 | 有効化は Rust 側の `RuntimeConfig`（L658〜L672 と L681〜L694 の Rust コード例） | 生成で置換（4.1） | 2.5（移設時）・4.1 |
| lc/references/runtime-api.md:703 | モジュールの有効／無効は `libs` 配列 | 生成で置換（4.1） | 2.5（移設時）・4.1 |
| ga/references/pasta-toml.md:377 | 節「[lua]（Lua ライブラリ）」の `libs`（L49・L145〜L146・L383〜L387 を含む） | 生成で置換（4.1） | 2.7（移設時）・4.1 |

### X19 REG ハンドラの戻り値（`RES.ok` の二重包み）

- grep: `return RES\.`、`RES\.ok\(\)|RES\.no_content\(\)`
- 実装照合: ps:pasta/shiori/event/init.lua `EVENT.fire`（戻り値が文字列なら `RES.ok(result)`、thread なら resume して `RES.ok(値)`、nil なら `RES.no_content()`）。同じファイルの冒頭の使用例は `return RES.ok(act:build())`、`elseif` 節のコメントは「文字列をそのまま返す」と書かれ、コードと食い違う。実測: `return RES.ok("hi")` → 応答の `Value:` に応答全体が入れ子になる（`Value: SHIORI/3.0 200 OK…Value: hi`）。`return "raw"` → `Value: raw`。
- 結論: 台帳本体の申し送り「二重包みの疑い」は実測で確定した。ハンドラは Value にする文字列（またはシーンのコルーチン、nil）を返す。`RES.ok(…)`・`RES.no_content()` を返す例はすべて不正な応答を生む。ソースの注記・使用例との矛盾はバグ候補の判定基準 b・c に当たり得るため、付録への追加は 1.2 の担当範囲で扱う（本節は記録のみ）。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| lua/patterns.md:96 | `return RES.ok("\\h\\s[0]起動しました。\\e")`（L102 を含む） | 訂正対象 | 2.8 |
| lua/patterns.md:104 | `return RES.no_content()  -- 表示なしで処理完了` | 訂正対象 | 2.8 |
| lua/patterns.md:88 | 「`RES` でレスポンスを返す」（L119〜L126 の API 表の位置づけを含む） | 訂正対象 | 2.8 |
| lc/SKILL.md:148 | REG に登録し `RES.ok()`／`RES.no_content()` 等でレスポンスを返す | 訂正対象 | 4.4 |
| lc/SKILL.md:85 | `pasta.shiori.res`: `RES.ok()`, `RES.no_content()`（モジュール一覧としては正） | 正 | — |
| lc/references/shiori-handlers.md:22 | 登録パターン `return RES.ok(…)  -- または RES.no_content()`（L106・L125・L141・L159・L205・L226・L257 の例も同じ）（追記: L74・L108・L143・L207・L259 も同じ） | 生成で置換（4.1） | 2.6（移設時）・4.1 |

### X20 OnNotifyCallbackResponse

- grep: `OnNotifyCallbackResponse|コールバック`、`OnPastaCallBack`
- 実装照合: ps:pasta/shiori/event/callback.lua `CALLBACK.next_event_id`（コールバックのイベント名は `"OnPastaCallBack" .. N` で毎回固有）・`try_route`（`CALLBACK.pending[req.id]` と一致すれば待機中のコルーチンを再開）、ps:pasta/shiori/event/init.lua `EVENT.fire`（REG より先に `CALLBACK.try_route(req)`）。`OnNotifyCallbackResponse` という名前はコードに無い。
- 結論: 台帳本体の申し送りどおり、「event.init で REG に自動登録」「独自ハンドラで機構が止まる」は誤り。加えて、イベント名そのものが実装と違う（`OnPastaCallBack{N}`）。2.6 の収録先見出し「#OnNotifyCallbackResponse」は実装の名前と挙動に合わせて立てる。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| lc/references/shiori-handlers.md:230 | 節「OnNotifyCallbackResponse — SSPコールバック応答」（L232〜L242。`pasta.shiori.callback` モジュール、event.init で自動登録、独自ハンドラで止まる） | 生成で置換（4.1） | 2.6（移設時）・4.1 |
| lc/references/internal-modules.md:190 | `get_property` はタグを発行しコールバック到着まで待機（イベント名の記述なし） | 正 | — |

### X21 `spot_newlines` の出力

- grep: `spot_newlines|\\n\[half\]|half`
- 実装照合: lua:loader/config/sections.rs `default_spot_newlines`（1.5）、ps:pasta/shiori/sakura_builder.lua `BUILDER.build`（`string.format("\\n[%d]", math.floor(spot_newlines * 100))`）。
- 結論: 台帳本体の該当行（`pasta-toml.md` L235）の訂正どおり。既定 1.5 は `\n[150]`（百分率）になり、`\n[half]` ではない。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| ga/references/pasta-toml.md:223 | スポット切替時の改行量（`\n[半角]` の倍率） | 生成で置換（4.1） | 2.7（移設時）・4.1 |
| ga/references/pasta-toml.md:237 | 値 `1.5` は `\n[half]`（1.5 行） | 生成で置換（4.1） | 2.7（移設時）・4.1 |
| ga/references/pasta-toml.md:45 | 既定 `1.5`（L116 を含む） | 正 | — |

### X22 実装に無い名前（`pasta.word_lookup`・`LabelNotFound`・`ScriptEvent::Error`・`CALLBACK.resume_pending`）

- grep: `word_lookup|LabelNotFound|ScriptEvent|resume_pending`
- 実装照合: ps:pasta/act.lua `ACT_IMPL.word`・`ACT_IMPL.call`（未発見は警告ログを出して nil）、ps:pasta/shiori/entry.lua `SHIORI.request`（実行時エラーは `RES.err` で 500 と `X-Error-Reason`）、ps:pasta/shiori/event/callback.lua（`resume_pending` は無い）。いずれの名前も crates 配下の実装に無い。
- 結論: 台帳本体の備考どおり。範囲内のヒットは `resume_pending` の 1 件だけ（D16 と同じ行）。他の 3 つは撤去される `GRAMMAR.md`・doc/spec にだけある。

| 位置 | 記述（要旨） | 判定 | 訂正先 |
| ---- | ------------ | ---- | ------ |
| lc/references/shiori-handlers.md:217 | `CALLBACK.resume_pending()` | 生成で置換（4.1） | 2.6（移設時）・4.1 |

### 「→1.3」申し送りの解決

台帳本体の備考で「→1.3」とした行と、それを確定させた項目の対応。結論の要旨は各項目の「結論」に書いた。

| 台帳行 | 確定させた項目 | 実装照合の結論 |
| ------ | -------------- | -------------- |
| `02-markers.md` L16 | X01 | 空白は列挙した文字だけ |
| `02-markers.md` L33 | X02 | 代入のコロン形は受理されない |
| `02-markers.md` L135 | D06 | 引数は読点／カンマ区切り。空白区切りはパースエラー、名前付きは名前が無視される |
| `02-markers.md` L151 | X02 | `＄my_var ： 10` はパースエラー |
| `02-markers.md` L203 | D10 | フェンスは行頭の 3 個以上のバッククォート＋任意識別子、インデント不可 |
| `02-markers.md` L211 | X03 | 内容は検証されず、関数定義以外の文も実行される |
| `02-markers.md` L272 | X04 | 式に比較演算子は無い |
| `03-block-structure.md` L99 | X05 | Lua ブロックはロード時に評価（`__start__` の中ではない） |
| `03-block-structure.md` L127 | D10 | インデントしたフェンスはパースエラー |
| `04-call-spec.md` L23 | X06 | ターゲットは任意の式、`tostring` してから検索 |
| `05-literals.md` L7 | X07 | 真偽値リテラルは無い |
| `07-sakura-script.md` L15 | X08 | `\]` は特別扱いされない |
| `09-variables.md` L21 | X09 | `var` はイベント処理ごと。yield したシーンは再開後も同じ `var` |
| `09-variables.md` L107 | X10 | 範囲内にヒットなし。例は `function SCENE.add(act, x, y)` で書く |
| `10-words.md` L3 | X11 | グローバル単語は全辞書ファイル共通 |
| `11-actor-dictionary.md` L91 | X12 | `lua` ブロックは出力され A1 で到達する（design #14 と食い違い、実装が正。書き方は 1.4） |
| `12-future.md` L80 | X08 | `\]` は無い |
| `12-future.md` L109 | X13 | BOM 付きはパースエラー（「BOM は許容」は否定） |
| `GRAMMAR.md` L221 | X02・X07 | コロン形・`true` とも受理されない |
| `GRAMMAR.md` L317 | X02・X06 | `＄スコア：75` は受理されない。`＞＠分岐判定` は `"nil"` で再検索し警告 |
| `GRAMMAR.md` L509 | X14 | `「「セリフ」」` の値は `セリフ` |
| `GRAMMAR.md` L539 | X08 | `\s[a\]b]` は `\s[a\]` と台詞 `b]` に分かれる |
| `GRAMMAR.md` L590 | X03 | 「関数定義のみ」は検証されない |
| `grammar-model.md` L128 | X05・X03 | 配置と評価時期は X05、「関数定義のみ」は X03 |
| `grammar-model.md` L148 | X07 | 真偽値リテラルは無い |
| `call-spec.md` L112 | X06 | DSL からの呼び出しは nil ガードに届かない |
| `sakura-script.md` L44 | X08 | `\]` を削除 |
| `variables.md` L27 | X02 | コロン形は受理されない |
| `actor-dictionary.md` L38 | X15 | 立ち位置は次の `％` 行までイベントをまたいで保たれる |
| `pasta-toml.md` L290 | X16 | main 取り込みで `surface / dressup` 節に置換済み、実装と一致 |
| `runtime-api.md` L245 | X17 | 読み取り専用は強制されない |
| `runtime-api.md` L651 | X18 | `[lua] libs` はロード時に読まれず、ゴーストから `@env` を有効にする手段は無い（台帳本体の備考を否定） |
| `shiori-handlers.md` L17 | D13・X19 | `function(act)`。文字列を返すと `RES.ok` で包まれ、`RES.ok(…)` を返すと二重になる（実測で確定） |
| `shiori-handlers.md` L230 | X20 | イベント名は `OnPastaCallBack{N}`、REG より先に `CALLBACK.try_route` が処理 |

### 台帳本体と異なる結論（実装を正とする）

実装照合で、台帳本体の記述と異なる結論になった点。台帳本体の行はこの節を正として読む。

- `runtime-api.md` L651（X18）: 本体の「pasta.toml `[lua] libs` への `"env"` 追加として書く」は誤り。`[lua]` はロード経路で読まれない。
- `12-future.md` L109（X13）: 本体の「『BOM は許容』は実測で確定」は、実測で「BOM 付きはパースエラー」と確定した。
- `pasta-toml.md` L290（X16）: 本体の「`default_surface` を読むコードが見当たらない」は main 取り込み後も正しいが、スキル側の記述は `surface / dressup` に置き換わった。収録先の節名は `surface / dressup` とする。
- `shiori-handlers.md` L230（X20）: 収録先見出しの「OnNotifyCallbackResponse」は実装に無いイベント名。実装の名前（`OnPastaCallBack{N}`）と挙動で節を立てる。
- `11-actor-dictionary.md` L91（X12）: design #14 の「処理に反映されない」と実装が食い違う（本体の備考どおり。扱いは 1.4）。

新規のバグ候補の可能性（X19 二重包み・D07 改行入り `"` 文字列・D07 単語値 `""`/`「」`・X18 `[lua]` 未使用・D03 OnChoiceSelectEx のローカル限定）は 1.4 で付録への追記要否を決める。

## 将来仕様の仕分け表

`doc/spec/` ch08・ch12 の各見出しと、台帳本体で「→1.4」と申し送った項目を、design.md「FutureSpecRouting」の仕分け規則で分類し、行き先を確定する（要件 1.3, 1.4）。

- 区分: **M**（現行実装で確認できる事実 → マニュアルへ収録）／**B**（構文が定義され範囲が明確な未実装機能 → brief 起票＋roadmap のキー行）／**R**（方針未定・DSL 範囲外の留保 → roadmap のキー行のみ）／**除外**（理由を記す）。仕分けは優先度でなく具体度で行う。
- 1 つの見出しが現行挙動と未実装の部分を併せ持つときは、部分ごとに行を分ける（吸収元列に部分を括弧書きする）。M が実装で確認できない部分は R へ倒した（design の規則）。design の仕分け結果表に無い R4〜R6 は、この規則で追加したもの。
- 行き先の `B1`・`R1` などは下の「B・R の行き先（大タスク 6 への申し送り）」の ID。M の行き先は収録先表と台帳本体の該当行に従う（`章#節` は台帳本体と同じ書き方）。
- 実装照合: コミット `2d98dcf4` の実装を、付録と同じスクラッチの検証プログラムで動かした結果（`d95e12f2` から crates 配下は変わっていない）。略記は凡例「実装照合列の略記」と同じ。

### design #14 の前提の訂正（アクタースコープ内コードブロック）

design.md（ContentMigration の「マニュアル既存の『将来変更あり』節の整理」と決定 #14）は、アクタースコープ内のコードブロックを「構文が受理されるが処理に反映されない」ものとして扱う。実装照合の結果、この前提は誤りだった（X12・台帳本体 ch11 L91 行の備考）。実装を正とし、次の現行挙動に基づいて仕分ける。

- 出力: 識別子が小文字の `lua` と完全一致するブロックだけが、そのアクターの `do` ブロック内（単語定義の後）に出力され、辞書のロード時に実行される。ブロック内では局所変数 `ACTOR`（そのアクターの表）を使える。識別子が `text`・`Lua` のブロックや識別子なしのブロックは何も出力されない（シーン内の Lua ブロックが識別子を問わず出力されるのと異なる）。
- 到達: `ACTOR.名前` に入れた値は、そのアクターのアクション行の単語参照 `＠名前` で A1（アクター表の完全一致）として最優先に見つかる。関数ならアクタープロキシ（`.actor` にアクター、`.act` に act）を唯一の引数として呼ばれ、戻り値が出力される。関数以外は `tostring` した値が出力される。
- 届かない経路: アクション行の関数呼び出し `＠名前（）`（式の検索は A1・A2 を通らず L1〜L5 だけを探す）、アクターなしの単語参照（`＄x＝＠名前` など）、SHIORI イベントからは届かない。
- 実装照合: pest:`actor_scope_item`（`code_scope`）・`` code_open = _{ PUSH("`"{3,}) ~ id? ~ eol } ``、dsl:parser/parse_elements.rs（識別子をそのまま `language` に入れる）、gen:scope `generate_actor`（`language.as_deref() == Some("lua")` のときだけ `generate_code_block`）と `generate_global_scene`（シーン内のブロックは無条件に出力）、ps:pasta/actor.lua `PROXY_IMPL.find_actor_handler`（`mode ~= "word"` なら nil。A1 は `self.actor[key]`）・`PROXY_IMPL.word`（関数なら `handler(self)`、他は `tostring`）、gen:elem `generate_action`（`＠名前（）` は `act.アクター:expr_fn`）。テスト: crates/pasta_lua/src/code_gen/scope_gen_tests.rs `actor_skips_empty_words_and_non_lua_code_blocks`、crates/pasta_dsl/tests/actor_code_block_test.rs。
- 実測: `％さくら` 直下の ```` ```lua ```` で `function ACTOR.挨拶(p, ...)`（引数の種類と個数を返す）と `ACTOR.値 = "VAL"` を定義し、```` ```text ````・識別子なし・```` ```Lua ```` のブロックでも別の値を定義した。`さくら：[＠挨拶][＠値][＠テキスト][＠無印][＠大文字]` → `[ARG=proxy:さくら:0][VAL][][][]`。`さくら：[＠挨拶（）]` → `[]`。`＄x＝＠挨拶` の後の `[＄x]` → `[]`。
- 帰結: `grammar/actor-dictionary.md`「コードブロック（将来変更あり）」は「現行挙動へ書き換え」とし、上の出力・到達・届かない経路を書く。ch11 §11.5 が予定する「アクター固有のイベントハンドラ・状態管理関数」という用途のうち、現行挙動を超える部分（イベント・式の呼び出しからの到達、アクター単位の状態の扱い）を R2 とする。design の仕分け結果表の「§11.5 … R（現行挙動は actor-dictionary.md へ）」という行き先は変えず、マニュアルへ書く現行挙動の内容だけを訂正する。

### ch08 の仕分け

| 吸収元（見出し） | 区分 | 行き先 | 実装照合 | 備考 |
| ---------------- | ---- | ------ | -------- | ---- |
| L1 8. 属性（Attribute） | 除外 | 章題（各節の行で仕分ける。章の受け皿は grammar/block-structure.md#属性） | 不要（章題） | — |
| L3 8.1 構文 | M | grammar/block-structure.md#属性（`＆名前：値`。1 行に複数並べられる。値は整数・小数・引用文字列・引用なし文字列） | pest:`attr`・`attrs = attr+`・`attr_value` | 台帳本体 ch08 L3 行 |
| L9 8.2 配置ルール（配置と制約） | M | grammar/block-structure.md#属性（属性行を置けるのはグローバルシーン初期部・ファイルレベル・アクタースコープ。シーン宣言行への付記はグローバル・ローカルとも可。ローカルシーン宣言の次の行には置けない） | pest:`global_scene_attr_line`・`file_attr_line`・`actor_scope_item`・`local_scene_line`（`scene = _{ id ~ s ~ attrs? }`）。実測は D09 | 食い違い表「属性行の配置」（D09）。アクタースコープ配下の属性行は U13 |
| L9 8.2 配置ルール（セマンティクス「直前のシーンにメタデータを付与」） | B | B1（`.kiro/specs/scene-attribute-semantics/brief.md`＋roadmap キー行） | lua:context.rs `register_global_scene`（シーン自身の属性をシーン登録表に記録）、core:scene_table.rs `filter_by_attributes`（絞り込みの処理はある）、lua:search/context.rs `search_scene`（常に空の `filters` で検索）、gen:scope `generate_global_scene`（属性を Lua に出力しない）。実測: `＆警報：レッド`・`＊会話＆温度：暑い`・`＆作者：A`・`・ローカル＆優先：3` を含む辞書の生成 Lua に属性は現れない | 現行挙動（受理され、内部の登録表に記録されるが、シーンの選択・出力に影響しない）は M として grammar/block-structure.md#属性 へ |
| L32 8.3 ファイルレベル属性（構文と現行挙動） | M | grammar/block-structure.md#属性（ファイルレベルの属性行は受理され、後続のグローバルシーンの属性と統合されるが、処理には使われない） | lua:transpiler.rs `merge_attrs`、gen:scope `generate_global_scene`（`_file_attrs` は未使用） | 台帳本体 ch08 L32 行 |
| L32 8.3 ファイルレベル属性（継承のセマンティクス） | B | B1（`.kiro/specs/scene-attribute-semantics/brief.md`＋roadmap キー行） | lua:transpiler.rs `merge_attrs`（統合の結果を使う処理が無い） | 「将来予約」の扱いは B1 の brief で決める |
| （章末）関連章 | 除外 | doc/spec の章間ナビゲーション | 不要（ナビゲーション） | — |

### ch12 の仕分け

| 吸収元（見出し） | 区分 | 行き先 | 実装照合 | 備考 |
| ---------------- | ---- | ------ | -------- | ---- |
| L1 12. 未確定事項・検討中の仕様 | 除外 | 章題（各節の行で仕分ける） | 不要（章題） | — |
| L3 12.2 チェーントーク（DSL非採用） | M | grammar/call-jump.md#特殊な呼び出し（`＞チェイントーク`・`＞yield` による現行の実現方法で置き換える） | ps:pasta/global.lua（`GLOBAL.yield`・`GLOBAL["チェイントーク"]`）、ps:pasta/act.lua `ACT_IMPL.yield` | design の「M（置換）」 |
| L9 12.4 ローカルシーンのパラメータ（将来検討） | R | R1（roadmap キー行） | pest:`scene = _{ id ~ s ~ attrs? }`（宣言にパラメータの構文が無い） | シーンへの値の受け渡しの現行挙動（Call の引数 U04・シーン引数 `＄０` U03）は M として grammar/call-jump.md#引数リスト・grammar/variables.md#シーン引数 へ |
| L15 12.5 フィルター機能の詳細（初期版対応なし） | B | B1（`.kiro/specs/scene-attribute-semantics/brief.md`＋roadmap キー行） | pest:`call_scene = { call_marker ~ (id \| call_target_expr) ~ s ~ args? }`（`＆` を受ける規則が無い）。実測: `＞X＆k＝v` → パースエラー（`expected args`） | マニュアル grammar/call-jump.md「フィルター」節は削除（下の振り分け表）。OR/AND の結合も B1 |
| L21 12.6 単語定義の値の型変換ルール（初期版） | M | grammar/literals.md・grammar/words.md#グローバル単語定義（台帳本体 ch12 L21 行） | pest:`words`・`word`、gen:elem `generate_word_definition` | 吸収元の空白区切りの例は誤り（D08）。単語値の `「」`・`""` は U25（バグ候補） |
| L31 12.7 動的単語参照（＠＄var_name）の実装スケジュール | B | B2（`.kiro/specs/dynamic-word-reference/brief.md`＋roadmap キー行） | pest:`word_ref = word_marker ~ id ~ s`（`＠＄` の規則が無い）。実測: `さくら：[＠＄x]`・`＄y＝＠＄x` → パースエラー（`expected id`） | 「未実装期間は無視または警告」の方針も実装されていない（現行はパースエラー）。brief で扱う |
| L37 12.8 Callの戻り値と変数代入（DSL非定義） | R | R3（roadmap キー行） | pest:`set`（`＞` を値に取らない） | — |
| L43 12.9 ローカル変数のスコープ詳細（トランスパイラ/ランタイム） | 除外 | `ctx.local`／`ctx.global` は `var`／`save` に置き換わった旧方針。現行挙動は grammar/variables.md#ローカル変数・#グローバル変数（台帳本体 ch09 の行で収録） | gen:elem `resolve_var_path`（`var.`・`save.`・`args[n]`） | design の仕分け結果表どおり |
| L50 12.10 属性値の型解釈（値の型） | M | grammar/literals.md（台帳本体 ch12 L50 行） | dsl:parser/parse_elements.rs `parse_attr`・`parse_attr_number` | — |
| L50 12.10 属性値の型解釈（フィルターの比較演算との整合） | B | B1（`.kiro/specs/scene-attribute-semantics/brief.md`＋roadmap キー行） | pest:`call_scene`（フィルターが無い）、pest:`bin_op`（式に比較演算子が無い。X04） | — |
| L55 12.11 前方一致時の複数候補選択ルール（DSL非定義） | M | grammar/call-jump.md#スコープ解決アルゴリズム（ローカル優先・シャッフル＆順次消費） | core:scene_table.rs `collect_scene_candidates`、core:random.rs | 食い違い表「ローカル／グローバル候補」（D03） |
| L59 12.12 引数リストの値の型解釈 | M | grammar/literals.md・grammar/call-jump.md#引数リスト（引数は式。「文字列/リテラルのみ推奨」は現行の式の受理範囲で置き換える） | pest:`arg`・`key_expr`・`positional_arg = { expr }` | 食い違い表「引数」（D06） |
| L64 12.13 行継続のインデント深さ制約 | M | grammar/action-line.md#行継続（継続は `：` 始まり。深さは問わない） | pest:`continue_action_line`、gen:elem `generate_continue_action` | 食い違い表「行継続」（D01） |
| L69 12.14 コメント行の配置可能位置 | M | grammar/block-structure.md#コメント | pest:`blank_line`（全スコープの項目に含まれる） | 行末コメントは U05、単語定義行の行末は U06（バグ候補） |
| L74 12.15 識別子と予約語の制限（DSL外） | M | grammar/variables.md#Lua 予約語の制約 | gen:elem `resolve_var_path` | — |
| L80 12.16 Sakuraスクリプト括弧内のエスケープ（確定）（引用・`""`・非ネストの透過） | M | grammar/sakura-script.md#角括弧内のエスケープと引用 | pest:`sakura_args`・`sakura_body`・`sakura_str`（`"…"` 内の `""`） | 台帳本体 ch12 L80 行 |
| L80 12.16 Sakuraスクリプト括弧内のエスケープ（確定）（`\]`・`\%` のエスケープ） | R | R4（roadmap キー行） | pest:`sakura_body = @{ ( sakura_str \| (!PEEK ~ ANY) )* }`（`"…"` の外では最初の `]` で閉じる）・`sakura_id`（`%` を含まない）。実測: `\s[a\]b]` は `\s[a\]` と台詞 `b]` に分かれる（X08）、`さくら：100\%です` → パースエラー | ukadoc の `\\`（`\` の表示）は U08（バグ候補）でマニュアルに書かない |
| L109 12.17 ファイルエンコーディングとBOM（UTF-8 固定と BOM 付きの現行挙動） | M | grammar/block-structure.md#文字コード（UTF-8 固定。BOM 付きの `.pasta` はパースエラー） | lua:loader/process.rs（`fs::read_to_string`）→ dsl:lib.rs `parse_str`。実測は X13 | — |
| L109 12.17 ファイルエンコーディングとBOM（「BOM は許容」） | R | R5（roadmap キー行） | 同上（BOM を除く処理が無い） | 実測で否定（X13） |
| L114 12.18 ファイルレベル属性の継承詳細 | B | B1（`.kiro/specs/scene-attribute-semantics/brief.md`＋roadmap キー行） | lua:transpiler.rs `merge_attrs`（統合はするが利用されない） | 現行挙動は ch08 L32 行（M） |
| L119 12.19 空行の配置と解釈（空行は無視） | M | grammar/block-structure.md#空行（空行はどこでも無視。継続中も何も出力しない） | pest:`blank_line` | — |
| L119 12.19 空行の配置と解釈（継続領域の空行の糖衣構文） | R | R6（roadmap キー行） | pest:`blank_line`（継続中の空行の専用規則が無い）。実測は D02 | 食い違い表「継続内の空行」 |
| L124 12.20 全角・半角混在時の正規化 | M | grammar/markers.md 冒頭本文（全角と半角は同等。`＠＠`・`＄＄` は 2 文字目を出力） | pest:`at_escape = @{ at{2} }`・`dollar_escape`、gen:elem `generate_action` | `＄＄` は U07 |
| （章末）更新履歴・関連章 | 除外 | doc/spec の版管理とナビゲーション（履歴は git が保持） | 不要（履歴） | — |

### ch08・ch12 以外で仕分けた見出し

| 吸収元（見出し） | 区分 | 行き先 | 実装照合 | 備考 |
| ---------------- | ---- | ------ | -------- | ---- |
| `11-actor-dictionary.md` L91 11.5 将来拡張（コードブロック）（現行挙動） | M | grammar/actor-dictionary.md#コードブロック（「design #14 の前提の訂正」の出力・到達・届かない経路） | 「design #14 の前提の訂正」の実装照合 | design #14 の「処理に反映されない」を訂正 |
| `11-actor-dictionary.md` L91 11.5 将来拡張（コードブロック）（アクター固有のイベントハンドラ・状態管理という用途） | R | R2（roadmap キー行） | ps:pasta/actor.lua `PROXY_IMPL.find_actor_handler`（word 以外は nil） | design の仕分け結果表の §11.5 行 |
| `04-call-spec.md` L81 4.2 フィルター（属性フィルター） | B | B1（`.kiro/specs/scene-attribute-semantics/brief.md`＋roadmap キー行） | pest:`call_scene` | 台帳本体の行で除外済み。内容の行き先として記録 |

### B・R の行き先（大タスク 6 への申し送り）

大タスク 6 は、B の各行を brief として起票し、B・R の全行を roadmap の「将来仕様（doc/spec 廃止時の申し送り）」小節に 1 項目 1 行（名称 — 要旨 — brief 参照 or（brief なし））で書く。バグ候補のキー行は付録「未記載の実装事実」のバグ候補 13 行から作る（本表には含めない）。

| ID | 区分 | 名称 | 要旨（roadmap キー行の内容） | 行き先 |
| -- | ---- | ---- | -------------------------- | ------ |
| B1 | B | シーン属性のセマンティクス | 属性によるシーンへのメタデータ付与、ファイルレベル属性の継承と上書き（ローカルシーンには影響しない）、Call の属性フィルター（`＞シーン＆k＝v`・比較演算子・複数条件の結合）。現行は構文の受理と内部の登録表への記録まで | `.kiro/specs/scene-attribute-semantics/brief.md`＋roadmap キー行 |
| B2 | B | 動的単語参照 `＠＄` | `＠＄変数名` で変数の値を単語名として参照する。現行はパースエラー。未実装期間の扱い（無視・警告）も未定 | `.kiro/specs/dynamic-word-reference/brief.md`＋roadmap キー行 |
| R1 | R | シーンのパラメータ | シーン宣言での名前付きパラメータ。対応予定なし。現行は Call の位置引数とシーン引数 `＄０`… で代替できる | roadmap キー行（brief なし） |
| R2 | R | アクタースコープ内コードブロックの用途 | アクター固有のイベントハンドラ・状態管理関数としての用途。現行は `lua` ブロックで定義した値・関数がアクター付きの単語参照（A1）から使えるだけで、式の呼び出し・イベントからは届かない | roadmap キー行（brief なし） |
| R3 | R | Call の戻り値と変数代入 | `＄x＝＞シーン` のような呼び出し結果の代入。DSL では定義せず、ランタイム設計の領域 | roadmap キー行（brief なし） |
| R4 | R | さくらスクリプトの `\]`・`\%` | 角括弧内の `\]`（`]` を文字として含める）と `\%` を DSL から書けるようにするか。現行は `\]` を特別扱いせず、`\%` はパースエラー | roadmap キー行（brief なし） |
| R5 | R | BOM 付きファイルの受理 | UTF-8 の BOM 付き `.pasta` を受理するか。現行はパースエラー | roadmap キー行（brief なし） |
| R6 | R | 継続行内の空行の糖衣構文 | 行継続の途中の空行を改行として出力する糖衣構文。現行は何も出力しない | roadmap キー行（brief なし） |

### マニュアル既存の「将来変更あり」節の振り分け

| 節（`book/src/` 相対） | 振り分け | 実装照合 | 書き換え後の内容・移送先 | 担当 |
| ---------------------- | -------- | -------- | ------------------------ | ---- |
| grammar/call-jump.md「フィルター（将来変更あり）」（L109〜L119） | 削除して brief へ | pest:`call_scene`（`＆` を受けない）。実測: `＞X＆k＝v` → パースエラー | 節ごと削除する。比較演算子による拡張の予告を含め、内容は B1 へ移す | 2.2（削除）・6（brief） |
| grammar/words.md「動的単語参照（将来変更あり）」（L96〜L100） | 削除して brief へ | pest:`word_ref`（`＠＄` の規則が無い）。実測: `＠＄x` → パースエラー | 節ごと削除する。内容は B2 へ移す | 2.3（削除）・6（brief） |
| grammar/block-structure.md「属性（将来変更あり）」（L112〜L129） | 現行挙動へ書き換え | ch08 L3・L9・L32 行の実装照合 | 見出しから「（将来変更あり）」を外し、構文（ch08 L3 行）・配置（ch08 L9 行）・処理（受理され、内部の登録表に記録されるが、シーンの選択・出力に影響しない。ファイルレベル属性も統合されるだけで使われない）を書く。「将来予定」の注記と doc/spec へのリンクは削除し、継承・フィルターの意味論は書かない（B1） | 2.1 |
| grammar/index.md「網羅範囲についての注記」の属性注記（L80） | 現行挙動へ書き換え | 同上 | 「属性の構文は受理されるが処理には反映されない（詳細は block-structure の属性節）」という現行挙動の注記にし、「将来予定」「将来変更あり」の文言と doc/spec ch08 への言及を外す | 2.1 |
| grammar/actor-dictionary.md「コードブロック（将来変更あり）」（L149〜L164） | 現行挙動へ書き換え | 「design #14 の前提の訂正」の実装照合（X12） | 見出しから「（将来変更あり）」を外し、「design #14 の前提の訂正」の出力・到達・届かない経路を書く。例は `function SCENE.on_actor_event(act)` をやめ、`ACTOR.名前` に値・関数を入れて `アクター：＠名前` で使う形にする（フェンスは行頭。D10）。アクター固有のイベントハンドラという用途の予告は書かない（R2） | 2.3 |
| introduction.md「安定機能と『将来変更あり』の区別について」（L30〜L37） | 定義の書き換え（節は残す） | book/tools/verify-content.mjs `F-future`（introduction に「将来変更あり」の注記があることを検査） | 表記と `F-future` 検査は残し、定義を「現行挙動だが将来変わり得る箇所の注記」に限定する（未実装機能の予告には使わない。design #14） | 2.8 |

- `introduction.md`「安定機能と『将来変更あり』の区別について」は「削除・書き換え」の二択の対象外（定義の書き換えのみ）。design #14 のとおり表記と `verify-content.mjs` の `F-future` 検査は残し、定義を「現行挙動だが将来変わり得る箇所の注記」とする。現在の定義文（「未確定・実装予定…」）は #14 の定義と合わないため書き換えが要る。担当は 2.8（導入章の定義整理）。
- 上の 5 節を処理すると、`book/src` の「将来変更あり」は `introduction.md` の定義だけになる（2026-10-01 時点の grep: 上記 5 節と `introduction.md` L30・L36〜L37 のみ）。

### 「→1.4」申し送りの解決

台帳本体の備考・収録先列で 1.4 へ申し送った行と、それを確定させた仕分けの対応。

| 台帳行 | 確定させた仕分け | 結論 |
| ------ | ---------------- | ---- |
| `02-markers.md` L135 | ch12 L31 行（B2） | `＠＄` はマニュアルの単語参照から削除し B2 へ |
| `02-markers.md` L170 | ch12 L15 行（B1） | Call のフィルターは構文として受理されない。B1 へ |
| `02-markers.md` L272 | ch12 L50 行（フィルターの比較演算との整合、B1） | フィルター用の比較演算子は B1。現行の式に比較演算子は無い（X04） |
| `04-call-spec.md` L81 | 「ch08・ch12 以外で仕分けた見出し」の同行（B1）・振り分け表 grammar/call-jump.md | 節を削除し B1 へ |
| `08-attributes.md` L32 | ch08 L32 行（M・B1） | 構文と統合までは M、継承の意味論は B1 |
| `10-words.md` L38 | ch12 L31 行（B2） | `＠＄` は B2 |
| `11-actor-dictionary.md` L91 | 「ch08・ch12 以外で仕分けた見出し」の同行（M・R2）・「design #14 の前提の訂正」 | `lua` ブロックは出力され A1 で到達する（現行挙動へ書き換え）。用途は R2 |
| `12-future.md` L1 | ch12 L1 行（除外） | 章題。各節を仕分けた |
| `12-future.md` L9 | ch12 L9 行（R1） | R1 |
| `12-future.md` L15 | ch12 L15 行（B1） | B1 |
| `12-future.md` L31 | ch12 L31 行（B2） | B2 |
| `12-future.md` L37 | ch12 L37 行（R3） | R3 |
| `12-future.md` L43 | ch12 L43 行（除外） | 旧方針。現行挙動は variables.md |
| `12-future.md` L114 | ch12 L114 行（B1） | B1 |
| `grammar-model.md` L184 | ch08 L32 行（M・B1） | 構文と統合までは M、継承は B1 |
| `call-spec.md` L8 | ch12 L15 行（B1） | フィルター付きの行を削除し B1 へ |
| `call-spec.md` L46 | ch12 L15 行（B1） | B1 |
| `words.md` L65 | ch12 L31 行（B2）・振り分け表 grammar/words.md | 節を削除し B2 へ |
| 食い違い grep 記録 D05 | 振り分け表 grammar/call-jump.md（削除して brief へ）・B1 | 節の削除は 2.2、brief は 6 |
| 食い違い grep 記録 X12、「→1.3」申し送りの解決の `11-actor-dictionary.md` L91 行、「台帳本体と異なる結論」の X12 | 「design #14 の前提の訂正」 | 実装が正。書き方は振り分け表 grammar/actor-dictionary.md |
| 「台帳本体と異なる結論」末尾の新規バグ候補 5 件（X19・D07 ×2・X18・D03） | 付録 U23〜U27 | 5 件ともバグ候補。収録する現行挙動の範囲は各行の備考 |
