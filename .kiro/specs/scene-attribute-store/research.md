# Gap Analysis: scene-attribute-store

測定: ブランチ `claude/kiro-start-scene-attribute-store-af747b`（main `96b75cbf` 相当。brief の再測定 `c575febd` から roadmap のコミット 1 つ分だけ進んだ。属性に関わるコードの差は無い）。行番号は 2026-11-03 の grep による。

## 1. 現状の調査

### 属性が通る道と、どこで消えるか

| 段 | 場所 | 今の挙動 | 要件とのずれ |
| -- | ---- | -------- | ------------ |
| 文法 | `crates/pasta_dsl/src/parser/grammar.pest` 162〜166（`attr`・`key_attr`・`attr_value`）、173（`scene = _{ id ~ s ~ attrs? }`）、233（`file_attr_line`）、238〜239（`global_scene_line`・`global_scene_continue_line`）、241（`global_scene_attr_line`）、244（`local_scene_line`）、257（アクター配下）、261（初期部）、267（`local_scene_item` に属性行は無い） | 置ける場所はマニュアルどおり。名前なしの `＊` には `attrs?` が無い。全角の数字は `digit`（143）で数になる | 変更不要（仮置きで配置を広げない）。未決事項 2 の付随・7 を採るなら 239・267 を触る |
| 構文木 | `crates/pasta_dsl/src/parser/ast/action.rs` 123（`Attr{key, value, span}`）・134（`AttrValue::{Integer, Float, String, AttrString}`）。`ast/scene.rs` 48・111（`GlobalSceneScope.attrs`・`LocalSceneScope.attrs` は `Vec<Attr>`、順を持つ） | 型と順は構文木に残っている | 変更不要。`AttrValue` の 4 分類を実行時までの型（Requirement 3）にそのまま写せる |
| パーサ | `crates/pasta_dsl/src/parser/parse_scene.rs` 29〜33（初期部の属性行 → `attrs`）、73〜80（`parse_global_scene_start` は `global_scene_line` の `id` だけ返す）、88〜91（名前なし `＊` は名前だけ受け継ぐ）、210〜222（`parse_local_scene_scope` は `local_scene_line` の `id` だけ読む）。`parse_elements.rs` 6〜39（`parse_attr`） | **宣言行の属性を捨てる**（グローバル・ローカルとも） | **Missing**（Requirement 1.1・1.2）。`global_scene_line` と `local_scene_line` の子に `attr` が並んでいるので、`id` の後の `attr` を `parse_attr` に通して `attrs` の先頭に積むだけ。順は宣言行 → 初期部（Requirement 1.3） |
| 登録（トランスパイル時） | `crates/pasta_lua/src/context.rs` 17（`file_attrs: HashMap<String, AttrValue>`）、29〜34（`register_global_scene_named` が `HashMap<String, String>` に文字列化）、50〜60（ローカルも同じ）、72（`accumulate_file_attr`）、87〜93（`merge_attrs` → `HashMap<String, AttrValue>`） | 型は文字列化で消え、順は `HashMap` で消える。ファイル属性は書いた順に累積し後のシーンに効く（Requirement 2.2 の現行） | **Constraint**: 順と型を保つ入れ物に替える（`Vec<Attr>` か `IndexMap` 相当。`indexmap` は依存に無い。`Vec` と線形探索で足りる規模）。`merge_attrs` の「ファイル → シーン」の重ね方は Requirement 2.4・2.5 のとおりに書き直す |
| 生成 | `crates/pasta_lua/src/code_gen/scope_gen.rs` 95（`_file_attrs` 未使用）、262（`act:init_scene(SCENE, {}, {})`）。`transpiler.rs` 116（累積）・181（登録）・194〜195（統合して生成器へ） | 生成 Lua に属性は現れない | **Missing**（Requirement 4.1・6.1・6.2）。属性のあるシーンだけ、`init_scene` の引数を崩さない形で属性を Lua に出す。`sample.pasta` 14 行の `＆天気：晴れ` のため `tests/fixtures/sample.lua`（期待値）と `transpiler__final_regression_test__r7_1_off_path__fixture_sample.snap` の作り直しが要る |
| ランタイム（Lua） | `crates/pasta_lua/pasta_scripts/pasta/scene.lua` 67〜77（`create_global_table` が `{ __global_name__ = … }`）、104（`get_all_scenes`）、138（`SCENE.search(name, global_scene_name, attrs)` の `attrs` は未使用）。`act.lua` 155・205〜209（`current_scene`・`init_scene`） | 属性の置き場が無い | **Missing**（Requirement 4・5）。読み出しの口 `SCENE.attrs` を `scene.lua` に置けば `act.lua` を触らない（brief の申し送り）。倉を `STORE.scenes[global][local]` の表に混ぜると下の「偽のローカルシーン」に当たる |
| 辞書確定（Rust） | `crates/pasta_lua/src/runtime/finalize.rs` 44〜78（`collect_scenes` は `__global_name__` だけ除く）、160〜185（`build_scene_registry` が `register_global_raw(…, HashMap::new())`）、226（`finalize_scene_impl(lua, &aliases)` の署名は変えない。`runtime/factory.rs` 120 が登録） | **属性は空で登録される** | **Missing**（Requirement 4.1・4.3）。Lua に出した属性を集めて `register_global_raw` へ渡す |
| シーン表（Rust） | `crates/pasta_core/src/registry/scene_registry.rs` 18〜20（`attributes: HashMap<String, String>`）、147〜180（`register_global_raw` は親の属性をローカルへ写す。テスト 420）。`scene_types.rs` 31（`SceneInfo.attributes: HashMap<String, String>`）。`scene_table.rs` 98・283〜295（`filter_by_attributes` は文字列一致 AND）。`search/context.rs` 130・202（`filters` は常に空） | 型は文字列・順は無し・ローカルへ写す | **Constraint**: Requirement 3（型）・4.2（順）・2.6（ローカルへ写さない）に合わせて `attributes` の型を替える。`filter_by_attributes` の引数型（`HashMap<String, String>`）は `scene-condition-attributes` が作り直す見込みなので、この spec では「使わない所を壊さない」最小の合わせ方でよい |
| デバッグの索引 | `crates/pasta_lua/src/debug/source_map/scene_join.rs` 87（`__global_name__` だけ除く） | 属性をシーン表のフィールドに置くと偽のローカルが出る | **Constraint**（Requirement 4.4）。倉をシーン表の外に置けば触らない（`debug-mcp-runtime` が残すファイル） |
| 検査 | `crates/pasta_check/src/validate/scenes.rs` 48・56・156（`attr_map` が文字列化して `register_global`/`register_local`） | pasta.dll と同じ文字列化 | **Constraint**（Requirement 6.4）。レジストリの属性の型を替えれば追従が要る。一致テスト（`validate_parity_test.rs`）で確かめる |
| 着色 | `crates/pasta_lsp/src/analysis/visit_scope.rs` 35・59・76・244（宣言行の属性も着色済み）。`editors/vscode/syntaxes/pasta.tmLanguage.json` 55〜70（`decl-attr`） | 宣言行の属性は既に着色される | 変更不要（Requirement 6.5）。配置を広げるときだけ |
| マニュアル | `book/src/grammar/block-structure.md` 269〜303、`internals/transpiler.md` 164〜171、`internals/registry-search.md` 100・183・284、`lua/script-api.md`（`SCENE` の節は無い。26 行に `act.current_scene` は触るなと書いてある） | 「処理には反映されない」「属性は空で登録」 | **Missing**（Requirement 7）。`gen-skill-refs.mjs` で再生成 |

