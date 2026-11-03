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

## 申し送り（scene-name-alias より）

- `transpiler.rs` の `process_global_scene` はメソッドになり、冒頭で**宣言名**（`scene_aliases.resolve(&scene.name)` で別名を置き換えた名前）を 1 回だけ決める。登録・通し番号・単語のモジュール名・生成コード・突合キー・ローカルの親名はすべてこの宣言名から作る。属性を登録・生成に足すときも、シーン名は宣言名を使う。
- `code_gen/scope_gen.rs` の `generate_global_scene(scene, declared_name, scene_counter, context, file_attrs)` に `declared_name` 引数が増えた。`context.rs` に `register_global_scene_named(name, attrs)` が増え、`register_global_scene(scene)` はそれに委譲する。`finalize.rs` の `finalize_scene_impl(lua, &aliases)` も署名が変わった。
- 参照: `.kiro/specs/completed/scene-name-alias/design.md`「DeclaredNameResolver」「Revalidation Triggers」。

## 申し送り（2026-10-10 の discovery・0.5.0 のノベルゲームより）

0.5.0 の目標（areka でノベルゲームが作れる）の道筋に、この spec が入った。0.4.0 の後、先頭に置く。

- **読み手が 1 人増える**: 属性の倉は 1 つで、読み手が 2 人になる。Call の絞り込み（`call-attribute-filter`）は場面を選ぶときに読み、演出（`cue-command-runtime` の柱の分。2026-10-17 に `scene-stage-attributes` をまとめた）は場面に入るときに読む。演出は、語彙の表にある名前の属性だけを、台本へ流す。モードの切り替えは作らない（開発者の決定）。
- **順番を保つ**: 演出は、属性を書いた順に流したい。今は、統合した属性が順番を持たない表（`HashMap`）に入る（`crates/pasta_lua/src/context.rs` 19・106〜114 行）。倉の形を決めるときに、順番を保てる形にする。
- **ローカルシーンと名前なしの `＊`**: 演出の側は、ファイルの属性と親の属性を、ローカルシーンへも受け継がせたい（旧仕様は「ローカルシーンには影響しない」だった）。名前なしの `＊` は、今は属性を受け継がない（`crates/pasta_dsl/src/parser/parse_scene.rs` 88〜91 行）。受け継ぎの決まりを決めるときに、この使い方を材料にする。受け継ぐかどうかの最終の決定は、この spec の要件で行う。
- **値は 1 つ**: `＆背景：教室、夕方` は、`教室、夕方` という 1 つの文字列になる（`grammar.pest` 164〜166 行）。値を複数持たせる必要が出たら、演出の側の spec が言い出す。この spec では変えない。
- 参照: `.kiro/specs/cue-command-runtime/brief.md`（2026-10-17 に `scene-stage-attributes` をまとめた）、`.kiro/steering/roadmap.md` の「Phase 13」。

## 申し送り（2026-10-21 の discovery「PastaDSL機能ヒアリング」より）

- 属性の新しい読み手として `scene-pick-modes`（同名シーンの重み `＆重み：5` と定義順の巡回。2026-10-21 起票）が加わる。数の値（重み）を読む型の解釈と、ローカルシーン・名前なしの `＊` への受け継ぎを、要件の材料にする。
- 2026-10-21 の開発者の決定: `scene-pick-modes` は `＆重み` だけを先に特別扱いで読まない。この spec の後に入る。この spec の要件で、次の 2 つを決めておく。`scene-pick-modes` はその規則に従う。
  - ファイルの先頭の `＆重み` を、シーンが受け継ぐか
  - 全角の数字（`５`）を数として読むか
- ghost_dev の作者は、場面の側に条件を書く形（`＊OnMouseClick　＆r4：セキュリティボール`）も想像した。条件の向きは `call-attribute-filter` の申し送りに書いた。

## 申し送り（2026-10-29 の discovery「シーン属性による候補の絞り込み」より）

