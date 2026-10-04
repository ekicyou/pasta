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