### 既存のテストで触れるもの

- `crates/pasta_lua/tests/runtime/syntax_test.rs` 76〜（`test_attribute_inheritance`。ファイル属性とシーン属性で検索が通ることだけ見る。属性の中身は見ていない）
- `crates/pasta_dsl/tests/literal_fixes_test.rs` 216〜321（`all_attrs` でグローバル・アクターの属性値を見る。空の引用文字列が空文字列になることを確かめ済み。`dsl-literal-fixes` の成果）
- `crates/pasta_core/src/registry/scene_registry.rs` 420（`register_global_raw` が親の属性をローカルへ写すテスト。Requirement 2.6 を採るなら期待を反転する）
- `crates/pasta_lua/tests/transpiler/final_regression_test.rs` の `fixture_sample` スナップショットと `tests/fixtures/sample.lua`（見本の期待値）
- Lua の BDD テスト一覧 `crates/pasta_lua/tests/lua_specs/init.lua`（`act_init_scene_global_record_test.lua` の隣に読み出しのテストを足せる）

### 慣習

- 属性の AST は `Vec<Attr>`（順あり・型あり）。壊すのはトランスパイルの登録段だけなので、「順と型を保つ」は新しい作りではなく、既にある形を最後まで運ぶ話である。
- 生成 Lua の形は `local SCENE = PASTA.create_scene("名前")` → `function SCENE.__start__(act, ...)` → `act:init_scene(SCENE, "名前", nil, "行の対応")`。属性のある場合だけ 1 行足す形が、バイト不変（Requirement 6.1）と両立する。
- 辞書確定は Lua の `get_all_scenes()` を Rust から走査して登録表を作り直す。属性も同じ道で Lua → Rust へ運ぶのが既存の流れに沿う。
- `pasta_check validate` は pasta.dll と同じ `SceneRegistry` を使い、一致テストで差を検出する。

## 2. 要件の実現性

