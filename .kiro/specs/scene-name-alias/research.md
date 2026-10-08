# ギャップ分析: scene-name-alias

- 実施日: 2026-10-08
- 基準: worktree `claude/ontalk-scene-name-a0749f`（`b8efe25b`）
- 入力: `requirements.md`（Requirement 1〜10）、`brief.md`、steering（product / tech / structure / grammar / roadmap）

## 分析サマリー

- **別名の置き場所は `sanitize_name` そのものではない**。`SceneRegistry::sanitize_name` は静的関数で、グローバルシーン名のほかに単語のモジュール名・アクター名（`WordDefRegistry::sanitize_name` が委譲）・ローカルシーンの通し番号（`context.rs`）・単語検索の範囲（`search_word`）にも使われる。ここへ別名を入れるとアクター名やローカルシーン名まで置き換わる。別名は「グローバルシーン名の経路だけ」に差す別の関数（または値）として、`sanitize_name` の前段に置く必要がある。
- **登録はトランスパイル時、検索は実行時で、別名表を 2 か所へ渡す経路が無い**。宣言側は `scope_gen.rs` が `PASTA.create_scene("基本名")` を生成した時点で名前が決まり、実行時の `finalize_scene` はそれをそのまま登録表へ写す。検索側は `SearchContext::search_scene`（実行時）が照合用の名前に揃える。現在 `LuaTranspiler`（`TranspilerConfig`）にも `finalize_scene_impl`／`search::register` にも設定の入口は無い。
- **トランスパイルのキャッシュが pasta.toml の変更を検出しない**（Requirement 6 の最大のギャップ）。`CacheManager::needs_transpile` は `.pasta` とキャッシュ `.lua` の更新時刻だけを比べ、キャッシュ全体の破棄は pasta_lua のバージョン変更時だけである。別名表を変えても `.pasta` が古いままなら、宣言側は古い表で生成された `.lua` を使い、検索側だけ新しい表になる。
- **既存テストへの波及が大きい**。`PastaLoader` 経由で `＊会話` を読み込み登録名 `会話_1` などを検証するテスト（`scene_identity_index_test.rs` ほか）は、既定の別名で登録名が `OnTalk_N` に変わる。既定をどの層に置くか（ローダー層だけか、トランスパイラの既定にも入れるか）で影響範囲が大きく変わる。
- **推奨は案 C（ハイブリッド）**: `pasta_core` に「グローバルシーン名の別名表」の小さな型を新設し、トランスパイル時（宣言・単語のモジュール名・ソースマップの突合キー）と実行時（グローバル検索だけ）の両方で同じ値を使う。既定表はローダー層（pasta.toml の読み込み）で決め、キャッシュの有効性判定に別名表を含める。

## 1. 現状調査

### 1.1 シーン名の正規化と登録

| 資産 | 役割 | 別名との関係 |
| ---- | ---- | ------------ |
| `crates/pasta_core/src/registry/scene_registry.rs` `sanitize_name`（静的） | 英数字と `_` 以外を `_` に置換。照合用の名前の唯一の規則 | 前段に別名を置く対象。ただし共有利用者が多い（1.2） |
| 同 `register_global` / `increment_counter` / `registered_name` | 照合用の名前ごとの通し番号、登録名 `{照合用の名前}_{番号}` | 宣言側の別名適用後の名前で呼ばれる必要がある |
| `crates/pasta_lua/src/context.rs` `register_global_scene` | `scene.name` をそのまま `register_global` へ | 別名適用の入口候補 |
| `crates/pasta_lua/src/transpiler.rs` `process_global_scene`（l.197〜） | `registered_name(&scene.name, counter)` で単語のモジュール名を作る | 同上（置き換えないと単語の登録先がずれる） |
| `crates/pasta_lua/src/code_gen/scope_gen.rs` `generate_global_scene`（l.113〜） | `sanitize_name(&scene.name)` を基本名にして `PASTA.create_scene("…")` を出力し、ソースマップの突合キー `G:{base}#{counter}`・`L:{parent_base}#…` を記録 | 宣言側の実質的な登録点。突合キーも同じ基本名を使うため、ここで置き換えればデバッグの突合は一致する |
| `crates/pasta_lua/pasta_scripts/pasta/scene.lua` `SCENE.create_scene` | 受け取った基本名に実行時の通し番号を付けて `STORE.scenes` に登録 | 生成コードの基本名をそのまま使う（Lua 側に正規化は無い） |
| `crates/pasta_lua/src/runtime/finalize.rs` `finalize_scene_impl` | `STORE.scenes` から `SceneRegistry` を作り直し `@pasta_search` を登録 | 実行時の登録表の正本。設定を受け取る引数は無い（`lua` だけ） |

