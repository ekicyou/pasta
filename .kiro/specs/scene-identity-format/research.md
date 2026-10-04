# ギャップ分析: scene-identity-format

## 1. 現状調査（要件 ↔ 既存資産マップ）

| 要件 | 既存資産 | ギャップ |
|---|---|---|
| R1 登録名の形式 | `pasta_scripts/pasta/scene.lua` 132 行 `base_name .. counter`（実行時の登録名の唯一の生成元。生成コードは `PASTA.create_scene("基本名")` と基本名だけを渡し、通し番号は Lua が `STORE.counters[基本名]` で振る） | **Missing**: 区切りが無い。1 行の変更で形式は変わるが、作る・分ける関数が無い |
| R1/R6 トランスパイル時 | `src/transpiler.rs` 206 行 `format!("{}{}", sanitize_name, counter)`（トランスパイル時の単語レジストリのスコープ名）。一方 `pasta_core::SceneRegistry::register_global` は既に `{sanitize}_{counter}::__start__`（`_` 区切り） | **Constraint/発見**: Rust 側に既に 3 形式が混在（単語=区切り無し、シーン=`_`、ランタイム=区切り無し）。辞書確定前（`main.lua`・`entry.lua` 実行中）の `@pasta_search` はトランスパイル時レジストリを使うため、グローバル検索が `メイン_1` を返すのに `STORE.scenes` のキーは `メイン1` で `SCENE.get` が外れる潜在不整合がある。`_` 区切りに揃えるとこの不整合も解消する |
| R2 検索 | `pasta_core` `SceneTable` の `prefix_index`（グローバル=登録名、ローカル=`:親:ローカル`）への前方一致。`SearchContext::search_scene` は第 1 引数を照合規則で揃える | 形式を変えれば `＞A1` が `A_1` に一致しなくなる。アルゴリズム変更は不要。**Constraint**: 照合用の名前が `_数字`／`_` で終わる検索キーは、区切り `_` を越えて別名シーンの通し番号部分に前方一致しうる（R2.7・OQ1） |
| R3 Lua API | `WORD.create_local(global_name, key)`・`scene:create_word`（`__global_name__` 経由で透過）・`search_word` は第 2 引数を照合規則で揃える | `_` は照合規則で不変なので動く。旧形式を明示的に書いた利用者コードは壊れる（OQ2） |
| R4 デバッガ索引 | `src/debug/source_map/scene_join.rs` `split_runtime_global`（末尾 ASCII 数字を全部通し番号とみなす）→ `(base, counter)` 表で `G:{base}#{counter}` と突合 | **Missing**: 記録ごとに「基本名＋区切り＋番号」を組み立てて実行時の登録名集合と照合する方式へ置換（brief の方針）。`scene_index.rs`・`playscene.rs` は ID を不透明な文字列として扱うため変更不要の見込み |
| R5 キック | `kick.lua` 140–150 行: グローバルは `SCENE.co_exec(act, name)` → `act:find_scene` → `find_handler` の 5 段（L3 の act メソッド・L4 の `GLOBAL[key]`・L5 の前方一致まで探す）。ローカルは `SCENE.search(local, parent)`（前方一致） | **Missing**: 完全一致の口は既存（`SCENE.get_start(global)`・`SCENE.get(parent, local)`）。コルーチン化は既存の `wrap_local_func` を流用できる。`act.lua` に触れずに `kick.lua` だけで完結する |
| R6 一元化 | Rust: `SceneRegistry::sanitize_name` が照合規則の一元化点（前例）。Lua: `scene.lua` | **Missing**: 形式関数（作る・分ける）が無い。Rust 側の置き場所候補は `pasta_core::SceneRegistry`（transpiler・debug の両方が依存済み） |
| R7 マニュアル | `book/src/lua/modules/pasta-search.md` 62・74–80・108・119・150 行、`lua/patterns.md` 157–158 行、`lua/script-api.md` 449・484 行、`internals/internal-modules.md` 237–246・358–379 行（「区切り無し」）、`internals/debug.md` 348–349 行（`split_runtime_global`） | 記述・例の更新。スキル `references/` は `book/tools/gen-skill-refs.mjs` で再生成、`link-check.mjs` で検査 |
| R8 テスト | `scene_join_tests.rs`（`split_runtime_global` 直接テスト 9 箇所）、`scene_index_tests.rs`（28）・`playscene_tests.rs`（16）・`wiring_play_scene_at_tests.rs`（8）は `会話1` 形の文字列を ID として使用 | トランスパイラのスナップショットはローカル関数名（`SCENE.名前_1`）と `PASTA.create_scene("基本名")` だけを含み、グローバル登録名を含まない → **スナップショットはほぼ変わらない見込み**（brief の「スナップショットを広く更新」は過大評価の可能性）。期待値更新は Lua 側の統合テスト・lua_specs・デバッガの ID 文字列が中心（件数は設計時に確定: Research Needed） |

その他: VSCode 拡張（`editors/vscode/src/runSceneAtCursor.ts`）は scene 名を解析しない。サンプルゴースト辞書に登録名の直書きは見当たらない。永続化（`save`）に登録名が載るかは Research Needed（`STORE.last_global_scene` は非永続の見込み）。

## 2. 実装アプローチの選択肢

### Option A: 既存を拡張（最小差分）
- `scene.lua` に `SCENE.make_global_name(base, n)`／`split` を置き `create_scene` から使う。Rust は `SceneRegistry` に `global_name(base, n)` を足し、`transpiler.rs`・`register_global`・`scene_join.rs` から使う。`split_runtime_global` は削除し、記録ごとに組み立てた名前で `HashSet<登録名>` を引く。`kick.lua` は `get_start`／`get` の完全一致へ。
- ✅ 差分小・既存パターン（`sanitize_name` の一元化）に沿う。❌ R2.7 の残余制限は残る。

