# トランスパイルパイプライン

> 【目閉じ】作者の綴った `.pasta` が、Lua のコードへ化ける錬金術――パースから登録、生成、そしてキャッシュまで、その工程を一本の流れとして追いかけますわ。

> 【アンソニー】古い資料には、登録と生成を二度に分けて煮込む「2 パス」とございましたが。

> 【したり顔】そこが、いまの実装との違いですの。「2 パス」はもう使われていない呼び名で、ファイルを文書順に一度たどるだけで、登録も生成も済ませてしまいますのよ。

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

文法そのもの（何が書けて何がエラーになるか）は利用者向けの [文法リファレンス](../grammar/index.md) が正であり、この章では再記述しない。実行時の辞書確定の内部（Lua 側の収集とレジストリの再構築、検索）は [シーン・単語レジストリとシーン検索](registry-search.md)、起動シーケンス全体の中でトランスパイルがどこで呼ばれるかは [ローダ自己展開とモジュール解決](loader.md)、シーンマップの作り方は [開発用のイベントとシーンマップ](debug.md#シーンマップ) で扱う。

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
| 部分パース | `crates/pasta_dsl/src/partial.rs` | `parse_str_partial`。全体のパースに失敗したときに、字下げしない行頭の印でソースを塊に区切り、塊ごとに解析する。解けない塊は行の範囲だけを返す。エディタ拡張の色分け用であり、ランタイムのトランスパイルでは使わない |

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
| `TranspileContext` | `crates/pasta_lua/src/context.rs` | 走査中の状態。`SceneRegistry`・`WordDefRegistry`（`pasta_core`）を持ち、受け継いだ属性でシーンを登録する |
| `LuaCodeGenerator` | `crates/pasta_lua/src/code_gen/mod.rs` | 出力先・インデント・出力行番号（`out_line`）・ソースマップ用シンクを持つ生成器本体。ヘッダ出力と行出力の共通処理 |
| スコープ単位の生成 | `crates/pasta_lua/src/code_gen/scope_gen.rs` | アクター辞書・グローバルシーン・ローカルシーンの Lua ブロック、末尾呼び出しの判定、選択肢 |
| 要素単位の生成 | `crates/pasta_lua/src/code_gen/element_gen.rs` | アクション・変数代入・Call・単語定義・Lua ブロック、変数の経路と動的参照の引数 |
| 式の生成 | `crates/pasta_lua/src/code_gen/expr_gen.rs` | 式の値・関数呼び出し・引数リストと、算術・連結の連鎖の組み直し |
| ソースマップのシンク | `crates/pasta_lua/src/code_gen/source_map.rs` | 生成器が対応を記録する先の `SourceMapSink` トレイト |
| 出力の正規化 | `crates/pasta_lua/src/normalize.rs` | `normalize_output_with_shift` と、削除行の記録 `LineShift` |
| 文字列リテラル化 | `crates/pasta_lua/src/string_literalizer.rs` | `StringLiteralizer`。文字列を Lua のリテラル表記に変換する |
| 設定 | `crates/pasta_lua/src/config.rs` | `TranspilerConfig`（`scene_aliases`） |
| エラー | `crates/pasta_lua/src/error.rs` | `TranspileError` |

`TranspilerConfig` の設定は `scene_aliases` だけである。`scene_aliases` はグローバルシーン名の別名表で、既定は空の表（置き換えない）である。ローダは pasta.toml から決めた別名表を渡す（[登録と生成](#登録と生成単一走査)）。

### キャッシュ

| 要素 | 所在 | 役割 |
| ---- | ---- | ---- |
| `CacheManager` | `crates/pasta_lua/src/loader/cache.rs` | キャッシュディレクトリの版と別名表の指紋の管理、更新判定、キャッシュの更新時刻の写し、キャッシュ先とモジュール名の導出、`scene_dic.lua` の生成、孤立キャッシュの検出 |
| 増分処理 | `crates/pasta_lua/src/loader/process.rs` | ローダの増分処理。ファイルごとに更新判定・パース・トランスパイル・キャッシュ保存・更新時刻の写しを行う |

## 処理とデータの流れ

### 段階構成

シーンと単語が検索可能な辞書になるまでの処理は、次の 2 段から成る。

```text
第 1 段: トランスパイル時の単一走査（登録と生成）    … ローダの増分処理
  .pasta ──parse_str──▶ AST ──LuaTranspiler──▶ Lua ソース ──▶ キャッシュ
                              │ 項目ごとに「登録 → 生成」
                              ▼
                        TranspileContext（ファイルごと。統合しない）

第 2 段: 実行時の辞書確定                         … Lua VM の構築後
  require("pasta.scene_dic")
    ├ require("pasta.scene.…") × 全モジュール
    │   生成コードの実行で PASTA.create_actor / create_scene / create_word が
    │   Lua 側のレジストリへ登録する（シーンの属性は PASTA.set_scene_attrs が
    │   STORE.scene_attrs へ書く）
    └ require("pasta").finalize_scene()
        Lua 側の登録と属性を収集して SceneRegistry / WordDefRegistry を作り直し、
        @pasta_search を登録し直す
```

第 1 段では、`LuaTranspiler` がファイルの項目を文書順に 1 回だけたどり、各項目について `TranspileContext` への登録と Lua コードの生成を続けて行う。シーンを全部集めてから生成に入るといった、走査を分けた構成にはなっていない。

第 2 段では、第 1 段の生成コードを Lua VM で実行することで Lua 側のシーン・単語の登録が作られ、`finalize_scene` がそれを収集して、実行時に検索で使うレジストリを確定する。実行時のグローバルシーンの通し番号（同名のグローバルシーンを区別する `メイン_1`・`メイン_2` の番号）もこの段で、Lua 側の `create_scene` が全モジュールを通して振る。登録名の構成と分ける規則は [登録名の構成](registry-search.md#登録名の構成) で扱う。

第 1 段の `TranspileContext` はファイルごとに使い終えたら捨て、VM へは渡さない。VM の構築時の `@pasta_search` は、空の登録表で一度登録される（`PastaLuaRuntime::from_loader_with_scene_dic` が空の `TranspileContext` を渡す）。そのため finalize の前の検索は、キャッシュの状態や直前に編集した辞書によらず、どれも見つからない。検索で使われるのは、第 2 段の `finalize_scene` が Lua 側の登録から作り直して登録し直したレジストリである。

従来「2 パス」（Pass 1: シーン登録、Pass 2: コード生成）と呼ばれてきた区分は、現行実装には存在しない。現行の構成で旧称に対応させるなら、トランスパイル時の登録と生成は第 1 段の単一走査の中で項目ごとに行われ、検索に使う辞書の登録は第 2 段の実行時の辞書確定で行われる。「2 パス」という語を見かけた場合は、この 2 段を指すものと読み替える。

### パースと AST 構築時の正規化

`parse_str` は先頭の BOM を外した後、pest の前に `check_paren_depth`（`parser/mod.rs`）で各行の括弧の重なりを数える。`（`・`(` で 1 段深くなり、`）`・`)` で 1 段浅くなり（0 より下げない）、行が変わると 0 に戻る。台詞・文字列・コメント・Lua ブロックの中の括弧も数える。重なりが `MAX_PAREN_DEPTH`（64）を超えたら、65 段目の開き括弧の位置の構文の誤り（`ParseError::SyntaxError`）を返す。pest の再帰は開き括弧 1 つで 1 段深くなるので、この走査で解析と AST の片付けのスタックのあふれを防ぐ。行と桁は pest の行の索引と同じく、`'\n'` だけで行を分け、桁は文字の数で数える。

式の鎖を組む 3 つの関数（`parse_var_set`・`parse_expr_from_parts`・`parse_key_arg`）は、鎖を組む前に `check_op_count`（`parser/parse_action.rs`）で、渡された節の子孫の二項演算子（`＋`・`－`・`＊`・`／`・`％`・`＆`）を記述順に数える。1 つの式は、式を書ける位置（変数代入の右辺・Call の動的ターゲット・関数呼び出しと Call の 1 つの引数）に書いた式の全体で、その中の括弧と関数呼び出しの引数の中の式を含む。数が `MAX_EXPR_OPS`（256）を超えたら、257 個目の演算子の位置の構文の誤り（`ParseError::SyntaxError`）を返す。左結合の鎖は演算子 1 つで 1 段深くなり、AST の片付けとコード生成はその深さだけ再帰するので、この検査で深い木を作らせない。

括弧の走査を通れば、pest で `file` 規則を解析し、失敗すれば `ParseError::SyntaxError` を返す。成功すれば `build_ast` が解析結果を記述順にたどって `PastaFile` を組み立てる。AST を組み立てる時点で、次の正規化を行う。

- `file_scope` の属性と単語定義を、それぞれ `FileItem::FileAttr`・`FileItem::GlobalWord` に平坦化し、他の項目との記述順を保つ。
- 名前を省いたグローバルシーン（`＊` だけの行）は、直前のグローバルシーンの名前を引き継ぎ、`is_continuation` を真にする。ファイルの先頭で現れた場合はパースエラーにする。
- 全角の数字・マイナス・小数点を半角に直してから数値として解釈する（`normalize_number_str`）。式の数値、属性値、アクター番号、キューコマンドの引数、`＄０` などの引数番号が対象である。
- 数の字句（`number_literal`）の値は、式・属性値・キューコマンドの引数のどこでも `number_value` の 1 つの規則で決める。小数点が無く `i64` に収まれば整数、収まらなければいちばん近い小数（Lua と同じ）、小数点があれば小数である。有限の小数に収まらない数は、字句の始まり（符号を含む）の位置の構文の誤り（`ParseError::SyntaxError`）にする。
- 空の引用（`string_blank` 規則の `「」`・`""`）は、囲み文字ではなく空文字列として記録する。単語値は `""` の候補、属性値は `AttrValue::String("")`、キューコマンドの引数は空の文字列リテラル、式は `Expr::BlankString` になる。
- シーンのアクター指定行のアクター番号を採番する。番号を明示した項目はその値、省いた項目は直前の番号に 1 を足した値（先頭で省いた場合は 0）になる。明示した番号が 0〜4294967295（`u32`）に収まらなければ、番号（`digit_id`）の始まりの位置の構文の誤り（`ParseError::SyntaxError`）にする。4294967295 の次に省いた番号は 4294967295 に留まる。
- 引数番号（`＄０` などの数字の名前）は `VarScope::Args(番号)` になる（`parse_action.rs` の `parse_var_ref_local_inner`。アクション行・式の項・動的な単語参照と関数呼び出しの名前のどれもここを通る）。番号が 0〜255（`u8`）に収まらなければ、番号（`digit_id`、`＄` の次）の始まりの位置の構文の誤り（`ParseError::SyntaxError`）にする。

### 登録と生成（単一走査）

`LuaTranspiler::transpile_with_source_map` は、中間バッファに向けた `LuaCodeGenerator` を作り、ヘッダ（`pasta` と `pasta.global` の `require`）を出力したあと、`PastaFile.items` を記述順に処理する。

| 項目 | 登録（`TranspileContext`） | 生成 |
| ---- | -------------------------- | ---- |
| `FileAttr` | なし（受け継ぎは走査の前に `PastaFile::global_scene_attrs` が決める） | なし |
| `GlobalWord` | 名前ごとにグローバル単語として登録する | `PASTA.create_word(名前):entry(値, …)` をファイルの最上位に出力する |
| `ActorScope` | 名前ごとにアクター単語として登録する | アクターごとの `do` 〜 `end` ブロック |
| `GlobalSceneScope` | 宣言名を決め、その名前と受け継いだ属性の並びでグローバルシーンを登録し、同名の出現順の番号を得る。シーンレベルの単語をそのシーンのローカル単語として登録する | 同じ属性の並びを渡して、シーンごとの `do` 〜 `end` ブロックを出力する。生成後、名前付きローカルシーンを、生成器と同じ `local_scene_counters` の通し番号と、その宣言行の属性で登録する |

グローバルシーンの宣言名は、`process_global_scene` の冒頭で 1 回だけ決める。書いた名前（単独の `＊` 行ではパーサが引き継いだ名前）が `TranspilerConfig.scene_aliases` の別名に完全一致すれば置き換え先の名前、そうでなければ書いた名前である。登録・シーンレベルの単語のモジュール名・生成コードの `create_scene` の名前・名前付きローカルシーンの親の名前は、すべてこの宣言名から作る。そのため既定の表のもとで `＊会話` は `OnTalk` のシーンとして登録され、生成コードも `OnTalk` で `create_scene` を呼ぶ（規則は [シーン名の別名](../grammar/call-jump.md#シーン名の別名)）。

グローバルシーンの属性の並びは、走査の前に `PastaFile::global_scene_attrs`（`crates/pasta_dsl/src/parser/ast/attr_merge.rs`）がファイルの中のグローバルシーンごとに 1 本ずつ作り、走査は `GlobalSceneScope` に出会うたびに次の 1 本を使う。並びは、その位置のファイル属性の行 → シーンの属性（宣言行の付記 → 初期部の属性行）を `merge_attrs` で畳んだものである。ファイル属性の行は、次のファイル属性の行までのグローバルシーンに付き、次の行で丸ごと置き換わる（ファイル属性の行を 2 行続けて書くことはパーサが拒む）。ファイル属性は別のファイルへは持ち越さない。

`merge_attrs(inherited, own)` は `inherited` → `own` の順に並べ、宣言は同じ名前を先の位置に 1 つ残して値を後のものにし、条件とフラグは畳まずに足す。ローカルシーンは何も受け継がず、`merge_attrs(&[], 宣言行の属性)` の並びになる。登録（`TranspileContext`）では、並びを `scene_attrs_of` で `SceneAttr`（宣言・条件・フラグ）に写して `SceneRegistry` に渡す。生成器は同じ並びを属性の文にする（[属性の文](#属性の文)）。利用者向けの規則は [属性](../grammar/block-structure.md#属性) が正である。

### 生成される Lua コードの形

次の入力（ゴーストの `dic\メイン.pasta`）を例に、生成されるコードの形を示す。

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
        local save, var = act:init_scene(SCENE, "メイン", nil, [[dic\メイン.pasta|7:2,11:2,12:3]])

        act:actor_proxy("ぱすた"):talk("こんにちは")
        return act:call(SCENE.__global_name__, "サブ", {}, table.unpack(args))
    end

    function SCENE.サブ_1(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE, "メイン", "サブ", [[dic\メイン.pasta|15:5,19:6]])

        act:actor_proxy("ぱすた"):talk("サブです")
    end
end
```

スコープごとの形は次のとおりである。

- **アクター辞書**: `local ACTOR = PASTA.create_actor("名前")` に続けて、単語定義ごと・名前ごとに `ACTOR:create_word(名前):entry(値, …)` を出力する。配下の Lua ブロックは、言語識別子が `lua` のものだけを出力する。属性行と変数代入行は出力しない。
- **グローバルシーン**: `PASTA.create_scene` には、シーン名のうち Unicode の英字・数字（`char::is_alphanumeric`。かな・漢字を含む）と `_` 以外の文字を `_` に置き換えた基本名を渡す（`SceneRegistry::sanitize_name`）。番号は付けない。シーンに属性があれば、同じ行の後ろに `; PASTA.set_scene_attrs(SCENE, "関数名", { 項目, … })` を続ける（[属性の文](#属性の文)）。シーンレベルの単語定義は `SCENE:create_word(名前):entry(値, …)` として関数定義より前に出力する。グローバルシーンとローカルシーンに置いた Lua ブロックは、そのグローバルシーンのすべての関数定義の後に、言語識別子にかかわらず出力する。
- **ローカルシーン**: 暗黙の開始ブロックは常に `SCENE.__start__` になり、名前付きローカルシーンは `照合用の名前_通し番号` の関数になる（`SceneRegistry::registered_name`。`・選択・A` は `選択_A_1`）。照合用の名前はローカルシーン名をサニタイズしたもので、通し番号は同じグローバルシーンの中で照合用の名前ごとに数える出現順（1 始まり）である（`crates/pasta_lua/src/context.rs` の `local_scene_counters`）。`・挨拶・1` と `・挨拶_1` はどちらも照合用の名前が `挨拶_1` なので、`挨拶_1_1` と `挨拶_1_2` になる。トランスパイル時のレジストリの `register_local` も同じ通し番号を使うため、レジストリのローカルシーンの登録名は生成した関数名と一致する。どの関数も `local args = { ... }` と `local save, var = act:init_scene(SCENE, …)` で始まる（[init_scene の引数](#init_scene-の引数)）。`__start__` でシーンにアクター指定行がある場合は、続けて `act:clear_spot()` と、アクターごとの `act:set_spot(名前, 番号)` を出力する。
- **Lua ブロック**: 内容の行を書いたとおりに出力する。インデントを足さず、内容の中の空行（`end` の直前の空行も）も残す。

ローカルシーンの項目は、次の形に変換される。

| 項目 | 生成 |
| ---- | ---- |
| 変数代入（ローカル・グローバル） | `var.名前 = 式`・`save.名前 = 式`。右辺が単語参照なら `act:word(名前)`、動的単語参照なら `act:word(var.変数名, "var.変数名")`、プロパティ参照だけなら `act:get_property(名前)` |
| 変数代入（プロパティ） | `act:set_property(名前, 式)`。右辺が単語参照・動的単語参照なら、式の代わりに上と同じ `act:word(…)` を渡す |
| 式文（`＄＝`） | 式が関数呼び出しそのもの（`act:expr_fn(…)`・`act:global_fn(…)`・`act:expr_fn_var(…)`）なら、その呼び出しを 1 文として出力する。それ以外の式（演算・括弧・リテラル・変数参照）は Lua の文にならないため、`do local _ = 式 end` で値を捨てる（`element_gen.rs` の `generate_var_set`） |
| Call | ローカルシーンの最後の項目なら `return act:call(SCENE.__global_name__, キー, {}, 明示した引数…, table.unpack(args))`、それ以外なら `act:call_restore(SCENE.__global_name__, キー, {}, 明示した引数…, table.unpack(args))`（[末尾呼び出し](#末尾呼び出し)）。キーは、静的ターゲットなら `"名前"`、動的ターゲットなら `act:call_key(…)`（[動的コールのキー](#動的コールのキー)） |
| アクション行・継続行 | アクションごとに 1 文。アクターは `act:actor_proxy("アクター")` で得たプロキシで書く。発言は `act:actor_proxy("アクター"):talk(文字列)`、単語参照は `act:actor_proxy("アクター"):talk(act:actor_proxy("アクター"):word(名前))`、動的単語参照は `act:actor_proxy("アクター"):talk(act:actor_proxy("アクター"):word(var.変数名, "var.変数名"))`、さくらスクリプトは `act:actor_proxy("アクター"):sakura_script(文字列)` |
| 選択肢行 | `act:choice(ジャンプ先, 表示テキスト)` |
| キューコマンド行 | `!select` だけを `act:choice_timeout(秒数)`（引数が数値でなければ `nil`）に変換する。他のキューコマンドは出力しない |

変数参照のアクションは、値と変数の経路の文字列を渡す `act:actor_proxy("アクター"):talk(var.名前, "var.名前")` になり、プロパティ参照は `tostring(act:get_property(名前))` を話す。関数呼び出しのアクションは、戻り値の先頭だけを話すよう括弧で包む（`act:actor_proxy("アクター"):talk((act:actor_proxy("アクター"):expr_fn(名前, 引数…)))`、グローバル関数は `act:global_fn("名前", 引数…)`、動的関数呼び出しは `act:actor_proxy("アクター"):expr_fn_var(var.変数名, "var.変数名", 引数…)`）。式の中の関数呼び出しはアクターを付けず、`act:expr_fn(名前, 引数…)`・`act:global_fn("名前", 引数…)`・`act:expr_fn_var(var.変数名, "var.変数名", 引数…)` になる。キーワード引数は値だけを位置で渡す。

アクター名と `＠＊` の関数名は、`StringLiteralizer::literalize` で文字列リテラルにして渡す（`act:actor_proxy("ぱすた")`・`act:global_fn("関数名", 引数…)`）。名前を Lua の添字（`act.名前`・`GLOBAL.名前`）にしないため、act のメソッド名・フィールド名や Lua の予約語と同じ名前でも、別のものに解決されず、構文エラーにもならない。

エスケープのアクションは `talk` で出力する。`＠＠`・`＄＄` は 2 文字目の 1 文字を、`\\`・`\%` は 2 文字のままを話す（`C:\\new` は `talk("C:")`・`talk([[\\]])`・`talk("new")` の 3 文になる）。`\\`・`\%` の 2 文字はさくらスクリプトの後処理で 1 つの単位として扱われ、間にウェイトなどが入らない（[さくらスクリプトの後処理](talk-output.md#さくらスクリプトの後処理)）。

式の算術（`＋`・`－`・`＊`・`／`・`％`）と連結（`＆`）は、演算ごとに Lua の演算子（`+`・`-`・`*`・`/`・`%`・`..`）を括弧で囲んだ `(左 演算子 右)` になる。演算子の前後には必ず空白を 1 つ置く（`10－－3` は `(10 - -3)` になり、`--` が Lua のコメントにならない）。パーサは優先順位を付けずに左結合の木を作る（`1＋2＊3` は `(1＋2)＊3`、`「x」＆1＋2` は `(「x」＆1)＋2` の形）ため、`expr_gen.rs` の `binary_to_string` が木を項と演算子の列に戻し、優先順位の高い段から順に、段ごとに左から畳んで入れ子にする。段は `＊`・`／`・`％` → `＋`・`－` → `＆` の 3 段で、段の高さは `precedence` だけが、演算ごとの生成形は `binary_node` だけが決める。すべての演算を括弧で囲むため、生成形は Lua の優先順位・結合に頼らない。括弧（`Paren`）は 1 つの項で、`( … )` で囲んだまま出力する。

被演算子は、`binary_operand` が Lua の式・説明・生成時に分かる種類（`StaticKind`）の組にして運ぶ。種類は、数値のリテラルと算術の演算が数値（`Number`）、文字列のリテラル・空文字列と連結を含む演算が文字列（`String`）、変数参照・関数呼び出し・動的関数呼び出しが不明（`Unknown`）で、括弧は中身の種類になる。`binary_node` は、演算の文脈と被演算子の種類から、変換の呼び出しを付けるかを決める。

| 文脈 | 被演算子の種類 | 生成形 |
| ---- | -------------- | ------ |
| 算術 | 数値 | そのまま |
| 算術 | 文字列・不明 | `PASTA.num("演算子", 式)`、説明があれば `PASTA.num("演算子", 式, 説明)` |
| 連結 | 文字列 | そのまま |
| 連結 | 数値・不明 | `PASTA.str(式)`、説明があれば `PASTA.str(式, 説明)` |

```text
＄a＝＄x＋1         → var.a = (PASTA.num("+", var.x, "var.x") + 1)
＄b＝1＋2＊＄y      → var.b = (1 + (2 * PASTA.num("*", var.y, "var.y")))
＄c＝（＄x＋1）＊2  → var.c = (((PASTA.num("+", var.x, "var.x") + 1)) * 2)
＄d＝＠＊f（1）＋＠g（） → var.d = (PASTA.num("+", act:global_fn("f", 1), "@*f()") + PASTA.num("+", act:expr_fn("g"), "@g()"))
＄表示＝「合計」＆＄n＆「個」 → var.表示 = (("合計" .. PASTA.str(var.n, "var.n")) .. "個")
＄s＝「合計」＆＄a＋＄b → var.s = ("合計" .. PASTA.str((PASTA.num("+", var.a, "var.a") + PASTA.num("+", var.b, "var.b"))))
＄n＝（「1」＆「2」）＋1 → var.n = (PASTA.num("+", (("1" .. "2"))) + 1)
＞＄種類＆「_挨拶」  → return act:call(SCENE.__global_name__, act:call_key((PASTA.str(var.種類, "var.種類") .. "_挨拶")), {}, table.unpack(args))
＄＝＄未代入＆「x」  → do local _ = (PASTA.str(var.未代入, "var.未代入") .. "x") end
```

説明は、実行時の警告と失敗表記に被演算子の場所を出すための文字列リテラルである（`binary_operand` が `operand_desc` の結果を文字列リテラルにする）。変数参照は変数の経路（`"var.x"`・`"save.x"`・`"args[1]"`）、関数呼び出しは `"@名前()"`・`"@*名前()"`・`"@$変数の経路()"`、括弧は中身の説明になり、リテラルと入れ子の演算（算術・連結）には説明が無い。説明が無ければ説明の引数を省く。関数呼び出しには必ず説明があるため、関数の戻り値が変換の呼び出しの最後の引数になることはなく、関数が複数の値を返しても先頭の 1 つだけが渡る。`PASTA.num`・`PASTA.str` の実行時の振る舞い（`nil` を 0・空文字列にすること、変換できない値の報告）は [PASTA.num](../lua/script-api.md#pastanumop-v-desc)・[PASTA.str](../lua/script-api.md#pastastrv-desc) が正である。

動的参照の生成（`element_gen.rs` の `dynamic_ref_args`）は、変数の値を `tostring` せずにそのまま渡し、変数の経路（`var.変数名`・`save.変数名`・`args[番号+1]`）を文字列リテラルにして続けて渡す。値の検査と検索キーへの変換は実行時に行う（[Lua ランタイム内部モジュール](internal-modules.md#動的参照のキーworddynamic_key)）。ここで生成した `act` のメソッドが実行時に何をするかは、[ランタイム実行モデル](execution-model.md) と [Lua ランタイム内部モジュール](internal-modules.md) で扱う。

#### 属性の文

属性のあるシーンでは、グローバルシーンの `create_scene` の行の後ろに、属性を倉（`STORE.scene_attrs`）へ書く文を続ける（`code_gen/scope_gen.rs` の `append_set_scene_attrs`）。次は、`＆季節：春` の後の `＊メイン`（初期部に `＆＄＊経験値≧500`、ローカルシーン `・選択肢＆優先：3`）の行である。

```lua
    local SCENE = PASTA.create_scene("メイン"); PASTA.set_scene_attrs(SCENE, "__start__", { { key = "季節", value = "春" }, { key = "＄＊経験値", op = ">=", value = 500 } }); PASTA.set_scene_attrs(SCENE, "選択肢_1", { { key = "優先", value = 3 } })
```

- 文は、グローバルシーン自身（`"__start__"`。受け継いだ並び）のぶんと、属性のある名前付きローカルシーン（生成した関数名。宣言行の属性だけ）のぶんを、この順に並べる。属性の無いシーンのぶんは出さない。
- 項目は、宣言が `{ key = 名前, value = 値 }`、条件が `{ key = "＄…", op = "正規形の演算子", value = 値 }`、フラグが `{ key = "＄…" }` である。数は式の数値と同じ表記、文字列は `StringLiteralizer` のリテラルで書く。
- 行を足さずに同じ行へ続けるため、生成コードの行の数と `init_scene` に渡す行の対応は、属性の有無で変わらない。
- 文を作る前に、`優先`・`priority` の宣言の値を確かめる。値が整数（倉の規則で整数になる小数を含む。`pasta_core` の `priority_value`）でなければ、属性の位置の `TranspileError::InvalidPriority` で変換を止める（`「1」` の引用は文字列なので誤り）。
- `PASTA.set_scene_attrs` は `SCENE.set_attrs` そのもので、生成コードのための関数である。倉に書いた並びは、辞書確定が Rust の登録へ運び（[辞書確定](registry-search.md#辞書確定)）、`SCENE.attrs` が Lua から読む（[SCENE.attrs](../lua/script-api.md#sceneattrsglobal_name-local_name)）。

#### init_scene の引数

シーン関数の先頭の `act:init_scene` には、`SCENE` に続けて 3 つの定数を渡す。どれも [止まるエラーの表記](../reference/runtime-failures.md#止まるエラーの表記)で止まった場所を示すためだけのもので、出力・名前の検索には使わない。止まるエラーが起きない要求では、定数を渡して act に控える以外の仕事は増えない。受け取った側の控え方は [init_scene](internal-modules.md#init_scene) で扱う。

| 引数 | 値 |
| ---- | -- |
| 2 つ目 | グローバルシーンの、辞書に書いた名前。別名で置き換える前・サニタイズする前の名前で、単独の `＊` 行では受け継いだ名前（`＊会話` なら `"会話"`。`PASTA.create_scene` に渡す名前とは別） |
| 3 つ目 | ローカルシーンの、辞書に書いた名前（`・挨拶・1` なら `"挨拶・1"`）。`__start__` では `nil` |
| 4 つ目 | 行の対応。`"<.pasta のファイル>\|<.lua の行>:<.pasta の行>,..."` の 1 行の文字列 |

- 名前は `StringLiteralizer` で Lua の文字列にする（[文字列リテラルの表記選択](#文字列リテラルの表記選択)）。同じ名前のシーンが複数あっても、書いた名前だけを渡し、どれかを区別しない。
- 行の対応に入れる `.lua` の行は、そのシーン関数の行（`function` の行から `end` まで）と、同じグローバルシーンのすべての Lua ブロックの行である。Lua ブロックはシーン関数の外（グローバルシーンの末尾）に出力されるため、ブロックの行は同じグローバルシーンのどのシーン関数の対応にも載る。`.lua` の行は整形した後の最終的な行番号、`.pasta` の行は記録した構文要素の始まりの行である。`function` の行は、そのローカルシーンの始まり（暗黙の開始ブロックでは最初の項目、名前付きローカルシーンでは `・` の行）に対応する。
- 記録はソースマップと同じ（[ソースマップ生成の出入口](#ソースマップ生成の出入口)）。`LuaTranspiler::transpile_with_source_map` は、トランスパイルの間だけ小さな表（`LineRecorder`）に記録し、整形の後に `fill_line_maps` が各 `init_scene` の行の `)` の前へ文字列を埋める。行を足さないため、`.lua` の行番号は埋める前と変わらない。
- `.pasta` のファイルは、ローダが渡すゴーストのフォルダーから見たパス（`dic\メイン.pasta`）である。メモリ上でトランスパイルする場合（テストなど）は、パースに渡した名前のまま入る。ファイルに `\` があると、文字列はロングブラケット（`[[…]]`）になる。読み解く側は最後の `|` で切る。
- 引数の無い `act:init_scene(SCENE)` のままの古いキャッシュもそのまま動く。そのときの止まるエラーの表記は、登録名から通し番号を落とした名前で場所を示し、`.pasta` の行は示さない。キャッシュはクレートの版が変われば作り直される（[トランスパイル結果キャッシュ](#トランスパイル結果キャッシュ)）。

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

生成器は、ブロックの間を区切る空行をすぐには書かず、保留にしておく。保留の空行は次に何かを書くときに出力し、ブロックを閉じる `end` を書く前なら捨てる。そのため、生成器が足した空行が `end` の直前に残ることはない。Lua ブロックの内容の空行は保留にせず、そのまま書く。

中間バッファに生成したコードは、`normalize_output_with_shift` で正規化してから書き出す。

1. CRLF を LF に変換する。
2. 末尾の空白・改行を取り除き、最後に改行を 1 つだけ付ける。

正規化が触るのは末尾だけで、途中の行（`end` の直前の空行を含む）はそのまま残す。正規化は行の削除だけを行い、行の挿入・結合はしない。削除した行（末尾の空白だけの行）の番号（正規化前の 1 始まりの番号）は `LineShift` に記録され、正規化前の行番号を最終的な `.lua` の行番号へ写すのに使われる。

### トランスパイル結果キャッシュ

トランスパイル結果は、ゴーストのディレクトリ内のキャッシュ（既定は `profile/pasta/cache/lua`。設定は [pasta.toml リファレンス](../reference/pasta-toml.md)）へ書き出し、次回の起動で再利用する。ローダの増分処理は次の順に進む。

1. **版と別名表の確認**（`CacheManager::prepare_cache_dir`）: キャッシュ内の `.cache_version` に記録した版が `pasta_lua` クレートの版と異なるか、`.scene_alias` に記録した別名表の指紋（`SceneAliasTable::fingerprint`）が今回の別名表の指紋と異なれば、キャッシュディレクトリを丸ごと削除して作り直し、現在の版と指紋を書き込む。`.scene_alias` が無い場合も、指紋が異なるものとして全破棄する。キャッシュの `.lua` には別名で置き換えた宣言名が書かれているため、`.pasta` を変えずに pasta.toml の別名表だけを変えても、古い宣言名のキャッシュは使われない。
2. **更新判定**（`needs_transpile`）: 対象ファイルのキャッシュが無いか、ソースの更新時刻がキャッシュの更新時刻と違えば（新しい・古いを問わない）トランスパイルする。キャッシュの更新時刻は、書き出したときにソースの更新時刻へそろえてある（手順 4）。そのため、前の日付の `.pasta` をバックアップから戻した場合も変換し直される。更新時刻を取得できない場合もトランスパイルする。時刻が同じファイルはパースもせずに省く。
3. **パースとトランスパイル**: ソースの更新時刻を取ってから `.pasta` を読み込み、パースし、`LuaTranspiler::transpile` で生成する。時刻を内容より先に取るのは、読んだ後の書き換えを次の起動で拾うためである。得られた `TranspileContext` はファイルごとに捨てる（[段階構成](#段階構成)）。
4. **キャッシュへの保存**（`save_cache`・`stamp_cache`）: 生成した Lua を UTF-8 で書き出し、続けてキャッシュの更新時刻を、手順 3 で取ったソースの更新時刻にそろえる。基準ディレクトリの外にあるソースや、相対パスに `..` を含むソースは拒否する。書き出しの失敗は、そのファイルの失敗として手順 6 に集め、ロードを失敗させる（前の変換結果のまま動き続けない）。時刻をそろえられなかったときは警告を出して続ける。ソースの更新時刻を取れなかったときは、そろえない。どちらの場合もキャッシュの時刻がソースと違うので、次の起動で変換し直される。
5. **`.lua` の素通し**: `.pasta` の検出パターンから作った `.lua` のパターンに一致するファイルは、トランスパイルせずにそのままキャッシュへコピーし、手順 4 と同じく更新時刻をそろえる。更新判定も手順 2 と同じである。同じモジュール名（大文字と小文字の違いを除く）の `.pasta` がある `.lua` は、検出の時点で除かれている（[ファイル検出](loader.md#ファイル検出)）。
6. **失敗の集約**: 読み込み・パース・トランスパイル・キャッシュへの書き出し（`.pasta` の保存と `.lua` のコピー）の失敗は全ファイルを処理し終えてからまとめ、1 件でもあればロードを失敗させる。
7. **`scene_dic.lua` の生成**（`generate_scene_dic`）: 省いたファイルも含めた全モジュールの名前を並べ替えて `require` し、最後に `require("pasta").finalize_scene()` を呼ぶ `scene_dic.lua` を毎回生成する。これが第 2 段の入口になる。

キャッシュ先と `pasta.scene.*` のモジュール名の導出は [モジュール名の生成](loader.md#モジュール名の生成) で扱う。`scene_dic.lua` はキャッシュ直下の `pasta/scene_dic.lua` に置かれ、`require("pasta.scene_dic")` で読み込まれる。

ソースが削除されて対応を失ったキャッシュファイルは `find_orphaned_caches` で検出してログに出すが、削除はしない。ローダの各段階の順序とモジュールの検索パスは [ローダ自己展開とモジュール解決](loader.md) で扱う。

### ソースマップ生成の出入口

トランスパイラの入口は、ソースマップの記録の有無で 2 つある。

| 入口 | シンク | 戻り値 | 呼び出し元 |
| ---- | ------ | ------ | ---------- |
| `transpile` | なし | `TranspileContext` | ローダの増分処理 |
| `transpile_with_source_map` | 任意 | `TranspileContext` と `LineShift` | `transpile` の実体。デバッグ時のシーンマップ構築 |

`LuaCodeGenerator` は、出力した行数を `out_line` で数え、`Span` を持つ構文要素（スコープの見出し・アクション・変数代入・Call・選択肢・キューコマンド・グローバル・シーン・アクター辞書の単語定義）を出力した直後に「生成した Lua の行 → `.pasta` の位置」の対応を記録する。記録の規則は `record_span_since` の 1 つで、要素を出力する前の `out_line` から行が進んでいれば最後に書いた行を記録し、進んでいなければ（保留の空行しか無いなど）記録しない。Lua ブロックは内容の行ごとに `.pasta` の行と対応づける。`transpile_with_source_map` は、渡されたシンクの前に `LineRecorder` を挟んで生成器に付ける。`LineRecorder` は行の対応だけをトランスパイルの間の表に持ち（[init_scene の引数](#init_scene-の引数)）、シンクが渡されていればそのまま流す。シンクが無ければ、行の対応のほかは何も記録しない。

シンクの有無で出力のバイト列は変わらない。デバッグが有効なとき、ローダはトランスパイルの後に全 `.pasta` をもう一度パースし、シーンの宣言だけを控えるシンク（`MapBuilderSink`）を付けて `transpile_with_source_map` で生成し直す。生成した Lua と返された `LineShift` は捨て、控えた宣言をシーンマップに入れる。生成し直しには、増分処理と同じ 1 つのトランスパイラ（同じ別名表を持つ）を使うため、シーンマップの照合の鍵の宣言名はキャッシュの `.lua` と一致する。シーンマップの作り方は [シーンマップ](debug.md#シーンマップ) で扱う。

## 境界の受け渡し

| 境界 | 渡す側 → 受ける側 | 渡すもの | 所有 |
| ---- | ----------------- | -------- | ---- |
| `pasta_dsl` → `pasta_lua` | パーサ → トランスパイラ | `PastaFile`（AST） | トランスパイラは借用するだけで、AST を変更しない |
| トランスパイラ → ローダ | `LuaTranspiler` → 増分処理 | 生成した Lua のバイト列と `TranspileContext` | ローダが Lua をキャッシュへ書き出す。`TranspileContext` はファイルごとに捨てる |
| `pasta_core` ↔ `pasta_lua` | レジストリ型の利用 | `SceneRegistry`・`WordDefRegistry` | `TranspileContext` が所有する。名前の置換規則 `SceneRegistry::sanitize_name` は生成器も共有する |
| Rust → Lua（第 1 段 → 第 2 段） | キャッシュ → Lua VM | `pasta.scene.*` モジュールと `scene_dic.lua` | 生成コードの実行で Lua 側がシーン・単語の登録を所有する。`finalize_scene`（Rust）がそれを収集し、検索用のレジストリを作り直す |
| 生成器 → シーンマップ | `LuaCodeGenerator` → `SourceMapSink` の実装 | 行の対応とシーンの宣言の記録（シーンマップのシンクは宣言だけを控える） | シンクは呼び出し元が所有し、生成器は可変参照を借りる。`code_gen` は `debug` に依存しない |

## 不変条件と制約

- 生成される Lua コードは、ランタイムの Lua 方言である LuaJIT 2.1 で実行できる形でなければならない。生成コードは `table.unpack` を使うため、Lua 5.2 互換を有効にした LuaJIT（ルートの [Cargo.toml](https://github.com/ekicyou/pasta/blob/main/Cargo.toml) の `mlua` の `luajit52` 機能）を前提とする。
- ローカルシーンの関数名は Lua の識別子としてそのまま出力される（`SCENE.サブ_1`）。LuaJIT が 0x80 以上のバイトを識別子の文字として受け付けることを前提にしている。アクター名と `＠＊` の関数名は識別子にせず、文字列リテラルにして渡す（`act:actor_proxy("ぱすた")`・`act:global_fn("名前", …)`）。文法の識別子は `"` や `\` を含まないため、`"…"` の中にもそのまま埋め込む。
- 名前付きローカルシーンの関数名には常に `_番号` が付くため、利用者のローカルシーン名が `SCENE.__start__`・`SCENE.__global_name__` と衝突することはない。
- シンクを付けても付けなくても、書き出されるバイト列は同一である。出力の正規化は末尾の行の削除だけを行う。
- Lua ブロックの内容は、インデントを足さず、空行も含めて書いたとおりに出力する。生成器が区切りに足す空行は、`end` の直前には出力しない。
- 末尾呼び出しの `return act:call(…)` は、ローカルシーンの最後の項目が Call のときにだけ出力する。それ以外の Call は `act:call_restore(…)` になる。どちらも Call 1 つにつき 1 行である。
- 継続行には、同じローカルシーン内で先行するアクション行が必要である。親のグローバルシーンや別のローカルシーンからは引き継がない。
- ロングブラケットの `=` は最大 10 個である。
- トランスパイラは、参照先のシーン・単語の存在も、Lua ブロックの内容も検査しない。シーンと単語は実行時に検索で解決され、Lua ブロックの誤りはロード時の Lua エラーになる。`TranspileError` には未定義シーン・未定義単語を表す型も定義されているが、現行のコード生成はこれらを返さない。
- キャッシュの無効化は、クレートの版の変化・別名表の指紋（`.scene_alias`。指紋が違うときと、ファイルが無いときは全破棄）・ソースとキャッシュの更新時刻の違い（新旧を問わない）だけで判断する。同じ版のままコード生成を変えた場合、ソースの更新時刻が変わらない限り既存のキャッシュが使われる。
- `.pasta` のキャッシュへの書き出しに失敗したら、ロードを失敗させる。古いキャッシュが残っていても、それでは動かない。
- finalize の前の `@pasta_search` は空の登録表で動き、正しい引数のどの検索も見つからない。検索に使うレジストリは第 2 段の `finalize_scene` が作り直したものである。

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

> 【不安】流れはつかめまして？ 次は、登録されたシーンと単語がどう探し出されるのかへ進みますけれど……工程を一度に詰め込みすぎたかしら。

> 【アンソニー】煮込みが一度きりと伺いましたので、胃もたれもせずに済んでおります。