- 単独の `＊` 行は、パーサーが直前の名前を `GlobalSceneScope::continuation(name)` で受け継ぐ（`pasta_dsl/src/parser/ast/scene.rs`）。トランスパイラから見れば通常の宣言と同じ `scene.name` を持つため、宣言側で置き換えれば Requirement 4.2 は自然に満たされる。

### 1.2 `sanitize_name` の共有利用者（別名を入れてはいけない経路）

| 呼び出し元 | 対象 | 別名を適用すべきか |
| ---------- | ---- | ------------------ |
| `word_registry.rs` l.64・l.83（`WordDefRegistry::sanitize_name` 経由） | 単語のモジュール名・アクター名 | しない（Req 4.7） |
| `pasta_lua/src/context.rs` l.135 `local_scene_counters` | ローカルシーン名 | しない（Req 4.7・OQ-5） |
| `search/context.rs` l.177 `search_word` | 単語検索の範囲（登録名・アクター範囲） | しない |
| `search/context.rs` l.79 `search_scene` | グローバル検索とローカル検索の両方の `name` | **グローバル検索（`global_scene_name = None`）のときだけ**する |
| `scope_gen.rs` l.120 | グローバルシーンの基本名 | する |
| `debug/source_map/scene_join.rs`・`playscene.rs`・`dap/decode.rs`・`kick.lua` | 登録名の分解・突合・キック | しない（登録名を受け取る入口。Req 4.8）。宣言側で置き換えれば追加変更は不要の見込み |

### 1.3 検索の経路

- `ACT_IMPL.find_act_handler`（`pasta_scripts/pasta/act.lua` l.332〜）の 5 段: L1 シーン表の完全一致 → L2 ローカル辞書の前方一致（`SCENE.search(key, scene_name)`）→ L3 act メソッド → L4 `GLOBAL` 完全一致 → L5 グローバル前方一致（`SCENE.search(key, nil)`）。`act:call`・`SCENE.co_exec`（`act:find_scene` 経由）・SHIORI イベントのフォールバック・仮想ディスパッチャ（`virtual_dispatcher.lua` の `create_scene_thread` → `SCENE.co_exec`）はすべてここを通る。
- 選択肢の飛び先（`choice_select.lua`）は `SCENE.search(choice_id, scope) or SCENE.search(choice_id, nil)` を直接呼ぶ。
- **expr モード**（`＠名前（）` の関数呼び出し）も L2・L5 で `SCENE.search` を使うため、グローバル検索側に別名を入れると `＠会話（）` も OnTalk のシーンに解決されうる。要件の「グローバルシーンを名前で探す全ての入口」と整合するが、マニュアルで触れるかは設計で判断する（Research Needed R-5）。
- 検索結果の並び順の記録（`SceneCacheKey`）は検索キー（照合用の名前）ごとに持つ。置き換え後の名前で検索すれば、`＞会話` と仮想ディスパッチャの `OnTalk` が同じ記録を進める（Req 1.3 の「同じ候補の集まり」と整合）。
- **すべての入口が Rust の `SearchContext::search_scene` に集まる**。グローバル検索の分岐（`global_scene_name = None`）にだけ置き換えを入れれば、Req 4.3〜4.6 を 1 か所で満たせる。