| 要件 | 技術的に要るもの | 状態 |
| ---- | ---------------- | ---- |
| 1 宣言行の取り込み | `parse_global_scene_start`・`parse_local_scene_scope` で `attr` を読む。名前なし `＊` は現行のまま | Missing（小） |
| 2 受け継ぎと重なり | `context.rs` の累積と統合を「順あり・後勝ち・位置は先」で書き直す。ローカルへ写すのをやめる | Constraint（`HashMap` → 順ありの入れ物） |
| 3 型 | `AttrValue` を文字列化せず運ぶ。Lua へ出すときに integer/float → number、string → string | Constraint（`register_global_scene_named`・`pasta_check` の `attr_map`・`SceneInfo.attributes` の型） |
| 4 実行時の保持 | Lua に属性を出す → 辞書確定で集める → `register_global_raw` へ渡す。偽のローカルを出さない置き場 | Missing（中。この spec の中心） |
| 5 Lua の口 | `scene.lua` に `SCENE.attrs(global_name, local_name)` | Missing（小） |
| 6 互換 | 属性の無いシーンは生成を変えない。見本とスナップショットの更新。`validate` の一致 | Constraint |
| 7 マニュアル | 4 ページと `references/` の再生成 | Missing（文書） |

### 未知（Research Needed）

- **Lua → Rust の運び方**: 属性を Lua のどこに置くと `collect_scenes`（`finalize.rs` 56〜70）と `scene_join.rs` 87 の `__global_name__` 除外に当たらないか。候補: (a) `STORE` に属性だけの別表（`STORE.attrs[global][local]`）、(b) シーン表のフィールド `__attrs__` にして 2 か所の除外を広げる。(a) なら `debug-mcp-runtime` のファイルを触らない。
- **型の運び**: `AttrValue::Integer(i64)` は LuaJIT の number（double）で 2^53 を超えると精度が落ちる。属性の重みや優先の範囲では問題にならない（設計で `shortcut:` として記す）。
- **`SceneInfo.attributes` の型を替える影響**: `filter_by_attributes`（`scene_table.rs` 283）とそのテスト（`scene_table_resolve_filter_tests.rs`）が `HashMap<String, String>` 前提。`scene-condition-attributes` が条件の形に作り直す見込みなので、この spec では壊れない最小の合わせ方（文字列化した写しを残す／フィルタ側を新しい型で比べる）のどちらかを設計で選ぶ。
- **`register_global_raw` の親 → ローカルの写し**: Requirement 2.6（受け継がない）を採ると、`scene_registry.rs` 147〜180 の写しをやめ、ローカルの属性を別に受け取る口が要る。今は `&[String]`（ローカル名の列）しか受けない。

## 3. 実装の選択肢

### Option A: 既存の道を延ばす（Lua 経由で運ぶ）

- パーサ: 宣言行の `attr` を `attrs` に積む（2 関数）。
- トランスパイル: `context.rs` の入れ物を順ありに替え、`merge_attrs` を後勝ち・位置は先に。生成は属性のあるシーンだけ `SCENE.attrs` 相当の登録を 1 行出す（例: `PASTA.scene_attrs(SCENE, "__start__", { {"天気", "晴れ"} })`。形は設計で決める）。
- ランタイム: `scene.lua` に属性の別表と `SCENE.attrs`。`finalize.rs` が別表を集めて `register_global_raw` に渡す。`SceneInfo.attributes` を順あり・型ありに替える。
- トレードオフ: ✅ 既存の「Lua に出して辞書確定で集める」流れに沿う。✅ 偽のローカルを出さない（別表）。✅ `act.lua`・`scene_join.rs`・`search/context.rs` を触らない（`scene-count`・`debug-mcp-runtime` と重ならない）。❌ 属性のあるシーンの生成 Lua が変わる（見本とスナップショットの更新は必須だが、要件が認めている）。

### Option B: Rust だけで運ぶ（生成 Lua に出さない）

- トランスパイル時の登録表（`TranspileContext.scene_registry`）を辞書確定に持ち越す、またはサイドカーに書いて辞書確定で読む。
- トレードオフ: ✅ 生成 Lua が全く変わらない。❌ 辞書確定は Lua の `get_all_scenes()` を正として登録表を作り直す設計で、トランスパイル時の表は捨てている（キャッシュから読むときはトランスパイルを通らない）。別の持ち越し経路（サイドカー）を新設することになり、ソースマップのサイドカーと同じ鮮度の問題を抱える。❌ Lua から読むには結局 Rust → Lua の口が要る。採らない。

### Option C: A を基本に、シーン表の型だけ段階的に（ハイブリッド）

- A の Lua 側と辞書確定はそのまま。`SceneInfo.attributes` の型は、この spec では「順あり・型あり」を足して既存の `HashMap<String, String>` の写しも残し、`scene-condition-attributes` が条件の形に作り直すときに写しを消す。
- トレードオフ: ✅ `filter_by_attributes` とそのテストを触らずに済む。❌ 同じ属性を 2 つの形で持つ期間ができる（後の spec が消す約束が要る。brief の「申し送りはその場で書く」方針で `scene-condition-attributes` の brief に書く）。

