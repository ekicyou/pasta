# ギャップ分析: scene-search-key-normalization

> 実施日: 2026-10-04 ／ 対象: `requirements.md`（要件 1〜7）と現行 main（8033b4d7）のコード
> 位置づけ: 要件ディスカッションの入力。最終判断はしない（選択肢と論点を示す）。

## 1. 分析サマリ

- **根本原因は「検索の入口だけが照合規則を通らない」ことに尽きる**。照合規則 `SceneRegistry::sanitize_name`（`crates/pasta_core/src/registry/scene_registry.rs:241`）は、登録側（`register_global`・`register_local`・`increment_counter`・`WordDefRegistry::register_local`/`register_actor`）と生成側（`scope_gen.rs:120,241`・`transpiler.rs:206`）ですでに共有されている。共有されていないのは検索側（`SearchContext::search_scene` と `actor.lua` のアクタースコープ組み立て）だけである。
- **シーン（要件 1・2）は 1 行級の修正で全経路に効く**。`SearchContext::search_scene`（`crates/pasta_lua/src/search/context.rs:68`）の冒頭で第 1 引数 `name` だけを照合規則に通せば、`SCENE.search`・`act:find_scene`（Call の 2・5 段）・`SCENE.co_exec`・選択肢ルーティング・キック・辞書確定前の検索がすべてこの入口を通る。照合規則は登録名（英数字と `_` のみ）に対して冪等なので、要件 3.2 は追加の仕掛け無しで満たせる。
- **アクター（要件 4）は方向の選択が必要**。Lua 側に照合規則の実装が無く、`actor.lua:142` が元の名前でスコープ `__actor_{name}__` を組み立てている。「検索側で規則を通す（Rust に入口を足す）」か「登録側で規則をやめる」かの 2 方向があり、衝突の扱い（要件 5.2）とキー構文の安全性が変わる。
- **マニュアルは現行の不具合挙動を明文化している**（`grammar/call-jump.md:33`「書いた名前をそのまま検索キーにする」、`internals/registry-search.md:226`「検索キーとして渡す名前はサニタイズされない」）。要件 6.3 の書き換え対象が明確。
- **工数 S・リスク Low**。並走条件（編集可能範囲）の中で完結する。最大の論点はアクターの方向と衝突時の警告の有無で、どちらも設計判断であり技術的未知は小さい。

## 2. 現状調査

### 2.1 照合規則と登録キー

| 対象 | 登録キーの作り方 | 場所 |
| ---- | ---------------- | ---- |
| 照合規則 | `name.replace(|c| !c.is_alphanumeric() && c != '_', "_")`（Unicode の Alphabetic/Numeric を保持。かな・漢字・長音 `ー` は残り、`・`(U+30FB)・`·`(U+00B7)・`＿`(U+FF3F, Pc)・結合文字などが `_` になる） | `scene_registry.rs:241` |
| グローバルシーン（トランスパイル時） | `fn_name = {sanitize(name)}_{counter}::__start__`、検索キー `{sanitize}_{counter}`。カウンタは照合用の名前ごと | `scene_registry.rs:78,231` |
| グローバルシーン（実行時・確定後） | 生成コードが `PASTA.create_scene("{sanitize(name)}")` を呼び、Lua が `base .. counter`（例 `会話_朝1`）を作る。`finalize` が `register_global_raw` でそのまま登録 | `scope_gen.rs:120-136`、`scene.lua:129`、`finalize.rs:174` |
| ローカルシーン | 関数名 `{sanitize(name)}_{counter}`、検索キー `:{親の登録名}:{関数名}` | `scope_gen.rs:240-242`、`scene_table.rs:139-147` |
| アクター単語 | `:__actor_{sanitize(actor)}__:{単語名}`（トランスパイル時・確定時とも） | `word_registry.rs:83-84`、`transpiler.rs:239`、`finalize.rs:187` |
| ローカル単語 | `:{sanitize(scene_name)}:{単語名}`（scene_name は登録名なので冪等） | `word_registry.rs:64-65` |
| 単語名（キー）そのもの | 置き換えない（登録・検索とも書いたまま） | 同上 |

- 文法上、シーン名・アクター名は識別子（`XID_START`/`_` + `XID_CONTINUE*`、`grammar.pest:11-22`）。置き換え対象になり得るのは「XID_CONTINUE だが英数字でない」文字（中黒類・Pc 類・非英字の結合文字・ZWJ/ZWNJ 等）。
- 動的ターゲット（`＞＄x` 等）・Lua API（`SCENE.search`・`WORD.create_actor`）・pasta.toml の `[actor."名前"]` は任意の文字列を渡せる。

