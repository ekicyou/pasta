# Brief: scene-attribute-store

> **ステータス**: 未着手（`manual-ssot-authority` で `scene-attribute-semantics` として起票・2026-10-01。ロードマップ棚卸 2026-10-04 で規模が約 20 タスクの上限を超えると見積もり、本 spec（属性の保持・継承・読み出し）と `call-attribute-filter`（Call の属性フィルター）に分割した）。Wave 4（機能拡張）。着手するときは `/kiro-start scene-attribute-store` で開始する。

## Problem

Pasta DSL の属性（`＆名前：値`）は、構文だけが受理され、意味を持たない。旧文法仕様（`doc/spec/` ch08・ch04 §4.2・ch12 §12.5・§12.10・§12.18）は、属性によるシーンへのメタデータ付与、ファイルレベル属性の継承、Call の属性フィルター（`＞シーン＆k＝v`）を「将来予約」として定義していたが、どれも実装されていない。

`manual-ssot-authority` で旧文法仕様を廃止し、マニュアルを文法の唯一の権威とした。マニュアルには現行挙動（受理されるが処理に反映されない）だけを書き、未実装のセマンティクスとフィルターの予告はマニュアルから削除した。本 brief はその未実装部分のうち、**属性の保持・継承・読み出し**を引き継ぐ（吸収台帳の仕分け表 B1）。Call の属性フィルターは `call-attribute-filter` が引き継ぐ。

## Current State

実装照合は `manual-ssot-authority` の吸収台帳（`.kiro/specs/completed/manual-ssot-authority/absorption-ledger.md`「将来仕様の仕分け表」ch08・ch12 の B1 行、食い違い grep 記録 D05・D09）と、2026-10-04 の棚卸の再照合による。

- **構文（受理される）**: `＆名前：値`。
  - 1 行に複数並べられる。値は整数・小数・引用文字列（`「…」` / `"…"`）・引用なし文字列。
  - 置ける場所: グローバルシーンの初期部の属性行、ファイルレベルの属性行（インデントなし・シーンの外）、アクター辞書配下の属性行、グローバル／ローカルシーン宣言行への付記（`＊会話＆作者：Alice`・`・選択肢＆優先：3`）。
  - ローカルシーン宣言の次の行や、コンテンツ行の後の属性行はパースエラー（旧仕様 §8.2 の「ローカルシーンの直後の属性行」とは食い違う。D09）。
- **記録（実行時には残らない）**:
  - トランスパイル時の登録表には、グローバルシーン自身の属性が記録される（`pasta_lua` の `register_global_scene`）。
  - ファイルレベルの属性は後続のグローバルシーンの属性と統合される（`pasta_lua` transpiler の `merge_attrs`）が、統合結果を使う処理は無い（`generate_global_scene` の `_file_attrs` は未使用）。
  - **棚卸での訂正**: 実行時の登録表は `crates/pasta_lua/src/runtime/finalize.rs` 174 行付近で `register_global_raw(…, HashMap::new())` として作り直されるため、実行時には属性がどこにも残らない。生成 Lua にも属性は現れない（旧 brief の「シーン登録表に記録される」は、トランスパイル時に限った話だった）。
- **絞り込み（処理はあるが呼ばれない）**: `pasta_core` の `scene_table.rs` に `filter_by_attributes` がある（`HashMap<String,String>` の文字列一致・AND のみ）。候補キャッシュのキーにもフィルターが含まれる。`pasta_lua` の `search/context.rs` の `search_scene`（73 行付近）は常に空の `filters` で検索する。
- **マニュアル**: `book/src/grammar/block-structure.md#属性` に上記の現行挙動だけを書いている。

## 吸収元の内容（旧文法仕様・マニュアル旧版の記録）

旧文法仕様は `manual-ssot-authority` で削除される。要件化の材料として、属性・フィルターに関する記述をここに残す（要旨。原文は git 履歴の `doc/spec/08-attributes.md`・`04-call-spec.md`・`12-future.md`、`book/src/grammar/call-jump.md` の 2.2 以前の版）。

### ch08 属性

- **8.1 構文**: `属性行 ::= インデント ~ "＆" ~ key ~ "：" ~ value ~ NEWLINE`。
- **8.2 配置ルール**: 属性行はシーン定義の直後にだけ置ける。グローバルシーンの直後ならグローバルシーンに、ローカルシーンの直後ならローカルシーンに属性を付与する。同じシーンに複数の属性を連続して書ける。セマンティクスは「直前のシーンにメタデータを付与」。アクション行や変数代入行の後の属性行は文法エラー。例:

  ```text
  ＊グローバルシーン
    ＆author：Alice
    ＆genre：comedy
    ・ローカルシーン
      ＆priority：high
      ＆difficulty：3
      Alice：台詞内容
  ```

