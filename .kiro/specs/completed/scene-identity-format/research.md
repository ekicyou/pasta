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

## 7. 再検証: 根本原因と本質的な解決策（要件ディスカッション中の深掘り）

### 根本原因
不具合は 3 つに見えるが、根は 1 つである。**「シーンを 1 つに特定する名前（登録名）」を、「作者が書いた名前で探すための索引のキー」にも使っている**。

- 検索の索引（`pasta_core` `scene_table.rs` の `prefix_index`）のキーは登録名（名前＋通し番号）である（`fn_name_to_search_key`）。作者が書いた名前との前方一致は、この登録名に対して行う。
- そのため、検索キーが通し番号の部分にまで一致する。区切りが無ければ `＞A1` が `＊A` の 1 個目に一致し、区切りを `_` にしても `＞章・1`（照合用の名前 `章_1`）が `＊章` の 1 個目（`章_1`）に一致する。区切りの文字を何にしても、名前に出てくる文字である限り同じ種類の取り違えが残る。
- 位置からのキックは逆向きの同じ誤りである。1 つに特定済みの登録名を、名前で探すための前方一致の検索に渡している。
- デバッガの `split_runtime_global` は、登録名から名前と番号を推測で分けている。

マニュアル（`lua/modules/pasta-search.md`）は既に「`name` は**シーン名の照合用の名前**と前方一致させる」と書いている。登録名と前方一致させている実装のほうが、マニュアルと食い違っている。

### 本質的な解決策（Option D）
「特定する名前」と「探すための名前」を分ける。

1. **登録名**は `照合用の名前_通し番号` にする（一意で、最後の `_` の後ろの数字で一通りに分けられる）。Rust のトランスパイル時の登録表（`register_global`）とローカルシーンは既にこの形なので、3 つあった形式が 1 つにそろう。`_` は照合規則で変わらないので、単語のスコープ（`search_word` の第 2 引数・`register_local`）には手を入れなくてよい。
2. **検索の索引のキーは、通し番号を除いた照合用の名前にする**（グローバルは `名前`、ローカルは `:親の登録名:名前`）。前方一致・シャッフル＆順次消費はそのまま。検索キーが通し番号に一致することが原理的に無くなり、要件案 2.7 の「既知の制限」は不要になる。区切りに `#` を使う案（Option B）も不要になる。
3. **位置からのキック**は登録名の完全一致で引く（`SCENE.get_start`・`SCENE.get`）。
4. **デバッガの索引**は、記録（名前・通し番号）から登録名を組み立てて突き合わせる。推測で分けない。

### この策で変わること・注意点
- **`search_scene` の第 1 引数に登録名を渡す使い方は成立しなくなる**（`search_scene("メイン_1")` は「`メイン_1` で始まる名前のシーン」を探す）。上流 spec `scene-search-key-normalization` 要件 3.2（登録名を渡したときの結果を変えない）を、本 spec で意図して改める。この使い方をしている内部コードは位置からのキックだけで、それは完全一致に置き換える。マニュアルは第 1 引数を「シーン名」としか書いていない。
- **編集範囲が brief より広がる**: `pasta_core` の `scene_table.rs`（索引のキー）と、確定時に名前を渡す `runtime/finalize.rs`。Wave 2 で並走する `search-selector-indices` が `scene_table.rs` のシャッフル部分に触れる可能性があるので、重なりを確かめる。
- **確定時に通し番号を除いた名前を得る方法**（設計で決める）: 登録名を形式の関数で分けるか、Lua 側がシーン表に名前と番号を記録して渡すか。Lua で直接登録したシーン・関数（通し番号を持たない名前。例: Lua ブロックの `function SCENE.加算ループ`）は、名前全体をキーにする。
- **決まった順に選ぶとき（`set_scene_selector`）の並び**: 同名シーンは索引の同じキーに入るので、並びは通し番号順に決める必要がある（今は登録名の文字コード順で `メイン10` が `メイン2` より先。確定時の登録順は `HashMap` の走査順で不定）。
- **辞書確定前のローカルシーン**: トランスパイル時の `register_local` は番号に定義位置を使い、生成コードは名前ごとの通し番号を使う（既存の食い違い）。形式の関数を 1 つにするときにそろえる。

## 8. 設計フェーズの調査と決定（2026-10-04）