### 2.2 検索の経路（すべて `SearchContext::search_scene` に集まる）

| 経路 | 呼び出し | 渡す `name` | 第 2 引数 |
| ---- | -------- | ----------- | --------- |
| Call 2・5 段、`act:find_scene` | `act.lua:295` → `SCENE.search(key, scene_name)` | 作者が書いた名前 / 動的値 | 実行中の登録名 / nil |
| Lua API | `SCENE.search`・`SEARCH:search_scene` | 任意 | 登録名 / nil |
| SHIORI イベント | `boot.lua:19`・`init.lua:166`・`virtual_dispatcher.lua:120` → `SCENE.co_exec` | イベント ID（`OnBoot` 等） | nil |
| 選択肢 | `choice_select.lua:61-62` | `\q[...,ID]` の ID（作者が書いたジャンプ先名） | 直前の登録名 / nil |
| キック（デバッガ） | `kick.lua:143,149` | **登録名**（`会話_朝1`、ローカルは関数名 `挨拶_1`） | 登録名 |
| 辞書確定前 | 同上（トランスパイル時レジストリの `SearchContext`） | 同上 | 同上 |

- `search_scene` は第 2 引数ありならローカルのみ（`resolve_scene_id_unified(parent, name)` → `collect_scene_candidates` の `:{parent}:{prefix}` 前方一致）、なしならグローバルのみ（`:` で始まるキーを除外）。
- 「見つからない」警告は Lua 側（`act.lua:513` 付近）が **元の key** で出す。Rust 側で照合規則を通しても警告の表示名は変わらない（要件 3.5 は自動的に満たされる）。
- Call 1・3・4 段（`SCENE[key]`・act メソッド・`GLOBAL[key]`）とアクター A1 段（`actor[key]`）は Lua テーブルの完全一致で、この入口を通らない（要件の Out of scope と整合）。

### 2.3 アクター単語の検索

- `actor.lua:141-144`: `SEARCH:search_word(key, "__actor_" .. self.actor.name .. "__")`。
- `SearchContext::search_word(name, scope)` → `WordTable::search_word(scope, name)` → `:{scope}:{name}` 前方一致。スコープは置き換えない。
- `self.actor.name` は `％名前` の識別子、または pasta.toml `[actor."名前"]` のキー（任意文字列）、Lua の `ACTOR` で作った名前。
- Lua 側には照合規則の実装が無い（`pasta_scripts/pasta/*.lua` に該当する `gsub` は無い）。

### 2.4 既存テストの配置

- `pasta_core`: `scene_registry.rs`・`word_registry.rs` 内の `test_sanitize_name`（規則そのもの）、`test_register_local_sanitizes_names`。
- `pasta_lua` 単体: `src/search/context.rs` 末尾の `search_scene`/`search_word` テスト（`test_search_scene_global_excludes_local_keys` は `":メイン_1:選択肢"` → None。照合規則を通すと `_メイン_1_選択肢` になり、引き続き None で通る）。
- `pasta_lua` 統合: `tests/search/scene_search_test.rs`、`tests/transpiler/actor_word_dictionary_test.rs`、`tests/lua_specs/actor_word_test.lua`・`actor_module_test.lua`、`tests/scene_identity_index_test.rs`（`会話·A`/`会話_A` の衝突の特性化。採番のみで検索は見ていない）。
- **記号を含む名前で検索・Call・アクター単語参照する既存テストは無い**（`·`・`＿` の grep で該当は採番の特性化のみ）。要件 7.1〜7.4 はすべて新規。

### 2.5 マニュアル（`book/src/`）の該当箇所

| ファイル | 行 | 現状 | 要件 |
| -------- | -- | ---- | ---- |
| `grammar/call-jump.md` | 33 | 「書いた名前をそのまま検索キーにする」 | 6.1・6.3 |
| `grammar/actor-dictionary.md` | 28, 57-69 | 照合の記述なし | 6.2 |
| `grammar/markers.md` | 77-82 | 識別子の定義。照合への言及なし（参照先として使える） | 6.1（任意） |
| `lua/modules/pasta-search.md` | 54, 60 | `name` は「検索するシーン名（前方一致）」、`global_name` は「シーン名＋番号」 | 6.4 |
| `lua/script-api.md` | 393-398 | `WORD.create_actor`・`scene_name` の説明 | 6.2（任意） |
| `internals/internal-modules.md` | 201, 235 | A2 のスコープ組み立て、基本名がサニタイズ済み | 6.5 |
| `internals/registry-search.md` | 33, 115, 180, 226 | 226 が「検索キーとして渡す名前はサニタイズされない」 | 6.3・6.5 |
| `internals/transpiler.md` | 200, 300 | 規則の説明・共有の記述 | 6.5（整合確認） |
| `internals/debug.md` | 348, 376 | キック番兵の非衝突の根拠（「グローバル名は英数字と `_` だけ」） | 変更不要（根拠は保たれる） |

