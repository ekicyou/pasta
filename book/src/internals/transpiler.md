# トランスパイルパイプライン

ごきげんよう。作者の書いた `.pasta` が、どうやって Lua のコードに化けるのか――その錬金術の工程をお見せいたしますわ。
パースから登録、生成、そしてキャッシュまで。一本の流れとして追えば、何も怖くはございませんの。さあ、参りましょう。

---

この章では、`.pasta` 辞書を Lua コードへ変換するトランスパイルパイプラインを扱う。

## 目的と責務

トランスパイルパイプラインは、Pasta DSL のソースをパースして AST を作り、ファイル内の項目を文書順に 1 回たどりながら、シーン・単語のレジストリへの登録と Lua コードの生成を進める。生成したコードは出力の正規化を経て、トランスパイル結果キャッシュとしてゴーストのディレクトリへ書き出される。

この章が責務を持つのは次の事項である。

- パーサと AST の構成、AST を組み立てる時点で行う正規化
- `TranspileContext` によるトランスパイル時の登録と、要素ごとの Lua コード生成
- 生成時最適化（末尾呼び出し・継続行の話者引継ぎ・文字列リテラルの表記選択）
- 出力の正規化
- トランスパイル結果キャッシュ（更新判定・キャッシュ先・`scene_dic.lua` の生成）
- ソースマップを生成する側の出入口
- トランスパイル時の処理と実行時の辞書確定から成る**段階構成の定義**（この章が権威を持つ）

文法そのもの（何が書けて何がエラーになるか）は利用者向けの [文法リファレンス](../grammar/index.md) が正であり、この章では再記述しない。実行時の辞書確定の内部（Lua 側の収集とレジストリの再構築、検索）は [シーン・単語レジストリとシーン検索](registry-search.md)、起動シーケンス全体の中でトランスパイルがどこで呼ばれるかは [ローダ自己展開とモジュール解決](loader.md)、ソースマップの構造と解決は [デバッグ基盤とシーンキック](debug.md) で扱う。

## 構成要素

### パーサと AST

パーサと AST は `pasta_dsl` クレートにあり、Lua にもレジストリにも依存しない。

| 要素 | 所在 | 役割 |
| ---- | ---- | ---- |
| pest 文法 | `crates/pasta_dsl/src/parser/grammar.pest` | Pasta DSL の PEG 文法。`file = _{ SOI ~ ( file_scope \| global_scene_scope \| actor_scope )* ~ s ~ EOI }` を頂点とする |
| パーサ API | `crates/pasta_dsl/src/parser/mod.rs` | `parse_str`・`parse_file` と、pest の解析結果から AST を組み立てる `build_ast` |
| 規則ごとの組み立て | `crates/pasta_dsl/src/parser/parse_scene.rs`・`crates/pasta_dsl/src/parser/parse_action.rs`・`crates/pasta_dsl/src/parser/parse_elements.rs` | シーン・アクション・属性や単語定義などの要素を AST ノードへ変換する |
| AST 型 | `crates/pasta_dsl/src/parser/ast/` | `PastaFile`・`FileItem`・`GlobalSceneScope`・`LocalSceneScope`・`LocalSceneItem`・`ActorScope`・`Action`・`Expr`・`Span` など |
| パースエラー | `crates/pasta_dsl/src/error.rs` | `ParseError`（ファイル名・行・桁付きの構文エラー） |
| 部分パース | `crates/pasta_dsl/src/partial.rs` | `parse_str_partial`。全体のパースに失敗したときにスコープ単位・行単位で解析を続ける。言語サーバ用であり、ランタイムのトランスパイルでは使わない |

AST の骨格は次のとおりである。`PastaFile.items` はファイル内の項目を記述順に保持する。

```text
PastaFile
 └ items: Vec<FileItem>（記述順）
    ├ FileAttr(Attr)              ファイルレベルの属性
    ├ GlobalWord(KeyWords)        ファイルレベルの単語定義
    ├ ActorScope                  アクター辞書（name, attrs, words, var_sets, code_blocks）
    └ GlobalSceneScope            グローバルシーン
       ├ name / is_continuation
       ├ attrs, words, actors（SceneActorItem）, code_blocks
       └ local_scenes: Vec<LocalSceneScope>
          ├ 先頭は暗黙の開始ブロック（name = None）
          └ 以降は名前付きローカルシーン（name = Some）
             └ items: Vec<LocalSceneItem>
                VarSet / CallScene / ActionLine / ContinueAction / CueCommand / Choice
```

AST ノードは `Span`（開始・終了の行と桁、バイトオフセット）を持ち、エラー表示とソースマップに使われる。

#### 動的参照の規則と AST