- **8.3 ファイルレベル属性（将来予約）**: `file_level_attribute ::= "＆" ~ key ~ "：" ~ value ~ NEWLINE`。ファイル冒頭、すべてのグローバルシーン宣言より前に、インデントなしで置く。ファイル内のすべてのグローバルシーンに自動的に継承される。例: `＆警報レベル：レッド` の後に `＊会話１`（`＆温度：暑い`）と `＊会話２`（`＆温度：寒い`）を置くと、会話１は `＆警報レベル：レッド　＆温度：暑い`、会話２は `＆警報レベル：レッド　＆温度：寒い` になる。「現在は構文のみ許容し、セマンティクス実装は順次対応予定」。

### ch12 の関連節

- **§12.5 フィルター機能の詳細（初期版対応なし）**: フィルター機能は初期版では対応しない。複数フィルターの OR/AND 結合方法などは将来検討。構文のみ定義され（§4.2）、セマンティクス実装は将来予定。
- **§12.10 属性値の型解釈**: 属性の値は ch05 §5.2 のリテラル型変換ルールに従う。フィルターとの比較演算の整合性はこの前提で設計する。
- **§12.18 ファイルレベル属性の継承詳細**: ファイルレベル属性はグローバルシーンに継承されるが、グローバルシーン側で同じ属性が定義されれば上書きする。ファイルレベル属性はローカルシーンには影響しない。

### ch04 §4.2 フィルター（属性フィルター）

```text
filter_list ::= ("＆" ~ key ~ 比較演算子 ~ value)+
現在: ＆key＝value
将来: ＆score＞50　＆level＜10　など比較演算子ベースに拡張予定

例: ＞シーン名＆author＝Alice＆genre＝comedy
   ＠単語名＆category＝food＆season＝summer
```

- セマンティクス: ターゲット選択時に属性で絞り込む（将来予約）。
- 適用範囲: Call だけでなく、会話文内の単語呼び出し（`＠`）でも同様に使える。基本的な記述ルールは共通。
- 設計原則: フィルターは比較・条件判定のため、コロンではなく比較演算子（`＝`・`＞`・`＜` など）を使う。
- 「現在: フィルター機能は将来用に宣言; 現在は無視」（実際はパースエラー。上の Current State）。

### マニュアル旧版 `grammar/call-jump.md`「フィルター（将来変更あり）」

Call ターゲットの後ろに `＆key＝value` 形式のフィルターを付けて、属性で候補を絞り込む構文が予約されている、とし、例 `＞シーン名＆author＝Alice＆genre＝comedy` を挙げていた。フィルターは比較・条件判定のためコロンではなく比較演算子（`＝` `＞` `＜` など）を使う。「フィルター機能は将来用に宣言されているだけで、現状では無視される。比較演算子ベース（`＆score＞50` など）への拡張も将来予定」と注記していた。

## Desired Outcome

- シーンに付けた属性が実行時に保持され、Lua から読める（例 `SCENE` のシーン表や act から属性を引く口）。
- ファイルレベル属性の継承と上書きの規則が実装され、「将来予約」の扱いが解消されている。出発点は次のとおり: ファイル → グローバルシーンへ継承する、同名はシーン側が優先、ローカルシーンには影響しない。
- 属性値の型解釈（§12.10）が決まっている。後続の `call-attribute-filter` の比較演算が、この型解釈を前提にできる。
- 属性の無いシーンの生成 Lua はバイト不変。
- マニュアル（`grammar/block-structure.md#属性`・`lua/script-api.md` ほか）が新しい挙動を書き、スキル `references/` は `gen-skill-refs.mjs` で再生成されている。

## Approach

要件フェーズで次を決める（未決定）。

- **属性の用途の範囲**: Lua から属性を読めるようにする口の形（シーン表のフィールドか、act のメソッドか）。
- **ローカルシーンの属性**: 宣言行への付記（現行で受理）に加えて、宣言の次の行の属性行（旧 §8.2）を受理するか。
- **値の型解釈**: 整数・小数・文字列の区別をどこまで保つか（§12.10）。空文字列の属性値（`dsl-literal-fixes` が `parse_attr` の `「」`・`""` を直す）の扱い。
- **実行時までの運び方**:
  - 生成 Lua に属性を出力する（属性があるシーンだけ。属性の無い出力はバイト不変）。
  - `finalize` で実行時のシーン表へ渡す。