### 1.4 設定（pasta.toml）

- `PastaConfig::load` → `parse` が `[loader]` を型付きで取り出し（型が合わなければ読み込みエラー）、その他のセクションは `custom_fields`（`toml::Table`）に残す。`[talk]` などは `get_custom_config`（`try_into().ok()`）で取り出すため、型が合わなければ黙ってセクション全体が既定になる（マニュアル「値の型が合わないとき」の 2 行目）。
- `custom_fields` はそのまま `@pasta_config` として Lua へ公開される（`runtime/module_registry.rs`）。`[scene.alias]` を追加すると `@pasta_config.scene.alias` に現れる。作者が既に独自の `[scene]` セクションを使っている場合は衝突しうる（Constraint）。
- 設定はトランスパイルより前（Phase 1）に読まれる（`loader/mod.rs`）。トランスパイラへ渡すこと自体は容易だが、`process.rs` の `process_incremental` は `LuaTranspiler::default()` を使っており、設定を受け取る引数が無い。

### 1.5 キャッシュ

- `loader/cache.rs`: `.cache_version`（pasta_lua のバージョン）が一致すればキャッシュを保持し、ファイルごとに `.pasta` と `.lua` の更新時刻を比べる。pasta.toml の内容・更新時刻は判定に入っていない。
- 別名表をトランスパイル時に効かせる設計では、キャッシュの有効性判定に別名表を含めることが必須（Req 6.1〜6.3）。

### 1.6 デバッグ（ソースマップ・キック）

- 突合（`scene_join.rs`）は、ビルド側の `G:{base}#{k}`（ファイル内の基本名ごとの出現順）と、実行時の登録名を `split_registered_name` で分けた（名前, 番号）のファイル内順位を照合する。宣言側の基本名を `scope_gen.rs` で置き換えれば、`create_scene` の基本名と突合キーの基本名は同じになり、`＊会話` と `＊OnTalk` の混在（Req 7.3）もファイル内の宣言順で正しく対応づく見込み。
- 逆に、置き換えを実行時（Lua の `create_scene`）だけで行う設計では、突合キーの基本名（`会話`）と実行時の登録名（`OnTalk_N`）が一致せず、ブレークポイントとカーソル位置のシーン再生が壊れる。
- キック（`kick.lua`）は登録名の完全一致で引くため、置き換えの影響を受けない（Req 4.8）。

### 1.7 LSP・pasta_check

- `pasta_lsp` は semantic tokens とパースエラーの診断だけを持ち、シーン名の定義ジャンプや未解決参照の診断は無い（`analysis/mod.rs`・`server.rs`）。`pasta_check` は `pasta_lua` のトランスパイラも `SceneRegistry` も使わない。
- したがって現時点では「LSP・pasta_check が別名表を読まない」ことによる実害は無い。マニュアルの注記（Req 9.4）は将来の機能に向けた予防的な記述になる。

### 1.8 マニュアルとテスト