## 4. 規模とリスク

- **規模: M**（brief の 15〜18 タスクに一致）。パーサ 1、トランスパイル 3、生成 2、ランタイム Lua 2、辞書確定 2、シーン表 2、検査の一致 1、見本とスナップショット 1、マニュアルと再生成 2、ハイライトは 0。
- **リスク: Low〜Medium**。既存の流れを延ばすだけで新しい技術は無い。Medium にするのは、(1) `SceneInfo.attributes` の型変更が `pasta_check` と `scene_table` のテストに波及すること、(2) 未決事項 1・2（受け継ぎ）の決定で `register_global_raw` の作りが変わること。どちらも要件ディスカッションで決まれば Low に落ちる。

## 5. 設計フェーズへの推奨

- **推奨: Option A**（必要なら C）。倉は `scene.lua` の別表＋`SceneInfo` の順あり配列。読み出しの口は `SCENE.attrs` の 1 つ。`act.lua`・`search/context.rs`・`scene_join.rs`・`finalize_scene_impl` の署名は触らない。
- **設計で決めること**:
  - 生成 Lua の属性の出し方（1 行の形と置き場。`init_scene` の引数は崩さない）
  - 順ありの入れ物（`Vec<(String, AttrValue)>` で十分。`indexmap` は足さない）
  - `SceneInfo.attributes` の新しい型と、`filter_by_attributes` の合わせ方（A か C）
  - `register_global_raw` にローカルごとの属性を渡す口（Requirement 2.6 を採る場合）
  - 一致テスト（`pasta_check` ↔ pasta.dll）に属性の比較を足す位置
  - DSL の Lua ブロックからモジュールへ届く名前。生成コードの `local SCENE = PASTA.create_scene(…)` はシーン表であって `pasta.scene` モジュールではないので、ブロックの中では `require("pasta.scene")` か `PASTA` 経由の別名が要る（`act.scene_name`・`act.local_scene_name` が実行中の登録名を持つ）
- **要件ディスカッションに送る問い**: requirements.md の「未決事項」1〜11。特に 1（ローカルへの受け継ぎ）・2（名前なし `＊`）・6（条件と倉の共有）・9（口の形）は倉の形を変えるので、設計の前に決める。
- **申し送り**: Option C を採るなら、`scene-condition-attributes` の brief に「文字列の写しを消す」を書く。Requirement 2.6 を採るなら、`cue-command-runtime` の brief に「親の柱は親の登録名で読む」を書く。

## 6. 要件ディスカッション後の追記（2026-11-03）

議題の確定で、設計に渡す材料が次のように変わった。上の節の「Option A」は有効のまま。

- **条件の形をこの spec で収集する**（議題 5）。`grammar.pest` 162〜166 の `attr` に、`＆＄変数 演算子 値` と `＆＄変数`（フラグ）の枝を足す。属性の中でだけ `＞`（Call の印）と `＝`（代入の印）を比較として読む見分けを、文法で書く（`attr_string` は `kv_marker | attr_marker | comment_marker` を除外しているので、比較演算子の列も同じ場所で除外する）。構文木の `AttrValue` の隣に、項目の種類（宣言・条件・フラグ）と演算子（半角の正規形 6 種）を持つ形を足す。右辺に変数は書けない。実測（2026-11-03）: 今はどの位置でも条件の形はパースエラー（`＊雑談＆＄＊変数＞１００` は 1:5、ファイル先頭は 1:2）。
- **名前なしの `＊` に属性を書ける**（議題 2）。`global_scene_continue_line`（239）に `attrs?` を足す。受け継ぎは名前だけ（`parse_scene.rs` 88〜91 はそのまま）。
- **ファイル属性は 1 行で置き換え**（議題 4）。`context.rs` 72 の `accumulate_file_attr` は「累積」から「置き換え」に変える（後の行が前の組を捨てる）。シーンを挟まずに 2 行続いたらパースエラー。`file_scope`（253〜254）の中で属性行を高々 1 つに制限する。`internals/transpiler.md` 164・171 の「累積」の記述を直す。ついでに `file_scppe_item` の書き間違いを直す（`scene-condition-attributes` の棚卸で見つけたもの。文法を触るこの spec へ移した）。
- **重なりの畳み方**（議題 3・4）。宣言の属性だけ「同じ名前は先に書いた位置に 1 つ、値は後出し」で畳む。条件とフラグは畳まない。並びは「ファイル属性 → 宣言行 → 初期部」。`merge_attrs`（87〜93）をこの規則で書き直す。
- **着色**（Requirement 7.5）。`editors/vscode/syntaxes/pasta.tmLanguage.json` は `grammar.pest` を写す作りで `tmGrammar.test.ts` が照合する（`ponytail-editor-tooling`）。条件の形と名前なしの `＊` の属性を同じ写し方で足す。`pasta_lsp` の `visit_scope.rs` も宣言行の属性を着色しているので、条件の形に追従させる。
- **規模の見直し**: 文法と構文木 2・着色 1 が増えて 18〜21。`scene-condition-attributes` から同じ分が減るので、0.5.0 全体では増えない。設計で 20 をはっきり超えるなら、着色（LSP・VSCode）を後ろのタスクにまとめる。
- **設計で決めること（追加）**: 条件の項目を生成 Lua に出す形（宣言と同じ 1 行に混ぜるか）。Rust のシーン表で条件を持つ型（`filter_by_attributes` の `HashMap<String, String>` との合わせ方は Option A か C のまま）。

