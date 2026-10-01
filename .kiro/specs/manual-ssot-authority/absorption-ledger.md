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

タスク 1.2 が記入する（吸収元のどこにも書かれていない実装構文を「収録先（章#節）」または「バグ候補（根拠 a〜c）」で 1 行ずつ）。

## 食い違い grep 記録

タスク 1.3 が記入する（design.md の食い違い表の項目ごとに、マニュアル全章・両 SKILL.md・スキル手書きファイルの grep 結果と照合したソース位置）。

## 将来仕様の仕分け表

タスク 1.4 が記入する（ch08・ch12 の各項目の区分 M／B／R／除外と行き先、マニュアル既存の「将来変更あり」節の振り分け）。