- 生成スキル: `book/tools/gen-skill-refs.mjs` が `grammar/call-jump.md`・`grammar/actor-dictionary.md`（→ pasta-ghost-authoring）、`lua/modules/pasta-search.md`・`internals/internal-modules.md`（→ pasta-lua-coding）を写す。`--check` と `book/tools/link-check.mjs` が鮮度とリンクを検査する（要件 6.6）。`internals/registry-search.md` はスキルに写されない。

## 3. 要件と資産の対応（ギャップ）

| 要件 | 既存資産 | ギャップ | 種別 |
| ---- | -------- | -------- | ---- |
| 1.1〜1.5 グローバル Call・検索 | `search_scene`・`collect_scene_candidates`・照合規則 | 検索の `name` が照合規則を通らない | Missing |
| 1.4 前方一致 | 前方一致・シャッフルの既存実装 | 照合後の名前で前方一致するだけ。キャッシュキーも照合後になる（`会話・朝` と `会話_朝` が同じ順次消費の状態を共有する） | Constraint（挙動の帰結。要件 5.1 と整合） |
| 2.1〜2.3 ローカル | 同上（`:{parent}:{prefix}`） | 同上。第 2 引数は変えない | Missing |
| 3.1 規則の共有 | `SceneRegistry::sanitize_name` は登録・生成で共有済み | 検索側が呼んでいない。Lua 側に実装なし | Missing |
| 3.2 登録名の冪等性 | 照合規則は英数字と `_` だけの文字列に対し恒等 | 追加作業なし。テストで固定する | — |
| 3.3 第 2 引数は変えない | — | 実装上の注意のみ | Constraint |
| 3.5 警告の表示名 | `act.lua` が元の key で警告 | 追加作業なし | — |
| 3.6 `:` 始まりの除外 | `collect_scene_candidates` の `:` 除外 | 照合で `:` が `_` になるため、より強く保たれる | — |
| 3.7 辞書確定前 | 確定前も `SearchContext::search_scene` を通る | 入口の修正で自動的に満たす | — |
| 4.1〜4.4 アクター | `register_actor`・`actor.lua` A2 | 検索側のスコープが元の名前 | Missing（方向の選択が必要） |
| 5.1 シーンの衝突 | 照合用の名前ごとのカウンタ（`increment_counter`）で同名扱い済み | 現行どおりなら追加作業なし。警告するなら検出・出力の場所が要る | Unknown（方針） |
| 5.2 アクターの衝突 | 登録時に同じキーへ写る | 方向 B（登録で置き換えない）を選ぶと衝突自体が消え、要件 5.2 の文言が変わる | Unknown（方針） |
| 5.3・6.x マニュアル | 2.5 の表 | 記述の書き換え | Missing |
| 7.x テスト | 2.4 | 記号を含む名前の検索テストが無い | Missing |

## 4. 実装アプローチの選択肢

### 4.1 シーン名（要件 1〜3）

**案 S1: `SearchContext::search_scene` の入口で `name` を照合規則に通す（棚卸の推奨）**
- 変更: `context.rs` の `search_scene` 冒頭に 1 行（`let name = SceneRegistry::sanitize_name(name);` 相当）。`SceneRegistry` はすでに import 済み。
- ✅ 全呼び出し元（2.2 の表）に 1 か所で効く。並走条件の範囲（`crates/pasta_lua/src/search/`）。
- ✅ 第 2 引数に触れないので `scene-identity-format` の区切りを壊さない（区切りが `_` なら照合規則でも残る）。
- ❌ Lua から `search_scene` を直接呼ぶ利用者にも効く（意図どおりだが、マニュアルに書く必要がある＝要件 6.4）。

**案 S2: `SceneTable::collect_scene_candidates`（`pasta_core`）の `prefix` を照合規則に通す**
- ✅ `pasta_core` 内で完結し、`resolve_scene_id`（実行時は未使用）以外の利用者にも効く。
- ❌ `pasta_core` の検索表が「作者の名前」という上位の概念を知ることになる。`search-selector-indices` が後で同じファイル群を触る。
- ❌ `resolve_scene_id`・`find_scene` は別経路で、一貫させるには複数箇所の修正になる。

**案 S3: Lua 側（`scene.lua` の `SCENE.search`）で置き換える**
- ❌ `scene.lua` は並走条件の範囲外（Wave 2 の `scene-identity-format` が持つ）。規則が Rust と Lua に二重化し、要件 3.1 に反する。