- **Discovery Scope**: Extension（既存システムの拡張。light discovery）。外部依存の追加なし。
- **Key Findings**:
  - 生成コードは 1 バイトも変えずに済む（登録名は Lua の `create_scene` が作り、ローカル関数名の形は変えない）。トランスパイラの insta スナップショットは変わらない。
  - 検索キーの変更は `SceneTable::fn_name_to_search_key` の 1 関数で、辞書確定前・確定後の両方に効く。
  - 本番でキックを設置する経路は `playscene.rs` の `build_kick_scene`（確定済みの identity）とリロードの予約文字列だけである。

### 8.1 コードで確かめたこと

- **登録名の生成元**: `scene.lua` 132 行（`base_name .. counter`）、`scene_registry.rs` 78 行（`{}_{}::__start__`）、`transpiler.rs` 206 行（`{}{}`）。`register_local`（120〜126 行）は `{親}_{番号}::{名前}_{local_index}`。
- **検索キー**: `scene_table.rs` 139〜147 行 `fn_name_to_search_key`。検索の結果は `SearchContext::parse_fn_name` が `fn_name` を `::` で分けて返す（キーとは独立）。
- **辞書確定の順序**: `finalize.rs` 159〜178 行。`HashMap<String, Vec<String>>` を走査して `register_global_raw` を呼ぶ。ローカル名の順は Lua の `pairs` の順。現状はキーが 1 シーン 1 つなので候補の並びに影響しないが、同名シーンが同じキーに入ると登録順が並びを決める。
- **辞書確定前のローカルシーン**: `transpiler.rs` 220〜224 行は `local_idx + 1`（無名の開始シーンも数える定義位置）を渡す。`scope_gen.rs` 177〜187 行は名前ごとの通し番号（生の名前がキー）で関数名を作る。一致しない。
- **辞書確定前のレジストリの中身**: `loader/process.rs` はファイルごとに新しい `TranspileContext` でトランスパイルして `merge_from` する。通し番号はファイルごとに 1 から始まる（`merge_from` は番号を振り直さない）。キャッシュが新しいファイルはトランスパイルを飛ばすので、そのファイルのシーンはレジストリに入らない。さらに `main.lua`・`entry.lua` の実行中は `scene_dic` がまだ読み込まれておらず、`STORE.scenes` は空である。
- **デバッガの記録**: `loader/source_map_build.rs` もファイルごとにトランスパイルするので、`join_key` の通し番号はファイルごとである。複数ファイルに同名のグローバルシーンがあると、2 つ目以降のファイルの記録は 1 つ目のファイルのシーンに解決される（既存の制約。名前の形によらない。テストは 1 ファイルのフィクスチャだけ）。
- **キック**: `kick.lua` 140〜150 行。`KICK.install` を呼ぶのは `SHIORI.kick` だけで、`KickRequest` を作る本番コードは `debug/playscene.rs` 76 行と `debug/wiring/inbound.rs` 418 行（リロード）だけ。`virtual_dispatcher.lua` 245 行が `KICK.try_dispatch(act)` を呼ぶ。
- **登録名を使う Lua コード**: `act.lua`（`__global_name__` を読むだけ）、`choice_select.lua`（`SCENE.search(選択 ID, last_global_scene)`。選択 ID は作者が書いた名前）、`boot.lua`・`init.lua`・`virtual_dispatcher.lua`（イベント名で `co_exec`）。登録名を第 1 引数に渡して検索しているのはキックだけ。
- **`pasta_shiori` のテスト支援 `scene.lua`**: 現行 `scene.lua` の複製ではなく古いランタイムの写し（`SCENE.search`・`co_exec` を持たない）。113 行に `base_name .. counter` がある。
- **`search-selector-indices` との重なり**: 同 spec は `random.rs` を中心にし、`scene_table.rs` では `select_from_cache`（264〜289 行）の `shuffle_usize` の呼び出しに触れうる。本設計が触るのは 73〜147 行（`fn_name_to_search_key` とコメント）。ソースは重ならない。マニュアルは `lua/modules/pasta-search.md`（セレクタの節 124〜155 行）と `internals/registry-search.md` が重なる。

### 8.2 設計の決定

#### Decision: 通し番号を除いた名前は、1 つの分ける関数で得る

- **Context**: 辞書確定で、検索キーに使う「通し番号を除いた名前」をどう得るか（5 章・7 章の持ち越し）。
- **Alternatives Considered**:
  1. Rust の `split_registered_name` で登録名を分ける。
  2. Lua がシーン表に名前と番号を記録し、辞書確定で渡す（グローバルは `create_scene` が、ローカルは生成コードが記録する）。