- マニュアルの `＊会話`／`＞会話` 系の出現: `grammar/markers.md` 6、`grammar/call-jump.md` 5、`grammar/block-structure.md` 4、`lua/modules/pasta-search.md` 2、`grammar/action-line.md`・`actor-dictionary.md`・`words.md`・`lua/script-api.md`・`internals/internal-modules.md`・`internals/registry-search.md` 各 1。`会話・朝`／`会話_朝_1` は照合用の名前・登録名の説明の定番例になっている（`call-jump.md` l.137〜149、`pasta-search.md` l.63・72・118、`script-api.md` l.544、`internals/*`）。
- `call-jump.md` l.151 は「Call のほか、SHIORI イベントに対応するシーンや選択肢のジャンプ先を探すときも、同じ照合をする」と既に書いており、別名の規則はこの節に自然に乗る。
- **文法に `＞＞`（Jump）は無い**（`call-jump.md` は Call `＞` だけを定義）。brief の「`＞`・`＞＞` の Call/Jump」は Call だけと読んで要件を書いた。
- テスト: `＊会話` を含む fixture は `pasta_lua/tests/fixtures/` に 5 件（`sample.pasta`・`scene_identity_*.pasta`・loader の `with_custom_config/.../conversation.pasta`）と `symbol_name_search_test.rs`。`会話_1` などの登録名・`"会話"` を検証する箇所は約 290 行（`scene_table_candidate_tests.rs` 51、`scene_registry.rs` 32、`scene_identity_index_test.rs` 29、`kick_*_test.lua` 計 49、`scene_index_tests.rs` 25、`playscene_tests.rs` 17 など）。うち `PastaLoader` 経由で読み込むのは `scene_identity_index_test.rs`（10 か所）と `symbol_name_search_test.rs`。`sample.pasta` は `sample.generated.lua` との比較テストがある。
- SHIORI の E2E 基盤は `pasta_shiori/tests/fixtures/*`（ゴースト一式）と `ontalk_probe_test.rs`（OnBoot → OnSecondChange で OnTalk を発行させる手順）があり、`＊会話` だけのゴーストの E2E（Req 10.2）はこの形で書ける。

## 2. 要件と資産の対応表

| 要件 | 既存資産 | ギャップ | 分類 |
| ---- | -------- | -------- | ---- |
| 1.1〜1.3 既定の別名で OnTalk に | `scope_gen.rs`・`search/context.rs`・`virtual_dispatcher.lua` | 別名表の型・既定値・宣言側と検索側への適用が無い | Missing |
| 1.4〜1.5 `＊OnTalk` と発行条件の維持 | `virtual_dispatcher.lua` | 変更不要（`check_talk` の `"OnTalk"` 直書きのまま） | — |
| 2.1〜2.4 pasta.toml の別名表 | `PastaConfig`・`custom_fields` | セクションの定義・読み込み・置き換え（マージしない）・空の表の区別が無い | Missing |
| 2.5 型が合わない値 | `[loader]` の型付き読み込み／`get_custom_config` | どちらの規則に乗るかが未決。読み込みエラーにするなら `parse` に検証を足す | Missing / Unknown（OQ-2） |
| 2.6 有効な表のログ | `tracing` の info | 出力箇所が無い | Missing |
| 3.1〜3.5 完全一致・1 段・連鎖 | `sanitize_name` | 前段の置き換え関数が無い。連鎖の検出（値がキーに含まれるか）が無い | Missing |
| 3.6〜3.7 置き換え後の前方一致 | `SceneTable::resolve_scene_id_unified` | 変更不要（置き換え後の検索キーを渡すだけ） | — |
| 4.1〜4.2 宣言・単独 `＊` | `context.rs`・`transpiler.rs`・`scope_gen.rs` | 3 か所が `scene.name` を個別に使う。1 か所で置き換えてから渡す仕組みが必要 | Missing / Constraint |
| 4.3〜4.6 Call・選択肢・イベント・Lua API | `SearchContext::search_scene`（全入口が集まる） | グローバル検索の分岐に置き換えを入れる。`SearchContext` へ表を渡す経路（`finalize_scene_impl`・`search::register`）が無い | Missing |
| 4.7〜4.8 対象外の経路 | `sanitize_name` の共有利用者（1.2） | `sanitize_name` 自体は変えない設計にする必要 | Constraint |
| 5.1〜5.4・5.6 既存挙動の維持 | 既存テスト群 | 空の表で導入前と同じになることのテストが必要 | Missing（テスト） |
| 5.5 「会話」シーンの挙動の変化 | — | 互換性の変化。リリースノート／マニュアルの説明が必要 | Constraint |
| 6.1〜6.3 変更の反映 | `CacheManager` | キャッシュの有効性判定に別名表が入っていない | **Missing（重要）** |
| 7.1〜7.3 デバッグ | `scene_join.rs`・`scope_gen.rs` の突合キー | 宣言側の基本名を置き換えれば追加変更は不要の見込み。混在時の突合のテストが必要 | Unknown（R-3） |
| 8.1〜8.2 失敗表記・ログ | `act.lua` `ACT_IMPL.call`（`key` を出す）、`search/context.rs` | 失敗表記は現状で書いた名前が出る。ログに置き換え後の名前を添える仕組みが無い（置き換えは Rust 側で起き、Lua の警告側は知らない） | Missing / Unknown（R-4） |
| 8.3 登録名 | `SCENE.create_scene` | 宣言側で置き換えれば自然に `OnTalk_N` になる | — |
| 9.1〜9.7 マニュアル | `book/src/` 各章、`gen-skill-refs.mjs` | 規則・既定・互換性・LSP 注記・例題の整理が無い | Missing |
| 10.1〜10.5 検証 | `pasta_core`／`pasta_lua`／`pasta_shiori` のテスト基盤 | 新規テストと、既存テストの更新（1.8） | Missing |