### 4.2 アクター名（要件 4）

**案 A1: 検索側に照合を足す — `@pasta_search` にアクター単語検索の入口を足す**
- 例: `SEARCH:search_actor_word(key, actor_name)` を `SearchContext` に足し、Rust 側で `__actor_{sanitize(actor_name)}__` を組み立てる。`actor.lua:142-143` はその呼び出しに置き換える（`PROXY_IMPL` の呼び出し規約は不変）。
- ✅ 規則は Rust の 1 か所のまま（要件 3.1）。キー構文と衝突する文字（`:` 等）は登録・検索とも `_` になり安全。
- ❌ `@pasta_search` の公開メソッドが 1 つ増える → マニュアル（`pasta-search.md`）に書く必要。内部専用にするなら、その扱いをマニュアルの方針に照らして決める必要がある。
- 衝突（`さくら・改`/`さくら_改`）は現行どおり同じ辞書を共有（要件 5.2 の前提どおり）。

**案 A2: 検索側に照合を足す — 照合規則そのものを Lua に公開する**
- 例: `SEARCH:normalize_name(name)`（名前は仮）を公開し、`actor.lua` がスコープ組み立て前に呼ぶ。
- ✅ 汎用。後続 spec（`scene-identity-format` 等）が Lua 側で照合用の名前を必要としたときにも使える。
- ❌ 公開 API が増える点は A1 と同じ。呼び出し側が規則の適用を忘れる余地が残る（A1 より食い違いの再発防止が弱い）。

**案 A3: 登録側の置き換えをやめる — `register_actor` でアクター名を書いたまま使う**
- 変更: `word_registry.rs:83` の 1 行。`actor.lua` は無変更。
- ✅ 最小の差分。アクター名の衝突が消える（`さくら・改` と `さくら_改` を区別できる）。
- ❌ アクター名に `:` や `__:` を含む場合、キー `:__actor_{name}__:{word}` の区切りと紛れる。例: アクター `a__:b` の単語 `x` と、アクター `a` の単語 `b__:x` は、どちらもキー `:__actor_a__:b__:x` になる（pasta.toml / `WORD.create_actor` 経由で任意文字列を渡せるため、病的な名前なら理論上は起こる）。
- ❌ 「登録と検索が同じ規則を共有する」というより「アクターだけ規則を使わない」形になり、シーンとの対称性が崩れる。要件 5.2 の文言を書き換える必要がある。

**案 A4: `search_word` のスコープ引数が `__actor_` で始まるときだけ照合する**
- ❌ brief の「スコープ引数には適用しない」に反し、キー形式の知識が検索入口に漏れる。非推奨（参考として記録）。

### 4.3 衝突の扱い（要件 5）

- **案 C1: 現行の挙動を保ち、マニュアルに書く（要件の前提）**。追加の実装なし。`scene_identity_index_test.rs` の特性化テストとも整合する。
- **案 C2: 読み込み時に警告を出す**。元の名前（衝突前）を知っているのはトランスパイル時（`SceneRegistry::register_global` は元の名前を受け取る）だけで、確定時（`register_global_raw`）は照合後の名前しか持たない。警告をどこで出すか（`pasta_core` にはログ出力の仕組みが無い可能性。トランスパイラ・ローダーは並走条件の範囲外）が **Research Needed**。メモリの方針「作成ツールは問題があれば止める」は配布物作成ツール（pasta_check）向けで、ランタイムの照合とは別論点。
- **案 C3: `pasta_check` などの検査ツールで警告する**。本仕様の範囲外として後続へ回す選択肢。

### 4.4 規則の置き場所（要件 3.1 の「1 か所」）

- 現状でも規則の実体は `SceneRegistry::sanitize_name` 1 つ（`WordDefRegistry::sanitize_name` は委譲）。**関数を移動・改名する必然性は無い**。移動すると `scope_gen.rs`・`transpiler.rs`（並走条件の範囲外）の呼び出しも変わる。
- 名前が「Scene」に偏っている点を整えたい場合は、`pasta_core::registry` に自由関数を足して `SceneRegistry::sanitize_name` から委譲する形なら範囲内で済む（任意）。

## 5. 工数とリスク

- **工数: S（1〜3 日）** — シーンは入口 1 行、アクターは案により 1〜十数行。主な作業はテスト（E2E フィクスチャ 1 本程度）とマニュアル 6〜8 ファイルの書き換え・スキル再生成。
- **リスク: Low** — 既存の規則と経路を使うだけで、照合規則は登録名に対して冪等なため既存挙動は変わらない。不確実性はアクターの方向（公開 API を増やすか）と衝突警告の有無という方針判断に限られる。