## 7. 設計フェーズの調査と決定（2026-11-03・`/kiro-spec-design -y`）

> 1〜5 節の「Requirement N」は要件ディスカッションの前の番号である。今の requirements.md では、ファイル属性の受け継ぎが 3、型が 4、実行時の保持が 5、Lua の口が 6、互換が 7、マニュアルが 8 に当たる。この節から後は今の番号で書く。

### 要約

- **調査の種類**: 既存の仕組みの延長（Light discovery）。新しい依存は足さない（`indexmap` も足さない）。外部のライブラリ・API の調査は要らない（Web の調査はしていない）。
- **使った手順**: `kiro-spec-design` の `design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`。文法とランタイムの権威はマニュアル（`book/src/grammar/block-structure.md#属性`・`book/src/lua/script-api.md`）と現行のコードとし、ドメインのスキル（`pasta-ghost-authoring`・`pasta-lua-coding`）は読んでいない（どちらもマニュアルの写しで、新しい決まりを持たない）。
- **大事な発見**:
  1. 構文木はファイル属性を 1 つずつ `FileItem::FileAttr` に平らにする（`parser/mod.rs` 209〜217）。行の切れ目は残らないが、要件 3.3（シーンを挟まずに 2 行続いたらパースエラー）がパーサで効けば、「グローバルシーンの後に来た最初の `FileAttr` で組を捨てる」だけで要件 3.2 の置き換えになる。構文木の形を変えずに済む。
  2. `act` は実行中のシーンの**登録名**をグローバルだけ持つ（`act.current_scene.__global_name__`）。ローカルは**書いた名前**（`act.local_scene_name`。`act.lua` 205〜217）しか持たず、登録名（`選択肢_1`）を持たない。同じ名前のローカル（`・選択肢` が 2 つ）は、書いた名前では区別できない。要件 6.5 のローカルの場合は、生成コードを変えずには満たせない（下の決定 6・設計の未決 1）。
  3. `pasta_lsp` の `visit_decl_line`（`visit_scope.rs` 219〜243）は、宣言行の属性を構文木でなく文字から拾っている（パーサが捨てていたため）。パーサが宣言行の属性を残すと、`visit_global_scene` の `scene.attrs` の走査（35〜37）と二重に色が出る。宣言行の文字の走査から属性を外し、構文木の属性で色を出す形に揃える。
  4. VSCode の文法定義の `global-scene`（`pasta.tmLanguage.json` 78〜95）は名前を省ける形で、残りを `decl-rest` に流している。名前なしの `＊` 行の属性（`＊＆順番：２０`）は**今の定義のままで着色される**。足すのは条件の形（`＆＄…`）だけである。
  5. 2 行のファイル属性を使うテストが 2 本ある（`tests/transpiler/final_regression_test.rs` 133 の `kind_scene_attributes`、`tests/runtime/syntax_test.rs` 76〜の `test_attribute_inheritance`）。要件 3.3 でパースエラーになるので、1 行に書き直す。見本の辞書（`sample.pasta` 14 行）は 1 行なので形はそのままで、生成物だけ変わる。属性を持つ `.pasta` のフィクスチャは `sample.pasta` だけ（2026-11-03 の grep）。

### 調査の記録

#### 宣言行の属性の取り込み

- **きっかけ**: 要件 1.1・1.2・1.5。
- **見たもの**: `grammar.pest` 173（`scene = _{ id ~ s ~ attrs? }`）、238〜239・244、`parse_scene.rs` 73〜108・210〜232。
- **分かったこと**: `scene` は黙る規則なので、`attr` の対は `global_scene_line`・`local_scene_line` の子に直に並ぶ。`parse_global_scene_start` と `parse_local_scene_scope` は `id` だけ読んで残りを捨てている。`global_scene_continue_line` は `global_marker ~ or_comment_eol` で、`＊` の後に `s ~ attrs?` を足す余地がある。
- **影響**: `parse_global_scene_start` の戻りに宣言行の属性を足し、呼び出し側で `attrs` の先頭に積む。初期部の属性行（29〜35）は後に積まれるので、要件 1.3 の順（宣言行 → 初期部）は自然に出る。