## 3. 実装アプローチの選択肢

### 案 A: `sanitize_name` を拡張する（既存部品の拡張）

`SceneRegistry::sanitize_name` に別名表を渡す（またはグローバルな状態として持たせる）。

- 対象: `scene_registry.rs`、全呼び出し元。
- ✅ brief の文言（「`sanitize_name` の前段」）にいちばん素直。登録と検索の一致が構造的に保証される。
- ❌ `sanitize_name` は単語・アクター・ローカルシーン・単語検索の範囲でも使われ（1.2）、そのままでは Req 4.7 に反する。呼び出し元ごとに「別名あり／なし」を選ばせると、共有規則という長所が崩れる。
- ❌ 静的関数にグローバル状態を持たせると、1 プロセスで複数ゴーストを読み込む構成（`LoadDirGuard` が想定）で表が混ざる。

### 案 B: 実行時だけで置き換える（Lua `create_scene` と Rust 検索）

トランスパイル結果は変えず、`SCENE.create_scene`（登録）と `SearchContext::search_scene`（検索）で置き換える。

- 対象: `scene.lua`、`finalize.rs`／`search/`、設定の受け渡し。
- ✅ キャッシュの問題が生じない（生成 `.lua` は別名表に依存しない）。
- ❌ `create_scene` が受け取るのは照合用の名前に揃えた後の基本名で、書いた名前ではない。完全一致の判定が「照合用の名前どうし」に変わる（OQ-4 に影響）。
- ❌ ソースマップの突合キー（`G:会話#k`）と実行時の登録名（`OnTalk_N`）がずれ、デバッグが壊れる。突合側にも置き換えを入れる必要があり、変更点が散る。
- ❌ トランスパイル時の単語登録のモジュール名（`transpiler.rs`）とも食い違う（実行時の単語は Lua 側で登録されるため実害は限定的だが、二重の真実が残る）。

### 案 C: 共通の別名表の型＋トランスパイル時の宣言・実行時の検索（ハイブリッド）

`pasta_core` に「グローバルシーン名の別名表」の小さな型（完全一致の置き換え 1 段・連鎖の検出）を置き、`sanitize_name` は変えずにその前段として使う。

- 宣言側: `LuaTranspiler`（`TranspilerConfig` か `TranspileContext`）に表を持たせ、`process_global_scene` の先頭で `scene.name` を 1 回だけ置き換えた名前を作り、`register_global_scene`・`registered_name`・`generate_global_scene`（基本名と突合キー）へ同じ値を渡す。
- 検索側: `SearchContext` に表を持たせ、`search_scene` のグローバル検索の分岐（`global_scene_name = None`）でだけ置き換える。表は `finalize_scene_impl` → `search::register` へ渡す（Lua の app data か、ローダーから runtime へ渡す経路を新設）。
- 設定: `PastaConfig` に別名表の読み込み（既定値・空の表・型の検証）を追加。既定値はローダー層で決め、`LuaTranspiler::default()`・`SearchContext::new` の既定は空の表にする（ライブラリ直叩きのテストへの波及を抑える）。
- キャッシュ: `.cache_version` に別名表の指紋を含める、または pasta.toml の別名表が前回と違えばキャッシュを全破棄する。
- ✅ 宣言・突合キー・単語のモジュール名・実行時の登録名が 1 つの置き換え結果に揃い、デバッグは追加変更なしで動く見込み。
- ✅ 書いた名前で完全一致を判定できる（OQ-4 の推奨と整合）。
- ✅ 単語・アクター・ローカルシーンの経路は `sanitize_name` のままで、Req 4.7 を構造的に守れる。
- ❌ 表を渡す経路が 2 本（トランスパイラ・検索）とキャッシュの判定で計 3 か所に増える。どれか 1 つを忘れると宣言と検索が食い違う → 「既定・定義・空」×「宣言・検索」の結合テストで守る。
- ❌ 既定値の層の選び方で、既存テストへの波及が変わる（4 章のリスク）。