- `call-attribute-filter` は閉じて、新しい spec `scene-condition-attributes` に統合した（開発者の決定）。この brief の `call-attribute-filter` は `scene-condition-attributes` と読み替える。
- 中身が変わった。条件は呼ぶ側（`＞シーン＆k＝v`）でなく、場面の側に書く（`＊雑談_今の部屋　＆＄＊経験値≧500`）。
  - 呼ぶ側の Call の文法（`call_scene`・`parse_action.rs`・`element_gen.rs` の Call）は触らない見込みである。
  - 触るのは、属性の文法、`scene.lua`・`act.lua` の検索の道、`search/context.rs`・`scene_table.rs`、イベントからシーンを起こす道（選ぶ前の自動転記）、`pasta_check validate`、LSP と VSCode の文法定義である。
- 参照: `.kiro/specs/scene-condition-attributes/brief.md`。
- この spec への影響: 属性の読み手が 1 人増える。`＄` で始まる属性（`＆＄＊経験値≧500`・`＆＄＊初回待ち`）は条件で、`scene-condition-attributes` が候補を選ぶときに読む。宣言の属性（`＄` で始まらないもの）の意味は変わらない。宣言の属性 `＆優先：数` も増える。要件の材料は次の 2 つである。
  - 倉に条件も入れて読み手が分けるか、宣言と条件を分けて運ぶか。
  - 全角の数字の読み方と、ファイルの先頭の属性の受け継ぎの規則。`scene-condition-attributes` も条件と `＆優先` にこの規則を使う。

## 申し送り（2026-10-29 の版の組み替えより）

- 0.5.0 が「属性関係の整備」になり、この spec はそこに入った（開発者の決定。ノベルゲームは 0.6.0 へ順延）。順は `scene-attribute-store` → `scene-condition-attributes` → `scene-pick-modes` で、0.4.5（ponytail）を出した後から始める（「ポニーテールが片付いたら属性に手を出しましょう」）。
- 読み手は `scene-condition-attributes`・`scene-pick-modes`（どちらも 0.5.0）と、0.6.0 の `cue-command-runtime`（柱の属性）。0.6.0 の読み手も要件の材料に含め、倉の形が後で作り直しにならないようにする。
- 0.4.5 の ponytail のうち 5 本（`ponytail-dsl-parser`・`ponytail-lua-transpiler`・`ponytail-core-registry`・`ponytail-lua-loader`・`ponytail-lua-scripts`）とファイルを共有する。それらが入った後の main で測り直す。

## 2026-11-03 棚卸の再測定（main c575febd）