## 6. 設計フェーズへの推奨と持ち越し事項

### 推奨（判断はディスカッションで）

- シーン: **案 S1**（入口 1 か所、第 1 引数のみ）。
- アクター: **案 A1**（規則を Rust の 1 か所に保ち、キー構文の安全性を維持）。公開メソッドを増やすことのマニュアル上の扱いを決める。差分最小を優先するなら A3 だが、キー構文の衝突と要件 5.2 の書き換えを受け入れる必要がある。
- 衝突: **案 C1**（現行維持＋マニュアルに明記）。警告は需要が出たら別 spec。
- 規則の置き場所: 移動しない（4.4）。

### Research Needed

1. 選択肢 ID（`\q[title,ID]` の ID）が作者の書いたジャンプ先名のまま SHIORI の Reference に返ること、および `・` 等を含む ID がさくらスクリプト上で安全に往復することの確認（案 S1 で `＠？選択・A「…」` も解決するようになる副次効果の検証）。
2. 衝突警告を採る場合の出力場所（`pasta_core` のログ手段の有無、並走条件の範囲内で出せるか）。
3. 生成コード以外から `PASTA.create_scene` を元の名前（置き換え前）で呼ぶ利用があるか。現在は内部 API（マニュアルでは `internals/` のみに記載）だが、もし使われていれば案 S1 の後はその名前で検索できなくなる（現行は検索できる）。
4. 案 A1/A2 で増やすメソッドを公開 API としてマニュアルに載せるか、内部用として扱うか（マニュアルが API の唯一の権威という制約との整合）。
5. SHIORI 応答（204 にならないこと、要件 1.5）を固定するテストの置き場所（`pasta_lua/tests/shiori/` の E2E か `pasta_shiori` か）。

## 7. 要件ディスカッションでの決定