#### 条件の形の文法

- **きっかけ**: 要件 2.1〜2.3・2.6。
- **見たもの**: `grammar.pest` 81〜84（`var_ref_*`）・106〜108（`dyn_name_*`）・162〜166（`attr`）。
- **分かったこと**: 既存の `dyn_name_local` は `var_id`（`id | digit_id`）なので `＄０`（引数番号）も通す。要件 2.1 の変数はローカルの `＄名前` とグローバルの `＄＊名前` だけなので、属性用に `id` だけを受ける規則を 2 つ足す。`attr_string` は `＄` で始まる文字列も通すので、条件の値の前に `!var_marker` を置けば要件 2.6（右辺に変数はパースエラー）になる。`attr_string` の除外の列（`kv_marker | attr_marker | comment_marker`）は**変えない**。6 節は比較演算子も除外する案を書いたが、そうすると今の宣言の値（`＆k：a＝b` は `a＝b`）が壊れる。条件の値の終わりは空白・`＆`・`＃` で決まるので、除外を足さなくても読み分けは崩れない。
- **影響**: 比較演算子は `≧ ≦ ≠ >= <= != ＞ ＜ ＝ > < =` の順に試す（2 文字の半角を 1 文字より先に）。

#### Lua → Rust の運び方と値の型

- **きっかけ**: 要件 4.1・4.2・5.1・5.3。
- **見たもの**: `runtime/finalize.rs` 44〜79・163〜186、`code_gen/expr_gen.rs` 144〜149（`Expr::Float` は `write!("{}", f)`。`1.0` は `1` と書かれる）、`runtime/log.rs` 91〜92（mlua の `Value::Integer`・`Value::Number` を両方受ける例）。
- **分かったこと**: LuaJIT の数は倍精度の 1 種類で、`1.0` と `1` を区別できない。辞書確定は Lua の倉から Rust の表を作り直すので、Rust の表で整数と小数を分ける手がかりは「整数値かどうか」しか無い。キャッシュから読み込むときはトランスパイルを通らないので、構文木の型を辞書確定へ持ち越す道は無い（3 節の Option B を採らない理由と同じ）。
- **影響**: Rust の値の型は `Integer(i64)`・`Float(f64)`・`String(String)` の 3 つにし、小数は「整数値で i64 に収まれば `Integer`」の 1 つの規則（`SceneAttrValue::from_f64`）で決める。トランスパイル時（`pasta_check` を含む）と辞書確定の両方がこの規則を通るので、2 つの表は同じ値になる。`＆率：1.0` は Rust では `Integer(1)`、Lua では `1`（設計の未決 2）。2^53 を超える整数は Lua で精度が落ちる（`shortcut:` として記す）。

#### 倉の置き場と偽のシーン

- **きっかけ**: 要件 5.4。
- **見たもの**: `scene.lua` 67〜74（シーン表は `{ __global_name__ = … }`）、`finalize.rs` 64〜66、`debug/source_map/scene_join.rs` 87、`store.lua` 35・91〜108。
- **分かったこと**: シーン表に属性のフィールドを置くと、`collect_scenes` と `scene_join.rs` の両方が、`__global_name__` 以外のキーをシーン関数として数える。
- **影響**: 倉は `STORE.scene_attrs[グローバルの登録名][ローカルの登録名]` の別表にする（3 節の Option A の (a)）。`STORE.reset()` で空にする。辞書確定は `require("pasta.store").scene_attrs` を直に読む（`SCENE` に読み出し用の関数を足さない）。

#### 一致テストの口

- **きっかけ**: 要件 7.4。
- **見たもの**: `pasta_check/src/validate/mod.rs` 26〜56（`module_name_of`・`pasta_dll_paths` などの `#[doc(hidden)]` の一致テスト用の口）、`tests/validate_parity_test.rs`（772 行。600 行の目安を超えている）。
- **影響**: 一致テスト用の口 `scene_attrs` を同じ形で足し、テストは別ファイル `validate_attr_parity_test.rs` に置く。pasta.dll 側は、読み込んだ VM で `SCENE.attrs` を呼んで比べる（既存の `static_call_lookup_matches_runtime` と同じ、Lua を評価して比べる形）。

## 8. 設計の比較と決定

### 倉の形の比較