- **アクター辞書配下の属性**: 今回意味を与えるか、受理して無視のままにするか。

## Scope

- **In**:
  - シーン属性のセマンティクス（メタデータの付与・実行時の保持・Lua からの読み出し）
  - ファイルレベル属性の継承と上書き
  - 属性値の型解釈
  - マニュアル章の更新とスキル `references/` の再生成
  - VSCode 拡張の TextMate 文法（ハイライト）の追従（属性行の配置を広げる場合）
- **Out**:
  - Call の属性フィルター構文と実行時の絞り込み（`call-attribute-filter`）
  - 単語参照への属性フィルター（単語には属性を付ける構文が無い。バックログ）
  - 動的単語参照 `＠＄`（`dynamic-word-reference`）
  - 属性と無関係な Call・単語検索の挙動変更

## Boundary Candidates

- 文法（`grammar.pest`）とパーサ（`pasta_dsl`）: 属性行の配置（ローカルシーン）
- トランスパイラ（`pasta_lua` code_gen）: 属性の Lua 出力、ファイルレベル属性の継承
- ランタイム（`pasta_lua` の `scene.lua`・`runtime/finalize.rs`、`pasta_core` の `scene_table`）: 実行時の保持と読み出し
- マニュアル・生成スキル・ハイライト文法

## Out of Boundary

- マニュアル権威化とスキル生成の仕組み（`manual-ssot-authority` が提供）
- `search_scene` へのフィルターの受け渡し（`call-attribute-filter`）

## Upstream / Downstream

- **Upstream**:
  - `manual-ssot-authority`（旧仕様の廃止と本 brief の起票。吸収台帳が現行挙動の照合記録を持つ）。
  - `dsl-literal-fixes`（Wave 1。属性値の空文字列）。
  - `scene-identity-format`（Wave 2。`scene.lua` を先に触る）。
  - `call-execution-correctness`（Wave 3。コード生成を先に触る）。
- **Downstream**: `call-attribute-filter`（保持された属性と型解釈を前提にする）

## Existing Spec Touchpoints

- **Adjacent**: `manual-ssot-authority`（吸収台帳の仕分け表 B1、食い違い grep 記録 D05・D09）、`pasta-manual-syntax-highlight`（TextMate 文法の再利用）

## Constraints

- マニュアルが文法の唯一の権威。挙動を変えたら同じ変更でマニュアルを更新し、`node book/tools/gen-skill-refs.mjs` で生成スキルを再生成する（`--check` が CI で鮮度を見る）。
- 現在、属性はどこでも受理され無視される。保持と読み出しを足しても候補の選択は変わらないため、既存の辞書に対して非互換は生まれない（属性行の配置を広げる場合を除く）。
- 現行実装（`grammar.pest`・トランスパイラ・ランタイム）を正として設計する。旧仕様の記述は材料であり規範ではない（旧 §8.2 のように現行と食い違う記述がある）。

## 2026-10-07 棚卸の再測定（main 2cbaf510）