- **前提の変化**: 先に要った 0.4.5 の 4 本（`ponytail-dsl-parser`・`ponytail-lua-transpiler`・`ponytail-core-registry`・`ponytail-lua-scripts`）は全部入った。どれも属性の意味は決めていない。宣言行の属性は今も捨てられる（`parse_scene.rs` の `parse_global_scene_start` 73 行・`parse_local_scene_scope` 210 行が `id` だけを読む。`ponytail-editor-tooling` も同じことを確かめた）。ファイル属性の累積は残った（`crates/pasta_lua/src/context.rs` 17 行の `file_attrs`・87 行の `merge_attrs`。生成器の `_file_attrs` は `scope_gen.rs` 95 行で今も使われない）。`merge_from` は消えた。シーン表の索引は `BTreeMap` になったが、`filter_by_attributes`（`scene_table.rs` 283 行）と `SceneInfo.attributes`（`HashMap<String, String>`）はそのまま。
- **値の型（新しく分かったこと）**: パーサは属性の値を、整数・小数・引用文字列・引用なし文字列に分けて構文木に持つ（`AttrValue`）。全角の数字（`５`）は文法の `digit`（`grammar.pest` 143 行）で数になり、`parse_attr` が整数 5 にする（`parse_elements.rs` 18〜23 行）。型が失われるのは、登録表へ入れるときに文字列にする所（`context.rs` 29〜34 行）である。なので「全角の数字を数と読むか」は、パーサの今の読み方を実行時まで保つか、の問いに縮んだ。
- **触るファイル**: `parse_scene.rs` 430・`parser/ast/scene.rs` 235、配置を広げるなら `grammar.pest` 272、`scope_gen.rs` 404、`context.rs` 360、`transpiler.rs` 290（`fill_line_maps` 259 行）、`scene.lua` 261、`runtime/finalize.rs` 358（登録表を作り直すのは 182 行）、`scene_registry.rs` 464（`register_global_raw` 147 行）、`scene_types.rs` 36、型を変えるなら `scene_table.rs` 442、見本 `sample.expected.lua`・`sample.generated.lua` と属性の入ったスナップショット 2 つ、マニュアル（`grammar/block-structure.md`・`lua/script-api.md`・`internals/registry-search.md`・`internals/transpiler.md`）と生成スキル、配置を広げるならハイライト（`editors/vscode/syntaxes/pasta.tmLanguage.json`）。1,000 行に近いものは無い。
- **規模**: 15〜18 タスクのまま。
- **先に要るもの**: 機能の上では無い。ファイルの重なり（未完了）: `scene-condition-attributes`・`scene-pick-modes`・0.6.0 の `cue-command-runtime`（どれもこの spec の後）。`scene-count` とは、読み出しの口を `search/context.rs` に置かなければ重ならない（`scene.lua` に置けば重ならない）。`vscode-mcp-client` とは重ならない（あちらはハイライトの文法を触らない）。`debug-mcp-runtime` とは、`finalize_scene_impl` の署名を変えず（`runtime/factory.rs` 120 行が登録する）、属性をシーン表のフィールドに置かなければ（下の穴）重ならない。
- **種別**: 機能（書いた属性が、実行時にはどこにも残らない）。
- **要件定義のモデル**: Fable のまま。決めるのは Lua から読む口（公開の契約）と受け継ぎの意味（DSL の意味）で、0.5.0 の 2 本と 0.6.0 の `cue-command-runtime` がその上に乗る。読む口の形、ローカルシーン・名前なしの `＊`・ファイルの先頭からの受け継ぎ、重なったときの優先と書いた順（ファイル・宣言行・次の行の属性行）、条件の属性（`＄` で始まるもの）を同じ倉に入れるか、の 4 つが互いに絡み、どれもまだ推奨案が無い。値の型は上のとおり縮んだが、残りで「3 つ以上の絡む問い」が成り立つ。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**（前の節からの持ち越しを含む。今も成り立つ）:
  - 名前なしの `＊`（`grammar.pest` 239 行の `global_scene_continue_line`）は宣言行に属性を書けず、名前だけを前から受け継ぐ（`parse_scene.rs` 88〜90 行）。マニュアルの「内部に記録される」（`block-structure.md` 271・303 行）は、宣言行の属性では成り立たない。
  - 属性をシーン表のフィールドに置くと、偽のローカルシーンが出る所が 2 つある。`finalize.rs` の `collect_scenes`（65 行）と、デバッグのシーン索引 `crates/pasta_lua/src/debug/source_map/scene_join.rs` 87 行。どちらも `__global_name__` だけを外す。倉をシーン表の外に置くか、両方を直す（後者は `debug-mcp-runtime` が残すファイル）。`register_global_raw` は属性をローカルシーンへ同じ値で写す。
  - ファイル属性は書いた順に累積し、後のシーンにだけ効く（`internals/transpiler.md` 164・171 行）。旧 §8.3 の「ファイル冒頭だけ」とどちらを正とするかを要件で決める。別名で同じ名前になる宣言（`＊会話` と `＊OnTalk`）は、属性を宣言ごとに持つ。
  - 見本 `crates/pasta_lua/tests/fixtures/sample.pasta` 14 行の `＆天気：晴れ` のため、属性を Lua に出すと見本の期待値の作り直しが要る。`act:init_scene(…, 行の対応)` の最後の引数を崩さない置き方にする。読み出しの口を `scene.lua` に置けば `act.lua` を触らずに済む（実行中のシーンの表は `act.current_scene`）。確かめのテストを `pasta_lua` と `pasta_shiori` のどちらに置くかは要件・設計で決める。
