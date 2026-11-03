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