### Option B: 区切りを照合規則の外の文字にする（`#`・`.` 等）
- 前方一致が通し番号部へ越境しなくなり R2.7 の制限が消える。
- ❌ `scene-search-key-normalization` 要件 3.2（登録名を検索キー／スコープに渡すと照合規則で揃えられる）と衝突し、`search_word` のスコープ・`search_scene` の第 1 引数で登録名が一致しなくなる。照合の入口側の変更が必要で、Out of scope（サニタイズ規則）に踏み込む。

### Option C: ハイブリッド（段階出荷）
- 第 1 段: キックの完全一致＋索引の修正（形式非依存。小さく独立に出せる）。第 2 段: 形式変更＋一元化＋マニュアル。
- ✅ リスク分離。❌ 索引修正は形式変更後に組み立て方式へ再度書き換える二度手間になりうる（第 1 段で組み立て方式にしておけば回避可）。

## 3. 工数・リスク
- **工数: M**（3–7 日）。コード変更は数箇所だが、テスト期待値・マニュアル 5 章・references 再生成・Lua/Rust 両側の往復テストが必要。
- **リスク: Low〜Medium**。既存パターンの延長。公開 API（登録名の文字列）の破壊的変更と、R2.7 の残余制限の扱いが判断点。

## 4. 設計フェーズへの推奨
- 推奨は Option A（必要なら C の順序で出す）。区切りは `_`（照合規則で不変・`:` を含まない・最後の `_`＋数字で一意に分解可能）。
- 決めること: 形式関数の置き場所（Rust は `pasta_core::SceneRegistry` が有力）、辞書確定前の検索の整合（トランスパイル時単語レジストリのスコープ名も新形式へ）、旧形式の扱い（OQ2）、R2.7 の制限をマニュアルに書く文面。
- Research Needed: 期待値更新が要るテストの全数、永続化データに登録名が含まれるか、`scene-attribute-store`／`call-execution-correctness` が前提にする形式関数の公開範囲。

## 5. 要件ディスカッションから設計フェーズへ送る判断（カテゴリ B）
- **出す順序（OQ4）**: キックの完全一致と索引の修正を、形式の変更より先のタスクに置くか、別の PR として先に出すか。どちらも同じ spec の中で決める（設計・タスクフェーズ）。
- **スナップショットへの影響と Wave 2 の並走条件（OQ5）**: グローバルの登録名はスナップショットに出ない見込み。ただしローカルシーンの形式を変える場合は、生成コードのローカル関数名（`function SCENE.選択肢_1`）が変わるので、スナップショットが広く変わる。議題 2 の結論を受けて、設計で見積もり直す。下流の `call-execution-correctness` との順序制約は維持する。
- **キックの実装の道筋**: グローバルは `SCENE.get_start`、ローカルは `SCENE.get` で完全一致に引き、コルーチン化には `wrap_local_func` を流用する。`act.lua` には触れない。
- **辞書確定前の検索との整合**: トランスパイル時の単語レジストリのスコープ名（`transpiler.rs` 206 行）と `SceneRegistry::register_global` を、一か所の形式関数にそろえる。
- **要調査**: 期待値の更新が要るテストの全数。保存データ（`save`）に登録名が載るかどうか（載るなら移行の要件を足す）。

## 6. 追加調査の結果（遅れて戻ったコード調査）
- **保存データ**: フレームワークが永続化するのは利用者の `save` テーブルだけ。`STORE.scenes`・`counters`・`last_global_scene`・`kick_pending` はメモリ上のみで、`store.lua` 96–105 行でリセットされる。デバッグ用サイドカー（`sidecar.rs`）は行の対応だけを持つ。**移行の要件は不要**。利用者のスクリプトが `__global_name__` などを自分で保存している場合だけが例外。
- **見落としていた変更箇所**:
  - `crates/pasta_shiori/tests/support/scripts/pasta/scene.lua` 113 行は `scene.lua` の複製。同じ変更が要る。
  - `book/src/internals/registry-search.md`（101・111–114・130–131・195・239 行。239 行は辞書確定前と確定後で形式が違うことの説明で、変更後は不要になる）。
  - `book/src/internals/transpiler.md` 132 行。
- **デバッガの結合キー**: `scope_gen.rs` が出す `G:{名前}#{番号}`・`L:{親}#{番号}:{関数名}` は、既に `#` を区切りに使っている（`parse_base_counter`）。登録名の形式とは独立。
- **単語の登録キー**: 確定時に `WordDefRegistry::register_local` が `":{sanitize(登録名)}:{キー}"` を作る。区切りを照合規則で置き換わる文字にする場合は、ここと `search_word` の第 2 引数の照合をそろえる必要がある（議題 1 の結論しだい）。
- **期待値の更新が要るテスト**（形式に依存するもの）: `scene_identity_index_test.rs`（約 52 箇所）、`finalize_scene_test.rs`、`local_scene_call_test.rs`、`runtime_toggle_e2e_step_test.rs`、`runtime_safety_test.rs`、`lua_specs/scene_registry_test.lua`、`scene_join_tests.rs`。トランスパイラの insta スナップショット 29 件は影響を受けない。`search/context.rs` の「確定前／確定後」2 形式のテストは、形式が 1 つにそろうと区別する意味がなくなる。