- **前提の変化**: 上流の 3 spec（`dsl-literal-fixes`・`scene-identity-format`・`call-execution-correctness`）はすべて完了した。未完了の前提は無く、今すぐ着手できる。シーンの登録名は「名前_番号」（`SceneRegistry::registered_name`、Lua 側は `scene.lua` の `create_scene`）。空の引用の属性値は `AttrValue::String("")` になった。
- **触るファイル**: `pasta_dsl` の `parser/parse_scene.rs`（426 行）・`parser/ast/scene.rs`、`pasta_lua` の `code_gen/scope_gen.rs`（407 行）・`context.rs`・`transpiler.rs`、`pasta_scripts/pasta/scene.lua`（216 行）、`runtime/finalize.rs`（355 行）、`pasta_core` の `registry/scene_registry.rs`（576 行）・`scene_types.rs`、マニュアル `grammar/block-structure.md`・`lua/script-api.md`・`internals/registry-search.md`・`internals/transpiler.md` と生成スキル。1,000 行に近いファイルは無い。
- **規模**: 約 14〜18 タスク（宣言行の属性をパーサで拾う 1〜2、属性があるシーンだけ Lua に出す 2〜3、`scene.lua` の保持と読み出しの口 2、`finalize` で登録表へ渡す 2〜3、型の扱い 1〜2、マニュアルと生成 2、テスト 2）。上限の 20 に収まる。
- **先に要るもの**: なし。ほかの未完了 spec とソースは重ならない。Phase 12 の `manual-claudia-theme` とは同じマニュアルのページ（各章の導入と締め）と生成スキルを触るが、別の節なので並走できる（後から入る側が再生成で合わせる）。
- **種別**: 機能（実行時に属性が残らないものを、保持して読めるようにする）。
- **要件定義のモデル**: Fable（読み出しの口の形・型の解釈・ローカルシーンの属性の置き場所は、後続の `call-attribute-filter` が土台にする意味の決定で、開発者の判断の分かれ道が多い）。
- **分割の案**: 不要。読み出しの口を `act` ではなく `SCENE`（`scene.lua`）側に置けば `act.lua` を触らずに済み、`failure-output-unification` と同じウェーブで並走できる。
- **見つけた穴・古くなった記述**:
  - 宣言行への付記（`＊会話＆作者：Alice`・`・選択肢＆優先：3`）は文法では受理されるが、パーサが名前だけを拾って捨てる（`parse_scene.rs` の `parse_global_scene_start` 84 行・`parse_local_scene_scope` 202〜205 行）。`LocalSceneScope.attrs` は常に空。Current State の「置ける場所」は文法の話で、AST には残らない。
  - 登録表を作り直す場所は `finalize.rs` 182 行（174 行付近から移動）。
  - `register_global_raw` は渡した属性をローカルシーンにも同じ値で複製する（`scene_registry.rs` 150〜189 行）。「ローカルシーンには継承しない」「ローカルは自分の属性を持つ」とするなら、この API を変える必要がある。
  - `collect_scenes`（`finalize.rs` 61〜69 行）は、シーン表の `__global_name__` 以外のキーをすべてローカルシーンとして数える。属性をシーン表のフィールドに置くと、偽のローカルシーンが登録される。
  - 登録表の属性は `HashMap<String, String>`（`AttrValue` の表示文字列）で、型が失われる。トランスパイル時の登録（`context.rs` の `register_global_scene`）はファイル属性を統合しない。統合は `context.rs` の `merge_attrs`（brief の「transpiler の」は不正確）。
  - ファイル属性は記述順に累積する（シーンの間に置いた `＆` は後のシーンにだけ効き、同じキーは後の値で上書き。`internals/transpiler.md` 155 行）。旧 §8.3 の「ファイル冒頭だけ」とは違う。どちらを正とするかを要件で決める。
  - マニュアル `grammar/block-structure.md` 235 行の「内部に記録される」は、宣言行の付記では成り立たない（上の 1 点目）。

## 申し送り（scene-name-alias より）

- `transpiler.rs` の `process_global_scene` はメソッドになり、冒頭で**宣言名**（`scene_aliases.resolve(&scene.name)` で別名を置き換えた名前）を 1 回だけ決める。登録・通し番号・単語のモジュール名・生成コード・突合キー・ローカルの親名はすべてこの宣言名から作る。属性を登録・生成に足すときも、シーン名は宣言名を使う。
- `code_gen/scope_gen.rs` の `generate_global_scene(scene, declared_name, scene_counter, context, file_attrs)` に `declared_name` 引数が増えた。`context.rs` に `register_global_scene_named(name, attrs)` が増え、`register_global_scene(scene)` はそれに委譲する。`finalize.rs` の `finalize_scene_impl(lua, &aliases)` も署名が変わった。
- 参照: `.kiro/specs/completed/scene-name-alias/design.md`「DeclaredNameResolver」「Revalidation Triggers」。

## 2026-10-10 棚卸の再測定（main add05022）