## 4. 工数とリスク

- **工数: M（3〜7 日）**。中核の置き換えは小さいが、設定・トランスパイラ・検索・キャッシュの 4 経路への配線、約 290 行の既存テスト（とくに `PastaLoader` 経由の fixture）の見直し、マニュアル 10 章前後の改稿と references 再生成、E2E の追加が重なる。
- **リスク: Medium**。既存パターン（設定セクション、`SearchContext`、fixture ゴーストの E2E）に乗れるが、(1) キャッシュの有効性判定の変更、(2) 宣言と検索の片側だけに表が効く不整合、(3) 既定の別名で既存ゴースト・既存テストの「会話」の意味が変わる互換性の 3 点に注意が要る。後続 2 spec（`scene-attribute-store`・`call-attribute-filter`）が `search/`・`scene_table.rs`・`pasta_core` の登録を rebase で追従する前提も、変更面を小さく保つ動機になる。

## 5. 設計フェーズへの推奨

- **推奨: 案 C**。`sanitize_name` は変えず、グローバルシーン名の経路にだけ前段の置き換えを差す。宣言側はトランスパイル時に 1 回だけ置き換えて全利用箇所へ同じ値を渡し、検索側は `search_scene` のグローバル分岐だけで置き換える。
- 既定値（「会話 → OnTalk」）はローダー層で決め、ライブラリ層の既定は空にする案を第一候補として検討する。これで `pasta_core` の単体テストとトランスパイラ直叩きのテスト（`sample.pasta` の生成比較を含む）は変わらず、影響は `PastaLoader` 経由の fixture に限られる。
- キャッシュの有効性に別名表を含めることを、Boundary Commitments に明記する。

### Research Needed（設計で詰める）

- **R-1 キャッシュ無効化の方式**: `.cache_version` に別名表の指紋を混ぜる／別ファイルに前回の表を記録して比較する／pasta.toml の更新時刻を全 `.pasta` の判定に加える、のどれにするか。部分的な再トランスパイルで足りるか（`＊会話` を含むファイルだけ）、全破棄が安全か。
- **R-2 実行時へ表を渡す経路**: `finalize_scene_impl` は `lua` しか受け取らない。Lua の app data に置くか、`PastaLuaRuntime` の構築時に `@pasta_search` 登録へ渡すか。SHIORI の reload で表が読み直されることも確認する。
- **R-3 デバッグの突合の確認**: `＊会話` と `＊OnTalk` が同じファイル・別ファイルに混在する場合に、`scene_join.rs` のファイル内順位の突合が正しいことをテストで確かめる。
- **R-4 ログに両方の名前を出す方法（OQ-6）**: 置き換えは Rust の `search_scene` で起き、警告は Lua の `ACT_IMPL.call` が出す。置き換えが起きたことを Lua 側へ返すか、Rust 側で debug/info ログを出すか。`failure-output-unification` の一本化と衝突しない形にする。
- **R-5 expr モードと `GLOBAL`**: `＠会話（）`（関数呼び出し）がグローバルシーン検索の段で OnTalk のシーンに解決されることを仕様として認めるか、マニュアルに書くか。L4 の `GLOBAL["会話"]` は置き換えないこと（完全一致の段）を確認する。
- **R-6 `@pasta_config` への見え方**: 別名表を書かなかったとき `@pasta_config.scene.alias` に既定を補うか（`[ghost]` は補完している）。Lua から有効な表を読む需要があるか。
- **R-7 既存の独自 `[scene]` セクションとの衝突**: 作者が `[scene]` を独自用途に使っている場合の扱い（`[scene.alias]` 以外のキーは無視して `@pasta_config` に残す、で足りるか）。