- **議題 1（アクター名の直し方）**: 検索側でも同じ照合規則を通す方向（案 A1/A2）に決定。案 A3（登録時の置き換えをやめる）は不採用。要件 4.5・4.6 を追加。A1 と A2 のどちらの形にするかは設計フェーズで決める。
- **議題 2（衝突の扱い）**: 案 C1（現行の挙動を保ち、マニュアルに明記）に決定。読み込み時の警告（C2）は採らない。検出・警告は需要が出たら別 spec で検査ツールに持たせる（C3）。要件 5.1・5.2 の前提注記を外し、範囲外に追記。Research Needed 2 は不要になった。
- **議題 3（増えるメソッドの扱い）**: 公開 API として `lua/modules/pasta-search.md` に載せることに決定（内部用にしない）。要件 6.6 を追加（旧 6.6 は 6.7 へ）。メソッドの形は設計フェーズで `search_word` との一貫性を見て決める。Research Needed 4 は解決。**→ 設計ディスカッション（9 章）で置き換えた。メソッドは足さないので、このとき足した要件 6.6 は削除し、番号を元に戻した（生成スキルの再生成が 6.6）。**
- **議題 4（選択肢のジャンプ先）**: 要件 2.4 として明記し、要件 7.2 のテストで固定することに決定。選択肢のジャンプ先は `act:choice("…")` の文字列リテラルで Lua の識別子にはならず、`escape_choice` は `\`・`]`・`,` だけを逃がすため `・` 等は `\q[...]` をそのまま往復する（Research Needed 1 は解決）。選択肢の自動ルーティングの仕組み自体は引き続き範囲外。

## 8. 設計フェーズの調査と決定（2026-10-04）

- **Feature**: `scene-search-key-normalization`
- **Discovery Scope**: Extension（既存システムの拡張。軽量ディスカバリ）
- **Key Findings**:
  - ギャップ分析の行番号と記述はコードと一致した（`context.rs:68`・`scene_registry.rs:241`・`word_registry.rs:83-84`・`actor.lua:142-143`・`choice_select.lua:61-62`・`act.lua:513`）。文法ファイルの場所だけ補足が要る（`crates/pasta_dsl/src/parser/grammar.pest`）。
  - `・`（U+30FB）・`·`（U+00B7）・`＿`（U+FF3F）は pest 2.8.6／2.9.2 の `XID_CONTINUE` に含まれ、`XID_START` には含まれない。`＊会話・朝`・`％さくら・改`・`・選択・A` は識別子として読める（先頭には置けない）。
  - スコープ名 `__actor_…__` の形は、登録（`word_registry.rs`）と検索（`actor.lua`）が別々に書いている。形の食い違いはテストで検出する（9 章の決定で、形の共有はしないことにした）。
  - `@pasta_search` を表で差し替える既存の Lua テストがあり、A2 段が呼ぶメソッドを変えると追従が要る。メソッドを変えない案（`search_word` のスコープを照合する）なら追従は要らない。
  - 単語の登録側（`register_local`・`register_actor`）はスコープをすでに照合規則に通している。検索側（`search_word`）だけが通していない。

### 8.1 調査ログ

#### 記号が識別子として読めるか
- **Context**: 要件の例（`＊会話・朝`）が構文エラーにならないことの確認。`・` はローカルシーンのマーカーでもある。
- **Sources Consulted**: `crates/pasta_dsl/src/parser/grammar.pest`（`id`・`xidn`・`local_marker`）、pest 2.8.6／2.9.2 の `src/unicode/binary.rs`（`XID_CONTINUE`・`XID_START` の表を復号して確認）。
- **Findings**: U+30FB・U+00B7・U+FF3F・U+FF65・U+203F は `XID_CONTINUE` に含まれる。`id` は最長一致なので `会話・朝` は 1 つの識別子になる。行頭の `・` はマーカーとして先に読まれる（`XID_START` ではないので識別子の先頭にならない）。
- **Implications**: 文法は変えない。テストは `.pasta` のソースから書ける。

#### アクター単語の経路
- **Context**: 検索側のどこでアクター名を照合するか。
- **Sources Consulted**: `actor.lua:126-148`、`word_registry.rs:81-87`、`word_table.rs`（`collect_word_candidates`）、`runtime/finalize.rs:181-197`、`code_gen/scope_gen.rs:28-70`、`transpiler.rs:229-243`。
- **Findings**: 登録は確定時・トランスパイル時とも `register_actor`（元のアクター名を受け取り、内部で照合）。検索は `:{scope}:{key}` の前方一致で、スコープは渡された文字列のまま。`a__:b`＋`x` は `:__actor_a___b__:x`、`a`＋`b__:x` は `:__actor_a__:b__:x` になり、区別できる（要件 4.6）。
- **Implications**: 検索側でもスコープ名を `sanitize_name` に通せばよい。`search_word` の入口でスコープごと通せば、`actor.lua` が作る `__actor_さくら・改__` は `__actor_さくら_改__` になり、登録キーと一致する。`a__:b`＋`x` と `a`＋`b__:x` も区別できたままである（単語キーは置き換えないため）。

#### 既存テストへの影響
- **Context**: `actor.lua` の A2 段が呼ぶメソッドを変えたときの影響。
- **Sources Consulted**: `crates/pasta_lua/tests/lua_specs/proxy_find_handler_test.lua:74-94`、`act_dynamic_ref_test.lua:161-170,468`、`act_find_act_handler_test.lua:28`、`global_fallback_integration_test.lua:94`、`crates/pasta_lua/scriptlibs/lua_test/mocks.lua`。
- **Findings**: 前 2 つは A2 段が `search_word(key, "__actor_…__")` を呼ぶことを直接確かめている。`lua_test.mocks` の既定の代役は `__index` でどのメソッド名にも `nil` を返す関数を返すので影響を受けない。A2 段が新しいメソッドを呼ぶ形にすると、表で作った代役（`search_scene`・`search_word` だけを持つ）は A2 段に届いたときにエラーになる。
- **Implications**: メソッドを足す案では代役の追従が要る。9 章の決定（`search_word` のスコープを照合する）では `actor.lua` が呼ぶメソッドも引数も変わらないので、これらのテストは書き換えない。

#### SHIORI 応答テストの置き場所
- **Context**: 要件 1.5・2.4（204 にならないこと）をどこで確かめるか（旧 Research Needed 5）。
- **Sources Consulted**: `crates/pasta_lua/tests/common/mod.rs`（`create_temp_with_pasta`）、`crates/pasta_lua/tests/runtime/syntax_test.rs`（`EVENT.fire`）、`crates/pasta_lua/tests/shiori/event_dispatch_test.rs`、`crates/pasta_shiori/tests/common/`（`ShioriTestEnv`・`copy_fixture_into`）、`crates/pasta_shiori/tests/support/scripts/`。
- **Findings**: `pasta_shiori` のテスト環境は `tests/support/scripts` の写しを読み込む。この写しは本体（`crates/pasta_lua/pasta_scripts`）と内容がずれており、`pasta/shiori/event/`（選択肢のルーティングを含む）・`res.lua` などが無い。`pasta_lua` 側は実物の `pasta_scripts` を一時ゴーストへ写して `PastaLoader::load` できる。
- **Implications**: 結合テストは `pasta_lua/tests/` に新しいファイルとして置く。応答は `EVENT.fire` が返す文字列で確かめる。

### 8.2 アーキテクチャ案の評価

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 入口での正規化（採用） | `SearchContext` の入口で作者の名前を照合する | 全経路に 1 か所で効く。検索表は照合済みキーだけを扱う | Lua から直接呼ぶ利用者にも効く（マニュアルに書く） | シーンは案 S1。単語は `search_word` のスコープを照合（9 章） |
| 検索表での正規化 | `SceneTable::collect_scene_candidates` で照合する | `pasta_core` で完結 | 検索表が「作者の名前」を知る。経路が複数ある | 案 S2。不採用 |
| Lua 側での正規化 | `scene.lua`・`actor.lua` に規則を書く | Rust を変えない | 規則が二重になる（要件 3.1 に反する）。`scene.lua` は範囲外 | 案 S3。不採用 |

### 8.3 設計判断

#### Decision: シーン検索の修正場所
- **Context**: 要件 1〜3。持ち越し項目。
- **Alternatives Considered**: 案 S1（`search_scene` の入口）／案 S2（検索表）／案 S3（Lua 側）。
- **Selected Approach**: `SearchContext::search_scene` の冒頭で第 1 引数だけを `SceneRegistry::sanitize_name` に通す。
- **Rationale**: すべての呼び出し元がここを通る。並走条件の範囲内。第 2 引数に触れない。
- **Trade-offs**: 順次消費のキャッシュキーが照合後の名前になる（`会話・朝` と `会話_朝` が同じ記録を進める）。要件 5.1 と整合する。
- **Follow-up**: 登録名を渡したときの結果が変わらないことを単体テストで固定する。

#### Decision: 照合規則の関数は動かさない
- **Context**: 「規則を 1 か所にまとめる」の解釈。持ち越し項目。
- **Alternatives Considered**: `pasta_core::registry` に自由関数を足して委譲／`SceneRegistry::sanitize_name` のまま。
- **Selected Approach**: 動かさない。改名もしない。
- **Rationale**: 実体はすでに 1 つである。動かすと `scope_gen.rs`・`transpiler.rs`（並走条件の範囲外）の呼び出しも変わる。
- **Trade-offs**: 名前が「Scene」に偏ったままになる。
- **Follow-up**: なし。

#### Decision: アクター単語の直し方（設計ディスカッションで決定）
- **Context**: 要件 4.5。持ち越し項目（議題 1・3）。
- **Alternatives Considered**:
  1. 案 A1 — `@pasta_search` に `search_actor_word(name, actor_name)` を足し、`actor.lua` の A2 段がそれを呼ぶ。スコープ名の形は `pasta_core` の関数（`actor_scope`）に切り出して登録と共有する。
  2. 案 A2 — 照合規則そのもの（`normalize_name`）を公開し、Lua 側でスコープ名を組み立てる。
  3. `search_word` の第 2 引数（スコープ）を Rust 側で照合規則に通す（公開 API・Lua の変更なし）。
- **Selected Approach**: 3 番目の案。`SearchContext::search_word`（`context.rs:165`）でスコープを `SceneRegistry::sanitize_name` に通してから検索表へ渡す。
- **Rationale**: 登録側がスコープをすでに置き換えているので対称になる。公開 API を増やさない。Lua スクリプトを変えない。既存のテストを書き換えずに済む（要件 7.5 を文字どおり満たす）。ローカル単語のスコープ（登録名）は英数字と `_` だけなので、照合しても変わらない。
- **Trade-offs**: スコープ名の形 `__actor_…__` は登録（Rust）と検索（Lua）が別々に書いたままになる。登録名の区切りが照合規則で消える文字になると、ローカル単語の検索が壊れる（設計の Revalidation Triggers に記載）。設計の初版は案 A1 だったが、設計ディスカッションで置き換えた（9 章）。
- **Follow-up**: 形の食い違いは、往復テストと、実物の `actor.lua` を通す結合テストで検出する。

#### Decision: SHIORI 応答テストの置き場所
- **Context**: 要件 1.5・2.4・7.2。持ち越し項目。
- **Alternatives Considered**: `pasta_lua/tests/shiori/` に足す／`pasta_shiori` の `ShioriTestEnv`／`pasta_lua/tests/` の新しいファイル。
- **Selected Approach**: `crates/pasta_lua/tests/symbol_name_search_test.rs` を新しく作り、`create_temp_with_pasta` → `PastaLoader::load` → `EVENT.fire` で確かめる。
- **Rationale**: 実物のランタイムスクリプトで選択肢のルーティングまで通せる。独立したファイルなので、同じ波で走るほかの spec と衝突しない。
- **Trade-offs**: DLL の入口（`pasta_shiori`）までは通さない。
- **Follow-up**: `EVENT.fire` の水準で足りるかを設計ディスカッションで確認する（Open Question 4）。

### 8.4 統合（Synthesis）の結果

- **Generalization**: 要件 1〜4 は「検索の入口が照合規則を通らない」という 1 つの問題の変形である。シーンは `search_scene` の名前、アクターは `search_word` のスコープという、既存の入口 2 か所を同じ規則に通す。汎用の「名前の正規化 API」には広げない（要件が無い）。
- **Build vs. Adopt**: 新しい仕組みは作らず、既存の `SceneRegistry::sanitize_name` を検索の入口から呼ぶ。
- **Simplification**: 関数の移動・改名、検索表の変更、衝突の検出、Lua 側の規則の実装は行わない。メソッドの追加と Lua スクリプトの変更も行わない。変えるのは `context.rs` の 2 行（`search_scene` の名前と `search_word` のスコープの照合）だけである。

### 8.5 リスクと対策

- 記号を含む検索キー（SHIORI イベント ID の `sakura.recommendsites` など）が `_` の名前のシーンに一致するようになる — 変更前は一致しなかったので既存の挙動は壊れない。マニュアルには個別に書かない（設計ディスカッションで決定）。
- スコープ名の形 `__actor_…__` が登録と検索で別々に書かれている — 往復テストと、実物の `actor.lua` を通す結合テストで食い違いを検出する。
- 登録名の区切りが照合規則で消える文字になると、ローカル単語のスコープが検索時に変わってしまう — 設計の Revalidation Triggers に書き、`scene-identity-format` が区切りを照合規則で残る文字から選ぶ前提を保つ。
- `crates/pasta_shiori/tests/support/scripts/` の写しが古い — 本仕様では触らない。写しの `actor.lua` も `search_word` を呼ぶので、修正はそのまま効く。写しの同期は別の課題である。

### 8.6 References

- `crates/pasta_lua/src/search/context.rs` — 検索の入口
- `crates/pasta_core/src/registry/scene_registry.rs`・`word_registry.rs` — 照合規則と登録キー
- `crates/pasta_lua/pasta_scripts/pasta/actor.lua` — A2 段（変更しない）
- `crates/pasta_dsl/src/parser/grammar.pest` — 識別子の定義
- `book/tools/gen-skill-refs.mjs`・`book/tools/link-check.mjs` — 生成スキルの再生成と検査

## 9. 設計ディスカッションでの決定

- **議題 1（アクター単語の直し方）**: `SearchContext::search_word` の第 2 引数（スコープ）を照合規則に通す案に決定。`search_actor_word` と `WordDefRegistry::actor_scope` は足さない。`actor.lua` と既存の Lua テストは変えない。
  - この決定は、要件ディスカッションの議題 3（増えるメソッドを公開 API として載せる）を置き換える。メソッドが増えないので、要件 6.6（足したメソッドを公開 API として書く）を削除し、生成スキルの再生成を 6.6 に戻した。代わりに要件 6.4 に、`search_word` のスコープが照合用の名前に揃えてから照合されることを書くよう足した。
  - brief の「スコープ引数には適用しない」は、`search_word` についてはこの決定で置き換える（brief.md 自体は書き換えない）。`search_scene` の第 2 引数は今までどおり照合しない（要件 3.3。シーン検索のことだと分かるよう文言を直した）。
  - 理由: 公開 API が増えない。Lua スクリプトを変えない。既存のテストを書き換えずに済み、要件 7.5 を文字どおり満たす。
  - 副次効果: `crates/pasta_shiori/tests/support/scripts/` にある `actor.lua` の古い写しも `search_word` を呼ぶので、修正がそのまま効く。
- **スコープ名を作る公開関数**: 足さない（上の決定に含まれる）。
- **記号を含む SHIORI イベント ID の副次効果**: 意図した挙動の帰結として受け入れる。マニュアルには個別に書かない。
- **マニュアルの用語**: 利用者向けの章は「照合用の名前」、内部構造の章は「サニタイズ」を使い、内部構造の章で対応づける。
- **未決**: SHIORI 応答のテストの水準（`EVENT.fire` の水準で足りるか。設計の Open Question 4）。
- **議題 2（SHIORI 応答テストの水準）**: `pasta_lua` の結合テストで `EVENT.fire` の応答文字列を確かめる水準で足りると決定。直すのは検索の入口だけで、`EVENT.fire` から DLL の入口までの経路は変えないため。`crates/pasta_shiori/tests/support/scripts/` の写しを本体に合わせる作業は別の作業にする。