| 案 | 内容 | 強み | 弱み | 採否 |
| -- | ---- | ---- | ---- | ---- |
| A | Rust のシーン表の `attributes` を順あり・型ありの `Vec<SceneAttr>` に置き換える。`filter_by_attributes` は宣言の項目の文字列で比べるよう合わせる | 形が 1 つ。読み手は 1 つの型だけ見ればよい | `register_global`・`register_local` の呼び出し（テストに約 130 か所）を `HashMap::new()` → `Vec::new()` に機械的に直す | 採る |
| C | `HashMap<String, String>` の写しを残し、型ありを横に足す | 既存のテストを触らない | 同じ属性を 2 つの形で持ち、後の spec に消す約束が要る。要件 5.5 の「見分けの決まりは 1 本」に反しやすい | 採らない |

### 決定 1: 受け継ぎと重なりの規則を `pasta_dsl` の 1 か所に置く

- **背景**: 要件 3・7.4。トランスパイラ（`pasta_lua`）と検査（`pasta_check`）が同じ規則で属性を作らないと、一致テストが落ちる。
- **候補**: (1) `pasta_dsl` に構文木の関数として置く。(2) `pasta_core` に倉の型の関数として置き、ファイル属性の置き換えは両方に書く。(3) 両方に書いて一致テストだけで守る。
- **採用**: (1)。`PastaFile::global_scene_attrs()`（n 番目のグローバルシーンの、受け継ぎと畳みを済ませた構文木の属性の並び）と `merge_attrs(inherited, own)` を `parser/ast/attr_merge.rs` に置く。構文木 → 倉の項目の写し（20 行ほど）は `pasta_lua` と `pasta_check` にそれぞれ置く（`pasta_core` は `pasta_dsl` に依存しないので、写しの関数を共有できない）。写しのずれは一致テストが拾う。
- **理由**: 難しいのは置き換えと畳みであり、それを 1 か所に置けば、両方に残るのは機械的な型の写しだけになる。

### 決定 2: 構文木の `Attr` は `key` と `body` に分ける

- **背景**: 要件 2.1・2.3・5.5。宣言・条件・フラグを書いた順に 1 本の並びで持つ。
- **候補**: (1) `Attr { key, body: AttrBody, span }`（`AttrBody::{Decl(AttrValue), Cond { op, value }, Flag}`）。(2) 今の `value` を残して `kind` を足す（フラグの値が「無いのに有る」形になる）。(3) `Attr` を enum にする（`key`・`span` の読み手が全部変わる）。
- **採用**: (1)。`Attr::value()`（宣言と条件の値。フラグは `None`）を足し、`attr.value` を読む既存のテスト（`pasta_dsl/tests` の 4 本ほど）は `attr.value()` に直す。
- **トレードオフ**: `Attr { key, value, span }` を組み立てるコード（7 か所。ほとんどテスト）を直す。

### 決定 3: 条件とフラグの名前は全角の印に揃えた文字列で持つ

- **背景**: 要件 5.5・6.2（`key = "＄…"`）。
- **採用**: 構文木と倉の `key` は `＄名前`・`＄＊名前`（`$`・`*` で書いても全角の `＄`・`＊` に揃える。名前の部分は書いたまま）。変数の別は `key` の先頭が `＄＊` かどうかで分かる。
- **理由**: 読み手の見分けが「先頭が `＄`」の 1 本で済む（要件 5.5）。半角で書いた条件と全角で書いた条件が、同じ変数として並ぶ。（設計の未決 3）

### 決定 4: 生成 Lua はシーンの `do` ブロックの頭に 1 シーン 1 行を足す

- **背景**: 要件 7.1・7.2・5.1。
- **候補**: (1) `local SCENE = PASTA.create_scene(…)` の直後に `PASTA.set_scene_attrs(SCENE, "登録名", { … })` を、属性のあるシーン（グローバルは `__start__`、ローカルは関数名）ごとに 1 行。(2) `act:init_scene` の引数に足す（毎回の呼び出しで運ぶことになり、属性の無いシーンとは引数の数がずれる）。(3) シーン表のフィールドにする（偽のシーンが出る）。
- **採用**: (1)。属性の無いシーンには 1 行も足さないので、属性を持たない辞書の生成物はバイト不変（要件 7.1）。シーン関数の本体と `init_scene` の行は変わらない（要件 7.2）。

### 決定 5: ファイル属性の 2 行目はパーサの組み立て（`build_ast`）で止める

- **背景**: 要件 3.3。
- **候補**: (1) 文法で `file_scope` の中の属性行を 1 つにする。(2) `build_ast` で「前のグローバルシーンの後に属性行を見たか」を数える。
- **採用**: (2)。`file` は `( file_scope | global_scene_scope | actor_scope )*` の繰り返しなので、文法で `file_scope` の中を 1 行にしても、次の繰り返しで 2 つ目の `file_scope` が始まって通ってしまう（アクター辞書を挟む場合も同じ）。組み立ての段なら 1 つの状態で数えられ、誤りの位置と前の行の行番号を書ける。数え直すのはグローバルシーンの後だけ（要件 3.3 の文言どおり。アクター辞書を挟んでも 2 行目は誤り。設計の未決 4）。