- **前提の変化**: シーン名の別名（`scene-name-alias`）と、式の中の値なしの扱い（`expr-nil-coercion`）が入った。この spec で変わるのは、シーンを登録するときに使う名前だけである。上の「申し送り（scene-name-alias より）」は現在のコードと一致する（シーン 1 つを処理する `process_global_scene` は `crates/pasta_lua/src/transpiler.rs` 202 行、名前を受け取って登録する `register_global_scene_named` は `crates/pasta_lua/src/context.rs` 48 行、実行時の登録表を作る `finalize_scene_impl` は `crates/pasta_lua/src/runtime/finalize.rs` 226 行）。未完了の前提は無い。
- **触るファイル**: 前回と同じ。現在の行数は、宣言行を読む `crates/pasta_dsl/src/parser/parse_scene.rs` 426、シーンの Lua を出す `crates/pasta_lua/src/code_gen/scope_gen.rs` 410、`context.rs` 454、`transpiler.rs` 266、Lua 側のシーン表 `crates/pasta_lua/pasta_scripts/pasta/scene.lua` 216、`finalize.rs` 358、登録表 `crates/pasta_core/src/registry/scene_registry.rs` 576。1,000 行に近いものは無い。宣言行の属性を拾うなら、編集中の解析で位置をずらす処理（`crates/pasta_dsl/src/partial.rs` 477 行）と、エディタ向けの色付け（`crates/pasta_lsp/src/analysis/visit_scope.rs` 194 行）も確かめる。
- **規模**: 約 15〜18 タスク。前回の見積もりに、下の「テスト用の見本の作り直し」と「エディタ向けの確認」で 1 タスクを足した。上限の 20 に収まる。
- **先に要るもの**: なし。失敗の出力の一本化（`failure-output-unification`）と同じ時期に進められる。条件は、属性を読む口を `scene.lua` だけに置き、`crates/pasta_lua/pasta_scripts/pasta/act.lua` と、台詞と Call の Lua を出す `crates/pasta_lua/src/code_gen/element_gen.rs` を触らないこと。実行中のシーンの表は今でも `act.current_scene` で取れるので、それを `scene.lua` の関数に渡せば読める。`act.lua` を変える必要は無いと確かめた。
- **種別**: 機能（書いた属性が、実行時にはどこにも残らない）。
- **要件定義のモデル**: Fable（読み出しの口の形・値の型・ローカルシーンの属性の置き場所を決める。後の `call-attribute-filter` がその上に乗る）。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**:
  - 「Current State」の「`search_scene`（73 行付近）」は、別名の処理が入って `crates/pasta_lua/src/search/context.rs` の 91 行に動いた。空の絞り込み条件を作るのは 96 行。
  - テスト用の見本 `crates/pasta_lua/tests/fixtures/sample.pasta` の 14 行に、ファイルの属性 `＆天気：晴れ` がある。属性を Lua に出すと、この見本から作る `sample.generated.lua` と、比べる相手の `sample.expected.lua` が変わる。「属性の無いシーンの出力は変えない」という約束では守られないので、作り直しをタスクに入れる。
  - 宣言行の属性を拾うと、グローバルシーンでは「宣言行」と「次の行からの属性行」の 2 か所に同じ名前を書ける。どちらを優先するかを要件で決める。
  - 別名があると、`＊会話` と `＊OnTalk` の 2 つの宣言が同じ名前のシーンになる。属性は宣言ごと（通し番号ごと）に持つことを、要件に書く。
  - `pasta_shiori` の結合テストは、古いランタイムの写し（`crates/pasta_shiori/tests/support/scripts/pasta/scene.lua`）を読む。この spec の確認用のテストは `pasta_lua` 側に置く。写しは直さない（`shiori-test-support-runtime` の持ち場）。
  - 前回の節の行番号のうち、登録表を作り直す場所（`finalize.rs` 182 行）と、属性をローカルシーンへ写す処理（`scene_registry.rs` 155〜189 行）は今も合っている。マニュアルの「内部に記録される」は `book/src/grammar/block-structure.md` の 248 行と 280 行に、ファイルの属性の説明は `book/src/internals/transpiler.md` の 158 行と 165 行に動いた。

## 申し送り（2026-10-10 の discovery・0.5.0 のノベルゲームより）

0.5.0 の目標（areka でノベルゲームが作れる）の道筋に、この spec が入った。0.4.0 の後、先頭に置く。

- **読み手が 1 人増える**: 属性の倉は 1 つで、読み手が 2 人になる。Call の絞り込み（`call-attribute-filter`）は場面を選ぶときに読み、演出（`scene-stage-attributes`）は場面に入るときに読む。演出は、語彙の表にある名前の属性だけを、台本へ流す。モードの切り替えは作らない（開発者の決定）。
- **順番を保つ**: 演出は、属性を書いた順に流したい。今は、統合した属性が順番を持たない表（`HashMap`）に入る（`crates/pasta_lua/src/context.rs` 19・106〜114 行）。倉の形を決めるときに、順番を保てる形にする。
- **ローカルシーンと名前なしの `＊`**: 演出の側は、ファイルの属性と親の属性を、ローカルシーンへも受け継がせたい（旧仕様は「ローカルシーンには影響しない」だった）。名前なしの `＊` は、今は属性を受け継がない（`crates/pasta_dsl/src/parser/parse_scene.rs` 88〜91 行）。受け継ぎの決まりを決めるときに、この使い方を材料にする。受け継ぐかどうかの最終の決定は、この spec の要件で行う。
- **値は 1 つ**: `＆背景：教室、夕方` は、`教室、夕方` という 1 つの文字列になる（`grammar.pest` 164〜166 行）。値を複数持たせる必要が出たら、演出の側の spec が言い出す。この spec では変えない。
- 参照: `.kiro/specs/scene-stage-attributes/brief.md`、`.kiro/steering/roadmap.md` の「Phase 13」。