- **Selected Approach**: 1。`SceneTable::fn_name_to_search_key` が `fn_name` の各部分を分ける。
- **Rationale**: 2 はローカルシーンについて生成コードの変更が要る（`function SCENE.名前_N` は代入であり、生成されたものと手書きを表の上で見分けられない）。スナップショット 29 件が変わり、下流の spec との順序制約が重くなる。1 は生成コードを変えず、辞書確定前と確定後が同じ 1 関数を通る。
- **Trade-offs**: 手書きの `_数字` で終わるシーン関数は、生成されたローカルシーンと見分けられない。要件 2.11 が認める「見分けられない場合」として、最後の `_` と数字を除いた部分を照合相手にし、マニュアルに書く。
- **Follow-up**: 設計ディスカッションで確認する（design.md Open Questions 1）。

#### Decision: 同名シーンの並びは辞書確定で決める

- **Context**: 要件 2.9。同名シーンが同じキーに入るので、キーの中の並びが登録順になる。
- **Alternatives Considered**:
  1. `finalize.rs` の `build_scene_registry` が（名前, 番号）の昇順で登録する。
  2. `SceneTable::from_scene_registry` がキーごとに ID を並べ替える。
- **Selected Approach**: 1。
- **Rationale**: 順序が不定になる原因（`HashMap` と `pairs`）は辞書確定にある。`scene_table.rs` の変更を 1 関数に抑え、並走する `search-selector-indices` との重なりを最小にする。トランスパイル時レジストリは定義順に登録するので、並べ替えなくても同じ並びになる。
- **Trade-offs**: `SceneTable` は「登録順が通し番号順である」ことを呼び出し側に頼る。

#### Decision: キック・索引の修正と形式の変更は 1 つの PR で、キックを先のタスクにする

- **Context**: 5 章 OQ4。
- **Selected Approach**: 1 spec 1 PR。タスクの順序は「規則の関数 → キックの完全一致 → 形式の切り替え（Lua・Rust・索引を同時に）→ 検索キーと登録順 → マニュアル」。
- **Rationale**: キックの完全一致は形式に依存せず単独で通る。索引の組み立て方式は、Lua の形式と同時に切り替えないと突き合わせが全滅するので、形式の切り替えと同じタスクにする。別の PR に分けると、要件 3.5 の告知と完了フロー（squash マージ）が 2 回になる。

#### Decision: Lua 側には作る規則だけを置く

- **Context**: 要件 6.1「Rust 側とランタイム側でそれぞれ一か所」。
- **Selected Approach**: Lua は `create_scene` の 1 行が作る規則。分ける関数は Lua に置かない。
- **Rationale**: Lua に登録名を分ける処理が無い。使う所の無い関数を足さない。Rust と Lua の食い違いは、Rust が組み立てた名前と実行時の登録名を突き合わせるテスト（索引の統合テストを含む）で検出する（要件 6.3）。
- **Follow-up**: 設計ディスカッションで確認する（Open Questions 2）。

#### Decision: 辞書確定前は形式と照合相手だけをそろえる

- **Context**: 要件 2.12 と 8.1 の調査（確定前のレジストリはファイルごとの番号で、キャッシュ済みファイルを含まない）。
- **Selected Approach**: 単語スコープ名を `registered_name` に、ローカルシーンの番号を生成器と同じ名前ごとの通し番号にそろえる。ファイルをまたぐ番号とキャッシュ済みファイルは既存の制約のまま。
- **Follow-up**: Open Questions 3。

### 8.3 Synthesis（一般化・既存の採用・単純化）

- **一般化**: グローバルの登録名とローカルの登録名は同じ形（`名前_番号`）なので、作る・分けるの 2 関数を両方に使う。検索キーの規則もグローバル・ローカルで同じ「分けた名前」になる。
- **既存の採用**: `SceneRegistry::sanitize_name`（一元化の前例）の隣に置く。キックは既存の `SCENE.get`・`SCENE.get_start`・`wrap_local_func` を使う。索引は既存の `locals_by_global` のキーを実行時の登録名の集合として使う。
- **単純化**: 新しい型・モジュール・設定を足さない。Lua の分ける関数、`SceneTable` の並べ替え、生成コードへの記録、旧形式の別名は足さない。

### 8.4 リスク

- 手書きの `_数字` で終わるシーン関数の照合相手が変わる — マニュアルと PR 本文に書く。
- 期待値の更新が要るテストが多い（Lua のテスト・デバッガのテスト） — 最初のタスクで `cargo test --all` の失敗一覧から全数を確定する。
- `search-selector-indices` とマニュアル 2 章が重なる — 後からマージする側が取り込む。
- ファイルをまたぐ同名シーンのデバッガ索引は直らない（既存の制約） — 別の spec に送るかをディスカッションで決める。