### 決定 6: Lua の口のローカル名は登録名（関数名）

- **背景**: 要件 5.1・6.1・6.5。
- **分かったこと**: `act` はローカルの登録名を持たない（発見 2）。
- **採用（仮）**: `SCENE.attrs(global_name, local_name)` の `local_name` は、`SCENE.get` と同じ登録名（`選択肢_1`。`nil` と `"__start__"` はグローバル自身）。実行中のグローバルシーンは `SCENE.attrs(act.current_scene.__global_name__, nil)` で読める。実行中のローカルシーンを `act` だけから読む方法は、この設計では用意しない。（設計の未決 1。要件 6.5 の文言の手直しが要るかもしれない）

### 簡単にしたこと（Synthesis）

- **簡素化**: `SCENE` に足す関数は、生成コードが呼ぶ `SCENE.set_attrs` と、読み手の `SCENE.attrs` の 2 つ。辞書確定は `STORE.scene_attrs` を直に読み、読み出し用の関数を足さない。`indexmap` を足さない。宣言の畳みは `Vec` の線形探索（1 シーンの属性は数個）。
- **一般化**: 宣言・条件・フラグを 1 つの並びの 3 種の項目にしたので、後続の 3 つの読み手（`scene-condition-attributes`・`scene-pick-modes`・`cue-command-runtime`）は同じ並びを読む。評価・重み・演出の形は先取りしない（`SceneAttr` に評価の関数を置かない）。
- **Build vs Adopt**: 外部の仕組みで置き換えられる部分は無い（DSL の属性の運搬は pasta の内部の流れ）。TOML などへの書き出しも要らない（生成 Lua の表の字句で足りる）。

## 9. リスクと手当て

- 2 行のファイル属性を書いている既存のゴーストは、更新でパースエラーになる（要件 3.3 の意図した変化）。属性は今まで効いていなかったので影響は小さいが、マニュアルの「よくある間違い」に書く。
- `register_global`・`register_local` の引数の型の変更は、テストに約 130 か所の機械的な直しを生む。1 タスクにまとめ、`HashMap::new()` → `Vec::new()` の置き換えだけにする。
- 2^53 を超える整数は Lua で精度が落ちる。重みや優先の範囲では起きない（`shortcut:` のコメントを置く）。
- `pasta_lsp` の宣言行の走査を構文木の属性へ切り替えるので、部分解析（構文エラーのある文書）のときの宣言行の色が変わらないかを、既存の `partial_token_test.rs` と `semantic_token_test.rs` 186〜278 で確かめる。
- `pasta_scripts`（`scene.lua`・`store.lua`・`init.lua`）を変えるので、自己展開の基準ダイジェストが変わる。ビルドで作り直される（手で触る所は無い）。

## 10. 設計ディスカッションの決定（2026-11-03・全議題を推奨案で承認）

- **決定 1（要件 6.5）**: `act` が持つのはグローバルの登録名（`act.current_scene.__global_name__`）とローカルの書いた名前（`act.local_scene_name`）だけで、ローカルの登録名（`選択肢_1`）は持たない。`act` から読むのは実行中のグローバルシーンだけにし、ローカルは検索の結果の登録名で読む。要件 6.5 を改めた。`init_scene` に登録名を渡す案は全シーンの生成物を変えて 7.1 に反し、書いた名前を受ける案は同名のローカルを取り違える。
- **決定 2（要件 4.2）**: 整数の値になる小数（`＆率：1.0`）は整数と同じに扱う（Rust は `Integer(1)`）。LuaJIT が `1.0` と `1` を区別できず、辞書確定は Lua の倉から読むので、構文木・`pasta_check`・辞書確定の 3 経路を `from_f64` の 1 規則にそろえる。要件 4.2 に一言足した。
- **決定 3**: 条件の変数の `key` は印だけ全角にそろえる（`＆$*exp>=5` → `＄＊exp`）。`＄０`（引数番号）の条件はパースエラー。
- **決定 4**: ファイル属性の 2 行目の数え直しはグローバルシーンの後だけ。アクター辞書を挟んだ 2 行もパースエラー（要件 3.3 の文言どおり）。
- **決定 5**: `attr_string` は変えない（6 節の「比較演算子も除外」は撤回）。条件の枝を先に試すので要らず、`＆k：a＝b` の値を壊さない。
- **決定 6（要件 7.1・7.2）**: 属性の文は `create_scene` の行の後ろに `; ` で続け、改行を足さない。`init_scene` の最後の引数（`"sample.pasta|26:25,…"`）は生成 Lua の行番号を埋め込むので、行を足すと同じファイルの後のシーンの文字列が変わり 7.1 に反する（設計の草案の「行を足しても対応はずれない」は誤りだった。`sample.generated.lua` で確かめた）。