## 6. 未決事項（要件ディスカッション用）

| ID | 要件 | 論点 | 選択肢 | 推奨 | 理由 |
| -- | ---- | ---- | ------ | ---- | ---- |
| OQ-1 | 2.1 | pasta.toml のセクション名と形 | (a) `[scene.alias]` に `"会話" = "OnTalk"` (b) `[scene_alias]` (c) `[alias.scene]` | (a) | brief の推奨。将来シーン関連の設定を `[scene]` の下に足せる。独自 `[scene]` との衝突は R-7 で扱う |
| OQ-2 | 2.5 | 型が合わない値・空文字列 | (a) `[loader]` と同じく読み込みエラー (b) `[talk]` と同じく表全体を書かなかった扱い（既定の 1 件が効く）・エラーも警告も無し (c) 悪い行だけ無視して警告 | (a) | 別名表はどのシーンがどの名前で登録されるかを決め、`[loader]` と同じく辞書の意味に効く。(b) だと作者の他の行まで黙って消え、既定が効いて気づけない |
| OQ-3 | 3.4〜3.5 | 連鎖の形の扱い | (a) 1 段だけ・黙って続行 (b) 1 段だけ・警告をログ (c) 設定エラー | (b) | brief の Approach が「1 段だけ」と決めている。連鎖は作者の誤解の兆候なので知らせるが、止めるほどではない |
| OQ-4 | 3.2 | 完全一致を何どうしで比べるか | (a) 書いた名前とキーの文字列 (b) 照合用の名前に揃えた後どうし | (a) | 「完全一致」の字義どおりで予測しやすい。brief の「置き換えてから既存のサニタイズ」の順序とも合う。案 C なら実装上も自然 |
| OQ-5 | 4.7 | ローカルシーン名を対象外にするか | (a) 対象外（グローバル名だけ） (b) ローカルにも適用 | (a) | ローカルの検索にも置き換えを入れると `・会話` を `＞会話` で呼べなくなる。仮想イベントはグローバルシーンだけを探す |
| OQ-6 | 8.1〜8.3 | 失敗表記・ログ・登録名に出す名前 | (a) 失敗表記は書いた名前、ログは両方、登録名は置き換え後 (b) すべて書いた名前 (c) すべて置き換え後 | (a) | 失敗表記は作者が辞書の該当行を探す手がかりなので書いた名前。登録名はシーンの同一性そのもので、`OnTalk` の候補と同じ集まりに入るため置き換え後が一貫する。ログは両方あれば追える（実現方法は R-4） |
| OQ-7 | 9.6 | マニュアルの `＊会話` 例題 21 か所の整理 | (a) 照合の説明で使う `会話・朝` 系だけ別の例題名へ付け替え、ランダムトークとして読める `＊会話` の例は残す (b) すべて別の例題名へ付け替え (c) すべて別名を明示する例へ書き換え | (a) | `＊会話` の例の多くは「ひと続きのトーク」で、OnTalk の意味で読んでも食い違わない。紛らわしいのは「`会話・朝` は別名の対象外」と並ぶ照合の説明なので、そこだけ付け替えれば改稿量が小さい |
| OQ-8 | 2.6 | 有効な別名表を読み込み時にログへ出すか | (a) info で 1 回出す (b) 出さない (c) 既定と違うときだけ出す | (a) | `＊会話` がなぜ OnTalk になったか／ならなかったかをログで確かめられる。1 回だけなので負荷は無い |