変数の値を名前として使う動的参照（`＠＄変数名`・`＠＄変数名（…）`。書ける形と書けない形は利用者向けの [動的単語参照](../grammar/words.md#動的単語参照) が正）は、次の規則で解析する。

| 規則 | 形 | 使われる位置 |
| ---- | -- | ------------ |
| `dyn_name_local` | `var_marker ~ var_id`（`＄名前`・`＄０`） | `dyn_name` の選択肢 |
| `dyn_name_global` | `var_marker ~ global_marker ~ id`（`＄＊名前`） | `dyn_name` の選択肢（`dyn_name_local` より先に試す） |
| `word_ref_dynamic` | `word_marker ~ dyn_name ~ s` | `action` の選択肢（`word_ref` の後）と、変数代入の右辺（`set`） |
| `fn_call_dynamic` | `fn_marker ~ dyn_name ~ args` | `fn_call` の選択肢。`fn_call` はアクションと式の項の両方に現れる |

`dyn_name` はプロパティ変数（`＄％`）を含まない。`action` では `fn_call` が `word_ref_dynamic` より先に試されるため、引数が正しく解析できる `＠＄変数名（…）` は関数呼び出しになる。AST では次の変種になる。

| 構文の位置 | AST |
| ---------- | --- |
| アクションの `＠＄変数名` | `Action::DynamicWordRef { var_name, var_scope, span }` |
| アクションの `＠＄変数名（…）` | `Action::DynamicFnCall { var_name, var_scope, args, span }` |
| 式の中の `＠＄変数名（…）` | `Expr::DynamicFnCall { var_name, var_scope, args }` |
| 変数代入の右辺の `＠＄変数名` | `SetValue::DynamicWordRef { var_name, var_scope }` |

`var_scope` は `VarScope` であり、`dyn_name_global` は `VarScope::Global`、`dyn_name_local` は `var_ref_local` と同じ規則で、数字の名前なら `VarScope::Args(番号)`、それ以外なら `VarScope::Local` になる（`parse_action.rs` の `parse_dyn_name`）。部分パースの `shift_action` は、2 つの `Action` の変種の `Span`（`DynamicFnCall` は引数の `Span` も）を、静的な単語参照・関数呼び出しと同じくずらす。

### トランスパイラとコード生成

| 要素 | 所在 | 役割 |
| ---- | ---- | ---- |
| `LuaTranspiler` | `crates/pasta_lua/src/transpiler.rs` | 1 ファイル分の AST を受け取り、登録と生成の走査・出力の正規化を行う入口 |
| `TranspileContext` | `crates/pasta_lua/src/context.rs` | 走査中の状態。`SceneRegistry`・`WordDefRegistry`（`pasta_core`）とファイル属性の累積を持つ |
| `LuaCodeGenerator` | `crates/pasta_lua/src/code_gen/mod.rs` | 出力先・インデント・出力行番号（`out_line`）・ソースマップ用シンクを持つ生成器本体。ヘッダ出力と行出力の共通処理 |
| スコープ単位の生成 | `crates/pasta_lua/src/code_gen/scope_gen.rs` | アクター辞書・グローバルシーン・ローカルシーンの Lua ブロック、末尾呼び出しの判定、選択肢 |
| 要素単位の生成 | `crates/pasta_lua/src/code_gen/element_gen.rs` | アクション・変数代入・Call・単語定義・Lua ブロック、変数の経路と動的参照の引数 |
| 式の生成 | `crates/pasta_lua/src/code_gen/expr_gen.rs` | 式の値・関数呼び出し・引数リストと、算術・連結の連鎖の組み直し |
| ソースマップのシンク | `crates/pasta_lua/src/code_gen/source_map.rs` | 生成器が対応を記録する先の `SourceMapSink` トレイト |
| 出力の正規化 | `crates/pasta_lua/src/normalize.rs` | `normalize_output`・`normalize_output_with_shift` と、削除行の記録 `LineShift` |
| 文字列リテラル化 | `crates/pasta_lua/src/string_literalizer.rs` | `StringLiteralizer`。文字列を Lua のリテラル表記に変換する |
| 設定 | `crates/pasta_lua/src/config.rs` | `TranspilerConfig`（`comment_mode`・`line_ending`）と `LineEnding` |
| エラー | `crates/pasta_lua/src/error.rs` | `TranspileError` |

`TranspilerConfig` の `line_ending` は中間バッファの改行にだけ効き、出力の正規化で LF に統一されるため、書き出されるバイト列には影響しない。`comment_mode` は現行のコード生成からは参照されない。

### キャッシュ

| 要素 | 所在 | 役割 |
| ---- | ---- | ---- |
| `CacheManager` | `crates/pasta_lua/src/loader/cache.rs` | キャッシュディレクトリの版管理、更新判定、キャッシュ先とモジュール名の導出、`scene_dic.lua` の生成、孤立キャッシュの検出 |
| 増分処理 | `crates/pasta_lua/src/loader/process.rs` | ローダの増分処理。ファイルごとに更新判定・パース・トランスパイル・キャッシュ保存を行い、`TranspileContext` を統合する |

## 処理とデータの流れ

### 段階構成

シーンと単語が検索可能な辞書になるまでの処理は、次の 2 段から成る。

```text
第 1 段: トランスパイル時の単一走査（登録と生成）    … ローダの増分処理
  .pasta ──parse_str──▶ AST ──LuaTranspiler──▶ Lua ソース ──▶ キャッシュ
                              │ 項目ごとに「登録 → 生成」
                              ▼
                        TranspileContext（ファイルごと → 統合）

第 2 段: 実行時の辞書確定                         … Lua VM の構築後
  require("pasta.scene_dic")
    ├ require("pasta.scene.…") × 全モジュール
    │   生成コードの実行で PASTA.create_actor / create_scene / create_word が
    │   Lua 側のレジストリへ登録する
    └ require("pasta").finalize_scene()
        Lua 側の登録を収集して SceneRegistry / WordDefRegistry を作り直し、
        @pasta_search を登録し直す
```

第 1 段では、`LuaTranspiler` がファイルの項目を文書順に 1 回だけたどり、各項目について `TranspileContext` への登録と Lua コードの生成を続けて行う。シーンを全部集めてから生成に入るといった、走査を分けた構成にはなっていない。

第 2 段では、第 1 段の生成コードを Lua VM で実行することで Lua 側のシーン・単語の登録が作られ、`finalize_scene` がそれを収集して、実行時に検索で使うレジストリを確定する。実行時のグローバルシーンの通し番号（同名のグローバルシーンを区別する `メイン_1`・`メイン_2` の番号）もこの段で、Lua 側の `create_scene` が全モジュールを通して振る。登録名の構成と分ける規則は [登録名の構成](registry-search.md#登録名の構成) で扱う。

第 1 段の `TranspileContext` のレジストリも VM の構築時に `@pasta_search` として登録されるが、第 2 段の `finalize_scene` が登録し直すため、検索で使われるのは第 2 段の結果である。第 1 段のレジストリの登録名の形式（`メイン_1`）と検索キーは第 2 段と同じだが、キャッシュが最新でトランスパイルを省いたファイルの分を含まず、グローバルシーンの通し番号もファイル単位で振られる（複数のファイルに同名のシーンがあると実行時の番号と一致しない）。辞書の権威が第 2 段にあるのはこのためである。

従来「2 パス」（Pass 1: シーン登録、Pass 2: コード生成）と呼ばれてきた区分は、現行実装には存在しない。現行の構成で旧称に対応させるなら、トランスパイル時の登録と生成は第 1 段の単一走査の中で項目ごとに行われ、検索に使う辞書の登録は第 2 段の実行時の辞書確定で行われる。「2 パス」という語を見かけた場合は、この 2 段を指すものと読み替える。

### パースと AST 構築時の正規化

`parse_str` は pest で `file` 規則を解析し、失敗すれば `ParseError::SyntaxError` を返す。成功すれば `build_ast` が解析結果を記述順にたどって `PastaFile` を組み立てる。AST を組み立てる時点で、次の正規化を行う。

- `file_scope` の属性と単語定義を、それぞれ `FileItem::FileAttr`・`FileItem::GlobalWord` に平坦化し、他の項目との記述順を保つ。
- 名前を省いたグローバルシーン（`＊` だけの行）は、直前のグローバルシーンの名前を引き継ぎ、`is_continuation` を真にする。ファイルの先頭で現れた場合はパースエラーにする。
- 全角の数字・マイナス・小数点を半角に直してから数値として解釈する（`normalize_number_str`）。式の数値、属性値、アクター番号、キューコマンドの引数、`＄０` などの引数番号が対象である。
- 空の引用（`string_blank` 規則の `「」`・`""`）は、囲み文字ではなく空文字列として記録する。単語値は `""` の候補、属性値は `AttrValue::String("")`、キューコマンドの引数は空の文字列リテラル、式は `Expr::BlankString` になる。
- シーンのアクター指定行のアクター番号を採番する。番号を明示した項目はその値、省いた項目は直前の番号に 1 を足した値（先頭で省いた場合は 0）になる。

### 登録と生成（単一走査）

`LuaTranspiler::transpile_with_source_map` は、中間バッファに向けた `LuaCodeGenerator` を作り、ヘッダ（`pasta` と `pasta.global` の `require`）を出力したあと、`PastaFile.items` を記述順に処理する。

| 項目 | 登録（`TranspileContext`） | 生成 |
| ---- | -------------------------- | ---- |
| `FileAttr` | ファイル属性に累積する。同じキーは後の値で上書きする | なし |
| `GlobalWord` | 名前ごとにグローバル単語として登録する | `PASTA.create_word(名前):entry(値, …)` をファイルの最上位に出力する |
| `ActorScope` | 名前ごとにアクター単語として登録する | アクターごとの `do` 〜 `end` ブロック |
| `GlobalSceneScope` | グローバルシーンを登録し、同名の出現順の番号を得る。シーンレベルの単語をそのシーンのローカル単語として登録する | ファイル属性とシーン属性を統合（シーンが優先）したうえで、シーンごとの `do` 〜 `end` ブロックを出力する。生成後、名前付きローカルシーンを、生成器と同じ `local_scene_counters` の通し番号で登録する |

ファイル属性はファイルごとの状態であり、別のファイルへは持ち越さない。統合した属性は生成器へ渡されるが、現行のコード生成は属性を出力に使わない。

### 生成される Lua コードの形

次の入力を例に、生成されるコードの形を示す。

```pasta
＊メイン
    ぱすた：こんにちは
    ＞サブ

    ・サブ
        ぱすた：サブです
```

```lua
local PASTA = require "pasta"
local GLOBAL = require "pasta.global"

do
    local SCENE = PASTA.create_scene("メイン")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act:actor_proxy("ぱすた"):talk("こんにちは")
        return act:call(SCENE.__global_name__, "サブ", {}, table.unpack(args))
    end

    function SCENE.サブ_1(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act:actor_proxy("ぱすた"):talk("サブです")
    end
end
```

スコープごとの形は次のとおりである。

- **アクター辞書**: `local ACTOR = PASTA.create_actor("名前")` に続けて、単語定義ごと・名前ごとに `ACTOR:create_word(名前):entry(値, …)` を出力する。配下の Lua ブロックは、言語識別子が `lua` のものだけを出力する。属性行と変数代入行は出力しない。
- **グローバルシーン**: `PASTA.create_scene` には、シーン名のうち Unicode の英字・数字（`char::is_alphanumeric`。かな・漢字を含む）と `_` 以外の文字を `_` に置き換えた基本名を渡す（`SceneRegistry::sanitize_name`）。番号は付けない。シーンレベルの単語定義は `SCENE:create_word(名前):entry(値, …)` として関数定義より前に出力する。グローバルシーンとローカルシーンに置いた Lua ブロックは、そのグローバルシーンのすべての関数定義の後に、言語識別子にかかわらず出力する。
- **ローカルシーン**: 暗黙の開始ブロックは常に `SCENE.__start__` になり、名前付きローカルシーンは `照合用の名前_通し番号` の関数になる（`SceneRegistry::registered_name`。`・選択・A` は `選択_A_1`）。照合用の名前はローカルシーン名をサニタイズしたもので、通し番号は同じグローバルシーンの中で照合用の名前ごとに数える出現順（1 始まり）である（`crates/pasta_lua/src/context.rs` の `local_scene_counters`）。`・挨拶・1` と `・挨拶_1` はどちらも照合用の名前が `挨拶_1` なので、`挨拶_1_1` と `挨拶_1_2` になる。トランスパイル時のレジストリの `register_local` も同じ通し番号を使うため、レジストリのローカルシーンの登録名は生成した関数名と一致する。どの関数も `local args = { ... }` と `local save, var = act:init_scene(SCENE)` で始まる。`__start__` でシーンにアクター指定行がある場合は、続けて `act:clear_spot()` と、アクターごとの `act:set_spot(名前, 番号)` を出力する。

ローカルシーンの項目は、次の形に変換される。

| 項目 | 生成 |
| ---- | ---- |
| 変数代入（ローカル・グローバル） | `var.名前 = 式`・`save.名前 = 式`。右辺が単語参照なら `act:word(名前)`、動的単語参照なら `act:word(var.変数名, "var.変数名")`、プロパティ参照だけなら `act:get_property(名前)` |
| 変数代入（プロパティ） | `act:set_property(名前, 式)`。右辺が単語参照・動的単語参照なら、式の代わりに上と同じ `act:word(…)` を渡す |
| 式文（`＄＝`） | 式をそのまま 1 文として出力する |
| Call | ローカルシーンの最後の項目なら `return act:call(SCENE.__global_name__, キー, {}, 明示した引数…, table.unpack(args))`、それ以外なら `act:call_restore(SCENE.__global_name__, キー, {}, 明示した引数…, table.unpack(args))`（[末尾呼び出し](#末尾呼び出し)）。キーは、静的ターゲットなら `"名前"`、動的ターゲットなら `act:call_key(…)`（[動的コールのキー](#動的コールのキー)） |
| アクション行・継続行 | アクションごとに 1 文。アクターは `act:actor_proxy("アクター")` で得たプロキシで書く。発言は `act:actor_proxy("アクター"):talk(文字列)`、単語参照は `act:actor_proxy("アクター"):talk(act:actor_proxy("アクター"):word(名前))`、動的単語参照は `act:actor_proxy("アクター"):talk(act:actor_proxy("アクター"):word(var.変数名, "var.変数名"))`、さくらスクリプトは `act:actor_proxy("アクター"):sakura_script(文字列)` |
| 選択肢行 | `act:choice(ジャンプ先, 表示テキスト)` |
| キューコマンド行 | `!select` だけを `act:choice_timeout(秒数)`（引数が数値でなければ `nil`）に変換する。他のキューコマンドは出力しない |

変数参照のアクションは、値と変数の経路の文字列を渡す `act:actor_proxy("アクター"):talk(var.名前, "var.名前")` になり、プロパティ参照は `tostring(act:get_property(名前))` を話す。関数呼び出しのアクションは、戻り値の先頭だけを話すよう括弧で包む（`act:actor_proxy("アクター"):talk((act:actor_proxy("アクター"):expr_fn(名前, 引数…)))`、グローバル関数は `act:global_fn("名前", 引数…)`、動的関数呼び出しは `act:actor_proxy("アクター"):expr_fn_var(var.変数名, "var.変数名", 引数…)`）。式の中の関数呼び出しはアクターを付けず、`act:expr_fn(名前, 引数…)`・`act:global_fn("名前", 引数…)`・`act:expr_fn_var(var.変数名, "var.変数名", 引数…)` になる。キーワード引数は値だけを位置で渡す。

アクター名と `＠＊` の関数名は、`StringLiteralizer::literalize` で文字列リテラルにして渡す（`act:actor_proxy("ぱすた")`・`act:global_fn("関数名", 引数…)`）。名前を Lua の添字（`act.名前`・`GLOBAL.名前`）にしないため、act のメソッド名・フィールド名や Lua の予約語と同じ名前でも、別のものに解決されず、構文エラーにもならない。

エスケープのアクションは `talk` で出力する。`＠＠`・`＄＄` は 2 文字目の 1 文字を、`\\` は 2 文字のままを話す（`C:\\new` は `talk("C:")`・`talk([[\\]])`・`talk("new")` の 3 文になる）。`\\` の 2 文字はさくらスクリプトの後処理で 1 つのタグとして扱われ、間にウェイトなどが入らない（[さくらスクリプトの後処理](talk-output.md#さくらスクリプトの後処理)）。

式の算術（`＋`・`－`・`＊`・`／`・`％`）は演算ごとに `act:arith("演算子", 左, 右, 左の説明, 右の説明)` の呼び出しに、連結（`＆`）は演算ごとに `act:concat(左, 右, 左の説明, 右の説明)` の呼び出しになる。パーサは優先順位を付けずに左結合の木を作る（`1＋2＊3` は `(1＋2)＊3`、`「x」＆1＋2` は `(「x」＆1)＋2` の形）ため、`expr_gen.rs` の `binary_to_string` が木を項と演算子の列に戻し、優先順位の高い段から順に、段ごとに左から畳んで入れ子にする。段は `＊`・`／`・`％` → `＋`・`－` → `＆` の 3 段で、段の高さは `precedence` だけが、演算ごとの生成形は `binary_node` だけが決める。算術の 2 段は Lua の優先順位・結合と同じである。括弧（`Paren`）は 1 つの項で、`( … )` で囲んだまま出力する。

```text
＄a＝＄x＋1         → var.a = act:arith("+", var.x, 1, "var.x")
＄b＝1＋2＊＄y      → var.b = act:arith("+", 1, act:arith("*", 2, var.y, nil, "var.y"))
＄c＝（＄x＋1）＊2  → var.c = act:arith("*", (act:arith("+", var.x, 1, "var.x")), 2)
＄d＝＠＊f（1）＋＠g（） → var.d = act:arith("+", act:global_fn("f", 1), act:expr_fn("g"), "@*f()", "@g()")
＄表示＝「合計」＆＄n＆「個」 → var.表示 = act:concat(act:concat("合計", var.n, nil, "var.n"), "個")
＄s＝「合計」＆＄a＋＄b → var.s = act:concat("合計", act:arith("+", var.a, var.b, "var.a", "var.b"))
＄n＝（「1」＆「2」）＋1 → var.n = act:arith("+", (act:concat("1", "2")), 1)
＞＄種類＆「_挨拶」  → return act:call(SCENE.__global_name__, act:call_key(act:concat(var.種類, "_挨拶", "var.種類")), {}, table.unpack(args))
```

説明は、実行時の警告に被演算子の場所を出すための文字列リテラルである（`binary_operand` が `operand_desc` の結果を文字列リテラルにする）。変数参照は変数の経路（`"var.x"`・`"save.x"`・`"args[1]"`）、関数呼び出しは `"@名前()"`・`"@*名前()"`・`"@$変数の経路()"`、括弧は中身の説明になり、リテラルと入れ子の演算（算術・連結）には説明が無い。両方とも無ければ説明の引数を省き、右だけあるときは左に `nil` を置く。省き方は `act:arith` と `act:concat` で同じである。入れ子の演算に説明が無いため、内側が失敗して `nil` を返したとき、外側の演算は警告を重ねない。`act:arith`・`act:concat` の実行時の振る舞いは [arith](../lua/script-api.md#arithop-lhs-rhs-lhs_desc-rhs_desc)・[concat](../lua/script-api.md#concatlhs-rhs-lhs_desc-rhs_desc) が正である。

動的参照の生成（`element_gen.rs` の `dynamic_ref_args`）は、変数の値を `tostring` せずにそのまま渡し、変数の経路（`var.変数名`・`save.変数名`・`args[番号+1]`）を文字列リテラルにして続けて渡す。値の検査と検索キーへの変換は実行時に行う（[Lua ランタイム内部モジュール](internal-modules.md#動的参照のキーworddynamic_key)）。ここで生成した `act` のメソッドが実行時に何をするかは、[ランタイム実行モデル](execution-model.md) と [Lua ランタイム内部モジュール](internal-modules.md) で扱う。

#### 動的コールのキー

動的ターゲットの Call（`＞＄名前`・`＞＠名前（）` など）のキーは、`element_gen.rs` の `call_key` が、式を括弧の内側まで見て次の 3 つの形にする。式の値は `tostring` しない。`nil` の判定と検索キーへの変換は実行時の `act:call_key` が行う（[Call のキーと失敗表記](internal-modules.md#call-のキーと失敗表記call_keyfailure)）。

| 式 | キーの生成形 |
| -- | ------------ |
| 変数参照 1 つ（`＄x`・`＄＊x`・`＄０`） | `act:call_key(var.x, "var.x")`（引数は `dynamic_ref_args` と同じ） |
| 関数呼び出し 1 つ（`＠f（）`・`＠＊f（）`・`＠＄v（）`） | `act:call_key(act:expr_fn("f"), nil, "@f()")`（説明は `operand_desc`） |
| それ以外（文字列・数値・算術・連結） | `act:call_key(式)` |

キーの式は呼び出しの第 2 引数のため、引数の式より先に評価される。静的ターゲットのキーは文字列リテラル `"名前"` で、`act:call_key` を通らない。

### 生成時最適化

生成時に行う最適化と、それに類する生成上の扱いは次の 3 つである。

#### 末尾呼び出し

ローカルシーン（暗黙の開始ブロックと名前付きローカルシーンの両方）の項目列の**最後の項目が Call** の場合に限り、その Call 文を `return act:call(…)` にする。それ以外の Call（途中の Call）は `act:call_restore(…)` にする。判定は `scope_gen.rs` の `generate_local_scene_items` が行い（`is_callable_item` で Call かどうかを見る）、`element_gen.rs` の `generate_call_scene` が `is_tail_call` を受けて `return act:call` と `act:call_restore` を選ぶ。2 つの形は引数の並びが同じで、キーの形（静的・動的）とは独立している。

Call の後にアクション行などが続く場合、および最後の項目が Call 以外の場合の Call は、途中の Call になる。

```text
＞サブ                 → return act:call(SCENE.__global_name__, "サブ", {}, table.unpack(args))
＞サブ（Call の後に続きあり） → act:call_restore(SCENE.__global_name__, "サブ", {}, table.unpack(args))
```

Lua は末尾位置の関数呼び出しで呼び出し元のスタックフレームを再利用する。`act:call` の実装も見つけたシーン関数を `return handler(self, ...)` の末尾位置で呼ぶため、シーンの末尾で次々に Call してもスタックは深くならない。`return` が付いた Call は呼び出し先の戻り値をそのまま返す。末尾の Call は戻る行が無いため、シーン文脈を戻さない。

途中の Call の `act:call_restore` は、`act:call` から戻った後に `current_scene` を呼ぶ前の値に戻す（[シーン文脈の復元](internal-modules.md#シーン文脈の復元call_restorerestore_scene)）。戻った後に処理があるため、途中の Call は呼び出しの深さを 1 つ増やす。呼ばれた側の中の末尾の Call の連鎖は深くならない。

#### 継続行の話者引継ぎ

継続行（`：…`）のアクションは、同じローカルシーン内で直前に処理したアクション行のアクターの発言として生成する。`generate_local_scene_items` がローカルシーンごとに `last_actor` を持ち、アクション行を処理するたびに更新し、継続行ではその値をアクターとして使う。変数代入・Call・キューコマンド・選択肢は `last_actor` を変えない。

`last_actor` はローカルシーンごとに空から始まるため、先行するアクション行が無い位置の継続行は `TranspileError::InvalidContinuation` になる。

これは生成コードを小さくする最適化ではなく、継続行の話者を決めるための構文上の処理である。生成されるのはアクションごとの `act:actor_proxy("アクター"):…` の文であり、同じアクターが続いても文は省略しない。連続する同じアクターの発言をまとめる処理は実行時のトーク組立が担う（[トーク出力とアピアランス](talk-output.md)）。

#### 文字列リテラルの表記選択

AST の文字列（発言・単語の値・名前など）は `StringLiteralizer::literalize` で Lua のリテラルに変換する。規則は上から順に判定する。

0. 文字列が改行（CR・LF）を含めば、`"…"` の形にし、`\`→`\\`・`"`→`\"`・CR→`\r`・LF→`\n` を 1 回の走査で置き換える。出力は生の改行を含まない 1 行になる。ロングブラケットを使わないのは、Lua がその中の CR・CRLF を LF に正規化し、先頭の改行を捨てるため、値が変わるからである。文法は引用文字列に改行を含めないため、パーサ経由では通らない多層防御の規則である。
1. 文字列が `\` も `"` も含まなければ、`"…"` の形にする。日本語などの非 ASCII 文字や `]` を含むだけでは、この形のままである。
2. `\` か `"` を含む場合は、エスケープを使わずに内容をそのまま保つため、ロングブラケット `[[…]]` の形にする。`=` の数は 0 から順に試し、内容に閉じ括弧の前半（`]` の後に同じ数の `=`）が現れない最小の数を使う。たとえば `\s[0]` は `]` を含むので `[=[\s[0]]=]` になる。
3. `=` を 10 個まで試しても安全な数が無い場合は `TranspileError::StringLiteralError` になる。

さくらスクリプトのように `\` を含む値がロングブラケットで出力されるのは、この規則による。

### 出力の正規化

中間バッファに生成したコードは、`normalize_output_with_shift` で正規化してから書き出す。

1. CRLF を LF に変換する。
2. 空白だけの行のうち、その後に続く最初の空白でない行が `end` だけの行であるものを削除する（ブロックの閉じ直前の空行を除く）。
3. 末尾の空白・改行を取り除き、最後に改行を 1 つだけ付ける。

正規化は行の削除だけを行い、行の挿入・結合はしない。削除した行の番号（正規化前の 1 始まりの番号）は `LineShift` に記録され、正規化前の行番号を最終的な `.lua` の行番号へ写すのに使われる。`normalize_output` は同じ処理の文字列だけを返す薄い包みであり、出力のバイト列はどちらを通しても同一である。

### トランスパイル結果キャッシュ

トランスパイル結果は、ゴーストのディレクトリ内のキャッシュ（既定は `profile/pasta/cache/lua`。設定は [pasta.toml リファレンス](../reference/pasta-toml.md)）へ書き出し、次回の起動で再利用する。ローダの増分処理は次の順に進む。

1. **版の確認**（`CacheManager::prepare_cache_dir`）: キャッシュ内の `.cache_version` に記録した版が `pasta_lua` クレートの版と異なれば、キャッシュディレクトリを丸ごと削除して作り直し、現在の版を書き込む。
2. **更新判定**（`needs_transpile`）: 対象ファイルのキャッシュが無いか、ソースの更新時刻がキャッシュより新しければトランスパイルする。更新時刻を取得できない場合もトランスパイルする。最新のファイルはパースもせずに省く。
3. **パースとトランスパイル**: `.pasta` を読み込んでパースし、`LuaTranspiler::transpile` で生成する。得られた `TranspileContext` は全ファイル分を統合する（`TranspileContext::merge_from`。ファイル属性は統合しない）。
4. **キャッシュへの保存**（`save_cache`）: 生成した Lua を UTF-8 で書き出す。基準ディレクトリの外にあるソースや、相対パスに `..` を含むソースは拒否する。`.pasta` の保存失敗は警告にとどめて続行する。
5. **`.lua` の素通し**: `.pasta` の検出パターンから作った `.lua` のパターンに一致するファイルは、トランスパイルせずにそのままキャッシュへコピーする。同じモジュール名の `.pasta` がある場合は `.lua` を無視する。
6. **失敗の集約**: 読み込み・パース・トランスパイル・`.lua` のコピーの失敗は全ファイルを処理し終えてからまとめ、1 件でもあればロードを失敗させる。
7. **`scene_dic.lua` の生成**（`generate_scene_dic`）: 省いたファイルも含めた全モジュールの名前を並べ替えて `require` し、最後に `require("pasta").finalize_scene()` を呼ぶ `scene_dic.lua` を毎回生成する。これが第 2 段の入口になる。

キャッシュ先と `pasta.scene.*` のモジュール名の導出は [モジュール名の生成](loader.md#モジュール名の生成) で扱う。`scene_dic.lua` はキャッシュ直下の `pasta/scene_dic.lua` に置かれ、`require("pasta.scene_dic")` で読み込まれる。

ソースが削除されて対応を失ったキャッシュファイルは `find_orphaned_caches` で検出してログに出すが、削除はしない。ローダの各段階の順序とモジュールの検索パスは [ローダ自己展開とモジュール解決](loader.md) で扱う。

### ソースマップ生成の出入口

トランスパイラの入口は、ソースマップの記録の有無で 3 つある。

| 入口 | シンク | 戻り値 | 呼び出し元 |
| ---- | ------ | ------ | ---------- |
| `transpile` | なし | `TranspileContext` | ローダの増分処理 |
| `transpile_with_sink` | 任意 | `TranspileContext` | `transpile` が経由する |
| `transpile_with_source_map` | 任意 | `TranspileContext` と `LineShift` | 上の 2 つの実体。デバッグ時のソースマップ構築 |

`LuaCodeGenerator` は、出力した行数を `out_line` で数え、シンクが付いていれば、`Span` を持つ構文要素（スコープの見出し・アクション・変数代入・Call・選択肢・キューコマンド・グローバルとシーンの単語定義）を出力した直後に「生成した Lua の行 → `.pasta` の位置」の対応を記録する。Lua ブロックは内容の行ごとに `.pasta` の行と対応づける。シンクが付いていなければ記録は何もしない。

シンクの有無で出力のバイト列は変わらない。デバッグが有効なとき、ローダはトランスパイルの後に全 `.pasta` をもう一度パースし、シンクを付けて `transpile_with_source_map` で生成し直し、返された `LineShift` で記録を最終的な行番号へ写す。ソースマップの構造・サイドカー・解決は [デバッグ基盤とシーンキック](debug.md) で扱う。

## 境界の受け渡し

| 境界 | 渡す側 → 受ける側 | 渡すもの | 所有 |
| ---- | ----------------- | -------- | ---- |
| `pasta_dsl` → `pasta_lua` | パーサ → トランスパイラ | `PastaFile`（AST） | トランスパイラは借用するだけで、AST を変更しない |
| トランスパイラ → ローダ | `LuaTranspiler` → 増分処理 | 生成した Lua のバイト列と `TranspileContext` | ローダが Lua をキャッシュへ書き出し、`TranspileContext` を統合して保持する |
| `pasta_core` ↔ `pasta_lua` | レジストリ型の利用 | `SceneRegistry`・`WordDefRegistry` | `TranspileContext` が所有する。名前の置換規則 `SceneRegistry::sanitize_name` は生成器も共有する |
| ローダ → ランタイム | 増分処理 → VM の構築 | 統合した `TranspileContext` | VM の構築時に `@pasta_search` の初期値として渡り、消費される |
| Rust → Lua（第 1 段 → 第 2 段） | キャッシュ → Lua VM | `pasta.scene.*` モジュールと `scene_dic.lua` | 生成コードの実行で Lua 側がシーン・単語の登録を所有する。`finalize_scene`（Rust）がそれを収集し、検索用のレジストリを作り直す |
| 生成器 → デバッグ基盤 | `LuaCodeGenerator` → `SourceMapSink` の実装 | 行の対応の記録 | シンクは呼び出し元が所有し、生成器は可変参照を借りる。`code_gen` は `debug` に依存しない |

## 不変条件と制約

- 生成される Lua コードは、ランタイムの Lua 方言である LuaJIT 2.1 で実行できる形でなければならない。生成コードは `table.unpack` を使うため、Lua 5.2 互換を有効にした LuaJIT（ルートの [Cargo.toml](https://github.com/ekicyou/pasta/blob/main/Cargo.toml) の `mlua` の `luajit52` 機能）を前提とする。
- ローカルシーンの関数名は Lua の識別子としてそのまま出力される（`SCENE.サブ_1`）。LuaJIT が 0x80 以上のバイトを識別子の文字として受け付けることを前提にしている。アクター名と `＠＊` の関数名は識別子にせず、文字列リテラルにして渡す（`act:actor_proxy("ぱすた")`・`act:global_fn("名前", …)`）。文法の識別子は `"` や `\` を含まないため、`"…"` の中にもそのまま埋め込む。
- 名前付きローカルシーンの関数名には常に `_番号` が付くため、利用者のローカルシーン名が `SCENE.__start__`・`SCENE.__global_name__` と衝突することはない。
- シンクを付けても付けなくても、`normalize_output` を通しても `normalize_output_with_shift` を通しても、書き出されるバイト列は同一である。出力の正規化は行の削除だけを行う。
- 末尾呼び出しの `return act:call(…)` は、ローカルシーンの最後の項目が Call のときにだけ出力する。それ以外の Call は `act:call_restore(…)` になる。どちらも Call 1 つにつき 1 行である。
- 継続行には、同じローカルシーン内で先行するアクション行が必要である。親のグローバルシーンや別のローカルシーンからは引き継がない。
- ロングブラケットの `=` は最大 10 個である。
- トランスパイラは、参照先のシーン・単語の存在も、Lua ブロックの内容も検査しない。シーンと単語は実行時に検索で解決され、Lua ブロックの誤りはロード時の Lua エラーになる。`TranspileError` には未定義シーン・未定義単語を表す型も定義されているが、現行のコード生成はこれらを返さない。
- キャッシュの無効化は、クレートの版の変化とソースの更新時刻だけで判断する。同じ版のままコード生成を変えた場合、ソースが更新されない限り既存のキャッシュが使われる。
- 第 1 段の `TranspileContext` のレジストリは検索の権威ではない。検索に使うレジストリは第 2 段の `finalize_scene` が作り直したものである。

## ソースの所在

- `crates/pasta_dsl/src/`
- `crates/pasta_lua/src/transpiler.rs`
- `crates/pasta_lua/src/code_gen/`
- `crates/pasta_lua/src/context.rs`
- `crates/pasta_lua/src/normalize.rs`
- `crates/pasta_lua/src/string_literalizer.rs`
- `crates/pasta_lua/src/config.rs`
- `crates/pasta_lua/src/error.rs`
- `crates/pasta_lua/src/loader/cache.rs`
- `crates/pasta_lua/src/loader/process.rs`

テストは `crates/pasta_dsl/tests/`・`crates/pasta_lua/tests/transpiler/` にあり、末尾呼び出しの用例は `crates/pasta_lua/tests/fixtures/tail_call_optimization.pasta`、生成形の基準は `crates/pasta_lua/tests/fixtures/sample.expected.lua` にある。

## 経緯

- [dsl-separation](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/dsl-separation) — DSL パーサと AST の独立クレート化
- [pasta-lua-cache-transpiler](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-lua-cache-transpiler) — トランスパイル結果キャッシュと `scene_dic.lua`
- [pasta-scene-dictionary-finalization](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-scene-dictionary-finalization) — 実行時の辞書確定（第 2 段）への移行
- [transpiler-ctx-instead-of-act](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/transpiler-ctx-instead-of-act) — シーン関数の `act` を受け取る生成形
- [pasta-transpiler-variable-expansion](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-transpiler-variable-expansion) — 変数展開の生成
- [actor-word-dictionary](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/actor-word-dictionary) — アクター辞書の単語定義の生成
- [pasta-source-map](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-source-map) — 生成器のソースマップ記録と `LineShift`

---

流れはつかめまして？ フンッ、この程度で目を回すようでは困りますわよ。
次は、登録されたシーンと単語がどう探し出されるのかを見て参りましょう！
